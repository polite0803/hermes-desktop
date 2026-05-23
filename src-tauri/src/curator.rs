// Curator — autonomous background skill library maintenance agent
use crate::{config, hermes_cli, ssh};

#[tauri::command]
pub fn curator_status() -> Result<String, String> {
    let conn = config::get_connection_config_raw()?;
    if conn.mode == "ssh" {
        let cmd = ssh::build_remote_hermes_cmd(&["curator", "status"]);
        return ssh::ssh_exec(&conn.ssh, &cmd, None, 10000);
    }
    hermes_cli::run_hermes_cli(&["curator", "status"], None)
}

#[tauri::command]
pub fn curator_trigger() -> Result<String, String> {
    let conn = config::get_connection_config_raw()?;
    if conn.mode == "ssh" {
        let cmd = ssh::build_remote_hermes_cmd(&["curator", "run"]);
        return ssh::ssh_exec(&conn.ssh, &cmd, None, 30000);
    }
    hermes_cli::run_hermes_cli(&["curator", "run"], None)
}

#[tauri::command]
pub fn curator_report() -> Result<String, String> {
    let conn = config::get_connection_config_raw()?;
    if conn.mode == "ssh" {
        return ssh::ssh_read_file(&conn.ssh, "~/.hermes/logs/curator/REPORT.md");
    }
    let log_dir = hermes_cli::resolve_hermes_home().join("logs").join("curator");
    if !log_dir.exists() { return Ok("curator.noReportsYet".into()); }
    let report = log_dir.join("REPORT.md");
    if report.exists() {
        std::fs::read_to_string(&report).map_err(|e| format!("{}", e))
    } else {
        let run = log_dir.join("run.json");
        if run.exists() {
            std::fs::read_to_string(&run).map_err(|e| format!("{}", e))
        } else {
            Ok("No curator reports yet".into())
        }
    }
}
