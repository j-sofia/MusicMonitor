import { Clock } from "./Clock";
import { SpotifyLogo } from "./SpotifyLogo";
import type { Playback } from "../types/playback";
import type { IdleBehavior } from "../types/settings";

interface DisplayStateProps {
  playback: Playback;
  idleBehavior: IdleBehavior;
}

export function DisplayState({ playback, idleBehavior }: DisplayStateProps) {
  if (playback.status === "idle" && idleBehavior === "black") {
    return (
      <main
        className="display-state display-state--black"
        aria-label="Nothing playing"
      />
    );
  }

  const copy: Record<
    Playback["status"],
    { eyebrow: string; title: string; message: string }
  > = {
    playing: { eyebrow: "Spotify", title: playback.title, message: "" },
    paused: {
      eyebrow: "Paused",
      title: playback.title,
      message: "Playback is paused in Spotify",
    },
    idle: {
      eyebrow: "Spotify",
      title: "Nothing playing",
      message: "Start something in Spotify and it will appear here.",
    },
    offline: {
      eyebrow: "Connection",
      title: "Waiting for the network",
      message: playback.message ?? "MusicMonitor will retry automatically.",
    },
    authRequired: {
      eyebrow: "Setup required",
      title: "Connect Spotify",
      message:
        "Open MusicMonitor Settings from the tray to connect your account.",
    },
    error: {
      eyebrow: "MusicMonitor",
      title: "Something needs attention",
      message: playback.message ?? "Open Settings from the tray for details.",
    },
  };
  const content = copy[playback.status];

  if (playback.status === "idle" && idleBehavior === "ambientClock") {
    return (
      <main className="display-state">
        <Clock message="Nothing playing" />
        <div className="display-state__attribution">
          <SpotifyLogo />
        </div>
      </main>
    );
  }

  return (
    <main className="display-state">
      <div className="display-state__copy">
        <p className="eyebrow">{content.eyebrow}</p>
        <h1>{content.title}</h1>
        <p>{content.message}</p>
      </div>
      <div className="display-state__attribution">
        <SpotifyLogo />
      </div>
    </main>
  );
}
