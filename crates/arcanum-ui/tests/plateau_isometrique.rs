//! Scénarios de la spec 075 (plateau isométrique), sans fenêtre.
//! Aucune règle nouvelle du livret : chaque test cite le cas de la spec.

use std::collections::BTreeSet;

use arcanum_core::{
    Direction, Enemy, EnemyColor, GameConfig, GameState, Hero, Hex, Map, MapShape, PlayerView,
    Scenario, Terrain, TileId, TilePosition, legal_actions, view,
};
use arcanum_i18n::{Catalog, Language};
use arcanum_ui::{
    BOARD_TILT, BoardCamera, BoardInput, BoardScreen, CameraBounds, CombatChoice, FIT_ZOOM_OUT,
    FOG_IMAGE, GameSession, ImageId, MAX_ZOOM, MIN_ZOOM, Overlay, OverlayKind, SLAB_DEPTH,
    STANDING_HEIGHT, board_images, board_screen, combat_screen, enemy_color, figure_image,
    hero_color, hero_figure_image, hex_center, hex_corners, point_to_hex, relief_image,
    repository_assets_dir, terrain_image,
};
use eframe::egui::{Color32, Pos2, Rect, pos2, vec2};

// L'arène des tests d'arcanum-core, partagée plutôt que recopiée.
#[path = "../../arcanum-core/tests/arene/mod.rs"]
mod arene;
use arene::{
    Adventure, DUNGEON, Fortified, KEEP, MAGE_TOWER, ORCS_2, SOLO, SPAWNING_GROUNDS, Units,
    VILLAGE, challenge, done, gargoyle, guardsmen, minotaur, prowlers,
};

mod support;

use support::{board_input, demo_board, first_reconnaissance_config, map_input};

fn french() -> Catalog {
    Catalog::load(
        &repository_assets_dir().join("i18n"),
        &Language("fr".into()),
    )
    .expect("fr.json du dépôt")
}

/// Vue joueur du bac à sable après la mise en place (`docs/reference/bac-a-sable.md`).
fn sandbox_view() -> PlayerView {
    let config = GameConfig {
        scenario: Scenario::sandbox(),
        ..first_reconnaissance_config(3)
    };
    GameSession::new(config).view()
}

/// Cadrage assez large pour voir la tuile de départ, ses emplacements voisins et la mer.
fn wide_camera() -> BoardCamera {
    BoardCamera {
        center: Pos2::ZERO,
        zoom: 20.0,
    }
}

fn sandbox_screen(view: &PlayerView) -> BoardScreen {
    board_screen(
        &BoardInput {
            inventory: None,
            movement: None,
            ..board_input(view, &[], wide_camera(), area(), None)
        },
        &french(),
    )
}

/// Écran du plateau de démonstration, tout entier visible, avec la figurine d'Arythea en (0, 0).
fn demo_screen() -> (BoardScreen, BoardCamera) {
    let demo = demo_board();
    let camera = BoardCamera::initial(&demo.map, area());
    let screen = board_screen(
        &BoardInput {
            figure: Some(demo.figure),
            hero: Some(Hero::Arythea),
            shape: MapShape::Open(arcanum_core::Columns::Five),
            ..map_input(&demo.map, camera, area(), None)
        },
        &french(),
    );
    (screen, camera)
}

const EPS: f32 = 1e-3;

fn area() -> Rect {
    Rect::from_min_size(pos2(0.0, 0.0), vec2(1000.0, 700.0))
}

fn some_camera() -> BoardCamera {
    BoardCamera {
        center: pos2(3.5, -2.25),
        zoom: 37.0,
    }
}

fn close(a: Pos2, b: Pos2) -> bool {
    (a - b).length() < EPS * a.to_vec2().length().max(1.0)
}

/// Cases à moins de `radius` de l'origine.
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

// --- Projection et caméra ---

// Spec 075, C1 (Q1 : inclinaison 0,80).
#[test]
fn camera_squashes_the_vertical_axis() {
    assert_eq!(BOARD_TILT, 0.80);
    let camera = some_camera();
    let a = pos2(-1.5, 2.0);
    let b = a + vec2(1.0, 1.0);
    let delta = camera.to_screen(area(), b) - camera.to_screen(area(), a);
    assert!((delta.x - camera.zoom).abs() < EPS, "{delta:?}");
    assert!(
        (delta.y - camera.zoom * BOARD_TILT).abs() < EPS,
        "{delta:?}"
    );
}

