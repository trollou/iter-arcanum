# Plan de développement — spec #20 Obtention des cartes

- Statut : fil 1 (étapes 1 à 6) fait le 2026-11-27 ; fil 2 (étapes 7 à 9) à faire. Effort High : deux fils.
- Spec : [`issue-20-obtention-des-cartes.md`](../issue-20-obtention-des-cartes.md). Branche :
  `feature/20-obtention-des-cartes`.
- Méthode : celle de `CLAUDE.md` (tests rouges, code minimal, refactor ; `scripts/verifier.sh` vert avant chaque push).

## Découpage en PR

Une seule PR, #90 (la PR de la spec).

## Découpage en fils

1. **Fil 1, moteur et écrans calculés** : étapes 1 à 6. Le dessin egui n'est adapté que pour compiler et rester utilisable
   (nouveaux types lus là où les anciens l'étaient). Pousse la dernière étape verte et s'arrête.
2. **Fil 2, dessin** : étapes 7 à 9, puis PR prête et bilan.

Contrôle (issue privée 130) : chaque élément de la section Interface a son champ produit au fil 1 : fenêtre du niveau pair
(`PlayerView::skill_choice_cards`, étape 1 ; `LevelChoiceWindow`, étape 4), fenêtres de choix (`CardChoiceWindow`, étape 3),
fenêtre Offres (étape 5), infobulle des achats (`PurchaseButton::card`, étape 6).

## Étapes

| # | Étape | Cas | Rouge attendu | Fichiers | Isolée ? | Modèle |
|---|---|---|---|---|---|---|
| 1 | `PlayerView::skill_choice_cards`, règle de la dernière place partagée avec les actions légales | C9 | liste vide | `arcanum-core/src/view.rs`, `engine/legal/mod.rs`, `tests/competences.rs` | non | fil principal |
| 2 | Libellés nouveaux et changés (tableau de la spec) ; retrait des anciens à l'étape qui supprime leur dernier lecteur | — | — | `assets/i18n/*.json` | oui | fil principal |
| 3 | `CardChoiceWindow` (remplace `ArtifactChoice` de l'écran, `close`) : action avancée et unité d'une récompense, carte d'un effet « gagner », « Renoncer », « Annuler », Échap | C11 à C17, L3 | fenêtres vides (`advanced_action_reward`, `unit_reward`, `gain_choice`) | `screen/types.rs`, `screen/sites.rs`, `screen/gain.rs`, `session.rs`, `tests/ruines.rs`, `tests/actions_avancees_mana.rs`, `tests/artefacts.rs` | non | fil principal |
| 4 | `LevelChoiceWindow`, `ScreenMode::ChoosingLevelSkill`, `Command::SelectSkill`, `Command::TakeLevelReward`, touches 1 à 3 | C1 à C8, C10, C19, L2 | « Prendre » actif sans compétence encadrée | `screen/skills.rs`, `session.rs`, `tests/competences.rs`, `tests/offres.rs` | non | fil principal |
| 5 | Fenêtre Offres de consultation seulement : titre « Offres », sans « Renoncer », toujours fermable | C18, C20 | Offres ouverte d'office au niveau pair | `screen/offers.rs`, `session.rs`, `tests/offres.rs` | non | fil principal |
| 6 | `PurchaseButton::card` | C21 à C23 | `card` vide | `screen/sites.rs`, `screen/units.rs`, `tests/villages_et_monasteres.rs`, `tests/unites.rs` | non | fil principal |
| 7 | Dessin : fenêtre du niveau pair (maquette), fenêtres de choix, infobulle des achats | Interface, L1, L4 | — (vérification à l'œil) | `app/windows.rs`, `app/window_cards.rs` | non | fil principal |
| 8 | Parties préparées (promesses), vérification à l'œil | Interface | — | `arcanum-persistance/tests/parties_preparees.rs`, captures | non | fil principal |
| 9 | Docs (specs 045, 050, 051, 056 ; vérification manuelle, architecture, index par notion, raccourcis) | — | — | `docs/` | oui | Haiku |

## Risques repérés

- Les tests de la spec 050, 051 et 045 qui lisent la fenêtre Offres pendant un choix changent de sens (liste dans la spec,
  « Tests existants modifiés ») : chaque changement est signalé dans la PR.
- `TakeLevelReward` joue deux actions : la seconde doit être relue dans les actions légales après la première.
