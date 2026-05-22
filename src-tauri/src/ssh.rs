// SSH protocol stack — remote command execution, file I/O, tunnel management
// Full rewrite from original src/main/ssh-remote.ts (1,787 lines) +
// src/main/ssh-tunnel.ts (258 lines) + src/main/ssh-options.ts (32 lines)
//
// Architecture:
//   1. ssh_exec() — core: spawn `ssh user@host command`, collect stdout/stderr, timeout
//   2. ssh_read_file() / ssh_write_file() — remote file operations via bash scripts
//   3. Remote operations — skills, memory, soul, sessions, profiles, config, etc.
//   4. Tunnel management — long-lived `ssh -N -L` process for HTTP proxy

use std::collections::HashMap;
use std::io::Write;
use std::net::TcpStream;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::config::{self, SshConfig};

// ─── SSH Tunnel State ────────────────────────────────────

#[derive(Debug)]
pub enum SshTunnelState {
    Connected {
        local_port: u16,
        remote_host: String,
        remote_port: u16,
    },
    Disconnected,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SshConnectionResult {
    pub success: bool,
    pub message: String,
}

// ─── Build SSH Args ─────────────────────────────────────

/// Platform-aware SSH control options.
fn build_ssh_control_options(_for_tunnel: bool) -> Vec<String> {
    #[cfg(target_os = "windows")]
    {
        vec!["-o".into(), "ControlMaster=no".into(),
             "-o".into(), "ControlPath=none".into(),
             "-o".into(), "ControlPersist=no".into()]
    }
    #[cfg(not(target_os = "windows"))]
    {
        if for_tunnel {
            vec!["-o".into(), "ControlMaster=no".into(),
                 "-o".into(), "ControlPath=none".into(),
                 "-o".into(), "ControlPersist=no".into()]
        } else {
            vec!["-o".into(), "ControlMaster=auto".into(),
                 "-o".into(), "ControlPath=~/.ssh/cm-hermes-%r@%h:%p".into(),
                 "-o".into(), "ControlPersist=60s".into()]
        }
    }
}

/// Build SSH args for command execution (ssh user@host command).
fn build_exec_args(cfg: &SshConfig) -> Vec<String> {
    let key_path = if cfg.key_path.is_empty() {
        let home = dirs_next::home_dir().unwrap_or_default();
        home.join(".ssh").join("id_rsa").to_string_lossy().to_string()
    } else {
        cfg.key_path.clone()
    };

    let mut args = vec![
        "-o".to_string(), "BatchMode=yes".to_string(),
        "-o".to_string(), "StrictHostKeyChecking=accept-new".to_string(),
        "-o".to_string(), "ConnectTimeout=15".to_string(),
    ];
    args.extend(build_ssh_control_options(false));
    args.push("-i".to_string());
    args.push(key_path);
    args.push("-p".to_string());
    args.push(cfg.port.to_string());
    args.push(format!("{}@{}", cfg.username, cfg.host));
    args
}

/// Build SSH args for tunnel (ssh -N -L localPort:remoteHost:remotePort user@host).
fn build_tunnel_args(cfg: &SshConfig, local_port: u16) -> Vec<String> {
    let key_path = if cfg.key_path.is_empty() {
        let home = dirs_next::home_dir().unwrap_or_default();
        home.join(".ssh").join("id_rsa").to_string_lossy().to_string()
    } else {
        cfg.key_path.clone()
    };

    vec![
        "-N".to_string(),
        "-L".to_string(), format!("{}:127.0.0.1:{}", local_port, cfg.remote_port),
        "-p".to_string(), cfg.port.to_string(),
        "-i".to_string(), key_path,
        "-o".to_string(), "StrictHostKeyChecking=accept-new".to_string(),
        "-o".to_string(), "BatchMode=yes".to_string(),
    ]
    .into_iter()
    .chain(build_ssh_control_options(true))
    .chain(vec![
        "-o".into(), "ExitOnForwardFailure=yes".into(),
        "-o".into(), "ServerAliveInterval=30".into(),
        "-o".into(), "ServerAliveCountMax=3".into(),
        format!("{}@{}", cfg.username, cfg.host),
    ])
    .collect()
}

// ─── Core SSH Execution ─────────────────────────────────

/// Execute a command on the remote host via SSH subprocess.
pub fn ssh_exec(config: &SshConfig, command: &str, stdin: Option<&str>, timeout_ms: u64) -> Result<String, String> {
    let mut cmd = Command::new("ssh");
    cmd.args(&build_exec_args(config))
        .arg(command)
        .stdin(if stdin.is_some() { Stdio::piped() } else { Stdio::null() })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }

