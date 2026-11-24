//! Scénarios de la spec 060 (Tactiques) : le choix au début de chaque manche de Première
//! reconnaissance, le retrait des tactiques prises, et les douze effets.

mod arene;

use arcanum_core::{
    Action, Artifact, BasicAction, BasicColor, CardId, CardPlay, DeedCard, Effect, Event,
    GameConfig, GameState, Hero, LastTurn, ManaColor, ManaPayment, PlayerEvent, PlayerId, Scenario,
    Skill, Tactic, TacticSlot, TacticView, TileId, TimeOfDay, UndoStatus, Unit, UnitCard, apply,
    filter_event, legal_actions, new_game, view,
};
use arene::*;

use BasicAction::*;

fn reco_config(seed: u64) -> GameConfig {
    GameConfig {
        seed,
        scenario: Scenario::first_reconnaissance().without_dummy(),
        hero: Hero::Tovak,
        dummy_hero: Default::default(),
    }
}

fn started(config: &GameConfig) -> GameState {
    new_game(config, &mut config.rngs())
        .iter()
        .fold(GameState::default(), apply)
}

/// Première reconnaissance juste après la mise en place : le choix de la tactique attend.
fn reco_start() -> GameState {
    started(&reco_config(1))
}

fn choose(tactic: Tactic) -> Action {
    Action::ChooseTactic { tactic }
}

fn choices(time: TimeOfDay) -> Vec<Action> {
    Tactic::of(time).map(choose).collect()
}

fn tactic_chosen(tactic: Tactic) -> Event {
    Event::TacticChosen {
        player: SOLO,
        tactic,
    }
}

fn apply_all(state: GameState, events: &[Event]) -> GameState {
    events.iter().fold(state, apply)
}

/// Le deck pioché en entier : la fin de la manche peut s'annoncer (MKUE p. 6, 3.a).
fn deck_drawn(state: &GameState) -> GameState {
    let drawn: Vec<Event> = state
        .deck(SOLO)
        .iter()
        .map(|&card| Event::CardDrawn { player: SOLO, card })
        .collect();
    apply_all(state.clone(), &drawn)
}

/// Prend `tactic`, pioche tout le deck, et annonce la fin de la manche : la manche suivante
/// attend son choix.
fn next_round(state: &GameState, tactic: Tactic) -> GameState {
    let state = deck_drawn(&done(state, choose(tactic)));
    done(&state, Action::AnnounceEndOfRound)
}

// --- Choix et retrait ---

// Spec 060 C1 — MKUE p. 4, 3.a ; p. 3 (*Displayed Day Tactics*) : après la mise en place, le
// choix de la tactique, parmi les six de Jour, main déjà piochée.
#[test]
fn first_round_starts_with_tactic_choice() {
    let state = reco_start();
    assert_eq!(legal_actions(&state, SOLO), choices(TimeOfDay::Day));
    assert_eq!(state.hand(SOLO).len(), 5);
}

// Spec 060 C2 — Compendium, *Motivation* S6, *Turn Start* S1 : tant que le choix attend, rien
// d'autre ; ni carte, ni déplacement, ni Motivation, ni annonce de fin de manche.
#[test]
fn nothing_before_tactic_choice() {
    let state = apply_all(
        reco_start(),
        &[
            Event::SkillGained {
                player: SOLO,
                skill: Skill::MotivationTovak,
            },
            Event::RoundEnded { round: 1 },
            Event::RoundStarted {
                round: 2,
                time_of_day: TimeOfDay::Night,
            },
        ],
    );
    let state = deck_drawn(&state);
    assert_eq!(legal_actions(&state, SOLO), choices(TimeOfDay::Night));
}

// Spec 060 C3 — MKUE p. 4, 3 : la prise ouvre le tour 1.
#[test]
fn choosing_a_tactic_starts_the_turn() {
    let state = reco_start();
    let events = events_of(&state, &choose(Tactic::EarlyBird));
    assert_eq!(
        events,
        vec![
            tactic_chosen(Tactic::EarlyBird),
            Event::TurnStarted {
                player: SOLO,
                turn: 1,
            },
        ]
    );
    let state = apply_all(state, &events);
    let tactic = state.tactic(SOLO).expect("une tactique prise");
    assert_eq!(tactic.tactic, Tactic::EarlyBird);
    assert_eq!(tactic.tactic.number(), 1);
    assert!(!tactic.flipped);
    assert!(
        !legal_actions(&state, SOLO)
            .iter()
            .any(|action| matches!(action, Action::ChooseTactic { .. }))
    );
}

// Spec 060 C4 — Q2 : le bac à sable n'a pas de tactique.
#[test]
fn sandbox_has_no_tactics() {
    let state = started(&config(1));
    let actions = legal_actions(&state, SOLO);
    assert!(
        !actions
            .iter()
            .any(|action| matches!(action, Action::ChooseTactic { .. }))
    );
    assert!(
        actions
            .iter()
            .any(|action| matches!(action, Action::PlayCard { .. }))
    );
    assert_eq!(state.tactic(SOLO), None);
}

// Spec 060 C5 — MKUE p. 4, 2.f : à la manche de Nuit, après la pioche, les six de Nuit.
#[test]
fn night_round_offers_night_tactics() {
    let state = deck_drawn(&done(&reco_start(), choose(Tactic::Planning)));
    let events = events_of(&state, &Action::AnnounceEndOfRound);
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::TurnStarted { .. }))
    );
    let state = apply_all(state, &events);
    assert_eq!(state.round(), 2);
    assert_eq!(state.hand(SOLO).len(), 5);
    assert_eq!(state.tactic(SOLO), None);
    assert_eq!(legal_actions(&state, SOLO), choices(TimeOfDay::Night));
}

// Spec 060 C6 — MKUE p. 19, *Special Rules* ; Compendium, *Scenarios* Q8 : la tactique prise est
// retirée de la partie à la fin de la manche.
#[test]
fn taken_tactic_is_removed_at_round_end() {
    let state = deck_drawn(&done(&reco_start(), choose(Tactic::Planning)));
    let events = events_of(&state, &Action::AnnounceEndOfRound);
    assert_events_in_order(
        &events,
        &[
            Event::RoundEnded { round: 1 },
            Event::TacticsRemoved {
                tactics: vec![Tactic::Planning],
            },
        ],
    );
    let state = apply_all(state, &events);
    assert_eq!(state.removed_tactics(), &[Tactic::Planning]);
    let state = next_round(&state, Tactic::LongNight);
    let day: Vec<Action> = choices(TimeOfDay::Day)
        .into_iter()
        .filter(|action| *action != choose(Tactic::Planning))
        .collect();
    assert_eq!(legal_actions(&state, SOLO), day);
    let state = next_round(&state, Tactic::EarlyBird);
    let night: Vec<Action> = choices(TimeOfDay::Night)
        .into_iter()
        .filter(|action| *action != choose(Tactic::LongNight))
        .collect();
    assert_eq!(legal_actions(&state, SOLO), night);
}

