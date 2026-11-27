# 040 — Interaction et réputation

- Statut : implémentée (2026-11-03) ; validée (2026-10-31) ; propositions de Q1 à Q3 retenues par Mathieu
- Jalon : J4
- Dépend de : 013 (jouer des cartes, de côté), 014 (déplacement), 022 (effets renforcés : Menace, Nobles manières), 024 (retour en
  arrière), 029 (contenu en données), 030 et 031 (provocation, défi, combat, ennemis vaincus), 033 (pile de secours), 034 (repos,
  soins), 035 (gloire et niveaux), 065 (Norowas jouable) ; toutes implémentées
- Livret : MKUE p. 6 (6.b à 6.e, 7 : une seule action, points perdus au début de l'action, pas d'interaction au repos) ; p. 8
  (*Interacting with Locals* 1, 2, 3 et 5) ; p. 10 (*Combat Outcomes* 2.a) ; piste de réputation : scan
  `docs/regles/scans/misc/fame_board.jpg` ; FAQ p1 (*Nobles Manières*), p2 (carte inclinée), p3 (réputation des ennemis errants) ;
  Compendium, *Influence* S1, S2, S5, Q6, S7 ; *Combat* Q16 ; *Basic Action Cards* : *Noble Manners* S1 à S3, *Threaten* S1.
  Texte des cartes : `docs/regles/cartes/actions-de-base.md` (Menace, Nobles manières)
- Crates touchées : arcanum-core (règles, contenu), arcanum-ui, arcanum-persistance (une partie préparée), `assets/i18n/`

## Objectif

Le héros a une **réputation**, sur la piste de réputation, qui part de 0. Battre des ennemis errants la fait monter, Menace
renforcée la fait baisser. Sur un village ou un monastère, il peut **interagir avec les habitants** : c'est l'action du tour. Le
bonus ou malus de sa réputation s'ajoute alors une fois à son influence, et il peut jouer ses cartes d'influence. Nobles manières
jouée pendant l'interaction rapporte sa gloire (et sa réputation, renforcée).

Il n'y a **encore rien à acheter** : les soins et les actions avancées des monastères arrivent avec la spec 041, le recrutement des
unités avec la 042. Cette spec pose le cadre que ces deux specs remplissent (Q1).

## Règles

### Piste de réputation

- **Piste** (MKUE p. 3, *Fame and Reputation board* ; scan `misc/fame_board.jpg`) : quinze cases, de −7 à +7, le jeton part de la case
  centrale (0, la flèche). Chaque case porte le **bonus ou malus** d'influence à l'interaction, lu sur le scan :

  | Réputation | −7 | −6 | −5 | −4 | −3 | −2 | −1 | 0 | +1 | +2 | +3 | +4 | +5 | +6 | +7 |
  |---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
  | Bonus | **X** | −5 | −3 | −2 | −1 | −1 | 0 | 0 | 0 | +1 | +1 | +2 | +2 | +3 | +5 |

  Les exemples du livret le confirment : « start with −2 Reputation (i.e. −1 penalty) » (p. 17), « +2 Reputation (i.e. a bonus of
  +1) » (p. 19). La table est en données, comme la piste de gloire (spec 035). Réputation de départ d'un scénario autre que 0
  (Conquête en solitaire, p. 19) : spec 064.
- **Bornes** : la réputation ne sort pas de la piste. Au-delà de +7 elle reste à +7 ; en deçà de −7, à −7 (**X**) : on peut encore
  perdre de la réputation sur X, sans effet (Compendium, *Influence* S5 ; *Combat* Q16).
- **Case X** (MKUE p. 8, 2.a, dernier point) : pas d'interaction du tout. Seul moyen d'en sortir pour l'instant : battre des ennemis
  errants (Compendium, *Influence* Q6).
