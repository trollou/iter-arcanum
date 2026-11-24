//! Scénarios de la spec 016 côté affichage (numéro du tour, mode fin du tour, raccourcis).

use arcanum_core::{
    Action, BasicAction, CardId, CardPlay, DeedCard, Event, GameConfig, GameState, Hero, PlayerId,
    Scenario, SidewaysEffect, apply, decide, legal_actions, view,
};
use arcanum_i18n::{Catalog, Language};
use arcanum_ui::{
    Command, EndTurnChoice, GameScreen, GameSession, ScreenMode, ScreenPhase, Shortcut,
    game_screen, hover_text, repository_assets_dir,
};

mod support;
use support::{end_turn_button, first_reconnaissance_config, started_session};

const SOLO: PlayerId = PlayerId::SOLO;

use BasicAction::*;

fn french() -> Catalog {
    Catalog::load(
        &repository_assets_dir().join("i18n"),
        &Language("fr".into()),
    )
    .expect("fr.json du dépôt")
}

fn card(action: BasicAction, copy: u16) -> CardId {
    CardId::new(DeedCard::Basic(action), copy)
}

fn config() -> GameConfig {
    GameConfig {
        seed: 0,
        scenario: Scenario::sandbox(),
        hero: Hero::Arythea,
        dummy_hero: Default::default(),
    }
}

/// Partie dont la main est exactement `hand` et le deck `deck` derrière.
fn with_hand_and_deck(hand: &[CardId], deck: &[CardId]) -> GameState {
    let mut order = hand.to_vec();
    order.extend(deck);
    let mut events = vec![
        Event::GameStarted { config: config() },
        Event::DeckShuffled {
            player: SOLO,
            order,
        },
    ];
    events.extend(
        hand.iter()
            .map(|&card| Event::CardDrawn { player: SOLO, card }),
    );
    events.iter().fold(GameState::default(), apply)
}

/// Le reste du deck d'Arythea, sans les cartes de `hand`.
fn rest_of_deck(hand: &[CardId]) -> Vec<CardId> {
    Hero::Arythea
        .starting_deck()
        .into_iter()
        .filter(|card| !hand.contains(card))
        .collect()
}

fn hand() -> [CardId; 5] {
    [
        card(March, 1),
        card(Rage, 1),
        card(Promise, 1),
        card(Threaten, 1),
        card(Stamina, 1),
    ]
}

fn sideways_influence(state: &GameState, held: CardId) -> GameState {
    let action = Action::PlayCard {
        card: held,
        play: CardPlay::Sideways(SidewaysEffect::Influence),
    };
    decide(state, SOLO, &action, &mut config().rngs())
        .expect("légal")
        .iter()
        .fold(state.clone(), apply)
}

fn screen_of(state: &GameState, mode: &ScreenMode, catalog: &Catalog) -> GameScreen {
    let legal = legal_actions(state, SOLO);
    game_screen(
        &view(state, SOLO),
        ScreenPhase::Playing {
            legal_actions: &legal,
        },
        mode,
        catalog,
    )
}

fn ending(discard: &[CardId]) -> ScreenMode {
    ScreenMode::EndingTurn(EndTurnChoice {
        discard: discard.to_vec(),
        reclaim: None,
        glade: None,
        steady_tempo: None,
    })
}

fn session() -> GameSession {
    let mut session = started_session(first_reconnaissance_config(42));
    session.run_frame();
    session
}

/// Joue la première carte de la main de côté pour Influence, par son bouton.
fn play_first_card(session: &mut GameSession, catalog: &Catalog) {
    let command = session.screen(catalog).hand[0].sideways[1].command;
    session.command(command);
    session.run_frame();
}

// Spec 016 C15
#[test]
fn panel_shows_turn_number() {
    let catalog = french();
    let mut session = session();
    // Spec 074, C5 : la ligne est dans la barre de menu.
    assert_eq!(session.menu_bar(&catalog).round, "Manche 1/4 (tour 1)");

    play_first_card(&mut session, &catalog);
    session.command(Command::OpenEndTurn);
    session.command(Command::ConfirmEndTurn);
    session.run_frame();

    assert_eq!(session.menu_bar(&catalog).round, "Manche 1/4 (tour 2)");
}

