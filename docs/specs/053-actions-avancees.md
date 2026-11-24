# 053 — Actions avancées : combat, unités et soins

- Statut : implémentée (validée le 2026-11-12, questions Q1 à Q8 tranchées : réponses par défaut acceptées)
- Jalon : J5
- Dépend de : 052 (moteur d'effets, ADR 0008), 031 à 034 (combat, résistances, capacités des ennemis, Blessures), 035
  (assommé), 040 et 042 (réputation, unités, soins), 022 et 025 (effets renforcés, mana payé) ; toutes implémentées
- Livret : MKUE p. 4 (*Deed Cards* 3.e) ; p. 5 (*Effects* 1.b ; *"Composite" Effects* 1.b ; mana d'or, 2) ; p. 6 (5) ;
  p. 8 (*Combat with Enemies* 5 et 6) ; p. 9 (*Block Phase* 2.a ; *Assign Damage Phase* 3 à 5) ; p. 24 (*Defensive*, résistances).
  Texte des cartes : `docs/regles/cartes/actions-avancees.md` (version *Lost Legion* pour Marche revigorante et Besoin d'aide) et
  `cartes/unites.md` (Illusionnistes, Glaceurs d'Amotep, Mages d'Altem, Gardiens d'Altem). FAQ p2 (Au cœur du combat ; résistances
  et cartes jouées ensemble), FAQ p3 (résistances au Feu et à la Glace). Compendium, *Advanced Action Cards* (Blood Rage S1, S2,
  Heroic Tale S1, Ice Shield S1 à S4, In Need S1, Into the Heat S1, S2, Pure Magic S1, S2, Refreshing Walk S1, S3, Regeneration
  S1, S2, Song of Wind O1, S3, S4, O5), *Combat* Q7, S21, S23, S33, *Miscellaneous* S1, S9, *Silver Units* (Illusionists Q1, S2),
  *Gold Units* (Altem Mages Q1, Q2, S6, S7 ; Amotep Freezers S1, O2)
- ADR : [0008 Moteur d'effets](../decisions/0008-moteur-d-effets.md) (sortes nouvelles, premiers modificateurs d'ennemis)
- Crates touchées : arcanum-core (règles, contenu), arcanum-ui, arcanum-persistance (parties préparées), `assets/i18n/`

## Objectif

Neuf actions avancées de plus et les quatre capacités d'unités encore `Unsupported` font ce qu'elles disent : prendre une Blessure
pour frapper plus fort, réduire l'armure d'un ennemi ou l'empêcher d'attaquer, renforcer ses unités pour un combat, changer
l'élément ou la portée de ses attaques, se soigner en marchant, gagner de l'Influence par ses Blessures ou de la réputation par ses
recrues. La règle générale des **résistances aux effets** (MKUE p. 8, 5) est écrite une fois pour toutes.

Le lot est le premier des deux prévus par `PLAN.md` (Q1) : les **cartes de combat, d'unités et de soins**. Les huit actions
avancées de mana et de cartes (Rituel de sang, Décomposition, Maîtrise du cristal, Tempête de mana, Effet maximum, Apprentissage,
Entraînement, Don pour la magie renforcé) forment la spec 056 ; Don pour la magie de base reste dans la 054.

| Carte ou unité (fr.json) | Base | Renforcé | Ce qui est nouveau |
|---|---|---|---|
| Coup de sang (*Blood Rage*) | Attaque 2, ou Attaque 5 et une Blessure | Attaque 4, ou 9 et une Blessure | option avec Blessure en main |
| Bouclier de glace (*Ice Shield*), renforcé | (déjà jouée) | Parade de glace 3, armure de l'ennemi paré −3 (min. 1) | modificateur d'ennemi `ArmorReduction` |
| Marche revigorante (*Refreshing Walk*, LL) | Mouvement 2 et Soin 1 | Mouvement 4 et Soin 2 | deux sortes de points ; en combat, Mouvement seul |
| Mélodie du vent (*Song of Wind*) | Mouvement 2 ; plaine, désert, terre désolée −1 (min. 0) | Mouvement 2 ; ces terrains −2 (min. 0) ; mana bleu en plus : lacs franchissables à 0 | second mana payé |
| Magie pure (*Pure Magic*) | payer un mana : vert Mouvement 4, blanc Influence 4, bleu Parade 4, rouge Attaque 4 | idem 7 | option choisie par le mana payé |
| Récit héroïque (*Heroic Tale*) | Influence 3 ; Réputation +1 par unité recrutée ce tour | Influence 6 ; Gloire +1 et Réputation +1 par unité | modificateur de fin du tour par recrue |
| Régénération (*Regeneration*) | Soin 1, rendre disponible une unité de niveau I ou II | Soin 2, niveau III au plus | Soin et unité rendue disponible |
| Au cœur du combat (*Into the Heat*) | unités Attaque et Parade +2 ce combat ; pas de dégâts sur les unités ce tour | +3 | modificateurs d'unités |
| Besoin d'aide (*In Need*, LL) | Influence 3, +1 par Blessure en main ou sur une unité | Influence 5, +2 par Blessure | points calculés |
| Illusionnistes, 2e capacité *(mana blanc)* | un ennemi non fortifié n'attaque pas ce combat | — | modificateur d'ennemi `DoesNotAttack` |
| Glaceurs d'Amotep, 2e capacité *(mana bleu)* | un ennemi n'attaque pas ce combat, armure −3 (min. 1) | — | deux modificateurs d'ennemi |
| Mages d'Altem, 3e capacité *(mana noir)* | au choix : vos attaques de ce combat deviennent de feu glacé, ou de siège en plus | — | modificateurs d'attaque |
| Gardiens d'Altem, 3e capacité *(mana vert)* | vos unités ont toutes les résistances ce tour | — | modificateur d'unités |

## Règles

### Résistances aux effets (règle générale)

- **Effet d'une carte rouge ou bleue contre une résistance** (MKUE p. 8, *Combat with Enemies* 5.a et 5.b ; p. 24 ; FAQ p3 ;
  Compendium, *Combat* S33) : un modificateur d'ennemi qui vient d'une **carte rouge** (ou d'une capacité d'unité payée en mana
  rouge) n'est **pas posé** sur un ennemi résistant au Feu ; d'une **carte bleue** (ou d'une capacité payée en bleu), sur un
  ennemi résistant à la Glace. Les points d'Attaque et de Parade de la même carte comptent normalement (spec 032).
- **Couleur de l'effet** (MKUE p. 5, 1.b ; FAQ p2 ; Compendium, *Combat* Q7) : la couleur est celle de la carte qui porte l'effet,
  même jouée par Concentration (verte) : Bouclier de glace renforcé par Concentration reste bleu. C'est la couleur que porte déjà
  `ModifierSource` (ADR 0008) ; pour une unité, la couleur du mana de la capacité.
- **Ce qui ne vise pas un ennemi** (MKUE p. 8, 5.c) : les modificateurs du joueur et de ses unités (Au cœur du combat, rouge ;
  Gardiens d'Altem) ne sont jamais empêchés par une résistance.
- **Immunité arcanique** : aucun ennemi du jeu de base ne l'a (`enemies.ron`) ; hors périmètre.

### Modificateurs d'ennemis (ADR 0008)

Premiers modificateurs de `CombatEnemy::modifiers`, durée : le combat.

- **Armure réduite** (`ArmorReduction { by, min }`) : l'armure de l'ennemi baisse de `by`, jamais sous `min` (1). Elle compte
  partout où l'armure compte (attaque à distance ou d'attaque, groupe ciblé, spec 031). Plusieurs réductions s'ajoutent, toujours
  au minimum 1. Fortification et résistances ne changent pas.
- **N'attaque pas** (`DoesNotAttack`) : l'ennemi n'attaque pas dans ce combat : pas de parade possible contre lui (Compendium, *Ice
  Shield* S4), pas de dégâts à assigner, pas de créature invoquée s'il est Invocateur (Compendium, *Illusionists* Q1, S2 ; *Combat*
  S23). Il reste à vaincre pour sa gloire, et une cible d'attaque comme avant.

### Coup de sang

- **Blessure** (carte ; Compendium, *Blood Rage* S1, S2 ; *Combat* S21) : la seconde option met une Blessure **dans la main**
  (jamais sur une unité) et donne l'Attaque plus forte. La Blessure compte pour être assommé (spec 034) ; assommé par elle, le héros
  garde l'Attaque de la carte. Concentration renforcée ajoute son bonus à l'Attaque choisie (5 + 2).

### Bouclier de glace renforcé

- **Armure réduite** (carte ; Compendium, *Ice Shield* S1 à S4) : la carte se joue dans une **parade en cours** (comme sa Parade de
  glace 3) ; l'ennemi visé reçoit `ArmorReduction { by: 3, min: 1 }` quand la carte y est jouée, que la parade réussisse ou non
  (Q2 du compendium). Contre une créature invoquée, c'est la créature qui la reçoit, pas l'Invocateur (S3). Pas sur un ennemi
  résistant à la Glace (S1, règle générale).

### Marche revigorante et Régénération

- **Soin en combat** (MKUE p. 8, 6 ; Compendium, *Miscellaneous* S9) : aucun effet de soin ne se joue en combat, et la réserve de
  soin disparaît au début du combat. Marche revigorante (version LL) se joue en combat pour son **Mouvement seul** (utile avec
  Agilité, spec 052) ; Régénération ne se joue pas en combat (Q4).
- **Marche revigorante** (Compendium, *Refreshing Walk* S1, S3) : Mouvement dans sa réserve, Soin dans la réserve de soin
  (spec 042, Q1), qui s'ajoute au soin acheté au monastère. Jouée après le combat ou après l'action commencée, seul le Soin sert :
  la carte se joue quand même (règle « effet utilisable » de la 052).
- **Régénération** (carte ; Compendium, *Regeneration* S1, S2) : Soin dans la réserve, et l'unité choisie **en jouant la carte**
  redevient disponible tout de suite (comme Rajeunissement, `PlayCardOnUnit`, spec 042) ; pas de report. Sans unité épuisée du bon
  niveau, la carte se joue pour son Soin seul.

### Mélodie du vent

- **Coût réduit** (MKUE p. 7, 7.a ; Compendium, *Song of Wind* S4) : `TerrainCost` sur plaine, désert et terre désolée, `Reduce { by:
  1, min: 0 }` (renforcé `by: 2`), avec la règle de l'ordre le plus favorable de la 052 : avec Chemins de traverse de base, une
  plaine coûte 1.
- **Lacs** (Compendium, *Song of Wind* O1, S3, O5 ; spec 052, Q7) : l'effet renforcé a deux options ; la seconde coûte un **mana
  bleu en plus**, payé en jouant la carte (Q3), et pose aussi `Passable(Lake)` et `TerrainCost` lac `SetTo(0)`. S'arrêter sur un
  lac est permis, la retraite forcée en est le prix (spec 052).

### Magie pure

- **Option par couleur** (carte ; Compendium, *Pure Magic* S2) : l'effet de base a quatre options, chacune coûte un mana de sa
  couleur de base (vert, blanc, bleu, rouge) ; l'effet renforcé, le mana bleu de l'effet renforcé **plus** un mana de base pour
  l'option. Le mana d'or de jour vaut une couleur de base (MKUE p. 5, 2) ; le noir, jamais.
- **Effet utilisable** (Compendium, *Miscellaneous* S1 ; *Pure Magic* S2) : bleu (Parade) et rouge (Attaque) seulement quand ces
  points servent, donc en combat ; vert et blanc comme Mouvement et Influence (spec 052).
- **Résistances** (Compendium, *Pure Magic* S1) : ce ne sont que des points, aucune résistance ne les empêche.

### Récit héroïque et Besoin d'aide

- **Récit héroïque** (carte ; Compendium, *Heroic Tale* S1) : modificateur de fin du tour `EndOfTurnPerRecruit { fame, reputation }`
  (base 0 et +1, renforcé +1 et +1), multiplié par le nombre d'unités **recrutées ce tour** (`UnitRecruited`, avant ou après la
  carte ; une unité recrutée puis renvoyée compte). Une unité gagnée en récompense (ruines, spec 045) n'est pas recrutée (Q5).
  Sans recrue, rien. La réputation suit la piste et la case X comme Intimidation (spec 052).
