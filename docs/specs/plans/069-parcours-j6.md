# Plan de développement — spec 069 Parcours du jalon 6

- Statut : suivi par le fil d'implémentation (spec validée le 2026-11-19 ; plan de la même forme que celui de la 059).
- Spec : [`069-parcours-j6.md`](../069-parcours-j6.md). Branche : `spec/069-parcours-j6`.
- Méthode : celle de `CLAUDE.md` (tests rouges, code minimal, refactor ; `scripts/verifier.sh` vert avant chaque push), avec
  le rouge visible (issue privée 106) et le rouge d'un parcours (plan de la 049, issue privée 145).

## Découpage en PR

Une seule PR (celle qui porte la spec, issue privée 349) : tests seulement, dans `crates/arcanum-core/tests/`.

## Découpage en fils

Un seul fil : 4 étapes, arcanum-core seul, pas d'écran (issue privée 118). **Contrôle des champs de la vue** (issue privée 130) : sans objet,
la spec n'a pas d'interface ; les points d'étape lisent des champs existants (`tactic_choice`, `removed_tactics`, `tactic`,
`round_order`, `dummy`, `conquest`, `cities`, `face_down_defenders`, `last_turn`, `game_end`, `final_score`).

## Le rouge d'un parcours

Les comportements vérifiés sont déjà implémentés (specs du J6) : chaque étape écrit d'abord ses points d'étape avec l'attente
du moteur **avant la règle** qu'ils vérifient, voit l'échec sur cette assertion, commit local `Tests rouges : …`, puis met
l'attente juste. Plusieurs points rouges d'une même étape sont vus un par un (issue privée 145).

## Étapes

| # | Étape | Cas | Rouge attendu | Fichiers | Isolée ? | Modèle |
|---|---|---|---|---|---|---|
| 1 | Aides du support (ordre des tuiles royaume, choix des tactiques par le script, décisions gardées) ; parcours principal jusqu'au décompte ; balayage | C1 à C11, L1, L3 à L5 | C4 : 3 cartes par tour du fantôme, sans les cristaux (attente d'avant la 061) ; puis C6 : seule la tactique du joueur retirée (attente d'avant la 064) | `tests/support/`, `tests/parcours_j6.rs` | non | fil principal |
| 2 | Reprise au début de la manche 3 | C12 | C12 : aucune tactique retirée à la reprise (attente d'avant la 060, retrait hors du journal) | `tests/parcours_j6.rs` | non | fil principal |
| 3 | Épisode de la conquête : état posé, révélation, deux assauts, dernier tour, décompte | C13 à C19, L2, L6 | C16 : un tour du fantôme pendant le dernier tour (attente d'avant la 064) | `tests/support/arrangement.rs`, `tests/parcours_j6.rs` | non | fil principal |
| 4 | Docs : statut de la spec et de `PLAN.md`, index par notion, notes d'implémentation ; bilan | — | — | `docs/` | oui | fil principal |

## Risques

- **Explorateur bloqué** : orques, tours de mage, lacs, ou manches trop courtes face au fantôme. Mesuré : 16 graines sur 40
  révèlent une cité. Décision de Mathieu (2026-11-19) : les cités révélées sont une précondition de la graine du test seulement ;
  le balayage vérifie le reste, et C9 quand une cité sort (comme L5).
- **Combat de l'épisode** : quatre défenseurs de cité au niveau 8 ; le héros posé doit les prendre en deux assauts sans être
  assommé (L2).
