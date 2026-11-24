# Plan de développement — spec 055 Artefacts

- Statut : exécuté (fil moteur, étapes 1 à 12 ; fil écran, étapes 13 à 17, 2026-11-13).
- Spec : [`055-artefacts.md`](../055-artefacts.md). Branche : `spec/055-artefacts` (PR privée 266, rédaction et implémentation dans
  la même PR).
- Méthode : celle de `CLAUDE.md` (tests rouges, code minimal, refactor ; `scripts/verifier.sh` vert avant chaque push).

## Découpage en fils

Deux fils successifs sur la même branche (issue privée 118 : 17 étapes, moteur et écran) :

1. **Fil moteur** : étapes 1 à 12 (arcanum-core, contenu, persistance). Il pousse l'étape 12 verte et s'arrête.
2. **Fil écran** : étapes 13 à 17 (libellés, écrans, partie préparée, vérification à l'œil, docs, bilan).

Contrôle (issue privée 130) : l'écran lit les façons de jouer un artefact (`Artifact`, `ThrowAway` et leurs choix : cartes défaussées,
ennemis, bonus du Cor, couleurs, carte de l'offre), `AssignBanner`, `UseBanner`, `ReclaimBanners` et les paiements
`ManaPayment::Endless` dans les **actions légales** (étapes 1, 2, 3, 5, 7, 9 à 11) ; la bannière de chaque unité et si elle est
retournée, le dé or du Sac de gemmes en attente et le choix des bannières à reprendre dans `PlayerView` (étapes 2, 3, 10, 12) ; les
modificateurs nouveaux dans `PlayerView::modifiers` (étapes 4 à 9, 11). Aucun champ de vue n'est laissé au fil écran.

## Choix de modèle (noms indicatifs de la spec précisés)

- **Jouer un artefact** : deux variantes de `CardPlay`, `Artifact { option, choices }` (effet de base, sans mana) et `ThrowAway {
  option, choices }` (effet renforcé, carte détruite). `choices: ArtifactChoices` reste `Copy` (comme `CastTargets`, spec 057) :
  `discard: Picks<CardId>` (Épée de justice ; la carte détruite par le Livre de sagesse), `enemies: Picks<EnemyToken>` (Bannière de
  terreur renforcée), `bonus: u8` (Cor de colère renforcé), `colors: Picks<ManaColor>` (Amulette des ténèbres), `gain:
  Option<CardId>` (Livre de sagesse). Tous `serde(default)`. Un artefact n'a toujours pas de `CardDefinition` (pas de couleur) :
  `CardId::effect` et `CardId::modifiers` lisent ses effets dans `GameContent::artifact`.
- **Bannière** : son effet de base est `Effect::BannerAssigned` dans `artifacts.ron`, que seule `AssignBanner` joue (C11).
- **Amulettes** : `DayRules` (soleil) et `NightMana` (ténèbres) sont écrits dans le contenu ; l'`apply` de `CardPlayed` ne pose
  `DayRules` que de nuit et `NightMana` que de jour (L6).
- **Gloire « ce tour »** (Épée, anneaux renforcés, Bannière de gloire renforcée) : ce qui est déjà fait est compté en jouant la
  carte (faits du tour, spec 049/073), la suite par une passe de `decide` qui lit les événements produits par l'action.
- **Coupe d'or** : `With(Heal(2), [GrailFame(left: 2)])` et `With(Heal(6), [DrawPerHandWoundHealed])` dans le contenu, sans effet
  nouveau ; le compteur baisse dans l'`apply` des soins dépensés.

## Étapes

| # | Étape | Cas | Rouge attendu | Fichiers | Isolée ? | Modèle |
|---|---|---|---|---|---|---|
| 1 | Jouer et détruire un artefact : `CardPlay::Artifact`, `ThrowAway`, `ArtifactChoices`, `CardDestroyed`, tour minimal ; Bourse sans fond (`InfluenceAndFame`) ; ni Concentration ni couleur | C2 à C5 | C2 : effet de base de la Bourse absent des actions légales | `card.rs`, `ids.rs`, `engine.rs`, `content.rs`, `artifacts.ron`, test `artefacts.rs` | non | fil principal |
| 2 | Bannières : `AssignBanner`, `BannerAssigned`, `BannerLost` (autre bannière, unité détruite ou renvoyée), décompte final, renforcé depuis la main | C6 à C8, C10, C11, L1 | C6 : `AssignBanner` absente | `action.rs`, `event.rs`, `unit.rs`, `engine.rs`, `game_end.rs` | non | fil principal |
| 3 | Fin de manche : choix des bannières à reprendre (`ReclaimBanners`, `BannersReclaimed`), face visible | C9, L5 | C9 : aucun choix attendu à la fin de la manche | `action.rs`, `event.rs`, `engine.rs` | non | fil principal |
| 4 | Amulettes : jetons, `DayRules` (forêts, or, défenseurs et ruines révélés), `NightMana` (déserts, noir, dé noir) | C39 à C41, L6 | C39 : la forêt coûte encore 5 | `modifier.rs`, `movement.rs`, `engine.rs` | non | fil principal |
| 5 | Anneaux : jeton, cristal et gloire ; `EndlessMana` et `ManaPayment::Endless` ; gloire par sort de la couleur | C24 à C26 | C25 : aucun paiement `Endless` | `card.rs`, `modifier.rs`, `engine.rs` | non | fil principal |
| 6 | Bannière de gloire : Armure, +1 aux attaques et parades, gloire par activation ; renforcée (`UnitArmorBonus`, `FamePerUnitAttackOrBlock`) | C12 à C14 | C12 : Parade 2 au lieu de 3 | `unit.rs`, `combat.rs`, `engine.rs` | non | fil principal |
| 7 | Bannières de terreur et de courage : `UseBanner`, `BannerUsed`, `CancelAttacks`, `ReadyAllUnits` ; retournement | C15 à C18, C22, C23, L2, L3, L7 | C15 : `UseBanner` absente | `action.rs`, `event.rs`, `engine.rs` | non | fil principal |
| 8 | Bannière de protection : Armure et résistances ; Blessures reçues ce tour, `ThrowAwayReceivedWounds`, `WoundsThrownAway` | C19 à C21 | C20 : aucun `WoundsThrownAway` | `state.rs`, `event.rs`, `engine.rs` | non | fil principal |
| 9 | Épée de justice : défausses pour l'Attaque, `FamePerEnemyDefeated`, `DoublePhysicalAttacks`, résistance physique perdue | C27 à C30 | C27 : Épée de base absente | `engine.rs`, `combat.rs` | non | fil principal |
| 10 | Cor de colère et Sac de gemmes : `RngUsage::ArtifactDice`, `ArtifactDiceRolled`, Blessures, cristaux, dé or en attente | C31 à C34 | C31 : aucun `ArtifactDiceRolled` | `rng.rs`, `event.rs`, `engine.rs` | non | fil principal |
| 11 | Coupe d'or et Livre de sagesse : gloire par point de soin, pioche par Blessure soignée ; action avancée ou sort de l'offre en main | C35 à C38, L4 | C35 : aucune gloire | `modifier.rs`, `engine.rs` | non | fil principal |
| 12 | Contenu complet, vue, persistance (parties préparées régénérées si le format change) | C1, C42, C43 | C1 : un artefact encore `Unsupported` ; C42 : `banner` absent de la vue | `content/*.ron`, `view.rs`, `arcanum-persistance` | non | fil principal |
| 13 | Libellés `fr.json` et `en.json` (section Libellés) | — | — | `assets/i18n/*.json` | oui | Haiku |
| 14 | Écran : boutons Détruire et Assigner, bannière sous l'unité, `UseBanner`, effets en cours | C2, C6, C15, C22, C42 | `game_screen` : pas de bouton « Détruire » | `arcanum-ui` | non | fil principal |
| 15 | Écran : choix (Épée, Cor, Terreur, couleurs, dé or), mana illimité, fenêtre de fin de manche | C9, C18, C25, C27, C32, C33 | `game_screen` : pas de source « illimité » | `arcanum-ui` | non | fil principal |
| 16 | Partie préparée `artefacts` et vérification à l'œil | — | — (vérification à l'œil) | `arcanum-persistance`, `scripts/` | non | fil principal |
| 17 | Docs : `architecture.md`, `PLAN.md` (implémentée), index par notion, glossaire, raccourcis, bilan de fin de spec | — | — | `docs/` | oui | Sonnet |

## Risques repérés

- **Littéraux de `CardPlay`** : deux variantes nouvelles, les `match` sans joker du moteur et d'arcanum-ui à compléter.
- **Nombre d'actions légales** : l'Épée (sous-ensembles de la main) et la Bannière de terreur renforcée (jusqu'à trois ennemis)
  multiplient les actions ; l'Épée est bornée par la main, les ennemis par `Picks`.
- **Paiements** : le mana illimité et le mana noir ou or « comme de jour / de nuit » passent par les fonctions de paiement de
  toutes les cartes (specs 022, 025, 054) ; les tests de ces specs gardent le comportement sans artefact.
- **Test modifié** : `taken_artifact_plays_sideways` (spec 041) change d'assertion (C2), signalé dans la PR.
