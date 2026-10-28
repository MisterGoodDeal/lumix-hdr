import { useCallback, useSyncExternalStore } from "react";

import {
  getLocale,
  subscribeLocale,
  t,
  type TranslateVars,
  type TranslationKey,
} from "./index";

export function useLocale() {
  return useSyncExternalStore(subscribeLocale, getLocale, () => "en" as const);
}

export function useT() {
  // Abonnement locale : re-render quand la langue change ; `t` lit le module.
  useLocale();
  return useCallback(
    (key: TranslationKey, vars?: TranslateVars) => t(key, vars),
    [],
  );
}