- **Besoin d'aide** (version LL ; Compendium, *In Need* S1) : Influence 3 (5) plus 1 (2) par carte Blessure **en main ou sur une
  unité**, comptées quand la carte est jouée. Concentration renforcée ajoute son bonus au total.

### Au cœur du combat

- **Quand** (carte ; spec 052, Q6) : « au début du combat » se lit **tant qu'aucune unité n'a été activée dans ce combat**, en
  combat seulement, et avec au moins une unité (Q6).
- **Bonus des unités** (FAQ p2 ; Compendium, *Into the Heat* S1, S2) : `UnitBonus { attack, block }` (+2, renforcé +3), durée le
  combat : chaque option de capacité d'unité qui donne une Attaque ou une Parade, de tout genre et de tout élément, gagne le
  bonus ; une capacité sans Attaque ni Parade (Influence, Mouvement) rien.
- **Pas de dégâts sur les unités** (carte) : `NoDamageToUnits`, durée le tour : l'assignation des dégâts ne propose que le héros
  (MKUE p. 9, *Assign Damage Phase* 3 et 5).

### Capacités d'unités

- **Illusionnistes** (carte ; Compendium, *Illusionists* Q1, S2 ; *Combat* S23) : un ennemi **non fortifié** (ni Fortifié sur le
  jeton, ni défenseur d'un site, spec 043) reçoit `DoesNotAttack`. Cible : un ennemi pas vaincu, pas paré, qui attaque encore,
  pendant la phase à distance ou la phase de parade. Contre un Invocateur, avant que la créature soit tirée : comme le moteur la
  tire au début de la phase de parade (spec 033, `summoner_draws_a_brown_token_at_block_phase`), en phase à distance (Q7).
  Mana blanc : aucune résistance ne l'empêche.
- **Glaceurs d'Amotep** (carte ; Compendium, *Amotep Freezers* S1, O2) : l'ennemi visé, fortifié ou non, reçoit `DoesNotAttack`
  et `ArmorReduction { by: 3, min: 1 }` ; pendant la phase à distance, la phase de parade (mêmes cibles que les Illusionnistes) ou
  la phase d'attaque (armure seule, un ennemi pas vaincu) (Q7). Mana bleu : rien sur un ennemi résistant à la Glace.
- **Mages d'Altem** (carte ; Compendium, *Altem Mages* Q1, Q2, S6, S7) : mana noir, donc de nuit seulement (MKUE p. 5) ; deux
  options, durée le combat, **sans effet rétroactif** :
  - `AttacksBecome(ColdFire)` : toute Attaque ajoutée ensuite (carte, de côté, unité, à distance ou de siège) est de feu glacé ;
  - `AttacksGainSiege` : toute Attaque ajoutée ensuite est aussi de siège ; en phase à distance, une Attaque simple se joue alors
    comme une Attaque de siège. En phase d'attaque, sans effet (Compendium, *Combat* S6).
- **Gardiens d'Altem** (carte) : `UnitsAllResistances`, durée le tour : chaque unité du joueur est résistante au physique, au Feu
  et à la Glace (donc au feu glacé) pour l'assignation des dégâts (MKUE p. 9, 4.b ; spec 042).
- **Recrutement** : Mages et Gardiens d'Altem ne se recrutent qu'en cité (spec 062) ; leurs cas sont joués par une unité posée par
  le test (Q8).

### Exemple : Bouclier de glace et Illusionnistes contre deux Orques

Combat contre des Prowlers (`enemies.ron` : armure 3, attaque physique 4) et des Orc Summoners (armure 4, Invocation), avec des
Illusionnistes disponibles et Bouclier de glace en main :

1. Phase à distance : Illusionnistes activés (mana blanc) sur les Orc Summoners : `DoesNotAttack` ; au début de la phase de parade,
   aucune créature n'est tirée.
2. Phase de parade : parade des Prowlers, Bouclier de glace renforcé (mana bleu) : Parade de glace 3 contre 4, la parade échoue,
   mais l'armure des Prowlers passe à 1.
3. Dégâts des Prowlers : 4, assignés comme aujourd'hui.
4. Phase d'attaque : Attaque 1 bat les Prowlers, Attaque 4 les Summoners.

## Modèle

Termes du glossaire : Effet en cours (*Modifier*), Résistance, Fortifié. Termes à ajouter avant validation : *N'attaque pas*
(`DoesNotAttack`) et *Armure réduite* (`ArmorReduction`).

Données nouvelles (noms indicatifs, forme dans l'ADR 0008) :

- **`CombatEnemy::modifiers`** : la liste prévue par l'ADR 0008, vide jusqu'ici ; `ArmorReduction`, `DoesNotAttack`. Une fonction
  `effective_armor` remplace la lecture directe de l'armure du jeton là où le combat la compare.
- **`ModifierKind`** : `UnitBonus { attack, block }`, `NoDamageToUnits`, `EndOfTurnPerRecruit { fame, reputation }`,
  `AttacksBecome(Element)`, `AttacksGainSiege`, `UnitsAllResistances`. Durée `Combat` pour `UnitBonus` et les deux modificateurs
  d'attaque (déjà prévue par l'ADR 0008).
- **`Effect`** : `AttackWithWound` (Attaque, une Blessure en main), `MoveAndHeal { move, heal }`, `HealAndReady { heal, level }`,
  `InfluencePerWound { base, per }` ; une option peut demander un **mana en plus** d'une couleur (Magie pure, Mélodie du vent
  renforcée), comme `PayForCrystal` (spec 025) ; une option peut poser un **modificateur d'ennemi** sur la cible de la parade en
  cours (Bouclier de glace) ou sur l'ennemi choisi (unités).
