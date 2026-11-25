# Glossaire

Correspondance entre le livret (anglais), les specs et l'UI (français) et le code (anglais). Utiliser ces termes partout, sans synonymes.

Les traductions françaises sont une **proposition à valider par Mathieu** : en cas de doute, demander avant d'en introduire une nouvelle. Tout nouveau terme est ajouté ici avant d'être utilisé dans le code.

| Livret (EN) | Français (specs, UI) | Code |
|---|---|---|
| Round | Manche | `Round` |
| Turn | Tour | `Turn` |
| Day / Night | Jour / Nuit | `TimeOfDay::Day` / `Night` |
| Scenario | Scénario | `Scenario` |
| Dummy Player | Joueur fantôme | `DummyPlayer` |
| Tactic (card) | Tactique | `Tactic` |
| Early Bird, Rethink, Mana Steal, Planning, Great Start, The Right Moment | Lève-tôt, Faux départ, Vol de mana, Planification, Départ flamboyant, Au bon moment (tactiques de Jour 1 à 6, cartes FR) | `Tactic::EarlyBird` … `Tactic::TheRightMoment` (spec 060) |
| From the Dusk, Long Night, Mana Search, Midnight Meditation, Preparation, Sparing Power | Au crépuscule, Longue Nuit, Appel du mana, Méditation nocturne, Préparation, Provisions (tactiques de Nuit 1 à 6, cartes FR) | `Tactic::FromTheDusk` … `Tactic::SparingPower` (spec 060) |
| Remove (a Tactic) from the game | Retirer (une tactique) de la partie | `GameState::removed_tactics` (spec 060) |
| Flip (a Tactic card) face down | Retourner (une tactique) | `TacticState::flipped` (spec 060) |
| Round Order (token) | Ordre du tour | `GameState::round_order` (spec 061) |
| Dummy player's turn | Tour du fantôme | `Event::DummyTurnPlayed` (spec 061) |
| Flip over (cards from the Dummy's Deed deck) | Retourner (des cartes du deck du fantôme) | `DummyTurnPlayed::cards` (spec 061) |
| Deed deck | Deck | `DeedDeck` |
| Discard pile | Défausse | `DiscardPile` |
| Hand limit | Limite de main | `hand_limit` ; bonus des forteresses possédées : `GameState::keep_hand_bonus` (spec 043) |
| Advanced Action | Action avancée | `AdvancedAction` |
| Spell | Sort | `Spell` |
| Cast (a Spell) | Lancer (un sort) | `PlayCard` d'un sort, payé de sa couleur, et du noir pour l'effet renforcé (spec 054) |
| Artifact | Artefact | `Artifact` |
| Skill | Compétence | `Skill` |
| Pile of Skill tokens | Pile de compétences | `PlayerState::skill_pile` (spec 051) |
| Common Skill offer | Offre de compétences communes (FAQ p3) | `GameState::common_skills` (spec 051) |
| Once a turn / Once a Round (Skill) | Une fois par tour / une fois par manche | `SkillUsage::Turn` / `Round` (spec 051) |
| Flip (a Skill token) face down | Retourner (une compétence) | `SkillState::Flipped` (spec 051) |
| Competitive / cooperative interactive Skill | Compétence compétitive / coopérative | `Skill::is_cooperative` (spec 051) |
| Put (a Skill token) in the center | Poser (une compétence) au centre | `SkillState::InCenter` (spec 058) |
| Secondary advantage (of an interactive Skill) | Avantage différé | `CenterAdvantage` (spec 058) |
| Opposite color (mana token diagram) | Couleur opposée | `ManaColor::opposite` (spec 058) |
| Command token of Bonds of Loyalty | Jeton de Loyauté | `PlayerState::bonds_unit` (spec 058) |
| Unit | Unité | `Unit` |
| Regular Unit / Elite Unit | Unité régulière (FAQ p1) / Unité d'élite | `UnitRank::Regular` / `Elite` (spec 050) |
| Recruiting icons (Village, Monastery, Keep, Mage Tower, City) | Lieux de recrutement | `RecruitSite` (spec 050) |
| Offer | Offre | `Offer` (spec 050) |
| Advanced Action offer / Spell offer / Unit offer | Offre d'actions avancées / de sorts / d'unités (FAQ p2, p3) | `Offer::AdvancedActions` / `Spells` / `Units` |
| Advanced Action deck / Spells deck / Artifact deck / Regular, Elite Unit deck | Pioche d'actions avancées / de sorts / d'artefacts / d'unités régulières, d'élite (« deck » est réservé au deck du joueur) | `CommonDeck` (spec 050) |
| Top position / lowest position (of an offer) | Première place / dernière place | ordre de l'offre, première place en tête (spec 050) |
| Refresh (an offer) | Renouveler (une offre) | `OfferCardReturned` puis `OfferCardDealt` (spec 050) |
| Replenish (an offer) | Compléter (une offre) | `OfferCardDealt` après `OfferCardGained` (spec 050) |
| Gain (a card) | Gagner (une carte) | `OfferCardGained`, `GainAdvancedAction` (spec 050) |
| Command token | Jeton de commandement | `CommandToken` |
| Unit area | Zone des unités | `PlayerState::units` (spec 042) |
| Ready / Spent (Unit) | Disponible (FAQ p2) / Épuisée | `OwnedUnit::ready` (spec 042) |
| Wounded (Unit) | Blessée | `OwnedUnit::wounds` (spec 042) |
| Recruit (a Unit) | Recruter (FAQ p1, p2) | `Action::RecruitUnit` (spec 042) |
| Disband (a Unit) | Renvoyer (FAQ p1, p3) | `UnitDisbanded` (spec 042) |
| Destroyed (Unit) | Détruite | `UnitDestroyed` (spec 042) |
| Activate (a Unit), ability | Activer, capacité | `Action::ActivateUnit`, `UnitAbility` (spec 042) |
| Ready a Unit (effect) | Rendre disponible une unité | `Effect::ReadyUnit` (spec 042) |
| Assign damage | Assigner les dégâts (FAQ p3) | `Action::AssignDamage` (spec 042) |
| Healing points | Points de soin | `PlayerState::healing` (spec 042) |
| Level | Niveau | `Level` |
| Fame | Gloire | `Fame` |
| Fame track | Piste de gloire | `FameTrack` (seuils des niveaux, spec 035) |
| First Reconnaissance | Première reconnaissance | `ScenarioId::FirstReconnaissance` (spec 063) |
| Solo Conquest | Conquête en solitaire | `ScenarioId::SoloConquest` (spec 064) |
| Scenario End | Fin du scénario | `ScenarioEnd` (`RoundLimit`, `CityRevealed`, `AllCitiesConquered`) |
| (scenario) victorious / failed | Mission accomplie / échouée | partie « conquête » de `FinalScore` (spec 064) |
| Unplayed Round | Manche non jouée | partie « conquête » de `FinalScore` (spec 064) |
| One last turn | Dernier tour | `LastTurnAnnounced` |
| (Final) Scoring | Décompte final (FAQ p3) | `FinalScore` |
| Achievement | Haut Fait (prise en main FR p. 19) | `Achievement` |
| The Greatest Knowledge / Loot / Leader / Conqueror / Adventurer / Beating | Le Savant / Le Collectionneur / Le Commandant / Le Conquérant / L'Aventurier / La Victime (prise en main FR p. 19) | `Achievement::Knowledge` / `Loot` / `Leader` / `Conqueror` / `Adventurer` / `Beating` |
| Shield token | Jeton Bouclier (FAQ p3) | `GameState::shields` (spec 041) |
| Combat reward | Récompense (de combat) | `PlayerState::artifact_rewards` (spec 041) |
| Take an Artifact (reward) | Prendre un artefact (« Récompense : prenez un artefact ») | `Action::TakeArtifact`, `ArtifactsDrawn`, `ArtifactTaken`, `ArtifactReturned` (spec 041) |
| Level token | Jeton de niveau | `LevelToken` (armure, limite de main ; spec 035) |
| Level up | Changement de niveau (FAQ p3) | `LevelGained` (spec 035) |
| Reputation | Réputation | `Reputation` |
| Reputation track | Piste de réputation | `ReputationTrack` (spec 040) |
| Reputation bonus or penalty (modifier) | Bonus de réputation (négatif : malus) | `ReputationTrack::bonus` (spec 040) |
| X space (Reputation track) | Case X | `Reputation::is_x` (spec 040) |
| Wound | Blessure | `DeedCard::Wound` (carte, spec 031) |
| Healing | Soin | `Healing` |
| Heal X | Soin X | `Effect::Heal` (spec 034) |
| Throw away (a Wound) | Jeter (une Blessure) (FAQ p3) | `WoundsHealed` (spec 034) |
| Throw away (an Action card) | Détruire (une carte action) (cartes FR ; spec 056, Q4) | `CardDestroyed` (spec 056) |
| Throw away (an Artifact) | Détruire (un artefact), pour son effet renforcé (spec 055, Q9) | `CardPlay::ThrowAway`, `CardDestroyed` (spec 055) |
| Banner (Artifact) | Bannière | `Artifact::is_banner` (spec 055) |
| Assign (a Banner to a Unit) | Assigner (une bannière à une unité) | `Action::AssignBanner` (spec 055) |
| Flip (a Banner) face down | Retourner (une bannière) | `OwnedUnit::banner_flipped` (spec 055) |
| Reclaim (a Banner, end of Round) | Reprendre (une bannière) | `Action::ReclaimBanners` (spec 055) |
| Endless supply (of mana) | Mana illimité | `ModifierKind::EndlessMana`, `ManaPayment::Endless` (spec 055) |
| Cancel (an enemy attack) | Annuler (une attaque) | `UseBanner`, `DoesNotAttack` (spec 055) |
| Draw a card (effect) | Piocher une carte | `Effect::DrawCards` (spec 034) |
| Healing card | Carte de soin | carte dont tous les effets sont de soin (spec 034) |
| Move | Mouvement | `Move` |
| Influence | Influence | `Influence` |
| Attack | Attaque | `Attack` |
| Ranged Attack | Attaque à distance | `RangedAttack` |
| Siege Attack | Attaque de siège | `SiegeAttack` |
| Block | Parade | `Block` |
| Physical / Fire / Ice / Cold Fire | Physique / Feu / Glace / Feu glacé | `Element::Physical` / `Fire` / `Ice` / `ColdFire` |
| Resistance | Résistance | `Resistance` |
| Swift / Brutal / Poison / Paralyze | Vif / Brutal / Toxique / Paralysant (termes de la FAQ française, spec 033) | `EnemyAbility::Swift` / … |
| Source | Source | `Source` |
| Mana die | Dé de mana | `ManaDie` |
| Mana color | Couleur de mana | `ManaColor` (`Basic(BasicColor)`, `Gold`, `Black`) |
| Gold (mana) | Or | `ManaColor::Gold` |
| Black (mana) | Noir | `ManaColor::Black` |
| Pure mana | Mana fondamental | — (specs 021 et 022) |
| Roll / Reroll (dice) | Lancer / Relancer | `SourceRolled` |
| Depleted (die) | Épuisé | `ManaColor::is_depleted` |
| Reset the Source | Réinitialiser la Source | `SourceRolled` (à la préparation de la manche) |
| Crystal | Cristal | `Crystal` |
| Inventory | Inventaire | `Inventory` (au plus 3 cristaux par couleur de base, `CRYSTAL_LIMIT`) |
| Gain (a crystal) | Gagner (un cristal) | `CrystalGained` |
| Mana token | Jeton de mana | `ManaTokenGained`, `ManaTokenUsed`, `ManaPayment::Token` |
| Gain (a mana token) | Gagner (un jeton) | `ManaTokenGained` |
| Set (a die) to a color | Mettre (un dé) sur une couleur | `DieSet`, `DieSetting`, `Effect::SetDice` |
| Additional (mana die) | Dé de plus | `Effect::ExtraDie`, `GameState::dice_limit` |
| Composite effect | Effet composite | `Effect::Boost`, `CardPlay::Strong::with` (`BoostTarget`) ; la carte jouée avec elle : `CardPlay::Boosted` |
| Modifier (effect that modifies rules for the turn) | Effet en cours | `Modifier`, `PlayerState::modifiers` (spec 052, ADR 0008) |
| Take (a card) back to your hand | Reprendre (une carte en main) | `CardReturnedToHand` |
| Bank | Réserve | — (illimitée, rien à modéliser) |
| Explore | Explorer | `Explore` |
| Rampaging (enemy) | Ennemi errant | `Rampaging` |
| Provoke | Provoquer | `Provoke` |
| Interaction | Interaction | `Interaction` |
| Interact with the Locals | Interagir avec les habitants | `Action::Interact`, `InteractionStarted` (spec 040) |
| Locals | Habitants | — (spec 040) |
| Village | Village | `Village` |
| Monastery | Monastère | `Monastery` |
| Buy Healing (village, monastery) | Acheter un point de soin | `Action::BuyHealing`, `HealingBought` (spec 041) |
| Training (monastery) | Entraînement (carte FR) ; acheter une action avancée | `Action::BuyAdvancedAction`, `AdvancedActionBought` (spec 041) |
| Burn a monastery | Incendier un monastère (FAQ p3, carte FR) | `Action::BurnMonastery` (spec 041) |
| Burned monastery | Monastère incendié (carte FR : « brûlé ») | `MonasteryBurned` (spec 041) |
| Plunder (a village) | Piller (carte FR) | `Action::PlunderVillage` (spec 041) |
| Buy a Spell (mage tower) | Acheter un sort | `Action::BuySpell`, `SpellBought` (spec 043) |
| Spell reward (conquered Mage Tower) | Récompense : un sort | `spell_rewards`, `spell_choices`, `Action::TakeSpell` (spec 043) |
| Adventure site | Site d'aventure (carte FR : « site d'aventures ») | `Site::is_adventure` (spec 044) |
| Enter (an adventure site) | Entrer dans (carte FR : « pénétrer dans ») | `Action::EnterAdventureSite`, `AdventureSiteEntered` (spec 044) |
| Night rules (dungeon, tomb) | Règles de Nuit (carte FR) | `GameState::combat_time_of_day` (spec 044) |
| Mana die from the bank (reward roll) | Dé de récompense | `RewardDieRolled`, `RngUsage::RewardDice` (spec 044) |
| Random crystal (reward) | Cristal aléatoire | `crystal_rewards`, `Action::ChooseCrystalColor` (spec 044) |
| Healing Essence (magical glade) | Vapeurs curatives (carte FR) | `EndTurn { glade }`, `WoundThrownAway` (spec 044) |
| Imbued with Magic (magical glade) | Imprégnation magique (carte FR) | `ManaTokenGained` au début du tour (spec 044) |
| Ruins token (yellow token) | Jeton de ruines (carte FR : « jeton jaune ») | `RuinToken`, `content/ruins.ron` (spec 045) |
| Altar | Autel | `RuinToken::is_altar` (spec 045) |
| Pay (mana) as tribute | Faire une offrande (carte FR : « En offrande, vous pouvez payer ») | `Action::PayAltarTribute`, `AltarTributePaid` (spec 045) |
| Ruins reward | Récompense du jeton | `RuinReward` (spec 045) |
| Unit reward | Récompense : une unité | `unit_rewards`, `Action::TakeUnitReward`, `DeclineUnitReward` (spec 045) |
| One crystal of each basic color (reward) | Un cristal de chaque couleur | `RuinReward::CrystalSet`, `crystal_set_rewards` (spec 045) |
| Keep | Forteresse | `Keep` |
| Mage Tower | Tour de mage | `MageTower` |
| City | Cité | `City` |
| Dungeon | Donjon | `Dungeon` |
| Tomb | Tombeau | `Tomb` |
| Monster Den | Antre | `MonsterDen` |
| Spawning Grounds | Sombre Terrier | `SpawningGrounds` |
| Ruins | Ruines | `Ruins` |
| Mine (crystal mine) | Mine | `Site::Mine(BasicColor)` |
| Magical Glade | Clairière magique | `MagicalGlade` |
| Portal | Portail | `Portal` |
| Draconum | Draconien | `Draconum` |
| Orc Marauders | Orques maraudeurs | `OrcMarauders` |
| Enemy token | Jeton d'ennemi | `EnemyToken` (ennemi + numéro d'exemplaire, spec 030) |
| (enemy type, ex. Prowlers) | Ennemi | `Enemy` (spec 030) |
| (token back color) | Couleur d'ennemi | `EnemyColor` (`Green`, `Grey`, `Violet`, `Brown`, `Red`, `White`) |
| (token back category) | Catégorie d'ennemis | `EnemyColor`, libellés `enemy.color.*` (spec #14) |
| (token back category) | Catégorie d'ennemis | `EnemyColor`, libellés `enemy.color.*` : « orques maraudeurs », « gardes de forteresse »… (spec 070, spec #14) |
| Enemy token pile | Pile d'ennemis | `enemy_piles` (une pile par `EnemyColor`) |
| Face up / face down | Face visible / face cachée | défenseurs face cachée : `GameState::face_down_defenders`, `PlayerView::face_down_defenders` (spec 043) |
| Armor | Armure | `armor` |
| Fortified | Fortifié | `EnemyAbility::Fortified` |
| Armor reduced ("Armor -3") | Armure réduite | `ModifierKind`/modificateur d'ennemi `ArmorReduction` (spec 053) |
| "Does not attack this combat" | N'attaque pas | modificateur d'ennemi `DoesNotAttack` (spec 053) |
| Destroy (an enemy) | Détruire (un ennemi) : il est vaincu, sans attaque | `EnemyDestroyed` (spec 054) |
| Lose fortifications / resistances | Perdre ses fortifications / ses résistances | modificateurs d'ennemi `LosesFortifications`, `LosesResistances`, `LosesResistance` (spec 054) |
| Ignore site fortifications | Ignorer les fortifications des sites | `ModifierKind::IgnoreSiteFortifications` (spec 054) |
| Summon | Invocation | `EnemyAttack::Summon` |
| (enemy with Summon Attack) | Invocateur | ennemi dont l'attaque est `EnemyAttack::Summon` (spec 033) |
| Summoned monster | Créature invoquée | `CombatEnemy::summoned` (spec 033) |
| Challenge (a rampaging enemy) | Défier | `Action::Challenge`, `EnemyChallenged` (spec 031) |
| Combat (with enemies) | Combat | `Combat` (spec 031) |
| (start the combat) | Engager le combat | `Action::StartCombat`, `CombatStarted` (spec 031) |
| Ranged and Siege Attack phase | Phase d'attaque à distance | `CombatPhase::RangedAndSiege` (spec 031) |
| Block phase | Phase de parade | `CombatPhase::Block` (spec 031) |
| Assign Damage phase | Phase d'assignation des dégâts | `CombatPhase::AssignDamage` (spec 031) |
| Attack phase | Phase d'attaque | `CombatPhase::Attack` (spec 031) |
| (an) attack, its targets | Attaque (en cours), cibles | `Action::DeclareAttack`, `open_attack` (spec 031 ; pas « colonne », pris par les tuiles) |
| (a) block | Parade (en cours) | `Action::DeclareBlock`, `open_block` (spec 031) |
| Defeat (an enemy) | Vaincre ; ennemi vaincu | `EnemiesDefeated` (spec 031) |
| Blocked (enemy) | Paré | `EnemyBlocked` (spec 031) |
| Enemy discard pile | Défausse d'ennemis | `enemy_discards` (spec 031) |
| Wound pile | Pile de blessures | — (illimitée, spec 031) |
| (enemies to fight) | Ennemis à combattre | `enemies_to_fight` (spec 030) |
| Map | Plateau de jeu | `Map` |
| Space | Case | `Hex` (ses coordonnées) |
| Map tile | Tuile | `Tile`, identifiant `TileId` |
| Starting tile (A / B side) | Tuile de départ (face A / B) | `TileId::StartingA` / `StartingB` |
| Countryside tile | Tuile de campagne | `TileId::Countryside(n)` |
| Core tile | Tuile royaume | `TileId::Core(n)` |
| Terrain | Terrain | `Terrain` |
| Plains / Hills / Forest / Wasteland | Plaine / Colline / Forêt / Terre désolée | `Terrain::Plains` / `Hills` / `Forest` / `Wasteland` |
| Desert / Swamp / Lake / Mountain | Désert / Marais / Lac / Montagne | `Terrain::Desert` / `Swamp` / `Lake` / `Mountain` |
| — (sea of the starting tile) | Littoral | `Terrain::Sea` |
| Site | Site | `Site` |
| Red / Blue / White / Green (basic colors) | Rouge / Bleu / Blanc / Vert | `BasicColor::Red` / `Blue` / `White` / `Green` |
| Hero | Héros | `Hero` |
| Figure (of a hero) | Figurine | `HeroFigure` |
| Arythea / Goldyx / Norowas / Tovak | (noms propres, inchangés) | `Hero::Arythea` / `Goldyx` / `Norowas` / `Tovak` |
| Setup | Mise en place | `setup` |
| Map shape | Forme du plateau de jeu | `MapShape` |
| Wedge (map) | En coin | `MapShape::Wedge` |
| (Fully) Open map, limited to n columns | Ouverte (n colonnes) | `MapShape::Open(Columns)` |
| Column (of tiles) | Colonne | `Columns::Three` / `Four` / `Five` |
| Coastline | Côte | — (limite définie par la forme ; ne pas confondre avec le Littoral, `Terrain::Sea`) |
| Wedge map (standard) | En coin standard : en coin, sans tuile royaume sur la côte (MKUE p. 14) | `MapShape::StandardWedge` (spec 064) |
| Competitive Spell | Sort compétitif : l'un des quatre sorts qui visent les autres joueurs (17/20 à 20/20) | sorts du scénario (spec 064) |
| Tile deck | Pioche de tuiles | `TileDeck` |
| City tile / non-city tile (core) | Tuile royaume de cité / sans cité | `TileId::is_city()` |
| Deed card | Carte | `DeedCard` |
| Basic Action card | Carte d'action de base | `BasicAction` |
| Hand | Main | `hand` |
| Play area | Zone de jeu | `PlayArea` |
| Basic effect | Effet de base | `CardPlay::Basic` |
| Strong effect | Effet renforcé | `CardPlay::Strong` |
| Power (a card, with mana) | Renforcer (une carte) ; carte « renforcée » | `CardPlay::Strong` |
| Take (a die from the Source) | Prendre (un dé) | `DieTaken` (au plus `DICE_PER_TURN` par tour) |
| Return (dice to the Source), rerolled | Rendre (un dé), relancé | `DieReturned` |
| Turn a crystal into pure mana | Utiliser (un cristal) | `CrystalUsed` |
| (paying mana) | Payer (du mana) | `ManaPayment` (`Die(n)`, `Crystal`) |
| Played sideways | Jouée de côté | `CardPlay::Sideways` |
| Move points | Points de mouvement | `move_points` |
| Influence points | Points d'influence | `influence_points` |
| Discard (a card) | Défausser | `CardDiscarded` |
| Ice Attack / Ice Block | Attaque de glace / Parade de glace | `Attack` / `Block` avec `Element::Ice` |
| Fire Attack / Fire Block | Attaque de feu / Parade de feu | `Attack` / `Block` avec `Element::Fire` (spec 032) |
| Cold Fire Attack / Cold Fire Block | Attaque de feu glacé / Parade de feu glacé | `Attack` / `Block` avec `Element::ColdFire` (spec 032) |
| Efficient (attack, block) | Efficace (compte en plein) | `Efficiency::Full` (spec 032) |
| Inefficient (attack, block) | Inefficace (divisée par deux) | `Efficiency::Halved` (spec 032) |
| (card) color | Couleur (d'une carte) | `BasicColor` |
| Move cost | Coût de déplacement | `move_cost` |
| Inaccessible (space) | Infranchissable | `MoveCost::Inaccessible` |
| Adjacent (space) | Adjacente | `Hex::neighbors` |
| Fortified site | Site fortifié | `Site::is_fortified` |
| Assault (a fortified site) | Assaut ; assaillir (carte FR : « peut être assaillie ») | `AssaultStarted` (spec 043) |
| Defender (of a fortified site) | Défenseur (carte FR) | jeton de `enemy_tokens` sur un site fortifié non conquis, posé par `DefenderPlaced` (spec 043) |
| Fortified twice | Doublement fortifié (FAQ p3) | défenseur qui a aussi la capacité Fortifié (spec 043) |
| Conquer, conquered / unconquered (site) | Conquérir, conquis / non conquis (carte FR) | `SiteConquered` (spec 043) |
| (your) Keep | Forteresse possédée (carte FR : « vous appartient ») | Bouclier du joueur sur une forteresse (spec 043) |
| Withdraw (after a failed assault) | Battre en retraite, retraite (FAQ p3) | `HeroWithdrew` (spec 043) |
| Forced Withdrawal | Retraite forcée (FAQ p3) | `ForcedWithdrawal` (spec 052) |
| Safe space | Case sécurisée (FAQ p3) | `is_safe_space` (spec 052) |
| Reveal (an enemy token) | Révéler, retourner | `EnemyTokenRevealed` (spec 043) |
| City level | Niveau de la cité | `CityState::level` (spec 062) |
| City card (bonus, interaction) | Carte de cité ; bonus de la cité | `CityState::color` (spec 062) |
| City leader | Seigneur de la cité (carte FR) | `CityState::leader` (spec 062) |
| City figure (base circles) | Cercles de la figurine ; défenseurs de la cité | `cities.ron` (spec 062) |
| Megapolis | Mégapole | — (hors périmètre, spec 062) |
| Day/Night board | Plateau Jour/Nuit | — (la donnée est `TimeOfDay`) |
| Reveal (a tile) | Révéler | `TileRevealed` |
| Tile removed from the game during Setup | Tuile écartée | `removed_tiles` |
| Available position (for a new tile) | Emplacement libre | `Map::free_neighbor_positions` |
| End of the Turn | Fin du tour | `EndTurn` (action), `TurnEnded` (événement) |
| Announce (the end of one's turn) | Finir son tour | `EndTurn` |
| Movement phase | Phase de déplacement (avant l'action du tour) | `movement_ended` faux (spec 057) |
| Time Bent turn (Time Bending) | Tour supplémentaire | `extra_turn`, `ExtraTurnStarted` (spec 057) |
| Set aside (a card) | Écarter (une carte), carte écartée ; pas « mettre de côté », pris par « jouer de côté » | `set_aside` (spec 057) |
| Die kept in the Play area (Mana Claim) | Dé gardé | `kept_die`, `DieKept` (spec 057) |
| Draw (new cards) | Repiocher | `CardDrawn` |
| (Turn number) | Numéro du tour | `turn` |
| Regular turn | Tour normal | — (seul possible avant la spec 034) |
| Rest | Repos | fin de tour qui défausse des Blessures, héros ni déplacé ni au combat (spec 034) |
| Slow Recovery | Repos prolongé (FAQ p3) | fin de tour qui défausse une Blessure, main faite de Blessures (specs 031 Q12, 034) |
| Standard Rest | Repos standard (FAQ p3) | fin de tour qui défausse au moins une carte non-Blessure et des Blessures (spec 034) |
| Knock out / Knocked out | Assommer / Assommé (FAQ p2) | `KnockedOut` (spec 034) |
| Forfeit (one's turn) | Passer son tour | `AnnounceEndOfRound` ; fin de manche automatique quand le fantôme a annoncé et que main et deck sont vides (spec 061) |
| End of the Round | Fin de manche | `RoundEnded` |
| Announce the End of the Round | Annoncer la fin de la manche | `AnnounceEndOfRound` |
| Prepare the Round | Préparer la manche | `RoundStarted` et les événements qui le suivent |
| (Round number) | Numéro de la manche | `round` |
| Round limit | Nombre de manches (du scénario) | `round_limit` |

## Cartes d'action de base (spec 013)

Noms de l'édition française ; identifiants de code repris des noms anglais.

| Livret (EN) | Français | Code (`BasicAction::`) |
|---|---|---|
| Rage | Rage | `Rage` |
| Determination | Détermination | `Determination` |
| Swiftness | Célérité | `Swiftness` |
| March | Marche | `March` |
| Stamina | Endurance | `Stamina` |
| Tranquility | Tranquillité | `Tranquility` |
| Promise | Promesse | `Promise` |
| Threaten | Menace | `Threaten` |
| Crystallize | Cristallisation | `Crystallize` |
| Mana Draw | Source de mana | `ManaDraw` |
| Concentration | Concentration | `Concentration` |
| Improvisation | Improvisation | `Improvisation` |
| Battle Versatility (Arythea) | Polyvalence | `BattleVersatility` |
| Mana Pull (Arythea) | Attraction de mana | `ManaPull` |
| Will Focus (Goldyx) | Pouvoir de concentration | `WillFocus` |
| Crystal Joy (Goldyx) | Euphorie cristalline | `CrystalJoy` |
| Noble Manners (Norowas) | Nobles manières | `NobleManners` |
| Rejuvenate (Norowas) | Rajeunissement | `Rejuvenate` |
| Cold Toughness (Tovak) | Endurance givrée | `ColdToughness` |
| Instinct (Tovak) | Instinct | `Instinct` |

## Termes techniques du moteur

Pas de terme correspondant dans le livret. Introduits par la spec 000.

| Français | Code | Sens |
|---|---|---|
| Configuration de départ | `GameConfig` | Ce qui suffit à créer une partie : graine, et plus tard scénario, héros, variantes. |
| État de partie | `GameState` | Tout l'état, information cachée comprise. Clonable à bas coût. |
| Action | `Action` | Intention d'un joueur, à valider par le moteur. |
| Événement | `Event` | Fait accompli, hasard déjà résolu. |
| Décision en attente | `PendingDecision` | Quel joueur doit choisir, et quel type de choix. |
| Vue joueur | `PlayerView` | Ce qu'un joueur a le droit de connaître de l'état. |
| Journal | `EventLog` | Suite ordonnée et numérotée des événements de la partie. |
| Journal des décisions | `DecisionLog` | Pour chaque décision : vue, actions légales, choix. |
| Boucle de partie | `GameRunner` | Fait avancer la partie décision par décision, sans bloquer. |

### Libellés (spec 001)

| Français | Code | Sens |
|---|---|---|
| Libellé | `Label` | Texte affiché à l'écran, dans une langue donnée. |
| Clé de libellé | `LabelKey` | Identifiant hiérarchique d'un libellé, ex. `card.march.name`. |
| Langue | `Language` | Code de langue, tiré du nom du fichier : `fr`, `en`… |
| Catalogue | `Catalog` | Ensemble des libellés d'une langue, chargé depuis son fichier. |
| Langue du système | `system_language` | Langue de l'interface du système d'exploitation, lue au lancement (spec 076). |
| Langue demandée | `--lang` | Langue forcée au lancement par le paramètre `--lang` (spec 076). |
| Langue de repli | `FALLBACK_LANGUAGE` | L'anglais (`en`) : langue affichée quand ni la langue demandée ni celle du système n'ont de fichier. |
| Langue disponible | `available_languages` | Langue qui a un fichier `assets/i18n/<code>.json`. |
| Paramètre | — | Valeur nommée insérée dans un libellé, écrite `{amount}`. |

### Interface (spec 002)

| Français | Code | Sens |
|---|---|---|
| Joueur humain | `HumanPlayer` | Implémentation de `Player` alimentée par les clics de l'UI. |
| Écran de partie | `GameScreen` | Ce que la fenêtre doit afficher, déjà traduit : textes et boutons. Calculé sans egui, donc testable. |
| Dossier des ressources | `AssetsDir` | Dossier `assets` lu au lancement (libellés aujourd'hui, autres fichiers du jeu plus tard). |

### Logs techniques (spec 003)

À ne pas confondre avec le Journal (`EventLog`) ni avec le Niveau du héros (`Level`).

| Français | Code | Sens |
|---|---|---|
| Logs techniques | `log` (crate) | Messages de diagnostic écrits pendant l'exécution. Jamais appelés « journal ». |
| Gravité | `log::Level` | `error`, `warn`, `info`, `debug`, `trace`. Jamais appelée « niveau ». |
| Fichier de logs | `LogFile` | Fichier texte où l'exécutable écrit les logs d'un lancement. |
| Dossier des logs | `logs_dir` | Dossier qui contient les fichiers de logs : `logs` du dépôt en développement, dossier de données de l'utilisateur une fois déployé. |

### Plateau de jeu (spec 010)

| Français | Code | Sens |
|---|---|---|
| Emplacement de tuile | `TilePosition` | Place fixe où une tuile peut être posée, repérée par sa case centrale. |
| Direction | `Direction` | Une des six directions d'une case : `E`, `NE`, `NW`, `W`, `SW`, `SE`. |
| Sorte de tuile | `TileKind` | Départ, campagne ou royaume, d'après le dos de la tuile. |
| Définition de tuile | `TileDefinition` | Contenu statique d'une tuile : sa case centrale et ses six côtés. |
| Contenu d'une case | `SpaceContent` | Terrain d'une case et, au plus, un élément imprimé. |
| Élément imprimé | `Feature` | Site (`Site`) ou ennemi errant (`Rampaging::OrcMarauders` / `Draconum`) imprimé sur une case. |

### Affichage du plateau de jeu (spec 011)

| Français | Code | Sens |
|---|---|---|
| Cadrage | `BoardCamera` | Partie visible du plateau de jeu et son échelle : un point au centre de la zone et un zoom. Pas « vue », réservé à la vue joueur. |
| Écran du plateau | `BoardScreen` | Ce qu'il faut dessiner dans la zone du plateau de jeu, déjà calculé et traduit. Calculé sans egui, donc testable. |
| Infobulle de survol | `hover_lines` | Lignes qui décrivent ce qu'il y a sous le curseur sur le plateau de jeu, affichées à côté de lui (spec 037, remplace le « panneau de survol »). Pas un terme du livret. |

### Scénario et mise en place (spec 012)

| Français | Code | Sens |
|---|---|---|
| Bac à sable | `ScenarioId::Sandbox` | Notre scénario d'entraînement, absent du livret : 4 manches, ennemis errants (spec 030), sans autres sites actifs. |
| Composition de la pioche de tuiles | `TileDeckRecipe` | Nombre de tuiles de campagne et leur ordre (au hasard, par numéro), nombre de tuiles royaume de cité et sans cité. |
| Coordonnées d'emplacement | `TilePosition::lattice()` | Couple `(i, j)` d'un emplacement : sa case centrale vaut `i × (3, −2) + j × (1, −3)`. |
| Zone interdite | — (UI) | Emplacements sans tuile que la forme du plateau de jeu ne permet pas, hachurés à l'écran. |

### Nouvelle partie (spec 065)

| Français | Code | Sens |
|---|---|---|
| Graine | `GameConfig::seed` | Nombre dont découle tout le hasard d'une partie (spec 038) ; tirée de l'horloge, ou saisie pour rejouer une partie. Jamais affichée. |
| Cartes propres (d'un héros) | `Hero::own_cards` | Les deux cartes d'action de base du deck de départ d'un héros qui remplacent deux cartes communes (MKUE p. 3). |
| Écran de nouvelle partie | `NewGameScreen` | Ce que la fenêtre de nouvelle partie affiche, déjà traduit : héros, scénarios, graine, boutons. Calculé sans egui, donc testable. |
| Illustration du héros | `hero_image` (UI) | Buste du héros dans un médaillon de sa couleur (`hero_color`) : sa figurine sur le plateau de jeu, son portrait dans le panneau et sur l'écran de nouvelle partie. |
| Session du client | `ClientSession` (UI) | L'application vue par l'UI : l'écran de nouvelle partie seul au lancement, ou une partie en cours (`GameSession`) avec éventuellement la fenêtre de nouvelle partie par-dessus. |
| Étape (de la nouvelle partie) | `NewGameStep` (UI) | Scénario ou Héros : l'écran de nouvelle partie en montre une à la fois, la barre d'étapes passe de l'une à l'autre (spec 078). |
| Fiche du scénario, fiche du héros | `ScenarioSheet`, `HeroSheet` (UI) | Grande zone de droite d'une étape : ce que propose le scénario, ou ce que fait le héros et ses cartes propres (spec 078). |
| Accroche | — (UI) | Ligne courte sous le nom d'un scénario ou d'un héros dans la liste de l'écran de nouvelle partie (spec 078). |
| Conseil du Vide | — (texte) | *Council of the Void* (MKUE p. 22) : le pouvoir que servent les héros, cité dans leurs descriptions (spec 078). |
| Rupture | — (texte) | *The Breaking* (walkthrough p. 20) : le cataclysme d'où naît le chaos du pays (spec 078). |
| Mue | — (texte) | *The Surge* (walkthrough p. 20) : l'évolution qui fait de Goldyx le plus puissant des Draconides (spec 078). |
| Neuvième Cercle | — (texte) | *Order of the Ninth Circle* (walkthrough p. 20) : l'ordre de mercenaires dirigé par Tovak (spec 078). |

### Illustrations générées (spec 072)

| Français | Code | Sens |
|---|---|---|
| Illustration générée | `Illustration` (UI) | Image PNG en bande (404 × 168, 2,4:1, spec #9) d'une unité ou d'une carte, carrée pour la seule Blessure, produite par un générateur d'images hors du jeu et versionnée (ADR 0009). Le plateau, les héros, les ennemis et les icônes restent des SVG. |
| Famille d'illustrations | `IllustrationFamily` (UI) | Unité ou carte : fixe le dossier (`unit-art`, `card-art`) et le fichier des descriptions. |
| Vignette | — (UI) | Petite illustration (64 × 64) posée sur une carte ou une unité, sans infobulle. |
| Préambule de style | `scripts/illustrations/style.json` | Texte commun placé avant chaque description envoyée au générateur, pour que toutes les images aient le même style ; une variante par famille. |
| Lot | — | Ensemble d'illustrations générées et validées ensemble par Mathieu, sur des planches. |
| Génération différée | `generer --differe`, `recuperer` (script) | Appels au générateur groupés en un lot que l'API traite dans la journée, à moitié prix (*Batch API*, spec #9) ; `recuperer` écrit ensuite ses candidats. |
| Planche | — | Image qui montre côte à côte les candidats d'un lot, avec leur identifiant, pour choisir ceux à retenir. |

### Panneaux (spec 073)

| Français | Code | Sens |
|---|---|---|
| Panneau du bas | `BottomPanel` (UI) | Bande sur toute la largeur de la fenêtre, sur le plateau comme en combat : unités, main (puis compétences), et le bandeau du choix en cours. |
| Panneau de droite | `RightPanel` (UI) | Au-dessus du panneau du bas : joueur, actions du lieu (« Ici »), zone de jeu et faits du tour, boutons « Offres… » et « Finir le tour… » toujours visibles. Remplace le panneau latéral. |
| Chevauchement | `fan_layout` (UI) | Cartes d'une section posées l'une sur l'autre quand la place manque ; celle survolée s'affiche en entier et pousse les autres. |
| Bouton d'effet | `EffectButton` (UI) | Bouton d'une option d'une carte ou d'une capacité d'unité : icône et valeur de l'effet (texte sans icône), grisé quand l'option n'est pas légale. |
| Fait du tour | `TurnFact` | Ce que le tour a donné ou coûté au joueur (ennemis vaincus, gloire, carte ou unité gagnée, Blessures…), montré dans la zone de jeu jusqu'au début de son tour suivant. Pas un terme du livret. |

### Barre de menu (spec 074)

| Français | Code | Sens |
|---|---|---|
| Barre de menu | `MenuBar` (UI) | Ligne en haut de la fenêtre, sur le plateau comme en combat : bouton Menu, scénario, jour ou nuit, manche et tour, pioches de tuiles, message de la sauvegarde. |
| Menu | `MenuBar` (UI) | Bouton de la barre qui déroule les commandes de la partie (revenir en arrière, sauvegarder, charger, nouvelle partie, recentrer, plein écran, quitter), chacune avec sa touche en face. |
| Plein écran | `Command::ToggleFullscreen` (UI) | La fenêtre du jeu occupe tout l'écran, sans bordure ; F11 ou le Menu l'active et le désactive. |
| Demande à la fenêtre | `WindowRequest` (UI) | Recentrer, plein écran, fermer, changer de langue (spec 080) : notée par `ClientSession`, transmise à la fenêtre par l'application. |

### Préférences (spec 080)

| Français | Code | Sens |
|---|---|---|
| Préférences | `Preferences` | Réglages du joueur gardés d'une session à l'autre : langue, plein écran au lancement (puis volumes, animations). |
| Fichier des préférences | `preferences.json` | Fichier JSON où elles sont écrites, dans le dossier de configuration. |
| Dossier de configuration | `config_dir` | Dossier du fichier des préférences : `ARCANUM_CONFIG_DIR`, `config/` du dépôt lancé par cargo, sinon le dossier de configuration de l'utilisateur. |
| Fenêtre des préférences | `PreferencesWindow` (UI) | Fenêtre ouverte par Préférences… (Menu, pied de l'écran de lancement, Ctrl+,). |
| Langue des préférences | `Preferences::language` | `None` = langue du système ; sinon un code (`fr`, `en`). |
| Animations (préférence) | `Preferences::animations` | Case de la section Affichage qui permet les animations du plateau (spec #5) ; cochée par défaut. |

### Audio (spec 081)

| Français | Code | Sens |
|---|---|---|
| Musique | `Music` | Morceau en boucle choisi selon la situation : `Title`, `Day`, `Night`, `Combat`. |
| Ambiance | `Ambience` | Fond sonore en boucle sous la musique, de jour ou de nuit. |
| Effet sonore | `Cue` | Son court joué une fois, déclenché par un événement de la partie ou par la fenêtre des préférences. |
| Type de son | `SoundCategory` | Musique, ambiances ou effets : chacun sa piste et son volume, sous le volume général. |
| Volumes | `Volumes` | Les quatre réglages de 0 à 100 (général, musique, ambiances, effets), dans les préférences. |
| Situation sonore | `SoundSituation` | Ce que l'écran montre, pour choisir musique et ambiance : lancement, exploration (jour ou nuit), combat, fin de partie. |
| Directeur audio | `AudioDirector` | Partie pure d'arcanum-audio : situation et événements nouveaux en entrée, ordres de son en sortie. |
| Ordre de son | `AudioCommand` | Demande du directeur au lecteur : changer de musique ou d'ambiance (fondu), jouer un effet (délai), régler un volume. |
| Lecteur audio | `AudioPlayer` | Exécute les ordres sur la carte son (`KiraPlayer`) ou ne fait rien (`SilentPlayer`). |
| Banque de sons | `SoundBank` | Fichiers trouvés dans `assets/sounds/` au lancement, par musique, ambiance et effet, avec leurs variantes. |

### Plateau isométrique (spec 075)

| Français | Code | Sens |
|---|---|---|
| Vue isométrique | `BoardCamera` (UI) | Plateau vu de trois quarts, en 2D : l'axe vertical de l'écran est écrasé, les coordonnées du plateau ne changent pas. |
| Inclinaison | `BOARD_TILT` (UI) | Facteur d'écrasement de la verticale (1 : vue de dessus). |
| Élément debout | `Overlay` (UI) | Ce qui est dessiné droit sur une case, le pied dans la case : décor de relief, site, drapeau, figurine. Dessinés de l'arrière vers l'avant. |
| Décor de relief | `OverlayKind::Relief` (UI) | Arbres, buttes ou pics d'une forêt, de collines ou d'une montagne. |
| Figurine | `OverlayKind::Figure`, `Rampaging`, `Defender`, `SiteEnemy` (UI) | Héros ou monstre debout sur le plateau. |
| Disparition | `DepartingEnemy`, `BoardDeparture` (UI) | Les figurines des ennemis vaincus par le dernier combat, que le plateau efface à son retour (spec #5). Pas un terme du livret. |
| Brouillard | `FOG_IMAGE` (UI) | Silhouette d'un ennemi face cachée, teintée à la couleur de sa pile. |
| Bouton d'action du plateau | `BoardAction` (UI) | Bouton rond à icône posé sur la case concernée : défier, engager le combat, interagir, piller, incendier, entrer, offrande. |
| Menu du site | (UI) | Options d'une interaction en cours (recruter, acheter), ouvertes à côté de la case du site. |

### Exploration (spec 015)

| Français | Code | Sens |
|---|---|---|
| Prochaine tuile | `NextTile` | Ce qu'un joueur sait de la tuile qu'il révélerait : dos du dessus de la pioche, tuile écartée de campagne ou royaume si la pioche est vide, ou aucune. |

### Cases atteignables (spec 017)

| Français | Code | Sens |
|---|---|---|
| Trajet | `Route` | Suite de cases adjacentes accessibles depuis la figurine jusqu'à une case, avec son coût total ; on retient le moins cher (spec 017). Pas un terme du livret. |
| Suivre un trajet | `Action::FollowRoute` | Aller en une décision sur une case atteignable lointaine, par son trajet retenu, un `HeroMoved` par case (spec 036). Pas un terme du livret. |
| Case atteignable | `reachable` | Case dont le trajet retenu coûte au plus les points de mouvement disponibles. |

### Contenu du jeu en données (spec 029)

| Français | Code | Sens |
|---|---|---|
| Contenu du jeu | `GameContent` | Définitions des cartes, decks de départ, tuiles (puis ennemis, sites…), lues depuis les fichiers de contenu embarqués dans arcanum-core. Pas un terme du livret ; « catalogue » est déjà pris par les libellés. |
| Fichier de contenu | — | Fichier texte d'une famille de contenu, dans `crates/arcanum-core/content/`. |

### Sauvegarde et retour en arrière (spec 024)

| Livret (EN) | Français | Code | Sens |
|---|---|---|---|
| Reverting | Revenir en arrière | `GameRunner::undo` | Reprendre la dernière décision de son tour (MKUE p. 5). Pas « annuler », qui sert aux modes de choix. |
| — | Point de non-retour | `Event::blocks_undo` | Événement qui révèle une information, lance un dé ou ferme le tour : on ne revient pas avant. |
| — | Numéro de la décision | `decision` | Numéro, dans le journal, de la décision qui a produit un événement (0 = mise en place). |
| — | Sauvegarde | `SaveGame` | Fichier JSON : version, résumé, journal, positions des générateurs. |
| — | Résumé de la sauvegarde | `SaveSummary` | Scénario, héros, manche, tour, date : pour la liste, jamais relu pour jouer. |
| — | Sauvegarde automatique | `AUTOSAVE_NAME` | `auto.json`, écrite au début de chaque tour. |
| — | Dossier des sauvegardes | `saves_dir` | `ARCANUM_SAVE_DIR`, `saves/` du dépôt en développement, sinon dossier de données de l'utilisateur. |
| — | Usage du hasard | `RngUsage` | Famille de tirages qui partagent un générateur : Source, pioche de tuiles, deck d'un joueur, pile d'ennemis d'une couleur ; numéro de flux fixe (spec 038, ADR 0007). |
| — | Générateurs de la partie | `GameRngs` | Un générateur ChaCha8 par usage, dérivés de la graine, créés au premier tirage (spec 038). |
| — | Positions des générateurs | `rng_positions` | Pour chaque usage déjà tiré, où en est son générateur : la partie rechargée tire les mêmes dés et mélanges (spec 038). |

### Cartes du jeu de base (spec 050)

Noms français : ceux des cartes françaises photographiées (`../mage-knight/docs/regles/scans/fr/`) pour les actions avancées, les sorts, les artefacts (issue privée 313) et les unités. Reste une **proposition** à confirmer par Mathieu, faute de photo : le sort Mind Read / Mind Steal. Les noms anglais sont ceux imprimés sur les cartes ; les textes des effets sont dans `../mage-knight/docs/regles/cartes/`.

#### Actions avancées

| Livret/carte (EN) | Français | Code |
|---|---|---|
| Fire Bolt | Boule de feu | `AdvancedAction::FireBolt` |
| Ice Bolt | Boule de glace | `AdvancedAction::IceBolt` |
| Swift Bolt | Boule vivifiante | `AdvancedAction::SwiftBolt` |
| Crushing Bolt | Boule écrasante | `AdvancedAction::CrushingBolt` |
| Blood Rage | Coup de sang | `AdvancedAction::BloodRage` |
| Ice Shield | Bouclier de glace | `AdvancedAction::IceShield` |
| Agility | Agilité | `AdvancedAction::Agility` |
| Refreshing Walk | Marche revigorante | `AdvancedAction::RefreshingWalk` |
| Intimidate | Intimidation | `AdvancedAction::Intimidate` |
| Frost Bridge | Pont de glace | `AdvancedAction::FrostBridge` |
| Song of Wind | Mélodie du vent | `AdvancedAction::SongOfWind` |
| Path Finding | Chemins de traverse | `AdvancedAction::PathFinding` |
| Blood Ritual | Rituel de sang | `AdvancedAction::BloodRitual` |
| Pure Magic | Magie pure | `AdvancedAction::PureMagic` |
| Heroic Tale | Récit héroïque | `AdvancedAction::HeroicTale` |
| Regeneration | Régénération | `AdvancedAction::Regeneration` |
| Into the Heat | Au cœur du combat | `AdvancedAction::IntoTheHeat` |
| Steady Tempo | Temporisation | `AdvancedAction::SteadyTempo` |
| Diplomacy | Diplomatie | `AdvancedAction::Diplomacy` |
| In Need | Besoin d'aide | `AdvancedAction::InNeed` |
| Decompose | Décomposition | `AdvancedAction::Decompose` |
| Crystal Mastery | Maîtrise du cristal | `AdvancedAction::CrystalMastery` |
| Mana Storm | Tempête de mana | `AdvancedAction::ManaStorm` |
| Ambush | Embuscade | `AdvancedAction::Ambush` |
| Maximal Effect | Effet maximum | `AdvancedAction::MaximalEffect` |
| Magic Talent | Don pour la magie | `AdvancedAction::MagicTalent` |
| Learning | Apprentissage | `AdvancedAction::Learning` |
| Training | Entraînement | `AdvancedAction::Training` |

#### Sorts

Chaque sort a deux noms : l'effet de base, puis l'effet renforcé (mana noir, de nuit).

| Base (EN) | Base (FR) | Renforcé (EN) | Renforcé (FR) | Code |
|---|---|---|---|---|
| Fireball | Éclair de feu | Firestorm | Tempête de feu | `Spell::Fireball` |
| Snowstorm | Tempête de neige | Blizzard | Blizzard | `Spell::Snowstorm` |
| Expose | Exposition | Mass Expose | Exposition de masse | `Spell::Expose` |
| Tremor | Secousse | Earthquake | Tremblement de terre | `Spell::Tremor` |
| Flame Wall | Mur de flammes | Flame Wave | Déluge de flammes | `Spell::FlameWall` |
| Mana Bolt | Éclair de mana | Mana Thunderbolt | Tonnerre de mana | `Spell::ManaBolt` |
| Whirlwind | Tourbillon | Tornado | Tornade | `Spell::Whirlwind` |
| Underground Travel | Voyage souterrain | Underground Attack | Attaque souterraine | `Spell::UndergroundTravel` |
| Burning Shield | Bouclier ardent | Exploding Shield | Bouclier explosif | `Spell::BurningShield` |
| Chill | Frisson | Lethal Chill | Frisson fatal | `Spell::Chill` |
| Wings of Wind | Ailes du vent | Wings of Night | Ailes de la nuit | `Spell::WingsOfWind` |
| Restoration | Restauration | Rebirth | Renaissance | `Spell::Restoration` |
| Demolish | Démolition | Disintegrate | Désintégration | `Spell::Demolish` |
| Space Bending | Distorsion spatiale | Time Bending | Distorsion temporelle | `Spell::SpaceBending` |
| Call to Arms | Appel aux armes | Call to Glory | Appel à la gloire | `Spell::CallToArms` |
| Meditation | Méditation | Trance | Transe | `Spell::Meditation` |
| Mana Meltdown | Fuite de mana | Mana Radiance | Brûlure de mana | `Spell::ManaMeltdown` |
| Mana Claim | Puits de mana | Mana Curse | Malédiction de mana | `Spell::ManaClaim` |
| Mind Read | Lecture d'esprit *(proposition)* | Mind Steal | Vol d'esprit *(proposition)* | `Spell::MindRead` |
| Energy Flow | Flux d'énergie | Energy Steal | Vol d'énergie | `Spell::EnergyFlow` |

#### Artefacts

Noms des cartes françaises photographiées (`../mage-knight/docs/regles/scans/fr/artifacts/`, issue privée 313).

| Livret/carte (EN) | Français | Code |
|---|---|---|
| Banner of Glory | Bannière de gloire | `Artifact::BannerOfGlory` |
| Banner of Fear | Bannière de peur | `Artifact::BannerOfFear` |
| Banner of Protection | Bannière de protection | `Artifact::BannerOfProtection` |
| Banner of Courage | Bannière de courage | `Artifact::BannerOfCourage` |
| Ruby Ring | Anneau de rubis | `Artifact::RubyRing` |
| Sapphire Ring | Anneau de saphir | `Artifact::SapphireRing` |
| Diamond Ring | Anneau de diamant | `Artifact::DiamondRing` |
| Emerald Ring | Anneau d'émeraude | `Artifact::EmeraldRing` |
| Sword of Justice | Épée de justice | `Artifact::SwordOfJustice` |
| Horn of Wrath | Corne de la colère | `Artifact::HornOfWrath` |
| Endless Bag of Gold | Bourse sans fond | `Artifact::EndlessBagOfGold` |
| Endless Gem Pouch | Poche de mana | `Artifact::EndlessGemPouch` |
| Golden Grail | Graal doré | `Artifact::GoldenGrail` |
| Book of Wisdom | Livre de sagesse | `Artifact::BookOfWisdom` |
| Amulet of Sun | Amulette du soleil | `Artifact::AmuletOfSun` |
| Amulet of Darkness | Amulette des ténèbres | `Artifact::AmuletOfDarkness` |

#### Unités

| Livret/carte (EN) | Français | Code |
|---|---|---|
| Peasants | Paysans | `Unit::Peasants` |
| Herbalists | Herboristes | `Unit::Herbalists` |
| Foresters | Bûcherons | `Unit::Foresters` |
| Utem Crossbowmen | Arbalétriers d'Utem | `Unit::UtemCrossbowmen` |
| Utem Guardsmen | Gardes d'Utem | `Unit::UtemGuardsmen` |
| Utem Swordsmen | Guerriers d'Utem | `Unit::UtemSwordsmen` |
| Guardian Golems | Golems gardiens | `Unit::GuardianGolems` |
| Illusionists | Illusionnistes | `Unit::Illusionists` |
| Red Cape Monks | Moines du Cap Rouge | `Unit::RedCapeMonks` |
| Northern Monks | Moines du Nord | `Unit::NorthernMonks` |
| Savage Monks | Moines sauvages | `Unit::SavageMonks` |
| Fire Golems | Golem de feu | `Unit::FireGolems` |
| Ice Golems | Golem de glace | `Unit::IceGolems` |
| Fire Mages | Mages de feu | `Unit::FireMages` |
| Ice Mages | Mages de glace | `Unit::IceMages` |
| Amotep Gunners | Canonniers d'Amotep | `Unit::AmotepGunners` |
| Amotep Freezers | Glaceurs d'Amotep | `Unit::AmotepFreezers` |
| Catapults | Catapultes | `Unit::Catapults` |
| Altem Mages | Mages d'Altem | `Unit::AltemMages` |
| Altem Guardians | Gardiens d'Altem | `Unit::AltemGuardians` |
