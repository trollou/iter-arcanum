# 011 — Affichage du plateau de jeu

- Statut : implémentée (2026-10-24)
- Jalon : J1
- Dépend de : 002 (fenêtre, écran de partie, dossier des ressources), 003 (logs), 010 (plateau de jeu hexagonal)
- Livret : MKUE p. 3 (figurine du héros, orientation des tuiles), p. 7 (orientation imposée). Pas de nouvelle règle de jeu.
- Crates touchées : arcanum-ui, `assets/i18n/` (fr.json et en.json), `assets/images/` (nouveau)

## Objectif

Lancer `cargo run -p arcanum-ui` montre, dans la zone principale de la fenêtre, un plateau de jeu dessiné avec nos propres images :
les cases et leur terrain, le contour des tuiles, les sites et les ennemis errants imprimés, et la figurine du héros.
Le joueur peut zoomer, déplacer le cadrage (souris et clavier) et recentrer, et le survol d'une case affiche ce qu'elle contient.
Aucune action de jeu n'est ajoutée : on regarde, on ne joue pas encore sur le plateau de jeu.

## Règles

Aucune règle nouvelle. L'affichage doit respecter celles de la spec 010 :

- **Orientation** (MKUE p. 7, 5.d ; p. 3) : cases à pointe en haut, comme sur les scans ; l'affichage ne tourne jamais une tuile.
- **Contenu.** Ce qui est dessiné sur une case vient de `Map::space(case)` (spec 010), jamais d'un tableau propre à l'UI.
- **Figurine** (MKUE p. 3) : sa position est donnée, pas calculée : la mise en place la fixera (spec 012) et le déplacement la changera
  (spec 014).
- **Principes de l'UI** (`.claude/rules/arcanum-ui.md`, spec 002) : aucune règle du jeu dans arcanum-ui, aucun texte en dur, aucun visuel officiel,
  la boucle egui n'est jamais bloquée.

## Modèle

### Termes

Nouveaux termes, ajoutés au glossaire dans la même PR :

| Livret (EN) | Français | Code |
|---|---|---|
| Hero | Héros | `Hero` |
| Figure (of a hero) | Figurine | `HeroFigure` |

Termes techniques de l'UI :

| Français | Code | Sens |
|---|---|---|
| Cadrage | `BoardCamera` | Quelle partie du plateau de jeu est visible et à quelle échelle : un point du plateau au centre de la zone, et un zoom. Pas « vue », réservé à la vue joueur. |
| Écran du plateau | `BoardScreen` | Ce qu'il faut dessiner dans la zone du plateau de jeu, déjà calculé et traduit : polygones, images, case survolée. Calculé sans egui, donc testable (comme l'écran de partie de la spec 002). |

### D'où vient le plateau affiché

L'état de partie n'a pas encore de plateau de jeu : il arrive avec la mise en place (spec 012). En attendant, arcanum-ui affiche un
**plateau de démonstration** (terme temporaire, hors glossaire), qui sert aussi à vérifier à l'œil la transcription de `docs/reference/tuiles.md` :

- la tuile de départ A en `(0, 0)` ;
- puis les tuiles de campagne 1 à 11 et les tuiles royaume 1 à 8, dans cet ordre, chacune posée, parmi les emplacements rendus par
  `free_neighbor_positions()` (spec 010) au moment de la poser, sur celui dont la case centrale est la plus proche de `(0, 0)` (en nombre
  de cases), le premier dans l'ordre de cette liste en cas d'égalité. Le plateau reste ainsi compact autour de la tuile de départ.
  Cela donne 20 tuiles, sans règle d'exploration (ce n'est pas une partie) ;
- la figurine du héros sur la case du portail, `(0, 0)`.

L'écran du plateau prend en entrée un plateau de jeu et une position de figurine, pas le plateau de démonstration lui-même :
à la spec 012, il suffira de lui passer ceux de la vue joueur, et le plateau de démonstration disparaîtra.

### Géométrie de l'affichage

Le **rayon** d'une case est la distance de son centre à une pointe. En unités du plateau (avant zoom), le rayon vaut 1, et le centre de
la case `(q, r)` est en :

