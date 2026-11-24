# 022 — Effets renforcés

- Statut : implémentée (2026-10-25)
- Jalon : J2
- Dépend de : 013 (jouer des cartes), 016 (fin du tour), 020 (Source), 021 (inventaire de cristaux)
- Livret : MKUE p. 4 (« Deed cards » 3.a : effet renforcé ; 6 : coût qu'on ne peut pas payer), p. 5 (« Using mana » 1 à 3),
  p. 6 (8.a : rendre les dés), p. 11 (« End of the Turn » 1 : relancer et rendre les dés utilisés)
- Contenu : scans `docs/regles/scans/deed/` (effets renforcés repris, textes réécrits)
- Crates touchées : arcanum-core, arcanum-ui, `assets/i18n/` (fr.json et en.json)

## Objectif

Le joueur peut jouer une carte pour son **effet renforcé** en payant **un mana de la couleur de la carte** : soit en **prenant un dé**
de la Source (un par tour, l'or vaut n'importe quelle couleur de base de jour), soit en **utilisant un cristal** de son inventaire.
Marche donne alors Mouvement 4 au lieu de 2. À la fin du tour, le dé pris est **relancé et rendu** à la Source. C'est le premier usage
du mana : la Source (020) et les cristaux (021) servent enfin.

Les cartes qui **produisent** du mana (Concentration, Cristallisation, Source de mana…) et les **jetons de mana** passent dans une
nouvelle spec 025, Cartes de mana (Q1).

## Règles

- **Effet renforcé** (MKUE p. 4, 3.a) : en payant un mana de la couleur de la carte (spec 013).
- **Payer ou ne pas jouer** (MKUE p. 4, 6).
- **D'où vient le mana** (MKUE p. 5, « Using mana » 1, 1.b et 3) : un **dé de la Source**, qu'on **prend**, ou un **cristal** de
  l'inventaire, qu'on **utilise** et qui retourne à la réserve.
- **Simplification : un cristal n'est utilisé qu'au moment de payer** (décision de Mathieu, Q3). Le livret permet de transformer un
  cristal en mana fondamental « à tout moment » de son tour (p. 5, 1.b). Le jeu ne le fait qu'au moment où le mana est dépensé, dans
  l'action qui le dépense. C'est équivalent : le mana fondamental non utilisé disparaît en fin de tour (p. 5, 1.a ; p. 11, 3.a), et
  aucune règle du livret ne dépend d'un cristal transformé d'avance ; garder ses cristaux rapporte même de la Gloire en fin de partie
  (p. 15, 17). Seule exception relevée, hors périmètre : le défenseur d'un combat entre joueurs (p. 12). À revérifier quand des cartes
  ou des compétences parleront du mana fondamental de la zone de jeu (specs 025, 051, 052).
  Les jetons de mana de la zone de jeu sont une troisième source : il n'y en a pas encore, ils arrivent avec la spec 025 (Q1).
- **Un dé par tour** (MKUE p. 5, 3 et 3.b) : on le prend **au moment où on utilise** son mana. Dans le jeu, prendre le dé et payer
  l'effet ne font donc qu'**une seule action** (Q3).
- **Or et noir** (MKUE p. 5, 2.a, 2.b et 3.c ; p. 4, 3.b) : de jour, un dé or vaut un mana de **n'importe quelle couleur de base** ; le
  noir ne sert jamais. De nuit, l'or ne sert jamais ; le noir ne paie jamais l'effet renforcé d'une carte d'action. Un dé épuisé ne peut
  pas être pris. Un cristal ne paie que sa propre couleur (pas de cristal or ni noir, p. 5, 2).
- **Fin du tour** (MKUE p. 5, 3.a ; p. 6, 8.a ; p. 11, 1) : d'abord **relancer** les dés utilisés et les **rendre** à la Source, **sans**
  la règle « au moins la moitié de couleurs de base » (réservée à la mise en place et à la réinitialisation, p. 3 ; p. 4, 2.b). Un dé
  relancé peut donc revenir noir, épuisé de jour.
- **Cristal utilisé** (MKUE p. 5, 1.b) : l'inventaire baisse de un. Jamais de mana fondamental changé en cristal, sauf effet contraire
  (spec 025).

### Les effets renforcés des cartes d'action de base

Lus sur les scans. « Jouable » = jouable renforcé avec cette spec, hors combat (MKUE p. 6, 5.c).

| Carte | Couleur | Effet renforcé | Jouable |
|---|---|---|---|
| Marche | vert | Mouvement 4 | oui |
| Endurance | bleu | Mouvement 4 | oui |
| Célérité | blanc | Attaque à distance 3 | non (combat, spec 031) |
| Promesse | blanc | Influence 4 | oui |
| Menace | rouge | Influence 5 ; à la fin du tour, Réputation −1 | oui depuis la spec 040 (Q2) |
| Rage | rouge | Attaque 4 | non (combat) |
| Détermination | bleu | Parade 5 | non (combat) |
| Improvisation | rouge | Défausser une autre carte : Mouvement 5, Influence 5, Attaque 5 ou Parade 5 | oui, pour Mouvement 5 ou Influence 5 |
| Tranquillité | vert | Soin 2, ou piocher deux cartes | non (spec 034) |
| Cristallisation | bleu | Gagner un cristal de n'importe quelle couleur de base | non (spec 025) |
| Source de mana | blanc | Prendre un dé, le mettre sur une couleur sauf l'or, gagner deux jetons de cette couleur ; ne pas relancer ce dé | non (spec 025) |
| Concentration | vert | Jouer une autre carte d'action pour son effet renforcé sans payer ; +2 si c'est du Mouvement, de l'Influence, une Parade ou une Attaque | non (spec 025) |
| Polyvalence (Arythea) | rouge | Attaque 4, Parade 4, Attaque de feu 3, Parade de feu 3, Attaque à distance 3 ou Attaque de siège 2 | non (combat, specs 031-032) |
| Attraction de mana (Arythea) | blanc | Prendre deux dés, les mettre sur des couleurs sauf l'or, gagner un jeton de chacune ; ne pas les relancer | non (spec 025) |
| Pouvoir de concentration (Goldyx) | vert | Comme Concentration, avec +3 | non (spec 025) |
| Euphorie cristalline (Goldyx) | bleu | Gagner un cristal de n'importe quelle couleur de base ; en fin de tour, défausser une autre carte, même une Blessure, pour la reprendre en main | non (spec 025) |
| Nobles manières (Norowas) | blanc | Influence 4 ; si utilisée pendant une interaction, Gloire +1 et Réputation +1 en fin de tour | oui, pour Influence 4 (le bonus : spec 040, implémentée) |
| Rajeunissement (Norowas) | vert | Soin 2, piocher deux cartes, gagner un cristal vert, ou préparer une unité de niveau I à III | non (specs 025, 034, 042) |
| Endurance givrée (Tovak) | bleu | Parade de glace 5, +1 par capacité, couleur d'attaque et résistance de l'ennemi paré | non (combat, specs 031-033) |
| Instinct (Tovak) | rouge | Mouvement 4, Influence 4, Attaque 4 ou Parade 4 | oui, pour Mouvement 4 ou Influence 4 |

Comme pour l'effet de base (spec 013), les effets renforcés faits de points sans élément nouveau sont **décrits** dans le contenu même
s'ils ne sont pas jouables (Rage, Détermination, Célérité, Instinct, Improvisation) : la spec 031 n'aura qu'à les rendre jouables. Les
autres sont marqués « pas encore pris en charge ».

### Exemple : un tour de Tovak, de jour

Source : 🟩 vert, 🟨 or, ⬛ noir (épuisé). Inventaire : 1 bleu. Main : Marche, Endurance, Instinct, Rage, Promesse.

| Étape | Ce que fait le joueur | Mana payé | Résultat |
|---|---|---|---|
| 1 | Marche, renforcée | prend le dé vert | Mouvement 4. Plus aucun dé ne peut être pris ce tour : l'or reste dans la Source. |
| 2 | Endurance, renforcée | utilise le cristal bleu | Mouvement 4 (total 8). Inventaire : vide. |
| 3 | Instinct : renforcé impossible | — | Il faudrait du rouge : l'or le pourrait, mais le dé du tour est déjà pris ; pas de cristal rouge. Instinct se joue pour son effet de base ou de côté. |
| 4 | Se déplace, puis finit le tour | — | Le dé vert est relancé (il sort, par exemple, rouge) et rendu à la Source, avant le nettoyage de la zone de jeu. |

Tour suivant : Source 🟥 rouge, 🟨 or, ⬛ noir. Un dé peut de nouveau être pris.

### Schéma : peut-on payer l'effet renforcé d'une carte de couleur C ?

```
                    ┌─ un dé a déjà été pris ce tour ? ── oui ──┐
Dés de la Source ───┤                                           │
                    └─ non ── pour chaque dé non pris :          │
                              couleur C ................ oui     │
                              or, de jour .............. oui     │
                              noir, ou or de nuit ...... non     │
                              autre couleur de base .... non     │
                                                                 ▼
Inventaire ───────── au moins un cristal de couleur C ? ......... oui / non

Payable si au moins une des sources dit oui. Sinon, l'effet renforcé n'est pas proposé.
```

### Schéma : la fin du tour

```
EndTurn { discard }
   │
   ├─ 1. relancer et rendre les dés pris ..... DieReturned…         (022, un par dé pris, dans l'ordre où ils ont été pris)
   ├─ 3. nettoyer la zone de jeu ............. TurnEnded            (016)
   ├─ 4. bénéfice de la case : mine ? ........ CrystalGained        (021)
   ├─ 7. défausser, puis repiocher ........... CardDiscarded…, CardDrawn…   (016)
   └─ tour suivant ........................... TurnStarted          (016)
```

## Modèle

### Termes

Termes du glossaire déjà présents : Effet renforcé, Effet de base (`CardPlay::Basic`), Source (`Source`), Dé de mana (`ManaDie`),
Couleur de mana (`ManaColor`), Or, Noir, Épuisé, Mana fondamental, Cristal (`Crystal`), Inventaire (`Inventory`), Réserve, Relancer,
Zone de jeu (`PlayArea`), Fin du tour (`EndTurn`, `TurnEnded`).

Nouveaux termes, ajoutés au glossaire dans la même PR (à valider, Q5). « Mana » est masculin, comme dans « mana fondamental ».

| Livret (EN) | Français | Code |
|---|---|---|
| Strong effect | Effet renforcé | `CardPlay::Strong` (le terme existait, sans code) |
| Power (a card, with mana) | Renforcer (une carte) ; carte « renforcée » | `CardPlay::Strong` |
| Take (a die from the Source) | Prendre (un dé) | `DieTaken` (événement) |
| Return (dice to the Source), after rerolling them | Rendre (un dé), relancé | `DieReturned` (événement) |
| Turn a crystal into pure mana | Utiliser (un cristal) | `CrystalUsed` (événement) |
| (paying mana) | Payer (du mana) | `ManaPayment` (`Die(n)`, `Crystal`) |

### Contenu (arcanum-core, en Rust)

Chaque définition de carte gagne son **effet renforcé**, de la même forme que l'effet de base : une liste d'options faites de points,
avec au besoin un coût (« défausser une autre carte »), ou « pas encore pris en charge ». Valeurs : tableau des règles.

### Nouvelles données dans l'état (par joueur)

- les **dés pris ce tour** : la position de chaque dé pris dans la Source, dans l'ordre où ils ont été pris. Public. Vide au début du
  tour.

La Source garde ses dés à leur place (dé 1, dé 2, dé 3) : un dé pris y reste, avec sa couleur, mais n'est plus disponible jusqu'à ce
qu'il soit rendu. La limite d'un dé par tour est une constante (`DICE_PER_TURN`) ; Source de mana et Attraction de mana la
relèveront (spec 025).

## Actions et événements

`CardPlay` gagne une variante. Aucune action nouvelle.

| Action | Validation (refusée si…) | Événements produits |
|---|---|---|
| `PlayCard { card, play: Strong { option, discard, mana } }`, avec `mana` = `Die(n)` (le n-ième dé de la Source, à partir de 0) ou `Crystal` | comme `Basic` (carte pas en main, option inexistante ou pas jouable hors combat, effet renforcé pas encore pris en charge, `discard` absent, en trop, égal à la carte ou pas en main) ; **`Die(n)`** : pas de n-ième dé, dé déjà pris, un dé déjà pris ce tour, dé épuisé, dé qui n'est ni de la couleur de la carte ni or ; **`Crystal`** : aucun cristal de la couleur de la carte | `DieTaken` ou `CrystalUsed`, puis `CardPlayed`, puis `CardDiscarded` si l'option défausse une carte |
| `EndTurn { discard }` (016) | inchangée | **`DieReturned` pour chaque dé pris, dans l'ordre où ils ont été pris**, puis `TurnEnded` et la suite (016, 021) |

| Événement | Effet de `apply` | Révèle une information ? |
|---|---|---|
| `DieTaken { player, die, color }`, `color` = couleur du dé | ajoute `die` aux dés pris de `player` | oui, publique |
| `CrystalUsed { player, color }` | retire un cristal de `color` de l'inventaire de `player` (rien s'il n'y en a pas) | oui, publique |
| `CardPlayed { …, play: Strong { … }, effect }` (013) | inchangé : la carte va dans la zone de jeu avec `play`, l'effet renforcé s'ajoute aux points | oui, publique |
| `DieReturned { player, die, color }`, `color` = nouvelle couleur, hasard résolu | le dé `die` de la Source prend `color` et n'est plus pris par `player` | oui, publique |

**Qui décide.** `decide` vérifie le paiement et lance les dés à rendre (une face au hasard parmi les six, dé après dé, dans l'ordre où
ils ont été pris ; cet ordre d'usage du générateur fait partie du contrat). Une fin de tour sans dé pris n'utilise pas le générateur.
`apply` ne revérifie rien (même principe que les specs 016 et 021).

**Actions légales**, dans cet ordre pour chaque carte de la main : ses options d'effet de base (013), puis ses options d'effet renforcé
jouables, chacune avec chaque carte à défausser si l'option le demande (ordre de la main), et chacune avec chaque paiement possible :
les dés de la Source qui peuvent payer, dans l'ordre de la Source, puis le cristal s'il y en a un ; puis de côté Mouvement 1 et
Influence 1 (013). Deux dés de même couleur donnent chacun leur action (le moteur ne fusionne rien ; l'interface, si, C26).

**Décision en attente** : inchangée (toute carte se joue déjà de côté).

## Comportements attendus

Chaque cas devient au moins un test (nom indicatif entre parenthèses). Sauf mention, partie de bac à sable avec Tovak, de jour. Les
situations précises (Source, inventaire, main) sont construites en appliquant des événements, comme dans la spec 021.

Contenu (arcanum-core) :

- **C1** — Les effets renforcés des 20 cartes sont ceux du tableau des règles : options et coût, ou « pas encore pris en charge »
  (`strong_effect_definitions_match_cards`).

Payer avec un dé (arcanum-core) :

- **C2** — Marche en main, un dé vert en position 0 : l'action `Strong { option 0, mana: Die(0) }` est légale ; `decide` donne
  `DieTaken { die: 0, color: vert }` puis `CardPlayed` avec Mouvement 4 ; après `apply`, 4 points de mouvement, le dé 0 est pris
  (`powering_march_with_a_green_die_gives_four_move_points`) — MKUE p. 4, 3.a ; p. 5, 3.
- **C3** — Promesse (blanche), un seul dé utilisable, or : `Strong` avec `Die` sur ce dé donne Influence 4 ; de même pour une carte de
  chaque couleur (`gold_die_pays_any_basic_color_by_day`) — MKUE p. 5, 2.a.
- **C4** — Un dé noir, de jour, n'est jamais proposé ; l'action qui le désigne est refusée (`black_die_cannot_pay_by_day`) — MKUE p. 5,
  2.a et 3.c.
- **C5** — Marche (verte), Source bleu, rouge, blanc, sans cristal : aucun effet renforcé proposé pour Marche ; `Strong` avec un dé bleu
  est refusé (`die_must_match_the_card_color`) — MKUE p. 4, 6.
- **C6** — Après avoir renforcé une carte avec un dé, plus aucune action ne désigne un dé, même s'il reste un dé vert ou or ; une action
  `Die` est refusée (`only_one_die_per_turn`) — MKUE p. 5, 3.
- **C7** — Le dé pris reste dans la Source à sa place, avec sa couleur, marqué pris ; la vue joueur montre les dés pris
  (`taken_die_stays_unavailable_until_end_of_turn`).

Payer avec un cristal (arcanum-core) :

- **C8** — Endurance (bleue), 1 cristal bleu : `Strong { mana: Crystal }` donne `CrystalUsed { bleu }` puis `CardPlayed` avec
  Mouvement 4 ; l'inventaire passe à 0 bleu (`powering_with_a_crystal_spends_it`) — MKUE p. 5, 1.b.
- **C9** — Deux Marche et 2 cristaux verts, aucun dé vert ni or : les deux Marche se jouent renforcées l'une après l'autre ; ensuite,
  plus de cristal vert, plus d'effet renforcé vert (`each_crystal_pays_once`) — MKUE p. 5, 1.b.
- **C10** — L'exemple des règles, joué de bout en bout : Marche avec le dé vert, Endurance avec le cristal bleu, Instinct sans effet
  renforcé proposé ; 8 points de mouvement (`die_and_crystal_in_the_same_turn`) — MKUE p. 5, 1 et 3.
- **C11** — Un dé vert et un cristal vert : les deux paiements sont proposés pour Marche, le joueur choisit
  (`player_chooses_between_die_and_crystal`).

Options (arcanum-core) :

- **C12** — Instinct renforcé : Mouvement 4 et Influence 4 sont proposés, pas Attaque 4 ni Parade 4 ; Nobles manières renforcée :
  Influence 4 (`strong_choice_effects_offer_only_move_and_influence_out_of_combat`) — MKUE p. 6, 5.c.
- **C13** — Improvisation, deux autres cartes en main, un dé rouge : l'effet renforcé est proposé pour Mouvement 5 et Influence 5,
  avec chaque carte à défausser ; le jouer donne, dans l'ordre, `DieTaken`, `CardPlayed` (Mouvement 5), `CardDiscarded`
  (`improvisation_strong_discards_and_pays`) — MKUE p. 4, 6.
- **C14** — Rage, Détermination, Célérité, Polyvalence, Endurance givrée, Menace, Tranquillité et les cartes de mana : aucun effet
  renforcé proposé, même avec du mana de leur couleur (`out_of_scope_strong_effects_are_not_offered`).
- **C15** — Les actions légales d'une main, d'une Source et d'un inventaire donnés sont exactement celles de l'énumération, dans son
  ordre ; chacune est acceptée par `decide` (`legal_actions_are_exhaustive`, repris de la spec 013).

Fin du tour (arcanum-core) :

- **C16** — Un dé pris, fin du tour : les événements commencent par `DieReturned` pour ce dé, avant `TurnEnded` ; après `apply`, le
  dé a la couleur tirée et n'est plus pris ; même graine, même couleur (`taken_die_is_rerolled_and_returned_first`) — MKUE p. 5, 3.a ;
  p. 11, 1.
- **C17** — Au tour suivant, un dé peut de nouveau être pris (`die_limit_resets_each_turn`) — MKUE p. 5, 3.
- **C18** — Fin du tour sans dé pris (cristal seulement, ou rien de renforcé) : aucun `DieReturned`, la Source ne change pas
  (`untaken_dice_are_not_rerolled`). Le test `source_is_unchanged_by_turns` de la spec 020 reste vrai tel quel.
- **C19** — Avec une graine trouvée par le test et figée, le dé rendu revient noir alors que la Source n'a plus qu'un dé de couleur de
  base : on ne relance pas (`returned_die_ignores_the_half_rule`) — MKUE p. 5, 3.a ; p. 3.

Information cachée et journal (arcanum-core) :

- **C20** — La vue joueur contient les dés pris ; le journal filtré pour un autre joueur montre `DieTaken`, `CrystalUsed`,
  `CardPlayed` (renforcé) et `DieReturned` en entier (`mana_payments_are_public`).
- **C21** — Rejouer le journal après un tour avec dé et cristal redonne la même Source, le même inventaire et les mêmes dés pris
  (`replaying_log_rebuilds_mana_state`).

Affichage (arcanum-ui) :

- **C22** — Chaque carte de la main affiche sous son effet de base une ligne « Renforcé : Mouvement 4 » (ou « Renforcé : pas encore
  disponible ») ; un bouton « Renforcer : Mouvement 4 », marqué de la couleur de la carte, par option renforcée jouable
  (`hand_shows_strong_effects`).
- **C23** — Option renforcée jouable mais impossible à payer : bouton grisé, dont l'info-bulle dit pourquoi : « Il faut un mana vert :
  un dé vert ou or de la Source, ou un cristal vert. » ou, si le dé du tour est déjà pris, « Dé du tour déjà pris : il faut un cristal
  vert. » (`strong_button_explains_missing_mana`) (Q4).
- **C24** — Un seul paiement possible (un dé vert, pas de cristal vert, pas d'or) : cliquer « Renforcer : Mouvement 4 » joue Marche tout
  de suite avec ce dé (`single_payment_plays_directly`).
- **C25** — Plusieurs paiements possibles : le clic n'envoie rien encore ; l'écran passe en mode choix du mana, avec un bandeau
  « Marche renforcée : Mouvement 4. Choisissez le mana à payer. », un bouton par paiement et « Annuler » ; cliquer un paiement envoie
  l'action et quitte le mode (`mana_choice_lists_payments`) (Q4).
- **C26** — En mode choix du mana, deux dés de même couleur ne donnent qu'un bouton (« Dé vert ») qui désigne le premier dans l'ordre de
  la Source ; libellés : « Dé vert », « Dé or (vaut vert) », « Cristal vert (1 sur 3) » (`identical_dice_give_one_button`).
- **C27** — Improvisation renforcée : d'abord le choix de la carte à défausser (013), puis, s'il y a plusieurs paiements, le choix du
  mana ; « Annuler » à chaque étape revient à l'écran normal (`improvisation_strong_chooses_discard_then_mana`).
- **C28** — La ligne Source montre le dé pris après « pris : », cerclé ; son info-bulle dit « Dé de mana vert : pris ce tour, relancé à
  la fin du tour. » (`panel_shows_taken_die`).
- **C29** — La zone de jeu dit avec quoi la carte a été renforcée : « Marche (renforcée, dé vert) : Mouvement 4 », « Endurance
  (renforcée, cristal bleu) : Mouvement 4 », « Promesse (renforcée, dé or) : Influence 4 » (`play_area_names_the_mana_paid`).
- **C30** — Avec le vrai `fr.json`, calculer l'écran pour une main de chaque héros, avec et sans mana, en mode choix du mana, avec un dé
  pris, ne relève aucun manque (`every_strong_effect_key_exists_in_french`).

Vérification manuelle : `cargo run -p arcanum-ui` sous Linux ; renforcer une carte avec un dé de sa couleur ou or (Mouvement doublé), puis
avec un cristal gagné sur une mine ; finir le tour : le dé change de couleur et redevient disponible.

## Cas limites et refus

- **L1** — Nuit (spec 023) : un dé or est épuisé, donc jamais proposé ; un dé noir ne paie pas un effet renforcé de carte d'action.
  Testé sur un état construit (`gold_is_useless_at_night`) — MKUE p. 5, 2.b.
- **L2** — Improvisation renforcée sans autre carte en main : pas proposée (013, même règle pour la défausse).
- **L3** — `apply` d'un `CrystalUsed` sans cristal de cette couleur, ou d'un `DieTaken` sur un dé déjà pris : `decide` ne les produit
  jamais ; `apply` reste infaillible (rien à retirer, dé déjà marqué). Un test joue 200 tours aléatoires avec des paiements et vérifie
  qu'aucun inventaire ne devient négatif et qu'au plus un dé est pris par tour (`mana_payments_never_overspend`).
- **L4** — Fin du tour refusée (016) : aucun événement, le dé reste pris.
- **L5** — Le joueur a le droit de payer avec un cristal alors qu'un dé pourrait payer, et inversement : rien n'est imposé (C11).
- **L6** — Effet renforcé et effet de base d'une même carte ne se cumulent pas : on joue l'un ou l'autre (MKUE p. 4, 3.a).
- **L7** — Un dé pris qui était or reste or dans la Source jusqu'à la fin du tour ; la zone de jeu dit « dé or » (C29), même si la carte
  est rouge.

## Information cachée

- Publics : les dés pris, les paiements (`DieTaken`, `CrystalUsed`), les cartes renforcées et le résultat des relances. Sur la table,
  tout se voit.
- `DieReturned` porte le hasard résolu ; le générateur n'est dans aucune vue.

## Libellés (i18n)

Ajoutés dans `fr.json` et `en.json`. Couleurs au masculin (« dé », « cristal », « mana ») : clés de la spec 020 (`mana.green`…) ;
« renforcée » s'accorde avec « carte », sans accord calculé.

| Clé | Paramètres | Français | English |
|---|---|---|---|
| `card.strong` | `effect` | Renforcé : {effect} | Powered: {effect} |
| `play.strong` | `effect` | Renforcer : {effect} | Power: {effect} |
| `play.strong_discarding` | `effect` | Renforcer : {effect}… | Power: {effect}… |
| `play.strong_no_mana` | `color` | Il faut un mana {color} : un dé {color} ou or de la Source, ou un cristal {color}. | You need {color} mana: a {color} or gold die from the Source, or a {color} crystal. |
| `play.strong_die_taken` | `color` | Dé du tour déjà pris : il faut un cristal {color}. | Die already taken this turn: you need a {color} crystal. |
| `mana_choice.prompt` | `card`, `effect` | {card} renforcée : {effect}. Choisissez le mana à payer. | {card} powered: {effect}. Choose the mana to pay. |
| `mana_choice.die` | `color` | Dé {color} | {color} die |
| `mana_choice.gold_die` | `color` | Dé or (vaut {color}) | Gold die (as {color}) |
| `mana_choice.crystal` | `color`, `count`, `limit` | Cristal {color} ({count} sur {limit}) | {color} crystal ({count} of {limit}) |
| `mana_choice.cancel` | — | Annuler | Cancel |
| `source.taken` | — | pris : | taken: |
| `source.die.taken` | `color` | Dé de mana {color} : pris ce tour, relancé à la fin du tour. | {color} mana die: taken this turn, rerolled at the end of the turn. |
| `play_area.line_strong_die` | `card`, `color`, `effect` | {card} (renforcée, dé {color}) : {effect} | {card} (powered, {color} die): {effect} |
| `play_area.line_strong_crystal` | `card`, `color`, `effect` | {card} (renforcée, cristal {color}) : {effect} | {card} (powered, {color} crystal): {effect} |
| `card.strong_unsupported` | — | Renforcé : pas encore disponible | Powered: not yet available |

En anglais, l'ordre des mots diffère (« {color} die ») : les paramètres le permettent sans code.

## Interface

- **Main** : chaque carte affiche sa ligne d'effet renforcé et, à côté du bouton de l'effet de base, un bouton « Renforcer : … » par
  option jouable, avec une pastille de la couleur de la carte. Grisé avec une info-bulle quand on ne peut pas payer (C23).
- **Choix du mana** : bandeau et boutons, comme le choix de la carte à défausser (spec 013) ; la carte en cours n'est plus listée dans
  la main ; pas de menu de côté pendant le choix.
- **Source** : le dé pris passe après « pris : », cerclé, entre les dés utilisables et les épuisés, pour qu'on le repère (il reste à
  sa place dans le moteur).
- **Zone de jeu** : la ligne de la carte nomme le mana payé (C29).
- Pas de clic sur un dé ni sur un cristal : on part toujours de la carte (Q3).

```
┌─ Panneau latéral ─────────────────────────────────────────────┐
│ Jour · Tour 3                                                 │
│ Source :  ▣ or   pris : ◙ vert   épuisé : ▢ noir              │
│ Inventaire :  ◇◇◇   ◆◇◇   ◇◇◇   ◇◇◇                           │
│ Mouvement : 4 · Influence : 0                                 │
│ Zone de jeu                                                   │
│ ● Marche (renforcée, dé vert) : Mouvement 4                   │
│ Main                                                          │
│ ● Endurance     Mouvement 2 · Renforcé : Mouvement 4          │
│                 [Jouer : Mouvement 2] [● Renforcer : Mouvement 4] [Jouer de côté ▾] │
│ ● Instinct      Mouvement 2, … · Renforcé : Mouvement 4, …    │
│                 [Jouer : Mouvement 2] … [● Renforcer : Mouvement 4] (grisé)          │
│                   └ info-bulle : « Dé du tour déjà pris : il faut un cristal rouge. » │
└───────────────────────────────────────────────────────────────┘

Mode choix du mana (Marche, un dé vert et un cristal vert) :
┌───────────────────────────────────────────────────────────────┐
│ Marche renforcée : Mouvement 4. Choisissez le mana à payer.   │
│ [Dé vert]  [Cristal vert (1 sur 3)]               [Annuler]   │
└───────────────────────────────────────────────────────────────┘
```

### Raccourcis clavier

| Touche | Effet | Contexte |
|---|---|---|
| Échap | Annuler le choix du mana | Partie, en mode choix du mana |

Même touche que pour annuler le choix de la carte à défausser (013) : les deux modes ne sont jamais ouverts en même temps. Pas de
touche pour choisir un paiement (Q6). Reporté dans `docs/raccourcis.md` à l'implémentation.

## Effets sur les specs déjà implémentées

Signalés ici parce que le sens de tests existants change (règle « jamais modifier un test sans le signaler ») :

- **013** : `legal_actions_are_exhaustive` (C14) devient C15 de cette spec : l'énumération gagne les options renforcées entre l'effet de
  base et le côté. La Source de mise en place donne presque toujours un dé utilisable : le test est donc repris avec l'énumération
  complète. `basic_action_definitions_match_cards` (C2) gagne l'effet renforcé, ou C1 le complète.
- **016** : la fin du tour gagne l'étape 1 (rendre les dés). La liste exhaustive des événements gagne une branche `DieReturned`. Le
  joueur « premier choix » (C12) n'est pas touché : la première action légale reste un effet de base ou de côté.
- **020** : C11 `source_is_unchanged_by_turns` reste vrai (il ne renforce aucune carte) ; la ligne Source gagne « pris : ».
- **021** : rien ne change ; l'inventaire peut maintenant baisser.
- **002, 011, 012, 014, 016, 020, 021** : `PlayerView`, construit champ par champ dans des tests, gagne les dés pris.
- **Spec 024 (à venir)** : relancer un dé à la fin du tour est une révélation (MKUE p. 5, « Reverting » 2) ; l'annulation ne pourra
  pas remonter avant.

## Hors périmètre

- Cartes qui produisent du mana ou des cristaux, jetons de mana dans la zone de jeu et leur retour à la réserve (p. 11, 3.a), jeton
  gagné à la place d'un quatrième cristal (p. 5, « "Gain" effects » 2), dés supplémentaires et dés mis sur une couleur (Source de mana,
  Attraction de mana), effets composites (Concentration, Pouvoir de concentration), Euphorie cristalline et son retour en main : spec
  025, Cartes de mana (Q1).
- Effet renforcé de Menace (Réputation −1) et bonus de Nobles manières : spec 040 (Q2), implémentée.
- Effets renforcés d'Attaque et de Parade : spec 031 ; éléments Feu (Polyvalence) : 032 ; bonus d'Endurance givrée : 033.
- Soin et pioche (Tranquillité, Rajeunissement) : spec 034.
- Noir de nuit pour les sorts (p. 4, 3.b) : spec 052.
- Compétences, unités et tactiques qui consomment ou donnent du mana : specs 042, 051, 060.
- Nuit : spec 023 (le mécanisme Jour/Nuit est déjà en place et testé sur état construit, L1).

## Décisions prises à la relecture

Réponses de Mathieu (2026-10-25) :

1. **Q1** — Accepté : la 022 se limite au paiement (effet renforcé, dé de la Source, cristal, or et noir, relance en fin de tour) ;
   les cartes qui produisent du mana et les jetons de mana vont dans la nouvelle spec 025 Cartes de mana, juste après la 022
   (`PLAN.md` mis à jour).
2. **Q2** — Accepté : Menace renforcée attend la réputation (spec 040) ; Nobles manières renforcée est jouable dès maintenant.
3. **Q3** — Un cristal n'est consommé qu'au moment où l'on en a besoin (règle « Simplification », vérifiée sur tout le livret). Pas
   d'action séparée « prendre un dé » ni « utiliser un cristal ».
4. **Q4** — Accepté : bouton « Renforcer » grisé avec la raison ; choix du mana seulement s'il y a plusieurs paiements distincts.
5. **Q5** — Termes acceptés : « Effet renforcé », « Renforcer », carte « renforcée », « Prendre un dé », « Rendre », « Utiliser un
   cristal », « mana » au masculin.
6. **Q6** — Accepté : seulement Échap pour annuler le choix du mana, pas de touches chiffrées.

## Corrections pendant l'implémentation

- **Libellés** : `play.strong.no_mana` et `play.strong.die_taken` deviennent `play.strong_no_mana` et `play.strong_die_taken`, une clé
  ne pouvant pas être à la fois un texte (`play.strong`) et le parent d'autres clés dans le JSON imbriqué (comme pour la spec 020).
- **Spec 013, `legal_actions_are_exhaustive`** : inchangé. Ses états n'ont pas de Source, donc aucun effet renforcé ; C15 est un
  nouveau test, avec Source et cristal.
- **Spec 012, `setup_events_come_in_order`** : sa correspondance exhaustive sur les événements gagne les trois nouvelles branches.
- **Spec 002, `PlayerView`** construit champ par champ (`boucle_et_fenetre.rs`) : gagne `dice_taken`, comme prévu.
- **Choix du mana** : deux paiements qui se valent (deux dés de même couleur) comptent pour un ; avec un seul paiement distinct, le
  bouton joue tout de suite (C24, C26).
- **Zone de jeu** : une Improvisation renforcée nomme le mana payé, pas la carte défaussée (aucun libellé ne prévoit les deux).
- **Tests arcanum-ui** : écrits avec l'écran ; leur échec a été vérifié sur les libellés retirés (7 tests sur 9 rouges), pas avant le code
  de l'écran, qu'ils ne compilaient pas sans.

- **Interface** : les boutons « Renforcer » sont chacun sur leur ligne, sous ceux de l'effet de base ; côte à côte, ils élargissaient le
  panneau latéral (vu sous Xvfb).

Vérification manuelle sous Linux (Xvfb) : captures `spec-022-main.png` (boutons « Renforcer ») et `spec-022-carte-renforcee.png`
(Endurance renforcée avec le dé bleu : Mouvement 4, « pris : bleu », ligne de la zone de jeu) dans les fichiers du projet.

## Questions ouvertes

Aucune.
