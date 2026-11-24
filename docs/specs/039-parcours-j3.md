# 039 — Parcours du jalon 3

- Statut : implémentée (2026-10-30) ; validée le 2026-10-30 (Q1 à Q3 : propositions retenues par Mathieu) ; départ revu
  (issue privée 322, 2026-11-19) : mise en place arrangée, plus de graine choisie
- Jalon : J3
- Dépend de : 030, 031, 032, 033, 034, 035, 036 (J3, implémentées) ; 028 (joueur scripté, support) ; ADR 0006 (stratégie de tests),
  ADR 0007 (un générateur par usage)
- Livret : MKUE p. 7 (*Movement* 4.b, 4.c), p. 8 (*Combat with Enemies* 1.d, 1.e, 2, 3), p. 9 (*Block Phase* ; *Assign Damage
  Phase* 5.a à 5.c ; *Ranged and Siege Attack Phase* 5.b), p. 10 (*Attack Phase* ; *Combat Outcomes* 1, 2.a), p. 11 (*End of the
  Turn* 6 et 7 ; *Wounds and Healing* 1, 1.a, 1.c, 3)
- Crates touchées : arcanum-core (tests seulement : `crates/arcanum-core/tests/parcours_j3.rs`, et `crates/arcanum-core/tests/support/` si une aide
  y monte)

## Objectif

Vérifier que les specs du J3 **s'enchaînent dans une vraie partie**, comme le promet la ligne « Jouable » du jalon : *combattre les
ennemis errants et monter de niveau* (`PLAN.md`). Rien ne change pour le joueur : la spec n'ajoute que des tests.

Chaque spec du J3 a ses tests sur un état construit à la main (l'arène de `tests/arene/`) ; le parcours 019 ne fait qu'un combat
sans carte. Celui-ci part de la vraie mise en place de `new_game` et mène **deux combats complets** avec les cartes tirées : il attrape ce
que l'arène ne voit pas (des Blessures qui faussent la repioche, une gloire perdue d'un tour à l'autre, un niveau qui ne se gagne
pas, un combat qui ne se reprend pas depuis une sauvegarde).

## Règles

Aucune règle nouvelle : chaque point d'étape renvoie à la règle qu'il vérifie et à la spec qui l'implémente.

- **Jetons d'orques révélés à la mise en place** (MKUE p. 3, *Map* ; carte de site *Marauding Orcs*) : un sur la tuile de campagne 1,
  un sur la tuile 2, tirés de la pile verte : spec 030.
- **Provoquer** (MKUE p. 7, 4.c ; p. 8, 1.e) : un pas d'une case voisine d'un jeton à une autre case voisine du même jeton ; le combat
  est alors obligatoire : specs 030, 031.
- **Défier** (MKUE p. 8, 1.d) : figurine voisine d'un jeton, sans provocation : spec 031.
- **Un combat par tour** (MKUE p. 8, 2) et **pendant le combat, ni Mouvement ni Influence** : spec 031.
- **Phases** (MKUE p. 8, 3) : attaque à distance, parade, dégâts, attaque ; parade réussie ou ratée (p. 9, *Block Phase*), Blessures
  selon l'armure (p. 9, *Assign Damage Phase* 5.a à 5.c), victoire et gloire (p. 9, 5.b ; p. 10) : specs 031, 032, et 033 pour la
  capacité de l'ennemi tiré.
- **Case libérée** (MKUE p. 10, *Combat Outcomes* 1) : le jeton vaincu quitte le plateau de jeu, sa case ne provoque plus : spec 031.
- **Blessures en main** (MKUE p. 11, *Wounds and Healing* 1, 1.a) : jamais jouées ni défaussées en fin de tour, elles comptent
  dans la main pour la repioche (p. 11, 7.b) : spec 031.
