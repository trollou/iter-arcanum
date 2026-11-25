# #14 — Ennemis par catégorie

- Statut : à valider
- Issue : #14 (Epic #31 Finitions v0.1.0, milestone v0.1.0)
- Dépend de : spec 030 (jetons d'ennemis), spec 043 (défenseurs face cachée), spec 045 (ruines), spec 062 (cités), spec 070 (groupe dans l'infobulle), spec 075 (plateau isométrique, socle du héros)
- Livret : aucune règle nouvelle (MKUE p. 3 et p. 7, 4 pour les piles et les défenseurs, inchangés)
- Crates touchées : arcanum-ui, assets/i18n

## Objectif

Le joueur lit partout la **catégorie** d'un ennemi (« gardes de forteresse », « tour de mage », « donjon »…) au
lieu de la couleur de sa pile (« gris », « violet », « marron »). Sur le plateau, chaque figurine de monstre
repose sur un **socle à la couleur de sa catégorie**, comme le héros sur le sien : on reconnaît d'un coup d'œil des
orques (vert), un draconien (rouge) ou un défenseur de cité (blanc).

## Règles

- **Piles d'ennemis** (MKUE p. 3, *Enemy and Ruin token piles*) : une pile par couleur de dos. La spec ne change
  ni les piles ni les tirages ; elle ne change que la façon de les nommer et de les montrer.
- **Défenseurs face cachée** (MKUE p. 7, 4 ; spec 043, spec 062) : seule leur couleur est connue. La catégorie en
  découle (une couleur, une catégorie) : elle ne révèle rien de plus (voir Information cachée).

## Modèle

- **Catégorie d'ennemis** : le nom d'une pile, une par `EnemyColor`, déjà libellé par `enemy.color.*` et affiché
  dans l'infobulle d'un jeton depuis la spec 070 (« Rôdeurs (orques maraudeurs) », `enemy.token`).
  Terme à ajouter au glossaire, à côté de « Couleur d'ennemi » :
  `| (token back category) | Catégorie d'ennemis | EnemyColor, libellés enemy.color.* (spec #14) |`.
- Aucun champ nouveau dans l'état ni dans la vue : `PlayerView::face_down_defenders`, `enemy_tokens` et les
  jetons de ruines portent déjà la couleur.
- Écran : `Overlay` (plateau) gagne un **socle** facultatif, la couleur à peindre sous l'élément debout ;
  `EnemyCard` (combat) aussi, si Q3 est acceptée.

## Actions et événements

Aucun.

## Comportements attendus

Les catégories, relues dans `assets/i18n/fr.json` et `en.json` de `main` (groupe `enemy.color`) :

| `EnemyColor` | fr | en |
|---|---|---|
| Green | orques maraudeurs | marauding orcs |
| Grey | gardes de forteresse | keep |
| Violet | tour de mage | mage tower |
| Brown | donjon | dungeon |
| Red | draconien | draconum |
| White | cité | city |

### Textes

Chaque texte qui donnait la couleur d'une pile donne sa catégorie. Ce sont **tous** les lecteurs de
`pile_color_key` et des chaînes `"enemy.pile_color.*"` (`board/screen.rs`, `screen/ruins.rs`,
`screen/adventure.rs`) et les deux libellés qui écrivent une couleur en dur (`combat.summon_warning`,
`burn.confirm`).

- **C1** — Infobulle d'une ruine face visible à jeton d'ennemis : « Ennemis : gardes de forteresse, tour de mage ;
  récompense : une unité de l'offre » au lieu de « Ennemis : gris, violet ; … ». Les autres jetons du test :
  « donjon, draconien » (au lieu de « marron, rouge ») et « donjon, tour de mage » (au lieu de « marron, violet »).
  (test existant modifié : `ruines.rs`, lignes 285 à 302) — spec 045.
- **C2** — Confirmation d'entrée dans cette ruine, première entrée : « Combat contre des ennemis tirés au hasard :
  gardes de forteresse, tour de mage. » (test existant modifié : `ruines.rs`, ligne 461) — spec 045.
- **C3** — Infobulle d'une forteresse dont le défenseur est face cachée (partie `Fortified` des tests) :
  « défenseur face cachée (gardes de forteresse) » au lieu de « défenseur gris, face cachée » (test existant
  modifié : `sites_fortifies.rs`, ligne 191) — spec 043.
- **C4** — Infobulle d'une cité aux défenseurs face cachée : « Défenseurs face cachée : tour de mage, tour de mage,
  cité » au lieu de « violet, violet, blanc » (test existant modifié : `cites.rs`, ligne 136) — spec 062.
