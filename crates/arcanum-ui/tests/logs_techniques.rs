//! Scénarios de la spec 003 (logs techniques) côté arcanum-ui.
//! Aucune règle du livret ici : chaque test cite le cas de la spec.

mod support;

use std::ffi::OsString;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use arcanum_ui::{
    ArcanumLogger, KEPT_PREVIOUS_LOGS, LOG_FILE_NAME, LogDirChoice, LogDirSource, catalog_or_empty,
    choose_logs_dir, format_line, is_enabled, parse_level, prepare_log_file, rotate_log_files,
    startup_message, user_logs_dir,
};
use jiff::civil::date;
use jiff::tz::{TimeZone, offset};
use log::{Level, LevelFilter, Log, Record};
use support::logs::captured;

// --- Outils de test ---

/// Tampon partagé, pour lire ce qu'un `ArcanumLogger` a écrit.
#[derive(Clone, Default)]
struct Buffer(Arc<Mutex<Vec<u8>>>);

impl Buffer {
    fn text(&self) -> String {
        String::from_utf8(self.0.lock().expect("tampon").clone()).expect("UTF-8")
    }
}

impl Write for Buffer {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.lock().expect("tampon").extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn logger(ours: LevelFilter) -> (ArcanumLogger, Buffer) {
    let buffer = Buffer::default();
    (
        ArcanumLogger::new(ours, vec![Box::new(buffer.clone())]),
        buffer,
    )
}

fn send(logger: &ArcanumLogger, level: Level, target: &str, message: &str) {
    logger.log(
        &Record::builder()
            .level(level)
            .target(target)
            .args(format_args!("{message}"))
            .build(),
    );
}

fn french() -> arcanum_i18n::Language {
    arcanum_i18n::Language("fr".to_owned())
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("assets")
        .join(name)
}

fn rotated(dir: &Path, rank: usize) -> PathBuf {
    if rank == 0 {
        dir.join(LOG_FILE_NAME)
    } else {
        dir.join(format!("iter-arcanum.{rank}.log"))
    }
}

// --- Format et filtres ---

#[test]
fn log_line_has_local_timestamp_level_target_and_message() {
    // Spec 003 C1
    let time = date(2026, 10, 24)
        .at(15, 51, 32, 123_000_000)
        .to_zoned_datetime(TimeZone::fixed(offset(2)))
        .expect("date");
    let line = format_line(&time, Level::Warn, "arcanum_i18n", "clé manquante");
    assert_eq!(
        line,
        "2026-10-24T15:51:32.123+02:00 WARN  arcanum_i18n: clé manquante\n"
    );
}

#[test]
fn unknown_local_offset_falls_back_to_utc() {
    // Spec 003 L6 : sans décalage connu, l'heure est écrite en UTC, suffixe Z.
    let time = date(2026, 10, 24)
        .at(13, 51, 32, 0)
        .to_zoned_datetime(TimeZone::UTC)
        .expect("date");
    let line = format_line(&time, Level::Info, "arcanum_ui", "lancement");
    assert!(
        line.starts_with("2026-10-24T13:51:32.000Z INFO  arcanum_ui: "),
        "{line}"
    );
}

#[test]
fn default_level_is_info_for_our_crates() {
    // Spec 003 C2
    let choice = parse_level(None);
    assert_eq!(choice.level, LevelFilter::Info);
    let (logger, buffer) = logger(choice.level);
    send(&logger, Level::Info, "arcanum_ui", "écrit");
    send(&logger, Level::Debug, "arcanum_ui", "ignoré");
    let text = buffer.text();
    assert!(text.contains("écrit"), "{text}");
    assert!(!text.contains("ignoré"), "{text}");
}

#[test]
fn arcanum_log_variable_sets_our_level() {
    // Spec 003 C3
    assert_eq!(parse_level(Some("debug")).level, LevelFilter::Debug);
    assert_eq!(parse_level(Some("DEBUG")).level, LevelFilter::Debug);
    assert_eq!(parse_level(Some("trace")).level, LevelFilter::Trace);
    assert_eq!(parse_level(Some("error")).level, LevelFilter::Error);
    let (logger, buffer) = logger(LevelFilter::Debug);
    send(&logger, Level::Debug, "arcanum_core::runner", "détail");
    assert!(buffer.text().contains("détail"));
}

#[test]
fn unknown_arcanum_log_value_falls_back_to_info() {
    // Spec 003 L1
    let choice = parse_level(Some("bavard"));
    assert_eq!(choice.level, LevelFilter::Info);
    assert_eq!(choice.unknown.as_deref(), Some("bavard"));
    assert_eq!(parse_level(Some("info")).unknown, None);
}

#[test]
fn external_crates_are_filtered_at_warn() {
    // Spec 003 C4
    for ours in [LevelFilter::Error, LevelFilter::Info, LevelFilter::Trace] {
        assert!(!is_enabled("wgpu_core", Level::Info, ours));
        assert!(is_enabled("wgpu_core", Level::Warn, ours));
    }
    assert!(is_enabled("arcanum_i18n", Level::Trace, LevelFilter::Trace));
    let (logger, buffer) = logger(LevelFilter::Trace);
    send(&logger, Level::Info, "eframe::native", "bruit");
    send(&logger, Level::Warn, "eframe::native", "alerte");
    let text = buffer.text();
    assert!(!text.contains("bruit"), "{text}");
    assert!(text.contains("alerte"), "{text}");
}

// --- Dossier des logs ---

#[test]
fn arcanum_log_dir_overrides_everything() {
    // Spec 003 C5
    let choice = choose_logs_dir(
        Some(OsString::from("ailleurs")),
        Some(OsString::from("depot/crates/arcanum-ui")),
        Some(PathBuf::from("base")),
    );
    assert_eq!(
        choice,
        Some(LogDirChoice {
            dir: PathBuf::from("ailleurs"),
            source: LogDirSource::EnvVar,
        })
    );
}

#[test]
fn cargo_run_writes_logs_in_repository() {
    // Spec 003 C6 ; une variable ARCANUM_LOG_DIR vide compte comme absente.
    let manifest = PathBuf::from("racine").join("crates").join("arcanum-ui");
    let choice = choose_logs_dir(
        Some(OsString::new()),
        Some(manifest.into_os_string()),
        Some(PathBuf::from("base")),
    );
    assert_eq!(
        choice,
        Some(LogDirChoice {
            dir: PathBuf::from("racine").join("logs"),
            source: LogDirSource::Cargo,
        })
    );
}

#[test]
fn deployed_game_writes_logs_in_user_data_dir() {
    // Spec 003 C7
    let base = PathBuf::from("base");
    let expected = if cfg!(windows) {
        base.join("iter-arcanum").join("data").join("logs")
    } else if cfg!(target_os = "macos") {
        // Spec #7, Q5 : sous macOS, la base est le dossier personnel.
        base.join("Library").join("Logs").join("iter-arcanum")
    } else {
        base.join("iter-arcanum").join("logs")
    };
    assert_eq!(user_logs_dir(&base), expected);
    assert_eq!(
        choose_logs_dir(None, None, Some(base)),
        Some(LogDirChoice {
            dir: expected,
            source: LogDirSource::UserData,
        })
    );
    // Spec 003 L3 : aucun dossier personnel, aucun dossier des logs.
    assert_eq!(choose_logs_dir(None, None, None), None);
}

#[test]
fn logs_dir_is_created_if_missing() {
    // Spec 003 C9
    let temp = tempfile::tempdir().expect("dossier temporaire");
    let dir = temp.path().join("a").join("b").join("logs");
    let prepared = prepare_log_file(&dir).expect("fichier de logs");
    assert!(dir.is_dir());
    assert_eq!(prepared.path, dir.join(LOG_FILE_NAME));
    assert!(prepared.path.is_file());
    assert!(prepared.warnings.is_empty());
}

#[test]
fn startup_rotates_previous_log_files() {
    // Spec 003 C8
    let temp = tempfile::tempdir().expect("dossier temporaire");
    let dir = temp.path();
    for rank in 0..=KEPT_PREVIOUS_LOGS {
        fs::write(rotated(dir, rank), format!("lancement {rank}")).expect("écriture");
    }
    let failures = rotate_log_files(dir);
    assert!(failures.is_empty(), "{failures:?}");
    assert!(!rotated(dir, 0).exists());
    for rank in 1..=KEPT_PREVIOUS_LOGS {
        let content = fs::read_to_string(rotated(dir, rank)).expect("lecture");
        assert_eq!(content, format!("lancement {}", rank - 1));
    }
    assert!(!rotated(dir, KEPT_PREVIOUS_LOGS + 1).exists());

    // Le fichier du nouveau lancement est neuf.
    let prepared = prepare_log_file(dir).expect("fichier de logs");
    assert_eq!(fs::read_to_string(&prepared.path).expect("lecture"), "");
}

#[test]
fn unwritable_logs_dir_does_not_stop_the_game() {
    // Spec 003 L2 : un fichier à la place d'un dossier parent rend le dossier impossible à créer.
    let temp = tempfile::tempdir().expect("dossier temporaire");
    let blocker = temp.path().join("fichier");
    fs::write(&blocker, "pas un dossier").expect("écriture");
    assert!(prepare_log_file(&blocker.join("logs")).is_err());
}

#[test]
fn rotation_failure_is_not_fatal() {
    // Spec 003 L4 : un plus ancien fichier qui est en fait un dossier non vide ne peut pas être supprimé.
    let temp = tempfile::tempdir().expect("dossier temporaire");
    let dir = temp.path();
    fs::write(rotated(dir, 0), "précédent").expect("écriture");
    let blocker = rotated(dir, KEPT_PREVIOUS_LOGS);
    fs::create_dir(&blocker).expect("dossier");
    fs::write(blocker.join("dedans"), "x").expect("écriture");
    let prepared = prepare_log_file(dir).expect("on écrit quand même");
    assert!(!prepared.warnings.is_empty());
    assert_eq!(prepared.path, dir.join(LOG_FILE_NAME));
}

// --- Démarrage ---

#[test]
fn startup_line_describes_the_run() {
    // Spec 003 C10
    let file = PathBuf::from("logs").join(LOG_FILE_NAME);
    let message = startup_message(LevelFilter::Debug, Some((&file, &LogDirSource::Cargo)));
    assert!(message.contains(env!("CARGO_PKG_VERSION")), "{message}");
    assert!(message.contains(std::env::consts::OS), "{message}");
    assert!(message.contains("DEBUG"), "{message}");
    assert!(message.contains(&file.display().to_string()), "{message}");
    assert!(message.contains("cargo"), "{message}");
    let fallback = startup_message(LevelFilter::Info, None);
    assert!(fallback.contains("sortie d'erreur"), "{fallback}");
}

#[test]
fn assets_dir_search_is_logged() {
    // Spec 003 C17 : dossier retenu en info.
    captured();
    let premier = fixture("premier");
    let _ = catalog_or_empty(std::slice::from_ref(&premier), &french());
    let lines = captured();
    assert!(
        lines.iter().any(|(level, message)| *level == Level::Info
            && message.contains(&premier.display().to_string())),
        "{lines:?}"
    );

    // Aucun dossier : un error liste les emplacements essayés.
    let absent = fixture("absent");
    let _ = catalog_or_empty(std::slice::from_ref(&absent), &french());
    let lines = captured();
    assert!(
        lines.iter().any(|(level, message)| *level == Level::Error
            && message.contains(&absent.display().to_string())),
        "{lines:?}"
    );

    // fr.json invalide : un error donne l'erreur de chargement.
    let before = captured().len();
    let _ = catalog_or_empty(&[fixture("invalide")], &french());
    let lines = captured();
    assert!(
        lines[before..]
            .iter()
            .any(|(level, message)| *level == Level::Error && message.contains("JSON")),
        "{lines:?}"
    );
}

#[test]
fn log_file_is_named_iter_arcanum() {
    // Spec 082 C2 : le fichier de log porte le nom du jeu, ses anciens aussi (`rotated`).
    assert_eq!(LOG_FILE_NAME, "iter-arcanum.log");
}

#[test]
fn executable_logs_follow_our_level() {
    // Spec 003 C2 et spec 082 : l'exécutable `iter-arcanum` (cible `iter_arcanum`) est des nôtres.
    assert!(is_enabled("iter_arcanum", Level::Info, LevelFilter::Info));
    assert!(!is_enabled("iter_arcanum", Level::Debug, LevelFilter::Info));
}
