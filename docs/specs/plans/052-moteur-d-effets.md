# Plan de développement — spec 052 Moteur d'effets

- Statut : accepté par Mathieu avec la spec (2026-11-11).
- Spec : [`052-moteur-d-effets.md`](../052-moteur-d-effets.md), ADR [0008](../../decisions/0008-moteur-d-effets.md). Branche :
  `spec/052-moteur-d-effets`.
- Méthode : celle de `CLAUDE.md` (tests rouges, code minimal, refactor ; `scripts/verifier.sh` vert avant chaque push), avec le
  rouge visible (issue privée 106) : squelette `todo!()` d'abord, commit local `Tests rouges : …` qui nomme les tests en échec et leur
  nombre (`scripts/verifier.sh --rouge`), section « Rouge vu » dans la PR.

## Découpage en PR

Une seule PR d'implémentation : des effets en cours sans retraite forcée laisseraient le héros finir son tour sur un lac sans
conséquence. La PR de la spec (issue privée 215) est fusionnée d'abord ; la branche est recréée depuis `main` pour l'implémentation.

## Découpage en fils

Deux fils successifs sur la même branche (issue privée 118 : plus de 8 étapes, moteur et écran) :

1. **Fil moteur** : étapes 1 à 8 (C1 à C43, persistance, parties préparées, docs du moteur). Il pousse sa dernière étape verte et
   s'arrête.
