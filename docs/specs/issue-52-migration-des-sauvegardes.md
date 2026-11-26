# #52 — Migration des sauvegardes

- Statut : à valider (reprise le 2026-11-26 : portée réduite aux trois dernières versions publiées)
- Issue : #52 (milestone v0.2.0)
- Dépend de : spec 024 (sauvegarde et retour en arrière), spec 038 (un générateur par usage), tag `v0.1.0` (#49), #55 (remplacer
  `chrono`, avant celle-ci)
- Livret : aucun (outillage, pas de règle du jeu)
- Crates touchées : arcanum-persistance (migration, contrôle, témoins), arcanum-ui (messages de refus), arcanum-core (commentaire
  de `EVENT_FORMAT_VERSION` seulement)

## Objectif

À partir de la v0.1.0, première version publique, une partie sauvegardée avec l'une des **trois dernières versions publiées**
se recharge avec la version courante (en 0.4.0 : celles de 0.1.0, 0.2.0 et 0.3.0), et c'est la seule garantie (décisions de
Mathieu, 2026-11-26). Le joueur ne voit rien : l'ancienne sauvegarde apparaît dans la liste et se charge comme une autre ;
resauvegardée, elle passe au format courant. Une sauvegarde plus ancienne, ou écrite par une version de développement, n'est pas
garantie. Côté développement, un test empêche de changer la forme d'une sauvegarde sans le décider, la liste de ce qui a changé
depuis chaque version gardée se lit dans un diff, et les sauvegardes témoins des versions gardées doivent se charger à chaque
`cargo test`.

## Constat

- Une sauvegarde (spec 024, `arcanum-persistance/src/lib.rs`) est un fichier JSON : `version`, résumé (`SaveSummary`),
  positions des générateurs (`rng_positions`, spec 038) et journal (`events`, chaque `Event` avec son numéro de décision). La
  configuration de départ est dans `GameStarted` ; le scénario y est recopié en entier (`config.rs`), si bien qu'un scénario
  modifié plus tard ne change pas une partie en cours.
- `load` lit `version` avant le reste et refuse tout numéro différent de `EVENT_FORMAT_VERSION` (1) par
  `LoadError::OtherVersion`, affiché « Sauvegarde d'une autre version du jeu. » (`load.error.other_version`, `controls.rs`).
  Une forme d'événement inconnue donne `LoadError::Invalid` (test `outdated_event_shape_is_refused`).
- Le journal contient des **faits** (ADR 0001) : `GameRunner::resume` les rejoue par `apply`, pure et infaillible, sans
  repasser par `decide` ni par la validation. `CardPlayed` porte l'effet déjà résolu, les pioches et les dés portent leur
  résultat. Une règle de validation corrigée (action devenue légale ou illégale) ne change donc pas le rejeu ; seuls le
  changent : la forme JSON d'un type porté par la sauvegarde, et le sens que `apply` donne à un événement existant.
- Des changements compatibles existent déjà : `GameConfig::dummy_hero` porte `#[serde(default)]` (spec 061) pour qu'un journal
  d'avant se lise.
- Les préférences (spec 080) ont leur propre numéro, `PREFERENCES_VERSION` (1), et sont tolérantes : champ absent = valeur par
  défaut, champ inconnu ignoré, autre numéro lu quand même avec un `warn`.
- Les 51 parties préparées de `scripts/parties/` sont de vraies sauvegardes, régénérées par le test `parties_preparees`
  quand le format change. Elles contiennent 56 des 153 variantes de `Event` (mesuré le 2026-11-26 avec un script jetable sur
  les fichiers de `main`) : surtout des mises en place, peu de combats et de fins de partie.
- La sauvegarde automatique (`auto.json`) est réécrite au début de chaque tour (spec 024, Q5, `session.rs`) ; une
  sauvegarde manuelle n'écrase jamais un fichier (suffixe `-2`, `-3`…).

## Règles

Aucune règle du livret. Les décisions de cette spec révisent l'ADR 0001 (section « Révision (spec #52) », proposée dans la
même PR et validée avec la spec).

