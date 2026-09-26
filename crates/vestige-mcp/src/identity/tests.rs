use super::*;
use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, header},
    routing::{get, post},
};
use serde_json::{Value, json};
use tower::ServiceExt;
const ORIGIN: &str = "http://127.0.0.1:3931";
struct Fixture {
    _dir: tempfile::TempDir,
    hub: Arc<Hub>,
    app: Router,
    provider_task: tokio::task::JoinHandle<()>,
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.provider_task.abort();
    }
}
impl Fixture {
    async fn new() -> Self {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let upstream = format!("http://{}", listener.local_addr().unwrap());
        let mock=Router::new()
            .route("/auth/user/access_token",post(|bytes:axum::body::Bytes|async move{
                let v:Value=serde_json::from_slice(&provider::decode(&bytes).unwrap()).unwrap();
                if v["password"]!="test-only-password"{return (StatusCode::UNAUTHORIZED,Vec::new());}
                (StatusCode::OK,provider::encode(&serde_json::to_vec(&json!({"code":200,"result":{"access_token":format!("test-only:{}",v["user_name"].as_str().unwrap()),"exp_at":now()+3600}})).unwrap()))
            }))
            .route("/auth/user/user_info",get(|h:axum::http::HeaderMap|async move{
                let sub=h.get(header::AUTHORIZATION).and_then(|v|v.to_str().ok()).and_then(|v|v.strip_prefix("Bearer test-only:"));
                let Some(sub)=sub else{return (StatusCode::UNAUTHORIZED,Vec::new());};
                (StatusCode::OK,provider::encode(&serde_json::to_vec(&json!({"code":200,"result":{"id":sub,"name":sub,"enabled":sub!="disabled"}})).unwrap()))
            }))
            .route("/auth/dt/apps",get(||async{provider::encode(b"{\"code\":200,\"result\":[{\"app_key\":\"test\",\"app_name\":\"Test\",\"is_default\":true}]}")}))
            .route("/auth/dt/exchange",post(|bytes:axum::body::Bytes|async move{
                let v:Value=serde_json::from_slice(&provider::decode(&bytes).unwrap()).unwrap();
                if v["exchange_code"]!="test-only-code"{return (StatusCode::UNAUTHORIZED,Vec::new());}
                (StatusCode::OK,provider::encode(&serde_json::to_vec(&json!({"code":200,"result":{"access_token":"test-only:alice","exp_at":now()+3600}})).unwrap()))
            }));
        let mock = mock
            .route(
                "/oauth/token",
                post(|body: axum::body::Bytes| async move {
                    let form: url::form_urlencoded::Parse<'_> = url::form_urlencoded::parse(&body);
                    let form: std::collections::HashMap<_, _> = form.into_owned().collect();
                    assert_eq!(form.get("grant_type").unwrap(), "authorization_code");
                    assert_eq!(form.get("code").unwrap(), "test-only-oauth-code");
                    assert_eq!(form.get("code_verifier").unwrap().len(), 64);
                    assert_eq!(
                        form.get("redirect_uri").unwrap(),
                        &format!("{ORIGIN}/api/auth/callback")
                    );
                    Json(json!({"access_token":"test-only:alice","expires_in":3600}))
                }),
            )
            .route(
                "/oauth/userinfo",
                get(|headers: axum::http::HeaderMap| async move {
                    assert_eq!(headers["authorization"], "Bearer test-only:alice");
                    Json(json!({"sub":"alice","name":"OAuth Alice"}))
                }),
            );
        let task = tokio::spawn(async move { axum::serve(listener, mock).await.unwrap() });
        let dir = tempfile::tempdir().unwrap();
        let storage = Arc::new(Storage::new(Some(dir.path().join("vestige.db"))).unwrap());
        let config = Config {
            public_origin: ORIGIN.into(),
            kx_api: upstream,
            local_mcp: false,
            kx_app_id: "admin".into(),
            legacy_owner_subject: None,
            oauth: None,
        };
        let hub = Hub::new(
            config,
            AppState::new(
                storage,
                Some(Arc::new(tokio::sync::Mutex::new(CognitiveEngine::new()))),
            ),
        )
        .unwrap();
        let app = router(hub.clone());
        Self {
            _dir: dir,
            hub,
            app,
            provider_task: task,
        }
    }
    fn login(&self, name: &str) -> String {
        self.hub
            .new_session(
                provider::Identity {
                    issuer: self.hub.config.kx_api.clone(),
                    subject: name.into(),
                    name: name.into(),
                },
                "kx",
                &format!("test-only:{name}"),
                now() + 3600,
            )
            .unwrap()
    }
    async fn grant(&self, token: &str) -> Grant {
        self.hub.authenticate(token, "session").await.unwrap()
    }
    async fn req(
        &self,
        method: &str,
        path: &str,
        token: Option<&str>,
        value: Value,
    ) -> (StatusCode, axum::http::HeaderMap, Value) {
        let mut req = Request::builder()
            .method(method)
            .uri(path)
            .header("origin", ORIGIN)
            .header("content-type", "application/json");
        if let Some(token) = token {
            req = req.header("cookie", format!("vestige_session={token}"));
        }
        let r = self
            .app
            .clone()
            .oneshot(req.body(Body::from(value.to_string())).unwrap())
            .await
            .unwrap();
        let status = r.status();
        let headers = r.headers().clone();
        let bytes = to_bytes(r.into_body(), 1024 * 1024).await.unwrap();
        (
            status,
            headers,
            serde_json::from_slice(&bytes).unwrap_or(Value::Null),
        )
    }
    async fn rpc(
        &self,
        token: &str,
        session: Option<&str>,
        v: Value,
    ) -> (StatusCode, axum::http::HeaderMap, Value) {
        let mut req = Request::builder()
            .method("POST")
            .uri("/mcp")
            .header("authorization", format!("Bearer {token}"))
            .header("accept", "application/json, text/event-stream")
            .header("content-type", "application/json")
            .header("mcp-protocol-version", crate::protocol::types::MCP_VERSION);
        if let Some(s) = session {
            req = req.header("mcp-session-id", s);
        }
        let r = self
            .app
            .clone()
            .oneshot(req.body(Body::from(v.to_string())).unwrap())
            .await
            .unwrap();
        let status = r.status();
        let headers = r.headers().clone();
        let b = to_bytes(r.into_body(), 1024 * 1024).await.unwrap();
        (
            status,
            headers,
            serde_json::from_slice(&b).unwrap_or(Value::Null),
        )
    }
}
#[test]
fn kx_wire_roundtrip() {
    let v = "{\"name\":\"编剧\"}".as_bytes();
    assert_eq!(provider::decode(&provider::encode(v)).unwrap(), v);
    assert!(provider::decode(&[32, 1]).is_err());
}
#[tokio::test]
async fn password_identity_and_session_security() {
    let f = Fixture::new().await;
    let (s, h, _) = f
        .req(
            "POST",
            "/api/auth/login",
            None,
            json!({"username":"alice","password":"test-only-password"}),
        )
        .await;
    assert_eq!(s, 200);
    let cookie = h
        .get_all(header::SET_COOKIE)
        .iter()
        .find(|v| v.to_str().unwrap().starts_with("vestige_session="))
        .unwrap()
        .to_str()
        .unwrap();
    assert!(cookie.contains("HttpOnly"));
    assert!(cookie.contains("SameSite=Lax"));
    let token = cookie
        .split(';')
        .next()
        .unwrap()
        .strip_prefix("vestige_session=")
        .unwrap();
    let g = f.grant(token).await;
    assert_eq!(f.hub.me(&g).unwrap()["user"]["subject"], "alice");
    f.hub
        .db(|c| {
            let (hash, upstream): (String, Vec<u8>) = c.query_row(
                "SELECT hash,upstream FROM credentials WHERE id=?1",
                [&g.credential],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )?;
            assert_ne!(hash, token);
            assert!(!upstream.windows(15).any(|v| v == b"test-only:alice"));
            Ok(())
        })
        .unwrap();
    assert_eq!(
        f.req(
            "POST",
            "/api/auth/login",
            None,
            json!({"username":"disabled","password":"test-only-password"})
        )
        .await
        .0,
        401
    );
    assert_eq!(
        f.req(
            "POST",
            "/api/auth/login",
            None,
            json!({"username":"alice","password":"incorrect"})
        )
        .await
        .0,
        401
    );
    let request = Request::builder()
        .method("POST")
        .uri("/api/auth/account")
        .header("cookie", format!("vestige_session={token}"))
        .header("origin", "https://evil.invalid")
        .header("content-type", "application/json")
        .body(Body::from(
            "{\"action\":\"create_workspace\",\"name\":\"bad\"}",
        ))
        .unwrap();
    assert_eq!(f.app.clone().oneshot(request).await.unwrap().status(), 403);
    f.hub
        .account_action(&g, &json!({"action":"logout"}))
        .unwrap();
    assert!(f.hub.authenticate(token, "session").await.is_err());
}
#[tokio::test]
async fn private_spaces_isolate_all_writer_and_memory_data() {
    let f = Fixture::new().await;
    let a = f.login("alice");
    let b = f.login("bob");
    let ga = f.grant(&a).await;
    let gb = f.grant(&b).await;
    assert_ne!(ga.workspace, gb.workspace);
    let private = f
        .hub
        .runtime(&ga)
        .await
        .unwrap()
        .storage
        .ingest(vestige_core::IngestInput {
            content: "Alice unique-private-memory-canary".into(),
            node_type: "fact".into(),
            source: None,
            sentiment_score: 0.0,
            sentiment_magnitude: 0.0,
            tags: vec![],
            valid_from: None,
            valid_until: None,
            validity_inferred: false,
            source_envelope: None,
        })
        .unwrap();
    assert_eq!(
        f.req(
            "GET",
            &format!("/api/memories/{}", private.id),
            Some(&b),
            Value::Null
        )
        .await
        .0,
        404
    );
    for path in [
        "/api/memories",
        "/api/search?q=unique-private-memory-canary",
        "/api/graph",
        "/api/timeline",
        "/api/receipts",
        "/api/traces",
    ] {
        let (s, _, v) = f.req("GET", path, Some(&b), Value::Null).await;
        assert_eq!(s, 200, "{path}: {v}");
        assert!(
            !v.to_string().contains("Alice unique-private-memory-canary"),
            "{path}: {v}"
        );
        assert!(!v.to_string().contains(&private.id), "{path}: {v}");
    }

    let ra = f.hub.runtime(&ga).await.unwrap();
    let rb = f.hub.runtime(&gb).await.unwrap();
    assert_ne!(ra.storage.data_dir(), rb.storage.data_dir());
    let role = ra
        .storage
        .writer_execute("writer_role", json!({"action":"create","name":"私人编剧"}))
        .unwrap()["role"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    let source = ra
        .storage
        .writer_upload(
            &role,
            "secret.txt",
            "私有作品",
            b"private screenplay material",
        )
        .unwrap();
    assert_eq!(
        f.req(
            "POST",
            "/api/writer/writer_role",
            Some(&b),
            json!({"action":"get","role_id":role})
        )
        .await
        .0,
        404
    );
    assert_eq!(
        f.req(
            "POST",
            "/api/writer/writer_role",
            Some(&b),
            json!({"action":"list"})
        )
        .await
        .2["roles"],
        json!([])
    );
    assert_eq!(
        f.req(
            "POST",
            "/api/writer/download",
            Some(&b),
            json!({"role_id":role,"source_id":source["source"]["id"]})
        )
        .await
        .0,
        404
    );
    assert_eq!(
        f.req(
            "POST",
            "/api/writer/writer_task",
            Some(&b),
            json!({"action":"list","role_id":role})
        )
        .await
        .2["tasks"],
        json!([])
    );
    for path in [
        "/api/memories",
        "/api/search?q=private",
        "/api/graph",
        "/api/traces",
        "/api/receipts",
        "/ws",
    ] {
        assert_eq!(f.req("GET", path, None, Value::Null).await.0, 401, "{path}");
    }
    assert_eq!(
        f.req("GET", "/api/sanhedrin/latest", Some(&a), Value::Null)
            .await
            .0,
        403
    );
    assert_eq!(
        f.hub.me(&ga).unwrap()["workspaces"]
            .as_array()
            .unwrap()
            .len(),
        1,
        "legacy must not be claimed by first login"
    );
    assert!(
        f.hub
            .account_action(&ga, &json!({"action":"switch","workspace":gb.workspace}))
            .is_err()
    );
}
#[tokio::test]
async fn shared_role_contributions_permissions_and_revocation() {
    let f = Fixture::new().await;
    let a = f.login("alice");
    let b = f.login("bob");
    let c = f.login("carol");
    let mut ga = f.grant(&a).await;
    let id = f
        .hub
        .account_action(&ga, &json!({"action":"create_workspace","name":"共同编剧"}))
        .unwrap()["workspace"]
        .as_str()
        .unwrap()
        .to_string();
    f.hub
        .account_action(&ga, &json!({"action":"switch","workspace":id}))
        .unwrap();
    ga = f.grant(&a).await;
    for (token, role) in [(&b, "editor"), (&c, "viewer")] {
        let invite = f
            .hub
            .account_action(&ga, &json!({"action":"invite","role":role}))
            .unwrap()["invitation"]
            .clone();
        let g = f.grant(token).await;
        f.hub
            .account_action(&g, &json!({"action":"join","invitation":invite}))
            .unwrap();
        assert!(
            f.hub
                .account_action(&g, &json!({"action":"join","invitation":invite}))
                .is_err()
        );
        f.hub
            .account_action(&g, &json!({"action":"switch","workspace":id}))
            .unwrap();
    }
    let gb = f.grant(&b).await;
    let role = f
        .req(
            "POST",
            "/api/writer/writer_role",
            Some(&a),
            json!({"action":"create","name":"共同培养"}),
        )
        .await
        .2["role"]["id"]
        .clone();
    for (token, text) in [(&a, "甲上传的原创作品"), (&b, "乙补充的编剧知识")] {
        let (s, _, v) = f
            .req(
                "POST",
                "/api/writer/writer_source",
                Some(token),
                json!({"action":"attach_text","role_id":role,"title":"贡献","content":text}),
            )
            .await;
        assert_eq!(s, 200, "{v}");
    }
    let sources = f
        .req(
            "POST",
            "/api/writer/writer_source",
            Some(&c),
            json!({"action":"list","role_id":role}),
        )
        .await;
    assert_eq!(sources.0, 200);
    assert_eq!(sources.2["sources"].as_array().unwrap().len(), 2);
    assert_eq!(
        f.req(
            "POST",
            "/api/writer/writer_role",
            Some(&c),
            json!({"action":"create","name":"拒绝写入"})
        )
        .await
        .0,
        403
    );
    for action in ["publish", "rollback", "delete"] {
        assert_eq!(f.req("POST","/api/writer/writer_role",Some(&b),json!({"action":action,"role_id":role,"version":0,"expected_version":0,"confirm":true})).await.0,403);
    }
    let agent = f
        .hub
        .account_action(
            &gb,
            &json!({"action":"create_token","label":"editor agent","role":"editor"}),
        )
        .unwrap()["token"]
        .as_str()
        .unwrap()
        .to_string();
    f.hub
        .account_action(&ga, &json!({"action":"remove_member","user_id":gb.user}))
        .unwrap();
    assert!(f.hub.check(&gb).is_err());
    assert!(f.hub.authenticate(&agent, "agent").await.is_err());
    assert!(
        f.hub
            .account_action(&ga, &json!({"action":"remove_member","user_id":ga.user}))
            .is_err()
    );
}
#[tokio::test]
async fn mcp_sessions_are_bound_to_credential_user_and_space() {
    let f = Fixture::new().await;
    let a = f.login("alice");
    let b = f.login("bob");
    let ga = f.grant(&a).await;
    let gb = f.grant(&b).await;
    let agent = |g: &Grant| {
        f.hub
            .account_action(
                g,
                &json!({"action":"create_token","label":"test","role":"editor"}),
            )
            .unwrap()
    };
    let ta = agent(&ga);
    let tb = agent(&gb);
    let a = ta["token"].as_str().unwrap();
    let b = tb["token"].as_str().unwrap();
    let init = json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":crate::protocol::types::MCP_VERSION,"capabilities":{},"clientInfo":{"name":"test","version":"1"}}});
    let (s, h, v) = f.rpc(a, None, init).await;
    assert_eq!(s, 200, "{v}");
    let session = h["mcp-session-id"].to_str().unwrap();
    let call = json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"writer_role","arguments":{"action":"create","name":"Agent 编剧"}}});
    assert_eq!(f.rpc(b, Some(session), call.clone()).await.0, 403);
    let (s, _, v) = f.rpc(a, Some(session), call).await;
    assert_eq!(s, 200);
    assert!(v["result"]["isError"] != true, "{v}");
    let unsafe_call = json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"codebase","arguments":{"action":"index","path":"/"}}});
    assert_eq!(f.rpc(a, Some(session), unsafe_call).await.0, 403);
    f.hub
        .account_action(&ga, &json!({"action":"revoke_token","id":ta["id"]}))
        .unwrap();
    assert_eq!(
        f.rpc(
            a,
            Some(session),
            json!({"jsonrpc":"2.0","id":4,"method":"ping"})
        )
        .await
        .0,
        401
    );
}
#[tokio::test]
async fn callback_requires_binding_and_is_single_use() {
    let f = Fixture::new().await;
    let (s, h, _) = f
        .req(
            "GET",
            "/api/auth/start?provider=kx&app=test",
            None,
            Value::Null,
        )
        .await;
    assert_eq!(s, 303);
    let target = url::Url::parse(h["location"].to_str().unwrap()).unwrap();
    let callback = target
        .query_pairs()
        .find(|(k, _)| k == "redirect_url")
        .unwrap()
        .1
        .into_owned();
    let target = url::Url::parse(&callback).unwrap();
    let uri = format!(
        "{}?{}&exchange_code=test-only-code",
        target.path(),
        target.query().unwrap()
    );
    let binding = h["set-cookie"].to_str().unwrap().split(';').next().unwrap();
    let request = |cookie: &str| {
        Request::builder()
            .uri(&uri)
            .header("cookie", cookie)
            .body(Body::empty())
            .unwrap()
    };
    let wrong = f
        .app
        .clone()
        .oneshot(request("vestige_login=wrong"))
        .await
        .unwrap();
    assert_eq!(
        wrong.headers()["location"],
        "/dashboard/login?error=callback"
    );
    let ok = f.app.clone().oneshot(request(binding)).await.unwrap();
    assert_eq!(ok.headers()["location"], "/dashboard/writer");
    let replay = f.app.clone().oneshot(request(binding)).await.unwrap();
    assert_eq!(
        replay.headers()["location"],
        "/dashboard/login?error=callback"
    );
}
#[tokio::test]
async fn expired_invites_and_disabled_upstream_fail_closed() {
    let f = Fixture::new().await;
    let a = f.login("disabled");
    let g = f.grant(&a).await;
    f.hub
        .db(|c| {
            c.execute(
                "UPDATE credentials SET checked=0 WHERE id=?1",
                [&g.credential],
            )?;
            Ok(())
        })
        .unwrap();
    assert!(f.hub.authenticate(&a, "session").await.is_err());
    assert!(config::safe_url("http://example.com").is_err());
    assert!(config::safe_url("https://user:password@example.com").is_err());
    let encrypted = f.hub.encrypt("test-only-upstream-token").unwrap();
    let mut corrupt = encrypted.clone();
    *corrupt.last_mut().unwrap() ^= 1;
    assert!(f.hub.decrypt(&corrupt).is_err());
}

