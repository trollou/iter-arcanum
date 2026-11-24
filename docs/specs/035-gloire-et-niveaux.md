# 035 — Gloire et niveaux

- Statut : implémentée (2026-10-30)
- Jalon : J3
- Dépend de : 031 (gloire gagnée en combat, armure du héros 2), 016 (fin du tour, repioche), 023 (nouvelle manche), 024 (points
  de non-retour), 029 (contenu en données) ; recoupe la 034 (seuil de l'assommé), rédigée en parallèle
- Livret : MKUE p. 3 (*Player Area*, jetons de niveau et jeton de commandement), p. 9 (*Ranged and Siege Attack Phase* 5.b ;
  *Assign Damage Phase* 5.a et 5.c), p. 11 (*End of the Turn* 6 et 7.b), p. 4 (2.g, pioche de la nouvelle manche) ; piste de gloire
  et jetons de niveau : scans `docs/regles/scans/misc/fame_board.jpg` et `level_token_*.png` ; FAQ p3 (*Changements de Niveau*) ;
  Compendium, *End of Turn* S6, S7, *Miscellaneous* S8 (9)
- Crates touchées : arcanum-core (règles, contenu), arcanum-ui, arcanum-persistance (une partie préparée), `assets/i18n/` (fr.json et en.json)

## Objectif

La gloire gagnée en combat fait maintenant **changer de niveau** le héros, à la fin du tour où elle franchit une ligne de la piste de
gloire. Aux niveaux impairs, le héros retourne un jeton de niveau : son **armure** et sa **limite de main** augmentent, et il gagne un
**jeton de commandement**. Le panneau montre la gloire, le niveau, l'armure, la limite de main et les jetons de commandement.

## Règles

### Piste de gloire et niveaux

- **Piste de gloire** (MKUE p. 3, *Fame and Reputation board* ; scan `misc/fame_board.jpg`) : dix lignes, une par niveau. Le niveau
  est celui de la ligne où se trouve la gloire. Seuils lus sur le scan :

  | Niveau | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 |
  |---|---|---|---|---|---|---|---|---|---|---|
  | Gloire | 0 | 3 | 8 | 15 | 24 | 35 | 48 | 63 | 80 | 99 |
  | Icône | — | compétence | jeton | compétence | jeton | compétence | jeton | compétence | jeton | compétence |

- **Au-delà** (Compendium, *Miscellaneous* S8 (9)) : le niveau ne dépasse jamais 10 ; la gloire continue de compter après la fin de
  la piste (119). Le total de gloire n'a donc pas de plafond.
- **Gagner de la gloire** (MKUE p. 9, *Ranged and Siege Attack Phase* 5.b) : déjà fait par la 031 (total affiché). Seule source de
  gloire pour l'instant ; les autres (dé noir des récompenses, cartes, scénarios) viendront avec leurs specs. La gloire ne baisse
  jamais dans cette spec.
- **Pas tout de suite** (MKUE p. 9, 5.b, *If it crosses the end of a line…* ; FAQ p3, *Changements de Niveau*) : franchir une ligne
  pendant le tour ne change rien avant la fin du tour. L'armure et la limite de main restent celles du niveau d'avant jusque-là,
  même dans un second combat du même tour (impossible pour l'instant : un combat par tour).

### Changement de niveau, à la fin du tour

- **Moment** (MKUE p. 11, *End of the Turn* 6) : étape 6, après le bénéfice de la case (4, cristal d'une mine) et les récompenses de
  combat (5, spec 044), avant la pioche (7). Un niveau par ligne franchie : de 7 à 16 de gloire, le héros passe au niveau 3 puis au
  niveau 4. Atteindre exactement le seuil (8) franchit la ligne.
- **Niveau impair, icône jeton** (MKUE p. 11, 6.a ; p. 3, *Player Area*) : le jeton de niveau du dessus est retiré, celui de dessous
  donne la nouvelle armure et la nouvelle limite de main ; le jeton retiré devient un jeton de commandement. Valeurs lues sur les
  jetons (`misc/level_token_*.png`) :

  | Jeton | 1-2 | 3-4 | 5-6 | 7-8 | 9-10 |
  |---|---|---|---|---|---|
  | Armure | 2 | 3 | 3 | 4 | 4 |
  | Limite de main | 5 | 5 | 6 | 6 | 7 |
  | Jetons de commandement | 1 | 2 | 3 | 4 | 5 |

  Ces trois valeurs se déduisent du niveau : l'état ne garde que le **niveau**.
- **Niveau pair, icônes compétence et action avancée** (MKUE p. 11, 6.b) : hors périmètre, spec 051 (et 050 pour l'offre d'actions
  avancées). Le niveau est gagné et affiché ; rien d'autre ne se passe. La 051 dira si les récompenses manquées avant elle se
  rattrapent (rien n'est gardé ici pour cela).
- **Limite de main de la pioche** (MKUE p. 11, 7.b) : la pioche du même tour (étape 7) utilise déjà la limite du nouveau niveau.
  Passer au niveau 5 en fin de tour fait donc piocher jusqu'à 6. Les bonus de forteresse, de cité et de la tactique Planning
  (p. 11, 7.b ; p. 4, 2.g) : specs 043, 062, 060.
- **Nouvelle manche** (MKUE p. 4, 2.g) : la pioche de la nouvelle manche (spec 023) prend la limite de main du niveau.
- **Mise en place** (MKUE p. 3, *Player's hand*) : niveau 1, main de 5, inchangée.

### Ce que le niveau change ailleurs

- **Armure du héros** (MKUE p. 9, *Assign Damage Phase* 5.a) : la constante d'armure 2 de la 031 devient l'armure du niveau.
  Elle sert partout où la 031 et la 033 comptent les Blessures du héros (dégâts, Brutal, Toxique, Paralysant) et dans les enjeux
  affichés à la parade (« Si non paré : 5 dégâts, 2 blessure(s) »).
- **Assommé** (MKUE p. 9, 5.c, *unmodified Hand limit*) : le seuil de la 034 est la limite de main **du jeton de niveau**, sans
  bonus (forteresse, cité, Planning). Si la 034 est codée avant la 035, cette spec remplace son 5 fixe par la limite du niveau ;
  sinon, la 034 l'utilise directement.
- **Jetons de commandement** (MKUE p. 3, *Unit Area*) : un par unité recrutée. Il n'y a pas encore d'unités (spec 042) : le nombre
  est seulement affiché. Recruter une unité avec un jeton gagné pendant le tour est impossible avant la fin du tour (FAQ p3) : la
  042 s'appuiera sur le niveau de l'état, qui ne change qu'en fin de tour.

## Modèle

Termes du glossaire déjà présents : Gloire, Niveau, Armure, Limite de main, Jeton de commandement.

Nouveaux termes, ajoutés au glossaire par cette PR :

| Livret (EN) | Français | Code |
|---|---|---|
| Fame track | Piste de gloire | `FameTrack` (seuils des niveaux, contenu) |
| Level token | Jeton de niveau | `LevelToken` (armure, limite de main) |
| Level up | Changement de niveau (FAQ p3) | `LevelGained` |

Données nouvelles :

- dans l'état, par joueur : le **niveau** (1 au départ), changé seulement par `LevelGained` ; la gloire existe déjà (031) ;
- dans le contenu (ADR 0005, « valeurs en données ») : un fichier `crates/arcanum-core/content/levels.ron` qui décrit les dix niveaux
  (seuil de gloire, armure, limite de main, jetons de commandement, récompense de niveau pair ou impair), validé par un test (dix
  niveaux, seuils croissants à partir de 0, valeurs qui ne baissent pas) ;
- dans la vue du joueur : niveau, armure, limite de main, jetons de commandement, seuil du niveau suivant (aucun au niveau 10), et le
  niveau qui sera atteint à la fin du tour s'il a changé.

`STARTING_HAND_LIMIT` reste la main de départ (niveau 1) ; la repioche de fin de tour et celle de la nouvelle manche prennent la
limite du niveau.

## Actions et événements

Aucune action nouvelle : le changement de niveau fait partie de `EndTurn` (spec 016).

| Action | Validation (refusée si…) | Événements produits |
|---|---|---|
| `EndTurn` (016, inchangée) | inchangée | … `TurnEnded`, `CrystalGained` (mine), puis **un `LevelGained` par ligne franchie**, puis les défausses et les `CardDrawn` jusqu'à la limite du nouveau niveau, `TurnStarted` |

| Événement | Effet de `apply` | Révèle une information ? | Point de non-retour ? |
|---|---|---|---|
| `LevelGained { player, level }` | le niveau du joueur passe à `level` | non (publique) | non (`TurnEnded`, qui le précède, l'est déjà) |

Les sauvegardes d'avant restent lisibles (aucun événement existant ne change de forme, pas de migration avant la fin du J6, spec
024). Une partie sauvegardée avec une gloire déjà au-delà d'une ligne rattrape ses niveaux à sa prochaine fin de tour (L4).

## Comportements attendus

Situation de départ des tests moteur : l'arène de combat des tests (spec 032) ou le bac à sable, avec des Rôdeurs (armure 3,
gloire 2) ; une gloire de départ se construit par des `EnemiesDefeated` (et des `LevelGained` pour un niveau de départ).

- **C1** — Partie neuve : niveau 1, gloire 0, armure 2, limite de main 5, un jeton de commandement, niveau suivant à 3
  (`new_game_starts_at_level_one`) — MKUE p. 3, *Player Area*.
- **C2** — Le contenu décrit dix niveaux, seuils 0, 3, 8, 15, 24, 35, 48, 63, 80, 99, et les valeurs des jetons du tableau
  (`levels_content_matches_fame_track_and_level_tokens`) — scans `misc/`.
- **C3** — Gloire 1, Rôdeurs vaincus (+2, total 3) : pendant le reste du tour, le niveau vaut toujours 1 et l'armure 2 ; la vue annonce
  le niveau 2 en fin de tour (`fame_crossing_a_line_does_not_level_up_immediately`) — MKUE p. 9, 5.b.
- **C4** — Même situation, fin du tour : `LevelGained { level: 2 }` après `TurnEnded` et avant les `CardDrawn` ; niveau 2, armure 2,
  limite 5, un jeton de commandement (`end_of_turn_levels_up_after_crossing_a_line`) — MKUE p. 11, 6.
- **C5** — Gloire 6, niveau 2, Rôdeurs vaincus (+2, total 8, seuil atteint exactement) puis fin du tour : niveau 3, armure 3, limite 5,
  deux jetons de commandement (`reaching_exact_threshold_levels_up`) — MKUE p. 11, 6.a.
- **C6** — Gloire 7, niveau 2, ennemis vaincus pour 9 de gloire (total 16) : deux `LevelGained`, niveaux 3 puis 4, dans cet ordre
  (`crossing_two_lines_gains_two_levels`) — MKUE p. 11, 6.
- **C7** — Niveau 4, gloire 22, passage au niveau 5 en fin de tour, main gardée de 2 cartes : le joueur pioche 4 cartes (limite 6)
  (`level_up_raises_hand_limit_before_drawing`) — MKUE p. 11, 6.a et 7.b.
- **C8** — Fin de tour sans ligne franchie : aucun `LevelGained` (`no_level_up_without_crossing`).
- **C9** — Gloire 99, niveau 10, nouveaux ennemis vaincus jusqu'à 125 : niveau 10, aucun `LevelGained`, gloire 125
  (`level_never_exceeds_ten`) — Compendium, *Miscellaneous* S8 (9).
- **C10** — Niveau 3 : un ennemi non paré à 5 d'attaque donne 2 Blessures (armure 3) au lieu de 3 ; Brutal (10) en donne 4
  (`hero_armor_follows_level`) — MKUE p. 9, 5.a ; p. 24, *Brutal*.
- **C11** — Niveau 5 (limite 6), nouvelle manche : la nouvelle main a 6 cartes (`new_round_draws_up_to_level_hand_limit`) — MKUE
  p. 4, 2.g.
- **C12** — Revenir en arrière juste après la victoire qui a franchi la ligne (avant la fin du tour) rend la gloire d'avant, sans
  niveau à annoncer ; après la fin du tour, impossible, comme avant (`level_up_follows_undo_rules`) — spec 024.
- **C13** — Une sauvegarde rechargée après un changement de niveau rend le même niveau, la même armure, la même limite de main
  (`saved_game_keeps_level`) — spec 024.
- **C14** — Si la 034 est fusionnée : au niveau 5 (limite 6), 5 Blessures dans un combat n'assomment pas, 6 oui
  (`knock_out_threshold_follows_level`) — MKUE p. 9, 5.c.

Interface (arcanum-ui, modèle d'écran) :

- **C15** — Le panneau a la ligne `fame.line` (« Gloire : 9 · Niveau 3 · niveau 4 à 15 ») et la ligne `level.stats` (« Armure 3 ·
  Limite de main 5 · Jetons de commandement 2 ») ; au niveau 10, `fame.line_max` sans seuil suivant
  (`panel_shows_fame_level_and_stats`).
- **C16** — Ligne franchie pendant le tour : la ligne `fame.pending` (« Niveau 3 à la fin du tour ») sous `fame.line`, et le bandeau
  du mode fin du tour compte les cartes à piocher avec la nouvelle limite (`pending_level_shown_and_used_for_draw_count`).
- **C17** — Au début du tour qui suit un changement de niveau, le bandeau `level.gained` (« Niveau 3 : armure 3, limite de main 5,
  un jeton de commandement de plus ») ; à un niveau pair, `level.gained_even` (« Niveau 2 : compétence et action avancée pas encore
  disponibles ») ; il disparaît à la première décision du tour, comme `round.new` (`level_up_banner_at_next_turn`).

## Cas limites et refus

- **L1** — Gloire gagnée par un ennemi invoqué : aucune (031, 033), donc aucun effet sur le niveau.
- **L2** — Ligne franchie au dernier tour de la manche, puis fin de manche annoncée au tour suivant (deck vide) : le niveau a déjà
  changé à la fin du tour où la ligne a été franchie ; la nouvelle manche pioche avec la nouvelle limite (C11).
- **L3** — Passer son tour pour annoncer la fin de la manche (MKUE p. 6, 4) : pas de séquence de fin de tour, mais aucune gloire n'a
  pu être gagnée pendant ce tour ; rien à faire.
- **L4** — Sauvegarde d'avant cette spec avec une gloire au-delà d'un seuil : le niveau rechargé vaut 1 ; à la prochaine fin de tour,
  tous les niveaux manqués sont gagnés d'un coup (on compare la gloire au niveau de l'état, pas seulement la gloire du tour).
- **L5** — Plus de cartes en main que la nouvelle limite : on ne défausse rien et on ne pioche rien (MKUE p. 11, 7.c, spec 016,
  inchangé).

## Information cachée

Tout est public : gloire, niveau, armure, limite de main, jetons de commandement (piste et jetons sont sur la table). Rien à filtrer.

## Libellés (i18n)

| Clé | Paramètres | fr | en |
|---|---|---|---|
| `fame.line` (modifiée) | `fame`, `level`, `next_level`, `next_fame` | Gloire : {fame} · Niveau {level} · niveau {next_level} à {next_fame} | Fame: {fame} · Level {level} · level {next_level} at {next_fame} |
| `fame.line_max` | `fame` | Gloire : {fame} · Niveau 10 (maximum) | Fame: {fame} · Level 10 (maximum) |
| `fame.pending` | `level` | Niveau {level} à la fin du tour | Level {level} at the end of the turn |
| `level.stats` | `armor`, `hand_limit`, `command` | Armure {armor} · Limite de main {hand_limit} · Jetons de commandement {command} | Armor {armor} · Hand limit {hand_limit} · Command tokens {command} |
| `level.gained` | `level`, `armor`, `hand_limit` | Niveau {level} : armure {armor}, limite de main {hand_limit}, un jeton de commandement de plus | Level {level}: armor {armor}, hand limit {hand_limit}, one more Command token |
| `level.gained_even` | `level` | Niveau {level} : compétence et action avancée pas encore disponibles | Level {level}: Skill and Advanced Action not yet available |

Deux niveaux gagnés au même tour : un bandeau par niveau, l'un sous l'autre.

## Interface

- **Panneau latéral** : `fame.line` remplace « Gloire : 2 » (031) à la même place, suivie de `level.stats` ; `fame.pending` en
  dessous quand une ligne est franchie pendant le tour.
- **Mode fin du tour** (016) : « Vous piocherez N carte(s) » compte avec la limite du niveau à venir.
- **Début du tour suivant** : bandeau `level.gained` ou `level.gained_even`, à la place de celui de la nouvelle manche quand les deux
  tombent ensemble (`round.new` dessous).
- Pas de dessin de la piste de gloire ni des jetons de niveau : visuels provisoires, du texte suffit.

```
┌─ Panneau latéral ─────────────────────────────────┐
│ Niveau 3 : armure 3, limite de main 5, un jeton   │
│ de commandement de plus                           │
│ …                                                 │
│ Gloire : 9 · Niveau 3 · niveau 4 à 15             │
│ Armure 3 · Limite de main 5 · Jetons de           │
│ commandement 2                                    │
│ Main                                              │
│ ● Rage        Attaque 2 ou Parade 2               │
└───────────────────────────────────────────────────┘
```

Vérification à l'œil : une partie préparée `niveau-3` (bac à sable, gloire 6, niveau 2, figurine au village (2,−3) à côté des
Rôdeurs) : les vaincre, voir `fame.pending`, finir le tour, voir le bandeau et les nouvelles valeurs.

### Raccourcis clavier

Aucun : rien ne se choisit, le changement de niveau est automatique à la fin du tour.

## Effets sur les specs déjà implémentées

- **031** : la constante d'armure du héros (2) devient l'armure du niveau ; « Gloire : 2 » devient `fame.line`. Les tests de la 031
  restent valables (niveau 1, armure 2).
- **016 et 023** : la repioche prend la limite de main du niveau au lieu de 5.
- **034** (si fusionnée avant) : le seuil de l'assommé suit la limite du niveau ; `combat.knocked_out` prend le paramètre `limit`,
  comme la 034 le prévoit.
- **Parcours 019 et 028** : aucune gloire, donc inchangés.
- **024** : `Event::blocks_undo` gagne `LevelGained` (non).

## Hors périmètre

- Compétence et action avancée des niveaux pairs, offre de compétences communes : specs 051 et 050.
- Unités et usage des jetons de commandement, récompense d'unité reportée après un changement de niveau (MKUE p. 11, 5.d) : spec 042.
- Récompenses de combat, dont la gloire du dé noir (MKUE p. 11, 5.a) : spec 044.
- Bonus de limite de main (forteresse, cité, Planning) : specs 043, 062, 060.
- Gloire bonus à chaque ligne franchie de certains scénarios (MKUE p. 19 et 20, *You start with 1 Fame…*) : point d'extension de
  scénario, specs 063 et 064. Décompte final : 063.
- Ordre des tactiques selon la gloire (MKUE p. 4, 3.a) : spec 060.
- Gloire perdue (variantes, p. 16) : hors périmètre.

## Compte rendu d'implémentation

- Le bandeau `level.gained` (C17) vient des niveaux gagnés à la dernière fin de tour, gardés dans l'état (`levels_gained`, vidés
  par la fin du tour suivant ou de la manche). Il disparaît à la première décision du tour, repérée comme pour `round.new` : une
  carte jouée, un dé pris, un déplacement, une exploration ou un défi.
- La partie préparée `niveau-3` est construite avec un `LevelGained` : elle s'ouvre donc sur le bandeau du niveau 2.
- Issue privée 87 (seuil de l'assommé) : traitée ici, C14 ; `combat.knocked_out` prend le paramètre `limit`.

## Questions ouvertes

Aucune. Validée par Mathieu le 2026-10-30, avec les propositions des deux questions :

- **Q1 — Termes** : « changement de niveau » (FAQ française, p3) pour *Level up* ; « limite de main » est gardé (glossaire, code,
  UI), plutôt que « limite de cartes en main » de la FAQ.
- **Q2 — Niveaux pairs** : en attendant la 051, un niveau pair ne donne rien, avec le bandeau `level.gained_even` ; aucune
  récompense n'est gardée en attente (la 051 décidera).
