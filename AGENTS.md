# MusicMonitor

## Product intent

MusicMonitor is a Windows-first, read-only Spotify companion for a dedicated second monitor. It turns that monitor into a polished now-playing display while the monitor is idle, then gets completely out of the way when the user moves the mouse onto that monitor.

The core promise is:

- When the pointer is elsewhere, the selected monitor shows the current Spotify item full-screen.
- When the pointer enters that monitor, MusicMonitor disappears and the desktop/apps underneath are immediately usable.
- When the pointer leaves that monitor, the display returns after a short debounce.
- MusicMonitor never pauses, resumes, seeks, skips, changes volume, transfers playback, or otherwise controls Spotify.

Preserve that narrow focus. This is an ambient display, not another music player.

## Current repository state

This repository began as a greenfield project. Before making architectural changes, inspect the current tree, package manifests, and Git status; later sessions may have implemented part of this plan.

## Recommended architecture

Unless the repository already establishes another direction, use:

- Tauri 2 for the desktop shell and packaging.
- Rust for global cursor tracking, monitor enumeration, window placement, autostart/system-tray integration, and secure token persistence.
- React + TypeScript + Vite for the display and settings UI.
- CSS transforms/transitions or a lightweight canvas layer for restrained ambient animation. Do not add a large rendering/game dependency without a measured need.
- A small typed service boundary between Spotify responses, the normalized playback model, and UI state.

Why this split: the overlay must continue tracking the global pointer after its window becomes hidden or mouse-transparent. That behavior belongs in the native process, not in DOM `mouseenter` handlers.

Do not introduce a hosted backend for the personal-app MVP. Spotify Authorization Code with PKCE is designed for desktop/public clients and avoids embedding a client secret.

## Window and monitor behavior

Implement the app as a borderless window sized to the chosen monitor's physical bounds. The native layer owns a state machine similar to:

`DISPLAY_VISIBLE -> POINTER_ENTERED -> DISPLAY_HIDDEN -> POINTER_LEFT -> RETURN_DELAY -> DISPLAY_VISIBLE`

Behavioral requirements:

- Remember the target monitor using a stable best-effort identity plus geometry fallback; monitor ordering can change between boots.
- Correctly handle negative virtual-desktop coordinates, mixed DPI scaling, resolution changes, monitor disconnect/reconnect, sleep/wake, and Windows Explorer restarts.
- Poll or subscribe to the global cursor position from the native layer. A hidden window cannot receive DOM pointer-leave events.
- On pointer entry, fade quickly if it does not delay usability, then hide the window or make it fully transparent and input-transparent. The desktop underneath must receive the first intentional click.
- Keep tracking while hidden. Restore only after the pointer has left the target monitor and remained out for a configurable debounce (initial default: 500 ms).
- Add small edge hysteresis so one noisy coordinate at a monitor boundary does not flicker the display.
- The display must not steal keyboard focus when it appears. Prefer a non-activating show operation where Windows/Tauri support permits it.
- Default to covering the full selected monitor, including the taskbar, while visible. Make this configurable only if real use shows it is needed.
- Provide a tray menu and a settings window so the user can change the target monitor or quit while the main display is hidden.
- Include a safety escape hatch during development, such as a tray action and a documented keyboard shortcut, so a faulty overlay cannot trap the screen.

Do not equate "always on top" with the product requirement without testing it. The display should be present while the monitor is idle, but must never fight the user, steal focus, or reappear over active work while the pointer remains on that monitor.

## Spotify integration

Use Spotify Web API OAuth Authorization Code with PKCE. Never ship or request a Spotify client secret in this desktop app.

Initial permission scope should be only:

- `user-read-currently-playing` for `GET /v1/me/player/currently-playing`.

Use `user-read-playback-state` and `GET /v1/me/player` only if the implementation truly needs fields not provided by the currently-playing endpoint. Do not request playback-control scopes.

Authentication requirements:

