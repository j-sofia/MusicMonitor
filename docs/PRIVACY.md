# Privacy and data flow

MusicMonitor is local-first desktop software. It has no analytics, telemetry,
advertising, crash-upload service, or hosted backend.

## Data stored on the computer

| Data                                  | Location                                                                              | Purpose                                                                   |
| ------------------------------------- | ------------------------------------------------------------------------------------- | ------------------------------------------------------------------------- |
| Spotify access and refresh tokens     | Windows Credential Manager target `MusicMonitor/SpotifyOAuth`                         | Read the current Spotify item without repeated login                      |
| App preferences and Spotify client ID | Tauri app-data directory, normally `%APPDATA%\com.musicmonitor.desktop\settings.json` | Remember monitor, display, startup, and public OAuth-client configuration |
| Artwork-derived colors                | Memory only                                                                           | Render the current generated background                                   |
| Artist-image lookup results           | Bounded in-memory cache                                                               | Avoid repeated catalog lookups during a session                           |

The Spotify client ID identifies a public desktop OAuth application and is not
a password. MusicMonitor never accepts or stores a Spotify client secret.

## Network requests

MusicMonitor connects only to:

- `accounts.spotify.com` for OAuth authorization and token exchange;
- `api.spotify.com` for current playback and optional public artist metadata;
- artwork/image URLs returned by Spotify, normally under Spotify's CDN domains.

Authorization opens in the system browser. The callback returns to a temporary
listener on `http://127.0.0.1:43819/callback`; it is not sent to a MusicMonitor
server.

## Permissions

The app requests only `user-read-currently-playing`. It does not request or
implement permissions for pausing, skipping, seeking, volume, queue changes,
device transfer, library modification, or account modification.

## Removing data

Use **Disconnect Spotify** in Settings to remove MusicMonitor's OAuth credential.
Uninstalling the application may not remove every per-user preference on all
installer/Windows combinations; `%APPDATA%\com.musicmonitor.desktop` can be
removed manually after MusicMonitor is closed.

You can also revoke the application from your Spotify account's connected-apps
page. A revoked token will no longer authorize MusicMonitor.

## Logs and reports

MusicMonitor does not intentionally log tokens or complete OAuth callback URLs.
Before sharing diagnostic text, screenshots, or issue attachments, remove:

- access and refresh tokens;
- OAuth authorization codes and complete callback URLs;
- personal Spotify client IDs;
- Windows usernames and unrelated file paths.

See [SECURITY.md](../SECURITY.md) for private vulnerability reporting.
