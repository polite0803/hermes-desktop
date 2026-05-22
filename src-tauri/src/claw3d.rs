// Claw3D integration — dev server + adapter lifecycle, npm discovery, port detection
// Rewrite from original src/main/claw3d.ts (888 lines TS)

use serde::{Deserialize, Serialize};
use std::fs;
use std::net::TcpStream;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

use crate::hermes_cli;

// ─── Constants ──────────────────────────────────────────

const DEV_PID_FILE: &str = "claw3d-dev.pid";
const ADAPTER_PID_FILE: &str = "claw3d-adapter.pid";
const PORT_FILE: &str = "claw3d-port";
const WS_URL_FILE: &str = "claw3d-ws-url";
const DEFAULT_PORT: u16 = 3000;
const DEFAULT_WS_URL: &str = "ws://localhost:18789";

fn office_dir() -> PathBuf { hermes_cli::resolve_hermes_home().join("claw3d") }
fn setting_path(name: &str) -> PathBuf { hermes_cli::resolve_hermes_home().join(name) }

// ─── Data Types ──────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Claw3dStatus {
    pub cloned: bool,
    pub installed: bool,
    #[serde(rename = "devServerRunning")]
    pub dev_server_running: bool,
    #[serde(rename = "adapterRunning")]
    pub adapter_running: bool,
    pub port: u16,
    #[serde(rename = "portInUse")]
    pub port_in_use: bool,
    #[serde(rename = "wsUrl")]
    pub ws_url: String,
    pub running: bool,
    pub error: String,
    #[serde(rename = "remoteUrl")]
    pub remote_url: Option<String>,
    #[serde(rename = "remoteSource")]
    pub remote_source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Claw3dResult {
    pub success: bool,
    pub error: Option<String>,
}

// ─── Port & Process Helpers ─────────────────────────────

fn check_port(port: u16) -> bool {
    TcpStream::connect_timeout(
        &format!("127.0.0.1:{}", port).parse().unwrap(),
        Duration::from_millis(500),
    ).is_ok()
}

fn read_pid_file(file: &str) -> Option<u32> {
    fs::read_to_string(setting_path(file)).ok()?.trim().parse().ok()
}

fn write_pid_file(file: &str, pid: u32) {
    let _ = fs::write(setting_path(file), pid.to_string());
}

fn remove_pid_file(file: &str) {
    let _ = fs::remove_file(setting_path(file));
}

fn is_process_alive(pid: u32) -> bool {
    #[cfg(windows)]
    {
        let mut cmd = Command::new("tasklist");
        cmd.args(&["/FI", &format!("PID eq {}", pid)])
            .stdout(Stdio::null()).stderr(Stdio::null());
        hermes_cli::hide_window(&mut cmd);
        cmd.status().map(|s| s.success()).unwrap_or(false)
    }
    #[cfg(not(windows))]
    {
        unsafe { libc::kill(pid as i32, 0) == 0 }
    }
}

// ─── NPM Discovery ──────────────────────────────────────

static NPM_CACHE: once_cell::sync::Lazy<Mutex<Option<String>>> =
    once_cell::sync::Lazy::new(|| Mutex::new(None));

