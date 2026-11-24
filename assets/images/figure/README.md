# Figurines du plateau (spec 075)

Héros, monstres et brouillard du plateau isométrique et de l'écran de combat : des SVG **écrits à la main par le code**,
en pied, tous sur le même gabarit. Le modèle à imiter est `prowlers.svg` (les Rôdeurs, validés par Mathieu) ; les quatre
héros montrent comment varier la pose sans sortir du style.

Rien n'est repris ni décalqué du matériel du jeu (`CLAUDE.md`, Assets et droits) : on dessine d'après le nom, la pile et
les capacités de l'ennemi, ou d'après le portrait de référence du héros (`assets/images/hero-art/`).

## Fichiers

| Fichier | Contenu |
|---|---|
| `<héros>.svg` | `arythea`, `goldyx`, `norowas`, `tovak` |
| `<ennemi>.svg` | un par ennemi de `enemies.ron`, nom en `snake_case` (`prowlers`, `orc_summoners`…) ; tous dessinés : la pile verte et la pile rouge (fil D3), la grise et la violette (fil D4), la marron et la blanche (fil D5) |
| `fog.svg` | brouillard d'un ennemi face cachée, teinté par le code |

## Cadre

```
 0                60               120
 ┌─────────────────┬─────────────────┐ 0
 │   marge : seules une arme, des    │
 │   cornes ou des ailes y montent   │ 8
 │        ┌───────────────┐          │ 16   sommet de la tête ou du casque
 │        │     tête      │          │
 │        │  (40 de haut) │          │ 48   yeux
 │        └───────────────┘          │ 66   menton ; épaules à 62
 │      18 ├── corps ──┤ 102         │
 │           torse 62 à 108          │ 100  ceinture
 │           jambes 104 à 134        │
 │           pieds 130 à 146         │ 146  semelles : ligne de sol
 │        ombre (60, 150) 40 × 8     │ 150
 └───────────────────────────────────┘ 160
```

- `viewBox="0 0 120 160"`, avec `width="120" height="160"`.
- **Pieds** : semelles posées sur y = 146, de part et d'autre de x = 60. Le code pose le point (60, 150) sur la case.
- **Ombre** : premier élément dessiné, `<ellipse id="ombre" cx="60" cy="150" rx="40" ry="8">` en dégradé radial noir
  (opacité 0,55 au centre, 0 au bord). Une créature plus large peut aller jusqu'à `rx="54"`.
- **Proportions de figurine** : grosse tête (un tiers de la hauteur), corps trapu, grands pieds, mains en boule. Une
  créature plus grande que nature (dragon, golem) remplit le cadre en largeur plutôt qu'en hauteur.
- **Pose** : de face, campée sur ses deux pieds. Ce qui dit qui c'est (arme, bouclier, orbe, cornes) dépasse nettement de
  la silhouette, car à 20 points de haut il ne reste qu'elle.
- **Socle** : aucun dans le fichier. Celui du héros est dessiné par le code à sa couleur (`hero_color`).

## Parties nommées

