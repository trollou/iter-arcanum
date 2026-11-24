# Plan de développement — spec 033 Capacités des ennemis

- Statut : suivi (même démarche que la 032 : implémentation dès la validation de la spec, 2026-10-29).
- Spec : [`033-capacites-ennemis.md`](../033-capacites-ennemis.md). Branche : `spec/033-capacites-ennemis`.
- Méthode : celle de `CLAUDE.md` (tests rouges, code minimal, refactor ; `scripts/verifier.sh` vert avant chaque push).

## Découpage en PR

Une seule PR (privée 72, qui porte déjà la spec) : 35 cas, trois événements nouveaux, l'écran de combat existe déjà (031, 032).

## Étapes

Chaque étape = un commit de tests (rouges, local) puis un commit de code (vert), sauf mention.

| # | Étape | Cas | Fichiers | Isolée ? | Modèle |
|---|---|---|---|---|---|
| 1 | **Vif et Brutal** : parade à atteindre et dégâts doublés, exposés par le combat | C1 à C6 | `combat.rs`, nouveau test `crates/arcanum-core/tests/capacites_ennemis.rs` | non | fil principal |
| 2 | **Toxique et Paralysant** : `WoundsTaken.poisoned`, `HandParalyzed` | C7 à C13 | `combat.rs`, `event.rs`, `engine.rs` | non | fil principal |
| 3 | **Invocation** : `EnemySummoned`, `SummonedEnemyDiscarded`, créature invoquée, point de non-retour ; 031 C24 et 032 L4 ajustés (signalés dans la PR) | C14 à C19, C21 | `combat.rs`, `event.rs`, `engine.rs`, `tests/deroule_combat.rs` | non | fil principal |
| 4 | **Pile de secours** : tirage dans une autre pile quand une pile et sa défausse sont vides (invocation et révélation de tuile) | C20, C20 bis, C20 ter | `combat.rs`, `exploration.rs`, `rng.rs` | non | fil principal |
| 5 | **Endurance givrée renforcée** | C22 à C25 | `card.rs`, `content.rs`, `basic_actions.ron`, `combat.rs` | non | fil principal |
| 6 | **Vue, rejeu, sauvegarde ; parcours J1** | C26, C27, C33 | `view.rs`, tests | non | fil principal |
| 7 | **Libellés** `fr.json` et `en.json` (tableau de la spec ; `enemy.ability.*` aux termes de la FAQ française) | — | `assets/i18n/*.json` | oui | fil principal (court) |
| 8 | **Écran** : avertissement d'invocation, créature invoquée, ligne « À parer · Si non paré », ligne « Vif », historique des dégâts | C28 à C32 | `screen/combat.rs`, `screen/text.rs`, `app.rs` | non | fil principal |
| 9 | **Partie préparée** `combat-capacites` et vérification à l'œil sous Xvfb | — | `crates/arcanum-persistance/tests/parties_preparees.rs`, `scripts/parties/` | non | fil principal |
| 10 | **Docs** : architecture, PLAN et index (« implémentée ») | — | docs | oui | fil principal |
| 11 | **Bilan de fin de spec** | — | — | non | fil principal |

## Choix du modèle par étape

Tout reste au fil principal, comme pour la 032 : les étapes se suivent sur `combat.rs` et l'écran de combat ; les seules isolées
(libellés, docs) sont trop courtes pour valoir un brief (issue privée 66).

## Risques repérés

- **Point de non-retour au passage en phase de parade** : les joueurs de test et les parcours qui reviennent en arrière à travers
  ce passage, avec un Invocateur en jeu, changent de comportement.
- **Parcours J1** : les Sorcières maudites deviennent toxiques ; deux Blessures de plus dans la défausse peuvent changer les
  pioches des tours suivants (C33, découpage indicatif, ADR 0006).
- **Créature invoquée hors de la pile marron** (pile de secours) : elle retourne dans la défausse de sa couleur, pas forcément marron.
- **Quota CI** épuisé jusqu'au 1er novembre environ : vérification locale par `scripts/verifier.sh`, fusion seulement avec l'accord
  de Mathieu.
