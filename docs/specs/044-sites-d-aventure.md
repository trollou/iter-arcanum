# 044 — Sites d'aventure

- Statut : implémentée le 2026-11-05 (validée le même jour, propositions de Q1 à Q7 retenues par Mathieu)
- Jalon : J4
- Dépend de : 021 (cristaux), 024 (retour en arrière), 025 (jetons de mana), 030 à 034 (jetons d'ennemis, combat, éléments,
  capacités, Blessures), 035 (gloire, niveaux), 041 (Bouclier, combat sans unités, récompense artefact), 043 (site conquis,
  drapeau, récompense sort, jetons face visible), 050 (offres), 063 (décompte) ; toutes implémentées
- Livret : MKUE p. 6 (4 ; 6.c, 6.e) ; p. 7 (4.d) ; p. 8 (*Combat with Enemies* 1.b, 2.d, 3) ; p. 10 (*Combat Outcomes* 2.b, 3.b
  et 3.c) ; p. 11 (*End of the Turn* 4.a, 5.a à 5.c ; *Wounds and Healing*, note de 3.d) ; p. 15 (*The Greatest Adventurer*).
  Cartes de site Dungeon, Tomb, Monster Den, Spawning Grounds, Magical Glade : `docs/regles/cartes/sites-et-cites.md` ; termes
  français : cartes de site FR (`docs/regles/scans/fr/rule_cards/`, sans le tombeau). FAQ p1 (donjon de jour, mana noir) ; FAQ p3
  (ennemis défaussés ou restés, clairière). Compendium, *Turn Start* II ; *Mana* S3, S5, Q7, S14 ; *Movement* S11 ; *Combat* Q1.d,
  Q2, S15, S18, S39, S40, S43 ; *Healing* S2, S3 ; *Resting* S5 ; *End of Turn* Q4 ; *End of Round* S1 ; *Miscellaneous* S10, S14,
  S15
- Crates touchées : arcanum-core (règles), arcanum-ui, arcanum-persistance (parties préparées), `assets/i18n/`

## Objectif

Les donjons, tombeaux, antres et Sombres Terriers cessent d'être des cases vides. Debout sur l'un d'eux, le héros peut y
**entrer** : c'est l'action du tour, et un combat aussitôt contre un ou deux monstres tirés au hasard (marron, ou rouge pour le
tombeau). Au donjon et au tombeau, on combat **seul** (sans unités) et sous les **règles de Nuit**. Vaincus, les monstres laissent
le site **conquis** (Bouclier, drapeau) et une **récompense** prise en fin de tour : sort ou artefact selon un **dé de
récompense**, sort et artefact, ou des **cristaux aléatoires**. Un monstre d'antre ou de Sombre Terrier non vaincu attend le
prochain visiteur ; un donjon ou un tombeau conquis se rejoue pour la gloire. La **clairière magique** donne un jeton de mana
(or de jour, noir de nuit) au début du tour et permet de **jeter une Blessure** en fin de tour. L'Aventurier compte enfin ses
Boucliers.

## Règles

### Entrer dans un site d'aventure

