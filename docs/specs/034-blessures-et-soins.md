# 034 — Blessures et soins

- Statut : implémentée (2026-10-30)
- Jalon : J3
- Dépend de : 031 (déroulé d'un combat, Blessures en main, repos prolongé Q12), 033 (Toxique, Paralysant), 016 (fin du tour),
  013 et 022 (jouer une carte, effet renforcé), 025 (Euphorie cristalline, Concentration), 024 (points de non-retour)
- Livret : MKUE p. 6 (*Playing a Turn* 5.a, 5.b, 7, 7.a), p. 8 (*Combat with Enemies* 6), p. 9 (*Assign Damage Phase* 5.c),
  p. 11 (*Wounds and Healing* 1 et 3) ; FAQ p2 (*Toxique* et *Assommé*), p3 (*Repos*) ; Compendium, *Healing* S2, *Resting* Q1,
  Q2, S3, S4, *Minimum Turn* S1, *Combat* S20, Q22, S45 (`docs/regles/faq/compendium/`), *Tranquility* S1, *Rejuvenate* Q1,
  *Crystal Joy* S1, *Basic Action Cards* O5 (`23-basic-action-cards.md`)
- Crates touchées : arcanum-core (règles, contenu), arcanum-ui, arcanum-persistance (une partie préparée), `assets/i18n/` (fr.json et en.json)

## Objectif

Les Blessures ne sont plus seulement subies. Le héros peut se **soigner** avec Tranquillité (ou Rajeunissement pour Norowas), qui
peut aussi faire **piocher** ; il peut se **reposer** au lieu de se déplacer pour défausser ses Blessures ; et un combat trop
coûteux le laisse **assommé**, sa main défaussée. Euphorie cristalline renforcée peut se reprendre en défaussant une Blessure.

## Règles

### Soins

- **Soin X** (MKUE p. 11, *Wounds and Healing* 3.a et 3.c) : chaque point de soin **jette** une Blessure de la main (elle retourne à
  la pile de blessures, illimitée : elle quitte simplement la partie). Il n'y a pas encore d'unité (spec 042) ni de soin acheté (spec
  041) : le seul usage d'un point de soin est de jeter une Blessure de la main. Les points sont donc dépensés **aussitôt** la carte
  jouée, sans réserve de points de soin (Q2). Les Blessures étant identiques, le moteur jette les premières de la main.
- **Soin en trop** (Compendium, *Tranquility* S1) : on peut jouer un Soin avec moins de Blessures en main que de points, ou aucune.
  Les points en trop sont perdus.
- **Seulement la main** (Compendium, *Healing* S2) : les Blessures du deck et de la défausse ne sont pas soignées.
- **Pas en combat** (MKUE p. 8, *Combat with Enemies* 6 ; p. 6, 5.b ; p. 11, 3.d) : aucun effet de soin pendant un combat ; après la
  fin du combat, dans le même tour, oui. Une carte qui n'a que l'icône de soin, comme Tranquillité et Rajeunissement, est une
  **carte de soin** : aucun de ses effets ne se joue en combat, même « piocher » ou « gagner un jeton vert » (Compendium,
  *Rejuvenate* Q1 ; Q3). Elle se joue toujours de côté (Attaque 1, Parade 1), ce n'est pas un effet de soin.
- **Piocher** (texte des cartes ; Compendium, *Basic Action Cards* O5, avis de l'auteur) : « piocher une carte » prend la carte du dessus du deck et la
  met en main. Deck trop court : on pioche ce qu'il y a, rien s'il est vide ; la carte reste jouable. La défausse n'est jamais
  mélangée (spec 016). Piocher révèle une carte : **point de non-retour** (spec 024).
- **Concentration** (MKUE p. 5, *Composite effects* 1.b) : Concentration renforcée jouée avec Tranquillité donne l'effet renforcé
  (Soin 2 ou piocher deux cartes), sans le bonus +2, qui ne vaut que pour Mouvement, Influence, Parade et Attaque. Elle respecte les
  limites de la carte : pas en combat.

Cartes concernées (`cartes/actions-de-base.md`) :

| Carte | Base | Renforcé (vert) | Avant | Après |
|---|---|---|---|---|
| Tranquillité | Soin 1, ou piocher une carte | Soin 2, ou piocher deux cartes | pas encore disponible | jouable hors combat |
| Rajeunissement (Norowas) | Soin 1, piocher une carte, ou jeton vert [ou préparer une unité : 042] | Soin 2, piocher deux cartes, ou cristal vert [ou préparer une unité : 042] | jeton ou cristal, même en combat | quatre options sur trois, hors combat (Q3) |

### Repos

- **Se reposer** (MKUE p. 6, 7 et 7.a ; Compendium, *Resting* Q1) : un tour de repos est un tour sans déplacement, sans exploration
  et sans combat ; on peut y jouer des cartes (soins, mana… et même un Mouvement inutile ; Compendium, *Resting* S3). Le livret ne
  demande pas de l'annoncer avant de jouer ses cartes, et rien ne se joue différemment ensuite : le repos est une **façon de finir
  son tour**, pas une déclaration en début de tour (Q1).
- **Repos standard** (*Standard Rest*, MKUE p. 6, 7.a) : défausser une carte qui n'est pas une Blessure et autant de Blessures qu'on
  veut. À la fin du tour, on peut déjà défausser autant de cartes non-Blessure qu'on veut (MKUE p. 11, *End of the Turn* 7.a) : un
  repos standard est donc une fin de tour dont la défausse contient **au moins une carte non-Blessure et des Blessures**. Sans
  Blessure, c'est une fin de tour ordinaire, inchangée (spec 016).
- **Repos prolongé** (*Slow Recovery*, MKUE p. 6, 7.a) : la main ne contient que des Blessures ; on en défausse **une**. La spec 031
  (Q12) l'autorisait seulement si aucune carte n'avait été jouée ; il l'est désormais aussi après avoir joué des cartes, tant que le
  héros ne s'est pas déplacé ni n'a combattu (Compendium, *Resting* Q1, options 3 et 4 ; FAQ p3, *Repos*). Main vide (toutes les
  Blessures soignées) : rien à défausser, la fin de tour ordinaire suffit puisqu'une carte a été jouée (Compendium, *Resting* Q2).
- **Quand le repos est permis** : le héros ne s'est pas déplacé, n'a pas exploré, n'a pas défié ni combattu ce tour (MKUE p. 6, 7 ;
  la provocation suppose un déplacement). Après l'un d'eux, une Blessure ne se défausse pas à la fin du tour (spec 031, inchangé).
- **Les Blessures défaussées vont dans la défausse** (MKUE p. 11, *Wounds and Healing* 1.b) : ce n'est pas un soin, elles reviendront
  à la manche suivante (spec 023).
- **Tour minimum** (MKUE p. 6, 5.a ; Compendium, *Minimum Turn* S1) : inchangé. Le repos défausse toujours au moins une carte, sauf
  la main vide après des cartes jouées.
- **Main de Blessures, rien de joué** (Compendium, *Resting* S4) : on ne peut jouer aucune carte, donc le tour est forcément un repos
  prolongé. Défier un ennemi n'est pas permis (sans cela, la fin du tour serait impossible après le combat). Se déplacer et explorer
  demandent des points, donc des cartes jouées : rien à interdire de plus.

### Assommé

- **Assommé** (*Knock out*, MKUE p. 9, *Assign Damage Phase* 5.c) : dès que les Blessures **mises dans la main pendant ce combat**
  atteignent la limite de main non modifiée (**5**, niveau 1 ; les niveaux arrivent avec la spec 035), le héros défausse aussitôt
  toutes les cartes de sa main qui ne sont pas des Blessures, comme Paralysant (spec 033).
- **Ce qui compte** : seulement les Blessures mises en main dans ce combat, dégâts doublés (Brutal) compris. Pas les Blessures déjà
  en main avant le combat, ni celles qu'une attaque toxique met dans la défausse (Compendium, *Combat* S20 ; FAQ p2, *Toxique*).
- **Une fois** (Compendium, *Combat* Q22) : on n'est assommé qu'une fois ; les Blessures suivantes arrivent en main comme avant
  (MKUE p. 9, 5.c, dernière puce ; Compendium, *Combat* S45).
- **Après** : le combat continue ; les cartes déjà jouées, le mana et les effets restent (Compendium, *Combat* Q22). En fin de tour,
  le héros repioche jusqu'à sa limite de main, Blessures comprises (spec 016) : avec 5 Blessures ou plus en main, il ne pioche rien.
- La phase d'assignation des dégâts se résout seule, ennemi par ennemi dans l'ordre du combat (031, Q3) : l'assommé arrive après
  l'ennemi dont les Blessures font atteindre la limite. Aucune carte ne se joue entre deux ennemis, donc l'ordre ne change pas le
  résultat.

### Euphorie cristalline renforcée

- **Reprise avec une Blessure** (texte de la carte ; Compendium, *Crystal Joy* S1) : jouée renforcée, Euphorie cristalline se reprend
  en main à la fin du tour en défaussant une autre carte, **Blessure comprise**. Jouée pour son effet de base, la Blessure reste
  refusée (spec 025). Cette reprise vaut à toute fin de tour, même après un déplacement : c'est l'effet de la carte, pas un repos.

### Exemple : un tour de repos

Main au début du tour : Blessure ×3, Tranquillité, Rage. Deck de 8 cartes. Le héros ne bouge pas.

```
  Joue Tranquillité (base) : Soin 1          ─▶  main : Blessure ×2, Rage        (1 Blessure jetée)
  Finir le tour, repos standard :
    défausse Rage + Blessure ×2              ─▶  main : vide ; défausse : Rage, Blessure ×2 ; zone de jeu : Tranquillité
  Fin du tour (016) : zone de jeu défaussée, repioche 5 cartes
```

Sans jouer Tranquillité, le héros aurait pu défausser Tranquillité + 3 Blessures : main vide aussi, mais 3 Blessures dans la
défausse au lieu de 2 (FAQ p3, *Repos*).

## Modèle

### Termes

Termes du glossaire déjà présents : Blessure, Soin, Repos, Limite de main, Défausse, Pile de blessures, Tour normal.

Termes changés ou ajoutés au glossaire dans la même PR (termes de la FAQ française, qui font foi) :

| Livret (EN) | Français | Code |
|---|---|---|
| Standard Rest | Repos standard (FAQ p3 ; remplace « Repos normal ») | fin de tour qui défausse au moins une carte non-Blessure et des Blessures |
| Slow Recovery | Repos prolongé (FAQ p3 ; remplace « Repos long » de la 031) | fin de tour qui défausse une Blessure, main faite de Blessures |
| Knock out / Knocked out | Assommer / Assommé (FAQ p2) | `KnockedOut` |
| Heal X | Soin X | `Effect::Heal` |
| Throw away (a Wound) | Jeter (une Blessure) (FAQ p3) | `WoundsHealed` |
| Draw a card (effect) | Piocher une carte | `Effect::DrawCards` |
| Healing card | Carte de soin | une carte dont les effets sont tous de soin (contenu) |

### Données

- Le joueur sait, pour le tour en cours, si le héros **s'est déplacé, a exploré, a défié ou a combattu** : remis à faux par la fin du
  tour. C'est ce qui permet ou non le repos.
- Le combat compte les **Blessures mises en main** depuis son début, et retient si le héros a été **assommé**.
- Le contenu (`basic_actions.ron`) décrit Tranquillité et les options Soin et piocher de Rajeunissement, et marque ces deux cartes
  comme cartes de soin.

## Actions et événements

Aucune action nouvelle : le repos est une fin de tour (Q1).

| Action | Validation (refusée si…) | Événements produits |
|---|---|---|
| `PlayCard` (013) | en plus : effet d'une carte de soin pendant un combat | `CardPlayed`, puis pour un Soin `WoundsHealed` (s'il y a au moins une Blessure en main), pour piocher un `CardDrawn` par carte piochée |
| `EndTurn { discard, reclaim }` (016, 025) | une Blessure dans `discard` est refusée, **sauf** au repos : héros ni déplacé, ni explorateur, ni au combat ce tour, et soit `discard` contient aussi une carte non-Blessure (repos standard), soit la main n'a que des Blessures et `discard` en contient exactement une (repos prolongé). `reclaim` d'une Blessure : seulement après Euphorie cristalline renforcée | inchangé (`CardDiscarded` par carte, `TurnEnded`…) |
| `Challenge` (031) | en plus : la main n'a que des Blessures et aucune carte n'a été jouée ce tour | inchangé |
| `NextCombatPhase` (031) | inchangée | depuis la phase de parade : les dégâts comme dans la 033, avec `KnockedOut` juste après le `WoundsTaken` qui fait atteindre la limite |

