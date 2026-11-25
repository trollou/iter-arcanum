# 082 — Dépôt public

- Statut : implémentée (publication du 2026-11-24) ; validée le 2026-11-20 (Mathieu : propositions des questions retenues, sauf la reprise des anciens dossiers,
  retirée de la spec ; voir « Décisions »)
- Jalon : Finitions
- Dépend de : 003 (logs), 004 (lanceur Linux), 024 (sauvegardes), 080 (dossier de configuration), 081 (licences des sons) ;
  toutes implémentées. Issue privée 256.
- Livret : aucune règle de jeu
- Crates touchées : toutes (renommage), mk-core (test du garde-fou). Aussi :
  `Cargo.toml`, `Cargo.lock`, `.github/workflows/ci.yml`, `.gitignore`, `.claude/`, CLAUDE.md, `docs/architecture.md`,
  `docs/ci.md`, `docs/issues.md`, `docs/verification-manuelle.md`, `docs/qualite/`, `scripts/`, PLAN.md, NOTICE, README,
  et la mémoire du projet

## Objectif

Iter Arcanum devient un projet public : un **dépôt public neuf**, `trollou/iter-arcanum`, reçoit le code, `assets/`, nos
propres documents et les issues, et devient le dépôt de travail. Le dépôt privé actuel, `trollou/mage-knight`, ne garde que le
matériel de tiers qu'on ne peut pas redistribuer (livret, FAQ, compendium, scans et leurs transcriptions) et sert d'archive.
Les identifiants internes perdent le nom « Mage Knight ».

Décision de Mathieu (2026-11-14, issue privée 256, PR privée 308) : **variante C**, pas de miroir ; le public part d'un **instantané** de
`main` sans le matériel de tiers (pas de `git filter-repo`), protégé par un garde-fou. Les noms officiels des héros et des
ennemis sont gardés (Mathieu, 2026-11-13).

## Règles

Aucune règle de jeu. Les règles qui encadrent la spec sont celles des licences :

- **Licences du projet** (NOTICE, Mathieu 2026-11-12) : code et nos documents sous GPL-3.0-or-later, `assets/` sous
  CC BY-SA 4.0, sons sous la licence de chaque auteur (`assets/sounds/LICENSES.md`, spec 081). Inchangées par la spec.
- **Matériel de tiers** (NOTICE, « What these licenses do not cover ») : le livret, le walkthrough, la FAQ officielle et sa
  traduction, le compendium BGG, les scans et photos de cartes, et leurs transcriptions ne doivent jamais être publiés. C'est
  ce que le garde-fou protège.
- **Renvois au livret** (CLAUDE.md, gabarit des specs) : nos documents renvoient au livret (`MKUE p. 7, 5.b`) au lieu de le
  recopier. Un renvoi se publie ; une citation longue non (audit, étape B2).
- **Sons non commerciaux** : un seul fichier, `effects/owl.ogg` (Sampling+ 1.0), est marqué non commercial dans
  `LICENSES.md` et NOTICE ; sa publication dans un dépôt public non commercial est permise par sa licence (Q7).
- **Illustrations générées** (ADR 0009) : `assets/images/card-art/`, `unit-art/` et `hero-art/` ont été générées sans image
  officielle en entrée, et les descriptions de `scripts/illustrations/*.json` ne nomment pas le jeu d'origine (vérifié :
  aucune occurrence de « Mage Knight » dans `scripts/illustrations/`). Elles se publient telles quelles.

## Modèle

### Ce qui va où

Inventaire de `main` au 2026-11-20 (ba3fd3e).

