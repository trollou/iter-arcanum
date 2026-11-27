use crate::board::counts_of;
use crate::{
    BasicColor, CardDestination, CardId, CardPlay, CombatPhase, CommonCard, CommonDeck, Effect,
    Element, EnemyColor, EnemyToken, GameConfig, GameEndReason, Hex, ManaColor, Offer, PlayerId,
    RuinToken, Site, Skill, TileDeckCounts, TileId, TilePosition, TimeOfDay, UnitCard, WoundPile,
};

mod cards;
mod combat;
mod dummy;
mod mana;
mod map;
mod progression;
mod sites;
mod turn;
mod units;

/// Les thèmes des événements : un fichier par thème, comme `engine/apply/`.
#[derive(Clone, Copy)]
enum Theme {
    Map,
    Cards,
    Mana,
    Turn,
    Dummy,
    Combat,
    Units,
    Sites,
    Progression,
}

/// D'où une carte action est détruite (spec 056).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum DestroyedFrom {
    Hand,
    PlayArea,
}

/// Version du format des sauvegardes (journal compris). Change au plus une fois par version
/// publiée, avec une migration (spec #52, `docs/architecture.md`, « Ajouter un événement »).
pub const EVENT_FORMAT_VERSION: u16 = 1;

/// Fait accompli, hasard déjà résolu.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Event {
    GameStarted {
        config: GameConfig,
    },
    /// Tuile posée sans venir de la pioche : la tuile de départ.
    TilePlaced {
        tile: TileId,
        position: TilePosition,
    },
    /// Pioche de tuiles formée, hasard résolu ; la première est le dessus.
    TileDeckFormed {
        order: Vec<TileId>,
    },
    /// Tuile prise dans la pioche et posée.
    TileRevealed {
        tile: TileId,
        position: TilePosition,
    },
    HeroFigurePlaced {
        player: PlayerId,
        hex: Hex,
    },
    DeckShuffled {
        player: PlayerId,
        order: Vec<CardId>,
    },
    CardDrawn {
        player: PlayerId,
        card: CardId,
    },
    /// Carte jouée de la main vers la zone de jeu ; porte l'effet déjà résolu (spec 013).
    CardPlayed {
        player: PlayerId,
        card: CardId,
        play: CardPlay,
        effect: Effect,
    },
    /// Carte défaussée de la main : pour payer un effet (MKUE p. 4, 6) ou avant de
    /// repiocher à la fin du tour (MKUE p. 11, 7.a).
    CardDiscarded {
        player: PlayerId,
        card: CardId,
    },
    /// Figurine déplacée d'une case ; porte le coût déjà calculé (spec 014).
    HeroMoved {
        player: PlayerId,
        from: Hex,
        to: Hex,
        cost: u8,
    },
    /// Exploration payée ; la tuile est posée par le `TileRevealed` qui suit (spec 015).
    Explored {
        player: PlayerId,
        position: TilePosition,
        cost: u8,
    },
    /// Fin du tour : la zone de jeu va dans la défausse, les points sont perdus (spec 016).
    TurnEnded {
        player: PlayerId,
    },
    /// Début du tour numéro `turn` de la manche (spec 016).
    TurnStarted {
        player: PlayerId,
        turn: u32,
    },
    /// Source lancée, relances comprises : seul le résultat compte (spec 020).
    SourceRolled {
        dice: Vec<ManaColor>,
    },
    /// Cristal gagné dans l'inventaire : à la fin du tour sur une mine (spec 021).
    CrystalGained {
        player: PlayerId,
        color: BasicColor,
    },
    /// Dé de la Source pris pour payer un effet ; il reste à sa place jusqu'à la fin du tour
    /// (spec 022).
    DieTaken {
        player: PlayerId,
        die: u8,
        color: ManaColor,
    },
    /// Cristal de l'inventaire dépensé pour payer un effet (spec 022).
    CrystalUsed {
        player: PlayerId,
        color: BasicColor,
    },
    /// Dé pris relancé et rendu à la Source à la fin du tour ; `color` est le résultat
    /// (spec 022).
    DieReturned {
        player: PlayerId,
        die: u8,
        color: ManaColor,
    },
    /// Jeton de mana gagné dans la zone de jeu (spec 025).
    ManaTokenGained {
        player: PlayerId,
        color: ManaColor,
    },
    /// Jeton de mana dépensé pour payer un effet (spec 025).
    ManaTokenUsed {
        player: PlayerId,
        color: ManaColor,
    },
    /// Carte de la zone de jeu reprise en main à la fin du tour (Euphorie cristalline, spec 025).
    CardReturnedToHand {
        player: PlayerId,
        card: CardId,
    },
    /// Dé de la Source pris et mis sur `color` ; il ne sera pas relancé (Source de mana et
    /// Attraction de mana renforcées, spec 025).
    DieSet {
        player: PlayerId,
        die: u8,
        color: BasicColor,
    },
    /// Fin de la manche `round`, annoncée deck vide (spec 023) : plus de décision jusqu'à la
    /// manche suivante, ni après la dernière.
    RoundEnded {
        round: u32,
    },
    /// Début de la manche `round` : le plateau Jour/Nuit a été retourné, `time_of_day` est
    /// déjà calculé (spec 023).
    RoundStarted {
        round: u32,
        time_of_day: TimeOfDay,
    },
    /// Pile d'ennemis de `color` mélangée à la mise en place, le dessus en premier (spec 030).
    EnemyPileShuffled {
        color: EnemyColor,
        order: Vec<EnemyToken>,
    },
    /// Pile de `color` vide, refaite de sa défausse mélangée, le dessus en premier : la défausse
    /// est vidée (spec 031, MKUE p. 3). Suit toujours un `TileRevealed`.
    EnemyPileReshuffled {
        color: EnemyColor,
        order: Vec<EnemyToken>,
    },
    /// Jeton pris sur le dessus de sa pile et posé face visible sur `hex` (spec 030).
    EnemyTokenPlaced {
        token: EnemyToken,
        hex: Hex,
    },
    /// Défenseur d'une forteresse ou d'une tour de mage : jeton pris sur le dessus de sa pile et
    /// posé face cachée sur `hex` (cartes de site Keep et Mage Tower, *When Revealed* ; spec 043).
    DefenderPlaced {
        token: EnemyToken,
        hex: Hex,
    },
    /// Une cité entre en jeu à la révélation de sa tuile, avec sa couleur et le niveau que lui
    /// donne le scénario (MKUE p. 7, 5.g ; spec 062) ; ses défenseurs suivent (`DefenderPlaced`).
    CityPlaced {
        hex: Hex,
        color: crate::BasicColor,
        level: u8,
    },
    /// Le défenseur face cachée de `hex` est retourné : c'est `token` (cartes de site ; spec 043).
    EnemyTokenRevealed {
        hex: Hex,
        token: EnemyToken,
    },
    /// Le dernier pas entre dans la forteresse ou la tour de mage non conquise de `site`, depuis
    /// `from` : c'est un assaut, l'action du tour (MKUE p. 7, 4.a ; spec 043). Points de mouvement
    /// et d'influence perdus, déplacement terminé, le défenseur à combattre.
    AssaultStarted {
        player: PlayerId,
        site: Hex,
        from: Hex,
    },
    /// Ennemis errants provoqués par le dernier pas (cases de leurs jetons) : le déplacement est
    /// terminé (spec 030).
    EnemiesProvoked {
        player: PlayerId,
        enemies: Vec<Hex>,
    },
    /// L'ennemi errant de cette case rejoint les ennemis à combattre ; le déplacement est terminé
    /// (spec 031).
    EnemyChallenged {
        player: PlayerId,
        enemy: Hex,
    },
    /// Combat engagé contre ces ennemis (case et jeton), dans l'ordre (spec 031).
    CombatStarted {
        player: PlayerId,
        enemies: Vec<(Hex, EnemyToken)>,
    },
    /// Le combat passe à cette phase (spec 031).
    CombatPhaseStarted {
        player: PlayerId,
        phase: CombatPhase,
    },
    /// Combat terminé : plus de combat ce tour (spec 031).
    CombatEnded {
        player: PlayerId,
    },
    /// Attaque en cours sur ces cibles (jetons, spec 044), dans l'ordre du combat (spec 031).
    AttackDeclared {
        player: PlayerId,
        targets: Vec<EnemyToken>,
    },
    /// Attaque réussie : ces ennemis sont vaincus et rapportent `fame` en tout (spec 031).
    EnemiesDefeated {
        player: PlayerId,
        enemies: Vec<EnemyToken>,
        fame: u32,
    },
    /// Attaque ratée : rien n'est vaincu, les cibles ne peuvent plus être visées dans cette phase
    /// (spec 031, Q10).
    AttackFailed {
        player: PlayerId,
    },
    /// Parade en cours contre cet ennemi, désigné par son jeton (specs 031 et 044).
    BlockDeclared {
        player: PlayerId,
        target: EnemyToken,
    },
    /// Parade réussie : cet ennemi est paré, il n'infligera pas de dégâts (spec 031).
    EnemyBlocked {
        player: PlayerId,
        enemy: EnemyToken,
    },
    /// Parade ratée : l'ennemi visé ne peut plus être paré dans ce combat (spec 031, Q10).
    BlockFailed {
        player: PlayerId,
    },
    /// Les Blessures (cartes) infligées par cet ennemi vont dans la main (spec 031).
    WoundsTaken {
        player: PlayerId,
        enemy: EnemyToken,
        wounds: Vec<CardId>,
    },
    /// Une Blessure de la pile en main, prise par la carte jouée (Coup de sang, spec 053) : elle
    /// compte dans les Blessures du combat.
    WoundFromCard {
        player: PlayerId,
        card: CardId,
        wound: CardId,
    },
    /// Attaque toxique : une Blessure de plus dans la défausse par Blessure mise en main (spec 033).
    PoisonWoundsDiscarded {
        player: PlayerId,
        enemy: EnemyToken,
        wounds: Vec<CardId>,
    },
    /// Attaque paralysante : ces cartes, toute la main sauf les Blessures, vont dans la défausse
    /// (spec 033).
    HandParalyzed {
        player: PlayerId,
        cards: Vec<CardId>,
    },
    /// Au début de la phase de parade, l'Invocateur `summoner` tire cette créature
    /// (spec 033).
    EnemySummoned {
        player: PlayerId,
        summoner: EnemyToken,
        token: EnemyToken,
    },
    /// La créature invoquée par l'Invocateur `summoner` va dans la défausse de sa couleur
    /// (spec 033).
    SummonedEnemyDiscarded {
        player: PlayerId,
        summoner: EnemyToken,
    },
    /// Au début de la phase de parade, l'invocation attend (`held`) que la Bannière de terreur
    /// puisse l'annuler avant le tirage, ou reprend son cours (issue #8 ; Compendium, *Combat*
    /// S23).
    SummonsHeld {
        player: PlayerId,
        held: bool,
    },
    /// Un Soin jette ces Blessures de la main : elles retournent à la pile de blessures (spec 034).
    WoundsHealed {
        player: PlayerId,
        wounds: Vec<CardId>,
    },
    /// Le héros est assommé : ces cartes, toute la main sauf les Blessures (peut-être aucune),
    /// vont dans la défausse (spec 034).
    KnockedOut {
        player: PlayerId,
        cards: Vec<CardId>,
    },
    /// Changement de niveau à la fin du tour, un par ligne de la piste de gloire franchie : le
    /// héros passe au niveau `level` (spec 035).
    LevelGained {
        player: PlayerId,
        level: u8,
    },
    /// Gloire gagnée hors combat : une tuile révélée dans « Première reconnaissance » (spec 063).
    FameGained {
        player: PlayerId,
        fame: u32,
    },
    /// Une cité a été révélée : le tour qui suit celui-ci est le dernier (MKUE p. 18 ; spec 063).
    LastTurnAnnounced {
        player: PlayerId,
    },
    /// La partie est terminée : plus aucune décision (spec 063).
    /// Pioche commune mélangée, le dessus en premier (spec 050).
    CommonDeckShuffled {
        deck: CommonDeck,
        order: Vec<CommonCard>,
    },
    /// La carte du dessus de sa pioche entre dans l'offre : en première place pour les actions
    /// avancées et les sorts, à la suite pour les unités (spec 050).
    OfferCardDealt {
        offer: Offer,
        card: CommonCard,
    },
    /// La carte quitte l'offre et va sous sa pioche (MKUE p. 4, 2.c à 2.e ; spec 050).
    OfferCardReturned {
        offer: Offer,
        card: CommonCard,
    },
    /// Les deux compétences du dessus de la pile révélées à un niveau pair, en attente du choix
    /// (MKUE p. 11, 6.b ; spec 051).
    SkillsRevealed {
        player: PlayerId,
        skills: Vec<Skill>,
    },
    /// La compétence gagnée, face visible devant le joueur (MKUE p. 5, *Using Skills* 2.a).
    SkillGained {
        player: PlayerId,
        skill: Skill,
    },
    /// La compétence prise dans l'offre commune : l'action avancée sera la dernière (spec 051).
    SkillTakenFromCommonOffer {
        player: PlayerId,
        skill: Skill,
    },
    /// La compétence révélée non gardée, posée dans l'offre commune (spec 051).
    SkillToCommonOffer {
        skill: Skill,
    },
    /// La compétence utilisée, avec l'effet de l'option au moment de la journée (spec 051) :
    /// utilisée ce tour, ou retournée si elle sert une fois par manche.
    SkillUsed {
        player: PlayerId,
        skill: Skill,
        option: u8,
        effect: Effect,
    },
    /// Une compétence coopérative posée au centre (MKUE p. 5, *Using Skills* 4.b ; spec 058) : son
    /// avantage au tour suivant du joueur, avec la couleur de Surcharge de mana.
    SkillPlacedInCenter {
        player: PlayerId,
        skill: Skill,
        color: Option<ManaColor>,
    },
    /// L'avantage du centre de cette compétence, consommé (spec 058).
    CenterAdvantageUsed {
        player: PlayerId,
        skill: Skill,
    },
    /// À la fin du tour suivant, la compétence posée au centre revient retournée, son avantage
    /// perdu (MKUE p. 4, 2.g ; Compendium, *Skill Tokens* S4, S5 ; spec 058).
    CenterSkillReturned {
        player: PlayerId,
        skill: Skill,
    },
    /// L'unité recrutée va sous le jeton de Loyauté sans faille (spec 058).
    UnitUnderBonds {
        player: PlayerId,
        unit: UnitCard,
    },
    /// Le dé pris avec l'avantage d'Embrasure de la Source sera rendu tel quel à la fin du tour
    /// (spec 058).
    SourceDieKept {
        player: PlayerId,
        die: u8,
    },
    /// Botte secrète sur cet ennemi : Armure −`by` (ses résistances au moment de l'utiliser),
    /// jamais sous 1, pour le combat (spec 058).
    ResistanceBroken {
        player: PlayerId,
        enemy: EnemyToken,
        by: u8,
    },
    /// Le joueur gagne la carte de l'offre : sur le dessus de son deck (MKUE p. 5, *"Gain"
    /// Effects* 3 ; spec 050).
    OfferCardGained {
        player: PlayerId,
        offer: Offer,
        card: CommonCard,
        /// Où va la carte : le dessus du deck, sauf quand la carte qui la fait gagner dit autre
        /// chose (spec 056).
        #[serde(default)]
        to: CardDestination,
    },
    /// La réputation change de `change`, bornée à la piste, aussitôt (spec 040, Q3).
    ReputationChanged {
        player: PlayerId,
        change: i8,
    },
    /// Le héros interagit avec les habitants de ce site : mouvement non dépensé perdu, puis le
    /// bonus de réputation s'ajoute à l'influence déjà jouée, gardée par variante maison
    /// (MKUE p. 6, 6.e ; p. 8, 2.a ; spec 040 ; issue #95).
    InteractionStarted {
        player: PlayerId,
        site: Site,
        bonus: i8,
        /// Spec 062 : les Boucliers du joueur sur la cité, ajoutés à l'influence (MKUE p. 8, 3.a).
        #[serde(default)]
        city_shields: u8,
    },
    /// Unité recrutée : elle quitte l'offre (sans la compléter) et entre dans la zone des unités,
    /// disponible et non blessée ; l'influence baisse de `cost` (MKUE p. 8, 4.a ; spec 042).
    UnitRecruited {
        player: PlayerId,
        unit: UnitCard,
        cost: u8,
    },
    /// Spec 055 : une bannière de la main assignée à une unité (MKUE p. 5, 8).
    BannerAssigned {
        player: PlayerId,
        card: CardId,
        unit: UnitCard,
    },
    /// Spec 055 : la bannière d'une unité va dans la défausse : l'unité détruite ou renvoyée, ou une
    /// autre bannière assignée (MKUE p. 5, 8.a).
    BannerLost {
        player: PlayerId,
        card: CardId,
        unit: UnitCard,
    },
    /// Spec 055 (MKUE p. 5, 8.b ; Q2) : à la fin de la manche, ces bannières quittent leurs unités
    /// pour le deck, avant son mélange ; les bannières retournées se remettent face visible.
    BannersReclaimed {
        player: PlayerId,
        banners: Vec<CardId>,
    },
    /// Spec 055 : un mana payé par la face renforcée d'un anneau (mana illimité de sa couleur ou
    /// noir) ; rien n'est retiré.
    EndlessManaUsed {
        player: PlayerId,
        color: ManaColor,
    },
    /// Spec 055 : la bannière de l'unité a servi. Terreur : l'unité épuisée, `enemy` n'attaque pas
    /// ce combat ; Courage : la bannière retournée.
    BannerUsed {
        player: PlayerId,
        unit: UnitCard,
        enemy: Option<EnemyToken>,
    },
    /// Spec 055 : un dé de mana lancé par l'artefact `card` (Corne de la colère, Poche de mana) ;
    /// ce qu'il donne suit dans ses événements.
    ArtifactDieRolled {
        player: PlayerId,
        card: CardId,
        color: ManaColor,
    },
    /// Unité renvoyée pour en recruter une autre : elle quitte la partie (MKUE p. 5, 4.a).
    UnitDisbanded {
        player: PlayerId,
        unit: UnitCard,
    },
    /// Unité activée pour l'option `option` de sa capacité `ability`, d'effet `effect` : elle est
    /// épuisée, l'effet compte comme celui d'une carte (MKUE p. 5, 5 ; spec 042).
    UnitActivated {
        player: PlayerId,
        unit: UnitCard,
        ability: u8,
        option: u8,
        effect: Effect,
        /// L'ennemi du combat visé par la capacité (spec 053).
        #[serde(default)]
        enemy: Option<EnemyToken>,
    },
    /// L'unité redevient disponible (MKUE p. 5, 6 ; spec 042).
    UnitReadied {
        player: PlayerId,
        unit: UnitCard,
    },
    /// L'unité reçoit des Blessures hors dégâts (Épéistes d'Utem ; spec 042).
    UnitWounded {
        player: PlayerId,
        unit: UnitCard,
        wounds: u8,
    },
    /// Les dégâts de l'ennemi de `enemy` assignés à cette unité : `wounds` Blessures (0 à 2) ;
    /// les dégâts restants baissent de son armure (MKUE p. 9, *Assign Damage Phase* 4 ; spec 042).
    DamageAssignedToUnit {
        player: PlayerId,
        enemy: EnemyToken,
        unit: UnitCard,
        wounds: u8,
    },
    /// L'unité est détruite (paralysant, MKUE p. 10, 6.b) : elle quitte la partie.
    UnitDestroyed {
        player: PlayerId,
        unit: UnitCard,
    },
    /// Une Blessure de l'unité soignée pour `cost` points de soin (MKUE p. 11, 3.c ; spec 042).
    UnitHealed {
        player: PlayerId,
        unit: UnitCard,
        cost: u8,
    },
    /// Nouvelle manche : toutes les unités redeviennent disponibles (MKUE p. 4, 2.g ; spec 042).
    UnitsReadied {
        player: PlayerId,
    },
    /// Un point de soin acheté pour `cost` influence pendant une interaction (MKUE p. 8, 4.b ;
    /// spec 041).
    HealingBought {
        player: PlayerId,
        cost: u8,
    },
    /// Action avancée achetée au monastère : elle quitte l'offre d'unités, sans la compléter, pour
    /// le dessus du deck ; l'influence baisse de `cost` (MKUE p. 8, 4.c ; spec 041).
    AdvancedActionBought {
        player: PlayerId,
        card: CardId,
        cost: u8,
    },
    /// Spec 062 : dans la cité verte, la carte du dessus de la pioche d'actions avancées passe sur
    /// le dessus du deck ; l'influence baisse de `cost` (MKUE p. 8, 4.e).
    AdvancedActionDrawnBlind {
        player: PlayerId,
        card: CardId,
        cost: u8,
    },
    /// Spec 062 : dans la cité rouge, un artefact acheté, une récompense artefact de plus pour la
    /// fin du tour ; l'influence baisse de `cost` (MKUE p. 8, 4.e ; p. 11, 5.b).
    ArtifactBought {
        player: PlayerId,
        cost: u8,
    },
    /// Spec 062 : dans la cité blanche, la carte du dessus des unités d'élite rejoint l'offre
    /// d'unités ; l'influence baisse de `cost` (MKUE p. 8, 4.e).
    EliteUnitAdded {
        player: PlayerId,
        card: CommonCard,
        cost: u8,
    },
    /// Sort acheté à une tour de mage conquise : il quitte l'offre de sorts pour le dessus du
    /// deck ; l'influence baisse de `cost` (MKUE p. 8, 4.d ; spec 043).
    SpellBought {
        player: PlayerId,
        card: CardId,
        cost: u8,
    },
    /// Incendie du monastère de `hex` commencé : points non dépensés perdus, plus de déplacement
    /// ni de repos, le combat qui suit exclut les unités (carte du monastère ; spec 041).
    MonasteryBurnStarted {
        player: PlayerId,
        hex: Hex,
    },
    /// Ennemi du monastère vaincu : un jeton Bouclier du joueur sur `hex`, le monastère est
    /// incendié, une récompense artefact gagnée (carte du monastère ; spec 041).
    MonasteryBurned {
        player: PlayerId,
        hex: Hex,
    },
    /// Le jeton `token` de `hex` quitte le plateau de jeu pour la défausse de sa couleur, sans être vaincu
    /// (ennemi d'un incendie raté ; FAQ p3 ; spec 041).
    EnemyTokenDiscarded {
        hex: Hex,
        token: EnemyToken,
    },
    /// Défenseur vaincu : un jeton Bouclier du joueur sur le site de `hex`, conquis ; une tour de
    /// mage donne une récompense sort (MKUE p. 10, *Combat Outcomes* 2.c ; spec 043).
    SiteConquered {
        player: PlayerId,
        hex: Hex,
    },
    /// Spec 062 : à la fin du combat d'un assaut de cité, un Bouclier du joueur posé sur la cité
    /// pour le défenseur `enemy` vaincu, au bout de la rangée (MKUE p. 10, *Combat Outcomes* 1.a).
    CityShieldPlaced {
        player: PlayerId,
        hex: Hex,
        enemy: EnemyToken,
    },
    /// Spec 062 : plus aucun défenseur, la cité est conquise ; `leader` en est le Seigneur (MKUE
    /// p. 10, 2.c et *City Assaults* 2).
    CityConquered {
        hex: Hex,
        leader: PlayerId,
    },
    /// Assaut repoussé : la figurine revient de `from` (le site) sur `to`, sans coût (MKUE p. 10,
    /// *Combat Outcomes* 3.d ; spec 043).
    HeroWithdrew {
        player: PlayerId,
        from: Hex,
        to: Hex,
    },
    /// Artefacts tirés du dessus de la pioche pour une récompense : ils attendent le choix ; une
    /// récompense de moins (MKUE p. 11, *End of the Turn* 5.b ; spec 041).
    ArtifactsDrawn {
        player: PlayerId,
        cards: Vec<CardId>,
    },
    /// L'artefact pris va sur le dessus du deck (spec 041).
    ArtifactTaken {
        player: PlayerId,
        card: CardId,
    },
    /// L'artefact tiré non pris va sous la pioche d'artefacts (spec 041).
    ArtifactReturned {
        card: CardId,
    },
    /// Village pillé au début du tour : plus de pillage ce tour (carte du village ; spec 041).
    VillagePlundered {
        player: PlayerId,
    },
    /// Mouvement dépensé dans l'attaque en cours (Agilité, spec 052) : 1 point pour Attaque 1
    /// physique, ou 2 pour Attaque à distance 1 (`ranged`, phase d'attaque à distance).
    MoveSpentAsAttack {
        player: PlayerId,
        ranged: bool,
    },
    /// Influence dépensée dans la parade en cours (Diplomatie, spec 052) : 1 point pour Parade 1
    /// de cet élément.
    InfluenceSpentAsBlock {
        player: PlayerId,
        element: Element,
    },
    /// Retraite forcée à la fin du tour (MKUE p. 11, *End of the Turn* 2 ; spec 052) : la figurine
    /// revient sur ses pas par les cases de `path`, jusqu'à la dernière, et prend ces Blessures en
    /// main, une par case.
    ForcedWithdrawal {
        player: PlayerId,
        path: Vec<Hex>,
        wounds: Vec<CardId>,
    },
    /// Temporisation gardée à la fin du tour (spec 052) : la carte quitte la zone de jeu pour le
    /// dessus (`top`) ou le dessous du deck.
    SteadyTempoKept {
        player: PlayerId,
        card: CardId,
        top: bool,
    },
    /// Le héros entre dans le site d'aventure de `hex` : l'action du tour, points de mouvement et
    /// d'influence perdus, déplacement terminé ; le combat suit (spec 044).
    AdventureSiteEntered {
        player: PlayerId,
        hex: Hex,
    },
    /// Un dé de récompense lancé de la réserve, pas de la Source (Compendium, *Mana* S14 ;
    /// spec 044) : au donjon conquis, or ou noir donnent une récompense sort, sinon artefact ; en
    /// fin de tour, un cristal aléatoire.
    RewardDieRolled {
        player: PlayerId,
        color: ManaColor,
    },
    /// Une Blessure de `from` jetée sur une clairière magique, rendue à la pile de Blessures : pas un
    /// soin (carte de la clairière ; spec 044).
    WoundThrownAway {
        player: PlayerId,
        from: WoundPile,
    },
    /// Pile des jetons de ruines mélangée à la mise en place, le dessus en premier (spec 045).
    RuinPileShuffled {
        order: Vec<RuinToken>,
    },
    /// Pile de compétences du héros mélangée à la mise en place, le dessus en premier (MKUE p. 3 ;
    /// spec 051).
    SkillPileShuffled {
        player: PlayerId,
        order: Vec<Skill>,
    },
    /// Le jeton du dessus de la pile des jetons de ruines posé sur la ruine de `hex`, face visible
    /// de jour, face cachée de nuit (carte *Ancient Ruins*, *When Revealed* ; spec 045).
    RuinTokenPlaced {
        token: RuinToken,
        hex: Hex,
        face_up: bool,
    },
    /// Le jeton de ruines face cachée de `hex` est retourné : c'est `token` (carte *Ancient Ruins* ;
    /// spec 045).
    RuinTokenRevealed {
        hex: Hex,
        token: RuinToken,
    },
    /// Offrande payée à l'autel de `hex` : l'action du tour, points de mouvement et d'influence
    /// perdus, déplacement terminé ; le mana payé suit (carte *Ancient Ruins* ; spec 045).
    AltarTributePaid {
        player: PlayerId,
        hex: Hex,
    },
    /// L'unité de récompense quitte l'offre d'unités pour la zone du joueur, disponible ; l'offre
    /// n'est pas complétée (MKUE p. 11, 5.d ; spec 045).
    UnitRewardTaken {
        player: PlayerId,
        unit: UnitCard,
    },
    /// Le joueur renonce à la récompense unité (spec 045).
    UnitRewardDeclined {
        player: PlayerId,
    },
    /// Une carte action quitte le jeu (MKUE p. 5, *Discarding and Throwing Away* 2.b ; spec 056) :
    /// de la main (Décomposition, Entraînement) ou de la zone de jeu (cible d'Effet maximum).
    CardDestroyed {
        player: PlayerId,
        card: CardId,
        from: DestroyedFrom,
    },
    /// Le dé de la Source à cette position est relancé et y reste (Tempête de mana, spec 056).
    DieRerolled {
        die: u8,
        color: ManaColor,
    },
    /// Le joueur refuse le cristal de Rituel de sang renforcé (spec 056).
    CrystalDeclined {
        player: PlayerId,
    },
    /// Influence dépensée pour une action avancée par Apprentissage ; le modificateur est retiré
    /// (spec 056). La carte gagnée suit (`OfferCardGained`).
    AdvancedActionLearned {
        player: PlayerId,
        card: CardId,
        cost: u8,
    },
    /// Les cristaux dépensés ce tour reviennent dans l'inventaire (Maîtrise du cristal, spec 056).
    CrystalsReturned {
        player: PlayerId,
        colors: Vec<BasicColor>,
    },
    /// Distorsion temporelle : les cartes de la zone de jeu reviennent en main au lieu d'aller dans la défausse (spec 057).
    CardsReturnedToHand {
        player: PlayerId,
        cards: Vec<CardId>,
    },
    /// Cette carte de la zone de jeu est écartée jusqu'à la fin de la manche (Distorsion temporelle, spec 057).
    CardSetAside {
        player: PlayerId,
        card: CardId,
    },
    /// Le même joueur joue aussitôt un tour supplémentaire, sans pioche (Distorsion temporelle, spec 057).
    ExtraTurnStarted {
        player: PlayerId,
    },
    /// Les cartes tirées au hasard dans la défausse par Méditation, qui attendent leur place (spec 057).
    DiscardPicked {
        player: PlayerId,
        cards: Vec<CardId>,
    },
    /// Les cartes de Méditation remises de la défausse sur le deck (`top`) ou dessous, la première la plus haute (spec 057).
    CardsPlacedOnDeck {
        player: PlayerId,
        top: bool,
        order: Vec<CardId>,
    },
    /// La pioche a profité des cartes en plus de Méditation (spec 057).
    DrawBonusUsed {
        player: PlayerId,
    },
    /// Le dé de la Source gardé par Puits de mana jusqu'à la fin de la manche, par sa position (spec 057).
    DieKept {
        player: PlayerId,
        die: u8,
    },
    /// Cet ennemi est détruit par un sort : vaincu comme par une attaque (gloire), une créature
    /// invoquée sans gloire (spec 054, Q2).
    EnemyDestroyed {
        player: PlayerId,
        enemy: EnemyToken,
    },
    /// L'attaque différée du Bouclier ardent ajoutée à l'attaque en cours (spec 054).
    DeferredAttackUsed {
        player: PlayerId,
        amount: u8,
        element: Element,
    },
    /// Le joueur prend cette tactique pour la manche (spec 060).
    TacticChosen {
        player: PlayerId,
        tactic: crate::Tactic,
    },
    /// Ces tactiques sont retirées de la partie, à la fin de la manche (spec 060).
    TacticsRemoved {
        tactics: Vec<crate::Tactic>,
    },
    /// Le deck remélangé pendant la manche, dans cet ordre, sans toucher à la main (spec 060) : les
    /// cartes de la défausse qui y sont la quittent (Faux départ).
    DeckReshuffled {
        player: PlayerId,
        order: Vec<CardId>,
    },
    /// Préparation : cette carte passe du deck à la main (spec 060).
    CardSearched {
        player: PlayerId,
        card: CardId,
    },
    /// Vol de mana : ce dé quitte la Source et se pose sur la tactique (spec 060).
    DieStolen {
        player: PlayerId,
        die: u8,
    },
    /// Le dé volé paie un mana ; la tactique est retournée (spec 060).
    StolenDieUsed {
        player: PlayerId,
    },
    /// Au bon moment annoncé : un tour de plus après celui-ci ; la tactique est retournée (spec 060).
    RightMomentAnnounced {
        player: PlayerId,
    },
    /// Longue Nuit : ces cartes, tirées au hasard, passent de la défausse au deck, dans cet ordre ;
    /// la tactique est retournée (spec 060).
    LongNightUsed {
        player: PlayerId,
        cards: Vec<CardId>,
    },
    /// Méditation nocturne : ces cartes passent de la main au deck ; la tactique est retournée
    /// (spec 060). Le `DeckReshuffled` qui suit les mélange.
    CardsShuffledIntoDeck {
        player: PlayerId,
        cards: Vec<CardId>,
    },
    /// Provisions : la carte du dessus du deck passe sous la tactique, face cachée (spec 060).
    CardStored {
        player: PlayerId,
        card: CardId,
    },
    /// Provisions retournée : les cartes posées dessous vont en main (spec 060).
    StoredCardsTaken {
        player: PlayerId,
    },
    /// Appel du mana utilisé ce tour ; les `DieRerolled` suivent (spec 060).
    ManaSearchUsed {
        player: PlayerId,
    },
    /// La pioche de fin de tour s'arrête, deck épuisé, `missing` cartes avant la limite : Longue
    /// Nuit est proposée (spec 060, Q5).
    DrawStopped {
        player: PlayerId,
        missing: usize,
    },
    /// Le dé volé, relancé, revient à la Source à la fin du tour où il a servi (spec 060).
    StolenDieReturned {
        player: PlayerId,
        color: ManaColor,
    },
    /// Le joueur fantôme entre en jeu (spec 061) : son héros, ses cristaux (un par cercle), sa
    /// pile de compétences et son deck mélangés, le dessus en premier.
    DummySetUp {
        hero: crate::Hero,
        crystals: Vec<BasicColor>,
        skills: Vec<Skill>,
        deck: Vec<CardId>,
    },
    /// Le fantôme tire sa tactique parmi celles qui restent (spec 061).
    DummyTacticDrawn {
        tactic: crate::Tactic,
    },
    /// Le tour du fantôme : ces cartes passent du dessus de son deck à sa défausse (spec 061).
    DummyTurnPlayed {
        cards: Vec<CardId>,
    },
    /// Deck vide au début de son tour, le fantôme annonce la fin de la manche (spec 061).
    DummyAnnouncedEndOfRound,
    /// L'action avancée en dernière place quitte l'offre pour le deck du fantôme (spec 061).
    DummyCardAdded {
        card: CardId,
    },
    /// Le fantôme gagne un cristal de la couleur du sort en dernière place (spec 061).
    DummyCrystalGained {
        color: BasicColor,
    },
    /// Deck et défausse du fantôme réunis et mélangés, dans cet ordre (spec 061).
    DummyDeckShuffled {
        order: Vec<CardId>,
    },
    /// La compétence du dessus de la pile du fantôme va dans l'offre commune (spec 061).
    DummySkillRevealed {
        skill: Skill,
    },
    GameEnded {
        reason: GameEndReason,
    },
}

