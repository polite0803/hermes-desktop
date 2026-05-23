// Cron job management — job.json local reading + remote API, normalization
use serde::{Deserialize, Serialize};
use crate::config;
use crate::hermes_cli;
use crate::ssh;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CronJob {
    pub id: String, pub name: String, pub schedule: String, pub prompt: String,
    pub state: String, pub enabled: bool, pub next_run_at: Option<String>, pub last_run_at: Option<String>,
    pub last_status: Option<String>, pub last_error: Option<String>,
    pub repeat: Option<serde_json::Value>, pub deliver: Vec<String>, pub skills: Vec<String>, pub script: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CronResult { pub success: bool, pub error: Option<String> }

fn jobs_file(profile: Option<&str>) -> std::path::PathBuf {
    hermes_cli::resolve_profile_home(profile).join("cron").join("jobs.json")
}

fn read_local_jobs(profile: Option<&str>) -> Vec<serde_json::Value> {
    let path = jobs_file(profile);
    if !path.exists() { return Vec::new(); }
    std::fs::read_to_string(&path).ok()
        .and_then(|s| serde_json::from_str::<Vec<serde_json::Value>>(&s).ok())
        .unwrap_or_default()
}

fn normalize_job(raw: &serde_json::Value) -> CronJob {
    CronJob {
        id: raw["id"].as_str().unwrap_or("").to_string(),
        name: raw["name"].as_str().unwrap_or("").to_string(),
        schedule: raw["schedule"].as_str().or(raw["cron"].as_str()).unwrap_or("").to_string(),
        prompt: raw["prompt"].as_str().or(raw["message"].as_str()).unwrap_or("").to_string(),
        state: raw["state"].as_str().unwrap_or("active").to_string(),
        enabled: raw["enabled"].as_bool().unwrap_or(true),
        next_run_at: raw["next_run_at"].as_str().map(|s| s.to_string()),
        last_run_at: raw["last_run_at"].as_str().map(|s| s.to_string()),
        last_status: raw["last_status"].as_str().map(|s| s.to_string()),
        last_error: raw["last_error"].as_str().map(|s| s.to_string()),
        repeat: raw.get("repeat").cloned(),
        deliver: raw["deliver"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()).unwrap_or_default(),
        skills: raw["skills"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()).unwrap_or_default(),
        script: raw["script"].as_str().map(|s| s.to_string()),
    }
}

#[tauri::command]
pub fn list_cron_jobs(include_disabled: Option<bool>, profile: Option<String>) -> Result<Vec<CronJob>, String> {
    let conn = config::get_connection_config_raw()?;
    if conn.mode == "ssh" {
        let cmd = ssh::build_remote_hermes_cmd(&["cron", "list", "--json"]);
        match ssh::ssh_exec(&conn.ssh, &cmd, None, 10000) {
            Ok(out) => {
                let jobs: Vec<serde_json::Value> = serde_json::from_str(&out).unwrap_or_default();
                return Ok(jobs.iter().map(normalize_job)
                    .filter(|j| include_disabled.unwrap_or(false) || j.enabled)
                    .collect());
            }
            Err(_) => return Ok(Vec::new()),
        }
    }
    if conn.mode == "remote" && !conn.remote_url.is_empty() {
        // Remote API
        let url = format!("{}/api/jobs", conn.remote_url.trim_end_matches('/'));
        match reqwest::blocking::get(&url) {
            Ok(resp) => {
                let jobs: Vec<serde_json::Value> = resp.json().unwrap_or_default();
                return Ok(jobs.iter().map(normalize_job)
                    .filter(|j| include_disabled.unwrap_or(false) || j.enabled)
                    .collect());
            }
            Err(_) => return Ok(Vec::new()),
        }
    }

    // Local job.json
    let raw = read_local_jobs(profile.as_deref());
    Ok(raw.iter().map(normalize_job)
        .filter(|j| include_disabled.unwrap_or(false) || j.enabled)
        .collect())
}

fn run_cron_cmd(args: &[&str], profile: Option<&str>) -> Result<CronResult, String> {
    let conn = config::get_connection_config_raw()?;
    if conn.mode == "ssh" {
        let cmd = ssh::build_remote_hermes_cmd(args);
        match ssh::ssh_exec(&conn.ssh, &cmd, None, 15000) {
            Ok(_) => Ok(CronResult { success: true, error: None }),
            Err(e) => Ok(CronResult { success: false, error: Some(e) }),
        }
    } else {
        match hermes_cli::run_hermes_cli(args, profile) {
            Ok(_) => Ok(CronResult { success: true, error: None }),
            Err(e) => Ok(CronResult { success: false, error: Some(e) }),
        }
    }
}

#[tauri::command]
pub fn create_cron_job(name: Option<String>, schedule: String, prompt: Option<String>, deliver: Option<String>, profile: Option<String>) -> Result<CronResult, String> {
    let mut args: Vec<&str> = vec!["cron", "create", "--schedule", &schedule];
    if let Some(ref n) = name { args.push("--name"); args.push(n); }
    if let Some(ref p) = prompt { args.push("--prompt"); args.push(p); }
    if let Some(ref d) = deliver { args.push("--deliver"); args.push(d); }
    run_cron_cmd(&args, profile.as_deref())
}

#[tauri::command] pub fn remove_cron_job(id: String, profile: Option<String>) -> Result<CronResult, String> {
    run_cron_cmd(&["cron", "remove", &id], profile.as_deref())
}
#[tauri::command] pub fn pause_cron_job(id: String, profile: Option<String>) -> Result<CronResult, String> {
    run_cron_cmd(&["cron", "pause", &id], profile.as_deref())
}
#[tauri::command] pub fn resume_cron_job(id: String, profile: Option<String>) -> Result<CronResult, String> {
    run_cron_cmd(&["cron", "resume", &id], profile.as_deref())
}
#[tauri::command] pub fn trigger_cron_job(id: String, profile: Option<String>) -> Result<CronResult, String> {
    run_cron_cmd(&["cron", "trigger", &id], profile.as_deref())
}
