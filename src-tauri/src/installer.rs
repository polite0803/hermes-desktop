// Installation pipeline — async, streaming progress events
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tauri::{AppHandle, Emitter};

use crate::hermes_cli;

// ─── Constants ───

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

// ─── Data Types ───

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InstallStatus { pub installed: bool, pub configured: bool, pub has_api_key: bool }

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InstallProgress { pub step: u32, pub total_steps: u32, pub title: String, pub detail: String, pub log: Option<String> }

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InstallResult { pub success: bool, pub error: Option<String> }

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OpenClawStatus { pub found: bool, pub path: Option<String> }

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BackupResult { pub success: bool, pub path: Option<String>, pub error: Option<String> }

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct McpServer { pub name: String, #[serde(rename = "type")] pub server_type: String, pub enabled: bool, pub detail: String }

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MemoryProvider { pub name: String, pub description: String, pub installed: bool, pub active: bool, #[serde(rename = "envVars")] pub env_vars: Vec<String> }

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LogResult { pub content: String, pub path: String }

// ─── Provider Env Key Mapping ───

const PROVIDER_ENV_KEYS: &[(&str, &str)] = &[
    ("openrouter", "OPENROUTER_API_KEY"), ("anthropic", "ANTHROPIC_API_KEY"),
    ("openai", "OPENAI_API_KEY"), ("google", "GOOGLE_API_KEY"),
    ("xai", "XAI_API_KEY"), ("groq", "GROQ_API_KEY"),
    ("deepseek", "DEEPSEEK_API_KEY"), ("together", "TOGETHER_API_KEY"),
    ("fireworks", "FIREWORKS_API_KEY"), ("cerebras", "CEREBRAS_API_KEY"),
    ("mistral", "MISTRAL_API_KEY"), ("perplexity", "PERPLEXITY_API_KEY"),
    ("huggingface", "HF_TOKEN"), ("hf", "HF_TOKEN"),
    ("qwen", "QWEN_API_KEY"), ("minimax", "MINIMAX_API_KEY"),
    ("glm", "GLM_API_KEY"), ("kimi", "KIMI_API_KEY"), ("nvidia", "NVIDIA_API_KEY"),
];

const URL_TO_ENV_KEY: &[(&str, &str)] = &[
    ("openrouter.ai", "OPENROUTER_API_KEY"), ("anthropic.com", "ANTHROPIC_API_KEY"),
    ("openai.com", "OPENAI_API_KEY"), ("huggingface.co", "HF_TOKEN"),
    ("api.groq.com", "GROQ_API_KEY"), ("api.deepseek.com", "DEEPSEEK_API_KEY"),
    ("api.together.xyz", "TOGETHER_API_KEY"), ("api.fireworks.ai", "FIREWORKS_API_KEY"),
    ("api.cerebras.ai", "CEREBRAS_API_KEY"), ("api.mistral.ai", "MISTRAL_API_KEY"),
    ("api.perplexity.ai", "PERPLEXITY_API_KEY"),
];

fn env_has_usable_value(content: &str, expected_key: Option<&str>) -> bool {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') { continue; }
        if let Some((k, v)) = trimmed.split_once('=') {
            let key = k.trim(); let val = v.trim().trim_matches('"').trim_matches('\'');
            if val.is_empty() { continue; }
            if let Some(expected) = expected_key { if key == expected { return true; } }
            else if key.ends_with("_API_KEY") { return true; }
        }
    }
    false
}

fn get_enhanced_path() -> String {
    let home = dirs_next::home_dir().unwrap_or_default();
    let sep = if cfg!(windows) { ";" } else { ":" };
    let extras: Vec<PathBuf> = if cfg!(windows) {
        let mut v = vec![hermes_home().join("git").join("bin"), hermes_home().join("git").join("cmd"),
            hermes_home().join("node"), hermes_venv().join("Scripts")];
        if let Some(ad) = std::env::var_os("APPDATA") { v.push(PathBuf::from(&ad).join("npm")); }
        if let Some(pf) = std::env::var_os("ProgramFiles") { v.push(PathBuf::from(&pf).join("nodejs")); v.push(PathBuf::from(&pf).join("Git").join("cmd")); }
        if let Some(lc) = std::env::var_os("LOCALAPPDATA") { v.push(PathBuf::from(&lc).join("Programs").join("Git").join("cmd")); }
        v.push(home.join(".local").join("bin")); v.push(home.join(".cargo").join("bin"));
        v
    } else {
        vec![home.join(".local").join("bin"), home.join(".cargo").join("bin"), hermes_venv().join("bin"),
            home.join(".volta").join("bin"), home.join(".asdf").join("shims"),
            home.join(".fnm").join("aliases").join("default").join("bin"),
            PathBuf::from("/usr/local/bin"), PathBuf::from("/opt/homebrew/bin"), PathBuf::from("/opt/homebrew/sbin")]
    };
    let extra_str: Vec<String> = extras.iter().filter(|p| p.exists()).map(|p| p.to_string_lossy().to_string()).collect();
    let existing = std::env::var("PATH").unwrap_or_default();
    [extra_str.join(sep), existing].into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join(sep)
}

// ─── Async streaming spawn helper ───

fn spawn_and_stream<F>(
    app: AppHandle, cmd: &mut Command, step: u32, total: u32,
    title: String, on_done: F,
) where F: FnOnce(bool, String) + Send + 'static {
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    #[cfg(windows)] { use std::os::windows::process::CommandExt; cmd.creation_flags(0x08000000); }

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            let _ = app.emit("install-progress", InstallProgress { step, total_steps: total, title: title.clone(), detail: format!("Failed: {}", e), log: None });
            on_done(false, format!("{}", e));
            return;
        }
    };

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    std::thread::spawn(move || {
        let mut log = String::new();
        let t = title;

        if let Some(out) = stdout {
            for line in BufReader::new(out).lines().map_while(Result::ok) {
                let stripped = strip_ansi(&line);
                log.push_str(&stripped); log.push('\n');
                let detail: String = stripped.chars().take(120).collect();
                let _ = app.emit("install-progress", InstallProgress { step, total_steps: total, title: t.clone(), detail, log: Some(log.clone()) });
            }
        }
        if let Some(err) = stderr {
            for line in BufReader::new(err).lines().map_while(Result::ok) {
                let stripped = strip_ansi(&line);
                log.push_str(&stripped); log.push('\n');
                let detail: String = stripped.chars().take(120).collect();
                let _ = app.emit("install-progress", InstallProgress { step, total_steps: total, title: t.clone(), detail, log: Some(log.clone()) });
            }
        }

        let status = child.wait();
        let ok = status.map(|s| s.success()).unwrap_or(false) || (hermes_python().exists() && hermes_script().exists());
        on_done(ok, log);
    });
}