- **Soin** (MKUE p. 11, *Wounds and Healing* 3) : Tranquillité « Soin 1 » hors combat : spec 034.
- **Niveau** (MKUE p. 9, 5.b ; p. 11, 6) : gagné à la fin du tour où la gloire franchit une ligne, pas avant : spec 035.
- **Nouvelle manche** (MKUE p. 11, *Wounds and Healing* 1.c ; p. 4, 2.g) : les Blessures de la main et de la défausse sont
  remélangées avec le deck, le niveau est gardé : specs 023, 031, 035.

## Modèle

Aucun changement de l'état ni du glossaire. Le parcours **réutilise** le support des parcours (`crates/arcanum-core/tests/support/mod.rs`) :
`Parcours` (`play`, `view`, `legal`, `save`, `resume`, `undo`), le joueur scripté, `walk_to`, `in_hand`, `first_playable`,
`finish_round`. Les petites aides de combat (jouer une carte pour une option, déclarer une parade ou une attaque, passer à la phase
suivante) restent dans `parcours_j3.rs`, sauf si la 019 en a déjà une équivalente : elle monte alors dans `support/`.

### Le plateau de jeu traversé

Cases lues sur `docs/reference/bac-a-sable.md` (ne dépendent pas de la graine) ; le test revérifie le terrain au passage.

| Case | Terrain, site | Rôle dans le parcours |
|---|---|---|
| `(0, 0)` | Plaine, portail | départ |
| `(2, -3)` | Plaine, village | étape vers le premier combat ; voisine des orques de la tuile 2 |
| `(2, -2)` | Plaine | voisine des orques de la tuile 2 : le pas `(2, -3)` → `(2, -2)` les provoque |
| `(3, -3)` | Colline, orques (tuile 2) | **combat 1**, provoqué |
| `(1, -3)` | Forêt, clairière magique | voisine des orques de la tuile 1 : on les défie de là |
| `(1, -4)` | Forêt, orques (tuile 1) | **combat 2**, défié |

### Départ et préconditions (révision, issue privée 322)

La version d'origine partait de `new_game` avec une graine fixe : 6 graines sur 400 faisaient tenir les préconditions
(mesuré le 2026-11-17). Le parcours part maintenant d'une **mise en place arrangée** (ADR 0006, « Départ d'un parcours » ;
`tests/support/arrangement.rs`, `Setup`), comme la 059 ; **la graine n'est plus choisie**. Les points d'étape C1 à C15 ne
changent pas.

- **Pile verte** : des Rôdeurs puis des Creuseurs sur le dessus, les errants tirés à la mise en place sur `(1, -4)` (tuile 1)
  puis `(3, -3)` (tuile 2). Creuseurs au combat 1 : 2 de gloire, le niveau ne se gagne qu'au combat 2 ; ni l'un ni l'autre
  n'est un Invocateur.
- **Deck du héros** et **dés de la Source** : l'ordre complet et les trois dés, choisis à l'implémentation pour que les deux
  combats, le soin entre les deux et le niveau tiennent dans la **première manche, de jour** (la seule que l'arrangement
  contrôle) : de quoi payer la marche, vaincre chaque ennemi en phase d'attaque, parer l'ennemi du combat 2 et jouer
  Tranquillité avec une Blessure en main.
- **Figurine** : au portail, comme à l'origine (la marche jusqu'au village fait partie du parcours).

Le moteur joue tout le reste. La fin de la manche 1 (tours de remplissage) et le début de la manche 2 dépendent encore de la
graine, sans précondition propre. Le test vérifie encore chaque précondition juste avant le point d'étape qui en a besoin,
avec « départ à revoir » sinon. Le test ignoré `balayer_les_graines` (`support::sweep_seeds`) joue le parcours sur les graines
0 à 99 : **au moins 90 doivent passer**, seuil de la 059.

Les ennemis arrangés sont nommés dans le test, et les Blessures et la gloire attendues y sont écrites en chiffres, calculées
depuis leurs valeurs (`enemies.ron`). Héros : **Arythea**, comme les 019 et 028.

