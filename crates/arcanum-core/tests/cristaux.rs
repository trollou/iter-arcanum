//! Scénarios de la spec 021 (cristaux).

use arcanum_core::{
    Action, BasicAction, BasicColor, CRYSTAL_LIMIT, CardId, CardPlay, DeedCard, Event, Feature,
    GameConfig, GameState, Hero, Hex, PlayerEvent, PlayerId, Scenario, Site, TileId, TilePosition,
    apply, decide, filter_event, legal_actions, new_game, view,
};
use rand::SeedableRng;
use rand::seq::{IndexedRandom, SliceRandom};
use rand_chacha::ChaCha8Rng;

const SOLO: PlayerId = PlayerId::SOLO;

use BasicAction::*;
use BasicColor::*;

fn card(action: BasicAction, copy: u16) -> CardId {
    CardId::new(DeedCard::Basic(action), copy)
}

fn config(seed: u64) -> GameConfig {
    GameConfig {
        seed,
        scenario: Scenario::sandbox(),
        hero: Hero::Tovak,
        dummy_hero: Default::default(),
    }
}

/// Emplacement de la tuile posée à côté de la tuile de départ dans ces tests.
fn east() -> TilePosition {
    TilePosition::from_lattice(1, 0)
}

/// Partie de bac à sable construite par événements : tuile de départ, `tile` posée à l'est,
/// deck de Tovak mélangé par `seed`, main de 5 cartes, figurine sur `figure(map)`.
fn state_on(tile: TileId, seed: u64, figure: impl Fn(&GameState) -> Hex) -> GameState {
    let mut order = Hero::Tovak.starting_deck();
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    order.shuffle(&mut rng);
    state_with_deck(tile, order, figure)
}

/// Comme `state_on`, avec le deck `order` (dessus en premier).
fn state_with_deck(
    tile: TileId,
    order: Vec<CardId>,
    figure: impl Fn(&GameState) -> Hex,
) -> GameState {
    let mut events = vec![
        Event::GameStarted { config: config(0) },
        Event::TilePlaced {
            tile: TileId::StartingA,
            position: TilePosition::ORIGIN,
        },
        Event::TilePlaced {
            tile,
            position: east(),
        },
        Event::DeckShuffled {
            player: SOLO,
            order: order.clone(),
        },
    ];
    events.extend(
        order
            .into_iter()
            .take(5)
            .map(|card| Event::CardDrawn { player: SOLO, card }),
    );
    let state = events.iter().fold(GameState::default(), apply);
    let hex = figure(&state);
    apply(state, &Event::HeroFigurePlaced { player: SOLO, hex })
}

/// La case de la mine `color` de la tuile posée à l'est.
fn mine_hex(state: &GameState, color: BasicColor) -> Hex {
    east()
        .hexes()
        .into_iter()
        .find(|&hex| {
            state.map().space(hex).and_then(|space| space.feature)
                == Some(Feature::Site(Site::Mine(color)))
        })
        .unwrap_or_else(|| panic!("la tuile doit avoir une mine {color:?}"))
}

/// Figurine sur la mine verte de la tuile de campagne 2.
fn on_green_mine() -> GameState {
    state_on(TileId::Countryside(2), 0, |state| mine_hex(state, Green))
}

fn with_crystals(state: GameState, color: BasicColor, count: u8) -> GameState {
    (0..count).fold(state, |state, _| {
        apply(
            state,
            &Event::CrystalGained {
                player: SOLO,
                color,
            },
        )
    })
}

fn first_card(state: &GameState) -> CardId {
    state.hand(SOLO)[0]
}

fn play_sideways(state: &GameState) -> GameState {
    let action = Action::PlayCard {
        card: first_card(state),
        play: CardPlay::Sideways(arcanum_core::SidewaysEffect::Move),
    };
    done(state, action)
}

fn end_turn(discard: &[CardId]) -> Action {
    Action::EndTurn {
        discard: discard.to_vec(),
        reclaim: None,
        glade: None,
        steady_tempo: false,
    }
}

fn events_of(state: &GameState, action: Action) -> Vec<Event> {
    decide(state, SOLO, &action, &mut config(0).rngs())
        .unwrap_or_else(|error| panic!("{action:?} doit être légale : {error:?}"))
}

fn done(state: &GameState, action: Action) -> GameState {
    events_of(state, action).iter().fold(state.clone(), apply)
}

