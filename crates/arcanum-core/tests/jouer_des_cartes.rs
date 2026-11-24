//! Scénarios de la spec 013 (jouer des cartes).

use std::collections::BTreeMap;

use arcanum_core::{
    Action, ActionError, AttackKind, BasicAction, BasicColor, BasicEffect, CardId, CardPlay, Cost,
    DeedCard, Effect, Element, Event, GameConfig, GameRunner, GameState, Hero, PlayedCard,
    PlayerEvent, PlayerId, Scenario, SidewaysEffect, apply, decide, legal_actions, new_game,
    pending_decision, view,
};

mod premier_choix;
use premier_choix::FirstChoice;

const SOLO: PlayerId = PlayerId::SOLO;
const OTHER: PlayerId = PlayerId(1);

use BasicAction::*;

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

fn start(config: &GameConfig) -> (GameState, Vec<Event>) {
    let mut rng = config.rngs();
    let events = new_game(config, &mut rng);
    let state = events.iter().fold(GameState::default(), apply);
    (state, events)
}

/// Partie dont la main est exactement `hand` (piochée dans cet ordre), le reste du deck derrière.
fn with_hand(hand: &[CardId]) -> GameState {
    let hero = Hero::Arythea;
    let mut order = hand.to_vec();
    order.extend(
        hero.starting_deck()
            .into_iter()
            .filter(|card| !hand.contains(card)),
    );
    let mut events = vec![
        Event::GameStarted {
            config: config(0, hero),
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

fn play(card: CardId, play: CardPlay) -> Action {
    Action::PlayCard { card, play }
}

fn basic(option: u8) -> CardPlay {
    CardPlay::Basic {
        die: None,
        gain: None,
        with: None,
        option,
        discard: None,
        mana: None,
    }
}

fn basic_discarding(option: u8, discard: CardId) -> CardPlay {
    CardPlay::Basic {
        die: None,
        gain: None,
        with: None,
        option,
        discard: Some(discard),
        mana: None,
    }
}

/// Valide l'action et applique ses événements.
fn played(state: &GameState, action: Action) -> GameState {
    let events = decide(state, SOLO, &action, &mut config(0, Hero::Arythea).rngs())
        .expect("l'action doit être légale");
    events.iter().fold(state.clone(), apply)
}

fn composition(deck: &[CardId]) -> BTreeMap<BasicAction, usize> {
    let mut counts = BTreeMap::new();
    for card in deck {
        let DeedCard::Basic(action) = card.card() else {
            panic!("pas de Blessure dans un deck de départ");
        };
        *counts.entry(action).or_insert(0) += 1;
    }
    counts
}

fn expected(cards: &[(BasicAction, usize)]) -> BTreeMap<BasicAction, usize> {
    cards.iter().copied().collect()
}

// --- Contenu ---

// Spec 013 C1 — MKUE p. 3 ; docs/reference/decks-de-depart.md
#[test]
fn starting_decks_match_hero_cards() {
    let common = [
        (Rage, 2),
        (Determination, 1),
        (Swiftness, 2),
        (March, 2),
        (Stamina, 2),
        (Tranquility, 1),
        (Promise, 1),
        (Threaten, 1),
        (Crystallize, 1),
        (ManaDraw, 1),
        (Concentration, 1),
        (Improvisation, 1),
    ];
    let deck_of = |replaced: [(BasicAction, BasicAction); 2]| {
        let mut counts = expected(&common);
        for (old, new) in replaced {
            let count = counts.get_mut(&old).expect("carte remplacée présente");
            *count -= 1;
            if *count == 0 {
                counts.remove(&old);
            }
            *counts.entry(new).or_insert(0) += 1;
        }
        counts
    };

    for (hero, replaced) in [
        (
            Hero::Arythea,
            [(Rage, BattleVersatility), (ManaDraw, ManaPull)],
        ),
        (
            Hero::Goldyx,
            [(Concentration, WillFocus), (Crystallize, CrystalJoy)],
        ),
        (
            Hero::Norowas,
            [(Promise, NobleManners), (Tranquility, Rejuvenate)],
        ),
        (
            Hero::Tovak,
            [(Determination, ColdToughness), (Improvisation, Instinct)],
        ),
    ] {
        let deck = hero.starting_deck();
        assert_eq!(deck.len(), 16, "{hero:?}");
        assert_eq!(composition(&deck), deck_of(replaced), "{hero:?}");
    }
}

// Spec 013 C2 — effets lus sur les scans (../mage-knight/docs/regles/scans/deed/), MKUE p. 4 (couleur)
#[test]
fn basic_action_definitions_match_cards() {
    use BasicColor::{Blue, Green, Red, White};
    let attack = |amount| Effect::Attack {
        amount,
        element: Element::Physical,
        kind: AttackKind::Melee,
    };
    let block = |amount| Effect::Block {
        amount,
        element: Element::Physical,
    };
    let options = |options: &'static [Effect]| BasicEffect::Options {
        options,
        cost: Cost::None,
        modifiers: &[],
        mana: &[],
    };
    const MOVE_2: &[Effect] = &[Effect::Move(2)];
    const INFLUENCE_2: &[Effect] = &[Effect::Influence(2)];

    let cases: Vec<(BasicAction, BasicColor, BasicEffect)> = vec![
        (March, Green, options(MOVE_2)),
        (Stamina, Blue, options(MOVE_2)),
        (Swiftness, White, options(MOVE_2)),
        (Promise, White, options(INFLUENCE_2)),
        (Threaten, Red, options(INFLUENCE_2)),
        // Spec 040 : la gloire en plus pendant une interaction.
        (
            NobleManners,
            White,
            options(&[Effect::InfluenceInInteraction {
                amount: 2,
                fame: 1,
                reputation: 0,
            }]),
        ),
        (
            Tranquility,
            Green,
            options(&[Effect::Heal(1), Effect::DrawCards(1)]),
        ),
        // Cristallisation, Concentration, Pouvoir de concentration, Euphorie cristalline,
        // Rajeunissement, Source de mana et Attraction de mana : vérifiées par la spec 025
        // (`mana_card_definitions_match_cards`).
    ];
    for (action, color, effect) in cases {
        let definition = action.definition();
        assert_eq!(definition.color, color, "{action:?}");
        assert_eq!(definition.basic, effect, "{action:?}");
    }

    let options_of = |action: BasicAction| match action.definition().basic {
        BasicEffect::Options { options, cost, .. } => (options.to_vec(), cost),
        BasicEffect::Unsupported => panic!("{action:?} doit avoir des options"),
    };
    assert_eq!(Rage.definition().color, Red);
    assert_eq!(options_of(Rage), (vec![attack(2), block(2)], Cost::None));
    assert_eq!(Determination.definition().color, Blue);
    assert_eq!(
        options_of(Determination),
        (vec![attack(2), block(2)], Cost::None)
    );
    assert_eq!(Improvisation.definition().color, Red);
    assert_eq!(
        options_of(Improvisation),
        (
            vec![Effect::Move(3), Effect::Influence(3), attack(3), block(3)],
            Cost::DiscardAnotherCard
        )
    );
    assert_eq!(BattleVersatility.definition().color, Red);
    assert_eq!(
        options_of(BattleVersatility),
        (
            vec![
                attack(2),
                block(2),
                Effect::Attack {
                    amount: 1,
                    element: Element::Physical,
                    kind: AttackKind::Ranged
                }
            ],
            Cost::None
        )
    );
    // Tranché par Mathieu le 2026-10-25 : version révisée (Lost Legion, Ultimate), Attaque 2 physique.
    assert_eq!(ColdToughness.definition().color, Blue);
    assert_eq!(
        options_of(ColdToughness),
        (
            vec![
                attack(2),
                Effect::Block {
                    amount: 3,
                    element: Element::Ice
                }
            ],
            Cost::None
        )
    );
    assert_eq!(Instinct.definition().color, Red);
    assert_eq!(
        options_of(Instinct),
        (
            vec![Effect::Move(2), Effect::Influence(2), attack(2), block(2)],
            Cost::None
        )
    );
    assert_eq!(BasicAction::ALL.len(), 20);
}

// Spec 013 C3
#[test]
fn card_ids_name_a_copy_of_a_card() {
    let deck = Hero::Arythea.starting_deck();
    let marches: Vec<CardId> = deck
        .iter()
        .copied()
        .filter(|card| card.card() == DeedCard::Basic(March))
        .collect();
    assert_eq!(marches.len(), 2);
    assert_ne!(marches[0], marches[1]);
    let mut unique = deck.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(unique.len(), 16);
    assert_eq!(card(March, 2).card(), DeedCard::Basic(March));
    assert_eq!(card(March, 2).copy(), 2);
}

// --- Mise en place ---

// Spec 013 C4 — MKUE p. 3 : 5 cartes en main (limite de main au niveau 1)
#[test]
fn setup_draws_five_cards_from_hero_deck() {
    let (state, _) = start(&config(3, Hero::Tovak));
    assert_eq!(state.hand(SOLO).len(), 5);
    assert_eq!(state.deck(SOLO).len(), 11);
    let mut all: Vec<CardId> = state.hand(SOLO).to_vec();
    all.extend_from_slice(state.deck(SOLO));
    all.sort();
    let mut expected = Hero::Tovak.starting_deck();
    expected.sort();
    assert_eq!(all, expected);
}

// Spec 013 C5 — MKUE p. 3
#[test]
fn setup_events_end_with_five_draws() {
    let (_, events) = start(&config(4, Hero::Goldyx));
    let shuffled = events
        .iter()
        .position(|event| matches!(event, Event::DeckShuffled { .. }))
        .expect("le deck est mélangé");
    let Event::DeckShuffled { order, .. } = &events[shuffled] else {
        unreachable!()
    };
    // Spec 020 : le lancer de la Source vient après la main de départ ; spec 030 : les piles
    // d'ennemis et leurs jetons viennent après la Source.
    let rolled = events
        .iter()
        .position(|event| matches!(event, Event::SourceRolled { .. }))
        .expect("la Source est lancée");
    let (last, before) = events[shuffled + 1..=rolled]
        .split_last()
        .expect("des événements suivent le mélange");
    assert!(matches!(last, Event::SourceRolled { .. }));
    let draws: Vec<CardId> = before
        .iter()
        .map(|event| match event {
            Event::CardDrawn { player, card } if *player == SOLO => *card,
            other => panic!("seules des pioches suivent le mélange : {other:?}"),
        })
        .collect();
    assert_eq!(draws, order[..5]);
}

// Spec 013 C6
#[test]
fn starting_hand_depends_only_on_seed() {
    let hand = |seed| start(&config(seed, Hero::Arythea)).0.hand(SOLO).to_vec();
    assert!((1..20).any(|seed| hand(seed) != hand(0)));
    assert_eq!(
        start(&config(9, Hero::Arythea)).1,
        start(&config(9, Hero::Arythea)).1
    );
}

// --- Jouer une carte ---

// Spec 013 C7 — MKUE p. 4, 2 et 3.a
#[test]
fn playing_march_gives_two_move_points() {
    let march = card(March, 1);
    let state = with_hand(&[march, card(Rage, 1)]);

    let events = decide(
        &state,
        SOLO,
        &play(march, basic(0)),
        &mut config(0, Hero::Arythea).rngs(),
    )
    .expect("Marche se joue pour son effet de base");
    assert_eq!(
        events,
        vec![Event::CardPlayed {
            player: SOLO,
            card: march,
            play: basic(0),
            effect: Effect::Move(2),
        }]
    );

    let state = events.iter().fold(state, apply);
    assert!(!state.hand(SOLO).contains(&march));
    assert_eq!(
        state.play_area(SOLO),
        &[PlayedCard {
            card: march,
            play: basic(0),
            effect: Effect::Move(2)
        }][..]
    );
    assert_eq!(state.move_points(SOLO), 2);
    assert_eq!(state.influence_points(SOLO), 0);
}

// Spec 013 C8 — MKUE p. 4, 3.a
#[test]
fn playing_promise_gives_two_influence_points() {
    let promise = card(Promise, 1);
    let state = played(&with_hand(&[promise]), play(promise, basic(0)));
    assert_eq!(state.influence_points(SOLO), 2);
    assert_eq!(state.move_points(SOLO), 0);
}

// Spec 013 C9 — MKUE p. 4, 3.e : toute carte se joue de côté
#[test]
fn any_card_can_be_played_sideways() {
    let concentration = card(Concentration, 1);
    let rage = card(Rage, 1);
    let state = with_hand(&[concentration, rage]);

    let state = played(
        &state,
        play(concentration, CardPlay::Sideways(SidewaysEffect::Move)),
    );
    assert_eq!(state.move_points(SOLO), 1);
    let state = played(
        &state,
        play(rage, CardPlay::Sideways(SidewaysEffect::Influence)),
    );
    assert_eq!(state.influence_points(SOLO), 1);
    assert_eq!(state.play_area(SOLO)[1].effect, Effect::Influence(1));
}

// Spec 013 C10 — MKUE p. 4, 4 : les effets semblables s'additionnent
#[test]
fn move_effects_add_up() {
    let (march, swiftness, rage) = (card(March, 1), card(Swiftness, 1), card(Rage, 1));
    let mut state = with_hand(&[march, swiftness, rage]);
    for action in [
        play(march, basic(0)),
        play(swiftness, basic(0)),
        play(rage, CardPlay::Sideways(SidewaysEffect::Move)),
    ] {
        state = played(&state, action);
    }
    assert_eq!(state.move_points(SOLO), 5);
    assert_eq!(state.influence_points(SOLO), 0);
    let order: Vec<CardId> = state.play_area(SOLO).iter().map(|p| p.card).collect();
    assert_eq!(order, vec![march, swiftness, rage]);
    assert!(state.hand(SOLO).is_empty());
}

// Spec 013 C11 — MKUE p. 4, 4
#[test]
fn move_and_influence_are_kept_apart() {
    let (march, threaten) = (card(March, 1), card(Threaten, 1));
    let state = played(&with_hand(&[march, threaten]), play(march, basic(0)));
    let state = played(&state, play(threaten, basic(0)));
    assert_eq!(state.move_points(SOLO), 2);
    assert_eq!(state.influence_points(SOLO), 2);
}

// Spec 013 C12 — MKUE p. 6, 5.c : hors combat, seulement mouvement et influence
#[test]
fn choice_effects_offer_only_move_and_influence_out_of_combat() {
    let instinct = card(Instinct, 1);
    let manners = card(NobleManners, 1);
    let state = with_hand(&[instinct, manners]);
    let legal = legal_actions(&state, SOLO);

    let basics_of = |target: CardId| -> Vec<CardPlay> {
        legal
            .iter()
            .filter_map(|action| match action {
                Action::PlayCard { card, play } if *card == target => {
                    matches!(play, CardPlay::Basic { .. }).then_some(*play)
                }
                _ => None,
            })
            .collect()
    };
    assert_eq!(basics_of(instinct), vec![basic(0), basic(1)]);
    assert_eq!(basics_of(manners), vec![basic(0)]);

    let state = played(&state, play(instinct, basic(1)));
    assert_eq!(state.influence_points(SOLO), 2);
}

// Spec 013 C13 — MKUE p. 4, 6 ; p. 5 (défausser)
#[test]
fn improvisation_discards_another_card() {
    let (improvisation, march, rage) = (card(Improvisation, 1), card(March, 1), card(Rage, 1));
    let state = with_hand(&[improvisation, march, rage]);

    let basics: Vec<CardPlay> = legal_actions(&state, SOLO)
        .into_iter()
        .filter_map(|action| match action {
            Action::PlayCard { card, play } if card == improvisation => {
                matches!(play, CardPlay::Basic { .. }).then_some(play)
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        basics,
        vec![
            basic_discarding(0, march),
            basic_discarding(0, rage),
            basic_discarding(1, march),
            basic_discarding(1, rage),
        ]
    );

    let action = play(improvisation, basic_discarding(0, march));
    let events = decide(&state, SOLO, &action, &mut config(0, Hero::Arythea).rngs())
        .expect("Improvisation se joue en défaussant Marche");
    assert_eq!(
        events,
        vec![
            Event::CardPlayed {
                player: SOLO,
                card: improvisation,
                play: basic_discarding(0, march),
                effect: Effect::Move(3),
            },
            Event::CardDiscarded {
                player: SOLO,
                card: march
            },
        ]
    );
    let state = events.iter().fold(state, apply);
    assert_eq!(state.hand(SOLO), &[rage][..]);
    assert_eq!(state.discard_pile(SOLO), &[march][..]);
    assert_eq!(state.move_points(SOLO), 3);
}

// Spec 013 C14 — énumération exhaustive, dans l'ordre de la spec
#[test]
fn legal_actions_are_exhaustive() {
    let (march, rage, improvisation) = (card(March, 1), card(Rage, 1), card(Improvisation, 1));
    let state = with_hand(&[march, rage, improvisation]);
    let sideways = |card| {
        [
            play(card, CardPlay::Sideways(SidewaysEffect::Move)),
            play(card, CardPlay::Sideways(SidewaysEffect::Influence)),
        ]
    };
    let mut expected = vec![play(march, basic(0))];
    expected.extend(sideways(march));
    expected.extend(sideways(rage));
    expected.extend([
        play(improvisation, basic_discarding(0, march)),
        play(improvisation, basic_discarding(0, rage)),
        play(improvisation, basic_discarding(1, march)),
        play(improvisation, basic_discarding(1, rage)),
    ]);
    expected.extend(sideways(improvisation));
    // Spec 016 : puis les fins de tour ; rien n'est joué, il faut défausser au moins une carte.
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
        assert!(
            decide(&state, SOLO, action, &mut config(0, Hero::Arythea).rngs()).is_ok(),
            "{action:?}"
        );
    }
}

// Spec 013 C15, devenu spec 016 C10 : main vide, la décision continue ; il ne reste qu'à
// finir le tour.
#[test]
fn empty_hand_leaves_only_end_turn() {
    let (march, rage) = (card(March, 1), card(Rage, 1));
    let mut state = with_hand(&[march, rage]);
    for held in [march, rage] {
        state = played(
            &state,
            play(held, CardPlay::Sideways(SidewaysEffect::Influence)),
        );
    }
    assert!(state.hand(SOLO).is_empty());
    assert!(pending_decision(&state).is_some());
    assert_eq!(
        legal_actions(&state, SOLO),
        vec![Action::EndTurn {
            discard: vec![],
            reclaim: None,
            glade: None,
            steady_tempo: false
        }]
    );
}

// --- Information cachée ---

// Spec 013 C16
#[test]
fn view_shows_play_area_and_points() {
    let (improvisation, march, swiftness) =
        (card(Improvisation, 1), card(March, 1), card(Swiftness, 1));
    let state = with_hand(&[improvisation, march, swiftness]);
    let state = played(&state, play(improvisation, basic_discarding(1, march)));

    let seen = view(&state, SOLO);
    assert_eq!(seen.hand, vec![swiftness]);
    assert_eq!(seen.deck_size, 13);
    assert_eq!(seen.play_area, state.play_area(SOLO).to_vec());
    assert_eq!(seen.move_points, 0);
    assert_eq!(seen.influence_points, 3);
    assert_eq!(seen.discard, vec![march]);
}

// Spec 013 C17
#[test]
fn played_cards_are_public() {
    let improvisation = card(Improvisation, 1);
    let march = card(March, 1);
    let mut runner = GameRunner::new(
        config(5, Hero::Arythea),
        vec![Box::new(FirstChoice::default())],
    );
    runner.step().expect("le premier pas doit réussir");

    let played = arcanum_core::Event::CardPlayed {
        player: SOLO,
        card: improvisation,
        play: basic_discarding(0, march),
        effect: Effect::Move(3),
    };
    assert_eq!(
        arcanum_core::filter_event(&played, OTHER),
        PlayerEvent::CardPlayed {
            player: SOLO,
            card: improvisation,
            play: basic_discarding(0, march),
            effect: Effect::Move(3),
        }
    );
    assert_eq!(
        arcanum_core::filter_event(
            &Event::CardDiscarded {
                player: SOLO,
                card: march
            },
            OTHER
        ),
        PlayerEvent::CardDiscarded {
            player: SOLO,
            card: march
        }
    );
    let filtered = runner.log().filtered_for(OTHER);
    assert!(
        filtered
            .iter()
            .any(|event| matches!(event, PlayerEvent::CardPlayed { .. }))
    );
    assert!(
        filtered
            .iter()
            .all(|event| !matches!(event, PlayerEvent::CardDrawn { card: Some(_), .. }))
    );
}

// --- Cas limites et refus ---

// Spec 013 L1
#[test]
fn playing_a_card_not_in_hand_is_refused() {
    let march = card(March, 1);
    let state = with_hand(&[march]);
    let rng = &mut config(0, Hero::Arythea).rngs();
    // Dans le deck, pas en main.
    assert_eq!(
        decide(&state, SOLO, &play(card(March, 2), basic(0)), rng),
        Err(ActionError::NotLegal)
    );
    let after = played(&state, play(march, basic(0)));
    assert_eq!(
        decide(&after, SOLO, &play(march, basic(0)), rng),
        Err(ActionError::NotLegal)
    );
}

// Spec 013 L2 — MKUE p. 6, 5.c
#[test]
fn combat_effects_are_refused_out_of_combat() {
    let rage = card(Rage, 1);
    let state = with_hand(&[rage]);
    let rng = &mut config(0, Hero::Arythea).rngs();
    for refused in [
        basic(0),
        basic(1),
        CardPlay::Sideways(SidewaysEffect::Attack),
        CardPlay::Sideways(SidewaysEffect::Block),
    ] {
        assert_eq!(
            decide(&state, SOLO, &play(rage, refused), rng),
            Err(ActionError::NotLegal),
            "{refused:?}"
        );
    }
}

// Spec 013 L3 : depuis la spec 034, plus aucune carte n'a d'effet de base « pas encore disponible » ;
// l'effet renforcé de Menace, encore indisponible, est refusé par
// `out_of_scope_strong_effects_are_not_offered` (effets_renforces.rs).

// Spec 013 L4 — MKUE p. 4, 6
#[test]
fn improvisation_needs_another_card() {
    let improvisation = card(Improvisation, 1);
    let alone = with_hand(&[improvisation]);
    assert_eq!(
        legal_actions(&alone, SOLO),
        vec![
            play(improvisation, CardPlay::Sideways(SidewaysEffect::Move)),
            play(improvisation, CardPlay::Sideways(SidewaysEffect::Influence)),
            // Spec 016 : finir le tour en la défaussant.
            Action::EndTurn {
                discard: vec![improvisation],
                reclaim: None,
                glade: None,
                steady_tempo: false
            },
        ]
    );
    let rng = &mut config(0, Hero::Arythea).rngs();
    let with_march = with_hand(&[improvisation, card(March, 1)]);
    for refused in [basic(0), basic_discarding(0, improvisation)] {
        assert_eq!(
            decide(&with_march, SOLO, &play(improvisation, refused), rng),
            Err(ActionError::NotLegal),
            "{refused:?}"
        );
    }
    // Défausser une carte qui n'est pas en main.
    assert_eq!(
        decide(
            &with_march,
            SOLO,
            &play(improvisation, basic_discarding(0, card(Rage, 2))),
            rng
        ),
        Err(ActionError::NotLegal)
    );
    // Une option qui ne défausse pas ne prend pas de carte à défausser.
    let march = card(March, 1);
    assert_eq!(
        decide(
            &with_march,
            SOLO,
            &play(march, basic_discarding(0, improvisation)),
            rng
        ),
        Err(ActionError::NotLegal)
    );
}

// Spec 013 L5
#[test]
fn unknown_option_is_refused() {
    let march = card(March, 1);
    let state = with_hand(&[march]);
    assert_eq!(
        decide(
            &state,
            SOLO,
            &play(march, basic(3)),
            &mut config(0, Hero::Arythea).rngs()
        ),
        Err(ActionError::NotLegal)
    );
}

// Spec 013 L6
#[test]
fn invalid_card_event_changes_nothing() {
    let march = card(March, 1);
    let state = with_hand(&[march]);
    let elsewhere = card(Rage, 1);
    let after = apply(
        state.clone(),
        &Event::CardPlayed {
            player: SOLO,
            card: elsewhere,
            play: basic(0),
            effect: Effect::Move(2),
        },
    );
    assert_eq!(after, state);
    let after = apply(
        state.clone(),
        &Event::CardDiscarded {
            player: SOLO,
            card: elsewhere,
        },
    );
    assert_eq!(after, state);
}