- **Garantie** (Mathieu, 2026-11-26) : une sauvegarde de l'une des trois dernières versions publiées se charge avec la version
  courante. Rien d'autre n'est garanti : ni une sauvegarde plus ancienne, ni une sauvegarde écrite par une version de
  développement (`-dev`, `-rc`), comme celles d'avant la v0.1.0.
- **Versions gardées** : les trois dernières versions publiées (au début : la seule v0.1.0, puis 0.1.0 et 0.2.0…).
  `OLDEST_SUPPORTED_FORMAT` est le format de la plus ancienne d'entre elles (1 pour la v0.1.0, celui du tag `v0.1.0`) ;
  `PREVIOUS_RELEASE_FORMAT` celui de la dernière. Ils ne changent qu'à la release (voir Documentation, `docs/release.md`).
- **Changement compatible** : une sauvegarde d'une version gardée se lit **et** se rejoue à l'identique sans
  rien faire. Exemples : variante ajoutée à une enum, champ ajouté avec `#[serde(default)]` dont la valeur par défaut est celle
  qu'avait implicitement l'ancien journal. Pas de changement de numéro, pas de migration.
- **Changement incompatible** : tout le reste. Champ ou variante renommé ou retiré, champ obligatoire ajouté, type changé,
  usage de générateur renommé, ou sens nouveau donné par `apply` à un événement existant. Il exige la migration.
- **Numéro de format** (Q1, décidée) : `EVENT_FORMAT_VERSION` vaut `PREVIOUS_RELEASE_FORMAT` tant qu'aucun changement
  incompatible n'a été fait depuis la dernière release, `PREVIOUS_RELEASE_FORMAT + 1` sinon. Le premier changement
  incompatible du cycle fait le +1 et écrit la migration ; les suivants du même cycle complètent cette même migration.
- **Chaîne courte** : une migration par changement de format entre les versions gardées et le code courant, donc trois au
  plus (N-3 → N-2, N-2 → N-1, N-1 → courant). Charger une sauvegarde applique les migrations de son format jusqu'au format
  courant. À chaque release, la version la plus ancienne sort de la fenêtre : ses témoins, son empreinte et la migration qui
  partait de son format sont supprimés.

## Modèle

Pas de terme de jeu nouveau. Termes techniques employés ici et dans la doc :

- **Migration** : fonction pure qui transforme le JSON d'une sauvegarde du format N en JSON du format N+1, ou échoue avec une
  raison. Elle travaille sur `serde_json::Value`, pas sur des copies figées des anciens types Rust (Q2).
- **Chaîne de migrations** : la liste ordonnée des migrations, la première partant de `OLDEST_SUPPORTED_FORMAT` ; vide tant
  qu'aucun changement incompatible n'a été fait.
- **Témoin** : sauvegarde d'une version gardée, telle quelle dans les tests (`crates/arcanum-persistance/tests/temoins/v<X.Y.Z>/`,
  un dossier par version gardée, trois au plus), qui doit se charger avec le code courant.
- **Empreinte du format** : description, générée par le code, de la forme JSON de `SaveGame` et de tous les types qu'il porte
  (Q6, décidée). L'empreinte courante est commitée dans `crates/arcanum-persistance/tests/format/sauvegarde.yaml` ; celle de
  chaque version gardée, figée à sa release, dans `tests/format/v<X.Y.Z>.yaml`.

Où vit le code : un module `migration` d'`arcanum-persistance`, qui porte `OLDEST_SUPPORTED_FORMAT`, `PREVIOUS_RELEASE_FORMAT`
et la chaîne. `load` reste le seul point d'entrée : il lit `version`, puis

| `version` lue | Résultat |
|---|---|
| `EVENT_FORMAT_VERSION` | désérialisée sans migration |
| de `OLDEST_SUPPORTED_FORMAT` à `EVENT_FORMAT_VERSION - 1` | migrée en mémoire de proche en proche, puis désérialisée |
| plus grande que `EVENT_FORMAT_VERSION` | `NewerVersion` |
| plus petite que `OLDEST_SUPPORTED_FORMAT`, ou 0 | `TooOld` |

Le fichier sur disque n'est jamais réécrit par le chargement (Q4).

