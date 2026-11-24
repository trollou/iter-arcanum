# 061 — Joueur fantôme

- Statut : implémentée (le 2026-11-19 ; validée le même jour ; Q1, Q3 à Q6 : propositions acceptées ; Q2 : héros du fantôme au choix dans les Options, « Au hasard » par défaut) (reporter tout changement dans la colonne Statut de `PLAN.md`)
- Jalon : J6
- Dépend de : 060 (tactiques, `TacticRule`, retrait), 023 (fin de manche, annonce, nouvelle manche), 050 (offres et leur
  renouvellement), 051 (compétences, offre de compétences communes, seconde option du niveau pair), 057 (Distorsion temporelle),
  063 (Première reconnaissance, dernier tour après la cité), 024 (retour en arrière), 038 (générateurs par usage), 065 et 078
  (écran de nouvelle partie) ; toutes implémentées
- Livret : MKUE p. 4 (2.d, 2.e, 2.g, 3.a à 3.d, 4.a à 4.f) ; p. 6 (*Start of the turn*, 3 et 4) ; p. 15 (*Dummy Player*,
  *Skills in a Solo Game*) ; p. 18 (*First Reconnaissance*, *Scenario End*, *Solo Variant*) ; p. 19 (*Solo Conquest*, *Special
  Rules*, *Scenario End*). FAQ p1 (*Mise en place du jeu* : les trois cercles colorés, dés et unités en solo), FAQ p3
  (renouvellement des offres avec un joueur fantôme). Compendium : *Dummy Player* S1 à S3, *Setup* S1, *Miscellaneous* S2,
  *Scenarios* Q8, *Dual-Colored Action Cards* S3, *Time Bending* S2, *The Right Moment* S3
- Crates touchées : arcanum-core, arcanum-persistance (tests), arcanum-i18n (libellés), arcanum-ui

## Objectif

Première reconnaissance se joue comme la variante solo du livret : un joueur fantôme, automate tiré parmi les trois autres héros,
donne le tempo de la manche. À chaque manche, il tire une tactique parmi celles que le joueur a laissées, ce qui fixe qui joue en
premier ; à son tour, il retourne trois cartes de son deck, plus une par cristal de la couleur de la dernière, et annonce la fin de
la manche quand son deck est vide : le joueur n'a plus alors qu'un dernier tour. Son deck grossit de l'action avancée en dernière
place à chaque nouvelle manche, et ses cristaux d'un cristal de la couleur du sort en dernière place. Ses compétences remplissent
l'offre de compétences communes, où le joueur peut les prendre (seconde option du niveau pair, spec 051).

Le joueur voit en permanence le deck du fantôme (nombre de cartes), sa défausse, ses cristaux, sa tactique et ce qu'il a retourné
à son dernier tour, pour régler son propre tempo (MKUE p. 15, *Note 1*). Une option de l'écran de nouvelle partie permet de jouer
sans lui, comme le livret le permet pour une première partie.

### Le tour du fantôme, en schéma

```
début du tour du fantôme
  ├─ deck vide ? ── oui ──► il annonce la fin de la manche : le joueur joue un dernier tour, puis la manche finit
  └─ non : retourner 3 cartes (moins si le deck en a moins) du deck vers sa défausse
             couleur de la dernière carte retournée = C
             └─ n cristaux C dans son inventaire (n ≥ 0) : retourner encore n cartes (ce que le deck peut)
           fin de son tour (rien d'autre : ni Source, ni plateau, ni offre)
```

Exemple (cas C11) : fantôme Goldyx (cristaux vert, vert, bleu). Il retourne Rage, Célérité, Marche ; Marche est verte et il
a 2 cristaux verts : il retourne encore Endurance et Promesse. 5 cartes retournées, son deck passe de 16 à 11.

## Règles

### Mise en place

- **Héros du fantôme** (MKUE p. 15, *At the start of the game*, 1er point) : tiré au hasard parmi les trois héros que le joueur
  n'a pas pris, ou choisi parmi eux dans les Options de la nouvelle partie (Q2, Mathieu : liste « Au hasard » par défaut, puis
  les trois héros restants). Le fantôme n'a ni figurine, ni main, ni niveau, ni unité, ni Source : seulement un deck, une défausse, un
  inventaire de cristaux, une tactique et une pile de compétences.
- **Son deck** (p. 15, 1er point : *starting deck of 16 Basic Action cards*) : le deck de départ de son héros, ses deux cartes
  propres comprises (`heroes.ron`), mélangé (3e point).
- **Ses cristaux** (p. 15, 2e point ; FAQ p1 ; Compendium, *Dummy Player* S2) : un cristal par cercle coloré au bas de sa carte de
  héros. Lu sur les scans `docs/regles/scans/characters/<héros>/character_<héros>.jpg` (le livret donne Goldyx en exemple) :

  | Héros | Cercles |
  |---|---|
  | Arythea | rouge, rouge, blanc |
  | Goldyx | vert, vert, bleu |
  | Norowas | blanc, blanc, vert |
  | Tovak | bleu, bleu, rouge |

  Ce ne sont que des marqueurs (S2) : la limite de 3 par couleur ne s'applique pas (p. 15, *When preparing a new Round*, 2e
  point), et le joueur, lui, part toujours sans cristal (inchangé).
- **Ses compétences** (p. 15, *Skills in a Solo Game* ; Compendium, *Dummy Player* S1) : les dix compétences de son héros dans
  `skills.ron` (la coopérative gardée, la compétitive absente, comme pour le joueur, spec 051 et Compendium, *Skill Tokens* S5),
  mélangées en pile face cachée.
- **Ne compte pas comme un joueur** (p. 15, *Note* ; FAQ p1 ; Compendium, *Setup* S1) : 3 dés dans la Source et 3 unités dans
  l'offre, comme aujourd'hui (spec 020, `source_size(1)` ; spec 050, *actual players plus 2*).
- **Scénarios** (Q1) : Première reconnaissance a un joueur fantôme (MKUE p. 18, *Solo Variant* : *Use a Dummy player*), que
  l'écran de nouvelle partie permet de retirer (*you may also ignore the Dummy player in your first game*) ; le bac à sable n'en a
  pas. Le scénario dit s'il a un fantôme (point d'extension, comme `TacticRule`).

### Tactiques et ordre du tour

- **Sa tactique** (MKUE p. 19, *Special Rules*, 1er point ; p. 18, *Solo Variant*) : le joueur choisit la sienne d'abord
  (spec 060, inchangé), puis le fantôme en tire une **au hasard parmi celles qui restent** (ni retirées, ni prise par le joueur).
  Il n'en tire aucun effet (p. 15, *When picking Tactics*, 2e point) : elle ne sert qu'à l'ordre du tour.
