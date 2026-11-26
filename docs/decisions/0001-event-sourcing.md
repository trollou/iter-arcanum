# 0001 — Event sourcing léger

- Statut : acceptée (2026-10-23) ; révisée par la spec 024 (2026-10-27, voir « Révision » en fin de fichier)

## Contexte

On veut un journal de partie, la possibilité de revenir en arrière, et à terme des données pour entraîner des IA. Il faut aussi que le moteur reste rapide pour les simulations (MCTS) et que les sauvegardes restent lisibles dans le temps.

## Décision

Toute modification de l'état de partie passe par des événements historisés.

- **Action ≠ événement.** Une *action* est une intention d'un joueur ; le moteur la valide, puis produit un ou plusieurs *événements*, qui sont des faits accomplis (« carte X piochée », « 3 points de mouvement gagnés »).
- **`apply(state, event)` est pure et infaillible.** Toute la validation se fait avant, lors de la décision ; appliquer un événement ne peut pas échouer.
- **Le hasard est résolu dans l'événement.** L'événement enregistre le résultat (carte piochée, tuile révélée, dés obtenus), jamais l'opération aléatoire. Rejouer un journal donne toujours exactement le même état, sans générateur aléatoire.
- **Retour en arrière borné par les règles.** On peut annuler jusqu'à la dernière révélation d'information (pioche, tuile, dés de mana).
- **Information cachée.** Un joueur ne voit le journal qu'à travers un filtre adapté à ce qu'il a le droit de connaître.
- **Événements versionnés dès le départ.** Changer la forme d'un événement exige une migration testée.
- **Instantanés.** Un instantané de l'état est pris régulièrement, pour recharger ou revenir en arrière sans rejouer toute la partie.
- **Journal des décisions.** En plus des événements, on journalise pour chaque décision la vue du joueur, les actions légales et l'action choisie (données d'entraînement IA).
- **Simulation.** Les simulations (MCTS) appliquent les événements sur une copie de l'état sans écrire dans le journal.

## Conséquences

- Chaque type d'événement a ses tests d'application ; chaque action a ses tests de validation (actions refusées comprises).
- Une sauvegarde (`arcanum-persistance`) = configuration de départ + journal d'événements + instantanés, avec tests d'aller-retour.
- Légère complexité en plus par rapport à une mutation directe de l'état, acceptée pour les bénéfices ci-dessus.

## Révision (spec 024, acceptée le 2026-10-27)

- **Instantanés en mémoire seulement.** La sauvegarde n'écrit pas d'instantané d'état : configuration (dans `GameStarted`) + journal
  + position du générateur suffisent, le journal se rejoue en quelques millisecondes. Écrire `GameState`, qui change à chaque spec,
  imposerait une migration à chaque spec. La boucle de partie garde des instantanés en mémoire pour revenir en arrière vite.
- **Décisions dans le journal.** Chaque événement porte le numéro de la décision qui l'a produit : revenir en arrière retire une
  décision entière, et marche après un rechargement.
- **Point de non-retour.** En plus des révélations, finir son tour en est un (MKUE p. 5, *Reverting* 1).
- **Version** : une par fichier de sauvegarde (`EVENT_FORMAT_VERSION`), et non plus une par événement.
- **Pas de migration avant la fin du J6** (décision de Mathieu, 2026-10-27). Pendant le développement, une sauvegarde peut devenir
  illisible après une mise à jour : elle est refusée avec un message. Après le J6, une spec fixera la version de référence, un fichier
  témoin par version (une vraie sauvegarde gardée dans les tests, qui doit toujours se charger) et les migrations.
- **Un générateur par usage** (ADR 0007, spec 038) : la sauvegarde garde la position de chaque générateur, et non plus d'un seul.
- **Pas de mode debug d'annulation libre** (décision de Mathieu, 2026-10-27) : le retour en arrière s'arrête toujours au point de non-retour.

## Révision (spec #52, proposée, à valider avec la spec)

- **Migration à partir de la v0.1.0** : remplace « Pas de migration avant la fin du J6 ». Le format de la v0.1.0 est le
  format 1. Seule garantie : une sauvegarde de l'une des trois dernières versions publiées se charge avec la version courante
  (Mathieu, 2026-11-26). Rien pour les versions de développement ni pour les sauvegardes plus anciennes ; une sauvegarde plus
  récente que le jeu est refusée.
- **Changement compatible ou non** : compatible si une sauvegarde de la version publiée précédente se lit et se rejoue à
  l'identique sans rien faire (variante ajoutée, champ avec `#[serde(default)]`) ; sinon incompatible, ce qui comprend un
  `apply` qui change de sens.
- **Un numéro de format au plus par version publiée** : le premier changement incompatible d'un cycle passe
  `EVENT_FORMAT_VERSION` au format précédent + 1 et écrit la migration, que les suivants du même cycle complètent.
- **Migrations sur le JSON** (`serde_json::Value`), pures, enchaînées du format de la sauvegarde au format courant, en
  mémoire : le fichier n'est jamais réécrit par un chargement. Trois au plus : celle qui part du format d'une version sortie
  de la fenêtre est supprimée à la release.
- **Contrôle** : une empreinte du format (forme JSON de tous les types sauvegardés), commitée et comparée à chaque
  `cargo test`, une empreinte figée par version gardée (leur diff dit ce qui doit migrer), et des sauvegardes témoins de
  chaque version gardée, qui doivent se charger et se rejouer.
