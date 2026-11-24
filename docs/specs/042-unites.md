# 042 — Unités

- Statut : implémentée (2026-11-04) ; validée le 2026-11-03, propositions de Q1 à Q3 retenues par Mathieu
- Jalon : J4
- Dépend de : 013 (jouer des cartes), 022 et 025 (payer le mana, gagner jetons et cristaux), 024 (retour en arrière), 031 à 033
  (combat, éléments et résistances, capacités des ennemis), 034 (soins, assommé), 035 (jetons de commandement), 040 (interaction,
  influence signée), 050 (offre d'unités, unités en données), 063 (décompte final) ; toutes implémentées
- Livret : MKUE p. 5 (*Using Units* 2 à 7 et 9) ; p. 4 (2.g, deuxième puce : unités disponibles à la nouvelle manche) ; p. 8
  (*Interacting with Locals* 4.a et 5 ; *Combat with Enemies* 4 et 6) ; p. 9 (*Ranged and Siege Attack Phase* 3.a ; *Block Phase*
  4.a ; *Assign Damage Phase* 1 à 5) ; p. 10 (6.a, 6.b) ; p. 11 (*Wounds and Healing* 2 et 3) ; p. 15 (*The Greatest Leader*) ;
  p. 24 (*Unit Resistances*) ; FAQ p2 (unité et carte jouée), p3 (unité à deux Blessures, renvoyer une unité, dégâts assignés une
  fois) ; Compendium, *Silver Units* S4, S5, S9, S10, *HERBALIST* S1, *UTEM GUARDSMEN* S1, *UTEM SWORDSMEN* S2 ; *Gold Units*
  *ALTEM MAGES* Q3 ; *Combat* S34, S35, S38, S45, S47 ; *Healing* Q1. Texte des cartes : `docs/regles/cartes/unites.md` (version
  *Lost Legion* des Gardes d'Utem et des Gardiens d'Altem)
- Crates touchées : arcanum-core (règles, contenu), arcanum-ui, arcanum-persistance (une partie préparée), `assets/i18n/`

## Objectif

Le héros recrute des **unités** dans l'offre d'unités, en interagissant avec les habitants d'un village ou d'un monastère, autant
qu'il a de jetons de commandement. Une unité disponible s'**active** une fois par manche pour une de ses capacités : attaque,
parade, mouvement, influence, soin, mana. En combat, les dégâts non parés peuvent être **assignés aux unités** avant le héros :
elles sont blessées (ou détruites), selon leur armure et leurs résistances. Les points de soin soignent une Blessure de la main ou
une unité blessée. Au décompte final, Le Commandant compte les unités.

## Règles

### Avoir des unités

- **Zone des unités** (MKUE p. 5, 2 et 3) : les unités recrutées sont devant le joueur, jamais dans sa main, son deck ni sa
  défausse. Il en a au plus autant que de **jetons de commandement** (spec 035 : 1 au niveau 1, puis selon le niveau).
- **Deux états indépendants** (MKUE p. 5, 3.a à 3.c ; Compendium, *Silver Units* S9) : **disponible** ou **épuisée** ; **blessée**
  (une ou deux Blessures) ou non. Les quatre combinaisons existent.
- **Recruter** (MKUE p. 8, 4.a ; p. 5, 4) : pendant une interaction (spec 040), une unité de l'offre dont un des lieux de
  recrutement est le site de l'interaction (village ou monastère : les autres sites arrivent avec 043 et 062), pour son coût en
  influence. Elle arrive disponible et non blessée, utilisable aussitôt (4.a, deuxième puce). On peut en recruter plusieurs dans la
  même interaction (MKUE p. 8, 5 ; Compendium, *Silver Units* Q3). L'offre d'unités n'est **pas complétée** (spec 050).
- **Influence suffisante** : l'influence du tour (signée, spec 040) doit atteindre le coût ; elle baisse du coût.
- **Renvoyer** (MKUE p. 5, 4.a ; FAQ p3 ; Compendium, *Silver Units* S4) : si tous les jetons de commandement sont pris, recruter
  demande de renvoyer une unité, au choix ; elle quitte la partie (ni défausse ni pioche), Blessures comprises. Renvoyer n'est
  possible qu'à ce moment.
- **Jetons gagnés en fin de tour** (FAQ p3 ; spec 035) : le jeton d'un changement de niveau n'arrive qu'à la fin du tour ; il ne
  sert pas à recruter pendant le tour.
- **Nouvelle manche** (MKUE p. 4, 2.g ; p. 5, 9 ; p. 11, *Wounds and Healing* 2.b) : toutes les unités redeviennent disponibles,
  blessées comprises (elles restent blessées). La fin du **tour** ne les rend pas disponibles.

### Activer une unité

- **Activer** (MKUE p. 5, 5) : une unité **disponible et non blessée** s'active pour **une** de ses capacités ; elle est épuisée
  jusqu'à la fin de la manche. Une capacité précédée d'un mana se paie d'abord d'un mana de cette couleur, comme un effet renforcé
  (spec 022 : dé de la Source, un par tour ; cristal ; jeton, spec 025 ; dé doré de jour pour toute couleur de base).
- **Quand** : une capacité se joue quand une carte qui aurait le même effet se jouerait (specs 013, 031, 034, 040). Mouvement dans
  la phase de déplacement ; Influence hors combat (en interaction, elle compte) ; Attaque dans une attaque en cours, Attaque à
  distance ou de siège dès la première phase (fortifiés : siège seulement, spec 031), Parade dans une parade en cours ; gain de
  mana à tout moment. Une « Attaque ou Parade » est une capacité à deux options, comme Rage. Les points s'ajoutent à ceux des
  cartes, avec leur élément (spec 032).
- **Soin et préparer une unité** : pas en combat (MKUE p. 8, *Combat with Enemies* 6 ; Compendium, *HERBALIST* S1). Le scan des
  Herboristes porte l'icône de soin sur ces deux capacités, pas sur le jeton vert, qui se prend même en combat.
- **Une unité n'est pas une carte** (FAQ p2 ; Compendium, *Silver Units* S5) : Concentration renforcée et tout « +X à la prochaine
  carte » ne s'appliquent pas à une capacité d'unité. Une unité ne se joue jamais de côté.
- **Préparer une unité** (MKUE p. 5, 6 ; Compendium, *Silver Units* S10) : une unité **épuisée** du joueur, du niveau permis,
  redevient disponible (blessée, elle reste inactivable). Les Herboristes ne peuvent pas se préparer eux-mêmes : ils sont
  disponibles au moment de l'activation, donc pas une cible. Effets : Herboristes (niveau I ou II), Rajeunissement de Norowas (de
  base I ou II, renforcé I à III ; option promise par la spec 034).
