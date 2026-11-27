# 056 — Actions avancées : mana et cartes

- Statut : implémentée (validée le 2026-11-12, questions Q1 à Q9 tranchées : réponses par défaut acceptées)
- Jalon : J5
- Dépend de : 053 (premier lot, `extra_mana`, `WoundFromCard`), 052 (moteur d'effets, ADR 0008, règle « effet utilisable »),
  050 (offres, gagner une carte), 020 à 025 (Source, cristaux, jetons, Concentration), 034 (assommé) ; toutes implémentées
- Livret : MKUE p. 4 (3.a, 6 et 6.a : coût d'un effet, « carte action ») ; p. 5 (*Using Mana* 1.b et 3 ; *Effects* 1.b ;
  *"Composite" Effects* 1 ; *"Gain" Effects* 1 à 3 ; *Discarding and Throwing Away* 2 ; *Reverting* 2) ; p. 11 (*End of the Turn*
  1 et 3). Texte des cartes : `docs/regles/cartes/actions-avancees.md` (version *Lost Legion* pour Rituel de sang ; c'est aussi
  celle des cartes françaises, `scans/fr/advanced_actions/`). FAQ p1 et p2 (Effet maximum, Maîtrise du cristal, Don pour la
  magie). Compendium, *Advanced Action Cards* (Blood Ritual S1 à S3, Crystal Mastery Q1, S2, Learning S1, Magic Talent S1, O4,
  S5, Mana Storm S1, Maximal Effect S1 à S34, Training S2 à S4)
- ADR : [0008 Moteur d'effets](../decisions/0008-moteur-d-effets.md) (deux sortes de modificateurs nouvelles)
- Crates touchées : arcanum-core (règles, contenu), arcanum-ui, arcanum-persistance (parties préparées), `assets/i18n/`

## Objectif

Les huit dernières actions avancées du jeu de base font ce qu'elles disent : gagner des cristaux et des jetons en se blessant ou en
détruisant une carte, récupérer ses cristaux dépensés, relancer la Source, copier deux ou trois fois l'effet d'une autre carte, et
prendre des cartes des offres d'actions avancées et de sorts. C'est le second lot de la spec 053 (Q1) ; avec lui, plus aucune
action avancée n'est `Unsupported`, sauf l'effet de base de Don pour la magie, qui joue un sort (spec 054).

| Carte (fr.json) | Base | Renforcé | Ce qui est nouveau |
|---|---|---|---|
| Rituel de sang (*Blood Ritual*, LL) | Blessure ; cristal rouge et un jeton de n'importe quelle couleur | Blessure ; trois jetons de n'importe quelles couleurs ; payer un mana de base : cristal de cette couleur | choix de couleurs après la carte |
| Décomposition (*Decompose*) | détruire une carte action de la main : deux cristaux de sa couleur | idem : un cristal de chacune des trois autres couleurs de base | détruire une carte (`CardDestroyed`) |
| Maîtrise du cristal (*Crystal Mastery*) | cristal d'une couleur déjà possédée | les cristaux dépensés ce tour reviennent à la fin du tour | modificateur `ReturnSpentCrystals` |
| Tempête de mana (*Mana Storm*) | dé de la Source d'une couleur de base : cristal de cette couleur, le dé est relancé | relancer la Source ; trois dés de plus ; noir et or valent toute couleur de base | relance en cours de tour (`DieRerolled`) |
| Effet maximum (*Maximal Effect*) | une autre carte action : son effet de base trois fois, puis la détruire | son effet renforcé deux fois, gratuitement, puis la détruire | effet répété |
| Don pour la magie (*Magic Talent*), renforcé | (spec 054) | payer un mana : un sort de cette couleur de l'offre dans la défausse | carte gagnée dans la défausse |
| Apprentissage (*Learning*) | Influence 2 ; une fois, 6 Influence : action avancée de l'offre dans la défausse | Influence 4 ; une fois, 9 Influence : dans la main | action `LearnAdvancedAction` |
| Entraînement (*Training*) | détruire une carte action : une action avancée de même couleur de l'offre dans la défausse | idem, dans la main | carte gagnée dans la main |

## Règles

### Règles communes

- **Détruire une carte** (MKUE p. 5, *Discarding and Throwing Away* 2.b ; p. 4, 6 et 6.a) : la carte quitte le jeu, hors de
  toute pile du joueur ; elle ne revient pas en fin de tour (p. 11, 1). Une **carte action** est une action de base (du héros
  compris) ou une action avancée, jamais une Blessure, un sort ni un artefact. La carte détruite est une **autre** carte que celle
  qui le demande (Compendium, *Training* S2). Sans carte à détruire, la carte ne se joue pas (p. 4, 6). Terme : Q4.
- **Carte gagnée d'une offre** (MKUE p. 5, *"Gain" Effects* 3 et 3.a ; spec 050) : la carte va **où la carte le dit** (défausse ou
  main) au lieu du dessus du deck, puis l'offre est complétée comme aujourd'hui (`OfferCardDealt`, point de non-retour, spec 050
  C14). Les actions avancées des monastères, dans l'offre d'unités, ne se prennent pas ainsi (Compendium, *Training* S4).
- **Effet utilisable** (spec 052) : les cartes de mana de ce lot (Rituel de sang, Décomposition, Maîtrise du cristal, Tempête de
  mana) se jouent à tout moment du tour, combat compris, comme Concentration (spec 025). Entraînement et Don pour la magie aussi.
  Apprentissage suit la règle de son Influence.
- **Cristal en trop** (MKUE p. 5, *"Gain" Effects* 2 ; spec 021) : un 4e cristal d'une couleur devient un jeton, comme aujourd'hui.

### Rituel de sang

- **Blessure** (carte ; Compendium, *Blood Ritual* S3) : une Blessure en main, comme Coup de sang (`WoundFromCard`, spec 053) ;
  en combat, elle compte pour être assommé, hors combat non.
- **Base** : cristal rouge et un jeton d'une des six couleurs, or et noir compris (six options). Un jeton noir de jour, ou or de
  nuit, se prend même s'il ne sert à rien (Compendium, *Blood Ritual* S2).
- **Renforcé** : après la carte, le joueur choisit **trois couleurs de jetons**, une par une (or et noir compris). Puis il peut
  **payer un mana de base** pour un cristal de cette couleur, **une fois, aussitôt** (Compendium, *Blood Ritual* S1), avec l'un
  des jetons qu'il vient de gagner ou un autre mana (dé, cristal, jeton), ou refuser (Q5).

### Décomposition

- **Base** : la carte action détruite, choisie en jouant Décomposition, donne **deux cristaux de sa couleur** (spec 021 pour le
  quatrième). **Renforcé** : un cristal de chacune des trois autres couleurs de base.

### Maîtrise du cristal

- **Base** : une option par couleur de base, permise seulement si l'inventaire a au moins un cristal de cette couleur.
- **Renforcé** (FAQ p2 ; Compendium, *Crystal Mastery* Q1, S2) : modificateur de fin du tour `ReturnSpentCrystals` : à la fin du
  tour, chaque cristal dépensé **pendant ce tour** (`CrystalUsed`, avant ou après la carte, le cristal bleu qui l'a payée compris,
  l'offrande d'un autel aussi, spec 045) revient dans l'inventaire, au plus 3 par couleur (Q8). Ce qui a été payé par un dé ou un
  jeton ne revient pas.

### Tempête de mana

- **Base** (carte) : le joueur choisit un dé **de la Source** (pas pris ce tour) qui montre une couleur de base ; il gagne un
  cristal de cette couleur, puis le dé est **relancé aussitôt** et reste dans la Source (`DieRerolled`, hasard de la Source, ADR
  0007). Ce n'est pas le dé du tour : le joueur peut encore en prendre un. La relance est un point de non-retour (MKUE p. 5,
  *Reverting* 2).
