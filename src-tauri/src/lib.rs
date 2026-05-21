#![allow(dead_code)]

mod hermes_cli;
mod config;
mod locale;
mod security;
mod installer;
mod hermes;
mod sessions;
mod session_cache;
mod profiles;
mod memory;
mod soul;
mod models;
mod model_discovery;
mod providers;
mod tools;
mod skills;
mod cronjobs;
mod kanban;
mod claw3d;
mod ssh;
mod attachment_staging;
mod yaml_path;
mod sse_parser;

use tauri::Emitter;
use tauri::menu::{Menu, Submenu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::Manager;

pub struct AppState {
    pub chat_state: std::sync::Mutex<hermes::ChatState>,
    pub gateway_state: std::sync::Mutex<hermes::GatewayState>,
    pub ssh_tunnel_state: std::sync::Mutex<ssh::SshTunnelState>,
}

#[tauri::command]
fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
fn get_system_info() -> serde_json::Value {
    serde_json::json!({
        "platform": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "appVersion": env!("CARGO_PKG_VERSION"),
        "tauriVersion": "2",
    })
}

fn build_menu(app: &tauri::AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let new_chat = MenuItem::with_id(app, "new_chat", "New Chat", true, Some("CmdOrCtrl+N"))?;
    let search = MenuItem::with_id(app, "search_sessions", "Search Sessions", true, Some("CmdOrCtrl+K"))?;
    let chat_menu = Submenu::with_items(app, "Chat", true, &[&new_chat, &search])?;
    let edit_menu = Submenu::with_items(app, "Edit", true, &[
        &PredefinedMenuItem::undo(app, None)?, &PredefinedMenuItem::redo(app, None)?,
        &PredefinedMenuItem::separator(app)?,
        &PredefinedMenuItem::cut(app, None)?, &PredefinedMenuItem::copy(app, None)?,
        &PredefinedMenuItem::paste(app, None)?, &PredefinedMenuItem::select_all(app, None)?,
    ])?;
    let view_menu = Submenu::with_items(app, "View", true, &[
        &PredefinedMenuItem::fullscreen(app, None)?,
    ])?;
    let window_menu = Submenu::with_items(app, "Window", true, &[
        &PredefinedMenuItem::minimize(app, None)?, &PredefinedMenuItem::close_window(app, None)?,
    ])?;
    let help_menu = Submenu::with_items(app, "Help", true, &[
        &MenuItem::with_id(app, "about", "About Hermes Agent", true, None::<&str>)?,
    ])?;
    let menu = Menu::with_items(app, &[&chat_menu, &edit_menu, &view_menu, &window_menu, &help_menu])?;
    Ok(menu)
}

pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            let menu = build_menu(app.handle())?;
            app.set_menu(menu)?;
            app.on_menu_event(move |app_handle, event| {
                match event.id().as_ref() {
                    "new_chat" => { let _ = app_handle.emit("menu-new-chat", ()); }
                    "search_sessions" => { let _ = app_handle.emit("menu-search-sessions", ()); }
                    _ => {}
                }
            });

            // System tray (non-fatal if it fails)
            let icon = app.default_window_icon().cloned();
            let mut builder = TrayIconBuilder::new().tooltip("Hermes Agent");
            if let Some(ic) = icon { builder = builder.icon(ic); }
            let _tray = builder
                .on_menu_event(|app, event| {
                    if event.id().as_ref() == "quit" { app.exit(0); }
                    if event.id().as_ref() == "show" {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                })
                .menu(&tauri::menu::MenuBuilder::new(app)
                    .item(&MenuItem::with_id(app, "show", "Show Window", true, None::<&str>)?)
                    .separator()
                    .item(&MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?)
                    .build()?)
                .build(app)
                .ok();

            Ok(())
        })
        .manage(AppState {
            chat_state: std::sync::Mutex::new(hermes::ChatState::default()),
            gateway_state: std::sync::Mutex::new(hermes::GatewayState::NotRunning),
            ssh_tunnel_state: std::sync::Mutex::new(ssh::SshTunnelState::Disconnected),
        })
        .invoke_handler(tauri::generate_handler![
            // ── Installer ──
            installer::check_install,
            installer::verify_install,
            installer::start_install,
            installer::get_hermes_version,
            installer::refresh_hermes_version,
            installer::run_hermes_doctor,
            installer::run_hermes_update,
            installer::check_openclaw,
            installer::run_hermes_migrate,
            installer::run_hermes_backup,
            installer::run_hermes_import,
            installer::run_hermes_dump,
            installer::list_mcp_servers,
            installer::discover_memory_providers,
            installer::read_logs,
            installer::check_for_updates,
            installer::download_update,
            installer::install_update,
            installer::claw3d_setup,
            installer::select_folder,
            installer::open_external,
            // ── Config ──
            config::get_config_value,
            config::set_config_value,
            config::get_env_all,
            config::set_env,
            config::get_hermes_home,
            config::get_model_config,
            config::set_model_config,
            config::get_connection_config,
            config::set_connection_config,
            config::is_remote_mode,
            config::is_remote_only_mode,
            config::get_platform_enabled_all,
            config::set_platform_enabled,
            config::get_credential_pool,
            config::set_credential_pool,
            // ── Locale ──
            locale::get_locale,
            locale::set_locale,
            // ── Hermes / Chat ──
            hermes::send_message,
            hermes::abort_chat,
            hermes::start_gateway,
            hermes::stop_gateway,
            hermes::gateway_status,
            hermes::test_remote_connection,
            // ── Sessions ──
            sessions::list_sessions,
            sessions::get_session_messages,
            sessions::delete_session,
            sessions::search_sessions,
            session_cache::list_cached_sessions,
            session_cache::sync_session_cache,
            session_cache::update_session_title,
            // ── Profiles ──
            profiles::list_profiles,
            profiles::create_profile,
            profiles::delete_profile,
            profiles::set_active_profile,
            // ── Memory ──
            memory::read_memory,
            memory::add_memory_entry,
            memory::update_memory_entry,
            memory::remove_memory_entry,
            memory::write_user_profile,
            // ── Soul ──
            soul::read_soul,
            soul::write_soul,
            soul::reset_soul,
            // ── Models ──
            models::list_models,
            models::add_model,
            models::remove_model,
            models::update_model,
            model_discovery::discover_provider_models,
            // ── Tools ──
            tools::get_toolsets,
            tools::set_toolset_enabled,
            // ── Skills ──
            skills::list_installed_skills,
            skills::list_bundled_skills,
            skills::get_skill_content,
            skills::install_skill,
            skills::uninstall_skill,
            // ── Cron ──
            cronjobs::list_cron_jobs,
            cronjobs::create_cron_job,
            cronjobs::remove_cron_job,
            cronjobs::pause_cron_job,
            cronjobs::resume_cron_job,
            cronjobs::trigger_cron_job,
            // ── Kanban ──
            kanban::kanban_list_boards,
            kanban::kanban_current_board,
            kanban::kanban_switch_board,
            kanban::kanban_create_board,
            kanban::kanban_remove_board,
            kanban::kanban_list_tasks,
            kanban::kanban_get_task,
            kanban::kanban_create_task,
            kanban::kanban_assign_task,
            kanban::kanban_complete_task,
            kanban::kanban_block_task,
            kanban::kanban_unblock_task,
            kanban::kanban_archive_task,
            kanban::kanban_specify_task,
            kanban::kanban_reclaim_task,
            kanban::kanban_comment_task,
            kanban::kanban_dispatch_once,
            // ── Claw3D ──
            claw3d::claw3d_status,
            claw3d::claw3d_get_port,
            claw3d::claw3d_set_port,
            claw3d::claw3d_get_ws_url,
            claw3d::claw3d_set_ws_url,
            claw3d::claw3d_start_all,
            claw3d::claw3d_stop_all,
            claw3d::claw3d_get_logs,
            claw3d::claw3d_start_dev,
            claw3d::claw3d_stop_dev,
            claw3d::claw3d_start_adapter,
            claw3d::claw3d_stop_adapter,
            // ── SSH ──
            ssh::test_ssh_connection,
            ssh::start_ssh_tunnel,
            ssh::stop_ssh_tunnel,
            ssh::is_ssh_tunnel_active,
            // ── Attachment ──
            attachment_staging::stage_attachment,
            attachment_staging::clear_staged_attachments,
            // ── App ──
            get_app_version,
            get_system_info,
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            let msg = format!("Hermes Desktop failed to start: {:#?}", e);
            // Write crash log to desktop so user can see it
            if let Some(desktop) = dirs_next::desktop_dir() {
                let _ = std::fs::write(desktop.join("hermes-crash.log"), &msg);
            }
            eprintln!("{}", msg);
            std::process::exit(1);
        });
}

pub fn emit_event<R: tauri::Runtime>(
    app_handle: &tauri::AppHandle<R>,
    event: &str,
    payload: impl serde::Serialize + Clone,
) {
    let _ = app_handle.emit(event, payload);
}
