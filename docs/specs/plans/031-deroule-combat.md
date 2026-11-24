# Plan de développement — spec 031 Déroulé d'un combat

- Statut : accepté par Mathieu avec la spec (2026-10-29).
- Spec : [`031-deroule-combat.md`](../031-deroule-combat.md). Branche : `spec/031-deroule-combat`.
- Méthode : celle de `CLAUDE.md` (tests rouges, code minimal, refactor ; `scripts/verifier.sh` vert avant chaque push).

## Découpage en deux PR

La spec est la plus grosse depuis la 025 (40 cas, 13 événements, un nouvel écran). Comme pour la 025, deux PR fusionnables
l'une après l'autre, chacune verte et utilisable :

| PR | Contenu | Cas | Jouable après fusion ? |
|---|---|---|---|
| **A — moteur** | Blessures, défier, engager, les quatre phases, fin du combat, défausses d'ennemis, vue, journal, retour en arrière ; tests des specs déjà implémentées ajustés | C1 à C34 | Le moteur combat ; l'interface garde le bandeau de la 030, avec seulement « Engager le combat », « Phase suivante » et « Terminer le combat » pour ne jamais bloquer une partie (pas de nouveau visuel) |
| **B — interface** | Écran de combat, groupement, main en bas, Blessures, gloire, case vide, raccourci Entrée, libellés, vérification à l'œil | C35 à C40 | Oui, entièrement |

La PR A garde l'interface minimale pour qu'un provoqué ne bloque pas la partie : sans ces trois boutons, la fin du tour serait
impossible (C1). C'est la seule entorse à « rien hors spec » : ces boutons sont dans la spec, seule leur place change en PR B.

## Choix de modèle par étape

Mathieu demande d'économiser les modèles les plus chers là où il y a peu à réfléchir. Proposition :

- **Opus** (le fil principal) : ce qui fixe la forme du code et ce qui se trompe facilement sur les règles. Il écrit le brief de
  chaque étape déléguée, relit chaque diff et lance `scripts/verifier.sh` avant chaque push.
- **Sonnet** : étapes qui suivent un modèle déjà posé par Opus (nouvelle phase sur le patron de la première, écran sur le patron de
  `game_screen`), tests écrits à partir des cas de la spec.
- **Haiku** : étapes mécaniques, sans décision : libellés `fr.json`/`en.json`, champs ajoutés aux `PlayerView` des tests, docs
  (architecture, raccourcis, index, PLAN), captures sous Xvfb.

Une étape déléguée est lancée comme sous-agent (outil `Agent`, paramètre `model`), sur la branche du fil, avec un brief court :
cas de la spec à couvrir, fichiers à toucher, patron à suivre, ce qu'il ne faut pas faire (pas de nouvelle dépendance, pas de test
modifié sans le dire). Le sous-agent rend un diff, pas un push. Si une étape déléguée revient deux fois fausse, Opus la reprend.

## Étapes

Chaque étape = un commit de tests (rouges, local) puis un commit de code (vert), sauf mention. Ordre imposé par les dépendances.

### PR A — moteur (arcanum-core)

| # | Étape | Cas | Fichiers | Modèle | Pourquoi ce modèle |
|---|---|---|---|---|---|
| A1 | **Carte Blessure** : `DeedCard::Wound`, numéros d'exemplaire, jamais jouable ni défaussable (fin du tour, coût d'Improvisation), mélangée à la nouvelle manche | C26 à C29 | `card.rs`, `ids.rs`, `engine.rs` (`card_actions`, `playable_options`, `end_turn_actions`) | **Opus** | `DeedCard::definition()` est appelée à 23 endroits hors tests et suppose une carte d'action : choisir comment une Blessure y répond (définition sans option jouable, ou `Option`) touche tout le moteur |
| A2 | **Modèle du combat** : module `combat.rs` (état, phase, ennemis et leur état, attaque ou parade en cours, « déjà ciblé »), gloire, défausses d'ennemis ; `Challenge`, `StartCombat` et leurs événements ; `apply`, `blocks_undo`, `filter_event`, `TurnEnded` qui vide le combat ; actions légales autour de la provocation | C1 à C5 | nouveau `combat.rs`, `action.rs`, `event.rs`, `state.rs`, `engine.rs` | **Opus** | Pose les types et le patron que suivent A3 à A5 ; ordre des actions légales = contrat |
| A3 | **Attaques** (phase d'attaque à distance et phase d'attaque) : `DeclareAttack` (sous-ensembles, fortifié, une seule attaque par ennemi et par phase), cartes jouables pendant une attaque en cours, `ResolveAttack`, `EnemiesDefeated` (jeton à la défausse, gloire), `AttackFailed` | C6 à C11 bis, C19, C20, C23 | `combat.rs`, `engine.rs` | **Sonnet** | Suit le patron de A2 ; les règles sont dans les cas |
| A4 | **Parade et dégâts** : `DeclareBlock`, `ResolveBlock`, une seule parade par ennemi, `NextCombatPhase`, `WoundsTaken` (arrondi par ennemi), phase sautée sans dégâts ; règles ignorées jusqu'à 032/033 (invocateur, éléments) | C12 à C18, C14 bis, C24, C25 | `combat.rs`, `engine.rs` | **Sonnet** | Même patron qu'A3 |
| A5 | **Fin du combat** : `EndCombat`, case vide après victoire (déplacement et provocation), remélange d'une pile vide (`EnemyPileReshuffled`, générateur de la pile, spec 038) | C21, C22, C30 | `combat.rs`, `engine.rs`, `exploration.rs` | **Sonnet** | Petit, mais le remélange touche au hasard : relecture Opus attentive |
| A6 | **Vue, journal, rejeu, retour en arrière** | C31 à C34 | `view.rs`, `event.rs`, `log.rs` (tests surtout) | **Sonnet** | Tests sur des mécanismes existants |
| A7 | **Tests des specs déjà implémentées** (030 C16, C18, C26 ; 013 ; 016) et `PlayerView` construits champ par champ dans arcanum-ui ; boutons minimaux de la PR A dans `session.rs` / `app.rs` | — | `crates/arcanum-core/tests/ennemis_errants.rs`, `crates/arcanum-ui/tests/*`, `crates/arcanum-ui/tests/support/` | **Haiku** pour les champs et les libellés, **Sonnet** pour les trois tests de la 030 | Les tests de la 030 changent de sens : chaque changement est listé dans la PR, comme l'exige `CLAUDE.md` |
| A8 | **Refactor et docs** : relire `engine.rs` (déjà 1 068 lignes : ce qui est combat doit vivre dans `combat.rs`), `docs/architecture.md` (module, recette), PLAN (statut) | — | — | **Opus** pour le refactor, **Haiku** pour les docs | |

