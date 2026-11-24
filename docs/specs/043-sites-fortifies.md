# 043 — Sites fortifiés

- Statut : validée le 2026-11-04 (propositions de Q1 à Q5 retenues par Mathieu, Q2 revue en revue de PR : assaut en un clic,
  drapeau à la place du jeton Bouclier dessiné)
- Jalon : J4
- Dépend de : 014 (déplacement, `MoveCost::Fortified`), 024 (retour en arrière), 030 (jetons d'ennemis, provocation), 031 à
  034 (combat, éléments, capacités, assommé), 035 (gloire, limite de main du niveau), 036 (trajet en un clic), 040 (interaction,
  réputation), 041 (jetons Bouclier, récompense de fin de tour, Le Conquérant), 042 (unités, recrutement par type de site), 050
  (offre de sorts) ; toutes implémentées
- Livret : MKUE p. 4 (2.g, limite de main à la nouvelle manche) ; p. 6 (6.b et 6.e) ; p. 7 (4.a ; 7.d) ; p. 8 (*Interacting
  with Locals* 1, 4.a et 4.d ; *Combat with Enemies* 1.a, 2.b et 2.c) ; p. 9 (*Ranged and Siege Attack Phase* 3.a ; *Assign
  Damage Phase* 5.c) ; p. 10 (*Attack Phase* 1.a ; *Combat Outcomes* 2.c et 3.d) ; p. 11 (*End of the Turn* 2, 5.c et 7.b).
  Cartes de site Keep et Mage Tower : `docs/regles/cartes/sites-et-cites.md` ; termes français : carte de site FR
  (`docs/regles/scans/fr/rule_cards/`). FAQ p3 (retraite, retraite forcée, mana pour un sort). Compendium, *Combat* Q1, S9, S13,
  S14, Q16, S19, S36 ; *Movement* S4, S8, S11, Q12 ; *Cities* S3 ; *Forced Withdrawal* S1 à S3 ; *End of Round* S6 ; *End of Turn* S8 ;
  *Influence* S5, S7 ; *Mana* S4 ; *Miscellaneous* S7, S10
- Crates touchées : arcanum-core (règles), arcanum-ui, arcanum-persistance (parties préparées), `assets/i18n/`

## Objectif

Les forteresses et les tours de mage cessent d'être des cases fermées. Chacune reçoit un **défenseur** face cachée quand sa tuile
est révélée, retourné de jour quand le héros passe à côté. Y entrer est un **assaut** : Réputation −1 et combat obligatoire
contre un défenseur **fortifié** (siège seulement à distance ; un défenseur déjà Fortifié ne se vise pas du tout à distance).
Vaincu, le site est **conquis** : jeton Bouclier, la case devient sûre, et la tour de mage donne un **sort** en récompense de fin de
tour. Repoussé, le héros **bat en retraite** sur sa case de départ. Une forteresse possédée augmente la limite de main et permet
d'y recruter ; une tour de mage conquise permet d'y recruter et d'y **acheter des sorts** (7 influence et un mana de leur couleur).

## Règles

### Défenseurs

- **Défenseur posé à la révélation** (cartes de site Keep et Mage Tower, *When Revealed*) : un jeton **gris** sur chaque
  forteresse, un jeton **violet** sur chaque tour de mage, tirés comme les autres jetons (remélange de la défausse, pile de secours,
  specs 031 et 033), **face cachée**. Même chose pour les sites des tuiles de la mise en place (aucun scénario actuel n'en a : testé
  sur un scénario de test, comme les monastères de la spec 041).
- **Révélé de jour à côté d'un héros** (cartes de site : « revealed during the Day if a player is adjacent to it » ; carte FR :
  « lorsqu'un joueur entre sur une case adjacente » ; Compendium, *Movement* S8) : le défenseur est retourné dès que, de jour, la
  figurine est sur une case voisine : après un pas, après la révélation d'une tuile dont le site touche la figurine, et au début
  d'une manche de Jour pour une figurine déjà voisine (Compendium, *End of Round* S6). Lecture retenue : un **état** (« est
  voisine ») plutôt que le seul pas (« entre ») ; voir Q4. De nuit, rien n'est retourné en passant (Compendium, *Miscellaneous*
  S10).
- **Révélé à l'assaut** (Compendium, *Combat* Q1.a et S14) : un défenseur encore face cachée est retourné quand le combat est
  engagé, **après** les défis éventuels (de nuit, on défie avant de voir le défenseur).
- **Une fois retourné, il le reste** (Compendium, *Movement* S8 : « anyone can take a look ») : la nuit suivante ne le recache pas.
- **Ce n'est pas un ennemi errant** (MKUE p. 7, 4.b et 4.c) : un défenseur ne bloque pas le passage autrement que par son site,
  n'est jamais provoqué et ne se défie pas (`challengeable`, `provoked` et les trajets de la spec 036 ne regardent que les ennemis
  errants).

### Assaut

- **Entrer, c'est assaillir** (MKUE p. 7, 4.a ; p. 8, *Combat with Enemies* 1.a ; Compendium, *Movement* S4) : un pas vers une
  forteresse ou une tour de mage **non conquise** est permis au coût de son terrain (la donnée de `MoveCost::Fortified`, spec 014).
  Il termine le déplacement et commence l'action du tour, obligatoire (MKUE p. 6, 6.b) : points de mouvement et d'influence non
  dépensés perdus (6.e), plus d'interaction, de repos, ni d'autre combat ce tour (Compendium, *Influence* S7).
- **Aussi au bout d'un trajet** (Q2, demande de Mathieu en revue) : l'assaut est le dernier pas d'un `MoveHero` (case voisine) ou
  d'un `FollowRoute` (spec 036) qui **finit** sur le site ; un trajet ne **traverse** jamais un site non conquis. Le trajet retenu
  vers le site suit les règles de la spec 036 (le moins cher, sans pas qui provoque, dernier pas compris). Pas de confirmation :
  l'écran prévient avant le clic (voir *Interface*) et le retour en arrière reprend l'assaut tant que rien n'est révélé.
