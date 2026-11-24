//! Scénarios de la spec 038 (un générateur par usage, ADR 0007).

use std::collections::BTreeMap;

use arcanum_core::{
    Action, CommonDeck, EnemyColor, Event, GameConfig, GameContent, GameRng, GameRngs, GameState,
    Hero, PlayerId, RngUsage, Scenario, apply, decide, legal_actions, new_game, roll_die,
    roll_source, source_size,
};
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};

const SOLO: PlayerId = PlayerId::SOLO;

fn config(seed: u64) -> GameConfig {
    GameConfig {
        seed,
        scenario: Scenario::sandbox(),
        hero: Hero::Tovak,
        dummy_hero: Default::default(),
    }
}

/// Les usages de la spec 038, tous ceux qui existent aujourd'hui.
fn usages() -> Vec<RngUsage> {
    let mut usages = vec![
        RngUsage::Tiles,
        RngUsage::Source,
        RngUsage::Deck(SOLO),
        RngUsage::Deck(PlayerId(1)),
    ];
    usages.extend(EnemyColor::ALL.map(RngUsage::EnemyPile));
    usages
}

fn draws(rngs: &mut GameRngs, usage: RngUsage, count: usize) -> Vec<u64> {
    (0..count).map(|_| rngs.rng(usage).next_u64()).collect()
}

/// Générateur neuf de l'usage, pour la graine donnée.
fn fresh(seed: u64, usage: RngUsage) -> GameRng {
    GameRngs::new(seed).rng(usage).clone()
}

// Spec 038 C1
#[test]
fn drawing_in_one_usage_does_not_shift_another() {
    for drawn in usages() {
        for other in usages().into_iter().filter(|&usage| usage != drawn) {
            let mut busy = GameRngs::new(11);
            draws(&mut busy, drawn, 100);
            let mut idle = GameRngs::new(11);
            assert_eq!(
                draws(&mut busy, other, 10),
                draws(&mut idle, other, 10),
                "{drawn:?} décale {other:?}"
            );
        }
    }
}

// Spec 038 C2
#[test]
fn each_usage_has_its_own_sequence() {
    let sequences: Vec<Vec<u64>> = usages()
        .into_iter()
        .map(|usage| draws(&mut GameRngs::new(4), usage, 4))
        .collect();
    for (i, a) in sequences.iter().enumerate() {
        for b in &sequences[i + 1..] {
            assert_ne!(a, b);
        }
    }
    for usage in usages() {
        assert_eq!(
            draws(&mut GameRngs::new(4), usage, 4),
            draws(&mut GameRngs::new(4), usage, 4)
        );
    }
}

// Spec 038 C3 : les numéros de flux ne changent jamais (ADR 0007).
#[test]
fn usage_streams_are_frozen() {
    let expected = [
        (RngUsage::Tiles, 1),
        (RngUsage::Source, 2),
        (RngUsage::Deck(SOLO), 100),
        (RngUsage::Deck(PlayerId(1)), 101),
        (RngUsage::EnemyPile(EnemyColor::Green), 200),
        (RngUsage::EnemyPile(EnemyColor::Grey), 201),
        (RngUsage::EnemyPile(EnemyColor::Violet), 202),
        (RngUsage::EnemyPile(EnemyColor::Brown), 203),
        (RngUsage::EnemyPile(EnemyColor::Red), 204),
        (RngUsage::EnemyPile(EnemyColor::White), 205),
    ];
    for (usage, stream) in expected {
        assert_eq!(usage.stream(), stream, "{usage:?}");
        // Le générateur est ChaCha8 sur ce flux, initialisé par la graine de la partie.
        let mut derived = GameRng::seed_from_u64(7);
        derived.set_stream(stream);
        assert_eq!(fresh(7, usage), derived, "{usage:?}");
    }
}

fn setup(rngs: &mut GameRngs) -> Vec<Event> {
    new_game(&config(5), rngs)
}

fn tile_order(events: &[Event]) -> &Event {
    events
        .iter()
        .find(|event| matches!(event, Event::TileDeckFormed { .. }))
        .expect("pioche de tuiles formée")
}

