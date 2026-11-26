# 0010 — Audio

- Statut : acceptée (2026-11-20), avec la spec 081 Audio, qui la traduit en comportements.
- Complète la règle « Assets et droits » de `CLAUDE.md` : `assets/sounds/` contient des sons de tiers sous licences libres,
  à la différence des visuels, créés par nous.

## Contexte

Le jeu n'a pas de son (issue privée 187). Mathieu veut une musique qui suit la partie, des effets superposés sans couper la musique,
des fondus et des volumes indépendants par type de son, pilotés par les événements du jeu, dans une crate à part, avec Kira
comme bibliothèque envisagée. Les règles et l'affichage restent séparés (CLAUDE.md, ADR 0001 et 0002), les tests ne doivent
pas dépendre d'une carte son (ADR 0006), et le jeu doit tourner sous Windows et Linux (ADR 0004).

## Décision

- **Une crate `arcanum-audio`**, entre `arcanum-core` et `arcanum-ui` (`arcanum-ui → arcanum-audio → arcanum-core`). `arcanum-core` n'en sait rien.
- **Le son écoute, il ne décide rien.** Il reçoit les événements **filtrés pour le joueur humain** (comme tout joueur,
  ADR 0002) et la situation lue dans la vue. Rien du son ne va dans l'état, le journal ou les sauvegardes.
- **Deux parties** dans la crate :
  - un **directeur pur** (situation et événements nouveaux → ordres de son), sans I/O ni Kira, couvert par les tests ;
  - un **lecteur** qui exécute les ordres : Kira sur la carte son, ou un lecteur muet quand il n'y a pas de sortie audio.
    Le lecteur Kira est un câblage vérifié à l'oreille, comme `app/` est vérifié à l'œil (ADR 0006).
- **Kira** (MIT OR Apache-2.0), sans ses options par défaut : `cpal`, `ogg`, `vorbis`. Pistes et volumes par type de son,
  fondus, départs différés, lecture en flux. Sous Linux, la compilation demande `libasound2-dev` (ALSA, par cpal).
- **Sons de tiers sous licences libres** compatibles avec la redistribution du dépôt : CC0, CC BY, CC BY-SA, OGA-BY ;
  jamais NC (BY-NC, BY-NC-SA, Sampling+ : refusées par le test depuis la spec #70, Q5, la signature par la SignPath
  Foundation excluant tout composant non libre ; la spec 081, Q12, les permettait en second choix), ni ND, ni licence qui
  interdit de redistribuer les fichiers eux-mêmes. Rien d'officiel. Chaque fichier a sa
  ligne dans `assets/sounds/LICENSES.md` (titre, auteur, source, licence, modifications), vérifiée par un test.
- **Format** : Ogg Vorbis, dans `assets/sounds/` (musiques, ambiances, effets), chargés depuis le dossier des ressources
  et jamais embarqués dans l'exécutable. Un fichier manquant rend le son muet, jamais le jeu en erreur.

## Conséquences

- Une dépendance de plus (et ses dépendances : cpal, symphonia en MPL-2.0…), à garder dans `cargo deny check licenses`.
- `scripts/verifier.sh`, la CI Linux et le README doivent faire installer `libasound2-dev`.
- `NOTICE` distingue `assets/sounds/` (licence de chaque fichier) du reste de `assets/` (CC BY-SA 4.0).
- Un fil cloud n'a pas de carte son : il ne vérifie que ce qui s'affiche ; l'écoute revient à Mathieu.
- macOS (spec 085) : Kira passe par CoreAudio, rien de plus à prévoir ici.
