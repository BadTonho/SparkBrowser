/** The looks of the background editor and the controls of each one. Pure data (the drawing code is in
 *  `shaders.ts`), so it can be tested in node and used by the editor and by the new tab page.
 *
 *  Every control has a number as its value (a choice stores the number of the option, a toggle 0 or 1), so
 *  presets keep a single format. The `speed` and `direction` controls of a look are used by the renderer to
 *  advance the look's time; all the other controls reach its shader, in order, as `u_p0`, `u_p1`, ... */
import type { Spec } from "./params";

export type ControlGroup = "shape" | "motion";
interface Text { en: string; pt: string }
interface Base { key: string; label: Text; group: ControlGroup; default: number }
export interface SliderControl extends Base { type: "slider"; spec: Spec }
export interface ChoiceControl extends Base { type: "choice"; options: { value: number; label: Text }[] }
export interface ToggleControl extends Base { type: "toggle" }
export type Control = SliderControl | ChoiceControl | ToggleControl;
export interface LookDef {
  id: string;
  name: string;
  /** Key of the slider that sets how fast the look moves. */
  speed: string;
  /** Key of the choice (1 or −1) that sets which way it moves. */
  direction?: string;
  controls: Control[];
}

export const MAX_SHADER_CONTROLS = 8;
export const DEFAULT_LOOK = "nebula";

const slider = (group: ControlGroup, key: string, en: string, pt: string, min: number, max: number, step: number, decimals: number, value: number): SliderControl =>
  ({ type: "slider", key, label: { en, pt }, group, spec: { min, max, step, decimals }, default: value });
const direction = (forward: Text, backward: Text): ChoiceControl => ({
  type: "choice", key: "direction", group: "motion", label: { en: "Direction", pt: "Direção" }, default: 1,
  options: [{ value: 1, label: forward }, { value: -1, label: backward }],
});

export const LOOKS: LookDef[] = [
  {
    id: "nebula", name: "Nebula", speed: "drift", direction: "direction",
    controls: [
      slider("shape", "scale", "Scale", "Escala", 0.5, 8, 0.01, 2, 3),
      slider("shape", "billow", "Billow", "Ondulação", 0, 3, 0.01, 2, 1.42),
      slider("shape", "contrast", "Contrast", "Contraste", 0.2, 3, 0.01, 2, 1.35),
      slider("shape", "stars", "Stars", "Estrelas", 0, 4, 0.01, 2, 1.8),
      slider("motion", "drift", "Speed", "Velocidade", 0, 2, 0.005, 3, 0.29),
      direction({ en: "Forward", pt: "Para frente" }, { en: "Backward", pt: "Para trás" }),
      slider("motion", "angle", "Angle", "Ângulo", 0, 360, 1, 0, 0),
    ],
  },
  {
    id: "space", name: "Space", speed: "speed", direction: "direction",
    controls: [
      slider("shape", "density", "Density", "Densidade", 0.2, 3, 0.01, 2, 1),
      slider("shape", "size", "Size", "Tamanho", 0.3, 3, 0.01, 2, 1),
      slider("shape", "twinkle", "Twinkle", "Cintilação", 0, 1, 0.01, 2, 0.5),
      slider("shape", "depth", "Depth", "Profundidade", 0, 2, 0.01, 2, 1),
      slider("shape", "glow", "Glow", "Brilho de fundo", 0, 2, 0.01, 2, 0.8),
      slider("motion", "speed", "Speed", "Velocidade", 0, 2, 0.005, 3, 0.29),
      direction({ en: "Toward you", pt: "Na sua direção" }, { en: "Away from you", pt: "Para longe" }),
      slider("motion", "sway", "Sway", "Balanço", 0, 2, 0.01, 2, 0),
    ],
  },
  {
    id: "galaxy", name: "Galaxy", speed: "speed", direction: "direction",
    controls: [
      slider("shape", "arms", "Arms", "Braços", 1, 6, 1, 0, 2),
      slider("shape", "twist", "Twist", "Torção", 0, 8, 0.01, 2, 3.5),
      slider("shape", "core", "Core", "Núcleo", 0, 2, 0.01, 2, 1),
      slider("shape", "dust", "Dust", "Poeira", 0, 2, 0.01, 2, 1),
      slider("shape", "stars", "Stars", "Estrelas", 0, 3, 0.01, 2, 1),
      slider("shape", "tilt", "Tilt", "Inclinação", 0, 1, 0.01, 2, 0),
      slider("motion", "speed", "Rotation speed", "Velocidade de rotação", 0, 2, 0.005, 3, 0.5),
      direction({ en: "Clockwise", pt: "Horário" }, { en: "Counterclockwise", pt: "Anti-horário" }),
      slider("motion", "breathing", "Breathing", "Respiração", 0, 1, 0.01, 2, 0),
    ],
  },
];

export const lookById = (id: unknown): LookDef => LOOKS.find(look => look.id === id) ?? LOOKS[0];
export const isLook = (id: unknown): id is string => LOOKS.some(look => look.id === id);
/** The controls the shader receives (all but speed and direction), in the order of `u_p0`, `u_p1`, ... */
export const shaderControls = (look: LookDef): Control[] => look.controls.filter(item => item.key !== look.speed && item.key !== look.direction);
