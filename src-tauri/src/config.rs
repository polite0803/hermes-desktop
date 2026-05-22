// Configuration management — desktop.json / config.yaml / .env / auth.json
// Full rewrite from original src/main/config.ts (1,179 lines TS)
//
// Subsystems:
//   1. Desktop JSON (connection config) — desktop.json
//   2. TTL Cache (5s) — HashMap<String, (Instant, serde_json::Value)>
//   3. .env file management — read all / set one / validate
//   4. YAML navigation — hand-rolled line-by-line path resolution
//   5. Model config — read/write model: block in config.yaml
//   6. API Server Key — multi-source resolution
//   7. Platform toggles — env-check + config.yaml override
//   8. Credential pool — auth.json read/write + OAuth detection

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::hermes_cli;

// ─── TTL Cache ──────────────────────────────────────────

const CACHE_TTL_MS: u64 = 5000;

static CACHE: once_cell::sync::Lazy<Mutex<HashMap<String, (Instant, serde_json::Value)>>> =
    once_cell::sync::Lazy::new(|| Mutex::new(HashMap::new()));

fn cache_get(key: &str) -> Option<serde_json::Value> {
    let cache = CACHE.lock().ok()?;
    let (ts, ref val) = cache.get(key)?;
    if ts.elapsed() > Duration::from_millis(CACHE_TTL_MS) {
        drop(cache);
        cache_invalidate(key);
        return None;
    }
    Some(val.clone())
}

fn cache_set(key: &str, val: serde_json::Value) {
    if let Ok(mut cache) = CACHE.lock() {
        cache.insert(key.to_string(), (Instant::now(), val));
    }
}

fn cache_invalidate(prefix: &str) {
    if let Ok(mut cache) = CACHE.lock() {
        cache.retain(|k, _| !k.starts_with(prefix));
    }
}

// ─── Data Structures ─────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SshConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    #[serde(rename = "keyPath")]
    pub key_path: String,
    #[serde(rename = "remotePort")]
    pub remote_port: u16,
    #[serde(rename = "localPort")]
    pub local_port: u16,
}

