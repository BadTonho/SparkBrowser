/** GLSL ES 1.00 (WebGL 1 and 2) for the looks. Every look is `COMMON` + its own `lookValue` + `POST`:
 *  the look gives a 0..1 value, the shared tail maps it through the gradient strip, then grades, tones,
 *  vignettes, grains and optionally dithers it. A look's controls arrive in `u_p0..u_p5`, in the order
 *  they are listed in `looks.ts`. */
export const VERTEX_SHADER = `
attribute vec2 a_position;
void main() { gl_Position = vec4(a_position, 0.0, 1.0); }
`;

const COMMON = `
precision highp float;
uniform vec2 u_res;
uniform float u_phase;      // look time: grows with the look's speed and direction
uniform float u_clock;      // real seconds, for twinkling and animated grain
uniform sampler2D u_lut;
uniform float u_p0, u_p1, u_p2, u_p3, u_p4, u_p5, u_p6, u_p7;
uniform float u_bias, u_sharp;
uniform float u_bright, u_gcontrast, u_sat;
uniform float u_exposure, u_aces, u_vignette;
uniform float u_grainType, u_grainAmount, u_grainSize, u_grainColor, u_grainMid, u_seed, u_grainAnimate;
uniform float u_dither, u_dpr;

float hash21(vec2 p) {
  p = fract(p * vec2(123.34, 456.21));
  p += dot(p, p + 45.32);
  return fract(p.x * p.y);
}
float noise(vec2 p) {
  vec2 i = floor(p), f = fract(p);
  f = f * f * (3.0 - 2.0 * f);
  float a = hash21(i), b = hash21(i + vec2(1.0, 0.0)), c = hash21(i + vec2(0.0, 1.0)), d = hash21(i + vec2(1.0, 1.0));
  return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
}
// Fractal noise. \`billow\` goes from smooth clouds (0) to puffy, folded ones.
float fbm(vec2 p, float billow) {
  float sum = 0.0, amp = 0.5;
  mat2 rot = mat2(0.8, -0.6, 0.6, 0.8);
  for (int i = 0; i < 6; i++) {
    float n = noise(p);
    n = mix(n, abs(2.0 * n - 1.0), clamp(billow * 0.5, 0.0, 1.0));
    sum += amp * n;
    p = rot * p * 2.03 + 17.0;
    amp *= 0.5;
  }
  return sum;
}
// One layer of stars on a jittered grid. \`density\` is the share of cells that hold a star.
float starLayer(vec2 uv, float cells, float seed, float density, float size, float twinkle) {
  vec2 g = uv * cells, id = floor(g), f = fract(g) - 0.5;
  float h = hash21(id + seed);
  vec2 offset = (vec2(hash21(id + 3.1 + seed), hash21(id + 7.7 + seed)) - 0.5) * 0.7;
  float keep = step(1.0 - clamp(density, 0.0, 0.7), h);
  float radius = mix(0.04, 0.14, fract(h * 13.7)) * size;
  float d = length(f - offset);
  float wave = sin(u_clock * (1.5 + fract(h * 7.1) * 3.0) + h * 50.0);
  float flicker = 1.0 - twinkle * 0.5 * (1.0 + wave);
  float core = smoothstep(radius, 0.0, d);
  float halo = smoothstep(radius * 3.0, 0.0, d) * 0.25;
  return keep * (core + halo) * flicker * (0.5 + fract(h * 29.3));
}
mat2 rotation(float a) { float c = cos(a), s = sin(a); return mat2(c, -s, s, c); }
`;

/** Nebula: domain-warped clouds plus a star field. p0 scale, p1 billow, p2 contrast, p3 stars, p4 angle
 *  (the way the clouds drift, in degrees). */
