//! Scénarios de la spec 063 (Première reconnaissance) côté affichage, sans fenêtre : nom du
//! scénario, dernier tour, fenêtre de fin et son décompte (C23 à C27).

use arcanum_core::{
    Action, BasicAction, BasicColor, Event, GameEndReason, GameState, Scenario, ScenarioId, TileId,
    TilePosition, apply, view,
};
use arcanum_i18n::{Catalog, Language};
use arcanum_ui::{
    Command, GameScreen, GameSession, ScreenMode, ScreenPhase, Shortcut, game_screen,
    repository_assets_dir,
};

mod support;
use support::{first_reconnaissance_config, session_of};

// L'arène des tests d'arcanum-core, partagée plutôt que recopiée.
#[path = "../../arcanum-core/tests/arene/mod.rs"]
mod arene;
use arene::*;

use BasicAction::*;

fn catalog(language: &str) -> Catalog {
    Catalog::load(
        &repository_assets_dir().join("i18n"),
        &Language(language.into()),
    )
    .expect("fichier de langue du dépôt")
}

fn french() -> Catalog {
    catalog("fr")
}

/// Première reconnaissance (ou `scenario`), au village, la tuile `next` prochaine de la pioche.
fn city_in_sight(scenario: Scenario, next: TileId) -> Vec<Event> {
    Sandbox {
        scenario,
        figure: Some(VILLAGE),
        hand: vec![
            card(March, 1),
            card(March, 2),
            card(Stamina, 1),
            card(Rage, 1),
            card(Promise, 1),
        ],
        next: vec![next],
        ..Sandbox::default()
    }
    .events()
}

/// Joue l'action qui satisfait `wanted`, qui doit être légale.
fn act(session: &mut GameSession, wanted: impl Fn(&Action) -> bool) {
    let index = session
        .legal_actions()
        .iter()
        .position(wanted)
        .expect("action légale");
    session.command(Command::Play(index));
    session.run_frame();
}

fn explore_city(session: &mut GameSession) {
    act(session, |action| {
        *action
            == Action::PlayCard {
                card: card(March, 1),
                play: arcanum_core::CardPlay::Basic {
                    die: None,
                    gain: None,
                    with: None,
                    option: 0,
                    discard: None,
                    mana: None,
                },
            }
    });
    act(session, |action| {
        *action
            == Action::Explore {
                position: TilePosition::from_lattice(1, 1),
            }
    });
}

fn end_turn(session: &mut GameSession) {
    act(session, |action| matches!(action, Action::EndTurn { .. }));
}

/// La cité révélée, le tour fini, puis le dernier tour joué : la partie est terminée.
fn finished() -> GameSession {
    let mut session = session_of(city_in_sight(
        Scenario::first_reconnaissance().without_dummy(),
        TileId::Core(5),
    ));
    explore_city(&mut session);
    end_turn(&mut session);
    act(&mut session, |action| {
        matches!(
            action,
            Action::PlayCard {
                play: arcanum_core::CardPlay::Sideways(_),
                ..
            }
        )
    });
    end_turn(&mut session);
    assert_eq!(session.view().game_end, Some(GameEndReason::CityRevealed));
    session
}

// C23 ; spec 074, C4 : la barre de menu donne le nom du scénario seul, sans le héros
// (remplace `panel_names_first_reconnaissance`).
#[test]
fn bar_names_scenario_only() {
    let bar = GameSession::new(first_reconnaissance_config(5)).menu_bar(&french());
    assert_eq!(bar.scenario.as_deref(), Some("Première reconnaissance"));
}

// Spec 074, C15 : quitter une partie terminée, sans confirmation ni écriture.
#[test]
fn quit_finished_game_closes_at_once() {
    let catalog = french();
    let mut client = arcanum_ui::ClientSession::launch(None).with_game(finished());
    assert!(client.game().is_some_and(GameSession::is_finished));
    client.command(arcanum_ui::Command::Quit);
    assert_eq!(client.quit_window(&catalog), None);
    assert_eq!(
        client.take_window_requests(),
        [arcanum_ui::WindowRequest::Close]
    );
}

// C24 — MKUE p. 18, *Scenario End*
#[test]
fn panel_announces_last_turn() {
    let catalog = french();
    let mut session = session_of(city_in_sight(
        Scenario::first_reconnaissance().without_dummy(),
        TileId::Core(5),
    ));
    let screen = session.screen(&catalog);
    assert_eq!((screen.last_turn, screen.last_turn_next), (None, None));

    explore_city(&mut session);
    let screen = session.screen(&catalog);
    assert_eq!(
        screen.last_turn_next.as_deref(),
        Some("Cité révélée : après ce tour, encore un dernier tour.")
    );
    assert_eq!(screen.last_turn, None);

    end_turn(&mut session);
    let screen = session.screen(&catalog);
    assert_eq!(screen.last_turn.as_deref(), Some("Dernier tour"));
    assert_eq!(screen.last_turn_next, None);
}

