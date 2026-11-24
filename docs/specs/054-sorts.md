# 054 — Sorts : lancer un sort, sorts de combat

- Statut : implémentée (2026-11-12) ; validée le 2026-11-12 (questions Q1 à Q6 tranchées : réponses par défaut acceptées ; Don
  pour la magie, même couleur, tranché à part)
- Jalon : J5
- Dépend de : 053 (modificateurs d'ennemis, règle des résistances aux effets, `WoundFromCard`, `extra_mana`), 056 (Don pour la
  magie renforcé, `gain`), 052 (moteur d'effets, ADR 0008, règle « effet utilisable »), 050 (offres, sorts en données), 043 et 044
  (sorts gagnés, sites fortifiés, règles de nuit au donjon), 031 à 033 (combat, éléments, capacités des ennemis), 022 et 025
  (mana payé, jetons) ; toutes implémentées
- Livret : MKUE p. 4 (*Deed Cards* 3.b, 3.e, 6) ; p. 5 (*Using Mana* 2 ; *"Composite" Effects* 1 ; *"Gain" Effects* 3.a) ;
  p. 8 (*Combat with Enemies* 5) ; p. 9 (*Ranged and Siege Attack Phase* 3 ; *Block Phase*) ; p. 24 (*Defensive*, résistances).
  Texte des cartes : `docs/regles/cartes/sorts.md`, `cartes/actions-avancees.md` (Don pour la magie, version *Lost Legion*,
  comme la carte française de `scans/fr/advanced_actions/`). FAQ p1 (sort et résistances ; Démolition ; mana noir au donjon), FAQ p2 (Don pour la magie ;
  Tremblement de terre et Démolition ; Bouclier explosif). Compendium, *Spell Cards* (S1, S2 ; Burning Shield Q1, S2, S3 ; Chill
  S1 à S3 ; Demolish S1 à S5 ; Disintegrate S1, S3 ; Earthquake S1 à S5 ; Exploding Shield S1 à S3 ; Expose S1 ; Fireball S1 ;
  Firestorm S1 ; Flame Wave S2 à Q4 ; Mana Bolt S1 ; Mass Expose S1 ; Tremor S1, S2 ; Whirlwind S1), *Advanced Action Cards*
  (Magic Talent S1, S3, S5), *Basic Action Cards* S6 (Concentration), O5
- ADR : [0008 Moteur d'effets](../decisions/0008-moteur-d-effets.md) (sortes de modificateurs d'ennemis nouvelles)
- Crates touchées : arcanum-core (règles, contenu), arcanum-ui, arcanum-persistance (parties préparées), `assets/i18n/`

## Objectif

Un sort se **lance** : son effet de base se paie d'un mana de sa couleur, son effet renforcé, de nuit, d'un mana de sa couleur et
d'un mana noir. Les **dix sorts de combat** du jeu de base font ce qu'ils disent : attaques à distance et de siège élémentaires,
ennemis qui perdent fortifications et résistances, armure réduite, ennemis qui n'attaquent pas ou **détruits**, boucliers qui
contre-attaquent. Don pour la magie de base lance un sort **de l'offre**.

C'est le premier des deux lots de sorts (Q1), comme les actions avancées (053 puis 056) : les dix autres (déplacement, unités,
soins, mana et cartes) forment la spec 057.

| Sort (fr.json) | Couleur | Base | Renforcé | Ce qui est nouveau |
|---|---|---|---|---|
| Éclair de feu / Tempête de feu (*Fireball*) | rouge | Attaque à distance de feu 5 | Blessure ; Attaque de siège de feu 8 | sort payé ; Blessure (`WoundFromCard`) |
| Tempête de neige / Blizzard (*Snowstorm*) | bleu | Attaque à distance de glace 5 | Blessure ; Attaque de siège de glace 8 | idem |
| Exposition (*Expose*) | blanc | un ennemi perd fortifications et résistances ; Attaque à distance 2 | tous les ennemis perdent leurs fortifications, ou tous leurs résistances ; Attaque à distance 3 | `LosesFortifications`, `LosesResistances` ; effet sur tous les ennemis |
| Secousse / Tremblement de terre (*Tremor*) | vert | un ennemi Armure −3, ou tous −2 | idem, −6 / −4 contre un fortifié | armure réduite selon la fortification |
| Mur de flammes / Déluge de flammes (*Flame Wall*) | rouge | Attaque de feu 5 ou Parade de feu 7 | idem, +2 par jeton ennemi affronté | points comptés sur le combat |
| Éclair de mana / Tonnerre de mana (*Mana Bolt*) | bleu | payer un mana : Attaque de glace 8, de feu glacé 7, à distance de glace 6, de siège de glace 5 | idem +3 | trois mana pour le renforcé |
| Tourbillon / Tornade (*Whirlwind*) | blanc | un ennemi n'attaque pas | phase d'attaque : détruire un ennemi | détruire un ennemi |
| Bouclier ardent / explosif (*Burning Shield*) | rouge | Parade de feu 4 ; si la parade réussit, Attaque de feu 4 en phase d'attaque | Parade de feu 4 ; si elle réussit, l'ennemi paré est détruit | effet après la parade |
| Frisson / Frisson fatal (*Chill*) | bleu | un ennemi n'attaque pas, perd sa résistance au Feu | un ennemi n'attaque pas, Armure −4 | perdre une résistance |
| Démolition / Désintégration (*Demolish*) | rouge | ignorer les fortifications des sites ; ennemis Armure −1 | phase d'attaque : détruire un ennemi ; les autres Armure −1 | `IgnoreSiteFortifications` |
| Don pour la magie (*Magic Talent*), base | bleu | défausser une carte : lancer un sort de même couleur de l'offre | (spec 056) | carte jouée depuis une offre |