impl Event {
    /// Point de non-retour (spec 024, MKUE p. 5, Reverting) : l'événement révèle une information,
    /// lance un dé ou ferme le tour ; on ne revient pas en arrière avant lui. Classé dans le fichier
    /// de son thème (`theme`) : chaque nouvel événement doit y être classé par sa spec.
    pub fn blocks_undo(&self) -> bool {
        match self.theme() {
            Theme::Map => map::blocks_undo(self),
            Theme::Cards => cards::blocks_undo(self),
            Theme::Mana => mana::blocks_undo(self),
            Theme::Turn => turn::blocks_undo(self),
            Theme::Dummy => dummy::blocks_undo(self),
            Theme::Combat => combat::blocks_undo(self),
            Theme::Units => units::blocks_undo(self),
            Theme::Sites => sites::blocks_undo(self),
            Theme::Progression => progression::blocks_undo(self),
        }
    }

    /// Le thème de l'événement, dont le fichier le classe et le filtre. Aiguillage exhaustif, comme
    /// `apply_event` (`engine/apply/mod.rs`) : un nouvel événement doit être rangé dans un thème.
    fn theme(&self) -> Theme {
        match self {
            Event::GameStarted { .. }
            | Event::TilePlaced { .. }
            | Event::TileDeckFormed { .. }
            | Event::TileRevealed { .. }
            | Event::HeroFigurePlaced { .. }
            | Event::HeroMoved { .. }
            | Event::Explored { .. }
            | Event::EnemyPileShuffled { .. }
            | Event::EnemyPileReshuffled { .. }
            | Event::EnemyTokenPlaced { .. }
            | Event::DefenderPlaced { .. }
            | Event::CityPlaced { .. }
            | Event::EnemyTokenRevealed { .. }
            | Event::LastTurnAnnounced { .. }
            | Event::GameEnded { .. }
            | Event::HeroWithdrew { .. }
            | Event::EnemyTokenDiscarded { .. }
            | Event::ForcedWithdrawal { .. }
            | Event::RuinPileShuffled { .. }
            | Event::RuinTokenPlaced { .. }
            | Event::RuinTokenRevealed { .. } => Theme::Map,
            Event::DeckShuffled { .. }
            | Event::CardDrawn { .. }
            | Event::CardPlayed { .. }
            | Event::CardDiscarded { .. }
            | Event::CardReturnedToHand { .. }
            | Event::DeckReshuffled { .. }
            | Event::WoundsHealed { .. }
            | Event::SteadyTempoKept { .. }
            | Event::WoundThrownAway { .. }
            | Event::CardDestroyed { .. }
            | Event::CardsReturnedToHand { .. }
            | Event::CardSetAside { .. }
            | Event::DiscardPicked { .. }
            | Event::CardsPlacedOnDeck { .. }
            | Event::DrawBonusUsed { .. } => Theme::Cards,
            Event::SourceRolled { .. }
            | Event::CrystalGained { .. }
            | Event::DieTaken { .. }
            | Event::CrystalUsed { .. }
            | Event::DieReturned { .. }
            | Event::ManaTokenGained { .. }
            | Event::ManaTokenUsed { .. }
            | Event::DieSet { .. }
            | Event::EndlessManaUsed { .. }
            | Event::RewardDieRolled { .. }
            | Event::SourceDieKept { .. }
            | Event::CrystalDeclined { .. }
            | Event::CrystalsReturned { .. }
            | Event::DieKept { .. }
            | Event::DieRerolled { .. } => Theme::Mana,
            Event::TurnEnded { .. }
            | Event::TurnStarted { .. }
            | Event::TacticChosen { .. }
            | Event::DieStolen { .. }
            | Event::LongNightUsed { .. }
            | Event::CardStored { .. }
            | Event::StoredCardsTaken { .. }
            | Event::CardsShuffledIntoDeck { .. }
            | Event::ManaSearchUsed { .. }
            | Event::DrawStopped { .. }
            | Event::RightMomentAnnounced { .. }
            | Event::StolenDieUsed { .. }
            | Event::StolenDieReturned { .. }
            | Event::CardSearched { .. }
            | Event::TacticsRemoved { .. }
            | Event::RoundEnded { .. }
            | Event::RoundStarted { .. }
            | Event::ExtraTurnStarted { .. } => Theme::Turn,
            Event::DummySetUp { .. }
            | Event::DummyTacticDrawn { .. }
            | Event::DummyTurnPlayed { .. }
            | Event::DummyAnnouncedEndOfRound
            | Event::DummyCardAdded { .. }
            | Event::DummyCrystalGained { .. }
            | Event::DummyDeckShuffled { .. }
            | Event::DummySkillRevealed { .. } => Theme::Dummy,
            Event::AssaultStarted { .. }
            | Event::EnemiesProvoked { .. }
            | Event::EnemyChallenged { .. }
            | Event::CombatStarted { .. }
            | Event::CombatPhaseStarted { .. }
            | Event::CombatEnded { .. }
            | Event::AttackDeclared { .. }
            | Event::EnemiesDefeated { .. }
            | Event::AttackFailed { .. }
            | Event::BlockDeclared { .. }
            | Event::EnemyBlocked { .. }
            | Event::BlockFailed { .. }
            | Event::WoundsTaken { .. }
            | Event::WoundFromCard { .. }
            | Event::PoisonWoundsDiscarded { .. }
            | Event::HandParalyzed { .. }
            | Event::EnemySummoned { .. }
            | Event::SummonedEnemyDiscarded { .. }
            | Event::SummonsHeld { .. }
            | Event::KnockedOut { .. }
            | Event::MoveSpentAsAttack { .. }
            | Event::InfluenceSpentAsBlock { .. }
            | Event::ResistanceBroken { .. }
            | Event::EnemyDestroyed { .. }
            | Event::DeferredAttackUsed { .. } => Theme::Combat,
            Event::UnitRecruited { .. }
            | Event::BannerUsed { .. }
            | Event::BannersReclaimed { .. }
            | Event::BannerAssigned { .. }
            | Event::BannerLost { .. }
            | Event::UnitDisbanded { .. }
            | Event::UnitActivated { .. }
            | Event::UnitReadied { .. }
            | Event::UnitWounded { .. }
            | Event::UnitsReadied { .. }
            | Event::DamageAssignedToUnit { .. }
            | Event::UnitDestroyed { .. }
            | Event::UnitHealed { .. }
            | Event::UnitUnderBonds { .. }
            | Event::UnitRewardTaken { .. }
            | Event::UnitRewardDeclined { .. } => Theme::Units,
            Event::CommonDeckShuffled { .. }
            | Event::OfferCardDealt { .. }
            | Event::OfferCardReturned { .. }
            | Event::OfferCardGained { .. }
            | Event::InteractionStarted { .. }
            | Event::ArtifactDieRolled { .. }
            | Event::HealingBought { .. }
            | Event::VillagePlundered { .. }
            | Event::ArtifactsDrawn { .. }
            | Event::ArtifactTaken { .. }
            | Event::ArtifactReturned { .. }
            | Event::MonasteryBurnStarted { .. }
            | Event::MonasteryBurned { .. }
            | Event::SiteConquered { .. }
            | Event::CityShieldPlaced { .. }
            | Event::CityConquered { .. }
            | Event::AdvancedActionDrawnBlind { .. }
            | Event::ArtifactBought { .. }
            | Event::EliteUnitAdded { .. }
            | Event::AdventureSiteEntered { .. }
            | Event::SpellBought { .. }
            | Event::AdvancedActionBought { .. }
            | Event::AltarTributePaid { .. }
            | Event::AdvancedActionLearned { .. } => Theme::Sites,
            Event::FameGained { .. }
            | Event::LevelGained { .. }
            | Event::ReputationChanged { .. }
            | Event::SkillsRevealed { .. }
            | Event::SkillGained { .. }
            | Event::SkillTakenFromCommonOffer { .. }
            | Event::SkillUsed { .. }
            | Event::SkillPlacedInCenter { .. }
            | Event::CenterAdvantageUsed { .. }
            | Event::CenterSkillReturned { .. }
            | Event::SkillToCommonOffer { .. }
            | Event::SkillPileShuffled { .. } => Theme::Progression,
        }
    }

