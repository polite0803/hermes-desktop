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
mod mcp;
mod skills_hub;
mod plugins;
mod usage;
mod context_files;

use std::collections::HashMap;
use tauri::menu::MenuItem;
use tauri::tray::TrayIconBuilder;
use tauri::Manager;

pub struct AppState {
    pub chat_state: std::sync::Mutex<hermes::ChatState>,
    pub gateway_state: std::sync::Mutex<hermes::GatewayState>,
    pub ssh_tunnel_state: std::sync::Mutex<ssh::SshTunnelState>,
}

// ═══════════════════════════════════════════════
//  I18n — native UI strings (menu / tray / window)
// ═══════════════════════════════════════════════

type L10n = HashMap<&'static str, &'static str>;

const UI_STRINGS: &[(&str, &[(&str, &str)])] = &[
    ("en", &[
        ("menu.chat", "Chat"),
        ("menu.edit", "Edit"),
        ("menu.view", "View"),
        ("menu.window", "Window"),
        ("menu.help", "Help"),
        ("menu.newChat", "New Chat"),
        ("menu.searchSessions", "Search Sessions"),
        ("menu.about", "About Hermes Agent"),
        ("tray.tooltip", "Hermes Agent"),
        ("tray.showWindow", "Show Window"),
        ("tray.quit", "Quit"),
    ]),
    ("zh-CN", &[
        ("menu.chat", "聊天"),
        ("menu.edit", "编辑"),
        ("menu.view", "视图"),
        ("menu.window", "窗口"),
        ("menu.help", "帮助"),
        ("menu.newChat", "新建聊天"),
        ("menu.searchSessions", "搜索会话"),
        ("menu.about", "关于 Hermes Agent"),
        ("tray.tooltip", "Hermes Agent"),
        ("tray.showWindow", "显示窗口"),
        ("tray.quit", "退出"),
    ]),
    ("es", &[
        ("menu.chat", "Chat"),
        ("menu.edit", "Editar"),
        ("menu.view", "Ver"),
        ("menu.window", "Ventana"),
        ("menu.help", "Ayuda"),
        ("menu.newChat", "Nuevo Chat"),
        ("menu.searchSessions", "Buscar Sesiones"),
        ("menu.about", "Acerca de Hermes Agent"),
        ("tray.tooltip", "Hermes Agent"),
        ("tray.showWindow", "Mostrar Ventana"),
        ("tray.quit", "Salir"),
    ]),
    ("ja", &[
        ("menu.chat", "チャット"),
        ("menu.edit", "編集"),
        ("menu.view", "表示"),
        ("menu.window", "ウィンドウ"),
        ("menu.help", "ヘルプ"),
        ("menu.newChat", "新規チャット"),
        ("menu.searchSessions", "セッション検索"),
        ("menu.about", "Hermes Agent について"),
        ("tray.tooltip", "Hermes Agent"),
        ("tray.showWindow", "ウィンドウを表示"),
        ("tray.quit", "終了"),
    ]),
    ("id", &[
        ("menu.chat", "Obrolan"),
        ("menu.edit", "Edit"),
        ("menu.view", "Tampilan"),
        ("menu.window", "Jendela"),
        ("menu.help", "Bantuan"),
        ("menu.newChat", "Obrolan Baru"),
        ("menu.searchSessions", "Cari Sesi"),
        ("menu.about", "Tentang Hermes Agent"),
        ("tray.tooltip", "Hermes Agent"),
        ("tray.showWindow", "Tampilkan Jendela"),
        ("tray.quit", "Keluar"),
    ]),
    ("pt-BR", &[
        ("menu.chat", "Chat"),
        ("menu.edit", "Editar"),
        ("menu.view", "Ver"),
        ("menu.window", "Janela"),
        ("menu.help", "Ajuda"),
        ("menu.newChat", "Novo Chat"),
        ("menu.searchSessions", "Buscar Sessões"),
        ("menu.about", "Sobre o Hermes Agent"),
        ("tray.tooltip", "Hermes Agent"),
        ("tray.showWindow", "Mostrar Janela"),
        ("tray.quit", "Sair"),
    ]),
    ("pt-PT", &[
        ("menu.chat", "Chat"),
        ("menu.edit", "Editar"),
        ("menu.view", "Ver"),
        ("menu.window", "Janela"),
        ("menu.help", "Ajuda"),
        ("menu.newChat", "Novo Chat"),
        ("menu.searchSessions", "Pesquisar Sessões"),
        ("menu.about", "Sobre o Hermes Agent"),
        ("tray.tooltip", "Hermes Agent"),
        ("tray.showWindow", "Mostrar Janela"),
        ("tray.quit", "Sair"),
    ]),
    ("zh-TW", &[
        ("menu.chat", "聊天"),
        ("menu.edit", "編輯"),
        ("menu.view", "檢視"),
        ("menu.window", "視窗"),
        ("menu.help", "說明"),
        ("menu.newChat", "新增聊天"),
        ("menu.searchSessions", "搜尋工作階段"),
        ("menu.about", "關於 Hermes Agent"),
        ("tray.tooltip", "Hermes Agent"),
        ("tray.showWindow", "顯示視窗"),
        ("tray.quit", "結束"),
    ]),
];

fn t(key: &str, locale: &str) -> String {
    if let Some((_, strings)) = UI_STRINGS.iter().find(|(l, _)| *l == locale) {
        if let Some((_, v)) = strings.iter().find(|(k, _)| *k == key) {
            return v.to_string();
        }
    }
    // fallback to English
    if locale != "en" { return t(key, "en"); }
    key.to_string()
}

