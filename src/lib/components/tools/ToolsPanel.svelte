<script lang="ts">
  import { toolsPosition } from "../../services/tools-presentation";
  import { isTauriRuntime } from "../../services/bridge/runtime";
  import { mountToolsHost } from "./tools-host";
  import { t } from "../../i18n";
  import { focusDialog } from "../../actions/focusDialog";
  import { isToolsOpen, toolsSession, toolsSection, closeTools, type ToolsSection } from "../../stores/toolsStore";
  import TabDiagnosticsPanel from "./TabDiagnosticsPanel.svelte";
  import ClosedTabsPanel from "./ClosedTabsPanel.svelte";
  import TabsPanel from "./TabsPanel.svelte";
  import { selectedSession } from "../../stores/toolsContentStore";
  import SessionDetails from "./SessionDetails.svelte";
  import SessionsPanel from "./SessionsPanel.svelte";
  import NotesPanel from "./NotesPanel.svelte";
  import FeedsPanel from "./FeedsPanel.svelte";
  import "./tools.css";
  const sections: { id: ToolsSection; label: string }[] = $derived([{ id: "diagnostics", label: $t("diagnostics.heading") }, { id: "sessions", label: $t("tools.sessions_6fa3") }, { id: "tabs", label: $t("tools.tabs_8e5e") }, { id: "notes", label: $t("tools.notes_8a75") }, { id: "feeds", label: $t("tools.feeds") }, { id: "closed", label: $t("closed.heading") }]);
  const native = isTauriRuntime();
  let viewport = $state({ width: innerWidth, height: innerHeight });
  let anchor = $state({ left: 0, top: 0, width: 0, height: 0 });
  const position = $derived(toolsPosition(anchor, viewport.width, viewport.height));
  function host(node: HTMLElement) {
    const stop = mountToolsHost(node, native);
    const focus = native ? undefined : focusDialog(node, { onEscape: () => closeTools(true) });
    let frame = 0;
    const measure = () => {
      const r = $toolsSession?.anchor.getBoundingClientRect();
      if (r && (anchor.left !== r.left || anchor.top !== r.top || anchor.width !== r.width || anchor.height !== r.height)) anchor = { left:r.left, top:r.top, width:r.width, height:r.height };
      frame = requestAnimationFrame(measure);
    };
    if (!native) measure();
    return { destroy() { cancelAnimationFrame(frame); stop(); focus?.destroy(); } };
  }
  function tabKeys(event: KeyboardEvent) {
    if (!["ArrowLeft", "ArrowRight", "Home", "End"].includes(event.key)) return;
    event.preventDefault();
    const index = sections.findIndex(s => s.id === $toolsSection);
    const next = event.key === "Home" ? 0 : event.key === "End" ? sections.length - 1 : (index + (event.key === "ArrowRight" ? 1 : -1) + sections.length) % sections.length;
    toolsSection.set(sections[next].id);
    if (!native) document.querySelector<HTMLButtonElement>(`[data-testid="tools-${sections[next].id}"]`)?.focus();
  }
</script>

<svelte:window onresize={() => { viewport = { width: innerWidth, height: innerHeight }; }} />

{#if $isToolsOpen}
  <div class="tools-dismiss" class:tools-controller={native} aria-hidden={native ? "true" : undefined} role="presentation" onclick={(event) => { if (event.target === event.currentTarget) void closeTools(); }}>
    <div class="tools-surface" style:left="{position.left}px" style:top="{position.top}px" style:width="{position.width}px" style:max-height="{position.maxHeight}px">
    <div data-testid="tools-dialog" class="tools-panel" role="dialog" aria-modal="false" aria-label={$t("tools.tools_ea93")} tabindex="-1" use:host>
      <header class="tools-header"><h2>{$t("tools.tools_ea93")}</h2>
        <button class="tools-close" onclick={() => closeTools(true)} data-testid="close-tools" aria-label={$t("tools.close_tools_3f06")} title={$t("tools.close_tools_3f06")}>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" aria-hidden="true" focusable="false"><path d="m6 6 12 12M18 6 6 18" /></svg>
        </button>
      </header>
      <div class="tools-tabs" role="tablist" aria-label={$t("tools.tools_sections_9c6a")}>
        {#each sections as section}<button role="tab" id={`tools-tab-${section.id}`} data-testid={`tools-${section.id}`} aria-selected={$toolsSection === section.id} aria-controls="tools-content" tabindex={$toolsSection === section.id ? 0 : -1} onkeydown={tabKeys} onclick={() => toolsSection.set(section.id)}>{section.label}</button>{/each}
      </div>
      <div class="tools-content" id="tools-content" role="tabpanel" aria-labelledby={`tools-tab-${$toolsSection}`}>
        {#if $toolsSection === "diagnostics"}<TabDiagnosticsPanel />
        {:else if $toolsSection === "sessions"}{#if $selectedSession}<SessionDetails />{:else}<SessionsPanel />{/if}
        {:else if $toolsSection === "notes"}<NotesPanel />
        {:else if $toolsSection === "feeds"}<FeedsPanel />
        {:else if $toolsSection === "closed"}<ClosedTabsPanel />
        {:else}<TabsPanel />{/if}
      </div>
    </div>
    </div>
  </div>
{/if}
