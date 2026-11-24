# 014 — Déplacement

- Statut : implémentée (2026-10-25)
- Jalon : J1
- Dépend de : 010 (plateau de jeu hexagonal), 011 (affichage du plateau de jeu), 012 (scénario et mise en place), 013 (jouer des cartes)
- Livret : MKUE p. 3 (plateau Jour/Nuit côté jour à la mise en place), p. 6 (5.c, 6.a, 6.d, 6.e : quand se déplacer, points
  perdus), p. 7 (« Movement » 1 à 4 et 6 : coûts, cases infranchissables, limitations, cumul) ; plateau Jour/Nuit
  (`docs/regles/scans/misc/daynight_day.jpg` et `daynight_night.jpg`)
- Crates touchées : arcanum-core, arcanum-ui, `assets/i18n/` (fr.json et en.json)

## Objectif

Les points de mouvement gagnés en jouant des cartes (spec 013) servent enfin : le joueur **déplace sa figurine** case par case sur les
tuiles posées, en payant pour chaque case le **coût de déplacement** de son terrain, qui dépend du **jour** ou de la **nuit**. Lacs,
montagnes et littoral sont **infranchissables**. À l'écran, les cases voisines où le héros peut aller sont mises en évidence avec leur
coût, et un clic sur l'une d'elles y déplace la figurine.

## Règles

- **Points de mouvement** (MKUE p. 7, 2.b et 3) : on additionne les points de toutes ses cartes et effets, puis on avance **case par
  case**, vers une case **adjacente** et **accessible**, en payant le coût du terrain **dans lequel on entre**.
- **Coûts** (plateau Jour/Nuit) :

  | Terrain | Jour | Nuit |
  |---|---|---|
  | Plaine | 2 | 2 |
  | Colline | 3 | 3 |
  | Forêt | 3 | 5 |
  | Terre désolée | 4 | 4 |
  | Désert | 5 | 3 |
  | Marais | 5 | 5 |
  | Lac | infranchissable | infranchissable |
  | Montagne | infranchissable | infranchissable |
  | Littoral | infranchissable | infranchissable |

  Entrer dans une **cité** coûte toujours 2 (MKUE p. 7, 3.a, note). Le Littoral n'est pas sur le plateau Jour/Nuit : la mer de la tuile
  de départ est hors du jeu, donc infranchissable (`docs/reference/tuiles.md`, spec 010).
- **Jour d'abord** (MKUE p. 3) : la première manche est un jour ; l'alternance jour/nuit vient avec la fin de manche (spec 023).
- **Cumul et alternance** (MKUE p. 7, 6.b ; p. 6, 5.c) : on peut jouer d'autres cartes, de mouvement ou d'influence, à tout moment du
  déplacement.
- **Limitations** (MKUE p. 7, 4) :
  - **site fortifié** non conquis (4.a) : les sites fortifiés et leurs défenseurs viennent avec les specs 043 et 062 ; en attendant, on
    **ne peut pas y entrer** (Q2) ;
  - **ennemi errant** (4.b et 4.c) : pas encore de jetons d'ennemis (spec 030) ; le **symbole** d'orques ou de draconiens imprimé sur une
    case n'empêche rien, seul un jeton le fera ;
  - **site d'aventure** (4.d) et autres sites (village, monastère, mine, clairière magique, portail) : des cases ordinaires pour le
    déplacement.
- **Quand** (MKUE p. 6, 6.a et 6.d ; p. 7, 1) : il n'y a encore ni repos (spec 016) ni action (interaction : spec 040, combat : spec 031) :
  on peut se déplacer à tout moment.
- **Points perdus** (MKUE p. 6, 6.e) : ni la phase d'action ni la fin du tour n'existent encore ; la remise à zéro est laissée aux
  specs 016 (fin du tour) et 040 / 031 (première action), voir Q3.

## Modèle

### Termes

