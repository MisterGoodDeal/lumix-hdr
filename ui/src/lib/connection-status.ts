import { t } from "../i18n";
import type { AppStatus } from "../types";

export type ConnectionChipColor =
  | "default"
  | "accent"
  | "success"
  | "warning"
  | "danger";

export function connectionStatusLabel(
  status: AppStatus,
  cameraConnected: boolean,
): string {
  if (status.demo_mode) {
    return cameraConnected ? t("conn.demoReady") : t("conn.demo");
  }
  if (!status.camera_available) {
    return t("conn.cameraOffWindows");
  }
  return cameraConnected ? t("conn.connected") : t("conn.noDevice");
}

/** Couleur Chip « Statuses » HeroUI pour l’état de connexion. */
export function connectionStatusColor(
  status: AppStatus,
  cameraConnected: boolean,
): ConnectionChipColor {
  if (status.demo_mode) {
    return cameraConnected ? "accent" : "warning";
  }
  if (!status.camera_available) {
    return "danger";
  }
  return cameraConnected ? "success" : "warning";
}

export function connectionHelpText(status: AppStatus): string {
  if (status.demo_mode) {
    return t("conn.helpDemo");
  }
  if (!status.camera_available) {
    return t("conn.helpNonWindows");
  }
  return t("conn.helpWindows");
}

export function osLabel(os: string): string {
  switch (os) {
    case "macos":
      return "macOS";
    case "windows":
      return "Windows";
    case "linux":
      return "Linux";
    default:
      return os;
  }
}
