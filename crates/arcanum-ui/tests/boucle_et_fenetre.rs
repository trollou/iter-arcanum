//! Scénarios de la spec 002 (boucle de partie et fenêtre minimale), sans fenêtre.
//! Aucune règle du livret ici : chaque test cite le cas de la spec.

use std::path::PathBuf;

use arcanum_core::{
    Action, BasicAction, CardId, CardPlay, DeedCard, Event, GameConfig, GameRunner, GameState,
    Hero, Player, PlayerId, PlayerView, Scenario, SidewaysEffect, StepOutcome, apply, view,
};
use arcanum_i18n::{Catalog, Language};
use arcanum_ui::{
    AssetsError, Command, FrameOutcome, GameScreen, GameSession, MAX_STEPS_PER_FRAME, ScreenMode,
    ScreenPhase, assets_candidates, catalog_or_empty, find_assets_dir, game_screen, human_player,
    load_catalog, repository_assets_dir, run_frame, run_frame_limited,
};

mod support;
use support::{
    announce_button, end_turn_button, first_reconnaissance_config, past_tactic_choice,
    started_session,
};

#[path = "../../arcanum-core/tests/premier_choix/mod.rs"]
mod premier_choix;
use premier_choix::FirstChoice;

const SOLO: PlayerId = PlayerId::SOLO;

fn config() -> GameConfig {
    GameConfig {
        seed: 7,
        scenario: Scenario::sandbox(),
        hero: Hero::Arythea,
        dummy_hero: Default::default(),
    }
}

fn card(action: BasicAction, copy: u16) -> CardId {
    CardId::new(DeedCard::Basic(action), copy)
}

/// Une action quelconque pour les tests du joueur humain (spec 013 : plus de `DrawCard`).
fn some_action() -> Action {
    Action::PlayCard {
        card: card(BasicAction::March, 1),
        play: CardPlay::Sideways(SidewaysEffect::Move),
    }
}

/// Les commandes de tous les boutons des cartes, dans l'ordre d'affichage (boutons-icônes de
/// côté compris, spec 071).
fn all_buttons(screen: &GameScreen) -> Vec<Command> {
    screen
        .hand
        .iter()
        .flat_map(|card| {
            card.plays
                .iter()
                .map(|button| button.command)
                .chain(card.sideways.iter().map(|button| button.command))
        })
        .collect()
}

/// Le premier bouton « de côté » de la première carte : toujours une action légale.
fn first_sideways(screen: &GameScreen) -> Command {
    screen.hand[0].sideways[0].command
}

/// Jouer de côté pour Influence (deuxième entrée du menu) : aucun point de mouvement, donc
/// aucun déplacement ne prolonge la partie (spec 014).
fn first_sideways_influence(screen: &GameScreen) -> Command {
    screen.hand[0].sideways[1].command
}

fn french() -> Catalog {
    Catalog::load(
        &repository_assets_dir().join("i18n"),
        &Language("fr".into()),
    )
    .expect("fr.json du dépôt")
}

fn french_language() -> Language {
    Language("fr".to_owned())
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("assets")
        .join(name)
}

fn some_view() -> PlayerView {
    // La vue d'une partie vide : un champ ajouté par une spec n'est rempli qu'une fois, par `view`.
    PlayerView {
        deck_size: 3,
        has_pending_decision: true,
        ..view(&GameState::default(), SOLO)
    }
}

// --- Joueur humain ---

// Spec 002, C2.
#[test]
fn human_without_request_has_nothing_pending() {
    let (mut player, handle) = human_player();
    assert_eq!(player.poll(), None);
    assert_eq!(handle.pending(), None);
}

// Spec 002, C3.
#[test]
fn human_waits_until_a_choice_is_submitted() {
    let (mut player, handle) = human_player();
    player.request(&some_view(), &[some_action()]);

    let pending = handle.pending().expect("une demande en cours");
    assert_eq!(pending.view, some_view());
    assert_eq!(pending.legal_actions, vec![some_action()]);
    assert_eq!(player.poll(), None);
    assert_eq!(player.poll(), None);
}

