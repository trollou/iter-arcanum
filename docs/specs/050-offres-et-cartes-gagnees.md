# 050 — Offres et cartes gagnées

- Statut : validée (2026-10-31) ; propositions de Q1 à Q8 retenues par Mathieu, noms français des cartes suivis par l'issue privée 105
- Jalon : J4
- Dépend de : 012 (mise en place, scénario), 013 (jouer des cartes, de côté), 016 (fin du tour, pioche), 022 (effets renforcés),
  023 (nouvelle manche), 024 (sauvegarde, points de non-retour), 029 (contenu en données), 035 (niveaux), 038 (un générateur par
  usage), 063 (Première reconnaissance, décompte) ; toutes implémentées
- Livret : MKUE p. 3 (*Game Setup* : pioches et offres) ; p. 4 (2.c à 2.e : nouvelle offre d'unités, renouvellement des offres ;
  *Deed Cards* 3) ; p. 5 (*Using Units* 1 ; *"Gain" Effects* 3) ; p. 11 (*End of the Turn* 6.b et 7) ; p. 15 (*Standard
  Achievements Scoring*, Le Savant) ; p. 18 (*First Reconnaissance*, *Special Rules*). Compendium, *Setup* S1, *Miscellaneous* S7 et
  S8, *Silver Units* S1, *End of Turn* S7, *Map Revealing* S1. Texte des cartes : `docs/regles/cartes/actions-avancees.md`,
  `sorts.md`, `artefacts.md`, `unites.md`
- Crates touchées : arcanum-core (contenu, état, moteur), arcanum-ui, arcanum-persistance (une partie préparée), `assets/i18n/`

## Objectif

Les trois **offres** sont sur la table dès la mise en place : actions avancées, sorts, unités ; avec elles les pioches d'actions
avancées, de sorts, d'artefacts et d'unités (régulières et d'élite). Elles se renouvellent à chaque nouvelle manche, avec des unités
d'élite une fois une tuile royaume révélée. En atteignant un **niveau pair**, le joueur **gagne une action avancée** de l'offre,
posée sur son deck : il la pioche aussitôt et peut la jouer. Toutes les cartes du jeu de base sont décrites en données ; une carte
dont l'effet n'est pas encore codé se joue de côté (MKUE p. 4, 3.e) jusqu'à la spec qui le code (052 à 055, 042 pour les unités).

Une fenêtre **Offres** montre les trois offres à tout moment, et sert au choix de l'action avancée.

## Règles

### Pioches et offres à la mise en place

- **Pioches communes** (MKUE p. 3, *Spells deck*, *Advanced Action deck*, *Artifact deck*, *Regular Unit deck and Elite Unit deck*) :
  cinq pioches mélangées à la mise en place, face cachée : actions avancées, sorts, artefacts, unités régulières, unités d'élite.
  Chacune a son générateur (ADR 0007, spec 038) : ajouter ces tirages ne change ni les tuiles, ni la Source, ni le deck, ni les
  jetons d'ennemis d'une graine donnée (les graines des parcours restent valables).
- **Offres d'actions avancées et de sorts** (MKUE p. 3) : les trois premières cartes de leur pioche, face visible. Une offre est
  **ordonnée** : une carte nouvelle entre en **première place** (*top position*) ; la carte en **dernière place** (*lowest position*)
  est celle qui part au renouvellement (p. 4, 2.d).
- **Offre d'unités** (MKUE p. 3 ; Compendium, *Setup* S1) : *actual players plus 2*, donc **3** en solo (le joueur fantôme ne
  compte pas), tirées de la pioche d'unités régulières.