- **Sites concernés** (MKUE p. 8, *Combat with Enemies* 1.b) : donjon et tombeau, conquis ou non ; antre et Sombre Terrier **non
  conquis**. Les ruines (aussi sites d'aventure) attendent la spec 045 : rien ne change pour elles.
- **Être sur la case** (Compendium, *Miscellaneous* S14) : la figurine est sur le site, quel que soit le moment où elle y est
  arrivée (début du tour, pas de ce tour). Passer, s'arrêter ou finir son tour sur le site sans y entrer est permis
  (MKUE p. 7, 4.d) : le site ne termine pas le déplacement.
- **L'action du tour** (MKUE p. 6, 6.c et 6.e ; Compendium, *Movement* S11) : `EnterAdventureSite` termine le déplacement et
  ouvre l'action ; points de mouvement et d'influence non dépensés perdus ; ensuite ni pas, ni exploration, ni interaction, ni
  repos. Aux mêmes conditions que l'interaction et l'incendie (`turn_action_taken`, specs 040 et 041) : pas après une
  interaction, un combat, une provocation ou un défi ce tour (Compendium, *Combat* S40), ni avec une main de seules Blessures sans
  carte jouée. Permis sur la case X de réputation (il n'y a pas d'habitants), et au dernier tour de la partie (spec 063).
- **Combat aussitôt** (Compendium, *Combat* Q1.d et Q2) : les jetons sont tirés ou retournés et le combat commence dans la même
  décision, sans moment « à engager » : ni soin, ni défi d'ennemi errant voisin (MKUE p. 8, 2.d ; Compendium, *Combat* S40).
- **Jetons combattus** (cartes de site) :
  - **Donjon** : un jeton **marron** tiré ; **tombeau** : un jeton **rouge** (Draconien) tiré. Conquis ou non, un nouveau jeton à
    chaque entrée.
  - **Antre** : le jeton resté sur la case s'il y en a un, sinon un jeton marron tiré.
  - **Sombre Terrier** : les jetons restés, complétés à **deux** par des jetons marron tirés (Compendium, *Miscellaneous* S15 ;
    voir Q2 pour le moment du tirage).
  - Tirage comme les autres jetons (remélange de la défausse, pile de secours de la spec 033). Plus aucun jeton nulle part : le
    combat a moins d'ennemis ; sans aucun, le site est conquis sans combat (théorique, comme la spec 043, L4).
- **Ce ne sont pas des ennemis errants** (MKUE p. 7, 4.b à 4.d ; Compendium, *Combat* S15 et S40) : un jeton resté sur un antre ou
  un Sombre Terrier ne bloque pas la case, n'est pas provoqué, ne se défie pas, et les trajets (spec 036) passent dessus. Un
  Draconien vaincu dans un tombeau ne donne pas de réputation (seule la case imprimée d'un ennemi errant en donne, spec 040).

### Donjon et tombeau : seul, de nuit

- **Sans unités** (cartes Dungeon et Tomb ; Compendium, *Combat* S43) : comme l'incendie d'un monastère (spec 041) : aucune
  capacité d'unité en combat, aucun dégât assigné à une unité. Les unités restent dans la zone et servent avant l'entrée.
- **Règles de Nuit pendant le combat** (cartes ; FAQ p1 ; Compendium, *Mana* S3, S5 et Q7, *Miscellaneous* S10) : du début à la
  fin du combat, le moment de la journée **du combat** est la Nuit, même de jour. Le moteur le lit partout où le paiement du mana
  dépend du jour : le mana **or** (dé, jeton) ne paie plus rien, le mana **noir** est permis (dés noirs pris de la Source, jetons
  noirs). Comme le mana noir ne paie encore que l'effet renforcé des sorts (`Unsupported` jusqu'à la spec 054), l'effet visible
  aujourd'hui est la perte de l'or ; la règle est posée pour les sorts et les compétences à venir (effets qui varient selon le
  jour : specs 051, 052, 054). Le combat fini, on n'est plus dans le donjon (Compendium, *Combat* Q2) : la journée redevient
  celle de la manche.
- Un jeton or gagné de jour (clairière, Cristallisation…) ne paie donc rien au donjon ; un dé or pris de jour doit être utilisé
  aussitôt, il ne se garde pas pour le donjon (Compendium, *Mana* S5 : c'est déjà le cas, un dé pris paie une carte jouée).
- Antre et Sombre Terrier : règles de la manche, unités permises (rien n'est imprimé sur leurs cartes).

### Issue

- **Victoire, site non conquis** (MKUE p. 10, 2.b ; cartes de site) : tous les jetons du site vaincus, à la fin du combat, un
  **jeton Bouclier** du joueur marque le site (`GameState::shields`, drapeau de la spec 043) ; le héros reste sur la case
  (Compendium, *Combat* S18). Récompense, prise à la fin du tour :
  - **Donjon** : un **dé de récompense** est lancé aussitôt (carte : « mark the space … and roll a mana die ») ; or ou noir : un
    **sort**, sinon un **artefact**. Ce dé est un dé de la réserve, pas de la Source (Compendium, *Mana* S14) : il ne change pas
    la Source, il a son propre générateur (ADR 0007), et l'or comme le noir valent de jour comme de nuit.
  - **Tombeau** : un **sort** et un **artefact**.
  - **Antre** : deux **cristaux aléatoires**.
  - **Sombre Terrier** : un **artefact** et trois **cristaux aléatoires**.
- **Victoire, donjon ou tombeau déjà conquis** (cartes, *While Conquered*) : la gloire de l'ennemi seulement ; ni Bouclier, ni
  récompense. Antre et Sombre Terrier conquis : on n'y entre plus, ce sont des cases ordinaires.
- **Défaite** (MKUE p. 10, 3.b et 3.c ; FAQ p3 ; Compendium, *Combat* S39) : au donjon et au tombeau, les jetons non vaincus sont
  **défaussés** ; à l'antre et au Sombre Terrier, ils **restent face visible** sur la case et seront combattus par le prochain
  visiteur. Les jetons vaincus quittent la case comme d'habitude. Pas de retraite : le héros reste sur le site (Compendium,
  *Combat* S18), une case sécurisée.
- **Jamais de réputation** pour ces combats (FAQ p3, ennemis verts et rouges).

### Récompenses de fin de tour

- **Ordre** (MKUE p. 11, 4 et 5) : bénéfices de la case (mine, clairière), puis les récompenses : **cristaux aléatoires**, puis
  artefact (spec 041), puis sort (spec 043), puis le changement de niveau (spec 035) et l'action avancée d'un niveau pair (spec
  050), puis la pioche. Le livret laisse l'ordre des récompenses au joueur (5) ; en solo l'ordre fixe ne change rien.
- **Cristal aléatoire** (MKUE p. 11, 5.a ; cartes Monster Den et Spawning Grounds ; Compendium, *End of Turn* Q4) : un dé de
  récompense par cristal, **un à la fois** : couleur de base, ce cristal (rien si l'inventaire en a déjà 3, spec 021 : pas de
  jeton à la place, contrairement aux effets de la spec 025) ; **noir**, 1 gloire à la place ; **or**, le joueur **choisit** la
  couleur avant le dé suivant (une couleur dont l'inventaire n'est pas plein ; toutes pleines, rien à choisir).
- **Gloire du noir** : gagnée avant le changement de niveau, elle peut faire franchir un seuil ce tour (MKUE p. 11, 6).
- **Sort et artefact** : les mécanismes des specs 041 (deux artefacts tirés, un gardé) et 043 (un sort de l'offre) servent tels
  quels ; un tour ne compte qu'un combat, donc au plus un artefact et un sort par tour.

### Clairière magique

- **Imprégnation magique** (carte Magical Glade ; Compendium, *Turn Start* II) : au début de son tour, une figurine sur une
  clairière gagne un **jeton de mana** or de jour, noir de nuit, comme les jetons de la spec 025 (rendu en fin de tour s'il n'est
  pas utilisé). Au début de chaque tour, nouvelle manche comprise ; pas si le tour est perdu par l'annonce obligatoire de fin de
  manche (MKUE p. 6, 4). Le jeton noir ne paie encore rien (sorts, spec 054).
- **Vapeurs curatives** (carte ; MKUE p. 11, 4.a et note de *Wounds and Healing* 3.d ; FAQ p3 ; Compendium, *Healing* S2 et S3,
  *Resting* S5, *End of Round* S1) : en finissant son tour sur une clairière, le joueur **peut** jeter **une** Blessure de sa
  main ou de sa défausse (rendue à la pile de Blessures). Ce n'est pas un soin : pas les unités, pas de points de soin, rien qui
  compte comme une Blessure soignée. Permis après un repos ; pas en annonçant la fin de la manche (pas de fin de tour). Le choix
  fait partie de la fin du tour (`EndTurn`, voir Q5).

### Décompte

- **L'Aventurier** (MKUE p. 15, *The Greatest Adventurer*) : 2 points par Bouclier du joueur sur un site d'aventure (donjon,
  tombeau, antre, Sombre Terrier ; ruines à la spec 045). Sans titre en Première reconnaissance (spec 063).

### Désigner un ennemi du combat par son jeton

- Le Sombre Terrier met **deux jetons sur une même case** (et plus tard certaines ruines, spec 045, et les cités, spec 062). Les
  actions et événements de combat qui désignent un ennemi **par sa case** (cibles d'attaque et de parade, dégâts, ennemis
  vaincus, parés, invocations…) le désignent désormais **par son jeton** (`EnemyToken`, unique : ennemi et exemplaire). Une case
  du plateau de jeu porte une **liste** de jetons. Changement technique, sans effet pour le joueur (Q1).

### Exemple : un donjon de jour

Bac à sable, Jour. La campagne 7 est explorée ; la figurine arrive sur son donjon (plaine, côté S-E). Elle a une unité, un jeton
or (clairière du matin) et 2 points de mouvement de reste.

```
Entrer dans le donjon        → AdventureSiteEntered (mouvement et influence perdus)
                             → EnemyTokenPlaced (Minotaure, marron) — point de non-retour
                             → CombatStarted (règles de Nuit, sans unités)
  attaque à distance : rien · parade : Parade 5 (le jeton or ne paie pas Détermination renforcée)
  attaque : Attaque 5 → EnemiesDefeated (gloire 4)
Fin du combat                → CombatEnded ; SiteConquered (Bouclier, drapeau) ; RewardDieRolled (vert) : un artefact
Fin du tour                  → ArtifactsDrawn (deux) → choix → pioche
```

Variante, Sombre Terrier, le Minotaure vaincu, la Méduse non : la Méduse reste face visible sur la case. Au tour suivant, entrer à
nouveau tire un jeton marron et le combat oppose la Méduse et ce nouveau jeton ; vaincre les deux conquiert le terrier : un
artefact et trois cristaux aléatoires, par exemple `bleu`, `or` (le joueur choisit rouge), `noir` (+1 gloire).

## Modèle

Termes du glossaire : Site d'aventure (nouveau, voir plus bas), Donjon, Tombeau, Antre, Sombre Terrier, Clairière magique,
Jeton d'ennemi, Couleur d'ennemi, Face visible / face cachée, Jeton Bouclier, Récompense, Sort, Artefact, Cristal, Jeton de mana,
Jeter (une Blessure), L'Aventurier, Unité. **Nouveaux termes** (ajoutés au glossaire avec cette spec ; français des cartes de site
FR et de la FAQ) :

| Livret | Français | Code |
|---|---|---|
| Adventure site | Site d'aventure (carte FR : « site d'aventures ») | `Site::is_adventure` |
| Enter (an adventure site) | Entrer dans (carte FR : « pénétrer dans ») | `Action::EnterAdventureSite`, `AdventureSiteEntered` |
| Night rules (dungeon, tomb) | Règles de Nuit (carte FR) | `GameState::combat_time_of_day` |
| Mana die (from the bank), reward roll | Dé de récompense | `RewardDieRolled`, `RngUsage::RewardDice` |
| Random crystal (reward) | Cristal aléatoire | `PlayerState::crystal_rewards` |
| Choose the color (gold rolled) | Choisir la couleur du cristal | `Action::ChooseCrystalColor` |
| Healing Essence (glade) | Vapeurs curatives (carte FR) | `EndTurn { glade, … }`, `WoundThrownAway` |
| Imbued with Magic (glade) | Imprégnation magique (carte FR) | `ManaTokenGained` au début du tour |

Données nouvelles :

- **Plateau de jeu** : une case porte une **liste** de jetons d'ennemis (Q1) ; les jetons restés sur un antre ou un Sombre
  Terrier y sont, face visible. Un site d'aventure conquis porte un Bouclier (`GameState::shields`).
- **État du joueur** : le site d'aventure où il est entré ce tour (règles de Nuit et sans unités pour un donjon ou un tombeau,
  issue du combat), vidé à la fin du tour ; les **cristaux aléatoires** à lancer en fin de tour ; les choix de couleur en
  attente (or). Les récompenses sort et artefact existent déjà (specs 041 et 043).
