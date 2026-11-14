/**
 * One native error code, `backup.sync_gdrive_login_required`, covers many different causes. The
 * native side appends which check failed (`code: reason`), and this turns that reason into the one
 * thing the user can do about it. Unknown or missing reasons fall back to a generic hint.
 */
const FIXES = {
  link: ['account-not-linked', 'binding-unreadable'],
  open_tab: ['no-open-tab-in-the-linked-profile', 'no-tab-selected'],
  sign_in: [
    'google-session-cookies-missing',
    'no-sapisid-cookie',
    'drive-redirected-to-sign-in',
    'drive-page-rejected-401-403',
    'drive-api-401-v2',
    'drive-api-401-v3',
    'drive-file-lookup-401',
    'drive-upload-401',
    'drive-api-401',
    'drive-download-401',
    'drive-page-shows-sign-in',
    'hidden-page-shows-sign-in',
    'no-sapisid-cookie-readable',
  ],
  confirm_account: ['account-email-not-detected'],
  different_account: ['account-email-changed'],
  page_changed: ['no-drive-api-key-found'],
  link_failed: [
    'tab-not-found',
    'tab-webview-missing',
    'tab-url-invalid',
    'tab-is-not-a-drive-page',
    'account-index-not-found',
    'binding-not-saved',
    'database-unavailable',
    'state-unavailable',
  ],
};

/** @param {unknown} detail the technical text of a Drive login error */
export function gdriveReason(detail) {
  const match = /backup\.sync_gdrive_login_required:\s*([a-z0-9-]+)/.exec(String(detail ?? ''));
  return match ? match[1] : '';
}

/**
 * @param {unknown} detail the technical text of a Drive login error
 * @returns {'link'|'open_tab'|'sign_in'|'confirm_account'|'different_account'|'page_changed'|'link_failed'|'generic'}
 */
export function gdriveFix(detail) {
  const reason = gdriveReason(detail);
  for (const [fix, reasons] of Object.entries(FIXES)) {
    if (reasons.includes(reason)) return /** @type {any} */ (fix);
  }
  return 'generic';
}