// Spec 075, C2 : le sol est plat, la case sous le curseur se calcule exactement.
#[test]
fn picking_inverts_the_projection() {
    let camera = some_camera();
    for point in [pos2(0.0, 0.0), pos2(-12.5, 7.25), pos2(100.0, -40.0)] {
        let back = camera.to_board(area(), camera.to_screen(area(), point));
        assert!(close(back, point), "{point:?} → {back:?}");
    }
    for hex in hexes_within(3) {
        let on_screen = camera.to_screen(area(), hex_center(hex));
        assert_eq!(point_to_hex(camera.to_board(area(), on_screen)), hex);
    }
}

fn placed_hexes(map: &Map) -> Vec<Hex> {
    map.tiles()
        .flat_map(|(position, _)| position.hexes())
        .collect()
}

// Spec 075, C4 : cadrage sur le plateau projeté, la hauteur d'un élément debout laissée en haut.
#[test]
fn initial_camera_fits_the_projected_map() {
    let demo = demo_board();
    let camera = BoardCamera::initial(&demo.map, area());
    let inner = area().shrink2(area().size() * 0.05);
    let mut on_screen: Vec<Pos2> = placed_hexes(&demo.map)
        .into_iter()
        .flat_map(hex_corners)
        .map(|corner| camera.to_screen(area(), corner))
        .collect();
    // Le haut d'un élément debout posé au centre de chaque case.
    on_screen.extend(placed_hexes(&demo.map).into_iter().map(|hex| {
        camera.to_screen(area(), hex_center(hex)) - vec2(0.0, STANDING_HEIGHT * camera.zoom)
    }));
    let tolerance = 1e-2;
    for point in &on_screen {
        assert!(
            inner.expand(tolerance).contains(*point),
            "{point:?} hors de {inner:?}"
        );
    }
    let bounds = Rect::from_points(&on_screen);
    let touches = (bounds.left() - inner.left()).abs() < tolerance
        || (bounds.right() - inner.right()).abs() < tolerance
        || (bounds.top() - inner.top()).abs() < tolerance
        || (bounds.bottom() - inner.bottom()).abs() < tolerance;
    assert!(touches, "{bounds:?} ne touche pas {inner:?}");
}

// Spec 075, C5 (Q2).
#[test]
fn zoom_out_stops_near_the_fitted_board() {
    assert_eq!((MIN_ZOOM, MAX_ZOOM, FIT_ZOOM_OUT), (16.0, 120.0, 0.5));
    let demo = demo_board();
    let fitted = BoardCamera::initial(&demo.map, area());
    let bounds = CameraBounds::of(&demo.map, area());
    let cursor = pos2(420.0, 330.0);
    let mut camera = fitted;
    let before = camera.to_board(area(), cursor);
    camera.zoom_within(area(), cursor, -80.0, &bounds);
    assert!(
        (camera.zoom - FIT_ZOOM_OUT * fitted.zoom).abs() < EPS,
        "{} au lieu de {}",
        camera.zoom,
        FIT_ZOOM_OUT * fitted.zoom
    );
    assert!(close(camera.to_board(area(), cursor), before));
    camera.zoom_within(area(), cursor, 40.0, &bounds);
    assert_eq!(camera.zoom, MAX_ZOOM);

    // Zone si petite que 0,5 × le cadrage passe sous 16 : plancher absolu.
    let small = Rect::from_min_size(pos2(0.0, 0.0), vec2(200.0, 150.0));
    let fitted = BoardCamera::initial(&demo.map, small);
    let bounds = CameraBounds::of(&demo.map, small);
    assert_eq!(bounds.min_zoom, MIN_ZOOM);
    let mut camera = fitted;
    camera.zoom_within(small, small.center(), -80.0, &bounds);
    assert_eq!(camera.zoom, MIN_ZOOM);
}

// Spec 075, C6.
#[test]
fn zoom_floor_follows_the_map() {
    let mut small = Map::new();
    small
        .place(TileId::StartingA, TilePosition::ORIGIN)
        .expect("plateau vide");
    let large = demo_board().map;
    let floor_small = CameraBounds::of(&small, area()).min_zoom;
    let large_bounds = CameraBounds::of(&large, area());
    assert!(
        large_bounds.min_zoom < floor_small,
        "{} puis {}",
        floor_small,
        large_bounds.min_zoom
    );
    // Le zoom courant ne change pas quand la borne baisse.
    let mut camera = BoardCamera::initial(&small, area());
    let zoom = camera.zoom;
    camera.zoom_within(area(), area().center(), 0.0, &large_bounds);
    camera.keep_within(&large_bounds);
    assert_eq!(camera.zoom, zoom);
}

