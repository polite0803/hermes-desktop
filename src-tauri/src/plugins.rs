// Plugin management — list, enable, disable hermes-agent plugins
use serde::{Deserialize, Serialize};
use std::fs;
use crate::hermes_cli;

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
    hermes_cli::run_hermes_cli(&["plugins", "enable", &name], None)?;
    Ok(())
}

#[tauri::command]
pub fn disable_plugin(name: String) -> Result<(), String> {
    hermes_cli::run_hermes_cli(&["plugins", "disable", &name], None)?;
    Ok(())
}
