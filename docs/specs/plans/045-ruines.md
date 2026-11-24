# Plan de développement — spec 045 Ruines

- Statut : accepté par Mathieu avec la spec (2026-11-05).
- Spec : [`045-ruines.md`](../045-ruines.md). Branche : `spec/045-ruines`.
- Méthode : celle de `CLAUDE.md` (tests rouges, code minimal, refactor ; `scripts/verifier.sh` vert avant chaque push), avec
  le rouge visible (issue privée 106) : squelette `todo!()` d'abord, commit local `Tests rouges : …` qui nomme les tests en échec et
  leur nombre (`scripts/verifier.sh --rouge`), section « Rouge vu » dans la PR.

## Découpage en PR

Une seule PR (celle qui porte la spec) : des jetons de ruines posés sans autel ni combat, ou une ruine accomplie sans sa
récompense, laisseraient le jeu bancal.

## Découpage en fils

Deux fils successifs sur la même branche (issue privée 118 : plus de 8 étapes, moteur et écran) :

1. **Fil moteur** : étapes 1 à 8 (C1 à C30, persistance, parties préparées, docs du moteur). Il pousse sa dernière étape verte et
   s'arrête.
2. **Fil écran** : étapes 9 à 12 (libellés, C31 à C40, vérification à l'œil, docs de l'écran, bilan), à partir de la spec, de ce
   plan et de la branche.

**Contrôle des champs de la vue** (issue privée 130) : `ruins`, `ruin_pile`, `crystal_set_rewards`, `advanced_action_rewards`,
`unit_rewards`, `revealed_ruins` sont produits à l'étape 7 (fil 1) ; les autres champs nommés par la section Interface existent
déjà (`enemy_tokens`, `shields`, `artifact_rewards`, `drawn_artifacts`, `offers`, `units`, `command_tokens`).

## Choix de modèle

- **Contenu** : `RuinToken` (12 variantes, un exemplaire chacune) dans un module `ruin.rs` ; `RuinTokenDefinition` lue dans
  `content/ruins.ron` : `Altar(BasicColor)` ou `Enemies { colors: Vec<EnemyColor>, rewards: Vec<RuinReward> }` ;
  `RuinReward` : `Artifact`, `Spell`, `AdvancedAction`, `CrystalSet`, `Unit`.
- **État** : `GameState::ruin_pile`, `ruin_discard` (`Vec<RuinToken>`), `ruins : BTreeMap<Hex, PlacedRuin>` (jeton, face
  cachée) ; une ruine accomplie quitte `ruins` (son Bouclier est dans `shields`). Les jetons d'ennemis restés sont dans
  `enemy_tokens` (spec 044).
- **Première entrée** : une ruine de `ruins` sans jeton d'ennemi sur sa case tire les couleurs de son jeton ; sinon on combat les
  jetons restés. Pas de drapeau « déjà visitée ».
