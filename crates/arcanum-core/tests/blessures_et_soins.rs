//! Scénarios de la spec 034 (Blessures et soins).

use arcanum_core::{
    Action, BasicAction, BasicColor, BoostTarget, CardId, CardPlay, CombatPhase, CombatRecord,
    Effect, Enemy, EnemyToken, Event, GameConfig, GameState, HealingTarget, Hero, Hex, ManaPayment,
    TilePosition, apply, legal_actions,
};

mod arene;
use arene::*;

use BasicAction::*;

// --- Outils ---

fn end_turn(discard: &[CardId]) -> Action {
    Action::EndTurn {
        discard: discard.to_vec(),
        reclaim: None,
        glade: None,
        steady_tempo: false,
    }
}

fn with_crystal(state: GameState, color: BasicColor) -> GameState {
    apply(
        state,
        &Event::CrystalGained {
            player: SOLO,
            color,
        },
    )
}

/// L'option `option` de l'effet renforcé de `card`, payée par un cristal.
fn strong_by_crystal(card: CardId, option: u8) -> Action {
    Action::PlayCard {
        card,
        play: CardPlay::Strong {
            gain: None,
            extra_mana: None,
            option,
            discard: None,
            mana: ManaPayment::Crystal,
            set_dice: [None, None],
            with: None,
        },
    }
}

fn tranquility() -> CardId {
    card(Tranquility, 1)
}

/// Partie de bac à sable avec `hero` : la main `hand`, le deck de départ du héros derrière.
fn hero_state(hero: Hero, hand: &[CardId]) -> GameState {
    let mut order = hand.to_vec();
    order.extend(
        hero.starting_deck()
            .into_iter()
            .filter(|card| !hand.contains(card)),
    );
    let events = [
        Event::GameStarted {
            config: GameConfig { hero, ..config(0) },
        },
        Event::TilePlaced {
            tile: arcanum_core::TileId::StartingA,
            position: TilePosition::ORIGIN,
        },
        Event::HeroFigurePlaced {
            player: SOLO,
            hex: Hex::ORIGIN,
        },
        Event::DeckShuffled {
            player: SOLO,
            order: order.clone(),
        },
    ];
    let state = events.iter().fold(GameState::default(), apply);
    hand.iter().fold(state, |state, &card| {
        apply(state, &Event::CardDrawn { player: SOLO, card })
    })
}

/// Spec 042 (Q1) : le soin joué va dans la réserve ; il est ensuite dépensé sur les Blessures de la
/// main tant que c'est possible. Les événements de la carte, puis des dépenses.
fn heal_all(state: &GameState, action: &Action) -> Vec<Event> {
    let mut events = events_of(state, action);
    let mut current = events.iter().fold(state.clone(), apply);
    let spend = Action::SpendHealing {
        on: HealingTarget::Wound,
    };
    while legal_actions(&current, SOLO).contains(&spend) {
        let more = events_of(&current, &spend);
        current = more.iter().fold(current, apply);
        events.extend(more);
    }
    events
}

fn healed(events: &[Event]) -> Vec<CardId> {
    events
        .iter()
        .filter_map(|event| match event {
            Event::WoundsHealed { wounds, .. } => Some(wounds.clone()),
            _ => None,
        })
        .flatten()
        .collect()
}

fn drawn(events: &[Event]) -> Vec<CardId> {
    events
        .iter()
        .filter_map(|event| match event {
            Event::CardDrawn { card, .. } => Some(*card),
            _ => None,
        })
        .collect()
}

fn wounds_in(cards: &[CardId]) -> usize {
    cards.iter().filter(|card| card.card().is_wound()).count()
}

/// Les effets avec lesquels `card` peut être jouée, hors de côté.
fn effects_of(state: &GameState, wanted: CardId) -> Vec<Effect> {
    legal_actions(state, SOLO)
        .iter()
        .filter_map(|action| match action {
            Action::PlayCard { card, play } if *card == wanted => match play {
                CardPlay::Sideways(_) => None,
                _ => card.effect(*play),
            },
            _ => None,
        })
        .collect()
}

fn end_turns(state: &GameState) -> Vec<Vec<CardId>> {
    legal_actions(state, SOLO)
        .into_iter()
        .filter_map(|action| match action {
            Action::EndTurn {
                discard,
                reclaim: None,
                ..
            } => Some(discard),
            _ => None,
        })
        .collect()
}

