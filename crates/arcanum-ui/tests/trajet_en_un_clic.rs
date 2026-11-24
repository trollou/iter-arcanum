//! Scénarios de la spec 036 côté affichage (trajet en un clic), sans fenêtre.

use arcanum_core::{
    Action, BasicAction, CardId, CardPlay, GameConfig, GameState, Hero, Hex, Scenario,
    SidewaysEffect, TilePosition, apply, decide, legal_actions, view,
};
use arcanum_i18n::{Catalog, Language};
use arcanum_ui::repository_assets_dir;
use arcanum_ui::{BoardCamera, BoardInput, BoardScreen, Command, board_screen, hex_center};
use eframe::egui::{Pos2, Rect, pos2, vec2};

mod support;

use support::board_input;

// L'arène des tests d'arcanum-core, partagée plutôt que recopiée.
#[path = "../../arcanum-core/tests/arene/mod.rs"]
mod arene;
use arene::*;

use BasicAction::*;

fn french() -> Catalog {
    Catalog::load(
        &repository_assets_dir().join("i18n"),
        &Language("fr".into()),
    )
    .expect("fr.json du dépôt")
}

fn config() -> GameConfig {
    GameConfig {
        seed: 0,
        scenario: Scenario::sandbox(),
        hero: Hero::Arythea,
        dummy_hero: Default::default(),
    }
}

fn area() -> Rect {
    Rect::from_min_size(pos2(0.0, 0.0), vec2(1000.0, 700.0))
}

fn camera() -> BoardCamera {
    BoardCamera {
        center: Pos2::ZERO,
        zoom: 30.0,
    }
}

fn hand() -> [CardId; 6] {
    [
        card(March, 1),
        card(March, 2),
        card(Stamina, 1),
        card(Swiftness, 1),
        card(Rage, 1),
        card(Improvisation, 1),
    ]
}

/// Bac à sable construit par événements : figurine sur `figure`, la main de `hand()`, les
/// Creuseurs sur les orques ① et les Rôdeurs sur les ②.
fn sandbox(figure: Hex) -> GameState {
    Sandbox {
        figure: Some(figure),
        hand: hand().to_vec(),
        placed: sandbox_tokens(),
        ..Sandbox::default()
    }
    .state()
}

/// `state` après avoir joué `played`, pour leur effet de base ou de côté pour Mouvement.
fn played(state: GameState, played: &[CardId]) -> GameState {
    played.iter().fold(state, |state, &card| {
        let basic = Action::PlayCard {
            card,
            play: CardPlay::Basic {
                die: None,
                gain: None,
                with: None,
                option: 0,
                discard: None,
                mana: None,
            },
        };
        let sideways = Action::PlayCard {
            card,
            play: CardPlay::Sideways(SidewaysEffect::Move),
        };
        let action = if legal_actions(&state, SOLO).contains(&basic) {
            basic
        } else {
            sideways
        };
        decide(&state, SOLO, &action, &mut config().rngs())
            .expect("action légale")
            .iter()
            .fold(state, apply)
    })
}

/// Figurine sur `figure` avec 0, 2, 5, 7 ou 9 points de mouvement.
fn with_points(figure: Hex, points: u32) -> GameState {
    let [march1, march2, stamina, swiftness, rage, _] = hand();
    let cards: &[CardId] = match points {
        0 => &[],
        2 => &[march1],
        5 => &[march1, march2, rage],
        7 => &[march1, march2, stamina, rage],
        9 => &[march1, march2, stamina, swiftness, rage],
        _ => panic!("{points} points non prévus"),
    };
    let state = played(sandbox(figure), cards);
    assert_eq!(view(&state, SOLO).move_points, points);
    state
}

fn screen_at(state: &GameState, cursor: Option<Hex>, catalog: &Catalog) -> BoardScreen {
    let seen = view(state, SOLO);
    let legal = legal_actions(state, SOLO);
    board_screen(
        &board_input(
            &seen,
            &legal,
            camera(),
            area(),
            cursor.map(|hex| camera().to_screen(area(), hex_center(hex))),
        ),
        catalog,
    )
}

fn follow_index(state: &GameState, to: Hex) -> usize {
    legal_actions(state, SOLO)
        .iter()
        .position(|action| *action == Action::FollowRoute { to })
        .expect("trajet légal")
}

// C11
#[test]
fn clicking_a_far_reachable_space_follows_the_route() {
    let state = with_points(Hex::ORIGIN, 5);
    let screen = screen_at(&state, Some(PLAIN_W), &french());
    assert_eq!(
        screen.click,
        Some(Command::Play(follow_index(&state, PLAIN_W)))
    );
}

