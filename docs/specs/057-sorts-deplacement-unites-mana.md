# 057 — Sorts : déplacement, unités et mana

- Statut : implémentée (2026-11-13) ; validée le 2026-11-12 (questions Q1 à Q8 tranchées : réponses proposées acceptées ; Q3 confirmée par le wiki)
- Jalon : J5
- Dépend de : 054 (lancer un sort, mana noir, sort lancé depuis l'offre, `N'attaque pas` sur un ennemi visé), 052 (moteur
  d'effets, ADR 0008, règle « effet utilisable », case sécurisée), 053 et 056 (`WoundFromCard`, soin et unités rendues
  disponibles, choix en attente), 042 et 050 (unités, offres), 043 (assaut, retraite), 014, 015, 017, 030 et 036 (déplacement,
  exploration, cases atteignables, provocation), 016 et 023 (fin du tour, fin de manche), 020 à 025 (Source, cristaux, jetons),
  024 et 038 (retour en arrière, générateurs par usage) ; toutes implémentées
- Livret : MKUE p. 4 (*Deed Cards* 3.b ; *Round* 3.b, 4.e) ; p. 5 (*"Composite" Effects* 1.c ; *"Gain" Effects* 2 ; *Units* 6) ;
  p. 6 (*A Player's Turn* 5.b, 5.c, 6) ; p. 7 (*Movement* 4, 5.a, 7.c, 7.d) ; p. 8 (assaut) ; p. 11 (*End of the Turn* 1 à 7) ;
  p. 15 (*Dummy Player*). Texte des cartes : `docs/regles/cartes/sorts.md` (Ailes de la nuit : version *Lost Legion*, qui fait foi) ;
  Distorsion temporelle : texte du wiki (« set this card aside for the rest of the Round », page *Space Bending / Time Bending*) ;
  icônes lues sur les scans `docs/regles/scans/spells/`. Compendium, *Spell Cards* (Call to Arms S1 à S3, Q6, S13, S15 ; Call to
  Glory S1, S2, S5 ; Energy Steal Q1 à S3 ; Mana Meltdown ; Meditation S1 à S4 ; Rebirth Q1, S2 ; Space Bending Q1 à S7 ; Time
  Bending S1 à S7, S10, S13 ; Trance S1 ; Underground Attack S1 à S3 ; Underground Travel S1 à S4 ; Wings of Night S1 ; Wings of
  Wind S1 à S5), *Movement* S2, S5, S8, O9, *Dummy Player* S2
- ADR : [0007 Un générateur par usage](../decisions/0007-generateurs-par-usage.md) (usage nouveau), [0008 Moteur
  d'effets](../decisions/0008-moteur-d-effets.md) (modificateurs nouveaux, durée « manche »)
- Crates touchées : arcanum-core (règles, contenu), arcanum-ui, arcanum-persistance (parties préparées), `assets/i18n/`

## Objectif

Les **dix derniers sorts** du jeu de base font ce qu'ils disent : se déplacer sous terre ou en volant sans provoquer, sauter
une case, rejouer un tour, se soigner, rendre des unités disponibles, utiliser ou recruter une unité de l'offre, remettre des
cartes de la défausse sur le deck, garder un dé de la Source, gagner des cristaux. Ce qui ne vise que les autres joueurs ne fait
rien en solo. Don pour la magie de base lance désormais **tous** les sorts de l'offre.

C'est le second lot de sorts (spec 054, Q1). Après lui, plus aucun sort n'est `Unsupported`.

| Sort (fr.json) | Couleur | Icône | Base | Renforcé | Ce qui est nouveau |
|---|---|---|---|---|---|
| Voyage souterrain / Attaque souterraine (*Underground Travel*) | vert | déplacement | jusqu'à 3 cases, ni marais ni lac, case sécurisée, gratuit, sans provoquer | idem jusqu'à un site fortifié : assaut, fortifications du site ignorées, retraite au point de départ | déplacement par un effet ; assaut à distance |
| Ailes du vent / Ailes de la nuit (*Wings of Wind*) | blanc | déplacement / combat | 1 à 5 points de Mouvement, autant de cases, case sécurisée, sans provoquer | (*Lost Legion*) un ennemi n'attaque pas ; 1 Mouvement le 2e, 2 le 3e… | Mouvement en combat |
| Restauration / Renaissance (*Restoration*) | vert | soin | Soin 3, 5 en forêt | idem ; rendre disponibles des unités, 3 niveaux en tout (5 en forêt) | terrain de la case du héros ; plusieurs unités |
| Distorsion spatiale / temporelle (*Space Bending*) | bleu | déplacement / spécial | ce tour, les cases à 2 de distance comptent comme adjacentes ; aucune provocation | en fin de tour, les cartes jouées reviennent en main, pas de pioche, un autre tour | voisinage à 2 ; tour supplémentaire ; carte écartée |
| Appel aux armes / Appel à la gloire (*Call to Arms*) | blanc | spécial | capacité d'une unité de l'offre ce tour | recruter gratuitement une unité de l'offre | unité de l'offre activée ; recrutement sans interaction |
| Méditation / Transe (*Meditation*) | vert | spécial | deux cartes au hasard de la défausse, sur ou sous le deck ; 2 cartes de plus à la pioche | idem, cartes choisies | hasard nouveau ; limite de main augmentée |
| Fuite de mana / Brûlure de mana (*Mana Meltdown*) | rouge | spécial | (autres joueurs : rien en solo) | une couleur : une Blessure par cristal de cette couleur, puis 2 cristaux | effet sans objet en solo |
| Puits de mana / Malédiction de mana (*Mana Claim*) | bleu | spécial | garder un dé de la Source jusqu'à la fin de la manche : 3 jetons ce tour, ou 1 par tour | idem (malédiction des autres : rien en solo) | dé retiré de la Source ; effet de manche |
| Lecture d'esprit / Vol d'esprit (*Mind Read*) | blanc | spécial | une couleur : son cristal (autres joueurs : rien en solo) | idem | — |
| Flux d'énergie / Vol d'énergie (*Energy Flow*) | vert | soin | rendre disponible une unité (autres joueurs : rien en solo) | idem, et elle est soignée | unité de tout niveau |

Les noms Lecture d'esprit et Vol d'esprit restent des propositions (glossaire, issue privée 105) ; `PLAN.md` disait « Lecture des
pensées », corrigé dans cette PR d'après le glossaire.

## Règles

### Règles communes

- **Lancer un sort** : comme la spec 054 (mana de la couleur ; renforcé avec un mana noir, de nuit ou dans un donjon ou un tombeau).
- **Icône et moment** (MKUE p. 6, 5.b, 5.c ; icônes lues sur les scans) : un effet **spécial** se joue à tout moment du tour, combat
  compris ; un effet de **soin** à tout moment hors combat ; un effet de **déplacement** se joue à tout moment, mais ne déplace la
  figurine ou n'explore qu'en **phase de déplacement** (avant l'action du tour, `movement_ended` faux), et jamais en combat
  (Compendium, *Movement* S2) ; Ailes de la nuit, icône de combat, en combat seulement. Avec la règle « effet utilisable » (spec 052),
  un effet de déplacement n'est donc proposé qu'en phase de déplacement.
- **Ce qui vise les autres joueurs** (cartes) : rien en solo. Le joueur fantôme (spec 061) n'est pas un joueur visé : ses
  cristaux ne sont que des marqueurs (Compendium, *Dummy Player* S2). Un effet qui ne fait rien d'autre n'est **pas proposé**, la
  carte se joue de côté (Fuite de mana de base, Q2).
- **Se déplacer par un effet** (MKUE p. 7, 7.c, 7.d ; Compendium, *Movement* S5) : Voyage souterrain, Ailes du vent. Aucun coût
  de terrain, on passe sur les cases infranchissables (montagne, site fortifié non conquis, ennemi errant) sauf ce que la carte
  interdit, sur des cases **révélées** seulement ; l'arrivée est une **case sécurisée** (`is_safe_space`, spec 052). Aucune
  provocation. Les cases traversées ne révèlent rien ; la case d'arrivée révèle ce qu'une arrivée ordinaire révèle (défenseurs
  d'une forteresse ou d'une tour de mage voisine de jour, jeton de ruines de nuit ; spec 043, 045) (Compendium, *Movement* O9, Q7).
  Le déplacement se fait d'un coup, vers une case choisie en jouant la carte ; avant et après, le joueur se déplace normalement
  (Compendium, *Underground Travel* S3).

### Voyage souterrain et Attaque souterraine

- **Base** (carte ; Compendium, *Underground Travel* S1 à S4) : jusqu'à **3 cases** révélées, sans dépenser de points de
  Mouvement ; ni l'arrivée ni une case traversée n'est un marais ou un lac (le littoral, `Terrain::Sea`, compte comme un lac) ; les
  montagnes, sites et ennemis se traversent. Distance comptée sur le plus court chemin qui respecte ces interdits.
- **Renforcé** (carte ; Compendium, *Underground Attack* S1 à S3) : mêmes cases, mais l'arrivée est un **site fortifié non
  conquis** (forteresse, tour de mage ; les cités attendent la spec 062). Le déplacement s'arrête, c'est un **assaut** (spec 043 :
  réputation −1, défenseurs révélés et combattus), avec `IgnoreSiteFortifications` (spec 054) pour ce combat. En cas de
  **retraite**, le héros revient à sa case de départ (`Assault::from` = case d'où le sort a été joué). Vers un site déjà conquis,
  c'est la base qui sert (S3).

### Ailes du vent et Ailes de la nuit

- **Base** (carte ; Compendium, *Wings of Wind* S2 à S5) : dépenser de **1 à 5 points de Mouvement** de la réserve, déjà gagnés,
  une case par point, sur des cases révélées quelles qu'elles soient (lac, montagne, site, ennemi) ; arrivée sur une case sécurisée.
  Le nombre de points dépensés est la distance de la case choisie (plus court chemin sur les cases révélées). On n'explore pas
  avec ces points (S2).
- **Renforcé, version *Lost Legion*** (carte ; Compendium, *Wings of Night* S1) : l'ennemi visé `DoesNotAttack` ; d'autres
  ennemis aussi, contre 1 point de Mouvement pour le 2e, 2 pour le 3e, etc. (4 ennemis : 0 + 1 + 2 + 3 = 6 points). Mêmes moments
  et mêmes cibles que Tourbillon (spec 054) : phase à distance ou de parade, ennemis qui attaquent encore. Blanc : aucune
  résistance ne l'arrête.
- **Mouvement en combat** (MKUE p. 6, 5.c ; Compendium, *Movement* S2) : les effets de Mouvement (cartes, de côté compris, et
  unités) se jouent en combat quand ils servent, c'est-à-dire quand Ailes de la nuit peut être lancé (en main ou depuis l'offre
  avec Don pour la magie, mana payable) (Q4). Leurs points vont dans la réserve de Mouvement, que paie Ailes de la nuit.

### Restauration et Renaissance

- **Base** (carte) : Soin 3, ou Soin 5 si la case du héros est une **forêt** (terrain de la case, site compris). Les points vont dans
  la réserve de soin (spec 042). Icône de soin : pas en combat.
- **Renforcé** (carte ; Compendium, *Rebirth* Q1, S2) : le même Soin, et rendre disponibles des unités **épuisées** dont les niveaux
  font au plus 3 en tout (5 en forêt), choisies en jouant la carte et toutes rendues disponibles aussitôt. Une unité épuisée et
  blessée redevient disponible et reste blessée (spec 042).

### Distorsion spatiale et Distorsion temporelle

- **Base** (carte ; Compendium, *Space Bending* Q1 à S7) : pose `SpaceBending` pour le tour. Les cases à **distance 2** comptent comme
  voisines pour se déplacer et pour explorer ; la case sautée est ignorée (elle peut être un lac, une montagne, un ennemi, un site).
  Entrer sur la case coûte son coût de terrain (S3) ; une case infranchissable, un ennemi errant ou un site fortifié s'y traitent
  comme sur une case voisine (assaut compris). Explorer coûte 2, depuis une case à distance 2 au plus d'une case de l'emplacement
  (S2). **Aucun déplacement** du tour ne provoque, même un pas ordinaire. Un assaut repoussé ramène sur la case d'où l'on a
  sauté (S5). Les défenseurs d'une forteresse ou d'une tour de mage ne sont révélés, de jour, que par une case vraiment voisine
  (S7). Cases atteignables et trajet en un clic (specs 017, 036) en tiennent compte.
- **Renforcé** (carte ; MKUE p. 5, *"Composite" Effects* 1.c ; Compendium, *Time Bending* S1 à S7, S10, S13) : à la fin du tour,
  après les étapes ordinaires (dés rendus, retraite forcée, mine et clairière, récompenses, niveaux ; MKUE p. 11, 1 à 6), les cartes
  de la zone de jeu **reviennent en main** au lieu d'aller dans la défausse (pas celles défaussées ou détruites) ; **pas de
  pioche** ; puis le joueur joue aussitôt un **tour supplémentaire**. Distorsion temporelle est **écartée** (Don pour la magie,
  s'il l'a lancée depuis l'offre, l'est à sa place) jusqu'à la fin de la manche (« for the rest of the Round », texte du wiki), où elle revient dans le deck (Q3). Les Blessures en
  main y restent, les unités restent épuisées. Pendant un tour supplémentaire, Distorsion temporelle n'est pas proposée (S13).
  Jouable même après l'annonce de la fin de la manche ou au dernier tour du scénario : le tour supplémentaire a lieu (S2, S10).

### Appel aux armes et Appel à la gloire

- **Base** (carte ; Compendium, *Call to Arms* S1 à S3, S13, S15) : pose `UnitFromOffer` pour le tour. Une fois ce tour, le joueur
  active **une capacité d'une unité de l'offre d'unités** comme une de ses unités (mêmes moments, mana à payer, bonus d'unités
  comme Au cœur du combat) ; l'unité reste dans l'offre et ne reçoit jamais de dégâts. Pas une action avancée de l'offre
  (monastère) ; pas dans un combat sans unités (donjon, tombeau, monastère incendié : specs 041, 044). Les capacités qui blessent
  l'unité (Guerriers d'Utem, `Wounding`) ne sont pas proposées : simplification choisie par Mathieu, le compendium les permet
  (*Call to Arms* Q6, *Utem Swordsmen* S1 : l'unité reste blessée dans l'offre) (Q6).
- **Renforcé** (carte ; Compendium, *Call to Glory* S1, S2, S5) : **recruter gratuitement** une unité de l'offre, partout et à tout
  moment, combat compris, sans interaction ni restriction de lieu ; au maximum de jetons de commandement, une unité est d'abord
  renvoyée (comme `RecruitUnit`, spec 042). L'unité arrive disponible ; elle ne sert pas dans un combat sans unités. Elle ne compte
  pas comme recrutée par Influence (Récit héroïque, spec 053 Q5).

### Méditation et Transe

- **Base** (carte ; Compendium, *Meditation* S2, S4) : **deux cartes tirées au hasard** dans la défausse (moins s'il y en a moins ;
  jouable défausse vide) ; le joueur les voit, puis les met **toutes deux** sur le deck ou **toutes deux** dessous, dans l'ordre
  qu'il choisit (choix en attente, comme la spec 056). Le tirage est un **point de non-retour**, par un générateur nouveau
  `RngUsage::DiscardPick(joueur)` (ADR 0007 : les tirages des parcours ne bougent pas).
- **Renforcé** (carte) : les deux cartes sont **choisies**.
- **Deux cartes de plus** (carte ; Compendium, *Meditation* S1, O3 ; *Trance* S1) : pose `DrawBonus(2)`, qui dure jusqu'à la
  prochaine pioche du joueur : la fin du tour pioche jusqu'à la limite de main + 2. Si la manche finit d'abord, c'est la pioche de
  la nouvelle manche qui en profite ; après Distorsion temporelle (pas de pioche), la suivante. Deux Méditations : + 4 (O3).

### Fuite de mana et Brûlure de mana

- **Base** (carte) : ne vise que les autres joueurs, rien en solo : pas proposée, de côté seulement (Q2).
- **Renforcé** (carte ; MKUE p. 5, *"Gain" Effects* 2) : le joueur choisit une couleur de base ; il reçoit **une Blessure en main
  par cristal** de cette couleur qu'il possède (`WoundFromCard`, comme Tempête de feu : elles comptent pour être assommé en combat),
  puis gagne **deux cristaux** de cette couleur (au-delà de 3, des jetons).

### Puits de mana et Malédiction de mana

- **Base** (carte) : le joueur prend un **dé de couleur de base** de la Source (ni pris ce tour, ni mis sur une couleur par une
  autre carte) et le garde jusqu'à la **fin de la manche** : il n'est plus dans la Source, n'est pas relancé en fin de tour et ne
  sert pas de mana. Deux options : **3 jetons** de sa couleur aussitôt, ou **1 jeton** de sa couleur au début de chacun des tours
  suivants de la manche (`ManaClaimPerTurn`, durée la manche). Prendre ce dé ne compte pas comme le dé du tour (Q7). À la fin de la
  manche, il revient dans la Source (relancée pour la nouvelle manche, spec 023).
- **Renforcé** (carte) : la malédiction ne vise que les autres joueurs : en solo, le même effet que la base.

### Lecture d'esprit et Vol d'esprit

- **Base et renforcé** (carte) : en solo, choisir une couleur de base et gagner un **cristal** de cette couleur (au-delà de 3, un
  jeton). Les deux effets sont identiques en solo.

### Flux d'énergie et Vol d'énergie

- **Base** (carte) : rendre disponible une unité **épuisée** de tout niveau. Icône de soin : pas en combat (Compendium, *Energy
  Steal* Q1) (Q5).
- **Renforcé** (carte ; Compendium, *Energy Steal* S2, S3) : rendre disponible une unité épuisée et lui retirer **une** Blessure.
  Une unité disponible et blessée n'est pas une cible.

### Don pour la magie

- **Sorts de l'offre** (spec 054) : les sorts de ce lot se lancent depuis l'offre comme les sorts de combat (fin de la limite L4 de la
  spec 054). Distorsion temporelle lancée depuis l'offre : Don pour la magie est écarté à sa place, le sort reste dans l'offre
  (MKUE p. 5, *"Composite" Effects* 1.c).

### Exemple : Voyage souterrain sans provoquer

Bac à sable (`docs/reference/bac-a-sable.md`), de jour : le héros est en `(2, -2)` (plaine) ; des orques maraudeurs sont en `(3, -3)`.
Aller au village en `(2, -3)` à pied coûte 2 et provoque les orques (les deux cases touchent `(3, -3)`) : le déplacement s'arrête et
le combat suit. Avec Voyage souterrain (un cristal vert), le héros arrive au village sans dépenser de point ni provoquer, et peut
encore y interagir (son action du tour).

## Modèle

Termes du glossaire : Sort, Lancer, Case sécurisée, Provoquer, Assaut, Retraite, Offre d'unités, Unité disponible / épuisée, Soin,
Défausse, Deck, Limite de main, Source, Cristal, Jeton de mana. Termes ajoutés au glossaire dans cette PR : **Phase de déplacement**
(*Movement phase*, `movement_ended`), **Tour supplémentaire** (*Time Bent turn*, `extra_turn`), **Écarter (une carte)** (*set
aside*, `set_aside` ; « mettre de côté » se confondrait avec « jouer de côté »), **Dé gardé** (*kept die*, Puits de mana).

Données nouvelles (noms indicatifs, forme dans l'ADR 0008) :

- **`CardPlay`** : `destination: Option<Hex>` (Voyage souterrain, Ailes du vent) ; `units: Vec<UnitCard>` (Renaissance, Flux
  d'énergie) ; `enemies: Vec<EnemyToken>` (Ailes de la nuit, à la place de `enemy` pour ce sort) ; `color` et `die` (Brûlure de mana,
  Lecture d'esprit, Puits de mana). `serde(default)`.
- **`ModifierKind`** : `SpaceBending` (tour) ; `UnitFromOffer` (tour, une fois) ; `DrawBonus(u8)` (jusqu'à la prochaine pioche) ;
  `ManaClaimPerTurn(BasicColor)` (manche) ; `TimeBending` (fin du tour). **`ModifierDuration::Round`** (nouvelle) et une durée
  « jusqu'à la prochaine pioche ».
- **`PlayerState`** : `set_aside: Vec<CardId>` (rendues au deck à la fin de la manche), `extra_turn: bool`, `kept_die: Option<u8>`
  (indice du dé gardé, hors de la Source jusqu'à la fin de la manche).
- **`Effect`** : `UndergroundMove { assault: bool }`, `FlyMove { max: u8 }`, `DoesNotAttackPerMove` (Ailes de la nuit),
  `HealByTerrain { heal, forest }`, `HealAndReadyLevels { heal, levels, forest_heal, forest_levels }`, `SpaceBending`,
  `TimeBending`, `UnitFromOffer`, `RecruitFromOffer`, `DiscardToDeck { random: bool }`, `ClaimDie`, `WoundsPerCrystal`,
  `ReadyAnyUnit { heal: bool }`, `OthersOnly` (Fuite de mana de base : jamais proposé en solo) ; `GainCrystal` existant pour
  Lecture d'esprit (quatre options).
- **`RngUsage::DiscardPick(PlayerId)`** : flux nouveau (ADR 0007).
- **Contenu** : les dix sorts quittent `Unsupported` dans `spells.ron`.

## Actions et événements

| Action | Validation (refusée si…) | Événements produits |
|---|---|---|
| `PlayCard` (existante) | règles de la spec 054 ; déplacement hors phase de déplacement ou en combat ; destination non révélée, trop loin, non sécurisée, marais ou lac (souterrain), non fortifiée ou conquise (Attaque souterraine), plus de points que la réserve ou 5 (Ailes du vent) ; Ailes de la nuit sans assez de Mouvement pour ses cibles ; soin en combat ; unités non épuisées ou niveaux dépassés ; Distorsion temporelle pendant un tour supplémentaire ; Fuite de mana de base | `CardPlayed` ; `HeroMoved` (nouveau champ `by_effect`, sans provocation) ; `AssaultStarted` (existant) ; `UnitReadied` ; `UnitHealed` ; `WoundFromCard` ; `CrystalGained` ; `ManaTokenGained` ; `DieKept` (nouveau) ; `DiscardPicked` (nouveau, tirage) |
| `PlaceOnDeck { top: bool, order: Vec<CardId> }` (nouvelle) | pas de cartes de Méditation en attente ; ordre qui n'est pas celui des cartes tirées ou choisies | `CardsPlacedOnDeck` (nouveau) |
| `ActivateUnit` (existante) | en plus : unité de l'offre sans `UnitFromOffer`, ou capacité `Wounding` | `UnitActivated` (avec `from_offer`) |
| `RecruitUnit` (existante) | en plus : Appel à la gloire, sans interaction ni Influence | `UnitRecruited` (existant, `free: true`) |
| `EndTurn` (existante) | inchangée | en plus, avec Distorsion temporelle : `CardsReturnedToHand`, `CardSetAside`, `ExtraTurnStarted` (nouveaux), sans `CardDrawn` ; avec `DrawBonus` : la pioche va plus loin |

| Événement | Effet de `apply` | Révèle une information ? | Point de non-retour ? |
|---|---|---|---|
| `HeroMoved` (existant, `by_effect`) | la figurine change de case, sans coût de terrain ni provocation | non | non |
| `DiscardPicked { cards }` (nouveau) | les cartes tirées attendent leur place (choix en attente) | non (la défausse est publique) | **oui** (hasard) |
| `CardsPlacedOnDeck { top, order }` (nouveau) | les cartes passent de la défausse au dessus ou au-dessous du deck | non | non |
| `DieKept { die }` (nouveau) | le dé quitte la Source jusqu'à la fin de la manche | non | non |
| `CardsReturnedToHand`, `CardSetAside` (nouveaux) | cartes de la zone de jeu en main ; carte écartée | non | non (déjà après `TurnEnded`) |
| `ExtraTurnStarted` (nouveau) | nouveau tour du même joueur, `extra_turn` vrai | non | non |

Le jeton de Puits de mana du début de tour passe par `ManaTokenGained` dans `TurnStarted`. À la fin de la manche, le dé gardé
revient et les cartes écartées rejoignent le deck avant son mélange (`RoundEnded`, spec 023).

## Comportements attendus

### Contenu et Don pour la magie

- **C1** — Plus aucun sort n'est `Unsupported` ; Fuite de mana de base est `OthersOnly`, jamais proposée en solo, jouable de côté
  (`all_spells_supported`, `mana_meltdown_basic_sideways_only`) — contenu ; Q2.
- **C2** — Offre de sorts avec Restoration : Don pour la magie de base en défaussant Marche (verte), Restauration lancée depuis
  l'offre (mana vert) : Soin 3 ; le sort reste dans l'offre (`magic_talent_casts_any_spell`) — spec 054.

### Voyage souterrain et Attaque souterraine

- **C3** — Exemple ci-dessus : de `(2, -2)` vers le village `(2, -3)` par Voyage souterrain : pas de provocation, aucun point de
  Mouvement dépensé (Marche jouée avant : 2 points restent), l'interaction reste possible (`underground_travel_no_provocation`) —
  carte ; MKUE p. 7, 7.c.
- **C4** — Le lac `(2, -4)` n'est ni une arrivée ni un passage ; la case des orques `(3, -3)` n'est pas une arrivée (pas sécurisée) ;
  une case à 4 de distance non plus ; une montagne se traverse (`underground_travel_destinations`) — carte ; MKUE p. 7, 7.d ;
  Compendium, *Movement* S5.
- **C5** — Après l'action du tour, en combat, ou après un déplacement arrêté par une provocation : pas proposé
  (`underground_travel_only_in_movement_phase`) — MKUE p. 6, 5.c ; Compendium, *Movement* S2.
- **C6** — De nuit, à 3 cases d'une forteresse non conquise défendue par des Guardsmen (`enemies.ron` : armure 7, Fortifié) :
  Attaque souterraine (cristal vert, jeton noir) ; assaut, réputation −1, les Guardsmen ne sont fortifiés que par leur jeton
  (une Attaque de siège les vise en phase à distance) ; la retraite ramène le héros à sa case de départ, pas à la case voisine
  (`underground_attack_assault_and_retreat`) — carte ; Compendium, *Underground Attack* S1 ; spec 043.

### Ailes du vent et Ailes de la nuit

- **C7** — Héros sur le portail `(0, 0)`, Marche et Célérité jouées (Mouvement 4) : Ailes du vent (mana blanc) vers le village
  `(2, -3)`, 3 points dépensés, 1 reste pour un pas ordinaire ; aucune provocation ; une arrivée à plus de 4 points, ou à plus de 5
  cases, n'est pas proposée (`wings_of_wind_spends_move_points`) — carte ; Compendium, *Wings of Wind* S5.
- **C8** — De nuit, phase à distance contre Prowlers, Swordsmen et Wolf Riders : Marche jouée en combat (Mouvement 2), puis Ailes de
  la nuit sur les Prowlers et les Swordsmen (1 point) : aucun des deux n'attaque ; les trois exigeaient 3 points, pas proposé avec 2
  (`wings_of_night_extra_targets_cost_move`) — carte (LL) ; Compendium, *Wings of Night* S1.
- **C9** — En combat, sans Ailes de la nuit lançable, l'effet Mouvement de Marche n'est pas proposé (comme aujourd'hui, spec 052) ;
  avec Ailes de la nuit en main, de nuit, mana blanc et noir disponibles, il l'est (`move_effects_in_combat_for_wings_of_night`) —
  MKUE p. 6, 5.c ; Q4.

### Restauration et Renaissance

- **C10** — De jour, héros en forêt `(1, -1)`, deux Blessures en main, Paysans blessés : Restauration, 5 points de soin, deux
  Blessures jetées et les Paysans soignés (niveau I) ; sur la plaine `(0, 0)`, 3 points ; en combat, pas proposée
  (`restoration_heals_more_in_forest`) — carte ; MKUE p. 6, 5.b.
- **C11** — De nuit, en forêt, Arbalétriers d'Utem (II), Paysans (I) et Golems gardiens (II) épuisés : Renaissance les rend tous trois
  disponibles (5 niveaux) ; sur la plaine, au plus 3 niveaux (Arbalétriers et Paysans) (`rebirth_readies_levels`) — carte ;
  Compendium, *Rebirth* S2.

### Distorsion spatiale et Distorsion temporelle

- **C12** — Héros en `(0, 0)`, Distorsion spatiale (mana bleu) : `(1, -2)` (plaine, à 2 cases) s'atteint pour 2 points comme une
  voisine ; plus tard dans le tour, un pas ordinaire de `(2, -2)` à `(2, -3)` ne provoque pas les orques de `(3, -3)`
  (`space_bending_two_spaces_as_adjacent`) — carte ; Compendium, *Space Bending* Q1, S3.
- **C13** — Héros en `(1, -2)` avec Distorsion spatiale : l'emplacement A + B (case `(3, -4)` à 2 cases) s'explore pour 2 points
  (`space_bending_explores_two_spaces_away`) — Compendium, *Space Bending* S2 ; MKUE p. 7, 5.
- **C14** — Assaut d'un site fortifié à 2 cases par Distorsion spatiale, repoussé : retour sur la case d'où l'on a sauté ; de jour,
  à 2 cases d'une tour de mage, ses défenseurs ne sont pas révélés (`space_bending_assault_retreat_and_no_reveal`) — Compendium,
  *Space Bending* S5, S7.
- **C15** — De nuit, Marche, Rage, Célérité de côté et Improvisation (défausse de Concentration) jouées, puis Distorsion
  temporelle (cristal bleu, jeton noir) : en fin de tour, Marche, Rage, Célérité et Improvisation reviennent en main, pas
  Concentration ; Distorsion temporelle est écartée ; aucune carte piochée ; un nouveau tour commence, dés relancés et jetons
  rendus, unités toujours épuisées ; Distorsion temporelle depuis l'offre n'y est pas proposée ; à la fin de la manche, la carte
  écartée est remélangée dans le deck (`time_bending_extra_turn`) — carte ; MKUE p. 11 ; Compendium, *Time Bending* S7, S13 ; Q3.
- **C16** — Distorsion temporelle jouée après l'annonce de la fin de la manche, ou au dernier tour de Première reconnaissance : le
  tour supplémentaire a lieu avant la fin (`time_bending_after_end_announced`) — Compendium, *Time Bending* S2, S10.

### Appel aux armes et Appel à la gloire

- **C17** — Arbalétriers d'Utem dans l'offre d'unités ; en combat, Appel aux armes (mana blanc), puis la capacité Attaque à distance
  2 des Arbalétriers en phase à distance ; ils restent dans l'offre, une seconde activation est refusée ; ils ne sont pas une cible
  des dégâts ; les Golems gardiens de l'offre en Parade de feu 4 exigent leur mana rouge ; les Guerriers d'Utem ne proposent que leur
  première capacité ; une action avancée de l'offre (monastère) n'est pas proposée ; dans le combat d'un donjon, rien
  (`call_to_arms_uses_offer_unit`) — carte ; Compendium, *Call to Arms* S2, S3, S15 ; Q6.
- **C18** — De nuit, héros niveau 1 (un jeton de commandement) avec des Paysans, hors de tout site : Appel à la gloire, Épéistes
  d'Utem de l'offre recrutés sans Influence en renvoyant les Paysans ; disponibles ; un Récit héroïque en cours ne compte pas ce
  recrutement (`call_to_glory_recruits_free`) — carte ; Compendium, *Call to Glory* S2 ; spec 053 Q5.

### Méditation et Transe

- **C19** — Défausse de cinq cartes : Méditation tire deux cartes (événement `DiscardPicked`, retour en arrière impossible avant
  lui), le joueur les met sur le deck dans l'ordre choisi ; en fin de tour, pioche jusqu'à 5 + 2 = 7 cartes, les deux premières
  étant celles remises (`meditation_random_cards_on_deck`) — carte ; Compendium, *Meditation* S2.
- **C20** — Transe : deux cartes choisies, mises sous le deck ; défausse d'une seule carte : une seule remise ; défausse vide : jouable,
  la pioche reste à + 2 (`trance_chooses_cards`) — carte ; Compendium, *Meditation* S4.
- **C21** — Méditation au dernier tour de la manche : la pioche de la nouvelle manche va jusqu'à la limite + 2 ; deux Méditations le
  même tour : + 4 (`meditation_draw_bonus_carries_over`) — Compendium, *Meditation* S1, O3.

### Fuite de mana, Puits de mana, Lecture d'esprit

- **C22** — De nuit, deux cristaux verts : Brûlure de mana, couleur verte, deux Blessures en main, puis trois cristaux verts et un
  jeton vert ; couleur rouge sans cristal rouge : aucune Blessure, deux cristaux rouges (`mana_radiance_wounds_then_crystals`) —
  carte ; MKUE p. 5, *"Gain" Effects* 2.
- **C23** — Source bleue, rouge et noire, de jour : Puits de mana (cristal bleu) sur le dé rouge, option trois jetons : trois jetons
  rouges ; le joueur peut encore prendre le dé bleu de la Source ce tour ; aux tours suivants, la Source n'a plus que deux dés, le dé
  rouge n'est pas relancé ; à la nouvelle manche, il revient (`mana_claim_keeps_die_until_end_of_round`) — carte ; Q7.
- **C24** — Option un jeton par tour : un jeton rouge au début de chaque tour suivant de la manche, plus rien à la manche suivante ;
  le dé noir n'est pas proposé (`mana_claim_token_each_turn`) — carte.
- **C25** — Lecture d'esprit, couleur blanche : un cristal blanc ; avec déjà trois cristaux blancs, un jeton blanc ; Vol d'esprit
  fait de même (`mind_read_gains_crystal`) — carte ; MKUE p. 5, *"Gain" Effects* 2.

### Flux d'énergie et Vol d'énergie

- **C26** — Gardiens d'Altem (IV) épuisés : Flux d'énergie les rend disponibles ; en combat, pas proposé ; Vol d'énergie sur des
  Guerriers d'Utem épuisés et blessés : disponibles et soignés ; des Paysans disponibles et blessés ne sont pas une cible
  (`energy_flow_readies_any_level`) — carte ; Compendium, *Energy Steal* Q1, S2, S3.

### Vue et persistance

- **C27** — La vue donne le dé gardé, les cartes écartées, le tour supplémentaire, les cartes de Méditation en attente et les
  modificateurs nouveaux (`SpaceBending`, `UnitFromOffer`, `DrawBonus`, `ManaClaimPerTurn`) (`view_shows_spell_states`) —
  spec 052.
- **C28** — Sauvegarde pendant un tour supplémentaire, avec un dé gardé et une carte écartée, rechargée : tout est rétabli ; retour
  en arrière jusqu'avant Voyage souterrain : la figurine revient (`extra_turn_and_kept_die_follow_reload`) — spec 024.

## Cas limites et refus

- **L1** — Voyage souterrain ou Ailes du vent avec destination absente, ou vers la case du héros : refusés.
- **L2** — Ailes de la nuit hors phase à distance ou de parade, ou visant deux fois le même ennemi : refusé.
- **L3** — Renaissance visant une unité disponible, ou deux fois la même : refusée.
- **L4** — `PlaceOnDeck` sans cartes en attente : refusée ; tant qu'elles attendent, seule cette action est légale.
- **L5** — Puits de mana sans dé de couleur de base libre dans la Source : pas proposé.
- **L6** — Appel à la gloire avec une offre d'unités vide : pas proposé.

## Information cachée

Rien de nouveau de caché : la défausse, l'offre et la Source sont publiques ; les cartes remises sur le deck sont connues du
joueur, et la vue ne montre pas l'ordre du deck (inchangé). Champs ajoutés à la vue : `kept_die`, `set_aside`, `extra_turn`, les
cartes de Méditation en attente (`PlayerView`, C27) ; les modificateurs passent par `PlayerView::modifiers` (spec 052).

## Libellés (i18n)

Les textes `card.<sort>.name`, `strong_name`, `basic_text` et `strong_text` existent (relus dans `assets/i18n/fr.json`).
Corrigés : `card.restoration.strong_text` « rendre prêts » devient « rendre disponibles » (glossaire, spec 042) ;
`card.mana_meltdown.basic_text`, `card.mind_read.basic_text`, `card.energy_flow.*` et `card.mana_claim.strong_text` gardent le texte
de la carte (les autres joueurs y restent nommés). Nouvelles clés (fr ; en à l'avenant) :

- `move.underground` : « Voyage souterrain : choisir l'arrivée » ; `move.fly` : « Ailes du vent : choisir l'arrivée ({points} au
  plus) » (fr « Ailes du vent : choisir l'arrivée (4 au plus) »).
- `modifier.space_bending` : « cases à 2 comme voisines, sans provoquer » ; `modifier.unit_from_offer` : « une unité de l'offre ce
  tour » ; `modifier.draw_bonus` : « {count} cartes de plus à la pioche » (fr « 2 cartes de plus à la pioche ») ;
  `modifier.mana_claim` : « un jeton {color} par tour » (`{color}` repris de `effect.gain_spell`, relu).
- `turn.extra` : « Tour supplémentaire » (barre de menu, à côté du tour, spec 074) ; `set_aside.title` : « Écartées jusqu'à la fin de
  la manche ».
- `meditation.title` : « Méditation : où remettre ces cartes ? » ; `meditation.top` : « Sur le deck » ; `meditation.bottom` :
  « Sous le deck ».
- `mana_claim.kept_die` : « Dé gardé jusqu'à la fin de la manche » ; `mana_claim.now` : « 3 jetons ce tour » ;
  `mana_claim.each_turn` : « 1 jeton par tour ».
- `offers.use` : « Utiliser » (unité de l'offre avec Appel aux armes) ; `offers.recruit_free` : « Recruter (gratuit) ».
- `wings_of_night.cost` : « Mouvement : {points} » (coût des cibles cochées).

## Interface

- **Destination d'un déplacement par effet** : après le clic sur Voyage souterrain, Attaque souterraine ou Ailes du vent (et le
  paiement), les arrivées légales sont mises en évidence sur le plateau comme les cases atteignables (spec 017), le clic choisit
  (C3, C4, C6, C7, actions légales `PlayCard`) ; Échap annule.
- **Distorsion spatiale** : cases atteignables et trajets comptent les sauts (C12, `routes` étendu) ; les emplacements explorables à 2
  cases ont leur bouton d'exploration (C13).
- **Ailes de la nuit** : choix des ennemis comme pour un sort qui vise un ennemi (spec 054), plusieurs cochés, coût en Mouvement
  affiché (`wings_of_night.cost`), validation grisée si la réserve ne suffit pas (C8, actions légales).
- **Renaissance, Flux d'énergie** : unités cibles cochées dans le panneau du bas, niveaux comptés (C11, C26, actions légales).
- **Appel aux armes** : avec `UnitFromOffer`, bouton « Utiliser » sur les unités de l'offre dans la fenêtre des offres, qui ouvre
  leurs capacités comme pour une unité possédée (C17) ; **Appel à la gloire** : bouton « Recruter (gratuit) », puis l'unité à
  renvoyer s'il le faut (C18).
- **Méditation** : fenêtre avec les cartes tirées ou à choisir, deux boutons « Sur le deck », « Sous le deck », ordre par
  glisser ou flèches (C19, C20, `PlayerView` cartes en attente) ; mise en page : vérification à l'œil.
- **Puits de mana** : choix du dé dans la Source, puis des deux options (C23, C24) ; dé gardé montré à côté de la Source avec
  `mana_claim.kept_die` (C27, `PlayerView::kept_die`).
- **Brûlure de mana, Lecture d'esprit** : couleur par les boutons d'options existants (spec 073) (C22, C25).
- **Tour supplémentaire** : « Tour supplémentaire » dans la barre de menu (C15, `PlayerView::extra_turn`) ; cartes écartées
  sous la défausse, infobulle `set_aside.title` (C27) ; placement : vérification à l'œil.
- **Effets en cours** : `SpaceBending`, `UnitFromOffer`, `DrawBonus`, `ManaClaimPerTurn` dans la zone de jeu (C27).

Écarts de l'implémentation : le paiement d'un déplacement par effet se choisit dans le bandeau, puis l'arrivée d'un clic sur
le plateau (passé directement s'il n'y a qu'un paiement ; issue privée 257, 2026-11-13) ; les ennemis d'Ailes de la nuit se
cochent sur les cartes de l'écran de combat, les unités visées dans le panneau du bas, et le bandeau ne propose que les
paiements des lancers qui visent les cibles cochées, sans bouton quand il n'y en a pas (réserve de Mouvement insuffisante,
niveaux dépassés ; issue privée 258, 2026-11-13) ;
les cartes écartées sont une ligne sous le bouton de la défausse. Vérification à l'œil : partie préparée `sorts-deplacement`.

### Raccourcis clavier

Échap annule le choix de l'arrivée d'un déplacement par effet, comme le choix du mana (reporté dans `docs/raccourcis.md`). Rien
d'autre : tout passe par les boutons existants.

## Effets sur les specs déjà implémentées

- **014, 017, 030, 036** : un déplacement par effet ; voisinage à 2 avec Distorsion spatiale ; pas de provocation.
- **016, 023** : tour supplémentaire sans pioche ; pioche + 2 ; cartes écartées et dé gardé rendus à la fin de la manche.
- **020, 022** : un dé peut être hors de la Source pour la manche.
- **042** : une unité de l'offre peut être activée ; recrutement gratuit hors interaction.
- **043** : assaut lancé à distance, retraite vers la case de départ du sort.
- **054** : tous les sorts de l'offre se lancent avec Don pour la magie (fin de L4).

## Hors périmètre

- Ce qui vise les autres joueurs (Fuite de mana, Malédiction de mana, Lecture et Vol d'esprit, Flux et Vol d'énergie) : pas de jeu à
  plusieurs (`CLAUDE.md`, Périmètre).
- Cités comme arrivée d'Attaque souterraine : spec 062. Tactiques et Distorsion temporelle (ordre du tour) : spec 060.
- Artefacts qui lancent un sort (Tome de tous les sorts) : spec 055.

## Plan de développement

Grosse spec (moteur et écran, issue privée 118) : deux fils sur la même branche, plan `plans/057-sorts-deplacement-unites-mana.md`
rédigé par le fil d'implémentation :

1. **Moteur** : déplacements par effet et Distorsion spatiale, Distorsion temporelle, unités, Méditation, mana, contenu, vue,
   persistance (C1 à C28).
2. **Écran** : libellés, destination, Méditation, Puits de mana, offre d'unités, tour supplémentaire, vérification à l'œil, docs,
   bilan.

## Décisions (2026-11-12)

Mathieu accepte les réponses proposées (« le reste : ok »), appliquées dans le texte ci-dessus :

1. **Q1 — Découpage.** Une seule spec, deux fils (moteur, écran). **Décision : une seule spec.**
2. **Q2 — Fuite de mana de base en solo.** Pas proposée, jouable de côté. **Décision : oui.**
3. **Q3 — Distorsion temporelle écartée.** Jusqu'à la fin de la manche, puis remélangée dans le deck ; confirmé par le texte du wiki
   (« set this card aside for the rest of the Round »), le compendium ne disant rien de plus que *Time Bending* S5. **Décision :
   jusqu'à la fin de la manche.**
4. **Q4 — Mouvement en combat.** Effets de Mouvement proposés en combat seulement si Ailes de la nuit peut être lancé. **Décision :
   oui.**
5. **Q5 — Flux d'énergie en combat.** Icône de soin : pas en combat. **Décision : pas en combat.**
6. **Q6 — Appel aux armes et Guerriers d'Utem.** Le compendium (*Call to Arms* Q6, *Utem Swordsmen* S1) permet la capacité qui
   blesse l'unité de l'offre ; Mathieu choisit de simplifier, le cas étant rare. **Décision : capacités `Wounding` pas proposées.**
7. **Q7 — Dé de Puits de mana.** Ne compte pas comme le dé du tour ; dé de couleur de base libre seulement. **Décision : oui.**
8. **Q8 — Révélations d'un déplacement par effet.** Rien sur le chemin, l'arrivée comme une arrivée ordinaire. **Décision : oui.**

## Questions ouvertes

Aucune.