impl Default for SshConfig {
    fn default() -> Self {
        Self {
            host: String::new(), port: 22, username: String::new(),
            key_path: String::new(), remote_port: 8642, local_port: 18642,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionConfig {
    pub mode: String,
    #[serde(rename = "remoteUrl")]
    pub remote_url: String,
    #[serde(rename = "apiKey")]
    pub api_key: String,
    pub ssh: SshConfig,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            mode: "local".into(), remote_url: String::new(),
            api_key: String::new(), ssh: SshConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicConnectionConfig {
    pub mode: String,
    #[serde(rename = "remoteUrl")]
    pub remote_url: String,
    #[serde(rename = "hasApiKey")]
    pub has_api_key: bool,
    #[serde(rename = "apiKeyLength")]
    pub api_key_length: usize,
    pub ssh: SshConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ModelConfig {
    pub provider: String,
    pub model: String,
    #[serde(rename = "baseUrl")]
    pub base_url: String,
}

// ─── Path helpers ────────────────────────────────────────

fn desktop_config_path() -> PathBuf {
    hermes_cli::resolve_hermes_home().join("desktop.json")
}

fn config_yaml_path(profile: Option<&str>) -> PathBuf {
    hermes_cli::resolve_profile_home(profile).join("config.yaml")
}

fn env_file_path(profile: Option<&str>) -> PathBuf {
    hermes_cli::resolve_profile_home(profile).join(".env")
}

fn auth_json_path(profile: Option<&str>) -> PathBuf {
    hermes_cli::resolve_profile_home(profile).join("auth.json")
}

// ─── Desktop JSON ────────────────────────────────────────

fn read_desktop_config() -> Result<serde_json::Value, String> {
    let path = desktop_config_path();
    if !path.exists() { return Ok(serde_json::json!({})); }
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read desktop.json: {}", e))?;
    serde_json::from_str(&content).map_err(|e| format!("Invalid desktop.json: {}", e))
}

fn write_desktop_config(value: &serde_json::Value) -> Result<(), String> {
    let path = desktop_config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create dir: {}", e))?;
    }
    let content = serde_json::to_string_pretty(value)
        .map_err(|e| format!("Failed to serialize: {}", e))?;
    fs::write(&path, content).map_err(|e| format!("Failed to write desktop.json: {}", e))
}

// ─── Connection Config ───────────────────────────────────

pub fn get_connection_config_raw() -> Result<ConnectionConfig, String> {
    let data = read_desktop_config()?;
    let ssh = data.get("sshConfig").cloned().unwrap_or(serde_json::json!({}));
    Ok(ConnectionConfig {
        mode: data.get("connectionMode").and_then(|v| v.as_str()).unwrap_or("local").to_string(),
        remote_url: data.get("remoteUrl").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        api_key: data.get("remoteApiKey").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        ssh: SshConfig {
            host: ssh.get("host").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            port: ssh.get("port").and_then(|v| v.as_u64()).unwrap_or(22) as u16,
            username: ssh.get("username").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            key_path: ssh.get("keyPath").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            remote_port: ssh.get("remotePort").and_then(|v| v.as_u64()).unwrap_or(8642) as u16,
            local_port: ssh.get("localPort").and_then(|v| v.as_u64()).unwrap_or(18642) as u16,
        },
    })
}

#[tauri::command]
pub fn get_connection_config() -> Result<ConnectionConfig, String> {
    get_connection_config_raw()
}

#[tauri::command]
pub fn set_connection_config(mode: String, remote_url: String, api_key: Option<String>) -> Result<bool, String> {
    let mut data = read_desktop_config()?;
    data["connectionMode"] = serde_json::json!(mode);
    data["remoteUrl"] = serde_json::json!(remote_url);
    data["remoteApiKey"] = serde_json::json!(api_key.unwrap_or_default());
    if mode == "ssh" {
        // SSH config is set via setSshConfig → calls this with mode="ssh" + full config object
        // For the simple positional call, we preserve existing SSH config
    }
    write_desktop_config(&data)?;
    Ok(true)
}

#[tauri::command]
pub fn is_remote_mode() -> Result<bool, String> {
    let c = get_connection_config_raw()?;
    Ok(c.mode == "remote" || c.mode == "ssh")
}

#[tauri::command]
pub fn is_remote_only_mode() -> Result<bool, String> {
    let c = get_connection_config_raw()?;
    Ok(c.mode == "remote")
}

// ─── YAML Navigation (hand-rolled, no serde_yaml dependency for path ops) ──

struct YamlPathHit {
    value: String,
    value_start: usize,
    value_end: usize,
}

fn strip_yaml_quotes(raw: &str) -> String {
    let t = raw.trim();
    if (t.starts_with('"') && t.ends_with('"')) || (t.starts_with('\'') && t.ends_with('\'')) {
        t[1..t.len()-1].to_string()
    } else {
        t.to_string()
    }
}

fn escape_regex(s: &str) -> String {
    regex_lite::escape(s)
}

fn find_segment_in_block(
    content: &str,
    start_at: usize,
    parent_indent: isize,
    segment: &str,
) -> Option<(usize, String, usize, usize, usize)> {
    let escaped = escape_regex(segment);
    let mut direct_child_indent: Option<usize> = None;
    let mut cursor = start_at;

    while cursor < content.len() {
        let rest = &content[cursor..];
        let line_end = rest.find('\n').map(|p| cursor + p).unwrap_or(content.len());
        let line = &content[cursor..line_end];
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            cursor = if line_end < content.len() { line_end + 1 } else { content.len() };
            continue;
        }

        let indent = line.len() - line.trim_start().len();
        if indent as isize <= parent_indent { return None; }

        if direct_child_indent.is_none() { direct_child_indent = Some(indent); }

        if indent == direct_child_indent.unwrap() {
            let pattern = format!(r"^([ \t]*)({}):([ \t]*)([^\n#]*?)([ \t]*)(#.*)?$", escaped);
            if let Ok(re) = regex_lite::Regex::new(&pattern) {
                if let Some(caps) = re.captures(line) {
                    let indent_len = caps.get(1).map(|m| m.end() - m.start()).unwrap_or(0);
                    let gap = caps.get(3).map(|m| m.end() - m.start()).unwrap_or(0);
                    let raw_value = caps.get(4).map(|m| m.as_str()).unwrap_or("");
                    let key_end = cursor + indent_len + segment.len() + 1;
                    let value_start = key_end + gap;
                    let value_end = value_start + raw_value.len();
                    let after_line = if line_end < content.len() { line_end + 1 } else { content.len() };
                    return Some((indent_len, raw_value.to_string(), value_start, value_end, after_line));
                }
            }
        }
        cursor = if line_end < content.len() { line_end + 1 } else { content.len() };
    }
    None
}

fn find_yaml_path(content: &str, dotted_path: &str) -> Option<YamlPathHit> {
    let segments: Vec<&str> = dotted_path.split('.').filter(|s| !s.is_empty()).collect();
    if segments.is_empty() { return None; }

    let mut cursor = 0usize;
    let mut parent_indent: isize = -1;

    for (i, segment) in segments.iter().enumerate() {
        let is_last = i == segments.len() - 1;
        let found = find_segment_in_block(content, cursor, parent_indent, segment)?;
        if is_last {
            return Some(YamlPathHit {
                value: strip_yaml_quotes(&found.1),
                value_start: found.2,
                value_end: found.3,
            });
        }
        cursor = found.4;
        parent_indent = found.0 as isize;
    }
    None
}

fn find_top_level_key(content: &str, key: &str) -> Option<YamlPathHit> {
    let pattern = format!(r"(?m)^({}):([ \t]*)([^\n#]*?)([ \t]*)(#.*)?$", escape_regex(key));
    let re = regex_lite::Regex::new(&pattern).ok()?;
    let caps = re.captures(content)?;
    let full = caps.get(0)?;
    let line_start = full.start();
    let key_str = caps.get(1)?;
    let gap = caps.get(2).map(|m| m.as_str()).unwrap_or("");
    let raw_value = caps.get(3).map(|m| m.as_str()).unwrap_or("");
    let value_start = line_start + key_str.as_str().len() + 1 + gap.len();
    let value_end = value_start + raw_value.len();
    Some(YamlPathHit {
        value: strip_yaml_quotes(raw_value),
        value_start,
        value_end,
    })
}

// ─── Config Value Read/Write ─────────────────────────────

#[tauri::command]
pub fn get_config_value(key: String, profile: Option<String>) -> Result<Option<String>, String> {
    let path = config_yaml_path(profile.as_deref());
    if !path.exists() { return Ok(None); }
    let content = fs::read_to_string(&path).map_err(|e| format!("Failed to read config.yaml: {}", e))?;

    let segments: Vec<&str> = key.split('.').filter(|s| !s.is_empty()).collect();
    let hit = if segments.len() == 1 {
        find_top_level_key(&content, segments[0])
    } else {
        find_yaml_path(&content, &key)
    };
    Ok(hit.map(|h| h.value))
}

#[tauri::command]
pub fn set_config_value(key: String, value: String, profile: Option<String>) -> Result<bool, String> {
    if key == "API_SERVER_KEY" { cache_invalidate("apiServerKey:"); }
    let path = config_yaml_path(profile.as_deref());
    if !path.exists() { return Ok(false); }

    let mut content = fs::read_to_string(&path).map_err(|e| format!("Failed to read config.yaml: {}", e))?;
    let segments: Vec<&str> = key.split('.').filter(|s| !s.is_empty()).collect();
    if segments.is_empty() { return Ok(false); }

    let hit = if segments.len() == 1 {
        find_top_level_key(&content, segments[0])
    } else {
        find_yaml_path(&content, &key)
    };

    if let Some(h) = hit {
        let quoted = format!("\"{}\"", value);
        content.replace_range(h.value_start..h.value_end, &quoted);
    } else if segments.len() == 1 {
        let sep = if content.ends_with('\n') || content.is_empty() { "" } else { "\n" };
        content = format!("{}{}{}: \"{}\"\n", content, sep, key, value);
    }
    // Multi-segment new paths: silently dropped (no guessing where to insert)

    fs::write(&path, &content).map_err(|e| format!("Failed to write config.yaml: {}", e))?;
    Ok(true)
}

// ─── .env File Management ────────────────────────────────

struct EnvKeyRe;
impl EnvKeyRe {
    fn test(key: &str) -> bool {
        !key.is_empty() && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
            && key.chars().next().map_or(false, |c| c.is_ascii_alphabetic() || c == '_')
    }
}

pub fn get_env_all_raw(profile: Option<&str>) -> Result<HashMap<String, String>, String> {
    let cache_key = format!("env:{}", profile.unwrap_or("default"));
    if let Some(cached) = cache_get(&cache_key) {
        if let Some(map) = cached.as_object() {
            return Ok(map.iter().map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string())).collect());
        }
    }

    let path = env_file_path(profile);
    let mut result = HashMap::new();
    if !path.exists() { return Ok(result); }

    let content = fs::read_to_string(&path).map_err(|e| format!("Failed to read .env: {}", e))?;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') || !trimmed.contains('=') { continue; }
        if let Some((k, v)) = trimmed.split_once('=') {
            let key = k.trim().to_string();
            let mut val = v.trim().to_string();
            if (val.starts_with('"') && val.ends_with('"')) || (val.starts_with('\'') && val.ends_with('\'')) {
                val = val[1..val.len()-1].to_string();
            }
            result.insert(key, val);
        }
    }

