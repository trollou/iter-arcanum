---
paths:
  - "crates/arcanum-ui/**"
---

# Règles pour `arcanum-ui`

- Affichage et saisie uniquement : aucune règle du jeu ici. Toute décision de jeu passe par `arcanum-core` (actions légales, puis choix).
- Aucun texte en dur : tout libellé passe par une clé `arcanum-i18n`.
- Ne jamais bloquer la boucle egui en attendant un joueur (humain ou automatique) : la partie avance décision par décision.
- Le joueur humain implémente le trait `Player` comme les autres.
- Visuels : créés par nous (SVG dans `assets/images/`, chargés depuis le dossier des ressources), dans l'esprit du jeu de base, rien d'officiel ni de décalqué (voir « Assets et droits » dans `CLAUDE.md`).
- Code spécifique à un OS : isolé derrière `#[cfg(target_os = ...)]` dans un module dédié (voir `docs/decisions/0004-portabilite.md`).