// Spec 075, C7.
#[test]
fn camera_center_stays_over_the_map() {
    let demo = demo_board();
    let corners: Vec<Pos2> = placed_hexes(&demo.map)
        .into_iter()
        .flat_map(hex_corners)
        .collect();
    let map_rect = Rect::from_points(&corners);
    let bounds = CameraBounds::of(&demo.map, area());
    assert_eq!(bounds.center, Some(map_rect));

    // Le plateau glissé loin vers la droite et le haut : le centre recule vers la gauche et le bas.
    let mut camera = BoardCamera::initial(&demo.map, area());
    camera.drag(vec2(1.0e5, -1.0e5));
    camera.keep_within(&bounds);
    assert!(
        close(camera.center, map_rect.left_bottom()),
        "{:?}",
        camera.center
    );

    // Défilement aux flèches loin vers la droite et le haut.
    let mut camera = BoardCamera::initial(&demo.map, area());
    for _ in 0..1000 {
        camera.scroll(vec2(1.0, -1.0), 0.1);
        camera.keep_within(&bounds);
    }
    assert!(
        close(camera.center, map_rect.right_top()),
        "{:?}",
        camera.center
    );
}

// Spec 075, L5 : plateau vide, pas de borne de défilement.
#[test]
fn empty_map_has_no_scroll_bound() {
    let bounds = CameraBounds::of(&Map::new(), area());
    assert_eq!(bounds.center, None);
    let mut camera = BoardCamera::default();
    camera.drag(vec2(1.0e4, 0.0));
    let moved = camera;
    camera.keep_within(&bounds);
    assert_eq!(camera, moved);
}

// --- Sol, mer, tranche ---

// Spec 075, C8.
#[test]
fn ground_hexes_are_squashed() {
    let (screen, camera) = demo_screen();
    assert!(!screen.hexes.is_empty());
    for drawing in &screen.hexes {
        let expected = hex_corners(drawing.hex).map(|corner| camera.to_screen(area(), corner));
        for (corner, wanted) in drawing.corners.iter().zip(expected) {
            assert!(close(*corner, wanted), "{:?}", drawing.hex);
        }
        assert_eq!(drawing.rect, Rect::from_points(&drawing.corners));
        assert!((drawing.rect.height() - 2.0 * camera.zoom * BOARD_TILT).abs() < EPS);
    }
}

// Spec 075, C9 (MKUE p. 14, Map Shape : au-delà de la côte, aucune tuile).
#[test]
fn sea_covers_the_forbidden_zone() {
    let screen = sandbox_screen(&sandbox_view());
    assert!(!screen.forbidden.is_empty());
    for drawing in &screen.forbidden {
        assert_eq!(
            drawing.image,
            terrain_image(Terrain::Sea),
            "{:?}",
            drawing.hex
        );
        let [red, green, blue, _] = drawing.fallback.to_array();
        assert!(blue > red && blue > green, "{:?}", drawing.fallback);
    }
}

// Spec 075, C10 : un emplacement libre permis reste sombre (fond du plateau).
#[test]
fn free_positions_stay_dark() {
    let view = sandbox_view();
    let screen = sandbox_screen(&view);
    // Emplacement 2A, centre (6, -4) : libre et permis par la forme en coin.
    let free = TilePosition::from_lattice(2, 0);
    assert!(
        view.map
            .free_neighbor_positions()
            .contains(&TilePosition::from_lattice(1, 1))
    );
    let drawn: BTreeSet<Hex> = screen
        .hexes
        .iter()
        .chain(&screen.forbidden)
        .map(|drawing| drawing.hex)
        .collect();
    for hex in free
        .hexes()
        .into_iter()
        .chain(TilePosition::from_lattice(1, 1).hexes())
    {
        assert!(!drawn.contains(&hex), "{hex:?}");
    }
}

