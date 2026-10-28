import { toast } from "@heroui/react";
import { create } from "zustand";

import { applyLocalePreference, t } from "../i18n";
import {
  connectCamera as invokeConnectCamera,
  disconnectCamera as invokeDisconnectCamera,
  getAppStatus,
  onCameraDisconnected,
  parseAppError,
  setDemoMode as persistDemoMode,
  setFusionOptions as persistFusionOptions,
  setLocale as persistLocale,
  setLogLevel as persistLogLevel,
  setSessionsDir as persistSessionsDir,
  setTheme as persistTheme,
  startLiveView,
  stopLiveView,
} from "../lib/tauri";
import { applyThemePreference } from "../theme";
import type {
  AppError,
  AppStatus,
  FusionOptions,
  LocalePreference,
  LogLevel,
  Screen,
  ThemePreference,
} from "../types";

type AppState = {
  status: AppStatus | null;
  loading: boolean;
  error: AppError | null;
  screen: Screen;
  /** Démo mock ou appareil réel déjà vérifié via Connecter. */
  cameraConnected: boolean;
  connectedModel: string | null;
  loadStatus: () => Promise<void>;
  setScreen: (screen: Screen) => void;
  setDemoMode: (enabled: boolean) => Promise<void>;
  setLogLevel: (level: LogLevel) => Promise<void>;
  setFusionOptions: (options: FusionOptions) => Promise<void>;
  setLocale: (locale: LocalePreference) => Promise<void>;
  setTheme: (theme: ThemePreference) => Promise<void>;
  setSessionsDir: (path: string | null) => Promise<void>;
  connectCamera: () => Promise<void>;
  disconnectCamera: () => Promise<void>;
  handleCameraDisconnected: () => void;
  startListeningDisconnect: () => Promise<() => void>;
};

function showError(error: AppError) {
  toast.danger(error.message, {
    description: error.detail,
  });
}

export const useAppStore = create<AppState>((set, get) => ({
  status: null,
  loading: true,
  error: null,
  screen: "studio",
  cameraConnected: false,
  connectedModel: null,
  setScreen: (screen) => set({ screen }),
  loadStatus: async () => {
    set({ loading: true });
    try {
      const status = await getAppStatus();
      applyLocalePreference(status.locale);
      applyThemePreference(status.theme);
      set({
        status,
        loading: false,
        error: null,
      });
    } catch (cause) {
      const error = parseAppError(cause);
      set({ loading: false, error });
      showError(error);
    }
  },
  setDemoMode: async (enabled) => {
    try {
      const status = await persistDemoMode(enabled);
      set({
        status,
        error: null,
        cameraConnected: false,
        connectedModel: null,
      });
    } catch (cause) {
      const error = parseAppError(cause);
      set({ error });
      showError(error);
    }
  },
  setLogLevel: async (level) => {
    try {
      const status = await persistLogLevel(level);
      set({ status, error: null });
    } catch (cause) {
      const error = parseAppError(cause);
      set({ error });
      showError(error);
    }
  },
  setFusionOptions: async (options) => {
    try {
      const status = await persistFusionOptions(options);
      set({ status, error: null });
    } catch (cause) {
      const error = parseAppError(cause);
      set({ error });
      showError(error);
    }
  },
  setLocale: async (locale) => {
    try {
      const status = await persistLocale(locale);
      applyLocalePreference(status.locale);
      set({ status, error: null });
    } catch (cause) {
      const error = parseAppError(cause);
      set({ error });
      showError(error);
    }
  },
  setTheme: async (theme) => {
    try {
      const status = await persistTheme(theme);
      applyThemePreference(status.theme);
      set({ status, error: null });
    } catch (cause) {
      const error = parseAppError(cause);
      set({ error });
      showError(error);
    }
  },
  setSessionsDir: async (path) => {
    try {
      const status = await persistSessionsDir(path);
      set({ status, error: null });
      toast.success(
        path ? t("settings.sessionsChanged") : t("settings.sessionsReset"),
        { description: status.sessions_dir },
      );
    } catch (cause) {
      const error = parseAppError(cause);
      set({ error });
      showError(error);
    }
  },
  connectCamera: async () => {
    const { status } = get();
    if (!status) {
      return;
    }
    if (!status.demo_mode && !status.camera_available) {
      const error = {
        message: t("toast.cameraWindowsOnly"),
      };
      set({ error });
      showError(error);
      return;
    }
    try {
      const device = await invokeConnectCamera();
      set({
        cameraConnected: true,
        connectedModel: device.model,
        error: null,
      });
      try {
        await startLiveView();
      } catch (cause) {
        const error = parseAppError(cause);
        toast.danger(t("toast.liveViewKo"), {
          description: error.detail ?? error.message,
        });
      }
      toast.success(
        status.demo_mode ? t("toast.demoReady") : t("toast.connected"),
        { description: device.model },
      );
    } catch (cause) {
      const error = parseAppError(cause);
      set({ cameraConnected: false, connectedModel: null, error });
      showError(error);
    }
  },
  disconnectCamera: async () => {
    try {
      await stopLiveView();
    } catch {
      // ignore
    }
    try {
      await invokeDisconnectCamera();
    } catch (cause) {
      const error = parseAppError(cause);
      set({ error });
      showError(error);
      return;
    }
    set({ cameraConnected: false, connectedModel: null, error: null });
    toast.success(t("toast.disconnected"));
  },
  handleCameraDisconnected: () => {
    set({ cameraConnected: false, connectedModel: null });
    toast.danger(t("toast.unplugged"), {
      description: t("toast.sessionClosed"),
    });
  },
  startListeningDisconnect: async () => {
    return onCameraDisconnected(() => {
      get().handleCameraDisconnected();
    });
  },
}));
