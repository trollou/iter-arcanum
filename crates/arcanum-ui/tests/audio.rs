//! Spec 081 (audio), côté arcanum-ui : événements entendus, situation sonore et commandes de volume.
//! Aucune règle du livret ici : chaque test cite le cas de la spec.

mod support;

// L'arène des tests d'arcanum-core, partagée plutôt que recopiée.
#[path = "../../arcanum-core/tests/arene/mod.rs"]
mod arene;

use arcanum_audio::{Outcome, SoundSituation};
use arcanum_core::{
    Action, CardPlay, Event, GameEndReason, PlayerEvent, PlayerId, Scenario, ScenarioId, TimeOfDay,
};
use arcanum_i18n::{Catalog, Language};
use arcanum_persistance::{SaveGame, save_auto};
use arcanum_ui::{ClientSession, Command, GameSession, game_outcome, repository_assets_dir};
use arene::{ORCS_2, PLAIN_W, Sandbox, prowlers};
use support::{first_reconnaissance_config, past_tactic_choice, session_of, started_session};

fn seed() -> u64 {
    7
}

/// Une partie neuve de Première reconnaissance, lancée depuis l'écran de lancement.
fn new_game() -> ClientSession {
    let mut client = ClientSession::launch(None).with_seed_source(seed);
    client.command(Command::ChooseScenario(ScenarioId::FirstReconnaissance));
    client.command(Command::StartNewGame);
    client.run_frame();
    client
}

fn game(client: &ClientSession) -> &GameSession {
    client.game().expect("une partie en cours")
}

/// Joue l'action légale `wanted` par l'écran, comme un clic.
fn play_where(client: &mut ClientSession, wanted: impl Fn(&Action) -> bool) {
    let index = game(client)
        .legal_actions()
        .iter()
        .position(wanted)
        .expect("action légale");
    client.command(Command::Play(index));
    client.run_frame();
}

/// Passe le choix de la tactique (spec 060) puis joue une carte de côté.
fn play_a_card_sideways(client: &mut ClientSession) {
    if game(client)
        .legal_actions()
        .iter()
        .any(|action| matches!(action, Action::ChooseTactic { .. }))
    {
        play_where(client, |action| {
            matches!(action, Action::ChooseTactic { .. })
        });
    }
    play_where(client, |action| {
        matches!(
            action,
            Action::PlayCard {
                play: CardPlay::Sideways(_),
                ..
            }
        )
    });
}

fn played_cards(events: &[PlayerEvent]) -> usize {
    events
        .iter()
        .filter(|event| matches!(event, PlayerEvent::CardPlayed { .. }))
        .count()
}

#[test]
fn new_game_events_are_heard_once() {
    // C6 : la mise en place est entendue, une seule fois. Le premier jour n'a pas de
    // `RoundStarted` : son coq vient de `GameStarted` (spec 081, table des effets).
    let mut client = new_game();
    let heard = client.take_heard_events();
    assert!(
        matches!(heard.first(), Some(PlayerEvent::GameStarted { .. })),
        "{heard:?}"
    );
    assert_eq!(client.take_heard_events(), vec![]);
}

#[test]
fn undo_replays_no_sound() {
    // C7 : après un coup et Revenir en arrière, rien ; le coup rejoué est entendu.
    let mut client = new_game();
    client.take_heard_events();
    play_a_card_sideways(&mut client);
    assert_eq!(played_cards(&client.take_heard_events()), 1);
    client.command(Command::Undo);
    client.run_frame();
    assert_eq!(client.take_heard_events(), vec![]);
    play_where(&mut client, |action| {
        matches!(
            action,
            Action::PlayCard {
                play: CardPlay::Sideways(_),
                ..
            }
        )
    });
    assert_eq!(played_cards(&client.take_heard_events()), 1);
}

#[test]
fn loaded_game_is_not_replayed() {
    // C8 : « Continuer », puis la fenêtre de chargement en cours de partie.
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let mut saved = started_session(first_reconnaissance_config(3));
    past_tactic_choice(&mut saved);
    let game_save = SaveGame::from_runner(saved.runner(), "2026-11-20 20:00").expect("sauvegarde");
    save_auto(dir.path(), &game_save).expect("écriture");

    let mut client = ClientSession::launch(Some(dir.path().to_path_buf()));
    client.command(Command::ContinueGame);
    client.run_frame();
    assert!(client.game().is_some(), "partie reprise");
    assert_eq!(client.take_heard_events(), vec![]);

    // Une partie neuve entendue, puis une sauvegarde chargée par-dessus : rien de rejoué.
    let mut client = ClientSession::launch(Some(dir.path().to_path_buf())).with_seed_source(seed);
    client.command(Command::StartNewGame);
    client.run_frame();
    assert_ne!(client.take_heard_events(), vec![]);
    client.command(Command::OpenLoad);
    client.command(Command::Load(0));
    client.run_frame();
    assert_eq!(client.take_heard_events(), vec![]);
}

