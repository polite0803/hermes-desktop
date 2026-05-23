// Session cache — SQLite-to-JSON incremental sync with title generation
// Rewrite from original src/main/session-cache.ts (230 lines TS)
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::{config, hermes_cli, ssh};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CachedSession {
    pub id: String,
    pub title: String,
    #[serde(rename = "startedAt")]
    pub started_at: i64,
    pub source: String,
    #[serde(rename = "messageCount")]
    pub message_count: i64,
    pub model: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CacheData {
    sessions: Vec<CachedSession>,
    #[serde(rename = "lastSync")]
    last_sync: i64,
}

fn cache_file() -> PathBuf { hermes_cli::resolve_hermes_home().join("desktop").join("sessions.json") }
fn db_path() -> PathBuf { hermes_cli::resolve_hermes_home().join("state.db") }

fn read_cache() -> CacheData {
    let path = cache_file();
    if !path.exists() { return CacheData { sessions: vec![], last_sync: 0 }; }
    fs::read_to_string(&path).ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(CacheData { sessions: vec![], last_sync: 0 })
}

fn write_cache(data: &CacheData) {
    let path = cache_file();
    if let Some(parent) = path.parent() { let _ = fs::create_dir_all(parent); }
    let _ = fs::write(&path, serde_json::to_string(data).unwrap_or_default());
}

fn generate_title(message: &str) -> String {
    if message.trim().is_empty() { return "New conversation".into(); }
    let mut text = message.trim().to_string();
    // Remove markdown formatting and URLs
    text = text.replace(['#', '*', '_', '`', '~', '[', ']', '(', ')'], "");
    let url_re = regex_lite::Regex::new(r"https?://\S+").ok();
    if let Some(re) = url_re { text = re.replace_all(&text, "").to_string(); }
    text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if text.len() <= 50 { return text; }
    let words: Vec<&str> = text.split(' ').collect();
    let mut title = String::new();
    for w in words {
        if title.len() + w.len() + 1 > 45 { break; }
        if !title.is_empty() { title.push(' '); }
        title.push_str(w);
    }
    if title.is_empty() { text.chars().take(45).collect::<String>() + "..." } else { title }
}

#[tauri::command]
pub fn sync_session_cache() -> Result<Vec<CachedSession>, String> {
    let conn = config::get_connection_config_raw()?;
    if conn.mode == "ssh" {
        return Ok(Vec::new()); // No local SQLite to sync in SSH mode
    }
    let mut cache = read_cache();
    let db_path = db_path();
    if !db_path.exists() { return Ok(cache.sessions); }

    let db = Connection::open(&db_path).map_err(|e| e.to_string())?;

    // Fetch sessions newer than last sync
    let cutoff = if cache.last_sync > 0 { cache.last_sync - 300 } else { 0 };
    let mut stmt = db.prepare(
        "SELECT s.id, s.started_at, s.source, s.message_count, s.model, s.title FROM sessions s WHERE s.started_at > ?1 ORDER BY s.started_at DESC"
    ).map_err(|e| e.to_string())?;

    let rows: Vec<(String, i64, String, i64, String, Option<String>)> = stmt.query_map([cutoff], |row| Ok((
        row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get::<_, String>(4).unwrap_or_default(), row.get(5)?
    ))).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();

    let existing: std::collections::HashMap<String, usize> = cache.sessions.iter().enumerate().map(|(i, s)| (s.id.clone(), i)).collect();
    let mut refreshed = std::collections::HashSet::new();

    for (id, started_at, source, msg_count, model, row_title) in &rows {
        refreshed.insert(id.clone());
        if let Some(&idx) = existing.get(id) {
            cache.sessions[idx].message_count = *msg_count;
        } else {
            let title = row_title.clone().filter(|t| !t.is_empty()).unwrap_or_else(|| {
                // Try to get first user message for title
                let msg_stmt = db.prepare("SELECT content FROM messages WHERE session_id = ?1 AND role = 'user' AND content IS NOT NULL ORDER BY timestamp, id LIMIT 1");
                msg_stmt.ok().and_then(|mut s| s.query_row([id], |row| row.get::<_, String>(0)).ok())
                    .map(|c| generate_title(&c)).unwrap_or_else(|| "New conversation".into())
            });
            cache.sessions.push(CachedSession {
                id: id.clone(), title, started_at: *started_at, source: source.clone(), message_count: *msg_count, model: model.clone(),
            });
        }
    }

    // Refresh stale message counts
    let stale: Vec<String> = existing.keys().filter(|id| !refreshed.contains(*id)).cloned().collect();
    if !stale.is_empty() {
        for chunk in stale.chunks(500) {
            let placeholders: Vec<String> = chunk.iter().enumerate().map(|(i, _)| format!("?{}", i + 1)).collect();
            let sql = format!("SELECT id, message_count FROM sessions WHERE id IN ({})", placeholders.join(","));
            if let Ok(mut s) = db.prepare(&sql) {
                let counts: std::collections::HashMap<String, i64> = s.query_map(
                    rusqlite::params_from_iter(chunk.iter()),
                    |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
                ).ok().into_iter().flat_map(|r| r).filter_map(|r| r.ok()).collect();
                for s in &mut cache.sessions {
                    if let Some(&c) = counts.get(&s.id) { s.message_count = c; }
                }
            }
        }
    }

    cache.sessions.sort_by(|a, b| b.started_at.cmp(&a.started_at));
    cache.last_sync = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
    write_cache(&cache);
    Ok(cache.sessions)
}

#[tauri::command]
pub fn list_cached_sessions(limit: Option<u32>, offset: Option<u32>) -> Result<Vec<CachedSession>, String> {
    let conn = config::get_connection_config_raw()?;
    if conn.mode == "ssh" {
        let raw = ssh::ssh_list_sessions(&conn.ssh, limit, offset)?;
        return Ok(raw.iter().map(|v| CachedSession {
            id: v.get("id").and_then(|s| s.as_str()).unwrap_or("").to_string(),
            title: v.get("title").and_then(|s| s.as_str()).unwrap_or("").to_string(),
            started_at: v.get("startedAt").or_else(|| v.get("started_at")).and_then(|n| n.as_i64()).unwrap_or(0),
            source: v.get("source").and_then(|s| s.as_str()).unwrap_or("").to_string(),
            message_count: v.get("messageCount").or_else(|| v.get("message_count")).and_then(|n| n.as_i64()).unwrap_or(0),
            model: v.get("model").and_then(|s| s.as_str()).unwrap_or("").to_string(),
        }).collect());
    }
    let cache = read_cache();
    let off = offset.unwrap_or(0) as usize;
    let lim = limit.unwrap_or(50) as usize;
    Ok(cache.sessions.iter().skip(off).take(lim).cloned().collect())
}

#[tauri::command]
pub fn update_session_title(session_id: String, title: String) -> Result<(), String> {
    let conn = config::get_connection_config_raw()?;
    if conn.mode == "ssh" {
        return Ok(()); // Local cache not used in SSH mode
    }
    let mut cache = read_cache();
    if let Some(s) = cache.sessions.iter_mut().find(|s| s.id == session_id) { s.title = title; write_cache(&cache); }
    Ok(())
}
