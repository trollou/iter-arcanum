# 064 — Conquête en solitaire

- Statut : implémentée (validée le 2026-11-19 ; Q1 à Q4 : propositions retenues par Mathieu) (reporter tout changement dans la colonne Statut de `PLAN.md`)
- Jalon : J6
- Dépend de : 012 (scénario, mise en place, forme du plateau de jeu), 015 (exploration, pose des tuiles), 023 (fin de manche),
  054 et 057 (sorts), 060 (tactiques), 061 (joueur fantôme), 062 (cités), 063 (dernier tour, fin de partie, décompte), 065 et
  078 (écran de nouvelle partie) ; toutes implémentées
- Livret : MKUE p. 19 (*Solo Conquest* : *Map*, *Special Rules*, *Scenario End*, *Scoring*) ; p. 14 (*Wedge Map* : pas de tuile
  royaume sur la côte) ; p. 15 (*Standard Achievements Scoring*, *Dummy Player*) ; p. 3 (pioche de tuiles) ; p. 7 (5.e : pose
  des tuiles royaume) ; p. 4 (5.a : limite de manches). Carte de score, *Co-op and Solo Scenarios*
  (`docs/regles/cartes/sites-et-cites.md`). Compendium : *Scenarios* Q8, O30, O31 ; *Final Scoring* S2 à S5 ; *Setup* S1 à S3 ;
  *Dummy Player* S3 ; *Cities* S17
- Crates touchées : arcanum-core (scénario, exploration, fin de partie, décompte, vue), arcanum-persistance (tests, parties préparées),
  arcanum-i18n (libellés), arcanum-ui (écran de nouvelle partie, panneau, fenêtre de fin, infobulle d'exploration)

## Objectif

Le scénario solo de référence devient jouable de bout en bout : **Conquête en solitaire**, six manches pour trouver et conquérir
les deux cités de la région (niveau 5 puis 8), contre le tempo du joueur fantôme. Toutes les cités conquises, le joueur finit son
tour et en joue un dernier ; sinon la partie s'arrête à la fin de la 6e manche. L'écran de fin dit si la mission est accomplie
et donne le score solo : gloire, Hauts Faits sans titres, points des cités et du temps gagné.

Tout le mécanisme existe déjà (tactiques 060, fantôme 061, cités 062, dernier tour et décompte 063) : la spec ajoute un
scénario qui l'assemble, une fin de scénario, un décompte, la règle de pose « pas de tuile royaume sur la côte » de la forme en
coin standard (reportée ici par la spec 015, Q3) et le retrait des quatre sorts compétitifs.

## Règles

### Le scénario

- **Plateau de jeu** (MKUE p. 19, *Map* ; p. 3 ; Compendium, *Setup* Q2 et S3) : en coin, 7 tuiles de campagne **mélangées**
  (les deux révélées à la mise en place en font partie : il en reste 5 à explorer), 2 tuiles royaume de cité et 2 sans cité,
  tirées au hasard et mélangées sous les tuiles de campagne. C'est `form_tile_deck` avec `CountrysideOrder::Random` (spec 012).
- **Pas de tuile royaume sur la côte** (MKUE p. 14, *Wedge Map* : pas de tuile royaume sur la côte, c'est-à-dire
  les files de tuiles les plus à gauche et à droite) : la forme en coin **standard** interdit les emplacements des deux files du
  bord, `i = 0` ou `j = 0` (spec 012, *Emplacements*), à toute tuile royaume, de la pioche comme écartée (p. 7, 5.e et dernier
  point). Les autres conditions de pose ne changent pas (deux tuiles voisines, trois pour une tuile écartée). Première
  reconnaissance garde la forme en coin de la prise en main, sans cette règle (p. 14 : *similar to the one in the starting
  scenario, with one extra rule*) : rien ne change pour elle ni pour le bac à sable.
- **Durée** (p. 19) : 6 manches, Jour, Nuit, Jour, Nuit, Jour, Nuit (spec 023, paramètre `rounds`).
- **Cités** (p. 19, *Cities*) : la première révélée est de niveau 5, la seconde de niveau 8 : `CityRule::Levels([5, 8])`
  (spec 062, ordre de révélation déjà géré par `CityLevels::level`).
- **Joueur fantôme** (p. 19, *Dummy Player* : *one standard Dummy player*) : `DummyRule::Standard` (spec 061), **obligatoire**
  (Q3) ; héros au choix dans les Options, « Au hasard » par défaut, comme pour Première reconnaissance.
- **Tactiques** (p. 19, *Special Rules* ; Compendium, *Scenarios* Q8) : le joueur choisit en premier, le fantôme tire au hasard
  parmi les restantes, les deux sont retirées à la fin de la manche : `TacticRule::Chosen { PlayerFirst, AllTaken }`, comme
  Première reconnaissance (spec 060). Six manches, trois de chaque moment, deux tactiques retirées par manche : chaque tactique
  est prise **exactement une fois** ; à la 3e manche de Jour (ou de Nuit), le joueur choisit entre les deux dernières et le
  fantôme prend l'autre. Le cas « plus de tactique à tirer » de la spec 061 (L8) n'arrive donc pas.
- **Sorts compétitifs retirés** (p. 19, *Cards and Skills* : *Remove the four competitive Spells (109-112) from the Spell
  deck*) : les quatre sorts qui visent les autres joueurs, 17/20 à 20/20 sur les cartes (`docs/regles/cartes/sorts.md`) : Fuite
  de mana (*Mana Meltdown*), Puits de mana (*Mana Claim*), Lecture d'esprit (*Mind Read*), Flux d'énergie (*Energy Flow*). Ils
  ne sont ni dans la pioche de sorts ni dans l'offre : on ne peut ni les acheter, ni les gagner, ni les voir en dernière place
  (le cristal que le fantôme gagne à chaque manche vient donc d'un des seize autres, spec 061). Lecture : les numéros 109 à 112
  ne sont pas imprimés sur nos scans, mais ce sont les quatre seuls sorts « *each other player* » ; Première reconnaissance et le
  bac à sable les gardent (le livret ne les retire que pour ce scénario).
- **Compétence interactive retirée** (p. 19, *Cards and Skills*) : déjà le cas dans tous les scénarios solo, la compétitive est
  hors du contenu, la coopérative reste (spec 051, Q2 ; Compendium, *Skill Tokens* S5 ; test `skills_content_has_ten_skills_per_hero`).
- **Source et offre d'unités** (MKUE p. 3 ; Compendium, *Setup* S1 : le fantôme n'est pas un joueur) : 3 dés et 3 unités, comme
  aujourd'hui (test `setup_rolls_three_dice_in_solo` ; offre d'unités : spec 050). Unités d'élite et les 28 actions avancées :
  le scénario ne restreint rien (pas de règle comme celles de *First Reconnaissance*) ; pas d'unité de village forcée dans l'offre.