Erreurs de chargement (`LoadError`) :

| Avant | Après | Quand |
|---|---|---|
| `OtherVersion { found }` | `NewerVersion { found }` | `version` > `EVENT_FORMAT_VERSION` |
| — | `TooOld { found }` | `version` plus petite que `OLDEST_SUPPORTED_FORMAT`, ou 0 |
| — | `NotMigratable { found, reason }` | une migration de la chaîne échoue |
| `Invalid` | `Invalid` (inchangé) | JSON invalide, `version` absente, ou JSON migré qui ne se désérialise pas |

## Actions et événements

Aucun : la spec ne change ni `Action` ni `Event`.

## Comportements attendus

Chaque cas devient au moins un test (nom indicatif entre parenthèses). Les cas C2 à C7 injectent une chaîne de test
(fictive, 1 → 2 → 3, format courant 3) : la vraie chaîne est vide tant qu'aucun changement incompatible n'est fait. La fonction
de chargement prend donc le plus ancien format gardé, le format courant et la chaîne en paramètres, et `load` l'appelle avec
les vrais.

- **C1** — Comme aujourd'hui (spec 024, tests `save_load_round_trip` et `reloaded_state_equals_saved_state` de
  `tests/sauvegarde.rs`), une sauvegarde du format courant se charge sans migration et redonne le même état.
- **C2** — Avec la chaîne de test, une sauvegarde du format 1 passe par 1 → 2 puis 2 → 3, chacune une fois et dans cet
  ordre, et se charge (`old_save_runs_each_migration_once_in_order`).
- **C3** — Avec la même chaîne, une sauvegarde du format 2 ne passe que par 2 → 3, et une du format 3 par aucune
  (`migration_starts_at_save_format`).
- **C4** — Une sauvegarde d'un format plus récent est refusée par `NewerVersion { found }`, sans tenter de la lire
  (`newer_save_is_refused`). Remplace `other_version_is_refused`.
- **C5** — Une migration qui échoue donne `NotMigratable { found, reason }` ; le fichier sur disque est resté octet pour octet
  identique (`failed_migration_leaves_file_untouched`).
- **C6** — Une sauvegarde migrée puis sauvegardée de nouveau s'écrit au format courant ; l'original n'a pas changé
  (`migrated_save_is_written_in_current_format`).
- **C7** — Comme aujourd'hui (`load`, message « version absente »), une sauvegarde sans `version` est `Invalid`. Avec un
  plus ancien format gardé 2 (chaîne 2 → 3), une sauvegarde du format 1 est `TooOld`, et `version` à 0 aussi
  (`save_older_than_oldest_supported_is_too_old`).
- **C8** — `list` montre une sauvegarde migrable comme les autres, avec son résumé lu après migration ; une sauvegarde plus
  récente ou trop ancienne y est illisible avec sa raison (`list_shows_previous_saves_and_flags_others`). Complète
  `list_sorts_and_reports_unreadable`.
- **C9** — Une migration réussie écrit un `info` « sauvegarde <chemin> migrée du format N au format M » ; une migration ratée
  écrit un `warn` avec la raison (vérifié à la lecture du code, pas de test sur les logs, comme pour les autres messages de
  `arcanum-persistance`).
- **C10** — Chaque témoin de `tests/temoins/v<X.Y.Z>/` se charge avec la vraie chaîne, se rejoue (`SaveGame::resume`) sans erreur, et la partie reprise a le même résumé que celui écrit dans le fichier (manche, tour,
  Jour ou Nuit, héros, scénario) ; elle n'est pas bloquée : terminée, ou au moins une action légale
  (`every_witness_loads_and_resumes`). Un `apply` qui change de sens sans migration se voit ici dès qu'il change le résumé.
- **C11** — Cohérence : `EVENT_FORMAT_VERSION` vaut `PREVIOUS_RELEASE_FORMAT` ou `PREVIOUS_RELEASE_FORMAT + 1` ; la chaîne
  compte exactement `EVENT_FORMAT_VERSION - OLDEST_SUPPORTED_FORMAT` migrations, trois au plus ; il y a au plus trois dossiers
  de témoins, autant que d'empreintes `v<X.Y.Z>.yaml`, et les témoins du plus ancien portent `version` =
  `OLDEST_SUPPORTED_FORMAT` (`migration_chain_matches_supported_versions`).
