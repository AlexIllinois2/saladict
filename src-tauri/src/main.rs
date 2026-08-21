// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod backup;
mod cli;
mod clipboard;
mod cmd;
mod config;
mod error;
mod hotkey;
mod lang_detect;
mod screenshot;
mod selection;
mod server;
mod system_ocr;
mod updater;
mod window;
mod utils;
mod mouse_hook;

use backup::*;
use cli::CliAction;
use clipboard::*;
use cmd::*;
use config::*;
use hotkey::*;
use lang_detect::*;
use log::info;
use once_cell::sync::OnceCell;
use screenshot::screenshot;
use server::*;
use std::sync::Mutex;
use system_ocr::*;
use tauri::Manager;
use tauri_plugin_log::{Target, TargetKind};
use tauri_plugin_notification::NotificationExt;
use updater::{check_update, check_notify};
use window::config_window;
use window::updater_window;

// Global AppHandle
pub static APP: OnceCell<tauri::AppHandle> = OnceCell::new();

// Text to be translated
pub struct StringWrapper(pub Mutex<String>);

// CLI action passed to this process (e.g. `saladict --selection-translate`)
pub static CLI_ACTION: OnceCell<CliAction> = OnceCell::new();

fn main() {
    // On Linux, prefer the native Wayland backend (GTK/WebKitGTK) instead of
    // XWayland. With both WAYLAND_DISPLAY and DISPLAY set, GTK may otherwise
    // fall back to X11/XWayland, where the window can fail to appear on a
    // Wayland session. Only force it when a Wayland session actually exists so
    // pure-X11 sessions still work.
    #[cfg(target_os = "linux")]
    if std::env::var_os("WAYLAND_DISPLAY").is_some() && std::env::var_os("GDK_BACKEND").is_none() {
        std::env::set_var("GDK_BACKEND", "wayland");
    }

    let args: Vec<String> = std::env::args().collect();
    let action = cli::parse_from(&args);
    if action == CliAction::Help {
        cli::print_help();
        return;
    }
    CLI_ACTION.get_or_init(|| action);

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|_app, args, _cwd| {
            // The app is already running: forward the invocation (e.g. a
            // click on the desktop launcher icon or a system shortcut) to
            // this instance instead of starting a second process.
            cli::run(cli::parse_from(&args));
        }))
        .plugin(
            tauri_plugin_log::Builder::default()
                .targets([
                    Target::new(TargetKind::LogDir { file_name: None }),
                    Target::new(TargetKind::Stdout),
                ])
                .build(),
        )
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .plugin(tauri_plugin_sql::Builder::default().build())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            info!("============== Start App ==============");
            #[cfg(not(feature = "app-store"))]
            {
                utils::query_accessibility_permissions();
            }
            // Global AppHandle
            APP.get_or_init(|| app.handle().clone());
            // Init Config
            info!("Init Config Store");
            init_config(app);
            // Check First Run
            if is_first_run() {
                // Open Config Window
                info!("First Run, opening config window");
                config_window();
            }
            // create thumb window
            let _ = window::get_thumb_window(0, 0);
            #[cfg(target_os = "macos")]
            {
                // hide macos dock icon if set
                let hide_dock_icon = get("hide_dock_icon")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
                if hide_dock_icon {
                    app.set_activation_policy(tauri::ActivationPolicy::Accessory);
                } else {
                    app.set_activation_policy(tauri::ActivationPolicy::Regular);
                }
            }
            app.manage(StringWrapper(Mutex::new("".to_string())));
            // Start http server
            start_server();
            // Register Global Shortcut
            match register_shortcut("all") {
                Ok(()) => {}
                Err(e) => {
                    let _ = app
                        .notification()
                        .builder()
                        .title("Failed to register global shortcut")
                        .body(e)
                        .show();
                }
            }
            match get("proxy_enable") {
                Some(v) => {
                    if v.as_bool().unwrap() && get("proxy_host").map_or(false, |host| !host.as_str().unwrap().is_empty()) {
                        let _ = set_proxy();
                    }
                }
                None => {}
            }
            // Check Update
            check_update(app.handle().clone());
            check_notify();
            if let Some(engine) = get("translate_detect_engine") {
                if engine.as_str().unwrap() == "local" {
                    init_lang_detect();
                }
            }
            let clipboard_monitor = match get("clipboard_monitor") {
                Some(v) => v.as_bool().unwrap(),
                None => {
                    set("clipboard_monitor", false);
                    false
                }
            };
            app.manage(ClipboardMonitorEnableWrapper(Mutex::new(
                clipboard_monitor.to_string(),
            )));
            start_clipboard_monitor(app.handle().clone());
            // Execute the CLI action. The default action opens the input
            // translate window, so clicking the desktop launcher icon brings
            // up the translate popup (or forwards to the running instance).
            let action = *CLI_ACTION.get().unwrap();
            cli::run(action);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            reload_store,
            get_text,
            cut_image,
            get_base64,
            copy_img,
            system_ocr,
            set_proxy,
            unset_proxy,
            run_binary,
            open_devtools,
            register_shortcut_by_frontend,
            config_window,
            updater_window,
            screenshot,
            lang_detect,
            webdav,
            local,
            install_plugin,
            font_list,
            aliyun,
            is_app_store_version
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        // not exit when window close
        .run(|_app_handle, event| {
            match event {
                tauri::RunEvent::ExitRequested { api, code, .. } => {
                    // Only prevent exit when it was triggered by closing the
                    // last window (code == None), so the app keeps running in
                    // the background and global shortcuts keep working.
                    // Allow an explicit app.exit()/restart (code == Some) to
                    // actually quit the app.
                    if code.is_none() {
                        api.prevent_exit();
                    }
                }
                tauri::RunEvent::Ready => {
                    mouse_hook::bind_mouse_hook();
                }
                _ => {}
            }
        });
}
