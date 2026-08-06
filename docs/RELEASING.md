# Release process

This checklist is for maintainers. MusicMonitor is not ready for a public binary
release until the relevant hardware matrix is complete and installers are
signed.

## 1. Prepare the release

- Choose a Semantic Versioning release number and update `CHANGELOG.md`.
- Synchronize the version in `package.json`, `src-tauri/Cargo.toml`, the root
  MusicMonitor entry in `src-tauri/Cargo.lock`, and
  `src-tauri/tauri.conf.json`.
- Update the About-screen version in `src/SettingsApp.tsx` until it is sourced
  from build metadata.
- Confirm no personal client ID, token, OAuth callback, log, credential export,
  or signing material is present in the Git diff.
- Verify the Tauri identifier, publisher, copyright, and installer strategy.

## 2. Run automated validation

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

Review dependency advisories with the current pnpm and Rust security-audit tools.
Do not treat an unavailable advisory service as a passing result.

## 3. Run Windows validation

Complete and record the applicable checks in
[MANUAL_TESTS.md](MANUAL_TESTS.md), including at least two DPI scales and a
target monitor on both sides of the primary display. Run clean-VM install and
uninstall tests for any installer format being published.

## 4. Build, sign, and verify

```powershell
pnpm tauri build --bundles msi
Get-ChildItem src-tauri/target/release/bundle/msi/*.msi
```

The unsigned MSI is expected under `src-tauri/target/release/bundle/msi/`.
Authenticode-sign the release artifact using protected signing infrastructure;
never put a certificate or password in the repository. Verify the signature and
timestamp on a clean Windows machine, then generate a SHA-256 checksum:

```powershell
Get-AuthenticodeSignature .\path\to\MusicMonitor.msi
Get-FileHash .\path\to\MusicMonitor.msi -Algorithm SHA256
```

## 5. Publish

- Create an annotated `vX.Y.Z` tag from the reviewed release commit.
- Draft GitHub release notes from `CHANGELOG.md`.
- Attach only the signed, verified MSI and its checksum.
- State the supported Windows versions, pre-release/stability status, Spotify
  setup requirement, and known limitations.
- Install the uploaded artifact once more; do not rely only on the local copy.
