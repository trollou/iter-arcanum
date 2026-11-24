//! Scénarios de la spec 031 (déroulé d'un combat).

use arcanum_core::{
    Action, AttackKind, BasicAction, BoostTarget, CardId, CardPlay, CombatPhase, CombatRecord,
    Effect, EnemyColor, EnemyToken, Event, GameRunner, GameState, Hero, Hex, InProgress,
    ManaPayment, PlayerEvent, PlayerId, RngUsage, TileId, TilePosition, UndoError, UndoStatus,
    apply, filter_event, legal_actions, view,
};
use rand::seq::SliceRandom;

mod arene;
use arene::*;

use BasicAction::*;

// --- Bac à sable avec des ennemis errants (mêmes cases que la spec 030) ---

fn arena_hand() -> Vec<CardId> {
    vec![
        card(March, 1),
        card(March, 2),
        card(Promise, 1),
        card(Concentration, 1),
        card(Rage, 1),
    ]
}

/// Les Rôdeurs sur les orques ②, les Creuseurs sur les orques ①.
fn printed_tokens() -> Vec<(EnemyToken, Hex)> {
    vec![(diggers(), ORCS_1), (prowlers(), ORCS_2)]
}

/// Au village avec 4 points (deux Marche), puis le pas qui provoque les Rôdeurs.
fn after_provocation(placed: &[(EnemyToken, Hex)]) -> GameState {
    let state = arena(VILLAGE, &arena_hand(), placed);
    let state = done(&state, basic(card(March, 1)));
    let state = done(&state, basic(card(March, 2)));
    done(&state, Action::MoveHero { to: PLAIN_W })
}

fn end_turns(state: &GameState) -> Vec<Action> {
    legal_actions(state, SOLO)
        .into_iter()
        .filter(|action| matches!(action, Action::EndTurn { .. }))
        .collect()
}

/// Les Rôdeurs défiés depuis le village, combat engagé.
fn fighting_prowlers() -> GameState {
    let state = arena(VILLAGE, &arena_hand(), &printed_tokens());
    let state = done(&state, challenge(ORCS_2));
    done(&state, Action::StartCombat)
}

// --- Blessures ---

// Spec 031 C26
#[test]
fn wound_cannot_be_played() {
    // MKUE p. 4, 3.d et 3.e : une Blessure ne se joue d'aucune façon, pas même de côté.
    let state = state_with(&[wound(1), card(March, 1), card(Rage, 1)]);
    let actions = legal_actions(&state, SOLO);
    assert!(
        actions
            .iter()
            .all(|action| !matches!(action, Action::PlayCard { card, .. } if *card == wound(1)))
    );
    // Les autres cartes se jouent toujours.
    assert!(actions.contains(&sideways(card(March, 1))));
}

// Spec 031 C27
#[test]
fn wound_cannot_pay_a_discard_cost() {
    // MKUE p. 4, 6.a : « une autre carte » ne désigne jamais une Blessure.
    let improvisation = card(Improvisation, 1);
    let state = state_with(&[improvisation, wound(1), card(March, 1)]);
    let discards: Vec<Option<CardId>> = legal_actions(&state, SOLO)
        .into_iter()
        .filter_map(|action| match action {
            Action::PlayCard {
                card,
                play: CardPlay::Basic { discard, .. } | CardPlay::Strong { discard, .. },
            } if card == improvisation => Some(discard),
            _ => None,
        })
        .collect();
    assert!(discards.contains(&Some(card(March, 1))));
    assert!(!discards.contains(&Some(wound(1))));
}

// Spec 031 C28
#[test]
fn wounds_stay_in_hand_and_count_for_drawing() {
    // MKUE p. 11, 7.a à 7.c : on ne défausse pas une Blessure, et elle compte dans la main.
    // Spec 034 : sauf au repos ; le héros s'est donc déplacé (plaine voisine du portail).
    let state = state_with(&[wound(1), wound(2), card(Rage, 1), card(March, 1)]);
    let state = done(&state, basic(card(March, 1)));
    let state = done(&state, Action::MoveHero { to: Hex::new(1, 0) });
    let end_turns: Vec<Action> = legal_actions(&state, SOLO)
        .into_iter()
        .filter(|action| matches!(action, Action::EndTurn { .. }))
        .collect();
    assert!(!end_turns.is_empty());
    for action in &end_turns {
        let Action::EndTurn {
            discard, reclaim, ..
        } = action
        else {
            unreachable!();
        };
        assert!(!discard.contains(&wound(1)) && !discard.contains(&wound(2)));
        assert!(reclaim.is_none());
    }
    let state = done(
        &state,
        Action::EndTurn {
            discard: Vec::new(),
            reclaim: None,
            glade: None,
            steady_tempo: false,
        },
    );
    let hand = state.hand(SOLO);
    assert_eq!(hand.len(), 5);
    assert!(hand.contains(&wound(1)) && hand.contains(&wound(2)));
}

// Spec 031 C28 bis
#[test]
fn only_wounds_in_hand_allow_slow_recovery() {
    // MKUE p. 6, 7.a (*Slow Recovery*) : main faite seulement de Blessures, on en défausse une.
    let state = state_with(&[wound(1), wound(2)]);
    assert!(!state.deck(SOLO).is_empty());
    let slow_recovery = |copy| Action::EndTurn {
        discard: vec![wound(copy)],
        reclaim: None,
        glade: None,
        steady_tempo: false,
    };
    let end_turns: Vec<Action> = legal_actions(&state, SOLO)
        .into_iter()
        .filter(|action| matches!(action, Action::EndTurn { .. }))
        .collect();
    // Spec 034 : une seule combinaison par nombre de Blessures ; en nommer une autre reste valide.
    assert_eq!(end_turns, vec![slow_recovery(1)]);
    let state = done(&state, slow_recovery(2));
    assert_eq!(state.discard_pile(SOLO), [wound(2)]);
    assert!(state.hand(SOLO).contains(&wound(1)));
    assert!(!state.hand(SOLO).contains(&wound(2)));
}

// Spec 031 C29
#[test]
fn wounds_are_shuffled_into_the_new_deck() {
    // MKUE p. 11, *Wounds and Healing* 1.c : à la fin de la manche, les Blessures de la main
    // rejoignent le deck avec les autres cartes.
    let mut order = Hero::Tovak.starting_deck();
    order.extend([wound(1), wound(2)]);
    let count = order.len();
    let state = state_with_deck(order, count);
    let events = events_of(&state, &Action::AnnounceEndOfRound);
    let shuffled = events.iter().find_map(|event| match event {
        Event::DeckShuffled { order, .. } => Some(order),
        _ => None,
    });
    let shuffled = shuffled.expect("la nouvelle manche mélange le deck");
    assert_eq!(shuffled.len(), count);
    assert!(shuffled.contains(&wound(1)) && shuffled.contains(&wound(2)));
}

