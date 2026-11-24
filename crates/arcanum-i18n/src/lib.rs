//! Libellés par langue (spec 001, ADR 0003).
//! Charge `<langue>.json`, résout les clés et applique les paramètres nommés.
//! Une résolution ne plante jamais : un manque reste visible et il est relevé.

mod label;
mod tree;

use std::collections::{BTreeMap, BTreeSet};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use thiserror::Error;

use crate::label::{Label, Part};

/// Code de langue, tiré du nom du fichier : `fr`, `en`…
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Language(pub String);

/// Manque relevé pendant une résolution.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Gap {
    MissingKey(String),
    MissingParameter { key: String, parameter: String },
}

/// Erreur de chargement d'un fichier de langue.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum LoadError {
    #[error("fichier de langue introuvable : {0}")]
    FileNotFound(PathBuf),
    #[error("lecture impossible de {path} : {message}")]
    Io { path: PathBuf, message: String },
    #[error("{0} n'est pas de l'UTF-8 valide")]
    NotUtf8(PathBuf),
    #[error("JSON invalide : {0}")]
    InvalidJson(String),
    #[error("la racine du fichier doit être un objet JSON")]
    RootNotObject,
    #[error("{key} : un libellé doit être une chaîne")]
    NotAString { key: String },
    #[error("{key} : libellé vide")]
    EmptyLabel { key: String },
    #[error("nom de clé invalide : {key}")]
    InvalidKeyName { key: String },
    #[error("{key} : nom de paramètre invalide « {parameter} »")]
    InvalidParameterName { key: String, parameter: String },
    #[error("{key} : accolade mal formée")]
    MalformedBraces { key: String },
    #[error("clé en double : {key}")]
    DuplicateKey { key: String },
}

/// Ensemble des libellés d'une langue.
#[derive(Debug)]
pub struct Catalog {
    language: Language,
    labels: BTreeMap<String, Label>,
    // Mutex : la résolution prend `&self` et le catalogue doit rester partageable entre threads.
    gaps: Mutex<BTreeSet<Gap>>,
}

impl Catalog {
    pub fn from_json_str(language: Language, json: &str) -> Result<Self, LoadError> {
        let labels = tree::parse(json)?
            .into_iter()
            .map(|(key, text)| Label::parse(&key, &text).map(|label| (key, label)))
            .collect::<Result<_, _>>()?;
        Ok(Self {
            language,
            labels,
            gaps: Mutex::new(BTreeSet::new()),
        })
    }

    /// Lit `<dir>/<langue>.json`.
    pub fn load(dir: &Path, language: &Language) -> Result<Self, LoadError> {
        let path = dir.join(format!("{}.json", language.0));
        let bytes = std::fs::read(&path).map_err(|error| match error.kind() {
            ErrorKind::NotFound => LoadError::FileNotFound(path.clone()),
            _ => LoadError::Io {
                path: path.clone(),
                message: error.to_string(),
            },
        })?;
        let json = String::from_utf8(bytes).map_err(|_| LoadError::NotUtf8(path.clone()))?;
        Self::from_json_str(language.clone(), &json)
    }

    pub fn language(&self) -> &Language {
        &self.language
    }

    /// Libellé sans paramètre.
    pub fn text(&self, key: &str) -> String {
        self.format(key, &[])
    }

    /// Libellé avec ses paramètres remplacés ; un paramètre en trop est ignoré.
    pub fn format(&self, key: &str, parameters: &[(&str, &str)]) -> String {
        let Some(label) = self.labels.get(key) else {
            self.record(Gap::MissingKey(key.to_owned()));
            return format!("⟦{key}⟧");
        };
        let mut text = String::new();
        for part in &label.parts {
            match part {
                Part::Text(literal) => text.push_str(literal),
                Part::Parameter(name) => match parameters.iter().find(|(given, _)| given == name) {
                    Some((_, value)) => text.push_str(value),
                    None => {
                        self.record(Gap::MissingParameter {
                            key: key.to_owned(),
                            parameter: name.clone(),
                        });
                        text.push_str(&format!("{{{name}}}"));
                    }
                },
            }
        }
        text
    }

    pub fn keys(&self) -> BTreeSet<String> {
        self.labels.keys().cloned().collect()
    }

    /// Paramètres d'une clé, ou `None` si la clé n'existe pas.
    pub fn parameters(&self, key: &str) -> Option<BTreeSet<String>> {
        self.labels.get(key).map(Label::parameters)
    }

    /// Manques relevés depuis le chargement, triés et sans doublon.
    pub fn gaps(&self) -> Vec<Gap> {
        self.locked_gaps().iter().cloned().collect()
    }

    fn record(&self, gap: Gap) {
        let is_new = self.locked_gaps().insert(gap.clone());
        // Écrit hors du verrou, et une seule fois par manque (spec 003 C11).
        if is_new {
            match gap {
                Gap::MissingKey(key) => log::warn!("clé de libellé manquante : {key}"),
                Gap::MissingParameter { key, parameter } => {
                    log::warn!("paramètre « {parameter} » non fourni pour le libellé {key}")
                }
            }
        }
    }

    fn locked_gaps(&self) -> MutexGuard<'_, BTreeSet<Gap>> {
        // Un panic ailleurs ne doit pas empêcher d'afficher des libellés : on ignore l'empoisonnement.
        self.gaps
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

/// Langues disponibles dans un dossier : les fichiers `*.json`, triés.
pub fn available_languages(dir: &Path) -> Result<Vec<Language>, LoadError> {
    let io_error = |error: std::io::Error| LoadError::Io {
        path: dir.to_path_buf(),
        message: error.to_string(),
    };
    let mut languages = Vec::new();
    for entry in std::fs::read_dir(dir).map_err(io_error)? {
        let path = entry.map_err(io_error)?.path();
        let is_json = path
            .extension()
            .is_some_and(|extension| extension == "json");
        if let (true, Some(stem)) = (is_json, path.file_stem().and_then(|stem| stem.to_str())) {
            languages.push(Language(stem.to_owned()));
        }
    }
    languages.sort();
    Ok(languages)
}

/// Segment de clé ou nom de paramètre : minuscules ASCII, chiffres, `_`, commence par une lettre.
fn is_valid_name(name: &str) -> bool {
    let mut chars = name.chars();
    chars.next().is_some_and(|first| first.is_ascii_lowercase())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}
