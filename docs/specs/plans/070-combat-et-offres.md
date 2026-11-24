# Plan de développement — spec 070 Combat et offres

- Statut : suivi (implémentation dès la validation de la spec, 2026-11-06).
- Spec : [`070-combat-et-offres.md`](../070-combat-et-offres.md). Branche : `spec/070-combat-et-offres`.
- Méthode : celle de `CLAUDE.md` (tests rouges, code minimal, refactor ; `scripts/verifier.sh` vert avant chaque push).

## Découpage en PR

Une seule PR (privée 152, qui porte déjà la spec).

## Découpage en fils

Un seul fil : rien dans arcanum-core, six étapes. Contrôle de l'issue privée 130 : tout ce que l'écran calcule se lit déjà dans
`PlayerView::combat` (`in_progress_total`, `block_needed`, armures des cibles) ; `UndoStatus` vient du `GameRunner`, déjà
exposé par `undo_status()`.

## Étapes

| # | Étape | Cas | Rouge attendu | Fichiers | Isolée ? | Modèle |
|---|---|---|---|---|---|---|
| 1 | **Ennemi coché d'office** : une seule cible possible, pas de case, « Attaquer » actif | C1 à C4, L1 | C1 : la carte a encore sa case, « Attaquer » grisé | `screen/combat.rs`, nouveau `tests/combat_et_offres.rs`, tests 031 qui cochaient l'ennemi seul | non | fil principal |
| 2 | **Carte d'ennemi et « Parer »** : nom seul, famille en infobulle, état facultatif, enjeu sur deux lignes, historique compté ; libellés | C5, C6, C13 à C16 | C5 : « Parer Rôdeurs (orques maraudeurs) » ; C13 : nom avec famille, infobulle vide (squelette) | `screen/combat.rs`, `assets/i18n/*.json`, `deroule_combat.rs` | non | fil principal |
| 3 | **Attaque et parade insuffisantes** : `combat_screen` reçoit `UndoStatus`, commandes `CancelBlock` et `CancelAttack`, Entrée et Échap | C7 à C12, C19 à C21, L3 | C7 : « Valider la parade » au lieu d'« Annuler » ; C11 : `CancelBlock` sans effet (squelette) | `screen/combat.rs`, `screen/types.rs`, `session.rs`, appelants de `combat_screen` | non | fil principal |
| 4 | **Fenêtre Offres sans taille de pioche** | C17 | C17 : « Pioche : 25 » encore présent | `screen/offers.rs`, `tests/offres.rs`, `assets/i18n/*.json` | non | fil principal |
| 5 | **Rendu egui** (carte d'ennemi, cadre en cours, offres à la taille normale) et vérification à l'œil sous Xvfb | C18 | — | `app/combat.rs`, `app/windows.rs` | non | fil principal |
| 6 | **Docs** : PLAN, raccourcis, architecture, index par notion | — | — | docs | oui | fil principal (court) |

## Choix du modèle par étape

Rien de délégué : les étapes suivent les mêmes fichiers de l'écran, et les libellés tiennent en six clés.

## Risques repérés

- **C4** dit « un seul ennemi, fortifié » : depuis la spec 032 (Q1), un ennemi fortifié une fois reste une cible à distance
  (pour le siège). Le seul ennemi sans cible possible en phase d'attaque à distance est le doublement fortifié (spec 043, C39) :
  le test C4 prend celui-là, dans `sites_fortifies.rs` qui a déjà le décor d'un assaut.
- **Tests existants** : ceux qui cochent l'ennemi seul ou qui valident une parade ratée par le bouton changent (section « Effets
  sur les specs déjà implémentées » de la spec).
