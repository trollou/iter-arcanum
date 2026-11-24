//! Scénarios de la spec 015 côté affichage (fantômes, survol, clic, panneau), sans fenêtre.

use arcanum_core::{
    Action, BasicAction, CardId, CardPlay, DeedCard, Event, GameConfig, GameState, Hero, Hex,
    PlayerId, Scenario, SidewaysEffect, TileId, TilePosition, apply, decide, legal_actions, view,
};
use arcanum_i18n::{Catalog, Language};
use arcanum_ui::{
    BoardCamera, BoardScreen, Command, Icon, OVERLAY_MIN_ZOOM, ScreenMode, ScreenPhase,
    board_screen, game_screen, hex_center, repository_assets_dir,
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

fn english() -> Catalog {
    Catalog::load(
        &repository_assets_dir().join("i18n"),
        &Language("en".into()),
    )
    .expect("en.json du dépôt")
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

fn pos(i: i32, j: i32) -> TilePosition {
    TilePosition::from_lattice(i, j)
}

fn sandbox_deck() -> Vec<TileId> {
    let mut deck: Vec<TileId> = (1..=8).map(TileId::Countryside).collect();
    deck.extend([TileId::Core(2), TileId::Core(6), TileId::Core(3)]);
    deck
}

/// Mise en place du bac à sable par événements (pioche `deck`, ses deux premières tuiles en `B`
/// puis `A`), `extra` posées en plus, figurine sur `figure`, `hand` en main puis `played` joués
/// pour leur effet de base, ou de côté pour Mouvement s'ils n'en ont pas de jouable.
fn state_with(
    deck: Vec<TileId>,
    extra: &[(TileId, TilePosition)],
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
    let mut events = vec![
        Event::GameStarted { config: config() },
        Event::TilePlaced {
            tile: TileId::StartingA,
            position: TilePosition::ORIGIN,
        },
        Event::TileDeckFormed {
            order: deck.clone(),
        },
    ];
    events.extend(
        deck.iter()
            .zip([pos(0, 1), pos(1, 0)])
            .map(|(&tile, position)| Event::TileRevealed { tile, position }),
    );
    events.extend(
        extra
            .iter()
            .map(|&(tile, position)| Event::TileRevealed { tile, position }),
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

/// Figurine sur la Forêt N-E de la tuile 2, qui borde `A + B` et `2A`, avec 2 points (Marche).
fn on_forest_with_two_points() -> GameState {
    let march = card(March, 1);
    state_with(sandbox_deck(), &[], Hex::new(4, -3), &[march], &[march])
}

/// Même case, 1 point (Rage de côté).
fn on_forest_with_one_point() -> GameState {
    let rage = card(Rage, 1);
    state_with(sandbox_deck(), &[], Hex::new(4, -3), &[rage], &[rage])
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

fn ghosts(screen: &BoardScreen) -> Vec<(TilePosition, bool)> {
    screen
        .explore_ghosts
        .iter()
        .map(|ghost| (ghost.position, ghost.reachable))
        .collect()
}

// Spec 030 C26 bis — MKUE p. 6, 6.c : explorer fait partie du déplacement ; une fois celui-ci fini
// (provocation, défi), l'emplacement ne dit plus qu'il manque des points (issue privée 65).
#[test]
fn hover_says_exploration_is_over() {
    let state = on_forest_with_one_point();
    let mut seen = view(&state, SOLO);
    seen.movement_ended = true;
    // Le moteur ne propose plus d'exploration une fois le déplacement fini.
    let legal: Vec<Action> = legal_actions(&state, SOLO)
        .into_iter()
        .filter(|action| !matches!(action, Action::Explore { .. }))
        .collect();
    let screen = board_screen(
        &board_input(
            &seen,
            &legal,
            camera(),
            area(),
            Some(camera().to_screen(area(), hex_center(pos(2, 0).center()))),
        ),
        &french(),
    );
    assert_eq!(
        screen.hover_lines,
        vec![
            "Emplacement libre".to_owned(),
            "Explorer : 2 points de mouvement".to_owned(),
            "Déplacement terminé pour ce tour".to_owned(),
        ]
    );
}

// Spec 015 C17
#[test]
fn explorable_positions_are_drawn_as_ghosts() {
    let screen = screen_at(&on_forest_with_two_points(), camera(), None, &french());
    assert_eq!(ghosts(&screen), vec![(pos(1, 1), true), (pos(2, 0), true)]);
    for ghost in &screen.explore_ghosts {
        assert_eq!(ghost.hexes.len(), 7);
        assert!(!ghost.segments.is_empty());
        assert!(ghost.badge.is_some());
    }

    let poor = screen_at(&on_forest_with_one_point(), camera(), None, &french());
    assert_eq!(ghosts(&poor), vec![(pos(1, 1), false), (pos(2, 0), false)]);

    let march = card(March, 1);
    let portal = state_with(sandbox_deck(), &[], Hex::ORIGIN, &[march], &[march]);
    assert!(
        screen_at(&portal, camera(), None, &french())
            .explore_ghosts
            .is_empty()
    );

    // Sous le zoom minimal des surimpressions : le contour, pas le jeton.
    let far = BoardCamera {
        center: Pos2::ZERO,
        zoom: OVERLAY_MIN_ZOOM - 1.0,
    };
    let screen = screen_at(&on_forest_with_two_points(), far, None, &french());
    assert_eq!(screen.explore_ghosts.len(), 2);
    assert!(
        screen
            .explore_ghosts
            .iter()
            .all(|ghost| ghost.badge.is_none())
    );
}

/// Tuile 3 en 2A, figurine sur sa Colline E : 3A (refusée par la contrainte du dos) et
/// 3A − B (hachuré) touchent la figurine.
fn on_tile_three_east() -> GameState {
    let march = card(March, 1);
    state_with(
        sandbox_deck(),
        &[(TileId::Countryside(3), pos(2, 0))],
        Hex::new(7, -4),
        &[march],
        &[march],
    )
}

// Spec 015 C18
#[test]
fn refused_positions_have_no_ghost() {
    let screen = screen_at(&on_tile_three_east(), camera(), None, &french());
    assert!(screen.explore_ghosts.is_empty());
}

// Spec 015 C19
#[test]
fn hover_explains_exploration() {
    // On survole la case centrale de chaque emplacement.
    let center = |i, j| pos(i, j).center();

    let lines = hover(&on_forest_with_two_points(), center(2, 0));
    assert_eq!(
        lines,
        vec![
            "Emplacement libre".to_owned(),
            "Explorer : 2 points de mouvement".to_owned(),
            "Cliquez pour révéler une tuile de campagne".to_owned(),
        ]
    );

    let lines = hover(&on_forest_with_one_point(), center(2, 0));
    assert_eq!(
        lines,
        vec![
            "Emplacement libre".to_owned(),
            "Explorer : 2 points de mouvement".to_owned(),
            "Il vous manque 1 point(s) de mouvement".to_owned(),
        ]
    );

    let lines = hover(&on_tile_three_east(), center(3, 0));
    assert_eq!(
        lines,
        vec![
            "Emplacement libre".to_owned(),
            "Explorer : 2 points de mouvement".to_owned(),
            "Une tuile de campagne doit toucher deux tuiles posées, ou une tuile qui en touche deux"
                .to_owned(),
        ]
    );

    // Emplacement libre loin de la figurine : seulement son nom.
    let march = card(March, 1);
    let portal = state_with(sandbox_deck(), &[], Hex::ORIGIN, &[march], &[march]);
    assert_eq!(
        hover(&portal, center(0, 2)),
        vec!["Emplacement libre".to_owned()]
    );

    // Tuile royaume sur le dessus.
    let core_deck = vec![
        TileId::Countryside(1),
        TileId::Countryside(2),
        TileId::Core(1),
    ];
    let state = state_with(core_deck.clone(), &[], Hex::new(4, -3), &[march], &[march]);
    assert!(
        hover(&state, center(1, 1)).contains(&"Cliquez pour révéler une tuile royaume".to_owned())
    );
    assert!(
        hover(&state, center(2, 0))
            .contains(&"Une tuile royaume doit toucher au moins deux tuiles posées".to_owned())
    );

    // Pioche vide : tuile écartée.
    let empty_deck = vec![TileId::Countryside(1), TileId::Countryside(2)];
    let state = state_with(
        empty_deck.clone(),
        &[(TileId::Countryside(3), pos(2, 0))],
        Hex::new(2, -3),
        &[march],
        &[march],
    );
    assert!(
        hover(&state, center(1, 1))
            .contains(&"Cliquez pour révéler une tuile écartée, au hasard".to_owned())
    );
    let state = state_with(empty_deck.clone(), &[], Hex::new(2, -3), &[march], &[march]);
    assert!(
        hover(&state, center(1, 1))
            .contains(&"Une tuile écartée doit toucher au moins trois tuiles posées".to_owned())
    );

    // Plus rien à révéler.
    let mut extra: Vec<(TileId, TilePosition)> = (3..=11)
        .map(|n| (TileId::Countryside(n), pos(20 + i32::from(n), 0)))
        .collect();
    extra.extend((1..=4).map(|n| (TileId::Core(n), pos(40 + i32::from(n), 0))));
    let state = state_with(empty_deck, &extra, Hex::new(2, -3), &[march], &[march]);
    assert!(hover(&state, center(1, 1)).contains(&"Plus aucune tuile à révéler".to_owned()));
}

// Spec 015 C20
#[test]
fn clicking_a_ghost_explores() {
    let state = on_forest_with_two_points();
    let index = |position| {
        legal_actions(&state, SOLO)
            .iter()
            .position(|action| *action == Action::Explore { position })
            .expect("exploration légale")
    };
    let click =
        |state: &GameState, hex: Hex| screen_at(state, camera(), Some(hex), &french()).click;
    // N'importe quelle case du fantôme.
    for hex in pos(2, 0).hexes() {
        assert_eq!(
            click(&state, hex),
            Some(Command::Play(index(pos(2, 0)))),
            "{hex:?}"
        );
    }
    assert_eq!(
        click(&state, pos(1, 1).center()),
        Some(Command::Play(index(pos(1, 1))))
    );
    // Fantôme grisé, emplacement refusé, emplacement loin de la figurine, case hachurée.
    assert_eq!(click(&on_forest_with_one_point(), pos(2, 0).center()), None);
    assert_eq!(click(&on_tile_three_east(), pos(3, 0).center()), None);
    assert_eq!(click(&state, pos(0, 2).center()), None);
    assert_eq!(click(&state, pos(2, -1).center()), None);
}

// Spec 015 C21 ; spec 074, C7 : dans la barre de menu, un nombre par pioche (remplace
// `panel_shows_removed_tiles_when_deck_is_empty`).
#[test]
fn bar_shows_removed_tiles_when_decks_are_empty() {
    let catalog = french();
    let decks = |state: &GameState| {
        support::bar_of(&view(state, SOLO), &catalog)
            .tile_decks
            .into_iter()
            .map(|deck| (deck.icon, deck.value, deck.tooltip))
            .collect::<Vec<_>>()
    };
    let march = card(March, 1);
    let empty_deck = vec![TileId::Countryside(1), TileId::Countryside(2)];
    let state = state_with(empty_deck, &[], Hex::ORIGIN, &[march], &[]);
    assert_eq!(
        decks(&state),
        [
            (
                Icon::TileCountryside,
                "9".to_string(),
                "Pioche vide · tuiles de campagne écartées : 9".to_string()
            ),
            (
                Icon::TileCore,
                "4".to_string(),
                "Pioche vide · tuiles royaume écartées : 4".to_string()
            ),
        ]
    );
}

// Spec 015 C22
#[test]
fn every_exploration_key_exists_in_french() {
    let catalog = french();
    let march = card(March, 1);
    let rage = card(Rage, 1);
    let empty_deck = vec![TileId::Countryside(1), TileId::Countryside(2)];
    let core_deck = vec![
        TileId::Countryside(1),
        TileId::Countryside(2),
        TileId::Core(1),
    ];
    let mut nothing_left: Vec<(TileId, TilePosition)> = (3..=11)
        .map(|n| (TileId::Countryside(n), pos(20 + i32::from(n), 0)))
        .collect();
    nothing_left.extend((1..=4).map(|n| (TileId::Core(n), pos(40 + i32::from(n), 0))));
    let mut states = vec![];
    for (deck, extra) in [
        (sandbox_deck(), vec![]),
        (core_deck, vec![]),
        (
            empty_deck.clone(),
            vec![(TileId::Countryside(3), pos(2, 0))],
        ),
        (empty_deck, nothing_left),
    ] {
        for played in [&[][..], &[rage][..], &[march][..]] {
            for figure in [Hex::new(4, -3), Hex::new(2, -3), Hex::ORIGIN] {
                states.push(state_with(
                    deck.clone(),
                    &extra,
                    figure,
                    &[march, rage],
                    played,
                ));
            }
        }
    }
    // Toutes les cases autour du plateau de jeu du bac à sable.
    let hexes: Vec<Hex> = (-3..=3)
        .flat_map(|i| (-3..=3).map(move |j| pos(i, j)))
        .flat_map(|position| position.hexes())
        .collect();
    for state in &states {
        for &hex in &hexes {
            screen_at(state, camera(), Some(hex), &catalog);
        }
        let legal = legal_actions(state, SOLO);
        game_screen(
            &view(state, SOLO),
            ScreenPhase::Playing {
                legal_actions: &legal,
            },
            &ScreenMode::Normal,
            &catalog,
        );
    }
    let english = english();
    for language in [&catalog, &english] {
        for key in [
            "board.hover.free_position",
            "board.hover.click_to_explore.countryside",
            "board.hover.click_to_explore.core",
            "board.hover.click_to_explore.removed",
            "board.hover.explore_limit.countryside",
            "board.hover.explore_limit.core",
            "board.hover.explore_limit.removed",
            "board.hover.nothing_to_reveal",
        ] {
            language.text(key);
        }
        language.format("board.hover.explore_cost", &[("cost", "2")]);
        for key in [
            "menu.tiles.countryside",
            "menu.tiles.core",
            "menu.tiles.countryside_removed",
            "menu.tiles.core_removed",
        ] {
            language.format(key, &[("count", "1")]);
        }
    }
    assert_eq!(catalog.gaps(), vec![]);
    assert_eq!(english.gaps(), vec![]);
}
