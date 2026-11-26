---
paths:
  - "crates/arcanum-persistance/**"
---

# Règles pour `arcanum-persistance`

- Une sauvegarde = journal d'événements (la configuration de départ est dans `GameStarted`) + position du générateur, en JSON via `serde` ; pas d'instantané d'état sur disque (spec 024, Q1).
- Le fichier porte un numéro de format. Une sauvegarde de l'une des trois dernières versions publiées se charge (spec #52) : un changement incompatible (forme JSON, ou sens d'`apply`) passe `EVENT_FORMAT_VERSION` à `PREVIOUS_RELEASE_FORMAT + 1` une fois par cycle et écrit ou complète la migration (`src/migration.rs`, pure, sur le JSON) avec son test. Recette : `docs/architecture.md`, « Ajouter un événement », point 5.
- L'empreinte `tests/format/sauvegarde.yaml` se régénère pour un changement compatible ; les témoins `tests/temoins/` ne se régénèrent jamais. À la release : `docs/release.md`.
- Chaque format a des tests d'aller-retour : sauvegarder puis charger redonne exactement le même état.
- Rejouer le journal d'une sauvegarde redonne l'état sauvegardé, et une partie rechargée tire les mêmes dés et mélanges (tests obligatoires).
- Dossiers via une crate standard (ex. `directories`), chemins en `PathBuf`, fichiers en UTF-8.
