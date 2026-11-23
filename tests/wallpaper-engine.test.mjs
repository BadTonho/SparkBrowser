import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';
import ts from 'typescript';

function load(path) {
  const source = ts.transpileModule(readFileSync(path, 'utf8'), {
    compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 },
  }).outputText;
  const exports = {};
  runInNewContext(source, { exports, require: () => ({}), Math, Number, Uint8Array });
  return exports;
}
const color = load('src/lib/wallpaper/color.ts');
const params = load('src/lib/wallpaper/params.ts');
const near = (a, b, eps = 1e-3) => assert.ok(Math.abs(a - b) <= eps, `${a} is not close to ${b}`);
const plain = value => JSON.parse(JSON.stringify(value));

test('colors survive a trip through OKLab and OKLCH', () => {
  for (const rgb of [[1, 0, 0], [0.2, 0.5, 0.9], [0.5, 0.5, 0.5], [0, 0, 0], [1, 1, 1]]) {
    const back = color.oklchToRgb(color.rgbToOklch(rgb));
    for (let i = 0; i < 3; i++) near(back[i], rgb[i], 2e-3);
  }
  assert.equal(color.rgbToHex(color.hexToRgb('#ff9a3d')), '#ff9a3d');
  assert.deepEqual(plain(color.hexToRgb('nonsense')), [0, 0, 0]);
});

test('the gradient strip holds the end colors and passes through the stops in every blend', () => {
  const stops = [{ position: 0.25, color: [1, 0, 0] }, { position: 0.75, color: [0, 0, 1] }];
  for (const blend of ['srgb', 'linear', 'oklab', 'oklch']) {
    const lut = color.buildGradientLut(stops, blend, 256);
    assert.equal(lut.length, 256 * 4);
    assert.deepEqual(plain([...lut.slice(0, 4)]), [255, 0, 0, 255], `${blend} start`);
    assert.deepEqual(plain([...lut.slice(255 * 4, 255 * 4 + 4)]), [0, 0, 255, 255], `${blend} end`);
    // Stops given in a different order give the same strip.
    assert.deepEqual(plain([...color.buildGradientLut([...stops].reverse(), blend, 64)]), plain([...color.buildGradientLut(stops, blend, 64)]));
  }
  // The middle differs between spaces: sRGB mixes to a dark purple, OKLab stays lighter.
  const mid = blend => color.buildGradientLut(stops, blend, 3).slice(4, 7);
  assert.ok(mid('oklab')[0] > mid('srgb')[0] || mid('oklab')[2] > mid('srgb')[2]);
  assert.equal(color.buildGradientLut([], 'oklab', 4).length, 16);
});

test('OKLCH takes the short way around the hue circle', () => {
  const a = color.blendColors([1, 0, 0.1], [1, 0.1, 0], 0.5, 'oklch');
  assert.ok(a[0] > 0.8, 'red to red-orange stays red');
});

test('a random palette has the requested size and valid colors', () => {
  let seed = 7;
  const random = () => (seed = (seed * 16807) % 2147483647) / 2147483647;
  const palette = color.randomPalette(random, 8);
  assert.equal(palette.length, 8);
  for (const rgb of palette) for (const channel of rgb) assert.ok(channel >= 0 && channel <= 1);
});

test('the palette of an image has the colors that are in it, darkest first', () => {
  const pixels = [];
  for (const [r, g, b] of [[10, 10, 10], [250, 20, 20], [20, 20, 250], [250, 250, 250]]) {
    for (let i = 0; i < 50; i++) pixels.push(r, g, b, 255);
  }
  pixels.push(0, 255, 0, 0); // transparent, ignored
  const palette = color.paletteFromImage(pixels, 4);
  assert.equal(palette.length, 4);
  const hexes = palette.map(color.rgbToHex);
  assert.equal(hexes[0], '#0a0a0a');
  assert.equal(hexes[3], '#fafafa');
  assert.ok(hexes.includes('#fa1414') && hexes.includes('#1414fa'));
  assert.deepEqual(plain(color.paletteFromImage([], 8)), []);
});

