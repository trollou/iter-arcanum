# 059 — Parcours du jalon 5

- Statut : implémentée (révision du départ, issue privée 318, validée le 2026-11-17 : Q7 et Q9 retenues, Q8 reportée en issue privée 322 ;
  implémentée dans la PR privée 320) ; version d'origine validée le 2026-11-14 et implémentée (PR privée 304)
- Jalon : J5
- Dépend de : 051, 058 (compétences), 050 (cartes gagnées), 052, 053, 056 (moteur d'effets, actions avancées), 054, 057
  (sorts), 043 (tour de mage : assaut, récompense sort, achat), 063 (Première reconnaissance) ; 049 (parcours du J4 :
  explorateur scripté, `take_pending_reward`, recomptage du score) ; ADR 0006 (stratégie de tests), ADR 0007 (un générateur
  par usage)
- Livret : MKUE p. 11 (*End of the Turn* 2, 5.c et 6.b) ; p. 7 (7.b, 7.d : lacs franchissables, case sécurisée) ; FAQ p3
  (retraite forcée) ; Compendium, *Forced Withdrawal* S1 à S4, *Frost Bridge* S1, S2 ; p. 5 (*Using Skills* 2 à 4 ; *"Gain" Effects* 3) ; p. 4 (*Deed Cards*
  3.a et 3.b) ; p. 8 (*Interacting with Locals* 4.d ; *Combat with Enemies* 1.a) ; p. 7 (4.a) ; p. 10 (*Combat Outcomes*
  2.c) ; p. 15 (*The Greatest Knowledge*, *The Greatest Conqueror*) ; p. 18 (*First Reconnaissance*) ; carte de site Mage
  Tower (`docs/regles/cartes/sites-et-cites.md`)
- Crates touchées : arcanum-core (tests seulement : `crates/arcanum-core/tests/parcours_j5.rs`, et `crates/arcanum-core/tests/support/` si
  une aide y monte)

## Objectif

Vérifier que les specs du J5 **s'enchaînent dans une vraie partie**, comme le promet la ligne du plan : *monter de niveau,
choisir compétence et action avancée, les jouer ; acheter un sort et le lancer*. Rien ne change pour le joueur : la spec
n'ajoute que des tests.

Le parcours part d'une **mise en place arrangée** (ADR 0006, issue privée 318) de **Première reconnaissance**, avec Arythea, sur la
clairière magique de la campagne 2. Il explore jusqu'à la **tour de mage** de
la tuile de campagne 4, la conquiert, et la fin de ce tour donne d'un coup les trois cartes du jalon : le **sort** de la
récompense, puis la **compétence** et l'**action avancée** du niveau 2. Le tour suivant, les trois servent pour leur effet. Plus
tard, depuis la tour conquise, le joueur **achète** un second sort et le lance. La partie va ensuite jusqu'au **score**, où Le
Savant compte les deux sorts (Q3). Un **épisode posé** à part, sur le village de la campagne 1, joue **Pont de glace** renforcé
jusqu'au **lac** ; le héros y **défie** des orques, **échoue**, et la fin du tour sur le lac le contraint à la **retraite forcée**
(Mathieu, 2026-11-14 : dur à jouer et complexe, donc à bien tester).

Il attrape ce que les tests isolés ne voient pas : trois choix de fin de tour qui ne s'enchaînent pas (sort, compétence, action
avancée), des cartes gagnées qui ne reviennent pas en main dans l'ordre, une compétence « une fois par manche » qui ne se
retourne pas à la nouvelle manche, un sort payé du mauvais mana, un achat qui ne complète pas l'offre, une retraite forcée qui oublie le combat raté ou
qui ne suit pas le trajet du tour, un sort oublié au
décompte, une sauvegarde qui ne rejoue pas les compétences.

## Règles

Aucune règle nouvelle : chaque point d'étape renvoie à la règle qu'il vérifie et à la spec qui l'implémente.

- **Défenseur de la tour de mage** (carte de site Mage Tower) : jeton violet face cachée posé à la révélation de la tuile,
  retourné de jour quand la figurine arrive à côté : spec 043.
- **Assaut et conquête** (MKUE p. 7, 4.a ; p. 8, *Combat with Enemies* 1.a ; p. 10, *Combat Outcomes* 2.c) : Réputation −1,
  combat contre le défenseur fortifié ; vaincu, Bouclier et une récompense sort : spec 043.