- **Un trajet s'arrête sur une révélation** : de jour, si un pas du trajet retourne le défenseur du site visé (il était face
  cachée), le trajet s'arrête sur ce pas, à côté du site, sans assaillir : le joueur décide en voyant le défenseur (Compendium,
  *Movement* S13 : une information révélée ouvre une nouvelle décision). De nuit, ou défenseur déjà visible, le trajet va
  jusqu'au bout.
- **Réputation −1** (Compendium, *Combat* Q16) : au moment du pas, quelle que soit l'issue, même sur la case X ou depuis −7
  (Compendium, *Influence* S5 ; bornée à la piste, spec 040).
- **Ennemis errants** (MKUE p. 8, 2.b et 2.c ; Compendium, *Cities* S3) : le pas peut provoquer des ennemis errants ; depuis le
  site, le joueur peut aussi défier ceux qui lui sont voisins (moment « à engager » des specs 030 et 031). Ils combattent avec le
  défenseur, **non fortifiés**, et la conquête ne dépend pas d'eux.
- **Défenseur fortifié** (MKUE p. 9, 3.a ; carte de site) : le défenseur compte comme fortifié par son site. En phase d'attaque à
  distance, on ne le vise qu'avec du siège (comme un Fortifié, spec 032). S'il a lui-même la capacité Fortifié (Gardes, gris), il est
  **doublement fortifié** : aucune cible possible dans cette phase, ni seul ni dans un groupe (FAQ p3, Ver des Cryptes ; Compendium,
  *Combat* S13). En phase d'attaque, plus de fortification (MKUE p. 10, 1.a). Les ennemis errants du même combat ne gagnent rien.
