import { describe, expect, it } from "vitest";
import { EMPTY_PLAYBACK } from "../types/playback";
import {
  formatDuration,
  interpolatedProgress,
  progressFraction,
} from "./progress";

describe("progress interpolation", () => {
  it("advances active playback from the observation timestamp", () => {
    const playback = {
      ...EMPTY_PLAYBACK,
      status: "playing" as const,
      durationMs: 20_000,
      progressMs: 5_000,
      observedAt: 1_000,
    };
    expect(interpolatedProgress(playback, 3_500)).toBe(7_500);
  });

  it("does not advance paused playback", () => {
    const playback = {
      ...EMPTY_PLAYBACK,
      status: "paused" as const,
      durationMs: 20_000,
      progressMs: 5_000,
      observedAt: 1_000,
    };
    expect(interpolatedProgress(playback, 10_000)).toBe(5_000);
  });

  it("clamps progress at the duration", () => {
    const playback = {
      ...EMPTY_PLAYBACK,
      status: "playing" as const,
      durationMs: 10_000,
      progressMs: 9_500,
      observedAt: 1_000,
    };
    expect(progressFraction(playback, 3_000)).toBe(1);
  });

  it("formats duration for display", () => {
    expect(formatDuration(185_900)).toBe("3:05");
  });
});
