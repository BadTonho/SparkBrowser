<script lang="ts">
  import { t, labelText, formatDate, formatNumber, type MessageKey } from "../../i18n";
  import {
    backupBusy,
    backupPreview,
    dismissBackupPreview,
    importBackup,
    pendingBackupPreferences,
    refreshBackupPreview,
    restoreOptions,
    updateRestoreOptions,
  } from "../../stores/backupStore";
  import { SHORTCUT_DEFINITIONS, formatShortcut } from "../../stores/shortcutsStore";
  import type { BackupKind, RestoreGroup, RestoreMode } from "../../types/backup";

  const DATA_GROUPS: RestoreGroup[] = ["workspaces", "bookmarks", "sessions", "notes", "site_search"];
  const PREFERENCE_GROUPS: RestoreGroup[] = ["settings", "shortcuts"];
  /** Kinds shown under each group, so counts explain what a checkbox controls. */
  const GROUP_KINDS: Record<RestoreGroup, BackupKind[]> = {
    workspaces: ["workspaces", "tabs", "split_layouts"],
    bookmarks: ["folders", "bookmarks"],
    sessions: ["sessions"],
    notes: ["notes"],
    site_search: ["site_search"],
    settings: [],
    shortcuts: [],
  };
  /** Workspaces stay merge-only: replacing them would close live tabs. */
  const REPLACEABLE: RestoreGroup[] = ["bookmarks", "sessions", "notes", "site_search"];

  let skippedLimit = $state(100);
  $effect(() => {
    void $backupPreview?.token;
    skippedLimit = 100;
  });

  const picked = $derived($restoreOptions.groups);
  const replacing = $derived($restoreOptions.mode === "replace");
  const removedTotal = $derived(
    Object.values($backupPreview?.removed ?? {}).reduce((sum, n) => sum + (n ?? 0), 0)
  );
  const prefsLocked = $derived($backupBusy || Boolean($pendingBackupPreferences));

  function kindsOf(groups: RestoreGroup[]): BackupKind[] {
    return groups.flatMap((group) => GROUP_KINDS[group]);
  }
  function toggle(group: RestoreGroup, on: boolean): void {
    const groups = on ? [...picked, group] : picked.filter((item) => item !== group);
    void updateRestoreOptions({ ...$restoreOptions, groups });
  }
  function setMode(mode: RestoreMode): void {
    void updateRestoreOptions({ ...$restoreOptions, mode });
  }
  function groupCount(group: RestoreGroup): number {
    const counts = $backupPreview?.counts;
    return counts ? GROUP_KINDS[group].reduce((sum, kind) => sum + counts[kind], 0) : 0;
  }
  function valueLabel(value: unknown, name: string): string {
    if (name === "site_zoom" && Array.isArray(value))
      return value.map((row) => `${row.origin}: ${$formatNumber(Math.round(row.factor * 100))}%`).join(" · ") || "100%";
    if (typeof value === "number") return $formatNumber(value);
    const labels: Record<string, MessageKey> = {
      horizontal: "backup.value_horizontal",
      vertical: "backup.value_vertical",
      toolbar: "backup.value_toolbar",
      popup: "backup.value_popup",
      auto: "backup.value_auto",
      off: "backup.value_off",
      true: "backup.value_true",
      false: "backup.value_false",
      stable: "backup.value_stable",
      beta: "backup.value_beta",
      system: "common.system",
    };
    if (name === "language")
      return value === "pt-BR" ? "Português" : value === "en" ? "English" : $t("common.system");
    return labels[String(value)] ? $t(labels[String(value)]) : String(value);
  }
</script>

