# 003 — Logs techniques

- Statut : implémentée (2026-10-24)
- Jalon : J0
- Dépend de : 000 (boucle de partie), 001 (manques de libellés), 002 (création d'arcanum-ui et de l'exécutable)
- Livret : aucune règle de jeu
- Crates touchées : arcanum-ui (installation des logs, fichier), arcanum-core (boucle de partie), arcanum-i18n (écriture des manques)

## Objectif

Quand le jeu se comporte mal chez Mathieu, sous Windows comme sous Linux, on peut ouvrir un fichier texte qui dit ce qui s'est passé :
lancement, déroulé de la partie, erreurs, libellés manquants, plantage. Toutes les crates écrivent par la même façade ; seul
l'exécutable décide où et comment c'est écrit. En développement, le fichier est à portée de main, dans le dossier `logs` du dépôt.

Les logs techniques servent au diagnostic. Ils ne sont **pas** le journal de la partie (`EventLog`, spec 000) : ils ne sont ni
rejoués, ni sauvegardés, ni montrés au joueur, et le moteur ne s'en sert jamais pour décider.

## Règles

Pas de règle du livret. Contraintes reprises du projet :

- ADR 0003 : une clé de libellé manquante est « journalisée » ; cette spec dit où et comment.
- ADR 0004 : dossiers via une crate standard, chemins en `PathBuf`, fichiers en UTF-8, comportement identique sous Windows et Linux.
- `arcanum-core` reste pur (`.claude/rules/arcanum-core.md`) : il n'utilise que les macros de la façade, sans fichier ni horloge, et jamais dans
  ses fonctions pures (voir « Logs d'arcanum-core »).

## Modèle

Nouveaux termes à ajouter au glossaire (vocabulaire technique, dans la même PR que cette spec) :

| Français | Code | Sens |
|---|---|---|
| Logs techniques | `log` (crate) | Messages de diagnostic écrits pendant l'exécution. Jamais appelés « journal », réservé à `EventLog`. |
| Gravité | `log::Level` | `error`, `warn`, `info`, `debug`, `trace`. Jamais appelée « niveau », réservé au niveau du héros (`Level`). |
| Fichier de logs | `LogFile` | Fichier texte où l'exécutable écrit les logs d'un lancement. |
| Dossier des logs | `logs_dir` | Dossier qui contient les fichiers de logs. |

### Architecture

- **Façade** : la crate `log`, standard de l'écosystème Rust. Les bibliothèques (arcanum-core, arcanum-i18n, plus tard arcanum-persistance et
  arcanum-players) n'utilisent que ses macros (`warn!`, `info!`…) : sans installation, elles ne font rien. egui et eframe passent déjà
  par `log`, leurs messages arrivent donc au même endroit.
- **Écriture** : un module `logging` d'arcanum-ui, installé une fois au démarrage, avant la fenêtre. Il écrit dans le fichier de logs
  et, dans les builds de développement (`debug_assertions`), recopie aussi sur la sortie d'erreur.
- **Testabilité** : le logger global ne s'installe qu'une fois par processus. Le code est donc découpé en morceaux testables seuls :
  formater une ligne, filtrer selon la gravité et la crate, choisir le dossier, faire tourner les fichiers dans un dossier donné,
  lire la gravité demandée. Les tests passent toujours un dossier temporaire, jamais le vrai dossier de l'utilisateur ni celui du dépôt.
  Les tests qui vérifient ce qu'une bibliothèque écrit (C11, C12) installent un logger de capture qui range les lignes par thread,
  pour que les tests restent parallèles.

### Emplacement

Le dossier des logs est choisi au démarrage, dans cet ordre :

1. **`ARCANUM_LOG_DIR`** : si cette variable d'environnement est définie et non vide, c'est le dossier des logs, tel quel.
2. **Lancement par cargo** (`cargo run`, en debug comme en release) : cargo définit `CARGO_MANIFEST_DIR` au lancement ; les logs vont
   alors dans le dossier `logs` à la racine du dépôt, c'est-à-dire deux niveaux au-dessus de `crates/arcanum-ui`. Ce dossier est ajouté
   au `.gitignore` (`/logs/`).
