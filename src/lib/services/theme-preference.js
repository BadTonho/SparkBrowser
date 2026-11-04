export const THEME_MODE_STORAGE_KEY = 'veloce_theme_mode';
export const THEME_MODES = /** @type {const} */ (['dark', 'light']);

/**
 * @typedef {'dark' | 'light'} ThemeMode
 */

/**
 * @param {{ getItem: (key: string) => string | null } | undefined} [storage]
 * @returns {ThemeMode}
 */
export function readThemeMode(storage) {
  if (!storage) return 'dark';
  try {
    const saved = storage.getItem(THEME_MODE_STORAGE_KEY);
    return saved === 'light' ? 'light' : 'dark';
  } catch {
    return 'dark';
  }
}

/**
 * @param {HTMLElement | { setAttribute: (name: string, value: string) => void } | undefined} root
 * @param {ThemeMode} mode
 */
export function applyThemeModeToRoot(root, mode) {
  if (root && typeof root.setAttribute === 'function') {
    root.setAttribute('data-theme', mode);
  }
}

/**
 * @param {{
 *   storage?: { getItem: (key: string) => string | null, setItem: (key: string, value: string) => unknown },
 *   root?: HTMLElement | { setAttribute: (name: string, value: string) => void },
 *   changed?: (mode: ThemeMode) => void
 * }} [options]
 */
export function createThemePreference(options = {}) {
  const { storage, root, changed } = options;
  let current = readThemeMode(storage);
  applyThemeModeToRoot(root, current);

  return {
    get() {
      return current;
    },
    /** @param {string} mode */
    set(mode) {
      if (mode !== 'dark' && mode !== 'light') return current;
      current = mode;
      applyThemeModeToRoot(root, current);
      if (storage) {
        try {
          storage.setItem(THEME_MODE_STORAGE_KEY, mode);
        } catch {
          // Keep the in-memory preference when storage is unavailable.
        }
      }
      changed?.(current);
      return current;
    },
  };
}
