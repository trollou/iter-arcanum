# Contenu des tuiles du plateau de jeu

Sources et notes de lecture des 21 tuiles du jeu de base, transcrites des scans de `../mage-knight/docs/regles/scans/map_tiles/` pour la spec 010
(`docs/specs/010-plateau-hexagonal.md`). Le contenu lui-même est dans `crates/arcanum-core/content/tiles.ron` (spec 029).
Valeurs reprises, aucun visuel : le jeu dessine ses propres formes.

## Lecture

Toutes les tuiles sont scannées dans l'orientation imposée par le livret (« le numéro dans un coin orienté comme la tuile de départ »,
MKUE p. 7) : numéro en bas, cases à pointe en haut. Les sept cases d'une tuile sont nommées par leur direction depuis la case centrale :

```
     N-O   N-E
  O   Centre   E
     S-O   S-E
```

Le dos de la tuile (`Dos`) dit si c'est une tuile de départ, de campagne (dos vert) ou royaume (dos brun) (MKUE p. 3).

Chaque case : **terrain**, puis le site ou l'ennemi errant imprimé, s'il y en a un. La couleur d'une mine est celle de son cristal ;
celle d'une cité, celle de ses bâtiments.

Le Littoral (mer et falaises de la tuile de départ) est un terrain à part, distinct du Lac.

Transcription vérifiée par Mathieu sur les tuiles physiques le 2026-10-24.

## Contenu des tuiles

Le contenu des 21 tuiles (case centrale et six côtés) est dans
[`crates/arcanum-core/content/tiles.ron`](../../crates/arcanum-core/content/tiles.ron), seule source pour le code et pour la doc (spec 029).
Les termes du fichier sont ceux du glossaire : `Plains` = Plaine, `Sea` = Littoral, `Site(MagicalGlade)` = Clairière magique,
`Rampaging(OrcMarauders)` = Orques…

Les tuiles royaumes 5 à 8 ont une cité au centre (« City icon in the middle », MKUE p. 3) ; les tuiles 1 à 4 n'en ont pas.

## Décomptes (pour vérification)

| Élément | Nombre | Tuiles |
|---|---|---|
| Village | 6 | C1, C2, C3, C4, C8, R5 |
| Monastère | 4 | C5, C7, R1, R6 |
| Clairière magique | 6 | C1, C2, C5, C7, C8, R5 |
| Mine | 8 | C2 verte, C3 blanche, C5 bleue, C6 rouge, R2 verte, R3 blanche, R4 bleue, R8 rouge |
| Forteresse | 5 | C3, C9, C10, R4, R7 |
| Tour de mage | 5 | C4, C9, C11, R2, R3 |
| Donjon | 2 | C7, C9 |
| Antre | 2 | C6, C10 |
| Ruines | 7 | C8, C10, C11, R2, R3, R4, R8 |
| Tombeau | 2 | R1, R3 |
| Sombre Terrier | 2 | R1, R7 |
| Cité | 4 | R5 verte, R6 bleue, R7 blanche, R8 rouge |
| Orques (ennemi errant) | 10 | C1, C2, C4, C5, C6, C7, C8, C11, R5 (×2) |
| Draconien (ennemi errant) | 6 | R2, R4, R6, R7, R8 (×2) |

C = campagne, R = Royaume.