fn strip_ansi(s: &str) -> String {
    regex_lite::Regex::new(r"\x1b\[[0-9;]*m").ok().map(|re| re.replace_all(s, "").to_string()).unwrap_or_else(|| s.to_string())
}

// ─── Status ───

#[tauri::command]
pub fn check_install() -> Result<InstallStatus, String> {
    let installed = hermes_python().exists() && hermes_script().exists();
    let env = hermes_home().join(".env");
    let auth = hermes_home().join("auth.json");
    let configured = env.exists() || auth.exists();
    let mut has_api_key = false;
    if configured && env.exists() {
        if let Ok(c) = fs::read_to_string(&env) { has_api_key = env_has_usable_value(&c, None); }
    }
    Ok(InstallStatus { installed, configured, has_api_key })
}

#[tauri::command]
pub async fn start_pypi_install(app: AppHandle) -> Result<InstallResult, String> {
    let total = 4u32;
    let _ = app.emit("install-progress", InstallProgress { step: 1, total_steps: total, title: "PyPI Install".into(), detail: "Creating virtual environment...".into(), log: None });

    let hermes_home = hermes_home();
    let venv = hermes_venv();

    // Create venv
    let python = if cfg!(windows) { "python" } else { "python3" };
    let venv_out = std::process::Command::new(python).args(&["-m", "venv", &venv.to_string_lossy()]).output().map_err(|e| format!("{}", e))?;
    if !venv_out.status.success() && !venv.join(if cfg!(windows) { "Scripts/python.exe" } else { "bin/python3" }).exists() {
        return Ok(InstallResult { success: false, error: Some(String::from_utf8_lossy(&venv_out.stderr).to_string()) });
    }

    // Install hermes-agent via pip
    let _ = app.emit("install-progress", InstallProgress { step: 2, total_steps: total, title: "Installing hermes-agent".into(), detail: "pip install hermes-agent...".into(), log: None });
    let pip = if cfg!(windows) { venv.join("Scripts/pip.exe") } else { venv.join("bin/pip") };
    let (tx, rx) = std::sync::mpsc::channel();
    let app2 = app.clone();
    let mut pip_cmd = std::process::Command::new(&pip);
    pip_cmd.args(&["install", "hermes-agent"]).env("PIP_REQUIRE_VIRTUALENV", "false");
    spawn_and_stream(app2, &mut pip_cmd, 2, total, "Installing hermes-agent".into(), move |ok, log| { let _ = tx.send((ok, log)); });
    let (ok, log) = rx.recv().map_err(|e| format!("{}", e))?;

    let _ = app.emit("install-progress", InstallProgress { step: 3, total_steps: total, title: "Finalizing".into(), detail: "Writing config...".into(), log: None });
    let config = hermes_home.join("config.yaml");
    if !config.exists() {
        let _ = fs::write(&config, "model:\n  provider: auto\n");
    }

    let _ = app.emit("install-progress", InstallProgress { step: 4, total_steps: total, title: "Installation complete".into(), detail: if ok { "hermes-agent installed via PyPI" } else { "Finished with warnings" }.into(), log: Some(log.clone()) });
    Ok(InstallResult { success: ok, error: if ok { None } else { Some(log) } })
}