    let cache_val = serde_json::to_value(&result).unwrap_or_default();
    cache_set(&cache_key, cache_val);
    Ok(result)
}

#[tauri::command]
pub fn get_env_all(profile: Option<String>) -> Result<HashMap<String, String>, String> {
    get_env_all_raw(profile.as_deref())
}

#[tauri::command]
pub fn set_env(key: String, value: String, profile: Option<String>) -> Result<bool, String> {
    if !EnvKeyRe::test(&key) {
        return Err("config.invalidEnvKeyName".into());
    }
    if value.contains('\0') || value.contains('\r') || value.contains('\n') {
        return Err("config.envValueMustBeSingleLine".into());
    }

    cache_invalidate(&format!("env:{}", profile.as_deref().unwrap_or("default")));
    if key == "API_SERVER_KEY" { cache_invalidate("apiServerKey:"); }

    let path = env_file_path(profile.as_deref());
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create dir: {}", e))?;
    }

    if !path.exists() {
        fs::write(&path, format!("{}={}\n", key, value))
            .map_err(|e| format!("Failed to write .env: {}", e))?;
        return Ok(true);
    }

    let content = fs::read_to_string(&path).map_err(|e| format!("Failed to read .env: {}", e))?;
    let escaped = escape_regex(&key);
    let line_re = regex_lite::Regex::new(&format!(r"^#?\s*{}\s*=", escaped)).ok();

    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let mut found = false;
    for line in lines.iter_mut() {
        if let Some(ref re) = line_re {
            if re.is_match(line) {
                *line = format!("{}={}", key, value);
                found = true;
                break;
            }
        }
    }
    if !found { lines.push(format!("{}={}", key, value)); }

    fs::write(&path, lines.join("\n")).map_err(|e| format!("Failed to write .env: {}", e))?;
    Ok(true)
}

