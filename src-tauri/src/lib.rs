mod commands;
mod config;
mod models;
mod monitor;
mod oauth;
mod secrets;
mod spotify;
mod state;
mod time;
mod tray;

use std::sync::Arc;

use state::AppState;
use tauri::{Manager, WindowEvent};
use tauri_plugin_autostart::ManagerExt as AutostartExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

pub(crate) fn reveal_settings(app: &tauri::AppHandle) {
    monitor::hide_display(app);
    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            reveal_settings(app);
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_autostart::Builder::new()
                .args(["--background"])
                .build(),
        )
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        reveal_settings(app);
                    }
                })
                .build(),
        )
        .setup(|app| {
            let state = AppState::new(app.handle());
            let initial_settings = config::load(app.handle());
            app.manage::<Arc<AppState>>(state.clone());

            if let Some(display) = app.get_webview_window("display") {
                let _ = display.set_focusable(false);
                let _ = display.set_ignore_cursor_events(true);
            }
            if let Some(settings_window) = app.get_webview_window("settings") {
                let window_to_hide = settings_window.clone();
                settings_window.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = window_to_hide.hide();
                    }
                });
            }

            tray::build(app)?;

            if let Err(error) = app
                .global_shortcut()
                .register(initial_settings.safety_shortcut.as_str())
            {
                eprintln!("MusicMonitor safety shortcut could not be registered: {error}");
            }

            if initial_settings.launch_at_startup {
                let _ = app.autolaunch().enable();
            }

            monitor::start_worker(app.handle().clone(), state.clone());
            spotify::start_worker(app.handle().clone(), state);

            if !initial_settings.setup_complete || !initial_settings.start_minimized {
                reveal_settings(app.handle());
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::save_settings,
            commands::list_monitors,
            commands::get_auth_status,
            commands::begin_spotify_login,
            commands::disconnect_spotify,
            commands::get_playback,
            commands::complete_setup,
            commands::preview_display,
            commands::hide_display,
            commands::reveal_settings,
            commands::open_spotify_url,
            commands::open_external_url,
        ])
        .run(tauri::generate_context!())
        .expect("error while running MusicMonitor");
}
