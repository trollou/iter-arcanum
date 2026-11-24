//! Unités du jeu de base (spec 050) : identifiants, valeurs imprimées et capacités (spec 042), lues
//! dans `content/units.ron`.

use serde::Deserialize;

use crate::{BasicColor, CardId, Effect, GameContent, ManaColor, ModifierKind, Resistance};

/// Les 11 unités régulières puis les 9 d'élite, dans l'ordre des numéros n/20.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, serde::Serialize,
)]
pub enum Unit {
    Peasants,
    Herbalists,
    Foresters,
    UtemCrossbowmen,
    UtemGuardsmen,
    UtemSwordsmen,
    GuardianGolems,
    Illusionists,
    RedCapeMonks,
    NorthernMonks,
    SavageMonks,
    FireGolems,
    IceGolems,
    FireMages,
    IceMages,
    AmotepGunners,
    AmotepFreezers,
    Catapults,
    AltemMages,
    AltemGuardians,
}

/// Dos argent ou dos or : la pioche de l'unité (MKUE p. 3).
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, serde::Serialize,
)]
pub enum UnitRank {
    Regular,
    Elite,
}

/// Lieux de recrutement, les icônes de la colonne de gauche (MKUE p. 8, 4.a).
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, serde::Serialize,
)]
pub enum RecruitSite {
    Village,
    Monastery,
    Keep,
    MageTower,
    City,
}

impl RecruitSite {
    pub fn label_key(self) -> &'static str {
        match self {
            RecruitSite::Village => "unit.recruit.village",
            RecruitSite::Monastery => "unit.recruit.monastery",
            RecruitSite::Keep => "unit.recruit.keep",
            RecruitSite::MageTower => "unit.recruit.mage_tower",
            RecruitSite::City => "unit.recruit.city",
        }
    }
}

/// Valeurs imprimées sur une carte d'unité (MKUE p. 5, *Using Units* 1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnitDefinition {
    pub rank: UnitRank,
    /// Exemplaires dans la pioche.
    pub count: u8,
    /// Coût de recrutement, en Influence.
    pub influence: u8,
    /// Niveau, I à IV.
    pub level: u8,
    pub armor: u8,
    pub resistances: Vec<Resistance>,
    pub recruit: Vec<RecruitSite>,
    /// Les capacités, dans l'ordre de la carte (spec 042).
    pub abilities: Vec<UnitAbility>,
}

/// Une capacité imprimée sur une carte d'unité (MKUE p. 5, 5 ; spec 042).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnitAbility {
    /// Le mana à payer d'abord, comme un effet renforcé (MKUE p. 5, 5.b) ; noir pour une capacité
    /// des Mages d'Altem.
    pub mana: Option<ManaColor>,
    pub effect: AbilityEffect,
}

/// Ce que fait une capacité : une option au choix, ou rien que le moteur sache jouer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AbilityEffect {
    Options {
        options: Vec<AbilityOption>,
        /// L'unité reçoit une Blessure en s'activant ainsi (Épéistes d'Utem ; Compendium,
        /// *UTEM SWORDSMEN* S2).
        wounds_unit: bool,
    },
    /// Non activable avant le moteur d'effets (spec 052 ; spec 042, Q2).
    Unsupported,
}

/// Une option d'une capacité : son effet, et le mana payé en plus pour l'obtenir (Feu glacé 7 ou
/// 9 des Mages d'Altem).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AbilityOption {
    pub effect: Effect,
    pub extra_mana: Vec<BasicColor>,
    /// Les modificateurs posés par cette option (Bûcherons, spec 052).
    pub modifiers: Vec<ModifierKind>,
}

impl AbilityOption {
    pub fn new(effect: Effect) -> Self {
        Self {
            effect,
            extra_mana: Vec::new(),
            modifiers: Vec::new(),
        }
    }
}

impl UnitAbility {
    /// Vrai si le moteur sait jouer cette capacité.
    pub fn is_supported(&self) -> bool {
        matches!(self.effect, AbilityEffect::Options { .. })
    }

    /// Les options de la capacité ; aucune si elle n'est pas codée.
    pub fn options(&self) -> &[AbilityOption] {
        match &self.effect {
            AbilityEffect::Options { options, .. } => options,
            AbilityEffect::Unsupported => &[],
        }
    }

