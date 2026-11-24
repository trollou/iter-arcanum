# 063 — Première reconnaissance en solo

- Statut : implémentée (2026-10-30) ; validée le 2026-10-30 (Q1 et Q2 tranchées par la prise en main FR p. 19, Q3 : proposition retenue par Mathieu)
- Jalon : J4
- Dépend de : 012 (scénario, mise en place), 015 (exploration), 023 (fin de manche, fin de partie), 024 (sauvegarde, points de
  non-retour), 034 (Blessures), 035 (gloire et niveaux) ; toutes implémentées
- Livret : MKUE p. 18 (*First Reconnaissance*, dont *Solo Variant*), p. 15 (*Standard Achievements Scoring*), p. 4 (4.c, 4.d et 5 :
  fin de partie), p. 6 (3 : annonce de la fin de la manche), p. 7 (4.a : site fortifié ; 5.g : cité révélée) ; carte de score
  (`docs/regles/cartes/sites-et-cites.md`) ; Compendium, *Final Scoring* S4, S5, *Scenarios* S10, O30
- Crates touchées : arcanum-core, arcanum-ui, arcanum-persistance (une partie préparée), `assets/i18n/` (fr.json et en.json)

## Objectif

`cargo run -p arcanum-ui` lance une vraie partie courte : le scénario **Première reconnaissance** en solo, à la place du bac à sable.
Chaque tuile révélée rapporte 1 gloire ; révéler la cité annonce un dernier tour ; la partie finie, un **écran de fin** donne le
score : la gloire, plus le décompte des Hauts Faits sur ce qui existe déjà (cristaux, Blessures). Le bac à sable reste, pour les tests et
les parties préparées.

## Règles

### Le scénario

- **Plateau de jeu et durée** (MKUE p. 18, *Map* et *Solo Variant*) : en coin, 8 tuiles de campagne par numéro, 1 tuile royaume de
  cité, 2 sans cité, 4 manches (Jour, Nuit, Jour, Nuit). C'est déjà le plateau du bac à sable (spec 012) : rien ne change à la mise en
  place, à graine égale les deux scénarios posent les mêmes tuiles et les mêmes ennemis.
- **Sans joueur fantôme** (MKUE p. 18, *Solo Variant*, dernière phrase : *you may also ignore the Dummy player in your first
  game*) : choix permis par le livret pour une première partie. Joueur fantôme et tactiques solo : specs 061 et 060.
- **Cités ni conquises ni visitées** (MKUE p. 18, *Map*, *Cities*) : déjà vrai, une cité est un site fortifié dans lequel on ne
  peut pas entrer tant que l'assaut n'existe pas (spec 014, `MoveCost::Fortified` ; MKUE p. 7, 4.a). La cité révélée ne reçoit ni
  figurine de niveau ni défenseurs (p. 7, 5.g : spec 062 ; inutiles ici).
- **1 gloire par tuile révélée** (MKUE p. 18, *Special Rules*, 1er point, *no matter which type*) : chaque exploration rapporte 1
  gloire, tuile de campagne ou royaume, avec ou sans cité, y compris une tuile écartée reprise faute de pioche (spec 015). Les tuiles
  révélées par la mise en place ne rapportent rien : ce n'est pas le joueur qui les révèle. La gloire compte pour le changement de
  niveau à la fin du tour, comme celle des combats (spec 035).
- **Autres règles spéciales** (MKUE p. 18 ; Compendium, *Scenarios* S10) : combat entre joueurs, offre d'unités avec un village,
  pas d'unités d'élite, actions avancées 1 à 16, sorts et actions avancées révélés au besoin : sans objet tant qu'il n'y a ni
  autre joueur ni offres (specs 050 et 042, voir Hors périmètre).

### Fin du scénario

