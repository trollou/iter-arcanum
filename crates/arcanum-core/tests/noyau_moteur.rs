//! Scénarios de la spec 000 (noyau du moteur). Joués sur le jeu jouet jusqu'à la spec 013,
//! qui l'a remplacé par les vraies cartes : chaque test garde le sens du cas qu'il cite.

use arcanum_core::{
    Action, ActionError, BasicAction, CardId, CardPlay, DeedCard, Effect, Event, EventLog,
    GameConfig, GameRunner, GameState, Hero, Player, PlayerEvent, PlayerId, PlayerView,
    RunnerError, Scenario, SidewaysEffect, StepOutcome, apply, decide, legal_actions, new_game,
    pending_decision, simulate, view,
};

mod premier_choix;
use premier_choix::FirstChoice;

const SOLO: PlayerId = PlayerId::SOLO;

fn config(seed: u64) -> GameConfig {
    GameConfig {
        seed,
        scenario: Scenario::sandbox(),
        hero: Hero::Arythea,
        dummy_hero: Default::default(),
    }
}

fn card(action: BasicAction, copy: u16) -> CardId {
    CardId::new(DeedCard::Basic(action), copy)
}

/// Démarre une partie hors `GameRunner` : événements de démarrage, puis état obtenu par `apply`.
fn start(config: &GameConfig) -> (GameState, Vec<Event>) {
    let mut rng = config.rngs();
    let events = new_game(config, &mut rng);
    let state = events.iter().fold(GameState::default(), apply);
    (state, events)
}

fn deck_order(events: &[Event]) -> Vec<CardId> {
    events
        .iter()
        .find_map(|event| match event {
            Event::DeckShuffled { order, .. } => Some(order.clone()),
            _ => None,
        })
        .unwrap_or_default()
}

/// Première carte de la main jouée de côté pour Mouvement 1 : toujours légal.
fn sideways_move(state: &GameState) -> Action {
    Action::PlayCard {
        card: state.hand(SOLO)[0],
        play: CardPlay::Sideways(SidewaysEffect::Move),
    }
}

/// Jouer de côté pour Influence : aucun point de mouvement, donc aucun déplacement (spec 014).
fn sideways_influence(state: &GameState) -> Action {
    Action::PlayCard {
        card: state.hand(SOLO)[0],
        play: CardPlay::Sideways(SidewaysEffect::Influence),
    }
}

/// Joueur qui ne répond jamais (un humain qui réfléchit).
struct NeverAnswers;

impl Player for NeverAnswers {
    fn request(&mut self, _view: &PlayerView, _legal_actions: &[Action]) {}

    fn poll(&mut self) -> Option<usize> {
        None
    }
}

/// Joueur qui renvoie un index hors de la liste des actions légales.
struct OutOfRange;

impl Player for OutOfRange {
    fn request(&mut self, _view: &PlayerView, _legal_actions: &[Action]) {}

    fn poll(&mut self) -> Option<usize> {
        Some(1_000)
    }
}

fn run_to_end(runner: &mut GameRunner) {
    for _ in 0..1_000 {
        if runner.step() == Ok(StepOutcome::Finished) {
            return;
        }
    }
    panic!("la partie ne s'est pas terminée");
}

// Spec 000 C1
#[test]
fn same_seed_gives_same_game() {
    let (first, _) = start(&config(42));
    let (second, _) = start(&config(42));
    assert_eq!(first, second);
    assert_eq!(first.deck(SOLO), second.deck(SOLO));
}

// Spec 000 C2
#[test]
fn different_seeds_give_different_decks() {
    let (_, first) = start(&config(1));
    let (_, second) = start(&config(2));
    assert_ne!(deck_order(&first), deck_order(&second));
}

// Spec 000 C3, devenu spec 013 C4 : le deck mélangé est celui du héros.
#[test]
fn start_shuffles_the_hero_deck() {
    let config = config(7);
    let (_, events) = start(&config);

    assert_eq!(
        events[0],
        Event::GameStarted {
            config: config.clone()
        }
    );
    let mut sorted = deck_order(&events);
    sorted.sort();
    let mut expected = Hero::Arythea.starting_deck();
    expected.sort();
    assert_eq!(sorted, expected);
}

// Spec 000 C4, devenu spec 013 : une décision attend le joueur, qui a des cartes à jouer.
#[test]
fn a_decision_waits_for_the_player() {
    let (state, _) = start(&config(3));
    let decision = pending_decision(&state).expect("une décision doit être en attente");
    assert_eq!(decision.player, SOLO);
    assert!(legal_actions(&state, SOLO).contains(&sideways_move(&state)));
}