#[tauri::command]
pub fn get_hermes_home() -> String {
    hermes_cli::resolve_hermes_home().to_string_lossy().to_string()
}

// ─── Model Config ────────────────────────────────────────

pub fn get_model_config_raw(profile: Option<&str>) -> Result<ModelConfig, String> {
    let cache_key = format!("mc:{}", profile.unwrap_or("default"));
    if let Some(cached) = cache_get(&cache_key) {
        return serde_json::from_value(cached).map_err(|e| e.to_string());
    }

    let path = config_yaml_path(profile);
    let defaults = ModelConfig { provider: "auto".into(), model: String::new(), base_url: String::new() };
    if !path.exists() { return Ok(defaults); }

    let content = fs::read_to_string(&path).map_err(|e| format!("Failed to read config.yaml: {}", e))?;
    let children = read_top_level_block_children(&content, "model");

    let result = ModelConfig {
        provider: children.get("provider").cloned().unwrap_or_else(|| defaults.provider.clone()),
        model: children.get("default").cloned().unwrap_or_else(|| defaults.model.clone()),
        base_url: children.get("base_url").cloned().unwrap_or_else(|| defaults.base_url.clone()),
    };

    cache_set(&cache_key, serde_json::to_value(&result).unwrap_or_default());
    Ok(result)
}

fn read_top_level_block_children(content: &str, block_name: &str) -> HashMap<String, String> {
    let mut result = HashMap::new();
    let pattern = format!(r"(?m)^{}:[ \t]*\r?\n", escape_regex(block_name));
    let re = match regex_lite::Regex::new(&pattern) {
        Ok(r) => r,
        Err(_) => return result,
    };
    let start = match re.find(content) {
        Some(m) => m.end(),
        None => return result,
    };

    let rest = &content[start..];
    let child_re = regex_lite::Regex::new(r"^([ \t]+)([A-Za-z_][A-Za-z0-9_-]*):([ \t]*)([^\n#]*?)([ \t]*)(#.*)?$").ok();
    let mut first_indent: Option<String> = None;

    for line in rest.lines() {
        if line.trim().is_empty() { continue; }
        if !line.starts_with(' ') && !line.starts_with('\t') { break; } // next top-level key

        if let Some(ref re) = child_re {
            if let Some(caps) = re.captures(line) {
                let indent = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                let key = caps.get(2).map(|m| m.as_str()).unwrap_or("");
                let raw_val = caps.get(4).map(|m| m.as_str()).unwrap_or("");

                if first_indent.is_none() { first_indent = Some(indent.to_string()); }
                if indent == first_indent.as_deref().unwrap_or("") && !result.contains_key(key) {
                    result.insert(key.to_string(), strip_yaml_quotes(raw_val));
                }
            }
        }
    }
    result
}

