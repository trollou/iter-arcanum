//! Scénarios de la spec 061 (Joueur fantôme) côté sauvegarde : fantôme gardé (C34), sauvegarde
//! d'avant la spec (C35).

use arcanum_core::{DummyHeroChoice, GameConfig, GameRunner, Hero, Player, Scenario, StepOutcome};
use arcanum_persistance::{SaveGame, load, save};

#[path = "../../arcanum-core/tests/premier_choix/mod.rs"]
mod premier_choix;
use premier_choix::FirstChoice;

const SAVED_AT: &str = "2026-11-19T12:00:00+01:00";

fn reco_config(seed: u64, scenario: Scenario) -> GameConfig {
    GameConfig {
        seed,
        scenario,
        hero: Hero::Norowas,
        dummy_hero: DummyHeroChoice::Random,
    }
}

/// Le joueur « premier choix » joue `steps` décisions.
fn played(config: GameConfig, steps: usize) -> GameRunner {
    let players: Vec<Box<dyn Player>> = vec![Box::new(FirstChoice::default())];
    let mut runner = GameRunner::new(config, players);
    for _ in 0..steps {
        if runner.step().expect("la partie avance") == StepOutcome::Finished {
            break;
        }
    }
    runner
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

// Spec 061 C34 — ADR 0001 : rechargée en cours de manche, le fantôme ayant joué, la partie garde
// son deck, sa défausse, ses cristaux, sa tactique et sa pile, et tire la même suite.
#[test]
fn save_keeps_dummy() {
    let runner = played(reco_config(3, Scenario::first_reconnaissance()), 40);
    let dummy = runner.state().dummy().expect("un fantôme");
    assert!(!dummy.discard().is_empty(), "le fantôme a joué");
    let again = reloaded(&runner, |_| {});
    assert_eq!(again.state(), runner.state());
    assert_eq!(again.rng_positions(), runner.rng_positions());
}

// Spec 061 C35 — spec 024 : une sauvegarde d'avant la spec (scénario et configuration sans
// fantôme) se recharge et se joue sans fantôme.
#[test]
fn old_save_loads_without_dummy() {
    let runner = played(
        reco_config(4, Scenario::first_reconnaissance().without_dummy()),
        10,
    );
    let again = reloaded(&runner, |json| {
        let config = json["events"][0]["event"]["GameStarted"]["config"]
            .as_object_mut()
            .expect("la configuration de la mise en place");
        assert!(config.remove("dummy_hero").is_some());
        let scenario = config["scenario"]
            .as_object_mut()
            .expect("le scénario de la mise en place");
        assert!(scenario.remove("dummy").is_some());
    });
    assert!(again.state().dummy().is_none());
    assert_eq!(again.state(), runner.state());
}
