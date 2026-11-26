//! Spec 074 : la barre de menu (Menu, scénario, manche, pioches de tuiles, message) et les
//! commandes de la fenêtre (Recentrer, Plein écran, Quitter).

use arcanum_core::{GameConfig, Scenario};
use arcanum_i18n::{Catalog, Language};
use std::fs;
use std::path::Path;

use arcanum_persistance::AUTOSAVE_NAME;
use arcanum_ui::{
    ClientSession, Command, GameSession, KEY_BINDINGS, KeyModifier, MenuBar, MenuEntry, MenuItem,
    Shortcut, WindowRequest, repository_assets_dir, shortcut_label,
};

mod support;
use support::ctrl;
use support::first_reconnaissance_config;

fn french() -> Catalog {
    Catalog::load(
        &repository_assets_dir().join("i18n"),
        &Language("fr".into()),
    )
    .expect("fr.json du dépôt")
}

fn sandbox_config() -> GameConfig {
    GameConfig {
        scenario: Scenario::sandbox(),
        ..first_reconnaissance_config(3)
    }
}

/// Bac à sable au début du premier tour, sauvegarde automatique faite.
fn sandbox(saves_dir: Option<std::path::PathBuf>) -> GameSession {
    let mut session = GameSession::new(sandbox_config()).with_saves_dir(saves_dir);
    session.run_frame();
    session
}

fn entries(bar: &MenuBar) -> Vec<&MenuEntry> {
    bar.items
        .iter()
        .filter_map(|item| match item {
            MenuItem::Entry(entry) => Some(entry),
            MenuItem::Separator => None,
        })
        .collect()
}

fn entry(bar: &MenuBar, command: Command) -> &MenuEntry {
    entries(bar)
        .into_iter()
        .find(|entry| entry.command == command)
        .unwrap_or_else(|| panic!("{command:?} absent du Menu"))
}

/// Entre dans le mode choix de la carte à défausser (spec 013) avec la première carte de la main.
fn choose_discard(session: &mut GameSession) {
    let card = session.view().hand[0];
    session.command(Command::ChooseDiscard { card, option: 0 });
    assert!(session.is_choosing_discard());
}

// --- Le Menu ---

// C1
#[test]
fn menu_lists_commands_with_keys() {
    let catalog = french();
    let bar = sandbox(None).menu_bar(&catalog);
    assert_eq!(bar.title, "Menu");
    let lines: Vec<Option<(&str, Option<&str>)>> = bar
        .items
        .iter()
        .map(|item| match item {
            MenuItem::Entry(entry) => Some((entry.label.as_str(), entry.shortcut.as_deref())),
            MenuItem::Separator => None,
        })
        .collect();
    assert_eq!(
        lines,
        [
            Some(("Revenir en arrière", Some(ctrl("Z").as_str()))),
            None,
            Some(("Sauvegarder", Some(ctrl("S").as_str()))),
            Some(("Charger…", Some(ctrl("O").as_str()))),
            Some(("Nouvelle partie…", Some(ctrl("N").as_str()))),
            None,
            Some(("Recentrer le plateau", Some("Début"))),
            Some(("Plein écran", Some("F11"))),
            None,
            // Spec 080 C13
            Some(("Préférences…", Some(ctrl(",").as_str()))),
            // Spec #33 C3
            Some(("À propos…", None)),
            None,
            Some(("Quitter", Some(ctrl("Q").as_str()))),
        ]
    );
    let commands: Vec<Command> = entries(&bar).iter().map(|entry| entry.command).collect();
    assert_eq!(
        commands,
        [
            Command::Undo,
            Command::Save,
            Command::OpenLoad,
            Command::OpenNewGame,
            Command::Recenter,
            Command::ToggleFullscreen,
            Command::OpenPreferences,
            Command::OpenAbout,
            Command::Quit,
        ]
    );
    // Seul Plein écran porte une case ; Recentrer, Plein écran et Quitter n'ont pas de description.
    assert_eq!(entry(&bar, Command::ToggleFullscreen).checked, Some(false));
    for command in [Command::Recenter, Command::ToggleFullscreen, Command::Quit] {
        assert_eq!(entry(&bar, command).hint, None, "{command:?}");
    }
    assert_eq!(entry(&bar, Command::Save).checked, None);
}

