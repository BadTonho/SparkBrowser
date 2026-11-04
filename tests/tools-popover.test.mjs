import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';
import ts from 'typescript';

function load(path, imports = {}, globals = {}) {
  const exports = {};
  runInNewContext(ts.transpileModule(readFileSync(path, 'utf8'), { compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 } }).outputText,
    { exports, require: name => { if (name === './download-visual') return load('src/lib/services/download-visual.ts', {}, { atob }); if (!(name in imports)) throw Error(name); return imports[name]; }, ...globals });
  return exports;
}
function fixture(tick = async () => {}) {
  const writable = value => ({ set: next => { value = next; }, subscribe: fn => { fn(value); return () => {}; } });
  const get = store => { let result; store.subscribe(value => result = value)(); return result; };
  const anchor = () => ({ isConnected: true, visible: true, focused: 0, getClientRects() { return this.visible ? [1] : []; }, focus() { this.focused++; } });
  const tools = anchor(), menu = anchor(), calls = [], notices = [], zen = writable(true);
  const api = load('src/lib/stores/toolsStore.ts', {
    'svelte/store': { get, writable }, svelte: { tick },
    '../services/bridge/overlays': { notifyOpening: id => notices.push(id) },
    '../services/bridge/tools-popover': { setToolsPopover: async (...args) => calls.push(args) }, './settingsStore': { isZenMode: zen },
  }, { document: { querySelector: selector => selector.includes('titlebar-actions') ? tools : menu } });
  return { ...api, get, tools, menu, calls, notices, zen };
}

test('tools restore the toolbar, resolve the visible anchor and preserve an existing session when changing section', async () => {
  const f = fixture(); await f.openTools();
  assert.equal(f.get(f.zen), false); assert.equal(f.get(f.toolsSession).anchor, f.tools);
  const original = f.get(f.toolsSession);
  await f.openTools('notes');
  assert.equal(f.get(f.toolsSession), original); assert.equal(f.get(f.toolsSection), 'notes');
  assert.deepEqual(f.notices, ['tools']);
  await f.openTools('sessions', f.tools);
  assert.equal(f.get(f.isToolsOpen), false); assert.equal(f.tools.focused, 1);
});

test('compact mode anchors to More and a dismissed pending opening cannot reopen', async () => {
  const f = fixture(); f.tools.visible = false; await f.openTools();
  assert.equal(f.get(f.toolsSession).anchor, f.menu);
  await f.closeTools(true); assert.equal(f.menu.focused, 1);
  let resolve; const pending = fixture(() => new Promise(r => resolve = r));
  const opening = pending.openTools(); await pending.closeTools(); resolve(); await opening;
  assert.equal(pending.get(pending.isToolsOpen), false); assert.equal(pending.calls.length, 0);
});

test('panel remains below its anchor and within horizontal window margins', () => {
  const { panelPosition } = load('src/lib/services/panel-presentation.ts');
  for (const width of [400, 680, 1280, 1920]) for (const left of [10, width / 2, width - 40]) {
    const pos = panelPosition({ left, top: 40, width: 27, height: 26 }, width, 480);
    assert.equal(pos.top, 74); assert.ok(pos.left >= 8); assert.ok(pos.left + pos.width <= width - 8);
    assert.equal(pos.width, Math.min(640, width - 16)); assert.equal(pos.maxHeight, 398);
  }
});

test('quick settings menu uses a compact width and clamps below the toolbar anchor', () => {
  const { panelPosition } = load('src/lib/services/panel-presentation.ts');
  for (const width of [280, 360, 900, 1280]) for (const left of [8, width / 2, width - 40]) {
    const pos = panelPosition({ left, top: 24, width: 28, height: 28 }, width, 700, { width: 300, maxHeight: 360 });
    assert.equal(pos.top, 60);
    assert.ok(pos.left >= 8);
    assert.ok(pos.left + pos.width <= width - 8);
    assert.equal(pos.width, Math.min(300, width - 16));
    assert.equal(pos.maxHeight, 360);
  }
});

