# 024 — Sauvegarde et retour en arrière

- Statut : implémentée (2026-10-27)
- Jalon : J2
- Dépend de : 000 (journal, `GameRunner`), 012 (configuration de départ dans `GameStarted`), 015, 016, 020, 022, 023, 025
  (les événements qui révèlent une information)
- Livret : MKUE p. 5, *Reverting* 1, 2 et 2.a. ADR 0001 (event sourcing), ADR 0004 (portabilité)
- Crates touchées : arcanum-core, **arcanum-persistance (créée)**, arcanum-ui, `assets/i18n/` (fr.json et en.json)

## Objectif

Le joueur peut **revenir en arrière** pendant son tour, action par action, jusqu'à la dernière information révélée (tuile, carte
piochée, dé relancé) ou jusqu'au début du tour. Il peut **sauvegarder** la partie dans un fichier, la **recharger** plus tard
(tant que le jeu n'a pas été mis à jour : pas de migration avant la fin du J6), et une **sauvegarde automatique** est écrite au début de chaque tour. Une partie rechargée
continue exactement comme si on ne l'avait jamais quittée : mêmes tuiles, mêmes cartes, mêmes dés.

## Ce que les autres specs ont renvoyé ici

| Spec | Renvoi | Réponse de cette spec |
|---|---|---|
| 000 | Retour en arrière, instantanés, sauvegarde, migration des versions ; sérialisation (serde) des événements | Tout est ici (Modèle, C1 à C27), sauf les migrations : pas avant la fin du J6 (décision de Mathieu, 2026-10-27). |
| 002 | Sauvegarde et annulation hors de la fenêtre minimale | Boutons et raccourcis (Interface). |
| 012 | La configuration de départ est dans `GameStarted` ; `EVENT_FORMAT_VERSION` laissée à 1 | La sauvegarde n'enregistre pas la configuration à part : elle est dans le premier événement. La version reste à 1 et n'est pas tenue pendant le développement : pas de migration avant la fin du J6 (voir « Versions »). |
| 013 | Reprendre une carte jouée (*Reverting*, MKUE p. 5) | Revenir en arrière remet la carte en main (C1). |
| 014 | Annuler un déplacement | C2. |
| 015 | Une exploration ne s'annule jamais | `TileRevealed` est un point de non-retour (C5). |
| 016 | Une fin de tour ne s'annule pas | Fin du tour = point de non-retour, pioche ou non (C6). |
| 022 | Relancer un dé à la fin du tour est une révélation | `DieReturned` est un point de non-retour (C6). |
| 023 | Une annonce de fin de manche ne s'annule pas | Mélange et relance de la Source : points de non-retour (C7). |
| 025 | Un jeu de carte qui met un dé sur une couleur (`DieSet`) : pas de hasard, donc annulable « à confirmer » | Confirmé : annulable (C3). Prendre un dé de plus aussi. |
| 029 | La sauvegarde ne dépend pas du format des fichiers de contenu | Les événements ne portent que des identifiants (ADR 0005) ; voir L9. |

## Règles

- **Revenir en arrière** (MKUE p. 5, *Reverting* 1) : on peut reprendre les actions et décisions **de son tour**. Le jeu le permet
  toujours (le livret dit « sauf accord contraire » : pas d'option pour l'interdire). Le tour se reprend **une décision à la fois**,
  dans l'ordre inverse, autant de fois qu'on veut jusqu'au point de non-retour.
- **Point de non-retour** (MKUE p. 5, *Reverting* 2 et 2.a) : on ne revient pas avant une information révélée (tuile, jeton
  d'ennemi, carte) ni avant un dé lancé. La décision qui a révélé ne se reprend pas, ni rien avant elle. « Un autre joueur qui
  réagit » (même paragraphe) ne concerne pas le solo ; le joueur fantôme (spec 061) jouera entre les tours, déjà protégés par la règle
  suivante.
- **Fin du tour** (MKUE p. 5, *Reverting* 1, « during their turn ») : finir son tour est toujours un point de non-retour, même si
  rien n'est pioché ni relancé (deck vide, aucun dé pris). La mise en place et le début d'une manche aussi.
- **Combat** (MKUE p. 9, *Attack Phase* 3.a, « take back the cards you played ») : même mécanisme ; les points de non-retour propres
  au combat (ennemi révélé, phase validée) seront fixés par les specs 030 et 031.
- **Sauvegarde et format** (ADR 0001, `.claude/rules/arcanum-persistance.md`) : pas dans le livret. Ce qui est enregistré, et la révision
  de l'ADR sur les instantanés : voir Modèle et Q1.

### Schéma : un tour et ses points de non-retour

```
 TurnStarted ─┬─ D1 jouer Marche ─┬─ D2 se déplacer ─┬─ D3 explorer ─┬─ D4 se déplacer ─┬─ D5 finir le tour
   (non-retour)                   │                  │ TileRevealed   │                  │ CardDrawn, DieReturned…
              │                   │                  │ (non-retour)   │                  │ (non-retour)
              ◄── on revient jusqu'ici après D1, D2 ─┘                ◄── après D4, on ne revient qu'ici
```

Chaque décision (D1, D2…) est une action choisie par le joueur, avec **tous** les événements qu'elle a produits. Revenir en arrière
retire la dernière décision entière : pour Improvisation, la carte jouée et la carte défaussée reviennent ensemble.

## Modèle

### Termes

Termes déjà présents : Journal (`EventLog`), Journal des décisions (`DecisionLog`), Configuration de départ (`GameConfig`), Boucle de
partie (`GameRunner`), Révéler (`TileRevealed`).

Nouveaux termes, ajoutés au glossaire dans la même PR (Q8, validés) :

| Livret (EN) | Français | Code |
|---|---|---|
| Reverting | Revenir en arrière | `undo` (`GameRunner::undo`) |
| — | Point de non-retour | `Event::blocks_undo` |
| — | Numéro de la décision | `decision` (dans `LoggedEvent`) |
| — | Sauvegarde | `SaveGame` (arcanum-persistance) |
| — | Sauvegarde automatique | `AUTOSAVE_NAME` |
| — | Résumé de la sauvegarde | `SaveSummary` |
| — | Dossier des sauvegardes | `saves_dir` |
| — | Position du générateur | `rng_position` |

### Le journal connaît ses décisions

Aujourd'hui, `EventLog` est une suite plate d'événements : on ne sait pas lesquels viennent de la même décision. Chaque entrée porte
maintenant le **numéro de la décision** qui l'a produite : **0** pour la mise en place (`new_game`), puis 1, 2, 3… à chaque action
choisie par un joueur. Revenir en arrière retire toutes les entrées du dernier numéro. Comme ce numéro est dans le journal, on peut
revenir en arrière **après un rechargement** exactement comme avant (C14).

Le numéro de version par entrée (`LoggedEvent::version`, toujours égal à `EVENT_FORMAT_VERSION`) disparaît : la version est écrite une
fois, en tête du fichier.

### Points de non-retour

Un événement est un **point de non-retour** s'il révèle une information, lance un dé ou ferme le tour. La décision qui en contient un
ne peut plus être reprise, ni aucune avant. `Event::blocks_undo` le dit par un `match` **sans joker** : un nouvel événement ne compile
pas tant que sa spec n'a pas tranché. Chaque spec future remplit pour cela une colonne « Point de non-retour ? » dans sa table des
événements (le gabarit est complété dans cette PR).

| Événement | Point de non-retour ? | Pourquoi |
|---|---|---|
| `GameStarted`, `TilePlaced`, `TileDeckFormed`, `HeroFigurePlaced` | oui | mise en place |
| `DeckShuffled` | oui | hasard |
| `CardDrawn` | oui | carte révélée (p. 5, *Reverting* 2) |
| `TileRevealed` | oui | tuile révélée (idem) |
| `SourceRolled`, `DieReturned` | oui | dé lancé (idem) |
| `TurnEnded`, `TurnStarted`, `RoundEnded`, `RoundStarted` | oui | fin du tour (p. 5, *Reverting* 1) |
| `CardPlayed`, `CardDiscarded`, `HeroMoved`, `Explored`, `CrystalGained`, `DieTaken`, `CrystalUsed`, `ManaTokenGained`, `ManaTokenUsed`, `CardReturnedToHand`, `DieSet` | non | rien de caché, aucun hasard (`Explored` paie, la tuile vient avec `TileRevealed`) |

**Le hasard n'est tiré que dans des décisions de non-retour.** Aujourd'hui, `decide` ne se sert du générateur que pour la Source
(relance des dés) et le mélange du deck. Revenir en arrière ne rembobine donc jamais le générateur : rejouer la même action après
être revenu en arrière donne le même résultat. Un test le garantit pour toute décision qui n'est pas de non-retour (C8).

### Ce que garde la boucle de partie

- `GameRunner::undo()` retire la dernière décision : état, journal **et** journal des décisions reviennent exactement à ce qu'ils
  étaient avant elle (Q3). Il renvoie une erreur typée s'il n'y a rien à reprendre (L1).
- `GameRunner::undo_status()` dit si l'on peut revenir en arrière, et sinon pourquoi : **rien depuis le début du tour** ou **une
  information a été révélée** (texte de l'info-bulle).
- Revenir en arrière n'est **pas une action** : il n'est pas dans `legal_actions`, aucun `Player` ne le choisit (Q4). Seule l'UI
  l'appelle, pour le joueur humain. Le joueur à qui revient la décision est interrogé de nouveau (C10).
- Pour ne pas rejouer toute la partie, la boucle garde **en mémoire** l'état au dernier point de non-retour et les états qui ont
  suivi (instantanés en mémoire). C'est un choix d'implémentation : le test ne regarde que l'égalité des états.

### Une sauvegarde

```
iter-arcanum/saves/2026-10-26_21-40-52.json
{
  "version": 1,                          ← EVENT_FORMAT_VERSION, une fois pour tout le fichier
  "summary": {                           ← pour la liste des sauvegardes, jamais relu pour jouer
    "saved_at": "2026-10-26T21:40:52+01:00",
    "scenario": "Sandbox", "hero": "Arythea",
    "round": 2, "round_limit": 4, "time_of_day": "Night", "turn": 3
  },
  "rng_position": "123456",              ← position du générateur, en texte
  "events": [                            ← le journal entier, avec les numéros de décision
    { "decision": 0, "event": { "GameStarted": { "config": { "seed": 42, "scenario": {…}, "hero": "Arythea" } } } },
    { "decision": 0, "event": { "TilePlaced": {…} } },
    …
    { "decision": 57, "event": { "HeroMoved": { "player": 0, "from": [1, 2], "to": [2, 2], "cost": 2 } } }
  ]
}
```

- **Configuration + journal + position du générateur.** La configuration de départ est dans `GameStarted` (spec 012). Le journal
  rejoué par `apply` redonne l'état ; la position du générateur (ChaCha8, `get_word_pos`) fait que la suite tire les **mêmes** dés et
  mélanges que si on n'avait pas quitté (C13). Elle est écrite en texte : c'est un entier de 128 bits, que JSON lit mal ailleurs
  qu'en Rust.
- **Pas d'instantané d'état dans le fichier** (Q1). L'ADR 0001 et `.claude/rules/arcanum-persistance.md` prévoyaient des instantanés dans
  la sauvegarde. Décision : ne pas les écrire. `GameState` change à chaque spec ; une fois les migrations en place (après le J6), l'écrire en obligerait une à chaque spec,
  alors que le journal ne change que quand la **forme** d'un événement change. Rejouer une partie entière (quelques milliers
  d'événements) prend quelques millisecondes. L'ADR et les règles de la crate sont révisés en conséquence.
- **Format JSON**, UTF-8, indenté, `serde_json` (déjà utilisé par arcanum-i18n) : lisible, et lisible hors du jeu (outils d'analyse, données
  d'entraînement des IA). RON reste pour le contenu du jeu (Q2).
- **Le journal des décisions n'est pas sauvegardé** : il contient des vues et des actions dont la forme change souvent ; il sera
  enregistré par la spec des joueurs automatiques qui en aura besoin. Après un rechargement, il repart vide.

### Versions : pas de migration avant la fin du J6

Décision de Mathieu (2026-10-27, relecture de la PR) : **on ne gère pas les versions pendant le développement**. Tant que le jalon J6
n'est pas franchi, une sauvegarde peut devenir illisible après une mise à jour du jeu, et c'est accepté.

- `EVENT_FORMAT_VERSION` **reste à 1** jusqu'à la fin du J6, même quand un événement change de forme. Le fichier porte quand même
  `"version": 1` : un champ qui ne coûte rien et servira après le J6.
- Pas de fichier témoin, pas de migration, pas d'obligation d'augmenter la version quand un événement change de forme.
- Une sauvegarde qui ne se lit plus (forme d'un événement changée) ou dont la version n'est pas celle du jeu est **refusée** avec un
  message, sans toucher à la partie en cours (L5).
- Après le J6, une spec dédiée fixera la version de référence, le fichier témoin par version et les migrations (Hors périmètre).

### Crate `arcanum-persistance` (créée)

Lire et écrire des sauvegardes dans un dossier donné, sans rien savoir de l'UI ni du choix du dossier :

- `save(dossier, nom, &SaveGame)`, `load(chemin) -> Result<SaveGame, LoadError>`, `list(dossier) -> Vec<(chemin, SaveSummary)>`
  (les fichiers illisibles sont listés avec leur erreur, pas ignorés en silence) ;
- `SaveGame::from_runner(&GameRunner, horodatage)` et `GameRunner::resume(SaveGame, joueurs)` (le second dans arcanum-core, qui ne dépend
  pas d'arcanum-persistance : il prend le journal et la position du générateur) ;
- erreurs typées (`thiserror`) : fichier illisible, JSON invalide ou d'une forme ancienne, autre version que celle du jeu, journal qui ne commence pas par
  `GameStarted`, numéros de décision qui décroissent.

Dépendances : arcanum-core, `serde`, `serde_json`, `thiserror`, `log` ; `tempfile` en test. **Aucune nouvelle dépendance externe** : toutes
sont déjà dans le workspace. arcanum-core ajoute `Serialize`/`Deserialize` à `Event` et aux types qu'il porte (`serde` y est déjà).

### Où sont les sauvegardes

Comme le dossier des logs (spec 003), choisi par arcanum-ui au lancement, dans cet ordre :

1. `ARCANUM_SAVE_DIR` si la variable est définie et non vide ;
2. lancé par cargo : dossier `saves/` à la racine du dépôt (ajouté au `.gitignore`) ;
3. sinon `saves/` dans le dossier de données de l'utilisateur (`directories`, même application que les logs).

**Nom des fichiers** : date et heure locales, `AAAA-MM-JJ_HH-MM-SS.json` (pas de `:`, interdit sous Windows), suffixe `-2`, `-3`…
si le nom existe déjà. La sauvegarde automatique s'appelle `auto.json` et est remplacée à chaque fois.

## Actions et événements

Aucune action ni aucun événement de jeu nouveau : revenir en arrière, sauvegarder et charger ne passent pas par `decide`.

| Événement | Effet de `apply` | Point de non-retour ? |
|---|---|---|
| tous | inchangé | selon la table ci-dessus |

## Comportements attendus

### Revenir en arrière (arcanum-core, `tests/retour_en_arriere.rs`)

- **C1** — Étant donné une carte jouée pour son effet de base au début du tour, quand le joueur revient en arrière, alors la carte
  est en main, les points à 0, la zone de jeu vide, et l'état est **égal** à celui d'avant (`undo_play_card_restores_state`) —
  MKUE p. 5, *Reverting* 1.
- **C2** — Étant donné un déplacement, quand on revient en arrière, alors la figurine est sur sa case de départ et les points
  dépensés sont rendus (`undo_move_restores_figure_and_points`) — MKUE p. 5, *Reverting* 1.
- **C3** — Étant donné une carte renforcée payée par un dé, un cristal ou un jeton, ou Source de mana renforcée qui a mis un dé sur une
  couleur, quand on revient en arrière, alors le dé redevient libre avec sa couleur, le cristal ou le jeton est rendu
  (`undo_restores_mana_payment`, `undo_restores_set_die`) — MKUE p. 5, *Reverting* 2.a.
- **C4** — Étant donné une décision qui a produit plusieurs événements (Improvisation et sa défausse), quand on revient en arrière,
  alors tous ses événements sont retirés, et ceux-là seulement (`undo_removes_whole_decision`).
- **C5** — Étant donné une exploration, quand on demande de revenir en arrière, alors c'est refusé (« information révélée ») ; après
  un déplacement qui la suit, on peut reprendre ce déplacement, puis plus rien (`undo_stops_at_revealed_tile`) — MKUE p. 5,
  *Reverting* 2.
- **C6** — Étant donné une fin de tour, avec ou sans pioche ni dé relancé, quand on demande de revenir en arrière au tour suivant
  avant toute action, alors c'est refusé (« rien depuis le début du tour ») (`undo_stops_at_turn_start`) — MKUE p. 5, *Reverting* 1.
- **C7** — Étant donné l'annonce de la fin de la manche, alors elle ne se reprend pas (`undo_stops_at_round_start`) — MKUE p. 5,
  *Reverting* 2.
- **C8** — Pour chaque événement, `blocks_undo` suit la table ; et après toute décision qui n'est pas de non-retour, la position du
  générateur n'a pas bougé (`only_no_return_decisions_use_rng`).
- **C9** — Étant donné trois décisions reprises puis rejouées à l'identique, alors l'état, le journal et le journal des décisions
  sont égaux à ceux d'une partie où l'on n'est jamais revenu en arrière (`undo_then_redo_is_transparent`).
- **C10** — Étant donné un joueur à qui une décision a été demandée, quand on revient en arrière, alors la boucle lui redemande la
  décision d'avant (`undo_asks_player_again`).
- **C11** — Après être revenu en arrière, le journal rejoué depuis le début redonne l'état courant (`replay_after_undo_matches_state`).
- **C12** — Revenir en arrière n'est jamais une action légale (`undo_is_not_a_legal_action`).

### Sauvegarder et charger (arcanum-persistance, `tests/sauvegarde.rs`)

- **C13** — Étant donné une partie jouée sur plusieurs tours, quand on la sauvegarde, la recharge et joue les mêmes actions qu'une
  copie jamais sauvegardée, alors les deux journaux sont identiques, fin de tour (dés relancés) et fin de manche (mélange) comprises
  (`reloaded_game_draws_same_randomness`) — ADR 0001.
- **C14** — Étant donné une sauvegarde faite au milieu d'un tour, quand on la recharge, alors on peut revenir en arrière jusqu'au même
  point de non-retour qu'avant (`undo_works_after_reload`).
- **C15** — Sauvegarder puis charger redonne la même sauvegarde, champ par champ (`save_load_round_trip`).
- **C16** — L'état rechargé est égal à l'état sauvegardé (`reloaded_state_equals_saved_state`).
- **C17** — Le résumé porte scénario, héros, manche, nombre de manches, moment de la journée, tour (`summary_describes_game`).
- **C18** — Deux sauvegardes dans la même seconde ne s'écrasent pas (`same_second_saves_get_suffix`).
- **C19** — La liste des sauvegardes d'un dossier est triée de la plus récente à la plus ancienne, la sauvegarde automatique en tête ;
  un fichier illisible y figure avec son erreur (`list_sorts_and_reports_unreadable`).
- **C20** — Une sauvegarde dont la version n'est pas celle du jeu est refusée avec `LoadError::OtherVersion`, la partie en cours est
  inchangée (`other_version_is_refused`).
- **C21** — Une sauvegarde dont un événement n'a plus la forme attendue est refusée avec `LoadError::Invalid`, sans panique
  (`outdated_event_shape_is_refused`).
- **C22** — Les sauvegardes sont du JSON UTF-8 lisible ; un chemin avec espaces et accents fonctionne sous Windows et Linux
  (`save_path_with_spaces_and_accents`) — ADR 0004.

### Interface (arcanum-ui, `tests/sauvegarde_ui.rs`, sur `GameSession` et `game_screen`)

- **C23** — Au début du tour, le bouton « Revenir en arrière » est grisé, avec l'info-bulle « rien depuis le début du tour » ;
  après une carte jouée, il est actif ; après une exploration, il est grisé avec « une information a été révélée »
  (`undo_button_follows_status`).
- **C24** — Ctrl+Z revient en arrière hors des modes de choix ; dans un mode (défausse, mana, fin du tour…), ni le bouton ni la touche
  n'agissent (`undo_shortcut_outside_modes_only`).
- **C25** — Au début de chaque tour, `auto.json` est écrit ; il se recharge sur le même état (`autosave_at_turn_start`).
- **C26** — Charger une sauvegarde remplace la partie en cours ; le panneau montre la manche, le tour et la main de la sauvegarde
  (`loading_replaces_game`).
- **C27** — Le dossier des sauvegardes suit l'ordre `ARCANUM_SAVE_DIR`, dépôt, dossier de l'utilisateur (`saves_dir_choice`), comme les
  logs (spec 003).

## Cas limites et refus

- **L1** — Revenir en arrière sans rien à reprendre (début de partie, début du tour, juste après une révélation) : erreur typée, rien
  ne change.
- **L2** — Partie terminée (après la dernière manche) : rien à reprendre, `RoundEnded` est un point de non-retour.
- **L3** — Fichier absent, illisible ou pas du JSON : `LoadError`, la partie en cours n'est pas touchée, message dans la fenêtre de
  chargement.
- **L4** — Journal vide ou qui ne commence pas par `GameStarted`, numéros de décision qui décroissent : refusé.
- **L5** — Sauvegarde d'une autre version du jeu, ou devenue illisible après une mise à jour : refusée (« sauvegarde d'une autre
  version du jeu »). Accepté jusqu'à la fin du J6 (pas de migration).
- **L6** — Écriture impossible (dossier en lecture seule, disque plein) : erreur dans les logs techniques et message dans le panneau ;
  la partie continue. Pour la sauvegarde automatique, même message jusqu'à la prochaine réussite.
- **L7** — Sauvegarder, charger et revenir en arrière ne sont proposés que hors des modes de choix (défausse, mana, dés, fin du tour,
  confirmation d'annonce) : le mode est propre à l'UI et ne se sauvegarde pas.
- **L8** — Fichier modifié à la main : il est rejoué tel quel par `apply`, qui est infaillible ; l'état peut être absurde. Pas de
  contrôle anti-triche en solo (Information cachée).
- **L9** — Contenu du jeu modifié depuis la sauvegarde (une carte change de valeur dans un `.ron`) : les événements portent déjà les
  effets résolus (`CardPlayed` porte son effet), le passé ne change pas ; la suite utilise le nouveau contenu. Pas de version du
  contenu dans la sauvegarde.

## Information cachée

- Revenir en arrière ne fait jamais **oublier** une information : on ne revient pas avant une révélation, donc rien de ce que le
  joueur a vu ne redevient caché.
- La vue joueur gagne le **statut du retour en arrière** (possible, ou la raison du refus) : il ne dit rien de caché.
- Le fichier de sauvegarde contient **tout** l'état, information cachée comprise (ordre du deck et de la pioche de tuiles, position du
  générateur). Un joueur qui l'ouvre peut tricher ; accepté en solo.

## Libellés (i18n)

| Clé | Paramètres | fr | en |
|---|---|---|---|
| `undo.button` | — | Revenir en arrière | Undo |
| `undo.hint` | — | Reprendre votre dernière action (Ctrl+Z) | Take back your last action (Ctrl+Z) |
| `undo.blocked.turn_start` | — | Rien à reprendre depuis le début du tour. | Nothing to take back since the start of the turn. |
| `undo.blocked.revealed` | — | Une information a été révélée (tuile, carte ou dé) : impossible de revenir avant. | Information was revealed (tile, card or die): you cannot go back before it. |
| `save.button` | — | Sauvegarder | Save |
| `save.hint` | — | Sauvegarder la partie (Ctrl+S) | Save the game (Ctrl+S) |
| `save.done` | `name` | Partie sauvegardée : {name} | Game saved: {name} |
| `save.failed` | — | Sauvegarde impossible (voir les logs). | Could not save (see the logs). |
| `save.auto_failed` | — | Sauvegarde automatique impossible (voir les logs). | Autosave failed (see the logs). |
| `load.button` | — | Charger… | Load… |
| `load.hint` | — | Charger une partie (Ctrl+O) | Load a game (Ctrl+O) |
| `load.title` | — | Charger une partie | Load a game |
| `load.empty` | — | Aucune sauvegarde. | No saved game. |
| `load.auto` | — | Sauvegarde automatique | Autosave |
| `load.entry` | `date`, `scenario`, `hero`, `line` | {date} · {scenario} · {hero} · {line} | {date} · {scenario} · {hero} · {line} |
| `load.load` | — | Charger | Load |
| `load.close` | — | Fermer | Close |
| `load.failed` | — | Chargement impossible (voir les logs). | Could not load (see the logs). |
| `load.unreadable` | `name`, `reason` | {name} · {reason} | {name} · {reason} |
| `hero.<héros>.name` | — | Arythea, Goldyx, Norowas, Tovak | (idem) |
| `load.error.unreadable` | — | Fichier illisible. | Unreadable file. |
| `load.error.other_version` | — | Sauvegarde d'une autre version du jeu. | Saved by another version of the game. |
| `load.error.invalid` | — | Sauvegarde invalide. | Invalid save. |

Les noms de scénario, de héros et le moment de la journée reprennent les clés existantes.

## Interface

Une ligne de trois boutons en bas du panneau latéral, hors des modes de choix (L7) :

```
┌─ Panneau latéral ──────────────────────────────────────┐
│ Manche 2/4 · Nuit (tour 3)                             │
│ …                                                      │
│ [↶ Revenir en arrière] [Sauvegarder] [Charger…]        │
│ Partie sauvegardée : 2026-10-26_21-40-52               │
└────────────────────────────────────────────────────────┘
```

- **Revenir en arrière** : actif si `undo_status` le permet ; grisé sinon, l'info-bulle donnant la raison (C23).
- **Sauvegarder** : écrit un nouveau fichier ; une ligne confirme le nom (ou l'échec) jusqu'à la prochaine action.
- **Charger…** : ouvre une fenêtre avec la liste des sauvegardes (C19), une ligne par fichier avec son résumé et un bouton
  « Charger » ; un fichier illisible est grisé avec son erreur. Charger ferme la fenêtre et remplace la partie, sans confirmation
  (Q5). « Fermer » ou Échap ferme la fenêtre.

```
┌─ Charger une partie ──────────────────────────────────────────────────┐
│ Sauvegarde automatique · Bac à sable · Arythea · Manche 2/4 · Nuit (tour 3)  [Charger] │
│ 2026-10-26 21:40 · Bac à sable · Arythea · Manche 2/4 · Nuit (tour 2)        [Charger] │
│ 2026-10-25 18:02 · Fichier illisible.                                                   │
│                                                                    [Fermer]            │
└───────────────────────────────────────────────────────────────────────┘
```

Au lancement, le jeu démarre toujours une nouvelle partie, comme aujourd'hui ; reprendre la dernière partie au lancement viendra avec
l'écran de nouvelle partie (spec 065).

### Raccourcis clavier

Reportés dans `docs/raccourcis.md` (Q7). Ce sont les premiers avec Ctrl ; ils n'agissent pas dans les modes de choix (L7).

| Touche | Effet | Quand |
|---|---|---|
| Ctrl+Z | Revenir en arrière | Partie, hors mode de choix, si c'est possible |
| Ctrl+S | Sauvegarder | Partie, hors mode de choix |
| Ctrl+O | Ouvrir la fenêtre de chargement | Partie, hors mode de choix |
| Échap | Fermer la fenêtre de chargement | Fenêtre de chargement ouverte |

## Hors périmètre

- **Rejouer** une décision reprise (Ctrl+Y) : on la rejoue à la main, le résultat est le même (C9).
- **Retour en arrière libre** au-delà des points de non-retour : abandonné, retiré de l'ADR 0001 (décision de Mathieu, 2026-10-27).
- Supprimer ou renommer une sauvegarde depuis le jeu, choisir un fichier ailleurs (sélecteur de fichiers) : plus tard.
- Reprendre la dernière partie au lancement, écran de nouvelle partie : spec 065.
- Enregistrer le journal des décisions (données d'entraînement) : spec du premier joueur automatique.
- Points de non-retour du combat (ennemi révélé, phase validée) : specs 030 et 031. Joueur fantôme : spec 061.
- Versions de référence, fichier témoin par version et migrations : après le J6, dans une spec dédiée (décision de Mathieu,
  2026-10-27).
- Parcours de bout en bout du jalon (sauvegarde, rechargement et retour en arrière dans une vraie partie) : spec 028.

## Questions ouvertes

Aucune.

## Corrections pendant l'implémentation

Signalées ici parce que le sens de tests existants change (règle « jamais modifier un test sans le signaler ») :

- **000** : `log_entries_carry_sequence_and_version` devient `log_entries_carry_sequence_and_decision`. Le numéro de version par
  entrée disparaît (Modèle, « Le journal connaît ses décisions ») ; le test vérifie maintenant le numéro de décision, et que
  `EVENT_FORMAT_VERSION` vaut toujours 1.
- **000, 020** : `EventLog::push` prend le numéro de la décision ; les tests qui construisent un journal à la main passent 0
  (mise en place).

Écarts de la partie C (arcanum-ui) par rapport au texte validé, sans changement de comportement :

- `load.entry` prend un seul paramètre `line`, construit avec la clé existante `turn.line` (« Manche 2/4 · Nuit (tour 3) ») : la
  ligne du panneau et celle de la fenêtre de chargement ne peuvent pas diverger.
- Les héros n'avaient pas de libellé : `Hero::label_key` et les clés `hero.<héros>.name` sont ajoutés.
- Info-bulles `save.hint` et `load.hint` (elles donnent le raccourci), `load.failed` (chargement qui échoue malgré un résumé lisible,
  affiché comme ligne de message sous les boutons) et `load.unreadable` (nom du fichier et raison).
- L'état des boutons est calculé par `GameSession::controls` (module `controls.rs`) et non dans `game_screen` : il dépend de la
  boucle de partie (`undo_status`) et du dossier des sauvegardes, pas de la vue du joueur.
- Vu à la vérification à l'œil : la fenêtre de chargement n'est pas modale, les boutons de la main restaient cliquables. Tant
  qu'elle est ouverte, la session ignore toute commande autre que les siennes (L7, `load_window_blocks_other_commands`).
- Vérification à l'œil : `captures/spec-024-debut.png`, `spec-024-sauvegarde.png`, `spec-024-fenetre-chargement.png`.

## Décisions prises à la relecture

Réponses de Mathieu (2026-10-27) :

- **Versions** : on reste en version 1 et sans migration pendant le développement, tant que le J6 n'est pas franchi. Une sauvegarde
  devenue illisible est refusée proprement (section « Versions », C20, C21, L5).
- **Q6** : pas de mode debug d'annulation libre ; retiré de l'ADR 0001.
- **Q1** : pas d'instantané d'état dans la sauvegarde (journal + position du générateur) ; instantanés en mémoire pour revenir en
  arrière. ADR 0001 et `.claude/rules/arcanum-persistance.md` révisés.
- **Q2** : format JSON.
- **Q3** : revenir en arrière efface la décision du journal et du journal des décisions.
- **Q4** : revenir en arrière n'est pas une action du jeu, réservé à l'UI.
- **Q5** : sauvegardes dans le dossier de l'utilisateur, sans sélecteur de fichiers ; sauvegarde automatique au début de chaque tour ;
  chargement sans confirmation.
- **Q7** : raccourcis Ctrl+Z, Ctrl+S, Ctrl+O ; Échap ferme la fenêtre de chargement.
- **Q8** : termes « Revenir en arrière », « Point de non-retour », « Sauvegarde automatique » et les autres du glossaire.
