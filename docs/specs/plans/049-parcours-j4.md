# Plan de développement — spec 049 Parcours du jalon 4

- Statut : accepté par Mathieu avec la spec (2026-11-06).
- Spec : [`049-parcours-j4.md`](../049-parcours-j4.md). Branche : `spec/049-parcours-j4`.
- Méthode : celle de `CLAUDE.md` (tests rouges, code minimal, refactor ; `scripts/verifier.sh` vert avant chaque push), avec
  le rouge visible (issue privée 106).

## Découpage en PR

Une seule PR (celle qui porte la spec) : tests seulement, dans `crates/arcanum-core/tests/`.

## Découpage en fils

Un seul fil : 5 étapes, arcanum-core seul, pas d'écran (issue privée 118). **Contrôle des champs de la vue** (issue privée 130) : sans objet, la
spec n'a pas d'interface ; les points d'étape lisent des champs existants (`offers`, `units`, `face_down_defenders`,
`enemy_tokens`, `shields`, `fame`, `level`, score de fin).

## Le rouge d'un parcours

Les comportements vérifiés sont déjà implémentés (specs du J4) : un point d'étape **vert du premier coup est attendu** et signalé
dans la PR, comme le prévoit le gabarit. Pour montrer qu'il peut échouer, chaque étape écrit d'abord ses points d'étape avec
l'attente du moteur **avant la règle** qu'ils vérifient (valeur d'avant : gloire sans la tuile, unité absente, pas de Bouclier,
score sans la ligne du Haut Fait…), lance le test, voit l'échec sur cette assertion, commit local `Tests rouges : …`, puis met
l'attente juste. Un point d'étape **rouge pour une autre raison** que cette attente est un défaut du moteur : il est signalé à
Mathieu, et corrigé dans la PR avec un test unitaire dans le fichier de test de sa spec.

## Étapes

| # | Étape | Cas | Rouge attendu | Fichiers | Isolée ? | Modèle |
|---|---|---|---|---|---|---|
| 1 | Mise en place, marche et recrutement ; graine provisoire et `chercher_une_graine` (ignoré) | C1 à C3, L1 | C3 : unité absente de la vue après `RecruitUnit` (attente d'avant la 042) | `tests/parcours_j4.rs` | non | fil principal |
| 2 | Exploration, défenseur, assaut, combat avec l'unité, conquête ; niveau 2 et action avancée | C4 à C10, L2 | C8 : aucun Bouclier sur `(7, -5)` ; C9 : pas d'`OfferCardGained` à la fin du tour | `tests/parcours_j4.rs` | non | fil principal |
| 3 | Explorateur scripté (support), manche 2, tours d'exploration jusqu'à la cité, dernier tour, fin de partie | C11 à C15, L3, L4 | C12 : gloire sans la tuile révélée (attente « 0 par tuile » du bac à sable) | `tests/support/mod.rs`, `tests/parcours_j4.rs` | non | fil principal |
| 4 | Décompte recompté ; graine définitive choisie par `chercher_une_graine` ; reprise depuis la sauvegarde | C16, C17 | C16 : Le Conquérant à 0 (attente d'avant la 043) | `tests/parcours_j4.rs` | non | fil principal |
| 5 | Docs : statut de la spec et de `PLAN.md`, notes d'implémentation, `docs/architecture.md` si le support change ; bilan | — | — | `docs/` | oui | Haiku |

## Choix du modèle par étape

Étapes 1 à 4 au fil principal : elles se suivent sur le même fichier et dépendent de la graine. L'étape 5 peut être déléguée
(brief court : statuts, ligne de `PLAN.md`, notes).

## Risques

- **Graine** : quatre préconditions (unité de village utile, défenseur battable, mains, cité avant la fin de la 4e manche).
  `chercher_une_graine` dit pour chaque graine la première qui manque, comme dans la 039.
- **Durée du test** : une partie complète, une quinzaine de tours d'exploration ; à garder sous quelques secondes en `cargo test`
  (le nombre d'actions légales par décision reste petit).
- **Explorateur bloqué** (aucune case d'où explorer n'est atteignable sans provoquer) : le test échoue en le disant, et la graine
  est à rechoisir.
