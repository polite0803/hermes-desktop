use serde::{Deserialize, Serialize};

use crate::hermes_cli;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct KanbanResult<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

// ── Boards ──

#[tauri::command]
pub fn kanban_list_boards(include_archived: Option<bool>, profile: Option<String>) -> Result<KanbanResult<Vec<serde_json::Value>>, String> {
    let mut args = vec!["kanban", "list-boards", "--json"];
    if include_archived.unwrap_or(false) { args.push("--include-archived"); }
    match hermes_cli::run_hermes_cli(&args, profile.as_deref()) {
        Ok(json) => {
            let data: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap_or_default();
            Ok(KanbanResult { success: true, data: Some(data), error: None })
        }
        Err(e) => Ok(KanbanResult { success: false, data: None, error: Some(e) }),
    }
}

#[tauri::command]
pub fn kanban_current_board(profile: Option<String>) -> Result<KanbanResult<serde_json::Value>, String> {
    match hermes_cli::run_hermes_cli(&["kanban", "current-board", "--json"], profile.as_deref()) {
        Ok(json) => {
            let data = serde_json::from_str(&json).unwrap_or(serde_json::Value::Null);
            Ok(KanbanResult { success: true, data: Some(data), error: None })
        }
        Err(e) => Ok(KanbanResult { success: false, data: None, error: Some(e) }),
    }
}

#[tauri::command]
pub fn kanban_switch_board(slug: String, profile: Option<String>) -> Result<KanbanResult<bool>, String> {
    match hermes_cli::run_hermes_cli(&["kanban", "switch-board", &slug], profile.as_deref()) {
        Ok(_) => Ok(KanbanResult { success: true, data: Some(true), error: None }),
        Err(e) => Ok(KanbanResult { success: false, data: None, error: Some(e) }),
    }
}

#[tauri::command]
pub fn kanban_create_board(slug: String, name: Option<String>, switch_after: Option<bool>, profile: Option<String>) -> Result<KanbanResult<serde_json::Value>, String> {
    let mut args = vec!["kanban", "create-board", "--slug", &slug, "--json"];
    if let Some(n) = &name { args.push("--name"); args.push(n); }
    if switch_after.unwrap_or(false) { args.push("--switch"); }
    match hermes_cli::run_hermes_cli(&args, profile.as_deref()) {
        Ok(json) => {
            let data = serde_json::from_str(&json).unwrap_or(serde_json::Value::Null);
            Ok(KanbanResult { success: true, data: Some(data), error: None })
        }
        Err(e) => Ok(KanbanResult { success: false, data: None, error: Some(e) }),
    }
}

#[tauri::command]
pub fn kanban_remove_board(slug: String, hard_delete: Option<bool>, profile: Option<String>) -> Result<KanbanResult<bool>, String> {
    let mut args = vec!["kanban", "remove-board", &slug];
    if hard_delete.unwrap_or(false) { args.push("--hard"); }
    match hermes_cli::run_hermes_cli(&args, profile.as_deref()) {
        Ok(_) => Ok(KanbanResult { success: true, data: Some(true), error: None }),
        Err(e) => Ok(KanbanResult { success: false, data: None, error: Some(e) }),
    }
}

// ── Tasks ──

#[tauri::command]
pub fn kanban_list_tasks(filters: Option<serde_json::Value>, profile: Option<String>) -> Result<KanbanResult<Vec<serde_json::Value>>, String> {
    let mut args = vec!["kanban", "list-tasks", "--json"];
    if let Some(f) = &filters {
        // Pass filters as individual args
        if let Some(s) = f.get("status").and_then(|v| v.as_str()) { args.push("--status"); args.push(s); }
        if let Some(a) = f.get("assignee").and_then(|v| v.as_str()) { args.push("--assignee"); args.push(a); }
        if let Some(t) = f.get("tenant").and_then(|v| v.as_str()) { args.push("--tenant"); args.push(t); }
        if f.get("includeArchived").and_then(|v| v.as_bool()).unwrap_or(false) { args.push("--include-archived"); }
    }
    match hermes_cli::run_hermes_cli(&args, profile.as_deref()) {
        Ok(json) => {
            let data: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap_or_default();
            Ok(KanbanResult { success: true, data: Some(data), error: None })
        }
        Err(e) => Ok(KanbanResult { success: false, data: None, error: Some(e) }),
    }
}

#[tauri::command]
pub fn kanban_get_task(task_id: String, profile: Option<String>) -> Result<KanbanResult<serde_json::Value>, String> {
    match hermes_cli::run_hermes_cli(&["kanban", "get-task", &task_id, "--json"], profile.as_deref()) {
        Ok(json) => {
            let data = serde_json::from_str(&json).unwrap_or(serde_json::Value::Null);
            Ok(KanbanResult { success: true, data: Some(data), error: None })
        }
        Err(e) => Ok(KanbanResult { success: false, data: None, error: Some(e) }),
    }
}

