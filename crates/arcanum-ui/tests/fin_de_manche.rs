//! Scénarios de la spec 023 côté affichage (ligne de la manche, bandeau d'annonce, confirmation,
//! nouvelle manche, voile de nuit, fin de partie).

use std::fs;

use arcanum_core::{
    Action, BasicColor, CardId, Event, GameConfig, GameState, Hero, ManaColor, PlayerId, Scenario,
    TimeOfDay, apply, decide, legal_actions, view,
};
use arcanum_i18n::{Catalog, Language};
use arcanum_ui::{
    Command, GameScreen, GameSession, ImageId, ScreenMode, ScreenPhase, Shortcut, game_screen,
    night_veil, repository_assets_dir, time_of_day_image,
};

mod support;
use support::{
    announce_button, bar_of, end_turn_button, first_reconnaissance_config, labels_and_commands,
    past_tactic_choice, started_session,
};

const SOLO: PlayerId = PlayerId::SOLO;

fn french() -> Catalog {
    Catalog::load(
        &repository_assets_dir().join("i18n"),
        &Language("fr".into()),
    )
    .expect("fr.json du dépôt")
}

fn english() -> Catalog {
    Catalog::load(
        &repository_assets_dir().join("i18n"),
        &Language("en".into()),
    )
    .expect("en.json du dépôt")
}

fn config() -> GameConfig {
    GameConfig {
        seed: 0,
        scenario: Scenario::sandbox(),
        hero: Hero::Arythea,
        dummy_hero: Default::default(),
    }
}

/// Partie construite par événements : deck d'Arythea entièrement pioché, les `kept` premières
/// cartes gardées en main, les autres défaussées. Le deck est vide, le tour pas commencé.
fn exhausted_with(kept: usize) -> GameState {
    let order = Hero::Arythea.starting_deck();
    let mut events = vec![
        Event::GameStarted { config: config() },
        Event::DeckShuffled {
            player: SOLO,
            order: order.clone(),
        },
        Event::SourceRolled {
            dice: vec![
                ManaColor::Basic(BasicColor::Green),
                ManaColor::Gold,
                ManaColor::Black,
            ],
        },
    ];
    events.extend(
        order
            .iter()
            .map(|&card| Event::CardDrawn { player: SOLO, card }),
    );
    events.extend(
        order
            .iter()
            .skip(kept)
            .map(|&card| Event::CardDiscarded { player: SOLO, card }),
    );
    events.iter().fold(GameState::default(), apply)
}