- **`CardPlay::Strong`** : un second mana facultatif (`extra_mana`), pour le mana en plus de l'option renforcée (annoncé par la
  spec 052, Hors périmètre).
- **`Action::ActivateUnit`** : une cible ennemie facultative (`enemy: Option<EnemyToken>`) pour les Illusionnistes et les Glaceurs.
  Les littéraux `ActivateUnit { … }` des tests et du joueur scripté sont mis à jour (signalé dans la PR).
- **Contenu** : les 9 cartes et les 4 capacités quittent `Unsupported` / `UnsupportedMana` dans `advanced_actions.ron` et
  `units.ron`.

## Actions et événements

| Action | Validation (refusée si…) | Événements produits |
|---|---|---|
| `PlayCard` (existante) | règle « effet utilisable » (052) ; Au cœur du combat après une unité activée dans ce combat ou sans unité ; mana en plus absent ou d'une autre couleur ; Bouclier de glace renforcé sans parade en cours | `CardPlayed` (points, modificateurs du joueur et de l'ennemi) ; `WoundFromCard` (nouveau) pour la Blessure de Coup de sang |
| `PlayCardOnUnit` (existante) | Régénération : unité pas épuisée ou de niveau trop haut ; en combat | `CardPlayed`, `UnitReadied` (existants) |
| `ActivateUnit` (existante, champ `enemy`) | cible absente, vaincue, déjà parée ou hors phase (Q7) ; Illusionnistes contre un ennemi fortifié ; Invocateur dont la créature est tirée ; capacité noire de jour | `UnitActivated` (pose les modificateurs, sauf résistance) |
| `AssignDamage` (existante) | vers une unité avec `NoDamageToUnits` | inchangés |
| `EndTurn` (existante) | inchangée | `ReputationChanged`, `FameGained` (Récit héroïque), avant la défausse comme Intimidation |

