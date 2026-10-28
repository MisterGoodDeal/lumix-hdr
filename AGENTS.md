# LumixHDR — reprise (Cursor)

Lis ce fichier en premier. Réponds en **français**. Travaille **par phases** : ne commence pas la suivante tant que la courante n’est pas verte (fmt, clippy `-D warnings`, `cargo test --all`, `yarn lint`, `yarn typecheck`, `yarn test`).

## Où on en est (2026-10-28)

Branche : `main` (poussée). Version `0.1.0`.

| Phase | Statut |
| --- | --- |
| 1 — Squelette Tauri 2 + React 19 + HeroUI v3 + Yarn | **faite** |
| 2 — `crates/hdr-core` + fusion d’un dossier JPEG | **faite** (tests synthétiques + dossier GH6 OK Windows) |
| 3 — `capture` + `MockCamera` + écran Prise de vue | **faite** (MockCamera only ; mode démo requis) |
| 4 — SDK LUMIX (`lumix-sys` / `LumixCamera` / `lumix-probe`) | plus tard |
| 5 — Live view UI | plus tard |
| 6 — Historique, release, comparaison avant/après | plus tard |

**Le GH6 branché en USB n’est pas vu : ce n’est pas un bug.** `camera_available` vaut `cfg!(windows)` seulement. Aucun SDK n’est chargé avant la phase 4.

## Première chose sur Windows

1. `git pull`
2. LLVM + OpenCV 4 via vcpkg déjà OK sur cette machine (`VCPKG_ROOT`, `OpenCV_DIR`, `LIBCLANG_PATH`, `VCPKGRS_TRIPLET`).
3. `yarn install` puis `cargo test --all` / `yarn tauri dev`
4. Mode démo → Connexion → Connecter → **Prise de vue** (MockCamera). Fusion manuelle = écran **Résultat**.

## Phase 3 (faite)

Crates `lumix` + `capture`. Commandes Tauri : `preview_bracket_plan`, `start_capture`, `cancel_capture` + événement `capture-progress`. Sessions sous `app_data/sessions/`.

## Phase 4 (prochain travail)

SDK LUMIX réel (`lumix-sys` / `LumixCamera` / `lumix-probe`). Ne pas inventer l’API.

## Interdits

- Aucun `unwrap()` / `expect()` hors tests. Les crates prod ont `#![deny(clippy::unwrap_used, clippy::expect_used)]`.
- **Ne jamais inventer l’API du SDK LUMIX.** Headers dans `vendor/lumix-sdk/` (gitignoré). L’API est du **C++ WINAPI** avec arguments par défaut → bindgen plus tard via un shim `extern "C"` (`cc`), pas un bindgen direct sur les `.h`.
- **Ne jamais committer** `vendor/` ni la DLL.
- Tauri **2.12.x**, pas Tauri 3 alpha. OpenCV **4**, pas 5. TypeScript **5.9.x**.
- UI 100 % français, thème sombre, composants HeroUI v3 (pas de CSS maison). Toasts + `ErrorDetails` repliable.

## Stack épinglée

- Rust 1.99.1 (`rust-toolchain.toml`), édition 2021
- Workspace : `src-tauri`, `crates/hdr-core`, `crates/lumix`, `crates/capture`
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

`get_app_status`, `set_demo_mode`, `set_log_level`, `get_log_dir`, `set_fusion_options`, `list_jpeg_folder`, `merge_image_folder`, `cancel_fusion`, `open_path`, `preview_bracket_plan`, `start_capture`, `cancel_capture`. Événements `fusion-progress` (`f32`) et `capture-progress` (`CaptureProgress`). Permissions dans `src-tauri/permissions/commands.toml`.

Mode démo : réglage persisté **ou** `LUMIX_HDR_DEMO=1`. Prise de vue phase 3 exige le mode démo (MockCamera).

## Git

- Commits Gitmoji (`feat: ✨ …`), petits, un sujet chacun. Ne commit / push que si l’utilisateur le demande, sauf `/commit-changed-files`.
- Yarn workspaces, lockfile `yarn.lock` commité.