- Open authorization in the system browser.
- Use a loopback redirect such as `http://127.0.0.1:<port>` if supported by the registered Spotify app configuration; the registered and requested redirect URI must match exactly.
- Generate and validate PKCE verifier/challenge and OAuth `state` for every login.
- Refresh tokens before expiry and recover cleanly from revoked access.
- Store refresh/access tokens using Windows Credential Manager or an equivalent OS-protected secret store, not plaintext config, source files, logs, or browser local storage.
- The Spotify client ID is configuration, not a secret. Support it through a documented local config or first-run setup; never commit a personal credential.

Polling guidance:

- Start with a 2-3 second polling interval while playback is active.
- Poll more slowly when paused/no content, and pause network work when offline.
- Update the progress bar locally between responses using `progress_ms`, `duration_ms`, `is_playing`, and the response timestamp.
- On track changes, transition immediately and cache only the small number of images needed for smooth rendering.
- Honor HTTP 429 and `Retry-After`; use exponential backoff with jitter for transient failures.
- Treat 204/no item, private sessions, local files, ads, episodes, null fields, unknown future item types, and network/auth failures as normal states rather than crashes.

As of 2026-08-04, Spotify's official documentation still lists both currently-playing and playback-state endpoints. Development Mode requires the app owner to have Premium and is limited to five authorized users for new apps. Re-check Spotify's current official docs before relying on these details because platform access rules can change.

Official references:

- https://developer.spotify.com/documentation/web-api/reference/get-the-users-currently-playing-track
- https://developer.spotify.com/documentation/web-api/concepts/authorization
- https://developer.spotify.com/documentation/web-api/tutorials/code-pkce-flow
- https://developer.spotify.com/documentation/web-api/concepts/rate-limits
- https://developer.spotify.com/documentation/design
- https://developer.spotify.com/documentation/web-api/tutorials/february-2026-migration-guide

## Visual direction

The preferred first concept is "gallery mode":

- Large, uncropped square album/episode art with subtle rounded corners.
- Track title as the primary typographic element; artist and album/show secondary.
- A restrained progress indicator is allowed because it is display-only.
- Spotify attribution and a link/open-in-Spotify affordance should be available in the companion/settings experience and displayed where required by Spotify's current guidelines.
- A background built from extracted dominant artwork colors, layered gradients, grain, soft procedural light, and slow ambient motion.
- Smooth crossfades/color interpolation when the item changes.

Alternative themes can come later without changing the data layer:

1. Gallery: centered or offset artwork, editorial typography, generous negative space.
2. Vinyl room: artwork treated as a physical sleeve beside a subtle procedural record motif; do not alter the supplied artwork.
3. Color field: smaller intact artwork with large generative gradients derived from its palette.
4. Minimal OLED: near-black background, compact metadata, and low-motion presentation.

Theme rules:

- Keep Spotify-supplied artwork intact. Do not crop, blur, distort, animate, recolor, or put text/graphics over it.
- It is acceptable under Spotify's published design guidance to extract artwork colors for a separate background. Prefer generated gradients/shapes over a blurred enlargement of the cover.
- Metadata must remain legible and unmodified.
- Include Spotify attribution and link Spotify metadata/artwork back to Spotify as required by the current Spotify design guidelines.
- Do not scrape or call private Spotify endpoints for Spotify Canvas animations. The documented now-playing API does not expose the consumer-app Canvas asset. Treat Canvas as unavailable unless Spotify publishes a supported API for it.
- Do not make visuals audio-reactive or beat-synchronized. MusicMonitor does not ingest or analyze audio.
- Respect Windows reduced-motion and high-contrast preferences. Provide a static-background option.
- Avoid burn-in: allow slight layout drift/dimming after long unchanged playback, especially for OLED displays, without moving or modifying the artwork itself in a way that violates content guidelines.

## Normalized playback model

Keep Spotify transport objects out of React components. Normalize into a model along these lines (adapt names to existing code):

