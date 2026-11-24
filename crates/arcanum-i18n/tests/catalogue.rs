//! Scénarios de la spec 001 (libellés) : chargement et résolution.
//! Les JSON sont écrits dans les tests ; les vrais fichiers sont testés dans `fichiers_de_langue.rs`.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use arcanum_i18n::{Catalog, Gap, Language, LoadError, available_languages};

fn fr() -> Language {
    Language("fr".to_owned())
}

fn catalog(json: &str) -> Catalog {
    match Catalog::from_json_str(fr(), json) {
        Ok(catalog) => catalog,
        Err(error) => panic!("le JSON de test devrait se charger : {error}"),
    }
}

fn load_error(json: &str) -> LoadError {
    match Catalog::from_json_str(fr(), json) {
        Ok(_) => panic!("le chargement aurait dû échouer"),
        Err(error) => error,
    }
}

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

fn set(items: &[&str]) -> BTreeSet<String> {
    items.iter().map(|item| (*item).to_owned()).collect()
}

// Spec 001, C1
#[test]
fn nested_keys_resolve_by_dotted_path() {
    let catalog = catalog(r#"{"card": {"march": {"name": "Marche"}}}"#);
    assert_eq!(catalog.text("card.march.name"), "Marche");
}

// Spec 001, C2
#[test]
fn named_parameter_is_substituted() {
    let catalog = catalog(r#"{"card": {"march": {"basic": "Mouvement {amount}"}}}"#);
    assert_eq!(
        catalog.format("card.march.basic", &[("amount", "2")]),
        "Mouvement 2"
    );
}

// Spec 001, C3
#[test]
fn parameters_are_matched_by_name() {
    let catalog =
        catalog(r#"{"combat": {"hit": "{attacker} frappe {target}, puis {attacker} recule"}}"#);
    assert_eq!(
        catalog.format(
            "combat.hit",
            &[("target", "l'orque"), ("attacker", "Tovak")]
        ),
        "Tovak frappe l'orque, puis Tovak recule"
    );
}

// Spec 001, C4
#[test]
fn double_braces_are_literal() {
    let catalog = catalog(r#"{"help": {"syntax": "Écrire {{amount}} pour {what}"}}"#);
    assert_eq!(
        catalog.format("help.syntax", &[("what", "un paramètre")]),
        "Écrire {amount} pour un paramètre"
    );
    assert_eq!(catalog.parameters("help.syntax"), Some(set(&["what"])));
}

// Spec 001, C5
#[test]
fn missing_key_is_visible_and_recorded() {
    let catalog = catalog(r#"{"card": {"march": {"name": "Marche"}}}"#);
    assert_eq!(catalog.text("card.rage.name"), "⟦card.rage.name⟧");
    assert_eq!(
        catalog.format("card.rage.basic", &[("amount", "2")]),
        "⟦card.rage.basic⟧"
    );
    assert_eq!(
        catalog.gaps(),
        vec![
            Gap::MissingKey("card.rage.basic".to_owned()),
            Gap::MissingKey("card.rage.name".to_owned()),
        ]
    );
}

// Spec 001, C6
#[test]
fn missing_key_is_recorded_once() {
    let catalog = catalog(r#"{"card": {"march": {"name": "Marche"}}}"#);
    for _ in 0..3 {
        catalog.text("card.rage.name");
    }
    assert_eq!(
        catalog.gaps(),
        vec![Gap::MissingKey("card.rage.name".to_owned())]
    );
}

// Spec 001, C7
#[test]
fn missing_parameter_stays_visible() {
    let catalog = catalog(r#"{"card": {"march": {"basic": "Mouvement {amount}"}}}"#);
    assert_eq!(catalog.text("card.march.basic"), "Mouvement {amount}");
    assert_eq!(
        catalog.gaps(),
        vec![Gap::MissingParameter {
            key: "card.march.basic".to_owned(),
            parameter: "amount".to_owned(),
        }]
    );
}

// Spec 001, C8
#[test]
fn extra_parameter_is_ignored() {
    let catalog = catalog(r#"{"card": {"march": {"name": "Marche"}}}"#);
    assert_eq!(
        catalog.format("card.march.name", &[("amount", "2")]),
        "Marche"
    );
    assert!(catalog.gaps().is_empty());
}

// Spec 001, C9
#[test]
fn catalog_lists_keys_and_parameters() {
    let catalog = catalog(
        r#"{
            "language": {"name": "Français"},
            "card": {"march": {"name": "Marche", "basic": "Mouvement {amount}, {amount} {bonus}"}}
        }"#,
    );
    assert_eq!(
        catalog.keys(),
        set(&["card.march.basic", "card.march.name", "language.name"])
    );
    assert_eq!(
        catalog.parameters("card.march.basic"),
        Some(set(&["amount", "bonus"]))
    );
    assert_eq!(catalog.parameters("card.march.name"), Some(BTreeSet::new()));
    assert_eq!(catalog.parameters("card.rage.name"), None);
}

// Spec 001, C10
#[test]
fn load_from_directory_and_list_languages() {
    let dir = fixtures().join("langues");
    let catalog = match Catalog::load(&dir, &fr()) {
        Ok(catalog) => catalog,
        Err(error) => panic!("fr.json devrait se charger : {error}"),
    };
    assert_eq!(catalog.language(), &fr());
    assert_eq!(catalog.text("language.name"), "Français");
    assert_eq!(
        available_languages(&dir),
        Ok(vec![Language("en".to_owned()), fr()])
    );
}

// Spec 001, L1
#[test]
fn non_string_leaf_is_refused() {
    for leaf in ["3", "true", "null", r#"["Marche"]"#] {
        let json = format!(r#"{{"card": {{"march": {{"name": {leaf}}}}}}}"#);
        assert_eq!(
            load_error(&json),
            LoadError::NotAString {
                key: "card.march.name".to_owned()
            },
            "feuille {leaf}"
        );
    }
}

// Spec 001, L2
#[test]
fn empty_label_is_refused() {
    assert_eq!(
        load_error(r#"{"card": {"march": {"name": ""}}}"#),
        LoadError::EmptyLabel {
            key: "card.march.name".to_owned()
        }
    );
}

// Spec 001, L3
#[test]
fn invalid_key_or_parameter_name_is_refused() {
    for segment in ["March", "la marche", "la-marche", "1march", ""] {
        let json = format!(r#"{{"card": {{"{segment}": "Marche"}}}}"#);
        assert_eq!(
            load_error(&json),
            LoadError::InvalidKeyName {
                key: format!("card.{segment}")
            },
            "segment « {segment} »"
        );
    }
    for parameter in ["Amount", "the amount", "the-amount", "1amount"] {
        let json = format!(r#"{{"card": {{"basic": "Mouvement {{{parameter}}}"}}}}"#);
        assert_eq!(
            load_error(&json),
            LoadError::InvalidParameterName {
                key: "card.basic".to_owned(),
                parameter: parameter.to_owned(),
            },
            "paramètre « {parameter} »"
        );
    }
}

// Spec 001, L4
#[test]
fn malformed_braces_are_refused() {
    for label in ["Mouvement {amount", "Mouvement {}", "Mouvement }", "{"] {
        let json = format!(r#"{{"card": {{"basic": "{label}"}}}}"#);
        assert_eq!(
            load_error(&json),
            LoadError::MalformedBraces {
                key: "card.basic".to_owned()
            },
            "libellé « {label} »"
        );
    }
}

// Spec 001, L5
#[test]
fn invalid_json_is_refused() {
    assert!(matches!(
        load_error(r#"{"card": "#),
        LoadError::InvalidJson(_)
    ));
    for root in [r#""Marche""#, "[]", "3"] {
        assert_eq!(load_error(root), LoadError::RootNotObject, "racine {root}");
    }
}

// Spec 001, L6
#[test]
fn duplicate_key_is_refused() {
    assert_eq!(
        load_error(r#"{"card": {"march": {"name": "Marche", "name": "Rage"}}}"#),
        LoadError::DuplicateKey {
            key: "card.march.name".to_owned()
        }
    );
}

// Spec 001, L7
#[test]
fn missing_language_file_is_a_typed_error() {
    let dir = fixtures().join("langues");
    let de = Language("de".to_owned());
    assert_eq!(
        Catalog::load(&dir, &de).map(|_| ()),
        Err(LoadError::FileNotFound(dir.join("de.json")))
    );
}

// Spec 001, L8
#[test]
fn non_utf8_file_is_refused() {
    let dir = fixtures().join("invalides");
    let latin1 = Language("latin1".to_owned());
    assert_eq!(
        Catalog::load(&dir, &latin1).map(|_| ()),
        Err(LoadError::NotUtf8(dir.join("latin1.json")))
    );
}
