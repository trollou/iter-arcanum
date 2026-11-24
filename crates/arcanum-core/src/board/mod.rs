//! Plateau de jeu hexagonal (spec 010) : cases, emplacements de tuile, tuiles et leur contenu.

mod hex;
mod map;
mod position;
mod tile;
mod tile_deck;

pub use hex::{Direction, Hex};
pub use map::{Map, MapError};
pub use position::TilePosition;
pub use tile::{
    BasicColor, Feature, Rampaging, Site, SpaceContent, Terrain, TileDefinition, TileId, TileKind,
    base_game_tiles, tile,
};
pub(crate) use tile_deck::counts_of;
pub use tile_deck::{TileDeck, TileDeckCounts};