#[tokio::test]
async fn binary_upload_and_download_share_only_with_members() {
    let f = Fixture::new().await;
    let a = f.login("alice");
    let g = f.grant(&a).await;
    let role = f
        .hub
        .runtime(&g)
        .await
        .unwrap()
        .storage
        .writer_execute(
            "writer_role",
            json!({"action":"create","name":"upload-test"}),
        )
        .unwrap()["role"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    let req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/writer/upload?role_id={role}&filename=test.txt&title=test"
        ))
        .header("origin", ORIGIN)
        .header("cookie", format!("vestige_session={a}"))
        .header("content-type", "application/octet-stream")
        .body(Body::from("shared original screenplay"))
        .unwrap();
    let r = f.app.clone().oneshot(req).await.unwrap();
    assert_eq!(r.status(), 200);
    let v: Value =
        serde_json::from_slice(&to_bytes(r.into_body(), 1024 * 1024).await.unwrap()).unwrap();
    assert!(v["source"]["id"].is_string());
}

#[tokio::test]
async fn credentials_survive_restart_but_expired_invites_do_not_work() {
    let f = Fixture::new().await;
    let a = f.login("alice");
    let b = f.login("bob");
    let mut ga = f.grant(&a).await;
    let id = f
        .hub
        .account_action(&ga, &json!({"action":"create_workspace","name":"expiry"}))
        .unwrap()["workspace"]
        .clone();
    f.hub
        .account_action(&ga, &json!({"action":"switch","workspace":id}))
        .unwrap();
    ga = f.grant(&a).await;
    let invite = f
        .hub
        .account_action(&ga, &json!({"action":"invite","role":"editor"}))
        .unwrap()["invitation"]
        .clone();
    f.hub
        .db(|c| {
            c.execute("UPDATE invitations SET expires=0", [])?;
            Ok(())
        })
        .unwrap();
    assert!(
        f.hub
            .account_action(
                &f.grant(&b).await,
                &json!({"action":"join","invitation":invite})
            )
            .is_err()
    );
    let restarted = Hub::new(f.hub.config.clone(), f.hub.legacy.clone()).unwrap();
    assert_eq!(
        restarted
            .authenticate(&a, "session")
            .await
            .unwrap()
            .workspace,
        ga.workspace
    );
}