- **Autel** : `Action::PayAltarTribute { mana: Vec<ManaPayment> }` ; les paiements légaux sont les multiensembles de
  `mana_payments` de taille 3 (au plus `dice_limit − dés pris` dés, cristaux et jetons dans la limite de l'inventaire), dans un
  ordre canonique (dés, cristaux, jetons) et sans doublon.
- **Récompenses** : `SiteConquered` sur une ruine ajoute `artifact_rewards += n`, `spell_rewards`, `crystal_set_rewards`,
  `advanced_action_rewards`, `unit_rewards`. `ArtifactsDrawn` tire `artifact_rewards + 1` cartes ; `TakeArtifact` se répète
  jusqu'à `n` gardés. `advanced_action_rewards` s'ajoute aux `advanced_action_choices` du niveau. Le choix d'unité se place dans
  `turn_end_rewards` après les niveaux et l'action avancée du niveau, avant la pioche ; `TakeUnitReward` et `DeclineUnitReward`
  reprennent la suite comme `TakeArtifact`.
- **Révélation** : `reveals_next_to` (spec 043) a une sœur pour les ruines : sur la case du pas, de nuit, un jeton face cachée ;
  la nouvelle manche de Jour retourne tous les jetons face cachée. `FollowRoute` s'arrête sur le pas qui produit
  `RuinTokenRevealed`.
- **Générateur** : `RngUsage::RuinPile`, flux 4.

## Étapes

| # | Étape | Cas | Rouge attendu | Fichiers | Isolée ? | Modèle |
|---|---|---|---|---|---|---|
| 1 | **Contenu** : `RuinToken`, `ruins.ron`, lecture et validation | C1 | C1 : `GameContent::ruin_tokens` vide (squelette) | `ruin.rs`, `content.rs`, `content/ruins.ron`, `lib.rs`, nouveau `tests/ruines.rs` | oui | Sonnet |
| 2 | **Pile et pose** : `RuinPileShuffled` à la mise en place, `RuinTokenPlaced` à l'exploration (face selon le moment) | C2 à C4 | C3 : aucun `RuinTokenPlaced` en explorant la campagne 8 | `engine.rs`, `event.rs`, `state.rs`, `rng.rs`, `view.rs`, tests | non | fil principal |
| 3 | **Révélation** : pas sur la ruine, trajet arrêté, manche de Jour ; ruine ignorable | C5 à C9, L5 | C5 : pas de `RuinTokenRevealed` après le pas | `engine.rs`, `movement.rs`, `event.rs`, tests | non | fil principal |
| 4 | **Autel** : `PayAltarTribute`, paiements, conquête, 7 gloire | C10 à C16, L3 | C10 : `PayAltarTribute` absente des actions légales | `action.rs`, `engine.rs`, `event.rs`, tests | non | fil principal |
| 5 | **Ennemis** : entrée, tirage par couleurs, jetons restés sans remplacement, conquête, pas de réputation | C17 à C21, L1, L2 | C17 : `EnterAdventureSite` absente sur la ruine | `engine.rs`, tests | non | fil principal |
| 6 | **Récompenses** : un cristal de chaque couleur, *n* + 1 artefacts, action avancée, unité (prise, renvoi, renoncer, après le niveau) | C22 à C27, L4, L7, L8 | C23 : deux artefacts tirés au lieu de trois | `action.rs`, `engine.rs`, `event.rs`, `state.rs`, tests | non | fil principal |
| 7 | **Décompte, sauvegarde, vue** | C28, C29 | C29 : `PlayerView::ruins` vide | `view.rs`, tests arcanum-core et arcanum-persistance | non | fil principal |
| 8 | `FirstChoice` (C30), parties préparées `ruines-autel`, `ruines-nuit`, `ruines-unite`, parcours relus, docs du moteur (`architecture.md`, glossaire, `index-par-notion.md`) | C30 | C30 : `FirstChoice` fait l'offrande | `tests/premier_choix/`, `arcanum-persistance/tests/parties_preparees.rs`, `scripts/parties/`, `docs/` | non | fil principal |
| 9 | Libellés `fr.json`, `en.json` (tableau de la spec) | — | — | `assets/i18n/*.json` | oui | Haiku |
| 10 | **Écran** : jeton de ruines sur le plateau, infobulle, bouton et fenêtre de l'offrande, confirmation d'entrée, drapeau, récompenses, trois artefacts, fenêtre de l'unité, trajet arrêté | C31 à C40 | C33 : pas de bouton d'offrande sur l'autel | `arcanum-ui/src/board/`, `arcanum-ui/src/screen/`, `session.rs`, `app/`, `arcanum-ui/tests/` | non | fil principal |
| 11 | Vérification à l'œil (trois parties), captures | — | — | `docs/verification-manuelle.md` | non | fil principal |
| 12 | Docs : `architecture.md` (écran), `raccourcis.md`, `PLAN.md` (implémentée) ; bilan de fin de spec | — | — | `docs/` | oui | fil principal |

## Risques repérés

- **Mise en place** : `RuinPileShuffled` s'ajoute à toutes les mises en place ; les tests qui comptent ou comparent les événements
  de mise en place sont à relire (changement de forme, pas de sens), et les parties préparées sont à régénérer.
- **Parcours** : un parcours qui explore la campagne 8 voit un `RuinTokenPlaced` de plus ; les autres tirages ne bougent pas
  (usage à part, ADR 0007).
- **Artefacts** : passer de « deux tirés » à « *n* + 1 tirés » touche la spec 041 ; pour *n* = 1, rien ne change (ses tests
  doivent rester verts sans retouche).
- **Nombre d'actions** : les paiements de l'offrande restent peu nombreux (au plus quelques dizaines avec beaucoup de cristaux,
  jetons et dés) ; sans gêne.
- **Ordre des actions légales** : `TakeUnitReward` et `DeclineUnitReward` après `GainAdvancedAction` ; `PayAltarTribute` à la place
  d'`EnterAdventureSite` ; les tests et joueurs choisissent par valeur.
