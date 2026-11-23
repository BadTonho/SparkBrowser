import { fragmentShader, VERTEX_SHADER } from "./shaders";
import { lookById, MAX_SHADER_CONTROLS, shaderControls } from "./looks";
import type { EditorParams } from "./params";

export interface Renderer {
  /** Sends new settings, the gradient strip (256 x 1 RGBA bytes) and the look to draw to the next frame. */
  update(params: EditorParams, lut: Uint8Array, look: string): void;
  destroy(): void;
}

const MAX_PIXEL_RATIO = 1.5;
const UNIFORMS = [
  "u_res", "u_phase", "u_clock", "u_lut", "u_bias", "u_sharp", "u_bright", "u_gcontrast", "u_sat", "u_exposure", "u_aces",
  "u_vignette", "u_grainType", "u_grainAmount", "u_grainSize", "u_grainColor", "u_grainMid", "u_seed", "u_grainAnimate", "u_dither", "u_dpr",
  ...Array.from({ length: MAX_SHADER_CONTROLS }, (_, i) => `u_p${i}`),
];

interface Compiled { program: WebGLProgram; uniforms: Record<string, WebGLUniformLocation | null> }

/** Draws the looks into `canvas`, one compiled program per look (built the first time it is shown). Returns
 *  null (after reporting why) when WebGL cannot be used. Failures go to `report`, so the editor and the new tab
 *  page can each send them to their own error log. */
