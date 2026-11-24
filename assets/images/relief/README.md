# Décor de relief (spec 075)

Ce qui se dresse sur une case de forêt, de collines ou de montagne : arbres, buttes, pics. Le sol de ces cases est un sol
nu (`terrain/<terrain>.svg`), écrasé comme les autres ; le décor est **un élément debout** posé par-dessus, sans
écrasement. Écrits par le code : `scripts/figurines/relief.py` les régénère, sols nus compris.

## Fichiers

| Fichier | Contenu |
|---|---|
| `forest.svg`, `hills.svg`, `mountain.svg` | décor **plein**, pour une case vide |
| `forest_clear.svg`, `hills_clear.svg`, `mountain_clear.svg` | décor **dégagé**, quand la case porte un site, un jeton ou une figurine |
| `../terrain/forest.svg`, `hills.svg`, `mountain.svg` | sols nus, même cadre que `terrain/*.svg` (173,205 × 200, case droite que le code écrase) |

Les sols nus ont remplacé les anciennes images de ces terrains (fil B de la spec 075) : le décor est lu par le plateau
(`relief_image`, `crates/arcanum-ui/src/board/images.rs`).

## Cadre

- `viewBox="0 0 360 480"`, à la même échelle que les sites et les figurines : 200 unités pour un rayon de case. Le cadre
  s'affiche donc sur **1,8 × 2,4 rayons**.
- **Centre de la case** au point (180, 300) du cadre, soit (0,5 ; 0,625) en proportion. Les pieds des éléments sont
  placés dans la case **déjà inclinée** (`BOARD_TILT` = 0,80) : sommet de la case en y = 140, base en y = 460, flancs en
  x = 6,8 et 353,2.
- Les éléments sont dessinés de l'arrière vers l'avant dans le fichier ; ceux du fond montent jusqu'en haut du cadre.
- **Variante dégagée** : seuls restent les éléments du fond (pied au-dessus du centre de plus de 0,18 rayon) et ceux des
  flancs (à plus de 0,6 rayon du centre, pied au-dessus de 0,12 rayon) : le site, le héros et le défenseur ont leur place
  libre, et aucun élément n'a son pied devant le leur.
- **Variante pleine** : elle a aussi des éléments devant le centre ; posée sur une case vide, elle passe derrière ce qui
  se trouve sur les cases au sud (tri par le centre de la case).

## Style

Celui des figurines (`../figure/README.md`) : contour `#17120c` 2,2, dégradés lumière en haut à gauche, ombre portée en
dégradé radial à chaque pied. Sols nus : aplats sans contour, comme les autres terrains, aux couleurs de `terrain_color`.

## Vérifier

```sh
python3 scripts/figurines/relief.py
node scripts/figurines/planche-plateau.js sortie.png
```
