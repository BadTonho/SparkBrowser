/** @param {() => Storage} getStorage @param {(operation: string) => void} onError */
export function createStorageAdapter(getStorage, onError) {
  return {
    /** @param {string} key */
    getItem(key) { try { return getStorage().getItem(key); } catch { onError('read'); return null; } },
    /** @param {string} key @param {string} value */
    setItem(key, value) { try { getStorage().setItem(key, value); return true; } catch { onError('write'); return false; } },
    /** @param {string} key */
    removeItem(key) { try { getStorage().removeItem(key); return true; } catch { onError('write'); return false; } },
  };
}
const LEGACY_PREFIX = 'veloce_';
const PREFIX = 'spark_';
/**
 * Settings saved before the app was renamed live under `veloce_*` keys. Copies each one to its
 * `spark_*` name (never overwriting a value the new name already has) and removes the old key.
 * Runs when this module loads, which is before any store reads its setting.
 * @param {Storage} storage
 * @returns {number} how many keys were handled
 */
export function migrateLegacyKeys(storage) {
  try {
    const legacy = [];
    for (let index = 0; index < storage.length; index += 1) {
      const key = storage.key(index);
      if (key && key.startsWith(LEGACY_PREFIX)) legacy.push(key);
    }
    for (const key of legacy) {
      const next = PREFIX + key.slice(LEGACY_PREFIX.length);
      const value = storage.getItem(key);
      if (value !== null && storage.getItem(next) === null) storage.setItem(next, value);
      storage.removeItem(key);
    }
    return legacy.length;
  } catch {
    // Storage that cannot be read or written keeps the old keys; the next start tries again.
    return 0;
  }
}
let reported = false;
export const hasStorageFailure = () => reported;
export const safeStorage = createStorageAdapter(() => globalThis.localStorage, operation => {
  if (reported || typeof window === 'undefined') return;
  reported = true;
  queueMicrotask(() => window.dispatchEvent(new CustomEvent('storage-unavailable', { detail: operation })));
});
if (typeof globalThis.localStorage !== 'undefined') migrateLegacyKeys(globalThis.localStorage);
