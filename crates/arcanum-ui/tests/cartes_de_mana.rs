//! Scénarios de la spec 025 côté affichage : jetons, cristaux gagnés, reprise d'Euphorie (partie A),
//! dés mis sur une couleur (partie B), Concentration renforcée (partie C).

use arcanum_core::{
    Action, BasicAction, BasicColor, BoostTarget, CardId, CardPlay, DeedCard, DieSetting, Event,
    GameConfig, GameState, Hero, Hex, ManaColor, ManaPayment, PlayerId, Scenario, TileId,
    TilePosition, apply, decide, legal_actions, view,
};
use arcanum_i18n::{Catalog, Language};
use arcanum_ui::{
    BoostChoice, Command, EndTurnChoice, GameScreen, HandCard, ManaChoice, ScreenMode, ScreenPhase,
    SetDiceChoice, game_screen, repository_assets_dir,
};

const SOLO: PlayerId = PlayerId::SOLO;

use BasicAction::*;
use BasicColor::*;

const BLACK: ManaColor = ManaColor::Black;
const GOLD: ManaColor = ManaColor::Gold;

fn basic(color: BasicColor) -> ManaColor {
    ManaColor::Basic(color)
}

fn load(language: &str) -> Catalog {
    Catalog::load(
        &repository_assets_dir().join("i18n"),
        &Language(language.into()),
    )
    .expect("fichier de langue du dépôt")
}

fn french() -> Catalog {
    load("fr")
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

fn with_token(state: GameState, color: BasicColor) -> GameState {
    apply(
        state,
        &Event::ManaTokenGained {
            player: SOLO,
            color: ManaColor::Basic(color),
        },
    )
}

fn play(state: &GameState, action: Action) -> GameState {
    decide(state, SOLO, &action, &mut config(0, Hero::Tovak).rngs())
        .expect("action légale")
        .iter()
        .fold(state.clone(), apply)
}

fn basic_play(option: u8, mana: Option<ManaPayment>) -> CardPlay {
    CardPlay::Basic {
        die: None,
        gain: None,
        with: None,
        option,
        discard: None,
        mana,
    }
}

fn strong(option: u8, mana: ManaPayment) -> CardPlay {
    CardPlay::Strong {
        gain: None,
        extra_mana: None,
        option,
        discard: None,
        mana,
        set_dice: [None, None],
        with: None,
    }
}

fn setting(die: u8, color: BasicColor) -> Option<DieSetting> {
    Some(DieSetting { die, color })
}

/// Effet renforcé de Source de mana ou d'Attraction de mana.
fn strong_set(mana: ManaPayment, set_dice: [Option<DieSetting>; 2]) -> CardPlay {
    CardPlay::Strong {
        gain: None,
        extra_mana: None,
        option: 0,
        discard: None,
        mana,
        set_dice,
        with: None,
    }
}

/// Concentration renforcée (option 0) payée par `mana`, avec l'option `option` de `target`.
fn boosting(
    mana: ManaPayment,
    target: CardId,
    option: u8,
    discard: Option<CardId>,
    set_dice: [Option<DieSetting>; 2],
) -> CardPlay {
    CardPlay::Strong {
        gain: None,
        extra_mana: None,
        option: 0,
        discard: None,
        mana,
        set_dice: [None, None],
        with: Some(BoostTarget {
            card: target,
            option,
            discard,
            set_dice,
        }),
    }
}

/// Le mode Concentration au départ : aucune carte choisie.
fn boost_start(concentration: CardId) -> BoostChoice {
    BoostChoice {
        card: concentration,
        option: 0,
        basic: false,
        target: None,
        target_option: 0,
        discard: None,
        settings: [None, None],
        die: None,
    }
}

/// Le mode Concentration désigné par un bouton.
/// Les boutons actifs du bloc renforcé en mode Concentration (spec 073, C19 bis) : infobulle et
/// commande.
fn boost_buttons(card: &arcanum_ui::HandCard) -> Vec<(&str, Command)> {
    card.strong_effects
        .iter()
        .filter_map(|button| {
            button
                .command
                .map(|command| (button.tooltip.as_str(), command))
        })
        .collect()
}

fn boost_tooltips(card: &arcanum_ui::HandCard) -> Vec<&str> {
    boost_buttons(card)
        .into_iter()
        .map(|(tooltip, _)| tooltip)
        .collect()
}

fn boost_mode(command: Command) -> ScreenMode {
    let Command::ChooseBoost(choice) = command else {
        panic!("{command:?} doit avancer dans le mode Concentration");
    };
    ScreenMode::Boosting(choice)
}

/// Mode « mettre un dé sur une couleur » pour `card`, avec les dés déjà choisis.
fn setting_dice(card: CardId, settings: [Option<DieSetting>; 2], die: Option<u8>) -> ScreenMode {
    ScreenMode::SettingDice(SetDiceChoice {
        card,
        option: 0,
        settings,
        die,
    })
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

fn labels<'a>(buttons: impl IntoIterator<Item = &'a String>) -> Vec<&'a str> {
    buttons.into_iter().map(String::as_str).collect()
}

