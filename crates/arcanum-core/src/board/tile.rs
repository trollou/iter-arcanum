//! Tuiles, terrains et éléments imprimés (spec 010). Aucune règle attachée ici.

use serde::Deserialize;

use super::hex::Direction;
use crate::GameContent;

/// Identifiant d'une tuile. Les deux faces de la tuile de départ sont deux identifiants
/// d'une même tuile physique.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, serde::Serialize,
)]
pub enum TileId {
    StartingA,
    StartingB,
    /// Tuile de campagne (dos vert), numérotée à partir de 1.
    Countryside(u8),
    /// Tuile royaume (dos brun), numérotée à partir de 1.
    Core(u8),
}

/// Sorte de tuile, reconnue à son dos (MKUE p. 3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TileKind {
    Starting,
    Countryside,
    Core,
}

impl TileId {
    pub const fn kind(self) -> TileKind {
        match self {
            TileId::StartingA | TileId::StartingB => TileKind::Starting,
            TileId::Countryside(_) => TileKind::Countryside,
            TileId::Core(_) => TileKind::Core,
        }
    }

    /// Vrai pour une tuile royaume de cité (MKUE p. 3).
    pub fn is_city(self) -> bool {
        tile(self).is_some_and(TileDefinition::has_city)
    }

    /// Vrai si les deux identifiants désignent la même tuile physique (faces A et B).
    pub fn same_physical_tile(self, other: TileId) -> bool {
        self == other || (self.kind() == TileKind::Starting && other.kind() == TileKind::Starting)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum Terrain {
    Plains,
    Hills,
    Forest,
    Wasteland,
    Desert,
    Swamp,
    Lake,
    Mountain,
    /// Littoral : mer de la tuile de départ, toujours infranchissable.
    Sea,
}

impl Terrain {
    pub const ALL: [Terrain; 9] = [
        Terrain::Plains,
        Terrain::Hills,
        Terrain::Forest,
        Terrain::Wasteland,
        Terrain::Desert,
        Terrain::Swamp,
        Terrain::Lake,
        Terrain::Mountain,
        Terrain::Sea,
    ];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize, serde::Serialize)]
pub enum BasicColor {
    Red,
    Blue,
    White,
    Green,
}

impl BasicColor {
    pub const ALL: [BasicColor; 4] = [
        BasicColor::Red,
        BasicColor::Blue,
        BasicColor::White,
        BasicColor::Green,
    ];

    /// Position dans `ALL`.
    pub(crate) fn index(self) -> usize {
        match self {
            BasicColor::Red => 0,
            BasicColor::Blue => 1,
            BasicColor::White => 2,
            BasicColor::Green => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize, serde::Serialize)]
pub enum Site {
    Portal,
    Village,
    Monastery,
    MagicalGlade,
    Mine(BasicColor),
    Keep,
    MageTower,
    Dungeon,
    Tomb,
    MonsterDen,
    SpawningGrounds,
    Ruins,
    City(BasicColor),
}

impl Site {
    /// Site d'aventure : donjon, tombeau, antre, Sombre Terrier, ruines (MKUE p. 8, *Combat with
    /// Enemies* 1.b ; spec 044).
    pub fn is_adventure(self) -> bool {
        matches!(
            self,
            Site::Dungeon | Site::Tomb | Site::MonsterDen | Site::SpawningGrounds | Site::Ruins
        )
    }
}

/// Ennemi errant imprimé sur une case.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum Rampaging {
    OrcMarauders,
    Draconum,
}

/// Élément imprimé sur une case : un site ou un ennemi errant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum Feature {
    Site(Site),
    Rampaging(Rampaging),
}

/// Contenu d'une case : terrain et, au plus, un élément imprimé.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SpaceContent {
    pub terrain: Terrain,
    pub feature: Option<Feature>,
}

/// Contenu statique d'une tuile, dans l'orientation imposée (MKUE p. 7).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TileDefinition {
    pub id: TileId,
    pub center: SpaceContent,
    /// Une case par direction, dans l'ordre de [`Direction::ALL`].
    pub sides: [SpaceContent; 6],
}

impl TileDefinition {
    pub fn side(&self, direction: Direction) -> SpaceContent {
        // `Direction::ALL` est déclaré dans l'ordre des variantes : l'indice est la variante.
        self.sides[direction as usize]
    }

    /// Vrai si la case centrale porte une cité (MKUE p. 3).
    pub fn has_city(&self) -> bool {
        matches!(self.center.feature, Some(Feature::Site(Site::City(_))))
    }
}

/// Les 21 tuiles du jeu de base, lues dans `content/tiles.ron` (spec 029).
pub fn base_game_tiles() -> &'static [TileDefinition] {
    GameContent::embedded().tiles()
}

/// La définition d'une tuile du jeu de base, ou rien si l'identifiant n'existe pas.
pub fn tile(id: TileId) -> Option<&'static TileDefinition> {
    base_game_tiles()
        .iter()
        .find(|definition| definition.id == id)
}
