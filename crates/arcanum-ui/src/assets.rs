use std::path::{Path, PathBuf};

use arcanum_i18n::{Catalog, Language, LoadError};
use thiserror::Error;

use crate::language::FALLBACK_LANGUAGE;

/// Variable d'environnement qui désigne un dossier des ressources.
pub const ASSETS_ENV_VAR: &str = "ARCANUM_ASSETS_DIR";

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AssetsError {
    #[error("aucun dossier des ressources trouvé parmi {tried:?}")]
    NotFound { tried: Vec<PathBuf> },
    #[error(transparent)]
    Catalog(#[from] LoadError),
}

fn french() -> Language {
    Language("fr".to_owned())
}

fn i18n_dir(assets: &Path) -> PathBuf {
    assets.join("i18n")
}

/// Dossier `assets` du dépôt, connu à la compilation (développement).
pub fn repository_assets_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("assets")
}

/// Emplacements candidats, par ordre de priorité : variable d'environnement, à côté de l'exécutable,
/// ressources du paquet `.app` (`Contents/MacOS/../Resources/assets`, spec #33), dépôt.
/// Le candidat `.app` est proposé partout, sans `#[cfg]` : ailleurs il n'existe pas (ADR 0004).
pub fn assets_candidates(env: Option<PathBuf>, exe_dir: Option<PathBuf>) -> Vec<PathBuf> {
    let beside_exe = exe_dir.into_iter().flat_map(|dir| {
        [
            dir.join("assets"),
            dir.join("..").join("Resources").join("assets"),
        ]
    });
    env.into_iter()
        .chain(beside_exe)
        .chain([repository_assets_dir()])
        .collect()
}

/// Premier candidat qui contient `i18n/fr.json`.
pub fn find_assets_dir(candidates: &[PathBuf]) -> Result<PathBuf, AssetsError> {
    candidates
        .iter()
        .find(|dir| i18n_dir(dir).join("fr.json").is_file())
        .cloned()
        .ok_or_else(|| AssetsError::NotFound {
            tried: candidates.to_vec(),
        })
}

/// Catalogue d'une langue, pris dans le premier dossier des ressources trouvé.
pub fn load_catalog(candidates: &[PathBuf], language: &Language) -> Result<Catalog, AssetsError> {
    let assets = find_assets_dir(candidates)?;
    Ok(Catalog::load(&i18n_dir(&assets), language)?)
}

/// Catalogue d'une langue, sinon l'anglais, sinon un catalogue vide (clés visibles).
/// Le dossier retenu et les échecs sont écrits dans les logs (spec 003 C17).
pub fn catalog_or_empty(candidates: &[PathBuf], language: &Language) -> Catalog {
    let assets = match find_assets_dir(candidates) {
        Ok(assets) => assets,
        Err(_) => {
            // Seule erreur possible : aucun candidat. Chemins affichés tels quels : le format Debug doublerait les `\` sous Windows.
            let tried: Vec<String> = candidates
                .iter()
                .map(|dir| dir.display().to_string())
                .collect();
            log::error!(
                "aucun dossier des ressources trouvé parmi : {}",
                tried.join(", ")
            );
            return empty_catalog();
        }
    };
    log::info!("dossier des ressources : {}", assets.display());
    let dir = i18n_dir(&assets);
    Catalog::load(&dir, language).unwrap_or_else(|error| {
        log::error!("chargement de {}.json impossible : {error}", language.0);
        // Spec 076 L1 : repli en anglais, sauf si c'est lui qui vient d'échouer.
        let fallback = Language(FALLBACK_LANGUAGE.to_owned());
        if *language == fallback {
            return empty_catalog();
        }
        Catalog::load(&dir, &fallback).unwrap_or_else(|error| {
            log::error!("chargement de {}.json impossible : {error}", fallback.0);
            empty_catalog()
        })
    })
}

fn empty_catalog() -> Catalog {
    // Invariant : `{}` est un fichier de langue valide (racine objet, aucune feuille).
    Catalog::from_json_str(french(), "{}").expect("catalogue vide valide")
}
