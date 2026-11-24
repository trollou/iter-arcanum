//! Scénarios de la spec 012 côté affichage (plateau de la partie, zone interdite), sans fenêtre.

use std::collections::BTreeSet;

use arcanum_core::{GameConfig, Hex, MapShape, PlayerView, Scenario, Terrain, TilePosition};
use arcanum_i18n::{Catalog, Language};
use arcanum_ui::{
    BOARD_BACKGROUND, BoardCamera, BoardInput, BoardScreen, FORBIDDEN_COLOR, FORBIDDEN_IMAGE,
    GameSession, ScreenMode, ScreenPhase, board_screen, game_screen, hex_center,
    repository_assets_dir, terrain_color, terrain_image,
};
use eframe::egui::{Pos2, Rect, pos2, vec2};

mod support;

use support::first_reconnaissance_config;

use support::board_input;

fn french() -> Catalog {
    Catalog::load(
        &repository_assets_dir().join("i18n"),
        &Language("fr".into()),
    )
    .expect("fr.json du dépôt")
}

fn area() -> Rect {
    Rect::from_min_size(pos2(0.0, 0.0), vec2(1000.0, 700.0))
}

/// Partie de bac à sable : la configuration par défaut est passée à Première reconnaissance
/// (spec 063), qui a le même plateau de jeu.
fn sandbox_config() -> GameConfig {
    GameConfig {
        scenario: Scenario::sandbox(),
        ..first_reconnaissance_config(3)
    }
}

/// Vue joueur du bac à sable après la mise en place.
fn sandbox_view() -> PlayerView {
    let session = GameSession::new(sandbox_config());
    session.view()
}

/// Cadrage assez large pour voir la tuile de départ et ses six emplacements voisins.
fn wide_camera() -> BoardCamera {
    BoardCamera {
        center: Pos2::ZERO,
        zoom: 20.0,
    }
}

fn screen_of(view: &PlayerView, camera: BoardCamera, cursor: Option<Pos2>) -> BoardScreen {
    board_screen(
        &BoardInput {
            inventory: None,
            movement: None,
            ..board_input(view, &[], camera, area(), cursor)
        },
        &french(),
    )
}

fn forbidden_positions(screen: &BoardScreen) -> BTreeSet<TilePosition> {
    screen
        .forbidden
        .iter()
        .map(|drawing| TilePosition::containing(drawing.hex))
        .collect()
}

// Spec 012 C16 : l'application montre le plateau de jeu de la partie.
#[test]
fn board_shows_the_game_map() {
    let view = sandbox_view();
    assert_eq!(view.scenario, Some(Scenario::sandbox()));
    assert_eq!(view.figure, Some(Hex::ORIGIN));

    let camera = BoardCamera::initial(&view.map, area());
    let screen = screen_of(&view, camera, None);
    assert_eq!(screen.hexes.len(), 21);
    assert_eq!(screen.tile_outlines.len(), 3);
}

// Spec 012 C17 — MKUE p. 7 (5.a) et p. 14 : les emplacements vides hors forme sont hachurés.
#[test]
fn forbidden_zone_covers_empty_positions_outside_the_shape() {
    let view = sandbox_view();
    let screen = screen_of(&view, wide_camera(), None);
    let a_minus_b = TilePosition::from_lattice(1, -1);
    let b_minus_a = TilePosition::from_lattice(-1, 1);

    for position in [a_minus_b, b_minus_a] {
        let drawn: BTreeSet<Hex> = screen
            .forbidden
            .iter()
            .map(|drawing| drawing.hex)
            .filter(|hex| TilePosition::containing(*hex) == position)
            .collect();
        assert_eq!(drawn, position.hexes().into_iter().collect());
    }
    let placed: BTreeSet<TilePosition> = view.map.tiles().map(|(position, _)| position).collect();
    for position in forbidden_positions(&screen) {
        assert!(!MapShape::Wedge.allows(position), "{position:?} est permis");
        assert!(!placed.contains(&position), "{position:?} a une tuile");
    }
    assert!(!forbidden_positions(&screen).contains(&TilePosition::from_lattice(2, 0)));
}

