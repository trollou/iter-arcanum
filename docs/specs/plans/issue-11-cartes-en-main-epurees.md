# Plan de développement — spec #11 Cartes en main épurées

- Statut : accepté (fil d'implémentation, 2026-11-24 ; Effort High : deux fils).
- Spec : [`issue-11-cartes-en-main-epurees.md`](../issue-11-cartes-en-main-epurees.md). Branche :
  `feature/11-cartes-en-main-epurees`.
- Méthode : celle de `CLAUDE.md` (tests rouges, code minimal, refactor ; `scripts/verifier.sh` vert avant chaque push).

## Découpage en PR

Une seule PR, #39 (la PR de la spec).

## Découpage en fils

Rien dans arcanum-core : la coupure sépare la vue de l'écran (calculée, testée) du dessin egui.

1. **Fil 1, vue** : étapes 1 à 6. Pousse la dernière étape verte et s'arrête.
2. **Fil 2, écran** : étapes 7 à 9, puis PR prête et bilan.

Contrôle (issue privée 130) : chaque élément de la section Interface de la spec a son champ produit au fil 1 : en-tête et
fenêtre du « i » (`info`, `Icon::Info`), corps (`show_text`), séparation (`strong_heading`, `strong_colors`), bouton à mana
en plus (`EffectButton::color`, face sans « (mana …) »), options payantes de base (`basic_effects`).

## Étapes

| # | Étape | Cas | Rouge attendu | Fichiers | Isolée ? | Modèle |
|---|---|---|---|---|---|---|
| 1 | `Icon::Info`, `assets/images/ui/info.svg`, libellés `card.info`, `card.if_destroyed`, `effect.spell_from_offer` | spec, Icône | `every_icon_image_exists` : 37 images au lieu de 38 | `icons.rs`, `assets/`, `icones.rs` | oui | fil principal |
| 2 | `HandCard::info` et `show_text` | C1 à C6 | C1 : `info` vide | `screen/types.rs`, `screen/hand.rs`, `tests/panneaux.rs` | non | fil principal |
| 3 | `strong_heading`, `strong_colors` | C7 | titre vide | idem | non | fil principal |
| 4 | Le renforcé ne répète plus le mana de la carte | C8 à C10 | pastille rouge de Rage renforcée encore là | `screen/effects.rs`, `screen/spells.rs`, `tests/panneaux.rs`, `tests/sorts.rs` | non | fil principal |
| 5 | Options payantes de base dans le bloc de base, face sans paiement | C11, C12 | `basic_effects` de Cristallisation vide | `screen/effects.rs` | non | fil principal |
| 6 | Bouton d'unité à mana sans « (mana …) » | C13, C14 | face « (mana blanc) Un ennemi… » | `screen/effects.rs`, `tests/unites.rs` | non | fil principal |
| 7 | Dessin : « i » et sa fenêtre, corps sans texte, séparations, « pastille : effet », passage à la ligne de tout bouton | Interface, L1 à L4 | — (vérification à l'œil) | `app/bottom_panel.rs` | non | fil principal |
| 8 | Vérification à l'œil (`main-blessures`, `actions-avancees`, `sorts`, `artefacts`, `unites`) | Interface | — | captures | oui | fil principal |
| 9 | Docs (073, 077, index par notion, architecture) | — | — | `docs/` | oui | Haiku |