    let mut child = cmd.spawn().map_err(|e| format!("SSH spawn failed: {}", e))?;

    // Write stdin if provided
    if let Some(input) = stdin {
        if let Some(mut stdin_pipe) = child.stdin.take() {
            let _ = stdin_pipe.write_all(input.as_bytes());
            // stdin_pipe dropped here → EOF sent
        }
    }

    // Read with timeout via wait_timeout
    let start = Instant::now();
    let timeout = Duration::from_millis(timeout_ms);

    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let stdout = child.stdout.as_mut()
                    .and_then(|s| { let mut buf = String::new(); std::io::Read::read_to_string(s, &mut buf).ok().map(|_| buf) })
                    .unwrap_or_default();
                let stderr = child.stderr.as_mut()
                    .and_then(|s| { let mut buf = String::new(); std::io::Read::read_to_string(s, &mut buf).ok().map(|_| buf) })
                    .unwrap_or_default();

                if status.success() {
                    return Ok(stdout);
                } else {
                    let err = sanitize_ssh_error(&stderr);
                    return Err(if err.is_empty() { "SSH command failed".into() } else { err });
                }
            }
            Ok(None) => {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err("SSH command timed out".into());
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(e) => return Err(format!("SSH wait error: {}", e)),
        }
    }
}

fn sanitize_ssh_error(stderr: &str) -> String {
    stderr.lines()
        .filter(|l| !l.contains("Warning:") && !l.contains("Permanently added"))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

/// Shell-quote a value for safe bash command construction.
fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

/// Normalize a remote file path (handle ~/ prefix).
fn normalize_remote_path(path: &str) -> String {
    path.replacen("~/", "$HOME/", 1)
}

// ─── Remote File I/O ────────────────────────────────────

/// Read a file on the remote host via SSH.
pub fn ssh_read_file(config: &SshConfig, remote_path: &str) -> Result<String, String> {
    let norm = normalize_remote_path(remote_path);
    let script = format!(
        r#"bash -c 'case "$1" in "~/"*) p="$HOME/${{1#~/}}" ;; "\$HOME/"*) p="$HOME/${{1#\$HOME/}}" ;; *) p="$1" ;; esac; cat -- "$p" 2>/dev/null || true' -- {}"#,
        shell_quote(&norm)
    );
    match ssh_exec(config, &script, None, 10000) {
        Ok(out) => Ok(out),
        Err(_) => Ok(String::new()),
    }
}

/// Write content to a file on the remote host via SSH.
pub fn ssh_write_file(config: &SshConfig, remote_path: &str, content: &str) -> Result<(), String> {
    let norm = normalize_remote_path(remote_path);
    let dir = if norm.contains('/') {
        norm[..norm.rfind('/').unwrap()].to_string()
    } else {
        ".".to_string()
    };
    let script = format!(
        r#"bash -c 'expand(){{ case "$1" in "~/"*) printf "%s" "$HOME/${{1#~/}}" ;; "\$HOME/"*) printf "%s" "$HOME/${{1#\$HOME/}}" ;; *) printf "%s" "$1" ;; esac; }}; dir=$(expand "$1"); file=$(expand "$2"); mkdir -p -- "$dir" && cat > "$file"' -- {} {}"#,
        shell_quote(&dir), shell_quote(&norm)
    );
    ssh_exec(config, &script, Some(content), 15000)?;
    Ok(())
}

// ─── Python Execution on Remote ─────────────────────────

/// Execute a Python script on the remote host (via stdin piping).
fn ssh_python(config: &SshConfig, script: &str, stdin_payload: Option<&str>, timeout_ms: u64) -> Result<String, String> {
    let cmd = format!("python3 -c {}", shell_quote(script));
    ssh_exec(config, &cmd, stdin_payload, timeout_ms)
}

/// Build the remote Hermes CLI command string with venv probing.
fn build_remote_hermes_cmd(args: &[&str]) -> String {
    let candidates = [
        "$HOME/hermes-agent/.venv/bin/hermes",
        "$HOME/.hermes/hermes-agent/.venv/bin/hermes",
        "/opt/hermes/hermes-agent/.venv/bin/hermes",
    ];
    let quoted_args = args.iter().map(|a| shell_quote(a)).collect::<Vec<_>>().join(" ");
    let probe = candidates.iter()
        .map(|p| format!("[ -x {} ] && exec {} {}", p, p, quoted_args))
        .collect::<Vec<_>>()
        .join("; ");
    format!(
        "bash -c '{}; command -v hermes >/dev/null && exec hermes {}; echo \"ERR: hermes CLI not found on remote\" >&2; exit 1'",
        probe, quoted_args
    )
}

// ─── Remote Operations ──────────────────────────────────

/// Remote profile paths helper.
fn remote_profile_path(profile: Option<&str>) -> String {
    if let Some(p) = profile {
        if p != "default" {
            format!("~/.hermes/profiles/{}", p)
        } else {
            "~/.hermes".into()
        }
    } else {
        "~/.hermes".into()
    }
}

/// Remote: list installed skills
pub fn ssh_list_installed_skills(config: &SshConfig, _profile: Option<&str>) -> Result<Vec<serde_json::Value>, String> {
    let cmd = build_remote_hermes_cmd(&["skills", "list", "--json"]);
    let out = ssh_exec(config, &cmd, None, 15000)?;
    Ok(serde_json::from_str(&out).unwrap_or_default())
}

/// Remote: read memory
pub fn ssh_read_memory(config: &SshConfig, profile: Option<&str>) -> Result<serde_json::Value, String> {
    let base = remote_profile_path(profile);
    let mem = ssh_read_file(config, &format!("{}/memories/MEMORY.md", base))?;
    let user = ssh_read_file(config, &format!("{}/memories/USER.md", base))?;
    let stats = ssh_get_session_stats(config, profile)?;

    Ok(serde_json::json!({
        "memory": {
            "content": mem,
            "exists": !mem.is_empty(),
            "lastModified": null,
            "charCount": mem.len(),
            "charLimit": 2200
        },
        "user": {
            "content": user,
            "exists": !user.is_empty(),
            "lastModified": null,
            "charCount": user.len(),
            "charLimit": 1375
        },
        "stats": stats,
    }))
}

fn ssh_get_session_stats(config: &SshConfig, profile: Option<&str>) -> Result<serde_json::Value, String> {
    let db_path = format!("{}/state.db", remote_profile_path(profile));
    let script = format!(
        "import sqlite3,json;c=sqlite3.connect({});s=c.execute('SELECT COUNT(*) FROM sessions').fetchone()[0];m=c.execute('SELECT COUNT(*) FROM messages').fetchone()[0];print(json.dumps({{'totalSessions':s,'totalMessages':m}}))",
        shell_quote(&db_path)
    );
    let out = ssh_python(config, &script, None, 10000)?;
    Ok(serde_json::from_str(&out).unwrap_or(serde_json::json!({"totalSessions":0,"totalMessages":0})))
}

/// Remote: read soul
pub fn ssh_read_soul(config: &SshConfig, profile: Option<&str>) -> Result<String, String> {
    ssh_read_file(config, &format!("{}/SOUL.md", remote_profile_path(profile)))
}

/// Remote: write soul
pub fn ssh_write_soul(config: &SshConfig, profile: Option<&str>, content: &str) -> Result<(), String> {
    ssh_write_file(config, &format!("{}/SOUL.md", remote_profile_path(profile)), content)
}

/// Remote: get config value
pub fn ssh_get_config_value(config: &SshConfig, _profile: Option<&str>, key: &str) -> Result<Option<String>, String> {
    let cmd = build_remote_hermes_cmd(&["config", "get", key, "--json"]);
    match ssh_exec(config, &cmd, None, 10000) {
        Ok(out) => {
            let trimmed = out.trim();
            if trimmed.is_empty() || trimmed == "null" { Ok(None) } else { Ok(Some(trimmed.to_string())) }
        }
        Err(_) => Ok(None),
    }
}

/// Remote: set config value
pub fn ssh_set_config_value(config: &SshConfig, _profile: Option<&str>, key: &str, value: &str) -> Result<(), String> {
    let cmd = build_remote_hermes_cmd(&["config", "set", key, value]);
    ssh_exec(config, &cmd, None, 10000).map(|_| ())
}

/// Remote: get model config
pub fn ssh_get_model_config(config: &SshConfig, _profile: Option<&str>) -> Result<serde_json::Value, String> {
    let cmd = build_remote_hermes_cmd(&["config", "model", "--json"]);
    let out = ssh_exec(config, &cmd, None, 10000)?;
    Ok(serde_json::from_str(&out).unwrap_or(serde_json::json!({"provider":"auto","model":"","baseUrl":""})))
}

/// Remote: list profiles
pub fn ssh_list_profiles(config: &SshConfig) -> Result<Vec<serde_json::Value>, String> {
    let cmd = build_remote_hermes_cmd(&["profile", "list", "--json"]);
    let out = ssh_exec(config, &cmd, None, 15000)?;
    Ok(serde_json::from_str(&out).unwrap_or_default())
}

/// Remote: create profile
pub fn ssh_create_profile(config: &SshConfig, name: &str, clone: bool) -> Result<serde_json::Value, String> {
    let mut args = vec!["profile", "create", name, "--json"];
    if clone { args.push("--clone"); }
    let cmd = build_remote_hermes_cmd(&args);
    let out = ssh_exec(config, &cmd, None, 10000)?;
    Ok(serde_json::from_str(&out).unwrap_or(serde_json::json!({"success":true})))
}

/// Remote: delete profile
pub fn ssh_delete_profile(config: &SshConfig, name: &str) -> Result<(), String> {
    let cmd = build_remote_hermes_cmd(&["profile", "delete", name]);
    ssh_exec(config, &cmd, None, 10000).map(|_| ())
}

/// Remote: get toolsets
pub fn ssh_get_toolsets(config: &SshConfig, _profile: Option<&str>) -> Result<Vec<serde_json::Value>, String> {
    let cmd = build_remote_hermes_cmd(&["tools", "list", "--enabled", "--json"]);
    let out = ssh_exec(config, &cmd, None, 10000)?;
    Ok(serde_json::from_str(&out).unwrap_or_default())
}

/// Remote: set toolset enabled
pub fn ssh_set_toolset_enabled(config: &SshConfig, _profile: Option<&str>, name: &str, enabled: bool) -> Result<(), String> {
    let action = if enabled { "enable" } else { "disable" };
    let cmd = build_remote_hermes_cmd(&["tools", action, name]);
    ssh_exec(config, &cmd, None, 10000).map(|_| ())
}

/// Remote: list sessions
pub fn ssh_list_sessions(config: &SshConfig, limit: Option<u32>, offset: Option<u32>) -> Result<Vec<serde_json::Value>, String> {
    let mut args = vec!["sessions", "list", "--json"];
    let limit_str;
    let offset_str;
    if let Some(l) = limit { limit_str = l.to_string(); args.push("--limit"); args.push(&limit_str); }
    if let Some(o) = offset { offset_str = o.to_string(); args.push("--offset"); args.push(&offset_str); }
    let cmd = build_remote_hermes_cmd(&args);
    let out = ssh_exec(config, &cmd, None, 15000)?;
    Ok(serde_json::from_str(&out).unwrap_or_default())
}

/// Remote: get session messages
pub fn ssh_get_session_messages(config: &SshConfig, session_id: &str) -> Result<Vec<serde_json::Value>, String> {
    let cmd = build_remote_hermes_cmd(&["sessions", "messages", session_id, "--json"]);
    let out = ssh_exec(config, &cmd, None, 15000)?;
    Ok(serde_json::from_str(&out).unwrap_or_default())
}

/// Remote: search sessions
pub fn ssh_search_sessions(config: &SshConfig, query: &str, limit: Option<u32>) -> Result<Vec<serde_json::Value>, String> {
    let mut args = vec!["sessions", "search", query, "--json"];
    let limit_str;
    if let Some(l) = limit { limit_str = l.to_string(); args.push("--limit"); args.push(&limit_str); }
    let cmd = build_remote_hermes_cmd(&args);
    let out = ssh_exec(config, &cmd, None, 15000)?;
    Ok(serde_json::from_str(&out).unwrap_or_default())
}

/// Remote: list models
pub fn ssh_list_models(config: &SshConfig) -> Result<Vec<serde_json::Value>, String> {
    let cmd = build_remote_hermes_cmd(&["models", "list", "--json"]);
    let out = ssh_exec(config, &cmd, None, 10000)?;
    Ok(serde_json::from_str(&out).unwrap_or_default())
}

/// Remote: add model
pub fn ssh_add_model(config: &SshConfig, name: &str, provider: &str, model: &str, base_url: &str) -> Result<serde_json::Value, String> {
    let cmd = build_remote_hermes_cmd(&["models", "add", name, "--provider", provider, "--model", model, "--base-url", base_url, "--json"]);
    let out = ssh_exec(config, &cmd, None, 10000)?;
    Ok(serde_json::from_str(&out).unwrap_or(serde_json::json!({"id":"","name":"","provider":"","model":"","baseUrl":""})))
}

/// Remote: remove model
pub fn ssh_remove_model(config: &SshConfig, id: &str) -> Result<(), String> {
    let cmd = build_remote_hermes_cmd(&["models", "remove", id]);
    ssh_exec(config, &cmd, None, 10000).map(|_| ())
}

/// Remote: gateway status
pub fn ssh_gateway_status(config: &SshConfig) -> Result<bool, String> {
    let cmd = build_remote_hermes_cmd(&["gateway", "status"]);
    let out = ssh_exec(config, &cmd, None, 10000)?;
    Ok(out.trim().contains("running"))
}

/// Remote: start gateway
pub fn ssh_start_gateway(config: &SshConfig) -> Result<(), String> {
    let cmd = build_remote_hermes_cmd(&["gateway", "start"]);
    ssh_exec(config, &cmd, None, 30000).map(|_| ())
}

/// Remote: stop gateway
pub fn ssh_stop_gateway(config: &SshConfig) -> Result<(), String> {
    let cmd = build_remote_hermes_cmd(&["gateway", "stop"]);
    ssh_exec(config, &cmd, None, 15000).map(|_| ())
}

/// Remote: run hermes doctor
pub fn ssh_run_doctor(config: &SshConfig) -> Result<String, String> {
    let cmd = build_remote_hermes_cmd(&["doctor"]);
    ssh_exec(config, &cmd, None, 30000)
}

/// Remote: run hermes update
pub fn ssh_run_update(config: &SshConfig) -> Result<String, String> {
    let cmd = build_remote_hermes_cmd(&["update"]);
    ssh_exec(config, &cmd, None, 60000)
}

/// Remote: run hermes dump
pub fn ssh_run_dump(config: &SshConfig) -> Result<String, String> {
    let cmd = build_remote_hermes_cmd(&["dump"]);
    ssh_exec(config, &cmd, None, 30000)
}

/// Remote: read logs
pub fn ssh_read_logs(config: &SshConfig, log_file: Option<&str>, lines: Option<u32>) -> Result<serde_json::Value, String> {
    let mut args = vec!["logs"];
    if let Some(f) = log_file { args.push(f); }
    let lines_str;
    if let Some(l) = lines { lines_str = l.to_string(); args.push("--lines"); args.push(&lines_str); }
    let cmd = build_remote_hermes_cmd(&args);
    let out = ssh_exec(config, &cmd, None, 15000)?;
    Ok(serde_json::json!({"content": out, "path": ""}))
}

/// Remote: get platform enabled
pub fn ssh_get_platform_enabled(config: &SshConfig, _profile: Option<&str>) -> Result<HashMap<String, bool>, String> {
    let cmd = build_remote_hermes_cmd(&["platforms", "list", "--json"]);
    let out = ssh_exec(config, &cmd, None, 10000)?;
    Ok(serde_json::from_str(&out).unwrap_or_default())
}

/// Remote: set platform enabled
pub fn ssh_set_platform_enabled(config: &SshConfig, platform: &str, enabled: bool) -> Result<(), String> {
    let action = if enabled { "enable" } else { "disable" };
    let cmd = build_remote_hermes_cmd(&["platforms", action, platform]);
    ssh_exec(config, &cmd, None, 10000).map(|_| ())
}

/// Remote: run kanban command
pub fn ssh_run_kanban(config: &SshConfig, args: &[&str]) -> Result<String, String> {
    let mut full_args = vec!["kanban"];
    full_args.extend_from_slice(args);
    let cmd = build_remote_hermes_cmd(&full_args);
    ssh_exec(config, &cmd, None, 20000)
}

/// Remote: discover memory providers
pub fn ssh_discover_memory_providers(config: &SshConfig, _profile: Option<&str>) -> Result<Vec<serde_json::Value>, String> {
    let cmd = build_remote_hermes_cmd(&["memory", "list-providers", "--json"]);
    let out = ssh_exec(config, &cmd, None, 10000)?;
    Ok(serde_json::from_str(&out).unwrap_or_default())
}

/// Remote: list MCP servers
pub fn ssh_list_mcp_servers(config: &SshConfig, _profile: Option<&str>) -> Result<Vec<serde_json::Value>, String> {
    let cmd = build_remote_hermes_cmd(&["mcp", "list", "--json"]);
    let out = ssh_exec(config, &cmd, None, 10000)?;
    Ok(serde_json::from_str(&out).unwrap_or_default())
}

/// Remote: read hermes version
pub fn ssh_get_hermes_version(config: &SshConfig) -> Result<Option<String>, String> {
    match ssh_exec(config, "hermes --version", None, 10000) {
        Ok(out) => Ok(Some(out.trim().to_string())),
        Err(_) => Ok(None),
    }
}

/// Remote: read env file
pub fn ssh_read_env(config: &SshConfig, profile: Option<&str>) -> Result<HashMap<String, String>, String> {
    let base = remote_profile_path(profile);
    let content = ssh_read_file(config, &format!("{}/.env", base)).unwrap_or_default();
    let mut result = HashMap::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') || !trimmed.contains('=') { continue; }
        if let Some((k, v)) = trimmed.split_once('=') {
            result.insert(k.trim().to_string(), v.trim().trim_matches('"').to_string());
        }
    }
    Ok(result)
}

