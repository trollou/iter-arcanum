//! Scénarios de la spec 020 (Source et dés de mana).

use arcanum_core::{
    Action, BasicColor, Event, EventLog, GameConfig, GameRng, GameState, Hero, ManaColor,
    PlayerEvent, PlayerId, RngUsage, Scenario, SidewaysEffect, TimeOfDay, apply, basic_dice_needed,
    decide, filter_event, legal_actions, new_game, roll_die, roll_source, view,
};
use rand::SeedableRng;

const SOLO: PlayerId = PlayerId::SOLO;

fn config(seed: u64) -> GameConfig {
    GameConfig {
        seed,
        scenario: Scenario::sandbox(),
        hero: Hero::Tovak,
        dummy_hero: Default::default(),
    }
}

fn start(config: &GameConfig) -> (GameState, Vec<Event>) {
    let events = new_game(config, &mut config.rngs());
    let state = events.iter().fold(GameState::default(), apply);
    (state, events)
}

fn rng(seed: u64) -> GameRng {
    GameRng::seed_from_u64(seed)
}

fn basic_count(dice: &[ManaColor]) -> usize {
    dice.iter().filter(|die| die.is_basic()).count()
}

// MKUE p. 5, « Using mana » 1 et 2 : six couleurs, une face chacune.
#[test]
fn a_die_has_six_equally_likely_faces() {
    let mut rng = rng(1);
    let rolls: Vec<ManaColor> = (0..10_000).map(|_| roll_die(&mut rng)).collect();
    for color in ManaColor::ALL {
        let share = rolls.iter().filter(|&&rolled| rolled == color).count() as f64 / 10_000.0;
        assert!(
            (0.15..=0.185).contains(&share),
            "{color:?} sort {share} des fois"
        );
    }
    assert_eq!(ManaColor::ALL.len(), 6);
}

// MKUE p. 3 : au moins la moitié des dés montrent une couleur de base.
#[test]
fn at_least_half_of_the_dice_show_basic_colors() {
    for seed in 0..1_000 {
        let dice = roll_source(3, &mut rng(seed));
        assert_eq!(dice.len(), 3);
        assert!(basic_count(&dice) >= 2, "graine {seed} : {dice:?}");
    }
}

// MKUE p. 3 : on ne relance que les dés or et noir, ensemble.
#[test]
fn only_gold_and_black_dice_are_rerolled() {
    // Le lancer commence par un tirage par dé, dans l'ordre : on cherche une graine dont le
    // premier lancer n'a qu'un dé de couleur de base.
    let seed = (0..)
        .find(|&seed| {
            let mut rng = rng(seed);
            let first: Vec<ManaColor> = (0..3).map(|_| roll_die(&mut rng)).collect();
            basic_count(&first) == 1
        })
        .expect("une graine finit par donner un seul dé de couleur de base");
    let mut first_rng = rng(seed);
    let first: Vec<ManaColor> = (0..3).map(|_| roll_die(&mut first_rng)).collect();
    let dice = roll_source(3, &mut rng(seed));
    for (before, after) in first.iter().zip(&dice) {
        if before.is_basic() {
            assert_eq!(before, after, "un dé de couleur de base n'est pas relancé");
        }
    }
    assert!(basic_count(&dice) >= 2);
}

#[test]
fn rolling_is_deterministic() {
    for seed in 0..20 {
        assert_eq!(
            roll_source(3, &mut rng(seed)),
            roll_source(3, &mut rng(seed))
        );
    }
}

// MKUE p. 3 : « au moins la moitié », arrondi au-dessus.
#[test]
fn half_is_rounded_up() {
    assert_eq!(
        (1..=5).map(basic_dice_needed).collect::<Vec<_>>(),
        [1, 1, 2, 2, 3]
    );
    for count in 1..=5 {
        for seed in 0..200 {
            let dice = roll_source(count, &mut rng(seed));
            assert_eq!(dice.len(), count);
            assert!(basic_count(&dice) >= basic_dice_needed(count));
        }
    }
}

// MKUE p. 3 : joueurs réels plus 2, donc 3 dés en solo.
#[test]
fn setup_rolls_three_dice_in_solo() {
    let (state, events) = start(&config(4));
    assert_eq!(state.source().len(), 3);
    // Spec 030 : les piles d'ennemis sont mélangées après la Source, qui n'est plus le dernier
    // événement de la mise en place.
    assert!(events.iter().any(|event| matches!(
        event,
        Event::SourceRolled { dice } if dice.as_slice() == state.source()
    )));
}