// --- Engager le combat ---

// Spec 031 C1
#[test]
fn provoked_combat_is_mandatory() {
    // MKUE p. 6, 6.b : le combat provoqué est obligatoire ; Compendium, *Combat* Q1 : rien d'autre
    // à jouer avant de l'engager.
    let state = after_provocation(&printed_tokens());
    assert!(end_turns(&state).is_empty());
    assert_eq!(legal_actions(&state, SOLO), vec![Action::StartCombat]);
}

// Spec 031 C2
#[test]
fn challenging_starts_the_action() {
    // MKUE p. 8, 1.d ; p. 6, 6.d et 6.e.
    let state = arena(VILLAGE, &arena_hand(), &printed_tokens());
    let state = done(&state, basic(card(March, 1)));
    assert_eq!(state.move_points(SOLO), 2);
    assert!(legal_actions(&state, SOLO).contains(&challenge(ORCS_2)));
    let events = events_of(&state, &challenge(ORCS_2));
    assert_eq!(
        events,
        vec![Event::EnemyChallenged {
            player: SOLO,
            enemy: ORCS_2
        }]
    );
    let state = events.iter().fold(state, apply);
    assert_eq!(state.move_points(SOLO), 0);
    assert_eq!(state.enemies_to_fight(SOLO), [ORCS_2]);
    assert!(
        legal_actions(&state, SOLO)
            .iter()
            .all(|action| !matches!(action, Action::MoveHero { .. } | Action::Explore { .. }))
    );
    assert_eq!(legal_actions(&state, SOLO), vec![Action::StartCombat]);
}

// Spec 031 C3
#[test]
fn challenge_after_provocation_joins_the_fight() {
    // MKUE p. 8, 2.c : défier un ennemi voisin de la case où la figurine est entrée.
    let placed = [(prowlers(), ORCS_2), (diggers(), HILL)];
    let state = after_provocation(&placed);
    assert_eq!(
        legal_actions(&state, SOLO),
        vec![challenge(HILL), Action::StartCombat]
    );
    let state = done(&state, challenge(HILL));
    assert_eq!(
        events_of(&state, &Action::StartCombat),
        vec![Event::CombatStarted {
            player: SOLO,
            enemies: vec![(ORCS_2, prowlers()), (HILL, diggers())],
        }]
    );
}

// Spec 031 C4
#[test]
fn only_adjacent_new_enemies_can_be_challenged() {
    let state = after_provocation(&printed_tokens());
    refused(&state, challenge(ORCS_2));
    refused(&state, challenge(ORCS_1));
    // Une case voisine sans jeton.
    refused(&state, challenge(HILL));
}

/// Les phases d'un combat sans attaque ni parade, jusqu'à sa fin.
fn fight_to_the_end(state: &GameState) -> GameState {
    let state = done(state, Action::NextCombatPhase);
    let state = done(&state, Action::NextCombatPhase);
    assert!(!legal_actions(&state, SOLO).contains(&Action::NextCombatPhase));
    done(&state, Action::EndCombat)
}

// Spec 031 C5
#[test]
fn one_combat_per_turn() {
    // MKUE p. 8, 2 : un seul combat par tour.
    let placed = [(prowlers(), ORCS_2), (diggers(), HILL)];
    let state = arena(PLAIN_W, &arena_hand(), &placed);
    let state = done(&state, challenge(ORCS_2));
    let state = done(&state, Action::StartCombat);
    let state = fight_to_the_end(&state);
    let legal = legal_actions(&state, SOLO);
    assert!(
        legal
            .iter()
            .all(|action| !matches!(action, Action::Challenge { .. } | Action::StartCombat))
    );
    refused(&state, challenge(HILL));
    refused(&state, Action::StartCombat);
    // MKUE p. 10, *Combat Outcomes* 3.a : le combat fini, le tour peut finir.
    assert!(!end_turns(&state).is_empty());
}

// Spec 031 C23 (et C10 pour le mana)
#[test]
fn move_and_influence_are_not_played_in_combat() {
    // MKUE p. 6, 6.e : l'action a commencé ; p. 8, 6 : le mana se gagne dans toutes les phases.
    let state = fighting_prowlers();
    assert_eq!(
        state.combat(SOLO).map(|combat| combat.phase),
        Some(CombatPhase::RangedAndSiege)
    );
    let legal = legal_actions(&state, SOLO);
    for action in &legal {
        let Action::PlayCard { card, play } = action else {
            continue;
        };
        let effect = card.effect(*play);
        assert!(
            !matches!(effect, Some(Effect::Move(_) | Effect::Influence(_))),
            "{action:?}"
        );
    }
    // Concentration, effet de base : un jeton de mana.
    assert!(legal.contains(&basic(card(Concentration, 1))));
    refused(&state, basic(card(Promise, 1)));
    refused(&state, sideways(card(Rage, 1)));
}

// --- Aides pour les attaques ---

/// Les Rôdeurs seuls sur les orques ②, défiés depuis le village.
fn fight_prowlers_with(hand: &[CardId]) -> GameState {
    fight(VILLAGE, hand, &[(prowlers(), ORCS_2)], &[ORCS_2])
}

// --- Phase d'attaque à distance ---

// Spec 031 C6
#[test]
fn ranged_attack_defeats_enemy_and_gains_fame() {
    // MKUE p. 9, *Ranged and Siege Attack Phase* 5 : l'ennemi vaincu quitte le plateau de jeu et
    // rapporte sa gloire.
    let swiftness = card(Swiftness, 1);
    let state = fight_prowlers_with(&[swiftness]);
    assert!(legal_actions(&state, SOLO).contains(&declare(&[foe(&state, ORCS_2)])));
    let state = done(&state, declare(&[foe(&state, ORCS_2)]));
    let state = done(&state, strong(swiftness, WHITE_DIE));
    let events = events_of(&state, &Action::ResolveAttack);
    assert_events_in_order(
        &events,
        &[Event::EnemiesDefeated {
            player: SOLO,
            enemies: vec![foe(&state, ORCS_2)],
            fame: 2,
        }],
    );
    let state = events.iter().fold(state, apply);
    assert!(!state.enemy_tokens().contains_key(&ORCS_2));
    assert_eq!(state.enemy_discard(EnemyColor::Green), [prowlers()]);
    assert_eq!(state.fame(SOLO), 2);
    let combat = state.combat(SOLO).expect("le combat continue");
    assert!(combat.enemies[0].defeated);
    assert_eq!(combat.in_progress, None);
}

