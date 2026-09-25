use super::{Error, Hub, Result, http::csrf};
use crate::{protocol::types::JsonRpcRequest, server::McpServer};
use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use serde_json::{Value, json};
use std::{sync::Arc, time::Instant};
#[derive(Clone)]
pub(super) struct Session {
    _background: Option<Arc<crate::autopilot::AutopilotTasks>>,
    credential: String,
    user: String,
    workspace: String,
    protocol: String,
    last: Instant,
    server: Arc<tokio::sync::Mutex<McpServer>>,
}
impl Session {
    pub(super) fn expired(&self) -> bool {
        self.last.elapsed().as_secs() >= 1800
    }
}
async fn identity(h: &Hub, headers: &HeaderMap) -> Result<super::Grant> {
    if headers.contains_key("origin") {
        csrf(h, headers)?;
    }
    let token = headers
        .get("authorization")
        .and_then(|s| s.to_str().ok())
        .and_then(|s| s.split_once(' '))
        .filter(|(s, _)| s.eq_ignore_ascii_case("bearer"))
        .map(|(_, s)| s)
        .ok_or_else(|| Error::unauthorized("请使用个人空间 Agent 凭据"))?;
    h.authenticate(token, "agent").await
}
pub async fn post(
    State(h): State<Arc<Hub>>,
    headers: HeaderMap,
    Json(request): Json<JsonRpcRequest>,
) -> Result<Response> {
    let g = identity(&h, &headers).await?;
    if let Err((status, msg)) = crate::protocol::http::validate_accept(&headers) {
        return Err(Error(status, msg.into()));
    }
    if !matches!(
        request.method.as_str(),
        "initialize"
            | "ping"
            | "tools/list"
            | "tools/call"
            | "notifications/initialized"
            | "logging/setLevel"
    ) {
        return Err(Error::forbidden());
    }
    if request.method == "tools/call" {
        let p = request
            .params
            .as_ref()
            .ok_or_else(|| Error::bad("缺少调用参数"))?;
        super::policy::tool(&g, p["name"].as_str().unwrap_or(""), &p["arguments"])?;
    }
    let initializing = request.method == "initialize";
    let (id, session) = if initializing {
        let runtime = h.runtime(&g).await?;
        let session = Session {
            _background: runtime.background.clone(),
            credential: g.credential.clone(),
            user: g.user.clone(),
            workspace: g.workspace.clone(),
            protocol: crate::protocol::types::MCP_VERSION.into(),
            last: Instant::now(),
            server: Arc::new(tokio::sync::Mutex::new(McpServer::new_with_events(
                runtime.storage,
                runtime.cognitive.ok_or_else(Error::internal)?,
                runtime.event_tx,
            ))),
        };
        let mut all = h.sessions.lock().await;
        all.retain(|_, s| s.last.elapsed().as_secs() < 1800);
        if all.len() >= 100 || all.values().filter(|s| s.user == g.user).count() >= 10 {
            return Err(Error(
                StatusCode::TOO_MANY_REQUESTS,
                "MCP 会话数量已达上限".into(),
            ));
        }
        let id = uuid::Uuid::new_v4().to_string();
        all.insert(id.clone(), session.clone());
        (id, session)
    } else {
        let id = headers
            .get("mcp-session-id")
            .and_then(|s| s.to_str().ok())
            .ok_or_else(|| Error::bad("缺少 MCP 会话"))?
            .to_string();
        let mut all = h.sessions.lock().await;
        let session = all
            .get_mut(&id)
            .ok_or_else(|| Error::unauthorized("MCP 会话已失效"))?;
        if session.credential != g.credential
            || session.user != g.user
            || session.workspace != g.workspace
            || session.last.elapsed().as_secs() >= 1800
        {
            return Err(Error::forbidden());
        }
        if let Err((status, msg)) =
            crate::protocol::http::validate_protocol_version(&headers, &session.protocol)
        {
            return Err(Error(status, msg.into()));
        }
        session.last = Instant::now();
        (id, session.clone())
    };
    let tool_list = request.method == "tools/list";
    let method = request.method.clone();
    let mut server = session.server.lock().await;
    h.check(&g)?;
    let response = server.handle_request(request).await;
    let Some(mut response) = response else {
        if initializing {
            h.sessions.lock().await.remove(&id);
        }
        return Ok(StatusCode::ACCEPTED.into_response());
    };
    if initializing {
        let Some(protocol) = response
            .result
            .as_ref()
            .and_then(|v| v["protocolVersion"].as_str())
            .filter(|_| response.error.is_none())
        else {
            h.sessions.lock().await.remove(&id);
            return Ok(Json(response).into_response());
        };
        if let Some(s) = h.sessions.lock().await.get_mut(&id) {
            s.protocol = protocol.into();
        }
    }
    if tool_list
        && let Some(Value::Array(tools)) = response.result.as_mut().and_then(|v| v.get_mut("tools"))
    {
        tools.retain(|t| {
            let name = t["name"].as_str().unwrap_or("");
            name.starts_with("writer_")
                || super::policy::tool(&g, name, &json!({"action":"get"})).is_ok()
        });
    }
    h.audit(&g, &format!("mcp:{method}"))?;
    let mut response = Json(response).into_response();
    response
        .headers_mut()
        .insert("mcp-session-id", id.parse().unwrap());
    let protocol = h
        .sessions
        .lock()
        .await
        .get(&id)
        .map(|s| s.protocol.clone())
        .unwrap_or(session.protocol);
    response
        .headers_mut()
        .insert("mcp-protocol-version", protocol.parse().unwrap());
    Ok(response)
}
pub async fn delete(State(h): State<Arc<Hub>>, headers: HeaderMap) -> Result<Response> {
    let g = identity(&h, &headers).await?;
    let id = headers
        .get("mcp-session-id")
        .and_then(|s| s.to_str().ok())
        .ok_or_else(|| Error::bad("缺少 MCP 会话"))?;
    let mut all = h.sessions.lock().await;
    let s = all.get(id).ok_or_else(|| Error::bad("会话不存在"))?;
    if s.credential != g.credential || s.workspace != g.workspace {
        return Err(Error::forbidden());
    }
    if let Err((status, msg)) =
        crate::protocol::http::validate_protocol_version(&headers, &s.protocol)
    {
        return Err(Error(status, msg.into()));
    }
    all.remove(id);
    Ok(StatusCode::NO_CONTENT.into_response())
}