const NEBULA = `
float nebulaStars(vec2 uv) {
  float total = 0.0;
  for (int layer = 0; layer < 2; layer++) {
    float cells = 55.0 + float(layer) * 70.0;
    vec2 g = uv * cells, id = floor(g), f = fract(g) - 0.5;
    float h = hash21(id + float(layer) * 17.0);
    vec2 offset = (vec2(hash21(id + 3.1), hash21(id + 7.7)) - 0.5) * 0.6;
    float size = mix(0.03, 0.1, fract(h * 13.7));
    float twinkle = 0.65 + 0.35 * sin(u_clock * 2.0 + h * 40.0);
    float threshold = 1.0 - clamp(u_p3, 0.0, 4.0) * 0.025;
    total += step(threshold, h) * smoothstep(size, 0.0, length(f - offset)) * twinkle;
  }
  return total;
}
float lookValue(vec2 uv) {
  vec2 p = uv * u_p0;
  float t = u_phase;
  mat2 way = rotation(radians(u_p4));
  vec2 q = vec2(fbm(p + way * vec2(0.0, t * 0.6), u_p1), fbm(p + vec2(5.2, 1.3) - way * vec2(t * 0.5, 0.0), u_p1));
  float cloud = fbm(p + 2.0 * q + way * vec2(t * 0.25, -t * 0.15), u_p1);
  cloud = clamp((cloud - 0.5) * u_p2 + 0.5, 0.0, 1.0);
  return clamp(cloud + nebulaStars(uv) * 0.85, 0.0, 1.0);
}
`;

/** Space: stars in depth, each layer coming toward the viewer at its own speed, over a soft glow.
 *  p0 density, p1 size, p2 twinkle, p3 depth (how much the layers differ in speed), p4 glow, p5 sway
 *  (the whole field drifts side to side). */
const SPACE = `
float lookValue(vec2 uv) {
  uv += vec2(sin(u_phase * 0.9), cos(u_phase * 0.6) * 0.6) * 0.06 * u_p5;
  float total = 0.0;
  for (int i = 0; i < 5; i++) {
    float layer = float(i);
    float speed = 0.08 * (0.6 + layer * 0.25 * u_p3 + 0.2);
    // z runs 0..1 and starts over; a star grows and brightens as it gets close, and fades at both ends.
    float z = fract(u_phase * speed + layer / 5.0);
    float cells = mix(34.0, 6.0, z * z);
    float fade = smoothstep(0.0, 0.12, z) * smoothstep(1.0, 0.82, z);
    total += starLayer(uv + layer * 3.7, cells, layer * 13.0, u_p0 * 0.06, u_p1 * (0.7 + z), u_p2) * fade;
  }
  float glow = fbm(uv * 1.4 + vec2(u_phase * 0.02, 0.0), 0.3);
  return clamp(glow * u_p4 * 0.45 + total * 0.9, 0.0, 1.0);
}
`;

/** Galaxy: a spiral turning around a bright core, with dust lanes and stars.
 *  p0 arms, p1 twist, p2 core, p3 dust, p4 stars, p5 tilt (how much the disc is seen from the side),
 *  p6 breathing (the spiral slowly tightens and loosens). Positive phase turns it clockwise. */
const GALAXY = `
float lookValue(vec2 uv) {
  uv.y /= mix(1.0, 0.3, u_p5);
  vec2 p = rotation(-u_phase * 0.12) * uv;
  float r = length(p);
  float angle = atan(p.y, p.x);
  float arms = floor(u_p0 + 0.5);
  // A logarithmic spiral: the angle of the arm depends on the radius.
  float twist = u_p1 + u_p6 * 1.5 * sin(u_phase * 0.35);
  float wind = angle * arms + twist * arms * 0.5 * log(r + 0.08);
  float cloud = fbm(p * 4.0 + 3.0, 0.6);
  float arm = pow(0.5 + 0.5 * cos(wind), 2.5) * (0.55 + 0.9 * cloud) * exp(-r * 2.2);
  float core = u_p2 * exp(-r * r * 38.0);
  float dust = u_p3 * 0.55 * pow(0.5 + 0.5 * cos(wind + 1.4), 3.0) * cloud * exp(-r * 1.4);
  float stars = starLayer(p * 1.2, 60.0, 5.0, u_p4 * 0.08, 1.0, 0.4) * (0.4 + arm * 2.0);
  return clamp(arm * 0.9 + core + stars - dust, 0.0, 1.0);
}
`;

