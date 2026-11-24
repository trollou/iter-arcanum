//! Spec 081 : la banque de sons et les licences des fichiers (C16, C24, C25, L2).

#[path = "../../arcanum-ui/tests/support/logs.rs"]
mod logs;

use std::fs;
use std::path::{Path, PathBuf};

use arcanum_audio::{Ambience, Cue, LICENSES_FILE, Music, SoundBank, check_licenses};
use log::Level;
use tempfile::TempDir;

/// Un dossier des sons avec ces fichiers (contenu factice : la banque ne lit pas les fichiers).
fn sounds_dir(files: &[&str]) -> TempDir {
    let dir = TempDir::new().expect("dossier temporaire");
    for file in files {
        let path = dir.path().join(file);
        fs::create_dir_all(path.parent().expect("parent")).expect("dossier");
        fs::write(path, b"ogg").expect("fichier");
    }
    dir
}

fn names(paths: &[PathBuf]) -> Vec<String> {
    paths
        .iter()
        .map(|path| {
            path.file_name()
                .expect("nom")
                .to_string_lossy()
                .into_owned()
        })
        .collect()
}

fn warnings(lines: &[(Level, String)]) -> Vec<&str> {
    lines
        .iter()
        .filter(|(level, _)| *level == Level::Warn)
        .map(|(_, message)| message.as_str())
        .collect()
}

#[test]
fn bank_finds_files_and_variants() {
    // C24
    let dir = sounds_dir(&[
        "music/day.ogg",
        "effects/footstep_2.ogg",
        "effects/footstep_1.ogg",
    ]);
    let mut bank = SoundBank::scan(dir.path());
    assert_eq!(
        bank.music(Music::Day),
        Some(dir.path().join("music/day.ogg").as_path())
    );
    assert_eq!(bank.music(Music::Night), None);
    assert_eq!(bank.ambience(Ambience::Day), None);
    assert_eq!(
        names(bank.variants(Cue::Footstep)),
        vec!["footstep_1.ogg", "footstep_2.ogg"]
    );

    // Un effet sans fichier n'est pas une erreur : un seul `warn`, qui le nomme.
    let (_, lines) = logs::logs_of(|| {
        assert_eq!(bank.cue_file(Cue::Owl, 0), None);
        assert_eq!(bank.cue_file(Cue::Owl, 1), None);
    });
    let warned = warnings(&lines);
    assert_eq!(warned.len(), 1, "{lines:?}");
    assert!(warned[0].contains("owl"), "{warned:?}");
}

