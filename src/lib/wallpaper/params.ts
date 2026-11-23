/** What the background editor can change, with ranges and the defaults taken from the reference. No browser
 *  APIs here, so it can be tested in node. */
import type { Blend } from "./color";

export type GrainType = "classic" | "film";
export type ToneMode = "none" | "aces";

export interface EditorParams {
  /** Eight `#rrggbb` colors; only the first `activeColors` are used by the gradient. */
  colors: string[];
  activeColors: number;
  /** Position (0..1) of the stop of each color. */
  stops: number[];
  blend: Blend;
  bias: number;
  sharpness: number;
  /** Nebula's own controls. */
  shape: { scale: number; billow: number; contrast: number; stars: number };
  drift: number;
  grade: { brightness: number; contrast: number; saturation: number };
  effects: {
    grain: { type: GrainType; amount: number; size: number; color: number; midtone: number; seed: number; animate: boolean };
    tone: { mode: ToneMode; exposure: number };
    vignette: number;
    dither: boolean;
  };
}

export interface Spec { min: number; max: number; step: number; decimals: number }
/** Range, step and display precision of every number the editor shows. */
export const SPECS = {
  activeColors: { min: 2, max: 8, step: 1, decimals: 0 },
  bias: { min: -1, max: 1, step: 0.01, decimals: 2 },
  sharpness: { min: 0, max: 1, step: 0.01, decimals: 2 },
  scale: { min: 0.5, max: 8, step: 0.01, decimals: 2 },
  billow: { min: 0, max: 3, step: 0.01, decimals: 2 },
  shapeContrast: { min: 0.2, max: 3, step: 0.01, decimals: 2 },
  stars: { min: 0, max: 4, step: 0.01, decimals: 2 },
  drift: { min: 0, max: 2, step: 0.005, decimals: 3 },
  brightness: { min: 0, max: 2, step: 0.01, decimals: 2 },
  gradeContrast: { min: 0, max: 2, step: 0.01, decimals: 2 },
  saturation: { min: 0, max: 2, step: 0.01, decimals: 2 },
  grainAmount: { min: 0, max: 0.1, step: 0.001, decimals: 3 },
  grainSize: { min: 0.5, max: 4, step: 0.05, decimals: 2 },
  grainColor: { min: 0, max: 1, step: 0.01, decimals: 2 },
  grainMidtone: { min: 0, max: 1, step: 0.01, decimals: 2 },
  grainSeed: { min: 1, max: 999, step: 1, decimals: 0 },
  exposure: { min: -3, max: 3, step: 0.05, decimals: 2 },
  vignette: { min: 0, max: 1, step: 0.01, decimals: 2 },
} as const satisfies Record<string, Spec>;

export const clampTo = (spec: Spec, value: number) => {
  if (!Number.isFinite(value)) return spec.min;
  const stepped = Math.round((value - spec.min) / spec.step) * spec.step + spec.min;
  return Math.min(spec.max, Math.max(spec.min, Number(stepped.toFixed(6))));
};

export const evenStops = (count = 8) => Array.from({ length: count }, (_, i) => i / (count - 1));

export const defaultParams = (): EditorParams => ({
  colors: ["#05050c", "#120f2e", "#ff9a3d", "#ffc566", "#ff6b61", "#ffe6c2", "#2a1a0c", "#3a2412"],
  activeColors: 6,
  stops: evenStops(6).concat([0.9, 1]),
  blend: "oklab",
  bias: 0.22,
  sharpness: 0.18,
  shape: { scale: 3, billow: 1.42, contrast: 1.35, stars: 1.8 },
  drift: 0.29,
  grade: { brightness: 0.87, contrast: 1, saturation: 1 },
  effects: {
    grain: { type: "film", amount: 0.012, size: 1, color: 0.25, midtone: 0.45, seed: 1, animate: false },
    tone: { mode: "aces", exposure: 0.2 },
    vignette: 0,
    dither: false,
  },
});

/** The gradient section (blend, stops, bias, sharpness) back to its defaults. */
export function resetGradient(params: EditorParams): EditorParams {
  const fresh = defaultParams();
  return { ...params, blend: fresh.blend, stops: fresh.stops, bias: fresh.bias, sharpness: fresh.sharpness };
}
/** The effects section back to its defaults. */
export function resetEffects(params: EditorParams): EditorParams {
  return { ...params, effects: defaultParams().effects };
}

