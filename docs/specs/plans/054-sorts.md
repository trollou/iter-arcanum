# Plan de développement — spec 054 Sorts : lancer un sort, sorts de combat

- Statut : accepté par Mathieu (2026-11-12), avec la spec.
- Spec : [`054-sorts.md`](../054-sorts.md). Branche : `spec/054-sorts`.
- Méthode : celle de `CLAUDE.md` (tests rouges, code minimal, refactor ; `scripts/verifier.sh` vert avant chaque push).

## Découpage en PR

Comme la 056 : la PR de rédaction (spec et plan validés) est fusionnée d'abord ; l'implémentation (deux fils) fait une seconde PR,
sur la branche `spec/054-sorts` recréée depuis `main`.

## Découpage en fils

Deux fils successifs sur la même branche (issue privée 118 : 14 étapes, moteur et écran) :

1. **Fil moteur** : étapes 1 à 9 (arcanum-core, contenu, persistance). Il pousse l'étape 9 verte et s'arrête.
2. **Fil écran** : étapes 10 à 14 (libellés, écrans, partie préparée, vérification à l'œil, docs, bilan).

Contrôle (issue privée 130) : l'écran lit la fortification et les résistances effectives dans `PlayerView::combat` (étape 3),
`DeferredAttack`, `IgnoreSiteFortifications` et `CastFromOffer` dans `PlayerView::modifiers` (étapes 3, 6, 8), l'ennemi détruit dans
les faits du tour (étape 4), les paiements, les ennemis visés et les sorts de l'offre permis dans les actions légales (étapes 1, 2,
8). Aucun champ de vue n'est laissé au fil écran.

## Étapes

| # | Étape | Cas | Rouge attendu | Fichiers | Isolée ? | Modèle |
|---|---|---|---|---|---|---|
| 1 | Payer un sort : base de sa couleur, renforcé couleur et noir (`black` sur `CardPlay::Strong`), règles de nuit du donjon ; Éclair de feu et Tempête de neige (`AttackWithWound` élémentaire) ; Concentration sans sort | C1 à C6, L1 | C1 : Éclair de feu absent des actions légales payées | `card.rs`, `engine.rs`, `spells.ron`, test `sorts.rs` | non | fil principal |
| 2 | Ennemi visé (`enemy` sur `CardPlay`), `OnAllEnemies`, Secousse et Tremblement de terre (`OnEnemyByFortification`) | C10 à C12, L2 | C10 : armure des Swordsmen 5 au lieu de 2 | `card.rs`, `modifier.rs`, `engine.rs` | non | fil principal |
| 3 | `LosesFortifications`, `LosesResistances`, `LosesResistance`, fortification et résistances effectives (combat, résistances aux effets, vue) ; Exposition, Frisson | C7 à C9, C22, C23, C31 (ennemis) | C7 : Diggers encore fortifiés après Exposition | `combat.rs`, `modifier.rs`, `engine.rs`, `view.rs` | non | fil principal |
| 4 | Détruire un ennemi (`EnemyDestroyed`, gloire, créature invoquée, fait du tour, retour en arrière) ; Tourbillon et Tornade | C17, C18, C32, L5 | C18 : Fire Mages pas vaincus | `event.rs`, `engine.rs`, `turn_fact.rs` | non | fil principal |
| 5 | Démolition et Désintégration (`IgnoreSiteFortifications`) | C24 à C26 | C24 : Guardsmen encore doublement fortifiés | `modifier.rs`, `engine.rs` | non | fil principal |
| 6 | Bouclier ardent et explosif : `DeferredAttack`, `UseDeferredAttack`, `DestroyBlockedEnemy` à la parade réussie | C19 à C21, L3 | C19 : `UseDeferredAttack` absente des actions légales | `action.rs`, `modifier.rs`, `engine.rs` | non | fil principal |
| 7 | Mur de flammes, Déluge de flammes (ennemis affrontés), Éclair de mana et Tonnerre de mana (trois mana) | C13 à C16 | C14 : Parade de feu 7 au lieu de 11 | `card.rs`, `engine.rs` | non | fil principal |
| 8 | Don pour la magie de base : `CastFromOffer`, sort lancé depuis l'offre (`from_offer`), le sort reste dans l'offre | C27 à C29, L4 | C27 : Éclair de feu de l'offre absent des actions légales | `card.rs`, `engine.rs`, `advanced_actions.ron` | non | fil principal |
| 9 | Contenu complet, persistance (parties préparées régénérées si le format change) | C30 | C30 : un sort de combat encore `Unsupported` | `content/*.ron`, `arcanum-persistance` | non | fil principal |
| 10 | Libellés `fr.json` et `en.json` (section Libellés, texte de Démolition corrigé) | — | — | `assets/i18n/*.json` | oui | Haiku |
| 11 | Écran : paiement d'un sort (Renforcer grisé de jour), ennemi visé sur l'écran de combat, options | C2, C10, C15, C17 | `combat_screen` : pas de choix d'ennemi proposé pour Secousse | `arcanum-ui` | non | fil principal |
| 12 | Écran : ennemis sans fortification ni résistances, détruit ; bouton d'attaque différée ; bouton Lancer de l'offre ; effets en cours et faits du tour | C19, C27, C31 | `combat_screen` : pas de bouton d'attaque différée | `arcanum-ui` | non | fil principal |
| 13 | Partie préparée (combat de nuit contre une tour de mage, sorts en main et dans l'offre) et vérification à l'œil | — | — (vérification à l'œil) | `arcanum-persistance`, `scripts/` | non | fil principal |
| 14 | Docs : `architecture.md`, `PLAN.md` (implémentée), index par notion, glossaire, bilan de fin de spec | — | — | `docs/` | oui | Sonnet |

## Risques repérés

- **Fortification et résistances effectives** : beaucoup de lecteurs du jeton dans `combat.rs` et `engine.rs` (cibles à distance,
  efficacité, résistances aux effets). Chercher toutes les lectures (`abilities`, `resistances`, `site_fortified`) avant l'étape 3 ;
  les tests existants des specs 032, 043 et 053 gardent le comportement sans sort.
- **Nombre d'actions légales** : ennemi visé × paiements × options (Éclair de mana : trois mana) multiplient les `PlayCard`. Si la
  liste explose, passer par un choix en attente comme Rituel de sang (spec 056), signalé dans la PR.
- **Littéraux de `CardPlay`** dans les tests et le joueur scripté : nouveaux champs `enemy`, `black`, `from_offer` (signalé dans la
  PR) ; `serde(default)` pour les sauvegardes et les parties préparées.
- **Ennemi détruit** : réutiliser la logique de victoire (`EnemiesDefeated` : gloire, réputation, conquête, fin du combat) plutôt
  que la recopier.
