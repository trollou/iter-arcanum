//! Scénarios de la spec 016 (fin du tour).

use arcanum_core::{
    Action, ActionError, BasicAction, CardId, CardPlay, DeedCard, Event, GameConfig, GameRunner,
    GameState, Hero, Hex, PlayerEvent, PlayerId, Scenario, SidewaysEffect, StepOutcome, TileId,
    TilePosition, apply, decide, filter_event, legal_actions, new_game, pending_decision, view,
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
        hero: Hero::Tovak,
        dummy_hero: Default::default(),
    }
}

/// Partie de bac à sable construite par événements : les trois tuiles de la mise en place,
/// la figurine sur le portail, le deck `order` (dessus en premier) et ses `drawn` premières
/// cartes en main.
fn state_with_deck(order: Vec<CardId>, drawn: usize) -> GameState {
    let mut events = vec![
        Event::GameStarted { config: config(0) },
        Event::TilePlaced {
            tile: TileId::StartingA,
            position: TilePosition::ORIGIN,
        },
        Event::TilePlaced {
            tile: TileId::Countryside(1),
            position: TilePosition::from_lattice(0, 1),
        },
        Event::TilePlaced {
            tile: TileId::Countryside(2),
            position: TilePosition::from_lattice(1, 0),
        },
        Event::HeroFigurePlaced {
            player: SOLO,
            hex: Hex::ORIGIN,
        },
        Event::DeckShuffled {
            player: SOLO,
            order: order.clone(),
        },
    ];
    events.extend(
        order
            .into_iter()
            .take(drawn)
            .map(|card| Event::CardDrawn { player: SOLO, card }),
    );
    events.iter().fold(GameState::default(), apply)
}

/// Le reste du deck de Tovak, sans les cartes de `hand`, dans l'ordre du deck de départ.
fn rest_of_deck(hand: &[CardId]) -> Vec<CardId> {
    Hero::Tovak
        .starting_deck()
        .into_iter()
        .filter(|card| !hand.contains(card))
        .collect()
}

