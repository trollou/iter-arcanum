//! Cartes d'action de base (spec 013). Leurs définitions sont dans `content/basic_actions.ron` (spec 029).

use serde::Deserialize;

use crate::{
    AdvancedAction, Artifact, BasicColor, CardId, EnemyToken, GameContent, Hex, ManaColor,
    ModifierKind, Spell, UnitCard,
};

/// Limite de main au niveau 1 : la mise en place pioche autant de cartes (MKUE p. 3).
pub const STARTING_HAND_LIMIT: usize = 5;

/// Les 16 cartes d'action de base communes et les 2 cartes propres à chaque héros du jeu de base.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, serde::Serialize,
)]
pub enum BasicAction {
    Rage,
    Determination,
    Swiftness,
    March,
    Stamina,
    Tranquility,
    Promise,
    Threaten,
    Crystallize,
    ManaDraw,
    Concentration,
    Improvisation,
    BattleVersatility,
    ManaPull,
    WillFocus,
    CrystalJoy,
    NobleManners,
    Rejuvenate,
    ColdToughness,
    Instinct,
}

/// Une carte (dos « Deed ») : les cartes d'action de base, les Blessures (spec 031), les actions
/// avancées, les sorts et les artefacts (spec 050).
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum DeedCard {
    Basic(BasicAction),
    /// Prise dans la pile de blessures, illimitée (MKUE p. 3) ; ne se joue d'aucune façon
    /// (MKUE p. 4, 3.d).
    Wound,
    Advanced(AdvancedAction),
    Spell(Spell),
    Artifact(Artifact),
}

/// Élément d'une attaque ou d'une parade (MKUE p. 9 ; spec 032).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Element {
    Physical,
    Fire,
    Ice,
    ColdFire,
}