- **C12** — L'empreinte du format générée depuis les types courants est égale au fichier commité ; sinon le test échoue avec
  un message qui dit quoi faire : « changement compatible : régénérer l'empreinte ; incompatible : passer
  `EVENT_FORMAT_VERSION` à `PREVIOUS_RELEASE_FORMAT + 1` si ce n'est pas déjà fait dans ce cycle, et écrire ou compléter la
  migration » (`save_format_matches_fingerprint`). Régénération : `ARCANUM_REGENERER_DOCS=1 cargo test -p arcanum-persistance
  --test format`, comme les parties préparées. Le diff de l'empreinte rend tout changement de forme visible dans la PR.

### Savoir ce qui doit migrer entre deux versions

Deux sortes de changements incompatibles, deux moyens de les trouver :

- **Forme JSON** (champ, variante ou type changé, dans un événement ou dans ce qu'il porte : `Effect`, `GameConfig`…) :
  `diff tests/format/v<X.Y.Z>.yaml tests/format/sauvegarde.yaml` donne la liste exacte des types changés depuis la version
  X.Y.Z, et donc ce que la migration doit transformer. C12 empêche qu'un changement échappe à l'empreinte courante. La première
  empreinte figée, `v0.1.0.yaml`, est générée par le fil d'implémentation sur le code du tag `v0.1.0` (copie de travail du tag,
  test de génération ajouté le temps de l'écrire), pour que le premier diff compare bien à la v0.1.0.
- **Sens d'`apply`** (un événement existant, de même forme, rejoué autrement) : invisible dans l'empreinte. La recette
  « ajouter un événement » de `docs/architecture.md` le classe comme incompatible, et chaque PR qui change le bras d'`apply`
  d'un événement existant le dit en une ligne (« Sens d'`apply` changé : non » ou « oui : <événements> »). Les témoins (C10)
  rattrapent ce qui change le résumé ou bloque la partie.
- **C13** — L'écran de chargement affiche la raison de `NewerVersion` par `load.error.newer_version`, celle de `TooOld` par
  `load.error.too_old` et celle de `NotMigratable` par `load.error.not_migratable`
  (`load_window_explains_refused_saves`, fonction pure de `controls.rs`).

### Témoins de la v0.1.0 (Q7)

- Les 51 parties préparées de `scripts/parties/` **telles qu'au tag `v0.1.0`**, recopiées dans `tests/temoins/v0.1.0/` (elles
  continuent ensuite d'être régénérées dans `scripts/parties/`, les témoins jamais).
- Les deux parcours de fin de jalon de `parcours_j6.rs` (spec 069), joués sur le code du tag `v0.1.0` et sauvegardés en fin
  de partie : la partie complète (Conquête en solitaire, Tovak contre le fantôme Goldyx, six manches, mission échouée) et
  l'épisode de la conquête des cités (combats de cité, unités, retraite, mission accomplie).
- En option, une vraie partie jouée par Mathieu avec le binaire de la release, s'il en fournit une.
- Couverture mesurée le 2026-11-26 (script jetable, variantes de `Event` présentes) : parties préparées 56 sur 153, avec la
  partie complète du J6 63, avec l'épisode 74.
- Le fil d'implémentation mesure les variantes de `Event` présentes dans l'ensemble des témoins et note le compte dans la PR ;
  l'empreinte (C12) couvre les variantes absentes.

### Tests existants modifiés

Cherchés à l'usage (`grep -rn 'OtherVersion\|EVENT_FORMAT_VERSION\|other_version'` dans `crates/`) :

- `tests/sauvegarde.rs`, `other_version_is_refused` : devient C4 (`NewerVersion`).
- `crates/arcanum-ui/src/controls.rs` (bras `LoadError::OtherVersion`) : les trois nouvelles variantes (C13) ; pas de test
  existant qui le lise.
