# 049 — Parcours du jalon 4

- Statut : implémentée (2026-11-06) ; validée le 2026-11-06 (propositions de Q1 à Q5 retenues par Mathieu)
- Jalon : J4
- Dépend de : 063, 065, 050, 040, 042, 041, 043, 044, 045 (J4, implémentées) ; 028 et 039 (joueur scripté, aides des parcours) ;
  ADR 0006 (stratégie de tests), ADR 0007 (un générateur par usage)
- Livret : MKUE p. 18 (*First Reconnaissance* : *Special Rules*, *Scenario End*, *Scoring*, *Solo Variant*) ; p. 15 (*Standard
  Achievements Scoring*) ; p. 8 (*Interacting with Locals* 4.a ; *Combat with Enemies* 1.a) ; p. 5 (*Using Units* 2 à 4) ;
  p. 7 (4.a) ; p. 10 (*Combat Outcomes* 2.c) ; p. 11 (*End of the Turn* 6) ; p. 4 (2.g) ; carte de site Keep
  (`docs/regles/cartes/sites-et-cites.md`)
- Crates touchées : arcanum-core (tests seulement : `crates/arcanum-core/tests/parcours_j4.rs`, et `crates/arcanum-core/tests/support/` si une
  aide y monte)

## Objectif

Vérifier que les specs du J4 **s'enchaînent dans une vraie partie**, comme le promet la ligne « Jouable » du jalon : *une partie
complète de Première reconnaissance en solo, avec villages, unités, sites et récompenses* (`PLAN.md`). Rien ne change pour le
joueur : la spec n'ajoute que des tests.

Chaque spec du J4 a ses tests sur l'arène ou sur une partie préparée ; les parcours 019, 028 et 039 jouent le bac à sable. Celui-ci
part de `new_game` en **Première reconnaissance**, avec une graine fixe, et va **jusqu'au score** : il recrute une unité, s'en sert
pour conquérir une forteresse, gagne une action avancée au niveau 2, explore jusqu'à la cité et vérifie le décompte final ligne par
ligne. Il attrape ce que les tests isolés ne voient pas : une unité qui ne se réactive pas à la nouvelle manche, un Bouclier oublié
au décompte, une carte gagnée qui ne compte pas pour Le Savant, un dernier tour qui ne vient pas dans une partie longue, une
sauvegarde qui ne rejoue pas les offres.

## Règles

Aucune règle nouvelle : chaque point d'étape renvoie à la règle qu'il vérifie et à la spec qui l'implémente.

- **Mise en place de Première reconnaissance** (MKUE p. 18, *Special Rules*) : une unité de village au moins dans l'offre d'unités,
  pas d'unités d'élite, actions avancées 1 à 16 : specs 050, 063.
- **1 gloire par tuile révélée** (MKUE p. 18, *Special Rules*) : spec 063.
- **Interagir et recruter** (MKUE p. 8, *Interacting with Locals* 4.a ; p. 5, *Using Units* 2) : au village, une unité de
  l'offre recrutable au village, payée en influence, un jeton de commandement : specs 040, 042.
- **Défenseur de la forteresse** (carte de site Keep) : jeton gris face cachée posé à la révélation de la tuile, retourné de jour
  quand la figurine arrive à côté : spec 043.
- **Assaut et conquête** (MKUE p. 7, 4.a ; p. 8, *Combat with Enemies* 1.a ; p. 10, *Combat Outcomes* 2.c) : Réputation −1,
  combat contre le défenseur fortifié ; vaincu, Bouclier sur la forteresse : spec 043.
- **Unité en combat** (MKUE p. 5, *Using Units* 3 et 4) : activée pour une Attaque ou une Parade, elle est épuisée jusqu'à la
  nouvelle manche, où elle redevient disponible (p. 4, 2.g) : spec 042.
- **Niveau pair** (MKUE p. 11, *End of the Turn* 6) : au niveau 2, une action avancée de l'offre, posée sur le deck et piochée
  aussitôt ; la compétence attend le J5 : specs 035, 050. Une carte dont l'effet n'est pas codé se joue de côté (p. 4, 3.e).
- **Fin du scénario** (MKUE p. 18, *Scenario End*) : la cité révélée annonce un dernier tour, après celui de la révélation :
  spec 063.
- **Décompte final** (MKUE p. 18, *Scoring* et *Solo Variant* ; p. 15) : gloire et six Hauts Faits, sans titres : specs 063,
  041, 042, 043, 044, 050.

## Modèle

