# Plan de développement — spec 059 Parcours du jalon 5

- Statut : suivi par le fil d'implémentation (spec validée le 2026-11-14 ; plan de la même forme que celui de la 049).
- Spec : [`059-parcours-j5.md`](../059-parcours-j5.md). Branche : `spec/059-parcours-j5`.
- Méthode : celle de `CLAUDE.md` (tests rouges, code minimal, refactor ; `scripts/verifier.sh` vert avant chaque push), avec
  le rouge visible (issue privée 106) et le rouge d'un parcours (plan de la 049, issue privée 145).

## Découpage en PR

Une seule PR (celle qui porte la spec, issue privée 304) : tests seulement, dans `crates/arcanum-core/tests/`.

## Découpage en fils

Un seul fil : 7 étapes, arcanum-core seul, pas d'écran (issue privée 118). **Contrôle des champs de la vue** (issue privée 130) : sans objet,
la spec n'a pas d'interface ; les points d'étape lisent des champs existants (`offers`, `skills`, `skill_choice`,
`common_skills`, `skill_pile_count`, `spell_rewards`, `hand`, `influence_points`, `move_rules`, `forced_withdrawal`,
`enemy_tokens`, `shields`, `final_score`).

## Le rouge d'un parcours

Les comportements vérifiés sont déjà implémentés (specs du J5) : chaque étape écrit d'abord ses points d'étape avec l'attente
du moteur **avant la règle** qu'ils vérifient, lance le test, voit l'échec sur cette assertion, commit local
`Tests rouges : …`, puis met l'attente juste. Plusieurs points rouges d'une même étape sont vus un par un (issue privée 145). Un
point rouge pour une autre raison est un défaut du moteur : signalé à Mathieu, corrigé avec un test unitaire de sa spec.

## Étapes

| # | Étape | Cas | Rouge attendu | Fichiers | Isolée ? | Modèle |
|---|---|---|---|---|---|---|
| 1 | Mise en place, exploration de 2A et de A + B, défenseur, assaut et conquête ; `chercher_une_graine` (ignoré) | C1 à C4, L1, L2 | C2 : gloire sans la tuile ; C4 : pas de récompense sort (`spell_rewards` à 0, attente d'avant la 043) | `tests/parcours_j5.rs` | non | fil principal |
| 2 | Fin du tour de la conquête : sort, compétence, action avancée, pioche | C5 à C8 | C6 : pile de compétences à 10 après le choix (attente d'avant la 051) | `tests/parcours_j5.rs` | non | fil principal |
| 3 | Tour suivant : compétence utilisée, action avancée et sort joués | C9 à C11 | C9 : compétence encore prête après usage (attente d'avant la 051) | `tests/parcours_j5.rs` | non | fil principal |
| 4 | Achat d'un sort à la tour, puis lancer | C12, C13 | C12 : offre de sorts à deux cartes après l'achat (sans complément, attente d'avant la 050) | `tests/parcours_j5.rs` | non | fil principal |
| 5 | Lac : Pont de glace renforcé, défi raté, retraite forcée | C14 à C17, L2 | C16 : pas de `ForcedWithdrawal` (attente d'avant la 052) | `tests/parcours_j5.rs` | non | fil principal |
| 6 | Nouvelle manche, tours d'exploration, fin de partie, score recompté, reprise ; graine définitive | C18 à C22, L3, L4 | C20 : Le Savant sans les sorts (attente d'avant la 054) | `tests/parcours_j5.rs` | non | fil principal |
| 7 | Docs : statut de la spec et de `PLAN.md`, index par notion ; bilan | — | — | `docs/` | oui | fil principal |

## Risques

- **Graine** : beaucoup de préconditions (défenseur battable, sort hors combat, compétence, Pont de glace ou Mélodie du vent
  dans l'offre, mana au bon tour, lac atteint à temps). `chercher_une_graine` dit pour chaque graine la première qui manque.
- **Durée du test** : une partie complète ; à garder sous quelques secondes.
