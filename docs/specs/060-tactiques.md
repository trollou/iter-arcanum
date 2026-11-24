# 060 — Tactiques

- Statut : implémentée (le 2026-11-18 ; validée le 2026-11-18, Q1 à Q7 : propositions acceptées ; retrait des tactiques paramétré par le scénario) (reporter tout changement dans la colonne Statut de `PLAN.md`)
- Jalon : J6
- Dépend de : 023 (fin de manche, annonce, nouvelle manche), 016 (fin du tour, pioche), 020 et 025 (Source, dés, paiement du
  mana), 024 (retour en arrière, points de non-retour), 035 et 043 (limite de main, bonus des forteresses), 038 (générateurs par
  usage), 051 (Motivation), 057 (Distorsion temporelle, Méditation), 058 (Polarisation, Magie inutile), 063 (Première
  reconnaissance, dernier tour), 065 et 078 (écran de nouvelle partie) ; toutes implémentées
- Livret : MKUE p. 3 (*Displayed Day Tactics*, *Unused Tactics*) ; p. 4 (*One Round of the Game*, 2.f, 3.a à 3.d, 4.a) ; p. 11
  (*End of the Turn*, 7.b et 7.d) ; p. 15 (*Dummy Player*, *When picking Tactics*) ; p. 18 (*First Reconnaissance*, *Solo
  Variant*) ; p. 19 (*Solo Conquest*, *Special Rules*). Texte des cartes : `docs/regles/cartes/tactiques.md` ; noms français :
  `docs/regles/scans/fr/tactics/` (photos). FAQ p1 (*Vol de mana*, *Planification*), FAQ p2 (*Longue Nuit*). Compendium :
  *Tactic Cards* (S1 et chaque carte), *Turn Start* S1, *Scenarios* Q8, *Who Needs Magic* S3, *Motivation* S6, *Miscellaneous*
  (12)
- Crates touchées : arcanum-core, arcanum-persistance (tests), arcanum-i18n (libellés), arcanum-ui

## Objectif

Au début de chaque manche de Première reconnaissance, le joueur choisit une tactique parmi celles de Jour ou de Nuit qui restent,
et profite de son effet pendant la manche : pioche au choix (Lève-tôt n'a pas d'effet, Faux départ, Départ flamboyant,
Préparation), dé de mana gardé (Vol de mana), limite de main augmentée (Planification), tour en plus (Au bon moment), deck
regarni (Longue Nuit), dés relancés (Appel du mana), main renouvelée (Méditation nocturne), cartes mises de côté pour plus tard
(Provisions). Chaque tactique prise est retirée de la partie à la fin de la manche, comme dans la variante solo du livret.

L'ordre du tour et la tactique tirée par le joueur fantôme viennent avec lui (spec 061, Q1) : seul, le joueur joue tous les tours,
et le numéro de sa tactique ne change rien.

### Les douze tactiques

Noms français lus sur les photos de `docs/regles/scans/fr/tactics/`. « Au choix » : l'effet joue au moment où la tactique est
prise ; « manche » : elle sert pendant la manche, puis est retournée face cachée quand elle a servi (Compendium, *Miscellaneous*
(12)).

| N° | Jour | Quand | Nuit | Quand |
|---|---|---|---|---|
| 1 | Lève-tôt (*Early Bird*) | aucun effet | Au crépuscule (*From the Dusk*) | aucun effet |
| 2 | Faux départ (*Rethink*) | au choix | Longue Nuit (*Long Night*) | une fois dans la manche, deck vide |
| 3 | Vol de mana (*Mana Steal*) | au choix, puis manche | Appel du mana (*Mana Search*) | une fois par tour |
| 4 | Planification (*Planning*) | chaque fin de tour | Méditation nocturne (*Midnight Meditation*) | une fois, avant un tour |
| 5 | Départ flamboyant (*Great Start*) | au choix | Préparation (*Preparation*) | au choix |
| 6 | Au bon moment (*The Right Moment*) | une fois, pendant un tour | Provisions (*Sparing Power*) | avant chaque tour, jusqu'à retournement |

## Règles

### Le choix

- **Tactiques exposées** (MKUE p. 3, *Displayed Day Tactics* et *Unused Tactics* ; p. 4, 2.f) : les six de Jour à la mise en
  place, les six de Nuit à une manche de Nuit, et ainsi de suite. Rien n'est mélangé ni caché : aucun hasard.
- **Qui choisit et quand** (MKUE p. 4, 3.a ; p. 19, *Special Rules*, 1er point) : en solo, le joueur choisit toujours le premier,
  donc parmi toutes les tactiques exposées qui n'ont pas été retirées. Le choix suit la préparation de la manche (p. 4, 2.g : la
  main est déjà piochée, bannières reprises comprises, spec 055) et précède le tour 1. Première manche : il suit la mise en place.
  L'ordre par la gloire (3.a) n'a pas d'objet en solo.
- **Rien avant le choix** (Compendium, *Turn Start* S1, note †, et *Motivation* S6) : tant que la tactique n'est pas choisie, la
  seule action légale est de la choisir ; Motivation, la clairière magique et l'annonce de fin de manche attendent le tour 1.
- **« Quand vous prenez cette tactique »** (MKUE p. 4, 3.b) : Faux départ, Vol de mana, Départ flamboyant et Préparation jouent
  aussitôt, avant le tour 1 (Compendium, *Tactic Cards* S1 : une seule fois, au moment du choix).
- **Ordre du tour** (MKUE p. 4, 3.c et 4.a) : reporté à la spec 061 (Q1). Le numéro de la tactique est gardé dans l'état et la vue.
- **Tactiques retirées** (MKUE p. 19, *Special Rules*, 2e point ; Compendium, *Scenarios* Q8) : à la fin de chaque manche, la
  tactique prise est retirée de la partie ; elle ne sera plus proposée. Les tactiques non prises restent (la règle 3.d, *set
  aside*, les met de côté pour la manche seulement). Avec le joueur fantôme, sa tactique est retirée aussi (spec 061).
  **Le retrait est une règle du scénario, pas du jeu** (Mathieu, 2026-11-18) : hors du solo, les tactiques sont reprises à chaque
  manche (p. 4, 2.f : *Collect all Tactic cards from the previous Round*) et reviennent ; en coopératif avec fantôme ou contre
  Volkare, seule une partie est retirée (Compendium, *Scenarios* Q8). Seul le retrait solo est codé ici ; la structure (Modèle,
  `TacticRemoval`) accueille les autres sans refonte.
