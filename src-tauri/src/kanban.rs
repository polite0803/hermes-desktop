// Kanban — 17 commands mapped to hermes-agent v0.14 CLI
use serde::{Deserialize, Serialize};
use crate::hermes_cli;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct KanbanResult<T> {
    pub success: bool, pub data: Option<T>, pub error: Option<String>,
}

#[tauri::command]
pub fn kanban_list_boards(include_archived: Option<bool>, profile: Option<String>) -> Result<KanbanResult<Vec<serde_json::Value>>, String> {
    let mut args = vec!["kanban", "boards", "--json"];
    if include_archived.unwrap_or(false) { args.push("--include-archived"); }
    ok_json(&args, profile.as_deref())
}

#[tauri::command]
pub fn kanban_current_board(profile: Option<String>) -> Result<KanbanResult<serde_json::Value>, String> {
    ok_value(&["kanban", "boards", "--current", "--json"], profile.as_deref())
}

#[tauri::command]
pub fn kanban_switch_board(slug: String, profile: Option<String>) -> Result<KanbanResult<bool>, String> {
    let args = vec!["kanban", "boards", "--switch", slug.as_str()];
    ok_bool(&args, profile.as_deref())
}

#[tauri::command]
pub fn kanban_create_board(slug: String, name: Option<String>, switch_after: Option<bool>, profile: Option<String>) -> Result<KanbanResult<serde_json::Value>, String> {
    let mut args = vec!["kanban", "create", "--slug", &slug, "--json"];
    if let Some(n) = &name { args.push("--name"); args.push(n); }
    if switch_after.unwrap_or(false) { args.push("--switch"); }
    ok_value(&args, profile.as_deref())
}

#[tauri::command]
pub fn kanban_remove_board(slug: String, hard_delete: Option<bool>, profile: Option<String>) -> Result<KanbanResult<bool>, String> {
    let mut args = vec!["kanban", "archive", "--slug", &slug];
    if hard_delete.unwrap_or(false) { args.push("--hard"); }
    ok_bool(&args, profile.as_deref())
}

#[tauri::command]
pub fn kanban_list_tasks(filters: Option<serde_json::Value>, profile: Option<String>) -> Result<KanbanResult<Vec<serde_json::Value>>, String> {
    let mut args = vec!["kanban", "list", "--json"];
    if let Some(f) = &filters {
        if let Some(s) = f.get("status").and_then(|v| v.as_str()) { args.push("--status"); args.push(s); }
        if let Some(a) = f.get("assignee").and_then(|v| v.as_str()) { args.push("--assignee"); args.push(a); }
        if f.get("includeArchived").and_then(|v| v.as_bool()).unwrap_or(false) { args.push("--include-archived"); }
    }
    ok_json(&args, profile.as_deref())
}

#[tauri::command]
pub fn kanban_get_task(task_id: String, profile: Option<String>) -> Result<KanbanResult<serde_json::Value>, String> {
    ok_value(&["kanban", "show", &task_id, "--json"], profile.as_deref())
}

#[tauri::command]
pub fn kanban_create_task(input: serde_json::Value, profile: Option<String>) -> Result<KanbanResult<serde_json::Value>, String> {
    let mut args = vec!["kanban", "create", "--json"];
    if let Some(t) = input.get("title").and_then(|v| v.as_str()) { args.push("--title"); args.push(t); }
    if let Some(b) = input.get("body").and_then(|v| v.as_str()) { args.push("--body"); args.push(b); }
    if let Some(a) = input.get("assignee").and_then(|v| v.as_str()) { args.push("--assignee"); args.push(a); }
    let prio_str = input.get("priority").and_then(|v| v.as_i64()).map(|p| p.to_string());
    if let Some(ref p) = prio_str { args.push("--priority"); args.push(p); }
    ok_value(&args, profile.as_deref())
}

