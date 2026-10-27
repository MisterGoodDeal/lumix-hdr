# LumixHDR

Application de bureau pour le bracketing d’exposition d’un Panasonic LUMIX DC-GH6 et la fusion HDR locale. Cette version (0.1.0) pose le squelette : interface sombre, journaux, mode démo. Le pilotage de l’appareil et la fusion arriveront dans les phases suivantes.

## Prérequis

- **Windows 10/11 64 bits** pour le contrôle de l’appareil (SDK LUMIX USB). Le module de fusion sera testé sur toutes les plateformes ; le pilotage caméra refuse de démarrer ailleurs, avec un message clair.
- Appareil **LUMIX DC-GH6**, firmware à jour.
- Câble **USB-C direct** : pas de hub USB ni de rallonge (non supportés par Panasonic).
- Pour le développement : Rust 1.99.1 (via `rust-toolchain.toml`), Node 22, Yarn 1.

## Installation (développement)

```bash
yarn install
yarn tauri dev
```

## Utilisation rapide

1. Lancer l’application.
2. L’écran **Connexion** indique si le pilotage caméra est disponible (Windows uniquement).
3. Activer le **mode démo** dans Réglages pour parcourir l’interface sans appareil.
4. Les écrans Prise de vue, Résultat et Historique sont présents mais pas encore disponibles.

## Mode démo

Le mode démo s’active :

- dans **Réglages** (interrupteur persisté) ;
- ou avec la variable d’environnement `LUMIX_HDR_DEMO=1` (prioritaire sur le fichier de configuration).

En mode démo, le bouton **Connecter** confirme par un toast qu’aucune connexion à un appareil réel n’est tentée.

## Journaux

Les journaux roulants sont écrits dans le dossier de logs de l’application. Le chemin est affiché dans Réglages.
