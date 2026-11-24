//! Scénarios de la spec 036 (trajet en un clic).

use arcanum_core::{
    Action, BasicAction, Event, GameConfig, GameRunner, GameState, Hero, Hex, Scenario, TimeOfDay,
    apply, decide, legal_actions, routes_avoiding,
};

mod arene;
use arene::*;

use BasicAction::*;

fn config() -> GameConfig {
    GameConfig {
        seed: 0,
        scenario: Scenario::sandbox(),
        hero: Hero::Arythea,
        dummy_hero: Default::default(),
    }
}

/// Le bac à sable des tests de déplacement (`Sandbox::moving`), figurine sur `figure` (aucune si
/// `None`), de jour ou de nuit.
fn setup(figure: Option<Hex>, time: TimeOfDay) -> Vec<Event> {
    Sandbox {
        time,
        ..Sandbox::moving(figure)
    }
    .events()
}

/// Les cartes qui donnent `points` points de mouvement : Marche (2), Marche (2), puis Endurance
/// (2) ou Improvisation de côté (1).
fn cards_for(points: u32) -> Vec<Action> {
    let march = [basic(card(March, 1)), basic(card(March, 2))];
    let extra: Vec<Action> = match points {
        4 => vec![],
        5 => vec![sideways(card(Improvisation, 1))],
        6 => vec![basic(card(Stamina, 1))],
        7 => vec![basic(card(Stamina, 1)), sideways(card(Improvisation, 1))],
        _ => panic!("{points} points non prévus"),
    };
    march.into_iter().chain(extra).collect()
}

fn decided(state: &GameState, action: &Action) -> Vec<Event> {
    decide(state, SOLO, action, &mut config().rngs())
        .unwrap_or_else(|error| panic!("{action:?} doit être légale : {error:?}"))
}

/// Bac à sable, figurine sur `figure`, avec `points` points de mouvement.
fn at(figure: Hex, points: u32, time: TimeOfDay) -> GameState {
    let state = setup(Some(figure), time)
        .iter()
        .fold(GameState::default(), apply);
    cards_for(points)
        .into_iter()
        .fold(state, |s, a| done(&s, a))
}

fn day(figure: Hex, points: u32) -> GameState {
    at(figure, points, TimeOfDay::Day)
}

fn follow(to: Hex) -> Action {
    Action::FollowRoute { to }
}

/// Les pas `(from, to, cost)` des `HeroMoved` d'une suite d'événements.
fn steps(events: &[Event]) -> Vec<(Hex, Hex, u8)> {
    events
        .iter()
        .filter_map(|event| match event {
            Event::HeroMoved { from, to, cost, .. } => Some((*from, *to, *cost)),
            _ => None,
        })
        .collect()
}

fn follow_targets(state: &GameState) -> Vec<Hex> {
    legal_actions(state, SOLO)
        .into_iter()
        .filter_map(|action| match action {
            Action::FollowRoute { to } => Some(to),
            _ => None,
        })
        .collect()
}

// C1 — MKUE p. 7, 3
#[test]
fn following_a_route_moves_space_by_space() {
    let state = day(Hex::ORIGIN, 5);
    assert!(legal_actions(&state, SOLO).contains(&follow(PLAIN_W)));
    let events = decided(&state, &follow(PLAIN_W));
    assert_eq!(events.len(), 2, "{events:?}");
    assert_eq!(
        steps(&events),
        [
            (Hex::ORIGIN, Hex::new(1, -1), 3),
            (Hex::new(1, -1), PLAIN_W, 2)
        ]
    );
    let after = events.iter().fold(state, apply);
    assert_eq!(after.figure(SOLO), Some(PLAIN_W));
    assert_eq!(after.move_points(SOLO), 0);
}

// C2 — MKUE p. 7, 3 et 6
#[test]
fn following_a_three_space_route() {
    let state = day(Hex::ORIGIN, 6);
    let events = decided(&state, &follow(VILLAGE));
    assert_eq!(
        steps(&events),
        [
            (Hex::ORIGIN, Hex::new(0, -1), 2),
            (Hex::new(0, -1), PLAIN_SW, 2),
            (PLAIN_SW, VILLAGE, 2),
        ]
    );
    let after = events.iter().fold(state, apply);
    assert_eq!(after.figure(SOLO), Some(VILLAGE));
    assert_eq!(after.move_points(SOLO), 0);
}

// C3 — MKUE p. 7, 4.c ; spec 030, C19
#[test]
fn followed_route_never_provokes() {
    let state = day(VILLAGE, 7);
    let events = decided(&state, &follow(GREEN_MINE));
    assert_eq!(
        steps(&events),
        [
            (VILLAGE, PLAIN_SW, 2),
            (PLAIN_SW, PLAIN_W, 2),
            (PLAIN_W, GREEN_MINE, 3),
        ]
    );
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::EnemiesProvoked { .. }))
    );
    let after = events.iter().fold(state, apply);
    assert!(!after.movement_ended(SOLO));
    assert_eq!(after.move_points(SOLO), 0);
}

// C4 — MKUE p. 7, 4.c et 6
#[test]
fn route_that_would_need_a_provocation_is_not_offered() {
    let state = day(VILLAGE, 5);
    assert!(!follow_targets(&state).contains(&GREEN_MINE));
    refused(&state, follow(GREEN_MINE));
}

