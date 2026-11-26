# #27 — Icônes des capacités des ennemis

- Statut : à valider
- Issue : #27 (milestone v0.2.0, Effort Medium : spec codée en un fil)
- Dépend de : spec 033 (capacités des ennemis), spec 032 (éléments et résistances), spec 071 (icônes), spec 073 (boutons
  d'effet), spec 062 (capacités données par une cité) ; toutes implémentées
- Livret : aucune règle nouvelle. MKUE p. 24, *Enemy Token Abilities* (icônes du jeu, dont les nôtres s'inspirent sans les
  reprendre)
- Crates touchées : arcanum-ui, `assets/images/ui/` (6 SVG nouveaux), `assets/i18n/` (fr.json et en.json). Rien dans
  arcanum-core.
- Maquette : [planche des icônes](images/issue-27-icones/planche.png) et les SVG proposés dans
  [`images/issue-27-icones/`](images/issue-27-icones/)

## Objectif

Sur la carte d'un ennemi de l'écran de combat, chaque capacité (Fortifié, Vif, Brutal, Toxique, Paralysant, Invocation)
s'affiche en icône, comme les résistances depuis la spec 071, au lieu de la ligne « Capacités : vif, toxique ». Au survol, l'icône
donne son nom en titre, puis la règle de la capacité en une ou deux phrases : le joueur n'a plus besoin du livret pour savoir ce
que fait un Paralysant. La Parade des Gardes d'Utem (« Parade 4 ; compte double contre un ennemi Vif ») montre sur son bouton
« ×2 si » suivi de l'icône Vif.

## Règles

Aucune règle n'est ajoutée ni changée : seul l'écran montre autrement ce que le moteur applique déjà.

- **Capacités des ennemis du jeu de base** (MKUE p. 24, *Offensive* et *Defensive* ; spec 033) : Fortifié, Vif, Brutal, Toxique,
  Paralysant et l'attaque d'invocation. Ce sont les seules que portent les ennemis du jeu (`enemies.ron`). Les autres capacités de
  la p. 24 (Assassinat, Encombrant, Vampire, Insaisissable, Non fortifié, Immunité arcanique, Défense) n'existent que sur des
  ennemis des extensions : pas d'icône ici (Q1).
- **Icônes** (MKUE p. 24) : le livret en montre une par capacité (tour, oiseau aux ailes déployées, tête de démon, goutte verte,
  silhouette pétrifiée, cercle à griffe). Les nôtres en reprennent l'idée, pas le dessin (CLAUDE.md, « Assets et droits » ;
  `.claude/rules/arcanum-ui.md`) : SVG dessinés par nous, au style de la planche de la spec 071 (64 × 64, contour `#20242A`).
- **Texte des règles en infobulle** : écrit par nous, court, d'après la p. 24 et ce que le moteur applique (spec 033 ; spec 042
  pour les unités ; spec 053 pour les effets rouges et bleus contre les résistances). Il ne traduit pas le livret (voir
  Libellés).
- **Capacités données par une cité** (MKUE p. 10, *City Assaults* 1.c et 1.d ; spec 062) : Brutal ou Toxique pour les défenseurs à
  attaque physique. Même icône, signalée « (cité) » dans le titre de l'infobulle, comme aujourd'hui dans la ligne de texte (Q6).

## Modèle

Aucun terme nouveau au glossaire : Fortifié, Vif, Brutal, Toxique, Paralysant, Invocation, Résistance y sont déjà (termes de la
FAQ française).

Rien de nouveau dans l'état ni dans la vue : les capacités se lisent déjà dans la vue (`PlayerView::combat`, `CombatEnemy`,
définition du jeton, `CombatEnemy::city_abilities`). Ce que l'écran gagne :

- **Icône** (`Icon`) : `Icon::Ability(EnemyAbility)` pour les cinq capacités et `Icon::Summon` pour l'invocation, chacune avec
  son SVG `assets/images/ui/<nom>.svg` (`ability_fortified`, `ability_swift`, `ability_brutal`, `ability_poison`,
  `ability_paralyze`, `summon`) et sa clé de nom (`icon.ability.<capacité>`, `icon.summon`). La planche nommée (`Icon::all`)
  passe de 26 à 32 icônes, les images de `ui/` de 38 à 44.
