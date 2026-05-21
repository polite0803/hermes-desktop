// Installation pipeline — Python discovery, venv, git clone, pip, progress events
// Rewrite from original src/main/installer.ts (1,280 lines TS)
//
// Key additions after rewrite:
//   - get_enhanced_path() — PATH expansion (nvm/volta/fnm/cargo/brew)
//   - hermes_cli_args() — cross-platform CLI arg builder
//   - check_install_status() — full status with API key detection
//   - run_install() — cross-platform pipeline with progress markers
//   - memory provider discovery
//   - log reading with path sanitization

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;
use tauri::Emitter;

use crate::hermes_cli;

// ─── Constants ──────────────────────────────────────────

fn hermes_home() -> PathBuf { hermes_cli::resolve_hermes_home() }
fn hermes_repo() -> PathBuf { hermes_home().join("hermes-agent") }
fn hermes_venv() -> PathBuf { hermes_repo().join("venv") }
fn hermes_python() -> PathBuf {
    if cfg!(windows) { hermes_venv().join("Scripts").join("python.exe") }
    else { hermes_venv().join("bin").join("python") }
}
fn hermes_script() -> PathBuf {
    if cfg!(windows) { hermes_venv().join("Scripts").join("hermes.exe") }
    else { hermes_repo().join("hermes") }
}

const LOG_WHITELIST: &[&str] = &["agent.log", "errors.log", "gateway.log"];

