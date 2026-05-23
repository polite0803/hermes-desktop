// Hermes chat engine — SSE streaming + Gateway management
// Rewrite from original src/main/hermes.ts (1,043 lines TS)
//
// Three connection modes:
//   local  → spawns Python hermes CLI, or talks to http://127.0.0.1:8642
//   remote → talks to user-configured remote URL
//   ssh    → SSH tunnel to remote host, then local API
//
// Public API path: local API first, CLI fallback when API is unreachable.

use std::collections::HashMap;
use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri::Emitter;

use crate::config;
use crate::hermes_cli;
use crate::ssh;

// ─── Lazy Init ──────────────────────────────────────────

static INIT_DONE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static API_AVAILABLE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static API_SERVER_CONFIG_DONE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn ensure_api_server_config() {
    if API_SERVER_CONFIG_DONE.swap(true, Ordering::SeqCst) { return; }
    if is_remote() { return; }
    let config_path = hermes_cli::resolve_hermes_home().join("config.yaml");
    if !config_path.exists() { return; }
    if let Ok(content) = std::fs::read_to_string(&config_path) {
        if content.contains("api_server") { return; }
        let addition = "\n# Desktop app API server (auto-configured)\nplatforms:\n  api_server:\n    enabled: true\n    extra:\n      port: 8642\n      host: \"127.0.0.1\"\n";
        let _ = std::fs::write(&config_path, format!("{}{}", content, addition));
    }
}

fn ensure_initialized() {
    if INIT_DONE.swap(true, Ordering::SeqCst) { return; }
    if !is_remote() { ensure_api_server_config(); }
}

// ─── Constants ───

const LOCAL_API_URL: &str = "http://127.0.0.1:8642";
const API_SERVER_ENABLED_KEY: &str = "API_SERVER_ENABLED";

/// Providers that don't need an API key (local / self-hosted)
const LOCAL_PROVIDERS: &[&str] = &["custom", "lmstudio", "ollama", "vllm", "llamacpp"];

/// Known API key env vars injected into CLI subprocess env.
const KNOWN_API_KEYS: &[&str] = &[
    "OPENROUTER_API_KEY", "OPENAI_API_KEY", "ANTHROPIC_API_KEY",
    "GROQ_API_KEY", "GLM_API_KEY", "KIMI_API_KEY", "MINIMAX_API_KEY",
    "MINIMAX_CN_API_KEY", "HF_TOKEN", "EXA_API_KEY", "PARALLEL_API_KEY",
    "TAVILY_API_KEY", "FIRECRAWL_API_KEY", "FAL_KEY", "HONCHO_API_KEY",
    "BROWSERBASE_API_KEY", "BROWSERBASE_PROJECT_ID", "VOICE_TOOLS_OPENAI_KEY",
    "TINKER_API_KEY", "WANDB_API_KEY",
];

/// Map base-URL patterns to the API key env var they need.
const URL_KEY_MAP: &[(&str, &str)] = &[
    ("openrouter.ai", "OPENROUTER_API_KEY"),
    ("anthropic.com", "ANTHROPIC_API_KEY"),
    ("openai.com", "OPENAI_API_KEY"),
    ("huggingface.co", "HF_TOKEN"),
    ("api.groq.com", "GROQ_API_KEY"),
    ("api.deepseek.com", "DEEPSEEK_API_KEY"),
    ("api.together.xyz", "TOGETHER_API_KEY"),
    ("api.fireworks.ai", "FIREWORKS_API_KEY"),
    ("api.cerebras.ai", "CEREBRAS_API_KEY"),
    ("api.mistral.ai", "MISTRAL_API_KEY"),
    ("api.perplexity.ai", "PERPLEXITY_API_KEY"),
];

// ─── State ───

#[derive(Debug)]
pub struct ChatState {
    pub cancel_token: Arc<AtomicBool>,
    // Cached SSH remote API key read from remote .env when tunnel starts
    pub ssh_remote_api_key: Arc<Mutex<String>>,
}

impl Default for ChatState {
    fn default() -> Self {
        Self {
            cancel_token: Arc::new(AtomicBool::new(false)),
            ssh_remote_api_key: Arc::new(Mutex::new(String::new())),
        }
    }
}