/// Partie dont la main est exactement `hand`, le reste du deck de Tovak derrière.
fn state_with(hand: &[CardId]) -> GameState {
    let mut order = hand.to_vec();
    order.extend(rest_of_deck(hand));
    state_with_deck(order, hand.len())
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

fn sideways(card: CardId, effect: SidewaysEffect) -> Action {
    Action::PlayCard {
        card,
        play: CardPlay::Sideways(effect),
    }
}

fn end_turn(discard: &[CardId]) -> Action {
    Action::EndTurn {
        discard: discard.to_vec(),
        reclaim: None,
        glade: None,
        steady_tempo: false,
    }
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

fn end_turns(state: &GameState) -> Vec<Action> {
    legal_actions(state, SOLO)
        .into_iter()
        .filter(|action| matches!(action, Action::EndTurn { .. }))
        .collect()
}

/// Les cinq cartes de départ des cas C1 à C4 : Marche, Endurance, Rage, Promesse, Menace.
fn five() -> [CardId; 5] {
    [
        card(March, 1),
        card(Stamina, 1),
        card(Rage, 1),
        card(Promise, 1),
        card(Threaten, 1),
    ]
}

/// Marche, Endurance, puis Rage de côté pour Mouvement : 5 points de mouvement.
fn three_cards_played() -> GameState {
    let [march, stamina, rage, _, _] = five();
    let state = state_with(&five());
    let state = done(&state, basic(march));
    let state = done(&state, basic(stamina));
    done(&state, sideways(rage, SidewaysEffect::Move))
}

// --- Fin du tour ---

// Spec 016 C1
#[test]
fn ending_turn_discards_played_cards_and_draws_up_to_hand_limit() {
    // MKUE p. 4, 5 ; p. 11, 3.b et 7.b
    let [march, stamina, rage, promise, threaten] = five();
    let rest = rest_of_deck(&five());

    let state = done(&three_cards_played(), end_turn(&[]));

    assert!(state.play_area(SOLO).is_empty());
    assert_eq!(state.discard_pile(SOLO), &[march, stamina, rage]);
    let mut hand = vec![promise, threaten];
    hand.extend(&rest[..3]);
    assert_eq!(state.hand(SOLO), hand.as_slice());
    assert_eq!(state.deck(SOLO), &rest[3..]);
    assert_eq!(state.deck(SOLO).len(), 8);
}

// Spec 016 C2
#[test]
fn end_turn_events_come_in_rule_order() {
    // MKUE p. 11, 3 puis 7
    let [_, _, _, promise, _] = five();
    let rest = rest_of_deck(&five());

    let events = decide(
        &three_cards_played(),
        SOLO,
        &end_turn(&[promise]),
        &mut config(0).rngs(),
    )
    .expect("légal");

    let mut expected = vec![
        Event::TurnEnded { player: SOLO },
        Event::CardDiscarded {
            player: SOLO,
            card: promise,
        },
    ];
    expected.extend(
        rest[..4]
            .iter()
            .map(|&card| Event::CardDrawn { player: SOLO, card }),
    );
    expected.push(Event::TurnStarted {
        player: SOLO,
        turn: 2,
    });
    assert_eq!(events, expected);
}

// Spec 016 C3
#[test]
fn unspent_points_are_lost_at_end_of_turn() {
    // MKUE p. 6, 6.e
    let [march, _, rage, promise, _] = five();
    let state = state_with(&five());
    let state = done(&state, basic(march));
    let state = done(&state, sideways(rage, SidewaysEffect::Move));
    let state = done(&state, basic(promise));
    assert_eq!(state.move_points(SOLO), 3);
    assert_eq!(state.influence_points(SOLO), 2);

    let state = done(&state, end_turn(&[]));

    assert_eq!(state.move_points(SOLO), 0);
    assert_eq!(state.influence_points(SOLO), 0);
}

// Spec 016 C4
#[test]
fn discarding_before_drawing_draws_more_cards() {
    // MKUE p. 11, 7.a
    let [march, stamina, rage, promise, threaten] = five();
    let rest = rest_of_deck(&five());

    let state = done(&three_cards_played(), end_turn(&[promise]));

    assert_eq!(state.discard_pile(SOLO), &[march, stamina, rage, promise]);
    let mut hand = vec![threaten];
    hand.extend(&rest[..4]);
    assert_eq!(state.hand(SOLO), hand.as_slice());
    assert_eq!(state.deck(SOLO).len(), 7);
}

// Spec 016 C5
#[test]
fn discarding_one_card_is_enough_to_end_turn() {
    // MKUE p. 6, 5.a
    let [march, stamina, rage, promise, threaten] = five();
    let rest = rest_of_deck(&five());

    let state = done(&state_with(&five()), end_turn(&[march]));

    assert_eq!(state.discard_pile(SOLO), &[march]);
    assert_eq!(
        state.hand(SOLO),
        &[stamina, rage, promise, threaten, rest[0]]
    );
}

/// Main de 5, et une seule carte derrière dans le deck.
fn one_card_left_in_deck() -> (GameState, CardId) {
    let last = card(Instinct, 1);
    let mut order = five().to_vec();
    order.push(last);
    (state_with_deck(order, 5), last)
}

fn play_whole_hand(mut state: GameState) -> GameState {
    while let Some(&held) = state.hand(SOLO).first() {
        state = done(&state, sideways(held, SidewaysEffect::Influence));
    }
    state
}

// Spec 016 C6
#[test]
fn drawing_stops_when_deck_runs_out() {
    // MKUE p. 11, 7.d
    let (state, last) = one_card_left_in_deck();

    let state = done(&play_whole_hand(state), end_turn(&[]));

    assert_eq!(state.hand(SOLO), &[last]);
    assert!(state.deck(SOLO).is_empty());
    // La défausse n'est pas remélangée.
    assert_eq!(state.discard_pile(SOLO), five().as_slice());
}

// Spec 016 L4
#[test]
fn deck_empty_with_cards_in_hand_keeps_playing() {
    // MKUE p. 6, 3.c
    let (state, last) = one_card_left_in_deck();
    let state = done(&play_whole_hand(state), end_turn(&[]));
    assert!(state.deck(SOLO).is_empty());
    assert!(pending_decision(&state).is_some());

    // La fin du tour est acceptée sans rien piocher.
    let state = done(&state, end_turn(&[last]));
    assert!(state.hand(SOLO).is_empty());
    assert!(state.deck(SOLO).is_empty());
    assert_eq!(state.discard_pile(SOLO).len(), 6);
    assert_eq!(state.turn(), 3);
}

// Spec 016 C7
#[test]
fn hand_above_limit_draws_nothing() {
    // MKUE p. 11, 7.c : aucune carte ne fait encore piocher, l'état est construit.
    let hand = five();
    let rest = rest_of_deck(&hand);
    let mut order = hand.to_vec();
    order.extend(&rest);

    let six = state_with_deck(order.clone(), 6);
    let state = done(&six, basic(hand[0]));
    let state = done(&state, end_turn(&[]));
    assert_eq!(state.hand(SOLO).len(), 5);
    assert_eq!(state.deck(SOLO), &rest[1..]);

    let seven = state_with_deck(order, 7);
    let state = done(&seven, basic(hand[0]));
    let state = done(&state, end_turn(&[]));
    assert_eq!(state.hand(SOLO).len(), 6);
    assert_eq!(state.deck(SOLO), &rest[2..]);
}

// Spec 016 C8
#[test]
fn next_turn_starts_with_a_fresh_hand() {
    let state = state_with(&five());
    assert_eq!(state.turn(), 1);
    assert_eq!(view(&state, SOLO).turn, 1);

    // La figurine s'est déplacée au tour 1 ; elle peut se déplacer de nouveau au tour 2.
    let state = done(
        &three_cards_played(),
        Action::MoveHero { to: Hex::new(1, 0) },
    );
    let state = done(&state, end_turn(&[]));
    assert_eq!(state.turn(), 2);
    assert_eq!(view(&state, SOLO).turn, 2);

    // Toute la main de côté pour Mouvement : 5 points, la figurine repart.
    let mut state = state;
    while let Some(&held) = state.hand(SOLO).first() {
        state = done(&state, sideways(held, SidewaysEffect::Move));
    }
    assert_eq!(state.move_points(SOLO), 5);
    assert!(
        legal_actions(&state, SOLO)
            .iter()
            .any(|action| matches!(action, Action::MoveHero { .. }))
    );
}

// Spec 016 C9
#[test]
fn legal_actions_end_with_end_turn_choices() {
    let [march, stamina, rage, promise, threaten] = five();
    let state = state_with(&five());

    // Au début du tour, rien de joué : 31 fins de tour, pas de sous-ensemble vide.
    let start = end_turns(&state);
    assert_eq!(start.len(), 31);
    assert_eq!(start[0], end_turn(&[march]));
    assert_eq!(start[30], end_turn(&five()));

    // Promesse jouée pour Influence : pas de point de mouvement, donc ni déplacement ni
    // exploration ; restent les cartes puis les 16 fins de tour.
    let state = done(&state, basic(promise));
    let (a, b, c, d) = (march, stamina, rage, threaten);
    let expected_ends: Vec<Action> = [
        vec![],
        vec![a],
        vec![b],
        vec![c],
        vec![d],
        vec![a, b],
        vec![a, c],
        vec![a, d],
        vec![b, c],
        vec![b, d],
        vec![c, d],
        vec![a, b, c],
        vec![a, b, d],
        vec![a, c, d],
        vec![b, c, d],
        vec![a, b, c, d],
    ]
    .iter()
    .map(|discard| end_turn(discard))
    .collect();

    let legal = legal_actions(&state, SOLO);
    let first_end = legal
        .iter()
        .position(|action| matches!(action, Action::EndTurn { .. }))
        .expect("une fin de tour");
    assert!(
        legal[..first_end]
            .iter()
            .all(|action| matches!(action, Action::PlayCard { .. }))
    );
    assert_eq!(&legal[first_end..], expected_ends.as_slice());
    for action in &legal {
        assert!(
            decide(&state, SOLO, action, &mut config(0).rngs()).is_ok(),
            "{action:?}"
        );
    }
}

// Spec 016 C10 (remplace spec 013 C15)
#[test]
fn empty_hand_after_playing_can_still_end_turn() {
    let state = play_whole_hand(state_with(&five()));
    assert!(state.hand(SOLO).is_empty());
    assert!(pending_decision(&state).is_some());
    assert_eq!(legal_actions(&state, SOLO), vec![end_turn(&[])]);
}

/// Joue les `count` premières cartes de la main de côté pour Influence.
fn play_first(mut state: GameState, count: usize) -> GameState {
    for _ in 0..count {
        let held = state.hand(SOLO)[0];
        state = done(&state, sideways(held, SidewaysEffect::Influence));
    }
    state
}

fn sizes(state: &GameState) -> (usize, usize, usize) {
    (
        state.hand(SOLO).len(),
        state.deck(SOLO).len(),
        state.discard_pile(SOLO).len(),
    )
}

// Spec 016 C11
#[test]
fn whole_deck_is_played_over_several_turns() {
    // MKUE p. 6, 3.b : deck et main vides au début du tour, la manche doit finir (spec 023).
    let mut rng = config(7).rngs();
    let state = new_game(&config(7), &mut rng)
        .iter()
        .fold(GameState::default(), apply);
    assert_eq!(sizes(&state), (5, 11, 0));

    let state = done(&play_first(state, 3), end_turn(&[]));
    assert_eq!(sizes(&state), (5, 8, 3));

    let state = play_first(state, 4);
    let kept = state.hand(SOLO)[0];
    let state = done(&state, end_turn(&[kept]));
    assert_eq!(sizes(&state), (5, 3, 8));

    let state = done(&play_first(state, 2), end_turn(&[]));
    assert_eq!(sizes(&state), (5, 1, 10));

    let state = done(&play_first(state, 5), end_turn(&[]));
    assert_eq!(sizes(&state), (1, 0, 15));

    let state = done(&play_first(state, 1), end_turn(&[]));
    assert_eq!(sizes(&state), (0, 0, 16));
    assert_eq!(state.turn(), 6);

    // Spec 023 : la seule action est d'annoncer la fin de la manche.
    assert!(pending_decision(&state).is_some());
    assert_eq!(
        legal_actions(&state, SOLO),
        vec![Action::AnnounceEndOfRound]
    );
}

fn run_to_end(runner: &mut GameRunner) {
    for _ in 0..1_000 {
        if runner.step() == Ok(StepOutcome::Finished) {
            return;
        }
    }
    panic!("la partie ne s'est pas terminée");
}

// Spec 016 C12
#[test]
fn runner_plays_until_deck_and_hand_are_empty() {
    let mut runner = GameRunner::new(config(13), vec![Box::new(FirstChoice::default())]);

    run_to_end(&mut runner);

    let state = runner.state();
    assert!(state.hand(SOLO).is_empty());
    assert!(state.deck(SOLO).is_empty());
    assert!(state.play_area(SOLO).is_empty());
    // Les 16 cartes du deck de départ, plus les Blessures reçues en combat (spec 031), qui se
    // défaussent une à une par le repos prolongé.
    let discarded = state.discard_pile(SOLO);
    assert_eq!(
        discarded
            .iter()
            .filter(|card| !card.card().is_wound())
            .count(),
        16
    );
    assert_eq!(pending_decision(state), None);
}

// --- Information cachée et journal ---

// Spec 016 C13
#[test]
fn end_turn_hides_only_drawn_cards() {
    let [_, _, _, promise, _] = five();
    let rest = rest_of_deck(&five());

    assert_eq!(
        filter_event(&Event::TurnEnded { player: SOLO }, OTHER),
        PlayerEvent::TurnEnded { player: SOLO }
    );
    assert_eq!(
        filter_event(
            &Event::CardDiscarded {
                player: SOLO,
                card: promise
            },
            OTHER
        ),
        PlayerEvent::CardDiscarded {
            player: SOLO,
            card: promise
        }
    );
    assert_eq!(
        filter_event(
            &Event::CardDrawn {
                player: SOLO,
                card: rest[0]
            },
            OTHER
        ),
        PlayerEvent::CardDrawn {
            player: SOLO,
            card: None
        }
    );
    assert_eq!(
        filter_event(
            &Event::TurnStarted {
                player: SOLO,
                turn: 2
            },
            OTHER
        ),
        PlayerEvent::TurnStarted {
            player: SOLO,
            turn: 2
        }
    );
}

// Spec 016 C14
#[test]
fn replaying_log_rebuilds_turns() {
    let mut runner = GameRunner::new(config(5), vec![Box::new(FirstChoice::default())]);
    run_to_end(&mut runner);
    assert!(runner.state().turn() > 2);

    assert_eq!(&runner.log().replay(), runner.state());
}

// --- Cas limites et refus ---

// Spec 016 L1
#[test]
fn ending_turn_without_playing_or_discarding_is_refused() {
    // MKUE p. 6, 5.a
    let state = state_with(&five());
    refused(&state, end_turn(&[]));
}

// Spec 016 L2
#[test]
fn end_turn_discard_must_come_from_hand() {
    // MKUE p. 11, 7.a
    let [march, _, _, promise, _] = five();
    let state = three_cards_played();
    // Marche est dans la zone de jeu, Instinct dans le deck.
    refused(&state, end_turn(&[march]));
    refused(&state, end_turn(&[card(Instinct, 1)]));
    refused(&state, end_turn(&[promise, promise]));
}

// Spec 016 L5
#[test]
fn turn_ended_with_empty_play_area_only_clears_points() {
    let state = state_with(&five());
    let after = apply(state.clone(), &Event::TurnEnded { player: SOLO });
    assert_eq!(after, state);

    let state = three_cards_played();
    let after = apply(state.clone(), &Event::TurnEnded { player: SOLO });
    let again = apply(after.clone(), &Event::TurnEnded { player: SOLO });
    assert_eq!(again, after);
    assert_eq!(after.move_points(SOLO), 0);
    assert_eq!(after.hand(SOLO), state.hand(SOLO));
}
