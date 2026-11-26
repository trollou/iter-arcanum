# #52 — Migration des sauvegardes

- Statut : à valider
- Issue : #52 (milestone v0.2.0)
- Dépend de : spec 024 (sauvegarde et retour en arrière), spec 038 (un générateur par usage), tag `v0.1.0` (#49), #55 (remplacer
  `chrono`, avant celle-ci)
- Livret : aucun (outillage, pas de règle du jeu)
- Crates touchées : arcanum-persistance (migration, contrôle, témoins), arcanum-ui (messages de refus), arcanum-core (commentaire
  de `EVENT_FORMAT_VERSION` seulement)

## Objectif

À partir de la v0.1.0, première version publique, une partie sauvegardée avec une version du jeu se recharge avec toutes les
versions suivantes. Le joueur ne voit rien : l'ancienne sauvegarde apparaît dans la liste et se charge comme une autre. Côté
développement, un test empêche de changer la forme d'une sauvegarde sans le décider, et chaque version publiée laisse des
sauvegardes témoins qui doivent se charger à chaque `cargo test`.

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

- **Changement compatible** : une sauvegarde de la version publiée précédente se lit **et** se rejoue à l'identique sans
  rien faire. Exemples : variante ajoutée à une enum, champ ajouté avec `#[serde(default)]` dont la valeur par défaut est celle
  qu'avait implicitement l'ancien journal. Pas de changement de numéro, pas de migration.
- **Changement incompatible** : tout le reste. Champ ou variante renommé ou retiré, champ obligatoire ajouté, type changé,
  usage de générateur renommé, ou sens nouveau donné par `apply` à un événement existant. Il exige une migration.
- **Numéro de format** (Q1) : au plus un nouveau numéro par version publiée. Le premier changement incompatible d'un cycle
  passe `EVENT_FORMAT_VERSION` de N (format de la dernière version publiée) à N+1 et écrit la migration N → N+1 ; les
  changements incompatibles suivants du même cycle complètent cette migration au lieu d'en ajouter une. Une sauvegarde écrite
  entre deux versions publiées (développement, `-dev`, `-rc`) n'est pas garantie, comme celles d'avant la v0.1.0.
- **Format de la v0.1.0** : 1, celui du tag `v0.1.0`.

## Modèle

Pas de terme de jeu nouveau. Termes techniques employés ici et dans la doc :

- **Migration** : fonction pure qui transforme le JSON d'une sauvegarde du format N en JSON du format N+1, ou échoue avec une
  raison. Elle travaille sur `serde_json::Value`, pas sur des copies figées des anciens types Rust (Q2).
- **Chaîne de migrations** : la liste ordonnée des migrations, la première partant du format 1. Charger une sauvegarde du
  format N applique les migrations N → N+1, …, jusqu'au format courant, puis désérialise.
- **Témoin** : sauvegarde d'une version publiée, gardée telle quelle dans les tests (`crates/arcanum-persistance/tests/temoins/v<N>/`),
  qui doit toujours se charger.
- **Empreinte du format** : description, générée par le code, de la forme JSON de `SaveGame` et de tous les types qu'il porte,
  commitée dans `crates/arcanum-persistance/tests/format/sauvegarde.yaml` (Q6).

Où vit le code : un module `migration` d'`arcanum-persistance`. `load` reste le seul point d'entrée : il lit `version`, refuse
un format plus récent, migre un format plus ancien en mémoire, puis désérialise. Le fichier sur disque n'est jamais réécrit par
le chargement (Q4).

Erreurs de chargement (`LoadError`) :

| Avant | Après | Quand |
|---|---|---|
| `OtherVersion { found }` | `NewerVersion { found }` | `version` > `EVENT_FORMAT_VERSION` |
| — | `NotMigratable { found, reason }` | une migration de la chaîne échoue, ou `version` vaut 0 |
| `Invalid` | `Invalid` (inchangé) | JSON invalide, `version` absente, ou JSON migré qui ne se désérialise pas |

## Actions et événements

Aucun : la spec ne change ni `Action` ni `Event`.

## Comportements attendus

Chaque cas devient au moins un test (nom indicatif entre parenthèses). Les cas C2 à C6 injectent une chaîne de migrations de
test (fictives, 1 → 2 → 3) : la vraie chaîne est vide tant qu'aucun changement incompatible n'est fait. La fonction de
chargement prend donc la chaîne et le format courant en paramètres, et `load` l'appelle avec les vrais.

