---
paths:
  - "crates/arcanum-audio/**"
  - "assets/sounds/**"
---

# Règles pour `arcanum-audio`

- Le son écoute la partie et ne décide rien : rien dans l'état, le journal ni les sauvegardes (spec 081, ADR 0010).
- Il n'entend que ce que voit le joueur : événements filtrés (`filtered_for(PlayerId::SOLO)`), jamais l'état complet.
- Le directeur (`director.rs`, `cues.rs`) reste pur : ni fichier, ni Kira, ni horloge ; tout comportement s'y teste.
- Les tests d'arcanum-audio et d'arcanum-ui n'ouvrent jamais la carte son : seul `main.rs` crée `KiraPlayer` (L5).
- Un fichier de `assets/sounds/` = une ligne de `assets/sounds/LICENSES.md` (titre, auteur, source, licence,
  modifications) ; licences permises dans la spec 081 (jamais ND ; NC en second choix, signalé). Ogg Vorbis, effets en
  mono ; chaque son a son fichier et le dossier reste sous 20 Mo (test `every_cue_has_a_sound_within_budget`).
- Choisir ou remplacer un son : planche d'écoute de `scripts/sons/` (mode d'emploi dans son README).
- Logs en français, en dur (spec 003).