// Spec 016 C16
#[test]
fn end_turn_button_follows_legal_actions() {
    let catalog = french();
    let state = with_hand_and_deck(&hand(), &rest_of_deck(&hand()));
    let screen = screen_of(&state, &ScreenMode::Normal, &catalog);
    let button = end_turn_button(&screen).expect("bouton de fin du tour");
    assert_eq!(button.label, "Finir le tour…");
    assert_eq!(button.command, Some(Command::OpenEndTurn));

    let finished = game_screen(
        &view(&state, SOLO),
        ScreenPhase::Finished,
        &ScreenMode::Normal,
        &catalog,
    );
    assert_eq!(end_turn_button(&finished), None);
}

// Spec 016 C17
#[test]
fn end_turn_mode_lists_cards_to_discard() {
    let catalog = french();
    let [march, rage, promise, ..] = hand();
    let state = with_hand_and_deck(&hand(), &rest_of_deck(&hand()));
    let state = sideways_influence(&state, march);
    let state = sideways_influence(&state, rage);
    let state = sideways_influence(&state, promise);

    let screen = screen_of(&state, &ending(&[]), &catalog);

    let prompt = screen.end_turn_prompt.expect("bandeau de fin du tour");
    assert_eq!(
        prompt.text,
        "Fin du tour : choisissez les cartes à défausser avant de repiocher."
    );
    assert_eq!(prompt.draw, "Vous piocherez 3 carte(s).");
    assert_eq!(prompt.must_discard, None);
    assert_eq!(prompt.confirm.label, "Finir le tour");
    assert!(prompt.confirm_enabled);
    assert_eq!(prompt.cancel.label, "Annuler");
    // En bas : « Annuler » et « Finir le tour », plus « Finir le tour… » (spec 073, C34).
    assert!(
        screen
            .bottom_buttons
            .iter()
            .all(|button| button.label != "Finir le tour…")
    );
    assert_eq!(screen.hand.len(), 2);
    for held in &screen.hand {
        assert!(held.plays.is_empty());
        assert!(held.sideways.is_empty());
        let toggle = held.discard_toggle.as_ref().expect("case à cocher");
        assert_eq!(toggle.label, "Défausser");
        assert!(!toggle.checked);
        assert_eq!(toggle.command, Command::ToggleEndTurnDiscard(held.card));
    }
}

// Spec 016 C17 : le plateau de jeu ne propose plus de déplacement en mode fin du tour.
#[test]
fn end_turn_mode_hides_board_moves() {
    let catalog = french();
    let mut session = session();
    assert!(session.board_shows_moves());
    play_first_card(&mut session, &catalog);
    session.command(Command::OpenEndTurn);
    assert!(session.is_ending_turn());
    assert!(!session.board_shows_moves());
}

// Spec 016 C18
#[test]
fn end_turn_prompt_counts_cards_to_draw() {
    let catalog = french();
    let [march, rage, promise, threaten, _] = hand();
    let state = with_hand_and_deck(&hand(), &rest_of_deck(&hand()));
    let state = sideways_influence(&state, march);

    let screen = screen_of(&state, &ending(&[threaten]), &catalog);
    let prompt = screen.end_turn_prompt.expect("bandeau");
    assert_eq!(prompt.draw, "Vous piocherez 2 carte(s).");
    let checked: Vec<bool> = screen
        .hand
        .iter()
        .map(|held| held.discard_toggle.as_ref().is_some_and(|t| t.checked))
        .collect();
    assert_eq!(checked, [false, false, true, false]);

    let screen = screen_of(&state, &ending(&[rage, promise, threaten]), &catalog);
    assert_eq!(
        screen.end_turn_prompt.expect("bandeau").draw,
        "Vous piocherez 4 carte(s)."
    );

    // Deck vide : rien à piocher.
    let empty_deck = sideways_influence(&with_hand_and_deck(&hand(), &[]), march);
    let screen = screen_of(&empty_deck, &ending(&[rage]), &catalog);
    assert_eq!(
        screen.end_turn_prompt.expect("bandeau").draw,
        "Vous ne piocherez aucune carte."
    );
}