- **Moment du changement** : tout changement de réputation (et la gloire de Nobles manières) s'applique **aussitôt**, pas à la fin
  du tour. Les scans de Menace et de Nobles manières (édition d'origine, n/16) disent « at the end of your turn » ; l'Ultimate a
  retiré ces mots parce que *Lost Legion* a rendu ces changements immédiats (Compendium, *Noble Manners* S3, *Threaten* S1,
  *Combat* Q16). La version *Lost Legion* fait foi (`CLAUDE.md`). Comme le bonus est fixé au début de l'interaction (2.a), cela ne
  change rien pendant l'interaction ; cela compte pour une Menace jouée **avant** d'interagir (C12).

### Ce qui change la réputation dans cette spec

- **Ennemi errant vaincu** (MKUE p. 10, *Combat Outcomes* 2.a ; FAQ p3) : +1 par orques maraudeurs, +2 par draconien, au moment où
  il est vaincu, avec sa gloire (Compendium, *Combat* Q16). Seuls les ennemis errants comptent : la créature invoquée (spec 033)
  ne rapporte rien (FAQ p3, *Orques non-Maraudeurs*). Le type est celui **imprimé sur la case** (`Rampaging` de la tuile), pas la
  couleur du jeton : une case d'orques remplie par la pile de secours (spec 033) rapporte +1 (Q2).
- **Menace renforcée** (`cartes/actions-de-base.md`) : Influence 5 et Réputation −1, qu'elle soit jouée pendant une interaction ou
  non (spec 022, Q2). Jouée de base : Influence 2, inchangée.
- **Nobles manières** (Norowas) : de base, Influence 2 et, **si elle est jouée pendant une interaction**, Gloire +1 ; renforcée,
  Influence 4 et, pendant une interaction, Gloire +1 et Réputation +1. Le bonus ne demande pas de dépenser l'influence (FAQ p1 ;
  Compendium, *Noble Manners* S1). Jouée avant d'interagir, ou hors d'un village ou d'un monastère : pas de bonus (*Noble Manners*
  S2, S3). Jouée de côté, ce n'est qu'« Influence 1 » (FAQ p2) : pas de bonus.
- Ailleurs, plus tard : assaut d'un site fortifié (−1) et monastère brûlé (−3) : specs 043 et 041 ; pillage d'un village : hors
  périmètre (en solo, il n'y a pas de tour d'un autre joueur, carte de site du village) ; actions avancées qui touchent la réputation
  (Contrainte impitoyable, Récit héroïque…) : spec 053 ; héros et voyous des extensions : hors périmètre.

### Interagir avec les habitants

- **Où** (MKUE p. 8, 1 ; p. 6, 6.c) : la figurine est sur un **village** ou un **monastère**. Forteresse possédée, tour de mage et
  cité conquises : specs 043 et 062. Monastère brûlé : spec 041. Ailleurs, pas d'interaction (*Noble Manners* S2).
- **C'est l'action du tour** (MKUE p. 6, 6.c et 6.d ; Compendium, *Influence* S7) : une seule par tour, et ni combat avant ni combat
  après. Interagir est impossible si un combat a été engagé, si un ennemi a été provoqué ou défié ce tour (action obligatoire,
  6.b), ou pendant le choix d'une action avancée (spec 050). Après l'interaction : plus de déplacement, d'exploration, de défi, ni
  d'annonce de fin de manche (l'annonce remplace le tour, p. 6, 3).
- **Pas au repos** (MKUE p. 6, 7) : après une interaction, la fin du tour ne propose plus de repos, comme après un déplacement
  (spec 034).
- **Points perdus** (MKUE p. 6, 6.e ; Compendium, *Influence* S1) : au moment où l'interaction commence, les points de mouvement
  non dépensés sont perdus, comme au défi et à la provocation (spec 030). L'interface prévient (Interface).
- **Variante maison : l'influence reste** (Mathieu, 2026-11-27, issue #95) : contre MKUE p. 6, 6.e, l'influence jouée avant
  l'interaction est gardée, pour que l'ordre des clics ne coûte rien au joueur ; le bonus de réputation s'y ajoute. Seule
  l'interaction la garde : défi, incendie, aventure et assaut la perdent toujours (sinon Diplomatie bloquerait avec de l'influence
  du déplacement). Effet sur le jeu jugé négligeable : le reste de Learning après son achat servirait à l'interaction ; les bonus
  « pendant l'interaction » (Nobles manières) restent réservés aux cartes jouées après.
- **Bonus de réputation** (MKUE p. 8, 2.a et 5.a ; Compendium, *Influence* S2) : aussitôt après, le bonus ou malus de la case de
  réputation devient l'influence de départ de l'interaction. Il est appliqué **une fois** : un changement de réputation ensuite
  ne le change plus ce tour. Un malus donne une **influence négative** (Compendium, *Influence* S1, exemple du monastère) : les
  cartes jouées ensuite la comblent d'abord.
