<script lang="ts">
  // The right bar: the settings of the chosen look (Nebula's Shape controls plus colors, gradient, motion,
  // grade and effects). Every change goes straight to the live canvas through `editor`.
  import { logIgnored } from "../../services/ignored-error";
  import { paletteFromImage, randomPalette, rgbToHex } from "../../wallpaper/color";
  import { SPECS, clampTo, type GrainType, type ToneMode } from "../../wallpaper/params";
  import type { Blend } from "../../wallpaper/color";
  import { lutToCss } from "../../wallpaper/gradient";
  import { editor } from "./editorState.svelte";
  import { L } from "./strings";
  import { lookById } from "../../wallpaper/looks";
  import PanelSection from "./controls/PanelSection.svelte";
  import ParamSlider from "./controls/ParamSlider.svelte";
  import Segmented from "./controls/Segmented.svelte";
  import ToggleRow from "./controls/ToggleRow.svelte";
  import ColorSwatch from "./controls/ColorSwatch.svelte";
  import GradientStops from "./controls/GradientStops.svelte";
  import LookControl from "./controls/LookControl.svelte";

  const p = $derived(editor.params);
  const look = $derived(lookById(editor.look));
  const gradient = $derived(lutToCss(editor.lut));
  const sign = (value: number, unit = "") => `${value >= 0 ? "+" : "−"}${Math.abs(value).toFixed(2)}${unit}`;

  const blends: { id: Blend; label: string }[] = [{ id: "srgb", label: "sRGB" }, { id: "linear", label: "Linear" }, { id: "oklab", label: "OKLab" }, { id: "oklch", label: "OKLCH" }];
  const grains: { id: GrainType; label: string }[] = [{ id: "classic", label: L.classic }, { id: "film", label: L.film }];
  const tones: { id: ToneMode; label: string }[] = [{ id: "none", label: L.none }, { id: "aces", label: L.aces }];

  function randomize() { editor.params.colors = randomPalette().map(rgbToHex); }

  let picker = $state<HTMLInputElement>();
  /** The eight main colors of a picture chosen from disk. The picture never leaves this page. */
  async function fromImage(event: Event) {
    const input = event.currentTarget as HTMLInputElement, file = input.files?.[0];
    input.value = "";
    if (!file) return;
    try {
      const bitmap = await createImageBitmap(file);
      const canvas = document.createElement("canvas");
      canvas.width = canvas.height = 64;
      const context = canvas.getContext("2d", { willReadFrequently: true });
      if (!context) throw new Error("Background editor: no 2D canvas to read the picture");
      context.drawImage(bitmap, 0, 0, 64, 64);
      bitmap.close();
      const palette = paletteFromImage(context.getImageData(0, 0, 64, 64).data, 8);
      if (!palette.length) return;
      editor.params.colors = Array.from({ length: 8 }, (_, i) => rgbToHex(palette[Math.min(i, palette.length - 1)]));
    } catch (error) { logIgnored(error); }
  }
</script>