#[tauri::command]
pub fn verify_install() -> Result<bool, String> {
    // Try the hermess CLI directly — this uses resolve_python/script which handle multiple install locations
    match hermes_cli::run_hermes_cli(&["--version"], None) {
        Ok(v) => Ok(!v.is_empty()),
        Err(_) => {
            // Fallback: check file existence via installer paths
            Ok(hermes_python().exists() && hermes_script().exists())
        }
    }
}

// ─── Async Install ───

#[tauri::command]
pub async fn start_install(app: AppHandle) -> Result<InstallResult, String> {
    let total = 7u32;
    let _ = app.emit("install-progress", InstallProgress { step: 1, total_steps: total, title: "Starting installation".into(), detail: "Preparing...".into(), log: None });

    if cfg!(windows) {
        run_install_windows_async(app, total).await
    } else {
        run_install_unix_async(app, total).await
    }
}

async fn run_install_unix_async(app: AppHandle, total: u32) -> Result<InstallResult, String> {
    let home = dirs_next::home_dir().unwrap_or_default();
    let base_path = get_enhanced_path();

    let _ = app.emit("install-progress", InstallProgress { step: 2, total_steps: total, title: "Checking prerequisites".into(), detail: "Running installer...".into(), log: None });

    let (tx, rx) = std::sync::mpsc::channel();
    let home_path = home.to_string_lossy().to_string();
    let bp = base_path.clone();
    let app2 = app.clone();
    let mut cmd = Command::new("bash");
    cmd.arg("-c").arg("curl -fsSL https://raw.githubusercontent.com/NousResearch/hermes-agent/main/scripts/install.sh | bash -s -- --skip-setup")
        .current_dir(&home_path).env("PATH", &bp).env("HOME", &home_path).env("TERM", "dumb");
    spawn_and_stream(app2, &mut cmd, 2, total, "Installing Hermes Agent".into(),
        move |ok, log| { let _ = tx.send((ok, log)); }
    );

    let (ok, log) = rx.recv().map_err(|e| format!("{}", e))?;
    let _ = app.emit("install-progress", InstallProgress { step: 7, total_steps: total, title: "Installation complete".into(), detail: if ok { "Hermes is ready" } else { "Finished with warnings" }.into(), log: Some(log.clone()) });
    Ok(InstallResult { success: ok, error: if ok { None } else { Some(log) } })
}

