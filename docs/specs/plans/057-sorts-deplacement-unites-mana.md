# Plan de développement — spec 057 Sorts : déplacement, unités et mana

- Statut : rédigé par le fil moteur (2026-11-12), à relire par Mathieu avec la PR.
- Spec : [`057-sorts-deplacement-unites-mana.md`](../057-sorts-deplacement-unites-mana.md). Branche :
  `spec/057-sorts-deplacement-unites-mana` (PR privée 252, rédaction et implémentation dans la même PR).
- Méthode : celle de `CLAUDE.md` (tests rouges, code minimal, refactor ; `scripts/verifier.sh` vert avant chaque push).

## Découpage en fils

Deux fils successifs sur la même branche (issue privée 118 : 14 étapes, moteur et écran) :

1. **Fil moteur** : étapes 1 à 9 (arcanum-core, contenu, persistance). Il pousse l'étape 9 verte et s'arrête.
2. **Fil écran** : étapes 10 à 14 (libellés, écrans, partie préparée, vérification à l'œil, docs, bilan).

Contrôle (issue privée 130) : l'écran lit les arrivées d'un déplacement par effet, les unités cibles, les ennemis d'Ailes de la nuit,
les dés de Puits de mana et les capacités des unités de l'offre dans les **actions légales** (étapes 2, 3, 6, 8) ; le dé gardé,
les cartes écartées, le tour supplémentaire et les cartes de Méditation en attente dans `PlayerView` (étapes 5, 7, 8, 9) ; les
modificateurs nouveaux dans `PlayerView::modifiers` (étapes 4, 6, 7, 8). Aucun champ de vue n'est laissé au fil écran.

## Choix de modèle (noms indicatifs de la spec précisés)

- **`CardPlay::Cast`** reste `Copy` (il est copié partout) : les listes de la spec (`units`, `enemies`) sont des `Picks<T>`, une
  liste courte de taille fixe (8 au plus, `Copy`). Champs ajoutés, tous `serde(default)` : `destination: Option<Hex>`,
  `units: Picks<UnitCard>` (Renaissance, Flux d'énergie), `enemies: Picks<EnemyToken>` (Ailes de la nuit), `die: Option<u8>`
  (Puits de mana). La couleur de Brûlure de
  mana et de Lecture d'esprit est une option par couleur (comme Éclair de mana), sans champ `color`.
- **`HeroMoved`** ne change pas de forme : un déplacement par effet est un `HeroMoved` de coût 0 (Voyage souterrain) ou du coût
  payé (Ailes du vent), sans `EnemiesProvoked` (la provocation est déjà un événement à part). Pas de champ `by_effect`, signalé
  dans la PR.
- **Méditation** : `DiscardPicked` (tirage, Méditation) puis le choix en attente `PlayerState::meditation` ; pour Transe, le
  choix en attente porte sur toute la défausse et `PlaceOnDeck::order` nomme les cartes choisies.
- **`DrawBonus`** : durée nouvelle « jusqu'à la prochaine pioche », close par un événement `DrawBonusUsed` produit avec la
  pioche de fin de tour qui tire au moins une carte, ou celle de la nouvelle manche (C21 : deck vide au dernier tour).
- **Appel à la gloire** : le sort pose `RecruitFromOffer` (une fois ce tour) ; `RecruitUnit` (existante) devient légale pour
  toute unité de l'offre, partout, combat compris, et produit `UnitRecruited { cost: 0 }`, sans champ `recruit`/`disband` dans
  `CastTargets` (l'écran réutilise « Recruter ») ; un recrutement gratuit ne compte pas pour Récit héroïque. Tant que le
  recrutement gratuit attend, un recrutement au village le consomme aussi (pas d'Influence dépensée).
- **Appel aux armes** : `UnitActivated` d'une unité de l'offre consomme `UnitFromOffer`, sans épuiser ni blesser personne.

## Étapes

| # | Étape | Cas | Rouge attendu | Fichiers | Isolée ? | Modèle |
|---|---|---|---|---|---|---|
| 1 | Sorts sans mécanique nouvelle : Lecture d'esprit, Brûlure de mana (Fuite de base `OthersOnly`), Restauration et Renaissance (`units`), Flux et Vol d'énergie ; Don pour la magie lance tous les sorts | C2, C10, C11, C22, C25, C26, L3 | C25 : Lecture d'esprit absente des actions légales | `card.rs`, `engine.rs`, `combat.rs`, `spells.ron`, test `sorts_deplacement.rs` | non | fil principal |
| 2 | Voyage souterrain et Attaque souterraine (`destination`, assaut à distance, retraite au départ) | C3 à C6, L1 | C3 : Voyage souterrain vers le village absent | `card.rs`, `engine.rs`, `movement.rs` | non | fil principal |
| 3 | Ailes du vent, Ailes de la nuit (`enemies`), Mouvement en combat | C7 à C9, L2 | C7 : Ailes du vent vers le village absente | `card.rs`, `engine.rs`, `combat.rs` | non | fil principal |
| 4 | Distorsion spatiale : voisinage à 2, trajets, exploration, sans provocation | C12 à C14 | C12 : `(1, -2)` pas atteignable en un pas | `modifier.rs`, `movement.rs`, `engine.rs` | non | fil principal |
| 5 | Distorsion temporelle : cartes rendues, pas de pioche, tour supplémentaire, carte écartée jusqu'à la fin de la manche | C15, C16 | C15 : Marche dans la défausse au lieu de la main | `event.rs`, `state.rs`, `engine.rs` | non | fil principal |
| 6 | Appel aux armes (`UnitFromOffer`, unité de l'offre activée) et Appel à la gloire (recrutement gratuit) | C17, C18, L6 | C17 : capacité des Arbalétriers de l'offre absente | `modifier.rs`, `engine.rs` | non | fil principal |
| 7 | Méditation et Transe : `RngUsage::DiscardPick`, `DiscardPicked`, `PlaceOnDeck`, `DrawBonus` | C19 à C21, L4 | C19 : aucun `DiscardPicked` | `action.rs`, `event.rs`, `rng.rs`, `engine.rs` | non | fil principal |
| 8 | Puits de mana : `DieKept`, dé gardé hors de la Source, `ManaClaimPerTurn` (durée manche) | C23, C24, L5 | C23 : aucun `DieKept` | `event.rs`, `modifier.rs`, `engine.rs` | non | fil principal |
| 9 | Contenu complet, vue, persistance (parties préparées régénérées si le format change) | C1, C27, C28 | C1 : un sort encore `Unsupported` ; C27 : `kept_die` absent de la vue | `content/*.ron`, `view.rs`, `arcanum-persistance` | non | fil principal |
| 10 | Libellés `fr.json` et `en.json` (section Libellés) | — | — | `assets/i18n/*.json` | oui | Haiku |
| 11 | Écran : arrivée d'un déplacement par effet, Distorsion spatiale, Ailes de la nuit, cibles d'unités | C3, C7, C8, C11, C12 | `board_screen` : pas d'arrivée cliquable pour Voyage souterrain | `arcanum-ui` | non | fil principal |
| 12 | Écran : Méditation, Puits de mana, offre d'unités (Utiliser, Recruter gratuit), tour supplémentaire, cartes écartées, effets en cours | C15, C17 à C19, C23, C27 | `game_screen` : pas de fenêtre de Méditation | `arcanum-ui` | non | fil principal |
| 13 | Partie préparée et vérification à l'œil | — | — (vérification à l'œil) | `arcanum-persistance`, `scripts/` | non | fil principal |
| 14 | Docs : `architecture.md`, `PLAN.md` (implémentée), index par notion, glossaire, bilan de fin de spec | — | — | `docs/` | oui | Sonnet |

## Risques repérés

- **Nombre d'actions légales** : sous-ensembles d'unités (Renaissance) et d'ennemis (Ailes de la nuit) × paiements × arrivées ;
  la Transe passe par un choix en attente pour ne pas multiplier les paires de la défausse.
- **Littéraux de `CardPlay::Cast`** dans les tests de la spec 054 et dans arcanum-ui : nouveaux champs, signalé dans la PR.
- **Trajets** (`routes_ending_in`) : la clé de départage est une suite de `Direction` ; avec Distorsion spatiale, un saut a douze
  directions de plus. Les tests des specs 017 et 036 gardent l'ordre sans le sort.
- **Fin du tour** : Distorsion temporelle et `DrawBonus` touchent `turn_end_rewards` et `turn_end_draw` ; les tests des specs 016,
  023, 035 et 063 gardent le comportement sans sort.