- **Unités** : elles combattent normalement (aucune restriction, contrairement à l'incendie de la spec 041).
- **Gloire entière** du défenseur (Compendium, *Combat* S36 ; la demi-gloire ne vaut que pour la forteresse d'un autre joueur).

### Issue

- **Conquête** (MKUE p. 10, *Combat Outcomes* 2.c ; cartes de site) : le défenseur vaincu, à la fin du combat, un **jeton Bouclier**
  du joueur marque le site (`GameState::shields`, spec 041) : il est **conquis**. Le héros y reste (« you end your turn on the
  conquered space »). Forteresse : le joueur la **possède**. Tour de mage : une récompense **sort**, prise à la fin du tour.
- **Retraite** (MKUE p. 10, 3.d ; FAQ p3 ; Compendium, *Forced Withdrawal* S1 à S3, *Combat* S19) : défenseur non vaincu, à la fin
  du combat, la figurine revient sur la case d'où elle a assailli, **sans** coût, **sans** provoquer et **sans** Blessure. Le
  défenseur, retourné, reste sur le site. Les ennemis errants non vaincus restent où ils sont, avec ou sans conquête.
- **Après l'assaut**, pas plus qu'après un autre combat, ni déplacement ni interaction ce tour (Compendium, *Movement*
  S11) : interagir dans la forteresse conquise attend le tour suivant.
- **Case sécurisée et retraite forcée** (MKUE p. 7, 7.d ; p. 11, *End of the Turn* 2) : dans le jeu actuel, aucune fin de tour ne
  peut se faire hors d'une case sécurisée. La case d'où l'on assaille a été atteinte par un pas normal, donc elle l'est ; les
  effets qui font entrer sur une case infranchissable ou sauter des cases (Voyage souterrain, Ailes du vent…) sont `Unsupported`
  jusqu'au J5 ; un autre héros n'existe pas en solo. Décision proposée (Q1) : la retraite forcée et la notion de case sécurisée
  passent à la spec 052, avec le premier effet qui peut y mener.

### Site conquis

- **Plus d'assaut** : une forteresse possédée ou une tour de mage conquise est une case ordinaire de son terrain pour le
  déplacement (`MoveCost::Points`) : on y entre, on la traverse, on y finit son tour (cartes de site ; MKUE p. 7, 4.a ne vise que
  le site « unconquered »). Les cités restent fermées jusqu'à la spec 062.
- **Interaction** (MKUE p. 8, *Interacting with Locals* 1 ; cartes de site) : l'action `Interact` de la spec 040 est permise dans
  une forteresse **possédée** et dans une tour de mage **conquise**, avec le bonus ou le malus de réputation et le même refus sur
  la case X.
  - **Recruter** (MKUE p. 8, 4.a ; spec 042) : les unités dont un type est la forteresse, ou la tour de mage (Fusiliers, Golems,
    Mages…), selon le site.
  - **Acheter un sort** (MKUE p. 8, 4.d ; carte de la tour ; FAQ p3) : à la tour de mage seulement, n'importe quel sort de l'offre
    de sorts pour **7** influence **et un mana de sa couleur**, payé comme la couleur d'un effet renforcé (spec 022) : un dé de la
    Source (dans la limite des dés du tour, spec 025), un cristal ou un jeton de mana ; l'or tient lieu de n'importe quelle couleur
    de base, de jour seulement (Compendium, *Mana* S4). Le sort va sur le dessus du deck et l'offre de sorts est **complétée**
    aussitôt (Compendium, *Miscellaneous* S7). Plusieurs achats permis (MKUE p. 8, 5).
  - Ni soin ni action avancée : rien d'autre n'est imprimé sur ces cartes de site.
- **Limite de main** (carte Keep, *Your Keep* ; MKUE p. 11, 7.b ; p. 4, 2.g ; Compendium, *End of Turn* S8) : si la figurine est
  sur une forteresse possédée ou une case voisine, la limite de main est augmentée du nombre de forteresses possédées, pour la
  pioche de fin de tour et pour celle de la nouvelle manche. Deux forteresses voisines ne comptent pas double (le bonus est le
  nombre de forteresses, pas de voisines). Le seuil d'**assommé** reste la limite **non modifiée** (MKUE p. 9, 5.c ; spec 035).
- **Le Conquérant** (MKUE p. 15) : 2 par Bouclier sur forteresse ou tour de mage, déjà compté par la spec 041 (seuls les monastères
  en posaient).

### Récompense : un sort

- **Fin du tour** (MKUE p. 11, 5.c) : à l'étape des récompenses, après le cristal de la mine et l'artefact éventuel (spec 041),
  avant le choix d'action avancée d'un niveau pair (spec 050) : le joueur **choisit** un sort de l'offre de sorts, sur le dessus de
  son deck, et l'offre est complétée (déplacement des cartes, nouvelle carte en tête, spec 050). Puis la pioche de fin de tour :
  le sort est pioché aussitôt s'il reste de la place.
- **Offre vide** : la récompense est perdue (le livret ne dit rien ; la pioche de sorts compte 20 cartes, le cas est théorique).
- **Effet** : sorts en données, `Unsupported` jusqu'à la spec 054 ; ils se jouent de côté (spec 050, Q5).

### Exemple : assaut d'une forteresse

Bac à sable, Jour. La campagne 3 est explorée ; sa forteresse a reçu un jeton gris face cachée. La figurine arrive sur une case
voisine : le jeton est retourné, ce sont des **Gardes** (armure 7, Fortifié). Réputation 0.

```
Pas voisin de la forteresse   → HeroMoved ; EnemyTokenRevealed (Gardes)    — point de non-retour : le jeton est vu
  (le joueur voit « doublement fortifié » : aucun tir possible en phase d'attaque à distance)
Pas sur la forteresse         → HeroMoved ; AssaultStarted ; ReputationChanged −1 ; mouvement et influence perdus
  (moment « à engager » : défier un ennemi errant voisin, ou engager)
Engager                       → CombatStarted (Gardes, fortifiés deux fois)
  attaque à distance : Gardes pas une cible · parade : Parade 3 · attaque : Attaque 7 → EnemiesDefeated (gloire 3)
Fin du combat                 → CombatEnded ; SiteConquered : Bouclier sur la forteresse, le joueur la possède
Fin du tour (figurine dessus) → limite de main 5 + 1 (une forteresse possédée) : pioche jusqu'à 6
```

Variante, nuit, tour de mage : le jeton violet reste face cachée jusqu'à l'engagement. Les Mages de feu ne sont pas vaincus :

```
Pas sur la tour de mage       → HeroMoved ; AssaultStarted ; ReputationChanged −1
Engager                       → EnemyTokenRevealed (Mages de feu) — point de non-retour ; CombatStarted
… phase d'attaque : Attaque 3 sur armure 5, raté
Fin du combat                 → CombatEnded ; HeroWithdrew : retour sur la case de départ, sans coût ni Blessure
```

Schéma des cases (bac à sable, campagne 3 posée, `F` la forteresse, `·` ses voisines) : de jour, la figurine `H` qui entre sur
une case `·` retourne le défenseur ; qui passe ailleurs, non.

```
        ·   ·
      ·   F   ·
        ·   ·   H→
```

## Modèle

Termes du glossaire : Site fortifié, Forteresse, Tour de mage, Fortifié, Jeton d'ennemi, Face visible / face cachée, Jeton Bouclier,
Ennemi errant, Défier, Provoquer, Récompense, Offre de sorts, Sort, Limite de main, Le Conquérant. **Nouveaux termes** (ajoutés au
glossaire avec cette spec ; français de la carte de site FR et de la FAQ p3) :

| Livret | Français | Code |
|---|---|---|
| Assault (a fortified site) | Assaut ; assaillir (carte FR : « peut être assaillie ») | `AssaultStarted` |
| Defender (of a site) | Défenseur (carte FR) | jeton de `enemy_tokens` sur un site fortifié |
| Fortified twice | Doublement fortifié (FAQ p3) | — |
| Conquer, conquered / unconquered | Conquérir, conquis / non conquis (carte FR) | `SiteConquered` |
| (your) Keep | Forteresse possédée (carte FR : « vous appartient ») | Bouclier du joueur sur une forteresse |
| Withdraw (failed assault) | Battre en retraite, retraite (FAQ p3) | `HeroWithdrew` |
| Forced Withdrawal | Retraite forcée (FAQ p3) | — (spec 052, Q1) |
| Safe space | Case sécurisée (FAQ p3) | — (spec 052, Q1) |
| Buy a Spell | Acheter un sort | `Action::BuySpell`, `SpellBought` |
| Reveal (an enemy token) | Révéler, retourner | `EnemyTokenRevealed` |

La ligne « Assault | Assaut | — (specs 043, 062) » du glossaire est remplacée.

Données nouvelles :

- **Plateau de jeu** : les jetons d'ennemis **face cachée** (les défenseurs pas encore retournés), par case. Un site fortifié
  conquis est une forteresse ou une tour de mage qui porte un Bouclier (`GameState::shields`, spec 041). Le coût de déplacement
  dépend donc aussi des Boucliers : une fonction de l'état, plus seulement de la case.
- **État du joueur** : l'**assaut** en cours (site assailli et case de départ, pour la retraite), vidé à la fin du tour ; les
  **récompenses sort** du tour.
- **Contenu** : coût d'un sort (7 influence) en constante du moteur, comme les achats de la spec 041.
- **Vue du joueur** : pour un jeton face cachée, sa case et sa **couleur** seulement ; les Boucliers (déjà là), les sites conquis,
  l'assaut en cours, les récompenses sort, la limite de main avec son bonus.

## Actions et événements

| Action | Validation (refusée si…) | Événements produits |
|---|---|---|
| `Explore` (015, étendue) | inchangée | … `TileRevealed`, jetons errants, puis un `DefenderPlaced { token, hex }` par forteresse ou tour de mage de la tuile ; de jour, un `EnemyTokenRevealed` par défenseur voisin de la figurine |
| Mise en place (012) | — | un `DefenderPlaced` par site fortifié des tuiles posées (cités exclues, spec 062) |
| `MoveHero { to }` (014, étendue) | `to` site fortifié conquis : comme une case ordinaire ; `to` forteresse ou tour de mage non conquise : refusée si les points ne paient pas son terrain ; cité : toujours refusée (062) | vers un site non conquis : `HeroMoved`, `AssaultStarted { player, site, from }`, `ReputationChanged { change: −1 }`, `EnemiesProvoked` au besoin ; sinon `HeroMoved` comme avant ; de jour, après le pas, un `EnemyTokenRevealed` par défenseur face cachée voisin de l'arrivée |
| `FollowRoute` (036, étendue) | un trajet ne traverse pas un site non conquis, mais peut y finir (assaut) ; il traverse un site conquis | `HeroMoved` par pas et les `EnemyTokenRevealed` de chaque pas ; un pas qui retourne le défenseur du site visé arrête le trajet ; sinon le dernier pas sur le site produit les événements de l'assaut (`AssaultStarted`, `ReputationChanged`) |
| `Challenge` (031) | refusée sur un défenseur | — |
| `StartCombat` (031, étendue) | inchangée | défenseur face cachée : `EnemyTokenRevealed`, puis `CombatStarted` avec le défenseur et les ennemis errants |
| `DeclareAttack { targets }` (031, étendue) | en phase d'attaque à distance : refusée si une cible est doublement fortifiée | inchangés |
| `EndCombat` (031, étendue) | inchangée | `CombatEnded`, puis pour un assaut : `SiteConquered { player, hex }` si le défenseur est vaincu, sinon `HeroWithdrew { player, from, to }` |
| `Interact` (040, étendue) | permise aussi sur une forteresse possédée ou une tour de mage conquise | `InteractionStarted` (site `Keep` ou `MageTower`) |
| `RecruitUnit` (042) | type de site de l'unité : celui de l'interaction, forteresse ou tour de mage comprises | inchangés |
| `BuySpell { card, mana }` (nouvelle) | pas d'interaction en cours à une tour de mage ; carte absente de l'offre de sorts ; influence < 7 ; `mana` ne paie pas la couleur du sort (dé absent, épuisé ou déjà pris ce tour, cristal ou jeton absent, or de nuit) | `SpellBought { player, card, cost }`, les événements du mana payé (`DieTaken`, `CrystalUsed` ou `ManaTokenUsed`, spec 022 et 025), `OfferCardDealt { offer: Spells }` (offre complétée) |
| `EndTurn` (016, étendue) | inchangée | … cristal de la mine, artefacts tirés (041), puis, avec une récompense sort et une offre non vide, la pioche attend le choix ; limite de main augmentée près d'une forteresse possédée |
| `TakeSpell { card }` (nouvelle) | pas de récompense sort en attente ; carte absente de l'offre de sorts | `OfferCardGained { offer: Spells, card }` (dessus du deck, une récompense de moins), `OfferCardDealt { offer: Spells }` ; puis, plus de choix en attente, la suite de la fin du tour (choix d'action avancée, pioche) |
| `AnnounceEndOfRound` (023, nouvelle manche) | inchangée | une manche de Jour : un `EnemyTokenRevealed` par défenseur face cachée voisin de la figurine, après `RoundStarted` ; pioche de la nouvelle manche avec le bonus de forteresse |

Ordre des actions légales (recette de `architecture.md`) : les assauts sont des `MoveHero`, à leur place dans l'ordre des pas ;
choix de fin de tour : `TakeArtifact`, puis `TakeSpell` dans l'ordre de l'offre de sorts, puis `GainAdvancedAction` ; en
interaction : recrutements, `BuyAdvancedAction`, `BuySpell` (dans l'ordre de l'offre, puis des moyens de paiement comme pour un
effet renforcé), `BuyHealing`.

| Événement | Effet de `apply` | Révèle une information ? | Point de non-retour ? |
|---|---|---|---|
| `DefenderPlaced { token, hex }` | le jeton quitte sa pile et est posé face cachée sur `hex` | sa couleur seulement (`PlayerEvent` sans le jeton) | **oui**, comme `EnemyTokenPlaced` (le tirage a eu lieu) |
| `EnemyTokenRevealed { hex, token }` | le jeton de `hex` est face visible | **oui** | **oui** |
| `AssaultStarted { player, site, from }` | points de mouvement et d'influence à 0 ; déplacement terminé ; `site` parmi les ennemis à combattre ; assaut retenu (`from` pour la retraite) | non | non |
| `SiteConquered { player, hex }` | jeton Bouclier du joueur sur `hex` ; tour de mage : une récompense sort de plus | non | non |
| `HeroWithdrew { player, from, to }` | la figurine passe de `from` à `to`, sans coût | non | non |
| `SpellBought { player, card, cost }` | la carte quitte l'offre de sorts pour le dessus du deck ; influence − `cost` | non | non |
| `OfferCardGained`, `OfferCardDealt` (050) | inchangés ; un sort gagné en récompense décompte la récompense | inchangé | inchangé |
| `TurnEnded` (016) | vide aussi l'assaut et les récompenses sort non prises | inchangé | inchangé |

Aucun événement existant ne change de forme. Les parties préparées sont régénérées si leur construction change.

## Comportements attendus

Chaque cas devient au moins un test (nom indicatif entre parenthèses). C1 à C34 : moteur (arcanum-core) ; C35 à C45 : écran (arcanum-ui, sur
`game_screen`), un cas par point de la section « Interface » (issue privée 119).

Défenseurs :

- **C1** — Étant donné le bac à sable, quand l'exploration révèle la campagne 3, alors `TileRevealed` est suivi d'un
  `DefenderPlaced` d'un jeton gris sur la forteresse ; la vue du joueur donne la case et la couleur grise, pas l'ennemi
  (`revealed_keep_gets_a_face_down_grey_defender`) — carte Keep, *When Revealed*.
- **C2** — De même, la campagne 4 met un jeton violet face cachée sur sa tour de mage (`revealed_mage_tower_gets_a_violet_defender`)
  — carte Mage Tower.
- **C3** — Un scénario de test dont une tuile de la mise en place porte une forteresse : son défenseur est posé à la mise en place
  (`setup_fortified_sites_get_defenders`) — cartes de site.
- **C4** — De jour, un pas qui arrive à côté d'une forteresse dont le défenseur est face cachée produit `EnemyTokenRevealed` ; la vue
  donne l'ennemi (`day_step_next_to_keep_reveals_defender`) — carte Keep ; Compendium, *Movement* S8.
- **C5** — De nuit, le même pas ne retourne rien ; le jeton reste face cachée (`night_step_reveals_nothing`) — Compendium,
  *Miscellaneous* S10.
- **C6** — De jour, une exploration depuis une case voisine du site de la tuile révélée retourne son défenseur aussitôt
  (`day_exploration_next_to_site_reveals_defender`) — carte Keep (lecture « est voisine », Q4).
- **C7** — Une manche de Jour commence avec la figurine voisine d'une forteresse au défenseur face cachée : `EnemyTokenRevealed`
  après `RoundStarted` (`day_round_reveals_adjacent_defenders`) — Compendium, *End of Round* S6.
- **C8** — Un défenseur retourné de jour reste face visible la nuit suivante (`revealed_defender_stays_face_up`) — Compendium,
  *Movement* S8.
- **C9** — Un défenseur n'est ni provoqué par un pas qui longe son site, ni proposé à `Challenge`, et un trajet passe à côté sans
  détour (`defenders_are_not_rampaging`) — MKUE p. 7, 4.c.
- **C10** — Revenir en arrière ne passe pas avant un `EnemyTokenRevealed` ni un `DefenderPlaced` (`revealing_a_defender_blocks_undo`)
  — spec 024.

Assaut :

- **C11** — Figurine voisine d'une forteresse non conquise, assez de points pour son terrain : `MoveHero` vers elle est légal ;
  quand le joueur le joue, alors `HeroMoved`, `AssaultStarted`, `ReputationChanged { change: −1 }` ; points de mouvement et
  d'influence à 0, plus de pas, d'exploration, d'interaction ni de repos ; seules restent les actions du moment « à engager »
  (`stepping_into_a_keep_is_an_assault`) — MKUE p. 7, 4.a ; p. 8, 1.a ; p. 6, 6.b et 6.e.
- **C12** — Pas assez de points pour le terrain de la forteresse : le pas n'est pas légal ; une cité voisine : jamais
  (`assault_costs_the_terrain_and_cities_stay_closed`) — Compendium, *Movement* S4 ; spec 062.
- **C13** — Forteresse à trois cases, défenseur visible : `FollowRoute` vers elle est légal ; ses pas sont suivis de
  `AssaultStarted` et `ReputationChanged { change: −1 }` ; aucun trajet ne traverse un site non conquis pour aller plus loin
  (`route_can_end_in_an_assault`) — Q2 ; spec 036.
- **C13b** — De jour, défenseur face cachée : le trajet vers la forteresse s'arrête sur le pas qui le retourne, sans assaut ; de
  nuit, le même trajet va jusqu'à l'assaut (`route_stops_when_defender_is_revealed`) — Compendium, *Movement* S13.
- **C14** — Réputation sur X : l'assaut est légal, la réputation reste sur X ; réputation −7 : elle reste à −7
  (`assault_works_on_x`) — Compendium, *Influence* S5 ; spec 040.
- **C15** — Engager l'assaut : `CombatStarted` avec le défenseur, compté fortifié ; une Attaque à distance ne le vise pas en phase
  d'attaque à distance, une Attaque de siège oui (`defender_is_fortified_by_its_site`) — MKUE p. 9, 3.a.
- **C16** — Défenseur Gardes (capacité Fortifié) : en phase d'attaque à distance, il n'est pas une cible, même pour le siège, ni
  seul ni avec un autre ennemi ; en phase d'attaque, il l'est (`fortified_defender_is_fortified_twice`) — MKUE p. 9, 3.a ; p. 10,
  1.a ; Compendium, *Combat* S13.
- **C17** — De nuit, défenseur face cachée : après l'assaut, `Challenge` d'un ennemi errant voisin du site est légal et le jeton
  reste caché ; `StartCombat` produit `EnemyTokenRevealed` puis `CombatStarted` (`night_assault_reveals_defender_at_engagement`)
  — Compendium, *Combat* Q1.a et S14.
- **C18** — Le pas de l'assaut provoque un ennemi errant : il combat avec le défenseur, non fortifié (Attaque à distance permise
  contre lui) (`provoked_rampaging_enemies_join_unfortified`) — MKUE p. 8, 2.b.
