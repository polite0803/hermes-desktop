// Toolset management — 16 toolsets with enable/disable via config.yaml section editing
use serde::{Deserialize, Serialize};
use std::fs;
use crate::hermes_cli;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ToolsetInfo { pub key: String, pub label: String, pub description: String, pub enabled: bool }

const TOOLSET_DEFS: &[(&str, &str, &str)] = &[
    ("web", "Web Search", "Search the web for information"),
    ("browser", "Browser", "Browse web pages"),
    ("terminal", "Terminal", "Execute terminal commands"),
    ("file", "File Operations", "Read and write files"),
    ("code_execution", "Code Execution", "Execute code in sandbox"),
    ("vision", "Vision", "Analyze images"),
    ("image_gen", "Image Generation", "Generate images"),
    ("tts", "Text-to-Speech", "Convert text to speech"),
    ("skills", "Skills", "Use installed skills"),
    ("memory", "Memory", "Read and write memories"),
    ("session_search", "Session Search", "Search past conversations"),
    ("clarify", "Clarify", "Ask clarifying questions"),
    ("delegation", "Delegation", "Delegate tasks to agents"),
    ("cronjob", "Cron Jobs", "Schedule recurring tasks"),
    ("moa", "Mixture of Agents", "Use multiple agents"),
    ("todo", "Todo", "Manage todo lists"),
];

fn parse_enabled_toolsets(content: &str) -> Vec<String> {
    let mut enabled = Vec::new();
    let mut in_pts = false; let mut in_cli = false;
    for line in content.lines() {
        let t = line.trim_end();
        if t.trim_start().starts_with("platform_toolsets") && t.contains(':') { in_pts = true; in_cli = false; continue; }
        if in_pts && t.trim_start().starts_with("cli") && t.contains(':') && !t.contains("cli:") {} // skip if no match
        if in_pts && t.trim_start().starts_with("cli:") { in_cli = true; continue; }
        if in_pts && !t.starts_with(' ') && !t.is_empty() { in_pts = false; in_cli = false; continue; }
        if in_cli && t.starts_with("    ") && !t.trim_start().starts_with('-') && !t.trim().is_empty() { in_cli = false; continue; }
        if in_cli {
            if let Some(caps) = regex_lite::Regex::new(r#"^\s+-\s+["']?(\w+)["']?"#).ok().and_then(|re| re.captures(t)) {
                enabled.push(caps.get(1).unwrap().as_str().to_string());
            }
        }
    }
    enabled
}

#[tauri::command]
pub fn get_toolsets(profile: Option<String>) -> Result<Vec<ToolsetInfo>, String> {
    let config = hermes_cli::resolve_profile_home(profile.as_deref()).join("config.yaml");
    let enabled_set: Vec<String> = if config.exists() {
        fs::read_to_string(&config).ok().map(|c| parse_enabled_toolsets(&c)).unwrap_or_default()
    } else { Vec::new() };

    let all_enabled = enabled_set.is_empty() && !config.exists();
    Ok(TOOLSET_DEFS.iter().map(|(k, l, d)| ToolsetInfo {
        key: k.to_string(), label: l.to_string(), description: d.to_string(),
        enabled: all_enabled || enabled_set.contains(&k.to_string()),
    }).collect())
}

#[tauri::command]
pub fn set_toolset_enabled(name: String, enabled: bool, profile: Option<String>) -> Result<bool, String> {
    let config = hermes_cli::resolve_profile_home(profile.as_deref()).join("config.yaml");
    if !config.exists() { return Ok(false); }
    let content = fs::read_to_string(&config).map_err(|e| e.to_string())?;
    let mut current = parse_enabled_toolsets(&content);
    if enabled { if !current.contains(&name) { current.push(name); } }
    else { current.retain(|k| k != &name); }
    current.sort();

    let new_section = format!("  cli:\n{}", current.iter().map(|t| format!("      - {}", t)).collect::<Vec<_>>().join("\n"));

    let new_content = if content.contains("platform_toolsets") {
        let mut result = String::new(); let mut in_pts = false; let mut in_cli = false; let mut inserted = false;
        for line in content.lines() {
            let t = line.trim_end();
            if t.trim_start().starts_with("platform_toolsets") && t.contains(':') { in_pts = true; result.push_str(line); result.push('\n'); continue; }
            if in_pts && t.trim_start().starts_with("cli:") { in_cli = true; result.push_str(&new_section); result.push('\n'); inserted = true; continue; }
            if in_cli {
                if t.trim_start().starts_with('-') { continue; }
                if !t.starts_with(' ') || t.trim().is_empty() { in_cli = false; result.push_str(line); result.push('\n'); continue; }
                continue;
            }
            if in_pts && !t.starts_with(' ') && !t.is_empty() { in_pts = false; if !inserted { result.push_str(&new_section); result.push('\n'); inserted = true; } }
            result.push_str(line); result.push('\n');
        }
        result.trim_end().to_string() + "\n"
    } else {
        format!("{}\nplatform_toolsets:\n{}\n", content.trim_end(), new_section)
    };

    fs::write(&config, &new_content).map_err(|e| e.to_string())?;
    Ok(true)
}
