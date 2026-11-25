# #9 — Illustrations en bande

- Statut : validée (2026-11-25, Q1 à Q5 tranchées par Mathieu selon les propositions)
- Issue : #9 (milestone v0.1.0) ; reprise de l'issue privée 280, second volet de l'issue privée 180
- Dépend de : spec 072 (illustrations générées, script), spec 077 (bande dans le panneau du bas), spec #32 (bande dans les
  fenêtres Offres et de choix ; implémentation en cours, à fusionner avant le code de cette spec) ; ADR 0009
- Livret : aucune règle
- Crates touchées : arcanum-ui (recadrage de la bande) ; `assets/images/card-art/` et `unit-art/` (PNG refaits ou
  nouveaux) ; `scripts/illustrations.py` et `scripts/illustrations/` (format, modèle, génération différée, descriptions).
  Rien dans arcanum-core ni dans `assets/i18n/`.
- Essais : [planche](images/issue-9-illustrations-en-bande/planche-essais.png), 5 images générées le 2026-11-25 (Marche,
  Boule de feu, Instinct), comparées aux images carrées recadrées d'aujourd'hui ; candidats bruts dans les fichiers du
  projet (`illustrations/issue-9-essais/`).

## Objectif

Toutes les cartes et les unités ont une illustration faite pour la bande où le jeu l'affiche (202 × 84 points, sous le
nom) : une scène large, un sujet simple et centré, lisible en petit, au lieu d'une image carrée dont la bande garde 42 % de
la hauteur. Les actions avancées, les sorts et les artefacts, sans image aujourd'hui, en reçoivent une dans le même style.

## Règles

Aucune règle du jeu. Les règles de droits de la spec 072 (§ Règles, « Droits ») et de l'ADR 0009 valent sans changement :
pas de nom du jeu, de ses héros ni de ses ennemis dans les descriptions, aucun scan donné au générateur, image rejetée si
elle contient du texte ou reproduit une illustration officielle. Les descriptions des nouvelles cartes partent du nom et de
l'effet de la carte (`../mage-knight/docs/regles/cartes/actions-avancees.md`, `sorts.md`, `artefacts.md`), pas de la
composition de son illustration.

## Modèle

Terme du glossaire modifié : **illustration générée** (une scène en bande, sauf la Blessure qui reste carrée). Terme
ajouté (section « Illustrations générées ») : **génération différée** (appels groupés en un lot que l'API traite dans la
journée, à moitié prix ; *Batch API*). Le terme **lot** (images validées ensemble) ne change pas : une génération
différée produit les candidats d'un lot.

### Format

| Famille | Images | Rapport | PNG retenu |
|---|---|---|---|
| Unité (`unit-art`) | 20, toutes refaites | 2,4:1, la bande | 404 × 168, palette de 256 couleurs |
| Carte (`card-art`) | actions de base (20, refaites), actions avancées (28), sorts (20), artefacts (16) | 2,4:1 | 404 × 168, palette |
| Carte : Blessure | inchangée (goutte de sang, affichée en grand, spec 077) | carré | 256 × 256, palette, comme aujourd'hui |

- **404 × 168** : deux fois la bande de 202 × 84 (net sur un écran à haute densité) et à peu près le nombre de pixels du
  256 × 256 d'aujourd'hui, donc le même poids (environ 50 Ko).
- Le générateur ne connaît pas 2,4:1 ; il reçoit **21:9** (2,33:1, 1 584 × 672 constaté) et le script recadre au centre
  à 2,4:1 (3 % de la hauteur perdue), retire un cadre uni s'il y en a un (comme aujourd'hui), puis réduit.
- Les 104 images refaites ou nouvelles remplacent les fichiers actuels sous le même nom (`assets/images/<famille>/<id>.png`,
  spec 072 C1) ; les anciennes restent dans l'historique git.

### Style et modèle (`scripts/illustrations/style.json`)

- **Modèle** : `gemini-3.1-flash-lite-image` pour toutes les images, cartes de héros comprises (Q1). Aux essais, il ne pose
  ni cadre ni bandes noires (2.5 flash en a mis sur deux images sur deux) et garde Tovak reconnaissable avec son portrait de
  référence, comme le faisait `gemini-3-pro-image-preview` à quatre fois le prix. `hero_model` disparaît : le portrait de
  référence (`scripts/illustrations/heroes/`, spec 072) est toujours donné avec une carte de héros, au même modèle.
- **Préambule** : inchangé (style peint, sans texte ni cadre).
- **Variante carte et unité**, remplacée par celle des essais : « A wide panoramic banner illustration, about 2.4 times
  wider than tall, filling the whole image edge to edge. One clear subject, large and centered, fully inside the central
  area, readable at a small size; simple, uncluttered painted background; nothing important near the top or bottom edges. »
  La Blessure garde l'ancienne variante carrée si elle devait être refaite (elle ne l'est pas ici).
