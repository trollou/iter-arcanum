//! Scénarios de la spec 050 (offres et cartes gagnées).

mod arene;

use std::collections::BTreeSet;

use arcanum_core::{
    Achievement, Action, AdvancedAction, Artifact, AttackKind, BasicColor, CardId, CardPlay,
    CommonCard, CommonDeck, ContentError, ContentSources, DeedCard, Effect, Element, Event,
    GameConfig, GameContent, GameRngs, GameState, Hero, Offer, PlayerEvent, RecruitSite,
    RngPositions, RngUsage, Scenario, Spell, TileId, TilePosition, Unit, UnitRank, apply, decide,
    filter_event, final_score, legal_actions, new_game, view,
};
use arene::*;

fn sandbox(seed: u64) -> GameConfig {
    GameConfig {
        seed,
        scenario: Scenario::sandbox(),
        hero: Hero::Tovak,
        dummy_hero: Default::default(),
    }
}

fn reconnaissance(seed: u64) -> GameConfig {
    GameConfig {
        scenario: Scenario::first_reconnaissance().without_dummy(),
        ..sandbox(seed)
    }
}

fn setup(config: &GameConfig) -> Vec<Event> {
    new_game(config, &mut config.rngs())
}

fn started(config: &GameConfig) -> GameState {
    past_tactic_choice(setup(config).iter().fold(GameState::default(), apply))
}

fn is_offer_event(event: &Event) -> bool {
    matches!(
        event,
        Event::CommonDeckShuffled { .. } | Event::OfferCardDealt { .. }
    )
}

fn advanced(card: &CommonCard) -> Option<AdvancedAction> {
    match card {
        CommonCard::Deed(id) => match id.card() {
            DeedCard::Advanced(action) => Some(action),
            _ => None,
        },
        CommonCard::Unit(_) => None,
    }
}

fn unit_of(card: &CommonCard) -> Option<Unit> {
    match card {
        CommonCard::Unit(unit) => Some(unit.unit),
        CommonCard::Deed(_) => None,
    }
}

fn has_village_unit(offer: &[CommonCard]) -> bool {
    offer.iter().filter_map(unit_of).any(|unit| {
        GameContent::embedded()
            .unit(unit)
            .recruit
            .contains(&RecruitSite::Village)
    })
}

// --- Contenu ---

// C20 — spec 029 : tout le matériel du jeu de base en données.
#[test]
fn content_has_all_base_game_cards() {
    let content = GameContent::embedded();
    let numbers: BTreeSet<u8> = AdvancedAction::ALL
        .into_iter()
        .map(|card| content.advanced_action_number(card))
        .collect();
    assert_eq!(numbers, (1..=28).collect());
    // Sept actions avancées par couleur (cartes/actions-avancees.md).
    for color in BasicColor::ALL {
        let count = AdvancedAction::ALL
            .into_iter()
            .filter(|&card| DeedCard::Advanced(card).definition().map(|d| d.color) == Some(color))
            .count();
        assert_eq!(count, 7, "{color:?}");
    }
    assert_eq!(Spell::ALL.len(), 20);
    for spell in Spell::ALL {
        assert!(DeedCard::Spell(spell).definition().is_some(), "{spell:?}");
    }
    assert_eq!(Artifact::ALL.len(), 16);
    for rank in [UnitRank::Regular, UnitRank::Elite] {
        assert_eq!(content.unit_cards(rank).len(), 20, "{rank:?}");
    }
    assert_eq!(
        Unit::ALL
            .into_iter()
            .filter(|unit| content.unit(*unit).rank == UnitRank::Regular)
            .count(),
        11
    );
}