- **C19** — Les unités s'activent et reçoivent des dégâts pendant un assaut (`units_fight_in_assaults`) — spec 042.
- **C20** — Défenseur vaincu : sa gloire entière (`defender_gives_full_fame`) — Compendium, *Combat* S36.

Issue :

- **C21** — Défenseur vaincu, combat fini : `CombatEnded`, puis `SiteConquered` ; Bouclier sur la forteresse, figurine dessus ; ni
  pas ni interaction ensuite ce tour (`victory_conquers_the_keep`) — MKUE p. 10, 2.c ; Compendium, *Movement* S11.
- **C22** — Défenseur vaincu, ennemi errant défié non vaincu : conquête quand même ; l'ennemi errant reste sur sa case
  (`conquest_ignores_rampaging_enemies`) — MKUE p. 10, 2.c, note ; Compendium, *Cities* S3.
- **C23** — Défenseur non vaincu : `HeroWithdrew` vers la case de départ ; aucun point dépensé, aucun ennemi provoqué, aucune
  Blessure ajoutée ; le défenseur, face visible, reste sur le site (`failed_assault_withdraws`) — MKUE p. 10, 3.d ; Compendium,
  *Forced Withdrawal* S2 et S3.
- **C24** — Retraite depuis une case dont un ennemi errant est voisin de la case de départ : aucun `EnemiesProvoked`
  (`withdrawal_provokes_nobody`) — Compendium, *Combat* S19.

