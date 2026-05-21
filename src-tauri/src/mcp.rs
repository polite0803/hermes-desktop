// MCP Server management — add, remove, list, test MCP servers (mcp.json)
use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;
use crate::hermes_cli;

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

#[tauri::command]
pub fn list_mcp_servers() -> Result<Vec<McpServer>, String> {
    Ok(read_mcp_config())
}

#[tauri::command]
pub fn add_mcp_server(name: String, command: String, args: Vec<String>) -> Result<McpServer, String> {
    let mut servers = read_mcp_config();
    if servers.iter().any(|s| s.name == name) {
        return Err(format!("Server '{}' already exists", name));
    }
    let server = McpServer { name: name.clone(), command, args, enabled: true };
    servers.push(server.clone());
    write_mcp_config(&servers)?;
    Ok(server)
}

#[tauri::command]
pub fn remove_mcp_server(name: String) -> Result<(), String> {
    let mut servers = read_mcp_config();
    servers.retain(|s| s.name != name);
    write_mcp_config(&servers)
}

#[tauri::command]
pub fn update_mcp_server(name: String, command: Option<String>, args: Option<Vec<String>>, enabled: Option<bool>) -> Result<McpServer, String> {
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
    let mut child = Command::new(&server.command).args(&server.args)
        .stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null())
        .spawn().map_err(|e| format!("{}", e))?;
    std::thread::sleep(std::time::Duration::from_millis(1500));
    let _ = child.kill();
    Ok(true)
}

#[tauri::command]
pub fn install_computer_use_mcp() -> Result<bool, String> {
    #[cfg(not(target_os = "linux"))]
    { return Err("Computer-use MCP is only supported on Linux".into()); }

    #[cfg(target_os = "linux")]
    {
        let dir = hermes_cli::resolve_hermes_home().join("mcp-servers").join("computer-use-linux");
        if !dir.exists() {
            std::fs::create_dir_all(dir.parent().unwrap_or(&dir)).map_err(|e| e.to_string())?;
            let status = Command::new("git").args(&["clone", "https://github.com/avifenesh/computer-use-linux", &dir.to_string_lossy().to_string()])
                .status().map_err(|e| e.to_string())?;
            if !status.success() { return Err("Failed to clone repository".into()); }
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
