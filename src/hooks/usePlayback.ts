import { useEffect, useState } from "react";
import { isTauri, listenPlayback, native } from "../lib/tauri";
import { EMPTY_PLAYBACK, type Playback } from "../types/playback";

const MOCK_PLAYBACK: Playback = {
  ...EMPTY_PLAYBACK,
  status: "playing",
  itemType: "track",
  itemId: "preview",
  uri: "spotify:track:preview",
  title: "A Quiet Place for Sound",
  creators: ["MusicMonitor Preview"],
  collectionTitle: "Gallery Mode",
  artwork: [
    {
      url: "https://i.scdn.co/image/ab67616d00001e02ff9ca10b55ce82ae553c8228",
      width: 300,
      height: 300,
    },
  ],
  featuredArtist: {
    name: "MusicMonitor Preview",
    spotifyUrl: "https://open.spotify.com/artist/preview",
    artwork: [
      {
        url: "https://i.scdn.co/image/ab67616d00001e02ff9ca10b55ce82ae553c8228",
        width: 300,
        height: 300,
      },
    ],
  },
  durationMs: 256_000,
  progressMs: 84_000,
  observedAt: Date.now(),
  palette: {
    primary: "#253452",
    secondary: "#5b3157",
    accent: "#d6a67d",
  },
};

export function usePlayback(): Playback {
  const [playback, setPlayback] = useState<Playback>(
    isTauri() ? EMPTY_PLAYBACK : MOCK_PLAYBACK,
  );

  useEffect(() => {
    if (!isTauri()) return;

    let mounted = true;
    let stopListening: (() => void) | undefined;

    void native
      .getPlayback()
      .then((next) => {
        if (mounted) setPlayback(next);
      })
      .catch(() => undefined);

    void listenPlayback((event) => {
      if (mounted) setPlayback(event.payload);
    }).then((unlisten) => {
      if (mounted) stopListening = unlisten;
      else unlisten();
    });

    return () => {
      mounted = false;
      stopListening?.();
    };
  }, []);

  return playback;
}
