# 071 — Icônes et raccourcis visibles

- Statut : implémentée (2026-11-06 ; validée le même jour, Q1 à Q4 tranchées par Mathieu). Plan :
  [`plans/071-icones-et-raccourcis.md`](plans/071-icones-et-raccourcis.md).
- Jalon : IHM (refonte de l'interface, entre J4 et J5)
- Dépend de : 070 (carte d'ennemi simplifiée), 013 et 032 (jouer de côté, repères ÷2), 021 (inventaire), 024 et 065 (boutons
  et leurs infobulles), 042 (unités), 043 (drapeaux) ; toutes implémentées
- Livret : aucune règle nouvelle. Rappels : MKUE p. 4, *Deed Cards* 3.e (jouer de côté) ; p. 24 *Enemy Token Abilities*
  (icônes d'éléments et de résistances du jeu, dont les nôtres s'inspirent sans les reprendre)
- Crates touchées : arcanum-ui, `assets/images/` (SVG nouveaux), `assets/i18n/` (fr.json et en.json). Rien dans arcanum-core.
- Base : [synthèse de la refonte](refonte-ihm.md), sections « Divers » et « Iconographie » ; [ADR 0009](../decisions/0009-illustrations-generees.md)
  (icônes simples en SVG dessinés par nous)
- Maquette : [planche des icônes](images/071-icones/planche.png) et les SVG proposés dans
  [`images/071-icones/`](images/071-icones/)

## Objectif

Deuxième lot de la refonte de l'interface : moins de texte. Les valeurs du jeu (Mouvement, Influence, Armure, Gloire, attaques
par élément, parades, résistances, cristaux et jetons) s'affichent avec une icône dessinée par nous, et leur nom passe en
infobulle. Jouer une carte de côté se fait d'un clic sur l'icône de l'effet voulu, sans menu. Chaque bouton qui a un raccourci
clavier le dit dans son infobulle. Chaque héros a un emblème, repris sur ses drapeaux.

## Règles

Aucune règle n'est ajoutée ni changée : le moteur propose les mêmes actions légales, seul l'écran les montre autrement.

- **Jouer de côté** (MKUE p. 4, *Deed Cards* 3.e) : 1 Mouvement, 1 Influence, 1 Attaque ou 1 Parade. L'écran propose, comme
  aujourd'hui, seulement les effets que le moteur rend légaux (`hand.rs`, actions `CardPlay::Sideways` ; test
  `boucle_et_fenetre.rs`, `first_sideways_influence`, qui prend la deuxième entrée, Influence).
- **Éléments et résistances** (MKUE p. 24, *Offensive* et *Defensive*) : le livret montre une icône par élément et par
  résistance (flamme, flocon, écu de couleur). Nos icônes en reprennent l'idée, pas le dessin (CLAUDE.md, « Assets et droits »).

## Modèle

Aucun terme nouveau au glossaire : chaque icône porte un terme qui y est déjà (Mouvement, Influence, Armure, Gloire, Attaque à
distance, Attaque de siège, Parade, Résistance, cristal, jeton de mana, jouer de côté).

Rien de nouveau dans l'état ni dans la vue. Ce que l'écran gagne :

- **Icône** (`Icon`, arcanum-ui) : une valeur par dessin de la planche, chacune avec son fichier SVG `assets/images/ui/<nom>.svg` (un
  `ImageId` de répertoire `ui`, comme ceux du plateau) et sa clé de nom `icon.<nom>`. Les attaques, attaques à distance,
  attaques de siège et parades existent pour les quatre éléments (physique, feu, glace, feu glacé) ; les résistances pour
  physique, feu et glace ; les cristaux pour les quatre couleurs de base, plus or et noir pour les jetons, et un emplacement vide.
- **Valeur à icône** (`IconValue`) : une icône, une valeur (« 3 », « 3 / 7 », ou vide pour une résistance) et le texte de
  l'infobulle (le nom de l'icône : « Attaque de feu »). Sans fichier d'icône, l'écran écrit le nom puis la valeur, comme
  aujourd'hui (ADR 0009, « Une image manquante n'empêche pas de jouer »).
