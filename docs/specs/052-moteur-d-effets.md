# 052 — Moteur d'effets

- Statut : implémentée (PR privée 216, 2026-11-12) ; validée le 2026-11-11 (questions Q1 à Q8 tranchées, version revue après relecture critique)
- Jalon : J5
- Dépend de : 013, 014, 016, 017, 036 (cartes, déplacement, fin du tour, trajets), 022 et 025 (effets renforcés, Concentration), 024
  (sauvegarde et retour en arrière), 030 à 035 (combat, Blessures), 040 (réputation, interaction), 042 (unités), 043 (assaut,
  retraite), 044 (clairière), 050 (actions avancées en données), 073 (zone de jeu) ; toutes implémentées
- Livret : MKUE p. 4 (*Deed Cards* 3.e) ; p. 5 (*Effects* 1.b ; *"Composite" Effects* 1.b) ; p. 6 (5, 6.e) ; p. 7 (7.a à 7.d) ;
  p. 10 (*Combat Outcomes* 3.d) ; p. 11 (*End of the Turn* 2 à 4). Texte des cartes : `docs/regles/cartes/actions-avancees.md`
  (version *Lost Legion* pour Agilité) et `cartes/unites.md` (*Foresters*). FAQ p2 (Embuscade), FAQ p3 (retraite forcée).
  Compendium, *Advanced Action Cards* (Agility S1 à S3, Ambush S1 à S6, Diplomacy S1 à S3, Frost Bridge S1, S2, Intimidate S1,
  Path Finding S1, Song of Wind O1, S4, Steady Tempo S1, S2), *Forced Withdrawal* S1 à S4, *Movement* S1, *Miscellaneous* S1, S9,
  *Will Focus* S1
