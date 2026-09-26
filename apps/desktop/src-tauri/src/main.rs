#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(target_os = "macos")]
mod snapshot;

use serde::Serialize;
use std::{
    collections::BTreeMap,
    fs,
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tauri::{
    Manager, State, WebviewUrl, WebviewWindowBuilder,
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
    tray::TrayIconBuilder,
};

#[derive(Default)]
struct Service(Mutex<Option<Child>>);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct LocalMcpStatus {
    running: bool,
    owned_by_app: bool,
    pid: Option<u32>,
    dashboard_url: String,
    mcp_url: String,
    data_dir: String,
    config_file: String,
    service_log: String,
    health: Option<serde_json::Value>,
    config: BTreeMap<String, String>,
}

fn port(environment: &BTreeMap<String, String>, key: &str, default: u16) -> u16 {
    environment
        .get(key)
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

fn local_mcp_status(service: &Service) -> Result<LocalMcpStatus, String> {
    let environment = service_environment()?;
    let dashboard_port = port(&environment, "VESTIGE_DASHBOARD_PORT", 3927);
    let mcp_port = port(&environment, "VESTIGE_HTTP_PORT", 3928);
    let dashboard_url = format!("http://127.0.0.1:{dashboard_port}");
    let mcp_url = format!("http://127.0.0.1:{mcp_port}/mcp");
    let client = reqwest::blocking::Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(2))
        .build()
        .map_err(|error| error.to_string())?;
    let health = client
        .get(format!("{dashboard_url}/api/health"))
        .send()
        .ok()
        .filter(|response| response.status().is_success())
        .and_then(|response| response.json::<serde_json::Value>().ok());
    let (owned_by_app, pid) = service
        .0
        .lock()
        .map_err(|_| "服务状态锁错误")?
        .as_ref()
        .map(|child| (true, Some(child.id())))
        .unwrap_or((false, None));
    let data = data_dir()?;
    let mut display_config = BTreeMap::new();
    for key in [
        "VESTIGE_DATA_DIR",
        "VESTIGE_DASHBOARD_PORT",
        "VESTIGE_HTTP_PORT",
        "VESTIGE_HTTP_BIND",
        "VESTIGE_DASHBOARD_ENABLED",
        "VESTIGE_HTTP_ENABLED",
        "VESTIGE_AUTH_CONFIG",
        "VESTIGE_QWEN_DEVICE",
        "VESTIGE_RERANKER_MODEL",
    ] {
        if let Some(value) = environment.get(key) {
            display_config.insert(key.to_string(), value.clone());
        }
    }
    let auth_file = display_config
        .get("VESTIGE_AUTH_CONFIG")
        .map(PathBuf::from)
        .unwrap_or_else(|| data.join("auth.json"));
    if let Ok(bytes) = fs::read(&auth_file) {
        if let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes) {
            for (key, output_key) in [
                ("public_origin", "AUTH_PUBLIC_ORIGIN"),
                ("kx_api", "AUTH_KX_API"),
            ] {
                if let Some(value) = value.get(key).and_then(|value| value.as_str()) {
                    display_config.insert(output_key.to_string(), value.to_string());
                }
            }
            display_config.insert(
                "AUTH_KX_CONFIGURED".into(),
                if value.get("kx_api").is_some() {
                    "是"
                } else {
                    "否"
                }
                .into(),
            );
            display_config.insert(
                "AUTH_OAUTH_CONFIGURED".into(),
                if value.get("oauth").is_some() {
                    "是"
                } else {
                    "否"
                }
                .into(),
            );
            // Never return client_secret or upstream credentials to the webview.
        }
    }
    Ok(LocalMcpStatus {
        running: health.is_some(),
        owned_by_app,
        pid,
        dashboard_url,
        mcp_url,
        data_dir: data.to_string_lossy().into_owned(),
        config_file: auth_file.to_string_lossy().into_owned(),
        service_log: data
            .join("desktop-service.log")
            .to_string_lossy()
            .into_owned(),
        health,
        config: display_config,
    })
}

#[tauri::command]
async fn local_mcp_status_command(
    service: State<'_, Arc<Service>>,
) -> Result<LocalMcpStatus, String> {
    let service = service.inner().clone();
    tauri::async_runtime::spawn_blocking(move || local_mcp_status(&service))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn local_mcp_start_command(
    app: tauri::AppHandle,
    service: State<'_, Arc<Service>>,
) -> Result<LocalMcpStatus, String> {
    let service = service.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        connect(&app, &service)?;
        local_mcp_status(&service)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn local_mcp_stop_command(
    service: State<'_, Arc<Service>>,
) -> Result<LocalMcpStatus, String> {
    let service = service.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        stop_owned(&service);
        local_mcp_status(&service)
    })
    .await
    .map_err(|error| error.to_string())?
}