## Règles

### Lancer un sort

- **Effet de base** (MKUE p. 4, 3.b ; Compendium, *Spell Cards* S2) : il se paie **toujours** d'un mana de la couleur du sort (dé,
  cristal, jeton ; or de jour, MKUE p. 5, 2.a), de jour comme de nuit. Sans ce mana, le sort ne se joue que de côté (p. 4, 3.e,
  déjà le cas, spec 050).
- **Effet renforcé** (MKUE p. 4, 3.b ; p. 5, 2.b) : un mana de la couleur du sort **et** un mana noir. Le noir vient d'un dé noir de
  la Source ou d'un jeton noir (`black_payments`, spec 053) : de nuit, ou dans le combat d'un donjon ou d'un tombeau, qui suit les
  règles de nuit (FAQ p1 ; `PlayerState::combat_time`, spec 044). L'or ne sert jamais de nuit. Les deux mana peuvent venir de deux
  dés de la Source seulement si le joueur a des dés de plus (Source de mana, spec 025) ; sinon un seul dé par tour, comme
  aujourd'hui.
- **Couleur du sort** (Compendium, *Spell Cards* S1 ; FAQ p1) : celle de son effet de base, pour ses deux effets ; c'est elle que
  porte `ModifierSource` pour les résistances (spec 053).
- **Concentration et Effet maximum** (Compendium, *Basic Action Cards* S6 ; spec 056) : un sort n'est jamais leur cible (actions
  seulement), comme aujourd'hui pour Effet maximum ; Concentration le vérifie désormais (un sort devient jouable renforcé).
- **Effet utilisable** (spec 052) : les dix sorts de ce lot ne se jouent qu'**en combat**, chacun dans les phases où son effet sert
  (sections ci-dessous) ; hors combat, de côté seulement.

### Règles communes aux sorts de combat

- **Ennemi visé** (cartes) : « un ennemi » (*target enemy*) est choisi **en jouant le sort**, parmi les ennemis pas vaincus du
  combat ; les points d'Attaque ou de Parade du même sort vont à l'attaque ou à la parade en cours, quelle que soit la cible ;
  sans attaque en cours, les points d'Exposition sont perdus (Q3).
  « Les ennemis », « tous les ennemis » : tous ceux du combat qui ne sont pas vaincus (Compendium, *Tremor* S1), pas seulement les
  cibles de l'attaque en cours ; une créature invoquée compte comme un ennemi tant qu'elle est là (spec 033).
- **Résistances aux effets** (MKUE p. 8, 5 ; règle générale de la spec 053) : un modificateur d'ennemi d'un sort rouge n'est pas
  posé sur un ennemi résistant au Feu, d'un sort bleu sur un ennemi résistant à la Glace ; **détruire** est un tel effet
  (Compendium, *Disintegrate* S1, *Exploding Shield* S1). Les points du sort comptent normalement (spec 032). Blanc et vert : aucune
  résistance ne les arrête. Ce qui vise le site (Démolition, fortifications) n'est pas arrêté (Compendium, *Demolish* Q2).
- **Détruire un ennemi** (cartes ; Compendium, *Exploding Shield* Q2) : l'ennemi est **vaincu** aussitôt, comme par une attaque
  réussie (gloire, réputation d'un ennemi errant, conquête du site ; specs 031, 040, 043) (Q2). Une créature invoquée détruite
  disparaît **sans gloire**, et son Invocateur reste (spec 033).
- **Perdre ses fortifications** (cartes ; Compendium, *Demolish* S1, S4, *Earthquake* S1) : modificateur d'ennemi
  `LosesFortifications` : ni Fortifié du jeton ni fortification du site ; un doublement fortifié n'est plus fortifié du tout. Une
  **fortification de site ignorée** (`IgnoreSiteFortifications`, Démolition) n'enlève que celle du site : des Guardsmen défenseurs
  restent fortifiés par leur jeton.
- **Perdre ses résistances** (cartes) : `LosesResistances` (toutes : physique, Feu, Glace) ou `LosesResistance(Fire)` (Frisson) ;
  le combat, les résistances aux effets de la 053 et les infobulles lisent les **résistances effectives**.
- **Durée** : les modificateurs d'ennemis durent le combat (ADR 0008, spec 053) ; « ce tour » sur un ennemi revient au même,
  l'ennemi ne survit pas au combat. `IgnoreSiteFortifications` dure le tour (Compendium, *Demolish* S5 : le prochain assaut
  retrouve les fortifications).
- **Armure réduite** (spec 053) : plusieurs réductions s'ajoutent, jamais sous 1.

### Éclair de feu et Tempête de neige

- **Base** (cartes ; Compendium, *Fireball* S1) : Attaque à distance de feu (de glace) 5 ; jouable là où une attaque à distance
  sert : en phase à distance contre une cible non fortifiée, ou en phase d'attaque (spec 031).
- **Renforcé** (cartes ; Compendium, *Firestorm* S1) : une Blessure **en main** (`WoundFromCard`, comme Coup de sang, spec 053 :
  elle compte pour être assommé), puis Attaque de siège de feu (de glace) 8. Une seule Blessure, quelle que soit la suite.

### Exposition

