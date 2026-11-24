---
paths:
  - "crates/arcanum-persistance/**"
---

# Règles pour `arcanum-persistance`

- Une sauvegarde = journal d'événements (la configuration de départ est dans `GameStarted`) + position du générateur, en JSON via `serde` ; pas d'instantané d'état sur disque (spec 024, Q1).
- Le fichier porte une version. Pas de migration avant la fin du J6 (spec 024) : une sauvegarde illisible ou d'une autre version est refusée proprement. Après le J6, changer la forme d'un événement exigera une migration et son test.
- Chaque format a des tests d'aller-retour : sauvegarder puis charger redonne exactement le même état.
- Rejouer le journal d'une sauvegarde redonne l'état sauvegardé, et une partie rechargée tire les mêmes dés et mélanges (tests obligatoires).
- Dossiers via une crate standard (ex. `directories`), chemins en `PathBuf`, fichiers en UTF-8.
