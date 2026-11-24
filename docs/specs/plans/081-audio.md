# Plan de développement — spec 081 Audio

- Statut : exécuté (fils moteur, puis sons et écran, 2026-11-20).
- Spec : [`081-audio.md`](../081-audio.md). Branche : `spec/081-audio`.
- Méthode : celle de `CLAUDE.md` (tests rouges, code minimal, refactor ; `scripts/verifier.sh` vert avant chaque push).

## Découpage en PR

Une seule PR (privée 363), qui porte déjà la spec et l'ADR 0010.

## Découpage en fils

Deux fils (moteur et écran à la fois, issue privée 118), coupure fixée par la spec (section Découpage) :

1. **Fil moteur : étapes 1 à 10.** Crate arcanum-audio (directeur, banque, lecteurs), volumes dans arcanum-persistance, événements
   entendus, situation et commandes de volume dans arcanum-ui, outillage (`libasound2-dev`, CI, `cargo deny`).
2. **Fil sons et écran : étapes 11 à 16.** Planche d'écoute, fichiers et `LICENSES.md`, libellés, section « Son »,
   branchement dans `app/` et `main.rs`, docs, liste de vérification à l'oreille.

Contrôle (issue privée 130) : l'écran (C23) ne lit que `Preferences::volumes` (étape 7) et le résultat de la création du lecteur
(étape 6) : aucun champ de vue nouveau.

## Étapes

| # | Étape | Cas | Rouge attendu | Fichiers | Isolée ? | Modèle |
|---|---|---|---|---|---|---|
| 1 | Crate arcanum-audio vide au workspace, Kira sans options par défaut, `libasound2-dev` dans `verifier.sh` et la CI de nuit, `cargo deny check licenses` | — | — (outillage) | `Cargo.toml`, `crates/arcanum-audio/`, `scripts/verifier.sh`, `.github/workflows/ci.yml`, `docs/ci.md` | non | fil principal |
| 2 | Directeur : musique et ambiance selon la situation, table des fondus | C1 à C5, C27, L4 | C1 : aucun ordre rendu (squelette vide) | `arcanum-audio/src/director.rs`, `tests/directeur.rs` | non | fil principal |
| 3 | Directeur : effets sonores (table, une fois par image, quatre au plus, pas espacés, joueur humain seul, victoire ou défaite) | C12 à C15, C17, C29 | C12 : aucun effet rendu | idem, `src/cues.rs` | non | fil principal |
| 4 | Directeur : échantillons des curseurs | C30 | C30 : aucun ordre rendu | idem | non | fil principal |
| 5 | Banque de sons (dossiers, variantes à tour de rôle, effet manquant), `LICENSES.md` vide et son test | C16, C24, C25, L2 | C24 : banque vide ; C16 : aucune variante | `src/bank.rs`, `tests/banque.rs`, `assets/sounds/LICENSES.md` | non | fil principal |
| 6 | Lecteurs : trait `AudioPlayer`, `SilentPlayer`, `KiraPlayer` | L5 | — (câblage vérifié à l'oreille, ADR 0006) | `src/player.rs` | non | fil principal |
| 7 | Volumes dans les préférences | C18, C19 | C18 : défauts à 0 (squelette) | `arcanum-persistance/src/preferences.rs`, `tests/preferences.rs` | non | fil principal |
| 8 | Événements entendus de la session | C6 à C9 | C6 : rien rendu | `arcanum-ui/src/client.rs`, `tests/audio.rs` | non | fil principal |
| 9 | Situation sonore lue dans l'écran et la vue | C10, C11, C28 | C10 : situation fixe du squelette | `arcanum-ui/src/sound.rs`, `tests/audio.rs` | non | fil principal |
| 10 | Commandes `SetVolume` et `SaveVolumes`, demandes `Volumes` et `Preview` | C20 à C22 | C20 : aucune demande | `arcanum-ui/src/client.rs`, `screen/types.rs`, `tests/audio.rs` | non | fil principal |
| 11 | Planche d'écoute, choix de Mathieu | — | — | `/mnt/project-files/` | non | fil principal |
| 12 | Fichiers de sons, `LICENSES.md`, `NOTICE`, test du budget | C25, C26 | C26 : aucun fichier | `assets/sounds/`, `tests/banque.rs` | non | fil principal |
| 13 | Libellés `fr.json` et `en.json` (tableau de la spec) | — | — | `assets/i18n/*.json` | oui | Haiku |
| 14 | Section « Son » de la fenêtre des préférences | C23 | C23 : champs vides | `arcanum-ui/src/preferences.rs`, `app/windows.rs` | non | fil principal |
| 15 | Branchement dans `main.rs` et `app/` (lecteur, banque, directeur, demandes, échantillon au clavier) | L1, L3 | — (vérification à l'œil et à l'oreille) | `arcanum-ui/src/main.rs`, `app/mod.rs` | non | fil principal |
| 16 | Docs : CLAUDE.md, `.claude/rules/arcanum-audio.md`, architecture, README, NOTICE, PLAN.md, index ; liste de vérification à l'oreille | — | — | docs | oui | Sonnet |

## Choix faits au codage (fil moteur)

- Le directeur ne connaît pas la banque : il numérote chaque effet joué (`nth`), et la banque choisit la variante
  `nth % nombre de variantes` (C16 se vérifie donc sur la banque, à partir des numéros rendus par le directeur).
- Un sort joué de côté n'est pas « lancé » : seul un `CardPlayed` d'effet de base ou renforcé d'un sort donne `Spell`.
- `PlayerEvent::CityConquered` se rapporte à son `leader` ; `RoundStarted` et `TileRevealed`, sans joueur, sonnent toujours.

## Risques repérés

- `libasound2-dev` absent d'une machine Linux : la compilation de tout le workspace échoue ; `verifier.sh` l'installe.
- `cargo deny` à vérifier sur la vraie résolution des dépendances de Kira (essai fait à la rédaction seulement).
- `KiraPlayer` sans test : à écouter par Mathieu au fil 2.
