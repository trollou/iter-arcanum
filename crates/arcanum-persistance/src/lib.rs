//! Sauvegarde et chargement des parties (spec 024) : le journal (la configuration de départ est
//! dans `GameStarted`) et les positions des générateurs (spec 038), en JSON. Rien ne choisit ici le dossier :
//! l'appelant le donne.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use arcanum_core::{
    EVENT_FORMAT_VERSION, Event, EventLog, GameRunner, Hero, Player, ResumeError, RngPositions,
    ScenarioId, TimeOfDay,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod migration;
pub mod preferences;

use migration::Formats;

/// Nom (sans extension) de la sauvegarde automatique, remplacée à chaque fois.
pub const AUTOSAVE_NAME: &str = "auto";

const EXTENSION: &str = "json";

/// Une sauvegarde, telle qu'écrite dans le fichier.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveGame {
    /// Numéro de format (`EVENT_FORMAT_VERSION`) ; une sauvegarde d'un format gardé plus ancien
    /// est migrée au chargement (spec #52).
    pub version: u16,
    pub summary: SaveSummary,
    /// Position du générateur de chaque usage déjà tiré (spec 038), en texte : un entier de
    /// 128 bits que JSON lit mal hors de Rust.
    #[serde(with = "positions_as_text")]
    pub rng_positions: RngPositions,
    pub events: Vec<SavedEvent>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedEvent {
    pub decision: u64,
    pub event: Event,
}

