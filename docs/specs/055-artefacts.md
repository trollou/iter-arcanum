# 055 — Artefacts

- Statut : validée le 2026-11-13 (questions Q1 à Q10 tranchées : réponses proposées acceptées ; Q4 reportée, issue #8)
- Jalon : J5
- Dépend de : 052 (moteur d'effets, ADR 0008, règle « effet utilisable »), 053, 056 et 057 (`WoundFromCard`, `CardDestroyed`,
  Entraînement, choix en attente, unités rendues disponibles), 054 (lancer un sort, mana noir, `DoesNotAttack`, ennemi visé), 041
  et 050 (artefacts gagnés, offres), 042 (unités, soin, dégâts sur les unités), 033 (Invocateurs), 034 (Blessures, assommé), 035
  (gloire et niveaux), 016 et 023 (fin du tour, fin de manche), 020 à 025 (Source, cristaux, jetons), 044 et 045 (donjons, ruines,
  dés de récompense), 024 et 038 (retour en arrière, générateurs par usage), 063 (décompte final) ; toutes implémentées
- Livret : MKUE p. 4 (*Round* 2.g ; *Deed Cards* 3.c, 3.e) ; p. 5 (*Using Units* 8, 8.a, 8.b ; *"Gain" Effects* 2) ; p. 6 (*A
  Player's Turn* 5.b, 5.c) ; p. 7 (*Movement*, coûts) ; p. 9 et 10 (combat) ; p. 11 (*End of the Turn*) ; p. 15 (*The Greatest
  Loot*). Texte des cartes : `docs/regles/cartes/artefacts.md` (Bannière de peur et Corne de la colère : version *Lost Legion*, qui
  fait foi). Compendium, *Artifact Cards* (S1, Q2 ; Amulet of Darkness S1, S2 ; Amulet of the Sun S1 ; Banner of Courage S1 ;
  Banner of Fear Q1, Q2, S3, S4, Q5, Q6 ; Banner of Protection Q1, S2 ; Book of Wisdom S1 ; Rings S1 ; Endless Bag of Gold Q1, Q2 ;
  Endless Gem Pouch S1 ; Golden Grail S1 ; Horn of Wrath S1, S2 ; Sword of Justice S1, S3, Q4, S6, S7, S8), *Minimum Turn* S1, S2,
  *End of Turn* S5, *Miscellaneous* S10, S12, *Time Bending* S7, O12
- ADR : [0007 Un générateur par usage](../decisions/0007-generateurs-par-usage.md) (usage nouveau), [0008 Moteur
  d'effets](../decisions/0008-moteur-d-effets.md) (modificateurs nouveaux)
- Crates touchées : arcanum-core (règles, contenu, vue), arcanum-ui, arcanum-persistance (partie préparée), `assets/i18n/`

## Objectif

Les **16 artefacts** du jeu de base font ce qu'ils disent. Un artefact se joue pour son effet de base, ou se **détruit** (retiré du
jeu) pour son effet renforcé, sans mana. Les quatre **bannières** s'assignent à une unité, qu'elles renforcent tant qu'elles y
restent. Après cette spec, plus aucune carte n'est `Unsupported`.

| Artefact (fr.json) | Base | Renforcé (en le détruisant) | Ce qui est nouveau |
|---|---|---|---|
| Bannière de gloire (*Banner of Glory*) | assignée : l'unité Armure +1, +1 à ses attaques et parades ; Gloire +1 quand elle attaque ou pare | ce tour, toutes les unités Armure +1, +1 en attaque et parade ; Gloire +1 par unité qui attaque ou pare | bannière ; bonus d'Armure |
| Bannière de peur (*Banner of Fear*) | assignée : en phase de parade, épuiser l'unité pour annuler une attaque, Gloire +1 | (*Lost Legion*) jusqu'à 3 ennemis n'attaquent pas ce combat | capacité donnée à une unité |
| Bannière de protection (*Banner of Protection*) | assignée : l'unité Armure +1, résistances au Feu et à la Glace | à la fin du tour, jeter les Blessures reçues ce tour | Blessures reçues comptées |
| Bannière de courage (*Banner of Courage*) | assignée : une fois par manche, hors combat, la retourner pour rendre l'unité disponible | hors combat, rendre disponibles toutes les unités | bannière retournée |
| Anneau de rubis, de saphir, de diamant, d'émeraude (*Rings*) | un jeton et un cristal de la couleur ; Gloire +1 | ce tour, mana illimité de la couleur et noir ; Gloire +1 par sort de la couleur lancé | mana illimité |
| Épée de justice (*Sword of Justice*) | défausser des cartes : Attaque 3 par carte ; Gloire +1 par ennemi vaincu ce tour | Attaques physiques doublées en phase d'attaque ; les ennemis perdent la résistance physique ; Gloire +1 par ennemi vaincu | doublement |
| Corne de la colère (*Horn of Wrath*) | (*Lost Legion*) Attaque de siège 5 ; un dé de mana : noir ou rouge, une Blessure | Attaque de siège 5, jusqu'à +5, un dé par +1 | dés lancés par une carte |
| Bourse sans fond (*Endless Bag of Gold*) | Influence 4, Gloire +2 | Influence 9, Gloire +3 | — |
| Poche de mana (*Endless Gem Pouch*) | deux dés de mana : un cristal par couleur (or : au choix ; noir : Gloire +1) | un jeton de chaque couleur de base, plus or (jour) ou noir (nuit) | dés lancés par une carte |
| Graal doré (*Golden Grail*) | Soin 2 ; Gloire +1 par point de ce Soin dépensé | Soin 6 ; une carte piochée par Blessure de la main soignée ce tour | — |
| Livre de sagesse (*Book of Wisdom*) | détruire une carte action : une action avancée de sa couleur de l'offre, en main | idem avec un sort de l'offre, et un cristal de la couleur | sort gagné en main |
| Amulette du soleil (*Amulet of Sun*) | un jeton or ; de nuit, ce tour : forêts à 3, mana or utilisable, défenseurs et ruines révélés comme de jour | idem avec trois jetons or | règles de jour la nuit |
| Amulette des ténèbres (*Amulet of Darkness*) | un jeton de toute couleur ; de jour, ce tour : déserts à 3, mana noir utilisable comme de nuit | idem avec trois jetons | règles de nuit le jour |

Noms français des artefacts : ceux des cartes françaises (glossaire, issue privée 313).

## Règles

### Règles communes

- **Jouer un artefact** (MKUE p. 4, 3.c) : pour son **effet de base**, il va dans la zone de jeu comme une autre carte ; pour son
  **effet renforcé**, sans mana, il est **détruit** (retiré du jeu, `CardDestroyed`, comme une carte action détruite, spec 056) au
  lieu d'aller dans la zone de jeu. Les deux comptent comme une carte jouée pour le tour minimal (Compendium, *Minimum Turn* S2).
  **De côté** (MKUE p. 4, 3.e) : comme aujourd'hui (spec 041, test `taken_artifact_plays_sideways`).
- **Moments** (MKUE p. 6, 5.b, 5.c) : règle « effet utilisable » (spec 052) ; chaque artefact ci-dessous précise ce qui n'en découle
  pas. Un effet de **mana** (jeton, cristal, mana illimité) se joue à tout moment du tour, combat compris ; un **Soin** hors combat.
- **Pas une carte action** : Concentration, Effet maximum, Entraînement, Décomposition et le Livre de sagesse ne visent pas un
  artefact (comme aujourd'hui, spec 056, test `spells_and_artifacts_are_not_action_cards`). Défaussé pour sa couleur (Don pour la
  magie, Improvisation), un artefact ne donne rien : il n'en a pas (Compendium, *Artifact Cards*, Endless Gem Pouch S6, Tome S4).
- **Gloire** : « Gloire +N » est gagnée **aussitôt** (`FameGained`) ; le niveau ne change qu'à la fin du tour (spec 035 ;
  Compendium, *Endless Bag of Gold* Q1).
