# Plan de développement — spec 041 Villages et monastères

- Statut : terminé (2026-11-04) ; rédigé au début du fil moteur, la spec étant validée avec son découpage en deux fils. Fil
  moteur terminé le 2026-11-04 (étapes 1 à 7), fil écran le même jour (étapes 8 à 11).
- Spec : [`041-villages-et-monasteres.md`](../041-villages-et-monasteres.md). Branche : `spec/041-villages-et-monasteres`.
- Méthode : celle de `CLAUDE.md` (tests rouges, code minimal, refactor ; `scripts/verifier.sh` vert avant chaque push), avec
  le rouge visible (issue privée 106) : squelette `todo!()` d'abord, commit local `Tests rouges : …` qui nomme les tests en échec et
  leur nombre (`scripts/verifier.sh --rouge`), section « Rouge vu » dans la PR.

## Découpage en PR

Une seule PR (privée 122, qui porte la spec) : acheter sans l'écran, ou incendier sans la récompense, laisserait le jeu bancal.

## Découpage en fils

Deux fils successifs sur la même branche (issue privée 118 ; spec, *Plan de développement*) :

1. **Fil moteur** : étapes 1 à 7 (C1 à C33, persistance, partie préparée `monastere`, docs du moteur). Il pousse sa dernière
   étape verte et s'arrête.
2. **Fil écran** : étapes 8 à 11 (libellés, C34 à C43, vérification à l'œil, docs de l'écran, bilan), à partir de la spec, de
   ce plan et de la branche.

## Choix de modèle

- **État** : `GameState::shields` (case → joueur) ; un monastère incendié est une case de monastère qui porte un Bouclier.
  `PlayerState` : `artifact_rewards` (récompenses du tour), `drawn_artifacts` (tirés, en attente du choix), `burning` (la case
  du monastère incendié ce tour : le combat qui suit exclut les unités), `plundered`, `acted` (une carte jouée, un pas, une
  action ce tour : plus de pillage).
- **Unités exclues** : `unit_actions` et `damageable_units` ne rendent rien pendant le combat d'un incendie ; la phase
  d'assignation se résout donc seule (spec 042, C29), sans toucher à `CombatStarted`.
- **Récompense** : `EndTurn` produit `ArtifactsDrawn` après le cristal de la mine, puis les niveaux et la défausse, et s'arrête
  avant la pioche ; `TakeArtifact` reprend comme `GainAdvancedAction` (choix d'action avancée en attente, ou pioche). L'ordre
  des décisions suit MKUE p. 11, 5 puis 6 : le choix de l'artefact passe avant celui de l'action avancée.
- **Monastères du plateau** : une fonction `monasteries(map)` (cases des monastères des tuiles posées), pour la mise en place,
  l'exploration et la nouvelle manche.

## Étapes

| # | Étape | Cas | Rouge attendu | Fichiers | Isolée ? | Modèle |
|---|---|---|---|---|---|---|
| 1 | **Actions avancées des monastères** : exploration, mise en place, nouvelle manche | C1 à C4, C6 | C1 : pas d'`OfferCardDealt` après `TileRevealed` | `engine.rs`, `board/`, nouveau `tests/villages_et_monasteres.rs`, `tests/arene/` | non | fil principal |
| 2 | **Acheter** : `BuyHealing`, `BuyAdvancedAction`, leurs événements | C7 à C14, L1, L2 | C7 : `BuyHealing` absente des actions légales | `action.rs`, `event.rs`, `engine.rs`, tests | non | fil principal |
| 3 | **Incendier** : `BurnMonastery`, jetons Bouclier, unités exclues, victoire ou échec | C5, C15 à C23, L4, L5, L7 | C15 : `BurnMonastery` absente | `action.rs`, `event.rs`, `engine.rs`, `combat.rs`, `state.rs`, tests | non | fil principal |
| 4 | **Récompense artefact** : `ArtifactsDrawn`, `TakeArtifact`, suite de la fin du tour | C24 à C27, L6 | C24 : pas d'`ArtifactsDrawn` à la fin du tour | `engine.rs`, `event.rs`, tests | non | fil principal |
| 5 | **Décompte, retour en arrière, sauvegarde, vue** | C28 à C31 | C28 : Le Collectionneur vaut 1 | `game_end.rs`, `view.rs`, tests arcanum-core et arcanum-persistance | non | fil principal |
| 6 | **Piller un village** : `PlunderVillage`, `VillagePlundered` | C32, C33 | C32 : `PlunderVillage` absente | `action.rs`, `event.rs`, `engine.rs`, tests | non | fil principal |
| 7 | `FirstChoice` (ni incendie ni pillage), partie préparée `monastere`, docs du moteur (`architecture.md`, glossaire) | — | — | `tests/premier_choix/`, `arcanum-persistance/tests/parties_preparees.rs`, `scripts/parties/`, `docs/` | non | fil principal |
| 8 | Libellés `fr.json`, `en.json` (tableau de la spec), clé `interaction.nothing_to_buy` retirée | — | — | `assets/i18n/*.json` | oui | Haiku |
| 9 | **Écran** : achats, fenêtre Offres, incendie et sa confirmation, combat sans unités, plateau, récompense, score, pillage | C34 à C43 | C34 : pas de bouton d'achat de soin | `arcanum-ui/src/screen/`, `session.rs`, `app/`, `arcanum-ui/tests/` | non | fil principal |
| 10 | Vérification à l'œil (partie `monastere`), captures | — | — | `docs/verification-manuelle.md` | non | fil principal |
| 11 | Docs : `architecture.md` (écran), `raccourcis.md`, `index-par-notion.md`, `PLAN.md` (implémentée) ; bilan de fin de spec | — | — | `docs/` | oui | fil principal |

## Risques repérés

- **Ordre des actions légales** : `PlunderVillage` en tête au début du tour décale les index ; les tests et joueurs choisissent
  par valeur, `FirstChoice` l'écarte comme `Interact` (sinon ses parties changent).
- **Parcours** : ceux qui explorent la campagne 5 voient une carte de plus dans l'offre d'unités ; leurs points d'étape ne la
  vérifient pas.
- **Retour en arrière après `TakeArtifact`** : la pioche de fin de tour est un point de non-retour ; le retour n'est possible que
  si un choix d'action avancée suit (C29 testé ainsi).
- **`FinalScore`** gagne deux champs : les tests d'arcanum-ui qui le construisent champ par champ sont à compléter.