fn peasants() -> UnitCard {
    UnitCard {
        unit: Unit::Peasants,
        copy: 1,
    }
}

// Spec 060 C7 — MKUE p. 4, 2.g puis 3 : le choix vient après la reprise des bannières et la
// pioche.
#[test]
fn tactic_choice_follows_banner_reclaim() {
    let banner = CardId::new(DeedCard::Artifact(Artifact::BannerOfGlory), 1);
    let hand = vec![card(March, 1), banner];
    let mut events = Sandbox {
        scenario: Scenario::first_reconnaissance().without_dummy(),
        hero: Hero::Tovak,
        figure: Some(PLAIN_W),
        hand: hand.clone(),
        ..Sandbox::default()
    }
    .events();
    events.push(Event::DeckShuffled {
        player: SOLO,
        order: hand.clone(),
    });
    events.extend(
        hand.iter()
            .map(|&card| Event::CardDrawn { player: SOLO, card }),
    );
    events.extend(unit_events(&[peasants()], &[]));
    events.push(Event::BannerAssigned {
        player: SOLO,
        card: banner,
        unit: peasants(),
    });
    let state = apply_all(GameState::default(), &events);
    let state = done(&state, Action::AnnounceEndOfRound);
    assert!(
        !legal_actions(&state, SOLO)
            .iter()
            .any(|action| matches!(action, Action::ChooseTactic { .. }))
    );
    let state = done(
        &state,
        Action::ReclaimBanners {
            banners: vec![banner],
        },
    );
    assert_eq!(state.round(), 2);
    assert_eq!(legal_actions(&state, SOLO), choices(TimeOfDay::Night));
}

// Spec 060 C8 — spec 044 ; Compendium, *Turn Start* S1, II : le jeton de la clairière au début
// du tour 1, donc après le choix.
#[test]
fn glade_token_after_tactic_choice() {
    let state = Sandbox {
        scenario: Scenario::first_reconnaissance().without_dummy(),
        hero: Hero::Tovak,
        figure: Some(GLADE),
        hand: move_hand().to_vec(),
        tactic_pending: true,
        ..Sandbox::default()
    }
    .state();
    assert_eq!(legal_actions(&state, SOLO), choices(TimeOfDay::Day));
    let events = events_of(&state, &choose(Tactic::EarlyBird));
    assert_events_in_order(
        &events,
        &[
            tactic_chosen(Tactic::EarlyBird),
            Event::TurnStarted {
                player: SOLO,
                turn: 1,
            },
            Event::ManaTokenGained {
                player: SOLO,
                color: ManaColor::Gold,
            },
        ],
    );
}

// Spec 060 L1 — une tactique de l'autre moment de la journée, ou retirée, est refusée.
#[test]
fn tactic_must_be_available() {
    let state = reco_start();
    refused(&state, choose(Tactic::LongNight));
    let state = next_round(&state, Tactic::Planning);
    let state = next_round(&state, Tactic::FromTheDusk);
    refused(&state, choose(Tactic::Planning));
    refused(&state, choose(Tactic::FromTheDusk));
}

// Spec 060 L2 — pendant un tour, pas de choix de tactique.
#[test]
fn no_tactic_choice_during_a_turn() {
    let state = done(&reco_start(), choose(Tactic::EarlyBird));
    refused(&state, choose(Tactic::Rethink));
}

// Spec 060 L9 — Compendium, *Variants* S5 : une seule tactique restante est seule proposée ;
// aucune, le tour commence sans tactique.
#[test]
fn choice_with_one_or_no_tactic_left() {
    let state = deck_drawn(&done(&reco_start(), choose(Tactic::EarlyBird)));
    let five: Vec<Tactic> = Tactic::of(TimeOfDay::Night).skip(1).collect();
    let after = apply_all(state.clone(), &[Event::TacticsRemoved { tactics: five }]);
    let after = done(&after, Action::AnnounceEndOfRound);
    assert_eq!(
        legal_actions(&after, SOLO),
        vec![choose(Tactic::FromTheDusk)]
    );
    let all: Vec<Tactic> = Tactic::of(TimeOfDay::Night).collect();
    let after = apply_all(state, &[Event::TacticsRemoved { tactics: all }]);
    let events = events_of(&after, &Action::AnnounceEndOfRound);
    assert!(events.contains(&Event::TurnStarted {
        player: SOLO,
        turn: 1
    }));
    let after = apply_all(after, &events);
    assert_eq!(after.tactic(SOLO), None);
    assert!(
        legal_actions(&after, SOLO)
            .iter()
            .any(|action| matches!(action, Action::PlayCard { .. }))
    );
}

// --- Jour : au choix ---

/// La main des décors des tactiques.
fn tactic_hand() -> Vec<CardId> {
    vec![
        card(March, 1),
        card(March, 2),
        card(Stamina, 1),
        card(Rage, 1),
        card(Promise, 1),
    ]
}

/// Première reconnaissance avec Tovak sur une plaine, `hand` en main et, si donné, `deck` derrière
/// (sinon le reste de son deck de départ, 11 cartes) ; le choix de la tactique attend, de `time`.
fn decor(time: TimeOfDay, hand: &[CardId], deck: Option<&[CardId]>) -> GameState {
    let mut events = Sandbox {
        scenario: Scenario::first_reconnaissance().without_dummy(),
        hero: Hero::Tovak,
        figure: Some(PLAIN_W),
        hand: hand.to_vec(),
        time,
        tactic_pending: true,
        ..Sandbox::default()
    }
    .events();
    if let Some(deck) = deck {
        let mut order = hand.to_vec();
        order.extend_from_slice(deck);
        events.push(Event::DeckShuffled {
            player: SOLO,
            order,
        });
        events.extend(
            hand.iter()
                .map(|&card| Event::CardDrawn { player: SOLO, card }),
        );
    }
    apply_all(GameState::default(), &events)
}

fn day_decor() -> GameState {
    decor(TimeOfDay::Day, &tactic_hand(), None)
}

fn rethink(cards: &[CardId]) -> Action {
    Action::RethinkDiscard {
        cards: cards.to_vec(),
    }
}

fn turn_one() -> Event {
    Event::TurnStarted {
        player: SOLO,
        turn: 1,
    }
}

