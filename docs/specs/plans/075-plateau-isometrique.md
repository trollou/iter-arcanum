# Plan de développement — spec 075 Plateau isométrique

- Statut : accepté par Mathieu (2026-11-11), avec la spec.
- Spec : [`075-plateau-isometrique.md`](../075-plateau-isometrique.md). Branche de rédaction : `spec/075-plateau-isometrique`.
- Méthode : celle de `CLAUDE.md` (tests rouges, code minimal, refactor ; `scripts/verifier.sh` vert avant chaque push).

## Vue d'ensemble

Deux sortes de travail, qui ne touchent pas les mêmes fichiers :

- le **code** (arcanum-ui), en trois fils successifs **A**, **B**, **C** (spec, Q8) ;
- les **dessins** (SVG de `assets/images/`), en fils à part **D1** à **D5**, sans une ligne de Rust.

```
code     A géométrie ───────────► B éléments debout ─────────► C actions et essai d'animation
                                   ▲ a besoin de D1, D2, D3
dessins  D1 gabarit + héros ──► D2 sites et décor ──► D3 verts et rouges ──► D4, D5 (quand on veut, même après C)
```

## Faut-il des fils distincts pour les figurines ? Oui

1. **Rien de commun avec le code.** Un dessin est un fichier SVG nouveau dans `assets/images/figure/` (dossier neuf, spec) :
   il ne casse aucun test et n'attend aucun type Rust. Les jetons actuels de `enemy/` restent lus jusqu'au fil B, qui
   passe le plateau et l'écran de combat aux figurines et supprime les jetons : le jeu marche à chaque fusion.
2. **Le contexte se remplit vite.** Un dessin soigné se fait en boucle : écrire le SVG, le rendre en PNG, le regarder,
   corriger. Chaque regard est une image lue par le modèle. Les Rôdeurs ont demandé un rendu ; un dragon ou un héros
   ressemblant à son portrait en demandera trois ou quatre. À 33 figurines, un seul fil dépasserait la taille au-delà de
   laquelle il faut le résumer, et un fil résumé perd le style (bilan de la spec 042, issue privée 118).
3. **Ta relecture est différente.** Tu juges un dessin à l'œil sur une planche, pas un diff. Des lots courts te laissent
   corriger le style tôt (après les héros) plutôt qu'après 33 figurines.
4. **Le modèle peut différer** de celui du code (section suivante).

Découpage retenu : **un fil qui fixe le style, puis des lots d'une dizaine de dessins**. Un fil par figurine serait trop
cher (chaque fil relit le gabarit et la planche) ; un seul fil, trop long.

| Fil | Contenu | Nombre | Dépend de | Branche |
|---|---|---|---|---|
| D1 | Gabarit des figurines (`figure/README.md` : cadre, pieds, parties nommées, trait, palette, ombre), script de planche, Rôdeurs repris de l'essai, **4 héros**, brouillard | 6 | — | `chore/075-figurines-heros` |
| D2 | **Sites debout** (19), sol nu et décor de relief des forêts, collines, montagnes (3 sols, 6 décors), icônes des 7 boutons d'action | 35, simples | D1 (style) | `chore/075-sites-et-decor` |
| D3 | Monstres **verts** (5 restants : Creuseurs, Sorcières maudites, Chevaucheurs de loups, Cogneurs, Invocateurs orques) et **rouges** (4 dragons) : ceux qui ont un jeton illustré aujourd'hui | 9 | D1 | `chore/075-figurines-verts-rouges` |
| D4 | Monstres **gris** (4) et **violets** (6) | 10 | D1 | `chore/075-figurines-gris-violets` |
| D5 | Monstres **marron** (5) et **blancs** (4) | 9 | D1 | `chore/075-figurines-marron-blancs` |

- Chaque fil de dessin ouvre **sa PR depuis `main`**, doc et images seules (pas de CI), avec la planche du lot en capture.
  Il se termine par ta relecture à l'œil ; pas de bilan de spec (ce n'est pas une implémentation), sauf remarque de méthode.
