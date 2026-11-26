//! Lance la fenêtre du jeu (spec 002).

// Spec #33 : pas de console sous Windows pour un exécutable release ; les logs vont dans un fichier (spec 003).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use arcanum_audio::{AudioPlayer, KiraPlayer, SOUNDS_DIR, SilentPlayer, SoundBank};
use arcanum_persistance::preferences::PREFERENCES_FILE;
use arcanum_ui::{
    ANIMATION_SCALE_ENV_VAR, ASSETS_ENV_VAR, CONFIG_DIR_ENV_VAR, ClientSession, IterArcanumApp,
    LanguageChoices, SAVE_DIR_ENV_VAR, animation_scale, assets_candidates, choose_config_dir,
    choose_saves_dir, find_assets_dir, init_logging, launch_catalog, launch_preferences,
    system_language, user_config_dir, user_data_dir, viewport, window_icon,
};

fn main() -> eframe::Result {
    // Avant tout le reste, pour que le démarrage soit écrit dans les logs (spec 003).
    init_logging();
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.to_path_buf()));
    let env = std::env::var_os(ASSETS_ENV_VAR).map(Into::into);
    let candidates = assets_candidates(env, exe_dir);
    // Spec 080 : les préférences, dans le dossier de configuration (même ordre que les sauvegardes).
    let config_dir = choose_config_dir(
        std::env::var_os(CONFIG_DIR_ENV_VAR),
        std::env::var_os("CARGO_MANIFEST_DIR"),
        user_config_dir(),
    );
    log::info!("dossier de configuration : {config_dir:?}");
    let preferences_path = config_dir.map(|dir| dir.join(PREFERENCES_FILE));
    let preferences = launch_preferences(preferences_path.as_deref());
    // Spec 076 et 080 : `--lang`, sinon la langue des préférences, sinon celle du système, sinon
    // l'anglais.
    let args: Vec<String> = std::env::args().skip(1).collect();
    let system = system_language();
    let catalog = launch_catalog(
        &args,
        preferences.language.clone(),
        system.clone(),
        &candidates,
    );
    let languages = LanguageChoices::load(&candidates, system);
    // Sans dossier des ressources, le plateau de jeu s'affiche avec ses couleurs de repli.
    let images_dir = find_assets_dir(&candidates)
        .ok()
        .map(|assets| assets.join("images"));
    let title = catalog.text("app.title");
    // Spec 081 : les sons du dossier des ressources ; sans carte son, le jeu reste muet (L1, L2).
    let bank = find_assets_dir(&candidates)
        .map(|assets| SoundBank::scan(&assets.join(SOUNDS_DIR)))
        .unwrap_or_else(|_| {
            log::info!("aucun dossier des ressources : le jeu reste muet");
            SoundBank::default()
        });
    let (audio, audio_output): (Box<dyn AudioPlayer>, bool) = match KiraPlayer::new(bank) {
        Ok(player) => (Box::new(player), true),
        Err(error) => {
            log::warn!("{error} : le jeu reste muet");
            (Box::new(SilentPlayer), false)
        }
    };
    let volumes = preferences.volumes;
    // Issue #50 : ralenti des animations du plateau, pour les captures de la vérification à l'œil.
    let scale = animation_scale(std::env::var(ANIMATION_SCALE_ENV_VAR).ok().as_deref());

    // Dossier des sauvegardes (spec 024), dans le même ordre que celui des logs.
    let saves_dir = choose_saves_dir(
        std::env::var_os(SAVE_DIR_ENV_VAR),
        std::env::var_os("CARGO_MANIFEST_DIR"),
        user_data_dir(),
    );
    log::info!("dossier des sauvegardes : {saves_dir:?}");
    // Spec 065 : l'écran de nouvelle partie ; la graine est tirée au clic sur « Commencer ».
    let fullscreen_at_launch = preferences.fullscreen_at_launch;
    let client = ClientSession::launch(saves_dir)
        .with_preferences(preferences, preferences_path, languages)
        .with_audio_output(audio_output);

    // Sous Linux, un `.desktop` donne à la barre des tâches l'icône du thème, nette (spec 004, issue privée 315).
    #[cfg(target_os = "linux")]
    install_desktop_file();

    let options = eframe::NativeOptions {
        // Icône de la barre des tâches, Windows et Linux (spec 004).
        viewport: viewport(&title, window_icon(), fullscreen_at_launch),
        ..Default::default()
    };
    eframe::run_native(
        &title,
        options,
        Box::new(|creation| {
            // Chargeurs d'images d'egui_extras : fichiers et SVG (spec 011).
            egui_extras::install_image_loaders(&creation.egui_ctx);
            // Thème sombre, même si le système est en clair (issue #66).
            arcanum_ui::apply_theme(&creation.egui_ctx);
            Ok(Box::new(
                IterArcanumApp::new(client, catalog, images_dir)
                    .with_assets_candidates(candidates)
                    .with_audio(audio, volumes)
                    .with_animation_scale(scale),
            ))
        }),
    )
}

/// Écrit le `.desktop` et l'icône dans le dossier des données ; un échec est seulement écrit dans les logs.
#[cfg(target_os = "linux")]
fn install_desktop_file() {
    let data_home = directories::BaseDirs::new().map(|dirs| dirs.data_dir().to_path_buf());
    match (data_home, std::env::current_exe()) {
        (Some(data_home), Ok(exe)) => match arcanum_ui::install_desktop_entry(&data_home, &exe) {
            Ok(outcome) => log::info!("fichier .desktop : {outcome:?}"),
            Err(error) => log::error!("fichier .desktop : {error}"),
        },
        _ => log::error!("fichier .desktop : dossier des données ou exécutable introuvable"),
    }
}