- **Ordre des choix de fin de tour** (MKUE p. 11, 5.c puis 6.b) : le sort de la récompense, puis la compétence, puis l'action
  avancée ; chacun va sur le dessus du deck (p. 5, *"Gain" Effects* 3), puis la pioche : specs 043, 050, 051 (ordre codé dans
  les actions légales de fin de tour).
- **Compétence** (MKUE p. 5, *Using Skills* 2 à 4) : deux révélées, une gardée, l'autre dans l'offre commune ; une fois par
  tour (de nouveau prête au tour suivant) ou une fois par manche (retournée jusqu'à la nouvelle manche) : specs 051, 058.
- **Action avancée** (MKUE p. 4, *Deed Cards* 3.a) : jouée pour son effet de base ou renforcé ; toutes les cartes du jeu de base
  ont leur effet depuis la 056 (aucun `Unsupported` dans `advanced_actions.ron`).
- **Sort** (MKUE p. 4, *Deed Cards* 3.b) : effet de base payé d'un mana de sa couleur, effet renforcé de nuit seulement avec un
  mana noir en plus : specs 054, 057.
- **Acheter un sort** (MKUE p. 8, *Interacting with Locals* 4.d) : à la tour conquise, en interaction, 7 influence et un mana de
  sa couleur ; le sort va sur le dessus du deck, l'offre est complétée : spec 043. Un seul assaut (Réputation −1) ne change pas
  le bonus d'influence (`reputation.ron` : `Bonus(0)` de 0 à −1).
- **Lac franchissable** (MKUE p. 7, 7.b ; carte Frost Bridge ; Compendium, *Frost Bridge* S1, S2) : Pont de glace renforcé
  (mana bleu) : Mouvement 4, lacs et marais à 1 ce tour ; on peut s'arrêter sur le lac et défier depuis lui : spec 052.
- **Combat raté contre des errants** (MKUE p. 10, *Combat Outcomes*) : les ennemis non vaincus restent sur leur case, le héros
  reste où il est : spec 031.
- **Retraite forcée** (MKUE p. 11, *End of the Turn* 2 ; p. 7, 7.d ; FAQ p3 ; Compendium, *Forced Withdrawal* S1 à S4) : à la
  fin du tour, en premier, hors d'une case sécurisée, retour le long du trajet du tour jusqu'à la première case sécurisée, une
  Blessure en main par case, sans provoquer, sans assommer ; le reste de la fin du tour s'évalue après : spec 052.
- **Décompte final** (MKUE p. 15, *The Greatest Knowledge* : 2 par sort, 1 par action avancée ; *The Greatest Conqueror* : 2 par
  Bouclier sur une tour de mage ; p. 18, *Scoring*) : specs 063, 043.

## Modèle

Aucun changement de l'état ni du glossaire. Le parcours **réutilise** le support des parcours (`crates/arcanum-core/tests/support/mod.rs`) :
`Parcours` (`play`, `view`, `legal`, `save`, `resume`, `explorer_turn`, `filler_turn`), `walk_to`, `explorable_from`,
`take_pending_reward`, `end_turn`, `has`. Les aides de marche et de combat de `parcours_j4.rs` montent dans `support/` si le
parcours en a besoin telles quelles (deux utilisateurs, règle de la 049) ; sinon elles restent dans `parcours_j5.rs`.

### Le plateau de jeu traversé

Cases lues sur `docs/reference/bac-a-sable.md` ; les tuiles de campagne de la pioche sont dans l'ordre 3 à 8 quelle que soit la
graine. La campagne 3 se pose en **2A** (comme au J4) ; la campagne 4 en **A + B**, permise ensuite parce qu'elle touche A, B et
2A (`exploration::placement`). Le test revérifie le terrain au passage.

| Case | Terrain, site | Rôle dans le parcours |
|---|---|---|
| `(0, 0)` | Plaine, portail | départ |
| `(4, -2)` | Plaine, village (campagne 2, E) | facultatif : recruter une unité pour le combat de la tour (la 049 le vérifie déjà) |
| `(4, -3)` | Forêt, clairière magique (campagne 2, N-E) | **exploration de 2A** (par `(5, -3)`, `(5, -4)`), puis **de A + B** (par `(4, -4)`) |
| `(5, -4)` | Plaine (campagne 3 en 2A, O) | passage hors de portée des orques de `(3, -3)` |
| `(4, -4)` | Plaine, village (campagne 4 en A + B, S-E) | voisine de la tour : le pas qui y arrive de jour retourne le défenseur |
| `(4, -5)` | Désert, tour de mage (campagne 4 en A + B, centre) | **assaut**, conquête, puis **achat** du sort |
| `(2, -3)` | Plaine, village (campagne 1, E) | départ du **lac** ; voisin des orques de `(3, -3)`, on y arrive par `(1, -2)` ou `(1, -3)` (trajet de `routes_avoiding`) |
| `(2, -4)` | Lac (campagne 1, N-E) | entré avec Pont de glace renforcé ; **défi** raté, **retraite forcée** vers `(2, -3)` |
| `(1, -4)` | Forêt, orques maraudeurs (campagne 1, N-O) | défiés depuis le lac, non vaincus |

Aucun pas ne provoque d'ennemi errant : `(4, -3)` → `(5, -4)` quitte les orques de `(3, -3)` sans arriver à côté d'eux, `(4, -4)`
n'est pas voisine des orques de `(3, -5)`, et `(4, -4)` → `(4, -5)` ne va pas d'une case voisine d'un errant à une autre case
voisine du même. La forteresse de la campagne 3, en `(7, -5)`, n'est jamais approchée : son défenseur reste face cachée.

Le lac `(2, -4)` touche les orques de `(1, -4)` et ceux de `(3, -5)`, pas ceux de `(3, -3)` ; le village `(2, -3)` touche
seulement ceux de `(3, -3)` : le pas du village au lac ne provoque personne (comme l'exemple de la spec 052, « le lac de la
campagne 1 »). Aucun site fortifié des tuiles de campagne 3 à 8 ne touche un lac : l'assaut raté depuis un lac (retraite de
l'assaut sur le lac, puis retraite forcée) reste couvert par la spec 052 (C24, campagne 11 posée par le test) ; ici, le combat
raté est un défi (Q5).

### Schéma : l'épisode du lac

```
   (1,-4) Orques ── défi depuis le lac, raté : ils restent
        \\
   (2,-4) Lac ◄── Pont de glace renforcé : entré pour 1
        |              fin du tour : retraite forcée ──┐
   (2,-3) Village ◄──────────── 1 case, 1 Blessure ────┘
        |
   (3,-3) Orques (voisins du village, pas du lac)
```

### Les choix scriptés

Trois choix dépendent de la graine ; le script prend **le premier** de chaque liste d'actions légales qui remplit la règle :

- **Sort de la récompense, et sort acheté** : un sort dont l'effet de base sert hors combat (Ailes du vent, Voyage souterrain,
  Distorsion spatiale, Méditation, Puits de mana, Lecture d'esprit ; Restauration avec une Blessure en main, Flux d'énergie avec
  une unité épuisée, Appel aux armes avec une unité dans l'offre), pour le lancer sans dépendre d'un combat (Q2).
- **Compétence** : celle des deux révélées qui a une option utilisable hors combat au tour suivant (Chemins ténébreux, Sombre
  négociation, Magie noire, Motivation, Invocation, Polarisation… ; spec 051, tableau).
- **Action avancée** : **Pont de glace** (*Frost Bridge*), sinon **Mélodie du vent** (*Song of Wind*, renforcée avec un mana
  bleu en plus : lacs à 0), les deux seules actions avancées des cartes 1 à 16 qui rendent un lac franchissable (spec 052, 053).
  Son effet de base sert dès le tour suivant (Mouvement 2), l'effet renforcé à l'épisode du lac.

Hors de ces choix, le parcours ne vise pas l'optimum : il marche, se bat et explore avec les aides du J4 ; une récompense de fin
de tour inattendue (niveau 4) est prise par `take_pending_reward`, comme au J4 (L4).

### Départ et préconditions (révision, issue privée 318)

La version d'origine partait de `new_game` avec une graine fixe : une seule graine sur 3 000 faisait tenir toutes les
préconditions (bilan de la 059). Le parcours part maintenant de départs choisis par le test (ADR 0006, « Départ d'un
parcours ») ; **la graine n'est plus choisie**.

**Parcours principal : mise en place arrangée** (`tests/support/arrangement.rs`, `Setup`), sur la vraie mise en place de
`new_game` :

- **Figurine** posée sur la clairière `(4, -3)` au lieu du portail : la marche jusqu'à la clairière n'est pas l'objet du parcours
  (J3 et J4 la couvrent) et coûte un tour.
- **Pile violette** : des Moines sur le dessus, le défenseur de la tour (battable au corps à corps par la main, sans être
  assommé). **Pile verte** : des Rôdeurs puis des Creuseurs sur le dessus, les errants tirés à la mise en place sur `(1, -4)` puis
  `(3, -3)`.
- **Offre de sorts** : deux sorts de la règle « Les choix scriptés » en tête de l'offre (les dernières places repartent à la
  nouvelle manche, MKUE p. 4, 2.d), puis un troisième sur le dessus de la pioche, pour l'achat.