// Spec 075, C11.
#[test]
fn slab_sides_on_open_edges() {
    let view = sandbox_view();
    let screen = sandbox_screen(&view);
    let sides: BTreeSet<(Hex, Direction)> = screen
        .slabs
        .iter()
        .map(|slab| (slab.hex, slab.side))
        .collect();
    for drawing in &screen.hexes {
        for side in [Direction::SE, Direction::SW] {
            let open = view.map.space(drawing.hex.neighbor(side)).is_none();
            assert_eq!(
                sides.contains(&(drawing.hex, side)),
                open,
                "{:?} {side:?}",
                drawing.hex
            );
        }
    }
    // Le portail (0, 0), entouré de tuiles : aucune tranche.
    assert!(!sides.iter().any(|(hex, _)| *hex == Hex::ORIGIN));
    // Une case du bord bordée par la mer au-delà de la côte en porte une.
    let sea: BTreeSet<Hex> = screen.forbidden.iter().map(|drawing| drawing.hex).collect();
    assert!(
        sides
            .iter()
            .any(|(hex, side)| sea.contains(&hex.neighbor(*side))),
        "{sides:?}"
    );
    // La tranche pend sous l'arête, sans écrasement.
    let camera = wide_camera();
    for slab in &screen.slabs {
        let corners = hex_corners(slab.hex).map(|corner| camera.to_screen(area(), corner));
        let edge = match slab.side {
            Direction::SE => [corners[2], corners[3]],
            _ => [corners[3], corners[4]],
        };
        let depth = vec2(0.0, SLAB_DEPTH * camera.zoom);
        assert_eq!(
            slab.corners,
            [edge[0], edge[1], edge[1] + depth, edge[0] + depth]
        );
    }
}

// --- Éléments debout ---

/// Cadrage du plateau des décors de l'arène, assez large pour la forteresse et la tour de mage.
fn arena_camera() -> BoardCamera {
    BoardCamera {
        center: hex_center(Hex::new(3, -4)),
        zoom: 40.0,
    }
}

fn arena_screen(state: &GameState, cursor: Option<Pos2>) -> BoardScreen {
    let seen = view(state, SOLO);
    let legal = legal_actions(state, SOLO);
    board_screen(
        &board_input(&seen, &legal, arena_camera(), area(), cursor),
        &french(),
    )
}

/// Le pied d'un élément debout : le point posé sur sa case.
fn foot(overlay: &Overlay) -> Pos2 {
    overlay.foot
}

fn kinds_on(screen: &BoardScreen, hex: Hex) -> Vec<OverlayKind> {
    screen
        .overlays
        .iter()
        .filter(|overlay| overlay.hex == hex)
        .map(|overlay| overlay.kind)
        .collect()
}

/// Les écrans qui ont le plus d'éléments debout : le plateau de démonstration, les sites fortifiés
/// (défenseurs face cachée), la forteresse conquise avec la figurine (drapeau), le village.
fn screens_with_standing_elements() -> Vec<(BoardScreen, BoardCamera)> {
    let own_keep = Fortified {
        figure: KEEP,
        keep: None,
        conquered: vec![KEEP],
        ..Fortified::default()
    };
    vec![
        demo_screen(),
        (
            arena_screen(&Fortified::default().state(), None),
            arena_camera(),
        ),
        (arena_screen(&own_keep.state(), None), arena_camera()),
        (
            arena_screen(&Units::default().state(), None),
            arena_camera(),
        ),
    ]
}

// Spec 075, C13.
#[test]
fn standing_elements_are_sorted_back_to_front() {
    for (screen, _) in screens_with_standing_elements() {
        assert!(screen.overlays.len() > 1);
        for pair in screen.overlays.windows(2) {
            let (back, front) = (foot(&pair[0]), foot(&pair[1]));
            assert!(
                front.y > back.y + EPS || ((front.y - back.y).abs() <= EPS && front.x >= back.x),
                "{:?} en {:?} avant {:?} en {:?}",
                pair[0].kind,
                pair[0].hex,
                pair[1].kind,
                pair[1].hex
            );
        }
    }
}

// Spec 075, C14 : site, drapeau, défenseur, puis la figurine du héros devant.
#[test]
fn hero_stands_in_front_of_the_site() {
    let screen = arena_screen(&Units::default().state(), None);
    assert_eq!(
        kinds_on(&screen, VILLAGE),
        vec![OverlayKind::Site, OverlayKind::Figure]
    );
    let on_village = |kind: OverlayKind| {
        screen
            .overlays
            .iter()
            .find(|overlay| overlay.hex == VILLAGE && overlay.kind == kind)
            .expect("élément debout")
            .clone()
    };
    let (site, hero) = (
        on_village(OverlayKind::Site),
        on_village(OverlayKind::Figure),
    );
    assert!(
        !hero.rect.contains(site.rect.center()),
        "{:?} cache {:?}",
        hero.rect,
        site.rect
    );
    assert!(foot(&hero).x > foot(&site).x && foot(&hero).y > foot(&site).y);

    let own_keep = Fortified {
        figure: KEEP,
        keep: None,
        conquered: vec![KEEP],
        ..Fortified::default()
    };
    let conquered = arena_screen(&own_keep.state(), None);
    assert_eq!(
        kinds_on(&conquered, KEEP),
        vec![
            OverlayKind::Relief,
            OverlayKind::Site,
            OverlayKind::Flag,
            OverlayKind::Figure
        ]
    );
    let guarded = arena_screen(&Fortified::default().state(), None);
    assert_eq!(
        kinds_on(&guarded, KEEP),
        vec![
            OverlayKind::Relief,
            OverlayKind::Site,
            OverlayKind::Defender
        ]
    );
}

