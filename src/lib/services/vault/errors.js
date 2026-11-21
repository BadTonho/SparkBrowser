/** @type {Record<string, string>} */
const KNOWN = {
  'vault.wrong_secret': 'passwords.error_wrong_secret',
  'vault.throttled': 'passwords.error_throttled',
  'vault.locked': 'passwords.error_locked',
  'vault.invalid_entry': 'passwords.error_invalid_entry',
  'vault.password_invalid': 'passwords.error_password_invalid',
  'vault.full': 'passwords.error_full',
  'vault.corrupt': 'passwords.error_corrupt',
  'vault.private_window': 'passwords.private_unavailable',
  'vault.clipboard_failed': 'passwords.copy_failed',
  'vault.hello_cancelled': 'passwords.error_hello_cancelled',
  'vault.hello_failed': 'passwords.error_hello_failed',
  'vault.hello_unavailable': 'passwords.error_hello_unavailable',
  'vault.method_unavailable': 'passwords.error_method_unavailable',
  'vault.method_already_on': 'passwords.error_method_already_on',
  'vault.last_method': 'passwords.error_last_method',
  'vault.hello_required': 'passwords.error_hello_required',
  'vault.import_invalid': 'passwords.error_import_invalid',
  'vault.import_too_large': 'passwords.error_import_too_large',
  'vault.import_failed': 'passwords.error_import_failed',
  'vault.export_failed': 'passwords.error_export_failed',
};

/**
 * Maps what the backend says went wrong to a message key (and its values) for the interface.
 * Codes carry no secret; unknown ones fall back to a generic message.
 * @param {unknown} error
 * @returns {{ key: string, values: Record<string, unknown>, code: string }}
 */
export function vaultError(error) {
  const text = typeof error === 'string' ? error : error instanceof Error ? error.message : String(error ?? '');
  const [code, argument] = text.split(':');
  const key = KNOWN[code] ?? 'passwords.error_generic';
  return {
    key,
    values: code === 'vault.throttled' ? { seconds: Number(argument) || 1 } : {},
    code: KNOWN[code] ? code : 'vault.failed',
  };
}
