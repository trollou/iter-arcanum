//! Scénarios de la spec 080 (préférences) côté client : dossier de configuration, lancement,
//! fenêtre des préférences. Aucune règle du livret ici : chaque test cite le cas de la spec.
//! Arguments, variables d'environnement et dossiers passent en paramètres : rien ne dépend de la
//! machine.

mod support;

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use arcanum_core::{GameConfig, Scenario};
use arcanum_i18n::{Catalog, Language};
use arcanum_persistance::preferences::{PREFERENCES_FILE, Preferences};
use arcanum_ui::{
    ClientSession, Command, GameSession, KEY_BINDINGS, KeyModifier, LanguageChoices, MenuBar,
    MenuItem, PreferencesWindow, Shortcut, WindowRequest, catalog_or_empty, choose_config_dir,
    launch_catalog, launch_preferences, repository_assets_dir, shortcut_label, viewport,
    window_icon,
};
use log::Level;
use support::first_reconnaissance_config;
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

fn name(catalog: &Catalog) -> String {
    catalog.text("language.name")
}

// --- Le dossier de configuration ---

#[test]
fn config_dir_choice() {
    // Spec 080 C6 : même forme que `saves_dir_choice` (spec 024).
    let user = PathBuf::from("configuration");
    let repo = PathBuf::from("depot");
    let manifest = OsString::from(repo.join("crates").join("arcanum-ui"));

    assert_eq!(
        choose_config_dir(
            Some(OsString::from("ailleurs")),
            Some(manifest.clone()),
            Some(user.clone())
        ),
        Some(PathBuf::from("ailleurs"))
    );
    assert_eq!(
        choose_config_dir(Some(OsString::new()), Some(manifest), Some(user.clone())),
        Some(repo.join("config"))
    );
    assert_eq!(
        choose_config_dir(None, None, Some(user.clone())),
        Some(user)
    );
    assert_eq!(choose_config_dir(None, None, None), None);
}

// --- Le lancement ---

#[test]
fn preferred_language_beats_system_language() {
    // Spec 080 C7
    let (catalog, lines) =
        logs_of(|| launch_catalog(&[], Some("en".into()), Some("fr".into()), &repository()));
    assert_eq!(name(&catalog), "English");
    assert!(
        has_line(&lines, Level::Info, &["langue : en (préférences)"]),
        "{lines:?}"
    );
}

#[test]
fn lang_argument_beats_preferred_language() {
    // Spec 080 C8
    let (catalog, lines) = logs_of(|| {
        launch_catalog(
            &args(&["--lang", "fr"]),
            Some("en".into()),
            Some("en".into()),
            &repository(),
        )
    });
    assert_eq!(name(&catalog), "Français");
    assert!(
        has_line(&lines, Level::Info, &["langue : fr (paramètre --lang)"]),
        "{lines:?}"
    );
}

#[test]
fn unavailable_preferred_language_falls_back_to_system() {
    // Spec 080 C9
    let (catalog, lines) =
        logs_of(|| launch_catalog(&[], Some("de".into()), Some("fr".into()), &repository()));
    assert_eq!(name(&catalog), "Français");
    assert!(has_line(&lines, Level::Warn, &["de"]), "{lines:?}");
    assert!(
        has_line(&lines, Level::Info, &["langue : fr (langue du système)"]),
        "{lines:?}"
    );
}

#[test]
fn system_choice_keeps_spec_076_behaviour() {
    // Spec 080 C10 : sans langue des préférences, la spec 076 (C8 à C12) ne change pas.
    let (catalog, lines) = logs_of(|| launch_catalog(&[], None, Some("fr".into()), &repository()));
    assert_eq!(name(&catalog), "Français");
    assert!(
        has_line(&lines, Level::Info, &["langue : fr (langue du système)"]),
        "{lines:?}"
    );
    assert!(!lines.iter().any(|(_, line)| line.contains("préférences")));
}

