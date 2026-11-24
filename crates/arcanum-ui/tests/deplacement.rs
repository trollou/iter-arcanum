//! Scénarios de la spec 014 côté affichage (jour, cases mises en évidence, survol, clic), sans fenêtre.

use arcanum_core::{
    Action, BasicAction, CardId, CardPlay, DeedCard, Event, GameConfig, GameState, Hero, Hex,
    PlayerId, Scenario, SidewaysEffect, TileId, TilePosition, apply, decide, legal_actions, view,
};
use arcanum_i18n::{Catalog, Language};
use arcanum_ui::{
    BoardCamera, BoardScreen, Command, OVERLAY_MIN_ZOOM, ScreenMode, ScreenPhase, board_screen,
    game_screen, hex_center, repository_assets_dir,
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

fn hover(state: &GameState, hex: Hex) -> Vec<String> {
    screen_at(state, camera(), Some(hex), &french()).hover_lines
}

fn index_of(state: &GameState, to: Hex) -> usize {
    legal_actions(state, SOLO)
        .iter()
        .position(|action| *action == Action::MoveHero { to })
        .expect("déplacement légal")
}

// Spec 014 C17
#[test]
fn panel_shows_time_of_day() {
    let state = two_points();
    // Spec 016 : la ligne porte aussi le numéro du tour ; spec 074, C5 : dans la barre de menu,
    // le jour dit par l'icône.
    let bar = support::bar_of(&view(&state, SOLO), &french());
    assert_eq!(bar.round, "Manche 1/4 (tour 1)");
    assert_eq!(bar.clock.tooltip, "Jour");
}

// Spec 014 C18
#[test]
fn reachable_neighbors_are_highlighted() {
    let screen = screen_at(&two_points(), camera(), None, &french());
    let marks: Vec<(Hex, u8, bool)> = screen
        .move_marks
        .iter()
        .map(|mark| (mark.hex, mark.cost, mark.reachable))
        .collect();
    // Dans l'ordre des directions : E (Plaine), N-E (Forêt, trop chère), N-O (Plaine).
    assert_eq!(
        marks,
        vec![
            (Hex::new(1, 0), 2, true),
            (Hex::new(1, -1), 3, false),
            (Hex::new(0, -1), 2, true),
        ]
    );
    // Sous le zoom minimal des surimpressions, pas de marques.
    let far = BoardCamera {
        center: Pos2::ZERO,
        zoom: OVERLAY_MIN_ZOOM - 1.0,
    };
    assert!(
        screen_at(&two_points(), far, None, &french())
            .move_marks
            .is_empty()
    );
}

// Spec 014 C19
#[test]
fn hover_shows_move_cost() {
    let state = two_points();
    let plains = hover(&state, Hex::new(1, 0));
    assert!(
        plains.contains(&"Coût de déplacement : 2".to_owned()),
        "{plains:?}"
    );
    assert!(
        plains.contains(&"Cliquez pour vous y déplacer".to_owned()),
        "{plains:?}"
    );

    let forest = hover(&state, Hex::new(1, -1));
    assert!(
        forest.contains(&"Coût de déplacement : 3".to_owned()),
        "{forest:?}"
    );
    assert!(
        forest.contains(&"Il vous manque 1 point(s) de mouvement".to_owned()),
        "{forest:?}"
    );

    let sea = hover(&state, Hex::new(0, 1));
    assert!(sea.contains(&"Infranchissable".to_owned()), "{sea:?}");

    // Case non adjacente : le coût, sans invitation ; depuis la spec 017 (Q4), ce qui manque
    // pour son trajet (7 points, il en manque 5).
    let far = hover(&state, Hex::new(1, -3));
    assert!(
        far.contains(&"Coût de déplacement : 3".to_owned()),
        "{far:?}"
    );
    assert!(!far.iter().any(|line| line.starts_with("Cliquez")));
    assert!(
        far.contains(&"Il vous manque 5 point(s) de mouvement".to_owned()),
        "{far:?}"
    );

    // Forteresse de la tuile de campagne 3, voisine de la figurine : depuis la spec 043, un assaut
    // (sans défenseur ici, posé seulement à l'exploration).
    let tiles = [
        (TileId::StartingA, TilePosition::ORIGIN),
        (TileId::Countryside(3), TilePosition::from_lattice(1, 0)),
    ];
    let hand = [card(March, 1), card(March, 2)];
    let state = state_with(&tiles, Hex::new(3, -2), &hand, &hand);
    let keep = hover(&state, Hex::new(4, -3));
    assert!(
        keep.contains(&"⚔ Assaut : Réputation −1, combat obligatoire".to_owned()),
        "{keep:?}"
    );
    assert!(
        !keep.iter().any(|line| line.starts_with("Cité")),
        "{keep:?}"
    );
}

// Spec 014 C20
#[test]
fn clicking_a_highlighted_space_moves_the_hero() {
    let state = two_points();
    let click = |hex| screen_at(&state, camera(), Some(hex), &french()).click;
    assert_eq!(
        click(Hex::new(1, 0)),
        Some(Command::Play(index_of(&state, Hex::new(1, 0))))
    );
    assert_eq!(
        click(Hex::new(0, -1)),
        Some(Command::Play(index_of(&state, Hex::new(0, -1))))
    );
    // Trop chère, infranchissable, non adjacente, figurine, sans tuile.
    for hex in [
        Hex::new(1, -1),
        Hex::new(0, 1),
        Hex::new(1, -3),
        Hex::ORIGIN,
        Hex::new(-3, 3),
    ] {
        assert_eq!(click(hex), None, "{hex:?}");
    }
    // Sans curseur, rien.
    assert_eq!(screen_at(&state, camera(), None, &french()).click, None);
}

// Spec 014 C21
#[test]
fn every_move_key_exists_in_french() {
    let catalog = french();
    let hexes: Vec<Hex> = sandbox_tiles()
        .iter()
        .flat_map(|(_, position)| position.hexes())
        .collect();
    let rage = card(Rage, 1);
    let march = card(March, 1);
    let five = [card(March, 1), card(Stamina, 1), rage];
    for state in [
        state_with(&sandbox_tiles(), Hex::ORIGIN, &[rage], &[]),
        two_points(),
        state_with(&sandbox_tiles(), Hex::ORIGIN, &five, &five),
        state_with(&sandbox_tiles(), Hex::new(1, -3), &[march], &[march]),
    ] {
        for &hex in &hexes {
            screen_at(&state, camera(), Some(hex), &catalog);
        }
        let legal = legal_actions(&state, SOLO);
        game_screen(
            &view(&state, SOLO),
            ScreenPhase::Playing {
                legal_actions: &legal,
            },
            &ScreenMode::Normal,
            &catalog,
        );
    }
    let english = Catalog::load(
        &repository_assets_dir().join("i18n"),
        &Language("en".into()),
    )
    .expect("en.json du dépôt");
    for language in [&catalog, &english] {
        for key in [
            "time_of_day.day",
            "time_of_day.night",
            "board.hover.inaccessible",
            "board.hover.fortified",
            "board.hover.click_to_move",
        ] {
            language.text(key);
        }
        language.format("board.hover.move_cost", &[("cost", "2")]);
        language.format("board.hover.missing_points", &[("missing", "1")]);
    }
    assert_eq!(catalog.gaps(), vec![]);
    assert_eq!(english.gaps(), vec![]);
}