- **Ordre du tour** (p. 4, 3.c et 4.a) : le plus petit numéro de tactique joue en premier, puis on alterne. Le tirage du fantôme
  suit la prise du joueur (3.a) ; les effets « quand vous prenez cette tactique » du joueur viennent ensuite (3.b : Départ
  flamboyant, et les choix de Faux départ, Vol de mana, Préparation, spec 060), puis les tours (4).
- **Retrait** (p. 19, 2e point ; Compendium, *Scenarios* Q8) : à la fin de la manche, les **deux** tactiques prises sont
  retirées de la partie. C'est déjà la règle `TacticRemoval::AllTaken` de la spec 060, qui retire toutes les tactiques prises :
  elle compte maintenant celle du fantôme. Avec 6 tactiques par moment de la journée et 4 manches, il en reste toujours au moins
  une pour le fantôme (manche 3 : 4 de Jour restantes ; manche 4 : 4 de Nuit).
- **Effets de tactique liés à l'ordre** : Au bon moment (*The Right Moment*) donne un tour de plus « juste après celui-ci » :
  le tour du fantôme ne s'intercale pas (texte de la carte, `docs/regles/cartes/tactiques.md`). De même pour Distorsion temporelle (spec
  057). Lève-tôt et Au crépuscule n'ont toujours pas d'autre effet que l'ordre (spec 060).

### Le tour du fantôme

