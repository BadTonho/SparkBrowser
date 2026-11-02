<script lang="ts">
  import { t, formatNumber, type MessageKey } from "../../i18n";
  import type { DownloadTask } from "../../types/browser";
  import { controlDownload, setDownloadChecksum } from "../../services/bridge/downloads";
  import { loadDownloads } from "../../stores/downloadsStore";
  import { addTab } from "../../stores/browserStore";
  import { closeDownloadsDrawer } from "../../stores/downloadsStore";
  import { describeError, UserError } from "../../services/user-errors.js";
  import ErrorMessage from "../status/ErrorMessage.svelte";
  let { task }: { task: DownloadTask } = $props();
  let busy = $state(false); let error = $state(""); let restart = $state(false); let hashText = $state(""); let hashDirty = $state(false);
  $effect(() => { if (!hashDirty) hashText = task.expected_sha256 ?? ""; });
  async function run(action: () => Promise<void>) { if (busy) return; busy = true; error = ""; try { await action(); await loadDownloads(); } catch (e) { error = describeError(e); } finally { busy = false; } }
  const act = (action: "pause" | "resume" | "restart" | "cancel" | "verify") => run(() => controlDownload(task.id, action));
</script>
<article data-testid="download-task" data-task-id={task.id} data-status={task.status}>
  <strong>{task.file_name}</strong><span>{task.url}</span>
  <span>{$t(`common.download_${task.status.toLowerCase()}` as MessageKey)} · {$formatNumber(Math.round(task.received / 1024))} KiB{task.total ? ` / ${$formatNumber(Math.round(task.total / 1024))} KiB` : ""}</span>
  {#if task.status === "Downloading"}{#if task.total}<progress value={task.received} max={task.total} aria-label={$t("downloads.download_progress_984b")}></progress>{:else}<progress aria-label={$t("downloads.downloading_37b3")}></progress>{/if}{/if}
  {#if task.status === "Verifying"}<span>{$t("downloads.verification_progress", { count: $formatNumber(Math.round(task.verification_received / 1024)) })}</span>{/if}
  <ErrorMessage error={task.error ? new UserError(task.error.split(":", 1)[0], task.error) : ""} /><ErrorMessage {error} />
  <div>
    {#if task.can_pause}<button data-testid="download-pause" disabled={busy} onclick={() => act("pause")}>{$t("downloads.pause")}</button>{/if}
    {#if task.can_resume}<button data-testid="download-resume" disabled={busy} onclick={() => act("resume")}>{$t("downloads.resume")}</button>{/if}
    {#if task.can_restart}<button disabled={busy} onclick={() => restart = true}>{$t("downloads.restart")}</button>{/if}
    {#if task.cancellable}<button data-testid="download-cancel" disabled={busy || ["Cancelling", "Pausing"].includes(task.status)} onclick={() => act("cancel")}>{$t("downloads.cancel_1976")}</button>{/if}
    {#if task.source === "native" && task.status === "Interrupted" && task.source_page_url}<button onclick={async () => { await closeDownloadsDrawer(); await addTab(task.source_page_url!); }}>{$t("downloads.source_page")}</button>{/if}
  </div>
  {#if restart}<p>{$t("downloads.restart_confirm")}</p><button disabled={busy} onclick={async () => { await act("restart"); restart = false; }}>{$t("downloads.restart")}</button><button onclick={() => restart = false}>{$t("downloads.cancel_1976")}</button>{/if}
  {#if task.can_verify || task.can_restart}
    <label>{$t("downloads.hash_expected")}<input value={hashText} maxlength="64" oninput={event => { hashText = event.currentTarget.value; hashDirty = true; }} /></label>
    <button disabled={busy} onclick={() => run(async () => { await setDownloadChecksum(task.id, hashText || null); hashDirty = false; })}>{$t("downloads.hash_save")}</button>
  {/if}
  {#if task.can_verify}<button data-testid="download-verify" disabled={busy} onclick={() => act("verify")}>{$t("downloads.verify")}</button>{/if}
  {#if task.actual_sha256}<span class="hash">SHA-256: {task.actual_sha256}</span>{/if}
</article>
<style>article { padding: 12px; margin: 8px 0; display: flex; flex-direction: column; gap: 8px; border: 1px solid var(--border-subtle); border-radius: 8px; font-size: 12px; } span, .hash { overflow-wrap: anywhere; } article > div { display: flex; flex-wrap: wrap; gap: 8px; } button { color: var(--accent-primary); } input { width: 100%; } progress { width: 100%; }</style>
