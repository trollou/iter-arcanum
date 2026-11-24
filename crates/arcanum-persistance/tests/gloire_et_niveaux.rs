//! Scénario de la spec 035 (Gloire et niveaux) côté sauvegarde.

use arcanum_core::{Action, BasicAction, EventLog, GameRunner, GameState, RngPositions, apply};
use arcanum_persistance::{SaveGame, load, save};

// L'arène des tests de combat d'arcanum-core, partagée plutôt que recopiée.
#[path = "../../arcanum-core/tests/arene/mod.rs"]
mod arene;
use arene::*;

// Spec 035 C13 — spec 024.
#[test]
fn saved_game_keeps_level() {
    let rage = card(BasicAction::Rage, 1);
    let mut events = arena_events(VILLAGE, &[rage, card(BasicAction::March, 1)], &[], &[]);
    events.extend(progress(8, 1));
    let state = events.iter().fold(GameState::default(), apply);
    events.extend(events_of(&state, &sideways(rage)));
    let state = events.iter().fold(GameState::default(), apply);
    events.extend(events_of(
        &state,
        &Action::EndTurn {
            discard: Vec::new(),
            reclaim: None,
            glade: None,
            steady_tempo: false,
        },
    ));
    let mut log = EventLog::default();
    for event in events {
        log.push(0, event);
    }
    let runner = GameRunner::resume(log, RngPositions::new(), Vec::new()).expect("journal valide");
    assert_eq!(runner.state().level(SOLO), 3);

    let dir = tempfile::tempdir().expect("dossier temporaire");
    let game =
        SaveGame::from_runner(&runner, "2026-10-30T08:00:00+01:00").expect("partie commencée");
    let path = save(dir.path(), "niveau", &game).expect("sauvegarde écrite");
    let reloaded = load(&path)
        .expect("sauvegarde lue")
        .resume(Vec::new())
        .expect("partie reprise");
    let state = reloaded.state();
    assert_eq!(state.level(SOLO), 3);
    assert_eq!(state.armor(SOLO), 3);
    assert_eq!(state.hand_limit(SOLO), 5);
    assert_eq!(state, runner.state());
}