2. **Fil écran** : étapes 9 à 12 (libellés, zone de jeu, plateau, combat, fin du tour, vérification à l'œil, docs, bilan), à partir
   de la spec, de ce plan et de la branche.

**Contrôle des champs de la vue** (issue privée 130) : `PlayerView::modifiers` (étape 1), `move_rules` (étape 2), `forced_withdrawal`
(étape 6) sont produits par le fil 1 ; `is_safe_space` est publique dès l'étape 6 ; le choix de Temporisation se lit dans les
actions légales `EndTurn` (étape 7). Les autres données nommées par la section Interface existent déjà (`combat`, actions légales,
zone de jeu).

## Choix de conception (à confirmer par le fil moteur, sans nouvelle validation s'il reste dans l'ADR)

- **Types** (module `modifier.rs`) : `Modifier { kind, source, duration }`, `ModifierKind` (`TerrainCost { terrains, change }`,
  `Passable(Terrain)`, `Convert { from, to, rate }`, `NextCardBonus { attack, block }`, `EndOfTurnReputation(i8)`,
  `ReturnToDeck { top }`), `TerrainChange` (`Reduce { by, min }`, `SetTo(u8)`), `ModifierDuration` (`Turn`, `Combat`, réservés
  `UntilNextTurn`, `Round` documentés mais non créés si inutilisés : clippy), `ModifierSource` (`Card(CardId)`, `Unit(UnitCard)`).
  Le déclenchement se déduit de la sorte (Embuscade consommée, fin du tour pour les deux dernières).
- **Option en données** : l'option d'une carte ou d'une capacité devient « effet + modificateurs » avec un champ facultatif, pour
  que `basic_actions.ron`, `units.ron` et les tests de contenu restent valides ; `Effect` reste `Copy`, `with_bonus` inchangé.
- **Ordre le plus favorable** (C15) : au plus trois réductions de terrain en jeu ; essayer les permutations suffit.
- **Jouabilité** : un seul endroit, `is_playable_now` (issue privée 111), qui reçoit les modificateurs ; `movement_over` garde son sens
  (mouvement fini), c'est la jouabilité qui consulte « le modificateur sert-il encore ».

## Étapes

| # | Étape | Cas | Rouge attendu | Fichiers | Isolée ? | Modèle |
|---|---|---|---|---|---|---|
| 1 | Modificateurs : types, `PlayerState::modifiers`, pose par `CardPlayed` et `UnitActivated`, retrait par durée (`TurnEnded`, `CombatEnded`), vue `modifiers` ; option « effet + modificateurs » en données | C1, C3, C4 | C1 : la liste n'est pas vidée (`todo!()`) ; C4 : vue vide | `modifier.rs`, `card.rs`, `unit.rs`, `content.rs`, `state.rs`, `engine.rs`, `view.rs`, `content/advanced_actions.ron` (Chemins de traverse seul) | non | fil principal |
| 2 | Règles de déplacement `MoveRules` (`From<TimeOfDay>`), `TerrainCost`, `Passable` ; Chemins de traverse, Pont de glace, Bûcherons en données ; vue `move_rules` | C11 à C18, C43 | C11 : la forêt coûte 3 | `movement.rs`, `engine.rs`, `view.rs`, `content/*.ron`, tests `deplacement`, `cases_atteignables` | non | fil principal |
| 3 | Jouabilité : « les points servent / le modificateur sert » dans `is_playable_now` ; Agilité, Diplomatie, Embuscade, Intimidation, Temporisation en données | C6 à C10 | C6 : Agilité refusée en combat | `combat.rs`, `view.rs`, `content/advanced_actions.ron` | non | fil principal |
| 4 | Conversions : `SpendMoveAsAttack`, `SpendInfluenceAsBlock`, réserves gardées en combat | C28 à C33 | C28 : action absente des actions légales | `action.rs`, `event.rs`, `engine.rs`, `combat.rs` | non | fil principal |
| 5 | Embuscade : bonus appliqué et consommé par la prochaine carte d'Attaque ou de Parade ; Concentration | C2, C5, C34 à C37 | C34 : l'attaque compte 2 | `engine.rs`, `combat.rs` | non | fil principal |
| 6 | Case sécurisée, trajet du tour (`HeroMoved`, `HeroWithdrew`), `ForcedWithdrawal` en tête de la fin du tour, `EndTurn` évalué après la retraite ; vue `forced_withdrawal` | C19 à C27 | C19 : aucune retraite ; C24 : trois Blessures au lieu d'une | `movement.rs`, `state.rs`, `engine.rs`, `event.rs`, `view.rs` | non | fil principal |
| 7 | Fin du tour : Intimidation (réputation), Temporisation (champ de `EndTurn`, `SteadyTempoKept`), ordre | C38 à C41 | C38 : réputation inchangée | `action.rs`, `engine.rs`, `event.rs`, tests et joueur scripté (`EndTurn`) | non | fil principal |
| 8 | Contenu complet (C42), parties préparées régénérées, persistance vérifiée, docs du moteur (`architecture.md`, glossaire, index par notion) | C42 | C42 : une carte encore `Unsupported` | `content/*.ron`, `arcanum-persistance`, `docs/` | docs : oui | fil principal ; docs : Haiku |
| 9 | Libellés `fr.json` et `en.json` (tableau de la spec) | — | — | `assets/i18n/*.json` | oui | Haiku |
| 10 | Écran du plateau et zone de jeu : effets en cours, coûts et trajets par `move_rules`, infobulle de case non sécurisée | C4, C17, C27, C43 (écran) | — (vérification à l'œil) | `arcanum-ui` (panneaux, `board/screen.rs`) | non | fil principal |
| 11 | Combat et fin du tour : boutons de conversion, ligne de retraite forcée, case Temporisation | C26, C28, C31, C39 (écran) | — (vérification à l'œil) | `arcanum-ui` | non | fil principal |
| 12 | Partie préparée « lac » (Pont de glace renforcé en main, sur le village de la campagne 1) et « agilité » (combat), vérification à l'œil, captures, docs de l'écran, bilan de fin de spec | — | — | `arcanum-persistance` (parties préparées), `docs/` | captures : oui | fil principal ; captures : Sonnet |

## Risques repérés

- **`EndTurn` change de forme** : tous les littéraux des tests, du joueur scripté et des parcours ; le nombre d'actions légales
  `EndTurn` double quand Temporisation est jouée. Signaler les tests touchés dans la PR.
- **Signature de `move_cost` et des trajets** : la conversion `From<TimeOfDay>` doit garder les ~30 appels existants ; sinon les
  lister dans la PR.
- **Fin du tour évaluée après la retraite** : `end_turn_allowed`, `glade_allowed`, la mine et la limite de main lisent la case et
  la main ; les faire lire l'état simulé après `ForcedWithdrawal`, sans dupliquer leur logique.
- **Jouabilité** : Mouvement et Influence deviennent jouables en combat ; vérifier que les parcours J3 et J4 n'ont pas d'actions
  légales nouvelles qui changent l'index choisi par le joueur scripté.
- **Parties préparées** : format des événements changé, à régénérer (spec 024, Q1 : pas de migration).
