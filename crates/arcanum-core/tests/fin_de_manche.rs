//! Scénarios de la spec 023 (fin de manche).

use arcanum_core::{
    Action, ActionError, BasicColor, CardId, CardPlay, CountrysideOrder, Event, Feature,
    FinalScoring, GameConfig, GameRngs, GameRunner, GameState, Hero, Hex, ManaColor, MapShape,
    PlayerEvent, PlayerId, RngUsage, Scenario, ScenarioEnd, ScenarioError, ScenarioId,
    SidewaysEffect, Site, StepOutcome, TileDeckRecipe, TileId, TilePosition, TimeOfDay, apply,
    basic_dice_needed, decide, filter_event, legal_actions, pending_decision, roll_source,
    source_size, view,
};

mod premier_choix;
use premier_choix::FirstChoice;

const SOLO: PlayerId = PlayerId::SOLO;
const OTHER: PlayerId = PlayerId(1);

fn config_with(scenario: Scenario, seed: u64) -> GameConfig {
    GameConfig {
        seed,
        scenario,
        hero: Hero::Tovak,
        dummy_hero: Default::default(),
    }
}

fn config(seed: u64) -> GameConfig {
    config_with(Scenario::sandbox(), seed)
}

/// Scénario de test de la spec 012, avec 2 manches.
fn two_rounds() -> Scenario {
    Scenario::new(
        ScenarioId::Sandbox,
        MapShape::Wedge,
        TileDeckRecipe {
            countryside: 7,
            countryside_order: CountrysideOrder::Random,
            core_city: 2,
            core_non_city: 2,
        },
        2,
        0,
        ScenarioEnd::RoundLimit,
        FinalScoring::None,
    )
    .expect("le scénario de test est cohérent")
}

/// Emplacement de la tuile posée à l'est de la tuile de départ.
fn east() -> TilePosition {
    TilePosition::from_lattice(1, 0)
}

/// Partie construite par événements : tuile de départ, campagne 2 à l'est, figurine sur le
/// portail, deck de Tovak dans l'ordre de départ, entièrement pioché ; on garde les `kept`
/// premières cartes en main, les autres sont défaussées. Le deck est donc vide.
fn exhausted_with(scenario: Scenario, kept: usize) -> GameState {
    let order = Hero::Tovak.starting_deck();
    let mut events = vec![
        Event::GameStarted {
            config: config_with(scenario, 0),
        },
        Event::TilePlaced {
            tile: TileId::StartingA,
            position: TilePosition::ORIGIN,
        },
        Event::TilePlaced {
            tile: TileId::Countryside(2),
            position: east(),
        },
        Event::HeroFigurePlaced {
            player: SOLO,
            hex: Hex::ORIGIN,
        },
        Event::DeckShuffled {
            player: SOLO,
            order: order.clone(),
        },
        Event::SourceRolled {
            dice: vec![
                ManaColor::Basic(BasicColor::Green),
                ManaColor::Gold,
                ManaColor::Black,
            ],
        },
    ];
    events.extend(
        order
            .iter()
            .map(|&card| Event::CardDrawn { player: SOLO, card }),
    );
    events.extend(
        order
            .iter()
            .skip(kept)
            .map(|&card| Event::CardDiscarded { player: SOLO, card }),
    );
    events.iter().fold(GameState::default(), apply)
}

fn exhausted(kept: usize) -> GameState {
    exhausted_with(Scenario::sandbox(), kept)
}

/// Événements produits par l'action, qui doit être légale.
fn events_of(state: &GameState, action: &Action, seed: u64) -> Vec<Event> {
    decide(state, SOLO, action, &mut config(seed).rngs())
        .unwrap_or_else(|error| panic!("{action:?} doit être légale : {error:?}"))
}

fn done(state: &GameState, action: Action) -> GameState {
    events_of(state, &action, 0)
        .iter()
        .fold(state.clone(), apply)
}

fn announce(state: &GameState) -> GameState {
    done(state, Action::AnnounceEndOfRound)
}

fn refused(state: &GameState, action: Action) {
    assert_eq!(
        decide(state, SOLO, &action, &mut config(0).rngs()),
        Err(ActionError::NotLegal),
        "{action:?}"
    );
}