- **Renforcé** (carte ; Compendium, *Mana Storm* S1) : tous les dés **encore dans la Source** sont relancés (pas ceux pris ce tour,
  dont celui qui a peut-être payé la carte) ; puis le joueur peut prendre **trois dés de plus** ce tour, et tout dé noir ou or
  qu'il prend vaut n'importe quelle couleur de base, de jour comme de nuit, même épuisé (Q9). Comme Attraction de mana (spec 025),
  sans limite à un seul dé noir.

### Effet maximum

- **Composite** (MKUE p. 5, *"Composite" Effects* 1.a et 1.b ; FAQ p1, p2 ; Compendium, *Maximal Effect* S1, Q4, S5) : la carte
  jouée avec Effet maximum (la **cible**, choisie en le jouant, comme pour Concentration, spec 025) donne l'effet de **son option
  choisie** répété trois fois (base) ou deux fois (renforcé, la cible ne se paie pas). Des points répétés font **un seul** effet :
  Rage ×3 donne une Attaque 6, pas trois Attaques 2. La cible est ensuite **détruite** ; Effet maximum va dans la zone de jeu.
- **Une même option** pour toutes les répétitions (Q2).
- **Cibles** (Q1) : une carte action de la main dont l'option est **répétable** : elle ne pose aucun modificateur, ne demande ni
  carte à défausser ou détruire, ni mana en plus, ni unité, ni dé, et son effet est l'un de : Mouvement, Influence, Attaque,
  Parade (de tout élément), Parade Vif, Soin, Piocher, Influence et réputation (Menace renforcée), Influence en interaction (Nobles
  manières), gagner un jeton, gagner un cristal, Attaque avec Blessure (Coup de sang : une Blessure par répétition, Compendium,
  *Maximal Effect* S33), Mouvement et Soin, Influence par Blessure. La répétition multiplie les points, les jetons, les cristaux,
  les cartes piochées, la réputation, la gloire et les Blessures. Les autres options ne sont pas proposées (liste en exemple
  ci-dessous).