Aucun changement de l'état ni du glossaire. Le parcours **réutilise** le support des parcours (`crates/arcanum-core/tests/support/mod.rs`) :
`Parcours` (`play`, `view`, `legal`, `save`, `resume`), le joueur scripté, `walk_to`, `in_hand`, `first_playable`, `filler_turn`,
`end_turn`. Les aides de combat et de marche sur plusieurs tours de `parcours_j3.rs` montent dans `support/` si le parcours en a
besoin telles quelles (deux utilisateurs) ; sinon elles restent dans `parcours_j4.rs`.

### L'explorateur scripté

Après la conquête, il faut **révéler les tuiles une à une jusqu'à la cité** (6 tuiles de campagne, puis 1 à 3 tuiles royaumes) :
une quinzaine de tours sur trois manches, qui ne vérifient rien un par un. Comme le tour de remplissage de la 028, un **tour
d'exploration** applique une règle fixe, écrite dans le support :

1. si l'annonce de fin de manche est légale, annoncer (comme le tour de remplissage) ;
2. sinon, jouer de côté pour Mouvement les cartes de la main qui ne sont pas des Blessures, une à une, jusqu'à pouvoir explorer ou
   jusqu'à la main vide ;
3. si une exploration est légale, explorer (la première dans l'ordre des actions légales, retrouvée par sa valeur) ;
4. sinon, marcher vers la case la moins chère d'où une exploration serait possible (trajet de `routes_avoiding`, spec 036 : il
   évite les ennemis errants et ne provoque rien), aussi loin que les points le permettent ;
5. finir le tour sans défausser (repos prolongé si la main n'a que des Blessures, comme le tour de remplissage).

Une décision de fin de tour (action avancée d'un niveau pair, spec 050) prend la première carte proposée. Les détails (ordre des
cartes jouées, choix entre deux emplacements) se règlent à l'implémentation : seuls comptent les points d'étape.

### Le plateau de jeu traversé

Cases lues sur `docs/reference/bac-a-sable.md` (même plateau que Première reconnaissance, spec 063) ; les tuiles de campagne de la
pioche sont dans l'ordre 3 à 8 quelle que soit la graine. Le test revérifie le terrain au passage.

| Case | Terrain, site | Rôle dans le parcours |
|---|---|---|
| `(0, 0)` | Plaine, portail | départ |
| `(4, -2)` | Plaine, village (campagne 2, E) | **recrutement** ; on y arrive par `(1, 0)`, `(2, -1)`, `(3, -1)`, sans passer à côté des orques de `(3, -3)` |
| `(4, -3)` | Forêt, clairière magique (campagne 2, N-E) | **exploration de l'emplacement 2A**, voisin par `(5, -3)` et `(5, -4)` |
| `(6, -5)` | Plaine (campagne 3 en 2A, N-O) | voisine de la forteresse : le pas qui y arrive de jour retourne le défenseur |
| `(7, -5)` | Colline, forteresse (campagne 3 en 2A, N-E) | **assaut** et conquête |

La clairière `(4, -3)` est aussi voisine de l'emplacement A + B (par `(4, -4)`) : le parcours choisit 2A, dont les cases d'accès
à la forteresse ne sont pas voisines des orques de `(3, -3)`. Par A + B, chaque entrée dans la tuile passerait d'une case voisine
de ces orques à une autre, et les provoquerait.

### Graine et préconditions

Graine fixe, choisie à l'implémentation (comme les 019, 028, 039), avec un test ignoré `chercher_une_graine` qui liste les graines
qui passent. Le test vérifie chaque précondition juste avant le point d'étape qui en a besoin, avec « graine à rechoisir » sinon
(L1) :

- **Offre d'unités** : une unité recrutable au village, d'un coût que la main peut payer en influence, avec une capacité
  d'Attaque ou de Parade (Paysans, par exemple ; spec 042).
- **Défenseur de la forteresse** : un ennemi gris que la main du tour de l'assaut et l'unité peuvent vaincre (Golems exclus, s'ils
  résistent à tout ce que la main peut jouer).
- **Cité** : révélée avant la fin de la 4e manche (Q1).
- **Mains** : de quoi marcher, payer le recrutement, l'assaut et le combat, sans être assommé (L2).

Héros : **Arythea**, comme les autres parcours.

### Schéma : le parcours

```
new_game (graine fixe, Première reconnaissance, Arythea) ─ offre d'unités avec une unité de village   (C1)
│
├─ Manche 1/4 · Jour
│   ├─ portail → village (4, −2) ; interagir, recruter l'unité                                       (C2, C3)
│   ├─ → clairière (4, −3), explorer 2A : campagne 3, gloire 1, défenseur gris face cachée           (C4)
│   ├─ → (6, −5) : défenseur retourné                                                                (C5)
│   ├─ assaut de la forteresse (7, −5) : Réputation −1                                               (C6)
│   │   combat : unité activée, défenseur vaincu, Bouclier                                           (C7, C8)
│   │   fin du tour : niveau 2, action avancée gagnée, piochée aussitôt                              (C9)
│   │   ★ sauvegarde (après la fin de ce tour)                                                       (C17)
│   ├─ l'action avancée jouée (effet ou de côté)                                                     (C10)
│   └─ tours d'exploration jusqu'au deck vide, annonce
├─ Manche 2/4 · Nuit : unité de nouveau disponible, Bouclier et niveau gardés                        (C11)
├─ tours d'exploration : 1 gloire par tuile, niveaux en fin de tour                                  (C12)
│   └─ cité révélée : dernier tour annoncé                                                           (C13)
│      fin de ce tour, puis dernier tour                                                             (C14)
└─ fin du dernier tour : GameEnded { CityRevealed }, plus aucune décision                            (C15)
   score : gloire + six Hauts Faits, recomptés par le test                                           (C16)
```

Le découpage en tours est indicatif (ADR 0006) : le recrutement et l'exploration peuvent tomber dans le même tour ou non, la marche
vers la forteresse peut prendre un ou deux tours, et l'assaut peut glisser à la manche 2 (de nuit, le défenseur se retourne alors à
l'engagement, spec 043, C17 ; C5 est alors vérifié sur l'engagement). L'ordre qui compte : recrutement avant l'assaut, conquête
avant le niveau 2, niveau 2 avant la cité.

