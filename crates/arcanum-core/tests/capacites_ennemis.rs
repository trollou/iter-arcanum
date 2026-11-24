//! Scénarios de la spec 033 (capacités des ennemis).

use arcanum_core::{
    Action, BasicAction, BasicColor, CardId, CardPlay, CombatPhase, CombatRecord, Effect, Enemy,
    EnemyColor, EnemyToken, Event, GameState, ManaPayment, apply, cold_toughness_bonus,
    legal_actions, view,
};

mod arene;
use arene::*;

use BasicAction::*;

/// Le combat engagé contre `enemy` seul, sur les orques ②, défié depuis le village ; `crystals`
/// dans l'inventaire, pour renforcer plusieurs cartes dans le tour (un seul dé par tour).
fn fight_one(enemy: EnemyToken, hand: &[CardId], crystals: &[BasicColor]) -> GameState {
    let state = arena(VILLAGE, hand, &[(enemy, ORCS_2)]);
    let state = with_crystals(state, crystals);
    let state = done(&state, challenge(ORCS_2));
    done(&state, Action::StartCombat)
}

fn with_crystals(state: GameState, crystals: &[BasicColor]) -> GameState {
    crystals.iter().fold(state, |state, &color| {
        apply(
            state,
            &Event::CrystalGained {
                player: SOLO,
                color,
            },
        )
    })
}