- **Bouton-icône** (`IconButton`) : une icône, un repère facultatif (« ÷2 », spec 032), une infobulle et une `Command`.
- **Raccourci d'un bouton** : `Button`, `ControlButton`, `StrongButton`, `PurchaseButton` et les boutons des fenêtres gagnent le
  libellé de leur touche (`shortcut: Option<String>`, déjà traduit : « Ctrl+Z », « Entrée »). La touche d'un `Shortcut` se lit dans
  une seule table, partagée avec la lecture du clavier de `app/mod.rs` : le texte affiché ne peut pas dire une autre touche que
  celle qui agit.
- **Emblème de héros** : un SVG par héros, `assets/images/ui/hero_<héros>.svg` (flamme, griffes, feuille, hache), posé aussi sur
  l'étoffe de son drapeau (`assets/images/flag/<héros>.svg`, spec 043).

## Actions et événements

Aucune action ni aucun événement nouveau.

## Comportements attendus

Tests dans `crates/arcanum-ui/tests/icones.rs`, sauf mention.

Jeu d'icônes :

- **C1** — Chaque `Icon` a son fichier SVG dans `assets/images/ui/`, qui commence par `<svg`, et sa clé de nom en français
  (`every_icon_image_exists`, `every_icon_has_a_name`), comme `every_board_image_exists` (spec 011).
- **C2** — L'icône d'une attaque suit sa sorte et son élément : Attaque de feu → épée de feu, Attaque à distance de glace →
  arc de glace, Attaque de siège de feu glacé → catapulte de feu glacé ; une Parade de feu → rondache de feu
  (`attack_icon_follows_kind_and_element`, `block_icon_follows_element`).

Panneau latéral :

- **C3** — La ligne des points montre l'icône Mouvement et sa valeur, puis l'icône Influence et la sienne, infobulles
  « Mouvement » et « Influence » (`points_line_uses_icons`) ; elle remplace « Mouvement : 3 · Influence : 2 ».
- **C4** — La ligne de gloire commence par l'icône Gloire et la valeur (« 0 »), puis « Niveau 1 · niveau 2 à 3 » sans le mot
  « Gloire » (`fame_line_starts_with_icon`) ; au niveau 10, « Niveau 10 (maximum) ».
- **C5** — La ligne du niveau commence par l'icône Armure et la valeur de l'armure du héros, puis « Limite de main 5 · Jetons de
  commandement 1 » (`level_stats_start_with_armor_icon`).