#[test]
fn heard_events_are_filtered_for_the_player() {
    // C9 : la pioche mélangée s'entend sans l'ordre des cartes (ADR 0002).
    let mut client = new_game();
    let heard = client.take_heard_events();
    assert_eq!(
        heard,
        game(&client).runner().log().filtered_for(PlayerId::SOLO)
    );
    assert!(
        heard
            .iter()
            .any(|event| matches!(event, PlayerEvent::DeckShuffled { player, .. } if *player == PlayerId::SOLO)),
        "{heard:?}"
    );
}

// --- Situation sonore ---

fn french() -> Catalog {
    Catalog::load(
        &repository_assets_dir().join("i18n"),
        &Language("fr".into()),
    )
    .expect("fr.json du dépôt")
}

/// Une partie de bac à sable à ce moment de la journée, suivie de `more`.
fn sandbox_with(time: TimeOfDay, more: Vec<Event>) -> ClientSession {
    let mut events = Sandbox {
        figure: Some(PLAIN_W),
        placed: vec![(prowlers(), ORCS_2)],
        time,
        ..Sandbox::default()
    }
    .events();
    events.extend(more);
    ClientSession::launch(None).with_game(session_of(events))
}

#[test]
fn situation_follows_the_screen() {
    // C10
    let mut client = ClientSession::launch(None);
    assert_eq!(client.sound_situation(), SoundSituation::Title);
    client = new_game();
    assert_eq!(
        client.sound_situation(),
        SoundSituation::Exploration(TimeOfDay::Day)
    );
    client.command(Command::OpenNewGame);
    assert_eq!(client.sound_situation(), SoundSituation::Title);
    let night = sandbox_with(TimeOfDay::Night, vec![]);
    assert_eq!(
        night.sound_situation(),
        SoundSituation::Exploration(TimeOfDay::Night)
    );
}

#[test]
fn situation_in_combat_and_after_game_end() {
    // C11 : en combat, « Voir le plateau » compris ; puis la partie finie.
    let combat = Event::CombatStarted {
        player: PlayerId::SOLO,
        enemies: vec![(ORCS_2, prowlers())],
    };
    let mut client = sandbox_with(TimeOfDay::Night, vec![combat]);
    assert_eq!(
        client.sound_situation(),
        SoundSituation::Combat(TimeOfDay::Night)
    );
    let toggle = client
        .game()
        .and_then(|game| game.combat_screen(&french()))
        .expect("écran de combat")
        .board_toggle
        .command;
    client.command(toggle);
    assert_eq!(
        client.sound_situation(),
        SoundSituation::Combat(TimeOfDay::Night)
    );

    let ended = Event::GameEnded {
        reason: GameEndReason::LastRound,
    };
    let finished = sandbox_with(TimeOfDay::Day, vec![ended]);
    assert_eq!(
        finished.sound_situation(),
        SoundSituation::GameOver(TimeOfDay::Day, Outcome::Victory)
    );
}

#[test]
fn game_over_tells_victory_or_defeat() {
    // C28 : une ligne par case du tableau de la spec.
    use GameEndReason::*;
    use Outcome::*;
    let cases = [
        (ScenarioId::SoloConquest, AllCitiesConquered, Victory),
        (ScenarioId::SoloConquest, LastRound, Defeat),
        (ScenarioId::FirstReconnaissance, CityRevealed, Victory),
        (ScenarioId::FirstReconnaissance, LastRound, Defeat),
        (ScenarioId::Sandbox, LastRound, Victory),
    ];
    for (scenario, reason, expected) in cases {
        assert_eq!(
            game_outcome(Some(scenario), reason),
            expected,
            "{scenario:?}, {reason:?}"
        );
    }
    // La vue porte le scénario : Première reconnaissance perdue.
    let mut events = Sandbox {
        scenario: Scenario::first_reconnaissance().without_dummy(),
        figure: Some(PLAIN_W),
        ..Sandbox::default()
    }
    .events();
    events.push(Event::GameEnded { reason: LastRound });
    let client = ClientSession::launch(None).with_game(session_of(events));
    assert_eq!(
        client.sound_situation(),
        SoundSituation::GameOver(TimeOfDay::Day, Defeat)
    );
}

