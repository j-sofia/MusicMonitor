import { useCallback, useEffect, useMemo, useState } from "react";
import { SpotifyLogo } from "./components/SpotifyLogo";
import { usePlayback } from "./hooks/usePlayback";
import { isTauri, listenAuth, native } from "./lib/tauri";
import type { AppSettings, AuthStatus, MonitorInfo } from "./types/settings";

type Tab = "setup" | "display" | "behavior" | "system" | "about";

const INITIAL_AUTH: AuthStatus = { state: "disconnected", message: null };

const PREVIEW_SETTINGS: AppSettings = {
  schemaVersion: 1,
  spotifyClientId: "",
  selectedMonitorId: "preview-display-2",
  selectedMonitorName: "Display 2",
  selectedMonitorPosition: { x: 1920, y: 0 },
  selectedMonitorSize: { width: 2560, height: 1440 },
  returnDelayMs: 500,
  entryDwellMs: 0,
  edgeHysteresisPx: 4,
  coverTaskbar: true,
  launchAtStartup: false,
  startMinimized: true,
  staticBackground: false,
  showArtistView: true,
  theme: "gallery",
  pausedBehavior: "dimmedArtwork",
  idleBehavior: "ambientClock",
  oledDimAfterMinutes: 30,
  setupComplete: false,
  safetyShortcut: "Ctrl+Shift+F12",
};

const PREVIEW_MONITORS: MonitorInfo[] = [
  {
    id: "preview-display-1",
    name: "Display 1",
    isPrimary: true,
    position: { x: 0, y: 0 },
    size: { width: 1920, height: 1080 },
    workAreaPosition: { x: 0, y: 0 },
    workAreaSize: { width: 1920, height: 1040 },
    scaleFactor: 1,
  },
  {
    id: "preview-display-2",
    name: "Display 2",
    isPrimary: false,
    position: { x: 1920, y: 0 },
    size: { width: 2560, height: 1440 },
    workAreaPosition: { x: 1920, y: 0 },
    workAreaSize: { width: 2560, height: 1400 },
    scaleFactor: 1.5,
  },
];

function monitorDescription(monitor: MonitorInfo): string {
  const scale = Math.round(monitor.scaleFactor * 100);
  return `${monitor.size.width} × ${monitor.size.height} · ${scale}%${monitor.isPrimary ? " · Primary" : ""}`;
}

