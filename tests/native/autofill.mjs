import assert from 'node:assert/strict';

/**
 * Autofill in a real page: a saved login, card and contact are offered under the right fields,
 * filled in after a real click, refused when the vault is locked or autofill is off, and a login
 * saved for another site is never offered. Only artificial test values are used.
 */
export async function checkAutofill(context) {
  const { invoke, until, fixture, withWebview, script, request, wait } = context;
  const password = 'correct horse battery';
  await invoke('vault_create', { masterPassword: password, useHello: false });
  await invoke('vault_unlock', { password });
  await invoke('vault_save_entry', { id: null, entry: { title: 'Fixture login', url: fixture.origin, username: 'ana@example.test', password: 'secret-pass-1', notes: '' } });
  await invoke('vault_save_entry', { id: null, entry: { title: 'Other site', url: 'https://other.example.org', username: 'other-user', password: 'other-pass', notes: '' } });
  await invoke('vault_record_save', { id: null, record: { kind: 'card', nickname: 'Test card', holder: 'Ana Souza', number: '4242 4242 4242 4242', exp_month: 7, exp_year: 2031 } });
  await invoke('vault_record_save', { id: null, record: { kind: 'contact', name: 'Ana Souza', email: 'ana@example.test', phone: '+55 81 99999-0000', city: 'Recife' } });

  const path = '/autofill-probe';
  await invoke('create_tab', { url: fixture.origin + path, searchEngine: 'google' });
  const page = code => withWebview(path, () => script(code));
  await until(() => page('return Boolean(document.querySelector("#user"))'), 'autofill probe page ready');

  const reference = element => element['element-6066-11e4-a52e-4f735466cecf'];
  const click = selector => withWebview(path, async () => {
    const element = await request('POST', `/session/${context.sessionId()}/element`, { using: 'css selector', value: selector });
    await request('POST', `/session/${context.sessionId()}/element/${reference(element)}/click`, {});
  });
  const find = 'const host=[...document.documentElement.children].find(e=>e.shadowRoot&&e.shadowRoot.querySelector(".list"));';
  const listed = () => page(`${find}return host?[...host.shadowRoot.querySelectorAll("button,p")].map(e=>e.textContent):null;`);
  const pick = index => withWebview(path, async () => {
    const element = await script(`${find}return host.shadowRoot.querySelectorAll("button")[arguments[0]]`, index);
    await request('POST', `/session/${context.sessionId()}/element/${reference(element)}/click`, {});
  });
  const value = selector => page(`return document.querySelector(${JSON.stringify(selector)}).value`);

  // A login for this site is offered under the username field; the one for another site is not.
  await click('#user');
  const logins = await until(listed, 'login suggestion shown');
  assert.equal(logins.length, 1, 'only the login of this site is offered');
  assert.match(logins[0], /ana@example\.test/);
  assert.ok(!logins.join(' ').includes('secret-pass'), 'the list never carries the password');
  await pick(0);
  await until(async () => await value('#pass') === 'secret-pass-1', 'login filled in');
  assert.equal(await value('#user'), 'ana@example.test');

  // A card: the list shows only the last four digits, and the fill uses the form of each field.
  await click('#ccnum');
  const cards = await until(listed, 'card suggestion shown');
  assert.match(cards[0], /4242/);
  assert.ok(!cards.join(' ').includes('4242 4242'), 'the list never carries the whole number');
  await pick(0);
  await until(async () => await value('#ccnum') === '4242424242424242', 'card number filled in');
  assert.equal(await value('#ccname'), 'Ana Souza');
  assert.equal(await value('#ccmonth'), '07');
  assert.equal(await value('#ccyear'), '2031');

  // Contact details.
  await click('#cmail');
  await until(listed, 'contact suggestion shown');
  await pick(0);
  await until(async () => await value('#cname') === 'Ana Souza', 'contact filled in');
  assert.equal(await value('#cphone'), '+55 81 99999-0000');
  assert.equal(await value('#ccity'), 'Recife');

  // A search box is not a form field autofill touches.
  await click('#search');
  await wait(600);
  assert.equal(await listed(), null, 'no suggestions under a search box');

  // A locked vault offers nothing to fill, only a note.
  await invoke('vault_lock');
  await click('#user');
  const locked = await until(listed, 'locked note shown');
  assert.ok(locked.length >= 1);
  assert.ok(!locked.some(text => /secret-pass|4242/.test(text)));

  // Escape closes the list (it would otherwise cover the next field).
  await withWebview(path, () => request('POST', `/session/${context.sessionId()}/actions`, { actions: [{ type: 'key', id: 'keyboard', actions: [{ type: 'keyDown', value: '' }, { type: 'keyUp', value: '' }] }] }));
  await until(async () => await listed() === null, 'Escape closes the list');

  // Turned off, nothing is offered even when the vault is open.
  await invoke('vault_unlock', { password });
  await invoke('set_autofill_settings', { enabled: false, inPrivate: false });
  await click('#cmail');
  await wait(800);
  assert.equal(await listed(), null, 'autofill off: no suggestions');
  await invoke('set_autofill_settings', { enabled: true, inPrivate: false });
}
