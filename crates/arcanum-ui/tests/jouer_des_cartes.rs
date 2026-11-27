//! Scénarios de la spec 013 côté affichage (main, menu de côté, choix de la défausse, zone de jeu).

use arcanum_core::{
    Action, BasicAction, BasicColor, CardId, CardPlay, DeedCard, Event, GameConfig, GameState,
    Hero, PlayerId, Scenario, SidewaysEffect, apply, decide, legal_actions, view,
};
use arcanum_i18n::{Catalog, Language};
use arcanum_ui::{
    Button, Command, DiscardChoice, GameScreen, GameSession, HandCard, ScreenMode, ScreenPhase,
    game_screen, repository_assets_dir,
};

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

fn config(seed: u64, hero: Hero) -> GameConfig {
    GameConfig {
        seed,
        scenario: Scenario::sandbox(),
        hero,
        dummy_hero: Default::default(),
    }
}

/// Partie dont la main est exactement `hand`.
fn with_hand(hand: &[CardId]) -> GameState {
    let mut order = hand.to_vec();
    order.extend(
        Hero::Arythea
            .starting_deck()
            .into_iter()
            .filter(|card| !hand.contains(card)),
    );
    let mut events = vec![
        Event::GameStarted {
            config: config(0, Hero::Arythea),
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
    events.iter().fold(GameState::default(), apply)
}

fn play(state: &GameState, action: Action) -> GameState {
    decide(state, SOLO, &action, &mut config(0, Hero::Arythea).rngs())
        .expect("action légale")
        .iter()
        .fold(state.clone(), apply)
}

fn screen_of(state: &GameState, choice: Option<DiscardChoice>, catalog: &Catalog) -> GameScreen {
    let legal = legal_actions(state, SOLO);
    game_screen(
        &view(state, SOLO),
        ScreenPhase::Playing {
            legal_actions: &legal,
        },
        &choice.map_or(ScreenMode::Normal, ScreenMode::ChoosingDiscard),
        catalog,
    )
}

fn labels(buttons: &[Button]) -> Vec<&str> {
    buttons.iter().map(|button| button.label.as_str()).collect()
}

fn hand_card<'a>(screen: &'a GameScreen, name: &str) -> &'a HandCard {
    screen
        .hand
        .iter()
        .find(|card| card.name == name)
        .unwrap_or_else(|| panic!("{name} en main"))
}

/// Une partie d'Arythea dont la main de départ contient `wanted`.
fn session_with(wanted: &[BasicAction]) -> GameSession {
    for seed in 0..500 {
        let mut session = GameSession::new(config(seed, Hero::Arythea));
        session.run_frame();
        let hand = session.view().hand;
        if wanted.iter().all(|action| {
            hand.iter()
                .any(|card| card.card() == DeedCard::Basic(*action))
        }) {
            return session;
        }
    }
    panic!("aucune graine ne donne {wanted:?}");
}

// Spec 013 C19
#[test]
fn hand_shows_cards_with_their_plays() {
    let catalog = french();
    let state = with_hand(&[card(March, 1), card(Rage, 1), card(Tranquility, 1)]);
    let screen = screen_of(&state, None, &catalog);

    let march = hand_card(&screen, "Marche");
    assert_eq!(march.color, Some(BasicColor::Green));
    assert_eq!(march.effect, "Mouvement 2");
    assert_eq!(labels(&march.plays), ["Jouer : Mouvement 2"]);
    assert_eq!(march.sideways.len(), 2);

    let rage = hand_card(&screen, "Rage");
    assert_eq!(rage.color, Some(BasicColor::Red));
    assert_eq!(rage.effect, "Attaque 2 ou Parade 2");
    assert!(rage.plays.is_empty());
    assert_eq!(rage.sideways.len(), 2);

    // Spec 034 : Tranquillité est devenue jouable.
    let tranquility = hand_card(&screen, "Tranquillité");
    assert_eq!(tranquility.effect, "Soin 1 ou Piocher 1 carte(s)");
    assert_eq!(tranquility.plays.len(), 2);
}

