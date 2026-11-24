# Plan de développement — spec 060 Tactiques

- Statut : implémentée (fil moteur, étapes 1 à 11, et fil écran, étapes 12 à 15, le 2026-11-18). D'après la spec validée le
  2026-11-18 (Q7 : deux fils).
- Spec : [`060-tactiques.md`](../060-tactiques.md). Branche : `spec/060-tactiques`.
- Méthode : celle de `CLAUDE.md` (tests rouges, code minimal, refactor ; `scripts/verifier.sh` vert avant chaque push).

## Découpage en PR

Une seule PR (privée 329), celle de la spec.

## Découpage en fils

Deux fils successifs sur la même branche (spec, Q7 ; issue privée 118) :

1. **Moteur** : étapes 1 à 11 (C1 à C40, L1 à L9), tests existants adaptés, arms minimales de l'écran pour compiler.
2. **Écran** : étapes 12 à 15 (libellés, fenêtres et panneau, parties préparées, vérification à l'œil, docs).

**Contrôle de la vue** (issue privée 130) : tous les champs de la section *Interface* viennent du fil moteur (étape 10) :
`PlayerView::tactic_choice`, `removed_tactics`, `tactic`, `preparation_choice`, et les choix en attente (Faux départ, Vol de
mana, Provisions, Longue Nuit) dans les actions légales. Rien à recalculer à l'écran.

## Choix de modèle de données (fil moteur)

- **`Tactic`** (`tactic.rs`, nouveau module) : douze valeurs, `ALL`, `number()`, `time_of_day()`, `of()`, `id()` (clés `tactic.<id>.name`) ; pas de RON
  (ADR 0005, spec *Modèle*).
- **Règle du scénario** : `Scenario::tactics: TacticRule` (`None` | `Chosen { order: TacticOrder, removal: TacticRemoval }`),
  `serde(default)` à `None` (C40). `TacticOrder::PlayerFirst`, `TacticRemoval::AllTaken` seules codées ; le retrait passe par
  `removal`, jamais par un « si solo ».
- **Choix en attente** : posé par l'`apply` de `GameStarted` et de `RoundStarted` quand la règle est `Chosen` et qu'une
  tactique reste (L9) ; levé par `TacticChosen`. Après `RoundStarted`, `new_round_events` s'arrête après la pioche, sans
  `TurnStarted` : c'est la prise qui ouvre le tour 1. Le bac à sable n'a rien de changé (C4).
- **Offre** (remarque de Mathieu, grille 2 × 3) : la vue distingue, par tactique du moment, `Available`, `Removed` (trou) et
  `Taken(PlayerId)` (prise cette manche par un autre joueur, jamais produite en solo mais prévue pour la spec 061). Un seul
  type, `TacticSlot`, dans `PlayerView::tactic_choice`.
- **État du joueur** : `TacticState { tactic, flipped, stolen_die, stored, mana_search_used, right_moment }` ; les choix qui
  suivent la prise (`TacticFollowUp` : Faux départ, Vol de mana, Préparation), le choix de Provisions et l'arrêt de la pioche
  de Longue Nuit en champs d'attente, chacun en tête d'`available_actions` comme les autres choix (spec 056, 057).
- **Dé volé** : il reste à sa place dans `GameState::source`, exclu de la Source comme le dé de Puits de mana (`die_free`,
  spec 057) ; `TacticState::stolen_die` garde sa position. `ManaPayment::StolenDie` le paie (Polarisation : `PolarSource::StolenDie`).
- **Retrait** : `TacticsRemoved { tactics }` produit avec `RoundEnded` (`end_of_round`), d'après `removal`.

## Aide commune des tests existants

Les tests et parcours qui jouent Première reconnaissance prennent Lève-tôt (Au crépuscule de Nuit) avant leur première action,
sans changer leurs assertions (spec, *Effets sur les specs déjà implémentées*) : `take_no_effect_tactic` dans l'arène de
arcanum-core (et `Sandbox::events` ajoute `TacticChosen` quand le scénario a des tactiques), une aide équivalente dans le support de
arcanum-ui et d'arcanum-persistance. La PR liste les fichiers touchés.

## Étapes

| # | Étape | Cas | Rouge attendu | Fichiers | Isolée ? | Modèle |
|---|---|---|---|---|---|---|
| 1 | `Tactic`, `TacticRule`, choix et retrait ; Lève-tôt et Au crépuscule ; aide commune des tests existants | C1 à C8, L1, L2, L9 | C1 : pas de `ChooseTactic` légale (squelette : `ChooseTactic` jamais énumérée) | `tactic.rs`, `scenario.rs`, `action.rs`, `event.rs`, `state.rs`, `engine/` | non | fil principal |
| 2 | Au choix : Faux départ, Départ flamboyant, Préparation | C9, C10, C19, C30, L4, L5 | C9 : aucune `RethinkDiscard` légale après la prise | `engine/`, `event.rs`, `action.rs` | non | fil principal |
| 3 | Vol de mana : vol, paiement, retour | C11 à C15, L3 | C11 : aucune `StealDie` légale | `engine/`, `card.rs`, `event.rs` | non | fil principal |
| 4 | Planification | C16 à C18 | C16 : pioche jusqu'à 5 au lieu de 6 | `engine/events.rs` (`draw_limit`) | non | fil principal |
| 5 | Au bon moment | C20 à C22, L7, L8 | C20 : aucune `AnnounceRightMoment` légale | `engine/` | non | fil principal |
| 6 | Longue Nuit | C23 à C25 | C23 : aucune `UseLongNight` légale | `engine/` | non | fil principal |
| 7 | Appel du mana | C26, C27, L6 | C26 : aucune `ManaSearch` légale | `engine/` | non | fil principal |
| 8 | Méditation nocturne | C28, C29 | C28 : aucune `MidnightMeditation` légale | `engine/` | non | fil principal |
| 9 | Provisions | C31 à C34 | C31 : `TurnStarted` suit la prise au lieu du choix de Provisions | `engine/` | non | fil principal |
| 10 | Vue et filtrage du journal | C35, C36 | champs de la vue vides | `view.rs`, `event.rs` | non | fil principal |
| 11 | Retour en arrière, rejeu, persistance | C37 à C40 | C37 : `RightMomentAnnounced` bloque le retour (squelette de `blocks_undo`) ; C38 à C40 verts d'emblée possible (signalé) | `event.rs`, `arcanum-persistance/tests` | non | fil principal |
| 12 | Libellés `fr.json` et `en.json` (tableau de la spec), clés modifiées de Première reconnaissance | — | — | `assets/i18n/*.json` | oui | Haiku |
| 13 | Écran : fenêtre de choix en grille 2 × 3, choix qui suivent la prise, panneau de droite, dé volé dans le choix du mana, Provisions, Longue Nuit, raccourcis 1 à 6 et Entrée | interface | écrans calculés (cases et boutons proposés) | `arcanum-ui` | non | fil principal |
| 14 | Parties préparées `tactiques-jour`, `tactiques-nuit`, vérification à l'œil | interface | — (vérification à l'œil) | `arcanum-persistance`, `scripts/` | non | fil principal |
| 15 | Docs (`architecture.md`, `verification-manuelle.md`, `index-par-notion.md`, `glossaire.md`, `raccourcis.md`, PLAN) | — | — | `docs/` | oui | Sonnet |

