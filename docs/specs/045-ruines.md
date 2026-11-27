# 045 — Ruines

- Statut : implémentée le 2026-11-05 (validée le même jour : propositions de Q1 à Q7 retenues, Q6 précisée par Mathieu)
- Jalon : J4
- Dépend de : 015 (exploration), 021 (cristaux), 022 et 025 (payer du mana), 023 (nouvelle manche), 024 (retour en arrière),
  030 à 034 (jetons d'ennemis, combat), 035 (gloire, niveaux), 036 (trajets), 041 (Bouclier, récompense artefact), 042 (unités,
  renvoi), 043 (drapeau, récompense sort, révélation des défenseurs), 044 (sites d'aventure, entrer, jetons restés, récompenses de
  fin de tour), 050 (offres, action avancée) ; toutes implémentées
- Livret : MKUE p. 3 (*Enemy and Ruin token piles*) ; p. 6 (6.c, 6.e) ; p. 7 (4.d) ; p. 8 (*Combat with Enemies* 1.b) ; p. 10
  (*Combat Outcomes* 2.b, 3.c) ; p. 11 (*End of the Turn* 5.a à 5.d, 6) ; p. 15 (*The Greatest Adventurer*). Carte de site
  *Ancient Ruins* (`docs/regles/cartes/sites-et-cites.md`) et jetons de ruines (`docs/regles/cartes/ennemis.md`, *Jetons de
  ruines*) ; termes français : carte de site FR « Ruines » (`docs/regles/scans/fr/rule_cards/`). FAQ p3 (ruine révélée en s'y
  déplaçant, ennemis restés, pas de réputation). Compendium, *Mana* S4, S12 ; *Movement* S8, S11 ; *Combat* S18, S39 ; *Influence*
  S8 ; *End of Round* S6 ; *Miscellaneous* S6, S7, S11, S14, S16 ; *Silver Units* Q3
- Crates touchées : arcanum-core (règles, contenu), arcanum-ui, arcanum-persistance (parties préparées), `assets/i18n/`

## Objectif

Les ruines cessent d'être des cases vides. Chaque ruine révélée reçoit un **jeton de ruines** (jaune), face visible le jour, face
cachée la nuit jusqu'au prochain Jour ou jusqu'à ce que la figurine y entre. Debout sur la ruine, le héros y trouve soit un
**autel** : il paie en **offrande** 3 mana de la couleur du jeton et gagne 7 gloire ; soit des **ennemis** tirés selon le jeton, à
combattre (avec ses unités), qui restent sur place s'ils ne sont pas tous vaincus. Une ruine conquise est marquée de son drapeau (jeton Bouclier du livret, `GameState::shields`)
et donne la **récompense du jeton** en fin de tour : artefacts, sort, action avancée, un cristal de chaque couleur, ou une
**unité** de l'offre, gratuite.

## Règles

### Jetons de ruines

- **Pile** (MKUE p. 3, *Enemy and Ruin token piles*) : les 12 jetons de ruines du jeu de base (4 autels, 8 jetons d'ennemis,
  `cartes/ennemis.md`) forment une pile face cachée, mélangée à la mise en place avec son propre générateur (ADR 0007). Les
  ruines des extensions restent hors périmètre.
- **Défausse** (Compendium, *Miscellaneous* S6) : un jeton dont la ruine est conquise (offrande payée, ou dernier ennemi vaincu) part dans la défausse
  des ruines. Le remélange d'une pile vide (MKUE p. 3) ne peut pas arriver : 7 ruines sur toutes les tuiles pour 12 jetons (un test
  de contenu le garde, pas de code de remélange).
- **Pose** (carte *Ancient Ruins*, *When Revealed*) : quand une tuile qui porte des ruines est posée (exploration, mise en place),
  le jeton du dessus de la pile va sur la case, **face visible de jour, face cachée de nuit**.
- **Révélation** d'un jeton face cachée (carte ; Compendium, *Movement* S8, *End of Round* S6) : au début de chaque manche de
  **Jour**, avec les défenseurs (spec 043) ; ou quand la figurine **entre sur la case** par un pas (`MoveHero`, `FollowRoute`), même
  pour la traverser. La révélation ne tire pas d'ennemis et n'oblige à rien (FAQ p3) : la ruine reste un site d'aventure, une case
  sécurisée, qu'on peut ignorer.
- **Trajet qui révèle** (Q2 ; Compendium, *Movement* S13, comme la spec 043, C13b) : un trajet suivi s'arrête sur le pas qui
  retourne un jeton de ruines, figurine sur la ruine ; le reste du trajet n'est pas joué, les points restent.

### Autel

- **L'offrande** (carte, *Altar* ; Compendium, *Movement* S11 (d)) : debout sur une ruine non conquise dont le jeton est un autel,
  `PayAltarTribute` est l'**action du tour**, aux mêmes conditions qu'`EnterAdventureSite` (spec 044 : `turn_action_taken`, case X
  permise, dernier tour permis). Elle termine le déplacement (mouvement et influence perdus), sans combat.
- **Paiement** (carte ; Compendium, *Mana* S4 et S12) : exactement 3 mana de la couleur du jeton, par les moyens ordinaires : dés
  de la Source dans la limite du tour (spec 025), cristaux, jetons de mana ; le mana **or** paie de jour, pas de nuit ; le noir ne
  paie pas. Pas de paiement partiel : sans 3 mana, pas d'offrande.
- **Effet** : la ruine est conquise (Bouclier du joueur, drapeau, jeton défaussé) et le joueur gagne **7 gloire** aussitôt (Q3) ;
  le changement de niveau éventuel se fait en fin de tour comme toujours (MKUE p. 11, 6).

### Ennemis des ruines

- **Entrer** (carte, *Enemies* ; MKUE p. 8, 1.b) : debout sur une ruine non conquise dont le jeton montre des ennemis,
  `EnterAdventureSite` (spec 044) est permise, mêmes conditions, combat aussitôt.