const POST = `
vec3 aces(vec3 x) { return clamp((x * (2.51 * x + 0.03)) / (x * (2.43 * x + 0.59) + 0.14), 0.0, 1.0); }
float bayer2(float x, float y) { return 2.0 * x + 3.0 * y - 4.0 * x * y; }
// 4x4 Bayer threshold: the 2x2 pattern repeated at two scales.
float bayer4(vec2 p) {
  vec2 q = mod(floor(p), 4.0), lo = mod(q, 2.0), hi = floor(q / 2.0);
  return (4.0 * bayer2(lo.x, lo.y) + bayer2(hi.x, hi.y) + 0.5) / 16.0;
}

void main() {
  vec2 uv = (gl_FragCoord.xy - 0.5 * u_res) / u_res.y;
  float value = lookValue(uv);
  value = pow(value, exp2(-u_bias * 2.0));
  value = clamp((value - 0.5) * (1.0 + u_sharp * 5.0) + 0.5, 0.0, 1.0);
  vec3 color = texture2D(u_lut, vec2(value, 0.5)).rgb;

  // Grade, in display space.
  color *= u_bright;
  color = (color - 0.5) * u_gcontrast + 0.5;
  float luma = dot(color, vec3(0.2126, 0.7152, 0.0722));
  color = mix(vec3(luma), color, u_sat);
  color = max(color, 0.0);

  // Tone, in linear light.
  vec3 lin = pow(color, vec3(2.2)) * exp2(u_exposure);
  lin = u_aces > 0.5 ? aces(lin) : clamp(lin, 0.0, 1.0);
  color = pow(lin, vec3(1.0 / 2.2));

  float r = length(uv * vec2(1.0, 1.15));
  color *= 1.0 - u_vignette * smoothstep(0.25, 1.1, r);

  // Grain, in grain-sized cells. Mid-tone focus keeps it out of pure blacks and whites.
  if (u_grainAmount > 0.0) {
    vec2 cell = floor(gl_FragCoord.xy / max(u_grainSize * u_dpr, 0.5));
    float frame = u_grainAnimate > 0.5 ? floor(u_clock * 24.0) : 0.0;
    vec2 s = cell + vec2(u_seed * 37.0, frame * 11.0);
    float n1 = hash21(s) - 0.5, n2 = hash21(s + 19.7) - 0.5, n3 = hash21(s + 41.3) - 0.5;
    float n = u_grainType > 0.5 ? (n1 + n2 + n3) * 0.66 : n1 * 2.0;
    float l = dot(color, vec3(0.2126, 0.7152, 0.0722));
    float weight = mix(1.0, 4.0 * l * (1.0 - l), u_grainMid);
    vec3 grain = mix(vec3(n), vec3(n1, n2, n3) * 2.0, u_grainColor);
    color += grain * u_grainAmount * 4.0 * weight;
  }

  // Ordered dither: a handful of levels per channel, spread with a Bayer pattern.
  if (u_dither > 0.5) {
    float levels = 7.0;
    color = floor(color * levels + bayer4(gl_FragCoord.xy / max(u_dpr, 1.0))) / levels;
  }
  gl_FragColor = vec4(clamp(color, 0.0, 1.0), 1.0);
}
`;

const BODIES: Record<string, string> = { nebula: NEBULA, space: SPACE, galaxy: GALAXY };

/** The whole fragment shader of a look (an unknown id gives Nebula). */
export const fragmentShader = (look: string) => COMMON + (BODIES[look] ?? NEBULA) + POST;
export const SHADER_LOOKS = Object.keys(BODIES);
