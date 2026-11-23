<script lang="ts">
  // The page of the custom background editor. It has no access to the browser's commands and nothing here
  // leaves the page. Left bar: the looks. Right bar: the settings of the chosen look. Behind both: the canvas.
  import WallpaperPanel from "./WallpaperPanel.svelte";
  import WallpaperSettingsPanel from "./WallpaperSettingsPanel.svelte";
  import WallpaperCanvas from "./WallpaperCanvas.svelte";
  import WallpaperPresetsPanel from "./WallpaperPresetsPanel.svelte";
  import Segmented from "./controls/Segmented.svelte";
  import { LOOKS } from "../../wallpaper/looks";
  import { editor } from "./editorState.svelte";
  import { L } from "./strings";

  document.title = L.title;
  let tab = $state<"options" | "presets">("options");
  const tabs: { id: "options" | "presets"; label: string }[] = [{ id: "options", label: L.options }, { id: "presets", label: L.presets }];
</script>

<WallpaperCanvas />
<main>
  <WallpaperPanel label={L.leftPanel} width={280}>
    <Segmented label={L.leftPanel} options={tabs} value={tab} onchange={(value) => (tab = value)} />
    {#if tab === "options"}
      <div class="looks" role="radiogroup" aria-label={L.leftPanel}>
        {#each LOOKS as look (look.id)}
          <button type="button" role="radio" aria-checked={editor.look === look.id} class="look" class:selected={editor.look === look.id} onclick={() => (editor.look = look.id)}>{look.name}</button>
        {/each}
      </div>
    {:else}
      <WallpaperPresetsPanel />
    {/if}
  </WallpaperPanel>
  <WallpaperPanel label={L.rightPanel} width={310}>
    <WallpaperSettingsPanel />
  </WallpaperPanel>
</main>

<style>
  :global(html), :global(body) { margin: 0; height: 100%; overflow: hidden; }
  :global(body) { background: #0b0b10; color: #f5f5f5; font: 14px/1.5 system-ui, sans-serif; }
  main { position: relative; z-index: 1; display: flex; justify-content: space-between; height: 100vh; box-sizing: border-box; padding: 16px; pointer-events: none; }
  .looks { display: grid; grid-template-columns: repeat(3, 1fr); gap: 6px; }
  .look {
    display: flex; align-items: center; justify-content: center;
    min-width: 0; height: 36px; padding: 0 10px; box-sizing: border-box;
    border: 1px solid var(--item-border); border-radius: 12px; background: var(--item-bg); color: var(--muted);
    font: 600 12px/1 system-ui, sans-serif; cursor: pointer; transition: background 0.15s ease, color 0.15s ease;
  }
  .look:hover { background: var(--item-hover); color: var(--text); }
  .look.selected { background: var(--selected-bg); border-color: var(--selected-bg); color: var(--selected-text); }
  .look:focus-visible { outline: 2px solid var(--focus); outline-offset: 2px; }
  @media (prefers-color-scheme: light) {
    :global(body) { background: #eceef2; color: #1a1a1a; }
  }
</style>