- **Base** (carte ; Compendium, *Expose* S1) : l'ennemi visé reçoit `LosesFortifications` et `LosesResistances` ; Attaque à
  distance 2. Jouable en phase à distance et en phase d'attaque ; l'ennemi visé peut être fortifié (c'est le but).
- **Renforcé** (carte ; Compendium, *Mass Expose* S1) : deux options, **tous les ennemis** perdent leurs fortifications, ou tous
  leurs résistances ; Attaque à distance 3.
- **Sans attaque en cours** (Q3) : une attaque se déclare sur des cibles attaquables (spec 031), donc pas sur un ennemi fortifié en
  phase à distance. Exposition jouée **avec** une attaque en cours y met ses points ; jouée **sans**, elle ne fait que poser ses
  modificateurs, et ses points sont perdus. Le joueur déclare ensuite l'attaque sur l'ennemi qui n'est plus fortifié.

### Secousse et Tremblement de terre

- **Base** (carte ; Compendium, *Tremor* S1, S2) : deux options : l'ennemi visé `ArmorReduction { by: 3, min: 1 }`, ou **tous** les
  ennemis `{ by: 2, min: 1 }`. Jouable dans toute phase du combat, même sans effet (S2).
- **Renforcé** (carte ; Compendium, *Earthquake* S1 à S5 ; FAQ p2) : la réduction est doublée (−6, −4) pour un ennemi **fortifié
  au moment où le sort est joué** (jeton ou site, doublement fortifié compris, pas après Exposition ou Démolition) ; dans l'option
  « tous », chaque ennemi selon sa fortification (S5). En phase de parade aussi (S4).

### Mur de flammes et Déluge de flammes

- **Base** (carte) : Attaque de feu 5 (attaque en cours) ou Parade de feu 7 (parade en cours).
- **Renforcé** (carte ; Compendium, *Flame Wave* S2, S3, Q4) : +2 par **jeton ennemi affronté**, compté en jouant la carte : les
  ennemis du combat qui ne sont pas vaincus avant la **phase en cours** (vaincus pendant celle-ci compris), Invocateur compté,
  créature invoquée non (Q4).

### Éclair de mana et Tonnerre de mana

- **Mana en plus** (carte ; Compendium, *Mana Bolt* S1) : comme Magie pure (spec 053), une option par couleur de base, payée **en
  plus** du mana bleu du sort (`extra_mana`) : bleu Attaque de glace 8, rouge Attaque de feu glacé 7, blanc Attaque à distance de
  glace 6, vert Attaque de siège de glace 5 ; renforcé : 11, 10, 9, 8, avec le mana noir en plus, soit trois mana. Le mana en plus
  peut être d'or de jour (base seulement : l'or ne sert pas de nuit).
- **Effet utilisable** : chaque option là où son genre d'attaque sert (spec 031) : glace et feu glacé en phase d'attaque, distance
  et siège aussi en phase à distance.

### Tourbillon et Tornade