// C20 : un identifiant en double ou manquant est refusé.
#[test]
fn content_refuses_duplicate_or_missing_cards() {
    let embedded = ContentSources::EMBEDDED.advanced_actions;
    let duplicated = embedded.replacen("IceBolt: (number: 2", "FireBolt: (number: 2", 1);
    assert!(matches!(
        GameContent::parse(ContentSources {
            advanced_actions: &duplicated,
            ..ContentSources::EMBEDDED
        }),
        Err(ContentError::Duplicate { .. })
    ));
    // Spec 057 : l'entrée de Lecture d'esprit s'écrit sur plusieurs lignes, retirée jusqu'à sa
    // parenthèse fermante.
    let spells = ContentSources::EMBEDDED.spells;
    let start = spells.find("    MindRead: (").expect("Lecture d'esprit");
    let end = start + spells[start..].find("\n    ),\n").expect("fin de l'entrée") + 8;
    let missing = format!("{}{}", &spells[..start], &spells[end..]);
    assert!(matches!(
        GameContent::parse(ContentSources {
            spells: &missing,
            ..ContentSources::EMBEDDED
        }),
        Err(ContentError::Missing { .. })
    ));
    let renumbered = embedded.replacen("(number: 2,", "(number: 1,", 1);
    assert!(matches!(
        GameContent::parse(ContentSources {
            advanced_actions: &renumbered,
            ..ContentSources::EMBEDDED
        }),
        Err(ContentError::InvalidNumber { .. })
    ));
    let short_deck = ContentSources::EMBEDDED
        .units
        .replacen("count: 3", "count: 2", 1);
    assert!(matches!(
        GameContent::parse(ContentSources {
            units: &short_deck,
            ..ContentSources::EMBEDDED
        }),
        Err(ContentError::InvalidUnits { .. })
    ));
}

// --- Mise en place ---

// C1 — MKUE p. 3 : trois cartes par offre, le reste dans les pioches.
#[test]
fn setup_deals_three_cards_to_each_offer() {
    let state = started(&sandbox(7));
    assert_eq!(state.offer(Offer::AdvancedActions).len(), 3);
    assert_eq!(state.offer(Offer::Spells).len(), 3);
    let units = state.offer(Offer::Units);
    assert_eq!(units.len(), 3);
    assert!(
        units
            .iter()
            .filter_map(unit_of)
            .all(|unit| { GameContent::embedded().unit(unit).rank == UnitRank::Regular })
    );
    let counts = [
        (CommonDeck::AdvancedActions, 25),
        (CommonDeck::Spells, 17),
        (CommonDeck::Artifacts, 16),
        (CommonDeck::RegularUnits, 17),
        (CommonDeck::EliteUnits, 20),
    ];
    for (deck, count) in counts {
        assert_eq!(state.common_deck(deck).len(), count, "{deck:?}");
    }
}

// C1 — MKUE p. 4, 2.d : chaque carte tirée entre en première place.
#[test]
fn setup_puts_each_new_card_in_top_position() {
    let events = setup(&sandbox(7));
    let order = events
        .iter()
        .find_map(|event| match event {
            Event::CommonDeckShuffled {
                deck: CommonDeck::AdvancedActions,
                order,
            } => Some(order.clone()),
            _ => None,
        })
        .expect("la pioche d'actions avancées est mélangée");
    let state = events.iter().fold(GameState::default(), apply);
    assert_eq!(
        state.offer(Offer::AdvancedActions),
        [order[2], order[1], order[0]]
    );
}

// C2 — ADR 0007 : même graine, mêmes offres ; les autres tirages ne dépendent pas des pioches
// communes.
#[test]
fn offers_do_not_shift_other_draws() {
    let config = sandbox(11);
    assert_eq!(setup(&config), setup(&config));
    // Les pioches communes avancées loin : leurs offres changent, rien d'autre.
    let shifted: RngPositions = CommonDeck::ALL
        .into_iter()
        .map(|deck| (RngUsage::CommonDeck(deck), 4096))
        .collect();
    let mut rngs = GameRngs::resume(config.seed, &shifted);
    let other = new_game(&config, &mut rngs);
    let without_offers = |events: &[Event]| -> Vec<Event> {
        events
            .iter()
            .filter(|e| !is_offer_event(e))
            .cloned()
            .collect()
    };
    assert_eq!(without_offers(&setup(&config)), without_offers(&other));
    assert_ne!(setup(&config), other);
}

// C3 — MKUE p. 18 ; Compendium, *Miscellaneous* S8 : actions avancées 1 à 16, pas d'élites.
#[test]
fn first_reconnaissance_uses_advanced_actions_1_to_16() {
    let state = started(&reconnaissance(3));
    let content = GameContent::embedded();
    let cards: Vec<AdvancedAction> = state
        .common_deck(CommonDeck::AdvancedActions)
        .iter()
        .chain(state.offer(Offer::AdvancedActions))
        .filter_map(advanced)
        .collect();
    assert_eq!(cards.len(), 16);
    assert!(
        cards
            .iter()
            .all(|&card| content.advanced_action_number(card) <= 16)
    );
    assert!(state.common_deck(CommonDeck::EliteUnits).is_empty());
    assert!(!setup(&reconnaissance(3)).iter().any(|event| matches!(
        event,
        Event::CommonDeckShuffled {
            deck: CommonDeck::EliteUnits,
            ..
        }
    )));
}

