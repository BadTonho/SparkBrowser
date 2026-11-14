<script lang="ts">
  import AppIcon from "../icons/AppIcon.svelte";
  import { ensureSiteIcon, siteIcons, siteKey } from "../../stores/siteIconStore";

  let { url, glyph, dimmed = false }: { url: string; glyph: string; dimmed?: boolean } = $props();
  $effect(() => ensureSiteIcon(url));
  const icon = $derived.by(() => { const key = siteKey(url); return key ? $siteIcons[key] : undefined; });
</script>

{#if icon}
  <img class="site-icon" class:dimmed src={icon} alt="" width="14" height="14" draggable="false" />
{:else}
  <AppIcon name={glyph} />
{/if}

<style>
  .site-icon { width: 14px; height: 14px; object-fit: contain; border-radius: 3px; display: block; }
  .site-icon.dimmed { opacity: 0.55; }
</style>
