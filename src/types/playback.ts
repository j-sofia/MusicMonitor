export type PlaybackStatus =
  "playing" | "paused" | "idle" | "offline" | "authRequired" | "error";

export type PlaybackItemType = "track" | "episode" | "unknown";

export interface Artwork {
  url: string;
  width: number | null;
  height: number | null;
}

export interface ArtistVisual {
  name: string;
  spotifyUrl: string;
  artwork: Artwork[];
}

export interface Palette {
  primary: string;
  secondary: string;
  accent: string;
}

export interface Playback {
  status: PlaybackStatus;
  itemType: PlaybackItemType;
  itemId: string | null;
  uri: string | null;
  title: string;
  creators: string[];
  collectionTitle: string | null;
  artwork: Artwork[];
  featuredArtist: ArtistVisual | null;
  durationMs: number;
  progressMs: number;
  observedAt: number;
  isExplicit: boolean;
  spotifyUrl: string | null;
  palette: Palette;
  message: string | null;
}

export const DEFAULT_PALETTE: Palette = {
  primary: "#161b24",
  secondary: "#31233e",
  accent: "#b7a8d8",
};

export const EMPTY_PLAYBACK: Playback = {
  status: "authRequired",
  itemType: "unknown",
  itemId: null,
  uri: null,
  title: "Connect Spotify to begin",
  creators: [],
  collectionTitle: null,
  artwork: [],
  featuredArtist: null,
  durationMs: 0,
  progressMs: 0,
  observedAt: Date.now(),
  isExplicit: false,
  spotifyUrl: null,
  palette: DEFAULT_PALETTE,
  message: null,
};