// Spec 075, C15.
#[test]
fn standing_elements_stand_on_their_hex() {
    for (screen, camera) in screens_with_standing_elements() {
        for overlay in &screen.overlays {
            let ground = camera.to_board(area(), foot(overlay));
            assert_eq!(point_to_hex(ground), overlay.hex, "{:?}", overlay.kind);
            if matches!(
                overlay.kind,
                OverlayKind::Figure
                    | OverlayKind::Rampaging
                    | OverlayKind::Defender
                    | OverlayKind::SiteEnemy
            ) {
                assert!(
                    overlay.rect.height() > overlay.rect.width(),
                    "{:?} en {:?} : {:?}",
                    overlay.kind,
                    overlay.hex,
                    overlay.rect
                );
            }
        }
    }
}

// Spec 075, C23 et L2 : le haut de la tour de mage ne capte pas le curseur.
#[test]
fn standing_elements_do_not_catch_the_cursor() {
    let state = Fortified::default().state();
    let screen = arena_screen(&state, None);
    let tower = screen
        .overlays
        .iter()
        .find(|overlay| overlay.hex == MAGE_TOWER && overlay.kind == OverlayKind::Site)
        .expect("tour de mage debout");
    let camera = arena_camera();
    let cursor = tower.rect.center_top() + vec2(0.2, 0.02) * camera.zoom;
    assert!(tower.rect.contains(cursor));
    let north = point_to_hex(camera.to_board(area(), cursor));
    assert_eq!(north, MAGE_TOWER.neighbor(Direction::NE), "{cursor:?}");
    let hovered = arena_screen(&state, Some(cursor))
        .hovered
        .expect("case survolée");
    assert_eq!(hovered.hex, north);
}

// --- Figurines (fil B) ---

/// Les éléments debout d'une case.
fn overlays_on(screen: &BoardScreen, hex: Hex) -> Vec<Overlay> {
    screen
        .overlays
        .iter()
        .filter(|overlay| overlay.hex == hex)
        .cloned()
        .collect()
}

// Spec 075, C16 : la figurine debout du héros, pas son portrait.
#[test]
fn hero_figure_uses_its_standing_image() {
    for (hero, stem) in [
        (Hero::Arythea, "arythea"),
        (Hero::Goldyx, "goldyx"),
        (Hero::Norowas, "norowas"),
        (Hero::Tovak, "tovak"),
    ] {
        let demo = demo_board();
        let camera = BoardCamera::initial(&demo.map, area());
        let screen = board_screen(
            &BoardInput {
                figure: Some(demo.figure),
                hero: Some(hero),
                ..map_input(&demo.map, camera, area(), None)
            },
            &french(),
        );
        let figure = screen
            .overlays
            .iter()
            .find(|overlay| overlay.kind == OverlayKind::Figure)
            .expect("figurine du héros");
        assert_eq!(figure.image, ImageId::new("figure", stem));
        assert_eq!(figure.image, hero_figure_image(hero));
        assert_eq!(figure.fallback, hero_color(hero));
    }
}

// Spec 075, C17 : les Rôdeurs errants, en figurine ; infobulle inchangée (spec 037).
#[test]
fn rampaging_enemy_is_a_standing_figure() {
    let state = Units {
        placed: vec![(prowlers(), ORCS_2)],
        ..Units::default()
    }
    .state();
    let screen = arena_screen(&state, None);
    // La colline des Rôdeurs a aussi son décor de relief (C12).
    let on_orcs: Vec<Overlay> = overlays_on(&screen, ORCS_2)
        .into_iter()
        .filter(|overlay| overlay.kind != OverlayKind::Relief)
        .collect();
    assert_eq!(on_orcs.len(), 1, "{on_orcs:?}");
    assert_eq!(on_orcs[0].kind, OverlayKind::Rampaging);
    assert_eq!(on_orcs[0].image, ImageId::new("figure", "prowlers"));
    assert_eq!(on_orcs[0].image, figure_image(Enemy::Prowlers));

    let cursor = arena_camera().to_screen(area(), hex_center(ORCS_2));
    let hovered = arena_screen(&state, Some(cursor));
    assert!(
        hovered
            .hover_lines
            .iter()
            .any(|line| line.contains("Rôdeurs")),
        "{:?}",
        hovered.hover_lines
    );
}

