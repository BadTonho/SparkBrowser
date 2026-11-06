import test from 'node:test';
import assert from 'node:assert/strict';
import { calculateQuery } from '../src/lib/services/calculator.js';

test('arithmetic honors precedence, parentheses and unary signs', () => {
  assert.equal(calculateQuery('2 + 3 * 4').value, 14);
  assert.equal(calculateQuery('(2 + 3) * 4').value, 20);
  assert.equal(calculateQuery('-(2 + 3) / +5').value, -1);
});

test('decimal parsing and number formatting follow the interface language', () => {
  assert.equal(calculateQuery('1,5 + 2,25', 'pt-BR').value, 3.75);
  assert.equal(calculateQuery('1.5 + 2.25', 'en').value, 3.75);
  assert.match(calculateQuery('1,5 + 2,25', 'pt-BR').text, /3,75/);
});

test('converts supported units within their dimensions', () => {
  const conversions = [
    ['10 mm to cm', 1], ['10 cm em m', 0.1], ['1 km to mi', 0.621371192237],
    ['1 in to ft', 1 / 12], ['1 yd to m', 0.9144],
    ['1000 mg to g', 1], ['1 kg to g', 1000], ['1 oz to g', 28.349523125], ['1 lb to kg', 0.45359237],
    ['0 °C to °F', 32], ['32 °F to °C', 0], ['273.15 K to °C', 0],
    ['1000 mL to L', 1], ['1 L to US fl oz', 33.8140227], ['1 US cup to mL', 236.5882365], ['1 US gal to L', 3.785411784],
  ];
  for (const [query, expected] of conversions) assert.ok(Math.abs(calculateQuery(query).value - expected) < 1e-8, query);
});

test('reports conversion, arithmetic and numeric errors while ignoring ordinary searches', () => {
  assert.equal(calculateQuery('1 km to kg').error, 'calculator.incompatible_units');
  assert.equal(calculateQuery('5 xyz to mi').error, 'calculator.unknown_unit');
  assert.equal(calculateQuery('1 / 0').error, 'calculator.division_by_zero');
  const enormous = '9'.repeat(400);
  assert.equal(calculateQuery(`${enormous} * ${enormous}`).error, 'calculator.out_of_range');
  assert.equal(calculateQuery('weather tomorrow'), null);
  assert.equal(calculateQuery('15% + 2'), null);
});