#[derive(Debug)]
pub enum GatewayState {
    NotRunning,
    Running(Child),
}

// ─── Data Types ───

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Usage {
    #[serde(rename = "promptTokens")]
    pub prompt_tokens: u32,
    #[serde(rename = "completionTokens")]
    pub completion_tokens: u32,
    #[serde(rename = "totalTokens")]
    pub total_tokens: u32,
    pub cost: Option<f64>,
    #[serde(rename = "rateLimitRemaining")]
    pub rate_limit_remaining: Option<u32>,
    #[serde(rename = "rateLimitReset")]
    pub rate_limit_reset: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SendResult {
    pub response: String,
    #[serde(rename = "sessionId")]
    pub session_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChatRequest {
    pub model: Option<String>,
    pub messages: Vec<ChatMessage>,
    pub stream: Option<bool>,
    pub session_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Attachment {
    pub kind: Option<String>,
    pub name: String,
    pub text: Option<String>,
    pub path: Option<String>,
    pub mime: Option<String>,
    #[serde(rename = "dataUrl")]
    pub data_url: Option<String>,
}

// ─── internal helpers ───

/// Normalise a user-entered remote URL: strip trailing slashes and /v1.
fn normalise_remote_url(raw: &str) -> String {
    let mut url = raw.trim().to_string();
    while url.ends_with('/') { url.pop(); }
    if url.to_lowercase().ends_with("/v1") {
        url.truncate(url.len() - 3);
        while url.ends_with('/') { url.pop(); }
    }
    url
}

/// Resolve the active API base URL based on connection mode.
fn resolve_api_url() -> Result<String, String> {
    let conn = config::get_connection_config_raw()?;
    match conn.mode.as_str() {
        "ssh" => {
            // SSH mode: API is at the tunnel's local endpoint
            Ok(LOCAL_API_URL.to_string())
        }
        "remote" => {
            let remote = if conn.remote_url.is_empty() { LOCAL_API_URL.to_string() } else { conn.remote_url.clone() };
            Ok(normalise_remote_url(&remote))
        }
        _ => Ok(LOCAL_API_URL.to_string()),
    }
}

fn is_remote() -> bool {
    config::get_connection_config_raw()
        .map(|c| c.mode == "remote" || c.mode == "ssh")
        .unwrap_or(false)
}

fn is_remote_only() -> bool {
    config::get_connection_config_raw()
        .map(|c| c.mode == "remote")
        .unwrap_or(false)
}

/// Build auth headers for the API request.
fn build_auth_headers(profile: Option<&str>, state: &ChatState) -> HashMap<String, String> {
    let conn = match config::get_connection_config_raw() {
        Ok(c) => c,
        Err(_) => return HashMap::new(),
    };
    let mut headers = HashMap::new();
    headers.insert("Content-Type".into(), "application/json".into());

    match conn.mode.as_str() {
        "ssh" => {
            if let Ok(key) = state.ssh_remote_api_key.lock() {
                if !key.is_empty() {
                    headers.insert("Authorization".into(), format!("Bearer {}", key));
                }
            }
        }
        "remote" => {
            if !conn.api_key.is_empty() {
                headers.insert("Authorization".into(), format!("Bearer {}", conn.api_key));
            }
        }
        _ => {
            // Local mode: optionally use API_SERVER_KEY from config
            let api_server_key = config::get_api_server_key(profile).unwrap_or_default();
            if !api_server_key.is_empty() {
                headers.insert("Authorization".into(), format!("Bearer {}", api_server_key));
            }
        }
    }

    headers
}

/// Build the user content payload (text + optional attachments).
fn build_user_content(text: &str, attachments: &[serde_json::Value]) -> serde_json::Value {
    if attachments.is_empty() {
        return serde_json::Value::String(text.to_string());
    }

    let text_files: Vec<_> = attachments.iter()
        .filter(|a| a.get("kind").and_then(|v| v.as_str()) == Some("text-file"))
        .collect();
    let path_refs: Vec<_> = attachments.iter()
        .filter(|a| {
            a.get("kind").and_then(|v| v.as_str()) == Some("path-ref")
                && a.get("path").and_then(|v| v.as_str()).map_or(false, |p| !p.is_empty())
        })
        .collect();
    let images: Vec<_> = attachments.iter()
        .filter(|a| {
            a.get("kind").and_then(|v| v.as_str()) == Some("image")
                && a.get("dataUrl").and_then(|v| v.as_str()).map_or(false, |d| !d.is_empty())
        })
        .collect();

    let mut parts: Vec<String> = Vec::new();
    if !text.trim().is_empty() { parts.push(text.to_string()); }

    for f in &text_files {
        let name = f.get("name").and_then(|v| v.as_str()).unwrap_or("file");
        let mime = f.get("mime").and_then(|v| v.as_str()).unwrap_or("text/plain");
        if let Some(file_text) = f.get("text").and_then(|v| v.as_str()) {
            parts.push(format!("<file name=\"{}\" mime=\"{}\">\n{}\n</file>", name, mime, file_text));
        }
    }

    if !path_refs.is_empty() {
        let lines: Vec<String> = path_refs.iter()
            .filter_map(|f| f.get("path").and_then(|v| v.as_str()))
            .map(|p| format!("[Attached file: {}]", p))
            .collect();
        parts.push(lines.join("\n"));
    }

    let composed = parts.join("\n\n");

    if images.is_empty() {
        if composed.is_empty() {
            return serde_json::Value::String(text.to_string());
        }
        return serde_json::Value::String(composed);
    }

    let mut content_parts: Vec<serde_json::Value> = Vec::new();
    if !composed.is_empty() {
        content_parts.push(serde_json::json!({"type": "text", "text": composed}));
    }
    for img in &images {
        if let Some(data_url) = img.get("dataUrl").and_then(|v| v.as_str()) {
            content_parts.push(serde_json::json!({"type": "image_url", "image_url": {"url": data_url}}));
        }
    }
    serde_json::Value::Array(content_parts)
}

/// Check whether the local API server is healthy.
async fn is_api_ready() -> bool {
    let api_url = match resolve_api_url() {
        Ok(u) => u,
        Err(_) => return false,
    };
    let health_url = format!("{}/health", api_url);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(1500))
        .build();
    match client {
        Ok(c) => match c.get(&health_url).send().await {
            Ok(r) => r.status().is_success(),
            Err(_) => false,
        },
        Err(_) => false,
    }
}

/// Escape XML attribute values (basic implementation).
fn escape_xml_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

// ─── Gateway Management ───

#[tauri::command]
pub fn start_gateway(
    app: AppHandle,
    state: tauri::State<'_, crate::AppState>,
) -> Result<bool, String> {
    ensure_initialized();
    // Refuse to spawn local gateway in remote/SSH mode
    if is_remote() {
        log::warn!("[gateway] startGateway() called in remote/SSH mode — refusing local spawn");
        return Ok(false);
    }

    let mut gw = state.gateway_state.lock().map_err(|e| format!("Lock: {}", e))?;
    if matches!(&*gw, GatewayState::Running(_)) {
        return Ok(false);
    }

    let python = hermes_cli::resolve_python();
    let script = hermes_cli::resolve_hermes_script();
    let hermes_home = hermes_cli::resolve_hermes_home();

    // Build gateway env with all profile API keys
    let mut env_vars = Vec::new();
    if let Ok(profile_env) = config::get_env_all_raw(None) {
        for (k, v) in profile_env {
            if !v.is_empty() {
                env_vars.push((k, v));
            }
        }
    }
    env_vars.push(("HERMES_HOME".into(), hermes_home.to_string_lossy().to_string()));
    env_vars.push(("API_SERVER_ENABLED".into(), "true".into()));

    let mut cmd = Command::new(&python);
    cmd.arg(&script).arg("gateway")
        .envs(env_vars)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let child = cmd.spawn().map_err(|e| format!("Failed to start gateway: {}", e))?;

    let _ = app.emit("chat-gateway-started", "gateway.started");

    *gw = GatewayState::Running(child);
    Ok(true)
}

#[tauri::command]
pub fn stop_gateway(state: tauri::State<'_, crate::AppState>) -> Result<bool, String> {
    let conn = config::get_connection_config_raw()?;
    if conn.mode == "ssh" {
        ssh::ssh_stop_gateway(&conn.ssh)?;
        return Ok(true);
    }
    let mut gw = state.gateway_state.lock().map_err(|e| format!("Lock: {}", e))?;

    let old = std::mem::replace(&mut *gw, GatewayState::NotRunning);
    match old {
        GatewayState::Running(mut child) => {
            let _ = child.kill();
            let _ = child.wait();
        }
        GatewayState::NotRunning => {}
    }

    // Also try to kill via PID file
    let hermes_home = hermes_cli::resolve_hermes_home();
    let pid_file = hermes_home.join("gateway.pid");
    if pid_file.exists() {
        if let Ok(raw) = std::fs::read_to_string(&pid_file) {
            let pid: Option<u32> = raw.trim().parse().ok()
                .or_else(|| {
                    serde_json::from_str::<serde_json::Value>(&raw).ok()
                        .and_then(|v| v.get("pid").and_then(|p| p.as_u64()).map(|p| p as u32))
                });
            if let Some(pid) = pid {
                #[cfg(unix)]
                unsafe { libc::kill(pid as i32, libc::SIGTERM); }
                #[cfg(windows)]
                {
                    let _ = std::process::Command::new("taskkill")
                        .args(&["/PID", &pid.to_string(), "/F"])
                        .output();
                }
            }
        }
        let _ = std::fs::remove_file(&pid_file);
    }

    Ok(true)
}

#[tauri::command]
pub fn gateway_status(state: tauri::State<'_, crate::AppState>) -> Result<bool, String> {
    let conn = config::get_connection_config_raw()?;
    if conn.mode == "ssh" {
        return ssh::ssh_gateway_status(&conn.ssh);
    }
    let gw = state.gateway_state.lock().map_err(|e| format!("Lock: {}", e))?;

    match &*gw {
        GatewayState::Running(_child) => {
            // Child exists in state, so consider it running
            Ok(true)
        }
        GatewayState::NotRunning => {
            // Check PID file as fallback
            let hermes_home = hermes_cli::resolve_hermes_home();
            let pid_file = hermes_home.join("gateway.pid");
            if pid_file.exists() {
                Ok(true) // PID file exists, likely running
            } else {
                Ok(false)
            }
        }
    }
}

// ─── OpenAI-compatible proxy ──────────────────────────────

#[tauri::command]
pub fn start_proxy(app: AppHandle) -> Result<bool, String> {
    let python = hermes_cli::resolve_python();
    let script = hermes_cli::resolve_hermes_script();
    if !script.exists() { return Err("hermes.notInstalled".into()); }
    let mut cmd = std::process::Command::new(&python);
    cmd.arg(&script).arg("proxy").stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null());
    #[cfg(windows)] { use std::os::windows::process::CommandExt; cmd.creation_flags(0x08000000); }
    cmd.spawn().map_err(|e| format!("{}", e))?;
    let _ = app.emit("proxy-started", ());
    Ok(true)
}

// ─── SSE Streaming API ───

/// Send via HTTP API with SSE streaming.
async fn send_via_api(
    app: &AppHandle,
    state: &ChatState,
    message: &str,
    profile: Option<&str>,
    resume_session_id: Option<&str>,
    history: Option<&[ChatMessage]>,
    attachments: Option<&[serde_json::Value]>,
) -> Result<SendResult, String> {
    let api_url = resolve_api_url()?;
    let chat_url = format!("{}/v1/chat/completions", api_url);

    // Build messages
    let mut messages: Vec<serde_json::Value> = Vec::new();
    if let Some(hist) = history {
        for msg in hist {
            let role = if msg.role == "agent" { "assistant" } else { msg.role.as_str() };
            messages.push(serde_json::json!({"role": role, "content": msg.content}));
        }
    }
    let user_content = build_user_content(message, attachments.unwrap_or(&[]));
    messages.push(serde_json::json!({"role": "user", "content": user_content}));

    // Get model config
    let mc = config::get_model_config_raw(profile).unwrap_or_else(|_| config::ModelConfig::default());

    let mut body = serde_json::json!({
        "model": mc.model,
        "messages": messages,
        "stream": true,
    });
    if let Some(sid) = resume_session_id {
        body["session_id"] = serde_json::json!(sid);
    }

    let headers = build_auth_headers(profile, state);
    let cancel_token = state.cancel_token.clone();
    let cancel_token_stream = cancel_token.clone();

    let client = reqwest::Client::new();
    let mut req = client.post(&chat_url)
        .json(&body)
        .timeout(Duration::from_secs(120));

    for (k, v) in &headers {
        req = req.header(k.as_str(), v.as_str());
    }

    let response = req.send().await.map_err(|e| {
        if e.is_timeout() {
            "API request timed out. Check the SSH tunnel and remote Hermes gateway.".into()
        } else {
            format!("API request failed: {}", e)
        }
    })?;

    if !response.status().is_success() {
        let _status = response.status();
        let err_body = response.text().await.unwrap_or_default();
        if let Ok(err) = serde_json::from_str::<serde_json::Value>(&err_body) {
            let _msg = err["error"]["message"].as_str().unwrap_or(&err_body);
            return Err("hermes.apiError".into());
        }
        return Err("hermes.apiServerError".into());
    }

    // SSE streaming
    let mut buffer = String::new();
    let mut full_content = String::new();
    let mut session_id: Option<String> = resume_session_id.map(|s| s.to_string());
    let mut has_content = false;
    let mut last_error = String::new();

    use futures_util::StreamExt;
    let mut stream = response.bytes_stream();

    // Tool progress regex: `emoji tool_name`
    let tool_re = regex_lite::Regex::new(r"^`([^\s`]+)\s+([^`]+)`$").ok();

    while let Some(chunk_result) = stream.next().await {
        if cancel_token_stream.load(Ordering::SeqCst) {
            break;
        }
        match chunk_result {
            Ok(chunk) => {
                let text = String::from_utf8_lossy(&chunk);
                buffer.push_str(&text);

                // Split by SSE double-newline delimiter
                while let Some(pos) = buffer.find("\n\n") {
                    let block = buffer[..pos].to_string();
                    buffer = buffer[pos + 2..].to_string();

                    process_sse_block(
                        &block, app,
                        &mut full_content, &mut session_id,
                        &mut has_content, &mut last_error,
                        tool_re.as_ref(),
                    );
                }
            }
            Err(_) => {
                return Err("hermes.streamError".into());
            }
        }
    }

    // Flush remaining buffer
    if !buffer.trim().is_empty() {
        for part in buffer.split("\n\n") {
            process_sse_block(
                part, app,
                &mut full_content, &mut session_id,
                &mut has_content, &mut last_error,
                tool_re.as_ref(),
            );
        }
    }

    if !has_content && last_error.is_empty() {
        // Streaming returned empty — try non-streaming probe
        return probe_non_streaming(&chat_url, &body, &headers).await;
    }

    let sid = session_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let _ = app.emit("chat-done", serde_json::json!({
        "sessionId": sid, "content": full_content
    }));

    Ok(SendResult { response: full_content, session_id: Some(sid) })
}

/// Process a single SSE block (event: + data: line pair).
fn process_sse_block(
    block: &str,
    app: &AppHandle,
    full_content: &mut String,
    _session_id: &mut Option<String>,
    has_content: &mut bool,
    last_error: &mut String,
    tool_re: Option<&regex_lite::Regex>,
) {
    let mut event_type = String::new();
    let mut data_line = String::new();

    for line in block.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("event: ") {
            event_type = trimmed[7..].trim().to_string();
        } else if trimmed.starts_with("data: ") {
            data_line = trimmed[6..].to_string();
        }
    }

    if data_line.is_empty() { return; }

    if !event_type.is_empty() {
        // Custom event (e.g. hermes.tool.progress)
        if event_type == "hermes.tool.progress" {
            if let Ok(payload) = serde_json::from_str::<serde_json::Value>(&data_line) {
                let label = payload["label"].as_str()
                    .or(payload["tool"].as_str())
                    .unwrap_or("");
                let emoji = payload["emoji"].as_str().unwrap_or("");
                let tool_str = if emoji.is_empty() { label.to_string() } else { format!("{} {}", emoji, label) };
                let _ = app.emit("chat-tool-progress", &tool_str);
            }
        }
        return;
    }

    // Standard data line
    if data_line == "[DONE]" {
        if *has_content || !last_error.is_empty() {
            return; // handled by caller
        }
        // probe_real_error will be handled by caller
        return;
    }

    match serde_json::from_str::<serde_json::Value>(&data_line) {
        Ok(parsed) => {
            if let Some(err) = parsed["error"].as_object() {
                *last_error = err["message"].as_str().unwrap_or("Unknown error").to_string();
                return;
            }

            // Usage
            if let Some(usage) = parsed.get("usage") {
                let _ = app.emit("chat-usage", serde_json::json!({
                    "promptTokens": usage["prompt_tokens"].as_u64().unwrap_or(0),
                    "completionTokens": usage["completion_tokens"].as_u64().unwrap_or(0),
                    "totalTokens": usage["total_tokens"].as_u64().unwrap_or(0),
                    "cost": usage["cost"].as_f64(),
                    "rateLimitRemaining": usage["rate_limit_remaining"].as_u64(),
                    "rateLimitReset": usage["rate_limit_reset"].as_u64(),
                }));
            }

            let choice = parsed["choices"].as_array().and_then(|c| c.first());
            if let Some(delta_content) = choice.and_then(|c| c["delta"]["content"].as_str()) {
                let content = delta_content.trim();

                // Tool progress detection in content
                if let Some(ref re) = tool_re {
                    if let Some(caps) = re.captures(content) {
                        let tool_str = format!("{} {}", &caps[1], &caps[2]);
                        let _ = app.emit("chat-tool-progress", &tool_str);
                    } else {
                        *has_content = true;
                        let _ = app.emit("chat-chunk", delta_content);
                        full_content.push_str(delta_content);
                    }
                } else {
                    *has_content = true;
                    let _ = app.emit("chat-chunk", delta_content);
                    full_content.push_str(delta_content);
                }
            }
        }
        Err(_) => { /* malformed chunk, skip */ }
    }
}

