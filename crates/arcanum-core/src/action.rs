use thiserror::Error;

use crate::{
    BasicColor, CardId, CardPlay, EnemyToken, Hex, ManaColor, ManaPayment, PlayerId, Skill,
    TilePosition, UnitCard,
};

/// D'où vient la Blessure jetée sur une clairière magique (spec 044) : la main (hors défausse de
/// fin de tour) ou la défausse.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum WoundPile {
    Hand,
    DiscardPile,
}

/// Intention d'un joueur, à valider par le moteur.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    /// Jouer une carte de sa main (spec 013).
    PlayCard { card: CardId, play: CardPlay },
    /// Déplacer sa figurine dans une case adjacente (spec 014).
    MoveHero { to: Hex },
    /// Aller sur une case atteignable lointaine par son trajet retenu, un pas par case (spec 036).
    FollowRoute { to: Hex },
    /// Révéler une tuile à cet emplacement, voisin de la figurine (spec 015).
    Explore { position: TilePosition },
    /// Finir son tour, en défaussant ces cartes de la main avant de repiocher (spec 016), en
    /// défaussant `reclaim` pour reprendre Euphorie cristalline en main (spec 025), et en jetant
    /// une Blessure de `glade` sur une clairière magique (spec 044).
    EndTurn {
        discard: Vec<CardId>,
        reclaim: Option<CardId>,
        glade: Option<WoundPile>,
        /// Garder Temporisation : sous le deck (sur le deck, renforcée) au lieu de la défausse
        /// (spec 052).
        steady_tempo: bool,
    },
    /// Passer son tour pour annoncer la fin de la manche, deck vide (spec 023).
    AnnounceEndOfRound,
    /// Prendre cette tactique au début de la manche (spec 060).
    ChooseTactic { tactic: crate::Tactic },
    /// Faux départ : défausser ces cartes de la main (0 à 3), piocher autant, remélanger (spec 060).
    RethinkDiscard { cards: Vec<CardId> },
    /// Préparation : prendre cette carte du deck en main, puis mélanger le deck (spec 060).
    PrepareCard { card: CardId },
    /// Vol de mana : poser ce dé de couleur de base de la Source sur la tactique (spec 060).
    StealDie { die: u8 },
    /// Au bon moment : annoncer un tour de plus après celui-ci (spec 060).
    AnnounceRightMoment,
    /// Longue Nuit : 3 cartes de la défausse, au hasard, retournent dans le deck vide (spec 060).
    UseLongNight,
    /// Longue Nuit refusée quand la pioche de fin de tour s'est arrêtée (spec 060, Q5).
    DeclineLongNight,
    /// Appel du mana : relancer ces dés de la Source (1 ou 2, les dés or d'abord ; spec 060).
    ManaSearch { dice: Vec<u8> },
    /// Méditation nocturne : mélanger ces cartes de la main (1 à 5) dans le deck, puis piocher
    /// autant (spec 060).
    MidnightMeditation { cards: Vec<CardId> },
    /// Provisions, avant le tour : poser la carte du dessus du deck sous la tactique, ou (`take`)
    /// retourner la tactique et prendre en main les cartes posées (spec 060).
    SparingPower { take: bool },
    /// Défier l'ennemi errant de cette case, voisine de la figurine (spec 031).
    Challenge { enemy: Hex },
    /// Engager le combat contre les ennemis à combattre (spec 031).
    StartCombat,
    /// Passer à la phase suivante du combat (spec 031).
    NextCombatPhase,
    /// Laisser les Invocateurs tirer leur créature, au début de la phase de parade, au lieu d'annuler
    /// leur attaque par la Bannière de terreur (issue #8).
    CallSummons,
    /// Terminer le combat depuis la phase d'attaque (spec 031).
    EndCombat,
    /// Choisir les cibles d'une attaque, dans l'ordre du combat (spec 031), par leur jeton
    /// (spec 044, Q1).
    DeclareAttack { targets: Vec<EnemyToken> },
    /// Valider l'attaque en cours (spec 031).
    ResolveAttack,
    /// Choisir l'ennemi que vise une parade (spec 031), par son jeton (spec 044, Q1).
    DeclareBlock { target: EnemyToken },
    /// Valider la parade en cours (spec 031).
    ResolveBlock,
    /// Interagir avec les habitants du village ou du monastère de la figurine : l'action du tour
    /// (spec 040).
    Interact,
    /// Prendre cette action avancée de l'offre, à la fin d'un tour à niveau pair (spec 050).
    GainAdvancedAction { card: CardId },
    /// Garder cette compétence parmi les deux révélées, ou la prendre dans l'offre de compétences
    /// communes, à la fin d'un tour à niveau pair (MKUE p. 11, 6.b ; spec 051).
    ChooseSkill { skill: Skill },
    /// Utiliser l'option `option` de cette compétence gagnée, sur l'unité `unit` pour Inspiration
    /// (MKUE p. 5, *Using Skills* ; spec 051), sur la cible `target` (spec 058 : case du Vol,
    /// ennemi de Botte secrète, couleur de Surcharge de mana, dé d'Embrasure de la Source).
    UseSkill {
        skill: Skill,
        option: u8,
        unit: Option<UnitCard>,
        target: Option<SkillTarget>,
    },
    /// Recruter cette unité de l'offre pendant une interaction, en renvoyant `disband` quand tous
    /// les jetons de commandement sont pris (spec 042).
    RecruitUnit {
        unit: UnitCard,
        disband: Option<UnitCard>,
        /// Sous le jeton de Loyauté sans faille : 5 Influence de moins (spec 058).
        bonds: bool,
    },
    /// Activer une unité disponible pour l'option `option` de sa capacité `ability` (à partir de
    /// 0), en payant `mana` dans l'ordre des couleurs à payer (la capacité, puis le supplément) ;
    /// `target` : l'unité à rendre disponible (spec 042).
    ActivateUnit {
        unit: UnitCard,
        ability: u8,
        option: u8,
        mana: Vec<ManaPayment>,
        target: Option<UnitCard>,
        /// L'ennemi du combat visé par la capacité (Illusionnistes, Glaceurs d'Amotep ; spec 053).
        enemy: Option<EnemyToken>,
        /// Avec Charisme : le bonus ajouté à la capacité (spec 058).
        leadership: bool,
    },
    /// Jouer une carte dont l'effet rend disponible l'unité `target` (Rajeunissement, spec 042).
    PlayCardOnUnit {
        card: CardId,
        play: CardPlay,
        target: UnitCard,
    },
    /// Assigner les dégâts restants de cet ennemi à une unité ou au héros (spec 042).
    AssignDamage { enemy: EnemyToken, to: DamageTarget },
    /// Dépenser des points de soin hors combat (spec 042, Q1).
    SpendHealing { on: HealingTarget },
    /// Acheter un point de soin pendant une interaction : 3 influence au village, 2 au monastère
    /// (spec 041).
    BuyHealing,
    /// Acheter cette action avancée de l'offre d'unités pendant une interaction au monastère, pour
    /// 6 influence (spec 041).
    BuyAdvancedAction { card: CardId },
    /// Acheter ce sort de l'offre de sorts pendant une interaction à une tour de mage conquise,
    /// pour 7 influence et un mana de sa couleur payé par `mana` (spec 043).
    BuySpell { card: CardId, mana: ManaPayment },
    /// Dans la cité verte conquise, acheter la carte du dessus de la pioche d'actions avancées, sans
    /// la voir, pour 6 influence : sur le dessus du deck (spec 062).
    BuyAdvancedActionFromDeck,
    /// Dans la cité rouge conquise, acheter un artefact pour 12 influence : tiré à la fin du tour
    /// comme une récompense (spec 062).
    BuyArtifact,
    /// Dans la cité blanche conquise, pour 2 influence, la carte du dessus de la pioche d'unités
    /// d'élite rejoint l'offre d'unités (spec 062).
    AddEliteUnit,
    /// Incendier le monastère de la figurine : l'action du tour, un combat sans les unités contre
    /// un ennemi violet (spec 041).
    BurnMonastery,
    /// Prendre cet artefact parmi ceux tirés pour une récompense, à la fin du tour (spec 041).
    TakeArtifact { card: CardId },
    /// Prendre ce sort de l'offre de sorts pour la récompense d'une tour de mage, à la fin du tour
    /// (spec 043).
    TakeSpell { card: CardId },
    /// Choisir la couleur du cristal d'un dé de récompense or, à la fin du tour (spec 044).
    ChooseCrystalColor { color: BasicColor },
    /// Entrer dans le site d'aventure de la figurine : l'action du tour, un combat aussitôt
    /// (spec 044).
    EnterAdventureSite,
    /// Faire une offrande à l'autel de la ruine de la figurine : 3 mana de sa couleur payés par
    /// `mana`, 7 gloire ; l'action du tour (spec 045).
    PayAltarTribute { mana: Vec<ManaPayment> },
    /// Assigner une bannière de la main à une unité du joueur, à tout moment du tour (spec 055 ;
    /// MKUE p. 5, 8).
    AssignBanner { card: CardId, unit: UnitCard },
    /// À la fin de la manche, les bannières assignées reprises dans le deck ; les autres restent
    /// sur leur unité (spec 055, Q2 ; MKUE p. 5, 8.b).
    ReclaimBanners { banners: Vec<CardId> },
    /// Se servir de la bannière d'une unité (spec 055) : Bannière de peur, en phase de parade,
    /// l'unité épuisée, `enemy` n'attaque pas ; Bannière de courage, hors combat, la bannière
    /// retournée rend l'unité disponible.
    UseBanner {
        unit: UnitCard,
        enemy: Option<EnemyToken>,
    },
    /// Prendre cette unité de l'offre d'unités pour une récompense de ruines, gratuite, à la fin du
    /// tour ; `disband` : l'unité renvoyée quand les jetons de commandement sont tous pris
    /// (spec 045).
    TakeUnitReward {
        unit: UnitCard,
        disband: Option<UnitCard>,
    },
    /// Renoncer à la récompense unité, sans jeton de commandement libre (spec 045, Q4).
    DeclineUnitReward,
    /// Piller le village de la figurine au début du tour : piocher deux cartes, Réputation −1
    /// (spec 041, Q1).
    PlunderVillage,
    /// Dépenser 1 point de Mouvement (2 en phase d'attaque à distance) dans l'attaque en cours,
    /// avec Agilité (spec 052).
    SpendMoveAsAttack,
    /// Dépenser 1 point d'Influence dans la parade en cours, avec Diplomatie (spec 052).
    SpendInfluenceAsBlock,
    /// Choisir la couleur d'un jeton de Rituel de sang renforcé, or et noir compris (spec 056).
    ChooseTokenColor { color: ManaColor },
    /// Payer un mana de base par `mana` pour un cristal de cette couleur, après les jetons de
    /// Rituel de sang renforcé (spec 056).
    PayForCrystal {
        color: BasicColor,
        mana: ManaPayment,
    },
    /// Refuser le cristal de Rituel de sang renforcé (spec 056).
    DeclineCrystal,
    /// Remettre les cartes de Méditation en attente toutes sur le deck (`top`) ou toutes dessous,
    /// dans l'ordre `order`, la première étant la plus haute (spec 057).
    PlaceOnDeck { top: bool, order: Vec<CardId> },
    /// Dépenser l'Influence d'Apprentissage pour cette action avancée de l'offre (spec 056).
    LearnAdvancedAction { card: CardId },
    /// Garder tel quel, à la fin du tour, le dé pris en plus avec l'avantage d'Embrasure de la Source
    /// (spec 058).
    KeepSourceDie,
    /// Ajouter l'attaque différée du Bouclier ardent à l'attaque en cours de la phase d'attaque
    /// (spec 054).
    UseDeferredAttack,
}

/// La cible d'une compétence du second lot (spec 058).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum SkillTarget {
    /// La case d'arrivée du Vol.
    Space(Hex),
    /// L'ennemi du combat de Botte secrète.
    Enemy(EnemyToken),
    /// La couleur du jeton de Surcharge de mana.
    Color(ManaColor),
    /// Le dé de la Source relancé par Embrasure de la Source, par sa position.
    Die(u8),
}

/// Qui reçoit les dégâts d'un ennemi non paré (MKUE p. 9, *Assign Damage Phase* 3 à 5).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum DamageTarget {
    Unit(UnitCard),
    Hero,
}

/// Sur quoi dépenser des points de soin (MKUE p. 11, 3.c).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum HealingTarget {
    /// Jeter une Blessure de la main : 1 point.
    Wound,
    /// Retirer une Blessure de cette unité : son niveau en points.
    Unit(UnitCard),
}

#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum ActionError {
    #[error("action non légale dans l'état courant")]
    NotLegal,
    #[error("aucune décision n'attend le joueur {player:?}")]
    WrongPlayer { player: PlayerId },
}