| Événement | Effet de `apply` | Révèle une information ? | Point de non-retour ? |
|---|---|---|---|
| `CardPlayed`, `UnitActivated` (existants) | en plus : modificateurs d'ennemi posés sur la cible, sauf résistance ; bonus d'unité appliqué aux points de la capacité | non | non |
| `WoundFromCard { card }` (nouveau, nom indicatif) | une Blessure de la pile en main ; compte dans les Blessures du combat (`wounds_to_hand`, assommé, spec 034) ; `WoundsTaken` porte un ennemi et ne convient pas | non | non |
| `TurnEnded`, `CombatEnded` (existants) | retirent aussi les nouvelles sortes de leur durée ; la fin du combat efface `CombatEnemy::modifiers` avec le combat | non | inchangé |

Aucun hasard nouveau : les tirages des parcours ne bougent pas (ADR 0007).

## Comportements attendus

### Résistances aux effets

- **C1** — Bouclier de glace renforcé en parade contre des Ice Mages (`enemies.ron` : armure 6, attaque de glace 5, résistance à la
  Glace) : la Parade compte comme aujourd'hui (de glace contre une attaque de glace, inefficace, spec 032), aucun
  `ArmorReduction` posé, armure 6 (`ice_resistance_ignores_blue_effects`) — MKUE p. 8,
  5.b ; Compendium, *Ice Shield* S1.