- **Base** (carte ; Compendium, *Whirlwind* S1) : l'ennemi visé `DoesNotAttack` ; même moment et mêmes cibles que les
  Illusionnistes (spec 053, Q7) : phase à distance ou de parade, un ennemi qui attaque encore (pas paré, pas encore de dégâts) ;
  contre un Invocateur, en phase à distance seulement, avant le tirage de sa créature. Fortifié ou non. Un ennemi Vif reste Vif
  sans conséquence (il n'attaque pas).
- **Renforcé** (carte) : **en phase d'attaque seulement**, l'ennemi visé est détruit. Blanc : aucune résistance ne l'empêche.

### Bouclier ardent et Bouclier explosif

- **Base** (carte ; Compendium, *Burning Shield* Q1, S2, S3) : Parade de feu 4 dans la parade en cours. Si cette parade **réussit**,
  le joueur gagne une **Attaque de feu 4 différée** (`DeferredAttack`, durée le combat) qu'il ajoute, s'il le veut, à une attaque en
  cours de la **phase d'attaque**, contre n'importe quelle cible (S2), sans mana. Si la parade échoue, rien.
- **Renforcé** (carte ; Compendium, *Exploding Shield* S1 à S3 ; FAQ p2) : Parade de feu 4 ; si la parade réussit, l'ennemi paré est
  **détruit** (vaincu, gloire) ; s'il est résistant au Feu, la parade compte mais il n'est pas détruit. Contre la créature invoquée,
  c'est la créature qui est détruite, sans gloire (Q2 du compendium) ; l'Invocateur reste.
- Aucun ennemi du jeu de base n'a d'attaques multiples (`enemies.ron`) : la question du compendium (Q1, S3) ne se pose pas.

### Frisson et Frisson fatal

- **Base** (carte ; Compendium, *Chill* S1 à S3) : l'ennemi visé `DoesNotAttack` et `LosesResistance(Fire)` ; mêmes moments et
  cibles que Tourbillon (le compendium permet aussi après les dégâts, Q2 : non retenu, l'effet principal n'aurait plus d'objet, Q6).
  Bleu : rien sur un ennemi résistant à la Glace (S1), ce qui l'exclut des cibles. Contre un Invocateur, aucune créature tirée (S3).
- **Renforcé** (carte) : `DoesNotAttack` et `ArmorReduction { by: 4, min: 1 }`.

### Démolition et Désintégration

- **Base** (carte ; Compendium, *Demolish* S1 à S5 ; FAQ p1, p2) : `IgnoreSiteFortifications` pour le tour (modificateur du
  joueur : les défenseurs ne sont plus fortifiés par le site, toujours par leur jeton) ; tous les ennemis `ArmorReduction { by: 1,
  min: 1 }`, sauf les résistants au Feu. En combat, dans toute phase (Q5).
- **Renforcé** (carte ; Compendium, *Disintegrate* S1, S3) : en phase d'attaque seulement ; l'ennemi visé, **pas résistant au Feu**,
  est détruit, puis les **autres** ennemis non résistants au Feu `ArmorReduction { by: 1, min: 1 }`. Sans ennemi à détruire, pas
  jouable.

### Don pour la magie (base)

- **Défausse** (carte, version *Lost Legion* ; Compendium, *Magic Talent* S3, O4) : défausser une carte **de couleur** de la main (action de base
  ou avancée, sort ; jamais une Blessure ni un artefact), autre que Don pour la magie (`Cost::DiscardAnotherCard`, Improvisation,
  spec 025). Sans carte de couleur à défausser, pas jouable. Le sort lancé est **de la même couleur** que la carte défaussée :
  c'est l'errata *Lost Legion*, qui fait foi (le scan anglais du jeu de base ne le dit pas ; tranché par Mathieu, 2026-11-12).
- **Sort de l'offre** (carte ; Compendium, *Magic Talent* S1, S5 ; MKUE p. 5, *"Composite" Effects* 1) : la carte pose
  `CastFromOffer(color)` (couleur de la carte défaussée) pour le tour : le joueur peut lancer **un** sort de cette couleur **de
  l'offre de sorts**, comme s'il était en main, en payant son mana (base ou renforcé). Le modificateur est retiré par ce sort. Le
  sort **reste dans l'offre** : il n'entre ni dans la zone de jeu ni dans la défausse, l'offre ne bouge pas. Jouée en combat ou
  non ; le sort de l'offre suit ses propres règles (un sort de combat ne se lance qu'en combat, spec 057 pour les autres).
- Avec ce lot, seuls les sorts de combat de l'offre sont lançables ; un sort de la spec 057 reste `Unsupported` et n'est pas
  proposé (il ne se joue pas de côté depuis l'offre).

### Exemple : Démolition et Tremblement de terre contre une tour de mage

De nuit, assaut d'une tour de mage défendue par des Ice Mages (`enemies.ron` : armure 6, attaque de glace 5, résistance à la Glace,
fortifiés par le site) :

1. Phase à distance : Tremblement de terre (mana vert + mana noir), option un ennemi : fortifié, armure 6 − 6 → **1**.
2. Démolition (mana rouge, un cristal) : les Ice Mages ne sont plus fortifiés par le site, et ne sont pas résistants au Feu :
   armure 1 (minimum).
3. Éclair de feu (un jeton rouge) : Attaque à distance de feu 5 contre armure 1, les Ice Mages sont vaincus en phase à distance.

Dans l'autre ordre (Démolition d'abord), Tremblement de terre ne trouverait plus de fortification : −3 seulement (FAQ p2).

## Modèle

Termes du glossaire : Sort, Effet en cours, Armure réduite, N'attaque pas, Fortifié, Doublement fortifié, Résistance, Offre de
sorts. Termes ajoutés au glossaire dans cette PR : **Lancer (un sort)** (*cast*), **Détruire (un ennemi)** (*destroy*,
`EnemyDestroyed`), **Perdre ses fortifications / ses résistances** (*lose fortifications / resistances*), **Ignorer les
fortifications des sites** (*ignore site fortifications*).