#[test]
fn variants_rotate() {
    // C16 : trois variantes, quatre lectures (numérotées par le directeur) : 1, 2, 3, 1.
    let dir = sounds_dir(&[
        "effects/wound_1.ogg",
        "effects/wound_2.ogg",
        "effects/wound_3.ogg",
    ]);
    let mut bank = SoundBank::scan(dir.path());
    let played: Vec<String> = (0..4)
        .map(|nth| {
            let file = bank.cue_file(Cue::Wound, nth).expect("variante");
            file.file_name()
                .expect("nom")
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    assert_eq!(
        played,
        vec!["wound_1.ogg", "wound_2.ogg", "wound_3.ogg", "wound_1.ogg"]
    );
}

#[test]
fn single_file_cue_has_one_variant() {
    // Un effet sans numéro (`rooster.ogg`) est sa seule variante.
    let dir = sounds_dir(&["effects/rooster.ogg", "ambience/night.ogg"]);
    let mut bank = SoundBank::scan(dir.path());
    assert_eq!(names(bank.variants(Cue::Rooster)), vec!["rooster.ogg"]);
    assert!(bank.cue_file(Cue::Rooster, 5).is_some());
    assert!(bank.ambience(Ambience::Night).is_some());
}

#[test]
fn missing_sounds_dir_gives_empty_bank() {
    // L2 : banque vide, un seul `info`.
    let dir = TempDir::new().expect("dossier temporaire");
    let missing = dir.path().join("sounds");
    let (bank, lines) = logs::logs_of(|| SoundBank::scan(&missing));
    assert!(Music::ALL.iter().all(|music| bank.music(*music).is_none()));
    assert!(Cue::ALL.iter().all(|cue| bank.variants(*cue).is_empty()));
    let infos: Vec<_> = lines
        .iter()
        .filter(|(level, _)| *level == Level::Info)
        .collect();
    assert_eq!(infos.len(), 1, "{lines:?}");
}

/// `LICENSES.md` fait de ces lignes de tableau.
fn with_licenses(dir: &TempDir, rows: &[&str]) {
    let mut text = String::from(
        "# Licences\n\n| Fichier | Titre | Auteur | Source | Licence | Modifications |\n|---|---|---|---|---|---|\n",
    );
    for row in rows {
        text.push_str(row);
        text.push('\n');
    }
    fs::write(dir.path().join(LICENSES_FILE), text).expect("licences");
}

#[test]
fn license_check_finds_missing_and_bad_lines() {
    // C25, sur un dossier d'essai : un son sans ligne, une licence interdite, une ligne sans
    // fichier, et un son NC compté à part.
    let dir = sounds_dir(&[
        "music/day.ogg",
        "effects/owl.ogg",
        "effects/block.ogg",
        "effects/miss.ogg",
    ]);
    with_licenses(
        &dir,
        &[
            "| music/day.ogg | Jour | A | https://exemple.org/1 | CC0 | coupé |",
            "| effects/owl.ogg | Hibou | B | https://exemple.org/2 | CC BY-ND 4.0 | — |",
            "| effects/miss.ogg | Souffle | C | https://exemple.org/3 | CC BY-NC 4.0 | — |",
            "| effects/rooster.ogg | Coq | D | https://exemple.org/4 | CC BY 4.0 | — |",
        ],
    );
    let report = check_licenses(dir.path());
    assert_eq!(report.non_commercial, vec!["effects/miss.ogg"]);
    assert_eq!(report.errors.len(), 3, "{:?}", report.errors);
    for named in [
        "effects/block.ogg",
        "effects/owl.ogg",
        "effects/rooster.ogg",
    ] {
        assert!(
            report.errors.iter().any(|error| error.contains(named)),
            "{named} : {:?}",
            report.errors
        );
    }
}

#[test]
fn every_sound_has_a_permitted_license() {
    // C25, sur les vrais fichiers du jeu.
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/sounds");
    assert!(dir.join(LICENSES_FILE).is_file(), "LICENSES.md manquant");
    let report = check_licenses(&dir);
    assert_eq!(report.errors, Vec::<String>::new());
    println!(
        "{} sons, dont {} sous licence non commerciale : {:?}",
        report.files.len(),
        report.non_commercial.len(),
        report.non_commercial
    );
}

/// Budget du dossier des sons (Q10).
const SOUNDS_BUDGET_BYTES: u64 = 20 * 1024 * 1024;

fn size_of_dir(dir: &Path) -> u64 {
    fs::read_dir(dir)
        .expect("dossier lisible")
        .map(|entry| entry.expect("entrée").path())
        .map(|path| {
            if path.is_dir() {
                size_of_dir(&path)
            } else {
                fs::metadata(&path).expect("taille").len()
            }
        })
        .sum()
}

#[test]
fn every_cue_has_a_sound_within_budget() {
    // C26, sur les vrais fichiers du jeu : chaque musique, ambiance et effet a un fichier, et le
    // dossier fait 20 Mo au plus.
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/sounds");
    let bank = SoundBank::scan(&dir);
    let missing: Vec<String> = Music::ALL
        .into_iter()
        .filter(|music| bank.music(*music).is_none())
        .map(|music| format!("music/{}", music.file_stem()))
        .chain(
            Ambience::ALL
                .into_iter()
                .filter(|ambience| bank.ambience(*ambience).is_none())
                .map(|ambience| format!("ambience/{}", ambience.file_stem())),
        )
        .chain(
            Cue::ALL
                .into_iter()
                .filter(|cue| bank.variants(*cue).is_empty())
                .map(|cue| format!("effects/{}", cue.file_stem())),
        )
        .collect();
    assert_eq!(missing, Vec::<String>::new(), "sons sans fichier");
    let size = size_of_dir(&dir);
    assert!(
        size <= SOUNDS_BUDGET_BYTES,
        "{size} octets, au-delà de 20 Mo"
    );
}

#[test]
fn sampling_plus_counts_as_non_commercial() {
    // C25 : Sampling+ 1.0 (hibou de Freesound, Mathieu, 2026-11-20) est rangée avec les licences
    // NC de second choix.
    let dir = sounds_dir(&["effects/owl.ogg"]);
    with_licenses(
        &dir,
        &[
            "| effects/owl.ogg | Owl | Johnc | https://freesound.org/people/Johnc/sounds/20684/ | Sampling+ 1.0 | Mono |",
        ],
    );
    let report = check_licenses(dir.path());
    assert_eq!(report.errors, Vec::<String>::new());
    assert_eq!(report.non_commercial, vec!["effects/owl.ogg"]);
}