#[tokio::test]
async fn oauth_uses_pkce_and_provider_namespaced_subject() {
    use base64::Engine;
    let f = Fixture::new().await;
    let mut cfg = f.hub.config.clone();
    let upstream = cfg.kx_api.clone();
    cfg.oauth = Some(config::OAuth {
        name: "Test OAuth".into(),
        issuer: format!("{upstream}/oauth"),
        authorize_url: format!("{upstream}/oauth/authorize"),
        token_url: format!("{upstream}/oauth/token"),
        userinfo_url: format!("{upstream}/oauth/userinfo"),
        client_id: "test-only-client".into(),
        client_secret: None,
        scope: "profile".into(),
    });
    let hub = Hub::new(cfg, f.hub.legacy.clone()).unwrap();
    let app = router(hub.clone());
    let r = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/auth/start?provider=oauth")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let location = url::Url::parse(r.headers()["location"].to_str().unwrap()).unwrap();
    let q: std::collections::HashMap<_, _> = location.query_pairs().into_owned().collect();
    assert_eq!(q["code_challenge_method"], "S256");
    let verifier = hub
        .db(|c| {
            Ok(c.query_row(
                "SELECT verifier FROM flows WHERE state=?1",
                [hash(&q["state"])],
                |r| r.get::<_, String>(0),
            )?)
        })
        .unwrap();
    assert_eq!(
        q["code_challenge"],
        base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(Sha256::digest(verifier.as_bytes()))
    );
    let binding = r.headers()["set-cookie"]
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap();
    let r = app
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/api/auth/callback?state={}&code=test-only-oauth-code",
                    q["state"]
                ))
                .header("cookie", binding)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r.headers()["location"], "/dashboard/writer");
    let token = r
        .headers()
        .get_all("set-cookie")
        .iter()
        .find_map(|v| v.to_str().unwrap().strip_prefix("vestige_session="))
        .unwrap()
        .split(';')
        .next()
        .unwrap();
    let oauth_user = hub.authenticate(token, "session").await.unwrap();
    let kx_user = f.grant(&f.login("alice")).await;
    assert_ne!(oauth_user.user, kx_user.user);
    assert_ne!(oauth_user.workspace, kx_user.workspace);
}