- **Pas de gloire par tuile révélée** : règle propre à Première reconnaissance (p. 18) ; ici 0.

### Fin du scénario

- **Toutes les cités conquises** (p. 19, *Scenario End* : toutes les cités conquises, vous jouez un dernier tour, le joueur
  fantôme non) : « toutes » veut dire les **deux** cités du scénario, révélées ou non (Compendium, *Final Scoring* S3) ;
  une seule cité conquise ne suffit pas, même si l'autre n'est jamais révélée. La conquête de la dernière annonce le dernier
  tour (`LastTurnAnnounced`, spec 063), juste après `CityConquered` (spec 062) : le joueur **finit le tour** de la conquête (sans
  interaction dans la cité ce tour-là, Compendium, *Cities* S21), puis joue **un dernier tour** ; la partie s'arrête à sa fin.
  Même lecture que la révélation de la cité de Première reconnaissance (spec 063, Q1 ; Compendium, *Cities* S17 : *players get a
  last turn*).
- **Le fantôme ne joue plus** (p. 19, *the Dummy player does not*) : comme après la révélation de la cité de Première
  reconnaissance (spec 061, C29, test `dummy_does_not_play_after_city_revealed`), le mécanisme `LastTurn` sert tel quel.
- **La manche finit pendant ce délai** (Compendium, *Scenarios* O30 : la phrase *If the Round ends during this, the game ends
  immediately* de *Full Conquest* manque par oubli) : comme dans la spec 061 (C30, test `round_ending_during_last_turns_ends_game`)
  et la spec 063 (C10, C11) : si le fantôme avait annoncé la fin de la manche, ou si le dernier tour commence deck vide et que le
  joueur l'annonce, la manche finit et la partie avec, sans nouvelle manche.
- **Limite de manches** (MKUE p. 4, 5.a) : sinon, la partie s'arrête à la fin de la 6e manche, `GameEnded { LastRound }`. Le
  décompte s'applique dans les deux cas.
- **Mission accomplie ou échouée** (p. 19, *Scoring* : victoire si toutes les cités sont vaincues, échec
  sinon) : accomplie si et seulement si la partie finit par la conquête de toutes les cités ; l'écran de fin le dit.

### Décompte final

- **Base** (p. 19, *Scoring* ; p. 15 ; Compendium, *Final Scoring* S4, S5) : la gloire, plus les six Hauts Faits **sans
  titres**, exactement comme Première reconnaissance (spec 063, `FinalScore`). Les Boucliers des cités ne comptent pas dans le
  Conquérant (spec 062 ; Compendium, *Final Scoring* S2).
- **Points des cités et du temps** (p. 19, *Scoring* ; carte de score, *Co-op and Solo Scenarios*) :

  | Ligne | Barème | Quand |
  |---|---|---|
  | Cités conquises | **10** par cité conquise | toujours |
  | Toutes les cités | **+15** | si les deux sont conquises |
  | Manches non jouées | **30** par manche après celle où la partie finit (6 − numéro de cette manche) | seulement si la partie finit par la conquête (sinon elle finit à la 6e : 0) |
  | Deck du joueur fantôme | **1** par carte encore dans son deck (pas retournée cette manche) | si la mission est accomplie (Q2) |
  | Fin de manche non annoncée | **+5** si personne, ni le joueur ni le fantôme, n'a annoncé la fin de la manche pendant la dernière manche | si la mission est accomplie (Q2 ; en cas d'échec la 6e manche finit toujours par une annonce : 0) |

- **Pas de points de Seigneur** (Q1 ; Compendium, *Scenarios* O31) : les 7 gloire du Seigneur, 4 par contributeur et +5 au plus
  grand conquérant (carte de score, *Conquest Scenarios*) sont pour les scénarios compétitifs ; le livret ne les cite pas pour
  ce scénario.