3. **Jeu déployé** : le dossier de données de l'utilisateur, via la crate `directories`, application nommée `iter-arcanum` :

| Système | Dossier des logs du jeu déployé |
|---|---|
| Windows | `%LOCALAPPDATA%\iter-arcanum\data\logs` |
| Linux | `$XDG_STATE_HOME/iter-arcanum/logs`, par défaut `~/.local/state/iter-arcanum/logs` |

Sous Linux, la norme XDG range les logs dans le dossier d'état, pas dans celui des données. Sous Windows, dossier local et non
itinérant : des logs n'ont pas à suivre l'utilisateur d'un poste à l'autre. Le code propre à chaque système (choix du dossier d'état
ou de données) est isolé derrière `#[cfg]`.

Le choix lit l'environnement une seule fois, puis passe par une fonction pure (variables lues → dossier) qui est celle qu'on teste.

### Fichiers et rotation

- Le lancement en cours écrit dans `iter-arcanum.log`.
- Au démarrage, les fichiers précédents sont décalés : `iter-arcanum.log` → `iter-arcanum.1.log` → … → `iter-arcanum.5.log` ;
  l'ancien `iter-arcanum.5.log` est supprimé. On garde donc le lancement en cours et les 5 précédents, quel que soit le dossier.
- Pas de limite de taille dans un même lancement (voir Hors périmètre).

### Format d'une ligne

```
2026-10-24T15:51:32.123+02:00 WARN  arcanum_i18n: clé de libellé manquante : card.march.name
```

Horodatage en **heure locale** au format RFC 3339 à la milliseconde, avec le décalage horaire (indispensable pour lire les lignes
autour d'un changement d'heure), gravité sur 5 caractères, crate d'origine (cible `log`), message.
UTF-8, fin de ligne `\n` sur les deux systèmes. Chaque ligne est vidée sur le disque aussitôt, pour ne rien perdre en cas de plantage.
Les messages sont écrits en français, comme les erreurs déjà définies dans arcanum-core et arcanum-i18n ; ce ne sont pas des libellés affichés,
ils ne passent donc pas par `assets/i18n/`.

### Gravités

- Nos crates (`mk_*`) : `info` par défaut.
- La variable d'environnement `ARCANUM_LOG` change la gravité minimale de nos crates : `error`, `warn`, `info`, `debug` ou `trace`,
  sans tenir compte de la casse.
- Les autres crates (egui, eframe, wgpu…) : toujours `warn` et au-dessus, pour ne pas noyer le fichier.

### Logs d'arcanum-core

La boucle de partie (`GameRunner`) écrit ce qui aide à comprendre une partie, à partir de ce qu'elle fait déjà :

| Moment | Gravité | Contenu |
|---|---|---|
| Création de la partie | `info` | graine de la configuration de départ, nombre de joueurs |
| Décision demandée à un joueur (une fois par décision) | `debug` | joueur, nombre d'actions légales |
| Action choisie | `debug` | joueur, action |
| Événement journalisé | `trace` | numéro dans le journal, événement |
| Erreur renvoyée par `step` | `error` | l'erreur (joueur manquant, choix hors des actions légales, action refusée) |
| Fin de partie (première fois que `step` le constate) | `info` | nombre d'événements du journal |

Les fonctions pures du moteur (`apply`, `decide`, `legal_actions`, `view`, `pending_decision`, `simulate`) n'écrivent **jamais** de
logs : elles doivent rester sans effet de bord, et un joueur automatique (MCTS) les appellera des milliers de fois par décision.

La graine en `info` suffit à rejouer une partie qui a posé problème. Actions et événements peuvent révéler de l'information cachée
(carte piochée) : ils restent en `debug` et `trace`, jamais au-dessus (voir Information cachée).

## Actions et événements

Aucun nouveau : les logs ne passent pas par le moteur et n'en changent pas le comportement.

## Comportements attendus

- **C1** — Une ligne de logs est formatée `horodatage gravité cible: message`, avec un horodatage en heure locale RFC 3339 à la
  milliseconde, décalage horaire compris (`log_line_has_local_timestamp_level_target_and_message`).