- **Descriptions** (`cards.json`, `units.json`) : les 40 existantes sont réécrites pour un sujet unique et lisible en
  petit (une unité : deux ou trois personnages au premier plan plutôt qu'une foule) ; 64 nouvelles pour les actions
  avancées, sorts et artefacts. Un sort se dessine comme son effet de base, l'effet renforcé n'a pas d'image à part (une
  carte, une image).

### Le script (`scripts/illustrations.py`)

Commandes de la spec 072 conservées, avec ces changements :

| Commande | Changement |
|---|---|
| `generer <famille> [<id>…]` | Sans identifiant, prend chaque identifiant qui a une description et dont l'image retenue manque **ou n'est pas au format de la famille** (404 × 168 ; 256 × 256 pour la Blessure). Candidats écrits au format retenu, sans palette. Option `--differe` : envoie les appels en une **génération différée** au lieu d'appels immédiats, annonce le coût au tarif différé, note le lot en attente dans `<sortie>/differes.json` et rend la main. |
| `recuperer` (nouvelle) | Pour chaque lot en attente de `differes.json` : s'il est terminé, écrit ses candidats comme `generer` (même nommage `<id>-<n>.png`) et le retire de la liste ; sinon dit son état. Une requête du lot sans image est signalée, pas reprise (comme aujourd'hui, issue privée 170). |
| `planche <famille>` | Candidats montrés à la taille de la bande ×2 (404 × 168), identifiant et numéro au-dessus. |
| `retenir <famille> <id> <n>` | Format de la famille (404 × 168 ou 256 × 256), palette de 256 couleurs, modèle et date dans le JSON. |

`style.json` porte les deux tarifs (`usd_per_image` immédiat, `usd_per_image_deferred`), pour l'annonce du coût.

## Actions et événements

Aucun.

## Comportements attendus

Tests dans `crates/arcanum-ui/tests/illustrations.rs` (fichiers) et à côté de `layout.rs` (recadrage).

**Recadrage de la bande** : `band_uv` (spec 077 C2) prend aussi la taille de l'image, lue sur la texture chargée
(`SizedTexture::size`), et garde la plus grande part centrée qui a le rapport de la bande, en hauteur ou en largeur.

- **C1** — Image de 404 × 168 dans une bande de 202 × 84 : gardée entière, de 0 à 1 dans les deux sens
  (`band_keeps_whole_banner_image`).
- **C2** — Image carrée de 256 × 256 dans une bande de 202 × 84 : comme aujourd'hui (spec 077 C2, test
  `band_uv` de `layout.rs`), de 0,292 à 0,708 en hauteur, entière en largeur (`band_crops_square_image_as_before`).
