<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { t } from "../../i18n";
  import { createNewTab, isTauriRuntime } from "../../services/tauri-bridge";
  import { buildIssueUrl } from "../../services/error-report.js";
  import { addErrorReport, dismissErrorReport, errorReports, type ErrorReport } from "../../stores/errorReportStore";

  const visible = 3;
  let expanded = $state<string | null>(null);
  let sendFailed = $state<string | null>(null);

  onMount(() => {
    let disposed = false;
    let stop: (() => void) | undefined;
    if (isTauriRuntime()) {
      void listen<ErrorReport>("error-report", (event) => addErrorReport(event.payload))
        .then((unlisten) => { if (disposed) unlisten(); else stop = unlisten; })
        .catch(() => {});
    }
    return () => { disposed = true; stop?.(); };
  });

  async function send(report: ErrorReport) {
    sendFailed = null;
    try {
      await createNewTab(buildIssueUrl(report));
      dismissErrorReport(report.id);
    } catch {
      sendFailed = report.id;
    }
  }
</script>

{#if $errorReports.length > 0}
  <div class="reports" role="region" aria-label={$t("status.error_report_region")}>
    {#each $errorReports.slice(0, visible) as report (report.id)}
      <div class="card" class:critical={report.level === "critical"} role="alert">
        <div class="head">
          <strong>{report.level === "critical" ? $t("status.error_report_title_critical") : $t("status.error_report_title")}</strong>
          <button type="button" class="close" aria-label={$t("status.dismiss_error_2db0")} onclick={() => dismissErrorReport(report.id)}>×</button>
        </div>
        <p class="summary">{report.summary}</p>
        {#if expanded === report.id}
          <pre>{report.details || report.summary}</pre>
        {/if}
        {#if sendFailed === report.id}
          <p class="failed">{$t("status.error_report_send_failed")}</p>
        {/if}
        <div class="actions">
          <button type="button" onclick={() => { expanded = expanded === report.id ? null : report.id; }} aria-expanded={expanded === report.id}>
            {expanded === report.id ? $t("status.error_report_hide") : $t("status.error_report_view")}
          </button>
          <button type="button" class="primary" onclick={() => void send(report)}>{$t("status.error_report_send")}</button>
        </div>
      </div>
    {/each}
    {#if $errorReports.length > visible}
      <p class="more">{$t("status.error_report_more", { count: $errorReports.length - visible })}</p>
    {/if}
  </div>
{/if}

<style>
  .reports { position: fixed; right: 16px; bottom: 16px; z-index: 9998; display: flex; flex-direction: column; gap: 8px; width: min(380px, calc(100vw - 32px)); }
  .card { background: rgba(32, 20, 22, 0.96); color: #f4eded; border: 1px solid rgba(255, 69, 58, 0.5); border-radius: 10px; padding: 10px 12px; box-shadow: 0 8px 24px rgba(0, 0, 0, 0.45); font-size: 12px; animation: rise 0.2s cubic-bezier(0.16, 1, 0.3, 1); }
  .card.critical { border-color: #ff3b30; background: rgba(48, 14, 14, 0.97); }
  .head { display: flex; align-items: center; justify-content: space-between; gap: 8px; }
  .head strong { color: #ff6b61; font-size: 13px; }
  .summary { margin: 6px 0 8px; line-height: 1.4; overflow-wrap: anywhere; max-height: 4.2em; overflow: hidden; }
  pre { margin: 0 0 8px; padding: 8px; max-height: 180px; overflow: auto; background: rgba(0, 0, 0, 0.4); border-radius: 6px; font-size: 11px; white-space: pre-wrap; overflow-wrap: anywhere; user-select: text; }
  .failed { margin: 0 0 8px; color: #ffb4ab; }
  .actions { display: flex; gap: 8px; justify-content: flex-end; }
  .actions button { background: rgba(255, 255, 255, 0.1); border: 1px solid rgba(255, 255, 255, 0.2); color: inherit; border-radius: 6px; padding: 4px 10px; font-size: 12px; cursor: pointer; }
  .actions button:hover { background: rgba(255, 255, 255, 0.2); }
  .actions .primary { background: #c93a30; border-color: #e0564b; }
  .actions .primary:hover { background: #dc4a3f; }
  .close { background: transparent; border: 0; color: rgba(255, 255, 255, 0.6); font-size: 18px; line-height: 1; cursor: pointer; }
  .close:hover { color: #fff; }
  .more { margin: 0; text-align: right; font-size: 11px; opacity: 0.7; }
  @keyframes rise { from { opacity: 0; transform: translateY(8px); } to { opacity: 1; transform: translateY(0); } }
  @media (prefers-reduced-motion: reduce) { .card { animation: none; } }
</style>