- **Artefacts** (MKUE p. 3) : pioche seulement, jamais d'offre. Les tirer (récompenses) : specs 041, 044.
- **Les tirages sont résolus dans les événements** (ADR 0001) : l'ordre de chaque pioche est dans l'événement de mélange, caché au
  joueur (il n'en voit que le nombre de cartes), comme la pioche de tuiles et les piles d'ennemis.

### Renouvellement à chaque nouvelle manche

Dans l'ordre du livret, après la Source (p. 4, 2.b) et avant le mélange du deck (2.g) :

- **Nouvelle offre d'unités** (MKUE p. 4, 2.c) : les unités de l'offre retournent **sous** leur pioche (régulière ou d'élite), puis 3
  unités sont tirées :
  - aucune tuile royaume révélée : 3 régulières ;
  - au moins une tuile royaume révélée, et le scénario utilise les unités d'élite : **élite, régulière, élite** (Compendium,
    *Silver Units* S1 : on commence toujours par une élite) ;
  - le scénario n'utilise pas les unités d'élite (Première reconnaissance) : 3 régulières.
  Les actions avancées des monastères dans l'offre d'unités (2.c, derniers points) : spec 041 (Q4).
- **Offre d'actions avancées** (MKUE p. 4, 2.d) : la carte en dernière place va **sous** la pioche, les autres descendent d'une
  place, une carte tirée entre en première place. Le joueur fantôme qui la prend à la place (p. 15) : spec 061.
- **Offre de sorts** (MKUE p. 4, 2.e) : de même. Le cristal du joueur fantôme (p. 15) : spec 061.

### Gagner une action avancée à un niveau pair

- **Changement de niveau pair** (MKUE p. 11, 6.b) : à un niveau marqué compétence et action avancée (niveaux 2, 4, 6, 8, 10 ;
  `LevelReward::SkillAndAdvancedAction`, spec 035), le joueur **prend n'importe quelle carte de l'offre d'actions avancées**. C'est la
  première option de 6.b ; la seconde (une compétence de l'offre commune contre la carte en dernière place) arrive avec les
  compétences (spec 051), comme la compétence elle-même. La carte est obligatoire (Compendium, *End of Turn* S7) : pas de refus.
- **Où va la carte** (MKUE p. 11, 6.b ; p. 5, *"Gain" Effects* 3) : **sur le dessus du deck**, puis l'offre est **complétée** : les
  cartes restantes descendent, une carte tirée entre en première place (p. 5, 3.a ; Compendium, *Miscellaneous* S7).
- **Moment** (MKUE p. 11, 6 puis 7) : après le changement de niveau, **avant la pioche** de fin de tour. Comme la pioche prend le dessus
  du deck, la carte gagnée arrive en main dès cette pioche si le joueur pioche au moins une carte.
- **Plusieurs niveaux pairs d'un coup** (spec 035, L4) : un choix par niveau pair, l'offre complétée entre deux choix.
- **Dernier tour de la partie** (spec 063) : le choix a lieu **avant** la fin de la partie ; la carte compte au décompte (Le Savant).
- **Annonce de fin de manche** : pas de fin de tour, donc pas de changement de niveau ni de carte (spec 023 ; Compendium,
  *End of Round* S1) ; inchangé.
- **Gagner une carte d'une offre, en général** (MKUE p. 5, 3 et 3.a) : la règle « sur le deck, puis compléter l'offre » vaut pour
  toute carte gagnée d'une offre d'actions avancées ou de sorts. Cette spec n'a qu'une occasion de gagner une carte, le niveau pair ;
  les récompenses et achats (sorts, unités, artefacts) arrivent avec leur site (041 à 045) et réutilisent ce mécanisme. Une unité
  prise dans l'offre d'unités ne la complète pas (p. 5, 3.b) : spec 042.

### Les cartes en données

- **Actions avancées** (28), **sorts** (20), **artefacts** (16), **unités** (11 régulières en 20 exemplaires, 9 d'élite en 20
  exemplaires) : texte dans `docs/regles/cartes/`, version *Lost Legion* quand il y a un errata (`CLAUDE.md`).
- **Effets codés dès maintenant** : ceux qu'un `Effect` existant décrit exactement (spec 029), le reste est `Unsupported`. Pour les
  actions avancées : les quatre Éclairs (Fire, Ice, Swift, Crushing Bolt, effets de base et renforcés) et l'effet de base de Bouclier
  de glace (*Ice Shield*, Parade de glace 3). Une option composée (« Mouvement 2 **et** Soin 1 ») ou conditionnelle n'est pas un
  effet existant : `Unsupported`. Effets restants : spec 052 et 053.
- **Une carte `Unsupported` se joue de côté** (MKUE p. 4, 3.e), comme Menace renforcée aujourd'hui. Une carte entièrement
  `Unsupported` ne se joue que de côté.
- **Sorts** (MKUE p. 4, 3.b) : en données avec leur couleur et leurs deux noms ; leurs effets restent `Unsupported` jusqu'à la spec 054,
  qui apporte le paiement d'un sort (mana pour l'effet de base, mana noir en plus la nuit). Aucun sort ne peut être gagné avant la
  spec 043 (Q5).
- **Artefacts** (MKUE p. 4, 3.c) : en données, effets `Unsupported` jusqu'à la spec 055 (jeter un artefact pour l'effet renforcé).
- **Unités** (MKUE p. 5, *Using Units* 1) : en données avec leur pioche (régulière ou d'élite), leur nombre d'exemplaires, coût en
  Influence, niveau, armure, résistances et lieux de recrutement (village, monastère, donjon, tour de mage, cité). Leurs capacités
  ne sont qu'un texte affiché, jusqu'à la spec 042 qui les code.
- **Texte d'une carte non codée** (Q1) : pour que le joueur choisisse dans une offre en connaissance de cause, chaque effet
  `Unsupported` et chaque capacité d'unité a un texte court **écrit par nous** dans `assets/i18n/` (pas le texte officiel, `CLAUDE.md`,
  *Assets et droits*). Un effet codé garde son texte calculé (spec 013).

