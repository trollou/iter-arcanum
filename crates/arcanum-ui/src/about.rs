//! Fenêtre « À propos » (spec #33) : version, commit, licences et liens.

use arcanum_i18n::Catalog;

use crate::screen::{Button, Command};
use crate::{Shortcut, shortcut_label};

/// Le dépôt public du jeu, lien « Code source » : une adresse, pas un libellé.
pub const REPOSITORY_URL: &str = "https://github.com/trollou/iter-arcanum";

/// Le fichier des crédits des sons, livré avec le jeu (spec 081).
pub const SOUND_CREDITS_FILE: &str = "assets/sounds/LICENSES.md";

/// « 0.1.0 (a1b2c3d) », ou « 0.2.0-dev » sans commit (C1, D4).
pub fn version_label(version: &str, commit: Option<&str>) -> String {
    match commit {
        Some(commit) => format!("{version} ({commit})"),
        None => version.to_owned(),
    }
}

/// Les 7 premiers caractères d'un SHA hexadécimal ; `None` s'il est absent, vide ou invalide (C2).
pub fn short_commit(commit: Option<&str>) -> Option<&str> {
    // Un SHA court de git fait au moins 7 caractères : en dessous, ce n'est pas un commit.
    let short = commit?.get(..7)?;
    short
        .bytes()
        .all(|byte| byte.is_ascii_hexdigit())
        .then_some(short)
}

/// La version de ce binaire : version du workspace et commit passé par `build.rs`.
pub fn build_version_label() -> String {
    version_label(
        env!("CARGO_PKG_VERSION"),
        short_commit(option_env!("ARCANUM_COMMIT")),
    )
}

/// La fenêtre « À propos », déjà traduite (C4).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AboutWindow {
    /// « À propos ».
    pub title: String,
    /// « Iter Arcanum ».
    pub name: String,
    /// « Version 0.1.0 (a1b2c3d) ».
    pub version: String,
    /// Licence du code, puis celle des visuels, textes et sons.
    pub license_code: String,
    pub license_assets: String,
    /// « Projet de fan non officiel… ».
    pub unofficial: String,
    /// « Code source » et son adresse.
    pub repository_label: String,
    pub repository_url: String,
    /// « Crédits des sons : assets/sounds/LICENSES.md, livré avec le jeu ».
    pub sounds: String,
    /// « Fermer » (Échap).
    pub close: Button,
}

/// La fenêtre « À propos », traduite dans le catalogue affiché.
pub fn about_window(catalog: &Catalog) -> AboutWindow {
    AboutWindow {
        title: catalog.text("about.title"),
        name: catalog.text("about.name"),
        version: catalog.format("about.version", &[("version", &build_version_label())]),
        license_code: catalog.text("about.license_code"),
        license_assets: catalog.text("about.license_assets"),
        unofficial: catalog.text("about.unofficial"),
        repository_label: catalog.text("about.repository"),
        repository_url: REPOSITORY_URL.to_owned(),
        sounds: catalog.format("about.sounds", &[("file", SOUND_CREDITS_FILE)]),
        close: Button {
            label: catalog.text("about.close"),
            command: Command::CloseAbout,
            shortcut: shortcut_label(Shortcut::Cancel, catalog),
        },
    }
}
