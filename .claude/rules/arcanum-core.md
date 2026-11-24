---
paths:
  - "crates/arcanum-core/**"
---

# Règles pour `arcanum-core`

Détails : `docs/decisions/0001-event-sourcing.md` et `docs/decisions/0002-interface-joueurs.md`.

- Aucune dépendance à egui, aux fichiers, au réseau ni à l'horloge. Tout doit être testable sans fenêtre ni disque.
- Logs techniques (spec 003) : seulement les macros de la façade `log`, appelées par `::log::` (le module `log` d'arcanum-core est le journal), et seulement depuis la boucle de partie. Jamais dans `apply`, `decide`, `legal_actions`, `view`, `pending_decision` ni `simulate`. L'information cachée ne dépasse pas la gravité `debug`.
- Aucun texte affiché : seulement des clés i18n (ex. `card.march.name`).
- L'état ne change que via `apply(state, event)`, pure et infaillible. La validation se fait en amont, sur l'action.
- Un événement porte le résultat du hasard (carte piochée, tuile, dés), jamais l'opération aléatoire.
- Le hasard passe par un générateur à graine injectable ; jamais de `rand::thread_rng()` ou équivalent.
- L'énumération des actions légales est exhaustive : un `Player` ne peut choisir que parmi elles.
- Une vue joueur ne contient que ce que ce joueur a le droit de connaître.
- L'état est clonable à bas coût (pas de ressource externe dedans).
- Contenu du jeu (spec 029, ADR 0005) : un nouveau contenu s'écrit dans un fichier de `content/`, lu par `GameContent::parse` et embarqué par `include_str!` ; jamais de lecture de fichier à l'exécution. Le fichier ne contient que des valeurs (chiffres, couleurs, choix parmi les effets connus du moteur) et aucun texte pour le joueur. Toute règle de cohérence (identifiant manquant, en double, référence entre fichiers) est vérifiée par `parse` et testée.
- Chaque règle a au moins un test qui cite la page du livret en commentaire (`// MKUE p. 12`).
