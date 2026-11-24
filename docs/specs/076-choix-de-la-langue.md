# 076 — Choix de la langue

- Statut : implémentée (2026-11-13) ; validée le 2026-11-13 (Q1 à Q3 tranchées par Mathieu : réponses proposées retenues)
- Jalon : J5 (spec d'interface placée après les artefacts, Mathieu, 2026-11-12)
- Dépend de : 001 (libellés, `Catalog`, `available_languages`), 002 (dossier des ressources, replis L5 et L6), 003 (logs
  techniques) ; toutes implémentées
- Livret : aucune règle de jeu
- Crates touchées : arcanum-ui seulement (lancement et chargement du catalogue). Rien dans arcanum-core, arcanum-i18n ni `assets/i18n/` :
  aucun libellé nouveau. Aussi : `scripts/verif-manuelle.sh`, ADR 0003, README, `docs/verification-manuelle.md`,
  `docs/architecture.md`, glossaire
- Base : issue privée 195 (décisions de Mathieu en commentaire, 2026-11-11)

## Objectif

Le jeu s'affiche dans la langue du système quand il existe un fichier de libellés pour elle (aujourd'hui `fr` et `en`), et en
anglais sinon. Un paramètre facultatif au lancement, `--lang fr` ou `--lang en`, force la langue. Aujourd'hui le jeu ne charge
que `fr.json` (`french_catalog_or_empty`, `crates/arcanum-ui/src/assets.rs`), alors que `en.json` est complet et tenu cohérent par
test (spec 001, C12). Un écran de préférences remplacera plus tard le paramètre (spec 080).

## Règles

Aucune règle du livret. Décisions de l'issue privée 195 que la spec applique :

- **Langue du système, anglais par défaut** : une langue du système sans fichier de libellés donne l'anglais.
- **Paramètre `--lang`** facultatif, qui l'emporte sur la langue du système. Langue demandée sans fichier : **repli en
  anglais**, avec une erreur dans les logs.
- **Pas de dépendance nouvelle** : sous Windows, un appel direct à l'API du système derrière `#[cfg(windows)]` (Q1).
- **Logs en français**, quelle que soit la langue affichée (logs techniques, spec 003).
- Le **français reste la langue de référence** (ADR 0003, spec 001) : c'est lui qui désigne le dossier des ressources
  (`find_assets_dir` cherche `i18n/fr.json`, spec 002, inchangé) et lui que les tests de cohérence comparent. L'anglais n'est
  que la **langue de repli** à l'affichage.
- Le **contenu du jeu** (`crates/arcanum-core/content/*.ron`) ne porte que des clés, aucun texte : rien à faire de ce côté
  (vérifié à la rédaction : les noms et effets des cartes, ennemis et sites sont des clés `card.*`, `enemy.*`, `site.*`).

## Modèle

Termes ajoutés au glossaire (section arcanum-i18n, à côté de « Langue » et « Catalogue ») :

| Terme | Code | Sens |
|---|---|---|
| Langue du système | `system_language` | Langue de l'interface du système d'exploitation, lue au lancement. |
| Langue demandée | `--lang` | Langue forcée au lancement par le paramètre `--lang`. |
| Langue de repli | `FALLBACK_LANGUAGE` | L'anglais (`en`) : langue affichée quand ni la langue demandée ni celle du système n'ont de fichier. |
| Langue disponible | `available_languages` (spec 001) | Langue qui a un fichier `assets/i18n/<code>.json`. |

Rien dans l'état de la partie ni dans les sauvegardes : une sauvegarde ne contient aucun texte et se recharge dans n'importe
quelle langue.

### Ce que fait le lancement

```
arguments ──► langue demandée ? ──oui──► disponible ? ──oui──► elle
                    │                         └──non──► anglais + erreur dans les logs
                    non
                    ▼
            langue du système ? ──oui──► disponible ? ──oui──► elle
                    │                         └──non──► anglais (info dans les logs)
                    non (introuvable) ──────────────────────► anglais (info dans les logs)

puis : chargement du catalogue de la langue retenue ; s'il échoue, anglais ; s'il échoue aussi, catalogue vide (spec 002 L6)
```