- **Jetons combattus** (carte ; MKUE p. 10, 3.c ; Compendium, *Combat* S39) : à la **première** entrée, un jeton d'ennemi tiré
  par couleur montrée sur le jeton de ruines (deux pour « vert + vert »), comme les autres tirages (remélange, pile de secours de
  la spec 033). Ensuite, seulement les jetons **restés** sur la case : un ennemi vaincu n'est **pas** remplacé (différence avec le
  Sombre Terrier). Une ruine non conquise sans jeton d'ennemi sur la case n'a jamais été visitée : pas d'état de plus.
- **Combat ordinaire** (Compendium, *Miscellaneous* S16) : règles de la manche, unités permises, ennemis non fortifiés, **pas de
  réputation** (FAQ p3 : Orques et Draconiens des ruines ne sont pas des ennemis errants). La gloire de chaque ennemi vaincu se
  gagne comme d'habitude.
- **Issue** (MKUE p. 10, 2.b et 3.c ; Compendium, *Combat* S18) : tous les jetons du site vaincus, la ruine est conquise (Bouclier,
  drapeau, jeton de ruines défaussé) et la **récompense du jeton** attend la fin du tour. Sinon, les jetons non vaincus restent face
  visible sur la case, le héros y reste. Les jetons restés ne sont pas des ennemis errants (spec 044, C17).
- Plus aucun jeton d'ennemi nulle part à la première entrée (théorique) : la ruine est conquise sans combat, avec sa récompense
  (comme la spec 044, C20).

### Récompenses du jeton, en fin de tour

Ordre fixe (MKUE p. 11, 5 laisse l'ordre au joueur ; en solo l'ordre fixe ne change rien, comme la spec 044) :

1. **Cristaux** (MKUE p. 11, 5.a) : « un cristal de chaque couleur » donne un cristal rouge, bleu, blanc et vert, sauf dans une
   couleur déjà à 3 (rien à la place) ; pas de dé. Avec les cristaux aléatoires de la spec 044, ces cristaux-ci d'abord.
2. **Artefacts** (MKUE p. 11, 5.b) : pour *n* artefacts, *n* + 1 tirés ; le joueur en garde *n*, un à un (`TakeArtifact`), le
   dernier retourne sous la pioche. C'est la règle du livret ; la spec 041 l'appliquait déjà pour *n* = 1. Pioche courte : tous
   ceux qu'elle a, gardés jusqu'à *n* ; vide, rien.
3. **Sort** (MKUE p. 11, 5.c) : de l'offre de sorts, comme la spec 043.
4. **Action avancée** (MKUE p. 11, 5.c) : de l'offre d'actions avancées, pas une action avancée de monastère de l'offre d'unités ;
   même choix que celui d'un niveau pair (`GainAdvancedAction`, spec 050), qu'il précède s'il y a les deux. Offre vide : rien.
5. Changement de niveau (spec 035) et action avancée d'un niveau pair (spec 050).
6. **Unité** (MKUE p. 11, 5.d ; Compendium, *Miscellaneous* S11, *Silver Units* Q3) : une unité **de l'offre d'unités**, au choix,
   gratuite, quel que soit son type ou son lieu de recrutement ; l'offre n'est pas complétée (Compendium, *Miscellaneous* S7). Ce
   choix vient **après** le changement de niveau (Q4) : un jeton de commandement gagné ce tour sert donc déjà (exception de 5.d).
   Sans jeton libre, le joueur renvoie une de ses unités (comme le recrutement, spec 042) ou **renonce** à la récompense. Pas
   d'unité dans l'offre : rien.

Un tour ne compte qu'un combat ou une offrande : au plus une ruine conquise par tour.

### Décompte

- **L'Aventurier** (MKUE p. 15) compte déjà les Boucliers des ruines (spec 044, `Site::is_adventure`) : rien à coder, un cas le
  vérifie.

### Exemple : une ruine de nuit

Bac à sable, manche 2 (Nuit). La campagne 8 est explorée en 2A : son jeton de ruines est posé face cachée en (7, −5). Au tour
suivant, la figurine y entre par un pas.

```
Pas sur (7, −5)              → HeroMoved ; RuinTokenRevealed (gris + violet, une unité) — point de non-retour
Entrer dans les ruines       → AdventureSiteEntered ; EnemyTokenPlaced (gris), EnemyTokenPlaced (violet) ; CombatStarted
  le jeton gris vaincu, le violet non
Fin du combat                → CombatEnded (le violet reste face visible sur la case)
Tour suivant : entrer        → AdventureSiteEntered ; CombatStarted (le violet seul, aucun tirage)
  vaincu                     → CombatEnded ; SiteConquered (Bouclier, jeton défaussé), récompense : une unité
Fin du tour                  → … niveaux ; choix d'une unité de l'offre (jetons de commandement pris : renvoyer ou renoncer)
```

Variante autel, de jour : jeton « 3 mana bleu ». Figurine sur la ruine avec un cristal bleu, un jeton de mana bleu et un dé bleu
dans la Source : `PayAltarTribute` (dé, cristal, jeton) → `AltarTributePaid`, `DieTaken`, `CrystalUsed`, `ManaTokenUsed`,
`SiteConquered`, `FameGained { fame: 7 }`.

## Modèle

Termes du glossaire : Ruines, Site d'aventure, Entrer dans, Jeton d'ennemi, Couleur d'ennemi, Face visible / face cachée, Jeton
Bouclier, Récompense, Artefact, Sort, Action avancée, Unité, Offre d'unités, Renvoyer, Cristal, Dé, Jeton de mana, L'Aventurier.
**Nouveaux termes** (ajoutés au glossaire avec cette spec ; français de la carte de site FR) :

