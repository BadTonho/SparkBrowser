import { writable } from "svelte/store";
import {
  addLockMethod, disableLock, enableLock, eraseLockedProfile, getLockState, removeLockMethod, requestExit,
  unlockWithHello, unlockWithPassword, unlockWithVaultPassword,
} from "../services/bridge/appLock";
import { isTauriRuntime } from "../services/bridge/runtime";
import { lockError } from "../services/appLock/errors.js";
import type { MessageKey } from "../i18n";
import type { LockMethod, LockProof, LockState } from "../types/appLock";

/** How the profile is locked (null until the backend answered). */
export const lockState = writable<LockState | null>(null);
export const lockBusy = writable(false);
/** The last thing that went wrong, with the seconds to wait when the brake is on. */
export const lockFailure = writable<{ key: MessageKey; values: Record<string, unknown>; seconds: number } | null>(null);
/** A short confirmation of the last change made in Settings. */
export const lockNotice = writable<"on" | "off" | "changed" | null>(null);

async function run<T>(work: () => Promise<T>): Promise<T | undefined> {
  lockBusy.set(true);
  lockFailure.set(null);
  lockNotice.set(null);
  try {
    return await work();
  } catch (error) {
    const { key, values, seconds } = lockError(error);
    lockFailure.set({ key: key as MessageKey, values, seconds });
    return undefined;
  } finally {
    lockBusy.set(false);
  }
}

export async function loadLockState(): Promise<void> {
  if (!isTauriRuntime()) return;
  try {
    lockState.set(await getLockState());
  } catch (error) {
    const { key, values, seconds } = lockError(error);
    lockFailure.set({ key: key as MessageKey, values, seconds });
  }
}

/** Opens a locked profile. True when it opened (the browser shows itself); the reason stays in `lockFailure`. */
export async function unlock(how: "password" | "vault" | "hello", secret = ""): Promise<boolean> {
  const done = await run(async () => {
    if (how === "password") await unlockWithPassword(secret);
    else if (how === "vault") await unlockWithVaultPassword(secret);
    else await unlockWithHello();
    return true;
  });
  // A wrong secret moves the brake, so what the screen shows about the wait is read again.
  if (!done) await loadLockState();
  return done === true;
}

export async function quitLocked(): Promise<void> {
  await run(requestExit);
}

/** "Forgot the password": the profile is erased and Spark starts it again; this window ends. */
export async function eraseForgotten(): Promise<boolean> {
  const done = await run(async () => { await eraseLockedProfile(); return true; });
  return done === true;
}

async function change(work: () => Promise<void>, notice: "on" | "off" | "changed"): Promise<boolean> {
  const done = await run(async () => { await work(); return true; });
  await loadLockState();
  if (done) lockNotice.set(notice);
  return done === true;
}

export const turnLockOn = (method: LockMethod, secret?: string) => change(() => enableLock(method, secret), "on");
export const addLockWay = (method: LockMethod, secret: string | undefined, proof: LockProof) => change(() => addLockMethod(method, secret, proof), "changed");
export const removeLockWay = (method: LockMethod, proof: LockProof) => change(() => removeLockMethod(method, proof), "changed");
export const turnLockOff = (proof: LockProof) => change(() => disableLock(proof), "off");
