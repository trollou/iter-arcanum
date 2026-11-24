//! Scénarios de la spec 024 (sauvegarder et charger), partie B : arcanum-persistance.

use std::cell::RefCell;
use std::fs;
use std::path::Path;
use std::rc::Rc;

use arcanum_core::{
    Action, Event, GameConfig, GameRunner, Hero, Player, PlayerView, Scenario, ScenarioId,
    StepOutcome, UndoStatus,
};
use arcanum_persistance::{AUTOSAVE_NAME, LoadError, SaveGame, list, load, save, save_auto};
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;

fn config(seed: u64) -> GameConfig {
    GameConfig {
        seed,
        scenario: Scenario::sandbox(),
        hero: Hero::Arythea,
        dummy_hero: Default::default(),
    }
}

/// Joueur au hasard dont le générateur est partagé : le test peut le remettre à une graine.
#[derive(Clone)]
struct RandomPlayer {
    rng: Rc<RefCell<ChaCha8Rng>>,
    count: usize,
}

impl RandomPlayer {
    fn new(seed: u64) -> Self {
        Self {
            rng: Rc::new(RefCell::new(ChaCha8Rng::seed_from_u64(seed))),
            count: 0,
        }
    }

    fn reseed(&self, seed: u64) {
        *self.rng.borrow_mut() = ChaCha8Rng::seed_from_u64(seed);
    }
}

impl Player for RandomPlayer {
    fn request(&mut self, _view: &PlayerView, legal_actions: &[Action]) {
        self.count = legal_actions.len();
    }

    fn poll(&mut self) -> Option<usize> {
        Some(self.rng.borrow_mut().random_range(0..self.count))
    }
}

/// Avance d'au plus `decisions` décisions ; s'arrête si la partie est finie.
fn advance(runner: &mut GameRunner, decisions: usize) {
    let mut done = 0;
    while done < decisions {
        match runner.step().expect("pas sans erreur") {
            StepOutcome::Advanced => done += 1,
            StepOutcome::Waiting { .. } => {}
            StepOutcome::Finished => return,
        }
    }
}

fn played(seed: u64, decisions: usize) -> (GameRunner, RandomPlayer) {
    let player = RandomPlayer::new(seed);
    let mut runner = GameRunner::new(config(seed), vec![Box::new(player.clone())]);
    advance(&mut runner, decisions);
    (runner, player)
}

fn saved(runner: &GameRunner) -> SaveGame {
    SaveGame::from_runner(runner, "2026-10-27T18:00:00+01:00").expect("partie commencée")
}

fn count(runner: &GameRunner, wanted: fn(&Event) -> bool) -> usize {
    runner
        .log()
        .entries()
        .iter()
        .filter(|entry| wanted(&entry.event))
        .count()
}

// Spec 024 C13 — ADR 0001
#[test]
fn reloaded_game_draws_same_randomness() {
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let (mut straight, player) = played(4, 20);
    let path = save(dir.path(), "partie", &saved(&straight)).expect("sauvegarde écrite");

    let reloaded_player = RandomPlayer::new(99);
    let mut reloaded = load(&path)
        .expect("sauvegarde lue")
        .resume(vec![Box::new(reloaded_player)])
        .expect("partie reprise");
    player.reseed(99);
    let before = (
        count(&straight, |e| matches!(e, Event::DieReturned { .. })),
        count(&straight, |e| matches!(e, Event::RoundStarted { .. })),
    );

    advance(&mut straight, 400);
    advance(&mut reloaded, 400);

    assert_eq!(reloaded.log(), straight.log());
    assert_eq!(reloaded.state(), straight.state());
    // La suite a bien tiré du hasard : dés relancés et nouvelle manche (mélange).
    assert!(count(&straight, |e| matches!(e, Event::DieReturned { .. })) > before.0);
    assert!(count(&straight, |e| matches!(e, Event::RoundStarted { .. })) > before.1);
}

/// Partie au milieu d'un tour : au moins une décision reprenable.
fn mid_turn(seed: u64) -> GameRunner {
    let player = RandomPlayer::new(seed);
    let mut runner = GameRunner::new(config(seed), vec![Box::new(player)]);
    for _ in 0..200 {
        advance(&mut runner, 1);
        if runner.undo_status() == UndoStatus::Possible {
            return runner;
        }
    }
    panic!("aucune décision reprenable");
}

