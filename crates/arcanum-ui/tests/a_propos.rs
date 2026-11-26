//! Scénarios de la spec #33 (release) côté jeu : version, commit et fenêtre « À propos ».
//! Aucune règle du livret ici : chaque test cite le cas de la spec.

mod support;
use support::{ctrl, first_reconnaissance_config};

use std::path::{Path, PathBuf};

use arcanum_core::{GameConfig, Scenario};
use arcanum_i18n::{Catalog, Language};
use arcanum_persistance::preferences::{PREFERENCES_FILE, Preferences};
use arcanum_ui::{
    AboutWindow, ClientSession, Command, GameSession, LanguageChoices, MenuBar, MenuItem, Shortcut,
    WindowRequest, catalog_or_empty, repository_assets_dir, short_commit, version_label,
};

// --- Outils de test ---

fn repository() -> Vec<PathBuf> {
    vec![repository_assets_dir()]
}

fn french() -> Catalog {
    catalog_or_empty(&repository(), &Language("fr".into()))
}

fn english() -> Catalog {
    catalog_or_empty(&repository(), &Language("en".into()))
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
        LanguageChoices::load(&repository(), Some("fr".into())),
    )
}

/// Une partie en cours, préférences par défaut écrites dans `dir`.
fn in_game(dir: &Path) -> ClientSession {
    launch(dir).with_game(sandbox())
}

fn window(client: &ClientSession, catalog: &Catalog) -> AboutWindow {
    client
        .about_window(catalog)
        .expect("fenêtre « À propos » ouverte")
}

type Line = Option<(String, Option<String>, Option<String>, bool)>;

fn menu_lines(bar: &MenuBar) -> Vec<Line> {
    bar.items
        .iter()
        .map(|item| match item {
            MenuItem::Entry(entry) => Some((
                entry.label.clone(),
                entry.shortcut.clone(),
                entry.hint.clone(),
                entry.enabled,
            )),
            MenuItem::Separator => None,
        })
        .collect()
}

fn about_enabled(bar: &MenuBar) -> bool {
    bar.items
        .iter()
        .find_map(|item| match item {
            MenuItem::Entry(entry) if entry.command == Command::OpenAbout => Some(entry.enabled),
            _ => None,
        })
        .expect("élément « À propos… »")
}

fn line(label: &str, shortcut: Option<String>) -> Line {
    Some((label.to_owned(), shortcut, None, true))
}

// --- Version et commit ---

#[test]
fn version_label_adds_short_commit() {
    // Spec #33 C1, D4
    assert_eq!(version_label("0.1.0", Some("a1b2c3d")), "0.1.0 (a1b2c3d)");
}

#[test]
fn version_label_without_commit_is_bare_version() {
    // Spec #33 C1, L1
    assert_eq!(version_label("0.2.0-dev", None), "0.2.0-dev");
}

#[test]
fn short_commit_keeps_seven_characters_of_full_sha() {
    // Spec #33 C2
    assert_eq!(
        short_commit(Some("a1b2c3d4e5f60718293a4b5c6d7e8f9012345678")),
        Some("a1b2c3d")
    );
}

#[test]
fn short_commit_keeps_short_sha() {
    // Spec #33 C2
    assert_eq!(short_commit(Some("a1b2c3d")), Some("a1b2c3d"));
}

#[test]
fn short_commit_rejects_missing_empty_or_not_hexadecimal() {
    // Spec #33 C2
    assert_eq!(short_commit(None), None);
    assert_eq!(short_commit(Some("")), None);
    assert_eq!(short_commit(Some("pas-un-sha-git")), None);
    assert_eq!(short_commit(Some("a1b2")), None, "trop court");
}

// --- Le Menu ---

#[test]
fn menu_lists_about_after_preferences() {
    // Spec #33 C3 : sans touche ni description, avant le séparateur et « Quitter ».
    let bar = sandbox().menu_bar(&french());
    let lines = menu_lines(&bar);
    assert_eq!(
        lines[lines.len() - 4..].to_vec(),
        [
            line("Préférences…", Some(ctrl(","))),
            line("À propos…", None),
            None,
            line("Quitter", Some(ctrl("Q"))),
        ]
    );
}

#[test]
fn about_entry_is_always_enabled() {
    // Spec #33 C3, comme « Préférences… » (spec 080 C14) : en mode choix, derrière la fenêtre de
    // nouvelle partie, en combat.
    let catalog = french();
    let mut choosing = sandbox();
    let card = choosing.view().hand[0];
    choosing.command(Command::ChooseDiscard { card, option: 0 });
    assert!(choosing.is_choosing_discard());
    assert!(about_enabled(&choosing.menu_bar(&catalog)));

    let dir = tempfile::tempdir().expect("dossier temporaire");
    let mut client = in_game(dir.path());
    client.command(Command::OpenNewGame);
    assert!(about_enabled(&client.menu_bar(&catalog).expect("barre")));

    let mut loading = sandbox();
    loading.command(Command::OpenLoad);
    assert!(about_enabled(&loading.menu_bar(&catalog)));
}

// --- La fenêtre ---