- **C6** — L'inventaire montre, par couleur, un cristal plein par cristal possédé et un emplacement vide jusqu'à la limite,
  infobulle inchangée (« Cristaux bleus : 2 sur 3 ») : la donnée ne change pas (`CrystalGroup`, spec 021), seul le dessin
  (vérification à l'œil).
- **C7** — Les jetons de mana de la zone de jeu se dessinent avec le cristal de leur couleur (or et noir compris), infobulle
  inchangée (`TokenView`, spec 025 ; vérification à l'œil).

Écran de combat :

- **C8** — La carte d'un ennemi donne ses valeurs en icônes : Armure et sa valeur, l'attaque à l'icône de son élément et sa
  valeur, Gloire et sa valeur (« Gardes du feu » : armure 7, épée de feu 6, gloire 5) (`enemy_card_values_use_icons`).
- **C9** — Un Invocateur n'a pas d'icône d'attaque : « invocation » reste écrit entre l'armure et la gloire
  (`summoner_attack_stays_text`) — spec 033.
- **C10** — Les résistances d'un ennemi sont des icônes de résistance, sans valeur, infobulles « Résistance physique »,
  « Résistance au feu », « Résistance à la glace » ; elles remplacent les badges de la spec 032 (`resistances_use_icons`).
- **C11** — La créature invoquée (spec 033) a ses valeurs en icônes comme en C8 (`summoned_values_use_icons`).
- **C12** — Le total d'une attaque en cours s'affiche avec l'icône de la phase (arc en phase d'attaque à distance et de siège,
  épée en phase d'attaque) puis « 3 / 7 » ; celui d'une parade avec la rondache physique puis « 5 / 6 » ; infobulles « Attaque
  à distance ou de siège », « Attaque », « Parade » (`in_progress_total_uses_phase_icon`).
- **C13** — Une unité montre son nom et son niveau en texte, puis l'icône Armure et sa valeur, puis une icône par résistance
  (`unit_line_uses_icons`) ; elle remplace « Paysans · niveau I · armure 3 » et la liste des résistances.

Jouer de côté :

- **C14** — Une carte jouable de côté a un bouton-icône par effet légal, dans l'ordre Mouvement, Influence, Attaque, Parade ;
  infobulle « Jouer de côté : Mouvement 1 » ; un clic joue la carte de côté pour cet effet, sans menu
  (`sideways_buttons_one_per_effect`).
- **C15** — Les boutons-icônes suivent les effets légaux, comme aujourd'hui (`sideways_effects` et `sideways_in_combat` de
  `engine.rs` et `combat.rs` ; test arcanum-core `melee_and_sideways_attack_in_attack_phase`) : hors combat, Mouvement et Influence ;
  en combat, l'Attaque seule pendant une attaque en cours de la phase d'attaque, la Parade seule pendant une parade en cours,
  aucun sinon (`sideways_buttons_follow_legal_actions`).
- **C16** — Contre une résistance physique, le bouton-icône d'Attaque porte le repère « ÷2 », et son infobulle le dit comme
  aujourd'hui le texte (« Jouer de côté : Attaque 1 ÷2 ») (`sideways_halved_mark`) — spec 032.
- **C17** — Une Blessure, qui ne se joue pas de côté, n'a aucun bouton-icône (`wound_has_no_sideways_buttons`), comme
  aujourd'hui (test arcanum-core `wound_cannot_be_played`, spec 031).

Raccourcis affichés :

- **C18** — « Revenir en arrière », « Sauvegarder », « Charger… » et « Nouvelle partie… » gardent leur description en infobulle,
  sans la touche entre parenthèses, et y ajoutent une ligne « Raccourci : Ctrl+Z » (Ctrl+S, Ctrl+O, Ctrl+N), y compris grisés
  (`control_buttons_show_shortcut`, dans `sauvegarde_ui.rs`).
- **C19** — « Finir le tour… » dit « Raccourci : Fin » ; en mode fin du tour, « Finir le tour » dit « Raccourci : Entrée » et
  « Annuler » « Raccourci : Échap » (`end_turn_buttons_show_shortcut`, dans `fin_du_tour.rs`).
- **C20** — En combat, « Valider l'attaque » et « Valider la parade » disent « Raccourci : Entrée », « Annuler » (spec 070)
  « Raccourci : Échap » (`combat_buttons_show_shortcut`, dans `combat_et_offres.rs`).
- **C21** — « Offres… » dit « Raccourci : O », « Interagir avec les habitants » « Raccourci : I », « Entrer dans : … » et « Faire
  une offrande à l'autel » « Raccourci : E » (`site_buttons_show_shortcut`).
- **C22** — Dans un choix numéroté, chaque bouton dit sa touche : « Prendre » de la 2e action avancée « Raccourci : 2 », de même
  pour les artefacts, les sorts, les unités de récompense, la couleur du cristal (1 à 4) et les paiements de l'offrande (1 à 9)
  (`numbered_choices_show_shortcut`).
- **C23** — Chaque autre bouton « Annuler » ou « Fermer » qu'Échap actionne dans le même contexte (`docs/raccourcis.md` : choix de
  la carte à défausser, du mana, des dés, Concentration, annonce de la fin de la manche, incendie, entrée dans un site, mana d'un
  sort, fenêtres de chargement, de nouvelle partie, Offres, de l'offrande, de fin, choix de l'unité à renvoyer) dit « Raccourci :
  Échap » ; « Commencer » de la nouvelle partie dit « Raccourci : Entrée » (`escape_buttons_show_shortcut`).
- **C24** — Un bouton sans touche n'a pas de ligne « Raccourci » (`buttons_without_key_show_no_shortcut`).
- **C25** — Chaque touche affichée est celle que lit le clavier : pour chaque `Shortcut`, le libellé vient de la table des touches
  de l'application (`shown_shortcut_matches_bound_key`).

Héros :

