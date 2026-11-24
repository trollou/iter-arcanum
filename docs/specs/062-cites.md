# 062 — Cités

- Statut : implémentée (le 2026-11-19 ; validée le même jour, propositions de Q1 à Q6 retenues par Mathieu) (reporter tout changement dans la colonne Statut de `PLAN.md`)
- Jalon : J6
- Dépend de : 043 (sites fortifiés : défenseurs, assaut, retraite, Boucliers, achat de sort), 015 (exploration), 030 et 033
  (jetons d'ennemis, provocation, piles de secours, capacités), 031 et 032 (combat, éléments), 040 (interaction, réputation),
  041 (achats, récompense artefact), 042 et 050 (unités, offre d'unités, élites), 052 (cases sécurisées), 053 (Bouclier de glace,
  réduction d'armure), 054 et 057 (Exposition, Démolition, Attaque souterraine), 063 (Première reconnaissance : cités fermées) ;
  toutes implémentées
- Livret : MKUE p. 7 (3.a, note : une cité coûte 2 ; 4.a ; 5.g : cité révélée) ; p. 8 (*Interacting with Locals* 1, 3.a, 4.e,
  5.a ; *Combat with Enemies* 1.a) ; p. 10 (*Combat Outcomes* 1.a, 2.c, 3.d ; *City Assaults* 1 et 2) ; p. 11 (*End of the
  Turn* 7.b) ; p. 4 (2.g) ; p. 15 (*The Greatest Conqueror*) ; p. 18 (*City Levels*). Carte de site City et cartes de cité :
  `docs/regles/cartes/sites-et-cites.md` ; termes français : carte de site FR (`docs/regles/scans/fr/rule_cards/`). FAQ p2
  (*Cités* ; *Endurance givrée* ; *Tremblement de terre* et *Démolition*). Compendium : *Cities* S1 à S8, S19 à S23 ;
  *Movement* S4, S8 ; *End of Turn* S5, S8 ; *Influence* S1, S7, S10 ; *Forced Withdrawal* S2, S4 ; *Final Scoring* S2
- Crates touchées : arcanum-core (contenu, règles, vue), arcanum-persistance (tests, parties préparées), arcanum-i18n (libellés), arcanum-ui

## Objectif

Les cités cessent d'être des cases fermées. Révélée, une cité reçoit ses **défenseurs** selon son **niveau** et sa couleur, face
cachée, retournés dès que le héros passe à côté, de jour comme de nuit. Y entrer est un **assaut** contre tous les défenseurs
restants, fortifiés et renforcés par le **bonus de la cité** (armure, attaque, Brutal ou Toxique selon la couleur). Chaque
défenseur vaincu pose un **Bouclier** sur la cité ; un assaut repoussé garde les Boucliers et les défenseurs vaincus ne
reviennent pas. Le dernier défenseur vaincu, la cité est **conquise** et le joueur en devient le **Seigneur**. Une cité conquise
est une case ordinaire où l'on **interagit** avec un bonus d'Influence par Bouclier : recruter les unités à icône Cité, et un
achat propre à sa couleur (artefact, sort, action avancée, unités de tous types). Finir son tour sur elle ou à côté augmente la
limite de main.

Le scénario dit le niveau de chaque cité, dans l'ordre de révélation : le bac à sable en a une, Première reconnaissance garde ses
cités fermées (MKUE p. 18). Le scénario Conquête en solitaire et son décompte des cités viennent avec la spec 064.

### L'assaut d'une cité, en schéma

```
Révélation de la tuile ── CityPlaced (couleur, niveau) ── DefenderPlaced × n (face cachée)
        │
Pas à côté de la cité (jour ou nuit) ── EnemyTokenRevealed × n           — point de non-retour
        │
Pas sur la cité ── AssaultStarted ── Réputation −1   (moment « à engager » : défier un errant voisin, ou engager)
        │
Combat : tous les défenseurs restants, fortifiés, avec le bonus de la cité ; les errants provoqués ou défiés, sans rien
        │
Fin du combat ── un Bouclier par défenseur vaincu (CityShieldPlaced), dans l'ordre
        ├── plus aucun défenseur ── CityConquered (Seigneur) : la figurine reste
        └── sinon ── HeroWithdrew : retour sur la case de départ ; défenseurs restants et Boucliers restent
```

## Règles

### Niveau et défenseurs

- **Niveau de la cité** (MKUE p. 7, 5.g ; p. 18, *City Levels*) : donné par le scénario, selon l'ordre de révélation (Conquête en
  solitaire : la première révélée au niveau 5, la seconde au niveau 8, MKUE p. 19). Le scénario porte la **liste** des niveaux ;
  la n-ième cité révélée prend le n-ième niveau, et une cité au-delà de la liste prend le dernier. Bac à sable : une cité au
  **niveau 5** (Q1). Première reconnaissance : **cités fermées** (MKUE p. 18, *Cities cannot be conquered nor entered*) : ni
  niveau, ni défenseurs, et y entrer reste refusé comme aujourd'hui (Q2).
- **Défenseurs selon le niveau** (MKUE p. 7, 5.g ; carte City, *When Revealed*) : la base de la figurine montre des cercles de
  couleur ; un jeton d'ennemi de cette couleur par cercle, tiré comme les autres (remélange de la défausse, pile de secours,
  specs 031 et 033), posé **face cachée** sur la cité. Le tableau des cercles de chaque couleur de cité, niveaux 1 à 11, est lu
  sur les scans des cartes de cité (`docs/regles/cartes/sites-et-cites.md`, *Défenseurs par niveau*) et devient du **contenu**
  (`cities.ron`, ADR 0005). Exemple : cité blanche niveau 5 = un gris et deux blancs ; cité bleue niveau 5 = deux violets et un
  blanc.