- **Pendant l'interaction** (MKUE p. 8, 2.b, 2.d et 2.e) : le joueur peut jouer des effets d'influence (renforcés au besoin), des
  effets de mana et de soin, et toute carte sauf une Blessure de côté pour Influence 1. Les effets de mouvement ne sont plus
  proposés (ils ne servent plus) : une carte à options n'offre que ses options utiles, Mouvement 1 de côté non plus. Total de
  l'influence (3) : la somme, affichée. Les bonus d'une cité (3.a) : spec 062.
- **Dépenser** (MKUE p. 8, 4 et 5) : rien à acheter dans cette spec (Q1). L'influence reste jusqu'à la fin du tour, où elle est
  perdue (spec 016).

### Schéma : un tour d'interaction

```
Début du tour, figurine sur le village (2, −3), réputation +2 (bonus +1)
  Marche (Mouvement 2)                  ┐ phase de déplacement : bouger, explorer
  …                                     ┘ (Promesse jouée ici : son Influence 2 reste, issue #95)
  Interact                              → InteractionStarted { site: Village, bonus: +1 }
                                          mouvement 0, influence +1 (bonus), plus de déplacement
  Nobles manières renforcée (dé blanc)  → CardPlayed, influence 5 ; FameGained 1 ; ReputationChanged +1
                                          (réputation +3, le bonus reste +1 ce tour)
  Menace (de base)                      → influence 7
  EndTurn (sans repos)                  → influence perdue, pioche
```

## Modèle

Termes du glossaire : Réputation, Interaction, Influence, Points d'influence, Village, Monastère, Ennemi errant, Menace, Nobles
manières. **Nouveaux termes** (ajoutés au glossaire avec cette spec) :

| Livret | Français | Code |
|---|---|---|
| Reputation track | Piste de réputation | `ReputationTrack` |
| Reputation bonus or penalty (modifier) | Bonus de réputation (négatif : malus) | `ReputationTrack::bonus` |
| X space (Reputation track) | Case X | `Reputation::is_x` |
| Interact with the Locals | Interagir avec les habitants | `Action::Interact` |
| Locals | Habitants | — |

Données nouvelles :

- **État du joueur** : sa **réputation** (−7 à +7, 0 à la mise en place) ; son **interaction en cours** ce tour (le site et le
  bonus appliqué), vidée par la fin du tour. Ses **points d'influence** deviennent un nombre signé (influence négative).
- **Contenu** : la piste de réputation (bonus de chaque case, X), lue dans un fichier de `content/` et validée (quinze cases, une X
  au bas), comme la piste de gloire.
- **Effets de carte** : deux formes nouvelles d'`Effect`, que les fichiers de contenu écrivent :
  - Influence et réputation (Menace renforcée : Influence 5, Réputation −1) ;
  - Influence avec un bonus en interaction (Nobles manières : Influence 2 et Gloire +1 ; renforcée, Influence 4, Gloire +1 et
    Réputation +1).
  Concentration renforcée (spec 025) leur ajoute son bonus d'influence comme à une Influence simple ; le reste de l'effet ne
  change pas.
- **Vue du joueur** : réputation, bonus de la case, interaction en cours, influence signée ; tout est public.

## Actions et événements