#[tauri::command]
pub fn get_model_config(profile: Option<String>) -> Result<ModelConfig, String> {
    get_model_config_raw(profile.as_deref())
}

#[tauri::command]
pub fn set_model_config(provider: String, model: String, base_url: String, profile: Option<String>) -> Result<bool, String> {
    cache_invalidate(&format!("mc:{}", profile.as_deref().unwrap_or("default")));
    let path = config_yaml_path(profile.as_deref());

    let mut content = if path.exists() {
        fs::read_to_string(&path).map_err(|e| format!("Failed to read config.yaml: {}", e))?
    } else {
        String::new()
    };

    content = upsert_block_child(&content, "model", "provider", &provider);
    content = upsert_block_child(&content, "model", "default", &model);
    if !base_url.is_empty() {
        content = upsert_block_child(&content, "model", "base_url", &base_url);
    }

    // Enable streaming
    let streaming_re = regex_lite::Regex::new(r"(?m)^(\s*streaming:\s*)(\S+)").ok();
    if let Some(re) = streaming_re {
        if re.is_match(&content) {
            content = re.replace(&content, "${1}true").to_string();
        }
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create dir: {}", e))?;
    }
    fs::write(&path, &content).map_err(|e| format!("Failed to write config.yaml: {}", e))?;
    Ok(true)
}

fn upsert_block_child(content: &str, block_name: &str, key: &str, value: &str) -> String {
    let children = read_top_level_block_children(content, block_name);
    if children.contains_key(key) {
        return replace_block_child(content, block_name, key, value);
    }

    // Append to existing block or create new block
    let block_pattern = format!(r"(?m)^{}:[ \t]*\r?\n", escape_regex(block_name));
    if let Ok(re) = regex_lite::Regex::new(&block_pattern) {
        if let Some(m) = re.find(content) {
            let insert_at = m.end();
            let insertion = format!("  {}: \"{}\"\n", key, value);
            return format!("{}{}{}", &content[..insert_at], insertion, &content[insert_at..]);
        }
    }

    // No block at all — append one
    let sep = if content.is_empty() || content.ends_with('\n') { "" } else { "\n" };
    format!("{}{}{}:\n  {}: \"{}\"\n", content, sep, block_name, key, value)
}

fn replace_block_child(content: &str, block_name: &str, key: &str, value: &str) -> String {
    // Find the block body start
    let block_pattern = format!(r"(?m)^{}:[ \t]*\r?\n", escape_regex(block_name));
    let re = match regex_lite::Regex::new(&block_pattern) {
        Ok(r) => r,
        Err(_) => return content.to_string(),
    };
    let block_start = match re.find(content) {
        Some(m) => m.end(),
        None => return content.to_string(),
    };

    // Find the child and replace its value
    let rest = &content[block_start..];
    let escaped_key = escape_regex(key);
    let child_pattern = format!(r"(?m)^([ \t]+){}:([ \t]*)([^\n#]*)", escaped_key);
    let child_re = match regex_lite::Regex::new(&child_pattern) {
        Ok(r) => r,
        Err(_) => return content.to_string(),
    };

    if let Some(caps) = child_re.captures(rest) {
        let full_match = caps.get(0).unwrap();
        let absolut_start = block_start + full_match.start();
        let absolut_end = block_start + full_match.end();
        let indent = caps.get(1).map(|m| m.as_str()).unwrap_or("  ");
        let new_line = format!("{}{}: \"{}\"", indent, key, value);
        return format!("{}{}{}", &content[..absolut_start], new_line, &content[absolut_end..]);
    }

    content.to_string()
}

// ─── API Server Key ──────────────────────────────────────

pub fn get_api_server_key(profile: Option<&str>) -> Result<String, String> {
    let cache_key = format!("apiServerKey:{}", profile.unwrap_or("default"));
    if let Some(cached) = cache_get(&cache_key) {
        return Ok(cached.as_str().unwrap_or("").to_string());
    }

    // Search order: profile config → default config → profile .env → default .env
    let candidates: Vec<Option<String>> = vec![
        get_config_value("API_SERVER_KEY".into(), profile.map(|s| s.to_string())).ok().flatten(),
        if profile.is_some() { get_config_value("API_SERVER_KEY".into(), None).ok().flatten() } else { None },
        get_env_all_raw(profile).ok().and_then(|e| e.get("API_SERVER_KEY").cloned()),
        if profile.is_some() { get_env_all_raw(None).ok().and_then(|e| e.get("API_SERVER_KEY").cloned()) } else { None },
    ];

    let value = candidates.into_iter().flatten().find(|v| !v.trim().is_empty()).unwrap_or_default();
    cache_set(&cache_key, serde_json::json!(value));
    Ok(value)
}

