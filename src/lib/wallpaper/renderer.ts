import { NEBULA_FRAGMENT, VERTEX_SHADER } from "./nebula";
import type { EditorParams } from "./params";

export interface Renderer {
  /** Sends new settings and the gradient strip (256 x 1 RGBA bytes) to the next frame. */
  update(params: EditorParams, lut: Uint8Array): void;
  destroy(): void;
}

const MAX_PIXEL_RATIO = 1.5;

/** Draws the Nebula into `canvas`. Returns null (after reporting why) when WebGL cannot be used. Failures go
 *  to `report`, so the editor and the new tab page can each send them to their own error log. */
export function createRenderer(canvas: HTMLCanvasElement, report: (error: unknown) => void): Renderer | null {
  const gl = (canvas.getContext("webgl2", { antialias: false, alpha: false }) ?? canvas.getContext("webgl", { antialias: false, alpha: false })) as WebGLRenderingContext | null;
  if (!gl) { report(new Error("Background editor: WebGL is not available")); return null; }

  let program: WebGLProgram | null = null;
  let texture: WebGLTexture | null = null;
  let buffer: WebGLBuffer | null = null;
  let uniforms: Record<string, WebGLUniformLocation | null> = {};
  let params: EditorParams | null = null;
  let lut: Uint8Array | null = null;
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

  function setup(): boolean {
    try {
      const vertex = compile(gl!.VERTEX_SHADER, VERTEX_SHADER), fragment = compile(gl!.FRAGMENT_SHADER, NEBULA_FRAGMENT);
      program = gl!.createProgram();
      if (!program) throw new Error("Background editor: cannot create a program");
      gl!.attachShader(program, vertex);
      gl!.attachShader(program, fragment);
      gl!.linkProgram(program);
      gl!.deleteShader(vertex);
      gl!.deleteShader(fragment);
      if (!gl!.getProgramParameter(program, gl!.LINK_STATUS)) throw new Error(`Background editor: program failed to link: ${gl!.getProgramInfoLog(program)}`);
      gl!.useProgram(program);
      buffer = gl!.createBuffer();
      gl!.bindBuffer(gl!.ARRAY_BUFFER, buffer);
      // One triangle that covers the whole canvas.
      gl!.bufferData(gl!.ARRAY_BUFFER, new Float32Array([-1, -1, 3, -1, -1, 3]), gl!.STATIC_DRAW);
      const position = gl!.getAttribLocation(program, "a_position");
      gl!.enableVertexAttribArray(position);
      gl!.vertexAttribPointer(position, 2, gl!.FLOAT, false, 0, 0);
      texture = gl!.createTexture();
      gl!.bindTexture(gl!.TEXTURE_2D, texture);
      gl!.texParameteri(gl!.TEXTURE_2D, gl!.TEXTURE_MIN_FILTER, gl!.LINEAR);
      gl!.texParameteri(gl!.TEXTURE_2D, gl!.TEXTURE_MAG_FILTER, gl!.LINEAR);
      gl!.texParameteri(gl!.TEXTURE_2D, gl!.TEXTURE_WRAP_S, gl!.CLAMP_TO_EDGE);
      gl!.texParameteri(gl!.TEXTURE_2D, gl!.TEXTURE_WRAP_T, gl!.CLAMP_TO_EDGE);
      uniforms = {};
      for (const name of ["u_res", "u_phase", "u_clock", "u_lut", "u_scale", "u_billow", "u_contrast", "u_stars", "u_bias", "u_sharp", "u_bright", "u_gcontrast", "u_sat", "u_exposure", "u_aces", "u_vignette", "u_grainType", "u_grainAmount", "u_grainSize", "u_grainColor", "u_grainMid", "u_seed", "u_grainAnimate", "u_dither", "u_dpr"]) {
        uniforms[name] = gl!.getUniformLocation(program, name);
      }
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
    if (destroyed || lost || !program || !params) return;
    const seconds = now / 1000, delta = last ? Math.min(0.1, seconds - last) : 0;
    last = seconds;
    clock += delta;
    // The clouds advance by drift x time, so changing the speed never makes them jump.
    phase += delta * params.drift * 2;
    const ratio = resize();
    gl!.viewport(0, 0, canvas.width, canvas.height);
    if (lutDirty && lut) {
      gl!.activeTexture(gl!.TEXTURE0);
      gl!.bindTexture(gl!.TEXTURE_2D, texture);
      gl!.texImage2D(gl!.TEXTURE_2D, 0, gl!.RGBA, lut.length / 4, 1, 0, gl!.RGBA, gl!.UNSIGNED_BYTE, lut);
      lutDirty = false;
    }
    const set = (name: string, value: number) => gl!.uniform1f(uniforms[name], value);
    gl!.uniform2f(uniforms.u_res, canvas.width, canvas.height);
    gl!.uniform1i(uniforms.u_lut, 0);
    set("u_phase", phase); set("u_clock", clock); set("u_dpr", ratio);
    const p = params, grain = p.effects.grain;
    set("u_scale", p.shape.scale); set("u_billow", p.shape.billow); set("u_contrast", p.shape.contrast); set("u_stars", p.shape.stars);
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
    update(next, strip) {
      params = next;
      if (strip !== lut) { lut = strip; lutDirty = true; }
      schedule();
    },
    destroy() {
      destroyed = true;
      cancelAnimationFrame(frame);
      document.removeEventListener("visibilitychange", onVisibility);
      canvas.removeEventListener("webglcontextlost", onLost);
      canvas.removeEventListener("webglcontextrestored", onRestored);
      if (!lost) { gl.deleteTexture(texture); gl.deleteBuffer(buffer); gl.deleteProgram(program); }
    },
  };
}
