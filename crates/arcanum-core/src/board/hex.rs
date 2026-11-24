//! Coordonnées des cases et directions (spec 010).

/// Une des six directions d'une case, cases à pointe en haut.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Direction {
    E,
    NE,
    NW,
    W,
    SW,
    SE,
}

impl Direction {
    /// Les six directions, dans l'ordre de la spec 010.
    pub const ALL: [Direction; 6] = [
        Direction::E,
        Direction::NE,
        Direction::NW,
        Direction::W,
        Direction::SW,
        Direction::SE,
    ];

    /// Décalage axial `(q, r)` d'un pas dans cette direction.
    pub const fn offset(self) -> (i32, i32) {
        match self {
            Direction::E => (1, 0),
            Direction::NE => (1, -1),
            Direction::NW => (0, -1),
            Direction::W => (-1, 0),
            Direction::SW => (-1, 1),
            Direction::SE => (0, 1),
        }
    }

    pub const fn opposite(self) -> Direction {
        match self {
            Direction::E => Direction::W,
            Direction::NE => Direction::SW,
            Direction::NW => Direction::SE,
            Direction::W => Direction::E,
            Direction::SW => Direction::NE,
            Direction::SE => Direction::NW,
        }
    }
}

/// Coordonnées axiales d'une case : `q` croît vers l'est, `r` vers le sud-est.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct Hex {
    pub q: i32,
    pub r: i32,
}

impl Hex {
    pub const ORIGIN: Hex = Hex { q: 0, r: 0 };

    pub const fn new(q: i32, r: i32) -> Hex {
        Hex { q, r }
    }

    pub const fn neighbor(self, direction: Direction) -> Hex {
        let (dq, dr) = direction.offset();
        Hex::new(self.q + dq, self.r + dr)
    }

    /// Les six voisines, dans l'ordre de [`Direction::ALL`].
    pub fn neighbors(self) -> [Hex; 6] {
        Direction::ALL.map(|direction| self.neighbor(direction))
    }

    pub fn is_adjacent(self, other: Hex) -> bool {
        self.neighbors().contains(&other)
    }
}
