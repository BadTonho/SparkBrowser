<script lang="ts">
  import { untrack } from "svelte";
  import { isTauriRuntime } from "../../services/bridge/runtime";
  import { panelPosition } from "../../services/panel-presentation";
  import { mountPanelHost } from "../panels/panel-host";
  import { onVideoDownloadPopoverAction, setVideoDownloadPopover } from "../../services/bridge/video-download";
  import { t, type MessageKey } from "../../i18n";
  import { tabs } from "../../stores/browserStore";
  import { closeVideoDownload, isVideoDownloadOpen, videoDownloadSession } from "../../stores/videoDownloadStore";
  import { openDownloadsDrawer } from "../../stores/downloadsStore";
  import { downloadVideo, scanPageVideos, type VideoScan, type VideoSource } from "../../services/bridge/video-download";
  import { focusDialog } from "../../actions/focusDialog";
  import { logIgnored } from "../../services/ignored-error";
  import { videoOptions } from "../../services/video-download-options";
  import "./video-download.css";

  const problems: Record<string, MessageKey> = {
    "video.encrypted": "video_download.problem_encrypted",
    "video.live": "video_download.problem_live",
    "video.separate_audio": "video_download.problem_audio",
    "video.byterange": "video_download.problem_format",
    "video.unsupported": "video_download.problem_format",
    "video.invalid_playlist": "video_download.problem_format",
    "video.too_many_segments": "video_download.problem_too_large",
    "video.too_large": "video_download.problem_too_large",
    "video.unavailable": "video_download.problem_platform",
    "media.unavailable": "video_download.problem_page",
    "downloads.blocked": "video_download.problem_blocked",
    "downloads.http_error": "video_download.problem_server",
    "downloads.network_error": "video_download.problem_server",
    "downloads.queue_full": "video_download.problem_queue",
  };
  const problemKey = (code: unknown): MessageKey => problems[String(code)] ?? "video_download.failed";

  const active = $derived($tabs.find(tab => tab.is_active));
  // The page the list belongs to: another tab or address makes it stale.
  const page = $derived(active ? `${active.id}\n${active.url}` : "");
  let openedFor = "";
  let tabId = "";
  let scan = $state<VideoScan | null>(null);
  let scanError = $state<MessageKey | null>(null);
  let states = $state<Record<string, "starting" | "started" | MessageKey>>({});
  let generation = 0;

  async function search() {
    const current = ++generation;
    scan = null; scanError = null; states = {};
    if (!active || !/^https?:/i.test(active.url)) { scanError = "video_download.http_only"; return; }
    tabId = active.id;
    try {
      const result = await scanPageVideos(tabId);
      if (current === generation) scan = result;
    } catch (error) {
      if (current === generation) scanError = problemKey(error);
    }
  }

  $effect(() => {
    if (!$isVideoDownloadOpen) { openedFor = ""; generation++; return; }
    const current = page;
    untrack(() => {
      if (!openedFor) { openedFor = current; void search(); }
      else if (openedFor !== current) void closeVideoDownload();
    });
  });

  async function save(source: VideoSource) {
    if (states[source.url] === "starting") return;
    const current = generation;
    states[source.url] = "starting";
    try {
      const task = await downloadVideo(tabId, source);
      if (current !== generation) return;
      if (task) states[source.url] = "started";
      else delete states[source.url];
    } catch (error) {
      if (current === generation) states[source.url] = problemKey(error);
    }
  }

  async function showDownloads() {
    await closeVideoDownload();
    await openDownloadsDrawer().catch(logIgnored);
  }

  function describe(source: VideoSource) {
    const parts = [$t(source.kind === "hls" ? "video_download.kind_stream" : "video_download.kind_file")];
    if (source.height) parts.push(`${source.height}p`);
    else if (source.bandwidth) parts.push(`${Math.round(source.bandwidth / 1000)} kbps`);
    return parts.join(" · ");
  }
  function duration(seconds: number) {
    if (!seconds) return "";
    const pad = (value: number) => String(value).padStart(2, "0");
    const hours = Math.floor(seconds / 3600), minutes = Math.floor(seconds / 60) % 60;
    return hours ? `${hours}:${pad(minutes)}:${pad(seconds % 60)}` : `${minutes}:${pad(seconds % 60)}`;
  }
  function fileName(url: string) {
    try { return decodeURIComponent(new URL(url).pathname.split("/").pop() ?? "") || url; }
    catch { return url; }
  }

  const native = isTauriRuntime();
  let viewport = $state({ width: innerWidth, height: innerHeight });
  let anchor = $state({ left: 0, top: 0, width: 0, height: 0 });
  const position = $derived(panelPosition(anchor, viewport.width, viewport.height, { width: 360, maxHeight: 560 }));

  /** The panel stays in the shell; the native popover draws a copy of it above the page. */
  function host(node: HTMLElement) {
    const stop = native ? mountPanelHost(node, true, {
      id: "video-download",
      sessionStore: videoDownloadSession,
      close: restore => closeVideoDownload(Boolean(restore)),
      set: setVideoDownloadPopover,
      listen: onVideoDownloadPopoverAction,
    }) : () => {};
    const focus = native ? undefined : focusDialog(node, { onEscape: () => closeVideoDownload(true) });
    let frame = 0;
    const measure = () => {
      const rect = $videoDownloadSession?.anchor.getBoundingClientRect();
      if (rect && (anchor.left !== rect.left || anchor.top !== rect.top || anchor.width !== rect.width || anchor.height !== rect.height)) {
        anchor = { left: rect.left, top: rect.top, width: rect.width, height: rect.height };
      }
      frame = requestAnimationFrame(measure);
    };
    if (!native) measure();
    return { destroy() { cancelAnimationFrame(frame); stop(); focus?.destroy(); } };
  }
