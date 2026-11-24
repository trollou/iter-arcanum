# 023 — Fin de manche

- Statut : implémentée (2026-10-25)
- Jalon : J2
- Dépend de : 016 (fin du tour), 020 (Source), 022 (dés pris), 012 (scénario)
- Livret : MKUE p. 3 (plateau Jour/Nuit : Jour au début, retourné à chaque manche), p. 4 (« One Round of the Game » 2 : préparer la
  manche ; 4.b : annonce ; 5 : fin de partie), p. 6 (3 : annoncer la fin de la manche ; 4 : passer son tour), p. 11 (« Wounds » 1.c)
- Crates touchées : arcanum-core, arcanum-ui, `assets/i18n/` (fr.json et en.json)

## Objectif

La partie ne s'arrête plus quand le deck est épuisé. Au début d'un tour, deck vide, le joueur **annonce la fin de la manche** (il le
peut s'il a encore des cartes en main, il le doit sinon). Une **nouvelle manche** commence : Jour et Nuit **alternent**, la Source est
**relancée**, toutes les cartes du joueur sont **remélangées** en un nouveau deck et il pioche une main neuve. Le panneau affiche
« Manche 2/4 · Nuit (tour 1) », avec une icône lune. Le scénario fixe un **nombre de manches** (4 pour le bac à sable) : la dernière
finie, la partie est terminée.

## Règles

- **Quand annoncer** (MKUE p. 6, 3 à 3.c ; p. 4, 4.b) : au début de son tour, deck vide, si personne n'a encore annoncé, le joueur peut
  **passer son tour** pour annoncer la fin de la manche. Il le **doit** si sa main est vide aussi ; sinon, il choisit entre annoncer et
  jouer son tour.
- **Passer son tour** (MKUE p. 6, 4) : le tour se termine aussitôt, sans bénéfice de la case (mine, clairière magique). Ni carte jouée ni
  défaussée : « jouer ou défausser au moins une carte » (p. 6, 5.a) ne vaut que si l'on joue son tour.
- **Fin de la manche** (MKUE p. 6, 3) : en solo, sans joueur fantôme (spec 061), il n'y a pas d'autre joueur pour un dernier tour : la
  manche finit tout de suite.
- **Fin de partie** (MKUE p. 4, 5.a et 5.b) : la manche finie, partie terminée si le nombre de manches du scénario est atteint, sinon
  manche suivante. Les conditions de fin propres au scénario (p. 4, 4.c) et le décompte ne sont pas dans cette spec.
- **Préparer la manche** (MKUE p. 4, 2), dans l'ordre du livret, pour ce qui existe déjà :
  - a. **retourner le plateau Jour/Nuit** (aussi p. 3) ;
  - b. **réinitialiser la Source** : relancer **tous** les dés, avec la règle de la mise en place (spec 020) ; les dés épuisés dépendent
    du nouveau moment de la journée (p. 5, 3.c) ;
  - g. **remélanger** **toutes** ses cartes (deck, défausse, main) en un nouveau deck, puis piocher jusqu'à sa limite de main ; Blessures
    en main comprises (p. 11, « Wounds » 1.c ; pas encore de Blessure, spec 034) ;
  - c à f (offres d'unités, d'actions avancées et de sorts, tactiques) et le reste de g (bannières, compétences, unités) : hors
    périmètre.
- **Ce qui ne change pas.** L'inventaire de cristaux reste (aucune règle ne le vide), la figurine ne bouge pas, les tuiles restent.
- **Première manche** (MKUE p. 4, 2 ; p. 3) : la mise en place a déjà tout préparé ; la manche 1 est un Jour et commence avec
  `GameStarted`, comme aujourd'hui.
- **La nuit change déjà le jeu.** Coûts de déplacement de nuit (spec 014, forêt 5, désert 3), dés or épuisés et noirs utilisables
  (spec 020), or qui ne paie plus rien et noir qui ne paie pas une carte d'action (spec 022) : tout cela est en place, testé sur état
  construit ; cette spec le rend jouable.

### Schéma : une manche en solo

```
GameStarted ── Manche 1/4 · Jour (tour 1) ── tour 2 ── … ── tour n (deck vide au début du tour)
                                                    │
                       main vide ? ── oui ──▶ doit annoncer
                                   └─ non ──▶ peut annoncer, ou jouer son tour ── EndTurn ──▶ Tour n+1 …
                                                    │
                                     AnnounceEndOfRound
                                                    │
                                              RoundEnded { round: 1 }
                                                    │
                          nombre de manches atteint ? ── oui ──▶ partie terminée (plus de décision)
                                                    │ non
                                              RoundStarted { round: 2, Nuit }     a. plateau Jour/Nuit retourné
                                              SourceRolled { dés }                b. Source réinitialisée
                                              DeckShuffled { 16 cartes }          g. deck + défausse + main remélangés
                                              CardDrawn × 5                       g. main neuve
                                              TurnStarted { turn: 1 }
                                                    │
                                              Manche 2/4 · Nuit (tour 1) …
```

### Exemple : la fin de la manche de l'exemple de la spec 016

Même joueur que l'exemple de la spec 016 (16 cartes, sans Blessure), dans le bac à sable :

| Moment | Ce qui se passe | Main / deck / défausse | Panneau |
|---|---|---|---|
| Fin du tour 5 | Il joue sa dernière carte et finit son tour ; deck vide, rien n'est pioché. | 0 / 0 / 16 | Manche 1/4 · Jour (tour 5) |
| Début du tour 6 | Deck et main vides : seule action, « Annoncer la fin de la manche ». | 0 / 0 / 16 | Manche 1/4 · Jour (tour 6) |
| Annonce | La manche 1 finit. Manche 2 : Nuit ; la Source est relancée (le dé or est maintenant épuisé, un dé noir ne l'est plus) ; les 16 cartes sont mélangées ; il en pioche 5. | 5 / 11 / 0 | Manche 2/4 · Nuit (tour 1) |
| Tour 1 de nuit | Une forêt coûte maintenant 5 points de mouvement, un désert 3. | … | Manche 2/4 · Nuit (tour 1) |

Variante : au début du tour 5, il lui reste 2 cartes en main et son deck est vide. Il peut jouer son tour (puis annoncer au tour 6, ou
plus tard), ou annoncer tout de suite : ses 2 cartes repartent alors dans le nouveau deck de 16 cartes, sans être jouées.

## Modèle

### Termes

Termes du glossaire déjà présents : Manche (`Round`), Tour (`Turn`), Jour / Nuit (`TimeOfDay`), Plateau Jour/Nuit, Réinitialiser la
Source, Passer son tour, Deck (`DeedDeck`), Défausse, Main, Limite de main, Numéro du tour (`turn`), Scénario.

Nouveaux termes, ajoutés au glossaire dans la même PR (à valider, Q5) :

| Livret (EN) | Français | Code |
|---|---|---|
| End of the Round | Fin de manche | `RoundEnded` (événement) |
| Announce the End of the Round | Annoncer la fin de la manche | `AnnounceEndOfRound` (action) |
| Prepare the Round | Préparer la manche | `RoundStarted` (événement) et ceux qui le suivent |
| (Round number) | Numéro de la manche | `round` |
| Round limit | Nombre de manches (du scénario) | `round_limit` |

« Réinitialiser la Source » et « Passer son tour » perdent leur mention « (spec 023) » : ils s'écrivent `SourceRolled` et
`AnnounceEndOfRound` (passer son tour n'existe pas encore pour une autre raison, MKUE p. 6, 3.d, multijoueur).

### Nouvelles données dans l'état

- le **numéro de la manche**, public : **1** dès `GameStarted`, augmenté de 1 à chaque nouvelle manche ;
- un indicateur **manche finie** : vrai entre `RoundEnded` et le `RoundStarted` suivant. Il empêche toute décision après la dernière
  manche (sans lui, un joueur qui annonce avec des cartes en main pourrait encore jouer) ;
- dans le **scénario**, le **nombre de manches** : **4** pour le bac à sable, comme « Première reconnaissance » en solo, 2 jours et 2
  nuits (MKUE p. 18 ; Q1). Un nombre nul est refusé à la création du scénario (`Scenario::new`).

Le moment de la journée existe déjà (spec 014) ; il change maintenant. Le numéro du tour (spec 016) repart à 1 à chaque manche.

## Actions et événements

| Action | Validation (refusée si…) | Événements produits |
|---|---|---|
| `AnnounceEndOfRound` | le deck n'est pas vide ; le tour est commencé (zone de jeu non vide) ; la manche est finie | `RoundEnded`, puis, si le nombre de manches n'est pas atteint : `RoundStarted`, `SourceRolled`, `DeckShuffled`, un `CardDrawn` par carte piochée, `TurnStarted { turn: 1 }` |

**Tour commencé.** On ne peut annoncer qu'« au début du tour » (MKUE p. 6, 3). Aujourd'hui, tout ce qu'on fait pendant un tour commence
par jouer une carte : se déplacer et explorer demandent des points, prendre un dé ou utiliser un cristal se fait en jouant une carte
(spec 022), défausser pour Improvisation aussi. « Zone de jeu non vide » suffit donc à dire que le tour est commencé. Une spec qui
ajoutera une action sans carte (compétences, soins, spec 034 ou 051) devra revoir ce critère.

| Événement | Effet de `apply` | Révèle une information ? |
|---|---|---|
| `RoundEnded { round }` | la manche est finie : plus de décision | oui, publique |
| `RoundStarted { round, time_of_day }` | met le numéro de la manche à `round` et le moment de la journée à `time_of_day` ; la manche n'est plus finie | oui, publique |
| `SourceRolled { dice }` (020) | inchangé : remplace les dés de la Source ; **en plus**, plus aucun dé n'est pris (spec 022) | oui, publique |
| `DeckShuffled { player, order }` (000) | le deck devient `order` ; **en plus**, la main et la défausse sont vidées (leurs cartes sont dans `order`). À la mise en place, elles sont déjà vides : rien ne change pour la spec 012 | le nombre de cartes seulement (inchangé) |
| `CardDrawn`, `TurnStarted` | inchangés (000, 016) | inchangé |

**Ordre du hasard.** Comme le livret : la Source d'abord (p. 4, 2.b), puis le deck (2.g). Le deck est mélangé à partir de la suite deck,
défausse, main (chacune dans son ordre), pour que la même graine donne le même ordre.

**Moment de la journée.** `decide` calcule l'inverse du moment courant (MKUE p. 3) ; `RoundStarted` le porte, comme `CardPlayed` porte
son effet : `apply` ne recalcule rien. Le moment de la journée n'est pas déduit de la parité de la manche, pour laisser la place à la
variante « Daytime Variability » (p. 16, hors périmètre).

**Décision en attente.** Le joueur choisit tant qu'il a une action légale et que la manche n'est pas finie. Deck et main vides au début
du tour : la seule action légale est `AnnounceEndOfRound`.

**Actions légales**, dans cet ordre : celles des specs 013 à 016 et 022, inchangées, **puis** `AnnounceEndOfRound` si elle est légale.
En dernier, pour que le joueur « premier choix » de la boucle de partie joue ses cartes avant d'annoncer.

## Comportements attendus

Chaque cas devient au moins un test (nom indicatif entre parenthèses). Sauf mention, partie de bac à sable avec Tovak ; « deck épuisé »
veut dire qu'on a fini les tours comme dans l'exemple de la spec 016.

Annonce (arcanum-core) :

- **C1** — Deck et main vides au début du tour : la seule action légale est `AnnounceEndOfRound`
  (`empty_deck_and_hand_force_end_of_round`) — MKUE p. 6, 3.b.
- **C2** — Deck vide, 2 cartes en main, début du tour : les actions légales sont celles des cartes et des fins de tour, puis
  `AnnounceEndOfRound` en dernier ; jouer une carte reste possible (`empty_deck_lets_player_choose_to_announce`) — MKUE p. 6, 3.c.
- **C3** — Deck vide, une carte jouée ce tour : `AnnounceEndOfRound` n'est plus légale ; `decide` la refuse
  (`announcing_needs_the_start_of_the_turn`) — MKUE p. 6, 3.
- **C4** — Deck non vide : `AnnounceEndOfRound` n'est pas légale ; `decide` la refuse (`announcing_needs_an_empty_deck`) — MKUE p. 6, 3.a.
- **C5** — Annonce en manche 1 du bac à sable : les événements sont, dans l'ordre, `RoundEnded { round: 1 }`,
  `RoundStarted { round: 2, time_of_day: Nuit }`, `SourceRolled`, `DeckShuffled`, 5 `CardDrawn`, `TurnStarted { turn: 1 }`
  (`end_of_round_events_come_in_rule_order`) — MKUE p. 4, 2.

Nouvelle manche (arcanum-core) :

- **C6** — Après l'annonce : manche 2, Nuit, tour 1 ; la vue joueur montre les trois (`new_round_flips_day_to_night`) — MKUE p. 3 ; p. 4, 2.a.
- **C7** — Annonce en manche 2 (nuit) : la manche 3 est un Jour (`night_is_followed_by_day`) — MKUE p. 4, 2.a.
- **C8** — Après l'annonce, deck épuisé et main vide : le deck a 11 cartes, la main 5, la défausse 0 ; les 16 cartes de départ sont
  toutes là, une fois chacune (`new_round_reshuffles_every_card`) — MKUE p. 4, 2.g.
- **C9** — Annonce avec 2 cartes en main : ces 2 cartes sont dans le nouveau deck ou la nouvelle main, plus dans l'ancienne
  (`cards_in_hand_are_reshuffled_too`) — MKUE p. 4, 2.g.
- **C10** — La Source est relancée en entier, avec la règle de la moitié ; la même graine donne la même Source et le même deck
  (`new_round_resets_the_source`) — MKUE p. 4, 2.b ; p. 3.
- **C11** — Au tour 1 de la nuit, un dé or de la Source est épuisé et une forêt voisine coûte 5 (état construit pour avoir l'un et
  l'autre) (`night_rules_apply_in_the_new_round`) — MKUE p. 5, 3.c ; p. 7.
- **C12** — L'inventaire, la figurine et le plateau de jeu sont les mêmes avant et après l'annonce
  (`new_round_keeps_crystals_figure_and_map`).
- **C13** — Annonce en finissant sur une mine : aucun `CrystalGained` (`announcing_forfeits_the_mine`) — MKUE p. 6, 4.

Fin de partie (arcanum-core) :

- **C14** — Scénario de test de la spec 012 avec 2 manches : l'annonce en manche 2 ne produit que `RoundEnded { round: 2 }` ; ensuite,
  plus aucune décision, même avec des cartes en main (`last_round_ends_the_game`) — MKUE p. 4, 5.a.
- **C15** — Bac à sable : 4 manches, Jour, Nuit, Jour, Nuit ; après la 4e annonce, la partie est terminée (`sandbox_lasts_four_rounds`)
  — MKUE p. 18.
- **C16** — Créer un scénario avec 0 manche est refusé (`scenario_needs_at_least_one_round`).

Journal et boucle (arcanum-core) :

- **C17** — Dans le journal filtré pour un autre joueur, `RoundEnded`, `RoundStarted` et `SourceRolled` sont montrés en entier,
  `DeckShuffled` ne montre que le nombre de cartes, les `CardDrawn` cachent la carte (`end_of_round_hides_only_cards`).
- **C18** — Rejouer le journal après deux fins de manche redonne le même état (`replaying_log_rebuilds_rounds`).
- **C19** — Le joueur « premier choix » joue jusqu'au début de la manche 3 : chaque annonce n'arrive que deck et main vides
  (`runner_plays_several_rounds`).

Affichage (arcanum-ui) :

- **C20** — Le panneau affiche « Manche 1/4 · Jour (tour 1) » avec l'icône soleil, puis « Manche 2/4 · Nuit (tour 1) » avec l'icône
  lune après l'annonce (`panel_shows_round_number_and_time_icon`).
- **C21** — Deck vide, cartes en main, début du tour : un bandeau « Votre deck est vide : vous pouvez annoncer la fin de la manche au
  lieu de jouer ce tour. » avec le bouton « Annoncer la fin de la manche » ; les cartes restent jouables
  (`announce_banner_when_deck_is_empty`).
- **C22** — Deck et main vides : le bandeau dit « Votre deck et votre main sont vides : annoncez la fin de la manche. » ; il n'y a pas de
  bouton « Finir le tour… » (`announce_banner_when_it_is_mandatory`).
- **C23** — Cartes en main : le bouton n'envoie rien encore ; le bandeau devient « Annoncer la fin de la manche ? Vos 2 cartes en main
  seront remélangées sans être jouées. » avec « Confirmer » et « Annuler » ; « Confirmer » envoie `AnnounceEndOfRound`, « Annuler » ou
  Échap revient au bandeau (`announcing_with_cards_asks_confirmation`) (Q2).
- **C23 bis** — Deck et main vides : le bouton envoie `AnnounceEndOfRound` tout de suite, puisque c'est la seule action possible ; le
  bouton disparaît dès qu'une carte est jouée (`mandatory_announce_needs_no_confirmation`).
- **C24** — Au tour 1 d'une manche autre que la première, tant qu'aucune carte n'est jouée, une ligne « Nouvelle manche : la Source a
  été relancée et votre deck remélangé. » s'affiche sous la ligne du tour (`panel_announces_new_round`).
- **C25** — De nuit, le plateau de jeu est couvert d'un voile sombre ; de jour, non (`board_is_veiled_at_night`, sur la fonction qui
  donne la couleur du voile selon le moment de la journée) (Q3).
- **C26** — Partie terminée par le nombre de manches : « Partie terminée : la dernière manche est jouée. »
  (`screen_announces_last_round`).
- **C27** — Avec le vrai `fr.json`, calculer l'écran en manche 1 et 2, bandeau facultatif, obligatoire et de confirmation, ligne de
  nouvelle manche et fin de partie ne relève aucun manque (`every_round_key_exists_in_french`).

Vérification manuelle : `cargo run -p arcanum-ui` sous Linux, jouer tout le deck : au début du tour 6, le bandeau oblige à annoncer ; après,
« Manche 2/4 · Nuit (tour 1) », icône lune, plateau voilé, Source relancée, main neuve de 5 cartes.

## Cas limites et refus

- **L1** — `AnnounceEndOfRound` deck non vide, tour commencé, ou manche finie : `ActionError::NotLegal`, aucun événement (C3, C4, C14).
- **L2** — Deck vide en fin de tour, main non vide : rien ne change par rapport à la spec 016 (L4) ; l'annonce n'est possible qu'au
  début du tour suivant (MKUE p. 6, 3.a).
- **L3** — Un dé pris ne peut pas rester pris à l'annonce : on n'a rien joué. `SourceRolled` libère quand même tous les dés pris, pour
  que `apply` reste juste sur un état construit (`source_reset_frees_taken_dice`).
- **L4** — Main au-dessus de la limite de main au moment de l'annonce (impossible aujourd'hui, spec 016 C7) : toutes les cartes sont
  remélangées, la main neuve a 5 cartes.
- **L5** — Deck de moins de 5 cartes au total (impossible avec 16 cartes de départ ; possible avec des cartes jetées, J5) : on pioche
  tout le deck et on s'arrête (MKUE p. 11, 7.d). Pas de test.
- **L6** — Effets qui augmentent la limite de main « la prochaine fois qu'on pioche », reportés à la manche suivante (MKUE p. 4, 4.f) :
  aucun n'existe encore (tactique « Planning », spec 060).

## Information cachée

- Cachés : l'ordre du nouveau deck (`DeckShuffled` filtré, inchangé), les cartes piochées pour les autres joueurs.
- Publics : l'annonce, la fin de la manche, le numéro de la manche, le moment de la journée, la nouvelle Source, la taille du deck, de
  la main et de la défausse, le nombre de manches du scénario.

## Libellés (i18n)

Ajoutés ou modifiés dans `fr.json` et `en.json`.

| Clé | Paramètres | Français | English |
|---|---|---|---|
| `turn.line` (modifiée) | `round`, `limit`, `time_of_day`, `number` | Manche {round}/{limit} · {time_of_day} (tour {number}) | Round {round}/{limit} · {time_of_day} (turn {number}) |
| `round.announce.button` | — | Annoncer la fin de la manche | Announce the end of the round |
| `round.announce.optional` | — | Votre deck est vide : vous pouvez annoncer la fin de la manche au lieu de jouer ce tour. | Your deck is empty: you may announce the end of the round instead of playing this turn. |
| `round.announce.mandatory` | — | Votre deck et votre main sont vides : annoncez la fin de la manche. | Your deck and hand are empty: announce the end of the round. |
| `round.announce.confirm_prompt` | `count` | Annoncer la fin de la manche ? Vos {count} carte(s) en main seront remélangées sans être jouées. | Announce the end of the round? Your {count} card(s) in hand will be reshuffled without being played. |
| `round.announce.confirm` | — | Confirmer | Confirm |
| `round.announce.cancel` | — | Annuler | Cancel |
| `time_of_day.icon.day` | — | Jour | Day |
| `time_of_day.icon.night` | — | Nuit | Night |
| `round.new` | — | Nouvelle manche : la Source a été relancée et votre deck remélangé. | New round: the Source was rerolled and your deck reshuffled. |
| `game.last_round` | — | Partie terminée : la dernière manche est jouée. | Game over: the last round has been played. |

`game.finished` reste pour une partie qui n'a plus de décision pour une autre raison.

## Interface

- **Panneau latéral** : la ligne du tour devient « Manche 2/4 · Nuit (tour 1) » (affichage choisi par Mathieu, qui remplace celui
  proposé à la spec 016, Q5), précédée d'une **icône soleil ou lune** de la hauteur du texte (SVG créés par nous,
  `assets/images/time_of_day/day.svg` et `night.svg`, info-bulle « Jour » ou « Nuit » ; Q3). Au tour 1 d'une nouvelle manche, tant
  qu'aucune carte n'est jouée, une ligne en dessous rappelle ce qui vient de se passer (C24).
- **Bandeau d'annonce** : au-dessus de la main, quand `AnnounceEndOfRound` est légale, même place et même style que le bandeau du mode fin
  du tour (spec 016), mais ce n'est pas un mode : les cartes restent jouables. Il dit si l'annonce est facultative ou obligatoire et porte
  le bouton « Annoncer la fin de la manche ». S'il reste des cartes en main, le bouton demande d'abord une **confirmation** dans le
  bandeau (C23, Q2) ; si l'annonce est obligatoire, il n'y a rien à confirmer.
- **Plateau de jeu, nuit** : voile bleu nuit semi-transparent sur tout le plateau de jeu, sous la figurine, les mises en évidence et le
  trajet (spec 017), pour qu'ils restent lisibles (Q3, à essayer : on jugera sur les captures de l'implémentation).
