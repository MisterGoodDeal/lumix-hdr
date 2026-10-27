# LumixHDR

Application de bureau pour le bracketing d’exposition d’un Panasonic LUMIX DC-GH6 et la fusion HDR locale. Cette version (0.1.0) pose le squelette et la fusion OpenCV : interface sombre, journaux, mode démo, fusion d’un dossier de JPEG sans appareil. Le pilotage de l’appareil arrivera dans les phases suivantes.

## Prérequis

- **Windows 10/11 64 bits** pour le contrôle de l’appareil (SDK LUMIX USB). Le module de fusion sera testé sur toutes les plateformes ; le pilotage caméra refuse de démarrer ailleurs, avec un message clair.
- Appareil **LUMIX DC-GH6**, firmware à jour.
- Câble **USB-C direct** : pas de hub USB ni de rallonge (non supportés par Panasonic).
- Pour le développement : Rust 1.99.1 (via `rust-toolchain.toml`), Node 22, Yarn 1, OpenCV 4 + LLVM/libclang (voir [`docs/architecture.md`](docs/architecture.md)).

## Installation (développement)

```bash
yarn install
yarn tauri dev
```

## Utilisation rapide

1. Lancer l’application.
2. L’écran **Connexion** indique si le pilotage caméra est disponible (Windows uniquement).
3. Activer le **mode démo** dans Réglages pour parcourir l’interface sans appareil.
4. L’écran **Résultat** fusionne un dossier de JPEG (AlignMTB + Mertens, Debevec en option) sans appareil.
5. Les écrans Prise de vue et Historique arriveront plus tard.

## Mode démo

Le mode démo s’active :

- dans **Réglages** (interrupteur persisté) ;
- ou avec la variable d’environnement `LUMIX_HDR_DEMO=1` (prioritaire sur le fichier de configuration).

En mode démo, le bouton **Connecter** confirme par un toast qu’aucune connexion à un appareil réel n’est tentée.

## Journaux

Les journaux roulants sont écrits dans le dossier de logs de l’application. Le chemin est affiché dans Réglages.
