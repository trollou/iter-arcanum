//! Scénarios de la spec #7 (support de macOS).
//! Aucune règle du livret ici : chaque test cite le cas de la spec.
//! Les fonctions pures tournent sur tous les systèmes ; les tests marqués macOS ne tournent que dans la CI macOS.

use std::collections::HashMap;
use std::path::PathBuf;

use arcanum_i18n::{Catalog, Language};
use arcanum_ui::{
    KEY_BINDINGS, KeyPlatform, LogDirSource, Shortcut, apple_system_language, choose_logs_dir,
    hover_text, key_label_on, macos_logs_dir, repository_assets_dir,
};

fn catalog(code: &str) -> Catalog {
    Catalog::load(
        &repository_assets_dir().join("i18n"),
        &Language(code.into()),
    )
    .expect("fichier de libellés du dépôt")
}

fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
    let map: HashMap<String, String> = pairs
        .iter()
        .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
        .collect();
    move |key| map.get(key).cloned()
}

fn label(shortcut: Shortcut, catalog: &Catalog, platform: KeyPlatform) -> String {
    let binding = KEY_BINDINGS
        .iter()
        .find(|binding| binding.shortcut == shortcut)
        .expect("touche dans la table");
    key_label_on(binding, catalog, platform)
}

// --- Langue du système (C4, L1) ---

#[test]
fn apple_language_reads_first_preferred() {
    // Spec #7 C4 : la première langue préférée, même avec LANG dans l'environnement.
    let lang = env(&[("LANG", "fr_FR.UTF-8")]);
    assert_eq!(
        apple_system_language(Some("en-GB"), &lang),
        Some("en".into())
    );
    assert_eq!(
        apple_system_language(Some("fr-FR"), &lang),
        Some("fr".into())
    );
    assert_eq!(
        apple_system_language(Some("zh-Hans-CN"), &lang),
        Some("zh".into())
    );
    // Sans fichier de libellés, `launch_catalog` passera à l'anglais (spec 076).
    assert_eq!(
        apple_system_language(Some("de-DE"), &lang),
        Some("de".into())
    );
    // L1 : sans tiret.
    assert_eq!(
        apple_system_language(Some("fr"), env(&[])),
        Some("fr".into())
    );
}

#[test]
fn apple_language_falls_back_to_locale_variables() {
    // Spec #7 C4, L2 : liste vide ou illisible, repli sur LC_ALL, LC_MESSAGES, LANG.
    let lang = env(&[("LANG", "fr_FR.UTF-8")]);
    assert_eq!(apple_system_language(None, &lang), Some("fr".into()));
    assert_eq!(apple_system_language(Some(""), &lang), Some("fr".into()));
}

#[test]
fn apple_language_without_anything_is_none() {
    // Spec #7 C4
    assert_eq!(apple_system_language(None, env(&[])), None);
}

// --- Dossier des logs (C6) ---

#[test]
fn macos_logs_dir_from_home() {
    // Spec #7 C6 : composant par composant, sans séparateur en dur (ADR 0004).
    let home = PathBuf::from("maison");
    assert_eq!(
        macos_logs_dir(&home),
        home.join("Library").join("Logs").join("iter-arcanum")
    );
}

#[test]
fn macos_logs_dir_is_user_choice() {
    // Spec #7 C6 : sans variable ni cargo, le dossier de l'utilisateur.
    let home = PathBuf::from("maison");
    let choice = choose_logs_dir(None, None, Some(home.clone())).expect("dossier des logs");
    assert_eq!(choice.source, LogDirSource::UserData);
    if cfg!(target_os = "macos") {
        assert_eq!(choice.dir, macos_logs_dir(&home));
    }
}

// --- Libellé des touches (C7 à C9) ---

#[test]
fn mac_shortcut_label_reads_cmd() {
    // Spec #7 C7, en français comme en anglais.
    for code in ["fr", "en"] {
        let catalog = catalog(code);
        assert_eq!(label(Shortcut::Undo, &catalog, KeyPlatform::Mac), "Cmd+Z");
        assert_eq!(
            label(Shortcut::Preferences, &catalog, KeyPlatform::Mac),
            "Cmd+,"
        );
        assert_eq!(label(Shortcut::Quit, &catalog, KeyPlatform::Mac), "Cmd+Q");
    }
    // C9 : l'infobulle reprend le libellé.
    let catalog = catalog("fr");
    let undo = label(Shortcut::Undo, &catalog, KeyPlatform::Mac);
    let text = hover_text(
        Some("Reprendre votre dernière action"),
        Some(&undo),
        &catalog,
    )
    .expect("infobulle");
    assert!(text.ends_with("Raccourci : Cmd+Z"), "{text}");
}

#[test]
fn other_shortcut_label_reads_ctrl() {
    // Spec #7 C7 : Windows et Linux inchangés.
    let catalog = catalog("fr");
    assert_eq!(
        label(Shortcut::Undo, &catalog, KeyPlatform::Other),
        "Ctrl+Z"
    );
    assert_eq!(
        label(Shortcut::Preferences, &catalog, KeyPlatform::Other),
        "Ctrl+,"
    );
}

#[test]
fn plain_keys_label_same_on_mac() {
    // Spec #7 C8 : sans modificateur, même libellé partout.
    let catalog = catalog("fr");
    for shortcut in [
        Shortcut::Cancel,
        Shortcut::EndTurn,
        Shortcut::Confirm,
        Shortcut::Offers,
        Shortcut::Interact,
        Shortcut::EnterSite,
        Shortcut::Take(1),
    ] {
        assert_eq!(
            label(shortcut, &catalog, KeyPlatform::Mac),
            label(shortcut, &catalog, KeyPlatform::Other),
            "{shortcut:?}"
        );
    }
    assert_eq!(label(Shortcut::Cancel, &catalog, KeyPlatform::Mac), "Échap");
}

// --- Sur un vrai Mac : CI macOS seulement (C2, C3, C5) ---

#[cfg(target_os = "macos")]
mod on_macos {
    use super::env;
    use arcanum_ui::{
        macos_preferred_language, macos_system_language, user_base_dir, user_config_dir,
        user_data_dir, user_logs_dir,
    };
    use std::path::{Path, PathBuf};

    fn ends_with(path: &Path, parts: &[&str]) -> bool {
        path.ends_with(parts.iter().collect::<PathBuf>())
    }

    #[test]
    fn macos_preferred_language_is_read() {
        // Spec #7 C2 : le runner est en anglais, on vérifie que la lecture aboutit.
        let language = macos_preferred_language().expect("langue préférée");
        assert!(!language.is_empty());
    }

    #[test]
    fn macos_system_language_ignores_lang_variable() {
        // Spec #7 C3 : LANG en français ne l'emporte pas sur la langue préférée (anglais).
        assert_eq!(
            macos_system_language(env(&[("LANG", "fr_FR.UTF-8")])),
            Some("en".into())
        );
    }

    #[test]
    fn macos_user_dirs_in_application_support() {
        // Spec #7 C5
        let parts = ["Library", "Application Support", "iter-arcanum"];
        let data = user_data_dir().expect("dossier des données");
        let config = user_config_dir().expect("dossier de configuration");
        assert!(ends_with(&data, &parts), "{}", data.display());
        assert!(ends_with(&config, &parts), "{}", config.display());
    }

    #[test]
    fn macos_user_logs_dir_in_library_logs() {
        // Spec #7 C5
        let logs = user_logs_dir(&user_base_dir().expect("dossier personnel"));
        assert!(
            ends_with(&logs, &["Library", "Logs", "iter-arcanum"]),
            "{}",
            logs.display()
        );
    }
}
