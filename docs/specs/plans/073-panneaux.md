# Plan de développement — spec 073 Panneau du bas et panneau de droite

- Statut : réalisé (2026-11-10) : fil 1 (étapes 1 à 4, moteur) et fil 2 (étapes 5 à 13, écran).
- Spec : [`073-panneaux.md`](../073-panneaux.md). Branche : `spec/073-panneaux`.
- Méthode : celle de `CLAUDE.md` (tests rouges, code minimal, refactor ; `scripts/verifier.sh` vert avant chaque push).

## Découpage en PR

Une seule PR, la privée 174 de la rédaction.

## Découpage en fils

Deux fils, comme la spec le propose (moteur et écran, issue privée 118) :

1. étapes 1 à 4 : faits du tour et jetons dépensés (arcanum-core), fonctions pures de disposition (arcanum-ui, sans egui), docs du fil ;
2. étapes 5 à 13 : libellés, écran, raccourcis, vérification à l'œil, docs.

**Contrôle des champs de la vue** (issue privée 130) : `turn_facts` et `spent_mana_tokens` sont créés par les étapes 1 et 2 ; tous
les autres champs de la section Interface de la spec existent déjà dans `PlayerView` (`hand`, `discard`, `deck_size`,
`units`, `fame`, `level`, `next_level_fame`, `mana_tokens`, `play_area`, récompenses en attente, `combat`, `game_end`).

## Choix de conception

- `crates/arcanum-core/src/turn_fact.rs` (nouveau) : `TurnFact`, un variant par ligne du tableau de la spec (`TileRevealed`,
  `EnemiesDefeated { enemies, fame }`, `FameGained`, `ReputationChanged`, `WoundsTaken { count }`, `WoundsHealed`,
  `CrystalGained`, `CardGained { card }`, `UnitGained`, `UnitLost`, `UnitWounded`, `UnitHealed`, `SiteConquered { hex }`,
  `VillagePlundered`, `MonasteryBurned { hex }`) ; `turn_fact(&Event) -> Option<(PlayerId, TurnFact)>`, appelée par `apply`
  pour chaque événement (le fait suit l'événement tel que `decide` l'a validé).
- `PlayerState::turn_facts`, vidé par `TurnStarted` ; `PlayerState::spent_mana_tokens`, rempli par `ManaTokenUsed` (jeton
  présent), vidé par `TurnEnded` avec `mana_tokens`. Accesseurs publics de `GameState`, champs de `PlayerView`.
- Retour en arrière et sauvegarde : rien à faire, l'état est rejoué ou restauré (spec 024).
- `crates/arcanum-ui/src/layout.rs` (nouveau, sans egui) : `CARD_WIDTH` (200), `CARD_HEIGHT` (212), `CARD_GAP` (8),
  `MIN_SECTION_WIDTH` (400) ; `row_width(n)` (largeur naturelle) ; `section_widths(naturelles, disponible)` ;
  `fan_layout(n, w, survolée) -> FanLayout` : abscisses, ordre de dessin (la survolée en dernier), et `card_at(x)`, la carte
  du dessus sous le pointeur.

## Étapes

| # | Étape | Cas | Rouge attendu | Fichiers | Isolée ? | Modèle |
|---|---|---|---|---|---|---|
| 1 | Faits du tour | C1 à C5 | C1 à C5 : `turn_fact` rend `None`, la liste reste vide | `turn_fact.rs`, `state.rs`, `view.rs`, `engine.rs`, `lib.rs`, `tests/faits_du_tour.rs` | non | fil principal |
| 2 | Jetons dépensés | C6 | C6 : `spent_mana_tokens` jamais rempli | `state.rs`, `view.rs`, `engine.rs`, `tests/faits_du_tour.rs` | non | fil principal |
| 3 | Disposition des cartes | C9 à C12 | C9 à C12 : fonctions en `todo!()` (panique, donc échec à l'exécution) puis rendus vides | `arcanum-ui/src/layout.rs`, `lib.rs`, `arcanum-ui/tests/panneaux.rs` | non | fil principal |
| 4 | Docs du fil 1 : `architecture.md`, statut du plan | — | — | `docs/` | oui | fil principal |
| 5 | Libellés `fr.json` et `en.json` (tableau de la spec) | — | — | `assets/i18n/*.json` | oui | Haiku |
| 6 | Panneau du bas : sections, cartes de la main, boutons d'effet | C7, C8, C13 à C19 | à préciser par le fil 2 | `screen/` | non | fil principal |
| 7 | Unités : boutons d'effet par option | C20 à C22 | idem | `screen/units.rs` | non | fil principal |
| 8 | Panneau de droite : joueur, deck et défausse, Ici, zone de jeu, ce tour | C23 à C29 | idem | `screen/` | non | fil principal |
| 9 | Boutons du bas | C30 à C34 | idem | `screen/end_turn.rs`, `session.rs` | non | fil principal |
| 10 | Fenêtre de la défausse, touche D | C35, C36 | idem | `screen/`, `session.rs`, `keys.rs` | non | fil principal |
| 11 | Combat sans main ni unités | C37 | idem | `screen/combat.rs` | non | fil principal |
| 12 | Dessin (`app/`) : trois zones, chevauchement animé | — (à l'œil) | — | `app/*.rs` | non | fil principal |
| 13 | Vérification à l'œil (parties `unites`, `combat-deux-ennemis`, `monastere`, `clairiere`) ; docs : `architecture.md`, `raccourcis.md`, glossaire, `PLAN.md`, `index-par-notion.md` | — | — | `docs/` | oui | Sonnet |

## Risques repérés

- `UnitWounded` est le seul événement « unité blessée » du tableau : les Blessures assignées en combat
  (`DamageAssignedToUnit`) ne font pas de fait, comme la spec le dit (Q6). À revoir avec Mathieu si l'écran le demande.
- C6 : la spec cite Détermination renforcée, une Parade, qui ne se joue qu'en combat ; le test prend Endurance (même couleur,
  Mouvement), qui se paie de la même façon hors combat.
