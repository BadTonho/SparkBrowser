import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';
import assert from 'node:assert/strict';
import { test } from 'node:test';

const source = readFileSync(new URL('../public/overlays/media-popover.js', import.meta.url), 'utf8');

function fixture() {
  const listeners = new Map();
  const makeElement = id => ({
    id, style: { setProperty(name, value) { this[name] = value; } }, attributes: {},
    offsetWidth: 200, offsetHeight: 64, hidden: true, value: '100', disabled: false,
    setAttribute(name, value) { this.attributes[name] = value; },
    addEventListener(type, handler) { listeners.set(`${id}:${type}`, handler); },
    contains(target) { return target === this || target === slider || target === percentage; },
    focus() { document.activeElement = this; },
  });
  const popover = makeElement('media-popover'), slider = makeElement('page-volume'), percentage = makeElement('volume-percentage');
  const document = { documentElement: {}, getElementById: id => [popover, slider, percentage].find(element => element.id === id),
    addEventListener: (type, handler) => listeners.set(`document:${type}`, handler) };
  const sent = [], frames = new Map();
  let nextFrame = 0;
  const context = { document, innerWidth: 800, innerHeight: 600, URL,
    requestAnimationFrame: callback => { frames.set(++nextFrame, callback); return nextFrame; },
    cancelAnimationFrame: id => frames.delete(id),
    addEventListener: (type, handler) => listeners.set(`window:${type}`, handler),
    location: { set href(value) { sent.push(new URL(value)); } },
  };
  context.window = context;
  runInNewContext(source, context);
  const payload = { visible: true, token: 'artificial-session', anchor: { left: 400, top: 30, width: 25, height: 25 },
    volume: 100, disabled: false, locale: 'pt-BR', label: 'Volume', theme: { accent: '#eee' } };
  const update = patch => context.__veloceSetMediaPopover({ ...payload, ...patch });
  const fire = (target, type, event = {}) => listeners.get(`${target}:${type}`)?.(event);
  const frame = () => { const callbacks = [...frames.values()]; frames.clear(); for (const callback of callbacks) callback(); };
  return { popover, slider, percentage, document, sent, update, fire, frame, context };
}

test('popover anchors six pixels below the button and clamps to the window', () => {
  const f = fixture(); f.update();
  assert.equal(f.popover.style.top, '61px');
  assert.equal(f.popover.style.left, '312.5px');
  assert.equal(f.document.activeElement, f.slider);
  assert.equal(f.document.documentElement.lang, 'pt-BR');
  assert.equal(f.slider.attributes['aria-label'], 'Volume');
  assert.equal(f.slider.attributes['aria-valuetext'], '100%');
  assert.equal(f.percentage.textContent, '100%');
  f.update({ anchor: { left: 790, top: 590, width: 25, height: 25 } });
  assert.equal(f.popover.style.left, '592px');
  assert.equal(f.popover.style.top, '528px');
  f.update({ anchor: { left: -50, top: -90, width: 25, height: 25 } });
  assert.equal(f.popover.style.left, '8px');
  assert.equal(f.popover.style.top, '8px');
});

test('rapid input coalesces and dismissal carries the final value before the next frame', () => {
  const f = fixture(); f.update();
  for (const value of ['0', '100', '200']) {
    f.slider.value = value; f.fire('page-volume', 'input');
    assert.equal(f.percentage.textContent, `${value}%`, 'percentage updates before the queued native write');
  }
  assert.equal(f.sent.length, 0);
  f.frame();
  assert.equal(f.sent.length, 1);
  assert.equal(f.sent[0].pathname, '/volume');
  assert.equal(f.sent[0].searchParams.get('value'), '200');
  f.slider.value = '37'; f.fire('page-volume', 'input');
  assert.equal(f.percentage.textContent, '37%');
  f.fire('window', 'keydown', { key: 'Escape', preventDefault() {}, stopImmediatePropagation() {} });
  assert.equal(f.sent.length, 2);
  assert.equal(f.sent[1].pathname, '/dismiss');
  assert.equal(f.sent[1].searchParams.get('value'), '37');
  assert.equal(f.sent[1].searchParams.get('reason'), 'escape');
  assert.equal(f.sent[1].searchParams.get('sequence'), '2');
  f.update({ volume: 100 }); f.frame();
  assert.equal(f.sent.length, 2, 'delayed updates cannot reopen a closing session');
});

test('state refresh preserves a drag and reopening resets the session and focus', () => {
  const f = fixture(); f.update();
  f.fire('page-volume', 'pointerdown');
  f.slider.value = '48';
  f.fire('page-volume', 'input');
  f.update({ volume: 100 });
  assert.equal(f.slider.value, '48');
  assert.equal(f.percentage.textContent, '48%');
  f.fire('window', 'pointerup');
  assert.equal(f.sent[0].searchParams.get('value'), '48');
  f.update({ visible: false });
  assert.equal(f.popover.hidden, true);
  f.update({ token: 'artificial-next', volume: 80 });
  assert.equal(f.slider.value, '80');
  assert.equal(f.percentage.textContent, '80%');
  f.slider.value = '81'; f.fire('page-volume', 'change');
  assert.equal(f.sent[1].searchParams.get('token'), 'artificial-next');
  assert.equal(f.sent[1].searchParams.get('sequence'), '1');
  assert.equal(f.percentage.textContent, '81%');
  f.update({ volume: 100 });
  assert.equal(f.percentage.textContent, '100%', 'applied player volume replaces an unsupported request');
});

test('outside and trigger clicks close, while slider clicks stay open; disabled control sends no volume', () => {
  const f = fixture(); f.update({ disabled: true });
  assert.equal(f.document.activeElement, f.popover);
  assert.equal(f.percentage.textContent, '100%');
  f.fire('document', 'pointerdown', { target: f.slider });
  f.fire('document', 'pointerdown', { target: f.percentage });
  assert.equal(f.sent.length, 0);
  f.fire('document', 'pointerdown', { target: {}, clientX: 410, clientY: 40 });
  assert.equal(f.sent[0].searchParams.get('reason'), 'trigger');
  assert.equal(f.sent[0].searchParams.has('value'), false);
  f.update({ token: 'artificial-next' });
  f.fire('document', 'pointerdown', { target: {}, clientX: 10, clientY: 100 });
  assert.equal(f.sent[1].searchParams.get('reason'), 'outside');
});