// Spec 031 C7
#[test]
fn insufficient_attack_has_no_effect() {
    // MKUE p. 9, *Ranged and Siege Attack Phase* 6 : trop peu de points, rien ne se passe ; les
    // cartes jouées restent jouées.
    let versatility = card(BattleVersatility, 1);
    let state = fight_prowlers_with(&[versatility]);
    let state = done(&state, declare(&[foe(&state, ORCS_2)]));
    // Attaque à distance 1 (troisième option).
    let state = done(&state, option(versatility, 2));
    assert_eq!(
        state
            .combat(SOLO)
            .and_then(|combat| combat.in_progress.clone()),
        Some(InProgress::Attack {
            targets: vec![foe(&state, ORCS_2)],
            points: physical(1),
        })
    );
    let events = events_of(&state, &Action::ResolveAttack);
    assert_eq!(events, vec![Event::AttackFailed { player: SOLO }]);
    let state = events.iter().fold(state, apply);
    assert_eq!(state.play_area(SOLO).len(), 1);
    assert!(state.enemy_tokens().contains_key(&ORCS_2));
    assert_eq!(state.fame(SOLO), 0);
    let combat = state.combat(SOLO).expect("le combat continue");
    assert!(!combat.enemies[0].defeated);
    assert_eq!(combat.in_progress, None);
}

// Spec 031 C8
#[test]
fn ranged_phase_accepts_only_ranged_attacks() {
    // MKUE p. 9, *Ranged and Siege Attack Phase* 3 et 3.b : seulement l'attaque à distance, jamais
    // une carte de côté.
    let (rage, versatility, swiftness) = (
        card(Rage, 1),
        card(BattleVersatility, 1),
        card(Swiftness, 1),
    );
    let state = fight_prowlers_with(&[rage, versatility, swiftness, card(March, 1)]);
    let state = done(&state, declare(&[foe(&state, ORCS_2)]));
    let legal = legal_actions(&state, SOLO);
    assert!(legal.contains(&option(versatility, 2)));
    assert!(legal.contains(&strong(swiftness, WHITE_DIE)));
    // L'attaque de mêlée de Rage, et de Polyvalence, ne sont pas jouables.
    refused(&state, option(rage, 0));
    refused(&state, option(versatility, 0));
    // Distance ou siège seulement (le siège de la Polyvalence renforcée : spec 032, C13).
    for effect in attack_effects(&playable_effects(&state)) {
        assert!(
            matches!(
                effect,
                Effect::Attack {
                    kind: AttackKind::Ranged | AttackKind::Siege,
                    ..
                }
            ),
            "{effect:?}"
        );
    }
    // Aucune carte de côté, même avec une attaque en cours.
    assert!(legal.iter().all(|action| !matches!(
        action,
        Action::PlayCard {
            play: CardPlay::Sideways(_),
            ..
        }
    )));
    refused(&state, sideways_attack(card(March, 1)));
}

// Spec 031 C9, modifié par la spec 032 (Q1) : avec l'attaque de siège, un fortifié est une cible
// en phase d'attaque à distance (son attaque n'accepte que du siège : spec 032, C12).
#[test]
fn fortified_enemy_is_a_target_in_both_attack_phases() {
    // MKUE p. 9, *Ranged and Siege Attack Phase* 3.a ; p. 10, *Attack Phase* 1.a.
    let placed = [(prowlers(), ORCS_2), (diggers(), HILL)];
    let state = fight(PLAIN_W, &arena_hand(), &placed, &[ORCS_2, HILL]);
    let all = vec![vec![ORCS_2], vec![HILL], vec![ORCS_2, HILL]];
    assert_eq!(attack_targets(&state), all);
    let state = to_attack_phase(&state);
    assert_eq!(attack_targets(&state), all);
}

// Spec 031 C10
#[test]
fn attack_cards_need_an_open_attack() {
    // MKUE p. 9, *Ranged and Siege Attack Phase* 8 : pas de réserve de points ; p. 8, 6 : le mana
    // se gagne dans toutes les phases.
    let (rage, swiftness) = (card(Rage, 1), card(Swiftness, 1));
    let state = fight_prowlers_with(&[rage, swiftness, card(Concentration, 1)]);
    for state in [state.clone(), to_attack_phase(&state)] {
        assert!(attack_effects(&playable_effects(&state)).is_empty());
        refused(&state, strong(swiftness, WHITE_DIE));
        refused(&state, option(rage, 0));
        refused(&state, sideways_attack(rage));
        assert!(legal_actions(&state, SOLO).contains(&basic(card(Concentration, 1))));
    }
}

// Spec 031 C11
#[test]
fn one_attack_can_defeat_several_enemies() {
    // MKUE p. 9, *Ranged and Siege Attack Phase* 5 et 7 : un groupe de cibles est vaincu en
    // entier ou pas du tout, il faut la somme des armures (3 + 4).
    let (rage, determination, march) = (card(Rage, 1), card(Determination, 1), card(March, 1));
    let placed = [(prowlers(), ORCS_2), (wolf_riders(), HILL)];
    let state = fight(
        PLAIN_W,
        &[rage, determination, march],
        &placed,
        &[ORCS_2, HILL],
    );
    let state = to_attack_phase(&state);
    let state = done(&state, declare(&[foe(&state, ORCS_2), foe(&state, HILL)]));
    // Rage renforcée (Attaque 4, dé rouge) + Détermination (Attaque 2) : 6 < 7.
    let state = done(&state, strong(rage, RED_DIE));
    let state = done(&state, option(determination, 0));
    assert_eq!(
        events_of(&state, &Action::ResolveAttack),
        vec![Event::AttackFailed { player: SOLO }]
    );
    // Une carte de côté (Attaque 1) : 7, les deux sont vaincus, gloire 2 + 3.
    let state = done(&state, sideways_attack(march));
    let events = events_of(&state, &Action::ResolveAttack);
    assert_events_in_order(
        &events,
        &[Event::EnemiesDefeated {
            player: SOLO,
            enemies: vec![foe(&state, ORCS_2), foe(&state, HILL)],
            fame: 5,
        }],
    );
    let state = events.iter().fold(state, apply);
    assert_eq!(state.fame(SOLO), 5);
    assert!(
        !state.enemy_tokens().contains_key(&ORCS_2) && !state.enemy_tokens().contains_key(&HILL)
    );
    assert_eq!(
        state.enemy_discard(EnemyColor::Green),
        [prowlers(), wolf_riders()]
    );
}

// Spec 031 C11 bis
#[test]
fn failed_target_cannot_be_attacked_again_this_phase() {
    // Q10 ; Compendium, *Combat* S28 : une seule attaque par ennemi et par phase.
    let state = fight_prowlers_with(&arena_hand());
    let state = done(&state, declare(&[foe(&state, ORCS_2)]));
    // L2 : validée à 0 point, l'attaque échoue.
    let state = done(&state, Action::ResolveAttack);
    assert!(attack_targets(&state).is_empty());
    refused(&state, declare(&[foe(&state, ORCS_2)]));
    assert!(legal_actions(&state, SOLO).contains(&Action::NextCombatPhase));
    // La phase suivante lève la marque : MKUE p. 10, *Attack Phase*.
    let state = to_attack_phase(&state);
    assert_eq!(attack_targets(&state), vec![vec![ORCS_2]]);
}

