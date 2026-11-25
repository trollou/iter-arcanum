# #7 — Support de macOS

- Statut : à valider
- Issue : #7 (Epic #31 Finitions v0.1.0, milestone v0.1.0)
- Dépend de : spec 076 (langue du système), spec 080 (préférences), spec 081 (son), spec 082 (dépôt public), spec 003
  (logs), spec 004 (icône), spec 074 (touches du Menu)
- Livret : aucun (spec technique)
- Crates touchées : arcanum-ui (langue, dossiers, libellé des touches) ; CI, ADR 0004, docs

## Objectif

macOS rejoint Windows et Linux comme cible : le jeu se compile, passe ses tests et tourne sur un Mac depuis le dépôt
(`cargo run -p arcanum-ui --release`), dans la langue du système, avec ses fichiers rangés là où macOS les attend et les
touches affichées « Cmd+… ». La CI le vérifie à chaque PR prête, puisque personne dans le projet n'a de Mac.

Le paquet téléchargeable (`.app`, `.dmg`, signature) n'est pas ici : il relève du mécanisme de release (issue #33, bloquée
par celle-ci), qui s'appuiera sur ce que cette spec rend vérifiable.

## Règles

Aucune règle du jeu. Les décisions de portabilité suivent l'ADR 0004, que cette spec étend à macOS.

## Ce qui marche déjà sans rien faire (relu dans le code)

Relu pendant la rédaction, à confirmer par la première CI macOS (C1) :

- **Raccourcis** : ils sont lus avec `Modifiers::COMMAND` (`crates/arcanum-ui/src/app/mod.rs:145`), qui est Cmd sous macOS
  et Ctrl ailleurs. Seul leur **libellé** reste « Ctrl+… » (C7).
- **Son** : Kira passe par cpal, donc CoreAudio, sans dépendance système à installer (ADR 0010, spec 081). Les tests du son
  n'ouvrent pas de sortie réelle (`crates/arcanum-ui/tests/audio.rs` n'emploie ni Kira ni cpal).
- **Préférences et sauvegardes** : `directories::ProjectDirs::from("", "", "iter-arcanum")` (`preferences.rs:41`,
  `saves.rs:31`) donne sous macOS `~/Library/Application Support/iter-arcanum/` pour la configuration comme pour les données :
  `preferences.json` et le dossier des sauvegardes y sont côte à côte, sans collision de nom (C5).
- **Ressources** : `assets_candidates` (`assets.rs:36`) trouve `assets/` à côté de l'exécutable ou dans le dépôt ; `cargo run`
  et un `target/release/iter-arcanum` lancé depuis le dépôt marchent tels quels.
