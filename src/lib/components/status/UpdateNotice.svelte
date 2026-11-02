<script lang="ts">
  import { updateState } from "../../stores/updatesStore";
  import { openSettingsModal, setActiveSettingsCategory } from "../../stores/settingsStore";
  let dismissed = $state("");
  function open() { setActiveSettingsCategory("updates"); void openSettingsModal(); }
</script>
{#if $updateState.update && ["available", "ready"].includes($updateState.status) && dismissed !== $updateState.update.version}
  <aside aria-label="Browser update" role="status">
    <span>Veloce Browser {$updateState.update.version} is {$updateState.status === "ready" ? "ready to install" : "available"}.</span>
    <button onclick={open}>View update</button>
    <button onclick={() => { dismissed = $updateState.update?.version ?? ""; }}>Later</button>
  </aside>
{/if}
<style>
  aside { flex: 0 0 auto; display: flex; justify-content: center; align-items: center; gap: 12px; padding: 5px 12px; background: var(--bg-secondary, #202020); color: var(--text-primary); font-size: 12px; border-bottom: 1px solid var(--border-subtle); }
  button { border: 1px solid var(--border-subtle); border-radius: 6px; background: transparent; color: inherit; padding: 3px 8px; cursor: pointer; }
</style>