- **Gloire « ce tour »** (Épée de justice, anneaux renforcés, Bannière de gloire renforcée) : compte tout le tour, avant comme après
  la carte ; ce qui est déjà fait rapporte en jouant la carte, la suite au fur et à mesure (Q3). Une créature invoquée n'est pas un
  ennemi vaincu (spec 033) ; un ennemi détruit par un sort l'est (spec 054).
- **Lancer un dé de mana** (Corne de la colère, Poche de mana) : un dé de la réserve, six faces équiprobables, par un générateur nouveau
  `RngUsage::ArtifactDice` (ADR 0007 : les tirages des parcours ne bougent pas). Chaque lancer est un **point de non-retour**.
- **Règles de nuit** (Compendium, *Miscellaneous* S10) : de nuit, et en combat dans un donjon ou un tombeau (spec 044), le mana noir
  sert et l'or non. Les amulettes en changent une partie pour le tour, rien d'autre (tactiques, coûts non cités).

### Bannières

- **Assigner** (MKUE p. 5, 8 ; Compendium, *Artifact Cards* S1) : à tout moment du tour, combat compris, une bannière de la main va
  **sous une unité** du joueur, disponible ou épuisée, blessée ou non. C'est son effet de base : l'unité en profite tant que la
  bannière y reste ; l'effet renforcé ne se joue que depuis la main. Une bannière gagnée ce tour arrive à la fin du tour (spec 041),
  donc pas avant le tour suivant. Une unité porte **une bannière au plus**.
- **Perdre la bannière** (MKUE p. 5, 8.a) : l'unité détruite ou renvoyée, ou une autre bannière assignée à la même unité : la
  bannière va dans la **défausse**. Ce n'est pas une carte jouée (Compendium, *Minimum Turn* S1).
- **Fin de manche** (MKUE p. 4, 2.g ; p. 5, 8.b) : les bannières retournées (Bannière de courage) se remettent face visible ; le
  joueur choisit les bannières qu'il **reprend** (elles rejoignent le deck avant son mélange) ; les autres restent sur leur unité
  (Q2).
- **Distorsion temporelle** (Compendium, *Time Bending* S7, O12) : une bannière assignée reste sur son unité, une Bannière de
  courage retournée le reste.
- **Décompte** (MKUE p. 15, *The Greatest Loot*) : une bannière sur une unité compte comme un artefact du joueur ; un artefact
  détruit ne compte plus.

### Bannière de gloire

- **Base** (carte ; Compendium, *Miscellaneous* S12) : l'unité porteuse a Armure +1 (dégâts, spec 042) ; +1 à chaque Attaque (à
  distance et de siège comprises) ou Parade que donne une de ses capacités, du même élément, seulement si la capacité en donne une ;
  **Gloire +1** à chaque activation d'une capacité d'Attaque ou de Parade de l'unité, que l'attaque ou la parade réussisse ou non.
- **Renforcé** (carte) : pour le tour, toutes les unités ont Armure +1 et `UnitBonus { attack: 1, block: 1 }` (spec 053) ;
  **Gloire +1 par unité** qui active une capacité d'Attaque ou de Parade ce tour, une fois par unité (Q3). En combat seulement
  (effet utilisable).

### Bannière de peur

