use std::path::PathBuf;
use std::process::Command;

/// Resolve HERMES_HOME directory
/// Priority: env var -> profile config -> default locations
pub fn resolve_hermes_home() -> PathBuf {
    if let Ok(home) = std::env::var("HERMES_HOME") {
        return PathBuf::from(home);
    }

    // Default locations (platform-specific)
    #[cfg(target_os = "windows")]
    {
        // Prefer LOCALAPPDATA\hermes (install.ps1 default), fall back to USERPROFILE\.hermes
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            let p = PathBuf::from(&local).join("hermes");
            if p.join("hermes-agent").exists() || p.join("config.yaml").exists() {
                return p;
            }
        }
        let base = std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\".to_string());
        PathBuf::from(base).join(".hermes")
    }

    #[cfg(not(target_os = "windows"))]
    {
        let base = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        return PathBuf::from(base).join(".hermes");
    }
}

/// Resolve profile home directory
pub fn resolve_profile_home(profile: Option<&str>) -> PathBuf {
    let hermes_home = resolve_hermes_home();
    match profile {
        Some(name) => hermes_home.join("profiles").join(name),
        None => hermes_home.clone(),
    }
}

/// Resolve Python executable path from Hermes venv
pub fn resolve_python() -> PathBuf {
    let hermes_home = resolve_hermes_home();
    let repo_dir = hermes_home.join("hermes-agent");
    // Check both possible locations: hermes-agent/venv (install.sh) and venv (legacy)
    let venv_agent = repo_dir.join("venv");
    let venv_legacy = hermes_home.join("venv");

    #[cfg(target_os = "windows")]
    {
        let p = venv_agent.join("Scripts").join("python.exe");
        if p.exists() { return p; }
        venv_legacy.join("Scripts").join("python.exe")
    }

    #[cfg(not(target_os = "windows"))]
    {
        let p = venv_agent.join("bin").join("python3");
        if p.exists() { return p; }
        let p = venv_agent.join("bin").join("python");
        if p.exists() { return p; }
        venv_legacy.join("bin").join("python3")
    }
}

/// Resolve the Hermes agent script path
pub fn resolve_hermes_script() -> PathBuf {
    let hermes_home = resolve_hermes_home();
    let repo_dir = hermes_home.join("hermes-agent");

    #[cfg(target_os = "windows")]
    {
        let p = repo_dir.join("venv").join("Scripts").join("hermes.exe");
        if p.exists() { return p; }
        hermes_home.join("hermes")
    }

    #[cfg(not(target_os = "windows"))]
    {
        let p = repo_dir.join("hermes");
        if p.exists() { return p; }
        let p = repo_dir.join("venv").join("bin").join("hermes");
        if p.exists() { return p; }
        hermes_home.join("hermes")
    }
}

/// Run Hermes CLI command and capture stdout
pub fn run_hermes_cli(args: &[&str], profile: Option<&str>) -> Result<String, String> {
    let python = resolve_python();
    let script = resolve_hermes_script();

    if !python.exists() || !script.exists() {
        return Err("Hermes Agent is not installed. Please complete the installation first.".into());
    }

    let mut cmd = Command::new(&python);
    cmd.arg(&script);
    for arg in args {
        cmd.arg(arg);
    }

    // Set HERMES_HOME for the subprocess
    cmd.env("HERMES_HOME", resolve_hermes_home());

    if let Some(p) = profile {
        cmd.env("HERMES_PROFILE", p);
    }

    let output = cmd.output().map_err(|e| format!("Failed to execute hermes CLI: {}", e))?;

    if output.status.success() {
        String::from_utf8(output.stdout)
            .map(|s| s.trim().to_string())
            .map_err(|e| format!("Invalid UTF-8 output: {}", e))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Hermes CLI error: {}", stderr.trim()))
    }
}

/// Run Hermes CLI with stdin input and capture stdout
pub fn run_hermes_cli_with_input(
    args: &[&str],
    input: &str,
    profile: Option<&str>,
) -> Result<String, String> {
    let python = resolve_python();
    let script = resolve_hermes_script();

    let mut cmd = Command::new(&python);
    cmd.arg(&script);
    for arg in args {
        cmd.arg(arg);
    }
    cmd.env("HERMES_HOME", resolve_hermes_home());
    if let Some(p) = profile {
        cmd.env("HERMES_PROFILE", p);
    }

    // Write input to stdin
    let mut child = cmd.spawn().map_err(|e| format!("Failed to spawn: {}", e))?;
    use std::io::Write;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(input.as_bytes()).map_err(|e| format!("Failed to write stdin: {}", e))?;
    }
    let output = child.wait_with_output().map_err(|e| format!("Failed to wait: {}", e))?;

    if output.status.success() {
        String::from_utf8(output.stdout)
            .map(|s| s.trim().to_string())
            .map_err(|e| format!("Invalid UTF-8: {}", e))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Hermes CLI error: {}", stderr.trim()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_hermes_home() {
        let home = resolve_hermes_home();
        assert!(!home.as_os_str().is_empty());
    }

    #[test]
    fn test_resolve_profile_home_default() {
        let path = resolve_profile_home(None);
        assert_eq!(path, resolve_hermes_home());
    }

    #[test]
    fn test_resolve_profile_home_named() {
        let path = resolve_profile_home(Some("test-profile"));
        assert!(path.to_string_lossy().contains("test-profile"));
    }
}
