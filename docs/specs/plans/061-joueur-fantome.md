# Plan de développement — spec 061 Joueur fantôme

- Statut : réalisé (fil moteur, étapes 1 à 7, puis fil écran, étapes 8 à 12, 2026-11-19).
- Spec : [`061-joueur-fantome.md`](../061-joueur-fantome.md). Branche : `spec/061-joueur-fantome`.
- Méthode : celle de `CLAUDE.md` (tests rouges, code minimal, refactor ; `scripts/verifier.sh` vert avant chaque push).

## Découpage en PR

Une seule PR, la privée 336 de la rédaction.

## Découpage en fils

Deux fils (spec, Q6 ; issue privée 118) :

1. **fil moteur** : étapes 1 à 7 (arcanum-core, arcanum-persistance, tests existants passés « sans fantôme ») : C1 à C36, L1 à L10 ;
2. **fil écran** : étapes 8 à 12 (libellés, panneau, défausse, ordre du tour, bandeau, étape Options, fiche du scénario,
   parties préparées, vérification à l'œil, docs) : C37 à C42.

**Contrôle des champs de la vue** (issue privée 130) : la section Interface lit `PlayerView::dummy` (héros, tactique, deck,
défausse, cristaux, compétences cachées, cartes du dernier tour, annonce) et `PlayerView::round_order`, créés par l'étape 7 ;
`dummy.last_turn_extra` se déduit des cartes retournées (couleur de la 3e, nombre au-delà de 3). L'étape Options lit
`Scenario::dummy()`, `Scenario::without_dummy()`, `GameConfig::dummy_hero` et `GameConfig::validate()`, créés par
l'étape 1.

## Choix de conception

- **Contenu** : `dummy_crystals: { Arythea: [Red, Red, White], … }` au niveau du fichier `heroes.ron`, lu dans
  `GameContent::dummy_crystals(hero)` ; `parse` refuse une entrée qui n'a pas trois couleurs (`ContentError::InvalidDeck`,
  raison explicite) et exige les quatre héros.
- **Scénario** : `DummyRule { None, Standard }` (`#[serde(default)]`, `None`) dans `dummy.rs` ;
  `Scenario::dummy()`, `Scenario::without_dummy()` (constructrice de l'option), `Scenario::with_dummy(rule)` qui refuse
  `Standard` sans tactiques (`ScenarioError::DummyWithoutTactics`, L9). `first_reconnaissance()` a `Standard`.
- **Configuration** : `GameConfig::dummy_hero: DummyHeroChoice { Random, Hero(Hero) }` (`#[serde(default)]`) ;
  `GameConfig::validate() -> Result<(), ConfigError>` (`ConfigError::DummyHeroIsPlayerHero`, L10). Les ~120 littéraux
  `GameConfig { … }` des tests gagnent `dummy_hero: Default::default()`.
- **Hasard** : `RngUsage::Dummy` (flux 6, `"dummy"`).
- **État** : `GameState::dummy: Option<DummyState>` (héros, deck `Vec<CardId>`, défausse, cristaux par couleur sans limite,
  tactique, pile de compétences, cartes du dernier tour, `announced`). Accesseur public `GameState::dummy()`.
  `GameState::round_order() -> Option<RoundOrder>` (`PlayerFirst`, `DummyFirst`), déduit des deux tactiques.
- **Identité des cartes** : les exemplaires du fantôme sont numérotés à partir de `DUMMY_COPY_OFFSET` (100) : Rage n° 101,
  n° 102…, l'action avancée ajoutée garde son identifiant (exemplaire unique de l'offre).