Tests du fil moteur dans `crates/arcanum-core/tests/tactiques.rs` (C1 à C38, L1 à L9) et
`crates/arcanum-persistance/tests/tactiques.rs` (C39, C40).

## Risques repérés

- **Tests existants de Première reconnaissance** (une vingtaine de fichiers, arcanum-ui compris) : ils attendent tous le choix
  d'abord. Parade : l'aide commune à l'étape 1, assertions inchangées.
- **Ordre des actions légales** : les nouvelles actions du tour (`AnnounceRightMoment`, `UseLongNight`, `ManaSearch`,
  `MidnightMeditation`) vont après les cartes et avant les fins de tour ; les choix en attente en tête. Les tests choisissent
  par valeur.
- **Nombre d'actions** : Faux départ (jusqu'à 3 parmi 5 : 26 combinaisons) et Méditation nocturne (1 à 5 parmi la main :
  jusqu'à 31) restent raisonnables ; les Blessures identiques ne sont proposées qu'une fois par nombre.
- **Écran** : les `match` exhaustifs d'arcanum-ui sur `ManaPayment`, `Action` et `PlayerEvent` reçoivent des arms minimales au fil
  moteur ; leurs libellés au fil écran.

## Où en est l'implémentation (fin du fil moteur)

Fil moteur fini : C1 à C40 et L1 à L9 au vert (`crates/arcanum-core/tests/tactiques.rs`, 47 tests ;
`crates/arcanum-persistance/tests/tactiques.rs`, 2 tests). Le fil écran part d'ici.

**Ce que le moteur donne à l'écran** (rien à recalculer) :

- Vue : `PlayerView::tactic_choice` (les six tactiques du moment, `TacticSlot::Available`, `Removed` ou `Taken(joueur)`,
  seulement pendant le choix du joueur), `removed_tactics`, `tactic` (`TacticView` : tactique, retournée, dé volé et sa
  couleur, nombre de cartes sous Provisions, Appel du mana utilisé, Au bon moment annoncé), `preparation_choice` (cartes du
  deck triées, seulement pendant ce choix).
- Actions, toutes dans les actions légales : `ChooseTactic`, `RethinkDiscard { cards }`, `PrepareCard`, `StealDie`,
  `AnnounceRightMoment`, `UseLongNight`, `DeclineLongNight`, `ManaSearch { dice }`, `MidnightMeditation { cards }`,
  `SparingPower { take }` ; paiement `ManaPayment::StolenDie` (et `PolarSource::StolenDie`).
- Choix en attente, en tête des actions légales : la prise ; puis Faux départ, Vol de mana ou Préparation ; la pioche de fin de
  tour arrêtée (`UseLongNight` / `DeclineLongNight`) ; Provisions avant le tour (`SparingPower`, sans `take: false` deck vide).
- Journal : `TacticChosen`, `TacticsRemoved`, `DeckReshuffled`, `CardSearched`, `DieStolen`, `StolenDieUsed`,
  `StolenDieReturned`, `RightMomentAnnounced`, `LongNightUsed`, `DrawStopped`, `ManaSearchUsed`, `CardsShuffledIntoDeck`,
  `CardStored`, `StoredCardsTaken` : aucun libellé de journal encore.

**Écarts au modèle de la spec** (à relire au fil écran, rien ne change pour le joueur) :

- `DeckReshuffled { player, order }` (nouveau) au lieu de `DeckShuffled` pour Faux départ, Préparation et Méditation
  nocturne : `DeckShuffled` vide la main (mise en place), ici la main reste.
- `DrawStopped { player, missing }` (nouveau) pose l'arrêt de la pioche de Longue Nuit ; `ManaSearchUsed { player }`
  (nouveau) note Appel du mana utilisé avant les `DieRerolled`.
- Provisions : la prise est suivie de `TurnStarted` comme les autres, et le choix attend au début de chaque tour (posé par
  `TurnStarted`) ; la case de la clairière (jeton) se prend donc avant le choix de Provisions.
- Au bon moment s'annonce hors combat (tour libre ou interaction), pas pendant un combat.
- Méditation nocturne « avant le tour » : rien en zone de jeu, ni déplacement, ni dé, jeton ou cristal dépensé, ni unité
  activée, ni interaction, ni combat, ni fait du tour.
- Longue Nuit à la pioche de fin de tour : la défausse est regardée avant les défausses de fin de tour (cas d'une défausse
  vide qu'on remplit en fin de tour : pas d'arrêt).

**Arms provisoires d'arcanum-ui** (pour compiler) : `StolenDie` est ignoré par le choix du mana (`screen/mana.rs`), affiché en
ligne générique dans la zone de jeu, et rangé avec les invocations dans `ruins.rs` et `units.rs`. Le fil écran les remplace.

**À savoir pour le fil écran :**

- L'application n'a pas encore de fenêtre pour `ChooseTactic` : une partie de Première reconnaissance attend la prise dès la
  mise en place. Les tests d'arcanum-ui passent par `support::past_tactic_choice` et `started_session`.
- Pendant le choix d'une nouvelle manche, le compteur de tours montre encore le dernier tour de la manche finie (seul
  `TurnStarted` le remet à 1) : à masquer ou à remplacer par la manche pendant le choix.
