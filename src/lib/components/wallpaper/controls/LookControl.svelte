<script lang="ts">
  // One control of a look, drawn by its kind: a slider, a choice between options, or a switch.
  import type { Control } from "../../../wallpaper/looks";
  import { portuguese } from "../strings";
  import ParamSlider from "./ParamSlider.svelte";
  import Segmented from "./Segmented.svelte";
  import ToggleRow from "./ToggleRow.svelte";

  let { item, value, onchange }: { item: Control; value: number; onchange: (value: number) => void } = $props();
  const label = $derived(portuguese ? item.label.pt : item.label.en);
</script>

{#if item.type === "slider"}
  <ParamSlider {label} {value} spec={item.spec} {onchange} />
{:else if item.type === "choice"}
  <Segmented
    {label}
    options={item.options.map((option) => ({ id: String(option.value), label: portuguese ? option.label.pt : option.label.en }))}
    value={String(value)}
    onchange={(id) => onchange(Number(id))}
  />
{:else}
  <ToggleRow {label} checked={value >= 0.5} onchange={(checked) => onchange(checked ? 1 : 0)} />
{/if}