/// Remote: read remote API key
pub fn ssh_read_remote_api_key(config: &SshConfig) -> Result<String, String> {
    let env = ssh_read_env(config, None)?;
    Ok(env.get("API_SERVER_KEY").cloned().unwrap_or_default())
}

// ─── Tunnel Management ───────────────────────────────────

/// Check if a port on localhost is reachable.
fn check_port(port: u16) -> bool {
    TcpStream::connect_timeout(
        &format!("127.0.0.1:{}", port).parse().unwrap(),
        Duration::from_millis(500),
    ).is_ok()
}

/// Find a free local port.
fn find_free_port(preferred: u16) -> u16 {
    if !check_port(preferred) { return preferred; }
    for offset in 1..100 {
        let candidate = preferred + offset;
        if !check_port(candidate) { return candidate; }
    }
    preferred
}

/// Check tunnel health by hitting the /health endpoint.
async fn check_tunnel_health(port: u16) -> bool {
    let url = format!("http://127.0.0.1:{}/health", port);
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(3000))
        .build();
    match client {
        Ok(c) => match c.get(&url).send().await {
            Ok(r) => r.status().is_success(),
            Err(_) => false,
        },
        Err(_) => false,
    }
}

/// Wait for a port to open, polling up to timeout_ms.
fn wait_for_port(port: u16, timeout_ms: u64) -> Result<(), String> {
    let start = Instant::now();
    let timeout = Duration::from_millis(timeout_ms);
    loop {
        if check_port(port) { return Ok(()); }
        if start.elapsed() > timeout {
            return Err(format!("Timeout waiting for port {}", port));
        }
        std::thread::sleep(Duration::from_millis(200));
    }
}

