import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import { PALETTES, DEFAULT_PALETTE, PALETTE_STORAGE_KEY, readPalette, applyPaletteToRoot, createPalettePreference } from "../src/lib/services/palette-preference.js";

const root = () => {
  const attributes = new Map();
  return { attributes, setAttribute: (name, value) => attributes.set(name, value), removeAttribute: name => attributes.delete(name) };
};
const storage = (initial) => {
  const values = new Map(initial === undefined ? [] : [[PALETTE_STORAGE_KEY, initial]]);
  return { values, getItem: key => values.get(key) ?? null, setItem: (key, value) => values.set(key, value) };
};

test("the default palette is the original black and gray, and unknown or missing values fall back to it", () => {
  assert.equal(DEFAULT_PALETTE, "graphite");
  assert.equal(readPalette(undefined), "graphite");
  for (const bad of [null, "", "neon", "OCEAN", "__proto__"]) assert.equal(readPalette(storage(bad)), "graphite");
  assert.equal(readPalette({ getItem() { throw new Error("blocked"); } }), "graphite");
  assert.equal(readPalette(storage("ocean")), "ocean");
});

test("choosing a palette marks the page, saves it, and the default leaves the page unmarked", () => {
  const page = root(), saved = storage();
  const changes = [];
  const preference = createPalettePreference({ storage: saved, root: page, changed: value => changes.push(value) });
  assert.equal(page.attributes.has("data-palette"), false);
  assert.equal(preference.set("violet"), "violet");
  assert.equal(page.attributes.get("data-palette"), "violet");
  assert.equal(saved.values.get(PALETTE_STORAGE_KEY), "violet");
  assert.equal(preference.set("nope"), "violet", "an unknown palette changes nothing");
  assert.equal(preference.set("graphite"), "graphite");
  assert.equal(page.attributes.has("data-palette"), false);
  assert.deepEqual(changes, ["violet", "graphite"]);
  const restored = createPalettePreference({ storage: storage("amber"), root: root() });
  assert.equal(restored.get(), "amber");
});

test("a failing storage still lets the palette change for the session", () => {
  const page = root();
  const preference = createPalettePreference({ storage: { getItem: () => null, setItem() { throw new Error("full"); } }, root: page });
  assert.equal(preference.set("rose"), "rose");
  assert.equal(page.attributes.get("data-palette"), "rose");
  applyPaletteToRoot(undefined, "rose");
});

test("every palette has colors in the dark and the light theme, and the default adds none", () => {
  const css = readFileSync(new URL("../src/styles/theme.css", import.meta.url), "utf8");
  const rule = selector => {
    const start = css.indexOf(`${selector} {`);
    return start < 0 ? "" : css.slice(start, css.indexOf("}", start));
  };
  for (const palette of PALETTES.filter(id => id !== DEFAULT_PALETTE)) {
    assert.ok(rule(`html[data-palette="${palette}"]`).includes("--accent-primary"), `${palette} dark`);
    assert.ok(rule(`html[data-palette="${palette}"][data-theme="light"]`).includes("--accent-primary"), `${palette} light`);
  }
  assert.equal(css.includes('data-palette="graphite"'), false);
  const settings = readFileSync(new URL("../src/lib/components/settings/settings.css", import.meta.url), "utf8");
  for (const palette of PALETTES) assert.ok(settings.includes(`.palette-${palette} { --swatch`), `${palette} swatch`);
});

test("a window follows a palette chosen in another window without writing it back", () => {
  const page = root(), saved = storage("ocean");
  const changes = [];
  const preference = createPalettePreference({ storage: saved, root: page, changed: value => changes.push(value) });
  assert.equal(preference.sync(), "ocean");
  assert.deepEqual(changes, [], "nothing changed, nothing announced");
  saved.values.set(PALETTE_STORAGE_KEY, "rose");
  assert.equal(preference.sync(), "rose");
  assert.equal(page.attributes.get("data-palette"), "rose");
  saved.values.delete(PALETTE_STORAGE_KEY);
  assert.equal(preference.sync(), "graphite");
  assert.equal(page.attributes.has("data-palette"), false);
  assert.deepEqual(changes, ["rose", "graphite"]);
});

test("the new-tab page and the shield button follow the palette too", () => {
  const read = path => readFileSync(new URL(`../${path}`, import.meta.url), "utf8");
  const palette = read("public/new-tab/palette.css");
  for (const id of PALETTES.filter(name => name !== DEFAULT_PALETTE)) {
    assert.ok(palette.includes(`html[data-palette="${id}"] {`), `${id} dark on the new tab`);
    assert.ok(palette.includes(`html[data-palette="${id}"][data-theme="light"] {`), `${id} light on the new tab`);
  }
  assert.ok(read("public/new-tab.html").includes("/new-tab/palette.css"));
  const page = read("public/new-tab/page.js");
  assert.ok(page.includes("spark_color_palette") && page.includes("data-palette"));
  for (const file of ["page.css", "widgets.css", "choice-picker.css", "settings-layout.css", "wallpaper.css", "audio-background.css", "media-card.css"]) {
    assert.equal(/#e5e5e5|#bcb6df|#aaa5cf|#1db954/i.test(read(`public/new-tab/${file}`)), false, `${file} has no fixed accent left`);
  }
  const shield = read("src/lib/components/titlebar/TitlebarActions.svelte");
  assert.ok(shield.includes("var(--status-on)") && !shield.includes("#22c55e"));
  const theme = read("src/styles/theme.css");
  assert.ok(theme.includes("--status-on: #22c55e"), "the default palette keeps the original green");
});
