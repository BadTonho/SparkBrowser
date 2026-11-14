import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import vm from "node:vm";
import test from "node:test";

const rust = readFileSync(new URL("../src-tauri/src/privacy/adblock_engine.rs", import.meta.url), "utf8");
const domains = [...rust.match(/const BLOCKED_DOMAINS:[\s\S]*?= &\[([\s\S]*?)\];/)[1].matchAll(/"([^"]+)"/g)].map((match) => match[1]);
const template = readFileSync(new URL("../src-tauri/src/privacy/content_shield.js", import.meta.url), "utf8");

function page(enabled, replacementEnabled) {
  let fetches = 0, sends = 0, beacons = 0, aborted = 0;
  function XMLHttpRequest() {}
  XMLHttpRequest.prototype.open = function() {};
  XMLHttpRequest.prototype.send = function() { sends++; };
  XMLHttpRequest.prototype.abort = function() { aborted++; };
  const window = { fetch: async () => { fetches++; return { status: 200 }; } };
  const navigator = { sendBeacon: () => { beacons++; return true; } };
  const document = { baseURI: "https://example.test/", readyState: "loading", addEventListener() {} };
  const source = value => template.replace("__SHIELD_TOKEN__", JSON.stringify("test-policy")).replace("__SHIELD_ENABLED__", String(value)).replace("__BLOCKED_DOMAINS__", JSON.stringify(domains)).replace("__SHIELD_EXCEPTIONS__", "[]");
  const context = vm.createContext({ window, navigator, document, location: new URL(document.baseURI), XMLHttpRequest, URL, DOMException });
  const replacement = value => `if (typeof window.__sparkSetShieldPolicy === 'function') { window.__sparkSetShieldPolicy("test-policy", ${value}); } else { ${source(value)} }`;
  if (replacementEnabled !== undefined) vm.runInContext(replacement(replacementEnabled), context);
  vm.runInContext(source(enabled), context);
  return { window, navigator, XMLHttpRequest, replace: value => vm.runInContext(replacement(value), context), counts: () => ({ fetches, sends, beacons, aborted }) };
}

test("blocks tracker strings, URL and Request objects while allowing unrelated paths and hosts", async () => {
  const p = page(true);
  for (const input of ["https://google-analytics.com/collect", new URL("https://sub.google-analytics.com/collect"), new Request("https://doubleclick.net/ad")]) {
    await assert.rejects(p.window.fetch(input), /Blocked by Spark Shield/);
  }
  for (const input of ["https://example.test/google-analytics.com", "https://google-analytics.com.example.test/", "/docs/doubleclick.net"]) {
    assert.equal((await p.window.fetch(input)).status, 200);
  }
  assert.equal(p.counts().fetches, 3);
});

test("XHR reuse clears the previous block and sendBeacon uses the same host policy", () => {
  const p = page(true);
  const xhr = new p.XMLHttpRequest();
  xhr.open("GET", "https://doubleclick.net/ad");
  assert.throws(() => xhr.send(), /Blocked by Spark Shield/);
  xhr.open("GET", "/allowed");
  xhr.send();
  assert.equal(p.navigator.sendBeacon("https://doubleclick.net/collect"), false);
  assert.equal(p.navigator.sendBeacon("/allowed"), true);
  assert.deepEqual(p.counts(), { fetches: 0, sends: 1, beacons: 1, aborted: 1 });
});

test("new documents with protection disabled retain the original request APIs", async () => {
  const p = page(false);
  assert.equal((await p.window.fetch("https://doubleclick.net/ad")).status, 200);
  const xhr = new p.XMLHttpRequest();
  xhr.open("GET", "https://doubleclick.net/ad");
  xhr.send();
  assert.equal(p.navigator.sendBeacon("https://doubleclick.net/collect"), true);
  assert.deepEqual(p.counts(), { fetches: 1, sends: 1, beacons: 1, aborted: 0 });
});

test("live policy changes require the generation token and affect existing XHR/fetch/beacon wrappers", async () => {
  const p = page(true);
  p.window.__sparkSetShieldPolicy("wrong-token", false);
  await assert.rejects(p.window.fetch("https://doubleclick.net/ad"));
  const xhr = new p.XMLHttpRequest(); xhr.open("GET", "https://doubleclick.net/ad");
  p.window.__sparkSetShieldPolicy("test-policy", false);
  xhr.send();
  assert.equal((await p.window.fetch("https://doubleclick.net/ad")).status, 200);
  assert.equal(p.navigator.sendBeacon("https://doubleclick.net/ad"), true);
  p.window.__sparkSetShieldPolicy("test-policy", true);
  await assert.rejects(p.window.fetch("https://doubleclick.net/ad"));
});

test("future-document policy works before or after the immutable base initializer", async () => {
  for (const base of [true, false]) {
    const before = page(base, !base);
    const after = page(base); after.replace(!base);
    for (const p of [before, after]) {
      if (base) assert.equal((await p.window.fetch("https://doubleclick.net/ad")).status, 200);
      else await assert.rejects(p.window.fetch("https://doubleclick.net/ad"));
      p.replace(base);
      if (base) await assert.rejects(p.window.fetch("https://doubleclick.net/ad"));
      else assert.equal((await p.window.fetch("https://doubleclick.net/ad")).status, 200);
    }
  }
});