- **Lire le paramètre** : `--lang <code>` ou `--lang=<code>`. Le code se lit comme une langue du système (ci-dessous) :
  `FR`, `fr_FR` et `fr-FR` donnent `fr`.
- **Langue du système, Linux** (et tout ce qui n'est pas Windows, en attendant la spec 085 pour macOS) : la première variable
  non vide parmi `LC_ALL`, `LC_MESSAGES`, `LANG`, dans cet ordre (ordre POSIX pour les messages). `LANGUAGE` (liste GNU) est
  ignorée (hors périmètre).
- **Langue du système, Windows** : la langue de l'interface de Windows, `GetUserDefaultUILanguage` (kernel32, toujours lié),
  qui rend un identifiant de langue (un entier de 16 bits) ; ses 10 bits de poids faible donnent la langue principale, traduite
  par une petite table : `0x0C` → `fr`, `0x09` → `en`. Une autre valeur n'a pas de code : l'anglais s'applique. Ajouter une
  langue au jeu ajoutera une ligne à cette table.
- **D'une valeur de locale à un code** : les lettres avant le premier `_`, `-`, `.` ou `@`, en minuscules (`fr_FR.UTF-8` →
  `fr`, `en-GB` → `en`, `de_DE@euro` → `de`). `C` et `POSIX` donnent `c` et `posix`, qui n'ont pas de fichier : anglais.
- **Langues disponibles** : les fichiers `*.json` de `assets/i18n/` (`arcanum_i18n::available_languages`, spec 001), pas une liste
  en dur : un `de.json` ajouté plus tard sera choisi sans toucher au code (sauf la table Windows).

## Actions et événements

Aucun : le choix se fait au lancement, avant la partie, hors du moteur.

## Comportements attendus

Chaque cas devient au moins un test (nom indicatif entre parenthèses). Les tests passent les arguments et les variables
d'environnement en paramètres (fonctions pures), sans toucher à l'environnement du processus : ils restent parallèles et
indépendants de la machine. Le dossier des ressources de test est celui du dépôt (`repository_assets_dir`), où `fr.json` et
`en.json` existent ; `language.name` (« Français », « English ») sert à reconnaître le catalogue chargé.

**Paramètre `--lang`**

- **C1** — Avec les arguments `--lang en`, le catalogue chargé est l'anglais (`language.name` = « English »), même si la
  langue du système est le français (`lang_argument_overrides_system_language`).
- **C2** — `--lang=fr` est lu comme `--lang fr` (`lang_argument_accepts_equals_form`).
- **C3** — `--lang FR`, `--lang fr_FR` et `--lang fr-FR` donnent le français (`lang_argument_is_read_like_a_locale`).
- **C4** — `--lang de` (pas de `de.json`) : l'anglais est chargé, et une ligne `error` des logs nomme la langue demandée et
  les langues disponibles, par exemple « langue demandée introuvable : de (disponibles : en, fr) ; repli en anglais »
  (`unknown_lang_argument_falls_back_to_english_with_error`).
- **C5** — `--lang` en dernier argument, sans valeur : une ligne `error` le signale et le paramètre est ignoré, la langue du
  système s'applique (`lang_argument_without_value_is_ignored`).
- **C6** — `--lang` donné deux fois (`--lang fr --lang en`) : le dernier l'emporte, avec un `warn` dans les logs
  (`last_lang_argument_wins`).
- **C7** — Un argument inconnu (`--plein-ecran`) est ignoré avec un `warn` dans les logs, et ne gêne pas la lecture de
  `--lang` (`unknown_argument_is_ignored_with_warning`).

**Langue du système**

