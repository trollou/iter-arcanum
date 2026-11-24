//! Spec 003 C11 : les manques relevés par un catalogue sont écrits dans les logs, une seule fois.

use std::sync::{Mutex, OnceLock};
use std::thread::ThreadId;

use arcanum_i18n::{Catalog, Language};
use log::{Level, LevelFilter, Log, Metadata, Record};

/// Logger global de capture, qui range les lignes par thread pour que les tests restent parallèles.
struct Capture(Mutex<Vec<(ThreadId, Level, String, String)>>);

impl Log for Capture {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let line = (
            std::thread::current().id(),
            record.level(),
            record.target().to_owned(),
            record.args().to_string(),
        );
        self.0.lock().expect("capture").push(line);
    }

    fn flush(&self) {}
}

fn captured() -> Vec<(Level, String, String)> {
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
        .filter(|(thread, ..)| *thread == me)
        .map(|(_, level, target, message)| (*level, target.clone(), message.clone()))
        .collect()
}

#[test]
fn gap_is_logged_once_as_warning() {
    // Spec 003 C11
    captured();
    let catalog = Catalog::from_json_str(
        Language("fr".into()),
        r#"{"card": {"march": {"basic": "Mouvement {amount}"}}}"#,
    )
    .expect("catalogue");

    for _ in 0..3 {
        catalog.text("card.rage.name");
        catalog.text("card.march.basic");
    }

    let lines = captured();
    assert_eq!(lines.len(), 2, "{lines:?}");
    for (level, target, _) in &lines {
        assert_eq!(*level, Level::Warn);
        assert!(target.starts_with("arcanum_i18n"), "{target}");
    }
    assert!(lines[0].2.contains("card.rage.name"), "{lines:?}");
    assert!(lines[1].2.contains("card.march.basic"), "{lines:?}");
    assert!(lines[1].2.contains("amount"), "{lines:?}");
}
