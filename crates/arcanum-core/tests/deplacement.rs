//! Scénarios de la spec 014 (déplacement).

use arcanum_core::{
    Action, ActionError, BasicAction, BasicColor, CardId, CardPlay, DeedCard, Event, Feature,
    GameConfig, GameRunner, GameState, Hero, Hex, MoveCost, PlayerEvent, PlayerId, Rampaging,
    Scenario, SidewaysEffect, Site, SpaceContent, StepOutcome, Terrain, TileId, TilePosition,
    TimeOfDay, apply, decide, filter_event, legal_actions, move_cost, new_game, pending_decision,
    view,
};

mod premier_choix;
use premier_choix::FirstChoice;

const SOLO: PlayerId = PlayerId::SOLO;
const OTHER: PlayerId = PlayerId(1);

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

/// Emplacement `B` du bac à sable (tuile de campagne 1), centre (1, −3).
fn b() -> TilePosition {
    TilePosition::from_lattice(0, 1)
}

/// Emplacement `A` du bac à sable (tuile de campagne 2), centre (3, −2).
fn a() -> TilePosition {
    TilePosition::from_lattice(1, 0)
}

/// Les tuiles du bac à sable à la mise en place : départ A, campagne 1 en B, campagne 2 en A.
fn sandbox_tiles() -> Vec<(TileId, TilePosition)> {
    vec![
        (TileId::StartingA, TilePosition::ORIGIN),
        (TileId::Countryside(1), b()),
        (TileId::Countryside(2), a()),
    ]
}

