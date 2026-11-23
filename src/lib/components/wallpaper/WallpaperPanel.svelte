<script lang="ts">
  // A floating side bar of the editor. Its content is passed in. It also defines the colors the controls
  // inside use, for the dark and the light theme.
  import type { Snippet } from "svelte";
  let { label, width, children }: { label: string; width: number; children?: Snippet } = $props();
</script>

<aside class="panel" aria-label={label} style:width={`${width}px`}>
  {@render children?.()}
</aside>

<style>
  .panel {
    --panel-bg: rgba(24, 24, 27, 0.8);
    --panel-border: rgba(255, 255, 255, 0.09);
    --item-bg: rgba(255, 255, 255, 0.045);
    --item-border: rgba(255, 255, 255, 0.09);
    --item-hover: rgba(255, 255, 255, 0.1);
    --text: #f2f2f4;
    --muted: #8d8d96;
    --track: rgba(255, 255, 255, 0.16);
    --selected-bg: #ffffff;
    --selected-text: #111114;
    --focus: #7aa7ff;
    flex: none;
    box-sizing: border-box;
    height: 100%;
    padding: 14px;
    overflow-y: auto;
    border: 1px solid var(--panel-border);
    border-radius: 18px;
    background: var(--panel-bg);
    backdrop-filter: blur(20px);
    color: var(--text);
    font-size: 12px;
    pointer-events: auto;
    scrollbar-width: thin;
    scrollbar-color: var(--item-border) transparent;
  }
  @media (prefers-color-scheme: light) {
    .panel {
      --panel-bg: rgba(255, 255, 255, 0.82);
      --panel-border: rgba(0, 0, 0, 0.1);
      --item-bg: rgba(0, 0, 0, 0.04);
      --item-border: rgba(0, 0, 0, 0.1);
      --item-hover: rgba(0, 0, 0, 0.08);
      --text: #1a1a1d;
      --muted: #6e6e78;
      --track: rgba(0, 0, 0, 0.16);
      --selected-bg: #17171a;
      --selected-text: #ffffff;
    }
  }
</style>
