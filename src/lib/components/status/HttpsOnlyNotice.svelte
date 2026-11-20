<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { t } from "../../i18n";
  import { tabs } from "../../stores/browserStore";
  import { isTauriRuntime } from "../../services/bridge/runtime";
  import { allowHttpOnce } from "../../services/bridge/privacy-settings";
  import { logIgnored } from "../../services/ignored-error";
  import { describeError } from "../../services/user-errors.js";

  interface Blocked { tab_id: string; url: string }
  let blocked = $state<Blocked | null>(null);
  let failure = $state("");
  // The notice belongs to the tab that was blocked: it goes away when another tab is shown or the tab closes.
  const visible = $derived(blocked !== null && $tabs.some(tab => tab.id === blocked!.tab_id && tab.is_active));
  const host = $derived.by(() => { try { return blocked ? new URL(blocked.url).host : ""; } catch { return ""; } });

  $effect(() => { if (blocked && !$tabs.some(tab => tab.id === blocked!.tab_id)) blocked = null; });

  onMount(() => {
    if (!isTauriRuntime()) return;
    let disposed = false;
    let stop: (() => void) | undefined;
    void listen<Blocked>("https-only-blocked", event => { blocked = event.payload; failure = ""; })
      .then(unlisten => { if (disposed) unlisten(); else stop = unlisten; })
      .catch(logIgnored);
    return () => { disposed = true; stop?.(); };
  });

  async function continueOverHttp(): Promise<void> {
    if (!blocked) return;
    try {
      await allowHttpOnce(blocked.tab_id, blocked.url);
      blocked = null;
    } catch (cause) { failure = describeError(cause); }
  }
</script>

{#if visible}
  <div class="notice" role="alertdialog" aria-labelledby="https-only-title" aria-describedby="https-only-body" data-testid="https-only-notice">
    <div class="text">
      <strong id="https-only-title">{$t("status.https_only_title")}</strong>
      <span id="https-only-body">{$t("status.https_only_body", { host })}</span>
      {#if failure}<span class="failure" role="alert">{failure}</span>{/if}
    </div>
    <button type="button" class="go" onclick={continueOverHttp}>{$t("status.https_only_continue")}</button>
    <button type="button" class="close" onclick={() => { blocked = null; }} aria-label={$t("status.dismiss_error_2db0")}>×</button>
  </div>
{/if}

<style>
  .notice { position: fixed; top: 4px; left: 50%; transform: translateX(-50%); z-index: 3000; width: min(680px, 80vw); display: flex; align-items: center; gap: 12px; padding: 8px 12px; background: var(--bg-glass-elevated, #1c1c1c); color: var(--text-primary, #fff); border: 1px solid var(--border-subtle, #444); border-radius: 8px; font-size: 12px; }
  .text { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 2px; }
  .text span { color: var(--text-secondary, #bbb); line-height: 1.4; }
  .text .failure { color: #ffb4a8; }
  .go { flex: 0 0 auto; padding: 6px 10px; border-radius: 6px; border: 1px solid var(--border-subtle, #555); background: transparent; color: inherit; cursor: pointer; font: inherit; }
  .go:hover { background: rgba(255, 255, 255, 0.08); }
  .close { flex: 0 0 auto; color: inherit; background: transparent; border: 0; cursor: pointer; font-size: 18px; }
</style>
