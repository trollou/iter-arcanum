# 080 — Préférences

- Statut : implémentée (2026-11-20, plan [`plans/080-preferences.md`](plans/080-preferences.md)) ; validée le 2026-11-20
  (Q1 à Q5 tranchées par Mathieu : réponses proposées retenues, Q3 précisée)
- Jalon : Finitions
- Dépend de : 074 (barre de menu, Plein écran, `WindowRequest`), 076 (choix de la langue, `--lang`), 078 (écran de
  nouvelle partie, pied du lancement), 024 (ordre des dossiers des sauvegardes) ; toutes implémentées
- Livret : aucune règle de jeu
- Crates touchées : arcanum-persistance (fichier des préférences), arcanum-ui (fenêtre, Menu, lancement), `assets/i18n/`. Rien dans
  arcanum-core. Aussi : `scripts/verif-manuelle.sh`, `docs/verification-manuelle.md`, `docs/raccourcis.md`,
  `docs/architecture.md`, glossaire, README, `.gitignore`
- Base : issue privée 187 (commentaire de Mathieu, 2026-11-12 : « commencer l'écran de configuration des préférences, qui servira
  aussi pour le choix de la langue, le mode full screen par défaut, etc. ») ; spec 076, hors périmètre ; spec 075, Q7

## Objectif

Un élément **Préférences…** du Menu (et un bouton de l'écran de lancement) ouvre une fenêtre où le joueur choisit la
**langue** du jeu et s'il démarre **en plein écran**. Les choix s'appliquent tout de suite et sont **gardés d'une session à
l'autre** dans un petit fichier, lu au lancement. La fenêtre est faite pour grandir : l'audio (spec 081) y ajoutera ses
volumes, les animations (spec 079) leur case.

## Règles

Aucune règle du livret. Décisions que la spec applique :

- **Langue au lancement** (spec 076, complétée) : `--lang` d'abord, puis **la langue des préférences**, puis la langue du
  système, puis l'anglais. Une langue des préférences sans fichier (un `de` écrit à la main, ou une langue retirée) est sautée
  avec un `warn` dans les logs : on passe à la langue du système.
- **« Langue du système »** est un choix de la liste, et le choix par défaut : avec lui, rien ne change par rapport à la
  spec 076.
- **Changer de langue dans la fenêtre** vaut tout de suite, partie en cours comprise (l'écran ne lit que le catalogue,
  ADR 0003), et l'emporte sur `--lang` pour le reste de la session : le dernier choix du joueur gagne.
- **Plein écran au lancement** : une case, décochée par défaut. F11 et l'élément Plein écran du Menu (spec 074) ne la
  touchent pas : ils basculent la fenêtre pour la session seulement, et restent actifs quelle que soit la valeur de la case (Q3).
- **Enregistrement immédiat** : chaque changement est écrit dans le fichier (pas de bouton « Appliquer » ni « Annuler »).
- **Rien dans la partie** : les préférences ne vont ni dans l'état, ni dans le journal, ni dans les sauvegardes ; Revenir en
  arrière ne les touche pas.
- **Logs en français**, comme toujours (spec 003).

## Modèle

Termes ajoutés au glossaire (nouvelle section « Préférences (spec 080) ») :

| Terme | Code | Sens |
|---|---|---|
| Préférences | `Preferences` | Réglages du joueur gardés d'une session à l'autre : langue, plein écran au lancement (puis volumes, animations). |
| Fichier des préférences | `preferences.json` | Fichier JSON où elles sont écrites, dans le dossier de configuration. |
| Dossier de configuration | `config_dir` | Dossier du fichier des préférences (ordre ci-dessous). |
| Fenêtre des préférences | `PreferencesWindow` | Fenêtre ouverte par Préférences… |
| Langue des préférences | `Preferences::language` | `None` = langue du système ; sinon un code (`fr`, `en`). |

### Le fichier

```json
{
  "version": 1,
  "language": "fr",
  "fullscreen_at_launch": false
}
```

- **Format** : JSON, comme les sauvegardes (`serde_json`, déjà dans le projet) ; `"language": null` ou absent = langue du
  système.
- **Tolérant** : un champ absent prend sa valeur par défaut, un champ inconnu est ignoré. Ainsi la spec 081 ajoute ses
  volumes sans migration, et un fichier d'une version plus récente se lit encore (ses champs inconnus sont perdus à la
  prochaine écriture : sans gravité). `version` sert à une future migration ; une valeur autre que 1 donne un `warn` et la
  lecture continue.
