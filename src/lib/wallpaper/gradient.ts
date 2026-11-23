import { buildGradientLut, hexToRgb } from "./color";
import type { EditorParams } from "./params";

/** The gradient of some settings as a 256 x 1 RGBA strip. */
export function lutFromParams(params: Pick<EditorParams, "colors" | "stops" | "activeColors" | "blend">): Uint8Array {
  return buildGradientLut(
    Array.from({ length: params.activeColors }, (_, i) => ({ position: params.stops[i], color: hexToRgb(params.colors[i]) })),
    params.blend,
  );
}

/** A CSS `linear-gradient` that shows a gradient strip. */
export function lutToCss(strip: Uint8Array, steps = 16): string {
  const points: string[] = [];
  for (let i = 0; i <= steps; i++) {
    const at = Math.round((i / steps) * 255) * 4;
    points.push(`rgb(${strip[at]}, ${strip[at + 1]}, ${strip[at + 2]}) ${(i / steps) * 100}%`);
  }
  return `linear-gradient(90deg, ${points.join(", ")})`;
}