/// Ce que la liste des sauvegardes affiche ; jamais relu pour jouer.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveSummary {
    /// Date et heure locales, RFC 3339, données par l'appelant (arcanum-persistance n'a pas d'horloge).
    pub saved_at: String,
    pub scenario: ScenarioId,
    pub hero: Hero,
    pub round: u32,
    pub round_limit: u8,
    pub time_of_day: TimeOfDay,
    pub turn: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum SaveError {
    #[error("partie pas encore commencée")]
    NotStarted,
    #[error("écriture impossible : {0}")]
    Io(String),
    #[error("sérialisation impossible : {0}")]
    Serialize(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum LoadError {
    #[error("fichier illisible : {0}")]
    Io(String),
    /// JSON invalide, `version` absente, ou JSON (migré ou non) qui ne se lit pas comme une
    /// sauvegarde du format courant.
    #[error("sauvegarde invalide : {0}")]
    Invalid(String),
    #[error("sauvegarde du format {found}, plus récent que celui du jeu ({EVENT_FORMAT_VERSION})")]
    NewerVersion { found: u16 },
    /// Format plus ancien que celui de la plus ancienne des trois dernières versions publiées.
    #[error("sauvegarde du format {found}, trop ancien pour être migré")]
    TooOld { found: u16 },
    #[error("sauvegarde du format {found} impossible à migrer : {reason}")]
    NotMigratable { found: u16, reason: String },
    #[error(transparent)]
    Resume(#[from] ResumeError),
}

impl SaveGame {
    /// La partie telle qu'elle est maintenant, datée de `saved_at`.
    pub fn from_runner(runner: &GameRunner, saved_at: &str) -> Result<SaveGame, SaveError> {
        let state = runner.state();
        let scenario = state.scenario().ok_or(SaveError::NotStarted)?;
        let hero = state
            .hero(arcanum_core::PlayerId::SOLO)
            .ok_or(SaveError::NotStarted)?;
        Ok(SaveGame {
            version: EVENT_FORMAT_VERSION,
            summary: SaveSummary {
                saved_at: saved_at.to_string(),
                scenario: scenario.id(),
                hero,
                round: state.round(),
                round_limit: scenario.rounds(),
                time_of_day: state.time_of_day(),
                turn: state.turn(),
            },
            rng_positions: runner.rng_positions(),
            events: runner
                .log()
                .entries()
                .iter()
                .map(|entry| SavedEvent {
                    decision: entry.decision,
                    event: entry.event.clone(),
                })
                .collect(),
        })
    }

    /// Reprend la partie : le journal est rejoué, la suite tire le même hasard.
    pub fn resume(self, players: Vec<Box<dyn Player>>) -> Result<GameRunner, LoadError> {
        let mut log = EventLog::default();
        for saved in self.events {
            log.push(saved.decision, saved.event);
        }
        Ok(GameRunner::resume(log, self.rng_positions, players)?)
    }
}

/// Écrit une nouvelle sauvegarde `<stem>.json` dans `dir`, avec un suffixe `-2`, `-3`… si le nom
/// est déjà pris. Renvoie le chemin écrit.
pub fn save(dir: &Path, stem: &str, game: &SaveGame) -> Result<PathBuf, SaveError> {
    let mut path = dir.join(format!("{stem}.{EXTENSION}"));
    let mut suffix = 2;
    while path.exists() {
        path = dir.join(format!("{stem}-{suffix}.{EXTENSION}"));
        suffix += 1;
    }
    write(&path, game)?;
    Ok(path)
}

/// Écrit la sauvegarde automatique, qui remplace la précédente.
pub fn save_auto(dir: &Path, game: &SaveGame) -> Result<PathBuf, SaveError> {
    let path = dir.join(format!("{AUTOSAVE_NAME}.{EXTENSION}"));
    write(&path, game)?;
    Ok(path)
}

fn write(path: &Path, game: &SaveGame) -> Result<(), SaveError> {
    let text = serde_json::to_string_pretty(game)
        .map_err(|error| SaveError::Serialize(error.to_string()))?;
    fs::write(path, text).map_err(|error| SaveError::Io(error.to_string()))?;
    log::info!("partie sauvegardée : {}", path.display());
    Ok(())
}

/// Lit une sauvegarde, migrée en mémoire si elle vient d'une version publiée gardée (spec #52).
pub fn load(path: &Path) -> Result<SaveGame, LoadError> {
    load_with(path, &Formats::SUPPORTED)
}

/// `load` avec les formats donnés : ceux d'un test, ou `Formats::SUPPORTED`. Le fichier n'est
/// jamais réécrit.
pub fn load_with(path: &Path, formats: &Formats) -> Result<SaveGame, LoadError> {
    let text = fs::read_to_string(path).map_err(|error| LoadError::Io(error.to_string()))?;
    let value: serde_json::Value =
        serde_json::from_str(&text).map_err(|error| LoadError::Invalid(error.to_string()))?;
    let (value, _found) = formats.migrate(value)?;
    serde_json::from_value(value).map_err(|error| LoadError::Invalid(error.to_string()))
}

/// Une sauvegarde trouvée dans le dossier, lisible ou non.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SaveEntry {
    pub path: PathBuf,
    pub summary: Result<SaveSummary, LoadError>,
}

impl SaveEntry {
    pub fn is_auto(&self) -> bool {
        self.path.file_stem().and_then(|stem| stem.to_str()) == Some(AUTOSAVE_NAME)
    }
}

/// Les sauvegardes de `dir` : l'automatique en tête, puis de la plus récente à la plus ancienne,
/// les illisibles à la fin. Un dossier absent n'en contient aucune.
pub fn list(dir: &Path) -> Vec<SaveEntry> {
    list_with(dir, &Formats::SUPPORTED)
}

/// `list` avec les formats donnés (voir `load_with`).
pub fn list_with(dir: &Path, formats: &Formats) -> Vec<SaveEntry> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) => {
            if error.kind() != ErrorKind::NotFound {
                log::warn!("dossier des sauvegardes illisible : {error}");
            }
            return Vec::new();
        }
    };
    let mut saves: Vec<SaveEntry> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some(EXTENSION))
        .map(|path| SaveEntry {
            summary: load_with(&path, formats).map(|game| game.summary),
            path,
        })
        .collect();
    saves.sort_by(|a, b| {
        let rank = |entry: &SaveEntry| (!entry.is_auto(), entry.summary.is_err());
        rank(a)
            .cmp(&rank(b))
            .then_with(|| match (&a.summary, &b.summary) {
                (Ok(x), Ok(y)) => y.saved_at.cmp(&x.saved_at),
                _ => b.path.cmp(&a.path),
            })
    });
    saves
}

/// Les positions, chacune écrite en texte sous le nom de son usage.
mod positions_as_text {
    use std::collections::BTreeMap;

    use arcanum_core::{RngPositions, RngUsage};
    use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error};

    pub fn serialize<S: Serializer>(
        positions: &RngPositions,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        positions
            .iter()
            .map(|(usage, position)| (*usage, position.to_string()))
            .collect::<BTreeMap<RngUsage, String>>()
            .serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<RngPositions, D::Error> {
        BTreeMap::<RngUsage, String>::deserialize(deserializer)?
            .into_iter()
            .map(|(usage, text)| Ok((usage, text.parse().map_err(D::Error::custom)?)))
            .collect()
    }
}