async fn run_install_windows_async(app: AppHandle, total: u32) -> Result<InstallResult, String> {
    let home = dirs_next::home_dir().unwrap_or_default();
    let base_path = get_enhanced_path();
    let hermes_home_str = hermes_home().to_string_lossy().to_string();
    let install_dir = hermes_repo().to_string_lossy().to_string();
    let home_path = home.to_string_lossy().to_string();

    let _ = app.emit("install-progress", InstallProgress { step: 2, total_steps: total, title: "Downloading installer".into(), detail: "Fetching install script...".into(), log: None });

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

    let _ = app.emit("install-progress", InstallProgress { step: 3, total_steps: total, title: "Running installer".into(), detail: "Executing install.ps1...".into(), log: None });

    let ps = if cfg!(windows) { "powershell.exe" } else { "pwsh" };
    let tmp_path = tmp.to_string_lossy().to_string();
    let bp2 = base_path.clone();
    let hh2 = hermes_home_str.clone();

    let (tx, rx) = std::sync::mpsc::channel();
    let app2 = app.clone();
    let tmp2 = tmp;
    let mut ps_cmd = Command::new(ps);
    ps_cmd.args(&["-ExecutionPolicy","Bypass","-NoProfile","-NonInteractive","-File", &tmp_path])
        .current_dir(&home_path).env("PATH", &bp2).env("HERMES_HOME", &hh2).env("NO_COLOR", "1");
    spawn_and_stream(app2, &mut ps_cmd, 3, total, "Installing Hermes Agent".into(),
        move |ok, log| { let _ = fs::remove_file(&tmp2); let _ = tx.send((ok, log)); }
    );

    let (ok, log) = rx.recv().map_err(|e| format!("{}", e))?;
    let _ = app.emit("install-progress", InstallProgress { step: 7, total_steps: total, title: "Installation complete".into(), detail: if ok { "Hermes is ready" } else { "Finished with warnings" }.into(), log: Some(log.clone()) });
    Ok(InstallResult { success: ok, error: if ok { None } else { Some(log) } })
}

// ─── Hermes Engine ───

#[tauri::command]
pub fn get_hermes_version() -> Result<Option<String>, String> {
    if !hermes_python().exists() { return Ok(None); }
    Command::new(hermes_python()).arg(hermes_script()).arg("--version")
        .env("HERMES_HOME", hermes_home().to_string_lossy().as_ref())
        .output().ok().map(|o| {
            let v = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if v.is_empty() { None } else { Some(v) }
        }).ok_or("Failed".into())
}

#[tauri::command] pub fn refresh_hermes_version() -> Result<Option<String>, String> { get_hermes_version() }

#[tauri::command] pub fn run_hermes_doctor() -> Result<String, String> { hermes_cli::run_hermes_cli(&["doctor"], None) }

#[tauri::command]
pub async fn run_hermes_update(app: AppHandle) -> Result<InstallResult, String> {
    if !hermes_python().exists() { return Err("Hermes is not installed".into()); }
    let _ = app.emit("install-progress", InstallProgress { step: 1, total_steps: 1, title: "Updating Hermes Agent".into(), detail: "Running hermes update...".into(), log: None });

    let (tx, rx) = std::sync::mpsc::channel();
    let app2 = app.clone();
    let repo = hermes_repo().to_string_lossy().to_string();
    let hh = hermes_home().to_string_lossy().to_string();
    let bp = get_enhanced_path();
    let mut update_cmd = Command::new(hermes_python());
    update_cmd.arg(hermes_script()).args(&["update"]).current_dir(&repo).env("PATH", &bp).env("HERMES_HOME", &hh);
    spawn_and_stream(app2, &mut update_cmd, 1, 1, "Updating Hermes Agent".into(),
        move |ok, log| { let _ = tx.send((ok, log)); }
    );

    let (ok, log) = rx.recv().map_err(|e| format!("{}", e))?;
    Ok(InstallResult { success: ok, error: if ok { None } else { Some(log) } })
}

// ─── OpenClaw ───

#[tauri::command]
pub fn check_openclaw() -> Result<OpenClawStatus, String> {
    let home = dirs_next::home_dir().unwrap_or_default();
    for name in &[".claw3d_dir", ".clawdbot", ".moldbot"] {
        let dir = home.join(name);
        if dir.exists() { return Ok(OpenClawStatus { found: true, path: Some(dir.to_string_lossy().to_string()) }); }
    }
    Ok(OpenClawStatus { found: false, path: None })
}

