import { listen } from "@tauri-apps/api/event";
import { invoke, isTauriRuntime } from "./runtime";
import type { UpdateChannel, UpdateState } from "../../types/updates";
import { version } from "../../../../package.json";

export const previewUpdateState: UpdateState = {
  revision: 0, supported: false, message: "Updates require the installed Windows x64 release.",
  current_version: version, channel: "stable", last_checked: null, status: "unavailable",
  update: null, received: 0, total: null, error: null, retry_after: null,
};
export const fetchUpdateState = () => isTauriRuntime() ? invoke<UpdateState>("get_update_state") : Promise.resolve(previewUpdateState);
export const changeUpdateChannel = (channel: UpdateChannel) => invoke<UpdateState>("set_update_channel", { channel });
export const checkForUpdates = (startup = false) => invoke<UpdateState>("check_for_updates", { startup });
export const downloadUpdate = (expectedVersion: string) => invoke<UpdateState>("download_update", { expectedVersion });
export const installUpdate = (expectedVersion: string, confirmedActivity: boolean) => invoke<void>("install_update", { expectedVersion, confirmedActivity });
export const onUpdateState = (callback: (state: UpdateState) => void) => isTauriRuntime() ? listen<UpdateState>("update-state", event => callback(event.payload)) : Promise.resolve(() => {});
