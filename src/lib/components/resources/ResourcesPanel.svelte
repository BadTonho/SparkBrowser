<script lang="ts">
  import { onMount } from "svelte";
  import { isTauriRuntime } from "../../services/bridge/runtime";
  import { focusDialog } from "../../actions/focusDialog";
  import { panelPosition } from "../../services/panel-presentation";
  import { mountPanelHost } from "../panels/panel-host";
  import { onResourcesPopoverAction, setResourcesPopover } from "../../services/bridge/resources-popover";
  import { t, type MessageKey } from "../../i18n";
  import { describeError } from "../../services/user-errors.js";
  import { clearEngineData } from "../../services/browser-data";
  import { isResourcesOpen, resourcesSession, closeResourcesPanel } from "../../stores/resourcesPopoverStore";
  import { tabs } from "../../stores/browserStore";
  import { tabsMemory, refreshTabsMemory } from "../../stores/tabMemoryStore";
  import { wipeBrowsingHistory } from "../../stores/persistenceStore";
  import { wipeDownloads } from "../../stores/downloadsStore";
  import "./resources.css";

  type Choice = "cache" | "site" | "history" | "downloads";
  const CHOICES: { id: Choice; title: MessageKey; hint: MessageKey }[] = [
    { id: "cache", title: "resources.item_cache", hint: "resources.item_cache_hint" },
    { id: "site", title: "resources.item_site", hint: "resources.item_site_hint" },
    { id: "history", title: "resources.item_history", hint: "resources.item_history_hint" },
    { id: "downloads", title: "resources.item_downloads", hint: "resources.item_downloads_hint" },
  ];

  const native = isTauriRuntime();
  let viewport = $state({ width: innerWidth, height: innerHeight });
  let anchor = $state({ left: 0, top: 0, width: 0, height: 0 });
  const position = $derived(panelPosition(anchor, viewport.width, viewport.height, { width: 340, maxHeight: 560 }));

  let view = $state<"main" | "clean">("main");
  let chosen = $state<Record<Choice, boolean>>({ cache: true, site: false, history: false, downloads: false });
  let confirming = $state(false);
  let busy = $state(false);
  let result = $state<{ text: string; failed: boolean } | null>(null);
  const anyChosen = $derived(Object.values(chosen).some(Boolean));

  const resident = $derived($tabs.filter(tab => !tab.is_hibernated).length);
  const sleeping = $derived($tabs.length - resident);
  const usedBytes = $derived(Object.values($tabsMemory).reduce((total, bytes) => total + (bytes > 0 ? bytes : 0), 0));
  const usedText = $derived(usedBytes >= 1024 ** 3 ? `${(usedBytes / 1024 ** 3).toFixed(1)} GB` : `${Math.round(usedBytes / 1024 ** 2)} MB`);

  // A new opening starts from the main view with nothing half done.
  $effect(() => {
    if ($isResourcesOpen) {
      view = "main"; confirming = false; result = null; busy = false;
      void refreshTabsMemory(true);
      const timer = setInterval(() => void refreshTabsMemory(true), 4000);
      return () => clearInterval(timer);
    }
  });

  function host(node: HTMLElement) {
    const stop = native ? mountPanelHost(node, true, {
      id: "resources",
      sessionStore: resourcesSession,
      close: (restore) => closeResourcesPanel(Boolean(restore)),
      set: setResourcesPopover,
      listen: onResourcesPopoverAction,
    }) : () => {};
    const focus = native ? undefined : focusDialog(node, { onEscape: () => closeResourcesPanel(true) });
    let frame = 0;
    const measure = () => {
      const rect = $resourcesSession?.anchor.getBoundingClientRect();
      if (rect && (anchor.left !== rect.left || anchor.top !== rect.top || anchor.width !== rect.width || anchor.height !== rect.height)) {
        anchor = { left: rect.left, top: rect.top, width: rect.width, height: rect.height };
      }
      frame = requestAnimationFrame(measure);
    };
    if (!native) measure();
    return { destroy() { cancelAnimationFrame(frame); stop(); focus?.destroy(); } };
  }

  function toggle(id: Choice) {
    chosen[id] = !chosen[id];
    confirming = false;
    result = null;
  }

  async function run() {
    if (!anyChosen || busy) return;
    if (!confirming) { confirming = true; return; }
    busy = true; confirming = false; result = null;
    const done: string[] = [];
    const failed: string[] = [];
    const attempt = async (name: string, action: () => Promise<unknown>) => {
      try { await action(); done.push(name); }
      catch (error) { failed.push(`${name} (${describeError(error)})`); }
    };
    if (chosen.history) await attempt($t("resources.item_history"), wipeBrowsingHistory);
    if (chosen.downloads) await attempt($t("resources.item_downloads"), wipeDownloads);
    if (chosen.cache || chosen.site) {
      const names = [chosen.cache ? $t("resources.item_cache") : "", chosen.site ? $t("resources.item_site") : ""].filter(Boolean).join(", ");
      await attempt(names, async () => {
        const report = await clearEngineData({ cache: chosen.cache, siteData: chosen.site });
        if (report.failed > 0) throw new Error($t("resources.partial"));
      });
    }
    busy = false;
    result = failed.length
      ? { text: $t("resources.failed", { items: failed.join("; ") }), failed: true }
      : { text: $t("resources.cleared", { items: done.join(", ") }), failed: false };
  }

  onMount(() => () => {});