- **Effet utilisable** (spec 052) : l'option répétée doit servir maintenant (Rage en phase d'attaque, March hors combat…).
- **Couleur** (FAQ p2 ; Compendium, *Maximal Effect* Q3) : celle de la cible, pour les résistances ; aucune option répétable ne
  pose d'effet sur un ennemi, la question ne se pose pas dans ce lot.
- **Concentration** (Q3) : Effet maximum n'est pas une cible de Concentration ni de Pouvoir de concentration, et elles ne sont pas
  ses cibles (leur effet renforcé `Boost` n'est pas répétable ; leur effet de base, un jeton, l'est).
- **Euphorie cristalline** détruite par Effet maximum ne se reprend pas en main (elle n'est pas dans la zone de jeu).

### Don pour la magie (renforcé)

- **Sort de l'offre** (carte ; Compendium, *Magic Talent* O4, S5) : en plus du mana bleu de l'effet renforcé, le joueur paie **un
  mana d'une couleur de base** (`extra_mana`, spec 053 ; l'or de jour vaut une couleur de base, MKUE p. 5) et prend **un sort de
  cette couleur** de l'offre de sorts, choisi en jouant la carte, **dans sa défausse**. Une option par couleur, permise seulement si
  l'offre a un sort de cette couleur. L'effet de base (jouer un sort de l'offre) reste `Unsupported` jusqu'à la spec 054.

### Apprentissage

- **Achat** (carte ; Compendium, *Learning* S1) : la carte donne son Influence et pose `LearningDiscount { cost, to }` (6 et
  défausse ; renforcé 9 et main). Tant qu'il est là, l'action `LearnAdvancedAction { card }` dépense `cost` Influence de la réserve
  pour une carte de l'offre d'actions avancées ; le modificateur est alors retiré (une fois par tour). Elle se fait **hors combat**, tant que
  l'Influence sert (phase de mouvement ou interaction, spec 052), sans interaction nécessaire (Q7). En interaction, l'Influence de la réserve comprend déjà le bonus de réputation
  (spec 040) ; hors interaction, rien ne s'y ajoute.
- **Effet utilisable** : la carte se joue quand son Influence sert (spec 052), donc pas en combat sans conversion.

### Entraînement

- **Échange** (carte ; Compendium, *Training* S2, S3) : en jouant la carte, le joueur choisit la carte action à détruire **et**
  une action avancée **de la même couleur** dans l'offre ; il la prend (base : dans la défausse ; renforcé : dans la main, jouable
  aussitôt). Sans action avancée de la couleur d'une carte de sa main dans l'offre, la carte ne se joue pas (lecture stricte, Q6).

### Exemple : Effet maximum

Main : Effet maximum, Rage, Marche, Tranquillité, Improvisation, Agilité, Concentration.

| Jouée avec | Comment | Résultat |
|---|---|---|
| Rage, base, phase d'attaque | Effet maximum de base | Attaque 6 (2 × 3), Rage détruite |
| Marche, renforcé, phase de mouvement | Effet maximum payé en rouge | Mouvement 8 (4 × 2), Marche détruite |
| Tranquillité, base, hors combat | option Piocher | trois cartes piochées (point de non-retour) |
| Concentration, base | option jeton bleu | trois jetons bleus |
| Improvisation, Agilité, Concentration renforcée | — | pas proposées : défausse, modificateur, `Boost` |

## Modèle

Termes du glossaire : Offre, Gagner (une carte), Compléter (une offre), Relancer, Cristal, Jeton de mana. Terme à ajouter avant
validation : **Détruire (une carte)** (*throw away*, une carte action), `CardDestroyed` (Q4).

