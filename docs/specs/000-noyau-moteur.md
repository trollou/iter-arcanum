# 000 — Noyau du moteur

- Statut : implémentée (2026-10-23)
- Jalon : J0
- Dépend de : —
- Livret : aucune règle de jeu (traduit les ADR 0001 et 0002)
- Crates touchées : arcanum-core (création du workspace et de la crate)

## Objectif

Poser la mécanique sur laquelle reposera toute règle : configuration de départ, actions, événements, `apply`,
actions légales, décision en attente, hasard à graine, vue joueur, journal et trait `Player`.
Pour la tester sans règle de Mage Knight, on utilise un **jeu jouet** minimal : un deck mélangé, et une seule action,
« piocher une carte ». Ce jeu jouet est provisoire (voir Hors périmètre).

## Règles du jeu jouet

- La partie se lance à partir d'une configuration de départ : une graine et la liste des cartes du deck.
- Au démarrage, le deck est mélangé ; la main est vide.
- Tant que le deck n'est pas vide, le joueur a une seule action légale : piocher la carte du dessus.
- Quand le deck est vide, la partie est finie : plus aucune décision en attente.
- Un seul joueur pour l'instant ; les identifiants de joueur existent déjà pour ne pas figer le solo.

## Modèle

Termes du glossaire : Deck (`DeedDeck`). Les cartes du jeu jouet sont de simples identifiants opaques (`CardId`).

Nouveaux termes à ajouter au glossaire (vocabulaire technique, pas de jeu) :

| Français | Code | Sens |
|---|---|---|
| Configuration de départ | `GameConfig` | Ce qui suffit à créer une partie : graine, et plus tard scénario, héros, variantes. |
| État de partie | `GameState` | Tout l'état, information cachée comprise. Clonable à bas coût. |
| Action | `Action` | Intention d'un joueur, à valider. |
| Événement | `Event` | Fait accompli, hasard déjà résolu. |
| Décision en attente | `PendingDecision` | Quel joueur doit choisir, et quel type de choix. |
| Vue joueur | `PlayerView` | Ce qu'un joueur a le droit de connaître de l'état. |
| Journal | `EventLog` | Suite ordonnée et numérotée des événements de la partie. |
| Journal des décisions | `DecisionLog` | Pour chaque décision : vue, actions légales, choix (données d'entraînement IA). |

Fonctions du moteur (signatures données en mots, le code est libre tant que ce contrat tient) :

- `new_game(config, rng)` → événements de démarrage.
- `pending_decision(state)` → décision en attente, ou aucune si la partie est finie.
- `legal_actions(state, player)` → liste exhaustive et ordonnée de façon déterministe.
- `decide(state, player, action, rng)` → événements, ou erreur typée si l'action n'est pas légale. Ne modifie pas l'état.
- `apply(state, event)` → nouvel état. Pure, infaillible, **sans générateur aléatoire** dans sa signature.
- `view(state, player)` → vue joueur ; `filter_event(event, player)` → l'événement tel que ce joueur a le droit de le voir.

Le générateur aléatoire est injecté (graine) et n'apparaît que dans `new_game` et `decide`.

## Actions et événements

| Action | Validation (refusée si…) | Événements produits |
|---|---|---|
| `DrawCard` | ce n'est pas au joueur de décider, ou le deck est vide | `CardDrawn { player, card }` |

| Événement | Effet de `apply` | Révèle une information ? |
|---|---|---|
| `GameStarted { config }` | initialise l'état (main vide, deck vide) | non |
| `DeckShuffled { player, order }` | le deck prend exactement cet ordre | oui ; pour le joueur, filtré en `DeckShuffled { player, count }` |
| `CardDrawn { player, card }` | retire la carte du dessus du deck, l'ajoute à la main | oui ; visible du joueur qui pioche, filtré en `CardDrawn { player, card: None }` pour les autres |

Chaque événement du journal porte un numéro de séquence et la version du format d'événement (constante, 1 pour l'instant).

## Trait `Player` et boucle de partie

Le trait ne doit bloquer ni l'UI ni la boucle, pour qu'un humain ou un LLM puissent répondre plus tard :

- `request(view, legal_actions)` : le moteur signale au joueur qu'une décision l'attend.
- `poll()` → `Some(index)` dans la liste des actions légales quand le choix est fait, `None` sinon.

Un joueur automatique simple répond dès le premier `poll`. L'humain (spec 002) mémorise la demande et répond quand l'UI a recueilli le clic.

La boucle de partie (`GameRunner`, dans arcanum-core, sans thread ni horloge) avance par `step()` :
s'il y a une décision en attente, elle interroge le joueur concerné ; si un choix arrive, elle le valide, produit et applique les événements,
les ajoute au journal et enregistre l'entrée du journal des décisions. `step()` indique s'il a avancé, s'il attend un joueur, ou si la partie est finie.