// Spec 060 C9 — carte ; Compendium, *Rethink* S1 : défausser, piocher autant, puis remélanger la
// défausse dans le deck ; le tour 1 commence.
#[test]
fn rethink_discards_draws_and_reshuffles() {
    let state = day_decor();
    let events = events_of(&state, &choose(Tactic::Rethink));
    assert_eq!(events, vec![tactic_chosen(Tactic::Rethink)]);
    let state = apply_all(state, &events);
    let discarded = [card(March, 1), card(Rage, 1)];
    assert!(legal_actions(&state, SOLO).contains(&rethink(&discarded)));
    let top: Vec<CardId> = state.deck(SOLO)[..2].to_vec();
    let events = events_of(&state, &rethink(&discarded));
    assert_events_in_order(
        &events,
        &[
            Event::CardDiscarded {
                player: SOLO,
                card: discarded[0],
            },
            Event::CardDiscarded {
                player: SOLO,
                card: discarded[1],
            },
            Event::CardDrawn {
                player: SOLO,
                card: top[0],
            },
            Event::CardDrawn {
                player: SOLO,
                card: top[1],
            },
        ],
    );
    assert!(matches!(
        events[events.len() - 2],
        Event::DeckReshuffled { .. }
    ));
    assert_eq!(events.last(), Some(&turn_one()));
    let state = apply_all(state, &events);
    assert_eq!(state.deck(SOLO).len(), 11);
    assert_eq!(state.hand(SOLO).len(), 5);
    assert!(state.discard_pile(SOLO).is_empty());
    assert!(discarded.iter().all(|card| state.deck(SOLO).contains(card)));
    assert!(top.iter().all(|card| state.hand(SOLO).contains(card)));
}

// Spec 060 C10 — carte : aucune carte défaussée, rien ne change ; une Blessure se défausse.
#[test]
fn rethink_may_discard_wounds() {
    let state = done(&day_decor(), choose(Tactic::Rethink));
    let events = events_of(&state, &rethink(&[]));
    assert_eq!(events, vec![turn_one()]);
    let mut hand = tactic_hand();
    hand[4] = wound(1);
    let state = done(&decor(TimeOfDay::Day, &hand, None), choose(Tactic::Rethink));
    let legal = legal_actions(&state, SOLO);
    assert!(legal.contains(&rethink(&[wound(1)])));
    assert!(!legal.contains(&rethink(&hand[..4])));
    let state = done(&state, rethink(&[wound(1)]));
    assert!(!state.hand(SOLO).contains(&wound(1)));
    assert!(state.deck(SOLO).contains(&wound(1)));
}

// Spec 060 C19 — carte ; Compendium, *Great Start* S1 : deux cartes piochées, rien de défaussé.
#[test]
fn great_start_draws_two() {
    let state = day_decor();
    let top: Vec<CardId> = state.deck(SOLO)[..2].to_vec();
    let events = events_of(&state, &choose(Tactic::GreatStart));
    assert_eq!(
        events,
        vec![
            tactic_chosen(Tactic::GreatStart),
            Event::CardDrawn {
                player: SOLO,
                card: top[0],
            },
            Event::CardDrawn {
                player: SOLO,
                card: top[1],
            },
            turn_one(),
        ]
    );
    let state = apply_all(state, &events);
    assert_eq!(state.hand(SOLO).len(), 7);
    assert!(state.discard_pile(SOLO).is_empty());
}

// Spec 060 C30 — carte ; Compendium, *Preparation* S1, S2 : une carte du deck au choix en main,
// puis le deck mélangé.
#[test]
fn preparation_searches_the_deck() {
    let deck = [
        card(Swiftness, 1),
        card(Swiftness, 2),
        wound(1),
        card(Tranquility, 1),
    ];
    let state = decor(TimeOfDay::Night, &tactic_hand(), Some(&deck));
    let state = done(&state, choose(Tactic::Preparation));
    let prepare = |card| Action::PrepareCard { card };
    let mut distinct = vec![card(Swiftness, 1), wound(1), card(Tranquility, 1)];
    distinct.sort();
    assert_eq!(
        legal_actions(&state, SOLO),
        distinct.into_iter().map(prepare).collect::<Vec<_>>()
    );
    let events = events_of(&state, &prepare(card(Tranquility, 1)));
    assert_eq!(
        events[0],
        Event::CardSearched {
            player: SOLO,
            card: card(Tranquility, 1),
        }
    );
    assert!(matches!(events[1], Event::DeckReshuffled { .. }));
    assert_eq!(events.last(), Some(&turn_one()));
    let state = apply_all(state, &events);
    assert!(state.hand(SOLO).contains(&card(Tranquility, 1)));
    assert_eq!(state.hand(SOLO).len(), 6);
    assert_eq!(state.deck(SOLO).len(), 3);
}

// Spec 060 L4 — un deck trop court pioche ce qu'il peut ; Faux départ remet quand même les
// cartes défaussées dans le deck.
#[test]
fn short_deck_draws_what_it_can() {
    let deck = [card(Swiftness, 1)];
    let state = decor(TimeOfDay::Day, &tactic_hand(), Some(&deck));
    let after = done(&state, choose(Tactic::GreatStart));
    assert_eq!(after.hand(SOLO).len(), 6);
    assert!(after.deck(SOLO).is_empty());
    let after = done(&state, choose(Tactic::Rethink));
    let after = done(&after, rethink(&[card(March, 1), card(Rage, 1)]));
    assert_eq!(after.hand(SOLO).len(), 4);
    assert_eq!(after.deck(SOLO).len(), 2);
    assert!(after.discard_pile(SOLO).is_empty());
}

// Spec 060 L5 — Préparation avec un deck vide : rien à chercher, le tour 1 commence.
#[test]
fn preparation_with_empty_deck() {
    let state = decor(TimeOfDay::Night, &tactic_hand(), Some(&[]));
    let events = events_of(&state, &choose(Tactic::Preparation));
    assert_eq!(events, vec![tactic_chosen(Tactic::Preparation), turn_one()]);
}

// --- Vol de mana ---

fn steal(die: u8) -> Action {
    Action::StealDie { die }
}

/// L'effet renforcé de `card` payé par `mana`.
fn strong_with(card: CardId, mana: ManaPayment) -> Action {
    Action::PlayCard {
        card,
        play: CardPlay::Strong {
            gain: None,
            extra_mana: None,
            option: 0,
            discard: None,
            mana,
            set_dice: [None, None],
            with: None,
        },
    }
}

/// Vol de mana pris, le dé bleu (le 3e de la Source : vert, rouge, bleu, blanc) volé.
fn blue_stolen() -> GameState {
    let state = done(&day_decor(), choose(Tactic::ManaSteal));
    done(&state, steal(2))
}

/// Les paiements par un dé de la Source proposés dans les actions légales.
fn source_dice_offered(state: &GameState) -> Vec<u8> {
    let mut dice: Vec<u8> = legal_actions(state, SOLO)
        .iter()
        .filter_map(|action| match action {
            Action::PlayCard {
                play:
                    CardPlay::Strong {
                        mana: ManaPayment::Die(die),
                        ..
                    },
                ..
            } => Some(*die),
            _ => None,
        })
        .collect();
    dice.sort();
    dice.dedup();
    dice
}