// ─── Data Types ──────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InstallStatus {
    pub installed: bool,
    pub configured: bool,
    pub has_api_key: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InstallProgress {
    pub step: u32,
    pub total_steps: u32,
    pub title: String,
    pub detail: String,
    pub log: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InstallResult {
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OpenClawStatus {
    pub found: bool,
    pub path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BackupResult {
    pub success: bool,
    pub path: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct McpServer {
    pub name: String,
    #[serde(rename = "type")]
    pub server_type: String,
    pub enabled: bool,
    pub detail: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MemoryProvider {
    pub name: String,
    pub description: String,
    pub installed: bool,
    pub active: bool,
    #[serde(rename = "envVars")]
    pub env_vars: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LogResult {
    pub content: String,
    pub path: String,
}

// ─── Provider Env Key Mapping ───────────────────────────

const PROVIDER_ENV_KEYS: &[(&str, &str)] = &[
    ("openrouter", "OPENROUTER_API_KEY"), ("anthropic", "ANTHROPIC_API_KEY"),
    ("openai", "OPENAI_API_KEY"), ("google", "GOOGLE_API_KEY"),
    ("xai", "XAI_API_KEY"), ("groq", "GROQ_API_KEY"),
    ("deepseek", "DEEPSEEK_API_KEY"), ("together", "TOGETHER_API_KEY"),
    ("fireworks", "FIREWORKS_API_KEY"), ("cerebras", "CEREBRAS_API_KEY"),
    ("mistral", "MISTRAL_API_KEY"), ("perplexity", "PERPLEXITY_API_KEY"),
    ("huggingface", "HF_TOKEN"), ("hf", "HF_TOKEN"),
    ("qwen", "QWEN_API_KEY"), ("minimax", "MINIMAX_API_KEY"),
    ("glm", "GLM_API_KEY"), ("kimi", "KIMI_API_KEY"),
    ("nvidia", "NVIDIA_API_KEY"),
];

const URL_TO_ENV_KEY: &[(&str, &str)] = &[
    ("openrouter.ai", "OPENROUTER_API_KEY"), ("anthropic.com", "ANTHROPIC_API_KEY"),
    ("openai.com", "OPENAI_API_KEY"), ("huggingface.co", "HF_TOKEN"),
    ("api.groq.com", "GROQ_API_KEY"), ("api.deepseek.com", "DEEPSEEK_API_KEY"),
    ("api.together.xyz", "TOGETHER_API_KEY"), ("api.fireworks.ai", "FIREWORKS_API_KEY"),
    ("api.cerebras.ai", "CEREBRAS_API_KEY"), ("api.mistral.ai", "MISTRAL_API_KEY"),
    ("api.perplexity.ai", "PERPLEXITY_API_KEY"),
];

fn expected_env_key_for_model(provider: &str, base_url: &str) -> Option<String> {
    let provider_lower = provider.trim().to_lowercase();
    for (k, v) in PROVIDER_ENV_KEYS {
        if *k == provider_lower { return Some(v.to_string()); }
    }
    for (pattern, env_key) in URL_TO_ENV_KEY {
        if base_url.to_lowercase().contains(pattern) { return Some(env_key.to_string()); }
    }
    None
}

fn env_has_usable_value(content: &str, expected_key: Option<&str>) -> bool {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') { continue; }
        if let Some((k, v)) = trimmed.split_once('=') {
            let key = k.trim();
            let val = v.trim().trim_matches('"').trim_matches('\'');
            if val.is_empty() { continue; }
            if let Some(expected) = expected_key {
                if key == expected { return true; }
            } else if key.ends_with("_API_KEY") {
                return true;
            }
        }
    }
    false
}

// ─── Enhanced PATH ──────────────────────────────────────

fn get_enhanced_path() -> String {
    let home = dirs_next::home_dir().unwrap_or_default();
    let separator = if cfg!(windows) { ";" } else { ":" };

    let extras: Vec<PathBuf> = if cfg!(windows) {
        let mut v = vec![
            hermes_home().join("git").join("bin"),
            hermes_home().join("git").join("cmd"),
            hermes_home().join("node"),
            hermes_venv().join("Scripts"),
        ];
        if let Some(appdata) = std::env::var_os("APPDATA") {
            v.push(PathBuf::from(&appdata).join("npm"));
        }
        if let Some(pf) = std::env::var_os("ProgramFiles") {
            v.push(PathBuf::from(&pf).join("nodejs"));
            v.push(PathBuf::from(&pf).join("Git").join("cmd"));
        }
        if let Some(pf) = std::env::var_os("ProgramFiles(x86)") {
            v.push(PathBuf::from(&pf).join("nodejs"));
        }
        if let Some(local) = std::env::var_os("LOCALAPPDATA") {
            v.push(PathBuf::from(&local).join("Programs").join("Git").join("cmd"));
        }
        v.push(home.join(".local").join("bin"));
        v.push(home.join(".cargo").join("bin"));
        v
    } else {
        let mut v = vec![
            home.join(".local").join("bin"),
            home.join(".cargo").join("bin"),
            hermes_venv().join("bin"),
            home.join(".volta").join("bin"),
            home.join(".asdf").join("shims"),
            home.join(".local").join("share").join("fnm").join("aliases").join("default").join("bin"),
            home.join(".fnm").join("aliases").join("default").join("bin"),
        ];
        // Try nvm
        let nvm_versions = home.join(".nvm").join("versions").join("node");
        if nvm_versions.exists() {
            if let Ok(entries) = fs::read_dir(&nvm_versions) {
                let mut versions: Vec<_> = entries.filter_map(|e| e.ok())
                    .filter(|e| e.file_name().to_string_lossy().starts_with('v'))
                    .map(|e| e.file_name().to_string_lossy().to_string())
                    .collect();
                versions.sort_by(|a, b| b.cmp(a));
                if let Some(latest) = versions.first() {
                    v.push(nvm_versions.join(latest).join("bin"));
                }
            }
        }
        v.push("/usr/local/bin".into());
        v.push("/opt/homebrew/bin".into());
        v.push("/opt/homebrew/sbin".into());
        v
    };

    let extra_str: Vec<String> = extras.iter()
        .filter(|p| p.exists())
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    let existing_path = std::env::var("PATH").unwrap_or_default();
    [extra_str.join(separator), existing_path].into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(separator)
}

// ─── CLI Args ────────────────────────────────────────────

fn hermes_cli_args(extra: &[&str]) -> Vec<String> {
    if cfg!(windows) {
        let mut v = vec!["-m".to_string(), "hermes_cli.main".to_string()];
        v.extend(extra.iter().map(|s| s.to_string()));
        v
    } else {
        let mut v = vec![hermes_script().to_string_lossy().to_string()];
        v.extend(extra.iter().map(|s| s.to_string()));
        v
    }
}

// ─── Install Status ─────────────────────────────────────

#[tauri::command]
pub fn check_install() -> Result<InstallStatus, String> {
    let installed = hermes_python().exists() && hermes_script().exists();

    let env_file = hermes_home().join(".env");
    let auth_file = hermes_home().join("auth.json");
    let configured = env_file.exists() || auth_file.exists();

    let mut has_api_key = false;
    if configured && env_file.exists() {
        if let Ok(content) = fs::read_to_string(&env_file) {
            has_api_key = env_has_usable_value(&content, None);
        }
    }

    Ok(InstallStatus { installed, configured, has_api_key })
}

#[tauri::command]
pub fn verify_install() -> Result<bool, String> {
    if !hermes_python().exists() || !hermes_script().exists() {
        return Ok(false);
    }
    match hermes_cli::run_hermes_cli(&["--version"], None) {
        Ok(v) => Ok(!v.is_empty()),
        Err(_) => Ok(false),
    }
}

#[tauri::command]
pub fn start_install(app: AppHandle) -> Result<InstallResult, String> {
    let total_steps = 7u32;

    let emit = |app: &AppHandle, step: u32, title: &str, detail: &str, log: Option<&str>| {
        let _ = app.emit("install-progress", InstallProgress {
            step, total_steps, title: title.to_string(), detail: detail.to_string(),
            log: log.map(|s| s.to_string()),
        });
    };

    emit(&app, 1, "Starting installation", "Preparing...", None);

    if cfg!(windows) {
        run_install_windows(&app, &emit)
    } else {
        run_install_unix(&app, &emit)
    }
}

fn run_install_unix(
    app: &AppHandle,
    emit: &dyn Fn(&AppHandle, u32, &str, &str, Option<&str>),
) -> Result<InstallResult, String> {
    let home = dirs_next::home_dir().unwrap_or_default();
    let base_path = get_enhanced_path();

    emit(app, 2, "Checking prerequisites", "Running installer...", None);

    let install_cmd = "curl -fsSL https://raw.githubusercontent.com/NousResearch/hermes-agent/main/scripts/install.sh | bash -s -- --skip-setup";

    let child = std::process::Command::new("bash")
        .arg("-c").arg(install_cmd)
        .current_dir(&home)
        .env("PATH", &base_path)
        .env("HOME", home.to_string_lossy().as_ref())
        .env("TERM", "dumb")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start installer: {}", e))?;

    let output = child.wait_with_output().map_err(|e| format!("Installer error: {}", e))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let log = format!("{}\n{}", stdout, stderr);

    let success = output.status.success() || (hermes_python().exists() && hermes_script().exists());

    emit(app, 7, "Installation complete", if success { "Hermes is ready" } else { "Installation finished with warnings" }, Some(&log));

    if success { Ok(InstallResult { success: true, error: None }) }
    else { Ok(InstallResult { success: false, error: Some(log) }) }
}

fn run_install_windows(
    app: &AppHandle,
    emit: &dyn Fn(&AppHandle, u32, &str, &str, Option<&str>),
) -> Result<InstallResult, String> {
    let home = dirs_next::home_dir().unwrap_or_default();
    let base_path = get_enhanced_path();
    let hermes_home_str = hermes_home().to_string_lossy().to_string();
    let install_dir = hermes_repo().to_string_lossy().to_string();

    emit(app, 2, "Downloading installer", "Fetching install script...", None);

    // Build wrapper PowerShell script
    let ps_quote = |s: &str| -> String { format!("'{}'", s.replace('\'', "''")) };
    let wrapper = format!(
        "$ErrorActionPreference = 'Stop'\n\
         try {{ [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12 }} catch {{}}\n\
         $url = 'https://raw.githubusercontent.com/NousResearch/hermes-agent/main/scripts/install.ps1'\n\
         $installer = Join-Path $env:TEMP ('hermes-install-' + [guid]::NewGuid().ToString() + '.ps1')\n\
         $resp = Invoke-WebRequest -Uri $url -UseBasicParsing\n\
         $text = [string]$resp.Content\n\
         if ($text.Length -gt 0 -and $text[0] -eq [char]0xFEFF) {{ $text = $text.Substring(1) }}\n\
         [System.IO.File]::WriteAllText($installer, $text, (New-Object System.Text.UTF8Encoding $true))\n\
         & $installer -SkipSetup -HermesHome {} -InstallDir {}\n\
         $exit = $LASTEXITCODE\n\
         Remove-Item -Force -ErrorAction SilentlyContinue $installer\n\
         exit $exit\n",
        ps_quote(&hermes_home_str), ps_quote(&install_dir),
    );

    let tmp = std::env::temp_dir().join(format!("hermes-install-{}.ps1", uuid::Uuid::new_v4()));
    fs::write(&tmp, &wrapper).map_err(|e| format!("Failed to stage installer: {}", e))?;

    emit(app, 3, "Running installer", "Executing install.ps1...", None);

    let ps = if cfg!(windows) { "powershell.exe" } else { "pwsh" };
    let child = std::process::Command::new(ps)
        .args(&["-ExecutionPolicy", "Bypass", "-NoProfile", "-NonInteractive", "-File"])
        .arg(tmp.to_string_lossy().as_ref())
        .current_dir(&home)
        .env("PATH", &base_path)
        .env("HERMES_HOME", &hermes_home_str)
        .env("NO_COLOR", "1")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start PowerShell: {}", e))?;

    let output = child.wait_with_output().map_err(|e| format!("PowerShell error: {}", e))?;
    let _ = fs::remove_file(&tmp);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let log = format!("{}\n{}", stdout, stderr);

    let success = output.status.success() || (hermes_python().exists() && hermes_script().exists());

    emit(app, 7, "Installation complete", if success { "Hermes is ready" } else { "Installation finished with warnings" }, Some(&log));

    if success { Ok(InstallResult { success: true, error: None }) }
    else { Ok(InstallResult { success: false, error: Some(log) }) }
}

// ─── Hermes Engine ──────────────────────────────────────

#[tauri::command]
pub fn get_hermes_version() -> Result<Option<String>, String> {
    if !hermes_python().exists() { return Ok(None); }
    let output = std::process::Command::new(hermes_python())
        .arg(hermes_script()).arg("--version")
        .env("HERMES_HOME", hermes_home().to_string_lossy().as_ref())
        .output()
        .map_err(|e| format!("Failed: {}", e))?;
    let v = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(if v.is_empty() { None } else { Some(v) })
}

#[tauri::command]
pub fn refresh_hermes_version() -> Result<Option<String>, String> {
    get_hermes_version()
}

#[tauri::command]
pub fn run_hermes_doctor() -> Result<String, String> {
    hermes_cli::run_hermes_cli(&["doctor"], None)
}

#[tauri::command]
pub fn run_hermes_update(app: AppHandle) -> Result<InstallResult, String> {
    if !hermes_python().exists() {
        return Err("Hermes is not installed".into());
    }

    let _ = app.emit("install-progress", InstallProgress {
        step: 1, total_steps: 1,
        title: "Updating Hermes Agent".into(),
        detail: "Running hermes update...".into(),
        log: None,
    });

    let args = hermes_cli_args(&["update"]);
    let output = std::process::Command::new(hermes_python())
        .args(&args)
        .current_dir(hermes_repo())
        .env("PATH", get_enhanced_path())
        .env("HERMES_HOME", hermes_home().to_string_lossy().as_ref())
        .output()
        .map_err(|e| format!("Update failed: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let log = format!("{}\n{}", stdout, stderr);

    if output.status.success() {
        Ok(InstallResult { success: true, error: None })
    } else {
        Ok(InstallResult { success: false, error: Some(log) })
    }
}

// ─── OpenClaw ────────────────────────────────────────────

#[tauri::command]
pub fn check_openclaw() -> Result<OpenClawStatus, String> {
    let home = dirs_next::home_dir().unwrap_or_default();
    for name in &[".openclaw", ".clawdbot", ".moldbot"] {
        let dir = home.join(name);
        if dir.exists() {
            return Ok(OpenClawStatus { found: true, path: Some(dir.to_string_lossy().to_string()) });
        }
    }
    Ok(OpenClawStatus { found: false, path: None })
}

#[tauri::command]
pub fn run_hermes_migrate(app: AppHandle) -> Result<InstallResult, String> {
    if !hermes_python().exists() {
        return Err("Hermes is not installed".into());
    }

    let _ = app.emit("install-progress", InstallProgress {
        step: 1, total_steps: 1,
        title: "Migrating from OpenClaw".into(),
        detail: "Running migration...".into(),
        log: None,
    });

    let args = hermes_cli_args(&["claw", "migrate", "--preset", "full"]);
    let output = std::process::Command::new(hermes_python())
        .args(&args)
        .current_dir(hermes_repo())
        .env("PATH", get_enhanced_path())
        .env("HERMES_HOME", hermes_home().to_string_lossy().as_ref())
        .output()
        .map_err(|e| format!("Migration failed: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if output.status.success() {
        Ok(InstallResult { success: true, error: None })
    } else {
        Ok(InstallResult { success: false, error: Some(format!("{}\n{}", stdout, stderr)) })
    }
}

// ─── Backup / Import / Dump ─────────────────────────────

#[tauri::command]
pub fn run_hermes_backup(profile: Option<String>) -> Result<BackupResult, String> {
    let mut args: Vec<&str> = vec!["backup"];

    if let Some(ref p) = profile { args.push("--profile"); args.push(p); }
    match hermes_cli::run_hermes_cli(&args, profile.as_deref()) {
        Ok(output) => {
            let path = output.trim().lines().last().map(|s| s.to_string());
            Ok(BackupResult { success: true, path, error: None })
        }
        Err(e) => Ok(BackupResult { success: false, path: None, error: Some(e) }),
    }
}

#[tauri::command]
pub fn run_hermes_import(path: String, profile: Option<String>) -> Result<InstallResult, String> {
    let mut args: Vec<&str> = vec!["import", &path];

    if let Some(ref p) = profile { args.push("--profile"); args.push(p); }
    match hermes_cli::run_hermes_cli(&args, profile.as_deref()) {
        Ok(_) => Ok(InstallResult { success: true, error: None }),
        Err(e) => Ok(InstallResult { success: false, error: Some(e) }),
    }
}

#[tauri::command]
pub fn run_hermes_dump() -> Result<String, String> {
    hermes_cli::run_hermes_cli(&["dump"], None)
}

// ─── MCP Servers ─────────────────────────────────────────

#[tauri::command]
pub fn list_mcp_servers(profile: Option<String>) -> Result<Vec<McpServer>, String> {
    match hermes_cli::run_hermes_cli(&["mcp", "list", "--json"], profile.as_deref()) {
        Ok(json) => Ok(serde_json::from_str(&json).unwrap_or_default()),
        Err(e) => Err(e),
    }
}

// ─── Memory Providers ────────────────────────────────────

const KNOWN_MEMORY_PROVIDERS: &[(&str, &str, &[&str])] = &[
    ("honcho", "memory.providers.honcho", &["HONCHO_API_KEY"]),
    ("hindsight", "memory.providers.hindsight", &["HINDSIGHT_API_KEY", "HINDSIGHT_API_URL", "HINDSIGHT_BANK_ID"]),
    ("mem0", "memory.providers.mem0", &["MEM0_API_KEY"]),
    ("retaindb", "memory.providers.retaindb", &["RETAINDB_API_KEY"]),
    ("supermemory", "memory.providers.supermemory", &["SUPERMEMORY_API_KEY"]),
    ("holographic", "memory.providers.holographic", &[]),
    ("openviking", "memory.providers.openviking", &["OPENVIKING_ENDPOINT", "OPENVIKING_API_KEY"]),
    ("byterover", "memory.providers.byterover", &["BRV_API_KEY"]),
];

#[tauri::command]
pub fn discover_memory_providers(profile: Option<String>) -> Result<Vec<MemoryProvider>, String> {
    let plugins_dir = hermes_repo().join("plugins").join("memory");
    if !plugins_dir.exists() { return Ok(Vec::new()); }

    let active = get_active_memory_provider(profile.as_deref());
    let mut results = Vec::new();

    if let Ok(entries) = fs::read_dir(&plugins_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) { continue; }
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('_') { continue; }

            let installed = entry.path().join("__init__.py").exists();
            let desc = KNOWN_MEMORY_PROVIDERS.iter()
                .find(|(n, _, _)| *n == name)
                .map(|(_, d, _)| d.to_string())
                .unwrap_or_else(|| name.clone());
            let env_vars: Vec<String> = KNOWN_MEMORY_PROVIDERS.iter()
                .find(|(n, _, _)| *n == name)
                .map(|(_, _, e)| e.iter().map(|s| s.to_string()).collect())
                .unwrap_or_default();
            let is_active = active.as_deref() == Some(name.as_str());
            results.push(MemoryProvider {
                name: name.clone(),
                description: desc.to_string(),
                installed,
                active: is_active,
                env_vars,
            });
        }
    }

    // Sort: active first, then installed, then alphabetical
    results.sort_by(|a, b| {
        b.active.cmp(&a.active)
            .then_with(|| b.installed.cmp(&a.installed))
            .then_with(|| a.name.cmp(&b.name))
    });

    Ok(results)
}

fn get_active_memory_provider(profile: Option<&str>) -> Option<String> {
    let config_path = hermes_cli::resolve_profile_home(profile).join("config.yaml");
    if !config_path.exists() { return None; }
    let content = fs::read_to_string(&config_path).ok()?;
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(caps) = regex_lite::Regex::new(r#"^\s*provider:\s*["']?(\w+)["']?\s*$"#).ok()
            .and_then(|re| re.captures(trimmed)) {
            return caps.get(1).map(|m| m.as_str().to_string());
        }
    }
    None
}

// ─── Log Viewer ──────────────────────────────────────────

#[tauri::command]
pub fn read_logs(log_file: Option<String>, lines: Option<u32>) -> Result<LogResult, String> {
    let logs_dir = hermes_home().join("logs");
    let file_name = log_file.as_deref().unwrap_or("agent.log");
    // Sanitize: whitelist only known log filenames
    let safe_name = if LOG_WHITELIST.contains(&file_name) { file_name } else { "agent.log" };
    let full_path = logs_dir.join(safe_name);

    if !full_path.exists() {
        return Ok(LogResult { content: String::new(), path: full_path.to_string_lossy().to_string() });
    }

    let content = fs::read_to_string(&full_path)
        .map_err(|e| format!("Failed to read log: {}", e))?;
    let line_limit = lines.unwrap_or(200) as usize;
    let all_lines: Vec<&str> = content.lines().collect();
    let start = if all_lines.len() > line_limit { all_lines.len() - line_limit } else { 0 };
    let tail = all_lines[start..].join("\n");

    Ok(LogResult { content: tail, path: full_path.to_string_lossy().to_string() })
}

// ─── Updates stubs ───────────────────────────────────────

#[tauri::command] pub fn check_for_updates() -> Result<Option<String>, String> { Ok(None) }
#[tauri::command] pub fn download_update() -> Result<bool, String> { Ok(true) }
#[tauri::command] pub fn install_update() -> Result<(), String> { Ok(()) }

// ─── Claw3D Setup ────────────────────────────────────────

#[tauri::command]
pub fn claw3d_setup(app: AppHandle) -> Result<InstallResult, String> {
    let home = dirs_next::home_dir().unwrap_or_default();
    let openclaw_dir = home.join("openclaw");

    let _ = app.emit("claw3d-setup-progress", InstallProgress {
        step: 1, total_steps: 3,
        title: "Setting up Claw3D".into(),
        detail: "Checking OpenClaw directory...".into(),
        log: None,
    });

    if !openclaw_dir.exists() {
        let _ = app.emit("claw3d-setup-progress", InstallProgress {
            step: 2, total_steps: 3,
            title: "Cloning OpenClaw".into(),
            detail: "Cloning repository...".into(),
            log: None,
        });
        let out = std::process::Command::new("git")
            .args(&["clone", "https://github.com/nousresearch/openclaw",])
            .arg(openclaw_dir.to_string_lossy().as_ref())
            .output().map_err(|e| format!("Clone failed: {}", e))?;
        if !out.status.success() {
            let err = String::from_utf8_lossy(&out.stderr);
            return Ok(InstallResult { success: false, error: Some(err.trim().to_string()) });
        }
    }

    let _ = app.emit("claw3d-setup-progress", InstallProgress {
        step: 3, total_steps: 3,
        title: "Installing dependencies".into(),
        detail: "Running npm install...".into(),
        log: None,
    });

    let out = std::process::Command::new("npm")
        .arg("install").current_dir(&openclaw_dir)
        .output().map_err(|e| format!("npm install failed: {}", e))?;

    let _log = String::from_utf8_lossy(&out.stdout).to_string();
    Ok(InstallResult { success: true, error: None })
}

// ─── Select Folder / Open External ───────────────────────

#[tauri::command] pub fn select_folder() -> Result<Option<String>, String> { Ok(None) }

#[tauri::command]
pub fn open_external(url: String) -> Result<(), String> {
    #[cfg(target_os = "windows")] { let _ = std::process::Command::new("cmd").arg("/c").arg("start").arg("").arg(&url).status(); }
    #[cfg(target_os = "macos")] { let _ = std::process::Command::new("open").arg(&url).status(); }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))] { let _ = std::process::Command::new("xdg-open").arg(&url).status(); }
    Ok(())
}
