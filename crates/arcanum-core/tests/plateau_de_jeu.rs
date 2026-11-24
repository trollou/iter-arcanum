//! Scénarios de la spec 010 (plateau de jeu hexagonal).

use std::collections::{BTreeSet, HashMap};

use arcanum_core::{
    BasicColor, Direction, Feature, Hex, Map, MapError, Rampaging, Site, SpaceContent, Terrain,
    TileDefinition, TileId, TileKind, TilePosition, base_game_tiles, tile,
};

fn hex(q: i32, r: i32) -> Hex {
    Hex::new(q, r)
}

/// Distance entre deux cases, pour parcourir un disque dans les tests (hors périmètre du moteur).
fn distance(a: Hex, b: Hex) -> i32 {
    let dq = a.q - b.q;
    let dr = a.r - b.r;
    (dq.abs() + dr.abs() + (dq + dr).abs()) / 2
}

fn disk(radius: i32) -> Vec<Hex> {
    let mut hexes = Vec::new();
    for q in -radius..=radius {
        for r in -radius..=radius {
            let h = hex(q, r);
            if distance(h, Hex::ORIGIN) <= radius {
                hexes.push(h);
            }
        }
    }
    hexes
}

fn position(q: i32, r: i32) -> TilePosition {
    TilePosition::from_center(hex(q, r)).expect("centre du pavage")
}

// --- Géométrie des cases ---

// C1 — MKUE p. 7 (on ne se déplace que vers une case adjacente).
#[test]
fn neighbor_then_opposite_returns_to_start() {
    for start in [Hex::ORIGIN, hex(5, -3), hex(-7, 2)] {
        let neighbors = start.neighbors();
        let distinct: BTreeSet<Hex> = neighbors.iter().copied().collect();
        assert_eq!(distinct.len(), 6);
        assert!(!distinct.contains(&start));
        for direction in Direction::ALL {
            let next = start.neighbor(direction);
            assert_eq!(next.neighbor(direction.opposite()), start);
        }
    }
}

#[test]
fn direction_offsets_match_the_spec() {
    let offsets: Vec<(i32, i32)> = Direction::ALL.iter().map(|d| d.offset()).collect();
    assert_eq!(
        offsets,
        vec![(1, 0), (1, -1), (0, -1), (-1, 0), (-1, 1), (0, 1)]
    );
}

// C2 — MKUE p. 7.
#[test]
fn adjacency_is_symmetric() {
    let hexes = disk(3);
    for &a in &hexes {
        assert!(!a.is_adjacent(a));
        for &b in &hexes {
            assert_eq!(a.is_adjacent(b), b.is_adjacent(a));
            assert_eq!(a.is_adjacent(b), a.neighbors().contains(&b));
        }
    }
}

// --- Emplacements ---

// C3 — MKUE p. 2-3 (tuiles de sept cases).
#[test]
fn tile_position_covers_center_and_its_six_neighbors() {
    for p in [TilePosition::ORIGIN, position(3, -2), position(-1, 3)] {
        let hexes = p.hexes();
        assert_eq!(hexes[0], p.center());
        for (i, direction) in Direction::ALL.iter().enumerate() {
            assert_eq!(p.hex_at(*direction), p.center().neighbor(*direction));
            assert_eq!(hexes[i + 1], p.hex_at(*direction));
        }
    }
}

// C4 — MKUE p. 7, 5.a (emplacements fixes définis par les symboles des coins).
#[test]
fn neighbor_positions_match_corner_symbols() {
    let centers: Vec<Hex> = TilePosition::ORIGIN
        .neighbors()
        .iter()
        .map(|p| p.center())
        .collect();
    let expected: BTreeSet<Hex> = [(3, -2), (1, -3), (-2, -1), (-3, 2), (-1, 3), (2, 1)]
        .iter()
        .map(|&(q, r)| hex(q, r))
        .collect();
    assert_eq!(centers.iter().copied().collect::<BTreeSet<_>>(), expected);

    let own: BTreeSet<Hex> = TilePosition::ORIGIN.hexes().into_iter().collect();
    for neighbor in TilePosition::ORIGIN.neighbors() {
        assert!(neighbor.hexes().iter().all(|h| !own.contains(h)));
    }
}

#[test]
fn only_tiling_centers_are_positions() {
    assert!(TilePosition::from_center(hex(1, 0)).is_none());
    assert!(TilePosition::from_center(hex(3, -2)).is_some());
}