</script>

<svelte:window onresize={() => { viewport = { width: innerWidth, height: innerHeight }; }} />

{#if $isVideoDownloadOpen}
  <div
    class="video-download-dismiss"
    class:video-download-controller={native}
    aria-hidden={native ? "true" : undefined}
    role="presentation"
    onclick={event => { if (event.target === event.currentTarget) void closeVideoDownload(); }}
  >
    <div class="video-download-surface" style:left="{position.left}px" style:top="{position.top}px" style:width="{position.width}px" style:max-height="{position.maxHeight}px">
      <div class="video-download-panel" data-testid="video-download-dialog" role="dialog" aria-modal="false" aria-label={$t("video_download.title")} tabindex="-1" use:host>
        <h2>{$t("video_download.title")}</h2>
        {#if scanError}
          <p class="video-download-message" role="alert">{$t(scanError)}</p>
        {:else if !scan}
          <p class="video-download-message" role="status">{$t("video_download.searching")}</p>
        {:else if !scan.sources.length}
          <p class="video-download-message" role="status">{$t(scan.protected ? "video_download.protected" : "video_download.empty")}</p>
        {/if}
        {#if scan?.sources.length}
          {#each scan.videos as video, index (video.id)}
            {@const card = videoOptions(scan.sources, video.id)}
            <section class="video-download-video" data-testid="video-card">
              <div class="video-download-preview">
                <div class="video-download-thumb">
                  {#if video.thumbnail}<img src={video.thumbnail} alt="" width="96" height="64" />{:else}<span aria-hidden="true">▶</span>{/if}
                </div>
                <div class="video-download-copy">
                  <strong>{$t("video_download.video_number", { number: index + 1 })}</strong>
                  {#if video.duration}<span>{$t("video_download.duration", { time: duration(video.duration) })}</span>{/if}
                  {#if video.on_screen}<span class="video-download-badge">{$t("video_download.on_screen")}</span>{/if}
                  {#if !video.thumbnail}<span>{$t("video_download.no_preview")}</span>{/if}
                </div>
              </div>
              {#if card.options.length}
                <ul class="video-download-list">
                  {#each card.options as source (source.url)}
                    {@const state = states[source.url]}
                    <li class="video-download-item" data-testid="video-source">
                      <div class="video-download-copy">
                        <strong title={source.url}>{describe(source)}</strong>
                        <span title={source.url}>{fileName(source.url)} · {source.host}</span>
                        {#if state && state !== "starting" && state !== "started"}<span class="video-download-problem" role="alert">{$t(state)}</span>{/if}
                      </div>
                      <button class="video-download-save" disabled={state === "starting" || state === "started"} onclick={() => save(source)}>
                        {$t(state === "started" ? "video_download.added" : "video_download.download")}
                      </button>
                    </li>
                  {/each}
                </ul>
              {/if}
              {#each [...new Set(card.reasons.map(problemKey))] as reason (reason)}
                <p class="video-download-note" data-testid="video-unavailable">{$t(reason)}</p>
              {/each}
            </section>
          {/each}
        {/if}
        <div class="video-download-actions">
          <button onclick={() => search()}>{$t("video_download.search_again")}</button>
          <button onclick={showDownloads}>{$t("video_download.show_downloads")}</button>
        </div>
      </div>
    </div>
  </div>
{/if}
