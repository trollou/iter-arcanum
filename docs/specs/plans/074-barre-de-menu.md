# Plan de développement — spec 074 Barre de menu

- Statut : brouillon, suivi par le fil d'implémentation (2026-11-11) ; à relire par Mathieu avec la PR.
- Spec : [`074-barre-de-menu.md`](../074-barre-de-menu.md). Branche : `spec/074-barre-de-menu`.
- Méthode : celle de `CLAUDE.md` (tests rouges, code minimal, refactor ; `scripts/verifier.sh` vert avant chaque push).

## Découpage en PR

Une seule PR, privée 183 (celle de la rédaction).

## Découpage en fils

Un seul fil : 6 étapes, rien dans arcanum-core (la spec : « tout ce que la barre montre est déjà dans la vue »).

**Contrôle des champs de la vue** (issue privée 130) : la barre lit `PlayerView::scenario`, `time_of_day`, `round`, `turn`,
`tile_deck` et `removed_tiles`, tous présents ; Quitter lit `GameSession::autosaved_len` (arcanum-ui), présent.

## Étapes

| # | Étape | Cas | Rouge attendu | Fichiers | Isolée ? | Modèle |
|---|---|---|---|---|---|---|
| 1 | Libellés `fr.json` et `en.json` (tableau de la spec ; clés retirées à l'étape 3, quand plus rien ne les lit) | — | — | `assets/i18n/*.json` | oui | fil principal |
| 2 | Icônes `Icon::TileCountryside`, `Icon::TileCore` et leurs SVG | C8 | `every_icon_has_a_name` : 24 icônes au lieu de 26 ; `every_icon_image_exists` : SVG absents | `icons.rs`, `assets/images/ui/` | non | fil principal |
| 3 | `MenuBar` : Menu, scénario, manche, pioches, message ; champs de `GameScreen` retirés, tests existants migrés | C1 à C7, C9, C10, L1, L2 | `todo!()` évité : `menu_bar` rend une barre vide, les assertions sur les éléments et les textes échouent | `menu_bar.rs` (neuf), `screen/mod.rs`, `session.rs`, `client.rs`, tests | non | fil principal |
| 4 | Demandes à la fenêtre et Quitter : file `WindowRequest`, plein écran, confirmation, croix de la fenêtre, touches Ctrl+Q et F11 | C11 à C18, L3 à L5 | file vide, pas de confirmation, `KEY_BINDINGS` sans Ctrl+Q ni F11 | `client.rs`, `session.rs`, `keys.rs`, `screen/types.rs` | non | fil principal |
| 5 | Écran : barre en haut, Menu déroulant, confirmation de Quitter, panneau de droite sans en-tête, demandes transmises à la fenêtre | — (vérification à l'œil, L7) | — | `app/*.rs` | non | fil principal |
| 6 | Vérification à l'œil sous Xvfb ; docs (`architecture.md`, `raccourcis.md`, glossaire, `PLAN.md`, `index-par-notion.md`) | — | — | `docs/` | oui | fil principal |

## Risques repérés

- Plus de tests existants lisent `GameScreen::time_of_day` et `scenario` que la spec n'en cite (`deplacement.rs`,
  `fin_du_tour.rs`, `fin_de_manche.rs`, `nouvelle_partie.rs`, `support/mod.rs`) : migrés vers `MenuBar` à l'étape 3, avec le
  même texte attendu au nouveau format, et signalés dans la PR.
- La croix de la fenêtre : `ViewportCommand::Close` envoyé après « Quitter » repasse par la demande de fermeture du système ;
  la session retient qu'elle ferme, pour ne pas redemander.
