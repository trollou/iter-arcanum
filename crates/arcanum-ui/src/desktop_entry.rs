//! Fichier `.desktop` et icône du thème sous Linux (spec 004, issue privée 315) : sans eux, Cinnamon réduit l'unique
//! image de 256 px de la fenêtre sans lissage. Compilé partout pour être testé, appelé sous Linux seulement.

use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::icon::{APP_ID, ICON_SVG};

/// Marque d'un `.desktop` écrit par le jeu : un fichier sans elle appartient à l'utilisateur.
pub const DESKTOP_MARK: &str = "X-Iter-Arcanum-Generated=true";

#[derive(Debug, Error)]
pub enum DesktopError {
    #[error("écriture de {path:?} impossible : {source}")]
    Write {
        path: PathBuf,
        source: std::io::Error,
    },
}

/// Ce que l'installation a fait du `.desktop`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopInstall {
    Written,
    Unchanged,
    KeptUserFile,
}

/// `applications/iter-arcanum.desktop` sous le dossier des données.
pub fn desktop_file_path(data_home: &Path) -> PathBuf {
    data_home
        .join("applications")
        .join(format!("{APP_ID}.desktop"))
}

/// `icons/hicolor/scalable/apps/iter-arcanum.svg` sous le dossier des données.
pub fn theme_icon_path(data_home: &Path) -> PathBuf {
    data_home
        .join("icons")
        .join("hicolor")
        .join("scalable")
        .join("apps")
        .join(format!("{APP_ID}.svg"))
}

/// Contenu du `.desktop` qui lance `exe`.
pub fn desktop_entry(exe: &Path) -> String {
    format!(
        "[Desktop Entry]\nType=Application\nName=Iter Arcanum\nExec={}\nIcon={APP_ID}\n\
         StartupWMClass={APP_ID}\nCategories=Game;\nTerminal=false\n{DESKTOP_MARK}\n",
        quoted_exec(exe)
    )
}

/// Argument de `Exec` : entre guillemets, `"`, `` ` ``, `$` et `\` précédés d'un antislash, puis chaque antislash
/// doublé, car la valeur d'une clé du `.desktop` interprète elle-même les antislashs (norme Desktop Entry).
fn quoted_exec(exe: &Path) -> String {
    let mut quoted = String::from("\"");
    for character in exe.to_string_lossy().chars() {
        if matches!(character, '"' | '`' | '$' | '\\') {
            quoted.push_str("\\\\");
        }
        if character == '\\' {
            quoted.push_str("\\\\");
        } else {
            quoted.push(character);
        }
    }
    quoted.push('"');
    quoted
}

/// Écrit le `.desktop` et l'icône sous `data_home`, sans toucher à un `.desktop` de l'utilisateur.
pub fn install_desktop_entry(data_home: &Path, exe: &Path) -> Result<DesktopInstall, DesktopError> {
    write_if_changed(&theme_icon_path(data_home), ICON_SVG)?;
    let path = desktop_file_path(data_home);
    let entry = desktop_entry(exe);
    match std::fs::read_to_string(&path) {
        Ok(current) if !current.lines().any(|line| line == DESKTOP_MARK) => {
            Ok(DesktopInstall::KeptUserFile)
        }
        Ok(current) if current == entry => Ok(DesktopInstall::Unchanged),
        _ => {
            write(&path, entry.as_bytes())?;
            Ok(DesktopInstall::Written)
        }
    }
}

fn write_if_changed(path: &Path, content: &[u8]) -> Result<(), DesktopError> {
    if std::fs::read(path).is_ok_and(|current| current == content) {
        return Ok(());
    }
    write(path, content)
}

