<script lang="ts">
  import { stopRange } from "../../../wallpaper/params";

  /** The gradient with one handle per active color; a handle moves between its neighbours. */
  let { stops, colors, active, gradient, label, stopLabel, onmove }: {
    stops: number[]; colors: string[]; active: number; gradient: string; label: string; stopLabel: string;
    onmove: (index: number, position: number) => void;
  } = $props();
  let bar = $state<HTMLDivElement>();
  let dragging = $state(-1);

  const place = (index: number, position: number) => {
    const [low, high] = stopRange(stops, index, active);
    onmove(index, Math.min(high, Math.max(low, position)));
  };
  function pointerPosition(event: PointerEvent) {
    const box = bar!.getBoundingClientRect();
    return box.width > 0 ? (event.clientX - box.left) / box.width : 0;
  }
  function down(event: PointerEvent, index: number) {
    if (event.button !== 0) return;
    dragging = index;
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
    event.preventDefault();
  }
  const move = (event: PointerEvent, index: number) => { if (dragging === index) place(index, pointerPosition(event)); };
  const up = () => { dragging = -1; };
  function key(event: KeyboardEvent, index: number) {
    const amount = event.shiftKey ? 0.05 : 0.01;
    if (event.key === "ArrowLeft" || event.key === "ArrowDown") place(index, stops[index] - amount);
    else if (event.key === "ArrowRight" || event.key === "ArrowUp") place(index, stops[index] + amount);
    else return;
    event.preventDefault();
  }
</script>

<div class="stops" role="group" aria-label={label}>
  <div class="bar" bind:this={bar} style:background={gradient}>
    {#each { length: active } as _, index (index)}
      <button
        type="button"
        class="handle"
        class:dragging={dragging === index}
        role="slider"
        aria-label={`${stopLabel} ${index + 1}`}
        aria-valuemin="0"
        aria-valuemax="100"
        aria-valuenow={Math.round(stops[index] * 100)}
        style:left={`${stops[index] * 100}%`}
        style:--stop={colors[index]}
        onpointerdown={(event) => down(event, index)}
        onpointermove={(event) => move(event, index)}
        onpointerup={up}
        onpointercancel={up}
        onkeydown={(event) => key(event, index)}
      ></button>
    {/each}
  </div>
</div>

<style>
  .stops { padding: 0 9px; margin: 0 0 16px; }
  .bar { position: relative; height: 26px; border-radius: 99px; box-shadow: inset 0 0 0 1px var(--item-border); }
  .handle { position: absolute; top: 50%; width: 18px; height: 18px; margin: -9px 0 0 -9px; padding: 0; border: 2px solid #ffffff; border-radius: 50%; background: var(--stop); box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.5), 0 2px 5px rgba(0, 0, 0, 0.4); cursor: grab; touch-action: none; transition: transform 0.12s ease; }
  .handle:hover, .handle.dragging { transform: scale(1.15); }
  .handle.dragging { cursor: grabbing; z-index: 2; }
  .handle:focus-visible { outline: 2px solid var(--focus); outline-offset: 3px; }
</style>
