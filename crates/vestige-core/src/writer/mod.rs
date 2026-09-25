//! Local screenplay sources, versioned writer personas and external-Agent work.
pub mod import;
mod store;

use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::Storage;

pub const SCHEMA: &str = include_str!("schema.sql");
pub const API_VERSION: u32 = 1;
pub type WriterResult<T> = Result<T, WriterError>;

#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
#[error("{code}: {message}")]
pub struct WriterError {
    pub code: String,
    pub message: String,
}

impl WriterError {
    pub fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.to_string(),
            message: message.into(),
        }
    }
}
impl From<rusqlite::Error> for WriterError {
    fn from(error: rusqlite::Error) -> Self {
        tracing::warn!(%error, "Writer database operation failed");
        Self::new("STORAGE_ERROR", "编剧数据操作失败，请检查服务日志")
    }
}
impl From<serde_json::Error> for WriterError {
    fn from(_: serde_json::Error) -> Self {
        Self::new("INVALID_JSON", "数据不是有效的 JSON")
    }
}

impl Storage {
    /// One shared domain boundary for HTTP and MCP. Every mutation commits as
    /// one SQLite transaction; Agent completion cannot leave half a version.
    pub fn writer_execute(&self, tool: &str, arguments: Value) -> WriterResult<Value> {
        if serde_json::to_vec(&arguments)?.len() > 240 * 1024 {
            return Err(WriterError::new(
                "INPUT_LIMIT",
                "请求超过 240 KiB，请分页读取或拆分正文",
            ));
        }
        if !arguments.is_object() {
            return Err(WriterError::new("INVALID_ARGUMENT", "参数必须是对象"));
        }
        let action = arguments["action"]
            .as_str()
            .ok_or_else(|| WriterError::new("INVALID_ARGUMENT", "缺少 action"))?;
        let fields = allowed_fields(tool, action)
            .ok_or_else(|| WriterError::new("UNKNOWN_ACTION", "未知的编剧工具或操作"))?;
        for key in arguments.as_object().expect("object checked").keys() {
            if key != "action" && !fields.contains(&key.as_str()) {
                return Err(WriterError::new(
                    "UNSUPPORTED_FIELD",
                    format!("{tool}/{action} 不支持参数 {key}"),
                ));
            }
            let value = &arguments[key];
            let valid = match key.as_str() {
                "version" | "expected_version" | "revision" | "expected_revision" | "draft_revision" | "role_version" | "offset" | "limit" | "budget_chars" => value.as_i64().is_some_and(|number| number >= 0),
                "input" | "result" | "canon" | "metadata" => value.is_object(),
                "findings" => value.is_array(),
                "confirm" => value.is_boolean(),
                _ => value.is_string(),
            };
            if !valid { return Err(WriterError::new("INVALID_ARGUMENT", format!("{key} 的类型不符合工具契约"))); }
        }
        let mut connection = self
            .writer
            .lock()
            .map_err(|_| WriterError::new("STORAGE_ERROR", "数据库繁忙"))?;
        let transaction =
            connection.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let result = store::execute(&transaction, tool, &arguments)?;
        transaction.commit()?;
        Ok(result)
    }

    pub fn writer_upload(
        &self,
        role_id: &str,
        filename: &str,
        title: &str,
        bytes: &[u8],
    ) -> WriterResult<Value> {
        self.writer_upload_with_metadata(role_id, filename, title, bytes, &serde_json::json!({}))
    }

    pub fn writer_upload_with_metadata(
        &self,
        role_id: &str,
        filename: &str,
        title: &str,
        bytes: &[u8],
        metadata: &Value,
    ) -> WriterResult<Value> {
        if filename.len() > 300 || title.len() > 600 || title.trim().is_empty() {
            return Err(WriterError::new(
                "INVALID_ARGUMENT",
                "请填写有效的作品名和文件名",
            ));
        }
        // Parsing runs outside the DB lock; callers must use a bounded worker.
        let mut parsed = import::parse(filename, bytes)?;
        import::attribution(&mut parsed, metadata)?;
        let hash = format!("{:x}", Sha256::digest(bytes));
        let mut connection = self
            .writer
            .lock()
            .map_err(|_| WriterError::new("STORAGE_ERROR", "数据库繁忙"))?;
        let transaction =
            connection.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let result = store::insert_source(
            &transaction,
            role_id,
            filename,
            title,
            bytes,
            &hash,
            &parsed,
        )?;
        transaction.commit()?;
        Ok(result)
    }

