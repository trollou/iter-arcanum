//! Spec 001, C11 à C13 : cohérence des vrais fichiers `assets/i18n/*.json` avec `fr.json`.

use std::path::{Path, PathBuf};

use arcanum_i18n::{Catalog, Language, available_languages};

fn assets_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("assets")
        .join("i18n")
}

fn load(language: &Language) -> Catalog {
    match Catalog::load(&assets_dir(), language) {
        Ok(catalog) => catalog,
        Err(error) => panic!("{} ne se charge pas : {error}", language.0),
    }
}

fn reference() -> Catalog {
    load(&Language("fr".to_owned()))
}

/// Toutes les langues sauf la référence.
fn others() -> Vec<Catalog> {
    let languages = match available_languages(&assets_dir()) {
        Ok(languages) => languages,
        Err(error) => panic!("assets/i18n illisible : {error}"),
    };
    languages
        .iter()
        .filter(|language| language.0 != "fr")
        .map(load)
        .collect()
}

// Spec 001, C11
#[test]
fn all_language_files_load() {
    let languages = match available_languages(&assets_dir()) {
        Ok(languages) => languages,
        Err(error) => panic!("assets/i18n illisible : {error}"),
    };
    assert!(
        languages.contains(&Language("fr".to_owned())),
        "fr.json, langue de référence, doit exister"
    );
    for language in &languages {
        load(language);
    }
}

// Spec 001, C12
#[test]
fn all_languages_have_the_same_keys() {
    let expected = reference().keys();
    let mut problems = Vec::new();
    for catalog in others() {
        let keys = catalog.keys();
        let extra: Vec<_> = keys.difference(&expected).collect();
        let missing: Vec<_> = expected.difference(&keys).collect();
        if !extra.is_empty() || !missing.is_empty() {
            problems.push(format!(
                "{} : en trop {extra:?}, en moins {missing:?}",
                catalog.language().0
            ));
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

// Spec 001, C13
#[test]
fn all_languages_have_the_same_parameters() {
    let reference = reference();
    let mut problems = Vec::new();
    for catalog in others() {
        for key in reference.keys() {
            let expected = reference.parameters(&key).unwrap_or_default();
            // Une clé absente relève de C12 : on ne compare que les clés présentes.
            let Some(actual) = catalog.parameters(&key) else {
                continue;
            };
            if actual != expected {
                problems.push(format!(
                    "{} / {key} : {actual:?} au lieu de {expected:?}",
                    catalog.language().0
                ));
            }
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}