/// Non-streaming probe when streaming returns empty.
async fn probe_non_streaming(
    chat_url: &str,
    body: &serde_json::Value,
    headers: &HashMap<String, String>,
) -> Result<SendResult, String> {
    let mut probe_body = body.clone();
    probe_body["stream"] = serde_json::json!(false);

    let client = reqwest::Client::new();
    let mut req = client.post(chat_url).json(&probe_body);

    for (k, v) in headers {
        if k != "Content-Type" {
            req = req.header(k.as_str(), v.as_str());
        }
    }

    let response = req.send().await.map_err(|e| format!("Probe failed: {}", e))?;
    let text = response.text().await.unwrap_or_default();

    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
        if let Some(choices) = parsed["choices"].as_array() {
            if let Some(content) = choices.first().and_then(|c| c["message"]["content"].as_str()) {
                let sid = uuid::Uuid::new_v4().to_string();
                return Ok(SendResult { response: content.to_string(), session_id: Some(sid) });
            }
        }
        if let Some(err_msg) = parsed["error"]["message"].as_str() {
            return Err(err_msg.to_string());
        }
    }

    Err("hermes.noResponse".into())
}

// ─── CLI fallback ───

async fn send_via_cli(
    app: &AppHandle,
    message: &str,
    profile: Option<&str>,
    resume_session_id: Option<&str>,
    attachments: Option<&[serde_json::Value]>,
) -> Result<SendResult, String> {
    let python = hermes_cli::resolve_python();
    let script = hermes_cli::resolve_hermes_script();
    let hermes_home = hermes_cli::resolve_hermes_home();
    let mc = config::get_model_config_raw(profile).unwrap_or_else(|_| config::ModelConfig::default());

    // Inline text-file attachments into message text
    let mut message_text = message.to_string();
    if let Some(atts) = attachments {
        if !atts.is_empty() {
            let text_file_wraps: Vec<String> = atts.iter()
                .filter(|a| a.get("kind").and_then(|v| v.as_str()) == Some("text-file"))
                .filter_map(|a| {
                    let name = a.get("name").and_then(|v| v.as_str()).unwrap_or("file");
                    let mime = a.get("mime").and_then(|v| v.as_str()).unwrap_or("text/plain");
                    a.get("text").and_then(|v| v.as_str()).map(|file_text| {
                        format!("<file name=\"{}\" mime=\"{}\">\n{}\n</file>",
                            escape_xml_attr(name), escape_xml_attr(mime), file_text)
                    })
                })
                .collect();
            if !text_file_wraps.is_empty() {
                let wrapped = text_file_wraps.join("\n\n");
                if message_text.trim().is_empty() {
                    message_text = wrapped;
                } else {
                    message_text = format!("{}\n\n{}", message_text, wrapped);
                }
            }
        }
    }

    let mut args = vec!["chat".to_string(), "-q".to_string(), message_text, "-Q".to_string(),
        "--source".to_string(), "desktop".to_string()];

    if let Some(p) = profile {
        if p != "default" {
            args.push("-p".into());
            args.push(p.to_string());
        }
    }
    if let Some(sid) = resume_session_id {
        args.push("--resume".into());
        args.push(sid.to_string());
    }
    if !mc.model.is_empty() {
        args.push("-m".into());
        args.push(mc.model);
    }

    let mut cmd = Command::new(&python);
    cmd.arg(&script).args(&args)
        .env("HERMES_HOME", hermes_home.to_string_lossy().as_ref())
        .env("PYTHONUNBUFFERED", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }

    let mut child = cmd.spawn().map_err(|e| format!("Failed to spawn CLI: {}", e))?;

    let mut stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();

    let mut full_output = String::new();
    let mut buf = [0u8; 4096];
    let mut captured_sid = String::new();

    loop {
        match stdout.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                let text = String::from_utf8_lossy(&buf[..n]);
                let cleaned = strip_ansi(&text);

                // Extract session_id
                if let Some(sid_start) = cleaned.find("session_id:") {
                    let rest = &cleaned[sid_start + 11..];
                    if let Some(end) = rest.find(|c: char| c.is_whitespace()) {
                        captured_sid = rest[..end].to_string();
                    }
                }

                // Filter noise lines
                let filtered: String = cleaned.lines()
                    .filter(|line| {
                        let t = line.trim();
                        if t.is_empty() { return true; }
                        // Skip box-drawing and emoji header lines
                        !t.starts_with('╭') && !t.starts_with('╰') && !t.starts_with('│')
                            && !t.starts_with('╮') && !t.starts_with('╯')
                            && !t.starts_with("⚕")
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                if !filtered.trim().is_empty() {
                    let _ = app.emit("chat-chunk", &filtered);
                    full_output.push_str(&filtered);
                    full_output.push('\n');
                }
            }
            Err(_) => break,
        }
    }

    // Read stderr for errors
    let mut stderr_text = String::new();
    let _ = stderr.read_to_string(&mut stderr_text);

    let status = child.wait().map_err(|e| format!("Wait failed: {}", e))?;
    let sid = if captured_sid.is_empty() {
        uuid::Uuid::new_v4().to_string()
    } else {
        captured_sid
    };

    if status.success() || !full_output.trim().is_empty() {
        let _ = app.emit("chat-done", serde_json::json!({
            "sessionId": sid, "content": full_output
        }));
        Ok(SendResult { response: full_output, session_id: Some(sid) })
    } else {
        let detail = stderr_text.trim();
        let msg = if detail.is_empty() {
            format!("Hermes exited with code {:?}", status.code())
        } else {
            format!("Hermes exited with code {:?}: {}", status.code(), detail)
        };
        Err(msg)
    }
}

