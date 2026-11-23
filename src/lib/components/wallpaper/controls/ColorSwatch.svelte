<script lang="ts">
  let { label, value, dimmed = false, onchange }: { label: string; value: string; dimmed?: boolean; onchange: (value: string) => void } = $props();
</script>

<label class="swatch" class:dimmed style:background={value} title={`${label} ${value}`}>
  <input type="color" aria-label={`${label} ${value}`} {value} oninput={(event) => onchange(event.currentTarget.value)} />
</label>

<style>
  .swatch { position: relative; display: block; aspect-ratio: 1; border: 1px solid var(--item-border); border-radius: 12px; cursor: pointer; transition: transform 0.12s ease, opacity 0.15s ease; overflow: hidden; }
  .swatch:hover { transform: scale(1.04); }
  .swatch.dimmed { opacity: 0.4; }
  .swatch:focus-within { outline: 2px solid var(--focus); outline-offset: 2px; }
  input { position: absolute; inset: 0; width: 100%; height: 100%; opacity: 0; cursor: pointer; }
</style>