- **C2** — Glaceurs d'Amotep (mana bleu) sur des Ice Mages : ni `DoesNotAttack` ni armure réduite ; sur des Fire Mages (résistance
  au Feu) : les deux (`freezers_blocked_by_ice_resistance_only`) — MKUE p. 8, 5.b ; Compendium, *Amotep Freezers* S1.
- **C3** — Bouclier de glace renforcé joué par Concentration renforcée contre des Fire Mages : armure réduite (couleur de Bouclier
  de glace, bleue) (`boosted_card_keeps_its_color`) — FAQ p2 ; Compendium, *Combat* Q7.
- **C4** — Au cœur du combat (rouge) avec un ennemi résistant au Feu dans le combat (Fire Mages) : le bonus des unités s'applique
  (`effects_on_units_ignore_enemy_resistances`) — MKUE p. 8, 5.c.

### Modificateurs d'ennemis

- **C5** — Prowlers à armure réduite à 1 : Attaque 1 les bat en phase d'attaque (`reduced_armor_counts_in_attack`) — carte.
- **C6** — Guardsmen (armure 7, Fortifié) : Bouclier de glace renforcé en parade, puis Glaceurs d'Amotep en phase d'attaque :
  armure 1, pas moins ; toujours fortifiés
  (`armor_reductions_add_up_to_minimum_one`) — carte.
- **C7** — Ennemi `DoesNotAttack` : pas de parade possible contre lui, pas de dégâts à assigner ; il reste dans le combat, et la
  victoire demande toujours de le vaincre (`enemy_that_does_not_attack`) — Compendium, *Ice Shield* S4.
- **C8** — Les modificateurs d'ennemis disparaissent avec le combat ; un retour en arrière avant `UnitActivated` les retire ; une
  sauvegarde rechargée les retrouve (`enemy_modifiers_follow_combat_undo_and_reload`) — ADR 0008, spec 024.

### Coup de sang

- **C9** — Phase d'attaque contre des Prowlers : option 2 de base, Attaque 5 physique dans l'attaque en cours et une Blessure en
  main (`blood_rage_wound_for_attack`) — carte ; Compendium, *Blood Rage* S1.
- **C10** — Héros à quatre Blessures reçues dans ce combat : la Blessure de Coup de sang l'assomme (spec 034), l'Attaque 5 reste
  dans l'attaque en cours (`blood_rage_knock_out_keeps_attack`) — Compendium, *Combat* S21.