- **Tour du fantôme** : une passe finale de `decide`, `with_dummy_turns`, rejoue les événements produits et insère avant
  chaque `TurnStarted` le tour du fantôme quand il doit jouer (tour 1 : sa tactique a le plus petit numéro ; tour suivant :
  toujours, sauf le tour de plus d'Au bon moment ; jamais après la cité ni après son annonce). Ainsi `ChooseTactic`,
  `RethinkDiscard`, `StealDie`, `PrepareCard` et toutes les fins de tour passent par le même code. Après son annonce, si le
  joueur n'a plus rien en main ni dans le deck, `RoundEnded` suit `TurnStarted` (L5).
- **Fin de la manche annoncée par le fantôme** : `turn_end_draw` rend `RoundEnded` au lieu de la pioche et du tour suivant
  (après les tours supplémentaires) ; une passe de `decide`, `with_round_end`, complète un `RoundEnded` final par la suite de
  `end_of_round` (retrait des tactiques, nouvelle manche ou fin de partie), calculée sur l'état qui suit tous les événements.
  `end_of_round` est découpée en `RoundEnded` + `after_round_end`. La fin de partie par la cité y teste
  `last_turn != NotAnnounced` (C30).
- **Annonce du joueur** : `DummyTurnPlayed` (dernier tour du fantôme) puis `RoundEnded`, complétée par `with_round_end`.
- **Tactique du fantôme** : `choose_tactic_events` reçoit `rngs` et ajoute `DummyTacticDrawn` après `TacticChosen`.
- **Nouvelle manche** : `refreshed_offers` remplace l'`OfferCardReturned` de l'action avancée par `DummyCardAdded`, ajoute
  `DummyCrystalGained` après celui du sort ; `new_round_events` ajoute `DummyDeckShuffled` après le mélange du joueur.
- **Compétences** : après `SkillGained` (choix du niveau pair), `DummySkillRevealed` si sa pile n'est pas vide.
- **Vue** : `PlayerView::dummy: Option<DummyView>` et `PlayerView::round_order` ; journal filtré : `DummySetUp` sans l'ordre
  du deck ni de la pile (cartes triées), `DummyDeckShuffled` sans ordre (nombre de cartes).

## Étapes

| # | Étape | Cas | Rouge attendu | Fichiers | Isolée ? | Modèle |
|---|---|---|---|---|---|---|
| 1 | Contenu, scénario, configuration, mise en place ; tests existants « sans fantôme » | C1 à C5, L9, L10 | C1, C3 : pas de `DummySetUp` (squelette vide) ; C2 : `dummy_crystals` vide ; L9, L10 : validations qui acceptent | `heroes.ron`, `content.rs`, `dummy.rs`, `scenario.rs`, `config.rs`, `rng.rs`, `state.rs`, `event.rs`, `engine/mod.rs`, `apply/`, `tests/joueur_fantome.rs`, tests existants | non | fil principal |
| 2 | Tactique du fantôme, ordre du tour, tour du fantôme | C6 à C16, L1, L6 | C6 : pas de `DummyTacticDrawn` ; C7, C8, C11 à C15 : pas de `DummyTurnPlayed` | `engine/events/round.rs`, `engine/decide.rs`, `engine/events/dummy.rs`, `apply/` | non | fil principal |
| 3 | Fins de manche | C17 à C21, L2, L3, L5 | C17 : `AnnounceEndOfRound` légale, pas de `RoundEnded` ; C21 : pas de `DummyTurnPlayed` | `engine/events/turn_end.rs`, `round.rs`, `legal/`, `decide.rs` | non | fil principal |
| 4 | Nouvelle manche | C22 à C25, L7 | C22 : `OfferCardReturned` au lieu de `DummyCardAdded` ; C23 : pas de cristal ; C24 : deck de 16 | `round.rs`, `apply/` | non | fil principal |
| 5 | Compétences du fantôme | C26 à C28 | C26 : pas de `DummySkillRevealed` | `decide.rs`, `apply/progression.rs` | non | fil principal |
| 6 | Fin de partie | C29, C30 | C29 : `DummyTurnPlayed` après la cité ; C30 : pas de `GameEnded` | `turn_end.rs`, `round.rs`, `dummy.rs` | non | fil principal |
| 7 | Vue, journal filtré, retour en arrière, persistance, journal rejoué | C31 à C36 | C31 : `PlayerView::dummy` vide (squelette) ; C32 à C36 : peuvent passer d'emblée (classement exhaustif des événements dès leur création), signalé dans la PR | `view.rs`, `event.rs`, `arcanum-persistance/tests/` | non | fil principal |
| 8 | Libellés `fr.json` et `en.json` (tableau de la spec) | — | — | `assets/i18n/*.json` | oui | Haiku |
| 9 | Section du panneau, fenêtre de défausse, ordre du tour, bandeau | C37 à C40 | lignes calculées vides | `arcanum-ui` | non | fil principal |
| 10 | Étape Options, récapitulatif, fiche du scénario | C41, C42 | case absente, liste vide | `arcanum-ui/src/new_game.rs` | non | fil principal |
| 11 | Parties préparées, vérification à l'œil | — | — (vérification à l'œil) | `arcanum-persistance`, captures | non | fil principal |
| 12 | Docs : `architecture.md`, `verification-manuelle.md`, `index-par-notion.md`, `glossaire.md`, `raccourcis.md`, `PLAN.md` | — | — | `docs/` | oui | Sonnet |

## Notes du fil moteur (pour le fil écran)

- La mise en place arrangée (`tests/support/arrangement.rs`) connaît `DummySetUp` et range le deck du fantôme
  (`Setup::dummy_deck`) ; le héros du fantôme se choisit par `GameConfig::dummy_hero`, pas par l'arrangement.
- `docs/architecture.md` n'est pas encore à jour (étape 12) : y décrire `dummy.rs`, `engine/events/dummy.rs`,
  `engine/apply/dummy.rs`, les deux passes finales de `decide` (`with_dummy_turns`, `with_round_end`) et
  `PlayerView::dummy` / `round_order`.
- Passés d'emblée au rouge (signalé dans les commits) : C16, C28, C29, C32 à C36, L7.

## Notes du fil écran

- La ligne d'ordre du tour est en tête du panneau de droite, avec les bandeaux : l'en-tête de manche est dans la barre de menu
  depuis la spec 074.
- Retours de Mathieu (2026-11-19) : cristaux du fantôme en icônes de l'inventaire à la suite, sans emplacement vide ; pas de
  bandeau pour son annonce, seulement `dummy.announced` en orange dans sa section.
- Le récapitulatif réutilise `new_game.recap_choice` pour le second « · » (pas de clé nouvelle).
- Les choix de l'étape Options sont retenus par la configuration de la partie (`GameStarted`), au lancement comme à Ctrl+N.
- Vérification à l'œil : `fantome-premier`, `fantome-annonce` et l'étape Options parcourus (captures `spec-061-*` du dossier du
  projet) ; le chemin de `fantome-competence` est prouvé par son test (`check`) et par C26, pas rejoué à l'œil.

## Risques repérés

- **Tests existants** : Première reconnaissance prend un fantôme ; les tests et parcours qui la jouent passent par
  `Scenario::first_reconnaissance().without_dummy()` sans changer leurs assertions (spec, *Effets sur les specs déjà
  implémentées*), signalé dans la PR.
- **Ordre des générateurs** : le fantôme a son propre flux ; la carte, la Source et le deck du joueur restent ceux de la
  graine (C3).
- **Passes finales de `decide`** : elles rejouent les événements (coût en `apply`) ; acceptable, une décision produit peu
  d'événements.
