use std::fs;
use std::path::PathBuf;

use crate::hermes_cli;

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
    let path = soul_path(profile.as_deref());
    if path.exists() {
        fs::read_to_string(&path).map_err(|e| format!("Failed to read SOUL.md: {}", e))
    } else {
        Ok(DEFAULT_SOUL.to_string())
    }
}

#[tauri::command]
pub fn write_soul(content: String, profile: Option<String>) -> Result<(), String> {
    let path = soul_path(profile.as_deref());
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create dir: {}", e))?;
    }
    fs::write(&path, &content).map_err(|e| format!("Failed to write SOUL.md: {}", e))
}

#[tauri::command]
pub fn reset_soul(profile: Option<String>) -> Result<(), String> {
    let path = soul_path(profile.as_deref());
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create dir: {}", e))?;
    }
    fs::write(&path, DEFAULT_SOUL).map_err(|e| format!("Failed to reset SOUL.md: {}", e))
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