impl Element {
    pub const ALL: [Element; 4] = [
        Element::Physical,
        Element::Fire,
        Element::Ice,
        Element::ColdFire,
    ];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum AttackKind {
    Melee,
    Ranged,
    /// Seule attaque permise contre un fortifié en phase d'attaque à distance (MKUE p. 9, 3.a).
    Siege,
}

/// Effet d'une option : des points (spec 013) ou du mana (spec 025).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Effect {
    Move(u8),
    Influence(u8),
    Attack {
        amount: u8,
        element: Element,
        kind: AttackKind,
    },
    Block {
        amount: u8,
        element: Element,
    },
    /// Gagner un jeton de mana de cette couleur (MKUE p. 5, « Gain » 1).
    GainToken(BasicColor),
    /// Gagner un cristal de cette couleur, ou un jeton si l'inventaire en a déjà trois
    /// (MKUE p. 5, « Gain » 2).
    GainCrystal(BasicColor),
    /// Payer un mana de cette couleur pour en gagner un cristal (Cristallisation, effet de base).
    PayForCrystal(BasicColor),
    /// Un dé de plus ce tour (Source de mana, MKUE p. 5, 3).
    ExtraDie,
    /// Un dé de plus ce tour ; s'il est noir, il vaut toute couleur de base, même de jour
    /// (Attraction de mana, spec 025, Q3).
    ExtraDieBlackAnyColor,
    /// Prendre `dice` dés, mettre chacun sur une couleur de base, gagner `tokens` jetons de cette
    /// couleur par dé ; ces dés ne sont pas relancés (Source de mana et Attraction de mana
    /// renforcées, spec 025).
    SetDice {
        dice: u8,
        tokens: u8,
    },
    /// Jouer une autre carte d'action avec celle-ci, son effet renforcé sans payer, avec ce bonus
    /// sur les points (Concentration et Pouvoir de concentration renforcés, MKUE p. 5,
    /// « Composite »).
    Boost(u8),
    /// Parade de glace de cette valeur, plus 1 par capacité ou couleur d'attaque imprimée sur le
    /// jeton de l'ennemi paré (Endurance givrée renforcée, spec 033) : le compte se fait dans la
    /// parade en cours.
    IceBlockPerAbility(u8),
    /// Influence, et la réputation change de `reputation`, aussitôt (Menace renforcée, spec 040).
    InfluenceReputation {
        amount: u8,
        reputation: i8,
    },
    /// Influence, et pendant une interaction, `fame` de gloire et `reputation` de réputation de
    /// plus (Nobles manières, spec 040 ; FAQ p1).
    InfluenceInInteraction {
        amount: u8,
        fame: u8,
        reputation: i8,
    },
    /// Soin X : jeter jusqu'à X Blessures de la main, aussitôt (spec 034, Q2).
    Heal(u8),
    /// Piocher ce nombre de cartes du dessus du deck, sans remélanger (spec 034).
    DrawCards(u8),
    /// Parade physique qui compte double contre une attaque Vif (Gardes d'Utem et Gardiens
    /// d'Altem, version *Lost Legion* ; spec 042).
    SwiftBlock(u8),
    /// Rendre disponible une unité épuisée de ce niveau au plus (MKUE p. 5, 6 ; spec 042).
    ReadyUnit(u8),
    /// Un jeton de mana et un cristal de cette couleur (Mages de feu et de glace, spec 042).
    TokenAndCrystal(BasicColor),
    /// Deux jetons de mana de ces couleurs, or et noir compris (Mages d'Altem, spec 042).
    GainTokens(ManaColor, ManaColor),
    /// Aucun point : l'option ne fait que poser ses modificateurs (Au cœur du combat, capacités
    /// d'unités qui visent un ennemi ; spec 053).
    Nothing,
    /// Attaque, et une Blessure de la pile en main (Coup de sang, spec 053 ; Tempête de feu et
    /// Blizzard, spec 054).
    AttackWithWound {
        amount: u8,
        element: Element,
        kind: AttackKind,
    },
    /// Attaque `amount`, plus `per` par jeton ennemi affronté, compté en jouant la carte (Déluge
    /// de flammes, spec 054, Q4).
    AttackPerEnemy {
        amount: u8,
        element: Element,
        per: u8,
    },
    /// Parade `amount`, plus `per` par jeton ennemi affronté (Déluge de flammes, spec 054).
    BlockPerEnemy {
        amount: u8,
        element: Element,
        per: u8,
    },
    /// Détruire l'ennemi visé (Tornade, Désintégration ; spec 054).
    DestroyEnemy,
    /// Défausser une carte de couleur : un sort de sa couleur de l'offre se lance ce tour (Don
    /// pour la magie, spec 054).
    CastFromOffer,
    /// Mouvement et Soin ; en combat, le Mouvement seul (Marche revigorante, spec 053).
    MoveAndHeal {
        movement: u8,
        heal: u8,
    },
    /// Soin, et rendre disponible une unité épuisée de ce niveau au plus (Régénération, spec 053).
    HealAndReady {
        heal: u8,
        level: u8,
    },
    /// Influence `base`, plus `per` par Blessure en main ou sur une unité, comptées en jouant la
    /// carte (Besoin d'aide, spec 053).
    InfluencePerWound {
        base: u8,
        per: u8,
    },
    /// Une Blessure en main, un cristal et un jeton de ces couleurs (Rituel de sang, version *Lost
    /// Legion* ; spec 056).
    WoundCrystalAndToken {
        crystal: BasicColor,
        token: ManaColor,
    },
    /// Une Blessure en main, puis ce nombre de jetons de couleurs choisies une à une, puis un
    /// cristal payé d'un mana de base (Rituel de sang renforcé, spec 056, choix en attente).
    WoundAndTokens(u8),
    /// Ce nombre de cristaux de la couleur de la carte action détruite (Décomposition, spec 056).
    CrystalsFromDestroyed(u8),
    /// Un cristal de chaque autre couleur de base que la carte action détruite (Décomposition
    /// renforcée, spec 056).
    CrystalsOtherColors,
    /// Un cristal de cette couleur, déjà possédée (Maîtrise du cristal, spec 056).
    GainOwnedCrystal(BasicColor),
    /// Un cristal de la couleur du dé choisi dans la Source, puis le dé relancé (Tempête de mana,
    /// spec 056).
    CrystalFromDie,
    /// Relancer la Source, trois dés de plus, noir et or de toute couleur de base (Tempête de mana
    /// renforcée, spec 056).
    RerollSource,
    /// Jouer une autre carte action avec celle-ci, son option répétée ce nombre de fois, puis la
    /// détruire (Effet maximum, spec 056).
    Repeat(u8),
    /// Un sort de cette couleur de l'offre de sorts, dans la défausse (Don pour la magie renforcé,
    /// spec 056).
    GainSpell(BasicColor),
    /// Détruire une carte action et prendre une action avancée de sa couleur dans l'offre, mise à
    /// cette destination (Entraînement, spec 056).
    TrainFromOffer(CardDestination),
    /// Une Blessure en main par cristal de cette couleur possédé, puis deux cristaux de cette
    /// couleur (Brûlure de mana, spec 057).
    WoundsPerCrystal(BasicColor),
    /// Ne vise que les autres joueurs : rien en solo, jamais proposé (Fuite de mana, spec 057, Q2).
    OthersOnly,
    /// Soin `heal`, `forest` si la case du héros est une forêt (Restauration, spec 057).
    HealByTerrain {
        heal: u8,
        forest: u8,
    },
    /// Le Soin de Restauration, et rendre disponibles des unités épuisées de `levels` niveaux en
    /// tout (`forest_levels` en forêt) (Renaissance, spec 057).
    HealAndReadyLevels {
        heal: u8,
        forest_heal: u8,
        levels: u8,
        forest_levels: u8,
    },
    /// Rendre disponible une unité épuisée de tout niveau, et la soigner d'une Blessure si `heal`
    /// (Flux et Vol d'énergie, spec 057).
    ReadyAnyUnit {
        heal: bool,
    },
    /// Se déplacer de 3 cases au plus vers une case sécurisée, sans marais ni lac, sans payer ni
    /// provoquer ; `assault` : vers un site fortifié non conquis, assaut (Voyage et Attaque
    /// souterrains, spec 057).
    UndergroundMove {
        assault: bool,
    },
    /// Dépenser de 1 à `max` points de Mouvement, une case par point, vers une case sécurisée,
    /// sans provoquer (Ailes du vent, spec 057).
    FlyMove {
        max: u8,
    },
    /// Les ennemis visés n'attaquent pas : le 2e coûte 1 point de Mouvement, le 3e 2… (Ailes de la
    /// nuit, version *Lost Legion* ; spec 057).
    DoesNotAttackPerMove,
    /// Deux cartes de la défausse, tirées au hasard ou choisies, remises toutes deux sur ou sous le
    /// deck dans l'ordre choisi (Méditation, Transe ; spec 057).
    DiscardToDeck {
        random: bool,
    },
    /// Garder un dé de couleur de base de la Source jusqu'à la fin de la manche : 3 jetons de sa
    /// couleur aussitôt, ou 1 au début de chaque tour suivant (`each_turn`) (Puits de mana, spec
    /// 057).
    ClaimDie {
        each_turn: bool,
    },
    /// Spec 055 : l'effet de base d'une bannière, qui va sous une unité ; seule `AssignBanner` le
    /// joue.
    BannerAssigned,
    /// Influence, et de la gloire aussitôt (Bourse sans fond, spec 055).
    InfluenceAndFame {
        amount: u8,
        fame: u8,
    },
    /// Ce nombre de jetons or ; de nuit, les règles de jour pour le tour (Amulette du soleil, spec
    /// 055).
    AmuletOfSun {
        tokens: u8,
    },
    /// Ce nombre de jetons de couleurs choisies ; de jour, le mana noir comme de nuit pour le tour
    /// (Amulette des ténèbres, spec 055).
    AmuletOfDarkness {
        tokens: u8,
    },
    /// Un jeton et un cristal de cette couleur, et Gloire +1 (anneaux, spec 055).
    TokenCrystalAndFame(BasicColor),
    /// Un cristal et un jeton, de couleurs qui peuvent différer (Magie noire, Extractions ;
    /// spec 051).
    CrystalAndToken {
        crystal: BasicColor,
        token: ManaColor,
    },
    /// Piocher ce nombre de cartes, sans remélanger, et un jeton (Motivation, spec 051).
    DrawAndToken {
        cards: u8,
        token: BasicColor,
    },
    /// Soigner une Blessure d'une unité, sans dépenser de Soin (Inspiration, spec 051).
    HealUnit,
    /// Influence d'interaction, 1 par couleur de cristal de l'inventaire (Fortune clinquante,
    /// spec 058) ; résolue en `InteractionInfluence` à l'utilisation.
    CrystalColorInfluence,
    /// Influence qui ne sert qu'aux achats d'interaction (spec 058, Q3).
    InteractionInfluence(u8),
    /// Mouvement 1 par unité disponible et non blessée, `max` au plus (Marche en avant, spec
    /// 058) ; résolu en `Move` à l'utilisation.
    MovePerUnit {
        max: u8,
    },
    /// Défausser une Blessure de la main, puis piocher une carte (Même pas mal, spec 058).
    DiscardWoundToDraw,
    /// Jeter ce nombre de Blessures de la main dans la pile des Blessures (Rituel de guérison,
    /// spec 058).
    ThrowWounds(u8),
    /// Voler jusqu'à `spaces` cases (au sens du tour) pour `cost` Mouvement, sans provoquer (Vol,
    /// spec 058) ; la case d'arrivée est la cible de `UseSkill`.
    Flight {
        spaces: u8,
        cost: u8,
    },
    /// L'ennemi visé : Armure −1 par résistance, pour le combat (Botte secrète, spec 058).
    ResistanceBreak,
    /// Relancer un dé de la Source, ou aucun (Embrasure de la Source, spec 058).
    SourceOpening,
    /// Terrains −2 ce tour, −1 au tour suivant (Temps calme, spec 058).
    CalmWeather,
    /// Un jeton de la couleur choisie, sauf or (Surcharge de mana, spec 058).
    ManaOverload,
    /// Jusqu'à ce nombre d'ennemis choisis n'attaquent pas ce combat (Bannière de peur
    /// renforcée, spec 055).
    CancelAttacks(u8),
    /// Toutes les unités épuisées redeviennent disponibles (Bannière de courage renforcée, spec
    /// 055).
    ReadyAllUnits,
    /// Attaque physique 3 par carte défaussée de la main (Épée de justice, spec 055).
    SwordDiscard,
    /// Attaque de siège physique 5, plus le bonus choisi jusqu'à `extra` ; un dé de mana lancé
    /// (de base) ou un par point ajouté, une Blessure par noir ou rouge (Corne de la colère, spec 055).
    HornOfWrath {
        extra: u8,
    },
    /// Deux dés de mana : un cristal par couleur de base, au choix pour l'or, Gloire +1 pour le noir
    /// (Poche de mana, spec 055).
    GemPouchDice,
    /// Un jeton de chaque couleur de base, plus or de jour ou noir de nuit (Poche de mana
    /// renforcé, spec 055).
    TokensEachColorAndTime,
    /// Détruire une carte action de la main, prendre en main un sort de sa couleur dans l'offre, et
    /// un cristal de cette couleur (Livre de sagesse renforcé, spec 055).
    SpellAndCrystalFromOffer,
}