/// Goldyx a joué Euphorie cristalline, payée par un cristal vert ; en main : Marche et Endurance.
fn after_crystal_joy() -> (GameState, CardId, CardId) {
    let (joy, march, stamina) = (card(CrystalJoy, 1), card(March, 1), card(Stamina, 1));
    let state = with_crystals(
        state_of(Hero::Goldyx, &[joy, march, stamina], &[BLACK, BLACK, BLACK]),
        Green,
        1,
    );
    let state = play(
        &state,
        Action::PlayCard {
            card: joy,
            play: basic_play(3, Some(ManaPayment::Crystal)),
        },
    );
    (state, march, stamina)
}

// Spec 025 C25
#[test]
fn play_area_shows_mana_tokens() {
    let catalog = french();
    let concentration = card(Concentration, 1);
    let state = state_with(&[concentration], &[BLACK, BLACK, BLACK]);
    assert_eq!(screen_of(&state, &catalog).tokens, None);

    let state = with_token(
        play(
            &state,
            Action::PlayCard {
                card: concentration,
                play: basic_play(0, None),
            },
        ),
        Red,
    );
    let screen = screen_of(&state, &catalog);
    let tokens = screen.tokens.expect("ligne des jetons");
    assert_eq!(tokens.title, "Jetons :");
    assert_eq!(
        tokens
            .tokens
            .iter()
            .map(|token| token.color)
            .collect::<Vec<_>>(),
        [ManaColor::Basic(Blue), ManaColor::Basic(Red)]
    );
    assert_eq!(
        tokens.tokens[0].tooltip,
        "Jeton de mana bleu : à utiliser ce tour, rendu à la réserve à la fin du tour."
    );
    assert_eq!(screen.play_area[0].text, "Concentration : Jeton bleu");
}

// Spec 025 C26
#[test]
fn hand_shows_mana_options() {
    let catalog = french();
    let (concentration, crystallize) = (card(Concentration, 1), card(Crystallize, 1));
    let state = state_with(&[concentration, crystallize], &[basic(Green), BLACK, BLACK]);
    let screen = screen_of(&state, &catalog);

    let concentration_view = hand_card(&screen, "Concentration");
    assert_eq!(
        concentration_view.effect,
        "Jeton bleu ou Jeton blanc ou Jeton rouge"
    );
    assert_eq!(
        labels(concentration_view.plays.iter().map(|b| &b.label)),
        [
            "Jouer : Jeton bleu",
            "Jouer : Jeton blanc",
            "Jouer : Jeton rouge"
        ]
    );

    let crystallize_view = hand_card(&screen, "Cristallisation");
    assert!(crystallize_view.plays.is_empty());
    assert_eq!(
        labels(crystallize_view.paid_plays.iter().map(|b| &b.label)),
        [
            "Payer rouge : cristal rouge",
            "Payer bleu : cristal bleu",
            "Payer blanc : cristal blanc",
            "Payer vert : cristal vert"
        ]
    );
    let green = &crystallize_view.paid_plays[3];
    assert!(green.enabled);
    assert_eq!(green.color, Green);
    // Un seul paiement : le bouton joue tout de suite.
    assert_eq!(
        action_of(&state, green.command),
        Action::PlayCard {
            card: crystallize,
            play: basic_play(3, Some(ManaPayment::Die(0)))
        }
    );
    let red = &crystallize_view.paid_plays[0];
    assert!(!red.enabled);
    assert_eq!(
        red.tooltip.as_deref(),
        Some(
            "Il faut un mana rouge : un dé rouge ou or de la Source, un cristal ou un jeton rouge."
        )
    );
    assert_eq!(
        labels(crystallize_view.strong_plays.iter().map(|b| &b.label)),
        [
            "Renforcer : Cristal rouge",
            "Renforcer : Cristal bleu",
            "Renforcer : Cristal blanc",
            "Renforcer : Cristal vert"
        ]
    );
    assert!(crystallize_view.strong_plays.iter().all(|b| !b.enabled));
}

