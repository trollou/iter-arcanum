# #32 — Cartes des fenêtres

- Statut : à valider
- Issue : #32 (Epic #31 Finitions v0.1.0, milestone v0.1.0)
- Dépend de : spec #11 (cartes en main épurées, PR #40, **à fusionner avant l'implémentation** : elle crée `CardInfo`,
  `strong_heading`, `strong_colors` et la clé `card.if_destroyed` que cette spec reprend), spec 077 (cartes agrandies),
  spec 072 (illustrations, C6), spec 050 (offres), spec 070 (C17, C18) ; toutes implémentées sauf #11 (codée, PR prête)
- Livret : aucune règle
- Crates touchées : arcanum-ui, assets/i18n. Rien dans arcanum-core.
- Base : issue #32 (reprise de l'issue privée 384) ; spec 077, Q4 (Mathieu, 2026-11-13 : « peut-être une taille de carte
  minimale, pour que la bande d'une colonne étroite ne devienne pas minuscule », maquette avant de trancher) ; spec #11, Hors
  périmètre.
- Maquettes : [`images/issue-32-cartes-des-fenetres/`](images/issue-32-cartes-des-fenetres/), captures du jeu (branche de la
  PR #40) avec un essai de code jetable, non versionné (patch dans les fichiers du projet,
  `ihm/issue-32-maquette-jetable.patch`) : fenêtre Offres [avant](images/issue-32-cartes-des-fenetres/avant-offres.png) /
  [après](images/issue-32-cartes-des-fenetres/maquette-offres.png) (`tour-conquise`, touche O) ;
  [avec l'action avancée d'un monastère](images/issue-32-cartes-des-fenetres/maquette-monastere.png) (`monastere`) ;
  [choix du sort](images/issue-32-cartes-des-fenetres/maquette-sort.png) (`tour-conquise`, fin du tour) ;
  [choix de l'artefact](images/issue-32-cartes-des-fenetres/maquette-artefact.png) (`sombre-terrier`, touche 1).

## Objectif

Dans la fenêtre Offres et dans les fenêtres de choix d'un artefact et d'un sort, chaque carte est **dessinée comme une
carte de la main** : même cadre, même largeur (216 points), pastille et nom en en-tête, bande d'illustration sous le nom
quand la carte en a une, puis le texte de base, la séparation « —— (pastille) Renforcé —— » (le nom renforcé d'un sort,
« si détruit » d'un artefact) et le texte renforcé. Les cartes d'une offre sont **posées en rangée**, une rangée par offre,
au lieu de s'empiler en colonnes de 300 points avec une vignette de 64 en haut à droite.

Aujourd'hui, seules les unités ont une illustration parmi les cartes de ces fenêtres (spec 077, Modèle) : la bande n'y
apparaît que pour elles. Les actions avancées, sorts et artefacts la prendront dès que leur PNG existera (issue #9 et lots
suivants de la spec 072), sans autre changement.

## Règles

Aucune règle du jeu : spec d'interface.

- **Mana du renforcé** (MKUE p. 4, 3.d ; sorts : p. 5, 2.a) : les pastilles de la séparation sont celles de la main
  (spec #11, C7) : la couleur de la carte pour une action avancée, la couleur puis le noir pour un sort, aucune pour un
  artefact.

## Modèle

Aucun terme nouveau. La vue d'une carte des fenêtres reste `OfferCardView` (`arcanum-ui/src/screen/offers.rs`), commune à
la fenêtre Offres (`OffersWindow::columns`), au choix de l'artefact (`GameScreen::artifact_choice`) et au choix du sort
(`GameScreen::spell_choice`). Elle change ainsi :

- **`name`** : le nom de base, aussi pour un sort. Aujourd'hui, un sort de la fenêtre Offres s'appelle « Éclair de feu /
  Tempête de feu » (`offers.spell_name`) ; son nom renforcé passe dans la séparation.
- **`color`** : la couleur de la carte, aussi pour un sort de la fenêtre du choix du sort (aujourd'hui `None`,
  `sites.rs::spell_choice`). Un artefact et une unité n'en ont pas.
- **`lines`** d'une action avancée, d'un sort ou d'un artefact : deux lignes, le texte de base et le texte renforcé, **sans
  le préfixe « Renforcé : »** (`card.strong`), porté par la séparation. Le texte est **celui du « i » de la même carte en
  main** (spec #11, `CardInfo::basic` et `CardInfo::strong` : le texte écrit quand la carte en a un, sinon le texte calculé),
  par la même fonction ; pour un artefact, qui n'a pas de « i », son texte écrit (`card_effect_texts` aujourd'hui) (Q2).
  Les `lines` d'une unité ne changent pas (spec 050, Q1).
- **`strong_heading: String`** (nouveau) : le titre de la séparation, comme `HandCard::strong_heading` (spec #11) :
  « Renforcé » (`panel.strong`), le nom renforcé d'un sort (`card.<id>.strong_name`), « si détruit » pour un artefact
  (`card.if_destroyed`) ; vide pour une unité.
- **`strong_colors: Vec<ManaColor>`** (nouveau) : ses pastilles, comme `HandCard::strong_colors` ; vide pour un artefact et
  une unité.

`take`, `mark`, `illustration` et `uses` ne changent pas. `OffersWindow::columns` garde son nom et son ordre (actions
avancées, sorts, unités) : seul le dessin passe des colonnes aux rangées.

Calcul pur ajouté à `layout.rs` (sans egui) : **`cards_per_row(width) → usize`**, le nombre de cartes de 216 points, écart
de 8 (`CARD_GAP`), qui tiennent côte à côte dans cette largeur, au moins 1 (`row_width`, spec 073).

## Actions et événements

Aucun.

## Comportements attendus

Chaque cas devient au moins un test (nom indicatif entre parenthèses). Le dessin (cadre, rangées, séparation, bande) se
vérifie à l'œil (ADR 0006) ; la vue et le calcul des rangées sont testés.

**Fenêtre Offres**

- **C1** — Magie pure (`AdvancedAction` `PureMagic`, bleue) dans l'offre d'actions avancées : `name` « Magie pure »,
  `color` bleu, `lines` [`card.pure_magic.basic_text`, `card.pure_magic.strong_text`] (« Payer un mana : vert Mouvement 4,
  … », « Payer un mana : vert Mouvement 7, … », sans « Renforcé : »), `strong_heading` « Renforcé », `strong_colors`
  [bleu] (`offer_advanced_action_reads_like_hand_card`).
- **C2** — Éclair de feu (`Spell` `Fireball`, rouge) dans l'offre de sorts : `name` « Éclair de feu » (et non plus
  « Éclair de feu / Tempête de feu »), `lines` [« Attaque à distance de feu 5 », « Blessure ; Attaque de siège de feu 8 »]
  (`card.fireball.basic_text`, `strong_text` : le texte écrit, comme le « i », C7), `strong_heading` « Tempête de feu »
  (`card.fireball.strong_name`), `strong_colors` [rouge, noir] (`offer_spell_names_its_strong_effect_apart`).
- **C3** — Une unité de l'offre (partie de `offres.rs`, première unité) : `strong_heading` vide, `strong_colors` vide,
  `lines` inchangées (régulière ou élite, coût, armure, résistances, lieux, capacités ; test
  `offers_window_lists_the_three_offers`, inchangé) (`offer_unit_has_no_strong_block`).
- **C4** — L'action avancée d'un monastère dans l'offre d'unités (Bouclier de glace, `IceShield`, bleue, partie de
  `villages_et_monasteres.rs`) : `mark` « monastère » gardé, `strong_heading` « Renforcé », `strong_colors` [bleu], comme
  C1 (`monastery_advanced_action_reads_like_hand_card`).

**Fenêtres de choix**

- **C5** — Choix du sort à la fin du tour où la tour de mage est conquise (`sites_fortifies.rs`, `choosing_spell` : Éclair de
  feu, Tempête de neige, Exposition) : la carte d'Éclair de feu est **la même vue** que dans la fenêtre Offres (C2) à
  `take` près : `color` rouge (aujourd'hui `None`), mêmes `lines`, `strong_heading` et `strong_colors`
  (`spell_choice_card_matches_offer_card`).
- **C6** — Choix de l'artefact (`villages_et_monasteres.rs`, Anneau de rubis et Amulette du soleil) : Anneau de rubis
  (`RubyRing`) a `color` vide, `lines` [« Jeton et cristal rouges ; Gloire +1 », « Mana rouge et noir illimité ce tour ;
  Gloire +1 par sort rouge lancé »], `strong_heading` « si détruit », `strong_colors` vide (spec #11, C5 et C7)
  (`artifact_choice_card_reads_like_hand_artifact`).

**Même texte qu'en main**

- **C7** — Pour Magie pure, Éclair de feu et Bouclier de glace, les `lines` de la carte de l'offre valent `info.basic` et
  `info.strong` de la même carte en main (`HandCard::info`, spec #11), et `strong_heading` vaut son
  `HandCard::strong_heading` (`offer_text_matches_hand_info`).

**Rangées**

- **C8** — `cards_per_row` : 900 points → 4 (`row_width(4)` = 888) ; 888 → 4 ; 887 → 3 ; 700 → 3 ; 216 → 1 ; 100 → 1 (une
  carte au moins, quitte à défiler) (`cards_per_row_fills_the_width`).

### Tests existants modifiés

Cherchés à la rédaction, là où c'est lu (`grep -rn '\.lines\b\|\.name\b\|spell_name' crates/arcanum-ui/tests` sur les
tests qui ouvrent `offers_window`, `artifact_choice` ou `spell_choice`) :

- `crates/arcanum-ui/tests/actions_avancees_mana.rs` : la fonction d'aide `spell_name` (format `offers.spell_name`, « Éclair
  de feu / Tempête de feu ») cherche la carte de l'offre par son nom ; elle devient le nom de base
  (`catalog.text(spell.label_key())`). Deux appels, dans `strong_magic_talent_chooses_spell_in_offers`.
- `crates/arcanum-ui/tests/villages_et_monasteres.rs` : `panel_offers_advanced_actions_at_monastery` compare l'infobulle de
  l'achat au monastère à `lines.join("\n")` de la carte de l'offre ; l'infobulle garde « Renforcé : … » (`sites.rs`, achat,
  hors périmètre), la carte ne l'a plus. L'assertion compare désormais l'infobulle aux textes de `card_effect_texts`.
- Sans changement attendu, vérifiés : `offres.rs` (`lines` d'un sort sans « pas encore codé » ; unité), `sites_fortifies.rs`
  `spell_choice_window` (noms de base, `lines` non vides), `villages_et_monasteres.rs` (artefact : la ligne de base, égale),
  `illustrations.rs` (illustration des cartes des fenêtres, spec 072 C6), `competences.rs` (fenêtre des compétences, pas
  `OfferCardView`). Aucun champ retiré ni renommé.

## Cas limites et refus

- **L1** — Offre plus large que la fenêtre (offre d'unités de 5 ou 6 cartes, actions avancées de monastère comprises, à
  1 280 points de large) : la rangée passe à la ligne (C8). La fenêtre ne dépasse pas l'écran : au plus sa largeur moins
  une marge.
- **L2** — Fenêtre plus haute que l'écran (1 280 × 800, taille de départ du jeu, ou rangées passées à la ligne) : son
  contenu défile verticalement, titre et boutons « Fermer » ou « Renoncer » restant visibles. En 1 600 × 900, la fenêtre
  Offres de `tour-conquise` et de `monastere` tient sans défiler (maquettes ; spec 070, C18).
- **L3** — Cartes d'une même rangée **de même hauteur**, celle de la plus haute (maquette) ; pas de hauteur fixe de 240
  (spec 077) : une carte de fenêtre ne se coupe pas et ne grandit pas au survol, elle montre tout son texte. La bande d'une
  carte sans image n'est pas réservée (spec 077, L1).
- **L4** — Les boutons (« Prendre », « Apprendre (6 Influence) », « Lancer », « Recruter (gratuit) », « Utiliser : … ») et
  leurs touches restent sous le texte de leur carte, sans autre changement (specs 050, 056, 057, 071 C22).
- **L5** — Pas de bouton « i » sur ces cartes : leur texte complet est écrit dessus.

## Information cachée

Rien de nouveau : les offres et les cartes proposées sont déjà dans la vue (`PlayerView::offers`, `drawn_artifacts`,
`spell_choices`). Champs ajoutés à la vue de l'écran : `OfferCardView::strong_heading`, `strong_colors` (Modèle).

## Libellés (i18n)

Aucune clé nouvelle.

Clés réutilisées, relues : `panel.strong` « Renforcé » / « Powered » (dans `main`) ; `card.<id>.strong_name`
(« Tempête de feu », dans `main`) ; `card.if_destroyed` « si détruit » / « if destroyed » (ajoutée par la PR #40, spec #11,
absente de `main` tant qu'elle n'est pas fusionnée).

Clé retirée : `offers.spell_name` (« {basic} / {strong} »), qui n'a plus de lecteur une fois le nom d'un sort réduit à son
nom de base (C2) ; seule `offers.rs` la lit dans `src`.

## Interface

- **Carte d'une fenêtre** : le cadre d'une carte de la main (`card_frame`, fond, bordure, coins), 216 points de large
  (`CARD_WIDTH`) ; en-tête : pastille de la couleur (`OfferCardView::color`), nom en gras, puis la marque « monastère »
  (`mark`) ; bande d'illustration de 202 × 84 recadrée au milieu (`art_band`, spec 077 C2 ; `illustration`) ; texte de
  base ; séparation et texte renforcé en grisé ; boutons (C1 à C6 ; vérification à l'œil, maquettes
  [Offres](images/issue-32-cartes-des-fenetres/maquette-offres.png)).
- **Séparation** : celle de la main (spec #11, `strong_separator`) : pour une action avancée, un trait, la pastille,
  « Renforcé », un trait ; pour un sort, un trait simple puis les pastilles couleur et noir et le nom renforcé en gras ;
  pour un artefact, « —— si détruit —— » sans pastille (C1, C2, C6, `strong_heading`, `strong_colors` ; vérification à
  l'œil, maquettes [sort](images/issue-32-cartes-des-fenetres/maquette-sort.png) et
  [artefact](images/issue-32-cartes-des-fenetres/maquette-artefact.png)). Le dessin de la main est partagé, pas recopié.
- **Unité** : en-tête (nom), bande, puis ses lignes, comme aujourd'hui (C3).
- **Fenêtre Offres** (Q1) : une rangée par offre, dans l'ordre Actions avancées, Sorts, Unités, son titre au-dessus ;
  cartes côte à côte, écart de 8 points, à la hauteur de la plus haute de la rangée (L3) ; passage à la ligne (C8, L1) et
  défilement vertical (L2) ; « Fermer » et « Renoncer » en bas, comme aujourd'hui (maquettes
  [Offres](images/issue-32-cartes-des-fenetres/maquette-offres.png),
  [monastère](images/issue-32-cartes-des-fenetres/maquette-monastere.png)).
- **Fenêtres de choix de l'artefact et du sort** : une rangée, mêmes cartes (C5, C6).
- **Vignette de 64 points** (`with_thumbnail`, spec 072 C6) : plus utilisée par ces fenêtres ; retirée si plus rien ne la
  dessine (`thumbnail` aussi).

Parties préparées existantes, sans en ajouter : `tour-conquise` (touche O : les trois offres ; finir le tour en défaussant
une carte : choix du sort), `monastere` (touche O : action avancée du monastère dans l'offre d'unités), `sombre-terrier`
(touche 1 : choix de l'artefact), `niveau-pair` (choix d'une action avancée : « Prendre » et touches 1 à 3),
`sorts-deplacement` (Appel aux armes : « Utiliser » sous une unité de l'offre), `actions-avancees-mana` (Entraînement :
« Prendre » dans l'offre) ; et la fenêtre Offres en 1 280 × 800 (`ARCANUM_VERIF_TAILLE=1280x800`, L1 et L2).

### Raccourcis clavier

Aucun : les touches des boutons « Prendre » (1 à 3) et d'Échap ne changent pas.

## Documentation à mettre à jour (PR d'implémentation)

- `docs/specs/072-illustrations-generees.md` : C6 et « Interface » (vignette de 64 dans les fenêtres) renvoient à cette
  spec.
- `docs/specs/077-cartes-agrandies.md` : la ligne « Fenêtre Offres et fenêtres de choix » renvoie à cette spec.
- `docs/specs/050-offres-et-cartes-gagnees.md` et `070-combat-et-offres.md` (C18, colonnes) : renvoi à cette spec pour la
  disposition.
- `docs/decisions/0009-illustrations-generees.md` : « vignette de 64 dans les fenêtres » devient « bande dans les
  fenêtres aussi (spec #32) ».
- `docs/architecture.md` si elle décrit `OfferCardView` ou la fenêtre Offres ; `docs/reference/index-par-notion.md`,
  colonne Spec de l'interface des cartes.

## Hors périmètre

- **Infobulle de l'achat au monastère** et des autres achats : gardent « Renforcé : … » (`sites.rs`).
- **Fenêtre de la défausse, écran de combat** : déjà la carte de la main (spec #11, L4).
- **Fenêtre des compétences** (spec 051) et des tactiques (spec 060) : pas des cartes d'action, inchangées.
- **Illustrations des actions avancées, sorts et artefacts** : issue #9 et lots de la spec 072.
- **Taille des cartes réglable** : toujours pas demandé (spec 073).

## Questions ouvertes

- **Q1. Disposition** : une **rangée par offre** (maquette), plutôt que des colonnes de cartes empilées. Des colonnes de
  cartes de 216 points mettraient trois ou quatre cartes l'une sous l'autre (environ 750 points pour les unités) et
  défileraient dès 1 600 × 900 ; les rangées tiennent sans défiler à cette taille et règlent la « taille minimale » de la
  spec 077 (Q4) : chaque carte garde la largeur d'une carte de la main, rien ne rétrécit. Proposition : rangées.
- **Q2. Texte** : celui du « i » de la même carte en main (texte écrit d'abord, C7), pour qu'une carte dise partout la
  même chose ; il diffère parfois du texte calculé d'aujourd'hui (Éclair de feu renforcé : « Blessure ; Attaque de siège de
  feu 8 » au lieu de « Attaque de siège de feu 8 et une Blessure »). Proposition : le texte du « i ».