fn accepted(state: &GameState, action: Action) -> GameState {
    done(state, action)
}

// --- Soins ---

// Spec 034 C1 — MKUE p. 11, Wounds and Healing 3.a et 3.c.
#[test]
fn heal_throws_away_a_wound_from_hand() {
    let march = card(March, 1);
    let state = state_with(&[wound(1), wound(2), tranquility(), march]);
    let deck = state.deck(SOLO).to_vec();
    let events = heal_all(&state, &option(tranquility(), 0));
    assert_eq!(healed(&events), [wound(1)]);
    let state = events.iter().fold(state, apply);
    assert_eq!(state.hand(SOLO), [wound(2), march]);
    assert!(state.discard_pile(SOLO).is_empty());
    assert_eq!(state.deck(SOLO), deck);
}

// Spec 034 C2 — MKUE p. 11, 3.c.
#[test]
fn strong_heal_throws_away_two_wounds() {
    let march = card(March, 1);
    let state = state_with(&[wound(1), wound(2), wound(3), tranquility(), march]);
    let state = with_crystal(state, BasicColor::Green);
    let events = heal_all(&state, &strong_by_crystal(tranquility(), 0));
    assert_eq!(healed(&events), [wound(1), wound(2)]);
    let state = events.iter().fold(state, apply);
    assert_eq!(state.hand(SOLO), [wound(3), march]);
}

// Spec 034 C3 — Compendium, Tranquility S1.
#[test]
fn heal_beyond_wounds_in_hand_is_lost() {
    let state = with_crystal(state_with(&[wound(1), tranquility()]), BasicColor::Green);
    let events = heal_all(&state, &strong_by_crystal(tranquility(), 0));
    assert_eq!(healed(&events), [wound(1)]);
    assert!(events.iter().fold(state, apply).hand(SOLO).is_empty());

    let state = state_with(&[tranquility(), card(March, 1)]);
    let events = heal_all(&state, &option(tranquility(), 0));
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::WoundsHealed { .. }))
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::CardPlayed { .. }))
    );
}

// Spec 034 C4 — Compendium, Healing S2.
#[test]
fn heal_ignores_wounds_in_discard_pile() {
    let state = apply(
        state_with(&[tranquility(), card(March, 1)]),
        &Event::PoisonWoundsDiscarded {
            player: SOLO,
            enemy: prowlers(),
            wounds: vec![wound(9)],
        },
    );
    let events = heal_all(&state, &option(tranquility(), 0));
    assert!(healed(&events).is_empty());
    let state = events.iter().fold(state, apply);
    assert_eq!(state.discard_pile(SOLO), [wound(9)]);
}

// Spec 034 C5 — texte de la carte ; spec 024.
#[test]
fn draw_option_draws_top_card() {
    let state = state_with(&[tranquility(), card(March, 1)]);
    let top = state.deck(SOLO)[0];
    let events = events_of(&state, &option(tranquility(), 1));
    assert_eq!(drawn(&events), [top]);
    assert!(events.iter().any(Event::blocks_undo));
    let state = events.iter().fold(state, apply);
    assert_eq!(state.hand(SOLO), [card(March, 1), top]);
}

// Spec 034 C6 — Compendium, Basic Action Cards O5.
#[test]
fn draw_stops_when_deck_is_empty() {
    let march = card(March, 1);
    let rage = card(Rage, 1);
    let state = with_crystal(
        state_with_deck(vec![tranquility(), march, rage], 2),
        BasicColor::Green,
    );
    let events = events_of(&state, &strong_by_crystal(tranquility(), 1));
    assert_eq!(drawn(&events), [rage]);

    let state = state_with_deck(vec![tranquility(), march], 2);
    let events = events_of(&state, &option(tranquility(), 1));
    assert!(drawn(&events).is_empty());
    let state = events.iter().fold(state, apply);
    assert_eq!(state.play_area(SOLO).len(), 1);
}

/// Combat engagé par Arythea contre les Rôdeurs des orques ②, depuis le village.
fn fight_prowlers(hand: &[CardId]) -> GameState {
    fight(VILLAGE, hand, &[(prowlers(), ORCS_2)], &[ORCS_2])
}

