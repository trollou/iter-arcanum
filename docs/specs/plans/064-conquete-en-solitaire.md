# Plan de développement — spec 064 Conquête en solitaire

- Statut : fait (fil moteur, étapes 1 à 5, et fil écran, étapes 6 à 10, le 2026-11-19).
- Spec : [`064-conquete-en-solitaire.md`](../064-conquete-en-solitaire.md). Branche : `spec/064-conquete-en-solitaire`.
- Méthode : celle de `CLAUDE.md` (tests rouges, code minimal, refactor ; `scripts/verifier.sh` vert avant chaque push).

## Découpage en PR

Une seule PR (privée 347), en brouillon jusqu'à la fin du fil écran.

## Découpage en fils

Le plan a 10 étapes et touche le moteur et l'écran : deux fils (issue privée 118).

- **Fil 1 (moteur)** : étapes 1 à 5 (C1 à C25, L1 à L8). Le fil adapte arcanum-ui juste ce qu'il faut pour compiler (nouvelles
  variantes de `ScenarioId` et `GameEndReason`), sans rien afficher de neuf ; `ScenarioId::ALL` ne change qu'au fil 2 (C26).
- **Fil 2 (écran)** : étapes 6 à 10 (C26 à C33), parties préparées, vérification à l'œil, docs.

**Contrôle des champs de vue** (issue privée 130) : C26 lit `ScenarioId::ALL` et `Scenario::of` (étape 1) ; C27 lit
`Scenario::dummy_optional` (étape 1) ; C28 lit `PlayerView::conquest` (étape 4) ; C29 lit `PlayerView::last_turn` et
`PlayerView::scenario` (existants, `last_turn` posé par l'étape 3) ; C30, C31 lisent `PlayerView::final_score` (partie
« conquête », étape 4) et `PlayerView::game_end` (raison `AllCitiesConquered`, étape 3) ; C32 lit `PlayerView::scenario` (forme
`StandardWedge`, étape 1) et `PlayerView::next_tile` (existant), et le verdict `Placement::CoreOnCoastline` (étape 2).

## Étapes

| # | Étape | Cas | Rouge attendu | Fichiers | Isolée ? | Modèle |
|---|---|---|---|---|---|---|
| 1 | Scénario : `ScenarioId::SoloConquest`, `MapShape::StandardWedge`, `SpellSet`, `ScenarioEnd::AllCitiesConquered`, `FinalScoring::SoloConquest`, `Scenario::solo_conquest`, `dummy_optional`, `with_end` et `ConquestWithoutCities` ; sorts compétitifs retirés de la mise en place | C1 à C5, L7 | C1 : valeurs du bac à sable ; C3 : 20 sorts | `scenario.rs`, `offer.rs`, `engine/mod.rs`, `arcanum-ui` (compilation) | non | fil principal |
| 2 | Côte : `Placement::CoreOnCoastline` | C6 à C9, L6 | C6 : `Allowed` sur la côte | `exploration.rs` | non | fil principal |
| 3 | Fin par conquête : `LastTurnAnnounced` après la conquête de la dernière cité, `GameEndReason::AllCitiesConquered` | C10 à C15, L2, L4 | C11 : pas de `LastTurnAnnounced` | `engine/decide.rs`, `engine/mod.rs`, `engine/events/`, `game_end.rs` | non | fil principal |
| 4 | Décompte `ConquestScore`, vue `PlayerView::conquest` | C16 à C23, L8 | C16 : partie « conquête » absente | `game_end.rs`, `view.rs` | non | fil principal |
| 5 | Sauvegarde : dernier tour rechargé, sauvegarde d'avant (sorts, forme) | C24, C25 | C25 : champ `spells` exigé | `arcanum-persistance/tests` | non | fil principal |
| 6 | Libellés `fr.json` et `en.json` (tableau de la spec) | — | — | `assets/i18n/*.json` | oui | Haiku |
| 7 | Écran de nouvelle partie : liste, fiche, fantôme obligatoire | C26, C27 | C26 : deux scénarios | `arcanum-ui/src/new_game.rs`, `scenario.rs` (`ALL`) | non | fil principal |
| 8 | Panneau : progression des cités, dernier tour après conquête | C28, C29 | C28 : ligne absente | `arcanum-ui/src/` | non | fil principal |
| 9 | Fenêtre de fin, infobulle de la côte, clés | C30 à C33 | C30 : lignes de conquête absentes | `arcanum-ui/src/screen/score.rs`, `arcanum-ui/src/board/` | non | fil principal |
| 10 | Parties préparées, vérification à l'œil, docs (`architecture.md`, `verification-manuelle.md`, `index-par-notion.md`, `glossaire.md`, `PLAN.md`) | — | — (vérification à l'œil) | `arcanum-persistance`, `docs/` | non | fil principal |

## Choix du modèle par étape

Tout le fil moteur au fil principal : les étapes se suivent sur `scenario.rs`, `decide.rs` et `game_end.rs`. Seuls les
libellés (étape 6) se délèguent.

## Risques repérés

- `FinalScore` gagne un champ (`conquest`) : le test de la spec 063 qui l'écrit en entier (C14) suit, signalé dans la PR.
- `GameEndReason` gagne une variante : `arcanum-ui/src/screen/score.rs` doit la connaître pour compiler dès le fil 1.
- L'annonce de la fin de la manche pendant la dernière manche se lit dans `round_over` (vrai entre `RoundEnded` et la manche
  suivante, qui ne vient pas après `GameEnded`) : rien de nouveau dans l'état.
