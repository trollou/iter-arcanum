# 070 — Combat et offres

- Statut : implémentée (2026-11-06 ; validée le même jour, Q1 à Q3 tranchées par Mathieu). Plan :
  [`plans/070-combat-et-offres.md`](plans/070-combat-et-offres.md)
- Jalon : IHM (refonte de l'interface, entre J4 et J5)
- Dépend de : 031 (écran de combat), 033 (créature invoquée), 024 (retour en arrière), 050 (fenêtre Offres) ; toutes implémentées
- Livret : aucune règle nouvelle. Rappels : MKUE p. 9, *Ranged and Siege Attack Phase* 5 et 6, *Block Phase* 3 à 6 ;
  p. 5, *Reverting*
- Crates touchées : arcanum-ui, `assets/i18n/` (fr.json et en.json). Rien dans arcanum-core.
- Base : [synthèse de la refonte](refonte-ihm.md), sections « Écran de combat » et « Fenêtre des offres »

## Objectif

Premier lot de la refonte de l'interface : l'écran de combat demande moins de clics et se lit plus vite. Un ennemi seul est
déjà coché pour l'attaque, le bouton de parade dit seulement « Parer », une attaque ou une parade qui ne suffit pas n'est
plus proposée (on l'annule au lieu de la valider) et la carte de chaque ennemi va à l'essentiel. La fenêtre Offres s'écrit à la même taille
que le reste du jeu et n'affiche plus la taille des pioches.

## Règles

Aucune règle n'est ajoutée ni changée : le moteur propose les mêmes actions légales, seul l'écran en montre moins.

- **Attaque ratée** (MKUE p. 9, *Ranged and Siege Attack Phase* 6) : sans effet, les cartes jouées sont perdues et la cible ne
  peut plus être attaquée dans la phase (spec 031, Q10). Valider une attaque insuffisante ne fait donc que perdre les cartes
  jouées : l'écran ne la propose plus, comme pour la parade (Q2). Le moteur la garde comme action légale.
- **Parade ratée** (MKUE p. 9, *Block Phase* 6) : l'ennemi n'est pas paré et ne peut plus l'être dans ce combat (spec 031,
  Q10). Valider une parade insuffisante ne fait donc que perdre les cartes jouées : l'écran ne la propose plus. Le moteur la
  garde comme action légale (un joueur automatique peut toujours la choisir).
- **Revenir en arrière** (MKUE p. 5, *Reverting* ; spec 024) : « Annuler » la parade reprend les décisions prises depuis
  « Parer », et « Annuler » l'attaque celles prises depuis « Attaquer », comme autant de Ctrl+Z. Aucune n'est un point de
  non-retour pendant une attaque ou une parade (cartes jouées, mana, unités activées : `Event::blocks_undo` faux), sauf après le
  chargement d'une partie (L3, C21).

## Modèle

Aucun terme nouveau au glossaire (Parer, Parade, Valider, Offre y sont déjà).

Rien de nouveau dans l'état ni dans la vue. Ce que l'écran calcule en plus :

- **Ennemi coché d'office** : quand une seule cible est possible (un seul ennemi ciblable par les attaques légales), l'écran la
  compte comme cochée sans que le joueur clique ; `CombatChoice` (ce que l'écran retient du combat) ne change pas.
- **Attaque suffisante** : le total de l'attaque en cours (`Combat::in_progress_total`, résistances et fortifications
  comprises) atteint la somme des armures des cibles, déjà affichée (« Attaque : 3 / 6 »).
- **Parade suffisante** : le total de la parade en cours (`Combat::in_progress_total`) atteint ce qu'il faut parer
  (`CombatEnemy::block_needed`, vif compris). Ces deux valeurs se lisent déjà dans la vue (`PlayerView::combat`).
- **Retour en arrière possible** : `combat_screen` reçoit l'état du retour en arrière (`UndoStatus`, déjà passé aux boutons du
  panneau), qui n'est pas dans la vue mais dans le `GameRunner`.
- **Commandes d'écran** `CancelBlock` et `CancelAttack` (comme `ToggleTarget`, ce ne sont pas des actions du jeu) : la session
  revient en arrière tant qu'une parade, ou une attaque, est en cours et que c'est possible.

## Actions et événements

Aucune action ni aucun événement nouveau.

## Comportements attendus

Tests dans `crates/arcanum-ui/tests/combat_et_offres.rs`, sauf mention.

Ennemi coché d'office :