Termes du glossaire déjà présents : Jour / Nuit (`TimeOfDay::Day` / `Night`), Mouvement (`Move`), Points de mouvement
(`move_points`), Case (`Hex`), Terrain (`Terrain`), Figurine (`HeroFigure`), Site (`Site`), Cité (`City`), Forteresse (`Keep`),
Tour de mage (`MageTower`), Ennemi errant (`Rampaging`), Littoral (`Terrain::Sea`).

Nouveaux termes, ajoutés au glossaire dans la même PR (à valider) :

| Livret (EN) | Français | Code |
|---|---|---|
| Move cost | Coût de déplacement | `move_cost` |
| Inaccessible (space) | Infranchissable | `MoveCost::Inaccessible` |
| Adjacent (space) | Adjacente | `Hex::neighbors` (existant) |
| Fortified site | Site fortifié | `Site::is_fortified` |
| Assault | Assaut | — (specs 043, 062) |
| Day/Night board | Plateau Jour/Nuit | — (la donnée est `TimeOfDay`) |

### Contenu (arcanum-core, en Rust)

- Une fonction pure donne le **coût de déplacement** d'une case (son contenu) selon le moment de la journée : un nombre de points, ou
  **infranchissable**, ou **pas encore accessible** pour un site fortifié (Q2). Elle code le tableau des règles et l'exception de la cité.
- `Site::is_fortified` : vrai pour forteresse, tour de mage et cité.

### Nouvelles données dans l'état

- le **moment de la journée** (`TimeOfDay`), commun à tous les joueurs : **Jour** dès `GameStarted` (MKUE p. 3). Aucun événement ne le
  change encore (spec 023).

La position de la figurine (spec 012) et les points de mouvement (spec 013) existent déjà.

## Actions et événements

| Action | Validation (refusée si…) | Événements produits |
|---|---|---|
| `MoveHero { to }` | la figurine n'est pas sur le plateau de jeu ; `to` n'est pas adjacente à la figurine ; aucune tuile n'est posée sur `to` ; `to` est infranchissable ou pas encore accessible ; les points de mouvement sont inférieurs au coût | `HeroMoved` |

| Événement | Effet de `apply` | Révèle une information ? |
|---|---|---|
| `HeroMoved { player, from, to, cost }` | met la figurine sur `to` et retire `cost` aux points de mouvement | oui, publique |