fn can_announce(state: &GameState) -> bool {
    legal_actions(state, SOLO).contains(&Action::AnnounceEndOfRound)
}

fn sideways(card: CardId) -> Action {
    Action::PlayCard {
        card,
        play: CardPlay::Sideways(SidewaysEffect::Influence),
    }
}

/// Toutes les cartes du joueur, triées : elles ne se perdent ni ne se dupliquent.
fn all_cards(state: &GameState) -> Vec<CardId> {
    let mut cards: Vec<CardId> = state
        .deck(SOLO)
        .iter()
        .chain(state.hand(SOLO))
        .chain(state.discard_pile(SOLO))
        .copied()
        .chain(state.play_area(SOLO).iter().map(|played| played.card))
        .collect();
    cards.sort();
    cards
}

fn starting_cards() -> Vec<CardId> {
    let mut cards = Hero::Tovak.starting_deck();
    cards.sort();
    cards
}

// --- Annonce ---

// Spec 023 C1
#[test]
fn empty_deck_and_hand_force_end_of_round() {
    // MKUE p. 6, 3.b
    let state = exhausted(0);
    assert_eq!(
        legal_actions(&state, SOLO),
        vec![Action::AnnounceEndOfRound]
    );
}

// Spec 023 C2
#[test]
fn empty_deck_lets_player_choose_to_announce() {
    // MKUE p. 6, 3.c
    let state = exhausted(2);
    let actions = legal_actions(&state, SOLO);
    assert_eq!(actions.last(), Some(&Action::AnnounceEndOfRound));
    assert_eq!(
        actions
            .iter()
            .filter(|action| **action == Action::AnnounceEndOfRound)
            .count(),
        1
    );
    let first = state.hand(SOLO)[0];
    assert!(actions.contains(&sideways(first)));
    assert!(
        actions
            .iter()
            .any(|action| matches!(action, Action::EndTurn { .. }))
    );
}

// Spec 023 C3
#[test]
fn announcing_needs_the_start_of_the_turn() {
    // MKUE p. 6, 3
    let state = exhausted(2);
    let first = state.hand(SOLO)[0];
    let state = done(&state, sideways(first));
    assert!(!can_announce(&state));
    refused(&state, Action::AnnounceEndOfRound);
}

// Spec 023 C4
#[test]
fn announcing_needs_an_empty_deck() {
    // MKUE p. 6, 3.a
    let state = arcanum_core::new_game(&config(3), &mut config(3).rngs())
        .iter()
        .fold(GameState::default(), apply);
    assert!(!state.deck(SOLO).is_empty());
    assert!(!can_announce(&state));
    refused(&state, Action::AnnounceEndOfRound);
}

// Spec 023 C5
#[test]
fn end_of_round_events_come_in_rule_order() {
    // MKUE p. 4, 2
    let state = exhausted(0);
    let events = events_of(&state, &Action::AnnounceEndOfRound, 0);
    assert_eq!(events[0], Event::RoundEnded { round: 1 });
    assert_eq!(
        events[1],
        Event::RoundStarted {
            round: 2,
            time_of_day: TimeOfDay::Night
        }
    );
    assert!(matches!(events[2], Event::SourceRolled { .. }));
    assert!(matches!(
        events[3],
        Event::DeckShuffled { player: SOLO, .. }
    ));
    assert_eq!(events.len(), 4 + 5 + 1);
    assert!(
        events[4..9]
            .iter()
            .all(|event| matches!(event, Event::CardDrawn { player: SOLO, .. }))
    );
    assert_eq!(
        events[9],
        Event::TurnStarted {
            player: SOLO,
            turn: 1
        }
    );
}

// --- Nouvelle manche ---

// Spec 023 C6
#[test]
fn new_round_flips_day_to_night() {
    // MKUE p. 3 ; p. 4, 2.a
    let state = exhausted(0);
    assert_eq!(state.round(), 1);
    assert_eq!(state.time_of_day(), TimeOfDay::Day);
    let state = announce(&state);
    assert_eq!(state.round(), 2);
    assert_eq!(state.time_of_day(), TimeOfDay::Night);
    assert_eq!(state.turn(), 1);
    let seen = view(&state, SOLO);
    assert_eq!(seen.round, 2);
    assert_eq!(seen.time_of_day, TimeOfDay::Night);
    assert_eq!(seen.turn, 1);
}

