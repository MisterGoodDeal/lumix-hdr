# Release LumixHDR (phase 7)

Publication via **tag Git** `vX.Y.Z` sur
[MisterGoodDeal/lumix-hdr](https://github.com/MisterGoodDeal/lumix-hdr).
Le workflow [`.github/workflows/release.yml`](../.github/workflows/release.yml)
produit l’installeur **NSIS Windows**, sa signature updater (`.sig`) et
`latest.json` pour les mises à jour in-app.

## Secrets GitHub Actions

| Secret | Rôle |
| --- | --- |
| `TAURI_SIGNING_PRIVATE_KEY` | Contenu de `.secrets/tauri-updater.key` (jamais committer) |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Mot de passe de la clé (vide si générée avec `--ci`) |
| `LUMIX_SDK_PASSPHRASE` | Passphrase de `ci/lumix-sdk-library.zip.enc` |

Les secrets GitHub sont **plafonnés à ~64 Ko** : impossible d’y coller le zip du SDK
(~300 Ko rien que pour `Library/`). On versionne une archive **chiffrée**
(`ci/lumix-sdk-library.zip.enc`) et on ne met en secret que la passphrase.

La **clé publique** updater est dans `src-tauri/tauri.conf.json`
(`plugins.updater.pubkey`). Si tu régénères la paire, mets à jour la pubkey
**et** le secret privé — sinon les clients déjà installés ne pourront plus
vérifier les updates.

### Générer / enregistrer la clé updater

```powershell
yarn tauri signer generate -w .secrets/tauri-updater.key
# Coller le contenu de .secrets/tauri-updater.key dans TAURI_SIGNING_PRIVATE_KEY
# Coller le .pub dans plugins.updater.pubkey de tauri.conf.json
```

### Packer le SDK pour la CI

```powershell
.\scripts\pack-lumix-sdk.ps1
# → ci/lumix-sdk-library.zip.enc  (à committer)
# → .secrets/lumix-sdk.passphrase (à coller dans LUMIX_SDK_PASSPHRASE, jamais committer)
```

Tu peux supprimer l’ancien secret `LUMIX_SDK_ZIP_BASE64` s’il existe.

## Couper une release

1. `main` à jour, gates verts, `ci/lumix-sdk-library.zip.enc` présent.
2. `git tag v0.2.0` puis `git push origin v0.2.0`.
3. Le job **Release** synchronise les versions (`scripts/sync-version.mjs`),
   build OpenCV + SDK, publie la Release GitHub.
4. Dans l’app : **Réglages → Vérifier les mises à jour → Installer**.

Le endpoint updater :
`https://github.com/MisterGoodDeal/lumix-hdr/releases/latest/download/latest.json`.

## Build local (installeur)

```powershell
$env:TAURI_SIGNING_PRIVATE_KEY = Get-Content .secrets\tauri-updater.key -Raw
yarn tauri build -- --bundles nsis --config src-tauri/tauri.release.conf.json
```

Prérequis : `vendor/lumix-sdk/`, OpenCV vcpkg, **LLVM/Clang ≥ 20** (voir `AGENTS.md`).
Sur les runners `windows-2025` / VS 2026, Clang 18 fait échouer `opencv-binding-generator`
(`STL1000`). La CI utilise LLVM 20.1.8 + cache vcpkg fichiers (`actions/cache`) —
le backend `x-gha` de vcpkg a été retiré.
