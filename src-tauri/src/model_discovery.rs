// Provider model discovery — HTTP GET /v1/models with in-memory 5-minute TTL cache
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use crate::hermes_cli;

static CACHE: once_cell::sync::Lazy<Mutex<HashMap<String, (Instant, Vec<String>)>>> =
    once_cell::sync::Lazy::new(|| Mutex::new(HashMap::new()));
const CACHE_TTL: Duration = Duration::from_secs(300);

const PROVIDER_BASE_URLS: &[(&str, &str)] = &[
    ("openai", "https://api.openai.com/v1"), ("openrouter", "https://openrouter.ai/api/v1"),
    ("deepseek", "https://api.deepseek.com/v1"), ("groq", "https://api.groq.com/openai/v1"),
    ("mistral", "https://api.mistral.ai/v1"), ("together", "https://api.together.xyz/v1"),
    ("fireworks", "https://api.fireworks.ai/inference/v1"), ("cerebras", "https://api.cerebras.ai/v1"),
    ("perplexity", "https://api.perplexity.ai"), ("huggingface", "https://router.huggingface.co/v1"),
    ("zai", "https://api.z.ai/api/paas/v4"), ("anthropic", "https://api.anthropic.com/v1"),
];

const NON_DISCOVERABLE: &[&str] = &[
    "auto","custom","nous","google","xai","openai-codex","xai-oauth",
    "qwen-oauth","qwen","minimax","kimi-coding",
];

fn cache_key(provider: &str, base_url: &str) -> String {
    format!("{}|{}", provider.to_lowercase(), base_url.trim_end_matches('/').to_lowercase())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DiscoverModelsResult {
    pub models: Vec<String>,
    pub status: String, // "ok" | "no-key" | "unsupported" | "unknown-host"
    pub cached: bool,
}

fn canonical_base_url(provider: &str) -> Option<String> {
    PROVIDER_BASE_URLS.iter().find(|(k,_)| *k == provider.to_lowercase()).map(|(_,v)| v.to_string())
}

fn parse_model_ids(body: &str) -> Vec<String> {
    let json: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
    let mut ids = Vec::new();
    if let Some(data) = json["data"].as_array() {
        for item in data { if let Some(id) = item["id"].as_str() { ids.push(id.to_string()); } }
    }
    if let Some(models) = json["models"].as_array() {
        for item in models { if let Some(id) = item["id"].as_str().or(item["name"].as_str()) { ids.push(id.to_string()); } }
    }
    ids.sort(); ids.dedup();
    ids
}

#[tauri::command]
pub async fn discover_provider_models(
    provider: String, base_url: Option<String>, api_key: Option<String>, profile: Option<String>,
) -> Result<DiscoverModelsResult, String> {
    let lower = provider.trim().to_lowercase();
    if NON_DISCOVERABLE.contains(&lower.as_str()) && lower != "custom" {
        return Ok(DiscoverModelsResult { models: vec![], status: "unsupported".into(), cached: false });
    }

    let base = base_url.as_deref().unwrap_or("").trim_end_matches('/').to_string();
    let base = if base.is_empty() { canonical_base_url(&lower).unwrap_or_default() } else { base };
    if base.is_empty() { return Ok(DiscoverModelsResult { models: vec![], status: "unknown-host".into(), cached: false }); }

    // Check cache
    {
        let cache = CACHE.lock().map_err(|e| e.to_string())?;
        if let Some((ts, models)) = cache.get(&cache_key(&lower, &base)) {
            if ts.elapsed() < CACHE_TTL {
                return Ok(DiscoverModelsResult { models: models.clone(), status: "ok".into(), cached: true });
            }
        }
    }

    let key = api_key.unwrap_or_default();
    if key.is_empty() { return Ok(DiscoverModelsResult { models: vec![], status: "no-key".into(), cached: false }); }

    let url = format!("{}/models", base.trim_end_matches('/'));
    let client = reqwest::Client::builder().timeout(Duration::from_secs(10)).build().map_err(|e| e.to_string())?;

    let mut req = client.get(&url).header("Accept", "application/json");
    if lower == "anthropic" {
        req = req.header("x-api-key", &key).header("anthropic-version", "2023-06-01");
    } else {
        req = req.header("Authorization", format!("Bearer {}", key));
    }

    let models = match req.send().await {
        Ok(resp) if resp.status().is_success() => {
            parse_model_ids(&resp.text().await.unwrap_or_default())
        }
        _ => Vec::new(),
    };

    if let Ok(mut cache) = CACHE.lock() {
        cache.insert(cache_key(&lower, &base), (Instant::now(), models.clone()));
    }

    Ok(DiscoverModelsResult { models, status: "ok".into(), cached: false })
}