`HeroMoved` porte le **coût déjà calculé**, pas seulement la case : `apply` ne consulte ni le contenu des tuiles ni le moment de la
journée (même raison que l'effet dans `CardPlayed`, spec 013). Un `HeroMoved` dont `from` n'est pas la case de la figurine, ou dont le
coût dépasse les points de mouvement, ne change rien (`apply` reste infaillible).

Une action = **une case**. Pas d'itinéraire en plusieurs cases d'un coup (Q4) : c'est la règle (« space by space ») et ce sera
nécessaire pour la provocation (spec 030), qui arrête le déplacement au milieu d'un chemin.

**Décision en attente.** Le joueur doit choisir tant qu'il a **au moins une action légale** : une carte en main, ou une case adjacente
où il peut payer son déplacement. Main vide et pas assez de points pour aucune case voisine : plus de décision, la partie s'arrête sur
« Partie terminée », comme à la spec 013, jusqu'à la fin du tour (spec 016).

**Actions légales**, énumérées dans cet ordre : celles de la spec 013 (jouer une carte), inchangées, **puis** un `MoveHero` par case
adjacente où la figurine peut aller, dans l'ordre des directions `E`, `NE`, `NW`, `W`, `SW`, `SE` (`Direction::ALL`).

## Comportements attendus

Chaque cas devient au moins un test (nom indicatif entre parenthèses).

Coûts (arcanum-core, fonction pure) :

- **C1** — De jour : Plaine 2, Colline 3, Forêt 3, Terre désolée 4, Désert 5, Marais 5 ; Lac, Montagne et Littoral infranchissables
  (`move_costs_by_day`) — MKUE p. 7, 3 ; plateau Jour/Nuit.
- **C2** — De nuit : Forêt 5, Désert 3, les autres comme de jour (`move_costs_by_night`) — MKUE p. 7, 3.a, note.
- **C3** — Une case de cité coûte 2 de jour comme de nuit, quel que soit son terrain ; la fonction la dit pourtant « pas encore
  accessible » (site fortifié, Q2) : le coût 2 est porté par la donnée, pour la spec 062 (`city_costs_two_whatever_the_terrain`) —
  MKUE p. 7, 3.a.
- **C4** — Forteresse, tour de mage et cité sont des sites fortifiés ; village, monastère, mine, clairière magique, portail et les sites
  d'aventure ne le sont pas (`fortified_sites_are_keep_mage_tower_and_city`) — MKUE p. 7, 4.a.

Mise en place (arcanum-core) :

- **C5** — Une partie commence de jour ; la vue joueur le montre (`game_starts_by_day`) — MKUE p. 3.

Se déplacer (arcanum-core), sur le bac à sable (tuile de départ face A, figurine sur le portail) :

- **C6** — Avec 2 points de mouvement (Marche jouée), les actions légales contiennent `MoveHero` vers la Plaine à l'E et la Plaine au N-O,
  pas vers la Forêt au N-E (coût 3) ni vers le Littoral (S-E, S-O, O) (`only_affordable_accessible_neighbors_are_legal`) — MKUE p. 7, 3.
- **C7** — Aller dans la Plaine à l'E produit `HeroMoved` avec un coût de 2 ; après `apply`, la figurine est sur cette case et il reste
  0 point de mouvement (`moving_into_plains_costs_two_points`) — MKUE p. 7, 3.
- **C8** — Avec 3 points, aller dans la Forêt au N-E de jour coûte 3 et laisse 0 point (`moving_into_forest_costs_three_points_by_day`)
  — MKUE p. 7, 3 ; plateau Jour/Nuit.
- **C9** — Se déplacer puis jouer une carte puis se déplacer encore : les points de la nouvelle carte s'ajoutent à ceux qui restaient
  (Marche + Marche = 4 points, une Plaine à 2, il en reste 2, Endurance +2 = 4, une Forêt à 3, il en reste 1)
  (`move_points_left_add_up_with_new_cards`) — MKUE p. 7, 6.b.
- **C10** — Une suite de déplacements sur plusieurs cases, y compris d'une tuile à une tuile voisine révélée à la mise en place, met la
  figurine sur la dernière case et retire la somme des coûts (`moving_across_tiles_pays_each_space`) — MKUE p. 7, 3 et 6.
- **C11** — Une case avec un symbole d'orques maraudeurs imprimé (tuile de campagne 1, N-O) s'entre comme une Forêt ordinaire
  (`printed_rampaging_symbol_does_not_block`) — MKUE p. 7, 4.b (pas de jeton, spec 030).
- **C12** — Une case avec un site non fortifié (village de la tuile de campagne 1) s'entre au coût de son terrain, et on peut continuer
  ensuite (`non_fortified_site_does_not_stop_movement`) — MKUE p. 7, 4.d.
- **C13** — Main vide et points de mouvement restants suffisants : la décision continue avec les seuls `MoveHero` ; main vide et pas
  assez de points pour aucune case voisine : plus de décision (`decision_lasts_while_a_move_is_affordable`).
- **C14** — Les actions légales sont exactement celles de l'énumération, dans son ordre (cartes, puis cases par direction) ; chacune est
  acceptée par `decide` (`legal_actions_list_cards_then_moves`).

Information cachée et journal (arcanum-core) :

- **C15** — `HeroMoved` est montré en entier dans le journal filtré (`hero_moves_are_public`).
- **C16** — Rejouer le journal après quelques déplacements redonne le même état (`replaying_log_rebuilds_moves`).

Affichage (arcanum-ui) :

- **C17** — Le panneau latéral affiche le moment de la journée au-dessus de la ligne des points : « Jour »
  (`panel_shows_time_of_day`).
- **C18** — Avec 2 points de mouvement, l'écran du plateau met en évidence la Plaine E et la Plaine N-O (case jouable), et marque la Forêt
  N-E de son coût (3) sans la mettre en évidence ; les cases du Littoral n'ont aucune marque (`reachable_neighbors_are_highlighted`).
- **C19** — Survol : une case posée affiche une ligne de coût (« Coût de déplacement : 3 », « Infranchissable », « Site fortifié : l'assaut
  n'est pas encore disponible ») ; une case adjacente jouable ajoute « Cliquez pour vous y déplacer » ; une case adjacente trop chère
  ajoute « Il vous manque 1 point de mouvement » (`hover_shows_move_cost`).
- **C20** — Un clic sur une case mise en évidence envoie l'action `MoveHero` vers cette case ; un clic sur une autre case (non adjacente,
  trop chère, infranchissable, vide) ne fait rien ; un glissement ne déplace jamais la figurine (`clicking_a_highlighted_space_moves_the_hero`).
- **C21** — Avec le vrai `fr.json`, calculer l'écran et le survol de chaque case du plateau de jeu du bac à sable, de jour, avec 0, 2 et 5
  points, ne relève aucun manque (`every_move_key_exists_in_french`).

Vérification manuelle : `cargo run -p arcanum-ui` sous Linux montre « Jour » ; jouer Marche met en évidence deux Plaines ; cliquer l'une
déplace la figurine et ramène les points à 0 ; jouer d'autres cartes permet d'entrer dans la tuile de campagne voisine.

## Cas limites et refus

- **L1** — `MoveHero` vers une case non adjacente (deux cases plus loin) ou vers la case de la figurine : `ActionError::NotLegal`,
  aucun événement (`moving_to_a_non_adjacent_space_is_refused`) — MKUE p. 7, 3.
- **L2** — Vers le Littoral, un Lac ou une Montagne, même avec assez de points : refusé (`moving_into_inaccessible_terrain_is_refused`)
  — MKUE p. 7, 3.a.
- **L3** — Vers une case adjacente sans tuile posée : refusé, c'est de l'exploration (spec 015) (`moving_off_the_map_is_refused`).
- **L4** — Pas assez de points (1 point, Plaine à 2) : refusé, et les points ne deviennent jamais négatifs
  (`moving_without_enough_points_is_refused`) — MKUE p. 7, 3.
- **L5** — Vers une forteresse, une tour de mage ou une cité adjacente : refusé, même avec assez de points (Q2)
  (`entering_a_fortified_site_is_not_available_yet`) — MKUE p. 7, 4.a.
- **L6** — `apply` d'un `HeroMoved` dont `from` n'est pas la case de la figurine, ou dont le coût dépasse les points : état inchangé
  (`invalid_move_event_changes_nothing`).

## Information cachée

- Publics : la position de la figurine, chaque déplacement et son coût, le moment de la journée.
- Rien de nouveau n'est caché.

## Libellés (i18n)

Ajoutés dans `fr.json` et `en.json`.

| Clé | Paramètres | Français | English |
|---|---|---|---|
| `time_of_day.day` | — | Jour | Day |
| `time_of_day.night` | — | Nuit | Night |
| `board.hover.move_cost` | `cost` | Coût de déplacement : {cost} | Move cost: {cost} |
| `board.hover.inaccessible` | — | Infranchissable | Inaccessible |
| `board.hover.fortified` | — | Site fortifié : l'assaut n'est pas encore disponible | Fortified site: assault not available yet |
| `board.hover.click_to_move` | — | Cliquez pour vous y déplacer | Click to move there |
| `board.hover.missing_points` | `missing` | Il vous manque {missing} point(s) de mouvement | You need {missing} more Move point(s) |

`time_of_day.night` n'est affiché qu'à partir de la spec 023, mais la clé existe dès maintenant pour que l'écran sache afficher les deux.

## Interface

Maquette à valider : [Maquette spec 014](https://claude.ai/artifact/RzMh5Tay2cswdFNwisVgrg) (son bouton « Passer de nuit » n'existe que
dans la maquette).

- **Panneau latéral** : une ligne « Jour » (ou « Nuit ») juste au-dessus de la ligne des points (spec 013). Le reste est inchangé.
- **Plateau de jeu**, quand le joueur a une décision en attente :
  - chaque case **adjacente** à la figurine où elle peut aller avec ses points est **mise en évidence** : contour clair épais et voile
    lumineux, plus un **jeton de coût** (le nombre, dans un petit disque) ;
  - une case adjacente accessible mais **trop chère** porte seulement son jeton de coût, grisé ;
  - une case adjacente **infranchissable**, pas encore accessible ou sans tuile n'a aucune marque ;
  - les marques ne sont pas dessinées sous le zoom minimal des pictogrammes (`OVERLAY_MIN_ZOOM`, spec 011), comme les autres
    surimpressions.
- **Survol** : le panneau de survol (spec 011) gagne une ligne de coût, et pour une case adjacente une ligne « Cliquez pour vous y
  déplacer » ou « Il vous manque N point(s) de mouvement » (C19).
- **Clic** : un clic gauche **simple** sur une case mise en évidence déplace la figurine ; egui distingue déjà clic et glissement
  (spec 011), le glissement garde le cadrage. Le cadrage ne suit pas la figurine (bouton « Recentrer » et touche Début inchangés).
- Le calcul des marques et du clic se fait dans l'écran du plateau (`BoardScreen`), sans egui, donc testable (Q5).

### Raccourcis clavier

Aucun (Q6). Les flèches font déjà défiler le plateau de jeu (spec 011), et six directions sur une grille hexagonale ne tombent pas bien
sur un clavier ; le clic suffit pour une case à la fois. À revoir si on ajoute un itinéraire en plusieurs cases.

## Effets sur les specs déjà implémentées

Signalés ici parce que le sens de tests existants change (règle « jamais modifier un test sans le signaler ») :

- **013** : la décision en attente ne dépend plus seulement de la main (C13). `empty_hand_leaves_no_decision` (013 C15) et
  `runner_plays_until_the_hand_is_empty` (000) gardent leurs assertions : avec le joueur « premier choix », les cartes passent avant les
  déplacements, la main se vide d'abord, puis la figurine avance tant qu'elle peut payer ; la partie s'arrête quand il n'y a plus
  aucune action légale. Si une assertion de ces tests doit changer à l'implémentation, ce sera signalé.
- **013** : C14 (`legal_actions_are_exhaustive`) vérifiait l'énumération des seules cartes ; la liste gagne les `MoveHero` à la fin, le
  test est complété en conséquence.
- **011** : le clic simple sur le plateau de jeu, qui ne faisait rien, déplace la figurine sur une case mise en évidence.

## Hors périmètre

- Explorer (révéler une tuile, coût 2) : spec 015.
- Fin du tour, repos, remise à zéro des points : spec 016. Perte des points au début de la phase d'action : spec de la première action
  (040 ou 031).
- Nuit : alternance jour/nuit avec la fin de manche, spec 023. Le coût de nuit est déjà dans la fonction de coût (C2).
- Jetons d'ennemis errants, cases occupées, provocation : spec 030.
- Assaut d'un site fortifié, cités : specs 043 et 062. Sites conquis (une forteresse conquise devient accessible) : même specs.
- Effets qui changent les coûts ou permettent d'entrer en terrain infranchissable, case sûre, retraite forcée (MKUE p. 7, 7) : J5 (actions
  avancées, sorts, compétences).
