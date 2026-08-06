use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use image::ImageReader;
use rand::Rng;
use reqwest::StatusCode;
use serde::Deserialize;
use tauri::Emitter;

use crate::{
    models::{
        ArtistVisual, Artwork, AuthState, Palette, Playback, PlaybackItemType, PlaybackStatus,
    },
    oauth::{self, TokenError},
    state::AppState,
    time::unix_ms,
};

const CURRENTLY_PLAYING_URL: &str = "https://api.spotify.com/v1/me/player/currently-playing";
const ARTIST_URL_PREFIX: &str = "https://api.spotify.com/v1/artists/";
const ARTIST_CACHE_LIMIT: usize = 64;

#[derive(Debug, Deserialize)]
struct CurrentlyPlaying {
    #[serde(default)]
    progress_ms: Option<u64>,
    #[serde(default)]
    is_playing: bool,
    #[serde(default)]
    currently_playing_type: String,
    #[serde(default)]
    item: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct ImageTransport {
    url: String,
    width: Option<u32>,
    height: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct ExternalUrls {
    #[serde(default)]
    spotify: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ArtistSummaryTransport {
    #[serde(default)]
    id: Option<String>,
    name: String,
}

#[derive(Debug, Deserialize)]
struct AlbumTransport {
    name: String,
    #[serde(default)]
    images: Vec<ImageTransport>,
}

#[derive(Debug, Deserialize)]
struct TrackTransport {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    uri: Option<String>,
    name: String,
    #[serde(default)]
    duration_ms: u64,
    #[serde(default)]
    explicit: bool,
    #[serde(default)]
    artists: Vec<ArtistSummaryTransport>,
    album: AlbumTransport,
    #[serde(default)]
    external_urls: Option<ExternalUrls>,
}

#[derive(Debug, Deserialize)]
struct ShowTransport {
    name: String,
    #[serde(default)]
    publisher: Option<String>,
    #[serde(default)]
    images: Vec<ImageTransport>,
}

#[derive(Debug, Deserialize)]
struct EpisodeTransport {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    uri: Option<String>,
    name: String,
    #[serde(default)]
    duration_ms: u64,
    #[serde(default)]
    explicit: bool,
    #[serde(default)]
    images: Vec<ImageTransport>,
    show: ShowTransport,
    #[serde(default)]
    external_urls: Option<ExternalUrls>,
}

#[derive(Debug, Deserialize)]
struct ArtistTransport {
    name: String,
    #[serde(default)]
    images: Vec<ImageTransport>,
    #[serde(default)]
    external_urls: Option<ExternalUrls>,
}

#[derive(Debug)]
struct NormalizedPlayback {
    playback: Playback,
    primary_artist_id: Option<String>,
}

impl From<Playback> for NormalizedPlayback {
    fn from(playback: Playback) -> Self {
        Self {
            playback,
            primary_artist_id: None,
        }
    }
}

struct CachedArtist {
    visual: Option<ArtistVisual>,
    refresh_after: Instant,
}

struct ArtistLookup {
    artist_id: String,
    access_token: String,
}

struct PollResult {
    playback: Playback,
    artist_lookup: Option<ArtistLookup>,
}

enum ArtistVisualResult {
    Found(ArtistVisual),
    Unavailable,
    Retryable,
}

impl From<Playback> for PollResult {
    fn from(playback: Playback) -> Self {
        Self {
            playback,
            artist_lookup: None,
        }
    }
}

#[derive(Debug)]
enum PollError {
    RateLimited(Duration),
    AuthRequired,
    Offline(String),
    Temporary(String),
}

pub fn start_worker(app: tauri::AppHandle, state: Arc<AppState>) {
    tauri::async_runtime::spawn(async move {
        let mut failure_count = 0_u32;
        let mut last_palette_item: Option<String> = None;
        let artist_cache = Arc::new(tokio::sync::Mutex::new(
            HashMap::<String, CachedArtist>::new(),
        ));

        loop {
            let settings = state.settings.read().await.clone();
            if settings.spotify_client_id.is_empty() {
                publish(&app, &state, Playback::auth_required(unix_ms())).await;
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }

            match poll_once(&app, &state, &settings.spotify_client_id).await {
                Ok(mut result) => {
                    failure_count = 0;
                    let previous = state.playback.read().await.clone();
                    preserve_enrichment(&mut result.playback, &previous);
                    let artist_candidate = if let Some(lookup) = result.artist_lookup.take() {
                        let mut cache = artist_cache.lock().await;
                        match cached_artist_visual(&mut cache, &lookup.artist_id) {
                            Some(visual) => {
                                result.playback.featured_artist = visual;
                                None
                            }
                            None => {
                                mark_artist_lookup_pending(&mut cache, &lookup.artist_id);
                                result
                                    .playback
                                    .item_id
                                    .clone()
                                    .map(|item_id| (lookup, item_id))
                            }
                        }
                    } else {
                        None
                    };
                    let palette_candidate = result
                        .playback
                        .artwork
                        .first()
                        .map(|artwork| artwork.url.clone())
                        .zip(result.playback.item_id.clone());
                    let delay = match result.playback.status {
                        PlaybackStatus::Playing => Duration::from_millis(2_500),
                        PlaybackStatus::Paused => Duration::from_millis(7_500),
                        _ => Duration::from_secs(15),
                    };
                    publish(&app, &state, result.playback).await;

                    if let Some((lookup, item_id)) = artist_candidate {
                        let app = app.clone();
                        let state = state.clone();
                        let artist_cache = artist_cache.clone();
                        tauri::async_runtime::spawn(async move {
                            let result = fetch_artist_visual(
                                &state.http,
                                &lookup.access_token,
                                &lookup.artist_id,
                            )
                            .await;
                            match result {
                                ArtistVisualResult::Found(visual) => {
                                    {
                                        let mut cache = artist_cache.lock().await;
                                        cache_artist_visual(
                                            &mut cache,
                                            lookup.artist_id,
                                            Some(visual.clone()),
                                        );
                                    }
                                    let mut current = state.playback.write().await;
                                    if current.item_id.as_deref() == Some(&item_id) {
                                        current.featured_artist = Some(visual);
                                        let updated = current.clone();
                                        drop(current);
                                        let _ = app.emit("playback://updated", updated);
                                    }
                                }
                                ArtistVisualResult::Unavailable => {
                                    let mut cache = artist_cache.lock().await;
                                    cache_artist_visual(&mut cache, lookup.artist_id, None);
                                }
                                // Leave the short pending entry in place. A later poll retries
                                // transient network, authorization, rate-limit, and 5xx failures.
                                ArtistVisualResult::Retryable => {}
                            }
                        });
                    }

                    if let Some((url, item_id)) = palette_candidate {
                        if last_palette_item.as_deref() != Some(&item_id) {
                            last_palette_item = Some(item_id.clone());
                            let app = app.clone();
                            let state = state.clone();
                            tauri::async_runtime::spawn(async move {
                                if let Some(palette) = extract_palette(&state.http, &url).await {
                                    let mut current = state.playback.write().await;
                                    if current.item_id.as_deref() == Some(&item_id) {
                                        current.palette = palette;
                                        let updated = current.clone();
                                        drop(current);
                                        let _ = app.emit("playback://updated", updated);
                                    }
                                }
                            });
                        }
                    } else {
                        last_palette_item = None;
                    }
                    tokio::time::sleep(delay).await;
                }
                Err(PollError::RateLimited(delay)) => {
                    failure_count = failure_count.saturating_add(1);
                    tokio::time::sleep(delay).await;
                }
                Err(PollError::AuthRequired) => {
                    last_palette_item = None;
                    publish(&app, &state, Playback::auth_required(unix_ms())).await;
                    tokio::time::sleep(Duration::from_secs(10)).await;
                }
                Err(PollError::Offline(message)) | Err(PollError::Temporary(message)) => {
                    last_palette_item = None;
                    failure_count = failure_count.saturating_add(1);
                    publish(
                        &app,
                        &state,
                        Playback {
                            status: PlaybackStatus::Offline,
                            title: "Waiting for Spotify".to_owned(),
                            message: Some(message),
                            ..Playback::idle(unix_ms())
                        },
                    )
                    .await;
                    tokio::time::sleep(backoff_delay(failure_count)).await;
                }
            }
        }
    });
}

fn preserve_enrichment(playback: &mut Playback, previous: &Playback) {
    if playback.item_id.is_some() && playback.item_id == previous.item_id {
        playback.palette = previous.palette.clone();
        playback.featured_artist = previous.featured_artist.clone();
    }
}

async fn poll_once(
    app: &tauri::AppHandle,
    state: &Arc<AppState>,
    client_id: &str,
) -> Result<PollResult, PollError> {
    let mut access_token = match oauth::access_token(app, state, client_id, false).await {
        Ok(token) => token,
        Err(TokenError::Missing | TokenError::Expired) => return Err(PollError::AuthRequired),
        Err(TokenError::Network(message) | TokenError::Storage(message)) => {
            return Err(PollError::Offline(message))
        }
    };
    let mut response = request_current(&state.http, &access_token).await?;
    if response.status() == StatusCode::UNAUTHORIZED {
        let refreshed = oauth::access_token(app, state, client_id, true)
            .await
            .map_err(|error| match error {
                TokenError::Missing | TokenError::Expired => PollError::AuthRequired,
                TokenError::Network(message) | TokenError::Storage(message) => {
                    PollError::Offline(message)
                }
            })?;
        response = request_current(&state.http, &refreshed).await?;
        access_token = refreshed;
    }

    match response.status() {
        StatusCode::NO_CONTENT => Ok(Playback::idle(unix_ms()).into()),
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
            oauth::set_auth(
                app,
                state,
                AuthState::Expired,
                Some("Spotify authorization needs to be renewed.".to_owned()),
            )
            .await;
            Err(PollError::AuthRequired)
        }
        StatusCode::TOO_MANY_REQUESTS => {
            let retry = response
                .headers()
                .get(reqwest::header::RETRY_AFTER)
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or(5)
                .clamp(1, 300);
            Err(PollError::RateLimited(Duration::from_secs(retry)))
        }
        status if status.is_server_error() => Err(PollError::Temporary(format!(
            "Spotify is temporarily unavailable (HTTP {}).",
            status.as_u16()
        ))),
        status if !status.is_success() => Err(PollError::Temporary(format!(
            "Spotify returned HTTP {}.",
            status.as_u16()
        ))),
        _ => {
            let response: CurrentlyPlaying = response.json().await.map_err(|error| {
                PollError::Temporary(format!("Spotify returned invalid playback data: {error}"))
            })?;
            let normalized = normalize(response, unix_ms());
            Ok(PollResult {
                playback: normalized.playback,
                artist_lookup: normalized.primary_artist_id.map(|artist_id| ArtistLookup {
                    artist_id,
                    access_token,
                }),
            })
        }
    }
}

async fn request_current(
    client: &reqwest::Client,
    access_token: &str,
) -> Result<reqwest::Response, PollError> {
    client
        .get(CURRENTLY_PLAYING_URL)
        .query(&[("additional_types", "track,episode")])
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|error| {
            PollError::Offline(format!(
                "Network unavailable; retrying automatically ({error})."
            ))
        })
}

fn normalize(response: CurrentlyPlaying, observed_at: u64) -> NormalizedPlayback {
    let Some(item) = response.item else {
        return if response.currently_playing_type == "ad" {
            unsupported_item(
                observed_at,
                "Spotify is playing an item that cannot be displayed.",
            )
            .into()
        } else {
            Playback::idle(observed_at).into()
        };
    };

    let item_type = item
        .get("type")
        .and_then(serde_json::Value::as_str)
        .unwrap_or(response.currently_playing_type.as_str());
    let status = if response.is_playing {
        PlaybackStatus::Playing
    } else {
        PlaybackStatus::Paused
    };

    match item_type {
        "track" => serde_json::from_value::<TrackTransport>(item)
            .map(|track| {
                let primary_artist_id = track
                    .artists
                    .first()
                    .and_then(|artist| artist.id.clone())
                    .filter(|id| {
                        id.chars()
                            .all(|character| character.is_ascii_alphanumeric())
                    });
                NormalizedPlayback {
                    playback: Playback {
                        status,
                        item_type: PlaybackItemType::Track,
                        item_id: track.id.or_else(|| track.uri.clone()),
                        uri: track.uri,
                        title: track.name,
                        creators: track
                            .artists
                            .into_iter()
                            .map(|artist| artist.name)
                            .collect(),
                        collection_title: Some(track.album.name),
                        artwork: normalize_artwork(track.album.images),
                        featured_artist: None,
                        duration_ms: track.duration_ms,
                        progress_ms: response.progress_ms.unwrap_or(0).min(track.duration_ms),
                        observed_at,
                        is_explicit: track.explicit,
                        spotify_url: track.external_urls.and_then(|urls| urls.spotify),
                        palette: Palette::default(),
                        message: None,
                    },
                    primary_artist_id,
                }
            })
            .unwrap_or_else(|_| {
                unsupported_item(observed_at, "Spotify returned an unsupported track.").into()
            }),
        "episode" => serde_json::from_value::<EpisodeTransport>(item)
            .map(|episode| {
                let artwork = if episode.images.is_empty() {
                    normalize_artwork(episode.show.images)
                } else {
                    normalize_artwork(episode.images)
                };
                let creator = episode
                    .show
                    .publisher
                    .clone()
                    .unwrap_or_else(|| episode.show.name.clone());
                Playback {
                    status,
                    item_type: PlaybackItemType::Episode,
                    item_id: episode.id.or_else(|| episode.uri.clone()),
                    uri: episode.uri,
                    title: episode.name,
                    creators: vec![creator],
                    collection_title: Some(episode.show.name),
                    artwork,
                    featured_artist: None,
                    duration_ms: episode.duration_ms,
                    progress_ms: response.progress_ms.unwrap_or(0).min(episode.duration_ms),
                    observed_at,
                    is_explicit: episode.explicit,
                    spotify_url: episode.external_urls.and_then(|urls| urls.spotify),
                    palette: Palette::default(),
                    message: None,
                }
                .into()
            })
            .unwrap_or_else(|_| {
                unsupported_item(observed_at, "Spotify returned an unsupported episode.").into()
            }),
        _ => unsupported_item(
            observed_at,
            "Spotify is playing an unsupported or private item.",
        )
        .into(),
    }
}

fn cached_artist_visual(
    cache: &mut HashMap<String, CachedArtist>,
    artist_id: &str,
) -> Option<Option<ArtistVisual>> {
    let now = Instant::now();
    if let Some(cached) = cache.get(artist_id) {
        if cached.refresh_after > now {
            return Some(cached.visual.clone());
        }
    }
    cache.remove(artist_id);
    None
}

fn mark_artist_lookup_pending(cache: &mut HashMap<String, CachedArtist>, artist_id: &str) {
    cache.insert(
        artist_id.to_owned(),
        CachedArtist {
            visual: None,
            refresh_after: Instant::now() + Duration::from_secs(30),
        },
    );
}

fn cache_artist_visual(
    cache: &mut HashMap<String, CachedArtist>,
    artist_id: String,
    visual: Option<ArtistVisual>,
) {
    let now = Instant::now();
    cache.retain(|_, entry| entry.refresh_after > now);
    if cache.len() >= ARTIST_CACHE_LIMIT {
        cache.clear();
    }
    let cache_for = if visual.is_some() {
        Duration::from_secs(24 * 60 * 60)
    } else {
        Duration::from_secs(5 * 60)
    };
    cache.insert(
        artist_id,
        CachedArtist {
            visual,
            refresh_after: now + cache_for,
        },
    );
}

async fn fetch_artist_visual(
    client: &reqwest::Client,
    access_token: &str,
    artist_id: &str,
) -> ArtistVisualResult {
    let Ok(response) = client
        .get(format!("{ARTIST_URL_PREFIX}{artist_id}"))
        .bearer_auth(access_token)
        .send()
        .await
    else {
        return ArtistVisualResult::Retryable;
    };
    if response.status() == StatusCode::TOO_MANY_REQUESTS
        || response.status() == StatusCode::UNAUTHORIZED
        || response.status().is_server_error()
    {
        return ArtistVisualResult::Retryable;
    }
    if !response.status().is_success() {
        return ArtistVisualResult::Unavailable;
    }
    let Ok(artist) = response.json::<ArtistTransport>().await else {
        return ArtistVisualResult::Retryable;
    };
    let Some(spotify_url) = artist.external_urls.and_then(|urls| urls.spotify) else {
        return ArtistVisualResult::Unavailable;
    };
    if !spotify_url.starts_with("https://open.spotify.com/artist/") {
        return ArtistVisualResult::Unavailable;
    }
    let artwork = normalize_artwork(artist.images);
    if artwork.is_empty() {
        return ArtistVisualResult::Unavailable;
    }
    ArtistVisualResult::Found(ArtistVisual {
        name: artist.name,
        spotify_url,
        artwork,
    })
}

fn normalize_artwork(images: Vec<ImageTransport>) -> Vec<Artwork> {
    images
        .into_iter()
        .filter(|image| image.url.starts_with("https://"))
        .map(|image| Artwork {
            url: image.url,
            width: image.width,
            height: image.height,
        })
        .collect()
}

fn unsupported_item(observed_at: u64, message: &str) -> Playback {
    Playback {
        status: PlaybackStatus::Idle,
        item_type: PlaybackItemType::Unknown,
        title: "This item cannot be displayed".to_owned(),
        message: Some(message.to_owned()),
        ..Playback::idle(observed_at)
    }
}

async fn publish(app: &tauri::AppHandle, state: &Arc<AppState>, playback: Playback) {
    *state.playback.write().await = playback.clone();
    let _ = app.emit("playback://updated", playback);
}

fn backoff_delay(failures: u32) -> Duration {
    let exponent = failures.saturating_sub(1).min(6);
    let base_ms = 1_000_u64.saturating_mul(1_u64 << exponent).min(60_000);
    let jitter = rand::thread_rng().gen_range(0..=base_ms / 5);
    Duration::from_millis(base_ms.saturating_add(jitter))
}

async fn extract_palette(client: &reqwest::Client, url: &str) -> Option<Palette> {
    let response = client.get(url).send().await.ok()?;
    if !response.status().is_success()
        || response
            .content_length()
            .is_some_and(|length| length > 10 * 1024 * 1024)
    {
        return None;
    }
    let bytes = response.bytes().await.ok()?;
    if bytes.len() > 10 * 1024 * 1024 {
        return None;
    }
    let image = ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .ok()?
        .decode()
        .ok()?
        .thumbnail(48, 48)
        .to_rgb8();

    let mut buckets = std::collections::HashMap::<(u8, u8, u8), u32>::new();
    for pixel in image.pixels() {
        let [r, g, b] = pixel.0;
        let luminance = (u16::from(r) * 3 + u16::from(g) * 6 + u16::from(b)) / 10;
        if !(18..=240).contains(&luminance) {
            continue;
        }
        let key = (r / 24 * 24, g / 24 * 24, b / 24 * 24);
        *buckets.entry(key).or_default() += 1;
    }
    let mut ranked: Vec<_> = buckets.into_iter().collect();
    ranked.sort_unstable_by_key(|(_, count)| std::cmp::Reverse(*count));
    let primary = ranked.first().map(|(color, _)| *color)?;
    let secondary = ranked
        .iter()
        .map(|(color, _)| *color)
        .find(|color| color_distance(*color, primary) > 70)
        .unwrap_or_else(|| shift_color(primary, 36));
    let accent = ranked
        .iter()
        .map(|(color, _)| *color)
        .max_by_key(|color| saturation(*color))
        .unwrap_or_else(|| shift_color(secondary, 52));

    Some(Palette {
        primary: darken_hex(primary),
        secondary: darken_hex(secondary),
        accent: color_hex(accent),
    })
}

fn color_distance(a: (u8, u8, u8), b: (u8, u8, u8)) -> u16 {
    u16::from(a.0.abs_diff(b.0)) + u16::from(a.1.abs_diff(b.1)) + u16::from(a.2.abs_diff(b.2))
}

fn saturation(color: (u8, u8, u8)) -> u8 {
    color.0.max(color.1).max(color.2) - color.0.min(color.1).min(color.2)
}

fn shift_color((r, g, b): (u8, u8, u8), shift: u8) -> (u8, u8, u8) {
    (
        g.saturating_add(shift / 2),
        b.saturating_add(shift),
        r.saturating_add(shift / 3),
    )
}

fn darken_hex((r, g, b): (u8, u8, u8)) -> String {
    color_hex((r / 2, g / 2, b / 2))
}

fn color_hex((r, g, b): (u8, u8, u8)) -> String {
    format!("#{r:02x}{g:02x}{b:02x}")
}

#[cfg(test)]
mod tests {
    use super::{backoff_delay, normalize, preserve_enrichment, CurrentlyPlaying};
    use crate::models::{ArtistVisual, Artwork, Palette, PlaybackItemType, PlaybackStatus};