- **Niveau** : comme la spec 063, le décompte ne fait gagner aucun niveau ; le dernier tour finit normalement avant lui.

### Schéma : la fin de la partie

```
Tour T : assaut de la 2e cité ── combat gagné ── CityShieldPlaced… ── CityConquered ── LastTurnAnnounced
         …le joueur finit son tour (pas d'interaction dans la cité)…
         EndTurn ── TurnEnded … LevelGained? … CardDrawn … TurnStarted (tour T+1, sans tour du fantôme)
                                                             │
Tour T+1 (dernier tour) : deck vide au début ? ── oui ──▶ peut (ou doit) annoncer ── RoundEnded ── GameEnded { AllCitiesConquered }
                                     │ non (ou il joue quand même)
                          EndTurn ── TurnEnded … LevelGained? ── GameEnded { AllCitiesConquered }

Fantôme qui a annoncé avant T : la fin du tour T finit la manche ── RoundEnded ── GameEnded { AllCitiesConquered }
Sans la 2e cité : … RoundEnded { round: 6 } ── GameEnded { LastRound }   (mission échouée)
```

### Exemple de décompte

Mission accomplie : la 2e cité conquise pendant la 5e manche (Jour), le fantôme n'avait pas annoncé ; le dernier tour joué, il
reste 7 cartes dans le deck du fantôme. Le joueur finit avec 62 de gloire, 2 sorts et 3 actions avancées, 1 artefact et 4
cristaux, une unité de niveau 2 et une de niveau 3 blessée, 1 Bouclier sur une forteresse, 1 sur un site d'aventure, 3 Blessures.

| Ligne | Points |
|---|---|
| Gloire | 62 |
| Le Savant (2 sorts, 3 actions avancées) | 7 |
| Le Collectionneur (1 artefact, 4 cristaux) | 4 |
| Le Commandant (2, puis 3 blessée : 1) | 3 |
| Le Conquérant (1 Bouclier) | 2 |
| L'Aventurier (1 Bouclier) | 2 |
| La Victime (3 Blessures) | −6 |
| Cités conquises (2) | 20 |
| Toutes les cités conquises | 15 |
| Manches non jouées (1) | 30 |
| Deck du joueur fantôme (7 cartes) | 7 |
| Fin de manche non annoncée | 5 |
| **Score final** | **151** |

Mission échouée, une cité conquise, la 6e manche finie : gloire et Hauts Faits, plus 10 ; les quatre autres lignes à 0.

## Modèle

Termes du glossaire déjà présents : Scénario, Fin du scénario (`ScenarioEnd`), Dernier tour, Décompte final (`FinalScore`), Haut
Fait, Joueur fantôme, Tactique, Cité, Seigneur de la cité, Côte, Tuile royaume, Forme du plateau de jeu, Sort, Manche.

Nouveaux termes, ajoutés au glossaire par cette PR :

| Livret (EN) | Français | Code |
|---|---|---|
| Solo Conquest | Conquête en solitaire | `ScenarioId::SoloConquest` |
| Wedge map (standard, no core tile on the coastline) | En coin standard | `MapShape::StandardWedge` (nom indicatif) |
| Competitive Spell | Sort compétitif | `SpellSet::WithoutCompetitive` (nom indicatif) |
| (scenario) victorious / failed | Mission accomplie / échouée | `ConquestScore::all_conquered` (nom indicatif) |
| Unplayed Round | Manche non jouée | `ConquestScore::unplayed_rounds` |

Le scénario (spec 012, « une valeur ») gagne :

- une **forme** `StandardWedge` : en coin (face A, mêmes emplacements et mêmes premières tuiles que `Wedge`), plus le refus des
  tuiles royaume sur les emplacements `i = 0` ou `j = 0`. Nouvelle variante plutôt que changement de `Wedge`, pour que
  Première reconnaissance, le bac à sable et leurs sauvegardes ne bougent pas ;
