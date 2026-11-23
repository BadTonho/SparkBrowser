/** The Nebula look: GLSL ES 1.00 (works in WebGL 1 and 2). Domain-warped FBM clouds plus a star field, mapped
 *  through the gradient strip, then graded, toned, vignetted, grained and optionally dithered. */
export const VERTEX_SHADER = `
attribute vec2 a_position;
void main() { gl_Position = vec4(a_position, 0.0, 1.0); }
`;

export const NEBULA_FRAGMENT = `
precision highp float;
uniform vec2 u_res;
uniform float u_phase;      // cloud time: grows with the drift speed
uniform float u_clock;      // real seconds, for twinkling and animated grain
uniform sampler2D u_lut;
uniform float u_scale, u_billow, u_contrast, u_stars;
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
float fbm(vec2 p) {
  float sum = 0.0, amp = 0.5;
  mat2 rot = mat2(0.8, -0.6, 0.6, 0.8);
  for (int i = 0; i < 6; i++) {
    float n = noise(p);
    // Billow: from smooth clouds (0) to puffy, folded ones.
    n = mix(n, abs(2.0 * n - 1.0), clamp(u_billow * 0.5, 0.0, 1.0));
    sum += amp * n;
    p = rot * p * 2.03 + 17.0;
    amp *= 0.5;
  }
  return sum;
}
float stars(vec2 uv) {
  float total = 0.0;
  for (int layer = 0; layer < 2; layer++) {
    float cells = 55.0 + float(layer) * 70.0;
    vec2 g = uv * cells, id = floor(g), f = fract(g) - 0.5;
    float h = hash21(id + float(layer) * 17.0);
    vec2 offset = (vec2(hash21(id + 3.1), hash21(id + 7.7)) - 0.5) * 0.6;
    float size = mix(0.03, 0.1, fract(h * 13.7));
    float twinkle = 0.65 + 0.35 * sin(u_clock * 2.0 + h * 40.0);
    float threshold = 1.0 - clamp(u_stars, 0.0, 4.0) * 0.025;
    total += step(threshold, h) * smoothstep(size, 0.0, length(f - offset)) * twinkle;
  }
  return total;
}
vec3 aces(vec3 x) { return clamp((x * (2.51 * x + 0.03)) / (x * (2.43 * x + 0.59) + 0.14), 0.0, 1.0); }
float bayer2(float x, float y) { return 2.0 * x + 3.0 * y - 4.0 * x * y; }
// 4x4 Bayer threshold: the 2x2 pattern repeated at two scales.
float bayer4(vec2 p) {
  vec2 q = mod(floor(p), 4.0), lo = mod(q, 2.0), hi = floor(q / 2.0);
  return (4.0 * bayer2(lo.x, lo.y) + bayer2(hi.x, hi.y) + 0.5) / 16.0;
}

void main() {
  vec2 uv = (gl_FragCoord.xy - 0.5 * u_res) / u_res.y;
  vec2 p = uv * u_scale;
  float t = u_phase;
  vec2 q = vec2(fbm(p + vec2(0.0, t * 0.6)), fbm(p + vec2(5.2, 1.3) - vec2(t * 0.5, 0.0)));
  float cloud = fbm(p + 2.0 * q + vec2(t * 0.25, -t * 0.15));
  cloud = clamp((cloud - 0.5) * u_contrast + 0.5, 0.0, 1.0);
  float value = clamp(cloud + stars(uv) * 0.85, 0.0, 1.0);
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