// Spec 025 C27
#[test]
fn mana_choice_lists_tokens() {
    let catalog = french();
    let stamina = card(Stamina, 1);
    let state = with_token(state_with(&[stamina], &[basic(Blue), BLACK, BLACK]), Blue);
    let command = hand_card(&screen_of(&state, &catalog), "Endurance").strong_plays[0].command;
    assert_eq!(
        command,
        Command::ChooseMana {
            card: stamina,
            option: 0,
            discard: None
        }
    );
    let mode = ScreenMode::ChoosingMana(ManaChoice {
        card: stamina,
        option: 0,
        discard: None,
    });
    let prompt = screen_in(&state, &mode, &catalog)
        .mana_prompt
        .expect("bandeau du choix du mana");
    assert_eq!(
        labels(prompt.payments.iter().map(|b| &b.label)),
        ["Dé bleu", "Jeton bleu"]
    );
    let token = action_of(&state, prompt.payments[1].command);
    assert_eq!(
        token,
        Action::PlayCard {
            card: stamina,
            play: strong(0, ManaPayment::Token)
        }
    );
    let after = play(&state, token);
    assert_eq!(
        screen_of(&after, &catalog).play_area[0].text,
        "Endurance (renforcée, jeton bleu) : Mouvement 4"
    );
}

// Spec 025 C27 : même choix pour l'effet de base payant de Cristallisation.
#[test]
fn crystallize_chooses_its_mana() {
    let catalog = french();
    let crystallize = card(Crystallize, 1);
    let state = with_token(
        with_crystals(state_with(&[crystallize], &[BLACK, BLACK, BLACK]), Red, 1),
        Red,
    );
    let screen = screen_of(&state, &catalog);
    let button = &hand_card(&screen, "Cristallisation").paid_plays[0];
    assert_eq!(
        button.command,
        Command::ChooseBasicMana {
            card: crystallize,
            option: 0
        }
    );
    let mode = ScreenMode::ChoosingBasicMana(ManaChoice {
        card: crystallize,
        option: 0,
        discard: None,
    });
    let screen = screen_in(&state, &mode, &catalog);
    let prompt = screen.mana_prompt.expect("bandeau du choix du mana");
    assert_eq!(
        prompt.text,
        "Cristallisation : Payer rouge : cristal rouge. Choisissez le mana à payer."
    );
    assert_eq!(
        labels(prompt.payments.iter().map(|b| &b.label)),
        ["Cristal rouge (1 sur 3)", "Jeton rouge"]
    );
    assert_eq!(
        action_of(&state, prompt.payments[0].command),
        Action::PlayCard {
            card: crystallize,
            play: basic_play(0, Some(ManaPayment::Crystal))
        }
    );
    assert!(screen.hand.iter().all(|held| held.card != crystallize));
}

