mod commands;
mod error;
mod features;
mod models;
mod parser;
mod services;
mod system_paths;

use commands::AppState;
use services::ConfigService;
use std::sync::Mutex;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_global_shortcut::ShortcutState;

/// Apply DWM shadow to a borderless window on Windows.
/// Uses DwmExtendFrameIntoClientArea with a 1px left margin to trick
/// DWM into rendering a shadow without a visible border.
#[cfg(windows)]
fn apply_dwm_shadow(window: &tauri::WebviewWindow) {
    #[repr(C)]
    struct MARGINS {
        left: i32,
        right: i32,
        top: i32,
        bottom: i32,
    }

    #[link(name = "dwmapi")]
    extern "system" {
        fn DwmExtendFrameIntoClientArea(
            hwnd: *mut std::ffi::c_void,
            margins: *const MARGINS,
        ) -> i32;
    }

    if let Ok(hwnd) = window.hwnd() {
        let margins = MARGINS {
            left: 1,
            right: 0,
            top: 0,
            bottom: 0,
        };
        unsafe {
            DwmExtendFrameIntoClientArea(hwnd.0, &margins);
        }
    }
}

fn show_main_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config = ConfigService::load();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_main_window(app);
        }))
        .manage(AppState {
            groups: Mutex::new(Vec::new()),
            config: Mutex::new(config),
        })
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        show_main_window(app);
                    }
                })
                .build(),
        )
        .setup(|app| {
            // System tray
            let show_item =
                MenuItem::with_id(app, "show", "显示窗口 / Show", true, None::<&str>)?;
            let flush_item =
                MenuItem::with_id(app, "flush_dns", "刷新 DNS / Flush DNS", true, None::<&str>)?;
            let quit_item =
                MenuItem::with_id(app, "quit", "退出 / Exit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &flush_item, &quit_item])?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Guohe Hosts")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        show_main_window(app);
                    }
                    "flush_dns" => {
                        let _ = std::process::Command::new("ipconfig")
                            .arg("/flushdns")
                            .output();
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        show_main_window(&app);
                    }
                })
                .build(app)?;

            // Apply DWM shadow (no border) to the main window
            #[cfg(windows)]
            if let Some(window) = app.get_webview_window("main") {
                apply_dwm_shadow(&window);
            }

            // Register global shortcut from config
            commands::register_toggle_shortcut(app.handle());

            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            // Config
            commands::get_config,
            commands::save_config,
            // Hosts file
            commands::read_hosts_file,
            commands::write_hosts_file,
            features::update::command::check_update,
            commands::open_external_url,
            commands::get_system_hosts_path,
            // DNS
            commands::flush_dns,
            // Backup
            commands::create_backup,
            commands::list_backups,
            commands::restore_backup,
            commands::delete_backup,
            // Import/Export
            commands::import_from_text,
            commands::export_group,
            // Groups
            commands::get_groups,
            commands::add_group,
            commands::delete_group,
            commands::activate_group,
            commands::rename_group,
            commands::reorder_groups,
            // Entries
            commands::add_entry,
            commands::update_entry,
            commands::delete_entry,
            commands::toggle_entry,
            commands::reorder_entries,
            // CodeMirror
            commands::get_group_text,
            commands::update_group_from_text,
            // Shortcut
            commands::update_global_shortcut,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
