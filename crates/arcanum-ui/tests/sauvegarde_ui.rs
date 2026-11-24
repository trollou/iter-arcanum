//! Scénarios de la spec 024 côté affichage (partie C) : revenir en arrière, sauvegarder, charger,
//! sauvegarde automatique, dossier des sauvegardes.

use std::ffi::OsString;
use std::path::PathBuf;

use arcanum_core::{
    Action, BasicAction, BasicColor, CardId, CardPlay, DeedCard, Event, EventLog, GameConfig,
    GameRunner, Hero, Hex, ManaColor, PlayerId, RngPositions, Scenario, TileId, TilePosition,
};
use arcanum_i18n::{Catalog, Language};
use arcanum_persistance::{AUTOSAVE_NAME, load};
use arcanum_ui::{
    Command, GameSession, Shortcut, choose_saves_dir, hover_text, human_player,
    repository_assets_dir,
};
use chrono::{DateTime, Local, TimeZone};

mod support;
use support::{first_reconnaissance_config, play};

const SOLO: PlayerId = PlayerId::SOLO;

fn french() -> Catalog {
    Catalog::load(
        &repository_assets_dir().join("i18n"),
        &Language("fr".into()),
    )
    .expect("fr.json du dépôt")
}

fn fixed_clock() -> DateTime<Local> {
    Local
        .with_ymd_and_hms(2026, 10, 27, 18, 0, 0)
        .single()
        .expect("date non ambiguë")
}

fn card(action: BasicAction, copy: u16) -> CardId {
    CardId::new(DeedCard::Basic(action), copy)
}

fn basic(card: CardId) -> Action {
    Action::PlayCard {
        card,
        play: CardPlay::Basic {
            die: None,
            gain: None,
            with: None,
            option: 0,
            discard: None,
            mana: None,
        },
    }
}

/// Partie au bord de la tuile de départ (comme les tests d'exploration) : Arythea, deux Marches,
/// Endurance, Improvisation et Concentration en main.
fn edge_session(saves_dir: Option<PathBuf>) -> GameSession {
    let config = GameConfig {
        seed: 0,
        scenario: Scenario::sandbox(),
        hero: Hero::Arythea,
        dummy_hero: Default::default(),
    };
    let hand = [
        card(BasicAction::March, 1),
        card(BasicAction::March, 2),
        card(BasicAction::Stamina, 1),
        card(BasicAction::Improvisation, 1),
        card(BasicAction::Concentration, 1),
    ];
    let mut order = hand.to_vec();
    order.extend(
        Hero::Arythea
            .starting_deck()
            .into_iter()
            .filter(|card| !hand.contains(card)),
    );
    let mut tiles: Vec<TileId> = (1..=8).map(TileId::Countryside).collect();
    tiles.extend([TileId::Core(2), TileId::Core(6), TileId::Core(3)]);
    let mut events = vec![
        Event::GameStarted { config },
        Event::TilePlaced {
            tile: TileId::StartingA,
            position: TilePosition::ORIGIN,
        },
        Event::TileDeckFormed {
            order: tiles.clone(),
        },
        Event::TileRevealed {
            tile: tiles[0],
            position: TilePosition::from_lattice(0, 1),
        },
        Event::TileRevealed {
            tile: tiles[1],
            position: TilePosition::from_lattice(1, 0),
        },
        Event::HeroFigurePlaced {
            player: SOLO,
            hex: Hex::new(4, -3),
        },
        Event::DeckShuffled {
            player: SOLO,
            order,
        },
    ];
    events.extend(
        hand.iter()
            .map(|&card| Event::CardDrawn { player: SOLO, card }),
    );
    events.push(Event::SourceRolled {
        dice: vec![ManaColor::Basic(BasicColor::Green); 3],
    });
    let mut log = EventLog::default();
    for event in events {
        log.push(0, event);
    }
    let (player, human) = human_player();
    let runner = GameRunner::resume(log, RngPositions::new(), vec![Box::new(player)])
        .expect("journal valide");
    let mut session = GameSession::with_runner(runner, human)
        .with_clock(fixed_clock)
        .with_saves_dir(saves_dir);
    session.run_frame();
    session
}

