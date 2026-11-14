import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';
import assert from 'node:assert/strict';
import { test } from 'node:test';
const source = readFileSync('public/overlays/privacy-popover.js', 'utf8');
function fixture() {
  const listeners = new Map(), elements = new Map(), sent = [];
  const document = { documentElement: {}, activeElement: null,
    getElementById(id) { if (!elements.has(id)) elements.set(id, { style: { setProperty() {} }, offsetWidth: 300, offsetHeight: 200,
      hidden: true, setAttribute() {}, addEventListener: (type, fn) => listeners.set(id + ':' + type, fn),
      contains: target => [...elements.values()].includes(target), focus() { document.activeElement = this; } }); return elements.get(id); },
    addEventListener: (type, fn) => listeners.set('document:' + type, fn) };
  const context = { document, URL, innerWidth: 800, innerHeight: 600, location: { set href(value) { sent.push(new URL(value)); } },
    addEventListener: (type, fn) => listeners.set('window:' + type, fn) };
  context.window = context; runInNewContext(source, context);
  const payload = { visible: true, token: 'fixture', anchor: { left: 500, top: 30, width: 30, height: 30 }, shield: true, autoSkip: false, busy: false,
    label: 'Protection', adblockLabel: 'Adblock', autoSkipLabel: 'Auto skip ads', description: 'YouTube', blocked: '0 blocked', error: '', locale: 'en' };
  return { document, sent, element: id => document.getElementById(id), update: patch => context.__sparkSetPrivacyPopover({ ...payload, ...patch }),
    fire: (target, type, event) => listeners.get(target + ':' + type)?.(event) };
}
test('privacy panel aligns its right edge below the trigger, clamping to the window', () => {
  const f = fixture(); f.update(); const panel = f.element('privacy-popover');
  assert.equal(panel.style.left, '230px'); assert.equal(panel.style.top, '66px');
  assert.equal(f.document.activeElement, f.element('adblock'));
  f.update({ anchor: { left: 900, top: 590, width: 30, height: 30 } });
  assert.equal(panel.style.left, '492px'); assert.equal(panel.style.top, '392px');
});
test('controls send independent desired values without optimistically changing the checkbox', () => {
  const f = fixture(); f.update();
  f.element('auto-skip').checked = true; f.fire('auto-skip', 'change');
  assert.equal(f.element('auto-skip').checked, false);
  assert.equal(f.sent[0].pathname, '/auto-skip'); assert.equal(f.sent[0].searchParams.get('value'), 'true');
  f.fire('auto-skip', 'change'); assert.equal(f.sent.length, 1);
  f.update({ autoSkip: true }); f.element('adblock').checked = false; f.fire('adblock', 'change');
  assert.equal(f.sent[1].pathname, '/shield'); assert.equal(f.sent[1].searchParams.get('value'), 'false');
});
test('Escape closes busy panels, stale updates cannot reopen them, and errors stay visible', () => {
  const f = fixture(); f.update({ busy: true, error: 'fixture error' });
  assert.equal(f.element('privacy-error').hidden, false);
  f.fire('window', 'keydown', { key: 'Escape', preventDefault() {}, stopImmediatePropagation() {} });
  assert.equal(f.sent[0].pathname, '/dismiss'); assert.equal(f.sent[0].searchParams.get('reason'), 'escape');
  f.update(); f.fire('adblock', 'change'); assert.equal(f.sent.length, 1);
  f.update({ visible: false }); assert.equal(f.element('privacy-popover').hidden, true);
});
