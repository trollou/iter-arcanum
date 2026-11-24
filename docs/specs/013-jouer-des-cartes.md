# 013 — Jouer des cartes

- Statut : implémentée (2026-10-25)
- Jalon : J1
- Dépend de : 000 (noyau du moteur), 002 (boucle de partie et fenêtre), 012 (scénario et mise en place, héros dans la configuration)
- Livret : MKUE p. 3 (deck et main de départ), p. 4 (cartes : effet de base, de côté, cumul, défausser), p. 5 (effets),
  p. 6 (quels effets se jouent hors combat)
- Contenu : `docs/reference/decks-de-depart.md` et scans `docs/regles/scans/deed/` (effets repris, textes réécrits)
- Crates touchées : arcanum-core, arcanum-ui, `assets/i18n/` (fr.json et en.json)

## Objectif

Le jeu jouet (cartes numérotées, bouton « Piocher ») disparaît. Le héros a son **vrai deck de départ** de 16 cartes d'action de base,
mélangé à la mise en place, et commence avec **5 cartes en main**. Le joueur peut **jouer une carte** de sa main, pour son **effet de
base** ou **de côté** (Mouvement 1 ou Influence 1), et voit ses **points de mouvement** et **d'influence** s'additionner dans sa **zone
de jeu**. Pas de mana, pas de combat, pas encore de déplacement : les points s'accumulent sans servir, ce sera le rôle des specs 014 à 016.

## Règles

- **Deck et main de départ** (MKUE p. 3) : 16 cartes d'action de base mélangées, 5 cartes piochées (limite de main au niveau 1).
  Composition par héros : `docs/reference/decks-de-depart.md`.
- **Jouer une carte** (MKUE p. 4, 2 et 3.a) : dans sa zone de jeu, pour son effet de base. La **couleur** d'une carte d'action est celle
  de son effet renforcé. L'effet renforcé est hors périmètre (spec 022).
- **De côté** (MKUE p. 4, 3.e) : toute carte sauf une Blessure, pour Mouvement 1, Influence 1, Attaque 1 ou Parade 1.
- **Choix** (texte des cartes ; MKUE p. 5, « Effects », 1.a) : un effet « Attaque ou Parade 2 » donne **une** des options, choisie quand
  on joue la carte.
- **Cumul** (MKUE p. 4, 4) : Marche (Mouvement 2) + une carte de côté (Mouvement 1) = 3 points de mouvement.
- **Quand** (MKUE p. 6, 5.c ; p. 8-10) : **hors combat, Attaque et Parade ne sont pas jouables** (Q1). Il n'y a pas encore de combat
  (spec 031) : on ne peut donc jouer que du Mouvement et de l'Influence.
- **Coûts** (MKUE p. 4, 6 ; p. 5, « Discarding ») : un effet qui demande de défausser une autre carte n'est pas jouable sans carte à
  défausser, et une Blessure ne peut pas être défaussée ainsi. Concerne ici Improvisation.
- **Fin du tour** (MKUE p. 4, 5 ; p. 6, 6.e) : défausse des cartes jouées, spec 016 ; points non dépensés perdus, specs 014 et 016.

### Les cartes et leur effet de base

Effets lus sur les scans (`docs/regles/scans/deed/`). « Jouable » = jouable pour son effet de base avec cette spec. Toutes les cartes
restent jouables de côté.

