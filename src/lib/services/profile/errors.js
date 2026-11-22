/** @type {Record<string, string>} */
const KNOWN = {
  'profile.invalid_name': 'profiles.error_invalid_name',
  'profile.duplicate': 'profiles.error_duplicate',
  'profile.limit': 'profiles.error_limit',
  'profile.not_found': 'profiles.error_not_found',
  'profile.in_use': 'profiles.error_in_use',
  'profile.cannot_delete_current': 'profiles.error_cannot_delete_current',
  'profile.cannot_delete_default': 'profiles.error_cannot_delete_default',
  'profile.cannot_rename_default': 'profiles.error_cannot_rename_default',
  'profile.private_window': 'profiles.error_private',
  'profile.not_picker': 'profiles.error_failed',
  'profile.open_failed': 'profiles.error_open_failed',
};

/**
 * Maps what the backend says went wrong to a message key. Codes carry no secret; the ones that mean
 * "something failed, see the log" (create, rename, delete, open) share one generic message.
 * @param {unknown} error
 */
export function profileError(error) {
  const text = typeof error === 'string' ? error : error instanceof Error ? error.message : String(error ?? '');
  const code = text.split(':').find(part => part.trim().startsWith('profile.'))?.trim() ?? '';
  return { key: KNOWN[code] ?? 'profiles.error_failed', code: KNOWN[code] ? code : 'profile.failed' };
}