- `status`: `playing | paused | idle | offline | authRequired | error`
- `itemType`: `track | episode | unknown`
- `itemId` and `uri`
- `title`
- `creators[]` (artists, show/author as applicable)
- `collectionTitle` (album or show)
- `artwork[]` with URL, width, and height
- `durationMs`, `progressMs`, and `observedAt`
- `isExplicit`
- `spotifyUrl`

UI code should render explicit states for:

- Playing
- Paused
- Nothing currently playing
- Login required/session expired
- Offline/retrying
- Unsupported/private item
- First-run setup

Never leave stale artwork presented as if it were still playing. A paused display may retain the current item but must visibly communicate the paused state without adding controls.

## Non-goals and hard boundaries

- No play, pause, next, previous, seek, shuffle, repeat, volume, queue modification, or playback transfer.
- No embedded Spotify playback and no Web Playback SDK.
- No audio capture, audio analysis, beat detection, lyrics scraping, or visual synchronization to the recording.
- No downloading Spotify content beyond normal, bounded image caching needed for display.
- No private/reverse-engineered Spotify APIs.
- No account passwords handled by MusicMonitor.
- No telemetry in the MVP. If diagnostics are added later, make them local-first and never log tokens or full OAuth callback URLs.
- Do not advertise Spotify affiliation or copy Spotify's first-party interface. This is a companion display using permitted metadata.

## Suggested delivery phases

### Phase 1: interaction proof

- Tauri shell, monitor picker, borderless display, tray menu.
- Global pointer detection with hide-on-enter/show-on-leave.
- Use mocked playback fixtures; prove mixed-DPI and multi-monitor behavior before Spotify work.

### Phase 2: Spotify data

- PKCE login, secure token storage, refresh, typed API client.
- Poll and normalize current playback.
- Robust empty/error/offline states.

### Phase 3: visual MVP

- Gallery theme, palette extraction, generated background, metadata, local progress interpolation, transitions.
- Reduced motion and basic accessibility.

### Phase 4: resilience and packaging

- Sleep/wake, reconnect, monitor hot-plug, startup behavior, installer/signing plan, logging, and update strategy.
- Test on at least two different DPI scales and with the target monitor both left and right of the primary.

Do not start with elaborate animation before the window-hiding interaction is reliable. The pointer/monitor behavior is the defining feature and highest technical risk.

## Quality bar

For every implementation change:

- Preserve existing user changes and inspect `git status` before editing.
- Keep TypeScript strict; avoid `any` at external boundaries. Validate API payloads or parse defensively.
- Keep native commands narrow and typed. Document any Windows API `unsafe` code and isolate it behind a safe interface.
- Unit-test playback normalization, progress interpolation, backoff, token expiry decisions, and monitor-boundary state transitions.
- Add integration/manual checks for pointer transitions, focus behavior, taskbar coverage, DPI, sleep/wake, and monitor hot-plug.
- Run the relevant formatter, linter, typecheck, tests, and Tauri build/check before handing off.
- For every implementation change, increment the patch version consistently in `package.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, and `src-tauri/tauri.conf.json` before building the installer.
- After every new feature request, rebuild the Windows MSI (`pnpm tauri build --bundles msi`) and verify that the installer artifact was produced before handing off.
- Never commit generated secrets, OAuth tokens, personal client IDs, or logs containing authorization data.

## Decisions that still need user validation

Do not block initial scaffolding on these, but surface them before polishing behavior:

- Windows-only versus eventual macOS/Linux support.
- Whether the overlay should return immediately, after 500 ms, or only after a longer idle timeout once the pointer leaves.
- Whether accidental edge entry should hide instantly or require a short dwell (for example 100-150 ms).
- Preferred default composition: centered art, left-art/right-type, or full editorial layout.
- Startup with Windows and whether the app starts minimized to tray.
- Behavior while paused: retain a dimmed card, show a paused state, or switch to an idle clock.
