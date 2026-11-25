<script lang="ts">
  import { ensureSiteIcon, siteIcons, siteKey } from "../../stores/siteIconStore";

  // The icon of a suggestion row: the site's own icon when one is cached, otherwise a glyph for the kind.
  let { kind, url }: { kind: "navigate" | "search" | "bookmark" | "history" | "web"; url: string } = $props();
  const pageKind = $derived(kind === "bookmark" || kind === "history");
  $effect(() => { if (pageKind) ensureSiteIcon(url); });
  const icon = $derived.by(() => {
    if (!pageKind) return undefined;
    const key = siteKey(url);
    return key ? $siteIcons[key] : undefined;
  });
</script>

{#if icon}
  <img class="suggestion-favicon" src={icon} alt="" width="16" height="16" draggable="false" />
{:else if kind === "bookmark"}
  <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linejoin="round"><path d="m12 3 2.7 5.6 6.1.9-4.4 4.3 1 6.1L12 17l-5.4 2.9 1-6.1-4.4-4.3 6.1-.9Z" /></svg>
{:else if kind === "history"}
  <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="9" /><path d="M12 7v5l3 2" /></svg>
{:else if kind === "navigate"}
  <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="9" /><path d="M3 12h18M12 3c3 3.5 3 14.5 0 18M12 3c-3 3.5-3 14.5 0 18" /></svg>
{:else}
  <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round"><circle cx="11" cy="11" r="7" /><path d="m21 21-4.3-4.3" /></svg>
{/if}