// Spec #14, C9 : les Rôdeurs errants (pile verte) reposent sur un socle à la couleur de leur catégorie.
#[test]
fn rampaging_enemy_has_a_category_base() {
    let state = Units {
        placed: vec![(prowlers(), ORCS_2)],
        ..Units::default()
    }
    .state();
    let screen = arena_screen(&state, None);
    let rampaging = overlays_on(&screen, ORCS_2)
        .into_iter()
        .find(|overlay| overlay.kind == OverlayKind::Rampaging)
        .expect("Rôdeurs debout");
    assert_eq!(rampaging.base, Some(enemy_color(EnemyColor::Green)));
}

/// Écran des éléments debout autour d'une case, la caméra centrée sur elle.
fn screen_around(state: &GameState, hex: Hex) -> BoardScreen {
    let seen = view(state, SOLO);
    let camera = BoardCamera {
        center: hex_center(hex),
        zoom: 40.0,
    };
    board_screen(&board_input(&seen, &[], camera, area(), None), &french())
}

// Spec 075, C18 (issue privée 165) : face cachée, un brouillard teinté à la couleur de la pile, sans
// rien qui dise l'ennemi ; retourné, sa figurine ; au Sombre Terrier, deux figurines réduites.
#[test]
fn site_enemies_stand_or_hide_in_fog() {
    let hidden = screen_around(&Fortified::default().state(), KEEP);
    let defender = overlays_on(&hidden, KEEP)
        .into_iter()
        .find(|overlay| overlay.kind == OverlayKind::Defender)
        .expect("défenseur face cachée");
    assert_eq!(defender.image, ImageId::new("figure", "fog"));
    assert_eq!(defender.image, FOG_IMAGE);
    assert_eq!(defender.tint, enemy_color(EnemyColor::Grey));
    assert_eq!(defender.fallback, enemy_color(EnemyColor::Grey));
    let images: Vec<ImageId> = overlays_on(&hidden, KEEP)
        .iter()
        .map(|overlay| overlay.image)
        .collect();
    assert_eq!(
        images,
        [
            ImageId::new("relief", "hills_clear"),
            ImageId::new("site", "keep"),
            FOG_IMAGE
        ]
    );

    let revealed = Fortified {
        keep: Some((guardsmen(), true)),
        ..Fortified::default()
    };
    let screen = screen_around(&revealed.state(), KEEP);
    let defender = overlays_on(&screen, KEEP)
        .into_iter()
        .find(|overlay| overlay.kind == OverlayKind::Defender)
        .expect("défenseur retourné");
    assert_eq!(defender.image, figure_image(Enemy::Guardsmen));
    assert_eq!(defender.tint, Color32::WHITE);

    let alone = Adventure {
        figure: SPAWNING_GROUNDS,
        placed: vec![(gargoyle(), SPAWNING_GROUNDS)],
        ..Adventure::default()
    };
    let pair = Adventure {
        placed: vec![
            (gargoyle(), SPAWNING_GROUNDS),
            (minotaur(), SPAWNING_GROUNDS),
        ],
        ..alone.clone()
    };
    let site_enemies = |adventure: &Adventure| -> Vec<Overlay> {
        overlays_on(
            &screen_around(&adventure.state(), SPAWNING_GROUNDS),
            SPAWNING_GROUNDS,
        )
        .into_iter()
        .filter(|overlay| overlay.kind == OverlayKind::SiteEnemy)
        .collect()
    };
    let (alone, pair) = (site_enemies(&alone), site_enemies(&pair));
    assert_eq!(alone.len(), 1);
    assert_eq!(
        [pair[0].image, pair[1].image],
        [figure_image(Enemy::Gargoyle), figure_image(Enemy::Minotaur)]
    );
    assert!(foot(&pair[0]).x < foot(&pair[1]).x);
    assert!(pair[0].rect.height() < alone[0].rect.height());
}

