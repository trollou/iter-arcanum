//! Préférences au lancement (spec 080) : dossier de configuration, lecture du fichier.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use arcanum_audio::SoundCategory;
use arcanum_i18n::{Catalog, Language, available_languages};
use arcanum_persistance::preferences::{Preferences, PreferencesError, load};

use crate::language::FALLBACK_LANGUAGE;
use crate::{Button, Command, Shortcut, find_assets_dir, shortcut_label};

/// Clé du nom d'une langue dans son propre catalogue (spec 001).
const NAME_KEY: &str = "language.name";

/// Dossier de configuration imposé, prioritaire sur tout le reste.
pub const CONFIG_DIR_ENV_VAR: &str = "ARCANUM_CONFIG_DIR";

/// Dossier de configuration dans le dépôt, quand le jeu est lancé par cargo.
const CONFIG_DIR: &str = "config";

/// `ARCANUM_CONFIG_DIR`, puis `config/` du dépôt si le jeu est lancé par cargo, puis le dossier de
/// configuration de l'utilisateur. Pure : l'appelant lit l'environnement (même ordre que
/// `choose_saves_dir`, spec 024).
pub fn choose_config_dir(
    config_dir_var: Option<OsString>,
    cargo_manifest_dir: Option<OsString>,
    user_config: Option<PathBuf>,
) -> Option<PathBuf> {
    if let Some(dir) = config_dir_var.filter(|value| !value.is_empty()) {
        return Some(dir.into());
    }
    // `cargo run` définit CARGO_MANIFEST_DIR = <racine>/crates/arcanum-ui.
    let repository = cargo_manifest_dir
        .map(PathBuf::from)
        .and_then(|manifest| manifest.parent()?.parent().map(Path::to_path_buf));
    repository.map(|base| base.join(CONFIG_DIR)).or(user_config)
}

/// Dossier de configuration de l'application, `directories` choisit selon le système (ADR 0004).
pub fn user_config_dir() -> Option<PathBuf> {
    directories::ProjectDirs::from("", "", "iter-arcanum")
        .map(|dirs| dirs.config_dir().to_path_buf())
}

/// Préférences lues au lancement : les défauts si le fichier manque ou ne se lit pas (C11).
/// Le fichier illisible est laissé tel quel jusqu'au prochain changement.
pub fn launch_preferences(path: Option<&Path>) -> Preferences {
    let Some(path) = path else {
        log::info!("aucun dossier de configuration : préférences par défaut");
        return Preferences::default();
    };
    match load(path) {
        Ok(preferences) => {
            log::info!("préférences lues : {}", path.display());
            preferences
        }
        Err(PreferencesError::Missing) => {
            log::info!(
                "aucun fichier des préférences ({}) : préférences par défaut",
                path.display()
            );
            Preferences::default()
        }
        Err(error) => {
            log::error!(
                "préférences illisibles ({}) : {error} ; préférences par défaut",
                path.display()
            );
            Preferences::default()
        }
    }
}

/// Langues proposées par la fenêtre des préférences, lues au lancement (C16).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LanguageChoices {
    /// Les langues qui ont un fichier, dans l'ordre des codes, chacune avec son propre nom
    /// (`language.name` de son catalogue).
    pub available: Vec<(Language, String)>,
    /// Ce que donne « Langue du système » : la langue du système si elle a un fichier, sinon
    /// l'anglais.
    pub system: Language,
    /// Le nom de cette langue, lu dans son catalogue.
    pub system_name: String,
}

impl Default for LanguageChoices {
    /// Aucune langue disponible : la langue du système donne l'anglais (L1).
    fn default() -> Self {
        Self {
            available: Vec::new(),
            system: Language(FALLBACK_LANGUAGE.to_owned()),
            system_name: String::new(),
        }
    }
}

impl LanguageChoices {
    /// Lit les langues du premier dossier des ressources trouvé.
    pub fn load(candidates: &[PathBuf], system_language: Option<String>) -> Self {
        let Ok(assets) = find_assets_dir(candidates) else {
            return Self::default();
        };
        let dir = assets.join("i18n");
        let available: Vec<(Language, String)> = available_languages(&dir)
            .unwrap_or_default()
            .into_iter()
            .map(|language| {
                // Une langue qui ne se charge pas garde son code pour nom.
                let name = Catalog::load(&dir, &language)
                    .map_or_else(|_| language.0.clone(), |catalog| catalog.text(NAME_KEY));
                (language, name)
            })
            .collect();
        let system = system_language
            .filter(|code| available.iter().any(|(language, _)| language.0 == *code))
            .map_or_else(|| Language(FALLBACK_LANGUAGE.to_owned()), Language);
        let system_name = available
            .iter()
            .find(|(language, _)| *language == system)
            .map_or_else(|| system.0.clone(), |(_, name)| name.clone());
        Self {
            available,
            system,
            system_name,
        }
    }

