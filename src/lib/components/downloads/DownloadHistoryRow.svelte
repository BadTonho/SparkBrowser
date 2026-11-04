<script lang="ts">
  import { t, formatNumber, type MessageKey } from "../../i18n";
  import type { DownloadItem, DownloadVisual } from "../../types/browser";
  import { formatDownloadBytes, downloadStatusIcon } from "../../services/download-visual";
  import { loadDownloadVisual } from "../../stores/downloadVisuals";
  import DownloadFilePreview from "./DownloadFilePreview.svelte";
  import DownloadIcon from "./DownloadIcon.svelte";
  let { item }: { item: DownloadItem } = $props();
  let visual = $state<DownloadVisual | null>(null);
  $effect(() => {
    let active = true; visual = null;
    void loadDownloadVisual({ kind: "history", id: item.id }, String(item.created_at)).then(value => { if (active) visual = value; });
    return () => { active = false; };
  });
</script>
<article class="download-row download-history" data-testid="download-history">
  <DownloadFilePreview name={item.file_name} thumbnail={visual?.thumbnail_png} />
  <div class="download-copy"><strong class="download-name" title={item.file_name}>{item.file_name}</strong>
    <span class="download-size">{formatDownloadBytes(visual?.size_bytes ?? null, $formatNumber, $t("downloads.size_unknown"))}</span></div>
  <span class="download-state {downloadStatusIcon(item.status)}" role="img" aria-label={$t(`common.download_${item.status.toLowerCase()}` as MessageKey)} title={$t(`common.download_${item.status.toLowerCase()}` as MessageKey)}><DownloadIcon name={downloadStatusIcon(item.status)} /></span>
</article>
