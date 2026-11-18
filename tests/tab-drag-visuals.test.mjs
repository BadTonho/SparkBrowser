import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { runInNewContext } from "node:vm";
import test from "node:test";
import ts from "typescript";

function load(path, globals = {}) {
  const source = ts.transpileModule(readFileSync(path, "utf8"), {
    compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 },
  }).outputText;
  const exports = {};
  runInNewContext(source, { exports, ...globals });
  return exports;
}

function element(id, rect) {
  const animations = [];
  return {
    dataset: { tabId: id }, animations, rect,
    getBoundingClientRect() { return this.rect; },
    animate(frames, options) { animations.push({ frames, options }); },
  };
}

test("a reordered list slides each moved item from its old place, and nothing else", () => {
  const a = element("a", { left: 0, top: 0 }), b = element("b", { left: 100, top: 0 }), c = element("c", { left: 200, top: 0 });
  const list = { querySelectorAll: () => [a, b, c] };
  const { createListFlip } = load("src/lib/services/list-flip.ts", { matchMedia: () => ({ matches: false }) });
  const flip = createListFlip(() => list, ".h-tab");
  flip.measure();
  a.rect = { left: 100, top: 0 }; b.rect = { left: 0, top: 0 };
  flip.play();
  assert.equal(a.animations.length, 1);
  assert.equal(a.animations[0].frames[0].transform, "translate(-100px, 0px)");
  assert.equal(b.animations[0].frames[0].transform, "translate(100px, 0px)");
  assert.equal(c.animations.length, 0, "an item that did not move is not animated");
  flip.play();
  assert.equal(a.animations.length, 1, "a play without a measure does nothing");
});

test("adding or removing an item leaves the animation to the list's own transitions, and reduced motion skips it", () => {
  const a = element("a", { left: 0, top: 0 }), b = element("b", { left: 100, top: 0 });
  let items = [a, b];
  const list = { querySelectorAll: () => items };
  const { createListFlip } = load("src/lib/services/list-flip.ts", { matchMedia: () => ({ matches: false }) });
  const flip = createListFlip(() => list, ".h-tab");
  flip.measure();
  items = [b]; b.rect = { left: 0, top: 0 };
  flip.play();
  assert.equal(b.animations.length, 0);
  const reduced = load("src/lib/services/list-flip.ts", { matchMedia: () => ({ matches: true }) }).createListFlip(() => list, ".h-tab");
  items = [a, b]; reduced.measure(); a.rect = { left: 50, top: 0 }; reduced.play();
  assert.equal(a.animations.length, 0);
});

test("the ghost is a copy of the tab that keeps the grabbed point under the pointer and is not a tab itself", () => {
  const appended = [];
  const clone = {
    classList: { classes: new Set(["h-tab", "is-dragging"]), add(c) { this.classes.add(c); }, remove(c) { this.classes.delete(c); } },
    style: {}, removed: false, attributes: new Map([["id", "x"], ["data-tab-id", "a"], ["tabindex", "0"], ["title", "t"]]),
    querySelectorAll: () => [],
    removeAttribute(name) { this.attributes.delete(name); }, setAttribute(name, value) { this.attributes.set(name, value); },
    remove() { this.removed = true; },
  };
  const source = { getBoundingClientRect: () => ({ left: 100, top: 10, width: 240, height: 28 }), cloneNode: () => clone };
  const container = { appendChild: node => appended.push(node) };
  const { createTabGhost } = load("src/lib/services/tab-drag-ghost.ts");
  const ghost = createTabGhost(source, container, 130, 20);
  assert.deepEqual(appended, [clone]);
  assert.ok(clone.classList.classes.has("tab-drag-ghost") && !clone.classList.classes.has("is-dragging"));
  assert.equal(clone.attributes.has("data-tab-id") || clone.attributes.has("id") || clone.attributes.has("tabindex"), false);
  assert.equal(clone.style.pointerEvents, "none");
  assert.equal(clone.style.width, "240px");
  ghost.move(500, 60);
  // The pointer grabbed the tab 30px from its left edge and 10px from its top.
  assert.match(clone.style.transform, /translate3d\(470px, 50px, 0\)/);
  ghost.setHidden(true);
  assert.equal(clone.style.visibility, "hidden");
  ghost.destroy();
  assert.equal(clone.removed, true);
});
