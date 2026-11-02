import { derived, get, writable } from "svelte/store";
import { safeStorage } from "../services/storage.js";
import { english, portuguese, type MessageKey } from "./catalogs";
import { interpolate, resolveLanguage } from "./language.js";

export type LanguagePreference = "system" | "pt-BR" | "en";
export type Locale = "pt-BR" | "en";
export type Translate = (key: MessageKey, values?: Record<string, unknown>) => string;
const saved = safeStorage.getItem("veloce_language");
export const languagePreference = writable<LanguagePreference>(saved === "pt-BR" || saved === "en" ? saved : "system");
const systemLanguages = writable<readonly string[]>(typeof navigator === "undefined" ? ["en"] : navigator.languages);
export const locale = derived([languagePreference, systemLanguages], ([preference, languages]) => resolveLanguage(preference, languages));
export const t = derived(locale, language => ((key, values = {}) => interpolate((language === "pt-BR" ? portuguese : english)[key], values)) as Translate);
export function tr(key: MessageKey, values?: Record<string, unknown>): string { return get(t)(key, values); }

// Only use for predefined interface labels in settings/shortcut definitions.
// User content (page titles, workspace names, notes) never passes through this lookup.
const labelKeys = new Map<string, MessageKey>(Object.entries(english).map(([key, text]) => [text, key as MessageKey]));
export const labelText = derived(t, translate => (text: string) => {
  const key = labelKeys.get(text);
  return key ? translate(key) : text;
});
export function setLanguage(preference: LanguagePreference): boolean {
  if (!["system", "pt-BR", "en"].includes(preference)) return false;
  languagePreference.set(preference);
  return safeStorage.setItem("veloce_language", preference);
}
export const formatDate = derived(locale, language => (timestamp: number, options?: Intl.DateTimeFormatOptions) => new Intl.DateTimeFormat(language, options).format(timestamp));
export const formatNumber = derived(locale, language => (number: number, options?: Intl.NumberFormatOptions) => new Intl.NumberFormat(language, options).format(number));
export const plural = derived(locale, language => (count: number, one: string, other: string) => interpolate(new Intl.PluralRules(language).select(count) === "one" ? one : other, { count: new Intl.NumberFormat(language).format(count) }));
if (typeof document !== "undefined") locale.subscribe(language => { document.documentElement.lang = language; });
if (typeof window !== "undefined") window.addEventListener("languagechange", () => systemLanguages.set(navigator.languages));
export type { MessageKey };
