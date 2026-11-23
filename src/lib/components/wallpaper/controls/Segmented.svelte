<script lang="ts" generics="T extends string">
  let { label, options, value, onchange }: { label: string; options: { id: T; label: string }[]; value: T; onchange: (value: T) => void } = $props();
</script>

<div class="segmented" role="radiogroup" aria-label={label} style:grid-template-columns={`repeat(${options.length}, 1fr)`}>
  {#each options as option (option.id)}
    <button type="button" role="radio" aria-checked={value === option.id} class:selected={value === option.id} onclick={() => onchange(option.id)}>{option.label}</button>
  {/each}
</div>

<style>
  .segmented { display: grid; gap: 6px; margin: 0 0 14px; }
  button { height: 34px; padding: 0 6px; border: 1px solid var(--item-border); border-radius: 10px; background: var(--item-bg); color: var(--muted); font: inherit; font-size: 11px; font-weight: 700; cursor: pointer; transition: background 0.15s ease, color 0.15s ease; }
  button:hover { background: var(--item-hover); color: var(--text); }
  button.selected { background: var(--selected-bg); border-color: var(--selected-bg); color: var(--selected-text); }
  button:focus-visible { outline: 2px solid var(--focus); outline-offset: 2px; }
</style>