#[tauri::command]
pub fn test_ssh_connection() -> Result<SshConnectionResult, String> {
    let conn = config::get_connection_config()?;
    let cfg = &conn.ssh;
    if cfg.host.is_empty() {
        return Ok(SshConnectionResult { success: false, message: "No SSH configuration found".into() });
    }

    let local_port = find_free_port(19642);
    let mut cmd = Command::new("ssh");
    cmd.args(&build_tunnel_args(cfg, local_port))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }

    let mut child = cmd.spawn().map_err(|e| format!("SSH spawn failed: {}", e))?;

    let start = Instant::now();
    let deadline = Duration::from_secs(15);
    let mut connected = false;

    while start.elapsed() < deadline {
        if check_port(local_port) {
            // Port open — quick health probe
            if let Ok(stream) = TcpStream::connect_timeout(
                &format!("127.0.0.1:{}", local_port).parse().unwrap(),
                Duration::from_millis(3000),
            ) {
                drop(stream);
                connected = true;
            }
            break;
        }
        std::thread::sleep(Duration::from_millis(400));
    }

    let _ = child.kill();
    let _ = child.wait();

    if connected {
        Ok(SshConnectionResult { success: true, message: format!("Connected to {}@{}:{}", cfg.username, cfg.host, cfg.port) })
    } else {
        Ok(SshConnectionResult { success: false, message: "SSH connection failed or timed out".into() })
    }
}