- **C2** — Sans `ARCANUM_LOG`, un `info` de `arcanum_ui` est écrit et un `debug` de `arcanum_ui` ne l'est pas (`default_level_is_info_for_our_crates`).
- **C3** — Avec `ARCANUM_LOG=debug` (ou `DEBUG`), un `debug` de nos crates est écrit (`arcanum_log_variable_sets_our_level`).
- **C4** — Un `info` d'une crate externe (cible `wgpu_core`) n'est pas écrit, un `warn` l'est, quelle que soit `ARCANUM_LOG`
  (`external_crates_are_filtered_at_warn`).
- **C5** — Avec `ARCANUM_LOG_DIR` défini, le dossier des logs est cette valeur, même lancé par cargo (`arcanum_log_dir_overrides_everything`).
- **C6** — Sans `ARCANUM_LOG_DIR` et avec `CARGO_MANIFEST_DIR` valant `<racine>/crates/arcanum-ui`, le dossier des logs est `<racine>/logs`
  (`cargo_run_writes_logs_in_repository`).
- **C7** — Sans ces deux variables, le dossier des logs est construit à partir du dossier de base de l'utilisateur, sans séparateur
  en dur : `<base>/iter-arcanum/logs` sous Linux (dossier d'état), `<base>\iter-arcanum\data\logs` sous Windows
  (`deployed_game_writes_logs_in_user_data_dir`).
- **C8** — Au démarrage, dans un dossier contenant `iter-arcanum.log` et `iter-arcanum.1.log` à `iter-arcanum.5.log`, la rotation
  décale chaque fichier d'un rang, supprime l'ancien `.5` et laisse la place à un `iter-arcanum.log` neuf (`startup_rotates_previous_log_files`).
- **C9** — Si le dossier des logs n'existe pas, il est créé avec ses parents (`logs_dir_is_created_if_missing`).
- **C10** — Au démarrage, la première ligne `info` donne la version du jeu, le système, la gravité retenue, le chemin du fichier de
  logs et la raison du choix du dossier (`ARCANUM_LOG_DIR`, cargo ou jeu déployé) (`startup_line_describes_the_run`).
- **C11** — La première fois qu'un catalogue relève un manque (clé manquante ou paramètre non fourni, spec 001 C5 à C7), arcanum-i18n écrit
  un `warn` qui nomme la clé, et le paramètre s'il y en a un. Résoudre à nouveau le même manque n'écrit rien de plus
  (`gap_is_logged_once_as_warning`).
- **C12** — Une partie du jeu jouet (spec 000) menée par `GameRunner` jusqu'à sa fin écrit, dans l'ordre : la création avec la graine
  (`info`), pour chaque décision la demande puis l'action choisie (`debug`), chaque événement avec son numéro (`trace`), et la fin
  (`info`) une seule fois même si `step` est rappelé (`game_runner_logs_the_game`).
- **C13** — Quand `step` renvoie une erreur, elle est aussi écrite en `error` (`game_runner_logs_its_errors`).
- **C14** — `apply`, `decide`, `legal_actions`, `view`, `pending_decision` et `simulate` n'écrivent rien, à toute gravité
  (`pure_engine_functions_never_log`).
- **C15** — Un panic est écrit en `error` avec son message et son emplacement avant la fin du programme ; le comportement habituel du
  panic est conservé ensuite (`panic_is_written_to_the_log`).
- **C16** — `/logs/` figure dans le `.gitignore` du dépôt (vérifié à la relecture, pas par un test).
- **C17** — Ce que la spec 002 a laissé à la spec 003 : au démarrage, le dossier des ressources retenu est écrit en `info` ; si
  aucun emplacement ne contient `i18n/fr.json` (spec 002 L5), un `error` liste les emplacements essayés ; si `fr.json` ne se charge
  pas (spec 002 L6), un `error` donne l'erreur de chargement. Le jeu démarre ensuite comme le prévoit la spec 002
  (`assets_dir_search_is_logged`).

## Cas limites et refus

- **L1** — `ARCANUM_LOG` vaut une valeur inconnue (`bavard`) : gravité par défaut `info`, et un `warn` le signale au démarrage
  (`unknown_mk_log_value_falls_back_to_info`).
- **L2** — Le dossier des logs ne peut pas être créé ou le fichier ne peut pas être ouvert (droits, disque plein, `ARCANUM_LOG_DIR` invalide) :
  le jeu se lance quand même, les logs vont seulement sur la sortie d'erreur ; jamais de panic
  (`unwritable_logs_dir_does_not_stop_the_game`).
- **L3** — Jeu déployé et aucun dossier personnel trouvé (`directories` ne renvoie rien) : même repli que L2.
- **L4** — Un fichier de la rotation ne peut pas être renommé ou supprimé (par exemple ouvert par une autre instance du jeu sous
  Windows) : on n'interrompt pas la rotation, on écrit dans `iter-arcanum.log` si possible, sinon repli L2, et un `warn` le signale
  (`rotation_failure_is_not_fatal`).
- **L5** — Un message contenant des retours à la ligne (trace de panic) est écrit tel quel ; seule sa première ligne porte
  l'horodatage.
- **L6** — Le décalage horaire local ne peut pas être déterminé : l'horodatage est écrit en UTC (suffixe `Z`), sans erreur
  (`unknown_local_offset_falls_back_to_utc`).

## Information cachée

Les logs sont un outil de développement et ne sont pas montrés au joueur. Ils peuvent contenir de l'information cachée (carte
piochée, ordre du deck), mais seulement en `debug` et `trace` : à la gravité par défaut `info`, un fichier de logs ne révèle rien
qu'un joueur ne sache déjà. Toute spec qui ajoute des logs respecte cette règle.

