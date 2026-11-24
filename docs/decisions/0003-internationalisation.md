# 0003 — Internationalisation

- Statut : acceptée (2026-10-23)

## Contexte

Le jeu doit pouvoir être affiché dans plusieurs langues, avec un fichier JSON par langue contenant les libellés.

## Décision

- `arcanum-core` ne contient **aucun texte affiché**. Il manipule des clés (ex. `card.march.name`), et l'UI les traduit.
- Les libellés sont dans `assets/i18n/<langue>.json`, un fichier par langue, avec des clés hiérarchiques et des paramètres nommés (ex. `"Gagnez {amount} points de mouvement"`).
- `arcanum-i18n` charge le fichier de la langue choisie, résout les clés et applique les paramètres.
- Une clé manquante s'affiche visiblement (ex. `⟦card.march.name⟧`) et est journalisée, sans faire planter le jeu.
- Le français (`fr.json`) est la langue de référence.
- La langue affichée est celle du système si elle a un fichier, sinon l'anglais (langue de repli) ; le paramètre `--lang` la force (spec 076). Le français reste la langue de référence des tests et désigne le dossier des ressources.

## Conséquences

- Tests obligatoires : toutes les langues ont exactement les mêmes clés et les mêmes paramètres ; toute clé utilisée par le code existe dans `fr.json`.
- Aucun texte en dur dans `arcanum-ui`.
