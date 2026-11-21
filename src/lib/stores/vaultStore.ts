import { get, writable } from "svelte/store";
import { listen } from "@tauri-apps/api/event";
import { isTauriRuntime } from "../services/bridge/runtime";
import {
  changeVaultPassword, copyVaultPassword, copyVaultText, createVault, deleteVaultEntry, disableVaultHello, disableVaultPassword,
  enableVaultHello, enableVaultPassword, exportVaultCsv, getVaultStatus, importVaultCsv, listVaultEntries, lockVault, resetVault,
  revealVaultEntry, saveVaultEntry, saveVaultRecoveryFile, setVaultAutoLock, setVaultRequireHello, unlockVault, unlockVaultWithHello, unlockVaultWithRecovery,
} from "../services/bridge/vault";
import { vaultError } from "../services/vault/errors.js";
import { logIgnored } from "../services/ignored-error";
import type { MessageKey } from "../i18n";
import type { VaultEntryData, VaultRevealed, VaultStatus, VaultSummary } from "../types/vault";

/** What the vault is doing right now. Nothing here ever holds a password, except while the user edits one. */
export const vaultStatus = writable<VaultStatus | null>(null);
export const vaultEntries = writable<VaultSummary[]>([]);
export const vaultUnreadable = writable(0);
export const vaultBusy = writable(false);
export const vaultFailure = writable<{ key: MessageKey; values: Record<string, unknown> } | null>(null);
/** The recovery key of a vault just created: shown once, then dropped. */
export const vaultRecoveryKey = writable<string | null>(null);
export type VaultNotice = "password_changed" | "hello_on" | "hello_off" | "password_on" | "password_off";
export const vaultNotice = writable<VaultNotice | null>(null);
/** The outcome of an import or export, or of changing a setting: a message with its values. */
export const vaultInfo = writable<{ key: MessageKey; values: Record<string, unknown> } | null>(null);

function fail(error: unknown): undefined {
  const { key, values } = vaultError(error);
  vaultFailure.set({ key: key as MessageKey, values });
  return undefined;
}

/**
 * Runs one backend call. A user action clears the previous message first; a refresh that follows it
 * (`keepFailure`) must not, or the error of that action would vanish before it is seen.
 */
async function run<T>(work: () => Promise<T>, keepFailure = false): Promise<T | undefined> {
  vaultBusy.set(true);
  if (!keepFailure) vaultFailure.set(null);
  try {
    return await work();
  } catch (error) {
    return fail(error);
  } finally {
    vaultBusy.set(false);
  }
}

function forgetEntries(): void {
  vaultEntries.set([]);
  vaultUnreadable.set(0);
}

export async function refreshVaultStatus(): Promise<void> {
  if (!isTauriRuntime()) return;
  const status = await run(getVaultStatus, true);
  if (!status) return;
  vaultStatus.set(status);
  if (status.unlocked) await loadVaultEntries();
  else forgetEntries();
}

export async function loadVaultEntries(): Promise<void> {
  const listing = await run(listVaultEntries, true);
  if (!listing) {
    await syncLockState();
    return;
  }
  vaultEntries.set(listing.entries);
  vaultUnreadable.set(listing.unreadable);
}

/** A "locked" answer (for example after the idle time) turns the screen back to the unlock form. */
async function syncLockState(): Promise<void> {
  const failure = get(vaultFailure);
  if (failure?.key !== "passwords.error_locked") return;
  forgetEntries();
  vaultStatus.update(status => (status ? { ...status, unlocked: false } : status));
  vaultFailure.set(null);
}

/** `masterPassword` is null for a vault that opens with Windows Hello only. */
export async function createNewVault(masterPassword: string | null, useHello: boolean): Promise<boolean> {
  const recovery = await run(() => createVault(masterPassword, useHello));
  if (recovery === undefined) return false;
  vaultRecoveryKey.set(recovery);
  await refreshVaultStatus();
  return true;
}

/** Saves the recovery key just shown to a text file. True when a file was written (not when cancelled). */
export async function saveRecoveryKeyFile(recoveryKey: string): Promise<boolean> {
  return (await run(() => saveVaultRecoveryFile(recoveryKey))) === true;
}

export function dismissRecoveryKey(): void {
  vaultRecoveryKey.set(null);
}

export async function unlockWithPassword(password: string): Promise<boolean> {
  const done = await run(async () => { await unlockVault(password); return true; });
  await refreshVaultStatus();
  return done === true;
}

export async function unlockWithHello(): Promise<boolean> {
  const done = await run(async () => { await unlockVaultWithHello(); return true; });
  await refreshVaultStatus();
  return done === true;
}

