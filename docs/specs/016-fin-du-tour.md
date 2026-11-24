# 016 — Fin du tour

- Statut : implémentée (2026-10-25)
- Jalon : J1
- Dépend de : 013 (jouer des cartes), 014 (déplacement), 015 (exploration)
- Livret : MKUE p. 3 (limite de main de 5 au niveau 1, défausse), p. 4 (« Deed cards » 5 : fin du tour), p. 6 (5.a : jouer ou
  défausser au moins une carte ; 6.e : points perdus ; 7 : repos ; 8 : annoncer la fin du tour), p. 11 (« End of the Turn » 3 et 7)
- Crates touchées : arcanum-core, arcanum-ui, `assets/i18n/` (fr.json et en.json)

## Objectif

La partie ne s'arrête plus quand la main est vide. Le joueur **finit son tour** quand il le veut : les cartes jouées partent dans la
**défausse**, les points de mouvement et d'influence non dépensés sont **perdus**, il peut **défausser** des cartes de sa main dont il ne
veut pas, puis il **repioche jusqu'à sa limite de main** (5). Un nouveau tour commence, avec un numéro affiché. Quand le deck est épuisé
et la main vide, la partie s'arrête comme aujourd'hui, en attendant la fin de manche (spec 023).

## Règles

- **Annoncer la fin du tour** (MKUE p. 6, 8 et 8.c) : à tout moment, même avec des points ou des cartes non utilisés ; plus aucun effet
  ensuite.
- **Jouer ou défausser au moins une carte** (MKUE p. 6, 5.a). L'exception (main vide au début du tour, deck non vide) n'arrive pas
  encore (voir L6).
- **Nettoyer la zone de jeu** (MKUE p. 11, 3.b ; p. 4, 5) : les cartes jouées vont dans la défausse ; les autres restent en main, sauf
  celles que le joueur décide de défausser.
- **Points perdus** (MKUE p. 6, 6.e ; p. 11, 3) : les points de mouvement et d'influence non dépensés.
- **Défausser avant de repiocher** (MKUE p. 11, 7.a) : autant qu'on veut, sauf des Blessures (pas encore de Blessures, spec 034).
- **Repiocher** (MKUE p. 11, 7.b à 7.d ; p. 3) : jusqu'à la limite de main, **5** au niveau 1 ; au-dessus, ni défausse ni pioche ; deck
  épuisé, on s'arrête : **on ne mélange pas la défausse**.
- **Ordre** (MKUE p. 11) : nettoyer la zone de jeu (3), puis défausser et repiocher (7). Les autres étapes (dés de mana, retraite forcée,
  bénéfices de la case, récompenses, niveau) n'ont encore rien à faire (voir « Hors périmètre »).