- **C26** — Chaque héros a son emblème, `assets/images/ui/hero_<héros>.svg` (`every_hero_has_an_emblem`) ; son drapeau
  (spec 043) porte l'emblème sur l'étoffe (vérification à l'œil).

## Cas limites et refus

- **L1** — Fichier d'icône absent (dossier des ressources incomplet) : l'écran écrit le nom puis la valeur (« Mouvement 3 »), et
  le bouton-icône de côté devient un bouton texte « Mouvement 1 » ; le jeu reste jouable (vérification à l'œil avec un
  `ARCANUM_ASSETS_DIR` sans `images/ui/`).
- **L2** — Bouton grisé : l'infobulle (raison du grisé, puis la ligne « Raccourci ») s'affiche aussi au survol d'un bouton grisé,
  comme « Revenir en arrière » aujourd'hui (spec 024) (couvert par C18).
- **L3** — Bouton avec infobulle existante (« Il faut un mana vert… », spec 022 ; texte d'une action avancée, spec 041) : la ligne
  « Raccourci » vient après, s'il y a une touche (couvert par C22).

## Information cachée

Rien de nouveau : chaque valeur vient de la vue (`PlayerView`) comme aujourd'hui ; l'écran change seulement de forme.

Champs de la vue qui nourrissent les icônes : `PlayerView::move_points` et `influence_points` (C3), `fame`, `level` et `next_level_fame` (C4),
`armor`, `hand_limit` et `command_tokens` (C5), `inventory` (C6), `mana_tokens` (C7), `PlayerView::combat` (C8 à C12), les unités (C13), `hero` (C26) ; les boutons de côté viennent
des actions légales (C14 à C17).

## Libellés (i18n)

Valeurs relues dans `assets/i18n/fr.json` et `en.json` le 2026-11-06. `icon` et `shortcut` sont deux groupes nouveaux de premier
niveau (absents aujourd'hui ; `time_of_day.icon` est un autre groupe, sous `time_of_day`).

Noms des icônes (infobulles), groupe `icon` :

| Clé | fr | en |
|---|---|---|
| `icon.move` | « Mouvement » | « Move » |
| `icon.influence` | « Influence » | « Influence » |
| `icon.armor` | « Armure » | « Armor » |
| `icon.fame` | « Gloire » | « Fame » |
| `icon.attack.physical`, `.fire`, `.ice`, `.cold_fire` | « Attaque », « Attaque de feu », « Attaque de glace », « Attaque de feu glacé » | « Attack », « Fire Attack », « Ice Attack », « Cold Fire Attack » |
| `icon.ranged_attack.*` (mêmes éléments) | « Attaque à distance », « … de feu », « … de glace », « … de feu glacé » | « Ranged Attack », « Ranged Fire Attack »… |
| `icon.siege_attack.*` | « Attaque de siège », « … de feu »… | « Siege Attack », « Siege Fire Attack »… |
| `icon.block.*` | « Parade », « Parade de feu », « Parade de glace », « Parade de feu glacé » | « Block », « Fire Block », « Ice Block », « Cold Fire Block » |
| `icon.ranged_or_siege` | « Attaque à distance ou de siège » | « Ranged or Siege Attack » |
| `icon.resistance.physical`, `.fire`, `.ice` | « Résistance physique », « Résistance au feu », « Résistance à la glace » | « Physical Resistance », « Fire Resistance », « Ice Resistance » |

Les cristaux et jetons gardent leurs infobulles actuelles (`inventory.crystals.*`, `play_area.token_tooltip`).

Jouer de côté et raccourcis :

| Clé | fr | en | Note |
|---|---|---|---|
| `play.sideways_effect` | « Jouer de côté : {effect} » | « Play sideways: {effect} » | nouvelle, sœur de `play.sideways` (« Jouer de côté », gardée pour le repli) ; `{effect}` = « Attaque 1 ÷2 » donne « Jouer de côté : Attaque 1 ÷2 » / « Play sideways: Attack 1 ÷2 » |
| `shortcut.hint` | « Raccourci : {key} » | « Shortcut: {key} » | `{key}` = « Ctrl+Z » donne « Raccourci : Ctrl+Z » / « Shortcut: Ctrl+Z » |
| `shortcut.key.end`, `.enter`, `.escape` | « Fin », « Entrée », « Échap » | « End », « Enter », « Esc » | les lettres et chiffres ne se traduisent pas |
| `shortcut.key.ctrl` | « Ctrl+{key} » | « Ctrl+{key} » | `{key}` = « Z » donne « Ctrl+Z » |
| `undo.hint`, `save.hint`, `load.hint`, `new_game.hint` | « Reprendre votre dernière action », « Sauvegarder la partie », « Charger une partie », « Commencer une autre partie » | sans « (Ctrl+…) » | valeurs actuelles moins la touche entre parenthèses (C18) |

Supprimées : `points.line` (C3) ; `fame.line` et `fame.line_max` deviennent `fame.level` (« Niveau {level} · niveau
{next_level} à {next_fame} ») et `fame.level_max` (« Niveau 10 (maximum) ») ; `level.stats` perd « Armure {armor} · »
(C5) ; `units.line` devient « {name} · niveau {level} » (C13).

Aucun caractère nouveau : « ÷ » est déjà affiché (spec 032).

## Interface

- **Icônes** : SVG de 64 × 64 de la planche, affichés à la hauteur du texte (environ 18 à 20 px) dans les lignes, 16 px dans
  les boutons ; tailles et alignement : vérification à l'œil.
- **Panneau latéral** : ligne des points (C3, `PlayerView::move_points` et `influence_points`), gloire (C4), niveau (C5), inventaire (C6), jetons (C7).
- **Carte d'ennemi** (spec 070) : la ligne « armure 3, attaque 4, gloire 2 » devient une ligne d'icônes, résistances au bout de
  la ligne (C8 à C11, `PlayerView::combat`).
- **Cadres de l'attaque et de la parade en cours** : icône puis total (C12).
- **Unités**, dans le panneau et sous la main en combat : C13.
- **Carte de la main** : à droite du nom, un encart qui commence par l'icône « carte couchée » (infobulle « Jouer de côté »,
  non cliquable) suivie d'un bouton-icône par effet légal (C14 à C17, actions légales). Le menu de côté disparaît (variante B de
  la planche, retenue : Q1).
- **Infobulles des raccourcis** : C18 à C25.
- **Drapeaux** : emblème du héros sur l'étoffe (C26, vérification à l'œil).