    #[test]
    fn normalizes_track_without_leaking_transport_shape() {
        let response: CurrentlyPlaying = serde_json::from_value(serde_json::json!({
            "progress_ms": 1200,
            "is_playing": true,
            "currently_playing_type": "track",
            "item": {
                "type": "track",
                "id": "track-id",
                "uri": "spotify:track:track-id",
                "name": "Test Track",
                "duration_ms": 200000,
                "explicit": true,
                "artists": [{"id": "artist123", "name": "Artist"}],
                "album": {"name": "Album", "images": [{"url": "https://i.scdn.co/art", "width": 640, "height": 640}]},
                "external_urls": {"spotify": "https://open.spotify.com/track/track-id"}
            }
        })).unwrap();
        let normalized = normalize(response, 42);
        let playback = normalized.playback;
        assert_eq!(playback.status, PlaybackStatus::Playing);
        assert_eq!(playback.item_type, PlaybackItemType::Track);
        assert_eq!(playback.title, "Test Track");
        assert_eq!(playback.creators, vec!["Artist"]);
        assert!(playback.is_explicit);
        assert_eq!(normalized.primary_artist_id.as_deref(), Some("artist123"));
        assert!(playback.featured_artist.is_none());
    }

    #[test]
    fn null_item_clears_stale_artwork() {
        let response: CurrentlyPlaying = serde_json::from_value(serde_json::json!({
            "progress_ms": null,
            "is_playing": false,
            "currently_playing_type": "unknown",
            "item": null
        }))
        .unwrap();
        let playback = normalize(response, 42).playback;
        assert_eq!(playback.status, PlaybackStatus::Idle);
        assert!(playback.artwork.is_empty());
    }

