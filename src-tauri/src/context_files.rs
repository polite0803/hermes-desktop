// Context files — AGENTS.md, CLAUDE.md, PROJECT.md editor
use serde::{Deserialize, Serialize};
use std::fs;
use crate::hermes_cli;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContextFile {
    pub name: String,
    pub content: String,
}

#[tauri::command]
pub fn list_context_files() -> Result<Vec<String>, String> {
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
    let safe = name.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|', '.'], "_");
    let path = hermes_cli::resolve_hermes_home().join(&safe);
    let content = if path.exists() { fs::read_to_string(&path).unwrap_or_default() } else { String::new() };
    Ok(ContextFile { name: safe, content })
}

#[tauri::command]
pub fn write_context_file(name: String, content: String) -> Result<(), String> {
    let safe = name.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
    let path = hermes_cli::resolve_hermes_home().join(&safe);
    if let Some(p) = path.parent() { fs::create_dir_all(p).map_err(|e| e.to_string())?; }
    fs::write(&path, &content).map_err(|e| e.to_string())
}
