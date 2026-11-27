# 051 — Compétences : gain aux niveaux pairs et premier lot

- Statut : implémentée (validée le 2026-11-13, questions Q1 à Q5 tranchées : réponses proposées acceptées)
- Jalon : J5
- Dépend de : 035 (gloire et niveaux, `LevelReward::SkillAndAdvancedAction`), 050 (offre d'actions avancées, action avancée
  des niveaux pairs), 052 (moteur d'effets, règle « effet utilisable », `PlayMoment`), 013 (options d'une carte), 020 à 025
  (Source, cristaux, jetons, 4e cristal changé en jeton), 034 (soin, pioche, Assommé), 040 (Influence), 042 (unités rendues
  disponibles, unités blessées), 031 et 032 (combat, éléments), 023 (nouvelle manche), 024 et 038 (retour en arrière,
  générateurs par usage), 073 (panneau du bas : place réservée à la section Compétences) ; toutes implémentées
- Livret : MKUE p. 3 (*Player's area* : pile de compétences, *Common Skill offer*) ; p. 4 (*Preparing a new Round*, 2.g) ;
  p. 5 (*Using Skills* 1 à 4) ; p. 11 (*End of the Turn*, 6.b) ; p. 15 (*Skills in a Solo Game*). Texte des compétences :
  `docs/regles/cartes/heros-et-competences.md` (compétences coopératives : section ajoutée par cette PR, texte du wiki) ; noms
  français : photos `docs/regles/scans/fr/characters/<héros>/skills_fr_*.jpg`. FAQ p2 (*Compétences* : Motivation en solo) ;
  FAQ p3 (offre de compétences communes). Compendium, *Skill Tokens* S1 à S5, *Day Sharpshooting* S1, *Night Sharpshooting* S1,
  *Dark Negotiation* S1, *Dark Fire Magic* Q1, *Inspiration* Q1, S2, *Motivation* S1 à S6
- ADR : [0005 Contenu en données](../decisions/0005-contenu-en-donnees.md) (`content/skills.ron`), [0007 Un générateur par
  usage](../decisions/0007-generateurs-par-usage.md) (usage nouveau : pile de compétences)
- Crates touchées : arcanum-core (règles, contenu), arcanum-ui, arcanum-persistance (parties préparées), `assets/i18n/`

## Objectif

À chaque niveau pair, le joueur **révèle deux compétences** de la pile de son héros, **en garde une** et pose l'autre dans
l'**offre de compétences communes**, puis prend son action avancée comme aujourd'hui. Ses compétences s'affichent dans le panneau
du bas et **s'utilisent une fois par tour ou une fois par manche**, selon leur symbole.

Les quarante compétences du solo (dix par héros) sont dans le contenu. Cette spec code le mécanisme et les **23 compétences qui
n'ont besoin d'aucune règle nouvelle** : des points (Mouvement, Influence, Attaque, Parade), du mana et des cristaux, Motivation,
un soin, une unité rendue disponible ou soignée. Les **17 autres** forment un second lot, spec 058 (Q1) : d'ici là, elles se
gagnent et s'affichent avec leur texte, sans pouvoir servir (« Pas encore disponible »), comme une carte `Unsupported`.

### Les compétences du solo

Chaque héros a onze jetons, dont deux interactifs : un compétitif (casque noir) et un coopératif (casque blanc). En solo, on
garde le **coopératif** et on retire le compétitif (Compendium, *Skill Tokens* S5 ; MKUE p. 3, *Pile of Skill tokens*). Les
quatre compétitives (*Healing Ritual*, *Source Freeze*, *Prayer of Weather*, *Mana Exploit*) sont hors périmètre (Q2).

Noms français lus sur les photos des cartes françaises ; texte : `heros-et-competences.md`. **Attention** : la carte française
d'Arythea inverse les deux noms : la coopérative (*Ritual of Pain*) s'appelle « Rituel de guérison » et la compétitive (*Healing
Ritual*) « Rituel de souffrance » (photo `arythea/skills_fr_2.jpg`). On garde les noms de la carte française (Q3).

| Héros | Compétence (fr / en) | Fréquence | Effet | Lot |
|---|---|---|---|---|
| Arythea | Chemins ténébreux (*Dark Paths*) | tour | Mouvement 1 de jour, 2 de nuit | 051 |
| Arythea | Attaque brûlante (*Burning Power*) | tour | Attaque de siège 1, ou de siège de Feu 1 | 051 |
| Arythea | Bretteur de feu (*Hot Swordsmanship*) | tour | Attaque 2, ou de Feu 2 | 051 |
| Arythea | Sombre négociation (*Dark Negotiation*) | tour | Influence 2 de jour, 3 de nuit | 051 |
| Arythea | Magie noire (*Dark Fire Magic*) | manche | un cristal rouge, et un jeton rouge ou noir | 051 |
| Arythea | Motivation | manche | piocher 2 cartes, et un jeton rouge | 051 |
| Arythea | Souffrance sublimée (*Power of Pain*) | tour | une Blessure jouée de côté, +2 | 058 |
| Arythea | Invocation | tour | défausser une carte pour un jeton à dépenser aussitôt | 058 |
| Arythea | Polarisation (*Polarization*) | tour | un mana utilisé comme sa couleur opposée | 058 |
| Arythea | Rituel de guérison (*Ritual of Pain*, coopérative) | manche, au centre | jeter jusqu'à 2 Blessures ; au tour suivant, une Blessure de côté pour +3 | 058 |
| Goldyx | Attaque givrante (*Freezing Power*) | tour | Attaque de siège 1, ou de siège de Glace 1 | 051 |
| Goldyx | Potion de soin (*Potion Making*) | manche, hors combat | Soin 2 | 051 |
| Goldyx | Extraction blanche (*White Crystal Craft*) | manche | un cristal bleu et un jeton blanc | 051 |
| Goldyx | Extraction verte (*Green Crystal Craft*) | manche | un cristal bleu et un jeton vert | 051 |
| Goldyx | Extraction rouge (*Red Crystal Craft*) | manche | un cristal bleu et un jeton rouge | 051 |
| Goldyx | Motivation | manche | piocher 2 cartes, et un jeton vert | 051 |
| Goldyx | Fortune clinquante (*Glittering Fortune*) | tour, en interaction | Influence 1 par couleur de cristal | 058 |
| Goldyx | Vol (*Flight*) | manche | une case gratuite, ou deux pour 2 Mouvement, sans provoquer | 058 |
| Goldyx | Pouvoir universel (*Universal Power*) | tour | un mana sur une carte de côté : +3, ou +4 | 058 |
| Goldyx | Embrasure de la Source (*Source Opening*, coopérative) | manche, au centre | relancer un dé ; au tour suivant, un dé de plus et son cristal | 058 |
| Norowas | Œil de lynx (*Day Sharpshooting*) | tour | Attaque à distance 2 de jour, 1 de nuit | 051 |
| Norowas | Négociation brillante (*Bright Negotiation*) | tour | Influence 3 de jour, 2 de nuit | 051 |
| Norowas | Inspiration | manche, hors combat | rendre disponible une unité, ou la soigner | 051 |
| Norowas | Au cœur du vent (*Leaves in the Wind*) | manche | un cristal vert et un jeton blanc | 051 |
| Norowas | Murmure de la canopée (*Whisper in the Treetops*) | manche | un cristal blanc et un jeton vert | 051 |
| Norowas | Motivation | manche | piocher 2 cartes, et un jeton blanc | 051 |
| Norowas | Marche en avant (*Forward March*) | tour | Mouvement 1 par unité disponible et non blessée, 3 au plus | 058 |
| Norowas | Charisme (*Leadership*) | tour | bonus à l'unité activée | 058 |
| Norowas | Loyauté sans faille (*Bonds of Loyalty*) | permanente | jeton de commandement, recrutement à −5 | 058 |
| Norowas | Temps calme (*Calming the Weather*, coopérative) | manche, au centre | terrains −2 ce tour ; au tour suivant, −1 | 058 |
| Tovak | Marche forcée (*Double Time*) | tour | Mouvement 2 de jour, 1 de nuit | 051 |
| Tovak | Vision nocturne (*Night Sharpshooting*) | tour | Attaque à distance 1 de jour, 2 de nuit | 051 |
| Tovak | Bretteur de glace (*Cold Swordsmanship*) | tour | Attaque 2, ou de Glace 2 | 051 |
| Tovak | Maître bouclier (*Shield Mastery*) | tour | Parade 3, ou de Glace 2, ou de Feu 2 | 051 |
| Tovak | Motivation | manche | piocher 2 cartes, et un jeton bleu | 051 |
| Tovak | Botte secrète (*Resistance Break*) | tour | un ennemi : Armure −1 par résistance | 058 |
| Tovak | Même pas mal (*I Feel No Pain*) | tour, hors combat | défausser une Blessure, piocher une carte | 058 |
| Tovak | Flexibilité (*I Don't Give a Damn*) | tour | une carte de côté : +2, ou +3 | 058 |
| Tovak | Magie inutile (*Who Needs Magic?*) | tour | une carte de côté : +2, ou +3 sans dé de la Source | 058 |
| Tovak | Surcharge de mana (*Mana Overload*, coopérative) | manche, au centre | un jeton ; plus tard, +4 sur une carte renforcée de cette couleur | 058 |

Critère du partage : une compétence est du premier lot si son effet existe déjà pour une carte ou une unité (au pire avec une
valeur qui dépend du moment de la journée, ou un cristal et un jeton de couleurs différentes) ; elle est du second lot si elle
demande une règle nouvelle (carte jouée de côté modifiée, mana transformé, déplacement par effet, compétence posée au centre…).

## Règles

### Gagner une compétence

- **Pile de compétences** (MKUE p. 3, *Pile of Skill tokens*) : à la mise en place, les dix compétences du héros (sans la
  compétitive, Q2) sont mélangées en une pile face cachée, par un générateur propre (ADR 0007, usage `SkillPile`). Dix
  suffisent toujours : cinq niveaux pairs (2, 4, 6, 8, 10) révèlent deux compétences chacun.
- **Niveau pair** (MKUE p. 11, 6.b) : le joueur révèle les deux compétences du dessus de sa pile, puis choisit :
  - **en garder une**, poser l'autre dans l'offre de compétences communes, et prendre **n'importe quelle** action avancée de
    l'offre (ce que fait déjà la spec 050) ;
  - ou **prendre une compétence d'un autre héros** dans l'offre de compétences communes, poser ses deux compétences révélées dans
    l'offre, et prendre l'action avancée en **dernière place** de l'offre. Jamais une compétence de son propre héros
    (Compendium, *Skill Tokens* S3 ; FAQ p3).
- **Ordre** : la compétence se choisit d'abord, l'action avancée ensuite (la seconde option fixe la carte). Le choix se fait au
  même moment que l'action avancée aujourd'hui : après le changement de niveau, avant la pioche de fin de tour (spec 050). Deux
  niveaux pairs gagnés d'un coup se traitent l'un après l'autre (spec 035, L4) : la seconde paire n'est révélée qu'après le
  premier choix.
- **Offre d'actions avancées vide** (spec 050, L1) : la compétence est gagnée quand même, sans carte.
- **Offre de compétences communes en solo** (MKUE p. 15, *Skills in a Solo Game*) : elle reçoit les compétences du joueur fantôme,
  qui arrive avec la spec 061. D'ici là, elle ne contient que les compétences écartées par le joueur, qu'il ne peut pas prendre :
  la seconde option existe dans le moteur et à l'écran, mais n'est jamais proposée en partie ; ses cas se testent depuis une offre
  préparée (Q4).
- **Compétence gagnée** (MKUE p. 5, *Using Skills* 2.a) : face visible devant le joueur, utilisable dès son tour suivant (le
  choix se fait en fin de tour).

### Utiliser une compétence

- **Fréquence** (MKUE p. 5, *Using Skills* 4) :
  - **une fois par tour** (sans symbole, 4.c) : utilisable de nouveau au tour suivant, y compris un tour supplémentaire
    (Distorsion temporelle, spec 057) ;
  - **une fois par manche** (4.a) : la compétence est **retournée face cachée** et le reste jusqu'au début de la manche suivante
    (MKUE p. 4, 2.g) ;
  - les compétences « au centre » (4.b) sont toutes du second lot (spec 058).
- **Moment** (Compendium, *Skill Tokens* S1) : pendant son propre tour seulement, plateau ou combat. Utiliser une compétence
  n'est **pas l'action du tour** (S1) et ne compte pas comme jouer une carte. Motivation « pendant le tour de n'importe quel
  joueur » : en solo, cela revient au tour du joueur (le moment avant le choix des tactiques, Compendium *Motivation* S6,
  attend la spec 060).
- **Une seule fois** (Compendium, *Skill Tokens* S2) : une utilisation applique **une seule option**, une seule fois.
- **Quand elle se propose** : une option de compétence suit **la même règle que l'option d'une carte** (spec 052, `PlayMoment`) :
  des points de Mouvement ou d'Influence hors combat, une Attaque dans la bonne phase (siège, distance, attaque), une Parade en
  phase de parade, du mana à tout moment de son tour. « Sauf pendant un combat » (Potion de soin, Inspiration ; icône de la main,
  Compendium *Inspiration* Q1) : pas en combat, comme une carte de soin (spec 034, `healing_card_cannot_be_played_in_combat`).
- **Jour et nuit** (texte des compétences) : l'effet prend la valeur du moment de la journée **où il est utilisé** ; dans un
  donjon ou un tombeau, c'est toujours la nuit pendant le combat (Compendium, *Day Sharpshooting* S1, *Night Sharpshooting* S1,
  *Dark Negotiation* S1 ; spec 044, `combat_time`).

### Le premier lot

- **Points** (Chemins ténébreux, Attaque brûlante, Bretteur de feu, Sombre négociation, Attaque givrante, Œil de lynx,
  Négociation brillante, Marche forcée, Vision nocturne, Bretteur de glace, Maître bouclier) : chaque « ou » du texte est une
  option, comme les options d'une carte. Les points s'ajoutent à ceux des cartes (spec 013).
- **Cristal et jeton** (Magie noire, les trois Extractions, Au cœur du vent, Murmure de la canopée) : le cristal va dans
  l'inventaire, et un **4e cristal devient un jeton** (spec 025, `fourth_crystal_becomes_a_token`). Magie noire a deux options
  (jeton rouge, ou jeton noir) ; un jeton noir de jour ne sert que là où le noir est permis (donjon, tombeau ; Compendium, *Dark
  Fire Magic* Q1).
- **Motivation** (une par héros, couleur du jeton : Arythea rouge, Goldyx vert, Norowas blanc, Tovak bleu) : piocher 2 cartes du
  dessus du deck, sans remélanger (spec 034, `draw_stops_when_deck_is_empty` ; Compendium *Motivation* S3), et **toujours** le
  jeton, car en solo on a la moins de gloire (FAQ p2 ; Compendium *Motivation* S1). Utilisable en combat ; une Blessure piochée
  ainsi ne compte pas pour Assommé (Compendium *Motivation* S2), comme aujourd'hui : seules les Blessures reçues en combat
  comptent (spec 034, `knock_out_wounds` de `combat.rs`). **Une seule Motivation** de la fin de son utilisation jusqu'à la fin du
  tour suivant (texte de la compétence, lecture littérale ; levée à la fin de la manche, Compendium *Motivation* S5). Ne sert
  qu'avec deux Motivation, donc avec la seconde option du niveau pair (Q4).
- **Potion de soin** : Soin 2 hors combat (spec 034, effet `Heal`).
- **Inspiration** : deux options hors combat, **rendre disponible** une unité épuisée, de tout niveau (spec 042), ou **soigner**
  une unité blessée, sans payer son niveau en Soin (Compendium, *Inspiration* S2 : une seule Blessure). L'unité est choisie avec
  l'utilisation, comme la cible d'une capacité d'unité.

### Exemple : Tovak au niveau 2

Tovak gagne 3 gloire et atteint le niveau 2 à la fin de son tour. Sa pile révèle Bretteur de glace et Magie inutile ; il garde
Bretteur de glace, Magie inutile part dans l'offre commune, puis il prend Coup de sang (*Blood Rage*, `advanced_actions.ron`)
dans l'offre d'actions avancées, qui va sur son deck, avant sa pioche. Au tour suivant, en phase d'attaque, son panneau du bas montre Bretteur de glace
avec deux boutons, Attaque 2 et Attaque de Glace 2 ; il clique Attaque de Glace 2, le bouton s'éteint jusqu'à la fin du tour.

## Modèle

Termes du glossaire : Compétence (`Skill`), Offre, Action avancée, Dernière place, Joueur fantôme. **Termes nouveaux** (ajoutés au
glossaire par cette PR) :

| Livret (EN) | Français | Code |
|---|---|---|
| Pile of Skill tokens | Pile de compétences | `PlayerState::skill_pile` |
| Common Skill offer | Offre de compétences communes (FAQ p3) | `GameState::common_skills` |
| Once a turn / Once a Round | Une fois par tour / une fois par manche | `SkillUsage::Turn` / `Round` |
| Flip (a Skill token) face down | Retourner (une compétence) | `SkillState::Flipped` |
| Competitive / cooperative interactive Skill | Compétence compétitive / coopérative | `Skill::is_cooperative` |

Données nouvelles :

- **Contenu** : `content/skills.ron`, une entrée par compétence (40) : héros, fréquence, « hors combat », options (effets, comme
  les options d'une carte) ou `Unsupported` pour le second lot. Les identifiants (`Skill`, `Skill::ALL`, `Skill::label_key`) sont
  en Rust (ADR 0005). Deux formes d'effet nouvelles, si rien d'existant ne convient : une valeur **de jour et de nuit**, et
  **un cristal et un jeton** de couleurs différentes (`TokenAndCrystal` existe pour une même couleur, spec 042).
- **Joueur** : la pile de compétences (ordre caché), les compétences gagnées avec leur état (disponible, utilisée ce tour,
  retournée jusqu'à la manche suivante), les deux compétences révélées en attente de choix, le nombre de niveaux pairs encore à
  traiter, la compétence de l'offre commune prise (qui impose la dernière carte), la fin du blocage de Motivation.
- **Partie** : l'offre de compétences communes (liste ordonnée par arrivée).

## Actions et événements

| Action | Validation (refusée si…) | Événements produits |
|---|---|---|
| `ChooseSkill { skill }` | pas de compétence en attente de choix ; `skill` ni parmi les deux révélées, ni dans l'offre commune ; compétence du propre héros prise dans l'offre commune | `SkillGained`, puis `SkillToCommonOffer` pour chaque compétence révélée non gardée (les deux si `skill` vient de l'offre commune, après `SkillTakenFromCommonOffer`) |
| `GainAdvancedAction { card }` (spec 050) | en plus : une compétence encore à choisir pour ce niveau ; après une compétence de l'offre commune, `card` n'est pas la dernière place | comme aujourd'hui ; puis, s'il reste un niveau pair, `SkillsRevealed` du suivant |
| `UseSkill { skill, option, unit }` | compétence non possédée, déjà utilisée ce tour, retournée, `Unsupported` ; option absente ou qui ne sert pas maintenant (`PlayMoment`) ; « hors combat » en combat ; Motivation bloquée ; `unit` absente, ou qui ne convient pas (Inspiration : épuisée pour la rendre disponible, blessée pour la soigner) | `SkillUsed { skill, option }`, puis les événements de l'effet (points, `CrystalGained`, `ManaTokenGained`, `CardDrawn`, `UnitReadied`, `UnitHealed`…, existants) |

| Événement | Effet de `apply` | Révèle une information ? | Point de non-retour ? |
|---|---|---|---|
| `SkillPileShuffled { player, order }` | pose la pile (mise en place) | non (ordre caché, comme `CommonDeckShuffled`) | oui (hasard de la mise en place) |
| `SkillsRevealed { player, skills }` | les deux compétences du dessus quittent la pile, en attente de choix | oui | oui |
| `SkillGained { player, skill }` | compétence ajoutée, disponible | non | non |
| `SkillTakenFromCommonOffer { player, skill }` | retirée de l'offre commune ; la carte gagnée ensuite sera la dernière | non | non |
| `SkillToCommonOffer { skill }` | ajoutée à l'offre commune | non | non |
| `SkillUsed { player, skill, option }` | utilisée ce tour, ou retournée (une fois par manche) ; Motivation : blocage jusqu'à la fin du tour suivant | non | non (les événements de l'effet le sont s'ils piochent : `CardDrawn`) |

La première paire est révélée à la fin du tour, avec les autres choix de fin de tour (spec 050) : `SkillsRevealed` suit
`LevelGained`, ou, si un artefact ou un sort de récompense attend son choix, `ArtifactTaken` ou `OfferCardGained` du dernier de
ces choix (MKUE p. 11, 5 puis 6.b ; issue privée 317). Pas d'événement pour remettre les compétences : `TurnEnded` libère celles « par tour », `RoundStarted` retourne face
visible celles « par manche » (MKUE p. 4, 2.g) et lève le blocage de Motivation.

## Comportements attendus

### Contenu et mise en place

- **C1** — Le contenu décrit 40 compétences, dix par héros, dont une coopérative par héros ; les 23 du premier lot ont des
  options, les 17 autres sont `Unsupported` (`skills_content_has_ten_skills_per_hero`, `first_batch_skills_are_supported`).
- **C2** — À la mise en place, la pile de compétences du héros contient ses dix compétences, mélangées par l'usage `SkillPile` ;
  la même graine donne la même pile, et le tirage ne décale ni le deck ni la Source (ADR 0007)
  (`setup_shuffles_the_hero_skill_pile`, `skill_pile_does_not_shift_other_draws`) — MKUE p. 3.
- **C3** — Aucune compétence gagnée ni offre commune au départ (`no_skill_at_start`) — MKUE p. 5, *Using Skills* 2.

### Gagner une compétence

- **C4** — Tovak au niveau 1 avec 3 gloire finit son tour : `LevelGained` 2 puis `SkillsRevealed` avec les deux premières
  compétences de sa pile ; la pioche attend (`even_level_reveals_two_skills`) — MKUE p. 11, 6.b.
- **C5** — Il choisit l'une des deux : elle est gagnée, l'autre va dans l'offre commune ; il prend ensuite n'importe quelle carte
  de l'offre d'actions avancées, puis pioche (`keep_one_skill_and_any_advanced_action`) — MKUE p. 11, 6.b.
- **C6** — Tant que la compétence n'est pas choisie, `GainAdvancedAction` est refusée et n'est pas parmi les actions légales
  (`skill_is_chosen_before_advanced_action`).
- **C7** — Depuis une offre commune préparée avec Bretteur de feu (Arythea), Tovak au niveau pair prend Bretteur de feu : ses deux
  compétences révélées vont dans l'offre commune, et seule la carte en dernière place de l'offre d'actions avancées est légale
  (`common_offer_skill_forces_lowest_advanced_action`) — MKUE p. 11, 6.b ; FAQ p3.
- **C8** — Une compétence de Tovak dans l'offre commune n'est pas proposée à Tovak (`own_skill_in_common_offer_is_not_offered`) —
  Compendium, *Skill Tokens* S3 ; FAQ p3.
- **C9** — Offre d'actions avancées vide : la compétence se choisit, aucune carte n'est attendue, la pioche suit
  (`skill_gained_with_empty_advanced_action_offer`) — spec 050, L1.
- **C10** — Deux niveaux pairs gagnés d'un coup (du niveau 1 avec 15 gloire : niveaux 2, 3 et 4) : deux
  choix successifs de compétence et de carte, la seconde paire révélée après la première carte
  (`two_even_levels_reveal_pairs_one_after_the_other`) — spec 035, L4.
- **C11** — Le bandeau du niveau pair au tour suivant ne dit plus « pas encore disponible » (`level.gained_even`, Libellés)
  (`even_level_banner_mentions_skill`).

### Utiliser une compétence

- **C12** — Bretteur de glace (Tovak) en phase d'attaque : l'option Attaque de Glace 2 ajoute 2 d'Attaque de Glace ; utilisée
  une seconde fois dans le même tour, refusée ; au tour suivant, de nouveau légale (`once_a_turn_skill_resets_next_turn`) —
  MKUE p. 5, 4.c.
- **C13** — Bretteur de glace hors combat n'est pas proposée ; Maître bouclier n'est proposée qu'en phase de parade ;
  Attaque brûlante (siège) l'est en phase à distance (`skill_options_follow_play_moment`) — spec 052.
- **C14** — Chemins ténébreux de jour donne Mouvement 1, de nuit Mouvement 2 ; Œil de lynx dans un donjon de jour donne Attaque à
  distance 1 (`day_and_night_skill_values`) — texte ; Compendium, *Day Sharpshooting* S1.
- **C15** — Sombre négociation donne Influence 2 de jour, dépensée pour recruter comme celle d'une carte
  (`skill_influence_adds_to_card_influence`) — spec 040.
- **C16** — Magie noire (Arythea) : un cristal rouge et un jeton rouge, ou un jeton noir ; la compétence est retournée ; plus
  utilisable de la manche, de nouveau au début de la manche suivante (`once_a_round_skill_flips_until_next_round`) — MKUE p. 5,
  4.a ; p. 4, 2.g.
- **C17** — Extraction blanche avec déjà 3 cristaux bleus : le 4e devient un jeton bleu, en plus du jeton blanc
  (`crystal_skill_fourth_crystal_becomes_token`) — spec 025.
- **C18** — Motivation (Norowas) : pioche 2 cartes et un jeton blanc ; en combat aussi ; une Blessure piochée ne compte pas pour
  Assommé (`motivation_draws_two_and_gains_token`, `motivation_wound_does_not_knock_out`) — FAQ p2 ; Compendium *Motivation* S1,
  S2.
- **C19** — Motivation avec un deck d'une carte : une carte piochée, le jeton quand même (`motivation_with_short_deck`) —
  Compendium *Motivation* S3.
- **C20** — Avec deux Motivation (offre commune préparée), la seconde est refusée pendant le tour de la première et le tour
  suivant, et redevient légale à la manche suivante (`second_motivation_blocked_until_end_of_next_turn`) — texte ; Compendium
  *Motivation* S5.
- **C21** — Potion de soin hors combat : Soin 2 ; non proposée en combat (`potion_making_heals_two_out_of_combat`) — texte ;
  spec 034.
- **C22** — Inspiration : rend disponible une unité épuisée de niveau 3, ou soigne une unité blessée sans payer de Soin ; non
  proposée en combat ; refusée sur une unité disponible (pour la rendre disponible) ou non blessée (pour la soigner)
  (`inspiration_readies_or_heals_a_unit`) — texte ; Compendium, *Inspiration* Q1, S2.
- **C23** — Utiliser une compétence ne compte pas comme l'action du tour : après Sombre négociation, le joueur peut encore se
  reposer s'il n'a rien joué d'autre ; et elle n'entre pas dans la zone de jeu (`skill_use_is_not_an_action`) — Compendium,
  *Skill Tokens* S1.
- **C24** — Une compétence du second lot (Magie inutile) gagnée n'a aucune action légale (`unsupported_skill_cannot_be_used`).
- **C25** — Retour en arrière : une compétence utilisée sans pioche s'annule et redevient disponible ; Motivation ne s'annule pas
  (pioche) (`skill_use_undo`) — spec 024.

### Vue et persistance

- **C26** — La vue montre les compétences gagnées avec leur état et leurs options, les deux révélées en attente de choix, l'offre
  commune (prenable ou non), et la contrainte de dernière place ; jamais l'ordre de la pile (`view_shows_skills`).
- **C27** — Une partie sauvegardée avec des compétences gagnées, utilisées et retournées se recharge à l'identique
  (`skills_survive_save_and_load`) — spec 024.

## Cas limites et refus

- **L1** — `ChooseSkill` sans choix en attente, avec une compétence non révélée, ou deux fois pour le même niveau : refusée.
- **L2** — `UseSkill` pendant un choix en attente (fin de tour, Méditation…) : refusée, comme une carte.
- **L3** — Option dont les points ne servent pas (Bretteur de feu en phase de parade) : refusée (C13).
- **L4** — Compétence retournée gagnée sur le dernier tour d'une manche : face visible à la manche suivante, comme les autres.
- **L5** — Fin de partie annoncée (spec 063) : un niveau pair au dernier tour donne quand même la compétence (le décompte ne la
  compte pas).

## Information cachée

La pile de compétences est face cachée : la vue n'en montre que le nombre restant, pas l'ordre (`SkillPileShuffled` masqué comme
`CommonDeckShuffled`). Les deux compétences révélées et l'offre commune sont visibles.

Champs ajoutés à `PlayerView` (tout ce que l'écran affiche) :

- `skills` : les compétences gagnées, chacune avec son état (`Ready`, `UsedThisTurn`, `Flipped`), ses options utilisables
  maintenant (actions légales) et `unsupported` ;
- `skill_choice` : les deux compétences révélées en attente de choix (vide sinon) ;
- `common_skills` : l'offre commune, chaque compétence marquée prenable ou non (propre héros, pas de choix en cours) ;
- `lowest_advanced_action_only` : après une compétence de l'offre commune, seule la dernière carte est prenable ;
- `skill_pile_count` : le nombre de compétences encore dans la pile.

## Libellés (i18n)

Toutes les clés en fr et en. Les textes des compétences sont **écrits par nous** (ADR 0009 pour les visuels, même principe que
les textes `basic_text_key` des cartes, spec 050) ; les noms français sont ceux des cartes françaises, les noms anglais les noms
officiels (gardés, spec 082).

| Clé | Paramètres | fr | en |
|---|---|---|---|
| `skill.<id>.name` (40) | — | Chemins ténébreux… (tableau de l'Objectif) | Dark Paths… |
| `skill.<id>.text` (40) | — | texte court écrit par nous, ex. « Mouvement 1 le jour, 2 la nuit. » | ex. "Move 1 by day, 2 by night." |
| `skill.title` | — | Compétences | Skills |
| `skill.once_a_turn` | — | Une fois par tour | Once a turn |
| `skill.once_a_round` | — | Une fois par manche | Once a Round |
| `skill.out_of_combat` | — | Hors combat | Not in combat |
| `skill.used_this_turn` | — | Utilisée ce tour | Used this turn |
| `skill.flipped` | — | Retournée jusqu'à la prochaine manche | Flipped until next Round |
| `skill.motivation_blocked` | — | Une autre Motivation a servi : pas avant la fin du prochain tour | Another Motivation was used: not before the end of your next turn |
| `skill.choice.title` | `level` | Niveau {level} : choisir une compétence | Level {level}: choose a Skill |
| `skill.choice.revealed` | — | Compétences révélées | Revealed Skills |
| `skill.choice.common` | — | Offre de compétences communes | Common Skill offer |
| `skill.choice.common_note` | — | Prendre une compétence de l'offre commune impose l'action avancée en dernière place. | Taking a Skill from the common offer means taking the lowest Advanced Action. |
| `skill.choice.own` | — | Compétence de votre héros : pas prenable ici | Your Hero's Skill: cannot be taken here |
| `skill.pile` | `count` | Pile : {count} | Pile: {count} |
| `skill.choose_unit` | `skill` | {skill} : choisir l'unité | {skill}: choose the unit |
| `skill.pick_unit` | — | Choisir | Choose |

Clés réutilisées (valeurs relues dans `assets/i18n/fr.json`) : `offers.take` = « Prendre » (bouton de la fenêtre de choix) ;
`effect.unsupported` = « Pas encore disponible » (compétence du second lot). Clé modifiée : `level.gained_even`, aujourd'hui
« Niveau {level} : compétence pas encore disponible », devient « Niveau {level} : une compétence et une action avancée gagnées »
(en : "Level {level}: a Skill and an Advanced Action gained") ; avec `{level}` = 2 : « Niveau 2 : une compétence et une action
avancée gagnées ». Aucun symbole hors des polices du jeu.

## Interface

Schéma indicatif du panneau du bas (spec 073 l'avait réservé, à droite de la main) :

```
┌───────────────────────────────────────────────────────────────────────────────────────────┐
│ Unités ▢▢          Main ▢▢▢▢▢▢                          │ Compétences      Pile : 8         │
│                                                          │ ┌ Bretteur de glace ───── ↻ tour┐ │
│                                                          │ │ [⚔2] [❄⚔2]                    │ │
│                                                          │ ├ Magie noire ─────────── manche┤ │
│                                                          │ │ (retournée)                   │ │
│                                                          │ ├ Magie inutile ──────── tour ──┤ │
│                                                          │ │ Pas encore disponible         │ │
│                                                          │ └───────────────────────────────┘ │
└───────────────────────────────────────────────────────────────────────────────────────────┘
```

- **Section Compétences** du panneau du bas, à droite de la main, sur le plateau comme en combat : une ligne par compétence
  gagnée, avec son nom, sa fréquence et un **bouton-icône par option** (mêmes `IconValue` que les boutons Jouer des cartes,
  spec 071) ; la section n'apparaît qu'avec une compétence (C26, `PlayerView::skills` ; placement : vérification à l'œil).
- **État** : une option non utilisable maintenant est grisée ; une compétence utilisée ce tour ou retournée est grisée avec
  `skill.used_this_turn` ou `skill.flipped` ; une du second lot montre `effect.unsupported` (C12, C16, C24, `PlayerView::skills`).
- **Infobulle** de la ligne : nom, fréquence, « Hors combat » s'il y a lieu, texte `skill.<id>.text` (C26 ; vérification à
  l'œil).
- **Inspiration** : après le clic sur une option, choix de l'unité dans la section Unités, comme la cible d'une capacité
  (C22, `PlayerView::skills`) : bandeau `skill.choose_unit` avec « Annuler », bouton `skill.pick_unit` sur chaque unité
  possible (clés ajoutées au fil écran) ; une seule unité possible, le bouton de l'option l'utilise directement.
- **Fenêtre du choix de compétence** (titre `skill.choice.title`), ouverte d'office à la fin du tour d'un niveau pair, avant la
  fenêtre de l'action avancée : les deux compétences révélées (nom, fréquence, texte), un bouton « Prendre » chacune ; dessous,
  l'offre commune avec sa note, chaque compétence prenable ou non (`skill.choice.own`). Puis la fenêtre Offres en mode choix
  d'action avancée, comme aujourd'hui, avec seule la dernière carte prenable après une compétence de l'offre commune
  (C4 à C8, `PlayerView::skill_choice`, `common_skills`, `lowest_advanced_action_only`). Depuis la [spec #20](issue-20-obtention-des-cartes.md) : une seule
  fenêtre du niveau pair, compétence encadrée puis « Prendre » sur l'action avancée, qui prend les deux.
- **Bandeau** du niveau pair au tour suivant : `level.gained_even` (C11).

Partie préparée pour la vérification à l'œil (`docs/verification-manuelle.md`) :

- `competences` : Tovak, Première reconnaissance, niveau 3 avec 15 gloire (niveau 4 à la prochaine fin de tour), Bretteur de glace,
  Motivation et Magie inutile déjà gagnées (Motivation retournée) ; un ennemi errant adjacent pour voir les options en combat.
  Finir le tour ouvre la fenêtre du choix de compétence.

### Raccourcis clavier

Aucun : les compétences s'utilisent par leurs boutons, comme les capacités d'unités, qui n'ont pas de touche non plus. La fenêtre
du choix de compétence se ferme comme les autres choix (Échap ne refuse pas : le choix est obligatoire, comme l'action avancée).

## Effets sur les specs déjà implémentées

- **050** : `GainAdvancedAction` attend désormais la compétence du niveau ; la seconde option de 6.b (spec 050, hors périmètre)
  arrive ici. Les tests qui prennent une action avancée au niveau pair choisissent d'abord une compétence :
  `crates/arcanum-core/tests/offres.rs`, `gloire_et_niveaux.rs`, `sites_fortifies.rs`, `ruines.rs`, `villages_et_monasteres.rs`,
  `contenu_en_donnees.rs`, `support/mod.rs` ; `crates/arcanum-persistance/tests/offres.rs` ; `crates/arcanum-ui/tests/ruines.rs` ; les
  parcours `parcours_j3.rs` et `parcours_j4.rs` (le joueur scripté ajoute un `ChooseSkill` ; le nouvel usage du hasard ne décale
  pas leurs graines, ADR 0007). Liste relevée par `grep -rln "GainAdvancedAction\|advanced_action_choices\|gained_even"`.
- **035** : le bandeau `level.gained_even` change de valeur (C11) ; sa Q2 (« la 051 décidera ») est levée : pas de rattrapage des
  compétences manquées, aucune partie n'a de sauvegarde à migrer avant la fin du J6 (spec 024, Q1).
- **073** : la section Compétences prend la place réservée (spec 073, C7, qui dit qu'elle n'existe pas encore : test mis à jour).

## Hors périmètre

- Les 17 compétences du second lot, dont les quatre coopératives posées au centre : spec 058 (Q1).
- Les quatre compétences compétitives : jamais en solo (Q2) ; avec le jeu à plusieurs, hors plan.
- Les compétences du joueur fantôme dans l'offre commune (MKUE p. 15) : spec 061.
- Motivation avant le choix des tactiques (Compendium, *Motivation* S6) : spec 060.
- Les compétences des héros des extensions (Wolfhawk, Krang, Braevalar) : pas de héros des extensions au plan.
- Illustrations des compétences : rien de prévu (texte et icônes seulement).

## Plan de développement

Grosse spec (moteur et écran) : deux fils successifs sur la même branche (issue privée 118). Plan détaillé
(`docs/specs/plans/051-competences.md`) au début du fil moteur.

1. Fil moteur : contenu `skills.ron` et identifiants (C1), pile et mise en place (C2, C3), gain au niveau pair et offre commune
   (C4 à C11), utilisation (C12 à C25), vue et persistance (C26, C27), tests existants adaptés.
2. Fil écran : libellés, section Compétences du panneau du bas, fenêtre du choix, bandeau, partie préparée `competences`,
   vérification à l'œil, docs (`architecture.md`, `verification-manuelle.md`, `index-par-notion.md`).

## Décisions (2026-11-13)

Mathieu a accepté les réponses proposées (« Ok avec les choix par défaut ») :

- **Q1 — Deux lots** : cette spec code le mécanisme et les 23 compétences sans règle nouvelle ; la spec 058 « Compétences :
  second lot » code les 17 autres. D'ici là, une compétence du second lot se gagne et s'affiche, sans servir.
- **Q2 — Coopérative en solo** : la coopérative est dans la pile, la compétitive retirée (Compendium, *Skill Tokens* S5).
- **Q3 — Noms français** : ceux imprimés sur les cartes françaises ; la coopérative d'Arythea est « Rituel de guérison ».
  « Loyauté sans faille » vérifié sur la photo `norowas/skills_fr_2.jpg` au fil écran (le JSON d'extraction dit « Loyalité »).
- **Q4 — Seconde option du niveau pair** : codée ici, moteur et écran, testée depuis une offre commune préparée.
- **Q5 — Section Compétences** : à droite de la main, une ligne par compétence avec ses boutons d'options (schéma de la
  section Interface), sans maquette préalable ; vérification à l'œil.

## Questions ouvertes

Aucune.

<details><summary>Questions posées à la relecture</summary>

- **Q1 — Deux lots.** Quarante compétences, c'est trop pour une spec. Proposition : cette spec code le mécanisme et les 23
  compétences sans règle nouvelle ; une **spec 058 « Compétences : second lot »** (ligne ajoutée à `PLAN.md` juste après la 051)
  code les 17 autres (cartes de côté modifiées, mana transformé, Vol, Charisme, Loyauté sans faille, Botte secrète, les quatre
  coopératives au centre…). D'ici là, une compétence du second lot se gagne et s'affiche, sans servir. Autre choix possible : les
  retirer de la pile jusqu'à la 058, mais la pile ne suffirait plus aux cinq niveaux pairs (5 à 6 compétences par héros).
- **Q2 — Compétitive ou coopérative en solo.** Le livret ne dit que « coopératif : retirer la compétitive ; compétitif : retirer la
  coopérative » (p. 3) et, pour le fantôme, de retirer sa compétitive (p. 15). Le compendium tranche (*Skill Tokens* S5) : en
  solo, on joue avec la coopérative, dont l'avantage différé revient au joueur à son tour suivant. Proposition : suivre le
  compendium (la coopérative dans la pile, la compétitive retirée). Ce n'est pas une contradiction avec le livret, une précision.
- **Q3 — Noms d'Arythea inversés.** Sur la carte française, la coopérative *Ritual of Pain* s'appelle « Rituel de guérison » et
  la compétitive *Healing Ritual* « Rituel de souffrance » (l'inverse de la traduction littérale). Proposition : garder les noms
  imprimés sur la carte française (la coopérative est donc « Rituel de guérison »), comme pour les autres cartes. Autres noms
  à confirmer, lus sur les photos : Bretteur de feu, Œil de lynx (*Day Sharpshooting*), Charisme (*Leadership*), Botte secrète
  (*Resistance Break*), Flexibilité (*I Don't Give a Damn*), Magie inutile (*Who Needs Magic?*), Temps calme (*Calming the
  Weather*), Embrasure de la Source (*Source Opening*), Loyauté sans faille (*Bonds of Loyalty*, « Loyalité » dans le JSON
  d'extraction, à vérifier sur la photo pendant le fil écran).
- **Q4 — Seconde option du niveau pair dès maintenant.** En solo sans joueur fantôme, l'offre commune ne contient que les
  compétences écartées par le joueur, qu'il ne peut pas prendre : la seconde option (compétence d'un autre héros contre la
  dernière carte) ne se voit pas en partie avant la spec 061. Proposition : la coder ici, moteur et écran, testée depuis une offre
  commune préparée (C7, C8, C20), pour que la 061 n'ait qu'à remplir l'offre. Autre choix : la reporter à la 061.
- **Q5 — Place de la section Compétences.** Proposition : à droite de la main dans le panneau du bas, une ligne par compétence
  avec ses boutons d'options (schéma ci-dessus), comme le prévoyait la spec 073 (« colonne d'icônes avec le détail en
  infobulle »). Avec cinq compétences au plus, la colonne tient dans la hauteur du panneau (240 pixels de carte, spec 077). Faut-il
  une maquette avant de valider ?

</details>
