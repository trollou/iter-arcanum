# #70 — Signer l'exécutable Windows

- Statut : à valider
- Issue : #70 (milestone v0.2.0)
- Dépend de : spec #33 (mécanisme de release, fusionnée : PR #54) ; spec 004 (icône embarquée par `build.rs`) ;
  spec 081 (licences des sons)
- Revient sur : décision D6 de l'issue #33 (« pas de signature payante »), pour Windows seulement
- Livret : aucun (spec technique)
- Crates touchées : arcanum-ui (`build.rs`), arcanum-audio (licences des sons) ; `.github/workflows/release.yml`,
  `README.md`, `docs/release.md`, `docs/ci.md`, `assets/sounds/`

## Objectif

L'archive Windows d'une release contient un `iter-arcanum.exe` **signé** (Authenticode, horodaté), que Smart App Control
laisse lancer sans qu'on le désactive, et pour lequel SmartScreen n'affiche plus « éditeur inconnu ». La signature se fait
dans le workflow de release, sans clé ni certificat chez Mathieu, gratuitement.

Origine : Smart App Control a bloqué `v0.1.0-rc.3` chez Mathieu (#49). Il n'a pas d'exception par application : un
exécutable non signé et sans réputation chez Microsoft est bloqué, et chaque build a une empreinte nouvelle. Le
contournement du README (désactiver Smart App Control) ne se demande pas à un joueur.

## Comparaison des pistes

Relevé le 2026-11-26 (sources en fin de section).

| | SignPath Foundation | Azure Artifact Signing (ex-Trusted Signing) | Certificat individuel open source (Certum) |
|---|---|---|---|
| Coût | gratuit | abonnement Basic, quelques dollars par mois (5 000 signatures) | quelques dizaines d'euros par an (prix à relever) |
| Éditeur affiché | « SignPath Foundation » (le nom du jeu va dans les métadonnées du produit) | Mathieu (identité vérifiée) | Mathieu |
| Accessible à Mathieu | sur dossier, à la discrétion de la fondation | **non** : la validation d'un particulier n'est ouverte qu'aux États-Unis et au Canada, et l'inscription des particuliers est suspendue (réponse Microsoft Q&A du 2026-03-06 à un développeur français) | oui |
| Intégration CI | action GitHub officielle, signature dans le nuage, aucune clé dans le dépôt | action GitHub officielle | clé dans le nuage SimplySign, déverrouillée par un code à usage unique sur téléphone : mal adapté à une CI sans personne |
| Contraintes | licence OSI, aucun composant non libre, build vérifiable depuis le dépôt public sur un runner GitHub, approbation manuelle de chaque release, MFA, page « politique de signature » | facture mensuelle dès la création du compte | renouvellement annuel, signature à la main ou contournements |
| Réputation SmartScreen | celle, déjà acquise, du certificat de la fondation | à construire | à construire |

**Recommandation : SignPath Foundation** (Q1), comme le souhaite Mathieu (commentaire de l'issue #70). Azure est
inaccessible à un particulier en France aujourd'hui ; Certum reste le plan B si la fondation refuse le dossier.

Sources : conditions de la fondation (`signpath.org/terms`), intégration GitHub (`docs.signpath.io/trusted-build-systems/github`),
tarifs Azure (`azure.microsoft.com/pricing/details/artifact-signing/`), Microsoft Q&A « can't create a new trusted signing
individual identity » (2026-03-06) et « when will Azure Artifact Signing support Australia » (2026-07-23 : ouvert aux
organisations australiennes, rien pour les particuliers), offre « Open Source Code Signing » de `shop.certum.eu`.

## Règles

Aucune règle du jeu. Les conditions à tenir sont celles de la SignPath Foundation (`signpath.org/terms`) ; la spec dit
comment le dépôt les remplit et comment le workflow s'en sert.

| Condition de la fondation | Où elle est tenue |
|---|---|
| Licence OSI, aucun composant propriétaire ou non libre | GPL-3.0-or-later (`LICENSE`) ; dépendances par `cargo deny` (déjà bloquant, spec #33 E2) ; **sons** : le hibou (`effects/owl.ogg`, Sampling+ 1.0, non commercial) est le seul composant non libre, à remplacer (C3, Q5) |
| Projet maintenu, déjà publié sous la forme à signer | releases GitHub depuis `v0.1.0` (archive `.zip` avec l'exe) |
| Fonctions décrites sur la page de téléchargement | `README.md`, « What you can do today » |
| Build vérifiable depuis le dépôt public, runners hébergés par GitHub | `release.yml`, job `construire` sur `windows-latest` (inchangé) |
| Signer seulement nos binaires | seul `iter-arcanum.exe` est soumis (E6) |
| Métadonnées du produit imposées (nom du produit = nom du projet, même version) | ressource de version de l'exe (C1, E1) |
| Page « Code signing policy » : phrase de la fondation, rôles, confidentialité | section du `README.md` (C2), liée en tête des notes de release (C5) |
| Confidentialité | le jeu n'envoie rien sur le réseau (aucune dépendance réseau dans `crates/*/Cargo.toml`) : phrase type de la fondation |
| MFA sur GitHub et SignPath pour tous les membres | Mathieu, hors dépôt (étapes de Mathieu) |
| Approbation manuelle de chaque demande de signature | Mathieu, dans SignPath (E3) |

## Modèle

Rien dans l'état ni dans la vue de la partie. Vocabulaire de la spec (pas dans le glossaire, réservé aux termes du jeu) :

- **demande de signature** : envoi de l'exe à SignPath par le workflow ; elle attend l'approbation de Mathieu ;
- **politique de signature** : réglage SignPath qui choisit le certificat. `release-signing` (certificat de la fondation,
  approbation manuelle) et `test-signing` (certificat de test, non reconnu par Windows, sans approbation) ;
- **configuration d'artefact** : réglage SignPath qui dit quoi signer dans ce qu'on envoie (ici un fichier PE seul) et
  impose les métadonnées.

Réglages SignPath (dans le compte, pas dans le dépôt) : organisation, projet `iter-arcanum`, politiques `release-signing`
et `test-signing`, configuration d'artefact « exe Windows » (racine `<pe-file>`, nom de produit « Iter Arcanum »).
Dans le dépôt GitHub : le secret `SIGNPATH_API_TOKEN` et la variable `SIGNPATH_ORGANIZATION_ID`.

## Déroulé

### Étapes de Mathieu (hors code, dans l'ordre)

1. Activer la MFA sur GitHub (si ce n'est déjà fait).
2. Une règle de branche sur `main` : PR obligatoire, pas de `push --force` (déjà la pratique, CLAUDE.md).
3. Après fusion de cette spec implémentée (la politique de signature doit être en ligne) : demander l'admission sur
   `signpath.org/apply`. Délai inconnu, la décision appartient à la fondation.
4. Une fois admis : installer l'application GitHub SignPath sur le dépôt, créer dans SignPath le projet, les deux
   politiques et la configuration d'artefact (pas à pas dans `docs/release.md`, C6), puis le secret et la variable du
   dépôt.
5. À chaque tag : approuver la demande de signature dans SignPath (lien dans le journal du job, E3).

Le code ne dépend pas de l'admission : sans secret, le workflow publie comme aujourd'hui, sans signer (E5, Q4).

### Ressource de version de l'exe (`build.rs`)

`build.rs` embarque déjà l'icône par `winresource` (spec 004). Il y ajoute, depuis `CARGO_PKG_VERSION` :
`ProductName` et `FileDescription` « Iter Arcanum », `OriginalFilename` `iter-arcanum.exe`, `ProductVersion` = la version
du paquet, `FileVersion` numérique `X.Y.Z.0`, `LegalCopyright` « GPL-3.0-or-later ». Aucune dépendance nouvelle. La
compilation croisée depuis Linux reste sans ressource (spec 004, L1).

### Signature dans `release.yml`

Dans le job `construire`, entrée Windows, après la construction et le contrôle `dumpbin` (E6 de la spec #33), avant
l'empaquetage :

1. `actions/upload-artifact` de `iter-arcanum.exe` seul, non compressé (`archive: false`) ;
2. `signpath/github-action-submit-signing-request` (version épinglée par empreinte, comme les autres actions tierces),
   `skip-decompress: true`, `wait-for-completion: true`, sortie dans un dossier à part ;
3. l'exe signé remplace l'exe construit dans `target/x86_64-pc-windows-msvc/release/` ;
4. contrôle de la signature (E2), puis empaquetage et essai de lancement inchangés : l'archive contient l'exe signé, et
   c'est lui que l'essai lance.

| Exécution | Politique | Approbation |
|---|---|---|
| tag final `vX.Y.Z` | `release-signing` | Mathieu, dans SignPath |
| tag de test `vX.Y.Z-rc.N` | aucune signature (Q2), sans avertissement | — |
| essai à blanc (`workflow_dispatch`) | `test-signing` | aucune |
| PR qui touche l'empaquetage | aucune signature (pas de secret pour une PR) | — |

## Comportements attendus

Comme pour la spec #33, l'essentiel se vérifie dans le workflow (E) ; ce qui se vérifie en Rust a son test (C).

### Tests Rust

- **C1** — La ressource de version demandée à `winresource` porte le nom de produit « Iter Arcanum », la version du
  paquet et `X.Y.Z.0` comme version de fichier (`version_resource_names_the_product_and_version`). La construction des
  champs (version numérique tirée de `0.2.0`, `0.2.0-dev`, `1.10.3`) sort de `build.rs` dans une fonction testable sur
  toutes les plateformes ; seul l'appel à `winresource` reste derrière `#[cfg(windows)]`. Une version à suffixe donne
  `X.Y.Z.0` (`0.2.0-dev` → `0.2.0.0`) (`file_version_drops_the_prerelease_suffix`).
- **C2** — Le `README.md` a une section « Code signing policy » qui contient, mot pour mot, « Free code signing provided by
  SignPath.io, certificate by SignPath Foundation », les rôles (Mathieu, `trollou` : auteur, relecteur, approbateur) et la
  phrase de confidentialité « This program will not transfer any information to other networked systems unless
  specifically requested by the user or the person installing or operating it. » (`readme_states_the_code_signing_policy`,
  dans `crates/arcanum-core/tests/depot_public.rs`, à côté des autres garde-fous du dépôt public).
- **C3** — Aucun son sous licence non commerciale (Q5) : `NON_COMMERCIAL_LICENSES` devient une liste de licences refusées,
  et une ligne de `LICENSES.md` qui en porte une est une erreur (`license_check_refuses_non_commercial_licenses`, qui
  remplace le relevé « second choix » de `license_check_finds_missing_and_bad_lines`). Le hibou est remplacé par un son
  libre, choisi sur la planche d'écoute (`scripts/sons/`) et validé à l'oreille par Mathieu.
  `every_sound_has_a_permitted_license` reste vert avec le nouveau fichier.

Tests existants modifiés : `license_check_finds_missing_and_bad_lines` (banque.rs, son cas non commercial devient une
erreur) ; `every_sound_has_a_permitted_license` (son `println!` du relevé non commercial disparaît avec le champ
`LicenseReport::non_commercial`, seul lecteur trouvé par `grep -rn '\.non_commercial\b' crates/`).

### Contrôles du workflow

- **E1** — Après la construction Windows, `(Get-Item iter-arcanum.exe).VersionInfo` donne `ProductName` « Iter Arcanum » et
  `ProductVersion` = la version contrôlée par `controler-version.sh` ; sinon le job échoue (sur toute exécution, PR
  comprise : la ressource ne dépend pas de SignPath).
- **E2** — Après la signature, `Get-AuthenticodeSignature` sur l'exe signé : pour `release-signing`, statut `Valid`,
  certificat au nom de « SignPath Foundation », horodatage présent ; pour `test-signing`, une signature est présente
  (statut autre que `NotSigned`). Sinon le job échoue et rien n'est publié.
- **E3** — Le journal du job affiche le lien de la demande de signature (`signing-request-web-url`) avant d'attendre,
  pour que Mathieu l'approuve. L'attente dure au plus 2 heures (Q3) ; au-delà, le job échoue et se relance depuis
  l'onglet Actions une fois la demande approuvée (aucune règle `disallow_reruns` dans la politique SignPath).
- **E4** — L'archive Windows publiée contient l'exe signé : l'essai de lancement (spec #33 E8) tourne sur l'archive
  construite avec lui, et `SHA256SUMS.txt` est calculé après la signature (ordre des jobs inchangé).
- **E5** — Sur un tag de test `-rc.N` (Q2), aucune demande n'est envoyée, secret présent ou non : l'archive n'est pas
  signée, sans avertissement (choix voulu). Sans `SIGNPATH_API_TOKEN` (avant l'admission, fork, PR), aucune demande n'est envoyée : le job écrit un
  avertissement (`::warning::exe Windows non signé`) et continue, archive non signée (Q4).
- **E6** — Seul l'exe est envoyé à SignPath : ni l'archive, ni `assets/`, ni les autres archives (condition « nos
  binaires seulement »).
- **E7** — L'essai à blanc signe avec `test-signing`, sans approbation : il vérifie toute la chaîne (envoi, retour,
  remplacement, E2) avant le premier tag final signé.

### Documentation

- **C4** — (doc, sans test) `docs/release.md` : section « Signature Windows » (ce que fait le job, approuver une demande,
  relancer après dépassement, réglages SignPath pas à pas, plan B) ; `docs/ci.md` : le secret, la variable, l'action
  ajoutée ; `README.md` : Smart App Control et SmartScreen ne bloquent plus une version signée, le contournement est gardé
  pour les versions d'avant (v0.1.0).
- **C5** — Les notes de chaque release commencent par une ligne qui renvoie à la section « Code signing policy » du
  README (`gh release create --notes … --generate-notes`, la combinaison est à vérifier à l'implémentation ; sinon le
  texte s'ajoute par `gh release edit` après création).
- **C6** — `docs/release.md` liste les étapes de Mathieu (Déroulé, ci-dessus) avec l'état de chacune.

## Cas limites et refus

- **L1** — Demande refusée par Mathieu dans SignPath : le job échoue, aucun brouillon de release n'est créé ; on corrige
  par une PR et on pose un nouveau tag.
- **L2** — SignPath indisponible ou jeton expiré : le job échoue avec l'erreur de l'action ; pas de repli silencieux vers
  une archive non signée sur un tag quand le secret existe (E5 ne joue que sans secret).
- **L3** — Certificat de la fondation renouvelé : rien à changer dans le dépôt, E2 vérifie le nom, pas l'empreinte.
- **L4** — Admission refusée : la spec reste valable jusqu'à E7 ; le plan B (certificat individuel) fera l'objet d'une
  nouvelle issue, qui reprendra E1 à E6 avec un autre outil de signature.

## Information cachée

Aucune.

## Libellés (i18n)

Aucun : rien ne s'affiche dans le jeu. Les textes ajoutés sont dans le README et les docs, en anglais pour le README
(langue du README), en français pour les docs.

## Interface

Aucune dans le jeu. Vérification par Mathieu sur son PC Windows 11 avec Smart App Control activé, sur le **brouillon** de
release du tag final (les rc ne sont pas signées, Q2), avant de le publier : l'archive Windows se lance depuis l'archive dépaquetée, et les propriétés du fichier (onglet « Signatures numériques », onglet
« Détails ») montrent la signature et « Iter Arcanum ». C'est la vérification qui ferme l'issue.

### Raccourcis clavier

Aucun.

## Hors périmètre

- macOS : Gatekeeper, notarisation, compte Apple payant (D6 inchangée pour macOS, signature ad hoc gardée).
- Linux : pas de signature attendue des joueurs.
- Signature des archives elles-mêmes (`.zip`) ou des `SHA256SUMS.txt` (GPG, Sigstore).
- Installateur Windows (MSI, MSIX) et magasin Microsoft (D9 de #33).
- Le plan B (certificat individuel), sauf refus de la fondation (L4).

## Questions ouvertes

- **Q1 — Service.** (a) SignPath Foundation, gratuit, éditeur « SignPath Foundation », sur dossier ; (b) Azure Artifact
  Signing, payant, éditeur Mathieu, mais fermé aux particuliers hors États-Unis et Canada ; (c) certificat individuel
  (Certum), payant, mal automatisable. **Recommandé : (a)**, plan B (c) si refus.
- **Q2 — Signer les tags de test.** (a) oui, `-rc.N` aussi ; (b) seulement le tag final. **Décidé (Mathieu, 2026-11-26) :
  (b)**, les rc restent non signées pour aller plus vite ; avis à revoir plus tard peut-être.
- **Q3 — Durée d'attente de l'approbation.** (a) 2 heures, puis échec et relance du job ; (b) 6 heures (maximum d'un job
  GitHub), runner occupé tout ce temps ; (c) 10 minutes (valeur par défaut de l'action), Mathieu doit approuver aussitôt.
  **Recommandé : (a)** : Mathieu pose le tag et approuve dans la foulée, la relance couvre l'oubli.
- **Q4 — Sans secret.** (a) publier sans signer avec un avertissement, jusqu'à l'admission ; (b) refuser tout tag sans
  signature. **Décidé (Mathieu, 2026-11-26) : (a)** : la v0.2.0 n'attend pas la décision de la fondation, et L2 empêche le
  repli une fois le secret posé.
- **Q5 — Sons non commerciaux.** La fondation refuse tout composant non libre ; le hibou (Sampling+ 1.0) n'en est pas un.
  (a) remplacer le hibou et refuser désormais les licences NC et Sampling+ dans le test des licences (la règle « NC en
  second choix » de la spec 081 et d'ADR 0010 tombe) ; (b) remplacer le hibou seulement, sans changer la règle.
  **Décidé (Mathieu, 2026-11-26) : (a)** : la condition vaudra pour chaque release, un test la garde. ADR 0010 et
  `.claude/rules/arcanum-audio.md` mis à jour dans la même PR.
- **Q6 — D6.** Cette spec remplace D6 pour Windows (« signature gratuite par SignPath Foundation ») et la garde pour macOS.
  Faut-il un ADR ? **Recommandé : non**, la décision est consignée ici et dans `docs/release.md`, comme D6 l'était dans
  l'issue #33.