- **C5** — Confirmation d'entrée dans un site d'aventure qui tire un jeton : « Combat contre 1 ennemi(s) tiré(s) au
  hasard (donjon). » au lieu de « Combat contre 1 ennemi(s) marron, tiré(s) au hasard. » ; au tombeau,
  « (draconien) » (tests existants modifiés : `sites_d_aventure.rs`, lignes 299, 327 et 338) — spec 044.
- **C6** — Avertissement d'invocation en combat : « …invoquent une créature (donjon) : pas de retour en arrière
  après. » au lieu de « (pile marron) » (test existant modifié : `deroule_combat.rs`, ligne 1041) — spec 033.
- **C7** — Confirmation de l'incendie d'un monastère : « Incendier le monastère ? Réputation −3, combat contre un
  ennemi tiré au hasard (tour de mage), sans vos unités. Pas de retour en arrière. » au lieu de « contre un
  ennemi violet » (test existant modifié : `villages_et_monasteres.rs`, ligne 273) — spec 041.
- **C8** — Plus aucun texte ne nomme une pile par sa couleur : le groupe `enemy.pile_color` disparaît de `fr.json`
  et `en.json`, et `pile_color_key` du code (le test des clés communes à toutes les langues le garde, et le
  compilateur pour la fonction ; pas de test nouveau). Comme aujourd'hui, l'infobulle d'un jeton visible reste
  « Rôdeurs (orques maraudeurs) : armure 3, attaque 4, gloire 2 » (spec 070, `ennemis_errants.rs`, ligne 191).

### Socles sur le plateau

Le socle est un disque à plat sous les pieds de la figurine, comme celui du héros (spec 075, `paint_hero_base`),
rempli de `enemy_color(couleur)` (`board/images.rs`, les couleurs de repli des jetons, spec 030) et assombri la
nuit comme les éléments debout (spec 075, L8).

- **C9** — Les Rôdeurs errants (`enemies.ron`, Prowlers, pile verte) : leur élément debout (`OverlayKind::Rampaging`)
  a pour socle `enemy_color(Green)` (`rampaging_enemy_has_a_category_base`) — à côté de
  `rampaging_enemy_is_a_standing_figure` (spec 075, C17).
- **C10** — Au Sombre Terrier, la Gargouille et le Minotaure restés (`OverlayKind::SiteEnemy`, pile marron) ont
  chacun leur socle `enemy_color(Brown)`, réduit avec la figurine (`site_enemies_have_a_category_base`).
- **C11** — Défenseur de la forteresse (pile grise) : face cachée, le brouillard teinté (spec 075, C18) a pour socle
  `enemy_color(Grey)` ; retourné (Gardiens, Guardsmen), sa figurine aussi (`defenders_have_a_category_base`).
- **C12** — Les autres éléments debout (héros, site, drapeau, relief) n'ont pas de socle d'ennemi : le héros garde
  le sien à sa couleur, inchangé (spec 075, `hero_figure_uses_its_standing_image`)
  (`only_enemies_have_a_category_base`).

### Écran de combat (si Q3 est acceptée)

- **C13** — La carte des Rôdeurs (`EnemyCard`) a pour socle `enemy_color(Green)`, dessiné sous la figurine
  (`combat_card_has_a_category_base`) — à côté de `combat_shows_the_figure` (spec 075, C20b).

## Cas limites et refus

- **L1** — Une pile vide n'existe pas pour l'affichage : le texte nomme la catégorie du jeton à tirer, même si le
  moteur tire finalement dans une autre pile (pile et défausse vides, MEMORY « Piles de jetons vides ») ; c'est déjà
  le cas de la couleur aujourd'hui, la spec ne le change pas.
