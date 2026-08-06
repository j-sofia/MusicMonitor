# MusicMonitor

[![Status: pre-release alpha](https://img.shields.io/badge/status-pre--release_alpha-b06a3c)](CHANGELOG.md)
[![Platform: Windows](https://img.shields.io/badge/platform-Windows-0078d4)](#requirements)
[![License: MIT](https://img.shields.io/badge/license-MIT-7b68a8)](LICENSE)

MusicMonitor turns a dedicated Windows monitor into a polished, full-screen
Spotify now-playing display—and gets completely out of the way when you use that
monitor.

Move the pointer onto the selected display and MusicMonitor disappears so the
desktop receives your first click. Move away and it returns after a short
debounce. It is deliberately **not a player**: it cannot pause, skip, seek,
change volume, edit the queue, or transfer playback.

> **Project status:** pre-release Windows alpha. The core behavior is
> implemented and automated checks pass, but the real-hardware validation in
> [docs/MANUAL_TESTS.md](docs/MANUAL_TESTS.md) is not yet recorded as complete.
> There are currently no signed public installers; build from source.

## The interaction

```text
Pointer elsewhere                   Pointer enters target monitor
┌──────────────────────────┐        ┌──────────────────────────┐
│ Full-screen now playing  │  ───▶  │ Desktop/apps underneath │
│ artwork + metadata       │        │ immediately usable      │
└──────────────────────────┘        └──────────────────────────┘
                 ◀── pointer leaves + return delay ──
```

The native process tracks the global cursor even while the display window is
hidden. This avoids relying on webview pointer events that stop when a window is
invisible or input-transparent.

## Highlights

- Native global pointer tracking with configurable entry dwell, return delay,
  and edge hysteresis.
- Best-effort stable monitor identity plus geometry fallback when Windows
  monitor ordering changes.
- Physical/work-area placement logic for negative virtual-desktop coordinates,
  mixed scale factors, resolution changes, and monitor reconnects.
- Borderless, non-focusable, click-through overlay; tray controls and
  `Ctrl+Shift+F12` provide safety escape hatches.
- Spotify Authorization Code with PKCE, a local IPv4 loopback callback, fresh
  OAuth state per login, and only `user-read-currently-playing`.
- Tokens in Windows Credential Manager; preferences and the public client ID in
  local app data; no telemetry or hosted backend.
- Defensive handling for tracks, episodes, paused/idle/offline states, local or
  private items, advertisements, missing fields, revocation, and rate limits.
- Gallery, Minimal OLED, Color Field, and Vinyl Room themes with reduced-motion,
  high-contrast, static-background, and burn-in-conscious options.
- Intact Spotify artwork: no cropping, blurring, recoloring, overlays, or
  animation of supplied artwork.

Hardware-dependent focus, DPI, hot-plug, sleep/wake, and first-click behavior
must still be validated on each supported configuration before a public release.

## Non-goals

MusicMonitor intentionally has no:

- play, pause, previous, next, seek, shuffle, repeat, volume, queue, or playback
  transfer controls;
- embedded Spotify playback or Web Playback SDK;
- audio capture, analysis, beat detection, lyrics scraping, or music sync;
- private/reverse-engineered Spotify APIs or Canvas scraping;
- account passwords, client secrets, analytics, or telemetry.

That boundary is the product: MusicMonitor is an ambient companion, not another
music player.

## Requirements

- Windows 10 or 11
- [WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/)
  (normally present on Windows 11)
- Node.js 22.12 or newer
- pnpm 11
- Rust MSVC toolchain
- Visual Studio 2022 Build Tools with **Desktop development with C++**

Spotify currently requires each installation to use a personal Spotify
developer app and client ID. A client ID is public configuration. **Never enter
or commit a Spotify client secret.** Check Spotify's current
[authorization](https://developer.spotify.com/documentation/web-api/concepts/authorization)
and [quota-mode](https://developer.spotify.com/documentation/web-api/concepts/quota-modes)
documentation because platform rules can change.

## Build from source

```powershell
git clone <your-fork-or-repository-url>
cd MusicMonitor
pnpm install --frozen-lockfile
pnpm tauri dev
```

The standalone browser preview at `http://localhost:1420/?settings` demonstrates
the settings UI, but native cursor, monitor, tray, Credential Manager, OAuth, and
window behavior require `pnpm tauri dev` on Windows.

Build an unsigned local MSI with:

```powershell
pnpm tauri build --bundles msi
Get-ChildItem src-tauri/target/release/bundle/msi/*.msi
```

The artifact is written under `src-tauri/target/release/bundle/msi/`. Unsigned
local builds may trigger Microsoft SmartScreen; do not bypass warnings for an
artifact you do not trust. Public distribution should use Authenticode signing
with credentials stored outside this repository.

## Spotify first run

1. Open MusicMonitor Settings.
2. Create an app in the
   [Spotify Developer Dashboard](https://developer.spotify.com/dashboard).
3. Add `http://127.0.0.1:43819/callback` as a redirect URI and save. The requested
   and registered URI must match exactly; do not substitute `localhost`, HTTPS,
   another port, or a trailing slash.
4. Paste the app's client ID into MusicMonitor. Do not create or paste a client
   secret.
5. Choose the target monitor, connect Spotify in the system browser, and finish
   setup.

Spotify Development Mode account, Premium, allowlist, and user-limit rules can
change. Consult Spotify's official documentation when configuring the app.

## Everyday use

- Move the pointer onto the target monitor to reveal the desktop.
- Move it away and leave it out for the configured delay (500 ms by default) to
  restore the display.
- Left-click the tray icon or press `Ctrl+Shift+F12` to reveal Settings.
- Right-click the tray icon to preview, hide, open Settings, or quit.
- Open the current Spotify item from the companion card in Settings. The
  full-screen overlay is intentionally non-interactive.

## Architecture

MusicMonitor uses Tauri 2 with a Rust native process and a React/TypeScript/Vite
webview.

| Area                                                                        | Owner            |
| --------------------------------------------------------------------------- | ---------------- |
| Cursor polling, monitor geometry, window state, tray, shortcut, autostart   | Rust/Tauri       |
| PKCE, loopback callback, Credential Manager, Spotify polling, normalization | Rust             |
| Display and settings presentation, local progress interpolation             | React/TypeScript |
| Ambient color fields, transitions, reduced motion, high contrast            | CSS              |

Spotify transport objects and OAuth tokens never enter React. The webview
receives only normalized playback and settings models over typed Tauri
commands/events. See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for details.

## Validation

Run frontend checks:

```powershell
pnpm format:check
pnpm lint
pnpm test
pnpm build
```

Run native checks:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo test --manifest-path src-tauri/Cargo.toml --all-targets
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
```

GitHub Actions runs these checks on Windows for pushes and pull requests.
Native behavior changes also require the applicable real-hardware checks in
[docs/MANUAL_TESTS.md](docs/MANUAL_TESTS.md).

## Privacy and security

- Settings: Tauri app-data directory, normally
  `%APPDATA%\com.musicmonitor.desktop\settings.json`.
- OAuth tokens: Windows Credential Manager target
  `MusicMonitor/SpotifyOAuth`.
- Network destinations: Spotify Accounts, Spotify Web API, and image URLs
  returned by Spotify.
- External links: HTTPS with an exact native host allowlist.
- Telemetry: none.

Read [docs/PRIVACY.md](docs/PRIVACY.md) for the complete data flow and
[SECURITY.md](SECURITY.md) before reporting a vulnerability. Never post tokens,
complete OAuth callback URLs, personal client IDs, or credential exports in a
public issue.

## Project docs

- [Architecture](docs/ARCHITECTURE.md)
- [Manual Windows test matrix](docs/MANUAL_TESTS.md)
- [Troubleshooting](docs/TROUBLESHOOTING.md)
- [Privacy and data flow](docs/PRIVACY.md)
- [Release process](docs/RELEASING.md)
- [Changelog](CHANGELOG.md)

## Contributing

Issues and focused pull requests are welcome. Read
[CONTRIBUTING.md](CONTRIBUTING.md), [SUPPORT.md](SUPPORT.md), and the
[Code of Conduct](CODE_OF_CONDUCT.md) first. Feature proposals must preserve the
read-only scope and the hide-on-entry interaction.

## License and Spotify notice

MusicMonitor source code is available under the [MIT License](LICENSE). The
official Spotify logo asset and Spotify-supplied metadata/artwork are excluded
from that license and remain subject to their owners' rights; see
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

MusicMonitor is independent and is not affiliated with, sponsored by, or
endorsed by Spotify.
