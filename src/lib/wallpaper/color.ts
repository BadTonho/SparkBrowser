/** Color math for the background editor: sRGB, linear light, OKLab / OKLCH, the gradient look-up table and
 *  palettes. Pure functions with no browser APIs, so they can be tested in node. Channels are 0..1. */
export type Rgb = [number, number, number];
export type Blend = "srgb" | "linear" | "oklab" | "oklch";
export interface Stop { position: number; color: Rgb }

const clamp01 = (value: number) => Math.min(1, Math.max(0, value));

export const srgbToLinear = (value: number) => value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;
export const linearToSrgb = (value: number) => {
  const v = clamp01(value);
  return v <= 0.0031308 ? v * 12.92 : 1.055 * v ** (1 / 2.4) - 0.055;
};

export function linearToOklab([r, g, b]: Rgb): Rgb {
  const l = Math.cbrt(0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b);
  const m = Math.cbrt(0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b);
  const s = Math.cbrt(0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b);
  return [
    0.2104542553 * l + 0.793617785 * m - 0.0040720468 * s,
    1.9779984951 * l - 2.428592205 * m + 0.4505937099 * s,
    0.0259040371 * l + 0.7827717662 * m - 0.808675766 * s,
  ];
}
export function oklabToLinear([L, a, b]: Rgb): Rgb {
  const l = (L + 0.3963377774 * a + 0.2158037573 * b) ** 3;
  const m = (L - 0.1055613458 * a - 0.0638541728 * b) ** 3;
  const s = (L - 0.0894841775 * a - 1.291485548 * b) ** 3;
  return [
    4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
    -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
    -0.0041960863 * l - 0.7034186147 * m + 1.707614701 * s,
  ];
}
export const oklabToOklch = ([L, a, b]: Rgb): Rgb => [L, Math.hypot(a, b), (Math.atan2(b, a) * 180 / Math.PI + 360) % 360];
export const oklchToOklab = ([L, C, h]: Rgb): Rgb => [L, C * Math.cos(h * Math.PI / 180), C * Math.sin(h * Math.PI / 180)];

export const rgbToOklch = (rgb: Rgb): Rgb => oklabToOklch(linearToOklab(rgb.map(srgbToLinear) as Rgb));
export const oklchToRgb = (lch: Rgb): Rgb => oklabToLinear(oklchToOklab(lch)).map(linearToSrgb) as Rgb;

export function hexToRgb(hex: string): Rgb {
  const match = /^#?([0-9a-f]{6})$/i.exec(hex.trim());
  if (!match) return [0, 0, 0];
  const value = parseInt(match[1], 16);
  return [(value >> 16 & 255) / 255, (value >> 8 & 255) / 255, (value & 255) / 255];
}
export function rgbToHex(rgb: Rgb): string {
  return "#" + rgb.map(channel => Math.round(clamp01(channel) * 255).toString(16).padStart(2, "0")).join("");
}

const mix = (a: number, b: number, t: number) => a + (b - a) * t;

/** Mixes two sRGB colors at `t` in the chosen space and returns an sRGB color. */
export function blendColors(from: Rgb, to: Rgb, t: number, blend: Blend): Rgb {
  if (blend === "srgb") return [mix(from[0], to[0], t), mix(from[1], to[1], t), mix(from[2], to[2], t)];
  const a = from.map(srgbToLinear) as Rgb, b = to.map(srgbToLinear) as Rgb;
  if (blend === "linear") return [0, 1, 2].map(i => linearToSrgb(mix(a[i], b[i], t))) as Rgb;
  const labA = linearToOklab(a), labB = linearToOklab(b);
  if (blend === "oklab") return oklabToLinear([0, 1, 2].map(i => mix(labA[i], labB[i], t)) as Rgb).map(linearToSrgb) as Rgb;
  const [La, Ca, ha] = oklabToOklch(labA), [Lb, Cb, hb] = oklabToOklch(labB);
  // A grey has no hue: borrow the other color's so the blend does not swing through the color wheel.
  const hueA = Ca < 1e-4 ? hb : ha, hueB = Cb < 1e-4 ? ha : hb;
  let delta = hueB - hueA;
  if (delta > 180) delta -= 360;
  if (delta < -180) delta += 360;
  return oklabToLinear(oklchToOklab([mix(La, Lb, t), mix(Ca, Cb, t), (hueA + delta * t + 360) % 360])).map(linearToSrgb) as Rgb;
}