- **C11** — Coup de sang renforcé par Concentration renforcée, option 2 : Attaque 9 + 2 = 11, une Blessure
  (`concentration_boosts_blood_rage`) — MKUE p. 5, *"Composite"* 1.b.

### Bouclier de glace renforcé

- **C12** — Parade des Prowlers : Parade de glace 3 contre 4, parade ratée, armure des Prowlers 1 (`ice_shield_reduces_armor_even_if_block_fails`)
  — Compendium, *Ice Shield* Q2.
- **C13** — Contre la créature invoquée par des Orc Summoners : l'armure de la créature baisse, pas celle des Summoners
  (`ice_shield_on_summoned_monster`) — Compendium, *Ice Shield* S3.

### Marche revigorante et Régénération

- **C14** — En phase de mouvement, Marche revigorante de base : Mouvement 2 et Soin 1 ; le Soin jette une Blessure de la main
  (`refreshing_walk_moves_and_heals`) — carte.
- **C15** — En combat avec Agilité jouée avant, Marche revigorante renforcée : Mouvement 4 dans la réserve, aucun Soin
  (`refreshing_walk_in_combat_moves_only`) — carte (LL) ; MKUE p. 8, 6.
- **C16** — Après le combat, Marche revigorante se joue, son Soin sert (`refreshing_walk_after_combat_heals`) — Compendium,
  *Refreshing Walk* S3.
- **C17** — Paysans épuisés (`units.ron` : niveau I) : Régénération de base les rend disponibles et met Soin 1 dans la réserve ;
  des Glaceurs d'Amotep (niveau III) ne sont pas une cible de base, le sont renforcé (`regeneration_readies_and_heals`) — carte.
- **C18** — Régénération ne se joue pas en combat (`regeneration_not_in_combat`) — MKUE p. 8, 6.

### Mélodie du vent

- **C19** — De jour, Mélodie du vent de base : plaine 1, désert 4, terre désolée 3 ; renforcée : plaine 0, désert 3, terre désolée
  2 (`song_of_wind_reduces_open_terrains`) — carte.
- **C20** — Avec Chemins de traverse de base, dans les deux ordres : plaine 1 (`song_of_wind_with_path_finding`) — Compendium,
  *Song of Wind* S4.
- **C21** — Renforcée avec le mana bleu en plus : le lac `(2, -4)` du bac à sable coûte 0 et figure parmi les cases atteignables ;
  sans le mana bleu, il reste infranchissable (`strong_song_of_wind_crosses_lakes`) — carte ; spec 052.

### Magie pure

- **C22** — Phase de mouvement, base, mana vert payé : Mouvement 4 ; mana bleu : pas une action légale hors combat
  (`pure_magic_option_by_paid_color`) — carte ; Compendium, *Pure Magic* S2.
- **C23** — En parade, renforcée (mana bleu) et mana bleu en plus : Parade 7 physique ; de jour, un dé d'or payé comme rouge en
  phase d'attaque donne Attaque 7 (`strong_pure_magic_two_mana`) — Compendium, *Pure Magic* S2 ; MKUE p. 5, 2.

### Récit héroïque et Besoin d'aide

- **C24** — Au village, Récit héroïque de base puis Paysans recrutés : réputation +1 à la fin du tour ; renforcé : gloire +1 et
  réputation +1 (`heroic_tale_per_recruit`) — carte.
- **C25** — Unité recrutée avant la carte : comptée ; unité recrutée puis renvoyée au recrutement d'une autre : les deux comptent
  (`heroic_tale_counts_all_recruits`) — Compendium, *Heroic Tale* S1.
- **C26** — Deux Blessures en main et une unité blessée : Besoin d'aide de base donne Influence 6, renforcée 11
  (`in_need_counts_wounds`) — carte (LL).

### Au cœur du combat

- **C27** — Au début du combat, Au cœur du combat de base, puis Paysans activés pour Attaque 2 : Attaque 4 dans l'attaque en cours ;
  pour Influence 2 : rien de plus (`into_the_heat_boosts_unit_attack_and_block`) — FAQ p2 ; Compendium, *Into the Heat* S1.
- **C28** — Après une unité activée dans ce combat, Au cœur du combat ne se joue que de côté ; sans unité, aussi
  (`into_the_heat_only_at_start`) — carte ; Q6.
- **C29** — Dégâts non parés avec Au cœur du combat joué : les actions légales n'assignent qu'au héros
  (`into_the_heat_no_damage_to_units`) — carte.

### Capacités d'unités

- **C30** — Illusionnistes (mana blanc) en phase à distance sur des Orc Summoners : aucune créature tirée au début de la phase de
  parade ; sur des Diggers (Fortifié) : pas une action légale (`illusionists_cancel_unfortified_attack`) — carte ; Compendium,
  *Illusionists* S2.