// C2 : les quatre premiers éléments reprennent les boutons de `Controls` (spec 024).
#[test]
fn menu_follows_controls() {
    let catalog = french();
    let session = sandbox(None);
    let bar = session.menu_bar(&catalog);
    let controls = session.controls(&catalog);
    for button in [
        &controls.undo,
        &controls.save,
        &controls.load,
        &controls.new_game,
    ] {
        let entry = entry(&bar, button.command);
        assert_eq!(entry.label, button.label);
        assert_eq!(entry.hint.as_deref(), Some(button.hint.as_str()));
        assert_eq!(entry.enabled, button.enabled);
    }
    let undo = entry(&bar, Command::Undo);
    assert!(!undo.enabled);
    assert_eq!(
        undo.hint.as_deref(),
        Some("Rien à reprendre depuis le début du tour.")
    );
    assert!(entry(&bar, Command::Save).enabled);
}

// C3
#[test]
fn window_commands_stay_enabled_in_choice_mode() {
    let catalog = french();
    let mut session = sandbox(None);
    choose_discard(&mut session);
    let bar = session.menu_bar(&catalog);
    assert!(!entry(&bar, Command::Save).enabled);
    for command in [Command::Recenter, Command::ToggleFullscreen, Command::Quit] {
        assert!(entry(&bar, command).enabled, "{command:?}");
    }
}

// L1 : la fenêtre de nouvelle partie ouverte grise les commandes de la partie.
#[test]
fn menu_greys_game_commands_behind_new_game_window() {
    let catalog = french();
    let mut client = ClientSession::launch(None).with_game(sandbox(None));
    client.command(Command::OpenNewGame);
    assert!(client.new_game_screen(&catalog).is_some());
    let bar = client.menu_bar(&catalog).expect("une partie en cours");
    for command in [
        Command::Undo,
        Command::Save,
        Command::OpenLoad,
        Command::OpenNewGame,
    ] {
        assert!(!entry(&bar, command).enabled, "{command:?}");
    }
    for command in [Command::Recenter, Command::ToggleFullscreen, Command::Quit] {
        assert!(entry(&bar, command).enabled, "{command:?}");
    }
}

// L2 : la fenêtre de chargement ouverte, grisés comme aujourd'hui (spec 024, L7).
#[test]
fn menu_greys_game_commands_behind_load_window() {
    let catalog = french();
    let mut session = sandbox(None);
    session.command(Command::OpenLoad);
    let bar = session.menu_bar(&catalog);
    for command in [Command::Save, Command::OpenLoad, Command::OpenNewGame] {
        assert!(!entry(&bar, command).enabled, "{command:?}");
    }
    for command in [Command::ToggleFullscreen, Command::Quit] {
        assert!(entry(&bar, command).enabled, "{command:?}");
    }
}

// --- Les informations de la barre ---

// C6
#[test]
fn bar_shows_tile_decks() {
    let catalog = french();
    let bar = sandbox(None).menu_bar(&catalog);
    let decks: Vec<(arcanum_ui::Icon, &str, &str)> = bar
        .tile_decks
        .iter()
        .map(|deck| (deck.icon, deck.value.as_str(), deck.tooltip.as_str()))
        .collect();
    assert_eq!(
        decks,
        [
            (
                arcanum_ui::Icon::TileCountryside,
                "6",
                "Pioche de tuiles de campagne : 6"
            ),
            (
                arcanum_ui::Icon::TileCore,
                "3",
                "Pioche de tuiles royaume : 3"
            ),
        ]
    );
}

// C9
#[test]
fn bar_shows_save_notice() {
    let catalog = french();
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let mut session = sandbox(Some(dir.path().to_path_buf()));
    assert_eq!(session.menu_bar(&catalog).notice, None);
    session.command(Command::Save);
    let notice = session.menu_bar(&catalog).notice.expect("un message");
    assert!(
        notice.starts_with("Partie sauvegardée : "),
        "message : {notice}"
    );
}

// --- Plein écran, Recentrer, Quitter ---

fn client(saves_dir: Option<&Path>) -> ClientSession {
    ClientSession::launch(saves_dir.map(Path::to_path_buf))
        .with_game(sandbox(saves_dir.map(Path::to_path_buf)))
}

