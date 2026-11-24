//! Cités (spec 062) : l'état d'une cité révélée.

use crate::{BasicColor, PlayerId};

/// Une cité révélée : sa couleur, son niveau, ses Boucliers et son Seigneur (MKUE p. 7, 5.g ;
/// p. 10, *City Assaults* ; spec 062). Ses défenseurs restent dans les jetons d'ennemis du plateau.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CityState {
    pub color: BasicColor,
    pub level: u8,
    /// Les Boucliers posés sur la cité, dans l'ordre : le joueur de chacun (MKUE p. 10, 1.a).
    pub shields: Vec<crate::PlayerId>,
    /// Le Seigneur de la cité, une fois conquise (MKUE p. 10, *City Assaults* 2).
    pub leader: Option<PlayerId>,
}

impl CityState {
    /// Une cité qui entre en jeu, sans Bouclier.
    pub fn new(color: BasicColor, level: u8) -> CityState {
        CityState {
            color,
            level,
            shields: Vec::new(),
            leader: None,
        }
    }

    /// Vrai une fois conquise.
    pub fn is_conquered(&self) -> bool {
        self.leader.is_some()
    }

    /// Le Seigneur qu'auraient ces Boucliers : le joueur qui en a le plus, à égalité celui du
    /// premier posé (MKUE p. 10, *City Assaults* 2 ; spec 062).
    pub fn leader_of(shields: &[PlayerId]) -> Option<PlayerId> {
        let count = |player: PlayerId| shields.iter().filter(|&&owner| owner == player).count();
        let most = shields.iter().map(|&player| count(player)).max()?;
        shields
            .iter()
            .copied()
            .find(|&player| count(player) == most)
    }

    /// Le nombre de Boucliers de ce joueur sur la cité.
    pub fn shields_of(&self, player: PlayerId) -> usize {
        self.shields
            .iter()
            .filter(|&&owner| owner == player)
            .count()
    }
}