// Spec 002, C4.
#[test]
fn submitted_choice_is_returned_once() {
    let (mut player, handle) = human_player();
    player.request(&some_view(), &[some_action()]);
    handle.submit(0);

    assert_eq!(player.poll(), Some(0));
    assert_eq!(player.poll(), None);
    assert_eq!(handle.pending(), None);
}

// Spec 002, L1.
#[test]
fn choice_without_request_is_ignored() {
    let (mut player, handle) = human_player();
    handle.submit(0);
    assert_eq!(player.poll(), None);

    // Le choix ignoré ne doit pas resservir à la demande suivante.
    player.request(&some_view(), &[some_action()]);
    assert_eq!(player.poll(), None);
}

// Spec 002, L2.
#[test]
fn second_choice_for_same_request_is_ignored() {
    let (mut player, handle) = human_player();
    player.request(&some_view(), &[some_action(), some_action()]);
    handle.submit(1);
    handle.submit(0);

    assert_eq!(player.poll(), Some(1));
    assert_eq!(player.poll(), None);
}

// Spec 002, L3.
#[test]
fn out_of_range_choice_is_ignored() {
    let (mut player, handle) = human_player();
    player.request(&some_view(), &[some_action()]);
    handle.submit(1);

    assert_eq!(player.poll(), None);
    assert!(handle.pending().is_some(), "la demande reste ouverte");
    handle.submit(0);
    assert_eq!(player.poll(), Some(0));
}

// --- Boucle de partie ---

// Spec 002, C5.
#[test]
fn runner_with_human_waits_then_advances() {
    let (player, handle) = human_player();
    let mut runner = GameRunner::new(config(), vec![Box::new(player)]);
    let log_len = runner.log().len();

    assert_eq!(runner.step(), Ok(StepOutcome::Waiting { player: SOLO }));
    assert_eq!(runner.step(), Ok(StepOutcome::Waiting { player: SOLO }));
    assert_eq!(runner.log().len(), log_len, "rien n'est journalisé");

    handle.submit(0);
    assert_eq!(runner.step(), Ok(StepOutcome::Advanced));
    // Spec 013 : la première action légale joue une carte des 5 de la main de départ.
    assert_eq!(runner.state().play_area(SOLO).len(), 1);
}

// Spec 002, C6.
#[test]
fn frame_loop_stops_on_waiting_and_is_bounded() {
    let (player, handle) = human_player();
    let mut runner = GameRunner::new(config(), vec![Box::new(player)]);
    let report = run_frame(&mut runner);
    assert_eq!(report.outcome, FrameOutcome::Waiting);
    assert_eq!(report.steps, 0);

    handle.submit(0);
    let report = run_frame(&mut runner);
    assert_eq!(report.outcome, FrameOutcome::Waiting);
    assert_eq!(report.steps, 1);

    // Un joueur automatique qui répond toujours : la limite par image s'applique.
    // La limite se vérifie avec une limite de 2. Spec 023 : la partie dure 4 manches, plus de
    // 100 pas ; elle finit en quelques images, chacune bornée.
    assert_eq!(MAX_STEPS_PER_FRAME, 100);
    let mut runner = GameRunner::new(config(), vec![Box::new(FirstChoice::default())]);
    let report = run_frame_limited(&mut runner, 2);
    assert_eq!(report.outcome, FrameOutcome::StepLimitReached);
    assert_eq!(report.steps, 2);
    let mut frames = 0;
    loop {
        let report = run_frame(&mut runner);
        assert!(report.steps <= MAX_STEPS_PER_FRAME, "{report:?}");
        if report.outcome == FrameOutcome::Finished {
            break;
        }
        assert_eq!(report.outcome, FrameOutcome::StepLimitReached);
        frames += 1;
        assert!(frames < 100, "la partie doit finir");
    }
}