Données nouvelles (noms indicatifs, forme dans l'ADR 0008) :

- **`CardDefinition`** : `spell: bool` (ou la sorte de carte) : l'effet de base se paie de `color` ; l'effet renforcé de `color` et
  du noir. Les sorts de la spec 057 restent `Unsupported`.
- **`CardPlay`** : `Basic` reçoit le mana du sort dans son champ `mana` existant ; `Strong` reçoit `black: Option<ManaPayment>` ;
  `Basic` et `Strong` reçoivent `enemy: Option<EnemyToken>` (ennemi visé) et `from_offer: bool` (sort de l'offre, Don pour la
  magie). `serde(default)` pour les sauvegardes. Les littéraux des tests et du joueur scripté sont mis à jour (signalé dans la PR).
- **`EnemyModifierKind`** : `LosesFortifications`, `LosesResistances`, `LosesResistance(Element)`. `CombatEnemy` gagne
  `effective_fortification()` (aucune, simple, double) et `effective_resistances()`, lus partout où le combat lisait le jeton.
- **`ModifierKind`** : `OnAllEnemies(EnemyModifierKind)` ; `OnEnemyByFortification { plain, fortified }` (Tremblement de terre) ;
  `IgnoreSiteFortifications` (tour) ; `DeferredAttack { amount, element }` et `DestroyBlockedEnemy` (posés par la parade réussie,
  durée le combat) ; `CastFromOffer(BasicColor)` (tour, une fois).
- **`Effect`** : `AttackWithWound` prend un élément et un genre (Tempête de feu) ; `FlameWave { attack, block, per_enemy }` ou un
  bonus par ennemi sur `Attack`/`Block` ; `DestroyEnemy` (Tornade), `DestroyEnemyThenOthers(EnemyModifierKind)` (Désintégration) ;
  `Nothing` existant pour les sorts qui ne font que poser des modificateurs.
- **Contenu** : les dix sorts quittent `Unsupported` dans `spells.ron` ; l'effet de base de Don pour la magie dans
  `advanced_actions.ron`.

## Actions et événements

| Action | Validation (refusée si…) | Événements produits |
|---|---|---|
| `PlayCard` (existante) | sort sans le mana de sa couleur ; renforcé sans mana noir, de jour hors donjon ; ennemi visé absent, vaincu ou hors des cibles du sort ; Tornade, Désintégration hors phase d'attaque ; Désintégration sans ennemi non résistant au Feu ; Don pour la magie sans carte de couleur à défausser ; sort `from_offer` sans `CastFromOffer` de sa couleur, ou absent de l'offre | `CardPlayed` (points, modificateurs du joueur et des ennemis) ; `WoundFromCard` (Tempête de feu, Blizzard) ; `EnemyDestroyed` (nouveau) ; `CardDiscarded` (Don pour la magie) |
| `UseDeferredAttack` (nouvelle) | pas de `DeferredAttack` ; hors phase d'attaque ; pas d'attaque en cours | `DeferredAttackUsed` (nouveau) : Attaque de feu 4 ajoutée à l'attaque en cours |
| `ResolveBlock` (existante) | inchangée | `EnemyBlocked` (existant) ; en plus, parade réussie avec Bouclier ardent : `DeferredAttack` posé ; avec Bouclier explosif : `EnemyDestroyed` (sauf résistance au Feu) |

| Événement | Effet de `apply` | Révèle une information ? | Point de non-retour ? |
|---|---|---|---|
| `CardPlayed` (existant) | en plus : sort de l'offre laissé dans l'offre et `CastFromOffer` retiré ; modificateurs posés sur l'ennemi visé ou sur tous, sauf résistance | non | non |
| `EnemyDestroyed { enemy, source }` (nouveau) | l'ennemi est vaincu comme par une attaque (gloire, réputation, conquête : la logique de `EnemiesDefeated`, réutilisée) ; une créature invoquée est retirée sans gloire | non | non |
| `DeferredAttackUsed` (nouveau) | Attaque de feu 4 dans l'attaque en cours, modificateur retiré | non | non |

Aucun hasard nouveau : les tirages des parcours ne bougent pas (ADR 0007). Le sort lancé depuis l'offre ne révèle rien.

## Comportements attendus

### Lancer un sort

- **C1** — De jour, Éclair de feu en main, phase à distance contre des Prowlers (`enemies.ron` : armure 3, attaque physique 4) : base
  payée d'un cristal rouge, Attaque à distance de feu 5, les Prowlers vaincus ; sans mana rouge ni or, seulement de côté
  (`spell_basic_costs_its_color`) — MKUE p. 4, 3.b.
- **C2** — De jour, l'effet renforcé n'est jamais une action légale, même avec un jeton noir (`strong_spell_not_by_day`) — MKUE p. 5,
  2.a.
- **C3** — De nuit, Tempête de feu payée d'un cristal rouge et d'un dé noir : une Blessure en main et une Attaque de siège de feu 8 ;
  un dé or épuisé n'est pas proposé (`strong_spell_red_and_black_at_night`) — MKUE p. 4, 3.b ; p. 5, 2.b.
- **C4** — De jour, dans le combat d'un donjon (règles de nuit, spec 044), Tempête de feu payée d'un jeton rouge et d'un dé noir
  (`strong_spell_in_dungeon_by_day`) — FAQ p1.
- **C5** — Concentration renforcée ne propose pas un sort comme cible (`concentration_does_not_boost_spells`) — Compendium, *Basic
  Action Cards* S6.
- **C6** — Hors combat, les dix sorts ne se jouent que de côté (`combat_spells_only_in_combat`) — spec 052.

### Exposition

- **C7** — Phase à distance contre des Diggers (armure 3, Fortifié) seuls : aucune attaque ne peut être déclarée ; Exposition
  (mana blanc) visant les Diggers, sans attaque en cours : ils ne sont plus fortifiés, l'Attaque à distance 2 est perdue ; puis
  attaque déclarée sur les Diggers et Éclair de feu (cristal rouge) : 5 ≥ 3, vaincus en phase à distance. Avec une attaque en
  cours sur des Prowlers (armure 3) en plus, Exposition visant les Diggers met ses 2 points dans l'attaque des Prowlers
  (`expose_removes_fortification`) — carte ; MKUE p. 9, 3.a ; Q3.
- **C8** — Golems (armure 5, résistance physique) visés par Exposition : en phase d'attaque, Rage (Attaque 2) et Marche de côté
  (Attaque 1) comptent en plein : 3 points au lieu de 1 sans Exposition (`expose_removes_resistances`) — carte ; MKUE p. 9, 4.a.
- **C9** — Assaut d'une tour de mage, de nuit, Exposition de masse option fortifications : les Ice Mages défenseurs ne sont plus
  fortifiés ; option résistances : ils perdent la résistance à la Glace, et un effet bleu (Bouclier de glace renforcé) les atteint
  (`mass_expose_all_enemies_one_option`) — carte ; spec 053.

### Secousse et Tremblement de terre

- **C10** — Secousse option un ennemi sur des Swordsmen (armure 5) : armure 2 ; option tous, Swordsmen et Prowlers : 3 et 1
  (`tremor_target_or_all`) — carte ; Compendium, *Tremor* S1.
- **C11** — Tremblement de terre, Guardsmen (armure 7, Fortifié) et Prowlers, option tous : Guardsmen 3, Prowlers 1 ; en phase de
  parade aussi (`earthquake_doubles_against_fortified`) — Compendium, *Earthquake* S4, S5.
- **C12** — Exposition sur les Guardsmen, puis Tremblement de terre option un ennemi : −3 seulement, armure 4
  (`earthquake_after_lost_fortification`) — Compendium, *Earthquake* S1.

### Mur de flammes et Déluge de flammes

- **C13** — Phase de parade contre des Swordsmen (attaque physique 6) : Mur de flammes option Parade, Parade de feu 7, parade réussie
  (`flame_wall_block`) — carte.
- **C14** — De nuit, trois ennemis dont des Orc Summoners avec leur créature ; un des trois vaincu en phase à distance. Déluge de
  flammes en phase de parade : Parade de feu 7 + 2 × 2 = 11 (le vaincu et la créature non comptés) ; en phase d'attaque, après une
  attaque réussie dans cette phase, Attaque de feu 5 + 2 × 2 = 9 (`flame_wave_counts_enemies_faced`) — Compendium, *Flame Wave* S2,
  S3, Q4.

### Éclair de mana

- **C15** — Phase d'attaque, Éclair de mana (mana bleu) et mana rouge en plus : Attaque de feu glacé 7 ; en phase à distance, les
  options bleue et rouge ne sont pas proposées, la blanche (distance) et la verte (siège) oui (`mana_bolt_option_by_paid_color`) —
  carte ; Compendium, *Mana Bolt* S1.
- **C16** — De nuit, Tonnerre de mana payé d'un cristal bleu, d'un jeton noir et d'un cristal vert : Attaque de siège de glace 8
  (`mana_thunderbolt_three_mana`) — Compendium, *Mana Bolt* S1.

### Tourbillon et Tornade

- **C17** — Phase à distance, Tourbillon sur des Orc Summoners : aucune créature tirée au début de la phase de parade ; sur des
  Diggers (Fortifié) aussi permis ; sur un ennemi paré, pas une cible (`whirlwind_does_not_attack`) — carte ; spec 053.
- **C18** — De nuit, phase d'attaque, Tornade sur des Fire Mages (armure 5, résistance au Feu, gloire 5) : vaincus, gloire +5 ; en
  phase de parade, pas une action légale (`tornado_destroys_in_attack_phase`) — carte.

### Bouclier ardent et Bouclier explosif

- **C19** — Parade des Prowlers (attaque 4) : Bouclier ardent, Parade de feu 4, parade réussie ; en phase d'attaque, `UseDeferredAttack`
  ajoute une Attaque de feu 4 à l'attaque en cours contre des Wolf Riders (armure 4) : vaincus. Parade ratée (contre des Swordsmen,
  attaque 6) : pas d'attaque différée (`burning_shield_attack_after_successful_block`) — carte ; Compendium, *Burning Shield* S2, S3.