Site conquis :

- **C25** — Forteresse possédée : la figurine y entre au coût de son terrain sans assaut, la traverse et en ressort ; un trajet la
  traverse (`conquered_site_is_an_ordinary_space`) — MKUE p. 7, 4.a.
- **C26** — Sur sa forteresse : `Interact` est légal ; `RecruitUnit` propose les unités de type forteresse de l'offre, pas celles
  de village seulement (`own_keep_recruits_keep_units`) — carte Keep ; MKUE p. 8, 4.a.
- **C27** — Sur une tour de mage conquise, influence 7, un cristal rouge, Éclair de feu (rouge) dans l'offre de sorts : `BuySpell`
  payé par le cristal produit `SpellBought`, `CrystalUsed`, `OfferCardDealt` ; le sort est sur le dessus du deck, l'offre a de
  nouveau trois cartes, influence 0 (`mage_tower_sells_spells`) — MKUE p. 8, 4.d ; Compendium, *Miscellaneous* S7.
- **C28** — Achat de sort refusé sans mana de la couleur, avec un dé déjà pris ce tour, avec de l'or la nuit, avec 6 influence ;
  permis avec un dé or de jour (`spell_purchase_needs_matching_mana`) — FAQ p3 ; Compendium, *Mana* S4 ; spec 022.