- **D3 passe avant le fil B** : les verts et les rouges sont les seuls monstres qui ont un jeton illustré aujourd'hui. Ainsi,
  quand B supprime les jetons, aucun monstre ne perd son image. D4 et D5 ne bloquent rien : ils peuvent passer après le
  fil C ; leurs monstres gardent d'ici là le disque de couleur qu'ils ont déjà aujourd'hui.
- Règle « une chose à la fois » : les fils de dessin sont **successifs entre eux**. Un fil de dessin peut en revanche
  tourner pendant un fil de code, puisqu'ils ne partagent aucun fichier ; à toi de dire si tu le veux.

## Quel modèle pour les dessins ?

Ce qu'on sait, sans plus :

- Les Rôdeurs de l'essai ont été dessinés par **Fable 5.2** (ce fil), du premier coup, et tu les juges très bien.
- Les jetons et les sites actuels, plus simples, viennent des fils précédents (Opus). La comparaison n'est pas juste :
  on leur demandait un jeton rond, pas une figurine en pied. Il n'y a donc **aucune mesure** d'Opus ou de Sonnet sur ce
  travail.

Proposition :

- **D1 avec Fable 5.2** : c'est le fil qui fixe le style, et le seul modèle dont on a vu le résultat.
- **Un essai comparatif dans D1**, peu coûteux : une même figurine (les Creuseurs, `Diggers`) dessinée aussi par Opus 5.5
  et par Sonnet 5.5 en sous-agent, avec le gabarit et les Rôdeurs pour modèle. Les trois versions vont côte à côte sur la
  planche de D1, sans dire laquelle vient d'où. Tu choisis.
- **D2 à D5 avec le modèle le moins cher que tu juges au niveau des Rôdeurs** ; à défaut de différence nette, Fable 5.2,
  pour garder un seul style. D2 (sites, décor, icônes : formes simples) est le meilleur candidat pour un modèle moins cher.
- Haiku n'est pas proposé : le dessin demande de regarder son rendu et de le corriger, ce n'est pas du mécanique.

**Résultat de l'essai (fil D1, 2026-11-11)** : sur la planche à l'aveugle des Creuseurs, Mathieu retient la version
d'**Opus 5.5**, devant celle de Fable 5.2 ; celle de Sonnet 5.5 est écartée (« totalement incohérente »). D2 à D5 se font
donc avec Opus 5.5. Les Creuseurs retenus sont dans les fichiers du projet (`ihm/075/figurines/essai-modeles/C.svg`),
à reprendre comme `figure/diggers.svg` par le fil D3.

Le **code** (fils A, B, C) reste au modèle habituel des fils d'implémentation.

## Découpage en PR

Une PR par fil (Q8) : trois PR de code, cinq PR de dessin. Chacune est verte et laisse le jeu jouable.

| PR | État du jeu après fusion |
|---|---|
| A | Plateau isométrique avec les images actuelles (jetons ronds posés debout), mer, zoom et défilement bornés. |
| D1, D2, D3 | Rien de visible : images ajoutées, pas encore lues. |
| B | Sites, décor, héros et monstres debout, brouillard ; figurines sur l'écran de combat, jetons supprimés ; disque de couleur pour un monstre pas encore dessiné (comme aujourd'hui pour les mêmes). |
| C | Boutons d'action sur les cases, section Ici retirée, essai d'animation du héros. |
| D4, D5 | Les monstres du lot passent du disque à leur figurine. |

La PR privée 193 (rédaction) porte la spec et ce plan ; elle est fusionnée à la validation, et le fil A repart de `main` sur
`spec/075-plateau-isometrique`.

## Découpage en fils de code

Rien dans arcanum-core. **Contrôle des champs de la vue** (issue privée 130) : le plateau lit `PlayerView::map`, `figure`, `hero`,
`scenario`, `enemy_tokens`, `face_down_defenders`, `shields`, `ruins`, `enemies_to_fight`, `interaction`, `healing_points`,
`time_of_day`, tous présents (vérifié dans `crates/arcanum-core/src/view.rs` et `app/board.rs`).

