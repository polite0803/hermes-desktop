// Context files — AGENTS.md, CLAUDE.md, PROJECT.md editor
use serde::{Deserialize, Serialize};
use std::fs;
use crate::{config, hermes_cli, ssh};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextFile {
    pub name: String,
    pub content: String,
}

#[tauri::command]
pub fn list_context_files() -> Result<Vec<String>, String> {
    let conn = config::get_connection_config_raw()?;
    if conn.mode == "ssh" {
        let cmd = ssh::build_remote_hermes_cmd(&["context", "list", "--json"]);
        let out = ssh::ssh_exec(&conn.ssh, &cmd, None, 10000).unwrap_or_default();
        if let Ok(arr) = serde_json::from_str::<Vec<serde_json::Value>>(&out) {
            return Ok(arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect());
        }
        return Ok(Vec::new());
    }
    let home = hermes_cli::resolve_hermes_home();
    let mut files = Vec::new();
    for name in &["AGENTS.md", "CLAUDE.md", "PROJECT.md", "CONTEXT.md", "RULES.md"] {
        let path = home.join(name);
        if path.exists() { files.push(name.to_string()); }
    }
    Ok(files)
}

#[tauri::command]
pub fn read_context_file(name: String) -> Result<ContextFile, String> {
    let conn = config::get_connection_config_raw()?;
    let safe = name.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|', '.'], "_");
    if conn.mode == "ssh" {
        let remote_path = format!("~/.hermes/{}", safe);
        let content = ssh::ssh_read_file(&conn.ssh, &remote_path).unwrap_or_default();
        return Ok(ContextFile { name: safe, content });
    }
    let path = hermes_cli::resolve_hermes_home().join(&safe);
    let content = if path.exists() { fs::read_to_string(&path).unwrap_or_default() } else { String::new() };
    Ok(ContextFile { name: safe, content })
}

#[tauri::command]
pub fn write_context_file(name: String, content: String) -> Result<(), String> {
    let conn = config::get_connection_config_raw()?;
    let safe = name.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
    if conn.mode == "ssh" {
        let remote_path = format!("~/.hermes/{}", safe);
        return ssh::ssh_write_file(&conn.ssh, &remote_path, &content);
    }
    let path = hermes_cli::resolve_hermes_home().join(&safe);
    if let Some(p) = path.parent() { fs::create_dir_all(p).map_err(|e| e.to_string())?; }
    fs::write(&path, &content).map_err(|e| e.to_string())
}