// Spec 060 C11 — carte : les dés de couleur de base de la Source au choix ; le dé volé quitte la
// Source et se pose sur la tactique.
#[test]
fn mana_steal_takes_a_basic_die() {
    let state = apply_all(
        day_decor(),
        &[Event::SourceRolled {
            dice: vec![
                ManaColor::Gold,
                ManaColor::Basic(BasicColor::Red),
                ManaColor::Black,
                ManaColor::Basic(BasicColor::Blue),
            ],
        }],
    );
    let events = events_of(&state, &choose(Tactic::ManaSteal));
    assert_eq!(events, vec![tactic_chosen(Tactic::ManaSteal)]);
    let state = apply_all(state, &events);
    assert_eq!(legal_actions(&state, SOLO), vec![steal(1), steal(3)]);
    let events = events_of(&state, &steal(3));
    assert_eq!(
        events,
        vec![
            Event::DieStolen {
                player: SOLO,
                die: 3
            },
            turn_one()
        ]
    );
    let state = apply_all(state, &events);
    assert_eq!(state.tactic(SOLO).and_then(|t| t.stolen_die), Some(3));
    assert!(!source_dice_offered(&state).contains(&3));
}

// Spec 060 C12 — FAQ p1 ; Compendium, *Mana Steal* Q1 : le dé volé paie en plus du dé de la
// Source du tour.
#[test]
fn stolen_die_is_not_the_source_die() {
    let state = blue_stolen();
    let endurance = strong_with(card(Stamina, 1), ManaPayment::StolenDie);
    assert!(legal_actions(&state, SOLO).contains(&endurance));
    let events = events_of(&state, &endurance);
    assert_eq!(events[0], Event::StolenDieUsed { player: SOLO });
    let state = apply_all(state, &events);
    assert_eq!(state.move_points(SOLO), 4);
    assert!(state.tactic(SOLO).is_some_and(|t| t.flipped));
    // Le dé vert de la Source pour Marche renforcée, le même tour.
    assert!(legal_actions(&state, SOLO).contains(&strong(card(March, 1), 0)));
}

/// Finit le tour sans défausser.
fn end_turn(state: &GameState) -> GameState {
    let action = legal_actions(state, SOLO)
        .into_iter()
        .find(|action| matches!(action, Action::EndTurn { .. }))
        .expect("finir le tour");
    done(state, action)
}

// Spec 060 C13 — carte : le dé volé utilisé est relancé et rendu à la Source à la fin du tour.
#[test]
fn stolen_die_returns_after_use() {
    let state = done(
        &blue_stolen(),
        strong_with(card(Stamina, 1), ManaPayment::StolenDie),
    );
    let action = legal_actions(&state, SOLO)
        .into_iter()
        .find(|action| matches!(action, Action::EndTurn { .. }))
        .expect("finir le tour");
    let events = events_of(&state, &action);
    let color = events
        .iter()
        .find_map(|event| match event {
            Event::StolenDieReturned {
                player: SOLO,
                color,
            } => Some(*color),
            _ => None,
        })
        .expect("le dé volé rendu");
    let state = apply_all(state, &events);
    let tactic = state.tactic(SOLO).expect("Vol de mana");
    assert_eq!(tactic.stolen_die, None);
    assert!(tactic.flipped);
    assert_eq!(state.source()[2], color);
    assert!(!legal_actions(&state, SOLO).iter().any(|action| matches!(
        action,
        Action::PlayCard {
            play: CardPlay::Strong {
                mana: ManaPayment::StolenDie,
                ..
            },
            ..
        }
    )));
}

// Spec 060 C14 — MKUE p. 4, 2.b : un dé volé jamais utilisé reste sur la tactique de tour en
// tour ; à la manche suivante, la Source a de nouveau tous ses dés.
#[test]
fn unused_stolen_die_returns_with_new_round() {
    let state = end_turn(&blue_stolen());
    assert_eq!(state.tactic(SOLO).and_then(|t| t.stolen_die), Some(2));
    assert!(
        legal_actions(&state, SOLO)
            .contains(&strong_with(card(Stamina, 1), ManaPayment::StolenDie))
    );
    let state = deck_drawn(&state);
    let state = done(&state, Action::AnnounceEndOfRound);
    assert_eq!(state.tactic(SOLO), None);
    let state = done(&state, choose(Tactic::FromTheDusk));
    assert_eq!(state.tactic(SOLO).and_then(|t| t.stolen_die), None);
    assert_eq!(state.source().len(), 3);
}

// Spec 060 C15 — Compendium, *Who Needs Magic* S3 : après Magie inutile à +3, le dé volé paie
// encore.
#[test]
fn who_needs_magic_allows_stolen_die() {
    let state = apply_all(
        blue_stolen(),
        &[
            Event::SkillGained {
                player: SOLO,
                skill: Skill::WhoNeedsMagic,
            },
            Event::SkillUsed {
                player: SOLO,
                skill: Skill::WhoNeedsMagic,
                option: 1,
                effect: Effect::Nothing,
            },
        ],
    );
    assert!(source_dice_offered(&state).is_empty());
    assert!(
        legal_actions(&state, SOLO)
            .contains(&strong_with(card(Stamina, 1), ManaPayment::StolenDie))
    );
}

// Spec 060 L3 — sans dé de couleur de base dans la Source, la tactique se prend sans rien voler.
#[test]
fn mana_steal_without_basic_die() {
    let state = apply_all(
        day_decor(),
        &[Event::SourceRolled {
            dice: vec![ManaColor::Gold, ManaColor::Black, ManaColor::Gold],
        }],
    );
    let events = events_of(&state, &choose(Tactic::ManaSteal));
    assert_eq!(events, vec![tactic_chosen(Tactic::ManaSteal), turn_one()]);
    let state = apply_all(state, &events);
    assert_eq!(state.tactic(SOLO).and_then(|t| t.stolen_die), None);
}

// --- Planification ---

fn end_turn_discarding(cards: &[CardId]) -> Action {
    Action::EndTurn {
        discard: cards.to_vec(),
        reclaim: None,
        glade: None,
        steady_tempo: false,
    }
}

/// Planification prise, une carte jouée de côté, puis la fin du tour qui défausse `discard` :
/// la main après la pioche.
fn planning_hand_after(state: GameState, discard: &[CardId]) -> usize {
    let state = done(&state, choose(Tactic::Planning));
    let state = done(&state, sideways(state.hand(SOLO)[0]));
    done(&state, end_turn_discarding(discard)).hand(SOLO).len()
}

// Spec 060 C16 — carte ; MKUE p. 11, 7.b : avec au moins 2 cartes gardées avant la pioche, la
// limite de main + 1.
#[test]
fn planning_draws_one_more_with_two_cards() {
    assert_eq!(
        planning_hand_after(day_decor(), &[card(March, 2), card(Stamina, 1)]),
        6
    );
    assert_eq!(
        planning_hand_after(
            day_decor(),
            &[card(March, 2), card(Stamina, 1), card(Rage, 1)]
        ),
        5
    );
}