// Spec 013 C20
#[test]
fn play_buttons_describe_the_effect() {
    let catalog = french();
    let state = with_hand(&[card(Improvisation, 1), card(March, 1), card(Instinct, 1)]);
    let screen = screen_of(&state, None, &catalog);

    assert_eq!(screen.sideways_title, "Jouer de côté");
    let march = hand_card(&screen, "Marche");
    // Spec 071, C14 : des boutons-icônes, lus par leur texte de repli.
    assert_eq!(
        march
            .sideways
            .iter()
            .map(|button| button.label.as_str())
            .collect::<Vec<_>>(),
        ["Mouvement 1", "Influence 1"]
    );
    let instinct = hand_card(&screen, "Instinct");
    assert_eq!(
        instinct.effect,
        "Mouvement 2 ou Influence 2 ou Attaque 2 ou Parade 2"
    );
    assert_eq!(
        labels(&instinct.plays),
        ["Jouer : Mouvement 2", "Jouer : Influence 2"]
    );
    let improvisation = hand_card(&screen, "Improvisation");
    assert_eq!(
        improvisation.effect,
        "Défausser une autre carte : Mouvement 3 ou Influence 3 ou Attaque 3 ou Parade 3"
    );
    assert_eq!(
        labels(&improvisation.plays),
        ["Jouer : Mouvement 3…", "Jouer : Influence 3…"]
    );
    assert_eq!(
        improvisation.plays[0].command,
        Command::ChooseDiscard {
            card: card(Improvisation, 1),
            option: 0
        }
    );
}

// Spec 013 C21
#[test]
fn screen_shows_play_area_and_points() {
    let catalog = french();
    let (march, rage, improvisation, swiftness) = (
        card(March, 1),
        card(Rage, 1),
        card(Improvisation, 1),
        card(Swiftness, 1),
    );
    let state = with_hand(&[march, rage, improvisation, swiftness]);
    let empty = screen_of(&state, None, &catalog);
    assert_eq!(empty.play_area_empty.as_deref(), Some("Aucune carte jouée"));
    assert_eq!(empty.discard_button.label, "Défausse : 0 carte(s)");

    let state = play(
        &state,
        Action::PlayCard {
            card: march,
            play: CardPlay::Basic {
                die: None,
                gain: None,
                with: None,
                option: 0,
                discard: None,
                mana: None,
            },
        },
    );
    let state = play(
        &state,
        Action::PlayCard {
            card: rage,
            play: CardPlay::Sideways(SidewaysEffect::Influence),
        },
    );
    let state = play(
        &state,
        Action::PlayCard {
            card: improvisation,
            play: CardPlay::Basic {
                die: None,
                gain: None,
                with: None,
                option: 0,
                discard: Some(swiftness),
                mana: None,
            },
        },
    );
    let screen = screen_of(&state, None, &catalog);
    assert_eq!(screen.play_area_title, "Zone de jeu");
    let lines: Vec<&str> = screen
        .play_area
        .iter()
        .map(|line| line.text.as_str())
        .collect();
    assert_eq!(
        lines,
        [
            "Marche : Mouvement 2",
            "Rage : Influence 1 (de côté)",
            "Improvisation : Mouvement 3, Célérité défaussée"
        ]
    );
    assert_eq!(screen.play_area[0].color, BasicColor::Green);
    assert_eq!(screen.play_area_empty, None);
    // Spec 071, C3 : icônes, lues par leur texte de repli.
    assert_eq!(screen.points.fallback_text(), "Mouvement 5 · Influence 1");
    assert_eq!(screen.discard_button.label, "Défausse : 1 carte(s)");
}

// Spec 013 C22
#[test]
fn clicking_a_play_button_plays_the_card() {
    let catalog = french();
    let mut session = session_with(&[March, Rage]);
    let screen = session.screen(&catalog);
    let march = hand_card(&screen, "Marche").card;
    let command = hand_card(&screen, "Marche").plays[0].command;
    session.command(command);
    session.run_frame();
    let view = session.view();
    assert!(!view.hand.contains(&march));
    assert_eq!(view.move_points, 2);

    let screen = session.screen(&catalog);
    let rage = hand_card(&screen, "Rage");
    let influence = rage
        .sideways
        .iter()
        .find(|button| button.label == "Influence 1")
        .expect("Influence 1 dans le menu de côté")
        .command;
    let rage = rage.card;
    session.command(influence);
    session.run_frame();
    let view = session.view();
    assert!(!view.hand.contains(&rage));
    assert_eq!(view.influence_points, 1);
}

