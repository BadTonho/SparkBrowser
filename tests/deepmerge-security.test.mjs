import assert from "node:assert/strict";
import test from "node:test";
import { createRequire } from "node:module";
const require = createRequire(import.meta.url);
const pluginRequire = createRequire(import.meta.resolve("@sveltejs/vite-plugin-svelte"));
const merge = pluginRequire("deepmerge");

test("the actual plugin dependency cannot poison the returned prototype from target or source", () => {
  const target = JSON.parse('{"__proto__":{"polluted":true},"valid":1}');
  const source = Object.assign(Object.create(null), JSON.parse('{"__proto__":{"injected":true},"other":2}'));
  for (const result of [merge(target, {}), merge(source, source), merge({}, target)]) {
    assert.equal(Object.getPrototypeOf(result), Object.prototype);
    assert.equal(result.polluted, undefined);
    assert.equal(result.injected, undefined);
  }
  assert.equal({}.polluted, undefined);
});

test("nested options, arrays, symbols and custom merging keep their existing behavior", () => {
  const symbol = Symbol("option");
  assert.deepEqual(merge({ a: { b: 1 }, list: [1], [symbol]: 1 }, { a: { c: 2 }, list: [2] }), { a: { b: 1, c: 2 }, list: [1, 2], [symbol]: 1 });
  assert.deepEqual(merge.all([{ a: 1 }, { b: 2 }]), { a: 1, b: 2 });
  assert.equal(merge({ count: 1 }, { count: { n: 2 } }, { customMerge: () => () => 3 }).count, 3);
});
