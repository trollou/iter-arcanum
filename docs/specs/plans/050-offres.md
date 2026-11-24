# Plan de développement — spec 050 Offres et cartes gagnées

- Statut : suivi (implémentation dès la validation de la spec, 2026-10-31, comme pour les 032 et 033).
- Spec : [`050-offres-et-cartes-gagnees.md`](../050-offres-et-cartes-gagnees.md). Branche : `spec/050-offres-et-cartes-gagnees`.
- Méthode : celle de `CLAUDE.md` (tests rouges, code minimal, refactor ; `scripts/verifier.sh` vert avant chaque push).

## Découpage en PR

Une seule PR (privée 104, qui porte déjà la spec) : le contenu sans offres ne se voit pas, et les offres sans le choix du niveau pair
n'ont pas d'usage.

## Étapes

Chaque étape = un commit de tests (rouges, local) puis un commit de code (vert), sauf mention.

| # | Étape | Cas | Fichiers | Isolée ? | Modèle |
|---|---|---|---|---|---|
| 1 | **Contenu** : `AdvancedAction`, `Spell`, `Artifact`, `Unit` (`UnitCard`, `UnitRank`, `RecruitSite`), `DeedCard::Advanced/Spell/Artifact`, quatre fichiers RON validés ; `EffectText` gagne les attaques à distance de feu et de glace | C20 | `card.rs`, nouveau `offer.rs`, `content.rs`, `content/*.ron`, `tests/contenu_en_donnees.rs` | non | fil principal |
| 2 | **Pioches et offres à la mise en place** : `CommonDeck`, `Offer`, `RngUsage::CommonDeck`, paramètres du scénario, `CommonDeckShuffled`, `OfferCardDealt`, vue et filtrage | C1 à C4, C21 | `offer.rs`, `rng.rs`, `scenario.rs`, `event.rs`, `engine.rs`, `state.rs`, `view.rs`, nouveau `tests/offres.rs` | non | fil principal |
| 3 | **Renouvellement** à la nouvelle manche : `OfferCardReturned`, unités d'élite après une tuile royaume | C5 à C9, L2 | `engine.rs` | non | fil principal |
| 4 | **Action avancée d'un niveau pair** : choix en attente, `GainAdvancedAction`, `OfferCardGained`, suite de la fin du tour ; décompte Le Savant | C10 à C16, L1, L3 à L7 | `engine.rs`, `action.rs`, `game_end.rs` | non | fil principal |
| 5 | **Jouer une carte gagnée** (Éclairs, Bouclier de glace, carte non codée) et sauvegarde | C17 à C19, C22 | tests seulement, sauf correction | non | fil principal |
| 6 | **Libellés** : noms (fr proposés, issue privée 105) et textes écrits par nous des effets non codés et des capacités d'unités | — | `assets/i18n/*.json` | oui | Sonnet (brief : liste des clés, fichiers de `docs/regles/cartes/`) |
| 7 | **Écran** : fenêtre Offres (bouton, touche O), choix de l'action avancée (touches 1 à 3), main (couleur, texte non codé) | — | `arcanum-ui/src/screen/`, `session.rs`, `app/` | non | fil principal |
| 8 | **Partie préparée** `niveau-pair` et vérification à l'œil sous Xvfb | — | `arcanum-persistance/tests/parties_preparees.rs`, `scripts/parties/` | non | fil principal |
| 9 | **Docs** : architecture, PLAN, index, glossaire, raccourcis, vérification manuelle | — | docs | oui | fil principal (court) |
| 10 | **Bilan de fin de spec** | — | — | non | fil principal |

## Choix du modèle par étape

Seuls les libellés (étape 6) sont délégués : environ 140 textes courts en deux langues, à écrire depuis les fichiers de
`docs/regles/cartes/` sans connaître le moteur. Le reste suit les mêmes fichiers du moteur et de l'écran.

## Risques repérés

- **Parcours et joueurs de test** : un niveau pair attend désormais un choix ; un parcours qui atteint le niveau 2 doit le faire.
- **Graines** : les pioches communes ont leurs propres générateurs (ADR 0007), les autres tirages ne bougent pas (C2).
- **Parties préparées** : les fichiers de `scripts/parties/` changent (nouveaux événements de mise en place), à régénérer.