    /// Vrai si l'unité reçoit une Blessure en s'activant ainsi (Épéistes).
    pub fn wounds_unit(&self) -> bool {
        matches!(
            self.effect,
            AbilityEffect::Options {
                wounds_unit: true,
                ..
            }
        )
    }
}

/// Un exemplaire de carte d'unité : l'unité et son numéro d'exemplaire, comme `CardId`. Jamais
/// une carte Acte (spec 050).
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct UnitCard {
    pub unit: Unit,
    pub copy: u8,
}

/// Une unité de la zone des unités du joueur (MKUE p. 5, 2 et 3 ; spec 042) : disponible ou
/// épuisée, blessée ou non, deux états indépendants.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct OwnedUnit {
    pub card: UnitCard,
    /// Disponible : peut s'activer (épuisée sinon, jusqu'à la nouvelle manche).
    pub ready: bool,
    /// Blessures posées sur la carte, 0 à 2.
    pub wounds: u8,
    /// A reçu des dégâts dans le combat en cours (FAQ p3 ; Compendium, *Combat* S35).
    pub damaged: bool,
    /// La bannière assignée à l'unité (spec 055 ; MKUE p. 5, 8).
    #[serde(default)]
    pub banner: Option<CardId>,
    /// La Bannière de courage retournée jusqu'à la prochaine manche (spec 055).
    #[serde(default)]
    pub banner_flipped: bool,
}

impl OwnedUnit {
    /// Une unité recrutée : disponible, non blessée (MKUE p. 8, 4.a).
    pub fn recruited(card: UnitCard) -> Self {
        Self {
            card,
            ready: true,
            wounds: 0,
            damaged: false,
            banner: None,
            banner_flipped: false,
        }
    }

    pub fn is_wounded(&self) -> bool {
        self.wounds > 0
    }
}

impl UnitCard {
    pub fn new(unit: Unit, copy: u8) -> Self {
        Self { unit, copy }
    }
}

impl Unit {
    pub const ALL: [Unit; 20] = [
        Unit::Peasants,
        Unit::Herbalists,
        Unit::Foresters,
        Unit::UtemCrossbowmen,
        Unit::UtemGuardsmen,
        Unit::UtemSwordsmen,
        Unit::GuardianGolems,
        Unit::Illusionists,
        Unit::RedCapeMonks,
        Unit::NorthernMonks,
        Unit::SavageMonks,
        Unit::FireGolems,
        Unit::IceGolems,
        Unit::FireMages,
        Unit::IceMages,
        Unit::AmotepGunners,
        Unit::AmotepFreezers,
        Unit::Catapults,
        Unit::AltemMages,
        Unit::AltemGuardians,
    ];

    /// Lue dans `content/units.ron`.
    pub fn definition(self) -> &'static UnitDefinition {
        GameContent::embedded().unit(self)
    }

    /// Clé du nom : `unit.<id>.name`.
    pub fn label_key(self) -> &'static str {
        use Unit::*;
        match self {
            Peasants => "unit.peasants.name",
            Herbalists => "unit.herbalists.name",
            Foresters => "unit.foresters.name",
            UtemCrossbowmen => "unit.utem_crossbowmen.name",
            UtemGuardsmen => "unit.utem_guardsmen.name",
            UtemSwordsmen => "unit.utem_swordsmen.name",
            GuardianGolems => "unit.guardian_golems.name",
            Illusionists => "unit.illusionists.name",
            RedCapeMonks => "unit.red_cape_monks.name",
            NorthernMonks => "unit.northern_monks.name",
            SavageMonks => "unit.savage_monks.name",
            FireGolems => "unit.fire_golems.name",
            IceGolems => "unit.ice_golems.name",
            FireMages => "unit.fire_mages.name",
            IceMages => "unit.ice_mages.name",
            AmotepGunners => "unit.amotep_gunners.name",
            AmotepFreezers => "unit.amotep_freezers.name",
            Catapults => "unit.catapults.name",
            AltemMages => "unit.altem_mages.name",
            AltemGuardians => "unit.altem_guardians.name",
        }
    }

    /// Clé du texte de la capacité `ability` (à partir de 0), écrit par nous : `ability_1` pour la
    /// première (spec 050, Q1 ; une clé par capacité depuis la spec 042).
    pub fn ability_key(self, ability: usize) -> String {
        let prefix = self.label_key().trim_end_matches("name");
        format!("{prefix}ability_{}", ability + 1)
    }
}
