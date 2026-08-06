# Architecture

## Process boundary

The Tauri/Rust process owns behavior that must continue when the webview is hidden:

- monitor enumeration and physical/work-area bounds;
- global `GetCursorPos` polling;
- overlay visibility and placement;
- tray, global shortcut, single-instance, and autostart integration;
- OAuth loopback listener and PKCE exchange;
- Windows Credential Manager access;
- Spotify HTTP polling, response normalization, retry policy, and artwork palette extraction.

React receives only normalized playback and settings models over typed Tauri commands/events. Spotify transport objects and tokens never enter React, browser storage, or the webview console.

## Overlay state machine

The pure state machine in `src-tauri/src/monitor.rs` uses four phases:

```text
Visible -> EntryPending -> Hidden -> ReturnPending -> Visible
```

With a zero entry dwell, `Visible` transitions directly to `Hidden`. Entry checks use bounds contracted by the configured hysteresis; exit checks use bounds expanded by that amount. A brief noisy coordinate at a shared monitor edge therefore cannot produce a hide/show loop.

The worker refreshes monitor geometry once per second and cursor position every 16 ms. Monitor resolution, position, scale, taskbar work area, disconnect, and reconnect changes are reconciled without relying on DOM events. The display is non-focusable and click-through, so it cannot intercept the first click or steal keyboard focus.

## Monitor identity

Tauri's Windows monitor name is the primary persisted identity. The saved friendly name is the second match. Physical position and size are the final nearest-geometry fallback. This is best-effort because Windows and some display adapters do not expose an immutable identity through the high-level window API.

## Spotify data flow

1. Settings saves the non-secret client ID.
2. Rust creates fresh PKCE verifier/challenge and OAuth state values.
3. The fixed `http://127.0.0.1:43819/callback` loopback listener is bound before the authorization page opens. Using an explicit port matches Spotify's Developer Dashboard validation while keeping the callback local to the Windows machine.
4. The callback state is checked before the code is exchanged.
5. Tokens are written as one credential blob to Windows Credential Manager.
6. The native worker requests `/v1/me/player/currently-playing` with `user-read-currently-playing`. For tracks with an artist ID, it may also request the public read-only `/v1/artists/{id}` catalog endpoint asynchronously so the playback transition never waits on enrichment; successful artist visuals are cached in memory for 24 hours and failed/empty lookups for five minutes.
7. Defensive transport structs are normalized into `Playback` before emission, including an optional primary-artist visual with its Spotify link.
8. React locally interpolates progress from `progressMs`, `durationMs`, `is_playing`/status, and `observedAt`.

Artwork bytes fetched for palette extraction are bounded to 10 MiB, decoded into a 48×48 thumbnail, reduced to a small palette, and then dropped. Supplied album and artist image URLs are rendered separately with `object-fit: contain`; they are not cropped, modified, or composited into the background. Artist imagery appears only in compositions with enough space. The click-through display is intentionally non-interactive; the current item can be opened in Spotify from Settings.

Spotify Canvas and music-video assets are intentionally unavailable: the supported Web API does not expose them, and MusicMonitor does not use private or reverse-engineered endpoints.

## Failure behavior

- `204` or no item: idle state with no retained artwork.
- Advertisement/private/unknown item: unsupported state with no retained artwork.
- Paused: current item retained with an explicit paused presentation.
- Network or 5xx: offline/retrying state and exponential backoff with jitter.
- `429`: honor `Retry-After` up to five minutes.
- `401`: force one refresh and retry; revoked refresh fails closed into auth-required.
- Target monitor disconnected: display hides and the worker continues looking for an identity/geometry match.
- Explorer restart: Tauri's tray implementation recreates its notification icon; the global shortcut remains an independent escape hatch.

## Security boundaries

- No Spotify client secret is accepted or stored.
- No playback-control scopes or endpoints exist in the code.
- Credentials are isolated in `src-tauri/src/secrets.rs`; its documented `unsafe` blocks wrap only Win32 Credential Manager calls.
- Win32 cursor/monitor `unsafe` is isolated in `src-tauri/src/monitor.rs` behind safe data types.
- External URLs require HTTPS and an exact native host allowlist match.
- The production webview has a restrictive CSP and no direct arbitrary network command.
- No telemetry or remote diagnostics are present.