// --- Aides pour les parades et les dégâts ---

// --- Phase de parade ---

// Spec 031 C12
#[test]
fn block_equal_or_above_attack_blocks_enemy() {
    // MKUE p. 9, *Block Phase* 6 et 7 : Parade 5 contre attaque 4, l'ennemi est paré mais reste en
    // jeu.
    let determination = card(Determination, 1);
    let state = to_block_phase(&fight_prowlers_with(&[determination]));
    assert_eq!(block_targets(&state), vec![ORCS_2]);
    let state = done(&state, declare_block(foe(&state, ORCS_2)));
    let state = done(&state, strong(determination, BLUE_DIE));
    assert_eq!(
        state
            .combat(SOLO)
            .and_then(|combat| combat.in_progress.clone()),
        Some(InProgress::Block {
            target: foe(&state, ORCS_2),
            points: physical(5),
        })
    );
    let events = events_of(&state, &Action::ResolveBlock);
    assert_eq!(
        events,
        vec![Event::EnemyBlocked {
            player: SOLO,
            enemy: foe(&state, ORCS_2)
        }]
    );
    let state = events.iter().fold(state, apply);
    let combat = state.combat(SOLO).expect("le combat continue");
    assert!(combat.enemies[0].blocked);
    assert!(!combat.enemies[0].defeated);
    assert_eq!(combat.in_progress, None);
    assert!(state.enemy_tokens().contains_key(&ORCS_2));
    assert_eq!(state.fame(SOLO), 0);
}

// Spec 031 C13
#[test]
fn insufficient_block_has_no_effect() {
    // MKUE p. 9, *Block Phase* 4.b et 8 : pas de demi-parade, et les cartes jouées restent jouées.
    let (rage, march) = (card(Rage, 1), card(March, 1));
    let state = to_block_phase(&fight_prowlers_with(&[rage, march]));
    let state = done(&state, declare_block(foe(&state, ORCS_2)));
    // Rage, Parade 2 (seconde option), et une carte de côté, Parade 1 : 3 < 4.
    let state = done(&state, option(rage, 1));
    let state = done(&state, sideways_block(march));
    assert_eq!(
        state
            .combat(SOLO)
            .and_then(|combat| combat.in_progress.clone()),
        Some(InProgress::Block {
            target: foe(&state, ORCS_2),
            points: physical(3),
        })
    );
    let events = events_of(&state, &Action::ResolveBlock);
    assert_eq!(events, vec![Event::BlockFailed { player: SOLO }]);
    let state = events.iter().fold(state, apply);
    assert_eq!(state.play_area(SOLO).len(), 2);
    let combat = state.combat(SOLO).expect("le combat continue");
    assert!(!combat.enemies[0].blocked && !combat.enemies[0].defeated);
    assert_eq!(combat.in_progress, None);
}

// Spec 031 C14
#[test]
fn block_targets_one_unblocked_enemy() {
    // MKUE p. 9, *Block Phase* 3 et 9 : une parade ne vise qu'un ennemi ; un ennemi déjà paré n'est
    // plus une cible.
    let determination = card(Determination, 1);
    let placed = [(prowlers(), ORCS_2), (wolf_riders(), HILL)];
    let state = fight(PLAIN_W, &[determination], &placed, &[ORCS_2, HILL]);
    // Pas de parade avant la phase de parade.
    refused(&state, declare_block(foe(&state, ORCS_2)));
    let state = to_block_phase(&state);
    assert_eq!(block_targets(&state), vec![ORCS_2, HILL]);
    // Une parade en cours : aucune autre cible, seulement la valider.
    let state = done(&state, declare_block(foe(&state, ORCS_2)));
    assert!(block_targets(&state).is_empty());
    refused(&state, declare_block(foe(&state, HILL)));
    refused(&state, declare_block(foe(&state, ORCS_2)));
    // Parade 5 : Rôdeurs parés, il reste les Chevaucheurs de loups à parer.
    let state = done(&state, strong(determination, BLUE_DIE));
    let state = done(&state, Action::ResolveBlock);
    assert_eq!(block_targets(&state), vec![HILL]);
    refused(&state, declare_block(foe(&state, ORCS_2)));
}

// Spec 031 C14 bis
#[test]
fn failed_block_cannot_be_retried() {
    // Q10 : une seule parade par ennemi ; l'autre ennemi reste une cible.
    let placed = [(prowlers(), ORCS_2), (wolf_riders(), HILL)];
    let state = to_block_phase(&fight(PLAIN_W, &arena_hand(), &placed, &[ORCS_2, HILL]));
    let state = done(&state, declare_block(foe(&state, ORCS_2)));
    // Validée à 0 point : la parade échoue.
    assert_eq!(
        events_of(&state, &Action::ResolveBlock),
        vec![Event::BlockFailed { player: SOLO }]
    );
    let state = done(&state, Action::ResolveBlock);
    assert_eq!(block_targets(&state), vec![HILL]);
    refused(&state, declare_block(foe(&state, ORCS_2)));
    assert!(legal_actions(&state, SOLO).contains(&Action::NextCombatPhase));
}

// Spec 031 C15
#[test]
fn block_cards_need_an_open_block() {
    // MKUE p. 9, *Block Phase* 10 : pas de réserve de points, une Parade se joue dans une parade
    // en cours.
    let (rage, determination, march) = (card(Rage, 1), card(Determination, 1), card(March, 1));
    let state = fight_prowlers_with(&[rage, determination, march]);
    // Dans chaque phase, sans parade en cours : aucune carte de Parade, de côté non plus.
    for state in [
        state.clone(),
        to_block_phase(&state),
        to_attack_phase(&state),
    ] {
        assert!(block_effects(&state).is_empty());
        refused(&state, option(rage, 1));
        refused(&state, strong(determination, BLUE_DIE));
        refused(&state, sideways_block(march));
    }
    // Pendant une attaque, pas de Parade non plus.
    let attacking = done(&to_attack_phase(&state), declare(&[foe(&state, ORCS_2)]));
    assert!(block_effects(&attacking).is_empty());
    refused(&attacking, sideways_block(march));
    // Avec une parade en cours : la Parade, de base, renforcée ou de côté, mais pas d'Attaque.
    let blocking = done(&to_block_phase(&state), declare_block(foe(&state, ORCS_2)));
    let legal = legal_actions(&blocking, SOLO);
    assert!(legal.contains(&option(rage, 1)));
    assert!(legal.contains(&option(determination, 1)));
    assert!(legal.contains(&strong(determination, BLUE_DIE)));
    assert!(legal.contains(&sideways_block(march)));
    assert!(attack_effects(&playable_effects(&blocking)).is_empty());
    refused(&blocking, option(rage, 0));
    refused(&blocking, sideways_attack(march));
}