### Scénarios

- **Première reconnaissance** (MKUE p. 18, *Special Rules* ; Compendium, *Miscellaneous* S8, d'après la prise en main) (Q3) :
  - **pas d'unités d'élite** : la pioche d'élite n'est pas formée, l'offre d'unités ne tire que des régulières ;
  - **actions avancées 1 à 16 seulement** (de Fire Bolt à Regeneration, numéros n/28) ;
  - **au moins une unité de village dans l'offre à la mise en place** : sinon, l'offre retourne dans la pioche, qui est remélangée,
    et l'on tire à nouveau, jusqu'à ce que ce soit vrai (seul l'ordre final est dans l'événement) ; seulement à la mise en place ;
  - « ne révéler sorts et actions avancées qu'au besoin » : ignoré, l'écran montre les offres dès le début.
- **Bac à sable** (Q8) : unités d'élite et les 28 actions avancées, sans règle de village : il sert aux tests et aux parties
  préparées, qui couvrent ainsi les élites.
- Paramètres de `Scenario` (spec 012) : unités d'élite oui/non, actions avancées toutes ou 1 à 16, unité de village à la mise en
  place oui/non.

### Décompte

- **Le Savant** (MKUE p. 15) : 2 par sort, 1 par action avancée, comptés comme les Blessures de La Victime (spec 063) : deck, main,
  défausse, zone de jeu. La ligne de l'écran de fin n'est plus à 0 dès qu'une action avancée est gagnée.

### Schéma : fin d'un tour avec un niveau pair

```
EndTurn (défausses cochées)                       point de non-retour (ferme le tour)
  TurnEnded, bénéfice de la case
  LevelGained { level: 2 }                         niveau pair, offre non vide
  CardDiscarded…
  ── attente : « Niveau 2 : choisissez une action avancée »  (fenêtre Offres, cartes de l'offre seulement)
GainAdvancedAction(Ice Bolt)
  OfferCardGained { Ice Bolt }                     sur le dessus du deck
  OfferCardDealt { Swift Bolt }                    complète l'offre : point de non-retour (révèle)
  CardDrawn { Ice Bolt }, CardDrawn…               pioche jusqu'à la limite de main
  TurnStarted                                      (dernier tour : GameEnded à la place de la pioche)
```

## Modèle

Termes du glossaire existants : Action avancée, Sort, Artefact, Unité, Deck, Niveau, Changement de niveau, Haut Fait, Le Savant.
Nouveaux termes, ajoutés au glossaire par cette PR (noms des cartes : Q2, issue privée 105) :

| Livret (EN) | Français | Code |
|---|---|---|
| Offer | Offre | `Offer` (`AdvancedActions`, `Spells`, `Units`) |
| Advanced Action offer / Spell offer / Unit offer | Offre d'actions avancées / de sorts / d'unités (FAQ p2 et p3) | `Offer::AdvancedActions` / `Spells` / `Units` |
| Advanced Action deck, Spells deck, Artifact deck | Pioche d'actions avancées, de sorts, d'artefacts (« pioche » comme la pioche de tuiles : « deck » est réservé au deck du joueur) | `CommonDeck` (`AdvancedActions`, `Spells`, `Artifacts`, `RegularUnits`, `EliteUnits`) |
| Regular Unit / Elite Unit | Unité régulière (FAQ p1) / Unité d'élite | `UnitRank::Regular` / `Elite` |
| Regular / Elite Unit deck | Pioche d'unités régulières / d'élite | `CommonDeck::RegularUnits` / `EliteUnits` |
| top position / lowest position (of an offer) | Première place / dernière place | ordre de `GameState::offer` (première = indice 0) |
| Refresh (an offer) | Renouveler (une offre) | `OfferCardReturned` puis `OfferCardDealt` |
| Replenish (an offer) | Compléter (une offre) | `OfferCardDealt` après `OfferCardGained` |
| Gain (a card) | Gagner (une carte) | `OfferCardGained`, `GainAdvancedAction` |
| Recruiting icons | Lieux de recrutement | `RecruitSite` (`Village`, `Monastery`, `Keep`, `MageTower`, `City`) |
| Core tile revealed | Tuile royaume révélée | déjà : `TileId::Core(n)` sur le plateau de jeu |

Données nouvelles :

- **Contenu** : `content/advanced_actions.ron`, `spells.ron`, `artifacts.ron`, `units.ron`, validés par `GameContent::parse` (identifiant
  manquant ou en double, couleur, numéro n/28 unique pour les actions avancées, 20 exemplaires par pioche d'unités). Identifiants en
  Rust : `AdvancedAction`, `Spell`, `Artifact`, `Unit` (noms anglais des cartes, comme `BasicAction`). `DeedCard` gagne
  `Advanced`, `Spell`, `Artifact` ; une carte d'unité est `UnitCard` (unité et exemplaire, comme `CardId`), jamais une carte Acte.
- **État** : les cinq pioches communes (ordre caché), les trois offres (ordonnées, première place en tête), et le nombre de choix
  d'action avancée en attente du joueur (0 hors fin de tour à niveau pair).
- **Usages du hasard** (ADR 0007) : un `RngUsage` par pioche commune, numéros de flux nouveaux et figés.
- **Scénario** : les trois paramètres ci-dessus, avec une valeur par défaut (`serde(default)`) comme ceux de la spec 063.
- **Décompte** : `FinalScore` compte les actions avancées et les sorts du joueur.

## Actions et événements

| Action | Validation (refusée si…) | Événements produits |
|---|---|---|
| mise en place (`new_game`, 012) | — | en plus, pour chaque pioche commune utilisée : **`CommonDeckShuffled`**, puis **`OfferCardDealt`** pour les 3 actions avancées, les 3 sorts, les 3 unités |
| `AnnounceEndOfRound` (023, nouvelle manche) | inchangée | après `SourceRolled` : **`OfferCardReturned`** pour chaque unité de l'offre, **`OfferCardDealt`** ×3 (unités), puis pour chaque offre de cartes : **`OfferCardReturned`** (dernière place) et **`OfferCardDealt`** ; puis le deck comme avant |
| `EndTurn` (016) | inchangée | comme avant jusqu'aux défausses ; s'il y a au moins un niveau pair parmi les `LevelGained` et une carte dans l'offre d'actions avancées : **s'arrête là** (ni pioche, ni `TurnStarted`, ni `GameEnded`) |
| **`GainAdvancedAction { card }`** | pas de choix d'action avancée en attente, ou `card` absente de l'offre d'actions avancées | **`OfferCardGained`**, **`OfferCardDealt`** (si la pioche n'est pas vide) ; s'il reste un choix et une carte dans l'offre : s'arrête là ; sinon la suite de `EndTurn` : pioche et `TurnStarted`, ou `GameEnded` au dernier tour |

Pendant un choix d'action avancée, les seules actions légales sont les `GainAdvancedAction`, une par carte de l'offre, dans l'ordre
de l'offre (première place d'abord).