| Événement | Effet de `apply` | Révèle une information ? | Point de non-retour ? |
|---|---|---|---|
| `WoundsHealed { player, wounds }` (nouveau) | ces Blessures quittent la main et retournent à la pile de blessures | non | non |
| `CardDrawn` (016) | inchangé, mais aussi pendant le tour | oui : la carte, cachée aux autres (filtrée, inchangé) | **oui** (déjà) |
| `KnockedOut { player, cards }` (nouveau) | le héros est assommé ; `cards` (les cartes non-Blessure de la main, peut-être aucune) vont dans la défausse | non (en solo, rien à filtrer) | non |

**Actions légales** : les fins de tour avec Blessures s'ajoutent quand le repos est permis. Les Blessures étant identiques, les
actions légales n'en proposent qu'une combinaison par **nombre** de Blessures (les premières de la main), pour ne pas multiplier les
fins de tour ; une fin de tour qui en nomme d'autres reste valide. `Challenge` disparaît dans le cas de la main de Blessures.

## Comportements attendus

Chaque cas devient au moins un test (nom indicatif entre parenthèses). Sauf mention : héros Tovak (Tranquillité dans son deck), Jour,
bac à sable, héros qui ne s'est pas déplacé ; arène de combat des tests (spec 032) pour l'assommé, armure du héros 2.