- **Base** (carte, version *Lost Legion* ; Compendium, *Banner of Fear* Q1, Q2, S3, S4) : l'unité porteuse gagne une capacité. En
  **phase de parade**, si l'unité est **disponible et non blessée**, l'épuiser : un ennemi pas vaincu, pas paré, qui attaque encore,
  reçoit `DoesNotAttack` ; **Gloire +1**. L'unité n'utilise aucune autre capacité avec ; la bannière reste (rendue disponible,
  l'unité peut recommencer). Ce n'est pas une parade (Q1). Tous les ennemis du jeu de base n'ont qu'une attaque. Pas contre des
  **Invocateurs** ni leur créature : le moteur tire la créature au début de la phase de parade (spec 033), avant que la bannière
  puisse servir, et annuler l'attaque de la créature n'est pas permis (Q6). Simplification provisoire (Q4, issue #8).
- **Renforcé** (carte, version *Lost Legion* ; Compendium, *Banner of Fear* Q2) : **jusqu'à 3 ennemis** reçoivent `DoesNotAttack`,
  choisis en jouant la carte. Mêmes moments et mêmes cibles que Tourbillon (spec 054) : phase à distance ou de parade, ennemis
  qui attaquent encore ; en phase à distance, contre des Invocateurs aussi (aucune créature tirée).

### Bannière de protection

- **Base** (carte) : l'unité porteuse a Armure +1 et les résistances au **Feu** et à la **Glace**, en plus des siennes (spec 042).
- **Renforcé** (carte ; Compendium, *Banner of Protection* Q1, S2) : à tout moment du tour, combat compris ; pose
  `ThrowAwayReceivedWounds` pour le tour. **Au début de la fin du tour** (avant les défausses et la pioche, MKUE p. 11), les
  Blessures **reçues ce tour** sont jetées : en main, autant que celles reçues en main ce tour (dégâts, cartes comme Coup de sang ou
  la Corne de la colère), au plus celles qui y restent ; dans la défausse, autant que celles reçues dans la défausse (Toxique). Pas celles
  des unités, ni celles piochées. Toutes, sans choix (Q5). Elle n'empêche pas d'être assommé.

### Bannière de courage

- **Base** (carte ; Compendium, *Banner of Courage* S1 ; *Artifact Cards* Q2) : hors combat, une fois par manche, **retourner** la
  bannière pour rendre l'unité porteuse **disponible**, blessée ou non (elle le reste). Elle se remet face visible au début de la
  manche suivante. Pas de retournement si l'unité est disponible.
- **Renforcé** (carte) : hors combat, **toutes** les unités épuisées redeviennent disponibles (`UnitReadied`), blessées ou non.

### Anneaux

- **Base** (carte) : un jeton et un cristal de la couleur (`TokenAndCrystal`, spec 042 ; inventaire plein : un jeton, MKUE p. 5,
  « Gain » 2) ; Gloire +1.
- **Renforcé** (carte ; Compendium, *Rings* S1) : pose `EndlessMana(couleur)` pour le tour. Tout paiement de mana peut se faire en
  mana **de la couleur** ou **noir**, autant de fois qu'on veut, sans dé ni jeton ; le noir seulement là où il sert (nuit, donjon,
  Amulette des ténèbres). Ce n'est ni un dé de la Source ni un jeton. **Gloire +1 par sort de la couleur lancé** ce tour (base ou
  renforcé, depuis la main ou l'offre avec Don pour la magie) (Q3). Deux anneaux renforcés se cumulent.

### Épée de justice

- **Base** (carte ; Compendium, *Sword of Justice* Q4, S8) : en **phase d'attaque** ; défausser des cartes de la main, aucune
  Blessure, pas l'Épée : **Attaque physique 3 par carte**, une seule attaque (les points s'ajoutent à l'attaque en cours). **Gloire
  +1 par ennemi vaincu ce tour** (Q3). Zéro carte défaussée : permis, pour la seule gloire (Q8). Avec les Mages d'Altem (mana
  noir), l'Attaque devient de feu glacé, ou de siège et se joue alors aussi en phase à distance (S6, S2 ; issue privée 272).
- **Renforcé** (carte ; Compendium, *Sword of Justice* S1, S3, S6, S7) : en combat, à toute phase ; pose `DoublePhysicalAttacks`
  pour le tour : en **phase d'attaque**, chaque contribution d'**Attaque physique** ajoutée ensuite (carte, de côté, unité,
  Agilité) compte **double**, bonus compris (Concentration, Bannière de gloire) ; une Attaque d'un autre élément, non. Tous les
  ennemis reçoivent `LosesResistance(Physical)` pour le tour, aussitôt. Gloire +1 par ennemi vaincu ce tour. Avec les Mages d'Altem
  (feu glacé), le doublement s'applique avant la conversion ; avec leur siège, rien n'est doublé, le siège ne servant qu'en phase à
  distance (S6 ; issue privée 272).

### Corne de la colère

- **Base** (carte, version *Lost Legion* ; Compendium, *Horn of Wrath* S1) : **Attaque de siège physique 5** (effet utilisable :
  phase à distance ou d'attaque) ; lancer un dé de mana : **noir ou rouge**, une Blessure en main aussitôt (`WoundFromCard`, compte
  pour être assommé en combat, spec 056).
- **Renforcé** (carte, version *Lost Legion* ; Compendium, *Horn of Wrath* S2) : Attaque de siège 5, plus **0 à +5** choisi en
  jouant la carte, avant les dés ; un dé par point ajouté, une Blessure par noir ou rouge.

### Bourse sans fond

- **Base / Renforcé** (carte ; Compendium, *Endless Bag of Gold* Q1, Q2) : Influence 4 et Gloire +2 / Influence 9 et Gloire +3.
  Se joue quand l'Influence sert (effet utilisable, spec 052), interaction ou non ; la gloire vient même si l'Influence reste.

### Poche de mana

- **Base** (carte) : **deux dés** de mana. Par dé : sa couleur de base, un cristal (inventaire plein : un jeton) ; **or**, un cristal
  de la couleur choisie (choix en attente, comme le dé de récompense or, spec 044) ; **noir**, Gloire +1 au lieu d'un cristal.
- **Renforcé** (carte ; Compendium, *Endless Gem Pouch* S1) : un jeton de chaque couleur de base, plus un jeton **or** de jour, ou
  **noir** de nuit ou en combat dans un donjon ou un tombeau.

### Graal doré

- **Base** (carte ; Compendium, *Golden Grail* S1) : Soin 2 (spec 034 et 042 : Blessures de la main jetées aussitôt, le reste en
  réserve pour les unités). **Gloire +1 par point de ce Soin dépensé** ce tour : les points du Graal sont dépensés **en premier**,
  donc les deux premiers points de soin dépensés après elle rapportent (Q7).
- **Renforcé** (carte) : Soin 6 ; pose `DrawPerHandWoundHealed` pour le tour : chaque Blessure **de la main** jetée par un Soin ensuite
  (celles du Graal comprises) fait piocher une carte du deck, s'il en reste (spec 034).

### Livre de sagesse

- **Base** (carte ; Compendium, *Book of Wisdom* S1) : comme Entraînement renforcé (spec 056, `TrainFromOffer(Hand)`) : détruire
  une carte action de la main (de base ou avancée), prendre **en main** une action avancée de sa couleur dans l'offre, qui se
  remplit (spec 050). Mêmes moments qu'Entraînement.
- **Renforcé** (carte) : détruire une carte action de la main, prendre **en main** un sort de sa couleur dans l'offre de sorts, et un
  cristal de cette couleur (inventaire plein : un jeton).

### Amulettes

- **Amulette du soleil, base** (carte ; Compendium, *Amulet of the Sun* S1, *Amulet of Darkness* S2) : un jeton **or**. **De nuit**,
  pose `DayRules` pour le tour : forêts à **3** (désert inchangé : 3) ; le mana **or** sert (jetons, cristaux non : il n'y en a pas,
  dé or de la Source pris comme dé du tour) ; les défenseurs des forteresses et tours de mage voisines sont révélés comme de jour,
  y compris celles déjà voisines en jouant l'amulette ; les jetons de ruines face cachée du plateau sont retournés, et une ruine
  révélée ce tour reçoit son jeton face visible (spec 043, 045) (Q6). Le mana noir sert encore.
