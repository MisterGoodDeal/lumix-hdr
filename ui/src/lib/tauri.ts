import { invoke } from "@tauri-apps/api/core";

import type { AppError, AppStatus, LogLevel } from "../types";

const MOCK_STATUS: AppStatus = {
  version: "0.1.0",
  os: "macos",
  demo_mode: false,
  demo_forced_by_env: false,
  camera_available: false,
  log_level: "info",
  log_dir: "(navigateur — hors application Tauri)",
};

export function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export function parseAppError(error: unknown): AppError {
  if (typeof error === "object" && error !== null && "message" in error) {
    const payload = error as { message: unknown; detail?: unknown };
    if (typeof payload.message === "string") {
      return {
        message: payload.message,
        detail: typeof payload.detail === "string" ? payload.detail : undefined,
      };
    }
  }
  if (typeof error === "string") {
    try {
      return parseAppError(JSON.parse(error));
    } catch {
      return {
        message: "Une erreur est survenue.",
        detail: error,
      };
    }
  }
  return {
    message: "Une erreur est survenue.",
    detail: String(error),
  };
}

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri()) {
    throw new Error("Commande Tauri indisponible hors de l'application.");
  }
  return invoke<T>(command, args);
}

export async function getAppStatus(): Promise<AppStatus> {
  if (!isTauri()) {
    return MOCK_STATUS;
  }
  return call<AppStatus>("get_app_status");
}

export async function setDemoMode(enabled: boolean): Promise<AppStatus> {
  if (!isTauri()) {
    return { ...MOCK_STATUS, demo_mode: enabled };
  }
  return call<AppStatus>("set_demo_mode", { enabled });
}

export async function setLogLevel(level: LogLevel): Promise<AppStatus> {
  if (!isTauri()) {
    return { ...MOCK_STATUS, log_level: level };
  }
  return call<AppStatus>("set_log_level", { level });
}

export async function getLogDir(): Promise<string> {
  if (!isTauri()) {
    return MOCK_STATUS.log_dir;
  }
  return call<string>("get_log_dir");
}