// C5 — spec 017, C11
#[test]
fn follow_route_actions_list_far_reachable_spaces() {
    let state = day(Hex::ORIGIN, 5);
    assert_eq!(
        follow_targets(&state),
        [PLAIN_SW, Hex::new(0, -2), GREEN_MINE, PLAIN_W]
    );
    let legal = legal_actions(&state, SOLO);
    let last_move = legal
        .iter()
        .rposition(|action| matches!(action, Action::MoveHero { .. }))
        .expect("des pas légaux");
    let follows: Vec<usize> = legal
        .iter()
        .enumerate()
        .filter(|(_, action)| matches!(action, Action::FollowRoute { .. }))
        .map(|(index, _)| index)
        .collect();
    assert_eq!(follows, (last_move + 1..last_move + 5).collect::<Vec<_>>());
    if let Some(explore) = legal
        .iter()
        .position(|action| matches!(action, Action::Explore { .. }))
    {
        assert!(explore > last_move + 4);
    }
}

// C6 — spec 017, C10
#[test]
fn following_a_route_equals_clicking_each_step() {
    let mut checked = 0;
    for figure in [Hex::ORIGIN, VILLAGE, PLAIN_SW] {
        for points in [4, 5, 6, 7] {
            let state = day(figure, points);
            let enemies = [ORCS_1, ORCS_2];
            let routes = routes_avoiding(state.map(), figure, TimeOfDay::Day, &enemies);
            for to in follow_targets(&state) {
                let followed = done(&state, follow(to));
                let route = routes.iter().find(|route| route.to == to).expect("trajet");
                let clicked = route.steps.iter().fold(state.clone(), |s, &hex| {
                    done(&s, Action::MoveHero { to: hex })
                });
                assert_eq!(followed, clicked, "{figure:?} → {to:?}, {points} points");
                checked += 1;
            }
        }
    }
    assert!(checked > 10, "{checked} trajets vérifiés");
}

// C7 — MKUE p. 7, 3.a, note ; spec 017, C7
#[test]
fn night_changes_the_followed_route() {
    let state = at(Hex::ORIGIN, 6, TimeOfDay::Night);
    let events = decided(&state, &follow(PLAIN_W));
    assert_eq!(
        steps(&events),
        [
            (Hex::ORIGIN, Hex::new(0, -1), 2),
            (Hex::new(0, -1), PLAIN_SW, 2),
            (PLAIN_SW, PLAIN_W, 2),
        ]
    );
}

// C8 — MKUE p. 7, 4.c ; p. 6, 6.d
#[test]
fn no_route_once_movement_is_over() {
    let free = day(VILLAGE, 7);
    assert!(!follow_targets(&free).is_empty());
    let engaging = done(&free, Action::MoveHero { to: PLAIN_W });
    assert!(engaging.movement_ended(SOLO));
    assert!(follow_targets(&engaging).is_empty());
    refused(&engaging, follow(Hex::new(1, -1)));
    let fighting = done(&engaging, Action::StartCombat);
    assert!(follow_targets(&fighting).is_empty());
    refused(&fighting, follow(Hex::new(1, -1)));
}

fn portal_runner() -> (GameRunner, Puppet) {
    runner(setup(Some(Hex::ORIGIN), TimeOfDay::Day))
}

/// Portail, 5 points, puis le trajet suivi vers la Plaine (2, −2).
fn runner_after_route() -> GameRunner {
    let (mut runner, puppet) = portal_runner();
    for action in cards_for(5) {
        play(&mut runner, &puppet, action);
    }
    play(&mut runner, &puppet, follow(PLAIN_W));
    assert_eq!(runner.state().figure(SOLO), Some(PLAIN_W));
    runner
}

// C9 — Q1 ; MKUE p. 5, Reverting 1
#[test]
fn undo_takes_back_the_whole_route() {
    let mut runner = runner_after_route();
    runner.undo().expect("le trajet se reprend");
    assert_eq!(runner.state().figure(SOLO), Some(Hex::ORIGIN));
    assert_eq!(runner.state().move_points(SOLO), 5);
}

// C10
#[test]
fn replaying_log_after_a_followed_route() {
    let runner = runner_after_route();
    assert_eq!(runner.log().replay(), *runner.state());
}

// L1
#[test]
fn following_a_route_to_a_neighbor_is_refused() {
    let state = day(Hex::ORIGIN, 5);
    refused(&state, follow(Hex::new(1, -1)));
    refused(&state, follow(Hex::new(0, -1)));
}

// L2
#[test]
fn following_a_route_without_a_route_is_refused() {
    let state = day(VILLAGE, 7);
    // Case d'un ennemi errant, lac, hors des tuiles.
    for to in [ORCS_1, Hex::new(-1, 0), Hex::new(9, 9)] {
        refused(&state, follow(to));
    }
}

// L3
#[test]
fn no_route_without_a_figure() {
    let state = setup(None, TimeOfDay::Day)
        .iter()
        .fold(GameState::default(), apply);
    let state = cards_for(7).into_iter().fold(state, |s, a| done(&s, a));
    assert!(follow_targets(&state).is_empty());
    refused(&state, follow(PLAIN_W));
}
