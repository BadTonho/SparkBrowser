export const CLEAR_AFTER_MS = 30_000;

/** @type {unknown} */
let pending = null;

/**
 * Copies a secret and empties the clipboard again after a while, so it does not stay there.
 * Returns false when copying was not possible.
 * @param {string} text
 * @param {{ clipboard?: { writeText(text: string): Promise<void> }, setTimer?: (fn: () => void, ms: number) => unknown, clearTimer?: (id: unknown) => void, onError?: (error: unknown) => void }} [env]
 * @returns {Promise<boolean>}
 */
export async function copySecret(text, env = {}) {
  const clipboard = env.clipboard ?? (typeof navigator !== 'undefined' ? navigator.clipboard : undefined);
  const setTimer = env.setTimer ?? ((/** @type {() => void} */ fn, /** @type {number} */ ms) => setTimeout(fn, ms));
  const clearTimer = env.clearTimer ?? ((/** @type {any} */ id) => clearTimeout(id));
  if (!clipboard) return false;
  try {
    await clipboard.writeText(text);
  } catch (error) {
    env.onError?.(error);
    return false;
  }
  if (pending !== null) clearTimer(pending);
  pending = setTimer(() => {
    pending = null;
    Promise.resolve(clipboard.writeText('')).catch(error => env.onError?.(error));
  }, CLEAR_AFTER_MS);
  return true;
}

/** For tests: forget a scheduled clean-up. */
export function resetClipboardTimer() { pending = null; }