- **C1** — Comme aujourd'hui (spec 024, tests `save_load_round_trip` et `reloaded_state_equals_saved_state` de
  `tests/sauvegarde.rs`), une sauvegarde du format courant se charge sans migration et redonne le même état.
- **C2** — Avec la chaîne de test et le format courant 3, une sauvegarde du format 1 passe par 1 → 2 puis 2 → 3, chacune une
  fois et dans cet ordre, et se charge (`old_save_runs_each_migration_once_in_order`).
- **C3** — Avec la même chaîne, une sauvegarde du format 2 ne passe que par 2 → 3 (`migration_starts_at_save_format`).
- **C4** — Une sauvegarde d'un format plus récent est refusée par `NewerVersion { found }`, sans tenter de la lire
  (`newer_save_is_refused`). Remplace `other_version_is_refused`.
- **C5** — Une migration qui échoue donne `NotMigratable { found, reason }` ; le fichier sur disque est resté octet pour octet
  identique (`failed_migration_leaves_file_untouched`).
- **C6** — Une sauvegarde migrée puis sauvegardée de nouveau s'écrit au format courant ; l'original n'a pas changé
  (`migrated_save_is_written_in_current_format`).
- **C7** — Comme aujourd'hui (`load`, message « version absente »), une sauvegarde sans `version` est `Invalid` ; `version` à 0
  est `NotMigratable` (`version_zero_is_not_migratable`).
- **C8** — `list` montre une sauvegarde migrable comme les autres, avec son résumé lu après migration ; une sauvegarde plus
  récente y est illisible avec la raison `NewerVersion` (`list_shows_old_saves_and_flags_newer_ones`). Complète
  `list_sorts_and_reports_unreadable`.
- **C9** — Une migration réussie écrit un `info` « sauvegarde <chemin> migrée du format N au format M » ; une migration ratée
  écrit un `warn` avec la raison (vérifié à la lecture du code, pas de test sur les logs, comme pour les autres messages de
  `arcanum-persistance`).
- **C10** — Chaque témoin de `tests/temoins/v<N>/` se charge avec la vraie chaîne, se rejoue (`SaveGame::resume`) sans erreur,
  et la partie reprise a le même résumé que celui écrit dans le fichier (manche, tour, Jour ou Nuit, héros, scénario) ; elle
  n'est pas bloquée : terminée, ou au moins une action légale (`every_witness_loads_and_resumes`). Un `apply` qui change de sens
  sans migration se voit ici dès qu'il change le résumé.
- **C11** — Cohérence de la chaîne : elle compte exactement `EVENT_FORMAT_VERSION - 1` migrations, et le plus haut dossier de
  témoins `v<N>` ne dépasse pas `EVENT_FORMAT_VERSION` (`migration_chain_matches_format_version`).
- **C12** — L'empreinte du format générée depuis les types courants est égale au fichier commité ; sinon le test échoue avec
  un message qui dit quoi faire : « changement compatible : régénérer l'empreinte ; incompatible : passer
  `EVENT_FORMAT_VERSION` à N+1 si ce n'est pas déjà fait dans ce cycle, et écrire ou compléter la migration »
  (`save_format_matches_fingerprint`). Régénération : `ARCANUM_REGENERER_DOCS=1 cargo test -p arcanum-persistance --test format`,
  comme les parties préparées. Le diff de l'empreinte rend tout changement de forme visible dans la PR.
- **C13** — L'écran de chargement affiche la raison de `NewerVersion` par `load.error.newer_version` et celle de
  `NotMigratable` par `load.error.not_migratable` (`load_window_explains_newer_and_unmigratable_saves`, fonction pure de
  `controls.rs`).

### Témoins de la v0.1.0 (Q7)

- Les 51 parties préparées de `scripts/parties/` **telles qu'au tag `v0.1.0`**, recopiées dans `tests/temoins/v1/` (elles
  continuent ensuite d'être régénérées dans `scripts/parties/`, les témoins jamais).
- Une ou deux vraies parties jouées par Mathieu avec le binaire de la release v0.1.0 (sauvegarde manuelle en fin de partie, et
  si possible en plein combat), qui couvrent ce que les parties préparées n'ont pas.
- Le fil d'implémentation mesure les variantes de `Event` présentes dans l'ensemble des témoins et note le compte dans la PR ;
  l'empreinte (C12) couvre les variantes absentes.

### Tests existants modifiés

Cherchés à l'usage (`grep -rn 'OtherVersion\|EVENT_FORMAT_VERSION\|other_version'` dans `crates/`) :