- **Fil A** : étapes 1 à 5. **Fil B** : étapes 6 à 9. **Fil C** : étapes 10 à 15.
- Chaque fil pousse sa dernière étape verte, fait vérifier à l'œil, et s'arrête ; le suivant repart de la spec, du plan et
  de `main`. Le bilan de fin de spec est fait par le fil C.

## Étapes

| # | Étape | Cas | Rouge attendu | Fichiers | Isolée ? | Modèle |
|---|---|---|---|---|---|---|
| 1 | Caméra inclinée : `BOARD_TILT`, `to_screen`, `to_board` | C1 à C3, C22 | C1 : écart vertical égal au zoom au lieu de zoom × 0,80 | `board/camera.rs` | non | fil principal |
| 2 | Cadrage projeté et bornes : dézoom à 0,75 × le cadrage (plancher 16), centre retenu sur le plateau | C4 à C7, L5 | C5 : le dézoom descend à 12 ; C7 : le centre sort du plateau | `board/camera.rs`, `app/board.rs` | non | fil principal |
| 3 | Sol : cases écrasées, mer au-delà de la côte, emplacements libres sombres, tranche | C8 à C11 | C9 : image `board/forbidden` au lieu de la mer ; C11 : aucune tranche | `board/screen.rs`, `board/images.rs` | non | fil principal |
| 4 | Éléments debout avec les images actuelles : pied dans la case, tri de l'arrière vers l'avant, héros devant le site, survol non capté | C13 à C15, C23, L2 | C13 : ordre des cases au lieu de l'ordre de profondeur | `board/screen.rs` | non | fil principal |
| 5 | Dessin du plateau (sol écrasé, tranche, marques au sol, pastilles droites, voile de nuit), vérification à l'œil, docs du fil A | — (à l'œil) | — | `app/board.rs`, `docs/` | non | fil principal |
| 6 | Images des figurines : `figure_image`, repli, héros et monstres debout | C16, C17, L1 | C16 : image `hero/arythea` au lieu de `figure/arythea` | `board/images.rs`, `board/screen.rs` | non | fil principal |
| 7 | Ennemis des sites : figurines à gauche du site, deux réduites au Sombre Terrier, brouillard teinté face cachée, drapeau debout | C18, C19 | C18 : dos de jeton `enemy/back_grey` au lieu de `figure/fog` | `board/screen.rs` | non | fil principal |
| 8 | Décor de relief plein ou dégagé, sol nu des forêts, collines, montagnes | C12 | C12 : aucun élément de décor | `board/screen.rs`, `board/images.rs` | non | fil principal |
| 9 | Écran de combat aux figurines ; `assets/images/enemy/` supprimé ; test des fichiers d'images étendu (un monstre sans figurine est listé comme « à dessiner », pas en échec, tant que D4 et D5 ne sont pas fusionnés) ; dessin, vérification à l'œil, docs du fil B | C20, C20b | C20 : `figure/fog` absent de la liste vérifiée ; C20b : une image est encore cherchée dans `enemy/` | `board/images.rs`, `app/combat.rs`, `app/board.rs`, `tests/affichage_plateau.rs`, `assets/images/enemy/`, `docs/` | non | fil principal |
| 10 | Libellés `fr.json` et `en.json` (tableau de la spec), `panel.here` retirée à l'étape 13 | — | — | `assets/i18n/*.json` | oui | Haiku |
| 11 | `BoardAction` : boutons sur leur case, infobulle avec avertissement et touche, taille fixe, clic prioritaire, « Engager le combat » sur chaque ennemi provoqué | C24 à C29, C33, L3 | C24 : aucun bouton d'action dans `BoardScreen` | `board/screen.rs`, `screen/`, `icons.rs` | non | fil principal |
| 12 | Menu du site pendant l'interaction, `Command::ToggleSiteMenu`, confirmations attachées à la case | C30, C31 | C30 : pas de menu ; C31 : confirmation sans case | `screen/types.rs`, `session.rs`, `board/screen.rs` | non | fil principal |
| 13 | Panneau de droite : section Ici retirée, soins sous les points | C32 | C32 : la section Ici existe encore | `screen/panels.rs`, `screen/mod.rs`, `app/side_panel.rs`, `tests/panneaux.rs` | non | fil principal |
| 14 | Essai d'animation du héros : temps injecté, respiration, glissement le long du trajet, 30 images par seconde avec le focus | C34, C35, L6, L7 | C34 : hauteur constante ; C35 : figurine déjà sur sa case à 75 ms | `board/screen.rs`, `app/board.rs`, `session.rs` | non | fil principal |
| 15 | Dessin des boutons, du menu et des confirmations ; vérification à l'œil (parties préparées de la spec) ; docs (`architecture.md`, `raccourcis.md`, `PLAN.md`, index, `verification-manuelle.md`) ; bilan | — (à l'œil) | — | `app/*.rs`, `docs/` | docs : oui | fil principal, docs : Haiku |