#[tauri::command]
pub fn start_ssh_tunnel(
    state: tauri::State<'_, crate::AppState>,
) -> Result<bool, String> {
    let conn = config::get_connection_config()?;
    let cfg = &conn.ssh;
    if cfg.host.is_empty() {
        return Err("No SSH configuration found".into());
    }

    // Stop any existing tunnel first
    stop_ssh_tunnel_internal(&state);

    let local_port = find_free_port(18642);

    let mut cmd = Command::new("ssh");
    cmd.args(&build_tunnel_args(cfg, local_port))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }

    let _child = cmd.spawn().map_err(|e| format!("SSH tunnel spawn failed: {}", e))?;

    // Wait for port to open
    if let Err(e) = wait_for_port(local_port, 12000) {
        return Err(e);
    }

    // Store tunnel state
    {
        let mut tunnel = state.ssh_tunnel_state.lock().map_err(|e| format!("Lock: {}", e))?;
        *tunnel = SshTunnelState::Connected {
            local_port,
            remote_host: cfg.host.clone(),
            remote_port: cfg.remote_port,
        };
    }

    Ok(true)
}

fn stop_ssh_tunnel_internal(state: &tauri::State<'_, crate::AppState>) {
    if let Ok(mut tunnel) = state.ssh_tunnel_state.lock() {
        *tunnel = SshTunnelState::Disconnected;
    }
}

