//! Scénario de la spec 045 (Ruines) côté sauvegarde.

use arcanum_core::{
    Event, EventLog, GameRunner, GameState, RngPositions, RuinToken, TimeOfDay, UnitCard,
};
use arcanum_persistance::{SaveGame, load, save};

// L'arène des tests d'arcanum-core, partagée plutôt que recopiée.
#[path = "../../arcanum-core/tests/arene/mod.rs"]
mod arene;
use arene::*;

/// La partie de ces événements, sauvegardée puis rechargée : son état avant et après.
fn round_trip(events: Vec<Event>, name: &str) -> (GameState, GameState) {
    let mut log = EventLog::default();
    for event in events {
        log.push(0, event);
    }
    let runner = GameRunner::resume(log, RngPositions::new(), Vec::new()).expect("journal valide");
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let game =
        SaveGame::from_runner(&runner, "2026-11-05T21:00:00+01:00").expect("partie commencée");
    let path = save(dir.path(), name, &game).expect("sauvegarde écrite");
    let reloaded = load(&path)
        .expect("sauvegarde lue")
        .resume(Vec::new())
        .expect("partie reprise");
    (runner.state().clone(), reloaded.state().clone())
}

// C29 — spec 024 : jeton face cachée, autel face visible, jeton d'ennemi resté sur une ruine,
// récompense unité en attente.
#[test]
fn ruins_survive_save_and_load() {
    // Nuit, jeton face cachée.
    let events = Ruins {
        time: TimeOfDay::Night,
        face_up: false,
        ..Ruins::default()
    }
    .events();
    let (before, after) = round_trip(events, "ruine-cachee");
    assert_eq!(after, before);
    assert!(
        after
            .ruins()
            .get(&RUINS)
            .is_some_and(|placed| !placed.face_up)
    );

    // Autel face visible.
    let events = Ruins {
        token: RuinToken::BlueAltar,
        ..Ruins::default()
    }
    .events();
    let (before, after) = round_trip(events, "autel");
    assert_eq!(after, before);
    assert_eq!(
        after
            .ruins()
            .get(&RUINS)
            .map(|placed| (placed.token, placed.face_up)),
        Some((RuinToken::BlueAltar, true))
    );

    // Un jeton d'ennemi resté sur la ruine.
    let events = Ruins {
        token: RuinToken::GreyViolet,
        placed: vec![(gargoyle(), RUINS)],
        ..Ruins::default()
    }
    .events();
    let (before, after) = round_trip(events, "ruine-jeton-reste");
    assert_eq!(after, before);
    assert_eq!(after.enemy_tokens_at(RUINS), [gargoyle()]);

    // Récompense unité en attente du choix.
    let offer = UnitCard::new(arcanum_core::Unit::Foresters, 1);
    let mut events = Ruins {
        token: RuinToken::GreyViolet,
        conquered: true,
        offer: vec![offer],
        ..Ruins::default()
    }
    .events();
    events.push(Event::TurnEnded { player: SOLO });
    let (before, after) = round_trip(events, "ruine-unite");
    assert_eq!(after, before);
    assert_eq!(after.unit_choices(SOLO), 1);
}
