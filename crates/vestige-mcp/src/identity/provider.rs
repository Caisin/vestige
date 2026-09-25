//! KX transport compatibility; HTTPS provides confidentiality, KxEd is only framing.
use super::{Config, Error, Result};
use serde_json::{Value, json};
use std::time::Duration;

#[derive(Clone)]
pub struct Provider {
    pub client: reqwest::Client,
    pub config: Config,
}
#[derive(Clone)]
pub struct Identity {
    pub issuer: String,
    pub subject: String,
    pub name: String,
}
impl Provider {
    pub fn new(config: Config) -> Result<Self> {
        Ok(Self {
            config,
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(15))
                .redirect(reqwest::redirect::Policy::none())
                .build()?,
        })
    }
    pub async fn kx(&self, path: &str, body: Option<Value>, token: Option<&str>) -> Result<Value> {
        let url = format!("{}{path}", self.config.kx_api);
        let mut r = if body.is_some() {
            self.client.post(url)
        } else {
            self.client.get(url)
        };
        r = r.header("security", "true");
        if let Some(token) = token {
            r = r.bearer_auth(token);
        }
        if let Some(body) = body {
            r = r
                .header("content-type", "application/json")
                .body(encode(&serde_json::to_vec(&body)?));
        }
        let resp = r.send().await?;
        if !resp.status().is_success() {
            return Err(Error::unauthorized("KX 登录或身份验证失败"));
        }
        let bytes = limited(resp).await?;
        let v: Value = match serde_json::from_slice(&bytes) {
            Ok(v) => v,
            Err(_) => serde_json::from_slice(&decode(&bytes)?)?,
        };
        if v["code"] != 200 {
            return Err(Error::unauthorized(
                "KX 未批准登录，请检查账号、密码或授权票据",
            ));
        }
        Ok(v["result"].clone())
    }
    pub async fn verify(&self, kind: &str, token: &str) -> Result<Identity> {
        if kind == "kx" {
            let v = self.kx("/auth/user/user_info", None, Some(token)).await?;
            if v["enabled"] != true {
                return Err(Error::unauthorized("账号已停用"));
            }
            let subject = match &v["id"] {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                _ => return Err(Error::unauthorized("用户身份无效")),
            };
            if subject.is_empty() {
                return Err(Error::unauthorized("用户身份无效"));
            }
            Ok(Identity {
                issuer: self.config.kx_api.clone(),
                subject,
                name: v["name"].as_str().unwrap_or("KX 用户").to_string(),
            })
        } else if kind == "oauth" {
            let o = self
                .config
                .oauth
                .as_ref()
                .ok_or_else(|| Error::bad("未配置授权服务"))?;
            let response = self
                .client
                .get(&o.userinfo_url)
                .bearer_auth(token)
                .send()
                .await?;
            if !response.status().is_success() {
                return Err(Error::unauthorized("授权服务未批准身份"));
            }
            let v: Value = serde_json::from_slice(&limited(response).await?)?;
            let sub = v["sub"]
                .as_str()
                .filter(|s| !s.is_empty())
                .ok_or_else(|| Error::unauthorized("授权服务缺少 sub"))?;
            Ok(Identity {
                issuer: o.issuer.clone(),
                subject: sub.into(),
                name: v["name"].as_str().unwrap_or(sub).into(),
            })
        } else {
            Err(Error::bad("未知身份提供方"))
        }
    }
    pub async fn password(&self, username: &str, password: &str) -> Result<Value> {
        self.kx("/auth/user/access_token", Some(json!({"app_id":self.config.kx_app_id,"login_type":"user_name","user_name":username,"password":password,"platform":"vestige"})), None).await
    }
}
pub async fn limited(mut resp: reqwest::Response) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    while let Some(chunk) = resp.chunk().await? {
        if out.len() + chunk.len() > 1024 * 1024 {
            return Err(Error::bad("身份服务响应过大"));
        }
        out.extend_from_slice(&chunk);
    }
    Ok(out)
}
pub fn encode(data: &[u8]) -> Vec<u8> {
    let salt = uuid::Uuid::new_v4();
    let salt = salt.as_bytes();
    let mut out = vec![salt.len() as u8];
    out.extend_from_slice(salt);
    out.extend(
        data.iter()
            .enumerate()
            .map(|(i, b)| 255u8.wrapping_sub(b.wrapping_add(salt[i % salt.len()]))),
    );
    out
}
pub fn decode(data: &[u8]) -> Result<Vec<u8>> {
    let n = data.first().copied().unwrap_or(0) as usize;
    if n == 0 || data.len() <= n {
        return Err(Error::bad("KX 传输响应无效"));
    }
    Ok(data[n + 1..]
        .iter()
        .enumerate()
        .map(|(i, b)| 255u8.wrapping_sub(*b).wrapping_sub(data[1 + i % n]))
        .collect())
}
