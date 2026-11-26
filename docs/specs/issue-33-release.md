# #33 — Mécanisme de release

- Statut : validée (Mathieu, 2026-11-25 : Q1, Q2 et Q4 comme recommandé ; Q3 : « (prototype) » retiré du titre)
- Issue : #33 (Epic #31 Finitions v0.1.0, milestone v0.1.0) ; décisions D1 à D10 de Mathieu (2026-11-25) dans l'issue
- Dépend de : spec #7 (macOS, fusionnée : PR #45), spec 080 (fenêtre des préférences, modèle de la fenêtre « À propos »),
  spec 074 (Menu), spec 002 (recherche des ressources), spec 004 (icône), spec 082 (dépôt public, garde-fou)
- Bloque : #49 (première livraison : README, tag `v0.1.0-rc.1`, essai des binaires, tag `v0.1.0`)
- Livret : aucun (spec technique)
- Crates touchées : arcanum-ui (fenêtre « À propos », Menu, recherche des ressources, `build.rs`, `main.rs`) ;
  `.github/`, `scripts/`, `packaging/`, `deny.toml`, docs

## Objectif

Poser un tag `vX.Y.Z` sur `main` suffit pour que la CI vérifie le code, construise le jeu pour Windows, Linux et macOS,
l'empaquette avec ses ressources et ses licences, et prépare une release GitHub avec ses notes. Dans le jeu, une fenêtre
« À propos… » du Menu dit quelle version tourne. La procédure (release, correctif) est écrite dans `docs/release.md`.

Le mécanisme sert à chaque version ; sa première utilisation (README, tag de test, essai des binaires, tag `v0.1.0`) est
l'issue #49.

## Règles

Aucune règle du jeu. Les intentions et les décisions sont celles de l'issue #33 (D1 à D10), que la spec ne repose pas ;
elle dit comment les tenir et comment les vérifier.

## Modèle

Aucun terme de jeu, rien dans l'état ni dans la vue de la partie. Vocabulaire de la spec (pas dans le glossaire, qui ne
porte que les termes du jeu) :