- **Offre d'actions avancées** : Pont de glace en tête. **Compétences** : Chemins ténébreux et Sombre négociation sur le dessus de
  la pile (une option hors combat).
- **Deck du héros** et **dés de la Source** : l'ordre complet et les trois dés, choisis à l'implémentation pour que l'assaut, les
  trois choix, le tour qui suit et l'achat tiennent dans la **première manche, de jour** (la seule que l'arrangement contrôle : la
  nouvelle manche remélange le deck). Aucune unité n'est recrutée.

Le moteur joue tout le reste. Ce qui suit l'achat (nouvelle manche, exploration jusqu'à la fin de partie) dépend encore de la
graine, sans précondition propre : l'explorateur du J4 s'adapte à la main tirée.

**Épisode du lac : état posé** (second test, `parcours_lac_et_retraite_forcee`). Pont de glace, joué en C10 pour son effet de
base, ne revient en main qu'après la nouvelle manche, et la marche de la tour au village prend plusieurs tours : l'épisode
dépendait d'un hasard de cours de partie (13 tours après l'achat avec la graine 416). Il part donc de la mise en place arrangée
(figurine sur le village `(2, -3)`, Rôdeurs sur `(1, -4)`, un dé bleu dans la Source), plus un événement posé par le test :
**Pont de glace gagné** (`OfferCardGained` de l'offre d'actions avancées, comme à la récompense de niveau) puis pioché en main.
Rien de ce que vérifient C14 à C17 n'est posé.