/// Où va une carte gagnée d'une offre (MKUE p. 5, *"Gain" Effects* 3 et 3.a ; spec 056) : le dessus
/// du deck par défaut, ou là où la carte le dit.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum CardDestination {
    #[default]
    DeckTop,
    DiscardPile,
    Hand,
}

impl Effect {
    /// Hors combat, seuls le mouvement et l'influence se jouent (MKUE p. 6, 5.c) ; le mana se
    /// gagne à tout moment.
    pub fn is_playable_out_of_combat(self) -> bool {
        !matches!(
            self,
            Effect::Attack { .. }
                | Effect::Block { .. }
                | Effect::IceBlockPerAbility(_)
                | Effect::SwiftBlock(_)
                | Effect::Nothing
                | Effect::AttackWithWound { .. }
                | Effect::AttackPerEnemy { .. }
                | Effect::BlockPerEnemy { .. }
                | Effect::DestroyEnemy
                // Spec 057 : rien en solo (Q2) ; Ailes de la nuit en combat seulement.
                | Effect::OthersOnly
                | Effect::DoesNotAttackPerMove
                // Spec 055 : la Bannière de peur renforcée, en combat seulement.
                | Effect::CancelAttacks(_)
                | Effect::SwordDiscard
                | Effect::HornOfWrath { .. }
                // Spec 058 : Botte secrète, en combat seulement.
                | Effect::ResistanceBreak
        )
    }

