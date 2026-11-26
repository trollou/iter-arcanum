//! Spec #52 : migration des sauvegardes, avec une chaîne de test (1 → 2 → 3, format courant 3) ;
//! la vraie chaîne est vide tant qu'aucun changement incompatible n'est fait. Chaque migration de
//! test ajoute sa marque à la date du résumé, pour voir lesquelles ont tourné et dans quel ordre.

use std::fs;
use std::path::{Path, PathBuf};

use arcanum_persistance::migration::{Formats, Migration};
use arcanum_persistance::{LoadError, list_with, load_with, save};
use serde_json::Value;

fn mark(mut value: Value, step: &str) -> Result<Value, String> {
    let saved_at = value
        .pointer_mut("/summary/saved_at")
        .ok_or("résumé sans date")?;
    *saved_at = Value::String(format!("{}|{step}", saved_at.as_str().unwrap_or_default()));
    Ok(value)
}

fn one_to_two(value: Value) -> Result<Value, String> {
    mark(value, "1→2")
}

fn two_to_three(value: Value) -> Result<Value, String> {
    mark(value, "2→3")
}

fn failing(_value: Value) -> Result<Value, String> {
    Err("aucune transformation cohérente".to_string())
}

const CHAIN: &[Migration] = &[one_to_two, two_to_three];

const TEST_FORMATS: Formats = Formats {
    oldest: 1,
    current: 3,
    chain: CHAIN,
};

/// Une vraie sauvegarde (témoin de la v0.1.0), réécrite dans `dir` avec le format `version`.
fn save_with_version(dir: &Path, name: &str, version: u64) -> PathBuf {
    let witness = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/temoins/v0.1.0/agilite.json");
    let mut value: Value =
        serde_json::from_str(&fs::read_to_string(witness).expect("témoin lisible"))
            .expect("témoin en JSON");
    value["version"] = Value::from(version);
    let path = dir.join(format!("{name}.json"));
    fs::write(&path, serde_json::to_string_pretty(&value).expect("JSON")).expect("écrite");
    path
}

fn saved_at_marks(path: &Path, formats: &Formats) -> String {
    let game = load_with(path, formats).expect("chargée");
    game.summary
        .saved_at
        .split_once('|')
        .map(|(_, marks)| marks.to_string())
        .unwrap_or_default()
}

// Spec #52, C2.
#[test]
fn old_save_runs_each_migration_once_in_order() {
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let path = save_with_version(dir.path(), "v1", 1);
    assert_eq!(saved_at_marks(&path, &TEST_FORMATS), "1→2|2→3");
    assert_eq!(load_with(&path, &TEST_FORMATS).expect("chargée").version, 3);
}

// Spec #52, C3.
#[test]
fn migration_starts_at_save_format() {
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let v2 = save_with_version(dir.path(), "v2", 2);
    let v3 = save_with_version(dir.path(), "v3", 3);
    assert_eq!(saved_at_marks(&v2, &TEST_FORMATS), "2→3");
    assert_eq!(saved_at_marks(&v3, &TEST_FORMATS), "");
}

// Spec #52, C4 (avec la chaîne de test ; la vraie : `newer_save_is_refused` de `sauvegarde.rs`).
#[test]
fn newer_save_is_refused() {
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let path = save_with_version(dir.path(), "v4", 4);
    assert_eq!(
        load_with(&path, &TEST_FORMATS),
        Err(LoadError::NewerVersion { found: 4 })
    );
}

// Spec #52, C5.
#[test]
fn failed_migration_leaves_file_untouched() {
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let path = save_with_version(dir.path(), "v1", 1);
    let before = fs::read(&path).expect("lue");
    let formats = Formats {
        chain: &[one_to_two, failing],
        ..TEST_FORMATS
    };
    assert_eq!(
        load_with(&path, &formats),
        Err(LoadError::NotMigratable {
            found: 1,
            reason: "aucune transformation cohérente".to_string()
        })
    );
    assert_eq!(fs::read(&path).expect("lue"), before);
}

// Spec #52, C6.
#[test]
fn migrated_save_is_written_in_current_format() {
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let path = save_with_version(dir.path(), "v1", 1);
    let before = fs::read(&path).expect("lue");
    let game = load_with(&path, &TEST_FORMATS).expect("chargée");
    let resaved = save(dir.path(), "v1", &game).expect("écrite");

    assert_ne!(resaved, path);
    let written: Value =
        serde_json::from_str(&fs::read_to_string(&resaved).expect("lue")).expect("JSON");
    assert_eq!(written["version"], 3);
    assert_eq!(fs::read(&path).expect("lue"), before);
}

// Spec #52, C7.
#[test]
fn save_older_than_oldest_supported_is_too_old() {
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let formats = Formats {
        oldest: 2,
        chain: &[two_to_three],
        ..TEST_FORMATS
    };
    let v1 = save_with_version(dir.path(), "v1", 1);
    let v0 = save_with_version(dir.path(), "v0", 0);
    assert_eq!(
        load_with(&v1, &formats),
        Err(LoadError::TooOld { found: 1 })
    );
    assert_eq!(
        load_with(&v0, &formats),
        Err(LoadError::TooOld { found: 0 })
    );

    let mut missing: Value = serde_json::from_slice(&fs::read(&v1).expect("lue")).expect("JSON");
    missing.as_object_mut().expect("objet").remove("version");
    fs::write(&v1, missing.to_string()).expect("écrite");
    assert!(matches!(
        load_with(&v1, &formats),
        Err(LoadError::Invalid(_))
    ));
}

// Spec #52, C8.
#[test]
fn list_shows_previous_saves_and_flags_others() {
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let formats = Formats {
        oldest: 2,
        chain: &[two_to_three],
        ..TEST_FORMATS
    };
    save_with_version(dir.path(), "v2", 2);
    save_with_version(dir.path(), "v3", 3);
    save_with_version(dir.path(), "v1", 1);
    save_with_version(dir.path(), "v4", 4);

    let entries = list_with(dir.path(), &formats);
    let summary = |stem: &str| {
        entries
            .iter()
            .find(|entry| entry.path.file_stem().and_then(|s| s.to_str()) == Some(stem))
            .map(|entry| entry.summary.clone())
            .expect("listée")
    };
    assert!(summary("v2").expect("migrée").saved_at.ends_with("|2→3"));
    assert!(summary("v3").is_ok());
    assert_eq!(summary("v1"), Err(LoadError::TooOld { found: 1 }));
    assert_eq!(summary("v4"), Err(LoadError::NewerVersion { found: 4 }));
}
