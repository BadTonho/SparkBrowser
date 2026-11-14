<script lang="ts">
  import { SEARCH_ENGINE_ICON_PATHS } from "./searchEngineIcons.js";

  let { id, color, size = 22 }: { id: string; color: string; size?: number } = $props();
  const path = $derived(SEARCH_ENGINE_ICON_PATHS[id]);
</script>

{#if path}
  <svg width={size} height={size} viewBox="0 0 24 24" fill={color} aria-hidden="true" focusable="false"><path d={path} /></svg>
{/if}
