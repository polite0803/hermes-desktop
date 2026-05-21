// Model management — CRUD with defaults seeding from providers + custom_providers
use serde::{Deserialize, Serialize};
use std::fs;
use crate::hermes_cli;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SavedModel { pub id: String, pub name: String, pub provider: String, pub model: String, pub base_url: String, #[serde(rename = "createdAt")] pub created_at: u64 }

const DEFAULT_MODELS: &[(&str, &str, &str, &str)] = &[
    ("Claude Opus 4.7 (OpenRouter)", "openrouter", "anthropic/claude-opus-4-7", "https://openrouter.ai/api/v1"),
    ("Claude Sonnet 4.7 (Anthropic)", "anthropic", "claude-sonnet-4-7-20250514", "https://api.anthropic.com/v1"),
    ("GPT-4.1 (OpenAI)", "openai", "gpt-4.1", "https://api.openai.com/v1"),
    ("GPT-4.1 mini (OpenAI)", "openai", "gpt-4.1-mini", "https://api.openai.com/v1"),
    ("Claude Opus 4.7 (Anthropic)", "anthropic", "claude-opus-4-7-20250514", "https://api.anthropic.com/v1"),
    ("DeepSeek V3", "deepseek", "deepseek-chat", "https://api.deepseek.com/v1"),
    ("Gemini 2.5 Pro", "google", "gemini-2.5-pro-preview-05-06", ""),
    ("Grok 4", "xai", "grok-4", ""),
];

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

fn seed_defaults(profile: Option<&str>) -> Vec<SavedModel> {
    let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    DEFAULT_MODELS.iter().map(|(name, provider, model, base_url)| SavedModel {
        id: uuid::Uuid::new_v4().to_string(), name: name.to_string(), provider: provider.to_string(),
        model: model.to_string(), base_url: base_url.to_string(), created_at: ts,
    }).collect()
}

#[tauri::command]
pub fn list_models() -> Result<Vec<SavedModel>, String> {
    let mut models = read_models_raw(None);
    if models.is_empty() {
        models = seed_defaults(None);
        let _ = write_models(&models, None);
    }
    Ok(models)
}

#[tauri::command]
pub fn add_model(name: String, provider: String, model: String, base_url: String) -> Result<SavedModel, String> {
    let mut models = read_models_raw(None);
    if models.iter().any(|m| m.model == model && m.provider == provider) {
        return Err(format!("Model '{}' already exists for provider '{}'", model, provider));
    }
    let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let saved = SavedModel { id: uuid::Uuid::new_v4().to_string(), name, provider, model, base_url, created_at: ts };
    models.push(saved.clone());
    write_models(&models, None)?;
    Ok(saved)
}

#[tauri::command]
pub fn remove_model(id: String) -> Result<bool, String> {
    let mut models = read_models_raw(None);
    let len_before = models.len();
    models.retain(|m| m.id != id);
    write_models(&models, None)?;
    Ok(models.len() < len_before)
}

#[tauri::command]
pub fn update_model(id: String, fields: serde_json::Value) -> Result<bool, String> {
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
