use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

pub fn build(app: &tauri::App) -> tauri::Result<()> {
    let show = MenuItem::with_id(
        app,
        "show",
        "Show display for 15 seconds",
        true,
        None::<&str>,
    )?;
    let hide = MenuItem::with_id(app, "hide", "Hide display", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Settings…", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit MusicMonitor", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &hide, &settings, &separator, &quit])?;

    let mut builder = TrayIconBuilder::with_id("musicmonitor")
        .menu(&menu)
        .tooltip("MusicMonitor")
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => {
                let state = app
                    .state::<std::sync::Arc<crate::state::AppState>>()
                    .inner()
                    .clone();
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    let settings = state.settings.read().await.clone();
                    if let Ok(monitors) = crate::monitor::list(&app) {
                        if let Some(monitor) = crate::monitor::resolve(&settings, &monitors) {
                            state.preview_until_unix_ms.store(
                                crate::time::unix_ms().saturating_add(15_000),
                                std::sync::atomic::Ordering::Relaxed,
                            );
                            state
                                .display_suppressed
                                .store(false, std::sync::atomic::Ordering::Relaxed);
                            crate::monitor::show_display(&app, monitor, settings.cover_taskbar);
                        }
                    }
                });
            }
            "hide" => {
                let state = app.state::<std::sync::Arc<crate::state::AppState>>();
                state
                    .preview_until_unix_ms
                    .store(0, std::sync::atomic::Ordering::Relaxed);
                state
                    .display_suppressed
                    .store(true, std::sync::atomic::Ordering::Relaxed);
                crate::monitor::hide_display(app);
            }
            "settings" => crate::reveal_settings(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::DoubleClick {
                button: MouseButton::Left,
                ..
            }
            | TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                crate::reveal_settings(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(())
}
