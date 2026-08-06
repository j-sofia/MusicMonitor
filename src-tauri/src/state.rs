use std::sync::{
    atomic::{AtomicBool, AtomicU64},
    Arc,
};

use reqwest::Client;
use tauri::AppHandle;
use tokio::sync::{Mutex, RwLock};

use crate::models::{AppSettings, AuthState, AuthStatus, Playback};

pub struct AppState {
    pub settings: RwLock<AppSettings>,
    pub playback: RwLock<Playback>,
    pub auth: RwLock<AuthStatus>,
    pub oauth_lock: Mutex<()>,
    pub http: Client,
    pub preview_until_unix_ms: AtomicU64,
    pub display_suppressed: AtomicBool,
}

impl AppState {
    pub fn new(app: &AppHandle) -> Arc<Self> {
        let settings = crate::config::load(app);
        let now = crate::time::unix_ms();
        let has_tokens = crate::secrets::read().ok().flatten().is_some();
        let auth = if settings.spotify_client_id.is_empty() {
            AuthStatus {
                state: AuthState::NotConfigured,
                message: Some("Enter a Spotify client ID to connect.".to_owned()),
            }
        } else if has_tokens {
            AuthStatus {
                state: AuthState::Connected,
                message: Some("Tokens are protected by Windows Credential Manager.".to_owned()),
            }
        } else {
            AuthStatus {
                state: AuthState::Disconnected,
                message: None,
            }
        };
        let playback = if has_tokens {
            Playback::idle(now)
        } else {
            Playback::auth_required(now)
        };

        Arc::new(Self {
            settings: RwLock::new(settings),
            playback: RwLock::new(playback),
            auth: RwLock::new(auth),
            oauth_lock: Mutex::new(()),
            http: Client::builder()
                .user_agent(concat!("MusicMonitor/", env!("CARGO_PKG_VERSION")))
                .connect_timeout(std::time::Duration::from_secs(10))
                .timeout(std::time::Duration::from_secs(20))
                .build()
                .expect("static HTTP client configuration should be valid"),
            preview_until_unix_ms: AtomicU64::new(0),
            display_suppressed: AtomicBool::new(false),
        })
    }
}