- **C1** — Étant donné un combat contre un seul ennemi, en phase d'attaque, quand l'écran s'affiche, alors la carte de
  l'ennemi n'a pas de case « cible » et « Attaquer » est actif : il déclare l'attaque contre cet ennemi
  (`single_target_is_selected_without_checkbox`) — MKUE p. 9, *Ranged and Siege Attack Phase* 2.
- **C2** — Étant donné deux ennemis dont l'un est vaincu en phase d'attaque à distance, quand vient la phase d'attaque, alors
  le survivant, seule cible possible, est coché d'office comme en C1 (`last_target_left_is_selected`).
- **C3** — Étant donné deux ennemis ciblables, alors chacun a sa case « cible », décochée, et « Attaquer » est grisé tant
  qu'aucun n'est coché, comme aujourd'hui (`two_targets_keep_their_checkboxes`).
- **C4** — Étant donné un seul ennemi, fortifié, en phase d'attaque à distance, alors il n'y a ni case ni « Attaquer », comme
  aujourd'hui : aucune cible n'est possible (`fortified_single_enemy_has_no_attack_in_ranged_phase`) — MKUE p. 9,
  *Ranged and Siege Attack Phase* 3. Implémentation : fortifié une fois, un ennemi reste une cible à distance (spec 032, Q1) ;
  le test prend donc un défenseur **doublement** fortifié (spec 043, C39), dans `sites_fortifies.rs`.

Bouton « Parer » :

- **C5** — Étant donné la phase de parade, alors le bouton de la carte d'un ennemi qui attaque se lit « Parer », sans le nom de
  l'ennemi (`block_button_is_named_parer`).
- **C6** — Étant donné un Invocateur et sa créature invoquée, alors son bouton se lit aussi « Parer »
  (`summoned_block_button_is_named_parer`) — spec 033.

Parade insuffisante :

- **C7** — Étant donné une parade déclarée sans carte jouée, ou avec moins de points qu'il n'en faut, alors le bouton du cadre
  de la parade se lit « Annuler » et commande `CancelBlock` ; « Valider la parade » n'est pas proposé
  (`insufficient_block_offers_cancel`) — MKUE p. 9, *Block Phase* 6.
- **C8** — Étant donné une parade dont le total atteint ce qu'il faut parer, alors le bouton se lit « Valider la parade » et
  valide la parade (`sufficient_block_offers_resolve`) — MKUE p. 9, *Block Phase* 5.
- **C9** — Étant donné un ennemi vif (parade doublée) et une parade qui atteint son attaque simple mais pas le double, alors le
  bouton se lit « Annuler » (`swift_block_counts_doubled_value`) — spec 033.
- **C10** — Étant donné une parade contre une attaque de feu payée par des points de parade physiques (divisés par deux), alors
  c'est le total divisé qui compte (`inefficient_block_counts_halved_total`) — MKUE p. 9, *Block Phase* 5.b ; spec 032.
- **C11** — Étant donné une parade insuffisante avec deux cartes jouées et une unité activée, quand le joueur clique
  « Annuler », alors il n'y a plus de parade en cours, les deux cartes sont revenues en main, l'unité est de nouveau prête et
  l'ennemi peut toujours être paré (`cancel_block_reverts_to_before_declaration`, test de session) — MKUE p. 5, *Reverting*.
- **C12** — Étant donné une parade insuffisante, quand le joueur appuie sur Entrée, alors rien ne se passe ; quand il appuie sur
  Échap, alors la parade est annulée comme en C11 (`enter_does_not_resolve_insufficient_block`,
  `escape_cancels_insufficient_block`, tests de session).

Attaque insuffisante (Q2) :

- **C19** — Étant donné une attaque déclarée dont le total n'atteint pas la somme des armures des cibles (résistances comprises),
  alors le bouton du cadre de l'attaque se lit « Annuler » et commande `CancelAttack` ; quand le total l'atteint, il se lit
  « Valider l'attaque » (`insufficient_attack_offers_cancel`, `sufficient_attack_offers_resolve`) — MKUE p. 9, *Ranged and
  Siege Attack Phase* 5 et 6.
- **C20** — Étant donné une attaque insuffisante avec une carte jouée, quand le joueur clique « Annuler » (ou appuie sur Échap),
  alors il n'y a plus d'attaque en cours, la carte est revenue en main et l'ennemi peut toujours être attaqué dans la phase ;
  Entrée ne fait rien (`cancel_attack_reverts_to_before_declaration`, `escape_cancels_insufficient_attack`, tests de session)
  — MKUE p. 5, *Reverting*.
- **C21** — Retour en arrière impossible pendant une attaque (partie chargée) : le bouton reste « Valider l'attaque », comme L3
  pour la parade (`attack_without_undo_keeps_resolve`).

