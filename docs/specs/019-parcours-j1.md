# 019 — Parcours du jalon 1

- Statut : implémentée (2026-10-27) ; validée le 2026-10-27 (Q1 et Q2 : propositions retenues par Mathieu)
- Jalon : J1
- Dépend de : 012, 013, 014, 015, 016, 017 (J1, déjà implémentées) ; 028 (joueur scripté) ; ADR 0006 (stratégie de tests)
- Livret : MKUE p. 3 (mise en place), p. 4 (*Deed Cards* 3.a, 3.e et 4), p. 6 (5.a, 6.e, 8), p. 7 (3, 3.a, 4.a, 5.a à 5.e),
  p. 11 (*End of the Turn* 3, 7.a à 7.d)
- Crates touchées : arcanum-core (tests seulement : `crates/arcanum-core/tests/parcours_j1.rs`)

## Objectif

Vérifier que les specs du J1 **s'enchaînent dans une vraie partie**, comme le promet la ligne « Jouable » du jalon : *promener son
héros et révéler le plateau de jeu, sans ennemis* (`PLAN.md`). Rien ne change pour le joueur : la spec n'ajoute qu'un test.

Les specs 013 à 017 ont leurs tests sur un état construit à la main ; le parcours 028 traverse déjà le J1, mais pour vérifier la mana,
le retour en arrière et la fin de manche. Celui-ci part de `new_game` et vérifie ce que le J1 apporte : les coûts des terrains, les
cases infranchissables, les deux contraintes de pose d'une tuile de campagne, une tuile révélée qui devient praticable, la défausse
volontaire, les points perdus, et un deck qui s'épuise sans être remélangé.

## Règles

Aucune règle nouvelle : chaque point d'étape renvoie à la règle qu'il vérifie et à la spec qui l'implémente.

- **Mise en place** (MKUE p. 3 ; p. 14 ; p. 18) : bac à sable, tuile de départ A, tuiles de campagne 1 et 2 révélées, figurine sur le
  portail, main de 5 : spec 012, 013. Les tuiles de campagne sont **dans l'ordre des numéros** : la 3 puis la 4 seront révélées, quelle
  que soit la graine.
- **Effet de base, de côté, cumul** (MKUE p. 4, *Deed Cards* 3.a, 3.e et 4) : spec 013. Pas d'Attaque ni de Parade hors combat (spec 013
  Q1).
- **Déplacement case par case** (MKUE p. 7, 3), **coûts de jour** (plateau Jour/Nuit), **infranchissable** (p. 7, 3.a : Littoral, Lac,
  Montagne), **site fortifié** fermé tant qu'il n'y a pas d'assaut (p. 7, 4.a ; spec 014 Q2) : spec 014.
- **Trajet le moins cher** (MKUE p. 7, 3 et 6) : spec 017. Le parcours s'en sert pour marcher, comme le fait l'écran.
- **Explorer** (MKUE p. 7, 5.a à 5.e) : 2 points, figurine immobile, tuile du dessus de la pioche ; les deux contraintes de pose d'une
  tuile de campagne (« deux tuiles adjacentes », « une tuile adjacente qui en borde deux ») : spec 015.
- **Fin du tour** (MKUE p. 6, 5.a et 8 ; p. 11, 3, 7.a à 7.d) : au moins une carte jouée ou défaussée, défausse volontaire, points non
  dépensés perdus, repioche jusqu'à 5, **pas de remélange** quand le deck s'épuise : spec 016.

## Modèle

Aucun changement de l'état ni du glossaire. Le parcours **réutilise** le support de la 028 (`crates/arcanum-core/tests/support/mod.rs`) :
`Parcours` (`play`, `view`, `legal`), le joueur scripté qui choisit l'action **par sa valeur**, et `in_hand`. Il n'utilise ni
`undo`, ni la sauvegarde, ni les tours de remplissage, ni la mana (c'est le J2).

Petites fonctions d'aide propres au parcours, dans `parcours_j1.rs` : jouer une carte de côté pour Mouvement, marcher jusqu'à une case
par le trajet retenu (`routes`, spec 017), lister les cases où un `MoveHero` est légal. Celles de `parcours_j2.rs` qui servent aussi
(`sideways_move`, `walk_to`, `has`) **montent dans `support/`** pour ne pas être recopiées.

### Graine et préconditions

Le plateau de jeu ne dépend pas de la graine (tuiles par numéro) ; seule la **main** en dépend. La graine est fixe (celle de la 028 si
elle convient) ; une précondition est vérifiée juste avant le point d'étape qui en a besoin, avec « graine à rechoisir » sinon (comme
la 028, L2) : une carte de Mouvement 2 (Marche, Endurance ou Célérité) dans la main de départ. Ailleurs, pour obtenir des points, il joue
une carte de Mouvement 2 pour son effet de base s'il en a une, sinon la première de sa main de côté (toutes le peuvent, sans
Blessure) ; s'il n'a plus de carte avant d'avoir les points voulus, c'est aussi « graine à rechoisir ». Héros : **Arythea**, comme la 028.

