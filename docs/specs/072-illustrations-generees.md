# 072 — Illustrations générées

- Statut : validée (2026-11-06 ; Q1 à Q6 tranchées par Mathieu le même jour)
- Jalon : IHM (refonte de l'interface, entre J4 et J5)
- Dépend de : 071 (icônes, infobulles), 050 (fenêtre Offres), 042 (unités), 041 et 043 (choix de l'artefact et du sort),
  011 (images et repli) ; toutes implémentées
- Livret : aucune règle nouvelle
- Crates touchées : arcanum-ui ; `assets/images/` (PNG nouveaux) ; `scripts/` (script de génération et descriptions) ;
  `Cargo.toml` (fonction PNG d'`egui_extras`, ADR 0009 Q2). Rien dans arcanum-core ni dans `assets/i18n/`.
- Base : [ADR 0009](../decisions/0009-illustrations-generees.md), révisé par cette spec (Q5 : héros et monstres restent en
  SVG) ; [synthèse de la refonte](refonte-ihm.md), section « Disposition générale »
- Maquettes : [`images/072-illustrations/`](images/072-illustrations/), montages sur des captures du jeu actuel ; essais de
  style dans [`images/072-illustrations/essais/`](images/072-illustrations/essais/)

## Objectif

Troisième lot de la refonte : les cartes et les unités ont une vraie illustration. Un script de `scripts/` génère les images
dans un style commun, et Mathieu les valide par lots. Le jeu les affiche en vignette sur chaque carte et chaque unité. Une image absente ne change rien : l'écran reste celui d'aujourd'hui.

## Règles

Aucune règle n'est ajoutée ni changée : le moteur propose les mêmes actions légales, l'écran gagne des images.

- **Droits** (`CLAUDE.md`, « Assets et droits » ; ADR 0009, « Rien d'officiel, mais reconnaissable ») : les descriptions ne
  nomment ni le jeu, ni son éditeur, ni ses héros ou ennemis ; elles s'appuient sur les scans de `docs/regles/scans/` (sujet,
  couleurs, équipement, ambiance) sans en décrire la composition, et aucun scan n'est donné au générateur. Une image qui
  reproduit une illustration officielle, ou qui contient du texte, est rejetée.

## Modèle

Rien de nouveau dans l'état ni dans la vue (arcanum-core). Termes ajoutés au glossaire par cette PR (section « Illustrations
générées ») : illustration générée, famille d'illustrations, vignette, préambule de style, lot, planche.

### Familles et fichiers

Une **illustration** (`Illustration`, arcanum-ui) est un couple famille + identifiant. Son fichier est
`assets/images/<famille>/<identifiant>.png`. L'identifiant est celui de la clé de libellé de l'élément, comme pour les SVG
des jetons (`enemy_image`, spec 030) : `unit.peasants.name` → `peasants`, `card.battle_versatility.name` →
`battle_versatility`, `card.wound.name` → `wound`. Les clés étant uniques, deux éléments n'ont jamais le même fichier.

| Famille | Dossier | Éléments | Image | Taille du PNG |
|---|---|---|---|---|
| Unité | `unit-art` | `Unit` (20) | scène carrée, opaque | 256 × 256, palette |
| Carte | `card-art` | `DeedCard` : actions de base (20), Blessure, actions avancées (28), sorts (20), artefacts (16) | scène carrée, opaque | 256 × 256, palette |

Format carré (Q4) : la vignette occupe l'espace libre sous les boutons « jouer de côté » (spec 071) en haut à droite de la
carte. 512 px (ADR 0009 Q3) : bien plus que la vignette de 64 px, mais de quoi servir les cartes plus grandes de la
spec 073 sans régénérer. Images opaques : pas de détourage. Révisé le 2026-11-11 (issue privée 167, ADR 0009) : 256 × 256 et
palette de 256 couleurs (PNG à palette), dix fois plus léger, toujours quatre fois la vignette.

### Descriptions et style, versionnés à côté des images

Les descriptions servent au script, pas au jeu : elles vivent dans `scripts/illustrations/`, pas dans `assets/` (qui est
livré avec le jeu).

- `scripts/illustrations/style.json` : le **préambule de style** commun, une **variante par famille** et le modèle du
  générateur.
- `scripts/illustrations/<famille>.json` (`units.json`, `cards.json`) : une entrée par identifiant, avec sa **description**
  et, une fois l'image retenue, le **modèle** et la **date** de l'image versionnée (ADR 0009, « L'image versionnée fait
  foi »).

Descriptions et préambule sont écrits en anglais, la langue que le générateur comprend le mieux ; ce sont des données, pas
des textes affichés (ADR 0003 ne s'applique pas). Préambule, variante et descriptions des essais, ceux qui ont donné la
[planche de style](images/072-illustrations/planche-style.png) (modèle `gemini-2.5-flash-image`, format 1:1, 2026-11-06) :

- **Préambule** : « Stylized hand-painted fantasy game illustration, clean dark outlines, rich saturated colors, soft cel
  shading, warm light from the upper left, high detail, consistent with a colorful tabletop fantasy board game. Absolutely
  no text, no letters, no numbers, no logos, no frame, no border. »
- **Variante carte et unité** : « A square scene illustration filling the whole image edge to edge, dramatic composition,
  painted background. » Une unité est décrite comme une troupe de plusieurs personnages du même genre.
- **Descriptions** (units.json `peasants`, cards.json `rage` et `march`) :
  - Paysans (units.ron, Peasants) : « Angry medieval peasants in hooded, ragged brown and grey clothes, armed with a
    pitchfork, a wooden club and a scythe, rough unshaven faces, determined but poorly equipped. »
  - Rage (basic_actions.ron, Rage) : « A muscular bare-chested warrior screaming in uncontrolled battle fury, veins
    standing out, an aura of red and orange flames of anger around him, dark smoky background lit in red. »
  - Marche (basic_actions.ron, March) : « A lone traveller in a hooded green cloak striding briskly along a winding road
    through green hills and forest at dawn, walking staff in hand, sense of speed and determination, soft green light. »

Le prompt envoyé est : préambule, puis variante de la famille, puis « Subject: » et la description ; le format demandé au
générateur est 1:1.

### Le script

`scripts/illustrations.py` (Python 3 et Pillow, outil de développement lancé depuis un fil cloud, Q1). Il lit la clé dans le
secret d'environnement `GEMINI_API_KEY`, jamais versionnée ; sans clé, il s'arrête avec un message et n'appelle rien.

Candidats et planches vont dans le dossier de sortie : option `--sortie <dossier>` (avant la commande), sinon variable
`ILLUSTRATIONS_SORTIE`, sinon `target/illustrations/`. Ce dernier disparaît avec le conteneur d'un fil cloud : depuis un fil,
viser les fichiers du projet (`--sortie /mnt/project-files/illustrations`), pour que les images payées survivent à un
redémarrage et qu'un fil suivant puisse les retenir (issue privée 171).

| Commande | Effet |
|---|---|
| `essai <famille> <id>` | Affiche le prompt complet, sans appel ni coût. |
| `generer <famille> [<id>…] [--candidats N] [--max M]` | Pour chaque identifiant donné, ou à défaut chaque identifiant de la famille qui a une description et pas d'image retenue : appelle l'API N fois (1 par défaut), au plus M appels en tout (20 par défaut), reprises comprises. Un appel qui échoue sur une erreur réseau ou une 5xx est repris jusqu'à 3 essais, après 10 puis 20 s ; jamais sur une 4xx ni sur une réponse sans image (issue privée 170). Annonce d'abord le nombre d'appels et leur coût estimé. Écrit chaque candidat, réduit à 256 × 256 (512 avant l'issue privée 167), dans `<sortie>/<famille>/<id>-<n>.png` (non versionné). |
| `planche <famille>` | Monte les candidats de la famille en une planche (identifiant et numéro sous chacun), à poster dans le fil pour que Mathieu choisisse. |
| `retenir <famille> <id> <n>` | Copie le candidat n, réduit à une palette de 256 couleurs (issue privée 167), dans `assets/images/<famille>/<id>.png` et inscrit le modèle et la date dans `<famille>.json`. |

## Actions et événements

Aucune action ni aucun événement nouveau.

## Comportements attendus

Tests dans `crates/arcanum-ui/tests/illustrations.rs`, sauf mention.

Fichiers :

- **C1** — L'identifiant d'un élément est celui de sa clé de libellé : Paysans (`Unit::Peasants`) →
  `unit-art/peasants.png`, Polyvalence (`BasicAction::BattleVersatility`) → `card-art/battle_versatility.png`, Blessure →
  `card-art/wound.png` (`illustration_path_follows_label_key`).
- **C2** — Chaque PNG de `assets/images/unit-art/` et `card-art/` porte l'identifiant d'un élément de sa famille, se décode
  mesure 256 × 256 et a une palette de 256 couleurs au plus (`every_illustration_is_known_and_valid`) — ADR 0009,
  révision du 2026-11-11 (issue privée 167).
- **C3** — Chaque PNG a son entrée dans `scripts/illustrations/<famille>.json`, avec description, modèle et date ; chaque
  entrée porte un identifiant connu (`every_illustration_has_its_description`).
- **C4** — L'index des illustrations, lu une fois au lancement dans le dossier des images, sait lesquelles sont présentes :
  avec un dossier de test qui contient `card-art/rage.png` seul, Rage a son image, Marche (`March`) non
  (`illustrations_index_lists_present_files`).

Écran (chaque élément porte son illustration ; c'est l'application qui la dessine si l'index la connaît) :

- **C5** — Une carte de la main porte l'illustration de sa carte : Rage → `card-art/rage` ; une Blessure →
  `card-art/wound` (`hand_card_has_its_illustration`). Valable pour la main du panneau et celle de l'écran de combat, qui
  partagent `HandCard`.
- **C6** — Une carte de la fenêtre Offres (action avancée, sort, action avancée d'un monastère) et des fenêtres de choix de
  l'artefact et du sort porte l'illustration de sa carte ; une unité de l'offre porte celle de son unité
  (`offer_card_has_its_illustration`) — `OfferCardView`, specs 041, 043, 050.
- **C7** — Une ligne d'unité, dans le panneau et sous la main en combat, porte l'illustration de son unité : Paysans →
  `unit-art/peasants` (`unit_line_has_its_illustration`).

Affichage (vérification à l'œil, ADR 0006) :

- **C8** — Image présente : la vignette s'affiche à la taille et à la place du § Interface, sans infobulle (maquettes).
- **C9** — Image absente (lot pas encore généré) : l'écran est celui d'aujourd'hui, sans place réservée, et rien n'est écrit
  dans les logs (ADR 0009, « Une image manquante n'empêche pas de jouer »). Les maquettes le montrent pour Promesse et
  Concentration.

## Cas limites et refus

- **L1** — PNG présent mais illisible : repli comme en C9, et un avertissement dans les logs une seule fois par fichier,
  comme pour un SVG aujourd'hui (`image_paint`, `ImageWarnings`, spec 011 ; test
  `missing_image_falls_back_and_warns_once`).
- **L2** — Dossier des images absent (`ARCANUM_ASSETS_DIR` incomplet) : l'index est vide, le jeu se joue sans image
  (`illustrations_index_without_directory_is_empty`).
- **L3** — Fichier inconnu dans un dossier d'illustrations (`card-art/not_a_card.png`, aucune carte de ce nom ;
  `fireball`, l'exemple de la rédaction, est un sort du contenu) : refusé par C2 ; en jeu, il est ignoré
  (`illustrations_index_ignores_unknown_files`).

## Information cachée

Rien de nouveau dans la vue : chaque illustration se déduit d'un identifiant que l'écran a déjà : `PlayerView::hand` (C5),
`offers` et `drawn_artifacts` (C6), `units` (C7).

## Libellés (i18n)

Aucun : la vignette n'a pas d'infobulle (Q6).

## Interface

Tailles en points d'egui. Les images sont réduites avec lissage
(mipmaps) pour rester nettes en vignette : vérification à l'œil.

- Dans le panneau du bas (main, écran de combat, fenêtre de la défausse, unités), les vignettes ci-dessous sont remplacées
  par une bande sous le nom, et l'image en grand pour une Blessure : spec 077. Elles restent dans la fenêtre Offres et les
  fenêtres de choix (spec 086).
- **Vignette de carte** : 64 × 64, en haut à droite du cadre de la carte, sous l'encart « jouer de côté » s'il y en a un
  (spec 071), le texte et les boutons restant à sa gauche ; dans la main du panneau et de l'écran de combat (C5), dans la
  fenêtre Offres et les fenêtres de choix (C6). Maquettes
  [panneau-main](images/072-illustrations/panneau-main.png) et [combat](images/072-illustrations/combat.png) (Marche et Rage
  illustrées ; Promesse et Concentration sans image, comme aujourd'hui).
- **Vignette d'unité** : 64 × 64, à gauche de la ligne de l'unité, dans le panneau et sous la main en combat (C7) ; dans la
  fenêtre Offres, en haut à droite comme une carte (C6). Maquette
  [panneau-unite](images/072-illustrations/panneau-unite.png).
- **Chargement** : chaque PNG est lu au premier affichage par les chargeurs d'images d'egui, comme les SVG (spec 011), puis
  gardé en mémoire ; l'index (C4) évite de demander un fichier absent.

### Raccourcis clavier

Aucun : les images ne portent aucune action.

## Premier lot

41 images : les 20 unités, les 20 actions de base et la Blessure (Q2). Les actions avancées, sorts et artefacts suivent par
lots, sans nouvelle spec (ADR 0009) : l'affichage les prend en charge dès que leur PNG est là. Mathieu valide chaque lot sur
ses planches avant la fusion. Les trois essais (Paysans, Rage, Marche) peuvent être retenus tels quels.

Coût estimé avec `gemini-2.5-flash-image` : 1 290 jetons de sortie par image, soit environ 0,04 $ ; le lot fait 1,6 $ en
une passe, vers 2,2 $ avec une reprise sur un tiers des images. Crédit de 5 € sur le projet Google (Mathieu, 2026-11-06) ;
les essais de cette spec ont coûté environ 0,35 $.

**Découpage proposé de l'implémentation**, en deux fils successifs sur la même branche (issue privée 118) : d'abord le script, le
code d'affichage et ses tests, vérifiés à l'œil avec les trois essais ; puis la génération du lot, famille par famille, avec
une planche à valider à chaque fois.

**Réalisé** (2026-11-06) : 20 unités, 20 actions de base et la Blessure, validées par Mathieu sur planches. Deux ajustements
demandés à la validation : la Blessure est une simple goutte de sang ; les huit cartes de remplacement des héros montrent
le héros reconnaissable d'une carte à l'autre. Pour cela, un **portrait de référence** par héros (famille `heroes`,
`scripts/illustrations/heroes/`, jamais affiché) est donné au générateur avec chaque carte de héros (champ `hero` de
`cards.json`), et ces 12 images utilisent le modèle plus puissant `gemini-3-pro-image-preview` (`hero_model` de `style.json`).
Le script retire aussi le cadre uni que le générateur ajoute parfois. Coût total d'environ 3,7 $.

## Hors périmètre

- **Héros et monstres** : restent en SVG, à rendre plus soignés avec le plateau isométrique (spec 075) ; pas d'image
  générée (Q5).
- **Disposition des cartes et des unités** (panneau du bas, chevauchement, base et renforcé séparés) : spec 073. La vignette
  de cette spec se placera dans la nouvelle carte.
- **Compétences et tactiques** : icônes SVG (ADR 0009 Q1), avec leurs specs (051, 060).
- **Zone de jeu** (cartes jouées) et fenêtre de fin : pas de vignette ; la zone de jeu est refaite par la spec 073.
- **Génération depuis le jeu** : jamais (ADR 0009) ; le jeu ne fait que lire les PNG.

## Questions ouvertes

Aucune.

## Questions tranchées

Par Mathieu le 2026-11-06.

- **Q1. Langage du script** : Python 3 et Pillow, déjà présents dans le conteneur cloud, sans effet sur la compilation du
  jeu ni sur Windows (proposition acceptée).
- **Q2. Budget** : crédit de 5 € ajouté, suffisant pour le premier lot.
- **Q3. Placement** : vignettes comme sur les maquettes (proposition acceptée).
- **Q4. Format** : carré plutôt que portrait, pour occuper l'espace vide sous les boutons « jouer de côté » (Mathieu).
- **Q5. Héros et monstres** : une image générée n'apporte pas assez par rapport au SVG s'il ne sert qu'au survol ou en
  combat ; ils restent en SVG, à rendre plus soignés. Les images générées sont réservées aux cartes (unités, actions, sorts,
  artefacts). L'ADR 0009 est révisé dans cette PR.
- **Q6. Survol** : pas d'image en grand au survol de la vignette (Mathieu). Levée par la spec 077 : l'image s'agrandit en
  bande dans le panneau du bas, toujours sans survol.