- **L2** — Figurine sans image (repli, spec 075, L1) : le disque de repli garde la couleur de la pile ; le socle est
  dessiné dessous quand même (vérification à l'œil).

## Information cachée

Rien de nouveau dans la vue. Une catégorie se déduit de la seule couleur, déjà visible (`PlayerView::face_down_defenders`,
`PlayerView::ruins`) : nommer « gardes de forteresse » plutôt que « gris » ne révèle pas l'ennemi.

## Libellés (i18n)

Clés modifiées :

| Clé | fr | en |
|---|---|---|
| `enemy.face_down` | défenseur face cachée ({group}) | face-down defender ({group}) |
| `board.hover.ruins.enemies` | Ennemis : {groups} ; récompense : {reward} | Enemies: {groups}; reward: {reward} |
| `board.hover.city.face_down` | Défenseurs face cachée : {groups} | Face-down defenders: {groups} |
| `adventure.confirm.fight_drawn` | Combat contre {count} ennemi(s) tiré(s) au hasard ({group}). | Fight {count} enemy(ies) drawn at random ({group}). |
| `adventure.confirm.fight_colors` → `adventure.confirm.fight_groups` | Combat contre des ennemis tirés au hasard : {groups}. | Fight enemies drawn at random: {groups}. |
| `combat.summon_warning` | Au début de la phase de parade, {enemies} invoquent une créature (donjon) : pas de retour en arrière après. | At the start of the Block phase, {enemies} summon a creature (dungeon): no undo past that point. |
| `burn.confirm` | Incendier le monastère ? Réputation −3, combat contre un ennemi tiré au hasard (tour de mage), sans vos unités. Pas de retour en arrière. | Burn the monastery? Reputation −3, combat against an enemy drawn at random (mage tower), without your Units. This cannot be undone. |

Les paramètres `{color}` et `{colors}` deviennent `{group}` et `{groups}` : la valeur n'est plus une couleur.
`fight_colors` est renommée pour la même raison.

Clé réutilisée : `enemy.color.*` (tableau des catégories ci-dessus), avec ses valeurs actuelles. Texte obtenu, par
exemple `enemy.face_down` avec `{group}` = `enemy.color.grey` : « défenseur face cachée (gardes de forteresse) »,
« face-down defender (keep) ». La parenthèse évite l'accord : « défenseur gardes de forteresse » ne se dit pas.

Clés supprimées : le groupe `enemy.pile_color` (six clés), qui n'a plus de lecteur.

## Interface

- **Textes** des infobulles et des confirmations (C1 à C7), lus dans `PlayerView::ruins`,
  `PlayerView::face_down_defenders` et la pile du site, comme aujourd'hui.
- **Socle des figurines d'ennemis** sur le plateau (C9 à C12) : la couleur vient de `EnemyToken::color()` ou de
  `PlayerView::face_down_defenders`. Forme, taille et lisibilité de jour, de nuit et sur chaque terrain :
  vérification à l'œil.
- **Socle sur la carte d'ennemi** de l'écran de combat (C13, si Q3), même couleur ; placement : vérification à l'œil.
- **Pastilles du jeton de ruines** (spec 045, `RuinPip::Enemy`) : inchangées, déjà à la couleur de la pile.

Parties préparées existantes, sans partie nouvelle (`docs/verification-manuelle.md`) :

| Partie | Ce qu'on y vérifie |
|---|---|
| `forteresse` | infobulle du défenseur face cachée (C3) et socle gris sous le brouillard (C11) |
| `ruines-unite` | infobulle et confirmation de la ruine (C1, C2) |
| `ruines-nuit` | socles de nuit, assombris comme le héros |
| `sombre-terrier` | deux socles marron réduits (C10), confirmation (C5) |
| `cite-bleue` | infobulle des défenseurs face cachée (C4), socles blancs et violets autour de la cité |
| `monastere` | confirmation de l'incendie (C7) |
| `combat-deux-ennemis` | socle sur les cartes d'ennemis (C13, si Q3) |

### Raccourcis clavier

Aucun.

## Hors périmètre

- Plusieurs monstres sur une même case du plateau (issue #28) et icônes de capacité des monstres (issue #27).
- Noms des catégories eux-mêmes : repris tels quels de la spec 070, sauf réponse contraire à Q1.

## Questions ouvertes

- **Q1 — Noms des catégories.** Je propose de garder ceux de l'infobulle depuis la spec 070 (« orques maraudeurs »,
  « gardes de forteresse », « tour de mage », « donjon », « draconien », « cité » ; en anglais « keep », « mage
  tower », « dungeon »…). Ils se lisent bien dans « Ennemis : donjon, draconien » ; « tour de mage » et « cité »
  restent des noms de site plutôt que de troupes. Autre choix possible : des noms de troupes partout
  (« défenseurs de tour de mage », « monstres de donjon », « garnison de cité »), plus longs dans les infobulles.
- **Q2 — Forme du socle.** Je propose un disque plein à la couleur de la pile, cerné comme celui du héros : il se
  voit sur tous les terrains, et un anneau seul se perdrait dans la forêt et la nuit. Le héros garde son socle à sa
  couleur ; Arythea (rouge) et Goldyx (vert) ont un socle proche de celui des draconiens et des orques, mais leur
  figurine les distingue.
- **Q3 — Écran de combat.** Je propose d'y mettre aussi le socle sous la figurine de chaque carte d'ennemi (C13),
  pour garder le même repère qu'au plateau ; sinon, C13 sort de la spec.