// Spec 012 C18 : seules les cases hachurées visibles sont dans l'écran.
#[test]
fn forbidden_zone_is_culled() {
    let view = sandbox_view();
    let screen = screen_of(&view, wide_camera(), None);
    assert!(!screen.forbidden.is_empty());
    for drawing in &screen.forbidden {
        assert!(Rect::from_points(&drawing.corners).intersects(area()));
    }
    // Une case loin hors de la zone n'est pas dessinée.
    let far = Hex::new(-40, 0);
    assert!(screen.forbidden.iter().all(|drawing| drawing.hex != far));

    // Une case à cheval sur le bord gauche y est.
    let edge = screen
        .forbidden
        .iter()
        .find(|drawing| drawing.corners.iter().any(|corner| corner.x < 0.0));
    assert!(edge.is_some(), "aucune case à cheval sur le bord");
}

// Spec 012 C19, adapté par la spec 075 (C9) : la mer, et une couleur de repli bleue.
#[test]
fn forbidden_zone_has_an_image_and_a_fallback() {
    let view = sandbox_view();
    let screen = screen_of(&view, wide_camera(), None);
    assert!(screen.forbidden.iter().all(|drawing| {
        drawing.image == FORBIDDEN_IMAGE && drawing.fallback == FORBIDDEN_COLOR
    }));
    assert_eq!(FORBIDDEN_IMAGE, terrain_image(Terrain::Sea));

    assert_ne!(FORBIDDEN_COLOR, BOARD_BACKGROUND);
    for terrain in [
        Terrain::Plains,
        Terrain::Hills,
        Terrain::Forest,
        Terrain::Wasteland,
        Terrain::Desert,
        Terrain::Swamp,
        Terrain::Lake,
        Terrain::Mountain,
        Terrain::Sea,
    ] {
        assert_ne!(FORBIDDEN_COLOR, terrain_color(terrain));
    }
}

// Spec 012 C20 : le survol de la zone interdite l'annonce.
#[test]
fn hovering_the_forbidden_zone_says_so() {
    let view = sandbox_view();
    let camera = wide_camera();
    let cursor_on = |hex: Hex| Some(camera.to_screen(area(), hex_center(hex)));

    let forbidden = TilePosition::from_lattice(1, -1).center();
    let screen = screen_of(&view, camera, cursor_on(forbidden));
    assert_eq!(screen.hovered, None);
    assert_eq!(
        screen.hover_lines,
        vec!["Au-delà de la côte : aucune tuile ne peut être posée ici".to_string()]
    );

    let allowed_empty = TilePosition::from_lattice(2, 0).center();
    let screen = screen_of(&view, camera, cursor_on(allowed_empty));
    // Spec 037, C3 : hors partie, un emplacement permis n'a rien à décrire.
    assert!(screen.hover_lines.is_empty());
}

// Spec 012 C21 ; spec 074, C4 : la barre de menu donne le scénario seul, sans le héros (remplace
// `side_panel_shows_scenario_and_tile_deck` ; les pioches : `bar_shows_tile_decks`).
#[test]
fn bar_names_sandbox_only() {
    let session = GameSession::new(sandbox_config());
    let bar = session.menu_bar(&french());
    assert_eq!(bar.scenario.as_deref(), Some("Bac à sable"));
}

// Spec 012 C22 : aucune clé de la mise en place ne manque en français.
#[test]
fn every_setup_key_exists_in_french() {
    let catalog = french();
    let view = sandbox_view();
    let camera = wide_camera();
    let base = screen_of(&view, camera, None);
    let mut hexes: Vec<Hex> = base.hexes.iter().map(|drawing| drawing.hex).collect();
    hexes.extend(base.forbidden.iter().map(|drawing| drawing.hex));
    hexes.push(TilePosition::from_lattice(2, 0).center());
    for hex in hexes {
        let cursor = Some(camera.to_screen(area(), hex_center(hex)));
        board_screen(
            &BoardInput {
                inventory: None,
                movement: None,
                ..board_input(&view, &[], camera, area(), cursor)
            },
            &catalog,
        );
    }
    game_screen(&view, ScreenPhase::Finished, &ScreenMode::Normal, &catalog);
    assert_eq!(catalog.gaps(), vec![]);
}