- **Cité révélée** (MKUE p. 18, *Scenario End*) : *all players (including themselves) have one last turn*. En solo, lecture
  confirmée par la prise en main FR p. 19 (« Chaque joueur (y compris celui ayant dévoilé la Cité) joue un tour
  supplémentaire avant que la partie ne s'achève ») : le joueur **finit le tour** où il a révélé la cité, puis joue **un dernier tour** ; la partie s'arrête à la fin de ce
  dernier tour. Le contrôle se fait à la fin du tour (p. 4, 4.c).
- **Manche finie pendant le dernier tour** (MKUE p. 18, *If the Round ends during this, the game ends immediately* ; p. 4, 4.d) : en
  solo, la manche ne finit que par l'annonce, au début d'un tour, deck vide (spec 023). Si le dernier tour commence deck vide, le joueur
  peut l'annoncer au lieu de jouer (il le doit si sa main est vide aussi) : la partie s'arrête aussitôt. Le dernier tour ne déborde
  donc jamais sur une nouvelle manche.
- **Limite de manches** (MKUE p. 4, 5.a) : sans cité révélée, la partie s'arrête à la fin de la 4e manche, comme aujourd'hui (spec
  023). Le décompte s'applique dans les deux cas.
- **Cité révélée au tour même de l'annonce de la fin de la 4e manche** : impossible, passer son tour pour annoncer ne permet pas
  d'explorer.

### Décompte final

- **Score** (MKUE p. 18, *Scoring* et *Solo Variant*, *award no titles* ; p. 15 ; Compendium, *Final Scoring* S4) : la gloire, plus
  les points de chaque Haut Fait, **sans** les bonus (+3, +1, −3, −1) qui départagent les joueurs. Il n'y a ni vainqueur ni défaite en
  solo : le jeu donne le score.
- **Les six Hauts Faits** (MKUE p. 15 ; noms : prise en main FR p. 19), calculés sur ce que le jeu connaît aujourd'hui ; ce qui n'existe pas encore vaut 0 sans code
  dédié, et chaque spec qui l'ajoute (sorts, artefacts, unités, sites) remplit sa ligne :

  | Haut Fait | Barème (p. 15) | Aujourd'hui |
  |---|---|---|
  | Le Savant | 2 par sort, 1 par action avancée dans le deck | 0 (specs 050, 054) |
  | Le Collectionneur | 2 par artefact, 1 par **deux** cristaux de l'inventaire (arrondi à l'inférieur) | cristaux (spec 021) |
  | Le Commandant | niveau des unités, moitié pour une unité blessée | 0 (spec 042) |
  | Le Conquérant | 2 par jeton Bouclier sur forteresse, tour de mage, monastère | 0 (specs 041, 043) |
  | L'Aventurier | 2 par jeton Bouclier sur un site d'aventure | 0 (spec 044) |
  | La Victime | −2 par Blessure dans le deck (pas sur les unités) | Blessures (spec 034) |

- **« Dans le deck »** (p. 15 ; prise en main FR p. 19 : « regrouper toutes leurs cartes (de main, leur défausse et leur Deck
  Joueur) ») : toutes les cartes Acte du joueur, deck, main et défausse (la zone de jeu est vide après le dernier
  tour, et vide aussi après une annonce). Une carte jetée hors du jeu ne compte pas (Compendium, *Final Scoring* S5). Les jetons de
  mana, rendus à la fin du tour, ne comptent pas.
- **Niveau** (prise en main FR p. 19 : les joueurs gagnent de la gloire « mais ignorent les éventuels changements de Niveau ») : le
  décompte ne fait gagner aucun niveau. Le dernier tour
  finit normalement (bénéfice de la case, changement de niveau, spec 035) avant le décompte.

### Schéma : la fin de la partie

```
Tour T : Explore ── TileRevealed (tuile de cité) ── FameGained { 1 } ── LastTurnAnnounced
         …le joueur continue son tour…
         EndTurn ── TurnEnded … LevelGained? … CardDrawn … TurnStarted (tour T+1)
                                                             │
Tour T+1 (dernier tour) : deck vide au début ? ── oui ──▶ peut (ou doit) annoncer ── RoundEnded ── GameEnded { CityRevealed }
                                     │ non (ou il joue quand même)
                          EndTurn ── TurnEnded … LevelGained? ── GameEnded { CityRevealed }     (ni pioche, ni TurnStarted)

Sans cité révélée : … RoundEnded { round: 4 } ── GameEnded { LastRound }
```

### Exemple de décompte