/// Vide la main et le deck en défaussant tout, pour annoncer de nouveau.
fn exhaust(state: GameState) -> GameState {
    let mut state = state;
    let deck: Vec<CardId> = state.deck(SOLO).to_vec();
    for card in deck {
        state = apply(state, &Event::CardDrawn { player: SOLO, card });
    }
    let hand: Vec<CardId> = state.hand(SOLO).to_vec();
    for card in hand {
        state = apply(state, &Event::CardDiscarded { player: SOLO, card });
    }
    state
}

// Spec 023 C7
#[test]
fn night_is_followed_by_day() {
    // MKUE p. 4, 2.a
    let state = announce(&exhaust(announce(&exhausted(0))));
    assert_eq!(state.round(), 3);
    assert_eq!(state.time_of_day(), TimeOfDay::Day);
}

// Spec 023 C8
#[test]
fn new_round_reshuffles_every_card() {
    // MKUE p. 4, 2.g
    let state = announce(&exhausted(0));
    assert_eq!(state.deck(SOLO).len(), 11);
    assert_eq!(state.hand(SOLO).len(), 5);
    assert!(state.discard_pile(SOLO).is_empty());
    assert!(state.play_area(SOLO).is_empty());
    assert_eq!(all_cards(&state), starting_cards());
}

// Spec 023 C9
#[test]
fn cards_in_hand_are_reshuffled_too() {
    // MKUE p. 4, 2.g
    let before = exhausted(2);
    let kept: Vec<CardId> = before.hand(SOLO).to_vec();
    let state = announce(&before);
    for card in kept {
        assert!(state.deck(SOLO).contains(&card) || state.hand(SOLO).contains(&card));
    }
    assert_eq!(all_cards(&state), starting_cards());
}

// Spec 023 C10
#[test]
fn new_round_resets_the_source() {
    // MKUE p. 4, 2.b ; p. 3
    let state = exhausted(0);
    for seed in 0..20 {
        let events = events_of(&state, &Action::AnnounceEndOfRound, seed);
        let Event::SourceRolled { dice } = &events[2] else {
            panic!("la Source est relancée")
        };
        assert_eq!(dice.len(), 3);
        assert!(dice.iter().filter(|die| die.is_basic()).count() >= basic_dice_needed(3));
        assert_eq!(events, events_of(&state, &Action::AnnounceEndOfRound, seed));
    }
}

// Spec 023 C11
#[test]
fn night_rules_apply_in_the_new_round() {
    // MKUE p. 5, 3.c ; p. 7
    let state = announce(&exhausted(0));
    let state = apply(
        state,
        &Event::SourceRolled {
            dice: vec![
                ManaColor::Gold,
                ManaColor::Basic(BasicColor::Red),
                ManaColor::Black,
            ],
        },
    );
    assert_eq!(state.time_of_day(), TimeOfDay::Night);
    assert!(state.source()[0].is_depleted(state.time_of_day()));
    assert!(!state.source()[2].is_depleted(state.time_of_day()));
    // La tuile de départ A a une forêt voisine du portail : 5 points de nuit.
    let forest = Hex::ORIGIN
        .neighbors()
        .into_iter()
        .find(|&hex| {
            state
                .map()
                .space(hex)
                .is_some_and(|space| space.terrain == arcanum_core::Terrain::Forest)
        })
        .expect("une forêt voisine du portail");
    assert_eq!(
        arcanum_core::move_cost(
            state.map().space(forest).expect("case"),
            state.time_of_day()
        ),
        arcanum_core::MoveCost::Points(5)
    );
}

// Spec 023 C12
#[test]
fn new_round_keeps_crystals_figure_and_map() {
    let before = apply(
        exhausted(0),
        &Event::CrystalGained {
            player: SOLO,
            color: BasicColor::Blue,
        },
    );
    let after = announce(&before);
    assert_eq!(after.inventory(SOLO), before.inventory(SOLO));
    assert_eq!(after.figure(SOLO), before.figure(SOLO));
    assert_eq!(after.map(), before.map());
    assert_eq!(after.tile_deck(), before.tile_deck());
}