test('defaults are the reference values and every number sits inside its range', () => {
  const p = params.defaultParams();
  assert.equal(p.colors.length, 8);
  assert.equal(p.stops.length, 8);
  assert.equal(p.activeColors, 6);
  assert.equal(p.blend, 'oklab');
  assert.deepEqual(plain(p.shape), { scale: 3, billow: 1.42, contrast: 1.35, stars: 1.8 });
  assert.equal(p.effects.grain.type, 'film');
  assert.equal(p.effects.tone.mode, 'aces');
  const { SPECS } = params;
  const inside = (spec, value) => value >= spec.min && value <= spec.max;
  assert.ok(inside(SPECS.scale, p.shape.scale) && inside(SPECS.billow, p.shape.billow) && inside(SPECS.stars, p.shape.stars));
  assert.ok(inside(SPECS.drift, p.drift) && inside(SPECS.brightness, p.grade.brightness));
  assert.ok(inside(SPECS.grainAmount, p.effects.grain.amount) && inside(SPECS.exposure, p.effects.tone.exposure));
});

test('values are clamped to the range and snapped to the step', () => {
  const { SPECS, clampTo } = params;
  assert.equal(clampTo(SPECS.scale, 99), 8);
  assert.equal(clampTo(SPECS.scale, -5), 0.5);
  assert.equal(clampTo(SPECS.activeColors, 5.6), 6);
  assert.equal(clampTo(SPECS.grainSeed, 0), 1);
  assert.equal(clampTo(SPECS.bias, NaN), -1);
  near(clampTo(SPECS.exposure, 0.213), 0.2);
});

test('resetting a section changes only that section', () => {
  const changed = params.defaultParams();
  changed.blend = 'srgb'; changed.bias = -0.5; changed.stops = [0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7];
  changed.effects.grain.amount = 0.08; changed.effects.dither = true; changed.shape.scale = 6;
  const gradient = params.resetGradient(changed);
  assert.equal(gradient.blend, 'oklab');
  assert.equal(gradient.bias, 0.22);
  assert.deepEqual(plain(gradient.stops), plain(params.defaultParams().stops));
  assert.equal(gradient.effects.dither, true, 'effects are untouched by the gradient reset');
  assert.equal(gradient.shape.scale, 6);
  const effects = params.resetEffects(changed);
  assert.equal(effects.effects.grain.amount, 0.012);
  assert.equal(effects.effects.dither, false);
  assert.equal(effects.blend, 'srgb', 'the gradient is untouched by the effects reset');
});

test('a stop can only move between its neighbours', () => {
  const stops = [0, 0.2, 0.4, 0.6, 0.8, 1];
  assert.deepEqual(plain(params.stopRange(stops, 2, 6)), [0.21, 0.59]);
  assert.deepEqual(plain(params.stopRange(stops, 0, 6)), [0, 0.19]);
  assert.deepEqual(plain(params.stopRange(stops, 5, 6)), [0.81, 1]);
});

test('settings read from storage are checked, clamped and completed with defaults', () => {
  const defaults = plain(params.defaultParams());
  for (const junk of [null, 5, 'x', [], undefined]) assert.deepEqual(plain(params.sanitizeParams(junk)), defaults);
  const saved = params.sanitizeParams({
    colors: ['#FF0000', 'red', 5], activeColors: 99, stops: [2, -1, 'x'], blend: 'cmyk', bias: 'x',
    shape: { scale: 100, stars: 0.5 }, effects: { grain: { type: 'bogus', amount: -1, animate: 'yes' }, tone: { mode: 'none' }, dither: true },
    extra: { evil: true },
  });
  assert.equal(saved.colors[0], '#ff0000');
  assert.equal(saved.colors[1], defaults.colors[1]);
  assert.equal(saved.activeColors, 8);
  assert.deepEqual(plain(saved.stops.slice(0, 3)), [1, 0, defaults.stops[2]]);
  assert.equal(saved.blend, 'oklab');
  assert.equal(saved.bias, defaults.bias);
  assert.equal(saved.shape.scale, 8);
  assert.equal(saved.shape.stars, 0.5);
  assert.equal(saved.shape.billow, defaults.shape.billow);
  assert.equal(saved.effects.grain.type, 'film');
  assert.equal(saved.effects.grain.amount, 0);
  assert.equal(saved.effects.grain.animate, false);
  assert.equal(saved.effects.tone.mode, 'none');
  assert.equal(saved.effects.dither, true);
  assert.equal('extra' in saved, false);
  // What the editor produces survives a round trip unchanged.
  assert.deepEqual(plain(params.sanitizeParams(plain(params.defaultParams()))), defaults);
});
