# Plan de développement — spec 080 Préférences

- Statut : brouillon, suivi par le fil d'implémentation (2026-11-20) ; à relire par Mathieu avec la PR.
- Spec : [`080-preferences.md`](../080-preferences.md). Branche : `spec/080-preferences`.
- Méthode : celle de `CLAUDE.md` (tests rouges, code minimal, refactor ; `scripts/verifier.sh` vert avant chaque push).

## Découpage en PR

Une seule PR, privée 351 (celle de la rédaction).

## Découpage en fils

Un seul fil : 6 étapes, rien dans arcanum-core (la spec : « Rien dans arcanum-core »), ni vue de partie nouvelle.

**Contrôle des champs** (issue privée 130) : la fenêtre lit `PreferencesWindow`, calculée par `ClientSession` (étape 3) à partir des
préférences (étape 1) et des langues disponibles (étape 2) ; aucun champ de `PlayerView`.

## Étapes

| # | Étape | Cas | Rouge attendu | Fichiers | Isolée ? | Modèle |
|---|---|---|---|---|---|---|
| 1 | Module `preferences` d'arcanum-persistance : type, lecture tolérante, écriture sûre | C1 à C5 | `load` et `save` en `todo!()` remplacés par un squelette qui rend les défauts et n'écrit rien : les relectures et la présence du fichier échouent | `arcanum-persistance/src/preferences.rs` (neuf), `lib.rs`, `Cargo.toml`, `tests/preferences.rs` | non | fil principal |
| 2 | Dossier de configuration, préférences lues au lancement, langue des préférences dans `launch_catalog`, plein écran de `viewport` | C6 à C12, L5 | `choose_config_dir` rend `None`, la langue des préférences est ignorée, `viewport` ne pose pas le plein écran | `arcanum-ui/src/preferences.rs` (neuf), `language.rs`, `icon.rs`, `main.rs`, tests `choix_de_la_langue.rs`, `preferences.rs` (neuf) | non | fil principal |
| 3 | Fenêtre des préférences dans la session : commandes, `PreferencesWindow`, élément du Menu, Ctrl+,, Échap, demande `WindowRequest::Language`, message d'échec, bouton du pied du lancement | C13 à C23, L1 à L4 | fenêtre jamais ouverte (`None`), Menu sans l'élément, `KEY_BINDINGS` sans Ctrl+, | `client.rs`, `menu_bar.rs`, `keys.rs`, `session.rs`, `new_game.rs`, `screen/types.rs`, tests `barre_de_menu.rs`, `preferences.rs` | non | fil principal |
| 4 | Libellés `fr.json` et `en.json` (tableau de la spec) | — | — (les tests de l'étape 3 lisent ces textes : posés avec elle) | `assets/i18n/*.json` | oui | fil principal |
| 5 | Écran : fenêtre egui, bouton du pied, catalogue rechargé et titre de la fenêtre sur `WindowRequest::Language` | — (vérification à l'œil) | — | `app/*.rs` | non | fil principal |
| 6 | `verif-manuelle.sh` (`ARCANUM_CONFIG_DIR`), vérification à l'œil sous Xvfb ; docs (`architecture.md`, `raccourcis.md`, glossaire, README, `verification-manuelle.md`, `PLAN.md`, `index-par-notion.md`, `.gitignore`) | — | — | `scripts/`, `docs/` | oui | fil principal |

## Risques repérés

- `Command` est `Copy` (boutons copiés partout) et `Language` contient une `String` : `Command::SetLanguage` porte donc la
  place de la langue dans la liste disponible (`Option<usize>`, `None` = langue du système) au lieu de la `Language` elle-même.
  La spec est mise à jour dans la même PR, et la PR le signale.
- `WindowRequest` est `Copy` aussi : il perd `Copy` pour porter une `Language` (seule l'application le consomme).