// Spec 025 C30
#[test]
fn end_turn_mode_offers_crystal_joy() {
    let catalog = french();
    let (state, march, stamina) = after_crystal_joy();
    let mode = ScreenMode::EndingTurn(EndTurnChoice::default());
    let screen = screen_in(&state, &mode, &catalog);
    let prompt = screen.end_turn_prompt.expect("bandeau de fin du tour");
    let reclaim = prompt.reclaim.expect("menu de reprise");
    assert_eq!(
        reclaim.label,
        "Reprendre Euphorie cristalline en défaussant :"
    );
    assert_eq!(reclaim.selected, "(ne pas reprendre)");
    assert_eq!(
        labels(reclaim.options.iter().map(|b| &b.label)),
        ["(ne pas reprendre)", "Marche", "Endurance"]
    );
    assert_eq!(
        reclaim
            .options
            .iter()
            .map(|b| b.command)
            .collect::<Vec<_>>(),
        [
            Command::ChooseReclaim(None),
            Command::ChooseReclaim(Some(march)),
            Command::ChooseReclaim(Some(stamina))
        ]
    );
    assert_eq!(prompt.draw, "Vous piocherez 3 carte(s).");

    // Marche choisie pour la reprise : elle ne se coche plus, et la pioche ne change pas.
    let choice = EndTurnChoice {
        discard: vec![march],
        reclaim: Some(march),
        glade: None,
        steady_tempo: None,
    };
    assert_eq!(
        choice.action(&view(&state, SOLO), &legal_actions(&state, SOLO)),
        Action::EndTurn {
            discard: vec![],
            reclaim: Some(march),
            glade: None,
            steady_tempo: false
        }
    );
    let screen = screen_in(&state, &ScreenMode::EndingTurn(choice), &catalog);
    assert_eq!(hand_card(&screen, "Marche").discard_toggle, None);
    let prompt = screen.end_turn_prompt.expect("bandeau de fin du tour");
    assert_eq!(prompt.reclaim.expect("menu").selected, "Marche");
    assert_eq!(prompt.draw, "Vous piocherez 3 carte(s).");
    assert!(prompt.confirm_enabled);

    // Sans Euphorie jouée, pas de menu.
    let plain = state_with(&[march, stamina], &[BLACK, BLACK, BLACK]);
    let screen = screen_in(&plain, &mode, &catalog);
    assert_eq!(screen.end_turn_prompt.expect("bandeau").reclaim, None);
}

// --- B. Dés supplémentaires ---

// Spec 025 C28 — Source de mana renforcée, un seul paiement possible
#[test]
fn set_die_mode_chooses_die_then_color() {
    let catalog = french();
    let (mana_draw, march) = (card(ManaDraw, 1), card(March, 1));
    let state = with_crystals(
        state_with(&[mana_draw, march], &[basic(Green), GOLD, BLACK]),
        White,
        1,
    );
    let screen = screen_of(&state, &catalog);
    let view_card = hand_card(&screen, "Source de mana");
    assert_eq!(view_card.effect, "Un dé de plus ce tour");
    assert_eq!(
        labels(view_card.plays.iter().map(|button| &button.label)),
        ["Jouer : Un dé de plus ce tour"]
    );
    let button = &view_card.strong_plays[0];
    assert_eq!(
        button.label,
        "Renforcer : Mettre 1 dé(s) sur une couleur, gagner des jetons"
    );
    assert!(button.enabled);
    assert_eq!(
        button.command,
        Command::ChooseSetDice {
            card: mana_draw,
            option: 0
        }
    );

    // Choisir le dé : tous les dés libres, même épuisés, sont cliquables.
    let screen = screen_in(
        &state,
        &setting_dice(mana_draw, [None, None], None),
        &catalog,
    );
    let prompt = screen.set_dice_prompt.as_ref().expect("bandeau");
    assert_eq!(
        prompt.text,
        "Choisissez un dé de la Source, puis sa couleur."
    );
    assert!(prompt.colors.is_empty());
    assert_eq!(prompt.cancel.label, "Annuler");
    assert_eq!(prompt.cancel.command, Command::CancelChoice);
    let picks: Vec<Option<Command>> = screen
        .source
        .dice
        .iter()
        .chain(&screen.source.depleted)
        .map(|die| die.pick)
        .collect();
    assert_eq!(
        picks,
        [
            Some(Command::PickSetDie(0)),
            Some(Command::PickSetDie(1)),
            Some(Command::PickSetDie(2))
        ]
    );
    assert!(screen.hand.iter().all(|held| held.name != "Source de mana"));
    // « Finir le tour… » grisé pendant le choix (issue privée 259 : le bouton dessiné).
    assert_eq!(
        screen
            .bottom_buttons
            .last()
            .and_then(|button| button.command),
        None
    );

    // Puis sa couleur : le cristal blanc est le seul paiement, le bouton joue directement.
    let screen = screen_in(
        &state,
        &setting_dice(mana_draw, [None, None], Some(1)),
        &catalog,
    );
    let prompt = screen.set_dice_prompt.as_ref().expect("bandeau");
    assert_eq!(
        labels(prompt.colors.iter().map(|button| &button.label)),
        ["rouge", "bleu", "blanc", "vert"]
    );
    assert!(screen.source.dice.iter().all(|die| die.pick.is_none()));
    assert_eq!(
        action_of(&state, prompt.colors[3].command),
        Action::PlayCard {
            card: mana_draw,
            play: strong_set(ManaPayment::Crystal, [setting(1, Green), None]),
        }
    );
}

