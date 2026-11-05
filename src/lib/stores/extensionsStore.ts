import { writable, get } from "svelte/store";
import type { ExtensionDetails } from "../types/extensions";
import {
  fetchInstalledExtensions,
  loadUnpackedExtension,
  installExtensionFromUrl,
  installExtensionFromArchive,
  toggleNativeExtension,
  removeNativeExtension,
} from "../services/tauri-bridge";
import { setOverlayVisibility } from "../services/bridge/overlays";

export const installedExtensions = writable<ExtensionDetails[]>([]);
export const isExtensionsModalOpen = writable(false);
export const activeExtensionPopup = writable<{
  id: string;
  name: string;
  popup_url: string;
} | null>(null);
export const extensionsLoading = writable(false);
export const extensionsError = writable<string | null>(null);

export async function refreshExtensions(): Promise<void> {
  try {
    extensionsLoading.set(true);
    extensionsError.set(null);
    const list = await fetchInstalledExtensions();
    installedExtensions.set(list);
  } catch (e: any) {
    extensionsError.set(e?.message ?? String(e));
  } finally {
    extensionsLoading.set(false);
  }
}

export async function installUnpacked(path?: string): Promise<ExtensionDetails | null> {
  try {
    extensionsLoading.set(true);
    extensionsError.set(null);
    const result = await loadUnpackedExtension(path);
    if (result) {
      installedExtensions.update(list => {
        const filtered = list.filter(e => e.id !== result.id);
        return [...filtered, result];
      });
    }
    return result;
  } catch (e: any) {
    extensionsError.set(e?.message ?? String(e));
    return null;
  } finally {
    extensionsLoading.set(false);
  }
}

export async function installFromWebStore(url: string): Promise<ExtensionDetails | null> {
  try {
    extensionsLoading.set(true);
    extensionsError.set(null);
    const result = await installExtensionFromUrl(url);
    if (result) {
      installedExtensions.update(list => {
        const filtered = list.filter(e => e.id !== result.id);
        return [...filtered, result];
      });
    }
    return result;
  } catch (e: any) {
    extensionsError.set(e?.message ?? String(e));
    return null;
  } finally {
    extensionsLoading.set(false);
  }
}

export async function installArchive(path?: string): Promise<ExtensionDetails | null> {
  try {
    extensionsLoading.set(true);
    extensionsError.set(null);
    const result = await installExtensionFromArchive(path);
    if (result) {
      installedExtensions.update(list => {
        const filtered = list.filter(e => e.id !== result.id);
        return [...filtered, result];
      });
    }
    return result;
  } catch (e: any) {
    extensionsError.set(e?.message ?? String(e));
    return null;
  } finally {
    extensionsLoading.set(false);
  }
}

export async function toggleExtension(id: string, enabled: boolean): Promise<void> {
  try {
    await toggleNativeExtension(id, enabled);
    installedExtensions.update(list =>
      list.map(ext => (ext.id === id ? { ...ext, enabled } : ext))
    );
  } catch (e: any) {
    extensionsError.set(e?.message ?? String(e));
  }
}

export async function removeExtension(id: string): Promise<void> {
  try {
    await removeNativeExtension(id);
    installedExtensions.update(list => list.filter(ext => ext.id !== id));
  } catch (e: any) {
    extensionsError.set(e?.message ?? String(e));
  }
}

export async function openExtensionsModal(): Promise<void> {
  isExtensionsModalOpen.set(true);
  await setOverlayVisibility(true, "extensions");
  void refreshExtensions();
}

export async function closeExtensionsModal(): Promise<void> {
  isExtensionsModalOpen.set(false);
  await setOverlayVisibility(false, "extensions");
}

export async function toggleExtensionsModal(): Promise<void> {
  if (get(isExtensionsModalOpen)) {
    await closeExtensionsModal();
  } else {
    await openExtensionsModal();
  }
}

export function openExtensionPopup(ext: ExtensionDetails): void {
  if (ext.popup_url) {
    activeExtensionPopup.set({
      id: ext.id,
      name: ext.name,
      popup_url: ext.popup_url,
    });
  }
}

export function closeExtensionPopup(): void {
  activeExtensionPopup.set(null);
}