- `x = √3 × (q + r / 2)`
- `y = 1,5 × r`

(`x` vers la droite, `y` vers le bas, comme l'écran). Les six sommets sont aux angles −90°, −30°, 30°, 90°, 150° et 210° depuis le centre.
Le rectangle qui englobe une case mesure √3 de large et 2 de haut. La fonction inverse donne la case qui contient un point du plateau
(arrondi des coordonnées cubiques, règle classique), ce qui sert au survol.

### Cadrage

- **Position et zoom.** Le cadrage est un point du plateau (affiché au centre de la zone) et un zoom, exprimé comme le rayon d'une case
  à l'écran, en points egui. Le zoom est borné entre **12** et **120** points.
- **Conversions.** Une fonction passe d'un point du plateau à un point de l'écran, et l'inverse, pour une zone donnée (rectangle egui).
- **Zoomer.** Molette : chaque cran multiplie ou divise le zoom par 1,15, et le point du plateau sous le curseur reste sous le curseur.
  Touches `+` et `-` : même facteur, centré sur le milieu de la zone. Toujours dans les bornes.
- **Déplacer.** Glisser avec le bouton gauche ou du milieu : le plateau suit le curseur, au point près. egui distingue clic et glissement,
  donc le clic gauche reste libre pour choisir une case à la spec 014. Flèches du clavier : tant qu'une flèche est enfoncée, le plateau
  défile à **800 points par seconde** dans le sens de la flèche (le contenu se déplace comme si on tirait la caméra de ce côté),
  proportionnellement au temps de l'image ; deux flèches ensemble donnent une diagonale.
- **Recentrer** (touche Début ou bouton « Recentrer ») : revient au **cadrage initial**.
- **Cadrage initial** : le plus grand zoom, dans les bornes, pour lequel toutes les cases posées tiennent dans la zone avec une marge
  de 5 % de chaque côté, centré sur le centre du rectangle qui les englobe. Plateau vide : centré sur `(0, 0)`, zoom 40.
- Les touches ne comptent que si aucun champ de saisie egui n'a le focus (il n'y en a pas encore, mais la règle vaut pour la suite).
- Le cadrage n'est pas sauvegardé (il n'appartient pas à la partie).

### Images

Le plateau de jeu est dessiné avec des **images SVG** que nous créons, dans `assets/images/`, chargées au lancement depuis le dossier
des ressources (spec 002), comme les fichiers de langue. Premier jet volontairement simple, à peaufiner plus tard sans changer de code.

**Style.** Vue de dessus, aplats et quelques motifs simples (arbres pour la forêt, vagues pour le lac et le littoral, sommets pour la
montagne, craquelures pour la terre désolée…), avec des couleurs proches de celles du jeu de base pour que les joueurs s'y retrouvent
(plaine vert clair, désert sable, lac bleu…). Ce sont des dessins originaux : rien n'est décalqué, recopié ni extrait des scans ou du
livret (voir « Assets et droits » dans `CLAUDE.md`, mis à jour dans cette PR).

**Fichiers** (31) :

| Dossier | Fichiers | Forme |
|---|---|---|
| `terrain/` | `plains`, `hills`, `forest`, `wasteland`, `desert`, `swamp`, `lake`, `mountain`, `sea` | Case à pointe en haut qui remplit exactement son cadre (rapport √3 × 2), transparente hors de la case. |
| `site/` | `village`, `monastery`, `keep`, `mage_tower`, `dungeon`, `tomb`, `monster_den`, `spawning_grounds`, `ruins`, `magical_glade`, `portal`, `mine_red`, `mine_blue`, `mine_white`, `mine_green`, `city_red`, `city_blue`, `city_white`, `city_green` | Pictogramme carré, fond transparent. Mines et cités : une image par couleur. |
| `enemy/` | `orc_marauders`, `draconum` | Jeton rond, une couleur par sorte d'ennemi. |
| `hero/` | `figure` | Pion vu de dessus, couleur vive avec contour sombre. |