// Spec 024 C23
#[test]
fn undo_button_follows_status() {
    let catalog = french();
    let mut session = edge_session(None);

    let undo = session.controls(&catalog).undo;
    assert!(!undo.enabled);
    assert_eq!(undo.label, "Revenir en arrière");
    assert_eq!(undo.hint, catalog.text("undo.blocked.turn_start"));

    play(&mut session, &basic(card(BasicAction::March, 1)));
    let undo = session.controls(&catalog).undo;
    assert!(undo.enabled);
    assert_eq!(undo.hint, catalog.text("undo.hint"));

    play(
        &mut session,
        &Action::Explore {
            position: TilePosition::from_lattice(2, 0),
        },
    );
    let undo = session.controls(&catalog).undo;
    assert!(!undo.enabled);
    assert_eq!(undo.hint, catalog.text("undo.blocked.revealed"));
}

// Spec 024 C1 côté écran : le bouton reprend la carte jouée.
#[test]
fn undo_command_takes_card_back() {
    let catalog = french();
    let mut session = edge_session(None);
    let before = session.view();

    play(&mut session, &basic(card(BasicAction::March, 1)));
    session.command(session.controls(&catalog).undo.command);
    session.run_frame();

    assert_eq!(session.view(), before);
    assert!(
        session
            .legal_actions()
            .contains(&basic(card(BasicAction::March, 1)))
    );
}

// Spec 024 C24 et L7
#[test]
fn undo_shortcut_outside_modes_only() {
    let catalog = french();
    let mut session = edge_session(None);
    play(&mut session, &basic(card(BasicAction::March, 1)));

    session.command(Command::OpenEndTurn);
    assert!(session.is_ending_turn());
    let controls = session.controls(&catalog);
    assert!(!controls.undo.enabled);
    assert!(!controls.save.enabled);
    assert!(!controls.load.enabled);
    session.shortcut(Shortcut::Undo);
    session.run_frame();
    assert_eq!(session.view().play_area.len(), 1);

    session.shortcut(Shortcut::Cancel);
    session.shortcut(Shortcut::Undo);
    session.run_frame();
    assert!(session.view().play_area.is_empty());
}

// Spec 024 C25
#[test]
fn autosave_at_turn_start() {
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let mut session = edge_session(Some(dir.path().to_path_buf()));
    let auto = dir.path().join(format!("{AUTOSAVE_NAME}.json"));
    assert_eq!(load(&auto).map(|game| game.summary.turn), Ok(1));

    // Une carte jouée : pas de nouvelle sauvegarde automatique au milieu du tour.
    let at_start = load(&auto).expect("lue");
    play(&mut session, &basic(card(BasicAction::March, 1)));
    assert_eq!(load(&auto), Ok(at_start));

    session.command(Command::OpenEndTurn);
    session.command(Command::ConfirmEndTurn);
    session.run_frame();
    let game = load(&auto).expect("lue");
    assert_eq!(game.summary.turn, 2);
    let (player, _) = human_player();
    let reloaded = game.resume(vec![Box::new(player)]).expect("reprise");
    assert_eq!(arcanum_core::view(reloaded.state(), SOLO), session.view());
}

// Spec 024 C26
#[test]
fn loading_replaces_game() {
    let catalog = french();
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let mut session = edge_session(Some(dir.path().to_path_buf()));
    let saved_view = session.view();

    session.shortcut(Shortcut::Save);
    assert_eq!(
        session.controls(&catalog).notice.as_deref(),
        Some("Partie sauvegardée : 2026-10-27_18-00-00")
    );
    play(&mut session, &basic(card(BasicAction::March, 1)));
    assert_ne!(session.view(), saved_view);

    session.shortcut(Shortcut::OpenLoad);
    let window = session.load_window(&catalog).expect("fenêtre ouverte");
    assert_eq!(window.title, "Charger une partie");
    assert_eq!(window.lines.len(), 2, "automatique et manuelle");
    assert!(
        window.lines[0]
            .text
            .starts_with("Sauvegarde automatique · Bac à sable · Arythea")
    );
    assert!(
        window.lines[1]
            .text
            .starts_with("2026-10-27 18:00 · Bac à sable · Arythea")
    );
    let load_manual = window.lines[1].load.clone().expect("lisible");

    session.command(load_manual.command);
    session.run_frame();
    assert!(session.load_window(&catalog).is_none());
    assert_eq!(session.view(), saved_view);
}

