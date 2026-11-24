//! Scénarios de la spec 025 (cartes de mana) : jetons et cristaux (partie A), dés de plus et dés mis
//! sur une couleur (partie B), Concentration renforcée (partie C).

use arcanum_core::{
    Action, BasicAction, BasicColor, BasicEffect, BoostTarget, CRYSTAL_LIMIT, CardId, CardPlay,
    Cost, DICE_PER_TURN, DeedCard, DieSetting, Effect, Event, GameConfig, GameRngs, GameState,
    Hero, Hex, ManaColor, ManaPayment, PlayerEvent, PlayerId, Scenario, SidewaysEffect, TileId,
    TilePosition, apply, decide, filter_event, legal_actions, view,
};
use rand::SeedableRng;
use rand::seq::IndexedRandom;
use rand_chacha::ChaCha8Rng;

const SOLO: PlayerId = PlayerId::SOLO;

use BasicAction::*;
use BasicColor::*;

const GOLD: ManaColor = ManaColor::Gold;
const BLACK: ManaColor = ManaColor::Black;

fn basic(color: BasicColor) -> ManaColor {
    ManaColor::Basic(color)
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

/// Partie de bac à sable construite par événements, de jour : `hero`, `hand` en main (le reste de
/// son deck dessous), figurine au centre de la tuile de départ, Source `dice`.
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

/// Les cartes du deck standard (Concentration, Cristallisation…) : Tovak les a toutes.
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
            color: arcanum_core::ManaColor::Basic(color),
        },
    )
}

fn basic_play(option: u8) -> CardPlay {
    CardPlay::Basic {
        die: None,
        gain: None,
        with: None,
        option,
        discard: None,
        mana: None,
    }
}