/// L'effet renforcé de `card`, option `option`, payé par un cristal.
fn strong_crystal(card: CardId, option: u8) -> Action {
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

/// Polyvalence renforcée : Parade de feu 3 (option 3).
const VERSATILITY_FIRE_BLOCK: u8 = 3;

/// Parade du héros contre l'ennemi des orques ② : `actions` jouées, puis validée.
fn block_with(state: &GameState, actions: &[Action]) -> Vec<Event> {
    let state = done(state, declare_block(foe(state, ORCS_2)));
    let state = actions
        .iter()
        .fold(state, |state, action| done(&state, action.clone()));
    events_of(&state, &Action::ResolveBlock)
}

fn blocked(enemy: EnemyToken) -> Event {
    Event::EnemyBlocked {
        player: SOLO,
        enemy,
    }
}

fn failed_block() -> Event {
    Event::BlockFailed { player: SOLO }
}

fn wounds_in(cards: &[CardId]) -> usize {
    cards.iter().filter(|card| card.card().is_wound()).count()
}

/// Passe la phase de parade sans parer : les dégâts sont assignés, phase d'attaque.
fn take_damage(state: &GameState) -> (GameState, Vec<Event>) {
    let events = events_of(state, &Action::NextCombatPhase);
    (events.iter().fold(state.clone(), apply), events)
}

fn wound_counts(events: &[Event]) -> Vec<usize> {
    events
        .iter()
        .filter_map(|event| match event {
            Event::WoundsTaken { wounds, .. } => Some(wounds.len()),
            _ => None,
        })
        .collect()
}

fn poison_counts(events: &[Event]) -> Vec<usize> {
    events
        .iter()
        .filter_map(|event| match event {
            Event::PoisonWoundsDiscarded { wounds, .. } => Some(wounds.len()),
            _ => None,
        })
        .collect()
}

fn freezers() -> EnemyToken {
    token(Enemy::Freezers, 1)
}

fn cursed_hags() -> EnemyToken {
    token(Enemy::CursedHags, 1)
}

fn ice_dragon() -> EnemyToken {
    token(Enemy::IceDragon, 1)
}

fn high_dragon() -> EnemyToken {
    token(Enemy::HighDragon, 1)
}

fn altem_mages() -> EnemyToken {
    token(Enemy::AltemMages, 1)
}

fn werewolf() -> EnemyToken {
    token(Enemy::Werewolf, 1)
}

fn minotaur() -> EnemyToken {
    token(Enemy::Minotaur, 1)
}

// --- Vif ---

// Spec 033 C1
#[test]
fn swift_enemy_needs_double_block() {
    // MKUE p. 9, *Block Phase* 6.a : Chevaucheurs de loups, attaque 3, vifs : il faut 6.
    let determination = card(Determination, 1);
    let march = card(March, 1);
    let state = to_block_phase(&fight_one(
        wolf_riders(),
        &[determination, march],
        &[BasicColor::Blue],
    ));
    let combat = state.combat(SOLO).expect("combat");
    assert_eq!(combat.enemies[0].block_needed(), Some(6));
    // Parade 5 : ratée.
    assert_eq!(
        block_with(&state, &[strong_crystal(determination, 0)]),
        [failed_block()]
    );
    // Parade 5 + 1 : parés.
    assert_eq!(
        block_with(
            &state,
            &[strong_crystal(determination, 0), sideways_block(march)]
        ),
        [blocked(foe(&state, ORCS_2))]
    );
}

// Spec 033 C2
#[test]
fn swift_doubles_the_target_not_the_block_efficiency() {
    // MKUE p. 9, *Block Phase* 5.b et 6.a : Glaceurs (glace 3, vifs) : il faut 6 ; la parade de
    // feu compte en plein, la parade physique est divisée par deux.
    let versatility = card(BattleVersatility, 1);
    let determination = card(Determination, 1);
    let march = card(March, 1);
    let state = to_block_phase(&fight_one(
        freezers(),
        &[versatility, determination, march],
        &[BasicColor::Red, BasicColor::Blue],
    ));
    let fire_and_five = [
        strong_crystal(versatility, VERSATILITY_FIRE_BLOCK),
        strong_crystal(determination, 0),
    ];
    // 3 + 5 ÷ 2 = 5 < 6.
    assert_eq!(block_with(&state, &fire_and_five), [failed_block()]);
    // 3 + (5 + 1) ÷ 2 = 6.
    let mut all = fire_and_five.to_vec();
    all.push(sideways_block(march));
    assert_eq!(block_with(&state, &all), [blocked(foe(&state, ORCS_2))]);
}

// Spec 033 C3
#[test]
fn swift_does_not_double_damage() {
    // MKUE p. 24, *Swift* : seule la parade est doublée ; 3 points, 2 Blessures.
    let state = to_block_phase(&fight_one(wolf_riders(), &[], &[]));
    let (_, events) = take_damage(&state);
    assert_eq!(wound_counts(&events), [2]);
}

// --- Brutal ---

// Spec 033 C4
#[test]
fn brutal_enemy_deals_double_damage() {
    // MKUE p. 9, *Assign Damage Phase* 2.a ; Compendium, *Combat* S34 : Cogneurs, 4 × 2 = 8
    // points, 4 Blessures.
    let state = to_block_phase(&fight_one(ironclads(), &[], &[]));
    let combat = state.combat(SOLO).expect("combat");
    assert_eq!(combat.enemies[0].damage(), Some(8));
    let (state, events) = take_damage(&state);
    assert_eq!(wound_counts(&events), [4]);
    assert_eq!(wounds_in(state.hand(SOLO)), 4);
}

// Spec 033 C5
#[test]
fn brutal_does_not_change_block() {
    // MKUE p. 24, *Brutal* : on pare l'attaque simple, 4.
    let determination = card(Determination, 1);
    let rage = card(Rage, 1);
    let state = to_block_phase(&fight_one(ironclads(), &[determination, rage], &[]));
    assert_eq!(
        state.combat(SOLO).expect("combat").enemies[0].block_needed(),
        Some(4)
    );
    assert_eq!(
        block_with(&state, &[option(determination, 1), option(rage, 1)]),
        [blocked(foe(&state, ORCS_2))]
    );
}

// Spec 033 C6
#[test]
fn brutal_doubles_elemental_damage() {
    // Dragons supérieurs, feu glacé 6, brutaux : 12 points, 6 Blessures.
    let state = to_block_phase(&fight_one(high_dragon(), &[], &[]));
    let (_, events) = take_damage(&state);
    assert_eq!(wound_counts(&events), [6]);
}

// --- Toxique ---

// Spec 033 C7
#[test]
fn poison_puts_as_many_wounds_in_discard_pile() {
    // MKUE p. 10, *Assign Damage Phase* 6.a : Sorcières maudites, 3 points : 2 Blessures en main,
    // 2 dans la défausse.
    // Une Marche en main, défaussée en fin de tour (aucune carte jouée : il faut en défausser une,
    // spec 016).
    let march = card(March, 1);
    let state = to_block_phase(&fight_one(cursed_hags(), &[march], &[]));
    let (state, events) = take_damage(&state);
    assert_eq!(wound_counts(&events), [2]);
    assert_eq!(poison_counts(&events), [2]);
    assert_eq!(wounds_in(state.hand(SOLO)), 2);
    assert_eq!(wounds_in(state.discard_pile(SOLO)), 2);
    // Les numéros d'exemplaire continuent : 1 et 2 en main, 3 et 4 dans la défausse.
    assert_eq!(state.discard_pile(SOLO), [wound(3), wound(4)]);
    // À la fin du tour, la limite de main ne voit que les 2 Blessures de la main : 3 piochées.
    let state = done(&state, Action::EndCombat);
    let state = done(
        &state,
        Action::EndTurn {
            discard: vec![march],
            reclaim: None,
            glade: None,
            steady_tempo: false,
        },
    );
    assert_eq!(state.hand(SOLO).len(), 5);
    assert_eq!(wounds_in(state.hand(SOLO)), 2);
}

// Spec 033 C8
#[test]
fn poison_counts_brutal_wounds() {
    // MKUE p. 10, 6.a ; p. 9, 2.a : Mages d'Altem, feu glacé 4, brutaux, toxiques : 8 points, 4
    // Blessures en main, 4 dans la défausse.
    let state = to_block_phase(&fight_one(altem_mages(), &[], &[]));
    let (_, events) = take_damage(&state);
    assert_eq!(wound_counts(&events), [4]);
    assert_eq!(poison_counts(&events), [4]);
}

// Spec 033 C9
#[test]
fn blocked_poisonous_enemy_deals_nothing() {
    let determination = card(Determination, 1);
    let state = to_block_phase(&fight_one(
        cursed_hags(),
        &[determination],
        &[BasicColor::Blue],
    ));
    let state = done(&state, declare_block(foe(&state, ORCS_2)));
    let state = done(&state, strong_crystal(determination, 0));
    let state = done(&state, Action::ResolveBlock);
    let (state, events) = take_damage(&state);
    assert!(wound_counts(&events).is_empty());
    assert!(poison_counts(&events).is_empty());
    assert_eq!(wounds_in(state.discard_pile(SOLO)), 0);
}

// --- Paralysant ---

fn three_cards() -> [CardId; 3] {
    [card(March, 1), card(Stamina, 1), card(Swiftness, 1)]
}

// Spec 033 C10
#[test]
fn paralyze_discards_non_wound_cards_from_hand() {
    // MKUE p. 10, *Assign Damage Phase* 6.b : Dragons de glace, glace 6 : 3 Blessures en main, puis
    // les autres cartes de la main vont dans la défausse.
    let hand = three_cards();
    let state = to_block_phase(&fight_one(ice_dragon(), &hand, &[]));
    let (state, events) = take_damage(&state);
    assert_eq!(wound_counts(&events), [3]);
    assert!(events.contains(&Event::HandParalyzed {
        player: SOLO,
        cards: hand.to_vec(),
    }));
    assert_eq!(state.hand(SOLO), [wound(1), wound(2), wound(3)]);
    assert_eq!(state.discard_pile(SOLO), hand);
}

// Spec 033 C11
#[test]
fn blocked_paralyzing_enemy_discards_nothing() {
    let determination = card(Determination, 1);
    let versatility = card(BattleVersatility, 1);
    let march = card(March, 1);
    let state = to_block_phase(&fight_one(
        ice_dragon(),
        &[determination, versatility, march],
        &[BasicColor::Red],
    ));
    // Une parade réussie, construite par événement : aucune carte de la main ne pare 6 de glace.
    let state = done(&state, declare_block(ice_dragon()));
    let state = apply(state, &blocked(ice_dragon()));
    let (state, events) = take_damage(&state);
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::HandParalyzed { .. }))
    );
    assert_eq!(state.hand(SOLO), [determination, versatility, march]);
}

