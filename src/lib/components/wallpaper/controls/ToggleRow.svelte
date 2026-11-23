<script lang="ts">
  let { label, checked, onchange }: { label: string; checked: boolean; onchange: (checked: boolean) => void } = $props();
</script>

<label class="row">
  <span>{label}</span>
  <input type="checkbox" role="switch" {checked} onchange={(event) => onchange(event.currentTarget.checked)} />
</label>

<style>
  .row { display: flex; align-items: center; justify-content: space-between; gap: 12px; margin: 0 0 14px; font-size: 12px; cursor: pointer; }
  input { appearance: none; position: relative; flex: none; width: 38px; height: 22px; margin: 0; border-radius: 99px; background: var(--track); cursor: pointer; transition: background 0.15s ease; }
  input::before { content: ""; position: absolute; top: 3px; left: 3px; width: 16px; height: 16px; border-radius: 50%; background: var(--text); transition: transform 0.15s ease; }
  input:checked { background: var(--selected-bg); }
  input:checked::before { transform: translateX(16px); background: var(--selected-text); }
  input:focus-visible { outline: 2px solid var(--focus); outline-offset: 3px; }
</style>