### Schéma : le parcours

```
mise en place arrangée (bac à sable, Arythea) ─ orques révélés sur les tuiles 1 et 2          (C1)
│
├─ Manche 1/4 · Jour
│   ├─ portail → village (2, −3)                                                               
│   ├─ village → (2, −2) : orques de la tuile 2 provoqués, combat obligatoire                 (C2)
│   │   combat 1 : pas d'attaque à distance, pas de parade ─ Blessures en main                (C3, C4)
│   │              phase d'attaque : ennemi vaincu, gloire 2, case libérée                    (C5)
│   │   fin du tour : Blessures gardées en main, pas de niveau                                (C6)
│   ├─ Tranquillité « Soin 1 » : une Blessure de moins                                        (C7)
│   ├─ → clairière (1, −3) en repassant par (2, −3), sans provocation                         (C8)
│   ├─ défi des orques de la tuile 1                                                          (C9)
│   │   combat 2 : parade réussie, aucune Blessure                                            (C10)
│   │              ★ sauvegarde                                                               (C15)
│   │              phase d'attaque : vaincu, la gloire franchit la ligne, niveau toujours 1   (C11)
│   │              revenir en arrière, rejouer                                                (C12)
│   │   fin du tour : niveau 2                                                                (C13)
│   └─ tours de remplissage jusqu'au deck vide, annonce
└─ Manche 2/4 · Nuit : Blessures remélangées dans le deck, niveau 2 gardé                     (C14)
```

Le découpage en tours est indicatif (ADR 0006) : le soin (C7) peut tomber dans le tour de la marche ou dans celui du défi, et la
marche vers la clairière peut prendre un ou deux tours, selon la main. L'ordre qui compte : combat 1 fini avant le soin, soin avant
le combat 2, combat 2 avant la fin de manche.

## Actions et événements

Aucun nouveau. Le parcours n'utilise que les actions existantes : `MoveHero` (et `FollowRoute` par `walk_to`), `PlayCard` (de base, de
côté), `Challenge`, `StartCombat`, `NextCombatPhase`, `DeclareBlock`, `DeclareAttack`, `EndCombat`, `EndTurn`,
`AnnounceEndOfRound` ; et, hors du jeu, `GameRunner::undo` et `GameRunner::resume`.

## Comportements attendus

Tous dans `crates/arcanum-core/tests/parcours_j3.rs`. C1 à C14 sont des points d'étape d'**un seul parcours**
(`parcours_deux_combats_et_un_niveau`), vérifiés sur la **vue du joueur** et sur les événements de la décision qui vient d'être
prise. C15 est un second test qui rejoue le même script.

- **C1** — Étant donné la mise en place, la vue montre un jeton d'orques sur `(1, -4)` et un sur `(3, -3)`, face visible ; niveau 1,
  gloire 0, armure 2 — MKUE p. 3 ; specs 030, 035 (C1).
- **C2** — Depuis le village, quand le héros fait le pas vers `(2, -2)`, alors `EnemiesProvoked` nomme les orques de la tuile 2 ;
  aucune fin de tour n'est légale, `StartCombat` l'est, et aucune action de Mouvement ni d'Influence — MKUE p. 7, 4.c ; p. 8, 1.e ;
  specs 030, 031 (C1, C23).
- **C3** — Combat 1, phase d'attaque à distance : aucune carte d'Attaque de mêlée n'est légale ; le héros passe à la phase de parade
  sans jouer — MKUE p. 9, *Ranged and Siege Attack Phase* ; spec 031 (C8).
- **C4** — Sans parade, en passant à la phase suivante : `WoundsTaken` donne le nombre de Blessures attendu (attaque de l'ennemi,
  armure 2, tranche entamée), et elles sont dans la main — MKUE p. 9, *Assign Damage Phase* 5.a à 5.c ; spec 031 (C16, C17).
