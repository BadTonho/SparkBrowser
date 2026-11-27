<script lang="ts">
  import { localBackupError } from "../../stores/backupStore";
  import ErrorMessage from "../status/ErrorMessage.svelte";
  import BackupEncryptionCard from "./backup/BackupEncryptionCard.svelte";
  import BackupHistoryCard from "./backup/BackupHistoryCard.svelte";
  import BackupScheduleCard from "./backup/BackupScheduleCard.svelte";
  import BackupStatusCard from "./backup/BackupStatusCard.svelte";
</script>

<!-- The cards of the "Automatic backups" category; its title and description are in BackupSettings. -->
<div class="local-panel" data-testid="backup-local-panel">
  <BackupStatusCard />
  <div class="pair-cards">
    <BackupScheduleCard />
    <BackupEncryptionCard />
  </div>
  <BackupHistoryCard />

  <ErrorMessage error={$localBackupError} />
</div>

<style>
  .local-panel { display: flex; flex-direction: column; gap: 12px; }
  /* Two columns when there is room; each card keeps a readable minimum width. */
  .pair-cards { display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); align-items: start; gap: 12px; }
</style>