- `crates/arcanum-core/tests/noyau_moteur.rs:401`, `assert_eq!(EVENT_FORMAT_VERSION, 1)` : inchangé par cette spec ; le
  premier changement incompatible le modifiera (c'est voulu, il le signale).

## Cas limites et refus

- **L1** — Une sauvegarde d'avant la v0.1.0, ou d'une version de développement, peut porter un format gardé ou le format
  courant avec une autre forme : elle reste `Invalid` si elle ne se lit pas, ou peut se rejouer de travers. Pas de garantie
  (Règles).
- **L2** — Un journal migré peut se rejouer en un état différent de celui qu'avait le joueur si la migration est fausse ou si
  `apply` a changé de sens : la partie se charge quand même (Q3). La migration est le seul endroit où corriger : elle peut
  modifier, ajouter ou retirer des événements, ou déclarer la sauvegarde non migrable (`NotMigratable`) quand aucune
  transformation ne donne une partie cohérente.
- **L3** — La migration ne dépend que du JSON : ni horloge, ni fichier, ni contenu du jeu (le scénario est déjà dans
  `GameStarted`). Elle est pure, donc testable sans disque.
- **L4** — Le résumé (`SaveSummary`) et `rng_positions` passent par la migration comme le journal : la migration reçoit le
  fichier entier.
- **L5** — Un joueur qui revient avec une sauvegarde sortie de la fenêtre (0.1.0 lue en 0.5.0) voit `TooOld`. Le message lui
  dit de l'ouvrir avec une version intermédiaire et de la sauvegarder de nouveau ; les binaires des anciennes versions restent
  sur la page des releases.
- **L6** — Une version qui ne change pas le format compte quand même dans la fenêtre : les trois dernières versions publiées
  peuvent partager un même format, et la chaîne est alors plus courte.

## Information cachée

Sans objet : la sauvegarde est déjà le journal complet (spec 024) ; rien de la vue du joueur ne change.

## Libellés (i18n)

| Clé | fr | en |
|---|---|---|
| `load.error.newer_version` | Sauvegarde d'une version plus récente du jeu. | Saved by a newer version of the game. |
| `load.error.too_old` | Sauvegarde trop ancienne : l'ouvrir avec une version intermédiaire du jeu, puis la sauvegarder de nouveau. | Save too old: open it with an intermediate version of the game, then save it again. |
| `load.error.not_migratable` | Sauvegarde d'une ancienne version du jeu, qui ne se charge plus. | Saved by an old version of the game that can no longer be loaded. |

Clé retirée : `load.error.other_version` (fr « Sauvegarde d'une autre version du jeu. », en « Saved by another version of the
game. », relues dans `assets/i18n/*.json` de `main`), remplacée par les trois ci-dessus. Clé réutilisée sans changement :
`load.unreadable` (« {name} · {reason} »), qui donne par exemple « auto · Sauvegarde d'une version plus récente du jeu. ».

## Interface

- **Ligne d'une sauvegarde refusée** (plus récente, trop ancienne, non migrable) dans la fenêtre « Charger une partie » : nom
  et raison, sans bouton Charger, comme les autres illisibles aujourd'hui (C13, C8).
- **Sauvegarde migrée** : rien de visible, elle se charge comme une autre (C8, C2).
- Pas de partie préparée : la vérification à l'œil se limite à charger un témoin avec le jeu (`verif-manuelle.sh` sur une
  copie du témoin dans le dossier des sauvegardes), une fois qu'une vraie migration existera ; à cette spec, il n'y en a pas et
  rien ne change à l'écran hors des trois raisons.

### Raccourcis clavier

Aucun.

## Documentation

- `docs/architecture.md`, recette « ajouter un événement », point 5 : changement compatible ou non, numéro de format une fois
  par cycle, migration, empreinte à régénérer, ligne « Sens d'`apply` changé » dans la PR.
