import { useEffect, useMemo, useRef, useState } from "react";
import { Clock } from "./components/Clock";
import { DisplayState } from "./components/DisplayState";
import { NowPlaying } from "./components/NowPlaying";
import { usePlayback } from "./hooks/usePlayback";
import { isTauri, listenSettings, native } from "./lib/tauri";
import type { AppSettings } from "./types/settings";

const FALLBACK_SETTINGS: Pick<
  AppSettings,
  | "theme"
  | "staticBackground"
  | "showArtistView"
  | "pausedBehavior"
  | "idleBehavior"
  | "oledDimAfterMinutes"
> = {
  theme: "gallery",
  staticBackground: false,
  showArtistView: true,
  pausedBehavior: "dimmedArtwork",
  idleBehavior: "ambientClock",
  oledDimAfterMinutes: 30,
};

export function DisplayApp() {
  const playback = usePlayback();
  const [settings, setSettings] = useState(FALLBACK_SETTINGS);
  const itemRef = useRef(playback.itemId);
  const [unchangedSince, setUnchangedSince] = useState(Date.now());

  useEffect(() => {
    if (!isTauri()) return;
    let unlisten: (() => void) | undefined;
    void native
      .getSettings()
      .then(setSettings)
      .catch(() => undefined);
    void listenSettings((event) => setSettings(event.payload)).then((stop) => {
      unlisten = stop;
    });
    return () => unlisten?.();
  }, []);

  useEffect(() => {
    if (itemRef.current !== playback.itemId) {
      itemRef.current = playback.itemId;
      setUnchangedSince(Date.now());
    }
  }, [playback.itemId]);

  const unchangedMinutes = Math.max(0, Date.now() - unchangedSince) / 60_000;
  const dimmed = unchangedMinutes >= settings.oledDimAfterMinutes;
  const style = useMemo(
    () =>
      ({
        "--palette-primary": playback.palette.primary,
        "--palette-secondary": playback.palette.secondary,
        "--palette-accent": playback.palette.accent,
      }) as React.CSSProperties,
    [playback.palette],
  );

  const showClock =
    playback.status === "paused" && settings.pausedBehavior === "clock";
  const hasItem =
    playback.itemId !== null &&
    (playback.status === "playing" || playback.status === "paused");

  return (
    <div
      className={`display-shell theme-${settings.theme} ${settings.staticBackground ? "is-static" : ""} ${dimmed ? "is-dimmed" : ""}`}
      style={style}
    >
      <div className="ambient ambient--one" />
      <div className="ambient ambient--two" />
      <div className="grain" />
      {showClock ? (
        <main className="display-state">
          <Clock message="Spotify is paused" />
        </main>
      ) : hasItem ? (
        <NowPlaying
          playback={playback}
          pausedMode={settings.pausedBehavior}
          showArtistVisual={
            settings.showArtistView &&
            (settings.theme === "gallery" || settings.theme === "colorField")
          }
        />
      ) : (
        <DisplayState
          playback={playback}
          idleBehavior={settings.idleBehavior}
        />
      )}
    </div>
  );
}
