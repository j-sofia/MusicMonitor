# Windows manual test matrix

Automated tests cover state transitions, negative coordinates, hysteresis, normalization, progress interpolation, token refresh decisions, OAuth state, backoff bounds, and external-link host validation. The following checks require real Windows display hardware and should be completed before a public release.

> **Status:** This is a release-validation checklist, not a record of completed certification. Leave items unchecked until the named configuration has actually been tested and record the environment in the release or pull request notes.

## Core interaction

- [ ] Select a secondary monitor and finish setup with the pointer outside it; display appears after the configured delay.
- [ ] Enter the monitor slowly from every edge; display hides and the underlying desktop receives the first click.
- [ ] Flick across a corner/shared edge; hysteresis prevents visible flicker.
- [ ] Leave, re-enter during the return delay, then leave again; the delay restarts rather than using stale elapsed time.
- [ ] Set entry dwell to 150 ms; a brief edge crossing is ignored, while a sustained entry hides the display.
- [ ] Press `Ctrl+Shift+F12` and use tray Settings while the overlay is visible, hidden, and previewing.
- [ ] Confirm the overlay never takes keyboard focus when it returns over an inactive monitor.

## Monitor geometry and DPI

- [ ] Target at 100% scale to the right of primary at 100%.
- [ ] Target at 125%, 150%, and 200% scale while primary uses a different scale.
- [ ] Target positioned left of primary (negative X coordinates).
- [ ] Target positioned above primary (negative Y coordinates).
- [ ] Change target resolution/orientation while MusicMonitor runs; bounds update without restart.
- [ ] Toggle “Cover the taskbar”; full bounds and work-area bounds are both exact.
- [ ] Disconnect/reconnect the target monitor; display remains hidden while absent and returns on a correct match.
- [ ] Reorder monitor numbering in Windows Settings and restart; identity/geometry fallback chooses the intended target.

## Windows lifecycle

- [ ] Lock/unlock Windows and sleep/wake with playback active, paused, and offline.
- [ ] Restart Windows Explorer; tray icon and tray actions recover.
- [ ] Enable startup, sign out/in, and confirm MusicMonitor starts silently without focus theft.
- [ ] Disable startup and confirm its current-user startup registration is removed.
- [ ] Launch a second instance; it reveals the first instance's Settings instead of starting a second worker.
- [ ] Quit from the tray; both windows and native workers exit.

## Spotify and network

- [ ] Complete PKCE login with the exact registered loopback URI.
- [ ] Cancel consent and retry; no credentials are stored after cancellation.
- [ ] Play tracks, paused tracks, podcast episodes, local files, advertisements, and a private session.
- [ ] Switch items rapidly; no stale art remains after idle/unsupported states.
- [ ] Play a track whose primary artist has an image; the intact artist visual appears in Gallery and Color Field and changes or clears with the track.
- [ ] Play tracks whose artist lookup is empty, forbidden, rate-limited, or offline; album art and playback polling continue normally without a broken artist panel.
- [ ] Disable/re-enable networking; offline state appears and polling recovers without user action.
- [ ] Revoke MusicMonitor in Spotify account settings; app requests reconnection and removes unusable credentials.
- [ ] Inspect requested OAuth scopes; only `user-read-currently-playing` is present.
- [ ] Confirm current metadata and artwork open Spotify through the Settings companion card.

## Visual and accessibility

- [ ] Inspect 16:9, 16:10, 21:9, portrait, 1080p, 1440p, and 4K layouts.
- [ ] Verify square artwork is fully visible and neither cropped, blurred, overlaid, recolored, nor animated.
- [ ] Verify artist imagery is fully visible, uncropped, has no metadata or controls over it, and disappears at narrow/portrait proportions instead of being squeezed.
- [ ] Verify the official Spotify logo stays legible and at least 70 CSS pixels wide.
- [ ] Test long track, artist, album, show, and episode names; full values remain available as native title tooltips in Settings/display metadata.
- [ ] Enable Windows “Animation effects” off / reduced motion; ambient and transition motion stops.
- [ ] Enable a Windows high-contrast theme; controls and metadata remain visible.
- [ ] Enable static background and each of the four themes.
- [ ] Leave an unchanged long-form episode/paused item beyond the OLED dim delay; brightness reduces without changing supplied artwork geometry.

## Packaging

- [ ] Install and uninstall the NSIS current-user package on a clean Windows 10 VM.
- [ ] Install and uninstall the MSI on a clean Windows 11 VM.
- [ ] Verify WebView2 bootstrap behavior on a Windows image without WebView2.
- [ ] Sign release installers with Authenticode and verify signature/timestamp before distribution.
