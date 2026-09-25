//! Same-origin desktop/browser adapter over the exact MCP writer domain.
use super::state::AppState;
use axum::{
    Json, Router,
    body::Bytes,
    extract::{DefaultBodyLimit, Path, Query, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::post,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::{Arc, OnceLock};
use subtle::ConstantTimeEq;
use tokio::sync::Semaphore;
use vestige_core::writer::{WriterError, import::MAX_UPLOAD_BYTES};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/writer/upload",
            post(upload).layer(DefaultBodyLimit::max(MAX_UPLOAD_BYTES)),
        )
        .route("/api/writer/download", post(download))
        .route(
            "/api/writer/{tool}",
            post(action).layer(DefaultBodyLimit::max(256 * 1024)),
        )
}
fn failure(error: WriterError) -> Response {
    let status = match error.code.as_str() {
        "NOT_FOUND" => StatusCode::NOT_FOUND,
        "VERSION_CONFLICT" | "TASK_UNAVAILABLE" | "LEASE_INVALID" | "LEASE_EXPIRED"
        | "SOURCE_IN_USE" | "ROLE_IN_USE" => StatusCode::CONFLICT,
        "STORAGE_ERROR" => StatusCode::INTERNAL_SERVER_ERROR,
        "BUSY" => StatusCode::TOO_MANY_REQUESTS,
        "FORBIDDEN" => StatusCode::FORBIDDEN,
        _ => StatusCode::BAD_REQUEST,
    };
    (
        status,
        Json(json!({"code":error.code,"error":error.message})),
    )
        .into_response()
}
fn allowed(headers: &HeaderMap) -> Result<(), WriterError> {
    let host = headers
        .get(header::HOST)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    let authority = host
        .parse::<axum::http::uri::Authority>()
        .map_err(|_| WriterError::new("FORBIDDEN", "无效的本地服务地址"))?;
    if !["127.0.0.1", "localhost", "[::1]"].contains(&authority.host()) {
        return Err(WriterError::new("FORBIDDEN", "编剧接口只接受本机来源"));
    }
    if let Some(origin) = headers.get(header::ORIGIN) {
        if origin.to_str().ok() == Some(format!("http://{host}").as_str()) {
            return Ok(());
        }
        return Err(WriterError::new("FORBIDDEN", "拒绝跨来源请求"));
    }
    let bearer = headers
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));
    let expected = crate::protocol::auth::get_or_create_auth_token()
        .map_err(|_| WriterError::new("FORBIDDEN", "无法读取服务凭据"))?;
    if bearer.is_some_and(|token| bool::from(token.as_bytes().ct_eq(expected.as_bytes()))) {
        Ok(())
    } else {
        Err(WriterError::new("FORBIDDEN", "需要同源界面或有效服务凭据"))
    }
}
async fn action(
    State(state): State<AppState>,
    Path(tool): Path<String>,
    headers: HeaderMap,
    Json(args): Json<Value>,
) -> Response {
    if let Err(error) = state
        .access
        .as_ref()
        .map_or_else(|| allowed(&headers), |_| Ok(()))
    {
        return failure(error);
    }
    if !crate::tools::writer::TOOLS
        .iter()
        .any(|(name, _, _, _)| *name == tool)
    {
        return failure(WriterError::new("NOT_FOUND", "未知编剧接口"));
    }
    let storage = state.storage;
    match tokio::task::spawn_blocking(move || storage.writer_execute(&tool, args)).await {
        Ok(Ok(value)) => Json(value).into_response(),
        Ok(Err(error)) => failure(error),
        Err(_) => failure(WriterError::new("STORAGE_ERROR", "编剧请求处理失败")),
    }
}
#[derive(Deserialize)]
struct UploadParams {
    role_id: String,
    filename: String,
    title: String,
    author: Option<String>,
    episode: Option<String>,
}
async fn upload(
    State(state): State<AppState>,
    Query(query): Query<UploadParams>,
    headers: HeaderMap,
    bytes: Bytes,
) -> Response {
    if let Err(error) = state
        .access
        .as_ref()
        .map_or_else(|| allowed(&headers), |_| Ok(()))
    {
        return failure(error);
    }
    static PARSERS: OnceLock<Arc<Semaphore>> = OnceLock::new();
    let Ok(permit) = PARSERS
        .get_or_init(|| Arc::new(Semaphore::new(1)))
        .clone()
        .try_acquire_owned()
    else {
        return failure(WriterError::new("BUSY", "正在解析其他作品，请稍后重试"));
    };
    match tokio::task::spawn_blocking(move || {
        let _permit = permit;
        state
            .storage
            .writer_upload_with_metadata(&query.role_id, &query.filename, &query.title, &bytes, &json!({"author":query.author.unwrap_or_default(),"episode":query.episode.unwrap_or_default()}))
    })
    .await
    {
        Ok(Ok(value)) => Json(value).into_response(),
        Ok(Err(error)) => failure(error),
        Err(_) => failure(WriterError::new("STORAGE_ERROR", "作品解析失败")),
    }
}
#[derive(Deserialize)]
struct DownloadParams {
    role_id: String,
    source_id: String,
}
async fn download(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(query): Json<DownloadParams>,
) -> Response {
    if let Err(error) = state
        .access
        .as_ref()
        .map_or_else(|| allowed(&headers), |_| Ok(()))
    {
        return failure(error);
    }
    match tokio::task::spawn_blocking(move || {
        state
            .storage
            .writer_source_bytes(&query.role_id, &query.source_id)
    })
    .await
    {
        Ok(Ok((_, bytes))) => (
            [
                (header::CONTENT_TYPE, "application/octet-stream"),
                (
                    header::CONTENT_DISPOSITION,
                    "attachment; filename=screenplay-source",
                ),
            ],
            bytes,
        )
            .into_response(),
        Ok(Err(error)) => failure(error),
        Err(_) => failure(WriterError::new("STORAGE_ERROR", "下载失败")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn browser_origin_is_checked_not_just_cors() {
        let mut h = HeaderMap::new();
        h.insert(header::HOST, "127.0.0.1:3927".parse().unwrap());
        h.insert(header::ORIGIN, "http://127.0.0.1:3927".parse().unwrap());
        assert!(allowed(&h).is_ok());
        h.insert(header::ORIGIN, "https://untrusted.invalid".parse().unwrap());
        assert!(allowed(&h).is_err());
        h.insert(header::HOST, "attacker.invalid:3927".parse().unwrap());
        h.insert(
            header::ORIGIN,
            "http://attacker.invalid:3927".parse().unwrap(),
        );
        assert!(allowed(&h).is_err());
    }
}