Le test vérifie chaque précondition juste avant le point d'étape qui en a besoin, avec « départ à revoir » sinon (L1) :

- **Défenseur** : un ennemi violet que la main du tour de l'assaut peut vaincre (posé, vérifié quand même).
- **Choix** : un sort hors combat à la récompense et à l'achat, une compétence utilisable hors combat, Pont de glace dans l'offre.
- **Mana et influence** : au tour qui suit la conquête, un mana de la couleur du sort gagné ; au tour de l'achat, 7 influence et un
  mana de la couleur du sort acheté ; ensuite, le mana du sort acheté (dé, cristal ou jeton, Concentration comprise).
- **Mains** : de quoi marcher et vaincre le défenseur sans être assommé (L2).
- **Lac** : les Rôdeurs n'assomment pas le héros sans parade.

**Robustesse** (Q7) : un test ignoré `balayer_les_graines` (dans `tests/support/`, commun aux parcours) joue le parcours sur les
graines 0 à 99 et compte celles qui passent, avec la raison de l'échec des autres ; il remplace `chercher_une_graine`.

Au combat du lac, le script **rate exprès** : il ne pare pas, ne déclare aucune attaque et finit le combat ; les dégâts vont au
héros (pas à l'unité, s'il en a une), pour que le décompte des Blessures soit simple (Q6).

### Schéma : le parcours

```
Parcours principal : mise en place arrangée (Première reconnaissance, Arythea, figurine sur la clairière)  (C1)
│
├─ Manche 1/4 · Jour (deck et Source arrangés)
│   ├─ clairière (4, −3) : explorer 2A (campagne 3), puis A + B (campagne 4) : défenseur violet       (C2)
│   ├─ → (5, −4) → village (4, −4) : défenseur retourné                                              (C3)
│   ├─ assaut de la tour (4, −5) : Réputation −1 ; combat, défenseur vaincu, Bouclier                (C4)
│   │   fin du tour : sort de la récompense, niveau 2, compétence, action avancée                    (C5, C6, C7)
│   │   pioche : les trois cartes nouvelles en main                                                  (C8)
│   ├─ tour suivant : compétence utilisée, action avancée et sort joués pour leur effet              (C9, C10, C11)
│   │   ★ sauvegarde (après la fin de ce tour)                                                       (C22)
│   └─ tours suivants, sur la tour : interagir, acheter un sort ; au tour d'après, le lancer          (C12, C13)
├─ Nouvelle manche : compétence de nouveau prête, cartes gagnées dans le deck                         (C18)
├─ tours d'exploration jusqu'à la fin de la partie (cité révélée ou limite de manches)               (C19)
└─ fin de partie : score, Le Savant et Le Conquérant recomptés par le test                          (C20, C21)
                                                                         ★ reprise                    (C22)

Épisode du lac : état posé (mise en place arrangée sur le village (2, −3), Pont de glace gagné et en main)
│
└─ Manche 1/4 · Jour
    ├─ Pont de glace renforcé, lac (2, −4)                                                           (C14)
    ├─ défi des orques (1, −4), raté                                                                 (C15)
    └─ fin du tour : retraite forcée vers (2, −3), une Blessure ; tour suivant sur le village        (C16, C17)
```