- `tests/sauvegarde.rs`, `other_version_is_refused` : devient C4 (`NewerVersion`).
- `crates/arcanum-ui/src/controls.rs` (bras `LoadError::OtherVersion`) : les deux nouvelles variantes (C13) ; pas de test
  existant qui le lise.
- `crates/arcanum-core/tests/noyau_moteur.rs:401`, `assert_eq!(EVENT_FORMAT_VERSION, 1)` : inchangé par cette spec ; le
  premier changement incompatible le modifiera (c'est voulu, il le signale).

## Cas limites et refus

- **L1** — Une sauvegarde d'avant la v0.1.0 porte aussi `version: 1` mais peut avoir une autre forme : elle reste `Invalid`
  si elle ne se lit pas (hors périmètre, notes de la v0.1.0, #49).
- **L2** — Un journal migré peut se rejouer en un état différent de celui qu'avait le joueur si une migration est fausse ou si
  `apply` a changé de sens : la partie se charge quand même (Q3). La migration est le seul endroit où corriger : elle peut
  modifier, ajouter ou retirer des événements, ou déclarer le format non migrable (`NotMigratable`) quand aucune
  transformation ne donne une partie cohérente.
- **L3** — Une migration ne dépend que du JSON : ni horloge, ni fichier, ni contenu du jeu (le scénario est déjà dans
  `GameStarted`). Elle est pure, donc testable sans disque.
- **L4** — Le résumé (`SaveSummary`) et `rng_positions` passent par la migration comme le journal : la migration reçoit le
  fichier entier.

## Information cachée

Sans objet : la sauvegarde est déjà le journal complet (spec 024) ; rien de la vue du joueur ne change.

## Libellés (i18n)

| Clé | fr | en |
|---|---|---|
| `load.error.newer_version` | Sauvegarde d'une version plus récente du jeu. | Saved by a newer version of the game. |
| `load.error.not_migratable` | Sauvegarde d'une ancienne version du jeu, qui ne se charge plus. | Saved by an old version of the game that can no longer be loaded. |

Clé retirée : `load.error.other_version` (fr « Sauvegarde d'une autre version du jeu. », en « Saved by another version of the
game. », relues dans `assets/i18n/*.json` de `main`), remplacée par les deux ci-dessus. Clé réutilisée sans changement :
`load.unreadable` (« {name} · {reason} »), qui donne par exemple « auto · Sauvegarde d'une version plus récente du jeu. ».

## Interface

- **Ligne d'une sauvegarde plus récente ou non migrable** dans la fenêtre « Charger une partie » : nom et raison, sans
  bouton Charger, comme les autres illisibles aujourd'hui (C13, C8).
- **Sauvegarde migrée** : rien de visible, elle se charge comme une autre (C8, C2).
- Pas de partie préparée : la vérification à l'œil se limite à charger un témoin v1 avec le jeu (`verif-manuelle.sh` sur une
  copie du témoin dans le dossier des sauvegardes), une fois qu'une vraie migration existera ; à cette spec, la chaîne est
  vide et rien ne change à l'écran hors des deux raisons.

### Raccourcis clavier

Aucun.

## Documentation

- `docs/architecture.md`, recette « ajouter un événement », point 5 : changement compatible ou non, numéro de format une fois
  par cycle, migration, empreinte à régénérer.
- `docs/release.md` (créé par la spec #33) : avant le tag d'une version dont `EVENT_FORMAT_VERSION` dépasse le plus haut
  dossier de témoins, ajouter `tests/temoins/v<N>/` (parties préparées du tag + une vraie partie de Mathieu, comme pour la v1).
- `.claude/rules/arcanum-persistance.md` : la règle « pas de migration avant la fin du J6 » remplacée par celles de cette spec.
- ADR 0001 : section « Révision (spec #52) » (proposée dans cette PR).
- Commentaires de `SaveGame::version`, `LoadError::Invalid` et `EVENT_FORMAT_VERSION` (« reste 1 jusqu'à la fin du J6 ») mis à
  jour.

## Hors périmètre

- Sauvegardes d'avant la v0.1.0 (#49).
- Migration des préférences : leur lecture tolérante suffit (Q5).
- Copie de secours d'une sauvegarde avant migration (Q4).
- Une vraie migration : aucun changement incompatible n'est prévu par cette spec ; la première spec qui en fait un écrira la
  migration 1 → 2 avec ce mécanisme.
- Sauvegardes d'une version plus récente lues par une version plus ancienne (refusées, C4).

## Questions ouvertes

**Q1 — Quand le numéro de format change-t-il ?**
(a) Au plus une fois par version publiée : le premier changement incompatible du cycle passe à N+1, les suivants complètent la
même migration. (b) À chaque changement incompatible fusionné dans `main`, avec une migration chacun.
*Recommandation : (a).* Seules les sauvegardes des versions publiées comptent pour les joueurs ; (b) multiplierait les
migrations (une par spec ou presque) pour protéger des sauvegardes de développement que personne d'autre n'a.

**Q2 — Sur quoi travaille une migration ?**
(a) Sur le JSON (`serde_json::Value`) du fichier entier. (b) Sur des copies figées des anciens types Rust (`v1::Event`…),
converties vers les nouveaux.
*Recommandation : (a).* `Event` et ce qu'il porte font près de 3 000 lignes : les figer à chaque version alourdirait le code
pour rien, alors qu'une migration ne touche en général qu'un champ ou une variante. Le témoin (C10) vérifie qu'elle est juste.

**Q3 — Un journal migré se rejoue en un état différent : que faire ?**
(a) Charger quand même ; le témoin (C10) et l'empreinte (C12) servent à le voir avant la release, et la migration à le
corriger (L2). (b) Comparer à chaque chargement le résumé recalculé au résumé du fichier, et refuser s'ils diffèrent.
*Recommandation : (a).* Le rejeu n'exécute que des faits (`apply`), pas la validation : une règle corrigée ne le casse pas.
Seul un `apply` qui change de sens le peut, et c'est un changement incompatible que la migration doit traiter. Refuser au
chargement priverait le joueur d'une partie jouable pour un écart de résumé parfois anodin.

**Q4 — Garder une copie de secours avant migration ?**
(a) Non : la migration se fait en mémoire, le fichier n'est réécrit que par une nouvelle sauvegarde (C5, C6). (b) Oui : copie
de l'ancien fichier dans un sous-dossier avant la première écriture au nouveau format.
*Recommandation : (a).* Une sauvegarde manuelle n'est jamais écrasée ; seule `auto.json` l'est, et elle l'est déjà à chaque
tour. Une copie n'aiderait que si la migration se trompe sans échouer, ce que les témoins doivent empêcher.

**Q5 — Même mécanisme pour les préférences ?**
(a) Non : leur lecture tolérante (spec 080, C3 et C4) reste ; `PREFERENCES_VERSION` ne change que si un champ change de sens,
et une conversion ad hoc suffira alors. (b) Oui : chaîne de migrations et témoins aussi.
*Recommandation : (a).* Quatre réglages, tous avec une valeur par défaut : au pire le joueur refait un réglage. Une chaîne
serait plus lourde que le risque.

**Q6 — Comment détecter un changement de forme non décidé ?**
(a) Empreinte du format générée par `serde-reflection` (dev-dependency d'`arcanum-persistance` seulement, licence MIT OR
Apache-2.0, à confirmer par `cargo deny check licenses`) et commitée (C12), en plus des témoins (C10). (b) Témoins seuls.
*Recommandation : (a).* Les témoins ne voient que les variantes qu'ils contiennent (56 sur 153 pour les parties préparées) ;
l'empreinte voit tous les types, y compris `Effect` et ce que portent les événements jamais joués dans un témoin. La
dépendance n'entre pas dans le jeu livré. Nouvelle dépendance à valider (CLAUDE.md) ; l'essai de faisabilité (`trace_type`
sur `SaveGame`, avec les sérialisations faites main de `RngUsage` et de `rng_positions`) n'a pas été lancé dans ce fil faute
de cette validation : le fil d'implémentation le fait en premier, et si le traçage échoue, revient à Mathieu avant de passer
à (b).

**Q7 — Quels témoins pour la v0.1.0 ?**
(a) Les parties préparées du tag `v0.1.0` et une ou deux vraies parties de Mathieu jouées avec le binaire de la release.
(b) Les parties préparées seules. (c) Des parties générées par les parcours de bout en bout (`parcours_*.rs`) rejoués au tag.
*Recommandation : (a).* Les parties préparées sont déjà de vraies sauvegardes produites par le code du tag, sans rien à
écrire ; une vraie partie couvre combats, niveaux et fin de partie mieux qu'elles. (c) demanderait d'ajouter au tag un
générateur qui n'y est pas. Mathieu fournit sa sauvegarde dans la bibliothèque du projet quand il a joué.

**Q8 — Un message distinct pour une sauvegarde plus récente ?**
(a) Oui, `NewerVersion` et `NotMigratable` séparés (C4, C13). (b) Garder un seul message « autre version ».
*Recommandation : (a).* Les deux appellent un geste différent : mettre le jeu à jour, ou renoncer à la partie.