// ─── Platform Toggles ────────────────────────────────────

const SUPPORTED_PLATFORMS: &[(&str, &str, &str)] = &[
    // (key, env_var, config_key)
    ("telegram", "TELEGRAM_BOT_TOKEN", "telegram"),
    ("discord", "DISCORD_BOT_TOKEN", "discord"),
    ("slack", "SLACK_BOT_TOKEN", "slack"),
    ("whatsapp", "WHATSAPP_ENABLED", "whatsapp"),
    ("signal", "SIGNAL_HTTP_URL", "signal"),
    ("matrix", "MATRIX_ACCESS_TOKEN", "matrix"),
    ("mattermost", "MATTERMOST_TOKEN", "mattermost"),
    ("home_assistant", "HASS_TOKEN", "homeassistant"),
];

fn read_platform_override(content: &str, config_key: &str) -> Option<bool> {
    let block_start_re = regex_lite::Regex::new(&format!(r"(?m)^{}:[ \t]*\r?\n", escape_regex(config_key))).ok()?;
    let start_match = block_start_re.find(content)?;
    let after = &content[start_match.end()..];

    for line in after.lines() {
        if line.trim().is_empty() { continue; }
        if !line.starts_with(' ') && !line.starts_with('\t') { break; }
        if let Ok(enabled_re) = regex_lite::Regex::new(r"^[ \t]+enabled:[ \t]*(true|false)\b") {
            if let Some(caps) = enabled_re.captures(line) {
                return Some(caps.get(1).map(|m| m.as_str()) == Some("true"));
            }
        }
    }
    None
}

#[tauri::command]
pub fn get_platform_enabled_all(profile: Option<String>) -> Result<HashMap<String, bool>, String> {
    let env = get_env_all_raw(profile.as_deref()).unwrap_or_default();
    let path = config_yaml_path(profile.as_deref());
    let content = if path.exists() {
        fs::read_to_string(&path).unwrap_or_default()
    } else {
        String::new()
    };

    let mut result = HashMap::new();
    for (platform, env_var, config_key) in SUPPORTED_PLATFORMS {
        let env_enabled = match *platform {
            "whatsapp" => {
                let val = env.get(*env_var).map(|v| v.to_lowercase()).unwrap_or_default();
                val == "true" || val == "1" || val == "yes" || val == "on"
            }
            "signal" => {
                env.contains_key("SIGNAL_HTTP_URL") && env.contains_key("SIGNAL_ACCOUNT")
            }
            "matrix" => {
                env.contains_key("MATRIX_ACCESS_TOKEN") || env.contains_key("MATRIX_PASSWORD")
            }
            _ => env.get(*env_var).map_or(false, |v| !v.trim().is_empty()),
        };
        let override_val = if !content.is_empty() {
            read_platform_override(&content, config_key)
        } else {
            None
        };
        result.insert(platform.to_string(), env_enabled && override_val != Some(false));
    }
    Ok(result)
}

