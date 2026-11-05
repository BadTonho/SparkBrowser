import { invoke, isTauriRuntime } from "./runtime";
import type { ExtensionAction, ExtensionDetails } from "../../types/extensions";

export async function fetchInstalledExtensions(): Promise<ExtensionDetails[]> {
  if (!isTauriRuntime()) return [];
  return await invoke<ExtensionDetails[]>("get_installed_extensions");
}

export async function loadUnpackedExtension(path?: string): Promise<ExtensionDetails | null> {
  if (!isTauriRuntime()) return null;
  return await invoke<ExtensionDetails | null>("load_unpacked_extension", { path: path ?? null });
}

export async function toggleNativeExtension(id: string, enabled: boolean): Promise<void> {
  if (!isTauriRuntime()) return;
  await invoke<void>("toggle_extension", { id, enabled });
}

export async function removeNativeExtension(id: string): Promise<void> {
  if (!isTauriRuntime()) return;
  await invoke<void>("remove_extension", { id });
}

export async function fetchExtensionAction(id: string): Promise<ExtensionAction | null> {
  if (!isTauriRuntime()) return null;
  return await invoke<ExtensionAction | null>("get_extension_action", { id });
}