#[tokio::test]
async fn websocket_is_scoped_and_closes_on_logout() {
    use futures_util::StreamExt;
    use tokio_tungstenite::tungstenite::client::IntoClientRequest;
    let f = Fixture::new().await;
    let a = f.login("alice");
    let b = f.login("bob");
    let ga = f.grant(&a).await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = f.app.clone();
    let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let mut req = format!("ws://{addr}/ws").into_client_request().unwrap();
    req.headers_mut().insert("origin", ORIGIN.parse().unwrap());
    req.headers_mut()
        .insert("cookie", format!("vestige_session={b}").parse().unwrap());
    let (mut socket, _) = tokio_tungstenite::connect_async(req).await.unwrap();
    let welcome = socket.next().await.unwrap().unwrap();
    assert!(welcome.to_text().unwrap().contains("Connected"));
    f.hub
        .runtime(&ga)
        .await
        .unwrap()
        .event_tx
        .send(crate::dashboard::events::VestigeEvent::Heartbeat {
            uptime_secs: 1,
            memory_count: 777,
            avg_retention: 1.0,
            suppressed_count: 0,
            timestamp: chrono::Utc::now(),
        })
        .ok();
    let message = socket.next().await.unwrap().unwrap();
    let heartbeat: Value = serde_json::from_str(message.to_text().unwrap()).unwrap();
    assert_eq!(heartbeat["data"]["memory_count"], 0);
    let gb = f.grant(&b).await;
    let revoked_runtime = f.hub.runtime(&gb).await.unwrap();
    f.hub
        .account_action(&gb, &json!({"action":"logout"}))
        .unwrap();
    revoked_runtime
        .event_tx
        .send(crate::dashboard::events::VestigeEvent::Heartbeat {
            uptime_secs: 1,
            memory_count: 888,
            avg_retention: 1.0,
            suppressed_count: 0,
            timestamp: chrono::Utc::now(),
        })
        .ok();
    tokio::time::timeout(std::time::Duration::from_secs(7), async {
        while let Some(Ok(message)) = socket.next().await {
            if let Ok(text) = message.to_text() {
                let value: Value = serde_json::from_str(text).unwrap();
                assert_ne!(value["data"]["memory_count"], 888);
            }
            if message.is_close() {
                break;
            }
        }
    })
    .await
    .expect("revoked WebSocket must close");
    task.abort();
}