- Autres joueurs sur une case, portail en fin de tour, cité conquise (MKUE p. 7, 8) : hors périmètre (solo d'abord).
- Mise en évidence de **toutes les cases atteignables** avec les points disponibles (plus seulement les voisines), itinéraire en
  plusieurs cases, aperçu d'un chemin : itération suivante, demandée par Mathieu (spec 017).
- Annuler un déplacement : spec 024.

## Décisions prises à la relecture

Réponses de Mathieu (2026-10-25), propositions acceptées :

1. **Q1** — Moment de la journée dans l'état, toujours Jour jusqu'à la spec 023 ; coûts de nuit testés par la fonction de coût seulement.
2. **Q2** — Forteresse, tour de mage et cité pas accessibles tant que l'assaut n'existe pas (specs 043 et 062).
3. **Q3** — Aucune remise à zéro des points dans cette spec : fin du tour (spec 016) et première action (spec 040 ou 031). `PLAN.md` mis à jour.
4. **Q4** — Une case par action, sans itinéraire calculé.
5. **Q5** — Toujours pas d'`egui_kittest` : le clic est testé dans l'écran du plateau, sans egui (issue #2).
6. **Q6** — Aucun raccourci pour se déplacer.
7. **Mise en évidence** — Voisines seulement ; mettre en évidence toutes les cases atteignables avec les points disponibles viendra dans
   une itération suivante (spec 017). Maquette validée.

## Corrections pendant l'implémentation

Tests existants modifiés, même sens sauf mention (règle « jamais modifier un test sans le signaler ») :

- **Spec 000, `empty_hand_ends_the_game`** : vidait la main en jouant de côté pour Mouvement, ce qui donne maintenant 5 points et donc
  des déplacements. Il joue de côté pour Influence : main vide et aucun point de mouvement, plus de décision.
- **Spec 000, `runner_records_each_decision`** : vérifiait que la dernière décision avait une main non vide ; la dernière peut
  maintenant être un déplacement, main vide. Il vérifie que la dernière décision choisit une action légale.
- **Spec 003, `game_runner_logs_the_game`** : chaque action choisie devait être un `PlayCard` ; elle peut aussi être un `MoveHero`.
- **Spec 012, `setup_events_come_in_order`** : sa correspondance exhaustive sur les événements gagne le cas `HeroMoved`.
- **Spec 002, `frame_loop_stops_on_waiting_and_is_bounded`** : la fin de partie après la limite de 2 pas prenait au plus 3 pas ; avec
  les déplacements, elle en prend davantage. Il vérifie qu'elle tient dans une image (moins de `MAX_STEPS_PER_FRAME`).
- **Spec 002, `screen_at_end_has_no_button_and_announces_end`** : joue les 5 cartes de côté pour Influence au lieu de Mouvement, pour
  que la partie s'arrête main vide ; la ligne des points attendue devient « Mouvement : 0 · Influence : 5 ».
- **Spec 002, `screen_buttons_match_legal_actions`** : une action légale peut être un déplacement ; un bouton de carte doit toujours
  mener à un `PlayCard`.
- **`BoardInput` et `PlayerView`** : les tests qui les construisent champ par champ (specs 002, 011, 012) ajoutent `movement: None`
  et `time_of_day: TimeOfDay::Day`. Avec `movement: None`, l'écran du plateau est exactement celui d'avant.
- **Spec 013** : `empty_hand_leaves_no_decision`, `legal_actions_are_exhaustive` et `runner_plays_until_the_hand_is_empty` passent sans
  changement (la partie de `legal_actions_are_exhaustive` n'a pas de figurine, donc pas de déplacement) ; l'ordre cartes puis
  déplacements est vérifié par C14 de cette spec.
- **Coût d'un site fortifié** : `MoveCost::Fortified(n)` garde le coût qu'aura l'entrée (2 pour une cité, celui du terrain sinon), pour
  les specs 043 et 062.
- **Glissement** (C20) : l'écran du plateau donne l'action d'un clic ; l'application ne l'envoie que sur un clic egui (`clicked`), jamais
  sur un glissement. Ce branchement n'est pas testé sans fenêtre (Q5) ; il a été vérifié à la main.

Vérification manuelle sous Linux (Xvfb) : captures `spec-014-debut.png`, `spec-014-cases-en-evidence.png`,
`spec-014-apres-deplacement.png` dans les fichiers du projet.

## Questions ouvertes

Aucune.
