# #5 — Disparition des monstres vaincus

- Statut : validée le 2026-11-25 (Q1 à Q3 tranchées par Mathieu : réponses proposées retenues)
- Issue : #5 (Epic #31 Finitions v0.1.0, milestone v0.1.0)
- Dépend de : spec 075 (plateau isométrique, essai d'animation du héros, `BoardInput::time`, `animation_repaint`),
  spec 080 (préférences, section « Affichage »), spec 031 (fin du combat) ; toutes implémentées
- Livret : aucune règle de jeu
- Crates touchées : arcanum-ui (session, plateau, préférences), arcanum-persistance (champ des préférences),
  `assets/i18n/`. Rien dans arcanum-core. Aussi : `docs/architecture.md`, `docs/glossaire.md`
- Base : issue #5 (Mathieu, 2026-11-11) ; ligne 079 « Animations de fin de combat » de l'ancien `PLAN.md` ; spec 075, Q6
  et Q7 ; spec 080, hors périmètre (« Case Animations »)

## Objectif

À la fin d'un combat, quand le plateau revient à la place de l'écran de combat, les figurines des monstres vaincus
s'affaissent et s'effacent en moins d'une seconde, au lieu d'avoir déjà disparu. Une case « Animations » des
préférences coupe cette animation et celles de l'essai de la spec 075 (respiration et glissement du héros).

## Règles

Aucune règle du livret : l'animation ne change rien à la partie. Décisions que la spec applique :

- **Qui disparaît** : les ennemis vaincus pendant le combat (`EnemiesDefeated`, ou `EnemyDestroyed` par un sort, spec 054)
  **dont la figurine ou le brouillard était sur le plateau avant le début du combat** : ennemis errants, défenseurs d'un
  site fortifié ou d'une cité (spec 043, 062), jetons restés sur un site d'aventure après un premier échec (spec 044, C41).
  Un ennemi tiré au moment même où le combat commence (donjon, tombeau, incendie du monastère, repaire visité pour la
  première fois) n'a jamais été vu sur le plateau : il n'apparaît pas pour s'effacer. Une créature invoquée non plus.
- **Où** : à l'endroit où le plateau la dessinait (place du jeton errant ou du site d'aventure, paire réduite comprise ;
  place du défenseur à gauche d'un site fortifié), avec l'image de l'ennemi. Un défenseur face cachée avant le combat
  s'efface sous sa figure retournée, pas en brouillard : le combat l'a montré.
- **Quand** : toutes ensemble, à partir de la première image du plateau après la fin du combat (`Action::EndCombat`). Le
  reste du plateau est déjà à jour : drapeau d'une conquête, site redevenu visible sous le jeton errant (comme aujourd'hui).
- **Rien sans vaincu** : un combat sans ennemi vaincu (retraite, tous les ennemis survivent) n'anime rien ; les survivants
  restent comme aujourd'hui.
- **Comme l'essai de la spec 075** : l'écran reste une fonction pure du temps injecté (`BoardInput::time`) ; le plateau
  se redessine au plus 30 fois par seconde, et seulement avec le focus (spec 075, L7).
- **Pendant le combat**, « Voir le plateau » montre le plateau sans les ennemis déjà vaincus, comme aujourd'hui : l'animation
  n'a lieu qu'au retour sur le plateau.
- **Case « Animations »** dans la section « Affichage » des préférences (spec 080), cochée par défaut, gardée d'une session à
  l'autre. Décochée : aucune animation du plateau (ni respiration, ni glissement, ni disparition) et plus d'image redemandée
  pour elles. Les transitions propres à egui (chevauchement des cartes de la main, spec 073) ne sont pas concernées.

## Modèle

Termes nouveaux du glossaire (section du code) :

| Terme | Code | Sens |
|---|---|---|
| Disparition | `Departure` (UI) | Les figurines des ennemis vaincus par le dernier combat, que le plateau efface à son retour. Pas un terme du livret. |
| Animations (préférence) | `Preferences::animations` | Case des préférences qui permet les animations du plateau. |