// Spec 033 C11b (issue privée 85) — MKUE p. 6, 5.a et 7 : sans carte jouée, les cartes défaussées par
// l'attaque paralysante comptent pour le tour minimum ; pas de repos après un combat.
#[test]
fn paralyzed_hero_ends_turn_without_discard() {
    let state = to_block_phase(&fight_one(ice_dragon(), &three_cards(), &[]));
    let (state, _) = take_damage(&state);
    let state = done(&state, Action::EndCombat);
    let ends: Vec<Action> = legal_actions(&state, SOLO)
        .into_iter()
        .filter(|action| matches!(action, Action::EndTurn { .. }))
        .collect();
    let without_discard = Action::EndTurn {
        discard: Vec::new(),
        reclaim: None,
        glade: None,
        steady_tempo: false,
    };
    assert_eq!(ends, [without_discard]);
}

// Spec 033 C12
#[test]
fn paralyzed_hero_redraws_at_end_of_turn() {
    // Spec 016 : 3 Blessures en main, limite 5 : 2 cartes piochées. Une Rage jouée dans une parade
    // ratée (sans carte jouée, voir `paralyzed_hero_ends_turn_without_discard`).
    let rage = card(Rage, 1);
    let mut hand = three_cards().to_vec();
    hand.push(rage);
    let state = to_block_phase(&fight_one(ice_dragon(), &hand, &[]));
    let state = done(&state, declare_block(foe(&state, ORCS_2)));
    let state = done(&state, option(rage, 1));
    let state = done(&state, Action::ResolveBlock);
    let (state, _) = take_damage(&state);
    let state = done(&state, Action::EndCombat);
    let state = done(
        &state,
        Action::EndTurn {
            discard: Vec::new(),
            reclaim: None,
            glade: None,
            steady_tempo: false,
        },
    );
    assert_eq!(state.hand(SOLO).len(), 5);
    assert_eq!(wounds_in(state.hand(SOLO)), 3);
}

