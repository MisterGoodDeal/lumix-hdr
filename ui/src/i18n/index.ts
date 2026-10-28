import { en } from "./locales/en";
import { fr } from "./locales/fr";
import type { AppLocale, LocalePreference, TranslationKey } from "./types";

export type { AppLocale, LocalePreference, TranslationKey };

const catalogs: Record<AppLocale, Record<TranslationKey, string>> = {
  en,
  fr,
};

let currentLocale: AppLocale = "en";
const listeners = new Set<() => void>();

export function detectSystemLocale(): AppLocale {
  if (typeof navigator === "undefined") {
    return "en";
  }
  const lang = (navigator.language || "en").toLowerCase();
  return lang.startsWith("fr") ? "fr" : "en";
}

export function resolveLocale(preference: LocalePreference): AppLocale {
  if (preference === "en" || preference === "fr") {
    return preference;
  }
  return detectSystemLocale();
}

export function getLocale(): AppLocale {
  return currentLocale;
}

export function setLocale(locale: AppLocale): void {
  if (currentLocale === locale) {
    return;
  }
  currentLocale = locale;
  if (typeof document !== "undefined") {
    document.documentElement.lang = locale;
  }
  for (const listener of listeners) {
    listener();
  }
}

export function applyLocalePreference(preference: LocalePreference): AppLocale {
  const resolved = resolveLocale(preference);
  setLocale(resolved);
  return resolved;
}

export function subscribeLocale(listener: () => void): () => void {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

export type TranslateVars = Record<string, string | number>;

export function t(key: TranslationKey, vars?: TranslateVars): string {
  const catalog = catalogs[currentLocale] ?? catalogs.en;
  let text = catalog[key] ?? catalogs.en[key] ?? key;
  if (vars) {
    for (const [name, value] of Object.entries(vars)) {
      text = text.replaceAll(`{{${name}}}`, String(value));
    }
  }
  return text;
}

export function isLocalePreference(value: string): value is LocalePreference {
  return value === "system" || value === "en" || value === "fr";
}
