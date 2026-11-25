//! Scénarios de la spec 056 côté affichage (actions avancées : mana et cartes) : carte à détruire,
//! carte d'une offre, dé de Tempête de mana, cible d'Effet maximum, fenêtre de Rituel de sang,
//! bouton Apprendre, effets en cours et faits du tour.

use arcanum_core::{
    Action, AdvancedAction, BasicColor, BasicEffect, BoostTarget, CardId, CardPlay, DeedCard,
    Effect, Event, GameState, ManaColor, ManaPayment, Spell, TimeOfDay, apply, legal_actions, view,
};
use arcanum_i18n::{Catalog, Language};
use arcanum_ui::{
    BoostChoice, Command, DiscardChoice, GainChoice, GameScreen, HandCard, OffersWindow,
    ScreenMode, ScreenPhase, game_screen, offers_window_in, repository_assets_dir,
};

// L'arène des tests d'arcanum-core, partagée plutôt que recopiée.
#[path = "../../arcanum-core/tests/arene/mod.rs"]
mod arene;
use arene::*;

use AdvancedAction::*;
use arcanum_core::BasicAction::{Determination, Improvisation, March, Promise, Rage, Threaten};

// --- Outils ---

fn french() -> Catalog {
    Catalog::load(
        &repository_assets_dir().join("i18n"),
        &Language("fr".into()),
    )
    .expect("fr.json du dépôt")
}

/// Un décor de la spec 056, comme celui d'arcanum-core : le bac à sable (Arythea) de jour, sur la
/// plaine à l'O du village, la main, les dés de la Source, les offres d'actions avancées et de
/// sorts, puis des événements de plus.
#[derive(Clone)]
struct Decor {
    hand: Vec<CardId>,
    dice: Vec<ManaColor>,
    advanced: Vec<CardId>,
    spells: Vec<CardId>,
    extra: Vec<Event>,
}

impl Default for Decor {
    fn default() -> Self {
        Self {
            hand: Vec::new(),
            dice: Sandbox::default().dice,
            advanced: Vec::new(),
            spells: Vec::new(),
            extra: Vec::new(),
        }
    }
}

impl Decor {
    fn state(&self) -> GameState {
        let mut events = Sandbox {
            time: TimeOfDay::Day,
            figure: Some(PLAIN_W),
            hand: self.hand.clone(),
            dice: self.dice.clone(),
            ..Sandbox::default()
        }
        .events();
        if !self.advanced.is_empty() {
            events.extend(advanced_offer(&self.advanced));
        }
        if !self.spells.is_empty() {
            events.extend(spell_offer(&self.spells));
        }
        events.extend(self.extra.iter().cloned());
        events.iter().fold(GameState::default(), apply)
    }
}

fn crystal(color: BasicColor) -> Event {
    Event::CrystalGained {
        player: SOLO,
        color,
    }
}

