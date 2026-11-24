//! Scénarios de la spec 064 (Conquête en solitaire) côté sauvegarde : dernier tour rechargé (C24),
//! sauvegarde d'avant la spec (C25).

use arcanum_core::{
    Action, BasicAction, BasicColor, Event, GameEndReason, GameRunner, Hex, LastTurn, MapShape,
    Scenario, SpellSet, final_score, legal_actions,
};
use arcanum_persistance::{SaveGame, load, save};

// L'arène des tests d'arcanum-core, partagée plutôt que recopiée (bilan de la spec 032).
#[path = "../../arcanum-core/tests/arene/mod.rs"]
mod arene;
use arene::*;

use BasicAction::*;

const SAVED_AT: &str = "2026-11-19T12:00:00+01:00";

fn end_turn(runner: &GameRunner) -> Action {
    legal_actions(runner.state(), SOLO)
        .into_iter()
        .find(|action| matches!(action, Action::EndTurn { .. }))
        .expect("finir le tour est légal")
}

/// Conquête en solitaire : l'autre cité déjà conquise, la cité bleue sans défenseur prise en y
/// entrant, puis la fin de ce tour : le dernier tour commence.
fn in_last_turn() -> (GameRunner, Puppet) {
    let mut events = City {
        scenario: Scenario::solo_conquest(),
        figure: NEAR_CITY,
        defenders: Vec::new(),
        hand: vec![card(March, 1), card(Stamina, 1), card(Rage, 1)],
        ..City::default()
    }
    .events();
    let other = Hex::new(30, -30);
    events.extend([
        Event::CityPlaced {
            hex: other,
            color: BasicColor::Green,
            level: 5,
        },
        Event::CityConquered {
            hex: other,
            leader: SOLO,
        },
    ]);
    let (mut runner, puppet) = runner(events);
    play(&mut runner, &puppet, basic(card(March, 1)));
    play(&mut runner, &puppet, Action::MoveHero { to: CITY });
    assert_eq!(runner.state().last_turn(), LastTurn::Next);
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

/// Joue la première carte de la main de côté, puis finit le tour.
fn finish(runner: &mut GameRunner, puppet: &Puppet) {
    let hand = runner.state().hand(SOLO)[0];
    play(runner, puppet, sideways(hand));
    let action = end_turn(runner);
    play(runner, puppet, action);
}

// C24 — spec 024 : la partie sauvegardée pendant le dernier tour finit pareil une fois rechargée.
#[test]
fn saved_solo_conquest_keeps_last_turn() {
    let (mut original, puppet) = in_last_turn();
    let (mut reloaded, reloaded_puppet) = reloaded(&original);
    assert_eq!(reloaded.state().last_turn(), LastTurn::Current);
    finish(&mut original, &puppet);
    finish(&mut reloaded, &reloaded_puppet);
    assert_eq!(
        reloaded.state().game_end(),
        Some(GameEndReason::AllCitiesConquered)
    );
    let score = final_score(reloaded.state(), SOLO);
    assert!(
        score
            .conquest
            .is_some_and(|conquest| conquest.all_conquered)
    );
    assert_eq!(score, final_score(original.state(), SOLO), "même score");
}

// C25 — une sauvegarde d'avant la spec 064 n'a pas le champ des sorts : elle se recharge avec
// les 20 sorts et sa forme.
#[test]
fn old_save_keeps_all_spells() {
    for scenario in [Scenario::first_reconnaissance(), Scenario::sandbox()] {
        let (runner, _) = runner(
            Sandbox {
                scenario,
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
        let fields = json["events"][0]["event"]["GameStarted"]["config"]["scenario"]
            .as_object_mut()
            .expect("le scénario de la mise en place");
        assert!(fields.remove("spells").is_some());
        std::fs::write(&path, json.to_string()).expect("écriture");

        let reloaded = load(&path)
            .expect("sauvegarde lue")
            .resume(Vec::new())
            .expect("partie reprise");
        let loaded = reloaded.state().scenario().copied().expect("scénario");
        assert_eq!(loaded, scenario);
        assert_eq!(loaded.spells(), SpellSet::All);
        assert_eq!(loaded.shape(), MapShape::Wedge);
    }
}
