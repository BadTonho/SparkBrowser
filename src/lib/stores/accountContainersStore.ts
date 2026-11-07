import { writable } from "svelte/store";
import { invoke, isTauriRuntime } from "../services/bridge/runtime";
import { applyTabsSnapshot } from "./browserStore";
import type { TabSnapshot } from "../types/browser";

export interface AccountContainer { id: string; name: string; deleting: boolean }
export const accountContainers = writable<AccountContainer[]>([]);
let loading: Promise<void> | null = null;
export function loadAccountContainers(): Promise<void> {
  if (!isTauriRuntime()) return Promise.resolve();
  if (!loading) loading = invoke<AccountContainer[]>("get_account_containers").then(value => accountContainers.set(value)).finally(() => { loading = null; });
  return loading;
}
export async function createAccountContainer(name: string): Promise<void> {
  accountContainers.set(await invoke<AccountContainer[]>("create_account_container", { name }));
}
export async function renameAccountContainer(id: string, name: string): Promise<void> {
  accountContainers.set(await invoke<AccountContainer[]>("rename_account_container", { id, name }));
}
export async function deleteAccountContainer(id: string): Promise<void> {
  try { accountContainers.set(await invoke<AccountContainer[]>("delete_account_container", { id })); }
  catch (error) { await loadAccountContainers(); throw error; }
}
export async function setTabAccountContainer(tabId: string, containerId: string | null): Promise<void> {
  applyTabsSnapshot(await invoke<TabSnapshot>("set_tab_account_container", { tabId, containerId }));
}
export async function createContainerTab(id: string): Promise<void> {
  applyTabsSnapshot(await invoke<TabSnapshot>("create_container_tab", { id }));
}
