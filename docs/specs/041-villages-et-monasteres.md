# 041 — Villages et monastères

- Statut : implémentée (2026-11-04) ; validée le 2026-11-04, propositions de Q1 à Q3 retenues par Mathieu, « prenez un
  artefact » pour Q3
- Jalon : J4
- Dépend de : 015 (exploration), 023 (nouvelle manche), 024 (retour en arrière), 031 à 034 (combat, capacités des ennemis,
  soins), 035 (gloire et niveaux), 040 (interaction, réputation), 042 (unités, réserve de points de soin), 050 (offres, pioches
  communes, choix d'action avancée en fin de tour), 063 (décompte final) ; toutes implémentées
- Livret : MKUE p. 4 (2.c, deux derniers points) ; p. 7 (5.f) ; p. 8 (*Interacting with Locals* 4.b, 4.c et 5 ; *Combat with
  Enemies* 1.c et 2) ; p. 10 (*Combat Outcomes* 1 et 2) ; p. 11 (*End of the Turn* 5.b ; *Wounds and Healing* 3.b et 3.c) ; p. 15
  (*The Greatest Loot*, *The Greatest Conqueror*). Cartes de site Village et Monastère : `docs/regles/cartes/sites-et-cites.md` ;
  termes français : photo `docs/regles/scans/fr/rule_cards/`. FAQ p3 (*Règles généralement oubliées* : monastère révélé, ennemi
  défaussé, défenseur non fortifié). Compendium, *Map Revealing* S1, S2 ; *Combat* S9, S17, S36, S39, S43 ; *Influence* S5, S7,
  S8 ; *Healing* S4 ; *Resting* S5 ; *Miscellaneous* S5, S7 ; *Turn Start* S1 ; *End of Turn* S5 ; *Training* S4
- Crates touchées : arcanum-core (règles), arcanum-ui, arcanum-persistance (une partie préparée), `assets/i18n/`

## Objectif

Au village et au monastère, l'interaction (spec 040) achète enfin autre chose que des unités : des **points de soin** (3 influence
au village, 2 au monastère) et, au monastère, les **actions avancées** que chaque monastère révélé met dans l'offre d'unités
(6 influence). Le héros peut aussi **incendier un monastère** : Réputation −3, combat sans ses unités contre un ennemi violet tiré
au hasard ; s'il gagne, un jeton Bouclier marque la case, le monastère est détruit et un **artefact** arrive en récompense à la
fin du tour. Le décompte final compte enfin les artefacts (Le Collectionneur) et les monastères incendiés (Le Conquérant).

## Règles

### Actions avancées des monastères (« Entraînement »)

- **Monastère révélé** (MKUE p. 7, 5.f ; carte du monastère, *Training*) : pour chaque monastère d'une tuile qui vient d'être
  révélée, la carte du dessus de la pioche d'actions avancées entre dans l'**offre d'unités**, face visible, à la suite des unités.
  Pioche vide : rien n'entre (le livret ne dit rien ; pas de remélange, cette pioche n'a pas de défausse).
- **Monastères de la mise en place** (Compendium, *Map Revealing* S1) : de même, une carte par monastère des tuiles posées par la
  mise en place, une fois l'offre d'unités faite. Aucun scénario actuel n'en a (bac à sable et Première reconnaissance posent les
  campagnes 1 et 2) : la règle est testée sur un scénario de test.
- **Nouvelle manche** (MKUE p. 4, 2.c ; FAQ p3) : les actions avancées de l'offre d'unités vont **sous** la pioche d'actions
  avancées avec les unités, puis, une fois les nouvelles unités tirées, une carte entre par monastère **non incendié** du plateau
  de jeu. Avant le renouvellement de l'offre d'actions avancées (2.d), dans l'ordre du livret. Un monastère incendié en cours de
  manche n'enlève rien de l'offre avant la manche suivante (Compendium, *Map Revealing* S2).
- **Elles restent dans l'offre d'unités** : on ne les recrute pas (spec 042), et le choix d'action avancée d'un niveau pair ne les
  prend pas (Compendium, *Training* S4) : il ne regarde que l'offre d'actions avancées (spec 050).

### Acheter au village et au monastère

