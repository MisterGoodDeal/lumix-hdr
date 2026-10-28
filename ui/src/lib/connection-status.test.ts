import { describe, expect, it } from "vitest";

import {
  connectionHelpText,
  connectionStatusLabel,
  osLabel,
} from "./connection-status";
import { DEFAULT_FUSION_OPTIONS, type AppStatus } from "../types";

function status(overrides: Partial<AppStatus> = {}): AppStatus {
  return {
    version: "0.1.0",
    os: "macos",
    demo_mode: false,
    demo_forced_by_env: false,
    camera_available: false,
    log_level: "info",
    log_dir: "/tmp/logs",
    sessions_dir: "/tmp/sessions",
    fusion: DEFAULT_FUSION_OPTIONS,
    ...overrides,
  };
}

describe("connectionStatusLabel", () => {
  it("affiche le mode démo", () => {
    expect(connectionStatusLabel(status({ demo_mode: true }), false)).toBe(
      "Mode démo",
    );
  });

  it("affiche le mode démo connecté", () => {
    expect(connectionStatusLabel(status({ demo_mode: true }), true)).toBe(
      "Mode démo — connecté",
    );
  });

  it("indique l'indisponibilité hors Windows", () => {
    expect(connectionStatusLabel(status(), false)).toBe(
      "Pilotage caméra indisponible",
    );
  });

  it("indique l'absence d'appareil sur Windows", () => {
    expect(
      connectionStatusLabel(status({ os: "windows", camera_available: true }), false),
    ).toBe("Aucun appareil");
  });
});

describe("connectionHelpText", () => {
  it("rappelle le câble USB direct hors Windows", () => {
    const text = connectionHelpText(status());
    expect(text).toContain("Windows 10/11");
    expect(text).toContain("sans hub");
  });
});

describe("osLabel", () => {
  it("traduit macos", () => {
    expect(osLabel("macos")).toBe("macOS");
  });
});