// Spec 033 C13
#[test]
fn paralyzed_hero_keeps_played_effects() {
    // Compendium, *Combat* Q22 : la carte jouée avant reste jouée ; plus rien en main à jouer.
    let mut hand = three_cards().to_vec();
    let rage = card(Rage, 1);
    hand.push(rage);
    let state = to_block_phase(&fight_one(ice_dragon(), &hand, &[]));
    let state = done(&state, declare_block(foe(&state, ORCS_2)));
    let state = done(&state, option(rage, 1));
    let state = done(&state, Action::ResolveBlock);
    let (state, _) = take_damage(&state);
    assert!(
        state
            .play_area(SOLO)
            .iter()
            .any(|played| played.card == rage)
    );
    let state = done(&state, declare(&[foe(&state, ORCS_2)]));
    assert!(
        legal_actions(&state, SOLO)
            .iter()
            .all(|action| !matches!(action, Action::PlayCard { .. }))
    );
}

// --- Invocation ---

/// Invocateurs orques sur les orques ② ; la pile marron réordonnée : `brown` sur le dessus.
fn fight_summoners(hand: &[CardId], brown: &[EnemyToken]) -> GameState {
    let state = arena(VILLAGE, hand, &[(orc_summoners(), ORCS_2)]);
    let state = with_brown_on_top(state, brown);
    let state = with_crystals(state, &[BasicColor::Blue, BasicColor::White]);
    let state = done(&state, challenge(ORCS_2));
    done(&state, Action::StartCombat)
}

fn with_brown_on_top(state: GameState, brown: &[EnemyToken]) -> GameState {
    let mut order = brown.to_vec();
    order.extend(
        state
            .enemy_pile(EnemyColor::Brown)
            .iter()
            .copied()
            .filter(|token| !brown.contains(token)),
    );
    apply(
        state,
        &Event::EnemyPileShuffled {
            color: EnemyColor::Brown,
            order,
        },
    )
}