| Livret | Français | Code |
|---|---|---|
| Ruins token (yellow token) | Jeton de ruines (carte FR : « jeton jaune ») | `RuinToken`, `RuinTokenDefinition` (`content/ruins.ron`) |
| Altar | Autel | `RuinToken::is_altar`, `RuinKind::Altar` |
| Pay … as tribute | Offrande (carte FR : « En offrande, vous pouvez payer ») ; « faire une offrande » | `Action::PayAltarTribute`, `AltarTributePaid` |
| (Ruins) reward | Récompense du jeton | `RuinReward` (artefact, sort, action avancée, un cristal de chaque couleur, unité) |
| Unit reward | Récompense : une unité | `PlayerState::unit_rewards`, `Action::TakeUnitReward`, `DeclineUnitReward` |

Données nouvelles :

- **Contenu** : `content/ruins.ron`, un jeton par entrée (12, un exemplaire chacun) : un autel et sa couleur, ou des couleurs de
  piles d'ennemis et une liste de récompenses. Identifiants `RuinToken` en Rust (ADR 0005).
- **État** : la pile et la défausse des jetons de ruines ; le jeton de ruines de chaque ruine non conquise (`GameState::ruins`,
  par case) et s'il est face cachée ; les jetons d'ennemis restés sur une ruine sont dans la liste de jetons de la case (spec 044).
- **Récompenses en attente** (`PlayerState`) : `artifact_rewards` (existe, peut valoir 2), `spell_rewards` (existe), et nouveaux
  `crystal_set_rewards`, `advanced_action_rewards`, `unit_rewards` ; le choix d'unité en cours.
- **Générateur** : un usage de plus, `RngUsage::RuinPile`, numéro de flux libre (4).

## Actions et événements

| Action | Validation (refusée si…) | Événements produits |
|---|---|---|
| `MoveHero`, `FollowRoute` (014, 036, étendues) | inchangée | après le `HeroMoved` d'un pas sur une ruine face cachée, `RuinTokenRevealed { hex, token }` ; un trajet s'arrête sur ce pas |
| Explorer (015, étendue) ; mise en place | inchangée | pour chaque ruine de la tuile, `RuinTokenPlaced { token, hex, face_up }` (face visible de jour) |
| `AnnounceEndOfRound` (023, nouvelle manche) | inchangée | manche de Jour : un `RuinTokenRevealed` par jeton face cachée, avec les défenseurs (spec 043) |
| `PayAltarTribute { mana }` (nouvelle) | figurine hors d'une ruine non conquise à jeton d'autel ; action du tour déjà prise ou impossible (`turn_action_taken`) ; `mana` n'est pas 3 paiements de la couleur de l'autel (dé absent, épuisé ou au-delà de la limite, cristal ou jeton absent, or de nuit) | `AltarTributePaid { player, hex }`, les événements du mana (`DieTaken`, `CrystalUsed`, `ManaTokenUsed`), `SiteConquered { player, hex }`, `FameGained { fame: 7 }` |
| `EnterAdventureSite` (044, étendue) | aussi sur une ruine non conquise à jeton d'ennemis | `AdventureSiteEntered` ; à la première entrée, un `EnemyTokenPlaced` par couleur du jeton de ruines (avec les `EnemyPileReshuffled` au besoin) ; `CombatStarted` (jetons tirés ou restés) |
| `EndCombat` (031, 044, étendue) | inchangée | `CombatEnded`, puis sur une ruine, tous ses jetons vaincus : `SiteConquered` ; sinon rien (les jetons restent) |
| `EndTurn` (016, 044, étendue) | inchangée | … puis `CrystalGained` par couleur non pleine d'« un cristal de chaque couleur » (avant les dés des cristaux aléatoires) ; `ArtifactsDrawn` (*n* + 1) ; sort ; action avancée de la récompense ; niveaux ; action avancée du niveau ; puis le choix d'unité attend |
| `TakeArtifact` (041, étendue) | inchangée | `ArtifactTaken` ; quand *n* sont gardés, `ArtifactReturned` pour le dernier |
| `GainAdvancedAction` (050, étendue) | inchangée ; proposée aussi pour une récompense d'action avancée | inchangés |
| `TakeUnitReward { unit, disband }` (nouvelle) | pas de récompense unité en attente ; `unit` absente de l'offre d'unités (ou action avancée de monastère) ; `disband` donné avec un jeton libre, absent sans jeton libre, ou unité qui n'est pas au joueur | `UnitDisbanded` au besoin, `UnitRewardTaken { player, unit }`, puis la suite de la fin du tour |
| `DeclineUnitReward` (nouvelle) | pas de récompense unité en attente ; un jeton de commandement libre | `UnitRewardDeclined { player }`, puis la suite |

