/** @type {Record<string, string>} */
const KNOWN = {
  'lock.wrong_secret': 'lock.error_wrong_secret',
  'lock.wrong_secret_wait': 'lock.error_wrong_secret_wait',
  'lock.throttled': 'lock.error_throttled',
  'lock.not_enabled': 'lock.error_not_enabled',
  'lock.not_locked': 'lock.error_not_locked',
  'lock.method_unavailable': 'lock.error_method_unavailable',
  'lock.already_on': 'lock.error_already_on',
  'lock.last_method': 'lock.error_last_method',
  'lock.password_invalid': 'lock.error_password_invalid',
  'lock.vault_without_password': 'lock.error_vault_without_password',
  'lock.invalid_request': 'lock.error_failed',
  'lock.start_failed': 'lock.error_start_failed',
  'lock.erase_failed': 'lock.error_erase_failed',
  'lock.not_confirmed': 'lock.error_failed',
  'lock.corrupt': 'lock.error_failed',
  'lock.crypto_failed': 'lock.error_failed',
  // Windows Hello answers with the codes it has always used.
  'vault.hello_cancelled': 'lock.error_hello_cancelled',
  'vault.hello_failed': 'lock.error_hello_failed',
  'vault.hello_unavailable': 'lock.error_hello_unavailable',
  'vault.private_window': 'lock.error_private',
};

/**
 * Maps what the backend says went wrong to a message key (and its values). Codes carry no secret;
 * an unknown one falls back to a generic message. `lock.throttled:N` and `lock.wrong_secret_wait:N`
 * carry the seconds to wait.
 * @param {unknown} error
 */
export function lockError(error) {
  const text = typeof error === 'string' ? error : error instanceof Error ? error.message : String(error ?? '');
  const match = /((?:lock|vault)\.[a-z_]+)(?::(\d+))?/.exec(text);
  const code = match?.[1] ?? '';
  const seconds = match?.[2] ? Number(match[2]) : 0;
  return {
    key: KNOWN[code] ?? 'lock.error_failed',
    values: seconds ? { seconds } : {},
    code: KNOWN[code] ? code : 'lock.failed',
    seconds,
  };
}