fn screen_in(state: &GameState, mode: &ScreenMode, catalog: &Catalog) -> GameScreen {
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

fn screen_of(state: &GameState, catalog: &Catalog) -> GameScreen {
    screen_in(state, &ScreenMode::Normal, catalog)
}

fn announce_index(state: &GameState) -> usize {
    legal_actions(state, SOLO)
        .iter()
        .position(|action| *action == Action::AnnounceEndOfRound)
        .expect("annonce légale")
}

fn act(state: &GameState, action: &Action) -> GameState {
    decide(state, SOLO, action, &mut config().rngs())
        .expect("légal")
        .iter()
        .fold(state.clone(), apply)
}

fn announce(state: &GameState) -> GameState {
    act(state, &Action::AnnounceEndOfRound)
}

/// Pioche tout le deck puis défausse toute la main : deck et main vides.
fn exhaust(state: GameState) -> GameState {
    let mut state = state;
    let deck: Vec<CardId> = state.deck(SOLO).to_vec();
    for card in deck {
        state = apply(state, &Event::CardDrawn { player: SOLO, card });
    }
    let hand: Vec<CardId> = state.hand(SOLO).to_vec();
    for card in hand {
        state = apply(state, &Event::CardDiscarded { player: SOLO, card });
    }
    state
}

fn session() -> GameSession {
    let mut session = started_session(first_reconnaissance_config(42));
    session.run_frame();
    session
}

/// Joue la première carte de côté pour Influence puis finit le tour sans défausser.
fn play_one_turn(session: &mut GameSession, catalog: &Catalog) {
    let command = session.screen(catalog).hand[0].sideways[1].command;
    session.command(command);
    session.run_frame();
    session.command(Command::OpenEndTurn);
    session.command(Command::ConfirmEndTurn);
    session.run_frame();
}

/// Joue des tours jusqu'à ce que « Finir la manche… » apparaisse.
fn until_announce(session: &mut GameSession, catalog: &Catalog) -> GameScreen {
    for _ in 0..100 {
        let screen = session.screen(catalog);
        if announce_button(&screen, catalog).is_some() {
            return screen;
        }
        play_one_turn(session, catalog);
    }
    panic!("le bandeau d'annonce n'est jamais apparu");
}

// Spec 023 C20 ; spec 074, C5 : dans la barre de menu, sans le mot « Jour » (remplace
// `panel_shows_round_number_and_time_icon`).
#[test]
fn bar_shows_round_and_turn() {
    let catalog = french();
    let state = exhausted_with(0);
    let bar = bar_of(&view(&state, SOLO), &catalog);
    assert_eq!(bar.round, "Manche 1/4 (tour 1)");
    assert_eq!(bar.clock.time_of_day, TimeOfDay::Day);
    assert_eq!(bar.clock.tooltip, "Jour");

    // MKUE p. 4, 2.a : la manche suivante est de nuit.
    let bar = bar_of(&view(&announce(&state), SOLO), &catalog);
    assert_eq!(bar.round, "Manche 2/4 (tour 1)");
    assert_eq!(bar.clock.time_of_day, TimeOfDay::Night);
    assert_eq!(bar.clock.tooltip, "Nuit");
}

// Spec 023 C20 : les icônes sont des SVG du dépôt.
#[test]
fn time_of_day_icons_exist() {
    let images_dir = repository_assets_dir().join("images");
    assert_eq!(
        time_of_day_image(TimeOfDay::Day),
        ImageId::new("time_of_day", "day")
    );
    assert_eq!(
        time_of_day_image(TimeOfDay::Night),
        ImageId::new("time_of_day", "night")
    );
    for time in [TimeOfDay::Day, TimeOfDay::Night] {
        let path = time_of_day_image(time).path(&images_dir);
        let text = fs::read_to_string(&path).unwrap_or_else(|_| panic!("{path:?} manque"));
        assert!(text.starts_with("<svg"), "{path:?} n'est pas un SVG");
    }
}

// Spec 023 C21
#[test]
fn announce_banner_when_deck_is_empty() {
    // MKUE p. 6, 3.a : deck vide au début du tour, l'annonce est possible.
    let catalog = french();
    let screen = screen_of(&exhausted_with(2), &catalog);
    // Spec 073, C33 : « Finir la manche… » en bas du panneau, le texte du bandeau d'avant en
    // infobulle (issue privée 259 : ce qui est dessiné).
    let button = announce_button(&screen, &catalog).expect("Finir la manche…");
    assert_eq!(
        button.tooltip.as_deref(),
        Some(
            "Votre deck est vide : vous pouvez annoncer la fin de la manche au lieu de jouer ce tour."
        )
    );
    assert_eq!(button.command, Some(Command::OpenAnnounce));
    // Ce n'est pas un mode : les cartes restent jouables.
    assert_eq!(screen.hand.len(), 2);
    assert!(screen.hand.iter().all(|card| !card.sideways.is_empty()));
    assert!(end_turn_button(&screen).is_some());
}

// Spec 023 C21 : pas de bandeau tant que le deck n'est pas vide.
#[test]
fn no_announce_banner_with_cards_in_deck() {
    let catalog = french();
    let session = session();
    assert_eq!(announce_button(&session.screen(&catalog), &catalog), None);
}

// Spec 023 C22
#[test]
fn announce_banner_when_it_is_mandatory() {
    // MKUE p. 6, 3.b : sans carte en main, il faut annoncer.
    let catalog = french();
    let state = exhausted_with(0);
    let screen = screen_of(&state, &catalog);
    let button = announce_button(&screen, &catalog).expect("Finir la manche…");
    assert_eq!(
        button.tooltip.as_deref(),
        Some("Votre deck et votre main sont vides : annoncez la fin de la manche.")
    );
    assert_eq!(end_turn_button(&screen), None);
    assert!(screen.hand.is_empty());
    // C23 bis : rien à confirmer, le bouton envoie l'annonce.
    assert_eq!(button.command, Some(Command::Play(announce_index(&state))));
}

// Spec 023 C23
#[test]
fn announcing_with_cards_asks_confirmation() {
    let catalog = french();
    let state = exhausted_with(2);
    let screen = screen_in(&state, &ScreenMode::ConfirmingEndOfRound, &catalog);
    // Spec 073, C33 : la confirmation, dans le bandeau du panneau du bas.
    let prompt = screen
        .round_end_confirm
        .as_ref()
        .expect("bandeau de confirmation");
    assert_eq!(
        prompt.text,
        "Annoncer la fin de la manche ? Vos 2 carte(s) en main seront remélangées sans être jouées."
    );
    assert_eq!(
        labels_and_commands(&prompt.buttons),
        [
            ("Confirmer", &Command::Play(announce_index(&state))),
            ("Annuler", &Command::CancelChoice),
        ]
    );
    // Spec 071, C23 : Échap annule.
    assert_eq!(prompt.buttons[1].shortcut.as_deref(), Some("Échap"));
    // Pendant la confirmation, rien d'autre ne se joue.
    assert!(screen.hand.iter().all(|card| card.plays.is_empty()
        && card.sideways.is_empty()
        && card.strong_plays.is_empty()));
    assert_eq!(end_turn_button(&screen), None);
}

// Spec 023 C23 : la session ouvre la confirmation, Échap l'annule, Confirmer annonce.
#[test]
fn session_confirms_the_announce() {
    let catalog = french();
    let mut session = session();
    let screen = until_announce(&mut session, &catalog);
    let open = announce_button(&screen, &catalog)
        .and_then(|button| button.command)
        .expect("Finir la manche…");
    assert_eq!(open, Command::OpenAnnounce);

    session.command(open);
    assert!(session.is_confirming_end_of_round());
    session.shortcut(Shortcut::Cancel);
    assert!(!session.is_confirming_end_of_round());
    assert_eq!(
        announce_button(&session.screen(&catalog), &catalog).and_then(|button| button.command),
        Some(Command::OpenAnnounce)
    );

    session.command(open);
    let confirm = session
        .screen(&catalog)
        .round_end_confirm
        .expect("confirmation")
        .buttons[0]
        .command;
    session.command(confirm);
    session.run_frame();
    // Spec 060 : la manche 2 s'ouvre sur le choix de la tactique.
    assert!(past_tactic_choice(&mut session));
    let screen = session.screen(&catalog);
    assert_eq!(session.menu_bar(&catalog).round, "Manche 2/4 (tour 1)");
    assert_eq!(announce_button(&screen, &catalog), None);
    assert_eq!(screen.hand.len(), 5);
}

// Spec 023 C23 bis
#[test]
fn mandatory_announce_needs_no_confirmation() {
    let catalog = french();
    let state = exhausted_with(0);
    let screen = screen_of(&state, &catalog);
    let command = announce_button(&screen, &catalog)
        .and_then(|button| button.command)
        .expect("Finir la manche…");
    // Ouvrir la confirmation n'a pas de sens ici : le bouton joue l'annonce.
    assert_eq!(command, Command::Play(announce_index(&state)));

    // Une carte jouée : le tour est commencé, plus de bandeau.
    let state = exhausted_with(1);
    let card = state.hand(SOLO)[0];
    let played = act(
        &state,
        &Action::PlayCard {
            card,
            play: arcanum_core::CardPlay::Sideways(arcanum_core::SidewaysEffect::Influence),
        },
    );
    assert_eq!(
        announce_button(&screen_of(&played, &catalog), &catalog),
        None
    );
}

// Spec 023 C24
#[test]
fn panel_announces_new_round() {
    let catalog = french();
    let state = exhausted_with(0);
    assert_eq!(screen_of(&state, &catalog).new_round, None);

    let state = announce(&state);
    assert_eq!(
        screen_of(&state, &catalog).new_round.as_deref(),
        Some("Nouvelle manche : la Source a été relancée et votre deck remélangé.")
    );

    // Dès qu'une carte est jouée, la ligne disparaît.
    let card: CardId = state.hand(SOLO)[0];
    let played = act(
        &state,
        &Action::PlayCard {
            card,
            play: arcanum_core::CardPlay::Sideways(arcanum_core::SidewaysEffect::Influence),
        },
    );
    assert_eq!(screen_of(&played, &catalog).new_round, None);
}

// Spec 023 C25
#[test]
fn board_is_veiled_at_night() {
    assert_eq!(night_veil(TimeOfDay::Day), None);
    let veil = night_veil(TimeOfDay::Night).expect("voile de nuit");
    // Semi-transparent : le plateau de jeu reste lisible.
    assert!(veil.a() > 0 && veil.a() < 255);
}

// Spec 023 C26
#[test]
fn screen_announces_last_round() {
    // MKUE p. 18 : le bac à sable dure 4 manches.
    let catalog = french();
    let mut session = GameSession::new(first_reconnaissance_config(42));
    let mut rounds = 0;
    for _ in 0..1_000 {
        session.run_frame();
        if past_tactic_choice(&mut session) {
            continue;
        }
        let screen = session.screen(&catalog);
        if screen.message.is_some() {
            break;
        }
        match (
            announce_button(&screen, &catalog).and_then(|button| button.command),
            screen.hand.is_empty(),
        ) {
            (Some(command), true) => {
                session.command(command);
                rounds += 1;
            }
            _ => {
                let command = screen.hand[0].sideways[1].command;
                session.command(command);
                session.run_frame();
                session.command(Command::OpenEndTurn);
                session.command(Command::ConfirmEndTurn);
            }
        }
    }
    let screen = session.screen(&catalog);
    assert_eq!(rounds, 4);
    assert_eq!(
        screen.message.as_deref(),
        Some("Partie terminée : la dernière manche est jouée.")
    );
    assert_eq!(announce_button(&screen, &catalog), None);
    assert_eq!(end_turn_button(&screen), None);
}

// Spec 023 C27
#[test]
fn every_round_key_exists_in_french() {
    let catalog = french();
    let english = english();
    let two_cards = exhausted_with(2);
    let empty = exhausted_with(0);
    let night = announce(&empty);
    let last = announce(&exhaust(announce(&exhaust(announce(&exhaust(
        night.clone(),
    ))))));
    for catalog in [&catalog, &english] {
        screen_of(&two_cards, catalog);
        screen_in(&two_cards, &ScreenMode::ConfirmingEndOfRound, catalog);
        screen_of(&empty, catalog);
        screen_of(&night, catalog);
        for state in [&night, &last] {
            game_screen(
                &view(state, SOLO),
                ScreenPhase::Finished,
                &ScreenMode::Normal,
                catalog,
            );
        }
    }
    assert_eq!(catalog.gaps(), vec![]);
    assert_eq!(english.gaps(), vec![]);
}