// Spec 000 C5, devenu spec 013 C18
#[test]
fn decide_plays_card_without_mutating_state() {
    let config = config(3);
    let (state, _) = start(&config);
    let before = state.clone();
    let action = sideways_move(&state);

    let produced = decide(&state, SOLO, &action, &mut config.rngs());

    assert_eq!(
        produced,
        Ok(vec![Event::CardPlayed {
            player: SOLO,
            card: state.hand(SOLO)[0],
            play: CardPlay::Sideways(SidewaysEffect::Move),
            effect: Effect::Move(1),
        }])
    );
    assert_eq!(state, before);
}

// Spec 000 C6 : une pioche met la carte du dessus en main.
#[test]
fn apply_card_drawn_moves_card_to_hand() {
    let (state, events) = start(&config(3));
    let order = deck_order(&events);

    let state = apply(
        state,
        &Event::CardDrawn {
            player: SOLO,
            card: order[5],
        },
    );

    assert_eq!(state.hand(SOLO), &order[..6]);
    assert_eq!(state.deck(SOLO), &order[6..]);
}

// Spec 000 C7, devenu spec 013 C15 : main vide, plus de décision (spec 014 : et aucun
// déplacement payable ; spec 016 : et le deck vide, la main se repioche à chaque fin de tour ;
// spec 023 : il ne reste alors qu'à annoncer la fin de la manche).
#[test]
fn empty_hand_and_deck_leave_only_end_of_round() {
    let (mut state, _) = start(&config(3));
    while !(state.hand(SOLO).is_empty()
        && state.deck(SOLO).is_empty()
        && state.play_area(SOLO).is_empty())
    {
        let action = if state.hand(SOLO).is_empty() {
            Action::EndTurn {
                discard: vec![],
                reclaim: None,
                glade: None,
                steady_tempo: false,
            }
        } else {
            sideways_influence(&state)
        };
        let events = decide(&state, SOLO, &action, &mut config(3).rngs()).expect("légal");
        state = events.iter().fold(state, apply);
    }
    assert_eq!(
        legal_actions(&state, SOLO),
        vec![Action::AnnounceEndOfRound]
    );
}

// Spec 000 C8
#[test]
fn replaying_log_rebuilds_state() {
    let mut runner = GameRunner::new(config(11), vec![Box::new(FirstChoice::default())]);
    run_to_end(&mut runner);

    assert_eq!(&runner.log().replay(), runner.state());
}

// Spec 000 C9
#[test]
fn view_hides_deck_order() {
    let started = apply(
        GameState::default(),
        &Event::GameStarted { config: config(0) },
    );
    let (march, rage, swiftness) = (
        card(BasicAction::March, 1),
        card(BasicAction::Rage, 1),
        card(BasicAction::Swiftness, 1),
    );
    let after_draw = |order: Vec<CardId>| {
        let state = apply(
            started.clone(),
            &Event::DeckShuffled {
                player: SOLO,
                order,
            },
        );
        apply(
            state,
            &Event::CardDrawn {
                player: SOLO,
                card: march,
            },
        )
    };
    // Même main, decks restants permutés.
    let one = after_draw(vec![march, rage, swiftness]);
    let other = after_draw(vec![march, swiftness, rage]);
    assert_ne!(one, other);

    assert_eq!(view(&one, SOLO), view(&other, SOLO));
    let seen = view(&one, SOLO);
    assert_eq!(seen.hand, vec![march]);
    assert_eq!(seen.deck_size, 2);
    assert!(seen.has_pending_decision);
}

// Spec 000 C10
#[test]
fn filtered_log_hides_shuffle_order() {
    let runner = GameRunner::new(config(5), vec![Box::new(FirstChoice::default())]);
    let first_card = runner.state().hand(SOLO)[0];

    let filtered = runner.log().filtered_for(SOLO);

    assert!(filtered.contains(&PlayerEvent::DeckShuffled {
        player: SOLO,
        count: 16
    }));
    assert!(filtered.contains(&PlayerEvent::CardDrawn {
        player: SOLO,
        card: Some(first_card)
    }));
}

// Spec 000 C11 : la boucle joue jusqu'au bout ; devenu spec 016 C12
// (`runner_plays_until_deck_and_hand_are_empty`, dans `fin_du_tour.rs`).