| Action | Validation (refusée si…) | Événements produits |
|---|---|---|
| `Interact` (nouvelle) | figurine ni sur un village ni sur un monastère ; réputation sur X ; interaction déjà faite ce tour ; combat engagé, ennemi provoqué ou défié ce tour ; choix d'une action avancée en attente ; partie finie | `InteractionStarted { player, site, bonus }` |
| `PlayCard` (013, étendue) | en interaction, effet de mouvement ou Mouvement 1 de côté | `CardPlayed`, puis pour Menace renforcée `ReputationChanged { change: −1 }` ; pour Nobles manières en interaction `FameGained { fame: 1 }` et, renforcée, `ReputationChanged { change: +1 }` |
| `Attack` (031, étendue) | inchangée | … `EnemiesDefeated`, puis `ReputationChanged` si des ennemis errants sont vaincus (une fois, le total : +1 par case d'orques, +2 par case de draconien) |
| `EndTurn` (016, étendue) | en interaction, repos refusé | inchangés |

Ordre des actions légales (recette de `architecture.md`) : `Interact` vient **après** les explorations et **avant** les défis et
les fins de tour. Pendant l'interaction : les cartes, puis les fins de tour.

| Événement | Effet de `apply` | Révèle une information ? | Point de non-retour ? |
|---|---|---|---|
| `InteractionStarted { player, site, bonus }` (nouveau) | mouvement et influence à 0, puis influence = `bonus` ; interaction en cours ; plus de déplacement ni de repos ce tour | non (publique) | non |
| `ReputationChanged { player, change }` (nouveau) | réputation + `change`, bornée à −7 et +7 | non (publique) | non |
| `FameGained` (063, réutilisé) | inchangé : gloire + `fame` | non | non |
| `TurnEnded` (016) | vide aussi l'interaction en cours | inchangé | inchangé |

Aucun événement existant ne change de forme. Les parties préparées sont régénérées (nouveaux événements dans leur construction).

## Comportements attendus

Piste de réputation :

- **C1** — Étant donné une nouvelle partie (bac à sable ou Première reconnaissance), alors la réputation est 0 et son bonus 0
  (`reputation_starts_at_zero`) — MKUE p. 3.
- **C2** — La table du contenu donne, de −7 à +7 : X, −5, −3, −2, −1, −1, 0, 0, 0, +1, +1, +2, +2, +3, +5
  (`reputation_track_matches_the_board`) — scan `fame_board.jpg`.
- **C3** — Étant donné une réputation +7, quand un changement de +2 arrive, alors elle reste à +7 ; à −7 (X), un changement de −1
  la laisse à −7 (`reputation_stays_on_the_track`) — Compendium, *Influence* S5.

Ennemis errants :

- **C4** — Étant donné des orques maraudeurs défiés, quand l'attaque les vainc, alors `EnemiesDefeated` est suivi de
  `ReputationChanged { change: +1 }` (`defeating_orc_marauders_gains_one_reputation`) — MKUE p. 10, 2.a.
- **C5** — Un draconien vaincu : +2 ; un orque et un draconien vaincus par la même attaque : un seul `ReputationChanged` de +3
  (`defeating_draconum_gains_two_reputation`) — MKUE p. 10, 2.a.
- **C6** — Une attaque ratée, ou un combat fini sans victoire : aucun `ReputationChanged` (`no_reputation_without_defeat`).
- **C7** — Une case d'orques maraudeurs remplie par la pile de secours (jeton d'une autre couleur, spec 033) : vaincu, il rapporte
  +1 (`reputation_follows_the_printed_space`) — Q2.

Interagir :

- **C8** — Étant donné la figurine sur le village du bac à sable (2, −3), réputation 0, alors `Interact` est légale ; sur la plaine
  (2, −2), non (`interact_only_on_a_village_or_monastery`) — MKUE p. 8, 1.