// Spec 016 C19
#[test]
fn end_turn_needs_a_discard_when_nothing_was_played() {
    let catalog = french();
    let [march, ..] = hand();
    let state = with_hand_and_deck(&hand(), &rest_of_deck(&hand()));

    let prompt = screen_of(&state, &ending(&[]), &catalog)
        .end_turn_prompt
        .expect("bandeau");
    assert!(!prompt.confirm_enabled);
    assert_eq!(
        prompt.must_discard.as_deref(),
        Some("Vous n'avez joué aucune carte ce tour : défaussez-en au moins une.")
    );

    let prompt = screen_of(&state, &ending(&[march]), &catalog)
        .end_turn_prompt
        .expect("bandeau");
    assert!(prompt.confirm_enabled);
    assert_eq!(prompt.must_discard, None);
}

// Spec 016 C20
#[test]
fn confirming_end_turn_sends_the_action() {
    let catalog = french();
    let mut session = session();
    let played = session.view().hand[0];
    play_first_card(&mut session, &catalog);
    let hand = session.view().hand;

    session.command(Command::OpenEndTurn);
    // Cochées dans le désordre : l'action les donne dans l'ordre de la main.
    session.command(Command::ToggleEndTurnDiscard(hand[2]));
    session.command(Command::ToggleEndTurnDiscard(hand[0]));
    session.command(Command::ToggleEndTurnDiscard(hand[1]));
    session.command(Command::ToggleEndTurnDiscard(hand[1]));
    session.command(Command::ConfirmEndTurn);
    session.run_frame();

    assert!(!session.is_ending_turn());
    let seen = session.view();
    assert_eq!(seen.discard, vec![played, hand[0], hand[2]]);
    assert_eq!(seen.hand.len(), 5);
    assert_eq!(seen.turn, 2);
}

// Spec 016 C21
#[test]
fn end_turn_mode_can_be_cancelled() {
    let catalog = french();
    let mut session = session();
    play_first_card(&mut session, &catalog);
    let before = session.view();

    session.command(Command::OpenEndTurn);
    session.command(Command::ToggleEndTurnDiscard(before.hand[0]));
    session.command(Command::CancelChoice);
    session.run_frame();
    assert!(!session.is_ending_turn());
    assert_eq!(session.view(), before);
    assert_eq!(session.screen(&catalog).end_turn_prompt, None);

    // Échap fait de même.
    session.command(Command::OpenEndTurn);
    session.shortcut(Shortcut::Cancel);
    session.run_frame();
    assert!(!session.is_ending_turn());
    assert_eq!(session.view(), before);
}

/// Une partie d'Arythea dont la main de départ contient Improvisation.
fn session_with_improvisation() -> GameSession {
    for seed in 0..500 {
        let mut session = started_session(first_reconnaissance_config(seed));
        session.run_frame();
        if session
            .view()
            .hand
            .iter()
            .any(|held| held.card() == DeedCard::Basic(Improvisation))
        {
            return session;
        }
    }
    panic!("aucune graine ne donne Improvisation");
}

// Spec 016 C22
#[test]
fn end_turn_keyboard_shortcuts() {
    let catalog = french();
    let mut session = session();

    // Entrée hors du mode fin du tour : rien.
    let before = session.view();
    session.shortcut(Shortcut::Confirm);
    session.run_frame();
    assert_eq!(session.view(), before);

    // Fin ouvre le mode ; rien de joué ni de coché, Entrée ne fait rien.
    session.shortcut(Shortcut::EndTurn);
    assert!(session.is_ending_turn());
    session.shortcut(Shortcut::Confirm);
    session.run_frame();
    assert_eq!(session.view(), before);

    // Une carte cochée : Entrée finit le tour.
    session.command(Command::ToggleEndTurnDiscard(before.hand[0]));
    session.shortcut(Shortcut::Confirm);
    session.run_frame();
    assert!(!session.is_ending_turn());
    assert_eq!(session.view().turn, 2);
    assert_eq!(session.view().discard, vec![before.hand[0]]);

    // Pendant le choix de la carte à défausser d'Improvisation, Fin ne fait rien.
    let mut session = session_with_improvisation();
    let screen = session.screen(&catalog);
    let improvisation = screen
        .hand
        .iter()
        .find(|held| held.name == "Improvisation")
        .expect("Improvisation en main");
    session.command(improvisation.plays[0].command);
    assert!(session.is_choosing_discard());
    session.shortcut(Shortcut::EndTurn);
    assert!(!session.is_ending_turn());
    assert!(session.is_choosing_discard());
}