// Spec 060 C17 — FAQ p1 ; Compendium, *Planning* Q1 : une Blessure gardée compte.
#[test]
fn planning_counts_wounds() {
    let hand = [
        card(March, 1),
        card(March, 2),
        card(Stamina, 1),
        card(Promise, 1),
        wound(1),
    ];
    let state = decor(TimeOfDay::Day, &hand, None);
    assert_eq!(
        planning_hand_after(state, &[card(March, 2), card(Stamina, 1)]),
        6
    );
}

// Spec 060 C18 — Compendium, *Planning* S4 : avec une forteresse possédée voisine (+1, spec 043),
// jusqu'à 7.
#[test]
fn planning_stacks_with_keep() {
    let mut events = Sandbox {
        scenario: Scenario::first_reconnaissance().without_dummy(),
        hero: Hero::Tovak,
        figure: Some(NEAR_KEEP),
        hand: tactic_hand(),
        tactic_pending: true,
        ..Sandbox::default()
    }
    .events();
    events.push(Event::TilePlaced {
        tile: TileId::Countryside(3),
        position: KEEP_POSITION,
    });
    events.push(Event::SiteConquered {
        player: SOLO,
        hex: KEEP,
    });
    let state = apply_all(GameState::default(), &events);
    assert_eq!(state.keep_hand_bonus(SOLO), 1);
    assert_eq!(
        planning_hand_after(state, &[card(March, 2), card(Stamina, 1)]),
        7
    );
}

// --- Au bon moment ---

/// Joue la première carte de la main de côté, puis finit le tour sans défausser.
fn filler_turn_events(state: &GameState) -> Vec<Event> {
    let state = done(state, sideways(state.hand(SOLO)[0]));
    events_of(&state, &end_turn_discarding(&[]))
}

fn filler_turn(state: &GameState) -> GameState {
    let events = filler_turn_events(state);
    let after = done(state, sideways(state.hand(SOLO)[0]));
    apply_all(after, &events)
}

fn right_moment_taken() -> GameState {
    done(&day_decor(), choose(Tactic::TheRightMoment))
}

// Spec 060 C20 — carte ; Compendium, *The Right Moment* S4 : annoncé, un tour complet de plus
// après celui-ci ; la tactique est retournée.
#[test]
fn right_moment_grants_a_full_turn() {
    let state = filler_turn(&filler_turn(&right_moment_taken()));
    assert_eq!(state.turn(), 3);
    assert!(legal_actions(&state, SOLO).contains(&Action::AnnounceRightMoment));
    let events = events_of(&state, &Action::AnnounceRightMoment);
    assert_eq!(events, vec![Event::RightMomentAnnounced { player: SOLO }]);
    let state = apply_all(state, &events);
    let tactic = state.tactic(SOLO).expect("Au bon moment");
    assert!(tactic.flipped && tactic.right_moment);
    assert!(!legal_actions(&state, SOLO).contains(&Action::AnnounceRightMoment));
    let events = filler_turn_events(&state);
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::CardDrawn { .. }))
    );
    assert_eq!(
        events.last(),
        Some(&Event::TurnStarted {
            player: SOLO,
            turn: 4,
        })
    );
    let state = filler_turn(&state);
    assert!(!legal_actions(&state, SOLO).contains(&Action::AnnounceRightMoment));
}

/// Au bon moment pris, au dernier tour de Première reconnaissance (cité révélée, spec 063).
fn last_turn_with_right_moment() -> GameState {
    apply_all(
        right_moment_taken(),
        &[
            Event::LastTurnAnnounced { player: SOLO },
            Event::TurnStarted {
                player: SOLO,
                turn: 2,
            },
        ],
    )
}

fn ends_game(events: &[Event]) -> bool {
    events
        .iter()
        .any(|event| matches!(event, Event::GameEnded { .. }))
}

// Spec 060 C21 — Compendium, *The Right Moment* S3 ; *Scenarios* O30 : au dernier tour, le tour
// de plus se joue avant la fin de la partie.
#[test]
fn right_moment_extends_last_turn() {
    let state = last_turn_with_right_moment();
    assert_eq!(state.last_turn(), LastTurn::Current);
    let state = done(&state, Action::AnnounceRightMoment);
    let events = filler_turn_events(&state);
    assert!(!ends_game(&events), "{events:?}");
    let state = filler_turn(&state);
    assert_eq!(state.turn(), 3);
    assert!(ends_game(&filler_turn_events(&state)));
}

// Spec 060 C22 — Compendium, *The Right Moment* S1 : pas pendant un choix en attente, ni une fois
// la fin du tour engagée.
#[test]
fn right_moment_only_during_the_turn() {
    let state = done(&reco_start(), choose(Tactic::TheRightMoment));
    assert!(legal_actions(&state, SOLO).contains(&Action::AnnounceRightMoment));
    // Niveau 2 atteint : la fin du tour attend le choix de la compétence.
    let state = apply(
        state,
        &Event::EnemiesDefeated {
            player: SOLO,
            enemies: Vec::new(),
            fame: 3,
        },
    );
    let state = filler_turn(&state);
    assert!(
        legal_actions(&state, SOLO)
            .iter()
            .any(|action| matches!(action, Action::ChooseSkill { .. }))
    );
    assert!(!legal_actions(&state, SOLO).contains(&Action::AnnounceRightMoment));
}

// Spec 060 L7 — avec Distorsion temporelle (spec 057) au même tour, le tour sans pioche de la
// Distorsion d'abord, puis la pioche et le tour d'Au bon moment.
#[test]
fn right_moment_after_time_bending() {
    let state = done(&last_turn_with_right_moment(), Action::AnnounceRightMoment);
    // Le tour supplémentaire de Distorsion temporelle attend (état posé).
    let state = apply(
        state,
        &Event::CardsReturnedToHand {
            player: SOLO,
            cards: Vec::new(),
        },
    );
    let events = filler_turn_events(&state);
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::ExtraTurnStarted { .. }))
    );
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::CardDrawn { .. }))
    );
    let state = filler_turn(&state);
    let events = filler_turn_events(&state);
    assert!(!ends_game(&events), "{events:?}");
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::TurnStarted { .. }))
    );
    let state = filler_turn(&state);
    assert!(ends_game(&filler_turn_events(&state)));
}

// Spec 060 L8 — MKUE p. 4, 4.b : la fin de la manche annoncée au début du tour de plus le perd.
#[test]
fn round_end_cancels_the_extra_turn() {
    let state = done(&right_moment_taken(), Action::AnnounceRightMoment);
    let state = deck_drawn(&filler_turn(&state));
    let events = events_of(&state, &Action::AnnounceEndOfRound);
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::RoundEnded { round: 1 }))
    );
    let state = apply_all(state, &events);
    assert_eq!(state.round(), 2);
}

// --- Longue Nuit ---