Les noms reprennent les identifiants du code en `snake_case`, avec l'extension `.svg`. Une table unique dans arcanum-ui associe chaque terrain,
site (avec sa couleur), ennemi et la figurine à son fichier.

**Placement.** L'image du terrain occupe le rectangle qui englobe la case. Le pictogramme d'un site ou le jeton d'un ennemi errant, de
côté 0,9 rayon, est centré dans la moitié haute de la case ; la figurine, de même taille, dans la moitié basse, pour ne pas cacher le
portail. Sous un zoom de **16** points, les pictogrammes, jetons et la figurine ne sont pas dessinés (illisibles), seulement les terrains.

**Netteté et coût.** Une image SVG est convertie en pixels à la taille demandée. Pour ne pas la reconvertir à chaque cran de zoom, on
demande la plus petite taille de la série 32, 64, 128, 256 pixels qui est au moins la taille affichée (en pixels physiques), et egui
réduit le reste. Chaque image n'est donc convertie qu'au plus quatre fois.

**Image manquante ou illisible.** Le jeu ne plante pas : le terrain est dessiné comme un polygone de sa couleur de repli (table
ci-dessous), un site, un ennemi ou la figurine comme un disque de sa couleur de repli. Chaque fichier en défaut est signalé une seule
fois dans les logs, en `warn`, avec son chemin (spec 003).

| Terrain | Repli | | Autre | Repli |
|---|---|---|---|---|
| Plaine | `#A5D36A` | | Rouge (mine, cité) | `#D84040` |
| Colline | `#C9A15E` | | Bleu | `#3F7FD6` |
| Forêt | `#2F7A36` | | Blanc | `#F2F2F2` |
| Terre désolée | `#9A9580` | | Vert | `#3FA34D` |
| Désert | `#EAD58A` | | Autre site | `#FAF5E6` |
| Marais | `#5E7D6B` | | Orques maraudeurs | `#2E5E2E` |
| Lac | `#5AA0E0` | | Draconien | `#8E2A2A` |
| Montagne | `#6E5E58` | | Figurine | `#FFB300` |
| Littoral (`Sea`) | `#1E4A86` | | | |

Les mêmes couleurs servent de base aux images SVG, pour que le repli et l'image se ressemblent.

### Ce que contient l'écran du plateau

