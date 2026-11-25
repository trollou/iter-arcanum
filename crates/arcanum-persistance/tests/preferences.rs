//! Scénarios de la spec 080 (préférences), fichier des préférences.
//! Aucune règle du livret ici : chaque test cite le cas de la spec.

use std::fs;

use arcanum_persistance::preferences::{PREFERENCES_FILE, Preferences, Volumes, load, save};
use log::Level;

// Logger de capture partagé avec arcanum-ui (issue privée 358).
#[path = "../../arcanum-ui/tests/support/logs.rs"]
mod logs;
use logs::logged;

#[test]
fn default_preferences_follow_system_and_window() {
    // Spec 080 C1
    let preferences = Preferences::default();
    assert_eq!(preferences.language, None);
    assert!(!preferences.fullscreen_at_launch);
}

#[test]
fn preferences_round_trip() {
    // Spec 080 C2
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let path = dir.path().join(PREFERENCES_FILE);
    let preferences = Preferences {
        language: Some("fr".to_owned()),
        fullscreen_at_launch: true,
        ..Preferences::default()
    };
    save(&path, &preferences).expect("écriture");
    assert_eq!(load(&path), Ok(preferences));
}

#[test]
fn preferences_tolerate_missing_and_unknown_fields() {
    // Spec 080 C3
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let path = dir.path().join(PREFERENCES_FILE);
    fs::write(&path, r#"{"version": 1, "language": "en", "music": 40}"#).expect("fichier");
    assert_eq!(
        load(&path),
        Ok(Preferences {
            language: Some("en".to_owned()),
            fullscreen_at_launch: false,
            ..Preferences::default()
        })
    );
}

#[test]
fn newer_preferences_version_is_read_with_warning() {
    // Spec 080 C4
    logged(Level::Warn); // installe la capture avant la lecture
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let path = dir.path().join(PREFERENCES_FILE);
    fs::write(
        &path,
        r#"{"version": 2, "language": "fr", "fullscreen_at_launch": true}"#,
    )
    .expect("fichier");
    assert_eq!(
        load(&path),
        Ok(Preferences {
            language: Some("fr".to_owned()),
            fullscreen_at_launch: true,
            ..Preferences::default()
        })
    );
    assert!(
        logged(Level::Warn)
            .iter()
            .any(|line| line.contains("version 2")),
        "un warn nomme la version"
    );
}

#[test]
fn saving_preferences_creates_dir_and_leaves_no_temp_file() {
    // Spec 080 C5
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let config = dir.path().join("config");
    let path = config.join(PREFERENCES_FILE);
    save(&path, &Preferences::default()).expect("écriture");
    let names: Vec<String> = fs::read_dir(&config)
        .expect("dossier créé")
        .map(|entry| {
            entry
                .expect("entrée")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    assert_eq!(names, vec![PREFERENCES_FILE.to_owned()]);
    let text = fs::read_to_string(&path).expect("fichier");
    assert!(text.contains("\"version\": 1"), "{text}");
}

#[test]
fn missing_preferences_file_is_reported_as_missing() {
    // Spec 080 C11 (lecture) : l'appelant distingue un fichier absent d'un fichier illisible.
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let path = dir.path().join(PREFERENCES_FILE);
    assert_eq!(
        load(&path),
        Err(arcanum_persistance::preferences::PreferencesError::Missing)
    );
}

#[test]
fn volumes_default_and_round_trip() {
    // Spec 081 C18
    assert_eq!(
        Volumes::default(),
        Volumes {
            master: 80,
            music: 50,
            ambience: 50,
            effects: 80
        }
    );
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let path = dir.path().join(PREFERENCES_FILE);
    let preferences = Preferences {
        volumes: Volumes {
            master: 100,
            music: 0,
            ambience: 30,
            effects: 65,
        },
        ..Preferences::default()
    };
    save(&path, &preferences).expect("écriture");
    assert_eq!(load(&path), Ok(preferences));
}

#[test]
fn volumes_tolerate_old_and_out_of_range_files() {
    // Spec 081 C19 : un fichier de la spec 080 se lit avec les volumes par défaut ; une valeur
    // au-delà de 100 est ramenée à 100, avec un `warn`.
    logged(Level::Warn); // installe la capture avant la lecture
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let path = dir.path().join(PREFERENCES_FILE);
    fs::write(&path, r#"{"version": 1, "language": "fr"}"#).expect("fichier");
    let old = load(&path).expect("lecture");
    assert_eq!(old.volumes, Volumes::default());

    fs::write(&path, r#"{"version": 1, "volumes": {"music": 140}}"#).expect("fichier");
    let loud = load(&path).expect("lecture");
    assert_eq!(
        loud.volumes,
        Volumes {
            music: 100,
            ..Volumes::default()
        }
    );
    assert!(
        logged(Level::Warn).iter().any(|line| line.contains("140")),
        "un warn nomme la valeur ramenée"
    );
}

#[test]
fn animations_default_to_on() {
    // Spec #5 C12 : un fichier sans le champ donne `animations` vrai ; un fichier qui l'a garde sa
    // valeur.
    assert!(Preferences::default().animations);
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let path = dir.path().join(PREFERENCES_FILE);
    fs::write(&path, r#"{"version": 1, "language": "fr"}"#).expect("fichier");
    assert!(load(&path).expect("lecture").animations);

    fs::write(&path, r#"{"version": 1, "animations": false}"#).expect("fichier");
    assert!(!load(&path).expect("lecture").animations);
    fs::write(&path, r#"{"version": 1, "animations": true}"#).expect("fichier");
    assert!(load(&path).expect("lecture").animations);
}
