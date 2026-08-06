import { invoke } from "@tauri-apps/api/core";
import { listen, type Event, type UnlistenFn } from "@tauri-apps/api/event";
import type { Playback } from "../types/playback";
import type { AppSettings, AuthStatus, MonitorInfo } from "../types/settings";

export const isTauri = (): boolean => "__TAURI_INTERNALS__" in window;

export const native = {
  getSettings: (): Promise<AppSettings> => invoke("get_settings"),
  saveSettings: (settings: AppSettings): Promise<AppSettings> =>
    invoke("save_settings", { settings }),
  listMonitors: (): Promise<MonitorInfo[]> => invoke("list_monitors"),
  getAuthStatus: (): Promise<AuthStatus> => invoke("get_auth_status"),
  beginSpotifyLogin: (clientId: string): Promise<void> =>
    invoke("begin_spotify_login", { clientId }),
  disconnectSpotify: (): Promise<void> => invoke("disconnect_spotify"),
  getPlayback: (): Promise<Playback> => invoke("get_playback"),
  completeSetup: (): Promise<void> => invoke("complete_setup"),
  previewDisplay: (): Promise<void> => invoke("preview_display"),
  hideDisplay: (): Promise<void> => invoke("hide_display"),
  openSpotify: (url: string): Promise<void> =>
    invoke("open_spotify_url", { url }),
  openExternal: (url: string): Promise<void> =>
    invoke("open_external_url", { url }),
  revealSettings: (): Promise<void> => invoke("reveal_settings"),
};

export const listenPlayback = (
  handler: (event: Event<Playback>) => void,
): Promise<UnlistenFn> => listen<Playback>("playback://updated", handler);

export const listenAuth = (
  handler: (event: Event<AuthStatus>) => void,
): Promise<UnlistenFn> => listen<AuthStatus>("auth://status", handler);

export const listenSettings = (
  handler: (event: Event<AppSettings>) => void,
): Promise<UnlistenFn> => listen<AppSettings>("settings://updated", handler);