fn strip_ansi(text: &str) -> String {
    let re = regex_lite::Regex::new(r"\x1b\[[0-9;]*m").ok();
    match re {
        Some(r) => r.replace_all(text, "").to_string(),
        None => text.to_string(),
    }
}

// ─── Public command ───

#[tauri::command]
pub async fn send_message(
    app: AppHandle,
    state: tauri::State<'_, crate::AppState>,
    message: String,
    profile: Option<String>,
    resume_session_id: Option<String>,
    history: Option<Vec<ChatMessage>>,
    attachments: Option<Vec<serde_json::Value>>,
) -> Result<SendResult, String> {
    ensure_initialized();
    // Reset cancel token and extract the shared state
    let (cancel_tok, ssh_key) = {
        let chat = state.chat_state.lock().map_err(|e| format!("Lock: {}", e))?;
        chat.cancel_token.store(false, Ordering::SeqCst);
        (chat.cancel_token.clone(), chat.ssh_remote_api_key.clone())
    };

    let profile = profile.as_deref();
    let resume_sid = resume_session_id.as_deref();
    let synthetic_state = ChatState { cancel_token: cancel_tok, ssh_remote_api_key: ssh_key };

    if is_remote() {
        send_via_api(&app, &synthetic_state, &message, profile, resume_sid,
            history.as_deref(), attachments.as_deref()).await
    } else {
        let api_ready = is_api_ready().await;
        if api_ready {
            send_via_api(&app, &synthetic_state, &message, profile, resume_sid,
                history.as_deref(), attachments.as_deref()).await
        } else {
            send_via_cli(&app, &message, profile, resume_sid, attachments.as_deref()).await
        }
    }
}