| Carte (EN) | Couleur | Effet de base | Jouable |
|---|---|---|---|
| March | vert | Mouvement 2 | oui |
| Stamina | bleu | Mouvement 2 | oui |
| Swiftness | blanc | Mouvement 2 | oui |
| Promise | blanc | Influence 2 | oui |
| Threaten | rouge | Influence 2 | oui |
| Rage | rouge | Attaque 2 ou Parade 2 | non (combat, spec 031) |
| Determination | bleu | Attaque 2 ou Parade 2 | non (combat) |
| Improvisation | rouge | Défausser une autre carte de sa main : Mouvement 3, Influence 3, Attaque 3 ou Parade 3 | oui, pour Mouvement 3 ou Influence 3 |
| Tranquility | vert | Soin 1, ou piocher une carte | non (spec 034) |
| Crystallize | bleu | Payer une mana d'une couleur de base, gagner un cristal de cette couleur | non (spec 021) |
| Mana Draw | blanc | Utiliser un dé de mana de plus ce tour | non (spec 020) |
| Concentration | vert | Gagner une mana bleue, blanche ou rouge | non (spec 020) |
| Battle Versatility (Arythea) | rouge | Attaque 2, Parade 2 ou Attaque à distance 1 | non (combat) |
| Mana Pull (Arythea) | blanc | Utiliser un dé de mana de plus ce tour ; s'il est noir, il donne une mana de n'importe quelle couleur | non (spec 020) |
| Will Focus (Goldyx) | vert | Gagner une mana bleue, blanche ou rouge, ou un cristal vert | non (specs 020, 021) |
| Crystal Joy (Goldyx) | bleu | Payer une mana, gagner un cristal ; en fin de tour, défausser une autre carte pour la reprendre en main | non (specs 021, 016) |
| Noble Manners (Norowas) | blanc | Influence 2 ; si utilisée pendant une interaction, Gloire +1 en fin de tour | oui, pour Influence 2 (le bonus : spec 040, implémentée) |
| Rejuvenate (Norowas) | vert | Soin 1, piocher une carte, gagner une mana verte ou préparer une unité de niveau I ou II | non (specs 034, 020, 042) |
| Cold Toughness (Tovak) | bleu | Attaque 2 ou Parade de glace 3 (version révisée) | non (combat) |
| Instinct (Tovak) | rouge | Mouvement 2, Influence 2, Attaque 2 ou Parade 2 | oui, pour Mouvement 2 ou Influence 2 |

Les effets non jouables sont tout de même **décrits** dans le contenu quand ils sont faits de points (Attaque, Parade, Attaque à
distance, Attaque de glace, Parade de glace) : la spec 031 n'aura qu'à les rendre jouables. Les autres (mana, cristal, soin, pioche,
dé, unité) sont marqués « pas encore pris en charge » et seront décrits par la spec qui les apporte.

## Modèle

### Termes

Termes du glossaire déjà présents : Deck (`DeedDeck`), Défausse (`DiscardPile`), Limite de main (`hand_limit`), Mouvement (`Move`),
Influence (`Influence`), Attaque (`Attack`), Attaque à distance (`RangedAttack`), Parade (`Block`), Glace (`Element::Ice`),
Rouge / Bleu / Blanc / Vert (`BasicColor`), Blessure (`Wound`), Héros (`Hero`).

Nouveaux termes, ajoutés au glossaire dans la même PR (à valider) :