// Spec 031 L1 (parades)
#[test]
fn block_actions_are_refused_out_of_their_moment() {
    // Valider sans parade en cours, phase suivante avec une parade en cours.
    let state = fight_prowlers_with(&arena_hand());
    refused(&state, Action::ResolveBlock);
    let block = to_block_phase(&state);
    refused(&block, Action::ResolveBlock);
    refused(&block, Action::ResolveAttack);
    refused(&block, declare(&[foe(&state, ORCS_2)]));
    let declared = done(&block, declare_block(foe(&state, ORCS_2)));
    refused(&declared, Action::NextCombatPhase);
    refused(&declared, Action::ResolveAttack);
    // Plus de parade une fois la phase passée.
    refused(&to_attack_phase(&state), declare_block(foe(&state, ORCS_2)));
}

// --- Phase d'assignation des dégâts ---

// Spec 031 C16
#[test]
fn unblocked_enemy_wounds_the_hero() {
    // MKUE p. 9, *Assign Damage Phase* 2 et 5 : attaque 4, armure 2 du héros (MKUE p. 3) : 2
    // Blessures, mises dans la main (MKUE p. 11, *Wounds and Healing* 1).
    let march = card(March, 1);
    let state = to_block_phase(&fight_prowlers_with(&[march]));
    let events = events_of(&state, &Action::NextCombatPhase);
    assert_eq!(
        events,
        vec![
            Event::CombatPhaseStarted {
                player: SOLO,
                phase: CombatPhase::AssignDamage
            },
            Event::WoundsTaken {
                player: SOLO,
                enemy: foe(&state, ORCS_2),
                wounds: vec![wound(1), wound(2)]
            },
            Event::CombatPhaseStarted {
                player: SOLO,
                phase: CombatPhase::Attack
            },
        ]
    );
    let state = events.iter().fold(state, apply);
    assert_eq!(state.hand(SOLO), [march, wound(1), wound(2)]);
    assert_eq!(current_phase(&state), Some(CombatPhase::Attack));
}

// Spec 031 C17
#[test]
fn wounds_round_up_against_armor() {
    // MKUE p. 9, *Assign Damage Phase* 5.b ; Compendium, *Combat* S38 : une Blessure par tranche
    // entamée de 2 points, ennemi par ennemi : attaque 3 donne 2, attaque 5 donne 3, soit 5 en
    // tout, pas 4.
    // Gargouille plutôt que Moines (toxiques depuis la spec 033) : attaque 5, sans capacité.
    let placed = [
        (wolf_riders(), ORCS_2),
        (token(arcanum_core::Enemy::Gargoyle, 1), HILL),
    ];
    let state = to_block_phase(&fight(PLAIN_W, &[], &placed, &[ORCS_2, HILL]));
    let events = events_of(&state, &Action::NextCombatPhase);
    assert_eq!(
        events,
        vec![
            Event::CombatPhaseStarted {
                player: SOLO,
                phase: CombatPhase::AssignDamage
            },
            Event::WoundsTaken {
                player: SOLO,
                enemy: foe(&state, ORCS_2),
                wounds: vec![wound(1), wound(2)]
            },
            Event::WoundsTaken {
                player: SOLO,
                enemy: foe(&state, HILL),
                wounds: vec![wound(3), wound(4), wound(5)]
            },
            // Spec 034 : 5 Blessures dans ce combat, le héros est assommé (main vide).
            Event::KnockedOut {
                player: SOLO,
                cards: Vec::new(),
            },
            Event::CombatPhaseStarted {
                player: SOLO,
                phase: CombatPhase::Attack
            },
        ]
    );
    let state = events.iter().fold(state, apply);
    assert_eq!(
        state.hand(SOLO),
        [wound(1), wound(2), wound(3), wound(4), wound(5)]
    );
}

// Spec 031 C17 (numéros d'exemplaire des Blessures)
#[test]
fn wound_copies_continue_after_earlier_wounds() {
    // Spec 031, « Nouvelles données dans l'état » : chaque Blessure prise a son numéro, le moteur
    // compte celles déjà données, même si elles ne sont plus en main.
    let state = to_block_phase(&fight_prowlers_with(&[]));
    let given = Event::WoundsTaken {
        player: SOLO,
        enemy: foe(&state, ORCS_2),
        wounds: vec![wound(1), wound(2), wound(3)],
    };
    let state = apply(state, &given);
    let state = done(&state, Action::NextCombatPhase);
    assert_eq!(
        state.hand(SOLO),
        [wound(1), wound(2), wound(3), wound(4), wound(5)]
    );
}

// Spec 031, « Nouvelles données dans l'état » (issue privée 76)
#[test]
fn wound_copies_go_beyond_255() {
    // Une longue partie donne plus de 255 Blessures : les suivantes gardent des numéros distincts.
    let state = to_block_phase(&fight_prowlers_with(&[]));
    let given = Event::WoundsTaken {
        player: SOLO,
        enemy: foe(&state, ORCS_2),
        wounds: (1..=255).map(wound).collect(),
    };
    let state = apply(state, &given);
    let state = done(&state, Action::NextCombatPhase);
    assert_eq!(state.hand(SOLO)[255..], [wound(256), wound(257)]);
}

// Spec 031 C18
#[test]
fn no_damage_skips_the_phase() {
    // MKUE p. 9, *Assign Damage Phase* 1.a : ennemi paré ou vaincu, aucun dégât, la phase est
    // sautée et le combat passe à la phase d'attaque.
    let attack_phase = vec![Event::CombatPhaseStarted {
        player: SOLO,
        phase: CombatPhase::Attack,
    }];
    // Paré.
    let determination = card(Determination, 1);
    let state = to_block_phase(&fight_prowlers_with(&[determination]));
    let state = done(&state, declare_block(foe(&state, ORCS_2)));
    let state = done(&state, strong(determination, BLUE_DIE));
    let state = done(&state, Action::ResolveBlock);
    assert_eq!(events_of(&state, &Action::NextCombatPhase), attack_phase);
    let state = done(&state, Action::NextCombatPhase);
    assert_eq!(current_phase(&state), Some(CombatPhase::Attack));
    assert_eq!(state.hand(SOLO), []);
    // Vaincu à distance : le combat se termine tout de suite (L4, amendée le 2026-10-29 ;
    // `combat_can_end_once_every_enemy_is_defeated`).
}

// --- Phase d'attaque et fin du combat ---

