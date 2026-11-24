//! Garde-fou du dépôt public (spec 082, C4 et C5) : aucun matériel de tiers (livret, FAQ, compendium,
//! scans et leurs transcriptions) ne doit être suivi par git. Le livret et les scans vivent dans le dépôt
//! privé voisin `../mage-knight/docs/regles/`, jamais ici.
//!
//! Lancé par `scripts/verifier.sh` (donc avant chaque push) et par la CI : une fuite poussée resterait dans
//! l'historique public.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Taille au-delà de laquelle un fichier est refusé (le plus gros aujourd'hui, `music/night.ogg`, fait 3 Mo).
const MAX_SIZE: u64 = 5 * 1024 * 1024;

/// Dossiers de matériel de tiers, refusés en entier.
const THIRD_PARTY_FOLDERS: [&str; 2] = ["docs/regles/", "scripts/faq-compendium/"];

/// Dossiers où vivent nos images matricielles (illustrations générées, captures et maquettes des specs).
const IMAGE_FOLDERS: [&str; 2] = ["assets/", "docs/specs/images/"];

const RASTER_EXTENSIONS: [&str; 8] = ["png", "jpg", "jpeg", "webp", "gif", "bmp", "tif", "tiff"];

/// Règles qu'un fichier enfreint ; vide si le fichier peut être publié.
fn forbidden_file_reasons(path: &str, size: u64) -> Vec<&'static str> {
    let extension = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();
    let mut reasons = Vec::new();
    if THIRD_PARTY_FOLDERS.iter().any(|f| path.starts_with(f)) {
        reasons.push("dossier de matériel de tiers");
    }
    if extension == "pdf" {
        reasons.push("fichier PDF");
    }
    if RASTER_EXTENSIONS.contains(&extension.as_str())
        && !IMAGE_FOLDERS.iter().any(|f| path.starts_with(f))
    {
        reasons.push("image matricielle hors de assets/ et docs/specs/images/");
    }
    if size > MAX_SIZE {
        reasons.push("fichier de plus de 5 Mo");
    }
    reasons
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Fichiers suivis par git, avec leur taille.
fn tracked_files(root: &Path) -> Vec<(String, u64)> {
    let output = Command::new("git")
        .args(["-c", "core.quotepath=off", "ls-files", "-z"])
        .current_dir(root)
        .output()
        .expect("git ls-files doit tourner à la racine du dépôt");
    assert!(output.status.success(), "git ls-files a échoué");
    String::from_utf8(output.stdout)
        .expect("chemins en UTF-8")
        .split('\0')
        .filter(|path| !path.is_empty())
        .map(|path| {
            // Un fichier suivi mais supprimé de la copie de travail compte pour 0 : il n'y a rien à publier.
            let size = std::fs::metadata(root.join(path)).map_or(0, |m| m.len());
            (path.to_owned(), size)
        })
        .collect()
}

// C4 — Spec 082 : le matériel de tiers est refusé, avec la règle en cause.
#[test]
fn rules_folder_is_refused() {
    assert!(!forbidden_file_reasons("docs/regles/livret/p07.md", 4_000).is_empty());
}

#[test]
fn faq_compendium_script_is_refused() {
    assert!(!forbidden_file_reasons("scripts/faq-compendium/convertir.py", 2_000).is_empty());
}

#[test]
fn pdf_is_refused_anywhere() {
    assert!(!forbidden_file_reasons("docs/livret.pdf", 10_000).is_empty());
    assert!(!forbidden_file_reasons("assets/notes.PDF", 10_000).is_empty());
}

#[test]
fn raster_image_outside_assets_and_spec_images_is_refused() {
    assert!(!forbidden_file_reasons("docs/regles-copie/scan.jpg", 50_000).is_empty());
    assert!(!forbidden_file_reasons("scripts/carte.webp", 50_000).is_empty());
}

#[test]
fn our_images_are_accepted() {
    assert_eq!(
        forbidden_file_reasons("docs/specs/images/072-illustrations/combat.png", 500_000),
        Vec::<&str>::new()
    );
    assert_eq!(
        forbidden_file_reasons("assets/images/card-art/march.png", 900_000),
        Vec::<&str>::new()
    );
    assert_eq!(
        forbidden_file_reasons("assets/images/icon/iter-arcanum.svg", 5_000),
        Vec::<&str>::new()
    );
}

#[test]
fn large_file_is_refused() {
    assert!(!forbidden_file_reasons("assets/sounds/music/long.ogg", MAX_SIZE + 1).is_empty());
    assert_eq!(
        forbidden_file_reasons("assets/sounds/music/night.ogg", 3_000_000),
        Vec::<&str>::new()
    );
}

#[test]
fn every_broken_rule_is_reported() {
    assert_eq!(
        forbidden_file_reasons("docs/regles/scans/carte.jpg", MAX_SIZE + 1).len(),
        3
    );
}

// C5 — Spec 082 : le dépôt n'a aucun fichier refusé ; sinon chaque fichier est listé avec sa règle.
#[test]
fn public_repository_has_no_third_party_material() {
    let root = workspace_root();
    let files = tracked_files(&root);
    assert!(
        files.iter().any(|(path, _)| path == "Cargo.toml"),
        "la liste des fichiers suivis devrait contenir Cargo.toml"
    );
    let refused: Vec<String> = files
        .iter()
        .flat_map(|(path, size)| {
            forbidden_file_reasons(path, *size)
                .into_iter()
                .map(move |reason| format!("{path} : {reason}"))
        })
        .collect();
    assert!(
        refused.is_empty(),
        "matériel de tiers ou fichier trop gros suivi par git :\n{}",
        refused.join("\n")
    );
}
