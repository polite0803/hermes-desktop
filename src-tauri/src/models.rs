// Model management — CRUD with defaults seeding from providers + custom_providers
use serde::{Deserialize, Serialize};
use std::fs;
use crate::{config, hermes_cli, ssh};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SavedModel { pub id: String, pub name: String, pub provider: String, pub model: String, pub base_url: String, #[serde(rename = "createdAt")] pub created_at: u64 }

fn models_path(profile: Option<&str>) -> std::path::PathBuf { hermes_cli::resolve_profile_home(profile).join("models.json") }

fn read_models_raw(profile: Option<&str>) -> Vec<SavedModel> {
    let path = models_path(profile);
    if !path.exists() { return Vec::new(); }
    fs::read_to_string(&path).ok().and_then(|c| serde_json::from_str(&c).ok()).unwrap_or_default()
}

fn write_models(models: &[SavedModel], profile: Option<&str>) -> Result<(), String> {
    let path = models_path(profile);
    if let Some(p) = path.parent() { fs::create_dir_all(p).map_err(|e| e.to_string())?; }
    fs::write(&path, serde_json::to_string_pretty(models).unwrap_or_default()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_models() -> Result<Vec<SavedModel>, String> {
    let conn = config::get_connection_config_raw()?;
    if conn.mode == "ssh" {
        let raw = ssh::ssh_list_models(&conn.ssh)?;
        return Ok(raw.iter().map(|v| SavedModel {
            id: v.get("id").and_then(|s| s.as_str()).unwrap_or("").to_string(),
            name: v.get("name").and_then(|s| s.as_str()).unwrap_or("").to_string(),
            provider: v.get("provider").and_then(|s| s.as_str()).unwrap_or("").to_string(),
            model: v.get("model").and_then(|s| s.as_str()).unwrap_or("").to_string(),
            base_url: v.get("baseUrl").and_then(|s| s.as_str()).unwrap_or("").to_string(),
            created_at: v.get("createdAt").and_then(|n| n.as_u64()).unwrap_or(0),
        }).collect());
    }
    Ok(read_models_raw(None))
}

#[tauri::command]
pub fn add_model(name: String, provider: String, model: String, base_url: String) -> Result<SavedModel, String> {
    let conn = config::get_connection_config_raw()?;
    if conn.mode == "ssh" {
        let v = ssh::ssh_add_model(&conn.ssh, &name, &provider, &model, &base_url)?;
        let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        return Ok(SavedModel {
            id: v.get("id").and_then(|s| s.as_str()).unwrap_or("").to_string(),
            name: v.get("name").and_then(|s| s.as_str()).map(|s| s.to_string()).unwrap_or(name),
            provider: v.get("provider").and_then(|s| s.as_str()).map(|s| s.to_string()).unwrap_or(provider),
            model: v.get("model").and_then(|s| s.as_str()).map(|s| s.to_string()).unwrap_or(model),
            base_url: v.get("baseUrl").and_then(|s| s.as_str()).map(|s| s.to_string()).unwrap_or(base_url),
            created_at: v.get("createdAt").and_then(|n| n.as_u64()).unwrap_or(ts),
        });
    }
    let mut models = read_models_raw(None);
    if models.iter().any(|m| m.model == model && m.provider == provider) {
        return Err("models.alreadyExists".into());
    }
    let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let saved = SavedModel { id: uuid::Uuid::new_v4().to_string(), name, provider, model, base_url, created_at: ts };
    models.push(saved.clone());
    write_models(&models, None)?;
    Ok(saved)
}

#[tauri::command]
pub fn remove_model(id: String) -> Result<bool, String> {
    let conn = config::get_connection_config_raw()?;
    if conn.mode == "ssh" {
        ssh::ssh_remove_model(&conn.ssh, &id)?;
        return Ok(true);
    }
    let mut models = read_models_raw(None);
    let len_before = models.len();
    models.retain(|m| m.id != id);
    write_models(&models, None)?;
    Ok(models.len() < len_before)
}

#[tauri::command]
pub fn update_model(id: String, fields: serde_json::Value) -> Result<bool, String> {
    let conn = config::get_connection_config_raw()?;
    if conn.mode == "ssh" {
        let mut ssh_args: Vec<&str> = vec!["models", "update", &id];
        let name_str; let provider_str; let model_str; let url_str;
        if let Some(n) = fields.get("name").and_then(|v| v.as_str()) { name_str = n.to_string(); ssh_args.push("--name"); ssh_args.push(&name_str); }
        if let Some(p) = fields.get("provider").and_then(|v| v.as_str()) { provider_str = p.to_string(); ssh_args.push("--provider"); ssh_args.push(&provider_str); }
        if let Some(m) = fields.get("model").and_then(|v| v.as_str()) { model_str = m.to_string(); ssh_args.push("--model"); ssh_args.push(&model_str); }
        if let Some(b) = fields.get("baseUrl").and_then(|v| v.as_str()) { url_str = b.to_string(); ssh_args.push("--base-url"); ssh_args.push(&url_str); }
        let cmd = ssh::build_remote_hermes_cmd(&ssh_args);
        ssh::ssh_exec(&conn.ssh, &cmd, None, 10000)?;
        return Ok(true);
    }
    let mut models = read_models_raw(None);
    if let Some(idx) = models.iter().position(|m| m.id == id) {
        if let Some(n) = fields.get("name").and_then(|v| v.as_str()) { models[idx].name = n.to_string(); }
        if let Some(p) = fields.get("provider").and_then(|v| v.as_str()) { models[idx].provider = p.to_string(); }
        if let Some(m) = fields.get("model").and_then(|v| v.as_str()) { models[idx].model = m.to_string(); }
        if let Some(b) = fields.get("baseUrl").and_then(|v| v.as_str()) { models[idx].base_url = b.to_string(); }
        write_models(&models, None)?;
        Ok(true)
    } else { Ok(false) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_model_serde() {
        let m = SavedModel { id: "t1".into(), name: "Test".into(), provider: "openai".into(), model: "gpt-4".into(), base_url: "".into(), created_at: 0 };
        let json = serde_json::to_string(&vec![m]).unwrap();
        let parsed: Vec<SavedModel> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed[0].name, "Test");
    }
}