- **C8** — Sans `--lang`, avec `LANG=fr_FR.UTF-8`, le français est chargé (`system_language_french_is_used`).
- **C9** — Sans `--lang`, avec `LANG=en_US.UTF-8`, l'anglais est chargé (`system_language_english_is_used`).
- **C10** — Sans `--lang`, avec `LANG=de_DE.UTF-8`, l'anglais est chargé ; une ligne `info` (pas `error` : ce n'est pas une
  faute) dit que la langue du système n'a pas de fichier (`system_language_without_file_falls_back_to_english`).
- **C11** — `LC_ALL` l'emporte sur `LC_MESSAGES`, qui l'emporte sur `LANG` ; une variable vide est sautée (`LC_ALL=""`,
  `LANG=fr_FR.UTF-8` donne `fr`) (`unix_locale_variables_priority`).
- **C12** — Aucune des trois variables, ou `LANG=C` : l'anglais est chargé, avec une ligne `info`
  (`no_system_language_falls_back_to_english`).
- **C13** — Table Windows (fonction pure, testée sous Linux comme sous Windows) : `0x040C` (français de France) et `0x0C0C`
  (français du Canada) donnent `fr`, `0x0409` et `0x0809` donnent `en`, `0x0407` (allemand) ne donne rien
  (`windows_language_id_to_code`).
- **C14** — Sous Windows seulement (`#[cfg(windows)]`, CI Windows) : la lecture de la langue de l'interface rend sans planter
  (`windows_system_language_is_read`). Pas d'assertion sur la valeur, qui dépend de la machine.

**Chargement et logs**

- **C15** — La langue retenue et sa raison sont écrites en `info` dans les logs, en français : « langue : en (paramètre
  --lang) », « langue : fr (langue du système) », « langue : en (repli) » (`chosen_language_is_logged`) — spec 003.
- **C16** — Le titre de la fenêtre se lit dans le catalogue retenu (`app.title` ; même texte « Iter Arcanum (prototype) » dans
  les deux langues, `fr.json` et `en.json`) : rien à tester de plus que C1 ; `main.rs` passe le catalogue retenu, comme
  aujourd'hui le français.

## Cas limites et refus

- **L1** — Le fichier de la langue retenue existe mais ne se charge pas (JSON invalide) : `error` dans les logs, l'anglais est
  chargé à la place ; si c'est l'anglais qui ne se charge pas, catalogue vide et clés visibles `⟦clé⟧`, comme la spec 002 L6
  aujourd'hui (`unloadable_language_falls_back_to_english_then_visible_keys`). Dossier de test : une fixture à côté de
  `crates/arcanum-ui/tests/` où `fr.json` est invalide et `en.json` valide.
- **L2** — Aucun dossier des ressources : inchangé, catalogue vide et clés visibles, quelle que soit la langue (spec 002 L5,
  test `missing_assets_dir_falls_back_to_visible_keys`, adapté au renommage ci-dessous).
- **L3** — Les logs restent en français dans toutes les langues : les messages de C4, C5, C6, C7, C10, C12 et C15 sont en
  dur dans le code, jamais tirés du catalogue (vérifié par leurs tests, qui lisent le texte français).

### Tests existants touchés

`load_french_catalog` et `french_catalog_or_empty` (`crates/arcanum-ui/src/assets.rs`) prennent la langue en paramètre et
changent de nom (`load_catalog`, `catalog_or_empty`, noms indicatifs). Leurs lecteurs, relevés par
`grep -rn "load_french_catalog\|french_catalog_or_empty" crates` : `crates/arcanum-ui/src/lib.rs` (réexport), `main.rs`,
`crates/arcanum-ui/tests/boucle_et_fenetre.rs` (tests `assets_dir_is_the_first_candidate_with_french`,
`repository_assets_dir_is_found`, `missing_assets_dir_falls_back_to_visible_keys`,
`unloadable_catalog_falls_back_to_visible_keys`) et `crates/arcanum-ui/tests/logs_techniques.rs` (quatre appels). Ils passent
le français explicitement : même comportement, seul l'appel change.

## Information cachée

Sans objet : aucune donnée de partie, la vue (`PlayerView`) ne change pas.

