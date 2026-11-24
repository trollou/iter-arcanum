# Plan de développement — spec 042 Unités

- Statut : suivi (2026-11-04) ; proposé à Mathieu au début du fil d'implémentation (2026-11-03).
- Spec : [`042-unites.md`](../042-unites.md). Branche : `spec/042-unites`.
- Méthode : celle de `CLAUDE.md` (tests rouges, code minimal, refactor ; `scripts/verifier.sh` vert avant chaque push), avec
  les règles du rouge visible (issue privée 106) : squelette `todo!()` d'abord, commit local `Tests rouges : …` qui nomme les tests
  en échec et leur nombre, section « Rouge vu » dans la PR.

## Découpage en PR

Une seule PR : recruter sans pouvoir activer, ou activer sans assigner de dégâts, laisserait le jeu dans un état bancal.

## Choix de modèle (à valider avec le plan)

- **Capacités en données** : `UnitDefinition::abilities`, une `UnitAbility` par capacité imprimée, dans l'ordre de la carte :
  le mana à payer (`Option<ManaColor>`, noir pour les Mages d'Altem) et ses options (`Vec<AbilityOption>` : un `Effect` et les
  couleurs de mana payées en plus, pour le Feu glacé 7 et 9 des Mages d'Altem), ou `Unsupported`. Nouvelles formes d'`Effect` :
  `ReadyUnit(niveau max)`, `SwiftBlock(n)` (parade qui compte double contre Vif), `TokenAndCrystal(couleur)`,
  `GainTokens(couleur, couleur)` ; l'Épéiste blessé par sa capacité est un drapeau de la capacité (`wounds_unit`).
- **Jetons de mana de toutes les couleurs** : « deux jetons de n'importe quelles couleurs, noir compris » (Mages d'Altem) demande
  des jetons or et noir. `mana_tokens` et `ManaTokenGained`/`ManaTokenUsed` passent de `BasicColor` à `ManaColor` ; un jeton or
  paie une couleur de base de jour (MKUE p. 5, 3.c), un noir rien encore (sorts : spec 053). Payer avec un jeton prend d'abord
  la couleur exacte. Les tests des specs 025 et 024 qui construisent ces événements changent de type (signalé en PR).
- **`ActivateUnit { unit, ability, option, mana, target }`** : `mana` est la liste des paiements, dans l'ordre des couleurs à
  payer (capacité, puis supplément) ; `target` l'unité à rendre disponible.
- **Assignation des dégâts** : `Combat` garde l'ennemi en cours et ses dégâts restants, et les ennemis traités ; la phase
  d'assignation devient une vraie phase quand une unité peut recevoir des dégâts (sinon, résolution seule comme avant).

## Étapes

| # | Étape | Cas | Rouge attendu | Fichiers | Isolée ? | Modèle |
|---|---|---|---|---|---|---|
| 1 | **Capacités en données** : `UnitAbility`, `units.ron`, lecture et validation | C16 (données) | les capacités lues (squelette : aucune) diffèrent de la liste de `cartes/unites.md` | `unit.rs`, `card.rs`, `content.rs`, `units.ron`, `tests/contenu_en_donnees.rs` | non | fil principal |
| 2 | **Zone des unités et recrutement** : `OwnedUnit`, `RecruitUnit`, `UnitRecruited`, `UnitDisbanded`, vue | C1 à C7, L1, L2 | C1 : `RecruitUnit` absente des actions légales | `state.rs`, `view.rs`, `action.rs`, `event.rs`, `engine.rs`, nouveau `tests/unites.rs` | non | fil principal |
| 3 | **Activer hors combat** : `ActivateUnit`, mana, gains de mana (jetons `ManaColor`), rendre disponible, nouvelle manche | C8 à C10, C15 à C20, L3, L8 | C8 : pas d'`ActivateUnit` légale pour les Paysans | `engine.rs`, `mana.rs`, `card.rs`, `event.rs`, `tests/unites.rs` | non | fil principal |
| 4 | **Unités en combat** : phases, éléments, Gardes (Vif), Épéistes | C11 à C14 | C13 : la parade des Gardes compte 4 | `combat.rs`, `tests/unites.rs` | non | fil principal |
| 5 | **Assigner les dégâts** : `AssignDamage`, résistances, toxique, paralysant, ordre, assommé | C21 à C30, L6, L7 | C21 : la phase se résout seule, sans `AssignDamage` | `combat.rs`, `engine.rs`, `event.rs`, `tests/unites.rs` | non | fil principal |
| 6 | **Points de soin** : réserve, `SpendHealing`, `UnitHealed`, perte ; tests de la 034 revus | C31 à C34, L4 | C31 : Tranquillité jette encore la Blessure aussitôt | `engine.rs`, `state.rs`, `tests/blessures_et_soins.rs` | non | fil principal |
| 7 | **Décompte, retour en arrière, sauvegarde, vue** | C35 à C38 | C35 : Le Commandant vaut 0 | `game_end.rs`, tests arcanum-core et arcanum-persistance | non | fil principal |
| 8 | Libellés `fr.json`, `en.json`, glossaire | — | — | `assets/i18n/*.json`, `docs/glossaire.md` | oui | fil principal (lié aux écrans) |
| 9 | **Écran** : panneau des unités, boutons de capacité, recrutement, assignation, soins, score | C39 à C44 | C39 : pas de section Unités dans `GameScreen` | `arcanum-ui/src/screen/`, `app/`, `arcanum-ui/tests/unites.rs` | non | fil principal |
| 10 | Partie préparée `unites`, vérification à l'œil, captures | — | — | `arcanum-persistance/tests/parties_preparees.rs`, `scripts/parties/`, `docs/verification-manuelle.md` | non | fil principal |
| 11 | Docs : `architecture.md`, `PLAN.md`, `index-par-notion.md` | — | — | `docs/` | oui | fil principal |

## Risques repérés

- Le nombre d'actions légales grandit (capacités × paiements) : ordre stable documenté, les tests choisissent par valeur.
- Le passage des jetons en `ManaColor` touche les specs 025 et 024 (tests et écran) : étape 3, signalé en PR.
- La phase d'assignation qui attend une décision change le parcours d'un combat avec unités ; sans unité, rien ne change
  (C29), les parcours 019, 028, 039 doivent rester verts sans retouche.
- Soin X en réserve (Q1) change des tests de la 034 : revus dans l'étape 6 et listés en PR.