// Spec 025 C28 — plusieurs paiements : le choix du mana vient après
#[test]
fn set_die_mode_then_chooses_mana() {
    let catalog = french();
    let mana_draw = card(ManaDraw, 1);
    let state = with_crystals(
        state_with(&[mana_draw], &[basic(White), basic(Green), BLACK]),
        White,
        1,
    );
    let screen = screen_in(
        &state,
        &setting_dice(mana_draw, [None, None], Some(1)),
        &catalog,
    );
    let chosen = [setting(1, Red), None];
    let command = screen.set_dice_prompt.expect("bandeau").colors[0].command;
    assert_eq!(
        command,
        Command::ChooseSetDiceMana {
            card: mana_draw,
            option: 0,
            set_dice: chosen
        }
    );
    let mode = ScreenMode::ChoosingSetDiceMana(SetDiceChoice {
        card: mana_draw,
        option: 0,
        settings: chosen,
        die: None,
    });
    let screen = screen_in(&state, &mode, &catalog);
    let prompt = screen.mana_prompt.expect("choix du mana");
    assert_eq!(
        labels(prompt.payments.iter().map(|button| &button.label)),
        ["Dé blanc", "Cristal blanc (1 sur 3)"]
    );
    assert_eq!(
        action_of(&state, prompt.payments[0].command),
        Action::PlayCard {
            card: mana_draw,
            play: strong_set(ManaPayment::Die(0), chosen),
        }
    );
}

// Spec 025 C28 — Attraction de mana : deux dés, l'un après l'autre
#[test]
fn mana_pull_sets_two_dice_one_after_the_other() {
    let catalog = french();
    let mana_pull = card(ManaPull, 1);
    let state = with_crystals(
        state_of(Hero::Arythea, &[mana_pull], &[basic(Green), GOLD, BLACK]),
        White,
        1,
    );
    // Les dés se choisissent dans n'importe quel ordre : tous sont cliquables au départ.
    let screen = screen_in(
        &state,
        &setting_dice(mana_pull, [None, None], None),
        &catalog,
    );
    let picks: Vec<Option<Command>> = screen
        .source
        .dice
        .iter()
        .chain(&screen.source.depleted)
        .map(|die| die.pick)
        .collect();
    assert_eq!(
        picks,
        [
            Some(Command::PickSetDie(0)),
            Some(Command::PickSetDie(1)),
            Some(Command::PickSetDie(2))
        ]
    );
    // Le dernier dé de la Source d'abord : l'action garde l'ordre de la Source.
    let screen = screen_in(
        &state,
        &setting_dice(mana_pull, [setting(2, Blue), None], Some(0)),
        &catalog,
    );
    assert_eq!(
        screen.set_dice_prompt.expect("bandeau").colors[0].command,
        Command::ChooseSetDiceMana {
            card: mana_pull,
            option: 0,
            set_dice: [setting(0, Red), setting(2, Blue)],
        }
    );
    let screen = screen_in(
        &state,
        &setting_dice(mana_pull, [None, None], Some(0)),
        &catalog,
    );
    // Premier dé : la couleur choisie attend le second dé.
    assert_eq!(
        screen.set_dice_prompt.expect("bandeau").colors[0].command,
        Command::PickSetColor(Red)
    );
    let screen = screen_in(
        &state,
        &setting_dice(mana_pull, [setting(0, Red), None], None),
        &catalog,
    );
    let picks: Vec<Option<Command>> = screen
        .source
        .dice
        .iter()
        .chain(&screen.source.depleted)
        .map(|die| die.pick)
        .collect();
    // Le dé 0, déjà choisi, ne l'est plus.
    assert_eq!(
        picks,
        [
            None,
            Some(Command::PickSetDie(1)),
            Some(Command::PickSetDie(2))
        ]
    );
    let screen = screen_in(
        &state,
        &setting_dice(mana_pull, [setting(0, Red), None], Some(2)),
        &catalog,
    );
    // Le dé or ou le cristal blanc paie : le choix du mana suit.
    assert_eq!(
        screen.set_dice_prompt.expect("bandeau").colors[1].command,
        Command::ChooseSetDiceMana {
            card: mana_pull,
            option: 0,
            set_dice: [setting(0, Red), setting(2, Blue)],
        }
    );
}