// Spec 013 C23
#[test]
fn every_card_key_exists_in_french() {
    let catalog = french();
    for hero in [Hero::Arythea, Hero::Goldyx, Hero::Norowas, Hero::Tovak] {
        let deck = hero.starting_deck();
        for hand in deck.chunks(5) {
            let state = with_hand(hand);
            screen_of(&state, None, &catalog);
            let first = legal_actions(&state, SOLO)[0].clone();
            let played = play(&state, first);
            screen_of(&played, None, &catalog);
        }
    }
    let improvisation = card(Improvisation, 1);
    let state = with_hand(&[improvisation, card(March, 1)]);
    screen_of(
        &state,
        Some(DiscardChoice {
            card: improvisation,
            option: 1,
        }),
        &catalog,
    );
    let english = Catalog::load(
        &repository_assets_dir().join("i18n"),
        &Language("en".into()),
    )
    .expect("en.json du dépôt");
    for action in BasicAction::ALL {
        catalog.text(action.label_key());
        english.text(action.label_key());
    }
    assert_eq!(catalog.gaps(), vec![]);
    assert_eq!(english.gaps(), vec![]);
}

// Spec 013 C24
#[test]
fn discard_choice_lists_the_other_cards() {
    let catalog = french();
    let mut session = session_with(&[Improvisation]);
    let screen = session.screen(&catalog);
    let command = hand_card(&screen, "Improvisation").plays[0].command;
    let moves_before = session.view().move_points;
    session.command(command);
    session.run_frame();

    assert_eq!(session.view().move_points, moves_before, "rien n'est joué");
    let screen = session.screen(&catalog);
    let prompt = screen.discard_prompt.expect("bandeau du mode choix");
    assert_eq!(
        prompt.text,
        "Improvisation : Mouvement 3. Choisissez la carte à défausser."
    );
    assert_eq!(prompt.cancel.label, "Annuler");
    assert_eq!(prompt.cancel.command, Command::CancelChoice);
    assert_eq!(screen.hand.len(), 4);
    assert!(screen.hand.iter().all(|card| card.name != "Improvisation"));
    for card in &screen.hand {
        assert_eq!(labels(&card.plays), ["Défausser"]);
        assert!(card.sideways.is_empty());
    }
}

// Spec 013 C25
#[test]
fn choosing_a_card_to_discard_plays_improvisation() {
    let catalog = french();
    let mut session = session_with(&[Improvisation]);
    let screen = session.screen(&catalog);
    let improvisation = hand_card(&screen, "Improvisation").card;
    session.command(hand_card(&screen, "Improvisation").plays[0].command);

    let screen = session.screen(&catalog);
    let victim = screen.hand[0].card;
    session.command(screen.hand[0].plays[0].command);
    session.run_frame();

    let view = session.view();
    assert_eq!(view.move_points, 3);
    assert_eq!(view.discard, vec![victim]);
    assert_eq!(view.play_area[0].card, improvisation);
    assert!(!session.is_choosing_discard());
    assert_eq!(session.screen(&catalog).discard_prompt, None);
}

// Spec 013 C26
#[test]
fn discard_choice_can_be_cancelled() {
    let catalog = french();
    let mut session = session_with(&[Improvisation]);
    let before = session.view();
    let screen = session.screen(&catalog);
    session.command(hand_card(&screen, "Improvisation").plays[0].command);
    assert!(session.is_choosing_discard());

    session.command(Command::CancelChoice);
    session.run_frame();
    assert!(!session.is_choosing_discard());
    assert_eq!(session.view(), before);
    let screen = session.screen(&catalog);
    assert_eq!(screen.discard_prompt, None);
    assert_eq!(screen.hand.len(), 5);
}
