import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import vm from "node:vm";
import test from "node:test";

const read = (name) => readFileSync(new URL(`../src-tauri/src/privacy/${name}`, import.meta.url), "utf8");
const template = read("content_shield.js");
const youtube = read("youtube_ads.js");

// Builds the script the way `build_content_shield_script_with_exceptions` does.
function page({ url = "https://www.youtube.com/watch?v=x", enabled = true, exceptions = [] } = {}) {
  const sheets = [];
  class CSSStyleSheet {
    replaceSync(css) { this.css = css; }
    disabled = false;
  }
  class Response {
    constructor(body) { this.body = body; }
    json() { return Promise.resolve(JSON.parse(this.body)); }
  }
  const sandbox = {
    URL, Proxy, Reflect, Promise, CSSStyleSheet, Response, XMLHttpRequest: function () {}, DOMException,
    navigator: {},
    location: new URL(url),
    document: {
      baseURI: url, readyState: "complete", addEventListener() {}, getElementById: () => null,
      createElement: () => ({}), head: { appendChild() {} }, documentElement: { appendChild() {} },
      get adoptedStyleSheets() { return sheets; }, set adoptedStyleSheets(value) { sheets.splice(0, sheets.length, ...value); },
    },
  };
  sandbox.XMLHttpRequest.prototype.open = function () {};
  sandbox.XMLHttpRequest.prototype.send = function () {};
  sandbox.window = sandbox;
  const context = vm.createContext(sandbox);
  vm.runInContext(
    template
      .replace("/*__YOUTUBE_ADS__*/", `${youtube}refreshYoutube = youtubeAds(() => enabled && !siteExcluded());`)
      .replace("__SHIELD_TOKEN__", JSON.stringify("t"))
      .replace("__SHIELD_ENABLED__", String(enabled))
      .replace("__BLOCKED_DOMAINS__", "[]")
      .replace("__SHIELD_EXCEPTIONS__", JSON.stringify(exceptions)),
    context,
  );
  return { context, sheets, run: (code) => vm.runInContext(code, context) };
}

const player = () => ({
  videoDetails: { videoId: "x" },
  adPlacements: [{ adPlacementRenderer: {} }],
  playerAds: [{}],
  adSlots: [{}],
  streamingData: { formats: [] },
});

test("ad lists are removed from the page's first player data and from parsed and fetched data", async () => {
  const p = page();
  p.context.ytInitialPlayerResponse = player();
  const initial = p.run("ytInitialPlayerResponse");
  assert.deepEqual(Object.keys(initial).sort(), ["streamingData", "videoDetails"]);

  const parsed = p.run(`JSON.parse(${JSON.stringify(JSON.stringify({ playerResponse: player(), adSlots: [1] }))})`);
  assert.deepEqual(Object.keys(parsed.playerResponse).sort(), ["streamingData", "videoDetails"]);
  assert.equal("adSlots" in parsed, false);

  const fetched = await p.run(`new Response(${JSON.stringify(JSON.stringify(player()))}).json()`);
  assert.deepEqual(Object.keys(fetched).sort(), ["streamingData", "videoDetails"]);
});

test("pages that are not YouTube are left alone", () => {
  for (const url of ["https://example.test/", "https://youtube.com.example.test/", "https://music.youtube.com/"]) {
    const p = page({ url });
    const before = p.run("JSON.parse").toString();
    const parsed = p.run(`JSON.parse('{"adPlacements":[1]}')`);
    assert.deepEqual(Object.keys(parsed), ["adPlacements"], url);
    assert.equal(p.sheets.length, 0, url);
    assert.ok(before.includes("native code"));
  }
});

test("a disabled shield or a site exception keeps the data intact, and switching it takes effect at once", () => {
  for (const options of [{ enabled: false }, { exceptions: ["youtube.com"] }]) {
    const p = page(options);
    assert.deepEqual(Object.keys(p.run(`JSON.parse('{"adPlacements":[1]}')`)), ["adPlacements"]);
  }
  const p = page({ enabled: false });
  p.context.__sparkSetShieldPolicy("wrong", true);
  assert.deepEqual(Object.keys(p.run(`JSON.parse('{"adPlacements":[1]}')`)), ["adPlacements"]);
  p.context.__sparkSetShieldPolicy("t", true);
  assert.deepEqual(Object.keys(p.run(`JSON.parse('{"adPlacements":[1]}')`)), []);
  assert.equal(p.sheets.length, 1);
  assert.equal(p.sheets[0].disabled, false);
  p.context.__sparkSetShieldPolicy("t", false);
  assert.equal(p.sheets[0].disabled, true);
});

test("the page's ad boxes are hidden with a style sheet that cannot be refused by a content security policy", () => {
  const p = page();
  assert.equal(p.sheets.length, 1);
  assert.match(p.sheets[0].css, /ytd-ad-slot-renderer/);
  assert.match(p.sheets[0].css, /display:none!important/);
  assert.doesNotMatch(p.sheets[0].css, /#movie_player|ytd-player(?!-legacy)/, "the player itself is never hidden");
});
