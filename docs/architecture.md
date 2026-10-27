# Architecture — fusion HDR (`hdr-core`)

LumixHDR sépare le pilotage caméra (phases 3–4, Windows + SDK LUMIX) de la fusion d’images, testable sur toutes les plateformes.

```
Dossier de JPEG
        │
        ▼
 jpeg_files_in_dir + ImageSource::decode
        │
        ▼
 AlignMTB (optionnel)
        │
        ├─ MergeMertens (défaut, poids 1/1/1)
        └─ MergeDebevec + CalibrateDebevec + TonemapReinhard|Mantiuk
        │
        ▼
 JPEG q=95 (EXIF recopié)  +  TIFF 16 bits optionnel
 écriture atomique (fichier temporaire + rename)
```

La crate [`crates/hdr-core`](../crates/hdr-core) n’a aucune dépendance Tauri. L’application appelle `fuse_folder` depuis `spawn_blocking`, avec un `AtomicBool` d’annulation et un callback de progression (`fusion-progress`).

## OpenCV

- Crate Rust : `opencv` **0.101.1**, features `clang-runtime`, `imgcodecs`, `imgproc`, `photo` (le module `core` vient avec).
- Bibliothèque C++ : **OpenCV 4** (pas OpenCV 5, actuellement le défaut Homebrew `opencv`).
- Version de référence CI Linux : paquet `libopencv-dev` d’Ubuntu.
- Version de référence Windows : vcpkg `opencv4[jpeg,png,tiff]` (modules core / imgproc / imgcodecs / photo inclus), triplet `x64-windows-static-md`.
- Version locale macOS utilisée pour le développement : Homebrew `opencv@4` (4.14.x).

Ne pas lier l’intégralité de `pkg-config opencv4` : trop de modules (DNN, VTK…) tirent protobuf, OpenEXR, etc. Pour un build local ciblé :

```bash
export OPENCV_LINK_LIBS=opencv_core,opencv_imgproc,opencv_imgcodecs,opencv_photo
export OPENCV_LINK_PATHS="$(brew --prefix opencv@4)/lib"
export OPENCV_INCLUDE_PATHS="$(brew --prefix opencv@4)/include/opencv4"
export PKG_CONFIG_PATH="$(brew --prefix opencv@4)/lib/pkgconfig:${PKG_CONFIG_PATH:-}"
export LIBCLANG_PATH="$(brew --prefix llvm)/lib"
```

## Installation locale

### macOS

```bash
brew install opencv@4 llvm pkg-config
```

`opencv@4` est keg-only. LLVM aussi : `LIBCLANG_PATH` est requis par bindgen (`clang-runtime` charge `libclang` au build).

### Ubuntu

```bash
sudo apt-get install -y clang libclang-dev libopencv-dev pkg-config
```

### Windows

Installer LLVM (libclang) puis, avec vcpkg :

```text
vcpkg install opencv4[jpeg,png,tiff]:x64-windows-static-md
```

Renseigner `OpenCV_DIR` vers `installed/x64-windows-static-md/share/opencv4` et `LIBCLANG_PATH` vers le dossier contenant `libclang.dll`.

## Commandes Tauri (phase 2)

| Commande | Rôle |
| --- | --- |
| `list_jpeg_folder` | Liste les `.jpg`/`.jpeg` d’un dossier |
| `merge_image_folder` | Fusion hors thread UI, persiste les options |
| `cancel_fusion` | Pose le drapeau d’annulation coopérative |
| `set_fusion_options` | Enregistre alignement / méthode / formats |
| `open_path` | Ouvre le fichier ou le dossier dans l’explorateur |

L’aperçu JPEG utilise le protocole `asset:` (`convertFileSrc`).
