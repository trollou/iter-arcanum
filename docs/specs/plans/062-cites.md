# Plan de développement — spec 062 Cités

- Statut : réalisé (fil moteur, étapes 1 à 8, puis fil écran, étapes 9 à 12, 2026-11-19), découpage fixé par la spec (Q6 : deux fils).
- Spec : [`062-cites.md`](../062-cites.md). Branche : `spec/062-cites`.
- Méthode : celle de `CLAUDE.md` (tests rouges, code minimal, refactor ; `scripts/verifier.sh` vert avant chaque push).

## Découpage en PR

Une seule PR (privée 339), en brouillon jusqu'à la fin du fil écran.

## Découpage en fils

- **Fil 1 (moteur)** : étapes 1 à 8 (C1 à C36, L1 à L10). Le fil adapte arcanum-ui juste ce qu'il faut pour compiler
  (`face_down_defenders` à plusieurs couleurs), sans rien afficher de neuf.
- **Fil 2 (écran)** : étapes 9 à 12 (C37 à C41), parties préparées, vérification à l'œil, docs.

**Contrôle des champs de vue** (issue privée 130) : C37 lit `PlayerView::cities`, `face_down_defenders`, `enemy_tokens` (étape 8) ;
C38 lit le scénario (`PlayerView` a déjà le scénario) ; C39 lit `CombatEnemy` (valeurs effectives, étape 4) ; C40 lit
`PlayerView::interaction` (bonus des Boucliers, étape 6) ; C41 lit `PlayerView::city_hand_bonus` (étape 7).

## Étapes

| # | Étape | Cas | Rouge attendu | Fichiers | Isolée ? | Modèle |
|---|---|---|---|---|---|---|
| 1 | Contenu `cities.ron`, `CityRule` et `CityLevels` du scénario | C1 à C3 | C1 : `city_garrison` vide ; C3 : sandbox sans niveaux | `content/cities.ron`, `content.rs`, `scenario.rs` | non | fil principal |
| 2 | Révélation : `CityPlaced`, défenseurs selon le niveau, `CityState`, retournés à côté même de nuit, trajet arrêté, cités fermées | C4 à C8, L1 (tirage) | C4 : pas de `CityPlaced` ; C7 : rien de retourné de nuit | `engine/mod.rs`, `engine/decide.rs`, `event/`, `apply/map.rs`, `state.rs`, `view.rs` | non | fil principal |
| 3 | Assaut : pas à coût 2, `AssaultStarted`, combat contre tous les défenseurs fortifiés, Attaque souterraine | C9 à C11, C18, L2, L3 | C9 : pas vers la cité refusé | `engine/legal/mod.rs`, `engine/decide.rs`, `engine/legal/spells.rs`, `apply/combat.rs` | non | fil principal |
| 4 | Bonus de la cité en combat : armure, attaque, Brutal, Toxique, invocation, capacités imprimées | C12 à C17, L8, L9 | C12 : armure 7 au lieu de 8 | `combat.rs`, `apply/combat.rs` | non | fil principal |
| 5 | Issue : Boucliers dans l'ordre, conquête, Seigneur, retraite, pas d'interaction ensuite | C19 à C22, L1 (sans défenseur) | C19 : pas de `CityShieldPlaced` | `engine/decide.rs`, `event/`, `apply/sites.rs` | non | fil principal |
| 6 | Cité conquise : case ordinaire, interaction et bonus des Boucliers, recrutement, achats de chaque couleur, pas de soin | C23 à C31, L5 à L7 | C24 : bonus des Boucliers absent | `engine/legal/sites.rs`, `engine/decide.rs`, `action.rs`, `event/`, `apply/sites.rs`, `reputation.rs` | non | fil principal |
| 7 | Limite de main (plus grand bonus forteresse ou cité), Le Conquérant | C32, C33, L10 | C32 : pioche de 5 au lieu de 7 | `state.rs`, `events/turn_end.rs`, `game_end.rs` | non | fil principal |
| 8 | Vue, retour en arrière, persistance ; tests existants du bac à sable | C34 à C36 | C34 : `PlayerView::cities` vide | `view.rs`, `event/`, `arcanum-persistance/tests` | non | fil principal |
| 9 | Libellés `fr.json` et `en.json` (tableau de la spec) | — | — | `assets/i18n/*.json` | oui | Haiku |
| 10 | Infobulle d'une cité, cité fermée | C37, C38 | C37 : lignes absentes | `arcanum-ui/src/board/` | non | fil principal |
| 11 | Écran de combat, interaction, limite de main | C39 à C41 | C40 : boutons absents | `arcanum-ui/src/` | non | fil principal |
| 12 | Parties préparées, vérification à l'œil, docs (`architecture.md`, `verification-manuelle.md`, `index-par-notion.md`, `glossaire.md`, `PLAN.md`) | — | — (vérification à l'œil) | `arcanum-persistance`, `docs/` | non | fil principal |

## Choix du modèle par étape

Tout le fil moteur au fil principal : les étapes se suivent sur les mêmes fichiers (`decide.rs`, `combat.rs`). Seuls les
libellés (étape 9) se délèguent.

## Risques repérés

- `Scenario` est `Copy` : la liste des niveaux est un tableau borné (`CityLevels`, quatre cités au plus, autant que de tuiles
  de cité) plutôt qu'un `Vec<u8>` (nom indicatif de la spec), pour ne pas retirer `Copy` partout.
- `face_down_defenders` passe à plusieurs couleurs par case : l'écran et ses tests suivent (arcanum-ui).
- Les tests qui révèlent la tuile de cité du bac à sable voient maintenant des défenseurs : chaque assertion changée est
  signalée dans la PR.
