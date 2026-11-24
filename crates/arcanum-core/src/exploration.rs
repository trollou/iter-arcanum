//! Exploration (spec 015) : où une tuile peut être révélée, et laquelle.

use std::collections::BTreeSet;

use crate::{Hex, Map, MapShape, TileDeck, TileDeckCounts, TileId, TileKind, TilePosition};
use crate::{base_game_tiles, board::counts_of};

/// Révéler une tuile coûte 2 points de mouvement (MKUE p. 7, 5.b).
pub const EXPLORE_COST: u8 = 2;

/// Ce qu'un joueur sait de la tuile qu'il révélerait (MKUE p. 7, 5.c et 5.e).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NextTile {
    /// Dos vert sur le dessus de la pioche.
    Countryside,
    /// Dos brun sur le dessus de la pioche.
    Core,
    /// Pioche vide : une tuile de campagne écartée, tirée au hasard.
    RemovedCountryside,
    /// Pioche vide, plus de tuile de campagne écartée : une tuile royaume sans cité écartée.
    RemovedCore,
    /// Plus rien à révéler.
    None,
}

/// Verdict sur un emplacement pour la prochaine tuile.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Placement {
    Allowed,
    Occupied,
    /// Hors de la forme du plateau de jeu (MKUE p. 7, 5.a ; p. 14).
    BehindCoastline,
    NothingToReveal,
    /// Contrainte du dos non respectée (MKUE p. 7, 5.e).
    TooFewNeighbors,
    /// Tuile royaume sur la côte de la forme en coin standard (MKUE p. 14, *Wedge Map* ; spec 064).
    CoreOnCoastline,
}

/// Tuiles du jeu de base, hors tuile de départ, ni posées ni dans la pioche, dans l'ordre des
/// identifiants : campagne, puis royaume.
pub fn removed_tiles(map: &Map, deck: &TileDeck) -> Vec<TileId> {
    let used: BTreeSet<TileId> = map
        .tiles()
        .map(|(_, tile)| tile)
        .chain(deck.tiles().iter().copied())
        .collect();
    let mut removed: Vec<TileId> = base_game_tiles()
        .iter()
        .map(|definition| definition.id)
        .filter(|id| id.kind() != TileKind::Starting && !used.contains(id))
        .collect();
    removed.sort();
    removed
}

/// Tuiles écartées qui peuvent encore être révélées : de campagne, puis royaume sans cité.
pub(crate) fn revealable_removed(map: &Map, deck: &TileDeck) -> Vec<TileId> {
    let removed = removed_tiles(map, deck);
    let countryside: Vec<TileId> = removed
        .iter()
        .copied()
        .filter(|tile| tile.kind() == TileKind::Countryside)
        .collect();
    if !countryside.is_empty() {
        return countryside;
    }
    removed
        .into_iter()
        .filter(|tile| tile.kind() == TileKind::Core && !tile.is_city())
        .collect()
}

/// Nombre de tuiles écartées encore révélables, par dos (public : se déduit du scénario).
pub(crate) fn removed_counts(map: &Map, deck: &TileDeck) -> TileDeckCounts {
    let usable: Vec<TileId> = removed_tiles(map, deck)
        .into_iter()
        .filter(|tile| !tile.is_city())
        .collect();
    counts_of(&usable)
}

pub fn next_tile(map: &Map, deck: &TileDeck) -> NextTile {
    if let Some(top) = deck.tiles().first() {
        return match top.kind() {
            TileKind::Core => NextTile::Core,
            _ => NextTile::Countryside,
        };
    }
    match revealable_removed(map, deck)
        .first()
        .map(|tile| tile.kind())
    {
        Some(TileKind::Countryside) => NextTile::RemovedCountryside,
        Some(_) => NextTile::RemovedCore,
        None => NextTile::None,
    }
}

/// Nombre de tuiles posées sur les emplacements voisins.
fn placed_neighbors(map: &Map, position: TilePosition) -> usize {
    position
        .neighbors()
        .into_iter()
        .filter(|neighbor| map.tile_at(*neighbor).is_some())
        .count()
}

/// Peut-on poser la prochaine tuile à cet emplacement (MKUE p. 7, 5.a et 5.e) ?
pub fn placement(map: &Map, shape: MapShape, position: TilePosition, next: NextTile) -> Placement {
    if map.tile_at(position).is_some() {
        return Placement::Occupied;
    }
    if !shape.allows(position) {
        return Placement::BehindCoastline;
    }
    if next == NextTile::None {
        return Placement::NothingToReveal;
    }
    // MKUE p. 14, *Wedge Map* : en coin standard, ni tuile royaume ni tuile royaume écartée sur
    // les deux files du bord (spec 064).
    let (i, j) = position.lattice();
    if shape == MapShape::StandardWedge
        && matches!(next, NextTile::Core | NextTile::RemovedCore)
        && (i == 0 || j == 0)
    {
        return Placement::CoreOnCoastline;
    }
    let neighbors = placed_neighbors(map, position);
    let enough = match next {
        NextTile::None => return Placement::NothingToReveal,
        // Q1 : la tuile adjacente doit toucher deux autres tuiles posées, sans compter la nouvelle.
        NextTile::Countryside => {
            neighbors >= 2
                || position.neighbors().into_iter().any(|neighbor| {
                    map.tile_at(neighbor).is_some() && placed_neighbors(map, neighbor) >= 2
                })
        }
        NextTile::Core => neighbors >= 2,
        // « Pour boucher les trous ».
        NextTile::RemovedCountryside | NextTile::RemovedCore => neighbors >= 3,
    };
    if enough {
        Placement::Allowed
    } else {
        Placement::TooFewNeighbors
    }
}

/// Emplacements libres dont une case touche `hex`, dans l'ordre des emplacements.
pub fn free_positions_touching(map: &Map, hex: Hex) -> Vec<TilePosition> {
    let positions: BTreeSet<TilePosition> = hex
        .neighbors()
        .into_iter()
        .filter(|neighbor| map.space(*neighbor).is_none())
        .map(TilePosition::containing)
        .filter(|position| map.tile_at(*position).is_none())
        .collect();
    positions.into_iter().collect()
}
