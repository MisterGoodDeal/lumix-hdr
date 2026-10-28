import { check, type DownloadEvent, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

import { isTauri } from "./tauri";

export type UpdateProgress = {
  downloaded: number;
  total: number | null;
};

/** Pourcentage 0–100, ou null si la taille totale est inconnue. */
export function updateProgressPercent(progress: UpdateProgress): number | null {
  if (progress.total == null || progress.total <= 0) {
    return null;
  }
  return Math.min(100, Math.round((100 * progress.downloaded) / progress.total));
}

export type AvailableUpdate = {
  version: string;
  body: string | null;
  date: string | null;
};

export async function checkForAppUpdate(): Promise<AvailableUpdate | null> {
  if (!isTauri()) {
    return null;
  }
  const update = await check();
  if (!update) {
    return null;
  }
  return {
    version: update.version,
    body: update.body ?? null,
    date: update.date ?? null,
  };
}

export async function downloadAndInstallAppUpdate(
  onProgress?: (progress: UpdateProgress) => void,
): Promise<void> {
  if (!isTauri()) {
    throw new Error("Updater disponible uniquement dans l’app Tauri");
  }
  const update = await check();
  if (!update) {
    throw new Error("Aucune mise à jour disponible");
  }
  await installUpdate(update, onProgress);
  await relaunch();
}

async function installUpdate(
  update: Update,
  onProgress?: (progress: UpdateProgress) => void,
): Promise<void> {
  let downloaded = 0;
  let total: number | null = null;

  await update.downloadAndInstall((event: DownloadEvent) => {
    switch (event.event) {
      case "Started":
        total = event.data.contentLength ?? null;
        downloaded = 0;
        onProgress?.({ downloaded, total });
        break;
      case "Progress":
        downloaded += event.data.chunkLength;
        onProgress?.({ downloaded, total });
        break;
      case "Finished":
        onProgress?.({ downloaded, total });
        break;
    }
  });
}
