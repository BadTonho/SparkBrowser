import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';

const roles = runInNewContext(readFileSync('src-tauri/src/autofill/roles.js', 'utf8'), {});
const role = (text, extra = {}) => roles.roleFor({ type: 'text', autocomplete: '', text, ...extra });

test('the autocomplete attribute decides first', () => {
  assert.equal(role('anything', { autocomplete: 'cc-number' }), 'cc-number');
  assert.equal(role('foo', { autocomplete: 'section-x shipping postal-code' }), 'postal-code');
  assert.equal(role('card number', { autocomplete: 'off' }), 'cc-number');
  assert.equal(role('x', { type: 'password', autocomplete: 'new-password' }), 'password');
});

test('card fields are recognised by name, in English and Portuguese', () => {
  assert.equal(role('cardNumber'), 'cc-number');
  assert.equal(role('Número do cartão'), 'cc-number');
  assert.equal(role('card_holder'), 'cc-name');
  assert.equal(role('Nome no cartão'), 'cc-name');
  assert.equal(role('exp-month'), 'cc-exp-month');
  assert.equal(role('expiry-year'), 'cc-exp-year');
  assert.equal(role('Validade MM/AA'), 'cc-exp');
  assert.equal(role('expiry'), 'cc-exp');
});

test('contact fields are recognised, and the card name wins over the plain name', () => {
  assert.equal(role('first_name'), 'given-name');
  assert.equal(role('Sobrenome'), 'family-name');
  assert.equal(role('Nome completo'), 'name');
  assert.equal(role('zip'), 'postal-code');
  assert.equal(role('CEP'), 'postal-code');
  assert.equal(role('address line 2'), 'address-line2');
  assert.equal(role('Endereço'), 'address-line1');
  assert.equal(role('city'), 'address-level2');
  assert.equal(role('Estado'), 'address-level1');
  assert.equal(role('country'), 'country-name');
  assert.equal(role('company'), 'organization');
  assert.equal(role('', { type: 'email' }), 'email');
  assert.equal(role('', { type: 'tel' }), 'tel');
});

test('search boxes, buttons and unknown fields are left alone', () => {
  assert.equal(role('q'), null);
  assert.equal(role('site search'), null);
  assert.equal(role('whatever'), null);
  assert.equal(roles.roleFor({ type: 'checkbox', text: 'password' }), null);
  assert.equal(roles.roleFor({ type: 'hidden', text: 'email' }), null);
  assert.equal(roles.roleFor({ type: 'search', text: 'email' }), null);
});

test('a username only counts inside a login form, and an email field joins it there', () => {
  assert.equal(roles.kindOf('username', false), null);
  assert.equal(roles.kindOf('username', true), 'login');
  assert.equal(roles.kindOf('email', true), 'login');
  assert.equal(roles.kindOf('email', false), 'contact');
  assert.equal(roles.kindOf('password', false), 'login');
  assert.equal(roles.kindOf('cc-number', false), 'card');
  assert.equal(roles.kindOf('postal-code', false), 'contact');
});

test('the card expiry takes the shape the field asks for', () => {
  assert.equal(roles.expiry(7, 2031, { placeholder: 'MM/YY', maxLength: 5 }), '07/31');
  assert.equal(roles.expiry('07', '2031', { placeholder: 'MM / YYYY', maxLength: -1 }), '07 / 2031');
  assert.equal(roles.expiry(12, 2030, { placeholder: '', maxLength: 7 }), '12/2030');
  assert.equal(roles.expiry(1, 2030, { placeholder: '', maxLength: -1 }), '01/30');
});
