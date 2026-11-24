//! Logger global de capture des tests qui vérifient les logs (issue privée 275). Chaque binaire de test
//! qui l'utilise l'installe au premier appel ; les lignes sont rangées par thread pour que les
//! tests restent parallèles. Lu aussi par arcanum-persistance par
//! `#[path = "../../arcanum-ui/tests/support/logs.rs"] mod logs;` (issue privée 358) : chaque crate n'en
//! utilise qu'une partie.
#![allow(dead_code)]

use std::sync::{Mutex, OnceLock};
use std::thread::ThreadId;

use log::{Level, LevelFilter, Log, Metadata, Record};

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

/// Installe le logger de capture si besoin et rend les lignes écrites jusqu'ici par ce thread.
pub fn captured() -> Vec<(Level, String)> {
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

/// Messages de niveau `level` écrits jusqu'ici par ce thread.
pub fn logged(level: Level) -> Vec<String> {
    captured()
        .into_iter()
        .filter(|(l, _)| *l == level)
        .map(|(_, message)| message)
        .collect()
}

/// Lignes écrites par `action` seulement.
pub fn logs_of<T>(action: impl FnOnce() -> T) -> (T, Vec<(Level, String)>) {
    let before = captured().len();
    let result = action();
    let lines = captured().split_off(before);
    (result, lines)
}