// Spec 025 C31
#[test]
fn panel_shows_set_die() {
    let catalog = french();
    let mana_draw = card(ManaDraw, 1);
    let state = with_crystals(
        state_with(&[mana_draw], &[basic(Green), GOLD, BLACK]),
        White,
        1,
    );
    let state = play(
        &state,
        Action::PlayCard {
            card: mana_draw,
            play: strong_set(ManaPayment::Crystal, [setting(1, Green), None]),
        },
    );
    let source = screen_of(&state, &catalog).source;
    assert_eq!(source.taken_title.as_deref(), Some("pris :"));
    assert_eq!(source.taken.len(), 1);
    assert_eq!(source.taken[0].color, basic(Green));
    assert_eq!(
        source.taken[0].tooltip,
        "Dé de mana vert : mis sur vert ce tour, rendu sans être relancé."
    );
}

// Spec 025 C29
#[test]
fn concentration_mode_lists_targets() {
    let catalog = french();
    let (concentration, march, improvisation, rage) = (
        card(Concentration, 1),
        card(March, 1),
        card(Improvisation, 1),
        card(Rage, 1),
    );
    let state = state_with(
        &[concentration, march, improvisation, rage],
        &[basic(Green), GOLD, BLACK],
    );
    let screen = screen_of(&state, &catalog);
    let button = &hand_card(&screen, "Concentration").strong_plays[0];
    assert_eq!(
        button.label,
        "Renforcer : Renforcer une autre carte gratuitement, +2"
    );
    assert!(button.enabled);
    assert_eq!(
        button.command,
        Command::ChooseBoost(boost_start(concentration))
    );

    // Choisir la carte : chaque option renforcée jouable, avec le bonus.
    let mode = ScreenMode::Boosting(boost_start(concentration));
    let screen = screen_in(&state, &mode, &catalog);
    let prompt = screen.boost_prompt.as_ref().expect("bandeau");
    assert_eq!(
        prompt.text,
        "Concentration : choisissez la carte à renforcer gratuitement (+2)."
    );
    assert_eq!(prompt.cancel.label, "Annuler");
    assert_eq!(prompt.cancel.command, Command::CancelChoice);
    assert!(screen.hand.iter().all(|held| held.name != "Concentration"));
    assert_eq!(
        boost_tooltips(hand_card(&screen, "Marche")),
        ["Renforcer avec Concentration : Mouvement 6"]
    );
    let improvisation_card = hand_card(&screen, "Improvisation");
    assert_eq!(
        boost_tooltips(improvisation_card),
        [
            "Renforcer avec Concentration : Mouvement 7",
            "Renforcer avec Concentration : Influence 7"
        ]
    );
    // Rage : Attaque renforcée, pas hors combat.
    let rage_card = hand_card(&screen, "Rage");
    assert!(rage_card.plays.is_empty() && rage_card.sideways.is_empty());
    assert!(rage_card.strong_plays.is_empty() && rage_card.paid_plays.is_empty());
    // « Finir le tour… » grisé pendant le choix (issue privée 259 : le bouton dessiné).
    assert_eq!(
        screen
            .bottom_buttons
            .last()
            .and_then(|button| button.command),
        None
    );

    // Improvisation : puis la carte à défausser, ni Concentration ni Improvisation.
    let mode = boost_mode(boost_buttons(improvisation_card)[0].1);
    let screen = screen_in(&state, &mode, &catalog);
    assert!(screen.boost_prompt.is_none());
    assert_eq!(
        screen.discard_prompt.as_ref().expect("bandeau").text,
        "Improvisation : Mouvement 7. Choisissez la carte à défausser."
    );
    assert_eq!(
        screen
            .hand
            .iter()
            .map(|held| held.name.as_str())
            .collect::<Vec<_>>(),
        ["Marche", "Rage"]
    );
    let pick = &hand_card(&screen, "Rage").plays[0];
    assert_eq!(pick.label, "Défausser");

    // Puis le mana de Concentration : dé vert ou dé or.
    let mode = boost_mode(pick.command);
    let screen = screen_in(&state, &mode, &catalog);
    let prompt = screen.mana_prompt.expect("choix du mana");
    assert_eq!(
        prompt.text,
        "Concentration renforcée : Renforcer une autre carte gratuitement, +2. Choisissez le mana à payer."
    );
    assert_eq!(
        labels(prompt.payments.iter().map(|b| &b.label)),
        ["Dé vert", "Dé or (vaut vert)"]
    );
    assert_eq!(
        action_of(&state, prompt.payments[0].command),
        Action::PlayCard {
            card: concentration,
            play: boosting(
                ManaPayment::Die(0),
                improvisation,
                0,
                Some(rage),
                [None, None]
            ),
        }
    );

    // La zone de jeu montre les deux cartes.
    let played = play(
        &state,
        Action::PlayCard {
            card: concentration,
            play: boosting(ManaPayment::Die(0), march, 0, None, [None, None]),
        },
    );
    let screen = screen_of(&played, &catalog);
    assert_eq!(
        screen
            .play_area
            .iter()
            .map(|line| line.text.as_str())
            .collect::<Vec<_>>(),
        [
            "Concentration (renforcée, dé vert) : Renforcer une autre carte gratuitement, +2",
            "Marche (renforcée gratuitement) : Mouvement 6"
        ]
    );
}

