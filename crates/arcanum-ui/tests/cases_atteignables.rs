//! Scénarios de la spec 017 côté affichage (cases atteignables, tracé du trajet, survol), sans fenêtre.

use arcanum_core::{
    Action, BasicAction, CardId, CardPlay, DeedCard, Event, GameConfig, GameState, Hero, Hex,
    PlayerId, Scenario, SidewaysEffect, TileId, TilePosition, apply, decide, legal_actions, view,
};
use arcanum_i18n::{Catalog, Language};
use arcanum_ui::{
    BoardCamera, BoardInput, BoardScreen, Command, OVERLAY_MIN_ZOOM, board_screen, hex_center,
    repository_assets_dir,
};
use eframe::egui::{Pos2, Rect, pos2, vec2};

mod support;

use support::board_input;

const SOLO: PlayerId = PlayerId::SOLO;

use BasicAction::*;

fn french() -> Catalog {
    Catalog::load(
        &repository_assets_dir().join("i18n"),
        &Language("fr".into()),
    )
    .expect("fr.json du dépôt")
}

fn card(action: BasicAction, copy: u16) -> CardId {
    CardId::new(DeedCard::Basic(action), copy)
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

/// Tuiles du bac à sable à la mise en place : départ A, campagne 1 en B, campagne 2 en A.
fn sandbox_tiles() -> Vec<(TileId, TilePosition)> {
    vec![
        (TileId::StartingA, TilePosition::ORIGIN),
        (TileId::Countryside(1), TilePosition::from_lattice(0, 1)),
        (TileId::Countryside(2), TilePosition::from_lattice(1, 0)),
    ]
}

/// Partie construite par événements, figurine sur `figure`, `hand` en main, puis `played`
/// joués pour leur effet de base (ou de côté pour Mouvement s'ils n'en ont pas de jouable).
fn state_with(
    tiles: &[(TileId, TilePosition)],
    figure: Hex,
    hand: &[CardId],
    played: &[CardId],
) -> GameState {
    let mut order = hand.to_vec();
    order.extend(
        Hero::Arythea
            .starting_deck()
            .into_iter()
            .filter(|card| !hand.contains(card)),
    );
    let mut events = vec![Event::GameStarted { config: config() }];
    events.extend(
        tiles
            .iter()
            .map(|&(tile, position)| Event::TilePlaced { tile, position }),
    );
    events.extend([
        Event::HeroFigurePlaced {
            player: SOLO,
            hex: figure,
        },
        Event::DeckShuffled {
            player: SOLO,
            order,
        },
    ]);
    events.extend(
        hand.iter()
            .map(|&card| Event::CardDrawn { player: SOLO, card }),
    );
    let mut state = events.iter().fold(GameState::default(), apply);
    for &card in played {
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
        state = decide(&state, SOLO, &action, &mut config().rngs())
            .expect("action légale")
            .iter()
            .fold(state, apply);
    }
    state
}

/// Bac à sable, figurine sur le portail, Marche jouée : 2 points de mouvement.
fn two_points() -> GameState {
    let march = card(March, 1);
    state_with(&sandbox_tiles(), Hex::ORIGIN, &[march], &[march])
}

fn screen_at(
    state: &GameState,
    camera: BoardCamera,
    cursor: Option<Hex>,
    catalog: &Catalog,
) -> BoardScreen {
    let seen = view(state, SOLO);
    let legal = legal_actions(state, SOLO);
    board_screen(
        &board_input(
            &seen,
            &legal,
            camera,
            area(),
            cursor.map(|hex| camera.to_screen(area(), hex_center(hex))),
        ),
        catalog,
    )
}

fn index_of(state: &GameState, to: Hex) -> usize {
    legal_actions(state, SOLO)
        .iter()
        .position(|action| *action == Action::MoveHero { to })
        .expect("déplacement légal")
}

/// Bac à sable, figurine sur le portail, Marche, Marche et Rapidité de côté : 5 points.
fn five_points() -> GameState {
    let hand = [card(March, 1), card(March, 2), card(Swiftness, 1)];
    let state = state_with(&sandbox_tiles(), Hex::ORIGIN, &hand, &hand[..2]);
    let sideways = Action::PlayCard {
        card: hand[2],
        play: CardPlay::Sideways(SidewaysEffect::Move),
    };
    let state = decide(&state, SOLO, &sideways, &mut config().rngs())
        .expect("action légale")
        .iter()
        .fold(state, apply);
    assert_eq!(view(&state, SOLO).move_points, 5);
    state
}

fn marks(screen: &BoardScreen) -> Vec<(Hex, u8, bool, bool)> {
    screen
        .move_marks
        .iter()
        .map(|mark| (mark.hex, mark.cost, mark.reachable, mark.neighbor))
        .collect()
}

fn center(hex: Hex) -> Pos2 {
    camera().to_screen(area(), hex_center(hex))
}

// Spec 017 C11
#[test]
fn all_reachable_spaces_are_highlighted() {
    let screen = screen_at(&five_points(), camera(), None, &french());
    assert_eq!(
        marks(&screen),
        vec![
            // Voisines, dans l'ordre des directions : E, N-E, N-O.
            (Hex::new(1, 0), 2, true, true),
            (Hex::new(1, -1), 3, true, true),
            (Hex::new(0, -1), 2, true, true),
            // Les autres, par coût croissant puis dans l'ordre de leur trajet.
            // À coût égal : NW, NE avant NW, NW.
            (Hex::new(1, -2), 4, true, false),
            (Hex::new(0, -2), 4, true, false),
            (Hex::new(2, -1), 5, true, false),
            (Hex::new(2, -2), 5, true, false),
        ]
    );
}

// Spec 017 C12 : le résultat de la spec 014 ne change pas.
#[test]
fn two_points_highlight_only_neighbors() {
    let screen = screen_at(&two_points(), camera(), None, &french());
    assert_eq!(
        marks(&screen),
        vec![
            (Hex::new(1, 0), 2, true, true),
            (Hex::new(1, -1), 3, false, true),
            (Hex::new(0, -1), 2, true, true),
        ]
    );
    assert!(screen.route.is_empty());
}

// Spec 017 C13
#[test]
fn hover_shows_the_route() {
    let target = Hex::new(2, -2);
    let screen = screen_at(&five_points(), camera(), Some(target), &french());
    assert_eq!(
        screen.route,
        vec![center(Hex::ORIGIN), center(Hex::new(1, -1)), center(target)]
    );
    let lines = &screen.hover_lines;
    assert!(
        lines.contains(&"Coût de déplacement : 2".to_owned()),
        "{lines:?}"
    );
    assert!(
        lines.contains(&"Trajet : 2 case(s), 5 points de mouvement".to_owned()),
        "{lines:?}"
    );
    // Spec 036 : la case lointaine se suit en un clic, sans « Cliquez pour vous y déplacer ».
    assert!(
        !lines.contains(&"Cliquez pour vous y déplacer".to_owned()),
        "{lines:?}"
    );
    assert!(
        lines.contains(&"Cliquez pour suivre ce trajet".to_owned()),
        "{lines:?}"
    );
}

// Spec 017 C14
#[test]
fn hover_on_a_neighbor_has_no_route_line() {
    let target = Hex::new(1, -1);
    let screen = screen_at(&five_points(), camera(), Some(target), &french());
    assert_eq!(screen.route, vec![center(Hex::ORIGIN), center(target)]);
    let lines = &screen.hover_lines;
    assert!(
        !lines.iter().any(|line| line.starts_with("Trajet")),
        "{lines:?}"
    );
    assert!(
        lines.contains(&"Cliquez pour vous y déplacer".to_owned()),
        "{lines:?}"
    );
}

// Spec 017 C15
#[test]
fn hover_on_a_far_space_tells_missing_points() {
    let screen = screen_at(&five_points(), camera(), Some(Hex::new(2, -3)), &french());
    assert!(screen.route.is_empty());
    let lines = &screen.hover_lines;
    assert!(
        lines.contains(&"Il vous manque 1 point(s) de mouvement".to_owned()),
        "{lines:?}"
    );
    assert!(
        !lines
            .iter()
            .any(|line| line.starts_with("Cliquez") || line.starts_with("Trajet")),
        "{lines:?}"
    );
}

// Spec 017 C16 (Q1), revu par la spec 036 : une case lointaine atteignable suit son trajet ; une
// case trop chère et la case de la figurine ne font toujours rien.
#[test]
fn clicking_a_far_space_does_nothing() {
    let state = five_points();
    let click = |hex| screen_at(&state, camera(), Some(hex), &french()).click;
    assert_eq!(
        click(Hex::new(1, -1)),
        Some(Command::Play(index_of(&state, Hex::new(1, -1))))
    );
    for to in [Hex::new(2, -2), Hex::new(0, -2)] {
        let follow = legal_actions(&state, SOLO)
            .iter()
            .position(|action| *action == Action::FollowRoute { to })
            .expect("trajet légal");
        assert_eq!(click(to), Some(Command::Play(follow)), "{to:?}");
    }
    for hex in [Hex::new(2, -3), Hex::ORIGIN] {
        assert_eq!(click(hex), None, "{hex:?}");
    }
}

// Spec 017 C17 : en mode fin du tour, l'écran ne reçoit pas les données du déplacement (spec 016).
#[test]
fn end_turn_mode_hides_routes() {
    let state = five_points();
    let seen = view(&state, SOLO);
    let legal = legal_actions(&state, SOLO);
    let screen = board_screen(
        &BoardInput {
            movement: None,
            ..board_input(
                &seen,
                &legal,
                camera(),
                area(),
                Some(center(Hex::new(2, -2))),
            )
        },
        &french(),
    );
    assert!(screen.move_marks.is_empty());
    assert!(screen.route.is_empty());
    assert_eq!(screen.click, None);
}

// Spec 017 C17 bis : sans décision en attente, aucune case lointaine n'est marquée.
#[test]
fn no_pending_decision_marks_no_far_space() {
    let state = five_points();
    let seen = view(&state, SOLO);
    let screen = board_screen(
        &board_input(&seen, &[], camera(), area(), Some(center(Hex::new(2, -2)))),
        &french(),
    );
    assert!(
        screen
            .move_marks
            .iter()
            .all(|mark| mark.neighbor && !mark.reachable)
    );
    assert!(screen.route.is_empty());
}

// Spec 017 C18
#[test]
fn routes_are_not_drawn_when_zoomed_out() {
    let far = BoardCamera {
        center: Pos2::ZERO,
        zoom: OVERLAY_MIN_ZOOM - 1.0,
    };
    let target = Hex::new(2, -2);
    let cursor = far.to_screen(area(), hex_center(target));
    let state = five_points();
    let seen = view(&state, SOLO);
    let legal = legal_actions(&state, SOLO);
    let screen = board_screen(
        &board_input(&seen, &legal, far, area(), Some(cursor)),
        &french(),
    );
    assert!(screen.move_marks.is_empty());
    assert!(screen.route.is_empty());
}

// Spec 017 L2
#[test]
fn zero_points_mark_nothing_reachable() {
    let state = state_with(&sandbox_tiles(), Hex::ORIGIN, &[card(March, 1)], &[]);
    let screen = screen_at(&state, camera(), Some(Hex::new(1, -2)), &french());
    assert!(screen.move_marks.iter().all(|mark| !mark.reachable));
    assert!(screen.route.is_empty());
    assert!(
        screen
            .hover_lines
            .contains(&"Il vous manque 4 point(s) de mouvement".to_owned()),
        "{:?}",
        screen.hover_lines
    );
}

// Spec 017 C19
#[test]
fn every_route_key_exists_in_french() {
    let catalog = french();
    let hexes: Vec<Hex> = sandbox_tiles()
        .iter()
        .flat_map(|(_, position)| position.hexes())
        .collect();
    let zero = state_with(&sandbox_tiles(), Hex::ORIGIN, &[card(March, 1)], &[]);
    let nine = {
        let hand = [
            card(March, 1),
            card(March, 2),
            card(Swiftness, 1),
            card(Stamina, 1),
            card(Rage, 1),
        ];
        state_with(&sandbox_tiles(), Hex::ORIGIN, &hand, &hand)
    };
    for state in [zero, two_points(), five_points(), nine] {
        for hex in &hexes {
            screen_at(&state, camera(), Some(*hex), &catalog);
        }
    }
    let english = Catalog::load(
        &repository_assets_dir().join("i18n"),
        &Language("en".into()),
    )
    .expect("en.json du dépôt");
    english.format("board.hover.route", &[("steps", "2"), ("cost", "5")]);
    assert_eq!(catalog.gaps(), vec![]);
    assert_eq!(english.gaps(), vec![]);
}