// ═══════════════════════════════════════════════

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

fn current_locale() -> String {
    crate::locale::get_locale()
}

pub fn run() {
    env_logger::init();
    let locale = current_locale();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_process::init())
        .setup(move |app| {
            let loc = locale.clone();

            let icon = app.default_window_icon().cloned();
            let mut builder = TrayIconBuilder::new().tooltip(t("tray.tooltip", &loc));
            if let Some(ic) = icon { builder = builder.icon(ic); }
            let _tray = builder
                .on_menu_event(|app, event| {
                    if event.id().as_ref() == "quit" { app.exit(0); }
                    if event.id().as_ref() == "show" {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show(); let _ = w.set_focus();
                        }
                    }
                })
                .menu(&tauri::menu::MenuBuilder::new(app)
                    .item(&MenuItem::with_id(app, "show", t("tray.showWindow", &loc), true, None::<&str>)?)
                    .separator()
                    .item(&MenuItem::with_id(app, "quit", t("tray.quit", &loc), true, None::<&str>)?)
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
            installer::check_install, installer::verify_install, installer::start_install,
            installer::get_hermes_version, installer::refresh_hermes_version,
            installer::run_hermes_doctor, installer::run_hermes_update,
            installer::check_openclaw, installer::run_hermes_migrate,
            installer::run_hermes_backup, installer::run_hermes_import, installer::run_hermes_dump,
            installer::discover_memory_providers, installer::read_logs,
            installer::check_for_updates, installer::download_update, installer::install_update,
            installer::claw3d_setup, installer::select_folder, installer::open_external,
            config::get_config_value, config::set_config_value, config::get_env_all, config::set_env,
            config::get_hermes_home, config::get_model_config, config::set_model_config,
            config::get_connection_config, config::set_connection_config,
            config::is_remote_mode, config::is_remote_only_mode,
            config::get_platform_enabled_all, config::set_platform_enabled,
            config::get_credential_pool, config::set_credential_pool,
            locale::get_locale, locale::set_locale,
            hermes::send_message, hermes::abort_chat,
            hermes::start_gateway, hermes::stop_gateway, hermes::gateway_status,
            hermes::test_remote_connection,
            sessions::list_sessions, sessions::get_session_messages,
            sessions::delete_session, sessions::search_sessions,
            session_cache::list_cached_sessions, session_cache::sync_session_cache,
            session_cache::update_session_title,
            profiles::list_profiles, profiles::create_profile,
            profiles::delete_profile, profiles::set_active_profile,
            memory::read_memory, memory::add_memory_entry, memory::update_memory_entry,
            memory::remove_memory_entry, memory::write_user_profile,
            soul::read_soul, soul::write_soul, soul::reset_soul,
            soul::list_personalities, soul::apply_personality,
            models::list_models, models::add_model, models::remove_model, models::update_model,
            model_discovery::discover_provider_models,
            tools::get_toolsets, tools::set_toolset_enabled,
            skills::list_installed_skills, skills::list_bundled_skills,
            skills::get_skill_content, skills::install_skill, skills::uninstall_skill,
            skills_hub::search_skills_hub, skills_hub::install_from_hub,
            cronjobs::list_cron_jobs, cronjobs::create_cron_job, cronjobs::remove_cron_job,
            cronjobs::pause_cron_job, cronjobs::resume_cron_job, cronjobs::trigger_cron_job,
            kanban::kanban_list_boards, kanban::kanban_current_board, kanban::kanban_switch_board,
            kanban::kanban_create_board, kanban::kanban_remove_board, kanban::kanban_list_tasks,
            kanban::kanban_get_task, kanban::kanban_create_task, kanban::kanban_assign_task,
            kanban::kanban_complete_task, kanban::kanban_block_task, kanban::kanban_unblock_task,
            kanban::kanban_archive_task, kanban::kanban_specify_task, kanban::kanban_reclaim_task,
            kanban::kanban_comment_task, kanban::kanban_dispatch_once,
            claw3d::claw3d_status, claw3d::claw3d_get_port, claw3d::claw3d_set_port,
            claw3d::claw3d_get_ws_url, claw3d::claw3d_set_ws_url,
            claw3d::claw3d_start_all, claw3d::claw3d_stop_all, claw3d::claw3d_get_logs,
            claw3d::claw3d_start_dev, claw3d::claw3d_stop_dev,
            claw3d::claw3d_start_adapter, claw3d::claw3d_stop_adapter,
            ssh::test_ssh_connection, ssh::start_ssh_tunnel,
            ssh::stop_ssh_tunnel, ssh::is_ssh_tunnel_active,
            mcp::list_mcp_servers, mcp::add_mcp_server, mcp::remove_mcp_server, mcp::update_mcp_server, mcp::test_mcp_server,
            attachment_staging::stage_attachment, attachment_staging::clear_staged_attachments,
            plugins::list_plugins, plugins::enable_plugin, plugins::disable_plugin,
            usage::get_usage_stats, usage::get_insights,
            context_files::list_context_files, context_files::read_context_file, context_files::write_context_file,
            get_app_version, get_system_info,
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            let msg = format!("Hermes Desktop failed to start: {:#?}", e);
            if let Some(desktop) = dirs_next::desktop_dir() {
                let _ = std::fs::write(desktop.join("hermes-crash.log"), &msg);
            }
            eprintln!("{}", msg);
            std::process::exit(1);
        });
}