    /// La couleur du mana à payer en plus pour jouer cette option (MKUE p. 4, 6).
    pub fn mana_cost(self) -> Option<BasicColor> {
        match self {
            Effect::PayForCrystal(color) => Some(color),
            _ => None,
        }
    }

    /// L'effet avec `bonus` de plus s'il donne des points (Mouvement, Influence, Attaque, Parade) ;
    /// les autres effets ne changent pas (MKUE p. 5, « Composite » 1.b).
    pub fn with_bonus(self, bonus: u8) -> Effect {
        match self {
            Effect::Move(amount) => Effect::Move(amount + bonus),
            Effect::Influence(amount) => Effect::Influence(amount + bonus),
            Effect::Attack {
                amount,
                element,
                kind,
            } => Effect::Attack {
                amount: amount + bonus,
                element,
                kind,
            },
            Effect::Block { amount, element } => Effect::Block {
                amount: amount + bonus,
                element,
            },
            Effect::IceBlockPerAbility(amount) => Effect::IceBlockPerAbility(amount + bonus),
            Effect::SwiftBlock(amount) => Effect::SwiftBlock(amount + bonus),
            Effect::MoveAndHeal { movement, heal } => Effect::MoveAndHeal {
                movement: movement + bonus,
                heal,
            },
            Effect::InfluencePerWound { base, per } => Effect::InfluencePerWound {
                base: base + bonus,
                per,
            },
            // Spec 053 : le bonus va à l'Attaque choisie (C11).
            Effect::AttackWithWound {
                amount,
                element,
                kind,
            } => Effect::AttackWithWound {
                amount: amount + bonus,
                element,
                kind,
            },
            // Spec 040 : le bonus va à l'influence, le reste de l'effet ne change pas (L6).
            Effect::InfluenceReputation { amount, reputation } => Effect::InfluenceReputation {
                amount: amount + bonus,
                reputation,
            },
            Effect::InfluenceInInteraction {
                amount,
                fame,
                reputation,
            } => Effect::InfluenceInInteraction {
                amount: amount + bonus,
                fame,
                reputation,
            },
            other => other,
        }
    }

    /// Vrai si l'effet se répète avec Effet maximum (spec 056, Q1) : des points, un gain de mana ou
    /// de cartes, avec réputation, gloire ou Blessure. L'option ne doit en plus poser aucun
    /// modificateur ni coûter de mana ou de carte (vérifié par le moteur).
    pub fn is_repeatable(self) -> bool {
        matches!(
            self,
            Effect::Move(_)
                | Effect::Influence(_)
                | Effect::Attack { .. }
                | Effect::Block { .. }
                | Effect::SwiftBlock(_)
                | Effect::Heal(_)
                | Effect::DrawCards(_)
                | Effect::InfluenceReputation { .. }
                | Effect::InfluenceInInteraction { .. }
                | Effect::GainToken(_)
                | Effect::GainCrystal(_)
                | Effect::AttackWithWound { .. }
                | Effect::MoveAndHeal { .. }
                | Effect::InfluencePerWound { .. }
        )
    }

    /// L'effet répété `times` fois en un seul effet (spec 056 ; MKUE p. 5, *"Composite" Effects*
    /// 1.a) : points, cartes piochées, réputation et gloire multipliés. Les jetons, les cristaux et
    /// les Blessures, gagnés un par un, ne changent pas ici.
    pub fn repeated(self, times: u8) -> Effect {
        let times_i = |value: i8| value.saturating_mul(i8::try_from(times).unwrap_or(i8::MAX));
        match self {
            Effect::Move(amount) => Effect::Move(amount.saturating_mul(times)),
            Effect::Influence(amount) => Effect::Influence(amount.saturating_mul(times)),
            Effect::Attack {
                amount,
                element,
                kind,
            } => Effect::Attack {
                amount: amount.saturating_mul(times),
                element,
                kind,
            },
            Effect::Block { amount, element } => Effect::Block {
                amount: amount.saturating_mul(times),
                element,
            },
            Effect::SwiftBlock(amount) => Effect::SwiftBlock(amount.saturating_mul(times)),
            Effect::Heal(amount) => Effect::Heal(amount.saturating_mul(times)),
            Effect::DrawCards(count) => Effect::DrawCards(count.saturating_mul(times)),
            Effect::InfluenceReputation { amount, reputation } => Effect::InfluenceReputation {
                amount: amount.saturating_mul(times),
                reputation: times_i(reputation),
            },
            Effect::InfluenceInInteraction {
                amount,
                fame,
                reputation,
            } => Effect::InfluenceInInteraction {
                amount: amount.saturating_mul(times),
                fame: fame.saturating_mul(times),
                reputation: times_i(reputation),
            },
            Effect::AttackWithWound {
                amount,
                element,
                kind,
            } => Effect::AttackWithWound {
                amount: amount.saturating_mul(times),
                element,
                kind,
            },
            Effect::MoveAndHeal { movement, heal } => Effect::MoveAndHeal {
                movement: movement.saturating_mul(times),
                heal: heal.saturating_mul(times),
            },
            Effect::InfluencePerWound { base, per } => Effect::InfluencePerWound {
                base: base.saturating_mul(times),
                per: per.saturating_mul(times),
            },
            other => other,
        }
    }

