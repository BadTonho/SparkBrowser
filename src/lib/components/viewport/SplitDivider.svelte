<script lang="ts">
  import { onDestroy } from "svelte";
  import { splitLayout, splitDragRatio, splitKey } from "../../stores/splitViewStore";
  import { commitSplitRatio } from "../../services/bridge/split-view";
  import { applyTabsSnapshot } from "../../stores/browserStore";
  let { frame, flush }: { frame: HTMLDivElement; flush: () => Promise<void> } = $props();
  let error = $state(""); let dragging = $state(false); let dragKey = "";
  const ratio = $derived($splitDragRatio ?? $splitLayout?.ratio ?? 0.5);
  function change(event: PointerEvent) {
    if (!dragging) return;
    const box = frame.getBoundingClientRect();
    splitDragRatio.set(Math.max(.2, Math.min(.8, (event.clientX - box.left - 4) / Math.max(200, box.width - 8))));
  }
  async function commit() {
    const key = dragKey; const value = ratio; dragging = false;
    try { await flush(); applyTabsSnapshot(await commitSplitRatio(key, value)); error = ""; }
    catch (e) { error = String(e); }
    finally { splitDragRatio.set(null); }
  }
  onDestroy(() => splitDragRatio.set(null));
</script>
{#if $splitLayout}
  <!-- A focusable ARIA separator implements the window splitter keyboard pattern. -->
  <!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
  <div class="split-divider" role="separator" aria-label="Resize split view" aria-orientation="vertical" aria-valuemin="20" aria-valuemax="80" aria-valuenow={Math.round(ratio * 100)} tabindex="0"
    style:left={`clamp(100px, calc(${ratio * 100}% - ${ratio * 8}px), calc(100% - 108px))`}
    onpointerdown={(event) => { dragging = true; dragKey = splitKey($splitLayout!); event.currentTarget.setPointerCapture(event.pointerId); change(event); }}
    onpointermove={change} onpointerup={() => { if (dragging) void commit(); }} onpointercancel={() => { dragging = false; splitDragRatio.set(null); }}
    onkeydown={(event) => { if (!["ArrowLeft", "ArrowRight", "Home"].includes(event.key)) return; event.preventDefault(); dragKey = splitKey($splitLayout!); splitDragRatio.set(event.key === "Home" ? .5 : Math.max(.2, Math.min(.8, ratio + (event.key === "ArrowLeft" ? -.05 : .05)))); void commit(); }}></div>
{/if}
{#if error}<p class="split-error" role="alert">{error}</p>{/if}
<style>
  .split-divider { position: absolute; top: 0; bottom: 0; width: 8px; background: var(--border-subtle, #555); cursor: col-resize; touch-action: none; }
  .split-divider:focus-visible { outline: 2px solid var(--accent-primary, #a99aff); outline-offset: -2px; }
  .split-error { position: absolute; top: 0; left: 0; padding: 8px; background: #351515; color: white; }
</style>
