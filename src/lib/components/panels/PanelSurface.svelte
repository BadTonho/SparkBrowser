<script lang="ts">
  import { onMount, tick } from "svelte";
  import PanelViewNode from "./PanelViewNode.svelte";
  import { focusDialog } from "../../actions/focusDialog";
  let { surface }: { surface: "tools" | "downloads" } = $props();
  const downloads = $derived(surface === "downloads");
  const setter = $derived(downloads ? "__veloceSetDownloadsPopover" : "__veloceSetToolsPopover");
  import { panelPosition, isPanelTrigger, type PanelPresentation, type PanelViewAction } from "../../services/panel-presentation";
  import "../tools/tools.css";
  import "../downloads/downloads.css";

  type Payload = PanelPresentation & { visible: boolean; token: string; sessionId: number };
  let presentation = $state<Payload | null>(null);
  let root = $state<HTMLDivElement>(null!);
  const pending = new Map<number, number>();
  let sequence = 0;
  let closing = false;
  let closedSession = 0;
  const outbox: PanelViewAction[] = [];
  let inFlight: number | null = null;
  let viewport = $state({ width: innerWidth, height: innerHeight });
  const position = $derived(presentation ? panelPosition(presentation.anchor, viewport.width, viewport.height, downloads ? { width: 480, maxHeight: 560 } : undefined) : null);
  function downloadFocus(node: HTMLElement) { return downloads ? focusDialog(node, { onEscape: () => send({ kind: "dismiss", reason: "escape" }) }) : undefined; }
  function pump() {
    if (!presentation || inFlight !== null) return;
    const action = outbox.shift();
    if (!action) return;
    const next = ++sequence;
    inFlight = next;
    if ("target" in action && ["input", "change"].includes(action.kind) && !outbox.some(queued => "target" in queued && queued.target === action.target && ["input", "change"].includes(queued.kind))) pending.set(action.target, next);
    const url = new URL(`veloce-internal://${surface}-popover/action`);
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
        if (!presentation || presentation.sessionId <= value.sessionId) presentation = null;
        outbox.length = 0; inFlight = null;
        return;
      }
      if (value.sessionId <= closedSession) return;
      const opening = presentation?.sessionId !== value.sessionId;
      if (!opening && presentation && value.revision < presentation.revision) return;
      if (opening) { sequence = 0; pending.clear(); closing = false; outbox.length = 0; inFlight = null; }
      presentation = value;
      if (inFlight !== null && value.ack >= inFlight) inFlight = null;
      pump();
      document.documentElement.lang = value.locale;
      for (const [key, color] of Object.entries(value.theme)) document.documentElement.style.setProperty(key, color);
      if (opening) { await tick(); root?.querySelector<HTMLElement>(downloads ? "[data-testid=download-url]" : "[role=tab][aria-selected=true]")?.focus(); }
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
  <div class={downloads ? "downloads-dismiss" : "tools-dismiss"} role="presentation" onclick={event => {
    if (event.target === event.currentTarget) send({ kind: "dismiss", reason: downloads && presentation && isPanelTrigger(presentation.anchor, event.clientX, event.clientY) ? "trigger" : "outside" });
  }}>
    <div class={downloads ? "downloads-surface" : "tools-surface"} bind:this={root} use:downloadFocus style:left="{position.left}px" style:top="{position.top}px" style:width="{position.width}px" style:max-height="{position.maxHeight}px">
      <PanelViewNode node={presentation.tree} {pending} ack={presentation.ack} />
    </div>
  </div>
{/if}
