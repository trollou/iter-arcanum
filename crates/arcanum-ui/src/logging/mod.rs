//! Logs techniques (spec 003) : où écrire, quoi garder, comment formater.
//! Les bibliothèques n'utilisent que la façade `log` ; seul l'exécutable installe `ArcanumLogger`.

mod platform;

use std::ffi::OsString;
use std::fmt::Display;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use chrono::{DateTime, Local, SecondsFormat, TimeZone};
use log::{Level, LevelFilter, Log, Metadata, Record};

pub use platform::{macos_logs_dir, user_base_dir, user_logs_dir};

/// Gravité minimale de nos crates.
pub const LOG_LEVEL_ENV_VAR: &str = "ARCANUM_LOG";
/// Dossier des logs imposé, prioritaire sur tout le reste.
pub const LOG_DIR_ENV_VAR: &str = "ARCANUM_LOG_DIR";
pub const LOG_FILE_NAME: &str = "iter-arcanum.log";
/// Nombre de lancements précédents gardés en plus du lancement en cours.
pub const KEPT_PREVIOUS_LOGS: usize = 5;

/// Gravité par défaut de nos crates, et gravité fixe des crates externes.
const DEFAULT_LEVEL: LevelFilter = LevelFilter::Info;
const EXTERNAL_LEVEL: LevelFilter = LevelFilter::Warn;
const OUR_CRATES_PREFIX: &str = "arcanum_";
/// Cible des logs de `main.rs` : le nom de l'exécutable, qui n'a pas le préfixe des crates.
const OUR_EXECUTABLE: &str = "iter_arcanum";

/// Raison du choix du dossier des logs, écrite au démarrage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogDirSource {
    EnvVar,
    Cargo,
    UserData,
}

impl Display for LogDirSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::EnvVar => "variable ARCANUM_LOG_DIR",
            Self::Cargo => "lancement par cargo",
            Self::UserData => "jeu déployé",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogDirChoice {
    pub dir: PathBuf,
    pub source: LogDirSource,
}

/// Gravité retenue, et la valeur de `ARCANUM_LOG` si elle n'a pas été comprise.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LevelChoice {
    pub level: LevelFilter,
    pub unknown: Option<String>,
}

/// Fichier de logs ouvert, et ce qui n'a pas pu être fait en le préparant.
#[derive(Debug)]
pub struct PreparedLogFile {
    pub path: PathBuf,
    pub file: File,
    pub warnings: Vec<String>,
}

/// Choisit le dossier des logs : `ARCANUM_LOG_DIR`, puis lancement par cargo, puis dossier de l'utilisateur.
/// Pure : l'appelant lit l'environnement.
pub fn choose_logs_dir(
    log_dir_var: Option<OsString>,
    cargo_manifest_dir: Option<OsString>,
    user_base: Option<PathBuf>,
) -> Option<LogDirChoice> {
    if let Some(dir) = log_dir_var.filter(|value| !value.is_empty()) {
        return Some(LogDirChoice {
            dir: dir.into(),
            source: LogDirSource::EnvVar,
        });
    }
    // `cargo run` définit CARGO_MANIFEST_DIR = <racine>/crates/arcanum-ui.
    let repository = cargo_manifest_dir
        .map(PathBuf::from)
        .and_then(|manifest| manifest.parent()?.parent().map(Path::to_path_buf));
    if let Some(root) = repository {
        return Some(LogDirChoice {
            dir: root.join("logs"),
            source: LogDirSource::Cargo,
        });
    }
    user_base.map(|base| LogDirChoice {
        dir: user_logs_dir(&base),
        source: LogDirSource::UserData,
    })
}

pub fn parse_level(value: Option<&str>) -> LevelChoice {
    match value {
        None => LevelChoice {
            level: DEFAULT_LEVEL,
            unknown: None,
        },
        Some(text) => match text.trim().to_ascii_lowercase().as_str() {
            "error" => known(LevelFilter::Error),
            "warn" => known(LevelFilter::Warn),
            "info" => known(LevelFilter::Info),
            "debug" => known(LevelFilter::Debug),
            "trace" => known(LevelFilter::Trace),
            _ => LevelChoice {
                level: DEFAULT_LEVEL,
                unknown: Some(text.to_owned()),
            },
        },
    }
}

fn known(level: LevelFilter) -> LevelChoice {
    LevelChoice {
        level,
        unknown: None,
    }
}

/// Nos crates suivent `ours` ; les autres restent à `warn`, pour ne pas noyer le fichier.
pub fn is_enabled(target: &str, level: Level, ours: LevelFilter) -> bool {
    let ours_target = target.starts_with(OUR_CRATES_PREFIX)
        || target
            .strip_prefix(OUR_EXECUTABLE)
            .is_some_and(|rest| rest.is_empty() || rest.starts_with("::"));
    let threshold = if ours_target { ours } else { EXTERNAL_LEVEL };
    level <= threshold
}

/// `horodatage gravité cible: message`, horodatage RFC 3339 à la milliseconde, `Z` pour un décalage nul.
pub fn format_line<Tz: TimeZone>(
    time: &DateTime<Tz>,
    level: Level,
    target: &str,
    message: &str,
) -> String
where
    Tz::Offset: Display,
{
    let timestamp = time.to_rfc3339_opts(SecondsFormat::Millis, true);
    format!("{timestamp} {:<5} {target}: {message}\n", level.as_str())
}

fn rotated_path(dir: &Path, rank: usize) -> PathBuf {
    if rank == 0 {
        dir.join(LOG_FILE_NAME)
    } else {
        dir.join(format!("iter-arcanum.{rank}.log"))
    }
}

