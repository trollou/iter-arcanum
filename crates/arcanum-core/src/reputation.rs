//! Piste de réputation (spec 040) : quinze cases de −7 à +7, lues dans `content/reputation.ron`.

use crate::{GameContent, Site};

/// La réputation du héros, sur la piste : de −7 (la case X) à +7, 0 à la mise en place (MKUE
/// p. 3). Elle ne sort jamais de la piste (Compendium, *Influence* S5).
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct Reputation(i8);

impl Reputation {
    /// La case X, au bas de la piste.
    pub const MIN: i8 = -7;
    pub const MAX: i8 = 7;

    /// La réputation `value`, ramenée sur la piste.
    pub fn new(value: i8) -> Self {
        Reputation(value.clamp(Self::MIN, Self::MAX))
    }

    pub fn value(self) -> i8 {
        self.0
    }

    /// Sur la case X : pas d'interaction (MKUE p. 8, 2.a).
    pub fn is_x(self) -> bool {
        self.0 == Self::MIN
    }

    /// La réputation après un changement de `change`, bornée à la piste : on peut encore en
    /// perdre sur X, sans effet (Compendium, *Influence* S5 ; *Combat* Q16).
    pub fn changed(self, change: i8) -> Self {
        Reputation::new(self.0.saturating_add(change))
    }
}

/// L'interaction en cours ce tour : le site et le bonus de réputation compté à son début
/// (MKUE p. 8, 2.a ; spec 040).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Interaction {
    pub site: Site,
    pub bonus: i8,
    /// Spec 062 : les Boucliers du joueur sur la cité, comptés au début (MKUE p. 8, 3.a).
    pub city_shields: u8,
}

/// La piste de réputation : le bonus ou malus d'influence de chaque case, de −7 à +7 ; `None`
/// pour la case X (scan `misc/fame_board.jpg`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReputationTrack {
    bonuses: Vec<Option<i8>>,
}

impl ReputationTrack {
    /// Invariant : `bonuses` a été validé par `GameContent::parse` (quinze cases, X au bas).
    pub(crate) fn new(bonuses: Vec<Option<i8>>) -> Self {
        ReputationTrack { bonuses }
    }

    /// Les quinze cases, de −7 à +7.
    pub fn spaces(&self) -> &[Option<i8>] {
        &self.bonuses
    }

    /// Le bonus (ou malus, négatif) d'influence de cette réputation à l'interaction ; `None` sur X.
    pub fn bonus(&self, reputation: Reputation) -> Option<i8> {
        let index = usize::try_from(reputation.0 - Reputation::MIN).ok()?;
        self.bonuses.get(index).copied().flatten()
    }
}

/// Le bonus de cette réputation, d'après le contenu embarqué ; `None` sur X.
pub fn reputation_bonus(reputation: Reputation) -> Option<i8> {
    GameContent::embedded().reputation_track().bonus(reputation)
}