/// Joue de côté la première carte de la main (Mouvement 1 hors combat, MKUE p. 4, 3.e).
fn play_sideways(client: &mut ClientSession, catalog: &Catalog) {
    let command = client.game_screen(catalog).expect("une partie").hand[0].sideways[1].command;
    client.command(command);
    client.run_frame();
    let game = client.game().expect("une partie");
    assert_eq!(game.view().play_area.len(), 1);
}

fn autosave_text(dir: &Path) -> String {
    fs::read_to_string(dir.join(format!("{AUTOSAVE_NAME}.json"))).expect("sauvegarde automatique")
}

/// Les fichiers du dossier des sauvegardes.
fn files_in(dir: &Path) -> usize {
    fs::read_dir(dir).map_or(0, |entries| entries.count())
}

// C11
#[test]
fn toggle_fullscreen_requests_the_opposite_state() {
    let catalog = french();
    let mut client = client(None);
    client.command(Command::ToggleFullscreen);
    assert_eq!(
        client.take_window_requests(),
        [WindowRequest::Fullscreen(true)]
    );
    assert_eq!(client.take_window_requests(), []);

    client.set_fullscreen(true);
    let bar = client.menu_bar(&catalog).expect("une partie");
    assert_eq!(entry(&bar, Command::ToggleFullscreen).checked, Some(true));
    client.command(Command::ToggleFullscreen);
    assert_eq!(
        client.take_window_requests(),
        [WindowRequest::Fullscreen(false)]
    );
    // F11 fait de même.
    client.set_fullscreen(false);
    client.shortcut(Shortcut::Fullscreen);
    assert_eq!(
        client.take_window_requests(),
        [WindowRequest::Fullscreen(true)]
    );
}

// C12
#[test]
fn recenter_is_a_window_request() {
    let mut client = client(None);
    client.command(Command::Recenter);
    assert_eq!(client.take_window_requests(), [WindowRequest::Recenter]);
}

// C13 — Q1
#[test]
fn quit_at_turn_start_closes_at_once() {
    let catalog = french();
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let mut client = client(Some(dir.path()));
    let before = autosave_text(dir.path());
    let files = files_in(dir.path());

    client.command(Command::Quit);
    assert_eq!(client.quit_window(&catalog), None);
    assert_eq!(client.take_window_requests(), [WindowRequest::Close]);
    assert_eq!(autosave_text(dir.path()), before);
    assert_eq!(files_in(dir.path()), files);
}

// C14 — Q1
#[test]
fn quit_after_play_asks_confirmation() {
    let catalog = french();
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let mut client = client(Some(dir.path()));
    let before = autosave_text(dir.path());
    let files = files_in(dir.path());
    play_sideways(&mut client, &catalog);
    assert!(client.game().expect("une partie").advanced_since_autosave());

    client.command(Command::Quit);
    let window = client.quit_window(&catalog).expect("la confirmation");
    assert_eq!(window.title, "Quitter sans sauvegarder ?");
    assert_eq!(
        window.text,
        "Ce qui a été joué depuis le début du tour sera perdu."
    );
    assert_eq!(
        (window.quit.label.as_str(), window.quit.shortcut.as_deref()),
        ("Quitter", Some("Entrée"))
    );
    assert_eq!(
        (
            window.cancel.label.as_str(),
            window.cancel.shortcut.as_deref()
        ),
        ("Annuler", Some("Échap"))
    );
    assert_eq!(client.take_window_requests(), []);

    // « Annuler » : la partie continue, carte toujours jouée.
    client.command(window.cancel.command);
    assert_eq!(client.quit_window(&catalog), None);
    assert_eq!(client.game().expect("une partie").view().play_area.len(), 1);
    // Échap fait de même.
    client.shortcut(Shortcut::Quit);
    assert!(client.quit_window(&catalog).is_some());
    client.shortcut(Shortcut::Cancel);
    assert_eq!(client.quit_window(&catalog), None);
    assert_eq!(client.take_window_requests(), []);

    // « Quitter » (Entrée) : la fenêtre se ferme, rien n'est écrit.
    client.command(Command::Quit);
    client.shortcut(Shortcut::Confirm);
    assert_eq!(client.take_window_requests(), [WindowRequest::Close]);
    assert_eq!(autosave_text(dir.path()), before);
    assert_eq!(files_in(dir.path()), files);
    // Le bouton « Quitter » aussi.
    client.command(Command::Quit);
    client.command(window.quit.command);
    assert_eq!(client.take_window_requests(), [WindowRequest::Close]);
}

