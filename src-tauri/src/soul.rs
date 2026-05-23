use std::fs;
use std::path::PathBuf;

use crate::{config, hermes_cli, ssh};

const DEFAULT_SOUL: &str = r#"You are Hermes, a helpful AI assistant focused on helping the user achieve their goals. You have access to various tools and capabilities.

## Core Principles
1. Be helpful, honest, and harmless
2. Use available tools when appropriate
3. Ask clarifying questions when needed
4. Admit when you don't know something
5. Think step by step for complex problems

## Tools
You have access to web search, file operations, code execution, and more through your tool system. Use them when they would help the user.
"#;

fn soul_path(profile: Option<&str>) -> PathBuf {
    hermes_cli::resolve_profile_home(profile).join("SOUL.md")
}

#[tauri::command]
pub fn read_soul(profile: Option<String>) -> Result<String, String> {
    let conn = config::get_connection_config_raw()?;
    if conn.mode == "ssh" {
        return ssh::ssh_read_soul(&conn.ssh, profile.as_deref());
    }
    let path = soul_path(profile.as_deref());
    if path.exists() {
        fs::read_to_string(&path).map_err(|e| format!("Failed to read SOUL.md: {}", e))
    } else {
        Ok(DEFAULT_SOUL.to_string())
    }
}

#[tauri::command]
pub fn write_soul(content: String, profile: Option<String>) -> Result<(), String> {
    let conn = config::get_connection_config_raw()?;
    if conn.mode == "ssh" {
        return ssh::ssh_write_soul(&conn.ssh, profile.as_deref(), &content);
    }
    let path = soul_path(profile.as_deref());
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create dir: {}", e))?;
    }
    fs::write(&path, &content).map_err(|e| format!("Failed to write SOUL.md: {}", e))
}

#[tauri::command]
pub fn reset_soul(profile: Option<String>) -> Result<(), String> {
    let conn = config::get_connection_config_raw()?;
    if conn.mode == "ssh" {
        return ssh::ssh_write_soul(&conn.ssh, profile.as_deref(), "");
    }
    let path = soul_path(profile.as_deref());
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create dir: {}", e))?;
    }
    fs::write(&path, DEFAULT_SOUL).map_err(|e| format!("Failed to reset SOUL.md: {}", e))
}

#[tauri::command]
pub fn list_personalities(_profile: Option<String>) -> Result<Vec<serde_json::Value>, String> {
    let personalities_dir = hermes_cli::resolve_hermes_home().join("personalities");
    if !personalities_dir.exists() { return Ok(Vec::new()); }

    let mut presets = Vec::new();
    if let Ok(entries) = fs::read_dir(&personalities_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().map_or(true, |e| e != "yaml" && e != "yml") { continue; }
            let name = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
            if let Ok(content) = fs::read_to_string(&path) {
                let desc = content.lines().find(|l| l.trim().starts_with("description:")).and_then(|l| l.split_once(':')).map(|(_,v)| v.trim().trim_matches('"').trim_matches('\'').to_string()).unwrap_or_default();
                presets.push(serde_json::json!({"name": name, "description": desc}));
            }
        }
    }
    Ok(presets)
}

#[tauri::command]
pub fn apply_personality(name: String, profile: Option<String>) -> Result<(), String> {
    let preset_path = hermes_cli::resolve_hermes_home().join("personalities").join(format!("{}.yaml", name));
    if !preset_path.exists() { return Err("soul.personalityNotFound".into()); }
    let content = fs::read_to_string(&preset_path).map_err(|e| format!("Failed: {}", e))?;
    write_soul(content, profile)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_soul_not_empty() {
        assert!(!DEFAULT_SOUL.is_empty());
        assert!(DEFAULT_SOUL.contains("Hermes"));
    }
}