// C5 — MKUE p. 7, 5.a (pavage sans chevauchement ni trou).
#[test]
fn every_hex_belongs_to_exactly_one_position() {
    let mut owners: HashMap<Hex, Vec<TilePosition>> = HashMap::new();
    let mut positions = BTreeSet::new();
    for h in disk(16) {
        positions.insert(TilePosition::containing(h));
    }
    for p in &positions {
        for h in p.hexes() {
            owners.entry(h).or_default().push(*p);
        }
    }
    for h in disk(12) {
        let found = owners.get(&h).cloned().unwrap_or_default();
        assert_eq!(found.len(), 1, "case {h:?}");
        assert_eq!(TilePosition::containing(h), found[0]);
    }
}

// C6 — MKUE p. 7.
#[test]
fn tile_north_east_touches_by_its_west_and_south_west() {
    let start = TilePosition::ORIGIN;
    let other = position(3, -2);
    let start_ne = start.hex_at(Direction::NE);
    let start_e = start.hex_at(Direction::E);
    assert!(other.hex_at(Direction::W).is_adjacent(start_ne));
    assert!(other.hex_at(Direction::SW).is_adjacent(start_ne));
    assert!(other.hex_at(Direction::SW).is_adjacent(start_e));
}

// --- Contenu ---

// C7 — MKUE p. 3.
#[test]
fn base_game_defines_twenty_one_tiles() {
    let tiles = base_game_tiles();
    assert_eq!(tiles.len(), 21);
    let ids: BTreeSet<TileId> = tiles.iter().map(|t| t.id).collect();
    let mut expected = BTreeSet::from([TileId::StartingA, TileId::StartingB]);
    expected.extend((1..=11).map(TileId::Countryside));
    expected.extend((1..=8).map(TileId::Core));
    assert_eq!(ids, expected);
    for t in tiles {
        assert_eq!(tile(t.id), Some(t));
    }
    assert_eq!(tile(TileId::Countryside(12)), None);
    assert_eq!(tile(TileId::Core(0)), None);
}

#[test]
fn tile_kind_comes_from_its_back() {
    assert_eq!(TileId::StartingA.kind(), TileKind::Starting);
    assert_eq!(TileId::StartingB.kind(), TileKind::Starting);
    assert_eq!(TileId::Countryside(4).kind(), TileKind::Countryside);
    assert_eq!(TileId::Core(4).kind(), TileKind::Core);
}

// C8 — MKUE p. 3 (« City icon in the middle »).
#[test]
fn core_tiles_five_to_eight_have_a_city() {
    for n in 1..=8 {
        let has_city = tile(TileId::Core(n)).is_some_and(TileDefinition::has_city);
        assert_eq!(has_city, n >= 5, "tuile royaume {n}");
    }
    for t in base_game_tiles() {
        if t.id.kind() != TileKind::Core {
            assert!(!t.has_city(), "{:?}", t.id);
        }
    }
}

// C9 — le contenu des tuiles n'est plus recopié dans `docs/reference/tuiles.md` : sa seule source est
// `crates/arcanum-core/content/tiles.ron` (spec 029, Q4), validé par `tests/contenu_en_donnees.rs`.

// C10 — décomptes du tableau « Décomptes » de `tuiles.md`.
#[test]
fn site_counts_match_transcription() {
    let mut counts: HashMap<Feature, usize> = HashMap::new();
    for t in base_game_tiles() {
        let spaces = std::iter::once(t.center).chain(t.sides);
        for feature in spaces.filter_map(|s| s.feature) {
            *counts.entry(feature).or_default() += 1;
        }
    }
    let count = |f: Feature| counts.get(&f).copied().unwrap_or(0);
    let site = |s: Site| count(Feature::Site(s));
    assert_eq!(site(Site::Portal), 2);
    assert_eq!(site(Site::Village), 6);
    assert_eq!(site(Site::Monastery), 4);
    assert_eq!(site(Site::MagicalGlade), 6);
    assert_eq!(site(Site::Keep), 5);
    assert_eq!(site(Site::MageTower), 5);
    assert_eq!(site(Site::Dungeon), 2);
    assert_eq!(site(Site::MonsterDen), 2);
    assert_eq!(site(Site::Ruins), 7);
    assert_eq!(site(Site::Tomb), 2);
    assert_eq!(site(Site::SpawningGrounds), 2);
    for color in [
        BasicColor::Red,
        BasicColor::Blue,
        BasicColor::White,
        BasicColor::Green,
    ] {
        assert_eq!(site(Site::Mine(color)), 2, "mines {color:?}");
        assert_eq!(site(Site::City(color)), 1, "cité {color:?}");
    }
    assert_eq!(count(Feature::Rampaging(Rampaging::OrcMarauders)), 10);
    assert_eq!(count(Feature::Rampaging(Rampaging::Draconum)), 6);
}

