//! Scénarios de la spec 003 (logs techniques) côté arcanum-core : la boucle de partie écrit, les fonctions pures jamais.
//! Aucune règle du livret ici : chaque test cite le cas de la spec.

use std::sync::{Mutex, OnceLock};
use std::thread::ThreadId;

use arcanum_core::{
    Action, GameConfig, GameRunner, GameState, Hero, Player, PlayerId, PlayerView, Scenario,
    StepOutcome, apply, decide, legal_actions, new_game, pending_decision, simulate, view,
};
use log::{Level, LevelFilter, Log, Metadata, Record};

mod premier_choix;
use premier_choix::FirstChoice;

const SOLO: PlayerId = PlayerId::SOLO;

/// Logger global de capture, qui range les lignes par thread pour que les tests restent parallèles.
struct Capture(Mutex<Vec<(ThreadId, Level, String)>>);

impl Log for Capture {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let line = (
            std::thread::current().id(),
            record.level(),
            record.args().to_string(),
        );
        self.0.lock().expect("capture").push(line);
    }

    fn flush(&self) {}
}

fn captured() -> Vec<(Level, String)> {
    static CAPTURE: OnceLock<Capture> = OnceLock::new();
    let capture = CAPTURE.get_or_init(|| Capture(Mutex::new(Vec::new())));
    let _ = log::set_logger(capture);
    log::set_max_level(LevelFilter::Trace);
    let me = std::thread::current().id();
    capture
        .0
        .lock()
        .expect("capture")
        .iter()
        .filter(|(thread, _, _)| *thread == me)
        .map(|(_, level, message)| (*level, message.clone()))
        .collect()
}

fn config(seed: u64) -> GameConfig {
    GameConfig {
        seed,
        scenario: Scenario::sandbox(),
        hero: Hero::Arythea,
        dummy_hero: Default::default(),
    }
}

/// Joueur qui renvoie un index hors de la liste des actions légales.
struct OutOfRange;

impl Player for OutOfRange {
    fn request(&mut self, _view: &PlayerView, _legal_actions: &[Action]) {}

    fn poll(&mut self) -> Option<usize> {
        // Spec 013 : une main de 5 cartes offre plus de 5 actions ; 1000 reste hors bornes.
        Some(1_000)
    }
}

#[test]
fn game_runner_logs_the_game() {
    // Spec 003 C12
    captured();
    let mut runner = GameRunner::new(config(42), vec![Box::new(FirstChoice::default())]);
    // Spec 016 : la partie va jusqu'au bout du deck, en plusieurs tours.
    for _ in 0..1_000 {
        if runner.step().expect("pas") == StepOutcome::Finished {
            break;
        }
    }
    // Rappeler step après la fin n'écrit plus rien.
    assert_eq!(runner.step(), Ok(StepOutcome::Finished));
    assert_eq!(runner.step(), Ok(StepOutcome::Finished));

    let lines = captured();
    let (first_level, first) = &lines[0];
    assert_eq!(*first_level, Level::Info);
    assert!(first.contains("graine 42"), "{first}");
    assert!(first.contains("1 joueur"), "{first}");

    let (last_level, last) = lines.last().expect("lignes");
    assert_eq!(*last_level, Level::Info);
    assert!(last.contains("terminée"), "{last}");
    let events = runner.log().len();
    assert!(last.contains(&format!("{events} événement")), "{last}");
    let ends = lines.iter().filter(|(_, m)| m.contains("terminée")).count();
    assert_eq!(ends, 1);

    // Chaque événement du journal, avec son numéro, en trace et dans l'ordre.
    let traces: Vec<&String> = lines
        .iter()
        .filter(|(level, _)| *level == Level::Trace)
        .map(|(_, message)| message)
        .collect();
    assert_eq!(traces.len(), events);
    for (sequence, message) in traces.iter().enumerate() {
        assert!(message.contains(&format!("n°{sequence} ")), "{message}");
    }

    // Pour chaque décision : la demande, puis l'action choisie, puis ses événements.
    let debugs: Vec<&String> = lines
        .iter()
        .filter(|(level, _)| *level == Level::Debug)
        .map(|(_, message)| message)
        .collect();
    // Spec 013 : une décision par carte jouée ; spec 014 : puis par déplacement payable ;
    // spec 015 : par exploration ; spec 016 : par fin de tour ; spec 023 : par fin de manche ;
    // spec 031 : par action du combat (la parade depuis la spec 033 : une créature invoquée attaque).
    assert_eq!(debugs.len(), 2 * runner.decision_log().len());
    for pair in debugs.chunks(2) {
        assert!(pair[0].contains("demandée"), "{}", pair[0]);
        assert!(pair[1].contains("choisit"), "{}", pair[1]);
        assert!(
            [
                "PlayCard",
                "MoveHero",
                "Explore",
                "EndTurn",
                "AnnounceEndOfRound",
                "Challenge",
                "StartCombat",
                "NextCombatPhase",
                "EndCombat",
                "DeclareAttack",
                "ResolveAttack",
                "DeclareBlock",
                "ResolveBlock",
                "SpendHealing",
            ]
            .iter()
            .any(|kind| pair[1].contains(kind)),
            "{}",
            pair[1]
        );
    }
    let position = |text: &str| {
        lines
            .iter()
            .position(|(_, message)| message.contains(text))
            .expect("ligne présente")
    };
    assert!(position("demandée") < position("choisit"));
    // Premier événement après ceux de démarrage (spec 012 : mise en place comprise).
    let setup = config(42);
    let setup_events = arcanum_core::new_game(&setup, &mut setup.rngs()).len();
    assert!(position("choisit") < position(&format!("n°{setup_events} ")));
}

#[test]
fn game_runner_logs_its_errors() {
    // Spec 003 C13
    captured();
    let mut runner = GameRunner::new(config(1), vec![Box::new(OutOfRange)]);
    let error = runner.step().expect_err("choix hors des actions légales");
    let lines = captured();
    assert!(
        lines
            .iter()
            .any(|(level, message)| *level == Level::Error && message.contains(&error.to_string())),
        "{lines:?}"
    );
}

#[test]
fn pure_engine_functions_never_log() {
    // Spec 003 C14
    let before = captured().len();
    let config = config(9);
    let mut rng = config.rngs();
    let events = new_game(&config, &mut rng);
    let state = events.iter().fold(GameState::default(), apply);
    let _ = pending_decision(&state);
    let legal = legal_actions(&state, SOLO);
    let _ = view(&state, SOLO);
    let drawn = decide(&state, SOLO, &legal[0], &mut rng).expect("décision");
    let _ = drawn.iter().fold(state.clone(), apply);
    let _ = simulate(&state, SOLO, &legal[0], &mut rng).expect("simulation");
    let _ = decide(&state, PlayerId(3), &legal[0], &mut rng);
    assert_eq!(captured().len(), before);
}