// Spec #14, C10 : au Sombre Terrier, chaque monstre resté a son socle marron.
#[test]
fn site_enemies_have_a_category_base() {
    let pair = Adventure {
        figure: SPAWNING_GROUNDS,
        placed: vec![
            (gargoyle(), SPAWNING_GROUNDS),
            (minotaur(), SPAWNING_GROUNDS),
        ],
        ..Adventure::default()
    };
    let site_enemies: Vec<Overlay> = overlays_on(
        &screen_around(&pair.state(), SPAWNING_GROUNDS),
        SPAWNING_GROUNDS,
    )
    .into_iter()
    .filter(|overlay| overlay.kind == OverlayKind::SiteEnemy)
    .collect();
    assert_eq!(site_enemies.len(), 2);
    for enemy in &site_enemies {
        assert_eq!(enemy.base, Some(enemy_color(EnemyColor::Brown)));
    }
}

// Spec #14, C11 : le défenseur de la forteresse (pile grise) a son socle gris, face cachée sous le
// brouillard comme retourné.
#[test]
fn defenders_have_a_category_base() {
    let defender_base = |fortified: &Fortified| {
        overlays_on(&screen_around(&fortified.state(), KEEP), KEEP)
            .into_iter()
            .find(|overlay| overlay.kind == OverlayKind::Defender)
            .expect("défenseur")
            .base
    };
    let grey = Some(enemy_color(EnemyColor::Grey));
    assert_eq!(defender_base(&Fortified::default()), grey);
    let revealed = Fortified {
        keep: Some((guardsmen(), true)),
        ..Fortified::default()
    };
    assert_eq!(defender_base(&revealed), grey);
}

// Spec #14, C12 : héros, sites, drapeaux et relief n'ont pas de socle d'ennemi.
#[test]
fn only_enemies_have_a_category_base() {
    let enemy_kinds = [
        OverlayKind::Rampaging,
        OverlayKind::SiteEnemy,
        OverlayKind::Defender,
    ];
    let fortified = screen_around(&Fortified::default().state(), KEEP);
    let state = Units {
        placed: vec![(prowlers(), ORCS_2)],
        ..Units::default()
    }
    .state();
    let units = arena_screen(&state, None);
    let others: Vec<&Overlay> = fortified
        .overlays
        .iter()
        .chain(&units.overlays)
        .filter(|overlay| !enemy_kinds.contains(&overlay.kind))
        .collect();
    assert!(
        others.iter().any(|overlay| overlay.kind == OverlayKind::Figure),
        "le héros est à l'écran"
    );
    assert!(others.iter().any(|overlay| overlay.kind == OverlayKind::Site));
    for overlay in others {
        assert_eq!(overlay.base, None, "{overlay:?}");
    }
}

// Spec 075, C19 : le drapeau d'un site conquis, debout à côté du site (à gauche depuis l'issue privée 292).
#[test]
fn flag_stands_beside_the_site() {
    let own_keep = Fortified {
        figure: KEEP,
        keep: None,
        conquered: vec![KEEP],
        ..Fortified::default()
    };
    let on_keep = overlays_on(&screen_around(&own_keep.state(), KEEP), KEEP);
    let of = |kind: OverlayKind| {
        on_keep
            .iter()
            .find(|overlay| overlay.kind == kind)
            .expect("élément debout")
            .clone()
    };
    let (site, flag) = (of(OverlayKind::Site), of(OverlayKind::Flag));
    assert_eq!(flag.image, ImageId::new("flag", "arythea"));
    // Issue privée 292 : à gauche du site, et non plus à droite, où la figurine le cachait.
    assert!(foot(&flag).x < foot(&site).x);
}

// Issue privée 292 : le héros resté sur le site qu'il vient de conquérir ne cache pas son drapeau ;
// aucun élément dessiné après le drapeau sur sa case ne le recouvre.
#[test]
fn flag_stays_visible_under_the_hero() {
    let keep = Fortified {
        figure: KEEP,
        keep: None,
        conquered: vec![KEEP],
        ..Fortified::default()
    }
    .state();
    let dungeon = Adventure {
        figure: DUNGEON,
        conquered: vec![DUNGEON],
        ..Adventure::default()
    }
    .state();
    for (state, hex) in [(keep, KEEP), (dungeon, DUNGEON)] {
        let on_site = overlays_on(&screen_around(&state, hex), hex);
        let at = on_site
            .iter()
            .position(|overlay| overlay.kind == OverlayKind::Flag)
            .expect("drapeau");
        assert!(
            on_site
                .iter()
                .any(|overlay| overlay.kind == OverlayKind::Figure),
            "{on_site:?}"
        );
        let flag = on_site[at].rect;
        for later in &on_site[at + 1..] {
            assert!(
                !later.rect.intersects(flag),
                "{:?} recouvre le drapeau {flag:?} sur {hex:?}",
                later.kind
            );
        }
    }
}