- **Amulette des ténèbres, base** (carte ; Compendium, *Amulet of Darkness* S1, S2) : un jeton de **toute couleur**, or et noir
  compris. **De jour**, pose `NightMana` pour le tour : déserts à **3** ; le mana **noir** sert comme de nuit (jetons, dé noir de la
  Source pris comme dé du tour, sort renforcé) (Q10). L'or sert encore ; défenseurs et ruines restent révélés comme de jour.
- **Renforcé** (cartes) : le même effet, avec trois jetons or (soleil) ou trois jetons de toutes couleurs, choisies une à une
  (ténèbres).

### Exemple : Bannière de gloire et Épée de justice

Les Paysans (`units.ron` : Attaque 2 ou Parade 2, Armure 3) portent la Bannière de gloire. Combat contre des Ironclads
(`enemies.ron` : armure 3, Attaque physique 4 Brutale, résistance physique, gloire 4). En phase de parade, les Paysans parent avec
Parade 3 (2 + 1) : Gloire +1, même si la parade ne suffit pas contre l'Attaque 4. En phase d'attaque, Épée de justice renforcée (détruite) :
les Ironclads perdent leur résistance physique. Rage (Attaque 2) compte 4 ; l'armure 3 est dépassée, Ironclads vaincus, gloire 4,
plus Gloire +1 (Épée). Sans l'Épée, les 2 points de Rage auraient été divisés par deux (résistance physique).

## Modèle

Termes du glossaire : Artefact, Unité disponible / épuisée, Défausse, Deck, Zone de jeu, Détruire (une carte), Jeter (une Blessure),
Soin, Source, Jeton de mana, Cristal, Gloire, Invocateur, Phase de parade. Termes ajoutés au glossaire dans cette PR :
**Bannière** (*Banner*, `Artifact::is_banner`), **Assigner (une bannière)** (*assign*, `Action::AssignBanner`), **Détruire (un
artefact)** (*throw away*, `CardDestroyed`, comme une carte action, spec 056 Q4), **Retourner (une bannière)** (*flip face down*,
`banner_flipped`), **Reprendre (une bannière)** (*reclaim*, fin de manche), **Mana illimité** (*endless supply*,
`EndlessMana`), **Annuler une attaque** (*cancel an attack*, Bannière de peur, `DoesNotAttack`).

