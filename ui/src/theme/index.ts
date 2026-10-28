export type AppTheme = "dark" | "light";

/** Préférence utilisateur : système (défaut) ou forcée. */
export type ThemePreference = "system" | AppTheme;

let currentTheme: AppTheme = "dark";
let mediaQuery: MediaQueryList | null = null;
let mediaHandler: (() => void) | null = null;
const listeners = new Set<() => void>();

export function detectSystemTheme(): AppTheme {
  if (typeof window === "undefined" || !window.matchMedia) {
    return "dark";
  }
  return window.matchMedia("(prefers-color-scheme: light)").matches
    ? "light"
    : "dark";
}

export function resolveTheme(preference: ThemePreference): AppTheme {
  if (preference === "dark" || preference === "light") {
    return preference;
  }
  return detectSystemTheme();
}

function paintTheme(theme: AppTheme): void {
  if (typeof document === "undefined") {
    return;
  }
  const root = document.documentElement;
  if (theme === "dark") {
    root.classList.add("dark");
    root.setAttribute("data-theme", "dark");
  } else {
    root.classList.remove("dark");
    root.setAttribute("data-theme", "default");
  }
}

export function getTheme(): AppTheme {
  return currentTheme;
}

export function setTheme(theme: AppTheme): void {
  if (currentTheme === theme) {
    paintTheme(theme);
    return;
  }
  currentTheme = theme;
  paintTheme(theme);
  for (const listener of listeners) {
    listener();
  }
}

function clearSystemListener(): void {
  if (mediaQuery && mediaHandler) {
    mediaQuery.removeEventListener("change", mediaHandler);
  }
  mediaQuery = null;
  mediaHandler = null;
}

export function applyThemePreference(preference: ThemePreference): AppTheme {
  clearSystemListener();
  const resolved = resolveTheme(preference);
  setTheme(resolved);

  if (preference === "system" && typeof window !== "undefined") {
    mediaQuery = window.matchMedia("(prefers-color-scheme: light)");
    mediaHandler = () => {
      setTheme(detectSystemTheme());
    };
    mediaQuery.addEventListener("change", mediaHandler);
  }

  return resolved;
}

export function subscribeTheme(listener: () => void): () => void {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

export function isThemePreference(value: string): value is ThemePreference {
  return value === "system" || value === "dark" || value === "light";
}
