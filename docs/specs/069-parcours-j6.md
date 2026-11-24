# 069 — Parcours du jalon 6

- Statut : implémentée (validée le 2026-11-19, Q1 à Q6 : propositions retenues par Mathieu) (reporter tout changement dans la colonne Statut de `PLAN.md`)
- Jalon : J6
- Dépend de : 060 (tactiques), 061 (joueur fantôme), 062 (cités), 064 (Conquête en solitaire : côte, sorts compétitifs, fin
  du scénario, décompte), 063 (dernier tour, fin de partie) ; 059 (parcours du J5 : mise en place arrangée, `balayer_les_graines`) ;
  049 (explorateur scripté) ; ADR 0006 (stratégie de tests, « Départ d'un parcours »), ADR 0007 (un générateur par usage)
- Livret : MKUE p. 19 (*Solo Conquest*) ; p. 4 (3.a à 3.c : tactiques et ordre du tour ; 5.a : limite de manches) ; p. 15
  (*Dummy Player*) ; p. 14 (*Wedge Map*) ; p. 7 (5.g : cités révélées) ; p. 10 (*City Assaults* ; *Combat Outcomes* 1.a, 2.c,
  3.d) ; p. 11 (*End of the Turn*) ; Compendium, *Scenarios* Q8, O30 ; *Final Scoring* S2, S3 ; *Cities* S21, S23
- Crates touchées : arcanum-core (tests seulement : `crates/arcanum-core/tests/parcours_j6.rs`, et `crates/arcanum-core/tests/support/` pour
  deux aides : l'ordre de la pioche de tuiles, le choix des tactiques)

## Objectif

Vérifier que les specs du J6 **s'enchaînent dans une vraie partie**, comme le promet la ligne du plan : *partie solo complète
d'un scénario (tactiques, joueur fantôme, conquête d'une cité), jusqu'à la fin de partie et au score*. Rien ne change pour le
joueur : la spec n'ajoute que des tests.

Le scénario est **Conquête en solitaire** (Q1), le seul qui assemble tout le jalon. Deux tests, et une reprise :

- **Parcours principal** (mise en place arrangée) : une partie **entière**, six manches, avec Tovak contre le fantôme Goldyx.
  Chaque manche commence par le choix des tactiques (le joueur d'abord, le fantôme tire la sienne) et l'ordre du tour qui en
  découle ; le fantôme joue ses tours, annonce la fin de la manche deck vide, grossit à chaque nouvelle manche. L'explorateur
  du J4 révèle la région, cités comprises, sans jamais poser de tuile royaume sur la côte. Il n'assaille rien : la partie finit
  par la **limite de manches**, **mission échouée**, et le décompte solo se recompte (Q2).
- **Épisode de la conquête** (état posé) : la première cité déjà conquise, un héros armé pour la seconde ; le moteur **révèle**
  la cité de niveau 8, l'**assaille**, **bat en retraite** en gardant ses Boucliers, la **conquiert** à un second assaut ; le
  **dernier tour** se joue sans le fantôme, la partie finit, **mission accomplie**, et le décompte compte les cités, les manches
  non jouées, le deck du fantôme et la fin de manche non annoncée (Q3).
- **Reprise** : la partie du parcours principal sauvegardée en cours de route et reprise finit avec le même journal et le même
  score.

Il attrape ce que les tests isolés ne voient pas : une tactique retirée qui revient, une tactique tirée deux fois en six
manches, un ordre du tour qui ne suit pas les numéros, un fantôme qui joue pendant le dernier tour ou qui oublie son cristal de
fin de manche, une tuile royaume posée sur la côte, une cité au mauvais niveau, des Boucliers perdus à la retraite, une
conquête qui n'annonce pas le dernier tour, un décompte solo qui oublie une ligne, une reprise qui ne retire pas les mêmes
tactiques au fantôme.

## Règles

Aucune règle nouvelle : chaque point d'étape renvoie à la règle qu'il vérifie et à la spec qui l'implémente.

- **Choix des tactiques** (MKUE p. 4, 3.a ; p. 19, *Special Rules*) : le joueur choisit le premier, le fantôme tire au hasard
  parmi les restantes : specs 060, 061.
- **Ordre du tour** (MKUE p. 4, 3.c et 4.a) : le plus petit numéro de tactique joue le premier : spec 061 (C7, C8).
- **Tactiques retirées** (MKUE p. 19, *Special Rules* ; Compendium, *Scenarios* Q8) : les deux de la manche, à sa fin ; sur
  six manches, chaque tactique est prise une fois : specs 060, 064 (C5).
- **Effets choisis à la prise** (cartes de tactique) : Départ flamboyant pioche 2 cartes ; Préparation cherche une carte du
  deck : spec 060 (C19, C30). Les autres tactiques prises par le script sont prises sans être exploitées (Q4).
- **Tour du fantôme** (MKUE p. 15, *Dummy Player*) : 3 cartes retournées, plus une par cristal de la couleur de la
  dernière ; deck vide au début de son tour, il annonce la fin de la manche, le joueur joue un dernier tour : spec 061.
- **Nouvelle manche du fantôme** (MKUE p. 15) : l'action avancée en dernière place rejoint son deck, il gagne un cristal de
  la couleur du sort en dernière place, son deck est remélangé : spec 061 (C22 à C24).
- **Côte** (MKUE p. 14, *Wedge Map*) : pas de tuile royaume sur les emplacements `i = 0` ou `j = 0` : spec 064 (C6).
- **Cités révélées** (MKUE p. 7, 5.g ; p. 19, *Cities*) : la première de niveau 5, la seconde de niveau 8, avec leurs
  défenseurs (`cities.ron`) : specs 062, 064.
- **Assaut, retraite, conquête** (MKUE p. 10, *City Assaults*, *Combat Outcomes* 1.a, 2.c, 3.d ; Compendium, *Cities* S23) :
  Réputation −1 à chaque assaut, bonus de la cité, un Bouclier par défenseur vaincu, retraite sans les perdre, conquête quand il
  n'en reste aucun : spec 062.
- **Fin du scénario** (MKUE p. 19, *Scenario End* ; p. 4, 5.a ; Compendium, *Scenarios* O30) : toutes les cités conquises,
  dernier tour sans le fantôme ; sinon fin de la 6e manche : specs 063, 064.
- **Décompte solo** (MKUE p. 19, *Scoring* ; p. 15 ; Compendium, *Final Scoring* S2, S3) : gloire et Hauts Faits sans
  titres, +10 par cité, +15 toutes, et si la mission est accomplie +30 par manche non jouée, +1 par carte du deck du fantôme,
  +5 sans annonce de fin de manche : spec 064.

## Modèle

Aucun changement de l'état ni du glossaire. Le parcours réutilise le support des parcours (`crates/arcanum-core/tests/support/`) :
`Parcours` (`play`, `view`, `legal`, `save`, `resume`, `arranged`, `explorer_turn`), `Setup` (`deck_cards`, `enemies`,
`next_enemies`, `dummy_deck`, `figure`, `source`), `take_pending_reward`, `precondition`, `sweep_seeds`. Deux aides
s'ajoutent au support, toutes deux pour les tests seulement :

- **`Setup::tiles`** (nom indicatif) : l'ordre de la pioche de tuiles, comme `enemies` pour une pile d'ennemis. Il échange des
  tuiles dans `TileDeckFormed` (et dans ce que la mise en place en a déjà tiré), comme si le mélange avait donné cet ordre ;
  une tuile royaume hors de la pioche peut prendre la place d'une tuile royaume tirée (comme le tirage des tuiles royaume au
  hasard l'aurait fait).
- **Choix des tactiques par le script** : aujourd'hui `Parcours::play` prend d'office Lève-tôt ou Au crépuscule à chaque
  nouvelle manche (spec 060, pour que les parcours d'avant jouent comme avant). Le parcours du J6 choisit lui-même : une option
  du `Parcours` (`Parcours::choosing_tactics`, nom indicatif) coupe ce choix d'office ; les parcours J1 à J5 ne changent pas.

### Le choix des tactiques par le script

Le script choisit la **première tactique légale** d'une liste de préférences, et répond à toute décision qu'une tactique
ouvre par l'option la plus neutre (Q4) :

| Moment | Préférences, dans l'ordre | Décision ouverte, et réponse du script |
|---|---|---|
| Jour | Départ flamboyant (5), Lève-tôt (1), Au bon moment (6), Planification (4), Faux départ (2), Vol de mana (3) | Faux départ : défausser 0 carte ; Vol de mana : le premier dé de couleur de base ; Au bon moment : jamais annoncé |
| Nuit | Préparation (5), Au crépuscule (1), Appel du mana (3), Méditation nocturne (4), Provisions (6), Longue Nuit (2) | Préparation : la première carte du deck qui n'est pas une Blessure ; Méditation nocturne, Provisions : jamais utilisées ; Longue Nuit : refusée |

Le fantôme tire au hasard parmi les restantes : sa tactique, l'ordre du tour et donc les préférences atteintes dépendent de la
graine. Aux manches 5 et 6, le joueur choisit entre les deux dernières de son moment.

### Départ et préconditions

**Parcours principal : mise en place arrangée** (`Setup`), sur la vraie mise en place de `new_game` avec Conquête en solitaire,
Tovak, fantôme Goldyx (cristaux 2 verts, 1 bleu ; `heroes.ron`, `dummy_crystals`) :

- **Pioche de tuiles** : les deux tuiles royaume de cité en tête des tuiles royaume (les cinq tuiles de campagne restent dans
  l'ordre de la graine), pour que l'explorateur révèle une cité dès la sixième tuile.
- **Deck du héros** et **dés de la Source** : rien d'imposé, sauf si l'implémentation en a besoin pour la première manche (à
  noter dans les Notes d'implémentation).
- **Deck du fantôme** : l'ordre de la graine (Q5). Ce qu'on vérifie de ses tours se recompte depuis `DummyView::last_turn`.

Le moteur joue tout le reste, jusqu'à `GameEnded`. Le test vérifie chaque précondition juste avant le point d'étape qui en a
besoin, avec « départ à revoir » sinon (L1) : une cité révélée avant la fin de la partie ; le fantôme qui annonce la fin d'au
moins une manche ; Départ flamboyant et Préparation pris au moins une fois par le joueur.

**Épisode de la conquête : état posé** (second test). Arriver devant la seconde cité avec un héros capable de la prendre
demande plusieurs manches de hasard (main tirée, offres, niveaux, unités) : un hasard de cours de partie (ADR 0006). L'épisode
part de la mise en place arrangée (même scénario, même héros, même fantôme ; défenseurs choisis en tête des piles), puis des
événements posés par le test :

- les cinq tuiles de campagne révélées et la **première cité** (niveau 5) révélée et conquise : `TileRevealed`, `CityPlaced`,
  ses défenseurs, un `CityShieldPlaced` par défenseur, `CityConquered`, comme la partie préparée `conquete-derniere-cite`
  (spec 064) ;
- la **prochaine tuile** : la seconde cité, la tuile royaume 8 (cité rouge, `tiles.ron`) ; la figurine sur une case qui
  touche un emplacement intérieur où la poser ;
- de quoi la prendre en **deux assauts au moins** : unités recrutées, cartes gagnées (comme `gain_advanced_action`, depuis
  les offres), cristaux, niveau ; choisis à l'implémentation et notés dans les Notes d'implémentation (Q3) ;
- la **manche 1, de Jour**, au choix de la tactique : rien d'autre n'est posé sur le temps ni sur le fantôme, pour que les
  manches non jouées et le deck du fantôme se comptent sur une vraie première manche.

Rien de ce que vérifient C13 à C19 n'est posé : la révélation de la seconde cité, ses défenseurs, les assauts, les Boucliers,
la conquête, le dernier tour, la fin et le décompte viennent du moteur.

**Robustesse** : `balayer_les_graines` (spec 059, Q7) joue le parcours principal sur les graines 0 à 99 ; il en passe **au
moins 90** (Q6), mesure notée dans la PR. L'épisode est presque entièrement posé : une seule graine suffit, balayée aussi.

### Schéma : le parcours principal

```
Mise en place arrangée (Conquête en solitaire, Tovak, fantôme Goldyx)                          (C1)
│
├─ Manche 1 · Jour
│   ├─ choix : Départ flamboyant (si pris : main de 7) ; le fantôme tire parmi les 5 autres    (C2)
│   ├─ ordre du tour selon les numéros ; tours du joueur (explorateur) et du fantôme             (C3, C4)
│   └─ deck du fantôme vide : il annonce, dernier tour du joueur, fin de manche                  (C5)
├─ Nouvelle manche : 2 tactiques retirées ; le fantôme prend action avancée et cristal          (C6)
├─ Manche 2 · Nuit : Préparation (si prise), etc.                                                (C7)
│   ★ sauvegarde (au début de la manche 3)                                                       (C12)
├─ Manches 3 à 6 : six tactiques de chaque moment, chacune prise une fois                        (C8)
│   ├─ exploration : la sixième tuile est une cité de niveau 5, plus tard la seconde de niveau 8 (C9)
│   └─ aucune tuile royaume sur la côte, aucun assaut                                            (C10, L3)
└─ Fin de la 6e manche : GameEnded { LastRound }, mission échouée, décompte recompté             (C11)
                                                                         ★ reprise               (C12)
```

### Schéma : l'épisode de la conquête

```
État posé : 5 tuiles de campagne, cité de niveau 5 conquise (1/2), héros armé, manche 1 de Jour
│
├─ explorer : tuile royaume 8, cité rouge de niveau 8 et ses 4 défenseurs                        (C13)
├─ assaut 1 : Réputation −1 ; une partie des défenseurs vaincus ; retraite, Boucliers gardés     (C14)
├─ assaut 2 (un tour suivant) : Réputation −1 ; les restants vaincus ; conquête (2/2)             (C15)
│   └─ LastTurnAnnounced ; fin du tour sans tour du fantôme                                       (C16)
├─ dernier tour ── EndTurn ── GameEnded { AllCitiesConquered }                                   (C17)
└─ décompte : mission accomplie, cités 20 + 15, manches non jouées, deck du fantôme, +5         (C18, C19)
```

Le découpage en tours est indicatif (ADR 0006). L'ordre qui compte : la tactique choisie avant tout tour de la manche ; le
fantôme tire après le joueur ; les tactiques retirées à la fin de la manche, avant le choix suivant ; la cité de niveau 5
révélée avant celle de niveau 8 ; la retraite avant la conquête ; la conquête de la seconde cité avant le dernier tour ; le
dernier tour avant `GameEnded`.

## Actions et événements

Aucun nouveau. Le parcours n'utilise que des actions existantes : `ChooseTactic`, `RethinkDiscard`, `StealDie`,
`PrepareCard`, `DeclineLongNight`, `MoveHero`, `PlayCard`, `Explore`, `StartCombat`, `NextCombatPhase`, `DeclareBlock`,
`ResolveBlock`, `DeclareAttack`, `ResolveAttack`, `ActivateUnit`, `AssignDamage`, `EndCombat`, `ChooseSkill`,
`GainAdvancedAction`, `EndTurn`, `AnnounceEndOfRound` ; et, hors du jeu, `GameRunner::resume`.

## Comportements attendus

Tous dans `crates/arcanum-core/tests/parcours_j6.rs`. C1 à C11 sont des points d'étape du **parcours principal**
(`parcours_conquete_en_solitaire`), C13 à C19 ceux de l'**épisode de la conquête** (`parcours_conquete_des_cites`), vérifiés
sur la **vue du joueur** et sur les événements de la décision qui vient d'être prise. C12 est un troisième test
(`parcours_reprise_avec_le_fantome`).

Parcours principal :

- **C1** — Mise en place arrangée : la vue donne le scénario Conquête en solitaire, la manche 1 de Jour, la progression des
  cités à 0/2 (`PlayerView::conquest`), le fantôme Goldyx avec 16 cartes au deck et ses cristaux (2 verts, 1 bleu ;
  `PlayerView::dummy`), 16 sorts en tout (offre et pioche ; aucun des quatre compétitifs) et les six tactiques de Jour au
  choix (`PlayerView::tactic_choice`), aucune retirée — MKUE p. 19 ; p. 15 ; specs 061 (C3), 064 (C1, C3, C22).
- **C2** — À chaque manche, la première décision est le choix de la tactique, et lui seul : les actions légales sont des
  `ChooseTactic` des tactiques du moment non retirées ; après le choix, `TacticChosen` puis `DummyTacticDrawn` d'une autre
  tactique du même moment. Départ flamboyant, quand le script le prend, produit 2 `CardDrawn` : main de 7 — MKUE p. 4, 3.a ;
  p. 19, *Special Rules* ; specs 060 (C19), 061 (C6).
- **C3** — L'ordre du tour suit les numéros : si la tactique du fantôme a le plus petit numéro, un `DummyTurnPlayed` précède le
  `TurnStarted` du premier tour du joueur, sinon aucun ; ensuite, joueur et fantôme alternent, un `DummyTurnPlayed` entre deux
  tours du joueur ; `PlayerView::round_order` le dit — MKUE p. 4, 3.c et 4.a ; spec 061 (C7, C8, C39).
- **C4** — Chaque `DummyTurnPlayed` retourne 3 cartes, plus une par cristal du fantôme de la couleur de la troisième (moins
  s'il en reste moins), recompté par le test depuis `DummyView::last_turn` et `DummyView::crystals` ; `DummyView::deck_size`
  baisse d'autant — MKUE p. 15 ; spec 061 (C11 à C15).
- **C5** — Deck du fantôme vide au début de son tour : `DummyAnnouncedEndOfRound` ; au tour du joueur qui suit,
  `AnnounceEndOfRound` n'est pas légale ; sa fin produit `RoundEnded`, sans autre `DummyTurnPlayed` entre les deux. Quand le
  joueur annonce le premier (deck vide), le fantôme joue son dernier tour, puis `RoundEnded` — MKUE p. 15 ; p. 6, 4 ;
  spec 061 (C17, C21).
- **C6** — À chaque fin de manche, avant la manche suivante : `TacticsRemoved` avec la tactique du joueur et celle du fantôme ;
  `DummyCardAdded` de l'action avancée qui était en dernière place, `DummyCrystalGained` de la couleur du sort qui était en
  dernière place (un des seize : jamais un sort compétitif), `DummyDeckShuffled` d'un deck d'une carte de plus que la manche
  d'avant ; puis `RoundStarted` du moment suivant — MKUE p. 15 ; p. 19 ; specs 060 (C9), 061 (C9, C22 à C24), 064 (C3).
- **C7** — Préparation, quand le script la prend : les actions légales sont des `PrepareCard` des cartes du deck ; après le
  choix, `CardSearched` et la carte est dans la main de la vue — spec 060 (C30).
- **C8** — Sur la partie, les tactiques prises (celles du joueur et du fantôme, depuis `TacticChosen` et `DummyTacticDrawn`)
  sont les douze, chacune une fois ; aux manches 5 et 6, le choix ne propose que deux tactiques, et le fantôme prend celle que
  le joueur laisse ; aucune tactique retirée n'est jamais parmi les actions légales — MKUE p. 19, *Special Rules* ;
  Compendium, *Scenarios* Q8 ; spec 064 (C5).
- **C9** — La première tuile royaume révélée est une cité : `TileRevealed`, puis `CityPlaced` de **niveau 5** et ses
  défenseurs (`DefenderPlaced`, couleurs de `cities.ron` pour sa couleur au niveau 5) ; si la seconde est révélée avant la fin,
  elle est de **niveau 8**. Aucune gloire pour une tuile révélée — MKUE p. 7, 5.g ; p. 19, *Cities* ; specs 062 (C4, C5),
  064 (C1).
- **C10** — Quand la prochaine tuile est une tuile royaume, aucun `Explore` vers un emplacement de la côte (`i = 0` ou
  `j = 0`) n'est parmi les actions légales ; à la fin, aucune tuile royaume du plateau de la vue n'est sur la côte — MKUE
  p. 14, *Wedge Map* ; spec 064 (C6).
- **C11** — Fin de la 6e manche : `RoundEnded { round: 6 }` puis `GameEnded { LastRound }`, plus aucune action légale ; la
  vue donne le décompte (`PlayerView::final_score`) : partie « conquête » avec 0 cité conquise sur 2, mission échouée, manches
  non jouées, deck du fantôme et annonce à 0 ; le total est la gloire plus les Hauts Faits, recomptés par le test depuis la
  vue — MKUE p. 4, 5.a ; p. 19, *Scoring* ; spec 064 (C15, C19).

Reprise :

- **C12** — Étant donné la partie du parcours principal sauvegardée au début de la manche 3, avant le choix de la tactique
  (journal + positions des générateurs), quand une seconde partie est reprise avec un nouveau joueur scripté qui joue la suite
  du même script, alors elle finit avec **le même journal**, **le même état** et **le même score** que la partie jouée d'une
  traite : mêmes tactiques tirées par le fantôme, mêmes cartes retournées, mêmes tuiles — ADR 0001 ; specs 024, 038, 061 (C34).

Épisode de la conquête :

- **C13** — Depuis la case posée, `Explore` révèle la tuile royaume 8 : `TileRevealed`, `CityPlaced { color: Red, level: 8 }`
  et quatre `DefenderPlaced` (brun, violet, blanc, blanc : `cities.ron`, niveau 8 de la cité rouge) ; la vue donne la
  progression 1/2 — MKUE p. 7, 5.g ; p. 19, *Cities* ; specs 062 (C4, C5), 064 (C22).
- **C14** — Premier assaut : le pas sur la cité produit `AssaultStarted` et `ReputationChanged { change: −1 }` ; le combat a
  les quatre défenseurs, fortifiés, avec le bonus de la cité rouge (un défenseur à attaque physique est Brutal) ; le script en
  bat une partie, pas tous, et finit le combat : un `CityShieldPlaced` par défenseur vaincu, dans l'ordre, puis `HeroWithdrew`
  vers la case d'où il a assailli ; pas de `CityConquered` ni de `LastTurnAnnounced` ; la vue garde ces Boucliers sur la cité
  et la progression à 1/2 — MKUE p. 10, *City Assaults*, 1.a, 3.d ; Compendium, *Cities* S23 ; spec 062 (C9, C10, C19).
- **C15** — Second assaut, un tour suivant : de nouveau `ReputationChanged { change: −1 }` ; le combat n'a que les défenseurs
  restants ; tous vaincus : leurs `CityShieldPlaced`, puis `CityConquered { leader: joueur }`, puis `LastTurnAnnounced` ; la vue
  donne 2/2, les Boucliers des deux assauts sur la cité — MKUE p. 10, 2.c ; p. 19, *Scenario End* ; specs 062 (C20), 064 (C11).
- **C16** — Le joueur finit ce tour (pas d'interaction dans la cité ce tour-là) : `TurnEnded`, pioche, `TurnStarted`, **sans**
  `DummyTurnPlayed` ; `PlayerView::last_turn` dit que ce tour est le dernier — MKUE p. 19, *the Dummy player does not* ;
  Compendium, *Cities* S21 ; spec 064 (C12).
- **C17** — La fin du dernier tour produit `GameEnded { AllCitiesConquered }` ; plus aucune action légale — MKUE p. 19 ;
  spec 064 (C12).
- **C18** — Le décompte de la vue : partie « conquête » avec 2 cités conquises sur 2, mission accomplie, manches non jouées =
  6 − la manche où la partie finit, deck du fantôme = `DummyView::deck_size` à la fin, fin de manche non annoncée (le fantôme
  n'a pas annoncé pendant cette manche : précondition) ; le total est gloire + Hauts Faits + 10 × 2 + 15 + 30 × manches non
  jouées + deck + 5, recompté par le test — MKUE p. 19, *Scoring* ; spec 064 (C16, C17).
- **C19** — Les Boucliers des deux cités ne comptent pas dans Le Conquérant ; la gloire de la vue contient la gloire entière
  de chaque défenseur vaincu aux deux assauts (`enemies.ron`), recomptée par le test — Compendium, *Final Scoring* S2,
  *Cities* S23 ; specs 062, 064 (C20).

## Cas limites et refus

- **L1** — Un départ qui ne remplit plus une précondition (contenu changé, nouvel usage du hasard) fait échouer le test avec
  « départ à revoir », avant le point d'étape qui en dépend : on revoit l'arrangement, pas la graine.
- **L2** — Héros assommé (spec 034) : aucun `KnockedOut` pendant les assauts de l'épisode ; le parcours principal n'a pas de
  combat voulu.
- **L3** — Le parcours principal n'assaille aucun site et ne provoque aucun ennemi errant : aucun `AssaultStarted` ni
  `EnemiesProvoked` (l'explorateur évite les cases des ennemis errants et ne traverse aucun site fortifié non conquis, cités comprises :
  `routes_avoiding`, specs 030, 043).
- **L4** — Une récompense de fin de tour inattendue (niveau pair) est prise par `take_pending_reward`, comme au J4 ; la
  compétence révélée par le fantôme à ce niveau va dans l'offre commune (spec 061, C26), sans précondition propre.
- **L5** — La seconde cité jamais révélée dans le parcours principal (côte, pioche) n'est pas un échec : C9 ne vérifie le
  niveau 8 que si elle l'est (spec 064, L1).
- **L6** — Épisode : si le fantôme a déjà annoncé la fin de la manche au moment de la conquête, la fin du tour de la conquête
  finit la manche et la partie (spec 064, C13) ; le deck du fantôme est arrangé pour que cela n'arrive pas, et C18 le vérifie
  comme précondition.

## Information cachée

Les points d'étape se lisent sur la **vue du joueur** : tactiques au choix et retirées, tactique et ordre du tour, fantôme
(héros, tactique, taille du deck, défausse, dernier tour joué, cristaux, annonce), plateau, cités (niveau, Boucliers, Seigneur),
défenseurs face cachée (couleur seule) puis retournés, progression des cités, dernier tour, raison de fin, décompte. L'ordre du
deck du fantôme, de sa pile de compétences et de la pioche de tuiles n'est pas lu. L'état complet ne sert qu'à comparer les deux
parties de C12.

## Libellés (i18n)

Aucun.

## Interface

Aucune. L'écran de chaque spec du J6 a été vérifié à l'œil par sa spec, depuis ses parties préparées (`conquete-cote`,
`conquete-derniere-cite`, `conquete-accomplie`, `conquete-echouee` pour la 064) ; pas de test egui (ADR 0006).

### Raccourcis clavier

Aucun.

## Hors périmètre

- **Interaction dans une cité conquise** (recrutement d'unités à icône Cité, achat propre à sa couleur, limite de main) :
  testée par la spec 062 ; l'épisode finit avant.
- **Effets de chaque tactique** : testés par la spec 060 ; le parcours exploite Départ flamboyant et Préparation (Q4).
- **Fin de manche pendant le dernier tour** (Compendium, *Scenarios* O30) : testée par les specs 061 (C30) et 064 (C13, C14).
- **Première reconnaissance** jusqu'au score : parcourue par les J4 et J5.
- **Variantes** du scénario et autres missions solo : hors plan.

## Questions ouvertes

Aucune. Décisions (Mathieu, 2026-11-19 : propositions retenues) :

- **Q1 — Scénario du parcours** : **Conquête en solitaire**, le seul qui réunit tactiques, fantôme, cités
  ouvertes et décompte solo ; Première reconnaissance a ses cités fermées et ses tactiques sont déjà jouées par le J5.
- **Q2 — Fin du parcours principal** : l'explorateur joue les six manches sans rien assaillir, la partie finit par la limite
  de manches, **mission échouée**. Un script qui monte en puissance et prend deux cités sur une graine
  quelconque demanderait un joueur bien plus fin que l'explorateur, et le parcours dépendrait de la graine ; la conquête est
  vérifiée par l'épisode (Q3).
- **Q3 — Épisode de la conquête** : la première cité posée conquise, et le moteur révèle puis prend la **seconde** (niveau 8,
  quatre défenseurs) en **deux assauts**, une retraite puis la conquête, avec un héros armé par des gains posés (unités,
  cartes des offres, cristaux, niveau). Les défenseurs sont choisis en tête des piles (ADR 0006) pour rester
  battables. Repli, si l'implémentation n'y arrive pas en deux ou trois assauts : poser des Boucliers sur une partie des
  défenseurs, comme `conquete-derniere-cite`, en gardant au moins un assaut avec retraite, et le signaler dans la PR.
- **Q4 — Tactiques du script** : les listes de préférences du Modèle, avec deux tactiques exploitées et vérifiées (Départ
  flamboyant, Préparation) et les autres prises sans servir ; les effets restent couverts par la spec 060.
- **Q5 — Héros** : Tovak pour le joueur, **Goldyx** pour le fantôme : ses cristaux (2 verts, 1 bleu) font varier
  le nombre de cartes retournées, ce que C4 vérifie ; Tovak change d'Arythea, le héros du parcours du J5.
- **Q6 — Seuil de robustesse** : au moins **90 graines sur 100** pour le parcours principal, comme le J5 ; six
  manches font un balayage plus long (test ignoré, lancé à la main).

## Notes d'implémentation

Implémentée sur la branche `spec/069-parcours-j6` (PR privée 349), tests seulement : `crates/arcanum-core/tests/parcours_j6.rs` et
`crates/arcanum-core/tests/support/`. Plan : `plans/069-parcours-j6.md`.

- **Graine et préconditions** (décision de Mathieu, 2026-11-19) : l'explorateur du J4 ne révèle une cité que sur 16 graines
  sur 40 (orques, tours de mage, lacs, ou manches trop courtes face au fantôme), et arranger l'ordre des tuiles de campagne
  n'y change rien (2 graines sur 5 au mieux). Les deux cités révélées (niveaux 5 puis 8) et une annonce du fantôme sont donc
  des préconditions de la **graine du test** seulement (graine 22) ; au balayage, C5 et C9 ne se vérifient que si cela arrive
  (comme L5).
- **Mise en place arrangée** : `Setup::core_tiles` met les deux tuiles de cité en tête des tuiles royaume ; deck du héros,
  dés et deck du fantôme dans l'ordre de la graine.
- **Choix des tactiques** : `Parcours::arranged_choosing_tactics` coupe le choix d'office des parcours d'avant. Provisions,
  prise par le script, ouvre un choix avant chaque tour : le script retourne la tactique sans rien avoir posé dessous
  (`SparingPower { take: true }`), le plus neutre des deux choix légaux.
- **C6, ordre des événements** : le moteur pose `DummyCardAdded`, `DummyCrystalGained` et `DummyDeckShuffled` dans la
  préparation de la nouvelle manche, **après** `RoundStarted` (spec 061, C22 à C24) ; le test vérifie qu'ils suivent
  `RoundEnded` et précèdent tout tour, et que `TacticsRemoved` précède `RoundStarted`.
- **C3, C4 et C5** se vérifient sur chaque décision gardée (`Parcours::record_steps`) : la vue et les actions légales d'avant,
  les événements, la vue d'après.
- **Épisode de la conquête**, état posé (`episode_setup`) : tuiles de campagne fixées (10 et 1 à la mise en place, puis 4,
  3, 2, 8, 7), pour que le terrain autour de la seconde cité ne dépende pas de la graine ; Mages de glace en tête de la pile
  violette, pour les tours de mage de la mise en place ; tuiles de campagne posées en (2, 0), (0, 2), (1, 2), (3, 0), (0, 3) ;
  première cité en (2, 1), conquise avec trois Boucliers ; la seconde (tuile royaume 8) se révèle en (1, 1), depuis une case
  voisine de sa colline N-E, seule case voisine de la cité d'où l'assaillir sans provoquer ses deux Draconum. Défenseurs en
  tête des piles : Minotaure, Moines, deux Congeleurs. Gloire 80 (niveau 9), cristaux (2 rouges, 3 bleus, 2 verts,
  1 blanc), deux Catapultes et deux Artilleurs d'Amotep recrutés, Frisson et Tourbillon gagnés sur le deck (la main du
  deuxième tour) ; main du premier tour : Marche, Endurance ×2, Célérité ×2.
  - **Premier assaut** : les Catapultes (siège, mana rouge et bleu) battent le Minotaure et les Moines en phase de tir ; les
    Congeleurs blessent le héros (2 Blessures, pas assommé), retraite avec deux Boucliers.
  - **Second assaut**, au tour suivant : Tourbillon et Frisson empêchent les Congeleurs d'attaquer ; les Artilleurs (5 + 5)
    et Rage renforcée (4) les battent ; conquête, dernier tour annoncé.
  - La gloire des défenseurs (26) fait passer le niveau 10 à la fin du tour de la conquête : la compétence et l'action
    avancée sont prises (L4) avant la pioche et le dernier tour (C16).
- **Robustesse** : `balayer_les_graines` joue le parcours principal (préconditions de graine levées, voir plus haut) puis
  l'épisode sur les graines 0 à 99 : **100 sur 100** passent (2026-11-19).
