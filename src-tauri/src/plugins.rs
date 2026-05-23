// Plugin management — list, enable, disable hermes-agent plugins
use serde::{Deserialize, Serialize};
use std::fs;
use crate::{config, hermes_cli, ssh};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PluginInfo {
    pub name: String,
    pub description: String,
    pub installed: bool,
    pub enabled: bool,
}

#[tauri::command]
pub fn list_plugins() -> Result<Vec<PluginInfo>, String> {
    let conn = config::get_connection_config_raw()?;
    if conn.mode == "ssh" {
        let cmd = ssh::build_remote_hermes_cmd(&["plugins", "list", "--json"]);
        let out = ssh::ssh_exec(&conn.ssh, &cmd, None, 10000).unwrap_or_default();
        let raw: Vec<serde_json::Value> = serde_json::from_str(&out).unwrap_or_default();
        return Ok(raw.iter().map(|v| PluginInfo {
            name: v.get("name").and_then(|s| s.as_str()).unwrap_or("").to_string(),
            description: v.get("description").and_then(|s| s.as_str()).unwrap_or("").to_string(),
            installed: v.get("installed").and_then(|b| b.as_bool()).unwrap_or(true),
            enabled: v.get("enabled").and_then(|b| b.as_bool()).unwrap_or(true),
        }).collect());
    }
    let plugins_dir = hermes_cli::resolve_hermes_home().join("hermes-agent").join("plugins");
    if !plugins_dir.exists() { return Ok(Vec::new()); }

    let mut plugins = Vec::new();
    if let Ok(entries) = fs::read_dir(&plugins_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_dir() || path.file_name().unwrap_or_default().to_string_lossy().starts_with('_') || path.file_name().unwrap_or_default().to_string_lossy().starts_with('.') { continue; }
            let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
            let installed = path.join("__init__.py").exists();

            // Read plugin metadata from config.yaml if available
            let enabled = true; // default
            let description = String::new();

            plugins.push(PluginInfo { name, description, installed, enabled });
        }
    }
    Ok(plugins)
}

#[tauri::command]
pub fn enable_plugin(name: String) -> Result<(), String> {
    let conn = config::get_connection_config_raw()?;
    if conn.mode == "ssh" {
        let cmd = ssh::build_remote_hermes_cmd(&["plugins", "enable", &name]);
        ssh::ssh_exec(&conn.ssh, &cmd, None, 10000)?;
        return Ok(());
    }
    hermes_cli::run_hermes_cli(&["plugins", "enable", &name], None)?;
    Ok(())
}

#[tauri::command]
pub fn disable_plugin(name: String) -> Result<(), String> {
    let conn = config::get_connection_config_raw()?;
    if conn.mode == "ssh" {
        let cmd = ssh::build_remote_hermes_cmd(&["plugins", "disable", &name]);
        ssh::ssh_exec(&conn.ssh, &cmd, None, 10000)?;
        return Ok(());
    }
    hermes_cli::run_hermes_cli(&["plugins", "disable", &name], None)?;
    Ok(())
}
