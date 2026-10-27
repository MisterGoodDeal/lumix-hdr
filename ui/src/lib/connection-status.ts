import type { AppStatus } from "../types";

export function connectionStatusLabel(
  status: AppStatus,
  demoConnected: boolean,
): string {
  if (status.demo_mode) {
    return demoConnected ? "Mode démo — connecté" : "Mode démo";
  }
  if (!status.camera_available) {
    return "Pilotage caméra indisponible";
  }
  return "Aucun appareil";
}

export function connectionHelpText(status: AppStatus): string {
  if (status.demo_mode) {
    return "Le mode démo permet de parcourir l'interface sans appareil. Aucune communication avec un GH6 n'est établie.";
  }
  if (!status.camera_available) {
    return "Le pilotage du Panasonic LUMIX DC-GH6 est réservé à Windows 10/11 64 bits. Branchez l'appareil avec un câble USB-C direct, sans hub ni rallonge, firmware à jour, mode USB « PC (Tether) ».";
  }
  return "Branchez le GH6 en USB-C direct (sans hub ni rallonge), firmware à jour, mode USB « PC (Tether) ».";
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
