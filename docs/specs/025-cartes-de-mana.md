# 025 — Cartes de mana

- Statut : implémentée
- Jalon : J2
- Dépend de : 013 (jouer des cartes), 016 (fin du tour), 020 (Source), 021 (inventaire), 022 (effets renforcés et paiement),
  023 (fin de manche), 029 (contenu en données)
- Livret : MKUE p. 4 (« Deed cards » 6 : payer du mana en plus), p. 5 (« Using mana » 1 à 3 ; « "Composite" effects » ; « "Gain"
  effects » 1 et 2), p. 11 (« End of the Turn » 3.a : jetons rendus à la réserve)
- Contenu : scans `docs/regles/scans/deed/` (Concentration, Cristallisation, Source de mana ; Attraction de mana, Pouvoir de
  concentration, Euphorie cristalline, Rajeunissement), effets repris, textes réécrits
- Crates touchées : arcanum-core (code et `content/basic_actions.ron`), arcanum-ui, `assets/i18n/` (fr.json et en.json)

## Objectif

Les cartes de mana deviennent jouables. Le joueur **gagne des jetons de mana** (Concentration) et des **cristaux** (Cristallisation),
utilise **des dés de plus** dans la Source (Source de mana), **met un dé sur une couleur** pour gagner des jetons, et **renforce
gratuitement** une autre carte avec Concentration (Mouvement 4 + 2 = 6 avec Marche). Les jetons de mana deviennent une troisième
façon de payer, après le dé et le cristal de la spec 022, et sont rendus à la réserve en fin de tour.

La spec est en trois parties, implémentables l'une après l'autre (Q1) : **A** jetons et cristaux, **B** dés supplémentaires, **C**
Concentration et Pouvoir de concentration renforcés.

## Règles

- **Jeton de mana** (MKUE p. 5, « Gain » 1 ; « Using mana » 1 et 1.a ; p. 11, 3.a) : posé dans la zone de jeu, il paie comme un dé ou
  un cristal de sa couleur ; en fin de tour, utilisé ou non, il retourne à la réserve.
- **Gagner un cristal** (MKUE p. 5, « Gain » 2) : déjà 3 cristaux de cette couleur, on gagne un **jeton** de la couleur à la place. À
  ne pas confondre avec la mine, où l'on ne gagne rien (p. 11, 4.b ; spec 021).
- **Payer du mana en plus** (MKUE p. 4, 6) : comme la spec 022 (un dé par tour, or de jour, noir jamais pour une couleur de base), avec
  le jeton comme troisième moyen de payer. La simplification de la 022 (Q3 : un cristal n'est utilisé qu'au moment de payer) reste
  vraie : aucune des 7 cartes ne parle du mana fondamental déjà dans la zone de jeu.