- **Moment de la journée du combat** : celui de la manche, sauf la Nuit pendant un combat de donjon ou de tombeau.
- **Générateur** : un usage de plus, `RewardDice`, avec un numéro de flux libre (ADR 0007).
- **Vue du joueur** : les jetons par case (listes), le site d'aventure en cours, le moment de la journée du combat, les
  cristaux aléatoires et choix de couleur en attente, le dé de récompense du donjon (dans les événements).

## Actions et événements

| Action | Validation (refusée si…) | Événements produits |
|---|---|---|
| `EnterAdventureSite` (nouvelle) | figurine hors d'un donjon, d'un tombeau, ou d'un antre ou Sombre Terrier non conquis ; action du tour déjà prise ou impossible (`turn_action_taken`) | `AdventureSiteEntered { player, hex }` ; les `EnemyPileReshuffled` au besoin, un `EnemyTokenPlaced { token, hex }` par jeton tiré ; `CombatStarted { player, enemies }` (jetons restés et tirés) ; sans aucun jeton, `SiteConquered` sans combat |
| Actions de combat (031, 042, étendues) | inchangées ; au donjon et au tombeau, aucune capacité d'unité, aucun dégât sur une unité ; mana payé selon la journée du combat | inchangés |
| `DeclareAttack`, `DeclareBlock`, `AssignDamage` (031, 042, changées, Q1) | inchangée, l'ennemi désigné par son jeton | inchangés, ennemis désignés par leur jeton |
| `EndCombat` (031, étendue) | inchangée | `CombatEnded`, puis pour un site d'aventure : tous ses jetons vaincus et site non conquis, `SiteConquered { player, hex }` (et au donjon `RewardDieRolled { player, color }`) ; défaite au donjon ou au tombeau, un `EnemyTokenDiscarded` par jeton non vaincu |
| `EndTurn { discard, reclaim, glade }` (016, étendue) | `glade` donné hors d'une clairière, ou sans Blessure à l'endroit dit (main hors défausse de fin de tour, défausse) | … `TurnEnded`, cristal de la mine, `WoundThrownAway { player, from }`, puis un `RewardDieRolled` par cristal aléatoire, suivi de `CrystalGained` (couleur de base non pleine), `FameGained { fame: 1 }` (noir) ou de l'attente d'un choix (or : la suite attend) ; puis artefacts, sort, niveaux, pioche comme avant ; après `TurnStarted`, `ManaTokenGained` sur une clairière |
| `ChooseCrystalColor { color }` (nouvelle) | pas de choix de couleur en attente ; inventaire plein de `color` | `CrystalGained { player, color }`, puis les dés suivants, puis la suite de la fin du tour |
| `AnnounceEndOfRound` (023) | inchangée | une nouvelle manche : `ManaTokenGained` après `TurnStarted` si la figurine est sur une clairière |

