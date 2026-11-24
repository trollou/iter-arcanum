//! Scénario de la spec 042 (Unités) côté sauvegarde.

use arcanum_core::{
    BasicAction, CardPlay, CommonCard, CommonDeck, Effect, Event, EventLog, GameRunner, Offer,
    OwnedUnit, RngPositions, SidewaysEffect, Unit, UnitCard,
};
use arcanum_persistance::{SaveGame, load, save};

// L'arène des tests d'arcanum-core, partagée plutôt que recopiée.
#[path = "../../arcanum-core/tests/arene/mod.rs"]
mod arene;
use arene::*;

// Spec 042 C37 — spec 024 : deux unités (une épuisée, une blessée) et 2 points de soin.
#[test]
fn units_survive_save_and_load() {
    let peasants = UnitCard::new(Unit::Peasants, 1);
    let guardsmen = UnitCard::new(Unit::UtemGuardsmen, 1);
    let march = card(BasicAction::March, 1);
    let mut events = arena_events(VILLAGE, &[march, card(BasicAction::Rage, 1)], &[], &[]);
    events.push(Event::CommonDeckShuffled {
        deck: CommonDeck::RegularUnits,
        order: vec![CommonCard::Unit(peasants), CommonCard::Unit(guardsmen)],
    });
    for unit in [peasants, guardsmen] {
        events.push(Event::OfferCardDealt {
            offer: Offer::Units,
            card: CommonCard::Unit(unit),
        });
        events.push(Event::UnitRecruited {
            player: SOLO,
            unit,
            cost: 0,
        });
    }
    events.push(Event::UnitActivated {
        enemy: None,
        player: SOLO,
        unit: peasants,
        ability: 2,
        option: 0,
        effect: Effect::Move(2),
    });
    events.push(Event::UnitWounded {
        player: SOLO,
        unit: guardsmen,
        wounds: 1,
    });
    events.push(Event::CardPlayed {
        player: SOLO,
        card: march,
        play: CardPlay::Sideways(SidewaysEffect::Move),
        effect: Effect::Heal(2),
    });
    let mut log = EventLog::default();
    for event in events {
        log.push(0, event);
    }
    let runner = GameRunner::resume(log, RngPositions::new(), Vec::new()).expect("journal valide");

    let dir = tempfile::tempdir().expect("dossier temporaire");
    let game =
        SaveGame::from_runner(&runner, "2026-11-04T02:00:00+01:00").expect("partie commencée");
    let path = save(dir.path(), "unites", &game).expect("sauvegarde écrite");
    let reloaded = load(&path)
        .expect("sauvegarde lue")
        .resume(Vec::new())
        .expect("partie reprise");
    let state = reloaded.state();
    assert_eq!(
        state.units(SOLO),
        [
            OwnedUnit {
                ready: false,
                ..OwnedUnit::recruited(peasants)
            },
            OwnedUnit {
                wounds: 1,
                ..OwnedUnit::recruited(guardsmen)
            },
        ]
    );
    assert_eq!(state.healing_points(SOLO), 2);
    assert_eq!(state, runner.state());
}
