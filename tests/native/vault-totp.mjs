import { spawnSync } from 'node:child_process';
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import QRCode from 'qrcode';
import assert from 'node:assert/strict';

/**
 * Optional 2FA codes in the password vault, in the real app: the command is allowed by the ACL, the
 * secret is stored with the entry and never listed, a bad secret is refused, a login without 2FA is
 * unchanged, and Settings shows the code, keeps the secret when a login is edited and clears it on request.
 * Only an artificial secret is used (the RFC 6238 test key).
 */
export async function checkVaultTotp(context) {
  const { invoke, until, script, click, withWebview, request } = context;
  const settings = action => withWebview('/index.html?surface=settings', action);
  const password = 'correct horse battery';
  const secret = 'GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ';
  await invoke('vault_create', { masterPassword: password, useHello: false });
  await invoke('vault_unlock', { password });

  // Backend: stored with the entry, never in the list, computed on request.
  const entry = { title: 'Mail', url: 'https://mail.example.test', username: 'ana', password: 'pw-1', notes: '', totp: secret };
  const saved = await invoke('vault_save_entry', { id: null, entry });
  assert.equal(saved.has_totp, true);
  await invoke('vault_save_entry', { id: null, entry: { title: 'Shop', url: 'https://shop.example.test', username: 'bia', password: 'pw-2', notes: '' } });
  const listed = await invoke('vault_list');
  assert.equal(JSON.stringify(listed).includes(secret), false, 'the list never carries the secret');
  assert.deepEqual(listed.entries.map(item => [item.title, item.has_totp]).sort(), [['Mail', true], ['Shop', false]]);
  const code = await invoke('vault_totp_code', { secret });
  assert.match(code.code, /^\d{6}$/);
  assert.ok(code.seconds_left >= 1 && code.seconds_left <= 30 && code.period === 30);
  assert.deepEqual((await invoke('vault_totp_code', { secret: 'otpauth://totp/x?secret=' + secret + '&digits=8' })).code.length, 8);
  const refusal = async (command, args) => { try { await invoke(command, args); return null; } catch (error) { return String(error?.message ?? error); } };
  assert.match(await refusal('vault_totp_code', { secret: 'not a key!' }) ?? '', /vault\.invalid_totp/);
  assert.match(await refusal('vault_save_entry', { id: null, entry: { ...entry, title: 'Bad', totp: '!!' } }) ?? '', /vault\.invalid_totp/);
  const revealed = await invoke('vault_reveal', { id: saved.id, currentPassword: password });
  assert.equal(revealed.totp, secret);

  // Settings: open the login after confirming who is asking, and read the code.
  await context.focusWindow();
  await click('[data-testid="open-settings"]');
  await until(() => withWebview('/index.html?surface=settings-quick', () => script('return !!document.querySelector("[data-testid=settings-quick-all]")')), 'quick settings');
  await withWebview('/index.html?surface=settings-quick', () => click('[data-testid="settings-quick-all"]'));
  await until(() => settings(() => script('return !!document.querySelector("#settings-search-input")')), 'settings dialog');
  await settings(() => click('[data-testid="settings-passwords"]'));
  await until(() => settings(() => script('return !!document.querySelector("[data-testid=autofill-open-passwords]")')), 'Autofill & Passwords home');
  await settings(() => click('[data-testid="autofill-open-passwords"]'));
  await until(() => settings(() => script('return !!document.querySelector("[data-testid=vault-list]")')), 'password list');
  const open = async title => {
    await until(() => settings(() => script(`return [...document.querySelectorAll('[data-testid="vault-group"]')].some(row=>row.textContent.includes(${JSON.stringify(title)}))`)), `list row for ${title}`);
    await settings(() => script(`const rows=[...document.querySelectorAll('[data-testid="vault-group"]')];rows.find(row=>row.textContent.includes(${JSON.stringify(title)})).click();`));
    await until(() => settings(() => script('return !!document.querySelector("[data-testid=vault-confirm-password]")')), 'identity question');
    await settings(() => script(`const field=document.querySelector('[data-testid="vault-confirm-password"]');field.value=${JSON.stringify(password)};field.dispatchEvent(new Event('input',{bubbles:true}));`));
    await settings(() => click('[data-testid="vault-confirm-submit"]'));
    await until(() => settings(() => script('return !!document.querySelector("[data-testid=vault-detail]")')), 'login page');
  };

  await open('mail.example.test');
  await until(() => settings(() => script('return !!document.querySelector("[data-testid=vault-totp-code]")')), 'the 2FA code is shown');
  await settings(() => context.screenshot('vault-totp-detail'));
  const shown = await settings(() => script('return document.querySelector("[data-testid=vault-totp-code]").textContent.trim()'));
  assert.match(shown, /^\d{3} \d{3}$/);
  const left = await settings(() => script('return document.querySelector(".vault-totp-left").textContent.trim()'));
  assert.match(left, /^\d{1,2}s$/);
  // The code may roll over between reads, so the shown one must match the backend's before or after.
  const after = (await invoke('vault_totp_code', { secret })).code;
  assert.ok([code.code, after].includes(shown.replace(' ', '')), 'the code shown is the backend code');

  // Editing keeps the secret, and removing it from the field removes the 2FA.
  await settings(() => click('[data-testid="vault-edit"]'));
  await until(() => settings(() => script('return !!document.querySelector("[data-testid=vault-field-totp]")')), 'editor');
  assert.equal(await settings(() => script('return document.querySelector("[data-testid=vault-field-totp]").value')), secret, 'the editor starts with the secret');
  await until(() => settings(() => script('return !!document.querySelector("[data-testid=vault-totp]")')), 'the preview shows a code in the editor');
  await settings(() => context.screenshot('vault-totp-editor'));
  await settings(() => script(`const field=document.querySelector('[data-testid="vault-field-title"]');field.value='Mail edited';field.dispatchEvent(new Event('input',{bubbles:true}));`));
  await settings(() => click('[data-testid="vault-save"]'));
  await until(async () => (await invoke('vault_list')).entries.some(item => item.title === 'Mail edited'), 'edit saved');
  assert.equal((await invoke('vault_reveal', { id: saved.id, currentPassword: password })).totp, secret, 'editing other fields keeps the 2FA');

  // A wrong key in the editor shows a hint and a login without 2FA has no code.
  await open('mail.example.test');
  await settings(() => click('[data-testid="vault-edit"]'));
  await until(() => settings(() => script('return !!document.querySelector("[data-testid=vault-field-totp]")')), 'editor again');
  await settings(() => script(`const field=document.querySelector('[data-testid="vault-field-totp"]');field.value='not a key!';field.dispatchEvent(new Event('input',{bubbles:true}));`));
  await until(() => settings(() => script('return !!document.querySelector("[data-testid=vault-totp-invalid]")')), 'the invalid key hint');
  await settings(() => script(`const field=document.querySelector('[data-testid="vault-field-totp"]');field.value='';field.dispatchEvent(new Event('input',{bubbles:true}));`));
  await settings(() => click('[data-testid="vault-save"]'));
  await until(async () => (await invoke('vault_list')).entries.every(item => !item.has_totp), 'clearing the field removes the 2FA');
  await open('shop.example.test');
  assert.equal(await settings(() => script('return !!document.querySelector("[data-testid=vault-totp]")')), false, 'a login without 2FA shows no code');

  // QR code from a picture: the backend opens the native file dialog and reads the picture itself. A
  // QR code that is not a 2FA setup and a file that is not a picture are refused with a message and
  // nothing is filled in; a real 2FA QR code fills in the secret.
  const folder = mkdtempSync(join(tmpdir(), 'spark-qr-'));
  const link = 'otpauth://totp/Shop:bia?secret=' + secret + '&issuer=Shop';
  const good = join(folder, '2fa.png');
  const web = join(folder, 'web.png');
  const text = join(folder, 'notes.png');
  await QRCode.toFile(good, link, { width: 360, margin: 4 });
  await QRCode.toFile(web, 'https://example.test/login?token=abc', { width: 360, margin: 4 });
  writeFileSync(text, 'this is not a picture');
  const field = () => settings(() => script('return document.querySelector("[data-testid=vault-field-totp]").value'));
  const failure = () => settings(() => script('const e=document.querySelector("[data-testid=vault-error]");return e?e.textContent.trim():null'));
  // Clicks the button, then picks the file in the real Windows dialog like a person would.
  const choose = async file => {
    await settings(() => click('[data-testid="vault-totp-qr"]'));
    const picked = spawnSync('powershell', ['-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', resolve('tests/native/file-dialog.ps1'), '-Path', file], { encoding: 'utf8', timeout: 40_000 });
    assert.equal(picked.status, 0, `the file dialog was driven: ${picked.stdout}${picked.stderr}`);
  };
  await settings(() => click('[data-testid="vault-edit"]'));
  await until(() => settings(() => script('return !!document.querySelector("[data-testid=vault-totp-qr]")')), 'editor with the QR button');
  await choose(web);
  const refusedWeb = await until(failure, 'a web address QR is refused');
  assert.match(refusedWeb, /2FA/, 'the message says it is not a 2FA setup');
  assert.equal(await field(), '', 'nothing was filled in');
  await choose(text);
  const refusedFile = await until(async () => { const message = await failure(); return message && message !== refusedWeb ? message : null; }, 'a file that is not a picture is refused');
  assert.match(refusedFile, /(imagem|picture)/i, 'the message says it is not a usable picture');
  assert.equal(await field(), '');
  await choose(good);
  await until(() => settings(() => script('return !!document.querySelector("[data-testid=vault-totp-scanned]")')), 'the QR code was read');
  assert.equal(await field(), link, 'the secret came from the QR code');
  await until(() => settings(() => script('return !!document.querySelector("[data-testid=vault-totp-code]")')), 'the preview shows the code of the scanned secret');
  await settings(() => context.screenshot('vault-totp-qr'));
  await settings(() => click('[data-testid="vault-save"]'));
  await until(async () => (await invoke('vault_list')).entries.find(item => item.title === 'Shop')?.has_totp === true, 'the scanned secret is saved');
  const stored = (await invoke('vault_list')).entries.find(item => item.title === 'Shop');
  assert.equal((await invoke('vault_reveal', { id: stored.id, currentPassword: password })).totp, link);
  rmSync(folder, { recursive: true, force: true });
}
