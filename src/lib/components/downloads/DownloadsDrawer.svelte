<script lang="ts">
  import DownloadTaskCard from "./DownloadTaskCard.svelte";
  import ErrorMessage from "../status/ErrorMessage.svelte";
  import { describeError } from "../../services/user-errors.js";
  import { t, type MessageKey } from "../../i18n";
  import { focusDialog } from "../../actions/focusDialog";
  import { isTauriRuntime } from "../../services/bridge/runtime";
  import { panelPosition, isPanelTrigger } from "../../services/panel-presentation";
  import { mountDownloadsHost } from "./downloads-host";
  import { downloads, downloadTasks, downloadsLoadError, downloadsSession, isDownloadsDrawerOpen, closeDownloadsDrawer, startDownloadFromUrl, wipeDownloads, acknowledgeDownloadNotices } from "../../stores/downloadsStore";
  import { currentUrl } from "../../stores/browserStore";
  import "./downloads.css";

  const native = isTauriRuntime();
  let downloadUrlInput = $state("");
  let expectedHash = $state("");
  let isDownloading = $state(false);
  let downloadError = $state("");
  let viewport = $state({ width: innerWidth, height: innerHeight });
  let anchor = $state({ left: 0, top: 0, width: 0, height: 0 });
  const position = $derived(panelPosition(anchor, viewport.width, viewport.height, { width: 480, maxHeight: 560 }));
  const history = $derived($downloads.filter(item => !$downloadTasks.some(task => task.save_path === item.save_path)));
  const clearable = $derived($downloads.length > 0 || $downloadTasks.some(task => ["Completed", "Cancelled", "Failed"].includes(task.status)));
  function host(node: HTMLElement) {
    const stop = mountDownloadsHost(node, native);
    const focus = native ? undefined : focusDialog(node, { onEscape: () => closeDownloadsDrawer(true) });
    if (!native) queueMicrotask(() => node.querySelector<HTMLInputElement>('[data-testid="download-url"]')?.focus());
    let frame = 0;
    const measure = () => {
      const r = $downloadsSession?.anchor.getBoundingClientRect();
      if (r && (anchor.left !== r.left || anchor.top !== r.top || anchor.width !== r.width || anchor.height !== r.height)) anchor = { left: r.left, top: r.top, width: r.width, height: r.height };
      frame = requestAnimationFrame(measure);
    };
    measure();
    return { destroy() { cancelAnimationFrame(frame); stop(); focus?.destroy(); } };
  }
  async function handleQuickDownload(event: SubmitEvent) {
    event.preventDefault();
    const target = downloadUrlInput.trim() || $currentUrl;
    if (!target || isDownloading) return;
    isDownloading = true; downloadError = "";
    try { await startDownloadFromUrl(target, expectedHash || null); expectedHash = ""; downloadUrlInput = ""; }
    catch (error) { downloadError = describeError(error); }
    finally { isDownloading = false; }
  }
  async function clear() { try { await wipeDownloads(); } catch (error) { downloadError = describeError(error); } }
  function downloadStatus(status: string) { return $t(`common.download_${status.toLowerCase()}` as MessageKey); }
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
      <div class="downloads-panel" data-testid="downloads-dialog" role="dialog" aria-modal="false" aria-label={$t("downloads.download_manager_160a")} tabindex="-1" use:host>
        <header class="downloads-header">
          <h2>{$t("downloads.download_manager_160a")}</h2>
          <div class="downloads-header-actions">
            {#if clearable}<button class="clear-downloads" onclick={clear}>{$t("downloads.clear_list_a2a6")}</button>{/if}
            <button class="downloads-close" data-testid="close-downloads" onclick={() => closeDownloadsDrawer(true)} aria-label={$t("downloads.close")} title={$t("downloads.close")}>
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" aria-hidden="true" focusable="false"><path d="m6 6 12 12M18 6 6 18" /></svg>
            </button>
          </div>
        </header>
        <form class="quick-download" onsubmit={handleQuickDownload}>
          <div class="quick-download-row">
            <input data-testid="download-url" type="text" bind:value={downloadUrlInput} aria-label={$t("downloads.file_url")} placeholder={$t("downloads.paste_direct_file_url_or_leave_empty_to_save_active_page_ff6b")} spellcheck="false" />
            <button type="submit" data-testid="download-start" class="download-start" disabled={isDownloading}>{isDownloading ? $t("downloads.downloading_37b3") + "…" : $t("downloads.download_d6ea")}</button>
          </div>
          <details><summary>{$t("downloads.integrity_options")}</summary>
            <input data-testid="download-expected-hash" aria-label={$t("downloads.hash_expected")} placeholder={$t("downloads.hash_expected")} bind:value={expectedHash} maxlength="64" />
          </details>
        </form>
        {#if downloadError || $downloadsLoadError}<ErrorMessage error={downloadError || $downloadsLoadError} />{/if}
        <div class="downloads-list">
          {#each $downloadTasks as task (task.id)}<DownloadTaskCard {task} />{/each}
          {#each history as item (item.id)}
            <article class="download-history">
              <strong title={item.file_name}>{item.file_name}</strong><span class="download-path" title={item.save_path}>{item.save_path}</span>
              <span class="download-status">{downloadStatus(item.status)}</span>
            </article>
          {/each}
          {#if !$downloadTasks.length && !history.length}
            <div class="downloads-empty"><svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M12 3v12m-4-4 4 4 4-4M5 16v4h14v-4" /></svg>
              <p>{$t("downloads.no_downloads_recorded_yet_5b64")}</p><span>{$t("downloads.press_download_above_to_save_a_resource_to_your_os_downloads_folder_f5e0")}</span>
            </div>
          {/if}
        </div>
      </div>
    </div>
  </div>
{/if}