#[tauri::command]
pub fn kanban_assign_task(task_id: String, assignee: Option<String>, profile: Option<String>) -> Result<KanbanResult<bool>, String> {
    let mut args = vec!["kanban", "assign", &task_id];
    if let Some(a) = &assignee { args.push("--assignee"); args.push(a); }
    ok_bool(&args, profile.as_deref())
}

#[tauri::command]
pub fn kanban_complete_task(task_id: String, result: Option<String>, profile: Option<String>) -> Result<KanbanResult<bool>, String> {
    let mut args = vec!["kanban", "complete", &task_id];
    if let Some(r) = &result { args.push("--result"); args.push(r); }
    ok_bool(&args, profile.as_deref())
}

#[tauri::command]
pub fn kanban_block_task(task_id: String, reason: Option<String>, profile: Option<String>) -> Result<KanbanResult<bool>, String> {
    let mut args = vec!["kanban", "block", &task_id];
    if let Some(r) = &reason { args.push("--reason"); args.push(r); }
    ok_bool(&args, profile.as_deref())
}

#[tauri::command]
pub fn kanban_unblock_task(task_id: String, profile: Option<String>) -> Result<KanbanResult<bool>, String> {
    ok_bool(&["kanban", "unblock", &task_id], profile.as_deref())
}

#[tauri::command]
pub fn kanban_archive_task(task_id: String, profile: Option<String>) -> Result<KanbanResult<bool>, String> {
    ok_bool(&["kanban", "archive", &task_id], profile.as_deref())
}

#[tauri::command]
pub fn kanban_specify_task(task_id: String, profile: Option<String>) -> Result<KanbanResult<bool>, String> {
    ok_bool(&["kanban", "specify", &task_id], profile.as_deref())
}

#[tauri::command]
pub fn kanban_reclaim_task(task_id: String, reason: Option<String>, profile: Option<String>) -> Result<KanbanResult<bool>, String> {
    let mut args = vec!["kanban", "reclaim", &task_id];
    if let Some(r) = &reason { args.push("--reason"); args.push(r); }
    ok_bool(&args, profile.as_deref())
}

#[tauri::command]
pub fn kanban_comment_task(task_id: String, body: String, profile: Option<String>) -> Result<KanbanResult<serde_json::Value>, String> {
    ok_value(&["kanban", "comment", &task_id, "--body", &body, "--json"], profile.as_deref())
}

#[tauri::command]
pub fn kanban_dispatch_once(dry_run: Option<bool>, profile: Option<String>) -> Result<KanbanResult<bool>, String> {
    let mut args = vec!["kanban", "dispatch"];
    if dry_run.unwrap_or(false) { args.push("--dry-run"); }
    ok_bool(&args, profile.as_deref())
}

// ── helpers ──

fn ok_json(args: &[&str], profile: Option<&str>) -> Result<KanbanResult<Vec<serde_json::Value>>, String> {
    match hermes_cli::run_hermes_cli(args, profile) {
        Ok(json) => {
            let data: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap_or_default();
            Ok(KanbanResult { success: true, data: Some(data), error: None })
        }
        Err(e) => Ok(KanbanResult { success: false, data: None, error: Some(e) }),
    }
}

fn ok_value(args: &[&str], profile: Option<&str>) -> Result<KanbanResult<serde_json::Value>, String> {
    match hermes_cli::run_hermes_cli(args, profile) {
        Ok(json) => {
            let data = serde_json::from_str(&json).unwrap_or(serde_json::Value::Null);
            Ok(KanbanResult { success: true, data: Some(data), error: None })
        }
        Err(e) => Ok(KanbanResult { success: false, data: None, error: Some(e) }),
    }
}

fn ok_bool(args: &[&str], profile: Option<&str>) -> Result<KanbanResult<bool>, String> {
    match hermes_cli::run_hermes_cli(args, profile) {
        Ok(_) => Ok(KanbanResult { success: true, data: Some(true), error: None }),
        Err(e) => Ok(KanbanResult { success: false, data: None, error: Some(e) }),
    }
}