| Livret (EN) | Français | Code |
|---|---|---|
| Deed card | Carte | `DeedCard` |
| Basic Action card | Carte d'action de base | `BasicAction` |
| Hand | Main | `hand` |
| Play area | Zone de jeu | `PlayArea` |
| Basic effect | Effet de base | `CardPlay::Basic` |
| Strong effect | Effet renforcé | (spec 022) |
| Played sideways | Jouée de côté | `CardPlay::Sideways` |
| Move points | Points de mouvement | `move_points` |
| Influence points | Points d'influence | `influence_points` |
| Discard (a card) | Défausser | `CardDiscarded` |
| Ice Attack / Ice Block | Attaque de glace / Parade de glace | `Attack` / `Block` avec `Element::Ice` |
| (card) color | Couleur (d'une carte) | `BasicColor` |

Noms des 20 cartes (Q2) : l'identifiant de code reprend le nom anglais, le nom français est celui de l'édition française, relevé
sur les cartes de Mathieu (photos du 2026-10-25).

| Livret (EN) | Français | Code (`BasicAction::`) |
|---|---|---|
| Rage | Rage | `Rage` |
| Determination | Détermination | `Determination` |
| Swiftness | Célérité | `Swiftness` |
| March | Marche | `March` |
| Stamina | Endurance | `Stamina` |
| Tranquility | Tranquillité | `Tranquility` |
| Promise | Promesse | `Promise` |
| Threaten | Menace | `Threaten` |
| Crystallize | Cristallisation | `Crystallize` |
| Mana Draw | Source de mana | `ManaDraw` |
| Concentration | Concentration | `Concentration` |
| Improvisation | Improvisation | `Improvisation` |
| Battle Versatility | Polyvalence | `BattleVersatility` |
| Mana Pull | Attraction de mana | `ManaPull` |
| Will Focus | Pouvoir de concentration | `WillFocus` |
| Crystal Joy | Euphorie cristalline | `CrystalJoy` |
| Noble Manners | Nobles manières | `NobleManners` |
| Rejuvenate | Rajeunissement | `Rejuvenate` |
| Cold Toughness | Endurance givrée | `ColdToughness` |
| Instinct | Instinct | `Instinct` |

Endurance givrée : le scan anglais (édition originale) donne « Ice Attack 2 or Ice Block 3 » ; la carte a été révisée avec
l'extension Lost Legion en « Attaque 2 ou Parade de glace 3 », texte de la carte française Ultimate. On retient la version révisée
(décision de Mathieu, 2026-10-25).

### Contenu (arcanum-core, en Rust)

- Chaque carte d'action de base a une **définition** : sa couleur et son effet de base. Un effet de base est soit une **liste
  d'options** (une seule pour Marche, plusieurs pour Rage), chacune un **effet de points** (Mouvement n, Influence n, Attaque n,
  Attaque à distance n, Parade n, avec un élément pour l'attaque et la parade), avec au besoin un **coût** (« défausser une autre
  carte »), soit **pas encore pris en charge**.
- Chaque héros donne son **deck de départ** : la liste de ses 16 cartes, selon `decks-de-depart.md`.
- Un **identifiant de carte** (`CardId`) désigne un exemplaire : la carte et son numéro d'exemplaire (Marche n° 1, Marche n° 2). Il
  remplace le numéro opaque du jeu jouet ; l'identifiant suffit à connaître la carte, sans table dans l'état.

### Configuration de départ

`GameConfig` perd la liste de cartes du jeu jouet : le deck vient du héros. Configuration = graine, scénario, héros.

### Nouvelles données dans l'état (par joueur)

- la **zone de jeu** : les cartes jouées ce tour, dans l'ordre, chacune avec la façon dont elle a été jouée ;
- les **points de mouvement** et les **points d'influence** du tour ;
- la **défausse** : suite de cartes.

## Actions et événements

L'action `DrawCard` du jeu jouet est **supprimée**.

| Action | Validation (refusée si…) | Événements produits |
|---|---|---|
| `PlayCard { card, play }`, avec `play` = `Basic { option, discard }` ou `Sideways(effet)` | la carte n'est pas en main ; option inexistante ou pas jouable hors combat ; effet de côté autre que Mouvement ou Influence ; carte non prise en charge jouée pour son effet de base ; `discard` absent alors que l'option le demande, présent alors qu'elle ne le demande pas, égal à la carte jouée ou pas en main | `CardPlayed`, puis `CardDiscarded` si l'option défausse une carte |

| Événement | Effet de `apply` | Révèle une information ? |
|---|---|---|
| `CardPlayed { player, card, play, effect }` | retire `card` de la main, la met dans la zone de jeu avec `play` ; ajoute `effect` (ex. Mouvement 2) aux points du tour | oui, publique : la carte jouée et l'effet |
| `CardDiscarded { player, card }` | retire `card` de la main et la met sur la défausse | oui, publique |
| `CardDrawn { player, card }` | inchangé (spec 000) | inchangé (la carte n'est connue que du joueur) |

`CardPlayed` porte l'effet obtenu, déjà résolu (Mouvement 2), pas seulement la carte et l'option : `apply` ne consulte pas le contenu,
et un journal rejoue la même partie même si la définition d'une carte change (même raison que le scénario entier dans la configuration,
spec 012, Q1). Un `CardPlayed` ou `CardDiscarded` dont la carte n'est pas en main ne change rien (`apply` reste infaillible).