- **C29** — Fin du tour sur ou à côté d'une forteresse possédée, une seule possédée : pioche jusqu'à la limite + 1 ; deux possédées,
  figurine voisine des deux : + 2, pas + 4 ; loin de toute forteresse : limite du niveau (`owned_keeps_raise_hand_limit`) — carte
  Keep ; Compendium, *End of Turn* S8.
- **C30** — Nouvelle manche, figurine sur sa forteresse : la pioche de la manche suit la limite augmentée ; le seuil d'assommé, lui,
  reste la limite du niveau (`keep_bonus_at_new_round_not_for_knock_out`) — MKUE p. 4, 2.g ; p. 9, 5.c.

Récompense, décompte, sauvegarde, vue :

- **C31** — Fin du tour après la conquête d'une tour de mage : la pioche attend ; `TakeSpell` est proposée pour chaque sort de
  l'offre ; en prendre un le met sur le dessus du deck, complète l'offre, puis la pioche le prend
  (`mage_tower_reward_is_a_spell_from_the_offer`) — MKUE p. 11, 5.c.
- **C32** — Le même tour, la gloire franchit un niveau pair : le choix du sort vient d'abord, puis celui de l'action avancée
  (`spell_reward_comes_before_level_up`) — MKUE p. 11, 5 et 6.
- **C33** — Fin de partie avec une forteresse et une tour de mage conquises : Le Conquérant 4 (`conqueror_counts_keeps_and_towers`)
  — MKUE p. 15. (Déjà compté par la spec 041 : ce test peut passer du premier coup, à signaler dans la PR.)
- **C34** — Une partie sauvegardée avec un défenseur face cachée, un retourné, une forteresse possédée et une récompense sort en
  attente se recharge à l'identique ; un assaut en cours (moment « à engager ») aussi (`fortified_sites_survive_save_and_load`) —
  spec 024.

Écran (arcanum-ui, sur `game_screen`) :

- **C35** — Plateau : un défenseur face cachée est dessiné comme un dos de jeton de sa couleur avec « ? » ; retourné, comme les
  autres jetons d'ennemis (`board_draws_face_down_defenders`).
- **C36** — Infobulle d'un site fortifié : « Forteresse · défenseur gris, face cachée », ou le nom du défenseur une fois retourné ;
  « Forteresse conquise » ; sur une cible d'assaut, en tête et en rouge, « ⚔ Assaut : Réputation −1, combat obligatoire », puis la
  fortification du défenseur et le trajet, à la place de « l'assaut n'est pas encore disponible » (`hover_describes_fortified_sites`).
- **C37** — Une forteresse ou une tour de mage qu'on peut assaillir (voisine ou au bout d'un trajet) porte, dès l'affichage, une
  pastille rouge « ⚔ » avec le coût, à la place de la pastille crème des cases atteignables ; au survol, le trajet est tracé en rouge
  (`board_marks_assault_targets`).
- **C38** — Un clic sur une cible d'assaut joue l'assaut aussitôt, sans confirmation (le pas, ou le trajet) ; « Revenir en
  arrière » le reprend tant que rien n'est révélé (`assault_in_one_click`).
- **C38b** — Site conquis (forteresse, tour de mage) ou monastère incendié : la case porte un **drapeau** aux couleurs du héros à la
  place du jeton Bouclier de la spec 041 (`conquered_sites_show_a_flag`).
- **C39** — Écran de combat d'un assaut : le défenseur porte « fortifié (site) », ou « doublement fortifié : pas de cible à
  distance » ; un ennemi errant du même combat, rien (`combat_marks_site_fortification`).
- **C40** — Après une retraite, une ligne du panneau dit « Assaut repoussé : retour sur la case de départ » jusqu'à la fin du tour
  (`panel_announces_withdrawal`).
- **C41** — En interaction à une tour de mage conquise : un bouton « Acheter : Éclair de feu (7 + mana rouge) » par sort de l'offre,
  grisé si l'influence ou le mana manque ; le choix du mana suit celui d'un effet renforcé (`panel_offers_spells_at_mage_tower`).
- **C42** — En interaction dans sa forteresse : le bouton « Interagir avec les habitants » existe sur la case, et la liste
  « Recruter » montre les unités de type forteresse (`keep_interaction_lists_keep_units`).
- **C43** — Fin du tour avec une récompense sort : la fenêtre « Récompense : prenez un sort » montre les sorts de l'offre (nom,
  texte) et un bouton « Prendre » chacun (`spell_choice_window`).
- **C44** — Panneau : près d'une forteresse possédée, « Limite de main : 6 (+1 forteresse) » ; ailleurs, la ligne habituelle
  (`panel_shows_keep_hand_limit`).
- **C45** — Panneau : la récompense du tour s'annonce « Récompense en fin de tour : 1 sort » (`panel_announces_spell_reward`).

## Cas limites et refus

- **L1** — Assaillir avec une main de seules Blessures sans carte jouée : impossible, faute de points de mouvement (aucun pas n'est
  gratuit).
- **L2** — Héros assommé pendant l'assaut : le combat continue (spec 034) ; vaincre quand même le défenseur conquiert le site.
- **L3** — Défenseur Illusionnistes (violet, Invocation) : la créature invoquée n'est pas le défenseur ; seule la défaite du jeton
  violet conquiert la tour (comme L5 de la spec 041).
- **L4** — Piles grise ou violette et leurs défausses vides à la révélation : la pile de secours de la spec 033 fournit le jeton ;
  plus aucun jeton nulle part : le site n'a pas de défenseur, et y entrer le conquiert sans combat (Réputation −1 quand même).
  Théorique.
- **L5** — Assaut au dernier tour de la partie (spec 063) ou après l'annonce de fin de manche : permis ; la récompense sort est prise
  à la fin du tour, avant le décompte.
- **L6** — Acheter deux sorts dans la même interaction : permis, l'offre est complétée entre les deux (Compendium, *Miscellaneous*
  S7).
- **L7** — Un défenseur retourné vu depuis une autre case, puis un assaut de jour : aucun nouvel `EnemyTokenRevealed` à
  l'engagement.
- **L8** — Recruter dans une forteresse sans jeton de commandement libre : renvoi d'une unité, comme ailleurs (spec 042).