// Spec 034 C7 — MKUE p. 8, Combat with Enemies 6.
#[test]
fn healing_card_cannot_be_played_in_combat() {
    let rage = card(Rage, 1);
    let state = fight_prowlers(&[tranquility(), rage]);
    assert!(effects_of(&state, tranquility()).is_empty());
    let attacking = done(&state, declare(&[foe(&state, ORCS_2)]));
    assert!(effects_of(&attacking, tranquility()).is_empty());
    let blocking = done(&to_block_phase(&state), declare_block(foe(&state, ORCS_2)));
    assert!(effects_of(&blocking, tranquility()).is_empty());
    assert!(legal_actions(&blocking, SOLO).contains(&sideways_block(tranquility())));
    let attack_phase = to_attack_phase(&state);
    let attacking = done(&attack_phase, declare(&[foe(&state, ORCS_2)]));
    assert!(effects_of(&attacking, tranquility()).is_empty());
    assert!(legal_actions(&attacking, SOLO).contains(&sideways_attack(tranquility())));
}

// Spec 034 C8 — Compendium, Rejuvenate Q1 (Q3).
#[test]
fn rejuvenate_is_a_healing_card() {
    let rejuvenate = card(Rejuvenate, 1);
    let state = hero_state(Hero::Norowas, &[rejuvenate, card(March, 1)]);
    let effects = effects_of(&state, rejuvenate);
    assert_eq!(
        effects,
        [
            Effect::Heal(1),
            Effect::DrawCards(1),
            Effect::GainToken(BasicColor::Green)
        ]
    );

    let mut events = arena_events(
        VILLAGE,
        &[rejuvenate, card(Rage, 1)],
        &[(prowlers(), ORCS_2)],
        &[],
    );
    events[0] = Event::GameStarted {
        config: GameConfig {
            hero: Hero::Norowas,
            ..arythea_config()
        },
    };
    let state = events.iter().fold(GameState::default(), apply);
    let state = done(&state, challenge(ORCS_2));
    let state = done(&state, Action::StartCombat);
    assert!(effects_of(&state, rejuvenate).is_empty());
}

// Spec 034 C9 — MKUE p. 11, 3.d.
#[test]
fn heal_after_combat_same_turn() {
    let state = to_attack_phase(&fight_prowlers(&[tranquility(), card(Rage, 1)]));
    assert_eq!(wounds_in(state.hand(SOLO)), 2);
    let state = done(&state, Action::EndCombat);
    let events = heal_all(&state, &option(tranquility(), 0));
    assert_eq!(healed(&events).len(), 1);
    let state = events.iter().fold(state, apply);
    assert_eq!(wounds_in(state.hand(SOLO)), 1);
}

/// Concentration renforcée payée par un cristal vert, avec `target` option `option`.
fn boosted(concentration: CardId, target: CardId, option: u8) -> Action {
    Action::PlayCard {
        card: concentration,
        play: CardPlay::Strong {
            gain: None,
            extra_mana: None,
            option: 0,
            discard: None,
            mana: ManaPayment::Crystal,
            set_dice: [None, None],
            with: Some(BoostTarget {
                card: target,
                option,
                discard: None,
                set_dice: [None, None],
            }),
        },
    }
}

// Spec 034 C10 — MKUE p. 5, Composite 1.b.
#[test]
fn concentration_boosts_tranquility_without_bonus() {
    let concentration = card(Concentration, 1);
    let state = state_with(&[wound(1), wound(2), wound(3), concentration, tranquility()]);
    let state = with_crystal(state, BasicColor::Green);
    let events = heal_all(&state, &boosted(concentration, tranquility(), 0));
    assert!(events.contains(&Event::CardPlayed {
        player: SOLO,
        card: tranquility(),
        play: CardPlay::Boosted {
            option: 0,
            discard: None,
            set_dice: [None, None],
        },
        effect: Effect::Heal(2),
    }));
    assert_eq!(healed(&events), [wound(1), wound(2)]);
    assert!(legal_actions(&state, SOLO).contains(&boosted(concentration, tranquility(), 1)));

    let state = with_crystal(
        fight_prowlers(&[concentration, tranquility()]),
        BasicColor::Green,
    );
    let state = done(&to_block_phase(&state), declare_block(foe(&state, ORCS_2)));
    assert!(!legal_actions(&state, SOLO).iter().any(|action| matches!(
        action,
        Action::PlayCard {
            play: CardPlay::Strong {
                with: Some(BoostTarget { card, .. }),
                ..
            },
            ..
        } if *card == tranquility()
    )));
}

