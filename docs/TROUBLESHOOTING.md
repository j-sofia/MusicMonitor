# Troubleshooting

## Spotify will not connect

### Redirect URI mismatch

The redirect URI in the Spotify Developer Dashboard must be exactly:

```text
http://127.0.0.1:43819/callback
```

Do not use `localhost`, HTTPS, a different port, or a trailing slash. Save the
Spotify app settings before trying again.

### Port 43819 is unavailable

Close other MusicMonitor instances and retry. If another program owns the port,
identify it from an elevated PowerShell window:

```powershell
Get-NetTCPConnection -LocalPort 43819 -ErrorAction SilentlyContinue
```

Do not terminate an unfamiliar process solely because it uses the port.

### Spotify returns 403 or the account is not authorized

For Spotify Development Mode, confirm that the app owner meets Spotify's current
account requirements and that the signing-in user is included in the app's user
allowlist. Spotify platform rules can change; check the current official Web API
documentation.

### Reconnect after revocation

Open Settings, disconnect Spotify, then connect again. This clears the unusable
credential before starting a fresh PKCE authorization.

## Nothing is displayed

- Confirm setup is complete and the intended monitor is selected.
- Move the pointer fully off the target monitor and wait through the configured
  return delay.
- Use the tray **Preview display** action.
- Press `Ctrl+Shift+F12` to reopen Settings.
- Confirm Spotify is playing a supported track or episode. Idle, private,
  advertisement, and unsupported items intentionally do not retain stale art.

If a monitor was disconnected or its topology changed, reconnect it and reselect
the target in Settings. Monitor identity is best-effort because Windows monitor
names can change after driver or docking changes.

## The safety shortcut does not work

Another application may already own the shortcut. Use the tray icon to open
Settings, choose another shortcut, save, and restart MusicMonitor.

## WebView2 is missing

Install the current Microsoft Edge WebView2 Runtime from Microsoft, then restart
MusicMonitor. Current Windows 11 installations normally include it; Windows 10
systems vary.

## Windows warns about an unsigned installer

Local development installers are unsigned and may trigger Microsoft SmartScreen.
Only install artifacts you built yourself or obtained from a release whose
publisher and checksum you trust. Public releases should be Authenticode-signed;
do not bypass warnings for an untrusted download.

## Reset local state

1. Quit MusicMonitor from the tray.
2. Delete the `MusicMonitor/SpotifyOAuth` generic credential from Windows
   Credential Manager.
3. Remove `%APPDATA%\com.musicmonitor.desktop` if you also want to reset
   preferences.
4. Start MusicMonitor and complete setup again.

This removes the local session; it does not delete the Spotify developer app or
change the Spotify account.