- **Règle d'une valeur à icône** : `IconValue` gagne `rule: Option<String>`, la règle affichée sous le nom dans l'infobulle.
  `None` partout où rien ne change (Mouvement, Armure, résistances des unités…).
- **Carte d'ennemi** (`EnemyCard`) : `abilities` devient une liste de valeurs à icône (`Vec<IconValue>`, sans valeur, avec
  règle), au lieu de `Option<String>` ; de même pour `SummonedCard::abilities`.
- **Condition d'un bouton d'effet** : le visage à icône d'un effet (`EffectFace::Icon`) peut porter une condition, un court texte
  suivi d'une icône (« ×2 si » et Vif) ; seule la Parade vive (`Effect::SwiftBlock`) en a une (Q4).

## Actions et événements

Aucune action ni aucun événement nouveau.

## Comportements attendus

Les capacités montrées sont exactement celles d'aujourd'hui, dans le même ordre (celles du jeton dans l'ordre de `enemies.ron`,
puis celles de la cité, `enemy_abilities` de `screen/combat.rs`) : seule leur forme change.

### Icônes et noms

- **C1** — Chaque image de `ui/` existe et est un SVG : 44 images, dont les 6 nouvelles (test existant modifié :
  `every_icon_image_exists`, `icones.rs`, 38 → 44).
- **C2** — Chaque icône a son nom en français : 32 icônes ; `Icon::Ability(EnemyAbility::Swift)` se nomme « Vif »,
  `Icon::Summon` « Invocation » (test existant modifié : `every_icon_has_a_name`, `icones.rs`, 26 → 32).

### Carte d'ennemi de l'écran de combat

- **C3** — Glaceurs (enemies.ron, Freezers : armure 7, attaque de glace 3, Fortifié, Vif, Paralysant, résistance au feu, gloire
  7) : trois icônes de capacité, dans l'ordre Fortifié, Vif, Paralysant, sans valeur ; leurs titres « Fortifié », « Vif »,
  « Paralysant » ; chacune a sa règle (`abilities_use_icons`).
- **C4** — La règle de chaque capacité est celle de la section Libellés : Vif des Chevaucheurs de loups (WolfRiders : armure 4,
  attaque 3, Vif) a pour règle « Pour le parer, il faut une Parade du double de son attaque. » (`ability_tooltip_gives_rule`).
- **C5** — Un ennemi sans capacité (Rôdeurs, Prowlers : armure 3, attaque 4, gloire 2) n'a aucune icône de capacité
  (`no_ability_no_icon`).
- **C6** — L'attaque d'un Invocateur (Invocateurs orques, OrcSummoners : armure 4, invocation, gloire 4) s'affiche avec l'icône
  Invocation, sans valeur, au lieu du mot « invocation » ; titre « Invocation » et sa règle (test existant modifié :
  `summoner_attack_stays_text`, `icones.rs`, spec 071 C9, renommé `summoner_attack_uses_icon`).
- **C7** — La créature invoquée (Minotaure, Minotaur : armure 5, attaque 5, Brutal, gloire 4) montre l'icône Brutal dans sa carte,
  et la carte d'un Minotaure ordinaire aussi (test existant modifié : `block_phase_shows_summoned_monster_and_stakes`,
  `deroule_combat.rs`, lignes 1064 à 1068, qui lisaient « Capacités : brutal »).
- **C8** — En cité rouge, les Gardiens (Guardsmen : armure 7, attaque physique 3, Fortifié) montrent Fortifié puis Brutal ; le
  titre de Brutal est « Brutal (cité) », sa règle celle de Brutal. Hors de la cité rouge, le titre est « Brutal » sans « (cité) »
  (tests existants modifiés : `cites.rs`, lignes 386 et 396 à 399, qui lisaient le texte « brutal (cité) ») — spec 062.
- **C9** — Comme aujourd'hui (spec 054), un ennemi qui a perdu sa fortification garde son icône Fortifié ; c'est son statut qui dit
  « plus fortifié » (`combat.enemy.no_fortification`). Les résistances perdues, elles, disparaissent déjà des icônes de
  résistance (`resistance_icons` lit `effective_resistances`).

### Règle des résistances et des attaques des ennemis (Q2)

- **C10** — Sur la carte d'ennemi, chaque icône de résistance garde son nom en titre et gagne sa règle (Gardiens d'Altem,
  AltemGuardsmen : résistances au feu, à la glace et physique) (test existant modifié : `resistances_use_icons`, `icones.rs`, qui
  compare les infobulles).