/** Where a stop may be dragged: between its neighbours among the active stops. */
export function stopRange(stops: number[], index: number, active: number, gap = 0.01): [number, number] {
  const lower = index > 0 ? stops[index - 1] + gap : 0;
  const upper = index < active - 1 ? stops[index + 1] - gap : 1;
  const round = (value: number) => Number(value.toFixed(6));
  return [round(Math.min(lower, upper)), round(Math.max(lower, upper))];
}

const isRecord = (value: unknown): value is Record<string, unknown> => typeof value === "object" && value !== null && !Array.isArray(value);
const number = (spec: Spec, value: unknown, fallback: number) => typeof value === "number" && Number.isFinite(value) ? clampTo(spec, value) : fallback;
const choice = <T extends string>(value: unknown, allowed: readonly T[], fallback: T): T => allowed.includes(value as T) ? (value as T) : fallback;

/** Settings read from storage or from a saved preset: every value is checked, clamped to its range and
 *  replaced by the default when it is missing or of the wrong kind, so a damaged entry cannot break the editor. */
export function sanitizeParams(raw: unknown): EditorParams {
  const base = defaultParams();
  if (!isRecord(raw)) return base;
  const shape = isRecord(raw.shape) ? raw.shape : {}, grade = isRecord(raw.grade) ? raw.grade : {};
  const effects = isRecord(raw.effects) ? raw.effects : {};
  const grain = isRecord(effects.grain) ? effects.grain : {}, tone = isRecord(effects.tone) ? effects.tone : {};
  const list = (value: unknown) => (Array.isArray(value) ? value : []);
  const colors = list(raw.colors), stops = list(raw.stops);
  const fallback = base.effects.grain;
  return {
    colors: base.colors.map((color, i) => typeof colors[i] === "string" && /^#[0-9a-f]{6}$/i.test(colors[i]) ? colors[i].toLowerCase() : color),
    activeColors: number(SPECS.activeColors, raw.activeColors, base.activeColors),
    stops: base.stops.map((stop, i) => typeof stops[i] === "number" && Number.isFinite(stops[i]) ? Math.min(1, Math.max(0, stops[i])) : stop),
    blend: choice(raw.blend, ["srgb", "linear", "oklab", "oklch"] as const, base.blend),
    bias: number(SPECS.bias, raw.bias, base.bias),
    sharpness: number(SPECS.sharpness, raw.sharpness, base.sharpness),
    shape: {
      scale: number(SPECS.scale, shape.scale, base.shape.scale),
      billow: number(SPECS.billow, shape.billow, base.shape.billow),
      contrast: number(SPECS.shapeContrast, shape.contrast, base.shape.contrast),
      stars: number(SPECS.stars, shape.stars, base.shape.stars),
    },
    drift: number(SPECS.drift, raw.drift, base.drift),
    grade: {
      brightness: number(SPECS.brightness, grade.brightness, base.grade.brightness),
      contrast: number(SPECS.gradeContrast, grade.contrast, base.grade.contrast),
      saturation: number(SPECS.saturation, grade.saturation, base.grade.saturation),
    },
    effects: {
      grain: {
        type: choice(grain.type, ["classic", "film"] as const, fallback.type),
        amount: number(SPECS.grainAmount, grain.amount, fallback.amount),
        size: number(SPECS.grainSize, grain.size, fallback.size),
        color: number(SPECS.grainColor, grain.color, fallback.color),
        midtone: number(SPECS.grainMidtone, grain.midtone, fallback.midtone),
        seed: number(SPECS.grainSeed, grain.seed, fallback.seed),
        animate: typeof grain.animate === "boolean" ? grain.animate : fallback.animate,
      },
      tone: { mode: choice(tone.mode, ["none", "aces"] as const, base.effects.tone.mode), exposure: number(SPECS.exposure, tone.exposure, base.effects.tone.exposure) },
      vignette: number(SPECS.vignette, effects.vignette, base.effects.vignette),
      dither: typeof effects.dither === "boolean" ? effects.dither : base.effects.dither,
    },
  };
}
