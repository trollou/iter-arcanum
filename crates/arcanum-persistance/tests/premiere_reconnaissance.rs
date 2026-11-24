//! Scénarios de la spec 063 (Première reconnaissance) côté sauvegarde : dernier tour et partie
//! terminée rechargés, sauvegarde d'avant la spec (C21, C22, L6).

use arcanum_core::{
    Action, BasicAction, Event, GameEndReason, GameRunner, LastTurn, Scenario, StepOutcome, TileId,
    TilePosition, final_score, legal_actions, view,
};
use arcanum_persistance::{SaveGame, load, save};

// L'arène des tests d'arcanum-core, partagée plutôt que recopiée (bilan de la spec 032).
#[path = "../../arcanum-core/tests/arene/mod.rs"]
mod arene;
use arene::*;

use BasicAction::*;

const SAVED_AT: &str = "2026-10-30T12:00:00+01:00";

/// Première reconnaissance, figurine au village, la tuile de cité royaume 5 prochaine.
fn city_in_sight() -> Vec<Event> {
    Sandbox {
        scenario: Scenario::first_reconnaissance().without_dummy(),
        figure: Some(VILLAGE),
        hand: vec![
            card(March, 1),
            card(March, 2),
            card(Stamina, 1),
            card(Rage, 1),
            card(Promise, 1),
        ],
        next: vec![TileId::Core(5)],
        ..Sandbox::default()
    }
    .events()
}

fn end_turn(runner: &GameRunner) -> Action {
    legal_actions(runner.state(), SOLO)
        .into_iter()
        .find(|action| matches!(action, Action::EndTurn { .. }))
        .expect("finir le tour est légal")
}

/// La cité révélée, puis la fin de ce tour : le dernier tour commence.
fn in_last_turn() -> (GameRunner, Puppet) {
    let (mut runner, puppet) = runner(city_in_sight());
    play(&mut runner, &puppet, basic(card(March, 1)));
    play(
        &mut runner,
        &puppet,
        Action::Explore {
            position: TilePosition::from_lattice(1, 1),
        },
    );
    let action = end_turn(&runner);
    play(&mut runner, &puppet, action);
    assert_eq!(runner.state().last_turn(), LastTurn::Current);
    (runner, puppet)
}

/// Sauvegarde puis recharge `runner`, avec un nouveau `Puppet`.
fn reloaded(runner: &GameRunner) -> (GameRunner, Puppet) {
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let game = SaveGame::from_runner(runner, SAVED_AT).expect("partie commencée");
    let path = save(dir.path(), "partie", &game).expect("sauvegarde écrite");
    let puppet = Puppet::default();
    let runner = load(&path)
        .expect("sauvegarde lue")
        .resume(vec![Box::new(puppet.clone())])
        .expect("partie reprise");
    (runner, puppet)
}

// C21 — spec 024
#[test]
fn saved_game_keeps_last_turn() {
    let (runner, _) = in_last_turn();
    let (mut runner, puppet) = reloaded(&runner);
    assert_eq!(runner.state().last_turn(), LastTurn::Current);
    play(&mut runner, &puppet, basic(card(Stamina, 1)));
    let action = end_turn(&runner);
    play(&mut runner, &puppet, action);
    assert_eq!(runner.state().game_end(), Some(GameEndReason::CityRevealed));
}

// C22
#[test]
fn saved_finished_game_keeps_score() {
    let (mut runner, puppet) = in_last_turn();
    let hand = runner.state().hand(SOLO)[0];
    play(&mut runner, &puppet, sideways(hand));
    let action = end_turn(&runner);
    play(&mut runner, &puppet, action);
    let score = view(runner.state(), SOLO).final_score;
    assert!(score.is_some());

    let (mut reloaded, _) = reloaded(&runner);
    assert_eq!(
        reloaded.state().game_end(),
        Some(GameEndReason::CityRevealed)
    );
    assert_eq!(view(reloaded.state(), SOLO).final_score, score);
    assert_eq!(
        Some(final_score(reloaded.state(), SOLO)),
        score,
        "même score"
    );
    assert_eq!(reloaded.step(), Ok(StepOutcome::Finished));
}

// L6 — une sauvegarde d'avant la spec 063 n'a pas les trois champs du scénario : elle se
// recharge en bac à sable, sans gloire de tuile ni dernier tour.
#[test]
fn save_without_scenario_rules_loads_as_sandbox() {
    let (runner, _) = runner(
        Sandbox {
            figure: Some(VILLAGE),
            ..Sandbox::default()
        }
        .events(),
    );
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let game = SaveGame::from_runner(&runner, SAVED_AT).expect("partie commencée");
    let path = save(dir.path(), "partie", &game).expect("sauvegarde écrite");

    let mut json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("lecture")).expect("JSON");
    let scenario = json["events"][0]["event"]["GameStarted"]["config"]["scenario"]
        .as_object_mut()
        .expect("le scénario de la mise en place");
    for field in ["fame_per_tile", "end", "scoring"] {
        assert!(scenario.remove(field).is_some(), "{field}");
    }
    std::fs::write(&path, json.to_string()).expect("écriture");

    let reloaded = load(&path)
        .expect("sauvegarde lue")
        .resume(Vec::new())
        .expect("partie reprise");
    assert_eq!(reloaded.state().scenario(), Some(&Scenario::sandbox()));
}