- **Épéistes d'Utem, seconde capacité** (Compendium, *UTEM SWORDSMEN* S2) : Attaque ou Parade 6, et l'unité reçoit une Blessure
  aussitôt. Ce n'est pas un dégât assigné : un ennemi toxique ou paralysant n'y change rien.
- **Gardes d'Utem et Gardiens d'Altem, parade** (version *Lost Legion*, `cartes/unites.md` ; Compendium, *UTEM GUARDSMEN* S1) :
  contre un ennemi Vif (spec 033), la parade compte double (Parade 4 vaut 8), puis l'efficacité de l'élément s'applique comme
  d'habitude.
- **Mages d'Altem** : « deux jetons de mana de n'importe quelles couleurs » : les six couleurs, noir compris, même de jour, où il ne
  sert à rien (Compendium, *ALTEM MAGES* Q3) ; « Feu glacé 5, 7 avec un mana bleu ou rouge, 9 avec les deux » : chaque mana payé
  en plus ajoute 2.

Capacités codées par cette spec, ou `Unsupported` (non activables, texte affiché « pas encore disponible ») jusqu'au moteur
d'effets (spec 052), parce qu'elles durent le tour ou visent un ennemi (Q2) :

| Unité | Codées | `Unsupported` |
|---|---|---|
| Paysans, Arbalétriers d'Utem, Épéistes d'Utem, Golems gardiens, Moines (les trois), Golems de feu, Golems de glace, Mages de feu, Mages de glace, Fusiliers d'Amotep, Catapultes | toutes | — |
| Herboristes | toutes (Soin 2, préparer une unité, jeton vert) | — |
| Gardes d'Utem | toutes (Parade 4 qui compte double contre Vif) | — |
| Forestiers | Parade 3 | Mouvement 2 et terrains moins chers ce tour |
| Illusionnistes | Influence 4, cristal blanc | (blanc) un ennemi non fortifié n'attaque pas |
| Geleurs d'Amotep | Attaque ou Parade 5 | (bleu) un ennemi n'attaque pas, armure −3 |
| Mages d'Altem | deux jetons, Feu glacé 5 à 9 | (noir) attaques de feu glacé ou de siège ce combat |
| Gardiens d'Altem | Attaque 5, Parade 8 (double contre Vif) | (vert) toutes les résistances aux unités ce tour |

### Unités en combat : assigner les dégâts

- **Ordre** (MKUE p. 9, *Assign Damage Phase* 1.b) : le joueur traite les ennemis non parés **un par un, dans l'ordre qu'il
  choisit** ; un ennemi commencé est fini avant le suivant.
