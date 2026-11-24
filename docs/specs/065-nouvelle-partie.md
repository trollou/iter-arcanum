# 065 — Nouvelle partie

- Statut : implémentée (validée le 2026-10-30, Q1 à Q4 tranchées par Mathieu)
- Jalon : J4
- Dépend de : 002 (fenêtre, graine tirée de l'horloge), 012 (configuration de départ), 024 (sauvegarde, fenêtre de chargement),
  063 (Première reconnaissance, fenêtre de fin) ; toutes implémentées
- Livret : MKUE p. 1 (*Game Flow*, 1 et 2) ; decks de départ et figurine p. 3 (`docs/reference/decks-de-depart.md`)
- Crates touchées : arcanum-core (listes des scénarios et des héros, cartes propres d'un héros), arcanum-ui, `assets/i18n/` (fr.json et en.json),
  `assets/images/hero/` (une illustration par héros, demandée par Mathieu le 2026-10-30)

## Objectif

Au lancement, le joueur choisit sa partie au lieu de recevoir Première reconnaissance avec Arythea : un **écran de nouvelle partie**
propose le héros (les quatre du jeu de base), le scénario (Première reconnaissance ou bac à sable) et, s'il le veut, une graine. Le
même écran s'ouvre en cours de partie et depuis la fenêtre de fin, pour enchaîner une autre partie sans relancer le jeu. Chaque héros a sa propre **illustration** : sa figurine sur le plateau de
jeu, son portrait en tête des données du joueur dans le panneau latéral, et dans l'écran de nouvelle partie.

## Règles

- **Choisir un scénario** (MKUE p. 1, *Game Flow*, 1) : deux scénarios jouables, Première reconnaissance (spec 063) et le bac à
  sable (spec 012, absent du livret). Les variantes (*You may also use any variant rules*) restent hors périmètre (PLAN.md).
- **Choisir un héros** (MKUE p. 1, *Game Flow*, 2) : en solo, pas d'ordre des joueurs ni de jeton d'ordre de manche à poser ; le
  joueur prend l'un des quatre héros. Aujourd'hui un héros ne diffère des autres que par son deck de départ, deux cartes propres à
  la place de deux cartes communes (MKUE p. 3 ; `docs/reference/decks-de-depart.md`) : c'est ce que l'écran en montre, avec son illustration. Compétences :
  spec 051.
- **Figurine** (MKUE p. 3, *Player Area* : chaque joueur prend la figurine de son héros) : la figurine du plateau de jeu devient
  celle du héros de la partie, au lieu du pion unique de la spec 011.
- **Mise en place** (MKUE p. 1, *Game Flow*, 3) : inchangée, pilotée par le scénario (spec 012). La configuration de départ
  (scénario, héros, graine) est enregistrée dans `GameStarted`, donc dans la sauvegarde, comme aujourd'hui (spec 024).
- **Graine** : pas une règle du jeu. Laissée vide, elle est tirée de l'horloge et **jamais affichée** (spec 002 : elle révèle l'ordre
  du deck). Saisie, elle permet de rejouer une partie : même graine, même scénario et même héros, même mise en place et mêmes tirages
  (spec 038). Elle est écrite dans les logs dans les deux cas, pour reproduire un bogue (spec 003).

## Modèle

Termes du glossaire déjà présents : Scénario, Bac à sable, Héros, Configuration de départ (`GameConfig`), Deck.

Nouveaux termes, ajoutés au glossaire par cette PR (section « Nouvelle partie », hors livret) : **Graine** (`GameConfig::seed`),
**Cartes propres** d'un héros (`Hero::own_cards`), **Écran de nouvelle partie** (`NewGameScreen`, calculé sans egui, donc testable).

arcanum-core gagne, sans rien changer aux règles :

- la liste des scénarios jouables, dans l'ordre de l'écran : Première reconnaissance, bac à sable (`ScenarioId::ALL`), et le scénario
  d'un identifiant (`Scenario::of(ScenarioId)`) ;
- les cartes propres d'un héros, lues dans `content/heroes.ron` : pour chacune, la carte commune qu'elle remplace
  (`Hero::own_cards`). `Hero::ALL` existe déjà (Arythea, Goldyx, Norowas, Tovak).

**Illustrations des héros** (demande de Mathieu, 2026-10-30 ; Q4) : quatre SVG dessinés par nous, `assets/images/hero/<héros>.svg`
(`arythea`, `goldyx`, `norowas`, `tovak`), qui remplacent `hero/figure.svg`. Chacune est un **buste** du héros (tête et épaules, de
trois quarts ou de profil) dans un **médaillon** rond : fond et bordure épaisse de la couleur du héros, contour sombre, ombre portée.
La même image sert de figurine sur le plateau de jeu et de portrait dans le panneau ; la bordure de couleur garde la figurine
reconnaissable sur une case dézoomée, quand les traits du visage ne se lisent plus. Rien n'est décalqué des scans ni du livret
(CLAUDE.md, *Assets et droits*) : silhouettes simples, en aplats, dans l'esprit des autres SVG du jeu.

| Héros | Couleur (fond et bordure) | Buste (signes distinctifs) | Couleur de repli |
|---|---|---|---|
| Arythea | rouge | femme, capuche sombre, cheveux longs | `#C62828` |
| Goldyx | vert | tête de dragon, cornes et écailles | `#2E7D32` |
| Norowas | bleu argenté | elfe, oreilles pointues, cheveux clairs | `#5C7FB0` |
| Tovak | or | chevalier, heaume fermé | `#C9A227` |

Les couleurs évoquent celles des cartes de héros du jeu de base (`docs/regles/scans/characters/<héros>/character_<héros>.jpg` :
Tovak doré, Norowas bleu argenté), sans en reprendre les dessins, pour ne pas dérouter
les joueurs. La même image sert, à des tailles différentes, de figurine, de portrait du panneau et d'illustration de l'écran de
nouvelle partie. Une table unique d'arcanum-ui associe chaque héros à son image et à sa couleur de repli (comme les terrains, spec 011) ;
la couleur de repli s'affiche si l'image manque.

arcanum-ui gagne un **formulaire de nouvelle partie** (héros coché, scénario coché, texte de la graine) et l'écran calculé à partir de lui.
L'application a désormais deux états : **écran de nouvelle partie seul** (au lancement, aucune partie) ou **partie en cours**, avec
éventuellement la fenêtre de nouvelle partie ouverte par-dessus. `default_config` disparaît : la configuration vient du formulaire.

Pas d'action ni d'événement nouveau : l'écran construit une `GameConfig` et lance une partie neuve, comme `main.rs` aujourd'hui.

## Actions et événements

Aucun.

## Comportements attendus

Situation de départ des tests : le formulaire et `new_game_screen(form, contexte, catalog)` avec le `fr.json` du dépôt ; la session
(`GameSession` ou l'objet qui la porte) pour lancer, ouvrir, fermer ; un dossier de sauvegardes temporaire pour C13 à C18.

Écran :

- **C1** — Au lancement : héros Arythea, scénario Première reconnaissance, graine vide ; quatre héros dans l'ordre Arythea, Goldyx,
  Norowas, Tovak ; deux scénarios dans l'ordre Première reconnaissance, bac à sable (`new_game_screen_defaults`) — MKUE p. 1.
- **C2** — Chaque héros affiche ses deux cartes propres et la carte qu'elles remplacent, lues dans le contenu ; Arythea :
  « Polyvalence (à la place de Rage), Attraction de mana (à la place de Source de mana) » (`hero_shows_own_cards`) — MKUE p. 3.
- **C3** — Chaque scénario affiche une ligne de résumé (`scenario_shows_summary`).
- **C4** — Cocher Tovak puis le bac à sable : l'écran les montre cochés, les autres non (`choosing_hero_and_scenario`).
- **C4b** — Chaque héros de l'écran a l'illustration de son héros (`hero_choices_show_their_illustration`).

Illustrations :

- **C4c** — Chaque héros a une image `hero/<héros>.svg` qui existe et est un SVG ; les quatre sont différentes ; `hero/figure.svg`
  n'est plus listée (`every_hero_image_exists`) — spec 011, C21.
- **C4d** — Partie avec Norowas : la surimpression de la case de la figurine est `hero/norowas.svg`, avec la couleur de repli de
  Norowas (`figure_overlay_is_the_hero_image`) — MKUE p. 3.
- **C4e** — Partie avec Goldyx : le panneau latéral a, en tête des données du joueur (au-dessus de l'inventaire), le portrait
  `hero/goldyx.svg` et le nom « Goldyx » (`panel_shows_hero_portrait`).

Lancer une partie :

- **C5** — « Commencer » avec Goldyx, bac à sable, graine vide : la partie neuve a ce héros, ce scénario, et une graine tirée de
  l'horloge (injectable dans les tests) ; son deck de départ est celui de Goldyx (`start_uses_chosen_hero_and_scenario`) — MKUE p. 1, 3.
- **C6** — Graine « 42 » : la partie a la graine 42 ; deux parties lancées avec 42, le même héros et le même scénario ont les mêmes
  événements de mise en place (`typed_seed_replays_the_same_setup`).
- **C7** — Graine « 007 » ou «  42  » (espaces autour) : acceptée, graine 7 ou 42 (`seed_is_trimmed_and_parsed`).
- **C8** — Graine « abc », « -1 », « 4.2 » ou plus grande que 18446744073709551615 : « Commencer » grisé et la ligne
  `new_game.seed_invalid` sous le champ (`invalid_seed_disables_start`).
- **C9** — La graine, saisie ou tirée, n'apparaît dans aucun texte de l'écran de partie ni du panneau (`seed_is_never_displayed`) —
  spec 002.
- **C10** — Le panneau latéral montre en tête « Première reconnaissance · Goldyx » : scénario et héros de la partie
  (`panel_shows_scenario_and_hero`).

En cours de partie :

- **C11** — Le bouton « Nouvelle partie… » du panneau (ou Ctrl+N) ouvre la fenêtre, pré-remplie avec le héros et le scénario de la
  partie en cours, graine vide ; la partie en cours ne change pas (`new_game_window_prefills_current_choice`).
- **C12** — « Annuler » (ou Échap) ferme la fenêtre ; la partie continue là où elle était (`cancel_returns_to_game`).
- **C13** — « Commencer » remplace la partie en cours par la nouvelle ; la partie abandonnée, si elle n'est pas terminée,
  est d'abord sauvegardée comme par Ctrl+S, et le message `new_game.previous_saved` donne son nom
  (`abandoned_game_is_saved_first`) — spec 024.
- **C14** — Partie terminée (spec 063) : la fenêtre de fin a un bouton « Nouvelle partie… » qui ouvre la même fenêtre ; rien n'est
  sauvegardé en plus, la partie terminée a déjà sa sauvegarde automatique (`end_window_offers_new_game`).

Au lancement :

- **C15** — Une sauvegarde automatique existe et sa partie n'est pas terminée : l'écran propose « Continuer », avec le
  résumé de la fenêtre de chargement (scénario, héros, manche) ; « Continuer » la charge (`continue_loads_autosave`) — spec 024.
- **C16** — Pas de sauvegarde automatique, ou partie terminée, ou illisible : pas de « Continuer » (`no_continue_without_autosave`).
- **C17** — « Charger… » ouvre la fenêtre de chargement de la spec 024 ; charger une sauvegarde y lance la partie, l'écran de
  nouvelle partie disparaît (`load_from_new_game_screen`).
- **C18** — « Commencer » alors que la sauvegarde automatique contient une partie non terminée : elle est d'abord copiée
  en sauvegarde ordinaire (même nom que Ctrl+S), puisque le premier tour de la nouvelle partie va la remplacer
  (`autosave_is_kept_before_new_game`).
- **C19** — Chaque clé nouvelle existe en français et en anglais (`every_new_game_key_exists_in_french`).

## Cas limites et refus

- **L1** — Pas de dossier de sauvegardes (spec 024) : ni « Continuer » ni copie ; « Commencer » lance quand même la partie.
- **L2** — La sauvegarde de la partie abandonnée échoue (disque plein…) : la nouvelle partie ne démarre pas, message `save.failed`
  et la fenêtre reste ouverte ; le joueur peut annuler, ou recommencer (`failed_save_keeps_current_game`).
- **L3** — Au lancement, Échap ne fait rien : il n'y a pas de partie où revenir. « Annuler » n'est pas affiché.
- **L4** — Fenêtre ouverte pendant un mode choix (défausse, mana, fin du tour…) : impossible, Ctrl+N n'agit que hors mode choix et
  le bouton est grisé, comme Ctrl+S et Ctrl+O. Pendant un combat aussi, hors mode choix.
- **L5** — Fenêtre de nouvelle partie et fenêtre de chargement ne sont jamais ouvertes ensemble : « Charger… » ferme la première.
- **L6** — Une sauvegarde chargée garde son scénario, son héros et sa graine : l'écran ne les change pas.
- **L7** — Le texte de la graine est limité à 20 caractères (la plus grande graine en a 20).

## Information cachée

La graine tirée de l'horloge n'est jamais montrée (C9). Une graine saisie est connue du joueur, qui l'a choisie : elle ne s'affiche
pas pour autant une fois la partie lancée. Le résumé de « Continuer » est celui de la fenêtre de chargement, public.

## Libellés (i18n)

| Clé | Paramètres | fr | en |
|---|---|---|---|
| `new_game.title` | — | Nouvelle partie | New game |
| `new_game.button` | — | Nouvelle partie… | New game… |
| `new_game.hint` | — | Commencer une autre partie (Ctrl+N) | Start another game (Ctrl+N) |
| `new_game.hero` | — | Héros | Hero |
| `new_game.own_cards` | `cards` | Cartes propres : {cards} | Own cards: {cards} |
| `new_game.replaces` | `card`, `replaced` | {card} (à la place de {replaced}) | {card} (instead of {replaced}) |
| `new_game.scenario` | — | Scénario | Scenario |
| `scenario.first_reconnaissance.summary` | — | 4 manches. Chaque tuile révélée rapporte 1 gloire ; un dernier tour après la révélation d'une cité, puis le score final. | 4 rounds. Each revealed tile gives 1 Fame; one last turn after a city is revealed, then the final score. |
| `scenario.sandbox.summary` | — | Entraînement : 4 manches, sans gloire pour les tuiles ni score final. | Training: 4 rounds, no Fame for tiles and no final score. |
| `new_game.seed` | — | Graine (facultatif) | Seed (optional) |
| `new_game.seed_hint` | — | Vide : une partie au hasard. La même graine avec le même héros et le même scénario rejoue la même partie. | Empty: a random game. The same seed with the same hero and scenario replays the same game. |
| `new_game.seed_invalid` | — | La graine est un nombre entier de 0 à 18446744073709551615. | The seed is a whole number from 0 to 18446744073709551615. |
| `new_game.start` | — | Commencer | Start |
| `new_game.cancel` | — | Annuler | Cancel |
| `new_game.continue` | `summary` | Continuer : {summary} | Continue: {summary} |
| `new_game.load` | — | Charger… | Load… |
| `new_game.abandon_warning` | — | La partie en cours sera sauvegardée, puis remplacée. | The current game will be saved, then replaced. |
| `new_game.previous_saved` | `name` | Partie précédente sauvegardée : {name} | Previous game saved: {name} |
| `panel.scenario_and_hero` | `scenario`, `hero` | {scenario} · {hero} | {scenario} · {hero} |

Le résumé de « Continuer » reprend `load.entry` sans la date. Les noms des cartes et des héros existent déjà (`card.*.name`,
`hero.*.name`).

## Interface

- **Au lancement** : la fenêtre « Nouvelle partie » centrée sur un fond uni, sans plateau ni panneau latéral.
- **En cours de partie** : la même fenêtre, par-dessus le plateau ; le panneau latéral gagne « Nouvelle partie… » à côté de
  « Sauvegarder » et « Charger… », et la fenêtre de fin (spec 063) le même bouton à côté de « Voir le plateau ».
- **Contenu** : les héros en boutons radio, chacun suivi de sa ligne de cartes propres en petit ; les scénarios en boutons radio,
  chacun suivi de son résumé en petit ; le champ de la graine, son aide en petit et, si elle est invalide, `new_game.seed_invalid` en
  rouge ; en cours de partie non terminée, `new_game.abandon_warning` ; en bas « Commencer », puis « Annuler » (en cours de partie) ou
  « Continuer » et « Charger… » (au lancement).
- **Panneau latéral** : sa ligne de tête devient « {scénario} · {héros} » (spec 063 n'y mettait que le scénario). Les données du
  joueur (inventaire, points, gloire, niveau) commencent par le **portrait** du héros (environ 48 points de côté) suivi de son nom,
  sur une ligne.
- **Plateau de jeu** : la figurine est l'illustration du héros, à la place et à la taille du pion actuel (spec 011, sous le site de
  la case). L'infobulle de survol de la case de la figurine reste « Votre héros ».
- **Écran de nouvelle partie** : chaque bouton radio de héros est précédé de son illustration (environ 32 points).

```
┌─ Nouvelle partie ──────────────────────────────────────────────────┐
│ Héros                                                              │
│ (•) [◉] Arythea   Cartes propres : Polyvalence (à la place de Rage),   │
│                   Attraction de mana (à la place de Source de mana)│
│ ( ) Goldyx    Cartes propres : Pouvoir de concentration (…), …     │
│ ( ) Norowas   …                                                    │
│ ( ) Tovak     …                                                    │
│                                                                    │
│ Scénario                                                           │
│ (•) Première reconnaissance   4 manches. Chaque tuile révélée …    │
│ ( ) Bac à sable               Entraînement : 4 manches, …          │
│                                                                    │
│ Graine (facultatif)  [            ]                                │
│ Vide : une partie au hasard. La même graine …                      │
│                                                                    │
│ [Commencer]   [Continuer : Première reconnaissance · Arythea · …]  │
│               [Charger…]                                           │
└────────────────────────────────────────────────────────────────────┘
```

Les bustes sont un essai (Mathieu, « essayons ») : la PR d'implémentation joint une capture de chaque buste au zoom le plus faible
et dans le panneau ; s'ils ne se lisent pas sur le plateau, Mathieu tranche avant la fusion (dessin retouché ou autre forme).

Vérification à l'œil : lancer le jeu, voir l'écran et les quatre illustrations, choisir Tovak et le bac à sable, commencer (deck de
Tovak dans la main, panneau « Bac à sable · Tovak » et son portrait, sa figurine sur le plateau, au zoom le plus fort et le plus faible) ; Ctrl+N, Échap ; Ctrl+N, Commencer (message de sauvegarde) ; relancer le jeu, voir « Continuer » ; saisir
« abc » dans la graine. Depuis la partie préparée de fin de partie de la spec 063 (`cite-en-vue`), aller à la fenêtre de fin et
cliquer « Nouvelle partie… ».

### Raccourcis clavier

Reportés dans `docs/raccourcis.md` :

- **Ctrl+N** : ouvrir la fenêtre de nouvelle partie. Partie, hors mode choix (comme Ctrl+S et Ctrl+O), y compris partie terminée.
- **Entrée** : commencer, quand la fenêtre de nouvelle partie est ouverte et la graine valide, y compris depuis le champ de la graine.
- **Échap** : fermer la fenêtre de nouvelle partie, quand une partie est en cours (sans effet au lancement, L3).
- Pas de touche pour cocher un héros ou un scénario : quatre et deux boutons radio suffisent à la souris, et Tab parcourt la fenêtre.

## Effets sur les specs déjà implémentées

- **002** : la graine n'est plus tirée dans `main.rs` mais au clic sur « Commencer » (toujours dans arcanum-ui) ; elle reste invisible.
- **012** et **063** : `default_config` disparaît ; les tests qui l'utilisent prennent une configuration explicite (une fonction de
  test, Première reconnaissance et Arythea, pour ne pas changer leurs graines). La ligne de tête du panneau gagne le héros.
- **024** : la fenêtre de chargement s'ouvre aussi depuis l'écran de nouvelle partie ; la copie de la sauvegarde automatique (C18)
  passe par `save`, sans format nouveau.
- **011** : `hero/figure.svg` et la couleur `#FFB300` de la figurine laissent la place aux quatre illustrations et à leurs
  couleurs de repli ; le test des images (C21) en compte 35 au lieu de 32.
- **063** : la fenêtre de fin gagne « Nouvelle partie… » (la spec 063 le renvoyait ici).

## Hors périmètre

- Variantes (MKUE p. 16-17) : hors périmètre (PLAN.md). Un scénario de plus (spec 064) n'ajoute qu'une ligne à `ScenarioId::ALL`
  et son résumé.
- Carte de héros, jetons de niveau ou de compétence illustrés : seule l'illustration du héros est dessinée.
- Compétences des héros (spec 051), joueur fantôme (spec 061) et son héros : rien à choisir aujourd'hui.
- Se souvenir des choix d'un lancement à l'autre (fichier de préférences) : pas demandé ; dans une même session, la fenêtre reprend
  la partie en cours (C11).
- Afficher la graine d'une partie au hasard pour la rejouer : écarté (Q3).

## Précisions de l'implémentation

Deux points que la spec ne tranchait pas, à confirmer par Mathieu à la revue :

- **L1 en cours de partie** : sans dossier de sauvegardes, « Commencer » ne tente aucune sauvegarde de la partie abandonnée et
  lance la nouvelle partie (sinon L2 bloquerait toute nouvelle partie). L2 ne vaut que pour une écriture qui échoue.
- **Ctrl+O au lancement** : fait comme « Charger… » (le bouton porte cette touche depuis la spec 024) ; `scripts/verif-manuelle.sh
  charger <nom>` s'en sert toujours. Un chargement qui échoue au lancement laisse l'écran avec `load.failed`.

## Questions ouvertes

Aucune. Décisions (Mathieu, 2026-10-30) :

- **Q1 — Au lancement** : toujours l'écran de nouvelle partie, avec « Continuer » quand la sauvegarde automatique contient une partie
  non terminée (C15, C16) ; relancer le jeu n'écrase plus cette sauvegarde sans y penser.
- **Q2 — Partie abandonnée** : « Commencer » sauvegarde d'abord, comme Ctrl+S, la partie non terminée qu'il ferait perdre (C13 en
  cours de partie, C18 au lancement pour la sauvegarde automatique).
- **Q3 — Graine d'une partie au hasard** : jamais affichée (spec 002) ; pour rejouer une partie, on la sauvegarde ou on saisit une
  graine dès le départ.
- **Q4 — Illustrations des héros** : un buste par héros, commun à la figurine et au portrait du panneau (Modèle, *Illustrations des
  héros*), à essayer (captures jointes à la PR d'implémentation).
