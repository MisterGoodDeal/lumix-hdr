import { beforeEach, describe, expect, it } from "vitest";

import { setLocale } from "../i18n";
import {
  connectionHelpText,
  connectionStatusColor,
  connectionStatusLabel,
  osLabel,
} from "./connection-status";
import {
  DEFAULT_CAMERA_DEFAULTS,
  DEFAULT_CAPTURE_DEFAULTS,
  DEFAULT_FUSION_OPTIONS,
  type AppStatus,
} from "../types";

function status(overrides: Partial<AppStatus> = {}): AppStatus {
  return {
    version: "0.1.0",
    os: "macos",
    demo_mode: false,
    demo_forced_by_env: false,
    camera_available: false,
    log_level: "info",
    locale: "system",
    theme: "system",
    log_dir: "/tmp/logs",
    sessions_dir: "/tmp/sessions",
    sessions_dir_custom: false,
    fusion: DEFAULT_FUSION_OPTIONS,
    capture: DEFAULT_CAPTURE_DEFAULTS,
    camera: DEFAULT_CAMERA_DEFAULTS,
    ...overrides,
  };
}

beforeEach(() => {
  setLocale("en");
});

describe("connectionStatusLabel", () => {
  it("shows demo mode", () => {
    expect(connectionStatusLabel(status({ demo_mode: true }), false)).toBe(
      "Demo mode",
    );
  });

  it("shows demo ready", () => {
    expect(connectionStatusLabel(status({ demo_mode: true }), true)).toBe(
      "Demo — ready",
    );
  });

  it("shows unavailable off Windows", () => {
    expect(connectionStatusLabel(status(), false)).toBe("Camera off Windows");
  });

  it("shows no device on Windows", () => {
    expect(
      connectionStatusLabel(
        status({ os: "windows", camera_available: true }),
        false,
      ),
    ).toBe("No device");
  });

  it("shows connected on Windows", () => {
    expect(
      connectionStatusLabel(
        status({ os: "windows", camera_available: true }),
        true,
      ),
    ).toBe("Connected");
  });
});

describe("connectionHelpText", () => {
  it("stays short off Windows", () => {
    const text = connectionHelpText(status());
    expect(text).toContain("Windows");
    expect(text.toLowerCase()).toContain("demo");
    expect(text).not.toContain("Lmxptpif");
  });

  it("mentions USB on Windows", () => {
    const text = connectionHelpText(
      status({ os: "windows", camera_available: true }),
    );
    expect(text).toContain("USB-C");
    expect(text).toContain("PC (Tether)");
  });

  it("switches to French", () => {
    setLocale("fr");
    expect(connectionStatusLabel(status({ demo_mode: true }), false)).toBe(
      "Mode démo",
    );
  });
});

describe("connectionStatusColor", () => {
  it("marks connected as success", () => {
    expect(
      connectionStatusColor(
        status({ os: "windows", camera_available: true }),
        true,
      ),
    ).toBe("success");
  });

  it("marks unavailable as danger", () => {
    expect(connectionStatusColor(status(), false)).toBe("danger");
  });
});

describe("osLabel", () => {
  it("maps macos", () => {
    expect(osLabel("macos")).toBe("macOS");
  });
});