Données nouvelles (noms indicatifs, forme dans l'ADR 0008) :

- **`Effect`** : `WoundCrystalAndToken { crystal, token }` (Rituel de sang de base), `WoundAndTokens(3)` (renforcé),
  `CrystalsFromDestroyed { same: 2 }` / `CrystalsOtherColors` (Décomposition), `GainOwnedCrystal(BasicColor)` (Maîtrise du
  cristal), `CrystalFromDie` / `RerollSource` (Tempête de mana), `Repeat(u8)` (Effet maximum), `GainSpell(BasicColor)` (Don pour
  la magie), `TrainFromOffer { to }` (Entraînement).
- **`Cost::DestroyActionCard`** : comme `DiscardAnotherCard` (Improvisation), la carte choisie dans le champ `discard` de
  `CardPlay`, détruite au lieu d'être défaussée.
- **`CardPlay`** : `Basic` reçoit `with` (la cible d'Effet maximum, `BoostTarget` existant) ; un champ `gain` (la carte prise dans
  une offre : Entraînement, Don pour la magie) ; un champ `die` (le dé choisi par Tempête de mana de base). `serde(default)` pour
  les sauvegardes.
- **`ModifierKind`** : `ReturnSpentCrystals` (fin du tour), `LearningDiscount { cost, to }` (consommé par l'achat) ; Tempête de
  mana renforcée réutilise `extra_dice` et `black_any_dice` (spec 025) et y ajoute l'or et les dés épuisés.
- **Choix en attente** de Rituel de sang renforcé : jetons restant à choisir (3, puis 2, 1), puis cristal proposé ; les actions
  légales ne proposent alors que `ChooseTokenColor { color }`, puis `PayForCrystal { color, mana }` ou `DeclineCrystal`, comme le
  choix de couleur du dé d'or du donjon (spec 044, `ChooseCrystalColor`).
- **`OfferCardGained`** reçoit sa destination (`to` : dessus du deck, défausse, main ; dessus du deck par défaut).
- **Contenu** : les 7 cartes et l'effet renforcé de Don pour la magie quittent `Unsupported` dans `advanced_actions.ron`.

## Actions et événements

| Action | Validation (refusée si…) | Événements produits |
|---|---|---|
| `PlayCard` (existante) | règle « effet utilisable » ; carte à détruire absente de la main, égale à la carte jouée ou pas une carte action ; Entraînement sans action avancée de sa couleur dans l'offre, ou `gain` d'une autre couleur ; Don pour la magie sans sort de la couleur payée ; Maîtrise du cristal sans cristal de la couleur ; Tempête de mana sur un dé pris, épuisé, noir ou or ; Effet maximum sans cible, avec une option non répétable, ou une cible qui ne sert pas maintenant | `CardPlayed` ; `CardDestroyed` (nouveau) ; `CrystalGained`, `ManaTokenGained`, `WoundFromCard` (existants) ; `DieRerolled` (nouveau) ; `OfferCardGained` (`to`) puis `OfferCardDealt` |
| `ChooseTokenColor { color }` (nouvelle) | pas de jeton de Rituel de sang à choisir | `ManaTokenGained` |
| `PayForCrystal { color, mana }` / `DeclineCrystal` (nouvelles) | pas de cristal proposé ; mana absent ou d'une autre couleur | `ManaTokenUsed`/`DieTaken`/`CrystalUsed`, `CrystalGained` ; ou `CrystalDeclined` (nouveau) |
| `LearnAdvancedAction { card }` (nouvelle) | pas de `LearningDiscount` ; en combat ; Influence de la réserve < coût ; carte absente de l'offre d'actions avancées | `AdvancedActionLearned` (nouveau, retire l'Influence et le modificateur), `OfferCardGained` (`to`), `OfferCardDealt` |
| `EndTurn` (existante) | inchangée | `CrystalsReturned` (nouveau, Maîtrise du cristal) avant la défausse |

| Événement | Effet de `apply` | Révèle une information ? | Point de non-retour ? |
|---|---|---|---|
| `CardDestroyed { card, from }` | la carte quitte la main (Décomposition, Entraînement) ou la cible d'Effet maximum quitte le jeu | non | non |
| `DieRerolled { die, color }` | le dé de la Source prend `color` ; reste dans la Source | oui (dé lancé) | **oui** |
| `CrystalDeclined` | fin du choix de Rituel de sang | non | non |
| `AdvancedActionLearned { card, cost }` | Influence −`cost`, modificateur retiré | non | non |
| `CrystalsReturned { colors }` | cristaux remis dans l'inventaire (3 par couleur au plus) | non | non (la fin du tour l'est déjà) |
| `OfferCardGained` (existant, `to`) | la carte va sur le deck, dans la défausse ou dans la main | non | non (`OfferCardDealt`, qui suit, l'est) |

Hasard nouveau : les relances de Tempête de mana tirent dans le générateur de la Source (ADR 0007) ; les parcours qui ne la
jouent pas ne bougent pas.

## Comportements attendus

### Rituel de sang

- **C1** — Hors combat, Rituel de sang de base, option jeton vert : une Blessure en main, un cristal rouge, un jeton vert ; pas
  assommé (`blood_ritual_wound_crystal_and_token`) — carte (LL) ; Compendium, *Blood Ritual* S3.
- **C2** — En combat, héros à quatre Blessures reçues dans ce combat (comme Coup de sang, spec 053 C10) : la Blessure de Rituel
  de sang l'assomme (`blood_ritual_wound_counts_in_combat`) — Compendium, *Blood Ritual* S3.
- **C3** — De jour, option jeton noir : le jeton est gagné, il ne paie aucun effet (`blood_ritual_black_token_by_day`) —
  Compendium, *Blood Ritual* S2.
- **C4** — Renforcé (mana rouge) : les actions légales ne proposent que les six couleurs de jeton, trois fois ; puis le cristal
  (une option par couleur de base, payée par un mana de cette couleur) ou le refus. Avec les jetons vert, vert, or, payer avec un
  jeton vert donne un cristal vert, il reste un jeton vert et un or (`strong_blood_ritual_three_tokens_then_crystal`) — carte ;
  Compendium, *Blood Ritual* S1.
- **C5** — Après le refus ou le cristal, plus de cristal proposé (`blood_ritual_crystal_only_once`) — Compendium, S1.

### Décomposition

- **C6** — Inventaire : deux cristaux rouges. Décomposition de base en détruisant Rage (rouge) : un cristal rouge et un jeton
  rouge (le 4e) ; Rage n'est ni dans la défausse, ni dans la main, ni dans le deck, ni en fin de tour dans la défausse
  (`decompose_two_crystals_of_destroyed_color`) — carte ; MKUE p. 5, 2.b et *"Gain"* 2.
- **C7** — Renforcé en détruisant Marche (vert) : un cristal rouge, bleu, blanc (`strong_decompose_other_colors`) — carte.
- **C8** — Main : Décomposition, deux Blessures : pas une action légale (seulement de côté) (`decompose_needs_an_action_card`) —
  MKUE p. 4, 6.a.

### Maîtrise du cristal

- **C9** — Inventaire : un cristal bleu. Base : options cristal bleu seulement ; un second cristal bleu. Inventaire vide : pas
  une action légale (`crystal_mastery_needs_an_owned_color`) — carte.
- **C10** — Phase de mouvement, inventaire : un bleu, un vert. Renforcé payé par le cristal bleu, puis Marche renforcée payée par
  le cristal vert :
  à la fin du tour, les deux cristaux sont revenus (`crystal_mastery_returns_spent_crystals`) — FAQ p2.
- **C11** — Renforcé payé par un dé bleu, cristal vert dépensé **avant** la carte (Marche renforcée) : seul le vert revient
  (`crystal_mastery_counts_crystals_spent_before`) — FAQ p2 ; Compendium, *Crystal Mastery* Q1.
- **C12** — Retour en arrière avant le cristal dépensé : il ne revient pas ; sauvegarde rechargée avant la fin du tour : il revient
  (`crystal_mastery_follows_undo_and_reload`) — spec 024.

### Tempête de mana

- **C13** — Source : vert, bleu, noir (jour). Base payée sans mana, dé vert : cristal vert, `DieRerolled` pour ce dé, qui reste dans
  la Source ; le joueur peut encore prendre un dé ce tour ; le dé noir n'est pas proposé (`mana_storm_crystal_and_reroll`) — carte.
- **C14** — Après `DieRerolled`, le retour en arrière ne peut pas revenir avant lui (`mana_storm_reroll_cannot_be_undone`) — MKUE
  p. 5, *Reverting* 2.
- **C15** — Renforcé payé par le dé blanc : les deux autres dés sont relancés, pas le blanc ; ensuite quatre dés au plus ce tour
  (`strong_mana_storm_rerolls_and_three_more_dice`) — carte ; Compendium, *Mana Storm* S1.
- **C16** — Renforcé, de jour, un dé noir dans la Source : il se prend comme mana vert pour Marche renforcée ; de nuit, un dé or
  aussi (`strong_mana_storm_black_and_gold_any_color`) — carte ; Q9.

### Effet maximum

- **C17** — Phase d'attaque contre des Prowlers (`enemies.ron` : armure 3, attaque physique 4) : Effet maximum de base avec Rage,
  option Attaque : une Attaque physique 6 dans l'attaque en cours ; Rage détruite, Effet maximum dans la zone de jeu
  (`maximal_effect_triples_basic_effect`) — FAQ p1 ; Compendium, *Maximal Effect* S1, Q4.
- **C18** — Phase de mouvement, renforcé (mana rouge) avec Marche : Mouvement 8, Marche non payée
  (`strong_maximal_effect_doubles_strong_effect`) — carte.
- **C19** — Phase de parade, renforcé avec Détermination : une Parade 10, contre une seule attaque
  (`maximal_effect_block_is_one_effect`) — FAQ p2 ; Compendium, *Maximal Effect* S5.
- **C20** — Base avec Concentration, option jeton bleu : trois jetons bleus ; avec Cristallisation renforcée (renforcé), option
  cristal blanc : deux cristaux blancs (`maximal_effect_repeats_gains`) — FAQ p1 ; Q2.
- **C21** — Base avec Coup de sang, option avec Blessure, phase d'attaque : Attaque 15 et trois Blessures en main
  (`maximal_effect_with_blood_rage_three_wounds`) — Compendium, *Maximal Effect* S33.
- **C22** — Hors combat, base avec Tranquillité, option Piocher : trois cartes piochées (`maximal_effect_draws_three`) — carte.
- **C23** — Ni Improvisation, ni Agilité, ni Concentration renforcée, ni Rage hors combat ne sont proposées comme cible ;
  Effet maximum n'est pas une cible de Concentration renforcée (`maximal_effect_targets`) — Q1, Q3.

### Don pour la magie

- **C24** — Offre de sorts : Fireball (rouge), Snowstorm (bleu), Tremor (vert) (`spells.ron`). Don pour la magie renforcé (mana
  bleu), mana rouge en plus : Fireball dans la défausse, l'offre complétée ; option blanche absente (aucun sort blanc)
  (`strong_magic_talent_gains_spell_to_discard`) — carte ; Compendium, *Magic Talent* O4.
- **C25** — De jour, renforcé payé par un cristal bleu, le mana en plus par un dé or pour Tremor (`magic_talent_gold_by_day`) — MKUE p. 5.

### Apprentissage

- **C26** — Hors interaction, phase de mouvement : Apprentissage de base, Promesse, Menace : Influence 6 ; `LearnAdvancedAction` sur la
  première carte de l'offre : elle va dans la défausse, l'Influence tombe à 0, l'offre est complétée
  (`learning_buys_advanced_action_to_discard`) — carte ; Compendium, *Learning* S1.
- **C27** — Renforcé (mana blanc) avec Menace renforcée : Influence 9 ; la carte achetée arrive **en main** et se joue ce tour
  (`strong_learning_to_hand`) — carte.
- **C28** — Après un achat, plus de `LearnAdvancedAction` ce tour ; avec 5 Influence, pas d'achat ; en combat, pas d'achat
  (`learning_once_per_turn`) — carte ; Q7.

### Entraînement

- **C29** — Offre d'actions avancées : Path Finding (vert), Fire Bolt (rouge), Agility (blanc) ; main : Entraînement, Marche,
  Rage, Détermination. Base en détruisant Marche : Path Finding dans la défausse, Marche détruite, l'offre complétée ; Rage avec
  Fire Bolt aussi permis ; Détermination (bleu) n'est proposée avec aucune carte (`training_takes_same_color_to_discard`) — carte ;
  Compendium, *Training* S3.
- **C30** — Renforcé (mana vert) en détruisant Rage : Fire Bolt en main, jouable aussitôt (`strong_training_to_hand`) — carte.
- **C31** — Entraînement ne se détruit pas lui-même (`training_cannot_destroy_itself`) — Compendium, *Training* S2.

### Contenu et vue

- **C32** — Plus aucune action avancée `Unsupported`, sauf l'effet de base de Don pour la magie (`advanced_actions_all_supported`)
  — contenu.
- **C33** — La vue donne le choix en attente de Rituel de sang (jetons restant à choisir, cristal proposé) et, avec Maîtrise du
  cristal renforcée, les cristaux qui reviendront en fin de tour (`view_shows_mana_choice_and_crystals_to_return`).

## Cas limites et refus

- **L1** — Décomposition et Entraînement avec un sort ou un artefact en main comme seule « autre » carte : pas jouables.
- **L2** — `LearnAdvancedAction` sur une action avancée de l'offre d'unités (monastère) : refusée.
- **L3** — Effet maximum renforcé sans mana rouge : refusé ; sa cible ne se paie jamais.
- **L4** — Tempête de mana de base sans dé de couleur de base dans la Source : pas une action légale.
- **L5** — Entraînement ou Apprentissage avec l'offre d'actions avancées vide, Don pour la magie avec l'offre de sorts vide : pas
  d'option ; l'offre qui ne peut être complétée garde moins de cartes (spec 050, L1).

## Information cachée

Rien de nouveau de caché : les offres et la Source sont publiques, la carte détruite était dans la main du joueur. Champs ajoutés à
`PlayerView` : le **choix en attente** de Rituel de sang (`mana_choice` : jetons restant à choisir, cristal proposé) et les
**cristaux à rendre** en fin de tour (`crystals_to_return`) (C33). La carte détruite entre dans les faits du tour (spec 073,
`TurnFact::CardDestroyed`) ; les cartes gagnées y sont déjà (`CardGained`).

## Libellés (i18n)

Les textes `card.<carte>.basic_text` / `strong_text` existent (relus dans `assets/i18n/fr.json`) ; ceux de Décomposition,
Entraînement et Effet maximum disent « écarter » / « l'écarter » : remplacés par « détruire » selon Q4. Nouvelles clés (fr ; en à
l'avenant) :

- `effect.wound_crystal_and_token` : « Blessure ; cristal {crystal} et jeton {token} » ; `effect.wound_and_tokens` : « Blessure ;
  {count} jetons au choix » ; `effect.crystals_from_destroyed` : « Détruire une carte action : {count} cristaux de sa couleur » ;
  `effect.crystals_other_colors` : « Détruire une carte action : un cristal de chaque autre couleur » ; `effect.gain_owned_crystal` :
  « Cristal {color} (déjà possédé) » ; `effect.crystal_from_die` : « Dé {color} : cristal {color}, puis relancer le dé » ;
  `effect.reroll_source` : « Relancer la Source ; 3 dés de plus » ; `effect.repeat` : « Une autre carte action, {times} fois » ;
  `effect.gain_spell` : « Payer {color} : un sort {color} de l'offre » ; `effect.train` : « Détruire une carte action : action
  avancée de même couleur ». `{color}` reprend les noms de couleur de `effect.gain_crystal` (relu : « Cristal {color} »).
- `modifier.return_spent_crystals` : « cristaux dépensés rendus à la fin du tour » ; `modifier.learning` : « une fois : {cost}
  Influence pour une action avancée ({to}) », `{to}` = `destination.discard` « dans la défausse » ou `destination.hand` « en main ».
- `choice.token_color` : « Rituel de sang : couleur du jeton ({left} à choisir) » ; `choice.crystal` : « Rituel de sang : payer un
  mana de base pour un cristal ? » ; `choice.decline` : « Non merci ».
- `offer.learn` : « Apprendre ({cost} Influence) » (bouton de la carte dans l'offre, à côté des boutons d'achat de la spec 041).
- `turn_fact.card_destroyed` : « {card} détruite » (fr « Marche détruite », en « March destroyed ») ; `turn_fact.crystals_returned`
  : « Cristaux rendus : {count} ».

Ajouts de l'implémentation (fil écran, signalés dans la PR) : `effect.crystal_from_die` sans `{color}` (« Dé de la Source :
cristal de sa couleur, puis relancer le dé », l'effet ne connaît pas le dé avant le choix) ; le bouton « Apprendre » sous
`offers.learn` (section `offers` existante) ; `discard_choice.destroy_prompt` et `discard_choice.destroy` (« Choisissez la carte à
détruire », « Détruire »), `play_area.line_destroying` (« Rage détruite » dans la zone de jeu),
`concentration.maximal_prompt` (cible d'Effet maximum), `gain_choice.prompt` (carte à prendre dans la fenêtre Offres ; depuis la [spec #20](issue-20-obtention-des-cartes.md), dans une fenêtre de choix des seules cartes prenables, `gain_choice.title`),
`die_choice.prompt` (dé de Tempête de mana), `modifier.return_spent_crystals_list` (cristaux à rendre, C33).

Issue privée 234 : l'effet de base de Rituel de sang se lit sur la carte et dans les infobulles par `card.blood_ritual.basic_text`, pas
par les six options `effect.wound_crystal_and_token` jointes par « ou » ; chaque bouton d'option garde son texte calculé.

Issue privée 235 : dans la zone de jeu, la ligne d'Effet maximum nomme la cible détruite et l'effet obtenu, par `play_area.repeated`
(« Effet maximum : Marche 3 fois (Mouvement 6), détruite »), au lieu du texte de la carte (`effect.repeat`).

## Interface

- **Carte à détruire** (Décomposition, Entraînement) : choisie dans la main comme la carte à défausser d'Improvisation (spec 025,
  mode existant), seules les cartes permises cliquables (C8, C29, actions légales).
- **Carte d'une offre** (Entraînement, Don pour la magie) : choisie dans la fenêtre des offres, seules les cartes permises
  cliquables, comme l'action avancée d'un niveau pair (spec 050) (C24, C29, actions légales).
- **Cible d'Effet maximum** : comme la carte renforcée par Concentration (spec 025, C29), puis son option par les boutons d'options
  (spec 073) (C17, C23, actions légales).
- **Dé de Tempête de mana** : choisi dans la Source, seuls les dés permis cliquables (C13, actions légales).
- **Choix de Rituel de sang** : fenêtre de six couleurs, puis proposition du cristal avec la fenêtre de paiement de la spec 022 et
  « Non merci » (C4, C5, `PlayerView::mana_choice`).
- **Apprentissage** : bouton « Apprendre (6 Influence) » sur les cartes de l'offre d'actions avancées tant que l'achat est permis
  (C26, C28, actions légales).
- **Effets en cours** : `ReturnSpentCrystals` et `LearningDiscount` dans la zone de jeu (C10, C26, `PlayerView::modifiers`), avec
  les cristaux à rendre (C33, `PlayerView::crystals_to_return`). Placement : vérification à l'œil.
- **Faits du tour** : carte détruite, cristaux rendus (C6, C10, `TurnFact`).

### Raccourcis clavier

Aucun : tout passe par les boutons et les fenêtres existants.

## Effets sur les specs déjà implémentées

- **025** : Concentration ne prend pas Effet maximum pour cible ; `black_any_dice` vaut aussi pour l'or et les dés épuisés avec
  Tempête de mana.
- **050** : `OfferCardGained` porte sa destination ; 7 actions avancées et demie quittent `Unsupported`.
- **053** : `WoundFromCard` sert aussi à Rituel de sang ; `extra_mana` à Don pour la magie.
- **016, 024** : fin du tour avec `CrystalsReturned` ; `DieRerolled` est un point de non-retour.

## Hors périmètre

- Don pour la magie de base (jouer un sort de l'offre) : spec 054.
- Effet maximum sur une option non répétable (Q1) et avec des options différentes à chaque répétition (Q2) : reportés, issue #6.
- Cartes des extensions (Attaque rituelle, Montée d'adrénaline…).

## Plan de développement

Moteur et écran (issue privée 118) : deux fils sur la même branche, plan
[`plans/056-actions-avancees-mana.md`](plans/056-actions-avancees-mana.md) :

1. **Moteur** : détruire une carte, cartes gagnées hors du deck, chaque carte, choix en attente, contenu, vue, persistance (C1 à
   C33).
2. **Écran** : libellés, choix dans la main, les offres et la Source, fenêtre de Rituel de sang, Apprentissage, vérification à
   l'œil, docs, bilan.

## Décisions (2026-11-12)

Mathieu accepte toutes les réponses par défaut (« Ok avec défaut »), appliquées dans le texte ci-dessus :

1. **Q1 — Cibles d'Effet maximum.** Le compendium (S6 à S34) règle des dizaines de combinaisons : défausses multipliées
   (Improvisation), modificateurs doublés (Embuscade, Au cœur du combat), Concentration en chaîne, Entraînement répété… Proposé :
   seulement les options **répétables** (liste de la section Effet maximum), qui couvrent les cartes de points et de gain ; les
   autres reportées dans une issue. **Décision : oui** (issue #6).
2. **Q2 — Une option par répétition.** La FAQ (p1) permet deux couleurs avec Cristallisation renforcée ×2. Proposé : **une seule
   option** répétée (deux cristaux de la même couleur), plus simple à choisir et à afficher ; l'autre lecture rejoint l'issue de
   Q1. **Décision : une seule option** (issue #6).
3. **Q3 — Effet maximum et Concentration.** Proposé : ni l'une cible de l'autre (comme Concentration sur Concentration, spec 025,
   Q5) ; le gain serait nul (Compendium, S13) ou demanderait une chaîne de cibles. **Décision : oui.**
4. **Q4 — Terme de *throw away*.** Le glossaire dit « Jeter » pour une Blessure (FAQ p3) ; les textes de cartes écrits par nous
   (spec 050) disent « écarter » ; les cartes françaises disent « **Détruisez** une carte Action ». Proposé : **détruire** pour une
   carte action (`CardDestroyed`), « jeter » gardé pour une Blessure ; les trois textes corrigés. **Décision : détruire.**
5. **Q5 — Cristal de Rituel de sang renforcé.** Proposé : proposé **aussitôt** après les trois jetons, une fois, refusable, payable
   avec un des jetons gagnés (lecture stricte de S1). **Décision : oui.**
6. **Q6 — Entraînement strict.** Le compendium (S3) exige une carte de même couleur dans l'offre et de la prendre ; d'autres
   joueurs laissent détruire sans prendre. Proposé : la lecture stricte. **Décision : stricte.**
7. **Q7 — Moment de l'achat d'Apprentissage.** Proposé : **hors combat**, après la carte, une fois, tant que l'Influence de la
   réserve sert (phase de mouvement ou interaction ; perdue comme aujourd'hui au début de l'action, specs 040 et 052). **Décision : oui.**
8. **Q8 — Cristaux rendus au-delà de 3.** Avec des cristaux gagnés entre-temps, le retour peut dépasser 3 d'une couleur. Proposé :
   au plus 3, le surplus perdu (un jeton rendu en fin de tour ne servirait à rien). **Décision : au plus 3.**
9. **Q9 — Dés de Tempête de mana renforcée.** Proposé : relancer les dés **encore dans la Source** (pas ceux pris ce tour) ; ensuite
   un dé noir ou or, épuisé ou non, vaut n'importe quelle couleur de base, et un dé noir de nuit peut encore servir de noir. **Décision : oui.**

## Questions ouvertes

Aucune.