#[tauri::command]
pub async fn run_hermes_migrate(app: AppHandle) -> Result<InstallResult, String> {
    if !hermes_python().exists() { return Err("Hermes is not installed".into()); }
    let _ = app.emit("install-progress", InstallProgress { step: 1, total_steps: 1, title: "Migrating from OpenClaw".into(), detail: "Running migration...".into(), log: None });

    let (tx, rx) = std::sync::mpsc::channel();
    let app2 = app.clone();
    let repo = hermes_repo().to_string_lossy().to_string();
    let hh = hermes_home().to_string_lossy().to_string();
    let bp = get_enhanced_path();
    let mut migrate_cmd = Command::new(hermes_python());
    migrate_cmd.arg(hermes_script()).args(&["claw","migrate","--preset","full"]).current_dir(&repo).env("PATH", &bp).env("HERMES_HOME", &hh);
    spawn_and_stream(app2, &mut migrate_cmd, 1, 1, "Migrating from OpenClaw".into(),
        move |ok, log| { let _ = tx.send((ok, log)); }
    );

    let (ok, log) = rx.recv().map_err(|e| format!("{}", e))?;
    Ok(InstallResult { success: ok, error: if ok { None } else { Some(log) } })
}

// ─── Backup / Import / Dump ───

#[tauri::command] pub fn run_hermes_backup(profile: Option<String>) -> Result<BackupResult, String> {
    let mut args: Vec<&str> = vec!["backup"];
    if let Some(ref p) = profile { args.push("--profile"); args.push(p); }
    match hermes_cli::run_hermes_cli(&args, profile.as_deref()) {
        Ok(o) => Ok(BackupResult { success: true, path: o.trim().lines().last().map(|s| s.to_string()), error: None }),
        Err(e) => Ok(BackupResult { success: false, path: None, error: Some(e) }),
    }
}

#[tauri::command] pub fn run_hermes_import(path: String, profile: Option<String>) -> Result<InstallResult, String> {
    let mut args: Vec<&str> = vec!["import", &path];
    if let Some(ref p) = profile { args.push("--profile"); args.push(p); }
    match hermes_cli::run_hermes_cli(&args, profile.as_deref()) { Ok(_) => Ok(InstallResult { success: true, error: None }), Err(e) => Ok(InstallResult { success: false, error: Some(e) }) }
}

#[tauri::command] pub fn run_hermes_dump() -> Result<String, String> { hermes_cli::run_hermes_cli(&["dump"], None) }

// ─── MCP / Memory Providers / Logs ───

const KNOWN_MEMORY_PROVIDERS: &[(&str, &str, &[&str])] = &[
    ("honcho", "Honcho", &["HONCHO_API_KEY"]), ("hindsight", "Hindsight", &["HINDSIGHT_API_KEY","HINDSIGHT_API_URL","HINDSIGHT_BANK_ID"]),
    ("mem0", "Mem0", &["MEM0_API_KEY"]), ("retaindb", "RetainDB", &["RETAINDB_API_KEY"]),
    ("supermemory", "SuperMemory", &["SUPERMEMORY_API_KEY"]), ("holographic", "Holographic", &[]),
    ("openviking", "OpenViking", &["OPENVIKING_ENDPOINT","OPENVIKING_API_KEY"]), ("byterover", "ByteRover", &["BRV_API_KEY"]),
];