<div class="settings">
  <PanelSection title={L.colors}>
    <div class="swatches">
      {#each p.colors as hex, index (index)}
        <ColorSwatch label={L.swatch} value={hex} dimmed={index >= p.activeColors} onchange={(value) => (editor.params.colors[index] = value)} />
      {/each}
    </div>
    <div class="pair">
      <button type="button" class="action" onclick={randomize}>{L.randomize}</button>
      <button type="button" class="action" onclick={() => picker?.click()}>{L.fromImage}</button>
      <input bind:this={picker} type="file" accept="image/*" hidden onchange={fromImage} />
    </div>
    <ParamSlider label={L.activeColors} value={p.activeColors} spec={SPECS.activeColors} onchange={(value) => (editor.params.activeColors = clampTo(SPECS.activeColors, value))} />
  </PanelSection>

  <PanelSection title={L.gradientMap} resetLabel={L.reset} onreset={() => editor.resetGradient()}>
    <h3>{L.blend}</h3>
    <Segmented label={L.blend} options={blends} value={p.blend} onchange={(value) => (editor.params.blend = value)} />
    <div class="caption"><h3>{L.stops}</h3><span>{L.dragToReposition}</span></div>
    <GradientStops
      stops={p.stops} colors={p.colors} active={p.activeColors} {gradient} label={L.stops} stopLabel={L.stop}
      onmove={(index, position) => (editor.params.stops[index] = position)}
    />
    <ParamSlider label={L.bias} value={p.bias} spec={SPECS.bias} onchange={(value) => (editor.params.bias = value)} />
    <ParamSlider label={L.sharpness} value={p.sharpness} spec={SPECS.sharpness} onchange={(value) => (editor.params.sharpness = value)} />
  </PanelSection>

  <PanelSection title={L.shape}>
    {#each look.controls.filter((item) => item.group === "shape") as item (`${look.id}-${item.key}`)}
      <LookControl {item} value={p.shapes[look.id]?.[item.key] ?? item.default} onchange={(value) => (editor.params.shapes[look.id][item.key] = value)} />
    {/each}
  </PanelSection>

  <PanelSection title={L.motion}>
    {#each look.controls.filter((item) => item.group === "motion") as item (`${look.id}-${item.key}`)}
      <LookControl {item} value={p.shapes[look.id]?.[item.key] ?? item.default} onchange={(value) => (editor.params.shapes[look.id][item.key] = value)} />
    {/each}
  </PanelSection>

  <PanelSection title={L.grade}>
    <ParamSlider label={L.brightness} value={p.grade.brightness} spec={SPECS.brightness} onchange={(value) => (editor.params.grade.brightness = value)} />
    <ParamSlider label={L.contrast} value={p.grade.contrast} spec={SPECS.gradeContrast} onchange={(value) => (editor.params.grade.contrast = value)} />
    <ParamSlider label={L.saturation} value={p.grade.saturation} spec={SPECS.saturation} onchange={(value) => (editor.params.grade.saturation = value)} />
  </PanelSection>

  <PanelSection title={L.effects} resetLabel={L.reset} onreset={() => editor.resetEffects()}>
    <h3>{L.grain}</h3>
    <Segmented label={L.grain} options={grains} value={p.effects.grain.type} onchange={(value) => (editor.params.effects.grain.type = value)} />
    <ParamSlider label={L.amount} value={p.effects.grain.amount} spec={SPECS.grainAmount} onchange={(value) => (editor.params.effects.grain.amount = value)} />
    <ParamSlider label={L.size} value={p.effects.grain.size} spec={SPECS.grainSize} format={(value) => `${value.toFixed(2)} px`} onchange={(value) => (editor.params.effects.grain.size = value)} />
    <ParamSlider label={L.color} value={p.effects.grain.color} spec={SPECS.grainColor} onchange={(value) => (editor.params.effects.grain.color = value)} />
    <ParamSlider label={L.midtone} value={p.effects.grain.midtone} spec={SPECS.grainMidtone} onchange={(value) => (editor.params.effects.grain.midtone = value)} />
    <ParamSlider label={L.seed} value={p.effects.grain.seed} spec={SPECS.grainSeed} onchange={(value) => (editor.params.effects.grain.seed = value)} />
    <ToggleRow label={L.animateGrain} checked={p.effects.grain.animate} onchange={(value) => (editor.params.effects.grain.animate = value)} />

    <h3>{L.tone}</h3>
    <Segmented label={L.tone} options={tones} value={p.effects.tone.mode} onchange={(value) => (editor.params.effects.tone.mode = value)} />
    <ParamSlider label={L.exposure} value={p.effects.tone.exposure} spec={SPECS.exposure} format={(value) => sign(value, " EV")} onchange={(value) => (editor.params.effects.tone.exposure = value)} />

    <h3>{L.vignette}</h3>
    <ParamSlider label={L.amount} value={p.effects.vignette} spec={SPECS.vignette} onchange={(value) => (editor.params.effects.vignette = value)} />

    <h3>{L.ditherArt}</h3>
    <ToggleRow label={L.orderedDither} checked={p.effects.dither} onchange={(value) => (editor.params.effects.dither = value)} />
  </PanelSection>
</div>

<style>
  .settings { font-size: 12px; }
  h3 { margin: 0 0 8px; color: var(--muted); font-size: 10px; font-weight: 700; letter-spacing: 0.7px; text-transform: uppercase; }
  .caption { display: flex; align-items: baseline; justify-content: space-between; margin: 0 0 8px; color: var(--muted); font-size: 10px; }
  .caption h3 { margin: 0; }
  .swatches { display: grid; grid-template-columns: repeat(4, 1fr); gap: 8px; margin: 0 0 12px; }
  .pair { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; margin: 0 0 14px; }
  .action { height: 34px; border: 1px solid var(--item-border); border-radius: 10px; background: var(--item-bg); color: var(--text); font: inherit; font-size: 11px; font-weight: 700; cursor: pointer; transition: background 0.15s ease; }
  .action:hover { background: var(--item-hover); }
  .action:focus-visible { outline: 2px solid var(--focus); outline-offset: 2px; }
</style>
