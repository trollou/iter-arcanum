//! Scénarios de la spec 015 (exploration).

use arcanum_core::{
    Action, ActionError, BasicAction, CardId, CardPlay, DeedCard, EXPLORE_COST, Event, GameConfig,
    GameState, Hero, Hex, Map, MapShape, NextTile, Placement, PlayerEvent, PlayerId, RngUsage,
    Scenario, SidewaysEffect, TileDeck, TileId, TilePosition, apply, decide, filter_event,
    free_positions_touching, legal_actions, new_game, next_tile, pending_decision, placement,
    removed_tiles, view,
};

const SOLO: PlayerId = PlayerId::SOLO;

use BasicAction::*;

fn card(action: BasicAction, copy: u16) -> CardId {
    CardId::new(DeedCard::Basic(action), copy)
}

fn config(seed: u64) -> GameConfig {
    GameConfig {
        seed,
        scenario: Scenario::sandbox(),
        hero: Hero::Arythea,
        dummy_hero: Default::default(),
    }
}

/// Emplacement de coordonnées `i A + j B` (spec 012).
fn pos(i: i32, j: i32) -> TilePosition {
    TilePosition::from_lattice(i, j)
}

/// Pioche du bac à sable après tirage : campagne 1 à 8, puis trois tuiles royaume.
fn sandbox_deck() -> Vec<TileId> {
    let mut deck: Vec<TileId> = (1..=8).map(TileId::Countryside).collect();
    deck.extend([TileId::Core(2), TileId::Core(6), TileId::Core(3)]);
    deck
}

