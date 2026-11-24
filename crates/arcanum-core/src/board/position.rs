//! Emplacements de tuile (spec 010).

use super::hex::{Direction, Hex};

/// Centres des six emplacements voisins de l'origine, imposés par les symboles des coins
/// des tuiles (MKUE p. 7, 5.a).
const NEIGHBOR_OFFSETS: [(i32, i32); 6] = [(3, -2), (1, -3), (-2, -1), (-3, 2), (-1, 3), (2, 1)];

/// Classe d'une case modulo le pavage : `3q + r` s'annule (mod 7) sur les deux vecteurs
/// du pavage (3, −2) et (1, −3), et prend une valeur distincte et non nulle pour chaque direction.
/// Les centres d'emplacement sont donc exactement les cases de classe 0.
const fn tiling_class(hex: Hex) -> i32 {
    (3 * hex.q + hex.r).rem_euclid(7)
}

/// Place fixe où une tuile peut être posée, repérée par sa case centrale.
/// Seules les cases du pavage des tuiles sont des centres valides : on l'obtient
/// par [`TilePosition::ORIGIN`], [`TilePosition::containing`] ou [`TilePosition::neighbors`].
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct TilePosition {
    center: Hex,
}

impl TilePosition {
    /// Emplacement de la tuile de départ.
    pub const ORIGIN: TilePosition = TilePosition {
        center: Hex::ORIGIN,
    };

    /// L'emplacement centré sur cette case, si c'est un centre du pavage.
    pub fn from_center(center: Hex) -> Option<TilePosition> {
        (tiling_class(center) == 0).then_some(TilePosition { center })
    }

    /// L'unique emplacement qui contient cette case.
    pub fn containing(hex: Hex) -> TilePosition {
        let class = tiling_class(hex);
        let center = Direction::ALL
            .into_iter()
            .find(|direction| tiling_class(Hex::ORIGIN.neighbor(*direction)) == class)
            .map_or(hex, |direction| hex.neighbor(direction.opposite()));
        TilePosition { center }
    }

    /// Emplacement de coordonnées d'emplacement `(i, j)` : sa case centrale vaut
    /// `i × (3, −2) + j × (1, −3)` (spec 012).
    pub const fn from_lattice(i: i32, j: i32) -> TilePosition {
        TilePosition {
            center: Hex::new(3 * i + j, -2 * i - 3 * j),
        }
    }

    /// Coordonnées d'emplacement `(i, j)` (spec 012). La division est exacte : `3q + r`
    /// est un multiple de 7 pour un centre du pavage.
    pub const fn lattice(self) -> (i32, i32) {
        let i = (3 * self.center.q + self.center.r) / 7;
        (i, self.center.q - 3 * i)
    }

    pub const fn center(self) -> Hex {
        self.center
    }

    /// La case de l'emplacement dans cette direction depuis le centre.
    pub const fn hex_at(self, direction: Direction) -> Hex {
        self.center.neighbor(direction)
    }

    /// Les sept cases : le centre, puis une par direction dans l'ordre de [`Direction::ALL`].
    pub fn hexes(self) -> [Hex; 7] {
        let sides = self.center.neighbors();
        [
            self.center,
            sides[0],
            sides[1],
            sides[2],
            sides[3],
            sides[4],
            sides[5],
        ]
    }

    /// Les six emplacements voisins.
    pub fn neighbors(self) -> [TilePosition; 6] {
        NEIGHBOR_OFFSETS.map(|(dq, dr)| TilePosition {
            center: Hex::new(self.center.q + dq, self.center.r + dr),
        })
    }
}