// C4 — MKUE p. 18 : au moins une unité de village dans l'offre de la mise en place.
#[test]
fn first_reconnaissance_offers_a_village_unit() {
    let seed = (0..1000)
        .find(|&seed| !has_village_unit(started(&sandbox(seed)).offer(Offer::Units)))
        .expect("une graine sans unité de village dans l'offre du bac à sable");
    assert!(has_village_unit(
        started(&reconnaissance(seed)).offer(Offer::Units)
    ));
    assert!(!has_village_unit(
        started(&sandbox(seed)).offer(Offer::Units)
    ));
}

// --- Vue ---

// C21 : les offres et le nombre de cartes des pioches se voient, jamais leur ordre.
#[test]
fn player_view_shows_offers_not_deck_order() {
    let events = setup(&sandbox(5));
    let state = events.iter().fold(GameState::default(), apply);
    let seen = view(&state, SOLO);
    for offer in Offer::ALL {
        assert_eq!(seen.offers[&offer], state.offer(offer), "{offer:?}");
    }
    assert_eq!(seen.common_decks[&CommonDeck::Spells], 17);
    let shuffled = events
        .iter()
        .find(|event| matches!(event, Event::CommonDeckShuffled { .. }))
        .expect("une pioche commune est mélangée");
    assert!(matches!(
        filter_event(shuffled, SOLO),
        PlayerEvent::CommonDeckShuffled { count, .. } if count > 0
    ));
}

// --- Renouvellement ---

/// Deck et main vidés : le joueur doit annoncer la fin de la manche (MKUE p. 6, 3.b).
fn empty_deck(state: GameState) -> GameState {
    apply(
        state,
        &Event::DeckShuffled {
            player: SOLO,
            order: Vec::new(),
        },
    )
}

/// La nouvelle manche qui suit l'annonce : ses événements et l'état après.
fn new_round(state: &GameState, seed: u64) -> (Vec<Event>, GameState) {
    let events = decide(
        &empty_deck(state.clone()),
        SOLO,
        &Action::AnnounceEndOfRound,
        &mut sandbox(seed).rngs(),
    )
    .expect("annoncer la fin de la manche est légal");
    let after = events.iter().fold(empty_deck(state.clone()), apply);
    (events, after)
}

/// Une tuile royaume révélée à côté des deux tuiles de campagne.
fn with_core_tile(state: GameState) -> GameState {
    apply(
        state,
        &Event::TileRevealed {
            tile: TileId::Core(1),
            position: TilePosition::from_lattice(1, 1),
        },
    )
}

fn rank_of(card: &CommonCard) -> Option<UnitRank> {
    unit_of(card).map(|unit| GameContent::embedded().unit(unit).rank)
}

fn refreshes(offer: Offer, deck: CommonDeck) {
    let state = started(&sandbox(21));
    let before = state.offer(offer).to_vec();
    let top = state.common_deck(deck)[0];
    let (_, after) = new_round(&state, 21);
    assert_eq!(after.offer(offer), [top, before[0], before[1]]);
    assert_eq!(after.common_deck(deck).last(), Some(&before[2]));
    assert_eq!(after.common_deck(deck).len(), state.common_deck(deck).len());
}

// C5 — MKUE p. 4, 2.d : la dernière place sous la pioche, une carte en première place.
#[test]
fn new_round_refreshes_advanced_action_offer() {
    refreshes(Offer::AdvancedActions, CommonDeck::AdvancedActions);
}

// C6 — MKUE p. 4, 2.e
#[test]
fn new_round_refreshes_spell_offer() {
    refreshes(Offer::Spells, CommonDeck::Spells);
}