| Contenu | Taille | Public `iter-arcanum` | Privé `mage-knight` |
|---|---|---|---|
| `crates/`, `Cargo.*`, `deny.toml`, `.github/`, `.gitattributes`, `.gitignore` | 6 Mo | oui | retiré (archive) |
| `assets/` (images, i18n, sons, licences) | 16 Mo | oui | retiré |
| `scripts/` sauf `faq-compendium/` | 1,5 Mo | oui | retiré |
| `scripts/faq-compendium/` (conversion du compendium) | — | non | gardé |
| `docs/specs/` (specs, plans, images de specs : captures et maquettes à nous), PLAN.md | — | oui | retiré |
| `docs/decisions/`, `architecture.md`, `glossaire.md`, `ci.md`, `issues.md`, `raccourcis.md`, `verification-manuelle.md`, `qualite/` | — | oui | retiré |
| CLAUDE.md, `.claude/` (règles par crate, hooks) | — | oui, CLAUDE.md réécrit (B4) | CLAUDE.md court (C1) |
| LICENSE, `assets/LICENSE`, NOTICE, README | — | oui | README court (C1) |
| Nos pages de référence de `docs/regles/` : `bac-a-sable.md` et `.svg` (générées par un test), `index-par-notion.md`, `tuiles.md`, `decks-de-depart.md` | 60 Ko | oui, déplacées (Q3) | retirées |
| `docs/regles/livret/` et `mkue-rulebook.pdf`, `MK_walkthrough_EN.pdf` | 18 Mo | **jamais** | gardés |
| `docs/regles/faq/` (FAQ officielle, compendium et transcriptions) | 14 Mo | **jamais** | gardé |
| `docs/regles/scans/`, `docs/regles/cartes/` (transcriptions des cartes) | 71 Mo | **jamais** | gardés |
| Issues ouvertes (21 aujourd'hui, dont issue privée 256 que la spec ferme) | — | recréées (B5) | fermées avec un lien (C2) |
| Issues fermées, PR, historique git | — | non | gardés en archive |

`tuiles.md` et `decks-de-depart.md` ne figuraient pas dans la ligne de PLAN.md : ce sont nos notes de lecture (valeurs déjà
publiques dans `tiles.ron` et `heroes.ron`), sans texte de tiers ; elles suivent les autres pages de référence.

### Disposition des deux dépôts

Les deux dépôts sont clonés **côte à côte**, dans le même dossier parent, sous leur nom :

```
<parent>/
├── iter-arcanum/      # dépôt de travail public : code, docs, issues
└── mage-knight/       # bibliothèque privée, lecture seule : docs/regles/ (livret, faq, cartes, scans)
```

- **Fil cloud** : chaque dépôt du projet est cloné dans `/home/user/<nom du dépôt>` (constaté dans ce fil :
  `/home/user/mage-knight`). Avec les deux dépôts déclarés dans les ressources du projet, un fil voit donc
  `/home/user/iter-arcanum` et `/home/user/mage-knight`. Un fil ne peut pas le prouver tant que le dépôt public n'existe pas :
  c'est la première vérification du fil de publication (B1), bloquante.
- **Poste de Mathieu (Windows) et Linux** : même disposition, à faire une fois à la main (`git clone` des deux dans le même
  dossier).
- Le chemin du livret dans CLAUDE.md devient **relatif** : `../mage-knight/docs/regles/livret/` (barres obliques, comprises par
  Git Bash, PowerShell et les outils des fils). Garder le nom du dépôt privé (Q2) garde ce chemin stable.
- Un contributeur extérieur n'a pas ce dossier : les renvois `MKUE p. 7` restent lisibles comme références au livret qu'il
  possède. CLAUDE.md le dit.

### Identifiants renommés (Q1)

| Aujourd'hui | Après |
|---|---|
| crates `mk-core`, `mk-persistance`, `mk-i18n`, `mk-audio`, `mk-ui` (et `mk_*` dans le code, 2 344 occurrences) | `arcanum-core`, `arcanum-persistance`, `arcanum-i18n`, `arcanum-audio`, `arcanum-ui` |
| binaire `mk-ui` | `iter-arcanum` (`[[bin]]` de `arcanum-ui`) |
| `MageKnightApp` | `IterArcanumApp` |
| `APP_ID = "mage-knight"` (fichier `.desktop`, `StartupWMClass`, icône) | `"iter-arcanum"` |
| `assets/images/icon/mage-knight.svg` et `.ico` | `iter-arcanum.svg` et `.ico` |
| `mage-knight.log`, `mage-knight.<n>.log` | `iter-arcanum.log`, `iter-arcanum.<n>.log` |
| dossiers `directories` « mage-knight » (sauvegardes, configuration, logs) | « iter-arcanum », sans reprise des anciens (ci-dessous) |
| variables `MK_SAVE_DIR`, `MK_CONFIG_DIR`, `MK_LOG`, `MK_LOG_DIR`, `MK_ASSETS_DIR` ; `MK_VERIF_LANGUE` des scripts | `ARCANUM_*`, sans relire les anciennes |
| `.claude/rules/mk-*.md` | `.claude/rules/arcanum-*.md` |

### Anciens dossiers : pas de reprise

Décision de Mathieu (2026-11-20) : seul joueur pour l'instant, il accepte de perdre ses sauvegardes, préférences et logs. Le
jeu n'utilise que les nouveaux dossiers `iter-arcanum` ; les anciens dossiers `mage-knight` (sauvegardes, préférences, logs,
lanceur Linux `mage-knight.desktop`) ne sont ni lus, ni déplacés, ni supprimés. Mathieu les efface à la main s'il le souhaite.