- **C11** — Sur la carte d'ennemi, l'icône d'une attaque de feu, de glace ou de feu glacé gagne sa règle (Mages de feu,
  FireMages : attaque de feu 6 → « Seules les Parades de glace et de feu glacé comptent en entier ; les autres, à moitié. ») ;
  une attaque physique n'en a pas (test existant modifié : `enemy_card_values_use_icons`, `icones.rs`, spec 071 C8).
- **C12** — Ailleurs, rien ne change : les résistances des unités (`panel_lists_units`, `unites.rs` ; `unit_line_uses_icons`, `icones.rs`) et
  les attaques des cartes et des unités n'ont pas de règle (`rule` vaut `None`).

### Bouton de la Parade vive

- **C13** — En phase de parade, le bouton de la Parade des Gardes d'Utem (UtemGuardsmen, deuxième capacité : `SwiftBlock(4)`)
  montre la rondache, « 4 », puis « ×2 si » et l'icône Vif, qui a son infobulle (titre et règle) ; son texte de repli est
  « Parade 4 ×2 si Vif » (`swift_block_button_shows_condition`). Idem pour les Gardiens d'Altem (`SwiftBlock(8)`).
- **C14** — Comme aujourd'hui, l'infobulle du bouton reste « Parade 4, double contre Vif » (`effect.swift_block`) et la Parade
  compte double contre un ennemi vif (spec 033 ; test `swift_block_counts_doubled_value`, `combat_et_offres.rs`, inchangé).

### Ce qui ne change pas

- **C15** — L'infobulle d'un ennemi sur le plateau garde ses lignes de texte « Capacités : brutal » et « Résistances :
  physique » (test `ennemis_errants.rs`, ligne 201, inchangé) (Q3).
- **C16** — Les lignes d'enjeu « À parer : 6 (vif, 2 × 3) » et « Si non paré : 8 (brutal, 2 × 4) dégâts… », et la ligne « Vif :
  il faut parer le double… » de la parade en cours restent en texte (tests `swift_block_shows_doubled_target`,
  `deroule_combat.rs` ; `cites.rs`, ligne 380, inchangés) (Q4).

## Cas limites et refus

- **L1** — Sans fichier d'icône, l'écran écrit le nom de la capacité, comme pour les autres icônes (spec 071, L1 ;
  `IconValue::fallback_text`).