### Le plateau de jeu traversé

Coordonnées `(q, r)` des cases, emplacements `(i, j)` comme aux specs 012 et 015 (`A = (1, 0)`, `B = (0, 1)`). Contenu des tuiles :
`crates/arcanum-core/content/tiles.ron`. Coordonnées vérifiées à la rédaction sur le plateau de jeu ; le test les revérifie au passage (terrain attendu, L3).

| Case | Tuile | Terrain, site | Rôle dans le parcours |
|---|---|---|---|
| `(0, 0)` | départ A | Plaine, portail | départ |
| `(-1, 0)`, `(-1, 1)`, `(0, 1)` | départ A | Littoral | voisines infranchissables (C3) |
| `(0, -1)` | départ A | Plaine | premier pas (C3) |
| `(2, -3)` | campagne 1 | Plaine, village | d'où l'on explore `A + B` (C6) ; voisin du Lac `(2, -4)` (C4) |
| `(3, -4)` | campagne 3 | Plaine, village | première case de la tuile révélée (C7) |
| `(3, -5)`, `(4, -6)` | campagne 3 | Plaine | d'où l'on explore `2B` (C9) |
| `(5, -6)` | campagne 3 | Colline, forteresse | site fortifié voisin, fermé (C8) |
| `(3, -6)` | campagne 4 | Montagne | infranchissable, révélée en C9 |

### Schéma : le parcours

```
new_game (graine fixe, bac à sable, Arythea) ─ mise en place                              (C1)
│
├─ tour 1 : Mouvement 2 + 1 de côté = 3 points                                           (C2)
│           3 cases voisines légales, pas de Littoral ; un pas sur la Plaine (0,-1) : 1 point
│           plus aucun déplacement légal                                                 (C3)
│           finir le tour en défaussant 1 carte : 1 point perdu, repioche à 5            (C5)
├─ tour 2 : marcher jusqu'au village (2,-3)                                              (C4)
├─ tour 3 : le Lac voisin n'est pas légal ; explorer `A + B` : tuile 3 ─ entrer (3,-4)   (C4, C6, C7)
├─ tour 4 : marcher jusqu'à (4,-6) ; la forteresse voisine n'est pas légale              (C8)
├─ tour 5 : explorer `2B` (adjacent aux tuiles 1 et 3) : tuile 4 ; Montagne voisine      (C9)
└─ dès que le deck ne peut plus remplir la main : repioche partielle, sans remélange     (C10)
```

Le découpage en tours est indicatif (ADR 0006) : seuls les points d'étape, leurs préconditions et leur ordre comptent. Le test peut
placer un point d'étape un tour plus tôt ou plus tard selon la main tirée.

L'emplacement `2A`, exploré par une tuile qui en borde deux (spec 015, Q1), est vérifié en C6 sans être joué : depuis le village,
seul `A + B` est proposé.

## Actions et événements

Aucun nouveau. Le parcours n'utilise que les actions existantes : `PlayCard` (`CardPlay::Basic` sans mana, `CardPlay::Sideways`),
`MoveHero`, `Explore`, `EndTurn` (avec et sans défausse).

## Comportements attendus

Tous dans `crates/arcanum-core/tests/parcours_j1.rs`, points d'étape d'**un seul parcours** (`parcours_se_deplacer_et_explorer`), vérifiés sur
la **vue du joueur**, sur les **actions légales** et sur les événements de la décision qui vient d'être prise. À chaque fin de tour, le
parcours vérifie aussi que **les 16 cartes sont toutes là** : main + deck + zone de jeu + défausse = 16.

- **C1** — Étant donné `new_game`, alors la figurine est sur le portail `(0, 0)`, la main a 5 cartes et le deck 11, la défausse est
  vide, c'est le tour 1 de la manche 1, de jour, sans point ; 3 tuiles sont posées, la pioche a 6 tuiles de campagne et 3 royaume,
  la prochaine tuile est de campagne, et aucun `MoveHero` ni `Explore` n'est légal — MKUE p. 3 ; specs 012, 013, 015.
- **C2** — Quand il joue une carte de Mouvement 2 pour son effet de base, puis une autre de côté pour Mouvement 1, alors il a 3 points
  de mouvement, les deux cartes sont dans la zone de jeu, et aucune action légale ne joue une carte pour Attaque ou Parade — MKUE p. 4,
  *Deed Cards* 3.a, 3.e et 4 ; spec 013.
- **C3** — Alors les seules cases où un `MoveHero` est légal sont les trois voisines de terre du portail (E, N-E, N-O), jamais les
  trois de Littoral ; quand il entre sur la Plaine `(0, -1)`, `HeroMoved` coûte 2 et il reste 1 point ; plus aucun `MoveHero` n'est
  légal (toutes les voisines coûtent 2 ou plus) — MKUE p. 7, 3 et 3.a ; spec 014.