### Raccourcis clavier

Aucune touche ajoutée ni modifiée : la spec rend visibles les touches existantes de `docs/raccourcis.md`. Le registre gagne une
phrase d'en-tête : chaque bouton qui a une touche la montre en infobulle (spec 071).

## Hors périmètre

- **Texte des cartes** (effets de base et renforcés, boutons « Jouer : Mouvement 2 ») : reste du texte (synthèse, « pas dans la
  description des cartes ») ; la séparation base / renforcé vient avec la spec 073.
- **Infobulles du plateau** (jeton d'ennemi survolé) : restent du texte ; le plateau est refait par la spec 075.
- **Capacités des ennemis** (vif, brutal, toxique, paralysant, fortifié, invocation) : restent du texte (Q3).
- **Réputation, Soin, Limite de main, jetons de commandement** : pas d'icône demandée, texte inchangé.
- **Raccourcis en face du libellé dans les menus** : avec la barre de menu, spec 074.
- **Touches du plateau** (flèches, `+`/`-`, Début) : sans bouton, rien à afficher ici ; une aide des touches pourra venir avec la
  spec 074.
- **Disposition** du panneau (joueur, zone de jeu, boutons du bas) : spec 073.

## Questions ouvertes

Aucune.

## Questions tranchées

Toutes tranchées par Mathieu le 2026-11-06, au vu de la planche.

- **Q1. Jouer de côté** : variante B, un encart avec l'icône « carte couchée » devant les boutons-icônes.
- **Q2. Icône de héros** : un emblème par héros (flamme, griffes, feuille, hache), posé sur le drapeau ; le portrait reste dans
  le panneau (proposition acceptée).
- **Q3. Capacités des ennemis** : restent du texte ; des icônes plus tard, dans une spec à part si le texte gêne encore après
  la spec 073 (proposition acceptée).
- **Q4. Visuels** : planche acceptée, sauf l'icône Mouvement, redessinée en coureur stylisé (essai B), jambe arrière abaissée pour
  ne pas toucher le bras. Les SVG de `images/071-icones/` font foi : le fil d'implémentation les copie tels quels dans
  `assets/images/ui/` (drapeaux compris, dans `assets/images/flag/`).
