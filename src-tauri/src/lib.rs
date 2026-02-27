mod commands;
mod config_handler;
mod log_handler;
mod monitoring;
mod process_manager;
mod terminal;
mod types;

use commands::AppState;
use config_handler::ConfigHandler;
use log_handler::LogHandler;
use process_manager::ProcessManager;
use std::sync::Arc;
use sysinfo;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    Emitter, Manager,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize config and log directories
    let log_dir = ConfigHandler::get_logs_dir()
        .expect("Failed to get logs directory");
    let log_handler = LogHandler::new(log_dir)
        .expect("Failed to initialize log handler");

    let process_manager = Arc::new(std::sync::Mutex::new(ProcessManager::new()));
    let log_handler = Arc::new(log_handler);
    
    // Pre-warm sysinfo so the first CPU reading has a prior snapshot to diff against.
    let sys = {
        let mut s = sysinfo::System::new_all();
        s.refresh_all();
        s
    };

    let app_state = AppState {
        manager: Arc::clone(&process_manager),
        log_handler: Arc::clone(&log_handler),
        system: Arc::new(std::sync::Mutex::new(sys)),
        terminal: Arc::new(std::sync::Mutex::new(terminal::TerminalManager::new())),
    };

    tauri::Builder::default()
        .manage(app_state)
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Build system tray menu
            let show = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
            let hide = MenuItem::with_id(app, "hide", "Hide", true, None::<&str>)?;
            let start_all_item = MenuItem::with_id(app, "start_all", "Start All", true, None::<&str>)?;
            let stop_all_item = MenuItem::with_id(app, "stop_all", "Stop All", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let sep1 = PredefinedMenuItem::separator(app)?;
            let sep2 = PredefinedMenuItem::separator(app)?;

            let menu = Menu::with_items(
                app,
                &[&show, &hide, &sep1, &start_all_item, &stop_all_item, &sep2, &quit],
            )?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_tray_icon_event(|tray, event| {
                    // Left-click on tray icon → show window.
                    // Right-click is handled automatically by the menu.
                    if let tauri::tray::TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        button_state: tauri::tray::MouseButtonState::Up,
                        ..
                    } = event
                    {
                        if let Some(w) = tray.app_handle().get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                })
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    "hide" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.hide();
                        }
                    }
                    "start_all" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.emit("app:start_all", ());
                        }
                    }
                    "stop_all" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.emit("app:stop_all", ());
                        }
                    }
                    "quit" => {
                        std::process::exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_processes,
            commands::start_process,
            commands::stop_process,
            commands::restart_process,
            commands::add_process,
            commands::remove_process,
            commands::update_process,
            commands::get_metrics,
            commands::get_logs,
            commands::clear_logs,
            commands::save_config,
            commands::load_config,
            commands::set_auto_start,
            commands::start_all,
            commands::stop_all,
            commands::terminal_run,
            commands::terminal_kill,
            commands::terminal_set_cwd,
            commands::terminal_get_cwd,
            commands::terminal_add_process,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

