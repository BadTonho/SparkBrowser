<script lang="ts">
  let {
    label,
    hint = "",
    checked,
    disabled = false,
    testid,
    onchange,
  }: {
    label: string;
    hint?: string;
    checked: boolean;
    disabled?: boolean;
    testid?: string;
    onchange: (checked: boolean) => void;
  } = $props();
</script>

<label class="row" class:disabled>
  <span class="text">
    <strong>{label}</strong>
    {#if hint}<small>{hint}</small>{/if}
  </span>
  <span class="switch">
    <input type="checkbox" role="switch" data-testid={testid} {checked} {disabled} onchange={(event) => onchange(event.currentTarget.checked)} />
    <span class="track" aria-hidden="true"></span>
  </span>
</label>

<style>
  .row { display: flex; align-items: center; justify-content: space-between; gap: 14px; cursor: pointer; }
  .row.disabled { cursor: default; opacity: 0.6; }
  .text { display: flex; min-width: 0; flex-direction: column; gap: 2px; }
  strong { color: var(--text-primary); font-size: 12.5px; font-weight: 600; line-height: 1.4; }
  small { color: var(--text-secondary); font-size: 11.5px; line-height: 1.45; }
  .switch { position: relative; flex: 0 0 auto; width: 40px; height: 22px; }
  .switch input { position: absolute; inset: 0; z-index: 1; width: 100%; height: 100%; margin: 0; opacity: 0; cursor: inherit; }
  .track { position: absolute; inset: 0; border-radius: 99px; background: color-mix(in srgb, var(--text-primary) 24%, transparent); transition: background 0.16s ease; }
  .track::before { content: ""; position: absolute; top: 3px; left: 3px; width: 16px; height: 16px; border-radius: 50%; background: #fff; transition: transform 0.16s ease; }
  input:checked + .track { background: var(--accent-primary); }
  input:checked + .track::before { transform: translateX(18px); background: var(--accent-foreground); }
  input:focus-visible + .track { outline: 2px solid var(--accent-primary); outline-offset: 3px; }
  @media (prefers-reduced-motion: reduce) { .track, .track::before { transition: none; } }
</style>