- **Nouveau tour.** En solo, le tour suivant est de nouveau celui du joueur (il n'y a pas encore de joueur fantôme, spec 061). Il peut de
  nouveau se déplacer et explorer.
- **Deck épuisé** (MKUE p. 6, 3) : la fin de manche est la spec 023. En attendant, deck vide et main vide = plus aucune action, la partie
  s'arrête sur « Partie terminée » (Q4) ; deck vide et cartes en main = on continue de jouer.
- **Repos** (MKUE p. 6, 7) : sans Blessure, cela revient à finir son tour sans bouger en défaussant une carte, ce que cette spec permet
  déjà : le repos est reporté à la spec 034 (Q1).

### Schéma : où vont les cartes

```
                     repiocher jusqu'à la limite de main (016)
        Deck ──────────────────────────────────────────────────▶ Main
   (ordre caché,                                                │    │
    jamais remélangé                     jouer (013)            │    │  défausser avant de repiocher (016)
    avant la 023)                                               ▼    │  ou pour payer un coût (Improvisation, 013)
                                                         Zone de jeu │
                                                                │    │
                                    fin du tour (016)           ▼    ▼
                                                              Défausse
                                                     (publique, dans l'ordre d'arrivée)
```

### Exemple : un deck de 16 cartes, tour par tour

Un joueur humain (pas le joueur « premier choix » des tests), sans Blessure :

| Tour | Pendant le tour | Fin du tour | Après la pioche : main / deck / défausse |
|---|---|---|---|
| 1 | Main 5, deck 11. Joue Marche, Endurance et Rage de côté. | Garde ses 2 autres cartes. Défausse : 3. Pioche 3. | 5 / 8 / 3 |
| 2 | Joue 4 cartes. | Défausse Concentration, inutile pour l'instant. Défausse : 3 + 4 + 1 = 8. Pioche 5. | 5 / 3 / 8 |
| 3 | Joue 2 cartes. | Garde 3 cartes. Défausse : 10. Pioche 2. | 5 / 1 / 10 |
| 4 | Joue ses 5 cartes. | Défausse : 15. Pioche la dernière carte du deck. | 1 / 0 / 15 |
| 5 | Joue sa dernière carte (ou la défausse à la fin du tour). | Défausse : 16. Deck vide : ne pioche rien. | 0 / 0 / 16 |
| 6 | Deck et main vides : aucune action, « Partie terminée » (fin de manche : spec 023). | — | — |

## Modèle

### Termes

Termes du glossaire déjà présents : Tour (`Turn`), Main (`hand`), Zone de jeu (`PlayArea`), Défausse (`DiscardPile`), Défausser
(`CardDiscarded`), Deck (`DeedDeck`), Limite de main (`hand_limit`), Points de mouvement (`move_points`), Points d'influence
(`influence_points`), Blessure (`Wound`), Manche (`Round`).

Nouveaux termes, ajoutés au glossaire dans la même PR (à valider) :

| Livret (EN) | Français | Code |
|---|---|---|
| End of the Turn | Fin du tour | `EndTurn` (action), `TurnEnded` (événement) |
| Announce (the end of one's turn) | Finir son tour | `EndTurn` |
| Draw (new cards) | Repiocher | `CardDrawn` (existant) |
| (Turn number) | Numéro du tour | `turn` |
| Regular turn | Tour normal | — (seul possible avant la spec 034) |
| Rest | Repos | — (spec 034) |
| Forfeit (one's turn) | Passer son tour | — (spec 023) |

### Nouvelles données dans l'état

- le **numéro du tour** dans la manche, public : **1** dès `GameStarted`, augmenté de 1 à chaque nouveau tour (Q5).

La limite de main reste la constante de la spec 013 (5, niveau 1) ; elle deviendra une donnée du héros avec les niveaux (spec 035).
La zone de jeu, les points, la défausse et le deck existent déjà.

## Actions et événements

| Action | Validation (refusée si…) | Événements produits |
|---|---|---|
| `EndTurn { discard }`, `discard` = cartes de la main à défausser avant de repiocher, dans l'ordre de la main | une carte de `discard` n'est pas en main, ou y figure deux fois ; la zone de jeu est vide **et** `discard` est vide (MKUE p. 6, 5.a) | `TurnEnded`, puis un `CardDiscarded` par carte de `discard`, puis un `CardDrawn` par carte piochée, puis `TurnStarted` |

| Événement | Effet de `apply` | Révèle une information ? |
|---|---|---|
| `TurnEnded { player }` | met les cartes de la zone de jeu sur la défausse, dans l'ordre où elles ont été jouées ; vide la zone de jeu ; remet les points de mouvement et d'influence à 0 | oui, publique |
| `CardDiscarded { player, card }` | inchangé (spec 013) : retire `card` de la main et la met sur la défausse | oui, publique |
| `CardDrawn { player, card }` | inchangé (spec 000) | oui, la carte n'est connue que du joueur |
| `TurnStarted { player, turn }` | met le numéro du tour à `turn` | oui, publique |

**Combien de cartes piocher.** `decide` calcule : limite de main − (taille de la main − cartes de `discard`), au moins 0, au plus la taille
du deck. Les cartes piochées sont celles du dessus du deck, dans l'ordre. Le nombre est porté par les `CardDrawn` produits : `apply`
ne recalcule rien (même raison que l'effet dans `CardPlayed`, spec 013).

**Décision en attente.** Inchangée dans son principe : le joueur choisit tant qu'il a au moins une action légale. `EndTurn` est légale
dès que la zone de jeu n'est pas vide, ou que la main n'est pas vide (en défaussant au moins une carte). Il ne reste donc plus aucune
action seulement quand main, zone de jeu et deck sont vides et qu'aucun déplacement ni exploration n'est payable : c'est le début d'un
tour avec un deck épuisé (Q4).

**Actions légales**, énumérées dans cet ordre : celles des specs 013, 014 et 015 (cartes, déplacements, explorations), inchangées,
**puis** les `EndTurn`, un par sous-ensemble de la main à défausser, par taille croissante, et à taille égale dans l'ordre de la main
(pour une main A, B, C : {}, {A}, {B}, {C}, {A, B}, {A, C}, {B, C}, {A, B, C}). Le sous-ensemble vide n'y est que si la zone de jeu n'est
pas vide. Une main de 5 cartes donne au plus 32 actions `EndTurn` (Q2).

## Comportements attendus

Chaque cas devient au moins un test (nom indicatif entre parenthèses). Sauf mention, partie de bac à sable avec Tovak.

Fin du tour (arcanum-core) :

- **C1** — Marche, Endurance puis Rage de côté jouées, `EndTurn` sans défausse : la zone de jeu est vide, la défausse contient Marche,
  Endurance, Rage dans cet ordre, la main a de nouveau 5 cartes (les 2 gardées + les 3 du dessus du deck), le deck 8
  (`ending_turn_discards_played_cards_and_draws_up_to_hand_limit`) — MKUE p. 4, 5 ; p. 11, 3.b et 7.b.
- **C2** — Les événements produits sont, dans l'ordre : `TurnEnded`, les `CardDiscarded` de la défausse choisie, les `CardDrawn`,
  `TurnStarted { turn: 2 }` (`end_turn_events_come_in_rule_order`) — MKUE p. 11, 3 puis 7.
- **C3** — Avec 3 points de mouvement et 2 d'influence non dépensés, après la fin du tour : 0 et 0 (`unspent_points_are_lost_at_end_of_turn`)
  — MKUE p. 6, 6.e.
- **C4** — 3 cartes jouées, 2 gardées en main, `EndTurn` en défaussant l'une d'elles : elle est sur la défausse après les cartes jouées,
  l'autre reste en main, et le joueur pioche 4 cartes (`discarding_before_drawing_draws_more_cards`) — MKUE p. 11, 7.a.
- **C5** — Aucune carte jouée, `EndTurn` en défaussant une carte : accepté ; défausse 1, main de nouveau 5
  (`discarding_one_card_is_enough_to_end_turn`) — MKUE p. 6, 5.a.
- **C6** — Deck de 1 carte et main vide après avoir joué : la fin du tour pioche 1 carte et s'arrête ; la défausse n'est pas remélangée
  (`drawing_stops_when_deck_runs_out`) — MKUE p. 11, 7.d.
- **C7** — Main de 6 cartes (état construit en appliquant un `CardDrawn` de plus, aucune carte ne fait encore piocher), une carte jouée,
  `EndTurn` sans défausse : la main garde ses 5 cartes restantes et ne pioche rien ; avec 7 cartes, la main en garde 6, sans défausse
  forcée (`hand_above_limit_draws_nothing`) — MKUE p. 11, 7.c.
- **C8** — Après la fin du tour, le numéro du tour vaut 2 ; la vue joueur le montre ; le joueur peut de nouveau jouer des cartes, se
  déplacer et explorer (`next_turn_starts_with_a_fresh_hand`).
- **C9** — Actions légales après avoir joué une carte, main de 4 cartes : les actions des specs 013 à 015 puis 16 `EndTurn` dans
  l'ordre des sous-ensembles ; au début du tour, main de 5 sans carte jouée : 31 `EndTurn` (pas de sous-ensemble vide) ; chacune est
  acceptée par `decide` (`legal_actions_end_with_end_turn_choices`).
- **C10** — Main vidée en jouant toutes ses cartes de côté pour Influence : la décision continue, la seule action légale est `EndTurn`
  sans défausse (`empty_hand_after_playing_can_still_end_turn`).
- **C11** — L'exemple du tableau des règles, joué de bout en bout (16 cartes, 5 fins de tour) : à chaque fin de tour, main, deck et
  défausse ont les tailles du tableau ; au début du tour 6, plus de décision (`whole_deck_is_played_over_several_turns`) — MKUE p. 6, 3.b.
- **C12** — Le joueur « premier choix » (boucle de partie) joue jusqu'à épuiser deck et main : 16 cartes dans la défausse, deck et main
  vides, plus de décision (`runner_plays_until_deck_and_hand_are_empty`).

Information cachée et journal (arcanum-core) :

- **C13** — Dans le journal filtré pour un autre joueur, `TurnEnded`, `CardDiscarded` et `TurnStarted` sont montrés en entier, les
  `CardDrawn` de la fin du tour cachent la carte (`end_turn_hides_only_drawn_cards`).
- **C14** — Rejouer le journal après deux fins de tour redonne le même état (`replaying_log_rebuilds_turns`).

Affichage (arcanum-ui) :

- **C15** — Le panneau latéral affiche « Jour · Tour 1 », puis « Jour · Tour 2 » après la fin du tour (`panel_shows_turn_number`).
- **C16** — Un bouton « Finir le tour… » est présent quand une action `EndTurn` est légale, absent sinon (fin de partie)
  (`end_turn_button_follows_legal_actions`).
- **C17** — Cliquer « Finir le tour… » n'envoie rien encore : l'écran passe en mode fin du tour. Un bandeau « Fin du tour : choisissez
  les cartes à défausser avant de repiocher. Vous piocherez 3 carte(s). » avec « Finir le tour » et « Annuler » ; chaque carte de la main
  n'a plus qu'une case « Défausser » ; pas de bouton de jeu ni de menu de côté ; le plateau de jeu ne met plus en évidence de case ni
  d'emplacement et ignore les clics (`end_turn_mode_lists_cards_to_discard`).
- **C18** — Cocher une carte met à jour le nombre de cartes piochées dans le bandeau (« Vous piocherez 4 carte(s). ») ; deck vide :
  « Vous ne piocherez aucune carte. » (`end_turn_prompt_counts_cards_to_draw`).
- **C19** — Sans carte jouée et sans carte cochée, « Finir le tour » est désactivé et le bandeau ajoute « Vous n'avez joué aucune carte
  ce tour : défaussez-en au moins une. » (`end_turn_needs_a_discard_when_nothing_was_played`).
- **C20** — « Finir le tour » envoie `EndTurn` avec les cartes cochées, dans l'ordre de la main, et quitte le mode
  (`confirming_end_turn_sends_the_action`).
- **C21** — « Annuler » ou Échap quitte le mode sans rien envoyer : main, zone de jeu et points inchangés (`end_turn_mode_can_be_cancelled`).
- **C22** — Touche Fin : ouvre le mode fin du tour si `EndTurn` est légale et qu'aucun autre mode n'est ouvert (choix de la carte à
  défausser d'Improvisation) ; Entrée : confirme, seulement en mode fin du tour et si « Finir le tour » est actif
  (`end_turn_keyboard_shortcuts`).
- **C23** — Avec le vrai `fr.json`, calculer l'écran en tour 1 et 2, en mode fin du tour avec 0, 1 et 2 cartes cochées, deck vide
  compris, ne relève aucun manque (`every_end_turn_key_exists_in_french`).

Vérification manuelle : `cargo run -p arcanum-ui` sous Linux montre « Jour · Tour 1 » ; jouer deux cartes, finir le tour : elles passent
dans la défausse, la main revient à 5 cartes, « Tour 2 » ; continuer jusqu'à épuiser le deck : « Partie terminée ».

## Cas limites et refus

- **L1** — `EndTurn` sans carte jouée ni défaussée (zone de jeu vide, `discard` vide) : `ActionError::NotLegal`, aucun événement
  (`ending_turn_without_playing_or_discarding_is_refused`) — MKUE p. 6, 5.a.
- **L2** — `EndTurn` dont `discard` contient une carte qui n'est pas en main (déjà jouée, dans le deck) ou deux fois la même : refusé
  (`end_turn_discard_must_come_from_hand`) — MKUE p. 11, 7.a.
- **L3** — Une carte défaussée pour Improvisation pendant le tour compte comme défaussée (MKUE p. 6, 5.a), mais Improvisation est
  alors dans la zone de jeu : la règle de L1 est déjà respectée, pas de cas à part.
- **L4** — Fin du tour avec deck vide et main non vide : accepté, rien n'est pioché ; la partie continue tant qu'il reste des cartes en
  main (`deck_empty_with_cards_in_hand_keeps_playing`) — MKUE p. 6, 3.c.
- **L5** — `apply` d'un `TurnEnded` avec une zone de jeu vide : seuls les points sont remis à 0, rien d'autre ne change
  (`turn_ended_with_empty_play_area_only_clears_points`).
- **L6** — Exception de MKUE p. 6, 5.a (main vide au début du tour, deck non vide) : impossible, la fin du tour repioche jusqu'à 5 tant
  que le deck a des cartes. Elle deviendra possible avec des effets qui font jouer ou défausser hors du tour ; pas de test.
- **L7** — Blessures : ne se défaussent pas à la fin du tour (MKUE p. 11, 7.a ; p. 11, « Wounds » 1.a). Pas de Blessure avant la spec
  034, qui ajoutera le refus.

## Information cachée

- Cachées : les cartes piochées pour les autres joueurs (`CardDrawn` filtré, inchangé), l'ordre du deck.
- Publics : la fin du tour, les cartes défaussées (zone de jeu et main), le numéro du tour, la taille de la main et du deck.

## Libellés (i18n)

Ajoutés dans `fr.json` et `en.json`.

| Clé | Paramètres | Français | English |
|---|---|---|---|
| `turn.line` | `time_of_day`, `number` | {time_of_day} · Tour {number} | {time_of_day} · Turn {number} |
| `end_turn.button` | — | Finir le tour… | End turn… |
| `end_turn.prompt` | — | Fin du tour : choisissez les cartes à défausser avant de repiocher. | End of turn: choose the cards to discard before drawing. |
| `end_turn.draw` | `count` | Vous piocherez {count} carte(s). | You will draw {count} card(s). |
| `end_turn.draw_none` | — | Vous ne piocherez aucune carte. | You will draw no card. |
| `end_turn.must_discard` | — | Vous n'avez joué aucune carte ce tour : défaussez-en au moins une. | You played no card this turn: discard at least one. |
| `end_turn.discard` | — | Défausser | Discard |
| `end_turn.confirm` | — | Finir le tour | End turn |
| `end_turn.cancel` | — | Annuler | Cancel |

La ligne « Jour » de la spec 014 devient « Jour · Tour 1 » : `time_of_day.day` et `time_of_day.night` restent, insérés dans `turn.line`.

## Interface

- **Panneau latéral** : la ligne du moment de la journée devient « Jour · Tour 2 ». Sous la zone de jeu, à côté des tailles du deck et de
  la défausse, un bouton **« Finir le tour… »**.
- **Mode fin du tour** : même principe que le choix de la carte à défausser d'Improvisation (spec 013), sans fenêtre modale. Un bandeau
  en haut de la main rappelle ce qu'on fait et combien de cartes on piochera, avec « Finir le tour » et « Annuler ». Chaque carte de la
  main garde son nom, sa pastille et son effet, mais ses boutons sont remplacés par une case **« Défausser »**. Le mode est un état de
  l'écran (arcanum-ui) : le moteur ne reçoit qu'un `EndTurn` complet.
- **Plateau de jeu** : en mode fin du tour, pas de case mise en évidence ni d'emplacement fantôme, et les clics ne déplacent rien ; le
  cadrage reste libre.
- **Fin** : deck et main vides, le message « Partie terminée » s'affiche comme aujourd'hui (Q4).

```
┌─ Panneau latéral ─────────────────────────────────┐
│ Bac à sable                                       │
│ Jour · Tour 2                                     │
│ Mouvement : 1 · Influence : 0                     │
│ ┌───────────────────────────────────────────────┐ │
│ │ Fin du tour : choisissez les cartes à         │ │
│ │ défausser avant de repiocher.                 │ │
│ │ Vous piocherez 4 carte(s).                    │ │
│ │              [Finir le tour]  [Annuler]       │ │
│ └───────────────────────────────────────────────┘ │
│ Main                                              │
│ ● Concentration   Pas encore disponible  ☑ Défausser│
│ ● Instinct        Mouvement 2 ou …       ☐ Défausser│
│ Zone de jeu                                       │
│ ● Marche : Mouvement 2                            │
│ ● Rage : Mouvement 1 (de côté)                    │
│ ● Endurance : Mouvement 2                         │
│ Deck : 8 cartes · Défausse : 0 cartes             │
└───────────────────────────────────────────────────┘
```

### Raccourcis clavier

| Touche | Effet | Contexte |
|---|---|---|
| Fin | Ouvrir le mode fin du tour (main vide : finir le tour directement) | Partie, si finir le tour est possible, hors mode choix |
| Entrée | Finir le tour avec les cartes cochées | Partie, en mode fin du tour, si « Finir le tour » est actif |
| Échap | Quitter le mode fin du tour sans rien faire | Partie, en mode fin du tour |

Fin répond à Début (recentrer, spec 011) et se trouve sur tous les claviers de Windows et de Linux. Pas de touche pour cocher une carte
(même raison que la spec 013, Q4). Ajoutées à `docs/raccourcis.md` (Q6).

## Effets sur les specs déjà implémentées

Signalés ici parce que le sens de tests existants change (règle « jamais modifier un test sans le signaler ») :

- **000** : `empty_hand_ends_the_game` vidait la main pour arrêter la partie ; main vide, finir le tour reste possible. Il devient
  « deck et main vides, plus de décision » (C11). `runner_plays_until_the_hand_is_empty` devient C12
  (`runner_plays_until_deck_and_hand_are_empty`). `runner_records_each_decision` : la dernière décision peut être un `EndTurn`.
- **003** : `game_runner_logs_the_game` : une action choisie peut aussi être un `EndTurn`.
- **012** : `setup_events_come_in_order` : sa correspondance exhaustive sur les événements gagne `TurnEnded` et `TurnStarted`. Les
  événements de démarrage ne changent pas (le tour 1 commence avec `GameStarted`).
- **013** : C15 `empty_hand_leaves_no_decision` devient C10 de cette spec (main vide, la décision continue avec `EndTurn`).
- **014** : C13 `decision_lasts_while_a_move_is_affordable` : « main vide et pas assez de points, plus de décision » devient « …, seule
  action : `EndTurn` ». C14 `legal_actions_list_cards_then_moves` : la liste gagne les `EndTurn` à la fin.
- **015** : `decision_lasts_while_exploring_is_possible` : même changement que la 014 C13.
- **002** : `screen_at_end_has_no_button_and_announces_end` et `frame_loop_stops_on_waiting_and_is_bounded` arrivaient en fin de partie
  en vidant la main ; ils doivent finir les tours jusqu'à épuiser le deck. La ligne « Jour » devient « Jour · Tour n ». `PlayerView`,
  construit champ par champ dans des tests (002, 011, 012, 014), gagne le numéro du tour.

## Hors périmètre

- Repos (tour de repos, récupération lente) : spec 034, avec les Blessures (Q1).
- Annoncer la fin de la manche, passer son tour, remélanger le deck, alternance jour/nuit : spec 023. D'ici là, deck et main vides =
  « Partie terminée » (Q4).
- Rendre les dés de mana à la Source : spec 020. Cristaux de la zone de jeu rendus à la réserve : spec 021. Effet de fin de tour
  d'Euphorie cristalline : spec 021.
- Retraite forcée (p. 11, 2) : J5, avec les effets qui la provoquent.
- Bénéfices de la case en fin de tour (clairière magique, mine de cristal, p. 11, 4) : spec 044.
- Récompenses de combat (p. 11, 5) : spec 031. Montée de niveau et limite de main qui augmente (p. 11, 6) : spec 035.
- Limite de main augmentée près d'une forteresse ou d'une cité, tactiques « Planning » et « Long Night » (p. 11, 7.b, 7.d) : specs 043,
  062 et 060.
- Points perdus au début de la phase d'action : spec de la première action (040 ou 031). Bonus de gloire de Nobles manières : spec 040.
- Vérifier la fin du scénario à la fin de chaque tour (MKUE p. 4, 4.c) : specs 063 et 064 ; le bac à sable n'a pas de fin.
- Tour du joueur fantôme entre deux tours du joueur : spec 061.
- Annuler une fin de tour : spec 024 (la pioche révèle des cartes, l'annulation s'y arrêtera).

## Décisions prises à la relecture

Réponses de Mathieu (2026-10-25), toutes les propositions acceptées :

1. **Q1** — Repos reporté à la spec 034 (Blessures).
2. **Q2** — Une seule action `EndTurn { discard }`, toutes les défausses possibles en actions légales. La décision en deux temps
   attend la spec du premier joueur automatique (Mathieu, 2026-11-13, issue #3) : 1 023 fins de tour pour une main de 10,
   calculées en 2 ms, sans gêne pour le joueur humain.
3. **Q3** — Règle « jouer ou défausser au moins une carte » appliquée dès maintenant.
4. **Q4** — Deck et main vides : « Partie terminée » jusqu'à la fin de manche (spec 023).
5. **Q5** — Numéro du tour dans l'état, affiché « Jour · Tour 2 ». Avec les manches (spec 023), la ligne pourra montrer aussi le numéro
   de la manche : « Jour 1 · Tour 2 », « Nuit 2 · Tour 1 » (noté dans `PLAN.md`).
6. **Q6** — Raccourcis Fin, Entrée et Échap.

## Corrections pendant l'implémentation

Tests existants modifiés, même sens sauf mention (règle « jamais modifier un test sans le signaler ») :

- **Spec 000, `empty_hand_ends_the_game`** renommé `empty_hand_and_deck_end_the_game` : il finit les tours (`EndTurn`) quand la main
  est vide, jusqu'à épuiser le deck, puis vérifie main et deck vides et plus aucune action.
- **Spec 000, `runner_plays_until_the_hand_is_empty`** supprimé : remplacé par C12 (`runner_plays_until_deck_and_hand_are_empty`,
  dans `fin_du_tour.rs`), comme prévu.
- **Spec 003, `game_runner_logs_the_game`** : jusqu'à 1 000 pas au lieu de 100 (la partie dure plusieurs tours) ; l'action choisie peut
  être `PlayCard`, `MoveHero`, `Explore` ou `EndTurn` (`Explore` manquait depuis la spec 015 sans faire échouer le test).
- **Spec 012, `setup_events_come_in_order`** : sa correspondance exhaustive gagne `TurnEnded` et `TurnStarted`.
- **Spec 013, `empty_hand_leaves_no_decision`** renommé `empty_hand_leaves_only_end_turn` : main vidée, la seule action est
  `EndTurn` sans défausse (comme C10).
- **Spec 013, `legal_actions_are_exhaustive`** et **`improvisation_needs_another_card`** : non listés dans la section « Effets », ils
  énuméraient les actions légales exactes ; la liste gagne les `EndTurn` (sous-ensembles non vides, rien n'étant joué).
- **Spec 014, `decision_lasts_while_a_move_is_affordable`** et **`legal_actions_list_cards_then_moves`**, **spec 015,
  `decision_lasts_while_exploring_is_possible`** et **`legal_actions_list_cards_moves_then_explorations`** : les `EndTurn` en fin de
  liste ; « plus de décision » devient « seule action : `EndTurn` ».
- **Spec 014, `panel_shows_time_of_day`** : la ligne attendue devient « Jour · Tour 1 ».
- **Spec 002, `screen_at_end_has_no_button_and_announces_end`** : finit les tours jusqu'à épuiser le deck ; deck attendu 0, défausse 16,
  points 0 et 0. `frame_loop_stops_on_waiting_and_is_bounded` passe sans changement.
- **`PlayerView`** construit champ par champ (spec 002) : `turn: 1` ajouté.
- **`game_screen`** : son paramètre « choix de la carte à défausser » devient un mode d'écran (`ScreenMode` : normal, choix de la
  carte à défausser, fin du tour). Les tests qui l'appellent passent `ScreenMode::Normal` au lieu de `None`, même sens.
- **Raccourcis** : traduits en `Shortcut` (Fin, Entrée, Échap) par l'application et exécutés par la session, ce qui rend C22 testable
  sans egui. Échap quitte désormais le mode en cours, quel qu'il soit.
- **Libellés** : le bandeau dit « carte(s) », comme le tableau des libellés ; les exemples de C17, C18 et du croquis sont alignés.
- **C8** : le test vérifie qu'on peut de nouveau se déplacer au tour 2 ; l'exploration, déjà couverte par la spec 015, n'y est pas
  revérifiée (la figurine du test n'est près d'aucun emplacement libre).

Vérification manuelle sous Linux (Xvfb) : captures `spec-016-mode-fin-du-tour.png`, `spec-016-tour-2.png`,
`spec-016-aucune-carte-jouee.png` dans les fichiers du projet.

## Amendement : main vide (issue privée 25)

Constat (2026-10-25) : main vide, le mode fin du tour affiche toujours « Fin du tour : choisissez les cartes à défausser avant de
repiocher. », alors qu'il n'y a rien à défausser.

Choix de Mathieu (2026-10-26, Q7) : main vide, il n'y a rien à choisir, donc pas de mode fin du tour. Le bouton finit le tour
directement en envoyant `EndTurn { discard: [] }`. Le bandeau avec « Annuler » a été écarté : il ne protégeait que d'un clic par
erreur, au prix d'un clic de plus à chaque tour main vide.

- **C24** — Main vide et au moins une carte jouée : le bouton du panneau s'appelle « Finir le tour » (sans points de suspension, qui
  annoncent un choix à venir) ; le cliquer envoie `EndTurn { discard: [] }` sans passer par le mode fin du tour : le tour suivant
  commence (`end_turn_with_empty_hand_is_immediate`).
- **C25** — Touche Fin, main vide : même effet que le bouton, le tour se finit directement
  (`end_turn_key_with_empty_hand_is_immediate`).
- **L8** — Main vide et zone de jeu vide : aucune `EndTurn` n'est légale (L1), donc pas de bouton ; la phrase « défaussez-en au moins
  une » ne peut jamais s'afficher main vide (MKUE p. 6, 5.a suppose une carte à défausser). Déjà vrai, pas de nouveau test.

Libellés : pas de nouvelle clé, le bouton main vide reprend `end_turn.confirm` (« Finir le tour »). Main non vide, rien ne change.

## Questions ouvertes

Aucune.