#[tauri::command]
pub fn set_platform_enabled(platform: String, enabled: bool, profile: Option<String>) -> Result<bool, String> {
    let config_key = match platform.as_str() {
        "home_assistant" => "homeassistant",
        other => other,
    };
    let path = config_yaml_path(profile.as_deref());

    if !path.exists() {
        if enabled { return Ok(true); }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create dir: {}", e))?;
        }
        fs::write(&path, format!("{}:\n  enabled: false\n", config_key))
            .map_err(|e| format!("Failed to write config.yaml: {}", e))?;
        return Ok(true);
    }

    let mut content = fs::read_to_string(&path).map_err(|e| format!("Failed to read config.yaml: {}", e))?;

    // Check for flow-style empty block: `platform: {}`
    let flow_re = regex_lite::Regex::new(&format!(r"(?m)^{}:[ \t]*\{{\s*\}}[ \t]*$", escape_regex(config_key))).ok();
    if let Some(ref re) = flow_re {
        if re.is_match(&content) {
            content = re.replace(&content, format!("{}:\n  enabled: {}", config_key, enabled)).to_string();
            fs::write(&path, &content).map_err(|e| format!("Failed to write config.yaml: {}", e))?;
            return Ok(true);
        }
    }

    // Check for existing block with enabled line
    let block_re = regex_lite::Regex::new(&format!(r"(?m)^{}:[ \t]*\r?\n", escape_regex(config_key))).ok();
    if let Some(ref re) = block_re {
        if let Some(block_match) = re.find(&content) {
            let block_start = block_match.end();
            let rest = &content[block_start..];

            // Look for existing `enabled: true/false` in the block
            let enabled_line_re = regex_lite::Regex::new(r"^([ \t]+enabled:[ \t]*)(true|false)\b([ \t]*)$").ok();
            let mut existing_line_start: Option<usize> = None;
            let mut existing_line_end: Option<usize> = None;
            let mut offset = 0usize;

            for line in rest.lines() {
                let line_len = line.len() + 1;
                if line.trim().is_empty() { offset += line_len; continue; }
                if !line.starts_with(' ') && !line.starts_with('\t') { break; }
                if let Some(ref e_re) = enabled_line_re {
                    if let Some(caps) = e_re.captures(line) {
                        let full_match = caps.get(0).unwrap();
                        existing_line_start = Some(block_start + offset + full_match.start());
                        existing_line_end = Some(block_start + offset + full_match.end());
                        break;
                    }
                }
                offset += line_len;
            }

            if let (Some(ls), Some(le)) = (existing_line_start, existing_line_end) {
                if enabled {
                    // Remove the disable override line
                    let remove_end = if content.as_bytes().get(le) == Some(&b'\n') { le + 1 } else { le };
                    content = format!("{}{}", &content[..ls], &content[remove_end..]);
                } else {
                    // Replace the value
                    content.replace_range(ls..le, &format!("  enabled: false"));
                }
            } else if !enabled {
                // Insert `enabled: false` as first child
                content = format!("{}  enabled: false\n{}", &content[..block_start], &content[block_start..]);
            }

            fs::write(&path, &content).map_err(|e| format!("Failed to write config.yaml: {}", e))?;
            return Ok(true);
        }
    }

    // No block at all — create one only for disabling
    if !enabled {
        let sep = if content.ends_with('\n') { "" } else { "\n" };
        content = format!("{}{}{}:\n  enabled: false\n", content, sep, config_key);
        fs::write(&path, &content).map_err(|e| format!("Failed to write config.yaml: {}", e))?;
    }

    Ok(true)
}

// ─── Credential Pool ─────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CredentialEntry {
    pub key: Option<String>,
    pub label: Option<String>,
}

fn read_auth_store(profile: Option<&str>) -> Result<serde_json::Value, String> {
    let path = auth_json_path(profile);
    if !path.exists() { return Ok(serde_json::json!({})); }
    let content = fs::read_to_string(&path).map_err(|e| format!("Failed to read auth.json: {}", e))?;
    serde_json::from_str(&content).unwrap_or(Ok(serde_json::json!({})))
}

fn write_auth_store(store: &serde_json::Value, profile: Option<&str>) -> Result<(), String> {
    let path = auth_json_path(profile);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create dir: {}", e))?;
    }
    let content = serde_json::to_string_pretty(store).map_err(|e| format!("Serialize error: {}", e))?;
    fs::write(&path, content).map_err(|e| format!("Failed to write auth.json: {}", e))
}

#[tauri::command]
pub fn get_credential_pool(profile: Option<String>) -> Result<serde_json::Value, String> {
    let store = read_auth_store(profile.as_deref())?;
    Ok(store.get("credential_pool").cloned().unwrap_or(serde_json::json!({})))
}

#[tauri::command]
pub fn set_credential_pool(provider: String, entries: Vec<CredentialEntry>, profile: Option<String>) -> Result<bool, String> {
    let mut store = read_auth_store(profile.as_deref())?;
    if store.get("credential_pool").is_none() {
        store["credential_pool"] = serde_json::json!({});
    }
    store["credential_pool"][&provider] = serde_json::to_value(&entries).unwrap_or_default();
    write_auth_store(&store, profile.as_deref())?;
    Ok(true)
}

// ─── OAuth Credentials Detection ─────────────────────────