## Libellés (i18n)

Aucune clé nouvelle. `language.name` existe déjà (spec 001) : « Français » dans `fr.json`, « English » dans `en.json` ; les
tests s'en servent pour reconnaître le catalogue chargé.

## Interface

Aucun élément d'écran nouveau : tout l'écran passe dans la langue retenue, puisqu'il ne lit que le catalogue (ADR 0003).

- **Jeu en anglais** : écran de nouvelle partie, plateau et combat lancés avec `--lang en` (C1 ; vérification à l'œil, deux
  ou trois captures, pour repérer un libellé trop long en anglais qui déborderait).
- **Vérification à l'œil en français par défaut** : sous Xvfb la langue du système vaut souvent `C` ou `en`, ce qui
  passerait toutes les captures futures en anglais. `scripts/verif-manuelle.sh demarrer` (et `charger`) lance donc le jeu
  avec `--lang fr`, sauf si la variable `ARCANUM_VERIF_LANGUE` en donne une autre (`ARCANUM_VERIF_LANGUE=en`). Documenté dans
  `docs/verification-manuelle.md` (pas de test : script d'outillage).
- **README** : la phrase « The game interface is currently in French only » devient : langue du système, anglais par défaut,
  `--lang fr|en` pour la forcer (`cargo run -p arcanum-ui -- --lang en`).

### Raccourcis clavier

Aucun : le choix se fait au lancement.

## Documentation

- **ADR 0003** : une ligne dans « Décision » : la langue affichée est celle du système si elle a un fichier, sinon l'anglais ;
  le paramètre `--lang` la force (spec 076) ; le français reste la langue de référence.
- **`docs/architecture.md`** : le chargement du catalogue (ligne qui cite `Catalog::load(dossier, &Language("fr".into()))`)
  et le module de choix de la langue.
- **Glossaire** : les termes de la section Modèle.

## Hors périmètre

- **Écran de préférences** et langue mémorisée d'une session à l'autre : spec 080, qui reprend ce choix.
- **Changer de langue en cours de partie** : spec 080.
- **macOS** : sa détection propre (`#[cfg(target_os = "macos")]`) relève de la spec 085 ; d'ici là, macOS passe par les
  variables d'environnement, comme Linux.
- **Variable `LANGUAGE`** (liste de langues GNU, `fr:en`) : ignorée ; `LC_ALL`, `LC_MESSAGES` et `LANG` suffisent sur les
  bureaux Linux courants, qui posent `LANG`.
- **Variable d'environnement pour forcer la langue** (type `ARCANUM_LANG`) : pas demandée ; `--lang` suffit.
- **Relecture de la traduction anglaise** : `en.json` est déjà complet (1012 clés, mêmes clés que `fr.json`, test de la spec
  001) ; une coquille relevée pendant la vérification à l'œil devient une issue, pas une correction dans cette spec.
- **Texte des images** : les illustrations n'en ont pas (ADR 0009).

## Décisions

Validées par Mathieu le 2026-11-13 (« Ok avec défauts ») : les trois réponses proposées sont retenues.

- **Q1 — Premier `unsafe` du projet** : accepté. `GetUserDefaultUILanguage` est déclarée directement
  (`unsafe extern "system"`, édition 2024) dans un module `#[cfg(windows)]` de cinq lignes, avec un commentaire `SAFETY`
  qui justifie l'invariant (aucun argument, aucun pointeur, un entier en retour, kernel32 toujours chargé). Aucune dépendance
  (`windows-sys`, `sys-locale`) ; rien d'autre en `unsafe` dans le dépôt.
- **Q2 — Langue de l'interface plutôt que format régional sous Windows** : `GetUserDefaultUILanguage`, comme `LC_MESSAGES`
  sous Linux.
- **Q3 — `--lang` sans valeur** : la langue du système s'applique, avec l'erreur dans les logs (C5) ; seule une langue nommée
  et absente déclenche le repli en anglais (C4).

## Questions ouvertes

Aucune.
