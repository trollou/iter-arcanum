//! Scénarios de la spec 017 (cases atteignables).

use arcanum_core::{
    Action, BasicAction, CardId, CardPlay, DeedCard, Direction, Event, GameConfig, GameState, Hero,
    Hex, Map, PlayerId, Route, Scenario, SidewaysEffect, TileId, TilePosition, TimeOfDay, apply,
    decide, legal_actions, routes,
};

const SOLO: PlayerId = PlayerId::SOLO;

use BasicAction::*;
use Direction::*;

/// Emplacement `A` du bac à sable (tuile de campagne 2), centre (3, −2).
fn a() -> TilePosition {
    TilePosition::from_lattice(1, 0)
}

/// Emplacement `B` du bac à sable (tuile de campagne 1), centre (1, −3).
fn b() -> TilePosition {
    TilePosition::from_lattice(0, 1)
}

fn map_of(tiles: &[(TileId, TilePosition)]) -> Map {
    let mut map = Map::new();
    for &(tile, position) in tiles {
        map.place(tile, position).expect("emplacement libre");
    }
    map
}

/// Le bac à sable à la mise en place : départ A, campagne 1 en B, campagne 2 en A.
fn sandbox_tiles() -> Vec<(TileId, TilePosition)> {
    vec![
        (TileId::StartingA, TilePosition::ORIGIN),
        (TileId::Countryside(1), b()),
        (TileId::Countryside(2), a()),
    ]
}

fn sandbox() -> Map {
    map_of(&sandbox_tiles())
}

/// Cases traversées en partant du portail, direction par direction.
fn walk(directions: &[Direction]) -> Vec<Hex> {
    let mut hex = Hex::ORIGIN;
    directions
        .iter()
        .map(|direction| {
            hex = hex.neighbor(*direction);
            hex
        })
        .collect()
}

fn route_to(routes: &[Route], to: Hex) -> Option<&Route> {
    routes.iter().find(|route| route.to == to)
}

fn day_routes(map: &Map) -> Vec<Route> {
    routes(map, Hex::ORIGIN, TimeOfDay::Day)
}

// --- Trajets ---

// Spec 017 C1 — MKUE p. 7, 3 et 6
#[test]
fn routes_follow_the_cheapest_spaces() {
    let routes = day_routes(&sandbox());
    let expected: [(Hex, &[Direction], u32); 9] = [
        (Hex::new(1, 0), &[E], 2),
        (Hex::new(0, -1), &[NW], 2),
        (Hex::new(1, -1), &[NE], 3),
        (Hex::new(0, -2), &[NW, NW], 4),
        (Hex::new(1, -2), &[NW, NE], 4),
        (Hex::new(2, -2), &[NE, NE], 5),
        (Hex::new(2, -1), &[E, NE], 5),
        (Hex::new(2, -3), &[NW, NE, NE], 6),
        (Hex::new(1, -3), &[NW, NE, NW], 7),
    ];
    for (to, directions, cost) in expected {
        let route = route_to(&routes, to).unwrap_or_else(|| panic!("pas de trajet vers {to:?}"));
        assert_eq!(route.steps, walk(directions), "{to:?}");
        assert_eq!(route.cost, cost, "{to:?}");
        assert_eq!(route.steps.last(), Some(&to));
    }
}

// Spec 017 C2 — MKUE p. 7, 3.a
#[test]
fn routes_avoid_inaccessible_spaces() {
    let map = sandbox();
    let routes = day_routes(&map);
    let inaccessible = [
        Hex::new(-1, 0),
        Hex::new(-1, 1),
        Hex::new(0, 1),
        Hex::new(2, -4),
    ];
    for hex in inaccessible {
        assert!(route_to(&routes, hex).is_none(), "{hex:?}");
    }
    assert!(route_to(&routes, Hex::ORIGIN).is_none());
    for route in &routes {
        for step in &route.steps {
            assert!(!inaccessible.contains(step), "{route:?}");
        }
    }
}