    /// A binary download is never embedded into an MCP response.
    pub fn writer_source_bytes(
        &self,
        role_id: &str,
        source_id: &str,
    ) -> WriterResult<(String, Vec<u8>)> {
        let connection = self
            .reader
            .lock()
            .map_err(|_| WriterError::new("STORAGE_ERROR", "数据库繁忙"))?;
        connection
            .query_row(
                "SELECT filename,raw_bytes FROM writer_sources WHERE id=?1 AND role_id=?2",
                params![source_id, role_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|_| WriterError::new("NOT_FOUND", "作品不存在或不属于该角色"))
    }
}

/// One contract registry shared by validation and MCP schema generation.
pub fn allowed_fields(tool: &str, action: &str) -> Option<&'static [&'static str]> {
    Some(match (tool, action) {
        ("writer_role", "list") => &["limit", "offset"],
        ("writer_role", "create") => &["name", "description"],
        ("writer_role", "get") => &["role_id"],
        ("writer_role", "versions") => &["role_id", "limit", "offset"],
        ("writer_role", "publish" | "rollback") => &["role_id", "version", "expected_version"],
        ("writer_role", "delete") => &["role_id", "confirm"],
        ("writer_source", "list") => &["role_id", "limit", "offset"],
        ("writer_source", "get") => &["role_id", "source_id", "segment_id", "limit", "offset"],
        ("writer_source", "attach_text") => &["role_id", "title", "content", "metadata"],
        ("writer_source", "delete") => &["role_id", "source_id", "confirm"],
        ("writer_task", "list") => &["role_id", "status", "limit", "offset"],
        ("writer_task", "create") => &["role_id", "project_id", "kind", "input"],
        ("writer_task", "claim") => &["task_id", "agent_id"],
        ("writer_task", "get" | "cancel") => &["task_id"],
        ("writer_task", "heartbeat") => &["task_id", "lease_token"],
        ("writer_task", "complete") => &["task_id", "lease_token", "result"],
        ("writer_task", "fail") => &["task_id", "lease_token", "error"],
        ("writer_context", "prepare") => &["role_id", "project_id", "query", "budget_chars"],
        ("writer_project", "list") => &["role_id", "limit", "offset"],
        ("writer_project", "get") => &["project_id"],
        ("writer_project", "create") => &[
            "role_id",
            "role_version",
            "name",
            "format",
            "brief",
            "canon",
        ],
        ("writer_project", "update") => &[
            "project_id",
            "expected_revision",
            "role_id",
            "role_version",
            "name",
            "format",
            "brief",
            "canon",
        ],
        ("writer_project", "delete") => &["project_id", "confirm"],
        ("writer_draft", "list") => &["project_id", "limit", "offset"],
        ("writer_draft", "get") => &["draft_id", "revision"],
        ("writer_draft", "save") => &[
            "project_id",
            "role_version",
            "kind",
            "title",
            "content",
            "draft_id",
            "expected_revision",
        ],
        ("writer_draft", "delete") => &["draft_id", "confirm"],
        ("writer_review", "list") => &["draft_id", "limit", "offset"],
        ("writer_review", "get") => &["review_id"],
        ("writer_review", "save") => &["draft_id", "draft_revision", "summary", "findings"],
        _ => return None,
    })
}

fn timestamp() -> String {
    chrono::Utc::now().to_rfc3339()
}
fn identifier() -> String {
    uuid::Uuid::new_v4().to_string()
}
fn digest(value: &Value) -> WriterResult<String> {
    Ok(format!("{:x}", Sha256::digest(serde_json::to_vec(value)?)))
}

fn audit(connection: &Connection, action: &str, entity: &str) -> WriterResult<()> {
    connection.execute(
        "INSERT INTO writer_audit(id,action,entity_id,created_at) VALUES(?1,?2,?3,?4)",
        params![identifier(), action, entity, timestamp()],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests;