    #[test]
    fn repeated_poll_preserves_palette_and_artist_enrichment() {
        let response: CurrentlyPlaying = serde_json::from_value(serde_json::json!({
            "progress_ms": 2500,
            "is_playing": true,
            "currently_playing_type": "track",
            "item": {
                "type": "track",
                "id": "same-track",
                "name": "Test Track",
                "duration_ms": 200000,
                "artists": [{"id": "artist123", "name": "Artist"}],
                "album": {"name": "Album", "images": []}
            }
        }))
        .unwrap();
        let mut next = normalize(response, 100).playback;
        let mut previous = next.clone();
        previous.palette = Palette {
            primary: "#010203".into(),
            secondary: "#040506".into(),
            accent: "#070809".into(),
        };
        previous.featured_artist = Some(ArtistVisual {
            name: "Artist".into(),
            spotify_url: "https://open.spotify.com/artist/artist123".into(),
            artwork: vec![Artwork {
                url: "https://i.scdn.co/artist".into(),
                width: Some(640),
                height: Some(640),
            }],
        });

        preserve_enrichment(&mut next, &previous);

        assert_eq!(next.palette, previous.palette);
        assert_eq!(next.featured_artist, previous.featured_artist);
    }

    #[test]
    fn new_item_does_not_reuse_previous_enrichment() {
        let mut next = crate::models::Playback::idle(100);
        next.item_id = Some("new-track".into());
        let mut previous = crate::models::Playback::idle(50);
        previous.item_id = Some("old-track".into());
        previous.palette.primary = "#ffffff".into();

        preserve_enrichment(&mut next, &previous);

        assert_ne!(next.palette, previous.palette);
        assert!(next.featured_artist.is_none());
    }

    #[test]
    fn backoff_is_bounded_and_exponential() {
        let first = backoff_delay(1);
        let later = backoff_delay(10);
        assert!(first >= std::time::Duration::from_secs(1));
        assert!(first <= std::time::Duration::from_millis(1_200));
        assert!(later >= std::time::Duration::from_secs(60));
        assert!(later <= std::time::Duration::from_secs(72));
    }
}
