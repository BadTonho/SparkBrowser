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
  {:else if action === "extensions"}
    <path d="M20.5 11H19V7c0-1.1-.9-2-2-2h-4V3.5a2.5 2.5 0 0 0-5 0V5H4c-1.1 0-1.99.9-1.99 2v3.8H3.5c1.49 0 2.7 1.21 2.7 2.7s-1.21 2.7-2.7 2.7H2V20c0 1.1.9 2 2 2h3.8v-1.5c0-1.49 1.21-2.7 2.7-2.7 1.49 0 2.7 1.21 2.7 2.7V22H17c1.1 0 2-.9 2-2v-4h1.5a2.5 2.5 0 0 0 0-5z" />
  {:else if action === "settings"}
    <path d="m9 3-.5 3-2 .9L4 6l-2 3 2.5 2v2L2 15l2 3 2.5-.9 2 .9.5 3h6l.5-3 2-.9 2.5.9 2-3-2.5-2v-2L22 9l-2-3-2.5.9-2-.9L15 3H9Z" />
    <circle cx="12" cy="12" r="3" />
  {:else if action === "zen"}
    <path d="M8 3H3v5m13-5h5v5M3 16v5h5m13-5v5h-5" />
  {:else if action === "qr"}
    <path d="M3 3h7v7H3zM14 3h7v7h-7zM3 14h7v7H3zM14 14h3v3h-3zM19 14v2m2 2v3h-3m-4 0v-2" />
  {:else if action === "whatsapp"}
    <path d="M21 11.5a8.5 8.5 0 0 1-12.4 7.5L3 20l1-5.2A8.5 8.5 0 1 1 21 11.5Z" />
  {/if}
</svg>

<style>
  svg { display: block; flex-shrink: 0; pointer-events: none; }
</style>
