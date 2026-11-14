<script lang="ts">
  import { t, formatNumber } from "../../i18n";
  import { backupBusy, bookmarkExportResult, exportBookmarksAs } from "../../stores/backupStore";
</script>

<article class="bookmark-export-card" aria-labelledby="bookmark-export-heading" data-testid="bookmark-export">
  <h3 id="bookmark-export-heading">{$t("backup.bookmarks_heading")}</h3>
  <p>{$t("backup.bookmarks_hint")}</p>
  <div class="actions">
    <button class="pill-action" data-testid="bookmarks-export-html" disabled={$backupBusy} onclick={() => exportBookmarksAs("html")}>{$t("backup.bookmarks_html")}</button>
    <button class="pill-action" data-testid="bookmarks-export-csv" disabled={$backupBusy} onclick={() => exportBookmarksAs("csv")}>{$t("backup.bookmarks_csv")}</button>
  </div>
  {#if $bookmarkExportResult}
    <p role="status" data-testid="bookmarks-export-result">
      {$t("backup.bookmarks_exported", { count: $formatNumber($bookmarkExportResult.count) })}
      {#if $bookmarkExportResult.skipped > 0}{$t("backup.bookmarks_skipped", { count: $formatNumber($bookmarkExportResult.skipped) })}{/if}
    </p>
  {/if}
</article>

<style>
  .bookmark-export-card { display: flex; box-sizing: border-box; width: 100%; max-width: 270px; min-width: 0; flex-direction: column; gap: 8px; padding: 14px; border: 1px solid var(--border-subtle); border-radius: 12px; background: var(--bg-glass, rgba(255, 255, 255, 0.025)); }
  h3 { margin: 0; color: var(--text-primary); font-size: 16px; line-height: 1.35; }
  p { margin: 0; font-size: 12px; line-height: 1.6; color: var(--text-secondary); overflow-wrap: anywhere; }
  .actions { display: flex; flex-wrap: wrap; gap: 8px; }
  .actions button { white-space: normal; }
</style>
