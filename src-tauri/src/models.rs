use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MonitorInfo {
    pub id: String,
    pub name: String,
    pub is_primary: bool,
    pub position: Point,
    pub size: Size,
    pub work_area_position: Point,
    pub work_area_size: Size,
    pub scale_factor: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Theme {
    Gallery,
    MinimalOled,
    ColorField,
    VinylRoom,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PausedBehavior {
    DimmedArtwork,
    PausedComposition,
    Clock,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum IdleBehavior {
    AmbientClock,
    Status,
    Black,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct AppSettings {
    pub schema_version: u32,
    pub spotify_client_id: String,
    pub selected_monitor_id: Option<String>,
    pub selected_monitor_name: Option<String>,
    pub selected_monitor_position: Option<Point>,
    pub selected_monitor_size: Option<Size>,
    pub return_delay_ms: u64,
    pub entry_dwell_ms: u64,
    pub edge_hysteresis_px: i32,
    pub cover_taskbar: bool,
    pub launch_at_startup: bool,
    pub start_minimized: bool,
    pub static_background: bool,
    pub show_artist_view: bool,
    pub theme: Theme,
    pub paused_behavior: PausedBehavior,
    pub idle_behavior: IdleBehavior,
    pub oled_dim_after_minutes: u32,
    pub setup_complete: bool,
    pub safety_shortcut: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            schema_version: 1,
            spotify_client_id: String::new(),
            selected_monitor_id: None,
            selected_monitor_name: None,
            selected_monitor_position: None,
            selected_monitor_size: None,
            return_delay_ms: 500,
            entry_dwell_ms: 0,
            edge_hysteresis_px: 4,
            cover_taskbar: true,
            launch_at_startup: false,
            start_minimized: true,
            static_background: false,
            show_artist_view: true,
            theme: Theme::Gallery,
            paused_behavior: PausedBehavior::DimmedArtwork,
            idle_behavior: IdleBehavior::AmbientClock,
            oled_dim_after_minutes: 30,
            setup_complete: false,
            safety_shortcut: "Ctrl+Shift+F12".to_owned(),
        }
    }
}

impl AppSettings {
    pub fn sanitize(mut self) -> Self {
        self.schema_version = 1;
        self.spotify_client_id = self.spotify_client_id.trim().to_owned();
        self.return_delay_ms = self.return_delay_ms.min(30_000);
        self.entry_dwell_ms = self.entry_dwell_ms.min(2_000);
        self.edge_hysteresis_px = self.edge_hysteresis_px.clamp(0, 64);
        self.oled_dim_after_minutes = self.oled_dim_after_minutes.clamp(5, 240);
        if self.safety_shortcut.trim().is_empty() {
            self.safety_shortcut = "Ctrl+Shift+F12".to_owned();
        }
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Artwork {
    pub url: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ArtistVisual {
    pub name: String,
    pub spotify_url: String,
    pub artwork: Vec<Artwork>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Palette {
    pub primary: String,
    pub secondary: String,
    pub accent: String,
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            primary: "#161b24".to_owned(),
            secondary: "#31233e".to_owned(),
            accent: "#b7a8d8".to_owned(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PlaybackStatus {
    Playing,
    Paused,
    Idle,
    Offline,
    AuthRequired,
    Error,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PlaybackItemType {
    Track,
    Episode,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Playback {
    pub status: PlaybackStatus,
    pub item_type: PlaybackItemType,
    pub item_id: Option<String>,
    pub uri: Option<String>,
    pub title: String,
    pub creators: Vec<String>,
    pub collection_title: Option<String>,
    pub artwork: Vec<Artwork>,
    pub featured_artist: Option<ArtistVisual>,
    pub duration_ms: u64,
    pub progress_ms: u64,
    pub observed_at: u64,
    pub is_explicit: bool,
    pub spotify_url: Option<String>,
    pub palette: Palette,
    pub message: Option<String>,
}

impl Playback {
    pub fn auth_required(now: u64) -> Self {
        Self {
            status: PlaybackStatus::AuthRequired,
            item_type: PlaybackItemType::Unknown,
            item_id: None,
            uri: None,
            title: "Connect Spotify to begin".to_owned(),
            creators: Vec::new(),
            collection_title: None,
            artwork: Vec::new(),
            featured_artist: None,
            duration_ms: 0,
            progress_ms: 0,
            observed_at: now,
            is_explicit: false,
            spotify_url: None,
            palette: Palette::default(),
            message: None,
        }
    }

    pub fn idle(now: u64) -> Self {
        Self {
            status: PlaybackStatus::Idle,
            title: "Nothing playing".to_owned(),
            ..Self::auth_required(now)
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum AuthState {
    NotConfigured,
    Disconnected,
    Authorizing,
    Connected,
    Expired,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AuthStatus {
    pub state: AuthState,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at_unix_ms: u64,
    pub scope: String,
}

impl StoredTokens {
    pub fn needs_refresh(&self, now_unix_ms: u64) -> bool {
        now_unix_ms.saturating_add(60_000) >= self.expires_at_unix_ms
    }
}

#[cfg(test)]
mod tests {
    use super::StoredTokens;

    #[test]
    fn refreshes_one_minute_before_expiry() {
        let tokens = StoredTokens {
            access_token: "access".into(),
            refresh_token: "refresh".into(),
            expires_at_unix_ms: 120_000,
            scope: "read".into(),
        };
        assert!(tokens.needs_refresh(60_000));
        assert!(!tokens.needs_refresh(59_999));
    }
}
