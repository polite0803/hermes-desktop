// MCP Server management — add, remove, list, test MCP servers (mcp.json)
use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;
use crate::{config, hermes_cli, ssh};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct McpServer {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub enabled: bool,
}

fn mcp_config_path() -> std::path::PathBuf {
    hermes_cli::resolve_hermes_home().join("mcp.json")
}

fn read_mcp_config() -> Vec<McpServer> {
    let path = mcp_config_path();
    if !path.exists() { return Vec::new(); }
    fs::read_to_string(&path).ok()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        .and_then(|v| v.get("servers").cloned())
        .and_then(|s| serde_json::from_value::<Vec<McpServer>>(s).ok())
        .unwrap_or_default()
}

fn write_mcp_config(servers: &[McpServer]) -> Result<(), String> {
    let path = mcp_config_path();
    if let Some(p) = path.parent() { fs::create_dir_all(p).map_err(|e| e.to_string())?; }
    let json = serde_json::json!({ "servers": servers });
    fs::write(&path, serde_json::to_string_pretty(&json).unwrap_or_default())
        .map_err(|e| e.to_string())
}

fn read_mcp_config_via_ssh(config: &config::SshConfig) -> Result<Vec<McpServer>, String> {
    let content = ssh::ssh_read_file(config, "~/.hermes/mcp.json").unwrap_or_default();
    if content.is_empty() { return Ok(Vec::new()); }
    serde_json::from_str::<serde_json::Value>(&content).ok()
        .and_then(|v| v.get("servers").cloned())
        .and_then(|s| serde_json::from_value::<Vec<McpServer>>(s).ok())
        .ok_or_else(|| "Failed to parse remote mcp.json".into())
}

fn write_mcp_config_via_ssh(config: &config::SshConfig, servers: &[McpServer]) -> Result<(), String> {
    let json = serde_json::json!({ "servers": servers });
    let content = serde_json::to_string_pretty(&json).unwrap_or_default();
    ssh::ssh_write_file(config, "~/.hermes/mcp.json", &content)
}

#[tauri::command]
pub fn list_mcp_servers() -> Result<Vec<McpServer>, String> {
    let conn = config::get_connection_config_raw()?;
    if conn.mode == "ssh" {
        let raw = ssh::ssh_list_mcp_servers(&conn.ssh, None)?;
        return Ok(raw.iter().map(|v| McpServer {
            name: v.get("name").and_then(|s| s.as_str()).unwrap_or("").to_string(),
            command: v.get("command").and_then(|s| s.as_str()).unwrap_or("").to_string(),
            args: v.get("args").and_then(|a| a.as_array()).map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()).unwrap_or_default(),
            enabled: v.get("enabled").and_then(|b| b.as_bool()).unwrap_or(true),
        }).collect());
    }
    Ok(read_mcp_config())
}

#[tauri::command]
pub fn add_mcp_server(name: String, command: String, args: Vec<String>) -> Result<McpServer, String> {
    let conn = config::get_connection_config_raw()?;
    if conn.mode == "ssh" {
        let mut servers = read_mcp_config_via_ssh(&conn.ssh)?;
        if servers.iter().any(|s| s.name == name) {
            return Err("mcp.serverExists".into());
        }
        let server = McpServer { name: name.clone(), command, args, enabled: true };
        servers.push(server.clone());
        write_mcp_config_via_ssh(&conn.ssh, &servers)?;
        return Ok(server);
    }
    let mut servers = read_mcp_config();
    if servers.iter().any(|s| s.name == name) {
        return Err("mcp.serverExists".into());
    }
    let server = McpServer { name: name.clone(), command, args, enabled: true };
    servers.push(server.clone());
    write_mcp_config(&servers)?;
    Ok(server)
}

#[tauri::command]
pub fn remove_mcp_server(name: String) -> Result<(), String> {
    let conn = config::get_connection_config_raw()?;
    if conn.mode == "ssh" {
        let mut servers = read_mcp_config_via_ssh(&conn.ssh)?;
        servers.retain(|s| s.name != name);
        return write_mcp_config_via_ssh(&conn.ssh, &servers);
    }
    let mut servers = read_mcp_config();
    servers.retain(|s| s.name != name);
    write_mcp_config(&servers)
}

#[tauri::command]
pub fn update_mcp_server(name: String, command: Option<String>, args: Option<Vec<String>>, enabled: Option<bool>) -> Result<McpServer, String> {
    let conn = config::get_connection_config_raw()?;
    if conn.mode == "ssh" {
        let mut servers = read_mcp_config_via_ssh(&conn.ssh)?;
        let idx = servers.iter().position(|s| s.name == name).ok_or_else(|| format!("Server '{}' not found", name))?;
        if let Some(c) = command { servers[idx].command = c; }
        if let Some(a) = args { servers[idx].args = a; }
        if let Some(e) = enabled { servers[idx].enabled = e; }
        write_mcp_config_via_ssh(&conn.ssh, &servers)?;
        return Ok(servers[idx].clone());
    }
    let mut servers = read_mcp_config();
    let idx = servers.iter().position(|s| s.name == name).ok_or_else(|| format!("Server '{}' not found", name))?;
    if let Some(c) = command { servers[idx].command = c; }
    if let Some(a) = args { servers[idx].args = a; }
    if let Some(e) = enabled { servers[idx].enabled = e; }
    write_mcp_config(&servers)?;
    Ok(servers[idx].clone())
}

#[tauri::command]
pub fn test_mcp_server(name: String) -> Result<bool, String> {
    let servers = read_mcp_config();
    let server = servers.iter().find(|s| s.name == name).ok_or("Server not found")?;
    let mut cmd = Command::new(&server.command);
    cmd.args(&server.args)
        .stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null());
    hermes_cli::hide_window(&mut cmd);
    let mut child = cmd.spawn().map_err(|e| format!("{}", e))?;
    std::thread::sleep(std::time::Duration::from_millis(1500));
    let _ = child.kill();
    Ok(true)
}

#[tauri::command]
pub fn install_computer_use_mcp() -> Result<bool, String> {
    #[cfg(not(target_os = "linux"))]
    { return Err("mcp.computerUseLinuxOnly".into()); }

    #[cfg(target_os = "linux")]
    {
        let dir = hermes_cli::resolve_hermes_home().join("mcp-servers").join("computer-use-linux");
        if !dir.exists() {
            std::fs::create_dir_all(dir.parent().unwrap_or(&dir)).map_err(|e| e.to_string())?;
            let status = Command::new("git").args(&["clone", "https://github.com/avifenesh/computer-use-linux", &dir.to_string_lossy().to_string()])
                .status().map_err(|e| e.to_string())?;
            if !status.success() { return Err("mcp.cloneFailed".into()); }
        }
        // Add to MCP config
        let mut servers = read_mcp_config();
        if servers.iter().any(|s| s.name == "computer-use") { return Ok(true); }
        servers.push(McpServer {
            name: "computer-use".into(),
            command: "python".into(),
            args: vec!["-m".into(), "computer_use".into()],
            enabled: true,
        });
        write_mcp_config(&servers)?;
        Ok(true)
    }
}