- **Scénarios** (Q2) : Première reconnaissance utilise les tactiques (MKUE p. 18, *Solo Variant* : « *special rules for Tactic
  selection from the Solo Conquest mission* ») ; le bac à sable non, pour garder ses tests et ses parties préparées. Le scénario
  dit sa règle des tactiques (point d'extension prévu par la spec 012, tableau du modèle).
- **Retournée face cachée** (Compendium, *Miscellaneous* (12)) : une tactique « une fois » est retournée quand elle a servi ; elle
  ne sert plus de la manche. Les tactiques sont reprises à la fin de la manche (p. 4, 2.f), face visible ou non.
- **Fin de la manche** (MKUE p. 4, 2.f et 2.g) : ce que porte la tactique revient à sa place avant le mélange du deck : le dé de
  Vol de mana à la Source (relancée de toute façon, 2.b), les cartes sous Provisions dans les cartes Acte qui forment le nouveau
  deck. Les effets « ce Jour », « cette Nuit » s'arrêtent avec la manche.

### Jour

- **Lève-tôt (1)** : aucun effet. Le plus petit numéro ne sert qu'à l'ordre du tour (spec 061).
- **Faux départ (2)** (carte ; Compendium, *Rethink* S1) : défausser de 0 à 3 cartes de la main, Blessures comprises ; piocher
  autant de cartes ; puis mélanger la défausse dans le deck. À cet instant la défausse ne contient que les cartes défaussées (la
  préparation de la manche a tout mélangé) : ce sont elles qui reviennent dans le deck. Un deck trop court pioche ce qu'il peut.
- **Vol de mana (3)** (carte ; FAQ p1 ; Compendium, *Mana Steal* Q1, *Who Needs Magic* S3) : prendre un dé d'une couleur de base
  de la Source et le poser sur la tactique ; la Source a un dé de moins tant qu'il y est. Pendant n'importe quel tour de ce Jour,
  ce dé paie un mana de sa couleur, **en plus** du dé de la Source du tour : ce n'est pas un dé de la Source (Magie inutile à +3
  ne l'interdit pas). Une fois utilisé, il est relancé à la fin de ce tour et rendu à la Source ; la tactique est retournée. Sans
  dé de couleur de base dans la Source, la tactique se prend sans rien voler (L3).
- **Planification (4)** (carte ; MKUE p. 11, 7.b ; FAQ p1 ; Compendium, *Planning* Q1, S2 à S4) : à chaque fin de tour, si la
  main compte au moins 2 cartes avant la pioche (après les défausses de fin de tour, Blessures comprises), la pioche va jusqu'à la
  limite de main **plus une**. Se cumule avec les forteresses et Méditation (S4) ; pas à la pioche de la manche suivante (S3) ;
  pas de pioche du tout après Distorsion temporelle (spec 057, inchangé).
- **Départ flamboyant (5)** (carte ; Compendium, *Great Start* S1) : piocher 2 cartes, sans rien défausser. Deck trop court :
  ce qu'il peut.
- **Au bon moment (6)** (carte ; Compendium, *The Right Moment* S1, S3, S4) : une fois dans ce Jour, pendant son tour et avant
  la fin du tour, le joueur annonce qu'il jouera un tour de plus ; la tactique est retournée. Le tour en cours finit en entier
  (pioche comprise), puis un **tour complet** commence. Seul, cela ne change rien d'ordinaire, sauf au **dernier tour** de
  Première reconnaissance (cité révélée, spec 063) : le tour de plus se joue avant la fin de la partie (S3). Avec Distorsion
  temporelle annoncée au même tour : le tour sans pioche de la Distorsion d'abord, puis la pioche et le tour d'Au bon moment (L7).

### Nuit

- **Au crépuscule (1)** : aucun effet.
- **Longue Nuit (2)** (carte ; MKUE p. 11, 7.d et sa note ; FAQ p2 ; Compendium, *Long Night* Q1, Q2) : une fois dans cette
  Nuit, quand le deck est vide, mélanger la défausse et en remettre 3 cartes au hasard dans le deck (moins si la défausse en a
  moins) ; le reste de la défausse y reste ; la tactique est retournée. Utilisable :
  - pendant son tour, quand le deck est vide (y compris au début du tour, à la place d'annoncer la fin de la manche) ;
  - **pendant la pioche de fin de tour** : quand le deck s'épuise avant la limite de main, la pioche s'arrête sur le choix
    d'utiliser Longue Nuit ou non ; si oui, elle continue dans les 3 cartes (Q5). Pas au milieu des défausses (Q2) : la défausse
    de fin de tour est faite d'un bloc avant la pioche.
- **Appel du mana (3)** (carte ; Compendium, *Mana Search* S1, S2) : une fois par tour, tant qu'aucun dé de la Source n'a été
  pris ce tour, relancer 1 ou 2 dés de la Source, choisis avant de lancer ; s'il y a des dés or (épuisés la nuit), ils doivent
  être choisis d'abord. Le résultat est gardé, même mauvais. Pas de retournement : une fois par tour toute la Nuit.
- **Méditation nocturne (4)** (carte ; Compendium, *Midnight Meditation* S2, *Turn Start* S1) : une fois dans cette Nuit,
  **avant un de ses tours**, mélanger de 1 à 5 cartes de la main (Blessures comprises, choisies par le joueur) dans le deck, puis
  piocher autant de cartes ; la tactique est retournée. En solo, « avant un tour » se lit : au début du tour, avant toute autre
  action (aucune carte jouée, aucun déplacement, rien de fait), y compris le tour 1 qui suit le choix de la tactique.
- **Préparation (5)** (carte ; Compendium, *Preparation* S1, S2) : chercher une carte au choix dans le deck (une Blessure est
  permise), la prendre en main, puis mélanger le deck. Une seule fois, au choix de la tactique.
- **Provisions (6)** (carte ; Compendium, *Sparing Power* S1, S2) : **avant chacun de ses tours**, tant que la tactique est face
  visible, le joueur **doit** choisir : poser la carte du dessus du deck face cachée sous la tactique, **ou** retourner la tactique
  et prendre en main toutes les cartes posées dessous. Le premier choix a lieu avant le tour 1 qui suit la prise de la tactique
  (S2). Deck vide : seul le second choix reste. Le choix précède les étapes du début du tour (Compendium, *Turn Start* S1, I avant
  II) : prendre les cartes en main évite l'annonce obligatoire de fin de manche d'une main et d'un deck vides (C33).

### Exemple : une manche de Nuit avec Provisions

Tovak, manche 2 (Nuit), main de 5 piochée, deck de 11. Le joueur prend Provisions (6). Avant le tour 1 : une carte du dessus
du deck passe sous la tactique (deck 10, sous Provisions 1). Avant le tour 2 : une autre (deck 9 après la pioche du tour 1,
puis 8). Avant le tour 4, la main est faible : il retourne Provisions, ses 3 cartes vont en main, et le tour commence avec elles.
À la fin de la Nuit, Provisions est retirée de la partie ; Au crépuscule, Longue Nuit, Appel du mana, Méditation nocturne et
Préparation restent pour la manche 4.

## Modèle

Termes du glossaire : Tactique (`Tactic`), Manche, Tour, Deck, Défausse, Limite de main, Source, Dé épuisé, Face visible / face
cachée, Tour supplémentaire (spec 057). **Termes nouveaux** (ajoutés au glossaire par cette PR) :

| Livret (EN) | Français | Code |
|---|---|---|
| Tactic card (the twelve) | Les douze tactiques : noms du tableau ci-dessus | `Tactic` (`EarlyBird` … `SparingPower`), `Tactic::number`, `Tactic::time_of_day` |
| Remove (a Tactic) from the game | Retirer (une tactique) de la partie | `GameState::removed_tactics` |
| Flip (a Tactic card) face down | Retourner (une tactique) | `TacticState::flipped` |
| Round Order (token) | Ordre du tour | spec 061 |

Données nouvelles :

- **Contenu** : aucun fichier RON. Une tactique n'a qu'un numéro et une mécanique : `Tactic` est une énumération Rust de douze
  valeurs (ADR 0005 : identifiants et mécaniques restent en Rust), avec `Tactic::ALL`, `number()`, `time_of_day()`, et ses clés
  `tactic.<id>.name` et `.text`.
- **Scénario** : sa règle des tactiques, `TacticRule` : `None` pour le bac à sable, ou `Chosen { order, removal }` pour
  Première reconnaissance. Deux paramètres séparés pour que d'autres scénarios s'ajoutent sans refonte (Mathieu, 2026-11-18) :
  - `order`, qui choisit en premier : `PlayerFirst` (solo, MKUE p. 19) ; plus tard l'ordre par la gloire (p. 4, 3.a) ;
  - `removal`, ce qui est retiré à la fin de la manche : `AllTaken` (solo : la tactique du joueur, et celle du fantôme avec la
    spec 061) ; plus tard `None` (reprises à chaque manche, p. 4, 2.f) et `OwnOnly` (seule celle du joueur, Compendium,
    *Scenarios* Q8). Seule `AllTaken` est codée ; les variantes futures ne sont écrites qu'au moment où un scénario en a besoin
    (rien hors de la spec), mais le retrait passe déjà par ce paramètre et non par un « si solo ».
  Champ sérialisé avec une valeur par défaut `None` : une sauvegarde d'avant cette spec se recharge et se joue sans tactiques
  (C40).
- **Partie** : les tactiques retirées ; « choix de la tactique en attente » (après la mise en place et après `RoundStarted`).
- **Joueur** : la tactique de la manche et son état (`TacticState` : tactique, retournée ou non, le dé volé et sa couleur, les
  cartes sous Provisions, Appel du mana utilisé ce tour, Au bon moment annoncé) ; les choix en attente qui suivent la prise
  (cartes de Faux départ, dé de Vol de mana, carte de Préparation) ; le choix de Provisions en attente au début du tour ; le
  choix de Longue Nuit en attente pendant la pioche.
- **Paiement** : une forme nouvelle de `ManaPayment`, le dé de Vol de mana (`ManaPayment::StolenDie`, nom au fil
  d'implémentation). Polarisation (spec 058) l'accepte comme un dé.
- **Hasard** : Faux départ, Méditation nocturne, Préparation et Longue Nuit mélangent avec le générateur du deck du joueur
  (`RngUsage::Deck`), Appel du mana et la relance du dé volé avec celui de la Source (`RngUsage::Source`) : pas d'usage nouveau
  (spec 038).

## Actions et événements

Noms indicatifs ; le fil d'implémentation peut regrouper les choix qui suivent la prise dans `ChooseTactic` si l'énumération des
actions légales reste raisonnable.

| Action | Validation (refusée si…) | Événements produits |
|---|---|---|
| `ChooseTactic { tactic }` (nouvelle) | pas de choix en attente ; tactique de l'autre moment de la journée, ou retirée | `TacticChosen` ; Départ flamboyant : `CardDrawn` ×2 ; puis `TurnStarted` (tour 1), sauf si un choix suit (Faux départ, Vol de mana, Préparation) ou Provisions |
| `RethinkDiscard { cards }` (nouvelle) | plus de 3 cartes ; carte absente de la main | `CardDiscarded` ×n, `CardDrawn` ×n, `DeckShuffled` (deck restant + défausse), `TurnStarted` |
| `StealDie { die }` (nouvelle) | dé absent, pris, ou pas d'une couleur de base | `DieStolen`, `TurnStarted` |
| `PrepareCard { card }` (nouvelle) | carte absente du deck | `CardSearched`, `DeckShuffled`, `TurnStarted` |
| Paiement par le dé volé (dans toute action qui paie) | pas de dé volé ; couleur différente (sauf Polarisation) | `StolenDieUsed`, puis l'effet comme aujourd'hui |
| `EndTurn` (existante) | inchangée | en plus : Planification, pioche + 1 ; dé volé utilisé : `StolenDieReturned` (relancé) ; Au bon moment annoncé : après la pioche, `TurnStarted` d'un tour de plus ; Longue Nuit disponible et deck épuisé avant la limite : la pioche s'arrête (choix en attente) |
| `AnnounceRightMoment` (nouvelle) | pas Au bon moment ; retournée ; hors de son tour ou après la fin du tour engagée ; choix en attente | `RightMomentAnnounced` |
| `UseLongNight` (nouvelle) | pas Longue Nuit ; retournée ; deck non vide ; défausse vide | `LongNightUsed { cards }` (les cartes remises, tirées au hasard), et la suite de la pioche si elle était arrêtée |
| `DeclineLongNight` (nouvelle) | pas de pioche arrêtée | la fin de la pioche (rien), puis la suite de la fin du tour |
| `ManaSearch { dice }` (nouvelle) | pas Appel du mana ; déjà utilisée ce tour ; un dé de la Source déjà pris ce tour ; 0 ou plus de 2 dés ; un dé or laissé alors qu'un dé non or est choisi | `DieRerolled` ×n (existant) |
| `MidnightMeditation { cards }` (nouvelle) | pas Méditation nocturne ; retournée ; 0 ou plus de 5 cartes ; carte absente de la main ; tour déjà entamé | `CardsShuffledIntoDeck`, `DeckShuffled`, `CardDrawn` ×n |
| `SparingPower { take }` (nouvelle) | pas de choix de Provisions en attente ; `take = false` avec deck vide | `CardStored` (dessus du deck sous la tactique) ou `StoredCardsTaken` (tactique retournée, cartes en main) ; puis `TurnStarted` s'il attendait |
| `AnnounceEndOfRound` (existante) | inchangée ; en plus, refusée tant que la tactique n'est pas choisie ou que Provisions attend | inchangés ; `RoundEnded` retire la tactique prise |

| Événement | Effet de `apply` | Révèle une information ? | Point de non-retour ? |
|---|---|---|---|
| `TacticChosen { player, tactic }` | tactique du joueur, choix terminé | non (public) | non ; mais `TurnStarted` le suit dans la même décision (C3) |
| `DieStolen { player, die }` | le dé quitte la Source et se pose sur la tactique | non | non |
| `StolenDieUsed { player }` | le dé volé est dépensé ; tactique retournée | non | non |
| `StolenDieReturned { player, color }` | le dé, relancé, revient à la Source | oui (dé lancé) | oui |
| `CardSearched { player, card }` | la carte passe du deck à la main | non | non (le `DeckShuffled` qui suit l'est) |
| `RightMomentAnnounced { player }` | tour de plus en attente ; tactique retournée | non | non |
| `LongNightUsed { player, cards }` | les cartes passent de la défausse au deck ; tactique retournée | oui (hasard) | oui |
| `CardsShuffledIntoDeck { player, cards }` | les cartes passent de la main au deck ; tactique retournée | non | non (le `DeckShuffled` qui suit l'est) |
| `CardStored { player, card }` | carte du dessus du deck sous Provisions | non (face cachée, Q4) | oui (carte inconnue tirée) |
| `StoredCardsTaken { player }` | cartes sous Provisions en main ; tactique retournée | non | non |
| `TacticsRemoved { tactics }` (à la fin de la manche, avec `RoundEnded`) | tactiques retirées de la partie | non | oui (avec `RoundEnded`) |

`CardDrawn`, `CardDiscarded`, `DeckShuffled`, `DieRerolled`, `TurnStarted`, `RoundEnded` existent et restent des points de non-retour
quand ils l'étaient (spec 024).

## Comportements attendus

Chaque cas devient au moins un test (nom indicatif entre parenthèses). Sauf mention, la partie est Première reconnaissance avec
Tovak (`Scenario::first_reconnaissance`, `heroes.ron`), limite de main 5 au niveau 1 (`levels.ron`).

### Choix et retrait

- **C1** — Après la mise en place de Première reconnaissance, la décision en attente est le choix de la tactique : les actions
  légales sont exactement les six `ChooseTactic` de Jour, dans l'ordre des numéros ; la main est déjà piochée (5 cartes)
  (`first_round_starts_with_tactic_choice`) — MKUE p. 4, 3.a ; p. 3.
- **C2** — Tant que le choix attend : pas de carte jouable, pas de déplacement, pas de Motivation (Tovak n'a pas la compétence
  au départ : test sur un état posé avec Motivation gagnée, manche 2), pas d'annonce de fin de manche
  (`nothing_before_tactic_choice`) — Compendium, *Motivation* S6, *Turn Start* S1.
- **C3** — Choisir Lève-tôt produit `TacticChosen` puis `TurnStarted` ; la vue montre Lève-tôt, numéro 1, face visible ; le
  retour en arrière ne revient pas avant le choix (`choosing_a_tactic_starts_the_turn`) — MKUE p. 4, 3 ; spec 024.
- **C4** — Bac à sable : aucun choix de tactique, le tour 1 commence comme aujourd'hui (tests existants inchangés, ex.
  `fin_de_manche.rs`, `new_round_flips_day_to_night`) (`sandbox_has_no_tactics`) — Q2.
- **C5** — Annonce de la fin de la manche 1 avec Planification prise : à la manche 2 (Nuit), après la pioche, le choix porte sur
  les six tactiques de Nuit (`night_round_offers_night_tactics`) — MKUE p. 4, 2.f.
- **C6** — À la manche 3 (Jour), Planification n'est plus proposée : cinq choix ; à la manche 4, la tactique de Nuit prise en
  manche 2 non plus (`taken_tactic_is_removed_at_round_end`) — MKUE p. 19, *Special Rules* ; Compendium, *Scenarios* Q8.
- **C7** — Avec des bannières à reprendre (spec 055), le choix de la tactique vient après la reprise et la pioche
  (`tactic_choice_follows_banner_reclaim`) — MKUE p. 4, 2.g puis 3.
- **C8** — Une clairière magique sous la figurine donne son jeton au début du tour 1, donc après le choix
  (`glade_token_after_tactic_choice`) — spec 044 ; Compendium, *Turn Start* S1, II.

### Jour

- **C9** — Faux départ, défausser 2 cartes de la main : la défausse les reçoit, 2 cartes sont piochées, puis `DeckShuffled`
  remet ces 2 cartes dans le deck (deck 11 − 2 + 2 = 11, main 5, défausse vide) ; le tour 1 commence
  (`rethink_discards_draws_and_reshuffles`) — carte ; Compendium, *Rethink* S1.
- **C10** — Faux départ avec 0 carte : rien ne change, le tour commence ; avec une Blessure en main (état posé, manche 3), elle
  peut être défaussée (`rethink_may_discard_wounds`) — carte.
- **C11** — Vol de mana : les actions légales sont les dés de couleur de base de la Source ; le dé choisi quitte la Source (la vue
  montre un dé de moins et le dé sur la tactique) (`mana_steal_takes_a_basic_die`) — carte.
- **C12** — Avec le dé volé bleu, jouer Endurance renforcée (Mouvement 2 ; renforcé bleu : Mouvement 4) en le payant, puis prendre
  un dé de la Source pour une autre carte le même tour : les deux paiements sont légaux (`stolen_die_is_not_the_source_die`) —
  FAQ p1 ; Compendium, *Mana Steal* Q1.
- **C13** — À la fin du tour où il a servi, le dé volé est relancé et rendu à la Source (`StolenDieReturned`) ; la tactique est
  retournée ; aux tours suivants, plus de dé volé (`stolen_die_returns_after_use`) — carte.
- **C14** — Dé volé jamais utilisé : il reste sur la tactique de tour en tour ; à la manche suivante, la Source a de nouveau son
  compte de dés (`unused_stolen_die_returns_with_new_round`) — MKUE p. 4, 2.b.
- **C15** — Magie inutile à +3 utilisée (état posé, spec 058) : le dé volé paie encore (`who_needs_magic_allows_stolen_die`) —
  Compendium, *Who Needs Magic* S3.
- **C16** — Planification, fin de tour avec 2 cartes gardées en main : pioche jusqu'à 6 ; avec 1 carte gardée : jusqu'à 5
  (`planning_draws_one_more_with_two_cards`) — carte ; MKUE p. 11, 7.b.
- **C17** — Planification, 2 cartes gardées dont une Blessure : pioche jusqu'à 6 (`planning_counts_wounds`) — FAQ p1 ;
  Compendium, *Planning* Q1.
- **C18** — Planification et forteresse conquise voisine (+1, spec 043) : pioche jusqu'à 7 (`planning_stacks_with_keep`) —
  Compendium, *Planning* S4.
- **C19** — Départ flamboyant : 2 cartes piochées aussitôt, main de 7, rien de défaussé (`great_start_draws_two`) — carte ;
  Compendium, *Great Start* S1.
- **C20** — Au bon moment annoncé au tour 3 : le tour 3 finit avec sa pioche, puis `TurnStarted` du tour 4 ; la tactique est
  retournée et l'annonce n'est plus légale de la manche (`right_moment_grants_a_full_turn`) — carte ; Compendium, *The Right
  Moment* S4.
- **C21** — Dernier tour de Première reconnaissance (cité révélée, `LastTurn::Current`, spec 063) : Au bon moment annoncé, le
  joueur joue encore un tour complet avant `GameEnded` (`right_moment_extends_last_turn`) — Compendium, *The Right Moment* S3 ;
  *Scenarios* O30.
- **C22** — Au bon moment n'est pas légal pendant un choix en attente, ni une fois la fin du tour engagée (choix de fin de tour
  en cours) (`right_moment_only_during_the_turn`) — Compendium, *The Right Moment* S1.

### Nuit

- **C23** — Longue Nuit, deck vide au début du tour, défausse de 6 cartes : `UseLongNight` est légale, à côté de l'annonce de fin
  de manche ; utilisée, 3 cartes tirées au hasard passent dans le deck, 3 restent en défausse, la tactique est retournée
  (`long_night_returns_three_cards`) — carte ; FAQ p2.
- **C24** — Fin de tour avec Longue Nuit face visible, deck de 2 cartes pour 4 à piocher : la pioche s'arrête après 2 cartes,
  la décision en attente est `UseLongNight` ou `DeclineLongNight` ; utilisée, la pioche continue (2 cartes de plus prises dans les
  3 remises) ; refusée, le tour suivant commence avec la main incomplète (`long_night_during_end_of_turn_draw`) — MKUE p. 11,
  7.d, note ; Compendium, *Long Night* Q1.
- **C25** — Longue Nuit retournée, ou défausse vide : ni `UseLongNight` ni arrêt de la pioche (`long_night_once_per_night`) —
  carte.
- **C26** — Appel du mana au début du tour, Source avec un dé or et un dé vert : relancer le vert seul est refusé ; relancer l'or
  seul, ou l'or et le vert, est légal ; les dés relancés ont leur nouvelle couleur (`mana_search_rerolls_gold_first`) — carte.
- **C27** — Appel du mana après avoir pris un dé de la Source ce tour : refusé ; une seconde fois le même tour : refusée ; au tour
  suivant : de nouveau légale (`mana_search_once_per_turn_before_source`) — carte ; Compendium, *Mana Search* S2.
- **C28** — Méditation nocturne au début du tour, 3 cartes de la main (dont une Blessure) : elles vont au deck, le deck est
  mélangé, 3 cartes sont piochées ; la tactique est retournée (`midnight_meditation_swaps_cards`) — carte ; Compendium,
  *Midnight Meditation* S2.
- **C29** — Méditation nocturne après une carte jouée ou un déplacement ce tour : refusée ; avant le tour 1, juste après le choix
  de la tactique : légale (`midnight_meditation_before_acting`) — carte ; Compendium, *Turn Start* S1.
- **C30** — Préparation : les actions légales sont les cartes du deck (une par carte distincte) ; la carte choisie passe en main,
  puis le deck est mélangé (`preparation_searches_the_deck`) — carte ; Compendium, *Preparation* S1, S2.
- **C31** — Provisions : avant le tour 1, la décision en attente est `SparingPower` ; poser une carte retire la carte du dessus
  du deck ; la vue montre 1 carte sous la tactique, sans dire laquelle (`sparing_power_stores_top_card`) — carte ; Q4.
- **C32** — Provisions, au début de chaque tour suivant, le même choix revient ; retourner la tactique met toutes les cartes
  posées en main et le choix ne revient plus (`sparing_power_takes_stored_cards`) — Compendium, *Sparing Power* S2.
- **C33** — Provisions, deck et main vides au début du tour avec 2 cartes sous la tactique : seul le retournement est légal ;
  après lui, l'annonce de fin de manche n'est plus obligatoire (`sparing_power_before_forced_announcement`) — Compendium, *Turn
  Start* S1, I avant II ; MKUE p. 6, 3.b.
- **C34** — Fin de la Nuit avec 2 cartes encore sous Provisions : elles sont dans le nouveau deck mélangé
  (`stored_cards_return_with_new_round`) — MKUE p. 4, 2.g.

### Vue, retour en arrière, persistance

- **C35** — La vue montre : les tactiques proposées pendant le choix (`PlayerView::tactic_choice`), les tactiques retirées
  (`PlayerView::removed_tactics`), la tactique du joueur et son état (`PlayerView::tactic` : tactique, numéro, retournée, dé
  volé et couleur, nombre de cartes sous Provisions, Appel du mana utilisé ce tour, Au bon moment annoncé), les cartes du deck
  pendant le choix de Préparation (`PlayerView::preparation_choice`) (`view_shows_tactics`).
- **C36** — Le journal filtré pour un autre joueur montre `TacticChosen`, `DieStolen`, `RightMomentAnnounced`,
  `TacticsRemoved` en entier ; `CardSearched` et `CardStored` sans la carte (`tactic_events_are_filtered`) — spec 000.
- **C37** — L'annonce d'Au bon moment se reprend par le retour en arrière tant qu'aucun point de non-retour ne l'a suivie ;
  la prise d'une tactique ne se reprend pas (`TurnStarted` la suit, C3) (`right_moment_can_be_undone`) — spec 024.
- **C38** — Rejouer le journal d'une partie de quatre manches avec une tactique de chaque sorte redonne le même état
  (`replaying_log_rebuilds_tactics`) — ADR 0001.
- **C39** — Sauvegarder pendant le choix de la tactique, puis recharger : le choix attend toujours, avec les mêmes tactiques
  (`save_during_tactic_choice`, arcanum-persistance) — spec 024.
- **C40** — Une sauvegarde de Première reconnaissance d'avant cette spec (scénario sans `TacticRule` dans `GameStarted`) se
  recharge et se joue sans tactiques (`old_save_has_no_tactics`, arcanum-persistance) — spec 024 (pas de migration avant la fin du J6).

## Cas limites et refus

- **L1** — `ChooseTactic` d'une tactique de Nuit en manche de Jour, ou retirée : refusée (`tactic_must_be_available`).
- **L2** — `ChooseTactic` hors du choix (pendant un tour) : refusée.
- **L3** — Vol de mana sans dé de couleur de base dans la Source (état posé : Source tout or et noir) : la tactique se prend,
  rien n'est volé, le tour 1 commence ; la vue ne montre pas de dé (`mana_steal_without_basic_die`).
- **L4** — Faux départ ou Départ flamboyant avec un deck trop court (état posé) : pioche ce qu'il peut ; Faux départ remet quand
  même les cartes défaussées dans le deck.
- **L5** — Préparation avec un deck vide (état posé) : rien à chercher, le tour 1 commence.
- **L6** — Appel du mana avec un seul dé dans la Source : 1 dé au plus ; avec deux dés or et un vert : les deux dés or sont les
  seuls choix à deux dés.
- **L7** — Au bon moment et Distorsion temporelle (spec 057) au même tour : le tour supplémentaire sans pioche de la Distorsion
  d'abord ; à sa fin, la pioche ordinaire, puis le tour d'Au bon moment (`right_moment_after_time_bending`).
- **L8** — Au bon moment annoncé, puis la fin de la manche annoncée au début du tour de plus (deck vide) : la manche finit ; le
  tour de plus est perdu comme n'importe quel tour (MKUE p. 4, 4.b).
- **L9** — Une seule tactique encore disponible (scénario de test à plus de quatre manches, ou état posé) : le choix n'offre
  qu'elle ; aucune disponible : pas de choix, le tour commence sans tactique (Compendium, *Variants* S5, cas qu'aucun scénario du
  jeu n'atteint en solo).

## Information cachée

- Les tactiques exposées, retirées et prises sont publiques ; le dé volé et sa couleur aussi.
- Préparation montre au joueur les cartes de son deck (lesquelles, pas leur ordre) pendant ce seul choix ; le joueur peut de
  toute façon les déduire (toutes ses cartes Acte moins la main, la défausse et la zone de jeu).
- Provisions : les cartes posées sous la tactique ne sont connues de personne, le joueur compris (Q4) : la vue n'en donne que le
  nombre jusqu'au retournement.
- Longue Nuit : les cartes remises sont tirées au hasard ; le joueur voit lesquelles quittent sa défausse (la défausse est
  publique, Compendium, *Long Night* Q1), pas leur ordre dans le deck.

Champs de la vue ajoutés (C35) : `PlayerView::tactic_choice`, `removed_tactics`, `tactic`, `preparation_choice`, et les choix en
attente (Faux départ, Vol de mana, Provisions, Longue Nuit) dans la décision en attente comme les autres choix.

## Libellés (i18n)

Noms des douze tactiques : ceux des cartes françaises (photos) et anglaises (scans). Textes réécrits par nous, courts, sans
recopier la carte (CLAUDE.md, *Assets et droits*). Clés nouvelles, en fr et en :

| Clé | Paramètres | fr | en |
|---|---|---|---|
| `tactic.early_bird.name` / `.text` | — | Lève-tôt / Aucun effet. | Early Bird / No effect. |
| `tactic.rethink.name` / `.text` | — | Faux départ / En la prenant : défaussez jusqu'à 3 cartes (Blessures comprises), piochez-en autant, puis remélangez la défausse dans le deck. | Rethink / When taken: discard up to 3 cards (Wounds included), draw as many, then shuffle the discard pile into your deck. |
| `tactic.mana_steal.name` / `.text` | — | Vol de mana / En la prenant : posez ici un dé de couleur de base de la Source. Ce Jour, il paie un mana en plus du dé du tour ; relancé et rendu à la fin du tour où il sert. | Mana Steal / When taken: put a basic-color Source die here. This Day, it pays one mana besides your Source die; rerolled and returned at the end of the turn you use it. |
| `tactic.planning.name` / `.text` | — | Planification / Fin du tour : avec au moins 2 cartes en main avant de piocher, limite de main + 1. | Planning / End of turn: with at least 2 cards in hand before drawing, Hand limit + 1. |
| `tactic.great_start.name` / `.text` | — | Départ flamboyant / En la prenant : piochez 2 cartes. | Great Start / When taken: draw 2 cards. |
| `tactic.right_moment.name` / `.text` | — | Au bon moment / Une fois ce Jour, pendant votre tour : annoncez un tour de plus, joué juste après celui-ci. | The Right Moment / Once this Day, during your turn: announce another turn, played right after this one. |
| `tactic.from_the_dusk.name` / `.text` | — | Au crépuscule / Aucun effet. | From the Dusk / No effect. |
| `tactic.long_night.name` / `.text` | — | Longue Nuit / Une fois cette Nuit, deck vide : 3 cartes de la défausse, au hasard, retournent dans le deck. | Long Night / Once this Night, with an empty deck: 3 random cards from your discard pile go back into your deck. |
| `tactic.mana_search.name` / `.text` | — | Appel du mana / Une fois par tour, avant de prendre un dé de la Source : relancez jusqu'à 2 dés, les dés or d'abord. | Mana Search / Once per turn, before taking a Source die: reroll up to 2 dice, gold dice first. |
| `tactic.midnight_meditation.name` / `.text` | — | Méditation nocturne / Une fois cette Nuit, au début d'un tour : remélangez jusqu'à 5 cartes de la main dans le deck et piochez-en autant. | Midnight Meditation / Once this Night, at the start of a turn: shuffle up to 5 cards from your hand into your deck and draw as many. |
| `tactic.preparation.name` / `.text` | — | Préparation / En la prenant : prenez en main une carte de votre choix dans le deck, puis mélangez-le. | Preparation / When taken: put any card from your deck into your hand, then shuffle it. |
| `tactic.sparing_power.name` / `.text` | — | Provisions / Avant chaque tour : posez ici la carte du dessus du deck, ou retournez cette tactique et prenez en main les cartes posées. | Sparing Power / Before each turn: put the top card of your deck here, or flip this Tactic and take the cards here into your hand. |
| `tactic.choose_title` | `time` | Choisir une tactique ({time}) | Choose a Tactic ({time}) |
| `tactic.take` | — | Prendre cette tactique | Take this Tactic |
| `tactic.flipped` | — | Retournée : a servi | Flipped: used |
| `tactic.number` | `number` | N° {number} | No. {number} |
| `tactic.rethink_prompt` | — | Faux départ : cochez jusqu'à 3 cartes à défausser | Rethink: check up to 3 cards to discard |
| `tactic.rethink_confirm` | `count` | Défausser et piocher {count} carte(s) | Discard and draw {count} card(s) |
| `tactic.steal_prompt` | — | Vol de mana : choisissez un dé de couleur de base | Mana Steal: choose a basic-color die |
| `tactic.stolen_die` | `color` | Dé volé : {color} | Stolen die: {color} |
| `tactic.preparation_prompt` | — | Préparation : choisissez la carte à prendre en main | Preparation: choose the card to take into your hand |
| `tactic.right_moment_button` | — | Jouer un tour de plus | Take another turn |
| `tactic.right_moment_announced` | — | Au bon moment : un tour de plus après celui-ci | The Right Moment: another turn after this one |
| `tactic.long_night_button` | — | Remettre 3 cartes dans le deck | Put 3 cards back into the deck |
| `tactic.long_night_prompt` | — | Deck épuisé : utiliser Longue Nuit pour continuer la pioche ? | Deck empty: use Long Night to keep drawing? |
| `tactic.long_night_decline` | — | Non, arrêter la pioche | No, stop drawing |
| `tactic.mana_search_button` | — | Relancer des dés | Reroll dice |
| `tactic.mana_search_prompt` | — | Appel du mana : choisissez 1 ou 2 dés (or d'abord) | Mana Search: choose 1 or 2 dice (gold first) |
| `tactic.mana_search_confirm` | `count` | Relancer {count} dé(s) | Reroll {count} die/dice |
| `tactic.meditation_button` | — | Remélanger des cartes | Shuffle cards back |
| `tactic.meditation_prompt` | — | Méditation nocturne : cochez jusqu'à 5 cartes | Midnight Meditation: check up to 5 cards |
| `tactic.meditation_confirm` | `count` | Remélanger et piocher {count} carte(s) | Shuffle back and draw {count} card(s) |
| `tactic.stored` | `count` | Dessous : {count} carte(s) | Under it: {count} card(s) |
| `tactic.sparing_prompt` | `count` | Provisions ({count} carte(s) dessous) : avant ce tour | Sparing Power ({count} card(s) under it): before this turn |
| `tactic.sparing_store` | — | Poser la carte du dessus du deck | Put the top card of the deck here |
| `tactic.sparing_take` | `count` | Retourner et prendre {count} carte(s) | Flip and take {count} card(s) |

Exemples avec paramètres : `tactic.choose_title` avec `{time}` = « Jour » (`time_of_day.day` = « Jour », relu) : « Choisir une tactique
(Jour) » ; `tactic.stolen_die` avec `{color}` = « bleu » (`mana.blue`) : « Dé volé : bleu » ; `tactic.sparing_take` avec
`{count}` = 3 : « Retourner et prendre 3 carte(s) » (accord par la parenthèse, comme `skill.throw_wounds`).

Clés réutilisées, relues dans `assets/i18n/fr.json` de `main` : `time_of_day.day` = « Jour », `time_of_day.night` = « Nuit »,
`mana.red` … `mana.black` (`mana.blue` = « bleu »). **Clé modifiée** : `scenario.first_reconnaissance.missing`, aujourd'hui
« Pas encore dans le jeu : le joueur fantôme et le choix des tactiques de la variante solo. On peut s'en passer pour une première
partie. », devient « Pas encore dans le jeu : le joueur fantôme de la variante solo. On peut s'en passer pour une première
partie. » (en : « Not in the game yet: the solo variant's Dummy player. You can do without it for a first game. ») ; et
`scenario.first_reconnaissance.special_rules` gagne une ligne : « Tactiques : vous choisissez la vôtre en premier ; elle est
retirée de la partie à la fin de la manche. » (en : « Tactics: you choose yours first; it is removed from the game at the end of
the Round. »). Aucun symbole hors des polices du jeu.

## Interface

- **Fenêtre de choix de la tactique** (modale, comme le choix d'un artefact) : au début de la manche, titre
  `tactic.choose_title` ; les six tactiques du moment en **grille de 2 lignes et 3 colonnes** (Mathieu, 2026-11-18), chaque
  tactique à la place fixe de son numéro (1, 2, 3 en haut ; 4, 5, 6 en bas) : grand numéro, nom, texte. Une tactique retirée de
  la partie laisse un **trou** à sa place (emplacement vide), les autres ne bougent pas. Plus tard (spec 061 et parties à
  plusieurs), une tactique déjà prise cette manche par un autre joueur restera à sa place, grisée, avec l'icône de ce joueur :
  la grille est prévue pour cet état, sans le coder ici. Un clic choisit la carte (encadrée),
  le bouton `tactic.take` la prend : le choix ouvre le tour et ne se reprend pas (C1, C3, C6, `PlayerView::tactic_choice`,
  `removed_tactics`). Main visible sous la fenêtre (panneau du bas), pour choisir en connaissance de cause (vérification à l'œil).
- **Choix qui suivent la prise** : Faux départ, cartes de la main à cocher dans le panneau du bas comme le mode fin du tour
  (spec 016), bandeau `tactic.rethink_prompt`, bouton `tactic.rethink_confirm` (C9, C10) ; Vol de mana, dés de couleur de base
  de la Source cliquables, bandeau `tactic.steal_prompt` (C11) ; Préparation, fenêtre des cartes du deck (comme la fenêtre de la
  défausse, spec 073), bandeau `tactic.preparation_prompt` (C30, `PlayerView::preparation_choice`). Ces choix ne s'annulent pas :
  la tactique est prise.
- **Tactique du joueur dans le panneau de droite**, section du joueur : numéro, nom, texte en infobulle ; `tactic.flipped` quand
  elle est retournée ; le dé volé (`tactic.stolen_die`), le nombre de cartes sous Provisions ; ses boutons quand ils sont légaux :
  `tactic.right_moment_button` (rappel `tactic.right_moment_announced` ensuite), `tactic.long_night_button`,
  `tactic.mana_search_button` (dés de la Source cochables, bandeau `tactic.mana_search_prompt`, `tactic.mana_search_confirm`),
  `tactic.meditation_button` (cartes de la main cochables, `tactic.meditation_prompt`, `tactic.meditation_confirm`) (C12, C20,
  C23, C26, C28, C35, `PlayerView::tactic`).
- **Dé volé dans le choix du mana** (fenêtre du mana, spec 025) : une ligne de plus, « Dé volé : bleu » (C12).
- **Provisions** : au début du tour, fenêtre modale `tactic.sparing_prompt` avec `tactic.sparing_store` et `tactic.sparing_take`
  (le premier grisé, deck vide) (C31 à C33).
- **Longue Nuit pendant la pioche** : fenêtre `tactic.long_night_prompt` avec `tactic.long_night_button` et
  `tactic.long_night_decline` (C24).
- **Source** : le dé volé n'est plus dans la Source ; il apparaît sur la tactique (C11, `PlayerView::source`, `tactic`).
- **Écran de nouvelle partie** : fiche de Première reconnaissance, ligne « Pas encore dans le jeu » et règles spéciales modifiées
  (clés ci-dessus) (C1 pour la règle ; texte : vérification à l'œil).
- Pas d'illustration ni d'icône de tactique (Q6) : le grand numéro tient lieu de visuel. Placement et tailles : vérification à
  l'œil.

Parties préparées pour la vérification à l'œil (`docs/verification-manuelle.md`), sur Première reconnaissance (au fil écran :
`tactiques-jour`, `tactiques-manche-3` (trou de Planification, le choix ouvert), `tactiques-de-vole`, `tactiques-nuit`,
`tactiques-longue-nuit` ; un retour en arrière ne remonte pas avant le début du tour, d'où deux parties pour la manche 3) :

- `tactiques-jour` : manche 1, choix de la tactique en attente (fenêtre ouverte) ; et une manche 3 où Planification est retirée
  et Vol de mana pris, dé posé.
- `tactiques-nuit` : manche 2, Provisions prise avec 2 cartes dessous ; Longue Nuit, deck presque vide, au fil écran selon ce qui
  se montre le mieux.

### Raccourcis clavier

- **1 à 6** : choisir la tactique de ce numéro (l'encadrer) ; **Entrée** : la prendre. Contexte : fenêtre de choix de la
  tactique ouverte. Reporté dans `docs/raccourcis.md`.
- Les autres choix se font aux boutons ; Échap n'annule ni la prise ni Provisions, qui ne s'annulent pas. Les modes « cocher des
  cartes » (Méditation nocturne) et « cocher des dés » (Appel du mana), annulables, répondent à Échap comme les autres modes de
  choix.

## Effets sur les specs déjà implémentées

- **023 et 012** : après la mise en place et après `RoundStarted`, Première reconnaissance attend le choix de la tactique avant
  `TurnStarted` (le tour 1 commence au choix). Tous les tests et parcours qui jouent Première reconnaissance prennent une tactique
  d'abord ; relevé à la rédaction (`grep -rln "first_reconnaissance\|FirstReconnaissance" crates/*/tests`) :
  arcanum-core `premiere_reconnaissance.rs`, `competences.rs`, `parcours_j4.rs`, `parcours_j5.rs`, `nouvelle_partie.rs`,
  `mise_en_place_arrangee.rs`, `scenario_mise_en_place.rs`, `offres.rs`, `interaction.rs` ; arcanum-persistance
  `premiere_reconnaissance.rs`, `parties_preparees.rs` ; arcanum-ui `fin_de_manche.rs`, `premiere_reconnaissance.rs`, `source.rs`,
  `nouvelle_partie.rs`, `barre_de_menu.rs`, `panneaux.rs`, `scenario_affichage.rs`, `sauvegarde_ui.rs`, `fin_du_tour.rs`,
  `boucle_et_fenetre.rs`, `plateau_isometrique.rs`, `support/mod.rs`. Le fil moteur choisit l'aide commune (prendre Lève-tôt
  pour ne rien changer) plutôt que de modifier chaque test ; les assertions ne changent pas, et la PR le signale.
- **016 et 035** : la pioche de fin de tour gagne Planification et l'arrêt de Longue Nuit (`draw_limit` et la pioche de
  `end_turn`).
- **025** : `ManaPayment` gagne le dé volé (lecteurs : `grep -rn "ManaPayment::" crates`).
- **057** : l'ordre Distorsion temporelle puis Au bon moment (L7).
- **065 et 078** : textes de la fiche de Première reconnaissance.

## Hors périmètre

- Le joueur fantôme, sa tactique tirée au hasard parmi les restantes et l'**ordre du tour** (MKUE p. 4, 3.c ; p. 15 ; p. 19) :
  spec 061 (Q1). Au bon moment y prend son plein sens.
- Le choix par ordre de gloire (MKUE p. 4, 3.a) et les tactiques non prises mises de côté pour la manche (3.d) : plusieurs joueurs.
- Les usages hors de son tour (combat entre joueurs, assaut coopératif, Volkare : Compendium, *Tactic Cards* S1, *Sparing Power*
  S1) : sans objet en solo.
- Les Familiers magiques (paiement « au début de la manche », avant les tactiques, Compendium, *Silver Units* Q4) : unité de
  l'extension, absente de `units.ron`.
- Illustrations ou icônes de tactiques (Q6).

## Plan de développement

Spec moteur et écran à la fois : deux fils successifs sur la même branche (issue privée 118, Q7), plan détaillé
(`docs/specs/plans/060-tactiques.md`) au début du premier.

1. **Fil moteur** : `Tactic`, `TacticRule` (avec `TacticOrder` et `TacticRemoval`), choix et retrait, les douze effets, paiement par le dé volé, vue, persistance,
  tests existants adaptés : C1 à C40, L1 à L9.
2. **Fil écran** : libellés, fenêtre de choix, panneau de droite, choix et fenêtres de Provisions et de Longue Nuit, fiche du
  scénario, raccourcis, parties préparées, vérification à l'œil, docs (`architecture.md`, `verification-manuelle.md`,
  `index-par-notion.md`, `glossaire.md`, `raccourcis.md`).

## Décisions (2026-11-18)

Mathieu a accepté les sept propositions (« ok pour les propositions ») :

- **Q1** — Ordre du tour et tactique tirée par le joueur fantôme : spec 061 ; la 060 garde le numéro de la tactique.
- **Q2** — Tactiques pour Première reconnaissance seulement ; le bac à sable n'en a pas.
- **Q3** — `ChooseTactic`, puis une décision à part pour Faux départ, Vol de mana et Préparation ; ni la prise ni ces choix ne
  s'annulent.
- **Q4** — Cartes sous Provisions inconnues de tous ; la vue n'en donne que le nombre.
- **Q5** — Longue Nuit en pleine pioche de fin de tour : la pioche s'arrête et demande.
- **Q6** — Aucune image de tactique ; le grand numéro suffit.
- **Q7** — Deux fils : moteur, puis écran.

Remarque de Mathieu (affichage) : grille de 2 lignes et 3 colonnes, une place fixe par numéro, un trou pour une tactique
retirée ; plus tard, une tactique prise cette manche par un autre joueur grisée avec son icône (Interface).

Remarque de Mathieu : hors du solo, les tactiques reviennent aux manches suivantes ; le retrait est donc un paramètre du scénario.
Seul le retrait solo est codé, avec une structure prête pour les autres (Modèle, `TacticRule` ; Règles, *Tactiques retirées*).
Q6 n'ouvre pas d'issue maintenant : elle ne viendra que si l'écran paraît pauvre à la vérification à l'œil du fil écran.

## Questions ouvertes

Aucune.

<details><summary>Questions posées à la relecture</summary>

- **Q1 — Ordre du tour et tactique du joueur fantôme : spec 061 ?** Le plan met « ordre du tour » dans la 060, mais seul le joueur
  fantôme donne un second rang. Proposition : la 060 garde le numéro de la tactique dans l'état et la vue ; la 061 ajoute la
  tactique tirée au hasard par le fantôme parmi les restantes (MKUE p. 19), son retrait en fin de manche, et l'ordre du tour.
  La ligne 060 de `PLAN.md` dit alors « ordre du tour : spec 061 ».
- **Q2 — Quels scénarios ?** Proposition : Première reconnaissance (sa variante solo l'exige) et pas le bac à sable, qui garde
  ses tests et ses parties préparées. Les parties préparées de cette spec partent de Première reconnaissance.
- **Q3 — Choix « en la prenant » : une action ou deux ?** Proposition : `ChooseTactic` seul, puis une décision à part pour les
  cartes de Faux départ, le dé de Vol de mana et la carte de Préparation (le fil moteur peut les fondre si les actions légales
  restent peu nombreuses). À l'écran, la prise et ces choix ne s'annulent pas : `TurnStarted` suit dans la même décision, et une
  pioche révèle des cartes.
- **Q4 — Cartes sous Provisions : connues du joueur ?** La carte dit « face cachée » et le joueur ne regarde pas la carte du
  dessus de son deck. Proposition : inconnues de tous, la vue n'en donne que le nombre jusqu'au retournement.
- **Q5 — Longue Nuit au milieu de la pioche.** Le livret le permet (p. 11, 7.d, note). Proposition : la pioche de fin de tour
  s'arrête quand le deck s'épuise avant la limite, si Longue Nuit est utilisable, et demande (fenêtre à deux boutons). Plus
  simple mais moins fidèle : ne l'offrir que pendant le tour, ce qui oblige à l'utiliser avant de finir le tour.
- **Q6 — Visuel des tactiques.** La spec 072 prévoyait des icônes SVG « avec leur spec ». Proposition : aucune image pour
  l'instant, le grand numéro sur la carte suffit ; une issue `planifié` pour des illustrations plus tard si l'écran paraît pauvre
  (comme les sorts, spec 087).
- **Q7 — Deux fils.** Proposition : un fil moteur (les douze tactiques, environ 40 cas) puis un fil écran, comme la spec 055.
  Alternative : trois fils (moteur « au choix » et Planification, moteur « pendant la manche », écran), comme la 058.

</details>
