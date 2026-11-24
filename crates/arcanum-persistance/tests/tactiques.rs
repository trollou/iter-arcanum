//! Scénarios de la spec 060 (Tactiques) côté sauvegarde : choix en attente rechargé, sauvegarde
//! d'avant la spec (C39, C40).

use arcanum_core::{Action, BasicAction, Event, GameRunner, Hero, Scenario, legal_actions, view};
use arcanum_persistance::{SaveGame, load, save};

// L'arène des tests d'arcanum-core, partagée plutôt que recopiée (bilan de la spec 032).
#[path = "../../arcanum-core/tests/arene/mod.rs"]
mod arene;
use arene::*;

use BasicAction::*;

const SAVED_AT: &str = "2026-11-18T12:00:00+01:00";

/// Première reconnaissance, le choix de la tactique en attente.
fn tactic_choice() -> Vec<Event> {
    Sandbox {
        scenario: Scenario::first_reconnaissance().without_dummy(),
        hero: Hero::Tovak,
        figure: Some(PLAIN_W),
        hand: vec![
            card(March, 1),
            card(March, 2),
            card(Stamina, 1),
            card(Rage, 1),
            card(Promise, 1),
        ],
        tactic_pending: true,
        ..Sandbox::default()
    }
    .events()
}

/// Sauvegarde `runner`, change le JSON écrit par `edit`, puis recharge.
fn reloaded(runner: &GameRunner, edit: impl FnOnce(&mut serde_json::Value)) -> GameRunner {
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let game = SaveGame::from_runner(runner, SAVED_AT).expect("partie commencée");
    let path = save(dir.path(), "partie", &game).expect("sauvegarde écrite");
    let mut json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("lecture")).expect("JSON");
    edit(&mut json);
    std::fs::write(&path, json.to_string()).expect("écriture");
    load(&path)
        .expect("sauvegarde lue")
        .resume(Vec::new())
        .expect("partie reprise")
}

// Spec 060 C39 — spec 024 : sauvegardée pendant le choix, la partie le reprend, mêmes tactiques.
#[test]
fn save_during_tactic_choice() {
    let (runner, _) = runner(tactic_choice());
    let before = legal_actions(runner.state(), SOLO);
    assert!(matches!(before[0], Action::ChooseTactic { .. }));
    let reloaded = reloaded(&runner, |_| {});
    assert_eq!(legal_actions(reloaded.state(), SOLO), before);
    assert_eq!(
        view(reloaded.state(), SOLO).tactic_choice,
        view(runner.state(), SOLO).tactic_choice
    );
}

// Spec 060 C40 — spec 024 : une sauvegarde d'avant la spec (scénario sans règle des tactiques) se
// recharge et se joue sans tactiques.
#[test]
fn old_save_has_no_tactics() {
    let (runner, _) = runner(tactic_choice());
    let reloaded = reloaded(&runner, |json| {
        let scenario = json["events"][0]["event"]["GameStarted"]["config"]["scenario"]
            .as_object_mut()
            .expect("le scénario de la mise en place");
        assert!(scenario.remove("tactics").is_some());
    });
    let legal = legal_actions(reloaded.state(), SOLO);
    assert!(
        !legal
            .iter()
            .any(|action| matches!(action, Action::ChooseTactic { .. }))
    );
    assert!(legal.contains(&sideways(card(March, 1))));
    assert!(view(reloaded.state(), SOLO).tactic_choice.is_empty());
}
