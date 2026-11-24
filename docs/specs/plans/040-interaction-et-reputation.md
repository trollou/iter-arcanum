# Plan de développement — spec 040 Interaction et réputation

- Statut : accepté avec la spec (2026-10-31).
- Spec : [`040-interaction-et-reputation.md`](../040-interaction-et-reputation.md). Branche : `spec/040-interaction-et-reputation`.
- Méthode : celle de `CLAUDE.md` (tests rouges, code minimal, refactor ; `scripts/verifier.sh` vert avant chaque push), avec les
  règles du rouge visible de l'issue privée 106 (gabarit `_gabarit.md`) : squelette `todo!()` d'abord, commit local
  `Tests rouges : …` qui nomme les tests en échec et leur nombre, section « Rouge vu » dans la PR.

## Découpage en PR

Une seule PR (celle qui porte la spec) : la réputation sans interaction ni écran ne se voit pas, et l'interaction a besoin de la
réputation pour son bonus.

## Étapes

Chaque étape qui code un comportement = un commit `Tests rouges : <étape>` (local) puis un commit de code (vert). Sans
comportement (libellés, docs) : pas de rouge (« — »).

| # | Étape | Cas | Rouge attendu | Fichiers | Isolée ? | Modèle |
|---|---|---|---|---|---|---|
| 1 | **Piste de réputation** : contenu (`content/reputation.ron` ou dans `levels.ron`), `ReputationTrack` (`bonus`, X), réputation dans l'état et la vue, `ReputationChanged` borné | C1 à C3, C25 | C2 : la table rendue par le squelette (`todo!()` ou table vide) diffère de celle du scan ; C3 : la réputation dépasse +7 | nouveau `reputation.rs`, `content.rs`, `state.rs`, `view.rs`, `event.rs`, `engine.rs`, nouveau `tests/interaction.rs` | non | fil principal |
| 2 | **Ennemis errants vaincus** : `ReputationChanged` après `EnemiesDefeated`, type imprimé sur la case | C4 à C7 | C4 : pas de `ReputationChanged` après la victoire sur les orques | `combat.rs`, `engine.rs`, `tests/interaction.rs` (arène) | non | fil principal |
| 3 | **Interagir** : `Action::Interact`, `InteractionStarted`, points perdus, bonus, influence signée, action du tour (plus de déplacement, défi, annonce, repos), cartes proposées en interaction ; `FirstChoice` ne choisit pas `Interact` | C8 à C19, L1 à L3, L7 | C8 : `Interact` absente des actions légales sur le village ; C9 : influence 2 au lieu de 1 après `Interact` | `action.rs`, `engine.rs`, `combat.rs` (`Moment`), `state.rs`, `tests/premier_choix/` | non | fil principal |
| 4 | **Menace et Nobles manières** : deux formes d'`Effect` et leur forme écrite, contenu de `basic_actions.ron`, événements produits au `CardPlayed` | C20 à C23, L4 à L6 | C20 : Menace renforcée ne produit pas `ReputationChanged { change: −1 }` ; C21 : pas de `FameGained` en interaction | `card.rs`, `content.rs`, `content/basic_actions.ron`, `engine.rs` | non | fil principal |
| 5 | **Sauvegarde** : partie en pleine interaction | C24 | C24 : l'interaction en cours perdue au rechargement (si elle passe du premier coup, le signaler dans la PR) | `arcanum-persistance/tests/` | non | fil principal |
| 6 | **Libellés** `fr.json` et `en.json` (tableau de la spec) | — | — | `assets/i18n/*.json` | oui | Haiku (brief : le tableau de la spec) |
| 7 | **Écran** : ligne de réputation, bouton et avertissement, en-tête d'interaction, influence signée, textes des effets, historique du combat, touche I | C26 à C30 | C26 : `game_screen` sans ligne de réputation (champ du squelette vide) | `arcanum-ui/src/screen/`, `session.rs`, `app/side_panel.rs`, `app/mod.rs` | non | fil principal |
| 8 | **Partie préparée** `interaction` (Norowas au village) et vérification à l'œil sous Xvfb | — | — | `arcanum-persistance/tests/parties_preparees.rs`, `scripts/parties/` | non | fil principal |
| 9 | **Docs** : architecture, PLAN (implémentée), index par notion, raccourcis, vérification manuelle, tableaux des cartes des specs 013 et 022 | — | — | docs | oui | fil principal (court) |
| 10 | **Bilan de fin de spec** | — | — | — | non | fil principal |

## Choix du modèle par étape

Seuls les libellés (étape 6) sont délégués : une douzaine de clés, recopiées du tableau de la spec. Le reste suit les mêmes
fichiers du moteur et de l'écran.

## Risques repérés

- **Index des actions** : `Interact` s'insère entre les explorations et les défis ; les tests et joueurs qui choisissent par index
  décalent. Les parcours choisissent par valeur ; `FirstChoice` l'ignore (sinon il interagirait au village au lieu de finir son
  tour, et les parties de `boucle_et_fenetre.rs` changeraient).
- **Influence signée** : `influence_points` passe de `u32` à un entier signé ; l'écran et les tests qui le lisent sont à adapter.
- **Menace renforcée** : des tests de la spec 022 la jouent comme `Unsupported` (de côté seulement) ; ils changent de sens, à
  signaler dans la PR.
- **Parties préparées** : leur construction ne change pas, mais tout fichier de `scripts/parties/` est vérifié ; à régénérer si
  la mise en place gagne un événement.
- **CI** : quota GitHub Actions possiblement épuisé ; `scripts/verifier.sh` en local fait foi, fusion sans CI seulement avec
  l'accord de Mathieu.