Soins (arcanum-core) :

- **C1** — Main : Blessure ×2, Tranquillité, Marche. Tranquillité jouée pour « Soin 1 » : une Blessure est jetée, la main garde
  une Blessure et Marche ; la défausse et le deck ne changent pas (`heal_throws_away_a_wound_from_hand`) — MKUE p. 11, 3.a et 3.c.
- **C2** — Même main avec trois Blessures, Tranquillité renforcée (mana vert), « Soin 2 » : deux Blessures jetées
  (`strong_heal_throws_away_two_wounds`) — MKUE p. 11, 3.c.
- **C3** — Une seule Blessure en main, « Soin 2 » : elle est jetée, le point en trop est perdu ; aucune Blessure en main : l'option
  reste jouable, `WoundsHealed` n'est pas produit (`heal_beyond_wounds_in_hand_is_lost`) — Compendium, *Tranquility* S1.
- **C4** — Une Blessure dans la défausse (attaque toxique), aucune en main : Soin 1 ne la touche pas
  (`heal_ignores_wounds_in_discard_pile`) — Compendium, *Healing* S2.
- **C5** — Tranquillité pour « piocher une carte » : la carte du dessus du deck passe en main ; la décision est un point de
  non-retour (`draw_option_draws_top_card`) — texte de la carte, spec 024.