    /// L'effet tel qu'il compte une fois joué (spec 053) : l'Attaque de Coup de sang est une
    /// Attaque ; Déluge de flammes, sans les ennemis comptés (spec 054) ; les autres effets ne
    /// changent pas.
    pub fn resolved(self) -> Effect {
        match self {
            Effect::AttackWithWound {
                amount,
                element,
                kind,
            } => Effect::Attack {
                amount,
                element,
                kind,
            },
            Effect::AttackPerEnemy {
                amount, element, ..
            } => Effect::Attack {
                amount,
                element,
                kind: AttackKind::Melee,
            },
            Effect::BlockPerEnemy {
                amount, element, ..
            } => Effect::Block { amount, element },
            // Spec 055 : la gloire de la Bourse sans fond vient par son événement.
            Effect::InfluenceAndFame { amount, .. } => Effect::Influence(amount),
            other => other,
        }
    }

    /// Déluge de flammes (spec 054, Q4) : l'Attaque ou la Parade obtenue contre `faced` jetons
    /// ennemis affrontés, `per` de plus pour chacun ; les autres effets ne changent pas.
    pub fn against_enemies(self, faced: usize) -> Effect {
        let (Effect::AttackPerEnemy { per, .. } | Effect::BlockPerEnemy { per, .. }) = self else {
            return self;
        };
        let bonus = u8::try_from(usize::from(per).saturating_mul(faced)).unwrap_or(u8::MAX);
        match self.resolved() {
            Effect::Attack {
                amount,
                element,
                kind,
            } => Effect::Attack {
                amount: amount.saturating_add(bonus),
                element,
                kind,
            },
            Effect::Block { amount, element } => Effect::Block {
                amount: amount.saturating_add(bonus),
                element,
            },
            other => other,
        }
    }

    /// Les points de l'effet, s'il en donne.
    pub fn points(self) -> Option<u8> {
        match self {
            Effect::Move(amount)
            | Effect::Influence(amount)
            | Effect::InfluenceReputation { amount, .. }
            | Effect::InfluenceInInteraction { amount, .. } => Some(amount),
            Effect::Attack { amount, .. }
            | Effect::Block { amount, .. }
            | Effect::IceBlockPerAbility(amount)
            | Effect::SwiftBlock(amount)
            | Effect::AttackWithWound { amount, .. }
            | Effect::AttackPerEnemy { amount, .. }
            | Effect::BlockPerEnemy { amount, .. } => Some(amount),
            Effect::MoveAndHeal { movement, .. } => Some(movement),
            Effect::InfluencePerWound { base, .. } => Some(base),
            Effect::InfluenceAndFame { amount, .. } => Some(amount),
            Effect::InteractionInfluence(amount) => Some(amount),
            Effect::CrystalColorInfluence
            | Effect::MovePerUnit { .. }
            | Effect::DiscardWoundToDraw
            | Effect::ThrowWounds(_)
            | Effect::Flight { .. }
            | Effect::ResistanceBreak
            | Effect::SourceOpening
            | Effect::CalmWeather
            | Effect::ManaOverload => None,
            Effect::GainToken(_)
            | Effect::BannerAssigned
            | Effect::AmuletOfSun { .. }
            | Effect::AmuletOfDarkness { .. }
            | Effect::TokenCrystalAndFame(_)
            | Effect::CrystalAndToken { .. }
            | Effect::DrawAndToken { .. }
            | Effect::HealUnit
            | Effect::CancelAttacks(_)
            | Effect::ReadyAllUnits
            | Effect::SwordDiscard
            | Effect::HornOfWrath { .. }
            | Effect::GemPouchDice
            | Effect::TokensEachColorAndTime
            | Effect::SpellAndCrystalFromOffer
            | Effect::GainCrystal(_)
            | Effect::PayForCrystal(_)
            | Effect::ExtraDie
            | Effect::ExtraDieBlackAnyColor
            | Effect::SetDice { .. }
            | Effect::Boost(_)
            | Effect::Heal(_)
            | Effect::DrawCards(_)
            | Effect::ReadyUnit(_)
            | Effect::TokenAndCrystal(_)
            | Effect::GainTokens(..)
            | Effect::Nothing
            | Effect::WoundsPerCrystal(_)
            | Effect::OthersOnly
            | Effect::HealByTerrain { .. }
            | Effect::HealAndReadyLevels { .. }
            | Effect::ReadyAnyUnit { .. }
            | Effect::UndergroundMove { .. }
            | Effect::FlyMove { .. }
            | Effect::DoesNotAttackPerMove
            | Effect::DiscardToDeck { .. }
            | Effect::ClaimDie { .. }
            | Effect::HealAndReady { .. }
            | Effect::WoundCrystalAndToken { .. }
            | Effect::WoundAndTokens(_)
            | Effect::CrystalsFromDestroyed(_)
            | Effect::CrystalsOtherColors
            | Effect::GainOwnedCrystal(_)
            | Effect::CrystalFromDie
            | Effect::RerollSource
            | Effect::Repeat(_)
            | Effect::GainSpell(_)
            | Effect::TrainFromOffer(_)
            | Effect::DestroyEnemy
            | Effect::CastFromOffer => None,
        }
    }
}

