# 010 — Plateau de jeu hexagonal

- Statut : implémentée (2026-10-24)
- Jalon : J1
- Dépend de : 000 (arcanum-core)
- Livret : MKUE p. 2-3 (mise en place du plateau de jeu), p. 7 (positions et orientation des tuiles)
- Crates touchées : arcanum-core
- Contenu : `docs/reference/tuiles.md` (transcription des 21 tuiles)

## Objectif

Le moteur sait représenter le plateau de jeu : des cases hexagonales repérées par des coordonnées, regroupées en tuiles de sept cases
posées à des emplacements fixes, chaque case ayant un terrain et éventuellement un site ou un ennemi errant imprimé.
Il connaît le voisinage des cases et des emplacements de tuile, et le contenu des 21 tuiles du jeu de base.
C'est la base des specs 011 (affichage), 012 (mise en place), 014 (déplacement) et 015 (exploration).
Aucune action de jeu n'est ajoutée ici.

## Règles

- **Tuiles de sept cases** : une case centrale et ses six voisines (MKUE p. 2-3, illustration).
- **Trois sortes de tuiles** (MKUE p. 3) : la tuile de départ (faces A et B), les tuiles de campagne (dos vert, 1 à 11) et les tuiles
  royaume (dos brun, 1 à 8), dites « de cité » si une cité est en leur centre.
- **Emplacements fixes, orientation imposée** (MKUE p. 7, 5.a et 5.d ; p. 3). Par défaut, une tuile n'est donc jamais tournée ; une
  exception déclarée par un scénario est hors périmètre.
- **Terrains** : les huit du plateau Jour/Nuit (MKUE p. 2 et p. 7), plus le **littoral** (mer et falaises de la tuile de départ, relevé
  par Mathieu sur les tuiles). Le littoral est toujours infranchissable, alors qu'un lac peut l'être sous certaines conditions (effets de
  cartes) : d'où un terrain distinct. Les coûts et l'inaccessibilité relèvent de la spec 014.
- **Voisinage** : se déplacer vers une case adjacente (MKUE p. 7, 3), explorer depuis une case adjacente à un emplacement libre (p. 7,
  5.a). Cette spec fournit ces relations ; les règles qui s'en servent sont dans 014 et 015.

### Géométrie retenue

Les cases ont une pointe en haut (comme sur les scans, numéro de tuile en bas). Une case est repérée par des **coordonnées axiales**
`(q, r)` : `q` croît vers l'est, `r` vers le sud-est. Les six directions et leur décalage :

| Direction | Code | Décalage `(q, r)` |
|---|---|---|
| Est | `E` | (+1, 0) |
| Nord-Est | `NE` | (+1, −1) |
| Nord-Ouest | `NW` | (0, −1) |
| Ouest | `W` | (−1, 0) |
| Sud-Ouest | `SW` | (−1, +1) |
| Sud-Est | `SE` | (0, +1) |

Un **emplacement de tuile** est repéré par sa case centrale. L'emplacement de la tuile de départ est la case `(0, 0)`.
Les six emplacements voisins de `(0, 0)` sont centrés en :

`(3, −2)`, `(1, −3)`, `(−2, −1)`, `(−3, 2)`, `(−1, 3)`, `(2, 1)`

et ceux d'un emplacement quelconque s'en déduisent par translation. Ce choix est imposé par les symboles des coins : un demi-cercle
au coin droit de la case E, une étoile à la pointe de la case N-E, etc. (voir les scans). Il en découle par exemple que la tuile posée
en `(3, −2)` touche la tuile de départ par ses cases O et S-O, qui bordent les cases N-E et E de la tuile de départ.
Les emplacements forment un pavage : **chaque case appartient à exactement un emplacement**.

## Modèle

Termes du glossaire déjà présents : Village, Monastère, Forteresse, Tour de mage, Cité, Donjon, Tombeau, Antre, Sombre Terrier,
Ruines, Mine, Clairière magique, Portail, Draconien, Ennemi errant, Cristal.

Nouveaux termes, ajoutés au glossaire dans la même PR (traductions à valider, voir Q2) :

