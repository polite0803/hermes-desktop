use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::Mutex;
use std::sync::LazyLock;

/// Supported application locales
pub const APP_LOCALES: &[&str] = &[
    "en", "es", "id", "ja", "pt-BR", "pt-PT", "zh-CN", "zh-TW",
];

pub static CURRENT_LOCALE: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new("en".to_string()));

#[derive(Debug, Serialize, Deserialize)]
struct LocaleConfig {
    locale: Option<String>,
}

/// Get the cached locale, falling back to desktop.json config
#[tauri::command]
pub fn get_locale() -> String {
    if let Ok(locale) = CURRENT_LOCALE.lock() {
        if !locale.is_empty() {
            return locale.clone();
        }
    }

    // Fall back to reading from desktop.json
    let desktop_path = crate::hermes_cli::resolve_hermes_home().join("desktop.json");
    if let Ok(desktop_str) = fs::read_to_string(desktop_path) {
        if let Ok(locale_cfg) = serde_json::from_str::<LocaleConfig>(&desktop_str) {
            if let Some(l) = locale_cfg.locale {
                if APP_LOCALES.contains(&l.as_str()) {
                    if let Ok(mut current) = CURRENT_LOCALE.lock() {
                        *current = l.clone();
                    }
                    return l;
                }
            }
        }
    }

    "en".to_string()
}

/// Set the locale and persist to desktop.json
#[tauri::command]
pub fn set_locale(locale: String) -> Result<(), String> {
    if !APP_LOCALES.contains(&locale.as_str()) {
        return Err(format!("Unsupported locale: {}", locale));
    }

    // Update in-memory cache
    if let Ok(mut current) = CURRENT_LOCALE.lock() {
        *current = locale.clone();
    }

    // Persist to desktop.json
    let desktop_path = crate::hermes_cli::resolve_hermes_home().join("desktop.json");
    let mut config = if desktop_path.exists() {
        let content = fs::read_to_string(&desktop_path)
            .map_err(|e| format!("Failed to read: {}", e))?;
        serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };
    config["locale"] = serde_json::json!(locale);
    let content = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Serialize error: {}", e))?;
    fs::write(&desktop_path, content).map_err(|e| format!("Failed to write: {}", e))
}
