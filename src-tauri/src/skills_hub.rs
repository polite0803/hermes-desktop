// Skills Hub — search and install skills from agentskills.io
use serde::{Deserialize, Serialize};
use std::time::Duration;
use crate::{config, hermes_cli, ssh};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HubSkill {
    pub name: String,
    pub description: String,
    pub category: String,
    pub author: String,
    pub downloads: u32,
    pub installed: bool,
}

#[tauri::command]
pub async fn search_skills_hub(query: String) -> Result<Vec<HubSkill>, String> {
    let url = format!("https://agentskills.io/api/skills?q={}", urlencoding(&query));
    let client = reqwest::Client::builder().timeout(Duration::from_secs(10)).build().map_err(|e| e.to_string())?;
    let resp = client.get(&url).header("Accept", "application/json").send().await.map_err(|e| format!("Request failed: {}", e))?;
    let body = resp.text().await.unwrap_or_default();
    let json: serde_json::Value = serde_json::from_str(&body).map_err(|e| format!("Parse error: {}", e))?;

    let empty = vec![];
    let items: &Vec<serde_json::Value> = json.get("skills").and_then(|v| v.as_array()).or_else(|| json.as_array()).unwrap_or(&empty);
    let skills: Vec<HubSkill> = items.iter().map(|item| {
            HubSkill {
                name: item["name"].as_str().unwrap_or("").to_string(),
                description: item["description"].as_str().unwrap_or("").to_string(),
                category: item["category"].as_str().unwrap_or("").to_string(),
                author: item["author"].as_str().unwrap_or("").to_string(),
                downloads: item["downloads"].as_u64().unwrap_or(0) as u32,
                installed: false,
            }
        }).collect();

    Ok(skills)
}

#[tauri::command]
pub fn install_from_hub(name: String, profile: Option<String>) -> Result<serde_json::Value, String> {
    let conn = config::get_connection_config_raw()?;
    if conn.mode == "ssh" {
        let cmd = ssh::build_remote_hermes_cmd(&["skills", "install", &name, "--yes"]);
        match ssh::ssh_exec(&conn.ssh, &cmd, None, 30000) {
            Ok(_) => return Ok(serde_json::json!({"success": true})),
            Err(e) => return Ok(serde_json::json!({"success": false, "error": e})),
        }
    }
    match hermes_cli::run_hermes_cli(&["skills", "install", &name, "--yes"], profile.as_deref()) {
        Ok(_) => Ok(serde_json::json!({"success": true})),
        Err(e) => Ok(serde_json::json!({"success": false, "error": e})),
    }
}

/// Search huggingface/skills tap
#[tauri::command]
pub async fn search_huggingface_skills(query: String) -> Result<Vec<HubSkill>, String> {
    let url = format!("https://huggingface.co/api/skills?search={}", urlencoding(&query));
    let client = reqwest::Client::builder().timeout(Duration::from_secs(10)).build().map_err(|e| e.to_string())?;
    let resp = client.get(&url).header("Accept", "application/json").send().await.map_err(|e| format!("Request failed: {}", e))?;
    let body = resp.text().await.unwrap_or_default();
    let json: serde_json::Value = serde_json::from_str(&body).unwrap_or_default();
    let skills: Vec<HubSkill> = json.as_array().map_or(Vec::new(), |arr| arr.iter().map(|item| HubSkill {
        name: item["name"].as_str().unwrap_or("").to_string(),
        description: item["description"].as_str().unwrap_or("").to_string(),
        category: item["category"].as_str().unwrap_or("").to_string(),
        author: item["author"].as_str().unwrap_or("").to_string(),
        downloads: item["downloads"].as_u64().unwrap_or(0) as u32,
        installed: false,
    }).collect());
    Ok(skills)
}

fn urlencoding(s: &str) -> String {
    let mut result = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => result.push(b as char),
            b' ' => result.push('+'),
            _ => { result.push('%'); result.push_str(&format!("{:02X}", b)); }
        }
    }
    result
}