- **C6** — « Piocher deux cartes » avec un deck d'une carte : une carte piochée ; deck vide : rien, la carte est jouée
  (`draw_stops_when_deck_is_empty`) — Compendium, *Basic Action Cards* O5, avis de l'auteur.
- **C7** — En combat, à toute phase : aucune option de Tranquillité n'est légale, ni renforcée ; Tranquillité de côté l'est là où
  une carte de côté l'est (031) (`healing_card_cannot_be_played_in_combat`) — MKUE p. 8, 6.
- **C8** — Norowas : Rajeunissement propose Soin 1, piocher une carte et jeton vert hors combat ; en combat, aucune
  (`rejuvenate_is_a_healing_card`) — Compendium, *Rejuvenate* Q1 (Q3).
- **C9** — Après la fin d'un combat, dans le même tour, Tranquillité « Soin 1 » jette une Blessure reçue dans ce combat
  (`heal_after_combat_same_turn`) — MKUE p. 11, 3.d.
- **C10** — Concentration renforcée jouée avec Tranquillité : options Soin 2 ou piocher deux cartes, sans +2 ; en combat, Tranquillité
  n'est pas une cible de Concentration (`concentration_boosts_tranquility_without_bonus`) — MKUE p. 5, *Composite* 1.b.

Repos (arcanum-core) :

- **C11** — Main : Blessure ×2, Marche, Rage ; aucune carte jouée. Fin du tour en défaussant Marche et les deux Blessures : accepté ;
  la défausse reçoit Marche et 2 Blessures, la main garde Rage et repioche 4 cartes (`standard_rest_discards_wounds`) — MKUE p. 6,
  7.a ; p. 11, 1.b.
- **C12** — Même main : défausser les deux Blessures **sans** carte non-Blessure est refusé ; défausser Marche, Rage et une seule
  Blessure est accepté (`standard_rest_needs_a_non_wound_card`) — MKUE p. 6, 7.a.