    /// Ferme le tour (ou ouvre la partie) : la raison du refus est « rien depuis le début du tour ».
    pub(crate) fn closes_turn(&self) -> bool {
        matches!(
            self,
            Event::GameStarted { .. }
                | Event::TurnEnded { .. }
                | Event::TurnStarted { .. }
                | Event::RoundEnded { .. }
                | Event::RoundStarted { .. }
                | Event::GameEnded { .. }
        )
    }
}

/// Un événement tel qu'un joueur a le droit de le voir.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlayerEvent {
    GameStarted {
        config: GameConfig,
    },
    TilePlaced {
        tile: TileId,
        position: TilePosition,
    },
    /// Seul le dos des tuiles se voit.
    TileDeckFormed {
        counts: TileDeckCounts,
    },
    TileRevealed {
        tile: TileId,
        position: TilePosition,
    },
    HeroFigurePlaced {
        player: PlayerId,
        hex: Hex,
    },
    DeckShuffled {
        player: PlayerId,
        count: usize,
    },
    /// `card` n'est connue que du joueur qui pioche.
    CardDrawn {
        player: PlayerId,
        card: Option<CardId>,
    },
    CardPlayed {
        player: PlayerId,
        card: CardId,
        play: CardPlay,
        effect: Effect,
    },
    CardDiscarded {
        player: PlayerId,
        card: CardId,
    },
    HeroMoved {
        player: PlayerId,
        from: Hex,
        to: Hex,
        cost: u8,
    },
    Explored {
        player: PlayerId,
        position: TilePosition,
        cost: u8,
    },
    TurnEnded {
        player: PlayerId,
    },
    TurnStarted {
        player: PlayerId,
        turn: u32,
    },
    /// Publique : la Source se voit sur la table.
    SourceRolled {
        dice: Vec<ManaColor>,
    },
    /// Cristal gagné dans l'inventaire : à la fin du tour sur une mine (spec 021).
    CrystalGained {
        player: PlayerId,
        color: BasicColor,
    },
    /// Dé de la Source pris pour payer un effet ; il reste à sa place jusqu'à la fin du tour
    /// (spec 022).
    DieTaken {
        player: PlayerId,
        die: u8,
        color: ManaColor,
    },
    /// Cristal de l'inventaire dépensé pour payer un effet (spec 022).
    CrystalUsed {
        player: PlayerId,
        color: BasicColor,
    },
    /// Dé pris relancé et rendu à la Source à la fin du tour ; `color` est le résultat
    /// (spec 022).
    DieReturned {
        player: PlayerId,
        die: u8,
        color: ManaColor,
    },
    /// Spec 025 : jetons et reprise, publics.
    ManaTokenGained {
        player: PlayerId,
        color: ManaColor,
    },
    ManaTokenUsed {
        player: PlayerId,
        color: ManaColor,
    },
    CardReturnedToHand {
        player: PlayerId,
        card: CardId,
    },
    DieSet {
        player: PlayerId,
        die: u8,
        color: BasicColor,
    },
    /// Fin de la manche `round`, annoncée deck vide (spec 023) : plus de décision jusqu'à la
    /// manche suivante, ni après la dernière.
    RoundEnded {
        round: u32,
    },
    /// Début de la manche `round` : le plateau Jour/Nuit a été retourné, `time_of_day` est
    /// déjà calculé (spec 023).
    RoundStarted {
        round: u32,
        time_of_day: TimeOfDay,
    },
    /// Seul le nombre de jetons se voit (spec 030).
    EnemyPileShuffled {
        color: EnemyColor,
        count: usize,
    },
    /// Seul le nombre de jetons se voit (spec 031).
    EnemyPileReshuffled {
        color: EnemyColor,
        count: usize,
    },
    EnemyTokenPlaced {
        token: EnemyToken,
        hex: Hex,
    },
    /// Spec 043 : un défenseur face cachée ne montre que sa couleur.
    DefenderPlaced {
        color: EnemyColor,
        hex: Hex,
    },
    CityPlaced {
        hex: Hex,
        color: crate::BasicColor,
        level: u8,
    },
    EnemyTokenRevealed {
        hex: Hex,
        token: EnemyToken,
    },
    AssaultStarted {
        player: PlayerId,
        site: Hex,
        from: Hex,
    },
    EnemiesProvoked {
        player: PlayerId,
        enemies: Vec<Hex>,
    },
    /// Spec 031 : le combat est public.
    EnemyChallenged {
        player: PlayerId,
        enemy: Hex,
    },
    CombatStarted {
        player: PlayerId,
        enemies: Vec<(Hex, EnemyToken)>,
    },
    CombatPhaseStarted {
        player: PlayerId,
        phase: CombatPhase,
    },
    CombatEnded {
        player: PlayerId,
    },
    AttackDeclared {
        player: PlayerId,
        targets: Vec<EnemyToken>,
    },
    EnemiesDefeated {
        player: PlayerId,
        enemies: Vec<EnemyToken>,
        fame: u32,
    },
    AttackFailed {
        player: PlayerId,
    },
    BlockDeclared {
        player: PlayerId,
        target: EnemyToken,
    },
    EnemyBlocked {
        player: PlayerId,
        enemy: EnemyToken,
    },
    BlockFailed {
        player: PlayerId,
    },
    /// Les Blessures n'ont rien de caché : copiées telles quelles.
    WoundsTaken {
        player: PlayerId,
        enemy: EnemyToken,
        wounds: Vec<CardId>,
    },
    /// Spec 053 : public.
    WoundFromCard {
        player: PlayerId,
        card: CardId,
        wound: CardId,
    },
    /// Spec 033 : le combat est public.
    PoisonWoundsDiscarded {
        player: PlayerId,
        enemy: EnemyToken,
        wounds: Vec<CardId>,
    },
    HandParalyzed {
        player: PlayerId,
        cards: Vec<CardId>,
    },
    EnemySummoned {
        player: PlayerId,
        summoner: EnemyToken,
        token: EnemyToken,
    },
    SummonedEnemyDiscarded {
        player: PlayerId,
        summoner: EnemyToken,
    },
    SummonsHeld {
        player: PlayerId,
        held: bool,
    },
    WoundsHealed {
        player: PlayerId,
        wounds: Vec<CardId>,
    },
    KnockedOut {
        player: PlayerId,
        cards: Vec<CardId>,
    },
    /// Spec 035 : public.
    LevelGained {
        player: PlayerId,
        level: u8,
    },
    /// Gloire gagnée hors combat : une tuile révélée dans « Première reconnaissance » (spec 063).
    FameGained {
        player: PlayerId,
        fame: u32,
    },
    /// Une cité a été révélée : le tour qui suit celui-ci est le dernier (MKUE p. 18 ; spec 063).
    LastTurnAnnounced {
        player: PlayerId,
    },
    /// La partie est terminée : plus aucune décision (spec 063).
    /// Seul le nombre de cartes se voit (spec 050).
    CommonDeckShuffled {
        deck: CommonDeck,
        count: usize,
    },
    /// La carte du dessus de sa pioche entre dans l'offre : en première place pour les actions
    /// avancées et les sorts, à la suite pour les unités (spec 050).
    OfferCardDealt {
        offer: Offer,
        card: CommonCard,
    },
    /// La carte quitte l'offre et va sous sa pioche (MKUE p. 4, 2.c à 2.e ; spec 050).
    OfferCardReturned {
        offer: Offer,
        card: CommonCard,
    },
    /// Les deux compétences du dessus de la pile révélées à un niveau pair, en attente du choix
    /// (MKUE p. 11, 6.b ; spec 051).
    SkillsRevealed {
        player: PlayerId,
        skills: Vec<Skill>,
    },
    /// La compétence gagnée, face visible devant le joueur (MKUE p. 5, *Using Skills* 2.a).
    SkillGained {
        player: PlayerId,
        skill: Skill,
    },
    /// La compétence prise dans l'offre commune : l'action avancée sera la dernière (spec 051).
    SkillTakenFromCommonOffer {
        player: PlayerId,
        skill: Skill,
    },
    /// La compétence révélée non gardée, posée dans l'offre commune (spec 051).
    SkillToCommonOffer {
        skill: Skill,
    },
    /// La compétence utilisée, avec l'effet de l'option au moment de la journée (spec 051) :
    /// utilisée ce tour, ou retournée si elle sert une fois par manche.
    SkillUsed {
        player: PlayerId,
        skill: Skill,
        option: u8,
        effect: Effect,
    },
    /// Une compétence coopérative posée au centre (MKUE p. 5, *Using Skills* 4.b ; spec 058) : son
    /// avantage au tour suivant du joueur, avec la couleur de Surcharge de mana.
    SkillPlacedInCenter {
        player: PlayerId,
        skill: Skill,
        color: Option<ManaColor>,
    },
    /// L'avantage du centre de cette compétence, consommé (spec 058).
    CenterAdvantageUsed {
        player: PlayerId,
        skill: Skill,
    },
    /// À la fin du tour suivant, la compétence posée au centre revient retournée, son avantage
    /// perdu (MKUE p. 4, 2.g ; Compendium, *Skill Tokens* S4, S5 ; spec 058).
    CenterSkillReturned {
        player: PlayerId,
        skill: Skill,
    },
    /// L'unité recrutée va sous le jeton de Loyauté sans faille (spec 058).
    UnitUnderBonds {
        player: PlayerId,
        unit: UnitCard,
    },
    /// Le dé pris avec l'avantage d'Embrasure de la Source sera rendu tel quel à la fin du tour
    /// (spec 058).
    SourceDieKept {
        player: PlayerId,
        die: u8,
    },
    /// Botte secrète sur cet ennemi : Armure −`by` (ses résistances au moment de l'utiliser),
    /// jamais sous 1, pour le combat (spec 058).
    ResistanceBroken {
        player: PlayerId,
        enemy: EnemyToken,
        by: u8,
    },
    /// Le joueur gagne la carte de l'offre : sur le dessus de son deck (MKUE p. 5, *"Gain"
    /// Effects* 3 ; spec 050).
    OfferCardGained {
        player: PlayerId,
        offer: Offer,
        card: CommonCard,
        /// Où va la carte : le dessus du deck, sauf quand la carte qui la fait gagner dit autre
        /// chose (spec 056).
        to: CardDestination,
    },
    /// La réputation change de `change`, bornée à la piste, aussitôt (spec 040, Q3).
    ReputationChanged {
        player: PlayerId,
        change: i8,
    },
    /// Le héros interagit avec les habitants de ce site : mouvement non dépensé perdu, puis le
    /// bonus de réputation s'ajoute à l'influence déjà jouée, gardée par variante maison
    /// (MKUE p. 6, 6.e ; p. 8, 2.a ; spec 040 ; issue #95).
    InteractionStarted {
        player: PlayerId,
        site: Site,
        bonus: i8,
        city_shields: u8,
    },
    /// Unité recrutée : elle quitte l'offre (sans la compléter) et entre dans la zone des unités,
    /// disponible et non blessée ; l'influence baisse de `cost` (MKUE p. 8, 4.a ; spec 042).
    UnitRecruited {
        player: PlayerId,
        unit: UnitCard,
        cost: u8,
    },
    /// Spec 055 : une bannière de la main assignée à une unité (MKUE p. 5, 8).
    BannerAssigned {
        player: PlayerId,
        card: CardId,
        unit: UnitCard,
    },
    /// Spec 055 : la bannière d'une unité va dans la défausse : l'unité détruite ou renvoyée, ou une
    /// autre bannière assignée (MKUE p. 5, 8.a).
    BannerLost {
        player: PlayerId,
        card: CardId,
        unit: UnitCard,
    },
    /// Spec 055 (MKUE p. 5, 8.b ; Q2) : à la fin de la manche, ces bannières quittent leurs unités
    /// pour le deck, avant son mélange ; les bannières retournées se remettent face visible.
    BannersReclaimed {
        player: PlayerId,
        banners: Vec<CardId>,
    },
    /// Spec 055 : un mana payé par la face renforcée d'un anneau (mana illimité de sa couleur ou
    /// noir) ; rien n'est retiré.
    EndlessManaUsed {
        player: PlayerId,
        color: ManaColor,
    },
    /// Spec 055 : la bannière de l'unité a servi. Terreur : l'unité épuisée, `enemy` n'attaque pas
    /// ce combat ; Courage : la bannière retournée.
    BannerUsed {
        player: PlayerId,
        unit: UnitCard,
        enemy: Option<EnemyToken>,
    },
    /// Spec 055 : un dé de mana lancé par l'artefact `card` (Corne de la colère, Poche de mana) ;
    /// ce qu'il donne suit dans ses événements.
    ArtifactDieRolled {
        player: PlayerId,
        card: CardId,
        color: ManaColor,
    },
    /// Unité renvoyée pour en recruter une autre : elle quitte la partie (MKUE p. 5, 4.a).
    UnitDisbanded {
        player: PlayerId,
        unit: UnitCard,
    },
    /// Unité activée pour l'option `option` de sa capacité `ability`, d'effet `effect` : elle est
    /// épuisée, l'effet compte comme celui d'une carte (MKUE p. 5, 5 ; spec 042).
    UnitActivated {
        player: PlayerId,
        unit: UnitCard,
        ability: u8,
        option: u8,
        effect: Effect,
        /// L'ennemi du combat visé par la capacité (spec 053).
        enemy: Option<EnemyToken>,
    },
    /// L'unité redevient disponible (MKUE p. 5, 6 ; spec 042).
    UnitReadied {
        player: PlayerId,
        unit: UnitCard,
    },
    /// L'unité reçoit des Blessures hors dégâts (Épéistes d'Utem ; spec 042).
    UnitWounded {
        player: PlayerId,
        unit: UnitCard,
        wounds: u8,
    },
    /// Les dégâts de l'ennemi de `enemy` assignés à cette unité : `wounds` Blessures (0 à 2) ;
    /// les dégâts restants baissent de son armure (MKUE p. 9, *Assign Damage Phase* 4 ; spec 042).
    DamageAssignedToUnit {
        player: PlayerId,
        enemy: EnemyToken,
        unit: UnitCard,
        wounds: u8,
    },
    /// L'unité est détruite (paralysant, MKUE p. 10, 6.b) : elle quitte la partie.
    UnitDestroyed {
        player: PlayerId,
        unit: UnitCard,
    },
    /// Une Blessure de l'unité soignée pour `cost` points de soin (MKUE p. 11, 3.c ; spec 042).
    UnitHealed {
        player: PlayerId,
        unit: UnitCard,
        cost: u8,
    },
    /// Nouvelle manche : toutes les unités redeviennent disponibles (MKUE p. 4, 2.g ; spec 042).
    UnitsReadied {
        player: PlayerId,
    },
    /// Un point de soin acheté pour `cost` influence pendant une interaction (MKUE p. 8, 4.b ;
    /// spec 041).
    HealingBought {
        player: PlayerId,
        cost: u8,
    },
    /// Action avancée achetée au monastère : elle quitte l'offre d'unités, sans la compléter, pour
    /// le dessus du deck ; l'influence baisse de `cost` (MKUE p. 8, 4.c ; spec 041).
    AdvancedActionBought {
        player: PlayerId,
        card: CardId,
        cost: u8,
    },
    AdvancedActionDrawnBlind {
        player: PlayerId,
        card: CardId,
        cost: u8,
    },
    ArtifactBought {
        player: PlayerId,
        cost: u8,
    },
    EliteUnitAdded {
        player: PlayerId,
        card: CommonCard,
        cost: u8,
    },
    /// Sort acheté à une tour de mage conquise : il quitte l'offre de sorts pour le dessus du
    /// deck ; l'influence baisse de `cost` (MKUE p. 8, 4.d ; spec 043).
    SpellBought {
        player: PlayerId,
        card: CardId,
        cost: u8,
    },
    /// Incendie du monastère de `hex` commencé : points non dépensés perdus, plus de déplacement
    /// ni de repos, le combat qui suit exclut les unités (carte du monastère ; spec 041).
    MonasteryBurnStarted {
        player: PlayerId,
        hex: Hex,
    },
    /// Ennemi du monastère vaincu : un jeton Bouclier du joueur sur `hex`, le monastère est
    /// incendié, une récompense artefact gagnée (carte du monastère ; spec 041).
    MonasteryBurned {
        player: PlayerId,
        hex: Hex,
    },
    /// Le jeton `token` de `hex` quitte le plateau de jeu pour la défausse de sa couleur, sans être vaincu
    /// (ennemi d'un incendie raté ; FAQ p3 ; spec 041).
    EnemyTokenDiscarded {
        hex: Hex,
        token: EnemyToken,
    },
    SiteConquered {
        player: PlayerId,
        hex: Hex,
    },
    CityShieldPlaced {
        player: PlayerId,
        hex: Hex,
        enemy: EnemyToken,
    },
    CityConquered {
        hex: Hex,
        leader: PlayerId,
    },
    HeroWithdrew {
        player: PlayerId,
        from: Hex,
        to: Hex,
    },
    /// Artefacts tirés du dessus de la pioche pour une récompense : ils attendent le choix ; une
    /// récompense de moins (MKUE p. 11, *End of the Turn* 5.b ; spec 041).
    ArtifactsDrawn {
        player: PlayerId,
        cards: Vec<CardId>,
    },
    /// L'artefact pris va sur le dessus du deck (spec 041).
    ArtifactTaken {
        player: PlayerId,
        card: CardId,
    },
    /// L'artefact tiré non pris va sous la pioche d'artefacts (spec 041).
    ArtifactReturned {
        card: CardId,
    },
    /// Village pillé au début du tour : plus de pillage ce tour (carte du village ; spec 041).
    VillagePlundered {
        player: PlayerId,
    },
    /// Mouvement dépensé dans l'attaque en cours (Agilité, spec 052) : 1 point pour Attaque 1
    /// physique, ou 2 pour Attaque à distance 1 (`ranged`, phase d'attaque à distance).
    MoveSpentAsAttack {
        player: PlayerId,
        ranged: bool,
    },
    /// Influence dépensée dans la parade en cours (Diplomatie, spec 052) : 1 point pour Parade 1
    /// de cet élément.
    InfluenceSpentAsBlock {
        player: PlayerId,
        element: Element,
    },
    /// Retraite forcée à la fin du tour (MKUE p. 11, *End of the Turn* 2 ; spec 052) : la figurine
    /// revient sur ses pas par les cases de `path`, jusqu'à la dernière, et prend ces Blessures en
    /// main, une par case.
    ForcedWithdrawal {
        player: PlayerId,
        path: Vec<Hex>,
        wounds: Vec<CardId>,
    },
    /// Temporisation gardée à la fin du tour (spec 052) : la carte quitte la zone de jeu pour le
    /// dessus (`top`) ou le dessous du deck.
    SteadyTempoKept {
        player: PlayerId,
        card: CardId,
        top: bool,
    },
    AdventureSiteEntered {
        player: PlayerId,
        hex: Hex,
    },
    /// Un dé de récompense lancé de la réserve, pas de la Source (Compendium, *Mana* S14 ;
    /// spec 044) : au donjon conquis, or ou noir donnent une récompense sort, sinon artefact ; en
    /// fin de tour, un cristal aléatoire.
    RewardDieRolled {
        player: PlayerId,
        color: ManaColor,
    },
    /// Une Blessure de `from` jetée sur une clairière magique, rendue à la pile de Blessures : pas un
    /// soin (carte de la clairière ; spec 044).
    WoundThrownAway {
        player: PlayerId,
        from: WoundPile,
    },
    /// Seul le nombre de jetons se voit (spec 045).
    RuinPileShuffled {
        count: usize,
    },
    /// La pile est face cachée : seul le nombre de compétences se voit (spec 051).
    SkillPileShuffled {
        player: PlayerId,
        count: usize,
    },
    /// Spec 045 : un jeton posé face cachée ne se voit pas.
    RuinTokenPlaced {
        token: Option<RuinToken>,
        hex: Hex,
        face_up: bool,
    },
    RuinTokenRevealed {
        hex: Hex,
        token: RuinToken,
    },
    AltarTributePaid {
        player: PlayerId,
        hex: Hex,
    },
    UnitRewardTaken {
        player: PlayerId,
        unit: UnitCard,
    },
    UnitRewardDeclined {
        player: PlayerId,
    },
    /// Une carte action quitte le jeu (MKUE p. 5, *Discarding and Throwing Away* 2.b ; spec 056) :
    /// de la main (Décomposition, Entraînement) ou de la zone de jeu (cible d'Effet maximum).
    CardDestroyed {
        player: PlayerId,
        card: CardId,
        from: DestroyedFrom,
    },
    /// Le dé de la Source à cette position est relancé et y reste (Tempête de mana, spec 056).
    DieRerolled {
        die: u8,
        color: ManaColor,
    },
    /// Le joueur refuse le cristal de Rituel de sang renforcé (spec 056).
    CrystalDeclined {
        player: PlayerId,
    },
    /// Influence dépensée pour une action avancée par Apprentissage ; le modificateur est retiré
    /// (spec 056). La carte gagnée suit (`OfferCardGained`).
    AdvancedActionLearned {
        player: PlayerId,
        card: CardId,
        cost: u8,
    },
    /// Les cristaux dépensés ce tour reviennent dans l'inventaire (Maîtrise du cristal, spec 056).
    CrystalsReturned {
        player: PlayerId,
        colors: Vec<BasicColor>,
    },
    /// Distorsion temporelle : les cartes de la zone de jeu reviennent en main au lieu d'aller dans la défausse (spec 057).
    CardsReturnedToHand {
        player: PlayerId,
        cards: Vec<CardId>,
    },
    /// Cette carte de la zone de jeu est écartée jusqu'à la fin de la manche (Distorsion temporelle, spec 057).
    CardSetAside {
        player: PlayerId,
        card: CardId,
    },
    /// Le même joueur joue aussitôt un tour supplémentaire, sans pioche (Distorsion temporelle, spec 057).
    ExtraTurnStarted {
        player: PlayerId,
    },
    /// Les cartes tirées au hasard dans la défausse par Méditation, qui attendent leur place (spec 057).
    DiscardPicked {
        player: PlayerId,
        cards: Vec<CardId>,
    },
    /// Les cartes de Méditation remises de la défausse sur le deck (`top`) ou dessous, la première la plus haute (spec 057).
    CardsPlacedOnDeck {
        player: PlayerId,
        top: bool,
        order: Vec<CardId>,
    },
    /// La pioche a profité des cartes en plus de Méditation (spec 057).
    DrawBonusUsed {
        player: PlayerId,
    },
    /// Le dé de la Source gardé par Puits de mana jusqu'à la fin de la manche, par sa position (spec 057).
    DieKept {
        player: PlayerId,
        die: u8,
    },
    /// Cet ennemi est détruit par un sort : vaincu comme par une attaque (gloire), une créature
    /// invoquée sans gloire (spec 054, Q2).
    EnemyDestroyed {
        player: PlayerId,
        enemy: EnemyToken,
    },
    /// L'attaque différée du Bouclier ardent ajoutée à l'attaque en cours (spec 054).
    DeferredAttackUsed {
        player: PlayerId,
        amount: u8,
        element: Element,
    },
    TacticChosen {
        player: PlayerId,
        tactic: crate::Tactic,
    },
    TacticsRemoved {
        tactics: Vec<crate::Tactic>,
    },
    DeckReshuffled {
        player: PlayerId,
        count: usize,
    },
    DieStolen {
        player: PlayerId,
        die: u8,
    },
    StolenDieUsed {
        player: PlayerId,
    },
    RightMomentAnnounced {
        player: PlayerId,
    },
    /// Les cartes remises, triées : leur ordre dans le deck reste caché.
    LongNightUsed {
        player: PlayerId,
        cards: Vec<CardId>,
    },
    DrawStopped {
        player: PlayerId,
        missing: usize,
    },
    ManaSearchUsed {
        player: PlayerId,
    },
    CardsShuffledIntoDeck {
        player: PlayerId,
        cards: Vec<CardId>,
    },
    /// La carte posée reste cachée, au joueur compris (Q4).
    CardStored {
        player: PlayerId,
    },
    StoredCardsTaken {
        player: PlayerId,
    },
    StolenDieReturned {
        player: PlayerId,
        color: ManaColor,
    },
    CardSearched {
        player: PlayerId,
        card: Option<CardId>,
    },
    /// Le deck et la pile triés : leur ordre reste caché (spec 061, C32).
    DummySetUp {
        hero: crate::Hero,
        crystals: Vec<BasicColor>,
        skills: Vec<Skill>,
        deck: Vec<CardId>,
    },
    DummyTacticDrawn {
        tactic: crate::Tactic,
    },
    DummyTurnPlayed {
        cards: Vec<CardId>,
    },
    DummyAnnouncedEndOfRound,
    DummyCardAdded {
        card: CardId,
    },
    DummyCrystalGained {
        color: BasicColor,
    },
    /// Seul le nombre de cartes se voit.
    DummyDeckShuffled {
        count: usize,
    },
    DummySkillRevealed {
        skill: Skill,
    },
    GameEnded {
        reason: GameEndReason,
    },
}