// Spec 025 C29 — avec Source de mana : le dé, sa couleur, et un seul paiement
#[test]
fn concentration_with_mana_draw_sets_a_die() {
    let catalog = french();
    let (concentration, mana_draw) = (card(Concentration, 1), card(ManaDraw, 1));
    let state = state_with(
        &[concentration, mana_draw],
        &[basic(Green), basic(Blue), BLACK],
    );
    let screen = screen_in(
        &state,
        &ScreenMode::Boosting(boost_start(concentration)),
        &catalog,
    );
    let (tooltip, command) = boost_buttons(hand_card(&screen, "Source de mana"))[0];
    assert_eq!(
        tooltip,
        "Renforcer avec Concentration : Mettre 1 dé(s) sur une couleur, gagner des jetons"
    );
    let mode = boost_mode(command);
    let screen = screen_in(&state, &mode, &catalog);
    let prompt = screen.set_dice_prompt.as_ref().expect("bandeau");
    assert!(prompt.colors.is_empty());
    // Le dé vert paie Concentration : il n'est pas proposé.
    let picks: Vec<bool> = screen
        .source
        .dice
        .iter()
        .chain(&screen.source.depleted)
        .map(|die| die.pick.is_some())
        .collect();
    assert_eq!(picks, [false, true, true]);
    let pick = screen.source.dice[1].pick.expect("dé bleu cliquable");
    let screen = screen_in(&state, &boost_mode(pick), &catalog);
    let prompt = screen.set_dice_prompt.expect("bandeau");
    assert_eq!(
        labels(prompt.colors.iter().map(|b| &b.label)),
        ["rouge", "bleu", "blanc", "vert"]
    );
    assert_eq!(
        action_of(&state, prompt.colors[2].command),
        Action::PlayCard {
            card: concentration,
            play: boosting(
                ManaPayment::Die(0),
                mana_draw,
                0,
                None,
                [setting(1, White), None]
            ),
        }
    );
}

// Spec 025 C29 — sans autre carte à renforcer hors combat
#[test]
fn concentration_without_target_says_why() {
    let catalog = french();
    let (concentration, rage) = (card(Concentration, 1), card(Rage, 1));
    let state = state_with(&[concentration, rage], &[basic(Green), GOLD, BLACK]);
    let screen = screen_of(&state, &catalog);
    let button = &hand_card(&screen, "Concentration").strong_plays[0];
    assert!(!button.enabled);
    assert_eq!(
        button.tooltip.as_deref(),
        Some("Aucune autre carte à renforcer hors combat.")
    );
}

