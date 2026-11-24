# Sites debout du plateau (spec 075)

Les 19 sites imprimés sur les tuiles, dessinés **debout** comme des maquettes posées sur la case : des SVG écrits par le
code (`scripts/figurines/sites.py`, qui les régénère), dans le style des figurines (`../figure/README.md` : trait, dégradés,
lumière en haut à gauche, rien de repris du matériel du jeu).

## Fichiers

Un fichier par site, nommé par `site_stem` (`crates/arcanum-ui/src/board/images.rs`) : `village`, `monastery`, `keep`,
`mage_tower`, `city_<couleur>` et `mine_<couleur>` (`blue`, `green`, `red`, `white`), `magical_glade`, `dungeon`, `tomb`,
`monster_den`, `spawning_grounds`, `ruins`, `portal`. La couleur d'une cité est celle de ses toits et fanions, celle d'une
mine celle de ses cristaux.

## Cadre

- `viewBox="0 0 200 200"`, avec `width="200" height="200"`.
- **Échelle** : dessiné comme les figurines, 200 unités pour un rayon de case, mais affiché **agrandi** sur 1,3 × 1,3
  rayon (`SITE_SIZE`, fil B de la spec 075, accepté par Mathieu) pour dominer les figurines ; le pied posé sur la case
  (`SITE_FOOT`).
- **Pied** : (100, 190), au milieu du bas ; le bâti repose sur y ≈ 186. Ombre : premier élément,
  `<ellipse id="ombre" cx="100" cy="184" rx="60 à 92" ry="8">` en dégradé radial noir, **sous** la base : plus bas ou plus
  épaisse, elle fait léviter le bâti. Un bâtiment à base droite peut s'en passer (monastère) ; le village repose sur une
  cour de terre battue (retours de Mathieu, 2026-11-11).
- **Largeur** : au plus 180 unités (0,9 rayon), pour rester dans la case ; la hauteur peut aller jusqu'au haut du cadre
  (tour de mage, cité).
- Le site est centré : le héros se tient devant lui à droite, un défenseur à gauche (spec 075, C14) ; rien d'important ne
  se cache donc dans le coin bas droit.

## Parties

Un groupe `<g id="…">` par partie, de l'arrière vers l'avant, trait posé sur le groupe : le gabarit des figurines
(contour `#17120c`, 2,2 ; détails 1,2 à 1,6). Les noms dépendent du site (`arriere`, `devant`, `remparts`, `tours`,
`lueur`…) : ils ne servent pour l'instant qu'à lire le fichier.

## Ce que le SVG peut contenir

Comme les figurines : `path`, `ellipse`, `circle`, `rect`, `g`, `linearGradient`, `radialGradient`, sans `filter`, `mask`,
`text`, `image`, `style` ni `use`. Moins de 8 Ko.

## Vérifier

```sh
python3 scripts/figurines/sites.py [nom…]                 # régénère tous les sites, ou seulement ceux-là
node scripts/figurines/planche.js sortie.png village keep  # en grand et aux tailles du plateau
node scripts/figurines/planche-plateau.js sortie.png       # sur un petit plateau incliné, avec décor et héros
```
