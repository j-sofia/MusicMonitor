# Security policy

## Supported versions

MusicMonitor is currently a pre-release project. Security fixes are made on the
`main` branch and, once public releases exist, on the latest release only.

## Reporting a vulnerability

Please do not open a public issue for a suspected vulnerability. Use GitHub's
private vulnerability reporting flow from the repository's **Security** tab.
Include the affected version, Windows version, impact, reproduction steps, and
any suggested mitigation. Remove access tokens, refresh tokens, OAuth callback
URLs, personal Spotify client IDs, and other private data from reports and
attachments.

The maintainers will acknowledge a report as soon as practical, validate its
impact, and coordinate a fix and disclosure timeline with the reporter. This is
a volunteer project and does not currently promise a formal response SLA.

Repository owners should enable GitHub private vulnerability reporting before
making the repository public.

## Security model

- MusicMonitor requests only `user-read-currently-playing`.
- It does not accept a Spotify client secret or implement playback controls.
- OAuth tokens are stored in Windows Credential Manager, not in repository
  files, browser storage, or plaintext settings.
- The app has no telemetry or hosted backend.

See [docs/PRIVACY.md](docs/PRIVACY.md) for the data-flow details.
