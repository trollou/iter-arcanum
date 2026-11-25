//! Choix de la langue au lancement (spec 076) : paramètre `--lang`, sinon langue du système, sinon l'anglais.
//! Les messages des logs restent en français quelle que soit la langue retenue (spec 003, L3).

use std::path::{Path, PathBuf};

use arcanum_i18n::{Catalog, Language, available_languages};

use crate::assets::{catalog_or_empty, find_assets_dir};

/// Langue affichée quand ni la langue demandée ni celle du système n'ont de fichier.
pub const FALLBACK_LANGUAGE: &str = "en";

/// Variables lues pour la langue du système hors Windows, par ordre de priorité (ordre POSIX des messages).
pub const UNIX_LOCALE_VARIABLES: [&str; 3] = ["LC_ALL", "LC_MESSAGES", "LANG"];

const LANG_ARGUMENT: &str = "--lang";

/// Langue demandée par les arguments du lancement (sans le nom du programme) : valeur brute du dernier `--lang`.
/// Les anomalies (valeur manquante, `--lang` répété, argument inconnu) sont écrites dans les logs.
pub fn requested_language(args: &[String]) -> Option<String> {
    let mut requested = None;
    let mut seen = false;
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        let value = if arg == LANG_ARGUMENT {
            match rest.next() {
                Some(value) => value.clone(),
                None => {
                    log::error!("paramètre {LANG_ARGUMENT} sans valeur, ignoré");
                    continue;
                }
            }
        } else if let Some(value) = arg.strip_prefix("--lang=") {
            value.to_owned()
        } else {
            log::warn!("argument inconnu ignoré : {arg}");
            continue;
        };
        if seen {
            log::warn!(
                "paramètre {LANG_ARGUMENT} donné plusieurs fois : le dernier l'emporte ({value})"
            );
        }
        seen = true;
        requested = Some(value);
    }
    requested
}

/// Code de langue d'une valeur de locale : les lettres avant le premier `_`, `-`, `.` ou `@`, en minuscules.
pub fn locale_to_code(value: &str) -> Option<String> {
    let code = value
        .split(['_', '-', '.', '@'])
        .next()
        .unwrap_or_default()
        .trim()
        .to_lowercase();
    (!code.is_empty()).then_some(code)
}

/// Langue du système hors Windows : première variable non vide de [`UNIX_LOCALE_VARIABLES`].
pub fn unix_system_language(lookup: impl Fn(&str) -> Option<String>) -> Option<String> {
    UNIX_LOCALE_VARIABLES
        .iter()
        .filter_map(|name| lookup(name))
        .find(|value| !value.is_empty())
        .and_then(|value| locale_to_code(&value))
}

/// Code de la langue principale d'un identifiant de langue Windows (10 bits de poids faible).
/// Ajouter une langue au jeu ajoute une ligne ici.
pub fn windows_language_id_to_code(id: u16) -> Option<&'static str> {
    match id & 0x03FF {
        0x0C => Some("fr"),
        0x09 => Some("en"),
        _ => None,
    }
}

/// Langue du système sous macOS (spec #7) : la première langue préférée de l'utilisateur, sinon
/// les variables de locale. Pure : l'appelant lit CoreFoundation et l'environnement.
pub fn apple_system_language(
    first_preferred: Option<&str>,
    lookup: impl Fn(&str) -> Option<String>,
) -> Option<String> {
    let _ = (first_preferred, lookup);
    todo!()
}

/// Langue du système, lue au lancement.
pub fn system_language() -> Option<String> {
    #[cfg(windows)]
    {
        windows_system_language()
    }
    #[cfg(target_os = "macos")]
    {
        macos_system_language(|name| std::env::var(name).ok())
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        unix_system_language(|name| std::env::var(name).ok())
    }
}

/// Première langue préférée de l'utilisateur macOS, lue dans CoreFoundation (spec #7, Q1).
#[cfg(target_os = "macos")]
pub fn macos_preferred_language() -> Option<String> {
    todo!()
}

/// Langue du système sous macOS : langues préférées, puis `lookup` pour les variables de locale.
#[cfg(target_os = "macos")]
pub fn macos_system_language(lookup: impl Fn(&str) -> Option<String>) -> Option<String> {
    let _ = lookup;
    todo!()
}

/// Lecture de la langue de l'interface de Windows (spec 076, Q1 et Q2).
#[cfg(windows)]
pub fn windows_system_language() -> Option<String> {
    windows_language_id_to_code(windows::user_default_ui_language()).map(str::to_owned)
}

#[cfg(windows)]
mod windows {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetUserDefaultUILanguage() -> u16;
    }

    pub fn user_default_ui_language() -> u16 {
        // SAFETY : aucun argument ni pointeur, un entier en retour ; kernel32 est toujours chargé.
        unsafe { GetUserDefaultUILanguage() }
    }
}

/// Catalogue de la langue retenue pour ce lancement : arguments, langue des préférences (spec 080),
/// langue du système, anglais.
pub fn launch_catalog(
    args: &[String],
    preferred_language: Option<String>,
    system_language: Option<String>,
    candidates: &[PathBuf],
) -> Catalog {
    let requested = requested_language(args);
    let available = find_assets_dir(candidates)
        .ok()
        .map(|assets| languages_in(&assets))
        .unwrap_or_default();
    let language = choose_language(
        requested.as_deref(),
        preferred_language.as_deref(),
        system_language.as_deref(),
        &available,
    );
    catalog_or_empty(candidates, &language)
}

fn languages_in(assets: &Path) -> Vec<Language> {
    available_languages(&assets.join("i18n")).unwrap_or_else(|error| {
        log::error!("lecture des langues disponibles impossible : {error}");
        Vec::new()
    })
}

fn choose_language(
    requested: Option<&str>,
    preferred: Option<&str>,
    system: Option<&str>,
    available: &[Language],
) -> Language {
    let has_file = |code: &str| available.iter().any(|language| language.0 == code);
    if let Some(requested) = requested {
        match locale_to_code(requested).filter(|code| has_file(code)) {
            Some(code) => {
                log::info!("langue : {code} (paramètre {LANG_ARGUMENT})");
                return Language(code);
            }
            None => {
                let names: Vec<&str> = available
                    .iter()
                    .map(|language| language.0.as_str())
                    .collect();
                log::error!(
                    "langue demandée introuvable : {requested} (disponibles : {}) ; repli en anglais",
                    names.join(", ")
                );
                return fallback();
            }
        }
    }
    // Spec 080 : la langue des préférences, sautée si elle n'a pas de fichier (C9).
    if let Some(code) = preferred {
        if has_file(code) {
            log::info!("langue : {code} (préférences)");
            return Language(code.to_owned());
        }
        log::warn!("langue des préférences sans fichier de libellés : {code} ; langue du système");
    }
    match system {
        Some(code) if has_file(code) => {
            log::info!("langue : {code} (langue du système)");
            Language(code.to_owned())
        }
        Some(code) => {
            log::info!("langue du système sans fichier de libellés : {code}");
            fallback()
        }
        None => {
            log::info!("langue du système introuvable");
            fallback()
        }
    }
}

fn fallback() -> Language {
    log::info!("langue : {FALLBACK_LANGUAGE} (repli)");
    Language(FALLBACK_LANGUAGE.to_owned())
}