- **D'abord les unités, puis le héros** (MKUE p. 9, 3 et 5 ; FAQ p3 ; Compendium, *Combat* S34) : pour l'ennemi en cours, le joueur
  assigne les dégâts restants à une unité, puis éventuellement à une autre, puis tout le reste au héros (comme aujourd'hui, une
  Blessure par tranche d'armure, spec 031). Une fois le héros choisi, plus d'unité pour cet ennemi.
- **Unité qui peut recevoir des dégâts** (MKUE p. 9, 4 ; p. 11, 2.a ; FAQ p3 ; Compendium, *Silver Units* S9, *Combat* S35) : non
  blessée, disponible ou épuisée, et sans dégâts déjà assignés **dans ce combat**.
- **Unité sans résistance à l'attaque** (MKUE p. 9, 4.a) : elle reçoit une Blessure, quels que soient les dégâts, et les dégâts
  restants baissent de son armure.
- **Unité résistante** (MKUE p. 9, 4.b ; p. 24, *Unit Resistances*) : résistance physique contre une attaque physique, au feu contre
  le feu, à la glace contre la glace, **les deux** contre le feu glacé. Les dégâts baissent d'abord de son armure sans Blessure ;
  s'il en reste, elle reçoit une Blessure et ils baissent encore de son armure. Si elle n'est pas blessée, elle ne reçoit plus de
  dégâts de ce combat mais reste activable (Compendium, *Combat* S35, Q42).
- **Toxique** (MKUE p. 10, 6.a) : une unité qui serait blessée reçoit **deux** Blessures. **Paralysant** (6.b ; Compendium,
  *Combat* S47) : une unité qui serait blessée est **détruite** (elle quitte la partie, son jeton de commandement se libère), après
  avoir absorbé les dégâts comme d'habitude. **Brutal** : les dégâts sont déjà doublés (spec 033).
- **Sans unité possible** (spec 031, Q3) : quand aucune unité ne peut plus recevoir de dégâts, les ennemis restants se résolvent
  seuls, comme aujourd'hui, dans l'ordre du combat (l'ordre ne change alors rien, spec 034). Un combat sans unité se joue donc
  exactement comme avant cette spec.
- **Assommé** (MKUE p. 9, 5.c ; Compendium, *Combat* S45) : seules les Blessures en main comptent ; une fois assommé, le joueur peut
  toujours assigner des dégâts à ses unités, et elles peuvent encore s'activer (5.c, dernière puce).

### Unités blessées et soins

- **Unité blessée** (MKUE p. 11, *Wounds and Healing* 2) : une ou deux Blessures posées sur la carte ; elle ne s'active pas et ne
  reçoit pas de dégâts. Ses Blessures ne sont ni en main ni dans le deck (La Victime ne les compte pas, spec 063).
- **Points de soin** (MKUE p. 11, 3.c et 3.d) : les Soins X joués s'additionnent en une **réserve de points de soin**, comme
  le mouvement et l'influence (Q1). Ils se dépensent hors combat : 1 point jette une Blessure de la main (spec 034) ; un nombre de
  points égal au **niveau** de l'unité retire **une** Blessure d'une unité (FAQ p3 ; Compendium, *Healing* Q1). Les points non
  dépensés sont perdus quand un combat commence et à la fin du tour.
- Achat de points de soin au village et au monastère : spec 041, qui alimente la même réserve.

### Décompte final

- **Le Commandant** (MKUE p. 15, *The Greatest Leader*) : la somme des niveaux des unités, une unité blessée comptant pour la
  moitié de son niveau arrondie à l'inférieur. Sans le titre (+3), comme les autres lignes (spec 063).

### Exemple : recruter au village, puis combattre avec ses unités

Bac à sable, Jour, niveau 3 (deux jetons de commandement), figurine sur le village (2, −3), réputation 0. Les Paysans (coût 4,
niveau I, armure 3) sont déjà dans la zone des unités, disponibles. L'offre d'unités contient des Gardes d'Utem (coût 5, niveau II,
armure 5). Des Rôdeurs (attaque 4, armure 3) sont sur la colline voisine (3, −3).

| Étape | Action | Résultat |
|---|---|---|
| 1 | Interagir ; Promesse (Influence 2) et Menace (Influence 2) | influence 4 |
| 2 | Activer les Paysans : Influence 2 | influence 6 ; Paysans épuisés |
| 3 | Recruter les Gardes d'Utem | influence 1 ; Gardes disponibles ; deux unités pour deux jetons |

Tour suivant, défi des Rôdeurs (attaque 4) : pas de parade. Phase d'assignation des dégâts : le joueur assigne aux Paysans
(épuisés mais non blessés) : une Blessure, 4 − 3 = 1 dégât restant ; puis au héros : une Blessure en main. Les Gardes d'Utem,
disponibles, s'activent ensuite en phase d'attaque : Attaque 2, plus Rage (Attaque 2), vainquent les Rôdeurs (armure 3).

```
 Phase d'assignation des dégâts (un ennemi non paré, deux unités)

   Rôdeurs : 4 dégâts ── Paysans (armure 3) ──► Blessure, reste 1 ── Héros (armure 2) ──► 1 Blessure en main
                    └─ ou Gardes (armure 5) ──► Blessure, reste 0 : fini
                    └─ ou Héros d'emblée   ──► 2 Blessures en main
```

## Modèle

Termes du glossaire : Unité, Unité régulière, Unité d'élite, Offre d'unités, Lieux de recrutement, Jeton de commandement,
Blessure, Soin X, Jeter (une Blessure), Le Commandant, Interaction, Influence. **Nouveaux termes** (ajoutés au glossaire avec
cette spec, Q3) :

| Livret | Français | Code |
|---|---|---|
| Unit area | Zone des unités | `PlayerState::units` |
| Ready / Spent (Unit) | Disponible (FAQ p2) / Épuisée | `OwnedUnit::ready` |
| Wounded (Unit) | Blessée | `OwnedUnit::wounds` |
| Recruit (a Unit) | Recruter (FAQ p1, p2) | `Action::RecruitUnit` |
| Disband (a Unit) | Renvoyer (FAQ p1, p3) | `UnitDisbanded` |
| Destroyed (Unit) | Détruite | `UnitDestroyed` |
| Activate (a Unit), ability | Activer, capacité | `Action::ActivateUnit`, `UnitAbility` |
| Ready a Unit (effect) | Rendre disponible une unité | `Effect::ReadyUnit` |
| Assign damage | Assigner les dégâts (FAQ p3) | `Action::AssignDamage` |
| Healing points | Points de soin | `PlayerState::healing` |

Données nouvelles :

- **État du joueur** : sa **zone des unités**, dans l'ordre du recrutement : pour chaque unité, sa carte (`UnitCard`), disponible
  ou non, ses Blessures (0 à 2), et si elle a reçu des dégâts dans le combat en cours. Ses **points de soin**.
- **Combat** : l'ennemi dont les dégâts sont en cours d'assignation et les dégâts qui lui restent ; les ennemis déjà traités.
- **Contenu** : `units.ron` gagne les capacités de chaque unité (effet, mana à payer, `Unsupported`), écrites avec les formes
  d'`Effect` existantes et quelques nouvelles : préparer une unité (niveau maximal), Blessure sur l'unité elle-même (Épéistes),
  parade double contre Vif, jetons de couleurs au choix et Feu glacé majoré (Mages d'Altem), jeton et cristal ensemble (Mages de
  feu et de glace). Validé par un test : chaque unité a les capacités de `cartes/unites.md`, dans l'ordre.
- **Effets de carte** : Rajeunissement gagne son option « rendre disponible une unité » ; Soin X alimente la réserve au lieu de
  jeter aussitôt (Q1).
- **Vue du joueur** : zone des unités, points de soin, dégâts en cours d'assignation ; tout est public.

## Actions et événements

| Action | Validation (refusée si…) | Événements produits |
|---|---|---|
| `RecruitUnit { unit, disband }` (nouvelle) | pas d'interaction en cours ; unité absente de l'offre ; site de l'interaction absent de ses lieux de recrutement ; influence < coût ; `disband` absent alors que les jetons sont tous pris, ou présent alors qu'il en reste, ou qui n'est pas une unité du joueur | `UnitDisbanded` (si `disband`), `UnitRecruited` |
| `ActivateUnit { unit, ability, option, mana, target }` (nouvelle) | unité absente, épuisée ou blessée ; capacité `Unsupported` ; effet pas jouable maintenant (règle « Quand ») ; mana absent, de la mauvaise couleur ou indisponible ; `target` (préparer une unité) qui n'est pas une unité épuisée du niveau permis | `UnitActivated`, le paiement du mana (événements des specs 022 et 025), puis les événements de l'effet (points, jeton, cristal, `UnitReadied`, `UnitWounded` pour les Épéistes) |
| `AssignDamage { enemy, to }` (nouvelle) ; `to` : une unité ou le héros | pas en phase d'assignation des dégâts ; ennemi déjà traité, paré ou vaincu, ou autre que l'ennemi commencé ; unité qui ne peut pas recevoir de dégâts | `DamageAssignedToUnit` (puis `UnitDestroyed` si paralysant) ; vers le héros : `WoundsTaken`, `PoisonWoundsDiscarded`, `HandParalyzed`, `KnockedOut` comme aujourd'hui ; puis, quand plus aucune unité ne peut recevoir de dégâts, les ennemis restants résolus seuls (031) |
| `SpendHealing { on }` (nouvelle) ; `on` : une Blessure de la main ou une unité | en combat ; points de soin insuffisants (1, ou le niveau de l'unité) ; pas de Blessure en main ; unité non blessée | `WoundsHealed` (034, 1 point) ou `UnitHealed` |
| `PlayCard` (013, étendue) | Rajeunissement « rendre disponible » : comme `ActivateUnit` ; Soin X en combat (inchangé) | Soin X : `CardPlayed` ajoute les points à la réserve (plus de `WoundsHealed` aussitôt, Q1) |
| `StartCombat` (031, étendue) | inchangée | les points de soin sont perdus |
| `AnnounceEndOfRound` (023, nouvelle manche) | inchangée | `UnitsReadied` |

Ordre des actions légales (recette de `architecture.md`) : `ActivateUnit` après les cartes de la main, dans l'ordre de la zone des
unités ; `SpendHealing` après eux ; en interaction, `RecruitUnit` dans l'ordre de l'offre ; en phase d'assignation, `AssignDamage`
dans l'ordre du combat, les unités dans l'ordre de la zone puis le héros.

| Événement | Effet de `apply` | Révèle une information ? | Point de non-retour ? |
|---|---|---|---|
| `UnitRecruited { player, unit, cost }` | l'unité quitte l'offre (sans la compléter) et entre dans la zone, disponible, non blessée ; influence − `cost` | non (publique) | non |
| `UnitDisbanded { player, unit }` | l'unité quitte la partie | non | non |
| `UnitActivated { player, unit, ability }` | l'unité est épuisée ; l'effet suit, comme pour une carte | non | non |
| `UnitReadied { player, unit }` | l'unité redevient disponible | non | non |
| `UnitWounded { player, unit, wounds }` | Blessures de l'unité + `wounds` | non | non |
| `DamageAssignedToUnit { player, enemy, unit, wounds }` | l'unité a reçu des dégâts dans ce combat ; Blessures + `wounds` (0, 1 ou 2) ; dégâts restants de l'ennemi baissés de l'armure (deux fois si résistante et blessée) | non | non |
| `UnitDestroyed { player, unit }` | l'unité quitte la partie, Blessures comprises | non | non |
| `UnitHealed { player, unit, cost }` | une Blessure de moins ; points de soin − `cost` | non | non |
| `UnitsReadied { player }` | toutes les unités redeviennent disponibles ; Blessures inchangées | non | non |
| `WoundsHealed` (034) | coûte désormais 1 point de soin par Blessure | inchangé | inchangé |
| `TurnEnded` (016) | vide aussi les points de soin | inchangé | inchangé |

Nouvel événement de combat dans l'historique (`CombatRecord`) : dégâts assignés à une unité ; les attaques et parades nomment les
unités activées avec les cartes.

## Comportements attendus

Recruter :

- **C1** — Étant donné une interaction au village, influence 5, un jeton de commandement libre, des Gardes d'Utem (coût 5, village)
  dans l'offre, quand le joueur les recrute, alors `UnitRecruited` ; influence 0 ; les Gardes sont disponibles, non blessés ;
  l'offre a deux unités (`recruiting_takes_a_unit_from_the_offer`) — MKUE p. 8, 4.a ; p. 5, 4.
- **C2** — Au village, des Illusionnistes (monastère, tour de mage) ne sont pas recrutables ; au monastère, oui ; des Paysans,
  l'inverse (`recruiting_needs_a_matching_site`) — MKUE p. 8, 4.a.
- **C3** — Influence 4, Gardes d'Utem à 5 : pas recrutables ; influence négative (malus de réputation) : rien n'est recrutable
  (`recruiting_needs_enough_influence`) — MKUE p. 8, 4.a ; spec 040.
- **C4** — Un jeton de commandement, déjà une unité : recruter exige de renvoyer une unité ; `UnitDisbanded` puis `UnitRecruited`,
  l'unité renvoyée quitte la partie (ni offre ni pioche) (`recruiting_beyond_command_disbands_a_unit`) — MKUE p. 5, 4.a.
- **C5** — Hors interaction, `RecruitUnit` n'est pas légale (`recruiting_needs_an_interaction`) — MKUE p. 8, 4.
- **C6** — Une unité recrutée s'active dans la même interaction : Paysans recrutés, puis Influence 2, puis une seconde unité
  recrutée (`a_new_unit_can_be_used_at_once`) — MKUE p. 8, 4.a ; Compendium, *Silver Units* Q3.
- **C7** — Gloire franchissant une ligne de niveau pendant le tour : le nouveau jeton de commandement ne sert pas avant la fin du
  tour (`new_command_token_after_end_of_turn`) — FAQ p3.

Activer :

- **C8** — Paysans disponibles en phase de déplacement : `ActivateUnit` (Mouvement 2) donne 2 points de mouvement ; les Paysans sont
  épuisés et ne s'activent plus ce tour ni au suivant (`activating_spends_the_unit_until_the_round_ends`) — MKUE p. 5, 5.a et 9.
- **C9** — Une unité blessée ne s'active pas (`a_wounded_unit_cannot_be_activated`) — MKUE p. 11, 2.a.
- **C10** — Moines à cape rouge, Attaque de feu 4 : légale avec un dé rouge (ou doré de jour), un cristal ou un jeton rouges ;
  refusée sans mana rouge ; le dé compte comme le dé du tour (`mana_abilities_need_mana`) — MKUE p. 5, 5.b.
- **C11** — En combat, une Attaque d'unité ne s'active que dans une attaque en cours, une Parade dans une parade en cours ; les
  Arbalétriers (Attaque à distance 2) s'activent dès la première phase, pas contre un ennemi fortifié ; les Catapultes (siège), si
  (`unit_abilities_follow_combat_phases`) — MKUE p. 9, 3.a et 4.a.
- **C12** — Golems gardiens, Parade de glace 4 contre une attaque de feu : efficace ; Paysans, Parade 2 : inefficace, divisée par
  deux avec les autres parades inefficaces (`unit_blocks_have_elements`) — MKUE p. 9, *Block Phase* 5.
- **C13** — Gardes d'Utem, Parade 4 contre des Arbalétriers (ennemi gris, attaque 4, Vif : 8 pour la parade) : la parade compte 8,
  réussie seule ; Paysans, Parade 2 : compte 2
  (`guardsmen_block_counts_twice_against_swift`) — `cartes/unites.md`, *Lost Legion*.
- **C14** — Épéistes d'Utem, Parade 6 : la parade compte 6 et l'unité reçoit une Blessure (`UnitWounded`) ; contre un ennemi
  paralysant, elle n'est pas détruite (`swordsmen_strong_ability_wounds_them`) — Compendium, *UTEM SWORDSMEN* S2.
- **C15** — Mages de feu, troisième capacité : un jeton rouge et un cristal rouge ; Mages d'Altem, première capacité : deux jetons
  au choix, noir compris de jour (`unit_mana_abilities`) — `cartes/unites.md` ; Compendium, *ALTEM MAGES* Q3.
- **C16** — Une capacité `Unsupported` (Forestiers, Mouvement) n'est pas légale (`unsupported_abilities_are_not_offered`).
- **C17** — Concentration renforcée ne peut pas viser une capacité d'unité (`concentration_does_not_boost_units`) — FAQ p2.
- **C18** — Herboristes, « rendre disponible » : les Paysans épuisés redeviennent disponibles et s'activent à nouveau ; une unité de
  niveau III ne peut pas être visée ; hors combat seulement (`herbalists_ready_a_unit`) — MKUE p. 5, 6 ; Compendium, *Silver
  Units* S10, *HERBALIST* S1.
- **C19** — Rajeunissement (Norowas) : de base, rend disponible une unité de niveau I ou II ; renforcé, jusqu'au niveau III
  (`rejuvenate_readies_a_unit`) — `cartes/actions-de-base.md`.
- **C20** — Nouvelle manche : toutes les unités redeviennent disponibles (`UnitsReadied`), une unité blessée garde ses Blessures
  (`units_are_readied_at_the_new_round`) — MKUE p. 4, 2.g.

Assigner les dégâts :

- **C21** — Rôdeurs (4) non parés, Paysans (armure 3) : `AssignDamage` aux Paysans, une Blessure sur l'unité, reste 1 ; puis au
  héros : une Blessure en main (`damage_goes_to_units_then_hero`) — MKUE p. 9, 4.a et 5 ; exemple ci-dessus.
- **C22** — Même attaque assignée aux Gardes d'Utem (armure 5) : une Blessure, plus rien pour le héros ; l'ennemi est traité
  (`a_unit_can_absorb_all_damage`) — MKUE p. 9, 4.a.
- **C23** — Une unité épuisée non blessée reçoit des dégâts ; une unité blessée, non (`spent_units_can_take_damage`) — MKUE p. 9, 4.
- **C24** — Golems gardiens (armure 3, résistance physique) contre 3 dégâts physiques : aucune Blessure, plus de dégâts ; contre 4 :
  une Blessure, reste 0 ; contre 8 (brutal) : une Blessure, reste 2 (`resistant_units_absorb_twice_their_armor`) — MKUE p. 9, 4.b ;
  Compendium, *Combat* S34.
- **C25** — Une unité qui a reçu des dégâts sans être blessée (résistance) ne peut pas recevoir ceux d'un second ennemi, mais reste
  activable en phase d'attaque (`a_unit_takes_damage_once_per_combat`) — FAQ p3 ; Compendium, *Combat* S35.
- **C26** — Feu glacé contre des Golems de feu (feu et physique) : pas de résistance, Blessure d'emblée ; contre une unité feu et
  glace, oui (`cold_fire_needs_both_resistances`) — MKUE p. 9, 4.b.
- **C27** — Ennemi toxique : l'unité reçoit deux Blessures ; ennemi paralysant : `UnitDestroyed`, son jeton de commandement se
  libère, les dégâts restants baissent quand même de son armure (`poison_and_paralyze_on_units`) — MKUE p. 10, 6.a et 6.b ;
  Compendium, *Combat* S47.
- **C28** — Deux ennemis non parés : le joueur choisit lequel traiter d'abord ; une fois le héros choisi pour un ennemi, plus
  d'unité pour lui (`damage_order_is_chosen`) — MKUE p. 9, 1.b ; Compendium, *Combat* S34.
- **C29** — Sans unité pouvant recevoir des dégâts (aucune, ou toutes blessées), la phase se résout seule comme avant, sans
  décision (`no_eligible_unit_resolves_damage_alone`) — spec 031, Q3.
- **C30** — Assommé après le premier ennemi : le joueur assigne encore les dégâts du second à une unité ; les unités s'activent
  encore en phase d'attaque (`units_fight_on_when_knocked_out`) — MKUE p. 9, 5.c ; Compendium, *Combat* S45.

Soins :

- **C31** — Tranquillité (Soin 1) jouée : 1 point de soin, aucune Blessure jetée encore ; `SpendHealing` sur la main jette une
  Blessure (`healing_points_are_a_reserve`) — MKUE p. 11, 3.c ; Q1.
- **C32** — Gardes d'Utem blessés (niveau II) : Herboristes (Soin 2, mana vert) puis `SpendHealing` sur les Gardes : `UnitHealed`,
  0 point ; avec 1 point seulement, refusé (`healing_a_unit_costs_its_level`) — MKUE p. 11, 3.c.
- **C33** — Une unité à deux Blessures (toxique) est soignée une Blessure à la fois (`a_unit_heals_one_wound_at_a_time`) — FAQ p3.
- **C34** — Points de soin non dépensés : perdus au début d'un combat et à la fin du tour (`healing_points_are_lost`) — MKUE p. 11,
  3.d.

Décompte, sauvegarde, vue :

- **C35** — Fin de partie de Première reconnaissance avec Paysans (I) non blessés et Gardes d'Utem (II) blessés : Le Commandant 1 +
  1 = 2 (`leader_counts_unit_levels`) — MKUE p. 15.
- **C36** — Revenir en arrière après `RecruitUnit`, `ActivateUnit` ou `AssignDamage` rétablit l'état d'avant (`units_can_be_undone`)
  — spec 024.
- **C37** — Une partie sauvegardée avec deux unités (une épuisée, une blessée) et 2 points de soin se recharge à l'identique
  (`units_survive_save_and_load`) — spec 024.
- **C38** — La vue du joueur donne la zone des unités, les points de soin et les dégâts en cours d'assignation (`view_shows_units`).
- **C45** — Issue privée 341 : la vue donne aussi les unités de l'offre recrutables dans le lieu de l'interaction (MKUE p. 8, 4.a ; dans
  la cité blanche toutes, 4.e), dans l'ordre de l'offre, quelle que soit l'Influence ; aucune hors interaction
  (`view_lists_units_recruitable_here_whatever_the_influence`).

Écran (arcanum-ui, sur `game_screen`) :

- **C39** — Le panneau liste les unités : nom, niveau, armure, résistances, « Disponible », « Épuisée », « Blessée » (×2), et le
  nombre d'unités sur les jetons de commandement (« Unités : 1/2 ») (`panel_lists_units`).
- **C40** — Chaque capacité activable a son bouton ; une capacité `Unsupported` est grisée avec « pas encore disponible » ; une
  capacité payante ouvre le choix du mana (spec 022) (`unit_ability_buttons`).
- **C41** — En interaction, la ligne « Rien à acheter ici » laisse place à « Recruter » : les unités de l'offre recrutables ici,
  avec leur coût ; jetons tous pris, le recrutement ouvre le choix de l'unité à renvoyer (`panel_offers_recruiting`).
- **C46** — Issue privée 341 : une unité recrutable ici mais trop chère garde sa ligne, grisée, avec ses capacités en infobulle, comme
  les achats (spec 041) ; jetons tous pris et Influence suffisante, la ligne reste active et mène au renvoi ; « Recruter sous
  Loyauté » (spec 058) suit la même règle avec le coût réduit ; le recrutement gratuit d'Appel à la gloire (spec 057) est
  inchangé (`panel_greys_unaffordable_recruits`, `bonds_recruit_greyed_without_influence`).
- **C42** — En phase d'assignation : l'ennemi en cours et ses dégâts restants, un bouton par unité possible (« Sur les Paysans,
  armure 3 ») et « Sur le héros » ; un ennemi à choisir quand aucun n'est commencé (`combat_screen_assigns_damage`).
- **C43** — Points de soin : « Soin : 2 » sur la ligne des points ; boutons « Jeter une Blessure (1) » et « Soigner les Gardes
  d'Utem (2) » (`panel_offers_healing`).
- **C44** — L'écran de fin montre la ligne Le Commandant calculée (`score_shows_leader`).

## Cas limites et refus

- **L1** — Recruter la même unité deux fois, ou une unité déjà recrutée : refusé (elle n'est plus dans l'offre).
- **L2** — Renvoyer une unité blessée ou épuisée pour en recruter une : permis ; ses Blessures quittent la partie avec elle.
- **L3** — Activer une unité pour de l'influence avant d'interagir : permis, l'influence est perdue au début de l'interaction
  (spec 040), comme pour une carte.
- **L4** — Soin X sans Blessure en main ni unité blessée : permis, les points restent jusqu'à la fin du tour, puis sont perdus
  (spec 034, *Soin en trop*).
- **L5** — Une unité détruite ou renvoyée ne compte pas au décompte ; une unité disponible ou épuisée compte pareil.
- **L6** — Dégâts assignés à une unité, puis plus aucune unité possible : le reste de cet ennemi va au héros, sans décision.
- **L7** — Invocation (spec 033) : la créature invoquée inflige ses dégâts, assignables aux unités comme ceux d'un autre ennemi.
- **L8** — Repos : activer des unités est permis pendant un tour de repos (Compendium, *Resting* S3) ; un mouvement d'unité n'empêche
  pas le repos tant que la figurine ne bouge pas (spec 034).

## Information cachée

Rien de caché : la zone des unités, leur état, l'offre et les points de soin sont publics. Les événements nouveaux ne sont pas
filtrés.

## Libellés (i18n)

| Clé | Paramètres | fr | en |
|---|---|---|---|
| `units.header` | `count`, `command` | Unités : {count}/{command} | Units: {count}/{command} |
| `units.line` | `name`, `level`, `armor` | {name} · niveau {level} · armure {armor} | {name} · level {level} · armor {armor} |
| `units.ready` / `units.spent` | — | Disponible / Épuisée | Ready / Spent |
| `units.wounded` | `wounds` | Blessée ({wounds}) | Wounded ({wounds}) |
| `units.unsupported` | — | pas encore disponible | not available yet |
| `units.recruit` | `name`, `cost` | Recruter : {name} ({cost}) | Recruit: {name} ({cost}) |
| `units.disband_prompt` | — | Choisir l'unité à renvoyer | Choose a Unit to disband |
| `damage.remaining` | `enemy`, `damage` | {enemy} : {damage} dégâts à assigner | {enemy}: {damage} damage to assign |
| `damage.to_unit` | `name`, `armor` | Sur {name} (armure {armor}) | To {name} (armor {armor}) |
| `damage.to_hero` | — | Sur le héros | To the hero |
| `healing.points` | `points` | Soin : {points} | Healing: {points} |
| `healing.wound` | — | Jeter une Blessure (1) | Throw away a Wound (1) |
| `healing.unit` | `name`, `cost` | Soigner {name} ({cost}) | Heal {name} ({cost}) |
| `combat.history.unit_damage` | `name`, `wounds` | {name} : {wounds} Blessure(s) | {name}: {wounds} Wound(s) |
| `combat.history.unit_destroyed` | `name` | {name} détruite | {name} destroyed |
| `effect.ready_unit` | `level` | Rendre disponible une unité de niveau {level} au plus | Ready a Unit of level {level} or less |

Les capacités gardent leur texte `unit.<id>.abilities` (spec 050), découpé en une clé par capacité pour les boutons ; « Rendre
prête » devient « Rendre disponible » (Q3).

## Interface

- **Panneau latéral** : une section « Unités : 1/2 » sous les points, une ligne par unité (`units.line`, résistances, état), et
  dessous un bouton par capacité. Visuels provisoires : du texte, pas de carte dessinée.
- **Interaction** : la liste « Recruter » remplace « Rien à acheter ici » (spec 040) au village et au monastère ; un recrutement
  au-delà des jetons ouvre un mode choix (comme le choix de la carte à défausser, spec 013) où chaque unité a « Renvoyer ». Une
  ligne par unité recrutable dans le lieu, grisée si l'Influence manque (issue privée 341, C46).
- **Écran de combat** : les capacités d'unités sous la main, avec les mêmes repères d'efficacité que les cartes (spec 032) ; en
  phase d'assignation, le bloc de dégâts (C42) remplace « Phase suivante ».
- **Points de soin** : sur la ligne des points, avec leurs boutons (C43), hors combat.
- **Fenêtre Offres** (spec 050) : inchangée, l'unité recrutée en disparaît.

```
┌─ Panneau latéral ─────────────────────────────────┐
│ Interaction : Village                             │
│ Mouvement : 0 · Influence : 6 · Soin : 0          │
│ Recruter : Gardes d'Utem (5)                      │
│ Unités : 1/2                                      │
│ Paysans · niveau I · armure 3 · Épuisée           │
│   [Attaque ou Parade 2] [Influence 2] [Mvt 2]     │
└───────────────────────────────────────────────────┘
```

Vérification à l'œil : une partie préparée `unites` (bac à sable, Jour, niveau 3, figurine sur le village (2, −3), voisine des
Rôdeurs (3, −3) ; Paysans disponibles et Golems gardiens blessés dans la zone ; des Gardes d'Utem dans l'offre ; main : Promesse,
Menace, Tranquillité, Rage, Détermination ; un cristal vert) : soigner les Golems, interagir, recruter en renvoyant une unité ;
puis, depuis le chargement, défier les Rôdeurs et assigner leurs dégâts.

### Raccourcis clavier

Aucun. Les capacités sont nombreuses et changent d'une unité à l'autre ; une touche par capacité ou par unité serait arbitraire.
La validation de l'attaque ou de la parade (Entrée, spec 031) vaut aussi avec des unités.

## Effets sur les specs déjà implémentées

- **031** (Q3) : la phase d'assignation ne se résout seule que sans unité possible ; sinon elle attend `AssignDamage`.
- **034** (Q2) : Soin X alimente une réserve au lieu de jeter aussitôt (Q1). Les tests de la 034 qui attendent `WoundsHealed` juste
  après la carte sont à revoir (un `SpendHealing` en plus) et signalés dans la PR. Rajeunissement gagne son option « rendre
  disponible une unité ».
- **035** : les jetons de commandement limitent les unités.
- **040** : « Rien à acheter ici » est remplacé par le recrutement (la ligne reste au village et au monastère s'il n'y a rien à
  recruter, jusqu'à la 041).
- **050** : une unité recrutée quitte l'offre sans la compléter (déjà dit par la 050).
- **063** : Le Commandant est calculé.
- **023** : la nouvelle manche produit `UnitsReadied`.
- **024** : `Event::blocks_undo` gagne les nouveaux événements (non). Les parties préparées sont régénérées si leur construction
  change.
- **Parcours 019, 028, 039** : sans unité, inchangés. Le joueur de test `FirstChoice` n'interagit pas, il ne recrute donc pas.

## Hors périmètre

- Capacités `Unsupported` du tableau ci-dessus : spec 052 (moteur d'effets) ou la spec qui les demandera.
- Recruter dans une forteresse, une tour de mage, une cité (Fusiliers, Catapultes, Mages et Gardiens d'Altem) : specs 043 et 062.
  Une unité en récompense de combat (ruines, MKUE p. 11, 5.d, avec le report après un changement de niveau) : spec 045.
- Acheter des points de soin au village et au monastère : spec 041.
- Unités qui ne suivent pas le héros (monastère brûlé, donjon, tombeau, MKUE cartes de site) : specs 041 et 044.
- Bannières : spec 055. Sorts et actions avancées qui visent les unités (Appel aux armes, Au cœur du combat, Récit héroïque…) :
  specs 053 et 054. Compétences : 051.
- Effets rouges ou bleus d'unités contre les résistances des ennemis (MKUE p. 8, 5.a et 5.b) : seules les capacités `Unsupported`
  des Geleurs et des Illusionnistes sont concernées (Compendium, *Silver Units* S6).
- Unités des extensions (Éclaireurs, Voyous, Familiers…), combat entre joueurs (p. 24, second paragraphe) : hors périmètre.
- Dessin des cartes d'unité : plus tard, si le besoin se fait sentir.

## Décisions (2026-11-03)

Questions de la rédaction, tranchées par Mathieu (« ok pour tes 3 remarques ») :

1. **Q1** — Réserve de points de soin : les Soin X s'additionnent et se dépensent à la demande (`SpendHealing`), sur une Blessure
   de la main (1 point) ou sur une unité (son niveau). Le jet immédiat de la spec 034 (son Q2) est remplacé ; les tests de la 034
   concernés sont revus et signalés dans la PR d'implémentation. La spec 041 remplira la même réserve.
2. **Q2** — Cinq capacités restent `Unsupported` jusqu'à la spec 052 : mouvement des Forestiers, capacité blanche des
   Illusionnistes, bleue des Geleurs d'Amotep, noire des Mages d'Altem, verte des Gardiens d'Altem. Toutes les autres sont codées
   par cette spec, y compris celles d'unités pas encore recrutables.
3. **Q3** — Termes de la FAQ française : Disponible / Épuisée / Blessée, Recruter, Renvoyer, Détruite, Activer, Assigner les dégâts,
   Points de soin, et « Rendre disponible une unité » (glossaire et `fr.json`).

## Questions ouvertes

Aucune.

## Notes d'implémentation (2026-11-04)

Écarts à signaler, retenus pendant l'implémentation (PR de la spec) :

- **Rajeunissement** : son option « rendre disponible une unité » se joue par une action à part, `PlayCardOnUnit { card, play,
  target }`, plutôt qu'en étendant `PlayCard` : une cible dans `CardPlay` aurait touché toutes ses constructions.
- **Jetons de mana** : ils portent une `ManaColor` (or et noir compris) pour les jetons des Mages d'Altem ; `ManaTokenGained` et
  `ManaTokenUsed` changent de forme.
- **Libellés de plus** : `units.disband` (« Renvoyer ») et `units.choice_prompt` (« {name} : {ability} », bandeau du choix d'une
  capacité qui a plusieurs options, paiements ou cibles). Le texte des capacités passe en une clé par capacité,
  `unit.<id>.ability_N`.
- **Écran** : « Soin : N » ne s'ajoute à la ligne des points que s'il reste des points de soin ; en combat, les unités sont sous la
  main de l'écran de combat, et non dans le panneau latéral.