/// Ce qu'il faut payer en plus pour jouer l'effet (MKUE p. 4, 6).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum Cost {
    None,
    DiscardAnotherCard,
    /// Détruire une autre carte action de la main (Décomposition, Entraînement ; spec 056) :
    /// choisie dans le champ `discard` de `CardPlay`.
    DestroyActionCard,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BasicEffect {
    /// Une des options, au choix du joueur.
    Options {
        options: &'static [Effect],
        cost: Cost,
        /// Les modificateurs posés par chaque option, dans l'ordre des options (spec 052, ADR
        /// 0008) ; vide quand aucune option n'en pose.
        modifiers: &'static [Vec<ModifierKind>],
        /// Le mana payé en plus pour chaque option, dans l'ordre des options (Magie pure, Mélodie
        /// du vent ; spec 053) ; vide quand aucune option n'en coûte.
        mana: &'static [Option<BasicColor>],
    },
    /// Mana, cristal, soin, pioche, unité : décrit par la spec qui l'apporte.
    Unsupported,
}

impl BasicEffect {
    /// L'option `option` de cet effet, si elle existe.
    pub fn option(self, option: u8) -> Option<Effect> {
        match self {
            BasicEffect::Options { options, .. } => options.get(usize::from(option)).copied(),
            BasicEffect::Unsupported => None,
        }
    }

    /// Le mana à payer en plus pour l'option `option` (MKUE p. 4, 6) : celui de l'effet
    /// (Cristallisation, spec 025) ou celui de l'option (Magie pure, spec 053).
    pub fn option_mana(self, option: u8) -> Option<BasicColor> {
        match self {
            BasicEffect::Options { options, mana, .. } => mana
                .get(usize::from(option))
                .copied()
                .flatten()
                .or_else(|| options.get(usize::from(option)).and_then(|e| e.mana_cost())),
            BasicEffect::Unsupported => None,
        }
    }

    /// Les modificateurs posés par l'option `option` (spec 052) ; aucun si elle n'en pose pas.
    pub fn option_modifiers(self, option: u8) -> &'static [ModifierKind] {
        match self {
            BasicEffect::Options { modifiers, .. } => modifiers
                .get(usize::from(option))
                .map_or(&[], Vec::as_slice),
            BasicEffect::Unsupported => &[],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CardDefinition {
    /// Couleur de l'effet renforcé (MKUE p. 4, 3.a).
    pub color: BasicColor,
    pub basic: BasicEffect,
    /// Même forme que l'effet de base ; se paie d'un mana de `color` (spec 022).
    pub strong: BasicEffect,
    /// Jouée pour son effet, on peut la reprendre en main à la fin du tour en défaussant une
    /// autre carte (Euphorie cristalline, spec 025).
    pub reclaim: bool,
    /// Carte de soin : aucun de ses effets ne se joue en combat, de côté si (spec 034, Q3).
    pub healing: bool,
}

const fn attack(amount: u8) -> Effect {
    Effect::Attack {
        amount,
        element: Element::Physical,
        kind: AttackKind::Melee,
    }
}

const fn block(amount: u8) -> Effect {
    Effect::Block {
        amount,
        element: Element::Physical,
    }
}

impl BasicAction {
    pub const ALL: [BasicAction; 20] = [
        BasicAction::Rage,
        BasicAction::Determination,
        BasicAction::Swiftness,
        BasicAction::March,
        BasicAction::Stamina,
        BasicAction::Tranquility,
        BasicAction::Promise,
        BasicAction::Threaten,
        BasicAction::Crystallize,
        BasicAction::ManaDraw,
        BasicAction::Concentration,
        BasicAction::Improvisation,
        BasicAction::BattleVersatility,
        BasicAction::ManaPull,
        BasicAction::WillFocus,
        BasicAction::CrystalJoy,
        BasicAction::NobleManners,
        BasicAction::Rejuvenate,
        BasicAction::ColdToughness,
        BasicAction::Instinct,
    ];

    /// Lue dans `content/basic_actions.ron` (spec 029).
    pub fn definition(self) -> CardDefinition {
        GameContent::embedded().card(self)
    }

    /// Clé du nom de la carte : arcanum-core ne manipule que des clés (ADR 0003).
    pub fn label_key(self) -> &'static str {
        use BasicAction::*;
        match self {
            Rage => "card.rage.name",
            Determination => "card.determination.name",
            Swiftness => "card.swiftness.name",
            March => "card.march.name",
            Stamina => "card.stamina.name",
            Tranquility => "card.tranquility.name",
            Promise => "card.promise.name",
            Threaten => "card.threaten.name",
            Crystallize => "card.crystallize.name",
            ManaDraw => "card.mana_draw.name",
            Concentration => "card.concentration.name",
            Improvisation => "card.improvisation.name",
            BattleVersatility => "card.battle_versatility.name",
            ManaPull => "card.mana_pull.name",
            WillFocus => "card.will_focus.name",
            CrystalJoy => "card.crystal_joy.name",
            NobleManners => "card.noble_manners.name",
            Rejuvenate => "card.rejuvenate.name",
            ColdToughness => "card.cold_toughness.name",
            Instinct => "card.instinct.name",
        }
    }
}

impl DeedCard {
    /// `None` pour une Blessure : ni couleur, ni effet (MKUE p. 4, 3.d). L'appelant doit le
    /// traiter plutôt que lui prêter une couleur. `None` aussi pour un artefact, sans couleur :
    /// aucun ne peut être gagné avant la spec 044, sa façon de se jouer arrive avec la spec 055.
    pub fn definition(self) -> Option<CardDefinition> {
        match self {
            DeedCard::Basic(action) => Some(action.definition()),
            DeedCard::Advanced(card) => Some(GameContent::embedded().advanced_action(card)),
            DeedCard::Spell(card) => Some(GameContent::embedded().spell(card)),
            DeedCard::Wound | DeedCard::Artifact(_) => None,
        }
    }