Ordre des actions légales (recette de `architecture.md`) : choix de fin de tour : `ChooseCrystalColor` (rouge, bleu, blanc,
vert), puis `TakeArtifact`, `TakeSpell`, `GainAdvancedAction` ; hors combat, `EnterAdventureSite` après `BurnMonastery` et avant
les défis ; les `EndTurn` d'abord sans `glade`, puis main, puis défausse, pour chaque défausse.

| Événement | Effet de `apply` | Révèle une information ? | Point de non-retour ? |
|---|---|---|---|
| `AdventureSiteEntered { player, hex }` | points de mouvement et d'influence à 0 ; déplacement terminé ; site d'aventure du tour retenu (règles de Nuit et sans unités au donjon et au tombeau) | non | non |
| `EnemyTokenPlaced` (030) | le jeton s'ajoute à la liste de sa case | inchangé | **oui** (inchangé) |
| `SiteConquered { player, hex }` (043, étendu) | Bouclier sur `hex` ; récompense selon le site : tombeau, un sort et un artefact ; antre, deux cristaux aléatoires ; Sombre Terrier, un artefact et trois cristaux aléatoires ; donjon, rien (le dé suit) | non | non |
| `RewardDieRolled { player, color }` (nouveau) | au donjon, juste après la conquête : un sort (or, noir) ou un artefact de récompense ; en fin de tour : un cristal aléatoire de moins, et un choix de couleur de plus si or | **oui** | **oui** (un dé est lancé) |
| `WoundThrownAway { player, from }` (nouveau) | une Blessure quitte la main ou la défausse pour la pile de Blessures | non | non |
| `ManaTokenGained` (025) | inchangé | inchangé | inchangé |
| `TurnEnded` (016) | vide aussi le site d'aventure du tour | inchangé | inchangé |

Événements de combat qui changent de forme (Q1) : `AttackDeclared`, `EnemiesDefeated`, `BlockDeclared`, `EnemyBlocked`,
`WoundsTaken`, `PoisonWoundsDiscarded`, `EnemySummoned`, `SummonedEnemyDiscarded`, `DamageAssignedToUnit` désignent l'ennemi
par son jeton ; `EnemyTokenDiscarded` nomme aussi le jeton. Les sauvegardes d'avant sont refusées (`EVENT_FORMAT_VERSION`
inchangée jusqu'à la fin du J6, spec 024) ; parties préparées régénérées.

## Comportements attendus

Chaque cas devient au moins un test (nom indicatif entre parenthèses). C1 à C38 : moteur (arcanum-core) ; C39 à C49 : écran (arcanum-ui, sur
`game_screen`), un cas par point de la section « Interface » (issue privée 119).

Ennemis désignés par leur jeton (Q1) :

- **C1** — Deux jetons sur la même case dans un combat (scénario de test) : chacun se vise seul à l'attaque et à la parade, et
  vaincre l'un laisse l'autre en jeu (`two_enemies_on_one_space_are_distinct_targets`) — Compendium, *Miscellaneous* S15.

Entrer :

- **C2** — Figurine sur le donjon, rien fait ce tour : `EnterAdventureSite` est légal ; joué, alors `AdventureSiteEntered`,
  `EnemyTokenPlaced` d'un jeton marron, `CombatStarted` contre lui ; mouvement et influence à 0, plus de pas, d'exploration ni
  d'interaction, pas de défi (`entering_a_dungeon_starts_combat_at_once`) — MKUE p. 8, 1.b et 2.d ; p. 6, 6.e.
- **C3** — Même chose au tombeau avec un jeton rouge (`tomb_draws_a_red_token`) — carte Tomb.
- **C4** — Figurine voisine du donjon, ou sur une case sans site : pas d'`EnterAdventureSite` (`entering_needs_standing_on_the_site`)
  — Compendium, *Miscellaneous* S14.
- **C5** — Après une interaction, un combat, ou un pas qui a provoqué un ennemi errant : pas d'`EnterAdventureSite` ; sur la
  case X de réputation : permis (`entering_is_the_action_of_the_turn`) — Compendium, *Combat* S40 ; spec 040.
- **C6** — Un pas sur le donjon ne termine pas le déplacement : le pas suivant reste légal, et un trajet traverse le site
  (`adventure_site_does_not_stop_movement`) — MKUE p. 7, 4.d.

Donjon et tombeau :

- **C7** — Combat de donjon : aucune `ActivateUnit` de combat, aucune `AssignDamage` vers une unité ; les dégâts vont au héros
  (`no_units_in_dungeons_and_tombs`) — carte Dungeon ; Compendium, *Combat* S43.
- **C8** — De jour, dans le donjon : un jeton or ne paie pas l'effet renforcé d'une carte, un dé or de la Source non plus ; le
  même jeton le paie avant d'entrer et après le combat (`night_rules_in_dungeon_combat`) — FAQ p1 ; Compendium, *Mana* Q7,
  *Combat* Q2.
- **C9** — De jour, dans le donjon, la vue donne la Nuit comme journée du combat ; hors du combat, le Jour
  (`view_shows_the_combat_time_of_day`) — Compendium, *Miscellaneous* S10.
- **C10** — Antre de jour : unités permises, jeton or utilisable (`monster_den_uses_round_rules`) — carte Monster Den.

Issue :

- **C11** — Donjon : le jeton vaincu, fin du combat : `CombatEnded`, `SiteConquered`, `RewardDieRolled` ; or ou noir donnent une
  récompense sort, une autre couleur une récompense artefact ; la Source ne change pas (`dungeon_reward_die`) — carte Dungeon ;
  Compendium, *Mana* S14.