Les fils de dessin n'ont pas d'étape rouge : un SVG n'a pas de comportement. Leur contrôle est la planche (toutes les
figurines du lot aux zooms 40, 60 et 90, sur plaine et sur colline, de jour et sous le voile de nuit) et un test existant,
`every_board_image_exists`, qui lit les fichiers.

## Fil A : tests existants modifiés (fait)

Seuls ces tests ont changé ; tous les autres, dont ceux qui cliquent par `camera.to_screen` (C3, C22), passent tels quels.

| Test | Changement | Cas |
|---|---|---|
| `affichage_plateau.rs` : `zoom_is_clamped`, `initial_camera_fits_the_map`, `camera_can_leave_the_map_and_come_back` | retirés, remplacés par `zoom_out_stops_near_the_fitted_board`, `initial_camera_fits_the_projected_map`, `camera_center_stays_over_the_map` | C5, C4, C7 |
| `affichage_plateau.rs` : `overlays_are_hidden_when_zoomed_out` | retiré | C21 |
| `affichage_plateau.rs` : `hero_figure_does_not_hide_the_site` | adapté : pied de la figurine dans sa case, centre du site hors de son rectangle | C14 |
| `affichage_plateau.rs` : `every_board_image_exists` | 44 images au lieu de 45 (`board/forbidden.svg` supprimé) | C9 |
| `scenario_affichage.rs` : `forbidden_zone_has_an_image_and_a_fallback` | image de la mer, couleur de repli bleue | C9 |

`recenter_restores_initial_camera` passe sans changement. Les tests des sites (C17 à C19) changent avec le fil B.

## Fil B : ce qui a été fait

- Décision de Mathieu (2026-11-11) : dézoom arrêté à **0,5 ×** le cadrage (`FIT_ZOOM_OUT`), plancher de 16 conservé ;
  sites agrandis à 1,3 rayon (`SITE_SIZE`).
- `Overlay` gagne deux champs : `foot` (point posé sur la case, qui sert au tri : le décor de relief est posé par le centre
  de sa case, son rectangle descend plus bas) et `tint` (brouillard teinté). `EnemyCard` gagne `image` et `fallback`.
- Sols nus : `terrain/forest.svg`, `hills.svg`, `mountain.svg` remplacés par les `*_bare.svg` du fil D2.
- Sans son image (chargement en cours), le décor de relief n'est pas remplacé par un disque : le sol nu suffit.
- Retour de Mathieu sur la PR privée 200 : le panneau de droite montre la figurine du héros au lieu du jeton rond
  (`GameScreen::portrait`, test `panel_shows_hero_portrait` adapté).
- C19 (`flag_stands_beside_the_site`) était déjà vert : le drapeau debout vient du fil A.

Tests existants modifiés :