/** The gradient as a `size` x 1 RGBA strip (bytes), from stops that may be unsorted. Outside the first and
 *  last stop the end colors are held. */
export function buildGradientLut(stops: Stop[], blend: Blend, size = 256): Uint8Array {
  const sorted = [...stops].sort((a, b) => a.position - b.position);
  const out = new Uint8Array(size * 4);
  if (!sorted.length) return out;
  for (let i = 0; i < size; i++) {
    const t = size === 1 ? 0 : i / (size - 1);
    let color: Rgb;
    if (t <= sorted[0].position) color = sorted[0].color;
    else if (t >= sorted[sorted.length - 1].position) color = sorted[sorted.length - 1].color;
    else {
      let k = 0;
      while (k < sorted.length - 2 && t > sorted[k + 1].position) k++;
      const span = sorted[k + 1].position - sorted[k].position;
      color = blendColors(sorted[k].color, sorted[k + 1].color, span <= 1e-6 ? 0 : (t - sorted[k].position) / span, blend);
    }
    for (let c = 0; c < 3; c++) out[i * 4 + c] = Math.round(clamp01(color[c]) * 255);
    out[i * 4 + 3] = 255;
  }
  return out;
}

/** A pleasant random palette: two dark base colors and warm-to-light accents around a random hue (OKLCH). */
export function randomPalette(random: () => number = Math.random, count = 8): Rgb[] {
  const hue = random() * 360, spread = 25 + random() * 70;
  const colors: Rgb[] = [
    oklchToRgb([0.12 + random() * 0.06, 0.02 + random() * 0.03, hue]),
    oklchToRgb([0.2 + random() * 0.08, 0.06 + random() * 0.05, (hue + spread * 0.5) % 360]),
  ];
  for (let i = 2; i < count; i++) {
    const t = (i - 2) / Math.max(1, count - 3);
    colors.push(oklchToRgb([0.55 + t * 0.35, 0.1 + random() * 0.1, (hue + spread * (0.8 + t * 1.4) + random() * 14) % 360]));
  }
  return colors;
}

/** Up to `count` representative colors of an image (RGBA bytes), by median cut, darkest first. */
export function paletteFromImage(pixels: ArrayLike<number>, count = 8): Rgb[] {
  const samples: Rgb[] = [];
  for (let i = 0; i + 3 < pixels.length; i += 4) {
    if (pixels[i + 3] < 128) continue;
    samples.push([pixels[i] / 255, pixels[i + 1] / 255, pixels[i + 2] / 255]);
  }
  if (!samples.length) return [];
  let boxes: Rgb[][] = [samples];
  while (boxes.length < count) {
    let target = -1, widest = 0, channel = 0;
    boxes.forEach((box, index) => {
      if (box.length < 2) return;
      for (let c = 0; c < 3; c++) {
        let lo = 1, hi = 0;
        for (const color of box) { lo = Math.min(lo, color[c]); hi = Math.max(hi, color[c]); }
        if (hi - lo > widest) { widest = hi - lo; target = index; channel = c; }
      }
    });
    if (target < 0 || widest < 1e-3) break;
    const box = boxes[target].sort((a, b) => a[channel] - b[channel]);
    const half = box.length >> 1;
    boxes.splice(target, 1, box.slice(0, half), box.slice(half));
  }
  const luma = (c: Rgb) => 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
  return boxes
    .map(box => [0, 1, 2].map(c => box.reduce((sum, color) => sum + color[c], 0) / box.length) as Rgb)
    .sort((a, b) => luma(a) - luma(b));
}
