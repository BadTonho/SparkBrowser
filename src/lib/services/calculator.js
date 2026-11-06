const units = new Map();
/** @typedef {{ dimension: string, toBase: (value: number) => number, fromBase: (value: number) => number }} Unit */
/** @type {Map<string, Unit>} */
const typedUnits = units;
/** @param {string[]} names @param {string} dimension @param {(value: number) => number} toBase @param {(value: number) => number} [fromBase] */
function add(names, dimension, toBase, fromBase = value => value) {
  for (const name of names) typedUnits.set(name.toLocaleLowerCase('en-US'), { dimension, toBase, fromBase });
}
/** @param {string[]} names @param {string} dimension @param {number} factor */
const linear = (names, dimension, factor) => add(names, dimension, value => value * factor, value => value / factor);
linear(['mm'], 'length', 0.001); linear(['cm'], 'length', 0.01); linear(['m'], 'length', 1);
linear(['km'], 'length', 1000); linear(['in'], 'length', 0.0254); linear(['ft'], 'length', 0.3048);
linear(['yd'], 'length', 0.9144); linear(['mi'], 'length', 1609.344);
linear(['mg'], 'mass', 0.001); linear(['g'], 'mass', 1); linear(['kg'], 'mass', 1000);
linear(['oz'], 'mass', 28.349523125); linear(['lb'], 'mass', 453.59237);
add(['°c', 'c'], 'temperature', value => value, value => value);
add(['°f', 'f'], 'temperature', value => (value - 32) * 5 / 9, value => value * 9 / 5 + 32);
add(['k'], 'temperature', value => value - 273.15, value => value + 273.15);
linear(['ml'], 'volume', 0.001); linear(['l'], 'volume', 1);
linear(['us fl oz', 'fl oz'], 'volume', 0.0295735295625);
linear(['us cup', 'cup'], 'volume', 0.2365882365); linear(['us gal', 'gal'], 'volume', 3.785411784);

class Parser {
  /** @param {string} source */
  constructor(source) { this.source = source; this.index = 0; }
  /** @returns {void} */
  skip() { while (/\s/.test(this.source[this.index] ?? '')) this.index++; }
  /** @returns {number} */
  expression() {
    let value = this.term();
    while (true) {
      this.skip(); const op = this.source[this.index];
      if (op !== '+' && op !== '-') return value;
      this.index++; const right = this.term(); value = op === '+' ? value + right : value - right;
    }
  }
  /** @returns {number} */
  term() {
    let value = this.factor();
    while (true) {
      this.skip(); const op = this.source[this.index];
      if (op !== '*' && op !== '/') return value;
      this.index++; const right = this.factor();
      if (op === '/' && right === 0) throw new Error('division_by_zero');
      value = op === '*' ? value * right : value / right;
    }
  }
  /** @returns {number} */
  factor() {
    this.skip(); const sign = this.source[this.index];
    if (sign === '+' || sign === '-') { this.index++; const value = this.factor(); return sign === '-' ? -value : value; }
    if (this.source[this.index] === '(') {
      this.index++; const value = this.expression(); this.skip();
      if (this.source[this.index++] !== ')') throw new Error('invalid_expression');
      return value;
    }
    const match = /^(?:\d+(?:\.\d*)?|\.\d+)/.exec(this.source.slice(this.index));
    if (!match) throw new Error('invalid_expression');
    this.index += match[0].length; return Number(match[0]);
  }
  /** @returns {number} */
  parse() { const value = this.expression(); this.skip(); if (this.index !== this.source.length) throw new Error('invalid_expression'); return value; }
}

/** @param {string} value @param {string} locale @returns {number} */
function parseAmount(value, locale) {
  if (locale === 'pt-BR') {
    if (value.includes('.') || (value.match(/,/g) ?? []).length > 1) return NaN;
    value = value.replace(',', '.');
  } else if (value.includes(',')) return NaN;
  return Number(value);
}

/** @param {number} value @param {string} locale */
function format(value, locale) {
  return new Intl.NumberFormat(locale, { maximumSignificantDigits: 12 }).format(value);
}

/** @typedef {'calculator.unknown_unit' | 'calculator.incompatible_units' | 'calculator.out_of_range' | 'calculator.division_by_zero' | 'calculator.invalid_expression'} CalculatorError */
/** @typedef {{ value: number, text: string, error?: never } | { error: CalculatorError, value?: never, text?: never }} Calculation */
/** Returns null for ordinary searches, otherwise a calculator result or localized error key.
 * @param {string} query @param {string} [locale] @returns {Calculation | null}
 */
export function calculateQuery(query, locale = 'en') {
  const text = query.trim();
  if (!text) return null;
  const conversion = /^([+-]?(?:\d+(?:[.,]\d*)?|[.,]\d+))\s+(.+?)\s+(?:to|em)\s+(.+?)$/i.exec(text);
  if (conversion) {
    const value = parseAmount(conversion[1], locale);
    const from = typedUnits.get(conversion[2].trim().toLocaleLowerCase('en-US'));
    const to = typedUnits.get(conversion[3].trim().toLocaleLowerCase('en-US'));
    if (!from || !to) return { error: 'calculator.unknown_unit' };
    if (from.dimension !== to.dimension) return { error: 'calculator.incompatible_units' };
    const result = to.fromBase(from.toBase(value));
    if (!Number.isFinite(result)) return { error: 'calculator.out_of_range' };
    return { value: result, text: `${format(result, locale)} ${conversion[3].trim()}` };
  }
  const decimal = locale === 'pt-BR' ? ',' : '.';
  const expressionPattern = locale === 'pt-BR' ? /^[\d\s()+*/,-]+$/ : /^[\d\s()+*/.-]+$/;
  if (!expressionPattern.test(text) || !/[\d)]/.test(text)) return null;
  const source = decimal === ',' ? text.replaceAll(',', '.') : text;
  try {
    const result = new Parser(source).parse();
    if (!Number.isFinite(result)) return { error: 'calculator.out_of_range' };
    return { value: result, text: format(result, locale) };
  } catch (error) {
    return { error: error instanceof Error && error.message === 'division_by_zero' ? 'calculator.division_by_zero' : 'calculator.invalid_expression' };
  }
}
