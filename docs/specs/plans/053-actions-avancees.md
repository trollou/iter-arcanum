# Plan de développement — spec 053 Actions avancées : combat, unités et soins

- Statut : accepté par Mathieu (2026-11-12), avec la spec.
- Spec : [`053-actions-avancees.md`](../053-actions-avancees.md). Branche : `spec/053-actions-avancees`.
- Méthode : celle de `CLAUDE.md` (tests rouges, code minimal, refactor ; `scripts/verifier.sh` vert avant chaque push).

## Découpage en PR

Comme la 052 : la PR de rédaction (spec et plan validés, issue privée 221) est fusionnée d'abord ; l'implémentation (deux fils) fait une seconde PR, sur la branche `spec/053-actions-avancees` recréée depuis `main`.

## Découpage en fils

Deux fils successifs sur la même branche (issue privée 118 : 14 étapes, moteur et écran) :

1. **Fil moteur** : étapes 1 à 9 (arcanum-core, contenu, persistance). Il pousse l'étape 9 verte et s'arrête.
2. **Fil écran** : étapes 10 à 14 (libellés, écrans, partie préparée, vérification à l'œil, docs, bilan).

Contrôle (issue privée 130) : l'écran lit les modificateurs d'ennemis et l'armure effective dans `PlayerView::combat` (étape 1), les
modificateurs du joueur dans `PlayerView::modifiers` (spec 052), les cibles ennemies et les options à second mana dans les actions
légales (étapes 3 et 7). Aucun champ de vue n'est laissé au fil écran.

## Étapes

| # | Étape | Cas | Rouge attendu | Fichiers | Isolée ? | Modèle |
|---|---|---|---|---|---|---|
| 1 | Modificateurs d'ennemis (`CombatEnemy::modifiers`, `ArmorReduction`, `DoesNotAttack`), armure effective, règle des résistances aux effets, Bouclier de glace renforcé | C1, C3, C5 à C8, C12, C13, C37, L1 | C12 : armure des Prowlers 3 au lieu de 1 | `combat.rs`, `modifier.rs`, `engine.rs`, `view.rs`, `advanced_actions.ron`, test `actions_avancees.rs` | non | fil principal |
| 2 | Coup de sang : option avec Blessure, `WoundFromCard`, assommé | C9 à C11 | C9 : pas de Blessure en main | `card.rs`, `event.rs`, `engine.rs` | non | fil principal |
| 3 | Mana en plus d'une option (`extra_mana`), Magie pure, Mélodie du vent | C19 à C23, L3 | C22 : option verte absente des actions légales | `card.rs`, `engine.rs`, `movement.rs` | non | fil principal |
| 4 | Marche revigorante et Régénération | C14 à C18 | C14 : Soin 0 dans la réserve | `card.rs`, `engine.rs` | non | fil principal |
| 5 | Récit héroïque et Besoin d'aide | C24 à C26 | C26 : Influence 3 au lieu de 6 | `modifier.rs`, `engine.rs`, `turn_fact.rs` | non | fil principal |
| 6 | Au cœur du combat : bonus d'unités, pas de dégâts sur les unités | C4, C27 à C29 | C27 : Attaque 2 au lieu de 4 | `modifier.rs`, `engine.rs`, `combat.rs` | non | fil principal |
| 7 | Cible ennemie d'`ActivateUnit`, Illusionnistes et Glaceurs d'Amotep (littéraux des tests mis à jour) | C2, C30, C31, L2, L4 | C30 : une créature est tirée | `action.rs`, `engine.rs`, `units.ron`, tests et joueur scripté | non | fil principal |
| 8 | Mages et Gardiens d'Altem | C32 à C35 | C32 : attaque physique | `modifier.rs`, `engine.rs`, `combat.rs`, `units.ron` | non | fil principal |
| 9 | Contenu complet, persistance (parties préparées régénérées si le format change) | C36 | C36 : une carte du lot encore `Unsupported` | `content/*.ron`, `arcanum-persistance` | non | fil principal |
| 10 | Libellés `fr.json` et `en.json` (section Libellés) | — | — | `assets/i18n/*.json` | oui | Haiku |
| 11 | Écran de combat : armure réduite, « n'attaque pas », pas de bouton Parer, choix de l'ennemi pour une capacité d'unité | C7, C30, C31, C37 | `combat_screen` : pas de choix d'ennemi proposé | `arcanum-ui` | non | fil principal |
| 12 | Options à second mana, Régénération sur une unité, assignation sans unité, effets en cours | C17, C22, C29 | `combat_screen` : bouton vers une unité avec Au cœur du combat | `arcanum-ui` | non | fil principal |
| 13 | Partie préparée (combat avec Illusionnistes et Bouclier de glace, exemple de la spec) et vérification à l'œil | — | — (vérification à l'œil) | `arcanum-persistance`, `scripts/` | non | fil principal |
| 14 | Docs : `architecture.md`, `PLAN.md` (implémentée), index par notion, bilan de fin de spec | — | — | `docs/` | oui | Sonnet |
