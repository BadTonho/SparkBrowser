import { listen } from "@tauri-apps/api/event";
import { invoke, isTauriRuntime } from "./runtime";
import type { ShutdownState } from "../../types/shutdown";
export const idleShutdown: ShutdownState = { purpose: "exit", revision: 0, phase: "idle", attempt: null, update_version: null, review: null, error: null };
export const fetchShutdown = () => isTauriRuntime() ? invoke<ShutdownState>("get_shutdown_state") : Promise.resolve(idleShutdown);
export const prepareShutdown = (attempt: string) => invoke<ShutdownState>("prepare_shutdown", { attempt });
export const confirmShutdown = (attempt: string, revision: number) => invoke<ShutdownState>("confirm_shutdown", { attempt, revision });
export const cancelShutdown = (attempt: string) => invoke<ShutdownState>("cancel_shutdown", { attempt });
export const onShutdown = (callback: (state: ShutdownState) => void) => isTauriRuntime()
  ? listen<ShutdownState>("shutdown-state", event => callback(event.payload)) : Promise.resolve(() => {});
