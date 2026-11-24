//! Plateau de jeu : les tuiles posées et leurs emplacements (spec 010).

use std::collections::{BTreeMap, BTreeSet};

use thiserror::Error;

use super::hex::{Direction, Hex};
use super::position::TilePosition;
use super::tile::{SpaceContent, TileId, tile};

/// Refus de poser une tuile.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum MapError {
    #[error("l'emplacement {0:?} est déjà occupé")]
    PositionOccupied(TilePosition),
    #[error("la tuile {0:?} est déjà sur le plateau de jeu")]
    TileAlreadyPlaced(TileId),
    #[error("la tuile {0:?} n'existe pas dans le jeu de base")]
    UnknownTile(TileId),
}

/// Plateau de jeu : tuiles posées, chacune à un emplacement. Ne vérifie pas les
/// règles d'exploration (spec 015).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Map {
    tiles: BTreeMap<TilePosition, TileId>,
}

impl Map {
    pub fn new() -> Map {
        Map::default()
    }

    pub fn place(&mut self, id: TileId, position: TilePosition) -> Result<(), MapError> {
        if tile(id).is_none() {
            return Err(MapError::UnknownTile(id));
        }
        if self.tiles.contains_key(&position) {
            return Err(MapError::PositionOccupied(position));
        }
        if self
            .tiles
            .values()
            .any(|placed| placed.same_physical_tile(id))
        {
            return Err(MapError::TileAlreadyPlaced(id));
        }
        self.tiles.insert(position, id);
        Ok(())
    }

    /// Contenu de la case, ou rien si aucune tuile n'y est posée.
    pub fn space(&self, hex: Hex) -> Option<SpaceContent> {
        let position = TilePosition::containing(hex);
        let definition = tile(*self.tiles.get(&position)?)?;
        if hex == position.center() {
            return Some(definition.center);
        }
        Direction::ALL
            .into_iter()
            .find(|direction| position.hex_at(*direction) == hex)
            .map(|direction| definition.side(direction))
    }

    /// La tuile posée à cet emplacement, s'il y en a une.
    pub fn tile_at(&self, position: TilePosition) -> Option<TileId> {
        self.tiles.get(&position).copied()
    }

    /// Les tuiles posées, par emplacement.
    pub fn tiles(&self) -> impl Iterator<Item = (TilePosition, TileId)> + '_ {
        self.tiles.iter().map(|(position, tile)| (*position, *tile))
    }

    /// Emplacements libres voisins d'au moins une tuile posée, ordonnés par emplacement.
    pub fn free_neighbor_positions(&self) -> Vec<TilePosition> {
        let free: BTreeSet<TilePosition> = self
            .tiles
            .keys()
            .flat_map(|position| position.neighbors())
            .filter(|position| !self.tiles.contains_key(position))
            .collect();
        free.into_iter().collect()
    }
}