#[test]
fn missing_or_unreadable_preferences_give_defaults() {
    // Spec 080 C11
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let path = dir.path().join(PREFERENCES_FILE);

    let (preferences, lines) = logs_of(|| launch_preferences(Some(&path)));
    assert_eq!(preferences, Preferences::default());
    assert!(has_line(&lines, Level::Info, &["préférences"]), "{lines:?}");

    fs::write(&path, "{ pas du JSON").expect("fichier");
    let (preferences, lines) = logs_of(|| launch_preferences(Some(&path)));
    assert_eq!(preferences, Preferences::default());
    let file = path.display().to_string();
    assert!(has_line(&lines, Level::Error, &[&file]), "{lines:?}");
    assert_eq!(
        fs::read_to_string(&path).expect("fichier"),
        "{ pas du JSON",
        "le fichier illisible n'est pas réécrit au lancement"
    );
}

#[test]
fn preferences_are_read_at_launch() {
    // Spec 080 C11 (cas nominal) : le fichier lu donne ses préférences.
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let path = dir.path().join(PREFERENCES_FILE);
    fs::write(
        &path,
        r#"{"version": 1, "language": "en", "fullscreen_at_launch": true}"#,
    )
    .expect("fichier");
    assert_eq!(
        launch_preferences(Some(&path)),
        Preferences {
            language: Some("en".to_owned()),
            fullscreen_at_launch: true,
            ..Preferences::default()
        }
    );
}

#[test]
fn viewport_starts_fullscreen_when_preferred() {
    // Spec 080 C12
    assert_eq!(
        viewport("Iter Arcanum", window_icon(), true).fullscreen,
        Some(true)
    );
    assert_ne!(
        viewport("Iter Arcanum", window_icon(), false).fullscreen,
        Some(true)
    );
}

// --- La fenêtre : outils ---

fn french() -> Catalog {
    catalog_or_empty(&repository(), &Language("fr".into()))
}

fn english() -> Catalog {
    catalog_or_empty(&repository(), &Language("en".into()))
}

fn language(code: &str) -> Language {
    Language(code.to_owned())
}

/// Les langues du dépôt, sur un système français.
fn choices() -> LanguageChoices {
    LanguageChoices::load(&repository(), Some("fr".into()))
}

/// Bac à sable au début du premier tour.
fn sandbox() -> GameSession {
    let mut session = GameSession::new(GameConfig {
        scenario: Scenario::sandbox(),
        ..first_reconnaissance_config(3)
    });
    session.run_frame();
    session
}

/// L'écran de lancement, préférences par défaut écrites dans `dir`.
fn launch(dir: &Path) -> ClientSession {
    ClientSession::launch(None).with_preferences(
        Preferences::default(),
        Some(dir.join(PREFERENCES_FILE)),
        choices(),
    )
}

/// Une partie en cours, préférences par défaut écrites dans `dir`.
fn in_game(dir: &Path) -> ClientSession {
    launch(dir).with_game(sandbox())
}

fn window(client: &ClientSession, catalog: &Catalog) -> PreferencesWindow {
    client
        .preferences_window(catalog)
        .expect("fenêtre des préférences ouverte")
}

fn labels(window: &PreferencesWindow) -> Vec<(&str, bool)> {
    window
        .languages
        .iter()
        .map(|option| (option.label.as_str(), option.checked))
        .collect()
}

/// La commande de la ligne de ce libellé.
fn choose(client: &mut ClientSession, catalog: &Catalog, label: &str) {
    let command = window(client, catalog)
        .languages
        .iter()
        .find(|option| option.label == label)
        .unwrap_or_else(|| panic!("{label} absent de la liste"))
        .command;
    client.command(command);
}

fn file_text(dir: &Path) -> String {
    fs::read_to_string(dir.join(PREFERENCES_FILE)).expect("fichier des préférences écrit")
}

