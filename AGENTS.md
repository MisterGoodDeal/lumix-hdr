# LumixHDR — reprise (Cursor)

Lis ce fichier en premier. Réponds en **français**. Travaille **par phases** : ne commence pas la suivante tant que la courante n’est pas verte (fmt, clippy `-D warnings`, `cargo test --all`, `yarn lint`, `yarn typecheck`, `yarn test`).

## Où on en est (2026-10-28)

Branche : `main`. Version `0.1.0`.

| Phase | Statut |
| --- | --- |
| 1 — Squelette Tauri 2 + React 19 + HeroUI v3 + Yarn | **faite** |
| 2 — `crates/hdr-core` + fusion d’un dossier JPEG | **faite** |
| 3 — `capture` + `MockCamera` + écran Prise de vue | **faite** |
| 4 — SDK LUMIX (`lumix-sys` / `LumixCamera` / `lumix-probe`) | **faite** (Windows + `vendor/lumix-sdk`) |
| 5 — Live view UI + page Studio | **faite** |
| 6 — Historique (cartes + détail avant/après) | **faite** |
| 7 — Release / packaging (NSIS + updater GitHub) | **en cours** |

**Le GH6 branché en USB n’est pas vu tant qu’il n’est pas en mode PC (Tether) / drivers OK : ce n’est pas un bug UI.** `camera_available` vaut `cfg!(windows)`. Hors démo, Studio appelle le SDK ; sans appareil PnP la commande échoue clairement. Débranchement → event `camera-disconnected`.

## Première chose sur Windows

1. `git pull`
2. LLVM **20+** (VS 2026 / images `windows-2025` : Clang 18 → erreur STL1000) + OpenCV 4 via vcpkg (`VCPKG_ROOT`, `OpenCV_DIR`, `LIBCLANG_PATH`, `VCPKGRS_TRIPLET`).
3. SDK LUMIX dans `vendor/lumix-sdk/` (**gitignoré**, ne jamais committer) : `Library/Lmxptpif.dll`, `Lmxptpif.lib`, headers. Optionnel : `LUMIX_SDK_DIR`.
4. `yarn install` puis `cargo test --all` / `yarn tauri dev`
5. Mode démo → MockCamera **ou** hors démo → Studio → Connecter (PnP) → Live View + bracketing.

### Smoke test SDK

```text
cargo run -p lumix-probe
```

`Lmxptpif.dll` doit être dans le `PATH` ou à côté du binaire. Le `build.rs` de `src-tauri` copie la DLL vers `target/<profile>/` si le SDK est trouvé.

## Phase 4 (faite)

- `crates/lumix-sys` : shim C++ `extern "C"` (`cc` + MSVC), pas de bindgen direct.
- `crates/lumix` feature `sdk` : `LumixCamera` (trait `Camera`), mapping SS via `SS_Get_Capability` + encodage doc/sample.
- `crates/lumix-probe` : Init → PnP → Open_Session → lit ISO/SS/F/Mode → Close.
- Tauri : fabrique Mock vs Lumix, `list_cameras` / `connect_camera`, capture hors démo sous Windows.

## Phase 5 (faite)

- Page **Studio** (connexion + Live View + bracketing) : preview centrale, overlays +/- ISO/f/SS.
- Live View : `Ctrl_LiveView_Start/Stop` + `Get_LiveView_data` (JPEG), event `liveview-frame`.
- `config.json` : `capture` + `camera` persistés ; appliqués à la connexion.
- `disconnect_camera` + surveillance PnP (`camera-disconnected`).

## Interdits

- Aucun `unwrap()` / `expect()` hors tests. Les crates prod ont `#![deny(clippy::unwrap_used, clippy::expect_used)]`.
- **Ne jamais inventer l’API du SDK LUMIX.** Headers dans `vendor/lumix-sdk/` (gitignoré). Shim `extern "C"` uniquement.
- **Ne jamais committer** `vendor/`, la DLL, ni `test_ressources/`.
- Tauri **2.12.x**, pas Tauri 3 alpha. OpenCV **4**, pas 5. TypeScript **5.9.x**.
- UI i18n FR/EN (`ui/src/i18n`), préférence `system|en|fr` (défaut système → anglais si pas fr). Thème `system|dark|light` (`ui/src/theme`). HeroUI v3. Toasts + `ErrorDetails` repliable.

## Stack épinglée

- Rust 1.99.1 (`rust-toolchain.toml`), édition 2021
- Workspace : `src-tauri`, `crates/hdr-core`, `crates/lumix`, `crates/capture`, `crates/lumix-sys`, `crates/lumix-probe`
- `opencv` 0.101.1, features `clang-runtime`, `imgcodecs`, `imgproc`, `photo`
- Factories OpenCV 4.14 : `create_align_mtb_def`, `create_merge_mertens`, `create_merge_debevec`, `create_tonemap_reinhard_def` — **pas** `Type::create()`
- Encodage fichier : `imencode` (un `.tmp` n’a pas d’écrivain OpenCV)
- UI : Vite 8, React 19, Tailwind 4, Zustand, `@tauri-apps/plugin-dialog`
- Identifier Tauri : `app.lumixhdr.desktop`

## HeroUI v3 (pièges déjà vus)

- `Switch` : `Switch.Content` enveloppe `Switch.Control` **et** le libellé, sinon le clic est avalé.
- Progression = `ProgressBar` (`Track` / `Fill` / `Output`), pas `Progress`.
- `Button` : `onPress`, pas `onClick`.

## Commandes Tauri

`get_app_status`, `set_demo_mode`, `set_log_level`, `get_log_dir`, `set_fusion_options`, `set_locale`, `set_theme`, `set_sessions_dir`, `list_sessions`, `get_session`, `delete_session`, `list_jpeg_folder`, `merge_image_folder`, `cancel_fusion`, `open_path`, `preview_bracket_plan`, `start_capture`, `cancel_capture`, `list_cameras`, `connect_camera`, `disconnect_camera`, `get_camera_settings`, `set_camera_settings`, `set_capture_defaults`, `start_live_view`, `stop_live_view`. Événements `fusion-progress` / `capture-progress` / `liveview-frame` / `camera-disconnected`.

### Phase 6 (historique)

- `list_sessions` / `get_session` : lit les `manifest.json` du dossier séances.
- UI : grille de [Cards HeroUI](https://heroui.com/en/docs/react/components/card) + détail avec slider avant/après (expo centrale vs HDR).

Mode démo : réglage persisté **ou** `LUMIX_HDR_DEMO=1` → `MockCamera`. Hors démo (Windows) → `LumixCamera`.

## Phase 7 (release)

- Tag `vX.Y.Z` → workflow `Release` (Windows NSIS + `latest.json` updater).
- Détails secrets / signing / SDK chiffré : [`docs/release.md`](docs/release.md).
- SDK CI : `ci/lumix-sdk-library.zip.enc` + secret `LUMIX_SDK_PASSPHRASE` (pas de zip en secret : limite 64 Ko).
- UI : Réglages → vérifier / installer (plugins `updater` + `process`).
- Overlay packaging : `src-tauri/tauri.release.conf.json` (DLL + artifacts updater).
- Sync versions : `yarn sync-version 0.2.0` (`scripts/sync-version.mjs`).

## Git

- Commits Gitmoji (`feat: ✨ …`), petits, un sujet chacun. Ne commit / push que si l’utilisateur le demande, sauf `/commit-changed-files`.
- Yarn workspaces, lockfile `yarn.lock` commité.