// Spec 002, L4.
#[test]
fn runner_error_stops_the_game_without_panic() {
    // Aucun joueur branché : `step()` renvoie une erreur.
    let (_, handle) = human_player();
    let runner = GameRunner::new(config(), vec![]);
    assert!(matches!(
        run_frame(&mut GameRunner::new(config(), vec![])).outcome,
        FrameOutcome::Error(_)
    ));

    let mut session = GameSession::with_runner(runner, handle);
    session.run_frame();
    session.run_frame();
    let screen = session.screen(&french());
    assert!(all_buttons(&screen).is_empty());
    assert_eq!(
        screen.message.as_deref(),
        Some("La partie s'est arrêtée sur une erreur.")
    );
}

// --- Écran de partie ---

// Spec 002, C7 (spec 013) : au départ, la main de 5 cartes et leurs boutons.
#[test]
fn screen_at_start_shows_hand_and_play_buttons() {
    let mut session = started_session(first_reconnaissance_config(42));
    session.run_frame();
    let screen = session.screen(&french());

    // Spec #33 C11 (D13) : « (prototype) » retiré du titre.
    assert_eq!(screen.title, "Iter Arcanum");
    assert_eq!(screen.hand_title, "Main");
    assert_eq!(screen.hand.len(), 5);
    assert_eq!(screen.hand_empty, None);
    assert_eq!(screen.deck, "Deck : 11 carte(s)");
    assert_eq!(screen.points.fallback_text(), "Mouvement 0 · Influence 0");
    assert!(screen.hand.iter().all(|card| card.sideways.len() == 2));
    assert_eq!(screen.message, None);
}

// Spec 002, C8 (spec 013) : la main dans l'ordre de la pioche.
#[test]
fn screen_lists_hand_in_draw_order() {
    let mut session = GameSession::new(first_reconnaissance_config(42));
    session.run_frame();
    let catalog = french();
    let screen = session.screen(&catalog);

    let runner = GameRunner::new(
        first_reconnaissance_config(42),
        vec![Box::new(FirstChoice::default())],
    );
    let expected: Vec<String> = runner
        .state()
        .hand(SOLO)
        .iter()
        .map(|card| catalog.text(card.card().label_key()))
        .collect();
    let names: Vec<String> = screen.hand.iter().map(|card| card.name.clone()).collect();
    assert_eq!(names, expected);
}

// Spec 002, C9 (spec 013) : main vide, plus de bouton, fin annoncée ; spec 016 : il faut
// finir les tours jusqu'à épuiser le deck ; spec 023 : puis annoncer la fin de chacune des
// 4 manches du bac à sable.
#[test]
fn screen_at_end_has_no_button_and_announces_end() {
    let mut session = GameSession::new(first_reconnaissance_config(42));
    for _ in 0..1_000 {
        session.run_frame();
        if past_tactic_choice(&mut session) {
            continue;
        }
        let catalog = french();
        let screen = session.screen(&catalog);
        if screen.message.is_some() {
            break;
        }
        match (
            announce_button(&screen, &catalog).and_then(|button| button.command),
            screen.hand.is_empty(),
        ) {
            // Deck et main vides : l'annonce est obligatoire et part sans confirmation.
            (Some(command), true) => session.command(command),
            (_, true) => {
                session.command(Command::OpenEndTurn);
                session.command(Command::ConfirmEndTurn);
            }
            (_, false) => session.command(first_sideways_influence(&screen)),
        }
    }
    let catalog = french();
    let screen = session.screen(&catalog);

    assert!(all_buttons(&screen).is_empty());
    assert_eq!(end_turn_button(&screen), None);
    assert_eq!(announce_button(&screen, &catalog), None);
    assert_eq!(
        screen.message.as_deref(),
        Some("Partie terminée : la dernière manche est jouée.")
    );
    assert!(screen.hand.is_empty());
    assert_eq!(screen.deck, "Deck : 0 carte(s)");
    assert_eq!(screen.discard_button.label, "Défausse : 16 carte(s)");
    assert_eq!(screen.points.fallback_text(), "Mouvement 0 · Influence 0");
}