// --- Volumes ---

use arcanum_audio::SoundCategory;
use arcanum_persistance::preferences::{PREFERENCES_FILE, Preferences, load as load_preferences};
use arcanum_ui::{LanguageChoices, WindowRequest};

/// L'écran de lancement, préférences par défaut écrites dans `path`, fenêtre ouverte.
fn preferences_open(path: std::path::PathBuf) -> ClientSession {
    let mut client = ClientSession::launch(None).with_preferences(
        Preferences::default(),
        Some(path),
        LanguageChoices::default(),
    );
    client.command(Command::OpenPreferences);
    client.take_window_requests();
    client
}

#[test]
fn volume_applies_live_and_saves_on_release() {
    // C20
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let path = dir.path().join(PREFERENCES_FILE);
    let mut client = preferences_open(path.clone());
    client.command(Command::SetVolume(SoundCategory::Music, 30));
    let volumes = client.preferences().volumes;
    assert_eq!(volumes.music, 30);
    assert!(
        client
            .take_window_requests()
            .contains(&WindowRequest::Volumes(volumes))
    );
    assert!(!path.exists(), "rien d'écrit pendant le glissé");
    client.command(Command::SaveVolumes);
    assert_eq!(load_preferences(&path).expect("écrit").volumes.music, 30);
}

#[test]
fn dragging_a_volume_plays_its_sample() {
    // C21 : un échantillon par glissé, arrêté au lâcher.
    for category in [
        SoundCategory::Master,
        SoundCategory::Music,
        SoundCategory::Ambience,
        SoundCategory::Effects,
    ] {
        let dir = tempfile::tempdir().expect("dossier temporaire");
        let mut client = preferences_open(dir.path().join(PREFERENCES_FILE));
        let previews = |client: &mut ClientSession| -> Vec<WindowRequest> {
            client
                .take_window_requests()
                .into_iter()
                .filter(|request| matches!(request, WindowRequest::Preview(_)))
                .collect()
        };
        client.command(Command::SetVolume(category, 40));
        assert_eq!(
            previews(&mut client),
            vec![WindowRequest::Preview(Some(category))],
            "{category:?}"
        );
        client.command(Command::SetVolume(category, 45));
        assert_eq!(previews(&mut client), vec![], "{category:?}");
        client.command(Command::SaveVolumes);
        assert_eq!(
            previews(&mut client),
            vec![WindowRequest::Preview(None)],
            "{category:?}"
        );
    }
}

#[test]
fn unsaved_volumes_show_message() {
    // C22 : comme la spec 080 C21, le volume vaut pour la session.
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let blocker = dir.path().join("fichier");
    std::fs::write(&blocker, "pas un dossier").expect("fichier");
    let mut client = preferences_open(blocker.join(PREFERENCES_FILE));
    client.command(Command::SetVolume(SoundCategory::Effects, 10));
    client.command(Command::SaveVolumes);
    assert_eq!(client.preferences().volumes.effects, 10);
    let window = client
        .preferences_window(&french())
        .expect("fenêtre ouverte");
    assert_eq!(
        window.not_saved.as_deref(),
        Some("Préférences non enregistrées (voir les logs).")
    );
}

#[test]
fn preferences_window_lists_volumes() {
    // C23 : section « Son », quatre curseurs avec leur valeur ; sans sortie audio, une ligne
    // le dit (L1).
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let client = preferences_open(dir.path().join(PREFERENCES_FILE));
    let catalog = french();
    let window = client
        .preferences_window(&catalog)
        .expect("fenêtre ouverte");
    assert_eq!(window.sound_label, "Son");
    let sliders: Vec<(SoundCategory, &str, u8)> = window
        .volumes
        .iter()
        .map(|slider| (slider.category, slider.label.as_str(), slider.value))
        .collect();
    assert_eq!(
        sliders,
        vec![
            (SoundCategory::Master, "Volume général", 80),
            (SoundCategory::Music, "Musique", 50),
            (SoundCategory::Ambience, "Ambiances", 50),
            (SoundCategory::Effects, "Effets sonores", 80),
        ]
    );
    assert_eq!(arcanum_ui::volume_value_label(80, &catalog), "80\u{a0}%");
    assert_eq!(window.no_audio, None);

    let silent = preferences_open(dir.path().join(PREFERENCES_FILE)).with_audio_output(false);
    assert_eq!(
        silent
            .preferences_window(&catalog)
            .expect("fenêtre ouverte")
            .no_audio
            .as_deref(),
        Some("Aucune sortie audio : le jeu reste muet.")
    );
}