test('presentation excludes executable markup and opaque controls reject removed or disabled targets', () => {
  class Element {
    constructor(tag, attrs = {}, children = []) { this.localName = tag; this.attributes = Object.entries(attrs).map(([name, value]) => ({ name, value })); this.childNodes = children; this.isConnected = true; this.clicks = 0; }
    contains(node) { return this === node || this.childNodes.some(child => child.contains?.(node)); }
    closest() { return this.attributes.some(a => a.name === 'disabled') ? this : null; }
    matches() { return this.localName === 'button'; }
    click() { this.clicks++; }
  }
  class Input extends Element {} class Textarea extends Element {} class Select extends Element {} class Form extends Element {}
  const button = new Element('button', { 'data-testid': 'safe-button', onclick: 'unsafe()', style: 'unsafe' });
  const script = new Element('script');
  const progress = new Element('progress', { value: '25', max: '100', style: 'unsafe' });
  const root = new Element('div', {}, [button, script, { nodeType: 3, textContent: '<img src=x onerror=unsafe()>' }, progress]);
  const { createPanelProjection } = load('src/lib/services/panel-presentation.ts', {}, {
    Node: { TEXT_NODE: 3 }, Element, HTMLElement: Element, HTMLInputElement: Input, HTMLTextAreaElement: Textarea, HTMLSelectElement: Select, HTMLFormElement: Form,
  });
  const f = createPanelProjection(root), tree = f.snapshot(), control = tree.children[0];
  assert.equal(tree.children.length, 3); assert.equal(tree.children[1].text, '<img src=x onerror=unsafe()>');
  assert.equal(tree.children[2].tag, 'progress'); assert.equal(tree.children[2].attributes.value, '25'); assert.equal(tree.children[2].attributes.max, '100'); assert.equal(tree.children[2].attributes.style, undefined);
  assert.equal(control.attributes.onclick, undefined); assert.equal(control.attributes.style, undefined);
  f.dispatch({ kind: 'click', target: control.id }); assert.equal(button.clicks, 1);
  button.attributes.push({ name: 'disabled', value: '' });
  f.dispatch({ kind: 'click', target: control.id }); assert.equal(button.clicks, 1);
  button.attributes = []; button.isConnected = false;
  f.dispatch({ kind: 'click', target: control.id }); f.dispatch({ kind: 'click', target: 999999 }); assert.equal(button.clicks, 1);
});

test('image projection accepts only small local PNG data, enforces the total budget and resets it per snapshot', () => {
  class Element {
    constructor(tag, attrs = {}, children = []) { this.localName=tag; this.attributes=Object.entries(attrs).map(([name,value])=>({name,value})); this.childNodes=children; }
  }
  class Input extends Element {} class Textarea extends Element {} class Select extends Element {}
  const bytes=Buffer.alloc(45); Buffer.from([137,80,78,71,13,10,26,10]).copy(bytes); bytes.writeUInt32BE(13,8); bytes.write('IHDR',12); bytes.writeUInt32BE(96,16); bytes.writeUInt32BE(96,20);
  const valid='data:image/png;base64,'+bytes.toString('base64');
  const oversized=Buffer.from(bytes); oversized.writeUInt32BE(4096,16);
  const bad=['https://external.test/a.png','file:///C:/private/a.png','data:image/svg+xml,<svg/>','data:image/png;base64,'+oversized.toString('base64')];
  const root=new Element('div',{},[new Element('img',{src:valid,onerror:'unsafe()',style:'position:fixed'}), ...bad.map(src=>new Element('img',{src})),new Element('button',{src:valid})]);
  const {createPanelProjection}=load('src/lib/services/panel-presentation.ts',{}, {Element,HTMLElement:Element,HTMLInputElement:Input,HTMLTextAreaElement:Textarea,HTMLSelectElement:Select,Node:{TEXT_NODE:3}});
  const projection=createPanelProjection(root), tree=projection.snapshot();
  assert.equal(tree.children.length,2); assert.equal(tree.children[0].attributes.src,valid); assert.equal(tree.children[0].attributes.onerror,undefined); assert.equal(tree.children[0].attributes.style,undefined); assert.equal(tree.children[1].attributes.src,undefined);
  root.childNodes=Array.from({length:8000},()=>new Element('img',{src:valid}));
  const first=projection.snapshot(); assert.ok(first.children.length<8000); assert.ok(first.children.length*valid.length<=512*1024);
  assert.equal(projection.snapshot().children.length,first.children.length);
});