// Spec 024 L3 et L6 : sans dossier, sauvegarder échoue proprement ; Échap ferme la fenêtre.
#[test]
fn save_without_dir_reports_failure() {
    let catalog = french();
    let mut session = GameSession::new(first_reconnaissance_config(1)).with_clock(fixed_clock);
    session.run_frame();

    session.command(Command::Save);
    assert_eq!(
        session.controls(&catalog).notice,
        Some(catalog.text("save.failed"))
    );

    session.command(Command::OpenLoad);
    let window = session.load_window(&catalog).expect("fenêtre ouverte");
    assert_eq!(window.empty, Some(catalog.text("load.empty")));
    session.shortcut(Shortcut::Cancel);
    assert!(session.load_window(&catalog).is_none());
}

// Spec 024 C27 — même ordre que le dossier des logs (spec 003).
#[test]
fn saves_dir_choice() {
    let user = PathBuf::from("donnees");
    let repo = PathBuf::from("depot");
    let manifest = OsString::from(repo.join("crates").join("arcanum-ui"));

    assert_eq!(
        choose_saves_dir(
            Some(OsString::from("ailleurs")),
            Some(manifest.clone()),
            Some(user.clone())
        ),
        Some(PathBuf::from("ailleurs"))
    );
    assert_eq!(
        choose_saves_dir(Some(OsString::new()), Some(manifest), Some(user.clone())),
        Some(repo.join("saves"))
    );
    assert_eq!(
        choose_saves_dir(None, None, Some(user.clone())),
        Some(user.join("saves"))
    );
    assert_eq!(choose_saves_dir(None, None, None), None);
}

// Spec 024 L7 : la fenêtre de chargement ouverte, les autres boutons du panneau n'agissent pas.
#[test]
fn load_window_blocks_other_commands() {
    let catalog = french();
    let mut session = edge_session(None);
    let before = session.view();

    session.command(Command::OpenLoad);
    session.command(Command::Play(0));
    session.command(Command::OpenEndTurn);
    session.run_frame();

    assert_eq!(session.view(), before);
    assert!(!session.is_ending_turn());
    assert!(session.load_window(&catalog).is_some());
}

// Spec 071, C18 et L2 : description sans la touche, puis « Raccourci : Ctrl+Z », grisé compris.
#[test]
fn control_buttons_show_shortcut() {
    let catalog = french();
    let mut session = edge_session(None);
    let controls = session.controls(&catalog);
    // Au début du tour, rien à reprendre : grisé, la raison puis la touche.
    assert!(!controls.undo.enabled);
    assert_eq!(controls.undo.shortcut.as_deref(), Some("Ctrl+Z"));
    assert_eq!(
        hover_text(
            Some(&controls.undo.hint),
            controls.undo.shortcut.as_deref(),
            &catalog
        )
        .as_deref(),
        Some("Rien à reprendre depuis le début du tour.\nRaccourci : Ctrl+Z")
    );
    play(&mut session, &basic(card(BasicAction::March, 1)));
    let controls = session.controls(&catalog);
    let lines: Vec<(String, Option<String>)> = [
        &controls.undo,
        &controls.save,
        &controls.load,
        &controls.new_game,
    ]
    .iter()
    .map(|button| {
        (
            button.hint.clone(),
            hover_text(Some(&button.hint), button.shortcut.as_deref(), &catalog),
        )
    })
    .collect();
    assert_eq!(
        lines,
        [
            (
                "Reprendre votre dernière action".to_string(),
                Some("Reprendre votre dernière action\nRaccourci : Ctrl+Z".to_string())
            ),
            (
                "Sauvegarder la partie".to_string(),
                Some("Sauvegarder la partie\nRaccourci : Ctrl+S".to_string())
            ),
            (
                "Charger une partie".to_string(),
                Some("Charger une partie\nRaccourci : Ctrl+O".to_string())
            ),
            (
                "Commencer une autre partie".to_string(),
                Some("Commencer une autre partie\nRaccourci : Ctrl+N".to_string())
            ),
        ]
    );
}

#[test]
fn user_dirs_are_named_iter_arcanum() {
    // Spec 082 C3 : sauvegardes et préférences dans un dossier `iter-arcanum`, sans reprise des anciens.
    let named = |dir: PathBuf| {
        dir.components()
            .any(|part| part.as_os_str() == "iter-arcanum")
    };
    let data = arcanum_ui::user_data_dir().expect("dossier de données");
    assert!(named(data.clone()), "{data:?}");
    let config = arcanum_ui::user_config_dir().expect("dossier de configuration");
    assert!(named(config.clone()), "{config:?}");
}