    pub fn label_key(self) -> &'static str {
        match self {
            DeedCard::Basic(action) => action.label_key(),
            DeedCard::Wound => "card.wound.name",
            DeedCard::Advanced(card) => card.label_key(),
            DeedCard::Spell(card) => card.label_key(),
            DeedCard::Artifact(card) => card.label_key(),
        }
    }

    /// Clés des textes écrits par nous des effets de base et renforcé non codés (spec 050, Q1) ;
    /// `None` pour une carte d'action de base ou une Blessure.
    pub fn unsupported_text_keys(self) -> Option<(&'static str, &'static str)> {
        match self {
            DeedCard::Advanced(card) => Some((card.basic_text_key(), card.strong_text_key())),
            DeedCard::Spell(card) => Some((card.basic_text_key(), card.strong_text_key())),
            DeedCard::Artifact(card) => Some((card.basic_text_key(), card.strong_text_key())),
            DeedCard::Basic(_) | DeedCard::Wound => None,
        }
    }

    pub fn is_advanced_action(self) -> bool {
        matches!(self, DeedCard::Advanced(_))
    }

    pub fn is_spell(self) -> bool {
        matches!(self, DeedCard::Spell(_))
    }

    pub fn is_artifact(self) -> bool {
        matches!(self, DeedCard::Artifact(_))
    }

    pub fn is_wound(self) -> bool {
        self == DeedCard::Wound
    }
}

/// Jouer de côté : 1 point d'une de ces sortes (MKUE p. 4, 3.e).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum SidewaysEffect {
    Move,
    Influence,
    Attack,
    Block,
}

impl SidewaysEffect {
    pub fn effect(self) -> Effect {
        self.effect_with(1)
    }

    /// Les points de côté d'une autre valeur (compétences, spec 058).
    pub fn effect_with(self, amount: u8) -> Effect {
        match self {
            SidewaysEffect::Move => Effect::Move(amount),
            SidewaysEffect::Influence => Effect::Influence(amount),
            SidewaysEffect::Attack => attack(amount),
            SidewaysEffect::Block => block(amount),
        }
    }
}

/// Façon de jouer une carte.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum CardPlay {
    /// Option `option` de l'effet de base, avec la carte à défausser si l'option le demande, et
    /// le mana payé si elle en coûte (spec 025).
    Basic {
        option: u8,
        discard: Option<CardId>,
        mana: Option<ManaPayment>,
        /// La carte jouée avec Effet maximum, et son option (spec 056) ; `None` pour toute autre
        /// carte.
        #[serde(default)]
        with: Option<BoostTarget>,
        /// La carte prise dans une offre (Entraînement, spec 056).
        #[serde(default)]
        gain: Option<CardId>,
        /// Le dé de la Source choisi, par sa position (Tempête de mana, spec 056).
        #[serde(default)]
        die: Option<u8>,
    },
    Sideways(SidewaysEffect),
    /// De côté avec une compétence qui en change la valeur (spec 058, groupe A ; avantage du
    /// Rituel de guérison) : son option, et le mana ajouté par Pouvoir universel.
    SidewaysSkill {
        sideways: SidewaysEffect,
        skill: crate::Skill,
        option: u8,
        mana: Option<SkillMana>,
    },
    /// Option `option` de l'effet renforcé, payée par `mana` (spec 022), avec les dés mis sur une
    /// couleur si l'effet en demande (spec 025), dans l'ordre de la Source.
    Strong {
        option: u8,
        discard: Option<CardId>,
        mana: ManaPayment,
        set_dice: [Option<DieSetting>; 2],
        /// La carte jouée avec Concentration renforcée (spec 025) ; `None` pour toute autre carte.
        with: Option<BoostTarget>,
        /// Le mana en plus d'une option qui en coûte un (Magie pure, Mélodie du vent ; spec 053).
        #[serde(default)]
        extra_mana: Option<ManaPayment>,
        /// La carte prise dans une offre (Entraînement, Don pour la magie ; spec 056).
        #[serde(default)]
        gain: Option<CardId>,
    },
    /// Option `option` de l'effet renforcé, obtenue sans payer avec Concentration (spec 025).
    Boosted {
        option: u8,
        discard: Option<CardId>,
        set_dice: [Option<DieSetting>; 2],
    },
    /// Un sort lancé (spec 054) : son effet de base payé du mana de sa couleur (`mana`), ou son
    /// effet renforcé, payé en plus d'un mana noir (`black`) ; le mana en plus de l'option
    /// (Éclair de mana), l'ennemi visé, et s'il est lancé depuis l'offre de sorts (Don pour la
    /// magie).
    Cast {
        option: u8,
        mana: ManaPayment,
        /// `Some` pour l'effet renforcé (MKUE p. 4, 3.b ; p. 5, 2.b).
        black: Option<ManaPayment>,
        extra_mana: Option<ManaPayment>,
        enemy: Option<EnemyToken>,
        from_offer: bool,
        /// Les choix propres aux sorts de la spec 057 (arrivée, unités, ennemis, dé, unité de
        /// l'offre).
        #[serde(default)]
        targets: CastTargets,
    },
    /// L'effet de base d'un artefact, sans mana, et ses choix (spec 055).
    Artifact {
        option: u8,
        #[serde(default)]
        choices: ArtifactChoices,
    },
    /// L'effet renforcé d'un artefact, sans mana : la carte est détruite (MKUE p. 4, 3.c ; spec 055).
    ThrowAway {
        option: u8,
        #[serde(default)]
        choices: ArtifactChoices,
    },
}