// C7 — MKUE p. 4, 2.c : sans tuile royaume, trois régulières, l'ancienne offre sous la pioche.
#[test]
fn new_round_deals_regular_units_before_core_tile() {
    let state = started(&sandbox(21));
    let before = state.offer(Offer::Units).to_vec();
    let (events, after) = new_round(&state, 21);
    let returned: Vec<CommonCard> = events
        .iter()
        .filter_map(|event| match event {
            Event::OfferCardReturned {
                offer: Offer::Units,
                card,
            } => Some(*card),
            _ => None,
        })
        .collect();
    assert_eq!(returned, before);
    let regular = after.common_deck(CommonDeck::RegularUnits);
    assert_eq!(regular[regular.len() - 3..], before[..]);
    assert_eq!(
        after
            .offer(Offer::Units)
            .iter()
            .filter_map(rank_of)
            .collect::<Vec<_>>(),
        [UnitRank::Regular; 3]
    );
    // Les offres sont renouvelées après la Source, avant le deck (MKUE p. 4, 2.b à 2.g).
    let position = |wanted: fn(&Event) -> bool| events.iter().position(wanted);
    assert!(
        position(|e| matches!(e, Event::SourceRolled { .. }))
            < position(|e| matches!(e, Event::OfferCardReturned { .. }))
    );
    assert!(
        position(|e| matches!(e, Event::OfferCardDealt { .. }))
            < position(|e| matches!(e, Event::DeckShuffled { .. }))
    );
}

// C8 — MKUE p. 4, 2.c ; Compendium, *Silver Units* S1 : élite, régulière, élite.
#[test]
fn new_round_alternates_elite_units_after_core_tile() {
    let state = with_core_tile(started(&sandbox(21)));
    let (_, first) = new_round(&state, 21);
    let units = first.offer(Offer::Units).to_vec();
    assert_eq!(
        units.iter().filter_map(rank_of).collect::<Vec<_>>(),
        [UnitRank::Elite, UnitRank::Regular, UnitRank::Elite]
    );
    let (_, second) = new_round(&first, 21);
    assert_eq!(
        second.common_deck(CommonDeck::EliteUnits).last(),
        Some(&units[2])
    );
}

// C9 — MKUE p. 18 : jamais d'élite en Première reconnaissance.
#[test]
fn first_reconnaissance_never_deals_elite_units() {
    let state = with_core_tile(started(&reconnaissance(21)));
    let (_, after) = new_round(&state, 21);
    assert_eq!(
        after
            .offer(Offer::Units)
            .iter()
            .filter_map(rank_of)
            .collect::<Vec<_>>(),
        [UnitRank::Regular; 3]
    );
}

// L2 : pioche d'élite vide, une régulière à la place.
#[test]
fn empty_elite_deck_deals_a_regular_unit() {
    let state = with_core_tile(apply(
        started(&sandbox(21)),
        &Event::CommonDeckShuffled {
            deck: CommonDeck::EliteUnits,
            order: Vec::new(),
        },
    ));
    let (_, after) = new_round(&state, 21);
    assert_eq!(
        after
            .offer(Offer::Units)
            .iter()
            .filter_map(rank_of)
            .collect::<Vec<_>>(),
        [UnitRank::Regular; 3]
    );
}

// --- Action avancée d'un niveau pair ---

fn fame(state: GameState, fame: u32) -> GameState {
    apply(
        state,
        &Event::EnemiesDefeated {
            player: SOLO,
            enemies: Vec::new(),
            fame,
        },
    )
}

/// Deux cartes de la main jouées de côté : il en manquera deux à la fin du tour.
fn two_cards_played(state: GameState) -> GameState {
    let hand = state.hand(SOLO).to_vec();
    hand[..2]
        .iter()
        .fold(state, |state, &card| done(&state, sideways(card)))
}

fn end_turn_action(state: &GameState) -> Action {
    legal_actions(state, SOLO)
        .into_iter()
        .find(|action| matches!(action, Action::EndTurn { discard, .. } if discard.is_empty()))
        .expect("finir le tour sans défausse est légal")
}

/// Une carte jouée de côté (un tour sans rien jouer ne se finit que par un repos), puis la fin
/// du tour sans défausse ; à un niveau pair, la première compétence révélée gardée (spec 051).
fn end_turn(state: &GameState) -> (Vec<Event>, GameState) {
    let card = state.hand(SOLO)[0];
    let state = &done(state, sideways(card));
    let mut events = events_of(state, &end_turn_action(state));
    let after = events.iter().fold(state.clone(), apply);
    let (skill, after) = first_skill(&after);
    events.extend(skill);
    (events, after)
}

