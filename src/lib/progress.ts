import type { Playback } from "../types/playback";

export function interpolatedProgress(
  playback: Playback,
  now = Date.now(),
): number {
  if (playback.durationMs <= 0) return 0;
  const elapsed =
    playback.status === "playing" ? Math.max(0, now - playback.observedAt) : 0;
  return Math.min(
    playback.durationMs,
    Math.max(0, playback.progressMs + elapsed),
  );
}

export function progressFraction(playback: Playback, now = Date.now()): number {
  if (playback.durationMs <= 0) return 0;
  return interpolatedProgress(playback, now) / playback.durationMs;
}

export function formatDuration(milliseconds: number): string {
  const totalSeconds = Math.max(0, Math.floor(milliseconds / 1_000));
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return `${minutes}:${seconds.toString().padStart(2, "0")}`;
}
