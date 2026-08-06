export type Theme = "gallery" | "minimalOled" | "colorField" | "vinylRoom";
export type PausedBehavior = "dimmedArtwork" | "pausedComposition" | "clock";
export type IdleBehavior = "ambientClock" | "status" | "black";

export interface MonitorInfo {
  id: string;
  name: string;
  isPrimary: boolean;
  position: { x: number; y: number };
  size: { width: number; height: number };
  workAreaPosition: { x: number; y: number };
  workAreaSize: { width: number; height: number };
  scaleFactor: number;
}

export interface AppSettings {
  schemaVersion: number;
  spotifyClientId: string;
  selectedMonitorId: string | null;
  selectedMonitorName: string | null;
  selectedMonitorPosition: { x: number; y: number } | null;
  selectedMonitorSize: { width: number; height: number } | null;
  returnDelayMs: number;
  entryDwellMs: number;
  edgeHysteresisPx: number;
  coverTaskbar: boolean;
  launchAtStartup: boolean;
  startMinimized: boolean;
  staticBackground: boolean;
  showArtistView: boolean;
  theme: Theme;
  pausedBehavior: PausedBehavior;
  idleBehavior: IdleBehavior;
  oledDimAfterMinutes: number;
  setupComplete: boolean;
  safetyShortcut: string;
}

export interface AuthStatus {
  state:
    | "notConfigured"
    | "disconnected"
    | "authorizing"
    | "connected"
    | "expired"
    | "error";
  message: string | null;
}
