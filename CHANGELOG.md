# Journal des modifications

## 0.1.0 — 2026-10-27

### Ajouté

- Squelette de l’application Tauri 2 avec interface React, HeroUI et thème sombre.
- Configuration JSON versionnée, mode démo (réglage et `LUMIX_HDR_DEMO=1`).
- Journaux `tracing` (fichiers roulants) et hook de panic avec dialogue.
- Crate `hdr-core` : AlignMTB, MergeMertens, MergeDebevec optionnel, JPEG/TIFF atomiques, copie EXIF.
- Commande Tauri « fusionner un dossier » avec progression et annulation, écran Résultat.
- Intégration continue (fmt, clippy, tests Rust, OpenCV, lint / typecheck / tests UI).