- **C20** — De nuit, Bouclier explosif en parade des Prowlers avec Détermination (Parade 2) : parade réussie, Prowlers détruits,
  gloire +2 ; contre des Fire Golems (attaque de feu 3, résistance au Feu) parade réussie, pas détruits
  (`exploding_shield_destroys_blocked_enemy`) — carte ; Compendium, *Exploding Shield* S1.
- **C21** — Contre la créature invoquée par des Orc Summoners, parade réussie avec Bouclier explosif et assez d'autres points de
  Parade : la créature est détruite sans gloire, les Summoners restent
  (`exploding_shield_on_summoned_monster`) — FAQ p2 ; Compendium, *Exploding Shield* Q2.

### Frisson et Frisson fatal

- **C22** — Frisson sur des Fire Mages : ils n'attaquent pas ; en phase d'attaque, Éclair de feu (Attaque de feu 5) compte en plein
  contre leur armure 5 ; sur des Ice Mages, pas une cible (`chill_removes_fire_resistance`) — carte ; Compendium, *Chill* S1.
- **C23** — De nuit, Frisson fatal sur des Swordsmen (armure 5) : n'attaquent pas, armure 1 (`lethal_chill_armor`) — carte.

### Démolition et Désintégration

- **C24** — Assaut d'une forteresse défendue par des Guardsmen (armure 7, Fortifié : doublement fortifiés) : Démolition en phase à
  distance : fortifiés par leur jeton seulement, une Attaque de siège peut les viser ; armure 6. Après la retraite, au tour
  suivant, un nouvel assaut les retrouve doublement fortifiés (`demolish_ignores_site_fortifications`) — carte ; Compendium,
  *Demolish* S1, S5.
- **C25** — Assaut d'une tour de mage défendue par des Fire Mages (armure 5, résistance au Feu) : Démolition les rend non fortifiés,
  leur armure reste 5 (`demolish_armor_not_on_fire_resistant`) — FAQ p1 ; Compendium, *Demolish* Q2.
- **C26** — De nuit, phase d'attaque, Désintégration contre Prowlers, Swordsmen et Fire Mages : Prowlers détruits, Swordsmen armure 4,
  Fire Mages 5 ; les Fire Mages ne sont pas une cible (`disintegrate_destroys_and_weakens_others`) — Compendium, *Disintegrate* S1,
  S3.

### Don pour la magie

- **C27** — Offre de sorts : Fireball, Snowstorm, Tremor. En combat, Don pour la magie de base en défaussant Rage (rouge) ; puis
  Éclair de feu lancé depuis l'offre (mana rouge) : Attaque à distance de feu 5 ; Fireball reste dans l'offre, à sa place ; plus de
  sort de l'offre ce tour (`magic_talent_casts_spell_from_offer`) — carte ; Compendium, *Magic Talent* S1.
- **C28** — Défausser Marche (verte) : seule Tremor (verte) est proposée ; main sans carte de couleur (Blessures seulement) : pas
  jouable (`magic_talent_same_color`) — carte (LL).