fn summoned(state: &GameState) -> Option<EnemyToken> {
    state
        .combat(SOLO)
        .and_then(|combat| combat.enemies[0].summoned)
}

// Spec 033 C14
#[test]
fn summoner_draws_a_brown_token_at_block_phase() {
    // MKUE p. 9, *Block Phase* 2 : le premier jeton de la pile marron.
    let state = fight_summoners(&[], &[minotaur()]);
    let brown = state.enemy_pile(EnemyColor::Brown).len();
    let events = events_of(&state, &Action::NextCombatPhase);
    assert_eq!(
        events,
        [
            Event::CombatPhaseStarted {
                player: SOLO,
                phase: CombatPhase::Block
            },
            Event::EnemySummoned {
                player: SOLO,
                summoner: foe(&state, ORCS_2),
                token: minotaur(),
            },
        ]
    );
    let state = events.iter().fold(state, apply);
    assert_eq!(summoned(&state), Some(minotaur()));
    assert_eq!(state.enemy_pile(EnemyColor::Brown).len(), brown - 1);
    assert_eq!(block_targets(&state), [ORCS_2]);
}

// Spec 033 C15
#[test]
fn defeated_summoner_does_not_summon() {
    // Compendium, *ORC SUMMONERS* S1 : vaincus en phase d'attaque à distance, rien n'est invoqué.
    // Des Rôdeurs aussi, pour que le combat continue après la défaite des Invocateurs.
    let state = arena(
        PLAIN_W,
        &[],
        &[(orc_summoners(), ORCS_2), (prowlers(), HILL)],
    );
    let state = done(&state, challenge(ORCS_2));
    let state = done(&state, challenge(HILL));
    let state = done(&state, Action::StartCombat);
    let state = done(&state, declare(&[orc_summoners()]));
    let state = apply(
        state,
        &Event::EnemiesDefeated {
            player: SOLO,
            enemies: vec![orc_summoners()],
            fame: 4,
        },
    );
    let events = events_of(&state, &Action::NextCombatPhase);
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::EnemySummoned { .. }))
    );
}

// Spec 033 C16
#[test]
fn summoned_monster_is_blocked_with_its_own_abilities() {
    // MKUE p. 9, *Block Phase* 2.a et 7.a : la créature invoquée se pare avec ses propres valeurs.
    // Loup-garou, attaque 7, vif : 14 à parer.
    let state = to_block_phase(&fight_summoners(&[], &[werewolf()]));
    let combat = state.combat(SOLO).expect("combat");
    assert_eq!(combat.enemies[0].block_needed(), Some(14));
    // Minotaure, attaque 5 : Parade 5 le pare ; il va dans la défausse marron, sans gloire, et
    // l'Invocateur est marqué paré.
    let determination = card(Determination, 1);
    let state = to_block_phase(&fight_summoners(&[determination], &[minotaur()]));
    let events = block_with(&state, &[strong_crystal(determination, 0)]);
    assert_eq!(
        events,
        [
            blocked(foe(&state, ORCS_2)),
            Event::SummonedEnemyDiscarded {
                player: SOLO,
                summoner: foe(&state, ORCS_2),
            },
        ]
    );
    let fame = state.fame(SOLO);
    let state = done(&state, declare_block(foe(&state, ORCS_2)));
    let state = done(&state, strong_crystal(determination, 0));
    let state = done(&state, Action::ResolveBlock);
    assert_eq!(state.enemy_discard(EnemyColor::Brown), [minotaur()]);
    assert_eq!(summoned(&state), None);
    assert_eq!(state.fame(SOLO), fame);
    let combat = state.combat(SOLO).expect("combat");
    assert!(combat.enemies[0].blocked);
}