| Livret (EN) | Français | Code |
|---|---|---|
| Map | Plateau de jeu | `Map` |
| Space | Case | `Hex` (ses coordonnées) |
| Map tile | Tuile | `Tile`, identifiant `TileId` |
| Starting tile (A / B side) | Tuile de départ (face A / B) | `TileId::StartingA` / `StartingB` |
| Countryside tile | Tuile de campagne | `TileId::Countryside(n)` |
| Core tile | Tuile royaume | `TileId::Core(n)` |
| — (position of a tile) | Emplacement de tuile | `TilePosition` |
| Terrain | Terrain | `Terrain` |
| Plains / Hills / Forest / Wasteland / Desert / Swamp / Lake / Mountain | Plaine / Colline / Forêt / Terre désolée / Désert / Marais / Lac / Montagne | `Terrain::Plains` / `Hills` / `Forest` / `Wasteland` / `Desert` / `Swamp` / `Lake` / `Mountain` |
| — (mer de la tuile de départ) | Littoral | `Terrain::Sea` (voir Q2) |
| Site | Site | `Site` |
| Orc Marauders | Orques maraudeurs | `OrcMarauders` |
| Red / Blue / White / Green (basic colors) | Rouge / Bleu / Blanc / Vert | `BasicColor::Red` / … |
| Direction (of a space) | Direction | `Direction::E` / `NE` / … |

Données, toutes dans `arcanum-core` et sans règle attachée :

- **Définition d'une tuile** (contenu statique, `docs/reference/tuiles.md`) : son identifiant, sa sorte (départ, campagne, royaume),
  et pour la case centrale et chacune des six directions : un terrain, et au plus un **élément imprimé** : un site (avec sa couleur
  pour une mine ou une cité) ou un ennemi errant (orques maraudeurs ou draconien).
- **Plateau de jeu** : l'ensemble des tuiles posées, chacune à un emplacement. Il fait partie de l'état de partie à partir de la spec 012 ;
  ici, c'est une valeur autonome, clonable à bas coût.

Fonctions (le code est libre tant que ce contrat tient) :

