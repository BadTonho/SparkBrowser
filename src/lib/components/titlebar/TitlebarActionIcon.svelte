<script lang="ts">
  import type { TitlebarAction } from "../../stores/titlebarActionsStore";

  let { action }: { action: TitlebarAction } = $props();
</script>

<svg
  width="16"
  height="16"
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width="1.8"
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
  focusable="false"
>
  {#if action === "tools"}
    <path d="M9 6h11M9 12h11M9 18h11M4 6h.01M4 12h.01M4 18h.01" />
  {:else if action === "bookmarks"}
    <path d="M4 5.5h16M4 12h16M4 18.5h16M7 3.5v4M17 3.5v4" />
  {:else if action === "commands"}
    <path d="m13 2-9 12h7l-1 8 10-12h-7l1-8Z" />
  {:else if action === "history"}
    <circle cx="12" cy="12" r="9" />
    <path d="M12 7v5l3 2" />
  {:else if action === "downloads"}
    <path d="M12 3v12m-5-5 5 5 5-5M4 15v5h16v-5" />
  {:else if action === "privacy"}
    <path d="M12 3 4 6v6c0 4.5 4.5 7.5 8 9 3.5-1.5 8-4.5 8-9V6l-8-3Z" />
  {:else if action === "settings"}
    <path d="m9 3-.5 3-2 .9L4 6l-2 3 2.5 2v2L2 15l2 3 2.5-.9 2 .9.5 3h6l.5-3 2-.9 2.5.9 2-3-2.5-2v-2L22 9l-2-3-2.5.9-2-.9L15 3H9Z" />
    <circle cx="12" cy="12" r="3" />
  {:else if action === "zen"}
    <path d="M8 3H3v5m13-5h5v5M3 16v5h5m13-5v5h-5" />
  {/if}
</svg>

<style>
  svg { display: block; flex-shrink: 0; pointer-events: none; }
</style>
