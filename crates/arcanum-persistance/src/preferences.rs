//! Préférences du joueur (spec 080) : langue, plein écran au lancement et volumes (spec 081), gardées d'une session à
//! l'autre dans un petit fichier JSON. Rien ne choisit ici le dossier : l'appelant donne le chemin.

use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Version du fichier des préférences écrite par ce jeu.
pub const PREFERENCES_VERSION: u32 = 1;

/// Nom du fichier des préférences, dans le dossier de configuration.
pub const PREFERENCES_FILE: &str = "preferences.json";

/// Réglages du joueur. Champs absents : valeurs par défaut ; champs inconnus : ignorés (C3).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Preferences {
    /// `None` = langue du système ; sinon un code (`fr`, `en`).
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub fullscreen_at_launch: bool,
    /// Spec 081 : absents d'un fichier de la spec 080, les volumes par défaut (C19).
    #[serde(default)]
    pub volumes: Volumes,
    /// Spec #5 : les animations du plateau permises ; absent d'un fichier plus ancien, vrai (C12).
    #[serde(default = "animations_on")]
    pub animations: bool,
}

fn animations_on() -> bool {
    true
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            language: None,
            fullscreen_at_launch: false,
            volumes: Volumes::default(),
            animations: animations_on(),
        }
    }
}

/// Les quatre volumes, de 0 à 100 (spec 081). Le champ s'appelle `volumes` : `music` reste un
/// exemple de champ inconnu (spec 080 C3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Volumes {
    pub master: u8,
    pub music: u8,
    pub ambience: u8,
    pub effects: u8,
}

/// Volume le plus fort.
pub const MAX_VOLUME: u8 = 100;

impl Default for Volumes {
    /// Q6 : la musique et l'ambiance sous les effets.
    fn default() -> Self {
        Self {
            master: 80,
            music: 50,
            ambience: 50,
            effects: 80,
        }
    }
}

impl Volumes {
    /// Les valeurs au-delà de 100 ramenées à 100, chacune avec un `warn` (C19).
    fn clamped(self) -> Self {
        let clamp = |name: &str, value: u8| {
            if value > MAX_VOLUME {
                log::warn!("volume {name} de {value} dans les préférences : ramené à {MAX_VOLUME}");
            }
            value.min(MAX_VOLUME)
        };
        Self {
            master: clamp("master", self.master),
            music: clamp("music", self.music),
            ambience: clamp("ambience", self.ambience),
            effects: clamp("effects", self.effects),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum PreferencesError {
    #[error("fichier des préférences absent")]
    Missing,
    #[error("fichier des préférences illisible : {0}")]
    Io(String),
    #[error("fichier des préférences invalide : {0}")]
    Invalid(String),
    #[error("écriture des préférences impossible : {0}")]
    Write(String),
}

/// Forme du fichier : la version, puis les préférences à plat.
#[derive(Serialize, Deserialize)]
struct PreferencesFile {
    #[serde(default = "current_version")]
    version: u32,
    #[serde(flatten)]
    preferences: Preferences,
}

fn current_version() -> u32 {
    PREFERENCES_VERSION
}

/// Lit le fichier des préférences. Une autre version se lit quand même, avec un `warn` (C4).
pub fn load(path: &Path) -> Result<Preferences, PreferencesError> {
    let text = fs::read_to_string(path).map_err(|error| match error.kind() {
        ErrorKind::NotFound => PreferencesError::Missing,
        _ => PreferencesError::Io(error.to_string()),
    })?;
    let file: PreferencesFile = serde_json::from_str(&text)
        .map_err(|error| PreferencesError::Invalid(error.to_string()))?;
    if file.version != PREFERENCES_VERSION {
        log::warn!(
            "préférences de la version {}, le jeu écrit la version {PREFERENCES_VERSION} : lues quand même",
            file.version
        );
    }
    let mut preferences = file.preferences;
    preferences.volumes = preferences.volumes.clamped();
    Ok(preferences)
}

/// Écrit le fichier des préférences : dans un fichier temporaire du même dossier, puis renommé à
/// sa place, pour qu'un arrêt en pleine écriture ne laisse pas un fichier tronqué (C5).
pub fn save(path: &Path, preferences: &Preferences) -> Result<(), PreferencesError> {
    let write_error = |error: std::io::Error| PreferencesError::Write(error.to_string());
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(write_error)?;
    }
    let file = PreferencesFile {
        version: PREFERENCES_VERSION,
        preferences: preferences.clone(),
    };
    let text = serde_json::to_string_pretty(&file)
        .map_err(|error| PreferencesError::Write(error.to_string()))?;
    let temporary = path.with_extension("json.tmp");
    fs::write(&temporary, text).map_err(write_error)?;
    fs::rename(&temporary, path).map_err(|error| {
        let _ = fs::remove_file(&temporary);
        write_error(error)
    })
}
