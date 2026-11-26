# Faire une release

Procédure des versions d'Iter Arcanum (spec #33, décisions D1 à D14 de l'issue #33 et de la spec). Le workflow
`.github/workflows/release.yml` fait le travail ; cette page dit quoi pousser et quoi relire. Seul Mathieu pose les tags.

## Ce que fait le workflow

Un tag `v*` poussé lance, dans l'ordre :

1. **contrôles** : la version du tag est celle de `Cargo.toml` (`scripts/controler-version.sh`), puis
   `cargo deny check` complet (licences, interdits, sources, avis de sécurité), bloquant ;
2. **vérification** : `ci.yml` appelé tel quel, `fmt`, `clippy` et `test` sur les trois OS ;
3. **construction** sur chaque OS, contrôles propres à l'OS, signature de l'exe Windows (spec #70, ci-dessous),
   empaquetage (`scripts/empaqueter.sh`) et essai de lancement de l'archive dépaquetée (`scripts/essai-lancement.sh`) ;
4. **publication** : `SHA256SUMS.txt`, puis la release GitHub avec les trois archives et les notes générées.

| Archive | Contenu |
|---|---|
| `iter-arcanum-X.Y.Z-windows-x86_64.zip` | `iter-arcanum.exe` (C runtime statique, signé sur un tag final), `assets/`, `LICENSE`, `NOTICE`, `README.md` |
| `iter-arcanum-X.Y.Z-linux-x86_64.tar.gz` | `iter-arcanum` (glibc 2.35), `assets/`, `LICENSE`, `NOTICE`, `README.md` |
| `iter-arcanum-X.Y.Z-macos-universal.dmg` | `Iter Arcanum.app` (Apple Silicon et Intel, macOS 11, signature ad hoc), textes, lien vers `/Applications` |

**Essai à blanc** (rien n'est publié, les archives sont les artefacts de l'exécution) : onglet Actions, « Release »,
« Run workflow » depuis `main` ; ou une PR prête qui touche l'empaquetage (`release.yml`, `scripts/empaqueter.sh`,
`scripts/essai-lancement.sh`, `scripts/controler-version.sh`, `packaging/`, `crates/arcanum-ui/build.rs` et `build/`). Détail des
déclencheurs : `docs/ci.md`.

En local (Linux, fil cloud) : `cargo build -p arcanum-ui --release`, puis
`scripts/empaqueter.sh linux 0.1.0 /tmp/sortie` et `xvfb-run -a scripts/essai-lancement.sh linux /tmp/sortie/*.tar.gz`.

## Faire une release `X.Y.0`

1. Sur `main`, par une PR : `version = "X.Y.0"` dans `[workspace.package]` de `Cargo.toml` (au lieu de `X.Y.0-dev`, D2).
2. Tag de test : `git tag vX.Y.0-rc.1 && git push origin vX.Y.0-rc.1` sur le commit fusionné. La release est publiée
   aussitôt, marquée « pre-release » (D3). Télécharger les archives et les essayer ; un défaut se corrige par une PR,
   puis `-rc.2`, etc.
3. Tag final : `git tag vX.Y.0 && git push origin vX.Y.0`. **Approuver la demande de signature** dans SignPath (lien
   dans le journal du job Windows, voir Signature Windows), puis vérifier sur Windows 11 avec Smart App Control que l'exe
   de l'archive du brouillon se lance et que ses propriétés montrent la signature. La release est créée en **brouillon** : écrire le résumé en
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