- **C13** — Après un déplacement, une exploration, un défi ou un combat ce tour, une fin de tour qui défausse une Blessure est
  refusée (`no_rest_after_moving_or_fighting`) — MKUE p. 6, 7.
- **C14** — Une carte de Mouvement jouée, sans se déplacer : le repos standard reste permis (`rest_allowed_after_playing_move`) —
  Compendium, *Resting* S3.
- **C15** — L'exemple « un tour de repos » : Tranquillité « Soin 1 », puis repos standard (Rage + 2 Blessures) ; main vide, défausse
  Rage + 2 Blessures + Tranquillité, repioche 5 (`heal_then_standard_rest`) — FAQ p3, *Repos*.
- **C16** — Main : Blessure ×3, aucune carte jouée : les seules fins de tour défaussent exactement une Blessure ; en défausser deux
  est refusé (`slow_recovery_discards_one_wound`) — MKUE p. 6, 7.a (031, Q12).
- **C17** — Main : Blessure ×2, Tranquillité. Tranquillité « Soin 1 » : la main n'a plus qu'une Blessure ; finir en la défaussant
  (repos prolongé) ou sans rien défausser est accepté (`slow_recovery_after_playing_a_card`) — Compendium, *Resting* Q1, option 3.
- **C18** — Main de Blessures, aucune carte jouée, ennemi errant voisin : `Challenge` n'est pas légal ; s'il est demandé, refusé
  (`wounds_only_hand_cannot_challenge`) — Compendium, *Resting* S4.
- **C18b** — Main vide au début du tour, deck non vide (issue privée 86) : finir le tour sans rien défausser est légal
  (`empty_hand_ends_turn_without_discard`) — MKUE p. 6, 5.a.
- **C18c** — Main et deck vides au début du tour, ennemi errant voisin (issue privée 86) : seule l'annonce de la fin de la manche est
  légale, ni défi ni fin de tour (`empty_hand_and_deck_cannot_challenge`) — MKUE p. 6, 3.b.
- **C19** — Main de Blessures et deck vide au début du tour : annoncer la fin de la manche reste légal à côté du repos prolongé
  (`wounds_only_hand_may_announce_end_of_round`) — MKUE p. 6, 3 (spec 023).
- **C20** — Les actions légales d'un repos standard avec 3 Blessures en main proposent, pour chaque choix de cartes non-Blessure,
  0 à 3 Blessures, une seule combinaison par nombre (`rest_actions_offer_one_combination_per_wound_count`).

Euphorie cristalline (arcanum-core, héros Goldyx) :

- **C21** — Euphorie cristalline renforcée jouée, puis un déplacement : la fin de tour qui la reprend en défaussant une Blessure est
  acceptée ; jouée pour son effet de base, refusée (`strong_crystal_joy_reclaims_with_a_wound`) — texte de la carte.

Assommé (arcanum-core, arène) :

- **C22** — Minotaure (attaque 5, brutal) non paré : 10 points, 5 Blessures en main ; le héros est assommé, ses cartes non-Blessure
  vont dans la défausse (`five_wounds_in_one_combat_knock_out`) — MKUE p. 9, 5.c.
- **C22b** — Héros assommé sans carte jouée, combat fini : la seule fin de tour légale est sans défausse ; défausser une Blessure
  est refusé (`knocked_out_hero_ends_turn_without_discard`) ; à l'écran, pas de demande de défausse
  (`knocked_out_hero_is_not_asked_to_discard`) — MKUE p. 6, 5.a et 7 (issue privée 85).
- **C23** — Épéistes (6) et Rôdeurs (4) non parés : 3 puis 2 Blessures ; `KnockedOut` suit le `WoundsTaken` des Rôdeurs
  (`knock_out_counts_wounds_of_the_whole_combat`) — MKUE p. 9, 5.c.
- **C24** — Épéistes seuls : 3 Blessures, pas assommé (`fewer_wounds_than_hand_limit_no_knock_out`).
- **C25** — Moines (5, toxiques) seuls : 3 Blessures en main, 3 dans la défausse, pas assommé ; Moines et Sorcières maudites (3,
  toxiques) : 3 + 2 en main, assommé (`poison_wounds_in_discard_do_not_count`) — Compendium, *Combat* S20 ; FAQ p2.