/// Nuit, `hand` en main, `deck` derrière, `discard` en défausse ; Longue Nuit prise, tour 1.
fn long_night(hand: &[CardId], deck: &[CardId], discard: &[CardId]) -> GameState {
    let mut drawn = hand.to_vec();
    drawn.extend_from_slice(discard);
    let state = decor(TimeOfDay::Night, &drawn, Some(deck));
    let discarded: Vec<Event> = discard
        .iter()
        .map(|&card| Event::CardDiscarded { player: SOLO, card })
        .collect();
    done(&apply_all(state, &discarded), choose(Tactic::LongNight))
}

fn six_cards() -> Vec<CardId> {
    vec![
        card(Swiftness, 1),
        card(Swiftness, 2),
        card(Concentration, 1),
        card(Determination, 1),
        card(Improvisation, 1),
        card(Threaten, 1),
    ]
}

// Spec 060 C23 — carte ; FAQ p2 : deck vide, 3 cartes de la défausse au hasard retournent dans le
// deck ; la tactique est retournée.
#[test]
fn long_night_returns_three_cards() {
    let state = long_night(&tactic_hand(), &[], &six_cards());
    let legal = legal_actions(&state, SOLO);
    assert!(legal.contains(&Action::UseLongNight));
    assert!(legal.contains(&Action::AnnounceEndOfRound));
    let events = events_of(&state, &Action::UseLongNight);
    let [
        Event::LongNightUsed {
            player: SOLO,
            cards,
        },
    ] = events.as_slice()
    else {
        panic!("{events:?}");
    };
    assert_eq!(cards.len(), 3);
    assert!(cards.iter().all(|card| six_cards().contains(card)));
    let state = apply_all(state, &events);
    assert_eq!(state.deck(SOLO).len(), 3);
    assert_eq!(state.discard_pile(SOLO).len(), 3);
    assert!(state.tactic(SOLO).is_some_and(|tactic| tactic.flipped));
    assert!(!legal_actions(&state, SOLO).contains(&Action::UseLongNight));
}

// Spec 060 C24 — MKUE p. 11, 7.d, note ; Compendium, *Long Night* Q1 : la pioche de fin de tour
// s'arrête quand le deck s'épuise, et demande.
#[test]
fn long_night_during_end_of_turn_draw() {
    let deck = [card(Swiftness, 1), card(Swiftness, 2)];
    let state = long_night(&tactic_hand(), &deck, &[card(Threaten, 1)]);
    let state = done(&state, sideways(card(March, 1)));
    // Main de 1 après la défausse : 4 à piocher, 2 dans le deck.
    let discard = [card(March, 2), card(Stamina, 1), card(Rage, 1)];
    let events = events_of(&state, &end_turn_discarding(&discard));
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::TurnStarted { .. }))
    );
    let paused = apply_all(state, &events);
    assert_eq!(paused.hand(SOLO).len(), 3);
    assert_eq!(
        legal_actions(&paused, SOLO),
        vec![Action::UseLongNight, Action::DeclineLongNight]
    );
    let used = done(&paused, Action::UseLongNight);
    assert_eq!(used.hand(SOLO).len(), 5);
    assert_eq!(used.deck(SOLO).len(), 1);
    assert_eq!(used.turn(), 2);
    let declined = done(&paused, Action::DeclineLongNight);
    assert_eq!(declined.hand(SOLO).len(), 3);
    assert_eq!(declined.turn(), 2);
    assert!(declined.tactic(SOLO).is_some_and(|tactic| !tactic.flipped));
}

// Spec 060 C25 — carte : une fois dans la Nuit, et seulement avec une défausse.
#[test]
fn long_night_once_per_night() {
    let state = long_night(&tactic_hand(), &[], &six_cards());
    let state = done(&state, Action::UseLongNight);
    let state = done(&state, sideways(card(March, 1)));
    let events = events_of(&state, &end_turn_discarding(&[]));
    assert!(
        matches!(events.last(), Some(Event::TurnStarted { turn: 2, .. })),
        "{events:?}"
    );
    let empty = long_night(&tactic_hand(), &[], &[]);
    assert!(!legal_actions(&empty, SOLO).contains(&Action::UseLongNight));
}

// --- Appel du mana ---

/// Nuit, Source `dice`, Appel du mana pris : tour 1.
fn mana_search(dice: &[ManaColor]) -> GameState {
    let state = apply_all(
        decor(TimeOfDay::Night, &tactic_hand(), None),
        &[Event::SourceRolled {
            dice: dice.to_vec(),
        }],
    );
    done(&state, choose(Tactic::ManaSearch))
}

fn search(dice: &[u8]) -> Action {
    Action::ManaSearch {
        dice: dice.to_vec(),
    }
}

fn searches(state: &GameState) -> Vec<Action> {
    legal_actions(state, SOLO)
        .into_iter()
        .filter(|action| matches!(action, Action::ManaSearch { .. }))
        .collect()
}

const GREEN: ManaColor = ManaColor::Basic(BasicColor::Green);