- **C3** — Image de 404 × 168 dans un carré de 64 × 64 (vignette qui resterait après la spec #32) : entière en hauteur,
  de 0,292 à 0,708 en largeur (168 / 404 = 0,416 de la largeur, centrée) (`square_box_crops_banner_sides`).
- **C4** — Bande ou image de taille nulle : rien de recadré, de 0 à 1 (`band_uv_degenerate_sizes`), comme la garde
  `width <= 0.0` d'aujourd'hui.

**Fichiers** (révise la spec 072 C2) :

- **C5** — Chaque PNG de `unit-art/` et `card-art/` se décode, a une palette de 256 couleurs au plus et mesure 404 × 168,
  sauf `card-art/wound.png` en 256 × 256 (`every_illustration_is_known_and_valid`, modifié).
- **C6** — Chaque unité et chaque carte du contenu (20 unités ; 20 actions de base, Blessure, 28 actions avancées,
  20 sorts, 16 artefacts) a son PNG et son entrée dans `<famille>.json` (`every_unit_and_card_has_its_illustration`) : le
  lot est complet à la fusion. Les cas C3 de la spec 072 (description, modèle, date) valent pour toutes.

**Écran** (vérification à l'œil, ADR 0006), sur les parties préparées de la spec 077 et de la spec #32 :

- **C7** — Main (`main-blessures`) : Marche et Rage montrent toute leur scène dans la bande, sujet au centre ; la Blessure
  est inchangée.
- **C8** — Unités du panneau du bas et sous la main en combat : la troupe se lit dans la bande.
- **C9** — Fenêtre Offres et fenêtres de choix (`tour-conquise`, `monastere`, `sombre-terrier`) : actions avancées, sorts
  et artefacts ont leur bande, comme les unités.

## Cas limites et refus

- **L1** — Pendant la génération, une image encore carrée (lot pas encore refait) s'affiche recadrée comme aujourd'hui
  (C2) : la branche reste jouable entre deux lots. C5 et C6 ne passent qu'une fois tout refait ; la PR ne se fusionne
  qu'alors.
- **L2** — Candidat avec un cadre uni ou des bandes noires : retiré par le script avant le recadrage (`trim_border`, qui traite
  déjà le clair et le sombre) ; s'il en reste à l'œil, candidat rejeté.
- **L3** — Génération différée pas terminée au bout de 24 h, ou échouée : `recuperer` le dit ; on relance en appels
  immédiats les identifiants manquants (plafond `--max` inchangé).
- **L4** — Lot différé dont une requête échoue : les autres candidats sont écrits ; l'identifiant manquant repasse au
  `generer` suivant (il n'a toujours pas d'image au format).

## Information cachée

Rien : aucune donnée nouvelle dans la vue.

## Libellés (i18n)

Aucun.

## Interface

Aucun changement de disposition : bande de 202 × 84 sous le nom (spec 077, et spec #32 dans les fenêtres), Blessure en
grand. Seule l'image change.

### Raccourcis clavier

Aucun.

## Génération et coût

Tarifs de l'API Gemini (page des prix, lue le 2026-11-25), par image en 1K :

| Modèle | Immédiat | Différé |
|---|---|---|
| `gemini-3.1-flash-lite-image` (proposé) | 0,034 $ | 0,017 $ |
| `gemini-2.5-flash-image` (spec 072) | 0,039 $ | 0,020 $ |
| `gemini-3.1-flash-image`, en 0,5K | 0,045 $ | 0,022 $ |
| `gemini-3-pro-image` (cartes de héros, spec 072) | 0,134 $ | 0,067 $ |

Les essais de la rédaction ont coûté environ 0,18 $ (5 images). Le reste se fait en génération différée, **deux candidats
par image** (Q2) :

| Lot (ordre de génération) | Images | Candidats | Coût différé |
|---|---|---|---|
| Unités | 20 | 40 | 0,67 $ |
| Actions de base (dont 8 cartes de héros) | 20 | 40 | 0,67 $ |
| Actions avancées | 28 | 56 | 0,94 $ |
| Sorts | 20 | 40 | 0,67 $ |
| Artefacts | 16 | 32 | 0,54 $ |
| **Total** | **104** | **208** | **3,5 $** |

Avec une reprise (deux candidats de plus) pour un tiers des images, environ 4,7 $, soit quelque 4,1 € sur les 5,70 €
rechargés. Un candidat par image diviserait par deux (1,75 $, 2,4 $ avec reprises), au prix d'un choix plus pauvre. En
appels immédiats, tout coûte le double.

**Découpage proposé de l'implémentation**, en deux fils successifs sur la même branche (issue privée 118), après la fusion
de la spec #32 :

1. Script (format, modèle, `--differe`, `recuperer`, planche) et recadrage `band_uv` avec ses tests (C1 à C4) ; les
   descriptions des cinq lots, relues par Mathieu avant tout appel ; envoi des cinq générations différées.
2. Récupération, une planche par lot à valider par Mathieu, reprises, `retenir`, C5 et C6 au vert, vérification à l'œil
   (C7 à C9), docs.

## Docs mises à jour par la PR d'implémentation

- ADR 0009 : révision « 404 × 168 en bande, 2,4:1 (spec #9) ; Blessure en 256 × 256 », qui l'emporte sur les formats
  précédents.
- Spec 072 : C2 renvoie à cette spec (format) ; § « Le script » renvoie à cette spec pour `--differe` et `recuperer`.
- `docs/glossaire.md` : « Illustration générée », « Génération différée ».
- `docs/architecture.md` : `band_uv` (taille de l'image) et `illustrations.rs` (format retenu).

## Hors périmètre

- **Portraits des héros** (`hero-art`, écran de nouvelle partie, spec 078) : inchangés ; ils servent de référence.
- **Héros, ennemis et plateau** : SVG (ADR 0009, révision du 2026-11-06).
- **Blessure** : inchangée (format carré, affichée en grand).
- **Compétences et tactiques** : icônes SVG.
- **Disposition des cartes** : specs 077 et #32.

## Questions ouvertes

Aucune.

## Questions tranchées

Par Mathieu le 2026-11-25, toutes selon la proposition.

- **Q1. Modèle** : `gemini-3.1-flash-lite-image` pour toutes les images, cartes de héros comprises (avec leur portrait de
  référence).
- **Q2. Candidats** : deux par image (environ 3,5 $, 4,7 $ avec reprises).
- **Q3. Génération différée** : pour tous les lots (moitié prix, résultat dans la journée).
- **Q4. Descriptions** : relues par Mathieu avant tout appel, une liste par lot postée dans le fil, une ligne par carte.
- **Q5. Blessure** : gardée telle quelle (carrée, 256 × 256).
