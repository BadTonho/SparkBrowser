import { get, writable } from "svelte/store";
import { cancelNextcloudLogin, pollNextcloudLogin, startNextcloudLogin } from "../services/bridge/backup";
import { describeError } from "../services/user-errors.js";
import { logIgnored } from "../services/ignored-error";
import { addTab, removeTab, tabs } from "./browserStore";
import { adoptSyncState, openDataBackup } from "./backupStore";
import { closeSettingsModal } from "./settingsStore";

export type NextcloudLoginState =
  | { status: "idle" }
  | { status: "waiting" }
  | { status: "error"; error: string };

/** The Nextcloud sign-in in progress, shown by the sync settings. */
export const nextcloudLogin = writable<NextcloudLoginState>({ status: "idle" });

/** The server's login page is checked this often while the person approves Spark. */
const POLL_MS = 2000;

let flowId: number | null = null;
let loginTabId: string | null = null;
let timer: ReturnType<typeof setTimeout> | undefined;

function stop(): void {
  if (timer) clearTimeout(timer);
  timer = undefined;
  flowId = null;
}

function fail(error: unknown): void {
  const tab = loginTabId;
  stop();
  loginTabId = null;
  nextcloudLogin.set({ status: "error", error: describeError(error) });
  if (tab) void openDataBackup();
}

async function poll(id: number, autoSave: boolean, applyPreferences: boolean): Promise<void> {
  timer = undefined;
  if (flowId !== id) return;
  // Closing the login tab gives up the sign-in.
  if (loginTabId && !get(tabs).some((tab) => tab.id === loginTabId)) {
    cancelNextcloudSignIn();
    return;
  }
  try {
    const state = await pollNextcloudLogin(id, autoSave, applyPreferences);
    if (flowId !== id) return;
    if (!state) {
      timer = setTimeout(() => void poll(id, autoSave, applyPreferences), POLL_MS);
      return;
    }
    stop();
    const tab = loginTabId;
    loginTabId = null;
    nextcloudLogin.set({ status: "idle" });
    adoptSyncState(state);
    if (tab && get(tabs).some((item) => item.id === tab)) await removeTab(tab);
    openDataBackup();
  } catch (error) {
    if (flowId === id) fail(error);
  }
}

/**
 * Signs in to the Nextcloud server at `server`: its login page opens in a tab, and once the person
 * approves Spark there, the account's "Spark" folder becomes the sync target.
 */
export async function signInToNextcloud(server: string, autoSave: boolean, applyPreferences: boolean): Promise<void> {
  cancelNextcloudSignIn();
  nextcloudLogin.set({ status: "waiting" });
  try {
    const start = await startNextcloudLogin(server);
    flowId = start.flowId;
    await closeSettingsModal();
    await addTab(start.loginUrl);
    loginTabId = get(tabs).find((tab) => tab.is_active && !tab.is_ephemeral)?.id ?? null;
    const id = start.flowId;
    timer = setTimeout(() => void poll(id, autoSave, applyPreferences), POLL_MS);
  } catch (error) {
    fail(error);
  }
}

export function cancelNextcloudSignIn(): void {
  const waiting = flowId !== null;
  stop();
  loginTabId = null;
  nextcloudLogin.set({ status: "idle" });
  if (waiting) cancelNextcloudLogin().catch(logIgnored);
}