- **Dé de plus** (MKUE p. 5, 3) : la limite d'un dé par tour monte de un, pour ce tour.
- **Mettre un dé sur une couleur** (texte de Source de mana et d'Attraction de mana, scans) : le dé pris est tourné sur la couleur
  choisie (pas l'or) et n'est **pas relancé** en fin de tour : il revient dans la Source sur cette couleur, donc épuisé s'il est noir
  de jour. Hors limite de dés et dé épuisé permis (Q2, FAQ p1). À la fin de la manche, il est relancé avec les autres
  (spec 023).
- **Effet composite** (MKUE p. 5, « Composite » 1.a et 1.b) : Concentration renforcée joue une autre carte d'action renforcée **sans
  payer**, avec +2 sur un effet de Mouvement, d'Influence, d'Attaque ou de Parade ; un seul effet, dont on respecte les limites. Les
  deux cartes vont dans la zone de jeu.

### Les cartes

Lues sur les scans. Les options qui restent « pas encore prises en charge » sont entre crochets avec leur spec.

| Carte | Couleur | Effet de base | Effet renforcé | Partie |
|---|---|---|---|---|
| Concentration | vert | Gagner un jeton bleu, blanc ou rouge | Jouer une autre carte d'action renforcée sans payer ; +2 (Mouvement, Influence, Attaque, Parade) | A ; renforcé : C |
| Pouvoir de concentration (Goldyx) | vert | Gagner un jeton bleu, blanc ou rouge, ou un cristal vert | Comme Concentration, avec +3 | A ; renforcé : C |
| Cristallisation | bleu | Payer un mana d'une couleur de base : gagner un cristal de cette couleur | Gagner un cristal de n'importe quelle couleur de base | A |
| Euphorie cristalline (Goldyx) | bleu | Comme Cristallisation ; en fin de tour, on peut défausser une autre carte (pas une Blessure) pour la reprendre en main | Gagner un cristal de n'importe quelle couleur de base ; même reprise, Blessure permise | A |
| Rajeunissement (Norowas) | vert | Gagner un jeton vert [Soin 1, piocher une carte : 034 ; préparer une unité : 042] | Gagner un cristal vert [Soin 2, piocher deux cartes : 034 ; préparer une unité : 042] | A |
| Source de mana | blanc | Utiliser un dé de plus ce tour | Prendre un dé, le mettre sur une couleur sauf l'or, gagner deux jetons de cette couleur ; ne pas le relancer | B |
| Attraction de mana (Arythea) | blanc | Utiliser un dé de plus ce tour ; s'il est noir, il donne du mana de n'importe quelle couleur | Prendre deux dés, mettre chacun sur une couleur sauf l'or, gagner un jeton de chacune ; ne pas les relancer | B |

Lectures retenues (Q2, Q3, confirmées par la FAQ, voir Décisions) :

- les dés pris par Source de mana et Attraction de mana **renforcées** ne comptent pas dans la limite de dés du tour : ils ne paient
  rien, ils sont mis sur une couleur ; ils peuvent être **épuisés** (le dé change de couleur aussitôt) ;
- le dé de plus d'Attraction de mana, s'il est noir, peut être pris **même de jour** (où il est épuisé) : c'est le sens de « s'il est
  noir, il donne du mana de n'importe quelle couleur » ;
- les couleurs proposées pour un dé mis sur une couleur sont les quatre couleurs de base ; le **noir** (permis par le texte) ne sert
  qu'aux sorts : il sera proposé avec eux (spec 052, Q4).

### Exemple A : jetons et cristaux, un tour de Tovak, de jour

Source : 🟩 vert, 🟨 or, ⬛ noir (épuisé). Inventaire : 3 rouges. Main : Concentration, Endurance, Cristallisation, Marche, Rage.

| Étape | Ce que fait le joueur | Résultat |
|---|---|---|
| 1 | Concentration, effet de base : jeton bleu | Zone de jeu : un jeton bleu. |
| 2 | Endurance renforcée, payée avec le jeton bleu | Mouvement 4. Le jeton est dépensé. |
| 3 | Cristallisation, effet de base, en payant vert avec le dé vert | Cristal vert dans l'inventaire. Le dé du tour est pris. |
| 4 | (variante) Cristallisation renforcée, payée par un cristal bleu, en choisissant rouge | Déjà 3 rouges : il gagne un jeton rouge dans la zone de jeu à la place. |
| 5 | Finit le tour | Le dé vert est relancé et rendu ; les jetons non utilisés retournent à la réserve ; le cristal vert reste. |

### Exemple B : Source de mana, de nuit

Source : 🟦 bleu, 🟨 or (épuisé), ⬛ noir. Main : Source de mana, Endurance, Marche.

| Étape | Ce que fait le joueur | Résultat |
|---|---|---|
| 1 | Source de mana renforcée, payée avec le dé bleu… | Impossible : la carte est blanche ; il faut un mana blanc. Il la joue pour son effet de base. |
| 1 bis | Source de mana, effet de base | Il peut prendre 2 dés ce tour. |
| 2 | Endurance renforcée avec le dé bleu, puis Marche… | Marche est verte : ni l'or (épuisé de nuit) ni le noir ne paient. Le 2e dé ne sert à rien ici. |

Avec un cristal blanc, Source de mana renforcée : il prend le dé or épuisé, le met sur vert, gagne deux jetons verts ; Marche renforcée
avec un jeton vert : Mouvement 4. En fin de tour, le dé revient vert dans la Source, sans être relancé.

### Exemple C : Concentration renforcée

Main : Concentration, Marche, Promesse. Source : 🟩 vert, 🟦 bleu, 🟥 rouge.

Concentration renforcée, payée avec le dé vert, avec Marche : Marche joue son effet renforcé sans payer, Mouvement 4 + 2 = **6**. Avec
Promesse au lieu de Marche : Influence 4 + 2 = 6. Avec Cristallisation : un cristal, sans bonus (ce n'est pas un effet de points).

### Schéma : d'où vient le mana, et où il va

```
          Source (dés) ──── prendre (1 par tour, + Source de mana / Attraction de mana) ───┐
                                                                                           │
Inventaire (cristaux) ──── utiliser ─────────────────────────────────────────────────────┤──▶ payer
          ▲                                                                                │
          │ gagner un cristal (Cristallisation, Euphorie, Pouvoir, Rajeunissement)        │
          │   └─ déjà 3 de cette couleur ? ── jeton à la place ──┐                         │
          │                                                       ▼                         │
          └──────────────────────────────── Zone de jeu (jetons) ── utiliser ─────────────┘
                                                       │
                                     fin du tour ──────┴──▶ réserve (jetons non utilisés perdus)
```

## Modèle

### Termes

Termes du glossaire déjà présents : Jeton de mana (« (spec 025) », il reçoit son code), Cristal, Inventaire, Réserve, Prendre, Rendre,
Utiliser, Payer, Effet renforcé, Renforcer, Mana fondamental.

Nouveaux termes, ajoutés au glossaire dans la même PR (à valider, Q7) :

| Livret (EN) | Français | Code |
|---|---|---|
| Mana token | Jeton de mana | `ManaTokenGained`, `ManaTokenUsed` ; `ManaPayment::Token` |
| Gain (a mana token) | Gagner (un jeton) | `ManaTokenGained` |
| Set (a die) to a color | Mettre (un dé) sur une couleur | `DieSet` |
| Additional (mana die) | Dé de plus | `Effect::ExtraDie`, `GameState::dice_limit` |
| Composite effect | Effet composite | `CardPlay::Strong` avec une carte jouée avec elle |
| Take (this card) back to your hand | Reprendre (une carte en main) | `CardReturnedToHand` |

### Contenu (`crates/arcanum-core/content/basic_actions.ron`, spec 029)

Les 7 cartes sont décrites dans le fichier de données, plus en Rust. Le format gagne de nouvelles options, à côté de `Move`, `Influence`,
`Attack`… (noms indicatifs, fixés à l'implémentation) :

| Option dans le fichier | Sens | Cartes |
|---|---|---|
| `GainToken(Blue)` | gagner un jeton de cette couleur | Concentration, Pouvoir, Rajeunissement |
| `GainCrystal(Green)` | gagner un cristal de cette couleur (jeton si déjà 3) | Pouvoir, Cristallisation renforcée, Euphorie renforcée, Rajeunissement renforcé |
| `PayForCrystal(Red)` | payer un mana de cette couleur, gagner un cristal de cette couleur | Cristallisation, Euphorie |
| `ExtraDie`, `ExtraDieBlackAnyColor` | un dé de plus ce tour (le second : un dé noir vaut toute couleur) | Source de mana, Attraction de mana |
| `SetDice(2, tokens: 1)` | prendre n dés, les mettre sur une couleur, gagner des jetons par dé | Source de mana et Attraction de mana renforcées |
| `Boost(2)` | renforcer une autre carte sans payer, +n | Concentration et Pouvoir renforcés |

Une option par couleur quand on choisit une couleur (Cristallisation renforcée : 4 options), comme Instinct a une option par type de
points ; les options pas encore prises en charge (Soin, pioche, unité de Rajeunissement) restent hors du fichier, avec un commentaire
qui cite leur spec. Euphorie cristalline porte en plus `reclaim: AnyButWound` (base) ou `reclaim: Any` (renforcé). La validation du
chargement (spec 029, L5) s'étend : un jeton ou un cristal jamais or ni noir (Q4, garanti par le type : une couleur de base), un
`Boost` ou un `SetDice` d'au moins 1, et `PayForCrystal` seulement dans un effet de base (l'effet renforcé est déjà payé par la couleur
de la carte ; précisé à l'implémentation). Partie A : la reprise d'Euphorie s'écrit `reclaim: true` ; la nuance Blessure permise ou non
attend les Blessures (spec 034).

### Nouvelles données dans l'état (par joueur, toutes publiques)

- les **jetons de mana** de la zone de jeu, par couleur, dans l'ordre où ils ont été gagnés ; vides en début de tour ;
- les **dés de plus** du tour : combien, et combien autorisent le noir comme toute couleur ; à 0 en début de tour ;
- pour chaque dé pris, s'il a été **mis sur une couleur** (alors il n'est pas relancé) ;
- les cartes à **reprendre** en fin de tour (Euphorie cristalline jouée ce tour).

## Actions et événements

Aucune action nouvelle. `PlayCard` couvre tous les effets ; `EndTurn` gagne la reprise d'Euphorie cristalline.

| Action | Validation (refusée si…) | Événements produits |
|---|---|---|
| `PlayCard { card, play: Basic { option, discard, mana } }` : `mana` devient facultatif, exigé si l'option coûte du mana | comme 013 ; option qui coûte du mana sans paiement ou paiement impossible (règles de la 022, plus le jeton) ; paiement donné à une option qui n'en coûte pas | paiement (`DieTaken`, `CrystalUsed` ou `ManaTokenUsed`), `CardPlayed`, puis les gains (`CrystalGained` ou `ManaTokenGained`, `DieSet`) |
| `PlayCard { …, play: Strong { option, discard, mana, with } }` : `with` = la carte jouée avec Concentration et son propre choix (option, défausse, dés et couleurs), seulement pour Concentration et Pouvoir de concentration | comme 022 ; `with` absent pour Concentration, présent pour une autre carte ; carte de `with` absente de la main, égale à la carte, Concentration ou Pouvoir de concentration (Q5), ou dont l'effet renforcé n'est pas jouable hors combat | paiement, `CardPlayed` (Concentration), `CardPlayed` (l'autre carte, effet déjà augmenté), puis les gains de l'autre carte |
| `PlayCard`, Source de mana / Attraction de mana renforcées : les dés et leurs couleurs dans le choix | dé inexistant ou déjà pris ; deux fois le même dé ; couleur or (ou noire, Q4) | paiement, `CardPlayed`, puis un `DieSet` et ses `ManaTokenGained` par dé |
| `EndTurn { discard, reclaim }` (016) : `reclaim` = la carte de la main défaussée pour reprendre Euphorie cristalline | Euphorie cristalline pas jouée ce tour ; carte pas en main, égale à une carte de `discard` ; Blessure si l'effet de base a été joué (034) | `DieReturned…` (022 ; sans hasard pour un dé mis sur une couleur), `CardDiscarded` (reclaim), `CardReturnedToHand`, `TurnEnded`, puis la suite (016, 021) |

| Événement | Effet de `apply` | Révèle une information ? |
|---|---|---|
| `ManaTokenGained { player, color }` | ajoute un jeton de `color` à la zone de jeu | oui, publique |
| `ManaTokenUsed { player, color }` | retire un jeton de `color` (rien s'il n'y en a pas) | oui, publique |
| `CrystalGained` (021) | inchangé : il sert aussi aux cartes | oui, publique |
| `DieSet { player, die, color }` | le dé `die` de la Source prend `color`, est pris par `player`, et ne sera pas relancé | oui, publique |
| `CardPlayed` (013) | inchangé pour les points ; ajoute aussi les dés de plus. Les jetons, cristaux et dés mis sur une couleur ne changent que par les événements qui suivent | oui, publique |
| `CardReturnedToHand { player, card }` | retire `card` de la zone de jeu et la remet en main | oui, publique |
| `TurnEnded` (016) | en plus : vide les jetons, remet les dés de plus et les reprises à 0 | oui, publique |
| `DieReturned` (022) | inchangé ; pour un dé mis sur une couleur, `decide` donne la couleur qu'il a, sans tirer de dé | oui, publique |

**Qui décide.** `decide` choisit entre cristal et jeton (déjà 3 cristaux ?) et le note dans l'événement : `apply` ne recalcule rien.
Un dé mis sur une couleur n'utilise pas le générateur à la fin du tour ; les autres dés rendus le font dans l'ordre de la spec 022.

**Actions légales**, pour chaque carte de la main, dans l'ordre de la 022 : options de base (chacune, si elle coûte du mana, avec chaque
paiement : dés dans l'ordre de la Source, cristal, jeton), options renforcées (de même ; pour Concentration, chaque carte jouable avec
elle et chacun de ses choix, dans l'ordre de la main), puis de côté. Pour Source de mana et Attraction de mana renforcées, chaque dé
(ou paire de dés, dans l'ordre de la Source) et chaque couleur de base. Les `EndTurn` : sans reprise d'abord, puis une reprise par
carte de la main, chacune avec ses défausses (016). Le nombre d'actions grandit vite : une main de 5 cartes de mana, 3 dés, quelques
cristaux et jetons donne quelques centaines d'actions, ce qui reste raisonnable pour l'énumération exhaustive (à revoir avec le premier joueur automatique, issue #3).

## Comportements attendus

Chaque cas devient au moins un test (nom indicatif entre parenthèses). Sauf mention, partie de bac à sable, de jour ; situations
construites en appliquant des événements, comme dans la spec 022.

Contenu (arcanum-core) :

- **C1** — Les effets des 7 cartes, chargés depuis `basic_actions.ron`, sont ceux du tableau des règles, options non prises en charge
  comprises (`mana_card_definitions_match_cards`).

A. Jetons et cristaux (arcanum-core) :

- **C2** — Concentration, effet de base : trois options, jeton bleu, blanc, rouge ; la jouer pour le bleu donne `CardPlayed` puis
  `ManaTokenGained { bleu }` ; la vue montre un jeton bleu (`concentration_gains_a_mana_token`) — MKUE p. 5, « Gain » 1.
- **C3** — Avec un jeton bleu, Endurance renforcée est proposée avec `mana: Token` ; la jouer donne `ManaTokenUsed`, puis `CardPlayed`
  avec Mouvement 4 ; plus de jeton (`mana_token_pays_a_strong_effect`) — MKUE p. 5, 1.
- **C4** — Un jeton ne compte pas dans la limite d'un dé : on peut renforcer une carte avec le dé du tour puis une autre avec un jeton
  (`mana_token_is_not_a_die`) — MKUE p. 5, 3.
- **C5** — Cristallisation, effet de base : quatre options (une par couleur), chacune avec ses paiements ; payer vert avec le dé vert
  donne `DieTaken`, `CardPlayed`, `CrystalGained { vert }` (`crystallize_turns_mana_into_a_crystal`) — scan ; MKUE p. 4, 6.
- **C6** — Cristallisation, effet de base, payée avec un jeton rouge : cristal rouge (`crystallize_keeps_a_token_as_crystal`) — MKUE p. 5,
  1.b.
- **C7** — Cristallisation renforcée (payer bleu) : quatre options ; avec 3 cristaux rouges, choisir rouge donne `ManaTokenGained
  { rouge }` au lieu d'un cristal (`fourth_crystal_becomes_a_token`) — MKUE p. 5, « Gain » 2.
- **C8** — Pouvoir de concentration, effet de base : quatre options (jetons bleu, blanc, rouge, cristal vert) ; Rajeunissement : jeton vert
  (base), cristal vert (renforcé), les autres options pas proposées (`will_focus_and_rejuvenate_mana_options`).
- **C9** — Euphorie cristalline jouée, fin du tour avec `reclaim` = Marche : `CardDiscarded { Marche }`, `CardReturnedToHand { Euphorie }`
  avant `TurnEnded` ; Euphorie est en main, pas dans la défausse ; le joueur pioche une carte de moins
  (`crystal_joy_comes_back_to_hand`).
- **C10** — Sans Euphorie jouée, toute `EndTurn` avec `reclaim` est refusée ; avec Euphorie jouée, la carte de `reclaim` ne peut pas
  être aussi dans `discard` (`reclaim_needs_crystal_joy`).
- **C11** — Fin du tour avec deux jetons non utilisés : après `TurnEnded`, plus aucun jeton ; l'inventaire n'a pas changé
  (`unused_tokens_return_to_the_bank`) — MKUE p. 11, 3.a ; p. 5, 1.a.

B. Dés supplémentaires (arcanum-core) :

- **C12** — Source de mana, effet de base : ensuite, deux dés peuvent être pris ce tour ; au tour suivant, de nouveau un seul
  (`mana_draw_allows_one_more_die`) — scan ; MKUE p. 5, 3.
- **C13** — Attraction de mana, effet de base, de jour, un dé noir épuisé : il est proposé pour payer une carte de n'importe quelle
  couleur, comme le dé de plus, même si le dé régulier du tour n'a pas encore été pris ; un deuxième dé noir ne l'est pas
  (`mana_pull_turns_a_black_die_into_any_color`) (Q3, FAQ p2).
- **C14** — Source de mana renforcée (payée en blanc), dé 1 choisi, vert : `CardPlayed`, `DieSet { 1, vert }`, deux `ManaTokenGained
  { vert }` ; le dé 1 est pris, sur vert ; le dé du tour n'est pas consommé (`mana_draw_sets_a_die_and_gains_two_tokens`) (Q2).
- **C15** — Attraction de mana renforcée : deux dés, deux couleurs, un jeton de chacune (`mana_pull_sets_two_dice`).
- **C16** — Fin du tour : un dé mis sur vert revient vert dans la Source, sans tirage ; un dé pris normalement est relancé ; la même
  graine donne la même Source qu'un tour sans dé mis sur une couleur, pour les dés relancés (`set_die_is_not_rerolled`) — scan.
- **C17** — L'or n'est jamais proposé comme couleur d'un dé mis ; le noir non plus (Q4) (`set_die_never_becomes_gold`).

C. Concentration renforcée (arcanum-core) :

- **C18** — Concentration renforcée avec Marche, payée avec le dé vert : `DieTaken`, `CardPlayed { Concentration }`, `CardPlayed
  { Marche, renforcée, Mouvement 6 }` ; les deux cartes sont dans la zone de jeu ; aucun autre mana dépensé
  (`concentration_powers_another_card_for_free_plus_two`) — scan ; MKUE p. 5, « Composite ».
- **C19** — Pouvoir de concentration avec Promesse : Influence 7 (`will_focus_adds_three`).
- **C20** — Concentration avec Cristallisation : un cristal de la couleur choisie, sans bonus ; avec Source de mana : un dé mis sur une
  couleur et deux jetons, sans bonus (`concentration_bonus_only_for_points`) — MKUE p. 5, « Composite » 1.b.
- **C21** — Concentration avec Improvisation : une troisième carte à défausser, Mouvement 7 (`concentration_with_improvisation`).
- **C22** — Concentration renforcée n'est pas proposée sans autre carte jouable renforcée hors combat (main : Concentration, Rage,
  Détermination) ; ni avec une autre Concentration (Q5) (`concentration_needs_a_target`).

Journal (arcanum-core) :

- **C23** — Tous les nouveaux événements sont publics dans le journal filtré ; rejouer le journal d'un tour avec jetons, cristaux, dés mis
  et reprise redonne le même état (`mana_cards_replay_and_are_public`).
- **C24** — 200 tours aléatoires avec les decks des 4 héros : jamais de jeton ni de cristal négatif, jamais plus de 3 cristaux par
  couleur, jamais plus de dés pris (hors dés mis) que la limite du tour (`mana_cards_never_overspend`).

Affichage (arcanum-ui) :

- **C25** — La zone de jeu montre une ligne « Jetons : » avec une pastille par jeton, dans l'ordre où ils ont été gagnés ; info-bulle
  « Jeton de mana bleu : à utiliser ce tour, rendu à la réserve à la fin du tour. » (`play_area_shows_mana_tokens`).
- **C26** — Les boutons des options de mana : « Jeton bleu », « Cristal vert », « Payer rouge : cristal rouge » ; grisés avec la raison
  si l'on ne peut pas payer (`hand_shows_mana_options`).
- **C27** — En mode choix du mana (022), les jetons s'ajoutent : « Jeton bleu » (`mana_choice_lists_tokens`).
- **C28** — Source de mana renforcée : mode « Choisissez un dé de la Source, puis sa couleur. » ; les dés de la Source deviennent
  cliquables, puis quatre boutons de couleur ; Attraction de mana : deux fois, les dés dans n'importe quel ordre ; « Annuler » ou Échap
  à chaque étape ; le dernier choix joue la carte s'il n'y a qu'un paiement, sinon ouvre le choix du mana (022)
  (`set_die_mode_chooses_die_then_color`, `set_die_mode_then_chooses_mana`, `mana_pull_sets_two_dice_one_after_the_other`). Précisé
  à l'implémentation : ordre libre des dés, choix du mana après les dés.
- **C29** — Concentration renforcée : mode « Concentration : choisissez la carte à renforcer gratuitement (+2). » ; chaque carte possible
  montre « Renforcer avec Concentration : Mouvement 6 » ; les autres sont grisées ; puis, s'il le faut, le choix de la défausse, de la
  couleur ou du mana de Concentration (`concentration_mode_lists_targets`, `concentration_with_mana_draw_sets_a_die`). Précisé à
  l'implémentation : les cartes qui ne peuvent pas être renforcées n'ont aucun bouton, comme dans les autres modes ; la zone de jeu
  montre la carte renforcée « Marche (renforcée gratuitement) : Mouvement 6 » ; sans autre carte à renforcer hors combat, le bouton
  de Concentration est grisé avec « Aucune autre carte à renforcer hors combat. » (`concentration_without_target_says_why`).
- **C30** — Mode fin du tour avec Euphorie cristalline jouée : une ligne « Reprendre Euphorie cristalline en défaussant : » et un menu des
  cartes de la main (aucune par défaut) ; le nombre de cartes piochées en tient compte (`end_turn_mode_offers_crystal_joy`).
- **C31** — La ligne Source marque un dé mis sur une couleur : info-bulle « Dé de mana vert : mis sur vert ce tour, rendu sans être
  relancé. » (`panel_shows_set_die`).
- **C32** — Avec le vrai `fr.json`, l'écran de chaque cas ci-dessus ne relève aucun manque (`every_mana_card_key_exists_in_french`).

Vérification manuelle : `cargo run -p arcanum-ui` sous Linux avec Goldyx (Pouvoir de concentration, Euphorie cristalline) et Arythea
(Attraction de mana) : gagner un jeton, s'en servir, gagner un cristal, reprendre Euphorie, mettre un dé sur une couleur. Partie C :
Concentration renforcée avec Improvisation (défausse puis mana) et avec Attraction de mana (deux dés et leurs couleurs), captures
`spec-025c-*.png` des fichiers du projet.

## Cas limites et refus

- **L1** — Paiement de Cristallisation, effet de base, avec un cristal de la même couleur : permis (sans intérêt, mais la carte est jouée,
  ce qui compte pour MKUE p. 6, 5.a).
- **L2** — Jeton noir : n'existe pas tant que le noir n'est pas proposé (Q4). Jeton or : n'existe jamais (aucune carte n'en donne).
- **L3** — Source de mana renforcée alors que tous les dés sont pris : pas proposée. Attraction de mana renforcée avec un seul dé libre :
  pas proposée (il faut deux dés).
- **L4** — Le dé de plus d'Attraction de mana « noir vaut toute couleur » ne vaut que pour un dé : le premier dé noir pris après elle.
- **L5** — Concentration renforcée payée avec le jeton qu'a donné une autre Concentration : permis.
- **L6** — Reprise d'Euphorie cristalline alors qu'elle a été jouée avec Concentration (comme carte renforcée gratuitement) : la reprise
  vaut aussi, l'effet est le sien.
- **L7** — Deux Euphorie cristalline jouées (impossible avec un seul exemplaire dans le deck de départ) : pas de cas.
- **L8** — `apply` de `ManaTokenUsed` sans jeton : rien ne change (même principe que `CrystalUsed`, 022).
- **L9** — Euphorie cristalline jouée **de côté** : pas de reprise. La reprise fait partie de son effet (texte de la carte), qu'une
  carte jouée de côté n'a pas (MKUE p. 4, 3.e). Précisé à l'implémentation.

## Information cachée

Tout est public : jetons, cristaux gagnés, dés mis sur une couleur, dés de plus, reprise. Sur la table, tout se voit.

## Libellés (i18n)

Ajoutés dans `fr.json` et `en.json` (couleurs : clés `mana.*` de la spec 020).

| Clé | Paramètres | Français | English |
|---|---|---|---|
| `effect.gain_token` | `color` | Jeton {color} | {color} mana token |
| `effect.gain_crystal` | `color` | Cristal {color} | {color} crystal |
| `effect.pay_for_crystal` | `color` | Payer {color} : cristal {color} | Pay {color}: {color} crystal |
| `effect.extra_die` | — | Un dé de plus ce tour | One more die this turn |
| `effect.extra_die_black` | — | Un dé de plus ce tour ; noir : toute couleur | One more die this turn; black: any color |
| `effect.set_dice` | `amount` | Mettre {count} dé(s) sur une couleur, gagner des jetons | Set {count} die/dice to a color, gain tokens |
| `effect.boost` | `amount` | Renforcer une autre carte gratuitement, +{amount} | Power another card for free, +{amount} |
| `play_area.tokens` | — | Jetons : | Tokens: |
| `play_area.token_tooltip` | `color` | Jeton de mana {color} : à utiliser ce tour, rendu à la réserve à la fin du tour. | {color} mana token: use it this turn, returned to the bank at the end of the turn. |
| `play_area.line_strong_token` | `card`, `color`, `effect` | {card} (renforcée, jeton {color}) : {effect} | {card} (powered, {color} token): {effect} |
| `mana_choice.token` | `color` | Jeton {color} | {color} token |
| `mana_choice.prompt_basic` | `card`, `effect` | {card} : {effect}. Choisissez le mana à payer. | {card}: {effect}. Choose the mana to pay. |
| `play.strong_no_mana` (modifiée, 022) | `color` | Il faut un mana {color} : un dé {color} ou or de la Source, un cristal ou un jeton {color}. | You need {color} mana: a {color} or gold die from the Source, a {color} crystal or a {color} token. |
| `play.strong_die_taken` (modifiée, 022) | `color` | Dé du tour déjà pris : il faut un cristal ou un jeton {color}. | Die already taken this turn: you need a {color} crystal or token. |
| `set_die.prompt` | — | Choisissez un dé de la Source, puis sa couleur. | Choose a die from the Source, then its color. |
| `source.die.set` | `color` | Dé de mana {color} : mis sur {color} ce tour, rendu sans être relancé. | {color} mana die: set to {color} this turn, returned without reroll. |
| `concentration.prompt` | `card`, `amount` | {card} : choisissez la carte à renforcer gratuitement (+{amount}). | {card}: choose the card to power for free (+{amount}). |
| `concentration.button` | `card`, `effect` | Renforcer avec {card} : {effect} | Power with {card}: {effect} |
| `concentration.no_target` (précisée à l'implémentation) | — | Aucune autre carte à renforcer hors combat. | No other card to power outside combat. |
| `play_area.line_boosted` (précisée à l'implémentation) | `card`, `effect` | {card} (renforcée gratuitement) : {effect} | {card} (powered for free): {effect} |
| `end_turn.reclaim` | `card` | Reprendre {card} en défaussant : | Take {card} back by discarding: |
| `end_turn.reclaim_none` | — | (ne pas reprendre) | (do not take back) |

## Interface

- **Zone de jeu** : ligne « Jetons : » avec les pastilles (C25).
- **Main** : boutons d'options de mana comme les autres options (C26) ; Concentration renforcée ouvre son mode (C29).
- **Source** : en mode « mettre un dé sur une couleur », les dés libres sont cliquables (C28) ; un dé mis est marqué (C31).
- **Fin du tour** : menu de reprise d'Euphorie cristalline (C30).

```
┌─ Panneau latéral ─────────────────────────────────┐
│ Source  🟩 ●  🟦 ●  pris : ◉🟥     épuisé : ⬛     │
│ Inventaire  ◆◆◇ rouge  ◆◇◇ vert  ◇◇◇ …            │
│ Main                                              │
│ ● Cristallisation  [Payer vert : cristal vert]    │
│                    [Renforcer : Cristal rouge] …  │
│ ● Marche           [Jouer] [Renforcer] [Côté…]    │
│ Zone de jeu                                       │
│ Jetons : ● bleu  ● bleu                           │
│ ● Concentration : Jeton bleu                      │
│ ● Endurance (renforcée, jeton bleu) : Mouvement 4 │
└───────────────────────────────────────────────────┘
```

### Raccourcis clavier

Échap annule les nouveaux modes (mettre un dé sur une couleur, Concentration), comme les modes des specs 013, 016 et 022 : ils ne sont
jamais ouverts en même temps. Pas d'autre touche : comme pour le choix du mana (022, Q6), choisir un dé ou une couleur au clavier ne
ferait pas gagner de temps. Reporté dans `docs/raccourcis.md` à l'implémentation.

## Effets sur les specs déjà implémentées

Signalés ici parce que le sens de tests existants change (règle « jamais modifier un test sans le signaler ») :

- **013** : `basic_action_definitions_match_cards` (`tests/jouer_des_cartes.rs`) : Concentration, Pouvoir de concentration,
  Cristallisation, Euphorie cristalline, Source de mana, Attraction de mana et Rajeunissement (en partie) ne sont plus « pas encore pris
  en charge ». `legal_actions_are_exhaustive` (013 et 022) : l'énumération gagne leurs options.
- **029** : `basic_actions.ron` remplace les `Unsupported` de ces 7 cartes ; les tests de chargement (`tests/contenu_en_donnees.rs`)
  gagnent les nouveaux refus (jeton or, `Boost(0)`…), mêmes principes.
- **016** : `EndTurn` gagne `reclaim` : tous les tests qui construisent `EndTurn { discard }` passent `reclaim: None`, même sens.
- **022** : C14 `out_of_scope_strong_effects_are_not_offered` : les cartes de mana en sortent ; `ManaPayment` gagne `Token` ; `Basic`
  gagne un paiement facultatif ; `DICE_PER_TURN` devient la limite de base, relevée par les dés de plus ; C24 « un seul paiement possible »
  compte aussi les jetons.
- **020** : `source_is_unchanged_by_turns` reste vrai (aucune carte de mana jouée).
- **023** : rien ne change ; un dé mis sur une couleur est rendu en fin de tour, avant une éventuelle annonce de fin de manche, et la
  Source est de toute façon relancée en entier au début de la manche suivante.
- **`PlayerView`**, construit champ par champ dans des tests, gagne les jetons, les dés de plus et les dés mis.

## Hors périmètre

- Noir pour les sorts, jetons noirs et dé noir de nuit pour les sorts : spec 052 (Q4).
- Soin et pioche de Tranquillité et Rajeunissement : spec 034. Préparer une unité (Rajeunissement) : spec 042.
- Bonus de Concentration sur une Attaque ou une Parade : spec 031 (le +2 est déjà calculé pour tout effet de points, mais rien de combat
  n'est jouable).
- Effets composites des actions avancées, sorts et artefacts (Magic Talent, Maximal Effect, Time Bending) : spec 052.
- Compétences, unités et tactiques qui donnent du mana ou des dés : specs 042, 051, 060.
- Annuler un jeu de carte qui a mis un dé sur une couleur : spec 024 (pas de hasard, donc pas de révélation ; à confirmer là-bas).

## Questions ouvertes

Aucune.

## Décisions

Tranchées par Mathieu le 2026-10-26 :

- **Q1** : une seule spec, implémentée en trois PR successives (A, B, C).
- **Q2** : les dés pris par Source de mana et Attraction de mana renforcées ne comptent pas dans la limite de dés du tour, et peuvent
  être épuisés. Confirmé pour la limite par la FAQ (`docs/regles/faq/p1.md`, Mana : un effet qui fait prendre un dé de la Source ne
  compte pas comme le dé qu'on choisit de prendre pour l'utiliser comme mana). Pour l'épuisé, la FAQ
  (p3, « Épuisement ») ne définit un dé épuisé que comme un mana noir ou or inutilisable ; mettre un dé sur une couleur n'utilise pas
  son mana.
- **Q3** : Attraction de mana permet un dé noir de jour ; confirmé par la FAQ (p2, Attraction de mana), qui précise que ce dé de plus
  peut être pris même sans avoir pris le dé régulier du tour.
- **Q4** : pas de noir comme couleur d'un dé mis, tant qu'il n'y a pas de sorts (052).
- **Q5** : pas de Concentration sur une autre Concentration (ni Pouvoir de concentration).
- **Q6** : reprise d'Euphorie cristalline choisie dans le mode fin du tour, champ `reclaim` de `EndTurn`.
- **Q7** : « Jeton de mana », « Gagner un jeton », « Mettre un dé sur une couleur », « Dé de plus », « Effet composite » (*composite
  effect*, MKUE p. 5), « Reprendre (une carte en main) ».