export function createRenderer(canvas: HTMLCanvasElement, report: (error: unknown) => void): Renderer | null {
  const gl = (canvas.getContext("webgl2", { antialias: false, alpha: false }) ?? canvas.getContext("webgl", { antialias: false, alpha: false })) as WebGLRenderingContext | null;
  if (!gl) { report(new Error("Background editor: WebGL is not available")); return null; }

  let programs = new Map<string, Compiled | null>();
  let vertex: WebGLShader | null = null;
  let texture: WebGLTexture | null = null;
  let buffer: WebGLBuffer | null = null;
  let params: EditorParams | null = null;
  let lut: Uint8Array | null = null;
  let look = "nebula";
  let lutDirty = true;
  let frame = 0, last = 0, phase = 0, clock = 0, destroyed = false, lost = false;

  function compile(type: number, source: string): WebGLShader {
    const shader = gl!.createShader(type);
    if (!shader) throw new Error("Background editor: cannot create a shader");
    gl!.shaderSource(shader, source);
    gl!.compileShader(shader);
    if (!gl!.getShaderParameter(shader, gl!.COMPILE_STATUS)) {
      const info = gl!.getShaderInfoLog(shader);
      gl!.deleteShader(shader);
      throw new Error(`Background editor: shader failed to compile: ${info}`);
    }
    return shader;
  }

  /** The program of a look, compiled on first use. A look that fails to compile is reported once and skipped. */
  function programFor(id: string): Compiled | null {
    if (programs.has(id)) return programs.get(id)!;
    let compiled: Compiled | null = null;
    try {
      vertex ??= compile(gl!.VERTEX_SHADER, VERTEX_SHADER);
      const fragment = compile(gl!.FRAGMENT_SHADER, fragmentShader(id));
      const program = gl!.createProgram();
      if (!program) throw new Error("Background editor: cannot create a program");
      gl!.attachShader(program, vertex);
      gl!.attachShader(program, fragment);
      gl!.bindAttribLocation(program, 0, "a_position");
      gl!.linkProgram(program);
      gl!.deleteShader(fragment);
      if (!gl!.getProgramParameter(program, gl!.LINK_STATUS)) throw new Error(`Background editor: program failed to link: ${gl!.getProgramInfoLog(program)}`);
      compiled = { program, uniforms: Object.fromEntries(UNIFORMS.map(name => [name, gl!.getUniformLocation(program, name)])) };
    } catch (error) { report(error); }
    programs.set(id, compiled);
    return compiled;
  }

  function setup(): boolean {
    try {
      programs = new Map();
      vertex = null;
      buffer = gl!.createBuffer();
      gl!.bindBuffer(gl!.ARRAY_BUFFER, buffer);
      // One triangle that covers the whole canvas; the position attribute is bound to slot 0 in every program.
      gl!.bufferData(gl!.ARRAY_BUFFER, new Float32Array([-1, -1, 3, -1, -1, 3]), gl!.STATIC_DRAW);
      gl!.enableVertexAttribArray(0);
      gl!.vertexAttribPointer(0, 2, gl!.FLOAT, false, 0, 0);
      texture = gl!.createTexture();
      gl!.bindTexture(gl!.TEXTURE_2D, texture);
      gl!.texParameteri(gl!.TEXTURE_2D, gl!.TEXTURE_MIN_FILTER, gl!.LINEAR);
      gl!.texParameteri(gl!.TEXTURE_2D, gl!.TEXTURE_MAG_FILTER, gl!.LINEAR);
      gl!.texParameteri(gl!.TEXTURE_2D, gl!.TEXTURE_WRAP_S, gl!.CLAMP_TO_EDGE);
      gl!.texParameteri(gl!.TEXTURE_2D, gl!.TEXTURE_WRAP_T, gl!.CLAMP_TO_EDGE);
      lutDirty = true;
      return true;
    } catch (error) {
      report(error);
      return false;
    }
  }

  function resize(): number {
    const ratio = Math.min(window.devicePixelRatio || 1, MAX_PIXEL_RATIO);
    const width = Math.max(1, Math.round(canvas.clientWidth * ratio)), height = Math.max(1, Math.round(canvas.clientHeight * ratio));
    if (canvas.width !== width || canvas.height !== height) { canvas.width = width; canvas.height = height; }
    return ratio;
  }

  function draw(now: number) {
    frame = 0;
    if (destroyed || lost || !params) return;
    const compiled = programFor(look);
    if (!compiled) return;
    const seconds = now / 1000, delta = last ? Math.min(0.1, seconds - last) : 0;
    last = seconds;
    clock += delta;
    // The look advances by speed x direction x time, so changing either never makes it jump.
    const definition = lookById(look), values = params.shapes[definition.id] ?? {};
    const speed = values[definition.speed] ?? 0, sign = definition.direction ? (values[definition.direction] ?? 1) : 1;
    phase += delta * speed * sign * 2;
    const ratio = resize();
    gl!.viewport(0, 0, canvas.width, canvas.height);
    gl!.useProgram(compiled.program);
    if (lutDirty && lut) {
      gl!.activeTexture(gl!.TEXTURE0);
      gl!.bindTexture(gl!.TEXTURE_2D, texture);
      gl!.texImage2D(gl!.TEXTURE_2D, 0, gl!.RGBA, lut.length / 4, 1, 0, gl!.RGBA, gl!.UNSIGNED_BYTE, lut);
      lutDirty = false;
    }
    const { uniforms } = compiled;
    const set = (name: string, value: number) => gl!.uniform1f(uniforms[name], value);
    gl!.uniform2f(uniforms.u_res, canvas.width, canvas.height);
    gl!.uniform1i(uniforms.u_lut, 0);
    set("u_phase", phase); set("u_clock", clock); set("u_dpr", ratio);
    const p = params, grain = p.effects.grain;
    shaderControls(definition).forEach((item, index) => set(`u_p${index}`, values[item.key] ?? item.default));
    set("u_bias", p.bias); set("u_sharp", p.sharpness);
    set("u_bright", p.grade.brightness); set("u_gcontrast", p.grade.contrast); set("u_sat", p.grade.saturation);
    set("u_exposure", p.effects.tone.exposure); set("u_aces", p.effects.tone.mode === "aces" ? 1 : 0); set("u_vignette", p.effects.vignette);
    set("u_grainType", grain.type === "film" ? 1 : 0); set("u_grainAmount", grain.amount); set("u_grainSize", grain.size);
    set("u_grainColor", grain.color); set("u_grainMid", grain.midtone); set("u_seed", grain.seed); set("u_grainAnimate", grain.animate ? 1 : 0);
    set("u_dither", p.effects.dither ? 1 : 0);
    gl!.drawArrays(gl!.TRIANGLES, 0, 3);
    schedule();
  }

  function schedule() {
    if (!frame && !destroyed && !lost && !document.hidden) frame = requestAnimationFrame(draw);
  }
  // A hidden tab draws nothing, and starts again from where it was when it is shown.
  const onVisibility = () => { last = 0; schedule(); };
  const onLost = (event: Event) => { event.preventDefault(); lost = true; cancelAnimationFrame(frame); frame = 0; report(new Error("Background editor: the WebGL context was lost")); };
  const onRestored = () => { lost = false; if (setup()) { last = 0; schedule(); } };
  document.addEventListener("visibilitychange", onVisibility);
  canvas.addEventListener("webglcontextlost", onLost);
  canvas.addEventListener("webglcontextrestored", onRestored);

  if (!setup()) {
    document.removeEventListener("visibilitychange", onVisibility);
    canvas.removeEventListener("webglcontextlost", onLost);
    canvas.removeEventListener("webglcontextrestored", onRestored);
    return null;
  }
  return {
    update(next, strip, id) {
      params = next;
      look = lookById(id).id;
      if (strip !== lut) { lut = strip; lutDirty = true; }
      schedule();
    },
    destroy() {
      destroyed = true;
      cancelAnimationFrame(frame);
      document.removeEventListener("visibilitychange", onVisibility);
      canvas.removeEventListener("webglcontextlost", onLost);
      canvas.removeEventListener("webglcontextrestored", onRestored);
      if (!lost) {
        for (const compiled of programs.values()) if (compiled) gl.deleteProgram(compiled.program);
        gl.deleteTexture(texture);
        gl.deleteBuffer(buffer);
      }
    },
  };
}