// Spec 017 C3 — MKUE p. 7, 4.a ; spec 014 Q2
#[test]
fn routes_avoid_fortified_sites() {
    let map = map_of(&[
        (TileId::StartingA, TilePosition::ORIGIN),
        (TileId::Countryside(1), b()),
        (TileId::Countryside(3), a()),
    ]);
    let keep = Hex::new(4, -3);
    let routes = day_routes(&map);
    assert!(route_to(&routes, keep).is_none());
    assert!(routes.iter().all(|route| !route.steps.contains(&keep)));
}

// Spec 017 C4 — MKUE p. 7, 4.b et 4.d
#[test]
fn routes_cross_non_fortified_sites_and_printed_enemies() {
    let routes = day_routes(&sandbox());
    let orcs = route_to(&routes, Hex::new(3, -3)).expect("colline aux orques");
    assert_eq!(orcs.cost, 8);
    let village = route_to(&routes, Hex::new(4, -2)).expect("village de la tuile 2");
    assert_eq!(village.steps, walk(&[E, NE, E, NE]));
    assert_eq!(village.cost, 9);
    let forest = route_to(&routes, Hex::new(1, -4)).expect("forêt aux orques");
    assert_eq!(forest.steps, walk(&[NW, NE, NW, NW]));
    assert_eq!(forest.cost, 10);
}

// Spec 017 C5
#[test]
fn equal_routes_are_broken_by_direction_order() {
    let routes = day_routes(&sandbox());
    let glade = route_to(&routes, Hex::new(1, -3)).expect("clairière magique");
    // NW, NW, NE coûte aussi 7 : NE vient avant NW à la deuxième direction.
    assert_eq!(glade.steps, walk(&[NW, NE, NW]));
    assert_eq!(glade.cost, 7);
}

// Spec 017 C6
#[test]
fn equal_cost_prefers_fewer_spaces() {
    // Tuiles de campagne 1 et 2 échangées : vers (3, −3), des trajets de 3 et de 4 cases coûtent 9.
    let map = map_of(&[
        (TileId::StartingA, TilePosition::ORIGIN),
        (TileId::Countryside(1), a()),
        (TileId::Countryside(2), b()),
    ]);
    let routes = day_routes(&map);
    let route = route_to(&routes, Hex::new(3, -3)).expect("trajet");
    assert_eq!(route.cost, 9);
    assert_eq!(route.steps.len(), 3, "{route:?}");
}

// Spec 017 C7 — MKUE p. 7, 3.a, note
#[test]
fn night_costs_change_the_route() {
    let routes = routes(&sandbox(), Hex::ORIGIN, TimeOfDay::Night);
    let plains = route_to(&routes, Hex::new(2, -2)).expect("plaine");
    assert_eq!(plains.steps, walk(&[NW, NE, E]));
    assert_eq!(plains.cost, 6);
}

// Spec 017 C8
#[test]
fn neighbor_route_is_the_direct_step() {
    let map = sandbox();
    for from in [
        Hex::ORIGIN,
        Hex::new(1, -2),
        Hex::new(2, -2),
        Hex::new(3, -1),
    ] {
        let routes = routes(&map, from, TimeOfDay::Day);
        for neighbor in from.neighbors() {
            if let Some(route) = route_to(&routes, neighbor) {
                assert_eq!(route.steps, vec![neighbor], "{from:?} → {neighbor:?}");
            }
        }
    }
}

// Spec 017 C9 — MKUE p. 7, 5
#[test]
fn routes_only_cross_placed_tiles() {
    let mut map = map_of(&[(TileId::StartingA, TilePosition::ORIGIN)]);
    let before = day_routes(&map);
    assert!(before.iter().all(|route| map.space(route.to).is_some()));
    assert!(route_to(&before, Hex::new(1, -2)).is_none());

    map.place(TileId::Countryside(1), b())
        .expect("emplacement libre");
    let after = day_routes(&map);
    assert!(route_to(&after, Hex::new(1, -2)).is_some());
    assert!(route_to(&after, Hex::new(1, -3)).is_some());
}