- un champ **sorts** : tous (défaut, et valeur d'une sauvegarde d'avant) ou sans les quatre compétitifs ;
- une **fin du scénario** `AllCitiesConquered` (point d'extension de la spec 063) ; refusée à la création sans cités ouvertes
  (`CityRule::Levels`) ni tuile royaume de cité (`ScenarioError`, L7) ;
- un **décompte** `SoloConquest` : celui de `AchievementsWithoutTitles`, plus les points des cités et du temps ;
- `Scenario::solo_conquest()`, et `Scenario::of` le connaît ; `ScenarioId::ALL` devient Première reconnaissance, Conquête en
  solitaire, bac à sable (Q4) ;
- une question **« fantôme facultatif ? »** (`Scenario::dummy_optional`, nom indicatif) : vrai seulement pour Première
  reconnaissance (p. 18, *you may also ignore the Dummy player in your first game*), lue par l'étape Options (Q3).

La raison de fin `GameEndReason` gagne `AllCitiesConquered`. `Placement` (spec 015) gagne `CoreOnCoastline`.

Le décompte (`FinalScore`) gagne une partie « conquête », présente seulement pour ce décompte : cités conquises, cités du
scénario (2), manches non jouées, cartes du deck du fantôme, fin de manche annoncée pendant la dernière manche ou non, et
mission accomplie ou non. Rien de nouveau n'est gardé dans l'état pour cela : tout se lit dans les cités (spec 062), la manche
en cours, le deck du fantôme (spec 061) et la raison de la fin ; si l'annonce de la dernière manche ne s'y lit pas (annonce du
joueur pendant le dernier tour), le fil d'implémentation l'ajoute à l'état par l'événement qui la porte (`RoundEnded` ou
`DummyAnnouncedEndOfRound`), sans événement nouveau.

## Actions et événements

Aucune action ni aucun événement nouveaux.

| Action | Validation (refusée si…) | Événements produits |
|---|---|---|
| `Explore` (015) | en plus : la prochaine tuile est une tuile royaume (de la pioche ou écartée), la forme est `StandardWedge` et l'emplacement est sur la côte (`CoreOnCoastline`) | inchangés |
| Fin du combat d'un assaut de cité (062, inchangée) | inchangée | `CityShieldPlaced`, `CityConquered`, puis **`LastTurnAnnounced`** si la fin du scénario est `AllCitiesConquered`, que les cités conquises sont toutes celles du scénario et que rien n'est encore annoncé |
| `EndTurn` (016, 063) | inchangée | dernier tour : …, **`GameEnded { AllCitiesConquered }`** ; sinon inchangée |
| `AnnounceEndOfRound` (023, 063) | inchangée | dernier tour : `RoundEnded`, puis **`GameEnded { AllCitiesConquered }`** ; sinon inchangée |

| Événement | Effet de `apply` | Révèle une information ? | Point de non-retour ? |
|---|---|---|---|
| `GameEnded { reason: AllCitiesConquered }` (nouvelle raison) | comme les autres raisons (spec 063) | non | oui (spec 063) |

`LastTurnAnnounced` vient désormais aussi d'une conquête : son effet ne change pas (spec 063, `LastTurn::Next`).

## Comportements attendus

Situation de départ des tests moteur : `new_game` avec Conquête en solitaire (graine fixe) pour la mise en place ; l'arène des
tests (`tests/arene/`, qui prend un scénario depuis la spec 063) pour les états construits : cités posées (outils de la spec 062,
PR privée 345), fantôme dans un état donné. Nouveau fichier `crates/arcanum-core/tests/conquete_en_solitaire.rs`.

Scénario et mise en place :

- **C1** — Conquête en solitaire : forme en coin standard, 7 tuiles de campagne mélangées, 2 tuiles royaume de cité et 2 sans
  cité, 6 manches, cités de niveaux 5 puis 8, fantôme standard, tactiques « joueur d'abord, toutes retirées », 0 gloire par tuile,
  fin `AllCitiesConquered`, décompte `SoloConquest` (`solo_conquest_matches_livret`) — MKUE p. 19, *Map*.
- **C2** — Mise en place : la pioche de tuiles a 9 tuiles (5 de campagne puis 4 royaume, dont 2 de cité) ; deux graines
  différentes donnent un ordre de campagne différent (`solo_conquest_shuffles_countryside`) — MKUE p. 3 ; Compendium, *Setup*
  Q2, S3.
- **C3** — Mise en place : ni Fuite de mana, ni Puits de mana, ni Lecture d'esprit, ni Flux d'énergie dans la pioche de sorts ni
  dans l'offre (16 sorts en tout) ; Première reconnaissance garde les 20 (`solo_conquest_removes_competitive_spells`) — MKUE
  p. 19, *Cards and Skills*.
- **C4** — Mise en place : 3 dés dans la Source, 3 unités dans l'offre, pas d'unité de village forcée
  (`solo_conquest_setup_counts`) — MKUE p. 3 ; Compendium, *Setup* S1.
- **C5** — Six manches : les tactiques prises sur les trois Jours sont six tactiques différentes, et de même pour les Nuits ; à la
  5e manche, les deux tactiques de Jour restantes sont celle du joueur et celle du fantôme (`each_tactic_taken_once_in_six_rounds`)
  — MKUE p. 19, *Special Rules* ; Compendium, *Scenarios* Q8.

Côte :

- **C6** — Forme en coin standard, prochaine tuile royaume : explorer vers un emplacement de la côte (`i = 0` ou `j = 0`, deux
  tuiles voisines posées) est refusé, `placement` dit `CoreOnCoastline` ; vers `(1, 1)` (deux voisines) c'est permis
  (`core_tile_not_on_coastline`) — MKUE p. 14, *Wedge Map*.
- **C7** — Même forme, prochaine tuile de campagne : le même emplacement de la côte est permis (`countryside_tile_on_coastline`)
  — MKUE p. 14.
- **C8** — Pioche vide, tuile royaume sans cité écartée à révéler : refusée sur la côte aussi (`removed_core_tile_not_on_coastline`)
  — MKUE p. 7, 5.e ; p. 14.
- **C9** — Forme en coin de Première reconnaissance : la tuile de cité se pose sur la côte comme aujourd'hui
  (`first_reconnaissance_allows_core_on_coastline`) — MKUE p. 14 (*similar to the one in the starting scenario*).

Fin du scénario :

- **C10** — Une seule cité conquise (l'autre révélée ou non) : pas de `LastTurnAnnounced` (`one_city_conquered_is_not_the_end`) —
  MKUE p. 19, *Scenario End* ; Compendium, *Final Scoring* S3.
- **C11** — Conquête de la seconde cité : `CityConquered` puis `LastTurnAnnounced` ; le joueur finit son tour normalement
  (`conquering_last_city_announces_last_turn`) — MKUE p. 19.
- **C12** — Après C11, fin du tour : pioche et `TurnStarted`, **sans** tour du fantôme ; ce tour est le dernier ; sa fin produit
  `GameEnded { AllCitiesConquered }`, plus aucune action légale (`game_ends_after_last_turn_of_conquest`) — MKUE p. 19, *the
  Dummy player does not*.
- **C13** — Le fantôme avait annoncé la fin de la manche ; la seconde cité est conquise pendant le dernier tour de la manche : à
  sa fin, `RoundEnded` puis `GameEnded { AllCitiesConquered }`, sans nouvelle manche (`round_end_after_conquest_ends_game`) —
  Compendium, *Scenarios* O30.
- **C14** — Dernier tour qui commence deck vide : le joueur peut annoncer la fin de la manche ; `RoundEnded` puis
  `GameEnded { AllCitiesConquered }` (`announcing_on_last_turn_of_conquest_ends_game`) — Compendium, *Scenarios* O30 ; spec 063,
  C10.
- **C15** — Sans la seconde cité, fin de la 6e manche : `RoundEnded { round: 6 }` puis `GameEnded { LastRound }`
  (`solo_conquest_ends_after_sixth_round`) — MKUE p. 4, 5.a.

Décompte :

- **C16** — Score de l'exemple, mission accomplie : 151 ligne par ligne (gloire 62, Hauts Faits 7, 4, 3, 2, 2, −6, cités 20, toutes
  15, manches 30, deck 7, annonce 5) (`solo_conquest_score_example`) — MKUE p. 19, *Scoring* ; carte de score.
- **C17** — Manches non jouées : partie finie à la 4e manche → 2 × 30 ; à la 6e → 0 (`unplayed_rounds_score_thirty_each`) —
  MKUE p. 19.
- **C18** — Fin de manche annoncée pendant la dernière manche (par le fantôme, C13, ou par le joueur, C14) : 0 pour cette ligne
  (`announced_round_end_scores_nothing`) — MKUE p. 19.
- **C19** — Mission échouée (C15, une cité conquise, le joueur ayant annoncé la fin de la 6e manche, 5 cartes dans le deck du
  fantôme) : cités 10, toutes 0, manches 0, deck 0, annonce 0 ; la vue dit « échouée » (`failed_mission_scores_cities_only`) —
  MKUE p. 19 (Q2).
- **C20** — Les Boucliers du joueur sur les cités ne comptent ni dans le Conquérant ni comme points de Seigneur
  (`city_shields_score_nothing_in_solo`) — Compendium, *Final Scoring* S2, *Scenarios* O31 (Q1).
- **C21** — Première reconnaissance : le décompte n'a pas de partie « conquête », inchangé (`first_reconnaissance_score_unchanged`)
  — spec 063, C14.

Vue :

- **C22** — La vue d'une partie de Conquête en solitaire donne la progression : cités conquises et cités du scénario (0/2 au
  départ, 1/2 après une conquête) ; aucune pour Première reconnaissance ni le bac à sable (`view_shows_conquest_progress`).