Fin de partie au niveau 4 : 17 de gloire (5 tuiles révélées et trois combats), 3 cristaux, 2 Blessures dans le deck.

| Ligne | Points |
|---|---|
| Gloire | 17 |
| Le Savant | 0 |
| Le Collectionneur (3 cristaux) | 1 |
| Le Commandant | 0 |
| Le Conquérant | 0 |
| L'Aventurier | 0 |
| La Victime (2 Blessures) | −4 |
| **Score final** | **14** |

## Modèle

Termes du glossaire déjà présents : Scénario, Bac à sable, Gloire, Tuile, Révéler, Tuile royaume de cité, Cristal, Blessure, Manche,
Tour, Deck.

Nouveaux termes, ajoutés au glossaire par cette PR ; les noms français viennent du livret de prise en main français, p. 19 (scan `docs/regles/scans/misc/prise-en-main-fr-p19-fin-de-partie.jpg`, fourni par Mathieu) :

| Livret (EN) | Français | Code |
|---|---|---|
| First Reconnaissance | Première reconnaissance | `ScenarioId::FirstReconnaissance` |
| Scenario End | Fin du scénario | `ScenarioEnd` (`RoundLimit`, `CityRevealed`) |
| one last turn | Dernier tour | `LastTurnAnnounced` |
| (final) Scoring | Décompte final (FAQ p3) | `FinalScore` |
| Achievement | Haut Fait (prise en main FR p. 19) | `Achievement` |
| The Greatest Knowledge / Loot / Leader / Conqueror / Adventurer / Beating | Le Savant / Le Collectionneur / Le Commandant / Le Conquérant / L'Aventurier / La Victime (prise en main FR p. 19) | `Achievement::Knowledge` / `Loot` / `Leader` / `Conqueror` / `Adventurer` / `Beating` |
| Shield token | Jeton Bouclier (FAQ p3) | — (pas encore dans le jeu) |

Le scénario gagne trois champs typés (spec 012, « Le scénario, une valeur »), donnés au constructeur qui le valide :

