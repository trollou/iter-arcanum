//! Scénarios de la spec 022 (effets renforcés).

use arcanum_core::{
    Action, AttackKind, BasicAction, BasicColor, BasicEffect, CRYSTAL_LIMIT, CardId, CardPlay,
    Cost, DICE_PER_TURN, DeedCard, Effect, Element, Event, GameConfig, GameRngs, GameState, Hero,
    Hex, ManaColor, ManaPayment, PlayerEvent, PlayerId, Scenario, SidewaysEffect, TileId,
    TilePosition, TimeOfDay, apply, decide, filter_event, legal_actions, view,
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

fn config(seed: u64) -> GameConfig {
    GameConfig {
        seed,
        scenario: Scenario::sandbox(),
        hero: Hero::Tovak,
        dummy_hero: Default::default(),
    }
}

/// Partie de bac à sable construite par événements : Tovak, `hand` en main (le reste de son deck
/// dessous), figurine au centre de la tuile de départ, Source `dice`.
fn state_with(hand: &[CardId], dice: &[ManaColor]) -> GameState {
    let mut order = hand.to_vec();
    let mut rest = Hero::Tovak.starting_deck();
    for held in hand {
        if let Some(index) = rest.iter().position(|card| card == held) {
            rest.remove(index);
        }
    }
    order.extend(rest);
    let mut events = vec![
        Event::GameStarted { config: config(0) },
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

fn strong(option: u8, mana: ManaPayment) -> CardPlay {
    CardPlay::Strong {
        gain: None,
        extra_mana: None,
        option,
        discard: None,
        mana,
        // Spec 025 : aucun dé mis sur une couleur.
        set_dice: [None, None],
        // Spec 025 : pas jouée avec Concentration.
        with: None,
    }
}

fn play(card: CardId, play: CardPlay) -> Action {
    Action::PlayCard { card, play }
}

fn events_with_seed(state: &GameState, action: &Action, seed: u64) -> Vec<Event> {
    decide(state, SOLO, action, &mut config(seed).rngs())
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
    decide(state, SOLO, action, &mut config(0).rngs()).is_err()
}

/// Les effets renforcés proposés pour `card`, avec leur paiement.
fn strong_plays(state: &GameState, of: CardId) -> Vec<CardPlay> {
    legal_actions(state, SOLO)
        .into_iter()
        .filter_map(|action| match action {
            Action::PlayCard { card, play } if card == of => Some(play),
            _ => None,
        })
        .filter(|play| matches!(play, CardPlay::Strong { .. }))
        .collect()
}

fn payments(state: &GameState, of: CardId) -> Vec<ManaPayment> {
    strong_plays(state, of)
        .into_iter()
        .filter_map(|play| match play {
            CardPlay::Strong { mana, .. } => Some(mana),
            _ => None,
        })
        .collect()
}

fn end_turn() -> Action {
    Action::EndTurn {
        discard: vec![],
        reclaim: None,
        glade: None,
        steady_tempo: false,
    }
}

fn returned(events: &[Event]) -> Vec<&Event> {
    events
        .iter()
        .filter(|event| matches!(event, Event::DieReturned { .. }))
        .collect()
}

const fn attack(amount: u8) -> Effect {
    Effect::Attack {
        amount,
        element: Element::Physical,
        kind: AttackKind::Melee,
    }
}

const fn block(amount: u8) -> Effect {
    Effect::Block {
        amount,
        element: Element::Physical,
    }
}

// --- Contenu ---

// Spec 022 C1
#[test]
fn strong_effect_definitions_match_cards() {
    // Scans `../mage-knight/docs/regles/scans/deed/`.
    let options = |options: &'static [Effect]| BasicEffect::Options {
        options,
        cost: Cost::None,
        modifiers: &[],
        mana: &[],
    };
    const RANGED_3: &[Effect] = &[Effect::Attack {
        amount: 3,
        element: Element::Physical,
        kind: AttackKind::Ranged,
    }];
    const ATTACK_4: &[Effect] = &[attack(4)];
    const BLOCK_5: &[Effect] = &[block(5)];
    const IMPROVISATION: &[Effect] = &[Effect::Move(5), Effect::Influence(5), attack(5), block(5)];
    const INSTINCT: &[Effect] = &[Effect::Move(4), Effect::Influence(4), attack(4), block(4)];
    // Spec 025 : un cristal de n'importe quelle couleur de base.
    const ANY_CRYSTAL: &[Effect] = &[
        Effect::GainCrystal(Red),
        Effect::GainCrystal(Blue),
        Effect::GainCrystal(White),
        Effect::GainCrystal(Green),
    ];
    let cases: Vec<(BasicAction, BasicEffect)> = vec![
        (March, options(&[Effect::Move(4)])),
        (Stamina, options(&[Effect::Move(4)])),
        (Swiftness, options(RANGED_3)),
        (Promise, options(&[Effect::Influence(4)])),
        // Spec 040 : Menace renforcée fait perdre de la réputation.
        (
            Threaten,
            options(&[Effect::InfluenceReputation {
                amount: 5,
                reputation: -1,
            }]),
        ),
        (Rage, options(ATTACK_4)),
        (Determination, options(BLOCK_5)),
        (
            Improvisation,
            BasicEffect::Options {
                options: IMPROVISATION,
                cost: Cost::DiscardAnotherCard,
                modifiers: &[],
                mana: &[],
            },
        ),
        (
            Tranquility,
            options(&[Effect::Heal(2), Effect::DrawCards(2)]),
        ),
        (Crystallize, options(ANY_CRYSTAL)),
        (ManaDraw, options(&[Effect::SetDice { dice: 1, tokens: 2 }])),
        (Concentration, options(&[Effect::Boost(2)])),
        // Jouable depuis la spec 032 (Q1) : ses six options sont vérifiées par la spec 032, C22.
        (BattleVersatility, BattleVersatility.definition().strong),
        (ManaPull, options(&[Effect::SetDice { dice: 2, tokens: 1 }])),
        (WillFocus, options(&[Effect::Boost(3)])),
        (CrystalJoy, options(ANY_CRYSTAL)),
        // Spec 040 : gloire et réputation en plus pendant une interaction.
        (
            NobleManners,
            options(&[Effect::InfluenceInInteraction {
                amount: 4,
                fame: 1,
                reputation: 1,
            }]),
        ),
        (
            Rejuvenate,
            options(&[
                Effect::Heal(2),
                Effect::DrawCards(2),
                Effect::GainCrystal(Green),
                // Spec 042.
                Effect::ReadyUnit(3),
            ]),
        ),
        // Spec 033 : la Parade de glace se compte contre l'ennemi paré.
        (ColdToughness, options(&[Effect::IceBlockPerAbility(5)])),
        (Instinct, options(INSTINCT)),
    ];
    assert_eq!(cases.len(), BasicAction::ALL.len());
    for (action, expected) in cases {
        assert_eq!(action.definition().strong, expected, "{action:?}");
    }
}

// --- Payer avec un dé ---

// Spec 022 C2
#[test]
fn powering_march_with_a_green_die_gives_four_move_points() {
    // MKUE p. 4, 3.a ; p. 5, 3
    let march = card(March, 1);
    let state = state_with(&[march], &[basic(Green), basic(Blue), basic(Red)]);
    let action = play(march, strong(0, ManaPayment::Die(0)));
    assert!(is_legal(&state, &action));
    let events = events_of(&state, &action);
    assert_eq!(
        events,
        vec![
            Event::DieTaken {
                player: SOLO,
                die: 0,
                color: basic(Green)
            },
            Event::CardPlayed {
                player: SOLO,
                card: march,
                play: strong(0, ManaPayment::Die(0)),
                effect: Effect::Move(4)
            },
        ]
    );
    let after = events.iter().fold(state, apply);
    assert_eq!(after.move_points(SOLO), 4);
    assert_eq!(after.dice_taken(SOLO), &[0]);
}

// Spec 022 C3
#[test]
fn gold_die_pays_any_basic_color_by_day() {
    // MKUE p. 5, 2.a
    for (action, color, effect) in [
        (Promise, White, Effect::Influence(4)),
        (March, Green, Effect::Move(4)),
        (Stamina, Blue, Effect::Move(4)),
        (Instinct, Red, Effect::Move(4)),
    ] {
        let played = card(action, 1);
        let state = state_with(&[played], &[GOLD, BLACK, BLACK]);
        assert_eq!(action.definition().color, color);
        let events = events_of(&state, &play(played, strong(0, ManaPayment::Die(0))));
        assert_eq!(
            events[0],
            Event::DieTaken {
                player: SOLO,
                die: 0,
                color: GOLD
            }
        );
        assert!(
            matches!(events[1], Event::CardPlayed { effect: e, .. } if e == effect),
            "{action:?}"
        );
    }
}

// Spec 022 C4
#[test]
fn black_die_cannot_pay_by_day() {
    // MKUE p. 5, 2.a et 3.c
    let march = card(March, 1);
    let state = state_with(&[march], &[BLACK, basic(Blue), basic(Red)]);
    assert!(payments(&state, march).is_empty());
    assert!(refused(
        &state,
        &play(march, strong(0, ManaPayment::Die(0)))
    ));
}

// Spec 022 C5
#[test]
fn die_must_match_the_card_color() {
    // MKUE p. 4, 6
    let march = card(March, 1);
    let state = state_with(&[march], &[basic(Blue), basic(Red), basic(White)]);
    assert!(strong_plays(&state, march).is_empty());
    assert!(refused(
        &state,
        &play(march, strong(0, ManaPayment::Die(0)))
    ));
}

// Spec 022 C6
#[test]
fn only_one_die_per_turn() {
    // MKUE p. 5, 3
    assert_eq!(DICE_PER_TURN, 1);
    let (march, other) = (card(March, 1), card(March, 2));
    let state = state_with(&[march, other], &[basic(Green), GOLD, basic(Green)]);
    let state = done(&state, play(march, strong(0, ManaPayment::Die(0))));
    assert!(payments(&state, other).is_empty());
    for die in [1, 2] {
        assert!(refused(
            &state,
            &play(other, strong(0, ManaPayment::Die(die)))
        ));
    }
    // Un cristal reste utilisable.
    let state = with_crystals(state, Green, 1);
    assert_eq!(payments(&state, other), vec![ManaPayment::Crystal]);
}

// Spec 022 C7
#[test]
fn taken_die_stays_unavailable_until_end_of_turn() {
    let march = card(March, 1);
    let dice = [basic(Blue), basic(Green), GOLD];
    let state = state_with(&[march], &dice);
    let state = done(&state, play(march, strong(0, ManaPayment::Die(1))));
    assert_eq!(state.source(), &dice);
    assert_eq!(state.dice_taken(SOLO), &[1]);
    assert_eq!(view(&state, SOLO).dice_taken, vec![1]);
}

// --- Payer avec un cristal ---

// Spec 022 C8
#[test]
fn powering_with_a_crystal_spends_it() {
    // MKUE p. 5, 1.b
    let stamina = card(Stamina, 1);
    let state = with_crystals(
        state_with(&[stamina], &[basic(Red), basic(Red), basic(Red)]),
        Blue,
        1,
    );
    let action = play(stamina, strong(0, ManaPayment::Crystal));
    let events = events_of(&state, &action);
    assert_eq!(
        events[0],
        Event::CrystalUsed {
            player: SOLO,
            color: Blue
        }
    );
    assert!(matches!(
        events[1],
        Event::CardPlayed {
            effect: Effect::Move(4),
            ..
        }
    ));
    let after = events.iter().fold(state, apply);
    assert_eq!(after.inventory(SOLO).crystals(Blue), 0);
    assert_eq!(after.move_points(SOLO), 4);
    assert!(after.dice_taken(SOLO).is_empty());
}

// Spec 022 C9
#[test]
fn each_crystal_pays_once() {
    // MKUE p. 5, 1.b
    let (first, second) = (card(March, 1), card(March, 2));
    let state = with_crystals(
        state_with(&[first, second], &[basic(Red), basic(Blue), BLACK]),
        Green,
        2,
    );
    let state = done(&state, play(first, strong(0, ManaPayment::Crystal)));
    let state = done(&state, play(second, strong(0, ManaPayment::Crystal)));
    assert_eq!(state.move_points(SOLO), 8);
    assert_eq!(state.inventory(SOLO).crystals(Green), 0);
    let third = with_crystals(state, Red, 1);
    assert!(legal_actions(&third, SOLO).iter().all(|action| !matches!(
        action,
        Action::PlayCard {
            play: CardPlay::Strong { .. },
            ..
        }
    )));
}

// Spec 022 C10 : l'exemple des règles.
#[test]
fn die_and_crystal_in_the_same_turn() {
    // MKUE p. 5, 1 et 3
    let (march, stamina, instinct, rage, promise) = (
        card(March, 1),
        card(Stamina, 1),
        card(Instinct, 1),
        card(Rage, 1),
        card(Promise, 1),
    );
    let state = with_crystals(
        state_with(
            &[march, stamina, instinct, rage, promise],
            &[basic(Green), GOLD, BLACK],
        ),
        Blue,
        1,
    );
    let state = done(&state, play(march, strong(0, ManaPayment::Die(0))));
    let state = done(&state, play(stamina, strong(0, ManaPayment::Crystal)));
    assert!(strong_plays(&state, instinct).is_empty());
    assert_eq!(state.move_points(SOLO), 8);
    assert_eq!(state.inventory(SOLO).crystals(Blue), 0);
    assert_eq!(state.source(), &[basic(Green), GOLD, BLACK]);
}

// Spec 022 C11
#[test]
fn player_chooses_between_die_and_crystal() {
    let march = card(March, 1);
    let state = with_crystals(
        state_with(&[march], &[basic(Blue), basic(Green), GOLD]),
        Green,
        1,
    );
    assert_eq!(
        payments(&state, march),
        vec![
            ManaPayment::Die(1),
            ManaPayment::Die(2),
            ManaPayment::Crystal
        ]
    );
}

// --- Options ---

// Spec 022 C12
#[test]
fn strong_choice_effects_offer_only_move_and_influence_out_of_combat() {
    // MKUE p. 6, 5.c
    let (instinct, manners) = (card(Instinct, 1), card(NobleManners, 1));
    let state = state_with(&[instinct, manners], &[basic(Red), basic(White), BLACK]);
    assert_eq!(
        strong_plays(&state, instinct),
        vec![
            strong(0, ManaPayment::Die(0)),
            strong(1, ManaPayment::Die(0))
        ]
    );
    assert_eq!(
        strong_plays(&state, manners),
        vec![strong(0, ManaPayment::Die(1))]
    );
    let after = done(&state, play(manners, strong(0, ManaPayment::Die(1))));
    assert_eq!(after.influence_points(SOLO), 4);
}

// Spec 022 C13
#[test]
fn improvisation_strong_discards_and_pays() {
    // MKUE p. 4, 6
    let (improvisation, march, rage) = (card(Improvisation, 1), card(March, 1), card(Rage, 1));
    let state = state_with(
        &[improvisation, march, rage],
        &[basic(Red), basic(Blue), basic(Blue)],
    );
    let with = |option, discard| CardPlay::Strong {
        gain: None,
        extra_mana: None,
        option,
        discard: Some(discard),
        mana: ManaPayment::Die(0),
        set_dice: [None, None],
        with: None,
    };
    assert_eq!(
        strong_plays(&state, improvisation),
        vec![with(0, march), with(0, rage), with(1, march), with(1, rage)]
    );
    let events = events_of(&state, &play(improvisation, with(0, rage)));
    assert_eq!(
        events,
        vec![
            Event::DieTaken {
                player: SOLO,
                die: 0,
                color: basic(Red)
            },
            Event::CardPlayed {
                player: SOLO,
                card: improvisation,
                play: with(0, rage),
                effect: Effect::Move(5)
            },
            Event::CardDiscarded {
                player: SOLO,
                card: rage
            },
        ]
    );
}

// Spec 022 C14
#[test]
fn out_of_scope_strong_effects_are_not_offered() {
    let cards: Vec<CardId> = [
        Rage,
        Determination,
        Swiftness,
        BattleVersatility,
        ColdToughness,
        // Menace renforcée : jouable depuis la spec 040.
        // Tranquillité renforcée : jouable depuis la spec 034.
        // Cristallisation, Euphorie cristalline, Rajeunissement, Source de mana et Attraction de
        // mana : renforcées jouables (spec 025). Concentration renforcée aussi, mais seule en main
        // elle n'a pas de carte à renforcer (spec 025 C22).
        Concentration,
        WillFocus,
    ]
    .map(|action| card(action, 1))
    .to_vec();
    for &held in &cards {
        let mut state = state_with(&[held], &[GOLD, GOLD, GOLD]);
        for color in BasicColor::ALL {
            state = with_crystals(state, color, CRYSTAL_LIMIT);
        }
        assert!(strong_plays(&state, held).is_empty(), "{held:?}");
    }
}

// Spec 022 C15 (spec 013 C14 avec la Source)
#[test]
fn legal_actions_are_exhaustive() {
    let (march, rage, improvisation) = (card(March, 1), card(Rage, 1), card(Improvisation, 1));
    let state = with_crystals(
        state_with(&[march, rage, improvisation], &[basic(Green), GOLD, BLACK]),
        Red,
        1,
    );
    let sideways = |card| {
        [
            play(card, CardPlay::Sideways(SidewaysEffect::Move)),
            play(card, CardPlay::Sideways(SidewaysEffect::Influence)),
        ]
    };
    let basic_play = |option| CardPlay::Basic {
        die: None,
        gain: None,
        with: None,
        option,
        discard: None,
        mana: None,
    };
    let basic_discarding = |option, discard| CardPlay::Basic {
        die: None,
        gain: None,
        with: None,
        option,
        discard: Some(discard),
        mana: None,
    };
    let strong_discarding = |option, discard, mana| CardPlay::Strong {
        gain: None,
        extra_mana: None,
        option,
        discard: Some(discard),
        mana,
        set_dice: [None, None],
        with: None,
    };
    let mut expected = vec![
        play(march, basic_play(0)),
        play(march, strong(0, ManaPayment::Die(0))),
        play(march, strong(0, ManaPayment::Die(1))),
    ];
    expected.extend(sideways(march));
    expected.extend(sideways(rage));
    for option in [0, 1] {
        for discard in [march, rage] {
            expected.push(play(improvisation, basic_discarding(option, discard)));
        }
    }
    for option in [0, 1] {
        for discard in [march, rage] {
            for mana in [ManaPayment::Die(1), ManaPayment::Crystal] {
                expected.push(play(
                    improvisation,
                    strong_discarding(option, discard, mana),
                ));
            }
        }
    }
    expected.extend(sideways(improvisation));
    expected.extend(
        [
            vec![march],
            vec![rage],
            vec![improvisation],
            vec![march, rage],
            vec![march, improvisation],
            vec![rage, improvisation],
            vec![march, rage, improvisation],
        ]
        .map(|discard| Action::EndTurn {
            discard,
            reclaim: None,
            glade: None,
            steady_tempo: false,
        }),
    );
    let legal = legal_actions(&state, SOLO);
    assert_eq!(legal, expected);
    for action in &legal {
        assert!(!refused(&state, action), "{action:?}");
    }
}

// --- Fin du tour ---

// Spec 022 C16
#[test]
fn taken_die_is_rerolled_and_returned_first() {
    // MKUE p. 5, 3.a ; p. 11, 1
    let march = card(March, 1);
    let state = state_with(&[march], &[basic(Blue), basic(Green), BLACK]);
    let state = done(&state, play(march, strong(0, ManaPayment::Die(1))));
    let events = events_with_seed(&state, &end_turn(), 7);
    let Event::DieReturned { player, die, color } = events[0] else {
        panic!(
            "le premier événement doit être DieReturned : {:?}",
            events[0]
        );
    };
    assert_eq!((player, die), (SOLO, 1));
    assert_eq!(events[1], Event::TurnEnded { player: SOLO });
    assert_eq!(returned(&events).len(), 1);
    assert_eq!(events, events_with_seed(&state, &end_turn(), 7));
    let after = events.iter().fold(state, apply);
    assert_eq!(after.source(), &[basic(Blue), color, BLACK]);
    assert!(after.dice_taken(SOLO).is_empty());
}

// Spec 022 C17
#[test]
fn die_limit_resets_each_turn() {
    // MKUE p. 5, 3
    let (march, stamina) = (card(March, 1), card(Stamina, 1));
    let state = state_with(&[march, stamina], &[basic(Green), GOLD, GOLD]);
    let state = done(&state, play(march, strong(0, ManaPayment::Die(0))));
    assert!(payments(&state, stamina).is_empty());
    // Endurance reste en main : au tour suivant, un dé or peut de nouveau être pris.
    let state = done(&state, end_turn());
    assert!(state.dice_taken(SOLO).is_empty());
    assert!(state.hand(SOLO).contains(&stamina));
    let offered = payments(&state, stamina);
    assert!(offered.contains(&ManaPayment::Die(1)), "{offered:?}");
    assert!(offered.contains(&ManaPayment::Die(2)), "{offered:?}");
}

// Spec 022 C18
#[test]
fn untaken_dice_are_not_rerolled() {
    let stamina = card(Stamina, 1);
    let dice = [basic(Red), GOLD, BLACK];
    let state = with_crystals(state_with(&[stamina], &dice), Blue, 1);
    let state = done(&state, play(stamina, strong(0, ManaPayment::Crystal)));
    let events = events_of(&state, &end_turn());
    assert!(returned(&events).is_empty());
    let after = events.iter().fold(state, apply);
    assert_eq!(after.source(), &dice);
}

// Spec 022 C19
#[test]
fn returned_die_ignores_the_half_rule() {
    // MKUE p. 5, 3.a (une simple relance) ; la règle de la moitié est celle de la p. 3.
    let march = card(March, 1);
    let state = state_with(&[march], &[GOLD, basic(Green), BLACK]);
    let state = done(&state, play(march, strong(0, ManaPayment::Die(1))));
    let seed = (0..1000)
        .find(|&seed| {
            matches!(
                events_with_seed(&state, &end_turn(), seed)[0],
                Event::DieReturned { color: BLACK, .. }
            )
        })
        .expect("une graine relance le dé en noir");
    let after = events_with_seed(&state, &end_turn(), seed)
        .iter()
        .fold(state, apply);
    assert_eq!(after.source(), &[GOLD, BLACK, BLACK]);
}

// --- Information cachée et journal ---

// Spec 022 C20
#[test]
fn mana_payments_are_public() {
    let other = PlayerId(1);
    let cases = [
        (
            Event::DieTaken {
                player: SOLO,
                die: 2,
                color: GOLD,
            },
            PlayerEvent::DieTaken {
                player: SOLO,
                die: 2,
                color: GOLD,
            },
        ),
        (
            Event::CrystalUsed {
                player: SOLO,
                color: Blue,
            },
            PlayerEvent::CrystalUsed {
                player: SOLO,
                color: Blue,
            },
        ),
        (
            Event::DieReturned {
                player: SOLO,
                die: 0,
                color: BLACK,
            },
            PlayerEvent::DieReturned {
                player: SOLO,
                die: 0,
                color: BLACK,
            },
        ),
        (
            Event::CardPlayed {
                player: SOLO,
                card: card(March, 1),
                play: strong(0, ManaPayment::Crystal),
                effect: Effect::Move(4),
            },
            PlayerEvent::CardPlayed {
                player: SOLO,
                card: card(March, 1),
                play: strong(0, ManaPayment::Crystal),
                effect: Effect::Move(4),
            },
        ),
    ];
    for (event, expected) in cases {
        assert_eq!(filter_event(&event, other), expected);
    }
}

// Spec 022 C21
#[test]
fn replaying_log_rebuilds_mana_state() {
    let (march, stamina) = (card(March, 1), card(Stamina, 1));
    let start = with_crystals(
        state_with(&[march, stamina], &[basic(Green), GOLD, BLACK]),
        Blue,
        2,
    );
    let mut log = Vec::new();
    let mut state = start.clone();
    for action in [
        play(march, strong(0, ManaPayment::Die(0))),
        play(stamina, strong(0, ManaPayment::Crystal)),
        end_turn(),
    ] {
        let events = events_of(&state, &action);
        state = events.iter().fold(state, apply);
        log.extend(events);
    }
    let replayed = log.iter().fold(start, apply);
    assert_eq!(replayed, state);
    assert_eq!(replayed.inventory(SOLO).crystals(Blue), 1);
}

// --- Cas limites ---

// Spec 022 L1
#[test]
fn gold_is_useless_at_night() {
    // MKUE p. 5, 2.b et 3.c
    for color in BasicColor::ALL {
        assert!(GOLD.can_pay(color, TimeOfDay::Day));
        assert!(!GOLD.can_pay(color, TimeOfDay::Night));
        assert!(!BLACK.can_pay(color, TimeOfDay::Day));
        assert!(!BLACK.can_pay(color, TimeOfDay::Night));
        for time in [TimeOfDay::Day, TimeOfDay::Night] {
            for other in BasicColor::ALL {
                assert_eq!(basic(other).can_pay(color, time), other == color);
            }
        }
    }
}

// Spec 022 L2
#[test]
fn improvisation_strong_needs_another_card() {
    let improvisation = card(Improvisation, 1);
    let state = state_with(&[improvisation], &[basic(Red), GOLD, GOLD]);
    assert!(strong_plays(&state, improvisation).is_empty());
}

// Spec 022 L3
#[test]
fn mana_payments_never_overspend() {
    let mut turns = 0;
    for seed in 0..40 {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut rngs = GameRngs::new(seed);
        let hand: Vec<CardId> = {
            let mut deck = Hero::Tovak.starting_deck();
            rand::seq::SliceRandom::shuffle(deck.as_mut_slice(), &mut rng);
            deck.into_iter().take(5).collect()
        };
        let mut state = with_crystals(state_with(&hand, &[GOLD, basic(Red), basic(Green)]), Red, 2);
        state = with_crystals(state, Blue, 1);
        loop {
            let legal = legal_actions(&state, SOLO);
            let Some(action) = legal.choose(&mut rng).cloned() else {
                break;
            };
            let events = decide(&state, SOLO, &action, &mut rngs).expect("action légale");
            state = events.iter().fold(state, apply);
            // Spec 025 : Source de mana donne un dé de plus ; la pioche de Tranquillité (spec 034)
            // change les parties tirées au hasard et en fait jouer.
            assert!(state.dice_taken(SOLO).len() <= state.dice_limit(SOLO));
            if matches!(action, Action::EndTurn { .. }) {
                turns += 1;
                assert!(state.dice_taken(SOLO).is_empty());
            }
        }
    }
    assert!(turns >= 100, "{turns} tours seulement");
}

// Spec 022 L3 : `apply` reste infaillible.
#[test]
fn applying_impossible_payments_changes_nothing_harmful() {
    let state = state_with(&[card(March, 1)], &[basic(Green), GOLD, BLACK]);
    let after = apply(
        state.clone(),
        &Event::CrystalUsed {
            player: SOLO,
            color: Red,
        },
    );
    assert_eq!(after.inventory(SOLO).crystals(Red), 0);
    let taken = Event::DieTaken {
        player: SOLO,
        die: 0,
        color: basic(Green),
    };
    let twice = apply(apply(state, &taken), &taken);
    assert_eq!(twice.dice_taken(SOLO), &[0]);
}