/// Spec 051 : la compétence d'un niveau pair se choisit avant l'action avancée ; la première
/// révélée, s'il y a un choix en attente.
fn first_skill(state: &GameState) -> (Vec<Event>, GameState) {
    match legal_actions(state, SOLO)
        .into_iter()
        .find(|action| matches!(action, Action::ChooseSkill { .. }))
    {
        Some(action) => {
            let events = events_of(state, &action);
            let after = events.iter().fold(state.clone(), apply);
            (events, after)
        }
        None => (Vec::new(), state.clone()),
    }
}

fn gain(card: CommonCard) -> Action {
    Action::GainAdvancedAction {
        card: card.deed().expect("une action avancée est une carte Acte"),
    }
}

// C10 — MKUE p. 11, 6.b : au niveau 2, la seule décision est une action avancée de l'offre.
#[test]
fn even_level_asks_for_an_advanced_action() {
    let state = fame(started(&sandbox(4)), 3);
    let (events, after) = end_turn(&state);
    assert!(events.contains(&Event::LevelGained {
        player: SOLO,
        level: 2
    }));
    assert!(!events.iter().any(|event| matches!(
        event,
        Event::CardDrawn { .. } | Event::TurnStarted { .. } | Event::GameEnded { .. }
    )));
    assert_eq!(after.advanced_action_choices(SOLO), 1);
    let offer = after.offer(Offer::AdvancedActions).to_vec();
    assert_eq!(
        legal_actions(&after, SOLO),
        offer.iter().map(|&card| gain(card)).collect::<Vec<_>>()
    );
}

// C11 — MKUE p. 11, 6.b et 7 ; p. 5, 3.a : sur le deck, l'offre complétée, puis piochée.
#[test]
fn gained_advanced_action_goes_on_deck_and_is_drawn() {
    let state = two_cards_played(fame(started(&sandbox(4)), 3));
    let (_, choosing) = end_turn(&state);
    let offer = choosing.offer(Offer::AdvancedActions).to_vec();
    let top = choosing.common_deck(CommonDeck::AdvancedActions)[0];
    let events = events_of(&choosing, &gain(offer[1]));
    let after = events.iter().fold(choosing.clone(), apply);
    assert_eq!(
        after.offer(Offer::AdvancedActions),
        [top, offer[0], offer[2]]
    );
    let gained = offer[1].deed().expect("carte Acte");
    assert_eq!(
        events[..2],
        [
            Event::OfferCardGained {
                to: arcanum_core::CardDestination::DeckTop,
                player: SOLO,
                offer: Offer::AdvancedActions,
                card: offer[1],
            },
            Event::OfferCardDealt {
                offer: Offer::AdvancedActions,
                card: top,
            },
        ]
    );
    assert_eq!(
        events[2],
        Event::CardDrawn {
            player: SOLO,
            card: gained,
        }
    );
    assert!(after.hand(SOLO).contains(&gained));
    assert_eq!(after.hand(SOLO).len(), 5);
    assert_eq!(after.advanced_action_choices(SOLO), 0);
    assert!(matches!(
        events.last(),
        Some(Event::TurnStarted { turn: 2, .. })
    ));
}

// C12 — MKUE p. 11, 6.a : un niveau impair ne donne rien.
#[test]
fn odd_level_gives_no_advanced_action() {
    let state = apply(
        fame(started(&sandbox(4)), 8),
        &Event::LevelGained {
            player: SOLO,
            level: 2,
        },
    );
    let state = apply(
        state,
        &Event::TurnStarted {
            player: SOLO,
            turn: 1,
        },
    );
    let (events, after) = end_turn(&state);
    assert!(events.contains(&Event::LevelGained {
        player: SOLO,
        level: 3
    }));
    assert!(matches!(events.last(), Some(Event::TurnStarted { .. })));
    assert_eq!(after.advanced_action_choices(SOLO), 0);
}