- **C12** — Tombeau conquis : une récompense sort et une récompense artefact ; pas de réputation pour le Draconien
  (`tomb_rewards_spell_and_artifact`) — carte Tomb ; Compendium, *Combat* S15.
- **C13** — Antre conquis : deux cristaux aléatoires en attente ; Sombre Terrier conquis : un artefact et trois cristaux
  aléatoires (`den_and_grounds_reward_random_crystals`) — cartes.
- **C14** — Donjon non vaincu : le jeton est défaussé (`EnemyTokenDiscarded`), la case n'a plus de jeton, pas de Bouclier, la
  figurine reste (`failed_dungeon_discards_the_enemy`) — MKUE p. 10, 3.b ; Compendium, *Combat* S18.
- **C15** — Antre non vaincu : le jeton reste face visible sur la case ; au tour suivant, entrer combat ce même jeton sans en
  tirer (`monster_den_enemy_waits`) — MKUE p. 10, 3.c.
- **C16** — Sombre Terrier, un jeton vaincu sur deux : l'autre reste ; entrer à nouveau tire un seul jeton marron et combat les
  deux ; vaincre les deux conquiert le site (`spawning_grounds_refill_to_two`) — Compendium, *Miscellaneous* S15.
- **C17** — Un jeton resté sur un antre ne bloque pas la case, n'est pas provoqué par un pas qui le longe, ne se défie pas
  (`adventure_tokens_are_not_rampaging`) — MKUE p. 7, 4.d.
- **C18** — Donjon conquis : entrer est encore légal, un nouveau jeton marron ; vaincu, sa gloire seulement : ni Bouclier de plus,
  ni dé de récompense (`conquered_dungeon_for_fame_only`) — carte Dungeon, *While Conquered*.
- **C19** — Antre et Sombre Terrier conquis : plus d'`EnterAdventureSite` (`conquered_den_is_an_ordinary_space`) — cartes.
- **C20** — Le site conquis sans aucun jeton disponible (piles et défausses vides) : `SiteConquered` sans `CombatStarted`
  (`no_token_left_conquers_without_combat`) — spec 043, L4.

Récompenses de fin de tour :

- **C21** — Deux cristaux aléatoires, dés bleu puis vert : `RewardDieRolled` bleu, `CrystalGained` bleu, `RewardDieRolled` vert,
  `CrystalGained` vert, avant les artefacts et la pioche (`random_crystals_at_end_of_turn`) — MKUE p. 11, 5.a.
- **C22** — Dé noir : `FameGained { fame: 1 }` à la place du cristal ; s'il fait franchir un seuil, `LevelGained` suit
  (`black_die_gives_one_fame`) — MKUE p. 11, 5.a et 6.
- **C23** — Dé or : la fin du tour attend ; `ChooseCrystalColor` est proposée pour chaque couleur non pleine ; le choix donne le
  cristal, puis le dé suivant est lancé (`gold_die_waits_for_a_color`) — Compendium, *End of Turn* Q4.
- **C24** — Couleur déjà à 3 : rien (ni cristal ni jeton) ; or avec les quatre couleurs pleines : pas de choix
  (`full_inventory_gets_nothing`) — MKUE p. 11, 5.a.
- **C25** — Sombre Terrier : cristaux d'abord, puis les deux artefacts tirés attendent le choix (`crystals_before_artifact`) —
  MKUE p. 11, 5.
- **C26** — Le retour en arrière ne passe pas avant un `RewardDieRolled` (`reward_die_blocks_undo`) — spec 024.

Clairière magique :

- **C27** — Fin du tour sur une clairière, une Blessure en main : les `EndTurn` sans et avec `glade: Hand` sont proposées ; avec,
  `WoundThrownAway` et une carte de plus piochée (`glade_throws_a_wound_from_hand`) — carte Magical Glade ; MKUE p. 11, 4.a.
- **C28** — Blessure seulement dans la défausse : `glade: DiscardPile` la rend à la pile de Blessures ; aucune Blessure nulle part,
  ou hors d'une clairière : pas d'`EndTurn` avec `glade` (`glade_throws_from_discard_pile`) — FAQ p3 ; Compendium, *Healing* S2.
- **C29** — Repos sur une clairière : la Blessure se jette aussi ; ce n'est pas un soin (pas de point de soin, unité blessée
  intacte) (`glade_is_not_healing`) — Compendium, *Resting* S5, *Healing* S3.
- **C30** — Annonce de la fin de la manche sur une clairière : rien n'est jeté (`no_glade_when_announcing_end_of_round`) —
  Compendium, *End of Round* S1.
- **C31** — Le tour suivant commence sur une clairière de jour : `ManaTokenGained` or après `TurnStarted` ; de nuit, noir ; ailleurs,
  rien (`glade_gives_a_mana_token_at_turn_start`) — carte ; Compendium, *Turn Start* II.
- **C32** — Nouvelle manche, figurine sur une clairière : le jeton suit `TurnStarted` ; tour perdu par l'annonce obligatoire :
  pas de jeton (`glade_token_at_new_round_not_when_forfeiting`) — MKUE p. 6, 4.
- **C33** — Le jeton or de la clairière paie l'effet renforcé d'une carte de jour, et revient à la réserve en fin de tour s'il
  n'est pas utilisé (`glade_token_is_an_ordinary_token`) — spec 025.

Décompte, sauvegarde, vue :

- **C34** — Fin de partie avec un donjon et un antre conquis : L'Aventurier 4 (`adventurer_counts_adventure_shields`) — MKUE
  p. 15.
- **C35** — Une partie sauvegardée avec un jeton resté sur un antre, deux sur un Sombre Terrier (dont un resté), un donjon
  conquis, des cristaux aléatoires et un choix de couleur en attente se recharge à l'identique ; un combat de donjon en cours
  aussi (`adventure_sites_survive_save_and_load`) — spec 024.
- **C36** — La vue donne les jetons restés (avec leur ennemi), le site d'aventure en cours, les cristaux aléatoires et les choix
  en attente (`view_shows_adventure_state`).
- **C37** — Un jeton resté sur un antre s'affiche aussi dans `enemy_tokens` de la vue, avec sa case ; ce n'est pas un ennemi
  errant pour les trajets de la vue (`view_lists_adventure_tokens`).