Données nouvelles, toutes dans arcanum-ui et arcanum-persistance (rien dans l'état ni dans la vue) :

- **Session** : la dernière disparition, relevée dans le journal à la fin d'un combat comme le dernier déplacement (spec 075,
  C35) : un numéro qui change à chaque combat, et pour chaque ennemi sa case et son jeton (`GameSession::last_departure`).
  Effacée par un retour en arrière, un chargement ou une nouvelle partie.
- **Plateau** : `BoardInput::departure` (les ennemis et l'instant où l'effacement commence), dessinés en éléments debout
  `OverlayKind::Departing`, triés avec les autres.
- **Préférences** : `animations: bool`, vrai par défaut et pour un fichier qui ne l'a pas (spec 080, C3 : lecture tolérante).

Le journal est lu par la session, pas par l'écran, comme pour le glissement du héros : l'information n'a rien de caché (le
joueur a vu ces ennemis et les a vaincus).

## Actions et événements

Aucune action ni événement nouveau.

## Comportements attendus

Exemples pris du bac à sable : Rôdeurs (`enemies.ron`, `Prowlers` : armure 3, attaque 4, gloire 2) en (3, −3), Creuseurs
(`Diggers`, fortifiés) en (3, −2), village (2, −3) ; forteresse (7, −5) gardée par des Gardes (partie préparée
`forteresse`) ; donjon (6, −3) (partie préparée `donjon`).

### Disparition relevée par la session

- **C1** — Figurine voisine des Rôdeurs, combat contre eux, Rôdeurs vaincus, fin du combat : la session donne une
  disparition avec les Rôdeurs en (3, −3) (`defeated_rampaging_enemy_departs`).
- **C2** — Deux ennemis défiés (Rôdeurs et Creuseurs, `combat-deux-ennemis`), seuls les Rôdeurs vaincus : la disparition ne
  contient que les Rôdeurs ; les Creuseurs restent dessinés (`only_defeated_enemies_depart`).
- **C3** — Assaut de la forteresse, Gardes vaincus : la disparition contient les Gardes en (7, −5), et le plateau montre déjà
  le drapeau (`defeated_defender_departs`).
- **C4** — Combat du donjon gagné (le Minotaure tiré en entrant) : aucune disparition (`enemy_drawn_with_the_combat_does_not_depart`).
- **C5** — Combat fini sans ennemi vaincu : aucune disparition, et celle d'un combat précédent n'est pas rejouée
  (`no_departure_without_defeat`).
- **C6** — Deux combats de suite (tour suivant) : le numéro de la disparition change (`each_combat_has_its_own_departure`).
- **C7** — Après un retour en arrière, un chargement ou une nouvelle partie, aucune disparition (`departure_is_dropped_on_undo`) ;
  comme le glissement, spec 075, L6, test `walk_is_dropped_on_undo`.

### Effacement sur le plateau

- **C8** — Au début de l'effacement, la figurine de l'ennemi qui disparaît est dessinée à la place et à la taille qu'avait sa
  figurine avant le combat, opaque (`departing_figure_starts_where_it_stood`).
- **C9** — Au milieu de la durée, sa hauteur et son opacité sont à moitié, ses pieds restent posés au même endroit ; à la fin
  de la durée, elle n'est plus dessinée (`departing_figure_sinks_and_fades`).
- **C10** — Les autres éléments debout ne changent pas pendant l'effacement (`departure_leaves_other_overlays_alone`), comme
  la respiration ne touche que le héros (spec 075, C34).
- **C11** — Le plateau demande une image suivante tant qu'un effacement est en cours (au plus 30 par seconde, avec le focus) ;
  plus après (`departure_requests_repaints_until_done`, à côté de `animation_repaint`).

### Préférence « Animations »

- **C12** — Un fichier de préférences sans le champ donne `animations` vrai ; un fichier qui l'a garde sa valeur
  (`animations_default_to_on`, dans arcanum-persistance).
- **C13** — La fenêtre des préférences montre la case « Animations » dans « Affichage », cochée selon la préférence ; la
  changer l'enregistre tout de suite (`animations_checkbox_is_saved`, comme C17 et C20 de la spec 080).