Le découpage en tours est indicatif (ADR 0006). Le deck arrangé vise la première manche, de jour, pour C2 à C13 ; si
l'implémentation ne peut pas y faire tenir l'achat (C12, C13), il glisse à la manche 2 et le signale dans la PR, avec la mesure
de `balayer_les_graines`. L'ordre qui compte : 2A avant A + B, la conquête avant le niveau 2, les trois cartes gagnées à la même
fin de tour, le sort gagné lancé avant l'achat, l'achat avant le second lancer, l'entrée dans le lac et le défi raté dans le
même tour.

## Actions et événements

Aucun nouveau. Le parcours n'utilise que les actions existantes : `MoveHero` (et les pas de `walk_to`), `PlayCard` (effet de base
ou renforcé, de côté), `UseSkill`, `Interact`, `RecruitUnit`, `BuySpell`, `Explore`, `Challenge`, `StartCombat`, `NextCombatPhase`,
`DeclareBlock`, `ResolveBlock`, `DeclareAttack`, `ResolveAttack`, `ActivateUnit`, `AssignDamage`, `EndCombat`, `TakeSpell`,
`ChooseSkill`, `GainAdvancedAction`, `EndTurn`, `AnnounceEndOfRound` ; et, hors du jeu, `GameRunner::resume`.

## Comportements attendus

Tous dans `crates/arcanum-core/tests/parcours_j5.rs`. C1 à C13 et C18 à C21 sont des points d'étape du **parcours principal**
(`parcours_sorts_et_competences`), C14 à C17 ceux de l'**épisode du lac** (`parcours_lac_et_retraite_forcee`), vérifiés sur la
**vue du joueur** et sur les événements de la décision qui vient d'être prise. C22 est un troisième test qui rejoue le script
du parcours principal.

- **C1** — Étant donné la mise en place arrangée de Première reconnaissance, la vue donne niveau 1, gloire 0, aucune compétence, aucune
  compétence révélée ni offre commune, la pile de compétences d'Arythea à 10 jetons, et trois sorts dans l'offre de sorts —
  MKUE p. 5, *Using Skills* 2 ; specs 051 (C2, C3), 050.
- **C2** — Depuis la clairière `(4, -3)`, l'exploration de 2A révèle la campagne 3, puis celle de A + B la campagne 4 :
  `TileRevealed`, `DefenderPlaced` d'un jeton violet sur `(4, -5)`, `FameGained { fame: 1 }` à chaque tuile ; la vue montre le
  défenseur face cachée, sans dire lequel — carte Mage Tower ; specs 015, 043 (C1), 063 (C3).
- **C3** — De jour, le pas qui arrive sur `(4, -4)` produit `EnemyTokenRevealed` ; la vue donne l'ennemi violet du défenseur, et
  aucun `EnemiesProvoked` n'a été produit depuis le départ — carte Mage Tower ; specs 043 (C4), 030.
- **C4** — Le pas vers `(4, -5)` produit `AssaultStarted` et `ReputationChanged { change: −1 }` ; le combat contre le défenseur
  fortifié se gagne : `SiteConquered`, Bouclier sur `(4, -5)` dans la vue, `spell_rewards` à 1 — MKUE p. 7, 4.a ; p. 10, 2.c ;
  spec 043 (C11, C20).
- **C5** — Fin de ce tour : les actions légales sont d'abord les `TakeSpell` des sorts de l'offre, et elles seules ; après
  `TakeSpell`, `OfferCardGained { offer: Spells }` puis `OfferCardDealt { offer: Spells }` : l'offre a de nouveau trois sorts,
  sans celui pris — MKUE p. 11, 5.c ; p. 5, *"Gain" Effects* 3.a ; spec 043.