| Test | Changement | Cas |
|---|---|---|
| `plateau_isometrique.rs` : aide `foot()` | lit `Overlay::foot` au lieu du bas du rectangle | C13 à C15 |
| `plateau_isometrique.rs` : `hero_stands_in_front_of_the_site` | décor de relief en tête sur la forteresse (colline) | C12, C14 |
| `plateau_isometrique.rs` : `zoom_out_stops_near_the_fitted_board` | 0,5 au lieu de 0,75 | C5 |
| `affichage_plateau.rs` : `every_board_image_exists` | 76 images, liste `FIGURES_TO_DRAW` (19 ennemis des fils D4 et D5) | C20 |
| `affichage_plateau.rs` : `printed_sites_and_enemies_get_an_overlay` | ignore le décor de relief | C12 |
| `nouvelle_partie.rs` : `figure_overlay_is_the_hero_image` | `figure/norowas` au lieu de `hero/norowas` | C16 |
| `ennemis_errants.rs` : `each_rampaging_enemy_has_its_own_image` | figurines ; ignore le décor de la colline | C17 |
| `sites_fortifies.rs` : `board_draws_face_down_defenders` | brouillard au lieu des dos de jetons ; figurine retournée | C18 |
| `ruines.rs`, `sites_d_aventure.rs` : `board_draws_ruin_tokens`, `board_draws_adventure_tokens` | `figure_image` au lieu de `enemy_image` | C18 |

Les fils D4 et D5 retirent leurs ennemis de `FIGURES_TO_DRAW` : le test échoue si une figurine de la liste existe. Le fil D5 a vidé la liste et l'a retirée du test : chaque figurine d'ennemi est exigée.

## Fil C : ce qui a été fait

- `GameScreen::board_actions` reprend les boutons de la section Ici avec leur case ; `board_screen` les pose en rangées
  au-dessus de la case (à 1,1 rayon du centre), les garde dans la zone, écarte deux rangées qui se chevauchent (L3), et
  leur donne la priorité sur la case pour le clic et l'infobulle (C27).
- Pendant l'interaction, le bouton bulle reste sur le village avec `Command::ToggleSiteMenu` ; le repli est gardé par la
  session et oublié à l'interaction suivante. Le menu et les confirmations s'ouvrent dans une `Area` à droite de la case.
- `GameScreen::player_buttons` (les soins) remplace `here_labels` ; `here_title` et `panel.here` sont retirés.
- Essai d'animation : `GameSession::last_walk` relève les `HeroMoved` du journal après un `Play` ; l'application garde
  l'instant du début. Les cas négatifs C25 et C33 étaient verts d'office au rouge (squelette sans bouton).

Tests existants modifiés :

| Test | Changement | Cas |
|---|---|---|
| `panneaux.rs` : `here_section_lists_site_actions` | remplacé par `healing_buttons_follow_the_points` | C32 |
| `affichage_plateau.rs` : `every_board_image_exists` | 82 images, les 6 icônes d'action | C24 |

## Risques repérés

- **Tests existants** : beaucoup de tests d'arcanum-ui comparent des positions à l'écran. Ceux qui passent par
  `camera.to_screen` ne bougent pas ; ceux qui écrivent une ordonnée en dur changent à l'étape 1. Le fil A les liste tous
  avant ses tests rouges (spec, « Tests existants modifiés ») et les signale dans la PR.
- **Étape 9, monstres pas encore dessinés** : le test des images ne doit ni échouer tant que D4 et D5 manquent, ni laisser
  oublier un monstre. Il tient une liste « à dessiner » qui doit être vide quand D5 est fusionné ; D4 et D5 la raccourcissent.
- **Ressemblance des héros** : dessiner par le code un héros qui rappelle son portrait est plus dur qu'un orque. C'est
  pour cela que D1 commence par eux : si le résultat ne te convient pas, on le sait avant les 29 monstres.
- **Charge du processeur** de l'essai d'animation (étape 14) : à juger sur ton poste Windows ; la limite de 30 images par
  seconde et l'arrêt sans focus sont dans la spec.
- **Lisibilité à petit zoom** : au plancher du dézoom, une figurine fait une vingtaine de points de haut. La planche de
  chaque lot la montre à cette taille.