- **C38** — Joueur de test `FirstChoice` : il n'entre pas dans les sites d'aventure et choisit d'abord la fin de tour sans
  `glade` (`first_choice_ignores_adventure_sites`) — les parcours ne changent pas, hors jeton de clairière (risques du plan).

Écran (arcanum-ui, sur `game_screen`) :

- **C39** — Panneau : sur un site où l'on peut entrer, un bouton « Entrer dans le donjon » (nom du site) et, dessous, les points
  perdus ; absent sinon (`panel_offers_to_enter_adventure_site`).
- **C40** — Un clic sur le bouton (ou E) ouvre la confirmation : ce qui sera combattu (couleur du jeton à tirer ou ennemi resté),
  « règles de Nuit, sans vos unités » au donjon et au tombeau, la récompense ; « Entrer » joue l'action, « Annuler » ou Échap
  ferme (`enter_confirmation`).
- **C41** — Plateau : un jeton resté sur un antre est dessiné face visible sur sa case ; deux jetons d'un Sombre Terrier côte à
  côte, sans déborder de la case (`board_draws_adventure_tokens`).
- **C42** — Site d'aventure conquis : le drapeau du héros (spec 043) (`conquered_adventure_sites_show_a_flag`).
- **C43** — Infobulle d'un site d'aventure : nom, état (« Donjon conquis : combat pour la gloire seulement », ennemis restés), ce
  qu'on y combat et la récompense ; d'une clairière : ses deux effets (`hover_describes_adventure_sites`).
- **C44** — Écran de combat d'un donjon ou d'un tombeau : la ligne « Règles de Nuit : le mana or ne paie rien » et la ligne
  « Vos unités ne participent pas à ce combat » de la spec 041 (`combat_marks_night_rules_and_no_units`).
- **C45** — Panneau : après la conquête d'un donjon, « Dé de récompense : vert, un artefact » ; les récompenses en attente
  « Récompense en fin de tour : 2 cristaux aléatoires » (`panel_announces_adventure_rewards`).
- **C46** — Fin du tour avec un dé or : la fenêtre « Récompense : choisissez la couleur du cristal » montre les quatre couleurs,
  grisées si pleines, et un bouton chacune (`crystal_color_window`).
- **C47** — Mode fin du tour sur une clairière : la ligne « Clairière magique : jeter une Blessure » avec trois choix (de la main,
  de la défausse, ne pas jeter), ceux sans Blessure grisés ; choix par défaut : la main s'il y a une Blessure, sinon la défausse ;
  le nombre de cartes piochées suit le choix (`end_turn_glade_choice`).
- **C48** — Début du tour sur une clairière : la ligne « Clairière magique : +1 jeton de mana or » sous les points, jusqu'à la
  première action (`panel_announces_glade_token`).
- **C49** — Fenêtre de fin : L'Aventurier montre ses points (`final_window_shows_adventurer`).

## Cas limites et refus

- **L1** — Héros assommé dans un donjon : le combat continue (spec 034) ; vaincre le monstre conquiert le site.
- **L2** — Invocateur dans un antre ou un donjon : la créature invoquée n'est pas un jeton du site ; seule la défaite du jeton du
  site compte pour la conquête et pour ce qui reste sur la case.
- **L3** — Sombre Terrier quand les piles n'ont plus qu'un jeton nulle part : le combat n'en a qu'un, et le vaincre conquiert le
  site (tous les jetons combattus sont vaincus). Théorique, comme L4 de la spec 043.
- **L4** — Entrer dans un donjon au dernier tour de la partie ou après l'annonce de la fin de manche par un autre joueur (solo :
  sans objet) : permis ; les récompenses sont prises avant le décompte (spec 063).
- **L5** — Dé de récompense noir au donjon, de jour : un sort quand même (dé de la réserve, pas une règle de mana).
- **L6** — Offre de sorts vide pour la récompense sort : perdue (spec 043) ; pioche d'artefacts vide : perdue (spec 041).
- **L7** — Dans le donjon, de jour, un dé noir de la Source n'est plus épuisé, mais il ne paie encore rien (sorts, spec 054) :
  aucune action nouvelle.
- **L8** — Clairière et récompense dans le même tour : la Blessure se jette avant les cristaux et les artefacts (MKUE p. 11, 4 puis
  5).
- **L9** — Une Blessure défaussée par le repos de ce tour n'est pas « dans la défausse » pour la clairière au moment du choix ;
  en garder une en main et la jeter de la main revient au même.

## Information cachée

Un jeton tiré à l'entrée est révélé aussitôt (`EnemyTokenPlaced`, déjà public). Un jeton resté sur un antre ou un Sombre Terrier
est face visible : la vue le donne. Le dé de récompense est public. Rien de nouveau n'est caché.

## Libellés (i18n)