Données nouvelles (noms indicatifs, forme dans l'ADR 0008) :

- **`OwnedUnit`** : `banner: Option<CardId>` et `banner_flipped: bool` (Bannière de courage).
- **`CardPlay`** : une variante `ThrowAway { option, … }` (effet renforcé d'un artefact, sans mana) ; les choix des cartes s'y
  ajoutent : `discard: Vec<CardId>` (Épée de justice), `enemies` (Bannière de peur renforcée, comme Ailes de la nuit),
  `bonus: u8` (Corne de la colère renforcée), `colors: Vec<ManaColor>` (amulette des ténèbres), `gain` existant (Livre de sagesse).
  `serde(default)`.
- **`ManaPayment`** : une source `Endless(ManaColor)` (mana illimité).
- **`ModifierKind`** : `UnitArmorBonus(u8)` (tour), `FamePerUnitAttackOrBlock`, `EndlessMana(BasicColor)` (avec la gloire par sort),
  `FamePerEnemyDefeated`, `DoublePhysicalAttacks`, `ThrowAwayReceivedWounds`, `DrawPerHandWoundHealed`, `DayRules`, `NightMana`,
  et un compteur de gloire de le Graal doré (`GrailFame { left: u8 }`).
- **`PlayerState`** : Blessures reçues ce tour, en main et dans la défausse (pour la Bannière de protection) ; unités qui ont attaqué
  ou paré ce tour, sorts lancés ce tour par couleur, ennemis vaincus ce tour (déjà dans les faits du tour, spec 049 : à lire là
  plutôt que dupliquer si possible).
- **`Effect`** : `BannerAssigned` (base des bannières, jouée par `AssignBanner`), `TokenCrystalAndFame(couleur)`, `SwordDiscard`,
  `HornOfWrath { extra: u8 }`, `InfluenceAndFame { amount, fame }`, `GemPouchDice`, `TokensEachColorAndTime`, `HealWithFame(u8)`,
  `HealAndDraw(u8)`, `SpellAndCrystalFromOffer`, `AmuletOfSun { tokens }`, `AmuletOfDarkness { tokens }`, `CancelAttacks(3)`,
  `ReadyAllUnits`, et les modificateurs ci-dessus ; `TrainFromOffer(Hand)` existant (Livre de sagesse).
- **`RngUsage::ArtifactDice`** : flux nouveau (ADR 0007).
- **Contenu** : les seize artefacts quittent `Unsupported` dans `artifacts.ron` ; `Unsupported` ne sert plus à aucune carte.

## Actions et événements

| Action | Validation (refusée si…) | Événements produits |
|---|---|---|
| `PlayCard` (existante, `ThrowAway` nouveau) | règle « effet utilisable » ; bannière jouée autrement que par `AssignBanner` ou de côté ; Épée de base hors phase d'attaque ou défaussant une Blessure ou elle-même ; Bannière de peur renforcée : plus de 3 ennemis, ennemi vaincu, paré ou qui n'attaque plus ; Cor : bonus > 5 ; Livre : pas de carte action à détruire, ou rien de sa couleur dans l'offre ; Bannière de courage renforcée ou soin en combat | `CardPlayed` ; `CardDestroyed` (artefact renforcé, carte détruite par le Livre) ; `FameGained` ; `CrystalGained`, `ManaTokenGained` ; `WoundFromCard` ; `ArtifactDiceRolled` (nouveau) ; `OfferCardGained` (`to: Hand`) puis `OfferCardDealt` ; `UnitReadied` ; `RuinTokenRevealed`, `EnemyTokenRevealed` (existants, Amulette du soleil) |
| `AssignBanner { card, unit }` (nouvelle) | carte absente de la main ou pas une bannière ; unité absente | `BannerAssigned` (nouveau), et `BannerLost` pour la bannière remplacée |
| `UseBanner { unit, enemy: Option<EnemyToken> }` (nouvelle) | Terreur : hors phase de parade, unité épuisée ou blessée, ennemi absent, vaincu, paré, n'attaquant plus, Invocateur ou créature invoquée ; Courage : en combat, bannière déjà retournée, unité disponible | `BannerUsed` (nouveau ; Terreur : unité épuisée, `DoesNotAttack`, `FameGained` ; Courage : bannière retournée, `UnitReadied`) |
| `ChooseCrystalColor` (existante, spec 044) | en plus : dé or de la Poche de mana en attente | `CrystalGained` |
| `ReclaimBanners { banners: Vec<CardId> }` (nouvelle) | pas de choix de fin de manche en attente ; carte qui n'est pas une bannière assignée | `BannersReclaimed` (nouveau) |
| `ActivateUnit` (existante) | inchangée | en plus, Bannière de gloire : +1 aux points, `FameGained` |
| `EndTurn` (existante) | inchangée | en plus, avec `ThrowAwayReceivedWounds` : `WoundsThrownAway` (nouveau) d'abord |

| Événement | Effet de `apply` | Révèle une information ? | Point de non-retour ? |
|---|---|---|---|
| `BannerAssigned { card, unit }` | la carte quitte la main pour l'unité | non | non |
| `BannerLost { card, unit }` | la bannière va dans la défausse (unité détruite, renvoyée, autre bannière) | non | non |
| `BannerUsed { unit }` | unité épuisée (Terreur) ou bannière retournée (Courage) | non | non |
| `ArtifactDiceRolled { faces }` | les faces lancées (Wounds et cristaux suivent dans leurs événements) | non | **oui** (hasard) |
| `WoundsThrownAway { hand, discard }` | Blessures retirées de la main et de la défausse | non | non (déjà après `TurnEnded`) |
| `BannersReclaimed { banners }` | les bannières quittent leurs unités pour le deck, avant son mélange ; les autres bannières se remettent face visible | non | non |

À la fin de la manche (`RoundEnded`, spec 023), si une bannière est assignée, le choix des bannières à reprendre attend
`ReclaimBanners` avant le mélange du deck (Q2) ; sans bannière assignée, rien ne change.

## Comportements attendus

### Contenu et règles communes

- **C1** — Plus aucun artefact n'est `Unsupported` ; chacun a ses effets de base et renforcé (`all_artifacts_supported`) —
  contenu.
- **C2** — Bourse sans fond en main, hors combat : effet de base (Influence 4, Gloire +2, carte dans la zone de jeu), effet renforcé
  sans mana (Influence 9, Gloire +3, carte détruite : ni zone de jeu ni défausse ni deck) et de côté sont proposés
  (`artifact_plays_basic_strong_or_sideways`) — MKUE p. 4, 3.c, 3.e. Remplace l'assertion « que de côté » de
  `taken_artifact_plays_sideways` (test modifié, signalé).
- **C3** — Bourse sans fond renforcée seule ce tour : le tour minimal est respecté, la fin du tour ne demande pas de défausse
  (`thrown_away_artifact_counts_as_played`) — Compendium, *Minimum Turn* S2.
- **C4** — Bourse sans fond (Gloire +2) jouée à 2 points du niveau suivant : gloire aussitôt, niveau gagné à la fin du tour
  seulement (`artifact_fame_is_immediate_level_at_end_of_turn`) — Compendium, *Endless Bag of Gold* Q1 ; spec 035.
- **C5** — Concentration en main avec un anneau : l'anneau n'est pas une cible ; Don pour la magie ne propose pas de défausser un
  artefact (`artifacts_are_not_boosted_nor_colored`) — Compendium, *Tome of All Spells* S4.

### Bannières

- **C6** — Bannière de gloire en main, Paysans épuisés et blessés : `AssignBanner` proposé, même en combat ; la carte quitte la main,
  les Paysans la portent ; c'est une carte jouée pour le tour minimal (`banner_assigned_to_any_unit`) — MKUE p. 5, 8 ; Compendium,
  *Minimum Turn* S2.
- **C7** — Bannière de protection assignée aux Paysans qui portent la Bannière de gloire : la Bannière de gloire va dans la
  défausse (`second_banner_discards_the_first`) — MKUE p. 5, 8.a.
- **C8** — Paysans portant une bannière détruits par les dégâts, ou renvoyés pour recruter : la bannière va dans la défausse
  (`banner_discarded_with_its_unit`) — MKUE p. 5, 8.a.
- **C9** — Fin de manche avec deux bannières assignées : le choix est attendu ; une reprise rejoint le deck mélangé, l'autre reste
  sur son unité ; une Bannière de courage retournée se remet face visible (`round_end_reclaims_chosen_banners`) — MKUE p. 4, 2.g ;
  p. 5, 8.b ; Q2.
- **C10** — Décompte final avec une bannière sur une unité et un artefact détruit : Le Collectionneur compte la bannière, pas
  l'artefact détruit (`loot_counts_banners_on_units`) — MKUE p. 15. Étend `final_score_counts_artifacts_and_shields` (spec 041).
- **C11** — Une bannière ne peut pas être jouée pour son effet de base autrement que par `AssignBanner` ; son effet renforcé se joue
  depuis la main, pas depuis l'unité (`banner_strong_only_from_hand`) — Compendium, *Artifact Cards* S1.

### Bannière de gloire

- **C12** — Exemple ci-dessus, phase de parade : les Paysans parent avec Parade 3 contre les Ironclads, Gloire +1 même si la parade
  ne suffit pas ; leur capacité d'Influence n'a pas de +1 (`banner_of_glory_bonus_and_fame`) — carte ; Compendium,
  *Miscellaneous* S12.
- **C13** — Bannière de gloire assignée, dégâts des Prowlers (`enemies.ron` : Attaque physique 4) sur les Paysans (Armure 3 + 1 =
  4) : une Blessure, plus rien à assigner ; sans bannière, 1 point resterait (`banner_of_glory_armor`) — carte ; spec 042.
- **C14** — Combat contre des Prowlers (`enemies.ron` : armure 3, Attaque 4), Paysans et Gardes d'Utem : Bannière de gloire renforcée
  (détruite) en phase de parade ; les Gardes parent (Parade 4 + 1 = 5), puis en phase d'attaque les Paysans attaquent (Attaque 2
  + 1 = 3) : Gloire +2, une par unité ; une unité activée avant la bannière ce tour compte aussi
  (`banner_of_glory_strong_fame_per_unit`) — carte ; Q3.

### Bannière de peur

- **C15** — Paysans disponibles portant la Bannière de peur, phase de parade contre des Swordsmen (`enemies.ron` : Attaque 6) :
  `UseBanner` épuise les Paysans, les Swordsmen n'attaquent pas, Gloire +1 ; pas de dégâts à assigner pour eux
  (`banner_of_fear_cancels_attack`) — carte ; Compendium, *Banner of Fear* S3.
- **C16** — Paysans blessés ou épuisés, phase à distance ou d'attaque, ennemi paré : `UseBanner` pas proposé
  (`banner_of_fear_needs_ready_unwounded_unit`) — Compendium, *Banner of Fear* S4.
- **C17** — Contre des Orc Summoners et leur créature en phase de parade : ni les Summoners ni la créature ne sont une cible
  (`banner_of_fear_not_against_summoners`) — Compendium, *Banner of Fear* Q5, Q6 ; issue #8.
- **C18** — Phase à distance contre Prowlers, Swordsmen, Wolf Riders et Diggers : Bannière de peur renforcée sur trois d'entre eux,
  Diggers (Fortifié) compris : aucun des trois n'attaque ; quatre cibles : refusé (`banner_of_fear_strong_three_enemies`) — carte,
  version *Lost Legion*.

### Bannière de protection

- **C19** — Paysans portant la Bannière de protection, dégâts des Ice Golems (`enemies.ron` : Attaque de glace 2, Paralysant) sur
  eux : résistance à la Glace, l'Armure 4 absorbe les 2 points sans Blessure, l'unité n'est pas détruite ; sans bannière, elle
  serait blessée, donc détruite (`banner_of_protection_resistances`) — carte ; MKUE p. 9, 4.b ; spec 042.
- **C20** — Deux Blessures en main au début du tour, Bannière de protection renforcée, puis deux Blessures reçues des Cursed Hags
  (Toxique : deux de plus dans la défausse) : à la fin du tour, deux Blessures quittent la main et deux la défausse, les deux du
  début restent ; une Blessure soignée entre-temps réduit d'autant celles jetées de la main
  (`banner_of_protection_throws_received_wounds`) — Compendium, *Banner of Protection* Q1 ; Q5.
- **C21** — Héros assommé après la Bannière de protection renforcée : il l'est quand même, et les Blessures reçues sont jetées à
  la fin du tour (`banner_of_protection_does_not_prevent_knockout`) — Compendium, *Banner of Protection* S2.

### Bannière de courage

- **C22** — Paysans épuisés et blessés portant la Bannière de courage, hors combat : `UseBanner` les rend disponibles (blessés),
  bannière retournée ; plus proposé cette manche, en combat jamais ; face visible à la manche suivante
  (`banner_of_courage_once_per_round`) — carte ; Compendium, *Banner of Courage* S1.
- **C23** — Trois unités épuisées, Bannière de courage renforcée hors combat : toutes disponibles ; en combat, pas proposée
  (`banner_of_courage_strong_readies_all`) — carte.

### Anneaux

- **C24** — Anneau de rubis de base : un jeton rouge, un cristal rouge, Gloire +1 ; trois cristaux rouges déjà : deux jetons
  (`ring_token_crystal_and_fame`) — carte ; MKUE p. 5, « Gain » 2.
- **C25** — De nuit, phase à distance : Éclair de feu (`spells.ron`, rouge) lancé de base avec un jeton rouge ; puis Anneau de rubis
  renforcé (Gloire +1 pour Éclair de feu) ; en phase d'attaque, Mur de flammes (rouge) lancé renforcé (Déluge de flammes) en payant
  rouge et noir par le mana illimité, sans dé ni jeton : Gloire +1 de plus (`ruby_ring_endless_mana_and_fame`) — carte ; Q3.
- **C26** — De jour, Anneau de saphir renforcé : le mana noir illimité n'est pas proposé ; avec l'Amulette des ténèbres jouée ce
  tour, il l'est (`ring_black_mana_follows_night_rules`) — Compendium, *Rings* S1.

### Épée de justice

- **C27** — Phase d'attaque contre des Prowlers (armure 3) : Épée de justice de base en défaussant Marche et Promesse : Attaque 6,
  une seule attaque ; Blessure ou l'Épée elle-même : pas défaussables ; Prowlers vaincus : Gloire 2 + 1
  (`sword_of_justice_discards_for_attack`) — carte ; Compendium, *Sword of Justice* Q4, S8.
- **C28** — Phase à distance : Épée de base pas proposée, sauf siège des Mages d'Altem (C30b) (`sword_of_justice_basic_only_in_attack_phase`) — Compendium, *Sword of
  Justice* S2.
- **C29** — Exemple ci-dessus : Épée renforcée, Rage compte 4 contre les Ironclads sans résistance physique ; Éclair de feu
  (Attaque de feu) n'est pas doublé (`sword_of_justice_doubles_physical`) — carte ; Compendium, *Sword of Justice* S1, S7.
- **C30** — Épée renforcée en phase à distance : la résistance physique est perdue aussitôt, le doublement n'agit qu'en phase
  d'attaque, sur une attaque d'unité aussi (`sword_of_justice_strong_timing`) — Compendium, *Sword of Justice* S1, S3.
- **C30b** — Épée et Mages d'Altem (issue privée 272) : renforcée puis feu glacé, Rage compte feu glacé 4
  (`sword_of_justice_doubled_attack_becomes_cold_fire`) ; renforcée et siège, Rage en phase à distance compte physique 2
  (`sword_of_justice_strong_not_doubled_with_altem_siege`) ; base en feu glacé, une carte défaussée donne feu glacé 3
  (`sword_of_justice_basic_becomes_cold_fire`) ; base de siège, jouée en phase à distance (`sword_of_justice_basic_becomes_siege`)
  — Compendium, *Sword of Justice* S2, S6.

### Corne de la colère

- **C31** — Phase à distance contre des Diggers (Fortifié, armure 3) : Corne de la colère de base : Attaque de siège 5, Diggers vaincus ; dé
  rouge (graine choisie) : une Blessure en main, comptée pour être assommé (`horn_of_wrath_siege_and_die`) — carte, version *Lost
  Legion* ; Compendium, *Horn of Wrath* S1.
- **C32** — Cor renforcé avec +3 : Attaque de siège 8, trois dés lancés après le choix, une Blessure par noir ou rouge ; +6 refusé ;
  le lancer est un point de non-retour (`horn_of_wrath_strong_extra_dice`) — carte ; Compendium, *Horn of Wrath* S2 ; spec 024.

### Poche de mana et Graal doré

- **C33** — Poche de mana de base, dés vert et or : un cristal vert, puis le choix de la couleur pour l'or ; dés noir et bleu : Gloire
  +1 et un cristal bleu (`gem_pouch_rolls_two_dice`) — carte.
- **C34** — Poche de mana renforcé : quatre jetons de base et un or de jour, un noir de nuit
  (`gem_pouch_strong_tokens_by_time`) — carte ; Compendium, *Endless Gem Pouch* S1.
- **C35** — Une Blessure en main, Herboristes blessés (niveau 1) : Graal doré de base : la Blessure de la main est jetée (Gloire +1),
  le point restant soigne les Herboristes (Gloire +1) ; un second Soin ensuite ne rapporte plus (`golden_grail_fame_per_point`) —
  carte ; Compendium, *Golden Grail* S1 ; Q7.
- **C36** — Trois Blessures en main, deck de deux cartes : Graal doré renforcé : trois Blessures jetées, deux cartes piochées (le
  deck est vide) (`golden_grail_strong_draws_per_wound`) — carte. En combat : pas proposée.

### Livre de sagesse et amulettes

- **C37** — Offre d'actions avancées avec Coup de sang (rouge) : Livre de sagesse de base en détruisant Rage (rouge) : Coup de sang en
  main, l'offre se remplit ; Marche (verte) sans action avancée verte dans l'offre : pas proposée
  (`book_of_wisdom_gains_advanced_action`) — carte ; spec 056.
- **C38** — Offre de sorts avec Éclair de feu : Livre renforcé en détruisant Rage : Éclair de feu en main et un cristal rouge
  (`book_of_wisdom_strong_gains_spell_and_crystal`) — carte.
- **C39** — De nuit, héros sur le portail `(0, 0)` : Amulette du soleil : un jeton or utilisable pour payer un effet bleu ; la forêt
  `(1, -1)` coûte 3 au lieu de 5 ; le dé or de la Source peut être pris ; un jeton de ruines face cachée du plateau est retourné
  (`amulet_of_sun_day_rules_at_night`) — carte ; Compendium, *Amulet of the Sun* S1 ; Q6.
- **C40** — De jour, Amulette des ténèbres (jeton noir choisi) : un désert coûte 3 ; un sort se lance renforcé avec le jeton noir ;
  le dé noir de la Source peut être pris ; l'or sert encore (`amulet_of_darkness_night_mana_by_day`) — carte ; Compendium,
  *Amulet of Darkness* S1 ; Q10.
- **C41** — Amulette des ténèbres renforcée : trois jetons de couleurs choisies (rouge, noir, or) ; Amulette du soleil renforcée de jour :
  trois jetons or, aucun effet de nuit (`amulets_strong_three_tokens`) — carte.

### Vue et persistance

- **C42** — La vue donne la bannière de chaque unité et si elle est retournée, les modificateurs nouveaux, le dé or du Sac de
  gemmes en attente et le choix des bannières à reprendre (`view_shows_banners_and_artifact_states`) — spec 052.
- **C43** — Sauvegarde avec une bannière assignée et retournée, et l'Anneau de rubis renforcé en cours, rechargée : tout est
  rétabli ; retour en arrière jusqu'avant l'assignation : la bannière revient en main ; pas avant un lancer de dé
  (`banners_follow_reload_and_undo`) — spec 024.

## Cas limites et refus

- **L1** — `AssignBanner` sans unité, avec une carte qui n'est pas une bannière, ou une bannière absente de la main : refusé.
- **L2** — `UseBanner` sur une unité sans bannière, ou dont la bannière n'a pas de capacité (gloire, protection) : refusé.
- **L3** — Bannière de peur renforcée sans ennemi qui attaque encore : pas proposée ; deux fois le même ennemi : refusé.
- **L4** — Livre de sagesse sans carte action en main, ou sans offre de la couleur : pas proposé (renforcé : offre de sorts).
- **L5** — `ReclaimBanners` sans choix en attente : refusé ; tant qu'il attend, seule cette action est légale.
- **L6** — Amulette du soleil de jour : jeton(s) or seulement, pas de `DayRules` ; Amulette des ténèbres de nuit : jeton(s)
  seulement, pas de `NightMana`.
- **L7** — Combat sans unités (donjon, tombeau, monastère incendié, spec 041, 044) : `UseBanner` pas proposé ; les bonus de
  bannière n'y servent pas.

## Information cachée

Rien de nouveau de caché : les bannières assignées, retournées, et les modificateurs sont publics ; les faces des dés lancés sont
dans les événements et la vue. Champs ajoutés à la vue : `banner` et `banner_flipped` des unités, le dé or de la Poche de mana en
attente (comme `PlayerView::reward_die`), le choix des bannières à reprendre (`PlayerView`, C42) ; les modificateurs passent par
`PlayerView::modifiers` (spec 052).

## Libellés (i18n)

Les textes `card.<artefact>.name`, `basic_text` et `strong_text` existent (relus dans `assets/i18n/fr.json`, ex.
`card.banner_of_fear.strong_text` « Jusqu'à 3 ennemis n'attaquent pas ce combat » ; `card.horn_of_wrath.basic_text` « Attaque de
siège 5 ; lancer un dé de mana : noir ou rouge, une Blessure »). Réutilisé : `effect.fame` « Gloire +{amount} » (fr « Gloire +2 »).
Nouvelles clés (fr ; en à l'avenant) :

- `play.throw_away` : « Détruire : {effect} » (fr « Détruire : Influence 9, Gloire +3 ») ; `play_area.line_thrown_away` : « {card}
  (détruit) : {effect} » ; `play.assign_banner` : « Assigner à une unité ».
- `banner.choose_unit` : « {card} : à quelle unité l'assigner ? » ; `banner.on_unit` : « Bannière : {card} » (infobulle de l'unité) ;
  `banner.flipped` : « retournée jusqu'à la prochaine manche » ; `banner.use_fear` : « Annuler une attaque (Gloire +1) » ;
  `banner.use_courage` : « Rendre disponible (une fois par manche) ».
- `banner.reclaim_title` : « Fin de manche : bannières à reprendre dans le deck » ; `banner.reclaim_keep` : « Garder sur l'unité » ;
  `banner.reclaim_take` : « Reprendre ».
- `modifier.unit_armor_bonus` : « unités Armure +{amount} » (fr « unités Armure +1 ») ; `modifier.fame_per_unit` : « Gloire +1 par
  unité qui attaque ou pare » ; `modifier.endless_mana` : « mana {color} et noir illimité ; Gloire +1 par sort {color} » (`{color}`
  repris de `effect.gain_spell`, relu) ; `modifier.fame_per_enemy` : « Gloire +1 par ennemi vaincu » ; `modifier.double_physical` :
  « Attaques physiques doublées en phase d'attaque » ; `modifier.throw_away_wounds` : « fin du tour : jeter les Blessures reçues » ;
  `modifier.draw_per_wound` : « une carte par Blessure de la main soignée » ; `modifier.day_rules` : « règles de jour (forêts 3, or,
  révélations) » ; `modifier.night_mana` : « mana noir comme de nuit, déserts 3 » ; `modifier.grail_fame` : « Gloire +1 par point de
  soin ({left} restants) ».
- `horn.extra` : « Corne de la colère : ajouter combien ? (un dé par point) » ; `sword.discard_prompt` : « Épée de justice : cartes à
  défausser (Attaque 3 chacune) » ; `mana_choice.endless` : « illimité ({color}) ».
- `artifact_dice.rolled` : « Dés lancés : {faces} » (zone de jeu).

## Interface

- **Artefact en main** : boutons Jouer (base), « Détruire : … » (renforcé, sans choix de mana) et de côté, comme les autres cartes
  (spec 073) (C2, actions légales `PlayCard` avec `ThrowAway`).
- **Bannière en main** : bouton « Assigner à une unité », puis clic sur une unité du panneau du bas (Échap annule) (C6, C7,
  `AssignBanner`).
- **Unité porteuse** : la bannière dépasse sous la carte de l'unité, son nom en infobulle (`banner.on_unit`), grisée si retournée
  (C22, `PlayerView` `banner`, `banner_flipped`) ; placement : vérification à l'œil.
- **Capacités de bannière** : Bannière de peur, bouton « Annuler une attaque » sur l'unité en phase de parade, puis l'ennemi coché
  sur l'écran de combat (C15, C16, C17) ; Bannière de courage, bouton « Rendre disponible » hors combat (C22) ; actions légales
  `UseBanner`.
- **Choix des cartes** : Épée de justice de base, cartes à défausser cochées dans la main (C27) ; Bannière de peur renforcée,
  ennemis cochés comme pour Ailes de la nuit (C18) ; Cor renforcé, boutons +0 à +5 (C32) ; Livre de sagesse, carte à détruire comme
  Entraînement puis carte de l'offre (C37, C38) ; Amulette des ténèbres, couleurs par les boutons d'options (spec 073) (C40, C41).
- **Dé or de la Poche de mana** : choix de la couleur comme le dé de récompense or (spec 044) (C33, `PlayerView`).
- **Mana illimité** : une source « illimité (rouge) » et « illimité (noir) » dans le choix du mana, sans dé ni jeton (C25, C26,
  `ManaPayment::Endless` dans les actions légales).
- **Fin de manche** : fenêtre des bannières assignées, « Garder » ou « Reprendre » pour chacune (C9, `PlayerView`, choix en
  attente).
- **Effets en cours** : les modificateurs nouveaux dans la zone de jeu (C42) ; les dés lancés en ligne de la zone de jeu.
- **Vérification à l'œil** : partie préparée `artefacts` (les seize artefacts en main ou à portée, unités avec et sans bannière).

### Raccourcis clavier

Échap annule le choix de l'unité d'une bannière, comme le choix du mana (reporté dans `docs/raccourcis.md`). Rien d'autre : tout
passe par les boutons existants.

## Effets sur les specs déjà implémentées

- **016, 023** : fin du tour qui jette des Blessures ; fin de manche qui attend le choix des bannières et remet face visible la
  Bannière de courage.
- **020, 022, 025** : mana illimité ; or utilisable la nuit, noir le jour, pour le tour.
- **031, 032, 042** : Armure et résistances d'unité changées par une bannière ; capacité d'unité donnée par une bannière ; Attaques
  physiques doublées.
- **014, 017** : coûts de forêt et de désert changés par les amulettes (modificateurs de terrain existants, spec 052).
- **043, 045** : défenseurs et jetons de ruines révélés la nuit par l'Amulette du soleil.
- **041, 063** : Le Collectionneur compte les bannières sur les unités.
- **050** : plus aucune carte `Unsupported` ; le test `taken_artifact_plays_sideways` change d'assertion (C2).

## Hors périmètre

- Artefacts des extensions (Bannière de commandement, Tome de tous les sorts, Arc de Starsdawn…) : pas dans le jeu de base.
- Vol d'artefacts (MKUE p. 13) : joueur contre joueur, hors périmètre (`CLAUDE.md`).
- Achat d'artefacts à la cité rouge : spec 062.
- Compétences qui changent avec le jour ou la nuit (Compendium, *Amulet of the Sun* S1) : spec des compétences.

## Plan de développement

Grosse spec (moteur et écran, issue privée 118) : deux fils sur la même branche, plan `plans/055-artefacts.md` rédigé par le fil
d'implémentation :

1. **Moteur** : jouer et détruire un artefact, bannières (assigner, perdre, capacités, fin de manche, décompte), anneaux, Épée, Cor,
   Bourse, Sac, Coupe, Livre, amulettes, contenu, vue, persistance (C1 à C43).
2. **Écran** : libellés, boutons Détruire et Assigner, bannière sous l'unité, choix (Épée, Cor, Terreur, couleurs, dé or),
   mana illimité, fenêtre de fin de manche, partie préparée, vérification à l'œil, docs, bilan.

## Décisions (2026-11-13)

Mathieu accepte les réponses proposées (« ok avec le reste »), appliquées dans le texte ci-dessus :

1. **Q1 — Découpage.** Une seule spec, deux fils (moteur, écran). **Décision : une seule spec.**
2. **Q2 — Bannières en fin de manche.** **Décision : une fenêtre de choix** (garder ou reprendre chaque bannière assignée).
3. **Q3 — Gloire « ce tour ».** **Décision : tout le tour**, avant la carte comprise.
4. **Q4 — Bannière de peur contre les Invocateurs.** Ni l'Invocateur ni sa créature ne sont une cible ; gênant mais pas
   bloquant, **reporté, issue #8** (moment avant le tirage de la créature).
5. **Q5 — Bannière de protection renforcée.** **Décision : toutes les Blessures reçues jetées, sans choix.**
6. **Q6 — Amulette du soleil.** **Décision : jetons de ruines face cachée retournés en la jouant**, ruines révélées ce tour face
   visible, défenseurs voisins révélés, y compris ceux déjà voisins.
7. **Q7 — Graal doré.** **Décision : les points du Graal sont dépensés en premier.**
8. **Q8 — Épée de justice de base sans défausse.** **Décision : permis.**
9. **Q9 — Mot pour l'effet renforcé.** **Décision : « Détruire ».**
10. **Q10 — Dés de la Source avec les amulettes.** **Décision : dé noir (ténèbres) ou or (soleil, la nuit) pris comme dé du tour.**

## Questions ouvertes

Aucune.
