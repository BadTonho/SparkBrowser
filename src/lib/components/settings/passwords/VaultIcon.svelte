<script lang="ts">
  // The few icons of the password page, drawn inline (line style, like the rest of the interface).
  let { name }: { name: "back" | "chevron" | "copy" | "eye" | "eye-off" | "lock" | "plus" | "search" } = $props();
</script>

<svg class="vault-icon" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
  {#if name === "back"}
    <path d="M19 12H5" /><path d="m12 19-7-7 7-7" />
  {:else if name === "copy"}
    <path d="M10 8h10a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H10a2 2 0 0 1-2-2V10a2 2 0 0 1 2-2Z" /><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2" />
  {:else if name === "eye"}
    <path d="M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7S2 12 2 12Z" /><circle cx="12" cy="12" r="3" />
  {:else if name === "lock"}
    <path d="M7 10V7a5 5 0 0 1 10 0v3M6 10h12a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2v-8a2 2 0 0 1 2-2ZM12 15v2" />
  {:else if name === "chevron"}
    <path d="m9 6 6 6-6 6" />
  {:else if name === "plus"}
    <path d="M12 5v14M5 12h14" />
  {:else if name === "search"}
    <circle cx="10.5" cy="10.5" r="6.5" /><path d="m16 16 4 4" />
  {:else}
    <path d="M9.9 4.2A10 10 0 0 1 12 5c6.5 0 10 7 10 7a17 17 0 0 1-3.2 4.1" /><path d="M6.6 6.6A17 17 0 0 0 2 12s3.5 7 10 7a9.7 9.7 0 0 0 5.4-1.6" /><path d="M14.1 14.1a3 3 0 0 1-4.2-4.2" /><path d="m2 2 20 20" />
  {/if}
</svg>
