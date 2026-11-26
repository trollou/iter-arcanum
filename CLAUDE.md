# CLAUDE.md

Jeu PC **Iter Arcanum**, inspiré du jeu de plateau **Mage Knight** (édition Ultimate, MKUE), en client lourd. Les identifiants internes portent son nom depuis la spec 082 : crates `arcanum-*`, exécutable et dossiers de données `iter-arcanum`, variables d'environnement `ARCANUM_*`.

## Le projet

- **Rust** (édition 2024, toolchain stable), UI **egui** via **eframe**.
- Plateformes : **Windows, Linux et macOS**, à égalité (macOS depuis la spec #7, vérifié par la CI et le workflow manuel `capture-macos.yml` : personne n'a de Mac). Le jeu doit compiler, passer les tests et tourner sur les trois.
- **Matériel de tiers dans le dépôt voisin** : le livret, la FAQ, le compendium, les scans et leurs transcriptions ne sont pas redistribuables et ne sont **jamais** dans ce dépôt public. Ils vivent dans le dépôt privé `trollou/mage-knight`, cloné à côté de celui-ci (`../mage-knight/`, même dossier parent ; un fil cloud a `/home/user/iter-arcanum` et `/home/user/mage-knight`). Les chemins `../mage-knight/docs/regles/…` ci-dessous y renvoient. Un contributeur qui n'a pas ce dépôt lit les renvois `MKUE p. 7, 5.a` dans son propre livret : le jeu compile et les tests passent sans lui.
- Règles du jeu : livret `../mage-knight/docs/regles/mkue-rulebook.pdf`, transcrit page par page dans `../mage-knight/docs/regles/livret/` (`p01.md` à `p24.md`). **Lire la transcription, pas le PDF** : partir du sommaire `../mage-knight/docs/regles/livret/README.md`, n'ouvrir que les pages utiles, et le PDF seulement pour une illustration. En cas de doute sur une règle, citer la page et le paragraphe du livret (ex. `MKUE p. 7, 5.a`) plutôt que deviner ; si elle reste ambiguë, demander à Mathieu.
- FAQ : la FAQ officielle (v1.0, 2014, traduction française non officielle) est transcrite dans `../mage-knight/docs/regles/faq/` (`p1.md` à `p3.md`), et le *Compendium of Q&A* de BGG (2026, non officiel, 173 pages) dans `../mage-knight/docs/regles/faq/compendium/`. **Quand le livret ne tranche pas, ou qu'une règle semble en contredire une autre (livret, carte, spec, code)** : chercher la réponse dans la FAQ officielle, puis dans le compendium **par son index** (tableau « Compendium » de `docs/reference/index-par-notion.md`, ou `../mage-knight/docs/regles/faq/compendium/index.md` pour une carte), sans lire le compendium en entier. Citer l'entrée (`Compendium, Combat S34`). Ordre de consultation : livret MKUE et cartes, puis FAQ officielle, puis compendium ; une contradiction avec le livret devient une question ouverte pour Mathieu.
- Texte des cartes, jetons et cartes de site : transcrit dans `../mage-knight/docs/regles/cartes/` (sommaire `README.md`), un fichier par famille. **Le lire au lieu d'ouvrir les scans** de `../mage-knight/docs/regles/scans/`, réservés aux illustrations. Quand une carte a un errata *Lost Legion*, c'est cette version qui fait foi.
- Termes français des cartes : pour vérifier une traduction, photos de cartes françaises dans `../mage-knight/docs/regles/scans/fr/` (mode d'emploi et limites dans le README de `../mage-knight/docs/regles/scans/`).
- Vocabulaire du jeu : `docs/glossaire.md` (livret EN ↔ français ↔ code). Ne pas inventer de synonyme.

## Aides pour démarrer un fil

À lire avant d'explorer le code ou le livret, pour ne pas tout redécouvrir :

- **Planification dans GitHub** (depuis le 2026-11-24) : versions = milestones, grandes fonctionnalités = issues `Epic`, puis `Feature`, `Bug` et `Task` en sous-issues ; le reste à faire se lit dans les issues ouvertes du milestone en cours, par Rank (`docs/issues.md`, Planification). `docs/specs/PLAN.md` est archivé : il ne sert plus qu'à retrouver les jalons et les specs d'avant cette date.
- `docs/reference/index-par-notion.md` : quelles pages et quels paragraphes du livret lire pour une notion (mana, combat, fin du tour…), quelle spec la couvre, et quelle section du compendium FAQ consulter. **Tenir sa colonne Spec à jour** dans la PR qui rédige ou implémente une spec.
- `docs/architecture.md` : crates, modules, types clés, recettes « ajouter une action / un événement / un élément d'écran ». **La mettre à jour** dans la PR qui change ce qu'elle décrit.
- `docs/reference/bac-a-sable.md` : coordonnées `(q, r)` des cases et `(i, j)` des emplacements du bac à sable, avec terrain et site. **La lire au lieu de recalculer** depuis `tiles.ron` (page générée et vérifiée par un test).
- `docs/issues.md` : planifier (milestones, Epics, champs Priority, Effort et Rank), ouvrir, classer (type, labels), traiter et fermer une issue, et lier une spec à son issue. **À lire au début d'un fil d'issue ou de spec.** Une issue `réservé` n'est pas traitée par Claude.
- `docs/verification-manuelle.md` : vérifier l'UI à l'œil sous Linux sans écran avec `scripts/verif-manuelle.sh` (Xvfb, clics, captures), au besoin depuis une partie préparée (`charger <nom>` : combat, Blessures…).
- `docs/qualite/README.md` : rapport de qualité du code (`scripts/qualite.sh`) pour le bilan de fin de spec, et sa référence.

## Méthode de travail : itératif, spec-first, TDD, et bilan

Le cycle est toujours le même :

1. **Spec d'abord.** Une spec dans `docs/specs/`, nommée d'après son issue `Feature` (`issue-<N>-<sujet>.md`, « spec #N » ; `docs/issues.md`), décrit le comportement attendu : règles concernées, cas nominaux, cas limites, hors périmètre. Les règles **renvoient au livret** (`MKUE p. 7, 5.b`) au lieu de le recopier : la spec ne dit que ce qu'elle en fait (voir `docs/specs/_gabarit.md`). Elle est **validée par Mathieu** avant toute suite.
2. **Tests (TDD).** Écrire les tests qui traduisent la spec, les lancer, **vérifier qu'ils échouent** pour la bonne raison : sur une assertion, pas à la compilation (types nouveaux : signatures seules, `todo!()`, d'abord). Commit local `Tests rouges : …` avec les tests en échec, nommés et comptés par `scripts/verifier.sh --rouge`, avant le code.
3. **Code minimal** pour les faire passer.
4. **Refactor** avec les tests au vert.
5. **Vérifier** avant de déclarer une tâche terminée : `fmt`, `clippy` et `test` (voir Commandes), tous au vert.
6. **Rétrospective** Bilan de fin de spec, avant de clore le fil d'implémentation : points à améliorer (méthode, optimisation du code et ses tests, optimisation contexte et tokens…), en s'appuyant sur `scripts/qualite.sh` (taille, complexité, duplication comparées à la référence, puis `--enregistrer` ; `--couverture` en fin de milestone : `docs/qualite/README.md`). Une issue GitHub par point réel, confirmé avec Mathieu, avec le label `bilan` (`docs/issues.md`) ; s'il n'y a rien, le dire explicitement dans le compte rendu.

Règles qui en découlent :

- Jamais de code de production sans test qui l'exige.
- Jamais modifier un test pour le faire passer sans le signaler et le justifier.
- Rien qui ne soit pas dans la spec. Un besoin nouveau → on le signale et on met la spec à jour d'abord.
- Petites étapes, une spec à la fois. Proposer un plan avant une tâche de plus de quelques fichiers (gabarit `docs/specs/plans/_gabarit.md`, qui dit aussi quelles étapes confier à un modèle moins cher).
- **Débogage délégué** : un fil qui doit tâtonner (build ou test rouge inexpliqué, recherche de cause, essais successifs) confie l'enquête à un sous-agent (outil Agent), qui ne renvoie qu'une synthèse : cause, correctif, preuves (commande et sortie utiles). Le contexte du fil reste petit, faute de compactage manuel dans les fils. Chaque PR le note en une ligne : `Sous-agent de débogage : non` ou `oui (n fois) : <sujet>, <utile / inutile>`, pour que Mathieu en suive l'efficacité.
- Toute nouvelle dépendance externe est proposée et justifiée avant d'être ajoutée. Sa licence doit passer `cargo deny check licenses` (`deny.toml`, licences compatibles avec la GPL v3).
- Rédaction et implémentation d'une spec dans deux fils distincts : le fil de rédaction s'arrête quand Mathieu valide la spec (poussée, marquée validée, décisions consignées dans la spec) ; un fil neuf l'implémente sur la même branche, à partir de la spec et de son issue seulement. Une grosse spec (plan de plus de 8 étapes, ou moteur et écran à la fois) s'implémente en deux fils successifs sur la même branche : moteur, contenu et persistance, puis libellés, écran, partie préparée, vérification à l'œil et docs (issue privée 118, découpage dans le plan).
- **Maquette réutilisée** : un fil de rédaction qui modifie le code pour une maquette pousse ce code sur une branche à part `maquette/<n° issue>-<sujet>` (jamais fusionnée) et la cite dans la spec. Le fil d'implémentation écrit d'abord ses tests rouges (TDD inchangé), puis reprend de la maquette ce qui sert au lieu de repartir de zéro.
- Issues : traitées dans l'ordre du milestone en cours (Rank), celles d'un bilan juste après la fusion de la spec, une à la fois et une par fil, sauf si certaines se regroupent naturellement car touchant le même code par exemple (`docs/issues.md`).
- Claude remplit les champs d'issue Effort (s'il est vide) et Rank de chaque issue qu'il crée ou trouve sans eux dans un milestone. Il ne peut ni lire ni écrire le projet GitHub lui-même : tout ce qui guide le travail est porté par l'issue (type, milestone, parent, « blocked by », labels, champs).
- **Points ouverts dans des issues, pas dans le Markdown** : tout point qui reste ouvert au-delà de son fil (sujet reporté, décision provisoire, lecture à vérifier, idée pas encore planifiée) devient une issue (son type, labels `question ouverte` et `réservé`, sans milestone) ; la spec ou la doc n'en garde qu'un renvoi (« issue #N »). Seules les questions d'une spec en cours de rédaction restent dans sa section « Questions ouvertes » jusqu'à la validation (`docs/issues.md`, Points ouverts).

## Architecture

Workspace Cargo qui sépare strictement règles et affichage. C'est la cible : créer les crates au fur et à mesure des besoins, pas d'un bloc.

```
iter-arcanum/
├── Cargo.toml          # workspace
├── crates/
│   ├── arcanum-core/        # règles, état, contenu (cartes, tuiles, ennemis) : pur, sans UI ni I/O
│   ├── arcanum-persistance/ # sauvegarde et chargement des parties
│   ├── arcanum-i18n/        # libellés par langue
│   ├── arcanum-audio/       # son : directeur pur, banque de sons, lecteur Kira ou muet (spec 081)
│   ├── arcanum-players/     # joueurs automatiques (plus tard)
│   └── arcanum-ui/          # client eframe/egui : affichage et saisie uniquement
├── assets/i18n/        # un JSON par langue : fr.json (référence), en.json…
├── assets/sounds/      # musiques, ambiances, effets en Ogg Vorbis ; licence de chaque fichier dans LICENSES.md
├── scripts/            # outils de développement (verifier.sh, vérification manuelle sous Xvfb, empaquetage)
├── packaging/          # modèles d'empaquetage des releases (Info.plist du paquet macOS)
├── docs/
│   ├── architecture.md # carte du code et recettes
│   ├── issues-privees.md # titres des issues et PR du dépôt privé citées (« issue privée N »)
│   ├── reference/      # nos pages de référence : index-par-notion.md, bac-a-sable.md (généré), tuiles.md, decks-de-depart.md
│   ├── specs/          # une spec par fonctionnalité (issue-<N>-<sujet>.md) ; PLAN.md archivé
│   ├── decisions/      # décisions d'architecture (ADR)
│   └── glossaire.md
└── .claude/rules/      # règles propres à chaque crate, chargées à la demande
```

- Dépendances : `arcanum-ui` → `arcanum-persistance`, `arcanum-i18n`, `arcanum-audio`, `arcanum-players` → `arcanum-core`. Jamais l'inverse ; `arcanum-core` ne dépend de rien du projet.
- **Son** : `arcanum-audio` écoute la partie (événements filtrés pour le joueur) et ne la change jamais ; seul `main.rs` ouvre la carte son. Voir `docs/decisions/0010-audio.md`.
- **Event sourcing léger** : l'état ne change que par des événements historisés, produits par le moteur à partir d'actions validées. Voir `docs/decisions/0001-event-sourcing.md`.
- **Joueurs enfichables** : humain, règles, heuristique, MCTS ou LLM passent tous par le trait `Player` (vue filtrée + actions légales → choix). Voir `docs/decisions/0002-interface-joueurs.md`.
- **i18n** : aucun texte affiché hors des fichiers `assets/i18n/*.json`. Voir `docs/decisions/0003-internationalisation.md`.
- **Portabilité** : `PathBuf` partout, pas de séparateur en dur, code propre à un OS isolé derrière `#[cfg]`. Voir `docs/decisions/0004-portabilite.md`.
- Contenu du jeu (cartes, decks, tuiles, puis ennemis, sites…) décrit dans des fichiers RON de `crates/arcanum-core/content/`, embarqués à la compilation et validés par des tests. Identifiants et mécaniques restent en Rust, comme les scénarios. Voir `docs/decisions/0005-contenu-en-donnees.md`.
- Erreurs typées (`thiserror` dans les bibliothèques). Pas de `unwrap()`/`expect()` hors tests, sauf invariant justifié en commentaire.
- Toute nouvelle décision d'architecture fait l'objet d'un ADR dans `docs/decisions/`, validé par Mathieu.

## Tests

- Tests unitaires à côté du code (`#[cfg(test)]`), scénarios de règles dans `crates/arcanum-core/tests/`.
- Parcours de bout en bout (`crates/arcanum-core/tests/parcours_*.rs`) et vérification à l'œil de l'UI ; pas de tests d'interaction egui ni de captures de référence. Un test vérifie un comportement de la spec, pas une mise en page. Voir `docs/decisions/0006-strategie-de-tests.md`.
- Un test = un comportement, nommé pour le décrire (ex. `move_into_forest_costs_three_points_by_day`).
- Chaque règle implémentée a au moins un test qui cite la page du livret en commentaire (`// MKUE p. 12`).
- Le hasard passe par un générateur à graine injectable : les tests sont déterministes.

## Conventions

- Identifiants du code en **anglais**, termes de jeu selon `docs/glossaire.md`.
- Specs, docs, commentaires et échanges en **français**. Commentaires courts, sur le *pourquoi*.
- `rustfmt` par défaut ; aucun warning `clippy` toléré.

## Commandes

Valables sous PowerShell et sous Linux (sous Linux, la compilation demande `libasound2-dev` et `pkg-config` pour le son ; `scripts/verifier.sh` les installe s'ils manquent ; sous macOS, rien à installer) :

```
cargo build
cargo test --workspace
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo run -p arcanum-ui
```

Dans un fil, vérifier avec `scripts/verifier.sh` (bash, donc Git Bash sous Windows) plutôt qu'avec les trois commandes : il lance `fmt`, `clippy` et `test` et n'affiche qu'une ligne par étape verte, et seulement les erreurs sinon (842 lignes de `cargo test` deviennent 3). Pour un test précis : `cargo test -q -p <crate> <nom>`. Pour le commit `Tests rouges : …`, `scripts/verifier.sh --rouge` liste et compte les tests en échec de tous les fichiers de test (`--no-fail-fast` : `cargo test --workspace` seul s'arrête au premier fichier rouge et fausse le compte), et signale un échec de compilation, qui n'est pas un rouge. Pendant une étape, le limiter à la crate ou au fichier de test touché (`scripts/verifier.sh --rouge -p arcanum-ui --test preferences` : seuls cette crate et ce fichier sont compilés et lancés, au lieu des quatre minutes du passage complet, issue privée 359) ; le push refait le passage complet.

## Git et intégration continue

- Dépôt de travail : `trollou/iter-arcanum` sur GitHub, **public** depuis la spec 082 (code, assets, nos documents, issues). Le dépôt privé `trollou/mage-knight` n'est plus qu'une bibliothèque en lecture seule (matériel de tiers) et l'archive de l'historique : un renvoi « issue privée N » ou « PR privée N » y désigne une issue ou une PR, dont le titre est dans `docs/issues-privees.md`.
- **Jamais de matériel de tiers ici** : rien de `../mage-knight/docs/regles/` ne se copie dans ce dépôt (ni page du livret, ni scan, ni extrait long de la FAQ ou du compendium) ; on y renvoie (`MKUE p. 7, 5.b`, `Compendium, Combat S34`). Deux barrières : `.gitignore` (`docs/regles/`, `*.pdf`) et le test `crates/arcanum-core/tests/depot_public.rs`, lancé avant chaque push et par la CI. Une fuite poussée resterait dans l'historique public.
- Jamais de commit direct sur `main`. Une branche par spec ou par tâche, puis une pull request vers `main`.
- Nom de branche parlant, en minuscules et sans accents, créé avant le premier commit (jamais le nom automatique `claude/…`) : `feature/<n° issue>-<sujet>` pour une spec (rédaction et implémentation sur la même branche, ex. `feature/32-cartes-des-fenetres`), `fix/<n° issue>-<sujet>` pour une issue (ex. `fix/25-defausse-main-vide`), `docs/<sujet>` ou `chore/<sujet>` pour la doc ou l'outillage.
- **Toute PR porte un label de catégorie** dès sa création, pour les notes de version : `changement`, `correction` ou `outillage` (`docs/issues.md`, Labels).
- **Toute PR porte `Closes #N`** (ou `Fixes #N`) dans sa description, dès sa création, pour chaque issue qu'elle règle, y compris la PR de rédaction d'une spec, qui deviendra celle de son implémentation : GitHub lie ainsi l'issue à la PR (suivi du projet) et la ferme à la fusion. Après la fusion, vérifier que l'issue est fermée ; sinon la fermer en renvoyant à la PR (`docs/issues.md`).
- Ouvrir la PR **en brouillon** une fois `scripts/verifier.sh` vert en local, et ne la passer « prête » qu'à la toute fin (dernier fil d'une spec en plusieurs fils) : un brouillon ne lance pas la CI.
- Seul Mathieu fusionne une pull request, et seulement si la CI est verte, sauf la PR d'une issue sans question ouverte, que le fil fusionne lui-même (`docs/issues.md`). Pas de `push --force` ni de réécriture d'historique sur `main`.
- La CI (`.github/workflows/ci.yml`) tourne sous Linux, Windows et macOS sur chaque PR prête qui ne touche pas que la doc (filtre de chemins, issue #72), et sur `main` à chaque fusion ; un brouillon ne lance rien. Détail (déclencheurs, cache, Dependabot, suppression des branches à la fusion) : `docs/ci.md`, à mettre à jour dans la PR qui change `.github/`. Release (tag `vX.Y.Z`, correctif) : `docs/release.md`.
- Messages de commit en français, à l'impératif, une première ligne courte qui cite la spec ou l'issue concernée (ex. `Implémente la spec #32 Cartes des fenêtres`).
- Un commit = une étape cohérente (spec, tests, implémentation, refactor). Ne pas mélanger plusieurs specs dans un même commit. Les commits locaux peuvent être rouges (tests en phase TDD) ; on ne pousse que du vert.
- Fins de ligne en LF, imposées par `.gitattributes`. `target/` n'est jamais versionné.

## Assets et droits

Licences (Mathieu, 2026-11-12) : code sous GPL-3.0-or-later (`LICENSE`), visuels et libellés de `assets/` sous CC BY-SA 4.0 (`assets/LICENSE`) ; ce qui vient du jeu d'origine et les documents de référence du dépôt privé (`../mage-knight/docs/regles/`) ne sont couverts par aucune licence du projet (`NOTICE`).

Les sons de `assets/sounds/` ne sont pas sous CC BY-SA 4.0 : chacun garde la licence de son auteur (CC0, CC BY, CC BY-SA ; jamais NC, spec #70), avec une ligne par fichier dans `assets/sounds/LICENSES.md`, vérifiée par un test (spec 081).

Aucun visuel ni texte officiel de Mage Knight dans le jeu. Les visuels sont créés par nous : SVG dessinés par le code (plateau, icônes) et illustrations générées sans rien nommer du jeu ni lui reprendre d'image (cartes et unités ; ADR 0009), dans `assets/images/` : ils peuvent évoquer l'ambiance et les couleurs du jeu de base pour ne pas dérouter les joueurs, mais rien n'est décalqué ni repris des scans ou du livret. Le livret et les scans (`../mage-knight/docs/regles/`) servent uniquement de référence de règles.

## Périmètre

Solo d'abord, avec le joueur humain. Les joueurs automatiques viendront ensuite par le trait `Player`, chacun avec sa spec. Pas de jeu en réseau.

## Hooks

Définis dans `.claude/settings.json` (exécutés par bash, donc Git Bash sous Windows) :

- Après chaque modification d'un fichier `.rs` : `cargo fmt --all`.
- Avant chaque `git push` : `scripts/verifier.sh`, soit `fmt --check`, `clippy` et `test` (garde-fou du dépôt public compris) (sauf sous Windows, où l'antivirus bloque les exécutables de test). Si l'un échoue, le push est bloqué : corriger, ne pas contourner.
