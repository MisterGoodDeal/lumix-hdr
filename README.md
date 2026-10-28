# LumixHDR

[![CI](https://github.com/MisterGoodDeal/lumix-hdr/actions/workflows/ci.yml/badge.svg)](https://github.com/MisterGoodDeal/lumix-hdr/actions/workflows/ci.yml)

App de bureau pour bracketter un **Panasonic LUMIX GH6** en USB, fusionner les expo en HDR localement, et garder un historique des séances.

Pilotage caméra = **Windows** + SDK LUMIX. La fusion JPEG marche partout (et en mode démo sans boîtier).

## Ce que ça fait

- **Studio** — connexion PnP, Live View, réglages ISO / f / vitesse, bracketing ±EV puis fusion
- **Résultat** — fusion d’un dossier de JPEG déjà pris (AlignMTB, Mertens ou Debevec)
- **Historique** — cartes de séances, comparaison avant/après, suppression
- **Réglages** — démo, thème, langue FR/EN, dossiers, mises à jour depuis GitHub Releases

## Prérequis (caméra)

- Windows 10/11 64 bits
- GH6 à jour, USB-C **direct** (pas de hub / rallonge — Panasonic ne les gère pas)
- Mode USB du boîtier : **PC (Tether)** + drivers OK  
  Tant que ce n’est pas le cas, l’app ne voit rien : ce n’est pas un bug d’interface.

Sans boîtier : active le **mode démo** dans Réglages (ou `LUMIX_HDR_DEMO=1`).

## Installer

Les builds Windows (installeur NSIS) sortent sur les [Releases](https://github.com/MisterGoodDeal/lumix-hdr/releases) à chaque tag `vX.Y.Z`. Une fois installé, Réglages → *Vérifier les mises à jour* pour passer à la suivante.

## Développement

Rust 1.99.1, Node 22, Yarn 1, OpenCV 4 + LLVM. Sur Windows : vcpkg OpenCV + SDK LUMIX dans `vendor/lumix-sdk/` (gitignoré). Détail dans [`AGENTS.md`](AGENTS.md) et [`docs/architecture.md`](docs/architecture.md).

```bash
yarn install
yarn tauri dev
```

Gates locaux : `cargo fmt` / `clippy -D warnings` / `cargo test --all`, puis `yarn lint` / `typecheck` / `test`.

Release / secrets updater : [`docs/release.md`](docs/release.md).

## Licence

MIT OR Apache-2.0 — sauf le SDK Panasonic (`vendor/`), qui reste sous sa propre licence et n’est jamais versionné ici.
