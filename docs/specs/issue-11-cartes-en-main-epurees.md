# #11 — Cartes en main épurées

- Statut : à valider (Q1 à Q3 tranchées par Mathieu le 2026-11-24 ; reste à voir l'icône « i »)
- Issue : #11 (Epic #31 Finitions v0.1.0, milestone v0.1.0) ; répond aussi à #12 (Lecture d'esprit) et corrige #13 (cartes
  tronquées), fermées par la PR d'implémentation
- Dépend de : spec 073 (panneaux, boutons d'effet), spec 077 (cartes agrandies) ; implémentées. Après la spec #10 (Source dans
  la barre du haut, PR #38), qui ne touche pas la carte de la main.
- Livret : aucune règle (MKUE p. 4, *Deed Cards* 3.a à 3.d, pour le vocabulaire base / renforcé seulement)
- Crates touchées : arcanum-ui, assets/i18n. Rien dans arcanum-core.
- Base : issue #11 (maquette de Mathieu, reprise de l'issue privée 306), ligne « 089 Cartes en main épurées » de `PLAN.md`
  (Mathieu, 2026-11-18 : le texte complet derrière « i », y compris ce qui ne vaut qu'à plusieurs joueurs).
- Maquettes : [`images/issue-11-cartes-en-main-epurees/`](images/issue-11-cartes-en-main-epurees/), captures du jeu avec un
  essai de code jetable (non versionné, patch dans les fichiers du projet, `ihm/issue-11/maquette-jetable.patch`) :
  sorts [avant](images/issue-11-cartes-en-main-epurees/avant-sorts.png) /
  [après, « i » de Frisson ouvert](images/issue-11-cartes-en-main-epurees/maquette-sorts.png) ;
  actions avancées et unités [avant](images/issue-11-cartes-en-main-epurees/avant-actions-avancees.png) /
  [après](images/issue-11-cartes-en-main-epurees/maquette-actions-avancees.png) ;
  [artefact survolé, après](images/issue-11-cartes-en-main-epurees/maquette-artefact-survol.png).

## Objectif

Une carte de la main ne dit plus deux fois la même chose : ses boutons d'effet portent l'effet, et son texte complet, y
compris ce qui ne vaut qu'à plusieurs joueurs, s'ouvre par un bouton « i » à droite du nom. Le bloc renforcé s'annonce par
une séparation « —— (pastille) Renforcé —— » et ses boutons ne répètent plus le mana de la carte ; un mana en plus s'écrit
« pastille : effet ». Les boutons trop nombreux pour une ligne passent à la ligne au lieu de sortir de la carte.

## Règles

Aucune règle du jeu : spec d'interface.

- **Mana du renforcé** (MKUE p. 4, 3.d ; sorts : p. 5, 2.a, couleur et noir) : il se paie quelle que soit l'option choisie,
  d'où la pastille dans la séparation et plus devant chaque bouton. Une option qui demande un mana de plus (Magie pure, Don
  pour la magie) garde sa pastille, même de la couleur de la carte.
- **Lecture d'esprit** (#12) : en solo, seul le cristal joue (spec 057, `spells.ron`) ; la partie « les autres défaussent… »
  reste dans le texte complet, derrière « i ». La carte n'est pas inachevée.

## Modèle

Aucun terme nouveau. Données ajoutées à la vue de l'écran (`HandCard`, `arcanum-ui/src/screen/types.rs`) :

- `info: Option<CardInfo>` : le texte complet de la carte, montré par « i ». `CardInfo` porte `name` (nom de base),
  `basic` (texte de base), `strong_title` (« Renforcé », ou le nom renforcé d'un sort) et `strong` (texte renforcé). Le texte
  est le **texte écrit** de la carte (`card.<id>.basic_text` et `strong_text`, `unsupported_text_keys`) quand elle en a un
  (actions avancées, sorts), sinon le texte calculé d'aujourd'hui (`effect`, `strong_text` : actions de base). `None` pour une
  Blessure et un artefact.
- `show_text: bool` : la carte écrit son texte sur elle, vrai pour un artefact seulement (Q1).
- `strong_heading: String` : le titre du bloc renforcé, « Renforcé » (`panel.strong`), le nom renforcé d'un sort, ou « si
  détruit » pour un artefact.
- `strong_colors: Vec<ManaColor>` : les pastilles du titre : la couleur de la carte pour une action, la couleur puis le noir
  pour un sort, aucune pour un artefact.

Les champs `effect`, `strong_effect` et `strong_text` restent : `info` les reprend pour une action de base, et les tests qui
les lisent ne changent pas. Seul l'écran cesse de les dessiner.

`EffectButton::color` ne porte plus que le mana **en plus** de celui du renforcé ; il était aussi la couleur de la carte
(actions, renforcé) ou le noir (sorts, renforcé). Les options payantes de l'effet de base passent de `strong_effects` à
`basic_effects` (Q2).

## Actions et événements

Aucun.

## Comportements attendus

Chaque cas devient au moins un test (nom indicatif entre parenthèses). Le dessin (séparation, place du « i », fenêtre du
texte, passage à la ligne) se vérifie à l'œil (ADR 0006) ; la vue est testée.

**Texte complet derrière « i »**

- **C1** — Rage (`DeedCard` `Rage`, action de base rouge) en main : `show_text` faux ; `info` vaut nom « Rage », base
  « Attaque 2 ou Parade 2 », titre « Renforcé », renforcé « Attaque 4 » : les textes calculés d'aujourd'hui (test
  `hand_card_shows_every_option`, spec 073 C13) (`basic_action_info_holds_computed_text`).
- **C2** — Lecture d'esprit (`Spell` `MindRead`, blanc) en main : `info` vaut nom « Lecture d'esprit », base
  `card.mind_read.basic_text` (« Une couleur : gagner son cristal ; les autres défaussent un sort ou une action de cette
  couleur »), titre « Vol d'esprit » (`card.mind_read.strong_name`), renforcé `card.mind_read.strong_text` ; ses boutons de
  base restent les quatre cristaux (#12) (`spell_info_holds_written_text_for_other_players`).
- **C3** — Magie pure (`AdvancedAction` `PureMagic`, bleue) : `info.basic` vaut `card.pure_magic.basic_text` (texte écrit,
  pas le texte calculé) (`advanced_action_info_holds_written_text`).
- **C4** — Une Blessure : `info` vide, `show_text` faux, rien d'autre ne change (spec 077, C1) (`wound_has_no_info`).
- **C5** — Anneau de rubis (`Artifact` `RubyRing`) : `info` vide, `show_text` vrai ; `effect` reste « Jeton et cristal
  rouges ; Gloire +1 » et `strong_text` « Mana rouge et noir illimité ce tour ; Gloire +1 par sort rouge lancé »
  (`artifact_keeps_its_text`).
- **C6** — En mode choix (carte à défausser, spec 013), Rage garde son `info` : on peut lire la carte avant de la défausser
  (`info_stays_in_choice_mode`).

**Titre du bloc renforcé**

- **C7** — Rage : `strong_heading` « Renforcé », `strong_colors` [rouge] ; Frisson (`Spell` `Chill`, bleu) : « Frisson
  fatal », [bleu, noir] ; Anneau de rubis : « si détruit », [] (`strong_heading_names_its_mana`).

**Le mana de la carte n'est plus répété**

- **C8** — Rage : le bouton renforcé (Attaque 4) n'a plus de pastille ; Marche, Mouvement 4, et Tranquillité, Soin 2 et
  Piocher 2, non plus. Tests **modifiés** : `hand_card_shows_every_option`, `effect_button_tooltips`,
  `effect_buttons_without_icon` (panneaux.rs, spec 073 C13, C15, C16) (`strong_buttons_omit_card_mana`).
- **C9** — Boule de feu (`Spell` `Fireball`) renforcée de nuit : son bouton n'a plus la pastille noire ; la commande et le
  bandeau du paiement couleur puis noir ne changent pas. Test **modifié** : `strong_spell_at_night_pays_color_then_black`
  (sorts.rs, `strong[0].color` devient `None`).
- **C10** — Magie pure : les quatre boutons renforcés gardent chacun la pastille de son mana en plus, vert, blanc, **bleu**
  (la couleur de la carte, payée une seconde fois) et rouge (`extra_mana_keeps_its_swatch`).

**Options payantes de base** (Q2)

- **C11** — Cristallisation (`Crystallize`, bleue) en main avec un cristal rouge : `basic_effects` porte les quatre options
  payantes, faces cristal rouge, bleu, blanc, vert, chacune avec la pastille de sa couleur, la rouge seule jouable (même
  commande que `paid_plays[0]`) ; `strong_effects` porte les quatre cristaux du renforcé, sans pastille. Test **remplacé** :
  `paid_option_in_strong_block` (panneaux.rs, spec 073 C17) devient `paid_option_in_basic_block`.
- **C12** — Don pour la magie (`MagicTalent`, bleue) : ses boutons renforcés disent « un sort rouge de l'offre » (nouvelle
  clé `effect.spell_from_offer`) avec la pastille rouge, au lieu de « Payer rouge : un sort rouge de l'offre » ; l'infobulle
  garde « Renforcer : Payer rouge : un sort rouge de l'offre » (`paid_option_face_omits_payment`).

**Unités**

- **C13** — Illusionnistes (`Unit` `Illusionists`) : le bouton de la capacité 2 dit « Un ennemi non fortifié n'attaque pas »,
  avec la pastille blanche, au lieu de « (mana blanc) Un ennemi… » (`unit_mana_ability_omits_mana_prefix`).
- **C14** — Toute capacité d'unité à mana (`UnitAbility::mana`) a, en fr et en en, un texte qui commence par une parenthèse
  fermée puis une espace (« (mana blanc) », « (white mana) ») : la règle de C13 s'applique à toutes
  (`unit_mana_ability_texts_start_with_mana`).

### Tests existants modifiés

Cherchés à la rédaction (`grep -rn 'strong_effects\|basic_effects\|\.color' crates/arcanum-ui/tests`) :

- `crates/arcanum-ui/tests/panneaux.rs` : `hand_card_shows_every_option` (pastille rouge du renforcé retirée),
  `effect_button_tooltips` (pastille verte de Marche renforcée retirée), `effect_buttons_without_icon` (pastilles vertes de
  Tranquillité retirées), `paid_option_in_strong_block` (remplacé par C11).
- `crates/arcanum-ui/tests/sorts.rs` : `strong_spell_at_night_pays_color_then_black` (C9).
- `crates/arcanum-ui/tests/icones.rs` : `every_icon_image_exists` (37 images de `ui/`, 38 avec `info.svg`).
- À vérifier au fil d'implémentation, sans changement attendu : les tests qui indexent `strong_effects` d'une carte à options
  payantes (`actions_avancees_mana.rs`, Don pour la magie : indices des options renforcées inchangés, la carte n'a pas
  d'option payante de base) et `cartes_de_mana.rs` (`paid_plays`, inchangé). Aucun champ retiré ni renommé.

## Cas limites et refus

- **L1** — Nom long : le « i » se place entre le nom et les icônes de côté ; le nom se tronque avant lui, comme aujourd'hui
  devant les icônes (maquette : « Bouclier… », « Coup de s… ») ; le nom entier se lit dans la fenêtre du « i » (Q3).
- **L2** — Une seule fenêtre de texte ouverte à la fois : ouvrir le « i » d'une autre carte ferme la première ; un clic
  ailleurs, Échap, ou un second clic sur le « i » la ferment.
- **L3** — La carte survolée qui grandit (spec 073 C12 bis) et la carte coupée marquée « … » (spec 077, L2) restent ; elles
  deviennent rares, le texte n'étant plus écrit.
- **L4** — Fenêtre de la défausse et écran de combat : même carte (`card_tile`), même « i ».

## Information cachée

Rien de nouveau : le texte d'une carte en main est public pour son joueur. Champs ajoutés à la vue de l'écran :
`HandCard::info`, `show_text`, `strong_heading`, `strong_colors` (voir Modèle).

## Libellés (i18n)

Nouvelles clés (fr, en) :

| Clé | fr | en |
|---|---|---|
| `card.info` | Texte de la carte | Card text |
| `card.if_destroyed` | si détruit | if destroyed |
| `effect.spell_from_offer` | un sort {color} de l'offre | a {color} Spell from the offer |

Avec `{color}` = « rouge » / « red » : « un sort rouge de l'offre », « a red Spell from the offer ». `card.info` et
`card.if_destroyed` sont des libellés à côté des groupes de cartes (`card.rage`…) : aucune carte n'a l'identifiant `info` ni
`if_destroyed`.

Clés réutilisées, relues dans `main` : `panel.strong` « Renforcé » / « Powered » ; `card.<id>.strong_name` (« Vol
d'esprit ») ; `card.<id>.basic_text` et `strong_text` ; `effect.gain_crystal` « Cristal {color} » (texte de repli de la face
cristal, « Cristal rouge »). `effect.pay_for_crystal` et `effect.gain_spell` restent pour les infobulles et le bandeau.

Le « i » est une icône SVG, pas un caractère : aucun souci de police. Les traits de la séparation sont dessinés, pas écrits.

## Icône « i »

Demandée par Mathieu (2026-11-24, sur un modèle : rond bleu, « i » blanc calligraphié). Dessinée pour cette spec,
[`images/issue-11-cartes-en-main-epurees/info.svg`](images/issue-11-cartes-en-main-epurees/info.svg) (aperçu :
[`info-apercu.png`](images/issue-11-cartes-en-main-epurees/info-apercu.png), en grand, à 18 points devant un nom, à 32 et
64) : rond bleu `#1E4F8C` cerné du trait sombre des autres icônes (`#20242A`), point et jambage crème `#F4F1E8`, jambage
incliné avec son attaque et sa queue arrondies. Fond 64 × 64 comme les icônes de la spec 071.

La PR d'implémentation la copie en `assets/images/ui/info.svg` et ajoute `Icon::Info` (stem `info`) ; le compte de
`every_icon_image_exists` (icones.rs, spec 071 C1) passe de 37 à 38 : test **modifié**.

## Interface

- **En-tête** : pastille, nom, puis le bouton « i », l'icône `Icon::Info` à la taille des icônes de côté (infobulle `card.info`), puis les icônes de côté ou la case
  « Défausser » à droite, comme aujourd'hui (C1, C4, C5 : pas de « i » sur une Blessure ni un artefact ; place : vérification
  à l'œil).
- **Fenêtre du « i »** : au clic, au-dessus du bouton, par-dessus le reste ; le nom (en gras), le texte de base, un trait,
  le titre renforcé (en gras), le texte renforcé ; 260 points de large au plus (C1 à C3, `HandCard::info` ; vérification à
  l'œil, maquette [sorts](images/issue-11-cartes-en-main-epurees/maquette-sorts.png)).
- **Corps d'une action ou d'un sort** : bande d'illustration (spec 077), puis les boutons de base, sans le texte au-dessus
  (`show_text` faux) ; la note de combat (« siège seulement », « Soin : pas en combat ») reste (C1 ; vérification à l'œil).
- **Séparation d'une action** : un trait, la pastille de la carte, « Renforcé », un trait, centrés sur la largeur ; puis les
  boutons renforcés (C7, `strong_heading`, `strong_colors` ; vérification à l'œil).
- **Sort** : sous les boutons de base, un trait simple, puis une ligne avec les deux pastilles (couleur, noir) et le nom
  renforcé en gras ; puis ses boutons (C7 ; maquette [sorts](images/issue-11-cartes-en-main-epurees/maquette-sorts.png)).
- **Artefact** (Q1) : texte de base écrit sur la carte, bouton « Jouer » ou « Assigner », séparation « —— si détruit —— »
  sans pastille, texte renforcé, bouton « Détruire » ; pas de « i » (C5, C7 ; maquette
  [artefact](images/issue-11-cartes-en-main-epurees/maquette-artefact-survol.png)).
- **Bouton à mana en plus** : la pastille, « : », puis l'icône et sa valeur, le cristal ou le texte (C10 à C13 ; vérification
  à l'œil). Vaut pour les boutons d'unité.
- **Passage à la ligne** (#13) : tout bouton d'effet qui ne tient pas au bout de la ligne passe à la suivante, qu'il soit
  texte, icône ou cristal, pastille comprise. Aujourd'hui seuls les boutons texte sont mesurés avant d'être posés
  (`effect_button`), d'où Magie pure renforcée et la 3e capacité des Illusionnistes hors de la carte (vérification à l'œil,
  `actions-avancees`, maquette [après](images/issue-11-cartes-en-main-epurees/maquette-actions-avancees.png)).
- **Mode choix** : les boutons « Défausser cette carte » à la place des boutons d'effet, sans texte (C6 ; vérification à
  l'œil).

Parties préparées existantes, sans en ajouter : `main-blessures` (Marche, Promesse, Rage, Blessures), `actions-avancees`
(Magie pure, Bouclier de glace, Illusionnistes), `sorts` (sorts et Don pour la magie, de nuit : noir disponible), `artefacts`
(survoler une carte pour la voir entière), `unites`.

### Raccourcis clavier

Aucun : Échap ferme la fenêtre du « i » (L2), comme les autres fenêtres ; ouvrir le texte d'une carte au clavier
demanderait de désigner la carte, ce que rien d'autre ne fait encore.

## Documentation à mettre à jour (PR d'implémentation)

- `docs/specs/073-panneaux.md` : C13, C15 à C17 renvoient à cette spec (pastille du renforcé, options payantes).
- `docs/specs/077-cartes-agrandies.md` : « Carte de la main » renvoie à cette spec (texte de l'effet retiré).
- `docs/reference/index-par-notion.md` : colonne Spec des notions Cartes en main / Interface.
- `docs/architecture.md` si elle décrit `HandCard` ou `card_tile`.

## Hors périmètre

- **Fenêtre Offres et fenêtres de choix** (artefact, sort) présentées comme la main : issue #32 (spec 086), qui reprend cette
  présentation.
- **Compétences** : leur texte reste en infobulle (spec 051), inchangé.
- **Unités** : seuls le bouton à mana (C13) et le passage à la ligne changent ; pas de « i » (leurs boutons disent tout).
- **Texte complet au survol** du nom : non demandé (le « i » s'ouvre au clic).

## Questions ouvertes

Tranchées par Mathieu (2026-11-24) :

- **Q1. Artefacts** : ils gardent leur texte sur la carte, sans « i » ni ligne « Renforcé », avec la séparation « —— si
  détruit —— » (proposition retenue : leurs boutons ne disent que « Jouer » et « Détruire »).
- **Q2. Options payantes de base** (Cristallisation, Magie pure de base) : pas d'exception pour ces cartes, le formalisme
  général s'applique : dans le bloc de base, au format « pastille : effet » (C11 ; spec 073 C17 remplacée).
- **Q3. Nom tronqué** : accepté, le nom entier se lit dans la fenêtre du « i » (L1).
- **Icône « i »** : en SVG, dans le style du modèle fourni (section Icône « i ») ; dessin à valider.
