## Summary

<!-- What changed, and why does it belong in a read-only ambient display? -->

## Validation

- [ ] `pnpm format:check`
- [ ] `pnpm lint`
- [ ] `pnpm test`
- [ ] `pnpm build`
- [ ] `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check`
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml --all-targets`
- [ ] `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`
- [ ] `cargo check --manifest-path src-tauri/Cargo.toml --all-targets`

## Native/manual coverage

<!-- List Windows version, monitor geometry, DPI scales, and applicable MANUAL_TESTS checks. Use "Not applicable" only for docs-only changes. -->

- [ ] Relevant `docs/MANUAL_TESTS.md` checks completed or not applicable
- [ ] MSI rebuilt with `pnpm tauri build --bundles msi` for an implementation change
- [ ] MSI artifact verified under `src-tauri/target/release/bundle/msi/`

## Project boundaries

- [ ] No playback-control scope, endpoint, or UI was added
- [ ] No token, client secret, personal client ID, full OAuth callback, or signing material is present
- [ ] Supplied Spotify artwork remains intact
- [ ] Version is synchronized in all four manifests for an implementation change
- [ ] User-facing changes are documented
