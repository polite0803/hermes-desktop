// Sandbox/terminal backend configuration
use std::fs;
use crate::hermes_cli;

fn config_path() -> std::path::PathBuf {
    hermes_cli::resolve_hermes_home().join("config.yaml")
}

#[tauri::command]
pub fn get_terminal_backend() -> Result<String, String> {
    let path = config_path();
    if !path.exists() { return Ok("local".into()); }
    let content = fs::read_to_string(&path).unwrap_or_default();
    let mut in_section = false;
    for line in content.lines() {
        if line.trim_start().starts_with("terminal_backend:") { in_section = true; continue; }
        if in_section {
            if !line.starts_with(' ') && !line.starts_with('\t') && !line.trim().is_empty() { break; }
            if let Some(v) = line.trim_start().strip_prefix("type:") {
                return Ok(v.trim().trim_matches('"').trim_matches('\'').to_string());
            }
        }
    }
    Ok("local".into())
}

#[tauri::command]
pub fn set_terminal_backend(backend: String, _config_json: Option<String>) -> Result<bool, String> {
    let path = config_path();
    let mut content = if path.exists() { fs::read_to_string(&path).map_err(|e| format!("Failed: {}", e))? } else { String::new() };
    let section = format!("terminal_backend:\n  type: {}\n", backend);
    if content.contains("terminal_backend:") {
        let re = regex_lite::Regex::new(r"(?s)terminal_backend:.*?(\n\S|\z)").map_err(|e| e.to_string())?;
        content = re.replace(&content, format!("{}\n", section.trim_end())).to_string();
    } else {
        content = format!("{}\n{}", content.trim_end(), section);
    }
    fs::write(&path, &content).map_err(|e| format!("Failed: {}", e))?;
    Ok(true)
}
