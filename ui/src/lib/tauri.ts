import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";

import type {
  AppError,
  AppStatus,
  CameraDefaults,
  CameraDeviceInfo,
  CaptureDefaults,
  CameraSettingsDto,
  ExposureCapabilities,
  FusionOptions,
  FusionOutputs,
  LiveViewFrame,
  LocalePreference,
  LogLevel,
  ThemePreference,
} from "../types";
import { t } from "../i18n";
import {
  DEFAULT_CAMERA_DEFAULTS,
  DEFAULT_CAPTURE_DEFAULTS,
  DEFAULT_FUSION_OPTIONS,
} from "../types";

const MOCK_STATUS: AppStatus = {
  version: "0.1.0",
  os: "macos",
  demo_mode: false,
  demo_forced_by_env: false,
  camera_available: false,
  log_level: "info",
  locale: "system",
  theme: "system",
  log_dir: "(browser — outside Tauri)",
  sessions_dir: "(browser — outside Tauri)",
  sessions_dir_custom: false,
  fusion: DEFAULT_FUSION_OPTIONS,
  capture: DEFAULT_CAPTURE_DEFAULTS,
  camera: DEFAULT_CAMERA_DEFAULTS,
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
        message: t("common.error"),
        detail: error,
      };
    }
  }
  return {
    message: t("common.error"),
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

export async function listCameras(): Promise<CameraDeviceInfo[]> {
  if (!isTauri()) {
    return [{ index: 0, model: "MockCamera (navigateur)" }];
  }
  return call<CameraDeviceInfo[]>("list_cameras");
}

export async function connectCamera(
  deviceIndex?: number,
): Promise<CameraDeviceInfo> {
  if (!isTauri()) {
    return { index: 0, model: "MockCamera (navigateur)" };
  }
  return call<CameraDeviceInfo>("connect_camera", {
    device_index: deviceIndex ?? null,
  });
}

export async function disconnectCamera(): Promise<void> {
  if (!isTauri()) {
    return;
  }
  await call<void>("disconnect_camera");
}

export async function getCameraSettings(): Promise<ExposureCapabilities> {
  if (!isTauri()) {
    return {
      settings: {
        mode: "M",
        aperture: DEFAULT_CAMERA_DEFAULTS.aperture,
        iso: DEFAULT_CAMERA_DEFAULTS.iso,
        shutter_seconds: DEFAULT_CAMERA_DEFAULTS.shutter_seconds,
        shutter_label: "1/60",
      },
      iso_values: [100, 200, 400, 800, 1600],
      aperture_values: [1.4, 2, 2.8, 4, 5.6, 8, 11, 16],
      shutter_seconds: [1 / 1000, 1 / 500, 1 / 250, 1 / 125, 1 / 60, 1 / 30, 1 / 15],
      shutter_labels: ["1/1000", "1/500", "1/250", "1/125", "1/60", "1/30", "1/15"],
    };
  }
  return call<ExposureCapabilities>("get_camera_settings");
}

export async function setCameraSettings(
  settings: CameraDefaults,
): Promise<ExposureCapabilities> {
  if (!isTauri()) {
    return getCameraSettings();
  }
  return call<ExposureCapabilities>("set_camera_settings", { settings });
}

export async function setCaptureDefaults(
  defaults: CaptureDefaults,
): Promise<AppStatus> {
  if (!isTauri()) {
    return { ...MOCK_STATUS, capture: defaults };
  }
  return call<AppStatus>("set_capture_defaults", { defaults });
}

export async function startLiveView(): Promise<void> {
  if (!isTauri()) {
    return;
  }
  await call<void>("start_live_view");
}

export async function stopLiveView(): Promise<void> {
  if (!isTauri()) {
    return;
  }
  await call<void>("stop_live_view");
}

export async function onLiveViewFrame(
  handler: (frame: LiveViewFrame) => void,
): Promise<UnlistenFn> {
  if (!isTauri()) {
    return () => undefined;
  }
  return listen<LiveViewFrame>("liveview-frame", (event) => {
    handler(event.payload);
  });
}

export async function onCameraDisconnected(
  handler: () => void,
): Promise<UnlistenFn> {
  if (!isTauri()) {
    return () => undefined;
  }
  return listen("camera-disconnected", () => {
    handler();
  });
}

export async function onCameraSettingsChanged(
  handler: (settings: CameraSettingsDto) => void,
): Promise<UnlistenFn> {
  if (!isTauri()) {
    return () => undefined;
  }
  return listen<CameraSettingsDto>("camera-settings-changed", (event) => {
    handler(event.payload);
  });
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

export async function setFusionOptions(
  options: FusionOptions,
): Promise<AppStatus> {
  if (!isTauri()) {
    return { ...MOCK_STATUS, fusion: options };
  }
  return call<AppStatus>("set_fusion_options", { options });
}

export async function setLocale(
  locale: LocalePreference,
): Promise<AppStatus> {
  if (!isTauri()) {
    return { ...MOCK_STATUS, locale };
  }
  return call<AppStatus>("set_locale", { locale });
}

export async function setTheme(
  theme: ThemePreference,
): Promise<AppStatus> {
  if (!isTauri()) {
    return { ...MOCK_STATUS, theme };
  }
  return call<AppStatus>("set_theme", { theme });
}

export async function setSessionsDir(
  path: string | null,
): Promise<AppStatus> {
  if (!isTauri()) {
    return {
      ...MOCK_STATUS,
      sessions_dir: path ?? MOCK_STATUS.sessions_dir,
      sessions_dir_custom: path !== null,
    };
  }
  return call<AppStatus>("set_sessions_dir", { path });
}

export async function pickImageFolder(): Promise<string | null> {
  if (!isTauri()) {
    return "/demo/bracket";
  }
  const selected = await open({
    directory: true,
    multiple: false,
    title: t("result.pickJpeg"),
  });
  if (typeof selected === "string" && selected.length > 0) {
    return selected;
  }
  return null;
}

export async function pickSessionsFolder(): Promise<string | null> {
  if (!isTauri()) {
    return "/demo/sessions";
  }
  const selected = await open({
    directory: true,
    multiple: false,
    title: t("settings.pickSessions"),
  });
  if (typeof selected === "string" && selected.length > 0) {
    return selected;
  }
  return null;
}

export async function listJpegFolder(dir: string): Promise<string[]> {
  if (!isTauri()) {
    return [
      `${dir}/img-1.jpg`,
      `${dir}/img-2.jpg`,
      `${dir}/img-3.jpg`,
    ];
  }
  return call<string[]>("list_jpeg_folder", { dir });
}

export async function mergeImageFolder(
  dir: string,
  options: FusionOptions,
): Promise<FusionOutputs> {
  if (!isTauri()) {
    await new Promise((resolve) => {
      window.setTimeout(resolve, 350);
    });
    return {
      jpeg: `${dir}/hdr-fused.jpg`,
      tiff: options.output_tiff ? `${dir}/hdr-fused.tiff` : null,
      image_count: 3,
      method: options.method,
      aligned: options.align,
    };
  }
  return call<FusionOutputs>("merge_image_folder", { dir, options });
}

export async function cancelFusion(): Promise<void> {
  if (!isTauri()) {
    return;
  }
  await call<void>("cancel_fusion");
}

export async function openPath(path: string): Promise<void> {
  if (!isTauri()) {
    return;
  }
  await call<void>("open_path", { path });
}

export function previewSrc(path: string | null): string | null {
  if (!path) {
    return null;
  }
  if (!isTauri()) {
    return null;
  }
  return convertFileSrc(path);
}

export async function onFusionProgress(
  handler: (value: number) => void,
): Promise<UnlistenFn> {
  if (!isTauri()) {
    return () => undefined;
  }
  return listen<number>("fusion-progress", (event) => {
    handler(event.payload);
  });
}