/// Partie construite par événements : ces tuiles, la figurine sur `figure`, et `hand` en main
/// (piochée dans cet ordre), le reste du deck d'Arythea derrière.
fn state_with(tiles: &[(TileId, TilePosition)], figure: Hex, hand: &[CardId]) -> GameState {
    let mut order = hand.to_vec();
    order.extend(
        Hero::Arythea
            .starting_deck()
            .into_iter()
            .filter(|card| !hand.contains(card)),
    );
    let mut events = vec![Event::GameStarted { config: config(0) }];
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
    events.iter().fold(GameState::default(), apply)
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

fn move_to(q: i32, r: i32) -> Action {
    Action::MoveHero { to: Hex::new(q, r) }
}

/// Valide l'action et applique ses événements.
fn done(state: &GameState, action: Action) -> GameState {
    let events = decide(state, SOLO, &action, &mut config(0).rngs())
        .unwrap_or_else(|error| panic!("{action:?} doit être légale : {error:?}"));
    events.iter().fold(state.clone(), apply)
}

fn refused(state: &GameState, action: Action) {
    assert_eq!(
        decide(state, SOLO, &action, &mut config(0).rngs()),
        Err(ActionError::NotLegal),
        "{action:?}"
    );
}

fn moves(state: &GameState) -> Vec<Action> {
    legal_actions(state, SOLO)
        .into_iter()
        .filter(|action| matches!(action, Action::MoveHero { .. }))
        .collect()
}

fn bare(terrain: Terrain) -> SpaceContent {
    SpaceContent {
        terrain,
        feature: None,
    }
}

fn site(terrain: Terrain, site: Site) -> SpaceContent {
    SpaceContent {
        terrain,
        feature: Some(Feature::Site(site)),
    }
}

// --- Coûts ---

// Spec 014 C1 — MKUE p. 7, 3 ; plateau Jour/Nuit
#[test]
fn move_costs_by_day() {
    let day = |terrain| move_cost(bare(terrain), TimeOfDay::Day);
    assert_eq!(day(Terrain::Plains), MoveCost::Points(2));
    assert_eq!(day(Terrain::Hills), MoveCost::Points(3));
    assert_eq!(day(Terrain::Forest), MoveCost::Points(3));
    assert_eq!(day(Terrain::Wasteland), MoveCost::Points(4));
    assert_eq!(day(Terrain::Desert), MoveCost::Points(5));
    assert_eq!(day(Terrain::Swamp), MoveCost::Points(5));
    assert_eq!(day(Terrain::Lake), MoveCost::Inaccessible);
    assert_eq!(day(Terrain::Mountain), MoveCost::Inaccessible);
    assert_eq!(day(Terrain::Sea), MoveCost::Inaccessible);
}

// Spec 014 C2 — MKUE p. 7, 3.a, note ; plateau Jour/Nuit
#[test]
fn move_costs_by_night() {
    let night = |terrain| move_cost(bare(terrain), TimeOfDay::Night);
    assert_eq!(night(Terrain::Plains), MoveCost::Points(2));
    assert_eq!(night(Terrain::Hills), MoveCost::Points(3));
    assert_eq!(night(Terrain::Forest), MoveCost::Points(5));
    assert_eq!(night(Terrain::Wasteland), MoveCost::Points(4));
    assert_eq!(night(Terrain::Desert), MoveCost::Points(3));
    assert_eq!(night(Terrain::Swamp), MoveCost::Points(5));
    assert_eq!(night(Terrain::Lake), MoveCost::Inaccessible);
    assert_eq!(night(Terrain::Mountain), MoveCost::Inaccessible);
    assert_eq!(night(Terrain::Sea), MoveCost::Inaccessible);
}

// Spec 014 C3 — MKUE p. 7, 3.a, note : une cité coûte toujours 2
#[test]
fn city_costs_two_whatever_the_terrain() {
    for terrain in [Terrain::Plains, Terrain::Hills, Terrain::Desert] {
        for time in [TimeOfDay::Day, TimeOfDay::Night] {
            assert_eq!(
                move_cost(site(terrain, Site::City(BasicColor::Red)), time),
                MoveCost::Fortified(2),
                "{terrain:?} {time:?}"
            );
        }
    }
    // Les autres sites fortifiés gardent le coût de leur terrain.
    assert_eq!(
        move_cost(site(Terrain::Hills, Site::Keep), TimeOfDay::Day),
        MoveCost::Fortified(3)
    );
    assert_eq!(
        move_cost(site(Terrain::Desert, Site::MageTower), TimeOfDay::Night),
        MoveCost::Fortified(3)
    );
}

// Spec 014 C4 — MKUE p. 7, 4.a
#[test]
fn fortified_sites_are_keep_mage_tower_and_city() {
    assert!(Site::Keep.is_fortified());
    assert!(Site::MageTower.is_fortified());
    for color in [
        BasicColor::Red,
        BasicColor::Blue,
        BasicColor::White,
        BasicColor::Green,
    ] {
        assert!(Site::City(color).is_fortified());
        assert!(!Site::Mine(color).is_fortified());
    }
    for site in [
        Site::Village,
        Site::Monastery,
        Site::MagicalGlade,
        Site::Portal,
        Site::Dungeon,
        Site::Tomb,
        Site::MonsterDen,
        Site::SpawningGrounds,
        Site::Ruins,
    ] {
        assert!(!site.is_fortified(), "{site:?}");
    }
    // Un site non fortifié ne change pas le coût de son terrain.
    assert_eq!(
        move_cost(site(Terrain::Plains, Site::Village), TimeOfDay::Day),
        MoveCost::Points(2)
    );
}

// --- Mise en place ---

// Spec 014 C5 — MKUE p. 3 : plateau Jour/Nuit côté jour
#[test]
fn game_starts_by_day() {
    let config = config(4);
    let state = new_game(&config, &mut config.rngs())
        .iter()
        .fold(GameState::default(), apply);
    assert_eq!(state.time_of_day(), TimeOfDay::Day);
    assert_eq!(view(&state, SOLO).time_of_day, TimeOfDay::Day);
}

// --- Se déplacer ---

// Spec 014 C6 — MKUE p. 7, 3
#[test]
fn only_affordable_accessible_neighbors_are_legal() {
    let march = card(March, 1);
    let state = done(
        &state_with(&sandbox_tiles(), Hex::ORIGIN, &[march]),
        basic(march),
    );
    // E : Plaine ; N-O : Plaine ; N-E : Forêt (3) ; S-E, S-O, O : Littoral.
    assert_eq!(moves(&state), vec![move_to(1, 0), move_to(0, -1)]);
}

// Spec 014 C7 — MKUE p. 7, 3
#[test]
fn moving_into_plains_costs_two_points() {
    let march = card(March, 1);
    let state = done(
        &state_with(&sandbox_tiles(), Hex::ORIGIN, &[march]),
        basic(march),
    );
    let events = decide(&state, SOLO, &move_to(1, 0), &mut config(0).rngs())
        .expect("la Plaine à l'E est accessible");
    assert_eq!(
        events,
        vec![Event::HeroMoved {
            player: SOLO,
            from: Hex::ORIGIN,
            to: Hex::new(1, 0),
            cost: 2,
        }]
    );
    let after = events.iter().fold(state, apply);
    assert_eq!(after.figure(SOLO), Some(Hex::new(1, 0)));
    assert_eq!(after.move_points(SOLO), 0);
}

// Spec 014 C8 — MKUE p. 7, 3 ; plateau Jour/Nuit
#[test]
fn moving_into_forest_costs_three_points_by_day() {
    let (march, rage) = (card(March, 1), card(Rage, 1));
    let state = state_with(&sandbox_tiles(), Hex::ORIGIN, &[march, rage]);
    let state = done(&done(&state, basic(march)), sideways_move(rage));
    assert_eq!(state.move_points(SOLO), 3);

    let state = done(&state, move_to(1, -1));
    assert_eq!(state.figure(SOLO), Some(Hex::new(1, -1)));
    assert_eq!(state.move_points(SOLO), 0);
}

// Spec 014 C9 — MKUE p. 7, 6.b
#[test]
fn move_points_left_add_up_with_new_cards() {
    let (march1, march2, stamina) = (card(March, 1), card(March, 2), card(Stamina, 1));
    let state = state_with(&sandbox_tiles(), Hex::ORIGIN, &[march1, march2, stamina]);
    let state = done(&done(&state, basic(march1)), basic(march2));
    assert_eq!(state.move_points(SOLO), 4);
    let state = done(&state, move_to(0, -1));
    assert_eq!(state.move_points(SOLO), 2);
    let state = done(&state, basic(stamina));
    assert_eq!(state.move_points(SOLO), 4);
    // (1, −1) : Forêt de la tuile de départ, voisine de (0, −1) à l'E.
    let state = done(&state, move_to(1, -1));
    assert_eq!(state.figure(SOLO), Some(Hex::new(1, -1)));
    assert_eq!(state.move_points(SOLO), 1);
}

// Spec 014 C10 — MKUE p. 7, 3 et 6
#[test]
fn moving_across_tiles_pays_each_space() {
    let hand = [
        card(March, 1),
        card(March, 2),
        card(Stamina, 1),
        card(Swiftness, 1),
    ];
    let mut state = state_with(&sandbox_tiles(), Hex::ORIGIN, &hand);
    for card in hand {
        state = done(&state, basic(card));
    }
    assert_eq!(state.move_points(SOLO), 8);
    // Plaine N-O (2), puis Plaine S-O de la tuile de campagne 1 (2), puis son centre, Forêt (3).
    for (q, r) in [(0, -1), (0, -2), (1, -3)] {
        state = done(&state, move_to(q, r));
    }
    assert_eq!(state.figure(SOLO), Some(Hex::new(1, -3)));
    assert_eq!(state.move_points(SOLO), 1);
}

// Spec 014 C11 — MKUE p. 7, 4.b : seul un jeton d'ennemi (spec 030) bloque
#[test]
fn printed_rampaging_symbol_does_not_block() {
    let orcs = Hex::new(1, -4);
    let (march, stamina) = (card(March, 1), card(Stamina, 1));
    let state = state_with(&sandbox_tiles(), Hex::new(1, -3), &[march, stamina]);
    assert_eq!(
        state.map().space(orcs),
        Some(SpaceContent {
            terrain: Terrain::Forest,
            feature: Some(Feature::Rampaging(Rampaging::OrcMarauders)),
        })
    );
    let state = done(&done(&state, basic(march)), basic(stamina));
    let state = done(&state, move_to(orcs.q, orcs.r));
    assert_eq!(state.figure(SOLO), Some(orcs));
    assert_eq!(state.move_points(SOLO), 1);
}

// Spec 014 C12 — MKUE p. 7, 4.d
#[test]
fn non_fortified_site_does_not_stop_movement() {
    let village = Hex::new(2, -3);
    let (march, stamina) = (card(March, 1), card(Stamina, 1));
    let state = state_with(&sandbox_tiles(), Hex::new(1, -3), &[march, stamina]);
    assert_eq!(
        state.map().space(village),
        Some(site(Terrain::Plains, Site::Village))
    );
    let state = done(&done(&state, basic(march)), basic(stamina));
    let state = done(&state, move_to(village.q, village.r));
    assert_eq!(state.move_points(SOLO), 2);
    // On continue : Plaine S-E de la tuile de campagne 1, au S-O du village.
    let state = done(&state, move_to(1, -2));
    assert_eq!(state.figure(SOLO), Some(Hex::new(1, -2)));
    assert_eq!(state.move_points(SOLO), 0);
}

// Spec 014 C13
#[test]
fn decision_lasts_while_a_move_is_affordable() {
    let march = card(March, 1);
    let state = done(
        &state_with(&sandbox_tiles(), Hex::ORIGIN, &[march]),
        basic(march),
    );
    assert!(state.hand(SOLO).is_empty());
    assert!(pending_decision(&state).is_some());
    assert_eq!(
        legal_actions(&state, SOLO),
        vec![
            move_to(1, 0),
            move_to(0, -1),
            Action::EndTurn {
                discard: vec![],
                reclaim: None,
                glade: None,
                steady_tempo: false
            }
        ]
    );

    // Spec 016 : main vide et plus de point, il reste à finir le tour.
    let state = done(&state, move_to(1, 0));
    assert_eq!(state.move_points(SOLO), 0);
    assert!(pending_decision(&state).is_some());
    assert_eq!(
        legal_actions(&state, SOLO),
        vec![Action::EndTurn {
            discard: vec![],
            reclaim: None,
            glade: None,
            steady_tempo: false
        }]
    );
}

// Spec 014 C14
#[test]
fn legal_actions_list_cards_then_moves() {
    let (march, rage) = (card(March, 1), card(Rage, 1));
    let (stamina, swiftness) = (card(Stamina, 1), card(Swiftness, 1));
    let state = state_with(
        &sandbox_tiles(),
        Hex::ORIGIN,
        &[march, stamina, rage, swiftness],
    );
    let state = done(&done(&state, basic(march)), basic(stamina));
    assert_eq!(state.move_points(SOLO), 4);

    let sideways = |card, effect| Action::PlayCard {
        card,
        play: CardPlay::Sideways(effect),
    };
    let expected = vec![
        sideways(rage, SidewaysEffect::Move),
        sideways(rage, SidewaysEffect::Influence),
        basic(swiftness),
        sideways(swiftness, SidewaysEffect::Move),
        sideways(swiftness, SidewaysEffect::Influence),
        // E, N-E, N-O : Plaine, Forêt, Plaine ; le Littoral jamais.
        move_to(1, 0),
        move_to(1, -1),
        move_to(0, -1),
        // Spec 036 : puis les trajets suivis vers les cases lointaines qu'on peut payer.
        Action::FollowRoute {
            to: Hex::new(1, -2),
        },
        Action::FollowRoute {
            to: Hex::new(0, -2),
        },
        // Spec 016 : puis les fins de tour, une par défausse possible.
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
        Action::EndTurn {
            discard: vec![swiftness],
            reclaim: None,
            glade: None,
            steady_tempo: false,
        },
        Action::EndTurn {
            discard: vec![rage, swiftness],
            reclaim: None,
            glade: None,
            steady_tempo: false,
        },
    ];
    let legal = legal_actions(&state, SOLO);
    assert_eq!(legal, expected);
    for action in &legal {
        assert!(
            decide(&state, SOLO, action, &mut config(0).rngs()).is_ok(),
            "{action:?}"
        );
    }
}

// --- Information cachée et journal ---

// Spec 014 C15
#[test]
fn hero_moves_are_public() {
    let moved = Event::HeroMoved {
        player: SOLO,
        from: Hex::ORIGIN,
        to: Hex::new(1, 0),
        cost: 2,
    };
    assert_eq!(
        filter_event(&moved, OTHER),
        PlayerEvent::HeroMoved {
            player: SOLO,
            from: Hex::ORIGIN,
            to: Hex::new(1, 0),
            cost: 2,
        }
    );
}

// Spec 014 C16
#[test]
fn replaying_log_rebuilds_moves() {
    let mut runner = GameRunner::new(config(17), vec![Box::new(FirstChoice::default())]);
    let mut steps = 0;
    while runner.step() != Ok(StepOutcome::Finished) {
        steps += 1;
        assert!(steps < 1_000, "la partie doit s'arrêter");
    }
    assert!(
        runner
            .log()
            .entries()
            .iter()
            .any(|entry| matches!(entry.event, Event::HeroMoved { .. })),
        "la figurine doit s'être déplacée"
    );
    assert_eq!(&runner.log().replay(), runner.state());
}

// --- Cas limites et refus ---

// Spec 014 L1 — MKUE p. 7, 3
#[test]
fn moving_to_a_non_adjacent_space_is_refused() {
    let hand = [card(March, 1), card(March, 2)];
    let mut state = state_with(&sandbox_tiles(), Hex::ORIGIN, &hand);
    for card in hand {
        state = done(&state, basic(card));
    }
    refused(&state, move_to(0, -2));
    refused(&state, move_to(0, 0));
    assert_eq!(state.figure(SOLO), Some(Hex::ORIGIN));
}

// Spec 014 L2 — MKUE p. 7, 3.a
#[test]
fn moving_into_inaccessible_terrain_is_refused() {
    let hand = [card(March, 1), card(March, 2), card(Stamina, 1)];
    let tiles = vec![
        (TileId::StartingA, TilePosition::ORIGIN),
        (TileId::Countryside(1), b()),
        (TileId::Countryside(4), a()),
    ];
    let play_all = |state: GameState| hand.iter().fold(state, |s, &c| done(&s, basic(c)));
    // Littoral : S-E, S-O et O de la tuile de départ.
    let state = play_all(state_with(&tiles, Hex::ORIGIN, &hand));
    for (q, r) in [(0, 1), (-1, 1), (-1, 0)] {
        refused(&state, move_to(q, r));
    }
    // Lac : N-E de la tuile de campagne 1.
    let state = play_all(state_with(&tiles, Hex::new(1, -3), &hand));
    refused(&state, move_to(2, -4));
    // Montagne : E de la tuile de campagne 4.
    let state = play_all(state_with(&tiles, Hex::new(3, -2), &hand));
    refused(&state, move_to(4, -2));
    assert_eq!(state.move_points(SOLO), 6);
}

// Spec 014 L3 : c'est de l'exploration (spec 015)
#[test]
fn moving_off_the_map_is_refused() {
    let hand = [card(March, 1), card(March, 2)];
    let tiles = [(TileId::StartingA, TilePosition::ORIGIN)];
    let mut state = state_with(&tiles, Hex::new(1, -1), &hand);
    for card in hand {
        state = done(&state, basic(card));
    }
    // N-E et N-O de la Forêt (1, −1) : pas de tuile.
    refused(&state, move_to(2, -2));
    refused(&state, move_to(1, -2));
}

// Spec 014 L4 — MKUE p. 7, 3
#[test]
fn moving_without_enough_points_is_refused() {
    let rage = card(Rage, 1);
    let state = done(
        &state_with(&sandbox_tiles(), Hex::ORIGIN, &[rage]),
        sideways_move(rage),
    );
    assert_eq!(state.move_points(SOLO), 1);
    refused(&state, move_to(1, 0));
    assert!(moves(&state).is_empty());
}

// Spec 014 L5 — MKUE p. 7, 4.a. Depuis la spec 043, entrer dans une forteresse ou une tour de mage
// est un assaut, permis au coût du terrain ; la cité reste fermée jusqu'à la spec 062.
#[test]
fn entering_a_fortified_site_is_an_assault_except_cities() {
    let hand = [card(March, 1), card(March, 2), card(Stamina, 1)];
    let play_all = |state: GameState| hand.iter().fold(state, |s, &c| done(&s, basic(c)));
    let with_tile_at_a = |tile, figure| {
        let tiles = [(TileId::StartingA, TilePosition::ORIGIN), (tile, a())];
        play_all(state_with(&tiles, figure, &hand))
    };
    // Forteresse : N-E de la tuile de campagne 3 (Colline), depuis son centre.
    let state = with_tile_at_a(TileId::Countryside(3), Hex::new(3, -2));
    assert!(moves(&state).contains(&move_to(4, -3)));
    // Tour de mage : centre de la tuile de campagne 4, depuis son O.
    let state = with_tile_at_a(TileId::Countryside(4), Hex::new(2, -2));
    assert!(moves(&state).contains(&move_to(3, -2)));
    // Cité : centre de la tuile royaume 5, depuis son O.
    let state = with_tile_at_a(TileId::Core(5), Hex::new(2, -2));
    assert!(matches!(
        state.map().space(Hex::new(3, -2)).and_then(|s| s.feature),
        Some(Feature::Site(Site::City(_)))
    ));
    refused(&state, move_to(3, -2));
}

// Spec 014 L6
#[test]
fn invalid_move_event_changes_nothing() {
    let march = card(March, 1);
    let state = done(
        &state_with(&sandbox_tiles(), Hex::ORIGIN, &[march]),
        basic(march),
    );
    let wrong_origin = Event::HeroMoved {
        player: SOLO,
        from: Hex::new(1, 0),
        to: Hex::new(2, 0),
        cost: 2,
    };
    assert_eq!(apply(state.clone(), &wrong_origin), state);
    let too_expensive = Event::HeroMoved {
        player: SOLO,
        from: Hex::ORIGIN,
        to: Hex::new(1, -1),
        cost: 3,
    };
    assert_eq!(apply(state.clone(), &too_expensive), state);
}