- ADR : [0008 Moteur d'effets](../decisions/0008-moteur-d-effets.md) (accepté avec cette spec)
- Crates touchées : arcanum-core (règles, contenu), arcanum-ui, arcanum-persistance (parties préparées), `assets/i18n/`

## Objectif

Les cartes peuvent enfin **changer les règles du tour** : rendre des terrains moins chers ou les lacs franchissables, faire servir
le Mouvement en Attaque ou l'Influence en Parade, renforcer la prochaine carte d'Attaque ou de Parade, changer la réputation ou la
place d'une carte à la fin du tour. Le moteur garde ces **effets en cours** (modificateurs, ADR 0008), l'écran les montre dans la
zone de jeu, et le héros qui finit son tour sur une case qui n'est pas **sécurisée** subit la **retraite forcée**.

Le mécanisme est prouvé sur **7 actions avancées** et sur la capacité de déplacement des **Bûcherons** ; les 21 autres actions
avancées et les capacités d'unités encore `Unsupported` sont la spec 053 (Q1, Q3, Q8).

| Carte (fr.json) | Effet de points | Modificateurs posés (ADR 0008) |
|---|---|---|
| Chemins de traverse (*Path Finding*) | Mouvement 2 ; renforcé 4 | `TerrainCost` tous −1 (min. 2) ; renforcé : tous à 2 |
| Pont de glace (*Frost Bridge*) | Mouvement 2 ; renforcé 4 | `TerrainCost` marais à 1 ; renforcé : lacs et marais à 1, `Passable` lac |
| Bûcherons, 1re capacité | Mouvement 2 | `TerrainCost` forêts, collines, marais −1 (min. 0) |
| Agilité (*Agility*, LL) | Mouvement 2 ; renforcé 4 | `Convert` Mouvement → Attaque 1 pour 1 ; renforcé : en plus 2 → Attaque à distance 1 |
| Diplomatie (*Diplomacy*) | Influence 2 ; renforcé 4 | `Convert` Influence → Parade 1 pour 1 ; renforcé : de glace ou de feu (deux options) |
| Embuscade (*Ambush*) | Mouvement 2 ; renforcé 4 | `NextCardBonus` Attaque +1 ou Parade +2 ; renforcé +2 / +4 |
| Intimidation (*Intimidate*) | Influence 4 ou Attaque 3 ; renforcé 8 ou 7 | `EndOfTurnReputation` −1 ; renforcé −2 |
| Temporisation (*Steady Tempo*) | Mouvement 2 ; renforcé 4 | `ReturnToDeck` sous le deck ; renforcé : dessus |

## Règles

### Effets en cours (modificateurs)

- **Durée et déclenchement** (MKUE p. 5, *Effects* 1.b ; ADR 0008) : toutes les cartes de cette spec durent le tour (`Turn`).
  Embuscade est consommée par la carte qui reçoit son bonus ; Intimidation et Temporisation agissent à la fin du tour ; les autres
  sont lues tant qu'elles existent.
- **Effet composite** (MKUE p. 5, *"Composite" Effects* 1.b ; Compendium, *Will Focus* S1) : Concentration et Pouvoir de
  concentration renforcés ajoutent leur bonus aux seuls points de la carte jouée avec eux (`with_bonus`, spec 025) ; les
  modificateurs posés ne changent pas.

### Quand une carte se joue

- **Effet utilisable** (MKUE p. 6, 5 ; Compendium, *Miscellaneous* S1) : une option se joue si **ses points servent maintenant** ou
  si **l'un de ses modificateurs sert maintenant** ; ses points vont alors dans leur réserve, même s'ils ne servent plus (Compendium,
  *Steady Tempo* S1). La carte de côté reste toujours possible (MKUE p. 4, 3.e). Règle unique dans `is_playable_now` (issue privée 111).
- **Les points servent** : Mouvement pendant la phase de mouvement (aujourd'hui), et en combat tant qu'un `Convert` depuis le
  Mouvement existe et que le combat n'est pas fini ; Influence hors combat comme aujourd'hui (phase de mouvement, interaction), et en
  combat tant qu'un `Convert` depuis l'Influence existe et que la phase de parade n'est pas finie ; Attaque et Parade comme
  aujourd'hui (spec 031).
- **Le modificateur sert** :

| Sorte | Sert… | Ne sert plus… |
|---|---|---|
| `TerrainCost`, `Passable` | pendant la phase de mouvement | dès que le mouvement est fini (action, interaction, combat ; `movement_over`) |
| `Convert` depuis le Mouvement (Agilité) | phase de mouvement, et combat jusqu'à sa fin (Compendium, *Agility* S2) | après le combat, ou dans un tour sans combat une fois l'action commencée |
| `Convert` depuis l'Influence (Diplomatie) | phase de mouvement, et combat jusqu'à la fin de la phase de parade (Compendium, *Diplomacy* S1) | en phase d'attaque, après le combat |
| `NextCardBonus` (Embuscade) | phase de mouvement, et combat jusqu'à sa fin (Compendium, *Ambush* S4) | après le combat |
| `EndOfTurnReputation`, `ReturnToDeck` | jamais seuls : ce sont un coût ou un à-côté | — |

  Conséquences : Agilité, Diplomatie et Embuscade se jouent **aussi pendant le combat**, leurs points allant dans la réserve ;
  Intimidation se joue quand son Influence ou son Attaque sert ; Temporisation quand son Mouvement sert.

### Déplacement

- **Coût réduit** (MKUE p. 7, 7.a ; Compendium, *Path Finding* S1, *Song of Wind* S4) : les réductions s'appliquent à tous les pas
  qui suivent l'effet, jusqu'à la fin du tour. Plusieurs réductions s'appliquent dans l'ordre le plus favorable au joueur ; le
  moteur retient le coût le plus bas parmi les ordres possibles. Jamais sous 0. « Min. 2 » ne remonte jamais un coût déjà plus bas,
  « à 2 » non plus (Chemins de traverse renforcé ne change pas un marais déjà ramené à 1 par Pont de glace).
- **Lacs franchissables** (MKUE p. 7, 7.b ; Compendium, *Frost Bridge* S1, S2, *Song of Wind* O1 ; Q7) : avec Pont de glace
  renforcé, un lac s'entre au coût réduit (1). Le héros peut s'y arrêter, y être provoqué, y combattre, défier ou explorer depuis le
  lac ; un lac n'est jamais une case sécurisée. Les montagnes et la mer restent infranchissables.
- **Mouvement perdu** (MKUE p. 6, 6.e ; Compendium, *Movement* S1, *Agility* S2) : les points de Mouvement restants sont perdus au
  début de l'action, comme aujourd'hui (spec 040) ; les modificateurs restent.

### Case sécurisée et retraite forcée

- **Case sécurisée** (MKUE p. 7, 7.d) : accessible sans effet (ni lac, ni montagne, ni mer), et pas un site fortifié non conquis.
  La 3e condition (un autre héros) est sans objet en solo.
- **Retraite forcée** (MKUE p. 11, *End of the Turn* 2 ; FAQ p3 ; Compendium, *Forced Withdrawal* S1 à S4) : à la fin du tour, en
  premier, si la figurine n'est pas sur une case sécurisée, elle **revient sur ses pas** le long du trajet du tour jusqu'à la
  première case sécurisée, et le joueur prend **une Blessure en main par case parcourue**. Pas de provocation, pas de coût en
  Mouvement (S3). Hors combat, les Blessures ne rendent pas assommé.
- **Trajet du tour** : la case du début du tour, puis chaque case où la figurine est entrée par `HeroMoved`. La retraite d'un assaut
  (`HeroWithdrew`, spec 043) **retire** le site du trajet au lieu d'y ajouter la case de départ : la figurine est revenue d'où elle
  venait. La case du début du tour est toujours sécurisée (la fin du tour précédent l'a garanti), donc le retour trouve toujours une
  case.
- **Retraite après un assaut** (MKUE p. 10, *Combat Outcomes* 3.d ; FAQ p3) : la retraite de la spec 043 n'est pas une retraite
  forcée et ne blesse pas ; mais si la case de départ de l'assaut n'est pas sécurisée (un lac), la retraite forcée suit à la fin du
  tour (spec 043, Q1).
- **Fin du tour après la retraite** (MKUE p. 11, 2 à 4) : la retraite vient avant tout le reste ; la défausse, le repos, la
  clairière, la mine, la limite de main près d'une forteresse et la pioche s'évaluent **sur la case d'arrivée et la main d'après la
  retraite** (Blessures comprises). Les actions légales `EndTurn` sont donc calculées sur l'état qui suit la retraite.

### Conversions de points

- **Agilité** (version *Lost Legion* ; Compendium, *Agility* S1, S2 ; *Miscellaneous* S9) : en combat, la réserve de Mouvement
  n'est pas perdue et se dépense dans une **attaque en cours** : en phase d'attaque, 1 Mouvement pour Attaque 1 physique ; en phase
  d'attaque à distance, avec Agilité renforcée seulement, 2 Mouvement pour Attaque à distance 1 physique, contre des cibles non
  fortifiées. La phase décide du genre. Ce qui reste sert plus tard dans le combat.
- **Diplomatie** (Compendium, *Diplomacy* S1 à S3) : en combat, la réserve d'Influence se dépense dans une **parade en cours**,
  1 Influence pour Parade 1 : physique de base, de glace ou de feu (option choisie en jouant la carte) renforcée.
- **Pas une carte** (Compendium, *Ambush* S6) : une Attaque ou une Parade obtenue par conversion ne reçoit pas et ne consomme pas
  le bonus d'Embuscade.

### Prochaine carte : Embuscade

- **Bonus** (Compendium, *Ambush* S1 à S4 ; FAQ p2) : la **prochaine carte** jouée pour une Attaque ou une Parade après Embuscade,
  de côté comprise, reçoit +1 Attaque (+2 renforcé) si c'est une Attaque, +2 Parade (+4) si c'est une Parade, du même élément et du
  même genre que la carte. Le bonus est alors consommé. Les capacités d'unités ne le consomment pas et n'en profitent pas (FAQ p2).
  Une carte jouée avec Concentration reçoit aussi le bonus.

### Fin du tour

- **Intimidation** (Compendium, *Intimidate* S1) : réputation −1 (renforcé −2), même si l'Influence n'a pas servi ; rien sur la case X.
- **Temporisation** (Compendium, *Steady Tempo* S1, S2 ; Q5) : au moment de défausser les cartes jouées (MKUE p. 11, 3.b), le joueur
  peut mettre la carte **sous** son deck s'il n'est pas vide (renforcé : **sur** le deck, même vide) au lieu de la défausse.
- **Ordre** : retraite forcée, réputation (Intimidation), puis comme aujourd'hui : cartes jouées défaussées (Temporisation), bénéfices
  de la case, récompenses, niveaux, pioche.

### Exemple : le lac de la campagne 1

Bac à sable, campagne 1 posée en B (`docs/reference/bac-a-sable.md`) : clairière en `(1, -3)`, village en `(2, -3)`, lac en `(2, -4)`,
Orques maraudeurs (errants) en `(1, -4)`.

```
 (1,-4) Orques    (2,-4) Lac
        \           /      \
   (1,-3) clairière ── (2,-3) village
```

1. Le héros est sur le village `(2, -3)`, joue Pont de glace renforcé (mana bleu) : Mouvement 4, lacs à 1.
2. Il entre dans le lac `(2, -4)` pour 1 : `(2, -3)` ne touche pas les Orques, aucune provocation.
3. Il défie les Orques depuis le lac et les bat.
4. Il finit son tour sur le lac : retraite forcée vers `(2, -3)`, une case, une Blessure en main.

Depuis la clairière `(1, -3)`, le pas vers le lac provoque les Orques (`(1, -3)` et `(2, -4)` les touchent toutes deux) ; la
retraite ramène alors sur la clairière, où la Blessure de la retraite peut être jetée (spec 044).

## Modèle

Termes du glossaire : Case sécurisée (*Safe space*), Retraite forcée (*Forced Withdrawal*), Effet en cours (*Modifier*).

Données nouvelles dans l'état (noms indicatifs, forme dans l'ADR 0008) :

- **`PlayerState::modifiers`** : les effets en cours du joueur (sorte paramétrée, source, durée, déclenchement).
- **`PlayerState::turn_path`** : le trajet du tour (case de départ, puis chaque case entrée ; la retraite d'assaut retire le site),
  remis à la case de la figurine par `TurnStarted`.
- **Options en données** (`advanced_actions.ron`, `units.ron`) : une option devient « effet de points et modificateurs posés » ;
  les 7 cartes et la capacité des Bûcherons quittent `Unsupported`. Les options existantes n'ont aucun modificateur (format RON :
  champ facultatif, les fichiers actuels restent valides).
- **Règles de déplacement** (`MoveRules`, nom indicatif) : le moment de la journée et les modificateurs de terrain ; `move_cost` et
  les trajets (`routes`, `routes_avoiding`, `routes_ending_in`) les prennent à la place de `TimeOfDay`, avec une conversion
  `From<TimeOfDay>` pour que les ~30 appels des tests (`deplacement`, `cases_atteignables`, `trajet_en_un_clic`, parcours) et de
  arcanum-ui gardent leur forme. `is_safe_space(map, hex, …)`, publique.
- **`EndTurn`** : un champ de plus, garder Temporisation (oui ou non). Les littéraux `EndTurn { … }` des tests et du joueur scripté
  sont mis à jour (signalé dans la PR).

## Actions et événements

| Action | Validation (refusée si…) | Événements produits |
|---|---|---|
| `PlayCard` (existante) | règle « Quand une carte se joue » | `CardPlayed` (pose les modificateurs ; consomme Embuscade) |
| `ActivateUnit` (existante) | inchangée | `UnitActivated` (pose le modificateur des Bûcherons) |
| `SpendMoveAsAttack` (nouvelle, sans paramètre) | pas de `Convert` depuis le Mouvement ; pas d'attaque en cours ; en phase à distance : pas de conversion à distance (Agilité de base) ou cible fortifiée ; réserve < 1 (< 2 en phase à distance) | `MoveSpentAsAttack` |
| `SpendInfluenceAsBlock` (nouvelle, sans paramètre) | pas de `Convert` depuis l'Influence ; pas de parade en cours ; réserve < 1 | `InfluenceSpentAsBlock` |
| `EndTurn` (existante, nouveau champ) | règle actuelle, évaluée après la retraite forcée ; garder Temporisation de base avec un deck vide, ou sans Temporisation jouée | `ForcedWithdrawal` si la case n'est pas sécurisée ; `ReputationChanged` (Intimidation) ; `SteadyTempoKept` ; puis comme aujourd'hui |

| Événement | Effet de `apply` | Révèle une information ? | Point de non-retour ? |
|---|---|---|---|
| `CardPlayed`, `UnitActivated` (existants) | en plus : modificateurs posés, bonus d'Embuscade appliqué et consommé | non | non |
| `MoveSpentAsAttack { ranged }` | réserve de Mouvement −1 (−2 à distance), attaque en cours +1 physique (à distance) | non | non |
| `InfluenceSpentAsBlock { element }` | réserve d'Influence −1, parade en cours +1 de l'élément | non | non |
| `ForcedWithdrawal { path, wounds }` | la figurine va à la dernière case de `path`, `turn_path` raccourci, ces Blessures en main | non | non (la fin du tour l'est déjà) |
| `SteadyTempoKept { card, top }` | la carte quitte la zone de jeu, sur ou sous le deck | non | non |
| `TurnEnded`, `CombatEnded` (existants) | en plus : retirent les modificateurs de leur durée | non | inchangé |

Aucun hasard nouveau : les tirages des parcours ne bougent pas (ADR 0007).

## Comportements attendus

### Modificateurs (ADR 0008)

- **C1** — Chemins de traverse joué de base : au tour suivant, la liste des effets en cours est vide (`modifiers_end_with_the_turn`)
  — MKUE p. 5, *Effects* 1.b.
- **C2** — Embuscade jouée sans carte d'Attaque ni de Parade ensuite : le bonus disparaît à la fin du tour
  (`unused_ambush_ends_with_turn`).
- **C3** — Revenir en arrière avant `CardPlayed` retire le modificateur ; une sauvegarde rechargée après le retrouve
  (`modifiers_follow_undo_and_reload`) — ADR 0008, spec 024.
- **C4** — La vue liste les effets en cours avec leur source (`PlayerView::modifiers`) (`view_lists_modifiers`).
- **C5** — Concentration renforcée avec Chemins de traverse : Mouvement 4 + 2 = 6, tous les terrains à 2 ; avec Embuscade :
  Mouvement 6, bonus de l'effet renforcé inchangé (Attaque +2, Parade +4) (`concentration_boosts_points_not_modifiers`) — Compendium, *Will Focus* S1.

### Quand une carte se joue

- **C6** — En phase d'attaque contre des Prowlers (enemies.ron : armure 3, attaque physique 4), sans Agilité jouée : March (Marche)
  ne se joue que de côté (comme aujourd'hui, `is_playable_in_combat`) ; Agilité de base se joue, ses 2 Mouvement vont dans la
  réserve (`agility_playable_in_combat`) — Compendium, *Agility* S2.
- **C7** — En phase de parade, Diplomatie de base se joue (Influence 2 dans la réserve) ; en phase d'attaque, plus rien ne
  l'autorise : seulement de côté (`diplomacy_playable_until_block_phase_ends`) — Compendium, *Diplomacy* S1 ; *Miscellaneous* S1.
- **C8** — Embuscade se joue en combat, juste avant la carte visée (`ambush_playable_in_combat`) — Compendium, *Ambush* S4.
- **C9** — Après le combat (`movement_over`), Chemins de traverse, Agilité, Embuscade et Temporisation ne se jouent que de côté
  (`movement_cards_not_playable_after_combat`).
- **C10** — Hors combat et hors interaction, après l'action commencée, Intimidation ne se joue que de côté ; en interaction, son
  option Influence se joue (`intimidate_playable_when_points_serve`).

### Déplacement

- **C11** — De jour, après Chemins de traverse de base : forêt 2, colline 2, désert 4, marais 4, plaine 2
  (`path_finding_reduces_by_one_minimum_two`) — MKUE p. 7, 7.a.
- **C12** — Après Chemins de traverse renforcé, tous les terrains accessibles coûtent 2 ; lacs et montagnes restent
  infranchissables (`strong_path_finding_sets_two`) — Compendium, *Path Finding* S1.
- **C13** — Après Pont de glace de base, un marais coûte 1 ; un lac reste infranchissable (`frost_bridge_swamp_costs_one`).
- **C14** — Après Pont de glace renforcé, un lac coûte 1 et figure parmi les cases atteignables ; une montagne non
  (`strong_frost_bridge_crosses_lakes`) — MKUE p. 7, 7.b.
- **C15** — Pont de glace de base puis Chemins de traverse renforcé : un marais coûte 1, pas 2 (ordre le plus favorable)
  (`terrain_reductions_use_best_order`) — MKUE p. 7, 7.a.
- **C16** — Bûcherons activés : de jour, forêt 2, colline 2, marais 4, plaine 2 (`foresters_reduce_forest_hills_swamp`) —
  `cartes/unites.md`.
- **C17** — Les cases atteignables (spec 017) et le trajet en un clic (spec 036) suivent les coûts réduits
  (`routes_use_reduced_costs`).
- **C18** — Le modificateur reste après la perte des points de Mouvement au début de l'action (`modifier_survives_action_start`) —
  MKUE p. 6, 6.e.

### Case sécurisée et retraite forcée

- **C19** — Héros sur le lac `(2, -4)` venu du village `(2, -3)` : la fin du tour produit `ForcedWithdrawal` vers `(2, -3)` et une
  Blessure en main (`forced_withdrawal_from_lake`) — MKUE p. 11, 2.
- **C20** — Venu par deux lacs voisins : deux cases de retour, deux Blessures. La campagne 11 (`tiles.ron`, `Countryside(11)`, hors
  de la pioche du bac à sable) a deux lacs voisins, à l'E et au N-E ; le test la pose (`forced_withdrawal_counts_each_space`) —
  MKUE p. 11, 2.b.
- **C21** — Retraite depuis le lac vers la clairière `(1, -3)` : la fin du tour propose de jeter une Blessure de la main, dont celle
  de la retraite ; la défausse et le repos s'évaluent sur la main d'après retraite (`end_turn_evaluated_after_withdrawal`) — MKUE
  p. 11, 2 et 4.a.
- **C22** — Les Blessures de la retraite ne rendent pas assommé, même au-delà de la limite de main
  (`forced_withdrawal_does_not_knock_out`) — spec 034.
- **C23** — Le retour ne provoque aucun ennemi errant voisin du trajet (`forced_withdrawal_provokes_nobody`) — Compendium, *Forced
  Withdrawal* S3.
- **C24** — Assaut de la tour de mage au centre de la campagne 11 depuis son lac à l'E, venu d'une case sécurisée : assaut perdu,
  retraite sur le lac sans Blessure (spec 043), puis à la fin du tour retraite forcée d'**une** case, une Blessure
  (`failed_assault_from_lake_then_forced_withdrawal`) — MKUE p. 10, 3.d ; FAQ p3.
- **C25** — Sur une case sécurisée, la fin du tour ne produit pas de retraite ; les parcours existants ne changent pas
  (`no_withdrawal_on_safe_space`).
- **C26** — La vue annonce la retraite à venir (case d'arrivée, nombre de Blessures, `PlayerView::forced_withdrawal`) quand la
  figurine est sur une case non sécurisée, **hors assaut en cours** (`view_announces_forced_withdrawal`).
- **C27** — `is_safe_space` : plaine oui, lac non, montagne non, forteresse non conquise non, forteresse conquise oui
  (`safe_space_rules`) — MKUE p. 7, 7.d.

### Conversions

- **C28** — Agilité de base jouée en mouvement, combat contre des Prowlers : en phase d'attaque, March jouée (Mouvement 2) puis deux
  `SpendMoveAsAttack` donnent Attaque 2 physique dans l'attaque en cours (`agility_converts_move_to_attack`) — Compendium,
  *Agility* S2.
- **C29** — Agilité renforcée, phase d'attaque à distance : 2 Mouvement donnent Attaque à distance 1 ; contre des Diggers
  (fortifiés) l'action n'est pas légale ; avec Agilité de base non plus (`strong_agility_ranged_two_for_one`) — Compendium,
  *Agility* S1.
- **C30** — Les points de Mouvement d'avant le combat sont perdus au début de l'action, même avec Agilité
  (`agility_does_not_keep_movement_points`) — Compendium, *Movement* S1.
- **C31** — Diplomatie de base, phase de parade contre des Prowlers : Promesse et Menace jouées (Influence 2 chacune), puis quatre
  `SpendInfluenceAsBlock` : parade physique 4 réussie (`diplomacy_converts_influence_to_block`) — Compendium, *Diplomacy* S1.
- **C32** — Diplomatie renforcée, option glace, contre des Fire Mages (attaque de feu 6) : la parade obtenue est de glace et
  compte en plein (`strong_diplomacy_chosen_element`) — Compendium, *Diplomacy* S2, S3 ; spec 032.
- **C33** — L'Influence restante d'une parade sert à la parade d'un autre ennemi (`diplomacy_pool_serves_several_blocks`) —
  Compendium, *Diplomacy* S1.

### Embuscade

- **C34** — Embuscade de base jouée en mouvement, puis en combat Rage (Attaque 2) : l'attaque compte 3, le bonus est consommé ; la
  carte d'Attaque suivante n'a rien (`ambush_bonus_on_first_attack_card`) — carte.
- **C35** — Embuscade renforcée, puis une carte de côté pour Parade 1 : Parade 5 (`ambush_applies_to_sideways_block`) — FAQ p2.
- **C36** — Embuscade, puis Épéistes d'Utem activés pour Attaque 3 : Attaque 3, bonus toujours en attente
  (`ambush_ignores_units`) — FAQ p2 ; Compendium, *Ambush* S3.
- **C37** — Embuscade, puis Attaque obtenue par Agilité : bonus non consommé (`ambush_ignores_conversions`) — Compendium, *Ambush* S6.

### Fin du tour

- **C38** — Intimidation (Influence 4 non dépensée) : réputation −1 à la fin du tour ; sur la case X, rien
  (`intimidate_reputation_at_end_of_turn`) — carte ; Compendium, *Intimidate* S1.
- **C39** — Temporisation de base gardée : la carte va sous le deck ; deck vide : garder n'est pas une action légale
  (`steady_tempo_under_deck`) — carte.
- **C40** — Temporisation renforcée gardée avec un deck vide : la carte va sur le deck et revient dans la pioche de fin du tour
  (`strong_steady_tempo_on_top`) — carte ; Compendium, *Steady Tempo* S2.
- **C41** — Ordre : retraite forcée, réputation, défausse (`end_of_turn_order`) — MKUE p. 11.

### Contenu et vue

- **C42** — Les 7 cartes et la 1re capacité des Bûcherons ne sont plus `Unsupported` ; des 21 autres actions avancées, les quatre
  Projectiles l'étaient déjà (spec 050), les 17 restantes le restent au moins en partie (`first_lot_is_supported`) — contenu.
- **C43** — `PlayerView::move_rules` donne à l'écran les coûts réduits : le coût d'une forêt vaut 2 après Chemins de traverse
  (`view_move_rules_follow_modifiers`).

## Cas limites et refus

- **L1** — `SpendMoveAsAttack` sans attaque en cours, ou dans une parade : refusée (`ActionError::NotLegal`).
- **L2** — `SpendInfluenceAsBlock` en phase d'attaque : refusée.
- **L3** — Retraite forcée alors que la figurine est sur la case de départ du tour : impossible (case sécurisée) ; un test garde
  l'invariant.
- **L4** — Deux Embuscades, deux Diplomaties, deux Pont de glace : impossible avec un exemplaire de chaque carte ; le cumul viendra
  avec Effet maximum (053), sans cas ici.

## Information cachée

Rien de caché : effets en cours, trajet, retraite à venir sont publics. Champs de la vue ajoutés : `PlayerView::modifiers` (C4),
`move_rules` (C43), `forced_withdrawal` (case d'arrivée et nombre de Blessures, C26). Le choix « garder Temporisation » se lit dans
les actions légales `EndTurn` (C39).

## Libellés (i18n)

Nouvelles clés (fr ; en à l'avenant). Le signe « − » est déjà affiché par le jeu (`assault`, `burn.confirm`).

- `play_area.modifiers` : « Effets en cours » ; `play_area.modifier_line` : « {source} : {effect} ».
- `modifier.*`, texte d'un effet en cours selon sa sorte et ses paramètres : `terrain_reduce` « {terrains} : −{by} (min. {min}) »,
  `terrain_set` « {terrains} : {cost} », `passable` « {terrain} franchissable », `terrains_all` « tous les terrains »,
  `convert_attack` « {amount} Mouvement en Attaque {points} », `convert_ranged` « {amount} Mouvement en Attaque à distance {points} »,
  `convert_block` « Influence en Parade », `convert_block_element` « Influence en Parade de {element} », `next_card_bonus`
  « prochaine carte : Attaque +{attack} ou Parade +{block} », `end_of_turn_reputation` « réputation {change} à la fin du tour »,
  `return_under_deck` « sous le deck à la fin du tour », `return_on_deck` « sur le deck à la fin du tour ». Les noms de terrains
  réutilisent `terrain.*` (valeurs relues à l'écriture des libellés, issue privée 127).
- `combat.move_as_attack` : « Mouvement en Attaque ({move} restant) » ; `combat.move_as_ranged` : « 2 Mouvement en Attaque à
  distance ({move} restant) » ; `combat.influence_as_block` : « Influence en Parade ({influence} restante) ».
- `end_turn.forced_withdrawal` : « Retraite forcée : {count} case(s), {count} Blessure(s) en main » ;
  `end_turn.steady_tempo` : « Garder Temporisation sous le deck » ; `end_turn.steady_tempo_strong` : « Garder Temporisation sur le
  deck ».
- `board.hover.unsafe_space` : « Case non sécurisée : retraite forcée si vous y finissez le tour ».

Les textes `card.<carte>.basic_text` / `strong_text` (spec 050) restent pour l'infobulle des cartes ; ceux des 7 cartes sont relus
contre `cartes/actions-avancees.md` (version LL pour Agilité).

## Interface

- **Effets en cours** dans la zone de jeu du panneau de droite, sous les cartes jouées : une ligne par modificateur, source et texte
  (C4, `PlayerView::modifiers`). Placement : vérification à l'œil.
- **Coûts et trajets** : cases atteignables, trajet survolé et infobulle de coût suivent les réductions et les lacs (C17, C43,
  `PlayerView::move_rules`).
- **Case non sécurisée** : l'infobulle d'une case atteignable non sécurisée (un lac) ajoute `board.hover.unsafe_space` (C27,
  `is_safe_space` sur `PlayerView::move_rules` et la carte de la vue).
- **Combat** : boutons `combat.move_as_attack` ou `move_as_ranged` dans une attaque en cours, `combat.influence_as_block` dans une
  parade en cours, quand l'action est légale (C28, C29, C31 ; actions légales) ; un clic = un point (Q4).
- **Texte des cartes** : une carte dont une option pose un modificateur montre en main, dans les offres et dans l'infobulle de la
  zone de jeu son texte écrit (`card.<carte>.basic_text` / `strong_text`), le texte calculé ne disant que ses points (« Mouvement
  2 ») ; ajout du fil écran (2026-11-12).
- **Options de cartes** : Diplomatie renforcée et Intimidation se jouent par les boutons d'options existants (spec 073) ; une option
  illégale est grisée (C7, C10, actions légales).
- **Fin du tour** : dans la fenêtre de fin du tour, la ligne `end_turn.forced_withdrawal` (C26, `PlayerView::forced_withdrawal`) et la
  case Temporisation, cochée d'office, absente quand garder n'est pas légal (C39, C40, actions légales `EndTurn` ; Q5). La clairière
  et la défausse proposées suivent la main d'après retraite (C21).

### Raccourcis clavier

Aucun : les conversions se font par clics sur un bouton qui n'existe qu'en combat (Q4).

## Effets sur les specs déjà implémentées

- **014, 017, 036, 043** : `move_cost` et les trajets prennent des règles de déplacement (`From<TimeOfDay>` : appels inchangés).
- **016, 034, 040, 044** : la fin du tour commence par la retraite forcée et la réputation de fin de tour, et s'évalue ensuite ;
  `EndTurn` a un champ de plus (tests et joueur scripté mis à jour, signalés dans la PR).
- **031** : Mouvement et Influence jouables en combat avec une conversion (`is_playable_in_combat`) ; `movement_over` n'interdit plus
  une carte dont un modificateur sert encore (commentaire de `movement_over` sur Agilité et Embuscade).
- **043** : `HeroWithdrew` raccourcit le trajet du tour (spec 043, Q1 close).
- **050** : 7 actions avancées quittent `Unsupported` ; **042** : la capacité des Bûcherons aussi (spec 042, Q2 en partie).
- **Parcours et parties préparées** : aucun tirage nouveau ; parties préparées régénérées si le format change.

## Hors périmètre

- Les 21 autres actions avancées, dont Bouclier de glace renforcé, Au cœur du combat, Coup de sang, Magie pure (second mana dans
  `CardPlay::Strong`), Besoin d'aide, Marche revigorante, Mélodie du vent, Effet maximum : spec 053, avec la règle générale des
  résistances aux effets (FAQ p2) et les premiers modificateurs d'ennemis (Q8).
- Capacités d'unités encore `Unsupported` (Illusionnistes, Glaceurs d'Amotep, Mages d'Altem, Gardiens d'Altem) : spec 053 (Q3).
- Carte jouée depuis une offre (Don pour la magie de base, qui joue un sort de l'offre) : spec 054 (Q2).
- Déplacements directs qui sautent des cases (MKUE p. 7, 7.c), autres héros (7.d, 3e tiret ; 8.e) : avec les sorts, ou hors
  périmètre (multijoueur).
- Attaques Encombrantes réduites par le Mouvement (*Cumbersome*, extensions) : hors périmètre.

## Plan de développement

Moteur et écran (issue privée 118) : deux fils sur la même branche, plan [`plans/052-moteur-d-effets.md`](plans/052-moteur-d-effets.md) :

1. **Moteur** : modificateurs (ADR 0008), jouabilité, règles de déplacement, retraite forcée, conversions, Embuscade, fin du tour,
   contenu, vue, persistance (C1 à C43).
2. **Écran** : libellés, zone de jeu, boutons de combat, fin du tour, infobulles du plateau, vérification à l'œil, docs, bilan.

## Décisions (2026-11-11)

Propositions de la rédaction acceptées par Mathieu (« en phase avec tes propositions ») :

1. **Q1** — Lot de la spec : proposé à 12 actions avancées et les Bûcherons, ramené à 7 et les Bûcherons après la relecture (Q8).
2. **Q2** — Carte jouée depuis une offre : déplacée dans la spec 054, avec Don pour la magie (ligne de PLAN.md corrigée).
3. **Q3** — Autres capacités d'unités `Unsupported` : spec 053.
4. **Q4** — Conversion point par point, un clic par point.
5. **Q5** — Case « Garder Temporisation » cochée d'office.
6. **Q6** — « Au début du combat » (Au cœur du combat) : sans objet ici, la carte passe à la 053 (Q8) ; la 053 précisera « unité
   activée **dans ce combat** ».
7. **Q7** — S'arrêter, combattre et explorer sur un lac sont permis, la retraite forcée en est le prix.

Relecture critique (demandée par Mathieu, faite par un sous-agent Fable 5.2) : retenus le trajet du tour raccourci par la retraite
d'assaut, la jouabilité en combat d'Agilité, Diplomatie et Embuscade, la fin du tour évaluée après la retraite, l'annonce de
retraite hors assaut, des sortes de modificateurs paramétrées et « points + modificateurs » au lieu d'une variante d'`Effect` par
carte, durée et déclenchement séparés, modificateurs d'ennemis en liste (ADR 0008), `From<TimeOfDay>` pour les règles de
déplacement, l'infobulle de case non sécurisée, et le retrait des cas irréalisables (cartes en un exemplaire).

8. **Q8** — Lot réduit, choisi par Mathieu après la relecture : 7 actions avancées et les Bûcherons. Coup de sang, Magie pure,
   Besoin d'aide, Marche revigorante, Au cœur du combat et Bouclier de glace renforcé passent à la 053, avec les unités et la règle
   générale des résistances.

## Questions ouvertes

Aucune.