// Spec 002, C10 (spec 013) : chaque bouton désigne son action parmi les actions légales.
#[test]
fn screen_buttons_match_legal_actions() {
    let mut session = started_session(first_reconnaissance_config(42));
    session.run_frame();
    let view = session.view();
    let legal = session.legal_actions();
    let screen = game_screen(
        &view,
        ScreenPhase::Playing {
            legal_actions: &legal,
        },
        &ScreenMode::Normal,
        &french(),
    );
    for card in &screen.hand {
        let commands = card
            .plays
            .iter()
            .map(|button| button.command)
            .chain(card.sideways.iter().map(|button| button.command));
        for command in commands {
            if let Command::Play(index) = command {
                // Spec 014 : un bouton de carte ne mène jamais à un déplacement.
                let Action::PlayCard { card: played, .. } = legal[index] else {
                    panic!("{command:?} doit jouer une carte");
                };
                assert_eq!(played, card.card, "{command:?}");
            }
        }
    }
    let sideways: usize = screen.hand.iter().map(|card| card.sideways.len()).sum();
    assert_eq!(sideways, 10);
}

// Spec 002, C11.
#[test]
fn screen_does_not_reveal_deck_order() {
    let state_with = |order: Vec<CardId>| {
        [
            Event::GameStarted { config: config() },
            Event::DeckShuffled {
                player: SOLO,
                order,
            },
        ]
        .iter()
        .fold(GameState::default(), apply)
    };
    let (march, rage, swiftness) = (
        card(BasicAction::March, 1),
        card(BasicAction::Rage, 1),
        card(BasicAction::Swiftness, 1),
    );
    let first = state_with(vec![march, rage, swiftness]);
    let second = state_with(vec![swiftness, march, rage]);
    assert_ne!(first, second);

    let catalog = french();
    let screen = |state: &GameState| {
        game_screen(
            &view(state, SOLO),
            ScreenPhase::Playing { legal_actions: &[] },
            &ScreenMode::Normal,
            &catalog,
        )
    };
    assert_eq!(screen(&first), screen(&second));
}

// Spec 002, C12.
#[test]
fn every_ui_key_exists_in_french() {
    let catalog = french();
    let mut session = started_session(first_reconnaissance_config(1));
    for _ in 0..5 {
        session.run_frame();
        let command = first_sideways(&session.screen(&catalog));
        session.command(command);
    }
    session.run_frame();
    session.screen(&catalog);

    // L'écran d'arrêt sur erreur aussi.
    game_screen(
        &some_view(),
        ScreenPhase::Stopped,
        &ScreenMode::Normal,
        &catalog,
    );

    assert_eq!(catalog.gaps(), vec![]);
}

// --- Dossier des ressources ---

// Spec 002, C13.
#[test]
fn assets_dir_is_the_first_candidate_with_french() {
    let candidates = [fixture("absent"), fixture("premier"), fixture("second")];
    assert_eq!(find_assets_dir(&candidates), Ok(fixture("premier")));
    let catalog = load_catalog(&candidates, &french_language()).expect("catalogue du premier");
    assert_eq!(catalog.text("language.name"), "Premier");
}

// Spec 002, C14 ; spec #33, C9 : candidat du paquet `.app` après celui de l'exécutable.
#[test]
fn assets_candidates_are_in_priority_order() {
    let env = PathBuf::from("depuis-env");
    let exe_dir = PathBuf::from("bin");
    let bundle = exe_dir.join("..").join("Resources").join("assets");
    assert_eq!(
        assets_candidates(Some(env.clone()), Some(exe_dir.clone())),
        vec![
            env,
            exe_dir.join("assets"),
            bundle.clone(),
            repository_assets_dir()
        ]
    );
    assert_eq!(
        assets_candidates(None, Some(exe_dir.clone())),
        vec![exe_dir.join("assets"), bundle, repository_assets_dir()]
    );
}

