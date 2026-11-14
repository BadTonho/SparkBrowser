import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';
import { test } from 'node:test';
import assert from 'node:assert/strict';
const source = readFileSync('src-tauri/src/privacy/auto_skip.js', 'utf8').replace('__AUTO_SKIP_TOKEN__', '"fixture-token"');

function fixture(url = 'https://www.youtube.com/watch?v=fixture') {
  const events = new Map(), timers = new Map(); let now = 0, next = 0, observe, disconnected = 0;
  const button = () => ({ clicks: 0, disabled: false, hidden: false, style: {},
    getAttribute: () => null, closest() { return this.hidden ? {} : null; },
    getClientRects() { return this.hidden ? [] : [{}]; }, click() { this.clicks++; } });
  let buttons = [button()], ad = true, player = true;
  const context = { URL, location: new URL(url), performance: { now: () => now },
    WeakSet, getComputedStyle: element => ({ display: 'block', visibility: 'visible', opacity: '1', ...element.style }),
    setTimeout: (callback, delay) => { timers.set(++next, { callback, due: now + delay }); return next; },
    clearTimeout: id => timers.delete(id),
    addEventListener: (name, callback) => events.set(name, callback), removeEventListener: name => events.delete(name),
    MutationObserver: class { constructor(callback) { observe = callback; } observe() {} disconnect() { disconnected++; } },
    document: { querySelector: () => player ? { classList: { contains: () => ad }, querySelectorAll: () => buttons } : null,
      addEventListener: (name, callback) => events.set(name, callback), removeEventListener: name => events.delete(name) },
  };
  context.window = context; context.top = context;
  runInNewContext(source, context);
  return { context, button, buttons: () => buttons, setButtons: value => { buttons = value; },
    setAd: value => { ad = value; }, setPlayer: value => { player = value; },
    enable: value => context.__sparkSetAutoSkip('fixture-token', value),
    mutate: () => observe?.(), event: name => events.get(name)?.(), timers, disconnected: () => disconnected,
    advance(ms = 500) { now += ms; for (const [id, timer] of [...timers]) if (timer.due <= now) { timers.delete(id); timer.callback(); } },
  };
}
test('auto skip starts inert, requires token, ad state and a usable player control', () => {
  const f = fixture(), b = f.buttons()[0];
  f.advance(); assert.equal(b.clicks, 0);
  f.context.__sparkSetAutoSkip('wrong', true); f.advance(); assert.equal(b.clicks, 0);
  f.setAd(false); f.enable(true); f.advance(); assert.equal(b.clicks, 0);
  f.setAd(true); b.disabled = true; f.mutate(); f.advance(); assert.equal(b.clicks, 0);
  b.disabled = false; b.hidden = true; f.mutate(); f.advance(); assert.equal(b.clicks, 0);
  b.hidden = false; b.style.opacity = '0'; f.mutate(); f.advance(); assert.equal(b.clicks, 0);
  b.style.opacity = '1'; f.mutate(); f.advance(); assert.equal(b.clicks, 1);
});
test('groups mutations at 500ms, never repeats a button and accepts its replacement', () => {
  const f = fixture(); f.enable(true); f.advance(0); assert.equal(f.buttons()[0].clicks, 1);
  const replacement = f.button(); f.setButtons([replacement]);
  for (let n = 0; n < 20; n++) f.mutate();
  assert.equal(f.timers.size, 1); f.advance(499); assert.equal(replacement.clicks, 0);
  f.advance(1); assert.equal(replacement.clicks, 1);
  f.mutate(); f.advance(); assert.equal(replacement.clicks, 1);
  f.setAd(false); f.mutate(); f.advance();
  f.setAd(true); f.mutate(); f.advance(); assert.equal(replacement.clicks, 2, 'a later ad may reuse its control');
  f.enable(false); assert.equal(f.timers.size, 0); assert.ok(f.disconnected() > 0);
  f.setButtons([f.button()]); f.mutate(); f.advance(); assert.equal(f.buttons()[0].clicks, 0);
});
test('host/path/frame boundaries and SPA transitions keep ordinary content untouched', () => {
  for (const url of ['https://youtube.com.example.test/watch', 'https://notyoutube.com/watch', 'http://youtube.com/watch', 'https://m.youtube.com/watch', 'https://youtube.com/embed/a', 'https://youtube.com/shorts/a']) {
    const f = fixture(url); f.enable(true); f.advance(); assert.equal(f.buttons()[0].clicks, 0, url);
  }
  const f = fixture('https://www.youtube.com/'); f.enable(true); f.advance(); assert.equal(f.buttons()[0].clicks, 0);
  f.context.location = new URL('https://www.youtube.com/watch?v=a'); f.event('yt-navigate-finish'); f.advance(); assert.equal(f.buttons()[0].clicks, 1);
  f.setButtons([f.button()]); f.context.top = {}; f.mutate(); f.advance(); assert.equal(f.buttons()[0].clicks, 0);
  f.event('pagehide'); assert.equal(f.timers.size, 0);
});