// Spec 060 C26 — carte : 1 ou 2 dés relancés, les dés or (épuisés la nuit) d'abord.
#[test]
fn mana_search_rerolls_gold_first() {
    let state = mana_search(&[ManaColor::Gold, GREEN]);
    assert_eq!(searches(&state), vec![search(&[0]), search(&[0, 1])]);
    let events = events_of(&state, &search(&[0, 1]));
    assert_eq!(events[0], Event::ManaSearchUsed { player: SOLO });
    let colors: Vec<ManaColor> = events[1..]
        .iter()
        .map(|event| match event {
            Event::DieRerolled { color, .. } => *color,
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(colors.len(), 2);
    let state = apply_all(state, &events);
    assert_eq!(state.source(), colors.as_slice());
    assert!(
        state
            .tactic(SOLO)
            .is_some_and(|tactic| tactic.mana_search_used)
    );
}

// Spec 060 C27 — carte ; Compendium, *Mana Search* S2 : une fois par tour, avant de prendre un dé
// de la Source.
#[test]
fn mana_search_once_per_turn_before_source() {
    let state = mana_search(&[GREEN, GREEN, ManaColor::Gold]);
    let taken = done(&state, strong(card(March, 1), 0));
    assert!(searches(&taken).is_empty());
    let state = done(&state, search(&[2]));
    assert!(searches(&state).is_empty());
    let state = filler_turn(&state);
    assert!(!searches(&state).is_empty());
}

// Spec 060 L6 — un seul dé : 1 dé au plus ; deux dés or et un vert : les deux or seuls à deux dés.
#[test]
fn mana_search_dice_choices() {
    let state = mana_search(&[GREEN]);
    assert_eq!(searches(&state), vec![search(&[0])]);
    let state = mana_search(&[ManaColor::Gold, GREEN, ManaColor::Gold]);
    assert_eq!(
        searches(&state),
        vec![search(&[0]), search(&[2]), search(&[0, 2])]
    );
}

// --- Méditation nocturne ---

fn meditate(cards: &[CardId]) -> Action {
    Action::MidnightMeditation {
        cards: cards.to_vec(),
    }
}

fn meditations(state: &GameState) -> usize {
    legal_actions(state, SOLO)
        .iter()
        .filter(|action| matches!(action, Action::MidnightMeditation { .. }))
        .count()
}

/// Nuit, une Blessure en 5e carte de la main, Méditation nocturne prise : tour 1.
fn midnight_meditation() -> GameState {
    let mut hand = tactic_hand();
    hand[4] = wound(1);
    done(
        &decor(TimeOfDay::Night, &hand, None),
        choose(Tactic::MidnightMeditation),
    )
}

// Spec 060 C28 — carte ; Compendium, *Midnight Meditation* S2 : jusqu'à 5 cartes de la main,
// Blessures comprises, mélangées dans le deck, puis autant de piochées ; la tactique est retournée.
#[test]
fn midnight_meditation_swaps_cards() {
    let state = midnight_meditation();
    let chosen = [card(March, 1), card(Stamina, 1), wound(1)];
    let events = events_of(&state, &meditate(&chosen));
    assert_eq!(
        events[0],
        Event::CardsShuffledIntoDeck {
            player: SOLO,
            cards: chosen.to_vec(),
        }
    );
    let Event::DeckReshuffled { order, .. } = &events[1] else {
        panic!("{events:?}");
    };
    // Le deck de départ moins les 4 cartes en main (la Blessure n'en vient pas), plus les 3.
    assert_eq!(order.len(), 15);
    let drawn: Vec<CardId> = events[2..]
        .iter()
        .map(|event| match event {
            Event::CardDrawn { card, .. } => *card,
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(drawn, order[..3]);
    let state = apply_all(state, &events);
    assert_eq!(state.hand(SOLO).len(), 5);
    assert_eq!(state.deck(SOLO).len(), 12);
    assert!(state.tactic(SOLO).is_some_and(|tactic| tactic.flipped));
    assert_eq!(meditations(&state), 0);
}

// Spec 060 C29 — carte ; Compendium, *Turn Start* S1 : avant le tour, rien de fait ; dès le tour
// 1, juste après la prise.
#[test]
fn midnight_meditation_before_acting() {
    let state = midnight_meditation();
    // 1 à 5 cartes parmi 4 cartes et une Blessure : 31 choix.
    assert_eq!(meditations(&state), 31);
    let played = done(&state, sideways(card(March, 1)));
    assert_eq!(meditations(&played), 0);
    let next = filler_turn(&state);
    assert!(meditations(&next) > 0);
}

// --- Provisions ---

fn store() -> Action {
    Action::SparingPower { take: false }
}

fn take() -> Action {
    Action::SparingPower { take: true }
}

fn stored(state: &GameState) -> usize {
    state.tactic(SOLO).map_or(0, |tactic| tactic.stored_count())
}

/// Nuit, `hand` en main, `deck` derrière ; Provisions prise : le choix d'avant le tour 1 attend.
fn sparing_power(hand: &[CardId], deck: Option<&[CardId]>) -> GameState {
    done(
        &decor(TimeOfDay::Night, hand, deck),
        choose(Tactic::SparingPower),
    )
}

// Spec 060 C31 — carte ; Q4 : avant le tour 1, poser la carte du dessus du deck sous la tactique,
// face cachée.
#[test]
fn sparing_power_stores_top_card() {
    let state = sparing_power(&tactic_hand(), None);
    assert_eq!(legal_actions(&state, SOLO), vec![store(), take()]);
    let top = state.deck(SOLO)[0];
    let events = events_of(&state, &store());
    assert_eq!(
        events,
        vec![Event::CardStored {
            player: SOLO,
            card: top,
        }]
    );
    let state = apply_all(state, &events);
    assert_eq!(state.deck(SOLO).len(), 10);
    assert_eq!(stored(&state), 1);
    assert!(legal_actions(&state, SOLO).contains(&sideways(card(March, 1))));
}

// Spec 060 C32 — Compendium, *Sparing Power* S2 : le choix revient avant chaque tour ; retournée,
// les cartes posées vont en main et le choix ne revient plus.
#[test]
fn sparing_power_takes_stored_cards() {
    let state = done(&sparing_power(&tactic_hand(), None), store());
    let state = filler_turn(&state);
    assert_eq!(legal_actions(&state, SOLO), vec![store(), take()]);
    let state = done(&state, store());
    let state = filler_turn(&state);
    let hand = state.hand(SOLO).len();
    let events = events_of(&state, &take());
    assert_eq!(events, vec![Event::StoredCardsTaken { player: SOLO }]);
    let state = apply_all(state, &events);
    assert_eq!(state.hand(SOLO).len(), hand + 2);
    assert_eq!(stored(&state), 0);
    assert!(state.tactic(SOLO).is_some_and(|tactic| tactic.flipped));
    let state = filler_turn(&state);
    assert!(!legal_actions(&state, SOLO).contains(&take()));
}

/// Main vide, deck de 2 cartes ; Provisions : les deux posées, aux tours 1 et 2 (état posé).
fn two_stored_empty_handed() -> (GameState, [CardId; 2]) {
    let cards = [card(March, 1), card(Rage, 1)];
    let state = done(&sparing_power(&[], Some(&cards)), store());
    let state = apply(
        state,
        &Event::TurnStarted {
            player: SOLO,
            turn: 2,
        },
    );
    (done(&state, store()), cards)
}

// Spec 060 C33 — Compendium, *Turn Start* S1, I avant II ; MKUE p. 6, 3.b : deck vide, seul le
// retournement ; avec les cartes en main, l'annonce de la fin de la manche n'est plus obligatoire.
#[test]
fn sparing_power_before_forced_announcement() {
    let (state, _) = two_stored_empty_handed();
    let state = apply(
        state,
        &Event::TurnStarted {
            player: SOLO,
            turn: 3,
        },
    );
    assert_eq!(legal_actions(&state, SOLO), vec![take()]);
    let state = done(&state, take());
    assert_eq!(state.hand(SOLO).len(), 2);
    assert!(legal_actions(&state, SOLO).contains(&sideways(card(March, 1))));
}

// Spec 060 C34 — MKUE p. 4, 2.g : les cartes encore sous Provisions à la fin de la manche
// reviennent dans le nouveau deck.
#[test]
fn stored_cards_return_with_new_round() {
    let (state, cards) = two_stored_empty_handed();
    assert_eq!(stored(&state), 2);
    let round = state.round();
    let state = done(&state, Action::AnnounceEndOfRound);
    assert_eq!(state.round(), round + 1);
    let owned: Vec<CardId> = state
        .hand(SOLO)
        .iter()
        .chain(state.deck(SOLO))
        .copied()
        .collect();
    assert!(cards.iter().all(|card| owned.contains(card)), "{owned:?}");
}

// --- Vue et journal ---

fn slots(time: TimeOfDay, removed: &[Tactic]) -> Vec<(Tactic, TacticSlot)> {
    Tactic::of(time)
        .map(|tactic| {
            let slot = if removed.contains(&tactic) {
                TacticSlot::Removed
            } else {
                TacticSlot::Available
            };
            (tactic, slot)
        })
        .collect()
}

// Spec 060 C35 — la vue : l'offre pendant le choix (une case par tactique du moment, retirée ou
// non), les retirées, la tactique du joueur et son état, les cartes du choix de Préparation.
#[test]
fn view_shows_tactics() {
    let start = reco_start();
    let shown = view(&start, SOLO);
    assert_eq!(shown.tactic_choice, slots(TimeOfDay::Day, &[]));
    assert_eq!(shown.tactic, None);
    let night = next_round(&start, Tactic::EarlyBird);
    let day = next_round(&night, Tactic::FromTheDusk);
    let shown = view(&day, SOLO);
    assert_eq!(
        shown.removed_tactics,
        vec![Tactic::EarlyBird, Tactic::FromTheDusk]
    );
    assert_eq!(
        shown.tactic_choice,
        slots(TimeOfDay::Day, &[Tactic::EarlyBird])
    );
    let shown = view(&blue_stolen(), SOLO);
    assert!(shown.tactic_choice.is_empty());
    assert_eq!(
        shown.tactic,
        Some(TacticView {
            tactic: Tactic::ManaSteal,
            flipped: false,
            stolen_die: Some((2, ManaColor::Basic(BasicColor::Blue))),
            stored: 0,
            mana_search_used: false,
            right_moment: false,
        })
    );
    let preparing = done(
        &decor(TimeOfDay::Night, &tactic_hand(), None),
        choose(Tactic::Preparation),
    );
    let mut deck = preparing.deck(SOLO).to_vec();
    deck.sort();
    assert_eq!(view(&preparing, SOLO).preparation_choice, deck);
    let prepared = done(&preparing, Action::PrepareCard { card: deck[0] });
    assert!(view(&prepared, SOLO).preparation_choice.is_empty());
    let sparing = done(&sparing_power(&tactic_hand(), None), store());
    assert_eq!(
        view(&sparing, SOLO).tactic.map(|tactic| tactic.stored),
        Some(1)
    );
}

// Spec 060 C36 — spec 000 : les événements des tactiques sont publics, sauf la carte cherchée
// (Préparation) et la carte posée (Provisions).
#[test]
fn tactic_events_are_filtered() {
    let other = PlayerId(1);
    let public = [
        tactic_chosen(Tactic::ManaSteal),
        Event::DieStolen {
            player: SOLO,
            die: 2,
        },
        Event::RightMomentAnnounced { player: SOLO },
        Event::TacticsRemoved {
            tactics: vec![Tactic::EarlyBird],
        },
    ];
    for event in public {
        let filtered = format!("{:?}", filter_event(&event, other));
        assert_eq!(filtered, format!("{event:?}"));
    }
    let searched = Event::CardSearched {
        player: SOLO,
        card: card(March, 1),
    };
    assert_eq!(
        filter_event(&searched, other),
        PlayerEvent::CardSearched {
            player: SOLO,
            card: None,
        }
    );
    let stored = Event::CardStored {
        player: SOLO,
        card: card(March, 1),
    };
    for viewer in [SOLO, other] {
        assert_eq!(
            filter_event(&stored, viewer),
            PlayerEvent::CardStored { player: SOLO }
        );
    }
}

// --- Retour en arrière et rejeu ---

/// Les événements de la mise en place du décor de Jour, le choix de la tactique en attente.
fn day_decor_events() -> Vec<Event> {
    Sandbox {
        scenario: Scenario::first_reconnaissance().without_dummy(),
        hero: Hero::Tovak,
        figure: Some(PLAIN_W),
        hand: tactic_hand(),
        tactic_pending: true,
        ..Sandbox::default()
    }
    .events()
}

// Spec 060 C37 — spec 024 : la prise ne se reprend pas (le tour 1 la suit) ; l'annonce d'Au bon
// moment, si.
#[test]
fn right_moment_can_be_undone() {
    let (mut runner, puppet) = runner(day_decor_events());
    play(&mut runner, &puppet, choose(Tactic::TheRightMoment));
    assert_ne!(runner.undo_status(), UndoStatus::Possible);
    play(&mut runner, &puppet, Action::AnnounceRightMoment);
    assert_eq!(runner.undo_status(), UndoStatus::Possible);
    assert_eq!(runner.undo(), Ok(()));
    let tactic = runner.state().tactic(SOLO).expect("Au bon moment");
    assert!(!tactic.flipped && !tactic.right_moment);
}

/// Une partie jouée action par action, et son journal.
struct Recorded {
    state: GameState,
    log: Vec<Event>,
}

impl Recorded {
    fn act(&mut self, action: Action) {
        let events = events_of(&self.state, &action);
        self.posed(&events);
    }

    fn posed(&mut self, events: &[Event]) {
        self.state = apply_all(self.state.clone(), events);
        self.log.extend_from_slice(events);
    }

    /// Le deck pioché en entier, puis la fin de la manche annoncée.
    fn end_round(&mut self) {
        let drawn: Vec<Event> = self
            .state
            .deck(SOLO)
            .iter()
            .map(|&card| Event::CardDrawn { player: SOLO, card })
            .collect();
        self.posed(&drawn);
        self.act(Action::AnnounceEndOfRound);
    }
}

// Spec 060 C38 — ADR 0001 : rejouer le journal de quatre manches à tactiques redonne le même état.
#[test]
fn replaying_log_rebuilds_tactics() {
    let log = day_decor_events();
    let mut game = Recorded {
        state: apply_all(GameState::default(), &log),
        log,
    };
    // Manche 1, Jour : Vol de mana, le dé volé payé puis rendu.
    game.act(choose(Tactic::ManaSteal));
    game.act(steal(2));
    game.act(strong_with(card(Stamina, 1), ManaPayment::StolenDie));
    game.act(end_turn_discarding(&[]));
    game.end_round();
    // Manche 2, Nuit : Provisions, une carte posée.
    game.act(choose(Tactic::SparingPower));
    game.act(store());
    game.end_round();
    // Manche 3, Jour : Au bon moment annoncé.
    game.act(choose(Tactic::TheRightMoment));
    game.act(Action::AnnounceRightMoment);
    game.end_round();
    // Manche 4, Nuit : Longue Nuit, deck vide, une carte en défausse (état posé).
    game.act(choose(Tactic::LongNight));
    let drawn: Vec<Event> = game
        .state
        .deck(SOLO)
        .iter()
        .map(|&card| Event::CardDrawn { player: SOLO, card })
        .collect();
    game.posed(&drawn);
    let card = game.state.hand(SOLO)[0];
    game.posed(&[Event::CardDiscarded { player: SOLO, card }]);
    game.act(Action::UseLongNight);
    assert_eq!(game.state.round(), 4);
    assert_eq!(game.state.removed_tactics().len(), 3);
    let (replayed, _) = runner(game.log.clone());
    assert_eq!(replayed.state(), &game.state);
}
