<script lang="ts">
  import { t, formatNumber } from "../../i18n";
  import { backupBusy, bookmarkExportResult, exportBookmarksAs } from "../../stores/backupStore";
  import BackupCard from "./backup/BackupCard.svelte";

  const exported = $derived($bookmarkExportResult);
  // The result replaces the hint so the card keeps its height.
  const note = $derived(
    exported
      ? [
          $t("backup.bookmarks_exported", { count: $formatNumber(exported.count) }),
          exported.skipped > 0 ? $t("backup.bookmarks_skipped", { count: $formatNumber(exported.skipped) }) : "",
        ]
          .filter(Boolean)
          .join(" ")
      : ""
  );
</script>

<div data-testid="bookmark-export" class="wrap">
  <BackupCard id="bookmark-export-heading" icon="star" title={$t("backup.bookmarks_heading")} description={note || $t("backup.bookmarks_hint")} tone={note ? "ok" : undefined}>
    {#snippet actions()}
      <button class="pill-action" data-testid="bookmarks-export-html" title={$t("backup.bookmarks_html")} aria-label={$t("backup.bookmarks_html")} disabled={$backupBusy} onclick={() => exportBookmarksAs("html")}>{$t("backup.bookmarks_html_short")}</button>
      <button class="pill-action" data-testid="bookmarks-export-csv" title={$t("backup.bookmarks_csv")} aria-label={$t("backup.bookmarks_csv")} disabled={$backupBusy} onclick={() => exportBookmarksAs("csv")}>{$t("backup.bookmarks_csv_short")}</button>
    {/snippet}
  </BackupCard>
</div>

<style>
  /* Lets the card stretch to the row height set by the grid. */
  .wrap { display: flex; min-width: 0; }
</style>
