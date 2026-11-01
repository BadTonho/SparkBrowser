import { getCurrentWindow } from "@tauri-apps/api/window";

import { isTauriRuntime } from "./runtime";

export async function minimizeWindow(): Promise<void> {
  if(!isTauriRuntime()) return;
  await getCurrentWindow().minimize();
}

export async function toggleMaximizeWindow(): Promise<void> {
  if(!isTauriRuntime()) return;
  await getCurrentWindow().toggleMaximize();
}

export async function closeWindow(): Promise<void> {
  if(!isTauriRuntime()) return;
  await getCurrentWindow().close();
}