- **C26** — Trois Blessures en main avant le combat, Épéistes non parés (3 de plus) : pas assommé
  (`wounds_before_combat_do_not_count`) — MKUE p. 9, 5.c (« during a combat »).
- **C27** — Minotaure et Rôdeurs non parés : 7 Blessures en main, un seul `KnockedOut` (`knocked_out_only_once`) — Compendium,
  *Combat* Q22.
- **C28** — Méduse (6, paralysante) et Épéistes non parés : la main est défaussée par la paralysie, puis `KnockedOut` est produit
  sans carte (`knock_out_after_paralysis_discards_nothing`) — MKUE p. 9, 5.c ; p. 10, 6.b.
- **C29** — Assommé avec une Attaque jouée dans la zone de jeu : le combat passe en phase d'attaque, la zone de jeu est intacte ;
  en fin de tour, avec 5 Blessures en main, rien n'est pioché (`knocked_out_combat_goes_on`) — Compendium, *Combat* Q22.
- **C30** — Revenir en arrière avant la décision qui a assommé le héros rend sa main (`knock_out_can_be_undone`) — spec 024
  (`KnockedOut` n'est pas un point de non-retour).

Interface (arcanum-ui, modèle d'écran) :

- **C31** — En mode fin du tour, repos permis, Blessures en main : chaque Blessure a une case « Défausser » ; repos interdit, elle
  n'en a pas et la ligne `end_turn.no_rest` le dit (`end_turn_mode_offers_wounds_when_resting`).
- **C32** — Une Blessure cochée : le bandeau dit `end_turn.rest_standard` (ou `end_turn.rest_slow` pour une main de Blessures), et
  « Finir le tour » n'est actif que si la défausse cochée est une fin de tour légale (`rest_banner_follows_checked_cards`).
- **C33** — Héros assommé : l'historique du combat a la ligne `combat.history.knocked_out` (`history_shows_knock_out`).

## Cas limites et refus

- **L1** — Soin joué avec une Blessure dans la zone de jeu : impossible, une Blessure ne se joue pas (031).
- **L2** — Main vide après un soin (toutes les Blessures jetées) : la fin de tour ordinaire, sans défausse, suffit puisqu'une carte a
  été jouée (Compendium, *Resting* Q2).
- **L3** — Repos après avoir pris un dé ou un cristal : permis, ce ne sont ni des déplacements ni des combats.
- **L4** — Repos standard avec reprise d'Euphorie cristalline : permis ; la carte défaussée pour la reprise ne compte pas comme celle
  du repos (deux cartes distinctes, spec 025).
- **L5** — Piocher pendant le tour prend toujours le dessus du deck, jamais la défausse (spec 016) ; une Blessure du deck (attaque
  toxique d'une manche précédente) peut être piochée, elle reste en main comme toute Blessure.
- **L6** — Assommé avec une main vide (toutes les cartes jouées) : `KnockedOut` sans carte, comme C28.
- **L7** — Plus de 255 Blessures données au joueur (numéros d'exemplaire en `u8`, issue privée 76) : inchangé par cette spec, les soins ne
  rendent pas de numéro.

## Information cachée

Les cartes piochées pendant le tour sont cachées aux autres joueurs, comme à la fin du tour (`CardDrawn` filtré, inchangé). Les
Blessures jetées et les cartes défaussées par l'assommé sont publiques ; en solo, rien d'autre à filtrer. Le repos prolongé
« montre la main » (MKUE p. 6, 7.a) : en solo, rien à faire.

## Libellés (i18n)

| Clé | Paramètres | fr | en |
|---|---|---|---|
| `effect.heal` | `amount` | Soin {amount} | Heal {amount} |
| `effect.draw_cards` | `count` | Piocher {count} carte(s) | Draw {count} card(s) |
| `effect.not_in_combat` | — | Soin : pas en combat | Healing: not in combat |
| `end_turn.rest_standard` | — | Repos standard : défaussez une carte qui n'est pas une Blessure, et autant de Blessures que vous voulez. | Standard Rest: discard one non-Wound card and any number of Wounds. |
| `end_turn.rest_slow` | — | Repos prolongé : votre main n'a que des Blessures, défaussez-en une. | Slow Recovery: your hand holds only Wounds, discard one of them. |
| `end_turn.no_rest` | — | Pas de repos : vous vous êtes déplacé ou avez combattu ce tour, les Blessures restent en main. | No rest: you moved or fought this turn, Wounds stay in hand. |
| `combat.history.knocked_out` | `count` | Assommé : {count} carte(s) défaussée(s) de la main | Knocked out: {count} card(s) discarded from hand |
| `combat.knocked_out` | — | Assommé : 5 Blessures reçues dans ce combat. | Knocked out: 5 Wounds taken in this combat. |

`combat.knocked_out` prend la limite de main en paramètre (`limit`) si la spec 035 la fait varier ; d'ici là le texte la cite.

## Interface

- **Main** : Tranquillité et Rajeunissement affichent leurs options (« Soin 1 ou Piocher 1 carte ») au lieu de « Pas encore
  disponible ». En combat, leurs options sont grisées avec `effect.not_in_combat`, comme un Mouvement en combat (031) ; le bouton de
  côté reste.
- **Mode fin du tour** (016) : quand le repos est permis, chaque Blessure de la main a sa case « Défausser ». Dès qu'une Blessure est
  cochée, le bandeau remplace son texte par celui du repos standard ou prolongé, et dit combien de cartes on piochera, comme avant.
  Repos interdit et Blessures en main : la ligne `end_turn.no_rest` sous le bandeau, les Blessures sans case.
- **Combat** : assommé, une ligne `combat.knocked_out` sous le titre de la phase jusqu'à la fin du combat, et la ligne d'historique
  `combat.history.knocked_out` après celle des dégâts de l'ennemi qui l'a provoqué.

```
┌─ Panneau latéral, mode fin du tour ───────────────┐
│ ┌───────────────────────────────────────────────┐ │
│ │ Repos standard : défaussez une carte qui n'est│ │
│ │ pas une Blessure, et autant de Blessures que  │ │
│ │ vous voulez.                                  │ │
│ │ Vous piocherez 5 carte(s).                    │ │
│ │              [Finir le tour]  [Annuler]       │ │
│ └───────────────────────────────────────────────┘ │
│ Main                                              │
│ ● Blessure                             ☑ Défausser│
│ ● Blessure                             ☑ Défausser│
│ ● Rage        Attaque 2 ou Parade 2    ☑ Défausser│
│ Zone de jeu                                       │
│ ● Tranquillité : Soin 1                           │
└───────────────────────────────────────────────────┘
```

Vérification à l'œil : une partie préparée `soins` (même départ que `main-blessures`, main : Blessure ×3, Tranquillité, Rage) pour
le soin et le repos ; l'assommé se voit dans l'arène avec `combat-capacites` ou une partie `combat-assomme` (Minotaure), au choix de
l'implémentation.

### Raccourcis clavier

Aucun : le repos passe par le mode fin du tour (Fin, Entrée, Échap, spec 016), le soin par les boutons d'une carte comme toute option.

## Effets sur les specs déjà implémentées

- **016** : L7 (les Blessures ne se défaussent pas à la fin du tour) devient « sauf au repos ». Le mode fin du tour gagne les cases des
  Blessures et les bandeaux du repos.
- **031** : Q12 (repos long) est généralisé en repos prolongé, permis aussi après avoir joué des cartes ; « repos long » devient
  « repos prolongé » partout (glossaire, commentaires). `Challenge` est refusé avec une main de Blessures sans carte jouée.
- **025** : la reprise d'Euphorie cristalline renforcée avec une Blessure, reportée à la 034, est faite (C21).
- **013 / 022** : Tranquillité et les options Soin et piocher de Rajeunissement deviennent jouables ; Rajeunissement n'est plus
  jouable en combat (Q3).
- **Joueur « premier choix » des tests et parcours** (019, 028, `logs_moteur.rs`) : Tranquillité devient jouable, et les fins de
  tour avec Blessures s'ajoutent ; ce joueur peut donc jouer autrement. Les parcours s'ajustent en le signalant dans la PR (ADR 0006,
  découpage indicatif) ; les points d'étape ne changent pas.
- **024** : `Event::blocks_undo` gagne `WoundsHealed` et `KnockedOut` (non). Les sauvegardes d'avant restent lisibles (aucun
  événement existant ne change de forme).

## Hors périmètre

- Unités blessées, soin d'une unité, unités pendant l'assommé : spec 042 (avec la réserve de points de soin qu'il faudra alors, Q2).
- Acheter des soins au village ou au monastère : spec 041.
- Clairière magique (jeter une Blessure de la main ou de la défausse en fin de tour) : spec 044, comme ses jetons de mana.
- Retraite forcée et case sûre (MKUE p. 11, *End of the Turn* 2 ; p. 7, 7.d) : aucun effet ne permet encore de finir son tour hors
  d'une case sûre (Compendium, *Forced Withdrawal* S1) ; avec le premier qui le permet (specs 043 ou 052).
- Limite de main qui varie avec le niveau (et donc le seuil de l'assommé) : spec 035.
- Compétences et cartes qui jettent ou jouent des Blessures (Pouvoir de la douleur, Rituel de soin…), sorts de soin : specs 051, 052.

## Décisions prises à la relecture

Mathieu a validé la spec le 2026-10-30 (« Oui validé »), avec les trois propositions :

1. **Q1** — Le repos est une façon de finir son tour : cocher des Blessures dans le mode fin du tour, permis si le héros ne s'est
   ni déplacé ni battu. Pas d'action « Se reposer » à annoncer.
2. **Q2** — Pas de réserve de points de soin : un Soin jette aussitôt les Blessures de la main. La réserve viendra avec la 041 ou la
   042.
3. **Q3** — Tranquillité et Rajeunissement sont des cartes de soin : aucune de leurs options ne se joue en combat (le jeton vert de
   Rajeunissement non plus).

## Questions ouvertes

Aucune.

## Notes d'implémentation

- **Validation des fins de tour** : `end_turn_allowed` (arcanum-core) dit si une défausse respecte les règles ci-dessus ; le moteur
  l'utilise pour accepter une fin de tour qui nomme d'autres Blessures que les actions légales, et l'écran pour activer
  « Finir le tour » et cocher n'importe quelle Blessure.
- **Assommé ou paralysé sans carte jouée** (issue privée 85, choix de Mathieu le 2026-10-30) : les cartes défaussées par Assommé ou
  Paralysant comptent pour le tour minimum (MKUE p. 6, 5.a, à la lettre : « jouer ou défausser au moins une carte pendant le
  tour »), malgré l'avis du compendium (note 220) qu'une défausse imposée ne compte pas. Le héros a combattu : pas de repos, il
  finit son tour sans défausse (C22b, et spec 033 C11b pour Paralysant) ; le bandeau ne demande pas de défausser. Cela remplace
  l'exception de la 031 (Q12), qui lui faisait défausser une Blessure. Le moteur le déduit de la main : après un combat sans carte
  jouée, une main de Blessures vient d'Assommé ou de Paralysant (ou d'une main vide au début du tour, C18b).
- **Défier** est refusé avec une main **non vide** faite de Blessures, et avec une main et un deck vides (C18c, issue privée 86).
  Avec une main vide et un deck non vide, le défi reste permis : la fin du tour sans défausse l'est aussi (C18b). Le moteur ne
  garde pas la main du début du tour : une main vide sans carte jouée l'était dès le début, puisqu'une carte ne quitte la main
  qu'en étant jouée (Assommé et Paralysant y laissent des Blessures).
- Tests d'avant ajustés : `capacites_ennemis.rs` (Gargouille au lieu du Minotaure invoqué, qui assommerait), `elements_resistances.rs`
  et `arcanum-ui/tests/deroule_combat.rs` (passage en phase d'attaque sans dégâts, `skip_to_attack_phase` de l'arène, pour les tests
  d'attaque que 6 Blessures assommeraient), `deroule_combat.rs` (`KnockedOut` attendu après 5 Blessures ; repos prolongé : une
  seule combinaison ; Blessures gardées : le héros se déplace), `cartes_de_mana.rs` (l'option de mana de Rajeunissement devient
  la troisième), `jouer_des_cartes.rs` (plus aucune carte d'effet de base indisponible : test L3 de la 013 retiré),
  `effets_renforces.rs` (Tranquillité renforcée jouable ; dés pris bornés par `dice_limit`, la pioche changeant les parties
  tirées au hasard). Les parcours (019, 028) passent sans changement.

## Vérification manuelle

Parties préparées `soins` et `combat-assomme` (`docs/verification-manuelle.md`). Captures dans les fichiers du projet :
`spec-034-soins-main.png` (options de Tranquillité), `spec-034-soin-joue.png` (une Blessure jetée), `spec-034-fin-du-tour.png`
(cases « Défausser » des Blessures), `spec-034-repos-standard.png` (bandeau du repos standard), `spec-034-assomme.png` (ligne
« Assommé » et historique).