#[tokio::test]
async fn local_loopback_mcp_uses_logged_in_session_without_bearer_token() {
    let f = Fixture::new().await;
    let mut config = f.hub.config.clone();
    config.local_mcp = true;
    let hub = Hub::new(config, f.hub.legacy.clone()).unwrap();
    let token = hub
        .new_session(
            provider::Identity {
                issuer: hub.config.kx_api.clone(),
                subject: "alice".into(),
                name: "alice".into(),
            },
            "kx",
            "test-only:alice",
            now() + 3600,
        )
        .unwrap();
    let grant = hub.authenticate(&token, "session").await.unwrap();
    hub.bind_local(&grant).unwrap();
    let app = router(hub);
    let request = Request::builder()
        .method("POST")
        .uri("/mcp")
        .header("accept", "application/json, text/event-stream")
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::json!({
                "jsonrpc": "2.0", "id": 1, "method": "initialize",
                "params": {"protocolVersion": crate::protocol::types::MCP_VERSION, "capabilities": {}, "clientInfo": {"name": "local-agent", "version": "1"}}
            })
            .to_string(),
        ))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.headers().get("mcp-session-id").is_some());
}

#[tokio::test]
async fn invalid_identity_configuration_never_exposes_legacy_routes() {
    let f = Fixture::new().await;
    std::fs::write(
        f.hub.legacy.storage.data_dir().join("auth.json"),
        "invalid-json",
    )
    .unwrap();
    let (app, _) = crate::dashboard::build_router(
        f.hub.legacy.storage.clone(),
        f.hub.legacy.cognitive.clone(),
        3931,
    );
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/memories")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 503);
    assert!(
        crate::protocol::http::start_http_transport(
            f.hub.legacy.storage.clone(),
            f.hub.legacy.cognitive.clone().unwrap(),
            f.hub.legacy.event_tx.clone(),
            "test-only-old-global-token".into(),
            0
        )
        .await
        .is_err()
    );
}

