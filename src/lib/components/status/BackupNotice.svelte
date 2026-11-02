<script lang="ts">
  import { t } from "../../i18n";
  import { pendingBackupPreferences } from "../../stores/backupStore";
  import { openSettingsModal, setActiveSettingsCategory } from "../../stores/settingsStore";
</script>
{#if $pendingBackupPreferences}
  <aside data-testid="backup-pending" role="status"><span>{$t("backup.pending")}</span><button onclick={() => { setActiveSettingsCategory("data_backup"); void openSettingsModal(); }}>{$t("backup.open")}</button></aside>
{/if}
<style>aside { display: flex; gap: 12px; align-items: center; padding: 6px 14px; font-size: 12px; background: var(--bg-glass-elevated); } button { color: var(--accent-primary); }</style>
