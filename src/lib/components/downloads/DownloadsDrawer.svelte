<script lang="ts">
  import DownloadTaskRow from "./DownloadTaskRow.svelte";
  import DownloadHistoryRow from "./DownloadHistoryRow.svelte";
  import DownloadIcon from "./DownloadIcon.svelte";
  import { tick } from "svelte";
  import ErrorMessage from "../status/ErrorMessage.svelte";
  import { t } from "../../i18n";
  import { focusDialog } from "../../actions/focusDialog";
  import { isTauriRuntime } from "../../services/bridge/runtime";
  import { panelPosition, isPanelTrigger } from "../../services/panel-presentation";
  import { mountDownloadsHost } from "./downloads-host";
  import { downloads, downloadTasks, downloadsLoadError, downloadsSession, isDownloadsDrawerOpen, closeDownloadsDrawer, startDownloadFromUrl, wipeDownloads, acknowledgeDownloadNotices } from "../../stores/downloadsStore";
  import { currentUrl } from "../../stores/browserStore";
  import { resetDownloadVisuals } from "../../stores/downloadVisuals";
  import "./downloads.css";

  const native = isTauriRuntime();
  let downloadUrlInput = $state("");
  let formOpen = $state(false);
  let root = $state<HTMLElement>();
  let isDownloading = $state(false);
  let downloadError = $state("");
  let viewport = $state({ width: innerWidth, height: innerHeight });
  let anchor = $state({ left: 0, top: 0, width: 0, height: 0 });
  const position = $derived(panelPosition(anchor, viewport.width, viewport.height, { width: 480, maxHeight: 560 }));
  const history = $derived($downloads.filter(item => !$downloadTasks.some(task => task.save_path === item.save_path)));
  const clearable = $derived($downloads.length > 0 || $downloadTasks.some(task => ["Completed", "Cancelled", "Failed"].includes(task.status)));
  $effect.pre(() => {
    if ($downloadsSession) resetDownloadVisuals();
    formOpen = false;
  });
  async function toggleForm() {
    formOpen = !formOpen;
    if (!native) { await tick(); root?.querySelector<HTMLElement>(formOpen ? '[data-testid="download-url"]' : '[data-testid="download-add-toggle"]')?.focus(); }
  }
  function host(node: HTMLElement) {
    const stop = mountDownloadsHost(node, native);
    const focus = native ? undefined : focusDialog(node, { onEscape: () => closeDownloadsDrawer(true) });
    if (!native) queueMicrotask(() => node.querySelector<HTMLElement>('[data-testid="download-add-toggle"]')?.focus());
    let frame = 0;
    const measure = () => {
      const r = $downloadsSession?.anchor.getBoundingClientRect();
      if (r && (anchor.left !== r.left || anchor.top !== r.top || anchor.width !== r.width || anchor.height !== r.height)) anchor = { left: r.left, top: r.top, width: r.width, height: r.height };
      frame = requestAnimationFrame(measure);
    };
    if (!native) measure();
    return { destroy() { cancelAnimationFrame(frame); stop(); focus?.destroy(); } };
  }
  async function handleQuickDownload(event: SubmitEvent) {
    event.preventDefault();
    const target = downloadUrlInput.trim() || $currentUrl;
    if (!target || isDownloading) return;
    isDownloading = true; downloadError = "";
    let started = false;
    try {
      await startDownloadFromUrl(target); downloadUrlInput = ""; formOpen = false;
      started = true;
    }
    catch { downloadError = $t("downloads.link_failed"); }
    finally {
      isDownloading = false;
      if (started && !native) { await tick(); root?.querySelector<HTMLElement>('[data-testid="download-add-toggle"]')?.focus(); }
    }
  }
  async function clear() { try { await wipeDownloads(); } catch { downloadError = $t("downloads.action_failed"); } }
</script>

<svelte:window onresize={() => { viewport = { width: innerWidth, height: innerHeight }; }} />
{#if $isDownloadsDrawerOpen}
  <div class="downloads-dismiss" class:downloads-controller={native} aria-hidden={native ? "true" : undefined} role="presentation"
    onclick={event => {
      if (event.target !== event.currentTarget) return;
      const trigger = isPanelTrigger(anchor, event.clientX, event.clientY);
      if (trigger) acknowledgeDownloadNotices();
      void closeDownloadsDrawer(trigger);
    }}>
    <div class="downloads-surface" style:left="{position.left}px" style:top="{position.top}px" style:width="{position.width}px" style:max-height="{position.maxHeight}px">
      <div class="downloads-panel" data-testid="downloads-dialog" role="dialog" aria-modal="false" aria-label={$t("downloads.title")} tabindex="-1" bind:this={root} use:host>
        <header class="downloads-header">
          <h2>{$t("downloads.title")}</h2>
          <div class="downloads-header-actions">
            <button class="download-icon-button" data-testid="download-add-toggle" disabled={isDownloading} onclick={toggleForm} aria-label={$t("downloads.add_link")} title={$t("downloads.add_link")} aria-expanded={formOpen} aria-controls="download-link-form"><DownloadIcon name="search" /></button>
            {#if clearable}<button class="download-icon-button" data-testid="downloads-clear" onclick={clear} aria-label={$t("downloads.clear_list_a2a6")} title={$t("downloads.clear_list_a2a6")}><DownloadIcon name="trash" /></button>{/if}
            <button class="download-icon-button" data-testid="close-downloads" onclick={() => closeDownloadsDrawer(true)} aria-label={$t("downloads.close")} title={$t("downloads.close")}><DownloadIcon name="close" /></button>
          </div>
        </header>
        {#if formOpen}<form id="download-link-form" class="quick-download" onsubmit={handleQuickDownload}>
          <div class="quick-download-row">
            <input data-testid="download-url" type="text" bind:value={downloadUrlInput} aria-label={$t("downloads.file_url")} placeholder={$t("downloads.link_placeholder")} spellcheck="false" />
            <button type="submit" data-testid="download-start" class="download-start" disabled={isDownloading}>{isDownloading ? $t("downloads.downloading_37b3") + "…" : $t("downloads.download_d6ea")}</button>
          </div>
        </form>{/if}
        {#if downloadError || $downloadsLoadError}<ErrorMessage error={downloadError || $t("downloads.load_failed")} />{/if}
        <div class="downloads-list">
          {#each $downloadTasks as task (task.id)}<DownloadTaskRow {task} />{/each}
          {#each history as item (item.id)}<DownloadHistoryRow {item} />{/each}
          {#if !$downloadTasks.length && !history.length}
            <div class="downloads-empty"><svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M12 3v12m-4-4 4 4 4-4M5 16v4h14v-4" /></svg>
              <p>{$t("downloads.no_downloads_recorded_yet_5b64")}</p><span>{$t("downloads.empty_hint")}</span>
            </div>
          {/if}
        </div>
      </div>
    </div>
  </div>
{/if}
