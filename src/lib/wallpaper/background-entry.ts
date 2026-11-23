/** Entry of the bundle the new tab page loads (`public/new-tab/nebula-background.js`): the same engine and
 *  looks as the editor, behind a small global API, because the new tab page is plain scripts with no build step.
 *  Rebuild with `npm run build:nebula`. */
import { createRenderer } from "./renderer";
import { lutFromParams, lutToCss } from "./gradient";
import { lookById } from "./looks";
import { sanitizeParams } from "./params";

export interface NebulaBackground { update(settings: unknown, look?: string): void; destroy(): void }

function start(canvas: HTMLCanvasElement, report: (error: unknown) => void): NebulaBackground | null {
  const renderer = createRenderer(canvas, report);
  if (!renderer) return null;
  return {
    update(settings, look) {
      const params = sanitizeParams(settings);
      renderer.update(params, lutFromParams(params), lookById(look).id);
    },
    destroy() { renderer.destroy(); },
  };
}

const api = {
  start,
  sanitize: sanitizeParams,
  /** A small CSS gradient that shows the colors of some settings. */
  gradientCss: (settings: unknown) => lutToCss(lutFromParams(sanitizeParams(settings)), 8),
};
(window as unknown as { __sparkNebula: typeof api }).__sparkNebula = api;