// Spec 031 C19
#[test]
fn melee_and_sideways_attack_in_attack_phase() {
    // MKUE p. 10, *Attack Phase* 1.b : une carte de côté donne Attaque 1 ; les Rôdeurs sont parés
    // (Détermination renforcée), un ennemi paré est une cible comme un autre (1).
    let (rage, determination, march) = (card(Rage, 1), card(Determination, 1), card(March, 1));
    let state = to_block_phase(&fight_prowlers_with(&[rage, determination, march]));
    let state = done(&state, declare_block(foe(&state, ORCS_2)));
    let state = done(&state, strong(determination, BLUE_DIE));
    let state = done(&state, Action::ResolveBlock);
    let state = done(&state, Action::NextCombatPhase);
    assert_eq!(current_phase(&state), Some(CombatPhase::Attack));
    assert!(state.combat(SOLO).is_some_and(|c| c.enemies[0].blocked));
    assert_eq!(attack_targets(&state), vec![vec![ORCS_2]]);
    let state = done(&state, declare(&[foe(&state, ORCS_2)]));
    let state = done(&state, option(rage, 0));
    let state = done(&state, sideways_attack(march));
    let events = events_of(&state, &Action::ResolveAttack);
    assert_events_in_order(
        &events,
        &[Event::EnemiesDefeated {
            player: SOLO,
            enemies: vec![foe(&state, ORCS_2)],
            fame: 2,
        }],
    );
}

// Spec 031 L4 (amendée le 2026-10-29, demande de Mathieu)
#[test]
fn combat_can_end_once_every_enemy_is_defeated() {
    // MKUE p. 10, *Combat Outcomes* 1 : le combat finit après la phase d'attaque ; sans ennemi
    // restant, les phases suivantes ne font rien : « Terminer le combat » les saute.
    let swiftness = card(Swiftness, 1);
    let state = fight_prowlers_with(&[swiftness]);
    let state = done(&state, declare(&[foe(&state, ORCS_2)]));
    let state = done(&state, strong(swiftness, WHITE_DIE));
    let state = done(&state, Action::ResolveAttack);
    let legal = legal_actions(&state, SOLO);
    assert!(legal.contains(&Action::EndCombat));
    assert!(!legal.contains(&Action::NextCombatPhase));
    let state = done(&state, Action::EndCombat);
    assert!(state.combat(SOLO).is_some_and(|combat| combat.ended));
}

// Spec 031 C20
#[test]
fn ranged_attack_counts_in_attack_phase() {
    // MKUE p. 10, *Attack Phase* 1.a : mêlée et distance comptent ensemble.
    let swiftness = card(Swiftness, 1);
    let state = to_attack_phase(&fight_prowlers_with(&[swiftness]));
    let state = done(&state, declare(&[foe(&state, ORCS_2)]));
    let state = done(&state, strong(swiftness, WHITE_DIE));
    assert_events_in_order(
        &events_of(&state, &Action::ResolveAttack),
        &[Event::EnemiesDefeated {
            player: SOLO,
            enemies: vec![foe(&state, ORCS_2)],
            fame: 2,
        }],
    );
}

// Spec 031 L6
#[test]
fn boosted_attack_adds_its_bonus_to_the_attack() {
    // MKUE p. 5, « Composite » 1.b ; spec 025 : Concentration renforcée ajoute son bonus à
    // l'attaque en cours.
    let (concentration, rage) = (card(Concentration, 1), card(Rage, 1));
    let state = to_attack_phase(&fight_prowlers_with(&[concentration, rage]));
    // Sans attaque en cours, Concentration ne peut pas renforcer une attaque.
    assert!(legal_actions(&state, SOLO).iter().all(|action| !matches!(
        action,
        Action::PlayCard {
            play: CardPlay::Strong { with: Some(_), .. },
            ..
        }
    )));
    let state = done(&state, declare(&[foe(&state, ORCS_2)]));
    let boost = Action::PlayCard {
        card: concentration,
        play: CardPlay::Strong {
            gain: None,
            extra_mana: None,
            option: 0,
            discard: None,
            mana: ManaPayment::Die(GREEN_DIE),
            set_dice: [None, None],
            with: Some(BoostTarget {
                card: rage,
                option: 0,
                discard: None,
                set_dice: [None, None],
            }),
        },
    };
    let state = done(&state, boost);
    // Rage renforcée (4) + 2.
    assert_eq!(
        state
            .combat(SOLO)
            .and_then(|combat| combat.in_progress.clone()),
        Some(InProgress::Attack {
            targets: vec![foe(&state, ORCS_2)],
            points: physical(6),
        })
    );
}

// Spec 031 L1 (attaques)
#[test]
fn attack_actions_are_refused_out_of_their_moment() {
    // Valider sans attaque en cours, cibler pendant une attaque, phase suivante avec une attaque
    // en cours, cibler hors des phases d'attaque.
    let state = fight_prowlers_with(&arena_hand());
    refused(&state, Action::ResolveAttack);
    let declared = done(&state, declare(&[foe(&state, ORCS_2)]));
    refused(&declared, declare(&[foe(&state, ORCS_2)]));
    refused(&declared, Action::NextCombatPhase);
    let block = done(&state, Action::NextCombatPhase);
    refused(&block, declare(&[foe(&state, ORCS_2)]));
    refused(&block, Action::ResolveAttack);
}

// Spec 031 C21
#[test]
fn undefeated_rampaging_enemy_stays() {
    // MKUE p. 10, *Combat Outcomes* 3.a : un ennemi errant non vaincu reste sur sa case, face
    // visible ; le combat fini, le tour peut finir.
    let state = fight_to_the_end(&fighting_prowlers());
    assert!(state.combat(SOLO).is_some_and(|combat| combat.ended));
    assert_eq!(state.enemy_tokens_at(ORCS_2), [prowlers()]);
    assert!(state.enemy_discard(EnemyColor::Green).is_empty());
    assert_eq!(state.fame(SOLO), 0);
    let ends = end_turns(&state);
    assert!(!ends.is_empty());
    let state = done(&state, ends[0].clone());
    assert!(state.combat(SOLO).is_none());
    assert_eq!(state.enemy_tokens_at(ORCS_2), [prowlers()]);
}