    /// La langue d'un choix de la liste : `None` = langue du système.
    pub fn language_of(&self, choice: Option<usize>) -> Option<Language> {
        match choice {
            None => Some(self.system.clone()),
            Some(index) => self
                .available
                .get(index)
                .map(|(language, _)| language.clone()),
        }
    }
}

/// Une ligne de la liste des langues.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LanguageOption {
    /// « Langue du système (Français) », « English ».
    pub label: String,
    /// Le choix des préférences.
    pub checked: bool,
    pub command: Command,
}

/// Un curseur de volume de la section « Son » (spec 081, C23).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VolumeSlider {
    pub category: SoundCategory,
    /// « Volume général », « Musique »…
    pub label: String,
    /// De 0 à 100.
    pub value: u8,
}

/// Les quatre curseurs, dans l'ordre de la fenêtre.
fn volume_sliders(preferences: &Preferences, catalog: &Catalog) -> Vec<VolumeSlider> {
    let volumes = &preferences.volumes;
    [
        (
            SoundCategory::Master,
            "preferences.volume_master",
            volumes.master,
        ),
        (
            SoundCategory::Music,
            "preferences.volume_music",
            volumes.music,
        ),
        (
            SoundCategory::Ambience,
            "preferences.volume_ambience",
            volumes.ambience,
        ),
        (
            SoundCategory::Effects,
            "preferences.volume_effects",
            volumes.effects,
        ),
    ]
    .into_iter()
    .map(|(category, key, value)| VolumeSlider {
        category,
        label: catalog.text(key),
        value,
    })
    .collect()
}

/// La valeur d'un curseur de volume, « 80 % » (`preferences.volume_value`).
pub fn volume_value_label(percent: u8, catalog: &Catalog) -> String {
    catalog.format(
        "preferences.volume_value",
        &[("percent", &percent.to_string())],
    )
}

/// La fenêtre des préférences, déjà traduite (spec 080, C15).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreferencesWindow {
    /// « Préférences ».
    pub title: String,
    /// « Langue ».
    pub language_label: String,
    /// La langue des préférences : `None` = langue du système.
    pub language: Option<Language>,
    pub languages: Vec<LanguageOption>,
    /// « Plein écran au lancement », sa case et sa note.
    pub fullscreen_label: String,
    pub fullscreen_at_launch: bool,
    pub fullscreen_hint: String,
    /// Spec #5 : le titre « Affichage » et la case « Animations ».
    pub display_label: String,
    pub animations_label: String,
    pub animations: bool,
    /// Spec 081 : « Son » et ses quatre curseurs (C23).
    pub sound_label: String,
    pub volumes: Vec<VolumeSlider>,
    /// « Aucune sortie audio : le jeu reste muet. », sans carte son (C23, L1).
    pub no_audio: Option<String>,
    /// « Préférences non enregistrées (voir les logs). », après un échec (C21).
    pub not_saved: Option<String>,
    /// « Fermer » (Échap).
    pub close: Button,
}

/// La fenêtre des préférences, traduite dans le catalogue affiché.
pub fn preferences_window(
    preferences: &Preferences,
    languages: &LanguageChoices,
    not_saved: bool,
    audio_output: bool,
    catalog: &Catalog,
) -> PreferencesWindow {
    let chosen = preferences.language.as_deref();
    let system = LanguageOption {
        label: catalog.format(
            "preferences.system_language",
            &[("language", &languages.system_name)],
        ),
        checked: chosen.is_none(),
        command: Command::SetLanguage(None),
    };
    let others = languages
        .available
        .iter()
        .enumerate()
        .map(|(index, (language, name))| LanguageOption {
            label: name.clone(),
            checked: chosen == Some(language.0.as_str()),
            command: Command::SetLanguage(Some(index)),
        });
    PreferencesWindow {
        title: catalog.text("preferences.title"),
        language_label: catalog.text("preferences.language"),
        language: preferences.language.clone().map(Language),
        languages: std::iter::once(system).chain(others).collect(),
        fullscreen_label: catalog.text("preferences.fullscreen_at_launch"),
        fullscreen_at_launch: preferences.fullscreen_at_launch,
        fullscreen_hint: catalog.text("preferences.fullscreen_hint"),
        display_label: String::new(),
        animations_label: String::new(),
        animations: false,
        sound_label: catalog.text("preferences.sound"),
        volumes: volume_sliders(preferences, catalog),
        no_audio: (!audio_output).then(|| catalog.text("preferences.no_audio")),
        not_saved: not_saved.then(|| catalog.text("preferences.not_saved")),
        close: Button {
            label: catalog.text("preferences.close"),
            command: Command::ClosePreferences,
            shortcut: shortcut_label(Shortcut::Cancel, catalog),
        },
    }
}
