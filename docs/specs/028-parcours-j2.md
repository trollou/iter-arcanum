# 028 — Parcours du jalon 2

- Statut : implémentée (2026-10-27) ; validée le 2026-10-27 (Q1 et Q2 : propositions retenues par Mathieu)
- Jalon : J2
- Dépend de : 012, 013, 014, 015, 016 (J1, déjà implémentées), 020, 021, 022, 023, 024, 025 (J2) ; ADR 0006 (stratégie de tests)
- Livret : MKUE p. 4 (*One Round* 2 et 5), p. 5 (*Using Mana* 1 à 3 ; *"Gain" Effects* 1 ; *Reverting* 1, 2 et 2.a), p. 6 (3 et 4),
  p. 11 (*End of the Turn* 1, 3.a et 4.b)
- Crates touchées : arcanum-core (tests seulement : `crates/arcanum-core/tests/parcours_j2.rs` et `crates/arcanum-core/tests/support/`)

## Objectif

Vérifier que les specs du J2 **s'enchaînent dans une vraie partie**, de `new_game` à la fin de la partie, comme le promet la ligne
« Jouable » du jalon : *plusieurs manches de déplacement avec la mana, sauvegarde et annulation* (`PLAN.md`). Rien ne change pour le
joueur : la spec n'ajoute que des tests, plus le **joueur scripté** qui les joue (voir Q1).

Chaque spec du J2 a déjà ses tests sur un état construit à la main. Ce parcours, lui, part de la mise en place réelle avec une graine
fixe et ne construit jamais d'état : il attrape ce que ces tests ne voient pas (une manche qui ne repart pas, un cristal perdu au
changement de manche, un dé jamais rendu, une reprise de partie qui diverge).

## Règles

Aucune règle nouvelle : chaque point d'étape renvoie à la règle qu'il vérifie (colonne « Livret » des cas) et à la spec qui
l'implémente.

- **Effet renforcé payé** (MKUE p. 4, *Deed Cards* 3.a ; p. 5, *Using Mana* 1 à 3) : par un dé de la Source, un cristal ou un jeton
  de mana (specs 022, 025).
- **Cristal gagné sur une mine** (MKUE p. 11, *End of the Turn* 4.b) : spec 021.
- **Jeton gagné** (MKUE p. 5, *"Gain" Effects* 1), rendu à la réserve en fin de tour (p. 11, 3.a) : spec 025.
- **Dé rendu et relancé** en fin de tour (MKUE p. 5, *Using Mana* 3.b ; p. 11, 1) : spec 022.
- **Fin de manche, Nuit, fin de partie** (MKUE p. 6, 3 et 4 ; p. 4, *One Round* 2 et 5) : spec 023. Le bac à sable dure 4 manches.
- **Dés épuisés de nuit** (MKUE p. 5, *Using Mana* 3.c) : spec 020.
- **Revenir en arrière, points de non-retour** (MKUE p. 5, *Reverting* 1, 2 et 2.a) : spec 024.
- **Reprendre une partie** (ADR 0001, spec 024) : journal + position du générateur.

## Modèle

Aucun changement de l'état ni du glossaire. Deux outils de test, dans `crates/arcanum-core/tests/support/` (partagés par les futurs
parcours, dont la 019) :

- **Joueur scripté** (ADR 0006) : un `Player` à qui le test donne **l'action voulue, par sa valeur** (`Action`). À la demande, il la
  cherche dans les actions légales et répond son index. Si elle n'y est pas, le test échoue en affichant l'action voulue et les
  actions légales. Jamais d'index écrit dans le test : l'ordre des actions légales peut changer sans casser le parcours.
- **Tour de remplissage** : pour les tours qui ne vérifient rien, mais qu'il faut jouer pour avancer dans la partie. Une manche ne
  finit que quand le deck est vide (spec 023) : il faut donc jouer, à la fin de la manche 1 et de la manche 2, les tours qui restent
  après les points d'étape (C8), puis toutes les manches 3 et 4 pour atteindre la fin de partie (C11), soit une vingtaine de tours.
  Les écrire action par action allongerait le test sans rien vérifier de plus. Un tour de remplissage applique une règle fixe : si
  l'annonce de fin de manche est légale, il annonce ; sinon il joue la première carte de sa main de côté (Mouvement 1), puis finit
  son tour sans défausser.

Petites fonctions d'aide à côté (trouver une carte d'une sorte dans la main, la vue courante, jouer une action et vérifier qu'elle a
été acceptée).

### Graine et préconditions

