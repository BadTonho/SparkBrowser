<script lang="ts">
  import { t, formatNumber, type MessageKey } from "../../i18n";
  import type { DownloadTask, DownloadVisual } from "../../types/browser";
  import { controlDownload } from "../../services/bridge/downloads";
  import { loadDownloads, closeDownloadsDrawer } from "../../stores/downloadsStore";
  import { loadDownloadVisual } from "../../stores/downloadVisuals";
  import { downloadTaskSize, downloadStatusIcon } from "../../services/download-visual";
  import { addTab } from "../../stores/browserStore";
  import DownloadFilePreview from "./DownloadFilePreview.svelte";
  import DownloadIcon from "./DownloadIcon.svelte";
  let { task }: { task: DownloadTask } = $props();
  let busy = $state(false), error = $state(false), restart = $state(false);
  let visual = $state<DownloadVisual | null>(null);
  const statusIcon = $derived(error ? "error" : downloadStatusIcon(task.status));
  const statusLabel = $derived(error ? $t("downloads.action_failed") : $t(`common.download_${task.status.toLowerCase()}` as MessageKey));
  $effect(() => {
    let active = true; visual = null;
    if (task.status === "Completed") void loadDownloadVisual({ kind: "task", id: task.id }, String(task.revision)).then(value => { if (active) visual = value; });
    return () => { active = false; };
  });
  async function act(action: "pause" | "resume" | "restart" | "cancel") {
    if (busy) return;
    busy = true; error = false;
    try { await controlDownload(task.id, action); await loadDownloads(); restart = false; }
    catch { error = true; }
    finally { busy = false; }
  }
</script>
<article class="download-row" data-testid="download-task" data-task-id={task.id} data-status={task.status}>
  <DownloadFilePreview name={task.file_name} thumbnail={visual?.thumbnail_png} />
  <div class="download-copy"><strong class="download-name" title={task.file_name}>{task.file_name}</strong>
    <span class="download-size">{downloadTaskSize(task, $formatNumber, $t("downloads.size_unknown"))}</span></div>
  <div class="download-row-actions">
    <span class="download-state {statusIcon}" role="img" aria-label={statusLabel} title={statusLabel}><DownloadIcon name={statusIcon} size={14} /></span>
    {#if task.can_pause}<button class="download-icon-button" data-testid="download-pause" disabled={busy} onclick={() => act("pause")} aria-label={$t("downloads.pause")} title={$t("downloads.pause")}><DownloadIcon name="pause" /></button>{/if}
    {#if task.can_resume}<button class="download-icon-button" data-testid="download-resume" disabled={busy} onclick={() => act("resume")} aria-label={$t("downloads.resume")} title={$t("downloads.resume")}><DownloadIcon name="resume" /></button>{/if}
    {#if task.can_restart}<button class="download-icon-button" data-testid="download-restart" disabled={busy} onclick={() => restart = !restart} aria-label={$t("downloads.restart")} title={$t("downloads.restart")} aria-expanded={restart}><DownloadIcon name="restart" /></button>{/if}
    {#if task.cancellable}<button class="download-icon-button" data-testid="download-cancel" disabled={busy || ["Cancelling", "Pausing"].includes(task.status)} onclick={() => act("cancel")} aria-label={$t("downloads.cancel_1976")} title={$t("downloads.cancel_1976")}><DownloadIcon name="close" /></button>{/if}
    {#if task.source === "native" && task.status === "Interrupted" && task.source_page_url}<button class="download-icon-button" onclick={async () => { await closeDownloadsDrawer(); await addTab(task.source_page_url!); }} aria-label={$t("downloads.source_page")} title={$t("downloads.source_page")}><DownloadIcon name="source" /></button>{/if}
  </div>
  {#if restart}<div class="download-restart-confirm"><p>{$t("downloads.restart_confirm")}</p><button disabled={busy} onclick={() => act("restart")}>{$t("downloads.restart")}</button><button onclick={() => restart = false}>{$t("downloads.cancel_1976")}</button></div>{/if}
  {#if error}<span class="download-sr-only" role="alert">{$t("downloads.action_failed")}</span>{/if}
</article>