fn paid(option: u8, mana: ManaPayment) -> CardPlay {
    CardPlay::Basic {
        die: None,
        gain: None,
        with: None,
        option,
        discard: None,
        mana: Some(mana),
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

/// Concentration renforcée (option 0), payée par `mana`, avec l'option `option` de `target`.
fn boosting(mana: ManaPayment, target: CardId, option: u8, discard: Option<CardId>) -> CardPlay {
    boosting_set(mana, target, option, discard, [None, None])
}

fn boosting_set(
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

/// La carte jouée avec Concentration, telle qu'elle entre dans la zone de jeu.
fn boosted(option: u8, discard: Option<CardId>) -> CardPlay {
    CardPlay::Boosted {
        option,
        discard,
        set_dice: [None, None],
    }
}

/// Effet renforcé de Source de mana ou d'Attraction de mana : les dés mis et leur couleur.
fn strong_set(mana: ManaPayment, settings: &[(u8, BasicColor)]) -> CardPlay {
    let mut set_dice = [None, None];
    for (slot, &(die, color)) in set_dice.iter_mut().zip(settings) {
        *slot = Some(DieSetting { die, color });
    }
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

fn die_set(die: u8, color: BasicColor) -> Event {
    Event::DieSet {
        player: SOLO,
        die,
        color,
    }
}

/// Les paiements proposés pour l'effet renforcé `option` de `of`.
fn strong_payments(state: &GameState, of: CardId) -> Vec<ManaPayment> {
    plays_of(state, of)
        .into_iter()
        .filter_map(|play| match play {
            CardPlay::Strong { mana, .. } => Some(mana),
            _ => None,
        })
        .collect()
}

fn play(card: CardId, play: CardPlay) -> Action {
    Action::PlayCard { card, play }
}

fn end_turn(discard: Vec<CardId>, reclaim: Option<CardId>) -> Action {
    Action::EndTurn {
        discard,
        reclaim,
        glade: None,
        steady_tempo: false,
    }
}

fn events_with_seed(state: &GameState, action: &Action, seed: u64) -> Vec<Event> {
    decide(state, SOLO, action, &mut config(seed, Hero::Tovak).rngs())
        .unwrap_or_else(|error| panic!("{action:?} doit être légale : {error:?}"))
}

fn events_of(state: &GameState, action: &Action) -> Vec<Event> {
    events_with_seed(state, action, 0)
}

fn done(state: &GameState, action: Action) -> GameState {
    events_of(state, &action).iter().fold(state.clone(), apply)
}

fn is_legal(state: &GameState, action: &Action) -> bool {
    legal_actions(state, SOLO).contains(action)
}

fn refused(state: &GameState, action: &Action) -> bool {
    decide(state, SOLO, action, &mut config(0, Hero::Tovak).rngs()).is_err()
}

/// Les façons de jouer `of` proposées, hors de côté.
fn plays_of(state: &GameState, of: CardId) -> Vec<CardPlay> {
    legal_actions(state, SOLO)
        .into_iter()
        .filter_map(|action| match action {
            Action::PlayCard { card, play } if card == of => Some(play),
            _ => None,
        })
        .filter(|play| !matches!(play, CardPlay::Sideways(_)))
        .collect()
}

fn played(card: CardId, play: CardPlay, effect: Effect) -> Event {
    Event::CardPlayed {
        player: SOLO,
        card,
        play,
        effect,
    }
}

fn token_gained(color: BasicColor) -> Event {
    Event::ManaTokenGained {
        player: SOLO,
        color: arcanum_core::ManaColor::Basic(color),
    }
}

fn crystal_gained(color: BasicColor) -> Event {
    Event::CrystalGained {
        player: SOLO,
        color,
    }
}

// --- Contenu ---

// Spec 025 C1 — effets lus sur les scans (../mage-knight/docs/regles/scans/deed/)
#[test]
fn mana_card_definitions_match_cards() {
    let options = |options: &'static [Effect]| BasicEffect::Options {
        options,
        cost: Cost::None,
        modifiers: &[],
        mana: &[],
    };
    const TOKENS: &[Effect] = &[
        Effect::GainToken(Blue),
        Effect::GainToken(White),
        Effect::GainToken(Red),
    ];
    const WILL_FOCUS: &[Effect] = &[
        Effect::GainToken(Blue),
        Effect::GainToken(White),
        Effect::GainToken(Red),
        Effect::GainCrystal(Green),
    ];
    const PAY_FOR_CRYSTAL: &[Effect] = &[
        Effect::PayForCrystal(Red),
        Effect::PayForCrystal(Blue),
        Effect::PayForCrystal(White),
        Effect::PayForCrystal(Green),
    ];
    const ANY_CRYSTAL: &[Effect] = &[
        Effect::GainCrystal(Red),
        Effect::GainCrystal(Blue),
        Effect::GainCrystal(White),
        Effect::GainCrystal(Green),
    ];
    let cases: Vec<(BasicAction, BasicColor, BasicEffect, BasicEffect, bool)> = vec![
        (
            Concentration,
            Green,
            options(TOKENS),
            options(&[Effect::Boost(2)]),
            false,
        ),
        (
            WillFocus,
            Green,
            options(WILL_FOCUS),
            options(&[Effect::Boost(3)]),
            false,
        ),
        (
            Crystallize,
            Blue,
            options(PAY_FOR_CRYSTAL),
            options(ANY_CRYSTAL),
            false,
        ),
        (
            CrystalJoy,
            Blue,
            options(PAY_FOR_CRYSTAL),
            options(ANY_CRYSTAL),
            true,
        ),
        // Soin et pioche (spec 034) avant le mana, dans l'ordre de la carte ; puis rendre
        // disponible une unité (spec 042).
        (
            Rejuvenate,
            Green,
            options(&[
                Effect::Heal(1),
                Effect::DrawCards(1),
                Effect::GainToken(Green),
                Effect::ReadyUnit(2),
            ]),
            options(&[
                Effect::Heal(2),
                Effect::DrawCards(2),
                Effect::GainCrystal(Green),
                Effect::ReadyUnit(3),
            ]),
            false,
        ),
        (
            ManaDraw,
            White,
            options(&[Effect::ExtraDie]),
            options(&[Effect::SetDice { dice: 1, tokens: 2 }]),
            false,
        ),
        (
            ManaPull,
            White,
            options(&[Effect::ExtraDieBlackAnyColor]),
            options(&[Effect::SetDice { dice: 2, tokens: 1 }]),
            false,
        ),
    ];
    for (action, color, basic_effect, strong_effect, reclaim) in cases {
        let definition = action.definition();
        assert_eq!(definition.color, color, "{action:?}");
        assert_eq!(definition.basic, basic_effect, "{action:?}");
        assert_eq!(definition.strong, strong_effect, "{action:?}");
        assert_eq!(definition.reclaim, reclaim, "{action:?}");
    }
    // Seule Euphorie cristalline se reprend.
    for action in BasicAction::ALL {
        assert_eq!(
            action.definition().reclaim,
            action == CrystalJoy,
            "{action:?}"
        );
    }
}

// --- A. Jetons et cristaux ---

// Spec 025 C2
#[test]
fn concentration_gains_a_mana_token() {
    // MKUE p. 5, « Gain » 1
    let concentration = card(Concentration, 1);
    let state = state_with(&[concentration], &[basic(Green), GOLD, BLACK]);
    assert_eq!(
        plays_of(&state, concentration),
        vec![basic_play(0), basic_play(1), basic_play(2)]
    );
    let action = play(concentration, basic_play(0));
    let events = events_of(&state, &action);
    assert_eq!(
        events,
        vec![
            played(concentration, basic_play(0), Effect::GainToken(Blue)),
            token_gained(Blue),
        ]
    );
    let state = events.iter().fold(state, apply);
    assert_eq!(
        state.mana_tokens(SOLO),
        &[arcanum_core::ManaColor::Basic(Blue)][..]
    );
    assert_eq!(
        view(&state, SOLO).mana_tokens,
        vec![arcanum_core::ManaColor::Basic(Blue)]
    );
    assert_eq!(
        (state.move_points(SOLO), state.influence_points(SOLO)),
        (0, 0)
    );
}

// Spec 025 C3
#[test]
fn mana_token_pays_a_strong_effect() {
    // MKUE p. 5, « Using mana » 1
    let stamina = card(Stamina, 1);
    let state = with_token(state_with(&[stamina], &[basic(Red), BLACK, BLACK]), Blue);
    assert_eq!(
        plays_of(&state, stamina),
        vec![basic_play(0), strong(0, ManaPayment::Token)]
    );
    let action = play(stamina, strong(0, ManaPayment::Token));
    let events = events_of(&state, &action);
    assert_eq!(
        events,
        vec![
            Event::ManaTokenUsed {
                player: SOLO,
                color: arcanum_core::ManaColor::Basic(Blue)
            },
            played(stamina, strong(0, ManaPayment::Token), Effect::Move(4)),
        ]
    );
    let state = events.iter().fold(state, apply);
    assert!(state.mana_tokens(SOLO).is_empty());
    assert_eq!(state.move_points(SOLO), 4);
}

// Spec 025 C4
#[test]
fn mana_token_is_not_a_die() {
    // MKUE p. 5, 3 : la limite ne vaut que pour les dés.
    let (march, stamina) = (card(March, 1), card(Stamina, 1));
    let state = with_token(
        state_with(&[march, stamina], &[basic(Green), GOLD, BLACK]),
        Blue,
    );
    let state = done(&state, play(march, strong(0, ManaPayment::Die(0))));
    assert_eq!(state.dice_taken(SOLO).len(), DICE_PER_TURN);
    assert!(is_legal(
        &state,
        &play(stamina, strong(0, ManaPayment::Token))
    ));
    assert!(!is_legal(
        &state,
        &play(stamina, strong(0, ManaPayment::Die(1)))
    ));
}

// Spec 025 C5
#[test]
fn crystallize_turns_mana_into_a_crystal() {
    // Scan de Cristallisation ; MKUE p. 4, 6 : on ne joue l'effet que si l'on paie.
    let crystallize = card(Crystallize, 1);
    let state = state_with(&[crystallize], &[basic(Green), BLACK, basic(Red)]);
    // Rouge (dé 2), bleu et blanc sans mana, vert (dé 0) ; renforcé : bleu, impossible.
    assert_eq!(
        plays_of(&state, crystallize),
        vec![paid(0, ManaPayment::Die(2)), paid(3, ManaPayment::Die(0)),]
    );
    let action = play(crystallize, paid(3, ManaPayment::Die(0)));
    let events = events_of(&state, &action);
    assert_eq!(
        events,
        vec![
            Event::DieTaken {
                player: SOLO,
                die: 0,
                color: basic(Green)
            },
            played(
                crystallize,
                paid(3, ManaPayment::Die(0)),
                Effect::PayForCrystal(Green)
            ),
            crystal_gained(Green),
        ]
    );
    let state = events.iter().fold(state, apply);
    assert_eq!(state.inventory(SOLO).crystals(Green), 1);
    assert_eq!(state.dice_taken(SOLO), &[0][..]);
}

// Spec 025 C5 : un dé or paie n'importe quelle couleur de base, de jour.
#[test]
fn crystallize_with_gold_offers_every_color() {
    let crystallize = card(Crystallize, 1);
    let state = state_with(&[crystallize], &[GOLD, BLACK, BLACK]);
    assert_eq!(
        plays_of(&state, crystallize),
        vec![
            paid(0, ManaPayment::Die(0)),
            paid(1, ManaPayment::Die(0)),
            paid(2, ManaPayment::Die(0)),
            paid(3, ManaPayment::Die(0)),
            strong(0, ManaPayment::Die(0)),
            strong(1, ManaPayment::Die(0)),
            strong(2, ManaPayment::Die(0)),
            strong(3, ManaPayment::Die(0)),
        ]
    );
}

// Spec 025 C6
#[test]
fn crystallize_keeps_a_token_as_crystal() {
    // MKUE p. 5, 1.b
    let crystallize = card(Crystallize, 1);
    let state = with_token(state_with(&[crystallize], &[BLACK, BLACK, BLACK]), Red);
    assert_eq!(
        plays_of(&state, crystallize),
        vec![paid(0, ManaPayment::Token)]
    );
    let events = events_of(&state, &play(crystallize, paid(0, ManaPayment::Token)));
    assert_eq!(
        events,
        vec![
            Event::ManaTokenUsed {
                player: SOLO,
                color: arcanum_core::ManaColor::Basic(Red)
            },
            played(
                crystallize,
                paid(0, ManaPayment::Token),
                Effect::PayForCrystal(Red)
            ),
            crystal_gained(Red),
        ]
    );
    let state = events.iter().fold(state, apply);
    assert!(state.mana_tokens(SOLO).is_empty());
    assert_eq!(state.inventory(SOLO).crystals(Red), 1);
}

// Spec 025 C7
#[test]
fn fourth_crystal_becomes_a_token() {
    // MKUE p. 5, « Gain » 2
    let crystallize = card(Crystallize, 1);
    let state = with_crystals(
        with_crystals(state_with(&[crystallize], &[BLACK, BLACK, BLACK]), Red, 3),
        Blue,
        1,
    );
    let strong_plays: Vec<CardPlay> = plays_of(&state, crystallize)
        .into_iter()
        .filter(|play| matches!(play, CardPlay::Strong { .. }))
        .collect();
    assert_eq!(
        strong_plays,
        (0..4)
            .map(|option| strong(option, ManaPayment::Crystal))
            .collect::<Vec<_>>()
    );
    let events = events_of(&state, &play(crystallize, strong(0, ManaPayment::Crystal)));
    assert_eq!(
        events,
        vec![
            Event::CrystalUsed {
                player: SOLO,
                color: Blue
            },
            played(
                crystallize,
                strong(0, ManaPayment::Crystal),
                Effect::GainCrystal(Red)
            ),
            token_gained(Red),
        ]
    );
    let state = events.iter().fold(state, apply);
    assert_eq!(state.inventory(SOLO).crystals(Red), CRYSTAL_LIMIT);
    assert_eq!(
        state.mana_tokens(SOLO),
        &[arcanum_core::ManaColor::Basic(Red)][..]
    );
}

// Spec 025 C8
#[test]
fn will_focus_and_rejuvenate_mana_options() {
    let will_focus = card(WillFocus, 1);
    let state = state_of(Hero::Goldyx, &[will_focus], &[BLACK, BLACK, BLACK]);
    assert_eq!(
        plays_of(&state, will_focus),
        (0..4).map(basic_play).collect::<Vec<_>>()
    );
    assert_eq!(
        events_of(&state, &play(will_focus, basic_play(3))),
        vec![
            played(will_focus, basic_play(3), Effect::GainCrystal(Green)),
            crystal_gained(Green),
        ]
    );

    let rejuvenate = card(Rejuvenate, 1);
    let state = state_of(Hero::Norowas, &[rejuvenate], &[basic(Green), BLACK, BLACK]);
    // Spec 034 : Soin et piocher passent devant, l'option de mana devient la troisième (2).
    assert_eq!(
        plays_of(&state, rejuvenate),
        vec![
            basic_play(0),
            basic_play(1),
            basic_play(2),
            strong(0, ManaPayment::Die(0)),
            strong(1, ManaPayment::Die(0)),
            strong(2, ManaPayment::Die(0)),
        ]
    );
    assert_eq!(
        events_of(&state, &play(rejuvenate, basic_play(2))),
        vec![
            played(rejuvenate, basic_play(2), Effect::GainToken(Green)),
            token_gained(Green),
        ]
    );
    assert_eq!(
        events_of(&state, &play(rejuvenate, strong(2, ManaPayment::Die(0))))[1..],
        [
            played(
                rejuvenate,
                strong(2, ManaPayment::Die(0)),
                Effect::GainCrystal(Green)
            ),
            crystal_gained(Green),
        ]
    );
}

/// Goldyx a joué Euphorie cristalline (effet de base, payé par un cristal vert) ; en main : Marche
/// et Endurance.
fn after_crystal_joy() -> (GameState, CardId, CardId, CardId) {
    let (joy, march, stamina) = (card(CrystalJoy, 1), card(March, 1), card(Stamina, 1));
    let state = with_crystals(
        state_of(Hero::Goldyx, &[joy, march, stamina], &[BLACK, BLACK, BLACK]),
        Green,
        1,
    );
    let state = done(&state, play(joy, paid(3, ManaPayment::Crystal)));
    (state, joy, march, stamina)
}

// Spec 025 C9
#[test]
fn crystal_joy_comes_back_to_hand() {
    let (state, joy, march, stamina) = after_crystal_joy();
    let action = end_turn(vec![], Some(march));
    let events = events_of(&state, &action);
    assert_eq!(
        events[..3],
        [
            Event::CardDiscarded {
                player: SOLO,
                card: march
            },
            Event::CardReturnedToHand {
                player: SOLO,
                card: joy
            },
            Event::TurnEnded { player: SOLO },
        ]
    );
    let drawn = |events: &[Event]| {
        events
            .iter()
            .filter(|event| matches!(event, Event::CardDrawn { .. }))
            .count()
    };
    // Main gardée : Endurance et Euphorie ; sans reprise, en défaussant Marche : Endurance seule.
    assert_eq!(drawn(&events), 3);
    assert_eq!(drawn(&events_of(&state, &end_turn(vec![march], None))), 4);
    let after = events.iter().fold(state, apply);
    assert!(after.hand(SOLO).contains(&joy));
    assert!(after.hand(SOLO).contains(&stamina));
    assert!(!after.discard_pile(SOLO).contains(&joy));
    assert_eq!(after.discard_pile(SOLO), &[march][..]);
}

// Spec 025 C10
#[test]
fn reclaim_needs_crystal_joy() {
    // Sans Euphorie jouée : aucune reprise.
    let (march, stamina) = (card(March, 1), card(Stamina, 1));
    let state = state_of(Hero::Goldyx, &[march, stamina], &[BLACK, BLACK, BLACK]);
    let state = done(&state, play(march, basic_play(0)));
    assert!(legal_actions(&state, SOLO).iter().all(|action| !matches!(
        action,
        Action::EndTurn {
            reclaim: Some(_),
            ..
        }
    )));
    assert!(refused(&state, &end_turn(vec![], Some(stamina))));

    // Avec Euphorie jouée : la carte reprise n'est pas aussi défaussée, et elle est en main.
    let (state, joy, march, stamina) = after_crystal_joy();
    assert!(refused(&state, &end_turn(vec![march], Some(march))));
    assert!(refused(&state, &end_turn(vec![], Some(joy))));
    assert!(is_legal(&state, &end_turn(vec![stamina], Some(march))));
}

// Spec 025 C10 : l'ordre des fins de tour, sans reprise d'abord.
#[test]
fn end_turn_actions_with_reclaim_are_ordered() {
    let (state, _, march, stamina) = after_crystal_joy();
    let ends: Vec<Action> = legal_actions(&state, SOLO)
        .into_iter()
        .filter(|action| matches!(action, Action::EndTurn { .. }))
        .collect();
    assert_eq!(
        ends,
        vec![
            end_turn(vec![], None),
            end_turn(vec![march], None),
            end_turn(vec![stamina], None),
            end_turn(vec![march, stamina], None),
            end_turn(vec![], Some(march)),
            end_turn(vec![stamina], Some(march)),
            end_turn(vec![], Some(stamina)),
            end_turn(vec![march], Some(stamina)),
        ]
    );
}

// Spec 025 L9 : la reprise fait partie de l'effet ; jouée de côté, Euphorie n'en a pas.
#[test]
fn crystal_joy_played_sideways_is_not_reclaimed() {
    let (joy, march) = (card(CrystalJoy, 1), card(March, 1));
    let state = state_of(Hero::Goldyx, &[joy, march], &[BLACK, BLACK, BLACK]);
    let state = done(&state, play(joy, CardPlay::Sideways(SidewaysEffect::Move)));
    assert!(refused(&state, &end_turn(vec![], Some(march))));
}

// Spec 025 C11
#[test]
fn unused_tokens_return_to_the_bank() {
    // MKUE p. 11, 3.a ; p. 5, 1.a
    let march = card(March, 1);
    let state = with_crystals(state_with(&[march], &[BLACK, BLACK, BLACK]), Red, 1);
    let state = with_token(with_token(state, Blue), White);
    let state = done(&state, play(march, basic_play(0)));
    let after = done(&state, end_turn(vec![], None));
    assert!(after.mana_tokens(SOLO).is_empty());
    assert_eq!(after.inventory(SOLO), state.inventory(SOLO));
}

// --- B. Dés supplémentaires ---

// Spec 025 C12 — scan ; MKUE p. 5, 3
#[test]
fn mana_draw_allows_one_more_die() {
    let (mana_draw, march, promise) = (card(ManaDraw, 1), card(March, 1), card(Promise, 1));
    let state = state_with(
        &[mana_draw, march, promise],
        &[basic(Green), basic(White), BLACK],
    );
    assert_eq!(state.dice_limit(SOLO), DICE_PER_TURN);
    let events = events_of(&state, &play(mana_draw, basic_play(0)));
    assert_eq!(
        events,
        vec![played(mana_draw, basic_play(0), Effect::ExtraDie)]
    );
    let state = events.iter().fold(state, apply);
    assert_eq!(state.dice_limit(SOLO), DICE_PER_TURN + 1);

    let state = done(&state, play(march, strong(0, ManaPayment::Die(0))));
    // Le dé de plus : un deuxième dé ce tour.
    assert_eq!(strong_payments(&state, promise), [ManaPayment::Die(1)]);
    let state = done(&state, play(promise, strong(0, ManaPayment::Die(1))));
    assert_eq!(state.dice_taken(SOLO), [0, 1]);

    // Au tour suivant, de nouveau un seul dé.
    let state = done(&state, end_turn(vec![], None));
    assert_eq!(state.dice_limit(SOLO), DICE_PER_TURN);
}

// Spec 025 C13, L4 — Q3 ; FAQ p2, Attraction de mana
#[test]
fn mana_pull_turns_a_black_die_into_any_color() {
    let (mana_pull, march, promise) = (card(ManaPull, 1), card(March, 1), card(Promise, 1));
    // De jour : les dés noirs sont épuisés.
    let state = state_of(
        Hero::Arythea,
        &[mana_pull, march, promise],
        &[BLACK, BLACK, basic(White)],
    );
    assert_eq!(strong_payments(&state, march), []);
    let state = done(&state, play(mana_pull, basic_play(0)));
    assert_eq!(state.dice_limit(SOLO), DICE_PER_TURN + 1);
    // Chacun des dés noirs peut être le dé de plus, avant même le dé régulier du tour.
    assert_eq!(
        strong_payments(&state, march),
        [ManaPayment::Die(0), ManaPayment::Die(1)]
    );
    let events = events_of(&state, &play(march, strong(0, ManaPayment::Die(0))));
    assert_eq!(
        events[0],
        Event::DieTaken {
            player: SOLO,
            die: 0,
            color: BLACK
        }
    );
    let state = events.iter().fold(state, apply);
    // Un deuxième dé noir ne vaut rien ; le dé régulier reste disponible.
    assert_eq!(strong_payments(&state, promise), [ManaPayment::Die(2)]);
}

// Spec 025 C14 — Q2 ; FAQ p1, Mana
#[test]
fn mana_draw_sets_a_die_and_gains_two_tokens() {
    let (mana_draw, march) = (card(ManaDraw, 1), card(March, 1));
    let state = with_crystals(
        state_with(&[mana_draw, march], &[basic(Green), GOLD, BLACK]),
        White,
        1,
    );
    let action = play(mana_draw, strong_set(ManaPayment::Crystal, &[(1, Green)]));
    let events = events_of(&state, &action);
    assert_eq!(
        events,
        vec![
            Event::CrystalUsed {
                player: SOLO,
                color: White
            },
            played(
                mana_draw,
                strong_set(ManaPayment::Crystal, &[(1, Green)]),
                Effect::SetDice { dice: 1, tokens: 2 }
            ),
            die_set(1, Green),
            token_gained(Green),
            token_gained(Green),
        ]
    );
    let state = events.iter().fold(state, apply);
    assert_eq!(state.source()[1], basic(Green));
    assert_eq!(state.dice_set(SOLO), [1]);
    assert!(state.dice_taken(SOLO).is_empty());
    assert_eq!(
        state.mana_tokens(SOLO),
        [arcanum_core::ManaColor::Basic(Green); 2]
    );
    // Le dé mis ne paie rien ; le dé du tour n'est pas consommé.
    assert_eq!(
        strong_payments(&state, march),
        [ManaPayment::Die(0), ManaPayment::Token]
    );
}

// Spec 025 C15
#[test]
fn mana_pull_sets_two_dice() {
    let mana_pull = card(ManaPull, 1);
    let state = with_crystals(
        state_of(Hero::Arythea, &[mana_pull], &[basic(Green), GOLD, BLACK]),
        White,
        1,
    );
    let setting = strong_set(ManaPayment::Crystal, &[(0, Red), (2, Blue)]);
    let events = events_of(&state, &play(mana_pull, setting));
    assert_eq!(
        events,
        vec![
            Event::CrystalUsed {
                player: SOLO,
                color: White
            },
            played(mana_pull, setting, Effect::SetDice { dice: 2, tokens: 1 }),
            die_set(0, Red),
            token_gained(Red),
            die_set(2, Blue),
            token_gained(Blue),
        ]
    );
    let state = events.iter().fold(state, apply);
    assert_eq!(state.source(), [basic(Red), GOLD, basic(Blue)]);
    assert_eq!(state.dice_set(SOLO), [0, 2]);
}

// Spec 025 C16 — scan (« ne pas le relancer »)
#[test]
fn set_die_is_not_rerolled() {
    let (mana_draw, march) = (card(ManaDraw, 1), card(March, 1));
    let start = with_crystals(
        state_with(&[mana_draw, march], &[basic(Green), GOLD, BLACK]),
        White,
        1,
    );
    let with_set = done(
        &start,
        play(mana_draw, strong_set(ManaPayment::Crystal, &[(1, Green)])),
    );
    let with_set = done(&with_set, play(march, strong(0, ManaPayment::Die(0))));
    let without_set = done(&start, play(march, strong(0, ManaPayment::Die(0))));
    for seed in 0..20 {
        let returned = |state: &GameState| -> Vec<Event> {
            events_with_seed(state, &end_turn(vec![], None), seed)
                .into_iter()
                .filter(|event| matches!(event, Event::DieReturned { .. }))
                .collect()
        };
        let rerolled = returned(&without_set);
        assert_eq!(rerolled.len(), 1);
        // Le dé pris normalement est relancé comme sans dé mis ; le dé mis revient vert.
        assert_eq!(
            returned(&with_set),
            vec![
                rerolled[0].clone(),
                Event::DieReturned {
                    player: SOLO,
                    die: 1,
                    color: basic(Green)
                },
            ]
        );
    }
    let state = done(&with_set, end_turn(vec![], None));
    assert_eq!(state.source()[1], basic(Green));
    assert!(state.dice_set(SOLO).is_empty());
    assert!(state.dice_taken(SOLO).is_empty());
}

// Spec 025 C17 — Q4
#[test]
fn set_die_never_becomes_gold() {
    let mana_draw = card(ManaDraw, 1);
    let state = with_crystals(
        state_with(&[mana_draw], &[basic(Green), GOLD, BLACK]),
        White,
        1,
    );
    // Payée avec le dé or (de jour), puis avec le cristal : chaque autre dé, même épuisé, et chaque
    // couleur de base, jamais l'or ni le noir.
    let mut expected = Vec::new();
    for die in [0, 2] {
        for color in BasicColor::ALL {
            expected.push(strong_set(ManaPayment::Die(1), &[(die, color)]));
        }
    }
    for die in 0..3 {
        for color in BasicColor::ALL {
            expected.push(strong_set(ManaPayment::Crystal, &[(die, color)]));
        }
    }
    let strong_plays: Vec<CardPlay> = plays_of(&state, mana_draw)
        .into_iter()
        .filter(|play| matches!(play, CardPlay::Strong { .. }))
        .collect();
    assert_eq!(strong_plays, expected);
}

// Spec 025 L3
#[test]
fn set_dice_need_free_dice() {
    let (mana_pull, march) = (card(ManaPull, 1), card(March, 1));
    let state = with_crystals(
        state_of(
            Hero::Arythea,
            &[mana_pull, march],
            &[basic(White), basic(Green)],
        ),
        White,
        1,
    );
    // Payée avec le dé blanc, il ne resterait qu'un dé : seul le cristal convient.
    let pairs: Vec<ManaPayment> = strong_payments(&state, mana_pull);
    assert_eq!(pairs, [ManaPayment::Crystal; 16]);
    assert!(refused(
        &state,
        &play(
            mana_pull,
            strong_set(ManaPayment::Die(0), &[(0, Red), (1, Blue)])
        )
    ));
    assert!(refused(
        &state,
        &play(
            mana_pull,
            strong_set(ManaPayment::Crystal, &[(1, Red), (1, Blue)])
        )
    ));
    // Un seul dé libre : plus d'Attraction de mana renforcée.
    let state = done(&state, play(march, strong(0, ManaPayment::Die(1))));
    assert_eq!(strong_payments(&state, mana_pull), []);
}

// Spec 025 : la Source relancée en début de manche n'a plus de dé mis (spec 023).
#[test]
fn source_rolled_clears_set_dice() {
    let mana_draw = card(ManaDraw, 1);
    let state = with_crystals(
        state_with(&[mana_draw], &[basic(Green), GOLD, BLACK]),
        White,
        1,
    );
    let state = done(
        &state,
        play(mana_draw, strong_set(ManaPayment::Crystal, &[(2, Red)])),
    );
    let state = apply(
        state,
        &Event::SourceRolled {
            dice: vec![basic(Blue), basic(Blue), BLACK],
        },
    );
    assert!(state.dice_set(SOLO).is_empty());
}

// --- C. Concentration renforcée ---

// Spec 025 C18 — scan ; MKUE p. 5, « Composite » 1.a
#[test]
fn concentration_powers_another_card_for_free_plus_two() {
    let (concentration, march) = (card(Concentration, 1), card(March, 1));
    let state = state_with(&[concentration, march], &[basic(Green), BLACK, BLACK]);
    let action = play(concentration, boosting(ManaPayment::Die(0), march, 0, None));
    let events = events_of(&state, &action);
    assert_eq!(
        events,
        vec![
            Event::DieTaken {
                player: SOLO,
                die: 0,
                color: basic(Green)
            },
            played(
                concentration,
                boosting(ManaPayment::Die(0), march, 0, None),
                Effect::Boost(2)
            ),
            played(march, boosted(0, None), Effect::Move(6)),
        ]
    );
    let state = events.iter().fold(state, apply);
    assert_eq!(state.move_points(SOLO), 6);
    assert_eq!(state.play_area(SOLO).len(), 2);
    assert!(state.hand(SOLO).is_empty());
    assert_eq!(state.dice_taken(SOLO), [0]);
}

// Spec 025 C19
#[test]
fn will_focus_adds_three() {
    let (will_focus, promise) = (card(WillFocus, 1), card(Promise, 1));
    let state = state_of(
        Hero::Goldyx,
        &[will_focus, promise],
        &[basic(Green), BLACK, BLACK],
    );
    let events = events_of(
        &state,
        &play(will_focus, boosting(ManaPayment::Die(0), promise, 0, None)),
    );
    assert_eq!(
        events.last(),
        Some(&played(promise, boosted(0, None), Effect::Influence(7)))
    );
}

// Spec 025 C20 — MKUE p. 5, « Composite » 1.b
#[test]
fn concentration_bonus_only_for_points() {
    let (concentration, crystallize, mana_draw) = (
        card(Concentration, 1),
        card(Crystallize, 1),
        card(ManaDraw, 1),
    );
    let state = state_with(
        &[concentration, crystallize, mana_draw],
        &[basic(Green), GOLD, BLACK],
    );
    let events = events_of(
        &state,
        &play(
            concentration,
            boosting(ManaPayment::Die(0), crystallize, 0, None),
        ),
    );
    assert_eq!(
        events[2..],
        [
            played(crystallize, boosted(0, None), Effect::GainCrystal(Red)),
            crystal_gained(Red),
        ]
    );

    // Avec Source de mana : le dé qui paie Concentration n'est pas mis sur une couleur.
    let set = [
        Some(DieSetting {
            die: 1,
            color: Blue,
        }),
        None,
    ];
    let action = play(
        concentration,
        boosting_set(ManaPayment::Die(0), mana_draw, 0, None, set),
    );
    let events = events_of(&state, &action);
    assert_eq!(
        events[2..],
        [
            played(
                mana_draw,
                CardPlay::Boosted {
                    option: 0,
                    discard: None,
                    set_dice: set
                },
                Effect::SetDice { dice: 1, tokens: 2 }
            ),
            die_set(1, Blue),
            token_gained(Blue),
            token_gained(Blue),
        ]
    );
    assert!(refused(
        &state,
        &play(
            concentration,
            boosting_set(
                ManaPayment::Die(0),
                mana_draw,
                0,
                None,
                [
                    Some(DieSetting {
                        die: 0,
                        color: Blue
                    }),
                    None
                ]
            )
        )
    ));
}

// Spec 025 C21 — MKUE p. 4, 6
#[test]
fn concentration_with_improvisation() {
    let (concentration, improvisation, rage) = (
        card(Concentration, 1),
        card(Improvisation, 1),
        card(Rage, 1),
    );
    let state = state_with(
        &[concentration, improvisation, rage],
        &[basic(Green), BLACK, BLACK],
    );
    let events = events_of(
        &state,
        &play(
            concentration,
            boosting(ManaPayment::Die(0), improvisation, 0, Some(rage)),
        ),
    );
    assert_eq!(
        events[2..],
        [
            played(improvisation, boosted(0, Some(rage)), Effect::Move(7)),
            Event::CardDiscarded {
                player: SOLO,
                card: rage
            },
        ]
    );
    // Concentration elle-même n'est pas la carte défaussée.
    assert!(!plays_of(&state, concentration).iter().any(|play| matches!(
        play,
        CardPlay::Strong { with: Some(target), .. } if target.discard == Some(concentration)
    )));
}

// Spec 025 C22 — Q5
#[test]
fn concentration_needs_a_target() {
    let (concentration, rage, determination) = (
        card(Concentration, 1),
        card(Rage, 1),
        card(Determination, 1),
    );
    // Attaque et Parade renforcées attendent le combat (spec 031).
    let state = state_with(
        &[concentration, rage, determination],
        &[basic(Green), GOLD, BLACK],
    );
    assert_eq!(strong_payments(&state, concentration), []);
    // Pas de Concentration sur une autre Concentration.
    let other = card(Concentration, 2);
    let state = state_with(&[concentration, other], &[basic(Green), GOLD, BLACK]);
    assert_eq!(strong_payments(&state, concentration), []);
    assert_eq!(strong_payments(&state, other), []);
}

// Spec 025 L5
#[test]
fn concentration_paid_with_a_token() {
    let (concentration, march) = (card(Concentration, 1), card(March, 1));
    let state = with_token(
        state_with(&[concentration, march], &[BLACK, BLACK, BLACK]),
        Green,
    );
    assert_eq!(strong_payments(&state, concentration), [ManaPayment::Token]);
}

// Spec 025 L6
#[test]
fn crystal_joy_played_with_concentration_is_reclaimed() {
    let (will_focus, joy, march) = (card(WillFocus, 1), card(CrystalJoy, 1), card(March, 1));
    let state = state_of(
        Hero::Goldyx,
        &[will_focus, joy, march],
        &[basic(Green), BLACK, BLACK],
    );
    let state = done(
        &state,
        play(will_focus, boosting(ManaPayment::Die(0), joy, 1, None)),
    );
    assert_eq!(state.inventory(SOLO).crystals(Blue), 1);
    let events = events_of(&state, &end_turn(vec![], Some(march)));
    assert!(events.contains(&Event::CardReturnedToHand {
        player: SOLO,
        card: joy
    }));
}

// --- Journal ---

// Spec 025 C23
#[test]
fn mana_cards_replay_and_are_public() {
    let other = PlayerId(1);
    let joy = card(CrystalJoy, 1);
    let cases = [
        (
            token_gained(Blue),
            PlayerEvent::ManaTokenGained {
                player: SOLO,
                color: arcanum_core::ManaColor::Basic(Blue),
            },
        ),
        (
            Event::ManaTokenUsed {
                player: SOLO,
                color: arcanum_core::ManaColor::Basic(Red),
            },
            PlayerEvent::ManaTokenUsed {
                player: SOLO,
                color: arcanum_core::ManaColor::Basic(Red),
            },
        ),
        (
            Event::CardReturnedToHand {
                player: SOLO,
                card: joy,
            },
            PlayerEvent::CardReturnedToHand {
                player: SOLO,
                card: joy,
            },
        ),
        (
            die_set(1, Green),
            PlayerEvent::DieSet {
                player: SOLO,
                die: 1,
                color: Green,
            },
        ),
    ];
    for (event, expected) in cases {
        assert_eq!(filter_event(&event, other), expected);
    }

    // Un tour avec jeton, cristal gagné et reprise, rejoué depuis le journal.
    let (concentration, crystallize, march) =
        (card(Concentration, 1), card(Crystallize, 1), card(March, 1));
    let start = state_with(
        &[concentration, crystallize, march],
        &[basic(Green), GOLD, BLACK],
    );
    let mut log = Vec::new();
    let mut state = start.clone();
    for action in [
        play(concentration, basic_play(2)),
        play(crystallize, paid(0, ManaPayment::Token)),
        play(march, strong(0, ManaPayment::Die(0))),
        end_turn(vec![], None),
    ] {
        let events = events_of(&state, &action);
        state = events.iter().fold(state, apply);
        log.extend(events);
    }
    let replayed = log.iter().fold(start, apply);
    assert_eq!(replayed, state);
    assert_eq!(replayed.inventory(SOLO).crystals(Red), 1);

    // Un tour avec un dé de plus et un dé mis sur une couleur.
    let (mana_draw, march) = (card(ManaDraw, 1), card(March, 1));
    let start = with_crystals(
        state_with(&[mana_draw, march], &[basic(Green), GOLD, BLACK]),
        White,
        1,
    );
    let mut log = Vec::new();
    let mut state = start.clone();
    for action in [
        play(mana_draw, strong_set(ManaPayment::Crystal, &[(1, Green)])),
        play(march, strong(0, ManaPayment::Token)),
        end_turn(vec![], None),
    ] {
        let events = events_of(&state, &action);
        state = events.iter().fold(state, apply);
        log.extend(events);
    }
    assert_eq!(log.iter().fold(start, apply), state);

    // Un tour avec Concentration renforcée.
    let (concentration, march) = (card(Concentration, 1), card(March, 1));
    let start = state_with(&[concentration, march], &[basic(Green), GOLD, BLACK]);
    let mut log = Vec::new();
    let mut state = start.clone();
    for action in [
        play(concentration, boosting(ManaPayment::Die(0), march, 0, None)),
        end_turn(vec![], None),
    ] {
        let events = events_of(&state, &action);
        state = events.iter().fold(state, apply);
        log.extend(events);
    }
    assert_eq!(log.iter().fold(start, apply), state);

    let (state, _, march, _) = after_crystal_joy();
    let events = events_of(&state, &end_turn(vec![], Some(march)));
    assert_eq!(
        events.iter().fold(state.clone(), apply),
        done(&state, end_turn(vec![], Some(march)))
    );
}

// Spec 025 C24
#[test]
fn mana_cards_never_overspend() {
    let mut turns = 0;
    for seed in 0..50 {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut rngs = GameRngs::new(seed);
        let hero = Hero::ALL[(seed % 4) as usize];
        let hand: Vec<CardId> = {
            let mut deck = hero.starting_deck();
            rand::seq::SliceRandom::shuffle(deck.as_mut_slice(), &mut rng);
            deck.into_iter().take(5).collect()
        };
        let mut state = with_crystals(
            state_of(hero, &hand, &[GOLD, basic(Red), basic(Blue)]),
            Red,
            2,
        );
        loop {
            let legal = legal_actions(&state, SOLO);
            let Some(action) = legal.choose(&mut rng).cloned() else {
                break;
            };
            let events = decide(&state, SOLO, &action, &mut rngs).expect("action légale");
            state = events.iter().fold(state, apply);
            // Les dés mis sur une couleur ne comptent pas (Q2).
            assert!(state.dice_taken(SOLO).len() <= state.dice_limit(SOLO));
            assert!(state.dice_limit(SOLO) >= DICE_PER_TURN);
            for color in BasicColor::ALL {
                assert!(state.inventory(SOLO).crystals(color) <= CRYSTAL_LIMIT);
            }
            if matches!(action, Action::EndTurn { .. }) {
                turns += 1;
                assert!(state.mana_tokens(SOLO).is_empty());
                assert!(state.dice_set(SOLO).is_empty());
                assert_eq!(state.dice_limit(SOLO), DICE_PER_TURN);
            }
        }
    }
    assert!(turns >= 200, "{turns} tours seulement");
}

// --- Cas limites ---

// Spec 025 L1
#[test]
fn crystallize_paid_with_its_own_crystal() {
    let crystallize = card(Crystallize, 1);
    let state = with_crystals(
        state_with(&[crystallize], &[BLACK, BLACK, BLACK]),
        Red,
        CRYSTAL_LIMIT,
    );
    let events = events_of(&state, &play(crystallize, paid(0, ManaPayment::Crystal)));
    // Le cristal payé libère une place : c'est un cristal qu'on regagne, pas un jeton.
    assert_eq!(
        events,
        vec![
            Event::CrystalUsed {
                player: SOLO,
                color: Red
            },
            played(
                crystallize,
                paid(0, ManaPayment::Crystal),
                Effect::PayForCrystal(Red)
            ),
            crystal_gained(Red),
        ]
    );
    let state = events.iter().fold(state, apply);
    assert_eq!(state.inventory(SOLO).crystals(Red), CRYSTAL_LIMIT);
    assert!(state.mana_tokens(SOLO).is_empty());
}

// Spec 025 : un effet sans coût ne se paie pas.
#[test]
fn free_option_refuses_a_payment() {
    let concentration = card(Concentration, 1);
    let state = with_token(
        state_with(&[concentration], &[basic(Green), GOLD, BLACK]),
        Blue,
    );
    assert!(refused(
        &state,
        &play(concentration, paid(0, ManaPayment::Token))
    ));
    assert!(refused(
        &state,
        &play(concentration, paid(0, ManaPayment::Die(0)))
    ));
}

// Spec 025 L8
#[test]
fn using_an_absent_token_changes_nothing() {
    let state = with_token(state_with(&[card(March, 1)], &[BLACK, BLACK, BLACK]), Blue);
    let after = apply(
        state.clone(),
        &Event::ManaTokenUsed {
            player: SOLO,
            color: arcanum_core::ManaColor::Basic(Red),
        },
    );
    assert_eq!(after, state);
}