// C16 — Q1
#[test]
fn window_close_asks_confirmation_after_play() {
    let catalog = french();
    let mut at_start = client(None);
    assert!(at_start.window_close_requested());
    assert_eq!(at_start.quit_window(&catalog), None);

    let mut client = client(None);
    play_sideways(&mut client, &catalog);
    assert!(!client.window_close_requested());
    assert!(client.quit_window(&catalog).is_some());
    client.command(Command::ConfirmQuit);
    assert_eq!(client.take_window_requests(), [WindowRequest::Close]);
    // La fermeture demandée par « Quitter » repasse par la croix : elle passe.
    assert!(client.window_close_requested());
}

// C17
#[test]
fn quit_and_fullscreen_keys_are_bound() {
    let catalog = french();
    let bound = |shortcut| {
        KEY_BINDINGS
            .iter()
            .find(|binding| binding.shortcut == shortcut)
            .map(|binding| (binding.key.name(), binding.modifier))
    };
    assert_eq!(bound(Shortcut::Quit), Some(("Q", KeyModifier::Ctrl)));
    assert_eq!(bound(Shortcut::Fullscreen), Some(("F11", KeyModifier::Any)));
    assert_eq!(shortcut_label(Shortcut::Quit, &catalog), Some(ctrl("Q")));
    assert_eq!(
        shortcut_label(Shortcut::Fullscreen, &catalog).as_deref(),
        Some("F11")
    );
}

// C18 — Q5
#[test]
fn launch_screen_accepts_quit_and_fullscreen() {
    let catalog = french();
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let mut client = ClientSession::launch(Some(dir.path().to_path_buf()));
    assert!(client.new_game_screen(&catalog).is_some());
    assert_eq!(client.menu_bar(&catalog), None);

    client.shortcut(Shortcut::Fullscreen);
    assert_eq!(
        client.take_window_requests(),
        [WindowRequest::Fullscreen(true)]
    );
    client.shortcut(Shortcut::Quit);
    assert_eq!(client.quit_window(&catalog), None);
    assert_eq!(client.take_window_requests(), [WindowRequest::Close]);
    assert_eq!(files_in(dir.path()), 0);
}

// L3
#[test]
fn quit_without_saves_dir_follows_the_same_rule() {
    let catalog = french();
    let mut client = client(None);
    play_sideways(&mut client, &catalog);
    client.command(Command::Quit);
    assert!(client.quit_window(&catalog).is_some());
    assert_eq!(client.take_window_requests(), []);
}

// L4 : la confirmation ouverte, les autres touches et boutons ne jouent pas.
#[test]
fn quit_confirmation_blocks_other_input() {
    let catalog = french();
    let mut client = client(None);
    play_sideways(&mut client, &catalog);
    client.command(Command::Quit);
    let before = client.game().expect("une partie").view();

    client.command(Command::Play(0));
    client.command(Command::OpenEndTurn);
    client.shortcut(Shortcut::EndTurn);
    client.shortcut(Shortcut::Undo);
    client.shortcut(Shortcut::Quit);
    client.run_frame();

    let game = client.game().expect("une partie");
    assert_eq!(game.view(), before);
    assert!(!game.is_ending_turn());
    assert!(client.quit_window(&catalog).is_some());
    assert_eq!(client.take_window_requests(), []);
}

// L5 : en mode choix, Quitter suit la même règle ; « Annuler » rend le mode tel quel.
#[test]
fn quit_in_choice_mode_keeps_the_mode() {
    let catalog = french();
    let mut client = client(None);
    play_sideways(&mut client, &catalog);
    let card = client.game().expect("une partie").view().hand[0];
    client.command(Command::ChooseDiscard { card, option: 0 });
    assert!(client.game().expect("une partie").is_choosing_discard());

    client.shortcut(Shortcut::Quit);
    assert!(client.quit_window(&catalog).is_some());
    client.shortcut(Shortcut::Cancel);
    assert_eq!(client.quit_window(&catalog), None);
    assert!(client.game().expect("une partie").is_choosing_discard());
}