/// Partie construite par événements comme la mise en place du bac à sable : tuile de départ,
/// pioche `deck`, ses deux premières tuiles révélées en `B` puis `A`, puis `extra` révélées
/// (tuiles posées sans venir du dessus de la pioche), la figurine sur `figure` et `hand` en main.
fn state_with(
    deck: Vec<TileId>,
    extra: &[(TileId, TilePosition)],
    figure: Hex,
    hand: &[CardId],
) -> GameState {
    let mut order = hand.to_vec();
    order.extend(
        Hero::Arythea
            .starting_deck()
            .into_iter()
            .filter(|card| !hand.contains(card)),
    );
    let mut events = vec![
        Event::GameStarted { config: config(0) },
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
    events.iter().fold(GameState::default(), apply)
}

fn sandbox(figure: Hex, hand: &[CardId]) -> GameState {
    state_with(sandbox_deck(), &[], figure, hand)
}

fn basic(card: CardId) -> Action {
    Action::PlayCard {
        card,
        play: CardPlay::Basic {
            die: None,
            gain: None,
            with: None,
            option: 0,
            discard: None,
            mana: None,
        },
    }
}

fn sideways_move(card: CardId) -> Action {
    Action::PlayCard {
        card,
        play: CardPlay::Sideways(SidewaysEffect::Move),
    }
}

fn explore(i: i32, j: i32) -> Action {
    Action::Explore {
        position: pos(i, j),
    }
}

fn done_with(state: &GameState, action: Action, seed: u64) -> GameState {
    let events = decide(state, SOLO, &action, &mut config(seed).rngs())
        .unwrap_or_else(|error| panic!("{action:?} doit être légale : {error:?}"));
    events.iter().fold(state.clone(), apply)
}

fn done(state: &GameState, action: Action) -> GameState {
    done_with(state, action, 0)
}

fn refused(state: &GameState, action: Action) {
    assert_eq!(
        decide(state, SOLO, &action, &mut config(0).rngs()),
        Err(ActionError::NotLegal),
        "{action:?}"
    );
}

fn explorations(state: &GameState) -> Vec<Action> {
    legal_actions(state, SOLO)
        .into_iter()
        .filter(|action| matches!(action, Action::Explore { .. }))
        .collect()
}

/// Plateau de jeu du bac à sable après la mise en place.
fn setup_map() -> Map {
    let mut map = Map::new();
    for (tile, position) in [
        (TileId::StartingA, TilePosition::ORIGIN),
        (TileId::Countryside(1), pos(0, 1)),
        (TileId::Countryside(2), pos(1, 0)),
    ] {
        map.place(tile, position).expect("emplacement libre");
    }
    map
}

const WEDGE: MapShape = MapShape::Wedge;

// --- Règles de pose ---

// Spec 015 C1 — MKUE p. 7, 5.a et 5.e
#[test]
fn explorable_positions_after_sandbox_setup() {
    let map = setup_map();
    for (i, j) in [(1, 1), (0, 2), (2, 0)] {
        assert_eq!(
            placement(&map, WEDGE, pos(i, j), NextTile::Countryside),
            Placement::Allowed,
            "campagne en ({i}, {j})"
        );
    }
    assert_eq!(
        placement(&map, WEDGE, pos(1, 1), NextTile::Core),
        Placement::Allowed
    );
    for (i, j) in [(0, 2), (2, 0)] {
        assert_eq!(
            placement(&map, WEDGE, pos(i, j), NextTile::Core),
            Placement::TooFewNeighbors,
            "royaume en ({i}, {j})"
        );
    }
}

// Spec 015 C2 — MKUE p. 7, 5.e : une tuile adjacente qui en borde deux autres
#[test]
fn countryside_needs_two_neighbors_or_a_busy_one() {
    let mut map = setup_map();
    map.place(TileId::Countryside(3), pos(2, 0)).expect("libre");
    for (i, j) in [(3, 0), (2, 1)] {
        assert_eq!(
            placement(&map, WEDGE, pos(i, j), NextTile::Countryside),
            Placement::TooFewNeighbors,
            "({i}, {j})"
        );
    }
    assert_eq!(
        placement(&map, WEDGE, pos(1, 1), NextTile::RemovedCountryside),
        Placement::Allowed
    );
}

// Spec 015 C3 — MKUE p. 7, 5.e : une tuile écartée bouche un trou
#[test]
fn removed_tile_fills_holes_only() {
    let map = setup_map();
    for next in [NextTile::RemovedCountryside, NextTile::RemovedCore] {
        assert_eq!(
            placement(&map, WEDGE, pos(0, 2), next),
            Placement::TooFewNeighbors
        );
        assert_eq!(
            placement(&map, WEDGE, pos(1, 1), next),
            Placement::TooFewNeighbors
        );
    }
}

// Spec 015 C4 — MKUE p. 7, 5.e ; p. 18
#[test]
fn removed_tiles_are_neither_placed_nor_in_the_deck() {
    let config = config(3);
    let state = new_game(&config, &mut config.rngs())
        .iter()
        .fold(GameState::default(), apply);
    let removed = removed_tiles(state.map(), state.tile_deck());
    assert_eq!(removed.len(), 3 + 2 + 3);
    for tile in [9, 10, 11].map(TileId::Countryside) {
        assert!(removed.contains(&tile), "{tile:?}");
    }
    assert_eq!(removed.iter().filter(|t| t.is_city()).count(), 3);
    for tile in &removed {
        assert!(!state.tile_deck().tiles().contains(tile));
        assert!(state.map().tiles().all(|(_, placed)| placed != *tile));
        assert!(!matches!(tile, TileId::StartingA | TileId::StartingB));
    }
}

// Spec 015 C5 — MKUE p. 7, 5.c et 5.e
#[test]
fn next_tile_follows_deck_then_removed_tiles() {
    let map = setup_map();
    assert_eq!(
        next_tile(&map, &TileDeck::new(vec![TileId::Countryside(3)])),
        NextTile::Countryside
    );
    assert_eq!(
        next_tile(&map, &TileDeck::new(vec![TileId::Core(1)])),
        NextTile::Core
    );
    assert_eq!(
        next_tile(&map, &TileDeck::default()),
        NextTile::RemovedCountryside
    );

    // Toutes les tuiles de campagne posées, n'importe où : il reste des tuiles royaume sans cité.
    let mut full = Map::new();
    let mut lattice = (0..).map(|i| pos(i, 0));
    let mut put = |map: &mut Map, tile| {
        let position = lattice.next().expect("infini");
        map.place(tile, position).expect("libre");
    };
    for n in 1..=11 {
        put(&mut full, TileId::Countryside(n));
    }
    assert_eq!(
        next_tile(&full, &TileDeck::default()),
        NextTile::RemovedCore
    );
    for n in 1..=4 {
        put(&mut full, TileId::Core(n));
    }
    // Seules restent les tuiles royaume de cité, qui ne reviennent jamais.
    assert_eq!(next_tile(&full, &TileDeck::default()), NextTile::None);
    assert_eq!(
        placement(&full, WEDGE, pos(0, 1), NextTile::None),
        Placement::NothingToReveal
    );
}

// --- Explorer ---

// Spec 015 C6 — MKUE p. 7, 5.a
#[test]
fn hero_next_to_a_free_position_can_explore() {
    let march = card(March, 1);
    let state = done(&sandbox(Hex::new(2, -3), &[march]), basic(march));
    assert_eq!(
        free_positions_touching(state.map(), Hex::new(2, -3)),
        vec![pos(1, 1)]
    );
    assert_eq!(explorations(&state), vec![explore(1, 1)]);
}

// Spec 015 C7 — MKUE p. 7, 5.b et 5.c
#[test]
fn exploring_reveals_the_top_tile_for_two_points() {
    let march = card(March, 1);
    let state = done(&sandbox(Hex::new(2, -3), &[march]), basic(march));
    let deck_before = state.tile_deck().len();
    let events = decide(&state, SOLO, &explore(1, 1), &mut config(0).rngs()).expect("légale");
    assert_eq!(
        events,
        vec![
            Event::Explored {
                player: SOLO,
                position: pos(1, 1),
                cost: EXPLORE_COST,
            },
            Event::TileRevealed {
                tile: TileId::Countryside(3),
                position: pos(1, 1),
            },
        ]
    );
    assert_eq!(EXPLORE_COST, 2);
    let after = events.iter().fold(state, apply);
    assert!(
        after
            .map()
            .tiles()
            .any(|tile| tile == (pos(1, 1), TileId::Countryside(3)))
    );
    assert_eq!(after.tile_deck().len(), deck_before - 1);
    assert_eq!(after.figure(SOLO), Some(Hex::new(2, -3)));
    assert_eq!(after.move_points(SOLO), 0);
}

// Spec 015 C8 — MKUE p. 7, 5.c : on annonce l'emplacement
#[test]
fn hero_bordering_two_positions_chooses_one() {
    let march = card(March, 1);
    let state = done(&sandbox(Hex::new(4, -3), &[march]), basic(march));
    assert_eq!(explorations(&state), vec![explore(1, 1), explore(2, 0)]);
    let after = done(&state, explore(2, 0));
    let placed: Vec<_> = after.map().tiles().collect();
    assert!(placed.contains(&(pos(2, 0), TileId::Countryside(3))));
    assert!(placed.iter().all(|(position, _)| *position != pos(1, 1)));
}

// Spec 015 C9 — MKUE p. 7, 5.a ; p. 14
#[test]
fn positions_behind_the_coastline_cannot_be_explored() {
    let march = card(March, 1);
    let state = done(&sandbox(Hex::new(4, -2), &[march]), basic(march));
    assert_eq!(
        free_positions_touching(state.map(), Hex::new(4, -2)),
        vec![pos(2, -1), pos(2, 0)]
    );
    assert_eq!(explorations(&state), vec![explore(2, 0)]);
    assert_eq!(
        placement(state.map(), WEDGE, pos(2, -1), NextTile::Countryside),
        Placement::BehindCoastline
    );
}

// Spec 015 C10 — MKUE p. 7, 6.a et 6.b
#[test]
fn exploring_and_moving_alternate() {
    let (march1, march2, stamina) = (card(March, 1), card(March, 2), card(Stamina, 1));
    let state = sandbox(Hex::new(4, -3), &[march1, march2, stamina]);
    let state = done(&done(&state, basic(march1)), explore(2, 0));
    assert_eq!(state.move_points(SOLO), 0);
    // Plaine O de la tuile 3, révélée en 2A.
    let state = done(
        &done(&state, basic(march2)),
        Action::MoveHero {
            to: Hex::new(5, -4),
        },
    );
    assert_eq!(state.move_points(SOLO), 0);
    let state = done(&done(&state, basic(stamina)), explore(1, 1));
    assert!(
        state
            .map()
            .tiles()
            .any(|tile| tile == (pos(1, 1), TileId::Countryside(4)))
    );
    assert_eq!(state.figure(SOLO), Some(Hex::new(5, -4)));
    assert_eq!(state.move_points(SOLO), 0);
}

// Spec 015 C11 — MKUE p. 7, 5.e
#[test]
fn core_tile_needs_two_neighbors() {
    let march = card(March, 1);
    let deck = vec![
        TileId::Countryside(1),
        TileId::Countryside(2),
        TileId::Core(1),
        TileId::Core(5),
    ];
    let state = done(
        &state_with(deck, &[], Hex::new(4, -3), &[march]),
        basic(march),
    );
    assert_eq!(view(&state, SOLO).next_tile, NextTile::Core);
    assert_eq!(explorations(&state), vec![explore(1, 1)]);
    let state = done(&state, explore(1, 1));
    assert!(
        state
            .map()
            .tiles()
            .any(|tile| tile == (pos(1, 1), TileId::Core(1)))
    );
}

/// Pioche vide (seules les deux premières tuiles), tuile 3 posée en 2A : `A + B` touche trois
/// tuiles, la figurine est sur la Plaine E de la tuile 1.
fn empty_deck_state() -> GameState {
    let march = card(March, 1);
    let deck = vec![TileId::Countryside(1), TileId::Countryside(2)];
    let state = state_with(
        deck,
        &[(TileId::Countryside(3), pos(2, 0))],
        Hex::new(2, -3),
        &[march],
    );
    done(&state, basic(march))
}

fn revealed_tile(state: &GameState, seed: u64) -> TileId {
    let events = decide(state, SOLO, &explore(1, 1), &mut config(seed).rngs()).expect("légale");
    match events.as_slice() {
        [
            Event::Explored { .. },
            Event::TileRevealed { tile, position },
        ] => {
            assert_eq!(*position, pos(1, 1));
            *tile
        }
        other => panic!("événements inattendus : {other:?}"),
    }
}

// Spec 015 C12 — MKUE p. 7, 5.e
#[test]
fn empty_deck_reveals_a_random_removed_tile() {
    let state = empty_deck_state();
    assert!(state.tile_deck().is_empty());
    assert_eq!(view(&state, SOLO).next_tile, NextTile::RemovedCountryside);
    assert_eq!(explorations(&state), vec![explore(1, 1)]);

    let tile = revealed_tile(&state, 5);
    assert!(
        matches!(tile, TileId::Countryside(n) if (4..=11).contains(&n)),
        "{tile:?}"
    );
    assert_eq!(revealed_tile(&state, 5), tile);
    let drawn: std::collections::BTreeSet<TileId> =
        (0..20).map(|seed| revealed_tile(&state, seed)).collect();
    assert!(drawn.len() > 1, "le tirage dépend de la graine");

    let after = done_with(&state, explore(1, 1), 5);
    assert!(
        after
            .map()
            .tiles()
            .any(|placed| placed == (pos(1, 1), tile))
    );
    assert_eq!(after.move_points(SOLO), 0);
}

// Spec 015 C13
#[test]
fn decision_lasts_while_exploring_is_possible() {
    // Forêt N-O de la tuile 1 : ses voisines coûtent 3 ou sont un lac ; elle borde 2B.
    let march = card(March, 1);
    let state = done(&sandbox(Hex::new(1, -4), &[march]), basic(march));
    assert!(state.hand(SOLO).is_empty());
    assert_eq!(state.move_points(SOLO), 2);
    assert!(pending_decision(&state).is_some());
    let end_turn = Action::EndTurn {
        discard: vec![],
        reclaim: None,
        glade: None,
        steady_tempo: false,
    };
    assert_eq!(
        legal_actions(&state, SOLO),
        vec![explore(0, 2), end_turn.clone()]
    );

    // Spec 016 : plus de point, il reste à finir le tour.
    let state = done(&state, explore(0, 2));
    assert_eq!(legal_actions(&state, SOLO), vec![end_turn]);
}

// Spec 015 C14
#[test]
fn legal_actions_list_cards_moves_then_explorations() {
    let (march1, march2, rage) = (card(March, 1), card(March, 2), card(Rage, 1));
    let state = sandbox(Hex::new(4, -3), &[march1, march2, rage]);
    let state = done(&done(&state, basic(march1)), basic(march2));
    assert_eq!(state.move_points(SOLO), 4);

    let sideways = |card, effect| Action::PlayCard {
        card,
        play: CardPlay::Sideways(effect),
    };
    // Rage n'a pas d'effet de base jouable hors combat (spec 013).
    let expected = vec![
        sideways(rage, SidewaysEffect::Move),
        sideways(rage, SidewaysEffect::Influence),
        // Voisines posées, par direction : O Colline (3), S-O Colline (3), S-E Plaine (2).
        Action::MoveHero {
            to: Hex::new(3, -3),
        },
        Action::MoveHero {
            to: Hex::new(3, -2),
        },
        Action::MoveHero {
            to: Hex::new(4, -2),
        },
        // Spec 036 : le trajet suivi vers la seule case lointaine qu'on peut payer.
        Action::FollowRoute {
            to: Hex::new(3, -1),
        },
        explore(1, 1),
        explore(2, 0),
        // Spec 016 : puis les fins de tour.
        Action::EndTurn {
            discard: vec![],
            reclaim: None,
            glade: None,
            steady_tempo: false,
        },
        Action::EndTurn {
            discard: vec![rage],
            reclaim: None,
            glade: None,
            steady_tempo: false,
        },
    ];
    assert_eq!(legal_actions(&state, SOLO), expected);
    for action in expected {
        assert!(
            decide(&state, SOLO, &action, &mut config(0).rngs()).is_ok(),
            "{action:?}"
        );
    }
}

// --- Information cachée et journal ---

// Spec 015 C15
#[test]
fn exploration_reveals_only_the_revealed_tile() {
    let config = config(2);
    let state = new_game(&config, &mut config.rngs())
        .iter()
        .fold(GameState::default(), apply);
    let player_view = view(&state, SOLO);
    assert_eq!(player_view.next_tile, NextTile::Countryside);
    assert_eq!(player_view.removed_tiles.countryside, 3);
    assert_eq!(player_view.removed_tiles.core, 2);

    let explored = Event::Explored {
        player: SOLO,
        position: pos(1, 1),
        cost: 2,
    };
    assert_eq!(
        filter_event(&explored, SOLO),
        PlayerEvent::Explored {
            player: SOLO,
            position: pos(1, 1),
            cost: 2,
        }
    );
    let revealed = Event::TileRevealed {
        tile: TileId::Countryside(3),
        position: pos(1, 1),
    };
    assert_eq!(
        filter_event(&revealed, SOLO),
        PlayerEvent::TileRevealed {
            tile: TileId::Countryside(3),
            position: pos(1, 1),
        }
    );
}

// Spec 015 C16
#[test]
fn replaying_log_rebuilds_explorations() {
    // Une tuile écartée tirée au hasard en A + B, puis on rejoue les événements.
    let start = empty_deck_state();
    let events = decide(&start, SOLO, &explore(1, 1), &mut config(9).rngs()).expect("légale");
    let state = events.iter().fold(start.clone(), apply);
    let replayed = events.iter().fold(start, apply);
    assert_eq!(replayed, state);
}

// --- Cas limites et refus ---

// Spec 015 L1 — MKUE p. 7, 5.a
#[test]
fn exploring_far_from_the_hero_is_refused() {
    let march = card(March, 1);
    let state = done(&sandbox(Hex::ORIGIN, &[march]), basic(march));
    assert!(explorations(&state).is_empty());
    refused(&state, explore(0, 2));
}

// Spec 015 L2 — MKUE p. 7, 5.a
#[test]
fn exploring_an_occupied_or_forbidden_position_is_refused() {
    let march = card(March, 1);
    let state = done(&sandbox(Hex::new(1, 0), &[march]), basic(march));
    refused(&state, explore(0, 1));
    refused(&state, explore(1, -1));
    assert_eq!(
        placement(state.map(), WEDGE, pos(0, 1), NextTile::Countryside),
        Placement::Occupied
    );
}

// Spec 015 L3 — MKUE p. 7, 5.b
#[test]
fn exploring_without_two_points_is_refused() {
    let rage = card(Rage, 1);
    let state = done(&sandbox(Hex::new(2, -3), &[rage]), sideways_move(rage));
    assert_eq!(state.move_points(SOLO), 1);
    assert!(explorations(&state).is_empty());
    refused(&state, explore(1, 1));
}

// Spec 015 L4 — MKUE p. 7, 5.e
#[test]
fn exploring_against_placement_limits_is_refused() {
    let march = card(March, 1);
    let state = state_with(
        sandbox_deck(),
        &[(TileId::Countryside(3), pos(2, 0))],
        Hex::new(7, -4),
        &[march],
    );
    let state = done(&state, basic(march));
    assert!(explorations(&state).is_empty());
    refused(&state, explore(3, 0));
}

// Spec 015 L5 — MKUE p. 7, 5.e
#[test]
fn nothing_left_to_reveal_stops_exploration() {
    let march = card(March, 1);
    // Toutes les tuiles de campagne et royaume sans cité posées, loin, sauf les deux de la
    // mise en place ; la tuile 3 en 2A pour qu'A + B touche trois tuiles.
    let mut extra = vec![(TileId::Countryside(3), pos(2, 0))];
    extra.extend((4..=11).map(|n| (TileId::Countryside(n), pos(20 + i32::from(n), 0))));
    extra.extend((1..=4).map(|n| (TileId::Core(n), pos(40 + i32::from(n), 0))));
    let deck = vec![TileId::Countryside(1), TileId::Countryside(2)];
    let state = done(
        &state_with(deck, &extra, Hex::new(2, -3), &[march]),
        basic(march),
    );
    assert_eq!(view(&state, SOLO).next_tile, NextTile::None);
    assert!(explorations(&state).is_empty());
    refused(&state, explore(1, 1));
}

// Spec 015 L6
#[test]
fn invalid_exploration_events_change_nothing() {
    let rage = card(Rage, 1);
    let state = done(&sandbox(Hex::new(2, -3), &[rage]), sideways_move(rage));
    let too_expensive = Event::Explored {
        player: SOLO,
        position: pos(1, 1),
        cost: 2,
    };
    assert_eq!(apply(state.clone(), &too_expensive), state);
    let occupied = Event::TileRevealed {
        tile: TileId::Countryside(3),
        position: pos(0, 1),
    };
    assert_eq!(apply(state.clone(), &occupied), state);
}

// Spec 038 C7 — MKUE p. 7, 5.e : la tuile écartée est tirée dans le générateur de la pioche de
// tuiles ; la Source et le deck n'avancent pas.
#[test]
fn removed_tile_is_drawn_from_the_tiles_generator() {
    let state = empty_deck_state();
    let mut rngs = config(9).rngs();
    let events = decide(&state, SOLO, &explore(1, 1), &mut rngs).expect("légale");

    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::TileRevealed { .. }))
    );
    assert_eq!(
        rngs.positions().into_keys().collect::<Vec<_>>(),
        [RngUsage::Tiles]
    );
    // Avancer les autres usages ne change pas la tuile tirée.
    let mut shifted = config(9).rngs();
    for usage in [RngUsage::Source, RngUsage::Deck(SOLO)] {
        for _ in 0..20 {
            rand::Rng::next_u64(shifted.rng(usage));
        }
    }
    assert_eq!(
        decide(&state, SOLO, &explore(1, 1), &mut shifted).expect("légale"),
        events
    );
}
