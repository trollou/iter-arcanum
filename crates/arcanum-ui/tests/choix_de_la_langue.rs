//! Scénarios de la spec 076 (choix de la langue).
//! Aucune règle du livret ici : chaque test cite le cas de la spec.
//! Arguments et variables d'environnement passent en paramètres : rien ne dépend de la machine.

mod support;

use std::collections::HashMap;
use std::path::PathBuf;

use arcanum_i18n::Catalog;
use arcanum_ui::{
    UNIX_LOCALE_VARIABLES, launch_catalog, locale_to_code, repository_assets_dir,
    requested_language, unix_system_language, windows_language_id_to_code,
};
use log::Level;
use support::logs::logs_of;

// --- Outils de test ---

fn has_line(lines: &[(Level, String)], level: Level, parts: &[&str]) -> bool {
    lines
        .iter()
        .any(|(l, message)| *l == level && parts.iter().all(|part| message.contains(part)))
}

fn args(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

fn repository() -> Vec<PathBuf> {
    vec![repository_assets_dir()]
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("assets")
        .join(name)
}

fn name(catalog: &Catalog) -> String {
    catalog.text("language.name")
}

fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
    let map: HashMap<String, String> = pairs
        .iter()
        .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
        .collect();
    move |key| map.get(key).cloned()
}

fn system(pairs: &[(&str, &str)]) -> Option<String> {
    unix_system_language(env(pairs))
}

// --- Paramètre --lang ---

#[test]
fn lang_argument_overrides_system_language() {
    // Spec 076 C1
    let catalog = launch_catalog(
        &args(&["--lang", "en"]),
        None,
        Some("fr".into()),
        &repository(),
    );
    assert_eq!(name(&catalog), "English");
}

#[test]
fn lang_argument_accepts_equals_form() {
    // Spec 076 C2
    assert_eq!(
        requested_language(&args(&["--lang=fr"])),
        Some("fr".to_owned())
    );
    let catalog = launch_catalog(
        &args(&["--lang=fr"]),
        None,
        Some("en".into()),
        &repository(),
    );
    assert_eq!(name(&catalog), "Français");
}

#[test]
fn lang_argument_is_read_like_a_locale() {
    // Spec 076 C3
    for value in ["FR", "fr_FR", "fr-FR"] {
        let catalog = launch_catalog(
            &args(&["--lang", value]),
            None,
            Some("en".into()),
            &repository(),
        );
        assert_eq!(name(&catalog), "Français", "--lang {value}");
    }
}

#[test]
fn unknown_lang_argument_falls_back_to_english_with_error() {
    // Spec 076 C4
    let (catalog, lines) = logs_of(|| {
        launch_catalog(
            &args(&["--lang", "de"]),
            None,
            Some("fr".into()),
            &repository(),
        )
    });
    assert_eq!(name(&catalog), "English");
    assert!(
        has_line(
            &lines,
            Level::Error,
            &[
                "langue demandée introuvable : de",
                "en, fr",
                "repli en anglais"
            ]
        ),
        "{lines:?}"
    );
}

#[test]
fn lang_argument_without_value_is_ignored() {
    // Spec 076 C5 (Q3 : la langue du système s'applique)
    let (catalog, lines) =
        logs_of(|| launch_catalog(&args(&["--lang"]), None, Some("fr".into()), &repository()));
    assert_eq!(name(&catalog), "Français");
    assert!(has_line(&lines, Level::Error, &["--lang"]), "{lines:?}");
    assert_eq!(requested_language(&args(&["--lang"])), None);
}

#[test]
fn last_lang_argument_wins() {
    // Spec 076 C6
    let (catalog, lines) = logs_of(|| {
        launch_catalog(
            &args(&["--lang", "fr", "--lang", "en"]),
            None,
            Some("fr".into()),
            &repository(),
        )
    });
    assert_eq!(name(&catalog), "English");
    assert!(has_line(&lines, Level::Warn, &["--lang"]), "{lines:?}");
}

#[test]
fn unknown_argument_is_ignored_with_warning() {
    // Spec 076 C7
    let (catalog, lines) = logs_of(|| {
        launch_catalog(
            &args(&["--plein-ecran", "--lang", "en"]),
            None,
            Some("fr".into()),
            &repository(),
        )
    });
    assert_eq!(name(&catalog), "English");
    assert!(
        has_line(&lines, Level::Warn, &["--plein-ecran"]),
        "{lines:?}"
    );
}

// --- Langue du système ---

#[test]
fn system_language_french_is_used() {
    // Spec 076 C8
    let language = system(&[("LANG", "fr_FR.UTF-8")]);
    assert_eq!(
        name(&launch_catalog(&[], None, language, &repository())),
        "Français"
    );
}

#[test]
fn system_language_english_is_used() {
    // Spec 076 C9
    let language = system(&[("LANG", "en_US.UTF-8")]);
    assert_eq!(
        name(&launch_catalog(&[], None, language, &repository())),
        "English"
    );
}

