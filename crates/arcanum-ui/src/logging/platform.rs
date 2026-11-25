//! Ce qui dépend du système pour les logs (ADR 0004) : le dossier de base et la forme du dossier des logs.

use std::path::{Path, PathBuf};

const APP_DIR: &str = "iter-arcanum";

/// Dossier des logs sous macOS depuis le dossier personnel : `~/Library/Logs/iter-arcanum` (spec #7, Q5).
/// Compilé partout pour être testé partout.
pub fn macos_logs_dir(home: &Path) -> PathBuf {
    let _ = home;
    todo!()
}

/// Dossier local et non itinérant : des logs n'ont pas à suivre l'utilisateur.
#[cfg(windows)]
pub fn user_base_dir() -> Option<PathBuf> {
    directories::BaseDirs::new().map(|dirs| dirs.data_local_dir().to_path_buf())
}

/// Sous macOS, la base est le dossier personnel : les logs vont dans `~/Library/Logs` (spec #7).
#[cfg(target_os = "macos")]
pub fn user_base_dir() -> Option<PathBuf> {
    todo!()
}

/// La norme XDG range les logs dans le dossier d'état, pas dans celui des données.
#[cfg(not(any(windows, target_os = "macos")))]
pub fn user_base_dir() -> Option<PathBuf> {
    directories::BaseDirs::new().and_then(|dirs| dirs.state_dir().map(Path::to_path_buf))
}

/// Même forme que `directories::ProjectDirs` pour l'application `iter-arcanum`.
#[cfg(windows)]
pub fn user_logs_dir(base: &Path) -> PathBuf {
    base.join(APP_DIR).join("data").join("logs")
}

#[cfg(target_os = "macos")]
pub fn user_logs_dir(base: &Path) -> PathBuf {
    let _ = base;
    todo!()
}

#[cfg(not(any(windows, target_os = "macos")))]
pub fn user_logs_dir(base: &Path) -> PathBuf {
    base.join(APP_DIR).join("logs")
}