// --- Repos ---

// Spec 034 C11 — MKUE p. 6, 7.a ; p. 11, 1.b.
#[test]
fn standard_rest_discards_wounds() {
    let (march, rage) = (card(March, 1), card(Rage, 1));
    let state = state_with(&[wound(1), wound(2), march, rage]);
    let state = accepted(&state, end_turn(&[march, wound(1), wound(2)]));
    assert_eq!(state.discard_pile(SOLO), [march, wound(1), wound(2)]);
    assert_eq!(state.hand(SOLO).len(), 5);
    assert_eq!(state.hand(SOLO)[0], rage);
}

// Spec 034 C12 — MKUE p. 6, 7.a.
#[test]
fn standard_rest_needs_a_non_wound_card() {
    let (march, rage) = (card(March, 1), card(Rage, 1));
    let state = state_with(&[wound(1), wound(2), march, rage]);
    refused(&state, end_turn(&[wound(1), wound(2)]));
    accepted(&state, end_turn(&[march, rage, wound(1)]));
}

// Spec 034 C13 — MKUE p. 6, 7.
#[test]
fn no_rest_after_moving_or_fighting() {
    let (march, rage) = (card(March, 1), card(Rage, 1));
    let state = state_with(&[wound(1), march, rage]);
    let state = done(&state, option(march, 0));
    // Moved: la plaine voisine du portail coûte 2.
    let moved = done(&state, Action::MoveHero { to: Hex::new(1, 0) });
    refused(&moved, end_turn(&[rage, wound(1)]));
    accepted(&moved, end_turn(&[rage]));

    let explored = apply(
        state,
        &Event::Explored {
            player: SOLO,
            position: TilePosition::from_lattice(-1, 0),
            cost: 2,
        },
    );
    refused(&explored, end_turn(&[rage, wound(1)]));

    let swiftness = card(Swiftness, 1);
    let challenged = done(
        &arena(
            VILLAGE,
            &[wound(101), swiftness, rage],
            &[(prowlers(), ORCS_2)],
        ),
        challenge(ORCS_2),
    );
    let fought = done(&challenged, Action::StartCombat);
    let fought = done(&to_attack_phase(&fought), Action::EndCombat);
    refused(&fought, end_turn(&[swiftness, wound(101)]));
    accepted(&fought, end_turn(&[swiftness]));
}

// Spec 034 C14 — Compendium, Resting S3.
#[test]
fn rest_allowed_after_playing_move() {
    let (march, rage) = (card(March, 1), card(Rage, 1));
    let state = state_with(&[wound(1), march, rage]);
    let state = done(&state, option(march, 0));
    accepted(&state, end_turn(&[rage, wound(1)]));
}

// Spec 034 C15 — FAQ p3, Repos.
#[test]
fn heal_then_standard_rest() {
    let rage = card(Rage, 1);
    let state = state_with(&[wound(1), wound(2), wound(3), tranquility(), rage]);
    let state = done(&state, option(tranquility(), 0));
    let state = done(
        &state,
        Action::SpendHealing {
            on: HealingTarget::Wound,
        },
    );
    let events = events_of(&state, &end_turn(&[rage, wound(2), wound(3)]));
    assert_eq!(drawn(&events).len(), 5);
    let state = events.iter().fold(state, apply);
    let mut discard = state.discard_pile(SOLO).to_vec();
    discard.sort();
    let mut expected = vec![rage, tranquility(), wound(2), wound(3)];
    expected.sort();
    assert_eq!(discard, expected);
}

// Spec 034 C16 — MKUE p. 6, 7.a (031, Q12).
#[test]
fn slow_recovery_discards_one_wound() {
    let state = state_with(&[wound(1), wound(2), wound(3)]);
    assert_eq!(end_turns(&state), [vec![wound(1)]]);
    refused(&state, end_turn(&[wound(1), wound(2)]));
    // Les Blessures sont identiques : en nommer une autre reste valide.
    accepted(&state, end_turn(&[wound(3)]));
}

