//! Licences des sons (spec 081, C25) : chaque fichier `.ogg` de `assets/sounds/` a sa ligne dans
//! `LICENSES.md`, avec une licence permise, et chaque ligne nomme un fichier présent.

use std::fs;
use std::path::Path;

/// Fichier des licences, à la racine du dossier des sons.
pub const LICENSES_FILE: &str = "LICENSES.md";

/// Licences libres permises pour un son (ADR 0010).
pub const FREE_LICENSES: [&str; 6] = [
    "CC0",
    "CC BY 3.0",
    "CC BY 4.0",
    "CC BY-SA 3.0",
    "CC BY-SA 4.0",
    "OGA-BY 3.0",
];

/// Licences non commerciales, permises en second choix seulement (Q12).
/// Sampling+ 1.0 : le fichier tel quel ne se distribue que pour un usage non commercial (Mathieu,
/// 2026-11-20, pour le hibou).
pub const NON_COMMERCIAL_LICENSES: [&str; 5] = [
    "CC BY-NC 3.0",
    "CC BY-NC 4.0",
    "CC BY-NC-SA 3.0",
    "CC BY-NC-SA 4.0",
    "Sampling+ 1.0",
];

/// Ce que la vérification a trouvé.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LicenseReport {
    /// Fichiers relevés par `LICENSES.md` (chemins relatifs au dossier des sons, avec `/`).
    pub files: Vec<String>,
    /// Ceux sous licence non commerciale, à remplacer si le projet change d'avis.
    pub non_commercial: Vec<String>,
    /// Les manquements, un par phrase.
    pub errors: Vec<String>,
}

/// Vérifie le dossier des sons `dir` et son `LICENSES.md`.
pub fn check_licenses(dir: &Path) -> LicenseReport {
    let mut report = LicenseReport::default();
    let text = match fs::read_to_string(dir.join(LICENSES_FILE)) {
        Ok(text) => text,
        Err(error) => {
            report
                .errors
                .push(format!("{LICENSES_FILE} illisible : {error}"));
            return report;
        }
    };
    for (file, license) in rows(&text) {
        if !dir.join(&file).is_file() {
            report.errors.push(format!("{file} : ligne sans fichier"));
        }
        if NON_COMMERCIAL_LICENSES.contains(&license.as_str()) {
            report.non_commercial.push(file.clone());
        } else if !FREE_LICENSES.contains(&license.as_str()) {
            report
                .errors
                .push(format!("{file} : licence non permise ({license})"));
        }
        report.files.push(file);
    }
    let mut sounds = Vec::new();
    ogg_files(dir, "", &mut sounds);
    for sound in sounds {
        if !report.files.contains(&sound) {
            report
                .errors
                .push(format!("{sound} : aucune ligne dans {LICENSES_FILE}"));
        }
    }
    report
}

/// Les lignes du tableau : fichier (1re colonne) et licence (5e), sans l'en-tête ni le trait.
fn rows(text: &str) -> Vec<(String, String)> {
    text.lines()
        .filter(|line| line.trim_start().starts_with('|'))
        .map(|line| {
            line.trim()
                .trim_matches('|')
                .split('|')
                .map(|cell| cell.trim().trim_matches('`').to_owned())
                .collect::<Vec<_>>()
        })
        .filter(|cells| cells.len() >= 5)
        .filter(|cells| cells[0] != "Fichier" && !cells[0].starts_with("---"))
        .map(|cells| (cells[0].clone(), cells[4].clone()))
        .collect()
}

/// Les `.ogg` sous `dir`, en chemins relatifs écrits avec `/` (la même forme sous Windows).
fn ogg_files(dir: &Path, prefix: &str, found: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if path.is_dir() {
            ogg_files(&path, &format!("{prefix}{name}/"), found);
        } else if path.extension().is_some_and(|ext| ext == "ogg") {
            found.push(format!("{prefix}{name}"));
        }
    }
}