#[test]
fn system_language_without_file_falls_back_to_english() {
    // Spec 076 C10 : info, pas error, ce n'est pas une faute.
    let language = system(&[("LANG", "de_DE.UTF-8")]);
    let (catalog, lines) = logs_of(|| launch_catalog(&[], None, language, &repository()));
    assert_eq!(name(&catalog), "English");
    assert!(has_line(&lines, Level::Info, &["de"]), "{lines:?}");
    assert!(
        !lines.iter().any(|(level, _)| *level == Level::Error),
        "{lines:?}"
    );
}

#[test]
fn unix_locale_variables_priority() {
    // Spec 076 C11
    assert_eq!(UNIX_LOCALE_VARIABLES, ["LC_ALL", "LC_MESSAGES", "LANG"]);
    let all = [
        ("LC_ALL", "en_US.UTF-8"),
        ("LC_MESSAGES", "fr_FR.UTF-8"),
        ("LANG", "de_DE.UTF-8"),
    ];
    assert_eq!(system(&all), Some("en".to_owned()));
    assert_eq!(system(&all[1..]), Some("fr".to_owned()));
    assert_eq!(system(&all[2..]), Some("de".to_owned()));
    assert_eq!(
        system(&[("LC_ALL", ""), ("LANG", "fr_FR.UTF-8")]),
        Some("fr".to_owned())
    );
}

#[test]
fn locale_value_gives_language_code() {
    // Spec 076, « D'une valeur de locale à un code »
    assert_eq!(locale_to_code("fr_FR.UTF-8"), Some("fr".to_owned()));
    assert_eq!(locale_to_code("en-GB"), Some("en".to_owned()));
    assert_eq!(locale_to_code("de_DE@euro"), Some("de".to_owned()));
    assert_eq!(locale_to_code("fr.UTF-8"), Some("fr".to_owned()));
    assert_eq!(locale_to_code("C"), Some("c".to_owned()));
    assert_eq!(locale_to_code("POSIX"), Some("posix".to_owned()));
    assert_eq!(locale_to_code(""), None);
}

#[test]
fn no_system_language_falls_back_to_english() {
    // Spec 076 C12
    assert_eq!(system(&[]), None);
    for language in [None, system(&[("LANG", "C")])] {
        let (catalog, lines) =
            logs_of(|| launch_catalog(&[], None, language.clone(), &repository()));
        assert_eq!(name(&catalog), "English", "{language:?}");
        assert!(
            has_line(&lines, Level::Info, &["langue : en (repli)"]),
            "{lines:?}"
        );
    }
}

#[test]
fn windows_language_id_to_code_table() {
    // Spec 076 C13 (windows_language_id_to_code)
    assert_eq!(windows_language_id_to_code(0x040C), Some("fr"));
    assert_eq!(windows_language_id_to_code(0x0C0C), Some("fr"));
    assert_eq!(windows_language_id_to_code(0x0409), Some("en"));
    assert_eq!(windows_language_id_to_code(0x0809), Some("en"));
    assert_eq!(windows_language_id_to_code(0x0407), None);
}

#[cfg(windows)]
#[test]
fn windows_system_language_is_read() {
    // Spec 076 C14 : rend sans planter ; la valeur dépend de la machine.
    let _ = arcanum_ui::windows_system_language();
}

// --- Chargement et logs ---

#[test]
fn chosen_language_is_logged() {
    // Spec 076 C15, en français (L3)
    let (_, lines) =
        logs_of(|| launch_catalog(&args(&["--lang", "en"]), None, None, &repository()));
    assert!(
        has_line(&lines, Level::Info, &["langue : en (paramètre --lang)"]),
        "{lines:?}"
    );
    let (_, lines) = logs_of(|| launch_catalog(&[], None, Some("fr".into()), &repository()));
    assert!(
        has_line(&lines, Level::Info, &["langue : fr (langue du système)"]),
        "{lines:?}"
    );
    let (_, lines) = logs_of(|| launch_catalog(&[], None, None, &repository()));
    assert!(
        has_line(&lines, Level::Info, &["langue : en (repli)"]),
        "{lines:?}"
    );
}

#[test]
fn unloadable_language_falls_back_to_english_then_visible_keys() {
    // Spec 076 L1 : fr.json invalide, en.json valide.
    let (catalog, lines) = logs_of(|| {
        launch_catalog(
            &args(&["--lang", "fr"]),
            None,
            None,
            &[fixture("francais-invalide")],
        )
    });
    assert_eq!(name(&catalog), "English fixture");
    assert!(has_line(&lines, Level::Error, &["fr.json"]), "{lines:?}");

    // Ni fr.json ni en.json chargeables : clés visibles (spec 002 L6).
    let catalog = launch_catalog(&args(&["--lang", "fr"]), None, None, &[fixture("invalide")]);
    assert_eq!(catalog.text("app.title"), "⟦app.title⟧");
}

#[test]
fn missing_assets_dir_gives_visible_keys_whatever_the_language() {
    // Spec 076 L2
    for language in ["fr", "en"] {
        let catalog = launch_catalog(
            &args(&["--lang", language]),
            None,
            None,
            &[fixture("absent")],
        );
        assert_eq!(catalog.text("app.title"), "⟦app.title⟧");
    }
}