- **C29** — De nuit, depuis l'offre, Tempête de feu payée rouge et noir : la Blessure et l'Attaque de siège 8
  (`magic_talent_strong_spell_from_offer`) — Compendium, *Magic Talent* S1.
- **C33** — Un sort lancé depuis l'offre compte comme un sort de la main dans chaque phase (issue privée 246) : Éclair de feu en phase
  à distance contre des Prowlers (armure 3) les bat ; Mur de flammes (Parade de feu 7) en parade contre des Prowlers (Attaque 4)
  les pare ; en phase d'attaque, à l'assaut de la tour de mage défendue par des Fire Mages (armure 5, résistance au Feu), Éclair de
  feu seul ne suffit pas (5 divisé par deux) et l'assaut est repoussé, alors qu'après Exposition (fortification et résistances
  perdues) il les bat et la fin du combat donne `SiteConquered`. Le sort lancé est un fait du tour
  `TurnFact::SpellCastFromOffer` (carte, option, effet obtenu), le sort restant dans l'offre (`offer_spell_ranged_victory`,
  `offer_spell_block`, `offer_spell_attack_conquers_tower`) — Compendium, *Magic Talent* S1.

### Contenu et vue

- **C30** — Les dix sorts de combat et l'effet de base de Don pour la magie ne sont plus `Unsupported` ; les dix sorts de la 057 le
  restent (`combat_spells_supported`) — contenu.
- **C31** — La vue du combat donne pour chaque ennemi sa fortification et ses résistances effectives (en plus de l'armure effective,
  spec 053) ; la vue du joueur donne l'attaque différée et `CastFromOffer` dans les modificateurs (`view_shows_lost_fortifications_and_resistances`).
- **C32** — Retour en arrière avant `EnemyDestroyed` : l'ennemi revient, la gloire aussi ; sauvegarde rechargée après : il reste
  vaincu (`destroyed_enemy_follows_undo_and_reload`) — spec 024.

## Cas limites et refus

- **L1** — Sort renforcé payé de la couleur mais sans noir, ou d'or de nuit : refusé.
- **L2** — Exposition, Secousse, Tourbillon, Frisson sans ennemi visé : refusés ; visant un ennemi vaincu : refusés.
- **L3** — `UseDeferredAttack` deux fois : refusée la seconde ; hors phase d'attaque : refusée.
- **L4** — Don pour la magie de base, puis un second sort de l'offre : refusé ; un sort de l'offre de la spec 057 : jamais proposé.
- **L5** — Tous les ennemis détruits par Tornade ou Désintégration : victoire comme par une attaque (spec 031).

## Information cachée

Rien de caché : offres, combat et modificateurs sont publics. Champs ajoutés à la vue : fortification et résistances effectives de
chaque ennemi du combat (`PlayerView::combat`, C31) ; `DeferredAttack` et `CastFromOffer` passent par `PlayerView::modifiers`
(spec 052).

## Libellés (i18n)

Les textes `card.<sort>.name`, `strong_name`, `basic_text` et `strong_text` existent (relus dans `assets/i18n/fr.json`).
Corrigé : `card.demolish.basic_text` « Ignorer les fortifications ce tour ; ennemis Armure -1 (min. 1) » devient « Ignorer les
fortifications des sites ce tour ; ennemis Armure -1 (min. 1) » (Compendium, *Demolish* S1). Nouvelles clés (fr ; en à l'avenant) :

- `combat.enemy.no_fortification` : « plus fortifié » ; `combat.enemy.no_resistances` : « plus de résistances » ;
  `combat.enemy.no_fire_resistance` : « plus de résistance au Feu » ; `combat.enemy.destroyed` : « détruit ». Sous `combat.enemy`,
  à côté de `defeated` « vaincu » et `does_not_attack` « n'attaque pas » (relus).
- `modifier.ignore_site_fortifications` : « fortifications des sites ignorées » ; `modifier.deferred_attack` : « Attaque de feu
  {amount} en phase d'attaque » (fr « Attaque de feu 4 en phase d'attaque ») ; `modifier.cast_from_offer` : « un sort {color} de
  l'offre » (`{color}` repris de `effect.gain_spell`, relu : « Payer {color} : un sort {color} de l'offre »).
- `combat.use_deferred_attack` : « Ajouter l'Attaque de feu {amount} » (bouton de la phase d'attaque).
- `offers.cast` : « Lancer » (bouton d'un sort de l'offre avec `CastFromOffer`, à côté de `offers.learn`, spec 056).
- `spell.strong_needs_black` : « Renforcé : mana noir, de nuit » (infobulle du bouton Renforcer grisé de jour).
- `effect.on_all_enemies` : « tous les ennemis : {effect} » ; `effect.destroy_enemy` : « détruire un ennemi ».
- `play_area.line_from_offer` : « {card} (depuis l'offre) : {effect} » (issue privée 246).
- `turn_fact.enemy_destroyed` : « {enemy} détruit » (fr « Prowlers détruit » ; le nom des ennemis est au pluriel dans `fr.json`,
  l'accord ne suit pas : « Prowlers : détruits » proposé à l'implémentation si la relecture le préfère).

## Interface

- **Payer un sort** : bouton Jouer (base) avec la fenêtre de paiement de la spec 022, qui demande le mana de la couleur du sort ;
  bouton Renforcer actif seulement quand le noir est payable (C1, C2, C3, actions légales), sinon grisé avec l'infobulle
  `spell.strong_needs_black`. Le mana noir se choisit comme le premier mana (fenêtre de paiement), comme le mana en plus de la 053.
- **Ennemi visé** : après le clic sur le sort, choix de l'ennemi parmi les cibles légales sur l'écran de combat, comme les capacités
  d'unités qui visent un ennemi (spec 053) (C10, C17, C22, actions légales).
- **Options** : Secousse (un ennemi, tous), Exposition de masse, Mur de flammes, Éclair de mana (une par couleur) par les boutons
  d'options existants (spec 073) ; option illégale grisée (C15, actions légales).
- **Déluge de flammes** : en combat, ses boutons d'options disent le total avec les ennemis affrontés (Q4), « Attaque de feu 9 »
  contre deux jetons ; hors combat, les points de base (issue privée 245, `flame_wave_buttons_show_total_with_enemies_faced`).
- **Ennemis du combat** : « plus fortifié », « plus de résistances », « détruit » sur la carte de l'ennemi (C31,
  `PlayerView::combat`) ; placement : vérification à l'œil.