// --- Plateau de jeu ---

// C11 — MKUE p. 3.
#[test]
fn placed_starting_tile_reveals_its_spaces() {
    let mut map = Map::new();
    assert_eq!(map.place(TileId::StartingA, TilePosition::ORIGIN), Ok(()));
    assert_eq!(
        map.space(Hex::ORIGIN),
        Some(SpaceContent {
            terrain: Terrain::Plains,
            feature: Some(Feature::Site(Site::Portal)),
        })
    );
    assert_eq!(
        map.space(hex(1, -1)).map(|s| s.terrain),
        Some(Terrain::Forest)
    );
    assert_eq!(
        map.tiles().collect::<Vec<_>>(),
        vec![(TilePosition::ORIGIN, TileId::StartingA)]
    );
}

// C12 — MKUE p. 7, 5.d (orientation imposée).
#[test]
fn tiles_keep_the_starting_tile_orientation() {
    let mut map = Map::new();
    assert_eq!(map.place(TileId::Countryside(1), position(3, -2)), Ok(()));
    assert_eq!(
        map.space(hex(4, -3)).map(|s| s.terrain),
        Some(Terrain::Lake)
    );
}

// C13.
#[test]
fn space_without_tile_is_empty() {
    let mut map = Map::new();
    assert_eq!(map.space(Hex::ORIGIN), None);
    assert_eq!(map.place(TileId::StartingA, TilePosition::ORIGIN), Ok(()));
    assert_eq!(map.space(hex(3, -2)), None);
}

// C14 — MKUE p. 7, 5.a.
#[test]
fn free_neighbor_positions_grow_with_the_map() {
    let mut map = Map::new();
    assert!(map.free_neighbor_positions().is_empty());
    assert_eq!(map.place(TileId::StartingA, TilePosition::ORIGIN), Ok(()));
    let first: BTreeSet<_> = map.free_neighbor_positions().into_iter().collect();
    let expected: BTreeSet<_> = TilePosition::ORIGIN.neighbors().into_iter().collect();
    assert_eq!(first, expected);

    assert_eq!(map.place(TileId::Countryside(1), position(3, -2)), Ok(()));
    let free = map.free_neighbor_positions();
    assert_eq!(free.len(), 8);
    assert!(!free.contains(&TilePosition::ORIGIN));
    assert!(!free.contains(&position(3, -2)));
    let mut sorted = free.clone();
    sorted.sort();
    assert_eq!(free, sorted, "ordre déterministe");
}

// --- Cas limites ---

// L1.
#[test]
fn place_on_occupied_position_is_refused() {
    let mut map = Map::new();
    assert_eq!(map.place(TileId::StartingA, TilePosition::ORIGIN), Ok(()));
    let before = map.clone();
    assert_eq!(
        map.place(TileId::Countryside(2), TilePosition::ORIGIN),
        Err(MapError::PositionOccupied(TilePosition::ORIGIN))
    );
    assert_eq!(map, before);
}

// L2.
#[test]
fn same_tile_cannot_be_placed_twice() {
    let mut map = Map::new();
    assert_eq!(map.place(TileId::StartingA, TilePosition::ORIGIN), Ok(()));
    assert_eq!(map.place(TileId::Countryside(3), position(3, -2)), Ok(()));
    let before = map.clone();
    assert_eq!(
        map.place(TileId::Countryside(3), position(1, -3)),
        Err(MapError::TileAlreadyPlaced(TileId::Countryside(3)))
    );
    assert_eq!(
        map.place(TileId::StartingB, position(1, -3)),
        Err(MapError::TileAlreadyPlaced(TileId::StartingB))
    );
    assert_eq!(
        map.place(TileId::Core(9), position(1, -3)),
        Err(MapError::UnknownTile(TileId::Core(9)))
    );
    assert_eq!(map, before);
}

// L3.
#[test]
fn far_positions_work() {
    let mut far = TilePosition::ORIGIN;
    for _ in 0..50 {
        far = far.neighbors()[0];
    }
    let mut map = Map::new();
    assert_eq!(map.place(TileId::Core(5), far), Ok(()));
    assert_eq!(
        map.space(far.center()).and_then(|s| s.feature),
        Some(Feature::Site(Site::City(BasicColor::Green)))
    );
    assert_eq!(TilePosition::containing(far.hex_at(Direction::SW)), far);
}