// Spec 000 C12
#[test]
fn runner_waits_without_blocking() {
    let mut runner = GameRunner::new(config(13), vec![Box::new(NeverAnswers)]);
    let state = runner.state().clone();
    let logged = runner.log().len();

    assert_eq!(runner.step(), Ok(StepOutcome::Waiting { player: SOLO }));
    assert_eq!(runner.step(), Ok(StepOutcome::Waiting { player: SOLO }));
    assert_eq!(runner.state(), &state);
    assert_eq!(runner.log().len(), logged);
    assert!(runner.decision_log().is_empty());
}

// Spec 000 C13
#[test]
fn runner_records_each_decision() {
    let mut runner = GameRunner::new(config(17), vec![Box::new(FirstChoice::default())]);
    let first_view = view(runner.state(), SOLO);
    let first_legal = legal_actions(runner.state(), SOLO);

    run_to_end(&mut runner);

    let decisions = runner.decision_log().entries();
    assert!(!decisions.is_empty());
    assert_eq!(decisions[0].player, SOLO);
    assert_eq!(decisions[0].view, first_view);
    assert_eq!(decisions[0].legal_actions, first_legal);
    assert_eq!(decisions[0].chosen, 0);
    // Aucune décision n'est enregistrée sans action légale (spec 014 : la dernière peut être
    // un déplacement, main vide).
    let last = decisions.last().expect("au moins une décision");
    assert!(last.chosen < last.legal_actions.len());
}

// Spec 000 C14
#[test]
fn simulation_leaves_original_untouched() {
    let config = config(19);
    let mut runner = GameRunner::new(config.clone(), vec![Box::new(NeverAnswers)]);
    let state = runner.state().clone();
    let logged = runner.log().len();

    let simulated = simulate(
        runner.state(),
        SOLO,
        &sideways_move(runner.state()),
        &mut config.rngs(),
    )
    .expect("jouer de côté doit être légal");

    assert_eq!(simulated.hand(SOLO).len(), 4);
    assert_eq!(simulated.move_points(SOLO), 1);
    assert_eq!(runner.state(), &state);
    assert_eq!(runner.log().len(), logged);
    assert_eq!(runner.step(), Ok(StepOutcome::Waiting { player: SOLO }));
}

// Spec 000 L1, devenu spec 013 L1 : jouer une carte absente de la main est refusé.
#[test]
fn playing_a_card_outside_the_hand_is_refused() {
    let config = config(23);
    let (state, events) = start(&config);
    let in_deck = deck_order(&events)[10];
    assert_eq!(
        decide(
            &state,
            SOLO,
            &Action::PlayCard {
                card: in_deck,
                play: CardPlay::Sideways(SidewaysEffect::Move)
            },
            &mut config.rngs()
        ),
        Err(ActionError::NotLegal)
    );
}

// Spec 000 L2
#[test]
fn action_from_wrong_player_is_refused() {
    let config = config(23);
    let (state, _) = start(&config);
    let intruder = PlayerId(1);
    assert_eq!(
        decide(&state, intruder, &sideways_move(&state), &mut config.rngs()),
        Err(ActionError::WrongPlayer { player: intruder })
    );
}

// Spec 000 L3
#[test]
fn out_of_range_choice_is_refused() {
    let mut runner = GameRunner::new(config(29), vec![Box::new(OutOfRange)]);
    let state = runner.state().clone();
    let logged = runner.log().len();
    let count = legal_actions(&state, SOLO).len();

    assert_eq!(
        runner.step(),
        Err(RunnerError::ChoiceOutOfRange {
            index: 1_000,
            count
        })
    );
    assert_eq!(runner.state(), &state);
    assert_eq!(runner.log().len(), logged);
    assert!(runner.decision_log().is_empty());
}

// Journal : numéros de séquence (spec 000, « Actions et événements ») et de décision ; la version
// du format est écrite une fois par sauvegarde, et reste 1 jusqu'à la fin du J6 (spec 024).
#[test]
fn log_entries_carry_sequence_and_decision() {
    let mut log = EventLog::default();
    let (_, events) = start(&config(37));
    for event in events {
        log.push(0, event);
    }
    let entries = log.entries();
    assert_eq!(entries[0].sequence, 0);
    assert_eq!(entries[1].sequence, 1);
    assert!(entries.iter().all(|entry| entry.decision == 0));
    assert_eq!(arcanum_core::EVENT_FORMAT_VERSION, 1);
}
