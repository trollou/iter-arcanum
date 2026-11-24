# 0002 — Interface des joueurs

- Statut : acceptée (2026-10-23)

## Contexte

On commence en solo avec un joueur humain, mais on veut pouvoir brancher plus tard des joueurs automatiques : règles simples, IA heuristique, MCTS, voire LLM. Le moteur ne doit pas avoir à changer pour cela.

## Décision

Le moteur ne sait pas qui joue : tous les joueurs passent par un même trait `Player`, défini dans `arcanum-core`.

- Le moteur expose la **décision en attente** (quel joueur doit choisir, et quel type de choix) et la **liste exhaustive des actions légales**. Un joueur choisit parmi cette liste : il ne peut jamais produire une action illégale.
- Un joueur reçoit une **vue** de l'état limitée à ce qu'il a le droit de voir (pas la main des autres ni l'ordre de la pioche), jamais l'état complet.
- Le trait est minimal : en substance, « voici ta vue et tes actions légales, renvoie ton choix ». Le joueur humain l'implémente aussi, via `arcanum-ui`.
- Pour les joueurs lents (humain, LLM), la boucle de partie ne bloque pas l'UI : le moteur avance décision par décision, et c'est la boucle de partie qui demande et attend la réponse.
- Pour le MCTS, l'état est clonable à bas coût et le moteur permet de simuler une action sur une copie, de façon déterministe à graine donnée.
- Les joueurs automatiques vivent dans `arcanum-players` (ou leur propre crate s'ils tirent de grosses dépendances, comme un client LLM) et ne touchent pas `arcanum-core`.

## Conséquences

- L'énumération des actions légales est une fonctionnalité centrale du moteur, testée comme telle.
- Aucune implémentation de joueur automatique n'est écrite sans sa spec ; seul l'humain est prévu au départ.
