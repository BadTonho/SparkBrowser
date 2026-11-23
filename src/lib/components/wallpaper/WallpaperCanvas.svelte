<script lang="ts">
  // The live Nebula, behind the two bars. All drawing is local (WebGL); nothing is sent anywhere.
  import { onMount } from "svelte";
  import { createRenderer, type Renderer } from "../../wallpaper/renderer";
  import { logIgnored } from "../../services/ignored-error";
  import { editor } from "./editorState.svelte";
  import { L } from "./strings";

  let canvas: HTMLCanvasElement;
  let renderer = $state.raw<Renderer | null>(null);
  let failed = $state(false);

  onMount(() => {
    renderer = createRenderer(canvas, logIgnored);
    failed = !renderer;
    return () => renderer?.destroy();
  });
  $effect(() => { renderer?.update($state.snapshot(editor.params), editor.lut); });
</script>

<canvas bind:this={canvas} aria-hidden="true"></canvas>
{#if failed}<p role="alert">{L.noWebgl}</p>{/if}

<style>
  canvas { position: absolute; inset: 0; display: block; width: 100%; height: 100%; background: #0b0b10; }
  p { position: absolute; inset: 0; display: grid; place-items: center; margin: 0; padding: 24px; color: #a3a3a3; text-align: center; }
</style>