Ordre des actions légales (recette de `architecture.md`) : choix de fin de tour : `ChooseCrystalColor`, `TakeArtifact`,
`TakeSpell`, `GainAdvancedAction`, puis `TakeUnitReward` (ordre de l'offre, sans renvoi puis avec chaque renvoi) et
`DeclineUnitReward` ; hors combat, `PayAltarTribute` à la place d'`EnterAdventureSite` (une ruine a l'un ou l'autre), paiements
dans l'ordre de `mana_payments` : dé, puis cristaux, puis jetons, sans doublon.

| Événement | Effet de `apply` | Révèle une information ? | Point de non-retour ? |
|---|---|---|---|
| `RuinPileShuffled { order }` (nouveau, mise en place) | la pile des jetons de ruines | non (l'ordre est caché dans la vue) | non (mise en place) |
| `RuinTokenPlaced { token, hex, face_up }` (nouveau) | le jeton quitte la pile pour la case, face visible ou cachée | **oui** si face visible | oui (avec l'exploration, déjà point de non-retour) |
| `RuinTokenRevealed { hex, token }` (nouveau) | le jeton de la case est face visible | **oui** | **oui** |
| `AltarTributePaid { player, hex }` (nouveau) | mouvement et influence à 0, déplacement terminé, action du tour prise | non | non |
| `SiteConquered { player, hex }` (043, 044, étendu) | sur une ruine : Bouclier, jeton de ruines défaussé, récompenses du jeton ajoutées aux récompenses en attente (rien pour un autel) | non | non |
| `UnitRewardTaken { player, unit }` (nouveau) | l'unité quitte l'offre d'unités pour la zone du joueur, disponible ; `unit_rewards` − 1 | non | non |
| `UnitRewardDeclined { player }` (nouveau) | `unit_rewards` − 1 | non | non |

## Comportements attendus

Chaque cas devient au moins un test (nom indicatif entre parenthèses). C1 à C30 : moteur (arcanum-core, `tests/ruines.rs`) ; C31 à C41 :
écran (arcanum-ui, sur `game_screen`), un cas par point de la section « Interface » (issue privée 119).

Jetons de ruines :

- **C1** — Le contenu a 12 jetons de ruines (4 autels : vert, bleu, blanc, rouge ; 8 jetons d'ennemis avec les couleurs et
  récompenses de `cartes/ennemis.md`), et au moins autant que de ruines sur toutes les tuiles (`ruin_tokens_match_the_content`) —
  MKUE p. 3.
- **C2** — Mise en place : `RuinPileShuffled` avec les 12 jetons ; deux graines donnent deux ordres ; les autres tirages ne
  bougent pas (`ruin_pile_is_shuffled_at_setup`) — ADR 0007.
- **C3** — De jour, explorer la campagne 8 pose le jeton du dessus face visible sur sa ruine (`ruin_token_placed_face_up_by_day`)
  — carte *Ancient Ruins*.
- **C4** — De nuit, le même jeton est posé face cachée ; la vue ne montre qu'une ruine face cachée, sans le jeton
  (`ruin_token_placed_face_down_at_night`) — carte.
- **C5** — Nuit, jeton face cachée : un pas sur la ruine produit `RuinTokenRevealed` après `HeroMoved` ; un pas sur une case voisine
  non (`stepping_on_ruins_reveals_the_token`) — Compendium, *Movement* S8.
- **C6** — Nuit : un trajet qui passe par la ruine face cachée s'arrête sur elle, points restants gardés
  (`route_stops_on_revealed_ruins`) — Compendium, *Movement* S13 ; Q2.
- **C7** — Nouvelle manche de Jour : chaque jeton face cachée est révélé ; nouvelle manche de Nuit : rien
  (`day_round_reveals_ruin_tokens`) — Compendium, *End of Round* S6.
- **C8** — Le retour en arrière ne passe pas avant un `RuinTokenRevealed` (`ruin_reveal_blocks_undo`) — spec 024.
- **C9** — Une ruine révélée n'arrête pas le déplacement et n'oblige à rien : le pas suivant reste légal, et finir le tour sur la
  ruine aussi (`ruins_can_be_ignored`) — FAQ p3 ; MKUE p. 7, 4.d.

Autel :

- **C10** — Sur un autel bleu, de jour, avec un dé bleu dans la Source, un cristal bleu et un jeton bleu : `PayAltarTribute` est
  légale ; jouée, `AltarTributePaid`, les trois paiements, `SiteConquered`, `FameGained { fame: 7 }` ; Bouclier, jeton défaussé,
  mouvement et influence à 0 (`altar_tribute_gives_seven_fame`) — carte, *Altar*.
- **C11** — Deux mana bleus seulement : pas de `PayAltarTribute` (`altar_needs_three_mana`) — carte.
- **C12** — De jour, un dé or et deux cristaux bleus paient ; de nuit, le même dé or non, un dé noir jamais
  (`altar_gold_pays_by_day_only`) — Compendium, *Mana* S4.
- **C13** — Deux dés bleus dans la Source sans dé de plus : un seul sert ; les paiements proposés n'ont jamais deux dés
  (`altar_respects_the_dice_limit`) — spec 025.
- **C14** — Après une interaction, un combat ou une provocation ce tour : pas de `PayAltarTribute` ; après l'offrande : ni pas, ni
  exploration, ni interaction, ni entrée (`altar_is_the_action_of_the_turn`) — Compendium, *Movement* S11.
- **C15** — Autel conquis : plus de `PayAltarTribute` ni d'`EnterAdventureSite` sur la case ; le drapeau du joueur la marque
  (`fulfilled_altar_is_an_ordinary_space`) — Compendium, *Miscellaneous* S6.
- **C16** — 7 gloire qui franchissent un seuil : `LevelGained` en fin de tour, pas avant (`altar_fame_levels_up_at_end_of_turn`) —
  MKUE p. 11, 6.

Ennemis des ruines :

- **C17** — Ruine « marron + rouge » non visitée : `EnterAdventureSite` tire un jeton marron et un rouge, combat contre les deux,
  unités permises ; « vert + vert » tire deux verts (`entering_ruins_draws_the_token_enemies`) — carte, *Enemies*.
- **C18** — Un des deux vaincu : l'autre reste face visible sur la case ; entrer à nouveau combat celui-là **seul**, sans tirage
  (`ruins_enemies_are_not_replaced`) — MKUE p. 10, 3.c.
- **C19** — Tous vaincus : `SiteConquered` après `CombatEnded`, Bouclier, jeton de ruines défaussé, récompense en attente ; aucune
  `ReputationChanged` pour des Orques vaincus aux ruines (`ruins_conquest_and_no_reputation`) — MKUE p. 10, 2.b ; FAQ p3.
- **C20** — De jour, combat de ruines : le jeton or paie (pas de règles de Nuit) ; les ennemis ne sont pas fortifiés : l'attaque à
  distance les vise (`ruins_combat_uses_round_rules`) — Compendium, *Miscellaneous* S16.
- **C21** — Ruine non conquise avec un jeton resté : ce jeton n'est pas un ennemi errant (ni provoqué, ni défié, ni évité par les
  trajets) (`ruins_tokens_are_not_rampaging`) — spec 044, C17.

Récompenses de fin de tour :

- **C22** — Récompense « sort + un cristal de chaque couleur », inventaire déjà à 3 en rouge : `CrystalGained` bleu, blanc, vert
  (pas rouge), puis le choix du sort (`crystal_set_reward`) — MKUE p. 11, 5.a.
- **C23** — Récompense « artefact + artefact » : trois artefacts tirés ; deux `TakeArtifact` successifs, puis `ArtifactReturned`
  pour le troisième (`two_artifacts_draw_three`) — MKUE p. 11, 5.b.
- **C24** — Récompense « artefact + action avancée » : après le choix de l'artefact, `GainAdvancedAction` pour chaque carte de
  l'offre d'actions avancées, pas pour celles de monastère ; avec aussi un niveau pair ce tour, deux choix successifs
  (`advanced_action_reward`) — MKUE p. 11, 5.c.
- **C25** — Récompense unité, un jeton de commandement libre : `TakeUnitReward` pour chaque unité de l'offre (jamais une action
  avancée de monastère), sans `DeclineUnitReward` ; prise, l'unité est dans la zone, disponible, et l'offre n'est pas complétée
  (`unit_reward_from_the_offer`) — MKUE p. 11, 5.d ; Compendium, *Miscellaneous* S7.
- **C26** — Récompense unité sans jeton libre : `TakeUnitReward` avec chaque renvoi possible, et `DeclineUnitReward` ; si le niveau
  gagné ce tour donne un jeton de commandement, le choix vient après `LevelGained` et se fait sans renvoi
  (`unit_reward_after_level_up`) — MKUE p. 11, 5.d, exception.
- **C27** — Offre d'unités sans unité : la récompense est perdue, la fin du tour continue (`unit_reward_lost_without_units`) —
  Compendium, *Miscellaneous* S11.

Décompte, sauvegarde, vue, joueur de test :

- **C28** — Fin de partie avec un autel conquis : L'Aventurier 2 (`adventurer_counts_ruins`) — MKUE p. 15.
- **C29** — Une partie sauvegardée avec un jeton face cachée, un autel face visible, une ruine avec un jeton resté et une récompense
  unité en attente se recharge à l'identique ; la vue donne les ruines (jeton ou face cachée), les jetons restés et les
  récompenses en attente (`ruins_survive_save_and_load`, `view_shows_ruins`) — spec 024.
- **C30** — Joueur de test `FirstChoice` : ni offrande ni entrée dans une ruine ; il prend la première unité de récompense
  (`first_choice_ignores_ruins`) — les parcours ne changent pas (risques du plan).

Écran (arcanum-ui, sur `game_screen`) :

- **C31** — Plateau : une ruine face visible montre son jeton (autel : trois pastilles de la couleur ; ennemis : une pastille par
  couleur de pile) ; face cachée, le dos jaune ; les jetons d'ennemis restés à côté, comme au Sombre Terrier
  (`board_draws_ruin_tokens`, `PlayerView::ruins`).
- **C32** — Infobulle d'une ruine : autel (« offrande de 3 mana bleu : 7 Gloire »), ennemis et récompense, ennemis restés, ou
  « jeton face cachée » (`hover_describes_ruins`, `PlayerView::ruins`, `enemy_tokens`).
- **C33** — Panneau, sur un autel : bouton « Faire une offrande à l'autel » et les points perdus ; grisé avec « Il faut 3 mana
  bleu » quand aucun paiement n'est légal ; absent ailleurs (`panel_offers_altar_tribute`).
- **C33 bis** (issue privée 141) — Le bouton grisé n'apparaît que si l'action du tour est encore possible
  (`PlayerView::turn_action_taken` faux, le même test que le moteur) : avec une main de Blessures sans carte jouée
  (Compendium, *Resting* S4), même avec 3 cristaux bleus, pas de bouton, ni « Il faut 3 mana bleu »
  (`no_altar_button_once_turn_action_unavailable`, `view_tells_turn_action_taken`).
- **C34** — Le bouton (ou E) ouvre la fenêtre de l'offrande : une ligne par paiement légal (« dé bleu, cristal bleu, jeton
  bleu »), un clic la joue ; touches 1 à 9 ; « Annuler » ou Échap ferme (`altar_payment_window`).
