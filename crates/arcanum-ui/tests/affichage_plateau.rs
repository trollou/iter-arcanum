//! Scénarios de la spec 011 (affichage du plateau de jeu), sans fenêtre.
//! Aucune règle nouvelle du livret : chaque test cite le cas de la spec.

use std::collections::BTreeSet;
use std::fs;

use arcanum_core::{
    BasicColor, Columns, Direction, Enemy, Feature, Hero, Hex, Map, MapShape, Site, Terrain,
    TileId, TilePosition,
};
use arcanum_i18n::{Catalog, Language};
use arcanum_ui::{
    ARROW_SCROLL_SPEED, BOARD_ACTION_KINDS, BOARD_BACKGROUND, BoardCamera, BoardInput, BoardScreen,
    DEFAULT_ZOOM, FOG_IMAGE, ImageId, MAX_ZOOM, MIN_ZOOM, OverlayKind, ZOOM_STEP,
    board_action_image, board_images, board_screen, feature_image, figure_image, hero_figure_image,
    hex_center, hex_corners, point_to_hex, relief_image, repository_assets_dir, svg_size,
    terrain_color, terrain_image,
};
use eframe::egui::{Color32, Pos2, Rect, Vec2, pos2, vec2};

mod support;

use support::{demo_board, map_input};

/// Le plateau de démonstration n'a pas de forme : la plus large, que la zone interdite
/// ne change aucun décompte de la spec 011 (elle est une couche à part).
const SHAPE: MapShape = MapShape::Open(Columns::Five);

const EPS: f32 = 1e-4;
const SQRT_3: f32 = 1.732_050_8;