- **release** : une version publiée sur GitHub (page « Releases »), avec ses archives ;
- **tag de test** : `vX.Y.Z-rc.N`, publié en « pre-release » (D3) ;
- **archive** : le fichier téléchargeable d'un système (`.zip`, `.tar.gz`, `.dmg`) ;
- **essai à blanc** : le workflow construit et vérifie les archives sans rien publier (artefacts de l'exécution).

### Vue d'ensemble

```
tag v0.1.0 poussé sur main
        │
        ▼
release.yml ─┬─ contrôles : tag = version de Cargo.toml (E1) · cargo deny check (E2)
             ├─ ci.yml réutilisé : fmt, clippy, test sur les trois OS (E3)
             ├─ construction et empaquetage par OS (scripts/empaqueter.sh)
             │     windows-latest → iter-arcanum-0.1.0-windows-x86_64.zip   (E5, E6)
             │     ubuntu-22.04  → iter-arcanum-0.1.0-linux-x86_64.tar.gz  (E5)
             │     macos-latest  → iter-arcanum-0.1.0-macos-universal.dmg  (E5, E7)
             ├─ essai de lancement de chaque archive dépaquetée (E8)
             └─ publication : SHA256SUMS.txt + gh release create --generate-notes
                  v0.1.0 → brouillon que Mathieu relit et publie ; v0.1.0-rc.1 → pre-release (E10)
```

### Version et commit dans le jeu (D2, D4)

- La version affichée est `CARGO_PKG_VERSION` du workspace : `0.1.0` pour une release, `0.2.0-dev` entre deux (D2).
- Le commit vient de `build.rs`, qui le passe au code par `cargo:rustc-env=ARCANUM_COMMIT=…` : `GITHUB_SHA` s'il est défini
  (CI), sinon `git rev-parse HEAD` ; rien si les deux échouent (archive des sources, pas de git). Le code le lit par
  `option_env!("ARCANUM_COMMIT")` et n'en garde que les 7 premiers caractères s'ils sont hexadécimaux (C2).
- `build.rs` se relance quand `GITHUB_SHA` change et quand `.git/HEAD` ou la référence qu'il désigne change : en
  développement, `arcanum-ui` se recompile à chaque commit (édition de liens seulement). Un `cargo build` lancé sur des
  fichiers modifiés mais pas encore commités affiche le commit précédent : pas de marque « modifié », inutile pour une
  release, construite par la CI depuis le tag.

### Recherche des ressources (spec 002, C14)

`assets_candidates` ajoute, juste après `<exe>/assets`, le candidat `<exe>/../Resources/assets` : c'est là que le paquet
`.app` range les ressources (`Iter Arcanum.app/Contents/MacOS/iter-arcanum` → `Contents/Resources/assets`). Le candidat est
ajouté sur les trois systèmes, sans `#[cfg]` : il n'existe pas hors d'un `.app` et ne coûte qu'un test d'existence de
fichier, et la fonction reste pure et testée partout (ADR 0004).

Ordre : `ARCANUM_ASSETS_DIR`, `<exe>/assets`, `<exe>/../Resources/assets`, dépôt.

### Workflow `.github/workflows/release.yml`

Déclencheurs :

- `push` d'un tag `v*` : release ;
- `workflow_dispatch` (onglet Actions, depuis `main`) : essai à blanc ;
- `pull_request` qui touche l'empaquetage (`release.yml`, `scripts/empaqueter.sh`, `packaging/**`, `crates/arcanum-ui/build.rs`),
  PR prête seulement : essai à blanc. Sans lui, la PR d'implémentation ne pourrait pas essayer le workflow, GitHub
  refusant `workflow_dispatch` tant que le fichier n'est pas sur `main` (constaté avec `capture-macos.yml`, spec #7) (D11).

Jobs :

1. **`controles`** (Linux) : version du tag = version de `Cargo.toml` (`v0.1.0` exige `version = "0.1.0"`, `v0.1.0-rc.2`
   aussi : le suffixe `-rc.N` ne se reporte pas dans `Cargo.toml`) ; refus d'un tag sur une version `-dev`. Puis
   `cargo deny check` complet (D5). En essai à blanc, pas de tag : la version est lue dans `Cargo.toml`, rien n'est refusé
   pour elle.
2. **`verifier`** : `ci.yml` appelé tel quel (`workflow_call`, ajouté à ses déclencheurs), pour ne pas recopier `fmt`,
   `clippy` et `test` sur les trois OS.
3. **`construire`** (matrice des trois OS, après les deux premiers) : `cargo build -p arcanum-ui --release` pour la ou les
   cibles de l'OS, `scripts/empaqueter.sh`, contrôles propres à l'OS, essai de lancement, archive en artefact.
4. **`publier`** (Linux, tag seulement) : télécharge les archives, écrit `SHA256SUMS.txt`, `gh release create` (E10).

Permissions : `contents: read` partout, `contents: write` pour le seul job `publier`. Actions tierces épinglées par SHA,
actions `actions/*` par version majeure, comme dans `ci.yml`. Pas de cache enregistré (le cache reste à `main`, `docs/ci.md`).

### Cibles (tableau de l'issue, section 4)

| OS | Runner | Construction | Contrôle propre à l'OS |
|---|---|---|---|
| Windows 10 et 11 | `windows-latest` | `x86_64-pc-windows-msvc`, `RUSTFLAGS=-C target-feature=+crt-static` posé par le workflow (pas dans `.cargo/config.toml` : le développement n'en a pas besoin) | `dumpbin /dependents` ne cite pas `VCRUNTIME140.dll` (E6) |
| Linux | `ubuntu-22.04` | `x86_64-unknown-linux-gnu` (glibc 2.35) | aucun : le runner fixe la glibc |
| macOS 11 et plus | `macos-latest` | `aarch64-apple-darwin` et `x86_64-apple-darwin`, assemblés par `lipo -create` (D7) ; `MACOSX_DEPLOYMENT_TARGET=11.0` | `lipo -archs` donne `x86_64 arm64` (E7) |

- **Console Windows** : `main.rs` déclare `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`. Les logs
  vont déjà dans un fichier (spec 003) ; la sortie d'erreur, perdue en release sous Windows, ne portait qu'une copie.
- **Icône macOS** : `iter-arcanum.icns` produit dans le workflow, jamais commité : l'exemple `icone` (spec 004) gagne une
  option `--iconset <dossier>` qui écrit les PNG d'un `.iconset` (16 à 1024 px, `@2x` compris) depuis le SVG, puis
  `iconutil -c icns`. L'`.ico` de Windows reste commité et embarqué par `build.rs`, comme aujourd'hui.
- **`Info.plist`** : modèle `packaging/macos/Info.plist`, où `empaqueter.sh` remplace la version. Clés : `CFBundleName` et
  `CFBundleDisplayName` « Iter Arcanum », `CFBundleIdentifier` `io.github.trollou.iter-arcanum` (D12), `CFBundleExecutable`
  `iter-arcanum`, `CFBundleIconFile` `iter-arcanum`, `CFBundleShortVersionString` et `CFBundleVersion` = `X.Y.Z`,
  `CFBundlePackageType` `APPL`, `LSMinimumSystemVersion` `11.0`, `NSHighResolutionCapable` vrai.
- **Signature** : `codesign --force --deep -s - "Iter Arcanum.app"` (ad hoc, D6), sans quoi un binaire universel ne se lance
  pas sur Apple Silicon.

### Script `scripts/empaqueter.sh`

`scripts/empaqueter.sh <windows|linux|macos> <version> <dossier de sortie>`, en bash (Git Bash sur le runner Windows),
lancé depuis la racine du dépôt après la construction. Le même script sert dans la CI et en local (Linux dans un fil cloud).

| OS | Archive | Contenu |
|---|---|---|
| Windows | `iter-arcanum-<v>-windows-x86_64.zip` (`7z`, présent sur le runner) | dossier `iter-arcanum-<v>/` : `iter-arcanum.exe`, `assets/`, `LICENSE`, `NOTICE`, `README.md` |
| Linux | `iter-arcanum-<v>-linux-x86_64.tar.gz` (bit d'exécution gardé) | dossier `iter-arcanum-<v>/` : `iter-arcanum`, `assets/`, `LICENSE`, `NOTICE`, `README.md` |
| macOS | `iter-arcanum-<v>-macos-universal.dmg` (`hdiutil create -format UDZO`, D8) | volume « Iter Arcanum <v> » : `Iter Arcanum.app` (exécutable dans `Contents/MacOS/`, `assets/` dans `Contents/Resources/`, icône, `Info.plist`), `LICENSE`, `NOTICE`, `README.md` à côté, et un lien vers `/Applications` |

`assets/` est copié en entier depuis le dépôt (`git ls-files assets` : seuls les fichiers suivis, donc déjà passés par le
garde-fou `depot_public.rs`), licences comprises (`assets/LICENSE`, `assets/sounds/LICENSES.md`). Rien de `../mage-knight/`.

### Notes de version (section 6 de l'issue)

`gh release create --generate-notes` liste les PR fusionnées depuis le tag précédent. `.github/release.yml` les range en deux
groupes : « Bilans » (label `bilan`) puis « Changements » (tout le reste). Le classement lit les labels **des PR** : le fil
qui ouvre la PR d'une issue `bilan` lui met aussi ce label (règle ajoutée à `docs/issues.md`). Mathieu écrit le résumé en
tête sur le brouillon (D3).

## Comportements attendus

Deux familles de cas. Les cas **C** sont du comportement du jeu : chacun devient un test `cargo test`, rouge avant le code
(nom indicatif entre parenthèses). Les cas **E** sont ceux du workflow et du script : ils ne se testent pas par
`cargo test` ; chacun est une étape du workflow qui échoue si le cas n'est pas tenu, vérifiée par un essai à blanc dans la
PR d'implémentation (D11), puis par le tag de test de #49. Leur vérification est notée dans la PR (section « Essai à blanc » :
lien de l'exécution, archives produites, ou l'étape rouge quand le cas est un refus).

### Version et fenêtre « À propos »

- **C1** — `version_label("0.1.0", Some("a1b2c3d"))` donne « 0.1.0 (a1b2c3d) » ; `version_label("0.2.0-dev", None)` donne
  « 0.2.0-dev » (`version_label_adds_short_commit`, `version_label_without_commit_is_bare_version`). D4.
- **C2** — `short_commit` garde les 7 premiers caractères d'un SHA complet (`a1b2c3d4e5f6…` → `a1b2c3d`), garde tel quel un
  SHA court de 7 caractères, et rend `None` pour une valeur absente, vide ou non hexadécimale (`short_commit_*`).
- **C3** — Le Menu montre « À propos… » juste après « Préférences… », avant le séparateur et « Quitter » ; sans touche ni
  description, toujours actif, y compris en combat et derrière la fenêtre de nouvelle partie, comme « Préférences… »
  (spec 080 C14) (`menu_lists_about_after_preferences`, `about_entry_is_always_enabled`).
- **C4** — `Command::OpenAbout` ouvre la fenêtre ; `about_window(catalog)` la rend traduite (`AboutWindow`) : titre
  « À propos », nom « Iter Arcanum », ligne « Version 0.1.0 (a1b2c3d) » (C1, avec la version de compilation), deux lignes de
  licence, la mention de projet non officiel, le lien vers le dépôt (libellé « Code source », adresse
  `https://github.com/trollou/iter-arcanum`, constante du code, non traduite), la ligne des crédits des sons, le bouton
  « Fermer » (`about_window_lists_version_licenses_and_links`). Le test compare la ligne de version à
  `version_label(env!("CARGO_PKG_VERSION"), short_commit(option_env!("ARCANUM_COMMIT")))`, pas à une chaîne fixe.
- **C5** — « Fermer », Échap et la croix ferment la fenêtre ; `OpenAbout` une seconde fois la referme, comme
  `OpenPreferences` (`escape_closes_about`, `about_command_toggles_window`).
- **C6** — Fenêtre ouverte, la partie, l'écran de lancement et leurs touches n'agissent pas ; Ctrl+Q, F11 et Ctrl+, agissent,
  comme derrière les préférences (spec 080 L4) (`about_window_blocks_game_commands`).
- **C7** — Une seule des deux fenêtres à la fois : ouvrir « À propos » ferme les préférences et inversement ; Échap ferme
  celle qui est ouverte (`about_and_preferences_are_exclusive`).
- **C11** — Le titre de la fenêtre du jeu est « Iter Arcanum », sans « (prototype) » (D13) : **test existant modifié**,
  `boucle_et_fenetre.rs:262` (`screen.title`).
- **C8** — En anglais, la fenêtre a ses libellés (`about_window_in_english`) ; la parité des clés `about.*` entre `fr.json`
  et `en.json` est déjà gardée par `all_languages_have_the_same_keys` (`arcanum-i18n/tests/fichiers_de_langue.rs:57`).

### Recherche des ressources

- **C9** — `assets_candidates(Some(env), Some(bin))` rend `[env, bin/assets, bin/../Resources/assets, dépôt]`, et sans
  variable `[bin/assets, bin/../Resources/assets, dépôt]` (`assets_candidates_are_in_priority_order`, **test existant modifié**,
  `boucle_et_fenetre.rs:446`, spec 002 C14).
- **C10** — Dans un dossier temporaire organisé comme un `.app` (`Iter Arcanum.app/Contents/MacOS/` et
  `Contents/Resources/assets/i18n/fr.json`), `find_assets_dir(assets_candidates(None, Some(<…>/Contents/MacOS)))` rend
  `Contents/Resources/assets` (`assets_are_found_in_app_bundle_resources`).

### Workflow et empaquetage

- **E1** — Un tag dont la version diffère de `Cargo.toml` (`v0.1.1` avec `0.1.0`), ou posé sur une version `-dev`, arrête le
  workflow au job `controles`, avant toute construction, avec un message qui dit les deux versions. Vérifié en local
  (fonction bash du job lancée sur les deux cas) et noté dans la PR, sans pousser de faux tag.
- **E2** — `cargo deny check` (licences, interdits, sources, avis de sécurité) est bloquant (D5), et passe : l'avis
  `RUSTSEC-2026-0543` (`chrono` non maintenu) est ignoré dans `deny.toml` avec sa justification et le renvoi à l'issue #55 (D14). `deny.toml` déclare les
  cibles macOS (`aarch64-apple-darwin`, `x86_64-apple-darwin`) en plus de Linux et Windows : sans elles, les dépendances
  propres à macOS (CoreFoundation, objc2) ne sont pas vérifiées. Essai pendant la rédaction : voir « Essai de faisabilité ».
- **E3** — `fmt`, `clippy` et `test` tournent sur les trois OS par `ci.yml` appelé ; un échec arrête la release.
- **E4** — Les archives portent le nom `iter-arcanum-<version>-<os>-<arch>` et le contenu du tableau du script ; une étape
  liste l'archive et échoue si un fichier attendu manque (exécutable, `assets/i18n/fr.json`, `assets/LICENSE`,
  `assets/sounds/LICENSES.md`, `LICENSE`, `NOTICE`, `README.md`).
- **E5** — Chaque archive se construit sur son runner (Windows, Ubuntu 22.04, macOS) ; l'essai à blanc publie les trois en
  artefacts.
- **E6** — L'exécutable Windows ne dépend pas de `VCRUNTIME140.dll` (`dumpbin /dependents`).
- **E7** — L'exécutable du `.app` contient les deux architectures (`lipo -archs` : `x86_64 arm64`) et `codesign --verify`
  accepte le paquet.
- **E8** — Essai de lancement : l'archive dépaquetée hors du dépôt, le jeu est lancé sans `cargo` ni variable de dossier
  (Linux sous Xvfb, macOS depuis le `.dmg` monté, Windows directement), arrêté au bout de 15 s ; son log contient
  « dossier des ressources : » suivi d'un chemin **dans** l'archive dépaquetée (`assets.rs:78`). Le candidat du dépôt, qui
  existe aussi sur le runner, ne doit pas être celui retenu.
- **E9** — Un essai à blanc (`workflow_dispatch` ou PR qui touche l'empaquetage) ne crée ni tag ni release : le job
  `publier` ne tourne pas.
- **E10** — Tag `vX.Y.Z-rc.N` : release publiée directement, marquée « pre-release ». Tag `vX.Y.Z` : release en
  **brouillon** (D3). Les deux portent les trois archives, `SHA256SUMS.txt` et les notes générées. Vérifié par #49
  (`v0.1.0-rc.1`, puis `v0.1.0`) : la spec ne pose aucun tag.

## Cas limites et refus

- **L1** — Compilation hors git (archive des sources) : pas de commit, la fenêtre montre « Version 0.1.0 » seule (C1), et la
  compilation n'échoue pas (`build.rs` ignore l'échec de `git`).
- **L2** — Tag déjà publié, relancé : `gh release create` échoue sur une release existante ; on supprime la release (pas le
  tag) dans GitHub avant de relancer (`docs/release.md`).
- **L3** — Tag hors de `main` ou de `release/X.Y` : non vérifié par le workflow ; la procédure de `docs/release.md` le
  prévient. Un contrôle automatique n'apporterait rien tant que Mathieu seul pose les tags.
- **L4** — Ressources introuvables dans une archive mal faite : E4 et E8 l'arrêtent avant publication ; dans le jeu, le repli
  existant s'applique (clés visibles, spec 002 L5).
- **L5** — `cargo deny` trouve un nouvel avis de sécurité le jour du tag : la release s'arrête (D5) ; mettre à jour la
  dépendance, ou ignorer l'avis dans `deny.toml` avec sa justification, par une PR.

## Essai de faisabilité (rédaction, 2026-11-25)

- `cargo deny check` sur `main` (cargo-deny 0.22.1), avec et sans les cibles macOS ajoutées à `deny.toml` : licences,
  interdits et sources **passent** ; avis de sécurité **échoue** sur un seul avis, `RUSTSEC-2026-0543` « chrono is
  unmaintained » (aucune faille connue, aucune version corrigée). `chrono` est une dépendance directe d'`arcanum-ui`
  (horodatage des logs, des sauvegardes et de la session : `logging/mod.rs`, `saves.rs`, `client.rs`, `session.rs`). Avec D5
  tel quel, le premier tag échouerait (D14).
- Le workflow lui-même ne peut pas s'essayer avant la PR d'implémentation (D11) ; `empaqueter.sh linux` s'essaie en local
  dans le fil d'implémentation.

## Information cachée

Aucune : la fenêtre « À propos » ne lit pas la partie. `AboutWindow` est construite par `ClientSession`, comme
`PreferencesWindow`, à partir du catalogue et des constantes de compilation.

## Libellés (i18n)

Nouvelles clés, groupe `about` (aucune clé fille sous un libellé ; aucun caractère hors des polices : « … » s'affiche déjà
dans « Préférences… ») :

| Clé | fr | en |
|---|---|---|
| `about.button` | À propos… | About… |
| `about.title` | À propos | About |
| `about.name` | Iter Arcanum | Iter Arcanum |
| `about.version` | Version {version} | Version {version} |
| `about.license_code` | Code sous licence GNU GPL v3 ou ultérieure | Code under the GNU GPL v3 or later |
| `about.license_assets` | Visuels et textes sous licence CC BY-SA 4.0 ; chaque son garde la licence de son auteur | Visuals and texts under CC BY-SA 4.0; each sound keeps its author's license |
| `about.unofficial` | Projet de fan non officiel, sans lien avec les éditeurs de Mage Knight. | Unofficial fan project, not affiliated with the publishers of Mage Knight. |
| `about.repository` | Code source | Source code |
| `about.sounds` | Crédits des sons : {file}, livré avec le jeu | Sound credits: {file}, shipped with the game |
| `about.close` | Fermer | Close |

Avec les paramètres prévus : `about.version`, `{version}` = « 0.1.0 (a1b2c3d) » → « Version 0.1.0 (a1b2c3d) » ;
`about.sounds`, `{file}` = `assets/sounds/LICENSES.md` (constante du code) → « Crédits des sons : assets/sounds/LICENSES.md,
livré avec le jeu ».

Clé modifiée (D13) : `app.title`, titre de la fenêtre du jeu, passe de « Iter Arcanum (prototype) » à « Iter Arcanum » en
fr et en en (relu dans `fr.json` et `en.json`, l. 6). `about.name` reste à part : le titre de la fenêtre pourrait un jour
porter autre chose que le nom.

## Interface

- **Menu** : « À propos… » entre « Préférences… » et le séparateur de « Quitter » (C3).
- **Fenêtre « À propos »** : centrée, au premier plan, non redimensionnable, comme celle des préférences ; nom en titre de
  section, ligne de version, licences, mention non officielle, lien « Code source » cliquable qui ouvre le navigateur
  (`egui::Hyperlink`), ligne des crédits, bouton « Fermer » (C4, `AboutWindow`) ; mise en page : vérification à l'œil.
- **Fermeture** par « Fermer », Échap, la croix (C5).
- **Console Windows** absente au double-clic d'un exécutable release : vérification à l'œil dans #49 (essai des binaires).
- **Icône du Dock et nom de l'application** sous macOS (`.icns`, `Info.plist`) : vérification par #49, au mieux par la capture
  macOS (`capture-macos.yml`, à étendre au `.app` si l'essai le demande, issue à ouvrir alors).

**Partie préparée** : aucune. La fenêtre s'ouvre depuis le Menu dès le lancement ; la vérification à l'œil passe par
`scripts/verif-manuelle.sh demarrer` puis Menu, « À propos… », en français puis en anglais.

### Raccourcis clavier

Aucun. « À propos… » se consulte rarement ; Échap ferme la fenêtre (comme les préférences, déjà dans `docs/raccourcis.md`,
ligne Échap à compléter).

## Tests existants modifiés

Cherchés là où c'est utilisé (`grep` des lecteurs du Menu et de `assets_candidates`) :

- `crates/arcanum-ui/tests/boucle_et_fenetre.rs` `assets_candidates_are_in_priority_order` : nouveau candidat (C9).
- `crates/arcanum-ui/tests/boucle_et_fenetre.rs:262` : `screen.title` vaut « Iter Arcanum » (C11, D13). Aucun script ne
  cherche la fenêtre par ce titre (`grep prototype` sur `scripts/`, vide).
- `crates/arcanum-ui/tests/barre_de_menu.rs` (liste des libellés et des commandes du Menu, l. 85-110) : « À propos… » et
  `Command::OpenAbout` ajoutés.
- `crates/arcanum-ui/tests/icones.rs` `bar_stays_during_combat` : `bar.items.len()` passe de 12 à 13.
- `crates/arcanum-ui/tests/preferences.rs` `menu_lists_preferences_before_quit` : la fin du Menu (5 dernières lignes) gagne
  « À propos… » entre « Préférences… » et le séparateur.

## Documentation

- `docs/release.md` (nouveau) : faire une release (version de `Cargo.toml`, tag de test, tag final, relire et publier le
  brouillon, passer à `X.Y+1.0-dev`, D2) ; faire un correctif (branche `release/X.Y` née du tag au premier correctif,
  cherry-pick, tag `X.Y.Z+1`, D1) ; relancer un tag (L2) ; un avis `cargo deny` (L5).
- `docs/ci.md` : `release.yml`, l'appel de `ci.yml`, l'essai à blanc.
- `docs/architecture.md` : fenêtre « À propos » (`AboutWindow`, `Command::OpenAbout`), commit de `build.rs`, candidat
  `Resources/assets`.
- `docs/issues.md` : label `bilan` reporté sur la PR (notes de version).
- `docs/raccourcis.md` : Échap ferme aussi « À propos ».
- `CLAUDE.md` : arborescence (`packaging/`), une ligne vers `docs/release.md`.

## Découpage de l'implémentation

Deux fils sur `feature/33-release` (issue #33, « Mise en œuvre ») :

1. **Empaquetage** : C9, C10 (rouges d'abord), `deny.toml` (cibles macOS, exception D14), `release.yml`, `ci.yml` appelable, `empaqueter.sh`,
   `packaging/macos/Info.plist`, option `--iconset`, `windows_subsystem`, `docs/release.md`, `docs/ci.md` ; essai à blanc
   vert dans la PR (E1 à E9).
2. **« À propos »** : C1 à C8 et C11 (rouges d'abord), `build.rs` (commit), libellés, fenêtre, vérification à l'œil, autres docs ;
   PR passée prête, bilan.

## Hors périmètre

Celui de l'issue #33 : migration des sauvegardes (#52), signature payante et notarisation (D6), AppImage, installateurs,
`.deb`, Flatpak, Snap, magasins (D9), mise à jour automatique, ressources embarquées dans l'exécutable, Linux et Windows ARM.
Aussi : le README et les contournements SmartScreen et Gatekeeper (#49), le contrôle de la branche d'un tag (L3).

## Décisions (Mathieu, 2026-11-25)

Les décisions D1 à D10 sont dans l'issue #33. Celles de la relecture de la spec suivent la même numérotation.

- **D11 (Q1) — Essai à blanc dans les PR** : oui. `release.yml` se lance aussi, sans rien publier, sur une PR prête qui touche
  l'empaquetage (chemins listés dans « Workflow ») ; une PR ordinaire ne le lance pas.
- **D12 (Q2) — Identifiant macOS** : `io.github.trollou.iter-arcanum`, définitif dès la première release.
- **D13 (Q3) — Titre de la fenêtre** : « (prototype) » retiré ; `app.title` devient « Iter Arcanum » (C11).
- **D14 (Q4) — `chrono` non maintenu** : avis `RUSTSEC-2026-0543` ignoré dans `deny.toml`, justifié (non maintenu, pas une
  faille) ; remplacement par `jiff` ou `time` dans l'issue #55 (Task, milestone v0.2.0, avant #52).

## Questions ouvertes

Aucune.
