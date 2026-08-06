use std::{sync::Arc, time::Duration};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use rand::{distributions::Alphanumeric, Rng};
use reqwest::StatusCode;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tauri::Emitter;
use tauri_plugin_opener::OpenerExt;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};
use url::Url;

use crate::{
    models::{AuthState, AuthStatus, StoredTokens},
    state::AppState,
    time::unix_ms,
};

const AUTHORIZE_URL: &str = "https://accounts.spotify.com/authorize";
const TOKEN_URL: &str = "https://accounts.spotify.com/api/token";
const SCOPE: &str = "user-read-currently-playing";
const CALLBACK_HOST: &str = "127.0.0.1";
const CALLBACK_PORT: u16 = 43_819;

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: u64,
    #[serde(default)]
    scope: String,
}

pub fn begin(app: tauri::AppHandle, state: Arc<AppState>, client_id: String) -> Result<(), String> {
    let client_id = client_id.trim().to_owned();
    if client_id.len() < 16
        || !client_id
            .chars()
            .all(|character| character.is_ascii_alphanumeric())
    {
        return Err("The Spotify client ID does not look valid.".to_owned());
    }

    tauri::async_runtime::spawn(async move {
        set_auth(
            &app,
            &state,
            AuthState::Authorizing,
            Some("Waiting for authorization in your browser.".to_owned()),
        )
        .await;
        if let Err(error) = authorize(&app, &state, &client_id).await {
            set_auth(&app, &state, AuthState::Error, Some(error)).await;
        }
    });
    Ok(())
}

async fn authorize(
    app: &tauri::AppHandle,
    state: &Arc<AppState>,
    client_id: &str,
) -> Result<(), String> {
    let _guard = state.oauth_lock.lock().await;
    let redirect_uri = format!("http://{CALLBACK_HOST}:{CALLBACK_PORT}/callback");
    let listener = TcpListener::bind((CALLBACK_HOST, CALLBACK_PORT))
        .await
        .map_err(|error| {
            format!(
                "Could not listen for Spotify on {redirect_uri}: {error}. Close any other MusicMonitor instance and try again."
            )
        })?;
    let verifier = random_token(96);
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    let expected_state = random_token(40);

    let mut authorize_url = Url::parse(AUTHORIZE_URL).expect("Spotify authorize URL is static");
    authorize_url
        .query_pairs_mut()
        .append_pair("client_id", client_id)
        .append_pair("response_type", "code")
        .append_pair("redirect_uri", &redirect_uri)
        .append_pair("scope", SCOPE)
        .append_pair("code_challenge_method", "S256")
        .append_pair("code_challenge", &challenge)
        .append_pair("state", &expected_state);

    app.opener()
        .open_url(authorize_url.as_str(), None::<&str>)
        .map_err(|error| format!("Could not open the Spotify authorization page: {error}"))?;

    let (mut stream, _) = tokio::time::timeout(Duration::from_secs(180), listener.accept())
        .await
        .map_err(|_| "Spotify authorization timed out. Please try again.".to_owned())?
        .map_err(|error| format!("Could not receive the Spotify callback: {error}"))?;
    let mut request = vec![0_u8; 16_384];
    let count = tokio::time::timeout(Duration::from_secs(10), stream.read(&mut request))
        .await
        .map_err(|_| "The Spotify callback did not finish.".to_owned())?
        .map_err(|error| format!("Could not read the Spotify callback: {error}"))?;
    request.truncate(count);

    let result = parse_callback(&request, &expected_state);
    let success = result.is_ok();
    let response = callback_response(success);
    let _ = stream.write_all(response.as_bytes()).await;
    let _ = stream.shutdown().await;
    let code = result?;

    let token_response = state
        .http
        .post(TOKEN_URL)
        .form(&[
            ("client_id", client_id),
            ("grant_type", "authorization_code"),
            ("code", code.as_str()),
            ("redirect_uri", redirect_uri.as_str()),
            ("code_verifier", verifier.as_str()),
        ])
        .send()
        .await
        .map_err(|error| format!("Could not exchange the Spotify authorization: {error}"))?;
    if !token_response.status().is_success() {
        return Err(format!(
            "Spotify rejected the authorization exchange (HTTP {}). Check the client ID and exact redirect URI.",
            token_response.status().as_u16()
        ));
    }
    let response: TokenResponse = token_response
        .json()
        .await
        .map_err(|error| format!("Spotify returned an invalid token response: {error}"))?;
    let refresh_token = response
        .refresh_token
        .ok_or_else(|| "Spotify did not return a refresh token.".to_owned())?;
    let tokens = StoredTokens {
        access_token: response.access_token,
        refresh_token,
        expires_at_unix_ms: unix_ms().saturating_add(response.expires_in.saturating_mul(1_000)),
        scope: if response.scope.is_empty() {
            SCOPE.to_owned()
        } else {
            response.scope
        },
    };
    crate::secrets::write(&tokens)?;
    set_auth(
        app,
        state,
        AuthState::Connected,
        Some("Connected with read-only currently-playing access.".to_owned()),
    )
    .await;
    Ok(())
}