const TERRAINS: [Terrain; 9] = [
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

fn french() -> Catalog {
    Catalog::load(
        &repository_assets_dir().join("i18n"),
        &Language("fr".into()),
    )
    .expect("fr.json du dépôt")
}

fn close(a: Pos2, b: Pos2) -> bool {
    (a - b).length() < EPS * a.to_vec2().length().max(1.0)
}

fn area() -> Rect {
    Rect::from_min_size(pos2(0.0, 0.0), vec2(1000.0, 700.0))
}

fn hexes_within(radius: i32) -> Vec<Hex> {
    let mut hexes = vec![];
    for q in -radius..=radius {
        for r in -radius..=radius {
            if (q + r).abs() <= radius {
                hexes.push(Hex::new(q, r));
            }
        }
    }
    hexes
}

/// Écran du plateau de démonstration, tout entier visible, sans curseur.
fn demo_screen(cursor: Option<Pos2>) -> (BoardScreen, BoardCamera) {
    let demo = demo_board();
    let camera = BoardCamera::initial(&demo.map, area());
    let screen = board_screen(
        &BoardInput {
            figure: Some(demo.figure),
            hero: Some(Hero::Arythea),
            shape: SHAPE,
            ..map_input(&demo.map, camera, area(), cursor)
        },
        &french(),
    );
    (screen, camera)
}

fn hex_on_screen(camera: &BoardCamera, hex: Hex) -> Pos2 {
    camera.to_screen(area(), hex_center(hex))
}

// --- Géométrie ---

// Spec 011, C1 (orientation, MKUE p. 7).
#[test]
fn hex_centers_follow_pointy_top_layout() {
    assert!(close(hex_center(Hex::ORIGIN), pos2(0.0, 0.0)));
    assert!(close(
        hex_center(Hex::ORIGIN.neighbor(Direction::E)),
        pos2(SQRT_3, 0.0)
    ));
    assert!(close(
        hex_center(Hex::ORIGIN.neighbor(Direction::SE)),
        pos2(SQRT_3 / 2.0, 1.5)
    ));
}

// Spec 011, C2.
#[test]
fn hex_corners_are_shared_with_neighbors() {
    let corners = hex_corners(Hex::ORIGIN);
    assert!(close(corners[0], pos2(0.0, -1.0)));
    for corner in corners {
        assert!((corner.to_vec2().length() - 1.0).abs() < EPS);
    }
    for neighbor in Hex::ORIGIN.neighbors() {
        let shared = hex_corners(neighbor)
            .iter()
            .filter(|theirs| corners.iter().any(|ours| close(*ours, **theirs)))
            .count();
        assert_eq!(shared, 2, "voisine {neighbor:?}");
    }
}

// Spec 011, C3.
#[test]
fn point_to_hex_inverts_hex_center() {
    for hex in hexes_within(10) {
        let center = hex_center(hex);
        assert_eq!(point_to_hex(center), hex);
        for corner in hex_corners(hex) {
            let inside = center + (corner - center) * 0.8;
            assert_eq!(point_to_hex(inside), hex, "case {hex:?}, point {inside:?}");
        }
    }
}

// --- Cadrage ---

fn some_camera() -> BoardCamera {
    BoardCamera {
        center: pos2(3.5, -2.25),
        zoom: 37.0,
    }
}

// Spec 011, C4.
#[test]
fn camera_round_trip_is_identity() {
    let camera = some_camera();
    for point in [pos2(0.0, 0.0), pos2(-12.5, 7.25), pos2(100.0, -40.0)] {
        let back = camera.to_board(area(), camera.to_screen(area(), point));
        assert!(close(back, point), "{point:?} → {back:?}");
    }
}

// Spec 011, C5.
#[test]
fn wheel_zoom_keeps_point_under_cursor() {
    let mut camera = some_camera();
    let cursor = pos2(730.0, 120.0);
    let before = camera.to_board(area(), cursor);
    camera.zoom_at(area(), cursor, 1.0);
    assert!((camera.zoom - 37.0 * ZOOM_STEP).abs() < EPS);
    assert!(close(camera.to_board(area(), cursor), before));
    assert!((ZOOM_STEP - 1.15).abs() < EPS);
}

// Spec 011, C6.
#[test]
fn key_zoom_keeps_area_center() {
    let mut camera = some_camera();
    let middle = area().center();
    let before = camera.to_board(area(), middle);
    camera.zoom_at(area(), middle, 1.0);
    assert!((camera.zoom - 37.0 * 1.15).abs() < EPS);
    assert!(close(camera.to_board(area(), middle), before));
    camera.zoom_at(area(), middle, -1.0);
    assert!((camera.zoom - 37.0).abs() < EPS);
    assert!(close(camera.to_board(area(), middle), before));
}

// Spec 011, C8.
#[test]
fn drag_moves_board_with_cursor() {
    let mut camera = some_camera();
    let point = pos2(5.0, 1.0);
    let before = camera.to_screen(area(), point);
    camera.drag(vec2(37.0, -12.0));
    assert!(close(
        camera.to_screen(area(), point),
        before + vec2(37.0, -12.0)
    ));
}

// Spec 011, C9.
#[test]
fn arrow_keys_scroll_the_board() {
    assert_eq!(ARROW_SCROLL_SPEED, 800.0);
    let point = pos2(5.0, 1.0);

    let mut camera = some_camera();
    let before = camera.to_screen(area(), point);
    camera.scroll(vec2(1.0, 0.0), 0.05);
    assert!(close(
        camera.to_screen(area(), point),
        before + vec2(-40.0, 0.0)
    ));

    let mut camera = some_camera();
    let before = camera.to_screen(area(), point);
    camera.scroll(vec2(-1.0, -1.0), 0.1);
    assert!(close(
        camera.to_screen(area(), point),
        before + vec2(80.0, 80.0)
    ));
}

// Spec 011, C11.
#[test]
fn recenter_restores_initial_camera() {
    let demo = demo_board();
    let initial = BoardCamera::initial(&demo.map, area());
    let mut camera = initial;
    camera.zoom_at(area(), pos2(10.0, 10.0), 3.0);
    camera.drag(vec2(-250.0, 40.0));
    camera.scroll(vec2(0.0, 1.0), 0.05);
    camera.recenter(&demo.map, area());
    assert_eq!(camera, initial);
}

// --- Plateau de démonstration ---

// Spec 011, C12.
#[test]
fn demo_map_has_twenty_tiles_in_a_fixed_layout() {
    let demo = demo_board();
    assert_eq!(demo.figure, Hex::ORIGIN);
    assert_eq!(demo_board().map, demo.map);

    let tiles: Vec<(TilePosition, TileId)> = demo.map.tiles().collect();
    assert_eq!(tiles.len(), 20);
    assert!(tiles.contains(&(TilePosition::ORIGIN, TileId::StartingA)));
    assert!(!tiles.iter().any(|(_, id)| *id == TileId::StartingB));

    // Même plateau que l'ordre décrit par la spec : l'emplacement libre le plus proche de (0, 0).
    let mut expected = Map::new();
    expected
        .place(TileId::StartingA, TilePosition::ORIGIN)
        .expect("départ");
    let order = (1..=11)
        .map(TileId::Countryside)
        .chain((1..=8).map(TileId::Core));
    let distance = |hex: Hex| (hex.q.abs() + hex.r.abs() + (hex.q + hex.r).abs()) / 2;
    for id in order {
        let free = expected.free_neighbor_positions();
        let nearest = free
            .iter()
            .map(|position| distance(position.center()))
            .min()
            .expect("emplacement libre");
        let position = *free
            .iter()
            .find(|position| distance(position.center()) == nearest)
            .expect("le plus proche");
        expected.place(id, position).expect("emplacement libre");
    }
    assert_eq!(demo.map, expected);
}

// --- Écran du plateau ---

// Spec 011, C13.
#[test]
fn board_screen_draws_each_placed_hex_with_its_terrain() {
    let demo = demo_board();
    let (screen, _) = demo_screen(None);
    assert_eq!(screen.hexes.len(), 140);
    let drawn: BTreeSet<Hex> = screen.hexes.iter().map(|drawing| drawing.hex).collect();
    assert_eq!(drawn.len(), 140);
    for drawing in &screen.hexes {
        let content = demo.map.space(drawing.hex).expect("case posée");
        assert_eq!(drawing.image, terrain_image(content.terrain));
        assert_eq!(drawing.fallback, terrain_color(content.terrain));
    }
    for free in demo.map.free_neighbor_positions() {
        for hex in free.hexes() {
            assert!(!drawn.contains(&hex));
        }
    }
}

// Spec 011, C14.
#[test]
fn hexes_outside_the_area_are_culled() {
    let mut map = Map::new();
    map.place(TileId::StartingA, TilePosition::ORIGIN)
        .expect("départ");
    // Zoom 40 centré sur la case E : la case O est à 2 × √3 × 40 ≈ 139 points à gauche.
    let camera = BoardCamera {
        center: hex_center(Hex::new(1, 0)),
        zoom: 40.0,
    };
    let narrow = Rect::from_center_size(pos2(500.0, 300.0), vec2(200.0, 400.0));
    let screen = board_screen(
        &BoardInput {
            shape: SHAPE,
            ..map_input(&map, camera, narrow, None)
        },
        &french(),
    );
    let drawn: BTreeSet<Hex> = screen.hexes.iter().map(|drawing| drawing.hex).collect();
    // Case O : entièrement à gauche de la zone (son bord droit est à 500 − 139 + 35 < 400).
    assert!(!drawn.contains(&Hex::new(-1, 0)));
    // Centre : son bord gauche est hors de la zone, son bord droit dedans.
    assert!(drawn.contains(&Hex::ORIGIN));
    assert!(drawn.contains(&Hex::new(1, 0)));
}

// Spec 011, C15.
#[test]
fn board_screen_outlines_each_visible_tile() {
    let (screen, _) = demo_screen(None);
    assert_eq!(screen.tile_outlines.len(), 20);
    let tiles: BTreeSet<TileId> = screen.tile_outlines.iter().map(|o| o.tile).collect();
    assert_eq!(tiles.len(), 20);
    for outline in &screen.tile_outlines {
        // Contour extérieur de sept cases : 18 arêtes.
        assert_eq!(outline.segments.len(), 18, "{:?}", outline.tile);
    }
}

/// Case d'une tuile du plateau de démonstration, par direction (ou centre si `None`).
fn demo_hex(id: TileId, direction: Option<Direction>) -> Hex {
    let (position, _) = demo_board()
        .map
        .tiles()
        .find(|(_, tile)| *tile == id)
        .expect("tuile posée");
    direction.map_or(position.center(), |direction| position.hex_at(direction))
}

// Spec 011, C16.
#[test]
fn printed_sites_and_enemies_get_an_overlay() {
    let (screen, _) = demo_screen(None);
    let overlay_of = |hex: Hex| -> Vec<ImageId> {
        screen
            .overlays
            .iter()
            .filter(|overlay| {
                overlay.hex == hex
                    && !matches!(overlay.kind, OverlayKind::Figure | OverlayKind::Relief)
            })
            .map(|overlay| overlay.image)
            .collect()
    };
    let mine = demo_hex(TileId::Countryside(2), Some(Direction::SW));
    assert_eq!(overlay_of(mine), vec![ImageId::new("site", "mine_green")]);
    assert_eq!(
        feature_image(Feature::Site(Site::Mine(BasicColor::Green))),
        ImageId::new("site", "mine_green")
    );
    // Spec 031, Q5 : en partie, un jeton couvre toujours le symbole des orques (spec 030) ; sans
    // jeton, c'est un ennemi vaincu, et la case se dessine comme son terrain seul.
    let orcs = demo_hex(TileId::Countryside(2), Some(Direction::NW));
    assert_eq!(overlay_of(orcs), vec![]);
    let bare = demo_hex(TileId::Countryside(2), None);
    assert_eq!(overlay_of(bare), vec![]);
}

// Spec 011, C18, adapté par la spec 075 (C14) : la figurine debout devant le site, décalée à
// droite, n'en cache pas le centre.
#[test]
fn hero_figure_does_not_hide_the_site() {
    let (screen, camera) = demo_screen(None);
    let figures: Vec<_> = screen
        .overlays
        .iter()
        .filter(|overlay| overlay.kind == OverlayKind::Figure)
        .collect();
    assert_eq!(figures.len(), 1);
    assert_eq!(figures[0].hex, Hex::ORIGIN);
    let foot = camera.to_board(area(), figures[0].rect.center_bottom());
    assert_eq!(point_to_hex(foot), Hex::ORIGIN);
    let portal: Vec<_> = screen
        .overlays
        .iter()
        .filter(|overlay| overlay.hex == Hex::ORIGIN && overlay.kind == OverlayKind::Site)
        .collect();
    assert_eq!(portal.len(), 1);
    assert_eq!(portal[0].image, ImageId::new("site", "portal"));
    assert!(!figures[0].rect.contains(portal[0].rect.center()));
}

// Spec 011, C19.
#[test]
fn terrain_fallback_colors_are_distinct() {
    let colors: BTreeSet<[u8; 4]> = TERRAINS
        .iter()
        .map(|terrain| terrain_color(*terrain).to_array())
        .collect();
    assert_eq!(colors.len(), 9);
    assert!(!colors.contains(&BOARD_BACKGROUND.to_array()));
    assert_eq!(BOARD_BACKGROUND, Color32::from_rgb(0x20, 0x21, 0x24));
}

// Spec 011, C20.
#[test]
fn svg_size_is_picked_from_a_small_series() {
    assert_eq!(svg_size(20.0), 32);
    assert_eq!(svg_size(64.0), 64);
    assert_eq!(svg_size(65.0), 128);
    assert_eq!(svg_size(300.0), 256);
}

// --- Images ---

// Spec 011, C21 ; spec 075, C20.
#[test]
fn every_board_image_exists() {
    let images_dir = repository_assets_dir().join("images");
    let images = board_images();
    // Spec 012 : 32 avec l'image de la zone interdite ; spec 065 : 35, une par héros au lieu de
    // la figurine unique ; spec 041 : 39, un jeton Bouclier par héros (un drapeau depuis la spec
    // 043) ; spec 043 : 45, un dos de jeton d'ennemi par couleur ; spec 075 : 44, la mer
    // (image de terrain) remplace les hachures de la zone interdite ; puis 76 : 9 terrains,
    // 19 sites, 4 portraits, 4 drapeaux, 4 figurines de héros, 29 d'ennemis, le brouillard et
    // 6 décors de relief, sans les jetons ronds ; puis 82 : 6 icônes des boutons d'action du
    // plateau (fil C).
    assert_eq!(images.len(), 82);
    // Toutes les figurines d'ennemis sont dessinées depuis le fil D5 de la spec 075.
    for image in &images {
        let path = image.path(&images_dir);
        let text = fs::read_to_string(&path).unwrap_or_else(|_| panic!("{path:?} manque"));
        assert!(text.starts_with("<svg"), "{path:?} n'est pas un SVG");
    }
    for terrain in TERRAINS {
        assert!(images.contains(&terrain_image(terrain)));
    }
    let standing = Enemy::ALL
        .map(figure_image)
        .into_iter()
        .chain(Hero::ALL.map(hero_figure_image))
        .chain([FOG_IMAGE])
        .chain(
            [Terrain::Forest, Terrain::Hills, Terrain::Mountain]
                .into_iter()
                .flat_map(|terrain| [false, true].map(|clear| relief_image(terrain, clear)))
                .flatten(),
        );
    for image in standing.chain(BOARD_ACTION_KINDS.map(board_action_image)) {
        assert!(images.contains(&image), "{image:?}");
    }
}

// --- Survol ---

// Spec 011, C23.
#[test]
fn hovering_portal_lists_its_content() {
    let demo = demo_board();
    let camera = BoardCamera::initial(&demo.map, area());
    let cursor = hex_on_screen(&camera, Hex::ORIGIN);
    let (screen, _) = demo_screen(Some(cursor));
    assert_eq!(screen.hovered.map(|hovered| hovered.hex), Some(Hex::ORIGIN));
    assert_eq!(
        screen.hover_lines,
        vec![
            "Tuile de départ (face A)",
            "Plaine",
            "Portail",
            "Votre héros"
        ]
    );
}

// Spec 011, C25.
#[test]
fn hovering_a_mine_names_its_color() {
    let demo = demo_board();
    let camera = BoardCamera::initial(&demo.map, area());
    let hex = demo_hex(TileId::Core(8), Some(Direction::NE));
    let (screen, _) = demo_screen(Some(hex_on_screen(&camera, hex)));
    assert_eq!(
        screen.hover_lines,
        vec!["Tuile royaume 8", "Colline", "Mine rouge"]
    );
}

// Spec 037, C3 (remplace la spec 011, C26) : rien à décrire, donc pas d'infobulle.
#[test]
fn hovering_nothing_shows_no_lines() {
    let demo = demo_board();
    let camera = BoardCamera::initial(&demo.map, area());
    // Spec 012 : un emplacement libre hors forme annonce la zone interdite ; on prend donc
    // un emplacement libre que la forme permet.
    let free = demo
        .map
        .free_neighbor_positions()
        .into_iter()
        .find(|position| SHAPE.allows(*position))
        .expect("un emplacement libre permis")
        .center();
    let cursors = [
        Some(hex_on_screen(&camera, free)),
        Some(pos2(-50.0, 300.0)),
        None,
    ];
    for cursor in cursors {
        let (screen, _) = demo_screen(cursor);
        assert_eq!(screen.hovered, None, "{cursor:?}");
        assert!(screen.hover_lines.is_empty(), "{cursor:?}");
    }
}

// --- Libellés ---

// Spec 011, C27.
#[test]
fn every_board_key_exists_in_french() {
    let catalog = french();
    let demo = demo_board();
    let camera = BoardCamera::initial(&demo.map, area());
    let hexes: Vec<Hex> = demo
        .map
        .tiles()
        .flat_map(|(position, _)| position.hexes())
        .collect();
    for hex in hexes {
        board_screen(
            &BoardInput {
                figure: Some(demo.figure),
                hero: Some(Hero::Arythea),
                shape: SHAPE,
                ..map_input(&demo.map, camera, area(), Some(hex_on_screen(&camera, hex)))
            },
            &catalog,
        );
    }
    // Aucun survol, et le bouton du panneau latéral.
    board_screen(
        &BoardInput {
            figure: Some(demo.figure),
            hero: Some(Hero::Arythea),
            shape: SHAPE,
            ..map_input(&demo.map, camera, area(), None)
        },
        &catalog,
    );
    catalog.text("board.recenter");
    // Les tuiles absentes du plateau de démonstration ont aussi un nom.
    let mut other = Map::new();
    other
        .place(TileId::StartingB, TilePosition::ORIGIN)
        .expect("départ B");
    let camera = BoardCamera::initial(&other, area());
    board_screen(
        &BoardInput {
            shape: SHAPE,
            ..map_input(
                &other,
                camera,
                area(),
                Some(hex_on_screen(&camera, Hex::ORIGIN)),
            )
        },
        &catalog,
    );
    assert_eq!(catalog.gaps(), vec![]);
}

// --- Cas limites ---

// Spec 011, L1.
#[test]
fn empty_area_draws_nothing() {
    let demo = demo_board();
    let empty = Rect::from_min_size(pos2(10.0, 10.0), Vec2::ZERO);
    let camera = BoardCamera::initial(&demo.map, empty);
    assert_eq!(camera, BoardCamera::initial(&Map::new(), empty));
    let screen = board_screen(
        &BoardInput {
            figure: Some(demo.figure),
            hero: Some(Hero::Arythea),
            shape: SHAPE,
            ..map_input(&demo.map, camera, empty, Some(pos2(10.0, 10.0)))
        },
        &french(),
    );
    assert!(screen.hexes.is_empty());
    assert!(screen.overlays.is_empty());
    assert!(screen.tile_outlines.is_empty());
    assert_eq!(screen.hovered, None);
}

// Spec 011, L2.
#[test]
fn empty_map_gives_default_camera() {
    assert_eq!(DEFAULT_ZOOM, 40.0);
    let camera = BoardCamera::initial(&Map::new(), area());
    assert_eq!(
        camera,
        BoardCamera {
            center: pos2(0.0, 0.0),
            zoom: 40.0
        }
    );
    let screen = board_screen(
        &BoardInput {
            shape: SHAPE,
            ..map_input(&Map::new(), camera, area(), Some(area().center()))
        },
        &french(),
    );
    assert!(screen.hexes.is_empty());
    assert_eq!(screen.hovered, None);
}

// Spec 011, L3.
#[test]
fn hover_on_edge_is_deterministic() {
    // Milieu de l'arête commune à (0, 0) et à sa voisine E.
    let corners = hex_corners(Hex::ORIGIN);
    let edge = corners[1] + (corners[2] - corners[1]) * 0.5;
    let first = point_to_hex(edge);
    assert!(first == Hex::ORIGIN || first == Hex::new(1, 0));
    for _ in 0..10 {
        assert_eq!(point_to_hex(edge), first);
    }
}

// Spec 011, L4.
#[test]
fn many_wheel_steps_stay_finite() {
    let mut camera = some_camera();
    let cursor = pos2(100.0, 100.0);
    camera.zoom_at(area(), cursor, 1.0e6);
    assert_eq!(camera.zoom, MAX_ZOOM);
    camera.zoom_at(area(), cursor, -1.0e6);
    assert_eq!(camera.zoom, MIN_ZOOM);
    camera.zoom_at(area(), cursor, f32::INFINITY);
    camera.zoom_at(area(), cursor, f32::NAN);
    assert!(camera.zoom.is_finite() && camera.zoom >= MIN_ZOOM && camera.zoom <= MAX_ZOOM);
    assert!(camera.center.x.is_finite() && camera.center.y.is_finite());
}

// Spec 011, L6.
#[test]
fn long_frame_scroll_is_capped() {
    let point = pos2(5.0, 1.0);
    let mut camera = some_camera();
    let before = camera.to_screen(area(), point);
    camera.scroll(vec2(1.0, 0.0), 10.0);
    assert!(close(
        camera.to_screen(area(), point),
        before + vec2(-80.0, 0.0)
    ));
}