- Les parties préparées (`scripts/parties/*.json`) ont été régénérées (champ `tactics` du scénario, `TacticChosen` des parties
  de Première reconnaissance).

## Fil écran (étapes 12 à 15)

Fait le 2026-11-18 : `crates/arcanum-ui/tests/tactiques.rs` (23 tests ; 22 vus rouges, `no_tactic_line_in_the_sandbox` vert d'emblée),
`screen/tactics.rs`, fenêtres dans `app/windows.rs`, ligne du panneau dans `app/side_panel.rs`, bandeau dans
`app/bottom_panel.rs`, raccourcis dans `session.rs`. Parties préparées `tactiques-jour`, `tactiques-manche-3`,
`tactiques-de-vole`, `tactiques-nuit`, `tactiques-longue-nuit` ; captures `/mnt/project-files/captures/spec-060-*.png`.

Écarts et ajouts au fil écran :

- Clé `tactic.stored` (« Dessous : {count} carte(s) ») ajoutée au tableau des libellés : la spec demande le nombre de cartes
  sous Provisions dans le panneau sans lui donner de clé.
- Pendant le choix de la tactique, la barre de menu montre « (tour 1) » au lieu du dernier tour de la manche finie.
- Préparation propose une carte par sorte (comme les actions légales) : deux Marche du deck font un seul bouton ; le test l'a
  appris au vert (assertion corrigée, signalée dans la PR).
- Appel du mana : un dé se coche si les dés cochés et lui forment un choix légal ; le dé or vient donc d'abord.
- Le test de la fiche de Première reconnaissance (arcanum-ui, `nouvelle_partie.rs`) compte 3 règles spéciales au lieu de 2 (clé
  modifiée par la spec).