// Spec 031 C22
#[test]
fn defeated_enemy_space_is_empty() {
    // MKUE p. 10, *Combat Outcomes* 2.a : la case de l'ennemi vaincu redevient une case vide. Au
    // tour suivant on peut y entrer (colline : 3 points), et longer la case ne provoque plus.
    let swiftness = card(Swiftness, 1);
    let state = fight_prowlers_with(&[swiftness]);
    let state = done(&state, declare(&[foe(&state, ORCS_2)]));
    let state = done(&state, strong(swiftness, WHITE_DIE));
    let state = done(&state, Action::ResolveAttack);
    // L4 (amendée le 2026-10-29) : plus d'ennemi, le combat se termine sans les autres phases.
    let state = done(&state, Action::EndCombat);
    assert!(!state.enemy_tokens().contains_key(&ORCS_2));
    let ends = end_turns(&state);
    assert!(!ends.is_empty());
    let mut state = done(&state, ends[0].clone());
    for _ in 0..3 {
        let card = state.hand(SOLO)[0];
        state = done(&state, sideways(card));
    }
    assert_eq!(state.move_points(SOLO), 3);
    assert!(legal_actions(&state, SOLO).contains(&Action::MoveHero { to: ORCS_2 }));
    // Le pas du village à la plaine voisine aurait provoqué les Rôdeurs (MKUE p. 7, 4.c).
    let step = events_of(&state, &Action::MoveHero { to: PLAIN_W });
    assert!(
        step.iter()
            .all(|event| !matches!(event, Event::EnemiesProvoked { .. }))
    );
}

// --- Provisoire jusqu'aux specs 032 et 033 ---

// Spec 031 C24 (Invocateurs qui n'attaquent pas) : remplacé par la spec 033, C14 à C18.

// Spec 031 C25 (éléments et résistances ignorés) : remplacé par la spec 032, C1 et C6.

// --- Piles, vue, journal, retour en arrière ---

/// Pile verte réduite aux Creuseurs et aux Rôdeurs, posés sur leurs cases, puis vaincus : la pile
/// verte est vide et sa défausse a ces deux jetons (dans cet ordre). Une troisième tuile, Royaume 5
/// (deux orques), attend dans la pioche ; la figurine est au village avec 4 points de mouvement.
fn empty_green_pile() -> GameState {
    let mut events = arena_events(
        VILLAGE,
        &arena_hand(),
        &printed_tokens(),
        &[TileId::Core(5)],
    );
    for event in &mut events {
        if let Event::EnemyPileShuffled {
            color: EnemyColor::Green,
            order,
        } = event
        {
            *order = vec![diggers(), prowlers()];
        }
    }
    events.push(Event::EnemiesDefeated {
        player: SOLO,
        enemies: vec![diggers(), prowlers()],
        fame: 4,
    });
    let state = events.iter().fold(GameState::default(), apply);
    let state = done(&state, basic(card(March, 1)));
    done(&state, basic(card(March, 2)))
}

// Spec 031 C30
#[test]
fn empty_pile_reshuffles_its_discard() {
    // MKUE p. 3, *Enemy and Ruin token piles* : la pile verte est vide, on mélange sa défausse
    // avec le générateur de la pile (spec 038), puis on tire le jeton.
    let state = empty_green_pile();
    assert!(state.enemy_pile(EnemyColor::Green).is_empty());
    assert_eq!(
        state.enemy_discard(EnemyColor::Green),
        [diggers(), prowlers()]
    );
    let mut order = state.enemy_discard(EnemyColor::Green).to_vec();
    order.shuffle(config(0).rngs().rng(RngUsage::EnemyPile(EnemyColor::Green)));

    let explore = Action::Explore {
        position: TilePosition::from_lattice(1, 1),
    };
    let events = events_of(&state, &explore);
    assert!(matches!(events[0], Event::Explored { .. }));
    assert!(matches!(events[1], Event::TileRevealed { .. }));
    // Royaume 5 en (1,1) : orques à l'E puis au S-O (spec 030, C8) ; spec 062 : puis
    // la cité verte et ses défenseurs.
    let reshuffled = Event::EnemyPileReshuffled {
        color: EnemyColor::Green,
        order: order.clone(),
    };
    assert_eq!(
        events[2..5],
        [
            reshuffled.clone(),
            Event::EnemyTokenPlaced {
                token: order[0],
                hex: Hex::new(5, -5),
            },
            Event::EnemyTokenPlaced {
                token: order[1],
                hex: Hex::new(3, -4),
            },
        ]
    );
    // Le hasard est un point de non-retour (spec 024).
    assert!(reshuffled.blocks_undo());
    let after = events.iter().fold(state, apply);
    assert!(after.enemy_pile(EnemyColor::Green).is_empty());
    assert!(after.enemy_discard(EnemyColor::Green).is_empty());
    assert_eq!(after.enemy_tokens_at(Hex::new(5, -5)), [order[0]]);
}

// Spec 031 C30 (cas limite, spec 030 L5), changé par la spec 033 (C20 ter, Q2)
#[test]
fn revealed_tile_draws_from_another_pile_when_its_own_runs_out() {
    // Spec 033, Q2 : pile et défausse vertes vides, chaque jeton vient d'une autre pile, choisie
    // au hasard avec le générateur de la pile verte (avant la spec 033 : rien n'était posé).
    let mut state = empty_green_pile();
    state = apply(
        state,
        &Event::EnemyPileReshuffled {
            color: EnemyColor::Green,
            order: Vec::new(),
        },
    );
    assert!(state.enemy_discard(EnemyColor::Green).is_empty());
    let explore = Action::Explore {
        position: TilePosition::from_lattice(1, 1),
    };
    let events = events_of(&state, &explore);
    assert_eq!(
        events,
        events_of(&state, &explore),
        "même graine, même tirage"
    );
    let placed: Vec<EnemyToken> = events
        .iter()
        .filter_map(|event| match event {
            Event::EnemyTokenPlaced { token, .. } => Some(*token),
            _ => None,
        })
        .collect();
    assert_eq!(placed.len(), 2);
    assert!(
        placed
            .iter()
            .all(|token| token.color() != EnemyColor::Green)
    );
    let after = events.iter().fold(state, apply);
    for token in placed {
        assert!(
            after
                .enemy_tokens()
                .values()
                .flatten()
                .any(|placed| *placed == token)
        );
    }
}

// Spec 031 C31
#[test]
fn view_shows_combat_and_fame() {
    // La vue montre le combat entier, la gloire et les défausses d'ennemis ; de la pile, seulement
    // le nombre de jetons (information cachée).
    let swiftness = card(Swiftness, 1);
    let state = fight_prowlers_with(&[swiftness]);
    let seen = view(&state, SOLO);
    assert_eq!(seen.combat.as_ref(), state.combat(SOLO));
    assert_eq!(seen.fame, 0);
    let state = done(&state, declare(&[foe(&state, ORCS_2)]));
    let state = done(&state, strong(swiftness, WHITE_DIE));
    let seen = view(&state, SOLO);
    let combat = seen.combat.expect("le combat est public");
    assert_eq!(combat.phase, CombatPhase::RangedAndSiege);
    assert_eq!(combat.enemies[0].token, prowlers());
    assert!(matches!(
        combat.in_progress,
        Some(InProgress::Attack { ref targets, points }) if *targets == [prowlers()] && points.physical > 0
    ));
    let piles = view(&state, SOLO).enemy_piles;

    let state = done(&state, Action::ResolveAttack);
    let seen = view(&state, SOLO);
    assert_eq!(seen.fame, 2);
    assert_eq!(
        seen.enemy_discards.get(&EnemyColor::Green),
        Some(&vec![prowlers()])
    );
    assert!(seen.combat.is_some_and(|combat| combat.enemies[0].defeated));
    assert_eq!(seen.enemy_piles, piles);
    // Hors combat, rien à montrer.
    let seen = view(&arena(VILLAGE, &arena_hand(), &printed_tokens()), SOLO);
    assert_eq!(seen.combat, None);
    assert!(seen.enemy_discards.values().all(Vec::is_empty));
}

