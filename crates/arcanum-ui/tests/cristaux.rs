//! Scénarios de la spec 021 côté affichage (inventaire dans le panneau, survol des mines).

use arcanum_core::{
    BasicColor, Event, GameConfig, GameState, Hero, Hex, PlayerId, PlayerView, Scenario, TileId,
    TilePosition, apply, legal_actions, mine_color, view,
};
use arcanum_i18n::{Catalog, Language};
use arcanum_ui::{
    BoardCamera, GameScreen, ScreenMode, ScreenPhase, board_screen, game_screen, hex_center,
    repository_assets_dir,
};
use eframe::egui::{Pos2, Rect, pos2, vec2};

mod support;

use support::board_input;

const SOLO: PlayerId = PlayerId::SOLO;

use BasicColor::*;

fn french() -> Catalog {
    Catalog::load(
        &repository_assets_dir().join("i18n"),
        &Language("fr".into()),
    )
    .expect("fr.json du dépôt")
}

fn east() -> TilePosition {
    TilePosition::from_lattice(1, 0)
}

/// Bac à sable : tuile de départ et `tile` à l'est, figurine sur le portail, puis les
/// cristaux `crystals` gagnés.
fn state_with(tile: TileId, crystals: &[(BasicColor, u8)]) -> GameState {
    let config = GameConfig {
        seed: 0,
        scenario: Scenario::sandbox(),
        hero: Hero::Tovak,
        dummy_hero: Default::default(),
    };
    let mut events = vec![
        Event::GameStarted { config },
        Event::TilePlaced {
            tile: TileId::StartingA,
            position: TilePosition::ORIGIN,
        },
        Event::TilePlaced {
            tile,
            position: east(),
        },
        Event::HeroFigurePlaced {
            player: SOLO,
            hex: Hex::ORIGIN,
        },
    ];
    for &(color, count) in crystals {
        events.extend((0..count).map(|_| Event::CrystalGained {
            player: SOLO,
            color,
        }));
    }
    events.iter().fold(GameState::default(), apply)
}

fn screen(state: &GameState, catalog: &Catalog) -> GameScreen {
    let legal = legal_actions(state, SOLO);
    game_screen(
        &view(state, SOLO),
        ScreenPhase::Playing {
            legal_actions: &legal,
        },
        &ScreenMode::Normal,
        catalog,
    )
}

fn mine_of(state: &GameState) -> Hex {
    east()
        .hexes()
        .into_iter()
        .find(|&hex| mine_color(state.map(), hex).is_some())
        .expect("la tuile doit avoir une mine")
}

fn hover(state: &GameState, hex: Hex, catalog: &Catalog) -> Vec<String> {
    let seen: PlayerView = view(state, SOLO);
    let legal = legal_actions(state, SOLO);
    let area = Rect::from_min_size(pos2(0.0, 0.0), vec2(1000.0, 700.0));
    let camera = BoardCamera {
        center: Pos2::ZERO,
        zoom: 30.0,
    };
    board_screen(
        &board_input(
            &seen,
            &legal,
            camera,
            area,
            Some(camera.to_screen(area, hex_center(hex))),
        ),
        catalog,
    )
    .hover_lines
}

// Spec 021 C13 — MKUE p. 5, 1.b
#[test]
fn panel_shows_inventory() {
    let catalog = french();
    let state = state_with(TileId::Countryside(2), &[(Green, 2), (Red, 3)]);
    let inventory = screen(&state, &catalog).inventory;
    assert_eq!(inventory.title, "Inventaire");
    let groups: Vec<(BasicColor, u8, u8)> = inventory
        .groups
        .iter()
        .map(|group| (group.color, group.count, group.limit))
        .collect();
    assert_eq!(
        groups,
        [(Red, 3, 3), (Blue, 0, 3), (White, 0, 3), (Green, 2, 3)]
    );
}

// Spec 021 C14
#[test]
fn inventory_tooltip_shows_count_and_limit() {
    let catalog = french();
    let state = state_with(TileId::Countryside(2), &[(Green, 1)]);
    let inventory = screen(&state, &catalog).inventory;
    let tooltips: Vec<&str> = inventory
        .groups
        .iter()
        .map(|group| group.tooltip.as_str())
        .collect();
    assert_eq!(
        tooltips,
        [
            "Cristaux rouges : 0 sur 3",
            "Cristaux bleus : 0 sur 3",
            "Cristaux blancs : 0 sur 3",
            "Cristaux verts : 1 sur 3",
        ]
    );
}

// Spec 021 C15 — MKUE p. 11, 4.b
#[test]
fn hovering_a_mine_tells_what_it_gives() {
    let catalog = french();
    let state = state_with(TileId::Countryside(2), &[]);
    let lines = hover(&state, mine_of(&state), &catalog);
    let site = lines
        .iter()
        .position(|line| line == "Mine verte")
        .expect("le nom du site");
    assert_eq!(
        lines[site + 1],
        "Finir le tour ici : +1 cristal <green>vert</green>"
    );

    let full = state_with(TileId::Countryside(2), &[(Green, 3)]);
    let lines = hover(&full, mine_of(&full), &catalog);
    let site = lines
        .iter()
        .position(|line| line == "Mine verte")
        .expect("le nom du site");
    assert_eq!(
        lines[site + 1],
        "Vous avez déjà 3 cristaux de cette couleur : rien à gagner ici."
    );

    // Une case sans mine n'a pas ces lignes.
    let lines = hover(&state, east().hex_at(arcanum_core::Direction::W), &catalog);
    assert!(!lines.iter().any(|line| line.contains("cristal")));
}

// Spec 021 C16
#[test]
fn every_inventory_key_exists_in_french() {
    let catalog = french();
    let mines = [
        (TileId::Countryside(6), Red),
        (TileId::Countryside(5), Blue),
        (TileId::Countryside(3), White),
        (TileId::Countryside(2), Green),
    ];
    for count in 0..=3 {
        for (tile, color) in mines {
            let state = state_with(tile, &[(color, count)]);
            screen(&state, &catalog);
            hover(&state, mine_of(&state), &catalog);
        }
    }
    assert_eq!(catalog.gaps(), vec![]);
}
