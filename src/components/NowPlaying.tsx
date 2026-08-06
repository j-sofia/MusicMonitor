import { useEffect, useState } from "react";
import {
  formatDuration,
  interpolatedProgress,
  progressFraction,
} from "../lib/progress";
import { native } from "../lib/tauri";
import type { Playback } from "../types/playback";
import { SpotifyLogo } from "./SpotifyLogo";

interface NowPlayingProps {
  playback: Playback;
  pausedMode: "dimmedArtwork" | "pausedComposition" | "clock";
  showArtistVisual: boolean;
}

export function NowPlaying({
  playback,
  pausedMode,
  showArtistVisual,
}: NowPlayingProps) {
  const [, setTick] = useState(0);
  const artwork = playback.artwork[0];
  const featuredArtist = showArtistVisual ? playback.featuredArtist : null;
  const artistArtwork = featuredArtist?.artwork[0];

  useEffect(() => {
    const timer = window.setInterval(() => setTick((value) => value + 1), 500);
    return () => window.clearInterval(timer);
  }, []);

  const progress = interpolatedProgress(playback);
  const fraction = progressFraction(playback);
  const creators = playback.creators.join(", ");
  const isPaused = playback.status === "paused";

  return (
    <main
      className={`now-playing ${artistArtwork ? "now-playing--with-artist" : ""} ${isPaused ? "now-playing--paused" : ""} paused-${pausedMode}`}
      aria-label={`${isPaused ? "Paused" : "Now playing"}: ${playback.title}`}
    >
      <a
        className="artwork-frame"
        href={playback.spotifyUrl ?? undefined}
        aria-label={`Open ${playback.title} in Spotify`}
        onClick={(event) => {
          event.preventDefault();
          if (playback.spotifyUrl) void native.openSpotify(playback.spotifyUrl);
        }}
      >
        {artwork ? (
          <img
            key={artwork.url}
            className="artwork"
            src={artwork.url}
            alt={`${playback.collectionTitle ?? playback.title} artwork`}
            draggable={false}
          />
        ) : (
          <div className="artwork artwork--placeholder" aria-hidden="true">
            <span>MM</span>
          </div>
        )}
      </a>

      <section className="metadata">
        <div className="metadata__status-row">
          <p className="eyebrow">{isPaused ? "Paused" : "Now playing"}</p>
          {playback.isExplicit && (
            <span className="explicit" title="Explicit content">
              E
            </span>
          )}
        </div>
        <h1 title={playback.title}>{playback.title}</h1>
        {creators && (
          <p className="metadata__creators" title={creators}>
            {creators}
          </p>
        )}
        {playback.collectionTitle && (
          <p className="metadata__collection" title={playback.collectionTitle}>
            {playback.itemType === "episode" ? "From " : "On "}
            {playback.collectionTitle}
          </p>
        )}

        <div
          className="progress"
          aria-label={`${formatDuration(progress)} of ${formatDuration(playback.durationMs)}`}
        >
          <div className="progress__track">
            <div
              className="progress__fill"
              style={{ transform: `scaleX(${fraction})` }}
            />
          </div>
          <div className="progress__times">
            <span>{formatDuration(progress)}</span>
            <span>{formatDuration(playback.durationMs)}</span>
          </div>
        </div>

        <button
          className="spotify-link"
          type="button"
          disabled={!playback.spotifyUrl}
          onClick={() =>
            playback.spotifyUrl && void native.openSpotify(playback.spotifyUrl)
          }
        >
          <SpotifyLogo />
          <span className="spotify-link__action">Open Spotify</span>
        </button>
      </section>

      {featuredArtist && artistArtwork && (
        <a
          className="artist-feature"
          href={featuredArtist.spotifyUrl}
          aria-label={`Open ${featuredArtist.name} in Spotify`}
          onClick={(event) => {
            event.preventDefault();
            void native.openSpotify(featuredArtist.spotifyUrl);
          }}
        >
          <img
            key={artistArtwork.url}
            className="artist-feature__image"
            src={artistArtwork.url}
            alt={`${featuredArtist.name} artist portrait`}
            draggable={false}
          />
          <span className="artist-feature__eyebrow">Featured artist</span>
          <strong title={featuredArtist.name}>{featuredArtist.name}</strong>
          <span className="artist-feature__action">View on Spotify</span>
        </a>
      )}
    </main>
  );
}
