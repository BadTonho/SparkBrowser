/**
 * GitHub backups: a second destination next to the sync target in use. The automatic send runs at
 * most once an hour, counted from the last send saved on disk (or the last attempt of this run), and
 * the backend skips it when nothing changed. A pause of the automatic sync pauses it too.
 */
import { get, writable } from "svelte/store";
import { createSharedLifecycle } from "../services/lifecycle.js";
import { capturePreferences } from "../services/backup/preferences";
import { describeError } from "../services/user-errors.js";
import { isTauriRuntime } from "../services/bridge/runtime";
import {
  connectGithubBackup,
  disconnectGithubBackup,
  fetchGithubBackupState,
  listGithubBackupVersions,
  pushGithubBackup,
  reviewGithubBackup,
  reviewGithubBackupVersion,
  setGithubBackupAuto,
} from "../services/bridge/github-backup";
import { backupBusy, reviewFrom, syncPausedNow } from "./backupStore";
import type { GitHubBackupState, SyncVersion } from "../types/backup";

/** A repository is sent to at most this often. */
export const GITHUB_INTERVAL_MS = 60 * 60_000;
const FINISHED_MS = 2_000;

export const githubBackupState = writable<GitHubBackupState>({ target: null, has_token: false, last_synced_at: null, last_error: null });
/** A send or read is running (drawn as the card's ring). */
export const githubRunning = writable(false);
/** The last send ended well a moment ago. */
export const githubFinished = writable(false);
export const githubNotice = writable<"" | "pushed" | "unchanged" | "saved">("");
export const githubError = writable("");
export const githubVersions = writable<SyncVersion[] | null>(null);

let lastAttempt = 0;
let finishedTimer: ReturnType<typeof setTimeout> | undefined;

/** Whether the hourly tick should send now. */
export function githubDue(state: GitHubBackupState, input: { nowMs: number; lastAttemptMs: number; paused: boolean }): boolean {
  if (!state.target?.auto_save || !state.has_token || input.paused) return false;
  const last = Math.max(input.lastAttemptMs, (state.last_synced_at ?? 0) * 1000);
  return input.nowMs - last >= GITHUB_INTERVAL_MS;
}

async function run(work: () => Promise<void>, track = true): Promise<void> {
  if (get(backupBusy) || get(githubRunning) || !isTauriRuntime()) return;
  githubError.set("");
  if (track) githubRunning.set(true);
  try {
    await work();
  } catch (error) {
    githubError.set(describeError(error));
  } finally {
    githubRunning.set(false);
  }
}

function showFinished(): void {
  githubFinished.set(true);
  clearTimeout(finishedTimer);
  finishedTimer = setTimeout(() => githubFinished.set(false), FINISHED_MS);
}

async function send(onlyIfChanged: boolean): Promise<void> {
  const result = await pushGithubBackup(capturePreferences(), onlyIfChanged);
  githubBackupState.set(result.state);
  githubNotice.set(result.skipped_unchanged ? "unchanged" : "pushed");
  showFinished();
}

/** Checks the repository and token on GitHub and saves them; an empty token keeps the saved one. */
export const connectGithub = (repository: string, branch: string, token: string | null, autoSave: boolean) =>
  run(async () => {
    githubBackupState.set(await connectGithubBackup(repository, branch, token, autoSave));
    githubNotice.set("saved");
    lastAttempt = Date.now();
    await send(false);
  });

export const setGithubAuto = (autoSave: boolean) =>
  run(async () => githubBackupState.set(await setGithubBackupAuto(autoSave)), false);

export const disconnectGithub = () =>
  run(async () => {
    githubBackupState.set(await disconnectGithubBackup());
    githubVersions.set(null);
    githubNotice.set("");
  }, false);

/** "Send now": sends even when nothing changed. */
export const sendGithubNow = () =>
  run(async () => {
    lastAttempt = Date.now();
    await send(false);
  });

export const reviewGithub = () =>
  run(() => reviewFrom(() => reviewGithubBackup(capturePreferences())));

export const loadGithubVersions = () =>
  run(async () => githubVersions.set(await listGithubBackupVersions()));

export const reviewGithubVersion = (id: string) =>
  run(() => reviewFrom(() => reviewGithubBackupVersion(id, capturePreferences())));

async function tick(): Promise<void> {
  const now = Date.now();
  if (get(backupBusy) || get(githubRunning)) return;
  if (!githubDue(get(githubBackupState), { nowMs: now, lastAttemptMs: lastAttempt, paused: get(syncPausedNow) })) return;
  lastAttempt = now;
  try {
    githubRunning.set(true);
    await send(true);
  } catch (error) {
    const message = describeError(error);
    githubError.set(message);
    githubBackupState.update((state) => ({ ...state, last_error: message }));
  } finally {
    githubRunning.set(false);
  }
}

function startGithubBackup(): () => void {
  if (!isTauriRuntime()) return () => {};
  let stopped = false;
  fetchGithubBackupState()
    .then((state) => { if (!stopped) githubBackupState.set(state); })
    .catch((error) => githubError.set(describeError(error)));
  const interval = setInterval(() => void tick(), 60_000);
  return () => {
    stopped = true;
    clearInterval(interval);
    clearTimeout(finishedTimer);
  };
}

export const initializeGithubBackup = createSharedLifecycle((scope) => scope.own(startGithubBackup()));