La graine est **fixe**, choisie à l'implémentation pour que la main de départ et la Source permettent le parcours (un dé de la
couleur d'une carte en main, Concentration et une carte bleue ou blanche dans une même main, un dé or dans
la Source de la manche 2…). Le test **vérifie chaque précondition juste avant le point d'étape qui en a besoin** (certaines ne se
voient qu'en cours de partie, comme la Source de la manche 2) et, si elle ne tient plus, échoue avec un message clair : « graine à rechoisir ». Cela arrive si une spec change
l'ordre d'usage du générateur ; ce n'est pas une régression des règles. Le héros est **Arythea**, celui de la partie lancée par
l'interface (`default_config`).

### Schéma : le parcours

```
new_game (graine fixe, bac à sable, Arythea)
│
├─ Manche 1/4 · Jour
│   ├─ tour A : effet renforcé payé par un dé ──── revenir en arrière ─── rejouer          (C1, C2)
│   │           se déplacer ─ revenir ─ se déplacer ; explorer ─ revenir : refusé          (C3, C4)
│   │           finir le tour : dé rendu et relancé ; revenir : refusé                     (C5)
│   ├─ tour B…: aller sur la mine verte (tuile 2), finir le tour dessus ─ cristal vert     (C6)
│   ├─ tour C : Concentration ─ jeton ─ effet renforcé payé par le jeton                   (C7)
│   └─ tours de remplissage jusqu'au deck vide, annonce                                    (C8)
├─ Manche 2/4 · Nuit : Source relancée, dés or épuisés, cristal gardé                      (C9)
│   └─ effet renforcé payé par le cristal vert gagné de jour                               (C10)
│   └─ ★ sauvegarde au milieu d'un tour (journal + position du générateur)                 (C12)
├─ Manche 3/4 · Jour, Manche 4/4 · Nuit : tours de remplissage                             (C11)
└─ fin de partie : plus de décision
```

L'ordre des tours A, B, C peut changer selon la graine (par exemple, le jeton avant la mine) : seuls les points d'étape comptent ; le découpage en tours est indicatif (ADR 0006).

## Actions et événements

Aucun nouveau. Le parcours n'utilise que les actions existantes : `PlayCard` (de côté, renforcé payé par `ManaPayment::Die`,
`Crystal` ou `Token`, Concentration de base), `MoveHero`, `Explore`, `EndTurn`, `AnnounceEndOfRound` ; et, hors du jeu,
`GameRunner::undo` et `GameRunner::resume`.

## Comportements attendus

Tous dans `crates/arcanum-core/tests/parcours_j2.rs`. Les cas C1 à C11 sont des points d'étape d'**un seul parcours**
(`parcours_quatre_manches_avec_la_mana`), vérifiés sur la **vue du joueur** et sur les événements de la décision qui vient d'être
prise, pas sur le journal complet. C12 et C13 sont deux autres tests qui rejouent le même script.

- **C1** — Étant donné la mise en place, quand le joueur joue une carte pour son effet renforcé en payant avec un dé de la Source de
  sa couleur (ou or, de jour), alors l'effet renforcé est obtenu (points de mouvement), le dé apparaît dans les dés
  pris et aucun autre dé ne peut être pris ce tour (sans carte qui en donne un de plus) — MKUE p. 5, *Using Mana* 3.a ; spec 022.
- **C2** — Quand il revient en arrière, alors la carte revient en main, le dé revient dans la Source et les points retombent ; il
  rejoue la même action et retrouve le même résultat — MKUE p. 5, *Reverting* 1 ; spec 024.
- **C3** — Quand il se déplace puis revient en arrière, alors la figurine est revenue et les points rendus — MKUE p. 5, *Reverting*
  1 ; spec 024.
- **C4** — Quand il explore, alors une tuile est révélée et revenir en arrière est refusé (information révélée), même pour les
  actions d'avant l'exploration — MKUE p. 5, *Reverting* 2 ; spec 024.
- **C5** — Quand il finit son tour, alors le dé pris est rendu à la Source et relancé (toujours 3 dés), il repioche jusqu'à 5 cartes,
  un nouveau tour commence, et revenir en arrière est refusé (rien depuis le début du tour) — MKUE p. 5, *Using Mana* 3.b ; p. 11,
  1 ; spec 016, 022, 024.
- **C6** — Quand il finit un tour sur la mine verte (révélée à la mise en place sur la tuile de campagne 2), alors il gagne un
  cristal vert, visible dans son inventaire — MKUE p. 11, *End of the Turn* 4.b ; spec 021.