| Clé | Paramètres | fr | en |
|---|---|---|---|
| `adventure.enter` | `site` | Entrer dans : {site} | Enter: {site} |
| `adventure.lost_points` | `move`, `influence` | Entrer fait perdre : {move} mouvement, {influence} influence | Entering loses: {move} Move, {influence} Influence |
| `adventure.confirm.fight_drawn` | `count`, `color` | Combat contre {count} ennemi(s) {color}, tiré(s) au hasard. | Fight {count} {color} enemy(ies), drawn at random. |
| `adventure.confirm.fight_waiting` | `enemies` | Combat contre : {enemies}. | Fight: {enemies}. |
| `adventure.confirm.night_no_units` | — | Règles de Nuit, sans vos unités. | Night rules, without your Units. |
| `adventure.confirm.reward` | `reward` | Récompense si vous gagnez : {reward}. | Reward if you win: {reward}. |
| `adventure.confirm.no_undo` | — | Pas de retour en arrière. | No undo. |
| `adventure.confirm_yes` | — | Entrer | Enter |
| `adventure.cancel` | — | Annuler | Cancel |
| `adventure.reward.dungeon` | — | un sort (dé or ou noir) ou un artefact | a Spell (gold or black die) or an Artifact |
| `adventure.reward.tomb` | — | un sort et un artefact | a Spell and an Artifact |
| `adventure.reward.monster_den` | — | deux cristaux aléatoires | two random crystals |
| `adventure.reward.spawning_grounds` | — | un artefact et trois cristaux aléatoires | an Artifact and three random crystals |
| `adventure.reward.fame_only` | — | la gloire seulement | Fame only |
| `site.conquered.adventure` | `site` | {site} conquis | Conquered {site} |
| `board.hover.adventure.enemies_waiting` | `enemies` | Vous y attendent : {enemies} | Waiting here: {enemies} |
| `board.hover.adventure.fight` | `reward` | Entrer : combat, récompense : {reward} | Enter: combat, reward: {reward} |
| `board.hover.glade` | — | Jeton de mana au début du tour (or le jour, noir la nuit) ; finir le tour ici : jeter une Blessure | Mana token at turn start (gold by day, black at night); end your turn here: throw away a Wound |
| `combat.night_rules` | — | Règles de Nuit : le mana or ne paie rien | Night rules: gold mana pays nothing |
| `reward.die` | `color`, `reward` | Dé de récompense : {color}, {reward} | Reward die: {color}, {reward} |
| `reward.die_spell` | — | un sort | a Spell |
| `reward.die_artifact` | — | un artefact | an Artifact |
| `reward.pending_crystals` | `count` | Récompense en fin de tour : {count} cristal(aux) aléatoire(s) | Reward at end of turn: {count} random crystal(s) |
| `reward.choose_crystal` | — | Récompense : choisissez la couleur du cristal (dé or) | Reward: choose the crystal's color (gold die) |
| `end_turn.glade` | — | Clairière magique : jeter une Blessure | Magical glade: throw away a Wound |
| `end_turn.glade_hand` | — | de la main | from your hand |
| `end_turn.glade_discard` | — | de la défausse | from your discard pile |
| `end_turn.glade_none` | — | ne pas jeter | keep it |
| `glade.token` | `color` | Clairière magique : +1 jeton de mana {color} | Magical glade: +1 {color} mana token |