#[tokio::test]
async fn workspace_background_jobs_stop_with_their_runtime() {
    let f = Fixture::new().await;
    let state = AppState::new(
        f.hub.legacy.storage.clone(),
        Some(Arc::new(tokio::sync::Mutex::new(CognitiveEngine::new()))),
    );
    let jobs = crate::autopilot::spawn_managed(
        state.cognitive.clone().unwrap(),
        state.storage.clone(),
        state.event_tx.clone(),
    );
    tokio::task::yield_now().await;
    tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    assert!(state.event_tx.receiver_count() > 0);
    drop(jobs);
    for _ in 0..50 {
        if state.event_tx.receiver_count() == 0 {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    panic!("workspace subscriber survived retirement");
}

#[tokio::test]
async fn changing_legacy_owner_revokes_prior_membership_and_agent_grants() {
    let f = Fixture::new().await;
    let mut cfg = f.hub.config.clone();
    cfg.legacy_owner_subject = Some("alice".into());
    let h = Hub::new(cfg.clone(), f.hub.legacy.clone()).unwrap();
    let login = |hub: &Hub, name: &str| {
        hub.new_session(
            provider::Identity {
                issuer: hub.config.kx_api.clone(),
                subject: name.into(),
                name: name.into(),
            },
            "kx",
            &format!("test-only:{name}"),
            now() + 3600,
        )
        .unwrap()
    };
    let a = login(&h, "alice");
    let ga = h.authenticate(&a, "session").await.unwrap();
    h.account_action(&ga, &json!({"action":"switch","workspace":"legacy"}))
        .unwrap();
    let old = h.authenticate(&a, "session").await.unwrap();
    let token = h
        .account_action(
            &old,
            &json!({"action":"create_token","label":"old owner","role":"editor"}),
        )
        .unwrap()["token"]
        .as_str()
        .unwrap()
        .to_string();
    cfg.legacy_owner_subject = Some("bob".into());
    let h = Hub::new(cfg, f.hub.legacy.clone()).unwrap();
    assert!(h.check(&old).is_err());
    assert!(h.authenticate(&token, "agent").await.is_err());
    let restored = h.authenticate(&a, "session").await.unwrap();
    assert_ne!(restored.workspace, "legacy");
    assert!(
        h.account_action(&restored, &json!({"action":"switch","workspace":"legacy"}))
            .is_err()
    );
    let b = login(&h, "bob");
    let gb = h.authenticate(&b, "session").await.unwrap();
    h.account_action(&gb, &json!({"action":"switch","workspace":"legacy"}))
        .unwrap();
    assert_eq!(
        h.authenticate(&b, "session").await.unwrap().workspace,
        "legacy"
    );
}
