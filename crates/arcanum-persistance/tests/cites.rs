//! Scénario de la spec 062 (Cités) côté sauvegarde.

use arcanum_core::{CityRule, Event, EventLog, GameRunner, GameState, RngPositions, Scenario};
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
        SaveGame::from_runner(&runner, "2026-11-19T20:00:00+01:00").expect("partie commencée");
    let path = save(dir.path(), name, &game).expect("sauvegarde écrite");
    let reloaded = load(&path)
        .expect("sauvegarde lue")
        .resume(Vec::new())
        .expect("partie reprise");
    (runner.state().clone(), reloaded.state().clone())
}

// C36 — Spec 024 : une cité à moitié prise (un Bouclier, deux défenseurs restants) se recharge à
// l'identique ; une sauvegarde d'avant la spec 062 a ses cités fermées.
#[test]
fn city_survives_save_and_load() {
    let mut events = City {
        figure: NEAR_CITY,
        face_up: true,
        ..City::default()
    }
    .events();
    let defeated = blue_defenders()[0];
    events.push(Event::EnemyTokenDiscarded {
        hex: CITY,
        token: defeated,
    });
    events.push(Event::CityShieldPlaced {
        player: SOLO,
        hex: CITY,
        enemy: defeated,
    });
    let (before, after) = round_trip(events, "cite");
    assert_eq!(after, before);
    let city = after.city(CITY).expect("cité rechargée");
    assert_eq!(city.shields, vec![SOLO]);
    assert!(!city.is_conquered());
    assert_eq!(after.cities().len(), 1);

    // Le scénario d'une sauvegarde d'avant : sans le champ `cities`.
    let mut old = serde_json::to_value(Scenario::sandbox()).expect("scénario en JSON");
    old.as_object_mut()
        .expect("objet")
        .remove("cities")
        .expect("champ des cités");
    let old: Scenario = serde_json::from_value(old).expect("ancien scénario lu");
    assert_eq!(old.cities(), CityRule::Closed);
}