- **C9** — Étant donné 3 points de mouvement et 2 d'influence non dépensés, réputation +2, quand le joueur interagit, alors
  `InteractionStarted { bonus: +1 }` ; mouvement 0, influence 3 (`interacting_keeps_unspent_influence_and_adds_the_bonus`) —
  MKUE p. 6, 6.e ; p. 8, 2.a ; variante maison (issue #95).
- **C10** — Réputation −4 (malus −2), interaction sans carte jouée : influence −2 ; puis Promesse : influence 0
  (`a_penalty_gives_negative_influence`) — Compendium, *Influence* S1.
- **C11** — Pendant l'interaction, une carte qui fait changer la réputation (Nobles manières renforcée) ne change pas l'influence
  déjà comptée : le bonus reste celui du début (`the_bonus_is_applied_once`) — MKUE p. 8, 2.a et 5.a.
- **C12** — Réputation −1 (bonus 0), Menace renforcée jouée **avant** d'interagir : réputation −2 aussitôt, puis l'interaction donne
  le malus −1, qui s'ajoute aux 5 d'influence de Menace : 4 (`threaten_before_interacting_lowers_the_bonus`) — Compendium,
  *Threaten* S1 ; variante maison (issue #95).
- **C13** — Après `Interact` : plus de `MoveHero`, `FollowRoute`, `Explore`, `Challenge`, ni `AnnounceEndOfRound` ; plus de
  deuxième `Interact` ; les fins de tour n'ont pas de repos (`interacting_is_the_action_of_the_turn`) — MKUE p. 6, 6.d et 7.
- **C14** — Pendant l'interaction, Marche ne se joue que de côté pour Influence 1 ; Improvisation n'offre que Influence 3 ; un soin
  (Tranquillité) et un gain de mana (Concentration) se jouent (`interaction_offers_influence_mana_and_healing`) — MKUE p. 8, 2.b
  à 2.e.
- **C15** — Un ennemi errant voisin du village : `Interact` et `Challenge` sont légales au début du tour ; après le défi,
  `Interact` ne l'est plus (`challenging_forbids_interacting`) — Compendium, *Influence* S7.
- **C16** — Un pas qui provoque un ennemi errant en arrivant sur le village : `Interact` n'est pas légale (action obligatoire)
  (`a_provocation_forbids_interacting`) — MKUE p. 6, 6.b.
- **C17** — Réputation sur X : `Interact` n'est pas légale (`no_interaction_on_x`) — MKUE p. 8, 2.a.
- **C18** — Revenir en arrière juste après `Interact` rend les points de mouvement et d'influence d'avant
  (`interacting_can_be_undone`) — spec 024.
- **C18 bis** (issue privée 307) — Tant que rien n'a été joué ni acheté depuis `Interact`, `GameRunner::cancel_untouched_interaction`
  reprend l'interaction comme un retour en arrière : le héros peut de nouveau se déplacer ; après une carte jouée, ou sans
  interaction, elle ne reprend rien (`untouched_interaction_can_be_cancelled`,
  `interaction_with_a_card_played_is_not_cancelled`, `no_interaction_nothing_cancelled`) — MKUE p. 7, 1 et p. 5, Reverting.
- **C19** — Fin du tour : l'influence est perdue, l'interaction est vidée, le tour suivant peut à nouveau interagir
  (`interaction_ends_with_the_turn`) — MKUE p. 6, 6.d.

Cartes :

- **C20** — Menace renforcée hors interaction : Influence 5 et `ReputationChanged { change: −1 }` ; de base : Influence 2 seulement
  (`threaten_strong_loses_reputation`) — `cartes/actions-de-base.md`.
- **C21** — Norowas en interaction : Nobles manières de base donne Influence 2 et `FameGained { fame: 1 }` ; renforcée, Influence 4,
  `FameGained { fame: 1 }` et `ReputationChanged { change: +1 }` (`noble_manners_rewards_interaction`) — FAQ p1.
- **C22** — Nobles manières jouée hors interaction (avant `Interact`, ou loin d'un village) : Influence seulement ; jouée de côté
  pendant l'interaction : Influence 1 seulement (`noble_manners_needs_interaction`) — *Noble Manners* S2, S3 ; FAQ p2.
- **C23** — La gloire de Nobles manières compte pour le changement de niveau de la fin du tour (gloire 2, Nobles manières en
  interaction : niveau 2 à la fin du tour) (`noble_manners_fame_counts_for_level`) — spec 035.

Sauvegarde et vue :

- **C24** — Une partie sauvegardée en pleine interaction, réputation +3, se recharge avec la même réputation, l'interaction en cours
  et la même influence (`interaction_survives_save_and_load`) — spec 024.
- **C25** — La vue du joueur donne la réputation, son bonus et l'interaction en cours (`view_shows_reputation`).

Écran (arcanum-ui, sur `game_screen`) :

- **C26** — Le panneau montre « Réputation : +2 · bonus d'interaction +1 », et sur X « Réputation : X · interaction impossible »
  (`panel_shows_reputation`).
- **C27** — Sur un village, le bouton « Interagir avec les habitants » est là ; s'il reste des points de mouvement ou
  d'influence, la ligne d'avertissement les nomme (`panel_offers_interaction_and_warns`).
- **C28** — En interaction : l'en-tête « Interaction : Village » et la ligne « Rien à acheter ici pour l'instant »
  (`panel_shows_interaction_in_progress`).
- **C29** — Une influence négative s'affiche avec son signe : « Influence : −2 » (`negative_influence_is_shown`).
- **C30** — Les textes de Menace renforcée et de Nobles manières sont calculés (plus « Pas encore disponible ») ; la victoire
  sur des orques ajoute « Réputation +1 » à l'historique du combat (`reputation_texts`).

## Cas limites et refus

- **L1** — `Interact` hors des actions légales (plaine, X, deuxième fois, après un défi) : refusée par `decide`
  (`ActionError`), état inchangé.
- **L2** — Interagir sans jouer de carte puis finir le tour : permis ; la fin du tour demande toujours de jouer ou défausser une
  carte (spec 016, p. 6, 5.a) et ne propose pas de repos.
- **L3** — Deck vide au début du tour : `AnnounceEndOfRound` et `Interact` sont toutes deux légales ; après `Interact`, plus
  d'annonce (C13).
- **L4** — Dernier tour de la partie (spec 063) : l'interaction est permise, la gloire de Nobles manières compte au décompte.
- **L5** — Nobles manières renforcée sur une réputation +7 : Gloire +1, la réputation reste à +7 (C3).
- **L6** — Concentration renforcée sur Nobles manières en interaction : Influence 4 + 2, la gloire et la réputation ne
  doublent pas.
- **L7** — Une réputation qui sort de X (orques vaincus) rend l'interaction possible dès le tour suivant (un combat a eu lieu ce
  tour).

## Information cachée

Rien de caché : réputation, bonus, interaction en cours et influence sont publics (piste commune, MKUE p. 3). Les événements
nouveaux ne sont pas filtrés.

## Libellés (i18n)

| Clé | Paramètres | fr | en |
|---|---|---|---|
| `reputation.line` | `reputation`, `bonus` | Réputation : {reputation} · bonus d'interaction {bonus} | Reputation: {reputation} · interaction bonus {bonus} |
| `reputation.line_x` | — | Réputation : X · interaction impossible | Reputation: X · no interaction |
| `interaction.button` | — | Interagir avec les habitants | Interact with the locals |
| `interaction.lost_points` | `move` | Interagir fait perdre : {move} mouvement | Interacting loses: {move} move |
| `interaction.header` | `site` | Interaction : {site} | Interaction: {site} |
| `interaction.nothing_to_buy` | — | Rien à acheter ici pour l'instant | Nothing to buy here yet |
| `effect.influence_reputation` | `amount`, `reputation` | Influence {amount} ; Réputation {reputation} | Influence {amount}; Reputation {reputation} |
| `effect.influence_in_interaction` | `amount`, `bonus` | Influence {amount} ; en interaction : {bonus} | Influence {amount}; during interaction: {bonus} |
| `effect.fame` | `amount` | Gloire +{amount} | Fame +{amount} |
| `effect.reputation` | `amount` | Réputation {amount} | Reputation {amount} |
| `combat.history.reputation` | `amount` | Réputation {amount} | Reputation {amount} |

Les nombres signés s'écrivent avec leur signe (+1, −2, 0). `{bonus}` de `effect.influence_in_interaction` assemble `effect.fame` et
`effect.reputation` (« Gloire +1, Réputation +1 »). Noms des sites : clés `site.*` existantes.

## Interface

- **Panneau latéral** : `reputation.line` (ou `reputation.line_x`) sous la ligne de gloire et de niveau (spec 035). Pas de dessin
  de la piste : du texte suffit, visuels provisoires.
- **Bouton « Interagir avec les habitants »** sous les points, quand `Interact` est légale ; dessous, `interaction.lost_points`
  quand il reste des points de mouvement (ils seraient perdus, C9 ; l'influence reste, issue #95).
- **En interaction** : `interaction.header` en tête des points, puis `interaction.nothing_to_buy` ; la ligne `points.line`
  montre l'influence signée. Les specs 041 et 042 remplacent la ligne « Rien à acheter » par leurs achats.
- **Main** : les textes calculés des nouvelles formes d'effet (`effect.influence_reputation`, `effect.influence_in_interaction`).
- **Combat** : `combat.history.reputation` dans l'historique, après la ligne de l'attaque qui a vaincu des ennemis errants.

```
┌─ Panneau latéral ─────────────────────────────────┐
│ Gloire : 4 · Niveau 2 · niveau 3 à 8              │
│ Armure 2 · Limite de main 5 · Jetons de           │
│ commandement 1                                    │
│ Réputation : +2 · bonus d'interaction +1          │
│ Interaction : Village                             │
│ Mouvement : 0 · Influence : 5                     │
│ Rien à acheter ici pour l'instant                 │
│ Main                                              │
│ ● Menace      Influence 2                         │
└───────────────────────────────────────────────────┘
```

Vérification à l'œil : une partie préparée `interaction` (bac à sable, **Norowas**, jour, début du tour ; figurine sur le village
(2, −3), voisine des Rôdeurs (3, −3), orques maraudeurs ; main : Nobles manières, Menace, Promesse, Marche, Rage ; un cristal blanc
et un rouge) : interagir, jouer Nobles manières renforcée, voir la gloire et la réputation ; ou, depuis le chargement, défier et
vaincre les Rôdeurs pour voir « Réputation +1 ».

### Raccourcis clavier

| Touche | Effet | Contexte |
|---|---|---|
| I | Interagir avec les habitants | Partie, quand le bouton est actif, hors mode choix |

Reportée dans `docs/raccourcis.md` (touche libre).

## Effets sur les specs déjà implémentées

- **013 et 022** : Menace renforcée n'est plus `Unsupported` ; Nobles manières gagne son bonus (tableaux des cartes de ces specs :
  « spec 040 » devient fait). Les tests de la 022 qui jouent Menace renforcée de côté sont à revoir et signalés dans la PR.
- **014, 030, 031** : la perte des points au début de l'action vaut aussi pour l'interaction ; rien ne change pour la provocation
  et le défi.
- **034** : `can_rest` devient faux aussi après une interaction.
- **039 et autres parcours** : les orques vaincus donnent de la réputation ; leurs points d'étape ne la vérifient pas, ils restent
  valables. Le joueur de test `FirstChoice` ne choisit pas `Interact` (comme il ne défie pas), pour que ses parties ne changent pas.
- **024** : `Event::blocks_undo` gagne `InteractionStarted` et `ReputationChanged` (non).

## Hors périmètre

- Acheter des soins et des actions avancées au village et au monastère, brûler un monastère : spec 041. Recruter des unités, leurs
  effets d'influence : spec 042.
- Interagir dans une forteresse possédée, une tour de mage conquise (sorts) : spec 043 ; dans une cité (bonus des jetons Bouclier,
  achats par couleur) : spec 062.
- Perte de réputation des assauts : spec 043.
- Jouer de l'influence en combat (permis depuis *Lost Legion*, Compendium, *Influence* S1, pour Diplomatie par exemple) : avec la
  première carte qui en a besoin (spec 053) ; jusque-là l'influence ne se joue pas en combat (spec 031).
- Réputation de départ d'un scénario, Conquête en solitaire : spec 064. Compétences qui touchent la réputation ou l'interaction
  (Krang sur X) : spec 051. Pillage d'un village, ennemis héros et voyous : hors périmètre.
- Dessin de la piste de réputation, survol d'un village qui annonce l'interaction : plus tard, si le besoin se fait sentir.

## Plan de développement

Plan d'étapes : [`plans/040-interaction-et-reputation.md`](plans/040-interaction-et-reputation.md).

## Vérification manuelle (2026-11-03)

Depuis la partie préparée `interaction` (`scripts/verif-manuelle.sh charger interaction`), captures dans les fichiers du projet
(`captures/`) : `spec-040-debut.png` (ligne de réputation, bouton « Interagir avec les habitants »), `spec-040-interaction.png`
(en-tête, influence au bonus, « Rien à acheter », plus de bouton de mouvement), `spec-040-mana.png` et
`spec-040-nobles-manieres.png` (Nobles manières renforcée : gloire et réputation), `spec-040-combat-reputation.png` (Rôdeurs
vaincus, « Réputation +1 » dans l'historique du combat). Elle a fait voir un bouton « Renforcer : Mouvement 4 » grisé pendant
l'interaction, corrigé avec son test (`no_move_buttons_in_interaction`).

## Décisions (2026-10-31)

Questions de la rédaction, tranchées par Mathieu (« ok avec tes propositions ») :

1. **Q1** — Interaction sans achat dans cette spec : les soins et les actions avancées du monastère restent à la 041, les unités à
   la 042, comme le prévoit `PLAN.md`.
2. **Q2** — Une case d'ennemi errant remplie par la pile de secours rapporte la réputation du type imprimé sur la case (orques +1,
   draconien +2), pas celle de la couleur du jeton (C7).
3. **Q3** — Réputation et gloire changent aussitôt (version *Lost Legion*), pas à la fin du tour (C12).
4. Relecture de la PR privée 108 : l'en-tête de l'interaction ne répète plus le bonus (« Interaction : Village »). La ligne de
   réputation le donne déjà, et l'influence affichée l'inclut.

## Questions ouvertes

Aucune.
