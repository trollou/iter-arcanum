# 0004 — Portabilité Windows / Linux

- Statut : acceptée (2026-10-23)

## Contexte

Le jeu doit tourner sur Windows et sur Linux, à égalité. macOS n'est pas visé.

## Décision

- Aucun code spécifique à un OS dans `arcanum-core`, `arcanum-i18n` ni `arcanum-players`. Si `arcanum-ui` ou `arcanum-persistance` en ont besoin, il est isolé derrière `#[cfg(target_os = ...)]` dans un module dédié.
- Chemins : toujours `std::path::Path`/`PathBuf`, jamais de séparateur `/` ou `\` en dur. Les dossiers de sauvegarde et de configuration passent par une crate standard (ex. `directories`).
- Fichiers texte (JSON, sauvegardes) en UTF-8 ; ne pas dépendre des fins de ligne.
- Pas de dépendance qui ne compile que sur un des deux systèmes.
- Les commandes et scripts du projet fonctionnent sous PowerShell et sous un shell Linux, ou existent dans les deux versions.

## Conséquences

- La CI (`.github/workflows/ci.yml`) lance `fmt`, `clippy` et `test` sur Windows et sur Linux. Pour économiser les minutes (issue privée 48),
  Windows ne tourne en PR que si elle touche le code dépendant de l'OS (`arcanum-ui`, `arcanum-persistance`, `assets/`, dépendances), et chaque nuit sur `main`.