## Libellés (i18n)

Aucune clé : les logs ne sont pas affichés au joueur.

## Interface

Aucune. Le chemin du fichier est écrit dans la première ligne de logs (C10) et dans la sortie d'erreur des builds de développement.

## Hors périmètre

- **Limite de taille** d'un fichier dans un même lancement, compression, envoi des logs : plus tard si un besoin réel apparaît.
- **Bouton « ouvrir le dossier des logs »** dans l'UI : avec un futur menu.
- **Filtres par crate** plus fins que « nos crates / les autres » (syntaxe à la `RUST_LOG`).
- **Logs structurés** (JSON, `tracing`, spans) : la façade `log` suffit pour un client solo.
- **Logs des joueurs automatiques** (temps de calcul, raisonnement) : dans la spec de chaque joueur.

## Dépendances

Validées par Mathieu le 2026-10-24 : `log` (arcanum-core, arcanum-i18n, arcanum-ui), `directories` (arcanum-ui), `tempfile` (arcanum-ui, tests seulement), et :

- `chrono` (arcanum-ui), sans ses fonctions par défaut, avec seulement `clock` : heure locale et décalage horaire sous Windows et Linux.
  Écartée : `time`, qui refuse de lire le fuseau local sous Linux dès que le programme a plusieurs threads, ce qui est le cas
  d'eframe.

Pas de crate d'écriture de logs toute faite (`simplelog`, `flexi_logger`, `fern`) : ce qu'on demande tient en une centaine de lignes
testées.

## Décisions prises à la relecture (2026-10-24)

1. Dépendances `log`, `directories` et `tempfile` acceptées.
2. On garde le lancement en cours et les 5 précédents.
3. Horodatage en heure locale.
4. Le dossier des logs varie : dossier `logs` du dépôt quand le jeu est lancé par cargo, dossier de données de l'utilisateur une fois
   déployé.
5. arcanum-core écrit des logs, depuis la boucle de partie seulement. À l'implémentation, `.claude/rules/arcanum-core.md` sera complété en ce sens.
6. La variable `ARCANUM_LOG_DIR` (C5) est gardée.
7. `chrono` (fonction `clock` seule) remplace `humantime`, qui ne sait écrire que de l'UTC.
8. C17 ajouté après la fusion de la spec 002, pour écrire dans les logs ce qu'elle a laissé à la 003 : accepté.

## Questions ouvertes

Aucune.