## Déroulé

La spec s'implémente en **deux fils** (CLAUDE.md, grosse spec), puis une étape de ménage :

### Fil A — Renommage (dépôt privé, branche `spec/082-depot-public`)

- **A1** Tests rouges des cas C1 à C3 (tests existants mis à jour sur les nouveaux noms).
- **A2** Renommage des identifiants (tableau ci-dessus) ; mise à jour des scripts, de la CI (filtres de chemins), de
  `.claude/`, CLAUDE.md, `docs/architecture.md`, `docs/verification-manuelle.md`, `docs/qualite/`.
- **A3** Déplacement de nos pages de référence hors de `docs/regles/` (Q3) et de leurs renvois (tests `reference_bac_a_sable`,
  commentaires de 5 fichiers de test, CLAUDE.md, gabarit).
- **A4** `scripts/verifier.sh` et `cargo deny check` verts ; PR passée « prête », fusionnée par Mathieu.

### Fil B — Publication (dépôt public neuf)

Préalables, faits par Mathieu (un fil ne peut ni créer un dépôt ni changer les ressources du projet) : créer le dépôt
**vide** `trollou/iter-arcanum` (public, sans README ni licence générés), y installer l'application GitHub Claude, l'ajouter
aux ressources du projet. Toutes les PR du privé fusionnées ou fermées avant l'instantané.

- **B1** Vérifier que le fil voit les deux dépôts côte à côte (`/home/user/iter-arcanum`, `/home/user/mage-knight`).
  Sinon, s'arrêter et le dire à Mathieu.
- **B2** Construire l'instantané de `main` du privé dans `iter-arcanum` (script à usage unique `scripts/publication/`,
  commité dans le privé), sans `docs/regles/` ni `scripts/faq-compendium/`. **Audit** : chercher les citations longues du
  livret, de la FAQ ou du compendium dans `docs/`, CLAUDE.md, `.claude/`, les commentaires, `assets/i18n/` et les corps
  d'issues à recréer ; les remplacer par des renvois. (Sondage à la rédaction : aucune citation anglaise de plus de 120
  caractères dans `docs/specs`, `docs/decisions`, le glossaire ; aucune dans les commentaires `.rs` ; les textes de
  `assets/i18n/` sont les nôtres.) Lister aussi les 72 images hors de `assets/` (toutes dans `docs/specs/images/`) pour un
  coup d'œil de Mathieu.
- **B3** Garde-fou (C4, C5) : test rouge dans le privé (il y trouve le livret), vert dans l'instantané. Ajouter à
  `.gitignore` les dossiers de tiers (`docs/regles/`, `*.pdf`) comme première barrière.
- **B4** CLAUDE.md, `.claude/rules/`, `docs/issues.md`, `docs/ci.md`, gabarit, `index-par-notion.md` (liens vers le livret et
  le compendium réécrits en `../mage-knight/docs/regles/...`) : chemins du dépôt voisin, nom du dépôt de travail, numéros
  d'issues. CI publique (Q6).
- **B5** Renvois aux issues et aux PR (Q4) : environ 590 « issue #N » (119 numéros distincts : 219 dans le code, 358 dans le
  Markdown, 12 dans les scripts et la CI) et 17 « PR #N ». Le script les réécrit à partir de la table de correspondance et
  compte avant et après ; il échoue s'il reste un « issue #N » qui ne désigne pas une issue publique.
- **B6** `scripts/verifier.sh` vert dans l'instantané. Fil posté à Mathieu : liste des fichiers publiés par dossier, taille,
  images, résultat de l'audit. **Attente de son accord explicite** (publication irréversible).
- **B7** Après l'accord : recréer les issues ouvertes dans l'ordre croissant de leur numéro privé (dépôt vide : elles
  prennent les numéros 1, 2, 3… prévus par la table de B5 ; tout écart arrête le fil) avec leurs labels (Q5) ; puis pousser
  l'instantané en un seul commit sur `main` (« Instantané de trollou/mage-knight@<sha> »), seule exception à « jamais de
  commit sur main », faute de branche de base dans un dépôt vide.
