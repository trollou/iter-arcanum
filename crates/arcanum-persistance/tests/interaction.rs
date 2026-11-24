//! Scénario de la spec 040 (Interaction et réputation) côté sauvegarde.

use arcanum_core::{
    Action, BasicAction, Event, EventLog, GameRunner, GameState, Interaction, Reputation,
    RngPositions, Site, apply,
};
use arcanum_persistance::{SaveGame, load, save};

// L'arène des tests d'arcanum-core, partagée plutôt que recopiée.
#[path = "../../arcanum-core/tests/arene/mod.rs"]
mod arene;
use arene::*;

// Spec 040 C24 — spec 024 : une partie sauvegardée en pleine interaction.
#[test]
fn interaction_survives_save_and_load() {
    let promise = card(BasicAction::Promise, 1);
    let mut events = arena_events(VILLAGE, &[promise, card(BasicAction::March, 1)], &[], &[]);
    events.push(Event::ReputationChanged {
        player: SOLO,
        change: 3,
    });
    for action in [Action::Interact, basic(promise)] {
        let state = events.iter().fold(GameState::default(), apply);
        events.extend(events_of(&state, &action));
    }
    let mut log = EventLog::default();
    for event in events {
        log.push(0, event);
    }
    let runner = GameRunner::resume(log, RngPositions::new(), Vec::new()).expect("journal valide");

    let dir = tempfile::tempdir().expect("dossier temporaire");
    let game =
        SaveGame::from_runner(&runner, "2026-11-03T20:00:00+01:00").expect("partie commencée");
    let path = save(dir.path(), "interaction", &game).expect("sauvegarde écrite");
    let reloaded = load(&path)
        .expect("sauvegarde lue")
        .resume(Vec::new())
        .expect("partie reprise");
    let state = reloaded.state();
    assert_eq!(state.reputation(SOLO), Reputation::new(3));
    assert_eq!(
        state.interaction(SOLO),
        Some(Interaction {
            site: Site::Village,
            bonus: 1,
            city_shields: 0,
        })
    );
    assert_eq!(state.influence_points(SOLO), 3);
    assert_eq!(state, runner.state());
}
