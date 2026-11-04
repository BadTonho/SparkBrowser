import { listen } from "@tauri-apps/api/event";
import { invoke, isTauriRuntime } from "./runtime";
import type { WebviewRuntimeStatus } from "../../types/webview-runtime";
import type { ShutdownState } from "../../types/shutdown";
export const unavailableRuntime: WebviewRuntimeStatus = { supported: false, currentVersion: null, availableVersion: null, restartRequired: false, revision: 0, error: null };
export const fetchRuntimeStatus = () => isTauriRuntime() ? invoke<WebviewRuntimeStatus>("get_webview_runtime_status") : Promise.resolve(unavailableRuntime);
export const requestRuntimeRestart = () => invoke<ShutdownState>("restart_webview_runtime");
export const onRuntimeStatus = (callback: (status: WebviewRuntimeStatus) => void) => isTauriRuntime()
  ? listen<WebviewRuntimeStatus>("webview-runtime-status", event => callback(event.payload)) : Promise.resolve(() => {});
