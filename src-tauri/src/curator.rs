// Curator — autonomous background skill library maintenance agent
use crate::hermes_cli;

#[tauri::command]
pub fn curator_status() -> Result<String, String> {
    hermes_cli::run_hermes_cli(&["curator", "status"], None)
}

#[tauri::command]
pub fn curator_trigger() -> Result<String, String> {
    hermes_cli::run_hermes_cli(&["curator", "run"], None)
}

#[tauri::command]
pub fn curator_report() -> Result<String, String> {
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