- **C35** — Sur une ruine à ennemis : le bouton « Entrer dans : Ruines » de la spec 044 ; la confirmation donne les couleurs à
  tirer (ou les ennemis restés) et la récompense du jeton (`ruins_enter_confirmation`).
- **C36** — Ruine conquise : le drapeau du héros (`fulfilled_ruins_show_a_flag`, `PlayerView::shields`).
- **C37** — Panneau : récompenses en attente du jeton (« Récompense en fin de tour : 2 artefacts, une action avancée… »)
  (`panel_announces_ruin_rewards`, `PlayerView::artifact_rewards`, `advanced_action_rewards`, `crystal_set_rewards`,
  `unit_rewards`).
- **C38** — Choix de deux artefacts parmi trois : la fenêtre du choix (spec 041) montre trois artefacts, garde les deux pris, et
  « Prendre » reste jusqu'au second (`artifact_window_keeps_two`, `PlayerView::drawn_artifacts`, `artifact_rewards`).
- **C39** — Récompense unité : la fenêtre Offres s'ouvre comme pour l'action avancée (spec 050), un bouton « Prendre » par unité de
  l'offre, touches 1 à 3 ; sans jeton libre, « Prendre » ouvre le choix de l'unité à renvoyer (spec 042) et « Renoncer » est
  proposé (`unit_reward_window`, `PlayerView::unit_rewards`, `command_tokens`, `units`). Depuis la [spec #20](issue-20-obtention-des-cartes.md) : une fenêtre de
  choix des seules unités de l'offre, « Renoncer » sous les cartes, au lieu de la fenêtre Offres.
- **C40** — Nuit : le message d'un trajet arrêté par une ruine révélée (« Ruines révélées : le trajet s'arrête ») sous les points,
  jusqu'à l'action suivante (`panel_announces_revealed_ruins`, `PlayerView::revealed_ruins`).