#[tauri::command]
pub fn kanban_create_task(input: serde_json::Value, profile: Option<String>) -> Result<KanbanResult<serde_json::Value>, String> {
    let mut args = vec!["kanban", "create-task", "--json"];
    if let Some(t) = input.get("title").and_then(|v| v.as_str()) { args.push("--title"); args.push(t); }
    if let Some(b) = input.get("body").and_then(|v| v.as_str()) { args.push("--body"); args.push(b); }
    if let Some(a) = input.get("assignee").and_then(|v| v.as_str()) { args.push("--assignee"); args.push(a); }
    let priority_str: Option<String> = input.get("priority").and_then(|v| v.as_i64()).map(|p| p.to_string());
    if let Some(ref p) = priority_str { args.push("--priority"); args.push(p); }
    if let Some(t) = input.get("tenant").and_then(|v| v.as_str()) { args.push("--tenant"); args.push(t); }
    if let Some(w) = input.get("workspace").and_then(|v| v.as_str()) { args.push("--workspace"); args.push(w); }
    if input.get("triage").and_then(|v| v.as_bool()).unwrap_or(false) { args.push("--triage"); }
    match hermes_cli::run_hermes_cli(&args, profile.as_deref()) {
        Ok(json) => {
            let data = serde_json::from_str(&json).unwrap_or(serde_json::Value::Null);
            Ok(KanbanResult { success: true, data: Some(data), error: None })
        }
        Err(e) => Ok(KanbanResult { success: false, data: None, error: Some(e) }),
    }
}

#[tauri::command]
pub fn kanban_assign_task(task_id: String, assignee: Option<String>, profile: Option<String>) -> Result<KanbanResult<bool>, String> {
    let mut args = vec!["kanban", "assign-task", &task_id];
    if let Some(a) = &assignee { args.push("--assignee"); args.push(a); }
    match hermes_cli::run_hermes_cli(&args, profile.as_deref()) {
        Ok(_) => Ok(KanbanResult { success: true, data: Some(true), error: None }),
        Err(e) => Ok(KanbanResult { success: false, data: None, error: Some(e) }),
    }
}

#[tauri::command]
pub fn kanban_complete_task(task_id: String, result: Option<String>, profile: Option<String>) -> Result<KanbanResult<bool>, String> {
    let mut args = vec!["kanban", "complete-task", &task_id];
    if let Some(r) = &result { args.push("--result"); args.push(r); }
    match hermes_cli::run_hermes_cli(&args, profile.as_deref()) {
        Ok(_) => Ok(KanbanResult { success: true, data: Some(true), error: None }),
        Err(e) => Ok(KanbanResult { success: false, data: None, error: Some(e) }),
    }
}

#[tauri::command]
pub fn kanban_block_task(task_id: String, reason: Option<String>, profile: Option<String>) -> Result<KanbanResult<bool>, String> {
    let mut args = vec!["kanban", "block-task", &task_id];
    if let Some(r) = &reason { args.push("--reason"); args.push(r); }
    match hermes_cli::run_hermes_cli(&args, profile.as_deref()) {
        Ok(_) => Ok(KanbanResult { success: true, data: Some(true), error: None }),
        Err(e) => Ok(KanbanResult { success: false, data: None, error: Some(e) }),
    }
}

#[tauri::command]
pub fn kanban_unblock_task(task_id: String, profile: Option<String>) -> Result<KanbanResult<bool>, String> {
    match hermes_cli::run_hermes_cli(&["kanban", "unblock-task", &task_id], profile.as_deref()) {
        Ok(_) => Ok(KanbanResult { success: true, data: Some(true), error: None }),
        Err(e) => Ok(KanbanResult { success: false, data: None, error: Some(e) }),
    }
}

#[tauri::command]
pub fn kanban_archive_task(task_id: String, profile: Option<String>) -> Result<KanbanResult<bool>, String> {
    match hermes_cli::run_hermes_cli(&["kanban", "archive-task", &task_id], profile.as_deref()) {
        Ok(_) => Ok(KanbanResult { success: true, data: Some(true), error: None }),
        Err(e) => Ok(KanbanResult { success: false, data: None, error: Some(e) }),
    }
}

#[tauri::command]
pub fn kanban_specify_task(task_id: String, profile: Option<String>) -> Result<KanbanResult<bool>, String> {
    match hermes_cli::run_hermes_cli(&["kanban", "specify-task", &task_id], profile.as_deref()) {
        Ok(_) => Ok(KanbanResult { success: true, data: Some(true), error: None }),
        Err(e) => Ok(KanbanResult { success: false, data: None, error: Some(e) }),
    }
}

#[tauri::command]
pub fn kanban_reclaim_task(task_id: String, reason: Option<String>, profile: Option<String>) -> Result<KanbanResult<bool>, String> {
    let mut args = vec!["kanban", "reclaim-task", &task_id];
    if let Some(r) = &reason { args.push("--reason"); args.push(r); }
    match hermes_cli::run_hermes_cli(&args, profile.as_deref()) {
        Ok(_) => Ok(KanbanResult { success: true, data: Some(true), error: None }),
        Err(e) => Ok(KanbanResult { success: false, data: None, error: Some(e) }),
    }
}

#[tauri::command]
pub fn kanban_comment_task(task_id: String, body: String, profile: Option<String>) -> Result<KanbanResult<serde_json::Value>, String> {
    match hermes_cli::run_hermes_cli(&["kanban", "comment-task", &task_id, "--body", &body, "--json"], profile.as_deref()) {
        Ok(json) => {
            let data = serde_json::from_str(&json).unwrap_or(serde_json::Value::Null);
            Ok(KanbanResult { success: true, data: Some(data), error: None })
        }
        Err(e) => Ok(KanbanResult { success: false, data: None, error: Some(e) }),
    }
}

#[tauri::command]
pub fn kanban_dispatch_once(dry_run: Option<bool>, profile: Option<String>) -> Result<KanbanResult<bool>, String> {
    let mut args = vec!["kanban", "dispatch-once"];
    if dry_run.unwrap_or(false) { args.push("--dry-run"); }
    match hermes_cli::run_hermes_cli(&args, profile.as_deref()) {
        Ok(_) => Ok(KanbanResult { success: true, data: Some(true), error: None }),
        Err(e) => Ok(KanbanResult { success: false, data: None, error: Some(e) }),
    }
}
