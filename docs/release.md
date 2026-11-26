# Faire une release

Procédure des versions d'Iter Arcanum (spec #33, décisions D1 à D14 de l'issue #33 et de la spec). Le workflow
`.github/workflows/release.yml` fait le travail ; cette page dit quoi pousser et quoi relire. Seul Mathieu pose les tags.

## Ce que fait le workflow

Un tag `v*` poussé lance, dans l'ordre :

1. **contrôles** : la version du tag est celle de `Cargo.toml` (`scripts/controler-version.sh`), puis
   `cargo deny check` complet (licences, interdits, sources, avis de sécurité), bloquant ;
2. **vérification** : `ci.yml` appelé tel quel, `fmt`, `clippy` et `test` sur les trois OS ;
3. **construction** sur chaque OS, empaquetage (`scripts/empaqueter.sh`), contrôles propres à l'OS et essai de lancement
   de l'archive dépaquetée (`scripts/essai-lancement.sh`) ;
4. **publication** : `SHA256SUMS.txt`, puis la release GitHub avec les trois archives et les notes générées.

| Archive | Contenu |
|---|---|
| `iter-arcanum-X.Y.Z-windows-x86_64.zip` | `iter-arcanum.exe` (C runtime statique), `assets/`, `LICENSE`, `NOTICE`, `README.md` |
| `iter-arcanum-X.Y.Z-linux-x86_64.tar.gz` | `iter-arcanum` (glibc 2.35), `assets/`, `LICENSE`, `NOTICE`, `README.md` |
| `iter-arcanum-X.Y.Z-macos-universal.dmg` | `Iter Arcanum.app` (Apple Silicon et Intel, macOS 11, signature ad hoc), textes, lien vers `/Applications` |

**Essai à blanc** (rien n'est publié, les archives sont les artefacts de l'exécution) : onglet Actions, « Release »,
« Run workflow » depuis `main` ; ou une PR prête qui touche l'empaquetage (`release.yml`, `scripts/empaqueter.sh`,
`scripts/essai-lancement.sh`, `scripts/controler-version.sh`, `packaging/`, `crates/arcanum-ui/build.rs`). Détail des
déclencheurs : `docs/ci.md`.

En local (Linux, fil cloud) : `cargo build -p arcanum-ui --release`, puis
`scripts/empaqueter.sh linux 0.1.0 /tmp/sortie` et `xvfb-run -a scripts/essai-lancement.sh linux /tmp/sortie/*.tar.gz`.

## Faire une release `X.Y.0`

1. Sur `main`, par une PR : `version = "X.Y.0"` dans `[workspace.package]` de `Cargo.toml` (au lieu de `X.Y.0-dev`, D2).
2. Tag de test : `git tag vX.Y.0-rc.1 && git push origin vX.Y.0-rc.1` sur le commit fusionné. La release est publiée
   aussitôt, marquée « pre-release » (D3). Télécharger les archives et les essayer ; un défaut se corrige par une PR,
   puis `-rc.2`, etc.
3. Tag final : `git tag vX.Y.0 && git push origin vX.Y.0`. La release est créée en **brouillon** : écrire le résumé en
   tête des notes générées, puis « Publish release ». Les notes listent les PR fusionnées depuis la release finale
   précédente (le workflow supprime d'abord les pre-releases `-rc.N` de la version, pas leurs tags), rangées par label de
   PR en « Changements », « Corrections de bugs » et « Outillage et documentation » (`.github/release.yml`, issue #64).
4. Par une PR : `version = "X.Y+1.0-dev"` (D2). Un tag posé sur une version `-dev` est refusé. La même PR garde les
   sauvegardes de la version publiée (voir ci-dessous).

## Garder les sauvegardes d'une version publiée

Une sauvegarde de l'une des trois dernières versions publiées se charge avec la version courante (spec #52, ADR 0001).
Avant le tag, rien de plus que la CI : les tests `temoins` et `format` d'arcanum-persistance vérifient que les sauvegardes
des versions gardées se chargent. Juste après le tag, dans la PR qui passe à `X.Y+1.0-dev`
(`crates/arcanum-persistance/`) :

1. `scripts/temoins-release.sh vX.Y.Z` (issue #84) écrit, depuis une copie de travail du tag, `tests/temoins/vX.Y.Z/` :
   les parties préparées du tag (`scripts/parties/*.json`) et les parcours de fin de jalon joués sur le code du tag,
   sauvegardés en fin de partie (le script dit lesquels ; au jalon suivant, changer ses variables `PARCOURS` et
   `TEST_TEMOINS`), et `tests/format/vX.Y.Z.yaml`, copie de `tests/format/sauvegarde.yaml` du tag. Il refuse d'écraser
   des témoins existants : ils ne sont jamais régénérés. Une vraie partie de Mathieu s'y ajoute en option.
2. `PREVIOUS_RELEASE_FORMAT` (`src/migration.rs`) prend la valeur de `EVENT_FORMAT_VERSION`.
3. S'il y a alors plus de trois versions gardées : supprimer les témoins et l'empreinte de la plus ancienne, donner à
   `OLDEST_SUPPORTED_FORMAT` le format de la nouvelle plus ancienne, et retirer de `MIGRATIONS` les migrations qui partent
   d'un format plus ancien. Le test `migration_chain_matches_supported_versions` vérifie l'accord.

Un correctif `X.Y.Z+1` compte comme une version publiée s'il change le format ; sinon, ses sauvegardes sont celles de
`X.Y.Z`.

## Faire un correctif `X.Y.Z+1`

1. Au premier correctif d'une version : `git branch release/X.Y vX.Y.0 && git push origin release/X.Y` (D1). Les suivants
   réutilisent la branche.
2. Le correctif est d'abord fusionné dans `main` (PR habituelle), puis repris par `git cherry-pick` sur `release/X.Y`,
   avec `version = "X.Y.Z+1"` dans `Cargo.toml`.
3. Tag sur `release/X.Y` : `vX.Y.Z+1-rc.1` si besoin, puis `vX.Y.Z+1`, comme ci-dessus.

Le workflow ne vérifie pas la branche d'un tag (spec #33, L3) : ne tagger que `main` ou `release/X.Y`.

## Relancer un tag

`gh release create` refuse une release qui existe déjà (L2). Supprimer la **release** (pas le tag) dans la page
« Releases », puis relancer l'exécution échouée depuis l'onglet Actions (« Re-run all jobs »). Si le tag lui-même est
faux : supprimer la release et le tag (`git push origin :refs/tags/vX.Y.Z`), corriger, retagger.

## Un avis de `cargo deny`

Un nouvel avis de sécurité le jour du tag arrête la release (D5, L5). Par une PR : mettre à jour la dépendance, ou, si
l'avis ne touche pas le jeu, l'ajouter à `ignore` dans `[advisories]` de `deny.toml` avec sa justification et une issue
qui le traite (exemple : `RUSTSEC-2026-0543`, `chrono` non maintenu, issue #55). Puis relancer le tag.