- case : `neighbor(direction)`, `neighbors()` (les six, dans l'ordre du tableau), `is_adjacent(autre)`.
- emplacement : `center()`, `hexes()` (les sept cases), `hex_at(direction ou centre)`, `neighbors()` (les six emplacements voisins),
  `containing(case)` (l'unique emplacement qui contient une case).
- plateau de jeu : `place(tuile, emplacement)` → erreur typée si l'emplacement est déjà occupé ou si la tuile est déjà posée ;
  `space(case)` → terrain et élément imprimé de la case, ou rien si aucune tuile n'y est posée ; `tiles()` ;
  `free_neighbor_positions()` (emplacements libres voisins d'au moins une tuile posée, ordre déterministe).

`place` ne vérifie **pas** les règles d'exploration (côte, nombre de tuiles adjacentes) : c'est la spec 015.

## Actions et événements

Aucun. Poser une tuile deviendra un événement dans 012 (mise en place) et 015 (exploration).

## Comportements attendus

Géométrie des cases :

- **C1** — Étant donné une case, ses six voisines sont distinctes, aucune n'est la case elle-même, et la voisine dans une direction
  puis dans la direction opposée ramène à la case de départ (`neighbor_then_opposite_returns_to_start`).
- **C2** — L'adjacence est symétrique, et deux cases sont adjacentes si et seulement si l'une est dans `neighbors()` de l'autre
  (`adjacency_is_symmetric`) — MKUE p. 7.

Emplacements :

- **C3** — Un emplacement contient sa case centrale et ses six voisines, et la case d'une direction est `center().neighbor(direction)`
  (`tile_position_covers_center_and_its_six_neighbors`) — MKUE p. 2-3.
- **C4** — Les six emplacements voisins de `(0, 0)` sont exactement ceux de la géométrie retenue, et deux emplacements voisins
  n'ont aucune case en commun (`neighbor_positions_match_corner_symbols`) — MKUE p. 7, 5.a.
- **C5** — Sur toutes les cases d'un rayon de 12 autour de `(0, 0)`, chaque case appartient à exactement un emplacement, et
  `containing(case)` renvoie celui-là (`every_hex_belongs_to_exactly_one_position`) — MKUE p. 7, 5.a.
- **C6** — La tuile posée en `(3, −2)` a ses cases O et S-O adjacentes à la case N-E de la tuile de départ, et sa case S-O adjacente
  à la case E (`tile_north_east_touches_by_its_west_and_south_west`) — MKUE p. 7.

Contenu :

- **C7** — Le jeu de base définit 21 tuiles : départ A et B, campagne 1 à 11, royaume 1 à 8, chacune avec sept cases
  (`base_game_defines_twenty_one_tiles`) — MKUE p. 3.
- **C8** — Les tuiles royaume 5 à 8 ont une cité en leur centre, les tuiles 1 à 4 non (`core_tiles_five_to_eight_have_a_city`) — MKUE p. 3.
- **C9** — Le contenu de chaque tuile est celui de `docs/reference/tuiles.md` (un test par sorte de tuile, qui vérifie les sept cases)
  (`starting_tile_content`, `countryside_tiles_content`, `core_tiles_content`).
- **C10** — Les décomptes du contenu sont ceux du tableau « Décomptes » de `tuiles.md`, ex. 4 cités de 4 couleurs différentes, 8 mines
  (deux par couleur) (`site_counts_match_transcription`). Ce test protège contre une faute de saisie.

Plateau de jeu :

- **C11** — Étant donné un plateau de jeu vide, quand on pose la tuile de départ A en `(0, 0)`, alors `space((0, 0))` donne Plaine et Portail,
  et `space` de la case N-E `(1, −1)` donne Forêt (`placed_starting_tile_reveals_its_spaces`) — MKUE p. 3.
- **C12** — Orientation : la tuile de campagne 1 posée en `(3, −2)` a son lac sur la case N-E de l'emplacement, `(4, −3)` ;
  aucune fonction de cette spec ne permet de la tourner (`tiles_keep_the_starting_tile_orientation`) — MKUE p. 7, 5.d.
- **C13** — Une case d'un emplacement où aucune tuile n'est posée n'a pas de contenu (`space_without_tile_is_empty`).
- **C14** — Avec la seule tuile de départ posée, les emplacements libres voisins sont les six voisins de `(0, 0)` ; après une tuile
  en `(3, −2)`, ils sont huit, dans un ordre déterministe (`free_neighbor_positions_grow_with_the_map`).

## Cas limites et refus

- **L1** — Poser une tuile sur un emplacement occupé est refusé (erreur typée) et le plateau de jeu est inchangé (`place_on_occupied_position_is_refused`).
- **L2** — Poser une tuile déjà sur le plateau de jeu est refusé, y compris l'autre face de la tuile de départ : A et B ne peuvent pas être posées
  ensemble (`same_tile_cannot_be_placed_twice`).
- **L3** — Les coordonnées sont des entiers signés sans borne pratique : poser une tuile loin de l'origine (ex. 50 emplacements plus loin)
  fonctionne (`far_positions_work`).

## Information cachée

Une tuile posée est publique. La pioche de tuiles (cachée) arrive avec la spec 012.

## Libellés (i18n)

Aucun dans cette spec : `arcanum-core` ne contient pas de texte. Les clés des terrains et des sites (ex. `terrain.forest.name`) seront
ajoutées par la spec 011, qui les affiche.

## Interface

Aucune (spec moteur). L'affichage est la spec 011.

## Hors périmètre

- Coûts de déplacement, terrains infranchissables, cité à coût 2 : spec 014.
- Règles de pose en exploration (adjacence à 2 ou 3 tuiles, côte, tuiles royaume interdites sur la côte) : spec 015.
- Forme du plateau de jeu (coin, ouverte, nombre de colonnes) et composition de la pioche de tuiles : spec 012 (MKUE p. 14).
- Effets des sites et des ennemis imprimés (« When revealed », combat, provocation) : specs 030 et J4.
- Distance entre deux cases : ajoutée quand une spec en aura besoin.
- Rotation d'une tuile, si un scénario déclare une exception à l'orientation imposée : point d'extension du scénario (spec 012
  ou spec du scénario concerné), qui ajoutera alors l'orientation à la pose.
- Tuiles des extensions.

## Questions ouvertes

Aucune. Réponses de Mathieu (2026-10-24) :

- Q1 : les 21 tuiles sont transcrites dès cette spec ; `tuiles.md` corrigée et vérifiée sur les tuiles physiques. « Tuile royaume »
  pour les tuiles à dos brun.
- Q2 : le littoral s'appelle `Terrain::Sea` dans le code ; infranchissable comme la côte.
- Q3 : « plateau de jeu » pour la map (moins ambigu que « carte ») ; « carte » reste réservé aux cartes du deck (actions, sorts,
  artefacts…) ; « terre désolée », « case » et « emplacement de tuile » retenus.