fn find_npm() -> Option<String> {
    if let Ok(cache) = NPM_CACHE.lock() {
        if let Some(ref cmd) = *cache { return Some(cmd.clone()); }
    }

    let home = dirs_next::home_dir().unwrap_or_default();
    let suffix = if cfg!(windows) { ".cmd" } else { "" };
    let npm_name = format!("npm{}", suffix);

    let candidates = {
        let mut v: Vec<PathBuf> = Vec::new();
        if cfg!(windows) {
            if let Some(appdata) = std::env::var_os("APPDATA") {
                v.push(PathBuf::from(&appdata).join("npm").join(&npm_name));
            }
            if let Some(pf) = std::env::var_os("ProgramFiles") {
                v.push(PathBuf::from(&pf).join("nodejs").join(&npm_name));
            }
        }
        v.push(home.join(".volta").join("bin").join(&npm_name));
        v.push(home.join(".asdf").join("shims").join(&npm_name));
        v.push(home.join(".fnm").join("aliases").join("default").join("bin").join(&npm_name));
        v.push(PathBuf::from("/usr/local/bin/npm"));
        v.push(PathBuf::from("/opt/homebrew/bin/npm"));
        v
    };

    // Try nvm
    if let Ok(nvm_versions) = {
        let nvm_dir = std::env::var("NVM_DIR").ok().map(PathBuf::from)
            .unwrap_or_else(|| home.join(".nvm"));
        let v = nvm_dir.join("versions").join("node");
        if v.exists() { fs::read_dir(v) } else { Err(std::io::Error::from(std::io::ErrorKind::NotFound)) }
    } {
        let mut versions: Vec<_> = nvm_versions.filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().starts_with('v'))
            .map(|e| e.file_name())
            .collect();
        versions.sort_by(|a, b| b.cmp(a));
        for v in versions {
            let p = std::env::var("NVM_DIR").ok().map(PathBuf::from)
                .unwrap_or_else(|| home.join(".nvm"))
                .join("versions").join("node").join(v).join("bin").join(&npm_name);
            if p.exists() {
                let s = p.to_string_lossy().to_string();
                if let Ok(mut c) = NPM_CACHE.lock() { *c = Some(s.clone()); }
                return Some(s);
            }
        }
    }

    for c in &candidates {
        if c.exists() {
            let s = c.to_string_lossy().to_string();
            if let Ok(mut cache) = NPM_CACHE.lock() { *cache = Some(s.clone()); }
            return Some(s);
        }
    }

    // System PATH fallback
    let fallback = npm_name;
    if let Ok(mut cache) = NPM_CACHE.lock() { *cache = Some(fallback.clone()); }
    Some(fallback)
}

// ─── Saved Port / WS URL ────────────────────────────────

fn saved_port() -> u16 {
    fs::read_to_string(setting_path(PORT_FILE)).ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(DEFAULT_PORT)
}

fn saved_ws_url() -> String {
    fs::read_to_string(setting_path(WS_URL_FILE)).ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| DEFAULT_WS_URL.to_string())
}

// ─── Status ──────────────────────────────────────────────

#[tauri::command]
pub fn claw3d_status() -> Result<Claw3dStatus, String> {
    let office = office_dir();
    let cloned = office.join("package.json").exists();
    let installed = cloned && office.join("node_modules").exists();
    let port = saved_port();
    let dev_pid = read_pid_file(DEV_PID_FILE);
    let adapter_pid = read_pid_file(ADAPTER_PID_FILE);

    Ok(Claw3dStatus {
        cloned,
        installed,
        dev_server_running: dev_pid.map(is_process_alive).unwrap_or(false),
        adapter_running: adapter_pid.map(is_process_alive).unwrap_or(false),
        port,
        port_in_use: check_port(port),
        ws_url: saved_ws_url(),
        running: false, // computed below
        error: String::new(),
        remote_url: None,
        remote_source: None,
    })
    .map(|mut s| { s.running = s.dev_server_running || s.adapter_running; s })
}

// ─── Port / URL ─────────────────────────────────────────

#[tauri::command] pub fn claw3d_get_port() -> u16 { saved_port() }

#[tauri::command]
pub fn claw3d_set_port(port: u16) -> Result<bool, String> {
    fs::write(setting_path(PORT_FILE), port.to_string()).map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command] pub fn claw3d_get_ws_url() -> String { saved_ws_url() }

#[tauri::command]
pub fn claw3d_set_ws_url(url: String) -> Result<bool, String> {
    fs::write(setting_path(WS_URL_FILE), &url).map_err(|e| e.to_string())?;
    Ok(true)
}

// ─── Dev Server ─────────────────────────────────────────

fn kill_process(pid: u32) {
    #[cfg(windows)] {
        let mut cmd = Command::new("taskkill");
        cmd.args(&["/PID", &pid.to_string(), "/F"]);
        hermes_cli::hide_window(&mut cmd);
        let _ = cmd.output();
    }
    #[cfg(not(windows))] { unsafe { libc::kill(pid as i32, libc::SIGTERM); } }
}

