use serde::{Deserialize, Serialize};
use std::path::Path;
use url::Url;

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub public_origin: String,
    pub kx_api: String,
    #[serde(default)]
    pub local_mcp: bool,
    #[serde(default = "default_app")]
    pub kx_app_id: String,
    #[serde(default)]
    pub legacy_owner_subject: Option<String>,
    #[serde(default)]
    pub oauth: Option<OAuth>,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OAuth {
    pub name: String,
    pub issuer: String,
    pub authorize_url: String,
    pub token_url: String,
    pub userinfo_url: String,
    pub client_id: String,
    pub client_secret: Option<String>,
    #[serde(default)]
    pub scope: String,
}
fn default_app() -> String {
    "admin".into()
}
impl Config {
    pub fn load(data: &Path) -> anyhow::Result<Option<Self>> {
        let explicit = std::env::var_os("VESTIGE_AUTH_CONFIG");
        let path = explicit
            .clone()
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| data.join("auth.json"));
        if !path.exists() && explicit.is_none() {
            return Ok(None);
        }
        let mut c: Self = serde_json::from_slice(&std::fs::read(path)?)?;
        c.public_origin = c.public_origin.trim_end_matches('/').into();
        c.kx_api = c.kx_api.trim_end_matches('/').into();
        safe_url(&c.public_origin)?;
        let u = Url::parse(&c.public_origin)?;
        anyhow::ensure!(
            u.path() == "/" && u.query().is_none(),
            "public_origin must be an origin"
        );
        safe_url(&c.kx_api)?;
        if let Some(o) = &c.oauth {
            for u in [&o.issuer, &o.authorize_url, &o.token_url, &o.userinfo_url] {
                safe_url(u)?;
            }
            anyhow::ensure!(!o.client_id.is_empty(), "OAuth client_id required");
        }
        Ok(Some(c))
    }
    pub fn secure(&self) -> bool {
        self.public_origin.starts_with("https://")
    }
}
pub fn safe_url(s: &str) -> anyhow::Result<Url> {
    let u = Url::parse(s)?;
    anyhow::ensure!(
        u.username().is_empty() && u.password().is_none() && u.fragment().is_none(),
        "URL credentials/fragments forbidden"
    );
    anyhow::ensure!(
        u.scheme() == "https"
            || (u.scheme() == "http"
                && matches!(u.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"))),
        "HTTPS required outside loopback"
    );
    Ok(u)
}