// Spec 016 C23
#[test]
fn every_end_turn_key_exists_in_french() {
    let catalog = french();
    let [march, rage, promise, ..] = hand();
    let state = with_hand_and_deck(&hand(), &rest_of_deck(&hand()));
    screen_of(&state, &ScreenMode::Normal, &catalog);
    let played = sideways_influence(&state, march);
    for discard in [vec![], vec![rage], vec![rage, promise]] {
        screen_of(&state, &ending(&discard), &catalog);
        screen_of(&played, &ending(&discard), &catalog);
    }
    let empty_deck = sideways_influence(&with_hand_and_deck(&hand(), &[]), march);
    screen_of(&empty_deck, &ending(&[rage]), &catalog);

    let mut session = session();
    play_first_card(&mut session, &catalog);
    session.command(Command::OpenEndTurn);
    session.command(Command::ConfirmEndTurn);
    session.run_frame();
    session.screen(&catalog);

    assert_eq!(catalog.gaps(), vec![]);
}

/// Joue toute la main de côté : il ne reste que la zone de jeu.
fn session_with_empty_hand(catalog: &Catalog) -> GameSession {
    let mut session = session();
    while !session.view().hand.is_empty() {
        play_first_card(&mut session, catalog);
    }
    session
}

// Spec 016 C24 (issue privée 25) : main vide, rien à défausser (MKUE p. 6, 5.a), le tour se finit directement.
#[test]
fn end_turn_with_empty_hand_is_immediate() {
    let catalog = french();
    let mut session = session_with_empty_hand(&catalog);

    let screen = session.screen(&catalog);
    let button = end_turn_button(&screen).expect("bouton de fin du tour");
    assert_eq!(button.label, "Finir le tour");

    session.command(button.command.expect("actif"));
    assert!(!session.is_ending_turn());
    session.run_frame();

    let seen = session.view();
    assert_eq!(seen.turn, 2);
    assert_eq!(seen.hand.len(), 5);
    assert!(seen.play_area.is_empty());
    assert_eq!(session.screen(&catalog).end_turn_prompt, None);
}

// Spec 016 C25 (issue privée 25)
#[test]
fn end_turn_key_with_empty_hand_is_immediate() {
    let catalog = french();
    let mut session = session_with_empty_hand(&catalog);

    session.shortcut(Shortcut::EndTurn);
    assert!(!session.is_ending_turn());
    session.run_frame();

    assert_eq!(session.view().turn, 2);
    assert_eq!(session.view().hand.len(), 5);
}

// Spec 071, C19 : « Finir le tour… » dit Fin ; en mode fin du tour, Entrée et Échap.
#[test]
fn end_turn_buttons_show_shortcut() {
    let catalog = french();
    let state = with_hand_and_deck(&hand(), &rest_of_deck(&hand()));
    let screen = screen_of(&state, &ScreenMode::Normal, &catalog);
    let button = end_turn_button(&screen).expect("bouton de fin du tour");
    assert_eq!(
        hover_text(None, button.shortcut.as_deref(), &catalog).as_deref(),
        Some("Raccourci : Fin")
    );
    let state = sideways_influence(&state, hand()[0]);
    let screen = screen_of(&state, &ending(&[]), &catalog);
    let prompt = screen.end_turn_prompt.expect("mode fin du tour");
    assert_eq!(prompt.confirm.label, "Finir le tour");
    assert_eq!(prompt.confirm.shortcut.as_deref(), Some("Entrée"));
    assert_eq!(prompt.cancel.shortcut.as_deref(), Some("Échap"));
}