fn write(path: &Path, content: &[u8]) -> Result<(), DesktopError> {
    let error = |source| DesktopError::Write {
        path: path.to_path_buf(),
        source,
    };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(error)?;
    }
    std::fs::write(path, content).map_err(error)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Spec 004, C9.
    #[test]
    fn desktop_entry_names_exe_icon_and_window_class() {
        let entry = desktop_entry(Path::new("/opt/jeu/arcanum-ui"));
        let lines: Vec<&str> = entry.lines().collect();
        assert_eq!(lines[0], "[Desktop Entry]");
        for expected in [
            "Type=Application",
            "Name=Iter Arcanum",
            "Exec=\"/opt/jeu/arcanum-ui\"",
            "Icon=iter-arcanum",
            "StartupWMClass=iter-arcanum",
            "Categories=Game;",
            "Terminal=false",
            DESKTOP_MARK,
        ] {
            assert!(
                lines.contains(&expected),
                "ligne manquante : {expected}\n{entry}"
            );
        }
        assert_eq!(APP_ID, "iter-arcanum");
    }

    // Spec 004, C10 : norme Desktop Entry, « The Exec key ».
    #[test]
    fn desktop_entry_quotes_and_escapes_exec_path() {
        let entry = desktop_entry(Path::new("/home/a b/\"x\"/$y/c\\d/arcanum-ui"));
        assert!(
            entry
                .lines()
                .any(|line| line == r#"Exec="/home/a b/\\"x\\"/\\$y/c\\\\d/arcanum-ui""#),
            "{entry}"
        );
    }

    // Spec 004, C11.
    #[test]
    fn install_writes_desktop_file_and_svg_icon() {
        let dir = tempfile::tempdir().unwrap();
        let exe = Path::new("/opt/jeu/arcanum-ui");
        let outcome = install_desktop_entry(dir.path(), exe).unwrap();
        assert_eq!(outcome, DesktopInstall::Written);
        let desktop = dir.path().join("applications").join("iter-arcanum.desktop");
        assert_eq!(desktop_file_path(dir.path()), desktop);
        assert_eq!(
            std::fs::read_to_string(&desktop).unwrap(),
            desktop_entry(exe)
        );
        let icon = dir
            .path()
            .join("icons")
            .join("hicolor")
            .join("scalable")
            .join("apps")
            .join("iter-arcanum.svg");
        assert_eq!(theme_icon_path(dir.path()), icon);
        assert_eq!(std::fs::read(&icon).unwrap(), ICON_SVG);
        assert_eq!(
            install_desktop_entry(dir.path(), exe).unwrap(),
            DesktopInstall::Unchanged
        );
    }

    // Spec 004, C12.
    #[test]
    fn install_keeps_user_desktop_file() {
        let dir = tempfile::tempdir().unwrap();
        let desktop = desktop_file_path(dir.path());
        std::fs::create_dir_all(desktop.parent().unwrap()).unwrap();
        let user = "[Desktop Entry]\nName=Mon jeu\nExec=/ailleurs/arcanum-ui\n";
        std::fs::write(&desktop, user).unwrap();
        let outcome = install_desktop_entry(dir.path(), Path::new("/opt/jeu/arcanum-ui")).unwrap();
        assert_eq!(outcome, DesktopInstall::KeptUserFile);
        assert_eq!(std::fs::read_to_string(&desktop).unwrap(), user);
        assert_eq!(
            std::fs::read(theme_icon_path(dir.path())).unwrap(),
            ICON_SVG
        );
    }

    // Spec 004, C13.
    #[test]
    fn install_rewrites_own_desktop_file_when_exe_moves() {
        let dir = tempfile::tempdir().unwrap();
        install_desktop_entry(dir.path(), Path::new("/ancien/arcanum-ui")).unwrap();
        let moved = Path::new("/nouveau/arcanum-ui");
        let outcome = install_desktop_entry(dir.path(), moved).unwrap();
        assert_eq!(outcome, DesktopInstall::Written);
        assert_eq!(
            std::fs::read_to_string(desktop_file_path(dir.path())).unwrap(),
            desktop_entry(moved)
        );
    }
}
