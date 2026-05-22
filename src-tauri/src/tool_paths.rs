// External tool paths — git, npm configuration
use std::fs;
use std::path::PathBuf;
use crate::hermes_cli;

fn settings_path() -> PathBuf { hermes_cli::resolve_hermes_home().join("desktop.json") }

fn read_setting(key: &str) -> String {
    let path = settings_path();
    if !path.exists() { return String::new(); }
    fs::read_to_string(&path).ok()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        .and_then(|v| v.get(key).and_then(|v| v.as_str()).map(|s| s.to_string()))
        .unwrap_or_default()
}

fn write_setting(key: &str, value: &str) -> Result<(), String> {
    let path = settings_path();
    let mut config = if path.exists() {
        fs::read_to_string(&path).ok()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
            .unwrap_or(serde_json::json!({}))
    } else { serde_json::json!({}) };
    config[key] = serde_json::json!(value);
    if let Some(p) = path.parent() { fs::create_dir_all(p).map_err(|e| e.to_string())?; }
    fs::write(&path, serde_json::to_string_pretty(&config).unwrap_or_default()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_tool_paths() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "git": read_setting("gitPath"),
        "npm": read_setting("npmPath"),
    }))
}

#[tauri::command]
pub fn set_tool_paths(git: Option<String>, npm: Option<String>) -> Result<bool, String> {
    if let Some(g) = git { write_setting("gitPath", &g)?; }
    if let Some(n) = npm { write_setting("npmPath", &n)?; }
    Ok(true)
}

pub fn resolve_git_path() -> PathBuf {
    let custom = read_setting("gitPath");
    if !custom.is_empty() { return PathBuf::from(custom); }
    let h = hermes_cli::resolve_hermes_home().join("git");
    for sub in &[vec!["bin", "git.exe"], vec!["cmd", "git.exe"], vec!["usr", "bin", "git.exe"]] {
        let p: PathBuf = sub.iter().fold(h.clone(), |acc, s| acc.join(s));
        if p.exists() { return p; }
    }
    PathBuf::from("git")
}

pub fn resolve_npm_path() -> PathBuf {
    let custom = read_setting("npmPath");
    if !custom.is_empty() { return PathBuf::from(custom); }
    let n = hermes_cli::resolve_hermes_home().join("node");
    for name in &["npm.cmd", "npm"] {
        let p = n.join(name);
        if p.exists() { return p; }
    }
    PathBuf::from("npm")
}
