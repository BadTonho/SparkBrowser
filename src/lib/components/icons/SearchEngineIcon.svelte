<script lang="ts">
  import { SEARCH_ENGINE_ICON_PATHS } from "./searchEngineIcons.js";

  let { id, color }: { id: string; color: string } = $props();
  const path = $derived(SEARCH_ENGINE_ICON_PATHS[id]);
</script>

{#if id === "google" || id === "duckduckgo"}
  <span class="brand-mark" class:google={id === "google"} class:duckduckgo={id === "duckduckgo"} aria-hidden="true"></span>
{:else if path}
  <svg width="24" height="24" viewBox="0 0 24 24" fill={color} aria-hidden="true" focusable="false"><path d={path} /></svg>
{/if}

<style>
  .brand-mark { display: block; width: 24px; height: 24px; }
  .google {
    /* Use the bundled outline and the color stops from the Google brand asset. */
    mask: url("./search-engines/google.svg") center / contain no-repeat;
    background: conic-gradient(
      from 45deg, #ff4641 4deg, #3186ff 39deg 72deg, #00a5b7 97deg,
      #0ebc5f 121deg 155deg, #6cc400 179deg, #ffcc00 204deg,
      #ffd314 227deg, #ffcc00 252deg, #ff6a2b 273deg, #ff4641 289deg
    );
  }
  .duckduckgo { background: url("./search-engines/duckduckgo.svg") center / contain no-repeat; }
</style>