- **Déroulé** (MKUE p. 15, *When it is the Dummy player's turn* ; schéma ci-dessus) : deck vide au début de son tour, il annonce
  la fin de la manche ; sinon il retourne 3 cartes du dessus de son deck dans sa défausse, regarde la couleur de la dernière, et
  retourne autant de cartes en plus qu'il a de cristaux de cette couleur. La couleur des cartes en plus ne compte pas.
- **Deck trop court** (p. 15, *Note* ; Compendium, *Dummy Player* S3) : il retourne ce qu'il peut, et n'annonce la fin de la
  manche qu'au début de son tour suivant, jamais pendant celui où il vide son deck.
- **Couleur d'une carte** : la couleur de la carte (`color` de `basic_actions.ron` et `advanced_actions.ron`). Le jeu de base n'a
  pas de carte bicolore (Compendium, *Dual-Colored Action Cards* S3 : la couleur dont il a le plus de cristaux) : hors périmètre.
- **Automatique** : le tour du fantôme n'est la décision de personne. Il est joué par le moteur dans la décision qui le précède
  (la fin du tour du joueur, la prise de la tactique ou l'annonce de la fin de la manche) ; `pending_decision` ne désigne jamais
  le fantôme.

### Fin de la manche

- **Annoncée par le joueur** (MKUE p. 6, 3 ; p. 4, 4.b) : *each other player takes one more turn* : le fantôme joue un dernier
  tour (il retourne des cartes, ou rien si son deck est vide), puis la manche finit. Ce dernier tour ne change rien en Première
  reconnaissance, mais il est joué et montré : il comptera dans le score de Conquête en solitaire (spec 064, *1 point par carte
  du deck du fantôme*).
- **Annoncée par le fantôme** (p. 15 ; Compendium, *Dummy Player* S3) : le joueur joue un dernier tour ; à sa fin, après le
  changement de niveau et les récompenses (spec 016), la manche finit, sans pioche (les cartes sont remélangées de toute façon,
  p. 4, 2.g ; une augmentation de la prochaine pioche passe à la nouvelle manche, p. 4, 4.f, comme aujourd'hui avec
  `draw_bonus`). Pendant ce dernier tour, `AnnounceEndOfRound` n'est pas légale (p. 6, 3 : *if the End of the Round has not been
  announced yet*).
- **Tour en plus pendant le dernier tour** (Compendium, *Time Bending* S2 ; *The Right Moment* S3 par analogie) : un tour
  supplémentaire de Distorsion temporelle ou d'Au bon moment se joue avant la fin de la manche, comme avant la fin de la partie
  aujourd'hui (spec 060, `turn_end_draw`).
- **Rien en main ni dans le deck** quand le fantôme a annoncé (p. 6, 3.d et 4) : le joueur doit passer son tour, qui finit
  aussitôt ; la manche finit dans la même décision (Q5).
- **Après le dernier tour de la manche** (p. 4, 4.e) : rien ne se joue plus jusqu'à la nouvelle manche ; c'est déjà le cas, la
  fin de la manche s'enchaîne dans la même décision.

### Nouvelle manche

- **Action avancée en dernière place** (MKUE p. 15, *When preparing a new Round*, 1er point ; p. 4, 2.d ; FAQ p3) : elle va dans
  le deck du fantôme au lieu du dessous de la pioche. L'offre se décale et une carte entre en première place, comme aujourd'hui
  (spec 050). Offre vide : rien n'est ajouté.
- **Sort en dernière place** (p. 15, 2e point ; p. 4, 2.e) : il va sous la pioche comme aujourd'hui, et le fantôme gagne un cristal
  de la couleur de ce sort, sans limite. Offre vide : pas de cristal.
- **Mélange** (p. 4, 2.g ; p. 15, 1er point) : le fantôme reprend toutes ses cartes (deck et défausse, action avancée ajoutée
  comprise) et les mélange.
- **Ordre** : celui du livret, p. 4, 2.d (actions avancées), 2.e (sorts), puis 2.g (mélanges) ; ensuite le choix des tactiques
  (3).

### Compétences du fantôme

- **Révélée quand le joueur en gagne une** (MKUE p. 15, *Skills in a Solo Game*, 2e paragraphe) : chaque fois que le joueur gagne
  une compétence (niveau pair, spec 051), la compétence du dessus de la pile du fantôme est révélée et posée dans l'offre de
  compétences communes, **après** le choix du joueur : elle n'est disponible qu'au prochain niveau pair (*available to you the
  next time you may gain a Skill token*). Pile vide (dix compétences, cinq niveaux pairs au plus) : rien.
- **La prendre** : c'est la seconde option du niveau pair de la spec 051 (compétence d'un autre héros dans l'offre commune, et
  l'action avancée en dernière place), codée et testée sur une offre préparée par la 051 (Q4 de la 051) ; la 061 la rend
  atteignable en partie. Rien ne change dans ses règles.

### Fin de la partie

- **Dernier tour après la cité** (MKUE p. 18, *Scenario End* ; p. 19, *Scenario End*, par analogie : *the Dummy player does not*)
  : une fois le dernier tour annoncé par la révélation d'une cité (spec 063, `LastTurn`), le fantôme ne joue plus ; le tour du
  joueur qui suit est le dernier (Q3).
- **La manche finit pendant ce délai** (p. 18 : *If the Round ends during this, the game ends immediately*) : si le fantôme avait
  déjà annoncé la fin de la manche et que le joueur révèle la cité pendant son dernier tour de la manche, la manche finit à la fin
  de ce tour, donc la partie aussi (raison `CityRevealed`) (Q3).
- **Décompte** (p. 18, *Solo Variant* ; spec 063) : inchangé ; le fantôme ne marque rien en Première reconnaissance.

## Modèle

Termes du glossaire : Joueur fantôme (`DummyPlayer`), Tactique, Ordre du tour, Manche, Tour, Deck, Défausse, Inventaire,
Cristal, Offre, Action avancée, Sort, Compétence, Offre de compétences communes, Dernière place, Passer son tour. **Termes
nouveaux** (ajoutés au glossaire par cette PR) :

| Livret (EN) | Français | Code |
|---|---|---|
| Round Order (token) | Ordre du tour | `GameState::round_order` (remplace « spec 061 ») |
| Dummy player's turn | Tour du fantôme | `Event::DummyTurnPlayed` |
| Flip over (cards from the Dummy's Deed deck) | Retourner (des cartes du deck du fantôme) | `DummyTurnPlayed::cards` |
| Forfeit (one's turn) | Passer son tour | `AnnounceEndOfRound`, et la fin de manche automatique de L5 |

Données nouvelles :

- **Contenu** : les cercles colorés de chaque héros, `dummy_crystals` dans `heroes.ron` (trois couleurs de base par héros,
  tableau ci-dessus), lus par `GameContent::parse` et validés (trois couleurs de base exactement, C2).
- **Scénario** : `dummy: DummyRule`, `None` (bac à sable, et une sauvegarde d'avant cette spec : valeur par défaut serde) ou
  `Standard` (Première reconnaissance). Une constructrice garde l'option de l'écran de nouvelle partie
  (`Scenario::without_dummy`, nom au fil d'implémentation). Un fantôme suppose des tactiques : `Standard` avec
  `TacticRule::None` est refusé à la création (L9).
- **Partie** : `dummy: Option<DummyState>` (absent sans fantôme) : héros, deck (ordre caché), défausse, cristaux par couleur
  (sans limite), tactique de la manche, pile de compétences, cartes retournées à son dernier tour, fin de manche annoncée par lui.
  `round_order` : qui joue en premier dans la manche, déduit des deux tactiques (joueur ou fantôme), lu par la vue.
- **Identité des cartes** : le fantôme a ses propres exemplaires des cartes de base ; leurs `CardId` ne doivent pas se confondre
  avec ceux du joueur (numéros de copie distincts, choix du fil d'implémentation), pour que l'historique et l'écran ne mélangent
  pas une Rage du joueur et une Rage du fantôme.
- **Configuration** (Q2) : `GameConfig` gagne le choix du héros du fantôme, `dummy_hero: DummyHeroChoice`, `Random` (par défaut,
  et dans une sauvegarde d'avant cette spec) ou `Hero(h)`. Il n'a d'effet que si le scénario a un fantôme. Un héros du fantôme
  égal à celui du joueur est refusé par une validation typée de la configuration (`ConfigError::DummyHeroIsPlayerHero`, nom au
  fil d'implémentation) (L10).
- **Hasard** : un usage nouveau, `RngUsage::Dummy` (flux 6, spec 038), pour, dans cet ordre : le héros du fantôme (seulement
  avec `Random` : un héros choisi ne tire rien), sa pile de
  compétences et son deck à la mise en place ; puis, à chaque manche, le mélange de son deck (2.g) et le tirage de sa tactique
  (3.a). Ses tours n'utilisent pas le hasard : ils lisent l'ordre du deck mélangé. Ajouter le fantôme ne change ni la carte, ni la
  Source, ni le deck du joueur d'une même graine (C3).

## Actions et événements

Pas d'action nouvelle : le fantôme ne décide de rien. Les actions existantes produisent ses événements.

| Action | Validation (refusée si…) | Événements produits |
|---|---|---|
| (mise en place) | — | en plus, après les événements de la spec 060 : `DummySetUp` |
| `ChooseTactic` (existante) | inchangée | `TacticChosen`, puis `DummyTacticDrawn`, puis comme aujourd'hui ; avant le `TurnStarted` du tour 1 (là où la spec 060 le place, ici ou après le choix qui suit la prise), `DummyTurnPlayed` si le fantôme joue en premier |
| `EndTurn` (existante) et la fin du tour qui suit (pioche, Longue Nuit, récompenses) | inchangée | avant chaque `TurnStarted` d'un tour suivant (pas d'un tour supplémentaire) : `DummyTurnPlayed` ou `DummyAnnouncedEndOfRound` ; dernier tour de la manche : pas de pioche, la fin de la manche (`RoundEnded` …) |
| `AnnounceEndOfRound` (existante) | en plus, refusée si le fantôme a annoncé | `DummyTurnPlayed` (son dernier tour, `cards` vide si son deck l'est), puis `RoundEnded` et la suite comme aujourd'hui |
| Nouvelle manche (dans la décision qui finit la manche) | — | à la place du `OfferCardReturned` de l'action avancée : `DummyCardAdded` ; après celui du sort : `DummyCrystalGained` ; après le mélange du joueur : `DummyDeckShuffled` |
| Choix d'une compétence au niveau pair (existant, spec 051) | inchangée | en plus, après le choix : `DummySkillRevealed` |

| Événement | Effet de `apply` | Révèle une information ? | Point de non-retour ? |
|---|---|---|---|
| `DummySetUp { hero, crystals, skills, deck }` | le fantôme entre en jeu | le héros et les cristaux oui ; l'ordre du deck et de la pile non (filtrés) | oui (mise en place) |
| `DummyTacticDrawn { tactic }` | tactique du fantôme ; ordre du tour | oui (hasard) | oui |
| `DummyTurnPlayed { cards }` | les cartes passent du dessus de son deck à sa défausse ; elles deviennent ses « cartes du dernier tour » | oui (cartes du deck caché) | oui |
| `DummyAnnouncedEndOfRound` | le tour du joueur qui suit est le dernier de la manche | non (public) | non (suit un événement qui l'est) |
| `DummyCardAdded { card }` | l'action avancée en dernière place quitte l'offre pour son deck | non | oui (avec `RoundEnded`) |
| `DummyCrystalGained { color }` | un cristal de plus, sans limite | non | oui (avec `RoundEnded`) |
| `DummyDeckShuffled { order }` | deck et défausse réunis dans cet ordre ; cartes du dernier tour effacées | ordre caché (filtré) | oui |
| `DummySkillRevealed { skill }` | la compétence du dessus de sa pile va dans l'offre commune | oui (tirée d'une pile cachée) | oui |

`TacticsRemoved` (spec 060) compte la tactique du fantôme ; `RoundEnded`, `TurnStarted`, `GameEnded` restent ce qu'ils sont.

## Comportements attendus

Sauf mention, la partie est Première reconnaissance avec Tovak (`Scenario::first_reconnaissance`, `heroes.ron`), en mise en
place arrangée quand le cas dit le héros ou l'ordre du deck du fantôme (ADR 0006, issue privée 318 : l'arrangement choisit le héros du
fantôme parmi les trois et échange des cartes de son deck ; il apprend `DummySetUp`), en état posé seulement là où le cas le dit.
Couleurs des cartes lues dans `basic_actions.ron` : Rage rouge, Détermination bleue, Célérité blanche, Marche verte, Endurance
bleue, Tranquillité verte, Promesse blanche, Menace rouge, Cristallisation bleue, Attraction du mana blanche, Concentration verte,
Improvisation rouge ; Goldyx : Pouvoir de concentration vert, Euphorie cristalline bleue (noms français selon
`assets/i18n/fr.json`, relus au fil écran).

### Mise en place

- **C1** — Première reconnaissance avec Tovak : le fantôme est Arythea, Goldyx ou Norowas, jamais Tovak ; sur 30 graines, les
  trois sortent (`dummy_hero_is_one_of_the_three_others`) — MKUE p. 15.
- **C1 bis** — Même partie, héros du fantôme choisi : Goldyx ; le fantôme est Goldyx sur toutes les graines, avec le deck, les
  cristaux et la pile de compétences de Goldyx (C3) (`chosen_dummy_hero_is_used`) — Q2.
- **C2** — `heroes.ron` donne les cercles du tableau des règles ; `parse` refuse un héros sans exactement trois couleurs de base
  (`dummy_crystals_are_three_basic_colors`) — MKUE p. 15 ; scans des cartes de héros.
- **C3** — Fantôme Goldyx : 16 cartes au deck (le deck de départ de Goldyx, Pouvoir de concentration et Euphorie cristalline
  comprises), défausse vide, cristaux 2 verts et 1 bleu, pile de 10 compétences de Goldyx dont Embrasure de la Source
  (coopérative), sans sa compétitive (*Source Freeze*, absente de `skills.ron`). Le joueur n'a aucun cristal. La carte, la Source, le deck et la pile de
  compétences du joueur sont ceux de la même graine sans fantôme (`dummy_setup_uses_its_own_generator`) — MKUE p. 15 ; spec 038.
- **C4** — Avec fantôme : 3 dés dans la Source, 3 unités dans l'offre (`dummy_is_not_counted_as_a_player`) — MKUE p. 15, *Note* ;
  Compendium, *Setup* S1.
- **C5** — Bac à sable : pas de fantôme ; Première reconnaissance sans fantôme (option) : pas de fantôme, et la partie se joue
  comme avant cette spec (tests de la spec 063 et 060 inchangés) (`no_dummy_without_the_option`).

### Tactiques et ordre du tour

- **C6** — Manche 1, le joueur prend Planification (4) : `TacticChosen` puis `DummyTacticDrawn` d'une tactique de Jour autre que
  Planification ; sur 30 graines, chacune des cinq autres sort (`dummy_draws_among_remaining_tactics`) — MKUE p. 19, *Special
  Rules*.
- **C7** — Le joueur prend Lève-tôt (1) : le joueur joue en premier ; aucun `DummyTurnPlayed` avant le `TurnStarted` du tour 1 ; à
  la fin du tour 1, `DummyTurnPlayed` puis `TurnStarted { turn: 2 }` (`player_first_with_lower_tactic`) — MKUE p. 4, 3.c et 4.a.
- **C8** — Le joueur prend Au bon moment (6) : le fantôme joue en premier ; la décision `ChooseTactic` produit `TacticChosen`,
  `DummyTacticDrawn`, `DummyTurnPlayed`, `TurnStarted { turn: 1 }` ; avec Faux départ (2) pris par le joueur et le fantôme tiré
  sur Lève-tôt (1) (arrangé), `DummyTurnPlayed` vient dans la décision `RethinkDiscard`, avant `TurnStarted`
  (`dummy_first_with_lower_tactic`) — MKUE p. 4, 3.b et 3.c.
- **C9** — À la fin de la manche 1, `TacticsRemoved` contient la tactique du joueur et celle du fantôme ; en manche 3, aucune des
  deux n'est proposée au joueur ni tirée par le fantôme (`both_tactics_are_removed`) — MKUE p. 19, *Special Rules* ; Compendium,
  *Scenarios* Q8.
- **C10** — Au bon moment annoncé, puis fin du tour : le tour de plus suit sans `DummyTurnPlayed` entre les deux ; le fantôme joue
  après le tour de plus. Idem pour Distorsion temporelle (`extra_turn_skips_the_dummy`) — carte *The Right Moment* ; spec 057.

### Le tour du fantôme

- **C11** — Fantôme Goldyx (cristaux 2 verts, 1 bleu), deck arrangé Rage, Célérité, Marche, Endurance, Promesse, … : son tour
  retourne Rage, Célérité, Marche (verte : 2 cristaux) puis Endurance et Promesse ; `DummyTurnPlayed { cards }` en contient 5,
  son deck 11, sa défausse 5 (`dummy_flips_three_plus_matching_crystals`) — MKUE p. 15.
- **C12** — Même fantôme, la 3e carte est Rage (rouge, aucun cristal rouge) : 3 cartes retournées seulement
  (`dummy_flips_three_without_matching_crystal`) — MKUE p. 15.
- **C13** — Même fantôme, la 3e carte est Endurance (bleue, 1 cristal) : 4 cartes (`dummy_flips_one_per_crystal`) — MKUE p. 15.
- **C14** — Deck du fantôme à 2 cartes (état posé : `DummyTurnPlayed` écrits par le test jusqu'à 2 cartes, la mise en place ne
  donnant pas ce deck) : il retourne les 2 et n'annonce rien ; à son tour suivant, `DummyAnnouncedEndOfRound`
  (`dummy_announces_only_on_the_next_turn`) — MKUE p. 15, *Note* ; Compendium, *Dummy Player* S3.
- **C15** — Deck du fantôme à 4 cartes, la 3e verte avec 2 cristaux verts : il retourne les 4, pas plus
  (`dummy_flips_what_the_deck_has`) — MKUE p. 15, *Note*.
- **C16** — `pending_decision` ne désigne jamais le fantôme ; `legal_actions` n'a aucune action pour lui ; le joueur « premier
  choix » joue Première reconnaissance avec fantôme jusqu'à la fin de la partie sur 20 graines sans blocage
  (`dummy_never_has_a_decision`) — ADR 0002.

### Fin de la manche

- **C17** — Le fantôme annonce : le tour du joueur qui suit est le dernier ; `AnnounceEndOfRound` n'est pas légale, `decide` la
  refuse ; à la fin de ce tour, pas de `CardDrawn` de pioche, puis `RoundEnded`, la nouvelle manche et le choix de la tactique
  (`dummy_announcement_gives_one_last_turn`) — MKUE p. 15 ; p. 6, 3.
- **C18** — Le joueur gagne un niveau pendant ce dernier tour : le changement de niveau et son choix se font avant `RoundEnded`
  (`last_turn_still_levels_up`) — MKUE p. 11, 6 ; spec 035.
- **C19** — Méditation (spec 057) jouée au dernier tour : son bonus de pioche sert à la pioche de la nouvelle manche
  (`draw_bonus_carries_to_next_round`) — MKUE p. 4, 4.f.
- **C20** — Distorsion temporelle jouée au dernier tour de la manche : le tour supplémentaire se joue, puis la manche finit
  (`time_bending_on_last_turn_of_round`) — Compendium, *Time Bending* S2.
- **C21** — Le joueur annonce (deck vide) : `DummyTurnPlayed` (son dernier tour), puis `RoundEnded` ; si le deck du fantôme est
  vide, `DummyTurnPlayed { cards: [] }`, sans `DummyAnnouncedEndOfRound` (`player_announcement_gives_dummy_one_last_turn`) —
  MKUE p. 6, 3.

### Nouvelle manche

- **C22** — Offre d'actions avancées arrangée avec Boule de feu en dernière place (`advanced_actions.ron`, FireBolt, rouge) : à la nouvelle
  manche, `DummyCardAdded { FireBolt }` au lieu de `OfferCardReturned` ; la pioche d'actions avancées ne la reçoit pas ; l'offre a
  une nouvelle carte en première place (`lowest_advanced_action_joins_dummy_deck`) — MKUE p. 15 ; p. 4, 2.d ; FAQ p3.
- **C23** — Offre de sorts arrangée avec Éclair de feu (`spells.ron`, Fireball, rouge) en dernière place : `OfferCardReturned` du
  sort sous la pioche, puis `DummyCrystalGained { Red }` ; un fantôme Arythea passe de 2 à 3 cristaux rouges, puis à 4 la manche
  suivante avec un autre sort rouge, au-delà de la limite du joueur (`dummy_gains_crystal_of_lowest_spell`) — MKUE p. 15.
- **C24** — À la nouvelle manche, le deck du fantôme réunit deck, défausse et l'action avancée ajoutée : 17 cartes après la manche
  1, défausse vide, cartes du dernier tour effacées (`dummy_reshuffles_everything`) — MKUE p. 4, 2.g ; p. 15.
- **C25** — Offre d'actions avancées vide : pas de `DummyCardAdded` ; offre de sorts vide : pas de `DummyCrystalGained` (état posé :
  les offres vidées par des événements écrits, la partie ne les vide pas en quatre manches) (`empty_offers_add_nothing_to_dummy`).

### Compétences

- **C26** — Le joueur atteint le niveau 2 et garde une de ses deux compétences : l'offre commune reçoit l'autre (spec 051), puis
  `DummySkillRevealed` en ajoute une du fantôme ; la pile du fantôme passe de 10 à 9 (`dummy_skill_revealed_after_skill_gain`) —
  MKUE p. 15, *Skills in a Solo Game*.
- **C27** — Au niveau 4, la seconde option propose la compétence du fantôme révélée au niveau 2 (et la compétence écartée du
  joueur n'est toujours pas proposée, spec 051) ; la prendre donne l'action avancée en dernière place (spec 051, C7) ; ensuite
  une autre compétence du fantôme est révélée (`player_can_take_dummy_skill`) — MKUE p. 15 ; spec 051.
- **C28** — Au niveau 2, la compétence du fantôme révélée à ce niveau n'est pas proposée dans ce même choix (elle arrive après)
  (`dummy_skill_not_available_same_level`) — MKUE p. 15 (*the next time*).

### Fin de la partie

- **C29** — Une cité révélée pendant le tour du joueur (spec 063) : à la fin de ce tour, pas de `DummyTurnPlayed` ; le tour suivant
  du joueur est le dernier, puis `GameEnded { CityRevealed }` (`dummy_does_not_play_after_city_revealed`) — MKUE p. 18 et p. 19,
  *Scenario End*.
- **C30** — Le fantôme a annoncé la fin de la manche ; pendant son dernier tour, le joueur révèle la cité : à la fin de ce tour,
  `RoundEnded` puis `GameEnded { CityRevealed }`, sans nouvelle manche (état posé : `DummyAnnouncedEndOfRound` écrit, la cité à
  une case par la mise en place arrangée) (`round_ending_during_last_turns_ends_game`) — MKUE p. 18, *Scenario End*.

### Vue, retour en arrière, persistance

- **C31** — La vue du joueur montre le fantôme (`PlayerView::dummy`) : héros, tactique de la manche, nombre de cartes du deck,
  défausse entière (cartes), cristaux par couleur, cartes retournées à son dernier tour, fin de manche annoncée par lui, nombre de
  compétences dans sa pile ; et l'ordre du tour (`PlayerView::round_order`) ; jamais l'ordre de son deck ni de sa pile
  (`view_shows_dummy_without_deck_order`) — MKUE p. 15, *Note 1* ; Compendium, *Miscellaneous* S2.
- **C32** — Le journal filtré ne donne pas l'ordre de `DummySetUp::deck`, `DummySetUp::skills` ni `DummyDeckShuffled::order`
  (`log_hides_dummy_deck_order`) — spec 024 et ADR 0001.
- **C33** — Après `DummyTurnPlayed`, un retour en arrière ne remonte pas avant lui ; après `DummyTacticDrawn` non plus
  (`dummy_turn_blocks_undo`) — spec 024.
- **C34** — Sauvegarde et rechargement en cours de manche, fantôme ayant joué : même deck, défausse, cristaux, tactique, pile ;
  la suite de la partie est la même (`save_keeps_dummy`, arcanum-persistance) — ADR 0001.
- **C35** — Une sauvegarde d'avant cette spec (Première reconnaissance sans `dummy` dans le scénario) se recharge et se joue sans
  fantôme (`old_save_loads_without_dummy`, arcanum-persistance).
- **C36** — Rejouer le journal d'une partie avec fantôme redonne le même état (`replaying_log_rebuilds_dummy`) — ADR 0001.

### Écran

- **C37** — Section « Joueur fantôme » du panneau de droite, avec le vrai `fr.json` : titre `dummy.title` avec le nom du héros,
  `dummy.deck` (« Deck : 11 carte(s) »), `dummy.crystals` suivi de l'icône colorée de l'inventaire une fois par cristal, à la suite, rangées par couleur, sans emplacement vide (infobulle `dummy.crystal_count`), tactique (numéro et nom), `dummy.last_turn` avec les cartes
  retournées, `dummy.announced` quand il a annoncé (`dummy_panel_lines`, sur la fonction qui calcule les lignes) — C31.
- **C38** — Un clic sur la défausse du fantôme ouvre la fenêtre de défausse (spec 073) avec ses cartes et le titre
  `dummy.discard_title` (`dummy_discard_window_lists_cards`).
- **C39** — Ligne d'ordre du tour sous l'en-tête de manche : `round_order.player_first` ou `round_order.dummy_first`
  (`round_order_line`) — `PlayerView::round_order`.
- **C40** — Dernier tour annoncé par le fantôme : un seul message, `dummy.announced` dans sa section, en orange (retour de
  Mathieu du 2026-11-19 : pas de bandeau) ; annonce du joueur : la confirmation `round.announce.confirm_prompt` reste, le bouton
  est absent quand le fantôme a annoncé (`dummy_announcement_line`).
- **C41** — Écran de nouvelle partie : étape « Options » dans la barre d'étapes (après Héros), case `new_game.options.dummy`
  cochée par défaut pour Première reconnaissance, absente pour le bac à sable (texte `new_game.options.none`) ; décochée, la
  partie se crée avec `Scenario::without_dummy` et le récapitulatif ajoute `new_game.recap_no_dummy`
  (`options_step_toggles_dummy`) — Q1. Sous la case, une liste déroulante `new_game.options.dummy_hero` : en tête
  `new_game.options.dummy_hero_random` (choisi par défaut), puis les trois héros autres que celui de l'étape Héros, dans l'ordre de
  `Hero::ALL` ; grisée quand la case est décochée. Goldyx choisi, la partie se crée avec `DummyHeroChoice::Hero(Goldyx)` et le
  récapitulatif ajoute `new_game.recap_dummy_hero` ; si le joueur prend ensuite Goldyx à l'étape Héros, la liste revient à « Au
  hasard » (`options_step_chooses_dummy_hero`) — Q2. Contrairement au scénario et au héros (spec 078), ces choix ne sont pas
  retenus : chaque nouvelle partie, en cours de partie (Ctrl+N) comme au lancement, repart de la case cochée et de « Au hasard »
  (`options_reset_to_defaults_for_next_game`) — retour de Mathieu du 2026-11-19.
- **C42** — Fiche de Première reconnaissance : la ligne `scenario.first_reconnaissance.missing` disparaît (plus rien de la
  variante solo ne manque) ; `special_rules` gagne la ligne du fantôme (`first_reconnaissance_sheet_mentions_dummy`).

## Cas limites et refus

- **L1** — Fantôme dont la 3e carte est la dernière du deck, avec des cristaux de sa couleur : rien de plus (deck vide), et pas
  d'annonce avant son tour suivant (C14, C15).
- **L2** — Le joueur annonce alors que le fantôme a vidé son deck sans encore annoncer : le fantôme joue son dernier tour à vide
  (C21) ; il n'annonce pas en plus.
- **L3** — Le fantôme annonce alors que le joueur a un tour supplémentaire en attente (Distorsion temporelle jouée au tour
  précédent) : sans objet, le tour supplémentaire suit immédiatement le tour qui l'a gagné, avant le fantôme (C10).
- **L4** — Le fantôme joue en premier et son deck est vide au début de la manche : impossible (16 cartes au moins après le mélange).
- **L5** — Fantôme annoncé, main et deck du joueur vides au début de son dernier tour (p. 6, 3.d) : le tour est passé sans
  décision, la manche finit dans la décision précédente ; pas de mine ni de clairière (p. 6, 4 ; spec 023, C13)
  (`forfeit_when_nothing_left_after_dummy_announcement`).
- **L6** — Le fantôme ne prend jamais de compétence, d'unité ni de carte d'une offre pendant la manche ; la Source n'est jamais
  touchée par lui (MKUE p. 15) : ses tours ne produisent que `DummyTurnPlayed` ou `DummyAnnouncedEndOfRound`.
- **L7** — Manche 4 de Première reconnaissance (dernière) : à sa fin, ni `DummyCardAdded`, ni `DummyCrystalGained`, ni mélange
  (la partie est finie, spec 023 C15).
- **L8** — Plus de tactique à tirer pour le fantôme : impossible avec 4 manches (Règles, *Retrait*) ; si un scénario futur le
  permet, le tirage est sauté et le joueur joue en premier (cas non testé, noté pour la spec 064 qui a 6 manches : 6 tactiques
  par moment, 3 manches de chaque, 2 retirées par manche, il en reste 2 à la 3e manche).
- **L9** — `Scenario` avec un fantôme et sans tactiques : refusé à la création (`ScenarioError::DummyWithoutTactics`)
  (`dummy_needs_tactics`).
- **L10** — `GameConfig` dont le héros du fantôme est celui du joueur : refusée par la validation de la configuration, aucune
  partie créée (`dummy_hero_differs_from_player_hero`) ; l'écran ne le propose jamais (C41).

## Information cachée

- Visible du joueur (`PlayerView::dummy: Option<DummyView>`, nouveau) : héros, tactique, nombre de cartes du deck, défausse (open
  information, Compendium *Miscellaneous* S2), cristaux, cartes retournées à son dernier tour, fin de manche annoncée, nombre de
  compétences dans sa pile.
- Visible aussi : `PlayerView::round_order` (nouveau) : le joueur ou le fantôme en premier.
- Caché de tous : l'ordre du deck du fantôme et celui de sa pile de compétences (filtrés dans le journal, C32).
- L'offre commune (`PlayerView::common_skills`, spec 051) montre les compétences du fantôme révélées : inchangé.

## Libellés (i18n)

Nouvelles clés (fr / en) :

| Clé | fr | en |
|---|---|---|
| `dummy.title` | Joueur fantôme : {hero} | Dummy player: {hero} |
| `dummy.deck` | Deck : {count} carte(s) | Deck: {count} card(s) |
| `dummy.discard` | Défausse : {count} carte(s) | Discard pile: {count} card(s) |
| `dummy.discard_title` | Défausse du joueur fantôme | Dummy player's discard pile |
| `dummy.crystals` | Cristaux | Crystals |
| `dummy.crystal_count` | {color} : {count} | {color}: {count} |
| `dummy.tactic` | Tactique : {number} {name} | Tactic: {number} {name} |
| `dummy.last_turn` | Dernier tour : {cards} | Last turn: {cards} |
| `dummy.last_turn_extra` | Dernier tour : {cards} ; {color} : {count} de plus | Last turn: {cards}; {color}: {count} more |
| `dummy.last_turn_none` | Dernier tour : aucune carte | Last turn: no card |
| `dummy.announced` | A annoncé la fin de la manche : c'est votre dernier tour. | Announced the End of the Round: this is your last turn. |
| `dummy.skills` | Compétences cachées : {count} | Hidden Skills: {count} |
| `round_order.player_first` | Vous jouez en premier | You play first |
| `round_order.dummy_first` | Le joueur fantôme joue en premier | The Dummy player plays first |
| `new_game.step_options` | Options | Options |
| `new_game.options.dummy` | Joueur fantôme | Dummy player |
| `new_game.options.dummy_help` | Il donne le tempo de la manche. Sans lui, vous jouez à votre rythme : conseillé pour une toute première partie. | It sets the pace of the Round. Without it, you play at your own pace: recommended for a very first game. |
| `new_game.options.none` | Pas d'option pour ce scénario. | No option for this scenario. |
| `new_game.recap_no_dummy` | sans joueur fantôme | without Dummy player |
| `new_game.options.dummy_hero` | Héros du joueur fantôme | Dummy player's Hero |
| `new_game.options.dummy_hero_random` | Au hasard | Random |
| `new_game.recap_dummy_hero` | joueur fantôme : {hero} | Dummy player: {hero} |

`{cards}` : noms des cartes séparés par des virgules (clés `card.<id>.name`) ; `{color}` : `mana.<couleur>` (« vert ») ;
`{hero}` : `hero.<id>.name` ; `{number}` et `{name}` : numéro et `tactic.<id>.name` (spec 060). Exemple : `dummy.last_turn_extra`
avec Rage, Célérité, Marche, `{color}` = « vert », `{count}` = 2 donne « Dernier tour : Rage, Célérité, Marche ; vert : 2 de
plus » (en : « Last turn: Rage, Swiftness, March; green: 2 more »). Les deux cartes en plus sont citées dans la même liste,
après les trois premières, au fil écran si la place le permet ; sinon le nombre suffit (vérification à l'œil). Aucun symbole hors
des polices du jeu.

Clés modifiées (valeurs relues dans `assets/i18n/fr.json` de `main`) :

- `scenario.first_reconnaissance.special_rules` (« Chaque tuile révélée rapporte 1 gloire.\nLes cités ne peuvent être ni
  conquises ni visitées.\nTactiques : vous choisissez la vôtre en premier ; elle est retirée de la partie à la fin de la
  manche. ») devient : « Chaque tuile révélée rapporte 1 gloire.\nLes cités ne peuvent être ni conquises ni visitées.\nTactiques :
  vous choisissez la vôtre en premier, le joueur fantôme tire la sienne parmi les autres ; les deux sont retirées de la partie à
  la fin de la manche.\nJoueur fantôme : il retourne des cartes de son deck à chaque tour et annonce la fin de la manche quand il
  est vide. » (en : « … Tactics: you choose yours first, the Dummy player draws one among the others; both are removed from the
  game at the end of the Round.\nDummy player: it flips cards from its deck each turn and announces the End of the Round when it
  is empty. »).
- `scenario.first_reconnaissance.missing` (« Pas encore dans le jeu : le joueur fantôme de la variante solo. On peut s'en passer
  pour une première partie. ») : retirée de `fr.json` et `en.json` ; `has_missing_line` (arcanum-ui, `new_game.rs`) ne la demande plus
  pour Première reconnaissance, qui n'a plus rien de manquant.

Clés réutilisées (vérifiées dans `fr.json` de `main`) : `mana.red` « rouge », `mana.green` « vert », etc. ; `hero.goldyx.name`
« Goldyx » ; `discard.title` « Défausse » (non réutilisée pour la fenêtre du fantôme, d'où `dummy.discard_title`) ;
`round.announce.button` « Annoncer la fin de la manche » (masqué quand le fantôme a annoncé) ; `new_game.recap_choice`
« {scenario} · {hero} », suivi de « · » et `new_game.recap_no_dummy` : « Première reconnaissance · Tovak · sans joueur
fantôme », ou de « · » et `new_game.recap_dummy_hero` avec `{hero}` = « Goldyx » : « Première reconnaissance · Tovak · joueur
fantôme : Goldyx » (en : « First Reconnaissance · Tovak · Dummy player: Goldyx » ; rien d'ajouté avec « Au hasard ») ; `tactic.<id>.name` (spec 060).

## Interface

- **Section « Joueur fantôme »** dans le panneau de droite, sous celle du joueur : portrait du héros du fantôme (illustration de
  la spec 065, en petit), `dummy.title`, `dummy.tactic`, `dummy.deck` en évidence (c'est ce que le joueur surveille, *Note 1*),
  `dummy.discard` cliquable, `dummy.crystals` (icônes de l'inventaire à la suite, retour de Mathieu du 2026-11-19), `dummy.skills`, `dummy.last_turn` (ou ses variantes), `dummy.announced` (C37, C38,
  `PlayerView::dummy`). Placement, taille du portrait : vérification à l'œil.
- **Fenêtre de défausse du fantôme** : la fenêtre de défausse de la spec 073, titre `dummy.discard_title`, cartes lisibles comme
  celles du joueur (C38, `DummyView::discard`).
- **Ordre du tour** : une ligne sous l'en-tête de manche du panneau, `round_order.player_first` ou `round_order.dummy_first`
  (C39, `PlayerView::round_order`).
- **Dernier tour** : `dummy.announced` en orange dans la section du fantôme, seul message de son annonce (retour de Mathieu du 2026-11-19) ; le bouton `round.announce.button` n'y est pas (C40, `DummyView::announced`).
  `round.announce.button` n'y est pas (C40, `DummyView::announced`).
- **Tour du fantôme** : pas de fenêtre ni de pause ; son résultat se lit dans sa section (`dummy.last_turn`) dès que le tour du
  joueur suivant commence (C37, Q4). Animation : spec 079 ou 084 si Mathieu le veut.
- **Fenêtre de choix de la tactique** (spec 060) : inchangée ; le fantôme tire après le joueur, donc rien à griser.
- **Écran de nouvelle partie** : étape « Options » (`new_game.step_options`) dans la barre d'étapes, après Héros (spec 078, Q2 de
  la 078 : l'onglet apparaît avec la première option) ; liste à gauche réduite à la case `new_game.options.dummy` ; fiche à droite
  avec `new_game.options.dummy_help` ; sous la case, la liste déroulante du héros du fantôme (`new_game.options.dummy_hero`,
  « Au hasard » puis les trois autres héros, grisée sans fantôme) ; bac à sable : `new_game.options.none` ; récapitulatif avec `new_game.recap_no_dummy` quand
  la case est décochée, `new_game.recap_dummy_hero` quand un héros est choisi ; case et liste remises par défaut à chaque nouvelle partie (C41). Fiche du scénario : règles spéciales modifiées,
  plus de ligne « Pas encore dans le jeu » (C42). Mise en page : vérification à l'œil.

Parties préparées pour la vérification à l'œil (`docs/verification-manuelle.md`), sur Première reconnaissance avec fantôme,
chacune atteignable en avançant depuis son départ :

- `fantome-premier` : manche 1, tactique à choisir, fantôme Goldyx ; prendre Au bon moment (6) montre le fantôme jouer en premier
  (section remplie, ligne d'ordre du tour) avant le tour 1.
- `fantome-annonce` : manche 1, deck du fantôme à 2 cartes, le joueur en cours de tour ; finir le tour fait retourner les 2
  cartes ; finir le tour suivant fait annoncer le fantôme (message en orange dans sa section), puis le dernier tour mène à la manche 2 avec
  l'action avancée ajoutée à son deck et son cristal gagné (deck 17 ou plus, cristaux).
- `fantome-competence` : le joueur à 1 point de gloire du niveau 2, un ennemi facile à côté ; le combat et la fin du tour mènent
  au choix de la compétence, puis la compétence du fantôme apparaît dans l'offre commune.
- Écran de nouvelle partie : depuis le menu (Ctrl+N), étape Options (pas de partie préparée).

### Raccourcis clavier

Aucun nouveau. L'étape « Options » s'atteint comme les autres étapes de l'écran de nouvelle partie (clic sur la barre d'étapes,
flèches ← → de la spec 078, prolongées d'une étape) ; la case se coche à la souris ou à la barre d'espace quand elle a le focus
(comportement egui). Reporté dans `docs/raccourcis.md` (ligne des flèches de la nouvelle partie).

## Effets sur les specs déjà implémentées

- **023** : `AnnounceEndOfRound` produit d'abord le dernier tour du fantôme ; refusée quand il a annoncé. La fin du tour peut finir
  la manche (dernier tour annoncé par le fantôme). « Passer son tour » (glossaire) gagne le cas L5.
- **016, 035, 057, 060** : `turn_end_draw` gagne le tour du fantôme avant `TurnStarted`, et la fin de la manche au dernier tour
  (sans pioche), après les tours supplémentaires.
- **050** : `refreshed_offers` envoie l'action avancée en dernière place au fantôme et lui donne le cristal du sort.
- **051** : le choix d'une compétence révèle celle du fantôme ; la seconde option devient atteignable en partie.
- **060** : la prise de la tactique produit le tirage du fantôme ; `TacticRemoval::AllTaken` retire aussi la sienne ;
  `TacticSlot::Taken` reste inutilisé en solo.
- **063** : le dernier tour après la cité sans tour du fantôme ; la fin de manche pendant ce délai termine la partie.
- **065 et 078** : étape « Options », récapitulatif, fiche du scénario.
- **Tests existants** : Première reconnaissance prend un fantôme. Les tests et parcours qui la jouent aujourd'hui (relevé de la
  spec 060 : arcanum-core `premiere_reconnaissance.rs`, `tactiques.rs`, `competences.rs`, `parcours_j4.rs`, `parcours_j5.rs`,
  `nouvelle_partie.rs`, `mise_en_place_arrangee.rs`, `scenario_mise_en_place.rs`, `offres.rs`, `interaction.rs` ; arcanum-persistance
  `premiere_reconnaissance.rs`, `parties_preparees.rs` ; arcanum-ui `fin_de_manche.rs`, `premiere_reconnaissance.rs`, `source.rs`,
  `nouvelle_partie.rs`, `barre_de_menu.rs`, `panneaux.rs`, `scenario_affichage.rs`, `sauvegarde_ui.rs`, `fin_du_tour.rs`,
  `boucle_et_fenetre.rs`, `plateau_isometrique.rs`, `support/mod.rs` ; relevé à refaire au fil moteur par
  `grep -rln "first_reconnaissance\|FirstReconnaissance" crates/*/tests`) gardent leur comportement en créant le scénario
  **sans fantôme** (aide commune), sans changer leurs assertions ; la PR le signale. Seuls les tests de cette spec et le parcours
  du jalon 6 (spec 069) jouent avec le fantôme. L'arrangement des parcours (`tests/support/arrangement.rs`) apprend
  `DummySetUp`.

## Hors périmètre

- Score de Conquête en solitaire (cartes du deck du fantôme, fin de manche non annoncée) : spec 064.
- Cartes bicolores dans le deck du fantôme (Compendium, *Dual-Colored Action Cards* S3) : extensions.
- Plusieurs joueurs, coopératif avec fantôme (tactique du fantôme tirée en premier, MKUE p. 20 ; retrait d'une seule tactique,
  Compendium *Scenarios* Q8), assaut coopératif (Compendium, *Cities* S16) : hors du solo.
- Variante « Commencer à un niveau supérieur » (Compendium, *Variants* S2) : pas prévue.
- Animation du tour du fantôme (Q4) : spec 079 ou 084 si besoin.

## Plan de développement

Spec moteur et écran à la fois : deux fils successifs sur la même branche (issue privée 118), plan détaillé
(`docs/specs/plans/061-joueur-fantome.md`) au début du premier.

1. **Fil moteur** : contenu (`dummy_crystals`), `DummyRule`, `DummyState`, mise en place, tactique et ordre du tour, tour du
   fantôme, fins de manche, nouvelle manche, compétences, fin de partie, vue, journal filtré, retour en arrière, persistance,
   tests existants passés « sans fantôme », arrangement, choix du héros dans `GameConfig` : C1 à C36, L1 à L10.
2. **Fil écran** : libellés, section du panneau, fenêtre de défausse, ligne d'ordre du tour, bandeau, étape Options et
   récapitulatif, fiche du scénario, parties préparées, vérification à l'œil, docs (`architecture.md`,
   `verification-manuelle.md`, `index-par-notion.md`, `glossaire.md`, `raccourcis.md`) : C37 à C42.

## Décisions (2026-11-19)

Mathieu : « Ok sauf pour le choix du héros fantôme : proposer en option de choisir (dropdown avec "au hasard" par défaut et la
liste des héros restants sélectionnable) ».

- **Q1** — Fantôme par défaut dans Première reconnaissance ; case à décocher dans l'étape « Options » de la nouvelle partie,
  cochée à nouveau à chaque nouvelle partie (retour de Mathieu du 2026-11-19, qui remplace « retenue pour la partie suivante »).
- **Q2** — Héros du fantôme au choix dans les Options : liste déroulante, « Au hasard » par défaut (tirage parmi les trois autres
  héros), puis les trois héros restants (Règles, *Héros du fantôme* ; Modèle, *Configuration* ; C1 bis, C41, L10).
- **Q3** — Le fantôme ne joue plus après la révélation de la cité ; une manche qui finit pendant ce délai termine la partie (C29,
  C30).
- **Q4** — Pas de fenêtre ni de pause pour son tour : sa section du panneau se met à jour.
- **Q5** — Main et deck vides après son annonce : la manche finit d'elle-même, sans action (L5).
- **Q6** — Deux fils d'implémentation, moteur puis écran.

## Questions ouvertes

Aucune.
