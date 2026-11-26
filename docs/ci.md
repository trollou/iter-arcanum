# Intégration continue et dépôt

Détail de la CI et de l'automatisation du dépôt public `trollou/iter-arcanum`. Les règles que tout fil doit
connaître (branches, PR, fusion) restent dans `CLAUDE.md`, section « Git et intégration continue » ; cette page
sert aux fils qui touchent `.github/` ou qui s'étonnent d'un check.

## Ce que lance la CI

La CI (`.github/workflows/ci.yml`) lance `fmt`, `clippy` et `test` sous Linux, Windows et macOS (`macos-latest`, Apple Silicon ; spec #7). Sur un dépôt public,
les minutes GitHub Actions sont illimitées, Windows et macOS compris (spec 082, Q6) : plus de passage de nuit, qui
économisait le quota du dépôt privé (issues privées 48 et 305). Le filtre de chemins, retiré avec, est revenu pour la
doc seule (issue #72) : attendre trois OS pour une ligne de README ne sert à rien.

- **Sur une pull request prête**, Linux, Windows et macOS au complet, sauf si elle ne touche que de la doc. Le job Linux installe d'abord `libasound2-dev` et `pkg-config`, sans
  lesquels le son (cpal, ALSA) ne compile pas (spec 081) ; `scripts/verifier.sh` fait de même dans un fil. Le job
  macOS n'installe rien (CoreAudio est fourni).
- **Une PR de doc seule ne lance rien** (filtre `paths` du déclencheur `pull_request`, issue #72) : le Markdown
  (`README.md`, `CLAUDE.md`, `docs/**.md`…), `.claude/`, `LICENSE` et `NOTICE`. Restent dans la CI les fichiers lus
  par un test : `docs/reference/bac-a-sable.md` (et son SVG), `docs/verification-manuelle.md`, tout `assets/`
  (`assets/sounds/LICENSES.md`), ainsi que les fichiers non Markdown de `docs/` (images des specs, PDF égaré) et
  `docs/regles/`, que le garde-fou `depot_public.rs` inspecte. **Un test qui se met à lire un autre fichier de doc :
  l'ajouter aux exceptions de `ci.yml`.** Aucun check n'est exigé par la protection de `main` (le jeu de règles ne
  fait qu'interdire suppression et push forcé) : une PR filtrée reste fusionnable. CodeQL (`codeql.yml`) saute de
  même le Markdown, `.claude/`, `LICENSE` et `NOTICE`. Avant le push, `scripts/verifier.sh` (hook) garde le
  garde-fou du dépôt public pour toute PR.
- **Un brouillon ne lance rien** : la CI part quand la PR passe « prête » (`ready_for_review`), puis à
  chaque push tant qu'elle reste prête (l'exécution précédente est annulée). Les fils ouvrent leur PR en
  brouillon et la passent prête à la toute fin, une fois le dernier fil de la spec fini.
- **Sur `main`, à chaque fusion**, les trois OS au complet ; c'est aussi ce passage qui enregistre le cache.
- **À la main** (onglet Actions), sur les trois OS, au complet.
- **Garde-fou du dépôt public** : `cargo test` comprend `crates/arcanum-core/tests/depot_public.rs`, qui
  refuse tout matériel de tiers suivi par git (`docs/regles/`, PDF, images hors de `assets/` et
  `docs/specs/images/`, fichier de plus de 5 Mo ; spec 082, C4 et C5).

## Capture macOS

`.github/workflows/capture-macos.yml` ne tourne jamais tout seul : onglet Actions, « Capture macOS », « Run workflow »,
choisir la branche et la langue (`fr-FR` ou `en-US`). Il compile le jeu en release sur `macos-latest`, règle la langue
préférée du runner, lance le jeu comme un jeu déployé (sans `cargo`, sans `--lang` ni variable de dossier), prend une
capture d'écran au bout de 20 s et publie en artefact `capture-macos-<langue>` : `ecran.png`, les logs de
`~/Library/Logs/iter-arcanum/` (langue détectée, dossiers) et la liste de `~/Library/Application Support/iter-arcanum/`.
Si le runner refuse l'enregistrement de l'écran, la capture sort noire ; les logs restent probants (spec #7, Q3).

## Release

`.github/workflows/release.yml` construit, empaquette et publie le jeu (spec #33 ; procédure complète :
`docs/release.md`). Déclencheurs :

- **tag `v*`** : release. Brouillon pour `vX.Y.Z`, pre-release pour `vX.Y.Z-rc.N` ;
- **à la main** (onglet Actions, depuis `main`) : essai à blanc, le job `publier` ne tourne pas ;
- **PR prête qui touche l'empaquetage** (`release.yml`, `scripts/empaqueter.sh`, `scripts/essai-lancement.sh`,
  `scripts/controler-version.sh`, `packaging/**`, `crates/arcanum-ui/build.rs`) : essai à blanc (spec #33, D11), car
  GitHub refuse `workflow_dispatch` tant que le fichier n'est pas sur `main`. Une PR ordinaire ne le lance pas.

Jobs : `controles` (version du tag, `cargo deny check` complet, bloquant), `verifier` (`ci.yml` appelé par
`workflow_call` ; sauté sur une PR, où `ci.yml` tourne déjà), `construire` (matrice Windows, Ubuntu 22.04, macOS :
construction, empaquetage, `dumpbin` sous Windows, essai de lancement, archive en artefact `archive-<os>`), `publier`
(tag seulement, seul job en `contents: write`). Pas de cache enregistré. Les archives d'un essai à blanc se téléchargent
depuis la page de l'exécution (pas depuis un fil cloud : proxy 403).

## Contrôle qualité

Les outils de qualité (couverture `llvm-cov`, duplication `jscpd`, `cargo machete`, `cargo deny`) ne tournent pas
dans la CI ordinaire (`cargo deny check` tourne seulement dans `release.yml`, spec #33) : `scripts/qualite.sh` les lance en fin de spec, la couverture en fin de jalon
(`docs/qualite/README.md`). Décision de Mathieu, 2026-11-17 (issue privée 305), prise pour le quota du dépôt privé.

## Cache

Le cache `rust-cache` n'est enregistré que par `main` (fusion ou à la main) ; les PR le lisent sans
l'enregistrer, pour ne pas saturer le quota de 10 Go du dépôt.

Les symboles de débogage sont réduits aux numéros de ligne (`CARGO_PROFILE_DEV_DEBUG: line-tables-only`,
bloc `env:` du workflow) : `target/` et le cache sont bien plus petits, les éditions de liens plus rapides.
Sans ce réglage, le disque du runner Linux ne suffisait plus (PR privée 222, « No space left on device ») ;
perdu à la création du dépôt public, il a été remis par l'issue #57.

## Branches

GitHub supprime la branche à la fusion de la PR (« Automatically delete head branches »). Pour un plan à
plusieurs PR intermédiaires, restaurer la branche depuis la PR fusionnée ou la recréer depuis `main`.

## Dependabot

Dependabot (`.github/dependabot.yml`) ouvre chaque dimanche soir des PR groupées de mise à jour des
dépendances Cargo (compatibles, egui, majeures), et le 1er du mois une PR pour les actions GitHub. Elles
suivent les mêmes règles que les autres : CI verte, fusion par Mathieu.