## Actions et événements

Aucun nouveau. Le parcours n'utilise que les actions existantes : `MoveHero` (et les pas de `walk_to`), `PlayCard` (de base, de
côté), `Interact`, `RecruitUnit`, `Explore`, `StartCombat`, `NextCombatPhase`, `DeclareBlock`, `ResolveBlock`, `DeclareAttack`,
`ResolveAttack`, `ActivateUnit`, `AssignDamage`, `EndCombat`, `GainAdvancedAction`, `EndTurn`, `AnnounceEndOfRound` ; et, hors du
jeu, `GameRunner::resume`.

## Comportements attendus

Tous dans `crates/arcanum-core/tests/parcours_j4.rs`. C1 à C16 sont des points d'étape d'**un seul parcours**
(`parcours_premiere_reconnaissance`), vérifiés sur la **vue du joueur** et sur les événements de la décision qui vient d'être
prise. C17 est un second test qui rejoue le même script.

- **C1** — Étant donné `new_game` en Première reconnaissance, la vue donne le scénario, gloire 0, niveau 1, Réputation 0, aucune
  unité, et l'offre d'unités a au moins une unité recrutable au village, aucune d'élite ; aucun score — MKUE p. 18, *Special
  Rules* ; specs 050 (Q3), 063 (C1, C19).
- **C2** — Sur le village `(4, -2)`, `Interact` produit `InteractionStarted` ; les cartes d'influence jouées donnent les points
  d'influence que montre la vue — MKUE p. 8, *Interacting with Locals* ; spec 040.
- **C3** — `RecruitUnit` de l'unité choisie : `UnitRecruited` ; l'unité est dans la vue du joueur, disponible, et n'est plus dans
  l'offre ; l'influence a baissé de son coût — MKUE p. 8, 4.a ; p. 5, *Using Units* 2 ; spec 042.
- **C4** — Exploration de l'emplacement 2A depuis la clairière : `TileRevealed` (campagne 3), `DefenderPlaced` d'un jeton gris
  sur `(7, -5)`, `FameGained { fame: 1 }` ; la vue montre le défenseur face cachée, sans dire lequel — MKUE p. 18 ; carte Keep ;
  specs 043 (C1), 063 (C3).
- **C5** — De jour, le pas qui arrive sur `(6, -5)` produit `EnemyTokenRevealed` ; la vue donne l'ennemi gris du défenseur —
  carte Keep ; spec 043 (C4).
- **C6** — Le pas vers `(7, -5)` produit `AssaultStarted` et `ReputationChanged { change: −1 }` ; puis `StartCombat` ouvre le combat
  contre le défenseur, compté fortifié — MKUE p. 7, 4.a ; p. 8, 1.a ; spec 043 (C11, C15).
- **C7** — Pendant ce combat, l'unité s'active pour une Attaque ou une Parade (`UnitActivated`) et compte dans la parade ou
  l'attaque ; elle est ensuite épuisée dans la vue — MKUE p. 5, *Using Units* 3 ; specs 042, 043 (C19).