- **C7** — Quand il joue Concentration pour son effet de base (un jeton bleu ou blanc), puis une carte de cette couleur pour son
  effet renforcé payé par ce jeton, alors l'effet renforcé est obtenu, le jeton a disparu de la zone de jeu, et à la fin du tour il
  ne reste aucun jeton — MKUE p. 5, *"Gain" Effects* 1 ; p. 11, 3.a ; spec 025.
- **C8** — Quand son deck est vide au début d'un tour et qu'il annonce la fin de la manche, alors la manche 1 finit et la manche 2
  commence : Nuit, tour 1, 16 cartes remélangées, main de 5 — MKUE p. 6, 3 et 4 ; p. 4, *One Round* 2 ; spec 023.
- **C9** — Au début de la manche 2, la Source a été relancée (3 dés) et, bien qu'elle ait un dé or, aucune action légale
  ne paie avec lui, et le cristal vert
  gagné de jour est toujours dans l'inventaire — MKUE p. 4, *One Round* 2.b ; p. 5, *Using Mana* 3.c ; spec 020, 023.
- **C10** — De nuit, quand il joue une carte verte pour son effet renforcé en payant avec ce cristal, alors l'effet renforcé est
  obtenu et l'inventaire n'a plus de cristal vert — MKUE p. 5, *Using Mana* 1.b ; spec 022.
- **C11** — Après les tours de remplissage des manches 3 (Jour) et 4 (Nuit), quand il annonce la fin de la manche 4, alors la partie
  est finie : plus aucune décision en attente, `step` répond `Finished` — MKUE p. 4, *One Round* 5 ; spec 023.
- **C12** — Étant donné la même partie sauvegardée au milieu d'un tour de la manche 2, après une carte jouée (journal + position du
  générateur), quand une seconde partie est reprise depuis cette sauvegarde avec un nouveau joueur scripté qui joue la suite du même
  script, alors elle finit avec **le même journal** et **le même état** que la partie jouée d'une traite
  (`parcours_reprise_identique`) — ADR 0001 ; spec 024.
- **C13** — Juste après la reprise de C12, revenir en arrière est possible et remet la carte jouée en main
  (`parcours_annulation_apres_reprise`) — MKUE p. 5, *Reverting* 1 ; spec 024.

## Cas limites et refus

- **L1** — Une action voulue absente des actions légales fait échouer le test en nommant l'action et en listant les actions légales
  (test du joueur scripté lui-même, `scripted_player_reports_illegal_action`).
- **L2** — Une graine qui ne remplit plus les préconditions fait échouer le test avec « graine à rechoisir », avant le point d'étape
  qui en dépend.
- **L3** — Le parcours ne doit pas dépendre de l'ordre des actions légales : le joueur scripté ne mémorise jamais un index d'une
  décision à l'autre.

## Information cachée

Les points d'étape se lisent sur la **vue du joueur** (`view`), comme le ferait un joueur : main, Source, dés pris, inventaire,
jetons, manche, tour, moment de la journée. L'état complet ne sert qu'à comparer les deux parties de C12.

## Libellés (i18n)

Aucun.

## Interface

Aucune.

### Raccourcis clavier

Aucun.

## Hors périmètre

- **Le fichier de sauvegarde.** Le format JSON, l'écriture et la lecture sont testés par `crates/arcanum-persistance/tests/sauvegarde.rs`
  (aller-retour, reprise qui tire le même hasard) ; ce parcours reprend la partie en mémoire (voir Q2).
- **L'interface** (boutons Revenir en arrière, Sauvegarder, Charger) : vérifiée à l'œil par la spec 024 ; pas de test egui (ADR 0006).
- **Les coûts de déplacement de nuit** et Cristallisation, Source de mana, Attraction de mana, Pouvoir de concentration : déjà testés
  par leurs specs ; le parcours ne fait que traverser la nuit.
- **Le parcours du J1** (spec 019) : il réutilisera le joueur scripté, avec sa propre spec.

## Questions ouvertes

Aucune. Décisions de Mathieu (2026-10-27) :

- **Q1 — Qui apporte le joueur scripté ?** La 028, dans `crates/arcanum-core/tests/support/` ; la 019, rédigée ensuite, le réutilise.
  L'ADR 0006 et `PLAN.md` disent maintenant « la première spec de parcours l'apporte ».
- **Q2 — Sauvegarde : en mémoire ou par un fichier ?** En mémoire (`runner.log().clone()` et `rng_position()`, puis
  `GameRunner::resume`) : le test reste dans arcanum-core, le fichier est déjà testé par arcanum-persistance.