fn show(app: &tauri::AppHandle) {
    #[cfg(target_os = "macos")]
    let _ = app.show();
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}
fn data_dir() -> Result<PathBuf, String> {
    if let Some(path) = std::env::var_os("VESTIGE_DATA_DIR") {
        return Ok(PathBuf::from(path));
    }
    directories::ProjectDirs::from("com", "vestige", "core")
        .map(|p| p.data_dir().to_path_buf())
        .ok_or_else(|| "无法定位本地数据目录".into())
}
// Login redirects stay in this WebView so the state-binding cookie survives.
// Remote pages still receive no Tauri IPC capabilities.
fn login_origins(settings: &BTreeMap<String, String>) -> Vec<String> {
    let path = settings
        .get("VESTIGE_AUTH_CONFIG")
        .map(PathBuf::from)
        .or_else(|| data_dir().ok().map(|p| p.join("auth.json")));
    let config = path
        .and_then(|p| fs::read(p).ok())
        .and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok());
    let Some(config) = config else {
        return vec![];
    };
    let mut origins = vec![
        "https://login.dingtalk.com".to_string(),
        "https://oapi.dingtalk.com".to_string(),
        "https://passport.dingtalk.com".to_string(),
    ];
    for value in [
        config.get("kx_api"),
        config.get("oauth").and_then(|o| o.get("authorize_url")),
    ]
    .into_iter()
    .flatten()
    {
        if let Some(url) = value.as_str().and_then(|s| tauri::Url::parse(s).ok()) {
            if url.scheme() == "https" {
                origins.push(url.origin().ascii_serialization());
            }
        }
    }
    origins
}