- **C6** — Puis `LevelGained { level: 2 }` et `SkillsRevealed` avec deux compétences d'Arythea ; les actions légales sont les
  `ChooseSkill` de ces deux-là (l'offre commune est vide) ; après `ChooseSkill`, la compétence est dans la vue du joueur, prête,
  l'autre dans l'offre commune, et la pile a 8 jetons — MKUE p. 11, 6.b ; spec 051 (C4, C5).
- **C7** — Puis les actions légales sont les `GainAdvancedAction` de l'offre d'actions avancées ; après le choix,
  `OfferCardGained { offer: AdvancedActions }` et l'offre est complétée ; plus aucun choix n'attend, la pioche suit — MKUE p. 11,
  6.b ; specs 050, 051 (C5, C6).
- **C8** — Le sort, puis l'action avancée, posés tour à tour sur le dessus du deck (l'action avancée au-dessus), sont piochés
  les premiers par la pioche de fin de tour, l'action avancée d'abord : les deux sont dans la main de la vue au début du tour
  suivant (la compétence n'est pas une carte ; le script finit le tour de l'assaut avec au plus trois cartes en main, pour que la
  limite de main de 5 laisse place aux deux) — MKUE
  p. 5, 3 ; p. 11, 7.b ; specs 043, 050.
- **C9** — Au tour suivant, `UseSkill` de la compétence gagnée produit `SkillUsed` avec l'effet de l'option, visible dans la vue
  (points de Mouvement ou d'Influence, cristal, jeton, cartes piochées) ; la compétence n'est plus utilisable ce tour : utilisée
  ce tour (une fois par tour) ou retournée (une fois par manche) — MKUE p. 5, *Using Skills* 4 ; specs 051 (C12 à C18), 058.
- **C10** — Dans ce même tour, l'action avancée gagnée est jouée **pour son effet** (`CardPlayed` avec un effet de base ou
  renforcé, pas `Sideways`), et l'effet se lit dans la vue — MKUE p. 4, 3.a ; specs 053, 056.
- **C11** — Dans ce même tour (ou au premier tour où le mana de sa couleur est là, au plus tard le suivant), le sort gagné est
  lancé : `CardPlayed` avec l'effet de base, payé d'un mana de sa couleur (`DieTaken`, `CrystalUsed` ou `ManaTokenUsed` de cette
  couleur) ; de jour, l'effet renforcé n'est pas parmi les actions légales — MKUE p. 4, 3.b ; specs 054 (C1, C2), 057.
- **C12** — Sur la tour conquise, `Interact` produit `InteractionStarted` (site tour de mage) ; avec 7 influence ou plus,
  `BuySpell` du sort choisi produit `SpellBought { cost: 7 }` : le sort quitte l'offre, l'offre est complétée (trois sorts),
  l'influence a baissé de 7 ; avant ces 7 points, `BuySpell` n'est pas parmi les actions légales — MKUE p. 8, 4.d ; spec 043.
- **C13** — À la fin de ce tour, la pioche donne le sort acheté (posé sur le dessus du deck) ; au tour suivant, ou au premier
  où le mana de sa couleur est là, il est lancé pour son effet de base, comme en C11 — MKUE p. 8, 4.d ; p. 4, 3.b ; spec 054.
- **C14** — Dans l'épisode du lac, sur le village `(2, -3)`, Pont de glace renforcé, payé d'un mana bleu : `CardPlayed` avec Mouvement 4 ; le lac
  `(2, -4)` est parmi les cases atteignables de la vue, pour 1 ; le pas qui y entre produit `HeroMoved` sans `EnemiesProvoked` ; la
  vue dit la case non sécurisée — MKUE p. 7, 7.b et 7.d ; carte Frost Bridge ; spec 052 (C14, C27).
- **C15** — Depuis le lac, `Challenge` des orques de `(1, -4)` produit `EnemyChallenged` et ouvre le combat ; le script ne pare pas, n'attaque pas et finit
  le combat : `CombatEnded` sans `EnemiesDefeated`, les dégâts non parés deviennent des Blessures du héros, aucun `KnockedOut` ; les
  orques restent sur `(1, -4)` dans la vue, et la figurine sur le lac — MKUE p. 8, *Combat* 1 ; p. 10, *Combat Outcomes* ;
  specs 031, 034, 052.
- **C16** — Fin de ce tour : le premier événement est `ForcedWithdrawal` vers `(2, -3)`, **une** case, puis **une** Blessure en
  main de plus ; aucun `EnemiesProvoked` ni `KnockedOut`, même si les Blessures en main dépassent la limite de main — MKUE p. 11,
  2 ; FAQ p3 ; Compendium, *Forced Withdrawal* S3 ; spec 052 (C19, C22, C23).
- **C17** — Après la retraite, la fin du tour s'évalue sur le village et sur la main d'après la retraite (Blessures du combat et de
  la retraite comprises) : défausse et pioche comme d'habitude ; au tour suivant, la figurine est sur `(2, -3)`, les orques toujours
  sur `(1, -4)`, et le lac n'est plus atteignable sans un nouvel effet — MKUE p. 11, 2 et 4 ; spec 052 (C21, C41).