- **C41** — Récompense action avancée (issue privée 140) : la fenêtre Offres du choix titre « Récompense : prenez une action
  avancée » ; avec un niveau pair le même tour, le choix de la récompense vient d'abord (C24) et garde ce titre, puis celui du
  niveau reprend « Niveau 2 : choisissez une action avancée » (`advanced_action_reward_window`,
  `PlayerView::advanced_action_choices`, `levels_gained`). Depuis la [spec #20](issue-20-obtention-des-cartes.md) : une fenêtre de choix des seules actions
  avancées de l'offre ; celle du niveau est la fenêtre du niveau pair.

## Cas limites et refus

- **L1** — Héros assommé dans une ruine : le combat continue (spec 034) ; vaincre tous les jetons conquiert la ruine.
- **L2** — Invocateur (marron) dans une ruine : la créature invoquée n'est pas un jeton du site (spec 044, L2).
- **L3** — Autel payé avec un jeton de mana or gagné à la clairière, de jour : permis (mana or, carte *Mana* S4).
- **L4** — Pioche d'artefacts avec un seul artefact pour deux récompenses : il est pris sans choix, la seconde est perdue ; offre
  de sorts ou d'actions avancées vide : perdue (specs 041, 043, 050).
- **L5** — Nouvelle manche de Jour avec la figurine sur une ruine face cachée : révélée comme les autres, rien d'autre.
- **L6** — Ruine révélée au dernier tour de la partie (spec 063) : l'offrande et l'entrée restent permises, les récompenses sont
  prises avant le décompte.
- **L7** — Unité de récompense prise : elle peut servir dès le tour suivant ; elle n'est pas « recrutée » (pas d'influence, pas de
  réputation, pas d'interaction).
- **L8** — Renvoyer une unité blessée pour la récompense : permis (spec 042).

## Information cachée

L'ordre de la pile des jetons de ruines est caché (la vue donne le nombre de jetons). Un jeton face cachée n'est vu que comme
« ruine face cachée » : `PlayerView::ruins` donne, par case de ruine non conquise, le jeton face visible ou rien. Un jeton
révélé, les ennemis tirés et les récompenses sont publics.

Champs de la vue ajoutés : `ruins` (case → jeton ou face cachée), `ruin_pile` (nombre), `crystal_set_rewards`,
`advanced_action_rewards`, `unit_rewards`, `revealed_ruins` (jetons retournés par le dernier pas ou trajet, pour le message de
C40). Existants réutilisés : `enemy_tokens`, `shields`, `artifact_rewards`, `drawn_artifacts`, `spell_rewards`,
`advanced_action_choices`, `offers`, `units`, `command_tokens`. Ajouté par l'issue privée 141 : `turn_action_taken`, public (l'action
du tour n'est plus possible, ou pas encore : le test du moteur pour interagir, entrer, faire une offrande).

## Libellés (i18n)

| Clé | Paramètres | fr | en |
|---|---|---|---|
| `ruins.altar.tribute` | — | Faire une offrande à l'autel | Pay tribute at the altar |
| `ruins.altar.needs` | `color` | Il faut 3 mana {color} | You need 3 {color} mana |
| `ruins.altar.window` | `color` | Offrande : 3 mana {color}, 7 Gloire | Tribute: 3 {color} mana, 7 Fame |
| `ruins.altar.no_undo` | — | La gloire est gagnée aussitôt. | Fame is gained at once. |
| `ruins.payment.die` | `color` | dé {color} | {color} die |
| `ruins.payment.crystal` | `color` | cristal {color} | {color} crystal |
| `ruins.payment.token` | `color` | jeton {color} | {color} token |
| `ruins.reward.artifact` | — | un artefact | an Artifact |
| `ruins.reward.spell` | — | un sort | a Spell |
| `ruins.reward.advanced_action` | — | une action avancée | an Advanced Action |
| `ruins.reward.crystal_set` | — | un cristal de chaque couleur | one crystal of each color |
| `ruins.reward.unit` | — | une unité de l'offre | a Unit from the offer |
| `adventure.confirm.fight_colors` | `colors` | Combat contre des ennemis tirés au hasard : {colors}. | Fight enemies drawn at random: {colors}. |
| `board.hover.ruins.altar` | `color` | Autel : offrande de 3 mana {color} pour 7 Gloire | Altar: tribute of 3 {color} mana for 7 Fame |
| `board.hover.ruins.enemies` | `colors`, `reward` | Ennemis : {colors} ; récompense : {reward} | Enemies: {colors}; reward: {reward} |
| `board.hover.ruins.face_down` | — | Jeton de ruines face cachée (révélé au prochain Jour, ou en y entrant) | Face-down ruins token (revealed next Day, or when you enter) |
| `reward.pending_ruins` | `list` | Récompense en fin de tour : {list} | Reward at end of turn: {list} |
| `reward.artifacts_count` | `count` | {count} artefact(s) | {count} Artifact(s) |
| `reward.choose_unit` | — | Récompense : prenez une unité de l'offre | Reward: take a Unit from the offer |
| `reward.decline_unit` | — | Renoncer | Decline |
| `reward.choose_advanced_action` | — | Récompense : prenez une action avancée | Reward: take an Advanced Action |
| `ruins.revealed` | — | Ruines révélées : le trajet s'arrête | Ruins revealed: the route stops |
| `site.conquered.ruins` | — | Ruines conquises | Conquered ruins |

Vérifiés possibles (issue privée 129) : aucune clé fille sous un libellé existant (`ruins` est un groupe nouveau ; `board.hover.ruins`
à côté de `board.hover.adventure` et `board.hover.glade` ; `reward.pending_ruins`, `reward.artifacts_count`,
`reward.choose_unit`, `reward.decline_unit`, `reward.choose_advanced_action` à côté des clés `reward.*` existantes, qui sont des libellés : pas de clé fille) ;
aucun caractère hors de la police (pas de flèche : « pour 7 Gloire »).

**Clés existantes réutilisées** (vérifiées dans `fr.json` et `en.json`, issue privée 127) : `site.ruins.name` ; `adventure.enter`,
`adventure.lost_points`, `adventure.confirm.fight_waiting`, `adventure.confirm.reward`, `adventure.confirm.no_undo`,
`adventure.confirm_yes`, `adventure.cancel` ; `board.hover.adventure.enemies_waiting` ; `reward.take`, `reward.choose_artifact`, `reward.choose_spell` ; `mana.red`, `mana.blue`, `mana.white`, `mana.green`,
`mana.gold` ; `enemy.pile_color.*`.

## Interface

- **Jeton de ruines sur le plateau** (C31, `PlayerView::ruins`, `enemy_tokens`) : un hexagone jaune dessiné par le code, à côté du
  site comme un défenseur (spec 043) ; face visible : trois pastilles de la couleur de l'autel, ou une pastille par couleur de pile
  d'ennemis ; face cachée : jaune uni. Les jetons d'ennemis restés comme au Sombre Terrier (spec 044). Dessin et lisibilité aux
  zooms : vérification à l'œil (rien n'est repris des scans).