#[test]
fn about_window_lists_version_licenses_and_links() {
    // Spec #33 C4
    let catalog = french();
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let mut client = in_game(dir.path());
    assert!(client.about_window(&catalog).is_none());

    client.command(Command::OpenAbout);
    let shown = window(&client, &catalog);
    let version = version_label(
        env!("CARGO_PKG_VERSION"),
        short_commit(option_env!("ARCANUM_COMMIT")),
    );
    assert_eq!(shown.title, "À propos");
    assert_eq!(shown.name, "Iter Arcanum");
    assert_eq!(shown.version, format!("Version {version}"));
    assert_eq!(
        shown.license_code,
        "Code sous licence GNU GPL v3 ou ultérieure"
    );
    assert_eq!(
        shown.license_assets,
        "Visuels et textes sous licence CC BY-SA 4.0 ; chaque son garde la licence de son auteur"
    );
    assert_eq!(
        shown.unofficial,
        "Projet de fan non officiel, sans lien avec les éditeurs de Mage Knight."
    );
    assert_eq!(shown.repository_label, "Code source");
    assert_eq!(
        shown.repository_url,
        "https://github.com/trollou/iter-arcanum"
    );
    assert_eq!(
        shown.sounds,
        "Crédits des sons : assets/sounds/LICENSES.md, livré avec le jeu"
    );
    assert_eq!(shown.close.label, "Fermer");
    assert_eq!(shown.close.command, Command::CloseAbout);
}

#[test]
fn about_command_toggles_window() {
    // Spec #33 C5 : une seconde fois la referme, « Fermer » (et la croix) la ferment.
    let catalog = french();
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let mut client = in_game(dir.path());
    client.command(Command::OpenAbout);
    assert!(client.about_window(&catalog).is_some());
    client.command(Command::OpenAbout);
    assert!(client.about_window(&catalog).is_none());

    client.command(Command::OpenAbout);
    client.command(Command::CloseAbout);
    assert!(client.about_window(&catalog).is_none());
}

#[test]
fn escape_closes_about() {
    // Spec #33 C5 : Échap ferme « À propos » seulement, pas la fenêtre de chargement derrière.
    let catalog = french();
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let mut client = in_game(dir.path());
    client.command(Command::OpenLoad);
    client.command(Command::OpenAbout);
    client.shortcut(Shortcut::Cancel);
    assert!(client.about_window(&catalog).is_none());
    assert!(
        client.load_window(&catalog).is_some(),
        "Échap ne ferme qu'une fenêtre"
    );

    // Au lancement aussi, sans toucher à l'écran de nouvelle partie.
    let mut client = launch(dir.path());
    client.command(Command::OpenAbout);
    client.shortcut(Shortcut::Cancel);
    assert!(client.about_window(&catalog).is_none());
    assert!(client.new_game_screen(&catalog).is_some());
}

#[test]
fn about_window_blocks_game_commands() {
    // Spec #33 C6, comme spec 080 L4
    let catalog = french();
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let mut client = in_game(dir.path());
    let view = client.game().expect("partie").view();
    client.command(Command::OpenAbout);

    client.shortcut(Shortcut::EndTurn);
    client.shortcut(Shortcut::Take(1));
    client.command(Command::Play(0));
    client.run_frame();
    let game = client.game().expect("partie");
    assert!(!game.is_ending_turn());
    assert_eq!(game.view(), view);
    assert!(client.about_window(&catalog).is_some());

    client.shortcut(Shortcut::Fullscreen);
    assert_eq!(
        client.take_window_requests(),
        [WindowRequest::Fullscreen(true)]
    );
    client.shortcut(Shortcut::Quit);
    assert_eq!(client.take_window_requests(), [WindowRequest::Close]);

    // Au lancement, l'écran de nouvelle partie n'agit pas non plus.
    let mut client = launch(dir.path());
    let form = client.new_game_screen(&catalog);
    client.command(Command::OpenAbout);
    client.shortcut(Shortcut::NextStep);
    client.command(Command::StartNewGame);
    assert!(client.game().is_none());
    assert_eq!(client.new_game_screen(&catalog), form);
}

#[test]
fn about_and_preferences_are_exclusive() {
    // Spec #33 C7
    let catalog = french();
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let mut client = in_game(dir.path());
    client.command(Command::OpenPreferences);
    client.command(Command::OpenAbout);
    assert!(client.about_window(&catalog).is_some());
    assert!(client.preferences_window(&catalog).is_none());

    client.shortcut(Shortcut::Preferences);
    assert!(client.preferences_window(&catalog).is_some());
    assert!(client.about_window(&catalog).is_none());

    client.command(Command::OpenAbout);
    client.shortcut(Shortcut::Cancel);
    assert!(client.about_window(&catalog).is_none());
    assert!(client.preferences_window(&catalog).is_none());
}

#[test]
fn about_window_in_english() {
    // Spec #33 C8
    let catalog = english();
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let mut client = in_game(dir.path());
    client.command(Command::OpenAbout);
    let shown = window(&client, &catalog);
    assert_eq!(shown.title, "About");
    assert!(shown.version.starts_with("Version "), "{}", shown.version);
    assert_eq!(shown.license_code, "Code under the GNU GPL v3 or later");
    assert_eq!(
        shown.unofficial,
        "Unofficial fan project, not affiliated with the publishers of Mage Knight."
    );
    assert_eq!(shown.repository_label, "Source code");
    assert_eq!(
        shown.sounds,
        "Sound credits: assets/sounds/LICENSES.md, shipped with the game"
    );
    assert_eq!(shown.close.label, "Close");

    let bar = sandbox().menu_bar(&catalog);
    assert!(menu_lines(&bar).contains(&line("About…", None)));
}
