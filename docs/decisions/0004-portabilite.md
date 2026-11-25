# 0004 — Portabilité Windows / Linux / macOS

- Statut : acceptée (2026-10-23) ; étendue à macOS par la spec #7 (2026-11-25)

## Contexte

Le jeu doit tourner sur Windows, Linux et macOS, à égalité (macOS depuis la spec #7). Personne dans le projet n'a de Mac :
la CI est le seul banc d'essai de macOS.

## Décision

- Aucun code spécifique à un OS dans `arcanum-core`, `arcanum-i18n` ni `arcanum-players`. Si `arcanum-ui` ou `arcanum-persistance` en ont besoin, il est isolé derrière `#[cfg(target_os = ...)]` dans un module dédié.
- Chemins : toujours `std::path::Path`/`PathBuf`, jamais de séparateur `/` ou `\` en dur. Les dossiers de sauvegarde et de configuration passent par une crate standard (ex. `directories`).
- Fichiers texte (JSON, sauvegardes) en UTF-8 ; ne pas dépendre des fins de ligne.
- Pas de dépendance qui ne compile que sur un des trois systèmes.
- Ce qui interroge le système (langue de l'interface, dossiers) se découpe comme pour Windows : une fonction pure,
  compilée et testée partout, fait le travail ; seul l'appel au système est derrière `#[cfg]`. Les API natives sont
  appelées directement (kernel32 sous Windows, CoreFoundation sous macOS) plutôt que par une dépendance.
- Dossiers sous macOS : préférences et sauvegardes dans `~/Library/Application Support/iter-arcanum/` (`directories`),
  logs dans `~/Library/Logs/iter-arcanum/`, le dossier qu'Apple prévoit pour les logs et que l'app Console affiche.
- Les commandes et scripts du projet fonctionnent sous PowerShell et sous un shell Linux, ou existent dans les deux versions.
  Ils ne visent pas macOS (bash 3.2) : personne n'y développe.

## Conséquences

- La CI (`.github/workflows/ci.yml`) lance `clippy` et `test` sur Linux, Windows et macOS (Apple Silicon), et `fmt` sous Linux,
  à chaque PR prête et à chaque fusion sur `main`, sans filtre de chemins depuis que le dépôt est public (spec 082, `docs/ci.md`).
- Sans Mac, la vérification à l'œil sous macOS passe par le workflow manuel `capture-macos.yml` (spec #7, Q3).
- Le paquet macOS (`.app`, signature, binaire universel) relève du mécanisme de release (issue #33).