- **Infobulle** (C32, `PlayerView::ruins`, `enemy_tokens`) : nom du site ou `site.conquered.ruins`, puis
  `board.hover.ruins.altar`, `board.hover.ruins.enemies` (récompense par `ruins.reward.*`, séparées par des virgules),
  `board.hover.adventure.enemies_waiting`, ou `board.hover.ruins.face_down`.
- **Offrande** (C33, C33 bis, C34 ; Q1 ; `PlayerView::ruins`, `turn_action_taken`, actions légales) : bouton `ruins.altar.tribute` dans le panneau, comme
  « Entrer dans » (spec 044), `adventure.lost_points` dessous, ou grisé avec `ruins.altar.needs` ; il ouvre la fenêtre
  `ruins.altar.window` : une ligne-bouton par paiement légal (`ruins.payment.*` séparés par des virgules), `ruins.altar.no_undo`,
  `adventure.cancel`.
- **Entrer** (C35, `PlayerView::ruins`, `enemy_tokens`) : bouton et confirmation de la spec 044 ; `adventure.confirm.fight_colors`
  à la première entrée, `fight_waiting` ensuite, `adventure.confirm.reward` avec la récompense du jeton.
- **Drapeau** (C36, `PlayerView::shields`) : celui des sites conquis.
- **Récompenses en attente** (C37) : ligne `reward.pending_ruins` sous les points jusqu'à la fin du tour, à la place des lignes
  séparées quand une ruine est conquise.
- **Artefacts** (C38) : la fenêtre du choix d'artefact (spec 041) accepte trois artefacts et deux prises ; touches 1 à 3.
- **Unité** (C39, `PlayerView::unit_rewards`, `offers`, `command_tokens`, `units`) : fenêtre Offres ouverte d'office, titre
  `reward.choose_unit`, boutons `reward.take` sur les unités ; sans jeton libre, le choix de l'unité à renvoyer (spec 042) puis
  `reward.decline_unit`.