// Spec 017 L1 : une case de départ sans tuile n'a de voisines que sur le plateau de jeu.
#[test]
fn routes_never_leave_the_map() {
    let map = sandbox();
    for route in day_routes(&map) {
        for step in &route.steps {
            assert!(map.space(*step).is_some(), "{route:?}");
        }
    }
}

// --- Cohérence avec le moteur ---

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

/// Bac à sable construit par événements, la figurine au portail, `hand` en main.
fn sandbox_state(hand: &[CardId]) -> GameState {
    let mut order = hand.to_vec();
    order.extend(
        Hero::Arythea
            .starting_deck()
            .into_iter()
            .filter(|card| !hand.contains(card)),
    );
    let mut events = vec![Event::GameStarted { config: config() }];
    events.extend(
        sandbox_tiles()
            .into_iter()
            .map(|(tile, position)| Event::TilePlaced { tile, position }),
    );
    events.extend([
        Event::HeroFigurePlaced {
            player: SOLO,
            hex: Hex::ORIGIN,
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

fn done(state: &GameState, action: Action) -> GameState {
    let events = decide(state, SOLO, &action, &mut config().rngs())
        .unwrap_or_else(|error| panic!("{action:?} doit être légale : {error:?}"));
    events.iter().fold(state.clone(), apply)
}

fn play(state: &GameState, card: CardId, play: CardPlay) -> GameState {
    done(state, Action::PlayCard { card, play })
}

fn basic() -> CardPlay {
    CardPlay::Basic {
        die: None,
        gain: None,
        with: None,
        option: 0,
        discard: None,
        mana: None,
    }
}

fn legal_moves(state: &GameState) -> Vec<Hex> {
    legal_actions(state, SOLO)
        .into_iter()
        .filter_map(|action| match action {
            Action::MoveHero { to } => Some(to),
            _ => None,
        })
        .collect()
}

fn affordable(state: &GameState) -> Vec<Route> {
    let figure = state.figure(SOLO).expect("figurine");
    routes(state.map(), figure, state.time_of_day())
        .into_iter()
        .filter(|route| route.cost <= state.move_points(SOLO))
        .collect()
}

// Spec 017 C10 — MKUE p. 7, 3
#[test]
fn affordable_routes_are_playable_step_by_step() {
    let hand = [card(March, 1), card(March, 2), card(Swiftness, 1)];
    let mut states = vec![sandbox_state(&hand)];
    let after_march = play(&states[0], hand[0], basic());
    let after_two = play(&after_march, hand[1], basic());
    let after_three = play(
        &after_two,
        hand[2],
        CardPlay::Sideways(SidewaysEffect::Move),
    );
    states.extend([after_march, after_two, after_three.clone()]);
    // Une position de départ ailleurs que sur le portail.
    states.push(done(
        &after_three,
        Action::MoveHero {
            to: Hex::new(0, -1),
        },
    ));

    for state in &states {
        let affordable = affordable(state);
        let mut first_steps: Vec<Hex> = affordable
            .iter()
            .filter(|route| route.steps.len() == 1)
            .map(|route| route.to)
            .collect();
        let mut legal = legal_moves(state);
        first_steps.sort();
        legal.sort();
        assert_eq!(first_steps, legal, "points : {}", state.move_points(SOLO));

        for route in &affordable {
            let mut walked = state.clone();
            for step in &route.steps {
                walked = done(&walked, Action::MoveHero { to: *step });
            }
            assert_eq!(walked.figure(SOLO), Some(route.to));
            assert_eq!(
                walked.move_points(SOLO),
                state.move_points(SOLO) - route.cost,
                "{route:?}"
            );
        }
    }
    assert_eq!(after_three.move_points(SOLO), 5);
}
