<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { isTauriRuntime } from "../../services/tauri-bridge";

  let message = $state("");
  onMount(() => {
    let disposed = false;
    let stop: (() => void) | undefined;
    const showError = (event: PromiseRejectionEvent) => {
      message = event.reason instanceof Error ? event.reason.message : String(event.reason);
      event.preventDefault();
    };
    window.addEventListener("unhandledrejection", showError);
    if (isTauriRuntime()) {
      void listen<string>("browser-error", (event) => { message = event.payload; })
        .then((unlisten) => { if (disposed) unlisten(); else stop = unlisten; })
        .catch(() => { message = "Could not connect to browser error notifications"; });
    }
    return () => {
      disposed = true;
      stop?.();
      window.removeEventListener("unhandledrejection", showError);
    };
  });
</script>

{#if message}
  <aside role="alert">
    <span>{message}</span>
    <button onclick={() => { message = ""; }} aria-label="Dismiss error">×</button>
  </aside>
{/if}

<style>
  aside { position: fixed; top: 4px; left: 50%; transform: translateX(-50%); z-index: 3000; max-width: min(640px, 75vw); display: flex; align-items: center; gap: 12px; padding: 6px 12px; background: #431f23; color: #fff; border: 1px solid #bf7373; border-radius: 8px; font-size: 12px; }
  span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  button { color: inherit; background: transparent; border: 0; cursor: pointer; font-size: 18px; }
</style>
