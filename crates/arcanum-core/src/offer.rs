//! Pioches communes et offres (spec 050) : actions avancées, sorts, artefacts, unités.

use crate::{
    AdvancedAction, Artifact, CardId, DeedCard, GameContent, Spell, SpellSet, UnitCard, UnitRank,
};

/// Les pioches communes, face cachée (MKUE p. 3). « Pioche » et non « deck » : le deck est celui
/// du joueur.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum CommonDeck {
    AdvancedActions,
    Spells,
    Artifacts,
    RegularUnits,
    EliteUnits,
}

impl CommonDeck {
    pub const ALL: [CommonDeck; 5] = [
        CommonDeck::AdvancedActions,
        CommonDeck::Spells,
        CommonDeck::Artifacts,
        CommonDeck::RegularUnits,
        CommonDeck::EliteUnits,
    ];
}

/// Les trois offres, face visible (MKUE p. 3). Une offre est ordonnée : la première place en
/// tête.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum Offer {
    AdvancedActions,
    Spells,
    Units,
}

impl Offer {
    pub const ALL: [Offer; 3] = [Offer::AdvancedActions, Offer::Spells, Offer::Units];
}

/// Une carte d'une pioche commune : une carte Acte (action avancée, sort, artefact) ou une carte
/// d'unité.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum CommonCard {
    Deed(CardId),
    Unit(UnitCard),
}

impl CommonCard {
    /// La pioche d'où vient la carte, et où elle retourne (MKUE p. 4, 2.c à 2.e).
    pub fn deck(self) -> CommonDeck {
        match self {
            CommonCard::Deed(card) => match card.card() {
                DeedCard::Spell(_) => CommonDeck::Spells,
                DeedCard::Artifact(_) => CommonDeck::Artifacts,
                DeedCard::Basic(_) | DeedCard::Wound | DeedCard::Advanced(_) => {
                    CommonDeck::AdvancedActions
                }
            },
            CommonCard::Unit(unit) => match unit.unit.definition().rank {
                UnitRank::Regular => CommonDeck::RegularUnits,
                UnitRank::Elite => CommonDeck::EliteUnits,
            },
        }
    }

    /// La carte Acte, pour une action avancée, un sort ou un artefact.
    pub fn deed(self) -> Option<CardId> {
        match self {
            CommonCard::Deed(card) => Some(card),
            CommonCard::Unit(_) => None,
        }
    }

    pub fn unit(self) -> Option<UnitCard> {
        match self {
            CommonCard::Unit(unit) => Some(unit),
            CommonCard::Deed(_) => None,
        }
    }
}

/// Les actions avancées d'un scénario (spec 050).
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum AdvancedActionSet {
    /// Les 28.
    #[default]
    All,
    /// De Fire Bolt à Regeneration, numéros 1 à 16 (Première reconnaissance ; Compendium,
    /// *Miscellaneous* S8).
    FirstSixteen,
}

/// Une carte de chaque, un seul exemplaire : le numéro d'exemplaire est 1.
fn deed(card: DeedCard) -> CommonCard {
    CommonCard::Deed(CardId::new(card, 1))
}

/// Les cartes d'une pioche commune avant mélange, dans l'ordre des numéros imprimés (MKUE p. 3).
pub(crate) fn common_deck_cards(
    deck: CommonDeck,
    set: AdvancedActionSet,
    spells: SpellSet,
) -> Vec<CommonCard> {
    let content = GameContent::embedded();
    match deck {
        CommonDeck::AdvancedActions => AdvancedAction::ALL
            .into_iter()
            .filter(|&card| {
                set == AdvancedActionSet::All || content.advanced_action_number(card) <= 16
            })
            .map(|card| deed(DeedCard::Advanced(card)))
            .collect(),
        CommonDeck::Spells => Spell::ALL
            .into_iter()
            .filter(|&card| spells == SpellSet::All || !card.is_competitive())
            .map(|card| deed(DeedCard::Spell(card)))
            .collect(),
        CommonDeck::Artifacts => Artifact::ALL
            .into_iter()
            .map(|card| deed(DeedCard::Artifact(card)))
            .collect(),
        CommonDeck::RegularUnits => content
            .unit_cards(UnitRank::Regular)
            .into_iter()
            .map(CommonCard::Unit)
            .collect(),
        CommonDeck::EliteUnits => content
            .unit_cards(UnitRank::Elite)
            .into_iter()
            .map(CommonCard::Unit)
            .collect(),
    }
}
