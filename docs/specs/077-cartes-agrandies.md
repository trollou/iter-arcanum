# 077 — Cartes agrandies

- Statut : implémentée (validée le 2026-11-13, Q1 à Q4 tranchées par Mathieu ; reporter tout changement dans la colonne Statut de `PLAN.md`)
- Jalon : J5 (spec d'interface placée après les artefacts, `PLAN.md`)
- Dépend de : 072 (illustrations, vignettes), 073 (panneau du bas, cartes posées) ; toutes implémentées
- Livret : aucune règle
- Crates touchées : arcanum-ui seulement. Rien dans arcanum-core, ni dans `assets/i18n/`, ni dans `assets/images/`.
- Base : issue privée 180 et ses commentaires (décision A de Mathieu : agrandir l'image dans le panneau, quitte à élargir
  légèrement les cartes ; la Blessure montre son image en grand). Lève la Q6 de la spec 072 et la ligne « Vignette seule »
  de l'ADR 0009. Disposition choisie sur maquettes par Mathieu (2026-11-13, Q1 à Q3 tranchées).
- Maquettes : [`images/077-cartes-agrandies/`](images/077-cartes-agrandies/), captures du jeu avec un essai de code jetable
  (non versionné) : [avant](images/077-cartes-agrandies/avant.png), [retenue](images/077-cartes-agrandies/bande.png)
  (bande, cartes de 216 × 240 ; la Blessure y déborde encore, réglage de l'essai : voir « Interface »),
  [carré à gauche](images/077-cartes-agrandies/carre.png) (écartée).

## Objectif

Les illustrations des cartes et des unités se voient vraiment : dans le panneau du bas, l'image n'est plus une vignette de
48 points mais une **bande sur toute la largeur de la carte**, sous le nom ; les cartes grandissent un peu pour qu'une carte
simple (Marche) reste entière. La Blessure, qui n'a pas de texte, montre son image en grand.

## Règles

Aucune règle du jeu : spec d'interface.

## Modèle

Aucun terme nouveau. Les illustrations sont celles de la spec 072 (`Illustration`, PNG de 256 × 256 dans
`assets/images/card-art/` et `unit-art/`) ; aujourd'hui, les 20 actions de base, la Blessure et les 20 unités en ont une. Les
actions avancées, sorts et artefacts suivront par lots (spec 072, « Premier lot ») : la disposition de cette spec les prend en
charge dès que leur PNG est là, et une carte sans image garde la disposition d'aujourd'hui (pas de bande, pas de place
réservée).

Les images carrées sont **recadrées au milieu** pour la bande (le haut et le bas sont coupés). Les refaire pour ce format
viendra ensuite : issue #9.

## Actions et événements

Aucun.

## Comportements attendus

Chaque cas devient au moins un test (nom indicatif entre parenthèses). Le dessin lui-même (bande, place du texte, image de
la Blessure) se vérifie à l'œil (ADR 0006) ; seuls la donnée de la vue et les calculs purs sont testés.

**Blessure**

- **C1** — Dans la main de la partie préparée `main-blessures` (Blessure ×2, Marche, Promesse, Rage), la carte de la vue
  d'une Blessure dit que son image occupe la carte (`HandCard::art_fills_card` vrai) ; celle de Marche (`DeedCard`
  `March`, illustrée) ne le dit pas (`wound_art_fills_card`). La Blessure n'a ni effet ni texte renforcé (spec 031), d'où
  la place libre.

**Recadrage de la bande**

Fonction pure de `layout.rs`, sans egui (`band_uv(largeur, hauteur) → (haut, bas)`) : la part de l'image carrée gardée
dans la bande, centrée verticalement, en coordonnées de texture (0 en haut, 1 en bas).

- **C2** — Une bande de 202 × 84 garde l'image de 0,292 à 0,708 (84 / 202 = 0,416 de sa hauteur, centrée) ; une bande
  carrée (100 × 100) la garde entière, de 0 à 1 ; une bande plus haute que large n'agrandit pas au-delà, de 0 à 1
  (`band_crops_middle_of_square_art`).

**Cartes plus grandes** (Q2)

`CARD_WIDTH` passe de 200 à 216 points et `CARD_HEIGHT` de 212 à 240. Les cas C9 à C12 bis de la spec 073 gardent leurs
situations, avec les nombres recalculés ; leurs tests sont **modifiés** (voir « Tests existants modifiés »).

- **C3** — Largeurs (073 C9) : 2 unités et 5 cartes dans 1 600 points tiennent à leur largeur naturelle (440 et 1 112) ;
  3 unités et 7 cartes dans 1 228 points : 400 et 828 (inchangé) ; dans 1 800 : 537 et 1 263 (inchangé) ; 1 unité et
  9 cartes dans 1 000 : 216 et 784 (`section_widths_share_space`).
- **C4** — Chevauchement (073 C10) : 7 cartes dans 600 points à 0, 64, 128, 192, 256, 320, 384 ; 5 cartes dans 1 112
  points à 0, 224, 448, 672, 896, sans chevauchement (`fan_layout_overlaps_when_short`).
- **C5** — Survol (073 C11) : la 4e de 7 dans 600 points place les cartes à 0, 34, 67, 101, 317, 350, 384 ; sans
  chevauchement (5 cartes dans 1 112), rien ne change (`fan_layout_hovered_card_is_whole`).
- **C6** — (073 C12) Survoler la dernière ne change rien ; 4 cartes dans 300 points, la 2e survolée : 0, 0, 84, 84
  (`fan_layout_last_and_too_narrow`). La carte sous le pointeur reste la même aux abscisses 10, 70, 599, 200 et 330
  (`card_under_pointer_is_the_top_one`, inchangé : ses nombres tombent juste avec 216).
- **C7** — Hauteur d'une carte survolée (073 C12 bis) : un contenu de 150 points donne 240 (jamais plus petite), un
  contenu de 300 points donne 300, plafonné à 260 si la fenêtre n'a que 260 points au-dessus du bas de la carte
  (`hovered_card_grows_to_its_content`).

### Tests existants modifiés

Dans `crates/arcanum-ui/tests/panneaux.rs`, les nombres de cinq tests dépendent de `CARD_WIDTH` ou `CARD_HEIGHT` et sont
recalculés (C3 à C7) : `section_widths_share_space`, `fan_layout_overlaps_when_short`, `fan_layout_hovered_card_is_whole`,
`fan_layout_last_and_too_narrow`, `hovered_card_grows_to_its_content`. Leurs commentaires (« 3 unités (616) et 7 cartes
(1 448) ») aussi. Aucun champ n'est retiré ni renommé.

## Cas limites et refus

- **L1** — Une carte ou une unité sans image (actions avancées, sorts, artefacts aujourd'hui) : pas de bande, le texte
  commence sous le nom, comme aujourd'hui (spec 072, C9 : aucune image demandée).
- **L2** — Une carte dont le contenu dépasse 240 points (actions avancées au texte long, unité avec bannière) : coupée et
  marquée « … », elle grandit au survol (spec 073, C12 bis, ici C7), sans autre changement.
- **L3** — Une Blessure en mode choix (défausse, Concentration) garde ses boutons sous l'image : l'image a un côté fixe
  (180 points) qui leur laisse la place. En fin du tour, sa case « Défausser » est dans l'en-tête, comme celle des autres
  cartes (issue privée 282).
- **L4** — L'image ne porte aucune action : la cliquer fait comme cliquer le fond de la carte (rien).

## Information cachée

Rien de nouveau : les cartes de la main et les unités sont déjà dans la vue. Champ ajouté à la vue de l'écran :
`HandCard::art_fills_card` (C1), vrai pour une Blessure ; l'écran ne relit pas l'identifiant de la carte pour le décider.

## Libellés (i18n)

Aucun.

## Interface

Tailles en points d'egui.

- **Carte de la main** (panneau du bas, écran de combat, fenêtre de la défausse, qui dessinent tous la carte par
  `card_tile`) : en-tête inchangé (pastille, nom, encart « jouer de côté ») ; dessous, l'image en **bande de 202 × 84**
  (toute la largeur intérieure de la carte), recadrée au milieu (C2), coins arrondis de 3 points ; puis l'effet de base et
  ses boutons, et le bloc renforcé, sur toute la largeur, comme aujourd'hui. Une carte simple (Marche, Promesse, Rage)
  s'affiche entière, boutons renforcés compris. Maquette [bande](images/077-cartes-agrandies/bande.png) (vérification à
  l'œil, `main-blessures`). Depuis la spec [#11](issue-11-cartes-en-main-epurees.md), le texte de l'effet n'est plus écrit
  (bouton « i » à droite du nom, sauf un artefact) et le bloc renforcé s'ouvre sur « —— pastille Renforcé —— ».
- **Unité** (panneau du bas, et sous la main au combat) : même bande sous l'en-tête ; armure, résistances et état dessous,
  sur toute la largeur ; bannière et boutons ensuite (vérification à l'œil, partie préparée `unites`).
- **Carte plus grande** : 216 × 240 au lieu de 200 × 212 (C3 à C7) ; le panneau du bas prend 28 points de hauteur de plus
  au plateau (vérification à l'œil).
- **Blessure** : sous son nom, son image seule, **180 × 180, centrée**, non recadrée (C1, L3 ; vérification à l'œil,
  `main-blessures`).
- **Pas d'image agrandie au survol** (Q3) : la bande suffit.
- **Fenêtre Offres et fenêtres de choix** (artefact, sort) : inchangées dans cette spec, vignette de 64 en haut à droite
  (spec 072, C6) ; présentées comme la main par la spec 086 (Q4).
- **Zone de jeu** : inchangée, les cartes jouées n'y sont que des noms.

### Raccourcis clavier

Aucun : l'image ne porte aucune action.

## Documentation à mettre à jour (PR d'implémentation)

- `docs/specs/072-illustrations-generees.md` : Q6 et « Interface » (64 et 48 points dans le panneau) renvoient à cette spec.
- `docs/decisions/0009-illustrations-generees.md` : la ligne « Vignette seule : 64 × 64 sur la carte ou l'unité » devient
  « bande recadrée dans le panneau du bas (spec 077), vignette de 64 dans les fenêtres ».
- `docs/specs/073-panneaux.md` : les tailles de 200 et 212 points et les nombres de C9 à C12 bis renvoient à cette spec
  (C3 à C7).
- `docs/architecture.md` si elle cite les tailles de carte. L'issue privée 180 est fermée par la PR d'implémentation (label
  `planifié` retiré).

## Hors périmètre

- **Illustrations refaites pour la bande**, plus simples et lisibles (second volet de l'issue privée 180) : issue #9, dans un
  second temps.
- **Image agrandie au survol** : abandonnée (Q3).
- **Images des cartes jouées** dans la zone de jeu.
- **Cartes des fenêtres** Offres et de choix au format de la main : spec 086, jalon Finitions (Q4).
- **Taille des cartes réglable** : toujours pas demandé (spec 073).

## Questions ouvertes

Tranchées par Mathieu sur maquettes (2026-11-13) :

- **Q1. Disposition de l'image** : bande sous le nom, « vraiment supérieure » au carré à gauche (Mathieu) ; recadrée au
  milieu en attendant des illustrations faites pour elle (issue #9).
- **Q2. Taille** : cartes de 216 × 240 et bande de 84 (au lieu de 92) ; objectif : une carte simple comme Marche s'affiche
  entière (Mathieu). Vérifié sur la maquette : Marche, Promesse et Rage tiennent.
- **Q3. Survol** : « oublie le survol, il ne sert plus à rien avec cette réorganisation » (Mathieu).
- **Q4. Fenêtre Offres et fenêtres de choix** : inchangées ici ; l'incohérence avec la main est traitée par la spec 086
  (jalon Finitions), maquette à l'appui, avec peut-être une taille de carte minimale (Mathieu).