## Comportements attendus

- **C1** — Étant donné deux configurations identiques (même graine, mêmes cartes), quand on démarre deux parties, alors les états sont égaux, ordre du deck compris (`same_seed_gives_same_game`).
- **C2** — Étant donné deux graines différentes choisies pour le test, quand on démarre, alors l'ordre du deck diffère (`different_seeds_give_different_decks`).
- **C3** — Au démarrage, les événements produits sont `GameStarted` puis `DeckShuffled`, et l'ordre mélangé contient exactement les cartes de la configuration (`start_shuffles_all_configured_cards`).
- **C4** — Étant donné un deck non vide, la décision en attente est celle du joueur et `legal_actions` vaut exactement `[DrawCard]` (`draw_is_the_only_legal_action`).
- **C5** — Quand le joueur choisit `DrawCard`, `decide` produit `CardDrawn` avec la carte du dessus, sans modifier l'état (`decide_draws_top_card_without_mutating_state`).
- **C6** — `apply(CardDrawn)` retire la carte du dessus du deck et l'ajoute à la main (`apply_card_drawn_moves_card_to_hand`).
- **C7** — Étant donné un deck vide, il n'y a plus de décision en attente ni d'action légale (`empty_deck_ends_the_game`).
- **C8** — Rejouer le journal avec `apply` seul, sans générateur aléatoire, redonne exactement l'état final (`replaying_log_rebuilds_state`).
- **C9** — Deux états qui ne diffèrent que par l'ordre du deck donnent la même vue joueur ; la vue contient la main et le nombre de cartes du deck (`view_hides_deck_order`).
- **C10** — Le journal filtré pour le joueur montre `DeckShuffled` sans l'ordre, et `CardDrawn` avec la carte (`filtered_log_hides_shuffle_order`).
- **C11** — Une partie jouée jusqu'au bout par un joueur automatique de test via `GameRunner` se termine avec toutes les cartes en main, dans l'ordre du mélange (`runner_plays_toy_game_to_the_end`).
- **C12** — Avec un joueur dont `poll` renvoie `None`, `step()` rend la main en indiquant l'attente, sans modifier l'état ni le journal (`runner_waits_without_blocking`).
- **C13** — Chaque choix fait via `GameRunner` ajoute au journal des décisions une entrée avec la vue, les actions légales et l'index choisi (`runner_records_each_decision`).
- **C14** — Simuler une action sur une copie de l'état (pour le futur MCTS) ne modifie ni l'état d'origine ni le journal (`simulation_leaves_original_untouched`).

## Cas limites et refus

- **L1** — `decide(DrawCard)` sur un deck vide renvoie l'erreur `ActionError::NotLegal`, et aucun événement (`draw_on_empty_deck_is_refused`).
- **L2** — `decide` pour un joueur qui n'est pas celui de la décision en attente est refusé avec `ActionError::WrongPlayer` (`action_from_wrong_player_is_refused`).
- **L3** — Un joueur qui renvoie un index hors de la liste des actions légales : `step()` renvoie une erreur typée, rien n'est appliqué ni journalisé (`out_of_range_choice_is_refused`).
- **L4** — Configuration avec un deck vide : la partie démarre et est aussitôt finie (`empty_config_starts_finished_game`).

## Information cachée

La vue joueur contient : sa main, le nombre de cartes de son deck, et s'il a une décision en attente.
Elle ne contient jamais l'ordre du deck. Les tests C9 et C10 le garantissent.

## Libellés (i18n)

Aucun dans arcanum-core pour cette spec. Les actions seront identifiées par une clé (ex. `action.draw_card`) quand l'UI en aura besoin, en spec 002.

## Interface

Aucune. La fenêtre arrive en spec 002.

## Hors périmètre

- Retour en arrière, instantanés, sauvegarde, migration des versions d'événements : spec 024.
- Plusieurs joueurs, joueur fantôme : J6.
- Le jeu jouet et l'action `DrawCard` disparaissent quand de vraies actions existent (specs 012 à 016) ; leurs tests sont alors remplacés par des tests de règles équivalents (rejeu, vue, refus).
- Sérialisation des événements (serde) : spec 024.

## Décisions prises à la validation

1. Dépendances acceptées : `rand`, `rand_chacha` (ChaCha8), `thiserror`.
2. Jeu jouet provisoire accepté, supprimé dès que les vraies actions arrivent.
3. L2 renvoie une erreur distincte, `ActionError::WrongPlayer`, plutôt que `NotLegal` (précision de l'implémentation, confirmée par Mathieu le 2026-10-23).
4. Pour un autre joueur que celui qui pioche, `CardDrawn` est filtré sans la carte (précision de l'implémentation, confirmée par Mathieu le 2026-10-23).