/// Joue une carte de côté puis finit le tour, sans défausse.
fn play_and_end_turn(state: &GameState) -> GameState {
    done(&play_sideways(state), end_turn(&[]))
}

fn crystal_events(events: &[Event]) -> Vec<&Event> {
    events
        .iter()
        .filter(|event| matches!(event, Event::CrystalGained { .. }))
        .collect()
}

// --- Inventaire et mine ---

// Spec 021 C1
#[test]
fn hero_starts_without_crystals() {
    // MKUE p. 15 : « Other characters start with no crystals. »
    let state = new_game(&config(3), &mut config(3).rngs())
        .iter()
        .fold(GameState::default(), apply);
    for color in BasicColor::ALL {
        assert_eq!(state.inventory(SOLO).crystals(color), 0, "{color:?}");
    }
}

// Spec 021 C2
#[test]
fn ending_turn_on_a_mine_gains_a_crystal_of_its_color() {
    // MKUE p. 11, 4.b
    let state = play_and_end_turn(&on_green_mine());
    let inventory = state.inventory(SOLO);
    assert_eq!(inventory.crystals(Green), 1);
    for color in [Red, Blue, White] {
        assert_eq!(inventory.crystals(color), 0, "{color:?}");
    }
}

// Spec 021 C3
#[test]
fn mine_benefit_comes_after_clearing_play_area_and_before_drawing() {
    // MKUE p. 11, 3, 4 puis 7
    let state = play_sideways(&on_green_mine());
    let discarded = first_card(&state);
    let events = events_of(&state, end_turn(&[discarded]));
    assert_eq!(events[0], Event::TurnEnded { player: SOLO });
    assert_eq!(
        events[1],
        Event::CrystalGained {
            player: SOLO,
            color: Green
        }
    );
    assert_eq!(
        events[2],
        Event::CardDiscarded {
            player: SOLO,
            card: discarded
        }
    );
    assert!(
        events[3..events.len() - 1]
            .iter()
            .all(|event| matches!(event, Event::CardDrawn { .. }))
    );
    assert!(matches!(events.last(), Some(Event::TurnStarted { .. })));
}

// Spec 021 C4
#[test]
fn full_color_gains_nothing_from_a_mine() {
    // MKUE p. 11, 4.b : « in which case you get nothing »
    let state = with_crystals(on_green_mine(), Green, CRYSTAL_LIMIT);
    let played = play_sideways(&state);
    let events = events_of(&played, end_turn(&[]));
    assert!(crystal_events(&events).is_empty());
    let after = events.iter().fold(played, apply);
    assert_eq!(after.inventory(SOLO).crystals(Green), 3);
}

// Spec 021 C5
#[test]
fn crystal_limit_is_per_color() {
    // MKUE p. 5, 1.b : jusqu'à trois de chaque couleur de base
    let state = with_crystals(on_green_mine(), White, 3);
    let state = play_and_end_turn(&state);
    assert_eq!(state.inventory(SOLO).crystals(White), 3);
    assert_eq!(state.inventory(SOLO).crystals(Green), 1);
}

// Spec 021 C6
#[test]
fn staying_on_a_mine_gains_a_crystal_each_turn() {
    // MKUE p. 11, 4.b
    let mut state = on_green_mine();
    for expected in [1, 2, 3, 3] {
        state = play_and_end_turn(&state);
        assert_eq!(state.inventory(SOLO).crystals(Green), expected);
    }
}

// Spec 021 C7 et L5
#[test]
fn each_mine_gives_its_own_color() {
    // MKUE p. 11, 4.b
    for (tile, color) in [
        (TileId::Countryside(6), Red),
        (TileId::Countryside(5), Blue),
        (TileId::Countryside(3), White),
        (TileId::Countryside(2), Green),
    ] {
        let state = state_on(tile, 0, |state| mine_hex(state, color));
        let state = play_and_end_turn(&state);
        assert_eq!(state.inventory(SOLO).crystals(color), 1, "{tile:?}");
    }
}

// Spec 021 L5
#[test]
fn a_core_tile_mine_works_the_same() {
    let state = state_on(TileId::Core(2), 0, |state| mine_hex(state, Green));
    let state = play_and_end_turn(&state);
    assert_eq!(state.inventory(SOLO).crystals(Green), 1);
}

// Spec 021 C8
#[test]
fn ending_turn_elsewhere_gains_no_crystal() {
    // Plaine (côté ouest) et village (côté est) de la tuile de campagne 2.
    for direction in [arcanum_core::Direction::W, arcanum_core::Direction::E] {
        let state = state_on(TileId::Countryside(2), 0, |_| east().hex_at(direction));
        let played = play_sideways(&state);
        assert!(crystal_events(&events_of(&played, end_turn(&[]))).is_empty());
    }
}