{#if $backupPreview}
  <article data-testid="backup-preview">
    <h3>{$t("backup.preview")}</h3>
    <p data-testid="backup-preview-meta">
      {$t("backup.preview_meta", {
        date: $formatDate($backupPreview.created_at * 1000, { dateStyle: "medium", timeStyle: "short" }),
        version: $formatNumber($backupPreview.version),
      })}
    </p>

    <fieldset class="choices" disabled={$backupBusy}>
      <legend>{$t("backup.restore_what")}</legend>
      {#each DATA_GROUPS as group}
        <label class="preference-choice">
          <input type="checkbox" data-testid={"backup-group-" + group} checked={picked.includes(group)} onchange={(e) => toggle(group, e.currentTarget.checked)} />
          {$t(("backup.group_" + group) as MessageKey)}
          <span class="count">{$formatNumber(groupCount(group))}</span>
        </label>
      {/each}
    </fieldset>

    <fieldset class="choices" disabled={$backupBusy}>
      <legend>{$t("backup.restore_how")}</legend>
      <label class="preference-choice">
        <input type="radio" name="restore-mode" data-testid="backup-mode-merge" checked={!replacing} onchange={() => setMode("merge")} />
        {$t("backup.mode_merge")}
      </label>
      <label class="preference-choice">
        <input type="radio" name="restore-mode" data-testid="backup-mode-replace" checked={replacing} onchange={() => setMode("replace")} />
        {$t("backup.mode_replace")}
      </label>
      <p>{$t(replacing ? "backup.mode_replace_hint" : "backup.keep_current")}</p>
    </fieldset>

    <table><thead><tr><th>{$t("backup.kind")}</th><th>{$t("backup.total")}</th><th>{$t("backup.added")}</th>{#if replacing}<th>{$t("backup.removed")}</th>{/if}</tr></thead><tbody>
      {#each kindsOf(picked.filter((group) => DATA_GROUPS.includes(group))) as kind}
        <tr data-testid={"backup-row-" + kind}>
          <th>{$t(("backup.kind_" + kind) as MessageKey)}</th>
          <td>{$formatNumber($backupPreview.counts[kind])}</td>
          <td>{$formatNumber($backupPreview.added[kind])}</td>
          {#if replacing}<td>{$formatNumber($backupPreview.removed[kind] ?? 0)}</td>{/if}
        </tr>
      {/each}
    </tbody></table>
    {#if replacing && removedTotal > 0}
      <p role="alert" class="warning" data-testid="backup-replace-warning">{$t("backup.replace_warning", { count: $formatNumber(removedTotal) })}</p>
    {/if}
    {#if replacing && picked.some((group) => !REPLACEABLE.includes(group) && DATA_GROUPS.includes(group))}
      <p class="hint">{$t("backup.replace_workspaces_note")}</p>
    {/if}

    <details><summary>{$t("backup.skipped", { count: $formatNumber($backupPreview.skipped.length) })}</summary>
      <ul>{#each $backupPreview.skipped.slice(0, skippedLimit) as row}<li>{$t(("backup.kind_" + row.kind) as MessageKey)}: {row.key} — {$t(row.reason === "identical" ? "backup.identical" : "backup.existing")}</li>{/each}</ul>
      {#if skippedLimit < $backupPreview.skipped.length}<button onclick={() => skippedLimit += 100}>{$t("backup.more")}</button>{/if}
    </details>

    <h4>{$t("backup.preferences")}</h4>
    <fieldset class="choices" disabled={prefsLocked}>
      <legend>{$t("backup.apply_preferences")}</legend>
      {#each PREFERENCE_GROUPS as group}
        <label class="preference-choice">
          <input type="checkbox" data-testid={group === "settings" ? "backup-apply-preferences" : "backup-apply-shortcuts"} checked={picked.includes(group)} onchange={(e) => toggle(group, e.currentTarget.checked)} />
          {$t(("backup.group_" + group) as MessageKey)}
        </label>
      {/each}
    </fieldset>
    <dl>{#each Object.entries($backupPreview.preferences).filter(([key]) => key !== "keyboard_shortcuts") as [key, value]}<dt>{$t(("backup.pref_" + key) as MessageKey)}</dt><dd>{valueLabel(value, key)}</dd>{/each}</dl>
    <details><summary>{$t("backup.pref_keyboard_shortcuts")}</summary><dl>
      {#each Object.entries($backupPreview.preferences.keyboard_shortcuts) as [id, bindings]}<dt>{$labelText(SHORTCUT_DEFINITIONS.find(item => item.id === id)?.label ?? id)}</dt><dd>{bindings.map(binding => formatShortcut(binding, SHORTCUT_DEFINITIONS.find(item => item.id === id)?.sequence)).join(", ")}</dd>{/each}
    </dl></details>

    <div class="actions">
      <button data-testid="backup-confirm" disabled={$backupBusy || picked.length === 0} onclick={() => importBackup()}>{$t(replacing && removedTotal > 0 ? "backup.confirm_replace" : "backup.confirm")}</button>
      <button disabled={$backupBusy} onclick={refreshBackupPreview}>{$t("backup.refresh_preview")}</button>
      <button disabled={$backupBusy} onclick={dismissBackupPreview}>{$t("backup.cancel")}</button>
    </div>
    {#if picked.length === 0}<p role="status">{$t("backup.nothing_selected")}</p>{/if}
  </article>
{/if}

<style>
  p, li, dt, dd { font-size: 12px; line-height: 1.6; overflow-wrap: anywhere; }
  p { color: var(--text-secondary); }
  h3 { margin: 0; color: var(--text-primary); font-size: 16px; line-height: 1.35; }
  article { border: 1px solid var(--border-subtle); border-radius: 12px; padding: 14px; background: var(--bg-glass, rgba(255, 255, 255, 0.025)); }
  .actions { display: flex; flex-wrap: wrap; gap: 8px; margin: 10px 0; }
  table { width: 100%; font-size: 12px; border-collapse: collapse; } th, td { text-align: left; padding: 8px; border-bottom: 1px solid var(--border-subtle); }
  dl { display: grid; grid-template-columns: minmax(0, 1fr) minmax(0, 1fr); gap: 6px 14px; } dd { margin: 0; }
  details ul { max-height: 220px; overflow: auto; }
  .choices { margin: 10px 0; padding: 0; border: 0; min-width: 0; }
  .choices legend { padding: 0; font-size: 11.5px; font-weight: 600; color: var(--text-secondary); }
  .preference-choice { display: flex; align-items: center; gap: 8px; margin: 6px 0; font-size: 12.5px; color: var(--text-primary); cursor: pointer; }
  .count { margin-left: auto; font-size: 11.5px; color: var(--text-secondary); }
  .warning { padding: 8px 10px; border-radius: 8px; border: 1px solid rgba(248, 113, 113, .42); background: rgba(248, 113, 113, .12); color: #fca5a5; }
  .hint { font-size: 11.5px; }
</style>