**Mise en place.** Après `DeckShuffled`, `new_game` produit **5** `CardDrawn` (limite de main au niveau 1, MKUE p. 3). Ordre des
événements de démarrage : `GameStarted`, `TilePlaced`, `TileDeckFormed`, `TileRevealed` (2 ou 3), `HeroFigurePlaced`, `DeckShuffled`,
`CardDrawn` ×5. Le deck mélangé est celui du héros de la configuration.

**Décision en attente.** Le joueur doit choisir une action tant qu'il en a une légale, c'est-à-dire tant qu'il a une carte en main
(toute carte se joue de côté). Main vide : plus de décision, la boucle de partie s'arrête comme le jeu jouet sur un deck vide, jusqu'à la
fin du tour (spec 016). Voir Q3.

**Actions légales**, énumérées dans cet ordre : pour chaque carte de la main, dans l'ordre de la main, ses options d'effet de base
jouables (et, pour Improvisation, chaque option avec chaque autre carte de la main à défausser, dans l'ordre de la main), puis de côté
Mouvement 1, puis de côté Influence 1. Deux exemplaires d'une même carte donnent chacun leurs actions.

## Comportements attendus

Chaque cas devient au moins un test (nom indicatif entre parenthèses).

Contenu (arcanum-core) :

- **C1** — Le deck de départ de chaque héros a 16 cartes, et sa composition est celle de `decks-de-depart.md` (ex. Arythea : 1 Rage,
  1 Polyvalence, 1 Attraction de mana, 2 Marche…) (`starting_decks_match_hero_cards`) — MKUE p. 3.
- **C2** — Les couleurs et les effets de base des 20 cartes sont ceux du tableau des règles (`basic_action_definitions_match_cards`).
- **C3** — Deux exemplaires d'une même carte ont des identifiants différents ; chaque identifiant redonne sa carte
  (`card_ids_name_a_copy_of_a_card`).

Mise en place (arcanum-core) :

- **C4** — Après la mise en place avec Tovak, la main a 5 cartes, le deck 11, et main + deck = le deck de départ de Tovak
  (`setup_draws_five_cards_from_hero_deck`) — MKUE p. 3.
- **C5** — Les événements de démarrage se terminent par `DeckShuffled` puis 5 `CardDrawn`, qui piochent les 5 premières cartes de
  l'ordre mélangé (`setup_events_end_with_five_draws`).
- **C6** — Deux graines différentes donnent (pour au moins une paire parmi 20) des mains différentes ; une même graine donne les mêmes
  événements (`starting_hand_depends_only_on_seed`).

Jouer une carte (arcanum-core) :

- **C7** — Marche en main : la jouer pour son effet de base donne `CardPlayed` avec Mouvement 2 ; après `apply`, Marche est dans la zone
  de jeu, plus en main, et les points de mouvement valent 2 (`playing_march_gives_two_move_points`) — MKUE p. 4, 2-3.a.
- **C8** — Promesse pour son effet de base : 2 points d'influence (`playing_promise_gives_two_influence_points`) — MKUE p. 4, 3.a.
- **C9** — Une carte jouée de côté pour Mouvement donne 1 point de mouvement ; pour Influence, 1 point d'influence, y compris une carte
  dont l'effet de base n'est pas pris en charge (Concentration) (`any_card_can_be_played_sideways`) — MKUE p. 4, 3.e.
- **C10** — Marche (Mouvement 2), puis Célérité (Mouvement 2), puis Rage de côté (Mouvement 1) : 5 points de mouvement, 0 d'influence,
  trois cartes dans la zone de jeu dans l'ordre joué (`move_effects_add_up`) — MKUE p. 4, 4.
- **C11** — Mouvement et Influence se cumulent séparément : Marche puis Menace donnent 2 points de mouvement et 2 d'influence
  (`move_and_influence_are_kept_apart`) — MKUE p. 4, 4.
- **C12** — Instinct : actions légales de base Mouvement 2 et Influence 2, pas Attaque 2 ni Parade 2 ; Nobles manières : Influence 2
  (`choice_effects_offer_only_move_and_influence_out_of_combat`) — MKUE p. 6, 5.c.
- **C13** — Improvisation et deux autres cartes en main : 4 actions de base (Mouvement 3 ou Influence 3, × 2 cartes à défausser) ; en
  jouer une produit `CardPlayed` (Mouvement 3) puis `CardDiscarded` ; la carte défaussée est sur la défausse, plus en main
  (`improvisation_discards_another_card`) — MKUE p. 4, 6 ; p. 5.
- **C14** — Les actions légales d'une main donnée sont exactement celles de l'énumération, dans son ordre ; chacune est acceptée par
  `decide` (`legal_actions_are_exhaustive`).
- **C15** — Après avoir joué les 5 cartes de la main, il n'y a plus de décision en attente ; la boucle de partie s'arrête
  (`empty_hand_leaves_no_decision`).

Information cachée (arcanum-core) :

- **C16** — La vue joueur contient la main, la taille du deck, la zone de jeu (cartes et façon de jouer), les points de mouvement et
  d'influence, la défausse ; elle ne contient pas l'ordre du deck (`view_shows_play_area_and_points`).
- **C17** — Dans le journal filtré pour un autre joueur, `CardPlayed` et `CardDiscarded` sont montrés en entier, `CardDrawn` cache la
  carte comme avant (`played_cards_are_public`).

Noyau (repris de la spec 000 sur les vraies cartes, voir « Effets sur les specs déjà implémentées ») :

- **C18** — Rejouer le journal après quelques cartes jouées redonne le même état (`replaying_log_rebuilds_state`) ; `simulate` laisse
  l'état d'origine intact (`simulation_leaves_original_untouched`) ; `decide` ne modifie pas l'état
  (`decide_plays_card_without_mutating_state`).

Affichage (arcanum-ui) :

- **C19** — L'écran de partie montre la main : pour chaque carte, son nom traduit, sa couleur, le texte de son effet de base
  (« Mouvement 2 », « Attaque 2 ou Parade 2 », « Pas encore disponible » pour un effet non pris en charge), un bouton par option
  d'effet de base jouable, et un menu « de côté » (`hand_shows_cards_with_their_plays`).
- **C20** — Libellés : « Jouer : Mouvement 2 » (effet de base) ; « Jouer : Mouvement 3… » (effet de base qui demande une défausse) ;
  le menu de côté s'intitule « Jouer de côté » et propose « Mouvement 1 » et « Influence 1 » (`play_buttons_describe_the_effect`).
- **C21** — L'écran montre la zone de jeu (nom des cartes jouées et effet obtenu) et la ligne des points : « Mouvement : 3 · Influence : 1 »
  (`screen_shows_play_area_and_points`).
- **C22** — Cliquer « Jouer : Mouvement 2 » de Marche joue Marche pour son effet de base ; choisir « Influence 1 » dans le menu de côté
  de Rage joue Rage de côté (`clicking_a_play_button_plays_the_card`).
- **C23** — Avec le vrai `fr.json`, calculer l'écran pour une main de chaque héros, avec et sans cartes jouées, et en mode choix de la
  carte à défausser, ne relève aucun manque ; les 20 cartes ont un nom en français et en anglais (`every_card_key_exists_in_french`).
- **C24** — Choix de la carte à défausser : cliquer « Jouer : Mouvement 3… » d'Improvisation ne joue rien encore ; l'écran passe en mode
  choix : un bandeau « Improvisation : Mouvement 3. Choisissez la carte à défausser. » avec un bouton « Annuler » ; Improvisation n'est
  plus listée dans la main ; chaque autre carte n'a plus qu'un bouton « Défausser cette carte » ; pas de menu de côté
  (`discard_choice_lists_the_other_cards`).
- **C25** — En mode choix, cliquer « Défausser cette carte » sur Marche envoie l'action `PlayCard` d'Improvisation (Mouvement 3, défausser
  Marche) et quitte le mode choix (`choosing_a_card_to_discard_plays_improvisation`).
- **C26** — En mode choix, « Annuler » ou la touche Échap quitte le mode sans rien jouer : main, points et défausse inchangés
  (`discard_choice_can_be_cancelled`).

Vérification manuelle : `cargo run -p arcanum-ui` sous Linux (et Windows quand c'est possible) montre 5 cartes d'Arythea en main, le deck à 11 ;
jouer Marche puis une carte de côté affiche 3 points de mouvement et les deux cartes dans la zone de jeu ; une fois la main vide, le
message de fin s'affiche.

## Cas limites et refus

- **L1** — Jouer une carte qui n'est pas en main (déjà jouée, ou dans le deck) : `ActionError::NotLegal`, aucun événement
  (`playing_a_card_not_in_hand_is_refused`).
- **L2** — Rage pour son effet de base (Attaque 2) hors combat : refusé ; Rage de côté pour Attaque 1 ou Parade 1 : refusé
  (`combat_effects_are_refused_out_of_combat`) — MKUE p. 6, 5.c.
- **L3** — Concentration pour son effet de base : refusé, l'effet n'est pas encore pris en charge (`unsupported_basic_effect_is_refused`).
- **L4** — Improvisation seule en main : seulement les deux actions de côté ; Improvisation sans carte à défausser, ou en se défaussant
  elle-même : refusé (`improvisation_needs_another_card`) — MKUE p. 4, 6.
- **L5** — Option d'effet de base inexistante (option 3 de Marche) : refusé (`unknown_option_is_refused`).
- **L6** — `apply` d'un `CardPlayed` ou d'un `CardDiscarded` dont la carte n'est pas en main : état inchangé
  (`invalid_card_event_changes_nothing`).
- **L7** — Pas de Blessure dans le deck de départ : la règle « une Blessure ne se joue pas, ne se défausse pas » n'a encore rien à
  refuser. Elle sera testée avec la spec 034, qui ajoute les Blessures.

## Information cachée

- Cachés : l'ordre du deck (inchangé) ; la main pour les autres joueurs (inchangé, `CardDrawn` filtré).
- Publics : la zone de jeu (cartes, façon de les jouer, effets), les points du tour, la défausse, la taille du deck et de la main.
- La composition du deck de départ est publique (elle découle du héros), pas son ordre.

## Libellés (i18n)

Ajoutés dans `fr.json` et `en.json`. Les clés du jeu jouet (`toy.card.name`, `action.draw_card`) sont **supprimées**.

| Clé | Paramètres | Français | English |
|---|---|---|---|
| `card.<carte>.name` (×20, ex. `card.march.name`) | — | Marche… (tableau des noms) | March… |
| `effect.move` | `amount` | Mouvement {amount} | Move {amount} |
| `effect.influence` | `amount` | Influence {amount} | Influence {amount} |
| `effect.attack` | `amount` | Attaque {amount} | Attack {amount} |
| `effect.ranged_attack` | `amount` | Attaque à distance {amount} | Ranged Attack {amount} |
| `effect.block` | `amount` | Parade {amount} | Block {amount} |
| `effect.ice_attack` | `amount` | Attaque de glace {amount} | Ice Attack {amount} |
| `effect.ice_block` | `amount` | Parade de glace {amount} | Ice Block {amount} |
| `effect.or` | `first`, `second` | {first} ou {second} | {first} or {second} |
| `effect.discard_another` | `effects` | Défausser une autre carte : {effects} | Discard another card: {effects} |
| `effect.unsupported` | — | Pas encore disponible | Not available yet |
| `play.basic` | `effect` | Jouer : {effect} | Play: {effect} |
| `play.basic_discarding` | `effect` | Jouer : {effect}… | Play: {effect}… |
| `play.sideways` | — | Jouer de côté | Play sideways |
| `discard_choice.prompt` | `card`, `effect` | {card} : {effect}. Choisissez la carte à défausser. | {card}: {effect}. Choose the card to discard. |
| `discard_choice.pick` | — | Défausser cette carte | Discard this card |
| `discard_choice.cancel` | — | Annuler | Cancel |
| `play_area.title` | — | Zone de jeu | Play area |
| `play_area.empty` | — | Aucune carte jouée | No card played |
| `play_area.line` | `card`, `effect` | {card} : {effect} | {card}: {effect} |
| `play_area.line_sideways` | `card`, `effect` | {card} : {effect} (de côté) | {card}: {effect} (sideways) |
| `play_area.line_discarding` | `card`, `effect`, `discarded` | {card} : {effect}, {discarded} défaussée | {card}: {effect}, {discarded} discarded |
| `points.line` | `move`, `influence` | Mouvement : {move} · Influence : {influence} | Move: {move} · Influence: {influence} |
| `discard.count` | `count` | Défausse : {count} carte(s) | Discard pile: {count} card(s) |

Plus de deux options (Instinct, Polyvalence) : `effect.or` s'enchaîne (« Mouvement 2 ou Influence 2 ou Attaque 2 ou Parade 2 »).
arcanum-core ne donne que les clés (`BasicAction::label_key()` → `card.march.name`), jamais de texte (ADR 0003). Les noms des couleurs ne sont
pas affichés (la couleur est montrée par une pastille) : pas de clé.

## Interface

Panneau latéral, de haut en bas : nom du scénario et pioche de tuiles (inchangés, spec 012), **ligne des points**, **main**, **zone de jeu**,
taille du deck et de la défausse, bouton « Recentrer » et survol (inchangés).

- **Main** : un bloc par carte, dans l'ordre de la main (maquette validée :
  [Maquette spec 013](https://claude.ai/artifact/4bYU6LfBFeRwgPaJJGqeMA)) :
  - en haut : une pastille de la couleur de la carte (couleurs provisoires, codées dans arcanum-ui), son nom, et à droite un **bouton icône**
    (une carte couchée, infobulle « Jouer de côté ») qui ouvre un **menu déroulant** : Mouvement 1, Influence 1 ;
  - le texte de son effet de base ;
  - un bouton par option d'effet de base jouable (« Jouer : Mouvement 2 ») ; aucun si l'effet de base n'est pas jouable.
  Le bouton « Piocher » disparaît.
- **Choix de la carte à défausser** (Improvisation) : pas de fenêtre modale, le choix se fait dans la main (C24 à C26). La carte jouée
  disparaît de la liste, un bandeau rappelle l'effet choisi et propose « Annuler », chaque autre carte n'a plus que « Défausser cette
  carte ». Ce mode est un état de l'écran (arcanum-ui) : le moteur ne reçoit qu'une action `PlayCard` complète, carte à défausser comprise.
- **Zone de jeu** : une ligne par carte jouée : pastille, nom, puis l'effet obtenu (« Marche : Mouvement 2 », « Rage : Mouvement 1 (de
  côté) », « Improvisation : Mouvement 3, Marche défaussée »).
- **Fin** : main vide, le message de fin de partie actuel (`game.finished`) s'affiche (Q3).
- Pas de visuel de carte : du texte, une pastille et une icône, provisoires.

### Raccourcis clavier

| Touche | Effet | Contexte |
|---|---|---|
| Échap | Annuler le choix de la carte à défausser | Partie, en mode choix |

Pas de touche pour jouer une carte (Q4) : une carte offre plusieurs façons d'être jouée, une touche par bouton changerait de sens à chaque
carte jouée. À revoir quand les cartes seront dessinées. Échap est ajoutée à `docs/raccourcis.md`.

## Effets sur les specs déjà implémentées

Signalés ici parce que des tests existants changent (règle « jamais modifier un test sans le signaler ») :

- **000** : le jeu jouet disparaît. Les tests du noyau qui s'en servent sont **réécrits sur les vraies cartes**, même sens :
  - `draw_is_the_only_legal_action` → remplacé par C14 ;
  - `decide_draws_top_card_without_mutating_state` → `decide_plays_card_without_mutating_state` (C18) ;
  - `apply_card_drawn_moves_card_to_hand` → garde son sens (une pioche met la carte en main), vérifié par C5 ;
  - `empty_deck_ends_the_game` et `empty_config_starts_finished_game` → remplacés par C15 (main vide, plus de décision) ;
  - `draw_on_empty_deck_is_refused` → remplacé par L1 ;
  - `start_shuffles_all_configured_cards` → remplacé par C4 ;
  - `runner_plays_toy_game_to_the_end` → `runner_plays_until_the_hand_is_empty` ;
  - les autres (graine, vue, journal filtré, boucle non bloquante, journal des décisions, mauvais joueur, choix hors bornes, numéros
    du journal) gardent leur nom et leur sens ; seule la configuration (sans liste de cartes) et l'action jouée changent.
- **002** : l'écran de partie affiche les vraies cartes ; les tests du joueur humain et de l'écran utilisent `PlayCard` au lieu de
  `DrawCard`. Le libellé d'un bouton n'est plus une clé fixe par action : il est calculé à partir de l'action (C20).
- **003** : le test des logs de la boucle cherchait `DrawCard` dans la ligne de log d'une action ; il cherche `PlayCard`. Le numéro du
  premier événement d'action est déjà calculé (correction de la spec 012).
- **012** : `GameConfig` perd `deck` et les événements de démarrage gagnent 5 `CardDrawn` (C11 de la spec 012 s'allonge en conséquence).

## Hors périmètre

- Effet renforcé et mana : specs 020 à 022. Cartes dont l'effet de base touche à la mana ou aux cristaux : spec 020 ou 021.
- Attaque et Parade jouables, en combat : spec 031 (et éléments : 032).
- Soin et Blessures (Tranquillité, Rajeunissement, et leur option « piocher une carte » avec elles) : spec 034.
- Bonus de gloire de Nobles manières : spec 040 (implémentée). Préparer une unité : spec 042.
- Dépenser les points de mouvement : spec 014 ; explorer : spec 015.
- Fin du tour (défausser la zone de jeu, repiocher jusqu'à la limite de main, points perdus), et le repos : spec 016.
- Reprendre une carte jouée (« Reverting », MKUE p. 5) : spec 024 (annulation).
- Actions avancées, sorts, artefacts, effets « composites » (Concentration renforcée, MKUE p. 5) : J5.
- Visuels des cartes.

## Décisions prises à la relecture

Réponses de Mathieu (2026-10-25) :

1. **Q1** — Attaque et Parade non jouables hors combat, comme le livret : accepté.
2. **Q2** — Noms des cartes : ceux de l'édition française (tableau du modèle), d'après les cartes de Mathieu. Effets réécrits par nous.
3. **Q3** — Main vide : la partie s'arrête sur « Partie terminée » jusqu'à la spec 016 : accepté.
4. **Q4** — Aucun raccourci pour jouer une carte : accepté.
5. **Q5** — Effets de combat décrits dans le contenu mais pas jouables : accepté.

6. **Maquette** — Validée avec deux changements demandés par Mathieu : jouer de côté par une icône et un menu déroulant sur chaque
   carte ; choisir la carte à défausser dans la main plutôt que dans une fenêtre modale, sans y lister la carte jouée.

## Corrections pendant l'implémentation

Signalées à Mathieu (2026-10-25) :

- **Choix hors bornes (specs 000 et 003)** : `out_of_range_choice_is_refused` et le test des logs donnaient l'index 5, qui est
  maintenant une action légale (une main de 5 cartes en offre plus de 5) ; ils donnent l'index 1000. Même sens.
- **Limite de pas par image (spec 002)** : une partie de 5 cartes se finit avant la limite de 64 pas ; le test passe par
  `run_frame_limited(runner, 2)` (même boucle, limite en paramètre) pour vérifier qu'une image rend la main avant la fin.
- **Tests du noyau (spec 000)** : `empty_config_starts_finished_game` est supprimé (plus de liste de cartes dans la configuration,
  remplacé par C15) ; les tests renommés sont ceux de la section « Effets sur les specs déjà implémentées ».
- **`Action::label_key` (spec 002)** supprimé avec son test unitaire : le libellé d'un bouton est calculé par l'écran (C20).
- **Éléments** : `Element` n'a que Physique et Glace, les seuls utilisés par les cartes de base ; Feu et Feu glacé viendront avec
  la spec 032 (pas de variante morte).
- **Libellés** : clés `play_area.line`, `play_area.line_sideways` et `play_area.line_discarding` ajoutées au tableau pendant la
  maquette ; texte affiché dans la zone de jeu replié à la largeur du panneau.
- **Endurance givrée** : version révisée (Lost Legion), Attaque 2 physique.

## Questions ouvertes

Aucune.