// Spec 012 : lancer la Source ne change rien au reste de la mise en place. Depuis la spec 038,
// c'est vrai quel que soit le nombre de lancers : la Source a son propre générateur (les valeurs
// relevées avant la spec 020 ne tiennent plus, la spec 038 ayant changé tous les tirages).
#[test]
fn rolling_the_source_does_not_change_earlier_setup() {
    let config = config(20);
    let (_, events) = start(&config);
    let mut busy = config.rngs();
    for _ in 0..30 {
        roll_die(busy.rng(RngUsage::Source));
    }
    let shifted = new_game(&config, &mut busy);
    let not_source = |events: &[Event]| -> Vec<Event> {
        events
            .iter()
            .filter(|event| !matches!(event, Event::SourceRolled { .. }))
            .cloned()
            .collect()
    };
    assert_eq!(not_source(&shifted), not_source(&events));
    assert_ne!(shifted, events);
}

// MKUE p. 5, 3.c : noir épuisé de jour, or épuisé de nuit.
#[test]
fn black_is_depleted_by_day_and_gold_by_night() {
    assert!(ManaColor::Black.is_depleted(TimeOfDay::Day));
    assert!(!ManaColor::Gold.is_depleted(TimeOfDay::Day));
    assert!(ManaColor::Gold.is_depleted(TimeOfDay::Night));
    assert!(!ManaColor::Black.is_depleted(TimeOfDay::Night));
    for color in [
        BasicColor::Red,
        BasicColor::Blue,
        BasicColor::White,
        BasicColor::Green,
    ] {
        for time in [TimeOfDay::Day, TimeOfDay::Night] {
            assert!(!ManaColor::Basic(color).is_depleted(time));
        }
    }
}

#[test]
fn source_is_public() {
    let (state, events) = start(&config(5));
    assert_eq!(view(&state, SOLO).source, state.source());
    let rolled = events
        .iter()
        .find(|event| matches!(event, Event::SourceRolled { .. }))
        .expect("la mise en place lance la Source");
    let dice = state.source().to_vec();
    assert_eq!(
        filter_event(rolled, PlayerId(1)),
        PlayerEvent::SourceRolled { dice }
    );
}

#[test]
fn replaying_log_rebuilds_source() {
    let config = config(6);
    let (state, events) = start(&config);
    let mut log = EventLog::default();
    for event in events {
        log.push(0, event);
    }
    assert_eq!(log.replay().source(), state.source());
}

// Rien ne prend de dé avant la spec 022.
#[test]
fn source_is_unchanged_by_turns() {
    let config = config(7);
    let (mut state, _) = start(&config);
    let source = state.source().to_vec();
    let mut rng = config.rngs();
    for _ in 0..40 {
        let actions = legal_actions(&state, SOLO);
        // Joue de préférence une carte de côté pour Mouvement, sinon la dernière action
        // (une fin de tour), pour traverser plusieurs tours.
        let action = actions
            .iter()
            .find(|action| {
                matches!(
                    action,
                    Action::PlayCard {
                        play: arcanum_core::CardPlay::Sideways(SidewaysEffect::Move),
                        ..
                    }
                )
            })
            .or(actions.last())
            .cloned();
        // Spec 023 : la fin de manche réinitialise la Source ; on reste dans la manche 1.
        let Some(action) = action.filter(|action| *action != Action::AnnounceEndOfRound) else {
            break;
        };
        let events = decide(&state, SOLO, &action, &mut rng).expect("action légale");
        state = events.iter().fold(state, apply);
        assert_eq!(state.source(), source.as_slice());
    }
    assert!(state.turn() > 1, "la partie a traversé des fins de tour");
}

// Réinitialiser la Source (spec 023) remplace tous les dés.
#[test]
fn source_rolled_replaces_the_source() {
    let (state, _) = start(&config(8));
    let dice = vec![
        ManaColor::Basic(BasicColor::Red),
        ManaColor::Basic(BasicColor::Red),
        ManaColor::Gold,
    ];
    let state = apply(state, &Event::SourceRolled { dice: dice.clone() });
    assert_eq!(state.source(), dice.as_slice());
}
