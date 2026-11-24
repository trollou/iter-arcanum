# 021 — Cristaux

- Statut : implémentée (2026-10-25)
- Jalon : J2
- Dépend de : 010 (sites des tuiles), 016 (fin du tour), 020 (Source, panneau latéral)
- Livret : MKUE p. 3 (« Hero card » : l'inventaire ; « Bank »), p. 5 (« Using mana » 1.b ; « "Gain" effects » 2), p. 11
  (« End of the Turn » 4.b : mine de cristal), p. 15 (les héros commencent sans cristal)
- Crates touchées : arcanum-core, arcanum-ui, `assets/i18n/` (fr.json et en.json)

## Objectif

Le héros a un **inventaire** de cristaux, au plus trois par couleur de base. Il commence vide. Le joueur en gagne un en **finissant son
tour sur une mine** : un cristal de la couleur de la mine, sauf s'il en a déjà trois. Le panneau latéral montre l'inventaire, et le
survol d'une mine dit ce qu'elle rapporterait. Les cristaux ne servent encore à rien : on les utilisera pour payer un effet renforcé à la
spec 022 (Q1).

## Règles

- **Deux formes de mana** (MKUE p. 5, « Using mana » 1) : le **mana fondamental** (un dé de la Source, spec 020, ou un **jeton de mana**
  dans la zone de jeu, perdu en fin de tour s'il n'est pas utilisé) et le **cristal** (rangé dans l'**inventaire**, gardé d'un tour à
  l'autre).
- **Limite de trois** (MKUE p. 5, 1.b et 2) : trois cristaux par couleur de base ; pas de cristal or ni noir, l'inventaire n'a que quatre
  couleurs.
- **Départ** (MKUE p. 15) : les héros commencent sans cristal ; seul le joueur fantôme en reçoit trois (spec 061).
- **Réserve** (« Bank », MKUE p. 3 ; p. 11, 3.a) : on y prend les cristaux gagnés, on y rend les jetons de la zone de jeu. Dans le jeu,
  elle est **illimitée** : la boîte a assez de jetons, on ne compte pas ce qu'il en reste (Q2).
- **Mine de cristal** (MKUE p. 11, 4.b) : en **finissant son tour** sur une mine, on gagne un cristal de sa couleur, sauf si on en a déjà
  trois : **rien** alors. Le bénéfice est automatique, pas un choix. Passer sur une mine ne rapporte rien ; y rester plusieurs tours
  rapporte un cristal à chaque fin de tour.
- **Ordre dans la fin du tour** (MKUE p. 11) : le bénéfice de la case (4) vient après le nettoyage de la zone de jeu (3) et avant la
  défausse et la pioche (7).
- **Gagner un cristal en général** (spec 022 ; MKUE p. 5, « "Gain" effects » 2 ; p. 11, 4.b et 5.a) : par un **effet de carte** avec déjà
  trois cristaux, on gagne un **jeton de mana** à la place ; la mine et les récompenses de combat ne donnent rien. Aucun effet de carte
  ne fait encore gagner de cristal sans payer de mana (Cristallisation, Euphorie cristalline) : cette règle arrive avec les effets
  renforcés.
- **Utiliser un cristal** (spec 022, Q1 ; MKUE p. 5, 1.b) : tant que rien ne consomme de mana, transformer un cristal ne ferait que le
  perdre en fin de tour ; cette action arrive avec le premier usage du mana, la spec 022, comme la prise d'un dé (spec 020, Q1).

### Exemple : trois tours sur la mine verte de la tuile de campagne 2

Tovak, inventaire vide, arrive sur la mine verte (colline) au tour 3 et y reste.

| Tour | Pendant le tour | Fin du tour | Inventaire après |
|---|---|---|---|
| 3 | Marche et Rage de côté : 3 points de mouvement, il entre sur la colline de la mine. | Il finit sur la mine : +1 vert. | vert 1 |
| 4 | Joue Concentration de côté, ne bouge pas. | Toujours sur la mine : +1 vert. | vert 2 |
| 5 | Défausse une carte à la fin du tour, ne bouge pas. | +1 vert. | vert 3 |
| 6 | Joue une carte, ne bouge pas. | Déjà 3 verts : **rien**. | vert 3 |
| 7 | Quitte la mine et finit sur une plaine. | Pas sur une mine : rien. | vert 3 |

### Schéma : la fin du tour, avec l'étape de la case

```
EndTurn { discard }
   │
   ├─ 3. nettoyer la zone de jeu ............ TurnEnded            (016)
   ├─ 4. bénéfice de la case : mine ? ....... CrystalGained        (021, si moins de 3 cristaux de sa couleur)
   ├─ 7. défausser, puis repiocher .......... CardDiscarded…, CardDrawn…   (016)
   └─ tour suivant .......................... TurnStarted          (016)
```

## Modèle

### Termes

Termes du glossaire déjà présents : Cristal (`Crystal`), Mine (`Site::Mine(BasicColor)`), Rouge / Bleu / Blanc / Vert (`BasicColor`),
Zone de jeu (`PlayArea`), Fin du tour (`EndTurn`, `TurnEnded`), Source (`Source`), Dé de mana (`ManaDie`), Mana fondamental (spec 020).

Nouveaux termes, ajoutés au glossaire dans la même PR (à valider, Q2) :

| Livret (EN) | Français | Code |
|---|---|---|
| Inventory | Inventaire | `Inventory` |
| Gain (a crystal) | Gagner (un cristal) | `CrystalGained` (événement) |
| Mana token | Jeton de mana | — (spec 022) |
| Bank | Réserve | — (illimitée, rien à modéliser) |
| Crystal mine | Mine (de cristal) | `Site::Mine(BasicColor)` (existant) |

« Réserve » remplace « Banque », retiré du glossaire par la spec 020. La spec 016 parlait déjà de « réserve » (hors périmètre).

### Nouvelles données dans l'état

- l'**inventaire** du joueur : un nombre de cristaux, de 0 à 3, pour chacune des quatre couleurs de base. Public. Vide au départ.

La limite de trois est une constante (`CRYSTAL_LIMIT`). La couleur d'une mine est déjà dans le contenu des tuiles (spec 010).

## Actions et événements

Aucune action nouvelle. `EndTurn` (spec 016) produit un événement de plus quand le héros finit sur une mine.

| Action | Validation (refusée si…) | Événements produits |
|---|---|---|
| `EndTurn { discard }` (016) | inchangée | `TurnEnded`, puis **`CrystalGained` si la figurine est sur une mine et que l'inventaire a moins de 3 cristaux de sa couleur**, puis les `CardDiscarded`, les `CardDrawn` et `TurnStarted` (016) |

| Événement | Effet de `apply` | Révèle une information ? |
|---|---|---|
| `CrystalGained { player, color }`, `color` : couleur de base | ajoute un cristal de `color` à l'inventaire de `player` | oui, publique |

**Qui décide.** `decide` regarde la case de la figurine au moment de `EndTurn` et le nombre de cristaux de la couleur de la mine.
`apply` ne revérifie pas la limite (même principe que le nombre de cartes piochées, spec 016) : c'est `decide` qui garantit qu'aucun
`CrystalGained` ne la dépasse (L3).

**Actions légales et décision en attente** : inchangées. Le bénéfice de la mine n'ajoute aucun choix, donc aucune action `EndTurn` de
plus.

## Comportements attendus

Chaque cas devient au moins un test (nom indicatif entre parenthèses). Sauf mention, partie de bac à sable avec Tovak. Les cas « sur une
mine » partent d'un état construit en appliquant des événements (tuile de campagne posée, figurine placée sur la mine), pour ne pas
dépendre d'un long trajet.

Inventaire et mine (arcanum-core) :

- **C1** — Après `new_game`, l'inventaire a 0 cristal de chaque couleur (`hero_starts_without_crystals`) — MKUE p. 15.
- **C2** — Figurine sur la mine verte de la tuile de campagne 2, inventaire vide, `EndTurn` : l'inventaire a 1 vert, et rien d'autre
  (`ending_turn_on_a_mine_gains_a_crystal_of_its_color`) — MKUE p. 11, 4.b.
- **C3** — Même situation, `EndTurn` en défaussant une carte : les événements sont, dans l'ordre, `TurnEnded`, `CrystalGained { Green }`,
  `CardDiscarded`, les `CardDrawn`, `TurnStarted` (`mine_benefit_comes_after_clearing_play_area_and_before_drawing`) — MKUE p. 11, 3, 4
  puis 7.
- **C4** — Figurine sur la mine verte avec déjà 3 verts : aucun `CrystalGained`, l'inventaire reste à 3 verts, la fin du tour se passe
  comme dans la spec 016 (`full_color_gains_nothing_from_a_mine`) — MKUE p. 11, 4.b.
- **C5** — Figurine sur la mine verte avec 3 blancs et 0 vert : +1 vert ; la limite est par couleur (`crystal_limit_is_per_color`) —
  MKUE p. 5, 1.b.
- **C6** — Trois fins de tour de suite sur la mine verte, sans bouger : 1, 2 puis 3 verts ; à la quatrième, toujours 3
  (`staying_on_a_mine_gains_a_crystal_each_turn`) — MKUE p. 11, 4.b.
- **C7** — Chaque mine donne sa couleur : rouge (campagne 6), bleue (5), blanche (3), verte (2) (`each_mine_gives_its_own_color`) —
  MKUE p. 11, 4.b.
- **C8** — Fin du tour sur une case sans mine (plaine, village) : aucun `CrystalGained` (`ending_turn_elsewhere_gains_no_crystal`).
- **C9** — Traverser une mine pendant le tour puis finir sur une plaine : aucun cristal
  (`passing_through_a_mine_gains_nothing`) — MKUE p. 11, 4 (« if you end your turn on… »).
- **C10** — Sur une mine, les actions légales sont les mêmes qu'ailleurs avec la même main (`mine_adds_no_legal_action`).

Information cachée et journal (arcanum-core) :

- **C11** — La vue joueur contient l'inventaire ; le journal filtré montre `CrystalGained` en entier (`inventory_is_public`).
- **C12** — Rejouer le journal redonne le même inventaire (`replaying_log_rebuilds_inventory`).

Affichage (arcanum-ui) :

- **C13** — Le panneau latéral affiche une ligne « Inventaire », sous la ligne « Source » : pour chaque couleur de base, dans l'ordre
  rouge, bleu, blanc, vert, trois emplacements de cristal, autant de pleins que de cristaux (`panel_shows_inventory`).
- **C14** — Survoler une couleur de l'inventaire affiche « Cristaux verts : 1 sur 3 » (`inventory_tooltip_shows_count_and_limit`).
- **C15** — Survoler une case de mine ajoute, sous le nom du site, « Finir le tour ici : +1 cristal vert », ou « Vous avez déjà 3
  cristaux de cette couleur : rien à gagner ici. » si l'inventaire est plein dans cette couleur (`hovering_a_mine_tells_what_it_gives`)
  (Q4).
- **C16** — Avec le vrai `fr.json`, calculer l'écran pour un inventaire de 0 à 3 dans chaque couleur et survoler une mine de chaque
  couleur, pleine ou non, ne relève aucun manque (`every_inventory_key_exists_in_french`).

Vérification manuelle : `cargo run -p arcanum-ui` sous Linux ; explorer jusqu'à la tuile de campagne 2, finir le tour sur la mine verte :
l'inventaire passe à un cristal vert ; le survol de la mine annonce le gain, puis « rien à gagner » après trois tours.

## Cas limites et refus

- **L1** — Figurine sur une mine, fin du tour avec la zone de jeu vide et une carte défaussée (spec 016, C5) : le cristal est gagné
  quand même ; jouer une carte n'est pas une condition de la mine.
- **L2** — Fin du tour refusée (spec 016, L1 et L2) : aucun événement, donc aucun cristal.
- **L3** — `apply` d'un `CrystalGained` sur une couleur déjà à 3 : `decide` ne le produit jamais (C4) ; `apply` ajoute quand même, sans
  contrôle. Un test vérifie sur 200 fins de tour aléatoires sur des mines qu'aucune couleur ne dépasse 3
  (`inventory_never_exceeds_three_per_color`).
- **L4** — Jour ou nuit : la mine donne la même chose (aucune différence dans le livret). La nuit n'existe pas avant la spec 023.
- **L5** — Mine sur une tuile royaume (2, 3, 4 et 8 en ont une) : même règle, la mine ne dépend que du site de la case. C7 ajoute la
  mine verte de la tuile royaume 2 (`a_core_tile_mine_works_the_same`).

## Information cachée

- Publique : l'inventaire (nombre de cristaux de chaque couleur). Sur la table, la carte de héros est visible de tous.
- Rien de caché : `CrystalGained` ne porte aucun hasard.

## Libellés (i18n)

Ajoutés dans `fr.json` et `en.json`. Les noms de couleur au masculin singulier (`mana.green`…, spec 020) servent pour « cristal vert » ;
le pluriel a ses propres clés, sans accord calculé.

| Clé | Paramètres | Français | English |
|---|---|---|---|
| `inventory.title` | — | Inventaire | Inventory |
| `inventory.crystals.red` | `count`, `limit` | Cristaux rouges : {count} sur {limit} | Red crystals: {count} of {limit} |
| `inventory.crystals.blue` | `count`, `limit` | Cristaux bleus : {count} sur {limit} | Blue crystals: {count} of {limit} |
| `inventory.crystals.white` | `count`, `limit` | Cristaux blancs : {count} sur {limit} | White crystals: {count} of {limit} |
| `inventory.crystals.green` | `count`, `limit` | Cristaux verts : {count} sur {limit} | Green crystals: {count} of {limit} |
| `board.hover.mine.gain` | `color` | Finir le tour ici : +1 cristal {color} | End your turn here: +1 {color} crystal |
| `board.hover.mine.full` | — | Vous avez déjà 3 cristaux de cette couleur : rien à gagner ici. | You already have 3 crystals of this color: nothing to gain here. |

## Interface

- **Panneau latéral** : sous la ligne « Source » (spec 020), une ligne **Inventaire** : quatre groupes de trois petits losanges, un groupe
  par couleur de base, dans l'ordre rouge, bleu, blanc, vert. Un losange plein, de la couleur, par cristal ; les emplacements vides sont
  seulement cerclés. On voit d'un coup d'œil le nombre et la limite. Visuels provisoires, dessinés par nous (mêmes teintes que les
  pastilles des cartes et des dés).
- **Info-bulle** au survol d'un groupe (C14).
- **Survol d'une mine** : une ligne de plus dans la zone de survol (C15).
- Pas de clic sur un cristal : l'utiliser arrive avec la spec 022.

```
┌─ Panneau latéral ─────────────────────────────────┐
│ Bac à sable                                       │
│ Jour · Tour 4                                     │
│ Source :  ▣ vert  ▣ bleu   épuisé : ▢ noir        │
│ Inventaire :  ◇◇◇   ◇◇◇   ◇◇◇   ◆◆◇               │
│              rouge  bleu  blanc  vert (2 sur 3)   │
│ Mouvement : 0 · Influence : 0                     │
│ Main                                              │
│ …                                                 │
└───────────────────────────────────────────────────┘

Survol de la case :
  Colline · Mine verte
  Finir le tour ici : +1 cristal vert
```

(Les mots sous les losanges ne sont là que pour le schéma : à l'écran, la couleur suffit, et l'info-bulle donne le détail.)

### Raccourcis clavier

Aucun : l'inventaire se regarde, il ne se manipule pas encore (Q5).

## Effets sur les specs déjà implémentées

Signalés ici parce que le sens de tests existants change (règle « jamais modifier un test sans le signaler ») :

- **016** : la fin du tour gagne l'étape 4 (bénéfice de la case) pour la mine. Les tests de la 016 ne finissent pas sur une mine et
  ne changent pas ; la liste exhaustive des événements de `TurnEnded` à `TurnStarted` gagne une branche `CrystalGained`. Le « Hors
  périmètre » de la 016 renvoyait la mine à la spec 044 : elle est avancée ici (Q3), la clairière magique reste à la 044.
- **002, 011, 012, 014, 016, 020** : `PlayerView`, construit champ par champ dans des tests, gagne l'inventaire.
- **011, 012, 014, 015, 017** : `BoardInput` (arcanum-ui), construit dans des tests, gagne `inventory` (`None` hors partie) : c'est lui
  qui donne la ligne de survol des mines, y compris en mode fin du tour.

## Hors périmètre

- Utiliser un cristal (le transformer en mana fondamental), les jetons de mana dans la zone de jeu et leur retour à la réserve en fin de
  tour (p. 11, 3.a) : spec 022, avec le premier usage du mana (Q1).
- Effets de carte qui font gagner un cristal ou un jeton de mana (Cristallisation, Euphorie cristalline, Source de mana), et le jeton
  gagné à la place d'un quatrième cristal (p. 5, « "Gain" effects » 2) : spec 022.
- Récompenses de combat en cristaux, cristal au hasard en lançant un dé de la réserve (p. 11, 5.a) : spec 031.
- Clairière magique (p. 11, 4.a) : spec 044, avec les Blessures (034).
- Cristaux du joueur fantôme (trois au départ, plus de trois par couleur, p. 15) : spec 061.
- Score des cristaux en fin de partie (p. 15, 17) : specs 063 et 064.
- Mines gardées ou cédées par les scénarios (p. 20) : scénarios concernés, hors périmètre pour l'instant.

## Questions ouvertes

Aucune. Réponses de Mathieu (2026-10-25) :

1. **Q1** — Périmètre : l'inventaire, la limite de trois et la mine ; utiliser un cristal et les jetons de mana passent dans la 022.
2. **Q2** — Termes acceptés : « Inventaire » est bien le mot de l'édition française (carte de héros) ; « Réserve », « Jeton de mana »,
   « Mine ».
3. **Q3** — La mine est avancée de la 044 à la 021.
4. **Q4** — Oui : la ligne d'aide au survol d'une mine.
5. **Q5** — Aucun raccourci.