- **Fin de partie** : message « Partie terminée : la dernière manche est jouée. », plus de bouton.

```
┌─ Panneau latéral ─────────────────────────────────┐
│ Bac à sable                                       │
│ ☀ Manche 1/4 · Jour (tour 6)                      │
│ Mouvement : 0 · Influence : 0                     │
│ ┌───────────────────────────────────────────────┐ │
│ │ Votre deck et votre main sont vides :         │ │
│ │ annoncez la fin de la manche.                 │ │
│ │              [Annoncer la fin de la manche]   │ │
│ └───────────────────────────────────────────────┘ │
│ Main                                              │
│ Zone de jeu                                       │
│ Deck : 0 cartes · Défausse : 16 cartes            │
└───────────────────────────────────────────────────┘

                ▼ clic

┌─ Panneau latéral ─────────────────────────────────┐
│ Bac à sable                                       │
│ ☾ Manche 2/4 · Nuit (tour 1)                      │
│ Nouvelle manche : la Source a été relancée et     │
│ votre deck remélangé.                             │
│ Source  ● ● ●   épuisé : ● (or)                   │
│ Main                                              │
│ ● Marche          Mouvement 2      [Jouer] [Côté…]│
│ …                                                 │
│ Deck : 11 cartes · Défausse : 0 cartes            │
└───────────────────────────────────────────────────┘
```

