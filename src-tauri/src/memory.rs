// Memory system — MEMORY.md + USER.md read/write with § delimiter, SQLite stats
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use rusqlite::Connection;
use crate::{config, hermes_cli, ssh};

const MEMORY_CHAR_LIMIT: usize = 2200;
const USER_PROFILE_CHAR_LIMIT: usize = 1375;
const ENTRY_DELIMITER: &str = "\n\u{00A7}\n";

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MemoryEntry { pub index: usize, pub content: String }

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MemoryStats { pub total_sessions: i64, pub total_messages: i64 }

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
    pub content: String,
    pub exists: bool,
    #[serde(rename = "lastModified")]
    pub last_modified: Option<u64>,
    #[serde(rename = "charCount")]
    pub char_count: usize,
    #[serde(rename = "charLimit")]
    pub char_limit: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MemoryInfo {
    pub memory: Vec<MemoryEntry>,
    pub user: UserProfile,
    pub stats: MemoryStats,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MemoryResult { pub success: bool, pub error: Option<String>, pub error_key: Option<String> }

fn memory_path(profile: Option<&str>) -> PathBuf { hermes_cli::resolve_profile_home(profile).join("memories").join("MEMORY.md") }
fn user_path(profile: Option<&str>) -> PathBuf { hermes_cli::resolve_profile_home(profile).join("memories").join("USER.md") }
fn state_db_path() -> PathBuf { hermes_cli::resolve_hermes_home().join("state.db") }

fn parse_memory_entries(content: &str) -> Vec<MemoryEntry> {
    if content.trim().is_empty() { return Vec::new(); }
    content.split(ENTRY_DELIMITER).enumerate()
        .filter(|(_, s)| !s.trim().is_empty())
        .map(|(i, s)| MemoryEntry { index: i, content: s.trim().to_string() })
        .collect()
}

fn get_session_stats() -> MemoryStats {
    let db_path = state_db_path();
    if !db_path.exists() { return MemoryStats { total_sessions: 0, total_messages: 0 }; }
    Connection::open(&db_path).ok().map(|db| {
        let sessions = db.query_row("SELECT COUNT(*) FROM sessions", [], |r| r.get::<_, i64>(0)).unwrap_or(0);
        let messages = db.query_row("SELECT COUNT(*) FROM messages", [], |r| r.get::<_, i64>(0)).unwrap_or(0);
        MemoryStats { total_sessions: sessions, total_messages: messages }
    }).unwrap_or(MemoryStats { total_sessions: 0, total_messages: 0 })
}

fn read_memory_via_ssh(config: &config::SshConfig, profile: Option<&str>) -> Result<MemoryInfo, String> {
    let raw = ssh::ssh_read_memory(config, profile)?;
    let mem_content = raw["memory"]["content"].as_str().unwrap_or("");
    let user_content = raw["user"]["content"].as_str().unwrap_or("");
    let user_len = user_content.len();
    Ok(MemoryInfo {
        memory: parse_memory_entries(mem_content),
        user: UserProfile {
            content: user_content.to_string(),
            exists: raw["user"]["exists"].as_bool().unwrap_or(!user_content.is_empty()),
            last_modified: raw["user"]["lastModified"].as_u64(),
            char_count: raw["user"]["charCount"].as_u64().unwrap_or(user_len as u64) as usize,
            char_limit: raw["user"]["charLimit"].as_u64().unwrap_or(USER_PROFILE_CHAR_LIMIT as u64) as usize,
        },
        stats: MemoryStats {
            total_sessions: raw["stats"]["totalSessions"].as_i64().unwrap_or(0),
            total_messages: raw["stats"]["totalMessages"].as_i64().unwrap_or(0),
        },
    })
}

#[tauri::command]
pub fn read_memory(profile: Option<String>) -> Result<MemoryInfo, String> {
    let conn = config::get_connection_config_raw()?;
    if conn.mode == "ssh" {
        return read_memory_via_ssh(&conn.ssh, profile.as_deref());
    }
    let mem = memory_path(profile.as_deref());
    let usr = user_path(profile.as_deref());
    let mem_content = if mem.exists() { fs::read_to_string(&mem).unwrap_or_default() } else { String::new() };
    let user_content = if usr.exists() { fs::read_to_string(&usr).unwrap_or_default() } else { String::new() };
    let user_len = user_content.len();
    Ok(MemoryInfo {
        memory: parse_memory_entries(&mem_content),
        user: UserProfile {
            content: user_content.clone(),
            exists: !user_content.is_empty(),
            last_modified: None,
            char_count: user_len,
            char_limit: USER_PROFILE_CHAR_LIMIT,
        },
        stats: get_session_stats(),
    })
}

fn ssh_memory_path(profile: Option<&str>) -> String {
    match profile {
        Some(p) if p != "default" => format!("~/.hermes/profiles/{}/memories/MEMORY.md", p),
        _ => "~/.hermes/memories/MEMORY.md".to_string(),
    }
}

#[tauri::command]
pub fn add_memory_entry(content: String, profile: Option<String>) -> Result<MemoryResult, String> {
    let conn = config::get_connection_config_raw()?;
    if conn.mode == "ssh" {
        let remote_path = ssh_memory_path(profile.as_deref());
        let mut existing = ssh::ssh_read_file(&conn.ssh, &remote_path).unwrap_or_default();
        if existing.len() + content.len() > MEMORY_CHAR_LIMIT {
            return Ok(MemoryResult { success: false, error: Some(format!("Memory limit reached ({} chars)", MEMORY_CHAR_LIMIT)), error_key: Some("memory.limitReached".into()) });
        }
        if !existing.is_empty() && !existing.ends_with('\n') { existing.push('\n'); }
        existing.push_str(&content);
        ssh::ssh_write_file(&conn.ssh, &remote_path, &existing)?;
        return Ok(MemoryResult { success: true, error: None, error_key: None });
    }
    let path = memory_path(profile.as_deref());
    if let Some(p) = path.parent() { let _ = fs::create_dir_all(p); }
    let mut existing = if path.exists() { fs::read_to_string(&path).unwrap_or_default() } else { String::new() };
    if existing.len() + content.len() > MEMORY_CHAR_LIMIT {
        return Ok(MemoryResult { success: false, error: Some(format!("Memory limit reached ({} chars)", MEMORY_CHAR_LIMIT)), error_key: Some("memory.limitReached".into()) });
    }
    if !existing.is_empty() && !existing.ends_with('\n') { existing.push('\n'); }
    existing.push_str(&content);
    fs::write(&path, &existing).map_err(|e| e.to_string())?;
    Ok(MemoryResult { success: true, error: None, error_key: None })
}

#[tauri::command]
pub fn update_memory_entry(index: u32, content: String, profile: Option<String>) -> Result<MemoryResult, String> {
    let conn = config::get_connection_config_raw()?;
    if conn.mode == "ssh" {
        let remote_path = ssh_memory_path(profile.as_deref());
        let raw = ssh::ssh_read_file(&conn.ssh, &remote_path).unwrap_or_default();
        if raw.is_empty() { return Ok(MemoryResult { success: false, error: Some("Memory file not found".into()), error_key: Some("memory.fileNotFound".into()) }); }
        let entries: Vec<&str> = raw.split(ENTRY_DELIMITER).collect();
        if (index as usize) >= entries.len() { return Ok(MemoryResult { success: false, error: Some("Index out of range".into()), error_key: Some("memory.indexOutOfRange".into()) }); }
        let mut new_entries: Vec<&str> = entries.iter().map(|s| *s).collect();
        new_entries[index as usize] = &content;
        let new_content = new_entries.join(ENTRY_DELIMITER);
        if new_content.len() > MEMORY_CHAR_LIMIT {
            return Ok(MemoryResult { success: false, error: Some(format!("Memory limit reached ({} chars)", MEMORY_CHAR_LIMIT)), error_key: Some("memory.limitReached".into()) });
        }
        ssh::ssh_write_file(&conn.ssh, &remote_path, &new_content)?;
        return Ok(MemoryResult { success: true, error: None, error_key: None });
    }
    let path = memory_path(profile.as_deref());
    if !path.exists() { return Ok(MemoryResult { success: false, error: Some("Memory file not found".into()), error_key: Some("memory.fileNotFound".into()) }); }
    let raw = fs::read_to_string(&path).unwrap_or_default();
    let mut entries: Vec<&str> = raw.split(ENTRY_DELIMITER).collect();
    if (index as usize) >= entries.len() { return Ok(MemoryResult { success: false, error: Some("Index out of range".into()), error_key: Some("memory.indexOutOfRange".into()) }); }
    entries[index as usize] = &content;
    let new_content = entries.join(ENTRY_DELIMITER);
    if new_content.len() > MEMORY_CHAR_LIMIT {
        return Ok(MemoryResult { success: false, error: Some(format!("Memory limit reached ({} chars)", MEMORY_CHAR_LIMIT)), error_key: Some("memory.limitReached".into()) });
    }
    fs::write(&path, &new_content).map_err(|e| e.to_string())?;
    Ok(MemoryResult { success: true, error: None, error_key: None })
}

#[tauri::command]
pub fn remove_memory_entry(index: u32, profile: Option<String>) -> Result<bool, String> {
    let conn = config::get_connection_config_raw()?;
    if conn.mode == "ssh" {
        let remote_path = ssh_memory_path(profile.as_deref());
        let raw = ssh::ssh_read_file(&conn.ssh, &remote_path).unwrap_or_default();
        if raw.is_empty() { return Ok(false); }
        let mut entries: Vec<&str> = raw.split(ENTRY_DELIMITER).collect();
        if (index as usize) >= entries.len() { return Ok(false); }
        entries.remove(index as usize);
        ssh::ssh_write_file(&conn.ssh, &remote_path, &entries.join(ENTRY_DELIMITER))?;
        return Ok(true);
    }
    let path = memory_path(profile.as_deref());
    if !path.exists() { return Ok(false); }
    let raw = fs::read_to_string(&path).unwrap_or_default();
    let mut entries: Vec<&str> = raw.split(ENTRY_DELIMITER).collect();
    if (index as usize) >= entries.len() { return Ok(false); }
    entries.remove(index as usize);
    fs::write(&path, entries.join(ENTRY_DELIMITER)).map_err(|e| e.to_string())?;
    Ok(true)
}

fn ssh_user_path(profile: Option<&str>) -> String {
    match profile {
        Some(p) if p != "default" => format!("~/.hermes/profiles/{}/memories/USER.md", p),
        _ => "~/.hermes/memories/USER.md".to_string(),
    }
}

#[tauri::command]
pub fn write_user_profile(content: String, profile: Option<String>) -> Result<MemoryResult, String> {
    if content.len() > USER_PROFILE_CHAR_LIMIT {
        return Ok(MemoryResult { success: false, error: Some(format!("Profile limit reached ({} chars)", USER_PROFILE_CHAR_LIMIT)), error_key: Some("memory.profileLimitReached".into()) });
    }
    let conn = config::get_connection_config_raw()?;
    if conn.mode == "ssh" {
        let remote_path = ssh_user_path(profile.as_deref());
        ssh::ssh_write_file(&conn.ssh, &remote_path, &content)?;
        return Ok(MemoryResult { success: true, error: None, error_key: None });
    }
    let path = user_path(profile.as_deref());
    if let Some(p) = path.parent() { let _ = fs::create_dir_all(p); }
    fs::write(&path, &content).map_err(|e| e.to_string())?;
    Ok(MemoryResult { success: true, error: None, error_key: None })
}

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn test_parse_empty() { assert!(parse_memory_entries("").is_empty()); }
    #[test] fn test_parse_entries() { let e = parse_memory_entries("entry1\n§\nentry2"); assert_eq!(e.len(), 2); assert_eq!(e[0].content, "entry1"); }
    #[test] fn test_char_limits() { assert_eq!(MEMORY_CHAR_LIMIT, 2200); assert_eq!(USER_PROFILE_CHAR_LIMIT, 1375); }
}