- **C8** — Défenseur vaincu, combat fini : `SiteConquered` ; Bouclier sur `(7, -5)` dans la vue, figurine dessus ; la gloire
  compte la tuile et le défenseur — MKUE p. 10, *Combat Outcomes* 2.c ; spec 043 (C20, C21).
- **C9** — Fin de ce tour : `LevelGained { level: 2 }`, puis le choix d'une action avancée de l'offre (1 à 16) ; après
  `GainAdvancedAction`, `OfferCardGained`, la carte est dans la main après la pioche, et l'offre d'actions avancées est complétée —
  MKUE p. 11, *End of the Turn* 6 ; specs 035, 050.
- **C10** — Au tour suivant, l'action avancée gagnée se joue : pour son effet s'il est codé, sinon de côté — MKUE p. 4, 3.e ;
  spec 050.
- **C11** — Au début de la manche 2 (Nuit), l'unité est de nouveau disponible (`UnitsReadied`), le Bouclier, le
  niveau 2 et la gloire sont gardés ; l'offre d'unités est renouvelée sans unité d'élite — MKUE p. 4, 2.g ; specs 042, 050.
- **C12** — Pendant les tours d'exploration, chaque `TileRevealed` du joueur est suivi d'un `FameGained { fame: 1 }` ; un
  `LevelGained` ne vient qu'à une fin de tour — MKUE p. 18 ; p. 11, 6 ; specs 035, 063 (C3, C5).
- **C13** — La tuile royaume de cité révélée : `FameGained`, puis `LastTurnAnnounced` ; le tour continue — MKUE p. 18, *Scenario
  End* ; spec 063 (C7).
- **C14** — Fin de ce tour : pioche et `TurnStarted` comme d'habitude ; la vue dit que ce nouveau tour est le dernier — spec 063
  (C8).