- **Attaque différée** : bouton « Ajouter l'Attaque de feu 4 » sur l'écran de combat en phase d'attaque, tant que l'action est
  légale (C19, actions légales `UseDeferredAttack`).
- **Sort de l'offre** : avec `CastFromOffer`, bouton « Lancer » sur les sorts permis de la fenêtre des offres, puis paiement et
  ennemi visé comme pour un sort en main (C27, C28, actions légales).
- **Sort lancé depuis l'offre** : une ligne de la zone de jeu, après les cartes jouées, « Éclair de feu (depuis l'offre) : Attaque
  à distance de feu 5 », tirée du fait du tour `SpellCastFromOffer` et absente de la liste des faits du tour pour ne pas la dire deux
  fois ; tous les ennemis vaincus, plus de bouton « Lancer » dans l'offre (C33, issue privée 246).
- **Ligne d'un sort lancé** : Déluge de flammes y dit l'effet obtenu, « Attaque de feu 9 » contre deux jetons, pas les points de
  base (issue privée 246).
- **Effets en cours et faits du tour** : `IgnoreSiteFortifications`, `DeferredAttack`, `CastFromOffer` dans la zone de jeu (C31,
  `PlayerView::modifiers`) ; « Prowlers détruit » dans les faits du tour (C20, `TurnFact`).

### Raccourcis clavier

Aucun : tout passe par les boutons existants des cartes, des offres et du combat.

## Effets sur les specs déjà implémentées

- **022, 025** : un sort se paie de sa couleur dès la base ; le mana noir paie un effet renforcé (Q4 de la spec 025 levée pour le
  noir des sorts) ; Concentration ne prend pas un sort pour cible.
- **031, 032, 043** : la fortification et les résistances comparées deviennent les effectives ; un ennemi peut être vaincu sans
  attaque (`EnemyDestroyed`).
- **033** : la créature invoquée peut être détruite.
- **050** : dix sorts quittent `Unsupported` ; un sort de l'offre peut être lancé sans quitter l'offre.
- **053** : `AttackWithWound` prend un élément et un genre ; règle des résistances étendue à « détruire ».

## Hors périmètre

- Les dix autres sorts (Voyage souterrain, Ailes du vent, Restauration, Distorsion spatiale, Appel aux armes, Méditation, Fuite de
  mana, Puits de mana, Lecture des pensées, Flux d'énergie) : spec 057 (Q1).
- Ce qui ne vise que d'autres joueurs (aucun sort de ce lot) ; immunité arcanique (aucun ennemi du jeu de base, spec 053).
- Artefacts qui lancent un sort (Tome de tous les sorts) : spec 055.

## Plan de développement

Moteur et écran (issue privée 118) : deux fils sur la même branche, plan [`plans/054-sorts.md`](plans/054-sorts.md) :

1. **Moteur** : paiement des sorts, ennemi visé, modificateurs d'ennemis nouveaux, détruire, chaque sort, Don pour la magie,
   contenu, vue, persistance (C1 à C32).
2. **Écran** : libellés, paiement, ennemi visé, attaque différée, sort de l'offre, vérification à l'œil, docs, bilan.

## Décisions (2026-11-12)

Mathieu accepte toutes les réponses par défaut (« ok avec défauts »), appliquées dans le texte ci-dessus :

1. **Q1 — Découpage.** Ce lot (paiement des sorts, dix sorts de combat, Don pour la magie de base) en 054 ; une **spec 057
   « Sorts : déplacement, unités et mana »** pour les dix autres, insérée juste après dans `PLAN.md`. **Décision : oui.**
2. **Q2 — Ennemi détruit.** Un ennemi détruit est **vaincu** (gloire, réputation, conquête du site) ; une créature invoquée
   détruite, sans gloire. **Décision : vaincu.**
3. **Q3 — Ennemi visé et attaque en cours.** L'ennemi visé se choisit librement en jouant le sort ; les points vont à l'attaque en
   cours s'il y en a une, sinon ils sont perdus (C7). **Décision : points perdus sans attaque en cours.**
4. **Q4 — Ennemis affrontés de Déluge de flammes.** Les jetons pas vaincus avant la phase en cours (ceux vaincus pendant compris),
   sans la créature invoquée, comptés en jouant la carte. **Décision : oui.**
5. **Q5 — Démolition hors combat.** En combat seulement. **Décision : en combat.**
6. **Q6 — Frisson après les dégâts.** Non : mêmes moments que Tourbillon et les Illusionnistes. **Décision : non.**

Tranché à part (Mathieu, 2026-11-12) : Don pour la magie de base lance un sort **de la même couleur** que la carte défaussée,
version *Lost Legion*.

## Questions ouvertes

Aucune.