// Spec 024 C14
#[test]
fn undo_works_after_reload() {
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let mut original = mid_turn(5);
    let path = save(dir.path(), "partie", &saved(&original)).expect("sauvegarde écrite");
    let mut reloaded = load(&path)
        .expect("sauvegarde lue")
        .resume(vec![Box::new(RandomPlayer::new(0))])
        .expect("partie reprise");

    loop {
        assert_eq!(reloaded.undo_status(), original.undo_status());
        let (a, b) = (original.undo(), reloaded.undo());
        assert_eq!(a, b);
        assert_eq!(reloaded.state(), original.state());
        if a.is_err() {
            break;
        }
    }
}

// Spec 024 C15
#[test]
fn save_load_round_trip() {
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let (runner, _) = played(6, 30);
    let game = saved(&runner);
    let path = save(dir.path(), "partie", &game).expect("sauvegarde écrite");

    assert_eq!(load(&path), Ok(game));
}

// Spec 024 C16
#[test]
fn reloaded_state_equals_saved_state() {
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let (runner, _) = played(7, 30);
    let path = save(dir.path(), "partie", &saved(&runner)).expect("sauvegarde écrite");
    let reloaded = load(&path)
        .expect("sauvegarde lue")
        .resume(vec![Box::new(RandomPlayer::new(0))])
        .expect("partie reprise");

    assert_eq!(reloaded.state(), runner.state());
    assert_eq!(reloaded.rng_positions(), runner.rng_positions());
}

// Spec 024 C17
#[test]
fn summary_describes_game() {
    let (runner, _) = played(8, 30);
    let summary = saved(&runner).summary;

    assert_eq!(summary.saved_at, "2026-10-27T18:00:00+01:00");
    assert_eq!(summary.scenario, ScenarioId::Sandbox);
    assert_eq!(summary.hero, Hero::Arythea);
    assert_eq!(summary.round, runner.state().round());
    assert_eq!(summary.round_limit, 4);
    assert_eq!(summary.time_of_day, runner.state().time_of_day());
    assert_eq!(summary.turn, runner.state().turn());
}

// Spec 024 C18
#[test]
fn same_second_saves_get_suffix() {
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let (runner, _) = played(9, 5);
    let game = saved(&runner);
    let stem = "2026-10-27_18-00-00";

    let first = save(dir.path(), stem, &game).expect("première sauvegarde");
    let second = save(dir.path(), stem, &game).expect("deuxième sauvegarde");
    let third = save(dir.path(), stem, &game).expect("troisième sauvegarde");

    let name = |path: &Path| path.file_name().and_then(|n| n.to_str()).map(String::from);
    assert_eq!(name(&first).as_deref(), Some("2026-10-27_18-00-00.json"));
    assert_eq!(name(&second).as_deref(), Some("2026-10-27_18-00-00-2.json"));
    assert_eq!(name(&third).as_deref(), Some("2026-10-27_18-00-00-3.json"));
}

fn with_date(game: &SaveGame, saved_at: &str) -> SaveGame {
    let mut game = game.clone();
    game.summary.saved_at = saved_at.to_string();
    game
}

// Spec 024 C19
#[test]
fn list_sorts_and_reports_unreadable() {
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let (runner, _) = played(10, 5);
    let game = saved(&runner);
    save(
        dir.path(),
        "a",
        &with_date(&game, "2026-10-25T10:00:00+01:00"),
    )
    .expect("écrite");
    save(
        dir.path(),
        "b",
        &with_date(&game, "2026-10-27T10:00:00+01:00"),
    )
    .expect("écrite");
    save_auto(dir.path(), &with_date(&game, "2026-10-20T10:00:00+01:00")).expect("écrite");
    fs::write(dir.path().join("cassée.json"), "pas du JSON").expect("écrite");
    fs::write(dir.path().join("notes.txt"), "pas une sauvegarde").expect("écrite");

    let entries = list(dir.path());
    let names: Vec<String> = entries
        .iter()
        .filter_map(|entry| {
            entry
                .path
                .file_stem()
                .and_then(|n| n.to_str())
                .map(String::from)
        })
        .collect();

    assert_eq!(names, [AUTOSAVE_NAME, "b", "a", "cassée"]);
    assert!(entries[0].is_auto());
    assert!(entries[1].summary.is_ok());
    assert!(matches!(entries[3].summary, Err(LoadError::Invalid(_))));
}

// Spec 024 C20 et L5
#[test]
fn other_version_is_refused() {
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let (runner, _) = played(11, 5);
    let mut game = saved(&runner);
    game.version = arcanum_core::EVENT_FORMAT_VERSION + 1;
    let path = save(dir.path(), "future", &game).expect("écrite");

    assert_eq!(
        load(&path),
        Err(LoadError::OtherVersion {
            found: arcanum_core::EVENT_FORMAT_VERSION + 1
        })
    );
}