// C12 — Q2 ; spec 017, Q4
#[test]
fn far_reachable_spaces_are_highlighted_like_neighbors() {
    let state = with_points(Hex::ORIGIN, 5);
    let screen = screen_at(&state, None, &french());
    let far: Vec<(Hex, bool)> = screen
        .move_marks
        .iter()
        .filter(|mark| !mark.neighbor)
        .map(|mark| (mark.hex, mark.reachable))
        .collect();
    assert_eq!(
        far,
        [
            (Hex::new(1, -2), true),
            (Hex::new(0, -2), true),
            (GREEN_MINE, true),
            (PLAIN_W, true),
        ]
    );
    // Le village coûte 6 : pas de marque.
    assert!(screen.move_marks.iter().all(|mark| mark.hex != VILLAGE));
    // Chaque case mise en évidence est une case où un clic agit.
    for mark in screen.move_marks.iter().filter(|mark| mark.reachable) {
        let click = screen_at(&state, Some(mark.hex), &french()).click;
        assert!(click.is_some(), "{:?}", mark.hex);
    }
}

// C13 — spec 017, C13
#[test]
fn hover_invites_to_follow_the_route() {
    let screen = screen_at(&with_points(Hex::ORIGIN, 5), Some(PLAIN_W), &french());
    assert_eq!(screen.route.len(), 3);
    let lines = &screen.hover_lines;
    for expected in [
        "Coût de déplacement : 2",
        "Trajet : 2 case(s), 5 points de mouvement",
        "Cliquez pour suivre ce trajet",
    ] {
        assert!(lines.contains(&expected.to_owned()), "{lines:?}");
    }
}

// C14 — MKUE p. 7, 4.c et 6
#[test]
fn clicking_a_too_expensive_far_space_does_nothing() {
    let state = with_points(VILLAGE, 5);
    let screen = screen_at(&state, Some(GREEN_MINE), &french());
    let lines = &screen.hover_lines;
    assert!(
        lines.contains(&"Il vous manque 2 point(s) de mouvement".to_owned()),
        "{lines:?}"
    );
    assert!(
        !lines.contains(&"Cliquez pour suivre ce trajet".to_owned()),
        "{lines:?}"
    );
    assert_eq!(screen.click, None);
}

// C15 — spec 030, C26 bis ; spec 016
#[test]
fn no_follow_click_when_movement_is_over() {
    // Au village avec 7 points, le pas vers la Plaine (2, −2) provoque les orques ②.
    let state = with_points(VILLAGE, 7);
    let state = decide(
        &state,
        SOLO,
        &Action::MoveHero { to: PLAIN_W },
        &mut config().rngs(),
    )
    .expect("pas légal")
    .iter()
    .fold(state, apply);
    assert!(view(&state, SOLO).movement_ended);
    for hex in [Hex::new(1, -1), Hex::new(0, -1), Hex::ORIGIN] {
        let screen = screen_at(&state, Some(hex), &french());
        assert_eq!(screen.click, None, "{hex:?}");
        assert!(
            screen
                .hover_lines
                .contains(&"Déplacement terminé pour ce tour".to_owned()),
            "{:?}",
            screen.hover_lines
        );
    }

    // Mode fin du tour : l'écran ne reçoit pas les données du déplacement.
    let state = with_points(Hex::ORIGIN, 5);
    let seen = view(&state, SOLO);
    let legal = legal_actions(&state, SOLO);
    let cursor = camera().to_screen(area(), hex_center(PLAIN_W));
    let screen = board_screen(
        &BoardInput {
            movement: None,
            ..board_input(&seen, &legal, camera(), area(), Some(cursor))
        },
        &french(),
    );
    assert_eq!(screen.click, None);
}

// C16
#[test]
fn every_follow_route_key_exists_in_french() {
    let catalog = french();
    let hexes: Vec<Hex> = [
        TilePosition::ORIGIN,
        TilePosition::from_lattice(0, 1),
        TilePosition::from_lattice(1, 0),
    ]
    .iter()
    .flat_map(|position| position.hexes())
    .collect();
    for points in [0, 2, 5, 9] {
        let state = with_points(Hex::ORIGIN, points);
        for hex in &hexes {
            screen_at(&state, Some(*hex), &catalog);
        }
    }
    let english = Catalog::load(
        &repository_assets_dir().join("i18n"),
        &Language("en".into()),
    )
    .expect("en.json du dépôt");
    english.text("board.hover.click_to_follow");
    assert_eq!(catalog.gaps(), vec![]);
    assert_eq!(english.gaps(), vec![]);
}