## Information cachée

Un défenseur face cachée est connu du moteur (tiré à la révélation de la tuile, comme l'ordre des piles) mais pas du joueur : la vue
et `DefenderPlaced` filtré ne donnent que sa couleur, jusqu'à `EnemyTokenRevealed`. L'écran ne montre que la vue (le journal des
événements filtrés aussi). La sauvegarde garde le jeton, comme elle garde l'ordre des piles.

## Libellés (i18n)

| Clé | Paramètres | fr | en |
|---|---|---|---|
| `enemy.face_down` | `color` | défenseur {color}, face cachée | {color} defender, face down |
| `enemy.pile_color.<couleur>` (ajoutée à l'implémentation : `enemy.color.*` nomme le groupe, « gardes de forteresse ») | — | gris, violet, vert, marron, rouge, blanc | grey, violet, green, brown, red, white |
| `site.conquered.keep` | — | Forteresse conquise | Conquered keep |
| `site.conquered.mage_tower` | — | Tour de mage conquise | Conquered mage tower |
| `board.hover.assault` | — | ⚔ Assaut : Réputation −1, combat obligatoire | ⚔ Assault: Reputation −1, mandatory combat |
| `board.hover.defender_fortified` | — | Défenseur fortifié : siège seulement à distance | Fortified defender: siege only at range |
| `board.hover.defender_fortified_twice` | — | Défenseur doublement fortifié : aucun tir à distance | Defender fortified twice: no ranged attack |
| `board.hover.fortified` (changée) | — | Cité : l'assaut n'est pas encore disponible | City: assault not available yet |
| `combat.enemy.site_fortified` | — | fortifié (site) | fortified (site) |
| `combat.enemy.fortified_twice` | — | doublement fortifié : pas de cible à distance | fortified twice: no ranged target |
| `assault.withdrew` | — | Assaut repoussé : retour sur la case de départ | Assault repelled: back to the starting space |
| `purchase.spell` | `name`, `cost`, `color` | Acheter : {name} ({cost} + mana {color}) | Buy: {name} ({cost} + {color} mana) |
| `reward.choose_spell` | — | Récompense : prenez un sort | Reward: take one Spell |
| `reward.pending_spell` | `count` | Récompense en fin de tour : {count} sort(s) | Reward at end of turn: {count} Spell(s) |
| `level.hand_limit_keep` | `limit`, `bonus` | Limite de main : {limit} (+{bonus} forteresse) | Hand limit: {limit} (+{bonus} keep) |

`reward.take` (spec 041) sert aussi au sort. Les noms de couleur de mana et de jeton d'ennemi existent déjà (specs 022 et 030).

## Interface

- **Défenseurs sur le plateau** (C35) : dos de jeton de la couleur de la pile, avec « ? », dessiné par nous comme les autres jetons ;
  retourné, le jeton habituel (spec 030).
- **Infobulle** (C36) : ligne du site et de son défenseur (`enemy.face_down` ou nom de l'ennemi), `site.conquered.*` pour un site
  conquis, `board.hover.assault` pour une cible d'assaut ; `board.hover.fortified` ne vise plus que les cités.
- **Cibles d'assaut, sans confirmation** (C37, C38 ; Q2) : le joueur est prévenu à trois niveaux, sans clic de plus. (1) Avant
  tout survol, la case du site porte une pastille rouge « ⚔ coût » au lieu de la pastille crème des cases atteignables (spec 017) :
  on voit d'un coup d'œil quelles cases déclenchent un combat. (2) Au survol, le trajet est tracé en rouge jusqu'au site, et
  l'infobulle (spec 037) commence par `board.hover.assault` en rouge et en gras, puis le défenseur et sa fortification. (3) Après
  le clic, « Revenir en arrière » (spec 024) reprend l'assaut tant qu'aucun jeton n'a été retourné. Maquette :
  `maquettes/spec-043-drapeau-et-assaut.png` (dossier du projet).
- **Drapeau** (C38b, demande de Mathieu en revue) : un site conquis ou un monastère incendié porte un drapeau aux couleurs du héros
  (`hero_color`), planté en haut à droite du médaillon du site, dessiné par nous en SVG (un par héros, `assets/images/flag/`) ; il
  remplace le jeton Bouclier de la spec 041 sur le plateau. Le terme de règle reste « jeton Bouclier » (`GameState::shields`).
  Taille : il reste dans la case du site, sans déborder sur la voisine (maquette validée par Mathieu, 2026-11-04, même
  fichier).
- **Combat** (C39) : marque `combat.enemy.site_fortified` ou `combat.enemy.fortified_twice` sur la carte du défenseur, à côté de
  « fortifié : siège seulement » (spec 031).
- **Retraite** (C40) : ligne `assault.withdrew` sous les points, jusqu'à la fin du tour.
- **Interaction** (C41, C42) : sous « Recruter », `purchase.spell` par sort de l'offre, grisé si trop cher ou sans mana ; un clic
  ouvre le choix du mana des effets renforcés (spec 025). Le bouton « Interagir » apparaît sur une forteresse possédée et une tour
  conquise.
- **Récompense** (C43, C45) : fenêtre comme le choix d'artefact (spec 041), `reward.choose_spell`, les trois sorts (nom, texte
  `Unsupported` de la spec 050), `reward.take` sous chacun ; `reward.pending_spell` sous les points.
- **Limite de main** (C44) : `level.hand_limit_keep` à la place de la ligne de limite de main quand le bonus s'applique.

Point purement visuel, laissé à la vérification à l'œil : le dessin du dos de jeton (formes, lisibilité aux
zooms), la pastille rouge et le trait rouge du trajet, le dessin du drapeau.

```
┌─ Panneau latéral (tour de mage conquise) ────────────────┐
│ Interaction : Tour de mage                               │
│ Mouvement : 0 · Influence : 8                            │
│ Recruter : Golems (7)                                    │
│ [Acheter : Éclair de feu (7 + mana rouge)]               │
│ [Acheter : Tempête de neige (7 + mana bleu)]   (grisé)   │
│ Limite de main : 6 (+1 forteresse)                       │
└──────────────────────────────────────────────────────────┘
```

Vérification à l'œil, deux parties préparées :

- `forteresse` : bac à sable, Jour, campagnes 3 et 4 explorées, figurine à deux pas de la forteresse (défenseur face cachée), main
  avec de quoi marcher, parer et attaquer (siège compris) ; avancer à côté (jeton retourné), assaillir, gagner ou battre en
  retraite.
- `tour-conquise` : forteresse et tour de mage conquises, figurine sur la tour, influence en main, un cristal ; interagir,
  acheter un sort, recruter, finir le tour (limite de main augmentée si voisine de la forteresse) ; et la fenêtre du sort en
  récompense, depuis une variante qui vient de conquérir la tour.

### Raccourcis clavier

| Touche | Effet | Contexte |
|---|---|---|
| 1, 2, 3 | Prendre le 1er, 2e, 3e sort de l'offre | Choix du sort de la récompense |

Pas de touche pour assaillir ni acheter un sort : l'assaut part d'un clic sur la case, et les achats changent d'un site à l'autre
(spec 042). Reportées dans `docs/raccourcis.md` à l'implémentation (1 à 3 servent déjà à d'autres choix, dans d'autres contextes).

## Effets sur les specs déjà implémentées

- **012, 015** : la mise en place et l'exploration posent les défenseurs. Le bac à sable révèle une forteresse avec la campagne 3 et
  une tour de mage avec la campagne 4 : un jeton gris et un violet quittent leurs piles. Les parcours qui les explorent tirent des
  jetons de plus ; les piles ont chacune leur générateur (spec 038), les ennemis errants ne changent donc pas. Points d'étape à
  relire.
- **014** : `MoveCost::Fortified` ne ferme plus que les cités ; L5 (`entering_a_fortified_site_is_not_available_yet`) change de
  sens pour la forteresse et la tour de mage (assaut), à signaler et justifier dans la PR ; `board.hover.fortified` ne vise plus
  que les cités.
- **017, 036** : les trajets traversent les sites conquis, jamais les autres ; ils peuvent finir sur un site non conquis (assaut) et
  s'arrêtent quand un pas retourne le défenseur visé ; un défenseur ne provoque pas.
- **041** (écran) : le jeton Bouclier dessiné sur un monastère incendié devient le drapeau du héros.
- **024** : `Event::blocks_undo` gagne `DefenderPlaced` et `EnemyTokenRevealed` (oui) et les autres nouveaux événements (non). Un pas
  de jour à côté d'un défenseur caché ne se reprend plus.
- **030, 031** : `enemy_tokens` porte aussi les défenseurs ; provocation, défis et trajets ne regardent que les ennemis errants. Un
  combat peut contenir un ennemi fortifié par son site, ou deux fois.
- **035** : la pioche de fin de tour et de nouvelle manche ajoute le bonus de forteresse ; le seuil d'assommé ne change pas.
- **040, 042** : `Interact` et `RecruitUnit` s'étendent à la forteresse possédée et à la tour de mage conquise.
- **041** : la récompense de fin de tour gagne le sort, entre l'artefact et l'action avancée.
- **063** : Le Conquérant compte enfin des forteresses et des tours de mage (rien à coder).
- **Joueur de test `FirstChoice`** : il ne doit pas assaillir (un `MoveHero` vers un site non conquis est écarté comme `Challenge`),
  pour que les parcours ne changent pas.

## Hors périmètre

- Retraite forcée et case sécurisée : spec 052, avec le premier effet qui peut y mener (Q1).
- Cités (défenseurs, assaut, bonus des défenseurs, chef de la cité, limite de main près d'une cité) : spec 062.
- Forteresse d'un autre joueur (demi-gloire), combat entre joueurs, *Conquer and Hold* : multijoueur, hors périmètre.
- Murs (*Lost Legion*), capacités Non fortifié et autres capacités des extensions : hors périmètre.
- Sorts qui ignorent la fortification (Démolition) et autres effets de sorts : spec 054.

## Plan de développement

Gros morceau : défenseurs cachés, assaut, issue, sites conquis, récompense, et un écran à chaque fois. Suivant l'issue privée 118,
l'implémentation se fait en **deux fils** sur la même branche, selon le plan [`plans/043-sites-fortifies.md`](plans/043-sites-fortifies.md)
(étapes, colonne « Rouge attendu ») :

1. **Moteur** : C1 à C34, contenu, persistance (parties préparées `forteresse` et `tour-conquise`), docs du moteur.
2. **Écran** : libellés, C35 à C45, vérification à l'œil, docs de l'écran.

## Décisions (2026-11-04)

Questions de la rédaction, tranchées par Mathieu (« parfait, validé pour tout ; réduis juste un peu la taille du drapeau ») :

1. **Q1** — Retraite forcée et case sécurisée reportées à la spec 052, avec le premier effet qui permet de finir hors d'une case
   sécurisée ; `PLAN.md` corrigé.
2. **Q2** — Assaut aussi au bout d'un trajet en un clic, sans confirmation : pastille rouge « ⚔ coût », trajet rouge et infobulle au
   survol, retour en arrière ensuite ; le trajet s'arrête quand un de ses pas retourne le défenseur visé (C13, C13b, C37, C38).
   Demandé en revue de PR, avec le **drapeau** aux couleurs du héros à la place du jeton Bouclier dessiné, monastère incendié
   compris (C38b) ; maquette `maquettes/spec-043-drapeau-et-assaut.png` validée, drapeau réduit pour rester dans sa case.
3. **Q3** — Une seule spec, deux fils sur la même branche (moteur, puis écran), interaction dans les sites conquis comprise.
4. **Q4** — Défenseur retourné de jour dès que la figurine est voisine (pas, exploration, début d'une manche de Jour).
5. **Q5** — Termes : « défenseur », « assaillir », « conquis », « battre en retraite », « case sécurisée », « doublement fortifié »,
   « Acheter : <sort> (7 + mana <couleur>) », « Récompense : prenez un sort ».

Décision prise à l'implémentation de l'écran (2026-11-05) :

6. **Q6** — « ⚔ » n'est dans aucune police par défaut d'egui : `egui` (même version qu'eframe) est déclaré en dépendance
   directe d'arcanum-ui pour activer `monochrome_emoji_fonts` ; libellés inchangés (`default_fonts_draw_crossed_swords`).

## Questions ouvertes

Aucune.