- **B8** Mettre à jour la mémoire du projet (dépôt de travail, chemin du livret, numéros d'issues) et la ligne 082 de PLAN.md
  (dans le public).

### Étape C — Ménage du privé (fil B, ou fil d'issue juste après)

- **C1** PR dans le privé : retirer tout sauf `docs/regles/` (matériel de tiers), `scripts/faq-compendium/`,
  `scripts/publication/` ; README et CLAUDE.md courts (« bibliothèque de référence d'Iter Arcanum, ne pas redistribuer ;
  dépôt de travail : trollou/iter-arcanum »). Fusionnée par Mathieu.
- **C2** Fermer chaque issue recréée avec un commentaire « Suite : trollou/iter-arcanum#N ». Fermer issue privée 256 par la PR de
  publication.

## Comportements attendus

Chaque cas devient au moins un test (nom indicatif entre parenthèses). Tests existants modifiés par le renommage
(recherche faite à la rédaction, `grep -rn 'mage-knight\|Mage Knight\|APP_ID'` dans `crates/*/src` et `crates/*/tests`) :
`desktop_entry.rs` (tests en ligne 123 à 168), `tests/icone.rs` (`APP_ID`, chemin du `.ico`, titre d'essai « Mage Knight »),
`tests/icone_logs.rs`, `tests/logs_techniques.rs` (noms des fichiers et dossiers de logs) ; et tout `use mk_*` des tests,
renommé mécaniquement.

### Renommage

- **C1** — L'identifiant de l'application est `iter-arcanum` : le fichier `.desktop` écrit sous Linux s'appelle
  `iter-arcanum.desktop` et porte `Icon=iter-arcanum` et `StartupWMClass=iter-arcanum` (`app_id_is_iter_arcanum`, tests
  existants de `desktop_entry.rs` et `icone.rs` mis à jour).
- **C2** — Le fichier de log s'appelle `iter-arcanum.log`, et ses anciens `iter-arcanum.<n>.log`, dans le dossier
  `iter-arcanum` (`logs_techniques.rs` mis à jour).
- **C3** — Le dossier de sauvegardes et celui des préférences de l'utilisateur portent le nom `iter-arcanum`
  (`user_dirs_are_named_iter_arcanum`).

### Garde-fou du dépôt public

Test d'intégration `crates/arcanum-core/tests/depot_public.rs`, lancé par `scripts/verifier.sh` (donc avant chaque push) et
par la CI. Il lit la liste des fichiers suivis (`git ls-files`, avec leur taille) et la passe à une fonction de classement
pure.

- **C4** — La fonction de classement refuse, avec la règle en cause (`forbidden_file_reasons`) :
  - tout fichier sous `docs/regles/` ou `scripts/faq-compendium/` ;
  - tout `.pdf` ;
  - toute image matricielle (`.png`, `.jpg`, `.jpeg`, `.webp`, `.gif`, `.bmp`, `.tif`, `.tiff`) hors de `assets/` et de
    `docs/specs/images/` ;
  - tout fichier de plus de 5 Mo (le plus gros aujourd'hui : `music/night.ogg`, 3 Mo).

  Exemples : `docs/regles/livret/p07.md` refusé ; `docs/specs/images/072-illustrations/combat.png` accepté ;
  `docs/regles-copie/scan.jpg` refusé (image) ; `assets/sounds/music/night.ogg` accepté.
- **C5** — Le dépôt n'a aucun fichier refusé ; sinon le test échoue en listant chaque fichier et sa règle
  (`public_repository_has_no_third_party_material`). Rouge dans le privé (livret, scans), vert dans l'instantané.

Vérifiés par les scripts de publication, sans test (outils à usage unique, gardés dans le privé) : le compte des renvois
réécrits (B5) et la numérotation des issues recréées (B7).

## Cas limites et refus

- **L1** — Un fil qui recopie par erreur une page du livret dans le public : le push est bloqué par le hook (C5) ; s'il passe
  outre, la CI échoue sur la PR. Une fuite poussée resterait dans l'historique public : d'où les deux barrières et
  `.gitignore`.
- **L2** — Un contributeur sans le dépôt privé : le jeu compile et tous les tests passent (aucun test ne lit
  `../mage-knight/`, vérifié par B6 dans un clone seul).

## Information cachée

Sans objet.

## Libellés (i18n)

Aucune clé nouvelle : le titre de la fenêtre est déjà « Iter Arcanum » (PR privée 263, `app.title`).

## Interface

Aucune. Vérification à l'œil (fil A, sous Xvfb) : le jeu se lance et la fenêtre s'appelle toujours « Iter Arcanum ».

### Raccourcis clavier

Aucun.

## Hors périmètre

- Guide de contribution (`CONTRIBUTING.md`), modèles d'issues, protection de la branche `main` du public : réglages GitHub,
  à proposer dans une issue après la publication.
- Paquets et versions publiées (exécutables Windows, AppImage) : plus tard.
- Copie des issues fermées et de l'historique des PR : ils restent dans le privé, en archive.
- macOS : spec #7.
- Reprise des anciens dossiers `mage-knight` (sauvegardes, préférences, logs, lanceur) : écartée par Mathieu (2026-11-20).

## Questions ouvertes

Aucune.

## Décisions

Mathieu, 2026-11-20 : « ok avec tout », sauf la reprise des anciens dossiers, retirée (Q8 et Q9 deviennent sans objet).
Chaque question est tranchée selon sa proposition :

- **Q1 — Nouveau préfixe des crates.** Retenu : `arcanum-*` (court, sans le sigle « ia » qui se lit « intelligence
  artificielle »), binaire `iter-arcanum`, `IterArcanumApp`, variables d'environnement `ARCANUM_*` sans relecture des
  anciennes `MK_*` (outils de développement seulement). Écartés : `iter-*`, ou garder `mk-*` (moins de travail, mais le
  sigle reste celui du jeu d'origine).
- **Q2 — Nom du dépôt privé.** Retenu : garder `trollou/mage-knight` (chemin `../mage-knight/` stable, liens et mémoire
  inchangés ; le nom décrit bien son contenu). Écarté : le renommer `iter-arcanum-regles` (GitHub redirige, mais les
  clones locaux et le chemin de CLAUDE.md changent).
- **Q3 — Nos pages de référence.** Retenu : les déplacer de `docs/regles/` vers `docs/reference/` dans le fil A
  (`bac-a-sable.md` et `.svg`, `index-par-notion.md`, `tuiles.md`, `decks-de-depart.md`), pour que `docs/regles/` désigne
  toujours le matériel de tiers et que le garde-fou interdise ce dossier en entier. Écarté : les laisser en place, avec
  une liste d'exceptions dans le garde-fou.
- **Q4 — Renvois aux anciennes issues.** Retenu : un renvoi à une issue recréée prend son nouveau numéro public
  (« issue privée 3 ») ; un renvoi à une issue fermée devient « issue privée 256 » (sans dièse, pour que GitHub n'en fasse pas un
  lien vers l'issue publique issue privée 256), et de même « PR privée 330 ». Une page `docs/issues-privees.md`, générée une fois, donne
  le numéro et le titre de chaque issue privée citée (119), pour qu'un lecteur sache de quoi il s'agit. Écarté : un lien
  complet vers le privé (illisible pour un contributeur).
- **Q5 — Contenu des issues recréées.** Retenu : titre, labels et corps repris tels quels, les commentaires ajoutés à la
  suite du corps (auteur et date), après l'audit des citations ; une ligne finale « Reprise de l'issue privée N ». Les 18
  labels sont recréés à l'identique. Écarté : corps seul.
- **Q6 — CI du dépôt public.** Les minutes sont illimitées sur un dépôt public, Windows compris. Retenu : Linux et
  Windows sur chaque PR prête, sans filtre de chemins (une PR de doc tourne aussi, en quelques minutes) ; un brouillon ne lance
  toujours rien ; la nuit sur `main` est supprimée ; l'issue privée 305 (quota) est close. Écarté : garder les filtres et la
  nuit.
- **Q7 — Le son non commercial.** `effects/owl.ogg` (Sampling+ 1.0) peut être publié dans un projet non commercial ; il est
  déjà signalé dans NOTICE et `LICENSES.md`. Retenu : le garder. Écarté : le remplacer avant la publication par un
  hibou CC0 ou CC BY (recherche sur opengameart, freesound injoignable depuis le cloud).
- **Q8 et Q9 — Logs, échec du renommage.** Sans objet : pas de reprise des anciens dossiers.
- **Q10 — Qui publie.** Retenu : Mathieu crée le dépôt vide et l'ajoute au projet ; le fil B prépare tout, poste la liste
  des fichiers et l'audit, et ne crée les issues ni ne pousse qu'après son accord explicite dans le fil (B6, B7). Les règles de
  fusion ne changent pas (Mathieu fusionne).