export async function unlockWithRecoveryKey(recoveryKey: string): Promise<boolean> {
  const done = await run(async () => { await unlockVaultWithRecovery(recoveryKey); return true; });
  await refreshVaultStatus();
  return done === true;
}

export async function lockNow(): Promise<void> {
  await run(lockVault);
  vaultRecoveryKey.set(null);
  await refreshVaultStatus();
}

export async function revealEntry(id: string): Promise<VaultRevealed | undefined> {
  const entry = await run(() => revealVaultEntry(id));
  if (!entry) await syncLockState();
  return entry;
}

/** Copies an entry's password without it ever reaching the interface. */
export async function copyEntryPassword(id: string): Promise<boolean> {
  const done = await run(async () => { await copyVaultPassword(id); return true; });
  if (!done) await syncLockState();
  return done === true;
}

/** Copies a username or the recovery key; the backend clears the clipboard the same way. */
export async function copyVaultSecret(text: string): Promise<boolean> {
  const done = await run(async () => { await copyVaultText(text); return true; });
  return done === true;
}

export async function saveEntry(id: string | null, data: VaultEntryData): Promise<boolean> {
  const saved = await run(() => saveVaultEntry(id, data));
  if (!saved) {
    await syncLockState();
    return false;
  }
  await refreshVaultStatus();
  return true;
}

export async function removeEntry(id: string): Promise<void> {
  const done = await run(async () => { await deleteVaultEntry(id); return true; });
  if (!done) await syncLockState();
  await refreshVaultStatus();
}

export async function changeMasterPassword(current: string, next: string): Promise<boolean> {
  const done = await run(async () => { await changeVaultPassword(current, next); return true; });
  if (done) vaultNotice.set("password_changed");
  else await syncLockState();
  return done === true;
}

/** Runs a change to the unlock methods, then shows what it did (or why it could not). */
async function changeMethod(work: () => Promise<void>, notice: VaultNotice): Promise<boolean> {
  vaultNotice.set(null);
  const done = await run(async () => { await work(); return true; });
  if (done) vaultNotice.set(notice);
  else await syncLockState();
  await refreshVaultStatus();
  return done === true;
}

export const turnHelloOn = () => changeMethod(enableVaultHello, "hello_on");
export const turnHelloOff = () => changeMethod(disableVaultHello, "hello_off");
export const turnPasswordOn = (newPassword: string) => changeMethod(() => enableVaultPassword(newPassword), "password_on");
export const turnPasswordOff = (current: string) => changeMethod(() => disableVaultPassword(current), "password_off");

/** Asks for Windows Hello every time a password is shown, copied or exported. The screen follows the backend's answer. */
export async function turnRequireHello(on: boolean): Promise<void> {
  vaultInfo.set(null);
  const done = await run(async () => { await setVaultRequireHello(on); return true; });
  if (!done) await syncLockState();
  await refreshVaultStatus();
}

/** The backend opens the dialog and reads the file; a cancelled dialog changes nothing. */
export async function importPasswords(): Promise<void> {
  vaultInfo.set(null);
  const result = await run(importVaultCsv);
  if (result) vaultInfo.set({ key: "passwords.import_result" as MessageKey, values: { ...result } });
  else if (result === undefined) await syncLockState();
  await refreshVaultStatus();
}

/** `currentPassword` confirms the person; null means Windows Hello does. True when a file was written. */
export async function exportPasswords(currentPassword: string | null): Promise<boolean> {
  vaultInfo.set(null);
  const count = await run(() => exportVaultCsv(currentPassword));
  if (typeof count === "number") vaultInfo.set({ key: "passwords.export_done" as MessageKey, values: { count } });
  else if (count === undefined) await syncLockState();
  return typeof count === "number";
}

export async function chooseAutoLock(minutes: number): Promise<void> {
  const done = await run(async () => { await setVaultAutoLock(minutes); return true; });
  if (!done) await syncLockState();
  await refreshVaultStatus();
}

export async function eraseVault(): Promise<void> {
  await run(resetVault);
  vaultRecoveryKey.set(null);
  await refreshVaultStatus();
}

export function dismissVaultMessages(): void {
  vaultFailure.set(null);
  vaultNotice.set(null);
  vaultInfo.set(null);
}

/** Keeps the screen in step when the backend locks the vault by itself (idle time). */
export function initializeVaultEvents(): () => void {
  if (!isTauriRuntime()) return () => {};
  let disposed = false;
  let stop: (() => void) | undefined;
  listen("vault-locked", () => { forgetEntries(); void refreshVaultStatus(); })
    .then(unlisten => { if (disposed) unlisten(); else stop = unlisten; })
    .catch(logIgnored);
  return () => { disposed = true; stop?.(); };
}