// C13 — spec 035, L4 : deux niveaux pairs d'un coup, deux choix.
#[test]
fn two_even_levels_give_two_advanced_actions() {
    let state = fame(started(&sandbox(4)), 15);
    let (_, choosing) = end_turn(&state);
    assert_eq!(choosing.advanced_action_choices(SOLO), 2);
    let first = choosing.offer(Offer::AdvancedActions)[0];
    let events = events_of(&choosing, &gain(first));
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::TurnStarted { .. }))
    );
    let middle = events.iter().fold(choosing, apply);
    // Spec 051 : la compétence du niveau 4, avant sa carte.
    let (_, middle) = first_skill(&middle);
    assert_eq!(middle.offer(Offer::AdvancedActions).len(), 3);
    let second = middle.offer(Offer::AdvancedActions)[2];
    let after = done(&middle, gain(second));
    // Une carte à piocher : la seconde, au-dessus ; la première reste sur le deck (L5).
    assert_eq!(after.hand(SOLO).last(), second.deed().as_ref());
    assert_eq!(after.deck(SOLO).first(), first.deed().as_ref());
}

// C14 — MKUE p. 15 ; spec 063 : au dernier tour, le choix puis la fin de partie.
#[test]
fn last_turn_advanced_action_counts_at_scoring() {
    let mut state = fame(started(&reconnaissance(4)), 3);
    for event in [
        Event::LastTurnAnnounced { player: SOLO },
        Event::TurnStarted {
            player: SOLO,
            turn: 2,
        },
    ] {
        state = apply(state, &event);
    }
    let (events, choosing) = end_turn(&state);
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::GameEnded { .. }))
    );
    let card = choosing.offer(Offer::AdvancedActions)[0];
    let events = events_of(&choosing, &gain(card));
    assert!(matches!(events.last(), Some(Event::GameEnded { .. })));
    let after = events.iter().fold(choosing, apply);
    let score = final_score(&after, SOLO);
    assert_eq!(score.points(Achievement::Knowledge), 1);
}

// C15 — MKUE p. 15 : Le Savant, 1 par action avancée, 2 par sort.
#[test]
fn knowledge_counts_advanced_actions_and_spells() {
    let advanced = |card| CardId::new(DeedCard::Advanced(card), 1);
    let state = state_with_deck(
        vec![
            advanced(AdvancedAction::FireBolt),
            CardId::new(DeedCard::Spell(Spell::Fireball), 1),
            advanced(AdvancedAction::Agility),
        ],
        1,
    );
    let state = apply(
        state,
        &Event::CardDiscarded {
            player: SOLO,
            card: advanced(AdvancedAction::FireBolt),
        },
    );
    let score = final_score(&state, SOLO);
    assert_eq!((score.advanced_actions, score.spells), (2, 1));
    assert_eq!(score.points(Achievement::Knowledge), 4);
}

// C16 — spec 024 : ni avant la fin du tour, ni avant le choix une fois l'offre complétée.
#[test]
fn advanced_action_choice_cannot_be_undone() {
    let state = fame(started(&sandbox(4)), 3);
    let (events, choosing) = end_turn(&state);
    assert!(events.iter().any(Event::blocks_undo));
    let card = choosing.offer(Offer::AdvancedActions)[0];
    let events = events_of(&choosing, &gain(card));
    assert!(matches!(events[1], Event::OfferCardDealt { .. }));
    assert!(events[1].blocks_undo());
}

// L1 : offre vide au niveau pair, pas de choix ; pioche vide, l'offre garde moins de cartes.
#[test]
fn empty_offer_gives_no_choice() {
    let mut state = fame(started(&sandbox(4)), 3);
    for card in state.offer(Offer::AdvancedActions).to_vec() {
        state = apply(
            state,
            &Event::OfferCardReturned {
                offer: Offer::AdvancedActions,
                card,
            },
        );
    }
    let (events, after) = end_turn(&state);
    assert!(matches!(events.last(), Some(Event::TurnStarted { .. })));
    assert_eq!(after.advanced_action_choices(SOLO), 0);
}

#[test]
fn empty_deck_leaves_a_shorter_offer() {
    let state = apply(
        fame(started(&sandbox(4)), 3),
        &Event::CommonDeckShuffled {
            deck: CommonDeck::AdvancedActions,
            order: Vec::new(),
        },
    );
    let (_, choosing) = end_turn(&state);
    let card = choosing.offer(Offer::AdvancedActions)[0];
    let events = events_of(&choosing, &gain(card));
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::OfferCardDealt { .. }))
    );
    let after = events.iter().fold(choosing, apply);
    assert_eq!(after.offer(Offer::AdvancedActions).len(), 2);
}