fn menu_lines(bar: &MenuBar) -> Vec<Option<(String, Option<String>, bool)>> {
    bar.items
        .iter()
        .map(|item| match item {
            MenuItem::Entry(entry) => {
                Some((entry.label.clone(), entry.shortcut.clone(), entry.enabled))
            }
            MenuItem::Separator => None,
        })
        .collect()
}

fn preferences_entry(bar: &MenuBar) -> Option<(String, Option<String>, bool)> {
    bar.items.iter().find_map(|item| match item {
        MenuItem::Entry(entry) if entry.command == Command::OpenPreferences => {
            Some((entry.label.clone(), entry.shortcut.clone(), entry.enabled))
        }
        _ => None,
    })
}

// --- La fenêtre ---

#[test]
fn menu_lists_preferences_before_quit() {
    // Spec 080 C13
    let bar = sandbox().menu_bar(&french());
    let lines = menu_lines(&bar);
    let tail: Vec<_> = lines[lines.len() - 5..].to_vec();
    assert_eq!(
        tail,
        [
            Some(("Plein écran".to_owned(), Some("F11".to_owned()), true)),
            None,
            Some(("Préférences…".to_owned(), Some("Ctrl+,".to_owned()), true)),
            None,
            Some(("Quitter".to_owned(), Some("Ctrl+Q".to_owned()), true)),
        ]
    );
}

#[test]
fn preferences_entry_is_always_enabled() {
    // Spec 080 C14 : en mode choix, derrière la fenêtre de nouvelle partie et derrière celle de
    // chargement, comme Plein écran.
    let catalog = french();
    let mut choosing = sandbox();
    let card = choosing.view().hand[0];
    choosing.command(Command::ChooseDiscard { card, option: 0 });
    assert!(choosing.is_choosing_discard());
    assert!(
        preferences_entry(&choosing.menu_bar(&catalog))
            .expect("élément")
            .2
    );

    let dir = tempfile::tempdir().expect("dossier temporaire");
    let mut client = in_game(dir.path());
    client.command(Command::OpenNewGame);
    let bar = client.menu_bar(&catalog).expect("barre");
    assert!(preferences_entry(&bar).expect("élément").2);

    let mut loading = sandbox();
    loading.command(Command::OpenLoad);
    assert!(
        preferences_entry(&loading.menu_bar(&catalog))
            .expect("élément")
            .2
    );
}

#[test]
fn open_preferences_toggles_window() {
    // Spec 080 C15
    let catalog = french();
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let mut client = in_game(dir.path());
    assert!(client.preferences_window(&catalog).is_none());

    client.command(Command::OpenPreferences);
    let shown = window(&client, &catalog);
    assert_eq!(shown.title, "Préférences");
    assert_eq!(shown.language, None);
    assert!(!shown.fullscreen_at_launch);
    assert_eq!(shown.fullscreen_label, "Plein écran au lancement");
    assert_eq!(
        shown.fullscreen_hint,
        "F11 bascule la fenêtre sans changer ce réglage."
    );
    assert_eq!(shown.not_saved, None);
    assert_eq!(shown.close.label, "Fermer");
    assert_eq!(shown.close.command, Command::ClosePreferences);

    client.command(Command::OpenPreferences);
    assert!(client.preferences_window(&catalog).is_none());

    client.command(Command::OpenPreferences);
    client.command(Command::ClosePreferences);
    assert!(client.preferences_window(&catalog).is_none());
}

