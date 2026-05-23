// Profile management — list, create, delete, set active, with directory scanning
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use crate::{config, hermes_cli, ssh};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub name: String, pub path: String,
    #[serde(rename = "isDefault")] pub is_default: bool,
    #[serde(rename = "isActive")] pub is_active: bool,
    pub model: String, pub provider: String,
    #[serde(rename = "hasEnv")] pub has_env: bool,
    #[serde(rename = "hasSoul")] pub has_soul: bool,
    #[serde(rename = "skillCount")] pub skill_count: u32,
    #[serde(rename = "gatewayRunning")] pub gateway_running: bool,
}

fn profiles_dir() -> PathBuf { hermes_cli::resolve_hermes_home().join("profiles") }
fn active_profile_name() -> String {
    hermes_cli::resolve_hermes_home().join("desktop.json").exists()
        .then(|| {
            fs::read_to_string(hermes_cli::resolve_hermes_home().join("desktop.json")).ok()
                .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
                .and_then(|v| v.get("activeProfile").and_then(|p| p.as_str()).map(|s| s.to_string()))
        }).flatten().unwrap_or_else(|| "default".into())
}

fn count_skills(dir: &PathBuf) -> u32 {
    let skills_dir = dir.join("skills");
    if !skills_dir.exists() { return 0; }
    let mut count = 0u32;
    if let Ok(cats) = fs::read_dir(&skills_dir) {
        for cat in cats.filter_map(|e| e.ok()) {
            if !cat.file_type().map(|t| t.is_dir()).unwrap_or(false) { continue; }
            if let Ok(entries) = fs::read_dir(cat.path()) {
                for entry in entries.filter_map(|e| e.ok()) {
                    if entry.file_type().map(|t| t.is_dir()).unwrap_or(false)
                        && entry.path().join("SKILL.md").exists() { count += 1; }
                }
            }
        }
    }
    count
}

fn read_profile_metadata(dir: &PathBuf) -> (String, String) {
    let config = dir.join("config.yaml");
    if !config.exists() { return (String::new(), String::new()); }
    let content = fs::read_to_string(&config).unwrap_or_default();
    let provider = content.lines()
        .find(|l| l.trim().starts_with("provider:"))
        .and_then(|l| l.split_once(':').map(|(_, v)| v.trim().trim_matches('"').trim_matches('\'').to_string()))
        .unwrap_or_default();
    let model = content.lines()
        .find(|l| l.trim().starts_with("default:"))
        .and_then(|l| l.split_once(':').map(|(_, v)| v.trim().trim_matches('"').trim_matches('\'').to_string()))
        .unwrap_or_default();
    (provider, model)
}

fn list_profiles_via_ssh(ssh_config: &config::SshConfig) -> Result<Vec<Profile>, String> {
    let raw = ssh::ssh_list_profiles(ssh_config)?;
    Ok(raw.iter().map(|v| {
        let name = v.get("name").and_then(|s| s.as_str()).unwrap_or("").to_string();
        let is_default = name == "default";
        Profile {
            name: name.clone(),
            path: String::new(),
            is_default,
            is_active: v.get("active").and_then(|b| b.as_bool()).unwrap_or(false),
            model: v.get("model").and_then(|s| s.as_str()).unwrap_or("").to_string(),
            provider: v.get("provider").and_then(|s| s.as_str()).unwrap_or("").to_string(),
            has_env: false,
            has_soul: false,
            skill_count: v.get("skillCount").and_then(|n| n.as_u64()).unwrap_or(0) as u32,
            gateway_running: false,
        }
    }).collect())
}

#[tauri::command]
pub fn list_profiles() -> Result<Vec<Profile>, String> {
    let conn = config::get_connection_config_raw()?;
    if conn.mode == "ssh" {
        return list_profiles_via_ssh(&conn.ssh);
    }
    let mut profiles = Vec::new();
    let active = active_profile_name();

    // Default profile (HERMES_HOME root)
    let home = hermes_cli::resolve_hermes_home();
    let (provider, model) = read_profile_metadata(&home);
    profiles.push(Profile {
        name: "default".into(), path: home.to_string_lossy().to_string(), is_default: true,
        is_active: active == "default", model, provider,
        has_env: home.join(".env").exists(), has_soul: home.join("SOUL.md").exists(),
        skill_count: count_skills(&home), gateway_running: home.join("gateway.pid").exists(),
    });

    // Named profiles
    let pd = profiles_dir();
    if pd.exists() {
        if let Ok(entries) = fs::read_dir(&pd) {
            for entry in entries.filter_map(|e| e.ok()) {
                if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) { continue; }
                let name = entry.file_name().to_string_lossy().to_string();
                let dir = entry.path();
                let (p, m) = read_profile_metadata(&dir);
                profiles.push(Profile {
                    name: name.clone(), path: dir.to_string_lossy().to_string(), is_default: false,
                    is_active: active == name, model: m, provider: p,
                    has_env: dir.join(".env").exists(), has_soul: dir.join("SOUL.md").exists(),
                    skill_count: count_skills(&dir), gateway_running: dir.join("gateway.pid").exists(),
                });
            }
        }
    }
    Ok(profiles)
}

#[tauri::command]
pub fn create_profile(name: String, clone: Option<bool>) -> Result<serde_json::Value, String> {
    let conn = config::get_connection_config_raw()?;
    if conn.mode == "ssh" {
        return ssh::ssh_create_profile(&conn.ssh, &name, clone.unwrap_or(false));
    }
    let mut args: Vec<&str> = vec!["profile", "create", &name];
    if clone.unwrap_or(false) { /* hermes profile create doesn't have --clone on all versions */
        args.push("--clone-from"); args.push("default");
    }
    match hermes_cli::run_hermes_cli(&args, Some(&name)) {
        Ok(_) => Ok(serde_json::json!({"success": true})),
        Err(e) => Ok(serde_json::json!({"success": false, "error": e})),
    }
}

#[tauri::command]
pub fn delete_profile(name: String) -> Result<serde_json::Value, String> {
    let conn = config::get_connection_config_raw()?;
    if conn.mode == "ssh" {
        ssh::ssh_delete_profile(&conn.ssh, &name)?;
        return Ok(serde_json::json!({"success": true}));
    }
    match hermes_cli::run_hermes_cli(&["profile", "delete", &name], None) {
        Ok(_) => Ok(serde_json::json!({"success": true})),
        Err(e) => Ok(serde_json::json!({"success": false, "error": e})),
    }
}

#[tauri::command]
pub fn set_active_profile(name: String) -> Result<bool, String> {
    let path = hermes_cli::resolve_hermes_home().join("desktop.json");
    let mut config: serde_json::Value = if path.exists() {
        fs::read_to_string(&path).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or(serde_json::json!({}))
    } else { serde_json::json!({}) };
    config["activeProfile"] = serde_json::json!(name);
    if let Some(p) = path.parent() { let _ = fs::create_dir_all(p); }
    fs::write(&path, serde_json::to_string_pretty(&config).unwrap_or_default()).map_err(|e| e.to_string())?;
    Ok(true)
}

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn test_active_profile_default() { assert!(!active_profile_name().is_empty()); }
}