// Spec 023 C13
#[test]
fn announcing_forfeits_the_mine() {
    // MKUE p. 6, 4
    let state = exhausted(0);
    let mine = east()
        .hexes()
        .into_iter()
        .find(|&hex| {
            state.map().space(hex).and_then(|space| space.feature)
                == Some(Feature::Site(Site::Mine(BasicColor::Green)))
        })
        .expect("la tuile de campagne 2 a une mine verte");
    let state = apply(
        state,
        &Event::HeroFigurePlaced {
            player: SOLO,
            hex: mine,
        },
    );
    let events = events_of(&state, &Action::AnnounceEndOfRound, 0);
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::CrystalGained { .. }))
    );
}

// --- Fin de partie ---

// Spec 023 C14
#[test]
fn last_round_ends_the_game() {
    // MKUE p. 4, 5.a
    let state = announce(&exhausted_with(two_rounds(), 0));
    assert_eq!(state.round(), 2);
    let state = exhaust(state);
    let events = events_of(&state, &Action::AnnounceEndOfRound, 0);
    // Spec 063 (effets sur la 023) : la fin de partie produit aussi `GameEnded { LastRound }`.
    assert_eq!(
        events,
        vec![
            Event::RoundEnded { round: 2 },
            Event::GameEnded {
                reason: arcanum_core::GameEndReason::LastRound
            }
        ]
    );
    let state = events.iter().fold(state, apply);
    assert_eq!(pending_decision(&state), None);
    assert!(legal_actions(&state, SOLO).is_empty());
}

// Spec 023 C14, avec des cartes en main
#[test]
fn last_round_ends_the_game_even_with_cards_in_hand() {
    let state = exhausted_with(two_rounds(), 0);
    let state = announce(&state);
    // Deck vide, 2 cartes en main : on annonce sans les jouer.
    let mut state = state;
    let deck: Vec<CardId> = state.deck(SOLO).to_vec();
    for card in deck {
        state = apply(state, &Event::CardDrawn { player: SOLO, card });
    }
    let hand: Vec<CardId> = state.hand(SOLO).to_vec();
    for card in hand.into_iter().skip(2) {
        state = apply(state, &Event::CardDiscarded { player: SOLO, card });
    }
    assert_eq!(state.hand(SOLO).len(), 2);
    let state = announce(&state);
    assert_eq!(state.hand(SOLO).len(), 2);
    assert_eq!(pending_decision(&state), None);
}

// Spec 023 C15
#[test]
fn sandbox_lasts_four_rounds() {
    // MKUE p. 18
    assert_eq!(Scenario::sandbox().rounds(), 4);
    let mut state = exhausted(0);
    let mut times = vec![state.time_of_day()];
    for _ in 0..3 {
        state = exhaust(announce(&state));
        times.push(state.time_of_day());
    }
    assert_eq!(
        times,
        vec![
            TimeOfDay::Day,
            TimeOfDay::Night,
            TimeOfDay::Day,
            TimeOfDay::Night
        ]
    );
    assert_eq!(state.round(), 4);
    let state = announce(&state);
    assert_eq!(pending_decision(&state), None);
}

// Spec 023 C16
#[test]
fn scenario_needs_at_least_one_round() {
    let recipe = TileDeckRecipe {
        countryside: 7,
        countryside_order: CountrysideOrder::Random,
        core_city: 2,
        core_non_city: 2,
    };
    assert_eq!(
        Scenario::new(
            ScenarioId::Sandbox,
            MapShape::Wedge,
            recipe,
            0,
            0,
            ScenarioEnd::RoundLimit,
            FinalScoring::None,
        ),
        Err(ScenarioError::NoRound)
    );
    assert!(
        Scenario::new(
            ScenarioId::Sandbox,
            MapShape::Wedge,
            recipe,
            1,
            0,
            ScenarioEnd::RoundLimit,
            FinalScoring::None,
        )
        .is_ok()
    );
}

// --- Journal et boucle ---

