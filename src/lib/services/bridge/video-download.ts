import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { DownloadTask } from "../../types/browser";
import type { PanelPresentation, PanelViewAction } from "../panel-presentation";
import { invoke, isTauriRuntime } from "./runtime";

export interface VideoSource {
  url: string;
  kind: "file" | "hls";
  referer: string | null;
  host: string;
  height: number;
  bandwidth: number;
  /** Stable code that explains why this source cannot be saved. */
  problem: string | null;
  /** The `PageVideo` this option saves. */
  video: string;
}
/** One video of the page; `thumbnail` is a small PNG data address. */
export interface PageVideo { id: string; thumbnail: string | null; duration: number; on_screen: boolean }
export interface VideoScan { videos: PageVideo[]; sources: VideoSource[]; protected: boolean }

export const scanPageVideos = (tabId: string): Promise<VideoScan> =>
  isTauriRuntime() ? invoke<VideoScan>("scan_page_videos", { tabId }) : Promise.resolve({ videos: [], sources: [], protected: false });

/** Resolves to null when the user cancels the save dialog. */
export const downloadVideo = (tabId: string, source: VideoSource): Promise<DownloadTask | null> =>
  invoke<DownloadTask | null>("download_video", { tabId, url: source.url, referer: source.referer, kind: source.kind });

export interface VideoDownloadPopoverEvent { sessionId: number; sequence: number; action: PanelViewAction }
let queue: Promise<void> = Promise.resolve();
export function setVideoDownloadPopover(visible: boolean, sessionId: number, presentation?: PanelPresentation, restoreFocus = false): Promise<void> {
  if (!isTauriRuntime()) return Promise.resolve();
  queue = queue.catch(() => {}).then(() => invoke<void>("set_video_download_popover", { visible, sessionId, presentation: presentation ?? null, restoreFocus }));
  return queue;
}
export const onVideoDownloadPopoverAction = (callback: (event: VideoDownloadPopoverEvent) => void): Promise<UnlistenFn> =>
  isTauriRuntime() ? listen<VideoDownloadPopoverEvent>("video-download-popover-action", event => callback(event.payload)) : Promise.resolve(() => {});
