<script lang="ts">
  import { ICON_FILLS, ICON_PATHS, resolveIconName } from "./appIcons";

  let { name, size = 16 }: { name: string; size?: number } = $props();

  const resolved = $derived(resolveIconName(name ?? ""));
  const path = $derived(resolved ? ICON_PATHS[resolved] : undefined);
  const fill = $derived(resolved ? ICON_FILLS[resolved] : undefined);
</script>

{#if path}
  <svg
    class="app-icon"
    width={size}
    height={size}
    viewBox="0 0 24 24"
    fill={fill ?? "none"}
    stroke={fill ?? "currentColor"}
    stroke-width="1.7"
    stroke-linecap="round"
    stroke-linejoin="round"
    aria-hidden="true"
    focusable="false"
  >
    <path d={path} />
  </svg>
{:else}
  <span class="app-icon-glyph" aria-hidden="true">{name}</span>
{/if}

<style>
  .app-icon {
    display: inline-block;
    flex-shrink: 0;
    vertical-align: -0.125em;
    pointer-events: none;
  }
  .app-icon-glyph {
    display: inline-block;
    line-height: 1;
  }
</style>