// Spec 033 C17
#[test]
fn summoned_monster_deals_damage_then_is_discarded() {
    // MKUE p. 10, 6.c : Minotaure invoqué, attaque 5, brutal : 10 points, 5 Blessures, puis il
    // va dans la défausse marron.
    let state = to_block_phase(&fight_summoners(&[], &[minotaur()]));
    let (state, events) = take_damage(&state);
    assert_eq!(wound_counts(&events), [5]);
    assert!(events.contains(&Event::SummonedEnemyDiscarded {
        player: SOLO,
        summoner: foe(&state, ORCS_2),
    }));
    assert_eq!(state.enemy_discard(EnemyColor::Brown), [minotaur()]);
    assert_eq!(summoned(&state), None);
}

// Spec 033 C18
#[test]
fn summoner_is_back_in_attack_phase() {
    // MKUE p. 9, *Block Phase* 2.b : l'Invocateur (armure 4, gloire 4) redevient la cible.
    let rage = card(Rage, 1);
    let march = card(March, 1);
    let stamina = card(Stamina, 1);
    // Une Gargouille (3 Blessures) plutôt qu'un Minotaure : ses 5 Blessures assommeraient le
    // héros et défausseraient la main (spec 034).
    let gargoyle = token(Enemy::Gargoyle, 1);
    let state = to_block_phase(&fight_summoners(&[rage, march, stamina], &[gargoyle]));
    let (state, _) = take_damage(&state);
    assert_eq!(attack_targets(&state), [vec![ORCS_2]]);
    let state = done(&state, declare(&[foe(&state, ORCS_2)]));
    let state = done(&state, option(rage, 0));
    let state = done(&state, sideways_attack(march));
    let state = done(&state, sideways_attack(stamina));
    let events = events_of(&state, &Action::ResolveAttack);
    assert_events_in_order(
        &events,
        &[Event::EnemiesDefeated {
            player: SOLO,
            enemies: vec![foe(&state, ORCS_2)],
            fame: 4,
        }],
    );
}

// Spec 033 C19
#[test]
fn each_summoner_draws_its_own_monster() {
    let summoners_2 = token(Enemy::OrcSummoners, 2);
    let state = arena(
        PLAIN_W,
        &[],
        &[(orc_summoners(), ORCS_2), (summoners_2, HILL)],
    );
    let state = with_brown_on_top(state, &[minotaur(), werewolf()]);
    let state = done(&state, challenge(ORCS_2));
    let state = done(&state, challenge(HILL));
    let state = done(&state, Action::StartCombat);
    let summoned: Vec<(EnemyToken, EnemyToken)> = events_of(&state, &Action::NextCombatPhase)
        .into_iter()
        .filter_map(|event| match event {
            Event::EnemySummoned {
                summoner, token, ..
            } => Some((summoner, token)),
            _ => None,
        })
        .collect();
    assert_eq!(
        summoned,
        [(orc_summoners(), minotaur()), (summoners_2, werewolf())]
    );
}

/// Invocateurs orques sur les orques ② ; pile marron vide ; la défausse marron a `discard` (posés
/// sur la colline puis vaincus hors combat, comme la spec 031 vide la pile verte).
fn fight_summoners_without_brown(discard: &[EnemyToken]) -> GameState {
    let mut state = arena(VILLAGE, &[], &[(orc_summoners(), ORCS_2)]);
    for &token in discard {
        state = apply(state, &Event::EnemyTokenPlaced { token, hex: HILL });
        state = apply(
            state,
            &Event::EnemiesDefeated {
                player: SOLO,
                enemies: vec![token],
                fame: 0,
            },
        );
    }
    let state = apply(
        state,
        &Event::EnemyPileShuffled {
            color: EnemyColor::Brown,
            order: Vec::new(),
        },
    );
    let state = done(&state, challenge(ORCS_2));
    done(&state, Action::StartCombat)
}