- **C14** — Animations décochées : la hauteur du héros est constante, il est aussitôt sur sa case d'arrivée après un
  déplacement, une disparition n'est pas dessinée, et aucune image suivante n'est demandée
  (`no_board_animation_when_disabled`).

## Cas limites et refus

- **L1** — Un ennemi vaincu sur la case de la figurine (défenseur d'un site assailli, la figurine y est entrée) s'efface
  derrière ou devant elle selon le tri habituel par le pied (vérification à l'œil).
- **L2** — Plateau masqué pendant l'effacement (fenêtre Offres, fin du tour enchaînée) : le temps d'egui continue ; au retour,
  l'effacement est fini ou en cours selon l'heure, rien n'est rejoué.
- **L3** — Fin d'un assaut de cité sans conquête : seuls les défenseurs vaincus s'effacent, les autres restent à leur place
  (C2 pour un ennemi errant ; vérification à l'œil sur `cite-bleue`).

## Information cachée

Rien de nouveau dans la vue (`PlayerView`) : la disparition ne montre que des ennemis que le joueur a combattus, relevés par
la session dans le journal comme le glissement de la spec 075.

## Libellés (i18n)

| Clé | fr | en |
|---|---|---|
| `preferences.display` | Affichage | Display |
| `preferences.animations` | Animations | Animations |

Vérifié le 2026-11-25 : la section « Affichage » de la spec 080 n'a pas encore de titre (`app/windows.rs` dessine la case
du plein écran juste sous la langue ; `grep -n '"display"' assets/i18n/fr.json` ne donne rien). Avec une deuxième case, elle
prend le titre `preferences.display`, en gras comme « Langue » et « Son ».

## Interface

- **Effacement des figurines vaincues** au retour sur le plateau : la figurine s'affaisse vers ses pieds en s'effaçant, en
  `DEPARTURE_SECONDS` (0,8 s, Q1) (C8 à C10, `GameSession::last_departure`, `BoardInput::departure`). Fluidité :
  vérification à l'œil.
- **Case « Animations »** dans la section « Affichage » des préférences, sous « Plein écran au lancement » (C13,
  `PreferencesWindow::animations`).
- Animations décochées : plateau sans respiration, glissement ni effacement (C14).

Parties préparées : aucune nouvelle. Vérification à l'œil sur `combat-deux-ennemis` (défier les deux, vaincre les Rôdeurs
seulement : les Rôdeurs s'effacent, les Creuseurs restent), `forteresse` (conquête : les Gardes s'effacent près du drapeau),
`donjon` (rien ne s'efface) ; puis décocher « Animations » et refaire `combat-deux-ennemis`. Captures en cours d'effacement
par `scripts/verif-manuelle.sh` (une capture peu après la fin du combat).

### Raccourcis clavier

Aucun : la case se coche dans la fenêtre des préférences (Ctrl+,).

## Hors périmètre

- Animations des héros et des monstres au-delà de l'essai de la spec 075 (issue #4), animation du tour du fantôme
  (spec 061, Q4), des combats, des cartes et des dés.
- Disparition d'un ennemi sans victoire (ennemi d'un incendie raté défaussé, `EnemyTokenDiscarded`) : retiré d'un coup,
  comme aujourd'hui.
- Animation de l'apparition des ennemis (tuile révélée, défenseur retourné) et du drapeau posé.
- Son de la disparition : la spec 081 joue déjà ses effets à la victoire ; rien de nouveau.

## Questions ouvertes

Aucune. Décisions de Mathieu (2026-11-25), réponses proposées retenues :

- **Q1. Mouvement de l'effacement** : la figurine s'affaisse vers ses pieds (hauteur de 100 % à 0 %) en s'effaçant
  (opacité de 1 à 0), en 0,8 s (`DEPARTURE_SECONDS`), d'un mouvement qui ralentit à la fin.
- **Q2. Ligne « Animations » du Menu** : non, la seule case des préférences (spec 075, Q7, remplacée par cette décision).
- **Q3. Ennemis jamais vus sur le plateau** : un ennemi tiré au moment où le combat commence (donjon, tombeau, incendie)
  ne s'efface pas (Règles, « Qui disparaît » ; C4).