**Changements à l'implémentation de l'écran** (2026-11-05) : `reward.die` et `end_turn.glade` étant des libellés, leurs variantes ne peuvent pas être des clés filles dans le JSON ; elles s'écrivent `reward.die_spell`, `reward.die_artifact`, `end_turn.glade_hand`, `glade_discard`, `glade_none`. `adventure.confirm.fight_drawn` prend `count` : un Sombre Terrier sans jeton resté en tire deux. `reward.die` s'écrit sans la flèche « → », absente de la police du jeu (vérification à l'œil).

**Clés existantes réutilisées** (vérifiées dans `fr.json` et `en.json`, issue privée 127) : `site.dungeon.name`, `site.tomb.name`,
`site.monster_den.name`, `site.spawning_grounds.name`, `site.magical_glade.name` (noms des sites) ; `burn.no_units` (sans unités) ;
`reward.take`, `reward.choose_artifact`, `reward.choose_spell`, `reward.pending`, `reward.pending_spell` ; `mana.red`,
`mana.blue`, `mana.white`, `mana.green`, `mana.gold`, `mana.black` (couleurs du dé et du jeton) ; `enemy.pile_color.brown`,
`enemy.pile_color.red` (couleur du jeton à tirer). Le libellé de L'Aventurier dans la fenêtre de fin existe déjà (spec 063).

## Interface

- **Entrer** (C39, C40 ; Q4) : bouton `adventure.enter` dans le panneau, comme « Incendier le monastère » (spec 041), avec
  `adventure.lost_points` dessous ; il ouvre une confirmation : `adventure.confirm.fight_drawn` ou `fight_waiting`,
  `night_no_units` au donjon et au tombeau, `adventure.confirm.reward` (`adventure.reward.*`, `fame_only` pour un donjon ou un
  tombeau conquis), `no_undo` quand un jeton sera tiré ; boutons `adventure.confirm_yes` et `adventure.cancel`.
- **Jetons sur les sites** (C41) : jetons d'ennemis face visible, dessinés comme les autres (spec 030) ; deux sur une case, côte à
  côte et réduits. Lisibilité aux zooms : vérification à l'œil.
- **Drapeau** (C42) : celui des sites conquis (spec 043), sur les sites d'aventure conquis.
- **Infobulle** (C43) : nom du site ou `site.conquered.adventure`, `board.hover.adventure.enemies_waiting`,
  `board.hover.adventure.fight` ; `board.hover.glade` sur une clairière.
- **Combat** (C44) : `combat.night_rules` et `burn.no_units` en tête de l'écran de combat, au donjon et au tombeau.
- **Récompenses** (C45, C46) : `reward.die` sous les points après la conquête d'un donjon, jusqu'à la fin du tour ;
  `reward.pending_crystals` avec les autres récompenses en attente ; fenêtre `reward.choose_crystal` comme le choix d'artefact
  (spec 041), quatre boutons de couleur (`mana.*`), grisés si pleins.
- **Fin du tour** (C47) : en mode fin du tour sur une clairière, ligne `end_turn.glade` et trois boutons à choix unique
  (`end_turn.glade_hand`, `glade_discard`, `glade_none`) ; le compte « Vous piocherez … » (spec 016) en tient compte.
- **Jeton de clairière** (C48) : ligne `glade.token` sous les points au début du tour, jusqu'à la première action.
- **Fenêtre de fin** (C49) : L'Aventurier compte ses points.

```
┌─ Panneau latéral (sur le donjon, jour) ──────────────────┐
│ Mouvement : 2 · Influence : 0                            │
│ [Entrer dans : Donjon]                                   │
│ Entrer fait perdre : 2 mouvement, 0 influence            │
└──────────────────────────────────────────────────────────┘
┌─ Confirmation ───────────────────────────────────────────┐
│ Combat contre un ennemi marron, tiré au hasard.          │
│ Règles de Nuit, sans vos unités.                         │
│ Récompense si vous gagnez : un sort (dé or ou noir) ou   │
│ un artefact. Pas de retour en arrière.                   │
│                         [Entrer]  [Annuler]              │
└──────────────────────────────────────────────────────────┘
```

Point purement visuel, laissé à la vérification à l'œil : la taille des deux jetons d'un Sombre Terrier, la place des lignes du
panneau et de l'écran de combat.

Vérification à l'œil, deux parties préparées :

- `donjon` : bac à sable, Jour, campagne 7 explorée, figurine sur le donjon, une unité, un jeton or, une main de quoi parer et
  attaquer ; entrer (confirmation), voir les règles de Nuit et l'absence des unités, gagner, voir le dé, finir le tour.
- `sombre-terrier` : une graine où une tuile royaume du Sombre Terrier est explorée, figurine dessus, un jeton déjà resté ; entrer,
  vaincre les deux, finir le tour sur la fenêtre des cristaux (un dé or) ; et la clairière : une variante qui finit son tour sur
  une clairière avec une Blessure en main.

### Raccourcis clavier

| Touche | Effet | Contexte |
|---|---|---|
| E | Entrer dans le site d'aventure (ouvre la confirmation) | Partie, quand le bouton est actif, hors mode choix |
| Échap | Annuler l'entrée | Partie, en confirmation de l'entrée |
| 1, 2, 3, 4 | Choisir rouge, bleu, blanc, vert pour le cristal | Choix de la couleur du cristal (avant celui d'un artefact) |

Reportées dans `docs/raccourcis.md` à l'implémentation (E est libre ; 1 à 4 servent déjà à d'autres choix, dans d'autres
contextes). Pas de touche pour le choix de la clairière : il se fait à la souris dans le mode fin du tour, comme les cartes à
défausser.

## Effets sur les specs déjà implémentées

- **030, 031, 032, 033, 042** : les ennemis du combat sont désignés par leur jeton, une case porte une liste de jetons (Q1) ; les
  tests qui visent un ennemi par sa case changent de forme sans changer de sens (à signaler dans la PR, étape 1 du plan).
- **041** : le combat sans unités sert aussi au donjon et au tombeau ; `EnemyTokenDiscarded` nomme le jeton.
- **043** : `SiteConquered` sert aux sites d'aventure ; `rampaging_enemies` exclut aussi les sites d'aventure ; le drapeau s'y
  dessine.
- **016, 034** : `EndTurn` gagne le choix de la clairière ; la Blessure jetée n'est pas un soin.
- **022, 025** : le paiement du mana lit la journée du combat (règles de Nuit) ; la clairière donne un jeton au début du tour.
- **041, 043, 050** : l'étape des récompenses commence par les cristaux aléatoires, avant l'artefact, le sort et l'action
  avancée.
- **063** : L'Aventurier compte les Boucliers des sites d'aventure.
- **Parcours** : une figurine qui commence un tour sur une clairière gagne un jeton (bac à sable : clairière en `(1, -3)`) ;
  points d'étape à relire.

## Hors périmètre

- Ruines (autels, jetons de ruines, ennemis des ruines) : spec 045.
- Effets qui varient selon le jour (compétences, sorts, actions avancées) : specs 051 à 054, qui liront la journée du combat.
- Labyrinthe, dédale, cimetière, nécropole (extensions), scénarios *Druid Nights* et *Dungeon Lords* (souterrains, entrées
  secrètes) : hors périmètre.
- Choisir librement l'ordre des récompenses (MKUE p. 11, 5) : ordre fixe, sans effet en solo.
- Unités en récompense (ruines) : spec 045.

## Plan de développement

Gros morceau : un changement technique du combat, quatre sites, deux nouveaux choix de fin de tour, la clairière, et l'écran.
Suivant l'issue privée 118, l'implémentation se fait en **deux fils** sur la même branche, selon le plan
[`plans/044-sites-d-aventure.md`](plans/044-sites-d-aventure.md) (étapes, colonne « Rouge attendu ») :

1. **Moteur** : ennemis par jeton (C1), C2 à C38, persistance (parties préparées `donjon` et `sombre-terrier`), docs du moteur.
2. **Écran** : libellés, C39 à C49, vérification à l'œil, docs de l'écran.

## Décisions (2026-11-05)

Questions de la rédaction, tranchées par Mathieu (« Validé ») : toutes les propositions sont retenues.

1. **Q1 — Ennemis désignés par leur jeton.** Le Sombre Terrier met deux jetons sur une case ; le combat désigne aujourd'hui un
   ennemi par sa case. Proposition : faire le changement maintenant, en première étape purement technique (une case porte une
   liste de jetons, les actions et événements de combat nomment le jeton), puisque les ruines (045) et les cités (062) en auront
   besoin aussi. Autre choix : laisser le Sombre Terrier à la spec 062, avec les cités.
2. **Q2 — Le jeton qui remplace un monstre vaincu du Sombre Terrier** : la carte le pose face cachée à la fin du combat, le
   compendium (*Miscellaneous* S15) le tire à l'entrée suivante. En solo, c'est la même chose ; proposition : le tirer à
   l'entrée suivante (pas de jeton face cachée sur un site d'aventure à gérer).
3. **Q3 — Quand lancer les dés** : proposition : le dé du donjon tout de suite à la conquête (texte de la carte), les cristaux
   aléatoires en fin de tour, un dé à la fois (MKUE p. 11, 5.a ; Compendium, *End of Turn* Q4).
4. **Q4 — Entrer : bouton et confirmation.** Proposition : un bouton dans le panneau (touche E) qui ouvre une confirmation, comme
   l'incendie du monastère, puisqu'un jeton est révélé tout de suite et qu'on ne peut plus revenir en arrière. Autre choix : sans
   confirmation, le bouton et son infobulle suffisant (comme l'assaut, Q2 de la spec 043).
5. **Q5 — Jeter la Blessure de la clairière** : proposition : dans le mode fin du tour, trois choix (main, défausse, ne pas jeter),
   par défaut la main s'il y a une Blessure, sinon la défausse ; c'est un champ de plus d'`EndTurn`, pas une décision à part.
6. **Q6 — Termes** : « site d'aventure », « entrer dans » (la carte FR dit « pénétrer dans »), « règles de Nuit », « dé de
   récompense », « cristal aléatoire », « jeter une Blessure » (FAQ p3 ; la carte FR dit « détruire »), « Vapeurs curatives » et
   « Imprégnation magique » (carte FR).
7. **Q7 — Une seule spec, deux fils** (moteur, puis écran) sur la même branche, clairière comprise.

## Questions ouvertes

Aucune.