#[test]
fn language_list_names_each_language_in_itself() {
    // Spec 080 C16
    let catalog = french();
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let mut client = launch(dir.path());
    client.command(Command::OpenPreferences);
    let shown = window(&client, &catalog);
    assert_eq!(shown.language_label, "Langue");
    assert_eq!(
        labels(&shown),
        [
            ("Langue du système (Français)", true),
            ("English", false),
            ("Français", false),
        ]
    );

    // Choix des préférences coché ; le nom entre parenthèses vient de la langue du système.
    let mut client = ClientSession::launch(None).with_preferences(
        Preferences {
            language: Some("en".into()),
            fullscreen_at_launch: false,
            ..Preferences::default()
        },
        None,
        choices(),
    );
    client.command(Command::OpenPreferences);
    let shown = window(&client, &english());
    assert_eq!(shown.language, Some(language("en")));
    assert_eq!(
        labels(&shown),
        [
            ("System language (Français)", false),
            ("English", true),
            ("Français", false),
        ]
    );

    // Une langue du système sans fichier donne l'anglais.
    let mut client = ClientSession::launch(None).with_preferences(
        Preferences::default(),
        None,
        LanguageChoices::load(&repository(), Some("de".into())),
    );
    client.command(Command::OpenPreferences);
    assert_eq!(
        window(&client, &catalog).languages[0].label,
        "Langue du système (English)"
    );
}