// L3 et L4 : hors choix, ou une carte absente de l'offre, refusée ; pendant le choix, rien d'autre.
#[test]
fn advanced_action_outside_the_choice_is_refused() {
    let state = fame(started(&sandbox(4)), 3);
    let card = state.offer(Offer::AdvancedActions)[0];
    refused(&state, gain(card));
    let (_, choosing) = end_turn(&state);
    let absent = choosing.common_deck(CommonDeck::AdvancedActions)[0];
    refused(&choosing, gain(absent));
    refused(&choosing, Action::AnnounceEndOfRound);
}

// L7 — Compendium, *End of Round* S1 : annoncer la fin de la manche ne change pas de niveau.
#[test]
fn announcing_end_of_round_gives_no_advanced_action() {
    let state = fame(started(&sandbox(4)), 3);
    let (events, after) = new_round(&state, 4);
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::LevelGained { .. }))
    );
    assert_eq!(after.advanced_action_choices(SOLO), 0);
}

// --- Jouer une carte gagnée ---

fn advanced_card(card: AdvancedAction) -> CardId {
    CardId::new(DeedCard::Advanced(card), 1)
}

const RED_DIE: u8 = 1;

// C17 — cartes/actions-avancees.md : Fire Bolt, cristal rouge ; renforcée, attaque à distance
// de feu 3 en phase d'attaque à distance.
#[test]
fn fire_bolt_plays_its_effects() {
    let fire_bolt = advanced_card(AdvancedAction::FireBolt);
    let state = arena(PLAIN_W, &[fire_bolt], &[]);
    let events = events_of(&state, &option(fire_bolt, 0));
    assert!(events.contains(&Event::CrystalGained {
        player: SOLO,
        color: BasicColor::Red,
    }));

    let state = fight(PLAIN_W, &[fire_bolt], &[(prowlers(), ORCS_2)], &[ORCS_2]);
    let state = done(&state, declare(&[foe(&state, ORCS_2)]));
    let events = events_of(&state, &strong(fire_bolt, RED_DIE));
    assert!(events.iter().any(|event| matches!(
        event,
        Event::CardPlayed {
            effect: Effect::Attack {
                amount: 3,
                element: Element::Fire,
                kind: AttackKind::Ranged,
            },
            ..
        }
    )));
}

// C18 — MKUE p. 4, 3.e : Bouclier de glace, Parade de glace 3 ; renforcé, codé par la spec 053.
#[test]
fn ice_shield_plays_in_a_block() {
    let ice_shield = advanced_card(AdvancedAction::IceShield);
    let state = fight(PLAIN_W, &[ice_shield], &[(prowlers(), ORCS_2)], &[ORCS_2]);
    let state = done(&to_block_phase(&state), declare_block(foe(&state, ORCS_2)));
    let plays: Vec<CardPlay> = legal_actions(&state, SOLO)
        .into_iter()
        .filter_map(|action| match action {
            Action::PlayCard { card, play } if card == ice_shield => Some(play),
            _ => None,
        })
        .collect();
    assert!(
        plays
            .iter()
            .any(|play| matches!(play, CardPlay::Basic { .. }))
    );
    assert!(
        plays
            .iter()
            .any(|play| matches!(play, CardPlay::Sideways(_)))
    );
    assert!(
        plays
            .iter()
            .any(|play| matches!(play, CardPlay::Strong { .. }))
    );
    assert_eq!(
        ice_shield.effect(CardPlay::Basic {
            die: None,
            gain: None,
            with: None,
            option: 0,
            discard: None,
            mana: None,
        }),
        Some(Effect::Block {
            amount: 3,
            element: Element::Ice,
        })
    );
}

// C19 — MKUE p. 4, 3.e : une carte entièrement non codée ne se joue que de côté.
#[test]
fn unsupported_card_plays_sideways_only() {
    // Spec 056 : toutes les actions avancées sont codées ; un sort reste non codé (spec 054).
    let agility = CardId::new(DeedCard::Spell(arcanum_core::Spell::Fireball), 1);
    let state = arena(PLAIN_W, &[agility], &[]);
    let plays: Vec<CardPlay> = legal_actions(&state, SOLO)
        .into_iter()
        .filter_map(|action| match action {
            Action::PlayCard { card, play } if card == agility => Some(play),
            _ => None,
        })
        .collect();
    assert!(!plays.is_empty());
    assert!(
        plays
            .iter()
            .all(|play| matches!(play, CardPlay::Sideways(_)))
    );
}
