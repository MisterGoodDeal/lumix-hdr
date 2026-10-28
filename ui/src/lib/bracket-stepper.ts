import type { CaptureProgress, CaptureState } from "../types";

export type StepStatus = "pending" | "current" | "completed" | "failed";

function frameFromState(state: CaptureState): number | null {
  switch (state.kind) {
    case "set_exposure":
    case "verify_exposure":
    case "trigger":
    case "wait_capture_done":
    case "download":
    case "verify_file":
      return state.frame;
    default:
      return null;
  }
}

export function bracketStepStatus(
  index: number,
  slotCount: number,
  progress: CaptureProgress | null,
): StepStatus {
  if (!progress || slotCount <= 0) {
    return "pending";
  }

  const { state } = progress;
  switch (state.kind) {
    case "idle":
    case "preparing":
      return "pending";
    case "restoring":
    case "merging":
    case "done":
      return "completed";
    case "aborted":
    case "failed":
      return "pending";
    default: {
      const frame = frameFromState(state);
      if (frame === null) {
        return "pending";
      }
      if (index < frame) {
        return "completed";
      }
      if (index === frame) {
        return "current";
      }
      return "pending";
    }
  }
}

export function formatRelativeEv(value: number): string {
  const sign = value > 0 ? "+" : "";
  if (Number.isInteger(value)) {
    return `${sign}${value} IL`;
  }
  const rounded = Math.round(value * 100) / 100;
  return `${sign}${rounded} IL`;
}

export function connectorFill(
  left: StepStatus,
  right: StepStatus,
): "full" | "half" | "empty" {
  if (left === "completed" && (right === "completed" || right === "failed")) {
    return "full";
  }
  if (left === "completed" && right === "current") {
    return "half";
  }
  if (left === "current") {
    return "half";
  }
  return "empty";
}