fn parse_callback(request: &[u8], expected_state: &str) -> Result<String, String> {
    let request = std::str::from_utf8(request)
        .map_err(|_| "Spotify sent a malformed callback.".to_owned())?;
    let target = request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .ok_or_else(|| "Spotify sent a callback without a request target.".to_owned())?;
    let callback = Url::parse(&format!("http://127.0.0.1{target}"))
        .map_err(|_| "Spotify sent an invalid callback URL.".to_owned())?;
    let parameters: std::collections::HashMap<_, _> = callback.query_pairs().into_owned().collect();
    if let Some(error) = parameters.get("error") {
        return Err(if error == "access_denied" {
            "Spotify authorization was cancelled.".to_owned()
        } else {
            "Spotify could not authorize MusicMonitor.".to_owned()
        });
    }
    if parameters.get("state").map(String::as_str) != Some(expected_state) {
        return Err(
            "Spotify authorization state did not match. No credentials were stored.".to_owned(),
        );
    }
    parameters
        .get("code")
        .cloned()
        .ok_or_else(|| "Spotify did not return an authorization code.".to_owned())
}

fn callback_response(success: bool) -> String {
    let (title, message) = if success {
        (
            "MusicMonitor connected",
            "You can close this tab and return to MusicMonitor Settings.",
        )
    } else {
        (
            "MusicMonitor could not connect",
            "Return to MusicMonitor Settings for details, then try again.",
        )
    };
    let body = format!(
        "<!doctype html><meta charset=utf-8><meta name=viewport content='width=device-width'><title>{title}</title><style>html{{color-scheme:dark;font-family:Segoe UI,sans-serif;background:#0b0d11;color:#f4f5f7}}body{{display:grid;place-items:center;min-height:100vh;margin:0}}main{{max-width:520px;padding:48px}}p{{color:#9da4af;line-height:1.6}}</style><main><h1>{title}</h1><p>{message}</p></main>"
    );
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\nCache-Control: no-store\r\n\r\n{}",
        body.len(),
        body
    )
}

fn random_token(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

pub async fn access_token(
    app: &tauri::AppHandle,
    state: &Arc<AppState>,
    client_id: &str,
    force_refresh: bool,
) -> Result<String, TokenError> {
    let Some(mut tokens) = crate::secrets::read().map_err(TokenError::Storage)? else {
        return Err(TokenError::Missing);
    };
    if !force_refresh && !tokens.needs_refresh(unix_ms()) {
        return Ok(tokens.access_token);
    }

    let response = state
        .http
        .post(TOKEN_URL)
        .form(&[
            ("client_id", client_id),
            ("grant_type", "refresh_token"),
            ("refresh_token", tokens.refresh_token.as_str()),
        ])
        .send()
        .await
        .map_err(|error| TokenError::Network(error.to_string()))?;
    if response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::UNAUTHORIZED
    {
        let _ = crate::secrets::delete();
        set_auth(
            app,
            state,
            AuthState::Expired,
            Some("Spotify access was revoked or expired. Reconnect in Settings.".to_owned()),
        )
        .await;
        return Err(TokenError::Expired);
    }
    if !response.status().is_success() {
        return Err(TokenError::Network(format!(
            "Spotify token refresh returned HTTP {}",
            response.status().as_u16()
        )));
    }
    let refreshed: TokenResponse = response
        .json()
        .await
        .map_err(|error| TokenError::Network(error.to_string()))?;
    tokens.access_token = refreshed.access_token;
    if let Some(refresh_token) = refreshed.refresh_token {
        tokens.refresh_token = refresh_token;
    }
    tokens.expires_at_unix_ms =
        unix_ms().saturating_add(refreshed.expires_in.saturating_mul(1_000));
    if !refreshed.scope.is_empty() {
        tokens.scope = refreshed.scope;
    }
    crate::secrets::write(&tokens).map_err(TokenError::Storage)?;
    Ok(tokens.access_token)
}

pub async fn set_auth(
    app: &tauri::AppHandle,
    state: &Arc<AppState>,
    auth_state: AuthState,
    message: Option<String>,
) {
    let status = AuthStatus {
        state: auth_state,
        message,
    };
    *state.auth.write().await = status.clone();
    let _ = app.emit("auth://status", status);
}

#[derive(Debug)]
pub enum TokenError {
    Missing,
    Expired,
    Storage(String),
    Network(String),
}

#[cfg(test)]
mod tests {
    use super::parse_callback;

    #[test]
    fn validates_oauth_state() {
        let request = b"GET /callback?code=abc&state=correct HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n";
        assert_eq!(parse_callback(request, "correct").unwrap(), "abc");
        assert!(parse_callback(request, "wrong").is_err());
    }

    #[test]
    fn treats_access_denial_as_normal_error() {
        let request = b"GET /callback?error=access_denied&state=correct HTTP/1.1\r\n\r\n";
        assert!(parse_callback(request, "correct")
            .unwrap_err()
            .contains("cancelled"));
    }
}