Fin de PR A : `scripts/verifier.sh` vert, PR ouverte, CI (si quota revenu) ou accord de Mathieu pour fusionner.

### PR B — interface (arcanum-ui)

| # | Étape | Cas | Fichiers | Modèle | Pourquoi ce modèle |
|---|---|---|---|---|---|
| B1 | **Libellés** `fr.json` et `en.json` (tableau de la spec) | — | `assets/i18n/*.json` | **Haiku** | Recopie du tableau |
| B2 | **`combat_screen`** : fonction pure, depuis la vue seule, de l'écran de la maquette (phases, cartes d'ennemis, attaque ou parade en cours, historique « Dans ce combat », main en grand) ; `ScreenMode` de sélection des cibles ; « Voir le plateau » | C35 à C37, C36 bis | nouveau `screen/combat.rs`, `screen/types.rs`, `session.rs` | **Opus** pour la forme de `CombatScreen` et l'historique (à déduire de la zone de jeu et du journal filtré), **Sonnet** pour le remplir | L'historique par ennemi est le seul vrai choix de conception de la PR B |
| B3 | **Main, Blessures, gloire, case vide** | C38, C39 | `screen/hand.rs`, `board/screen.rs` | **Sonnet** | Petites touches sur des écrans existants |
| B4 | **Dessin** dans `app.rs` : zone centrale combat ou plateau, raccourci Entrée (`Shortcut`) | — | `app.rs`, `session.rs`, `docs/raccourcis.md` | **Sonnet** | Câblage sans décision (ADR 0006) |
| B5 | **Manques de libellés** | C40 | `crates/arcanum-ui/tests/` | **Haiku** | Patron de C28 (spec 030) |
| B6 | **Vérification à l'œil** sous Xvfb (`scripts/verif-manuelle.sh`) : provocation, chaque phase, parade réussie, Blessures, case vide ; captures dans `captures/` | — | — | **Haiku** pour lancer et capturer, **Opus** pour juger les captures | Juger l'ergonomie demande du recul, pas la capture |
| B7 | **Docs et statut** : architecture (`screen/combat.rs`), PLAN et index (« implémentée »), glossaire si un terme a bougé | — | docs | **Haiku** | |
| B8 | **Bilan de fin de spec** (règle du projet) : points à améliorer, une issue par point confirmé avec Mathieu | — | — | **Opus** | |

## Risques repérés

- **Blessure dans `DeedCard`** (A1) : le choix fait en A1 se propage partout ; à trancher avant d'écrire A2.
- **Nombre d'actions légales** : un `DeclareAttack` par sous-ensemble d'ennemis ciblables (au plus 2^n − 1). Avec des ennemis
  errants seuls, n ≤ 3 en pratique ; une cité a au plus 5 défenseurs (31 déclarations), sans gêne : le découpage attend la
  spec du premier joueur automatique, avec la fin du tour (Mathieu, 2026-11-13, issue #3).
- **Historique par ennemi** (B2) : il se déduit des événements du combat ; si le journal filtré ne suffit pas, l'état du combat
  gardera la liste des attaques et parades résolues (donnée de plus, à signaler dans la PR).
- **Parcours J2** : il ne provoque pas d'ennemi ; aucun changement attendu. S'il en provoquait un, ajuster le trajet scripté et le
  signaler.
- **Quota CI** épuisé jusqu'au 1er novembre environ : vérification locale par `scripts/verifier.sh`, fusion seulement avec l'accord de
  Mathieu.