### Raccourcis clavier

Aucun (Q4), sauf Échap qui annule la confirmation, comme les autres modes. L'annonce arrive une fois par manche et ne s'annule pas (le mélange est une révélation, spec 024) : une touche ferait
plus de mal par erreur que de bien. Fin reste « ouvrir le mode fin du tour » (016) ; quand l'annonce est obligatoire, Fin ne fait rien,
puisque finir le tour n'est pas possible.

## Effets sur les specs déjà implémentées

Signalés ici parce que le sens de tests existants change (règle « jamais modifier un test sans le signaler ») :

- **016** : « deck et main vides, plus de décision » (Q4 de la 016) devient « seule action : `AnnounceEndOfRound` ». Tests touchés :
  C11 `whole_deck_is_played_over_several_turns` (au début du tour 6, la seule action est l'annonce), C12
  `runner_plays_until_deck_and_hand_are_empty` (le joueur annonce et continue : il devient C19, borné en nombre de pas), et le test de
  la spec 000 `empty_hand_and_deck_end_the_game` (même changement que C11). C15 `panel_shows_turn_number` : « Manche 1/4 · Jour (tour 1) ».
- **014** : `panel_shows_time_of_day` : « Manche 1/4 · Jour (tour 1) ».
- **002** : `screen_at_end_has_no_button_and_announces_end` joue les 4 manches du bac à sable (ou prend un scénario d'une manche) et
  attend « Partie terminée : la dernière manche est jouée. » ;
  `frame_loop_stops_on_waiting_and_is_bounded` de même s'il a besoin d'une fin de partie.
- **003** : `game_runner_logs_the_game` : l'action choisie peut aussi être `AnnounceEndOfRound`.
- **012** : `Scenario::new` gagne le nombre de manches (les appels des tests passent un nombre, par exemple 4) ; la correspondance exhaustive de
  `setup_events_come_in_order` gagne `RoundEnded` et `RoundStarted`. Les événements de démarrage ne changent pas.
- **020** : `source_is_unchanged_by_turns` reste vrai (une seule manche).
- **022** : L1 (nuit) devient atteignable en jeu ; `SourceRolled` libère les dés pris (L3).
- **`PlayerView`**, construit champ par champ dans des tests (002, 011, 012, 014, 016, 020, 021, 022), gagne le numéro de la manche.

La spec s'appuie sur les dés pris de la 022, implémentée. La 025, rédigée mais mise de côté (une seule spec à la fois), viendra
après.

## Hors périmètre

- Joueur fantôme, dernier tour des autres joueurs après l'annonce, passer son tour quand un autre a annoncé (MKUE p. 6, 3.d ; p. 15) :
  spec 061.
- Tactiques (p. 4, 2.f et 3) : spec 060. Offres d'unités, d'actions avancées et de sorts (2.c à 2.e) : spec 050.
- Unités prêtes à nouveau (2.g ; p. 5, 9) : spec 042. Compétences et bannières retournées (2.g ; p. 5) : specs 051 et 052.
- Limite de main augmentée près d'une forteresse ou d'une cité à la pioche de début de manche (2.g) : specs 043 et 062.
- Conditions de fin propres au scénario, vérifiées à la fin de chaque tour (p. 4, 4.c, 4.d), et décompte : specs 063 et 064.
- Variante « Daytime Variability » (p. 16) : variantes, hors périmètre du plan.
- Effets d'un jeton de tour retourné (p. 6, 2) : combat entre joueurs et assaut coopératif, hors périmètre du plan.
- Annuler une annonce : spec 024 (le mélange et la relance de la Source sont des révélations).

## Décisions prises à la relecture

Réponses de Mathieu (2026-10-25) :

1. **Q1** — Nombre de manches dans le scénario : **4** pour le bac à sable (2 jours, 2 nuits, comme « Première reconnaissance » en
   solo, p. 18) ; 2 pour le scénario de test de la 012.
2. **Q2** — **Confirmation** avant d'annoncer quand il reste des cartes en main ; pas de confirmation quand l'annonce est obligatoire
   (c'est la seule action possible).
3. **Q3** — Voile de nuit à essayer, plus une icône soleil ou lune devant la ligne de la manche ; on jugera du rendu sur les captures.
4. **Q4** — Aucun raccourci pour annoncer.
5. **Q5** — Termes acceptés : « Fin de manche », « Annoncer la fin de la manche », « Préparer la manche », « Numéro de la manche »,
   « Nombre de manches ».
6. Affichage « Manche 2/4 · Nuit (tour 1) », avec « · ».

## Questions ouvertes

Aucune.

## Corrections pendant l'implémentation

Tests existants modifiés, au-delà de ce que prévoit « Effets sur les specs déjà implémentées » :

- **014, `replaying_log_rebuilds_moves`** (arcanum-core) : la partie dure maintenant 4 manches ; la borne de sécurité passe de 100 à
  1 000 pas, le test lui-même ne change pas.
- **020, `source_is_unchanged_by_turns`** : contrairement à ce qui était écrit, le joueur « premier choix » annonce la fin de la manche
  quand c'est la seule action, ce qui relance la Source. Le test s'arrête avant l'annonce, pour rester dans la manche 1.
- **000, `empty_hand_and_deck_end_the_game`** devient `empty_hand_and_deck_leave_only_end_of_round` : il joue jusqu'à deck, main et
  zone de jeu vides, puis attend `[AnnounceEndOfRound]`.
- **016 C12, `runner_plays_until_deck_and_hand_are_empty`** : inchangé, il passe tel quel.
- **002, `frame_loop_stops_on_waiting_and_is_bounded`** : la partie ne finit plus en une image ; il enchaîne les images (chacune bornée
  par `MAX_STEPS_PER_FRAME`) jusqu'à la fin, en moins de 100 images.
- **002, `screen_at_end_has_no_button_and_announces_end`** : joue les 4 manches du bac à sable, en annonçant la fin de chaque manche
  quand deck et main sont vides.

Choix d'affichage précisés en codant :

- **Confirmation** : c'est un mode de l'écran (`ConfirmingEndOfRound`) ; pendant la confirmation, les cartes ne se jouent pas et le
  bouton « Finir le tour… » est caché, comme dans les autres modes. Le bandeau est caché pendant les autres modes (choix de la carte à
  défausser, du mana, fin du tour).
- **Voile de nuit** : peint sur toute la zone du plateau de jeu juste après les tuiles, donc sous les emplacements explorables, les
  mises en évidence, les sites, les ennemis, la figurine et le trajet.
- **Partie sans scénario** (avant `GameStarted`, jamais affichée en jeu) : la limite affichée est la manche en cours.
- **Tests arcanum-ui** : leur échec a été vérifié avec l'API en place mais vide (10 tests rouges sur 12, plus les 3 tests modifiés).

Vérification manuelle sous Linux (Xvfb) : captures `spec-023-optional.png` (bandeau facultatif), `spec-023-confirm.png`
(confirmation), `spec-023-mandatory.png` (annonce obligatoire) et `spec-023-night.png` (manche 2, nuit, voile, lune, Source relancée,
main neuve).
