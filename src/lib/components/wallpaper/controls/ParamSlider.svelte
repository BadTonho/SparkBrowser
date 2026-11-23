<script lang="ts">
  import type { Spec } from "../../../wallpaper/params";
  let { label, value, spec, format, onchange }: { label: string; value: number; spec: Spec; format?: (value: number) => string; onchange: (value: number) => void } = $props();
  const fill = $derived(((value - spec.min) / (spec.max - spec.min)) * 100);
  const text = $derived(format ? format(value) : value.toFixed(spec.decimals));
</script>

<label class="slider">
  <span class="head"><span>{label}</span><output>{text}</output></span>
  <input type="range" min={spec.min} max={spec.max} step={spec.step} {value} style:--fill={`${fill}%`} oninput={(event) => onchange(Number(event.currentTarget.value))} />
</label>

<style>
  .slider { display: block; margin: 0 0 14px; }
  .head { display: flex; align-items: baseline; justify-content: space-between; margin: 0 0 6px; font-size: 12px; }
  output { font-variant-numeric: tabular-nums; }
  input { appearance: none; display: block; width: 100%; height: 16px; margin: 0; background: transparent; cursor: pointer; }
  input::-webkit-slider-runnable-track { height: 4px; border-radius: 99px; background: linear-gradient(to right, var(--text) var(--fill), var(--track) var(--fill)); }
  input::-webkit-slider-thumb { appearance: none; width: 16px; height: 16px; margin-top: -6px; border: 0; border-radius: 50%; background: var(--text); box-shadow: 0 1px 4px rgba(0, 0, 0, 0.45); transition: transform 0.12s ease; }
  input:hover::-webkit-slider-thumb { transform: scale(1.12); }
  input::-moz-range-track { height: 4px; border-radius: 99px; background: var(--track); }
  input::-moz-range-progress { height: 4px; border-radius: 99px; background: var(--text); }
  input::-moz-range-thumb { width: 16px; height: 16px; border: 0; border-radius: 50%; background: var(--text); }
  input:focus-visible { outline: 2px solid var(--focus); outline-offset: 3px; border-radius: 6px; }
</style>