Un groupe `<g id="…">` par partie, dans cet ordre de dessin (de l'arrière vers l'avant). Elles sont séparées pour pouvoir
être animées plus tard (spec 075, essai d'animation) : rien d'une partie ne doit être dessiné dans le groupe d'une autre.

| id | Partie | Remarque |
|---|---|---|
| `ombre` | ombre au sol | toujours |
| `cape` | cape, manteau, ailes repliées, queue | si la figurine en a |
| `jambes` | jambes et pieds | |
| `torse` | buste, ceinture | |
| `bras-gauche` | bras à gauche de l'écran, et ce qu'il tient | |
| `bras-droit` | bras à droite de l'écran, et ce qu'il tient | l'arme principale est de ce côté, sauf raison |
| `arme` | arme tenue à deux mains | à la place où elle doit passer (devant le torse, derrière les mains) |
| `tete` | tête, coiffe, cornes | en dernier : elle recouvre le col |

Une créature sans jambes ou sans bras garde les ids qui ont un sens pour elle (`cape`, `torse`, `tete`) et en ajoute au
besoin (`ailes`, `queue`, `monture`), en français, sans accent.

## Trait

- Contour `#17120c`, épaisseur **2,2**, `stroke-linejoin="round"`, posé une fois sur le groupe.
- Détails intérieurs (plis, coutures, reflets) : 1,2 à 1,6, souvent à demi-opacité. Sourcils et sangles : 3,2.
- Pas de trait plus fin que 1 : il disparaît aux tailles du plateau.

## Couleurs

- Aplats en **dégradé linéaire** à deux ou trois arrêts, lumière en haut à gauche. Pas plus de six dégradés par figurine.
- Une **couleur dominante** par figurine, qui la distingue des autres de sa pile ; pour un héros, elle rappelle sa couleur.
- Teintes communes, à reprendre telles quelles :

| Matière | Clair | Sombre |
|---|---|---|
| Acier | `#f4f7fa` | `#5f6b78` (milieu `#aab4bf`) |
| Or, laiton | `#d9b24a` | `#8a6524` |
| Cuir | `#8a5a34` | `#553419` |
| Cuir noirci, sangles | `#2b1a0c` | |
| Ivoire (crocs, griffes, os) | `#fbf6e6` | |
| Œil qui luit | `#ffd94a` | pupille `#17120c` |

- La nuit, le code assombrit toute la figurine d'un voile bleu : garder des clairs francs (visage, lame, yeux).

## Ce que le SVG peut contenir

Le jeu rend les SVG avec `resvg` : s'en tenir à `path`, `ellipse`, `circle`, `rect`, `g`, `linearGradient` et
`radialGradient`. Pas de `filter`, `mask`, `text`, `image`, `style`, `use` ni de référence externe. Ids des dégradés
courts et locaux au fichier. Poids visé : moins de 8 Ko.

## Dragons

Les quatre dragons (pile rouge) partagent un même corps assis, ailes ouvertes, qui remplit le cadre en largeur ; ils
sont générés par `scripts/figurines/dragons.py` (`python3 scripts/figurines/dragons.py`), qui ne change d'un dragon à
l'autre que la palette et quelques détails : crête et bave verte du dragon du marais, flammes du dragon de feu,
cristaux, glaçons et givre du dragon de glace, écailles d'or et flammes bleue et orange (feu glacial) du haut dragon.
Pour retoucher un dragon, modifier le script, pas le SVG. Parties nommées : `ailes` et `queue` à la place de `cape`.

## Golems et mages

Les trois golems (`golems`, `ice_golems`, `fire_golems`) partagent un même corps de roche trapu, qui remplit le cadre en
largeur ; ils sont générés par `scripts/figurines/golems.py`, qui ne change que la matière (pierre et rune d'ambre, glace
et cristaux, basalte fendu de lave et flammes). Les mages de glace et de feu (`ice_mages`, `fire_mages`) partagent de même
une robe à capuche, bâton à gauche et sort levé à droite, générés par `scripts/figurines/mages.py`. Pour retoucher l'un
d'eux, modifier le script, pas le SVG.

## Brouillard

`fog.svg` est en **gris clairs avec transparence**, sans contour sombre : le code le multiplie par la couleur de la pile
(gris, violet, blanc). Il occupe le même cadre, avec la même ligne de sol, et laisse deviner une silhouette sans rien dire
de l'ennemi.

## Vérifier un dessin

```sh
node scripts/figurines/planche.js sortie.png            # toutes les figurines du dossier
node scripts/figurines/planche.js sortie.png tovak fog  # seulement celles-ci
```

La planche montre chaque figurine en grand, puis aux tailles du plateau (dézoom maximal, zooms 40, 60 et 90) sur plaine
et sur colline, de jour et sous le voile de nuit. Un dessin est bon quand on le reconnaît encore à la plus petite taille.

Sites, décor de relief et icônes d'action ont leur propre planche, sur un petit plateau incliné :
`node scripts/figurines/planche-plateau.js sortie.png` (voir `../site/README.md`, `../relief/README.md`).