- **C23** — La vue ne donne le décompte, partie « conquête » comprise, qu'après `GameEnded` (`conquest_score_only_after_game_end`)
  — spec 063, C19.

Sauvegarde :

- **C24** — Une partie de Conquête en solitaire sauvegardée pendant le dernier tour, rechargée : le tour est le dernier, sa fin
  termine la partie avec le même score (`saved_solo_conquest_keeps_last_turn`, arcanum-persistance) — spec 024.
- **C25** — Une sauvegarde d'avant cette spec (Première reconnaissance, bac à sable) se recharge avec ses 20 sorts et sa forme
  (`old_save_keeps_all_spells`, arcanum-persistance).

Interface (arcanum-ui, modèle d'écran) :

- **C26** — Écran de nouvelle partie : la liste des scénarios est Première reconnaissance, Conquête en solitaire, bac à sable ;
  la fiche de Conquête en solitaire montre ses neuf textes (`scenario.solo_conquest.*`) (`new_game_lists_solo_conquest`) — Q4.
- **C27** — Étape Options pour Conquête en solitaire : pas de case `new_game.options.dummy`, le texte
  `new_game.options.dummy_required` à sa place, la liste du héros du fantôme présente ; la partie se crée toujours avec le
  fantôme (`solo_conquest_dummy_is_required`) — Q3.
- **C28** — Panneau : la ligne `scenario.cities_progress` (« Cités conquises : 1/2 ») sous la ligne de la manche, nourrie par
  `PlayerView::conquest` (`panel_shows_cities_progress`).
- **C29** — Après C11, le bandeau `scenario.last_turn_next_conquest` ; pendant le dernier tour, la ligne `scenario.last_turn`
  (`panel_announces_last_turn_after_conquest`).
- **C30** — Fenêtre de fin de C16 : la raison `score.reason.all_cities_conquered`, la ligne `score.outcome.success`, les sept
  lignes de la spec 063, les cinq lignes de conquête, le total, `score.no_titles_solo` (`end_window_shows_conquest_score`).
- **C31** — Fenêtre de fin de C19 : la raison `score.reason.last_round`, la ligne `score.outcome.failure`, les douze lignes, le
  total (`end_window_shows_failed_mission`).
- **C32** — Infobulle d'un emplacement libre de la côte qui touche la figurine, prochaine tuile royaume, forme en coin
  standard : `board.hover.core_on_coastline` (`hover_explains_core_on_coastline`).
- **C33** — Chaque clé nouvelle existe en français et en anglais (`every_solo_conquest_key_exists`).

## Cas limites et refus

- **L1** — Seconde cité jamais révélée (pioche vide sans elle, ou plus d'emplacement permis hors de la côte) : la partie finit par
  la limite de manches, mission échouée.
- **L2** — Une cité assaillie plusieurs fois, avec retraite (spec 062), n'annonce rien tant qu'elle n'est pas conquise ; si elle
  ne l'est jamais, ses Boucliers ne comptent pas au décompte (Compendium, *Final Scoring* S2).
- **L3** — Tour supplémentaire (Distorsion temporelle, Au bon moment) pendant le dernier tour : il se joue avant `GameEnded`,
  comme pour Première reconnaissance (spec 061, C20 et Règles, *Tour en plus pendant le dernier tour*).
- **L4** — Conquête de la seconde cité pendant la 6e manche, le fantôme n'ayant pas annoncé : le dernier tour se joue ; manches
  non jouées 0 ; +5 si la manche n'a pas été annoncée.
- **L5** — Annoncer la fin de la manche et conquérir une cité dans le même tour : impossible, l'annonce remplace le tour (MKUE
  p. 6, 4).
- **L6** — Toutes les tuiles royaume sur la côte seulement : le joueur ne peut explorer qu'ailleurs (C6) ; si aucun emplacement
  intérieur ne touche sa figurine, il ne révèle rien ce tour-là.
- **L7** — `Scenario::new` (ou un constructeur de test) avec la fin `AllCitiesConquered` et des cités fermées, ou sans tuile
  royaume de cité : refusé (`ScenarioError::ConquestWithoutCities`, nom indicatif) (`conquest_end_needs_cities`).
- **L8** — Le décompte se fait sur l'état complet ; le deck du fantôme est compté, jamais montré dans son ordre (spec 061).

## Information cachée

Tout est public : cités conquises, manche, nombre de cartes du deck du fantôme (déjà dans `DummyView::deck_size`, spec 061),
score. Champs ajoutés à la vue :

- `PlayerView::conquest` : la progression (cités conquises, cités du scénario), présente seulement quand la fin du scénario est
  `AllCitiesConquered` (C22).
- `FinalScore` (déjà dans `PlayerView::final_score`, spec 063) : sa partie « conquête » (C16, C19, C23).

La forme du plateau de jeu, lue par l'infobulle d'exploration, est déjà dans la vue (`PlayerView::scenario`).

## Libellés (i18n)

Clés réutilisées, relues dans `assets/i18n/fr.json` de `main` : `scenario.last_turn` (« Dernier tour »), `score.title` (« Fin de
partie »), `score.reason.last_round` (« La dernière manche est jouée. »), `score.fame`, `score.knowledge`, `score.loot`,
`score.leader`, `score.conqueror`, `score.adventurer`, `score.beating`, `score.total`, `score.show_board`, `score.reopen`
(spec 063, valeurs inchangées), `game.finished` (« Partie terminée », message du panneau pour la fin par conquête),
`new_game.options.dummy_hero` (« Héros du joueur fantôme »), `new_game.options.dummy_hero_random` (« Au hasard »).
`score.no_titles` (« Variante solo : pas de bonus de titre. ») ne convient pas : Conquête en solitaire n'est pas une variante.

| Clé | Paramètres | fr | en |
|---|---|---|---|
| `scenario.solo_conquest.name` | — | Conquête en solitaire | Solo Conquest |
| `scenario.solo_conquest.tagline` | — | Le scénario solo de référence | The reference solo scenario |
| `scenario.solo_conquest.flavor` | — | Vous êtes seul, mais le Conseil compte toujours sur vous : trouvez les deux cités de la région et prenez-les d'assaut avant la fin de la troisième nuit. | You are on your own, yet the Council still counts on you: find the two cities of the land and storm them before the third night is over. |
| `scenario.solo_conquest.length` | — | 6 manches : Jour, Nuit, Jour, Nuit, Jour, Nuit | 6 rounds: Day, Night, Day, Night, Day, Night |
| `scenario.solo_conquest.players` | — | Solo | Solo |
| `scenario.solo_conquest.kind` | — | Conquête | Conquest |
| `scenario.solo_conquest.objective` | — | Trouver et conquérir les deux cités : la première révélée est de niveau 5, la seconde de niveau 8.\nToutes conquises, la mission est accomplie. | Find and conquer both cities: the first one revealed is level 5, the second one level 8.\nWith both conquered, the mission is accomplished. |
| `scenario.solo_conquest.special_rules` | — | Pas de tuile royaume sur la côte (les deux files du bord).\nTactiques : vous choisissez la vôtre en premier, le joueur fantôme tire la sienne parmi les autres ; les deux sont retirées de la partie à la fin de la manche.\nJoueur fantôme : il retourne des cartes de son deck à chaque tour et annonce la fin de la manche quand il est vide.\nLes quatre sorts qui visent les autres joueurs sont retirés. | No core tile on the coastline (the two edge lanes).\nTactics: you choose yours first, the Dummy player draws one among the others; both are removed from the game at the end of the Round.\nDummy player: it flips cards from its deck each turn and announces the End of the Round when it is empty.\nThe four Spells that target other players are removed. |
| `scenario.solo_conquest.game_end` | — | Toutes les cités conquises : vous finissez ce tour, puis jouez un dernier tour, sans le joueur fantôme.\nSinon, la partie s'arrête à la fin de la 6e manche. | All cities conquered: finish this turn, then play one last turn, without the Dummy player.\nOtherwise, the game ends after the 6th round. |
| `scenario.solo_conquest.scoring` | — | Votre gloire, plus les Hauts Faits (pas de titres en solo).\n+10 par cité conquise, +15 si toutes le sont.\nMission accomplie : +30 par manche non jouée, +1 par carte du deck du joueur fantôme, +5 si la fin de la manche n'était pas annoncée. | Your Fame, plus the Achievements (no titles in solo play).\n+10 per conquered city, +15 if all of them are.\nMission accomplished: +30 per unplayed round, +1 per card in the Dummy player's deck, +5 if the End of the Round was not announced. |
| `scenario.cities_progress` | `conquered`, `total` | Cités conquises : {conquered}/{total} | Cities conquered: {conquered}/{total} |
| `scenario.last_turn_next_conquest` | — | Toutes les cités sont conquises : après ce tour, encore un dernier tour. | All cities are conquered: one last turn after this one. |
| `score.reason.all_cities_conquered` | — | Toutes les cités sont conquises et le dernier tour est joué. | All cities are conquered and the last turn is played. |
| `score.outcome.success` | — | Mission accomplie ! | Mission accomplished! |
| `score.outcome.failure` | — | Mission échouée : des cités restent à conquérir. | Mission failed: some cities remain unconquered. |
| `score.cities` | `count`, `points` | Cités conquises ({count}) : {points} | Conquered cities ({count}): {points} |
| `score.all_cities` | `points` | Toutes les cités conquises : {points} | All cities conquered: {points} |
| `score.unplayed_rounds` | `rounds`, `points` | Manches non jouées ({rounds}) : {points} | Unplayed rounds ({rounds}): {points} |
| `score.dummy_deck` | `cards`, `points` | Deck du joueur fantôme ({cards} carte(s)) : {points} | Dummy player's deck ({cards} card(s)): {points} |
| `score.round_not_announced` | `points` | Fin de manche non annoncée : {points} | End of Round not announced: {points} |
| `score.no_titles_solo` | — | Solo : pas de bonus de titre. | Solo: no title bonuses. |
| `new_game.options.dummy_required` | — | Ce scénario se joue avec le joueur fantôme. | This scenario is played with the Dummy player. |
| `board.hover.core_on_coastline` | — | Pas de tuile royaume sur la côte : explorez un emplacement plus à l'intérieur | No core tile on the coastline: explore a space further inland |

Textes obtenus : `scenario.cities_progress` avec 1 et 2 donne « Cités conquises : 1/2 » / « Cities conquered: 1/2 » ;
`score.dummy_deck` avec 7 donne « Deck du joueur fantôme (7 carte(s)) : 7 ». Les `\n` séparent les puces de la fiche (spec
078). Aucun symbole hors des polices : seulement des lettres et la ponctuation courante.

## Interface

- **Écran de nouvelle partie** (spec 078) : Conquête en solitaire entre Première reconnaissance et le bac à sable ; sa fiche
  avec les neuf textes `scenario.solo_conquest.*` (C26, `ScenarioId::ALL`). Étape Options : titre de la case remplacé par le
  texte `new_game.options.dummy_required`, puis la liste du héros du fantôme (C27). Récapitulatif : comme aujourd'hui, le héros
  du fantôme s'il est choisi.
- **Panneau latéral** : sous « Manche 2/6 · Nuit (tour 3) », la ligne « Cités conquises : 1/2 » (C28,
  `PlayerView::conquest`) ; après la conquête de la seconde cité, le bandeau `scenario.last_turn_next_conquest`, puis la ligne
  « Dernier tour » pendant le dernier tour (C29, `PlayerView::last_turn` et `PlayerView::scenario`). Fin de partie : message
  `game.finished` et bouton « Score final », comme la spec 063.
- **Fenêtre de fin** (spec 063, même cadre) : la raison, la ligne de mission en gras sous la raison, les sept lignes des Hauts
  Faits, les cinq lignes de conquête, le total en gras, `score.no_titles_solo` en petit (C30, C31, `PlayerView::final_score` et
  `PlayerView::game_end`). Mise en page : vérification à l'œil.

```
┌─ Fin de partie ──────────────────────────────────────────────┐
│ Toutes les cités sont conquises et le dernier tour est joué. │
│ Mission accomplie !                                          │
│                                                              │
│ Gloire : 62                                                  │
│ Le Savant : 7                                                │
│ Le Collectionneur (1 artefact(s), 4 cristaux) : 4            │
│ Le Commandant : 3                                            │
│ Le Conquérant (1 Bouclier(s)) : 2                            │
│ L'Aventurier : 2                                             │
│ La Victime (3 Blessure(s)) : -6                              │
│ Cités conquises (2) : 20                                     │
│ Toutes les cités conquises : 15                              │
│ Manches non jouées (1) : 30                                  │
│ Deck du joueur fantôme (7 carte(s)) : 7                      │
│ Fin de manche non annoncée : 5                               │
│ Score final : 151                                            │
│ Solo : pas de bonus de titre.                                │
│                        [Nouvelle partie…] [Voir le plateau]  │
└──────────────────────────────────────────────────────────────┘
```

- **Infobulle d'exploration** : sur un emplacement libre de la côte qui touche la figurine, quand la prochaine tuile est une
  tuile royaume et que la forme est en coin standard, la ligne `board.hover.core_on_coastline` à la place de « Cliquez pour
  révéler une tuile royaume » (C32, `PlayerView::scenario`, `PlayerView::next_tile`).

**Parties préparées** (`docs/verification-manuelle.md`), chacune atteignable en avançant depuis son départ :

| Nom | Départ | Ce qu'on y vérifie |
|---|---|---|
| `conquete-cote` | Conquête en solitaire, Arythea, jour, début du tour. Les 7 tuiles de campagne posées ; prochaine tuile : royaume. Figurine sur une case qui touche un emplacement libre de la côte (`i = 0` ou `j = 0`) et un emplacement intérieur permis ; 2 points de mouvement de reste en main (Marche). | Infobulles des deux emplacements : refus « côte » sur l'un, « Cliquez pour révéler une tuile royaume » sur l'autre ; explorer l'intérieur (C32). |
| `conquete-derniere-cite` | Conquête en solitaire, Arythea, jour, manche 3, début du tour. Première cité conquise (cité bleue, Seigneur) ; seconde cité, rouge niveau 8 (tuile royaume 8), dont il ne reste qu'un défenseur, le Minotaure (`enemies.ron`, Minotaur : armure 5, attaque 5, Brutal), et 3 Boucliers du joueur dessus ; figurine voisine. Main et cristaux de quoi vaincre le Minotaure fortifié en phase d'attaque (choisis au fil d'implémentation et notés dans le tableau). | Panneau « Cités conquises : 1/2 » ; assaut et victoire : bandeau `scenario.last_turn_next_conquest`, « 2/2 » ; fin du tour sans tour du fantôme ; « Dernier tour » ; fin du dernier tour : fenêtre de fin, « Mission accomplie ! » et les douze lignes (C28 à C30). |
| `conquete-accomplie` | `conquete-derniere-cite` jouée jusqu'au bout (comme `fin-de-partie` pour la spec 063). | La fenêtre de fin s'ouvre au chargement ; la fermer, la rouvrir (C30). |
| `conquete-echouee` | Conquête en solitaire, Arythea, nuit, manche 6, début du tour. Une cité conquise ; le fantôme a annoncé la fin de la manche (dernier tour du joueur). | Finir le tour : fenêtre de fin, raison « dernière manche », « Mission échouée… », cités 10, quatre lignes à 0 (C31). |

### Raccourcis clavier

Aucun : Échap ferme déjà la fenêtre de fin (spec 063).

## Effets sur les specs déjà implémentées

- **012** : `MapShape` gagne `StandardWedge` ; le tableau des prévisions (« pas de tuile royaume sur la côte ») est réalisé.
- **015** : `placement` gagne le refus `CoreOnCoastline` (Q3 de la 015 tranchée par cette spec).
- **061** : la note de L8 (« noté pour la spec 064 ») est réglée par C5 : avec six manches, chaque tactique est prise une fois.
- **062** : `Scenario::with_cities` sert aux tests ; Conquête en solitaire est le premier scénario jouable avec des cités ouvertes.
- **063** : `GameEndReason` et `FinalScoring` gagnent une variante ; `scenario.last_turn_next` reste pour la cité révélée.
- **065 et 078** : un troisième scénario dans la liste ; `has_missing_line` le connaît (pas de ligne « Pas encore dans le jeu »).
- **Parties préparées et parcours existants** : inchangés (aucun n'utilise ce scénario).

## Hors périmètre

- Variantes du scénario (p. 19, *Variants* : niveaux de cités réglables, Mégapole) : variantes hors périmètre (PLAN.md).
- Autres missions solo (p. 19, *Other Solo Missions*) et scénarios compétitifs (*Full Conquest*, *Blitz Conquest*) : hors plan.
- Points de Seigneur de cité en solo (Q1).
- Parcours complet d'une partie de Conquête en solitaire : spec 069 (Parcours du jalon 6).

## Questions ouvertes

Aucune. Décisions (Mathieu, 2026-11-19 : propositions retenues) :

- **Q1 — Points de Seigneur en solo** : non comptés, selon le livret p. 19 (Compendium, *Scenarios* O31).
- **Q2 — Bonus de temps en cas d'échec** : le deck du fantôme et le +5 ne comptent que si la mission est accomplie, comme les
  manches non jouées.
- **Q3 — Fantôme obligatoire** : pas de case « Joueur fantôme » dans les Options de Conquête en solitaire, remplacée par
  `new_game.options.dummy_required` ; la liste de son héros reste.
- **Q4 — Place dans la liste** : Première reconnaissance, Conquête en solitaire, bac à sable ; le scénario proposé par défaut
  reste le dernier choisi.