fn spell(spell: Spell) -> CardId {
    CardId::new(DeedCard::Spell(spell), 1)
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

fn offers_in(state: &GameState, mode: &ScreenMode, catalog: &Catalog) -> OffersWindow {
    offers_window_in(
        &view(state, SOLO),
        &legal_actions(state, SOLO),
        mode,
        catalog,
    )
}

fn hand_card(screen: &GameScreen, card: CardId) -> &HandCard {
    screen
        .hand
        .iter()
        .find(|shown| shown.card == card)
        .expect("carte en main")
}

/// L'index de l'action dans les actions légales.
fn index_of(state: &GameState, action: &Action) -> usize {
    legal_actions(state, SOLO)
        .iter()
        .position(|legal| legal == action)
        .unwrap_or_else(|| panic!("action légale : {action:?}"))
}

/// L'action légale que joue `command`, si c'est `Command::Play`.
fn played(state: &GameState, command: Option<Command>) -> Option<Action> {
    match command {
        Some(Command::Play(index)) => legal_actions(state, SOLO).get(index).cloned(),
        _ => None,
    }
}

/// Le bouton « Prendre » ou « Apprendre » de cette carte de l'offre.
fn take_of(window: &OffersWindow, column: usize, name: &str) -> Option<(String, Command)> {
    window.columns[column]
        .cards
        .iter()
        .find(|card| card.name == name)
        .unwrap_or_else(|| panic!("carte de l'offre : {name}"))
        .take
        .as_ref()
        .map(|button| (button.label.clone(), button.command))
}

fn basic_play(
    card: CardId,
    discard: Option<CardId>,
    gain: Option<CardId>,
    die: Option<u8>,
) -> Action {
    Action::PlayCard {
        card,
        play: CardPlay::Basic {
            option: 0,
            discard,
            mana: None,
            with: None,
            gain,
            die,
        },
    }
}

/// L'option de l'effet renforcé de `card` dont l'effet est `effect`.
fn strong_option(card: CardId, effect: Effect) -> u8 {
    match card.card().definition().map(|definition| definition.strong) {
        Some(BasicEffect::Options { options, .. }) => (0u8..)
            .zip(options)
            .find(|(_, each)| **each == effect)
            .map(|(option, _)| option)
            .expect("option renforcée"),
        _ => panic!("carte sans effet renforcé"),
    }
}

const GREEN: ManaColor = ManaColor::Basic(BasicColor::Green);
const BLUE: ManaColor = ManaColor::Basic(BasicColor::Blue);

// --- Étape 11 : carte à détruire, carte d'offre, dé de la Source, cible d'Effet maximum ---

// Spec 056 C8, C6 ; MKUE p. 4, 6.a : la carte détruite se choisit dans la main, au mot près.
#[test]
fn decompose_chooses_the_card_to_destroy() {
    let catalog = french();
    let decompose = advanced(Decompose);
    let wounds = Decor {
        hand: vec![decompose, wound(1), wound(2)],
        ..Decor::default()
    }
    .state();
    let shown = screen_of(&wounds, &catalog);
    assert!(
        hand_card(&shown, decompose)
            .basic_effects
            .iter()
            .all(|button| button.command.is_none())
    );

    let rage = card(Rage, 1);
    let state = Decor {
        hand: vec![decompose, rage, card(March, 1)],
        ..Decor::default()
    }
    .state();
    let choice = DiscardChoice {
        card: decompose,
        option: 0,
    };
    assert_eq!(
        hand_card(&screen_of(&state, &catalog), decompose).basic_effects[0].command,
        Some(Command::ChooseDiscard {
            card: decompose,
            option: 0
        })
    );
    let screen = screen_in(&state, &ScreenMode::ChoosingDiscard(choice), &catalog);
    let prompt = screen
        .discard_prompt
        .clone()
        .expect("bandeau du choix")
        .text;
    assert!(prompt.contains("détruire"), "{prompt}");
    let pick = &hand_card(&screen, rage).plays[0];
    assert_eq!(pick.label, "Détruire cette carte");
    assert_eq!(
        pick.command,
        Command::Play(index_of(
            &state,
            &basic_play(decompose, Some(rage), None, None)
        ))
    );
}

fn training_state() -> GameState {
    Decor {
        hand: vec![
            advanced(Training),
            card(March, 1),
            card(Rage, 1),
            card(Determination, 1),
        ],
        advanced: vec![advanced(PathFinding), advanced(FireBolt), advanced(Agility)],
        extra: vec![
            crystal(BasicColor::Green),
            crystal(BasicColor::Red),
            crystal(BasicColor::Blue),
            crystal(BasicColor::White),
        ],
        ..Decor::default()
    }
    .state()
}

// Spec 056 C29 ; carte Entraînement ; Compendium, *Training* S3 : la carte détruite, puis l'action
// avancée de sa couleur dans la fenêtre des offres.
#[test]
fn training_chooses_destroyed_card_then_offer_card() {
    let catalog = french();
    let state = training_state();
    let training = advanced(Training);
    let (march, determination) = (card(March, 1), card(Determination, 1));
    let start = GainChoice {
        card: training,
        option: 0,
        strong: false,
        discard: None,
        gain: None,
    };
    assert_eq!(
        hand_card(&screen_of(&state, &catalog), training).basic_effects[0].command,
        Some(Command::ChooseGain(start))
    );
    let screen = screen_in(&state, &ScreenMode::ChoosingGain(start), &catalog);
    let chosen = GainChoice {
        discard: Some(march),
        ..start
    };
    let pick = &hand_card(&screen, march).plays[0];
    assert_eq!(pick.label, "Détruire cette carte");
    assert_eq!(pick.command, Command::ChooseGain(chosen));
    assert!(hand_card(&screen, determination).plays.is_empty());
    assert!(screen.discard_prompt.is_some());

    let window = offers_in(&state, &ScreenMode::ChoosingGain(chosen), &catalog);
    let path_finding = advanced(PathFinding);
    assert_eq!(
        take_of(&window, 0, &catalog.text("card.path_finding.name")).map(|(_, command)| command),
        Some(Command::Play(index_of(
            &state,
            &basic_play(training, Some(march), Some(path_finding), None)
        )))
    );
    assert_eq!(
        take_of(&window, 0, &catalog.text("card.fire_bolt.name")),
        None
    );
    assert_eq!(
        take_of(&window, 0, &catalog.text("card.agility.name")),
        None
    );
}

// Spec 056 C30 ; carte Entraînement renforcée : la carte d'offre, puis le mana.
#[test]
fn strong_training_chooses_offer_card_then_mana() {
    let catalog = french();
    let state = training_state();
    let training = advanced(Training);
    let rage = card(Rage, 1);
    let shown = screen_of(&state, &catalog);
    let start = GainChoice {
        card: training,
        option: 0,
        strong: true,
        discard: None,
        gain: None,
    };
    assert_eq!(
        hand_card(&shown, training).strong_effects[0].command,
        Some(Command::ChooseGain(start))
    );
    let chosen = GainChoice {
        discard: Some(rage),
        ..start
    };
    let window = offers_in(&state, &ScreenMode::ChoosingGain(chosen), &catalog);
    let fire_bolt = advanced(FireBolt);
    let with_gain = GainChoice {
        gain: Some(fire_bolt),
        ..chosen
    };
    assert_eq!(
        take_of(&window, 0, &catalog.text("card.fire_bolt.name")).map(|(_, command)| command),
        Some(Command::ChooseGain(with_gain))
    );
    let screen = screen_in(&state, &ScreenMode::ChoosingGain(with_gain), &catalog);
    let prompt = screen.mana_prompt.expect("choix du mana");
    assert!(prompt.payments.len() >= 2);
    for button in &prompt.payments {
        let action = played(&state, Some(button.command));
        assert!(
            matches!(
                action,
                Some(Action::PlayCard {
                    card,
                    play: CardPlay::Strong {
                        discard: Some(destroyed),
                        gain: Some(gained),
                        ..
                    },
                }) if card == training && destroyed == rage && gained == fire_bolt
            ),
            "{action:?}"
        );
    }
}

// Spec 056 C24 ; carte Don pour la magie renforcée ; Compendium, *Magic Talent* O4 : le sort de
// la couleur payée, choisi dans la fenêtre des offres.
#[test]
fn strong_magic_talent_chooses_spell_in_offers() {
    let catalog = french();
    let talent = advanced(MagicTalent);
    let state = Decor {
        hand: vec![talent],
        spells: vec![
            spell(Spell::Fireball),
            spell(Spell::Snowstorm),
            spell(Spell::Tremor),
        ],
        extra: vec![crystal(BasicColor::Red)],
        ..Decor::default()
    }
    .state();
    let shown = screen_of(&state, &catalog);
    let card_shown = hand_card(&shown, talent);
    let red = strong_option(talent, Effect::GainSpell(BasicColor::Red));
    let white = strong_option(talent, Effect::GainSpell(BasicColor::White));
    let start = GainChoice {
        card: talent,
        option: red,
        strong: true,
        discard: None,
        gain: None,
    };
    assert_eq!(
        card_shown.strong_effects[usize::from(red)].command,
        Some(Command::ChooseGain(start))
    );
    assert_eq!(card_shown.strong_effects[usize::from(white)].command, None);

    let window = offers_in(&state, &ScreenMode::ChoosingGain(start), &catalog);
    let fireball =
        take_of(&window, 1, &spell_name(Spell::Fireball, &catalog)).map(|(_, command)| command);
    let action = played(&state, fireball);
    assert!(
        matches!(
            action,
            Some(Action::PlayCard {
                play: CardPlay::Strong { gain: Some(gained), .. },
                ..
            }) if gained == spell(Spell::Fireball)
        ),
        "{fireball:?}"
    );
    assert_eq!(
        take_of(&window, 1, &spell_name(Spell::Snowstorm, &catalog)),
        None
    );
}

/// Spec #32 : la carte d'un sort de l'offre porte son nom de base (test modifié).
fn spell_name(spell: Spell, catalog: &Catalog) -> String {
    catalog.text(spell.label_key())
}

// Spec 056 C13 ; carte Tempête de mana : le dé se choisit dans la Source, seuls les dés de couleur
// de base cliquables.
#[test]
fn mana_storm_chooses_a_source_die() {
    let catalog = french();
    let storm = advanced(ManaStorm);
    let state = Decor {
        hand: vec![storm],
        dice: vec![GREEN, BLUE, ManaColor::Black],
        ..Decor::default()
    }
    .state();
    assert_eq!(
        hand_card(&screen_of(&state, &catalog), storm).basic_effects[0].command,
        Some(Command::ChooseDie {
            card: storm,
            option: 0
        })
    );
    let mode = ScreenMode::ChoosingDie(DiscardChoice {
        card: storm,
        option: 0,
    });
    let screen = screen_in(&state, &mode, &catalog);
    assert_eq!(
        screen.source.dice[0].pick,
        Some(Command::Play(index_of(
            &state,
            &basic_play(storm, None, None, Some(0))
        )))
    );
    // De jour, le dé noir est épuisé (MKUE p. 5, 3.c) : à part, jamais cliquable.
    assert!(screen.source.dice[1].pick.is_some());
    assert!(screen.source.depleted.iter().all(|die| die.pick.is_none()));
    assert!(screen.set_dice_prompt.is_some());
}

fn maximal_start(card: CardId, basic: bool) -> BoostChoice {
    BoostChoice {
        card,
        option: 0,
        basic,
        target: None,
        target_option: 0,
        discard: None,
        settings: [None, None],
        die: None,
    }
}

// Spec 056 C22, C23 ; carte Effet maximum : la cible se choisit dans la main avec son effet de
// base répété ; Improvisation n'est pas proposée (Q1).
#[test]
fn maximal_effect_chooses_target_in_hand() {
    let catalog = french();
    let maximal = advanced(MaximalEffect);
    let (march, improvisation) = (card(March, 1), card(Improvisation, 1));
    let state = Decor {
        hand: vec![maximal, march, improvisation, card(Rage, 1)],
        ..Decor::default()
    }
    .state();
    let start = maximal_start(maximal, true);
    assert_eq!(
        hand_card(&screen_of(&state, &catalog), maximal).basic_effects[0].command,
        Some(Command::ChooseBoost(start))
    );
    let screen = screen_in(&state, &ScreenMode::Boosting(start), &catalog);
    assert!(screen.boost_prompt.is_some());
    let target = hand_card(&screen, march)
        .basic_effects
        .iter()
        .find(|button| button.command.is_some())
        .expect("Marche proposée");
    assert!(target.fallback_text().contains('6'), "{target:?}");
    let action = Action::PlayCard {
        card: maximal,
        play: CardPlay::Basic {
            option: 0,
            discard: None,
            mana: None,
            with: Some(BoostTarget {
                card: march,
                option: 0,
                discard: None,
                set_dice: [None, None],
            }),
            gain: None,
            die: None,
        },
    };
    assert_eq!(
        target.command,
        Some(Command::Play(index_of(&state, &action)))
    );
    let held = hand_card(&screen, improvisation);
    assert!(
        held.basic_effects
            .iter()
            .chain(&held.strong_effects)
            .all(|button| button.command.is_none())
    );
}

// Spec 056 C18 ; carte Effet maximum renforcée : l'effet renforcé de la cible deux fois.
#[test]
fn strong_maximal_effect_doubles_strong_effect_of_target() {
    let catalog = french();
    let maximal = advanced(MaximalEffect);
    let march = card(March, 1);
    let state = Decor {
        hand: vec![maximal, march],
        ..Decor::default()
    }
    .state();
    let start = maximal_start(maximal, false);
    assert_eq!(
        hand_card(&screen_of(&state, &catalog), maximal).strong_effects[0].command,
        Some(Command::ChooseBoost(start))
    );
    let screen = screen_in(&state, &ScreenMode::Boosting(start), &catalog);
    let target = hand_card(&screen, march)
        .strong_effects
        .iter()
        .find(|button| button.command.is_some())
        .expect("Marche proposée");
    assert!(target.fallback_text().contains('8'), "{target:?}");
}

// --- Étape 12 : Rituel de sang, Apprendre, effets en cours, faits du tour ---

fn strong_ritual_state() -> GameState {
    let ritual = advanced(BloodRitual);
    let state = Decor {
        hand: vec![ritual],
        ..Decor::default()
    }
    .state();
    done(
        &state,
        Action::PlayCard {
            card: ritual,
            play: CardPlay::Strong {
                option: 0,
                discard: None,
                mana: ManaPayment::Die(RED_DIE),
                set_dice: [None, None],
                with: None,
                extra_mana: None,
                gain: None,
            },
        },
    )
}

// Spec 056 C4, C5 ; carte Rituel de sang renforcée ; Compendium, *Blood Ritual* S1 : six couleurs
// de jeton, trois fois, puis le cristal ou « Non merci ».
#[test]
fn strong_blood_ritual_window() {
    let catalog = french();
    let state = strong_ritual_state();
    let window = screen_of(&state, &catalog)
        .blood_ritual
        .expect("choix de couleur proposé");
    assert!(window.title.contains('3'), "{}", window.title);
    let commands: Vec<Command> = window.tokens.iter().map(|button| button.command).collect();
    let expected: Vec<Command> = ManaColor::ALL
        .iter()
        .map(|&color| Command::Play(index_of(&state, &Action::ChooseTokenColor { color })))
        .collect();
    assert_eq!(commands, expected);
    assert!(window.payments.is_empty() && window.decline.is_none());

    let choose = |state: &GameState, color| done(state, Action::ChooseTokenColor { color });
    let state = choose(&state, GREEN);
    let state = choose(&state, GREEN);
    let state = choose(&state, ManaColor::Gold);
    let window = screen_of(&state, &catalog)
        .blood_ritual
        .expect("cristal proposé");
    assert!(window.tokens.is_empty());
    assert!(!window.payments.is_empty());
    for button in &window.payments {
        assert!(matches!(
            played(&state, Some(button.command)),
            Some(Action::PayForCrystal { .. })
        ));
    }
    let decline = window.decline.expect("Non merci");
    assert_eq!(decline.label, "Non merci");
    assert_eq!(
        decline.command,
        Command::Play(index_of(&state, &Action::DeclineCrystal))
    );
    let state = done(&state, Action::DeclineCrystal);
    assert_eq!(screen_of(&state, &catalog).blood_ritual, None);
}

// Issue privée 234 : l'effet de base de Rituel de sang (une option par couleur de jeton) se lit par
// notre texte écrit, pas par les six options énumérées. Carte Rituel de sang (*Lost Legion*).
#[test]
fn blood_ritual_basic_effect_uses_written_text() {
    let catalog = french();
    let ritual = advanced(BloodRitual);
    let state = Decor {
        hand: vec![ritual],
        ..Decor::default()
    }
    .state();
    let screen = screen_of(&state, &catalog);
    assert_eq!(
        hand_card(&screen, ritual).effect,
        catalog.text("card.blood_ritual.basic_text")
    );
}

fn learning_state() -> GameState {
    let learning = advanced(Learning);
    let (promise, threaten) = (card(Promise, 1), card(Threaten, 1));
    let state = Decor {
        hand: vec![learning, promise, threaten],
        advanced: vec![advanced(PathFinding), advanced(FireBolt), advanced(Agility)],
        ..Decor::default()
    }
    .state();
    let state = done(&state, basic(learning));
    let state = done(&state, basic(promise));
    done(&state, basic(threaten))
}

// Spec 056 C26, C28 ; carte Apprentissage : « Apprendre (6 Influence) » sur les cartes de l'offre
// d'actions avancées, une fois.
#[test]
fn learning_button_on_advanced_actions_offer() {
    let catalog = french();
    let state = learning_state();
    let window = offers_in(&state, &ScreenMode::Normal, &catalog);
    let path_finding = advanced(PathFinding);
    assert_eq!(
        take_of(&window, 0, &catalog.text("card.path_finding.name")),
        Some((
            "Apprendre (6 Influence)".to_string(),
            Command::Play(index_of(
                &state,
                &Action::LearnAdvancedAction { card: path_finding }
            ))
        ))
    );
    let state = done(&state, Action::LearnAdvancedAction { card: path_finding });
    let window = offers_in(&state, &ScreenMode::Normal, &catalog);
    assert!(
        window.columns[0]
            .cards
            .iter()
            .all(|card| card.take.is_none())
    );
}

// Spec 056 C10, C33 ; FAQ p2 : Maîtrise du cristal renforcée dans les effets en cours, avec les
// cristaux qui reviendront.
#[test]
fn crystal_mastery_in_progress_with_crystals_to_return() {
    let catalog = french();
    let mastery = advanced(CrystalMastery);
    let state = Decor {
        hand: vec![mastery],
        extra: vec![crystal(BasicColor::Blue)],
        ..Decor::default()
    }
    .state();
    let state = done(
        &state,
        Action::PlayCard {
            card: mastery,
            play: CardPlay::Strong {
                option: 0,
                discard: None,
                mana: ManaPayment::Crystal,
                set_dice: [None, None],
                with: None,
                extra_mana: None,
                gain: None,
            },
        },
    );
    let lines = screen_of(&state, &catalog)
        .modifiers
        .expect("effets en cours")
        .lines;
    let line = lines
        .iter()
        .find(|line| line.contains("cristaux dépensés rendus"))
        .expect("Maîtrise du cristal");
    assert!(line.contains("bleu"), "{line}");
}

// Spec 056 C6 : la carte détruite dans les faits du tour.
#[test]
fn destroyed_card_in_turn_facts() {
    let catalog = french();
    let decompose = advanced(Decompose);
    let rage = card(Rage, 1);
    let state = Decor {
        hand: vec![decompose, rage],
        ..Decor::default()
    }
    .state();
    let state = done(&state, basic_play(decompose, Some(rage), None, None));
    let facts = screen_of(&state, &catalog).turn_facts;
    assert!(
        facts.iter().any(|fact| fact == "Rage détruite"),
        "{facts:?}"
    );
}

// Spec 056 C6 ; MKUE p. 5, *Discarding and Throwing Away* 2.b : la zone de jeu dit la carte
// détruite, pas défaussée (vu à l'œil, étape 13).
#[test]
fn play_area_says_destroyed_card() {
    let catalog = french();
    let decompose = advanced(Decompose);
    let rage = card(Rage, 1);
    let state = Decor {
        hand: vec![decompose, rage],
        ..Decor::default()
    }
    .state();
    let state = done(&state, basic_play(decompose, Some(rage), None, None));
    let line = &screen_of(&state, &catalog).play_area[0].text;
    assert!(line.ends_with("Rage détruite"), "{line}");
}

/// Effet maximum joué sur Marche, de base ou renforcé (payé par le dé rouge).
fn maximal_on_march(strong: bool) -> GameState {
    let maximal = advanced(MaximalEffect);
    let march = card(March, 1);
    let state = Decor {
        hand: vec![maximal, march],
        ..Decor::default()
    }
    .state();
    let with = Some(BoostTarget {
        card: march,
        option: 0,
        discard: None,
        set_dice: [None, None],
    });
    let play = if strong {
        CardPlay::Strong {
            option: 0,
            discard: None,
            mana: ManaPayment::Die(RED_DIE),
            set_dice: [None, None],
            with,
            extra_mana: None,
            gain: None,
        }
    } else {
        CardPlay::Basic {
            option: 0,
            discard: None,
            mana: None,
            with,
            gain: None,
            die: None,
        }
    };
    done(
        &state,
        Action::PlayCard {
            card: maximal,
            play,
        },
    )
}

// Issue privée 235 ; spec 056 C22 ; MKUE p. 5, *"Composite" Effects* 1.a et 1.b : la ligne d'Effet
// maximum nomme la cible détruite et l'effet obtenu (Marche ×3 : Mouvement 6).
#[test]
fn play_area_names_maximal_effect_target_and_effect() {
    let catalog = french();
    let screen = screen_of(&maximal_on_march(false), &catalog);
    let [line] = screen.play_area.as_slice() else {
        panic!("une seule ligne : {:?}", screen.play_area);
    };
    assert_eq!(
        line.text,
        "Effet maximum : Marche 3 fois (Mouvement 6), détruite"
    );
}

// Issue privée 235 ; spec 056 C18 : renforcé, l'effet renforcé de la cible deux fois (Mouvement 8).
#[test]
fn play_area_names_strong_maximal_effect_target_and_effect() {
    let catalog = french();
    let screen = screen_of(&maximal_on_march(true), &catalog);
    let line = &screen.play_area[0].text;
    assert!(
        line.ends_with(": Marche 2 fois (Mouvement 8), détruite"),
        "{line}"
    );
}
