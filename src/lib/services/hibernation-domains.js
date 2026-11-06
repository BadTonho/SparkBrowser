/** @param {string} raw */
export function normalizeHibernationDomain(raw) {
  const value = String(raw ?? "").trim().replace(/\.+$/, "");
  if (!value || /[\s/:?#@]/u.test(value)) return null;
  try {
    const host = new URL(`http://${value}/`).hostname.toLowerCase().replace(/\.+$/, "");
    return host && !host.includes("/") ? host : null;
  } catch {
    return null;
  }
}

/** @param {string} host @param {readonly string[]} domains */
export function hibernationDomainMatches(host, domains) {
  const normalized = normalizeHibernationDomain(host);
  return !!normalized && domains.some(domain => normalized === domain || normalized.endsWith(`.${domain}`));
}

/** @param {string} raw @param {readonly string[]} current */
export function addHibernationDomain(raw, current) {
  const domain = normalizeHibernationDomain(raw);
  if (!domain || current.includes(domain)) return null;
  return [...current, domain].sort((a, b) => a.localeCompare(b));
}
