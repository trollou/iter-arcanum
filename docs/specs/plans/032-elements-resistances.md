# Plan de développement — spec 032 Éléments et résistances

- Statut : suivi (Mathieu a demandé de démarrer l'implémentation dès la validation de la spec, 2026-10-29).
- Spec : [`032-elements-resistances.md`](../032-elements-resistances.md). Branche : `spec/032-elements-resistances`.
- Méthode : celle de `CLAUDE.md` (tests rouges, code minimal, refactor ; `scripts/verifier.sh` vert avant chaque push).

## Découpage en PR

Une seule PR (privée 70, qui porte déjà la spec) : 23 cas, aucun événement nouveau, l'écran de combat existe déjà (031).

## Étapes

Chaque étape = un commit de tests (rouges, local) puis un commit de code (vert), sauf mention.

| # | Étape | Cas | Fichiers | Isolée ? | Modèle |
|---|---|---|---|---|---|
| 1 | **Éléments et siège dans le contenu** : `Element::Fire`, `ColdFire`, `AttackKind::Siege`, effets RON, Polyvalence renforcée | C22 (moteur), C11 | `card.rs`, `content.rs`, `basic_actions.ron` | non | fil principal |
| 2 | **Efficacité et points par élément** : table d'efficacité, points de l'attaque ou de la parade en cours par élément, total qui compte (une division), historique | C1 à C10, C14 à C16 | `combat.rs`, `engine.rs`, nouveau test `crates/arcanum-core/tests/elements_resistances.rs` | non | fil principal |
| 3 | **Siège et fortifiés** : cible fortifiée en phase d'attaque à distance, siège seul accepté ; 031 C9 et C25 ajustés (signalés dans la PR) | C12, C13 | `combat.rs`, `tests/deroule_combat.rs` | non | fil principal |
| 4 | **Rejeu et sauvegarde** | C17 | tests | non | fil principal |
| 5 | **Libellés** `fr.json` et `en.json` (tableau de la spec, `combat.ignored` modifié) | — | `assets/i18n/*.json` | oui | fil principal (court) |
| 6 | **Écran** : libellés des effets, badges de résistance, bandeau « Le groupe partage », repères ÷2 et « siège seulement », ligne de calcul, phrase de la parade | C18 à C23 | `screen/combat.rs`, `screen/text.rs`, `screen/hand.rs`, `app.rs` | non | fil principal |
| 7 | **Partie préparée** `combat-elements` et vérification à l'œil sous Xvfb | — | `crates/arcanum-persistance/tests/parties_preparees.rs`, `scripts/parties/` | non | fil principal |
| 8 | **Docs** : architecture, PLAN et index (« implémentée »), tableau « ignoré » de la 031 | — | docs | oui | fil principal |
| 9 | **Bilan de fin de spec** : points à améliorer, une issue par point confirmé avec Mathieu | — | — | non | fil principal |

## Choix du modèle par étape

Tout reste au fil principal : les étapes se suivent sur `combat.rs` et l'écran de combat, et les seules isolées (libellés, docs)
sont trop courtes pour valoir un brief (issue privée 66).

## Risques repérés

- **Points par élément** : l'attaque en cours garde aujourd'hui un seul total (`points: u32`). Le remplacer par des points par élément
  change la vue (`PlayerView`) et les tests de la 031 et d'arcanum-ui qui la construisent.
- **Fortifiés ciblables à distance** : plus de `DeclareAttack` légaux ; les joueurs de test « premier choix » peuvent déclarer une
  attaque sans siège et la rater. Pas de partie infinie : une attaque ratée bloque la cible jusqu'à la phase suivante.
- **Quota CI** épuisé jusqu'au 1er novembre environ : vérification locale par `scripts/verifier.sh`, fusion seulement avec l'accord de
  Mathieu.