#[test]
fn choosing_language_switches_catalog_and_saves() {
    // Spec 080 C17
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let mut client = in_game(dir.path());
    client.command(Command::OpenPreferences);
    choose(&mut client, &french(), "English");

    assert_eq!(
        client.take_window_requests(),
        [WindowRequest::Language(language("en"))]
    );
    // L'application recharge le catalogue de la langue demandée.
    let catalog = catalog_or_empty(&repository(), &language("en"));
    assert_eq!(window(&client, &catalog).title, "Preferences");
    assert_eq!(client.preferences().language.as_deref(), Some("en"));
    assert!(file_text(dir.path()).contains(r#""language": "en""#));
}

#[test]
fn choosing_system_language_saves_null() {
    // Spec 080 C18
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let mut client = in_game(dir.path());
    client.command(Command::OpenPreferences);
    choose(&mut client, &french(), "English");
    client.take_window_requests();
    choose(&mut client, &english(), "System language (Français)");

    assert_eq!(
        client.take_window_requests(),
        [WindowRequest::Language(language("fr"))]
    );
    assert_eq!(client.preferences().language, None);
    assert!(file_text(dir.path()).contains(r#""language": null"#));
}

#[test]
fn language_change_leaves_game_untouched() {
    // Spec 080 C19
    let catalog = french();
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let mut client = in_game(dir.path());
    let game = client.game().expect("partie");
    let (log_len, undo, view) = (
        game.runner().log().len(),
        game.runner().undo_status(),
        game.view(),
    );

    client.command(Command::OpenPreferences);
    choose(&mut client, &catalog, "English");
    client.command(Command::ClosePreferences);

    let game = client.game().expect("partie");
    assert_eq!(game.runner().log().len(), log_len);
    assert_eq!(game.runner().undo_status(), undo);
    assert_eq!(game.view(), view);
}

#[test]
fn fullscreen_at_launch_is_independent_of_f11() {
    // Spec 080 C20 (Q3)
    let catalog = french();
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let mut client = in_game(dir.path());
    client.command(Command::OpenPreferences);
    client.command(Command::SetFullscreenAtLaunch(true));

    assert!(client.take_window_requests().is_empty());
    assert!(window(&client, &catalog).fullscreen_at_launch);
    assert!(file_text(dir.path()).contains(r#""fullscreen_at_launch": true"#));

    client.shortcut(Shortcut::Fullscreen);
    assert_eq!(
        client.take_window_requests(),
        [WindowRequest::Fullscreen(true)]
    );
    assert!(client.preferences().fullscreen_at_launch);
    assert!(file_text(dir.path()).contains(r#""fullscreen_at_launch": true"#));
}

#[test]
fn unsaved_preferences_show_message() {
    // Spec 080 C21 : le dossier de configuration est un fichier, l'écriture échoue.
    let catalog = french();
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let blocker = dir.path().join("config");
    fs::write(&blocker, "pas un dossier").expect("fichier");
    let path = blocker.join(PREFERENCES_FILE);
    let mut client =
        ClientSession::launch(None).with_preferences(Preferences::default(), Some(path), choices());
    client.command(Command::OpenPreferences);

    let ((), lines) = logs_of(|| client.command(Command::SetFullscreenAtLaunch(true)));
    assert!(
        client.preferences().fullscreen_at_launch,
        "vaut pour la session"
    );
    assert_eq!(
        window(&client, &catalog).not_saved.as_deref(),
        Some("Préférences non enregistrées (voir les logs).")
    );
    let shown = blocker.display().to_string();
    assert!(has_line(&lines, Level::Error, &[&shown]), "{lines:?}");

    fs::remove_file(&blocker).expect("fichier retiré");
    client.command(Command::SetFullscreenAtLaunch(false));
    assert_eq!(window(&client, &catalog).not_saved, None);
}

#[test]
fn preferences_shortcuts() {
    // Spec 080 C22
    let catalog = french();
    let bound = KEY_BINDINGS
        .iter()
        .find(|binding| binding.shortcut == Shortcut::Preferences)
        .map(|binding| (binding.key.name(), binding.modifier));
    assert_eq!(bound, Some(("Comma", KeyModifier::Ctrl)));
    assert_eq!(
        shortcut_label(Shortcut::Preferences, &catalog).as_deref(),
        Some("Ctrl+,")
    );

    // Ctrl+, ouvre et ferme ; Échap ferme la fenêtre des préférences avant celle de chargement.
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let mut client = in_game(dir.path());
    client.shortcut(Shortcut::Preferences);
    assert!(client.preferences_window(&catalog).is_some());
    client.shortcut(Shortcut::Preferences);
    assert!(client.preferences_window(&catalog).is_none());

    client.command(Command::OpenLoad);
    assert!(client.load_window(&catalog).is_some());
    client.shortcut(Shortcut::Preferences);
    client.shortcut(Shortcut::Cancel);
    assert!(client.preferences_window(&catalog).is_none());
    assert!(
        client.load_window(&catalog).is_some(),
        "Échap ne ferme qu'une fenêtre"
    );
}

#[test]
fn launch_screen_opens_preferences() {
    // Spec 080 C23
    let catalog = french();
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let mut client = launch(dir.path());
    let button = client
        .new_game_screen(&catalog)
        .expect("écran de nouvelle partie")
        .preferences
        .expect("bouton Préférences…");
    assert_eq!(button.label, "Préférences…");
    assert_eq!(button.command, Command::OpenPreferences);

    client.command(button.command);
    assert!(client.preferences_window(&catalog).is_some());
    choose(&mut client, &catalog, "English");
    assert_eq!(
        client.take_window_requests(),
        [WindowRequest::Language(language("en"))]
    );
    client.command(Command::ClosePreferences);
    assert!(client.new_game_screen(&english()).is_some());

    // Ctrl+, l'ouvre aussi au lancement ; Échap la ferme sans toucher à l'écran de lancement.
    client.shortcut(Shortcut::Preferences);
    assert!(client.preferences_window(&catalog).is_some());
    client.shortcut(Shortcut::Cancel);
    assert!(client.preferences_window(&catalog).is_none());
    assert!(client.new_game_screen(&catalog).is_some());

    // En cours de partie, l'écran de nouvelle partie n'a pas ce bouton : le Menu l'a.
    let mut client = in_game(dir.path());
    client.command(Command::OpenNewGame);
    assert_eq!(
        client.new_game_screen(&catalog).expect("écran").preferences,
        None
    );
}

// --- Cas limites ---

#[test]
fn preferences_without_assets_list_system_only() {
    // Spec 080 L1
    let missing = vec![PathBuf::from("dossier-absent")];
    let mut client = ClientSession::launch(None).with_preferences(
        Preferences::default(),
        None,
        LanguageChoices::load(&missing, Some("fr".into())),
    );
    client.command(Command::OpenPreferences);
    let empty = catalog_or_empty(&missing, &language("fr"));
    let shown = window(&client, &empty);
    assert_eq!(shown.languages.len(), 1);
    client.command(shown.languages[0].command);
    assert_eq!(
        client.take_window_requests(),
        [WindowRequest::Language(language("en"))]
    );
}

#[test]
fn no_config_dir_keeps_session_preferences() {
    // Spec 080 L2
    let catalog = french();
    let mut client =
        ClientSession::launch(None).with_preferences(Preferences::default(), None, choices());
    client.command(Command::OpenPreferences);
    assert_eq!(window(&client, &catalog).not_saved, None);
    client.command(Command::SetFullscreenAtLaunch(true));
    assert!(client.preferences().fullscreen_at_launch);
    assert_eq!(
        window(&client, &catalog).not_saved.as_deref(),
        Some("Préférences non enregistrées (voir les logs).")
    );
}

#[test]
fn unloadable_chosen_language_falls_back() {
    // Spec 080 L3 : `fr.json` de cette ressource ne se charge pas.
    let assets = vec![
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("assets")
            .join("francais-invalide"),
    ];
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let mut client = ClientSession::launch(None).with_preferences(
        Preferences::default(),
        Some(dir.path().join(PREFERENCES_FILE)),
        LanguageChoices::load(&assets, Some("en".into())),
    );
    client.command(Command::OpenPreferences);
    let index = 1; // en, fr : fr est la deuxième langue disponible
    client.command(Command::SetLanguage(Some(index)));
    assert_eq!(
        client.take_window_requests(),
        [WindowRequest::Language(language("fr"))]
    );
    // Repli de la spec 076 L1 : l'anglais.
    assert_eq!(
        catalog_or_empty(&assets, &language("fr")).text("language.name"),
        "English fixture"
    );
    assert!(file_text(dir.path()).contains(r#""language": "fr""#));
}

#[test]
fn preferences_window_blocks_game_keys() {
    // Spec 080 L4
    let catalog = french();
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let mut client = in_game(dir.path());
    let view = client.game().expect("partie").view();
    client.command(Command::OpenPreferences);

    client.shortcut(Shortcut::EndTurn);
    client.shortcut(Shortcut::Take(1));
    client.command(Command::Play(0));
    client.run_frame();
    let game = client.game().expect("partie");
    assert!(!game.is_ending_turn());
    assert_eq!(game.view(), view);
    assert!(client.preferences_window(&catalog).is_some());

    client.shortcut(Shortcut::Fullscreen);
    assert_eq!(
        client.take_window_requests(),
        [WindowRequest::Fullscreen(true)]
    );
    client.shortcut(Shortcut::Quit);
    assert_eq!(client.take_window_requests(), [WindowRequest::Close]);
    client.shortcut(Shortcut::Preferences);
    assert!(client.preferences_window(&catalog).is_none());
}

#[test]
fn animations_checkbox_is_saved() {
    // Spec #5 C13 : la case « Animations » dans « Affichage », cochée selon la préférence ; la
    // changer l'enregistre tout de suite.
    let catalog = french();
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let mut client = in_game(dir.path());
    client.command(Command::OpenPreferences);
    let shown = window(&client, &catalog);
    assert_eq!(shown.display_label, "Affichage");
    assert_eq!(shown.animations_label, "Animations");
    assert!(shown.animations, "cochée par défaut");

    client.command(Command::SetAnimations(false));
    assert!(!window(&client, &catalog).animations);
    assert!(!client.preferences().animations);
    assert!(file_text(dir.path()).contains(r#""animations": false"#));

    client.command(Command::SetAnimations(true));
    assert!(window(&client, &catalog).animations);
    assert!(file_text(dir.path()).contains(r#""animations": true"#));
}
