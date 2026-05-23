// SQLite session management — list, search (FTS5 + LIKE fallback), messages, delete
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use crate::{config, hermes_cli, ssh};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SessionSummary { pub id: String, pub source: String, pub started_at: i64, pub ended_at: Option<i64>, pub message_count: i64, pub model: String, pub title: Option<String>, pub preview: String }

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SessionMessage { pub id: i64, pub role: String, pub content: String, pub timestamp: i64 }

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult { #[serde(rename = "sessionId")] pub session_id: String, pub title: Option<String>, pub started_at: i64, pub source: String, pub message_count: i64, pub model: String, pub snippet: String }

fn state_db_path() -> std::path::PathBuf { hermes_cli::resolve_hermes_home().join("state.db") }

fn open_db() -> Result<Connection, String> {
    let path = state_db_path();
    if !path.exists() { return Err("sessions.dbNotFound".into()); }
    Connection::open(&path).map_err(|e| e.to_string())
}

fn decode_content(raw: &str) -> String {
    if raw.starts_with("\0json:") {
        let json_part = &raw[6..];
        match serde_json::from_str::<serde_json::Value>(json_part) {
            Ok(v) => {
                // If the top-level has a "content" array, unwrap it
                let content_arr = v.get("content").and_then(|c| c.as_array())
                    .or_else(|| v.as_array());
                if let Some(items) = content_arr {
                    return items.iter().filter_map(|item| {
                        item.get("text").and_then(|t| t.as_str())
                            .or_else(|| item.get("content").and_then(|c| c.as_str()))
                            .map(|s| s.to_string())
                    }).collect::<Vec<_>>().join(" ");
                }
                // Plain string value
                v.as_str().map(|s| s.to_string()).unwrap_or_else(|| raw.to_string())
            }
            Err(_) => raw.to_string(),
        }
    } else {
        raw.to_string()
    }
}

fn convert_ssh_session(v: &serde_json::Value) -> SessionSummary {
    SessionSummary {
        id: v.get("id").or_else(|| v.get("sessionId")).and_then(|s| s.as_str()).unwrap_or("").to_string(),
        source: v.get("source").and_then(|s| s.as_str()).unwrap_or("").to_string(),
        started_at: v.get("startedAt").or_else(|| v.get("started_at")).and_then(|n| n.as_i64()).unwrap_or(0),
        ended_at: v.get("endedAt").or_else(|| v.get("ended_at")).and_then(|n| n.as_i64()),
        message_count: v.get("messageCount").or_else(|| v.get("message_count")).and_then(|n| n.as_i64()).unwrap_or(0),
        model: v.get("model").and_then(|s| s.as_str()).unwrap_or("").to_string(),
        title: v.get("title").and_then(|s| s.as_str()).map(|s| s.to_string()),
        preview: v.get("preview").and_then(|s| s.as_str()).unwrap_or("").to_string(),
    }
}

fn convert_ssh_message(v: &serde_json::Value) -> SessionMessage {
    SessionMessage {
        id: v.get("id").and_then(|n| n.as_i64()).unwrap_or(0),
        role: v.get("role").and_then(|s| s.as_str()).unwrap_or("").to_string(),
        content: v.get("content").and_then(|s| s.as_str()).unwrap_or("").to_string(),
        timestamp: v.get("timestamp").and_then(|n| n.as_i64()).unwrap_or(0),
    }
}

fn convert_ssh_search_result(v: &serde_json::Value) -> SearchResult {
    SearchResult {
        session_id: v.get("sessionId").or_else(|| v.get("session_id")).and_then(|s| s.as_str()).unwrap_or("").to_string(),
        title: v.get("title").and_then(|s| s.as_str()).map(|s| s.to_string()),
        started_at: v.get("startedAt").or_else(|| v.get("started_at")).and_then(|n| n.as_i64()).unwrap_or(0),
        source: v.get("source").and_then(|s| s.as_str()).unwrap_or("").to_string(),
        message_count: v.get("messageCount").or_else(|| v.get("message_count")).and_then(|n| n.as_i64()).unwrap_or(0),
        model: v.get("model").and_then(|s| s.as_str()).unwrap_or("").to_string(),
        snippet: v.get("snippet").and_then(|s| s.as_str()).unwrap_or("").to_string(),
    }
}

#[tauri::command]
pub fn list_sessions(limit: Option<u32>, offset: Option<u32>) -> Result<Vec<SessionSummary>, String> {
    let conn = config::get_connection_config_raw()?;
    if conn.mode == "ssh" {
        let raw = ssh::ssh_list_sessions(&conn.ssh, limit, offset)?;
        return Ok(raw.iter().map(convert_ssh_session).collect());
    }
    let db = open_db()?;
    let lim = limit.unwrap_or(50); let off = offset.unwrap_or(0);
    let mut stmt = db.prepare(
        "SELECT s.id, s.source, s.started_at, s.ended_at, s.message_count, COALESCE(s.model,''), s.title, (SELECT SUBSTR(m.content,1,200) FROM messages m WHERE m.session_id=s.id AND m.role='user' ORDER BY m.timestamp, m.id LIMIT 1) as preview FROM sessions s ORDER BY s.started_at DESC LIMIT ?1 OFFSET ?2"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(params![lim as i64, off as i64], |row| Ok(SessionSummary {
        id: row.get(0)?, source: row.get(1)?, started_at: row.get(2)?, ended_at: row.get(3)?,
        message_count: row.get(4)?, model: row.get(5)?, title: row.get(6)?,
        preview: row.get::<_, Option<String>>(7)?.unwrap_or_default(),
    })).map_err(|e| e.to_string())?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

#[tauri::command]
pub fn get_session_messages(session_id: String) -> Result<Vec<SessionMessage>, String> {
    let conn = config::get_connection_config_raw()?;
    if conn.mode == "ssh" {
        let raw = ssh::ssh_get_session_messages(&conn.ssh, &session_id)?;
        return Ok(raw.iter().map(convert_ssh_message).collect());
    }
    let db = open_db()?;
    let mut stmt = db.prepare(
        "SELECT id, role, content, timestamp FROM messages WHERE session_id=?1 ORDER BY timestamp, id"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(params![session_id], |row| Ok(SessionMessage {
        id: row.get(0)?, role: row.get(1)?, content: decode_content(&row.get::<_, String>(2)?), timestamp: row.get(3)?,
    })).map_err(|e| e.to_string())?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

#[tauri::command]
pub fn search_sessions(query: String, limit: Option<u32>) -> Result<Vec<SearchResult>, String> {
    let conn = config::get_connection_config_raw()?;
    if conn.mode == "ssh" {
        let raw = ssh::ssh_search_sessions(&conn.ssh, &query, limit)?;
        return Ok(raw.iter().map(convert_ssh_search_result).collect());
    }
    let db = open_db()?;
    let lim = limit.unwrap_or(20) as i64;

    // Try FTS5 first — collect immediately to avoid borrow issues
    let fts_result: Option<Vec<SearchResult>> = db.prepare(
        "SELECT s.id, s.title, s.started_at, s.source, s.message_count, COALESCE(s.model,''), snippet(messages_fts, 2, '<b>', '</b>', '...', 40) FROM messages_fts JOIN sessions s ON messages_fts.session_id = s.id WHERE messages_fts MATCH ?1 ORDER BY rank LIMIT ?2"
    ).ok().and_then(|mut stmt| {
        let rows: Vec<SearchResult> = stmt.query_map(params![query, lim], |row| Ok(SearchResult {
            session_id: row.get(0)?, title: row.get(1)?, started_at: row.get(2)?, source: row.get(3)?,
            message_count: row.get(4)?, model: row.get(5)?, snippet: row.get::<_, String>(6).unwrap_or_default(),
        })).ok()?.filter_map(|r| r.ok()).collect();
        if rows.is_empty() { None } else { Some(rows) }
    });

    if let Some(ref results) = fts_result { if !results.is_empty() { return Ok(results.clone()); } }

    // LIKE fallback
    let like_query = format!("%{}%", query);
    let mut stmt = db.prepare(
        "SELECT s.id, s.title, s.started_at, s.source, s.message_count, COALESCE(s.model,''), SUBSTR(m.content,1,200) FROM sessions s JOIN messages m ON m.session_id=s.id WHERE m.content LIKE ?1 GROUP BY s.id ORDER BY s.started_at DESC LIMIT ?2"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(params![like_query, lim], |row| Ok(SearchResult {
        session_id: row.get(0)?, title: row.get(1)?, started_at: row.get(2)?, source: row.get(3)?,
        message_count: row.get(4)?, model: row.get(5)?, snippet: row.get::<_, String>(6).unwrap_or_default(),
    })).map_err(|e| e.to_string())?;
    Ok(rows.filter_map(|r| r.ok()).collect())
}

#[tauri::command]
pub fn delete_session(session_id: String) -> Result<(), String> {
    let conn = config::get_connection_config_raw()?;
    if conn.mode == "ssh" {
        let cmd = ssh::build_remote_hermes_cmd(&["sessions", "delete", &session_id]);
        ssh::ssh_exec(&conn.ssh, &cmd, None, 15000)?;
        return Ok(());
    }
    let db = open_db()?;
    db.execute("DELETE FROM messages WHERE session_id=?1", params![session_id]).map_err(|e| e.to_string())?;
    db.execute("DELETE FROM sessions WHERE id=?1", params![session_id]).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn test_decode_plain() { assert_eq!(decode_content("hello"), "hello"); }
    #[test] fn test_decode_json_prefixed() {
        // Envelope with content array
        assert_eq!(decode_content("\0json:{\"role\":\"user\",\"content\":[{\"type\":\"text\",\"text\":\"hi\"}]}"), "hi");
        // Bare array
        assert!(decode_content("\0json:[{\"type\":\"text\",\"text\":\"hi there\"}]").contains("hi there"));
    }
}