fn sidecar() -> Result<PathBuf, String> {
    let extension = if cfg!(windows) { ".exe" } else { "" };
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let installed = exe
        .parent()
        .ok_or("无法定位应用程序")?
        .join(format!("vestige-service{extension}"));
    if installed.is_file() {
        return Ok(installed);
    }
    let development = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("binaries")
        .join(format!(
            "vestige-service-{}{extension}",
            env!("VESTIGE_TARGET")
        ));
    if development.is_file() {
        return Ok(development);
    }
    Err("应用缺少随包提供的本地服务。请重新构建或安装完整应用。".into())
}
fn service_environment() -> Result<BTreeMap<String, String>, String> {
    let root = data_dir()?;
    fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    let mut values = BTreeMap::new();
    // Optional per-user runtime settings contain model paths, never app-code
    // paths or third-party generation keys. The webview cannot change them.
    let config = root.join("desktop-service.json");
    if config.is_file() {
        let content = fs::read_to_string(&config).map_err(|e| e.to_string())?;
        let configured: serde_json::Value =
            serde_json::from_str(&content).map_err(|_| "桌面服务配置不是有效 JSON")?;
        for key in [
            "VESTIGE_AUTH_CONFIG",
            "VESTIGE_EMBEDDING_ARTIFACT_DIR",
            "VESTIGE_QWEN_DEVICE",
            "VESTIGE_RERANKER_MODEL",
            "FASTEMBED_CACHE_PATH",
            "VESTIGE_DASHBOARD_PORT",
            "VESTIGE_HTTP_PORT",
        ] {
            if let Some(value) = configured.get(key).and_then(|v| v.as_str()) {
                values.insert(key.into(), value.into());
            }
        }
    }
    for key in [
        "HOME",
        "PATH",
        "TMPDIR",
        "LANG",
        "USER",
        "LOGNAME",
        "VESTIGE_AUTH_CONFIG",
    ] {
        if let Ok(value) = std::env::var(key) {
            values.insert(key.into(), value);
        }
    }
    for (key, value) in [
        ("VESTIGE_DASHBOARD_ENABLED", "1"),
        ("VESTIGE_HTTP_ENABLED", "1"),
        ("VESTIGE_HTTP_BIND", "127.0.0.1"),
        // The desktop login session authorizes only the loopback MCP bridge;
        // remote listeners continue to require explicit Agent credentials.
        ("VESTIGE_LOCAL_MCP_MODE", "1"),
        ("VESTIGE_AUTO_CONSOLIDATE_MERGE", "0"),
        ("RUST_LOG", "info"),
        ("TOKENIZERS_PARALLELISM", "false"),
    ] {
        values.insert(key.into(), value.into());
    }
    values.insert(
        "VESTIGE_DATA_DIR".into(),
        root.to_string_lossy().into_owned(),
    );
    for key in ["VESTIGE_DASHBOARD_PORT", "VESTIGE_HTTP_PORT"] {
        if let Ok(value) = std::env::var(key) {
            values.insert(key.into(), value);
        }
    }
    Ok(values)
}
fn connect(app: &tauri::AppHandle, service: &Arc<Service>) -> Result<String, String> {
    let environment = service_environment()?;
    let port = environment
        .get("VESTIGE_DASHBOARD_PORT")
        .map(String::as_str)
        .unwrap_or("3927")
        .parse::<u16>()
        .map_err(|_| "无效的服务端口")?;
    let origin = format!("http://127.0.0.1:{port}");
    let client = reqwest::blocking::Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(3))
        .build()
        .map_err(|e| e.to_string())?;
    let probe = || -> Result<bool, String> {
        match client.get(format!("{origin}/api/health")).send() {
            Ok(response) => {
                let status = response.status();
                let value = response
                    .json::<serde_json::Value>()
                    .map_err(|_| "当前端口不是兼容的 Vestige 服务")?;
                if !status.is_success() || value["writerApiVersion"].as_u64().unwrap_or(0) < 1 {
                    return Err("当前端口运行的服务尚不支持编剧工作台。请先升级该服务，或使用独立的数据目录与端口。".into());
                }
                Ok(true)
            }
            Err(error) if error.is_connect() => Ok(false),
            Err(_) => Err("本地服务正在响应其他任务或暂时无响应，请稍后重新打开应用。".into()),
        }
    };
    if probe()? {
        return Ok(format!("{origin}/dashboard/writer"));
    }
    let root = data_dir()?;
    let path = root.join("desktop-service.log");
    if fs::metadata(&path)
        .map(|m| m.len() > 5 * 1024 * 1024)
        .unwrap_or(false)
    {
        let _ = fs::rename(&path, root.join("desktop-service.previous.log"));
    }
    let log = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    let child = Command::new(sidecar()?)
        .args(["--daemon", "--http"])
        .env_clear()
        .envs(environment)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(log)
        .spawn()
        .map_err(|e| format!("无法启动本地服务：{e}"))?;
    *service.0.lock().map_err(|_| "服务状态锁错误")? = Some(child);
    let started = Instant::now();
    loop {
        if let Some(child) = service.0.lock().map_err(|_| "服务状态锁错误")?.as_mut() {
            if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
                return Err(format!(
                    "本地服务已退出（{status}），请检查数据目录中的 desktop-service.log。"
                ));
            }
        }
        if probe().unwrap_or(false) {
            break;
        }
        if started.elapsed() > Duration::from_secs(120) {
            return Err("服务未在两分钟内就绪，请检查模型路径、端口和服务日志。".into());
        }
        if app.get_webview_window("main").is_none() {
            return Err("窗口已关闭".into());
        }
        std::thread::sleep(Duration::from_millis(300));
    }
    Ok(format!("{origin}/dashboard/writer"))
}
fn stop_owned(service: &Service) {
    if let Ok(mut guard) = service.0.lock() {
        if let Some(mut child) = guard.take() {
            #[cfg(unix)]
            unsafe {
                libc::kill(child.id() as i32, libc::SIGTERM);
            }
            #[cfg(not(unix))]
            {
                let _ = child.kill();
            }
            for _ in 0..50 {
                if matches!(child.try_wait(), Ok(Some(_))) {
                    return;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}
fn main() {
    let service = Arc::new(Service::default());
    let setup_service = service.clone();
    let app = tauri::Builder::default()
        .manage(service.clone())
        .invoke_handler(tauri::generate_handler![
            local_mcp_status_command,
            local_mcp_start_command,
            local_mcp_stop_command,
        ])
        .plugin(tauri_plugin_single_instance::init(|app, args, cwd| {
            if args.iter().any(|arg| arg == "--quit") {
                app.exit(0);
            } else if args.iter().any(|arg| arg == "--close-window") {
                if let Some(window) = app.get_webview_window("main") {
                    eprintln!("Desktop close-window command received");
                    if let Err(error) = window.close() {
                        eprintln!("Window close failed: {error}");
                    }
                }
            } else {
                #[cfg(target_os = "macos")]
                if let Some(index) = args.iter().position(|arg| arg == "--snapshot") {
                    if let (Some(path), Some(window)) =
                        (args.get(index + 1), app.get_webview_window("main"))
                    {
                        let output = PathBuf::from(path);
                        let output = if output.is_absolute() {
                            output
                        } else {
                            PathBuf::from(cwd).join(output)
                        };
                        snapshot::capture(&window, output);
                    }
                    return;
                }
                show(app);
            }
        }))
        .setup(move |app| {
            let settings = service_environment().map_err(std::io::Error::other)?;
            let port = settings
                .get("VESTIGE_DASHBOARD_PORT")
                .cloned()
                .unwrap_or_else(|| "3927".into());
            let origin = format!("http://127.0.0.1:{port}");
            let identity_origins = login_origins(&settings);
            let window =
                WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                    .title("Vestige 编剧工作台")
                    .inner_size(1440.0, 960.0)
                    .min_inner_size(850.0, 600.0)
                    .on_navigation(move |url| {
                        if url.scheme() == "tauri"
                            || url.origin().ascii_serialization() == "http://tauri.localhost"
                            || url.origin().ascii_serialization() == origin
                            || identity_origins.contains(&url.origin().ascii_serialization())
                            || (url.scheme() == "blob"
                                && url
                                    .path()
                                    .strip_prefix(&origin)
                                    .is_some_and(|suffix| suffix.starts_with('/')))
                        {
                            return true;
                        }
                        if matches!(url.scheme(), "https" | "http") {
                            let _ = open::that(url.as_str());
                        }
                        false
                    })
                    .on_new_window(|url, _| {
                        if matches!(url.scheme(), "http" | "https") {
                            let _ = open::that(url.as_str());
                        }
                        tauri::webview::NewWindowResponse::Deny
                    })
                    .build()?;
            let hidden = window.clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    // Tao's macOS orderOut can synchronously emit Focused.
                    // Close listeners still hold their listener mutex here;
                    // enqueue Hide from another thread after this callback,
                    // instead of re-entering that mutex on the event thread.
                    let window = hidden.clone();
                    std::thread::spawn(move || {
                        if let Err(error) = window.hide() {
                            eprintln!("Window hide failed: {error}");
                        }
                        eprintln!(
                            "Desktop window visibility after close: {:?}",
                            window.is_visible()
                        );
                    });
                }
            });
            let application_menu = Submenu::with_items(
                app,
                "Vestige",
                true,
                &[
                    &PredefinedMenuItem::about(app, Some("关于编剧工作台"), None)?,
                    &PredefinedMenuItem::separator(app)?,
                    &PredefinedMenuItem::hide(app, Some("隐藏工作台"))?,
                    &PredefinedMenuItem::quit(app, Some("退出应用"))?,
                ],
            )?;
            let edit_menu = Submenu::with_items(
                app,
                "编辑",
                true,
                &[
                    &PredefinedMenuItem::undo(app, Some("撤销"))?,
                    &PredefinedMenuItem::redo(app, Some("重做"))?,
                    &PredefinedMenuItem::separator(app)?,
                    &PredefinedMenuItem::cut(app, Some("剪切"))?,
                    &PredefinedMenuItem::copy(app, Some("复制"))?,
                    &PredefinedMenuItem::paste(app, Some("粘贴"))?,
                    &PredefinedMenuItem::select_all(app, Some("全选"))?,
                ],
            )?;
            let window_menu = Submenu::with_items(
                app,
                "窗口",
                true,
                &[
                    &PredefinedMenuItem::minimize(app, Some("最小化"))?,
                    &PredefinedMenuItem::close_window(app, Some("关闭窗口"))?,
                ],
            )?;
            app.set_menu(Menu::with_items(
                app,
                &[&application_menu, &edit_menu, &window_menu],
            )?)?;
            let open = MenuItem::with_id(app, "open", "打开编剧工作台", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "退出应用", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open, &quit])?;
            let mut rgba = vec![0u8; 32 * 32 * 4];
            for y in 0..32 {
                for x in 0..32 {
                    let i = (y * 32 + x) * 4;
                    let visible = (x as i32 - 16).pow(2) + (y as i32 - 16).pow(2) < 170;
                    rgba[i..i + 4].copy_from_slice(if visible {
                        &[110, 202, 153, 255]
                    } else {
                        &[0, 0, 0, 0]
                    });
                }
            }
            TrayIconBuilder::new()
                .icon(tauri::image::Image::new_owned(rgba, 32, 32))
                .tooltip("Vestige 编剧工作台")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => show(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;
            let handle = app.handle().clone();
            let running = setup_service.clone();
            std::thread::spawn(move || match connect(&handle, &running) {
                Ok(url) => {
                    if let Some(window) = handle.get_webview_window("main") {
                        if let Ok(url) = url.parse() {
                            let _ = window.navigate(url);
                        }
                    }
                }
                Err(message) => {
                    stop_owned(&running);
                    if let Some(window) = handle.get_webview_window("main") {
                        let script = format!(
                            "document.getElementById('status').textContent={};",
                            serde_json::to_string(&message).unwrap_or_default()
                        );
                        let _ = window.eval(&script);
                    }
                }
            });
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("无法启动 Vestige 桌面应用");
    app.run(move |app, event| match event {
        tauri::RunEvent::Exit => stop_owned(&service),
        #[cfg(target_os = "macos")]
        tauri::RunEvent::Reopen {
            has_visible_windows,
            ..
        } => {
            eprintln!("Desktop reopen event, visible={has_visible_windows}");
            show(app);
        }
        _ => {}
    });
}