/// L'événement tel que `viewer` a le droit de le voir, filtré dans le fichier de son thème.
pub fn filter_event(event: &Event, viewer: PlayerId) -> PlayerEvent {
    match event.theme() {
        Theme::Map => map::filter(event, viewer),
        Theme::Cards => cards::filter(event, viewer),
        Theme::Mana => mana::filter(event, viewer),
        Theme::Turn => turn::filter(event, viewer),
        Theme::Dummy => dummy::filter(event, viewer),
        Theme::Combat => combat::filter(event, viewer),
        Theme::Units => units::filter(event, viewer),
        Theme::Sites => sites::filter(event, viewer),
        Theme::Progression => progression::filter(event, viewer),
    }
}

/// Une copie triée : le contenu se voit, pas l'ordre (spec 061).
fn sorted<T: Ord + Clone>(items: &[T]) -> Vec<T> {
    let mut items = items.to_vec();
    items.sort();
    items
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BasicAction, DeedCard};

    fn card(copy: u16) -> CardId {
        CardId::new(DeedCard::Basic(BasicAction::March), copy)
    }

    // Spec 000, « Information cachée » : la carte n'est visible que du joueur qui pioche.
    #[test]
    fn drawn_card_is_hidden_from_other_players() {
        let event = Event::CardDrawn {
            player: PlayerId(1),
            card: card(1),
        };
        assert_eq!(
            filter_event(&event, PlayerId::SOLO),
            PlayerEvent::CardDrawn {
                player: PlayerId(1),
                card: None
            }
        );
    }

    // Spec 000, « Actions et événements » : l'ordre du mélange n'est jamais révélé.
    #[test]
    fn shuffle_shows_only_card_count() {
        let event = Event::DeckShuffled {
            player: PlayerId::SOLO,
            order: vec![card(2), card(1)],
        };
        assert_eq!(
            filter_event(&event, PlayerId::SOLO),
            PlayerEvent::DeckShuffled {
                player: PlayerId::SOLO,
                count: 2
            }
        );
    }
}