- **Retournés à côté du héros, même de nuit** (carte City : *revealed when a player is adjacent to the city (even at Night)* ;
  Compendium, *Movement* S8) : comme les défenseurs de la spec 043 (un état « est voisine » : après un pas, après la révélation
  d'une tuile), mais **de nuit aussi**. Une cité n'est jamais voisine de la case d'où l'on explore sa tuile (elle est au centre) :
  en pratique, c'est un pas qui les retourne. **Un trajet s'arrête** sur le pas qui retourne les défenseurs de la cité visée (spec
  043, Compendium *Movement* S13), de jour comme de nuit.
- **Toujours les mêmes** (Compendium, *Cities* S23 ; FAQ p2, *Cités*) : un défenseur vaincu quitte la cité pour la défausse de
  sa couleur et n'est jamais remplacé ; les défenseurs non vaincus restent, face visible.
- **Ce ne sont pas des ennemis errants** (spec 043) : ni provoqués, ni défiés, ni évités par les trajets.

### Assaut

- **Entrer, c'est assaillir** (MKUE p. 7, 4.a ; note de 3.a ; Compendium, *Movement* S4 ; FAQ p2) : un pas vers une cité **non
  conquise** coûte **2** quel que soit le terrain, finit le déplacement et commence l'action du tour, comme l'assaut d'une
  forteresse (spec 043) : mouvement et influence non dépensés perdus, **Réputation −1** à chaque assaut, quelle que soit l'issue,
  y compris sur la case X. Aussi au bout d'un trajet (spec 036) ; un trajet ne traverse jamais une cité non conquise.
- **Par Attaque souterraine** (Voyage souterrain renforcé : « Idem, mais finir sur un site fortifié : assaut en ignorant ses
  fortifications » ; Compendium, *Cities* S10, *Underground Attack* S1) : une cité non conquise est une arrivée possible, comme
  une forteresse (spec 057). Des défenseurs encore face cachée sont retournés à l'engagement (spec 043) ; la retraite ramène à la
  case de départ du sort.
- **Tous les défenseurs restants** (Compendium, *Cities* S1 ; FAQ p2 : « vous initiez le combat avec tous les ennemis ») plus les
  ennemis errants provoqués par le pas ou défiés depuis la cité (moment « à engager » des specs 030 et 031). Le combat à
  plusieurs ennemis est celui de la spec 031 : on n'est pas tenu de les parer ni de les attaquer tous.
- **Fortifiés** (MKUE p. 10, *City Assaults* 1 ; spec 043) : chaque défenseur est fortifié par la cité ; un défenseur qui a aussi
  la capacité Fortifié (Gardiens, Glaceurs, Gardiens d'Altem, Ver des cryptes) est **doublement fortifié**. Les règles de cible de
  la spec 043 valent : un groupe qui contient un défenseur doublement fortifié n'est pas une cible en phase d'attaque à distance,
  les autres défenseurs se visent au siège, seuls ou ensemble (Compendium, *Cities* Q2). Exposition, Démolition et Attaque
  souterraine retirent la fortification de la cité comme celle d'un autre site (specs 054 et 057). Les ennemis errants ne sont
  pas fortifiés et n'ont pas le bonus.
- **Bonus de la cité** (MKUE p. 10, *City Assaults* 1.a à 1.d ; face claire des cartes de cité), sur chaque défenseur, jusqu'à la
  fin du combat :
  - **Blanche** : **Armure +1**, appliquée **avant** toute réduction (Compendium, *Cities* S7 : Bouclier de glace renforcé sur
    des Gardiens donne 8 − 3 = 5).
  - **Bleue** : **attaque +2** si elle est de glace ou de feu, **+1** si elle est de feu glacé (Compendium, *Cities* S8). L'attaque
    augmentée est celle que l'on pare (Vif la double) et celle des dégâts (Brutal la double).
  - **Rouge** : un défenseur à attaque **physique** gagne **Brutal** ; déjà brutal, rien de plus (Compendium, *Cities* S5 et S6).
  - **Verte** : un défenseur à attaque **physique** gagne **Toxique** ; déjà toxique, rien de plus (mêmes renvois).
  - **Créature invoquée** (Compendium, *Cities* S4 ; FAQ p2 : « dans la Cité Rouge, … ce monstre devient Brutal ») : le monstre
    invoqué par un défenseur Invocateur (Illusionnistes) a le bonus de la cité et sa fortification.
  - **Capacités gagnées** : elles ne comptent pas pour les effets qui comptent les capacités **imprimées** de l'ennemi
    (FAQ p2 : Endurance givrée, « Parade de glace 5 ; +1 par capacité de l'ennemi paré », ne compte ni la fortification du site
    ni le Toxique de la cité verte).
- **Gloire entière** de chaque défenseur vaincu (Compendium, *Cities* S23, *Combat* S36), assaut réussi ou non. Aucune réputation
  pour un défenseur (MKUE p. 10, 2.a ne vaut que pour les orques et les draconiens errants).

### Issue

- **Un Bouclier par défenseur vaincu** (MKUE p. 10, *Combat Outcomes* 1.a ; FAQ p2 ; carte City) : à la fin du combat, dans
  l'ordre où ils ont été vaincus, que la cité tombe ou non. Les Boucliers sont posés **sur la cité**, en rangée : l'ordre sert à
  désigner le Seigneur en cas d'égalité.
- **Conquête** (MKUE p. 10, 2.c et *City Assaults* 2 ; Compendium, *Cities* S3) : plus aucun défenseur dans la cité à la fin du
  combat, elle est **conquise**. Le **Seigneur de la cité** est le joueur qui a le plus de Boucliers dessus (égalité : celui du
  premier Bouclier posé) ; en solo, c'est toujours le joueur. La figurine reste sur la cité. Les ennemis errants non vaincus
  restent où ils sont et ne changent rien à la conquête. Pas de récompense de fin de tour (Compendium, *Cities* S22) : la gloire
  des défenseurs, les Boucliers et ce que la cité conquise offre ensuite.
- **Retraite** (MKUE p. 10, 3.d ; Compendium, *Cities* S23, *Forced Withdrawal* S2) : il reste des défenseurs, la figurine
  revient sur la case d'où elle a assailli, sans coût ni Blessure, comme pour une forteresse (spec 043). Les Boucliers posés
  restent ; un nouvel assaut ne combat que les défenseurs restants.
- **Ni déplacement ni interaction** après l'assaut ce tour, conquête comprise (Compendium, *Cities* S21, *Influence* S7).

### Cité conquise

- **Case ordinaire** (carte City, *While Conquered* ; MKUE p. 18, *Megapolis* : « standard city Move cost of 2 ») : y entrer coûte
  2, on la traverse, on y finit son tour : c'est une case sécurisée (spec 052). Plus d'assaut.
- **Interaction** (MKUE p. 8, 1 ; carte City ; Compendium, *Cities* S19 et S21) : l'action `Interact` est permise dans une cité
  **conquise**, par n'importe quel joueur, même sans Bouclier dessus ; jamais dans une cité non conquise, même avec des Boucliers.
  - **Bonus d'Influence** (MKUE p. 8, 3.a et 5.a ; Compendium, *Influence* S1, S10) : +1 par Bouclier du joueur sur la cité,
    ajouté une fois par tour, au début de l'interaction, comme le bonus de réputation (et même sur la case X ? non : la case X
    interdit l'interaction, spec 040).
  - **Recruter** (MKUE p. 8, 4.a ; carte City) : les unités dont un lieu de recrutement est la cité (Canonniers d'Amotep,
    Glaceurs d'Amotep, Catapultes, Mages d'Altem, Gardiens d'Altem ; `units.ron`, `recruit: [.., City]`) dans toute cité
    conquise ; **dans la cité blanche, toutes les unités** de l'offre, quel que soit leur lieu (MKUE p. 8, 4.e).
  - **Selon la couleur** (MKUE p. 8, 4.e ; Compendium, *Cities* S21), plusieurs achats permis (MKUE p. 8, 5) :
    - **Rouge** : un **artefact** pour **12** Influence. Il est tiré à la fin du tour comme une récompense de combat (MKUE p. 11,
      5.b ; Compendium, *End of Turn* S5) : un achat de plus à la récompense artefact du tour (spec 041 : tirer un de plus,
      en remettre un sous la pioche).
    - **Bleue** : un **sort** de l'offre pour **7** Influence et un mana de sa couleur, comme à la tour de mage (spec 043,
      `BuySpell`).
    - **Verte** : une **action avancée** pour **6** Influence, soit de l'**offre d'actions avancées** (complétée aussitôt), soit
      la carte du **dessus de la pioche** d'actions avancées, sans la voir avant de payer ; sur le dessus du deck. Pas une action
      avancée de l'offre d'unités (réservée au monastère, Compendium, *Cities* S21).
    - **Blanche** : pour **2** Influence, la carte du dessus de la pioche d'**unités d'élite** est ajoutée à l'offre d'unités
      (MKUE p. 8, 4.e), en plus du recrutement de toutes les unités.
  - **Pas de soin** dans une cité (Compendium, *Cities* S21) : rien n'est imprimé pour cela.
- **Limite de main** (carte City, *Bonus* ; MKUE p. 11, 7.b ; p. 4, 2.g ; Compendium, *End of Turn* S8, *Cities* S20) : figurine
  sur une cité **conquise** ou à côté : **+1** si le joueur a au moins un Bouclier dessus, **+2** s'il en est le Seigneur. Avec
  une forteresse possédée aussi à portée, seul le **plus grand** des deux bonus compte. Pour la pioche de fin de tour et pour
  celle de la nouvelle manche ; le seuil d'**assommé** reste la limite non modifiée (spec 043). Une cité non conquise ne donne
  rien, même avec des Boucliers.
- **Le Conquérant** (MKUE p. 15 : *Shield token … on a keep, mage tower or monastery*) : les Boucliers d'une cité **ne comptent
  pas** ; les cités ont leur propre décompte dans les scénarios de conquête (spec 064 ; Compendium, *Final Scoring* S2). La carte de
  score montre bien trois icônes, monastère, forteresse et tour de mage (scan `scoring.jpg` relu) : notre transcription, qui
  lisait « cité » pour le monastère, est corrigée dans cette PR.

### Exemple : assaut de la cité bleue

Bac à sable, nuit. La tuile royaume 6 (cité bleue, `tiles.ron`) a été révélée : cité bleue niveau 5, deux jetons violets et un
blanc face cachée. La figurine arrive sur une case voisine : de nuit aussi, les trois jetons sont retournés : **Mages de glace**
(armure 6, attaque de glace 5, résistance à la glace, gloire 5), **Mages de feu** (armure 5, attaque de feu 6, résistance au feu,
gloire 5), **Glaceurs** (armure 7, attaque de glace 3, Fortifié, Vif, Paralysant, résistance au feu, gloire 7). Réputation 0.

```
Pas voisin de la cité          → HeroMoved ; EnemyTokenRevealed × 3       — point de non-retour
Pas sur la cité (coût 2)       → HeroMoved ; AssaultStarted ; ReputationChanged −1
Engager                        → CombatStarted : trois défenseurs, fortifiés ; Glaceurs doublement fortifiés
  attaques : Mages de glace 7 (5 + 2), Mages de feu 8 (6 + 2), Glaceurs 5 (3 + 2), vifs : 10 à parer
  à distance : siège sur les Mages de feu (armure 5) → vaincus
  parade : Mages de glace parés ; Glaceurs non parés : dégâts de glace 5, paralysant
  attaque : rien sur les Mages de glace (armure 6) ni sur les Glaceurs
Fin du combat                  → CombatEnded ; CityShieldPlaced (Mages de feu) ; HeroWithdrew : retour sur la case voisine
Plus tard, nouvel assaut       → Réputation −1 ; combat contre les Mages de glace et les Glaceurs seulement
  … les deux vaincus           → CityShieldPlaced × 2 ; CityConquered (Seigneur : le joueur, 3 Boucliers)
Fin du tour sur la cité        → limite de main + 2 (Seigneur)
```

## Modèle

Termes du glossaire : Cité, Site fortifié, Assaut, Défenseur, Doublement fortifié, Conquérir, Jeton Bouclier, Battre en retraite,
Case sécurisée, Révéler, Ennemi errant, Interaction, Influence, Réputation, Lieux de recrutement, Offre d'unités, Unité d'élite,
Offre d'actions avancées, Offre de sorts, Artefact, Récompense, Limite de main, Le Conquérant. **Nouveaux termes** (ajoutés au
glossaire par cette PR ; français de la carte de site FR) :

| Livret (EN) | Français | Code |
|---|---|---|
| City level | Niveau de la cité | `CityState::level` |
| City card (bonus, interaction) | Carte de cité ; bonus de la cité | `CityState::color` |
| City leader | Seigneur de la cité (carte FR) | `CityState::leader` |
| City figure (base circles) | Cercles de la figurine ; défenseurs de la cité | `cities.ron` |
| Megapolis | Mégapole | — (hors périmètre) |

Données nouvelles :

- **Contenu** : `cities.ron`, pour chaque couleur de cité, la liste des couleurs de jetons de chaque niveau, 1 à 11 (tableau de
  `docs/regles/cartes/sites-et-cites.md`). `GameContent::parse` refuse un niveau manquant ou en trop, une couleur de cité
  manquante, et une couleur de jeton qui n'est pas grise, violette, marron ou blanche.
- **Contenu, coûts** : 12 (artefact), 6 (action avancée), 2 (unité d'élite) en constantes du moteur, comme les achats des specs
  041 et 043 ; le sort reprend le coût de la tour de mage.
- **Scénario** : `cities: CityRule`, `Closed` (Première reconnaissance, et une sauvegarde d'avant cette spec : valeur par défaut
  serde) ou `Levels(Vec<u8>)` (bac à sable : `[5]`), niveaux de 1 à 11, liste non vide (refus typé à la création sinon).
- **Partie** : pour chaque cité révélée, `CityState` (nom au fil d'implémentation) : couleur, niveau, Boucliers dans l'ordre
  (joueur de chacun), Seigneur une fois conquise. Les défenseurs restent dans `enemy_tokens` et `face_down` (spec 043). La cité
  conquise compte comme un site conquis partout où la spec 043 regarde `shields` (case sécurisée, coût de déplacement, plus
  d'assaut) ; la représentation exacte est au choix du fil d'implémentation.
- **Interaction** : l'interaction en cours retient le bonus des Boucliers (`InteractionStarted` le porte, comme le bonus de
  réputation).
- **Combat** : un défenseur de cité porte la couleur de sa cité (`CombatEnemy`, champ au fil d'implémentation) ; ses valeurs
  effectives (armure, attaque, capacités) en tiennent compte, ainsi que la créature qu'il invoque. Les capacités **imprimées**
  restent distinctes des capacités effectives (Endurance givrée).
- **Récompenses** : l'achat d'artefact augmente la récompense artefact du tour (`artifact_rewards`, spec 041).

## Actions et événements

| Action | Validation (refusée si…) | Événements produits |
|---|---|---|
| `Explore` (015, étendue) | inchangée | … `TileRevealed`, jetons errants, défenseurs des forteresses et tours (043), puis, si la tuile porte une cité et que le scénario a des niveaux : `CityPlaced { hex, color, level }` et un `DefenderPlaced` par cercle, dans l'ordre des cercles ; puis les `EnemyTokenRevealed` des défenseurs voisins de la figurine (aucun en pratique) |
| `MoveHero { to }`, `FollowRoute` (014, 036, 043, étendues) | cité non conquise : refusée si le scénario a les cités fermées, sinon permise au coût 2 ; cité conquise : case ordinaire, coût 2 | vers une cité non conquise : `HeroMoved`, `AssaultStarted`, `ReputationChanged { −1 }`, `EnemiesProvoked` au besoin ; après chaque pas, de jour **ou de nuit**, un `EnemyTokenRevealed` par défenseur face cachée d'une cité voisine ; un pas qui retourne les défenseurs de la cité visée arrête le trajet |
| `PlayCard` Attaque souterraine (057, étendue) | une cité non conquise est une arrivée possible, sauf cités fermées | comme pour une forteresse |
| `StartCombat` (031, 043, étendue) | inchangée | défenseurs face cachée : `EnemyTokenRevealed` chacun ; `CombatStarted` avec **tous** les défenseurs restants de la cité et les ennemis errants |
| `EndCombat` (031, 043, étendue) | inchangée | `CombatEnded`, puis pour un assaut de cité : un `CityShieldPlaced { player, hex, enemy }` par défenseur vaincu, dans l'ordre ; puis `CityConquered { hex, leader }` s'il n'en reste aucun, sinon `HeroWithdrew` |
| `Interact` (040, étendue) | permise dans une cité conquise | `InteractionStarted { site, bonus, city_shields }` (le nombre de Boucliers du joueur sur cette cité, 0 ailleurs) |
| `RecruitUnit` (042, étendue) | dans une cité : refusée si l'unité n'a pas l'icône Cité, sauf dans la cité blanche | inchangés |
| `BuySpell` (043, étendue) | permise aussi dans la cité bleue | inchangés |
| `BuyAdvancedAction { card }` (041, étendue) | dans la cité verte : `card` est dans l'offre d'actions avancées, 6 Influence | `AdvancedActionBought` (existant ; complète l'offre d'actions avancées) |
| `BuyAdvancedActionFromDeck` (nouvelle) | pas dans la cité verte, moins de 6 Influence, pioche d'actions avancées vide | `AdvancedActionDrawnBlind { card }` : la carte du dessus va sur le dessus du deck |
| `BuyArtifact` (nouvelle) | pas dans la cité rouge, moins de 12 Influence, pioche d'artefacts trop courte (L7) | `ArtifactBought` : une récompense artefact de plus pour la fin du tour |
| `AddEliteUnit` (nouvelle) | pas dans la cité blanche, moins de 2 Influence, pioche d'unités d'élite vide | `EliteUnitAdded { card }` : la carte du dessus va dans l'offre d'unités |
| `BuyHealing` (041) | refusée dans une cité | — |

| Événement | Effet de `apply` | Révèle une information ? | Point de non-retour ? |
|---|---|---|---|
| `CityPlaced { hex, color, level }` | la cité entre en jeu, sans Bouclier | non (suit `TileRevealed`) | oui (avec la tuile) |
| `DefenderPlaced` (043) | inchangé : un défenseur face cachée de plus sur la case | couleur seulement | oui (avec la tuile) |
| `EnemyTokenRevealed` (043) | inchangé | oui | oui |
| `CityShieldPlaced { player, hex, enemy }` | un Bouclier de plus sur la cité, au bout de la rangée ; le défenseur `enemy` a déjà quitté la case (vaincu) | non | non |
| `CityConquered { hex, leader }` | la cité est conquise, son Seigneur | non | non |
| `InteractionStarted` (040, étendu) | retient aussi le bonus des Boucliers, ajouté à l'Influence | non | non |
| `AdvancedActionDrawnBlind { card }` | la carte du dessus de la pioche d'actions avancées passe sur le dessus du deck ; −6 Influence | oui (carte cachée) | oui |
| `ArtifactBought` | −12 Influence ; récompense artefact +1 | non | non |
| `EliteUnitAdded { card }` | la carte du dessus des élites rejoint l'offre d'unités ; −2 Influence | oui (carte cachée) | oui |

Noms indicatifs : le fil d'implémentation peut les ajuster en le signalant.

## Comportements attendus

Sauf mention, la partie est le **bac à sable** avec Arythea, en mise en place arrangée (ADR 0006 : l'arrangement choisit la tuile
royaume de cité et l'ordre des piles de jetons), ou en état posé (tuile de cité posée et défenseurs écrits par le test) quand le
cas ne porte pas sur la révélation. Valeurs des ennemis lues dans `enemies.ron`, unités dans `units.ron`, noms français dans
`assets/i18n/fr.json` (relus au fil écran).

### Contenu et scénario

- **C1** — `cities.ron` donne, pour chaque couleur et chaque niveau 1 à 11, les couleurs du tableau des cartes de cité (cité
  blanche niveau 5 : gris, blanc, blanc ; cité bleue niveau 5 : violet, violet, blanc ; cité rouge niveau 11 : marron, violet,
  blanc, blanc, blanc) (`city_garrisons_follow_city_cards`) — cartes de cité.
- **C2** — `parse` refuse un niveau manquant, un douzième niveau, une couleur de cité absente, un jeton vert ou rouge
  (`city_content_rejects_incoherent_levels`).
- **C3** — Scénarios : bac à sable `Levels([5])`, Première reconnaissance `Closed` ; une liste vide ou un niveau 0 ou 12 est
  refusé à la création (`scenario_city_levels`) — MKUE p. 18.

### Révélation

- **C4** — Bac à sable : révéler la tuile royaume 6 produit `TileRevealed`, puis `CityPlaced { couleur bleue, niveau 5 }` et
  trois `DefenderPlaced` face cachée, deux violets puis un blanc, tirés des piles (`revealed_city_gets_its_defenders`) — MKUE
  p. 7, 5.g.
- **C5** — Niveau selon l'ordre : un scénario de test `Levels([5, 8])` avec deux tuiles de cité donne le niveau 5 à la première
  révélée, 8 à la seconde ; `Levels([5])` donne 5 aux deux (`city_level_follows_reveal_order`) — MKUE p. 19, *Cities*.
- **C6** — Première reconnaissance : révéler la cité ne produit ni `CityPlaced` ni défenseur ; y entrer reste refusé
  (`first_reconnaissance_cities_stay_closed` ; reprend le cas cité de `entering_a_fortified_site_is_an_assault_except_cities`,
  `deplacement.rs`) — MKUE p. 18.
- **C7** — De nuit, un pas sur une case voisine de la cité retourne ses défenseurs face cachée (un `EnemyTokenRevealed` chacun) ;
  de jour aussi (`city_defenders_revealed_next_to_it_even_at_night`) — carte City ; Compendium, *Movement* S8.
- **C8** — Un trajet vers la cité, de nuit, s'arrête sur le pas qui retourne ses défenseurs, à côté d'elle, sans assaut
  (`route_to_city_stops_when_defenders_are_revealed`) — spec 043 ; Compendium, *Movement* S13.

### Assaut

- **C9** — Pas sur la cité bleue (terrain plaine) ou sur la cité verte (marais) : coût 2 ; `AssaultStarted`,
  `ReputationChanged −1`, mouvement et influence restants perdus (`entering_a_city_is_an_assault_costing_two`) — MKUE p. 7, 3.a
  note, 4.a ; Compendium, *Movement* S4.
- **C10** — Engager : `CombatStarted` avec les trois défenseurs et un ennemi errant provoqué par le pas ; les défenseurs sont
  fortifiés par le site, l'errant non (`city_assault_fights_all_defenders`) — Compendium, *Cities* S1 ; FAQ p2.
- **C11** — Cité blanche niveau 5 (Gardiens : armure 7, Fortifié) : en phase d'attaque à distance, les Gardiens, doublement
  fortifiés, ne sont pas une cible, seuls ou avec un autre défenseur ; un Canonnier (armure 6, attaque de feu 6, Brutal) seul est
  une cible de siège (`city_defenders_are_fortified`) — MKUE p. 9, 3.a ; Compendium, *Cities* Q2.
- **C12** — Cité blanche : Gardiens à armure 8 ; Bouclier de glace renforcé (« Parade de glace 3 ; l'ennemi paré perd 3
  d'Armure (min. 1) », payé d'un mana bleu en phase de parade) sur eux : armure 5 (`white_city_adds_armor_before_reductions`) —
  MKUE p. 10, *City Assaults* 1.a ; Compendium, *Cities* S7.
- **C13** — Cité bleue : Mages de glace attaque de glace 7, Mages de feu attaque de feu 8, Mages d'Altem attaque de feu glacé 5
  (4 + 1), Gardiens (physique 3) inchangés ; Glaceurs (glace 3, Vif) : 10 à parer en Parade de feu (`blue_city_raises_ice_and_fire_attacks`)
  — MKUE p. 10, 1.b ; Compendium, *Cities* S8.
- **C14** — Cité rouge : Minotaure (physique 5, Brutal) reste brutal une seule fois ; Gargouille (physique 5) devient brutale :
  10 dégâts non parée ; Mages de feu (attaque de feu) ne le deviennent pas (`red_city_makes_physical_attackers_brutal`) — MKUE
  p. 10, 1.c ; Compendium, *Cities* S5, S6.
- **C15** — Cité verte : Gardiens (physique 3) et Ver des cryptes (physique 6) deviennent toxiques ; Canonniers (attaque de feu)
  et Glaceurs (attaque de glace) non ; aucun ennemi déjà Toxique ne peut défendre une cité verte (gris, marron, blanc : le Toxique
  imprimé des Mages d'Altem va avec une attaque de feu glacé) (`green_city_makes_physical_attackers_poisonous`) — MKUE p. 10, 1.d.
- **C16** — Cité rouge niveau 2 (marron, violet) avec des Illusionnistes : le monstre invoqué (un Minotaure ou une Gargouille, en
  tête de la pile marron) est brutal et compte comme fortifié (`summoned_monster_gets_city_bonus`) — Compendium, *Cities* S4 ;
  FAQ p2.
- **C17** — Cité verte, Endurance givrée renforcée de Tovak (« Parade de glace 5, +1 par capacité de l'ennemi paré ») sur des
  Gardiens : 5 + 1 (Fortifié imprimé), pas + 2 (ni la fortification du site ni le Toxique de la cité)
  (`cold_toughness_ignores_city_abilities`) — FAQ p2.
- **C18** — Attaque souterraine depuis trois cases : arrivée sur la cité bleue non conquise, assaut ; défenseurs face cachée
  retournés à l'engagement ; fortifications ignorées ; retraite sur la case de départ du sort (`underground_attack_assaults_a_city`)
  — carte ; Compendium, *Cities* S10, *Forced Withdrawal* S4.

### Issue

- **C19** — Assaut de la cité bleue : seuls les Mages de feu sont vaincus : gloire 5, `CityShieldPlaced` (Mages de feu),
  `HeroWithdrew` vers la case d'où l'on a assailli, aucune Blessure ; les Mages de glace et les Glaceurs restent, face visible
  (`failed_city_assault_keeps_shields`) — MKUE p. 10, 1.a, 3.d ; Compendium, *Cities* S23.
- **C20** — Nouvel assaut après C19 : Réputation −1 de nouveau ; le combat n'a que les deux défenseurs restants
  (`second_assault_faces_remaining_defenders`) — Compendium, *Cities* S23 ; FAQ p2.
- **C21** — Tous les défenseurs vaincus : un `CityShieldPlaced` par défenseur dans l'ordre où ils l'ont été, puis
  `CityConquered { leader: le joueur }` ; la figurine reste sur la cité ; un errant provoqué non vaincu reste sur sa case
  (`city_conquered_when_all_defenders_are_defeated`) — MKUE p. 10, 2.c, *City Assaults* 2 ; Compendium, *Cities* S3.
- **C22** — Le tour de la conquête : ni `Interact`, ni pas (`no_interaction_after_conquering_a_city`) — Compendium, *Cities* S21.

### Cité conquise

- **C23** — Cité conquise : un pas dessus coûte 2, sans assaut ; un trajet la traverse ; y finir son tour n'est pas une retraite
  forcée (`conquered_city_is_an_ordinary_space`) — carte City ; MKUE p. 18.
- **C24** — Interaction dans la cité bleue conquise avec 3 Boucliers, réputation +1 (bonus +1) : Promesse (Influence 2) donne
  2 + 1 + 3 = 6 Influence ; acheter deux choses n'ajoute le bonus qu'une fois (`city_interaction_adds_shield_bonus`) — MKUE p. 8,
  3.a, 5.a.
- **C25** — Cité conquise sans Bouclier du joueur (état posé : Boucliers d'un autre joueur) : interaction permise, bonus 0 ; cité
  non conquise avec des Boucliers du joueur : interaction refusée (`city_interaction_needs_a_conquered_city`) — Compendium,
  *Cities* S19.
- **C26** — Recrutement : dans la cité bleue conquise, les Catapultes (élite, Forteresse et Cité) oui, les Bûcherons (Village)
  non ; dans la cité blanche, les deux (`city_recruits_city_units_white_city_all`) — MKUE p. 8, 4.e ; carte City.
- **C27** — Cité blanche : `AddEliteUnit` pour 2 Influence met la carte du dessus des élites dans l'offre d'unités (une carte de
  plus) ; deux fois de suite, deux cartes (`white_city_adds_elite_units_to_offer`) — MKUE p. 8, 4.e.
- **C28** — Cité bleue : `BuySpell` comme à la tour de mage, 7 Influence et un mana de la couleur du sort ; l'offre de sorts est
  complétée (`blue_city_sells_spells`) — MKUE p. 8, 4.e.
- **C29** — Cité verte : `BuyAdvancedAction` d'une carte de l'offre d'actions avancées (pas de l'offre d'unités) pour 6, offre
  complétée ; `BuyAdvancedActionFromDeck` pour 6 : la carte du dessus de la pioche sur le dessus du deck, pas l'offre
  (`green_city_sells_advanced_actions`) — MKUE p. 8, 4.e ; Compendium, *Cities* S21.
- **C30** — Cité rouge : deux `BuyArtifact` (24 Influence) ; à la fin du tour, trois artefacts tirés, deux gardés sur le dessus
  du deck, un sous la pioche (`red_city_sells_artifacts`) — MKUE p. 8, 4.e ; p. 11, 5.b ; Compendium, *End of Turn* S5.
- **C31** — Aucune cité ne vend de soin (`cities_sell_no_healing`) — Compendium, *Cities* S21.
- **C32** — Limite de main (Arythea niveau 1, limite 5) : fin du tour sur la cité conquise dont le joueur est Seigneur, ou à côté,
  pioche jusqu'à 7 ; avec une forteresse possédée voisine aussi (bonus 1), toujours 7 ; à deux cases, 5 ; cité non conquise avec
  2 Boucliers du joueur, à côté : 5 ; même bonus à la pioche de la nouvelle manche ; le seuil d'assommé reste 5
  (`conquered_city_raises_hand_limit`) — MKUE p. 11, 7.b ; p. 4, 2.g ; Compendium, *End of Turn* S8, *Cities* S20.
- **C33** — Le Conquérant ne compte pas les Boucliers d'une cité (`conqueror_ignores_city_shields`) — MKUE p. 15 ; carte de score.

### Vue, retour en arrière, persistance

- **C34** — La vue montre chaque cité révélée : couleur, niveau, défenseurs face cachée (couleurs seulement, `face_down_defenders`
  de la spec 043 étendu à plusieurs jetons par case) ou face visible, Boucliers dans l'ordre, conquise ou non, Seigneur ; et, en
  combat, les valeurs effectives des défenseurs (armure, attaque, capacités gagnées) (`view_shows_cities`).
- **C35** — Retour en arrière : le pas qui retourne les défenseurs ne s'annule pas ; un assaut dont les défenseurs étaient déjà
  visibles s'annule jusqu'à l'engagement ; `AdvancedActionDrawnBlind` et `EliteUnitAdded` ne s'annulent pas, `ArtifactBought` si
  (`city_undo_points`) — spec 024.
- **C36** — Une partie avec une cité à moitié prise (Boucliers posés, défenseurs restants) se sauvegarde et se recharge à
  l'identique ; une sauvegarde d'avant cette spec se charge avec `CityRule::Closed` (`city_survives_save_and_load`,
  arcanum-persistance) — spec 024.

### Écran

- **C37** — Infobulle d'une cité (lue dans `PlayerView::cities`) : nom et niveau, défenseurs (noms, ou couleurs face cachée), bonus
  de la cité, Boucliers du joueur, conquise et Seigneur, ce qu'on y achète (`city_tooltip_lines`).
- **C38** — Infobulle d'une cité fermée (Première reconnaissance) : « Cité : ni conquise ni visitée dans ce scénario »
  (`closed_city_tooltip`).
- **C39** — Écran de combat : armure et attaque modifiées par la cité affichées comme les réductions (« Armure 8 (au lieu de
  7) », « Attaque de glace 7 (au lieu de 5) »), capacités gagnées marquées « (cité) » (`combat_shows_city_bonus`).
- **C40** — Interaction dans une cité : en-tête « Interaction : Cité rouge », ligne des Boucliers, boutons d'achat de sa couleur,
  recrutement selon la cité, pas de bouton de soin (`city_interaction_buttons`).
- **C41** — Panneau : « Limite de main : 7 (+2 cité) » quand le bonus de la cité compte (`hand_limit_city_line`).

## Cas limites et refus

- **L1** — Piles de jetons vides à la révélation : remélange de la défausse, puis pile de secours (specs 031, 033) ; plus aucun
  jeton nulle part : la cité a moins de défenseurs ; sans aucun, le premier assaut la conquiert aussitôt, sans Bouclier, et le
  joueur en est le Seigneur (comme L4 de la spec 043).
- **L2** — Assaut sur la case X : permis, Réputation −1 bornée à la piste (spec 043) ; l'interaction dans la cité conquise reste
  refusée sur la case X (spec 040).
- **L3** — Défier un ennemi errant voisin depuis la cité, au moment « à engager » : permis (spec 043).
- **L4** — Retraite vers une case qui n'est plus sécurisée : impossible en solo (on a assailli depuis une case atteinte par un pas
  normal) ; après Attaque souterraine, la case de départ du sort (spec 057).
- **L5** — `AddEliteUnit` dans un scénario sans unités d'élite (aucun scénario actuel n'a à la fois des cités ouvertes et pas
  d'élites) : refusée.
- **L6** — Pioche d'actions avancées vide : `BuyAdvancedActionFromDeck` refusée ; offre vide : aucun achat de l'offre.
- **L7** — Pioche d'artefacts : `BuyArtifact` refusée si la pioche n'a pas au moins une carte de plus que les artefacts déjà dus
  ce tour (un achat de plus doit pouvoir tirer « un de plus », MKUE p. 11, 5.b).
- **L8** — Ennemi déjà Brutal dans la cité rouge : pas de double effet (C14) ; même règle pour Toxique dans la cité verte,
  sans cas possible avec les jetons du jeu de base (C15).
- **L9** — La cité blanche ne change rien aux résistances ; la cité bleue ne touche pas l'attaque physique ; un défenseur qui
  n'attaque pas (Illusionnistes) n'a que sa créature renforcée (C16).
- **L10** — Plusieurs cités conquises à portée (impossible avec les tuiles actuelles, toujours éloignées) : le plus grand bonus.

## Information cachée

- Visible du joueur (`PlayerView::cities`, nouveau) : pour chaque cité révélée, sa case, sa couleur, son niveau, ses Boucliers
  (joueur de chacun, dans l'ordre), conquise ou non, son Seigneur.
- Défenseurs : `PlayerView::face_down_defenders` (spec 043) passe à **plusieurs couleurs par case** (une cité en a jusqu'à cinq) ;
  les défenseurs face visible sont dans `enemy_tokens` (inchangé).
- Combat : les valeurs effectives d'un défenseur (armure, attaque, capacités gagnées, couleur de sa cité) sont lisibles dans la
  vue du combat, pour l'écran (C39).
- `PlayerView::city_hand_bonus` (nouveau, à côté de `keep_hand_bonus`) : le bonus de limite de main d'une cité à portée (C41).
- Interaction : `PlayerView::interaction` porte le bonus des Boucliers (C40).
- Caché : l'ordre des piles de jetons, d'actions avancées, d'artefacts et d'élites (inchangé).

## Libellés (i18n)

Nouvelles clés (fr / en). Le nom des cités vient de `site.city.*` (« Cité bleue » / « Blue City »).

| Clé | Paramètres | fr | en |
|---|---|---|---|
| `board.hover.city.level` | `{city}`, `{level}` | {city}, niveau {level} | {city}, level {level} |
| `board.hover.city.defenders` | `{enemies}` | Défenseurs : {enemies} | Defenders: {enemies} |
| `board.hover.city.face_down` | `{colors}` | Défenseurs face cachée : {colors} | Face-down defenders: {colors} |
| `board.hover.city.bonus_white` | — | Bonus des défenseurs : Armure +1 | Defenders' bonus: Armor +1 |
| `board.hover.city.bonus_blue` | — | Bonus des défenseurs : attaque de glace ou de feu +2, de feu glacé +1 | Defenders' bonus: Ice or Fire Attack +2, Cold Fire Attack +1 |
| `board.hover.city.bonus_red` | — | Bonus des défenseurs : brutal si leur attaque est physique | Defenders' bonus: Brutal if their attack is physical |
| `board.hover.city.bonus_green` | — | Bonus des défenseurs : toxique si leur attaque est physique | Defenders' bonus: Poison if their attack is physical |
| `board.hover.city.shields` | `{count}` | Vos Boucliers : {count} | Your Shields: {count} |
| `board.hover.city.leader` | — | Conquise : vous en êtes le Seigneur | Conquered: you are its Leader |
| `board.hover.city.offer_white` | — | On y recrute toutes les unités ; unité d'élite ajoutée à l'offre pour 2 Influence | Recruit any Unit here; add an Elite Unit to the offer for 2 Influence |
| `board.hover.city.offer_blue` | — | On y achète des sorts : 7 Influence et un mana | Buy Spells here: 7 Influence and one mana |
| `board.hover.city.offer_red` | — | On y achète des artefacts : 12 Influence | Buy Artifacts here: 12 Influence |
| `board.hover.city.offer_green` | — | On y achète des actions avancées : 6 Influence | Buy Advanced Actions here: 6 Influence |
| `board.hover.city.hand_limit` | — | Finir le tour ici ou à côté : limite de main +1, +2 pour le Seigneur | End your turn here or next to it: Hand limit +1, +2 for the Leader |
| `site.conquered.city` | `{site}` | {site} conquise | {site} conquered |
| `combat.enemy.attack_increased` | `{attack}`, `{printed}` | Attaque {attack} (au lieu de {printed}) | Attack {attack} (instead of {printed}) |
| `combat.enemy.city_ability` | `{ability}` | {ability} (cité) | {ability} (city) |
| `interaction.city_shields` | `{bonus}` | Boucliers dans la cité : Influence +{bonus} | Shields in the city: Influence +{bonus} |
| `purchase.artifact` | `{cost}` | Acheter un artefact ({cost}) | Buy an Artifact ({cost}) |
| `purchase.advanced_action_deck` | `{cost}` | Acheter la carte du dessus de la pioche d'actions avancées ({cost}) | Buy the top card of the Advanced Action deck ({cost}) |
| `purchase.elite_unit` | `{cost}` | Ajouter une unité d'élite à l'offre ({cost}) | Add an Elite Unit to the offer ({cost}) |
| `level.hand_limit_city` | `{limit}`, `{bonus}` | Limite de main : {limit} (+{bonus} cité) | Hand limit: {limit} (+{bonus} city) |

Textes obtenus : `board.hover.city.level` avec `{city}` = « Cité bleue », `{level}` = 5 donne « Cité bleue, niveau 5 » / « Blue
City, level 5 » ; `site.conquered.city` avec `{site}` = « Cité blanche » donne « Cité blanche conquise » / « White City
conquered » ; `combat.enemy.city_ability` avec `{ability}` = « brutal » (`enemy.ability.brutal`) donne « brutal (cité) ».

Clé dont la **valeur change** : `board.hover.fortified` (fr « Cité : l'assaut n'est pas encore disponible », en « City: assault
not available yet ») devient « Cité : ni conquise ni visitée dans ce scénario » / « City: neither conquered nor entered in this
scenario » (C38) ; son seul lecteur est l'infobulle de déplacement (`board/screen.rs`), et le test `arcanum-ui/tests/deplacement.rs`
qui la cite garde la clé.

Clés réutilisées, relues dans `assets/i18n/fr.json` de `main` :

- `site.city.red|blue|white|green` : « Cité rouge », « Cité bleue », « Cité blanche », « Cité verte ».
- `board.hover.assault` : « ⚔ Assaut : Réputation −1, combat obligatoire » (pas vers une cité non conquise).
- `board.hover.defender_fortified_twice` : « Défenseur doublement fortifié : aucun tir à distance ».
- `enemy.color.*` pour les couleurs face cachée : « gardes de forteresse », « tour de mage », « donjon », « cité ».
- `enemy.ability.brutal` « brutal », `enemy.ability.poison` « toxique ».
- `combat.enemy.armor_reduced` : « Armure {armor} (au lieu de {printed}) » avec 8 et 7 : « Armure 8 (au lieu de 7) » (cité
  blanche ; le libellé dit « au lieu de » sans dire réduite ou augmentée, il convient).
- `combat.enemy.site_fortified` « fortifié (site) », `combat.enemy.fortified_twice` « doublement fortifié : pas de cible à
  distance ».
- `interaction.header` « Interaction : {site} » avec `{site}` = « Cité rouge » : « Interaction : Cité rouge ».
- `purchase.spell` « Acheter : {name} ({cost} + mana {color}) » ; `purchase.advanced_action` « Acheter : {name} ({cost}) » avec
  6 ; `units.recruit` « Recruter : {name} ({cost}) ».
- `reward.pending` « Récompense en fin de tour : {count} artefact(s) » (après un achat d'artefact).
- `assault.withdrew` « Assaut repoussé : retour sur la case de départ ».
- `undo.blocked.revealed` « Une information a été révélée (tuile, carte ou dé) : impossible de revenir avant. »

Aucun symbole nouveau : seulement des mots et « ⚔ », déjà gardé par un test (spec 043).

## Interface

- **Infobulle d'une cité** (C37, `PlayerView::cities`, `face_down_defenders`, `enemy_tokens`) : « Cité bleue, niveau 5 », puis
  les défenseurs (noms, ou couleurs face cachée), le bonus de sa couleur, « Vos Boucliers : n » s'il y en a, « Conquise : vous en
  êtes le Seigneur » une fois conquise, ce qu'on y achète, la ligne de limite de main ; vers une cité non conquise atteignable, les
  lignes de l'assaut de la spec 043 (coût, « ⚔ Assaut … », doublement fortifié).
- **Cité fermée** (C38) : la ligne `board.hover.fortified` au nouveau texte.
- **Plateau** : le drapeau du joueur sur une cité conquise, comme sur un site conquis (spec 043) ; tant qu'elle ne l'est pas, rien
  de plus que la figurine de cité (vérification à l'œil). Le niveau ne s'affiche que dans l'infobulle.
- **Écran de combat** (C39, vue du combat) : armure et attaque effectives avec « (au lieu de …) », capacités gagnées suivies de
  « (cité) », mentions de fortification de la spec 043.
- **Interaction** (C40, `PlayerView::interaction`, `offers`) : en-tête avec le nom de la cité, ligne « Boucliers dans la cité :
  Influence +n » sous la réputation, boutons d'achat de la couleur (artefact ; sorts avec choix du mana ; actions avancées de
  l'offre et « carte du dessus » ; « Ajouter une unité d'élite »), recrutement depuis l'offre d'unités selon la cité ; pas de
  bouton de soin.
- **Limite de main** (C41, `PlayerView::city_hand_bonus`) : ligne du niveau dans le panneau, avec le plus grand des bonus.
- **Historique** : les nouveaux événements y passent par les libellés existants de l'historique ou ceux ci-dessus (choix du fil
  écran, vérification à l'œil).

**Parties préparées** (`docs/verification-manuelle.md`), toutes dans le bac à sable avec la tuile royaume de cité posée en
A + B (coordonnées exactes et arrivée sur une case voisine relues dans `docs/reference/bac-a-sable.md` au fil écran ; ennemis errants
retirés du chemin s'ils gênent) :

- `cite-bleue` : Arythea, **nuit**, début du tour, cité bleue niveau 5 (Mages de glace, Mages de feu, Glaceurs en tête des piles)
  face cachée, figurine à deux pas. Main : Marche ×2, Détermination, Rage, Endurance ; un cristal bleu et un rouge ; Catapultes
  dans la zone des unités. Avancer à côté retourne les défenseurs de nuit (plus de retour en arrière), infobulle de la cité,
  assaut, écran de combat (« Attaque de glace 7 (au lieu de 5) », Glaceurs doublement fortifiés), siège de glace 5 des Catapultes (mana bleu) sur
  les Mages de feu (armure 5) ; finir le combat sans tout vaincre : Bouclier et retraite, infobulle « Vos Boucliers : 1 ».
- `cite-blanche` : Arythea, jour, combat engagé dans la cité blanche niveau 5 (Gardiens, deux Canonniers) : « Armure 8 (au lieu
  de 7) », Gardiens doublement fortifiés, Bouclier de glace en main avec un cristal bleu.
- `cite-blanche-conquise` : Arythea, jour, début du tour, figurine sur la cité blanche conquise (3 Boucliers, Seigneur) ;
  Bûcherons et Catapultes dans l'offre d'unités ; main : Promesse, Menace, Marche, Rage, Détermination ; réputation 0.
  Interagir : « Boucliers dans la cité : Influence +3 », recruter les Bûcherons (icône Village seulement), « Ajouter une unité
  d'élite à l'offre (2) » ; finir le tour : « Limite de main : 7 (+2 cité) » et pioche de 7.
- `cite-rouge-conquise` : même départ, cité rouge conquise (3 Boucliers), main avec assez d'Influence pour un artefact :
  Promesse renforcée (4) et Menace renforcée (5), un cristal blanc et un rouge, plus 3 pour les Boucliers = 12. Acheter un artefact, finir le tour : fenêtre du choix des artefacts.
- `cite-verte-conquise` : même départ, cité verte conquise. Acheter une action avancée de l'offre, puis la carte du dessus de la
  pioche (plus de retour en arrière).

Chaque situation se joue en avançant depuis le départ de sa partie ; la retraite et la conquête ont des parties séparées
(issue privée 332).

### Raccourcis clavier

Aucun : les achats et le recrutement passent par les boutons de l'interaction, comme dans les autres sites ; l'assaut est un
pas (spec 043).

## Effets sur les specs déjà implémentées

- **043** : `defender_color` gagne les cités (couleurs par niveau, plusieurs défenseurs) ; `is_fortified_site` et
  `is_assault_target` comptent les cités selon le scénario ; la révélation des défenseurs de cité vaut aussi de nuit ;
  `EndCombat` d'un assaut de cité ; `face_down_defenders` porte plusieurs couleurs par case.
- **014** : `MoveCost::Fortified` d'une cité n'est plus « toujours refusé » ; le test
  `entering_a_fortified_site_is_an_assault_except_cities` (`arcanum-core/tests/deplacement.rs`) garde son cas cité en passant par
  Première reconnaissance (cités fermées), sans changer ses assertions ; la PR le signale.
- **040, 041, 042, 043** : `interaction_site`, le recrutement, les achats et le soin connaissent la cité.
- **063** : Première reconnaissance prend `CityRule::Closed` : rien ne change pour elle (C6).
- **057** : Attaque souterraine compte les cités parmi ses arrivées.
- **035, 043** : la pioche prend le plus grand des bonus forteresse et cité.
- **Bac à sable** : les tests qui révèlent sa tuile de cité voient maintenant `CityPlaced` et des défenseurs ; relevé au fil
  moteur (`grep -rln "Core(5)\|Core(6)\|Core(7)\|Core(8)\|core_city" crates/*/tests`) ; leurs assertions sur les jetons d'ennemis
  peuvent changer, et chaque changement est signalé dans la PR. La page `docs/reference/bac-a-sable.md` est régénérée si elle change.

## Hors périmètre

- Scénario Conquête en solitaire (niveaux 5 puis 8, fin quand toutes les cités sont conquises, décompte des cités : +10 par
  cité, +15 pour toutes, MKUE p. 19) : spec 064.
- Décompte des cités des scénarios compétitifs (7 et 4 gloire, Le plus grand conquérant de cités) : multijoueur.
- Mégapole (MKUE p. 18), cités au hasard (MKUE p. 16, *Random Cities*) : variantes, pas prévues.
- Assaut coopératif (MKUE p. 13), plusieurs joueurs dans une cité, combat entre joueurs : hors du solo.
- Tuile de cité posée à la mise en place : aucun scénario n'en a.

## Plan de développement

Spec moteur et écran à la fois : deux fils successifs sur la même branche (issue privée 118), plan détaillé
(`docs/specs/plans/062-cites.md`) au début du premier.

1. **Fil moteur** : contenu `cities.ron`, `CityRule`, révélation et défenseurs, assaut, bonus des cités en combat, Boucliers,
   conquête et retraite, cité conquise (déplacement, interaction, achats, recrutement, limite de main), vue, retour en arrière,
   persistance, tests existants du bac à sable : C1 à C36, L1 à L10.
2. **Fil écran** : libellés, infobulle, écran de combat, interaction, ligne de limite de main, parties préparées, vérification à
   l'œil, docs (`architecture.md`, `verification-manuelle.md`, `index-par-notion.md`, `glossaire.md`) : C37 à C41.

## Décisions (2026-11-19)

Mathieu : « validé », les six propositions retenues.

- **Q1** — Bac à sable : une cité de **niveau 5** (`CityRule::Levels([5])`).
- **Q2** — Première reconnaissance : cités fermées, **ni niveau ni défenseurs** (C6, C38).
- **Q3** — Tout ce qui concerne une cité se lit dans l'**infobulle** de sa case, sans fenêtre ni section du panneau (C37).
- **Q4** — L'action avancée à l'aveugle de la cité verte a son **propre bouton**, sans confirmation ; la carte est révélée à
  l'achat (C29, C35).
- **Q5** — Les unités d'élite ajoutées par la cité blanche **restent dans l'offre** jusqu'à son renouvellement à la nouvelle
  manche, sans autre règle (C27).
- **Q6** — **Deux fils** d'implémentation, moteur puis écran (Plan de développement).

## Questions ouvertes

Aucune.
