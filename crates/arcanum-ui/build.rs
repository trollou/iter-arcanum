//! Embarque l'icône dans `iter-arcanum.exe` quand on compile pour Windows (spec 004), et passe le
//! commit au code par `ARCANUM_COMMIT` pour la fenêtre « À propos » (spec #33).

use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    commit::pass_to_code();
    // La cible, pas la machine qui compile : `cfg!(windows)` décrirait celle-ci.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        windows::embed_icon();
    }
}

#[cfg(windows)]
mod windows {
    use std::path::PathBuf;

    pub fn embed_icon() {
        let ico = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("assets")
            .join("images")
            .join("icon")
            .join("iter-arcanum.ico");
        println!("cargo:rerun-if-changed={}", ico.display());
        let mut resource = winresource::WindowsResource::new();
        resource.set_icon(&ico.to_string_lossy());
        // Spec 004, L1 : sans compilateur de ressources, l'exe se compile sans icône de fichier.
        if let Err(error) = resource.compile() {
            println!(
                "cargo:warning=icône non embarquée dans iter-arcanum.exe (rc.exe ou windres introuvable ?) : {error}"
            );
        }
    }
}

// Compilation croisée vers Windows depuis Linux : hors périmètre, l'exe n'a pas d'icône de fichier.
#[cfg(not(windows))]
mod windows {
    pub fn embed_icon() {
        println!(
            "cargo:warning=icône non embarquée : compilation croisée vers Windows non prise en charge"
        );
    }
}

mod commit {
    use super::{Command, Path, PathBuf};

    /// `GITHUB_SHA` en CI, sinon `git rev-parse HEAD` ; rien hors git (spec #33, L1). Le code
    /// vérifie et raccourcit la valeur (`short_commit`).
    pub fn pass_to_code() {
        println!("cargo:rerun-if-env-changed=GITHUB_SHA");
        let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..");
        watch_head(&repository);
        let commit = std::env::var("GITHUB_SHA")
            .ok()
            .filter(|sha| !sha.trim().is_empty())
            .or_else(|| git_head(&repository));
        if let Some(commit) = commit {
            println!("cargo:rustc-env=ARCANUM_COMMIT={}", commit.trim());
        }
    }

    fn git_head(repository: &Path) -> Option<String> {
        let output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(repository)
            .output()
            .ok()?;
        output
            .status
            .success()
            .then(|| String::from_utf8_lossy(&output.stdout).into_owned())
    }

    /// Recompile à chaque commit : `.git/HEAD`, et la référence qu'il désigne (`ref: refs/heads/…`).
    fn watch_head(repository: &Path) {
        let git = repository.join(".git");
        let head = git.join("HEAD");
        if !head.is_file() {
            return;
        }
        println!("cargo:rerun-if-changed={}", head.display());
        if let Ok(text) = std::fs::read_to_string(&head)
            && let Some(reference) = text.trim().strip_prefix("ref: ")
        {
            let path = reference
                .split('/')
                .fold(git.clone(), |path, part| path.join(part));
            // Une référence tassée (`packed-refs`) n'a pas de fichier : on suit alors celui-ci.
            let watched = if path.is_file() {
                path
            } else {
                git.join("packed-refs")
            };
            println!("cargo:rerun-if-changed={}", watched.display());
        }
    }
}
