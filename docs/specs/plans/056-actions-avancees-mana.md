# Plan de développement — spec 056 Actions avancées : mana et cartes

- Statut : accepté par Mathieu (2026-11-12), avec la spec.
- Spec : [`056-actions-avancees-mana.md`](../056-actions-avancees-mana.md). Branche : `spec/056-actions-avancees-mana`.
- Méthode : celle de `CLAUDE.md` (tests rouges, code minimal, refactor ; `scripts/verifier.sh` vert avant chaque push).

## Découpage en PR

Comme la 053 : la PR de rédaction (spec et plan validés) est fusionnée d'abord ; l'implémentation (deux fils) fait une seconde PR,
sur la branche `spec/056-actions-avancees-mana` recréée depuis `main`.

## Découpage en fils

Deux fils successifs sur la même branche (issue privée 118 : 14 étapes, moteur et écran) :

1. **Fil moteur** : étapes 1 à 9 (arcanum-core, contenu, persistance). Il pousse l'étape 9 verte et s'arrête.
2. **Fil écran** : étapes 10 à 14 (libellés, écrans, partie préparée, vérification à l'œil, docs, bilan).

Contrôle (issue privée 130) : l'écran lit le choix en attente de Rituel de sang et les cristaux à rendre dans `PlayerView::mana_choice`
et `PlayerView::crystals_to_return` (étapes 3 et 4), les modificateurs dans `PlayerView::modifiers` (spec 052), la carte détruite
dans les faits du tour (étape 1), les cartes à détruire, les cartes d'offre, les dés et les cibles permis dans les actions légales
(étapes 1, 2, 5, 6, 8). Aucun champ de vue n'est laissé au fil écran.

## Étapes

| # | Étape | Cas | Rouge attendu | Fichiers | Isolée ? | Modèle |
|---|---|---|---|---|---|---|
| 1 | Détruire une carte action (`Cost::DestroyActionCard`, `CardDestroyed`, fait du tour), Décomposition | C6 à C8, L1 | C6 : Rage encore dans la défausse en fin de tour | `card.rs`, `event.rs`, `engine.rs`, `turn_fact.rs`, `advanced_actions.ron`, test `actions_avancees_mana.rs` | non | fil principal |
| 2 | Destination de `OfferCardGained`, champ `gain` de `CardPlay`, Entraînement, Don pour la magie renforcé | C24, C25, C29 à C31, L5 | C29 : Path Finding sur le deck au lieu de la défausse | `card.rs`, `event.rs`, `engine.rs` | non | fil principal |
| 3 | Rituel de sang : options de base, choix en attente (`ChooseTokenColor`, `PayForCrystal`, `DeclineCrystal`), vue `mana_choice` | C1 à C5, C33 (choix) | C4 : la carte joue sans proposer de couleur | `action.rs`, `engine.rs`, `state.rs`, `view.rs` | non | fil principal |
| 4 | Maîtrise du cristal : option par couleur possédée, `ReturnSpentCrystals`, `CrystalsReturned`, vue `crystals_to_return` | C9 à C12, C33 (cristaux) | C10 : aucun cristal revenu en fin de tour | `modifier.rs`, `engine.rs`, `view.rs` | non | fil principal |
| 5 | Tempête de mana : champ `die`, `DieRerolled` (point de non-retour), relance de la Source, dés de plus, noir et or | C13 à C16, L4 | C13 : pas de cristal vert | `card.rs`, `event.rs`, `engine.rs` | non | fil principal |
| 6 | Apprentissage : `LearningDiscount`, `LearnAdvancedAction`, `AdvancedActionLearned` | C26 à C28, L2 | C26 : `LearnAdvancedAction` absente des actions légales | `action.rs`, `modifier.rs`, `engine.rs` | non | fil principal |
| 7 | Effet maximum, moteur de répétition : `with` sur `CardPlay::Basic`, `Repeat`, options répétables, cible détruite | C17 à C19, C22, L3 | C17 : Attaque 2 au lieu de 6 | `card.rs`, `engine.rs` | non | fil principal |
| 8 | Effet maximum : gains, Blessures, cibles refusées, Concentration | C20, C21, C23 | C23 : Improvisation proposée comme cible | `card.rs`, `engine.rs` | non | fil principal |
| 9 | Contenu complet, persistance (parties préparées régénérées si le format change) | C32 | C32 : une carte encore `Unsupported` | `content/*.ron`, `arcanum-persistance` | non | fil principal |
| 10 | Libellés `fr.json` et `en.json` (section Libellés, textes « écarter » corrigés) | — | — | `assets/i18n/*.json` | oui | Haiku |
| 11 | Écran : carte à détruire dans la main, carte d'offre dans la fenêtre des offres, dé de la Source, cible d'Effet maximum | C8, C13, C23, C29 | `game_screen` : Marche pas cliquable pour Entraînement | `arcanum-ui` | non | fil principal |
| 12 | Écran : fenêtre de Rituel de sang, bouton Apprendre, effets en cours et cristaux à rendre, faits du tour | C4, C5, C26, C33 | `game_screen` : pas de choix de couleur proposé | `arcanum-ui` | non | fil principal |
| 13 | Partie préparée (main de mana et Effet maximum, offres choisies) et vérification à l'œil | — | — (vérification à l'œil) | `arcanum-persistance`, `scripts/` | non | fil principal |
| 14 | Docs : `architecture.md`, `PLAN.md` (implémentée), index par notion, glossaire, bilan de fin de spec | — | — | `docs/` | oui | Sonnet |

## Risques repérés

- **Effet maximum** : la répétition touche le calcul de chaque sorte d'effet ; la liste des options répétables (Q1) est la garde.
  Un test énumère les cartes du contenu et vérifie, pour chacune, si elle est proposée comme cible (C23).
- **Nombre d'actions légales** : Effet maximum (cible × option) et Entraînement (carte détruite × carte d'offre) multiplient les
  `PlayCard` ; Rituel de sang renforcé passe par un choix en attente pour ne pas proposer 56 combinaisons de couleurs.
- **Littéraux de `CardPlay::Basic`** dans les tests et le joueur scripté : nouveaux champs `with`, `gain`, `die` (signalé dans la
  PR) ; `serde(default)` pour les sauvegardes et les parties préparées.
- **Hasard** : la relance de Tempête de mana tire dans le générateur de la Source ; aucun parcours existant ne la joue.
