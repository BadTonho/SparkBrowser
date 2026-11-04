<script lang="ts">
  import DownloadIcon from "./DownloadIcon.svelte";
  import { downloadFileKind, isDownloadThumbnail } from "../../services/download-visual";
  let { name, thumbnail = null }: { name: string; thumbnail?: string | null } = $props();
</script>
<div class="download-file-preview" aria-hidden="true">
  {#if isDownloadThumbnail(thumbnail)}<img src={thumbnail} width="40" height="40" alt="" draggable="false" />
  {:else}<DownloadIcon name={downloadFileKind(name)} size={24} />{/if}
</div>
