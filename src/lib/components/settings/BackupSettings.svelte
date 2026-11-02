<script lang="ts">
  import { t, labelText, formatNumber, type MessageKey } from "../../i18n";
  import { isTauriRuntime } from "../../services/bridge/runtime";
  import { backupBusy, backupError, backupExported, backupPreview, backupResult, pendingBackupPreferences, saveBackup, selectBackup, importBackup, refreshBackupPreview, dismissBackupPreview, retryBackupPreferences } from "../../stores/backupStore";
  import { SHORTCUT_DEFINITIONS, formatShortcut } from "../../stores/shortcutsStore";
  import type { BackupKind } from "../../types/backup";
  import ErrorMessage from "../status/ErrorMessage.svelte";
  const desktop = isTauriRuntime();
  let applyPreferences = $state(false);
  let skippedLimit = $state(100);
  $effect(() => { void $backupPreview?.token; applyPreferences = false; skippedLimit = 100; });
  function valueLabel(value: unknown, name: string): string {
    if (name === "site_zoom" && Array.isArray(value)) return value.map(row => `${row.origin}: ${$formatNumber(Math.round(row.factor*100))}%`).join(" · ") || "100%";
    if (typeof value === "number") return $formatNumber(value);
    const labels: Record<string, MessageKey> = { horizontal: "backup.value_horizontal", vertical: "backup.value_vertical", toolbar: "backup.value_toolbar", popup: "backup.value_popup", auto: "backup.value_auto", off: "backup.value_off", true: "backup.value_true", false: "backup.value_false", stable: "backup.value_stable", beta: "backup.value_beta", system: "common.system" };
    if (name === "language") return value === "pt-BR" ? "Português" : value === "en" ? "English" : $t("common.system");
    return labels[String(value)] ? $t(labels[String(value)]) : String(value);
  }
</script>
<section class="setting-group backup-settings" aria-labelledby="backup-heading">
  <h2 class="group-title" id="backup-heading">{$t("backup.heading")}</h2>
  {#if !desktop}<p role="status">{$t("common.desktop_required")}</p>
  {:else}
    <p>{$t("backup.description")}</p><p>{$t("backup.scope")}</p><p>{$t("backup.exclusions")}</p><p>{$t("backup.limit")}</p>
    <div class="actions"><button class="pill-action" data-testid="backup-export" disabled={$backupBusy} onclick={saveBackup}>{$t("backup.export")}</button><button class="pill-action" data-testid="backup-select" disabled={$backupBusy} onclick={selectBackup}>{$t("backup.import")}</button></div>
    <ErrorMessage error={$backupError} />
    {#if $backupExported}<p role="status">{$t("backup.exported")}</p>{/if}
    {#if $pendingBackupPreferences}
      <aside role="status"><p>{$t("backup.pending")}</p><ErrorMessage error={$pendingBackupPreferences.error} /><button data-testid="backup-retry-preferences" disabled={$backupBusy} onclick={retryBackupPreferences}>{$t("backup.retry")}</button></aside>
    {/if}
    {#if $backupPreview}
      <article data-testid="backup-preview">
        <h3>{$t("backup.preview")}</h3><p>{$t("backup.keep_current")}</p>
        <table><thead><tr><th>{$t("backup.kind")}</th><th>{$t("backup.total")}</th><th>{$t("backup.added")}</th></tr></thead><tbody>
          {#each Object.keys($backupPreview.counts) as kind}
            <tr><th>{$t(("backup.kind_" + kind) as MessageKey)}</th><td>{$formatNumber($backupPreview.counts[kind as BackupKind])}</td><td>{$formatNumber($backupPreview.added[kind as BackupKind])}</td></tr>
          {/each}
        </tbody></table>
        <details><summary>{$t("backup.skipped", { count: $formatNumber($backupPreview.skipped.length) })}</summary>
          <ul>{#each $backupPreview.skipped.slice(0, skippedLimit) as row}<li>{$t(("backup.kind_" + row.kind) as MessageKey)}: {row.key} — {$t(row.reason === "identical" ? "backup.identical" : "backup.existing")}</li>{/each}</ul>
          {#if skippedLimit < $backupPreview.skipped.length}<button onclick={() => skippedLimit += 100}>{$t("backup.more")}</button>{/if}
        </details>
        <h4>{$t("backup.preferences")}</h4>
        <dl>{#each Object.entries($backupPreview.preferences).filter(([key]) => key !== "keyboard_shortcuts") as [key, value]}<dt>{$t(("backup.pref_" + key) as MessageKey)}</dt><dd>{valueLabel(value, key)}</dd>{/each}</dl>
        <details><summary>{$t("backup.pref_keyboard_shortcuts")}</summary><dl>
          {#each Object.entries($backupPreview.preferences.keyboard_shortcuts) as [id, bindings]}<dt>{$labelText(SHORTCUT_DEFINITIONS.find(item => item.id === id)?.label ?? id)}</dt><dd>{bindings.map(binding => formatShortcut(binding, SHORTCUT_DEFINITIONS.find(item => item.id === id)?.sequence)).join(", ")}</dd>{/each}
        </dl></details>
        <label class="preference-choice"><input type="checkbox" data-testid="backup-apply-preferences" bind:checked={applyPreferences} disabled={$backupBusy || Boolean($pendingBackupPreferences)} /> {$t("backup.apply_preferences")}</label>
        <div class="actions"><button data-testid="backup-confirm" disabled={$backupBusy} onclick={() => importBackup(applyPreferences)}>{$t("backup.confirm")}</button><button disabled={$backupBusy} onclick={refreshBackupPreview}>{$t("backup.refresh_preview")}</button><button disabled={$backupBusy} onclick={dismissBackupPreview}>{$t("backup.cancel")}</button></div>
      </article>
    {/if}
    {#if $backupResult}<p role="status" data-testid="backup-result">{$t("backup.imported")}</p>{/if}
  {/if}
</section>
<style>
  p, li, dt, dd { font-size: 12px; line-height: 1.6; overflow-wrap: anywhere; }
  p { color: var(--text-secondary); }
  .actions { display: flex; flex-wrap: wrap; gap: 10px; margin: 12px 0; }
  article, aside { border: 1px solid var(--border-subtle); border-radius: 10px; padding: 14px; }
  table { width: 100%; font-size: 12px; border-collapse: collapse; } th, td { text-align: left; padding: 8px; border-bottom: 1px solid var(--border-subtle); }
  dl { display: grid; grid-template-columns: minmax(0, 1fr) minmax(0, 1fr); gap: 6px 14px; } dd { margin: 0; }
  details ul { max-height: 220px; overflow: auto; }
  .preference-choice { display: block; margin: 18px 0; font-size: 13px; }
</style>