#[tauri::command]
pub fn has_oauth_credentials(provider: String, profile: Option<String>) -> Result<bool, String> {
    let clean = provider.trim();
    if clean.is_empty() { return Ok(false); }

    let stores = vec![
        read_auth_store(profile.as_deref()).unwrap_or_default(),
    ];
    let mut default_added = false;
    if profile.as_deref().map_or(false, |p| p != "default") {
        default_added = true;
    }
    let stores: Vec<_> = if default_added {
        let mut s = stores;
        s.push(read_auth_store(None).unwrap_or_default());
        s
    } else {
        stores
    };

    for store in &stores {
        // providers[provider].access_token / refresh_token / api_key
        if let Some(providers) = store.get("providers").and_then(|v| v.as_object()) {
            if let Some(entry) = providers.get(clean) {
                let has_token = entry.get("access_token").and_then(|v| v.as_str()).map_or(false, |v| !v.trim().is_empty())
                    || entry.get("refresh_token").and_then(|v| v.as_str()).map_or(false, |v| !v.trim().is_empty())
                    || entry.get("api_key").and_then(|v| v.as_str()).map_or(false, |v| !v.trim().is_empty());
                if has_token { return Ok(true); }
            }
        }

        // credential_pool[provider]
        if let Some(pool) = store.get("credential_pool").and_then(|v| v.as_object()) {
            if let Some(entries) = pool.get(clean).and_then(|v| v.as_array()) {
                for entry in entries {
                    let has = entry.get("api_key").and_then(|v| v.as_str()).map_or(false, |v| !v.trim().is_empty())
                        || entry.get("access_token").and_then(|v| v.as_str()).map_or(false, |v| !v.trim().is_empty())
                        || entry.get("refresh_token").and_then(|v| v.as_str()).map_or(false, |v| !v.trim().is_empty());
                    if has { return Ok(true); }
                }
            }
        }
    }

    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_yaml_quotes() {
        assert_eq!(strip_yaml_quotes("\"hello\""), "hello");
        assert_eq!(strip_yaml_quotes("'world'"), "world");
        assert_eq!(strip_yaml_quotes("plain"), "plain");
    }

    #[test]
    fn test_escape_regex() {
        let escaped = escape_regex("api.server");
        assert!(escaped.contains("\\."));
    }

    #[test]
    fn test_find_top_level_key() {
        let content = "model: gpt-4\nprovider: openai\n";
        let hit = find_top_level_key(content, "model").unwrap();
        assert_eq!(hit.value, "gpt-4");
    }

    #[test]
    fn test_find_top_level_key_with_quotes() {
        let content = "name: \"My Profile\"\n";
        let hit = find_top_level_key(content, "name").unwrap();
        assert_eq!(hit.value, "My Profile");
    }

    #[test]
    fn test_find_yaml_path_nested() {
        let content = "agent:\n  service_tier: premium\n  name: test\n";
        let hit = find_yaml_path(content, "agent.service_tier").unwrap();
        assert_eq!(hit.value, "premium");
    }

    #[test]
    fn test_find_yaml_path_deep() {
        let content = "memory:\n  provider: honcho\n  settings:\n    ttl: 3600\n";
        let hit = find_yaml_path(content, "memory.settings.ttl").unwrap();
        assert_eq!(hit.value, "3600");
    }

    #[test]
    fn test_find_yaml_path_missing() {
        let content = "model: gpt-4\n";
        assert!(find_yaml_path(content, "model.provider").is_none());
    }

    #[test]
    fn test_read_top_level_block_children() {
        let content = "model:\n  provider: openai\n  default: gpt-4\n  base_url: https://api.openai.com\n";
        let children = read_top_level_block_children(content, "model");
        assert_eq!(children.get("provider").unwrap(), "openai");
        assert_eq!(children.get("default").unwrap(), "gpt-4");
        assert_eq!(children.get("base_url").unwrap(), "https://api.openai.com");
    }

    #[test]
    fn test_env_key_validation() {
        assert!(EnvKeyRe::test("OPENAI_API_KEY"));
        assert!(EnvKeyRe::test("MY_VAR2"));
        assert!(!EnvKeyRe::test("2INVALID"));
        assert!(!EnvKeyRe::test(""));
    }

    #[test]
    fn test_cache_set_and_get() {
        cache_set("test_key", serde_json::json!("test_value"));
        let val = cache_get("test_key");
        assert_eq!(val.unwrap().as_str().unwrap(), "test_value");
        cache_invalidate("test_");
    }

    #[test]
    fn test_cache_invalidate() {
        cache_set("prefix:key1", serde_json::json!(1));
        cache_set("other:key2", serde_json::json!(2));
        cache_invalidate("prefix:");
        assert!(cache_get("prefix:key1").is_none());
        assert!(cache_get("other:key2").is_some());
        cache_invalidate("other:");
    }

    #[test]
    fn test_model_config_default() {
        let cfg = ModelConfig::default();
        assert_eq!(cfg.provider, "");
    }

    #[test]
    fn test_ssh_config_default() {
        let cfg = SshConfig::default();
        assert_eq!(cfg.port, 22);
    }

    #[test]
    fn test_connection_config_default() {
        let cfg = ConnectionConfig::default();
        assert_eq!(cfg.mode, "local");
    }
}