/// L'exemple de la spec : fin de partie au niveau 4, 17 de gloire, 3 cristaux, 2 Blessures.
fn example() -> GameState {
    let mut events = Sandbox {
        scenario: Scenario::first_reconnaissance().without_dummy(),
        figure: Some(VILLAGE),
        hand: vec![wound(1), wound(2), card(March, 1)],
        ..Sandbox::default()
    }
    .events();
    events.push(Event::CardDiscarded {
        player: SOLO,
        card: wound(1),
    });
    events.push(Event::CardDrawn {
        player: SOLO,
        card: card(March, 2),
    });
    events.extend(progress(17, 4));
    events.extend(
        [BasicColor::Red, BasicColor::Red, BasicColor::Blue].map(|color| Event::CrystalGained {
            player: SOLO,
            color,
        }),
    );
    events.push(Event::GameEnded {
        reason: GameEndReason::CityRevealed,
    });
    events.iter().fold(GameState::default(), apply)
}

fn finished_screen(state: &GameState, catalog: &Catalog) -> GameScreen {
    game_screen(
        &view(state, SOLO),
        ScreenPhase::Finished,
        &ScreenMode::Normal,
        catalog,
    )
}

// C25 — l'exemple de décompte de la spec
#[test]
fn end_window_shows_final_score() {
    let state = example();
    assert_eq!(arcanum_core::final_score(&state, SOLO).wounds, 2);
    let screen = finished_screen(&state, &french());
    let window = screen.score_window.expect("fenêtre de fin");
    assert_eq!(window.title, "Fin de partie");
    assert_eq!(
        window.reason,
        "Une cité a été révélée et le dernier tour est joué."
    );
    assert_eq!(
        window.lines,
        [
            "Gloire : 17",
            "Le Savant : 0",
            "Le Collectionneur (0 artefact(s), 3 cristaux) : 1",
            "Le Commandant : 0",
            "Le Conquérant (0 Bouclier(s)) : 0",
            "L'Aventurier : 0",
            "La Victime (2 Blessure(s)) : -4",
        ]
    );
    assert_eq!(window.total, "Score final : 14");
    assert_eq!(window.no_titles, "Variante solo : pas de bonus de titre.");
    assert_eq!(window.close.label, "Voir le plateau");
    assert_eq!(screen.reopen_score, None);
}

// C25 — « Voir le plateau » ferme la fenêtre, « Score final » la rouvre, Échap la ferme.
#[test]
fn end_window_closes_and_reopens() {
    let catalog = french();
    let mut session = finished();
    let screen = session.screen(&catalog);
    let window = screen.score_window.expect("fenêtre de fin");
    assert_eq!(window.lines[0], "Gloire : 1");
    assert!(screen.message.is_some());

    session.command(window.close.command);
    let screen = session.screen(&catalog);
    assert_eq!(screen.score_window, None);
    assert!(screen.message.is_some());
    let reopen = screen.reopen_score.expect("bouton Score final");
    assert_eq!(reopen.label, "Score final");

    session.command(reopen.command);
    let screen = session.screen(&catalog);
    assert!(screen.score_window.is_some());
    assert_eq!(screen.reopen_score, None);

    session.shortcut(Shortcut::Cancel);
    assert_eq!(session.screen(&catalog).score_window, None);
}

// C26 — spec 023
#[test]
fn sandbox_end_keeps_last_round_message() {
    let mut events = Sandbox {
        figure: Some(VILLAGE),
        time: arcanum_core::TimeOfDay::Night,
        ..Sandbox::default()
    }
    .events();
    events.extend([
        Event::RoundStarted {
            round: 4,
            time_of_day: arcanum_core::TimeOfDay::Night,
        },
        Event::DeckShuffled {
            player: SOLO,
            order: Vec::new(),
        },
    ]);
    let mut session = session_of(events);
    act(&mut session, |action| *action == Action::AnnounceEndOfRound);
    let screen = session.screen(&french());
    assert_eq!(
        screen.message.as_deref(),
        Some("Partie terminée : la dernière manche est jouée.")
    );
    assert_eq!(screen.score_window, None);
    assert_eq!(screen.reopen_score, None);
}

// C27
#[test]
fn every_scenario_key_exists_in_french() {
    let french = french();
    let english = catalog("en");
    let state = example();
    let last_round = apply(
        state.clone(),
        &Event::GameEnded {
            reason: GameEndReason::LastRound,
        },
    );
    let mut session = session_of(city_in_sight(
        Scenario::first_reconnaissance().without_dummy(),
        TileId::Core(5),
    ));
    for catalog in [&french, &english] {
        finished_screen(&state, catalog);
        finished_screen(&last_round, catalog);
        assert_eq!(
            catalog.text(ScenarioId::FirstReconnaissance.label_key()),
            catalog.text("scenario.first_reconnaissance.name")
        );
    }
    explore_city(&mut session);
    for catalog in [&french, &english] {
        session.screen(catalog);
    }
    end_turn(&mut session);
    for catalog in [&french, &english] {
        session.screen(catalog);
    }
    let mut finished = finished();
    finished.command(Command::CloseScore);
    for catalog in [&french, &english] {
        finished.screen(catalog);
    }
    assert_eq!(french.gaps(), vec![]);
    assert_eq!(english.gaps(), vec![]);
}