- **Points de soin** (MKUE p. 8, 4.b ; p. 11, *Wounds and Healing* 3.b ; cartes de site) : pendant une interaction, 1 point de soin
  pour **3** influence au village, **2** au monastère. Autant de fois que l'influence le permet (MKUE p. 8, 5 ; Compendium,
  *Healing* S4). Le point entre dans la **réserve de points de soin** de la spec 042 (son Q1) et se dépense comme les autres
  (`SpendHealing` : une Blessure de la main, ou une unité pour son niveau), aussitôt ou plus tard dans le tour, jamais en combat
  (il n'y en a plus ce tour : l'interaction est l'action).
- **Action avancée** (MKUE p. 8, 4.c ; carte du monastère) : pendant une interaction **au monastère**, n'importe quelle action
  avancée de l'offre d'unités, pour **6** influence, quel que soit le monastère qui l'y a mise. Elle va **sur le dessus du deck**
  (comme une carte gagnée, spec 050). L'offre d'unités n'est **pas complétée** (MKUE p. 8, 4.c ; Compendium, *Miscellaneous* S7).
  Au village : pas d'action avancée.
- **Influence suffisante** : comme pour recruter (spec 042) : l'influence du tour, signée (spec 040), atteint le coût ; elle baisse
  du coût. Achats, recrutements et soins se mêlent dans n'importe quel ordre (MKUE p. 8, 5 ; Compendium, *Influence* S8).
- **Repos** (Compendium, *Resting* S5) : acheter, c'est interagir ; donc pas pendant un tour de repos (spec 040, inchangé).

### Incendier un monastère

- **C'est l'action du tour** (MKUE p. 8, *Combat with Enemies* 1.c ; carte du monastère ; Compendium, *Influence* S7, *Combat*
  S17) : la figurine est sur un monastère non incendié. Mêmes conditions que l'interaction (spec 040) : ni interaction, ni combat
  engagé, ni ennemi provoqué ou défié ce tour, pas de choix d'action avancée en attente, partie en cours. Un pas qui provoque un
  ennemi errant en arrivant sur le monastère l'interdit donc ce tour (S17). Les points de mouvement et d'influence non dépensés
  sont perdus (MKUE p. 6, 6.e), plus de déplacement ni de repos, comme pour l'interaction.
- **Sur la case X aussi** (Compendium, *Influence* S5) : la réputation n'empêche pas d'incendier.
- **Déroulé** : Réputation **−3**, aussitôt (carte FR : « Perdez immédiatement 3 points de Réputation »), puis un jeton
  **violet** est tiré (pile de secours si la pile violette et sa défausse sont vides, spec 033) et le combat est **engagé
  aussitôt** contre lui seul : aucun ennemi errant ne peut y être défié (S17 ; MKUE p. 8, 2.c ne s'applique pas).
- **Ennemi non fortifié** (FAQ p3 ; Compendium, *Combat* S9) : le monastère n'est pas un site fortifié. Aucun jeton violet du jeu
  de base n'a la capacité Fortifié. Il rapporte sa gloire entière (Compendium, *Combat* S36).
- **Sans les unités** (carte du monastère : « Your Units cannot be used in this combat » ; Compendium, *Combat* S43) : du début à
  la fin de ce combat, aucune unité ne s'active et aucune ne reçoit de dégâts. La phase d'assignation se résout donc seule, sur
  le héros (spec 042, C29). Avant et après le combat, dans le même tour, les unités s'utilisent comme d'habitude.
- **Victoire** (carte du monastère ; MKUE p. 10, *Combat Outcomes* 2) : l'ennemi vaincu va dans la défausse violette (spec 031) ;
  un **jeton Bouclier** du joueur marque la case ; le monastère est **incendié** ; une récompense **artefact** est gagnée, prise à
  la fin du tour.
- **Échec** (carte FR : « Que l'ennemi soit vaincu ou non, défaussez-le » ; FAQ p3 ; Compendium, *Combat* S39) : l'ennemi va dans
  la défausse violette, le monastère reste intact, la réputation perdue l'est pour de bon. Une nouvelle tentative, un autre tour,
  tire un nouveau jeton.
- **Monastère incendié** (Compendium, *Combat* S17, second paragraphe) : la case est une case vide de son terrain. Plus
  d'interaction, plus d'action avancée à la nouvelle manche, plus d'incendie. On y entre et on y passe librement.

### Récompense : un artefact

- **Fin du tour** (MKUE p. 11, *End of the Turn* 5.b ; Compendium, *End of Turn* S5) : à l'étape des récompenses, après le
  cristal de la mine (4.b) et avant le changement de niveau (6), le joueur tire **deux** artefacts de la pioche d'artefacts, en
  **en prend un**, sur le dessus de son deck, et remet l'autre **sous** la pioche. Puis vient le choix d'action avancée d'un niveau
  pair (spec 050), puis la pioche de fin de tour : l'artefact pris est donc pioché aussitôt s'il reste de la place.
- **Pioche courte** : une seule carte, elle est prise sans choix ; aucune, la récompense est perdue. Le livret ne dit rien ;
  la pioche a 16 cartes et un seul tirage par monastère, le cas est théorique.
- **Effet** : les artefacts sont en données, effets `Unsupported` jusqu'à la spec 055 ; ils se jouent de côté (spec 050, Q5).
- **Dernier tour de la partie** (spec 063) : la récompense est prise à la fin du tour comme d'habitude, avant le décompte.

### Décompte final

- **Le Collectionneur** (MKUE p. 15, *The Greatest Loot*) : 2 par artefact du joueur (deck, main, défausse, zone de jeu, comptés
  comme les sorts, spec 050), plus 1 par deux cristaux (spec 063). « Sur ses unités » : bannières, spec 055.
- **Le Conquérant** (MKUE p. 15, *The Greatest Conqueror*) : 2 par jeton Bouclier du joueur sur une forteresse, une tour de mage
  ou un **monastère**. Seuls les monastères incendiés en posent pour l'instant (forteresses et tours de mage : spec 043). Sans bonus
  de titre (spec 063).

### Piller un village

- **Carte du village** (*Plundering* ; Compendium, *Turn Start* S1, *Miscellaneous* S5) : « avant son tour », figurine sur un
  village : piocher deux cartes, Réputation −1, une fois entre deux de ses tours. La spec 040 l'avait mis hors périmètre parce
  qu'en solo il n'y a pas de tour d'un autre joueur ; le compendium en fait pourtant une option **avant son propre tour**, permise
  en solo, même au premier tour d'une manche (S5). Décision (Q1) : la coder ici, comme une décision offerte **au début du tour,
  avant toute autre action** (avant la première carte, le premier pas ou l'annonce de fin de manche), sur un village, une fois par
  tour. Pioche sans remélange si le deck s'épuise (MKUE p. 11, 7.d) ; sur la case X aussi (*Influence* S5).

### Exemple : un tour au monastère, puis un incendie

Bac à sable, Jour. La figurine est sur le monastère de la campagne 5, révélé au tour d'avant : l'offre d'unités contient trois
unités et l'action avancée Bouclier de glace. Réputation 0, une Blessure en main, influence en main : Promesse, Menace, Rage.

| Étape | Action | Résultat |
|---|---|---|
| 1 | Interagir ; Promesse (Influence 2), Menace (Influence 2), Rage de côté (Influence 1) | influence 5 |
| 2 | Acheter un point de soin | influence 3 ; soin 1 |
| 3 | Dépenser le point de soin sur la main | la Blessure est jetée |
| 4 | Fin du tour | l'influence restante (3) est perdue |

Tour suivant, toujours sur le monastère, le joueur l'incendie plutôt que d'y acheter Bouclier de glace :

```
Incendier le monastère                 → ReputationChanged −3 ; jeton violet tiré (Moines) ; combat engagé, unités exclues
  phase d'attaque à distance … phase d'attaque : Moines vaincus (gloire 4)
Fin du combat                          → MonasteryBurned : Bouclier sur la case, monastère incendié, récompense : 1 artefact
Fin du tour                            → deux artefacts tirés ; le joueur en prend un (dessus du deck), l'autre sous la pioche
                                         puis pioche jusqu'à la limite de main : l'artefact arrive en main
Nouvelle manche                        → Bouclier de glace retourne sous la pioche ; aucune action avancée n'entre (plus de monastère)
```

## Modèle

Termes du glossaire : Village, Monastère, Interaction, Influence, Points de soin, Offre d'unités, Pioche d'actions avancées,
Pioche d'artefacts, Action avancée, Artefact, Jeton Bouclier, Réputation, Le Collectionneur, Le Conquérant. **Nouveaux termes**
(ajoutés au glossaire avec cette spec ; français de la carte de site FR, photo `scans/fr/rule_cards/`) :

| Livret | Français | Code |
|---|---|---|
| Buy Healing | Acheter un point de soin | `Action::BuyHealing`, `HealingBought` |
| Training (monastery) | Entraînement ; acheter une action avancée | `Action::BuyAdvancedAction`, `AdvancedActionBought` |
| Burn a monastery | Incendier un monastère | `Action::BurnMonastery` |
| Burned monastery | Monastère incendié (carte FR : « brûlé ») | `MonasteryBurned` |
| Shield token | Jeton Bouclier (FAQ p3) | `GameState::shields` |
| Combat reward | Récompense (de combat) | `PlayerState::artifact_rewards` |
| Plunder a village | Piller un village | `Action::PlunderVillage`, `VillagePlundered` |

Données nouvelles :

- **Plateau de jeu** : les **jetons Bouclier** posés, par case (le joueur qui l'a posé) ; les **monastères incendiés** (une case
  marquée d'un Bouclier sur un monastère). Les specs 043 et 044 en poseront d'autres.
- **Offre d'unités** : elle peut contenir des actions avancées (`CommonCard::Deed`), à la suite des unités ; rien ne change à sa
  forme (spec 050).
- **État du joueur** : les **récompenses artefact** gagnées ce tour, prises à la fin du tour ; les **artefacts tirés** en attente
  du choix ; si le combat en cours **exclut les unités** (incendie) ; le pillage fait ce tour.
- **Contenu** : coûts des achats (3 et 2 influence le point de soin, 6 l'action avancée) en constantes du moteur, comme le coût
  d'exploration ; ils sont imprimés sur les cartes de site, pas sur une carte de contenu.
- **Vue du joueur** : jetons Bouclier, monastères incendiés, actions avancées de l'offre d'unités, récompenses en attente, les deux
  artefacts tirés pendant le choix ; tout est public (en solo, personne d'autre ne regarde).

## Actions et événements

| Action | Validation (refusée si…) | Événements produits |
|---|---|---|
| `BuyHealing` (nouvelle) | pas d'interaction en cours ; influence < 3 (village) ou < 2 (monastère) | `HealingBought { player, cost }` |
| `BuyAdvancedAction { card }` (nouvelle) | pas d'interaction en cours au monastère ; carte absente de l'offre d'unités ; influence < 6 | `AdvancedActionBought { player, card, cost }` |
| `BurnMonastery` (nouvelle) | figurine pas sur un monastère, ou monastère incendié ; interaction faite, combat engagé, ennemi provoqué ou défié ce tour ; choix d'action avancée en attente ; partie finie | `MonasteryBurnStarted { player, hex }`, `ReputationChanged { change: −3 }`, tirage du jeton (`EnemyPileReshuffled` au besoin, 031 et 033), `EnemyTokenPlaced { token, hex }`, `CombatStarted` |
| `EndCombat` (031, étendue) | inchangée | `CombatEnded`, puis pour un incendie : `MonasteryBurned { player, hex }` si l'ennemi est vaincu, sinon `EnemyTokenDiscarded { hex }` |
| `ActivateUnit`, `AssignDamage` (042) | refusées pendant un incendie | — |
| `EndTurn` (016, étendue) | inchangée | … cristal de la mine, puis, avec une récompense artefact : `ArtifactsDrawn { player, cards }` ; la pioche de fin de tour attend le choix (comme le choix d'action avancée, spec 050) |
| `TakeArtifact { card }` (nouvelle) | pas d'artefacts tirés en attente ; carte pas parmi eux | `ArtifactTaken { player, card }`, `ArtifactReturned { card }` (l'autre, s'il y en a un) ; puis, plus de choix en attente, la suite de la fin du tour (choix d'action avancée, pioche) |
| `Explore` (015, étendue) | inchangée | … `TileRevealed`, jetons, puis un `OfferCardDealt { offer: Units, card }` d'action avancée par monastère de la tuile |
| `AnnounceEndOfRound` (023, nouvelle manche) | inchangée | `OfferCardReturned` aussi pour les actions avancées de l'offre d'unités, puis après les unités un `OfferCardDealt { offer: Units }` d'action avancée par monastère non incendié |
| Mise en place (012) | — | après l'offre d'unités, un `OfferCardDealt { offer: Units }` d'action avancée par monastère posé |
| `PlunderVillage` (nouvelle) | figurine pas sur un village ; une carte jouée, un pas, une action ou une décision déjà prise ce tour ; déjà pillé ce tour ; partie finie | `VillagePlundered { player }`, `ReputationChanged { change: −1 }`, `CardDrawn` (deux au plus) |

Ordre des actions légales (recette de `architecture.md`) : `PlunderVillage` en tête au début du tour ; `BurnMonastery` juste après
`Interact` ; en interaction, après les recrutements : `BuyAdvancedAction` dans l'ordre de l'offre d'unités, puis `BuyHealing` ;
`TakeArtifact` dans l'ordre du tirage.

| Événement | Effet de `apply` | Révèle une information ? | Point de non-retour ? |
|---|---|---|---|
| `HealingBought { player, cost }` | influence − `cost` ; points de soin + 1 | non (publique) | non |
| `AdvancedActionBought { player, card, cost }` | la carte quitte l'offre d'unités (sans la compléter) pour le dessus du deck ; influence − `cost` | non | non |
| `MonasteryBurnStarted { player, hex }` | points de mouvement et d'influence à 0 ; plus de déplacement ni de repos ; le combat qui suit exclut les unités | non | non |
| `EnemyTokenPlaced` (030, réutilisé) | le jeton violet tiré est posé sur le monastère | oui (jeton tiré) | **oui**, comme tout jeton révélé |
| `MonasteryBurned { player, hex }` | jeton Bouclier du joueur sur `hex` ; monastère incendié ; une récompense artefact de plus | non | non |
| `EnemyTokenDiscarded { hex }` | le jeton de `hex` quitte le plateau pour la défausse de sa couleur | non | non |
| `ArtifactsDrawn { player, cards }` | les cartes quittent le dessus de la pioche d'artefacts et attendent le choix ; une récompense de moins | oui (cartes tirées) | **oui** |
| `ArtifactTaken { player, card }` | la carte va sur le dessus du deck | non | non |
| `ArtifactReturned { card }` | la carte va sous la pioche d'artefacts | non | non |
| `OfferCardDealt`, `OfferCardReturned` (050) | inchangés ; une action avancée de l'offre d'unités retourne sous la pioche d'actions avancées (`CommonCard::deck`) | inchangé | inchangé |
| `VillagePlundered { player }` | pillage fait ce tour | non | non |
| `CardDrawn` (016, réutilisé) | inchangé | inchangé (pioche cachée) | inchangé |
| `TurnEnded` (016) | vide aussi le pillage et l'exclusion des unités | inchangé | inchangé |

Aucun événement existant ne change de forme. Les parties préparées sont régénérées si leur construction change.

## Comportements attendus

Chaque cas devient au moins un test (nom indicatif entre parenthèses). C1 à C33 : moteur (arcanum-core) ; C34 à C43 : écran (arcanum-ui,
sur `game_screen`), un cas par point de la section « Interface » (issue privée 119).

Actions avancées des monastères :

- **C1** — Étant donné le bac à sable, quand l'exploration révèle la campagne 5 (un monastère), alors `TileRevealed` est suivi d'un
  `OfferCardDealt { offer: Units }` de la carte du dessus de la pioche d'actions avancées ; l'offre d'unités a quatre cartes, la
  quatrième une action avancée (`revealed_monastery_adds_an_advanced_action`) — MKUE p. 7, 5.f.
- **C2** — Une tuile révélée sans monastère n'ajoute rien ; la pioche d'actions avancées vide n'ajoute rien non plus
  (`no_monastery_no_advanced_action`) — MKUE p. 7, 5.f.
- **C3** — Un scénario de test dont une tuile de la mise en place porte un monastère : l'offre d'unités de la manche 1 a une action
  avancée après ses unités (`setup_monastery_adds_an_advanced_action`) — Compendium, *Map Revealing* S1.
- **C4** — Nouvelle manche avec une action avancée dans l'offre d'unités et un monastère non incendié : l'action avancée retourne
  sous la pioche d'actions avancées avec les unités, les nouvelles unités sont tirées, puis une nouvelle action avancée entre, avant
  le renouvellement de l'offre d'actions avancées (`new_round_renews_monastery_advanced_actions`) — MKUE p. 4, 2.c ; FAQ p3.
- **C5** — Monastère incendié pendant la manche : son action avancée reste dans l'offre d'unités jusqu'à la nouvelle manche, qui
  n'en ajoute plus (`burned_monastery_adds_nothing_at_new_round`) — Compendium, *Map Revealing* S2.
- **C6** — Une action avancée de l'offre d'unités n'est ni recrutable (`RecruitUnit`) ni proposée au choix d'action avancée d'un
  niveau pair (`monastery_advanced_actions_are_not_units_nor_offer`) — Compendium, *Training* S4.

Acheter :

- **C7** — Interaction au village, influence 6 : `BuyHealing` deux fois, deux `HealingBought { cost: 3 }`, influence 0, soin 2 ; une
  troisième fois n'est plus légale (`village_sells_healing_for_three`) — MKUE p. 8, 4.b ; Compendium, *Healing* S4.
- **C8** — Au monastère, le point de soin coûte 2 (`monastery_sells_healing_for_two`) — carte du monastère.
- **C9** — Un point acheté se dépense par `SpendHealing` sur une Blessure de la main, ou s'ajoute à d'autres pour une unité
  blessée de niveau II (`bought_healing_joins_the_reserve`) — MKUE p. 11, 3.c ; spec 042.
- **C10** — Interaction au monastère, influence 6, Bouclier de glace dans l'offre d'unités : `AdvancedActionBought`, la carte est sur
  le dessus du deck, influence 0, l'offre d'unités a une carte de moins et n'est pas complétée
  (`monastery_sells_advanced_actions`) — MKUE p. 8, 4.c ; Compendium, *Miscellaneous* S7.
- **C11** — Deux actions avancées dans l'offre d'unités (deux monastères révélés) : les deux s'achètent au même monastère, l'une après
  l'autre avec 12 influence (`any_monastery_sells_every_monastery_card`) — carte du monastère ; Compendium, *Map Revealing* S2.
- **C12** — Au village, `BuyAdvancedAction` n'est pas légale ; influence 5 au monastère non plus ; influence négative : aucun achat
  (`buying_needs_the_site_and_enough_influence`) — MKUE p. 8, 4.
- **C13** — Hors interaction, ni `BuyHealing` ni `BuyAdvancedAction` (`buying_needs_an_interaction`) — MKUE p. 8, 4.
- **C14** — Recruter, acheter un point de soin, activer une unité pour de l'influence, acheter une action avancée : tout se mêle dans
  la même interaction (`purchases_mix_in_one_interaction`) — MKUE p. 8, 5 ; Compendium, *Influence* S8.

Incendier un monastère :

- **C15** — Figurine sur un monastère, réputation 0 : `BurnMonastery` est légale ; quand le joueur incendie, alors
  `MonasteryBurnStarted`, `ReputationChanged { change: −3 }`, `EnemyTokenPlaced` d'un jeton violet sur le monastère, puis
  `CombatStarted` avec ce seul ennemi (`burning_starts_a_combat_against_a_violet_enemy`) — MKUE p. 8, 1.c.
- **C16** — Les points de mouvement et d'influence non dépensés sont perdus ; plus de déplacement, d'interaction ni de repos
  (`burning_is_the_action_of_the_turn`) — MKUE p. 6, 6.e ; Compendium, *Influence* S7.
- **C17** — Après une interaction, après un défi, ou quand le pas sur le monastère a provoqué un ennemi errant : `BurnMonastery`
  n'est pas légale ; pendant l'incendie, `Challenge` non plus (`burning_excludes_other_combats`) — Compendium, *Combat* S17.
- **C18** — Réputation sur X : `BurnMonastery` est légale ; réputation −5 : elle tombe à −7 (`burning_works_on_x`) — Compendium,
  *Influence* S5 ; spec 040, C3.
- **C19** — L'ennemi du monastère n'est pas fortifié : une attaque à distance le vise dès la première phase
  (`monastery_defender_is_not_fortified`) — FAQ p3 ; Compendium, *Combat* S9.
- **C20** — Pendant l'incendie, avec des unités disponibles et non blessées : aucun `ActivateUnit` n'est légal, et la phase
  d'assignation se résout seule sur le héros ; après le combat, les unités s'activent à nouveau (`units_sit_out_the_burning`) —
  carte du monastère ; Compendium, *Combat* S43.
- **C21** — Ennemi vaincu, combat fini : `CombatEnded`, puis `MonasteryBurned` ; Bouclier sur la case, monastère incendié, gloire
  entière de l'ennemi, une récompense artefact (`victory_burns_the_monastery`) — carte du monastère ; Compendium, *Combat* S36.
- **C22** — Combat fini sans victoire : `EnemyTokenDiscarded` ; le jeton est dans la défausse violette, le monastère est intact, la
  réputation reste à −3 ; au tour suivant, un nouvel incendie tire un autre jeton (`failed_burning_discards_the_enemy`) — FAQ p3 ;
  Compendium, *Combat* S39.
- **C23** — Sur un monastère incendié : ni `Interact` ni `BurnMonastery` ; la figurine y entre et en sort comme sur une plaine
  (`burned_monastery_is_an_empty_space`) — Compendium, *Combat* S17.

Récompense :

- **C24** — Fin du tour après un incendie réussi : `ArtifactsDrawn` avec les deux cartes du dessus de la pioche d'artefacts, la
  pioche de fin de tour attend ; `TakeArtifact` sur la seconde : `ArtifactTaken` (dessus du deck), `ArtifactReturned` (la première,
  sous la pioche), puis la pioche jusqu'à la limite de main, qui prend l'artefact (`artifact_reward_draws_two_takes_one`) — MKUE
  p. 11, 5.b ; Compendium, *End of Turn* S5.
- **C25** — Le même tour, la gloire franchit un niveau pair : le choix de l'artefact vient d'abord, puis le choix d'action avancée,
  puis la pioche (`artifact_reward_comes_before_level_up`) — MKUE p. 11, 5 et 6.
- **C26** — Pioche d'artefacts à une carte : elle est prise sans choix ; vide : rien (`short_artifact_deck`) — interprétation.
- **C27** — L'artefact pris se joue de côté, son effet est `Unsupported` (`taken_artifact_plays_sideways`) — spec 050, Q5.

Décompte, retour en arrière, sauvegarde, vue :

- **C28** — Fin de partie avec un artefact dans le deck, deux cristaux et un monastère incendié : Le Collectionneur 2 + 1 = 3, Le
  Conquérant 2 (`final_score_counts_artifacts_and_shields`) — MKUE p. 15.
- **C29** — Revenir en arrière après `BuyHealing`, `BuyAdvancedAction` ou `TakeArtifact` rétablit l'état d'avant ; pas avant le jeton
  violet tiré, ni avant `ArtifactsDrawn` (`purchases_undo_burning_does_not`) — spec 024.
- **C30** — Une partie sauvegardée avec une action avancée dans l'offre d'unités, un monastère incendié et des artefacts tirés en
  attente du choix se recharge à l'identique (`monasteries_survive_save_and_load`) — spec 024.
- **C31** — La vue du joueur donne les jetons Bouclier, les monastères incendiés, les récompenses et les artefacts tirés en attente
  (`view_shows_shields_and_rewards`).

Piller un village :

- **C32** — Début du tour, figurine sur un village, réputation 0 : `PlunderVillage` est légale ; quand le joueur pille, alors
  `VillagePlundered`, `ReputationChanged { change: −1 }` et deux `CardDrawn` (`plundering_draws_two_cards`) — carte du village.
- **C33** — Après la première carte jouée, le premier pas, ou un premier pillage, `PlunderVillage` n'est plus légale ; deck à une
  carte : une seule piochée (`plundering_only_before_the_turn`) — Compendium, *Turn Start* S1 ; MKUE p. 11, 7.d.

Écran (arcanum-ui, sur `game_screen`) :

- **C34** — En interaction au village : le bouton « Acheter un point de soin (3) », grisé si l'influence ne suffit pas ; la ligne
  « Rien à acheter ici » n'apparaît plus (`panel_offers_healing_purchase`).
- **C35** — En interaction au monastère : un bouton « Acheter : Bouclier de glace (6) » par action avancée de l'offre d'unités, grisé
  si l'influence ne suffit pas, avec le texte de la carte en infobulle ; « Acheter un point de soin (2) »
  (`panel_offers_advanced_actions_at_monastery`).
- **C36** — La fenêtre Offres montre les actions avancées de l'offre d'unités après les unités, marquées « monastère », sans bouton
  de recrutement (`offers_window_shows_monastery_cards`).
- **C37** — Sur un monastère, le bouton « Incendier le monastère » quand `BurnMonastery` est légale, avec la ligne des points perdus
  (comme `interaction.lost_points`) ; un clic ouvre une confirmation qui dit le prix (Réputation −3, ennemi violet, sans les unités,
  pas de retour en arrière) ; « Annuler » ne fait rien (`burn_button_asks_confirmation`).
- **C38** — Écran de combat d'un incendie : la ligne « Vos unités ne participent pas à ce combat » à la place des capacités
  d'unités (`burning_combat_hides_units`).
- **C39** — Plateau : un monastère incendié est dessiné avec un jeton Bouclier sur sa case ; son infobulle dit « Monastère
  incendié » (`board_shows_burned_monastery`).
- **C40** — Fin du tour avec une récompense : la fenêtre « Récompense : prenez un artefact » montre les deux artefacts (nom et
  texte) et un bouton « Prendre » chacun (`artifact_choice_window`).
- **C41** — Écran de fin : « Le Collectionneur (1 artefact(s), 2 cristaux) : 3 » et « Le Conquérant (1 Bouclier(s)) : 2 »
  (`score_shows_artifacts_and_shields`).
- **C42** — Début du tour sur un village : le bouton « Piller le village (2 cartes, Réputation −1) » ; absent dès la première
  carte jouée ou le premier pas (`plunder_button_at_turn_start`).
- **C43** — Panneau : une récompense gagnée ce tour s'annonce sous les points, « Récompense en fin de tour : 1 artefact »
  (`panel_announces_pending_reward`).

## Cas limites et refus

- **L1** — Acheter la même action avancée deux fois : refusé (elle n'est plus dans l'offre d'unités).
- **L2** — Points de soin achetés sans Blessure en main ni unité blessée : permis ; ils sont perdus à la fin du tour (spec 042,
  L4).
- **L3** — Incendier avec un deck vide au début du tour : permis, comme l'interaction ; l'annonce de fin de manche n'est plus
  possible ensuite (spec 040, L3).
- **L4** — Héros assommé pendant l'incendie : le combat continue, comme tout combat (spec 034) ; une victoire incendie quand même
  le monastère.
- **L5** — Un Illusionniste violet tiré (Invocation) : il invoque une créature marron comme d'habitude (spec 033) ; la créature
  n'est pas « l'ennemi du monastère » : seul le jeton violet compte pour la victoire.
- **L6** — Monastère incendié au dernier tour (spec 063) : l'artefact est tiré et pris à la fin du tour, il compte au décompte.
- **L7** — Pile violette et sa défausse vides : la pile de secours de la spec 033 fournit le jeton.
- **L8** — Deux monastères révélés sur la même tuile : aucune tuile n'en a deux (`tuiles.md`) ; la règle « un par monastère »
  suffit.

## Information cachée

L'ordre des pioches reste caché (spec 050). Le jeton violet tiré et les deux artefacts tirés sont révélés par leurs événements ;
l'artefact remis sous la pioche est connu du joueur, qui l'a vu. En solo rien d'autre n'est à filtrer ; en multijoueur (hors
périmètre), les artefacts tirés ne seraient montrés qu'à leur joueur.

## Libellés (i18n)

| Clé | Paramètres | fr | en |
|---|---|---|---|
| `purchase.healing` | `cost` | Acheter un point de soin ({cost}) | Buy a Healing point ({cost}) |
| `purchase.advanced_action` | `name`, `cost` | Acheter : {name} ({cost}) | Buy: {name} ({cost}) |
| `offers.monastery_card` | — | monastère | monastery |
| `burn.button` | — | Incendier le monastère | Burn the monastery |
| `burn.confirm` | — | Incendier le monastère ? Réputation −3, combat contre un ennemi violet, sans vos unités. Pas de retour en arrière. | Burn the monastery? Reputation −3, combat against a violet enemy, without your Units. This cannot be undone. |
| `burn.confirm_yes` / `burn.cancel` | — | Incendier / Annuler | Burn / Cancel |
| `burn.no_units` | — | Vos unités ne participent pas à ce combat | Your Units do not take part in this combat |
| `site.monastery.burned` | — | Monastère incendié | Burned monastery |
| `reward.pending` | `count` | Récompense en fin de tour : {count} artefact(s) | Reward at end of turn: {count} Artifact(s) |
| `reward.choose_artifact` | — | Récompense : prenez un artefact | Reward: take one Artifact |
| `reward.take` | — | Prendre | Take |
| `plunder.button` | — | Piller le village (2 cartes, Réputation −1) | Plunder the village (2 cards, Reputation −1) |
| `score.loot` (changée) | `artifacts`, `crystals`, `points` | Le Collectionneur ({artifacts} artefact(s), {crystals} cristaux) : {points} | The Greatest Loot ({artifacts} Artifact(s), {crystals} crystals): {points} |
| `score.conqueror` (changée) | `shields`, `points` | Le Conquérant ({shields} Bouclier(s)) : {points} | The Greatest Conqueror ({shields} Shield(s)): {points} |

`interaction.nothing_to_buy` n'est plus affichée (village et monastère ont toujours le point de soin) : la clé est retirée.
« Points perdus » réutilise `interaction.lost_points` avec un libellé voisin, `burn.lost_points` (même paramètres) : « Incendier fait
perdre : {move} mouvement, {influence} influence ».

## Interface

- **Interaction** (C34, C35) : sous « Recruter » (spec 042), les achats du site : `purchase.advanced_action` au monastère, une
  ligne par action avancée de l'offre d'unités, avec le texte de la carte en infobulle ; puis `purchase.healing`. Un achat trop
  cher reste visible, grisé.
- **Fenêtre Offres** (C36) : dans la colonne des unités, les actions avancées des monastères à la suite, avec `offers.monastery_card`.
- **Incendier** (C37) : bouton `burn.button` sous « Interagir avec les habitants », quand `BurnMonastery` est légale, avec
  `burn.lost_points` s'il reste des points ; confirmation modale (`burn.confirm`, comme l'annonce de fin de manche, spec 023),
  parce que le jeton tiré empêche le retour en arrière.
- **Combat d'incendie** (C38) : `burn.no_units` à la place des capacités d'unités sous la main.
- **Plateau** (C39) : un petit jeton Bouclier dessiné par nous (SVG, couleur du héros) sur la case ; infobulle `site.monastery.burned`
  au lieu du nom du monastère.
- **Récompense** (C40, C43) : `reward.pending` sous les points tant que la récompense attend ; à la fin du tour, une fenêtre comme
  le choix d'action avancée (spec 050) : `reward.choose_artifact`, les deux artefacts (nom, texte `Unsupported` de la spec 050),
  `reward.take` sous chacun.
- **Écran de fin** (C41) : `score.loot` et `score.conqueror` avec leurs nouveaux paramètres.
- **Pillage** (C42) : bouton `plunder.button` en tête des actions, au début du tour, sur un village.

Point purement visuel, laissé à la vérification à l'œil : le dessin du jeton Bouclier (forme, taille, lisibilité aux zooms).

```
┌─ Panneau latéral ─────────────────────────────────┐
│ Réputation : 0 · bonus d'interaction 0            │
│ Interaction : Monastère                           │
│ Mouvement : 0 · Influence : 8                     │
│ Recruter : Moines du Nord (7)              │
│ [Acheter : Bouclier de glace (6)]                 │
│ [Acheter un point de soin (2)]                    │
└───────────────────────────────────────────────────┘

┌─ Récompense : prenez un artefact ─────────────────┐
│  Bannière de gloire        │  Amulette du soleil  │
│  (pas encore codé : se     │  (pas encore codé :  │
│   joue de côté)            │   se joue de côté)   │
│        [Prendre]           │       [Prendre]      │
└───────────────────────────────────────────────────┘
```

Vérification à l'œil : une partie préparée `monastere` (bac à sable, Jour, campagnes 3, 4 et 5 explorées, figurine sur le
monastère de la campagne 5 ; une action avancée dans l'offre d'unités ; réputation 0 ; main : Promesse, Menace, Rage, Détermination,
une Blessure ; Paysans dans la zone des unités) : interagir, acheter un point de soin et l'action avancée ; ou, depuis le
chargement, incendier le monastère, gagner le combat, prendre un artefact, voir le Bouclier sur le plateau.

### Raccourcis clavier

| Touche | Effet | Contexte |
|---|---|---|
| 1, 2 | Prendre le 1er, le 2e artefact tiré | Choix d'artefact en attente |
| Échap | Annuler la confirmation de l'incendie | Confirmation de l'incendie ouverte |

Pas de touche pour incendier, acheter ni piller : l'incendie est sans retour, les achats changent d'un site à l'autre (comme le
recrutement, spec 042). Reportées dans `docs/raccourcis.md` à l'implémentation (1 et 2 servent déjà au choix d'action avancée,
dans un autre contexte).

## Effets sur les specs déjà implémentées

- **012, 015, 023** : la mise en place, l'exploration et la nouvelle manche ajoutent les actions avancées des monastères ; sans
  monastère (mise en place actuelle, tuiles sans monastère), rien ne change. Le bac à sable révèle un monastère avec la campagne 5 :
  les parcours qui l'explorent voient une carte de plus dans l'offre d'unités ; leurs points d'étape ne la vérifient pas.
- **031** : un combat peut être engagé par l'incendie, sans `StartCombat` ; l'ennemi non vaincu d'un incendie est défaussé.
- **040** : `interaction.nothing_to_buy` disparaît ; `Interact` refusée sur un monastère incendié.
- **042** : les points de soin achetés entrent dans la réserve ; unités exclues du combat d'incendie (ni activation ni dégâts).
- **050** : l'offre d'unités contient des actions avancées ; la nouvelle manche les rend à la pioche d'actions avancées ; le choix
  d'action avancée de fin de tour vient après celui de l'artefact.
- **063** : Le Collectionneur compte les artefacts, Le Conquérant les Boucliers des monastères.
- **024** : `Event::blocks_undo` gagne `ArtifactsDrawn` (oui) et les autres nouveaux événements (non) ; `EnemyTokenPlaced` l'est déjà.
- **Joueur de test `FirstChoice`** : il n'interagit pas ; il ne doit ni incendier ni piller, pour que les parcours ne changent pas
  (`BurnMonastery` et `PlunderVillage` sont écartés comme `Interact` et `Challenge`).

## Hors périmètre

- Interagir dans une forteresse possédée ou une tour de mage conquise (sorts à 7 influence et un mana) : spec 043. Cités : 062.
- Récompenses des autres sites (sorts, cristaux, unités, actions avancées des ruines) : specs 043 à 045, qui réutiliseront la
  récompense de fin de tour posée ici. Plusieurs artefacts d'un coup (n + 1 tirés) : avec la première source qui en donne
  plusieurs (ruines, spec 045).
- Effets des artefacts et bannières : spec 055.
- Pillage pendant le tour d'un autre joueur, joueur fantôme : spec 061 et multijoueur.
- Unités qui ne suivent pas le héros dans un donjon ou un tombeau : spec 044, qui réutilisera l'exclusion des unités.
- Variantes et scénarios qui touchent les monastères (*Dungeon Lords*, tombeaux secrets) : hors périmètre.

## Plan de développement

Gros morceau : achats, incendie, récompense, décompte, et un écran à chaque fois. Suivant l'issue privée 118, l'implémentation se fait en
**deux fils** sur la même branche, chacun avec son plan d'étapes (`plans/041-villages-et-monasteres.md`, rédigé par le premier) :

1. **Moteur** : C1 à C33, contenu, persistance (sauvegarde, partie préparée `monastere`).
2. **Écran** : libellés, C34 à C43, vérification à l'œil, docs (`architecture.md`, `raccourcis.md`, `index-par-notion.md`).

## Décisions (2026-11-04)

Questions de la rédaction, tranchées par Mathieu (« Q3 : "gardez un artefact" remplacé par "prenez un artefact", sinon le reste
est validé ») :

1. **Q1** — Le pillage d'un village est codé par cette spec : une décision offerte au début du tour, avant toute autre action, sur
   un village, une fois par tour (Compendium, *Turn Start* S1, *Miscellaneous* S5), ce qui revient sur le hors périmètre de la
   spec 040 (C32, C33, C42).
2. **Q2** — Une seule spec, implémentée en deux fils sur la même branche : moteur, puis écran (issue privée 118).
3. **Q3** — Termes : « Incendier un monastère », « monastère incendié », « Piller », « Acheter un point de soin », « Acheter :
   <action avancée> », et « Récompense : prenez un artefact » (bouton « Prendre », action `TakeArtifact`) au lieu de « gardez ».

4. **Pluriel** (2026-11-04, à l'implémentation) : un nombre suivi d'un mot au pluriel en « s » s'écrit avec « (s) » :
   « 1 artefact(s) », « 1 Bouclier(s) », et « Blessure(s) » dans la ligne de La Victime (spec 063), arcanum-i18n n'accordant pas.

## Questions ouvertes

Aucune.