// Spec 024 C21 et L5 : forme d'un événement changée depuis la sauvegarde.
#[test]
fn outdated_event_shape_is_refused() {
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let (runner, _) = played(12, 5);
    let path = save(dir.path(), "ancienne", &saved(&runner)).expect("écrite");
    let text = fs::read_to_string(&path).expect("lue");
    fs::write(&path, text.replace("\"CardDrawn\"", "\"CardPicked\"")).expect("écrite");

    assert!(matches!(load(&path), Err(LoadError::Invalid(_))));
}

// Spec 024 L3 : fichier absent.
#[test]
fn missing_file_is_refused() {
    let dir = tempfile::tempdir().expect("dossier temporaire");
    assert!(matches!(
        load(&dir.path().join("absente.json")),
        Err(LoadError::Io(_))
    ));
}

// Spec 024 L4 : journal qui ne commence pas par GameStarted.
#[test]
fn log_without_game_start_is_refused() {
    let (runner, _) = played(13, 5);
    let mut game = saved(&runner);
    game.events.remove(0);

    assert!(matches!(
        game.resume(vec![Box::new(RandomPlayer::new(0))]),
        Err(LoadError::Resume(_))
    ));
}

// Spec 024 C22 — ADR 0004
#[test]
fn save_path_with_spaces_and_accents() {
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let folder = dir.path().join("Mes parties été");
    fs::create_dir_all(&folder).expect("dossier créé");
    let (runner, _) = played(14, 5);
    let game = saved(&runner);

    let path = save(&folder, "partie du soir", &game).expect("sauvegarde écrite");
    let text = fs::read_to_string(&path).expect("UTF-8 lisible");

    assert!(text.contains("\"version\": 1"));
    assert_eq!(load(&path), Ok(game));
    assert_eq!(list(&folder).len(), 1);
}

// Spec 024 : la sauvegarde automatique est remplacée à chaque fois.
#[test]
fn autosave_is_overwritten() {
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let (mut runner, _) = played(15, 3);
    save_auto(dir.path(), &saved(&runner)).expect("écrite");
    advance(&mut runner, 10);
    let path = save_auto(dir.path(), &saved(&runner)).expect("écrite");

    assert_eq!(list(dir.path()).len(), 1);
    assert_eq!(load(&path), Ok(saved(&runner)));
    assert_eq!(
        load(&path).map(|game| game.summary.time_of_day),
        Ok(runner.state().time_of_day())
    );
}

// Spec 038 : une position par usage déjà tiré, rangée sous le nom de l'usage.
#[test]
fn save_writes_one_position_per_usage() {
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let (runner, _) = played(13, 5);
    let path = save(dir.path(), "partie", &saved(&runner)).expect("écrite");
    let value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&path).expect("lue")).expect("JSON");
    let positions = value["rng_positions"]
        .as_object()
        .expect("objet des positions");

    for key in [
        "tiles",
        "source",
        "deck/0",
        "enemy_pile/green",
        "enemy_pile/red",
    ] {
        assert!(positions[key].is_string(), "{key}");
    }
    assert!(value.get("rng_position").is_none());
}

// Spec 038 L2 : une sauvegarde d'avant (une seule position) est refusée, sans migration.
#[test]
fn single_position_save_is_refused() {
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let (runner, _) = played(14, 5);
    let path = save(dir.path(), "ancienne", &saved(&runner)).expect("écrite");
    let mut value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&path).expect("lue")).expect("JSON");
    let object = value.as_object_mut().expect("objet");
    object.remove("rng_positions");
    object.insert("rng_position".to_string(), "123".into());
    fs::write(&path, value.to_string()).expect("écrite");

    assert!(matches!(load(&path), Err(LoadError::Invalid(_))));
}

// Spec 038 L3 : un usage inconnu est refusé, on ne le devine pas.
#[test]
fn unknown_usage_in_save_is_refused() {
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let (runner, _) = played(15, 5);
    let path = save(dir.path(), "inconnu", &saved(&runner)).expect("écrite");
    let mut value: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&path).expect("lue")).expect("JSON");
    value["rng_positions"]["inconnu"] = "3".into();
    fs::write(&path, value.to_string()).expect("écrite");

    assert!(matches!(load(&path), Err(LoadError::Invalid(_))));
}