- **C4** — Au tour 2, quand il marche jusqu'au village `(2, -3)` par le trajet retenu, alors chaque `HeroMoved` coûte le terrain où il
  entre et la somme vaut le coût du trajet (spec 017) ; au tour 3, toujours au village et avec au moins 2 points, le Lac `(2, -4)`
  voisin n'est pas un `MoveHero` légal alors que d'autres voisines le sont — MKUE p. 7, 3 et 3.a ; specs 014, 017.
- **C5** — À la fin du tour 1, quand il finit son tour en défaussant une carte de sa main, alors les 2 cartes jouées et la carte
  défaussée sont dans la défausse (3 cartes), le point restant est perdu (0 point), il repioche jusqu'à 5 (3 cartes, deck 8) et
  c'est le tour 2 — MKUE p. 6, 6.e et 8 ; p. 11, 3, 7.a et 7.b ; spec 016.
- **C6** — Au tour 3, sur le village `(2, -3)` : sans point, aucun `Explore` n'est légal ; avec au moins 2 points, `Explore` vers
  `A + B` l'est, et c'est le seul (`2A` ne borde pas le village). Quand il explore, alors il paie 2 points, la figurine ne bouge pas,
  la tuile de campagne 3 est révélée en `A + B` (`Explored` puis `TileRevealed`), la pioche a 5 tuiles de campagne — MKUE p. 7, 5.a à
  5.e ; spec 015.
- **C7** — Alors le village `(3, -4)` de la tuile révélée est un `MoveHero` légal depuis `(2, -3)` ; quand il y entre, il paie 2 — MKUE
  p. 7, 3 ; specs 014, 015.
- **C8** — Au tour 4, quand il marche jusqu'à la Plaine `(4, -6)` (par `(3, -5)`), alors, avec au moins 3 points, la forteresse
  `(5, -6)` voisine (Colline, 3 points) n'est pas un `MoveHero` légal, alors que la Forêt `(4, -5)` voisine (3 points) l'est — MKUE
  p. 7, 4.a ; spec 014 (Q2).
- **C9** — Au tour 5, quand il explore `2B` depuis `(4, -6)` (emplacement adjacent aux tuiles 1 et 3), alors la tuile de campagne 4
  est révélée en `2B`, et la Montagne `(3, -6)` voisine de la figurine n'est pas un `MoveHero` légal — MKUE p. 7, 3.a, 5.c et 5.e ;
  specs 014, 015.
- **C10** — À la première fin de tour où le deck ne peut plus remplir sa main (pendant les tours 2 à 5, ou ensuite, en jouant
  toute sa main de côté sans défausser) : la fin de tour pioche **toutes les cartes restantes** (moins que nécessaire), le deck est
  vide, la défausse n'est pas remélangée (elle a toutes les cartes qui ne sont pas en main), et la main a moins de 5 cartes — MKUE p. 11, 7.c et 7.d ; spec 016. La suite (fin
  de manche) est le parcours 028.

## Cas limites et refus

- **L1** — Le parcours ne dépend pas de l'ordre des actions légales (comme la 028, L3).
- **L2** — Une graine sans carte de Mouvement 2 dans la main de départ fait échouer le test avec « graine à rechoisir », avant C2.
- **L3** — Le plateau de jeu traversé ne dépend pas de la graine : si une case n'a pas le terrain attendu (tableau ci-dessus), le test
  échoue en nommant la case, ce qui signale un changement de la mise en place, pas de la graine.

## Information cachée

Les points d'étape se lisent sur la **vue du joueur** : main, taille du deck (pas son ordre), zone de jeu, défausse, points, figurine,
plateau de jeu, pioche de tuiles par dos et prochaine tuile (pas l'identité des tuiles de la pioche, qui ne se lit que dans le
`TileRevealed`).

## Libellés (i18n)

Aucun.

## Interface

Aucune.

### Raccourcis clavier

Aucun.

## Hors périmètre

- **Nuit, mana, retour en arrière, sauvegarde, fin de manche** : J2, parcours 028.
- **Exploration avec une tuile royaume ou une tuile écartée** (MKUE p. 7, 5.e) : il faudrait révéler les 6 tuiles de campagne de la
  pioche ; déjà testé par la spec 015.
- **Improvisation et Influence** : déjà testées par la spec 013 ; le parcours ne joue que du Mouvement.
- **L'écran** (cases atteignables, tracé, clic qui suit le trajet) : vérifié à l'œil par la spec 017 ; pas de test egui (ADR 0006).

## Questions ouvertes

Aucune. Décisions de Mathieu (2026-10-27) :

- **Q1 — Que fait le parcours en plus de la 028 ?** Rien de ce que la 028 vérifie déjà (mana, retour en arrière, manches) ; seulement
  ce que le J1 apporte et que la 028 traverse sans le vérifier.
- **Q2 — Mettre en commun les aides des deux parcours ?** Oui : `sideways_move`, `walk_to`, `has` et `end_turn` sont dans
  `support/`.
