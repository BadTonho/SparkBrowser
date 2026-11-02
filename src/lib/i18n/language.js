/** @param {string} preference @param {readonly string[]} languages */
export function resolveLanguage(preference, languages) {
  if (preference === 'en' || preference === 'pt-BR') return preference;
  return languages[0]?.toLowerCase().startsWith('pt') ? 'pt-BR' : 'en';
}
/** @param {string} template @param {Record<string, unknown>} values */
export function interpolate(template, values = {}) {
  return template.replace(/\{(\w+)\}/g, (match, key) => Object.hasOwn(values, key) ? String(values[key]) : match);
}