- **C31** — Glaceurs d'Amotep en phase d'attaque sur des Guardsmen : armure 4 (`freezers_reduce_armor_in_attack_phase`) — carte ; Q7.
- **C32** — De nuit, Mages d'Altem option feu glacé, puis Paysans activés pour Attaque 2 contre des Fire Mages (résistance au Feu
  seule) : Attaque de feu glacé 2, comptée en plein ; une Attaque jouée avant la capacité reste physique
  (`altem_mages_cold_fire_not_retroactive`) — Compendium, *Altem Mages* Q2 ; MKUE p. 9, 4.a.
- **C33** — De nuit, Mages d'Altem option siège, phase à distance contre des Diggers (fortifiés) : Rage (Attaque 2) se joue et
  compte comme Attaque de siège 2 (`altem_mages_siege_in_ranged_phase`) — Compendium, *Altem Mages* S7.
- **C34** — De jour, la capacité noire des Mages d'Altem n'est pas une action légale (`altem_mages_black_only_at_night`) — MKUE p. 5.
- **C35** — Gardiens d'Altem (mana vert), puis Wolf Riders (attaque physique 3) assignés aux Paysans (armure 3) : absorbés sans
  Blessure (`altem_guardians_give_all_resistances`) — MKUE p. 9, 4.b.

### Contenu et vue

- **C36** — Les 9 cartes et les 4 capacités ne sont plus `Unsupported` ; restent `Unsupported` les 8 cartes de la spec 056 (Don pour
  la magie compris) (`second_lot_is_supported`) — contenu.
- **C37** — La vue du combat donne pour chaque ennemi ses modificateurs et son armure effective (`Combat` de `PlayerView`, déjà
  public) (`view_shows_enemy_modifiers`).

## Cas limites et refus

- **L1** — Bouclier de glace renforcé hors parade en cours : seule la carte de côté ou la base (refus `ActionError::NotLegal`).
- **L2** — `ActivateUnit` des Illusionnistes sans cible, ou sur un ennemi vaincu ou déjà paré : refusée.
- **L3** — Magie pure renforcée sans le mana en plus : refusée.
- **L4** — Deux Coups de sang, deux Au cœur du combat : impossible, un exemplaire de chaque carte ; deux Illusionnistes (2
  exemplaires) sur deux ennemis : permis, un test le garde.

## Information cachée

Rien de caché : modificateurs du joueur (`PlayerView::modifiers`, spec 052) et des ennemis (`PlayerView::combat`, public, C37).
Champ ajouté : l'armure effective des ennemis du combat, calculée dans la vue (C37).

## Libellés (i18n)

