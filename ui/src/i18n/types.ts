export type AppLocale = "en" | "fr";

/** Préférence utilisateur : système (défaut) ou forcée. */
export type LocalePreference = "system" | AppLocale;

export type TranslationKey = keyof typeof import("./locales/en").en;