// Spec 033 C20
#[test]
fn summon_reshuffles_the_brown_discard_pile() {
    // MKUE p. 3, *Enemy and Ruin token piles* : pile marron vide, sa défausse est mélangée pour la
    // refaire, avec le générateur de la pile marron (spec 038).
    let state = fight_summoners_without_brown(&[minotaur()]);
    assert!(state.enemy_pile(EnemyColor::Brown).is_empty());
    assert_eq!(state.enemy_discard(EnemyColor::Brown), [minotaur()]);
    let events = events_of(&state, &Action::NextCombatPhase);
    assert_eq!(
        events[1..],
        [
            Event::EnemyPileReshuffled {
                color: EnemyColor::Brown,
                order: vec![minotaur()],
            },
            Event::EnemySummoned {
                player: SOLO,
                summoner: foe(&state, ORCS_2),
                token: minotaur(),
            },
        ]
    );
}

// Spec 033 C20 bis
#[test]
fn summon_draws_from_another_pile_when_brown_runs_out() {
    // Q2 (Mathieu) ; Compendium, *Scenarios* S5 : pile et défausse marron vides, la créature vient
    // d'une autre pile tirée au hasard (générateur de la pile marron) ; elle repart dans la
    // défausse de sa couleur.
    let state = fight_summoners_without_brown(&[]);
    assert!(state.enemy_discard(EnemyColor::Brown).is_empty());
    let events = events_of(&state, &Action::NextCombatPhase);
    assert_eq!(events, events_of(&state, &Action::NextCombatPhase));
    let token = events
        .iter()
        .find_map(|event| match event {
            Event::EnemySummoned { token, .. } => Some(*token),
            _ => None,
        })
        .expect("une créature est invoquée");
    assert_ne!(token.color(), EnemyColor::Brown);
    assert_eq!(state.enemy_pile(token.color()).first(), Some(&token));
    let state = events.iter().fold(state, apply);
    let (state, _) = take_damage(&state);
    assert_eq!(state.enemy_discard(token.color()).last(), Some(&token));
}

// Spec 033 C21
#[test]
fn summon_is_a_point_of_no_return() {
    // Spec 024 : le tirage révèle un jeton.
    let event = Event::EnemySummoned {
        player: SOLO,
        summoner: orc_summoners(),
        token: minotaur(),
    };
    assert!(event.blocks_undo());
    assert!(
        !Event::SummonedEnemyDiscarded {
            player: SOLO,
            summoner: orc_summoners(),
        }
        .blocks_undo()
    );
    assert!(
        !Event::HandParalyzed {
            player: SOLO,
            cards: Vec::new(),
        }
        .blocks_undo()
    );
    assert!(
        !Event::PoisonWoundsDiscarded {
            player: SOLO,
            enemy: orc_summoners(),
            wounds: Vec::new(),
        }
        .blocks_undo()
    );
}

// --- Endurance givrée renforcée ---

// Spec 033 C22
#[test]
fn cold_toughness_bonus_per_enemy() {
    // Compendium, *Cold Toughness* S1 : +1 par capacité, par résistance, par couleur d'attaque
    // (feu glacé : 2) ; l'invocation ne compte pas.
    for (enemy, bonus) in [
        (Enemy::Prowlers, 0),
        (Enemy::Diggers, 1),
        (Enemy::WolfRiders, 1),
        (Enemy::Ironclads, 2),
        (Enemy::OrcSummoners, 0),
        (Enemy::FireDragon, 3),
        (Enemy::IceDragon, 4),
        (Enemy::HighDragon, 5),
        (Enemy::SwampDragon, 2),
        (Enemy::Freezers, 5),
        (Enemy::AltemGuardsmen, 4),
        (Enemy::AltemMages, 5),
    ] {
        assert_eq!(cold_toughness_bonus(enemy), bonus, "{enemy:?}");
    }
}

/// Endurance givrée renforcée (option 0), payée par un cristal bleu.
fn cold_toughness_strong(card: CardId) -> Action {
    strong_crystal(card, 0)
}

