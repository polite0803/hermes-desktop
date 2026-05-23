// Skills management — install, uninstall, list, search, frontmatter parsing
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use crate::{config, hermes_cli, ssh};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InstalledSkill { pub name: String, pub category: String, pub description: String, pub path: String }

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BundledSkill { pub name: String, pub description: String, pub category: String, pub source: String, pub installed: bool }

fn skills_dir(profile: Option<&str>) -> PathBuf { hermes_cli::resolve_profile_home(profile).join("skills") }
fn bundled_dir() -> PathBuf { hermes_cli::resolve_hermes_home().join("hermes-agent").join("skills") }

fn parse_skill_frontmatter(content: &str) -> (String, String) {
    if !content.starts_with("---") {
        let name = content.lines().find(|l| l.starts_with("# ")).map(|l| l[2..].trim().to_string()).unwrap_or_default();
        let desc = content.lines().find(|l| !l.starts_with('#') && !l.starts_with("---") && !l.trim().is_empty())
            .map(|l| l.trim().chars().take(120).collect()).unwrap_or_default();
        return (name, desc);
    }
    let end = content[3..].find("---").map(|i| i + 3).unwrap_or(content.len());
    let fm = &content[3..end];
    let name = fm.lines().find(|l| l.trim().starts_with("name:")).and_then(|l| l.split_once(':')).map(|(_, v)| v.trim().trim_matches('"').trim_matches('\'').to_string()).unwrap_or_default();
    let desc = fm.lines().find(|l| l.trim().starts_with("description:")).and_then(|l| l.split_once(':')).map(|(_, v)| v.trim().trim_matches('"').trim_matches('\'').to_string()).unwrap_or_default();
    (name, desc)
}

fn walk_skill_dirs(base: &PathBuf) -> Vec<(String, String, String, PathBuf)> {
    let mut results = Vec::new();
    if !base.exists() { return results; }
    if let Ok(categories) = fs::read_dir(base) {
        for cat in categories.filter_map(|e| e.ok()) {
            if !cat.file_type().map(|t| t.is_dir()).unwrap_or(false) { continue; }
            let cat_name = cat.file_name().to_string_lossy().to_string();
            if let Ok(entries) = fs::read_dir(cat.path()) {
                for entry in entries.filter_map(|e| e.ok()) {
                    if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) { continue; }
                    let skill_name = entry.file_name().to_string_lossy().to_string();
                    let skill_file = entry.path().join("SKILL.md");
                    if skill_file.exists() {
                        if let Ok(c) = fs::read_to_string(&skill_file) {
                            let (n, d) = parse_skill_frontmatter(&c[..c.len().min(4000)]);
                            results.push((if n.is_empty() { skill_name.clone() } else { n }, cat_name.clone(), d, entry.path()));
                        } else { results.push((skill_name.clone(), cat_name.clone(), String::new(), entry.path())); }
                    }
                }
            }
        }
    }
    results.sort_by(|a,b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));
    results
}

#[tauri::command] pub fn list_installed_skills(profile: Option<String>) -> Result<Vec<InstalledSkill>, String> {
    let conn = config::get_connection_config_raw()?;
    if conn.mode == "ssh" {
        let raw = ssh::ssh_list_installed_skills(&conn.ssh, profile.as_deref())?;
        return Ok(raw.iter().map(|v| InstalledSkill {
            name: v.get("name").and_then(|s| s.as_str()).unwrap_or("").to_string(),
            category: v.get("category").and_then(|s| s.as_str()).unwrap_or("").to_string(),
            description: v.get("description").and_then(|s| s.as_str()).unwrap_or("").to_string(),
            path: v.get("path").and_then(|s| s.as_str()).unwrap_or("").to_string(),
        }).collect());
    }
    Ok(walk_skill_dirs(&skills_dir(profile.as_deref())).into_iter().map(|(name,category,description,path)| InstalledSkill { name, category, description, path: path.to_string_lossy().to_string() }).collect())
}
#[tauri::command] pub fn list_bundled_skills() -> Result<Vec<BundledSkill>, String> {
    let conn = config::get_connection_config_raw()?;
    if conn.mode == "ssh" {
        return Ok(Vec::new()); // bundled skills are local-only
    }
    Ok(walk_skill_dirs(&bundled_dir()).into_iter().map(|(name,category,description,_)| BundledSkill { name, description, category, source: "bundled".into(), installed: false }).collect())
}
#[tauri::command] pub fn get_skill_content(path: String) -> Result<String, String> {
    let conn = config::get_connection_config_raw()?;
    if conn.mode == "ssh" {
        let remote_path = format!("{}/SKILL.md", path);
        return Ok(ssh::ssh_read_file(&conn.ssh, &remote_path).unwrap_or_default());
    }
    let f = PathBuf::from(&path).join("SKILL.md"); if !f.exists() { return Ok(String::new()); } fs::read_to_string(&f).map_err(|e| e.to_string())
}
#[tauri::command] pub fn install_skill(name: String, profile: Option<String>) -> Result<serde_json::Value, String> {
    let conn = config::get_connection_config_raw()?;
    if conn.mode == "ssh" {
        let dash_p = "-p".to_string();
        let mut ssh_args: Vec<&str> = vec!["skills", "install", &name, "--yes"];
        if let Some(ref p) = profile { if p != "default" { ssh_args.push(&dash_p); ssh_args.push(p); } }
        let cmd = ssh::build_remote_hermes_cmd(&ssh_args);
        match ssh::ssh_exec(&conn.ssh, &cmd, None, 30000) {
            Ok(_) => return Ok(serde_json::json!({"success":true})),
            Err(e) => return Ok(serde_json::json!({"success":false,"error":e})),
        }
    }
    let mut args: Vec<&str> = vec!["skills","install",&name,"--yes"];
    if let Some(ref p) = profile { if p != "default" { args.push("-p"); args.push(p); } }
    match hermes_cli::run_hermes_cli(&args, profile.as_deref()) { Ok(_) => Ok(serde_json::json!({"success":true})), Err(e) => Ok(serde_json::json!({"success":false,"error":e})) }
}
#[tauri::command] pub fn uninstall_skill(name: String, profile: Option<String>) -> Result<serde_json::Value, String> {
    let conn = config::get_connection_config_raw()?;
    if conn.mode == "ssh" {
        let dash_p = "-p".to_string();
        let mut ssh_args: Vec<&str> = vec!["skills", "uninstall", &name];
        if let Some(ref p) = profile { if p != "default" { ssh_args.push(&dash_p); ssh_args.push(p); } }
        let cmd = ssh::build_remote_hermes_cmd(&ssh_args);
        match ssh::ssh_exec(&conn.ssh, &cmd, None, 15000) {
            Ok(_) => return Ok(serde_json::json!({"success":true})),
            Err(e) => return Ok(serde_json::json!({"success":false,"error":e})),
        }
    }
    let mut args: Vec<&str> = vec!["skills","uninstall",&name];
    if let Some(ref p) = profile { if p != "default" { args.push("-p"); args.push(p); } }
    match hermes_cli::run_hermes_cli(&args, profile.as_deref()) { Ok(_) => Ok(serde_json::json!({"success":true})), Err(e) => Ok(serde_json::json!({"success":false,"error":e})) }
}

#[cfg(test)] mod tests { use super::*;
    #[test] fn test_parse_frontmatter() { let (n,d)=parse_skill_frontmatter("---\nname: test-skill\ndescription: A test skill\n---\n# Body"); assert_eq!(n,"test-skill"); assert_eq!(d,"A test skill"); }
}
