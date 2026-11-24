//! Où écrire les sauvegardes (spec 024), dans le même ordre que le dossier des logs (spec 003).

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Local, SecondsFormat};

/// Dossier des sauvegardes imposé, prioritaire sur tout le reste.
pub const SAVE_DIR_ENV_VAR: &str = "ARCANUM_SAVE_DIR";

const SAVES_DIR: &str = "saves";

/// `ARCANUM_SAVE_DIR`, puis `saves/` du dépôt si le jeu est lancé par cargo, puis `saves/` dans le
/// dossier de données de l'utilisateur. Pure : l'appelant lit l'environnement.
pub fn choose_saves_dir(
    save_dir_var: Option<OsString>,
    cargo_manifest_dir: Option<OsString>,
    user_data: Option<PathBuf>,
) -> Option<PathBuf> {
    if let Some(dir) = save_dir_var.filter(|value| !value.is_empty()) {
        return Some(dir.into());
    }
    // `cargo run` définit CARGO_MANIFEST_DIR = <racine>/crates/arcanum-ui.
    let repository = cargo_manifest_dir
        .map(PathBuf::from)
        .and_then(|manifest| manifest.parent()?.parent().map(Path::to_path_buf));
    repository.or(user_data).map(|base| base.join(SAVES_DIR))
}

/// Dossier de données de l'application, `directories` choisit selon le système (ADR 0004).
pub fn user_data_dir() -> Option<PathBuf> {
    directories::ProjectDirs::from("", "", "iter-arcanum").map(|dirs| dirs.data_dir().to_path_buf())
}

/// Nom de fichier d'une sauvegarde : date et heure locales, sans `:` (interdit sous Windows).
pub fn save_stem(now: &DateTime<Local>) -> String {
    now.format("%Y-%m-%d_%H-%M-%S").to_string()
}

/// Date du résumé d'une sauvegarde, RFC 3339 : l'ordre alphabétique suit l'ordre des dates.
pub fn saved_at(now: &DateTime<Local>) -> String {
    now.to_rfc3339_opts(SecondsFormat::Secs, false)
}

/// Date affichée dans la liste des sauvegardes : « 2026-10-27 18:00 ».
pub fn display_date(saved_at: &str) -> String {
    DateTime::parse_from_rfc3339(saved_at).map_or_else(
        |_| saved_at.to_string(),
        |date| date.format("%Y-%m-%d %H:%M").to_string(),
    )
}