#[tauri::command]
pub fn stop_ssh_tunnel(
    state: tauri::State<'_, crate::AppState>,
) -> Result<bool, String> {
    stop_ssh_tunnel_internal(&state);
    Ok(true)
}

#[tauri::command]
pub fn is_ssh_tunnel_active(
    state: tauri::State<'_, crate::AppState>,
) -> Result<bool, String> {
    let tunnel = state.ssh_tunnel_state.lock().map_err(|e| format!("Lock: {}", e))?;
    match &*tunnel {
        SshTunnelState::Connected { .. } => Ok(true),
        SshTunnelState::Disconnected => Ok(false),
    }
}

// ─── Health Check ────────────────────────────────────────

pub async fn is_ssh_tunnel_healthy(state: &tauri::State<'_, crate::AppState>) -> bool {
    let port = {
        let tunnel = match state.ssh_tunnel_state.lock() {
            Ok(t) => t,
            Err(_) => return false,
        };
        match &*tunnel {
            SshTunnelState::Connected { local_port, .. } => *local_port,
            SshTunnelState::Disconnected => return false,
        }
    };
    check_tunnel_health(port).await
}

pub async fn ensure_ssh_tunnel(state: &tauri::State<'_, crate::AppState>) -> Result<(), String> {
    let active = {
        let tunnel = state.ssh_tunnel_state.lock().map_err(|e| format!("Lock: {}", e))?;
        matches!(&*tunnel, SshTunnelState::Connected { .. })
    };

    if active && is_ssh_tunnel_healthy(state).await {
        return Ok(());
    }

    // Need to restart tunnel — start_ssh_tunnel will be called externally
    Ok(())
}