Carte d'ennemi :

- **C13** — Le nom de la carte est celui de l'ennemi seul (« Rôdeurs »), sans sa famille ; la famille (« orques
  maraudeurs ») est dans l'infobulle du jeton (`enemy_card_name_without_family`).
- **C14** — L'état ne dit plus « en jeu » : la ligne d'état n'existe que pour « paré », « vaincu » ou une restriction
  (fortifié, siège seulement…) (`enemy_card_status_omits_in_play`).
- **C15** — L'historique « Dans ce combat » est replié par défaut, son titre donne le nombre de lignes (« Dans ce combat (2) ») ;
  sans ligne, il n'apparaît pas (`enemy_history_title_counts_lines`).
- **C16** — L'enjeu de la parade tient en deux lignes : « À parer : 6 (vif, 2 × 3) » puis « Si non paré : 3 dégâts,
  2 blessure(s) » (`block_stakes_on_two_lines`) — spec 033, Q4.

Fenêtre Offres :

- **C17** — Les colonnes de la fenêtre Offres n'ont plus de ligne « Pioche : … » ni « Élite : … »
  (`offers_window_shows_no_deck_size`, dans `offres.rs`, à la place de l'assertion actuelle sur `deck_count`).
- **C18** — Le texte des cartes de la fenêtre Offres et de la fenêtre de choix d'un artefact est à la taille du reste du jeu
  (vérification à l'œil).

## Cas limites et refus

- **L1** — Un ennemi seul déjà ciblé et raté dans la phase n'est plus une cible (spec 031, Q10) : ni case ni « Attaquer », comme
  en C4 (`failed_single_target_has_no_attack`).
- **L2** — Un ennemi seul, le joueur clique « Attaquer » puis Ctrl+Z : l'ennemi redevient coché d'office (couvert par C1, pas de
  test à part).
- **L3** — Retour en arrière impossible pendant une parade (partie chargée en pleine parade : la pile de retour est vide), alors
  le bouton reste « Valider la parade » même insuffisante, pour que le joueur ne soit pas bloqué : le moteur ne propose rien
  d'autre tant que la parade est en cours (`block_without_undo_keeps_resolve`).
- **L4** — « Annuler » sur une parade déclarée sans rien jouer reprend seulement « Parer » (couvert par C7 et C11).

## Information cachée

Rien de nouveau : tout ce que l'écran affiche se lit déjà dans `PlayerView::combat` ; seul l'état du retour en arrière vient du
`GameRunner`, comme pour le bouton « Revenir en arrière » (spec 024).

## Libellés (i18n)

Valeurs relues dans `assets/i18n/fr.json` et `en.json` le 2026-11-06.

| Clé | fr | en | Note |
|---|---|---|---|
| `combat.block` | « Parer » | « Block » | était « Parer {enemy} » / « Block {enemy} » : le paramètre disparaît |
| `combat.cancel` | « Annuler » | « Cancel » | nouvelle, pour l'attaque et la parade ; à côté de `combat.resolve_block` et `combat.resolve_attack`, pas de clé fille (`combat` est un groupe) |
| `combat.history.count` | « Dans ce combat ({count}) » | « In this combat ({count}) » | nouvelle ; `{count}` = 2 donne « Dans ce combat (2) » / « In this combat (2) » |
| `combat.enemy.in_play` | — | — | supprimée (C14) |
| `combat.enemy.summoned_name` | — | — | supprimée : seul le bouton de parade l'utilisait (C6) |
| `combat.history.title` | — | — | supprimée, remplacée par `combat.history.count` |
| `offers.deck_count`, `offers.elite_deck_count` | — | — | supprimées (C17) |

`combat.enemy.stakes` (« À parer : {block} · Si non paré : {damage} dégâts, {wounds} blessure(s) ») est coupée en deux clés
sœurs, `combat.enemy.stakes_block` (« À parer : {block} » / « To block: {block} ») et `combat.enemy.stakes_damage` (« Si non
paré : {damage} dégâts, {wounds} blessure(s) » / « If unblocked: {damage} damage, {wounds} wound(s) »), valeurs d'origine
reprises mot pour mot ; `combat.enemy.stakes` est supprimée.

L'infobulle du jeton (C13) reprend telle quelle la valeur de `enemy.color.*` (`enemy.color.green` = « orques maraudeurs ») :
pas de clé nouvelle.

Aucun caractère nouveau : parenthèses et chiffres seulement.

## Interface

- **Ennemi coché d'office** : pas de case « cible » quand une seule cible est possible, « Attaquer » actif (C1, C2 ;
  `PlayerView::combat`, actions légales).
- **« Parer »** sur la carte de l'ennemi (C5, C6).
- **Cadre de la parade en cours** : « Annuler » tant qu'elle ne suffit pas, « Valider la parade » ensuite ; le total
  « Parade : 5 / 6 » reste affiché (C7 à C10, L3 ; `PlayerView::combat`, `UndoStatus`).
- **Cadre de l'attaque en cours** : de même, « Annuler » puis « Valider l'attaque » (C19 à C21 ; `PlayerView::combat`,
  `UndoStatus`).
- **Carte d'ennemi simplifiée** (C13 à C16) :
  - en tête : le jeton, à droite le nom seul, puis les valeurs (« armure 4, attaque 3, gloire 3 »), les capacités et les badges
    de résistance, inchangés (les icônes viendront avec la spec 071) ;
  - au survol du jeton : la famille de l'ennemi (C13) ;
  - la ligne d'état seulement si elle dit quelque chose (C14) ;
  - la créature invoquée, inchangée ;
  - l'enjeu de la parade sur deux lignes (C16) ;
  - la case « cible » ou « Parer » ;
  - « Dans ce combat (n) », replié (C15).
  Mise en page et tailles : vérification à l'œil.
- **Fenêtre Offres** : sans taille de pioche (C17) ; nom et texte des cartes à la taille normale, colonnes élargies pour que la
  fenêtre tienne en 1600 × 900 sans défiler (C18, vérification à l'œil). Même taille de texte dans la fenêtre de choix d'un
  artefact, qui a la même présentation (C18).

### Raccourcis clavier

- **Entrée** (spec 031) : valide l'attaque ou la parade en cours seulement si son bouton dit « Valider » ; sans effet sur une
  attaque ou une parade insuffisante (C12, C20).
- **Échap** (nouveau) : annule l'attaque ou la parade en cours quand le bouton dit « Annuler » (C12, C20). Contexte libre : en combat, hors mode
  choix, Échap n'avait pas d'effet.

Les deux lignes sont reportées dans `docs/raccourcis.md`.

### Vérification à l'œil

Parties préparées `combat-deux-ennemis` (Rôdeurs seuls défiés), `niveau-pair` (fenêtre Offres, touche O) et `sombre-terrier`
(choix d'un artefact), captures `spec-070-*` des fichiers du projet : ennemi seul sans case et « Attaquer » actif, famille au
survol du jeton, « Parer », enjeu sur deux lignes, parade et attaque insuffisantes avec « Annuler » (Entrée sans effet, Échap
annule), « Valider l'attaque » une fois le total atteint, historique replié « Dans ce combat (1) », fenêtre Offres sans pioche
qui tient en 1600 × 900, fenêtre des artefacts au même texte.

## Effets sur les specs déjà implémentées

Tests changés, et pourquoi (la spec change le comportement qu'ils vérifient) :

- **031** : `deroule_combat.rs`, l'assertion « Parer Rôdeurs (orques maraudeurs) » devient « Parer » ; le nom de la carte
  d'ennemi perd sa famille ; les tests qui cochent l'ennemi seul avant « Attaquer » ne le cochent plus (sinon le clic le
  décocherait : voir C1). Les tests qui valident une attaque ou une parade insuffisante pour la rater passent par l'action légale du moteur,
  pas par le bouton.
- **033** : `deroule_combat.rs`, « Parer Minotaure (invoqué) » devient « Parer » ; l'enjeu se lit sur deux lignes.
- **050** : `offres.rs`, l'assertion sur `deck_count` est remplacée par C17.

## Hors périmètre

- Icônes à la place des mots (Armure, Attaque, élément, résistances) : spec 071.
- Illustrations des ennemis : spec 072.
- Main et unités de l'écran de combat dans le panneau du bas : spec 073.

## Questions ouvertes

Aucune.

## Décisions de Mathieu (2026-11-06)

- **Q1 — Carte d'ennemi simplifiée** : acceptée telle que proposée (nom sans la famille, famille en infobulle du jeton, plus de
  « en jeu », enjeu de la parade sur deux lignes, historique replié avec son nombre de lignes ; valeurs, capacités et
  résistances inchangées jusqu'à la spec 071).
- **Q2 — Attaque insuffisante** : même traitement que la parade, « Annuler » au lieu de « Valider l'attaque » (C19 à C21).
- **Q3 — Ennemi seul** : « une seule cible possible » (le survivant compte), et la case disparaît au lieu d'être cochée.