// Spec 038 C4
#[test]
fn setup_draws_each_usage_from_a_fresh_generator() {
    let events = setup(&mut config(5).rngs());

    let mut deck = Hero::Tovak.starting_deck();
    deck.shuffle(&mut fresh(5, RngUsage::Deck(SOLO)));
    assert!(events.contains(&Event::DeckShuffled {
        player: SOLO,
        order: deck,
    }));
    assert!(events.contains(&Event::SourceRolled {
        dice: roll_source(source_size(1), &mut fresh(5, RngUsage::Source)),
    }));
    for color in EnemyColor::ALL {
        let mut order = GameContent::embedded().enemy_tokens(color);
        order.shuffle(&mut fresh(5, RngUsage::EnemyPile(color)));
        assert!(
            events.contains(&Event::EnemyPileShuffled { color, order }),
            "{color:?}"
        );
    }
    // La pioche de tuiles ne dépend que de son générateur : avancer tous les autres n'y change rien.
    let mut shifted = config(5).rngs();
    for usage in usages().into_iter().filter(|&u| u != RngUsage::Tiles) {
        draws(&mut shifted, usage, 50);
    }
    assert_eq!(tile_order(&setup(&mut shifted)), tile_order(&events));
}

/// Partie mise en place avec la graine 5, et ses générateurs juste après.
fn started() -> (GameState, GameRngs) {
    let mut rngs = config(5).rngs();
    let state = new_game(&config(5), &mut rngs)
        .iter()
        .fold(GameState::default(), apply);
    (state, rngs)
}

/// Positions de tous les usages sauf `except`.
fn others(rngs: &GameRngs, except: RngUsage) -> BTreeMap<RngUsage, u128> {
    let mut positions = rngs.positions();
    positions.remove(&except);
    positions
}

// Spec 038 C5 — MKUE p. 11, 1 (dés pris relancés en fin de tour)
#[test]
fn end_of_turn_rerolls_dice_from_the_source_generator() {
    let (state, mut rngs) = started();
    let color = state.source()[0];
    let state = apply(
        state,
        &Event::DieTaken {
            player: SOLO,
            die: 0,
            color,
        },
    );
    let end = legal_actions(&state, SOLO)
        .into_iter()
        .find(|action| matches!(action, Action::EndTurn { .. }))
        .expect("finir son tour est légal");
    let mut source = rngs.rng(RngUsage::Source).clone();
    let before = others(&rngs, RngUsage::Source);

    let events = decide(&state, SOLO, &end, &mut rngs).expect("légale");

    assert!(events.contains(&Event::DieReturned {
        player: SOLO,
        die: 0,
        color: roll_die(&mut source),
    }));
    assert_eq!(rngs.rng(RngUsage::Source), &source);
    assert_eq!(others(&rngs, RngUsage::Source), before);
}

// Spec 038 C11
#[test]
fn unused_usage_resumes_from_the_start_of_its_stream() {
    let mut played = GameRngs::new(9);
    draws(&mut played, RngUsage::Source, 3);
    let positions = played.positions();
    assert_eq!(positions.keys().collect::<Vec<_>>(), [&RngUsage::Source]);

    let mut resumed = GameRngs::resume(9, &positions);
    assert_eq!(
        draws(&mut resumed, RngUsage::Source, 5),
        draws(&mut played, RngUsage::Source, 5)
    );
    assert_eq!(
        draws(&mut resumed, RngUsage::Tiles, 5),
        draws(&mut GameRngs::new(9), RngUsage::Tiles, 5)
    );
}

// Spec 038 C11 : un usage tiré a une position, les autres non.
#[test]
fn setup_positions_cover_the_setup_usages_only() {
    let (_, rngs) = started();
    let mut expected = vec![RngUsage::Tiles, RngUsage::Source, RngUsage::Deck(SOLO)];
    expected.extend(EnemyColor::ALL.map(RngUsage::EnemyPile));
    // Spec 050 : les pioches communes, mélangées à la mise en place.
    expected.extend(CommonDeck::ALL.map(RngUsage::CommonDeck));
    // Spec 045 : la pile des jetons de ruines.
    expected.push(RngUsage::RuinPile);
    // Spec 051 : la pile de compétences du héros.
    expected.push(RngUsage::SkillPile(SOLO));
    expected.sort();
    assert_eq!(rngs.positions().into_keys().collect::<Vec<_>>(), expected);
}
