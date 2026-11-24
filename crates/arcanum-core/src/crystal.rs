//! Cristaux et inventaire (spec 021).

use crate::{BasicColor, Feature, Hex, Map, Site};

/// Au plus trois cristaux de chaque couleur de base (MKUE p. 5, « Using mana » 1.b).
pub const CRYSTAL_LIMIT: u8 = 3;

/// Cristaux rangés sur la carte de héros, par couleur de base ; public.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Inventory {
    /// Dans l'ordre de `BasicColor::ALL`.
    crystals: [u8; 4],
}

impl Inventory {
    pub fn crystals(&self, color: BasicColor) -> u8 {
        self.crystals[color.index()]
    }

    /// Tous les cristaux, toutes couleurs confondues (décompte final, spec 063).
    pub fn total(&self) -> u32 {
        self.crystals.iter().map(|&count| u32::from(count)).sum()
    }

    pub fn is_full(&self, color: BasicColor) -> bool {
        self.crystals(color) >= CRYSTAL_LIMIT
    }

    /// Sans contrôle de la limite : c'est `decide` qui la garantit (spec 021, L3).
    pub(crate) fn add(&mut self, color: BasicColor) {
        let count = &mut self.crystals[color.index()];
        *count = count.saturating_add(1);
    }

    /// Rien s'il n'y en a pas : c'est `decide` qui vérifie le paiement (spec 022, L3).
    pub(crate) fn remove(&mut self, color: BasicColor) {
        let count = &mut self.crystals[color.index()];
        *count = count.saturating_sub(1);
    }
}

/// Couleur de la mine de cette case, s'il y en a une.
pub fn mine_color(map: &Map, hex: Hex) -> Option<BasicColor> {
    match map.space(hex)?.feature? {
        Feature::Site(Site::Mine(color)) => Some(color),
        _ => None,
    }
}