| Événement | Effet de `apply` | Révèle une information ? | Point de non-retour ? |
|---|---|---|---|
| `CommonDeckShuffled { deck, order }` | la pioche reçoit cet ordre, le dessus en premier | seul le nombre de cartes se voit | non (mise en place) |
| `OfferCardDealt { offer, card }` | la carte du dessus de la pioche de l'offre (`card`) entre dans l'offre : première place pour les actions avancées et les sorts, à la suite pour les unités (élite, régulière, élite se lit de gauche à droite) | oui, la carte est révélée à tous | oui (révèle) |
| `OfferCardReturned { offer, card }` | la carte quitte l'offre et va sous sa pioche | non (publique) | non |
| `OfferCardGained { player, offer, card }` | la carte quitte l'offre et va sur le dessus du deck du joueur ; un choix d'action avancée en attente de moins | non (publique : elle était visible) | non (`OfferCardDealt` qui suit l'est) |
| `LevelGained` (035) | en plus : un niveau pair ajoute un choix d'action avancée en attente | inchangé | inchangé |

## Comportements attendus

Chaque cas devient au moins un test (nom indicatif entre parenthèses).

### Mise en place et renouvellement

- **C1** — Étant donné une nouvelle partie du bac à sable, alors l'offre d'actions avancées et l'offre de sorts ont 3 cartes, l'offre
  d'unités 3 unités régulières, et les pioches ont 25 actions avancées, 17 sorts, 16 artefacts, 17 unités régulières et 20 d'élite
  (`setup_deals_three_cards_to_each_offer`) — MKUE p. 3
- **C2** — Étant donné deux parties de même graine, alors les offres sont les mêmes ; et une partie a les mêmes tuiles, Source, deck
  et jetons d'ennemis qu'avant cette spec (`offers_do_not_shift_other_draws`) — ADR 0007
- **C3** — Étant donné une nouvelle partie de Première reconnaissance, alors la pioche d'actions avancées et l'offre ne contiennent
  que les cartes 1 à 16, et il n'y a pas de pioche d'unités d'élite (`first_reconnaissance_uses_advanced_actions_1_to_16`) — MKUE
  p. 18 ; Compendium, *Miscellaneous* S8
- **C4** — Étant donné une graine dont la première offre tirée n'a aucune unité de village, quand une partie de Première
  reconnaissance commence, alors l'offre d'unités en contient au moins une ; en bac à sable, avec la même graine, l'offre reste sans
  village (`first_reconnaissance_offers_a_village_unit`) — MKUE p. 18
- **C5** — Étant donné une offre d'actions avancées A, B, C (A en première place), quand une nouvelle manche commence, alors C est sous
  la pioche et l'offre est N, A, B, N étant la carte qui était sur la pioche (`new_round_refreshes_advanced_action_offer`) — MKUE
  p. 4, 2.d
- **C6** — Même cas pour l'offre de sorts (`new_round_refreshes_spell_offer`) — MKUE p. 4, 2.e
- **C7** — Étant donné aucune tuile royaume révélée, quand une nouvelle manche commence, alors les 3 unités de l'offre sont sous la
  pioche d'unités régulières et 3 nouvelles régulières sont tirées (`new_round_deals_regular_units_before_core_tile`) — MKUE p. 4, 2.c
- **C8** — Étant donné une tuile royaume révélée dans le bac à sable, quand une nouvelle manche commence, alors l'offre d'unités est
  élite, régulière, élite, et une unité d'élite de l'ancienne offre est retournée sous la pioche d'élite
  (`new_round_alternates_elite_units_after_core_tile`) — MKUE p. 4, 2.c ; Compendium, *Silver Units* S1
- **C9** — Étant donné une tuile royaume révélée en Première reconnaissance, quand une nouvelle manche commence, alors l'offre
  d'unités n'a que des régulières (`first_reconnaissance_never_deals_elite_units`) — MKUE p. 18

### Action avancée d'un niveau pair

- **C10** — Étant donné 2 gloire et un combat qui en rapporte 1, quand le joueur finit son tour, alors il passe au niveau 2 et la
  seule décision est de choisir une des 3 actions avancées de l'offre ; ni pioche ni nouveau tour avant ce choix
  (`even_level_asks_for_an_advanced_action`) — MKUE p. 11, 6.b
- **C11** — Étant donné ce choix, quand le joueur prend la carte en deuxième place, alors elle quitte l'offre, l'offre est complétée
  en première place par la carte du dessus de la pioche, et la pioche de fin de tour lui met la carte en main
  (`gained_advanced_action_goes_on_deck_and_is_drawn`) — MKUE p. 11, 6.b et 7 ; p. 5, 3.a
- **C12** — Étant donné un niveau impair seulement (2 → 3), quand le joueur finit son tour, alors pas de choix : pioche et nouveau
  tour comme avant (`odd_level_gives_no_advanced_action`) — MKUE p. 11, 6.a
- **C13** — Étant donné deux niveaux pairs franchis d'un coup (niveau 1 → 4), alors deux choix l'un après l'autre, l'offre complétée
  entre les deux, et deux cartes sur le deck, la seconde au-dessus (`two_even_levels_give_two_advanced_actions`) — spec 035, L4
- **C14** — Étant donné le dernier tour de Première reconnaissance et un niveau pair atteint, quand le joueur choisit sa carte,
  alors la partie se termine et Le Savant vaut 1 (`last_turn_advanced_action_counts_at_scoring`) — MKUE p. 15 ; spec 063
- **C15** — Étant donné des actions avancées et un sort parmi les cartes du joueur (deck, main, défausse), alors Le Savant vaut 1 par
  action avancée et 2 par sort (`knowledge_counts_advanced_actions_and_spells`) — MKUE p. 15
- **C16** — Étant donné un choix en attente, alors on ne peut ni revenir avant `EndTurn` ni avant le choix une fois la carte
  révélée pour compléter l'offre (`advanced_action_choice_cannot_be_undone`) — spec 024

### Jouer une carte gagnée

- **C17** — Étant donné Fire Bolt en main, quand le joueur la joue pour son effet de base, alors il gagne un cristal rouge ;
  renforcée en combat, en phase d'attaque à distance, elle donne une attaque à distance de feu 3 (`fire_bolt_plays_its_effects`) —
  `cartes/actions-avancees.md`
- **C18** — Étant donné Bouclier de glace en main en phase de parade, alors son effet de base (Parade de glace 3) se joue, son effet
  renforcé n'est pas proposé, et elle se joue de côté (`ice_shield_strong_effect_is_not_yet_supported`) — MKUE p. 4, 3.e
- **C19** — Étant donné Agilité en main, alors seules les actions de côté sont proposées (`unsupported_card_plays_sideways_only`) —
  MKUE p. 4, 3.e

### Contenu, vue et sauvegarde

- **C20** — Le contenu embarqué a 28 actions avancées (couleurs et numéros uniques), 20 sorts, 16 artefacts, 20 unités par pioche ;
  un fichier de test avec un identifiant en double ou manquant est refusé (`content_has_all_base_game_cards`) — spec 029
- **C21** — La vue du joueur contient les trois offres et le nombre de cartes de chaque pioche, jamais leur ordre ; un
  `CommonDeckShuffled` filtré ne donne que le nombre (`player_view_shows_offers_not_deck_order`)
- **C22** — Étant donné une partie sauvegardée après un renouvellement et une action avancée gagnée, quand elle est rechargée, alors
  offres, pioches et deck sont identiques (`offers_survive_save_and_reload`) — spec 024

## Cas limites et refus

- **L1** — Pioche d'actions avancées vide quand il faut compléter ou renouveler l'offre : l'offre garde moins de cartes, sans erreur.
  Offre vide à un niveau pair : le niveau est gagné, pas de choix, la fin du tour continue.
- **L2** — Pioche d'élite vide au moment d'en tirer une : on tire une régulière à la place ; pioche régulière vide aussi : l'offre a
  moins de 3 unités.
- **L3** — `GainAdvancedAction` hors choix en attente, ou avec une carte absente de l'offre : refusée (`ActionError::NotLegal`).
- **L4** — Pendant le choix, aucune autre action (pas d'annulation de la fin du tour : elle est déjà un point de non-retour).
- **L5** — Main déjà à la limite après la fin du tour (rien à piocher) : la carte gagnée reste sur le deck, piochée au tour suivant.
- **L6** — Sauvegarde d'avant cette spec : rechargée sans offres ni pioches communes (leurs événements manquent) ; un niveau pair n'y
  donne alors rien (L1). Pas de migration avant la fin du J6 (spec 024).
- **L7** — Annonce de la fin de manche avec un niveau en attente : rien ne change (pas de fin de tour, Compendium, *End of Round* S1).

## Information cachée

- Visible : les trois offres et leur ordre, le nombre de cartes de chaque pioche, les cartes gagnées (elles viennent d'une offre
  visible), le choix en attente.
- Caché : l'ordre des pioches (filtré dans `CommonDeckShuffled`, comme `TileDeckFormed`). Une carte gagnée sur le dessus du deck est
  connue du joueur ; un autre joueur la verra aussi, puisqu'elle était dans l'offre (multijoueur : hors périmètre).

## Libellés (i18n)

- Noms : `card.<id>.name` pour chaque action avancée et artefact ; pour un sort, `card.<id>.name` (effet de base) et
  `card.<id>.strong_name` (effet renforcé) ; `unit.<id>.name`. Noms français : Q2, issue privée 105.
- Textes écrits par nous (Q1) : `card.<id>.basic_text` / `card.<id>.strong_text` pour chaque effet `Unsupported` ; `unit.<id>.abilities`
  (une ligne par capacité).
- Écran : `offers.title` (« Offres »), `offers.advanced_actions`, `offers.spells`, `offers.units`, `offers.deck_count` (`{count}` :
  « Pioche : {count} »), `offers.take` (« Prendre »), `offers.choose_advanced_action` (`{level}` : « Niveau {level} : choisissez une
  action avancée »), `offers.button` (« Offres… »), `unit.cost` (`{influence}`), `unit.level` (`{level}`), `unit.armor` (`{armor}`),
  `unit.recruit.<lieu>`, `unit.elite`, `unit.regular`.

## Interface

- **Fenêtre Offres** : ouverte par le bouton « Offres… » du panneau latéral ou la touche O, fermée par Échap ou sa croix. Trois
  colonnes (cartes de la main en rangées depuis la spec [#32](issue-32-cartes-des-fenetres.md)) : actions avancées et sorts dans l'ordre de l'offre (première place en haut), chacun avec nom, couleur, et ses deux
  effets (texte calculé, ou texte écrit par nous suivi de « pas encore codé : se joue de côté ») ; unités avec nom, régulière ou
  élite, niveau, coût, armure, résistances, lieux de recrutement, capacités (texte). Sous chaque colonne : « Pioche : N ».
  Consultation seule : on ne prend rien ici en dehors du choix ci-dessous (recruter : spec 042, acheter : 041 à 043).
- **Choix d'une action avancée** : à la fin d'un tour à niveau pair, la fenêtre s'ouvre seule, titrée « Niveau 2 : choisissez une
  action avancée », avec un bouton « Prendre » sous chaque action avancée ; elle ne se ferme pas tant que le choix est en attente
  (« Voir le plateau » comme en combat n'est pas nécessaire : rien d'autre n'est jouable). Le bandeau du niveau gagné (spec 035)
  reste au tour suivant.
- **Main** : une action avancée a la couleur de sa carte ; un effet non codé affiche son texte écrit par nous et n'est pas cliquable,
  comme Menace renforcée aujourd'hui.
- **Partie préparée** (arcanum-persistance, `docs/verification-manuelle.md`) : `niveau-pair`, bac à sable avec 2 gloire et un ennemi à
  1 de gloire à portée, pour vérifier à l'œil le choix, la pioche de la carte, puis la fenêtre Offres après une nouvelle manche.

### Raccourcis clavier

| Touche | Effet | Contexte |
|---|---|---|
| O | Ouvrir ou fermer la fenêtre Offres | Partie, hors mode choix et hors choix d'action avancée |
| Échap | Fermer la fenêtre Offres | Fenêtre Offres ouverte, hors choix d'action avancée |
| 1, 2, 3 | Prendre l'action avancée en 1re, 2e, 3e place | Choix d'action avancée en attente |

À reporter dans `docs/raccourcis.md` à l'implémentation.

## Hors périmètre

- Compétences et seconde option du niveau pair (offre commune, carte en dernière place) : spec 051.
- Actions avancées dans l'offre d'unités (monastères), achat d'une action avancée : spec 041 (Q4).
- Recruter une unité, unités en jeu, unité prise dans l'offre sans la compléter : spec 042.
- Gagner un sort, un artefact, une unité en récompense : specs 041, 043, 044, 045. Acheter un sort : 043, 062.
- Effets des actions avancées non codées : 052, 053 ; sorts : 054 ; artefacts et bannières : 055.
- Joueur fantôme qui prend la dernière action avancée et gagne un cristal au renouvellement des sorts : spec 061.
- Extensions (actions avancées bicolores, unités de la Légion oubliée).

## Questions ouvertes

Aucune.

## Décisions (2026-10-31)

Mathieu a retenu toutes les propositions (« OK pour tout ») :

- **Q1 — Texte d'une carte non codée** : un texte court écrit par nous (fr et en) pour chaque effet `Unsupported` et chaque capacité
  d'unité, remplacé par le texte calculé quand la spec qui le code arrive.
- **Q2 — Noms français des cartes** : l'implémentation propose les noms dans le glossaire, marqués « proposition » ; Mathieu les corrige
  d'après sa boîte. Suivi : issue privée 105.
- **Q3 — Première reconnaissance** : pas d'élites, actions avancées 1 à 16, au moins une unité de village (à la mise en place
  seulement) ; « révéler au besoin » ignoré.
- **Q4 — Actions avancées des monastères** : spec 041.
- **Q5 — Sorts et artefacts** : en données dès cette spec, effets `Unsupported` jusqu'aux specs 054 et 055.
- **Q6 — Choix de l'action avancée** : décision à part après `EndTurn`, avant la pioche.
- **Q7 — Interface** : fenêtre Offres (bouton « Offres… » et touche O).
- **Q8 — Bac à sable** : unités d'élite et 28 actions avancées, sans règle de village.