- **C5** — Phase d'attaque : les cartes jouées (de base et de côté) atteignent l'armure, `EnemiesDefeated` rapporte 2 de gloire ; après
  `EndCombat`, la case `(3, -3)` n'a plus de jeton, la gloire vaut 2, le niveau 1 et aucun niveau n'est annoncé — MKUE p. 9, 5.b ;
  p. 10, *Combat Outcomes* 1 ; specs 031 (C19, C22), 035.
- **C6** — Fin du tour du combat 1 : aucun `LevelGained` ; les Blessures restent en main et la repioche complète la main à 5 en les
  comptant — MKUE p. 11, 6 et 7.b ; *Wounds and Healing* 1.a ; specs 031 (C28), 035 (C8).
- **C7** — Hors combat, avec au moins une Blessure en main, Tranquillité jouée pour « Soin 1 » : `WoundsHealed` ; la main a une Blessure
  de moins — MKUE p. 11, *Wounds and Healing* 3 ; spec 034 (C1).
- **C8** — Le héros marche jusqu'à la clairière `(1, -3)` par le village `(2, -3)` : le pas `(2, -2)` → `(2, -3)`, entre deux cases
  voisines de l'ennemi vaincu, ne provoque rien — MKUE p. 7, 4.c ; spec 031 (C22).
- **C9** — Sur la clairière, sans provocation, `Challenge` des orques de la tuile 1 est légal ; il produit `EnemyChallenged`, puis
  `StartCombat` ouvre le combat 2 — MKUE p. 8, 1.d ; spec 031 (C2).
- **C10** — Combat 2, phase de parade : les cartes de Parade jouées atteignent la parade à atteindre que montre la vue (le double de
  l'attaque si l'ennemi est vif) ; `EnemyBlocked` ; en passant à la phase suivante, aucun `WoundsTaken` et la phase de dégâts est
  sautée — MKUE p. 9, *Block Phase* ; specs 031 (C12, C18), 033 (C1).
- **C11** — Phase d'attaque : ennemi vaincu ; la gloire franchit 3 ; pendant le reste du tour le niveau vaut toujours 1, l'armure 2,
  et la vue annonce le niveau 2 en fin de tour — MKUE p. 9, 5.b ; spec 035 (C3).
- **C12** — Revenir en arrière juste après cette victoire rend la gloire d'avant et l'annonce du niveau disparaît ; rejouer la même
  attaque redonne la même victoire — MKUE p. 5, *Reverting* 1 ; specs 024, 035 (C12).
- **C13** — Fin du tour du combat 2 : `LevelGained { level: 2 }` après `TurnEnded` et avant les `CardDrawn` ; niveau 2, armure 2,
  limite de main 5 — MKUE p. 11, 6 ; spec 035 (C4).
- **C14** — Après les tours de remplissage jusqu'au deck vide et l'annonce, la manche 2 commence : Nuit, le deck et la main
  comptent les 16 cartes plus toutes les Blessures gardées (en main ou en défausse), la main a 5 cartes, niveau 2 et gloire gardés —
  MKUE p. 11, *Wounds and Healing* 1.c ; p. 4, 2.g ; specs 023, 031 (C29), 035 (C11).
- **C15** — Étant donné la même partie sauvegardée en plein combat 2, après la parade (journal + positions des générateurs), quand
  une seconde partie est reprise depuis cette sauvegarde avec un nouveau joueur scripté qui joue la suite du même script, alors elle
  arrive au début de la manche 2 avec **le même journal** et **le même état** que la partie jouée d'une traite
  (`parcours_reprise_en_plein_combat`) — ADR 0001 ; spec 024.

## Cas limites et refus

- **L1** — Un départ qui ne remplit plus une précondition fait échouer le test avec « départ à revoir », avant le point d'étape
  qui en dépend (comme la 059, L1) : on revoit l'arrangement, pas la graine.
