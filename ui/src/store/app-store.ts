import { toast } from "@heroui/react";
import { create } from "zustand";

import {
  getAppStatus,
  parseAppError,
  setDemoMode as persistDemoMode,
  setLogLevel as persistLogLevel,
} from "../lib/tauri";
import type { AppError, AppStatus, LogLevel, Screen } from "../types";

type AppState = {
  status: AppStatus | null;
  loading: boolean;
  error: AppError | null;
  screen: Screen;
  demoConnected: boolean;
  loadStatus: () => Promise<void>;
  setScreen: (screen: Screen) => void;
  setDemoMode: (enabled: boolean) => Promise<void>;
  setLogLevel: (level: LogLevel) => Promise<void>;
  connectDemo: () => void;
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
  screen: "connexion",
  demoConnected: false,
  setScreen: (screen) => set({ screen }),
  loadStatus: async () => {
    set({ loading: true });
    try {
      const status = await getAppStatus();
      set({
        status,
        loading: false,
        error: null,
        demoConnected: status.demo_mode ? get().demoConnected : false,
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
        demoConnected: enabled ? get().demoConnected : false,
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
  connectDemo: () => {
    const { status } = get();
    if (!status?.demo_mode) {
      return;
    }
    set({ demoConnected: true });
    toast.success("Mode démo", {
      description:
        "Aucune connexion à un appareil réel n'a été tentée. L'interface est prête pour les phases suivantes.",
    });
  },
}));
