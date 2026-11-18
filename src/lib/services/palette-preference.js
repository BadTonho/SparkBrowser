export const PALETTE_STORAGE_KEY = 'spark_color_palette';
/** The first palette is the default: the browser's original black and gray. */
export const PALETTES = /** @type {const} */ (['graphite', 'ocean', 'violet', 'emerald', 'rose', 'amber']);
export const DEFAULT_PALETTE = PALETTES[0];

/**
 * @typedef {typeof PALETTES[number]} Palette
 */

/** @param {unknown} value @returns {value is Palette} */
export const isPalette = value => typeof value === 'string' && PALETTES.includes(/** @type {Palette} */ (value));

/**
 * @param {{ getItem: (key: string) => string | null } | undefined} [storage]
 * @returns {Palette}
 */
export function readPalette(storage) {
  if (!storage) return DEFAULT_PALETTE;
  try {
    const saved = storage.getItem(PALETTE_STORAGE_KEY);
    return isPalette(saved) ? saved : DEFAULT_PALETTE;
  } catch {
    return DEFAULT_PALETTE;
  }
}

/**
 * The colors live in `theme.css`, selected by this attribute. The default palette has no attribute, so
 * the original colors apply exactly as before.
 * @param {{ setAttribute: (name: string, value: string) => void, removeAttribute: (name: string) => void } | undefined} root
 * @param {Palette} palette
 */
export function applyPaletteToRoot(root, palette) {
  if (!root || typeof root.setAttribute !== 'function') return;
  if (palette === DEFAULT_PALETTE) root.removeAttribute('data-palette');
  else root.setAttribute('data-palette', palette);
}

/**
 * @param {{
 *   storage?: { getItem: (key: string) => string | null, setItem: (key: string, value: string) => unknown },
 *   root?: { setAttribute: (name: string, value: string) => void, removeAttribute: (name: string) => void },
 *   changed?: (palette: Palette) => void
 * }} [options]
 */
export function createPalettePreference(options = {}) {
  const { storage, root, changed } = options;
  let current = readPalette(storage);
  applyPaletteToRoot(root, current);
  return {
    get() {
      return current;
    },
    /** Re-reads the saved choice, for when another window changed it. */
    sync() {
      const saved = readPalette(storage);
      if (saved === current) return current;
      current = saved;
      applyPaletteToRoot(root, current);
      changed?.(current);
      return current;
    },
    /** @param {string} palette */
    set(palette) {
      if (!isPalette(palette)) return current;
      current = palette;
      applyPaletteToRoot(root, current);
      if (storage) {
        try {
          storage.setItem(PALETTE_STORAGE_KEY, palette);
        } catch {
          // Keep the in-memory preference when storage is unavailable.
        }
      }
      changed?.(current);
      return current;
    },
  };
}