/// Décale les fichiers des lancements précédents d'un rang et supprime le plus ancien.
/// Ne s'arrête jamais sur un échec : il est rendu pour être écrit dans les logs (L4).
pub fn rotate_log_files(dir: &Path) -> Vec<String> {
    let mut failures = Vec::new();
    let oldest = rotated_path(dir, KEPT_PREVIOUS_LOGS);
    if oldest.exists()
        && let Err(error) = fs::remove_file(&oldest)
    {
        failures.push(format!("suppression de {} : {error}", oldest.display()));
    }
    for rank in (0..KEPT_PREVIOUS_LOGS).rev() {
        let from = rotated_path(dir, rank);
        if !from.exists() {
            continue;
        }
        let to = rotated_path(dir, rank + 1);
        if let Err(error) = fs::rename(&from, &to) {
            failures.push(format!(
                "renommage de {} en {} : {error}",
                from.display(),
                to.display()
            ));
        }
    }
    failures
}

/// Crée le dossier si besoin, fait tourner les fichiers et ouvre celui du lancement.
pub fn prepare_log_file(dir: &Path) -> std::io::Result<PreparedLogFile> {
    fs::create_dir_all(dir)?;
    let warnings = rotate_log_files(dir);
    let path = rotated_path(dir, 0);
    // Ajout plutôt qu'écrasement : si la rotation a échoué, on ne perd pas le fichier d'une autre instance.
    let file = OpenOptions::new().create(true).append(true).open(&path)?;
    Ok(PreparedLogFile {
        path,
        file,
        warnings,
    })
}

/// Première ligne de logs d'un lancement (C10).
pub fn startup_message(level: LevelFilter, file: Option<(&Path, &LogDirSource)>) -> String {
    let destination = match file {
        Some((path, source)) => format!("logs dans {} ({source})", path.display()),
        None => "logs sur la sortie d'erreur seulement".to_owned(),
    };
    format!(
        "lancement d'Iter Arcanum {} sur {}, gravité {}, {destination}",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        level.as_str()
    )
}

/// Écrit chaque panic en `error`, puis laisse faire le comportement habituel (C15).
pub fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        log::error!("panic : {info}");
        previous(info);
    }));
}

type Sink = Mutex<Box<dyn Write + Send>>;

/// Écrit les lignes acceptées dans chacune de ses destinations (fichier, sortie d'erreur).
pub struct ArcanumLogger {
    ours: LevelFilter,
    sinks: Vec<Sink>,
}

impl ArcanumLogger {
    pub fn new(ours: LevelFilter, sinks: Vec<Box<dyn Write + Send>>) -> Self {
        Self {
            ours,
            sinks: sinks.into_iter().map(Mutex::new).collect(),
        }
    }

    /// Gravité maximale à donner à `log::set_max_level`.
    pub fn max_level(&self) -> LevelFilter {
        self.ours.max(EXTERNAL_LEVEL)
    }
}

impl Log for ArcanumLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        is_enabled(metadata.target(), metadata.level(), self.ours)
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let line = format_line(
            &Local::now(),
            record.level(),
            record.target(),
            &record.args().to_string(),
        );
        for sink in &self.sinks {
            // Un panic pendant une écriture ne doit pas faire taire les logs suivants.
            let mut sink = sink.lock().unwrap_or_else(PoisonError::into_inner);
            // Vidé à chaque ligne pour ne rien perdre en cas de plantage ; un échec d'écriture n'a nulle part où aller.
            let _ = sink.write_all(line.as_bytes());
            let _ = sink.flush();
        }
    }

    fn flush(&self) {
        for sink in &self.sinks {
            let _ = sink.lock().unwrap_or_else(PoisonError::into_inner).flush();
        }
    }
}

/// Installe les logs du processus. Ne fait jamais échouer le lancement (L2, L3).
pub fn init_logging() {
    let level = parse_level(std::env::var(LOG_LEVEL_ENV_VAR).ok().as_deref());
    let choice = choose_logs_dir(
        std::env::var_os(LOG_DIR_ENV_VAR),
        std::env::var_os("CARGO_MANIFEST_DIR"),
        platform::user_base_dir(),
    );

    let mut sinks: Vec<Box<dyn Write + Send>> = Vec::new();
    let mut problems = Vec::new();
    let mut opened = None;
    match &choice {
        Some(choice) => match prepare_log_file(&choice.dir) {
            Ok(prepared) => {
                problems.extend(prepared.warnings);
                sinks.push(Box::new(prepared.file));
                opened = Some((prepared.path, choice.source.clone()));
            }
            Err(error) => problems.push(format!(
                "fichier de logs impossible dans {} : {error}",
                choice.dir.display()
            )),
        },
        None => problems.push("aucun dossier personnel trouvé pour les logs".to_owned()),
    }
    // En développement on lit aussi les logs dans la console ; sans fichier, c'est le seul endroit.
    if cfg!(debug_assertions) || opened.is_none() {
        sinks.push(Box::new(std::io::stderr()));
    }

    let logger = ArcanumLogger::new(level.level, sinks);
    let max_level = logger.max_level();
    if log::set_boxed_logger(Box::new(logger)).is_ok() {
        log::set_max_level(max_level);
    }
    install_panic_hook();

    let file = opened
        .as_ref()
        .map(|(path, source)| (path.as_path(), source));
    log::info!("{}", startup_message(level.level, file));
    if let Some(unknown) = level.unknown {
        log::warn!("valeur de {LOG_LEVEL_ENV_VAR} inconnue : « {unknown} », gravité info retenue");
    }
    for problem in problems {
        log::warn!("{problem}");
    }
}
