# Plan de développement — spec 071 Icônes et raccourcis visibles

- Statut : réalisé (2026-11-06, huit étapes dans un seul fil).
- Spec : [`071-icones-et-raccourcis.md`](../071-icones-et-raccourcis.md). Branche : `spec/071-icones-et-raccourcis`.
- Méthode : celle de `CLAUDE.md` (tests rouges, code minimal, refactor ; `scripts/verifier.sh` vert avant chaque push).

## Découpage en PR

Une seule PR, la privée 156 de la rédaction.

## Découpage en fils

Un seul fil : 8 étapes, écran seul (rien dans arcanum-core, ni contenu, ni persistance). Chaque étape dessine aussi ce qu'elle
calcule, puisque les types de l'écran changent et que `app/` doit compiler à chaque commit.

**Contrôle des champs de la vue** (issue privée 130) : tous ceux de la section « Information cachée » de la spec existent déjà
(`move_points`, `influence_points`, `fame`, `level`, `next_level_fame`, `armor`, `hand_limit`, `command_tokens`, `inventory`,
`mana_tokens`, `combat`, `units`, `hero`) ; aucune étape moteur.

## Choix de conception

- `crates/arcanum-ui/src/icons.rs` (nouveau) : `Icon` (Mouvement, Influence, Armure, Gloire, `Attack { kind, element }`,
  `Block(element)`, `Resistance(resistance)`, `Sideways`), `Icon::ALL`, `image()` (`ImageId` de `ui`) et `name_key()` ;
  `attack_icon(kind, element)`, `block_icon(element)` ; images sans nom : `crystal_image(ManaColor)`, `CRYSTAL_EMPTY_IMAGE`,
  `hero_emblem(hero)` ; `ui_images()` les liste toutes (C1, C26). `IconValue { icon, value, tooltip }` et
  `IconButton { icon, label, mark, tooltip, command }` (`label` : le texte de repli de L1).
- Lignes du panneau : un type `IconLine { icons: Vec<IconValue>, text: String }` pour les points (le Soin reste en texte au
  bout), la gloire et le niveau.
- Carte d'ennemi et créature invoquée : `values: Vec<ValuePiece>` (`Icon(IconValue)` ou `Text(String)` pour « invocation »),
  `resistances: Vec<IconValue>`. Le plateau garde `token_values` en texte (hors périmètre).
- Raccourcis : `crates/arcanum-ui/src/keys.rs` (nouveau) : `KEY_BINDINGS`, la table (touche, modificateur, `Shortcut`) que lit
  `app/mod.rs` ; `shortcut_label(Shortcut, catalog)` en tire le texte (C25) ; `shortcut_hint` compose « Raccourci : … ».
  `shortcut: Option<String>` sur `Button`, `ControlButton`, `StrongButton`, `PurchaseButton`, `ActionButton`, `CrystalButton`
  et les boutons « Prendre » des offres. Les tests qui construisent un `Button` champ par champ gagnent `shortcut: None`
  (changement mécanique, signalé dans la PR).

## Étapes

| # | Étape | Cas | Rouge attendu | Fichiers | Isolée ? | Modèle |
|---|---|---|---|---|---|---|
| 1 | Jeu d'icônes : SVG de la planche copiés dans `assets/images/ui/` et `flag/`, `Icon`, images, libellés `icon.*` | C1, C2, C26 | C1 : fichiers et clés absents ; C2 : `attack_icon` en `todo!()` | `icons.rs`, `assets/images/`, `assets/i18n/*.json`, `tests/icones.rs` | non | fil principal |
| 2 | Panneau : points, gloire, niveau en icônes ; cristaux et jetons dessinés avec les SVG ; repli texte (L1) | C3 à C7, L1 | C3 à C5 : lignes encore en texte | `screen/mod.rs`, `level.rs`, `app/side_panel.rs`, `app/widgets.rs` | non | fil principal |
| 3 | Combat : valeurs et résistances des ennemis et de la créature, total en cours | C8 à C12 | C8, C10 à C12 : valeurs en texte, badges | `screen/combat.rs`, `app/combat.rs` | non | fil principal |
| 4 | Unités : armure et résistances en icônes | C13 | C13 : texte « armure 3 » | `screen/units.rs`, `app/widgets.rs` | non | fil principal |
| 5 | Jouer de côté : encart et boutons-icônes, plus de menu | C14 à C17 | C14 à C16 : boutons sans icône ni infobulle | `screen/hand.rs`, `app/widgets.rs` | non | fil principal |
| 6 | Table des touches ; raccourcis des contrôles, de la fin du tour et du combat | C18 à C20, C24, C25 | C18 à C20, C25 : `shortcut` vide | `keys.rs`, `app/mod.rs`, `controls.rs`, `end_turn.rs`, `combat.rs` | non | fil principal |
| 7 | Raccourcis des sites, des choix numérotés et d'Échap | C21 à C23 | C21 à C23 : `shortcut` vide | `screen/*.rs`, `app/windows.rs` | non | fil principal |
| 8 | Vérification à l'œil (Xvfb), docs : `raccourcis.md`, `architecture.md`, `PLAN.md`, `index-par-notion.md` | — | — | `docs/` | oui | fil principal |

## Risques repérés

- Beaucoup de tests existants comparent des textes que la spec change (`points.line`, `fame.line`, `level.stats`,
  `units.line`, valeurs d'ennemis, menu de côté) : chaque test changé est listé dans la PR avec sa raison (la spec le
  remplace).
- Rendu des SVG à 16-20 px : lisibilité à vérifier à l'œil à l'étape 8, retouche des SVG seulement avec l'accord de Mathieu
  (Q4 : ils font foi).
