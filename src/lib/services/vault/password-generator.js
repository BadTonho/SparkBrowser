const LOWER = 'abcdefghijkmnopqrstuvwxyz';
const UPPER = 'ABCDEFGHJKLMNPQRSTUVWXYZ';
const DIGITS = '23456789';
const SYMBOLS = '!@#$%^&*-_=+?';
export const GENERATED_LENGTH = 20;

/**
 * An unbiased random index below `limit`: values that would skew the result are thrown away.
 * @param {number} limit
 * @param {(array: Uint32Array) => Uint32Array} fill
 */
function below(limit, fill) {
  const range = 0x100000000 - (0x100000000 % limit);
  const buffer = new Uint32Array(1);
  for (;;) {
    fill(buffer);
    if (buffer[0] < range) return buffer[0] % limit;
  }
}

/**
 * A random password with at least one lowercase letter, uppercase letter, digit and symbol.
 * Look-alike characters (0/O, 1/l/I) are left out so it can be read and typed by hand.
 * @param {number} [length]
 * @param {(array: Uint32Array) => Uint32Array} [fill] The random source; the browser's by default.
 */
export function generatePassword(length = GENERATED_LENGTH, fill = array => crypto.getRandomValues(array)) {
  const groups = [LOWER, UPPER, DIGITS, SYMBOLS];
  const size = Math.min(128, Math.max(groups.length, Math.floor(length)));
  const all = groups.join('');
  const picked = groups.map(group => group[below(group.length, fill)]);
  while (picked.length < size) picked.push(all[below(all.length, fill)]);
  // Fisher-Yates, so the guaranteed characters do not always come first.
  for (let i = picked.length - 1; i > 0; i--) {
    const j = below(i + 1, fill);
    [picked[i], picked[j]] = [picked[j], picked[i]];
  }
  return picked.join('');
}
