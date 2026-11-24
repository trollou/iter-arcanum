//! Scénarios de la spec 022 côté affichage (effets renforcés, choix du mana, dé pris).

use arcanum_core::{
    Action, BasicAction, BasicColor, CardId, CardPlay, DeedCard, Event, GameConfig, GameState,
    Hero, Hex, ManaColor, ManaPayment, PlayerId, Scenario, TileId, TilePosition, apply, decide,
    legal_actions, view,
};
use arcanum_i18n::{Catalog, Language};
use arcanum_ui::{
    Command, DiscardChoice, GameScreen, GameSession, HandCard, ManaChoice, ScreenMode, ScreenPhase,
    game_screen, repository_assets_dir,
};

const SOLO: PlayerId = PlayerId::SOLO;

use BasicAction::*;
use BasicColor::*;

const GOLD: ManaColor = ManaColor::Gold;
const BLACK: ManaColor = ManaColor::Black;

fn basic(color: BasicColor) -> ManaColor {
    ManaColor::Basic(color)
}

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

/// Partie de `hero` dont la main est `hand`, avec la Source `dice`.
fn state_of(hero: Hero, hand: &[CardId], dice: &[ManaColor]) -> GameState {
    let mut order = hand.to_vec();
    let mut rest = hero.starting_deck();
    for held in hand {
        if let Some(index) = rest.iter().position(|card| card == held) {
            rest.remove(index);
        }
    }
    order.extend(rest);
    let mut events = vec![
        Event::GameStarted {
            config: config(0, hero),
        },
        Event::TilePlaced {
            tile: TileId::StartingA,
            position: TilePosition::ORIGIN,
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
    events.push(Event::HeroFigurePlaced {
        player: SOLO,
        hex: Hex::ORIGIN,
    });
    events.push(Event::SourceRolled {
        dice: dice.to_vec(),
    });
    events.iter().fold(GameState::default(), apply)
}

fn state_with(hand: &[CardId], dice: &[ManaColor]) -> GameState {
    state_of(Hero::Tovak, hand, dice)
}

fn with_crystals(state: GameState, color: BasicColor, count: u8) -> GameState {
    (0..count).fold(state, |state, _| {
        apply(
            state,
            &Event::CrystalGained {
                player: SOLO,
                color,
            },
        )
    })
}

fn play(state: &GameState, action: Action) -> GameState {
    decide(state, SOLO, &action, &mut config(0, Hero::Tovak).rngs())
        .expect("action légale")
        .iter()
        .fold(state.clone(), apply)
}

fn strong(option: u8, mana: ManaPayment) -> CardPlay {
    CardPlay::Strong {
        gain: None,
        extra_mana: None,
        option,
        discard: None,
        mana,
        // Spec 025 : aucun dé mis sur une couleur.
        set_dice: [None, None],
        with: None,
    }
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

fn hand_card<'a>(screen: &'a GameScreen, name: &str) -> &'a HandCard {
    screen
        .hand
        .iter()
        .find(|card| card.name == name)
        .unwrap_or_else(|| panic!("{name} en main"))
}

/// L'action légale désignée par un bouton `Play`.
fn action_of(state: &GameState, command: Command) -> Action {
    let Command::Play(index) = command else {
        panic!("{command:?} doit jouer une action");
    };
    legal_actions(state, SOLO)[index].clone()
}

// Spec 022 C22
#[test]
fn hand_shows_strong_effects() {
    let catalog = french();
    let state = state_with(
        // Spec 025 : Concentration renforcée devient jouable ; spec 034 : Tranquillité aussi.
        &[card(March, 1), card(Rage, 1), card(Tranquility, 1)],
        &[basic(Green), basic(Blue), BLACK],
    );
    let screen = screen_of(&state, &catalog);
    let march = hand_card(&screen, "Marche");
    assert_eq!(march.strong_effect, "Renforcé : Mouvement 4");
    assert_eq!(march.strong_plays.len(), 1);
    assert_eq!(march.strong_plays[0].label, "Renforcer : Mouvement 4");
    assert_eq!(march.strong_plays[0].color, Green);
    assert!(march.strong_plays[0].enabled);

    let rage = hand_card(&screen, "Rage");
    assert_eq!(rage.strong_effect, "Renforcé : Attaque 4");
    assert!(rage.strong_plays.is_empty());
    let tranquility = hand_card(&screen, "Tranquillité");
    assert_eq!(
        tranquility.strong_effect,
        "Renforcé : Soin 2 ou Piocher 2 carte(s)"
    );
    assert_eq!(tranquility.strong_plays.len(), 2);
}

// Spec 022 C23
#[test]
fn strong_button_explains_missing_mana() {
    let catalog = french();
    let (march, stamina) = (card(March, 1), card(Stamina, 1));
    let state = state_with(&[march, stamina], &[basic(Red), basic(Blue), BLACK]);
    let screen = screen_of(&state, &catalog);
    let button = &hand_card(&screen, "Marche").strong_plays[0];
    assert!(!button.enabled);
    assert_eq!(
        button.tooltip.as_deref(),
        Some("Il faut un mana vert : un dé vert ou or de la Source, un cristal ou un jeton vert.")
    );

    // Le dé bleu pris pour Endurance : plus de dé ce tour.
    let state = play(
        &state_with(
            &[march, stamina, card(Stamina, 2)],
            &[basic(Blue), basic(Green), BLACK],
        ),
        Action::PlayCard {
            card: card(Stamina, 2),
            play: strong(0, ManaPayment::Die(0)),
        },
    );
    let screen = screen_of(&state, &catalog);
    let button = &hand_card(&screen, "Marche").strong_plays[0];
    assert!(!button.enabled);
    assert_eq!(
        button.tooltip.as_deref(),
        Some("Dé du tour déjà pris : il faut un cristal ou un jeton vert.")
    );
}

// Spec 022 C24
#[test]
fn single_payment_plays_directly() {
    let catalog = french();
    let march = card(March, 1);
    let state = state_with(&[march], &[basic(Green), basic(Blue), BLACK]);
    let screen = screen_of(&state, &catalog);
    let command = hand_card(&screen, "Marche").strong_plays[0].command;
    assert_eq!(
        action_of(&state, command),
        Action::PlayCard {
            card: march,
            play: strong(0, ManaPayment::Die(0))
        }
    );
}

// Spec 022 C25 et C26
#[test]
fn mana_choice_lists_payments() {
    let catalog = french();
    let march = card(March, 1);
    let state = with_crystals(
        state_with(&[march], &[basic(Green), GOLD, basic(Green)]),
        Green,
        1,
    );
    let screen = screen_of(&state, &catalog);
    let command = hand_card(&screen, "Marche").strong_plays[0].command;
    assert_eq!(
        command,
        Command::ChooseMana {
            card: march,
            option: 0,
            discard: None
        }
    );
    let mode = ScreenMode::ChoosingMana(ManaChoice {
        card: march,
        option: 0,
        discard: None,
    });
    let screen = screen_in(&state, &mode, &catalog);
    let prompt = screen
        .mana_prompt
        .clone()
        .expect("bandeau du choix du mana");
    assert_eq!(
        prompt.text,
        "Marche renforcée : Mouvement 4. Choisissez le mana à payer."
    );
    let labels: Vec<&str> = prompt.payments.iter().map(|b| b.label.as_str()).collect();
    assert_eq!(
        labels,
        ["Dé vert", "Dé or (vaut vert)", "Cristal vert (1 sur 3)"]
    );
    let actions: Vec<Action> = prompt
        .payments
        .iter()
        .map(|button| action_of(&state, button.command))
        .collect();
    assert_eq!(
        actions,
        [
            ManaPayment::Die(0),
            ManaPayment::Die(1),
            ManaPayment::Crystal
        ]
        .map(|mana| Action::PlayCard {
            card: march,
            play: strong(0, mana)
        })
    );
    assert_eq!(prompt.cancel.label, "Annuler");
    assert_eq!(prompt.cancel.command, Command::CancelChoice);
    assert!(screen.hand.iter().all(|held| held.card != march));
    // « Finir le tour… » grisé pendant le choix (issue privée 259 : le bouton dessiné).
    assert_eq!(
        screen
            .bottom_buttons
            .last()
            .and_then(|button| button.command),
        None
    );
}

// Spec 022 C27
#[test]
fn improvisation_strong_chooses_discard_then_mana() {
    let catalog = french();
    let (improvisation, march, rage) = (card(Improvisation, 1), card(March, 1), card(Rage, 1));
    let state = with_crystals(
        state_with(
            &[improvisation, march, rage],
            &[basic(Red), basic(Blue), BLACK],
        ),
        Red,
        1,
    );
    let screen = screen_of(&state, &catalog);
    let card_view = hand_card(&screen, "Improvisation");
    assert_eq!(card_view.strong_plays[0].label, "Renforcer : Mouvement 5…");
    assert_eq!(
        card_view.strong_plays[0].command,
        Command::ChooseStrongDiscard {
            card: improvisation,
            option: 0
        }
    );
    let mode = ScreenMode::ChoosingStrongDiscard(DiscardChoice {
        card: improvisation,
        option: 0,
    });
    let screen = screen_in(&state, &mode, &catalog);
    let prompt = screen
        .discard_prompt
        .clone()
        .expect("bandeau du choix de la défausse");
    assert_eq!(
        prompt.text,
        "Improvisation : Mouvement 5. Choisissez la carte à défausser."
    );
    let rage_view = hand_card(&screen, "Rage");
    assert_eq!(rage_view.plays.len(), 1);
    assert_eq!(rage_view.plays[0].label, "Défausser cette carte");
    assert_eq!(
        rage_view.plays[0].command,
        Command::ChooseMana {
            card: improvisation,
            option: 0,
            discard: Some(rage)
        }
    );
    // Un seul paiement (sans le cristal rouge) : la défausse joue tout de suite.
    let single = state_with(
        &[improvisation, march, rage],
        &[basic(Red), basic(Blue), BLACK],
    );
    let screen = screen_in(&single, &mode, &catalog);
    let command = hand_card(&screen, "Rage").plays[0].command;
    assert_eq!(
        action_of(&single, command),
        Action::PlayCard {
            card: improvisation,
            play: CardPlay::Strong {
                gain: None,
                extra_mana: None,
                option: 0,
                discard: Some(rage),
                mana: ManaPayment::Die(0),
                set_dice: [None, None],
                with: None,
            }
        }
    );
}

// Spec 022 C28
#[test]
fn panel_shows_taken_die() {
    let catalog = french();
    let march = card(March, 1);
    let state = state_with(&[march], &[GOLD, basic(Green), BLACK]);
    let state = play(
        &state,
        Action::PlayCard {
            card: march,
            play: strong(0, ManaPayment::Die(1)),
        },
    );
    let source = screen_of(&state, &catalog).source;
    let names = |dice: &[arcanum_ui::SourceDie]| {
        dice.iter().map(|die| die.name.clone()).collect::<Vec<_>>()
    };
    assert_eq!(names(&source.dice), ["or"]);
    assert_eq!(source.taken_title.as_deref(), Some("pris :"));
    assert_eq!(names(&source.taken), ["vert"]);
    assert_eq!(
        source.taken[0].tooltip,
        "Dé de mana vert : pris ce tour, relancé à la fin du tour."
    );
    assert_eq!(names(&source.depleted), ["noir"]);
}

// Spec 022 C29
#[test]
fn play_area_names_the_mana_paid() {
    let catalog = french();
    let (march, stamina, promise) = (card(March, 1), card(Stamina, 1), card(Promise, 1));
    let state = with_crystals(
        state_with(&[march, stamina, promise], &[basic(Green), GOLD, BLACK]),
        Blue,
        1,
    );
    let state = play(
        &state,
        Action::PlayCard {
            card: march,
            play: strong(0, ManaPayment::Die(0)),
        },
    );
    let state = play(
        &state,
        Action::PlayCard {
            card: stamina,
            play: strong(0, ManaPayment::Crystal),
        },
    );
    let lines = |state: &GameState| {
        screen_of(state, &catalog)
            .play_area
            .iter()
            .map(|line| line.text.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(
        lines(&state),
        [
            "Marche (renforcée, dé vert) : Mouvement 4",
            "Endurance (renforcée, cristal bleu) : Mouvement 4"
        ]
    );
    let gold = play(
        &state_with(&[promise], &[GOLD, BLACK, BLACK]),
        Action::PlayCard {
            card: promise,
            play: strong(0, ManaPayment::Die(0)),
        },
    );
    assert_eq!(lines(&gold), ["Promesse (renforcée, dé or) : Influence 4"]);
}

// Spec 022 C30
#[test]
fn every_strong_effect_key_exists_in_french() {
    let catalog = french();
    let english = Catalog::load(
        &repository_assets_dir().join("i18n"),
        &Language("en".into()),
    )
    .expect("en.json du dépôt");
    for hero in [Hero::Arythea, Hero::Goldyx, Hero::Norowas, Hero::Tovak] {
        for hand in hero.starting_deck().chunks(5) {
            for dice in [
                [GOLD, BLACK, basic(Red)],
                [basic(Blue), basic(Green), basic(White)],
            ] {
                let mut state = state_of(hero, hand, &dice);
                for color in BasicColor::ALL {
                    state = with_crystals(state, color, 1);
                }
                for catalog in [&catalog, &english] {
                    screen_of(&state, catalog);
                }
                let strong_action = legal_actions(&state, SOLO).into_iter().find(|action| {
                    matches!(
                        action,
                        Action::PlayCard {
                            play: CardPlay::Strong { .. },
                            ..
                        }
                    )
                });
                if let Some(action) = strong_action {
                    if let Action::PlayCard {
                        card,
                        play:
                            CardPlay::Strong {
                                option, discard, ..
                            },
                    } = action
                    {
                        let mode = ScreenMode::ChoosingMana(ManaChoice {
                            card,
                            option,
                            discard,
                        });
                        for catalog in [&catalog, &english] {
                            screen_in(&state, &mode, catalog);
                        }
                    }
                    let played = play(&state, action);
                    for catalog in [&catalog, &english] {
                        screen_of(&played, catalog);
                    }
                }
            }
        }
    }
    assert_eq!(catalog.gaps(), vec![]);
    assert_eq!(english.gaps(), vec![]);
}

// Spec 022 C25 : la session ouvre et ferme le choix du mana, puis joue le paiement choisi.
#[test]
fn session_opens_cancels_and_completes_the_mana_choice() {
    let catalog = french();
    for seed in 0..2000 {
        let mut session = GameSession::new(config(seed, Hero::Arythea));
        session.run_frame();
        let screen = session.screen(&catalog);
        let Some(command) = screen
            .hand
            .iter()
            .flat_map(|card| &card.strong_plays)
            // Un bouton grisé ne se clique pas : son choix du mana serait vide.
            .filter(|button| button.enabled)
            .map(|button| button.command)
            .find(|command| matches!(command, Command::ChooseMana { .. }))
        else {
            continue;
        };
        let before = session.view();
        session.command(command);
        assert!(session.screen(&catalog).mana_prompt.is_some());
        session.command(Command::CancelChoice);
        assert_eq!(session.screen(&catalog).mana_prompt, None);
        assert_eq!(session.view(), before);

        session.command(command);
        let pick = session
            .screen(&catalog)
            .mana_prompt
            .expect("bandeau")
            .payments[0]
            .command;
        session.command(pick);
        session.run_frame();
        let after = session.view();
        assert_eq!(after.hand.len(), before.hand.len() - 1);
        assert_eq!(session.screen(&catalog).mana_prompt, None);
        return;
    }
    panic!("aucune graine ne propose de choix du mana");
}
