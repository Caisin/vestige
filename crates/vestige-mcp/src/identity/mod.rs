//! Optional KX identity boundary. Every authenticated space has its own data runtime.
mod config;
mod http;
mod mcp;
mod policy;
mod provider;
mod store;
#[cfg(test)]
mod tests;
use crate::{cognitive::CognitiveEngine, dashboard::state::AppState};
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
pub use config::Config;
pub use http::router;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use vestige_core::Storage;

pub type Result<T> = std::result::Result<T, Error>;
#[derive(Debug)]
pub struct Error(pub StatusCode, pub String);
impl Error {
    pub fn bad(s: impl Into<String>) -> Self {
        Self(StatusCode::BAD_REQUEST, s.into())
    }
    pub fn unauthorized(s: impl Into<String>) -> Self {
        Self(StatusCode::UNAUTHORIZED, s.into())
    }
    pub fn forbidden() -> Self {
        Self(
            StatusCode::FORBIDDEN,
            "当前身份没有该空间或操作的权限".into(),
        )
    }
    pub fn internal() -> Self {
        Self(StatusCode::INTERNAL_SERVER_ERROR, "身份服务处理失败".into())
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.1)
    }
}
impl std::error::Error for Error {}
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (self.0, Json(json!({"error":self.1}))).into_response()
    }
}
macro_rules! internal_from {($($ty:ty),*)=>{$(impl From<$ty> for Error{fn from(_: $ty)->Self{Self::internal()}})*};}
internal_from!(
    rusqlite::Error,
    serde_json::Error,
    std::io::Error,
    anyhow::Error,
    url::ParseError
);
impl From<reqwest::Error> for Error {
    fn from(_: reqwest::Error) -> Self {
        Self(
            StatusCode::BAD_GATEWAY,
            "暂时无法连接身份服务，请重试".into(),
        )
    }
}
pub fn now() -> i64 {
    chrono::Utc::now().timestamp()
}
pub fn random() -> String {
    format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    )
}
pub fn hash(s: &str) -> String {
    format!("{:x}", Sha256::digest(s.as_bytes()))
}
#[derive(Clone, Debug)]
pub struct Grant {
    pub credential: String,
    pub user: String,
    pub workspace: String,
    pub role: String,
    pub kind: String,
}
#[derive(Clone)]
pub struct Access {
    pub hub: Arc<Hub>,
    pub grant: Grant,
    pub token: String,
}
pub struct Hub {
    pub config: Config,
    pub provider: provider::Provider,
    pub(super) db: Mutex<rusqlite::Connection>,
    pub(super) key: [u8; 32],
    pub(super) root: PathBuf,
    pub(super) legacy: AppState,
    runtimes: tokio::sync::Mutex<HashMap<String, AppState>>,
    sessions: tokio::sync::Mutex<HashMap<String, mcp::Session>>,
    pub(super) login_limit: Mutex<Vec<std::time::Instant>>,
}
impl Hub {
    pub fn new(config: Config, legacy: AppState) -> Result<Arc<Self>> {
        let root = legacy.storage.data_dir().join("identity");
        std::fs::create_dir_all(&root)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700))?;
        }
        let key_path = root.join("session.key");
        if !key_path.exists() {
            use std::io::Write;
            let mut opts = std::fs::OpenOptions::new();
            opts.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                opts.mode(0o600);
            }
            let mut f = opts.open(&key_path)?;
            f.write_all(uuid::Uuid::new_v4().as_bytes())?;
            f.write_all(uuid::Uuid::new_v4().as_bytes())?;
            f.sync_all()?;
        }
        let bytes = std::fs::read(key_path)?;
        let key: [u8; 32] = bytes.try_into().map_err(|_| Error::internal())?;
        let mut db = rusqlite::Connection::open(root.join("identity.db"))?;
        db.busy_timeout(std::time::Duration::from_secs(5))?;
        db.execute_batch(include_str!("schema.sql"))?;
        // Rebinding (or disabling) legacy ownership revokes the previous account.
        // Browser sessions return to that account's private space; Agent grants are revoked.
        {
            let tx = db.transaction()?;
            let params = rusqlite::params![config.kx_api, config.legacy_owner_subject];
            tx.execute("UPDATE credentials SET workspace=(SELECT id FROM workspaces WHERE kind='personal' AND owner=credentials.user_id) WHERE workspace='legacy' AND kind='session' AND user_id NOT IN (SELECT id FROM users WHERE issuer=?1 AND subject=?2)", params)?;
            tx.execute("DELETE FROM credentials WHERE workspace='legacy' AND kind='agent' AND user_id NOT IN (SELECT id FROM users WHERE issuer=?1 AND subject=?2)", params)?;
            tx.execute("DELETE FROM members WHERE workspace='legacy' AND user_id NOT IN (SELECT id FROM users WHERE issuer=?1 AND subject=?2)", params)?;
            tx.commit()?;
        }

        Ok(Arc::new(Self {
            provider: provider::Provider::new(config.clone())?,
            config,
            db: Mutex::new(db),
            key,
            root,
            legacy,
            runtimes: Default::default(),
            sessions: Default::default(),
            login_limit: Default::default(),
        }))
    }
    pub async fn runtime(self: &Arc<Self>, g: &Grant) -> Result<AppState> {
        self.check(g)?;
        if g.workspace == "legacy" {
            return Ok(self.legacy.clone());
        }
        // Retire expired MCP owners before checking which cached runtimes are idle.
        self.sessions
            .lock()
            .await
            .retain(|_, session| !session.expired());
        let mut runtimes = self.runtimes.lock().await;
        if let Some(state) = runtimes.get(&g.workspace) {
            let state = state.clone();
            drop(runtimes);
            self.share_reranker(&state).await;
            return Ok(state);
        }
        // Bound live cognitive/index instances; evict only runtimes with no active requests or sessions.
        if runtimes.len() >= 32 {
            let id = runtimes
                .iter()
                .find(|(_, s)| {
                    s.background
                        .as_ref()
                        .is_some_and(|jobs| Arc::strong_count(jobs) == 1)
                })
                .map(|(id, _)| id.clone());
            if let Some(id) = id {
                runtimes.remove(&id);
            } else {
                return Err(Error(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "活跃空间已达上限，请稍后重试".into(),
                ));
            }
        }
        let path = self.root.join("spaces").join(&g.workspace);
        let source = self.legacy.storage.clone();
        let mut state = tokio::task::spawn_blocking(move || -> Result<AppState> {
            let path = Storage::db_path_for_data_dir(path).map_err(|_| Error::internal())?;
            #[allow(unused_mut)]
            let mut storage = Storage::new(Some(path)).map_err(|_| Error::internal())?;
            #[cfg(all(feature = "embeddings", feature = "vector-search"))]
            storage
                .inherit_workspace_embedding_runtime(&source)
                .map_err(|_| Error::internal())?;
            let _ = source;
            let storage = Arc::new(storage);
            let mut cognitive = CognitiveEngine::new();
            cognitive.hydrate(&storage);
            Ok(AppState::new(
                storage,
                Some(Arc::new(tokio::sync::Mutex::new(cognitive))),
            ))
        })
        .await
        .map_err(|_| Error::internal())??;
        if let Some(cognitive) = &state.cognitive {
            state.background = Some(Arc::new(crate::autopilot::spawn_managed(
                cognitive.clone(),
                state.storage.clone(),
                state.event_tx.clone(),
            )));
        }
        runtimes.insert(g.workspace.clone(), state.clone());
        drop(runtimes);
        self.share_reranker(&state).await;
        Ok(state)
    }
    async fn share_reranker(&self, state: &AppState) {
        let _ = state;
        #[cfg(feature = "vector-search")]
        if let (Some(source), Some(target)) = (&self.legacy.cognitive, &state.cognitive) {
            let source = source.lock().await;
            target
                .lock()
                .await
                .reranker
                .share_model_from(&source.reranker);
        }
    }
    pub fn limit_login(&self) -> Result<()> {
        let mut times = self.login_limit.lock().map_err(|_| Error::internal())?;
        times.retain(|t| t.elapsed().as_secs() < 60);
        if times.len() >= 30 {
            return Err(Error(
                StatusCode::TOO_MANY_REQUESTS,
                "登录请求过于频繁，请一分钟后重试".into(),
            ));
        }
        times.push(std::time::Instant::now());
        Ok(())
    }
}
pub fn enabled(path: &Path) -> bool {
    std::env::var_os("VESTIGE_AUTH_CONFIG").is_some() || path.join("auth.json").exists()
}
