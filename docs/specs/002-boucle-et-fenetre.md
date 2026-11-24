# 002 — Boucle de partie et fenêtre minimale

- Statut : implémentée (2026-10-24)
- Jalon : J0
- Dépend de : 000, 001
- Livret : aucune règle de jeu (traduit l'ADR 0002 côté UI, et l'ADR 0003 pour l'affichage)
- Crates touchées : arcanum-ui (création de la crate), arcanum-core (clé de libellé des actions), `assets/i18n/` (clés de l'UI)

## Objectif

Lancer `cargo run -p arcanum-ui` ouvre une fenêtre qui affiche la partie du jeu jouet de la spec 000 : la main, le nombre de
cartes du deck et un bouton « Piocher ». Cliquer pioche une carte ; quand le deck est vide, la fenêtre annonce la fin de
partie. Le joueur humain passe par le trait `Player` comme n'importe quel joueur, et la boucle de partie ne bloque jamais
la fenêtre.

## Règles

Pas de règle du livret. Comportement fixé par l'ADR 0002, l'ADR 0003 et `.claude/rules/arcanum-ui.md` :

- l'UI n'affiche que la **vue joueur** du joueur humain, jamais l'état complet ;
- l'UI ne propose que les **actions légales** données par le moteur, et ne contient aucune règle ;
- aucun texte affiché en dur : tout passe par une clé de `fr.json` ;
- la boucle egui n'attend jamais un joueur.

## Modèle

Nouveaux termes à ajouter au glossaire (vocabulaire technique, ajoutés dans la même PR que cette spec) :

| Français | Code | Sens |
|---|---|---|
| Joueur humain | `HumanPlayer` | Implémentation de `Player` alimentée par les clics de l'UI. |
| Écran de partie | `GameScreen` | Ce que la fenêtre doit afficher, déjà traduit : textes et boutons. Calculé sans egui, donc testable. |
| Dossier des ressources | `AssetsDir` | Dossier `assets` lu au lancement (libellés aujourd'hui, autres fichiers du jeu plus tard). |

### Organisation d'arcanum-ui

La crate a une partie bibliothèque, testable sans fenêtre, et un `main.rs` réduit au lancement d'eframe.

- **Joueur humain.** Il est découpé en deux moitiés qui partagent une même file d'attente (même thread, pas de verrou) :
  la moitié `Player`, confiée au `GameRunner`, et une poignée gardée par l'UI.
  - `request(view, legal_actions)` mémorise la demande ; la poignée sait alors qu'une décision attend l'humain et laquelle.
  - La poignée permet de soumettre un choix (un index dans les actions légales de la demande en cours).
  - `poll()` rend ce choix une seule fois, puis la demande est close. Sans choix soumis, `poll()` rend `None`.
- **Boucle dans l'UI.** À chaque image, avant de dessiner, l'application appelle `runner.step()` tant qu'il rend
  `Advanced`, dans une limite de 100 pas par image ; si la limite est atteinte, elle redemande une image à egui pour
  continuer sans geler la fenêtre. Elle s'arrête dès que `step()` rend `Waiting` ou `Finished`. Pas de thread : le
  `GameRunner` vit dans l'application egui.
- **Écran de partie.** Une fonction pure calcule le `GameScreen` à partir de la vue joueur, de la demande en cours
  du joueur humain (s'il y en a une) et du catalogue. egui ne fait que dessiner ce `GameScreen` et renvoyer l'index du
  bouton cliqué à la poignée.
- **Configuration de départ.** Le deck du jeu jouet contient les cartes 1 à 10. La graine est tirée de l'horloge au
  lancement (c'est de l'I/O, donc dans arcanum-ui, jamais dans arcanum-core). Elle n'est pas affichée : elle révèle l'ordre du deck.

### Clé de libellé d'une action (arcanum-core)

`Action` donne sa clé de libellé, une chaîne fixe : `DrawCard` → `action.draw_card`. C'est ce que prévoyait la spec 000
(« les actions seront identifiées par une clé quand l'UI en aura besoin »), et ADR 0003 : arcanum-core manipule des clés,
jamais des textes. Chaque nouvelle action devra fournir la sienne.

### Langue et fichiers de langue

La langue est le français, sans choix possible pour l'instant. Les fichiers de langue ne sont **pas** intégrés à
l'exécutable : ils sont lus au lancement, comme le seront plus tard les autres fichiers du jeu (décision de Mathieu, Q1).

Le **dossier des ressources** est le dossier `assets` qui contient `i18n/`. Au lancement, arcanum-ui essaie dans cet ordre
les emplacements suivants, et garde le premier qui contient `i18n/fr.json` :

1. le dossier donné par la variable d'environnement `ARCANUM_ASSETS_DIR`, si elle est définie (tests, installations
   particulières) ;
2. le dossier `assets` à côté de l'exécutable (jeu distribué) ;
3. le dossier `assets` à la racine du dépôt, dont le chemin est connu à la compilation par `CARGO_MANIFEST_DIR`
   (développement, `cargo run -p arcanum-ui` depuis n'importe quel dossier).

Tous les chemins sont construits avec `PathBuf` (ADR 0004). Ensuite `Catalog::load(<ressources>/i18n, fr)` charge le
français. La recherche est une fonction qui reçoit la liste des emplacements candidats, pour être testée sans dépendre
de l'endroit où tourne le test.

## Actions et événements

Aucun nouveau : on réutilise `DrawCard`, `CardDrawn` et `DeckShuffled` de la spec 000.

## Comportements attendus

Tests de la bibliothèqu'arcanum-ui, sans fenêtre.

- **C1** — `DrawCard` a la clé `action.draw_card` (`draw_card_has_a_label_key`, dans arcanum-core).
- **C2** — Étant donné un joueur humain sans demande, `poll()` rend `None` et la poignée indique qu'aucune décision n'attend (`human_without_request_has_nothing_pending`).
- **C3** — Après `request`, la poignée expose la demande (vue et actions légales) ; tant que rien n'est soumis, `poll()` rend `None` à chaque appel (`human_waits_until_a_choice_is_submitted`).
- **C4** — Après soumission de l'index 0, `poll()` rend `Some(0)` une seule fois, puis `None` ; la poignée n'indique plus de décision en attente (`submitted_choice_is_returned_once`).
- **C5** — Un `GameRunner` avec le joueur humain : `step()` rend `Waiting` sans rien journaliser tant que l'UI n'a rien soumis ; après soumission, `step()` rend `Advanced` et une carte est en main (`runner_with_human_waits_then_advances`).
- **C6** — La boucle d'une image (`step()` répété jusqu'à `Waiting` ou `Finished`, au plus 100 pas) s'arrête dès que l'humain doit décider, et ne fait jamais plus de 100 pas même avec un joueur automatique qui répond toujours (`frame_loop_stops_on_waiting_and_is_bounded`).
- **C7** — L'écran d'une partie qui commence montre : le titre, la main vide (libellé `hand.empty`), le nombre de cartes du deck (« Deck : 10 cartes ») et un seul bouton, « Piocher », d'index 0 (`screen_at_start_shows_empty_hand_and_draw_button`).
- **C8** — Après deux pioches, l'écran liste les deux cartes de la main dans l'ordre de pioche, chacune nommée par `toy.card.name` avec son numéro, et le deck indique 8 cartes (`screen_lists_hand_in_draw_order`).
- **C9** — Quand le deck est vide, l'écran n'a plus aucun bouton et affiche le libellé `game.finished` (`screen_at_end_has_no_button_and_announces_end`).
- **C10** — Les boutons de l'écran sont exactement les actions légales de la demande en cours, dans le même ordre, et l'index d'un bouton est celui de son action (`screen_buttons_match_legal_actions`).
- **C11** — Deux états qui ne diffèrent que par l'ordre du deck donnent le même écran : l'écran est calculé depuis la vue, pas depuis l'état (`screen_does_not_reveal_deck_order`).
- **C12** — En calculant les écrans d'une partie complète (début, milieu, fin) avec le vrai `fr.json`, le catalogue ne relève aucun manque : toute clé utilisée par l'UI existe dans `fr.json` (`every_ui_key_exists_in_french`).
- **C13** — Étant donné trois emplacements candidats dont seuls le deuxième et le troisième contiennent `i18n/fr.json` (dossiers de test dans `crates/arcanum-ui/tests/fixtures/`), la recherche retient le deuxième (`assets_dir_is_the_first_candidate_with_french`).
- **C14** — Les emplacements candidats sont, dans l'ordre : `ARCANUM_ASSETS_DIR` s'il est défini, `assets` à côté de l'exécutable, `assets` du dépôt ; sans `ARCANUM_ASSETS_DIR`, la liste n'a que les deux derniers (`assets_candidates_are_in_priority_order`).
- **C15** — Le dossier `assets` du dépôt, trouvé par le troisième emplacement, contient un `fr.json` qui se charge sans erreur (`repository_assets_dir_is_found`). La cohérence entre `fr.json` et `en.json` reste garantie par les tests C11 à C13 de la spec 001.

Vérification manuelle (pas de test automatique, une fenêtre ne s'ouvre pas en CI) : `cargo run -p arcanum-ui` sous Windows
et sous Linux ouvre la fenêtre, piocher dix fois vide le deck et affiche la fin de partie, la fenêtre reste réactive
(redimensionnement, survol) pendant toute la partie.

## Cas limites et refus

- **L1** — Soumettre un choix alors qu'aucune demande n'est en cours : ignoré, `poll()` continue de rendre `None` (`choice_without_request_is_ignored`).
- **L2** — Soumettre deux choix pour la même demande (double clic dans la même image) : seul le premier compte, le second est ignoré (`second_choice_for_same_request_is_ignored`).
- **L3** — Soumettre un index hors des actions légales de la demande : ignoré, la demande reste ouverte ; l'UI ne peut pas en produire, mais la poignée ne fait pas confiance à l'appelant (`out_of_range_choice_is_ignored`).
- **L4** — Si `step()` renvoie une erreur (cas qui ne devrait pas arriver), l'application ne plante pas : elle arrête de faire avancer la partie et affiche le libellé `error.game_stopped` à la place des boutons (`runner_error_stops_the_game_without_panic`).
- **L5** — Aucun emplacement candidat ne contient `i18n/fr.json` : la recherche renvoie une erreur typée qui liste les emplacements essayés ; le jeu ne plante pas, il démarre avec un catalogue vide et les clés s'affichent `⟦clé⟧` (`missing_assets_dir_falls_back_to_visible_keys`).
- **L6** — `fr.json` est trouvé mais ne se charge pas (JSON invalide, par exemple) : même repli, catalogue vide et clés visibles, sans panique (`unloadable_catalog_falls_back_to_visible_keys`).
- **L7** — `ARCANUM_ASSETS_DIR` est défini mais ne contient pas `i18n/fr.json` : on passe à l'emplacement suivant, sans erreur (`invalid_assets_env_var_is_skipped`).

## Information cachée

L'écran ne contient que ce que contient la vue du joueur humain : sa main, le nombre de cartes du deck, et la demande en
cours. Jamais l'ordre du deck ni la graine. C11 le garantit.

## Libellés (i18n)

Ajoutés dans `fr.json` et `en.json` :

| Clé | Paramètres | Français | English |
|---|---|---|---|
| `app.title` | — | Iter Arcanum (prototype) | Iter Arcanum (prototype) |
| `hand.title` | — | Main | Hand |
| `hand.empty` | — | Aucune carte en main | No card in hand |
| `deck.count` | `count` | Deck : {count} carte(s) | Deck: {count} card(s) |
| `toy.card.name` | `number` | Carte {number} | Card {number} |
| `action.draw_card` | — | Piocher | Draw |
| `game.finished` | — | Partie terminée | Game over |
| `error.game_stopped` | — | La partie s'est arrêtée sur une erreur. | The game stopped on an error. |

Les clés `toy.*` disparaîtront avec le jeu jouet.

## Interface

Une seule fenêtre, redimensionnable, titre `app.title`, taille initiale 800 × 600. Un panneau central avec, de haut en bas :
le titre de la main et la liste des cartes (ou `hand.empty`), le nombre de cartes du deck, puis une rangée de boutons
(un par action légale) ou le message de fin de partie. Thème egui par défaut, aucun visuel.

## Dépendances

À valider (CLAUDE.md : toute nouvelle dépendance est proposée et justifiée) :

- **`eframe`** (et donc `egui`), dans la dernière version stable au moment du code, renderer `glow` (OpenGL, le plus
  léger, compatible Windows et Linux), sans la fonctionnalité `persistence` (la sauvegarde relève d'arcanum-persistance).
  C'est la stack choisie dans CLAUDE.md. Aucune autre dépendance.

## Hors périmètre

- Choix de la langue dans l'UI : plus tard, avec un écran d'options (spec à créer).
- Écrire les manques de libellés et les erreurs dans un fichier de logs : spec 003, qui branchera arcanum-ui sur sa façade.
- Nouvelle partie, choix de la graine, du scénario, du héros : spec 065.
- Sauvegarde, annulation : spec 024.
- Joueurs automatiques en partie : leurs specs, plus tard. La boucle ici les accepte déjà (C6).
- Tests automatisés du rendu egui (capture, clics simulés) : pas pour l'instant (Q2), suivi dans l'issue #2.
- Copier `assets/` à côté de l'exécutable pour distribuer le jeu : spec de distribution, plus tard. L'emplacement 2 est déjà prévu pour elle.
- Les manques de libellés et l'échec de la recherche du dossier des ressources ne sont pas encore écrits dans un fichier : spec 003.

## Décisions prises à la relecture

1. **Q1** — Les fichiers de langue ne sont pas intégrés à l'exécutable : ils sont lus au lancement, car le jeu aura de
   toute façon beaucoup de fichiers à charger (Mathieu, 2026-10-24). D'où la recherche du dossier des ressources (C13
   à C15, L5 à L7).
2. **Q2** — Pas d'`egui_kittest` pour le moment ; Mathieu peut changer d'avis plus tard. Le point est suivi dans
   l'issue #2.
3. **Q3** — Graine tirée de l'horloge à chaque lancement : accepté.
4. **Q4** — Mathieu peut lancer le jeu sur un PC Linux Mint, où l'antivirus ne bloque rien : la vérification manuelle
   s'y fera, en plus de Windows quand c'est possible.
5. **Q5** — La variable d'environnement `ARCANUM_ASSETS_DIR` (emplacement 1) est gardée : elle permet de lancer le jeu avec
   un autre dossier de ressources sans le déplacer (Mathieu, 2026-10-24).

## Questions ouvertes

Aucune.