- **Action avancée de récompense** (C41, issue privée 140) : la fenêtre Offres du choix titre `reward.choose_advanced_action` tant que le choix en
  cours vient de la récompense (choix en attente plus nombreux que les niveaux pairs du tour non encore servis), puis
  `offers.choose_advanced_action` pour le niveau. Depuis la [spec #20](issue-20-obtention-des-cartes.md) : fenêtres de choix (`CardChoiceWindow`) et fenêtre du
  niveau pair, la fenêtre Offres n'est plus ouverte d'office.
- **Trajet arrêté** (C40, `PlayerView::revealed_ruins`) : ligne `ruins.revealed` sous les points.

```
┌─ Panneau latéral (sur un autel bleu, jour) ──────────────┐
│ Mouvement : 2 · Influence : 0                            │
│ [Faire une offrande à l'autel]                           │
│ Entrer fait perdre : 2 mouvement, 0 influence            │
└──────────────────────────────────────────────────────────┘
┌─ Offrande : 3 mana bleu, 7 Gloire ───────────────────────┐
│ [1  dé bleu, cristal bleu, jeton bleu]                   │
│ [2  cristal bleu, cristal bleu, jeton bleu]              │
│ La gloire est gagnée aussitôt.          [Annuler]        │
└──────────────────────────────────────────────────────────┘
```

Vérification à l'œil, trois parties préparées (bac à sable, campagne 8 posée en 2A, ruine en (7, −5)) :

- `ruines-autel` : jour, figurine sur la ruine, jeton autel bleu, un cristal bleu, un jeton bleu, un dé bleu dans la Source ;
  faire l'offrande (fenêtre des paiements), voir 7 gloire et le drapeau.
- `ruines-nuit` : nuit, figurine à deux pas de la ruine face cachée (jeton gris + violet, une unité), assez de mouvement pour la
  traverser ; un trajet qui la traverse s'arrête dessus (message), le jeton se retourne ; entrer, combattre.
- `ruines-unite` : la même ruine conquise ce tour, jetons de commandement tous pris, fin du tour lancée : **s'arrête sur le choix
  de l'unité** (renvoyer ou renoncer) (issue privée 131).

### Raccourcis clavier

| Touche | Effet | Contexte |
|---|---|---|
| E | Faire une offrande à l'autel (ouvre la fenêtre), comme « Entrer dans » | Partie, sur un autel, bouton actif, hors mode choix |
| 1 à 9 | Choisir le paiement de l'offrande | Fenêtre de l'offrande |
| Échap | Fermer la fenêtre de l'offrande | Fenêtre de l'offrande |
| 1, 2, 3 | Prendre le 1er, 2e, 3e artefact tiré (3 s'ils sont trois) | Choix d'artefact en attente (étend la spec 041) |
| 1, 2, 3 | Prendre la 1re, 2e, 3e unité de l'offre | Choix de l'unité de récompense |

Reportées dans `docs/raccourcis.md` à l'implémentation. E reste « entrer dans le site d'aventure » ; une ruine a un autel ou des
ennemis, jamais les deux. Pas de touche pour « Renoncer » : un clic, puisque renoncer perd la récompense.

## Effets sur les specs déjà implémentées

- **015, 012** : l'exploration et la mise en place posent les jetons de ruines ; nouveau `RuinPileShuffled` à la mise en place.
- **023, 043** : la nouvelle manche de Jour retourne aussi les jetons de ruines.
- **036, 043** : un trajet s'arrête aussi sur une ruine révélée (nuit).
- **041** : *n* artefacts tirent *n* + 1 cartes (aujourd'hui *n* vaut toujours 1 : inchangé pour le monastère, le tombeau, le
  Sombre Terrier).
- **044** : `EnterAdventureSite` s'ouvre aux ruines à ennemis ; `SiteConquered` donne aussi les récompenses d'un jeton de ruines.
- **050** : le choix d'action avancée sert aussi à la récompense ; **042** : le renvoi sert aussi à la récompense unité.
- **Parcours et parties préparées** : `RuinPileShuffled` s'ajoute à chaque mise en place et un jeton se pose sur la ruine de la
  campagne 8 ; nouvel usage du hasard, les autres tirages ne bougent pas (ADR 0007) ; parties préparées régénérées.

## Hors périmètre

- Ruines des extensions (*Lost Legion* : autel des quatre couleurs, trois Orques ; Compendium, *Miscellaneous* S11).
- Effets qui révèlent les ruines de nuit (artefact *Amulet of the Sun*) ou qui déplacent sans entrer sur les cases (*Space
  Bending*, *Temporal Portal* ; Compendium, *Movement* O9) : avec leurs cartes (J5).
- Choisir librement l'ordre des récompenses (MKUE p. 11, 5) : ordre fixe, sans effet en solo.
- Cimetières, nécropoles, souterrains (extensions, scénarios) : hors périmètre.

## Plan de développement

Selon l'issue privée 118 (moteur et écran à la fois), deux fils sur la même branche, plan
[`plans/045-ruines.md`](plans/045-ruines.md) (étapes, colonne « Rouge attendu ») :

1. **Moteur** : contenu, pile et pose, révélation, autel, ennemis, récompenses, vue, persistance, parties préparées, docs du moteur
   (C1 à C30).
2. **Écran** : libellés, C31 à C40, vérification à l'œil, docs de l'écran, bilan.

## Décisions (2026-11-05)

Questions de la rédaction, tranchées par Mathieu (« ok pour le reste ») : toutes les propositions sont retenues. Q6 précisée :
le Bouclier du livret est un état du moteur, l'écran montre le drapeau du héros, et une ruine est dite « conquise ».

1. **Q1 — Payer l'offrande.** Retenu : une fenêtre qui liste les paiements légaux (une ligne par combinaison : « dé bleu,
   cristal bleu, jeton bleu »), un clic paie. Autre choix : payer d'office (dé d'abord, puis jetons, puis cristaux) sans fenêtre.
2. **Q2 — Trajet qui retourne une ruine** (nuit) : retenu : il s'arrête sur la ruine, comme pour un défenseur retourné
   (spec 043) : le joueur décide en voyant le jeton. Autre choix : le trajet continue (la révélation n'oblige à rien).
3. **Q3 — Gloire de l'autel** : retenu : gagnée aussitôt (le niveau, lui, attend la fin du tour). Autre choix : en fin de tour
   avec les autres récompenses ; en solo, aucune différence de jeu.
4. **Q4 — Récompense unité** : retenu : choisie après le changement de niveau (couvre l'exception de 5.d), dans la fenêtre
   Offres ; « Renoncer » proposé seulement sans jeton de commandement libre (le livret ne prévoit de renoncer que dans ce cas).
5. **Q5 — Dessin du jeton** : retenu : hexagone jaune dessiné par le code, pastilles de couleur (mana de l'autel, piles
   d'ennemis), le détail (récompense) dans l'infobulle. Autre choix : une icône de récompense sur le jeton (à dessiner).
6. **Q6 — Termes** : « jeton de ruines » (la carte FR dit « jeton jaune »), « autel », « offrande » et « faire une offrande »
   (carte FR), « récompense du jeton », « un cristal de chaque couleur ». Une ruine dont l'offrande est payée ou dont tous les
   ennemis sont vaincus est dite **conquise**, comme les autres sites d'aventure (`site.conquered.ruins` : « Ruines
   conquises », clé ajoutée à l'implémentation de l'écran, car `site.conquered.adventure` donne « {site} conquis », au
   masculin). Le jeton Bouclier du livret n'existe que dans le moteur (`GameState::shields`) : **à l'écran, c'est le drapeau
   du héros** (spec 043), jamais un « Bouclier » ; aucun libellé de cette spec ne dit « Bouclier ».
7. **Q7 — Une spec, deux fils** (moteur, puis écran) sur la même branche.

## Mise à jour (issue privée 140, 2026-11-06)

Bilan de l'implémentation : le choix de l'action avancée d'une récompense de ruine gardait le titre générique « Offres ». Ajout
de C41 et du libellé `reward.choose_advanced_action`, proposés dans l'issue par Mathieu. Le moteur ne change pas : la vue dit
déjà assez (`advanced_action_choices`, `levels_gained`), puisque la récompense passe avant le niveau (C24).

## Questions ouvertes

Aucune.
