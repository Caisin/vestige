use super::{Access, Error, Grant, Hub, Result, hash, now, random, store::field};
use axum::{
    Json, Router,
    body::{Body, to_bytes},
    extract::{DefaultBodyLimit, Query, State},
    http::{HeaderMap, Request, StatusCode, header},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use rusqlite::{OptionalExtension, params};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{collections::HashMap, sync::Arc};
use tower::ServiceExt;

pub fn router(hub: Arc<Hub>) -> Router {
    Router::new()
        .route("/api/auth/config", get(configuration))
        .route("/api/auth/login", post(password))
        .route("/api/auth/start", get(start))
        .route("/api/auth/callback", get(callback))
        .route("/api/auth/me", get(me))
        .route("/api/auth/account", post(account))
        .route(
            "/mcp",
            post(super::mcp::post)
                .delete(super::mcp::delete)
                .layer(DefaultBodyLimit::max(256 * 1024)),
        )
        .fallback(dispatch)
        .layer(DefaultBodyLimit::max(16 * 1024))
        .layer(tower::limit::ConcurrencyLimitLayer::new(50))
        .layer(axum::middleware::from_fn(no_cache))
        .with_state(hub)
}
async fn no_cache(req: Request<Body>, next: axum::middleware::Next) -> Response {
    let mut r = next.run(req).await;
    r.headers_mut()
        .insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
    r.headers_mut()
        .insert("referrer-policy", "no-referrer".parse().unwrap());
    r.headers_mut()
        .insert("x-content-type-options", "nosniff".parse().unwrap());
    r.headers_mut()
        .insert("x-frame-options", "DENY".parse().unwrap());
    r
}
pub fn cookie(h: &HeaderMap, name: &str) -> Option<String> {
    h.get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .find_map(|v| {
            let (k, v) = v.trim().split_once('=')?;
            (k == name).then(|| v.into())
        })
}
fn set_cookie(hub: &Hub, name: &str, value: &str, age: i64) -> String {
    format!(
        "{name}={value}; Path=/; HttpOnly; SameSite=Lax; Max-Age={age}{}",
        if hub.config.secure() { "; Secure" } else { "" }
    )
}
pub fn csrf(hub: &Hub, h: &HeaderMap) -> Result<()> {
    if h.get(header::ORIGIN).and_then(|v| v.to_str().ok())
        != Some(hub.config.public_origin.as_str())
    {
        return Err(Error::forbidden());
    }
    Ok(())
}
pub async fn browser(hub: &Hub, h: &HeaderMap) -> Result<(Grant, String)> {
    let token = cookie(h, "vestige_session").ok_or_else(|| Error::unauthorized("请先登录"))?;
    let g = hub.authenticate(&token, "session").await?;
    Ok((g, token))
}
async fn configuration(State(h): State<Arc<Hub>>) -> Json<Value> {
    let apps = h.provider.kx("/auth/dt/apps", None, None).await;
    Json(
        json!({"enabled":true,"password":true,"dingtalk_apps":apps.as_ref().ok(),"provider_error":apps.is_err(),"oauth_name":h.config.oauth.as_ref().map(|o|o.name.clone())}),
    )
}
async fn me(State(h): State<Arc<Hub>>, headers: HeaderMap) -> Result<Json<Value>> {
    let (g, _) = browser(&h, &headers).await?;
    h.bind_local(&g)?;
    Ok(Json(h.me(&g)?))
}
async fn account(
    State(h): State<Arc<Hub>>,
    headers: HeaderMap,
    Json(a): Json<Value>,
) -> Result<Response> {
    csrf(&h, &headers)?;
    let (g, _) = browser(&h, &headers).await?;
    h.check(&g)?;
    let mut r = Json(h.account_action(&g, &a)?).into_response();
    if a["action"] == "logout" {
        h.unbind_local(&g.credential)?;
    } else if a["action"] == "switch" {
        // Keep the same authenticated browser session bound while its active
        // workspace changes atomically in the account action above.
        h.bind_local(&g)?;
    }
    if a["action"] == "logout" {
        r.headers_mut().insert(
            header::SET_COOKIE,
            set_cookie(&h, "vestige_session", "", 0).parse().unwrap(),
        );
    }
    Ok(r)
}
async fn finish(h: &Hub, provider: &str, v: Value) -> Result<Response> {
    let token = field(&v, "access_token", 32768)?;
    let identity = h.provider.verify(provider, token).await?;
    let expiry = if provider == "kx" {
        v["exp_at"]
            .as_i64()
            .or_else(|| v["exp_at"].as_str().and_then(|s| s.parse().ok()))
            .unwrap_or(now() + 3600)
    } else {
        now() + v["expires_in"].as_i64().unwrap_or(3600).clamp(1, 8 * 3600)
    };
    let session = h.new_session(identity, provider, token, expiry)?;
    let mut r = Redirect::to("/dashboard/writer").into_response();
    r.headers_mut().append(
        header::SET_COOKIE,
        set_cookie(
            h,
            "vestige_session",
            &session,
            (expiry - now()).min(8 * 3600),
        )
        .parse()
        .unwrap(),
    );
    r.headers_mut().append(
        header::SET_COOKIE,
        set_cookie(h, "vestige_login", "", 0).parse().unwrap(),
    );
    Ok(r)
}
async fn password(
    State(h): State<Arc<Hub>>,
    headers: HeaderMap,
    Json(v): Json<Value>,
) -> Result<Response> {
    csrf(&h, &headers)?;
    h.limit_login()?;
    let result = h
        .provider
        .password(field(&v, "username", 200)?, field(&v, "password", 1000)?)
        .await?;
    let mut r = finish(&h, "kx", result).await?;
    *r.status_mut() = StatusCode::OK;
    r.headers_mut().remove(header::LOCATION);
    *r.body_mut() = Body::from("{\"ok\":true}");
    r.headers_mut()
        .insert(header::CONTENT_TYPE, "application/json".parse().unwrap());
    Ok(r)
}
async fn start(
    State(h): State<Arc<Hub>>,
    Query(q): Query<HashMap<String, String>>,
) -> Result<Response> {
    h.limit_login()?;
    let provider = q.get("provider").map(String::as_str).unwrap_or("kx");
    let state = random();
    let binding = random();
    let verifier = random();
    let callback = format!("{}/api/auth/callback", h.config.public_origin);
    let mut url = if provider == "kx" {
        let app = q.get("app").map(String::as_str).unwrap_or("");
        if !app
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            || app.len() > 100
        {
            return Err(Error::bad("无效钉钉应用"));
        }
        let suffix = if app.is_empty() {
            String::new()
        } else {
            format!("/{app}")
        };
        let mut u = url::Url::parse(&format!("{}/auth/dt/login{suffix}", h.config.kx_api))?;
        u.query_pairs_mut()
            .append_pair("redirect_url", &format!("{callback}?state={state}"));
        u
    } else if provider == "oauth" {
        let o = h
            .config
            .oauth
            .as_ref()
            .ok_or_else(|| Error::bad("未配置授权服务"))?;
        let mut u = url::Url::parse(&o.authorize_url)?;
        u.query_pairs_mut()
            .append_pair("response_type", "code")
            .append_pair("client_id", &o.client_id)
            .append_pair("redirect_uri", &callback)
            .append_pair("scope", &o.scope)
            .append_pair("state", &state)
            .append_pair("code_challenge_method", "S256")
            .append_pair(
                "code_challenge",
                &URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes())),
            );
        u
    } else {
        return Err(Error::bad("未知登录方式"));
    };
    url.set_fragment(None);
    h.db(|c| {
        c.execute("DELETE FROM flows WHERE expires<=?1", [now()])?;
        c.execute(
            "INSERT INTO flows VALUES(?1,?2,?3,?4,?5)",
            params![
                hash(&state),
                hash(&binding),
                provider,
                verifier,
                now() + 300
            ],
        )?;
        Ok(())
    })?;
    let mut r = Redirect::to(url.as_str()).into_response();
    r.headers_mut().insert(
        header::SET_COOKIE,
        set_cookie(&h, "vestige_login", &binding, 300)
            .parse()
            .unwrap(),
    );
    Ok(r)
}
async fn callback(
    State(h): State<Arc<Hub>>,
    headers: HeaderMap,
    Query(q): Query<HashMap<String, String>>,
) -> Response {
    match callback_inner(&h, &headers, q).await {
        Ok(r) => r,
        Err(_) => Redirect::to("/dashboard/login?error=callback").into_response(),
    }
}
async fn callback_inner(
    h: &Hub,
    headers: &HeaderMap,
    q: HashMap<String, String>,
) -> Result<Response> {
    let state = q
        .get("state")
        .filter(|s| s.len() <= 200)
        .ok_or_else(|| Error::bad("缺少登录 state"))?;
    let binding = cookie(headers, "vestige_login").ok_or_else(|| Error::bad("登录浏览器不匹配"))?;
    let(provider,verifier)=h.db(|c|{
        c.query_row("DELETE FROM flows WHERE state=?1 AND binding=?2 AND expires>?3 RETURNING provider,verifier",params![hash(state),hash(&binding),now()],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?))).optional()?.ok_or_else(||Error::bad("登录请求已过期或已使用"))
    })?;
    let result = if provider == "kx" {
        let code = q
            .get("exchange_code")
            .filter(|s| s.len() <= 1024)
            .ok_or_else(|| Error::bad("缺少一次性登录票据"))?;
        h.provider
            .kx(
                "/auth/dt/exchange",
                Some(json!({"exchange_code":code})),
                None,
            )
            .await?
    } else {
        let o = h
            .config
            .oauth
            .as_ref()
            .ok_or_else(|| Error::bad("未配置授权服务"))?;
        let code = q
            .get("code")
            .filter(|s| s.len() <= 4096)
            .ok_or_else(|| Error::bad("缺少授权码"))?;
        let callback = format!("{}/api/auth/callback", h.config.public_origin);
        let mut form = vec![
            ("grant_type", "authorization_code"),
            ("code", code.as_str()),
            ("client_id", &o.client_id),
            ("redirect_uri", &callback),
            ("code_verifier", &verifier),
        ];
        if let Some(secret) = &o.client_secret {
            form.push(("client_secret", secret));
        }
        let r = h
            .provider
            .client
            .post(&o.token_url)
            .form(&form)
            .send()
            .await?;
        if !r.status().is_success() {
            return Err(Error::unauthorized("授权码交换失败"));
        }
        serde_json::from_slice(&super::provider::limited(r).await?)?
    };
    finish(h, &provider, result).await
}
async fn dispatch(State(h): State<Arc<Hub>>, mut req: Request<Body>) -> Result<Response> {
    let path = req.uri().path().to_string();
    if path == "/dashboard" || path.starts_with("/dashboard/") || path == "/" || path == "/graph" {
        let (router, _) = crate::dashboard::build_workspace_router(h.legacy.clone(), 3927);
        return Ok(router.oneshot(req).await.unwrap());
    }
    if path == "/api/health" {
        // Public readiness contains no tenant counts or model metadata.
        return Ok(Json(json!({"status":"healthy","authentication":true,"writerApiVersion":1,"version":env!("CARGO_PKG_VERSION")})).into_response());
    }
    let (g, token) = browser(&h, req.headers()).await?;
    if req.method() != axum::http::Method::GET {
        csrf(&h, req.headers())?;
    }
    if path == "/ws" {
        csrf(&h, req.headers())?;
    } else if let Some(tool) = path.strip_prefix("/api/writer/") {
        if tool == "upload" {
            if g.role == "viewer" {
                return Err(Error::forbidden());
            }
        } else if tool != "download" {
            let (parts, body) = req.into_parts();
            let bytes = to_bytes(body, 256 * 1024)
                .await
                .map_err(|_| Error::bad("请求过大"))?;
            let args: Value = serde_json::from_slice(&bytes)?;
            super::policy::writer(&g, tool, &args)?;
            req = Request::from_parts(parts, Body::from(bytes));
        }
    } else {
        super::policy::http(&g, req.method().as_str(), &path)?;
    }
    h.check(&g)?;
    let mut state = h.runtime(&g).await?;
    state.access = Some(Access {
        hub: h.clone(),
        grant: g.clone(),
        token,
    });
    let (router, _) = crate::dashboard::build_workspace_router(state, 3927);
    let response = router.oneshot(req).await.unwrap();
    if response.status().is_success() {
        h.audit(&g, &path)?;
    }
    Ok(response)
}