- **C18** — Au début de la manche qui suit l'usage de la compétence, elle est de nouveau prête (une fois par manche retournée,
  remise face visible ; une fois par tour, prête comme à chaque tour) ; le Bouclier et le niveau sont gardés, et les trois cartes
  gagnées sont dans le deck reformé — MKUE p. 4, 2.g ; p. 5, *Using Skills* 4 ; specs 023, 051.
- **C19** — Pendant les tours d'exploration, la compétence reste dans la vue ; un niveau pair de plus (niveau 4) est pris par
  `take_pending_reward` et compté comme les autres ; la partie finit : `GameEnded` (cité révélée ou limite de manches), plus
  aucune action légale — MKUE p. 18, *Scenario End* ; spec 063 (C9, C10, C12).
- **C20** — Le score de la vue donne Le Savant = 2 × (sorts gagnés : 2) + 1 × (actions avancées gagnées), recompté par le test
  depuis les `OfferCardGained` et `SpellBought` du journal — MKUE p. 15, *The Greatest Knowledge* ; spec 063 (C14).
- **C21** — Le Conquérant compte 2 pour le Bouclier de la tour (et 2 de plus par site fortifié conquis en route, s'il y en a) ;
  le total du score est la somme de la gloire et des Hauts Faits de la vue — MKUE p. 15, *The Greatest Conqueror* ; specs 043,
  063.
- **C22** — Étant donné la même partie sauvegardée après le tour de C9 à C11 (journal + positions des générateurs), quand une
  seconde partie est reprise avec un nouveau joueur scripté qui joue la suite du même script, alors elle finit avec **le même
  journal**, **le même état** et **le même score** que la partie jouée d'une traite (`parcours_reprise_apres_les_cartes_gagnees`)
  — ADR 0001 ; specs 024, 038.

## Cas limites et refus

- **L1** — Un départ qui ne remplit plus une précondition (contenu changé, nouvel usage du hasard) fait échouer le test avec
  « départ à revoir », avant le point d'étape qui en dépend : on revoit l'arrangement, pas la graine.
- **L2** — Un héros assommé (spec 034) fausserait le parcours : le test vérifie qu'aucun `KnockedOut` n'est produit pendant
  l'assaut ni pendant le défi du lac ; les autres tours n'ont pas de combat.
- **L3** — Les tours de marche et d'exploration ne provoquent aucun ennemi errant ni n'assaillent d'autre site que la tour : le
  test vérifie qu'aucun `EnemiesProvoked` n'est produit (le défi de C15 n'en est pas un), et qu'un seul `AssaultStarted` (celui de C4) l'est jusqu'à C19.
- **L4** — Une récompense de fin de tour inattendue (niveau 4 : compétence et action avancée) est prise par
  `take_pending_reward` (le premier choix proposé) et comptée par C20 comme les autres.

## Information cachée

Les points d'étape se lisent sur la **vue du joueur** : offres de sorts et d'actions avancées, compétences gagnées et leur état,
compétences révélées, offre commune, taille de la pile de compétences, main, mana, influence d'interaction, défenseur face cachée
(couleur seule) puis retourné, Boucliers, gloire, niveau, score. L'ordre de la pile de compétences et des pioches de sorts et
d'actions avancées n'est pas lu. L'état complet ne sert qu'à comparer les deux parties de C22.

## Libellés (i18n)

Aucun.

## Interface

Aucune. L'écran de chaque spec du J5 a été vérifié à l'œil par sa spec ; pas de test egui (ADR 0006).

### Raccourcis clavier

Aucun.

## Hors périmètre

- **Artefacts** (055) : aucun site du parcours n'en donne sans dépendre fortement de la graine (ruines, donjon, monastère
  incendié) ; ils sont testés par leur spec.
- **Sorts de combat et effets renforcés de nuit** : testés par la 054 ; le parcours lance des sorts hors combat (Q2).
- **Compétences de combat** et compétences du second lot une à une : testées par les 051 et 058 ; le parcours en utilise une.
- **Forteresse et recrutement vérifiés** : parcourus par la 049.
- **Assaut raté depuis un lac** (retraite de l'assaut sur le lac, puis retraite forcée) : aucun site fortifié des tuiles 3 à 8 ne
  touche un lac ; couvert par la spec 052 (C24), Q5.
- **Retraite forcée de plusieurs cases** : couverte par la spec 052 (C20, campagne 11).
- **Joueur fantôme et tactiques** : J6 (specs 060, 061).

## Questions ouvertes

Aucune. Révision du départ (issue privée 318), décisions de Mathieu (2026-11-17) :

- **Q7 — Seuil de robustesse** : le parcours principal passe **au moins 90 graines sur 100**, mesuré par `balayer_les_graines`
  et noté dans la PR ; en dessous, on arrange davantage ou on pose un épisode de plus.
- **Q8 — Parcours du J3** (6 graines sur 400) : reporté, issue privée 322. J1, J2 et J4 gardent leur graine.
- **Q9 — Pont de glace posé** dans l'épisode du lac par un `OfferCardGained` écrit par le test (la carte vient de l'offre, comme un
  vrai gain), pas glissé dans le deck de départ.

Version d'origine : décisions de Mathieu (2026-11-14), propositions retenues :

- **Q1 — Premier sort** : les deux chemins, le sort de la récompense de la tour lancé, puis un second acheté à la tour et lancé.
- **Q2 — Sorts lancés** : des sorts dont l'effet de base sert hors combat, choisis par le script dans l'offre ; les sorts de
  combat restent couverts par la 054.
- **Q3 — Fin** : jusqu'à la fin de partie et au score, par la cité révélée ou par la limite de manches selon la graine.
- **Q4 — Forteresse** : non, le niveau 2 tombe à la conquête de la tour, à la même fin de tour que le sort.
- **Q5 — Combat raté** : un défi raté des orques de `(1, -4)` depuis le lac `(2, -4)`, atteint avec Pont de glace (ou Mélodie du
  vent) renforcé, puis la retraite forcée à la fin du tour (demande de Mathieu : dur à jouer et complexe, donc à bien tester) ;
  l'assaut raté depuis un lac reste couvert par la 052 (C24).
- **Q6 — Échec** : le script rate exprès, ni parade ni attaque, dégâts au héros.

## Notes d'implémentation

Révision du départ (issue privée 318, PR privée 320).

- **Mise en place arrangée** (`main_setup`) : Moines en tête de la pile violette ; Rôdeurs puis Creuseurs en tête de la pile
  verte (`(1, -4)` puis `(3, -3)`, commun aux deux tests) ; offre de sorts Restauration puis Distorsion spatiale, et Lecture
  d'esprit sur le dessus de la pioche ; Pont de glace en tête de l'offre d'actions avancées ; Chemins ténébreux et Sombre
  négociation en tête de la pile de compétences.
- **Deck et Source** (`MAIN_DECK`, `MAIN_SOURCE` : blanc, vert, vert), tour par tour, tout dans la **manche 1, de jour** :
  1. Cristallisation paie le dé blanc (cristal blanc, gardé pour l'achat) ; Endurance ×2 et Rapidité ×2 : les deux explorations
     et la marche jusqu'à `(4, -4)` ;
  2. Marche renforcée (dé vert) et Marche : le désert (5) ; Rage, Polyvalence et Détermination : attaque 6 contre les Moines,
     dont l'attaque n'est pas parée (3 Blessures en main, 3 dans la défausse par Poison) ;
  3. Chemins ténébreux (Mouvement 1), Pont de glace (Mouvement 2), Restauration (dé vert) : les 3 points de soin jettent les
     Blessures de la main, qui se vide ;
  4. Promesse, Menace, Improvisation et deux cartes de côté : 7 Influence ; Lecture d'esprit achetée avec le cristal blanc ;
  5. Lecture d'esprit lancée avec un jeton blanc de Concentration (dernière carte du deck).
- **Pourquoi Restauration** à la récompense (permise par « Les choix scriptés » avec une Blessure en main) : parer les Moines
  demanderait un sixième effet au tour de l'assaut ; leurs Blessures soignées, la main est libre pour l'achat au tour suivant.
  Le script ne paie l'Influence de l'achat qu'avec des effets de base et de côté, pour garder le cristal.
- **Épisode du lac** : `Setup::gain_advanced_action` pose `OfferCardGained` (dessus du deck), `OfferCardDealt` (la carte
  suivante de la pioche) et `CardDrawn` ; le reste (C14 à C17) est joué par le moteur.
- **Robustesse** (Q7) : `balayer_les_graines` passe **100 graines sur 100** (0 à 99 ; 78 s en profil de test).
- **Rouge** (plan, « Le rouge d'un parcours ») : sans deck ni Source arrangés, le parcours principal (et la
  reprise) échoue sur « Cristallisation et un dé blanc au tour 1 » ; sans Pont de glace posé, l'épisode du lac échoue sur « Pont de glace en main ».
- Notes de la version d'origine (graine 416) : dans l'historique de la PR privée 304.
