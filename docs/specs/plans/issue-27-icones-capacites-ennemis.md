# Plan de développement — spec #27 Icônes des capacités des ennemis

- Statut : accepté (spec Effort Medium, un fil ; plan de 5 étapes, écran seulement).
- Spec : [`issue-27-icones-capacites-ennemis.md`](../issue-27-icones-capacites-ennemis.md). Branche :
  `feature/27-icones-capacites-ennemis`.
- Méthode : celle de `CLAUDE.md` (tests rouges, code minimal, refactor ; `scripts/verifier.sh` vert avant chaque push).

## Découpage

Une PR (#85, celle de la rédaction), un fil : rien dans arcanum-core, 5 étapes.

**Contrôle des champs de vue** : les capacités du jeton (`definition().abilities`), celles de la cité
(`CombatEnemy::city_abilities`), les résistances (`effective_resistances`) et l'élément d'attaque sont déjà lus par
`screen/combat.rs` : aucune étape moteur.

## Étapes

| # | Étape | Cas | Rouge attendu | Fichiers | Isolée ? | Modèle |
|---|---|---|---|---|---|---|
| 1 | Six icônes (`Icon::Ability`, `Icon::Summon`), SVG copiés dans `assets/images/ui/`, libellés de la spec (`icon.ability.*`, `icon.summon`, `icon.rule.*`, `effect.double_if`) | C1, C2 | C1 : 38 images au lieu de 44 ; C2 : 26 icônes au lieu de 32 | `icons.rs`, `assets/` | non | fil principal |
| 2 | `IconValue::rule` ; capacités de la carte d'ennemi et de la créature invoquée en icônes ; icône Invocation ; règles des résistances et des attaques élémentaires de l'ennemi | C3 à C12, L1 | C3 : `abilities` vide (squelette) ; C6 : texte « invocation » ; C10, C11 : `rule` à `None` | `icons.rs`, `screen/combat.rs`, tests `icones.rs`, `deroule_combat.rs`, `cites.rs` | non | fil principal |
| 3 | Texte à icônes : `icon_pieces`, `plain_text`, repères `[[nom]]` dans `fr.json` et `en.json`, `plain_text` dans les libellés de bouton | C15 à C21 (texte) | C15 : `icon_pieces` rend un seul morceau (squelette) ; C16 : aucun repère ; C17 à C20 : textes sans repère | `icons.rs`, `screen/*.rs`, `assets/i18n/`, tests existants (liste de C18) | non | fil principal |
| 4 | Parade vive : condition « ×2 si » + icône Vif sur le bouton | C13, C14, C21 (infobulle) | C13 : pas de condition (squelette) | `screen/types.rs`, `screen/effects.rs`, test `unites.rs` | non | fil principal |
| 5 | Dessin : rangée d'icônes de capacités, infobulle à deux étages, morceaux de texte à icônes dans les libellés et les infobulles, condition du bouton ; partie préparée `unites-vif` ; vérification à l'œil (densité des icônes de l'écran de combat) ; docs (`architecture.md`, `verification-manuelle.md`, `index-par-notion.md`) | L2, L3, Interface | — (vérification à l'œil) ; promesse de la partie préparée : test `parties_preparees.rs` | `app/*.rs`, `scripts/`, `docs/` | non | fil principal |