- **Code** : un module `preferences` de **arcanum-persistance** (Q1), qui a déjà `serde` et `serde_json` : le type
  `Preferences` (avec `Default`), `load(path) -> Result<Preferences, …>` et `save(path, &Preferences) -> Result<(), …>`,
  erreurs typées (`thiserror`). Aucune dépendance nouvelle.
- **Écriture sûre** : dans un fichier temporaire du même dossier, puis renommé à la place de l'ancien, pour qu'un arrêt en
  pleine écriture ne laisse pas un fichier tronqué ; le dossier est créé s'il manque.

### Le dossier de configuration

Même ordre que les sauvegardes (spec 024, `choose_saves_dir`) et les logs (spec 003), fonction pure
`choose_config_dir` :

1. la variable `ARCANUM_CONFIG_DIR` si elle est posée et non vide ;
2. lancé par cargo (`CARGO_MANIFEST_DIR`) : `config/` à la racine du dépôt, ajouté au `.gitignore` comme `saves/` et
   `logs/` ;
3. sinon le dossier de configuration de l'utilisateur (`directories::ProjectDirs::config_dir`, application
   `iter-arcanum` : `~/.config/iter-arcanum/` sous Linux, `%APPDATA%\iter-arcanum\config\` sous Windows).

Ce dossier s'appelait `mage-knight` ; la spec 082 l'a renommé `iter-arcanum`, comme ceux des sauvegardes et des logs, sans
reprise de l'ancien.

### Ce que fait le lancement

```
dossier de configuration ─► preferences.json ─► lu ? ──oui──► Preferences
                                                  ├─ absent ─────► défauts (info dans les logs)
                                                  └─ illisible ──► défauts (error dans les logs, fichier laissé tel quel
                                                                    jusqu'au prochain changement)

langue : --lang ► langue des préférences (si disponible) ► langue du système ► anglais
fenêtre : plein écran dès l'ouverture si fullscreen_at_launch (ViewportBuilder::with_fullscreen)
```

## Actions et événements

Aucune action ni événement de jeu. Commandes d'écran nouvelles (`Command`, comme Plein écran) :

| Commande | Effet |
|---|---|
| `Command::OpenPreferences` | Ouvre la fenêtre des préférences (la ferme si elle est ouverte). |
| `Command::ClosePreferences` | Ferme la fenêtre. |
| `Command::SetLanguage(Option<usize>)` | Change la langue des préférences (sa place dans la liste des langues disponibles, `None` = langue du système : `Command` reste `Copy`), recharge le catalogue, écrit le fichier. |
| `Command::SetFullscreenAtLaunch(bool)` | Change la case, écrit le fichier. Ne bascule pas la fenêtre. |

La session (`ClientSession`) tient les préférences et le chemin du fichier. Le rechargement du catalogue passe par une
demande à l'application (`WindowRequest::Language(Language)`, nouveau), qui remplace son `Catalog` et redonne le titre
de la fenêtre (`ViewportCommand::Title`) : la session reste testable sans fenêtre, comme pour Plein écran (spec 074).

## Comportements attendus

Chaque cas devient au moins un test (nom indicatif entre parenthèses). Le dossier des ressources des tests est celui du dépôt
(`repository_assets_dir`, `fr.json` et `en.json`) ; `language.name` (« Français », « English ») reconnaît le catalogue
chargé ; le dossier de configuration est un dossier temporaire (`tempfile`).

**Le fichier (arcanum-persistance)**

- **C1** — `Preferences::default()` : langue du système (`None`), pas de plein écran au lancement
  (`default_preferences_follow_system_and_window`).
- **C2** — Écrire puis relire des préférences (`fr`, plein écran) rend les mêmes (`preferences_round_trip`).
- **C3** — Un fichier sans `fullscreen_at_launch` se lit avec `false` ; un fichier avec un champ inconnu (`"music": 40`)
  se lit sans erreur (`preferences_tolerate_missing_and_unknown_fields`).
- **C4** — `"version": 2` se lit, avec un `warn` dans les logs (`newer_preferences_version_is_read_with_warning`).
- **C5** — Écrire dans un dossier qui n'existe pas le crée ; après l'écriture, aucun fichier temporaire ne reste à côté
  (`saving_preferences_creates_dir_and_leaves_no_temp_file`).

**Le dossier de configuration (arcanum-ui)**

- **C6** — `choose_config_dir` : `ARCANUM_CONFIG_DIR` l'emporte ; vide, il est sauté ; lancé par cargo, `config/` du dépôt ;
  sinon le dossier de l'utilisateur (`config_dir_choice`) — même forme que `saves_dir_choice` (spec 024, `sauvegarde_ui.rs`).

**Le lancement**

- **C7** — Sans `--lang`, langue des préférences `en`, langue du système `fr` : l'anglais est chargé, et les logs disent
  « langue : en (préférences) » (`preferred_language_beats_system_language`).
- **C8** — `--lang fr` avec la langue des préférences `en` : le français est chargé (« langue : fr (paramètre --lang) »)
  (`lang_argument_beats_preferred_language`).
- **C9** — Langue des préférences `de` (sans fichier), langue du système `fr` : le français est chargé, avec un `warn` qui
  nomme `de` (`unavailable_preferred_language_falls_back_to_system`).
- **C10** — Langue des préférences `None` : comme la spec 076 (C8 à C12 de celle-ci, inchangés)
  (`system_choice_keeps_spec_076_behaviour`).
- **C11** — Fichier absent : préférences par défaut, `info` dans les logs ; fichier illisible (JSON invalide) : préférences
  par défaut, `error` dans les logs qui nomme le fichier, et le fichier n'est pas réécrit au lancement
  (`missing_or_unreadable_preferences_give_defaults`).
- **C12** — `fullscreen_at_launch` vrai : la fenêtre décrite au lancement est en plein écran (`viewport(…)` reçoit les
  préférences, `ViewportBuilder::fullscreen` vaut `Some(true)`) ; faux : `None` ou `Some(false)`
  (`viewport_starts_fullscreen_when_preferred`).

**La fenêtre**

- **C13** — Le Menu a un élément « Préférences… » avec « Ctrl+, » en face, entre Plein écran et Quitter, séparateurs
  compris : …, Recentrer le plateau, Plein écran, séparateur, Préférences…, séparateur, Quitter
  (`menu_lists_preferences_before_quit`). Test existant modifié : `menu_lists_commands_with_keys` (spec 074, C1).
- **C14** — Préférences… est actif en mode choix, en combat et derrière la fenêtre de nouvelle partie, comme Plein écran
  (`preferences_entry_is_always_enabled`).
- **C15** — `Command::OpenPreferences` ouvre la fenêtre ; elle montre la langue retenue et la case
  (`PreferencesWindow::language`, `fullscreen_at_launch`) ; une seconde fois, ou `ClosePreferences`, la ferme
  (`open_preferences_toggles_window`).
- **C16** — La liste des langues : « Langue du système (Français) » en tête (le nom entre parenthèses est celui de la
  langue que la langue du système donne, anglais compris quand elle n'a pas de fichier), puis une ligne par langue
  disponible, sous son propre nom (« English », « Français »), dans l'ordre des codes ; le choix des préférences est coché
  (`language_list_names_each_language_in_itself`).
- **C17** — Choisir « English » : la session demande `WindowRequest::Language(en)`, la fenêtre et tout l'écran se lisent
  ensuite dans le catalogue anglais (titre de la fenêtre « Preferences »), et le fichier contient `"language": "en"`
  (`choosing_language_switches_catalog_and_saves`).
- **C18** — Choisir « Langue du système » : la langue du système (ou l'anglais) est rechargée, le fichier contient
  `"language": null` (`choosing_system_language_saves_null`).
- **C19** — En partie, changer de langue ne touche pas la partie : journal de même longueur, Revenir en arrière dans le même
  état qu'avant (`language_change_leaves_game_untouched`).
- **C20** — Cocher « Plein écran au lancement » écrit `true` dans le fichier, sans demande `WindowRequest::Fullscreen` ;
  F11 ensuite ne change pas la case (`fullscreen_at_launch_is_independent_of_f11`) — Q3.
- **C21** — Écriture impossible (dossier en lecture seule, ou chemin qui est un fichier) : le choix s'applique quand même
  pour la session, la fenêtre montre « Préférences non enregistrées (voir les logs). » et une ligne `error` des logs donne
  le chemin et la cause ; le message disparaît au prochain enregistrement réussi
  (`unsaved_preferences_show_message`).
- **C22** — Touches : `KEY_BINDINGS` lie Ctrl+, à Préférences (`Shortcut::Preferences`, libellé « Ctrl+, ») ; Échap
  ferme la fenêtre ouverte, avant toute autre fenêtre (`preferences_shortcuts`).
- **C23** — Au lancement (écran de nouvelle partie, sans barre de menu) : le pied montre « Préférences… » à côté de
  « Charger… », et Ctrl+, l'ouvre aussi ; changer de langue y traduit l'écran de nouvelle partie
  (`launch_screen_opens_preferences`).

## Cas limites et refus

- **L1** — Aucun dossier des ressources : la liste n'a que « Langue du système » (aucune langue disponible) ; le choix ne
  recharge rien d'autre que le catalogue vide (spec 002 L5) (`preferences_without_assets_list_system_only`).
- **L2** — Aucun dossier de configuration trouvable (`choose_config_dir` rend `None`, comme `choose_saves_dir`) : les
  préférences par défaut s'appliquent, la fenêtre fonctionne pour la session et montre le message de C21 au premier
  changement (`no_config_dir_keeps_session_preferences`).
- **L3** — Le fichier de la langue choisie existe mais ne se charge pas : même repli que la spec 076 L1 (anglais, puis
  clés visibles), et la préférence reste écrite telle que choisie (`unloadable_chosen_language_falls_back`).
- **L4** — Fenêtre des préférences ouverte : les touches de jeu ne jouent pas (comme derrière la fenêtre de chargement,
  spec 024 L7) ; F11, Ctrl+Q et Ctrl+, agissent (`preferences_window_blocks_game_keys`).
- **L5** — Les messages des logs de C4, C7, C9, C11 et C21 sont en français, en dur, jamais tirés du catalogue (spec 003).

### Tests existants touchés

Relevés par `grep -rn "launch_catalog\|fn viewport\|menu_lists_commands_with_keys" crates` :

- `launch_catalog` reçoit la langue des préférences en plus (nom et ordre indicatifs) : ses dix-sept appels de
  `crates/arcanum-ui/tests/choix_de_la_langue.rs` passent `None`, même comportement ; `main.rs` lui passe celle du fichier.
- `viewport(title, icon)` (`crates/arcanum-ui/src/icon.rs`) reçoit le plein écran au lancement : appel de `main.rs`, et quatre
  appels de `crates/arcanum-ui/tests/icone.rs` et `icone_logs.rs` (spec 004, oubliés à la rédaction) qui passent `false`, même
  comportement.
- `menu_lists_commands_with_keys` (`crates/arcanum-ui/tests/barre_de_menu.rs`, spec 074 C1) : la liste attendue gagne
  Préférences… et un séparateur (C13).

## Information cachée

Sans objet : aucune donnée de partie, la vue (`PlayerView`) ne change pas. La fenêtre lit `PreferencesWindow`, calculée par
la session à partir des préférences, des langues disponibles et de la langue du système (sans egui, comme `MenuBar`).

## Libellés (i18n)

Nouveau groupe `preferences` (absent de `fr.json`, vérifié : `grep -n '"preferences"' assets/i18n/fr.json` ne donne rien) :

| Clé | Paramètres | fr | en |
|---|---|---|---|
| `preferences.title` | — | Préférences | Preferences |
| `preferences.button` | — | Préférences… | Preferences… |
| `preferences.language` | — | Langue | Language |
| `preferences.system_language` | `language` | Langue du système ({language}) | System language ({language}) |
| `preferences.fullscreen_at_launch` | — | Plein écran au lancement | Full screen at launch |
| `preferences.fullscreen_hint` | — | F11 bascule la fenêtre sans changer ce réglage. | F11 toggles the window without changing this setting. |
| `preferences.not_saved` | — | Préférences non enregistrées (voir les logs). | Preferences not saved (see the logs). |
| `preferences.close` | — | Fermer | Close |

Texte obtenu : `preferences.system_language` avec `{language}` = « Français » (valeur de `language.name` dans `fr.json`)
donne « Langue du système (Français) », et en anglais avec « English » « System language (English) ». Le nom entre
parenthèses vient du catalogue de la langue du système, pas de celui affiché : un joueur en anglais sur un système français
lit « System language (Français) ».

Réutilisés :

- `language.name` (spec 001) : « Français » dans `fr.json`, « English » dans `en.json` ; lu dans le catalogue de chaque
  langue pour la liste (C16).
- `shortcut.key.ctrl` « Ctrl+{key} » avec `{key}` = « , » donne « Ctrl+, » (fr et en) ; egui nomme la touche « Comma »
  (`Key::Comma.name()`) : `key_label` (`crates/arcanum-ui/src/keys.rs`) gagne une ligne qui l'écrit « , », sans traduction.

`preferences.button` sert au Menu et au pied du lancement ; « … » est déjà affiché (Charger…). Aucun caractère nouveau.

## Interface

```
┌ Préférences ─────────────────────────────── × ┐
│ Langue                                         │
│   [Langue du système (Français)          ▾]    │
│      Langue du système (Français)              │
│      English                                   │
│      Français                                  │
│                                                │
│ [✓] Plein écran au lancement                   │
│     F11 bascule la fenêtre sans changer ce     │
│     réglage.                                   │
│                                                │
│ (Préférences non enregistrées (voir les logs).)│
│                                     [Fermer]   │
└────────────────────────────────────────────────┘
```

- **Fenêtre** : petite fenêtre centrée, au-dessus de l'écran comme la fenêtre de chargement, titre
  `preferences.title` (C15). Mise en page : vérification à l'œil.
- **Langue** : une liste déroulante (`ComboBox`, déjà employée par l'écran de nouvelle partie) avec les choix de C16
  (`PreferencesWindow::languages`) ; changer de choix vaut tout de suite (C17, C18).
- **Plein écran au lancement** : une case à cocher, et sous elle sa note `preferences.fullscreen_hint` en petit (C20).
- **Message d'échec** `preferences.not_saved`, en couleur d'avertissement, seulement après un échec (C21,
  `PreferencesWindow::not_saved`).
- **Fermer** (bouton), la croix de la fenêtre et Échap la ferment (C15, C22).
- **Menu** : « Préférences… » et sa touche (C13, C14).
- **Lancement** : bouton « Préférences… » dans le pied de l'écran de nouvelle partie, après « Charger… » (C23).
- **Sections à venir** : la fenêtre est une suite de sections (Langue, Affichage) ; la spec 081 y ajoutera « Son », la
  spec 079 sa case dans « Affichage ». Rien d'autre aujourd'hui : pas de section vide.

Parties préparées : aucune. La vérification à l'œil part du lancement (`scripts/verif-manuelle.sh demarrer`) : ouvrir
la fenêtre par Ctrl+,, passer en anglais (capture de l'écran de nouvelle partie traduit), commencer une partie, rouvrir
par le Menu et repasser en français (capture du plateau) ; cocher le plein écran, relancer, voir le jeu démarrer en plein
écran. `verif-manuelle.sh` pose `ARCANUM_CONFIG_DIR` sur un dossier temporaire neuf à chaque `demarrer` et `charger`, pour
qu'une préférence de développement (plein écran, langue) ne fausse pas les captures ; `--lang fr` reste passé (spec 076).

### Raccourcis clavier

| Touche | Effet | Contexte |
|---|---|---|
| Ctrl+, | Ouvrir ou fermer la fenêtre des préférences | Toujours, lancement compris |
| Échap | Fermer la fenêtre des préférences (avant toute autre fenêtre) | Fenêtre des préférences ouverte |

Reportées dans `docs/raccourcis.md`. Ctrl+, est la touche des préférences de nombreux logiciels ; sur un clavier AZERTY, la
virgule est une touche directe (Q2).

## Documentation

- **`docs/architecture.md`** : le module `preferences` d'arcanum-persistance, le dossier de configuration et l'ordre de la
  langue au lancement.
- **README** : où sont gardées les préférences ; `--lang` force la langue pour une session.
- **`docs/verification-manuelle.md`** : `ARCANUM_CONFIG_DIR` posé par le script.
- **Glossaire** : les termes de la section Modèle.

## Hors périmètre

- **Volumes par type de son** : spec 081, qui ajoute sa section et ses champs au fichier (tolérant, C3).
- **Case « Animations »** : spec 079 (spec 075, Q7).
- **Taille et position de la fenêtre** gardées d'une session à l'autre : pas demandées.
- **Thème clair ou sombre**, taille du texte : pas demandés.
- **Réglages de partie** (scénario, héros, options du fantôme par défaut) : l'écran de nouvelle partie les tient.
- **macOS** : son dossier de configuration vient de `directories` sans rien de plus ; la spec #7 le vérifie.
- **Renommage du dossier** `mage-knight` : fait par la spec 082, sans reprise.

## Décisions

Validées par Mathieu le 2026-11-20 : les cinq réponses proposées sont retenues, Q3 précisée.

- **Q1 — Code du fichier** : un module `preferences` d'arcanum-persistance, dont la description de crate est élargie
  (« sauvegarde des parties et des préférences »). Aucune dépendance nouvelle.
- **Q2 — Touche** : Ctrl+,.
- **Q3 — Plein écran au lancement** : une case des préférences, qui décide seule si le jeu démarre en plein écran (C12).
  F11 et l'élément Plein écran du Menu restent actifs quelle que soit sa valeur et ne la changent pas (C20) (Mathieu :
  « le paramètre influence bien le plein écran au démarrage, F11 reste fonctionnel quelle que soit sa valeur »).
- **Q4 — Changer de langue en partie** : tout de suite (C17, C19).
- **Q5 — Enregistrement** : immédiat, sans Appliquer ni Annuler.

## Questions ouvertes

Aucune.