// Spec #33, C10 : `Iter Arcanum.app/Contents/MacOS/iter-arcanum` trouve `Contents/Resources/assets`.
#[test]
fn assets_are_found_in_app_bundle_resources() {
    let root = std::env::temp_dir().join(format!("arcanum-app-{}", std::process::id()));
    let contents = root.join("Iter Arcanum.app").join("Contents");
    let macos = contents.join("MacOS");
    let i18n = contents.join("Resources").join("assets").join("i18n");
    std::fs::create_dir_all(&macos).expect("dossier MacOS");
    std::fs::create_dir_all(&i18n).expect("dossier i18n");
    std::fs::write(i18n.join("fr.json"), "{}").expect("fr.json");

    let found = find_assets_dir(&assets_candidates(None, Some(macos.clone())));

    let _ = std::fs::remove_dir_all(&root);
    assert_eq!(found, Ok(macos.join("..").join("Resources").join("assets")));
}

// Spec 002, C15.
#[test]
fn repository_assets_dir_is_found() {
    let candidates = assets_candidates(None, None);
    assert_eq!(find_assets_dir(&candidates), Ok(repository_assets_dir()));
    let catalog = load_catalog(&candidates, &french_language()).expect("fr.json du dépôt");
    assert_eq!(catalog.text("language.name"), "Français");
}

// Spec 002, L5.
#[test]
fn missing_assets_dir_falls_back_to_visible_keys() {
    let candidates = [fixture("absent"), fixture("introuvable")];
    assert_eq!(
        find_assets_dir(&candidates),
        Err(AssetsError::NotFound {
            tried: candidates.to_vec()
        })
    );
    let catalog = catalog_or_empty(&candidates, &french_language());
    assert_eq!(catalog.text("app.title"), "⟦app.title⟧");
}

// Spec 002, L6.
#[test]
fn unloadable_catalog_falls_back_to_visible_keys() {
    let candidates = [fixture("invalide")];
    assert!(matches!(
        load_catalog(&candidates, &french_language()),
        Err(AssetsError::Catalog(_))
    ));
    let catalog = catalog_or_empty(&candidates, &french_language());
    assert_eq!(catalog.text("app.title"), "⟦app.title⟧");
}

// Spec 002, L7.
#[test]
fn invalid_assets_env_var_is_skipped() {
    let candidates = assets_candidates(Some(fixture("absent")), None);
    assert_eq!(find_assets_dir(&candidates), Ok(repository_assets_dir()));
}

// Issue privée 124 : arcanum-i18n n'accorde pas, un nombre suivi d'un mot au pluriel en « s » s'écrit « mot(s) »,
// en français comme en anglais (« Deck : 1 carte(s) », pas « 1 cartes »).
#[test]
fn counted_labels_use_optional_plural_in_both_languages() {
    let catalog = |language: &str| {
        Catalog::load(
            &repository_assets_dir().join("i18n"),
            &Language(language.into()),
        )
        .expect("catalogue du dépôt")
    };
    let one = [("count", "1")];
    let route = [("steps", "1"), ("cost", "2")];
    let french = catalog("fr");
    assert_eq!(french.format("deck.count", &one), "Deck : 1 carte(s)");
    assert_eq!(
        french.format("discard.count", &one),
        "Défausse : 1 carte(s)"
    );
    assert_eq!(
        french.format("board.hover.route", &route),
        "Trajet : 1 case(s), 2 points de mouvement"
    );
    let english = catalog("en");
    assert_eq!(english.format("deck.count", &one), "Deck: 1 card(s)");
    assert_eq!(
        english.format("discard.count", &one),
        "Discard pile: 1 card(s)"
    );
    assert_eq!(
        english.format("board.hover.route", &route),
        "Route: 1 space(s), 2 Move points"
    );
}
