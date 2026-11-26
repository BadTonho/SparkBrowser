<script lang="ts">
  import { onMount, tick } from "svelte";
  import PanelViewNode from "./PanelViewNode.svelte";
  import { focusDialog } from "../../actions/focusDialog";
  import { syncDownloadFocus } from "../../services/download-panel-focus";
  let { surface }: { surface: "tools" | "downloads" | "settings-quick" | "suggestions" | "commands" | "resources" | "video-download" } = $props();
  const downloads = $derived(surface === "downloads");
  const quickSettings = $derived(surface === "settings-quick");
  // The address bar's suggestions: a list below the address bar that never takes the keyboard.
  const suggestions = $derived(surface === "suggestions");
  // The command bar: a window-wide layer above the page. The page shows, dimmed, behind it.
  const commands = $derived(surface === "commands");
  // The resources panel: a popover under its button, like the quick settings.
  const resources = $derived(surface === "resources");
  const video = $derived(surface === "video-download");
  const setter = $derived(video ? "__sparkSetVideoDownloadPopover" : downloads ? "__sparkSetDownloadsPopover" : quickSettings ? "__sparkSetQuickSettingsPopover" : suggestions ? "__sparkSetSuggestionsPopover" : commands ? "__sparkSetCommandsPopover" : resources ? "__sparkSetResourcesPopover" : "__sparkSetToolsPopover");
  import { panelPosition, isPanelTrigger, type PanelPresentation, type PanelViewAction } from "../../services/panel-presentation";
  import "../tools/tools.css";
  import "../downloads/downloads.css";
  import "../settings/settings-quick.css";
  import "../titlebar/suggestions.css";
  import "../commandbar/commandbar.css";
  import "../resources/resources.css";
  import "../video-download/video-download.css";
  import "./popover-motion.css";

  type Payload = PanelPresentation & { visible: boolean; token: string; sessionId: number };
  let presentation = $state<Payload | null>(null);
  let root = $state<HTMLDivElement>(null!);
  const pending = new Map<number, number>();
  let sequence = 0;
  let closing = false;
  let closedSession = 0;
  let presentationGeneration = 0;
  let pendingOpeningFocus = false;
  const outbox: PanelViewAction[] = [];
  let inFlight: number | null = null;
  let viewport = $state({ width: innerWidth, height: innerHeight });
  // This web view starts at the bottom of the address bar, so the list is placed from its top edge, at the bar's own width.
  const position = $derived(presentation ? (commands
    ? { left: 0, top: 0, width: viewport.width, maxHeight: viewport.height }
    : suggestions
    ? { left: presentation.anchor.left, top: 4, width: presentation.anchor.width, maxHeight: Math.max(0, viewport.height - 8) }
    : panelPosition(presentation.anchor, viewport.width, viewport.height, downloads ? { width: 480, maxHeight: 560 } : quickSettings ? { width: 300, maxHeight: 560 } : resources ? { width: 340, maxHeight: 560 } : video ? { width: 360, maxHeight: 560 } : undefined)) : null);
  function downloadFocus(node: HTMLElement) { return downloads || quickSettings || resources || video ? focusDialog(node, { onEscape: () => send({ kind: "dismiss", reason: "escape" }) }) : undefined; }
  function pump() {
    if (!presentation || inFlight !== null) return;
    const action = outbox.shift();
    if (!action) return;
    const next = ++sequence;
    inFlight = next;
    if ("target" in action && ["input", "change"].includes(action.kind) && !outbox.some(queued => "target" in queued && queued.target === action.target && ["input", "change"].includes(queued.kind))) pending.set(action.target, next);
    const url = new URL(`spark-internal://${surface}-popover/action`);
    url.searchParams.set("token", presentation.token);
    url.searchParams.set("sequence", String(next));
    url.searchParams.set("payload", JSON.stringify(action));
    location.href = url.toString();
  }
  function send(action: PanelViewAction) {
    if (!presentation || closing) return;
    if (action.kind === "dismiss") closing = true;
    const last = outbox.at(-1);
    if ((action.kind === "input" || action.kind === "change")) {
      pending.set(action.target, Infinity);
      if (last?.kind === action.kind && "target" in last && last.target === action.target) outbox.pop();
    }
    outbox.push(action);
    pump();
  }
  onMount(() => {
    const set = async (value: Payload) => {
      if (!value.visible) {
        closedSession = Math.max(closedSession, value.sessionId);
        if (presentation && presentation.sessionId > value.sessionId) return;
        presentationGeneration++;
        pendingOpeningFocus = false;
        presentation = null;
        outbox.length = 0; inFlight = null;
        return;
      }
      if (value.sessionId <= closedSession) return;
      const opening = presentation?.sessionId !== value.sessionId;
      if (!opening && presentation && value.revision < presentation.revision) return;
      const generation = ++presentationGeneration;
      if (opening) { sequence = 0; pending.clear(); closing = false; outbox.length = 0; inFlight = null; pendingOpeningFocus = true; }
      const hadForm = !!root?.querySelector('[data-testid="download-url"]');
      presentation = value;
      if (inFlight !== null && value.ack >= inFlight) inFlight = null;
      pump();
      document.documentElement.lang = value.locale;
      for (const [key, color] of Object.entries(value.theme)) document.documentElement.style.setProperty(key, color);
      await tick();
      if (generation !== presentationGeneration || closing) return;
      const focusOpening = pendingOpeningFocus;
      pendingOpeningFocus = false;
      if (downloads) syncDownloadFocus(root, focusOpening, hadForm);
      else if (quickSettings && focusOpening) root?.querySelector<HTMLElement>('[data-testid="settings-quick-theme-dark"], [data-testid="settings-quick-theme-light"]')?.focus();
      else if (video && focusOpening) root?.querySelector<HTMLElement>(".video-download-panel button")?.focus();
      else if (resources && focusOpening) root?.querySelector<HTMLElement>(".resources-panel button")?.focus();
      else if (focusOpening && commands) { const field = root?.querySelector<HTMLInputElement>(".command-input"); field?.focus(); field?.select(); }
      else if (focusOpening && !suggestions) root?.querySelector<HTMLElement>("[role=tab][aria-selected=true]")?.focus();
      // The highlight moves in the shell; this copy only has to keep the highlighted row in view.
      if (commands) root?.querySelector<HTMLElement>(".command-item.selected")?.scrollIntoView({ block: "nearest" });
    };
    (window as unknown as Record<string, typeof set>)[setter] = set;
    const click = (event: MouseEvent) => {
      const element = event.target instanceof Element ? event.target.closest<HTMLElement>("button[data-panel-control],summary[data-panel-control]") : null;
      if (!element || element.matches(":disabled")) return;
      event.preventDefault(); event.stopPropagation();
      send({ kind: "click", target: Number(element.dataset.panelControl) });
    };
    const edit = (event: Event) => {
      const target = event.target;
      if (!(target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement || target instanceof HTMLSelectElement)) return;
      send({ kind: event.type as "input" | "change", target: Number(target.dataset.panelControl), value: target.value, checked: target instanceof HTMLInputElement && target.checked });
    };
    const submit = (event: SubmitEvent) => { event.preventDefault(); const target = event.target as HTMLFormElement; send({ kind: "submit", target: Number(target.dataset.panelControl) }); };
    const keys = (event: KeyboardEvent) => {
      if (event.key === "Escape") { event.preventDefault(); send({ kind: "dismiss", reason: "escape" }); }
      const target = event.target as HTMLElement;
      // The shortcut that opens the bar also closes it.
      if (commands && (event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") { event.preventDefault(); send({ kind: "dismiss", reason: "escape" }); }
      // The bar's own key handling (moving the highlight, running the chosen row) lives in the shell.
      if (commands && target.matches?.(".command-input") && ["ArrowDown", "ArrowUp", "Enter"].includes(event.key)) {
        event.preventDefault();
        send({ kind: "keyboard", target: Number(target.dataset.panelControl), key: event.key, ctrl: event.ctrlKey, alt: event.altKey, shift: event.shiftKey, meta: event.metaKey });
      }
      if (target.getAttribute?.("role") === "tab" && ["ArrowLeft", "ArrowRight", "Home", "End"].includes(event.key)) {
        event.preventDefault();
        const items = [...root.querySelectorAll<HTMLElement>("[role=tab]")];
        const i = items.indexOf(target);
        const next = items[event.key === "Home" ? 0 : event.key === "End" ? items.length - 1 : (i + (event.key === "ArrowRight" ? 1 : -1) + items.length) % items.length];
        next.focus(); send({ kind: "click", target: Number(next.dataset.panelControl) });
      }
    };
    document.addEventListener("click", click, true);
    document.addEventListener("input", edit);
    document.addEventListener("change", edit);
    document.addEventListener("submit", submit);
    document.addEventListener("keydown", keys);
    return () => {
      document.removeEventListener("click", click, true); document.removeEventListener("input", edit);
      document.removeEventListener("change", edit); document.removeEventListener("submit", submit); document.removeEventListener("keydown", keys);
    };
  });
</script>

<svelte:window onresize={() => { viewport = { width: innerWidth, height: innerHeight }; }} />
{#if presentation && position}
  <div class={downloads ? "downloads-dismiss" : quickSettings ? "settings-quick-dismiss" : resources ? "resources-dismiss" : video ? "video-download-dismiss" : suggestions ? "suggestions-dismiss" : commands ? "commands-dismiss" : "tools-dismiss"} role="presentation" onclick={event => {
    if (event.target === event.currentTarget) send({ kind: "dismiss", reason: (downloads || quickSettings || resources || video) && presentation && isPanelTrigger(presentation.anchor, event.clientX, event.clientY) ? "trigger" : "outside" });
  }}>
    <div class="{commands ? "" : "popover-enter "}{downloads ? "downloads-surface" : quickSettings ? "settings-quick-surface" : resources ? "resources-surface" : video ? "video-download-surface" : suggestions ? "suggestions-surface" : commands ? "commands-surface" : "tools-surface"}" bind:this={root} use:downloadFocus style:left={commands ? undefined : `${position.left}px`} style:top={commands ? undefined : `${position.top}px`} style:width={commands ? undefined : `${position.width}px`} style:max-height={commands ? undefined : `${position.maxHeight}px`} style:height={resources ? `${position.maxHeight}px` : undefined}>
      <PanelViewNode node={presentation.tree} {pending} ack={presentation.ack} />
    </div>
  </div>
{/if}
