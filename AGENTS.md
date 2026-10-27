# LumixHDR — reprise (Cursor)

Lis ce fichier en premier. Réponds en **français**. Travaille **par phases** : ne commence pas la suivante tant que la courante n’est pas verte (fmt, clippy `-D warnings`, `cargo test --all`, `yarn lint`, `yarn typecheck`, `yarn test`).

## Où on en est (2026-10-27)

Branche : `main` (poussée). Version `0.1.0`.

| Phase | Statut |
| --- | --- |
| 1 — Squelette Tauri 2 + React 19 + HeroUI v3 + Yarn | **faite** |
| 2 — `crates/hdr-core` + fusion d’un dossier JPEG | **faite** (tests synthétiques OK sur macOS) |
| 3 — `capture` + `MockCamera` + écran Prise de vue | **à faire ensuite** |
| 4 — SDK LUMIX (`lumix-sys` / `LumixCamera` / `lumix-probe`) | plus tard |
| 5 — Live view UI | plus tard |
| 6 — Historique, release, comparaison avant/après | plus tard |

**Le GH6 branché en USB n’est pas vu : ce n’est pas un bug.** `camera_available` vaut `cfg!(windows)` seulement. Aucun SDK n’est chargé avant la phase 4.

## Première chose sur Windows

1. `git pull`
2. Installer **LLVM** (libclang) et **OpenCV 4** via vcpkg, voir [`docs/architecture.md`](docs/architecture.md) :

```text
vcpkg install opencv4[jpeg,png,tiff]:x64-windows-static-md
```

   Puis `OpenCV_DIR` → `installed\x64-windows-static-md\share\opencv4` et `LIBCLANG_PATH` vers le dossier de `libclang.dll`.

3. `yarn install` (Yarn 1.22.22, **pas pnpm**)
4. `cargo test --all` puis `yarn tauri dev`
5. Vérifier l’écran **Résultat** avec un vrai dossier de JPEG GH6 (AlignMTB + Mertens). Si OpenCV / vcpkg bloque, corrige ça avant la phase 3.

## Phase 3 (prochain travail)

Crates `lumix` + `capture` (pas `lumix-sys` : ça c’est la phase 4).

- Trait `Camera` + **`MockCamera`** uniquement. Images synthétiques dont la luminosité suit l’expo simulée. Injection d’erreurs (busy, timeout, déco).
- Machine à états : `Idle → Preparing → (SetExposure → VerifyExposure → Trigger → WaitCaptureDone → Download → VerifyFile) × N → Restoring → Merging → Done`, plus `Aborted` / `Failed { step, error }`.
- Plan : N ∈ {3,5,7,9}, écart IL ∈ {1/3 … 3}, centre −3…+3 IL. Défaut : mode **M**, ISO et ouverture fixes, on varie la **vitesse**. Refuser le plan si une vitesse sort de la plage.
- Sauvegarder les réglages avant, **restaurer toujours** (succès / échec / annulation).
- Dossier `AAAA-MM-JJ_HHMMSS_<nom>/` + originaux + JPEG fusionné via `hdr-core` + `manifest.json` (plan, hashes SHA-256, version, erreurs). Jamais d’écrasement.
- UI **Prise de vue** (HeroUI, français) : réglages rapides, aperçu du plan, Déclencher / Annuler, Progress. Live view réelle = phase 5 ; en démo, aperçu factice suffit.
- Espace = déclencher, Échap = annuler.
- Tests : calcul du plan, transitions, restauration après échec, annulation, déco simulée.

## Interdits

- Aucun `unwrap()` / `expect()` hors tests. Les crates prod ont `#![deny(clippy::unwrap_used, clippy::expect_used)]`.
- **Ne jamais inventer l’API du SDK LUMIX.** Headers dans `vendor/lumix-sdk/` (gitignoré). L’API est du **C++ WINAPI** avec arguments par défaut → bindgen plus tard via un shim `extern "C"` (`cc`), pas un bindgen direct sur les `.h`.
- **Ne jamais committer** `vendor/` ni la DLL.
- Tauri **2.12.x**, pas Tauri 3 alpha. OpenCV **4**, pas 5. TypeScript **5.9.x**.
- UI 100 % français, thème sombre, composants HeroUI v3 (pas de CSS maison). Toasts + `ErrorDetails` repliable.

## Stack épinglée

- Rust 1.99.1 (`rust-toolchain.toml`), édition 2021
- Workspace : `src-tauri`, `crates/hdr-core`
- `opencv` 0.101.1, features `clang-runtime`, `imgcodecs`, `imgproc`, `photo`
- Factories OpenCV 4.14 : `create_align_mtb_def`, `create_merge_mertens`, `create_merge_debevec`, `create_tonemap_reinhard_def` — **pas** `Type::create()`
- Encodage fichier : `imencode` (un `.tmp` n’a pas d’écrivain OpenCV)
- UI : Vite 8, React 19, Tailwind 4, Zustand, `@tauri-apps/plugin-dialog`
- Identifier Tauri : `app.lumixhdr.desktop`

## HeroUI v3 (pièges déjà vus)

- `Switch` : `Switch.Content` enveloppe `Switch.Control` **et** le libellé, sinon le clic est avalé.
- Progression = `ProgressBar` (`Track` / `Fill` / `Output`), pas `Progress`.
- `Button` : `onPress`, pas `onClick`.

## Commandes Tauri déjà là

`get_app_status`, `set_demo_mode`, `set_log_level`, `get_log_dir`, `set_fusion_options`, `list_jpeg_folder`, `merge_image_folder`, `cancel_fusion`, `open_path`. Événement `fusion-progress` (`f32`). Permissions dans `src-tauri/permissions/commands.toml`.

Mode démo : réglage persisté **ou** `LUMIX_HDR_DEMO=1`.

## Git

- Commits Gitmoji (`feat: ✨ …`), petits, un sujet chacun. Ne commit / push que si l’utilisateur le demande, sauf `/commit-changed-files`.
- Yarn workspaces, lockfile `yarn.lock` commité.
