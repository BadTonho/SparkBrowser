// Local, advisory heuristic; never blocks a page or claims it is malicious.
const BUILT_IN = ['google.com', 'youtube.com', 'github.com', 'microsoft.com', 'paypal.com', 'amazon.com', 'facebook.com', 'instagram.com', 'apple.com', 'wikipedia.org'];
/** @param {string} raw */
function host(raw) {
  try { const url = new URL(raw); return ['http:', 'https:'].includes(url.protocol) ? url.hostname.toLowerCase().replace(/\.$/, '') : ''; }
  catch { return ''; }
}
/** @param {string} left @param {string} right */
function near(left, right) {
  if (Math.abs(left.length - right.length) > 1) return false;
  let i = 0, j = 0, changes = 0;
  while (i < left.length && j < right.length) {
    if (left[i] === right[j]) { i++; j++; continue; }
    if (++changes > 1) return false;
    if (left.length > right.length) i++;
    else if (right.length > left.length) j++;
    else { i++; j++; }
  }
  return changes + (i < left.length || j < right.length ? 1 : 0) === 1;
}
/** @param {string} raw @param {string[]} bookmarks */
export function suspiciousHost(raw, bookmarks) {
  const actual = host(raw);
  if (!actual) return null;
  const targets = [...new Set([...BUILT_IN, ...bookmarks.map(host).filter(Boolean)])];
  if (targets.some(target => actual === target || actual.endsWith(`.${target}`))) return null;
  const target = targets.find(candidate => {
    const a = actual.split('.'), b = candidate.split('.');
    return a.length === b.length && a.length >= 2 && a.at(-1) === b.at(-1) && a.slice(1).join('.') === b.slice(1).join('.') && a[0].length >= 5 && near(a[0], b[0]);
  });
  if (target) return { host: actual, target };
  if (actual.split('.').some(label => label.startsWith('xn--'))) return { host: actual, target: '' };
  return null;
}
