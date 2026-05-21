// Usage dashboard — token/session stats from SQLite + hermes insights CLI
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use crate::hermes_cli;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UsageStats {
    pub total_sessions: i64,
    pub total_messages: i64,
    pub active_skills: i64,
    pub memory_entries: i64,
}

fn state_db_path() -> std::path::PathBuf { hermes_cli::resolve_hermes_home().join("state.db") }

#[tauri::command]
pub fn get_usage_stats() -> Result<UsageStats, String> {
    let db_path = state_db_path();
    if !db_path.exists() {
        return Ok(UsageStats { total_sessions: 0, total_messages: 0, active_skills: 0, memory_entries: 0 });
    }
    let db = Connection::open(&db_path).map_err(|e| e.to_string())?;
    let sessions = db.query_row("SELECT COUNT(*) FROM sessions", [], |r| r.get::<_, i64>(0)).unwrap_or(0);
    let messages = db.query_row("SELECT COUNT(*) FROM messages", [], |r| r.get::<_, i64>(0)).unwrap_or(0);

    // Count skills
    let skills_dir = hermes_cli::resolve_hermes_home().join("skills");
    let active_skills = if skills_dir.exists() {
        let mut count = 0i64;
        if let Ok(cats) = std::fs::read_dir(&skills_dir) {
            for cat in cats.filter_map(|e| e.ok()) {
                if cat.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    if let Ok(entries) = std::fs::read_dir(cat.path()) {
                        count += entries.filter_map(|e| e.ok()).filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false) && e.path().join("SKILL.md").exists()).count() as i64;
                    }
                }
            }
        }
        count
    } else { 0 };

    let mem_path = hermes_cli::resolve_hermes_home().join("memories").join("MEMORY.md");
    let memory_entries = if mem_path.exists() {
        std::fs::read_to_string(&mem_path).unwrap_or_default().split("\n\u{00A7}\n").filter(|s| !s.trim().is_empty()).count() as i64
    } else { 0 };

    Ok(UsageStats { total_sessions: sessions, total_messages: messages, active_skills, memory_entries })
}

#[tauri::command]
pub fn get_insights() -> Result<String, String> {
    hermes_cli::run_hermes_cli(&["insights", "--days", "30"], None)
}
