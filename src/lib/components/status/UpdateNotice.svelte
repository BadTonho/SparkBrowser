<script lang="ts">
  import { t } from "../../i18n";
  import { updateState } from "../../stores/updatesStore";
  import { openSettingsModal, setActiveSettingsCategory } from "../../stores/settingsStore";
  let dismissed = $state("");
  function open() { setActiveSettingsCategory("updates"); void openSettingsModal(); }
</script>
{#if $updateState.update && ["available", "ready"].includes($updateState.status) && dismissed !== $updateState.update.version}
  <aside aria-label={$t("status.browser_update_d514")} role="status">
    <span>Veloce Browser {$updateState.update.version} {$t("status.is_fa51")} {$updateState.status === "ready" ? $t("status.ready_to_install_519c") : "available"}.</span>
    <button onclick={open}>{$t("status.view_update_9785")}</button>
    <button onclick={() => { dismissed = $updateState.update?.version ?? ""; }}>{$t("status.later_73b6")}</button>
  </aside>
{/if}
<style>
  aside { flex: 0 0 auto; display: flex; justify-content: center; align-items: center; gap: 12px; padding: 5px 12px; background: var(--bg-secondary, #202020); color: var(--text-primary); font-size: 12px; border-bottom: 1px solid var(--border-subtle); }
  button { border: 1px solid var(--border-subtle); border-radius: 6px; background: transparent; color: inherit; padding: 3px 8px; cursor: pointer; }
</style>