// Spec 031 C32
#[test]
fn filtered_log_hides_reshuffled_order() {
    // Seul le nombre de jetons du remélange se voit ; les événements du combat sont publics.
    let reshuffled = Event::EnemyPileReshuffled {
        color: EnemyColor::Green,
        order: vec![diggers(), prowlers()],
    };
    assert_eq!(
        filter_event(&reshuffled, SOLO),
        PlayerEvent::EnemyPileReshuffled {
            color: EnemyColor::Green,
            count: 2,
        }
    );
    assert_eq!(
        filter_event(&reshuffled, PlayerId(1)),
        filter_event(&reshuffled, SOLO)
    );
    let started = Event::CombatStarted {
        player: SOLO,
        enemies: vec![(ORCS_2, prowlers())],
    };
    assert_eq!(
        filter_event(&started, PlayerId(1)),
        PlayerEvent::CombatStarted {
            player: SOLO,
            enemies: vec![(ORCS_2, prowlers())],
        }
    );
    let defeated = Event::EnemiesDefeated {
        player: SOLO,
        enemies: vec![prowlers()],
        fame: 2,
    };
    assert_eq!(
        filter_event(&defeated, PlayerId(1)),
        PlayerEvent::EnemiesDefeated {
            player: SOLO,
            enemies: vec![prowlers()],
            fame: 2,
        }
    );
    let wounds = Event::WoundsTaken {
        player: SOLO,
        enemy: prowlers(),
        wounds: vec![wound(1), wound(2)],
    };
    assert_eq!(
        filter_event(&wounds, PlayerId(1)),
        PlayerEvent::WoundsTaken {
            player: SOLO,
            enemy: prowlers(),
            wounds: vec![wound(1), wound(2)],
        }
    );
}

/// Joueur piloté par le test : il répond l'index de l'action voulue parmi les actions légales.
/// Un combat entier contre les Rôdeurs : défi, engagement, les phases, une attaque victorieuse en
/// phase d'attaque, fin du combat.
fn whole_combat(swiftness: CardId) -> Vec<Action> {
    vec![
        challenge(ORCS_2),
        Action::StartCombat,
        Action::NextCombatPhase,
        Action::NextCombatPhase,
        declare(&[prowlers()]),
        strong(swiftness, WHITE_DIE),
        Action::ResolveAttack,
        Action::EndCombat,
    ]
}

fn combat_runner() -> (GameRunner, Puppet, CardId) {
    let swiftness = card(Swiftness, 1);
    let events = arena_events(VILLAGE, &[swiftness], &[(prowlers(), ORCS_2)], &[]);
    let (runner, puppet) = runner(events);
    (runner, puppet, swiftness)
}

// Spec 031 C33
#[test]
fn replaying_log_rebuilds_combat() {
    let (mut runner, puppet, swiftness) = combat_runner();
    for action in whole_combat(swiftness) {
        play(&mut runner, &puppet, action);
    }
    let state = runner.state();
    assert!(state.combat(SOLO).is_some_and(|combat| combat.ended));
    assert_eq!(state.fame(SOLO), 2);
    assert_eq!(state.enemy_discard(EnemyColor::Green), [prowlers()]);

    let replayed = runner
        .log()
        .entries()
        .iter()
        .fold(GameState::default(), |state, entry| {
            apply(state, &entry.event)
        });
    assert_eq!(&replayed, runner.state());
}

// Spec 031 C34
#[test]
fn whole_combat_can_be_taken_back() {
    // MKUE p. 5, *Reverting* 1 (Q9) : aucun événement du combat n'est un point de non-retour ;
    // chaque décision se reprend, de la fin du combat jusqu'à avant le défi.
    let (mut runner, puppet, swiftness) = combat_runner();
    let mut before = Vec::new();
    for action in whole_combat(swiftness) {
        before.push(runner.state().clone());
        play(&mut runner, &puppet, action);
    }
    for (index, state) in before.iter().enumerate().rev() {
        assert_eq!(
            runner.undo_status(),
            UndoStatus::Possible,
            "décision {index}"
        );
        assert_eq!(runner.undo(), Ok(()), "décision {index}");
        assert_eq!(runner.state(), state, "décision {index}");
    }
    assert!(runner.state().combat(SOLO).is_none());
    assert!(runner.state().enemies_to_fight(SOLO).is_empty());
    assert_eq!(runner.state().enemy_tokens_at(ORCS_2), [prowlers()]);
    assert_eq!(runner.undo(), Err(UndoError::NothingSinceTurnStart));
}

// Spec 031 C36 bis, côté moteur : l'historique du combat, que l'écran range par ennemi.
#[test]
fn combat_history_records_attacks_blocks_and_damage() {
    // MKUE p. 9, *Block Phase* 8 et *Assign Damage Phase* 2 ; p. 9, *Ranged and Siege Attack Phase* 6.
    let state = done(&fighting_prowlers(), Action::NextCombatPhase);
    let state = done(&state, declare_block(foe(&state, ORCS_2)));
    let state = done(&state, option(card(Rage, 1), 1));
    let state = done(&state, Action::ResolveBlock);
    let state = done(&state, Action::NextCombatPhase);
    let state = done(&state, declare(&[foe(&state, ORCS_2)]));
    let state = done(&state, sideways_attack(card(March, 1)));
    let state = done(&state, Action::ResolveAttack);
    let history = state
        .combat(SOLO)
        .map(|combat| combat.history.clone())
        .unwrap_or_default();
    assert_eq!(
        history,
        vec![
            CombatRecord::Block {
                target: foe(&state, ORCS_2),
                cards: vec![card(Rage, 1)],
                units: vec![],
                points: 2,
                needed: 4,
                success: false,
            },
            CombatRecord::Damage {
                enemy: foe(&state, ORCS_2),
                attacker: prowlers(),
                damage: 4,
                wounds: 2,
                poisoned: 0,
                paralyzed: 0,
            },
            CombatRecord::Attack {
                phase: CombatPhase::Attack,
                targets: vec![foe(&state, ORCS_2)],
                cards: vec![card(March, 1)],
                units: vec![],
                points: 1,
                needed: 3,
                success: false,
            },
        ]
    );
}