- **L2** — Un ennemi vaincu : ses icônes restent, grisées avec la carte (vérification à l'œil).
- **L3** — Les Glaceurs défendant une cité sont doublement fortifiés (spec 062) : une seule icône Fortifié (celle du jeton) ; la
  fortification du site reste dans le statut « doublement fortifié : pas de cible à distance », comme aujourd'hui.

## Information cachée

Rien de nouveau : les icônes ne montrent que les capacités d'un ennemi déjà visible dans le combat. Un défenseur face cachée
n'apparaît pas sur l'écran de combat.

## Libellés (i18n)

Clés nouvelles. Les textes des règles sont les nôtres (courts, d'après la p. 24 et le moteur), en français et en anglais : ils
ne reprennent pas la phrase du livret.

| Clé | fr | en |
|---|---|---|
| `icon.ability.fortified` | Fortifié | Fortified |
| `icon.ability.swift` | Vif | Swift |
| `icon.ability.brutal` | Brutal | Brutal |
| `icon.ability.poison` | Toxique | Poison |
| `icon.ability.paralyze` | Paralysant | Paralyze |
| `icon.summon` | Invocation | Summon |
| `icon.rule.fortified` | Pendant les attaques à distance, seules les attaques de siège l'atteignent ; aucune s'il défend aussi un site fortifié. | During ranged attacks, only siege attacks can reach it; none at all if it also defends a fortified site. |
| `icon.rule.swift` | Pour le parer, il faut une Parade du double de son attaque. | Blocking it takes Block equal to twice its attack. |
| `icon.rule.brutal` | S'il n'est pas paré, il inflige le double de son attaque. | If not blocked, it deals twice its attack in damage. |
| `icon.rule.poison` | Une unité qu'il blesse reçoit deux Blessures. Pour chaque Blessure qu'il met dans la main du héros, une autre va dans la défausse. | A unit it wounds receives two Wounds. For each Wound it puts into the hero's hand, another one goes to the discard pile. |
| `icon.rule.paralyze` | Une unité qu'il blesse est détruite. Si le héros prend une Blessure en main, il défausse toutes ses cartes qui ne sont pas des Blessures. | A unit it wounds is destroyed. If the hero takes a Wound into their hand, they discard every card in hand that is not a Wound. |
| `icon.rule.summon` | Au début de la phase de parade, une créature tirée de la pile du donjon prend sa place pour la parade et les dégâts, puis est défaussée. | At the start of the block phase, a creature drawn from the dungeon pile takes its place for blocking and damage, then is discarded. |
| `icon.rule.resistance_physical` | Les attaques physiques, cartes jouées de côté comprises, ne comptent qu'à moitié contre lui. | Physical attacks, sideways cards included, only count half against it. |
| `icon.rule.resistance_fire` | Les attaques de feu ne comptent qu'à moitié contre lui, et le feu glacé aussi s'il résiste à la glace. Il ignore les autres effets des cartes rouges et des capacités d'unité payées en mana rouge. | Fire attacks only count half against it, and cold fire too if it also resists ice. It ignores other effects of red cards and of unit abilities paid with red mana. |
| `icon.rule.resistance_ice` | Les attaques de glace ne comptent qu'à moitié contre lui, et le feu glacé aussi s'il résiste au feu. Il ignore les autres effets des cartes bleues et des capacités d'unité payées en mana bleu. | Ice attacks only count half against it, and cold fire too if it also resists fire. It ignores other effects of blue cards and of unit abilities paid with blue mana. |
| `icon.rule.attack_fire` | Seules les Parades de glace et de feu glacé comptent en entier ; les autres, à moitié. | Only ice and cold fire blocks count in full; others count half. |
| `icon.rule.attack_ice` | Seules les Parades de feu et de feu glacé comptent en entier ; les autres, à moitié. | Only fire and cold fire blocks count in full; others count half. |
| `icon.rule.attack_cold_fire` | Seules les Parades de feu glacé comptent en entier ; les autres, à moitié. | Only cold fire blocks count in full; others count half. |
| `effect.double_if` | ×2 si | ×2 vs |

Vérifications faites sur `main` :

- `icon.ability` et `icon.rule` sont des groupes nouveaux du groupe `icon` (qui n'a ni `ability` ni `rule`) ; `icon.summon` est
  un libellé à côté de `icon.ranged_or_siege`. `effect.double_if` est un libellé du groupe `effect`, à côté de
  `effect.swift_block`. Aucune clé fille sous un libellé.
- « × » (U+00D7) s'affiche déjà dans le jeu : `combat.enemy.swift_block` = « {block} (vif, 2 × {attack}) ».
- **Réutilisée** : `combat.enemy.city_ability` = « {ability} (cité) » / « {ability} (city) », avec `{ability}` = « Brutal »
  (`icon.ability.brutal`) : « Brutal (cité) », « Brutal (city) » ; avec « Toxique » : « Toxique (cité) », « Poison (city) ».
  Elle servait au texte en minuscules (« brutal (cité) ») : la valeur convient aussi en titre.
- **Plus lue par l'écran de combat** : `enemy.abilities` (« Capacités : {list} ») reste lue par l'infobulle du plateau
  (`token_lines`, C15) ; `enemy.summon` (« invocation ») aussi (`attack_text`). Aucune clé ne disparaît.

## Interface

- **Capacités en icônes** sur la carte d'ennemi, en une rangée sous ses valeurs et ses résistances, à la place de la ligne
  « Capacités : … » (C3, C5, C8 ; `EnemyCard::abilities`, lu de `CombatEnemy` dans `PlayerView::combat`). Placement :
  vérification à l'œil.
- **Idem dans la carte de la créature invoquée** (C7 ; `SummonedCard::abilities`).
- **Icône Invocation** à la place du mot « invocation » parmi les valeurs de l'Invocateur (C6).
- **Infobulle à deux étages** sur ces icônes : le nom en gras, puis la règle à la ligne, sur une largeur limitée pour qu'elle
  passe à la ligne (C4, C10, C11 ; `IconValue::rule`). Mise en forme : vérification à l'œil.
- **« ×2 si » et l'icône Vif** après la valeur du bouton de la Parade vive, l'icône avec sa propre infobulle (C13). Taille et
  alignement dans le bouton : vérification à l'œil.
- **Dessins** des six icônes : ceux de la [planche](images/issue-27-icones/planche.png), copiés de
  `docs/specs/images/issue-27-icones/` dans `assets/images/ui/` (C1 ; lisibilité à 16 et 20 px sur fond clair et sombre :
  vérification à l'œil).

Parties préparées pour la vérification à l'œil (`docs/verification-manuelle.md`) :

- `combat-capacites` (existante) : défier les Chevaucheurs de loups et les Invocateurs orques. Icônes Vif et Invocation, puis en
  phase de parade le Minotaure invoqué avec Brutal ; infobulles.
- `combat-elements` (existante) : Cogneurs (Brutal, résistance physique) et Dragons de glace (attaque de glace, Paralysant,
  résistances à la glace et physique) ; règles des résistances et de l'attaque de glace.
- `cite-bleue` (existante) : assaut, Glaceurs avec Fortifié, Vif, Paralysant (L3).
- `unites-vif` (nouvelle) : comme `unites`, mais des Gardes d'Utem prêts dans la zone des unités et la figurine voisine des
  Chevaucheurs de loups (3, −2) ; défier, passer en phase de parade, déclarer la parade : le bouton « Parade 4 ×2 si [Vif] » (C13).

### Raccourcis clavier

Aucun : l'écran n'ajoute aucune action.

## Hors périmètre

- Icônes des capacités des extensions (Assassinat, Encombrant, Vampire, Insaisissable, Non fortifié, Immunité arcanique,
  Défense) et des attaques multiples : avec les premiers ennemis des extensions (Q1).
- Icônes dans l'infobulle du plateau et dans les lignes d'enjeu de la parade (Q3, Q4).
- Règles en infobulle des résistances des unités (p. 24, *Unit Resistances*), qui ne sont pas celles des ennemis.
- Textes des cartes et des unités qui nomment une capacité (« Un ennemi non fortifié n'attaque pas », « perd ses
  fortifications ») : ils restent en mots (Q4).

## Questions ouvertes

Chaque question a une recommandation ; la spec est écrite avec elle.

- **Q1 — Quelles capacités ?** Recommandé : les cinq des ennemis du jeu (Fortifié, Vif, Brutal, Toxique, Paralysant) et
  l'Invocation. L'issue cite « assassin » : l'Assassinat n'existe que sur des ennemis des extensions, absents du jeu ; dessiner
  son icône maintenant ne servirait à rien. Autre choix : dessiner aussi les sept capacités des extensions dès maintenant.
- **Q2 — La règle en infobulle aussi pour les résistances et les attaques élémentaires des ennemis ?** La p. 24 les range avec
  les capacités. Recommandé : oui, sur la carte d'ennemi seulement (C10, C11) ; ailleurs, la même icône garde son nom seul
  (C12), car la règle d'une résistance d'unité n'est pas la même. Autre choix : capacités seulement.
- **Q3 — Infobulle du plateau ?** Recommandé : la garder en texte (C15). Une infobulle ne peut pas en ouvrir une autre : une icône
  y serait muette, alors que le mot se lit. Autre choix : l'icône suivie du nom (« [Vif] vif »), qui demande de passer les lignes
  de survol du plateau de texte à morceaux.
- **Q4 — Jusqu'où généraliser « ×2 si [Vif] » ?** Recommandé : la seule Parade vive (Gardes d'Utem, Gardiens d'Altem), seul effet
  dont la valeur dépend d'une capacité. Les autres textes qui nomment une capacité (Illusionnistes, sorts, lignes d'enjeu)
  restent en mots (C16). Autre choix : remplacer aussi « vif » et « brutal » des lignes d'enjeu par leur icône.
- **Q5 — Les dessins de la planche conviennent-ils ?** Tour crénelée (Fortifié), oiseau aux ailes déployées (Vif), tête de démon
  cornue (Brutal), goutte verte (Toxique), silhouette de pierre fissurée (Paralysant), cercle brun à trois griffes (Invocation).
  Recommandé : les garder ; dites lesquels retoucher sinon.
- **Q6 — Repérer une capacité donnée par la cité ?** Recommandé : même icône, « (cité) » dans le titre de l'infobulle seulement
  (C8). Autre choix : un petit repère sur l'icône (contour de la couleur de la cité), vérifié à l'œil.