#[tauri::command]
pub fn abort_chat(state: tauri::State<'_, crate::AppState>) -> Result<(), String> {
    let chat = state.chat_state.lock().map_err(|e| format!("Lock: {}", e))?;
    chat.cancel_token.store(true, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
pub async fn test_remote_connection(url: String, api_key: Option<String>) -> Result<bool, String> {
    let target = format!("{}/health", normalise_remote_url(&url));

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| format!("Client error: {}", e))?;

    let mut req = client.get(&target);
    if let Some(ref key) = api_key {
        if !key.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", key));
        }
    }

    match req.send().await {
        Ok(r) => Ok(r.status().is_success()),
        Err(_) => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalise_remote_url() {
        assert_eq!(normalise_remote_url("http://example.com"), "http://example.com");
        assert_eq!(normalise_remote_url("http://example.com/"), "http://example.com");
        assert_eq!(normalise_remote_url("http://example.com/v1"), "http://example.com");
        assert_eq!(normalise_remote_url("http://example.com/v1/"), "http://example.com");
        assert_eq!(normalise_remote_url("  http://host/v1  "), "http://host");
    }

    #[test]
    fn test_build_user_content_plain() {
        let result = build_user_content("hello", &[]);
        assert_eq!(result, serde_json::json!("hello"));
    }

    #[test]
    fn test_build_user_content_empty_text_with_image() {
        let atts = vec![serde_json::json!({
            "kind": "image",
            "name": "photo.png",
            "dataUrl": "data:image/png;base64,abc"
        })];
        let result = build_user_content("", &atts);
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["type"], "image_url");
    }

    #[test]
    fn test_build_user_content_path_refs() {
        let atts = vec![serde_json::json!({
            "kind": "path-ref",
            "name": "doc.pdf",
            "path": "C:/staging/doc.pdf"
        })];
        let result = build_user_content("summarize this", &atts);
        let s = result.as_str().unwrap();
        assert!(s.contains("Attached file: C:/staging/doc.pdf"));
        assert!(s.contains("summarize this"));
    }

    #[test]
    fn test_headers_auth_local() {
        // Without a valid connection config this returns empty headers plus Content-Type
        let chat_state = ChatState::default();
        let headers = build_auth_headers(None, &chat_state);
        assert!(headers.contains_key("Content-Type"));
    }

    #[test]
    fn test_strip_ansi() {
        assert_eq!(strip_ansi("\x1b[32mgreen\x1b[0m"), "green");
        assert_eq!(strip_ansi("normal text"), "normal text");
    }
}
