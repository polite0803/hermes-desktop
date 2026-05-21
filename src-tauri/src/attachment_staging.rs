// Attachment staging — stage base64 attachments to disk, clear per session
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use base64::Engine;
use crate::hermes_cli;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StagedAttachment { pub id: String, pub filename: String, pub kind: String, pub mime: String, pub size: u64, #[serde(rename = "stagedPath")] pub staged_path: String }

static STAGING: once_cell::sync::Lazy<Mutex<HashMap<String, Vec<StagedAttachment>>>> = once_cell::sync::Lazy::new(|| Mutex::new(HashMap::new()));

fn staging_dir(session_id: &str) -> PathBuf { hermes_cli::resolve_hermes_home().join("staging").join(session_id) }

#[tauri::command]
pub fn stage_attachment(session_id: String, filename: String, kind: Option<String>, mime: Option<String>, base64_content: Option<String>) -> Result<StagedAttachment, String> {
    let kind = kind.unwrap_or_else(|| "file".into());
    let mime = mime.unwrap_or_else(|| "application/octet-stream".into());
    let b64 = base64_content.unwrap_or_default();

    let bytes = base64::engine::general_purpose::STANDARD.decode(&b64).map_err(|e| format!("Base64 decode failed: {}", e))?;
    let size = bytes.len() as u64;

    let dir = staging_dir(&session_id);
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create staging dir: {}", e))?;

    let safe_name = filename.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
    let staged_path = dir.join(&safe_name);
    fs::write(&staged_path, &bytes).map_err(|e| format!("Failed to write staged file: {}", e))?;

    let id = uuid::Uuid::new_v4().to_string();
    let attachment = StagedAttachment { id: id.clone(), filename: safe_name.clone(), kind, mime, size, staged_path: staged_path.to_string_lossy().to_string() };

    if let Ok(mut staging) = STAGING.lock() {
        staging.entry(session_id).or_default().push(attachment.clone());
    }

    Ok(attachment)
}

#[tauri::command]
pub fn clear_staged_attachments(session_id: String) -> Result<(), String> {
    // Remove from memory
    if let Ok(mut staging) = STAGING.lock() { staging.remove(&session_id); }
    // Clean up disk
    let dir = staging_dir(&session_id);
    if dir.exists() { let _ = fs::remove_dir_all(&dir); }
    Ok(())
}

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn test_staging_dir_resolution() { let d = staging_dir("sess-1"); assert!(d.to_string_lossy().contains("staging")); assert!(d.to_string_lossy().contains("sess-1")); }
}
