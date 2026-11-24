//! Pioche de tuiles (spec 012) : cachée, seul son dos se voit.

use super::tile::{TileId, TileKind};

/// Pioche de tuiles, dans l'ordre : la première est le dessus.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TileDeck {
    tiles: Vec<TileId>,
}

/// Ce qu'on voit d'une pioche de tuiles : le nombre de tuiles de chaque dos (MKUE p. 3).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TileDeckCounts {
    pub countryside: usize,
    pub core: usize,
}

impl TileDeck {
    pub fn new(tiles: Vec<TileId>) -> TileDeck {
        TileDeck { tiles }
    }

    /// Les tuiles dans l'ordre : information cachée, jamais dans une vue.
    pub fn tiles(&self) -> &[TileId] {
        &self.tiles
    }

    pub fn len(&self) -> usize {
        self.tiles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tiles.is_empty()
    }

    pub fn count(&self, kind: TileKind) -> usize {
        self.tiles.iter().filter(|tile| tile.kind() == kind).count()
    }

    pub fn counts(&self) -> TileDeckCounts {
        counts_of(&self.tiles)
    }

    /// Retire la tuile de la pioche ; rien si elle n'y est pas.
    pub(crate) fn remove(&mut self, tile: TileId) {
        self.tiles.retain(|kept| *kept != tile);
    }
}

/// Nombre de tuiles de chaque dos dans une suite de tuiles.
pub(crate) fn counts_of(tiles: &[TileId]) -> TileDeckCounts {
    let count = |kind| tiles.iter().filter(|tile| tile.kind() == kind).count();
    TileDeckCounts {
        countryside: count(TileKind::Countryside),
        core: count(TileKind::Core),
    }
}
