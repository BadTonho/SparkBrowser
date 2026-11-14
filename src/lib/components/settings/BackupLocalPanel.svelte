<script lang="ts">
  import { t } from "../../i18n";
  import { localBackupError } from "../../stores/backupStore";
  import ErrorMessage from "../status/ErrorMessage.svelte";
  import BackupEncryptionCard from "./backup/BackupEncryptionCard.svelte";
  import BackupHistoryCard from "./backup/BackupHistoryCard.svelte";
  import BackupScheduleCard from "./backup/BackupScheduleCard.svelte";
  import BackupStatusCard from "./backup/BackupStatusCard.svelte";
</script>

<section class="local-panel" aria-labelledby="backup-local-heading" data-testid="backup-local-panel">
  <header>
    <h3 id="backup-local-heading">{$t("backup.local_heading")}</h3>
    <p>{$t("backup.local_description")}</p>
  </header>

  <BackupStatusCard />
  <div class="pair-cards">
    <BackupScheduleCard />
    <BackupEncryptionCard />
  </div>
  <BackupHistoryCard />

  <ErrorMessage error={$localBackupError} />
</section>

<style>
  .local-panel { display: flex; max-width: 820px; flex-direction: column; gap: 12px; margin-top: 20px; }
  header { margin-bottom: 2px; }
  h3 { margin: 0; color: var(--text-primary); font-size: 16px; line-height: 1.35; }
  header p { margin: 3px 0 0; color: var(--text-secondary); font-size: 12px; line-height: 1.6; }
  /* Two columns when there is room; each card keeps a readable minimum width. */
  .pair-cards { display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); align-items: start; gap: 12px; }
</style>