- `docs/release.md` (créé par la spec #33), deux étapes :
  - avant le tag : rien de plus que la CI (C10 à C12 vérifient déjà que les sauvegardes des versions gardées se chargent) ;
  - juste après le tag, dans la PR qui passe à `X.Y.Z-dev` : ajouter `tests/temoins/v<X.Y.Z>/` (parties préparées du tag, plus
    sauvegardes des parcours de fin de jalon, une vraie partie de Mathieu en option) et `tests/format/v<X.Y.Z>.yaml` (copie de
    `sauvegarde.yaml`), donner à `PREVIOUS_RELEASE_FORMAT` la valeur de `EVENT_FORMAT_VERSION` ; s'il y a alors plus de trois
    versions gardées, supprimer les témoins et l'empreinte de la plus ancienne, avancer `OLDEST_SUPPORTED_FORMAT` et retirer
    les migrations qui partent d'un format plus ancien.
- `.claude/rules/arcanum-persistance.md` : la règle « pas de migration avant la fin du J6 » remplacée par celles de cette spec.
- ADR 0001 : section « Révision (spec #52) » (proposée dans cette PR).
- Commentaires de `SaveGame::version`, `LoadError::Invalid` et `EVENT_FORMAT_VERSION` (« reste 1 jusqu'à la fin du J6 ») mis à
  jour.

## Hors périmètre

- Sauvegardes d'avant la v0.1.0 (#49) et des versions de développement.
- Sauvegardes de plus de trois versions publiées (`TooOld`, L5).
- Migration des préférences : leur lecture tolérante suffit (Q5).
- Copie de secours d'une sauvegarde avant migration (Q4).
- Une vraie migration : aucun changement incompatible n'est prévu par cette spec ; la première spec qui en fait un écrira la
  migration 1 → 2 avec ce mécanisme.
- Sauvegardes d'une version plus récente lues par une version plus ancienne (refusées, C4).

## Décisions

- **Portée** (Mathieu, 2026-11-26) : seules les sauvegardes des trois dernières versions publiées sont garanties ; rien pour
  les versions de développement. Remplace la première rédaction, qui garantissait toutes les versions suivantes avec des
  témoins gardés à vie (et une étape intermédiaire, le même jour, limitée à la seule version précédente).
- **Q1 — Quand le numéro de format change-t-il ?** Au plus une fois par version publiée (Mathieu, 2026-11-26, avec la portée).
- **Q6 — Comment détecter un changement de forme non décidé ?** Empreinte du format et témoins (Mathieu, 2026-11-26).
  Empreinte générée par `serde-reflection`, dev-dependency d'`arcanum-persistance` seulement (licence MIT OR Apache-2.0, à
  confirmer par `cargo deny check licenses`) : elle n'entre pas dans le jeu livré. Les témoins ne voient que les variantes
  qu'ils contiennent (56 sur 153) ; l'empreinte voit tous les types. L'essai de faisabilité (`trace_type` sur `SaveGame`, avec
  les sérialisations faites main de `RngUsage` et de `rng_positions`) n'a pas été lancé dans le fil de rédaction : le fil
  d'implémentation le fait en premier, et si le traçage échoue, revient à Mathieu avant de passer aux témoins seuls.
- **Q2, Q3, Q4, Q5, Q8** (Mathieu, 2026-11-26, recommandations retenues) : la migration travaille sur le JSON
  (`serde_json::Value`) du fichier entier ; un journal migré qui se rejoue autrement se charge quand même (L2) ; pas de copie
  de secours avant migration (C5, C6) ; les préférences gardent leur lecture tolérante, sans migration ; trois messages de
  refus distincts (`NewerVersion`, `TooOld`, `NotMigratable`).

## Questions ouvertes

**Q7 — Quels témoins pour la v0.1.0 ?**
(a) Les parties préparées du tag `v0.1.0` et les deux parcours de fin de jalon (J6) sauvegardés, plus une vraie partie de
Mathieu en option. (b) Les parties préparées et une vraie partie de Mathieu obligatoire.
*Recommandation : (a)*, proposée par Mathieu. Les parcours du J6 apportent ce qui manquait aux parties préparées (fin de
manche, fantôme, combats de cité, retraite, Blessures, fin de partie : 74 variantes sur 153 au lieu de 56) et se refont à
chaque release sans effort de Mathieu. Une vraie partie reste un plus (niveaux, sorts et artefacts variés), mais n'est plus
indispensable : l'empreinte (C12) couvre la forme des variantes jamais jouées.
