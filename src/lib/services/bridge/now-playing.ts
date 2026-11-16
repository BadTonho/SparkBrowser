import { listen } from "@tauri-apps/api/event";
import { invoke, isTauriRuntime } from "./runtime";

/** The YouTube or YouTube Music video playing in one of the open tabs. */
export interface NowPlaying {
  tab_id: string;
  source: "youtube" | "music";
  video_id: string;
  title: string;
  playing: boolean;
  /** Seconds; null until the tab reported them, and for live streams. */
  position: number | null;
  duration: number | null;
  /** A `data:image/jpeg` thumbnail fetched by the backend, or null. */
  image: string | null;
}
export type NowPlayingOperation = "toggle" | "next" | "previous" | "open";

export const fetchNowPlaying = () => isTauriRuntime() ? invoke<NowPlaying | null>("get_now_playing") : Promise.resolve(null);
export const onNowPlaying = (callback: (video: NowPlaying | null) => void) => isTauriRuntime() ? listen<NowPlaying | null>("youtube-now-playing", event => callback(event.payload)) : Promise.resolve(() => {});
export const controlNowPlaying = (operation: NowPlayingOperation) => invoke<void>("control_now_playing", { operation });
