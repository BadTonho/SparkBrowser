import { get, writable } from "svelte/store";
import { createProfile, deleteProfile, getLauncherPrefs, listProfiles, openProfile, renameProfile, setAskProfileAtStartup } from "../services/bridge/profiles";
import { isTauriRuntime } from "../services/bridge/runtime";
import { profileError } from "../services/profile/errors.js";
import type { MessageKey } from "../i18n";
import type { CreateProfileForm, ProfileInfo, ProfileList } from "../types/profiles";
import { logIgnored } from "../services/ignored-error";

/** The profiles on this computer and which one this window runs (null until the backend answered). */
export const profiles = writable<ProfileList | null>(null);
export const profilesBusy = writable(false);
/** The last thing that went wrong, as a message key; cleared by the next action. */
export const profilesFailure = writable<MessageKey | null>(null);
/** A short confirmation of the last successful action. */
export const profilesNotice = writable<"opened" | null>(null);

async function run<T>(work: () => Promise<T>): Promise<T | undefined> {
  profilesBusy.set(true);
  profilesFailure.set(null);
  profilesNotice.set(null);
  try {
    return await work();
  } catch (error) {
    profilesFailure.set(profileError(error).key as MessageKey);
    return undefined;
  } finally {
    profilesBusy.set(false);
  }
}

export async function loadProfiles(): Promise<void> {
  if (!isTauriRuntime()) return;
  const list = await run(listProfiles);
  if (list) profiles.set(list);
}

async function reload(): Promise<void> {
  try {
    profiles.set(await listProfiles());
  } catch (error) {
    profilesFailure.set(profileError(error).key as MessageKey);
  }
}

export async function addProfile(form: CreateProfileForm): Promise<ProfileInfo | undefined> {
  const created = await run(() => createProfile(form));
  if (created) await reload();
  return created;
}

export async function submitProfile(form: CreateProfileForm): Promise<MessageKey | null> {
  const created = await addProfile(form);
  const failure = get(profilesFailure);
  // Errors from the form are displayed by its own dialog, even on projected settings surfaces.
  if (!created) {
    logIgnored(new Error(`Creating a profile from Settings failed (${failure ?? "profiles.error_failed"})`));
    profilesFailure.set(null);
  }
  return created ? null : (failure ?? "profiles.error_failed");
}

export async function changeProfileName(id: string, name: string): Promise<boolean> {
  const renamed = await run(() => renameProfile(id, name));
  if (renamed) await reload();
  return renamed !== undefined;
}

export async function removeProfile(id: string): Promise<boolean> {
  const done = await run(async () => { await deleteProfile(id); return true; });
  await reload();
  return done === true;
}

export async function openProfileWindow(id: string): Promise<boolean> {
  const done = await run(async () => { await openProfile(id); return true; });
  if (done) profilesNotice.set("opened");
  return done === true;
}

export function currentProfile(): ProfileInfo | undefined {
  const list = get(profiles);
  return list?.items.find(item => item.id === list.current);
}

/** Ask which profile to open every time Spark starts. */
export const askProfileAtStartup = writable(false);

export async function loadStartupPrefs(): Promise<void> {
  if (!isTauriRuntime()) return;
  try {
    askProfileAtStartup.set((await getLauncherPrefs()).ask_every_time);
  } catch (error) {
    profilesFailure.set(profileError(error).key as MessageKey);
  }
}

export async function changeAskAtStartup(enabled: boolean): Promise<void> {
  const done = await run(async () => { await setAskProfileAtStartup(enabled); return true; });
  askProfileAtStartup.set(done ? enabled : get(askProfileAtStartup));
  // Which profile opens at start-up is shown on the list, and it depends on this choice.
  if (done) await reload();
}