export function SettingsApp() {
  const playback = usePlayback();
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [monitors, setMonitors] = useState<MonitorInfo[]>([]);
  const [auth, setAuth] = useState<AuthStatus>(INITIAL_AUTH);
  const [tab, setTab] = useState<Tab>("setup");
  const [saving, setSaving] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    if (!isTauri()) {
      setSettings(PREVIEW_SETTINGS);
      setMonitors(PREVIEW_MONITORS);
      setAuth(INITIAL_AUTH);
      return;
    }
    const [nextSettings, nextMonitors, nextAuth] = await Promise.all([
      native.getSettings(),
      native.listMonitors(),
      native.getAuthStatus(),
    ]);
    setSettings(nextSettings);
    setMonitors(nextMonitors);
    setAuth(nextAuth);
    if (nextSettings.setupComplete) setTab("display");
  }, []);

  useEffect(() => {
    void refresh().catch((error: unknown) => {
      setNotice(
        error instanceof Error
          ? error.message
          : "Could not load MusicMonitor settings.",
      );
    });

    let unlisten: (() => void) | undefined;
    if (isTauri()) {
      void listenAuth((event) => setAuth(event.payload)).then((stop) => {
        unlisten = stop;
      });
    }
    return () => unlisten?.();
  }, [refresh]);

  const selectedMonitor = useMemo(
    () =>
      monitors.find((monitor) => monitor.id === settings?.selectedMonitorId),
    [monitors, settings?.selectedMonitorId],
  );

  function update<K extends keyof AppSettings>(key: K, value: AppSettings[K]) {
    setSettings((current) =>
      current ? { ...current, [key]: value } : current,
    );
  }

  async function save(message = "Settings saved"): Promise<boolean> {
    if (!settings) return false;
    setSaving(true);
    setNotice(null);
    try {
      const persisted = await native.saveSettings(settings);
      setSettings(persisted);
      setNotice(message);
      return true;
    } catch (error: unknown) {
      setNotice(
        error instanceof Error ? error.message : "Settings could not be saved.",
      );
      return false;
    } finally {
      setSaving(false);
    }
  }

  async function connectSpotify() {
    if (!settings) return;
    if (!settings.spotifyClientId.trim()) {
      setNotice("Enter the client ID from your Spotify developer app first.");
      return;
    }
    if (
      !(await save("Client ID saved. Complete authorization in your browser."))
    ) {
      return;
    }
    setAuth({
      state: "authorizing",
      message: "Waiting for Spotify authorization…",
    });
    try {
      await native.beginSpotifyLogin(settings.spotifyClientId.trim());
    } catch (error: unknown) {
      setAuth({
        state: "error",
        message:
          error instanceof Error
            ? error.message
            : "Spotify authorization could not start.",
      });
    }
  }

  async function finishSetup() {
    if (!settings?.selectedMonitorId) {
      setNotice("Choose a display first.");
      return;
    }
    if (!(await save())) return;
    await native.completeSetup();
    setSettings((current) =>
      current ? { ...current, setupComplete: true } : current,
    );
    setNotice(
      "MusicMonitor is ready. Move the pointer off the selected display to reveal it.",
    );
  }

  if (!settings) {
    return <div className="settings-loading">Loading MusicMonitor…</div>;
  }

  return (
    <div className="settings-shell">
      <aside className="settings-sidebar">
        <div className="app-mark" aria-hidden="true">
          <span>MM</span>
        </div>
        <div className="settings-brand">
          <strong>MusicMonitor</strong>
          <span>Ambient now playing</span>
        </div>
        <nav aria-label="Settings sections">
          {(
            [
              ["setup", "Spotify"],
              ["display", "Display"],
              ["behavior", "Behavior"],
              ["system", "Windows"],
              ["about", "About"],
            ] as const
          ).map(([id, label]) => (
            <button
              key={id}
              className={tab === id ? "active" : ""}
              onClick={() => setTab(id)}
            >
              {label}
            </button>
          ))}
        </nav>
        <p className="settings-sidebar__hint">
          Safety shortcut
          <br />
          <kbd>{settings.safetyShortcut}</kbd>
        </p>
      </aside>

      <main className="settings-main">
        {tab === "setup" && (
          <section className="settings-section">
            <header>
              <p className="eyebrow">Read-only Spotify access</p>
              <h1>Connect Spotify</h1>
              <p>
                MusicMonitor reads only the item currently playing. It cannot
                control playback.
              </p>
            </header>

            <div className="status-card">
              <span className={`status-dot status-dot--${auth.state}`} />
              <div>
                <strong>
                  {auth.state === "connected"
                    ? "Spotify connected"
                    : auth.state === "authorizing"
                      ? "Waiting for authorization"
                      : "Spotify not connected"}
                </strong>
                <p>
                  {auth.message ??
                    "A personal Spotify developer app is required for the current Web API."}
                </p>
              </div>
            </div>

            {auth.state === "connected" && (
              <div className="spotify-companion-card">
                <SpotifyLogo />
                <div>
                  <small>
                    {playback.itemId ? "Current item" : "Connected companion"}
                  </small>
                  <strong>
                    {playback.itemId ? playback.title : "Ready for Spotify"}
                  </strong>
                  <p>
                    {playback.itemId
                      ? playback.creators.join(", ")
                      : "Start playback in Spotify to fill the selected display."}
                  </p>
                </div>
                <button
                  className="button"
                  type="button"
                  onClick={() => {
                    void native.openSpotify(
                      playback.spotifyUrl ?? "https://open.spotify.com/",
                    );
                  }}
                >
                  Open Spotify
                </button>
              </div>
            )}

            <div className="setting-group">
              <label htmlFor="client-id">Spotify client ID</label>
              <input
                id="client-id"
                type="text"
                value={settings.spotifyClientId}
                spellCheck={false}
                autoComplete="off"
                placeholder="Paste your client ID"
                onChange={(event) =>
                  update("spotifyClientId", event.target.value.trim())
                }
              />
              <p className="field-help">
                The client ID identifies your app and is not a secret.
                MusicMonitor never asks for a client secret.
              </p>
            </div>

            <ol className="setup-steps">
              <li>Create an app in the Spotify Developer Dashboard.</li>
              <li>
                Add <code>http://127.0.0.1:43819/callback</code> as a redirect
                URI, then save the app settings.
              </li>
              <li>
                Paste the client ID above, then connect. Spotify permits HTTP
                for this explicit loopback address; do not change it to HTTPS or{" "}
                <code>localhost</code>.
              </li>
              <li>
                Spotify currently requires a Premium app owner and limits new
                Development Mode apps to five authorized users.
              </li>
            </ol>

            <div className="button-row">
              <button
                className="button button--primary"
                disabled={auth.state === "authorizing"}
                onClick={() => void connectSpotify()}
              >
                {auth.state === "connected"
                  ? "Reconnect Spotify"
                  : "Connect Spotify"}
              </button>
              <button
                className="button"
                onClick={() =>
                  void native.openExternal(
                    "https://developer.spotify.com/dashboard",
                  )
                }
              >
                Open developer dashboard
              </button>
              {auth.state === "connected" && (
                <button
                  className="button button--danger"
                  onClick={() =>
                    void native
                      .disconnectSpotify()
                      .then(() => setAuth(INITIAL_AUTH))
                  }
                >
                  Disconnect
                </button>
              )}
            </div>
          </section>
        )}

        {tab === "display" && (
          <section className="settings-section">
            <header>
              <p className="eyebrow">Target monitor</p>
              <h1>Choose the canvas</h1>
              <p>
                The selected display is covered only while your pointer is
                elsewhere.
              </p>
            </header>

            <div className="monitor-grid">
              {monitors.map((monitor, index) => (
                <button
                  key={monitor.id}
                  className={`monitor-card ${monitor.id === settings.selectedMonitorId ? "selected" : ""}`}
                  onClick={() => {
                    update("selectedMonitorId", monitor.id);
                    update("selectedMonitorName", monitor.name);
                    update("selectedMonitorPosition", monitor.position);
                    update("selectedMonitorSize", monitor.size);
                  }}
                >
                  <span className="monitor-card__screen">{index + 1}</span>
                  <span>
                    <strong>{monitor.name || `Display ${index + 1}`}</strong>
                    {monitorDescription(monitor)}
                  </span>
                </button>
              ))}
            </div>

            <div className="setting-grid">
              <label className="setting-group">
                <span>Theme</span>
                <select
                  value={settings.theme}
                  onChange={(event) =>
                    update("theme", event.target.value as AppSettings["theme"])
                  }
                >
                  <option value="gallery">Gallery</option>
                  <option value="minimalOled">Minimal OLED</option>
                  <option value="colorField">Color field</option>
                  <option value="vinylRoom">Vinyl room</option>
                </select>
              </label>
              <label className="toggle-row">
                <span>
                  <strong>Static background</strong>
                  <small>Turns off ambient gradient movement</small>
                </span>
                <input
                  type="checkbox"
                  checked={settings.staticBackground}
                  onChange={(event) =>
                    update("staticBackground", event.target.checked)
                  }
                />
              </label>
              <label className="toggle-row">
                <span>
                  <strong>Show artist view</strong>
                  <small>
                    Show the featured artist portrait when available
                  </small>
                </span>
                <input
                  type="checkbox"
                  checked={settings.showArtistView}
                  onChange={(event) =>
                    update("showArtistView", event.target.checked)
                  }
                />
              </label>
              <label className="toggle-row">
                <span>
                  <strong>Cover the taskbar</strong>
                  <small>Use the monitor's full physical bounds</small>
                </span>
                <input
                  type="checkbox"
                  checked={settings.coverTaskbar}
                  onChange={(event) =>
                    update("coverTaskbar", event.target.checked)
                  }
                />
              </label>
            </div>

            <div className="button-row">
              <button
                className="button button--primary"
                disabled={saving || !selectedMonitor}
                onClick={() => void save()}
              >
                {saving ? "Saving…" : "Save display"}
              </button>
              <button
                className="button"
                disabled={!selectedMonitor}
                onClick={() =>
                  void save().then((saved) => {
                    if (saved) return native.previewDisplay();
                  })
                }
              >
                Preview display
              </button>
              <button className="button" onClick={() => void refresh()}>
                Refresh monitors
              </button>
            </div>
          </section>
        )}

        {tab === "behavior" && (
          <section className="settings-section">
            <header>
              <p className="eyebrow">Pointer behavior</p>
              <h1>Disappear gracefully</h1>
              <p>
                Native tracking continues even while the display window is
                hidden.
              </p>
            </header>
            <div className="setting-grid">
              <label className="setting-group">
                <span>Return delay</span>
                <div className="range-row">
                  <input
                    type="range"
                    min="0"
                    max="5000"
                    step="100"
                    value={settings.returnDelayMs}
                    onChange={(event) =>
                      update("returnDelayMs", Number(event.target.value))
                    }
                  />
                  <output>{settings.returnDelayMs} ms</output>
                </div>
                <small>
                  How long the pointer must remain outside before the display
                  returns.
                </small>
              </label>
              <label className="setting-group">
                <span>Entry dwell</span>
                <div className="range-row">
                  <input
                    type="range"
                    min="0"
                    max="500"
                    step="25"
                    value={settings.entryDwellMs}
                    onChange={(event) =>
                      update("entryDwellMs", Number(event.target.value))
                    }
                  />
                  <output>{settings.entryDwellMs} ms</output>
                </div>
                <small>Set to zero for the fastest possible hide.</small>
              </label>
              <label className="setting-group">
                <span>Edge hysteresis</span>
                <div className="range-row">
                  <input
                    type="range"
                    min="0"
                    max="24"
                    step="1"
                    value={settings.edgeHysteresisPx}
                    onChange={(event) =>
                      update("edgeHysteresisPx", Number(event.target.value))
                    }
                  />
                  <output>{settings.edgeHysteresisPx} px</output>
                </div>
                <small>
                  Prevents flicker from noisy coordinates along monitor
                  boundaries.
                </small>
              </label>
              <label className="setting-group">
                <span>While paused</span>
                <select
                  value={settings.pausedBehavior}
                  onChange={(event) =>
                    update(
                      "pausedBehavior",
                      event.target.value as AppSettings["pausedBehavior"],
                    )
                  }
                >
                  <option value="dimmedArtwork">Keep dimmed artwork</option>
                  <option value="pausedComposition">
                    Dedicated paused composition
                  </option>
                  <option value="clock">Show a clock</option>
                </select>
              </label>
              <label className="setting-group">
                <span>Nothing playing</span>
                <select
                  value={settings.idleBehavior}
                  onChange={(event) =>
                    update(
                      "idleBehavior",
                      event.target.value as AppSettings["idleBehavior"],
                    )
                  }
                >
                  <option value="ambientClock">Ambient clock</option>
                  <option value="status">Minimal status</option>
                  <option value="black">Black screen</option>
                </select>
              </label>
              <label className="setting-group">
                <span>OLED dimming delay</span>
                <div className="range-row">
                  <input
                    type="range"
                    min="5"
                    max="120"
                    step="5"
                    value={settings.oledDimAfterMinutes}
                    onChange={(event) =>
                      update("oledDimAfterMinutes", Number(event.target.value))
                    }
                  />
                  <output>{settings.oledDimAfterMinutes} min</output>
                </div>
              </label>
            </div>
            <div className="button-row">
              <button
                className="button button--primary"
                disabled={saving}
                onClick={() => void save()}
              >
                {saving ? "Saving…" : "Save behavior"}
              </button>
            </div>
          </section>
        )}

        {tab === "system" && (
          <section className="settings-section">
            <header>
              <p className="eyebrow">Windows integration</p>
              <h1>Launch and recovery</h1>
              <p>
                MusicMonitor stays accessible from the notification-area tray.
              </p>
            </header>
            <div className="setting-grid">
              <label className="toggle-row">
                <span>
                  <strong>Launch with Windows</strong>
                  <small>Uses the current user's startup registration</small>
                </span>
                <input
                  type="checkbox"
                  checked={settings.launchAtStartup}
                  onChange={(event) =>
                    update("launchAtStartup", event.target.checked)
                  }
                />
              </label>
              <label className="toggle-row">
                <span>
                  <strong>Start silently in the tray</strong>
                  <small>
                    Settings still opens on first-run or when setup needs
                    attention
                  </small>
                </span>
                <input
                  type="checkbox"
                  checked={settings.startMinimized}
                  onChange={(event) =>
                    update("startMinimized", event.target.checked)
                  }
                />
              </label>
              <label className="setting-group">
                <span>Safety shortcut</span>
                <input
                  value={settings.safetyShortcut}
                  onChange={(event) =>
                    update("safetyShortcut", event.target.value)
                  }
                />
                <small>
                  Restart MusicMonitor after changing the global shortcut.
                </small>
              </label>
            </div>
            <div className="button-row">
              <button
                className="button button--primary"
                disabled={saving}
                onClick={() => void save()}
              >
                {saving ? "Saving…" : "Save Windows settings"}
              </button>
              <button
                className="button"
                onClick={() => void native.hideDisplay()}
              >
                Hide display now
              </button>
            </div>
          </section>
        )}

        {tab === "about" && (
          <section className="settings-section">
            <header>
              <p className="eyebrow">MusicMonitor 0.1.3</p>
              <h1>An ambient display, not a player</h1>
              <p>
                MusicMonitor cannot pause, seek, skip, change volume, transfer
                playback, or modify your Spotify account.
              </p>
            </header>
            <div className="about-card">
              <h2>Privacy by design</h2>
              <p>
                No telemetry. OAuth tokens stay in Windows Credential Manager.
                Preferences stay in your local app-data folder. The app
                communicates only with Spotify and the artwork URLs returned by
                Spotify.
              </p>
              <h2>Spotify attribution</h2>
              <p>
                Spotify metadata and artwork remain unmodified and link back to
                Spotify. MusicMonitor is an independent companion and is not
                affiliated with or endorsed by Spotify.
              </p>
            </div>
            <div className="button-row">
              <button
                className="button"
                onClick={() =>
                  void native.openExternal(
                    "https://developer.spotify.com/documentation/design",
                  )
                }
              >
                Spotify design guidelines
              </button>
            </div>
          </section>
        )}

        <footer className="settings-footer">
          <span
            className={
              notice?.includes("could") ? "notice notice--error" : "notice"
            }
          >
            {notice}
          </span>
          {!settings.setupComplete && (
            <button
              className="button button--primary"
              disabled={!settings.selectedMonitorId}
              onClick={() => void finishSetup()}
            >
              Finish setup
            </button>
          )}
        </footer>
      </main>
    </div>
  );
}
