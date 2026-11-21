(function () {
  // What a form field is for, judged from its autocomplete attribute first and from its name, id,
  // placeholder and label otherwise. Pure functions, so they are tested without a browser.
  const AUTOCOMPLETE = {
    'username': 'username', 'email': 'email', 'current-password': 'password', 'new-password': 'password',
    'cc-number': 'cc-number', 'cc-name': 'cc-name', 'cc-exp': 'cc-exp', 'cc-exp-month': 'cc-exp-month', 'cc-exp-year': 'cc-exp-year',
    'name': 'name', 'given-name': 'given-name', 'family-name': 'family-name', 'organization': 'organization',
    'tel': 'tel', 'tel-national': 'tel', 'street-address': 'street-address', 'address-line1': 'address-line1',
    'address-line2': 'address-line2', 'address-level2': 'address-level2', 'address-level1': 'address-level1',
    'postal-code': 'postal-code', 'country': 'country-name', 'country-name': 'country-name',
  };
  // Order matters: the first match wins, so the specific roles come before the general ones.
  const HINTS = [
    ['cc-number', /card.?(number|num|no)\b|cc.?(num|number)|n[uú]mero.{0,12}cart[aã]o|cart[aã]o.{0,12}n[uú]mero/],
    ['cc-name', /name.{0,10}on.{0,5}card|card.?holder|cc.?name|nome.{0,12}cart[aã]o|titular/],
    ['cc-exp-month', /exp.{0,10}month|cc.?exp.?mo|m[eê]s.{0,10}(validade|venc)/],
    ['cc-exp-year', /exp.{0,10}year|cc.?exp.?y|ano.{0,10}(validade|venc)/],
    ['cc-exp', /expir|validade|vencimento|\bexp\b|mm.?\/.?yy/],
    ['password', /pass(word)?|senha/],
    ['email', /e-?mail/],
    ['tel', /phone|mobile|telefone|celular|\btel\b/],
    ['postal-code', /\bzip|postal|\bcep\b|post.?code/],
    ['address-line2', /address.?(line)?.?2|complemento|apartment|\bapt\b/],
    ['address-line1', /address|street|endere[cç]o|logradouro|\brua\b/],
    ['address-level2', /\bcity\b|cidade/],
    ['address-level1', /\bstate\b|province|region|\bestado\b|\buf\b/],
    ['country-name', /country|pa[ií]s/],
    ['organization', /company|organi[sz]ation|empresa/],
    ['given-name', /first.?name|given.?name|primeiro.?nome/],
    ['family-name', /last.?name|family.?name|surname|sobrenome/],
    ['name', /full.?name|\bname\b|\bnome\b/],
    ['username', /user.?(name|id)?|login|usu[aá]rio/],
  ];
  const SEARCH = /search|query|\bq\b|busca|pesquis/;
  const TEXT_TYPES = new Set(['', 'text', 'email', 'tel', 'password', 'number', 'search']);

  /** info: { type, autocomplete, text } where text is the name, id, placeholder, aria-label and label joined. */
  function roleFor(info) {
    const type = String(info.type || '').toLowerCase();
    if (!TEXT_TYPES.has(type) && type !== 'select') return null;
    if (type === 'password') return 'password';
    for (const token of String(info.autocomplete || '').toLowerCase().split(/\s+/).reverse()) {
      if (AUTOCOMPLETE[token]) return AUTOCOMPLETE[token];
    }
    const text = String(info.text || '').toLowerCase();
    if (type === 'search' || SEARCH.test(text)) return null;
    if (type === 'email') return 'email';
    if (type === 'tel') return 'tel';
    for (const [role, pattern] of HINTS) if (pattern.test(text)) return role;
    return null;
  }

  /** Which group of saved data a role is filled from. A bare email field only counts as login inside a login form. */
  function kindOf(role, formHasPassword) {
    if (role === 'password') return 'login';
    if (role === 'username') return formHasPassword ? 'login' : null;
    if (role === 'email' && formHasPassword) return 'login';
    if (role.startsWith('cc-')) return 'card';
    return 'contact';
  }

  /** The month and year of a card in the shape a field asks for. */
  function expiry(month, year, hint) {
    const mm = String(month).padStart(2, '0');
    const yyyy = String(year);
    const long = /yyyy/i.test(hint.placeholder || '') || Number(hint.maxLength) >= 7;
    const separator = / \/ /.test(hint.placeholder || '') ? ' / ' : '/';
    return mm + separator + (long ? yyyy : yyyy.slice(-2));
  }

  const api = { roleFor, kindOf, expiry };
  if (typeof module === 'object' && module && module.exports) module.exports = api;
  return api;
})()