Calculé par une fonction pure à partir du plateau de jeu, de la position de la figurine, du cadrage, de la zone, de la position du curseur
(s'il y en a une) et du catalogue de libellés :

- **Cases** : pour chaque case posée dont le polygone touche la zone, ses six sommets et son rectangle à l'écran, son image de terrain et
  sa couleur de repli, et un contour fin. Les cases sans tuile ne sont pas dessinées.
- **Contours de tuile** : pour chaque tuile dont une case est dessinée, le contour extérieur de ses sept cases, plus épais, pour qu'on
  voie les tuiles.
- **Surimpressions** : pictogramme de site, jeton d'ennemi errant, figurine, chacun avec son image, sa couleur de repli et son rectangle.
- **Survol** : si le curseur est sur une case posée, cette case est entourée d'un contour clair épais, et le **panneau de survol** liste,
  une ligne chacun : la tuile (« Tuile de campagne 3 »), le terrain, le site s'il y en a un, l'ennemi errant s'il y en a un, « Votre
  héros » si la figurine y est. Les coordonnées de la case n'y figurent plus, même en développement (Mathieu, 2026-11-13). Sinon, le panneau affiche `board.hover.none`.

L'application passe l'indicateur à vrai dans les exécutables de développement (`debug_assertions`, c'est-à-dire `cargo run` sans
`--release`) et à faux dans les exécutables de publication.

### Disposition de la fenêtre

La fenêtre passe à 1280 × 800 au lancement (redimensionnable). Un panneau latéral droit de 260 points contient, de haut en bas :
le bouton « Recentrer » (le panneau de survol est devenu une infobulle près du curseur : spec 037), puis l'écran de partie du jeu jouet de la spec 002 (main, deck, boutons), inchangé.
La zone centrale est le plateau de jeu, sur un fond gris foncé neutre `#202124`.

## Actions et événements

Aucun. Le cadrage et le survol sont de l'état de l'UI, pas de la partie : ils ne passent pas par le moteur ni par le journal.

## Comportements attendus

Tests de la bibliothèqu'arcanum-ui, sans fenêtre. Les comparaisons de positions se font à 10⁻⁴ près.

Géométrie :

- **C1** — Le centre de `(0, 0)` est `(0, 0)` ; celui de sa voisine E est `(√3, 0)` ; celui de sa voisine S-E est `(√3/2, 1,5)`
  (`hex_centers_follow_pointy_top_layout`) — MKUE p. 7 (orientation).
- **C2** — Les sommets de la case `(0, 0)` sont à distance 1 du centre, le premier est la pointe du haut `(0, −1)`, et deux cases voisines
  partagent exactement deux sommets (`hex_corners_are_shared_with_neighbors`).
- **C3** — Pour toute case d'un rayon de 10 autour de `(0, 0)`, la case qui contient son centre est elle-même, ainsi que celle qui contient
  chacun des six points à 0,8 du centre vers ses sommets (`point_to_hex_inverts_hex_center`).

Cadrage :

- **C4** — Pour un cadrage et une zone quelconques, passer d'un point du plateau à l'écran puis revenir redonne le point
  (`camera_round_trip_is_identity`).
- **C5** — Après un cran de molette avec le curseur en un point de la zone, le point du plateau sous le curseur est le même qu'avant, et le
  zoom a été multiplié par 1,15 (`wheel_zoom_keeps_point_under_cursor`).
- **C6** — La touche `+` multiplie le zoom par 1,15 et garde fixe le point du plateau au milieu de la zone ; `-` le divise
  (`key_zoom_keeps_area_center`).
- **C7** — Zoomer au-delà des bornes s'arrête à 12 ou à 120 points, et le point de référence reste fixe même à la borne (`zoom_is_clamped`).
- **C8** — Un glissement de `(dx, dy)` points déplace l'image à l'écran de tout point du plateau d'exactement `(dx, dy)`
  (`drag_moves_board_with_cursor`).
- **C9** — Flèche droite enfoncée pendant une image de 0,05 s : l'image à l'écran de tout point du plateau se déplace de 40 points vers la
  gauche ; flèches haut et gauche ensemble pendant 0,1 s : de 80 points vers le bas et 80 vers la droite (`arrow_keys_scroll_the_board`).
- **C10** — Le cadrage initial du plateau de démonstration dans une zone de 1000 × 700 points place toutes les cases posées dans la zone
  moins la marge de 5 %, et touche la marge sur au moins un côté (le zoom n'est pas plus petit que nécessaire) (`initial_camera_fits_the_map`).
- **C11** — Après zoom et déplacement, recentrer redonne exactement le cadrage initial (`recenter_restores_initial_camera`).

Plateau de démonstration :

- **C12** — Il contient 20 tuiles : la tuile de départ A en `(0, 0)` (pas la B), les 11 tuiles de campagne et les 8 tuiles royaume, posées dans
  l'ordre décrit ; la figurine est en `(0, 0)` ; deux appels donnent le même plateau (`demo_map_has_twenty_tiles_in_a_fixed_layout`).

Écran du plateau :

- **C13** — Avec un cadrage qui montre tout le plateau de démonstration, l'écran a une case par case posée (140), chacune avec l'image
  `terrain/<terrain>.svg` et la couleur de repli de son terrain, et aucune case pour un emplacement sans tuile
  (`board_screen_draws_each_placed_hex_with_its_terrain`).
- **C14** — Une case dont le polygone est entièrement hors de la zone n'est pas dans l'écran ; une case à cheval sur le bord y est
  (`hexes_outside_the_area_are_culled`).
- **C15** — L'écran a un contour par tuile dont au moins une case est dessinée (20 quand tout est visible)
  (`board_screen_outlines_each_visible_tile`).
- **C16** — La case de la mine verte de la tuile de campagne 2 a la surimpression `site/mine_green.svg` ; la case des orques de la même tuile
  a `enemy/orc_marauders.svg` ; une case sans site ni ennemi n'a pas de surimpression (`printed_sites_and_enemies_get_an_overlay`).
- **C17** — Avec un zoom de 15 points, aucune surimpression n'est dessinée ; à 16 points, elles le sont (`overlays_are_hidden_when_zoomed_out`).
- **C18** — La figurine est dessinée une fois, dans la moitié basse de sa case, et le pictogramme du portail dans la moitié haute
  (`hero_figure_does_not_hide_the_site`).
- **C19** — Les neuf couleurs de repli des terrains sont deux à deux différentes, et différentes du fond de la zone
  (`terrain_fallback_colors_are_distinct`).
- **C20** — Taille de conversion : pour une image affichée sur 20, 64, 65 et 300 pixels, la taille demandée est 32, 64, 128 et 256
  (`svg_size_is_picked_from_a_small_series`).

Images :

- **C21** — Chaque fichier de la table des images existe dans le `assets/images/` du dépôt, et commence par une balise `<svg`
  (`every_board_image_exists`).
- **C22** — Une image absente donne la couleur de repli, et un seul `warn` dans les logs même si l'image est demandée à chaque image
  (`missing_image_falls_back_and_warns_once`).

Survol :

- **C23** — Curseur sur la case du portail du plateau de démonstration : la case est mise en évidence et le panneau
  affiche, dans l'ordre, « Tuile de départ (face A) », « Plaine », « Portail », « Votre héros »
  (`hovering_portal_lists_its_content`).
- **C24** — ~~Coordonnées de la case affichées en développement seulement~~ (supprimé le 2026-11-13 : plus de coordonnées dans le panneau).
- **C25** — Curseur sur la case N-E de la tuile royaume 8 (colline, mine rouge) : le panneau affiche « Tuile royaume 8 », « Colline »,
  « Mine rouge » (`hovering_a_mine_names_its_color`).
- **C26** — Curseur hors de toute case posée, ou hors de la zone, ou absent : aucune case n'est mise en évidence et le panneau affiche
  `board.hover.none` (`hovering_nothing_shows_placeholder`). *Remplacé par la spec 037, C3 : aucune ligne, donc pas d'infobulle
  (`hovering_nothing_shows_no_lines`).*

Libellés :

- **C27** — En calculant l'écran du plateau et le panneau de survol pour **chaque case** du plateau de démonstration avec le vrai `fr.json`,
  le catalogue ne relève aucun manque (`every_board_key_exists_in_french`). La cohérence entre `fr.json` et `en.json` reste garantie
  par les tests de la spec 001.

Vérification manuelle (une fenêtre ne s'ouvre pas en CI) : `cargo run -p arcanum-ui` sous Linux (et Windows quand c'est possible) montre le
plateau de démonstration entier avec ses images ; molette, glisser, flèches, `+`/`-`, Début et « Recentrer » fonctionnent ; les images
restent nettes à tous les zooms ; la fenêtre reste fluide ; le contenu affiché correspond aux tuiles physiques (c'est aussi une relecture
de `tuiles.md`) ; le jeu jouet de la spec 002 marche toujours dans le panneau latéral ; en renommant une image, le jeu affiche le repli
et le signale dans les logs.

## Cas limites et refus

- **L1** — Zone de taille nulle (fenêtre réduite) : l'écran du plateau est vide, le cadrage initial est celui d'un plateau vide, sans
  panique ni division par zéro (`empty_area_draws_nothing`).
- **L2** — Plateau de jeu vide : aucun dessin, cadrage initial centré sur `(0, 0)` au zoom 40 (`empty_map_gives_default_camera`).
- **L3** — Curseur exactement sur l'arête commune à deux cases : une seule case est survolée, toujours la même pour un même point
  (`hover_on_edge_is_deterministic`).
- **L4** — Beaucoup de crans de molette dans une seule image : le zoom reste dans les bornes et n'est jamais nul, infini ou NaN
  (`many_wheel_steps_stay_finite`).
- **L5** — Déplacement qui envoie le plateau loin de la zone : permis (pas de borne), « Recentrer » le ramène
  (`camera_can_leave_the_map_and_come_back`).
- **L6** — Image d'une très longue durée (l'application revient d'une mise en veille, 10 s) avec une flèche enfoncée : le défilement de
  cette image est limité à celui de 0,1 s, pour ne pas faire sauter le plateau hors de vue (`long_frame_scroll_is_capped`).

## Information cachée

Le plateau de jeu est public : tuiles posées, sites et ennemis imprimés. L'écran du plateau ne lit que le plateau de jeu et la position de
la figurine, qu'il recevra de la vue joueur à la spec 012. Aucun jeton d'ennemi n'existe encore (spec 030) : on ne dessine que le symbole
imprimé sur la tuile, que tout le monde voit.

## Libellés (i18n)

Ajoutés dans `fr.json` et `en.json`.

| Clé | Paramètres | Français | English |
|---|---|---|---|
| `board.recenter` | — | Recentrer | Recenter |
| `board.hover.none` | — | ~~Survolez une case~~ | ~~Hover over a space~~ (supprimée par la spec 037) |
| `board.hover.hero` | — | Votre héros | Your hero |
| `board.hover.coords` | `q`, `r` | ~~Case ({q}, {r})~~ | ~~Space ({q}, {r})~~ (supprimée le 2026-11-13) |
| `tile.starting_a` / `tile.starting_b` | — | Tuile de départ (face A) / (face B) | Starting tile (side A) / (side B) |
| `tile.countryside` | `number` | Tuile de campagne {number} | Countryside tile {number} |
| `tile.core` | `number` | Tuile royaume {number} | Core tile {number} |
| `terrain.<terrain>.name` | — | Plaine, Colline, Forêt, Terre désolée, Désert, Marais, Lac, Montagne, Littoral | Plains, Hills, Forest, Wasteland, Desert, Swamp, Lake, Mountain, Coast |
| `site.<site>.name` | — | Village, Monastère, Forteresse, Tour de mage, Donjon, Tombeau, Antre, Sombre Terrier, Ruines, Clairière magique, Portail | Village, Monastery, Keep, Mage Tower, Dungeon, Tomb, Monster Den, Spawning Grounds, Ruins, Magical Glade, Portal |
| `site.mine.<couleur>` | — | Mine rouge, Mine bleue, Mine blanche, Mine verte | Red Mine, Blue Mine, White Mine, Green Mine |
| `site.city.<couleur>` | — | Cité rouge, Cité bleue, Cité blanche, Cité verte | Red City, Blue City, White City, Green City |
| `enemy.orc_marauders.name` | — | Orques maraudeurs | Orc Marauders |
| `enemy.draconum.name` | — | Draconien | Draconum |

Les noms de clé des terrains et des sites reprennent les identifiants du code en `snake_case` (`terrain.wasteland.name`,
`site.spawning_grounds.name`, `site.mine.red`…). Les mines et cités ont une clé par couleur plutôt qu'un paramètre `{color}`, pour
l'accord en français (« Mine verte », « Cité blanche ») et l'ordre des mots en anglais. Quand une clé d'arcanum-core sera nécessaire
(par exemple `Terrain` qui donne sa clé, comme `Action` à la spec 002), elle sera ajoutée dans arcanum-core avec un test, sans texte.

## Interface

Voir « Disposition de la fenêtre », « Cadrage » et « Images ». Aucune action de jeu sur le plateau : un clic simple ne fait rien.

### Raccourcis clavier

Ajoutés aussi au registre `docs/raccourcis.md` :

| Touche | Effet | Quand |
|---|---|---|
| Flèches | Faire défiler le plateau de jeu | Toujours (hors champ de saisie) |
| `+` / `-` (rangée du haut ou pavé numérique) | Zoomer / dézoomer, centré sur le milieu de la zone | Toujours (hors champ de saisie) |
| Début | Recentrer | Toujours (hors champ de saisie) |

## Dépendances

Acceptée par Mathieu (Q8) :

- **`egui_extras`**, de la même version qu'egui (0.37), avec les fonctionnalités `svg` (conversion des SVG en pixels, par `resvg`, en Rust
  pur, sans bibliothèque système, donc identique sous Windows et Linux) et `file` (chargement d'une image depuis un chemin, construit avec
  `PathBuf`, ADR 0004). C'est la crate officielle de l'équipe egui pour les images. Aucune autre dépendance.

## Hors périmètre

- Cliquer une case pour déplacer le héros, afficher le coût ou les cases accessibles : spec 014.
- Montrer les emplacements libres où l'on peut explorer : spec 015.
- Distinguer à l'écran la zone interdite par la forme du plateau de jeu (au-delà du littoral, colonnes hors limite, MKUE p. 14), par
  exemple par un brouillard ou des hachures : spec 012, qui définit cette forme comme paramètre du scénario. Le plateau de démonstration
  n'a pas de forme, il n'y a donc rien à borner ici (remarque de Mathieu, 2026-10-24).
- Plateau de jeu tiré de la partie (vue joueur) au lieu du plateau de démonstration, héros choisi : spec 012.
- Jour et Nuit à l'écran (teinte, terrains dont le coût change) : spec 023.
- Jetons d'ennemis, face cachée ou visible : spec 030. Jetons de bouclier, sites conquis : J4.
- Plusieurs héros sur le plateau de jeu, une figurine par héros : avec le choix du héros (spec 065) ou le multijoueur.
- Artwork définitif : retouches des fichiers SVG, sans spec tant que les noms et formes de fichiers ne changent pas.
- Mini-carte, animations, options d'affichage (daltonisme, taille des pictogrammes), raccourcis configurables, mémorisation du cadrage
  entre deux lancements.
- Rotation d'une tuile (exception de scénario, spec 010) : l'affichage suivra quand la pose aura une orientation.
- Tests du rendu egui (`egui_kittest`) : reportés à la spec 014 (issue #2).

## Corrections pendant l'implémentation

Signalées à Mathieu (2026-10-24) :

- **C9** disait « une image de 0,5 s » alors que L6 limite le défilement d'une image à 0,1 s : C9 utilise désormais une image de 0,05 s
  (40 points).
- **Plateau de démonstration** : « le premier emplacement libre » posait les 19 tuiles en une seule ligne vers le sud-ouest (l'ordre de
  `free_neighbor_positions()` est celui des coordonnées), trop étirée pour le cadrage initial (le zoom tombait sous le minimum). On
  prend maintenant l'emplacement libre le plus proche de `(0, 0)`.

## Décisions prises à la relecture

Réponses de Mathieu (2026-10-24) :

1. **Q1** — Plateau de démonstration jusqu'à la spec 012 : accepté. C'est un terme temporaire, il ne va pas au glossaire.
2. **Q2** — « Héros », « Figurine », « Cadrage » : acceptés.
3. **Q3** — Commandes souris acceptées ; ajout du défilement aux flèches du clavier. Pour penser aux raccourcis à chaque spec : une
   sous-section « Raccourcis clavier » dans l'Interface du gabarit, une note dans `PLAN.md`, et un registre `docs/raccourcis.md` qui évite
   qu'une spec reprenne une touche déjà prise.
4. **Q4** — Des images SVG que nous créons, dans l'esprit du jeu de base pour ne pas dérouter les joueurs, plutôt que des formes et des
   abréviations ; l'artwork sera peaufiné plus tard. D'où les abréviations retirées, la section « Images », Q8 et Q9.
5. **Q5** — Coordonnées au survol seulement dans les exécutables de développement.
6. **Q6** — « Coast » en anglais pour le littoral.
7. **Q7** — Pas d'`egui_kittest` dans cette spec ; on y revient à la spec 014.
8. **Q8** — Dépendance `egui_extras` (fonctionnalités `svg` et `file`) : acceptée.
9. **Q9** — Reformulation de « Assets et droits » dans `CLAUDE.md` et de la règle des visuels dans `.claude/rules/arcanum-ui.md` : acceptée.

## Questions ouvertes

Aucune.
