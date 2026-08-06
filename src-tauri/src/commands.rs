use std::sync::{atomic::Ordering, Arc};

use tauri::{Emitter, Manager, State};
use tauri_plugin_autostart::ManagerExt as AutostartExt;
use tauri_plugin_opener::OpenerExt;
use url::Url;

use crate::{
    models::{AppSettings, AuthState, AuthStatus, MonitorInfo, Playback},
    state::AppState,
    time::unix_ms,
};

#[tauri::command]
pub async fn get_settings(state: State<'_, Arc<AppState>>) -> Result<AppSettings, String> {
    Ok(state.settings.read().await.clone())
}

#[tauri::command]
pub async fn save_settings(
    app: tauri::AppHandle,
    state: State<'_, Arc<AppState>>,
    settings: AppSettings,
) -> Result<AppSettings, String> {
    let settings = settings.sanitize();
    let launch_at_startup_changed = {
        let current = state.settings.read().await;
        current.launch_at_startup != settings.launch_at_startup
    };
    if launch_at_startup_changed && settings.launch_at_startup {
        app.autolaunch()
            .enable()
            .map_err(|error| format!("Could not enable launch with Windows: {error}"))?;
    } else if launch_at_startup_changed {
        app.autolaunch()
            .disable()
            .map_err(|error| format!("Could not disable launch with Windows: {error}"))?;
    }
    crate::config::save(&app, &settings)?;
    *state.settings.write().await = settings.clone();
    let _ = app.emit("settings://updated", &settings);

    if settings.spotify_client_id.is_empty() {
        crate::oauth::set_auth(
            &app,
            state.inner(),
            AuthState::NotConfigured,
            Some("Enter a Spotify client ID to connect.".to_owned()),
        )
        .await;
    }
    Ok(settings)
}

#[tauri::command]
pub fn list_monitors(app: tauri::AppHandle) -> Result<Vec<MonitorInfo>, String> {
    crate::monitor::list(&app)
}

#[tauri::command]
pub async fn get_auth_status(state: State<'_, Arc<AppState>>) -> Result<AuthStatus, String> {
    Ok(state.auth.read().await.clone())
}

#[tauri::command]
pub async fn begin_spotify_login(
    app: tauri::AppHandle,
    state: State<'_, Arc<AppState>>,
    client_id: String,
) -> Result<(), String> {
    let client_id = client_id.trim().to_owned();
    {
        let mut settings = state.settings.write().await;
        settings.spotify_client_id = client_id.clone();
        crate::config::save(&app, &settings)?;
    }
    crate::oauth::begin(app, state.inner().clone(), client_id)
}

#[tauri::command]
pub async fn disconnect_spotify(
    app: tauri::AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    crate::secrets::delete()?;
    let playback = Playback::auth_required(unix_ms());
    *state.playback.write().await = playback.clone();
    let _ = tauri::Emitter::emit(&app, "playback://updated", playback);
    crate::oauth::set_auth(
        &app,
        state.inner(),
        AuthState::Disconnected,
        Some("Spotify has been disconnected and local credentials were removed.".to_owned()),
    )
    .await;
    Ok(())
}

#[tauri::command]
pub async fn get_playback(state: State<'_, Arc<AppState>>) -> Result<Playback, String> {
    Ok(state.playback.read().await.clone())
}

#[tauri::command]
pub async fn complete_setup(
    app: tauri::AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let mut settings = state.settings.write().await;
    if settings.selected_monitor_id.is_none() {
        return Err("Choose a display before finishing setup.".to_owned());
    }
    settings.setup_complete = true;
    state.display_suppressed.store(false, Ordering::Relaxed);
    crate::config::save(&app, &settings)?;
    let _ = app.emit("settings://updated", settings.clone());
    if settings.start_minimized {
        if let Some(window) = app.get_webview_window("settings") {
            let _ = window.hide();
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn preview_display(
    app: tauri::AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<(), String> {
    let settings = state.settings.read().await.clone();
    let monitors = crate::monitor::list(&app)?;
    let monitor = crate::monitor::resolve(&settings, &monitors)
        .ok_or_else(|| "The selected display is not connected.".to_owned())?;
    state
        .preview_until_unix_ms
        .store(unix_ms().saturating_add(15_000), Ordering::Relaxed);
    state.display_suppressed.store(false, Ordering::Relaxed);
    crate::monitor::show_display(&app, monitor, settings.cover_taskbar);
    Ok(())
}

#[tauri::command]
pub fn hide_display(app: tauri::AppHandle, state: State<'_, Arc<AppState>>) {
    state.preview_until_unix_ms.store(0, Ordering::Relaxed);
    state.display_suppressed.store(true, Ordering::Relaxed);
    crate::monitor::hide_display(&app);
}

#[tauri::command]
pub fn reveal_settings(app: tauri::AppHandle) {
    crate::reveal_settings(&app);
}

#[tauri::command]
pub fn open_spotify_url(app: tauri::AppHandle, url: String) -> Result<(), String> {
    open_allowed_url(&app, &url, &["open.spotify.com"])
}

#[tauri::command]
pub fn open_external_url(app: tauri::AppHandle, url: String) -> Result<(), String> {
    open_allowed_url(
        &app,
        &url,
        &[
            "developer.spotify.com",
            "open.spotify.com",
            "www.spotify.com",
            "github.com",
        ],
    )
}

fn open_allowed_url(app: &tauri::AppHandle, value: &str, hosts: &[&str]) -> Result<(), String> {
    let parsed = Url::parse(value).map_err(|_| "The external link is invalid.".to_owned())?;
    if parsed.scheme() != "https" || !parsed.host_str().is_some_and(|host| hosts.contains(&host)) {
        return Err("MusicMonitor refused to open an untrusted external link.".to_owned());
    }
    app.opener()
        .open_url(parsed.as_str(), None::<&str>)
        .map_err(|error| format!("Could not open the external link: {error}"))
}

#[cfg(test)]
mod tests {
    use url::Url;

    #[test]
    fn external_url_policy_rejects_spoofed_hosts() {
        let allowed = ["open.spotify.com"];
        let valid = Url::parse("https://open.spotify.com/track/abc").unwrap();
        let spoofed = Url::parse("https://open.spotify.com.example.test/track/abc").unwrap();
        assert!(valid.host_str().is_some_and(|host| allowed.contains(&host)));
        assert!(!spoofed
            .host_str()
            .is_some_and(|host| allowed.contains(&host)));
    }
}
