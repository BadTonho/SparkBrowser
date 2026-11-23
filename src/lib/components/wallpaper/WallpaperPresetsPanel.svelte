<script lang="ts">
  // The Presets tab of the left bar: save the settings shown now under a name, show a saved one again, delete it.
  import { lutFromParams, lutToCss } from "../../wallpaper/gradient";
  import { presets } from "./presets.svelte";
  import { L } from "./strings";

  let naming = $state(false);
  let draft = $state("");

  function startNaming() { draft = `Preset ${presets.items.length + 1}`; naming = true; }
  function focusInput(node: HTMLInputElement) { node.focus(); node.select(); }
  function commit() {
    if (!naming) return;
    naming = false;
    presets.save(draft);
  }
  function key(event: KeyboardEvent) {
    if (event.key === "Enter") { event.preventDefault(); commit(); }
    else if (event.key === "Escape") { event.preventDefault(); naming = false; }
  }
</script>

<div class="presets">
  {#if naming}
    <input class="name" maxlength="40" aria-label={L.presetName} placeholder={L.presetName} bind:value={draft} use:focusInput onkeydown={key} onblur={commit} />
  {:else}
    <button type="button" class="save" onclick={startNaming}>{L.savePreset}</button>
  {/if}

  {#if presets.items.length === 0}
    <p class="empty">{L.noPresets}</p>
  {:else}
    <ul>
      {#each presets.items as preset (preset.id)}
        <li>
          <button type="button" class="apply" title={preset.name} onclick={() => presets.apply(preset.id)}>
            <span class="swatch" aria-hidden="true" style:background={lutToCss(lutFromParams(preset.params), 8)}></span>
            <span class="label">{preset.name}</span>
          </button>
          <button type="button" class="remove" aria-label={`${L.deletePreset}: ${preset.name}`} title={L.deletePreset} onclick={() => presets.remove(preset.id)}>✕</button>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .presets { display: flex; flex-direction: column; gap: 10px; }
  .save, .name { height: 36px; box-sizing: border-box; border-radius: 12px; font: 600 12px/1 system-ui, sans-serif; }
  .save { border: 1px solid var(--item-border); background: var(--item-bg); color: var(--text); cursor: pointer; transition: background 0.15s ease; }
  .save:hover { background: var(--item-hover); }
  .name { padding: 0 12px; border: 1px solid var(--focus); background: var(--item-bg); color: var(--text); outline: none; }
  .save:focus-visible { outline: 2px solid var(--focus); outline-offset: 2px; }
  .empty { margin: 0; padding: 12px; border: 1px dashed var(--item-border); border-radius: 12px; color: var(--muted); font-size: 11px; line-height: 1.45; }
  ul { display: flex; flex-direction: column; gap: 4px; margin: 0; padding: 0; list-style: none; }
  li { display: flex; align-items: center; gap: 4px; }
  .apply { flex: 1; min-width: 0; display: flex; align-items: center; gap: 10px; padding: 5px 6px; border: 0; border-radius: 10px; background: transparent; color: var(--text); font: 600 12px/1.2 system-ui, sans-serif; text-align: left; cursor: pointer; transition: background 0.15s ease; }
  .apply:hover { background: var(--item-hover); }
  .swatch { flex: none; width: 44px; height: 28px; border-radius: 8px; box-shadow: inset 0 0 0 1px var(--item-border); }
  .label { min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .remove { flex: none; width: 26px; height: 26px; border: 0; border-radius: 8px; background: transparent; color: var(--muted); font-size: 11px; cursor: pointer; opacity: 0.6; transition: opacity 0.15s ease, background 0.15s ease; }
  li:hover .remove, .remove:focus-visible { opacity: 1; }
  .remove:hover { background: var(--item-hover); color: var(--text); }
  .apply:focus-visible, .remove:focus-visible { outline: 2px solid var(--focus); outline-offset: 2px; }
</style>