// Spec 033 C23
#[test]
fn cold_toughness_counts_the_blocked_enemy() {
    // Chevaucheurs de loups (vifs) : Parade de glace 5 + 1 = 6, il faut 3 × 2 = 6.
    let cold_toughness = card(ColdToughness, 1);
    let state = to_block_phase(&fight_one(
        wolf_riders(),
        &[cold_toughness],
        &[BasicColor::Blue],
    ));
    let state = done(&state, declare_block(foe(&state, ORCS_2)));
    assert!(playable_effects(&state).contains(&Effect::IceBlockPerAbility(5)));
    let state = done(&state, cold_toughness_strong(cold_toughness));
    let combat = state.combat(SOLO).expect("combat");
    assert_eq!(combat.in_progress_total(), Some(6));
    assert_eq!(
        events_of(&state, &Action::ResolveBlock),
        [blocked(foe(&state, ORCS_2))]
    );
}

// Spec 033 C24
#[test]
fn cold_toughness_counts_the_summoned_monster() {
    // Compendium, *Cold Toughness* O4 : contre un Minotaure invoqué (brutal), Parade de glace 6.
    let cold_toughness = card(ColdToughness, 1);
    let state = to_block_phase(&fight_summoners(&[cold_toughness], &[minotaur()]));
    let state = done(&state, declare_block(foe(&state, ORCS_2)));
    let state = done(&state, cold_toughness_strong(cold_toughness));
    assert_eq!(
        state
            .combat(SOLO)
            .and_then(|combat| combat.in_progress_total()),
        Some(6)
    );
}

// Spec 033 C25
#[test]
fn cold_toughness_needs_a_block_in_progress() {
    let cold_toughness = card(ColdToughness, 1);
    let state = to_block_phase(&fight_one(
        wolf_riders(),
        &[cold_toughness],
        &[BasicColor::Blue],
    ));
    assert!(!playable_effects(&state).contains(&Effect::IceBlockPerAbility(5)));
    // Contre une attaque de glace, elle est divisée par deux (spec 032) : Dragons de glace, 9 ÷ 2.
    let state = to_block_phase(&fight_one(
        ice_dragon(),
        &[cold_toughness],
        &[BasicColor::Blue],
    ));
    let state = done(&state, declare_block(foe(&state, ORCS_2)));
    let state = done(&state, cold_toughness_strong(cold_toughness));
    assert_eq!(
        state
            .combat(SOLO)
            .and_then(|combat| combat.in_progress_total()),
        Some(4)
    );
}

// --- Vue, journal ---

// Spec 033 C26
#[test]
fn view_shows_block_needed_and_damage_if_unblocked() {
    let state = to_block_phase(&fight_summoners(&[], &[werewolf()]));
    let seen = view(&state, SOLO);
    let combat = seen.combat.expect("combat public");
    let summoner = &combat.enemies[0];
    assert_eq!(summoner.summoned, Some(werewolf()));
    assert_eq!(summoner.block_needed(), Some(14));
    assert_eq!(summoner.damage(), Some(7));
    assert_eq!(summoner.wounds_if_unblocked(seen.armor), Some(4));
    let state = to_block_phase(&fight_one(ironclads(), &[], &[]));
    let ironclads = &state.combat(SOLO).expect("combat").enemies[0];
    assert_eq!(ironclads.damage(), Some(8));
    assert_eq!(ironclads.wounds_if_unblocked(state.armor(SOLO)), Some(4));
}

// Spec 033 C27
#[test]
fn replaying_combat_with_abilities_rebuilds_state() {
    // Spec 024 : rejouer le journal redonne le même état ; l'historique du combat dit les effets
    // des capacités.
    let state = to_block_phase(&fight_summoners(&three_cards(), &[minotaur()]));
    let (state, events) = take_damage(&state);
    let replayed = events.iter().fold(
        to_block_phase(&fight_summoners(&three_cards(), &[minotaur()])),
        apply,
    );
    assert_eq!(replayed, state);
    let history = &state.combat(SOLO).expect("combat").history;
    assert!(history.iter().any(|record| matches!(
        record,
        CombatRecord::Damage {
            attacker,
            damage: 10,
            wounds: 5,
            ..
        } if *attacker == minotaur()
    )));
}
