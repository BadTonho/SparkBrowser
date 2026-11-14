export const VISUAL_PREFERENCE_STORAGE_KEYS = Object.freeze({
  wideAddressBar: "spark_visual_wide_address_bar",
  showFullUrls: "spark_visual_show_full_urls",
  compactToolbar: "spark_visual_compact_toolbar",
  roundedContentFrame: "spark_visual_rounded_content_frame",
});

export const DEFAULT_VISUAL_PREFERENCES = Object.freeze({
  wideAddressBar: false,
  showFullUrls: false,
  compactToolbar: false,
  roundedContentFrame: false,
});

/** @typedef {keyof typeof VISUAL_PREFERENCE_STORAGE_KEYS} VisualPreferenceName */
const preferenceNames = /** @type {VisualPreferenceName[]} */ (Object.keys(VISUAL_PREFERENCE_STORAGE_KEYS));

/** @param {Pick<Storage, "getItem">} storage */
export function readVisualPreferences(storage) {
  /** @type {Record<VisualPreferenceName, boolean>} */
  const preferences = { ...DEFAULT_VISUAL_PREFERENCES };
  if (!storage) return preferences;

  for (const name of preferenceNames) {
    try {
      const value = storage.getItem(VISUAL_PREFERENCE_STORAGE_KEYS[name]);
      if (value === "true") preferences[name] = true;
      else if (value === "false") preferences[name] = false;
    } catch {
      // Keep this preference at its safe default when storage is unavailable.
    }
  }
  return preferences;
}

/** @param {Pick<Storage, "setItem">} storage @param {keyof typeof VISUAL_PREFERENCE_STORAGE_KEYS} name @param {boolean} value */
export function writeVisualPreference(storage, name, value) {
  const key = VISUAL_PREFERENCE_STORAGE_KEYS[name];
  if (!key || typeof value !== "boolean" || !storage) return false;
  try {
    storage.setItem(key, String(value));
    return true;
  } catch {
    return false;
  }
}

/** Shorten only idle HTTP(S) presentation. The caller retains the real URL for editing and navigation.
 * @param {string} address
 * @param {boolean} showFullUrl
 * @returns {string}
 */
export function formatAddressForDisplay(address, showFullUrl) {
  if (showFullUrl || typeof address !== "string") return address;
  return address.replace(/^(https?:\/\/)(?:www\.)?/i, "");
}
