# Plan de développement — spec 051 Compétences : gain aux niveaux pairs et premier lot

- Statut : exécuté (fil moteur : étapes 1 à 7 ; fil écran : étapes 8 à 12).
- Spec : [`051-competences.md`](../051-competences.md). Branche : `spec/051-competences` (PR privée 288, rédaction et
  implémentation dans la même PR).
- Méthode : celle de `CLAUDE.md` (tests rouges, code minimal, refactor ; `scripts/verifier.sh` vert avant chaque push).

## Découpage en fils

Deux fils successifs sur la même branche (issue privée 118 : moteur et écran) :

1. **Fil moteur** : étapes 1 à 7 (arcanum-core, contenu, persistance). Il pousse l'étape 7 verte et s'arrête.
2. **Fil écran** : étapes 8 à 12 (libellés, écran, partie préparée, vérification à l'œil, docs, bilan).

Contrôle (issue privée 130) : l'écran lit `ChooseSkill` et `UseSkill` (avec l'unité visée d'Inspiration) dans les **actions
légales** (étapes 3 à 6) ; dans `PlayerView`, `skills` (état, options utilisables, `unsupported`, blocage de Motivation),
`skill_choice`, `common_skills` (prenable ou non), `lowest_advanced_action_only` et `skill_pile_count` (étapes 2 et 7). Aucun
champ de vue n'est laissé au fil écran. C11 (valeur du libellé `level.gained_even`) est un libellé : fil écran, étape 8.

## Choix de modèle (noms indicatifs de la spec précisés)

- **Module `skill.rs`** : `Skill` (40, `Skill::ALL`, `label_key`, `text_key`), `SkillUsage` (`Turn`, `Round`), `SkillDefinition`
  (héros, fréquence, coopérative, hors combat, `SkillEffect::Options(Vec<SkillOption>)` ou `Unsupported`), `SkillOption`
  (effet de jour, effet de nuit facultatif : la valeur « de jour et de nuit » est une donnée de l'option, résolue à
  l'utilisation), `SkillState` (`Ready`, `UsedThisTurn`, `Flipped`), `OwnedSkill`. Contenu `content/skills.ron`, lu par
  `GameContent::skill`.
- **Effets nouveaux** (variantes de `Effect`, faute d'existant qui convienne) : `CrystalAndToken { crystal, token }` (Magie
  noire, Extractions, Au cœur du vent, Murmure de la canopée ; `WoundCrystalAndToken` donne aussi une Blessure),
  `DrawAndToken { cards, token }` (Motivation : jouable en combat, contrairement à `DrawCards`) et `HealUnit` (Inspiration :
  une Blessure d'une unité, sans Soin). « Rendre disponible » d'Inspiration réutilise `ReadyAnyUnit { heal: false }`.
- **`SkillUsed { player, skill, option, effect }`** porte l'effet résolu (jour ou nuit), comme `UnitActivated` : `apply` n'a
  pas à recalculer le moment de la journée. Les points d'Attaque et de Parade s'ajoutent au combat comme une conversion
  (ni carte ni unité).
- **Fin du tour** : `PlayerState::skill_choices` compte les niveaux pairs dont la compétence reste à choisir (posé par
  `LevelGained` quand la pile a de quoi révéler) ; `skill_choice` garde les deux révélées. `GainAdvancedAction` n'est légale
  qu'une fois la compétence du niveau choisie (`advanced_action_choices > skill_choices`). La paire suivante est révélée par
  la décision qui termine le niveau précédent (`ChooseSkill` offre vide, ou `GainAdvancedAction`).
- **Motivation** : `PlayerState::motivation_block`, 2 à l'utilisation, −1 à chaque `TurnEnded`, 0 à `RoundStarted`.
- **Usage du hasard** : `RngUsage::SkillPile(joueur)`, flux 120 + joueur, nom `skill_pile/<n>`.

## Étapes

| # | Étape | Cas | Rouge attendu | Fichiers | Isolée ? | Modèle |
|---|---|---|---|---|---|---|
| 1 | Contenu : `skill.rs`, `skills.ron`, lecture et validation ; effets `CrystalAndToken`, `DrawAndToken`, `HealUnit` | C1 | C1 : contenu vide (`skills_content_has_ten_skills_per_hero`) | `skill.rs`, `content.rs`, `card.rs`, `content/skills.ron`, test `competences.rs` | non | fil principal |
| 2 | Pile et mise en place : `RngUsage::SkillPile`, `SkillPileShuffled`, `skill_pile_count` | C2, C3 | C2 : aucun `SkillPileShuffled` | `rng.rs`, `event.rs`, `engine.rs`, `state.rs`, `view.rs` | non | fil principal |
| 3 | Gain au niveau pair : `ChooseSkill`, `SkillsRevealed`, `SkillGained`, `SkillToCommonOffer` ; l'action avancée attend la compétence ; tests existants adaptés | C4 à C6, C9, C10, L1, L5 | C4 : aucun `SkillsRevealed` | `action.rs`, `event.rs`, `engine.rs`, `state.rs`, tests des specs 035, 041, 043, 045, 050 | non | fil principal |
| 4 | Offre commune : `SkillTakenFromCommonOffer`, dernière place imposée, compétence du propre héros refusée | C7, C8 | C7 : la compétence de l'offre n'est pas proposée | `engine.rs` | non | fil principal |
| 5 | Utiliser une compétence « par tour » : `UseSkill`, `SkillUsed`, moment de jeu, jour et nuit, pas l'action du tour, second lot sans action | C12 à C15, C23, C24, L2, L3 | C12 : `UseSkill` absente des actions légales | `action.rs`, `event.rs`, `engine.rs` | non | fil principal |
| 6 | Compétences « par manche » : retournées jusqu'à la manche suivante ; cristal et jeton, Motivation (pioche, jeton, blocage), Potion de soin, Inspiration | C16 à C22, L4 | C16 : Magie noire encore utilisable après usage | `engine.rs`, `combat.rs` | non | fil principal |
| 7 | Retour en arrière, vue, persistance | C25 à C27 | C26 : `skills` absent de la vue | `view.rs`, `arcanum-persistance` | non | fil principal |
| 8 | Libellés `fr.json` et `en.json` (section Libellés), `level.gained_even` | C11 | C11 : l'ancien bandeau | `assets/i18n/*.json` | oui | Haiku |
| 9 | Écran : section Compétences du panneau du bas, boutons d'options, Inspiration | C12, C16, C22, C24, C26 | `game_screen` : pas de section Compétences | `arcanum-ui` | non | fil principal |
| 10 | Écran : fenêtre du choix de compétence, offre commune, dernière place, bandeau | C4 à C8 | `game_screen` : pas de fenêtre du choix | `arcanum-ui` | non | fil principal |
| 11 | Partie préparée `competences` et vérification à l'œil | — | — (vérification à l'œil) | `arcanum-persistance`, `scripts/` | non | fil principal |
| 12 | Docs : `architecture.md`, `PLAN.md` (implémentée), index par notion, glossaire, vérification manuelle, bilan | — | — | `docs/` | oui | Sonnet |

## Risques repérés

- **Tests des specs déjà implémentées** : toute partie lancée par `new_game` qui atteint un niveau pair attend désormais une
  compétence ; les décors construits par événements (arène) n'ont pas de pile et gardent le comportement d'avant.
- **Littéraux et `match` sans joker** : trois variantes d'`Effect` et des événements nouveaux ; arcanum-ui (`effect_text`, journal)
  reçoit les branches minimales, ses libellés arrivent au fil écran.
- **Ordre des choix de fin de tour** : artefact, sort, compétence, action avancée, unité ; chaque décision qui termine un choix
  vérifie qu'il n'en reste aucun avant la pioche.