- **Icône de la fenêtre** : eframe pose l'icône du `ViewportBuilder` (`icon.rs:66`) dans le Dock quand le programme ne tourne
  pas depuis un paquet `.app` (vérification à l'œil, Q3).
- **Rendu** : glow (OpenGL) reste disponible sous macOS (déprécié par Apple mais fourni) ; les fonctionnalités `wayland` et
  `x11` d'eframe ne s'appliquent qu'à Linux.

## Ce qui manque

- **Langue du système** : sous macOS, `system_language` (`language.rs:84`) lit `LC_ALL`, `LC_MESSAGES`, `LANG`, comme Linux
  (spec 076, hors périmètre « macOS »). Lancé depuis le Finder ou le Dock, un programme ne reçoit pas ces variables, et depuis
  le Terminal, `LANG` suit la **région**, pas la **langue** de l'interface : un Mac en français peut donner `en_US.UTF-8`.
  macOS range la langue de l'interface dans la liste des langues préférées de l'utilisateur
  (`CFLocaleCopyPreferredLanguages`, CoreFoundation, toujours présent).
- **Logs** : `platform::user_base_dir` (`logging/platform.rs:16`) prend hors Windows `BaseDirs::state_dir()`, qui vaut
  `None` sous macOS (le dossier d'état est propre à XDG) : hors `cargo run` et sans `ARCANUM_LOG_DIR`, **aucun log n'est
  écrit** (`choose_logs_dir` rend `None`).
- **Libellé des touches** : `shortcut.key.ctrl` donne « Ctrl+Z » partout, alors que la touche à presser sous macOS est Cmd.
- **CI** : Linux et Windows seulement.
- **Docs** : ADR 0004 (« macOS n'est pas visé »), `CLAUDE.md` (plateformes), `README.md`, `docs/ci.md`, `docs/raccourcis.md`.

## Modèle

Aucun terme de jeu nouveau. Aucune donnée nouvelle dans l'état ni dans la vue.

Principe de découpage, celui de la spec 076 pour Windows : **une fonction pure, compilée et testée sur tous les systèmes,
fait le travail** (`windows_language_id_to_code` en est le modèle) ; **seul le branchement** (appel à CoreFoundation, choix au
lancement) est derrière `#[cfg(target_os = "macos")]`, dans un module dédié (ADR 0004). Les tests rouges se voient donc dès
Linux, sauf ceux qui interrogent le vrai système (C1 à C3, C5), rouges ou verts dans la CI macOS seulement.

## Décisions proposées

- **Langue du système, macOS** : la **première** langue préférée de l'utilisateur, traduite en code par `locale_to_code`
  (spec 076 : `fr-FR` → `fr`, `zh-Hans-CN` → `zh`, `en` → `en`). Seule la première compte, comme Windows ne donne qu'une
  langue d'interface : si elle n'a pas de fichier, l'anglais s'applique (spec 076), sans chercher une deuxième préférée.
  Si la liste est vide ou illisible, repli sur les variables `LC_ALL`, `LC_MESSAGES`, `LANG` (`unix_system_language`).
  L'ordre de la spec 076 et de la spec 080 est inchangé : `--lang`, puis préférences, puis système, puis anglais.
- **Lecture de la liste** : appel direct à CoreFoundation (`CFLocaleCopyPreferredLanguages`, `CFArrayGetCount`,
  `CFArrayGetValueAtIndex`, `CFStringGetCString`, `CFRelease`), lié par `#[link(name = "CoreFoundation", kind =
  "framework")]`, comme `GetUserDefaultUILanguage` l'est à kernel32 : aucune dépendance nouvelle (Q1).
- **Logs, macOS** : `~/Library/Logs/iter-arcanum/` (Q5). C'est le dossier qu'Apple prévoit pour les logs d'une application,
  que l'app Console affiche d'elle-même. Les autres sources de la spec 003 (variable `ARCANUM_LOG_DIR`, dossier `logs/` du dépôt
  sous `cargo run`) sont inchangées.
- **Libellé des touches, macOS** : « Cmd+{key} » au lieu de « Ctrl+{key} », en français comme en anglais (Q4). Le symbole ⌘
  n'est pas proposé : rien ne garantit qu'une police par défaut d'egui le dessine (gabarit, Libellés).
- **CI** : macOS (`macos-latest`, Apple Silicon) rejoint Linux et Windows dans la matrice de `ci.yml`, sur les mêmes
  déclencheurs (Q2). Sur un dépôt public, les minutes des runners standard sont gratuites, macOS compris (spec 082, Q6) : le
  coût n'est pas l'argent mais l'attente, et les trois jobs tournent en parallèle. Pas de paquet système à installer
  (CoreAudio est fourni). Le cache suit la même règle (enregistré par `main` seulement).

## Comportements attendus

Chaque cas devient au moins un test (nom indicatif entre parenthèses). Les cas marqués « macOS » ne compilent que sous
`#[cfg(target_os = "macos")]` et ne tournent que dans la CI macOS.

**Compilation et suite existante**

- **C1** — macOS : `cargo clippy --workspace --all-targets -- -D warnings` et `cargo test --workspace` passent dans la CI
  macOS, toute la suite existante comprise (pas de test nouveau : c'est le job lui-même). Un test qui échoue seulement sous
  macOS est corrigé dans ce fil, pas désactivé ; s'il révèle un besoin hors de cette spec, il devient une issue et la spec
  est mise à jour d'abord.

**Langue du système**

- **C2** — macOS : la lecture des langues préférées rend une valeur non vide sur le runner (`macos_preferred_language_is_read`).
  Le runner est en anglais : on vérifie que la lecture aboutit, pas une langue précise.
- **C3** — macOS : la langue du système passe par les langues préférées et non par `LANG` : le branchement macOS, appelé
  avec une lecture des variables qui donne `LANG=fr_FR.UTF-8` (injectée comme `unix_system_language`, sans `set_var`), rend
  `en` sur le runner en anglais (`macos_system_language_ignores_lang_variable`).
- **C4** — Pure, tous systèmes : la fonction de choix (`apple_system_language`, nom indicatif) reçoit la première langue
  préférée et la lecture des variables :
  - `Some("fr-FR")` → `fr`, `Some("en-GB")` → `en`, `Some("zh-Hans-CN")` → `zh` (`apple_language_reads_first_preferred`) ;
  - `Some("de-DE")` → `de`, que `launch_catalog` remplace par l'anglais faute de `de.json`, comme aujourd'hui (spec 076, test
    `system_language_without_file_falls_back_to_english`) ;
  - `None` ou `Some("")` avec `LANG=fr_FR.UTF-8` → `fr` (`apple_language_falls_back_to_locale_variables`) ;
  - `None` sans variable → `None` (`apple_language_without_anything_is_none`).

**Dossiers**

- **C5** — macOS : `user_data_dir()` et `user_config_dir()` finissent par `Library/Application Support/iter-arcanum`
  (`macos_user_dirs_in_application_support`) ; le dossier des logs de l'utilisateur finit par `Library/Logs/iter-arcanum`
  (`macos_user_logs_dir_in_library_logs`).
- **C6** — Pure, tous systèmes : la forme du dossier des logs macOS depuis le dossier personnel (`macos_logs_dir(home)`, nom
  indicatif) donne `home/Library/Logs/iter-arcanum` (`macos_logs_dir_from_home`), et `choose_logs_dir` avec cette base, sans
  variable ni `cargo run`, retient la source « dossier utilisateur » (`macos_logs_dir_is_user_choice`). Les chemins sont
  comparés composant par composant (`PathBuf::join`), sans séparateur en dur (ADR 0004).

**Libellé des touches**

- **C7** — Pure, tous systèmes : le libellé d'une touche avec modificateur pour macOS est « Cmd+Z » (fr et en), « Cmd+, »
  pour les préférences, « Cmd+Q » pour Quitter ; pour Windows et Linux il reste « Ctrl+Z » (`mac_shortcut_label_reads_cmd`,
  `other_shortcut_label_reads_ctrl`). La plateforme est un paramètre de la fonction pure (variante de `key_label`) ; le jeu
  passe celle de la compilation.
- **C8** — Les touches sans modificateur (Échap, Fin, Entrée, O, I, E, chiffres) ont le même libellé partout
  (`plain_keys_label_same_on_mac`).
- **C9** — Les infobulles et le Menu reprennent ce libellé : sous macOS, l'infobulle de Revenir en arrière finit par
  « Raccourci : Cmd+Z » (même test que C7, par `hover_text`).

**Tests existants modifiés** (relevés par `grep -rn 'Ctrl+' crates/*/tests`) : ils attendent « Ctrl+… » en dur et
échoueraient sous macOS. Ils comparent désormais au libellé de la plateforme de compilation, par un petit utilitaire de
`tests/support/` (`ctrl("Z")` → « Ctrl+Z » ou « Cmd+Z ») :

- `barre_de_menu.rs` (7 lignes : 83 à 95, 404) ;
- `sauvegarde_ui.rs` (7 lignes : 330 à 373) ;
- `preferences.rs` (5 lignes, dont 291, 293, 533) ;
- `icones.rs:669`, `nouvelle_partie.rs`, `joueur_fantome.rs` (1 ligne chacun).

Sous Linux et Windows, leur valeur attendue ne change pas.

## Cas limites et refus

- **L1** — La liste des langues préférées contient une valeur sans tiret (`fr`) : code `fr` (C4, `locale_to_code`).
- **L2** — La lecture CoreFoundation échoue (pointeur nul, chaîne non UTF-8 ou plus longue que le tampon) : `None`, puis
  repli sur les variables (C4) ; une ligne d'info dans les logs, comme pour Windows. Pas de panique, pas d'`unwrap()`.
- **L3** — Le dossier personnel est introuvable (`BaseDirs::new()` rend `None`) : pas de dossier de logs utilisateur, comme
  aujourd'hui sous Linux sans `HOME` (spec 003).
- **L4** — `install_desktop_file` (le `.desktop`, spec 004) reste Linux seulement (`#[cfg(target_os = "linux")]`,
  `main.rs:75`) ; l'icône intégrée à l'exécutable (`build.rs`) reste Windows seulement. Rien de nouveau sous macOS.

## Information cachée

Sans objet : rien ne change dans la vue du joueur.

## Libellés (i18n)

| Clé | fr | en |
|---|---|---|
| `shortcut.key.cmd` | `Cmd+{key}` | `Cmd+{key}` |

Clé réutilisée, vérifiée dans `assets/i18n/fr.json` (ligne 1539) : `shortcut.key.ctrl` = `Ctrl+{key}`, avec `{key}` = `Z`
donne « Ctrl+Z » (fr et en). `shortcut.key` est un groupe : `cmd` s'y ajoute à côté de `ctrl`, sans clé fille.

## Interface

- **Touches affichées** « Cmd+… » sous macOS dans le Menu, les infobulles et les préférences (C7, C9) ; inchangées ailleurs.
- **Dock** : icône du jeu quand on le lance depuis le dépôt (vérification à l'œil, Q3).
- **Langue** de l'écran de lancement selon la langue préférée de macOS (C3 ; vérification à l'œil, Q3).

Pas de partie préparée : `scripts/verif-manuelle.sh` est Linux seulement (Xvfb).

### Vérification à l'œil sans Mac (Q3)

Proposition : un workflow lancé à la main, `.github/workflows/capture-macos.yml` (onglet Actions, sur une branche), qui
compile le jeu en release sur `macos-latest`, règle la langue préférée du runner (`defaults write -g AppleLanguages -array
fr-FR`, ou `en-US` selon un champ du formulaire), lance `target/release/iter-arcanum` sans `--lang` ni variable de dossier,
attend une vingtaine de secondes, prend une capture d'écran (`screencapture -x`), puis publie en artefact la capture, le
contenu de `~/Library/Logs/iter-arcanum/` et la liste de `~/Library/Application Support/iter-arcanum/`. Il montre d'un coup
la fenêtre, l'icône dans le Dock, la langue détectée et les dossiers créés. Il ne tourne jamais tout seul (aucun coût hors
d'un clic). Limite connue : si le runner refuse l'enregistrement de l'écran, la capture sort noire ; les logs et les dossiers
restent probants, et le constat va dans la PR.

### Raccourcis clavier

Aucune touche ajoutée ni modifiée : seul le libellé du modificateur change sous macOS. `docs/raccourcis.md` le dit en tête
(« Ctrl, Cmd sous macOS »). F11 (plein écran) et Fin (fin du tour), moins commodes sur un clavier de Mac, restent : Q4.

## Docs à mettre à jour (dans la PR d'implémentation)

- **ADR 0004** : macOS devient cible, à égalité ; ses dossiers (Application Support, Library/Logs) ; la CI à trois systèmes.
  Le passage « Conséquences » sur le filtre de Windows, périmé depuis la spec 082, est corrigé au passage.
- **`CLAUDE.md`** (Le projet, Plateformes ; Commandes : rien à installer sous macOS), **`README.md`** (macOS dans les
  systèmes, dossier des préférences `~/Library/Application Support/iter-arcanum/`), **`docs/ci.md`** (job macOS),
  **`docs/raccourcis.md`** (Cmd), **`docs/architecture.md`** (module macOS de la langue).
- Specs 076, 080, 081 et 082 : leur renvoi « spec 085 » devient « spec #7 ».

## Hors périmètre

- **Paquet `.app`, icône `.icns`, `.dmg`, signature et notarisation Apple, binaire universel (Intel + Apple Silicon)** :
  issue #33 (mécanisme de release). Avec eux, la recherche des ressources dans `Contents/Resources/` du paquet.
  La CI ne vérifie qu'Apple Silicon (`macos-latest`) ; Intel n'est pas testé.
- **Scripts du dépôt sous macOS** (`scripts/verifier.sh`, `verif-manuelle.sh`, hooks) : personne ne développe sur Mac ; le
  bash 3.2 de macOS n'est pas visé.
- **Menu natif de macOS** (barre en haut de l'écran) : le jeu garde sa barre de Menu egui (spec 074).
- **Rendu Metal (wgpu)** au lieu d'OpenGL : seulement si glow pose problème sous macOS.

## Questions ouvertes

- **Q1 — Lire la langue de macOS : appel direct ou crate ?** Recommandé : appel direct à CoreFoundation (une trentaine de
  lignes `unsafe` commentées, aucune dépendance), sur le modèle de kernel32 pour Windows (spec 076). Autre choix : la crate
  `sys-locale` (MIT ou Apache-2.0, compatible GPL v3), qui couvre les trois systèmes et remplacerait aussi le code Windows,
  au prix d'une dépendance.
- **Q2 — Quand lancer la CI macOS ?** Recommandé : à chaque PR prête et à chaque fusion sur `main`, comme Linux et Windows
  (gratuit sur un dépôt public, en parallèle des deux autres ; la CI de `main` dure aujourd'hui 6 à 8 minutes). Autres choix :
  sur `main` et à la main seulement (une régression macOS ne se voit qu'après fusion) ; ou en PR seulement quand elle touche
  `arcanum-ui`, `arcanum-audio`, `Cargo.*` ou `.github/` (retour au filtre de chemins retiré par la spec 082).
- **Q3 — Vérifier à l'œil sans Mac : le workflow de capture ?** Recommandé : oui, `capture-macos.yml` lancé à la main
  (section Interface), une fois à l'implémentation, puis quand l'écran de lancement ou les dossiers changent. Autre choix :
  pas de capture, les tests de la CI suffisent, et un premier joueur sur Mac fera le retour.
- **Q4 — Touches sous macOS : seulement le libellé « Cmd+… » ?** Recommandé : oui, aucune touche ajoutée. F11 est pris par
  macOS (afficher le bureau) mais le bouton vert de la fenêtre et Menu > Plein écran restent ; Fin s'obtient par Fn+→ sur un
  portable. Autre choix : ajouter Ctrl+Cmd+F (plein écran habituel sous macOS) et une seconde touche de fin du tour.
- **Q5 — Logs sous macOS : `~/Library/Logs/iter-arcanum/` ?** Recommandé : oui (convention Apple, visible dans l'app
  Console). Autre choix : `~/Library/Application Support/iter-arcanum/logs/`, à côté des préférences et des sauvegardes.
- **Q6 — Partage avec l'issue #33 : d'accord ?** Cette spec s'arrête au jeu compilé depuis le dépôt et vérifié par la CI ; le
  paquet `.app`, la signature, le binaire universel et la recherche des ressources dans le paquet vont à #33. Autre choix :
  produire dès ici un `.app` non signé (artefact du workflow de capture), ce qui avance une partie de #33.
