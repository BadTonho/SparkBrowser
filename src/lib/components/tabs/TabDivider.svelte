<script lang="ts">
  import { t } from "../../i18n";
  import { slide } from "svelte/transition";
  import { cubicOut } from "svelte/easing";
  import { showTabDividerContextMenu } from "../../stores/persistenceStore";
  import { editingDividerId, renameTabDivider, type TabDivider } from "../../stores/tabDividersStore";

  /** `locate` turns a pointer height into the tab id the divider should sit below ("" = top). */
  let { divider, order, locate, move: reorder }: { divider: TabDivider; order: number; locate: (y: number) => string; move: (id: string, after: string) => Promise<void> } = $props();
  let dragging = $state(false);
  let editing = $derived($editingDividerId === divider.id);
  let draft = $state("");

  $effect(() => { if (editing) draft = divider.name; });

  function focusInput(node: HTMLInputElement) { node.focus(); node.select(); }
  function commit() {
    if ($editingDividerId !== divider.id) return;
    renameTabDivider(divider.id, draft);
    editingDividerId.set(null);
  }
  function onKey(event: KeyboardEvent) {
    event.stopPropagation();
    if (event.key === "Enter") { event.preventDefault(); commit(); }
    else if (event.key === "Escape") { event.preventDefault(); editingDividerId.set(null); }
  }
  let node: HTMLElement;
  let grab = 0, lastY = 0, offset = 0, busy = false;
  const reducedMotion = () => typeof matchMedia === "function" && matchMedia("(prefers-reduced-motion: reduce)").matches;
  function down(event: PointerEvent) {
    if (event.button !== 0 || editing) return;
    dragging = true;
    lastY = event.clientY;
    const box = node.getBoundingClientRect();
    grab = event.clientY - (box.top + box.height / 2);
    node.setPointerCapture(event.pointerId);
    event.preventDefault();
  }
  /** Keeps the divider under the pointer, whatever slot of the list it currently occupies. */
  function follow() {
    node.style.transform = "";
    const box = node.getBoundingClientRect();
    offset = lastY - grab - (box.top + box.height / 2);
    node.style.transform = `translateY(${offset}px)`;
  }
  async function move(event: PointerEvent) {
    if (!dragging) return;
    lastY = event.clientY;
    if (busy) { follow(); return; }
    const after = locate(lastY);
    if (after !== divider.after) {
      busy = true;
      try { await reorder(divider.id, after); } finally { busy = false; }
    }
    if (dragging) follow();
  }
  function up(event: PointerEvent) {
    if (!dragging) return;
    dragging = false;
    if (node.hasPointerCapture(event.pointerId)) node.releasePointerCapture(event.pointerId);
    const from = offset;
    node.style.transform = "";
    if (!reducedMotion() && Math.abs(from) > 0.5) {
      node.animate([{ transform: `translateY(${from}px)` }, { transform: "translateY(0)" }], { duration: 200, easing: "cubic-bezier(0.22, 1, 0.36, 1)" });
    }
  }
</script>

<div
  bind:this={node}
  style:order={order}
  class="tab-divider"
  data-tab-id={`divider:${divider.id}`}
  class:dragging
  class:named={divider.name || editing}
  role="separator"
  aria-label={divider.name || undefined}
  title={$t("tabs.divider_name")}
  onpointerdown={down}
  onpointermove={move}
  onpointerup={up}
  onpointercancel={up}
  ondblclick={() => editingDividerId.set(divider.id)}
  oncontextmenu={(event) => showTabDividerContextMenu(event, divider.id)}
  transition:slide={{ duration: 160, easing: cubicOut }}
>
  <span class="line"></span>
  {#if editing}
    <input
      class="name-input"
      maxlength="40"
      aria-label={$t("tabs.divider_name")}
      placeholder={$t("tabs.divider_name")}
      bind:value={draft}
      use:focusInput
      onkeydown={onKey}
      onblur={commit}
      onpointerdown={(event) => event.stopPropagation()}
    />
    <span class="line"></span>
  {:else if divider.name}
    <span class="name">{divider.name}</span>
    <span class="line"></span>
  {/if}
</div>

<style>
  .tab-divider {
    flex: none;
    display: flex;
    align-items: center;
    gap: 8px;
    min-height: 16px;
    margin: 2px 6px;
    padding: 4px 4px;
    cursor: grab;
    user-select: none;
    touch-action: none;
    border-radius: var(--radius-sm);
    transition: background 0.15s ease, box-shadow 0.15s ease;
  }
  .tab-divider:hover, .tab-divider.dragging { background: rgba(255, 255, 255, 0.05); }
  .tab-divider.dragging {
    position: relative;
    z-index: 5;
    cursor: grabbing;
    background: rgba(255, 255, 255, 0.09);
    box-shadow: 0 6px 18px rgba(0, 0, 0, 0.35);
  }
  .line { flex: 1; height: 1px; background: var(--border-subtle, rgba(255, 255, 255, 0.16)); }
  .named .line:first-child { flex: 0 0 10px; }
  .name {
    flex: 0 1 auto;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 10.5px;
    font-weight: 700;
    letter-spacing: 0.6px;
    color: var(--text-muted);
  }
  .name-input {
    flex: 0 1 140px;
    min-width: 0;
    padding: 2px 6px;
    border: 1px solid color-mix(in srgb, var(--accent-primary) 45%, transparent);
    border-radius: 6px;
    background: rgba(255, 255, 255, 0.06);
    color: var(--text-primary);
    font: inherit;
    font-size: 11px;
    outline: none;
  }
</style>