Nouvelles clés (fr ; en à l'avenant) ; les textes `card.<carte>.basic_text` / `strong_text` et `unit.<unité>.ability_*` existent
déjà (relus dans `assets/i18n/fr.json` à la rédaction) :

- `modifier.unit_bonus` : « unités : Attaque et Parade +{bonus} » ; `modifier.no_damage_to_units` : « pas de dégâts sur les unités » ;
  `modifier.end_of_turn_per_recruit` : « par unité recrutée ce tour : réputation +{reputation} » ;
  `modifier.end_of_turn_per_recruit_fame` : « par unité recrutée ce tour : gloire +{fame}, réputation +{reputation} » ;
  `modifier.attacks_cold_fire` : « attaques de feu glacé » ; `modifier.attacks_siege` : « attaques de siège en plus » ;
  `modifier.units_all_resistances` : « unités : toutes les résistances ».
- `combat.enemy.armor_reduced` : « Armure {armor} (au lieu de {printed}) » ; `combat.enemy.does_not_attack` : « n'attaque pas ».
  Sous `combat.enemy`, à côté de `blocked` « paré » et `defeated` « vaincu » (relus).
- `card.blood_rage.option_wound` : « Attaque {amount} et une Blessure » (bouton de la seconde option ; fr « Attaque 5 et une
  Blessure », en « Attack 5 and a Wound »).
- Ajoutées au fil écran (2026-11-12), pour nommer les effets nouveaux sur les boutons d'options et le choix de l'unité :
  `effect.move_and_heal` « Mouvement {movement} et Soin {heal} » ; `effect.heal_and_ready` « Soin {heal} ; rendre disponible une
  unité de niveau {level} au plus » ; `effect.influence_per_wound` « Influence {base}, +{per} par Blessure » ;
  `modifier.armor_reduction` « Armure −{by} (min. {min}) » (option d'une capacité qui vise un ennemi).

## Interface

- **Ennemis du combat** : armure réduite et « n'attaque pas » sur la carte de l'ennemi (C37, `PlayerView::combat`) ; un ennemi qui
  n'attaque pas n'a ni bouton « Parer » ni enjeu de parade (C7, actions légales). Placement : vérification à l'œil.
- **Effets en cours** : les nouvelles sortes dans la zone de jeu, comme en 052 (C27, C32, C35, `PlayerView::modifiers`).
- **Options de cartes** : Coup de sang, Magie pure (une option par couleur), Mélodie du vent renforcée (option lacs avec mana bleu en
  plus) par les boutons d'options existants (spec 073) ; option illégale grisée (C22, C28, actions légales). Le second mana se
  choisit comme le premier (fenêtre de paiement de la spec 022).
- **Capacités d'unités ciblant un ennemi** : après le clic sur la capacité, choix de l'ennemi parmi les cibles légales, comme le
  choix d'une unité pour Rajeunissement (C30, C31, actions légales `ActivateUnit`).
- **Régénération** : choix de l'unité comme Rajeunissement (C17).
- **Assignation des dégâts** : sans bouton vers une unité avec Au cœur du combat (C29, actions légales).

### Raccourcis clavier

Aucun : tout passe par les boutons existants des cartes et des unités.

## Effets sur les specs déjà implémentées

- **031, 032** : l'armure comparée devient l'armure effective ; Attaque simple jouable en phase à distance avec `AttacksGainSiege`.
- **033** : un Invocateur `DoesNotAttack` ne tire pas de créature.
- **034** : Coup de sang met une Blessure en main en combat (compte pour assommé).
- **042** : `ActivateUnit` prend une cible ennemie ; bonus et résistances données aux unités ; 4 capacités quittent
  `UnsupportedMana` (spec 042, Q2 close).
- **050** : 9 actions avancées quittent `Unsupported`.
- **052** : sortes de modificateurs et `extra_mana` ; ordre de fin du tour inchangé (Récit héroïque avec Intimidation).

## Hors périmètre

- Rituel de sang, Décomposition, Maîtrise du cristal, Tempête de mana, Effet maximum, Apprentissage, Entraînement, Don pour la magie
  renforcé : spec 056 (Q1). Don pour la magie de base : spec 054.
- Immunité arcanique, attaques Encombrantes et autres capacités des extensions : hors périmètre.
- Recrutement des unités de cité : spec 062.

## Plan de développement

Moteur et écran (issue privée 118) : deux fils sur la même branche, plan [`plans/053-actions-avancees.md`](plans/053-actions-avancees.md) :

1. **Moteur** : modificateurs d'ennemis et résistances, cartes, capacités d'unités, contenu, vue, persistance (C1 à C37).
2. **Écran** : libellés, ennemis du combat, cibles ennemies des unités, options à second mana, vérification à l'œil, docs, bilan.

## Décisions (2026-11-12)

Mathieu accepte toutes les réponses par défaut (« D'accord avec les défauts »), appliquées dans le texte ci-dessus :

1. **Q1 — Découpage.** `PLAN.md` prévoyait « un ou deux lots ». 17 cartes et 4 capacités dans une spec, c'est plus du double de
   la 052. Proposé : ce lot (combat, unités, soins : 9 cartes, 4 capacités) en 053, et une **spec 056 « Actions avancées : mana et
   cartes »** pour les 8 autres, insérée juste après dans `PLAN.md`. **Décision : oui.**
2. **Q2 — Bouclier de glace sur une parade ratée.** Le compendium (Q2, d'après la FAQ officielle) réduit l'armure même si la parade
   échoue. **Décision : oui (C12).**
3. **Q3 — Mélodie du vent, mana bleu.** Le compendium (O5) laisse ouvert le moment du paiement ; proposé : payé en jouant la carte,
   comme une seconde option. **Décision : en jouant la carte.**
4. **Q4 — Régénération en combat.** La carte porte un Soin (interdit en combat, MKUE p. 8, 6) ; rendre une unité disponible serait
   utile en combat. Proposé : la carte ne se joue pas du tout en combat (comme le `ReadyUnit` des Herboristes aujourd'hui).
   **Décision : pas en combat.**
5. **Q5 — Récit héroïque et unité gagnée en récompense.** Proposé : seule une unité recrutée par Influence compte. **Décision :
   oui.**
6. **Q6 — « Au début du combat ».** Proposé : tant qu'aucune unité n'a été activée dans ce combat (spec 052, Q6), et seulement avec
   au moins une unité. **Décision : oui.**
7. **Q7 — Moment des capacités qui ciblent un ennemi.** Proposé : Illusionnistes en phase à distance ou de parade (avant le tirage
   de la créature contre un Invocateur, donc en phase à distance) ; Glaceurs aussi en phase d'attaque, pour l'armure seule (lecture
   large du compendium, O2). **Décision : oui.**
8. **Q8 — Unités de cité.** Mages et Gardiens d'Altem ne se recrutent qu'en cité (spec 062). Proposé : les coder ici quand même
   (spec 052, Q3), testés avec une unité posée par le test. **Décision : oui.**

## Questions ouvertes

Aucune.