// Spec 034 C17 — Compendium, Resting Q1, option 3.
#[test]
fn slow_recovery_after_playing_a_card() {
    let state = state_with(&[wound(1), wound(2), tranquility()]);
    let action = option(tranquility(), 0);
    let state = heal_all(&state, &action).iter().fold(state, apply);
    assert_eq!(state.hand(SOLO), [wound(2)]);
    accepted(&state, end_turn(&[wound(2)]));
    accepted(&state, end_turn(&[]));
}

// Spec 034 C18 — Compendium, Resting S4.
#[test]
fn wounds_only_hand_cannot_challenge() {
    let state = arena(VILLAGE, &[wound(101), wound(102)], &[(prowlers(), ORCS_2)]);
    assert!(!legal_actions(&state, SOLO).contains(&challenge(ORCS_2)));
    refused(&state, challenge(ORCS_2));
}

/// L'arène avec une main vide et un deck vide.
fn empty_hand_and_deck_arena(placed: &[(EnemyToken, Hex)]) -> GameState {
    arena_events(VILLAGE, &[], placed, &[])
        .into_iter()
        .map(|event| match event {
            Event::DeckShuffled { player, .. } => Event::DeckShuffled {
                player,
                order: Vec::new(),
            },
            event => event,
        })
        .fold(GameState::default(), |state, event| apply(state, &event))
}

// Spec 034 C18b (issue privée 86) — MKUE p. 6, 5.a : main vide au début du tour, deck non vide, rien à
// jouer ni à défausser.
#[test]
fn empty_hand_ends_turn_without_discard() {
    let state = arena(VILLAGE, &[], &[(prowlers(), ORCS_2)]);
    assert!(legal_actions(&state, SOLO).contains(&end_turn(&[])));
    accepted(&state, end_turn(&[]));
}

// Spec 034 C18c (issue privée 86) — MKUE p. 6, 3.b : main et deck vides, le héros doit annoncer la fin
// de la manche ; il ne peut pas défier.
#[test]
fn empty_hand_and_deck_cannot_challenge() {
    let state = empty_hand_and_deck_arena(&[(prowlers(), ORCS_2)]);
    assert_eq!(
        legal_actions(&state, SOLO),
        vec![Action::AnnounceEndOfRound]
    );
    refused(&state, challenge(ORCS_2));
}

// Spec 034 C19 — MKUE p. 6, 3 (spec 023).
#[test]
fn wounds_only_hand_may_announce_end_of_round() {
    let state = state_with_deck(vec![wound(1), wound(2), wound(3)], 3);
    let legal = legal_actions(&state, SOLO);
    assert!(legal.contains(&Action::AnnounceEndOfRound));
    assert!(legal.contains(&end_turn(&[wound(1)])));
}

// Spec 034 C20.
#[test]
fn rest_actions_offer_one_combination_per_wound_count() {
    let (march, rage) = (card(March, 1), card(Rage, 1));
    let state = state_with(&[wound(1), wound(2), wound(3), march, rage]);
    let ends = end_turns(&state);
    let first_wounds = [wound(1), wound(2), wound(3)];
    for non_wounds in [vec![march], vec![rage], vec![march, rage]] {
        for count in 0..=3 {
            let mut expected: Vec<CardId> = first_wounds[..count].to_vec();
            expected.extend(&non_wounds);
            assert!(ends.contains(&expected), "{expected:?} absent de {ends:?}");
        }
    }
    assert_eq!(ends.len(), 12, "{ends:?}");
}

// --- Euphorie cristalline ---

