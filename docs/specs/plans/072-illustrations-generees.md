# Plan de développement — spec 072 Illustrations générées

- Statut : fil 1 réalisé (2026-11-06, étapes 1 à 6) ; fil 2 réalisé (2026-11-06) : premier lot généré et validé par Mathieu (41 images).
- Spec : [`072-illustrations-generees.md`](../072-illustrations-generees.md). Branche : `spec/072-illustrations-generees`.
- Méthode : celle de `CLAUDE.md` (tests rouges, code minimal, refactor ; `scripts/verifier.sh` vert avant chaque push).

## Découpage en PR

Une seule PR, la privée 161 de la rédaction.

## Découpage en fils

Deux fils, comme la spec le propose (« Premier lot ») :

1. étapes 1 à 6 : code d'affichage et ses tests, script de génération, les trois essais retenus, vérification à l'œil, docs ;
2. génération du premier lot (38 images de plus), famille par famille, une planche validée par Mathieu à chaque fois.

**Contrôle des champs de la vue** (issue privée 130) : `PlayerView::hand`, `offers`, `drawn_artifacts` et `units` existent ; rien
dans arcanum-core.

## Choix de conception

- `crates/arcanum-ui/src/illustrations.rs` (nouveau) : `IllustrationFamily` (`Unit` → `unit-art`, `Card` → `card-art`),
  `Illustration { family, id }`, `path(images_dir)` (`.png`), `unit_illustration(Unit)`, `card_illustration(DeedCard)`
  (identifiant tiré de la clé de libellé, comme `enemy_image`), `known_illustrations(family)` (tous les éléments de la
  famille, pour C2 et C3) ; `IllustrationIndex::load(images_dir)` lit les deux dossiers une fois (C4, L2), `contains`.
- Champ `illustration: Illustration` sur `HandCard`, `OfferCardView` et `UnitLine` (C5 à C7).
- Repli (L1) : `illustration_paint(load, path, warnings) -> Option<TextureId>`, qui partage `ImageWarnings` avec
  `image_paint` ; `ImageWarnings` retient les chemins signalés (un chemin par image, rien ne change pour les SVG).
- `app/` : l'index est lu dans `IterArcanumApp::new` ; une vignette n'est demandée à egui que si l'index la connaît (C9 :
  aucun log pour une image absente) ; texture avec mipmaps.
- Dépendances (ADR 0009 Q2, acceptée) : fonction `image` d'`egui_extras` et crate `image` réduite au PNG (déjà dans
  `Cargo.lock`, tirée par eframe).

## Étapes

| # | Étape | Cas | Rouge attendu | Fichiers | Isolée ? | Modèle |
|---|---|---|---|---|---|---|
| 1 | `Illustration`, familles, index ; dépendance PNG | C1 à C4, L2, L3 | C1, C4, L2 : fonctions en `todo!()` | `illustrations.rs`, `Cargo.toml`, `tests/illustrations.rs` | non | fil principal |
| 2 | Cartes de la main, des offres, des choix, lignes d'unité portent leur illustration | C5 à C7 | C5 à C7 : champ rempli d'une valeur fixe | `screen/hand.rs`, `offers.rs`, `sites.rs`, `units.rs`, `types.rs` | non | fil principal |
| 3 | Repli d'une image illisible, signalée une fois | L1 | L1 : `illustration_paint` en `todo!()` | `board/images.rs`, `tests/affichage_plateau_logs.rs` | non | fil principal |
| 4 | Dessin des vignettes (main, combat, Offres, choix, unités) | C8, C9 | — (à l'œil) | `app/*.rs`, `main.rs` | non | fil principal |
| 5 | Script `scripts/illustrations.py`, `style.json`, descriptions des trois essais, essais retenus | C2, C3 sur des images réelles | — | `scripts/`, `assets/images/` | non | fil principal |
| 6 | Vérification à l'œil (Xvfb) ; docs : `architecture.md`, `PLAN.md`, `index-par-notion.md` | — | — | `docs/` | oui | fil principal |

## Risques repérés

- Les PNG de 512 px réduits à 64 px : netteté à vérifier à l'œil (mipmaps).
- Budget de génération (5 €) : le fil 1 n'appelle pas l'API, les trois essais existent déjà.