</script>

<svelte:window onresize={() => { viewport = { width: innerWidth, height: innerHeight }; }} />

{#if $isResourcesOpen}
  <div
    class="resources-dismiss"
    class:resources-controller={native}
    aria-hidden={native ? "true" : undefined}
    role="presentation"
    onclick={(event) => { if (event.target === event.currentTarget) void closeResourcesPanel(false); }}
  >
    <div class="resources-surface" style:left="{position.left}px" style:top="{position.top}px" style:width="{position.width}px" style:max-height="{position.maxHeight}px">
      <div class="resources-panel" data-testid="resources-panel" role="dialog" aria-modal="false" aria-label={$t("resources.title")} tabindex="-1" use:host>
        {#if view === "main"}
          <div class="resources-head"><h2>{$t("resources.title")}</h2></div>

          <div class="resources-section">
            <h3>{$t("resources.memory")}</h3>
            <div class="resources-meter" data-testid="resources-memory">
              <span class="resources-meter-value">{usedBytes > 0 ? usedText : "—"}</span>
              <span class="resources-meter-note">{$t("resources.memory_tabs", { open: resident, sleeping })}</span>
            </div>
          </div>

          <div class="resources-section">
            <button class="resources-row" data-testid="resources-open-clean" onclick={() => { view = "clean"; confirming = false; result = null; }}>
              <span class="resources-row-text"><strong>{$t("resources.clean")}</strong><small>{$t("resources.clean_hint")}</small></span>
              <span class="resources-chevron" aria-hidden="true">›</span>
            </button>
          </div>
        {:else}
          <div class="resources-head">
            <button class="resources-back" data-testid="resources-back" aria-label={$t("resources.back")} onclick={() => { view = "main"; confirming = false; }}>‹</button>
            <h2>{$t("resources.clean")}</h2>
          </div>

          <div class="resources-section" role="group" aria-label={$t("resources.clean")}>
            {#each CHOICES as choice (choice.id)}
              <button
                class="resources-choice"
                data-testid="resources-choice-{choice.id}"
                aria-pressed={chosen[choice.id]}
                disabled={busy}
                onclick={() => toggle(choice.id)}
              >
                <span class="resources-box" aria-hidden="true">✓</span>
                <span class="resources-row-text"><strong>{$t(choice.title)}</strong><small>{$t(choice.hint)}</small></span>
              </button>
            {/each}
          </div>

          {#if chosen.site}<p class="resources-note">{$t("resources.site_warning")}</p>{/if}
          {#if result}<p class="resources-result" class:failed={result.failed} role="status">{result.text}</p>{/if}

          <button
            class="resources-action"
            class:danger={confirming}
            data-testid="resources-clear"
            disabled={!anyChosen || busy}
            onclick={run}
          >{busy ? $t("resources.clearing") : confirming ? $t("resources.confirm") : $t("resources.clear_selected")}</button>
        {/if}
      </div>
    </div>
  </div>
{/if}