// Spec 023 C17
#[test]
fn end_of_round_hides_only_cards() {
    assert_eq!(
        filter_event(&Event::RoundEnded { round: 1 }, OTHER),
        PlayerEvent::RoundEnded { round: 1 }
    );
    assert_eq!(
        filter_event(
            &Event::RoundStarted {
                round: 2,
                time_of_day: TimeOfDay::Night
            },
            OTHER
        ),
        PlayerEvent::RoundStarted {
            round: 2,
            time_of_day: TimeOfDay::Night
        }
    );
    let events = events_of(&exhausted(0), &Action::AnnounceEndOfRound, 0);
    for event in &events {
        match (event, filter_event(event, OTHER)) {
            (Event::SourceRolled { dice }, PlayerEvent::SourceRolled { dice: seen }) => {
                assert_eq!(&seen, dice)
            }
            (Event::DeckShuffled { order, .. }, PlayerEvent::DeckShuffled { count, .. }) => {
                assert_eq!(count, order.len())
            }
            (Event::CardDrawn { .. }, PlayerEvent::CardDrawn { card, .. }) => {
                assert_eq!(card, None)
            }
            _ => {}
        }
    }
}

// Spec 023 C18
#[test]
fn replaying_log_rebuilds_rounds() {
    let mut runner = GameRunner::new(config(21), vec![Box::new(FirstChoice::default())]);
    for _ in 0..5_000 {
        if runner.state().round() >= 3 {
            break;
        }
        runner.step().expect("la partie avance");
    }
    assert!(runner.state().round() >= 3, "deux fins de manche jouées");
    assert_eq!(&runner.log().replay(), runner.state());
}

// Spec 023 C19
#[test]
fn runner_plays_several_rounds() {
    let mut runner = GameRunner::new(config(13), vec![Box::new(FirstChoice::default())]);
    let mut announced = 0;
    for _ in 0..5_000 {
        if runner.state().round() >= 3 {
            break;
        }
        let state = runner.state().clone();
        // Le premier choix ne défie jamais (voir `FirstChoice`).
        let first = legal_actions(&state, SOLO)
            .into_iter()
            .find(|action| !matches!(action, Action::Challenge { .. }));
        if first == Some(Action::AnnounceEndOfRound) {
            // Le joueur « premier choix » n'annonce que deck et main vides.
            assert!(state.deck(SOLO).is_empty());
            assert!(state.hand(SOLO).is_empty());
            announced += 1;
        }
        assert_ne!(runner.step(), Ok(StepOutcome::Finished));
    }
    assert_eq!(runner.state().round(), 3);
    assert_eq!(announced, 2);
}

// --- Cas limites ---

// Spec 023 L3
#[test]
fn source_reset_frees_taken_dice() {
    let state = apply(
        exhausted(2),
        &Event::DieTaken {
            player: SOLO,
            die: 0,
            color: ManaColor::Basic(BasicColor::Green),
        },
    );
    assert_eq!(state.dice_taken(SOLO), &[0]);
    let state = apply(
        state,
        &Event::SourceRolled {
            dice: vec![ManaColor::Gold; 3],
        },
    );
    assert!(state.dice_taken(SOLO).is_empty());
}

// Spec 038 C6 — MKUE p. 4, 2.b et 2.g : la Source et le deck de la nouvelle manche sont tirés
// chacun dans son générateur ; la pioche de tuiles et les piles d'ennemis n'avancent pas.
#[test]
fn new_round_draws_source_and_deck_from_their_own_generators() {
    use rand::seq::SliceRandom;

    let state = exhausted(2);
    let mut rngs = config(4).rngs();
    let events = decide(&state, SOLO, &Action::AnnounceEndOfRound, &mut rngs).expect("légale");

    let fresh = |usage| GameRngs::new(4).rng(usage).clone();
    assert!(events.contains(&Event::SourceRolled {
        dice: roll_source(source_size(1), &mut fresh(RngUsage::Source)),
    }));
    let mut order: Vec<CardId> = state
        .deck(SOLO)
        .iter()
        .chain(state.discard_pile(SOLO))
        .chain(state.hand(SOLO))
        .copied()
        .collect();
    order.shuffle(&mut fresh(RngUsage::Deck(SOLO)));
    assert!(events.contains(&Event::DeckShuffled {
        player: SOLO,
        order
    }));
    assert_eq!(
        rngs.positions().into_keys().collect::<Vec<_>>(),
        [RngUsage::Source, RngUsage::Deck(SOLO)]
    );
}