#[tauri::command] pub fn discover_memory_providers(profile: Option<String>) -> Result<Vec<MemoryProvider>, String> {
    let plugins = hermes_repo().join("plugins").join("memory");
    if !plugins.exists() { return Ok(Vec::new()); }
    let active = hermes_cli::resolve_profile_home(profile.as_deref()).join("config.yaml").exists()
        .then(|| fs::read_to_string(hermes_cli::resolve_profile_home(profile.as_deref()).join("config.yaml")).ok())
        .flatten().and_then(|c| c.lines().find(|l| l.trim().starts_with("provider:")).and_then(|l| l.split_once(':').map(|(_,v)| v.trim().trim_matches('"').trim_matches('\'').to_string())));
    let mut results = Vec::new();
    if let Ok(entries) = fs::read_dir(&plugins) {
        for e in entries.filter_map(|e| e.ok()) {
            if !e.file_type().map(|t| t.is_dir()).unwrap_or(false) { continue; }
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with('_') { continue; }
            let installed = e.path().join("__init__.py").exists();
            let (desc, env_vars) = KNOWN_MEMORY_PROVIDERS.iter().find(|(n,_,_)| *n == name).map(|(_,d,e)| (d.to_string(), e.iter().map(|s| s.to_string()).collect())).unwrap_or_default();
            results.push(MemoryProvider { name: name.clone(), description: desc, installed, active: active.as_deref() == Some(&name), env_vars });
        }
    }
    results.sort_by(|a,b| b.active.cmp(&a.active).then_with(|| b.installed.cmp(&a.installed)).then_with(|| a.name.cmp(&b.name)));
    Ok(results)
}

#[tauri::command] pub fn read_logs(log_file: Option<String>, lines: Option<u32>) -> Result<LogResult, String> {
    let logs = hermes_home().join("logs");
    let safe = log_file.as_deref().filter(|f| LOG_WHITELIST.contains(f)).unwrap_or("agent.log");
    let path = logs.join(safe);
    if !path.exists() { return Ok(LogResult { content: String::new(), path: path.to_string_lossy().to_string() }); }
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let lim = lines.unwrap_or(200) as usize;
    let all: Vec<&str> = content.lines().collect();
    let tail = if all.len() > lim { all[all.len()-lim..].join("\n") } else { content };
    Ok(LogResult { content: tail, path: path.to_string_lossy().to_string() })
}

// ─── Updates ───

#[tauri::command] pub fn check_for_updates() -> Result<Option<String>, String> { Ok(None) }
#[tauri::command] pub fn download_update() -> Result<bool, String> { Ok(true) }
#[tauri::command] pub fn install_update() -> Result<(), String> { Ok(()) }

// ─── Claw3D Setup ───

#[tauri::command]
pub async fn claw3d_setup(app: AppHandle) -> Result<InstallResult, String> {
    let claw3d_dir = hermes_home().join("claw3d");

    let _ = app.emit("claw3d-setup-progress", InstallProgress { step: 1, total_steps: 3, title: "Setting up Claw3D".into(), detail: "Checking directory...".into(), log: None });

    if !claw3d_dir.exists() {
        let _ = app.emit("claw3d-setup-progress", InstallProgress { step: 2, total_steps: 3, title: "Cloning Claw3D".into(), detail: "Cloning repository...".into(), log: None });
        let out = Command::new("git").args(&["clone","https://github.com/iamlukethedev/Claw3D"]).arg(&claw3d_dir)
            .env("PATH", get_enhanced_path()).output().map_err(|e| format!("git not found: {}", e))?;
        if !out.status.success() {
            let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
            return Ok(InstallResult { success: false, error: Some(err) });
        }
    }

    let (tx, rx) = std::sync::mpsc::channel();
    let app2 = app.clone();
    let dir = claw3d_dir.to_string_lossy().to_string();
    let mut npm_cmd = Command::new("npm");
    npm_cmd.arg("install").current_dir(&dir).env("PATH", get_enhanced_path());
    spawn_and_stream(app2, &mut npm_cmd, 3, 3, "Installing dependencies".into(),
        move |ok, log| { let _ = tx.send((ok, log)); }
    );

    let (ok, log) = rx.recv().map_err(|e| format!("{}", e))?;
    Ok(InstallResult { success: ok, error: if ok { None } else { Some(log) } })
}

#[tauri::command] pub fn select_folder() -> Result<Option<String>, String> { Ok(None) }

#[tauri::command]
pub fn open_external(url: String) -> Result<(), String> {
    #[cfg(target_os="windows")] { let _ = Command::new("cmd").arg("/c").arg("start").arg("").arg(&url).status(); }
    #[cfg(target_os="macos")] { let _ = Command::new("open").arg(&url).status(); }
    #[cfg(not(any(target_os="windows",target_os="macos")))] { let _ = Command::new("xdg-open").arg(&url).status(); }
    Ok(())
}