// Spec 034 C21 — texte de la carte ; Compendium, Crystal Joy S1.
#[test]
fn strong_crystal_joy_reclaims_with_a_wound() {
    let (joy, march) = (card(CrystalJoy, 1), card(March, 1));
    let reclaim = Action::EndTurn {
        discard: Vec::new(),
        reclaim: Some(wound(1)),
        glade: None,
        steady_tempo: false,
    };
    let state = hero_state(Hero::Goldyx, &[joy, march, wound(1)]);
    let state = with_crystal(state, BasicColor::Blue);
    let state = done(&state, strong_by_crystal(joy, 0));
    let state = done(&state, option(march, 0));
    let state = done(&state, Action::MoveHero { to: Hex::new(1, 0) });
    let state = accepted(&state, reclaim.clone());
    assert!(state.hand(SOLO).contains(&joy));
    assert!(state.discard_pile(SOLO).contains(&wound(1)));

    let state = hero_state(Hero::Goldyx, &[joy, march, wound(1)]);
    let state = with_crystal(state, BasicColor::Red);
    let paid = Action::PlayCard {
        card: joy,
        play: CardPlay::Basic {
            die: None,
            gain: None,
            with: None,
            option: 0,
            discard: None,
            mana: Some(ManaPayment::Crystal),
        },
    };
    let state = done(&state, paid);
    refused(&state, reclaim);
}

// --- Assommé ---

fn swordsmen() -> EnemyToken {
    token(Enemy::Swordsmen, 1)
}

fn minotaur() -> EnemyToken {
    token(Enemy::Minotaur, 1)
}

fn medusa() -> EnemyToken {
    token(Enemy::Medusa, 1)
}

fn cursed_hags() -> EnemyToken {
    token(Enemy::CursedHags, 1)
}

fn three_cards() -> [CardId; 3] {
    [card(March, 1), card(March, 2), card(Stamina, 1)]
}

/// Combat contre `first` (orques ②) et `second` (colline), défiés depuis la plaine ; puis les dégâts
/// sans parade : l'état en phase d'attaque et les événements.
fn take_damage_from(hand: &[CardId], enemies: &[EnemyToken]) -> (GameState, Vec<Event>) {
    let hexes = [ORCS_2, HILL];
    let placed: Vec<(EnemyToken, Hex)> = enemies.iter().copied().zip(hexes).collect();
    let challenged = &hexes[..enemies.len()];
    let state = to_block_phase(&fight(PLAIN_W, hand, &placed, challenged));
    let events = events_of(&state, &Action::NextCombatPhase);
    (events.iter().fold(state, apply), events)
}

fn knocked_out(events: &[Event]) -> Vec<Vec<CardId>> {
    events
        .iter()
        .filter_map(|event| match event {
            Event::KnockedOut { cards, .. } => Some(cards.clone()),
            _ => None,
        })
        .collect()
}

// Spec 034 C22 — MKUE p. 9, Assign Damage Phase 5.c.
#[test]
fn five_wounds_in_one_combat_knock_out() {
    let (state, events) = take_damage_from(&three_cards(), &[minotaur()]);
    assert_eq!(knocked_out(&events), [three_cards().to_vec()]);
    assert_eq!(wounds_in(state.hand(SOLO)), 5);
    assert_eq!(state.hand(SOLO).len(), 5);
    for card in three_cards() {
        assert!(state.discard_pile(SOLO).contains(&card));
    }
    assert!(state.combat(SOLO).is_some_and(|combat| combat.knocked_out));
}

// Spec 034 C22b (issue privée 85) — MKUE p. 6, 5.a et 7 : les cartes défaussées par Assommé comptent pour
// le tour minimum ; le héros a combattu, il ne peut pas se reposer en défaussant une Blessure.
#[test]
fn knocked_out_hero_ends_turn_without_discard() {
    let (state, _) = take_damage_from(&three_cards(), &[minotaur()]);
    let state = done(&state, Action::EndCombat);
    assert_eq!(end_turns(&state), [Vec::<CardId>::new()]);
    accepted(&state, end_turn(&[]));
    refused(&state, end_turn(&[wound(1)]));
}

// Spec 034 C23 — MKUE p. 9, 5.c.
#[test]
fn knock_out_counts_wounds_of_the_whole_combat() {
    let (_, events) = take_damage_from(&three_cards(), &[swordsmen(), prowlers()]);
    let position = |wanted: fn(&Event) -> bool| events.iter().position(wanted);
    let knock_out = position(|event| matches!(event, Event::KnockedOut { .. }));
    let prowlers_wounds =
        position(|event| matches!(event, Event::WoundsTaken { enemy, .. } if *enemy == prowlers()));
    assert!(knock_out.is_some());
    assert_eq!(knock_out, prowlers_wounds.map(|index| index + 1));
}