- **gloire par tuile révélée** : 0 pour le bac à sable, 1 pour Première reconnaissance ;
- **fin du scénario** (point d'extension) : `RoundLimit` (la limite de manches seule, le bac à sable) ou `CityRevealed` (en plus,
  dernier tour après la révélation d'une cité) ;
- **décompte final** (point d'extension) : `None` (le bac à sable : la partie s'arrête sans score) ou `AchievementsWithoutTitles`.

Les deux scénarios ont le même plateau de jeu et 4 manches ; le bac à sable ne change pas. Une sauvegarde d'avant cette spec n'a pas
ces champs : ils prennent les valeurs du bac à sable, et la partie se recharge comme avant (L6).

Données nouvelles dans l'état : « dernier tour annoncé » (oui / non) et « partie terminée » avec sa raison, changés seulement par
leurs événements. Le score n'est pas gardé : une fonction d'arcanum-core le calcule depuis l'état (gloire, inventaire, cartes), et la vue
du joueur le donne une fois la partie terminée (ligne par ligne, puis le total).

La configuration par défaut d'arcanum-ui (`default_config`) passe à Première reconnaissance, toujours avec Arythea, jusqu'à l'écran de
nouvelle partie (spec 065). Les parties préparées et les parcours 019, 028, 039 gardent le bac à sable.

## Actions et événements

Aucune action nouvelle.

| Action | Validation (refusée si…) | Événements produits |
|---|---|---|
| `Explore` (015, inchangée) | inchangée | `Explored`, `TileRevealed`, les jetons d'ennemis (030), puis **`FameGained`** si le scénario donne de la gloire, puis **`LastTurnAnnounced`** si la tuile porte une cité, que la fin du scénario est `CityRevealed` et que rien n'est encore annoncé |
| `EndTurn` (016, inchangée) | inchangée | dernier tour : `TurnEnded`, bénéfice de la case, `LevelGained`, défausses, puis **`GameEnded`** au lieu de la pioche et de `TurnStarted` ; sinon inchangée |
| `AnnounceEndOfRound` (023, inchangée) | inchangée | `RoundEnded`, puis **`GameEnded`** si c'est le dernier tour ou la dernière manche ; sinon la nouvelle manche, comme avant |

| Événement | Effet de `apply` | Révèle une information ? | Point de non-retour ? |
|---|---|---|---|
| `FameGained { player, fame }` | la gloire du joueur augmente de `fame` | non (publique) | non (`TileRevealed`, qui le précède, l'est déjà) |
| `LastTurnAnnounced { player }` | « dernier tour annoncé » : le tour qui suit est le dernier | non (publique) | non (idem) |
| `GameEnded { reason }` (`CityRevealed`, `LastRound`) | partie terminée : plus aucune décision | non (publique) | oui : rien ne se joue après, rien à annuler |

`GameEnded` remplace, pour arcanum-core comme pour l'UI, la déduction actuelle « dernière manche finie, plus de décision » (spec 023) ; le
bac à sable le produit aussi, avec `LastRound`.

## Comportements attendus

Situation de départ des tests moteur : `new_game` avec Première reconnaissance (graine fixe), ou un scénario de test construit par le
constructeur ; l'arène des tests (`tests/arene/`) pour les états construits (cristaux, Blessures).

Scénario et gloire :

- **C1** — Première reconnaissance : en coin, 8 tuiles de campagne par numéro, 1 de cité, 2 sans cité, 4 manches, 1 gloire par tuile,
  fin `CityRevealed`, décompte `AchievementsWithoutTitles` ; à graine égale, même mise en place que le bac à sable
  (`first_reconnaissance_matches_livret_solo_variant`) — MKUE p. 18.
- **C2** — Partie neuve : gloire 0, les deux tuiles de la mise en place n'ont rien rapporté (`setup_tiles_give_no_fame`) — MKUE p. 18.
- **C3** — Explorer une tuile de campagne : `FameGained { fame: 1 }` juste après `TileRevealed` et les jetons, gloire 1
  (`revealing_a_tile_gives_one_fame`) — MKUE p. 18, *Special Rules*.
- **C4** — Bac à sable, même exploration : aucun `FameGained` (`sandbox_gives_no_fame_for_tiles`).
- **C5** — Gloire 2, niveau 1, une exploration (total 3) : niveau 2 à la fin du tour, pas avant (`tile_fame_levels_up_at_end_of_turn`)
  — MKUE p. 11, 6 ; spec 035.
- **C6** — Pioche vide, tuile écartée reprise (spec 015) : elle rapporte 1 gloire aussi (`removed_tile_revealed_gives_fame`) — MKUE
  p. 18, *no matter which type*.

Fin du scénario :

- **C7** — Révéler la tuile de cité : `FameGained`, puis `LastTurnAnnounced` ; le joueur continue son tour normalement
  (`revealing_city_announces_last_turn`) — MKUE p. 18, *Scenario End*.
- **C8** — Après C7, fin du tour : pioche et `TurnStarted` comme d'habitude ; ce nouveau tour est le dernier
  (`turn_after_city_is_the_last_one`).
- **C9** — Fin du dernier tour : `TurnEnded`, bénéfice de la case et `LevelGained` s'il y a lieu, puis `GameEnded { CityRevealed }` ;
  ni pioche, ni `TurnStarted`, plus aucune action légale (`game_ends_after_last_turn`) — MKUE p. 4, 4.c.
- **C10** — Dernier tour qui commence deck vide, main non vide : le joueur peut annoncer la fin de la manche ; s'il le fait,
  `RoundEnded` puis `GameEnded { CityRevealed }`, pas de nouvelle manche (`round_end_during_last_turn_ends_game`) — MKUE p. 18 ; p. 4,
  4.d.
- **C11** — Dernier tour, deck et main vides : seule action, annoncer ; même fin que C10 (`empty_hand_on_last_turn_ends_game`).
- **C12** — Sans cité révélée, fin de la 4e manche : `RoundEnded { round: 4 }` puis `GameEnded { LastRound }` ; pareil dans le bac à
  sable (`round_limit_ends_game_with_game_ended`) — MKUE p. 4, 5.a ; spec 023.
- **C13** — Bac à sable : révéler la cité n'annonce rien, la partie va jusqu'à la 4e manche (`sandbox_ignores_city_reveal`).

Décompte :

- **C14** — Score de l'exemple : gloire 17, 3 cristaux, 2 Blessures (main, deck et défausse confondus) : savoir 0, butin 1, chef 0,
  conquérant 0, aventurier 0, raclée −4, total 14 (`final_score_counts_achievements_without_titles`) — MKUE p. 15 ; Compendium,
  *Final Scoring* S4.
- **C15** — Butin : 1 cristal → 0, 2 → 1, 5 (toutes couleurs confondues) → 2 (`loot_counts_one_per_two_crystals`) — MKUE p. 15.
- **C16** — Une Blessure jetée par un soin (spec 034) ne compte pas dans la raclée (`healed_wounds_do_not_count`) — Compendium,
  *Final Scoring* S5.
- **C17** — Le score peut être négatif : gloire 2, 3 Blessures → −4 (`final_score_can_be_negative`).
- **C18** — Bac à sable fini : pas de score dans la vue (`sandbox_has_no_final_score`).
- **C19** — La vue ne donne le score qu'une fois `GameEnded` appliqué (`final_score_only_after_game_end`).

Retour en arrière et sauvegarde :

- **C20** — Après C7, revenir en arrière : impossible avant `TileRevealed`, comme avant (spec 024) ; une décision prise après se
  défait sans toucher à « dernier tour annoncé » (`last_turn_survives_undo_after_reveal`).
- **C21** — Une sauvegarde faite pendant le dernier tour, rechargée : le tour est toujours le dernier, et sa fin termine la partie
  (`saved_game_keeps_last_turn`) — spec 024.
- **C22** — Une sauvegarde d'une partie terminée, rechargée : partie terminée, même score (`saved_finished_game_keeps_score`).

Interface (arcanum-ui, modèle d'écran) :

- **C23** — Configuration par défaut : Première reconnaissance, Arythea ; le panneau affiche le nom du scénario
  (`default_config_is_first_reconnaissance`).
- **C24** — Après C7, le bandeau `scenario.last_turn_next` ; pendant le dernier tour, la ligne `scenario.last_turn` sous la ligne de
  la manche (`panel_announces_last_turn`).
- **C25** — Partie terminée avec un score : la fenêtre de fin montre la raison, les sept lignes de l'exemple et le total ; « Voir le
  plateau » la ferme, le bouton « Score final » du panneau la rouvre (`end_window_shows_final_score`).
- **C26** — Bac à sable terminé : le message « Partie terminée : la dernière manche est jouée. » comme avant, sans fenêtre de score
  (`sandbox_end_keeps_last_round_message`).
- **C27** — Chaque clé nouvelle existe en français et en anglais (`every_scenario_key_exists_in_french`).

## Cas limites et refus

- **L1** — Une seule tuile de cité dans la pioche : `LastTurnAnnounced` ne peut venir qu'une fois. S'il y en avait deux (scénario de
  test), la seconde n'annonce rien.
- **L2** — Cité révélée pendant la 4e manche : même règle, le dernier tour se joue (la manche ne finit que par l'annonce).
- **L3** — Cité révélée pendant un tour qui finit la manche : impossible en solo, l'annonce remplace le tour (MKUE p. 6, 4).
- **L4** — La pioche de tuiles ne peut pas s'épuiser avant la cité : elle est sous les tuiles de campagne, avec les deux tuiles sans
  cité (spec 012). Si l'on n'explore pas jusqu'à elle, la partie finit par la limite de manches (C12).
- **L5** — Pendant le dernier tour, explorer rapporte toujours 1 gloire ; un changement de niveau gagné ce tour-là se fait à sa fin,
  avant `GameEnded`.
- **L6** — Sauvegarde d'avant cette spec (bac à sable) : elle se recharge, sans gloire de tuile ni dernier tour. Une partie déjà
  terminée avant cette spec n'a pas de `GameEnded` : elle se recharge à la fin de sa 4e manche, sans décision possible, comme avant
  (pas de migration avant la fin du J6, spec 024).
- **L7** — Partie terminée : Ctrl+S et Ctrl+O restent possibles ; Ctrl+Z ne fait rien (`GameEnded` est un point de non-retour).

## Information cachée

Tout est public : la gloire, le dernier tour et le score se lisent sur la table. Le score compte aussi les Blessures du deck, dont
l'ordre est caché : arcanum-core le calcule sur l'état complet, et il n'entre dans la vue qu'une fois la partie terminée (C19), quand plus
rien ne se pioche. Il ne révèle que des nombres (Blessures, cristaux), jamais l'ordre du deck.

## Libellés (i18n)

| Clé | Paramètres | fr | en |
|---|---|---|---|
| `scenario.first_reconnaissance.name` | — | Première reconnaissance | First Reconnaissance |
| `scenario.last_turn_next` | — | Cité révélée : après ce tour, encore un dernier tour. | City revealed: one last turn after this one. |
| `scenario.last_turn` | — | Dernier tour | Last turn |
| `score.title` | — | Fin de partie | Game over |
| `score.reason.city_revealed` | — | Une cité a été révélée et le dernier tour est joué. | A city has been revealed and the last turn is played. |
| `score.reason.last_round` | — | La dernière manche est jouée. | The last round has been played. |
| `score.fame` | `fame` | Gloire : {fame} | Fame: {fame} |
| `score.knowledge` | `points` | Le Savant : {points} | The Greatest Knowledge: {points} |
| `score.loot` | `crystals`, `points` | Le Collectionneur ({crystals} cristaux) : {points} | The Greatest Loot ({crystals} crystals): {points} |
| `score.leader` | `points` | Le Commandant : {points} | The Greatest Leader: {points} |
| `score.conqueror` | `points` | Le Conquérant : {points} | The Greatest Conqueror: {points} |
| `score.adventurer` | `points` | L'Aventurier : {points} | The Greatest Adventurer: {points} |
| `score.beating` | `wounds`, `points` | La Victime ({wounds} Blessure(s)) : {points} | The Greatest Beating ({wounds} Wound(s)): {points} |
| `score.total` | `score` | Score final : {score} | Final score: {score} |
| `score.no_titles` | — | Variante solo : pas de bonus de titre. | Solo variant: no title bonuses. |
| `score.show_board` | — | Voir le plateau | Show the board |
| `score.reopen` | — | Score final | Final score |

## Interface

- **Panneau latéral** : le nom du scénario en tête (nouveau : il n'apparaissait que dans la fenêtre de chargement) ; pendant le reste du tour qui révèle la cité, le bandeau
  `scenario.last_turn_next` ; pendant le dernier tour, la ligne `scenario.last_turn` sous « Manche 2/4 · Nuit (tour 3) ».
- **Fenêtre de fin** (partie terminée avec un décompte) : centrée sur le plateau, titre `score.title`, la raison, les sept lignes, le
  total en gras, `score.no_titles` en petit, et le bouton « Voir le plateau ». Fermée, le panneau garde le message de fin et un bouton
  « Score final » pour la rouvrir. Pas encore de « Nouvelle partie » (spec 065) : on quitte ou on charge une sauvegarde.

```
┌─ Fin de partie ─────────────────────────────────────┐
│ Une cité a été révélée et le dernier tour est joué. │
│                                                     │
│ Gloire : 17                                         │
│ Le Savant : 0                            │
│ Le Collectionneur (3 cristaux) : 1                │
│ Le Commandant : 0                              │
│ Le Conquérant : 0                        │
│ L'Aventurier : 0                        │
│ La Victime (2 Blessure(s)) : −4         │
│ Score final : 14                                    │
│ Variante solo : pas de bonus de titre.              │
│                                   [Voir le plateau] │
└─────────────────────────────────────────────────────┘
```

Vérification à l'œil : une partie préparée `cite-en-vue` (Première reconnaissance, la tuile de cité prochaine de la pioche, la
figurine au bord du plateau de jeu avec de quoi explorer) : explorer, voir la gloire et le bandeau, finir deux tours, voir la
fenêtre de fin, la fermer, la rouvrir.

### Raccourcis clavier

- **Échap** : fermer la fenêtre de fin (comme « Voir le plateau »), quand elle est ouverte. Reporté dans `docs/raccourcis.md`.
- Pas de touche pour la rouvrir : une fois par partie, le bouton suffit.

## Effets sur les specs déjà implémentées

- **012** : `Scenario::new` prend les trois champs nouveaux ; `Scenario::sandbox()` leur donne les valeurs neutres ; le tableau des
  prévisions de la 012 se lit avec cette spec pour la fin de partie, le décompte et la gloire par tuile.
- **023** : la fin de partie produit `GameEnded { LastRound }` ; le message `game.last_round` reste pour le bac à sable.
- **024** : `Event::blocks_undo` gagne `FameGained` et `LastTurnAnnounced` (non), `GameEnded` (oui).
- **Parcours 019, 028, 039** : bac à sable, inchangés, sauf la fin de partie qui voit `GameEnded` s'ils vont jusque-là.

## Hors périmètre

- Joueur fantôme, tactiques solo de *Solo Conquest* : specs 061 et 060 (MKUE p. 18, *Solo Variant*).
- Offre d'unités avec un village, pas d'unités d'élite, actions avancées 1 à 16, sorts et actions avancées révélés au besoin
  (MKUE p. 18 ; Compendium, *Scenarios* S10) : specs 050 et 042, qui liront le scénario.
- Bonus de titre (+3 / +1 / −3 / −1) et vainqueur : seulement en multijoueur, hors périmètre.
- Choisir le scénario et le héros : spec 065. Points de cité (*Co-op and Solo Scenarios* de la carte de score) : scénarios de
  conquête, specs 062 et 064.
- Combat entre joueurs : hors périmètre (PLAN.md).

## Compte rendu d'implémentation

- Le dernier tour est un état à trois valeurs (`LastTurn` : non annoncé, au tour suivant, en cours) : `LastTurnAnnounced` le passe
  à « au tour suivant », le `TurnStarted` qui suit à « en cours ». C'est ce qui distingue le bandeau `scenario.last_turn_next` de la
  ligne `scenario.last_turn`.
- `GameEnded` ferme aussi le tour pour le retour en arrière (comme `TurnEnded`) : la partie terminée se sauvegarde
  automatiquement, et Ctrl+Z dit « rien depuis le début du tour ».
- Le score est `final_score(state, joueur)` (`FinalScore` : gloire, cristaux, Blessures ; points par `Achievement`), donné par la
  vue seulement après `GameEnded` et si le scénario a un décompte. Un score négatif s'affiche « -4 ».
- Panneau : le message de fin est `game.last_round` pour la limite de manches, `game.finished` (« Partie terminée ») pour la cité ;
  la raison détaillée est dans la fenêtre de fin.
- Tests : `crates/arcanum-core/tests/premiere_reconnaissance.rs` (C1 à C20, L1, L5, L7), `crates/arcanum-persistance/tests/premiere_reconnaissance.rs`
  (C21, C22, L6), `crates/arcanum-ui/tests/premiere_reconnaissance.rs` (C23 à C27). L'arène des tests (`Sandbox`) prend un scénario.
- Tests existants adaptés : `fin_de_manche.rs` attend `GameEnded { LastRound }` après `RoundEnded` (effet prévu sur la 023) ;
  `scenario_affichage.rs` (spec 012) prend le bac à sable explicitement, la configuration par défaut ayant changé. Les parties
  préparées sont régénérées (trois champs de plus dans le scénario).
- Vérification à l'œil depuis `cite-en-vue` : captures `spec-063-charge`, `spec-063-cite-revelee`, `spec-063-dernier-tour`,
  `spec-063-fin-de-partie`, `spec-063-voir-plateau`, `spec-063-rouverte`, `spec-063-echap` (fichiers du projet, `captures/`).

## Questions ouvertes

Aucune. Décisions (Mathieu, 2026-10-30) :

- **Q1 — Dernier tour** : un tour de plus après celui où la cité est révélée (prise en main FR p. 19, « un tour supplémentaire »).
- **Q2 — Noms français** : ceux de la prise en main FR p. 19 (Haut Fait ; Le Savant, Le Collectionneur, Le Commandant, Le
  Conquérant, L'Aventurier, La Victime).
- **Q3 — Hauts Faits vides** : les six sont affichés, même à 0, pour que l'écran de fin garde sa forme.