#[tauri::command]
pub fn claw3d_start_dev(app: AppHandle) -> Result<bool, String> {
    let office = office_dir();
    if !office.join("package.json").exists() {
        return Err("Claw3D not cloned. Run setup first.".into());
    }
    if let Some(pid) = read_pid_file(DEV_PID_FILE) {
        if is_process_alive(pid) { return Ok(true); }
    }

    let npm = find_npm().ok_or("npm not found")?;
    let port = saved_port();
    let ws_url = saved_ws_url();

    let mut child_cmd = Command::new(&npm);
    child_cmd.args(&["run", "dev"])
        .current_dir(&office)
        .env("PORT", port.to_string())
        .env("NEXT_PUBLIC_WS_URL", &ws_url)
        .env("NODE_ENV", "development")
        .stdout(Stdio::null()).stderr(Stdio::null());
    hermes_cli::hide_window(&mut child_cmd);
    let child = child_cmd.spawn().map_err(|e| format!("Failed: {}", e))?;

    write_pid_file(DEV_PID_FILE, child.id());
    let _ = app.emit("claw3d-dev-started", ());
    Ok(true)
}

#[tauri::command]
pub fn claw3d_stop_dev() -> Result<bool, String> {
    if let Some(pid) = read_pid_file(DEV_PID_FILE) { kill_process(pid); }
    remove_pid_file(DEV_PID_FILE);
    Ok(true)
}

// ─── Adapter ─────────────────────────────────────────────

#[tauri::command]
pub fn claw3d_start_adapter(app: AppHandle) -> Result<bool, String> {
    let office = office_dir();
    if !office.join("package.json").exists() {
        return Err("Claw3D not cloned. Run setup first.".into());
    }
    if let Some(pid) = read_pid_file(ADAPTER_PID_FILE) {
        if is_process_alive(pid) { return Ok(true); }
    }

    let npm = find_npm().ok_or("npm not found")?;
    let ws_url = saved_ws_url();

    let mut adapter_cmd = Command::new(&npm);
    adapter_cmd.args(&["run", "adapter", "--", &ws_url])
        .current_dir(&office)
        .env("NODE_ENV", "production")
        .stdout(Stdio::null()).stderr(Stdio::null());
    hermes_cli::hide_window(&mut adapter_cmd);
    let child = adapter_cmd.spawn().map_err(|e| format!("Failed: {}", e))?;

    write_pid_file(ADAPTER_PID_FILE, child.id());
    let _ = app.emit("claw3d-adapter-started", ());
    Ok(true)
}

#[tauri::command]
pub fn claw3d_stop_adapter() -> Result<bool, String> {
    if let Some(pid) = read_pid_file(ADAPTER_PID_FILE) { kill_process(pid); }
    remove_pid_file(ADAPTER_PID_FILE);
    Ok(true)
}

// ─── All ─────────────────────────────────────────────────

#[tauri::command]
pub fn claw3d_start_all(app: AppHandle) -> Result<Claw3dResult, String> {
    let a = app.clone();
    match claw3d_start_dev(app) {
        Ok(_) => match claw3d_start_adapter(a) {
            Ok(_) => Ok(Claw3dResult { success: true, error: None }),
            Err(e) => Ok(Claw3dResult { success: false, error: Some(e) }),
        },
        Err(e) => Ok(Claw3dResult { success: false, error: Some(e) }),
    }
}

#[tauri::command]
pub fn claw3d_stop_all() -> Result<bool, String> {
    let _ = claw3d_stop_dev();
    let _ = claw3d_stop_adapter();
    Ok(true)
}

// ─── Logs ────────────────────────────────────────────────

#[tauri::command]
pub fn claw3d_get_logs() -> String {
    let mut out = String::new();
    if let Ok(dir) = fs::read_dir(office_dir().join(".next")) {
        for e in dir.filter_map(|e| e.ok()) {
            let n = e.file_name().to_string_lossy().to_string();
            if n.ends_with(".log") {
                if let Ok(c) = fs::read_to_string(e.path()) {
                    out.push_str(&format!("--- {} ---\n{}\n", n, c));
                }
            }
        }
    }
    if out.is_empty() { "No logs".into() } else { out }
}
