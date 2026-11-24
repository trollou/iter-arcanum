# Intégration continue et dépôt

Détail de la CI et de l'automatisation du dépôt public `trollou/iter-arcanum`. Les règles que tout fil doit
connaître (branches, PR, fusion) restent dans `CLAUDE.md`, section « Git et intégration continue » ; cette page
sert aux fils qui touchent `.github/` ou qui s'étonnent d'un check.

## Ce que lance la CI

La CI (`.github/workflows/ci.yml`) lance `fmt`, `clippy` et `test` sous Linux et Windows. Sur un dépôt public,
les minutes GitHub Actions sont illimitées, Windows compris (spec 082, Q6) : plus de filtre de chemins ni de
passage de nuit, qui économisaient le quota du dépôt privé (issues privées 48 et 305).

- **Sur une pull request prête**, Linux et Windows au complet, quel que soit ce qu'elle touche (une PR de doc
  tourne aussi, en quelques minutes). Le job Linux installe d'abord `libasound2-dev` et `pkg-config`, sans
  lesquels le son (cpal, ALSA) ne compile pas (spec 081) ; `scripts/verifier.sh` fait de même dans un fil.
- **Un brouillon ne lance rien** : la CI part quand la PR passe « prête » (`ready_for_review`), puis à
  chaque push tant qu'elle reste prête (l'exécution précédente est annulée). Les fils ouvrent leur PR en
  brouillon et la passent prête à la toute fin, une fois le dernier fil de la spec fini.
- **Sur `main`, à chaque fusion**, les deux OS au complet ; c'est aussi ce passage qui enregistre le cache.
- **À la main** (onglet Actions), sur les deux OS, au complet.
- **Garde-fou du dépôt public** : `cargo test` comprend `crates/arcanum-core/tests/depot_public.rs`, qui
  refuse tout matériel de tiers suivi par git (`docs/regles/`, PDF, images hors de `assets/` et
  `docs/specs/images/`, fichier de plus de 5 Mo ; spec 082, C4 et C5).

## Contrôle qualité

Les outils de qualité (couverture `llvm-cov`, duplication `jscpd`, `cargo machete`, `cargo deny`) ne tournent pas
dans la CI : `scripts/qualite.sh` les lance en fin de spec, la couverture en fin de jalon
(`docs/qualite/README.md`). Décision de Mathieu, 2026-11-17 (issue privée 305), prise pour le quota du dépôt privé.

## Cache

Le cache `rust-cache` n'est enregistré que par `main` (fusion ou à la main) ; les PR le lisent sans
l'enregistrer, pour ne pas saturer le quota de 10 Go du dépôt.

## Branches

GitHub supprime la branche à la fusion de la PR (« Automatically delete head branches »). Pour un plan à
plusieurs PR intermédiaires, restaurer la branche depuis la PR fusionnée ou la recréer depuis `main`.

## Dependabot

Dependabot (`.github/dependabot.yml`) ouvre chaque dimanche soir des PR groupées de mise à jour des
dépendances Cargo (compatibles, egui, majeures), et le 1er du mois une PR pour les actions GitHub. Elles
suivent les mêmes règles que les autres : CI verte, fusion par Mathieu.
