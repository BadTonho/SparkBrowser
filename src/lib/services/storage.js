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
let reported = false;
export const hasStorageFailure = () => reported;
export const safeStorage = createStorageAdapter(() => globalThis.localStorage, operation => {
  if (reported || typeof window === 'undefined') return;
  reported = true;
  queueMicrotask(() => window.dispatchEvent(new CustomEvent('storage-unavailable', { detail: operation })));
});