/// Les choix d'un artefact joué (spec 055), vides pour les autres : les cartes défaussées (Épée de
/// justice) ou détruite (Livre de sagesse), les ennemis visés (Bannière de peur renforcée), le
/// bonus de la Corne de la colère renforcée, les couleurs des jetons (Amulette des ténèbres), la
/// carte prise dans une offre (Livre de sagesse).
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct ArtifactChoices {
    #[serde(default)]
    pub discard: Picks<CardId>,
    #[serde(default)]
    pub enemies: Picks<EnemyToken>,
    #[serde(default)]
    pub bonus: u8,
    #[serde(default)]
    pub colors: Picks<ManaColor>,
    #[serde(default)]
    pub gain: Option<CardId>,
}

/// Taille d'une `Picks` : plus que les unités d'un joueur et les ennemis d'un combat ordinaire.
pub const PICKS: usize = 8;

/// Une liste courte, `Copy`, pour garder `CardPlay` copiable (spec 057) : les unités de
/// Renaissance, les ennemis d'Ailes de la nuit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Picks<T: Copy>([Option<T>; PICKS]);

impl<T: Copy> Default for Picks<T> {
    fn default() -> Self {
        Picks([None; PICKS])
    }
}

impl<T: Copy + PartialEq> Picks<T> {
    /// Les `PICKS` premiers éléments de `items`.
    pub fn of(items: &[T]) -> Self {
        let mut picks = Self::default();
        for (slot, &item) in picks.0.iter_mut().zip(items) {
            *slot = Some(item);
        }
        picks
    }

    pub fn iter(&self) -> impl Iterator<Item = T> + '_ {
        self.0.iter().flatten().copied()
    }

    pub fn to_vec(&self) -> Vec<T> {
        self.iter().collect()
    }

    pub fn len(&self) -> usize {
        self.iter().count()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn contains(&self, item: T) -> bool {
        self.iter().any(|picked| picked == item)
    }
}

/// Les choix d'un sort de la spec 057, vides pour les autres : l'arrivée d'un déplacement par
/// effet, les unités visées, les ennemis d'Ailes de la nuit, le dé de Puits de mana.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct CastTargets {
    #[serde(default)]
    pub destination: Option<Hex>,
    #[serde(default)]
    pub units: Picks<UnitCard>,
    #[serde(default)]
    pub enemies: Picks<EnemyToken>,
    #[serde(default)]
    pub die: Option<u8>,
}

impl CardPlay {
    /// Vrai pour un effet renforcé : payé, obtenu par Concentration, ou sort payé du noir.
    pub fn is_strong(self) -> bool {
        match self {
            CardPlay::Strong { .. } | CardPlay::Boosted { .. } => true,
            CardPlay::Cast { black, .. } => black.is_some(),
            CardPlay::ThrowAway { .. } => true,
            CardPlay::Basic { .. }
            | CardPlay::Sideways(_)
            | CardPlay::SidewaysSkill { .. }
            | CardPlay::Artifact { .. } => false,
        }
    }
}

/// La carte jouée avec Concentration renforcée (spec 025) ou avec Effet maximum (spec 056), et ses
/// propres choix.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct BoostTarget {
    pub card: CardId,
    pub option: u8,
    pub discard: Option<CardId>,
    pub set_dice: [Option<DieSetting>; 2],
}

/// Un dé de la Source mis sur une couleur de base (spec 025) : jamais l'or, ni le noir tant qu'il
/// n'y a pas de sorts (Q4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct DieSetting {
    /// Position du dé dans la Source, à partir de 0.
    pub die: u8,
    pub color: BasicColor,
}

/// D'où vient le mana payé (MKUE p. 5, « Using mana » 1) : la couleur est celle de la carte
/// pour un effet renforcé, celle de l'option pour un effet de base payant (spec 025).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ManaPayment {
    /// Le dé de la Source à cette position, à partir de 0 (MKUE p. 5, 3).
    Die(u8),
    /// Un cristal de l'inventaire (MKUE p. 5, 1.b).
    Crystal,
    /// Un jeton de mana de la zone de jeu (spec 025).
    Token,
    /// Le mana illimité d'un anneau renforcé, de cette couleur ou noir, sans dé ni jeton (spec 055).
    Endless(ManaColor),
    /// Invocation (spec 058) : cette carte de la main défaussée pour un jeton dépensé aussitôt,
    /// rouge ou noir pour une Blessure, blanc ou vert pour une autre carte.
    Invocation(CardId),
    /// Polarisation (spec 058) : un mana de la couleur `from`, venu de `source`, payé comme la
    /// couleur opposée (ou noir de jour comme toute couleur de base, or de nuit comme noir).
    Polarized {
        from: ManaColor,
        source: PolarSource,
    },
    /// Le dé posé sur Vol de mana (spec 060) : pas un dé de la Source du tour.
    StolenDie,
}

/// D'où vient le mana changé par Polarisation (spec 058) : un dé, un cristal, un jeton, ou une
/// Invocation (Compendium, *Invocation* S3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum PolarSource {
    Die(u8),
    Crystal,
    Token,
    Invocation(CardId),
    /// Le dé de Vol de mana (spec 060).
    StolenDie,
}

/// Le mana ajouté à une carte jouée de côté par Pouvoir universel (spec 058) : sa couleur (de base,
/// ou noir de nuit) et son paiement.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct SkillMana {
    pub color: ManaColor,
    pub payment: ManaPayment,
}

/// Une carte de la zone de jeu, avec la façon dont elle a été jouée et l'effet obtenu.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PlayedCard {
    pub card: CardId,
    pub play: CardPlay,
    pub effect: Effect,
}
