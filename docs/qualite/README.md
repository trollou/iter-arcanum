# Qualité du code

`scripts/qualite.sh` mesure la qualité du code et la compare à une référence versionnée
(`reference.txt`, ici). C'est un **rapport, jamais bloquant** : il alimente le bilan de fin de spec
(CLAUDE.md, étape 6) et ne tourne pas dans la CI, pour ne consommer aucune minute.

## Quand le lancer

- **Fin de spec**, avant le bilan : `scripts/qualite.sh`, environ 10 secondes une fois le code compilé.
  Une mesure qui se dégrade nettement par rapport à la référence (fichier qui grossit au-delà du
  seuil, nouvelle fonction trop longue, clones ajoutés) est un point à discuter au bilan. Puis
  `scripts/qualite.sh --enregistrer`, et commiter `reference.txt` avec le bilan : la spec suivante se
  compare à celle-ci.
- **Fin de jalon**, en plus : `scripts/qualite.sh --couverture --enregistrer`. La couverture rejoue tous
  les tests instrumentés (4 à 5 minutes, plus d'1 Go de compilation, libéré à la fin) : trop long pour
  chaque spec. Sans `--couverture`, `--enregistrer` garde la couverture de la référence.

Le détail (fichiers trop longs, fonctions trop longues ou complexes, clones du code de production)
est écrit dans `target/qualite/details.txt` ; seules les mesures s'affichent.

## Mesures

| Mesure | Outil | Seuil |
|---|---|---|
| `lignes_src`, `lignes_tests` | `wc -l` sur `crates/*/src` et `crates/*/tests` | |
| `plus_gros_fichier`, `fichiers_trop_longs` | `wc -l` | 2 000 lignes (`SEUIL_FICHIER` du script) |
| `fonctions_trop_longues`, `plus_longue_fonction` | lint clippy `too_many_lines` | 100 lignes |
| `fonctions_trop_complexes`, `plus_complexe_fonction` | lint clippy `cognitive_complexity` | 25 |
| `duplication_src_*`, `duplication_tests_*` | `jscpd`, blocs d'au moins 70 jetons | |
| `dependances_inutilisees` | `cargo machete` | |
| `couverture_*` (option) | `cargo llvm-cov`, pourcentage de lignes | |

Les seuils des fonctions sont dans `scripts/qualite/clippy.toml`, lu seulement par ce script
(`CLIPPY_CONF_DIR`) : le clippy habituel de `verifier.sh` et de la CI n'est pas touché, et ces deux
lints restent désactivés hors du rapport.

Lecture : les gros `match` sur les événements (`apply_event`, `filter_event`) sont en partie
naturels avec l'event sourcing (ADR 0001) ; la mesure sert à voir la tendance, pas à viser zéro.
La duplication des tests vient surtout de la préparation des parties dans `crates/*/tests`.

## Outils

Un outil absent est sauté, avec sa commande d'installation. Dans un fil cloud :

```
npm install -g jscpd
cargo install --locked cargo-machete --version '~0.9'   # 0.10 demande un rustc plus récent
cargo install --locked cargo-llvm-cov                     # pour --couverture
rustup component add llvm-tools-preview                   # pour --couverture
```

`cargo deny check` (licences, sources, avis de sécurité ; `deny.toml`) se lance à part.

Pistes écartées ou reportées (fil du 2026-11-17) : `rustquty`, orchestrateur trop jeune (v0.4) qui ne
lit pas nos résultats de tests ; `cargo mutants`, très coûteux, à essayer plus tard sur le seul diff
d'une spec (`--in-diff`) ; l'intégration à la CI de nuit est discutée dans l'issue privée 305.