- **C15** — Fin du dernier tour (ou annonce de la fin de la manche, s'il commence deck vide) : `GameEnded { CityRevealed }` ; plus
  aucune action légale, `step` répond `Finished` — MKUE p. 4, 4.c et 4.d ; spec 063 (C9, C10).
- **C16** — La vue donne le score : la gloire de la vue ; Le Savant, 1 par action avancée gagnée (`OfferCardGained` des actions
  avancées du parcours) ; Le Collectionneur, 1 par deux cristaux de l'inventaire ; Le Commandant, le niveau de l'unité (moitié,
  arrondie à l'inférieur, si elle est blessée) ; Le Conquérant, 2 (la forteresse) ; L'Aventurier, 0 ; La Victime, −2 par Blessure
  prise et non soignée ou jetée ; le total. Les valeurs attendues sont **recomptées par le test** depuis les événements du parcours
  et la vue, pas recopiées du score — MKUE p. 15 ; p. 18, *Scoring* ; specs 063 (C14), 042, 043, 050.
- **C17** — Étant donné la même partie sauvegardée après le tour du niveau 2 (journal + positions des générateurs), quand une
  seconde partie est reprise depuis cette sauvegarde avec un nouveau joueur scripté qui joue la suite du même script, alors elle
  finit avec **le même journal**, **le même état** et **le même score** que la partie jouée d'une traite
  (`parcours_reprise_apres_la_conquete`) — ADR 0001 ; specs 024, 038.

## Cas limites et refus

- **L1** — Une graine qui ne remplit plus une précondition fait échouer le test avec « graine à rechoisir », avant le point d'étape
  qui en dépend (comme la 028, L2).
- **L2** — Un héros assommé (spec 034) fausserait le parcours : le test vérifie qu'aucun `KnockedOut` n'est produit pendant l'assaut ;
  les tours d'exploration n'ont pas de combat.
- **L3** — Les tours d'exploration ne provoquent aucun ennemi errant ni n'assaillent de site (trajets de `routes_avoiding`) : le test
  vérifie qu'aucun `EnemiesProvoked` ni `AssaultStarted` n'est produit après C8.
- **L4** — Une récompense de fin de tour inattendue (action avancée d'un niveau 4) est prise par l'explorateur (première carte
  proposée) et comptée par C16 comme les autres.

## Information cachée

Les points d'étape se lisent sur la **vue du joueur** : offres, unités, influence, défenseurs face cachée (couleur seule) puis
retournés, Boucliers, gloire, niveau, dernier tour, score. La composition des piles d'ennemis et l'ordre de la pioche de tuiles
royaumes ne sont pas lus. L'état complet ne sert qu'à comparer les deux parties de C17.

## Libellés (i18n)

Aucun.

## Interface

Aucune. L'écran de chaque spec du J4 a été vérifié à l'œil par sa spec ; pas de test egui (ADR 0006).

### Raccourcis clavier

Aucun.

## Hors périmètre

- **Monastère, ruines, sites d'aventure, sorts, artefacts** : déjà testés par leurs specs (041, 044, 045) ; trop dépendants de la
  graine et de la place des tuiles pour un parcours (Q2). L'Aventurier et les artefacts restent à 0 au décompte.
- **Assaut raté et retraite** (043) : le parcours veut la conquête.
- **Joueur fantôme et tactiques** : J6 (specs 060, 061).
- **Fin par la limite de manches** en Première reconnaissance : déjà parcourue par la 028 (bac à sable) et testée par la 063 (C12) ;
  ici, seulement si Q1 le retient.
- **L'interface** : vérifiée à l'œil par les specs du J4 et par la partie préparée `cite-en-vue` (spec 063).

## Questions ouvertes

Aucune. Décisions de Mathieu (2026-11-06), propositions retenues :

- **Q1 — Fin de la partie** : par la cité révélée (explorateur scripté, précondition « cité avant la fin de la 4e manche ») ; la
  limite de manches reste couverte par la 028.
- **Q2 — Site conquis** : la forteresse de la campagne 3, posée en 2A, avec l'unité recrutée.
- **Q3 — Carte gagnée** : l'action avancée du niveau 2 ; pas de tour de mage.
- **Q4 — Reprise** : oui, depuis une sauvegarde faite après le tour du niveau 2 (C17).
- **Q5 — Rouge d'un parcours** : chaque point d'étape est d'abord écrit avec l'attente d'avant la règle qu'il vérifie, vu en échec
  sur cette assertion (commit `Tests rouges`), puis l'attente est mise juste, et c'est signalé dans la PR (plan, « Le rouge d'un
  parcours »).

## Notes d'implémentation

- **Graine 38** : Paysans dans l'offre d'unités (4 influence, Attaque 2), recrutés au village ; défenseur gris battu par la main et
  les Paysans ; conquête à la manche 1, de jour (C5 vérifié sur le pas) ; cité révélée à la manche 4 ; score total 11. Le test
  ignoré `chercher_une_graine` liste les graines qui passent et la raison de l'échec des autres (26 sur 200 au moment du choix ;
  dans toutes, la cité vient à la manche 4, et la conquête glisse à la manche 2 dans les trois quarts).
- **Unité choisie** : la moins chère des unités recrutables au village qui ont une Attaque au corps à corps sans mana (Paysans,
  Gardes ou Arbalétriers d'Utem selon la graine). Elle s'active pour l'Attaque (C7) ; les dégâts non parés vont au héros, pour que
  Le Commandant compte le niveau plein (C16 sait aussi compter la moitié).
- **Marche et combat** : comme le 039, la marche joue la carte qui donne le plus de Mouvement (renforcée si le mana est là), en
  gardant les cartes d'influence jusqu'au recrutement et les cartes de combat jusqu'à la conquête ; faute de Mouvement pour le pas
  de l'assaut, le tour passe en défaussant une carte. Ces aides restent dans `parcours_j4.rs` : elles diffèrent de celles du 039
  (cartes gardées, mana), elles ne montent pas telles quelles.
- **Support** : `Parcours::explorer_turn` (l'explorateur scripté), `explorable_from` (emplacements où révéler la prochaine tuile
  depuis une case) et `take_pending_reward` (action avancée d'un niveau pair : la première proposée) montent dans
  `tests/support/` pour d'autres parcours. L'explorateur peut explorer plusieurs fois dans le tour si les points le permettent.
- **C10** : l'action avancée se joue au premier tour où elle est en main et où l'annonce de fin de manche n'est pas légale (avec
  la graine 38, le tour qui suit le niveau 2).
- **C11** se vérifie au début de la manche qui suit la conquête (la 2, de Nuit, avec la graine 38). L'offre d'unités renouvelée peut
  contenir l'action avancée d'un monastère révélé (spec 041) : seules les unités y sont dites régulières.
- **Rouge** (plan, « Le rouge d'un parcours ») : chaque étape a été vue rouge sur l'attente d'avant sa règle : C3 (aucune unité,
  avant la 042), C8 (aucun Bouclier, avant la 043) puis C9 (pas d'`OfferCardGained`, avant la 050), C12 (pas de gloire par tuile,
  bac à sable), C16 (Le Conquérant à 0, avant la 043 ; C17 rouge aussi, par le même décompte). Aucun point d'étape n'a été rouge
  pour une autre raison : pas de défaut du moteur trouvé.