// Spec 021 C9
#[test]
fn passing_through_a_mine_gains_nothing() {
    // MKUE p. 11, 4 : « If you end your turn on… » ; Marche donne 2, la plaine en coûte 2.
    let mut order = vec![card(March, 1)];
    order.extend(
        Hero::Tovak
            .starting_deck()
            .into_iter()
            .filter(|&held| held != card(March, 1)),
    );
    let state = state_with_deck(TileId::Countryside(2), order, |state| {
        mine_hex(state, Green)
    });
    let state = done(
        &state,
        Action::PlayCard {
            card: card(March, 1),
            play: CardPlay::Basic {
                die: None,
                gain: None,
                with: None,
                option: 0,
                discard: None,
                mana: None,
            },
        },
    );
    let plains = east().hex_at(arcanum_core::Direction::W);
    let state = done(&state, Action::MoveHero { to: plains });
    assert_eq!(state.figure(SOLO), Some(plains));
    assert!(crystal_events(&events_of(&state, end_turn(&[]))).is_empty());
}

// Spec 021 C10
#[test]
fn mine_adds_no_legal_action() {
    let on_mine = on_green_mine();
    let elsewhere = state_on(TileId::Countryside(2), 0, |_| {
        east().hex_at(arcanum_core::Direction::W)
    });
    let ends = |state: &GameState| {
        legal_actions(state, SOLO)
            .into_iter()
            .filter(|action| matches!(action, Action::EndTurn { .. }))
            .count()
    };
    assert_eq!(ends(&on_mine), ends(&elsewhere));
    let cards = |state: &GameState| {
        legal_actions(state, SOLO)
            .into_iter()
            .filter(|action| matches!(action, Action::PlayCard { .. }))
            .collect::<Vec<_>>()
    };
    assert_eq!(cards(&on_mine), cards(&elsewhere));
}

// --- Information cachée et journal ---

// Spec 021 C11
#[test]
fn inventory_is_public() {
    let state = with_crystals(on_green_mine(), Red, 2);
    let seen = view(&state, SOLO).inventory;
    assert_eq!(seen, state.inventory(SOLO));
    assert_eq!(seen.crystals(Red), 2);
    let event = Event::CrystalGained {
        player: SOLO,
        color: Blue,
    };
    assert_eq!(
        filter_event(&event, PlayerId(1)),
        PlayerEvent::CrystalGained {
            player: SOLO,
            color: Blue
        }
    );
}

// Spec 021 C12
#[test]
fn replaying_log_rebuilds_inventory() {
    let start = play_sideways(&on_green_mine());
    let events = events_of(&start, end_turn(&[]));
    let state = events.iter().fold(start.clone(), apply);
    let replayed = events.iter().fold(start, apply);
    assert_eq!(replayed.inventory(SOLO), state.inventory(SOLO));
    assert_eq!(replayed, state);
}

// --- Cas limites ---

// Spec 021 L1
#[test]
fn mine_pays_even_when_only_a_card_was_discarded() {
    let state = on_green_mine();
    let discarded = first_card(&state);
    let state = done(&state, end_turn(&[discarded]));
    assert_eq!(state.inventory(SOLO).crystals(Green), 1);
}

// Spec 021 L3
#[test]
fn inventory_never_exceeds_three_per_color() {
    let mines = [
        (TileId::Countryside(6), Red),
        (TileId::Countryside(5), Blue),
        (TileId::Countryside(3), White),
        (TileId::Countryside(2), Green),
    ];
    let mut turns = 0;
    for seed in 0..60 {
        let (tile, color) = mines[seed as usize % mines.len()];
        let mut state = state_on(tile, seed, |state| mine_hex(state, color));
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        loop {
            let ends: Vec<Action> = legal_actions(&state, SOLO)
                .into_iter()
                .filter(|action| matches!(action, Action::EndTurn { .. }))
                .collect();
            let Some(action) = ends.choose(&mut rng).cloned() else {
                break;
            };
            state = done(&state, action);
            turns += 1;
            for color in BasicColor::ALL {
                assert!(state.inventory(SOLO).crystals(color) <= CRYSTAL_LIMIT);
            }
        }
    }
    assert!(turns >= 200, "{turns} fins de tour seulement");
}