- **L2** — Un héros assommé (spec 034) ferait échouer le parcours : les préconditions l'excluent (2 à 4 Blessures au combat 1, aucune
  au combat 2) ; le test vérifie qu'aucun `KnockedOut` n'est produit.
- **L3** — Les tours de remplissage gardent leur règle (spec 028) avec les Blessures : une Blessure n'est jamais jouée, et une main
  faite de Blessures sans carte jouée finit par un repos prolongé (le support le fait déjà).

## Information cachée

Les points d'étape se lisent sur la **vue du joueur** : jetons d'ennemis visibles, combat (ennemis, phase, parade à atteindre),
main, défausse, gloire, niveau et niveau annoncé. La composition de la pile verte n'est pas lue. L'état complet ne sert qu'à comparer
les deux parties de C15.

## Libellés (i18n)

Aucun.

## Interface

Aucune.

### Raccourcis clavier

Aucun.

## Hors périmètre

- **Attaque à distance réussie, éléments et résistances** (032), **invocation** (033), **assommé** (034), **repos standard** : déjà
  testés sur l'arène, et trop dépendants de la graine pour un parcours. C3 ne vérifie que la phase à distance passée sans jouer.
- **Niveau 3 et au-delà** (armure 3, jetons de commandement) : il faudrait un troisième combat (Q1).
- **Clairière magique** (MKUE p. 11, 4.a) : son effet en fin de tour arrive avec la spec 044. Si le parcours finit un tour sur
  `(1, -3)`, la 044 adaptera ce point (choix de ne pas jeter de Blessure).
- **L'interface du combat** : vérifiée à l'œil par les specs 031 à 035 ; pas de test egui (ADR 0006).
- **La fin de la partie** : déjà parcourue par la 028 ; ce parcours s'arrête au début de la manche 2 (Q2).

## Questions ouvertes

Aucune. Décisions de Mathieu (2026-10-30) :

- **Q1 — Jusqu'à quel niveau ?** Le niveau 2, au second combat. Le niveau 3 (troisième combat, tuile à explorer) reste hors périmètre.
- **Q2 — Où s'arrête le parcours ?** Au début de la manche 2 (Blessures remélangées, niveau gardé) ; la fin de partie est déjà
  parcourue par la 028.
- **Q3 — Soin dans le parcours ?** Oui : Tranquillité « Soin 1 » entre les deux combats (C7).

## Notes d'implémentation

- **Départ arrangé** (issue privée 322, remplace la graine 135 et `chercher_une_graine`, 6 graines sur 400) : Rôdeurs puis Creuseurs sur
  le dessus de la pile verte (combat 1 contre les Creuseurs, deux Blessures ; combat 2 contre les Rôdeurs, parade 4, gloire 2 → 4).
  Deck d'Arythea, dans l'ordre : Marche, Endurance, Célérité, Rage, Tranquillité, Endurance, Promesse, Menace, Marche, Célérité,
  Détermination, Polyvalence, Improvisation, Concentration, Cristallisation, Attraction de mana. Source : un dé rouge (Rage
  renforcée, attaque 4, au combat 1) et deux blancs. `balayer_les_graines` : **100 graines sur 100** (0 à 99).
- **Joueur scripté sans plan fixe** : la marche joue d'abord les cartes de Mouvement, de côté les autres, en gardant les cartes de
  combat et Tranquillité ; en combat, chaque carte jouée est celle qui rapporte le plus de points par carte quittant la main.
  Les points d'étape ne dépendent donc pas de la main tirée, seules les préconditions.
- **Découpage en tours** (indicatif, ADR 0006) avec ce départ : marche jusqu'au village (tour 1), combat 1 (tour 2), Tranquillité
  puis retour au village (tour 3), marche vers la clairière, où le tour finit (tour 4), défi et combat 2 (tour 5, Détermination
  renforcée par le jeton d'or de la clairière).
- Aucune aide n'est montée dans `tests/support/` : celles du parcours (marche sur plusieurs tours, cartes de combat) ne servent
  qu'à lui.