// Spec 034 C24.
#[test]
fn fewer_wounds_than_hand_limit_no_knock_out() {
    let (state, events) = take_damage_from(&three_cards(), &[swordsmen()]);
    assert!(knocked_out(&events).is_empty());
    assert_eq!(state.hand(SOLO).len(), 6);
}

// Spec 034 C25 — Compendium, Combat S20 ; FAQ p2.
#[test]
fn poison_wounds_in_discard_do_not_count() {
    let (state, events) = take_damage_from(&three_cards(), &[monks()]);
    assert!(knocked_out(&events).is_empty());
    assert_eq!(wounds_in(state.hand(SOLO)), 3);
    assert_eq!(wounds_in(state.discard_pile(SOLO)), 3);

    let (_, events) = take_damage_from(&three_cards(), &[monks(), cursed_hags()]);
    assert_eq!(knocked_out(&events).len(), 1);
}

// Spec 034 C26 — MKUE p. 9, 5.c (« during a combat »).
#[test]
fn wounds_before_combat_do_not_count() {
    let mut hand = vec![wound(101), wound(102), wound(103)];
    hand.extend(three_cards());
    let (state, events) = take_damage_from(&hand, &[swordsmen()]);
    assert!(knocked_out(&events).is_empty());
    assert_eq!(wounds_in(state.hand(SOLO)), 6);
}

// Spec 034 C27 — Compendium, Combat Q22.
#[test]
fn knocked_out_only_once() {
    let (state, events) = take_damage_from(&three_cards(), &[minotaur(), prowlers()]);
    assert_eq!(knocked_out(&events).len(), 1);
    assert_eq!(wounds_in(state.hand(SOLO)), 7);
}

// Spec 034 C28 — MKUE p. 9, 5.c ; p. 10, 6.b.
#[test]
fn knock_out_after_paralysis_discards_nothing() {
    let (state, events) = take_damage_from(&three_cards(), &[medusa(), swordsmen()]);
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::HandParalyzed { .. }))
    );
    assert_eq!(knocked_out(&events), [Vec::<CardId>::new()]);
    let history = &state
        .combat(SOLO)
        .map(|combat| combat.history.clone())
        .unwrap_or_default();
    assert!(matches!(
        history.last(),
        Some(CombatRecord::KnockedOut { enemy, cards: 0 }) if *enemy == foe(&state, HILL)
    ));
}

// Spec 034 C29 — Compendium, Combat Q22.
#[test]
fn knocked_out_combat_goes_on() {
    let swiftness = card(Swiftness, 1);
    let mut hand = vec![swiftness];
    hand.extend(three_cards());
    let state = fight(PLAIN_W, &hand, &[(minotaur(), ORCS_2)], &[ORCS_2]);
    let state = done(&state, declare(&[foe(&state, ORCS_2)]));
    let state = done(&state, strong(swiftness, WHITE_DIE));
    let state = done(&state, Action::ResolveAttack);
    let state = to_block_phase(&state);
    let events = events_of(&state, &Action::NextCombatPhase);
    assert_eq!(knocked_out(&events).len(), 1);
    let state = events.iter().fold(state, apply);
    assert_eq!(
        state.combat(SOLO).map(|combat| combat.phase),
        Some(CombatPhase::Attack)
    );
    assert_eq!(state.play_area(SOLO).len(), 1);
    let state = done(&state, Action::EndCombat);
    let events = events_of(&state, &end_turn(&[]));
    assert!(drawn(&events).is_empty());
}

// Spec 034 C30 — spec 024.
#[test]
fn knock_out_can_be_undone() {
    let (_, events) = take_damage_from(&three_cards(), &[minotaur()]);
    assert_eq!(knocked_out(&events).len(), 1);
    assert!(!events.iter().any(Event::blocks_undo));
}

// Spec 034 L3 : prendre un cristal n'empêche pas le repos.
#[test]
fn rest_allowed_after_gaining_mana() {
    let (crystallize, rage) = (card(Crystallize, 1), card(Rage, 1));
    let state = with_crystal(state_with(&[wound(1), crystallize, rage]), BasicColor::Blue);
    let state = done(&state, strong_by_crystal(crystallize, 0));
    accepted(&state, end_turn(&[rage, wound(1)]));
}
