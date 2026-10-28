import type {
  BracketPlan,
  BracketRequest,
  CaptureProgress,
  CaptureSessionResult,
} from "../types";
import { isTauri, parseAppError } from "./tauri";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri()) {
    throw new Error("Commande Tauri indisponible hors de l'application.");
  }
  return invoke<T>(command, args);
}

export async function previewBracketPlan(
  request: BracketRequest,
): Promise<BracketPlan> {
  if (!isTauri()) {
    return mockPlan(request);
  }
  return call<BracketPlan>("preview_bracket_plan", { request });
}

export async function startCapture(
  request: BracketRequest,
): Promise<CaptureSessionResult> {
  if (!isTauri()) {
    await new Promise((resolve) => {
      window.setTimeout(resolve, 400);
    });
    return {
      session_dir: `/demo/sessions/2026-01-01_120000_${request.session_name}`,
      manifest_path: `/demo/sessions/2026-01-01_120000_${request.session_name}/manifest.json`,
      fused_jpeg: `/demo/sessions/2026-01-01_120000_${request.session_name}/hdr-fused.jpg`,
      state: { kind: "done" },
      originals: Array.from(
        { length: request.frame_count },
        (_, index) =>
          `/demo/sessions/2026-01-01_120000_${request.session_name}/frame-${index + 1}.jpg`,
      ),
    };
  }
  return call<CaptureSessionResult>("start_capture", { request });
}

export async function cancelCapture(): Promise<void> {
  if (!isTauri()) {
    return;
  }
  await call<void>("cancel_capture");
}

export async function onCaptureProgress(
  handler: (value: CaptureProgress) => void,
): Promise<UnlistenFn> {
  if (!isTauri()) {
    return () => undefined;
  }
  return listen<CaptureProgress>("capture-progress", (event) => {
    handler(event.payload);
  });
}

export function captureStateLabel(progress: CaptureProgress | null): string {
  if (!progress) {
    // Lazy import avoided — caller UI already i18n-aware; keep neutral.
    return "…";
  }
  return progress.message || progress.state.kind;
}

export { parseAppError };

function mockPlan(request: BracketRequest): BracketPlan {
  const half = (request.frame_count - 1) / 2;
  const slots = Array.from({ length: request.frame_count }, (_, i) => {
    const relative_ev = request.center_ev + (i - half) * request.step_ev;
    return {
      index: i,
      relative_ev,
      shutter_seconds: 1 / 60,
      shutter_label: "1/60",
    };
  });
  return {
    request,
    base: {
      mode: "M",
      aperture: 2.8,
      iso: 200,
      shutter_seconds: 1 / 60,
      shutter_label: "1/60",
    },
    slots,
  };
}