// Spec 075, C12 : décor plein sur une forêt vide, dégagé sous la figurine, aucun sur une plaine.
#[test]
fn relief_decor_follows_terrain_and_occupancy() {
    let forest = Hex::new(0, -3);
    let plain = Hex::new(2, -2);
    let relief = |screen: &BoardScreen, hex: Hex| -> Vec<Overlay> {
        overlays_on(screen, hex)
            .into_iter()
            .filter(|overlay| overlay.kind == OverlayKind::Relief)
            .collect()
    };
    let empty = screen_around(&Units::default().state(), forest);
    let decor = relief(&empty, forest);
    assert_eq!(decor.len(), 1, "{:?}", overlays_on(&empty, forest));
    assert_eq!(decor[0].image, ImageId::new("relief", "forest"));
    assert_eq!(decor[0].foot, arena_like(forest));
    assert!(relief(&empty, plain).is_empty());

    let occupied = Units {
        figure: forest,
        ..Units::default()
    };
    let screen = screen_around(&occupied.state(), forest);
    let decor = relief(&screen, forest);
    assert_eq!(decor.len(), 1);
    assert_eq!(decor[0].image, ImageId::new("relief", "forest_clear"));
    assert_eq!(
        kinds_on(&screen, forest),
        [OverlayKind::Relief, OverlayKind::Figure]
    );

    assert_eq!(
        relief_image(Terrain::Hills, false),
        Some(ImageId::new("relief", "hills"))
    );
    assert_eq!(
        relief_image(Terrain::Mountain, true),
        Some(ImageId::new("relief", "mountain_clear"))
    );
    for flat in [
        Terrain::Plains,
        Terrain::Desert,
        Terrain::Swamp,
        Terrain::Wasteland,
        Terrain::Lake,
        Terrain::Sea,
    ] {
        assert_eq!(relief_image(flat, false), None);
    }
}

/// Le centre d'une case à l'écran, avec la caméra de [`screen_around`] centrée sur elle.
fn arena_like(hex: Hex) -> Pos2 {
    BoardCamera {
        center: hex_center(hex),
        zoom: 40.0,
    }
    .to_screen(area(), hex_center(hex))
}

// --- Écran de combat et images (étape 9) ---

// Spec 075, C20b : la carte d'un ennemi montre sa figurine, comme le plateau.
#[test]
fn combat_shows_the_figure() {
    assert_eq!(
        figure_image(Enemy::Prowlers),
        ImageId::new("figure", "prowlers")
    );
    let state = Units {
        placed: vec![(prowlers(), ORCS_2)],
        ..Units::default()
    }
    .state();
    let state = done(&state, challenge(ORCS_2));
    let state = done(&state, arcanum_core::Action::StartCombat);
    let screen = combat_screen(
        &view(&state, SOLO),
        &legal_actions(&state, SOLO),
        &CombatChoice::default(),
        arcanum_core::UndoStatus::Possible,
        &french(),
    )
    .expect("écran de combat");
    assert_eq!(screen.enemies.len(), 1);
    assert_eq!(screen.enemies[0].image, ImageId::new("figure", "prowlers"));
    assert_eq!(screen.enemies[0].fallback, enemy_color(EnemyColor::Green));
}

// Spec #14, C13 : la carte des Rôdeurs a son socle vert sous la figurine.
#[test]
fn combat_card_has_a_category_base() {
    let state = Units {
        placed: vec![(prowlers(), ORCS_2)],
        ..Units::default()
    }
    .state();
    let state = done(&state, challenge(ORCS_2));
    let state = done(&state, arcanum_core::Action::StartCombat);
    let screen = combat_screen(
        &view(&state, SOLO),
        &legal_actions(&state, SOLO),
        &CombatChoice::default(),
        arcanum_core::UndoStatus::Possible,
        &french(),
    )
    .expect("écran de combat");
    assert_eq!(screen.enemies[0].base, enemy_color(EnemyColor::Green));
}

// Spec 075, C20b : plus aucun jeton ; le dossier `enemy/` est supprimé.
#[test]
fn no_token_image_is_left() {
    let images_dir = repository_assets_dir().join("images");
    assert!(!images_dir.join("enemy").exists());
    let token_dir = std::path::Path::new("enemy");
    let enemies = Enemy::ALL.map(figure_image);
    for image in board_images().iter().chain(&enemies) {
        assert!(
            !image.path(std::path::Path::new("")).starts_with(token_dir),
            "{image:?}"
        );
    }
}
