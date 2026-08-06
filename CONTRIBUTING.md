# Contributing to MusicMonitor

Thanks for helping improve MusicMonitor. The project is intentionally narrow:
it is a read-only ambient Spotify display that disappears when its monitor is
in use. Changes should protect that interaction above everything else.

## Before proposing a change

Please search existing issues first. For a significant behavior or architecture
change, open an issue before investing in an implementation so scope and Windows
test expectations can be agreed on.

The following are deliberate non-goals:

- playback, queue, volume, seeking, or device-transfer controls;
- embedded playback, audio capture, analysis, or beat synchronization;
- lyrics scraping, Spotify Canvas scraping, or private APIs;
- a hosted backend or telemetry for the personal-app MVP.

## Development setup

You need Windows 10 or 11, Node.js 22.12+, pnpm 11+, Rust 1.88+ with the MSVC toolchain,
WebView2, and Visual Studio 2022 Build Tools with the **Desktop development with
C++** workload.

```powershell
pnpm install --frozen-lockfile
pnpm tauri dev
```

The web UI can be previewed without Tauri at `http://localhost:1420/?settings`,
but native monitor, tray, credential, OAuth, and window behavior still requires
the Tauri application.

## Quality expectations

- Keep TypeScript strict and parse external responses defensively.
- Keep Spotify transport objects and tokens out of React.
- Isolate and document Win32 `unsafe` code behind safe Rust interfaces.
- Add focused tests for state transitions and data-boundary logic.
- Do not log tokens, complete OAuth callback URLs, personal client IDs, or
  sensitive Windows data.
- Preserve supplied Spotify artwork and the app's read-only permission scope.

Run all automated checks before opening a pull request:

```powershell
pnpm install --frozen-lockfile
pnpm format:check
pnpm lint
pnpm test
pnpm build
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo test --manifest-path src-tauri/Cargo.toml --all-targets
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
```

Changes to native window behavior also need relevant checks from
[docs/MANUAL_TESTS.md](docs/MANUAL_TESTS.md). Record what hardware and DPI
configuration you tested in the pull request.

## Versions and installers

For implementation changes, increment the patch version in all four places:

- `package.json`
- `src-tauri/Cargo.toml`
- the MusicMonitor package entry in `src-tauri/Cargo.lock`
- `src-tauri/tauri.conf.json`

Then build and verify the MSI:

```powershell
pnpm tauri build --bundles msi
Get-ChildItem src-tauri/target/release/bundle/msi/*.msi
```

Local installers are unsigned development artifacts. Do not attach an unsigned
installer to a public release.

## Pull requests

Keep pull requests focused. Explain the user-visible behavior, tests performed,
manual Windows coverage, privacy or scope impact, and any known limitations.
Screenshots are welcome for visual changes, but they do not replace native
interaction testing.

By contributing, you agree that your contribution is licensed under the
[MIT License](LICENSE) and that you will follow the
[Code of Conduct](CODE_OF_CONDUCT.md).
