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

/** Progression 0–1 à l’intérieur d’une vue (sous-étapes capture). */
function intraShotProgress(state: CaptureState): number {
  switch (state.kind) {
    case "set_exposure":
      return 0.12;
    case "verify_exposure":
      return 0.28;
    case "trigger":
      return 0.42;
    case "wait_capture_done":
      return 0.58;
    case "download":
      return 0.78;
    case "verify_file":
      return 0.94;
    default:
      return 0;
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

/**
 * Avancement 0–1 d’une vue : 0 (pas commencée), 0–1 (en cours), 1 (finie).
 * Sert aux barres de liaison et à la mini progress sous l’étape active.
 */
export function shotPhaseProgress(
  index: number,
  slotCount: number,
  progress: CaptureProgress | null,
): number {
  const status = bracketStepStatus(index, slotCount, progress);
  if (status === "completed") {
    return 1;
  }
  if (status === "pending" || status === "failed" || !progress) {
    return 0;
  }
  return intraShotProgress(progress.state);
}

/** Statut de l’étape finale « fusion HDR ». */
export function mergeStepStatus(
  progress: CaptureProgress | null,
): StepStatus {
  if (!progress) {
    return "pending";
  }

  const { state } = progress;
  switch (state.kind) {
    case "idle":
    case "preparing":
    case "aborted":
    case "restoring":
      return "pending";
    case "merging":
      return "current";
    case "done":
      return "completed";
    case "failed":
      return state.step === "merging" || /merge|fusion|hdr/i.test(state.step)
        ? "failed"
        : "pending";
    default:
      return "pending";
  }
}

/**
 * Progression fusion 0–1. Pendant `merging`, on mappe la jauge globale
 * (≈0.88 → 0.98) sur l’étape ; sinon 0 / 1 selon le statut.
 */
export function mergePhaseProgress(
  progress: CaptureProgress | null,
): number {
  const status = mergeStepStatus(progress);
  if (status === "completed") {
    return 1;
  }
  if (status !== "current" || !progress) {
    return 0;
  }
  const mapped = (progress.progress - 0.88) / 0.1;
  if (!Number.isFinite(mapped)) {
    return 0.15;
  }
  return Math.min(1, Math.max(0.08, mapped));
}

/**
 * Remplissage du trait entre l’étape `left` et la suivante.
 * Plein dès que la gauche est terminée — plus de demi-barre trompeuse
 * quand on passe à l’étape d’après.
 */
export function connectorRatio(leftPhase: number): number {
  return Math.min(1, Math.max(0, leftPhase));
}

/** @deprecated préférer connectorRatio — conservé pour compat tests. */
export function connectorFill(
  left: StepStatus,
  right: StepStatus,
): "full" | "half" | "empty" {
  if (left === "completed") {
    return "full";
  }
  if (left === "current") {
    return right === "pending" || right === "current" ? "half" : "half";
  }
  return "empty";
}

export function formatRelativeEv(value: number): string {
  const sign = value > 0 ? "+" : "";
  if (Number.isInteger(value)) {
    return `${sign}${value} IL`;
  }
  const rounded = Math.round(value * 100) / 100;
  return `${sign}${rounded} IL`;
}