// Spec 025 C32
#[test]
fn every_mana_card_key_exists_in_french() {
    let catalog = french();
    let english = load("en");
    for catalog in [&catalog, &english] {
        let (concentration, crystallize, stamina) = (
            card(Concentration, 1),
            card(Crystallize, 1),
            card(Stamina, 1),
        );
        let state = with_token(
            with_crystals(
                state_with(
                    &[concentration, crystallize, stamina],
                    &[basic(Blue), BLACK, BLACK],
                ),
                Red,
                1,
            ),
            Red,
        );
        screen_of(&state, catalog);
        screen_in(
            &state,
            &ScreenMode::ChoosingBasicMana(ManaChoice {
                card: crystallize,
                option: 0,
                discard: None,
            }),
            catalog,
        );
        let state = play(
            &state,
            Action::PlayCard {
                card: stamina,
                play: strong(0, ManaPayment::Die(0)),
            },
        );
        screen_of(&state, catalog);
        // Partie B : dés de plus et dés mis sur une couleur.
        let (mana_draw, mana_pull) = (card(ManaDraw, 1), card(ManaPull, 1));
        let dice = with_crystals(
            state_of(
                Hero::Arythea,
                &[mana_pull, card(March, 1)],
                &[basic(White), GOLD, BLACK],
            ),
            White,
            1,
        );
        for mode in [
            setting_dice(mana_pull, [None, None], None),
            setting_dice(mana_pull, [setting(0, Red), None], Some(1)),
            ScreenMode::ChoosingSetDiceMana(SetDiceChoice {
                card: mana_pull,
                option: 0,
                settings: [setting(1, Red), setting(2, Blue)],
                die: None,
            }),
        ] {
            screen_in(&dice, &mode, catalog);
        }
        let drawn = with_crystals(
            state_with(&[mana_draw], &[basic(Green), GOLD, BLACK]),
            White,
            1,
        );
        let drawn = play(
            &drawn,
            Action::PlayCard {
                card: mana_draw,
                play: strong_set(ManaPayment::Crystal, [setting(1, Green), None]),
            },
        );
        screen_of(&drawn, catalog);
        // Partie C : Concentration renforcée, à chaque étape.
        let (concentration, improvisation, rage) = (
            card(Concentration, 1),
            card(Improvisation, 1),
            card(Rage, 1),
        );
        let boost = state_with(
            &[concentration, improvisation, rage],
            &[basic(Green), GOLD, BLACK],
        );
        let start = boost_start(concentration);
        let chosen = BoostChoice {
            target: Some(improvisation),
            ..start
        };
        for choice in [
            start,
            chosen,
            BoostChoice {
                discard: Some(rage),
                ..chosen
            },
        ] {
            screen_in(&boost, &ScreenMode::Boosting(choice), catalog);
        }
        screen_of(
            &state_with(&[concentration, rage], &[BLACK, BLACK, BLACK]),
            catalog,
        );
        let boosted = play(
            &boost,
            Action::PlayCard {
                card: concentration,
                play: boosting(
                    ManaPayment::Die(0),
                    improvisation,
                    0,
                    Some(rage),
                    [None, None],
                ),
            },
        );
        screen_of(&boosted, catalog);
        let (joy, _, _) = after_crystal_joy();
        screen_in(
            &joy,
            &ScreenMode::EndingTurn(EndTurnChoice::default()),
            catalog,
        );
        for hero in Hero::ALL {
            let deck = hero.starting_deck();
            screen_of(&state_of(hero, &deck[..5], &[BLACK, BLACK, BLACK]), catalog);
            screen_of(
                &state_of(hero, &deck[5..10], &[BLACK, BLACK, BLACK]),
                catalog,
            );
            screen_of(
                &state_of(hero, &deck[10..], &[BLACK, BLACK, BLACK]),
                catalog,
            );
        }
    }
    assert_eq!(catalog.gaps(), vec![]);
    assert_eq!(english.gaps(), vec![]);
}