1. `tests/temoins/vX.Y.Z/` : les parties préparées du tag (`scripts/parties/*.json`), plus les deux parcours de fin de jalon
   (`parcours_j6.rs`) joués sur le code du tag et sauvegardés en fin de partie (copie de travail du tag, où
   `parcours_j6.rs` et `support/` sont recopiés dans `crates/arcanum-persistance/tests/` avec, en fin de
   `parcours_conquete_en_solitaire` et d'`episode`, un `save` de `SaveGame::from_runner`), et une vraie partie de Mathieu
   en option. Les témoins ne sont jamais régénérés.
2. `tests/format/vX.Y.Z.yaml` : copie de `tests/format/sauvegarde.yaml`.
3. `PREVIOUS_RELEASE_FORMAT` (`src/migration.rs`) prend la valeur de `EVENT_FORMAT_VERSION`.
4. S'il y a alors plus de trois versions gardées : supprimer les témoins et l'empreinte de la plus ancienne, donner à
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

## Signature Windows

Spec #70 : l'exe Windows d'un tag final est signé (Authenticode, horodaté) par la **SignPath Foundation**, gratuitement,
sans clé ni certificat chez Mathieu. Éditeur affiché : « SignPath Foundation » ; le nom du jeu est dans les métadonnées de
l'exe (ressource de version embarquée par `build.rs`). Cette signature remplace la décision D6 de l'issue #33 pour Windows ;
macOS garde sa signature ad hoc. Politique publiée : section « Code signing policy » du `README.md`, rappelée en tête des
notes de chaque release.

### Ce que fait le job

Dans `construire`, entrée Windows, après la construction et `dumpbin` :

1. contrôle de la ressource de version (`ProductName` « Iter Arcanum », `ProductVersion` = version de `Cargo.toml`), sur
   toute exécution ;
2. choix de la politique :

   | Exécution | Politique SignPath | Approbation |
   |---|---|---|
   | tag final `vX.Y.Z` | `release-signing` | Mathieu, dans SignPath |
   | tag de test `vX.Y.Z-rc.N` | aucune signature, sans avertissement (Q2) | — |
   | essai à blanc (onglet Actions) | `test-signing` (certificat de test, non reconnu par Windows) | aucune |
   | PR, ou secret `SIGNPATH_API_TOKEN` absent | aucune signature, avertissement `exe Windows non signé` (Q4) | — |

3. envoi de `iter-arcanum.exe` seul (artefact non compressé) par l'action `signpath/github-action-submit-signing-request` ;
   elle écrit le lien de la demande (« You can view the signing request here: … ») puis attend au plus **2 heures** ;
4. l'exe signé remplace l'exe construit, puis `Get-AuthenticodeSignature` : `Valid`, certificat « SignPath Foundation »
   et horodatage pour `release-signing` ; une signature quelconque pour `test-signing`. Sinon le job échoue ;
5. empaquetage, essai de lancement et `SHA256SUMS.txt` portent sur l'exe signé.

### Approuver, refuser, relancer

- **Approuver** : ouvrir le lien du journal (ou la liste des demandes du projet `iter-arcanum` dans SignPath), vérifier
  que la demande vient du bon tag et du bon commit, « Approve ».
- **Plus de 2 heures** sans approbation : le job échoue. Approuver la demande, puis « Re-run failed jobs » dans l'onglet
  Actions (une nouvelle demande part ; approuver celle-là).
- **Refus**, SignPath injoignable ou jeton expiré : le job échoue, aucune release n'est créée (pas de repli vers un exe
  non signé quand le secret existe). Corriger (PR, ou nouveau jeton dans les secrets), puis retagger (`-rc` ou correctif).
- Certificat de la fondation renouvelé : rien à changer, le contrôle vérifie le nom, pas l'empreinte.

### Réglages SignPath, pas à pas

Une fois admis par la fondation :

1. Installer l'application GitHub **SignPath** sur le dépôt `trollou/iter-arcanum`.
2. Dans SignPath, organisation de la fondation : créer le projet de slug **`iter-arcanum`**, relié au dépôt GitHub
   (Trusted Build System « GitHub.com »).
3. Configuration d'artefact de slug **`exe-windows`** : un fichier PE seul, nom de produit imposé « Iter Arcanum » :

   ```xml
   <artifact-configuration xmlns="http://signpath.io/artifact-configuration/v1">
     <pe-file product-name="Iter Arcanum">
       <authenticode-sign/>
     </pe-file>
   </artifact-configuration>
   ```

4. Politiques de signature : **`release-signing`** (certificat de la fondation, approbation manuelle par Mathieu) et
   **`test-signing`** (certificat de test, sans approbation). Aucune règle `disallow_reruns`, pour pouvoir relancer après
   l'attente de 2 heures.
5. Un utilisateur CI (Submitter des deux politiques) et son jeton d'API ; MFA activée sur le compte SignPath.
6. Dans le dépôt GitHub (Settings > Secrets and variables > Actions) : le secret **`SIGNPATH_API_TOKEN`** (ce jeton) et
   la variable **`SIGNPATH_ORGANIZATION_ID`** (identifiant de l'organisation, dans l'URL de SignPath).
7. Essai à blanc depuis `main` (onglet Actions) : il signe avec `test-signing` et vérifie toute la chaîne avant le
   premier tag final signé.

### Étapes de Mathieu

| Étape | État |
|---|---|
| MFA sur GitHub | à vérifier |
| Règle de branche sur `main` (PR obligatoire, pas de `push --force`) | à vérifier |
| Demande d'admission sur `signpath.org/apply` (après la fusion de la spec #70 : la politique doit être en ligne) | à faire |
| Réglages SignPath, secret et variable du dépôt (ci-dessus) | après l'admission |
| Essai à blanc signé `test-signing` | après les réglages |
| Approuver la demande de chaque tag final | à chaque release |

### Plan B

Si la fondation refuse le dossier : certificat individuel open source (Certum), dans une nouvelle issue qui reprend les
contrôles du job avec un autre outil de signature (spec #70, L4). Azure Artifact Signing reste fermé aux particuliers
en France (spec #70, comparaison).

## Relancer un tag

`gh release create` refuse une release qui existe déjà (L2). Supprimer la **release** (pas le tag) dans la page
« Releases », puis relancer l'exécution échouée depuis l'onglet Actions (« Re-run all jobs »). Si le tag lui-même est
faux : supprimer la release et le tag (`git push origin :refs/tags/vX.Y.Z`), corriger, retagger.

## Un avis de `cargo deny`

Un nouvel avis de sécurité le jour du tag arrête la release (D5, L5). Par une PR : mettre à jour la dépendance, ou, si
l'avis ne touche pas le jeu, l'ajouter à `ignore` dans `[advisories]` de `deny.toml` avec sa justification et une issue
qui le traite (exemple : `RUSTSEC-2026-0543`, `chrono` non maintenu, issue #55). Puis relancer le tag.
