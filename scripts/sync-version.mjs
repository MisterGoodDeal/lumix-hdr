#!/usr/bin/env node
/**
 * Aligne les versions Cargo / package.json / tauri.conf sur une SemVer
 * (ex. depuis un tag Git v0.2.0 → 0.2.0).
 *
 * Usage: node scripts/sync-version.mjs 0.2.0
 *    ou: VERSION=0.2.0 node scripts/sync-version.mjs
 */
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const raw = (process.argv[2] ?? process.env.VERSION ?? "").trim();
const version = raw.replace(/^v/i, "");

if (!/^\d+\.\d+\.\d+([+-][0-9A-Za-z.-]+)?$/.test(version)) {
  console.error(
    `Version invalide: "${raw}". Attendu SemVer (ex. 0.2.0 ou v0.2.0).`,
  );
  process.exit(1);
}

function replaceInFile(relPath, replacer) {
  const path = join(root, relPath);
  const before = readFileSync(path, "utf8");
  const after = replacer(before);
  if (after === before) {
    // Déjà à jour, ou motif manquant : on vérifie que la version cible est présente.
    if (!before.includes(`"${version}"`) && !before.includes(`= "${version}"`)) {
      console.error(`Aucune substitution dans ${relPath}`);
      process.exit(1);
    }
    console.log(`ok ${relPath} (déjà ${version})`);
    return;
  }
  writeFileSync(path, after);
  console.log(`ok ${relPath} → ${version}`);
}

replaceInFile("Cargo.toml", (text) =>
  text.replace(/^version = "[^"]+"/m, `version = "${version}"`),
);

replaceInFile("package.json", (text) =>
  text.replace(/"version": "[^"]+"/, `"version": "${version}"`),
);

replaceInFile("ui/package.json", (text) =>
  text.replace(/"version": "[^"]+"/, `"version": "${version}"`),
);

replaceInFile("src-tauri/tauri.conf.json", (text) =>
  text.replace(/"version": "[^"]+"/, `"version": "${version}"`),
);

console.log(`Versions synchronisées sur ${version}`);
