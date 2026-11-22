<script lang="ts">
  import { isDownloadThumbnail } from "../../services/download-visual";
  let { name, avatar = null }: { name: string; avatar?: string | null } = $props();
  const initial = $derived(Array.from(name.trim())[0]?.toUpperCase() ?? "");
</script>

<span class="profile-avatar" aria-hidden="true">
  {#if isDownloadThumbnail(avatar)}
    <img src={avatar} alt="" draggable="false" />
  {:else}
    {initial || "?"}
  {/if}
</span>
