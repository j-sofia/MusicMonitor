import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { EMPTY_PLAYBACK, type Playback } from "../types/playback";
import { NowPlaying } from "./NowPlaying";

const PLAYBACK: Playback = {
  ...EMPTY_PLAYBACK,
  status: "playing",
  itemType: "track",
  itemId: "track-id",
  title: "Test Track",
  creators: ["Test Artist"],
  collectionTitle: "Test Album",
  durationMs: 180_000,
  artwork: [{ url: "https://i.scdn.co/album", width: 640, height: 640 }],
  featuredArtist: {
    name: "Test Artist",
    spotifyUrl: "https://open.spotify.com/artist/artist-id",
    artwork: [{ url: "https://i.scdn.co/artist", width: 640, height: 640 }],
  },
};

afterEach(cleanup);

describe("NowPlaying", () => {
  it("shows the linked artist visual when the composition supports it", () => {
    render(
      <NowPlaying
        playback={PLAYBACK}
        pausedMode="dimmedArtwork"
        showArtistVisual
      />,
    );

    expect(screen.getByText("Featured artist")).toBeInTheDocument();
    expect(screen.getByAltText("Test Artist artist portrait")).toHaveAttribute(
      "src",
      "https://i.scdn.co/artist",
    );
    expect(
      screen.getByRole("link", { name: "Open Test Artist in Spotify" }),
    ).toHaveAttribute("href", "https://open.spotify.com/artist/artist-id");
  });

  it("keeps the artist visual out of minimal compositions", () => {
    render(
      <NowPlaying
        playback={PLAYBACK}
        pausedMode="dimmedArtwork"
        showArtistVisual={false}
      />,
    );

    expect(screen.queryByText("Featured artist")).not.toBeInTheDocument();
  });
});
