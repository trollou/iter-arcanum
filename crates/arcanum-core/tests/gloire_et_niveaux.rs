//! Scénarios de la spec 035 (Gloire et niveaux).

use arcanum_core::{
    Action, BasicAction, CardId, Enemy, Event, GameContent, GameState, Hero, LevelReward,
    PlayerEvent, UndoStatus, apply, filter_event, legal_actions, new_game, view,
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

fn gain_fame(state: GameState, fame: u32) -> GameState {
    progress(fame, 1).iter().fold(state, apply)
}

/// Gloire `fame`, niveau `level` : les Rôdeurs (gloire 2) vaincus à distance par Célérité
/// renforcée, combat fini.
fn after_prowlers(fame: u32, level: u8) -> GameState {
    let swiftness = card(Swiftness, 1);
    let hand = [swiftness, card(March, 1), card(Rage, 1)];
    let state = arena_at(VILLAGE, &hand, &[(prowlers(), ORCS_2)], fame, level);
    let state = done(&state, challenge(ORCS_2));
    let state = done(&state, Action::StartCombat);
    let state = done(&state, declare(&[foe(&state, ORCS_2)]));
    let state = done(&state, strong(swiftness, WHITE_DIE));
    let state = done(&state, Action::ResolveAttack);
    done(&state, Action::EndCombat)
}

fn levels_gained(events: &[Event]) -> Vec<u8> {
    events
        .iter()
        .filter_map(|event| match event {
            Event::LevelGained { level, .. } => Some(*level),
            _ => None,
        })
        .collect()
}

fn drawn(events: &[Event]) -> usize {
    events
        .iter()
        .filter(|event| matches!(event, Event::CardDrawn { .. }))
        .count()
}

fn position(events: &[Event], wanted: fn(&Event) -> bool) -> Option<usize> {
    events.iter().position(wanted)
}

fn wounds_taken(events: &[Event]) -> Vec<usize> {
    events
        .iter()
        .filter_map(|event| match event {
            Event::WoundsTaken { wounds, .. } => Some(wounds.len()),
            _ => None,
        })
        .collect()
}

// --- Départ et contenu ---

// Spec 035 C1 — MKUE p. 3, Player Area.
#[test]
fn new_game_starts_at_level_one() {
    let config = arythea_config();
    let state = new_game(&config, &mut config.rngs())
        .iter()
        .fold(GameState::default(), apply);
    let view = view(&state, SOLO);
    assert_eq!(view.level, 1);
    assert_eq!(view.fame, 0);
    assert_eq!(view.armor, 2);
    assert_eq!(view.hand_limit, 5);
    assert_eq!(view.command_tokens, 1);
    assert_eq!(view.next_level_fame, Some(3));
    assert_eq!(view.pending_level, None);
    assert!(view.levels_gained.is_empty());
}

// Spec 035 C2 — scans misc/fame_board.jpg et misc/level_token_*.png.
#[test]
fn levels_content_matches_fame_track_and_level_tokens() {
    let track = GameContent::embedded().fame_track();
    let levels = track.levels();
    assert_eq!(levels.len(), 10);
    let fame: Vec<u32> = levels.iter().map(|level| level.fame).collect();
    assert_eq!(fame, [0, 3, 8, 15, 24, 35, 48, 63, 80, 99]);
    let tokens: Vec<(u8, u8, u8)> = levels
        .iter()
        .map(|level| {
            (
                level.token.armor,
                level.token.hand_limit,
                level.token.command_tokens,
            )
        })
        .collect();
    assert_eq!(
        tokens,
        [
            (2, 5, 1),
            (2, 5, 1),
            (3, 5, 2),
            (3, 5, 2),
            (3, 6, 3),
            (3, 6, 3),
            (4, 6, 4),
            (4, 6, 4),
            (4, 7, 5),
            (4, 7, 5),
        ]
    );
    for (index, level) in levels.iter().enumerate() {
        let expected = match index {
            0 => LevelReward::Start,
            odd if odd % 2 == 0 => LevelReward::LevelToken,
            _ => LevelReward::SkillAndAdvancedAction,
        };
        assert_eq!(level.reward, expected, "niveau {}", index + 1);
    }
}

// --- Changement de niveau ---

// Spec 035 C3 — MKUE p. 9, Ranged and Siege Attack Phase 5.b.
#[test]
fn fame_crossing_a_line_does_not_level_up_immediately() {
    let state = after_prowlers(1, 1);
    assert_eq!(state.fame(SOLO), 3);
    assert_eq!(state.level(SOLO), 1);
    assert_eq!(state.armor(SOLO), 2);
    assert_eq!(view(&state, SOLO).pending_level, Some(2));
}

// Spec 035 C4 — MKUE p. 11, End of the Turn 6.
#[test]
fn end_of_turn_levels_up_after_crossing_a_line() {
    let state = after_prowlers(1, 1);
    let events = events_of(&state, &end_turn(&[]));
    assert_eq!(levels_gained(&events), [2]);
    let turn_ended = position(&events, |event| matches!(event, Event::TurnEnded { .. }));
    let level = position(&events, |event| matches!(event, Event::LevelGained { .. }));
    let first_draw = position(&events, |event| matches!(event, Event::CardDrawn { .. }));
    assert!(turn_ended < level);
    assert!(level < first_draw);
    let state = events.iter().fold(state, apply);
    assert_eq!(state.level(SOLO), 2);
    assert_eq!(state.armor(SOLO), 2);
    assert_eq!(state.hand_limit(SOLO), 5);
    assert_eq!(state.command_tokens(SOLO), 1);
    assert_eq!(view(&state, SOLO).pending_level, None);
}

// Spec 035 C5 — MKUE p. 11, 6.a.
#[test]
fn reaching_exact_threshold_levels_up() {
    let state = after_prowlers(6, 2);
    let state = done(&state, end_turn(&[]));
    assert_eq!(state.fame(SOLO), 8);
    assert_eq!(state.level(SOLO), 3);
    assert_eq!(state.armor(SOLO), 3);
    assert_eq!(state.hand_limit(SOLO), 5);
    assert_eq!(state.command_tokens(SOLO), 2);
}

// Spec 035 C6 — MKUE p. 11, 6.
#[test]
fn crossing_two_lines_gains_two_levels() {
    let rage = card(Rage, 1);
    let state = arena_at(VILLAGE, &[rage, card(March, 1)], &[], 7, 2);
    let state = gain_fame(state, 9);
    let state = done(&state, sideways(rage));
    let events = events_of(&state, &end_turn(&[]));
    assert_eq!(levels_gained(&events), [3, 4]);
    let state = events.iter().fold(state, apply);
    assert_eq!(state.level(SOLO), 4);
}

// Spec 035 C7 — MKUE p. 11, 6.a et 7.b.
#[test]
fn level_up_raises_hand_limit_before_drawing() {
    let rage = card(Rage, 1);
    let state = arena_at(VILLAGE, &[rage, card(March, 1), card(March, 2)], &[], 22, 4);
    let state = gain_fame(state, 2);
    let state = done(&state, sideways(rage));
    let events = events_of(&state, &end_turn(&[]));
    assert_eq!(levels_gained(&events), [5]);
    assert_eq!(drawn(&events), 4);
    let state = events.iter().fold(state, apply);
    assert_eq!(state.hand(SOLO).len(), 6);
}

// Spec 035 C8.
#[test]
fn no_level_up_without_crossing() {
    let rage = card(Rage, 1);
    let state = arena_at(VILLAGE, &[rage, card(March, 1)], &[], 1, 1);
    let state = done(&state, sideways(rage));
    let events = events_of(&state, &end_turn(&[]));
    assert!(levels_gained(&events).is_empty());
}

// Spec 035 C9 — Compendium, Miscellaneous S8 (9).
#[test]
fn level_never_exceeds_ten() {
    let rage = card(Rage, 1);
    let state = arena_at(VILLAGE, &[rage, card(March, 1)], &[], 99, 10);
    assert_eq!(state.level(SOLO), 10);
    assert_eq!(view(&state, SOLO).next_level_fame, None);
    let state = gain_fame(state, 26);
    assert_eq!(view(&state, SOLO).pending_level, None);
    let state = done(&state, sideways(rage));
    let events = events_of(&state, &end_turn(&[]));
    assert!(levels_gained(&events).is_empty());
    let state = events.iter().fold(state, apply);
    assert_eq!(state.level(SOLO), 10);
    assert_eq!(state.fame(SOLO), 125);
}

// Spec 035 L4 : une gloire au-delà de plusieurs seuils, niveau 1 (sauvegarde d'avant la spec).
#[test]
fn missed_levels_are_caught_up_at_end_of_turn() {
    let rage = card(Rage, 1);
    let state = arena_at(VILLAGE, &[rage, card(March, 1)], &[], 9, 1);
    let state = done(&state, sideways(rage));
    let events = events_of(&state, &end_turn(&[]));
    assert_eq!(levels_gained(&events), [2, 3]);
}

// --- Ce que le niveau change ailleurs ---

// Spec 035 C10 — MKUE p. 9, Assign Damage Phase 5.a ; p. 24, Brutal.
#[test]
fn hero_armor_follows_level() {
    let hand = [card(March, 1), card(March, 2), card(Stamina, 1)];
    for (enemy, wounds) in [(Enemy::Gargoyle, 2), (Enemy::Minotaur, 4)] {
        let state = arena_at(VILLAGE, &hand, &[(token(enemy, 1), ORCS_2)], 8, 3);
        let state = done(&state, challenge(ORCS_2));
        let state = done(&state, Action::StartCombat);
        let state = to_block_phase(&state);
        let combat = state.combat(SOLO).expect("combat engagé");
        assert_eq!(
            combat.enemies[0].wounds_if_unblocked(state.armor(SOLO)),
            Some(wounds),
            "{enemy:?}"
        );
        let events = events_of(&state, &Action::NextCombatPhase);
        assert_eq!(wounds_taken(&events), [wounds as usize], "{enemy:?}");
    }
}

// Spec 035 C11 — MKUE p. 4, 2.g.
#[test]
fn new_round_draws_up_to_level_hand_limit() {
    let whole_deck = Hero::Arythea.starting_deck();
    let state = arena_at(VILLAGE, &whole_deck, &[], 24, 5);
    assert!(legal_actions(&state, SOLO).contains(&Action::AnnounceEndOfRound));
    let events = events_of(&state, &Action::AnnounceEndOfRound);
    assert_eq!(drawn(&events), 6);
    let state = events.iter().fold(state, apply);
    assert_eq!(state.hand(SOLO).len(), 6);
}

// Spec 035 C14 — MKUE p. 9, Assign Damage Phase 5.c.
#[test]
fn knock_out_threshold_follows_level() {
    let hand = [card(March, 1), card(March, 2), card(Stamina, 1)];
    // Niveau 5, armure 3 : Loup-garou 7 → 3 Blessures, Minotaure 10 → 4, Épéistes 6 → 2.
    for (first, total, knocked_out) in [(Enemy::Werewolf, 5, false), (Enemy::Minotaur, 6, true)] {
        let placed = [
            (token(first, 1), ORCS_2),
            (token(Enemy::Swordsmen, 1), HILL),
        ];
        let state = arena_at(PLAIN_W, &hand, &placed, 24, 5);
        let state = done(&state, challenge(ORCS_2));
        let state = done(&state, challenge(HILL));
        let state = done(&state, Action::StartCombat);
        let state = to_block_phase(&state);
        let events = events_of(&state, &Action::NextCombatPhase);
        assert_eq!(wounds_taken(&events).iter().sum::<usize>(), total);
        assert_eq!(
            events
                .iter()
                .any(|event| matches!(event, Event::KnockedOut { .. })),
            knocked_out,
            "{total} Blessures"
        );
    }
}

// --- Événement et retour en arrière ---

// Spec 035, Actions et événements : public, pas un point de non-retour.
#[test]
fn level_gained_is_public_and_does_not_block_undo() {
    let event = Event::LevelGained {
        player: SOLO,
        level: 3,
    };
    assert_eq!(
        filter_event(&event, SOLO),
        PlayerEvent::LevelGained {
            player: SOLO,
            level: 3
        }
    );
    assert!(!event.blocks_undo());
}

/// Joueur piloté par le test : il garde les actions légales reçues et répond l'index déposé.
// Spec 035 C12 — spec 024.
#[test]
fn level_up_follows_undo_rules() {
    let swiftness = card(Swiftness, 1);
    let mut events = arena_events(
        VILLAGE,
        &[swiftness, card(March, 1)],
        &[(prowlers(), ORCS_2)],
        &[],
    );
    events.extend(progress(1, 1));
    let (mut runner, puppet) = runner(events);
    for action in [
        challenge(ORCS_2),
        Action::StartCombat,
        declare(&[prowlers()]),
        strong(swiftness, WHITE_DIE),
        Action::ResolveAttack,
    ] {
        play(&mut runner, &puppet, action);
    }
    assert_eq!(view(runner.state(), SOLO).pending_level, Some(2));
    assert_eq!(runner.undo(), Ok(()));
    assert_eq!(runner.state().fame(SOLO), 1);
    assert_eq!(view(runner.state(), SOLO).pending_level, None);

    play(&mut runner, &puppet, Action::ResolveAttack);
    play(&mut runner, &puppet, Action::EndCombat);
    play(&mut runner, &puppet, end_turn(&[]));
    assert_eq!(runner.state().level(SOLO), 2);
    assert_ne!(runner.undo_status(), UndoStatus::Possible);
}
