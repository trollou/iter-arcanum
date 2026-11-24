//! Scénarios de la spec 032 (éléments et résistances).

use arcanum_core::{
    Action, AttackKind, BasicAction, BasicColor, BoostTarget, CardId, CardPlay, CombatRecord,
    Effect, Efficiency, Element, ElementPoints, Enemy, EnemyAttack, EnemyToken, Event, GameState,
    InProgress, ManaPayment, Resistance, apply, attack_efficiency, block_efficiency, legal_actions,
    view,
};

mod arene;
use arene::*;

use BasicAction::*;

/// Le combat engagé contre `enemy` seul, sur les orques ②, défié depuis le village ; `crystals`
/// dans l'inventaire, pour renforcer plusieurs cartes dans le tour (un seul dé par tour).
fn fight_one(enemy: EnemyToken, hand: &[CardId], crystals: &[BasicColor]) -> GameState {
    let state = arena(VILLAGE, hand, &[(enemy, ORCS_2)]);
    let state = crystals.iter().fold(state, |state, &color| {
        apply(
            state,
            &Event::CrystalGained {
                player: SOLO,
                color,
            },
        )
    });
    let state = done(&state, challenge(ORCS_2));
    done(&state, Action::StartCombat)
}

/// Options de la Polyvalence renforcée, dans l'ordre de la carte.
const VERSATILITY_ATTACK: u8 = 0;
const VERSATILITY_FIRE_ATTACK: u8 = 2;
const VERSATILITY_FIRE_BLOCK: u8 = 3;
const VERSATILITY_RANGED: u8 = 4;
const VERSATILITY_SIEGE: u8 = 5;

/// Le total qui compte de l'attaque ou de la parade en cours.
fn total(state: &GameState) -> Option<u32> {
    state
        .combat(SOLO)
        .and_then(|combat| combat.in_progress_total())
}

fn blocked(enemy: EnemyToken) -> Vec<Event> {
    vec![Event::EnemyBlocked {
        player: SOLO,
        enemy,
    }]
}

fn failed_block() -> Vec<Event> {
    vec![Event::BlockFailed { player: SOLO }]
}

fn defeated(enemies: &[EnemyToken], fame: u32) -> Event {
    Event::EnemiesDefeated {
        player: SOLO,
        enemies: enemies.to_vec(),
        fame,
    }
}

// --- Contenu ---

// Spec 032 C22 (moteur)
#[test]
fn battle_versatility_strong_has_six_options() {
    // `../mage-knight/docs/regles/cartes/actions-de-base.md` : Attack 4, Block 4, Fire Attack 3, Fire Block 3,
    // Ranged Attack 3, or Siege Attack 2.
    let versatility = card(BattleVersatility, 1);
    let effect = |option| {
        versatility.effect(CardPlay::Strong {
            gain: None,
            extra_mana: None,
            option,
            discard: None,
            mana: ManaPayment::Crystal,
            set_dice: [None, None],
            with: None,
        })
    };
    let attack = |amount, element, kind| {
        Some(Effect::Attack {
            amount,
            element,
            kind,
        })
    };
    assert_eq!(effect(0), attack(4, Element::Physical, AttackKind::Melee));
    assert_eq!(
        effect(1),
        Some(Effect::Block {
            amount: 4,
            element: Element::Physical
        })
    );
    assert_eq!(effect(2), attack(3, Element::Fire, AttackKind::Melee));
    assert_eq!(
        effect(3),
        Some(Effect::Block {
            amount: 3,
            element: Element::Fire
        })
    );
    assert_eq!(effect(4), attack(3, Element::Physical, AttackKind::Ranged));
    assert_eq!(effect(5), attack(2, Element::Physical, AttackKind::Siege));
    assert_eq!(effect(6), None);
}

// --- Parade ---

// Spec 032 C1
#[test]
fn any_block_is_efficient_against_physical() {
    // MKUE p. 9, *Block Phase* 5.a : toute parade est efficace contre une attaque physique.
    let cold_toughness = card(ColdToughness, 1);
    let state = fight_one(prowlers(), &[cold_toughness, card(March, 1)], &[]);
    let state = done(&to_block_phase(&state), declare_block(foe(&state, ORCS_2)));
    let state = done(&state, option(cold_toughness, 1));
    let state = done(&state, sideways_block(card(March, 1)));
    assert_eq!(total(&state), Some(4));
    assert_eq!(
        events_of(&state, &Action::ResolveBlock),
        blocked(foe(&state, ORCS_2))
    );
}

// Spec 032 C2
#[test]
fn fire_attack_needs_ice_or_cold_fire_block() {
    // MKUE p. 9, *Block Phase* 5.a et 5.b : contre le feu, seule la glace (ou le feu glacé) pare en
    // plein ; la Parade 5 et la Parade de feu 3 sont divisées ensemble.
    let (cold_toughness, determination, versatility) = (
        card(ColdToughness, 1),
        card(Determination, 1),
        card(BattleVersatility, 1),
    );
    let state = fight_one(
        fire_dragon(),
        &[cold_toughness, determination, versatility],
        &[BasicColor::Blue, BasicColor::Red],
    );
    let state = done(&to_block_phase(&state), declare_block(foe(&state, ORCS_2)));
    let state = done(&state, option(cold_toughness, 1));
    let state = done(&state, strong_crystal(determination, 0));
    let state = done(&state, strong_crystal(versatility, VERSATILITY_FIRE_BLOCK));
    // 3 + (5 + 3) ÷ 2 = 7 < 9.
    assert_eq!(total(&state), Some(7));
    assert_eq!(events_of(&state, &Action::ResolveBlock), failed_block());
}

// Spec 032 C3
#[test]
fn ice_attack_needs_fire_or_cold_fire_block() {
    // MKUE p. 9, *Block Phase* 5.a : le feu pare la glace en plein ; l'exemple de la spec.
    let (versatility, determination, march) = (
        card(BattleVersatility, 1),
        card(Determination, 1),
        card(March, 1),
    );
    let hand = [versatility, determination, march];
    let colors = [BasicColor::Red, BasicColor::Blue];
    let start = done(
        &to_block_phase(&fight_one(ice_dragon(), &hand, &colors)),
        declare_block(ice_dragon()),
    );
    let state = done(&start, strong_crystal(versatility, VERSATILITY_FIRE_BLOCK));
    let state = done(&state, strong_crystal(determination, 0));
    // Sans Marche : 3 + 5 ÷ 2 = 5 < 6.
    assert_eq!(total(&state), Some(5));
    assert_eq!(events_of(&state, &Action::ResolveBlock), failed_block());
    // Avec Marche de côté : 3 + (5 + 1) ÷ 2 = 6.
    let state = done(&state, sideways_block(march));
    assert_eq!(total(&state), Some(6));
    assert_eq!(
        events_of(&state, &Action::ResolveBlock),
        blocked(foe(&state, ORCS_2))
    );
}

// Spec 032 C4
#[test]
fn cold_fire_attack_needs_cold_fire_block() {
    // MKUE p. 9, *Block Phase* 5.a : contre le feu glacé, ni le feu ni la glace ne parent en plein.
    let (versatility, cold_toughness) = (card(BattleVersatility, 1), card(ColdToughness, 1));
    let state = fight_one(
        high_dragon(),
        &[versatility, cold_toughness],
        &[BasicColor::Red],
    );
    let state = done(&to_block_phase(&state), declare_block(foe(&state, ORCS_2)));
    let state = done(&state, strong_crystal(versatility, VERSATILITY_FIRE_BLOCK));
    let state = done(&state, option(cold_toughness, 1));
    assert_eq!(total(&state), Some(3));
}

// Spec 032 C5
#[test]
fn inefficient_blocks_are_halved_together() {
    // MKUE p. 9, *Block Phase* 5.b : une seule division sur la somme des parades inefficaces.
    let state = fight_one(fire_dragon(), &[card(March, 1), card(March, 2)], &[]);
    let state = done(&to_block_phase(&state), declare_block(foe(&state, ORCS_2)));
    let state = done(&state, sideways_block(card(March, 1)));
    assert_eq!(total(&state), Some(0));
    let state = done(&state, sideways_block(card(March, 2)));
    assert_eq!(total(&state), Some(1));
}

// --- Attaque ---

// Spec 032 C6
#[test]
fn physical_resistance_halves_physical_attacks() {
    // MKUE p. 9, *Ranged and Siege Attack Phase* 4.a ; p. 24 : la résistance physique divise les
    // attaques physiques, cartes de côté comprises.
    let (rage, versatility) = (card(Rage, 1), card(BattleVersatility, 1));
    let state = fight_one(ironclads(), &[rage, versatility, card(March, 1)], &[]);
    let state = done(&to_attack_phase(&state), declare(&[foe(&state, ORCS_2)]));
    let state = done(&state, option(rage, 0));
    let state = done(&state, sideways_attack(card(March, 1)));
    // (2 + 1) ÷ 2 = 1.
    assert_eq!(total(&state), Some(1));
    // L'Attaque de feu 3 compte en plein : 1 + 3 = 4 ≥ 3.
    let state = done(
        &state,
        strong_with_die(versatility, VERSATILITY_FIRE_ATTACK, RED_DIE),
    );
    assert_eq!(total(&state), Some(4));
    assert_events_in_order(
        &events_of(&state, &Action::ResolveAttack),
        &[defeated(&[foe(&state, ORCS_2)], 4)],
    );
}

// Spec 032 C7
#[test]
fn group_shares_resistances() {
    // MKUE p. 9, 4.a et 7.b ; Compendium, *Combat* S12 : la résistance des Cogneurs divise les
    // attaques physiques contre tout le groupe.
    let (rage, versatility, march) = (card(Rage, 1), card(BattleVersatility, 1), card(March, 1));
    let placed = [(prowlers(), ORCS_2), (ironclads(), HILL)];
    let state = fight(
        PLAIN_W,
        &[rage, versatility, march],
        &placed,
        &[ORCS_2, HILL],
    );
    // Sans dégâts : les 6 Blessures assommeraient le héros (spec 034).
    let state = skip_to_attack_phase(&state);
    let group = done(&state, declare(&[foe(&state, ORCS_2), foe(&state, HILL)]));
    let group = done(&group, option(rage, 0));
    let group = done(&group, sideways_attack(march));
    let group = done(
        &group,
        strong_with_die(versatility, VERSATILITY_FIRE_ATTACK, RED_DIE),
    );
    assert_eq!(total(&group), Some(4));
    assert_eq!(
        events_of(&group, &Action::ResolveAttack),
        vec![Event::AttackFailed { player: SOLO }]
    );
    // Les Rôdeurs seuls : 3 / 3.
    let alone = done(&state, declare(&[foe(&state, ORCS_2)]));
    let alone = done(&alone, option(rage, 0));
    let alone = done(&alone, sideways_attack(march));
    assert_eq!(total(&alone), Some(3));
    assert_events_in_order(
        &events_of(&alone, &Action::ResolveAttack),
        &[defeated(&[foe(&state, ORCS_2)], 2)],
    );
}

fn strong_with_die(card: CardId, option: u8, die: u8) -> Action {
    Action::PlayCard {
        card,
        play: CardPlay::Strong {
            gain: None,
            extra_mana: None,
            option,
            discard: None,
            mana: ManaPayment::Die(die),
            set_dice: [None, None],
            with: None,
        },
    }
}

// Spec 032 C8
#[test]
fn resistances_halve_all_matching_attacks_together() {
    // MKUE p. 9, 4.a : contre les Dragons de feu (feu et physique), l'Attaque de feu 3 et l'Attaque 4
    // sont divisées ensemble : (3 + 4) ÷ 2 = 3.
    let (rage, versatility) = (card(Rage, 1), card(BattleVersatility, 1));
    let state = fight_one(fire_dragon(), &[rage, versatility], &[BasicColor::Red]);
    // Sans dégâts : les Blessures du dragon assommeraient le héros (spec 034).
    let state = done(
        &skip_to_attack_phase(&state),
        declare(&[foe(&state, ORCS_2)]),
    );
    let state = done(&state, strong_crystal(versatility, VERSATILITY_FIRE_ATTACK));
    let state = done(&state, strong(rage, RED_DIE));
    assert_eq!(total(&state), Some(3));
}

// Spec 032 C9
#[test]
fn cold_fire_attack_halved_only_by_both_resistances_on_one_enemy() {
    // MKUE p. 9, 4.a, puce : le feu glacé n'est divisé que si un même ennemi résiste au feu et à la
    // glace. Aucune carte n'en donne : le calcul seul.
    let resistances = |enemy: Enemy| enemy.definition().resistances.clone();
    let high = resistances(Enemy::HighDragon);
    let fire = resistances(Enemy::FireDragon);
    let ice = resistances(Enemy::IceDragon);
    assert_eq!(
        attack_efficiency(Element::ColdFire, [high.as_slice()]),
        Efficiency::Halved
    );
    assert_eq!(
        attack_efficiency(Element::ColdFire, [fire.as_slice()]),
        Efficiency::Full
    );
    assert_eq!(
        attack_efficiency(Element::ColdFire, [fire.as_slice(), ice.as_slice()]),
        Efficiency::Full
    );
}

// Spec 032 C10
#[test]
fn resistances_apply_in_ranged_phase() {
    // Compendium, *Combat* S37 : Célérité renforcée (Attaque à distance 3) contre les Cogneurs : 1.
    let swiftness = card(Swiftness, 1);
    let state = fight_one(ironclads(), &[swiftness], &[]);
    let state = done(&state, declare(&[foe(&state, ORCS_2)]));
    let state = done(&state, strong(swiftness, WHITE_DIE));
    assert_eq!(total(&state), Some(1));
}

// Spec 032 C11
#[test]
fn concentration_keeps_the_element() {
    // Compendium, *Concentration* S2 : Polyvalence renforcée (Attaque de feu 3) + 2 = Attaque de
    // feu 5, entière contre les Cogneurs.
    let (concentration, versatility) = (card(Concentration, 1), card(BattleVersatility, 1));
    let state = fight_one(ironclads(), &[concentration, versatility], &[]);
    let state = done(&to_attack_phase(&state), declare(&[foe(&state, ORCS_2)]));
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
                card: versatility,
                option: VERSATILITY_FIRE_ATTACK,
                discard: None,
                set_dice: [None, None],
            }),
        },
    };
    let state = done(&state, boost);
    assert_eq!(total(&state), Some(5));
}

// --- Siège et fortifiés ---

// Spec 032 C12
#[test]
fn fortified_enemy_takes_only_siege_attacks_at_range() {
    // MKUE p. 9, *Ranged and Siege Attack Phase* 3.a : contre un fortifié, seulement le siège.
    let (versatility, swiftness) = (card(BattleVersatility, 1), card(Swiftness, 1));
    let state = fight_one(diggers(), &[versatility, swiftness], &[]);
    assert!(attack_targets(&state).contains(&vec![ORCS_2]));
    let state = done(&state, declare(&[foe(&state, ORCS_2)]));
    let legal = legal_actions(&state, SOLO);
    assert!(legal.contains(&strong_with_die(versatility, VERSATILITY_SIEGE, RED_DIE)));
    assert!(!legal.contains(&strong_with_die(versatility, VERSATILITY_RANGED, RED_DIE)));
    refused(&state, strong(swiftness, WHITE_DIE));
}

// Spec 032 C13
#[test]
fn siege_attack_counts_as_ranged_and_as_attack() {
    // MKUE p. 9, 3 : le siège se joue à distance contre un non-fortifié ; p. 10, 1.a : en phase
    // d'attaque, il compte avec les attaques de mêlée.
    let (versatility, swiftness, rage) = (
        card(BattleVersatility, 1),
        card(Swiftness, 1),
        card(Rage, 1),
    );
    let state = fight_one(prowlers(), &[versatility, swiftness, rage], &[]);
    let ranged = done(&state, declare(&[foe(&state, ORCS_2)]));
    let legal = legal_actions(&ranged, SOLO);
    assert!(legal.contains(&strong_with_die(versatility, VERSATILITY_SIEGE, RED_DIE)));
    assert!(legal.contains(&strong(swiftness, WHITE_DIE)));
    let attack = done(&to_attack_phase(&state), declare(&[foe(&state, ORCS_2)]));
    let attack = done(
        &attack,
        strong_with_die(versatility, VERSATILITY_SIEGE, RED_DIE),
    );
    let attack = done(&attack, option(rage, 0));
    assert_eq!(total(&attack), Some(4));
    assert_events_in_order(
        &events_of(&attack, &Action::ResolveAttack),
        &[defeated(&[foe(&state, ORCS_2)], 2)],
    );
}

// --- Calcul, vue, journal ---

// Spec 032 C14
#[test]
fn efficiency_table() {
    // MKUE p. 9, *Block Phase* 5.a.
    use Efficiency::{Full, Halved};
    use Element::{ColdFire, Fire, Ice, Physical};
    let table = [
        (Physical, [Full, Full, Full, Full]),
        (Fire, [Halved, Halved, Full, Full]),
        (Ice, [Halved, Full, Halved, Full]),
        (ColdFire, [Halved, Halved, Halved, Full]),
    ];
    for (attack, expected) in table {
        for (block, efficiency) in [Physical, Fire, Ice, ColdFire].into_iter().zip(expected) {
            assert_eq!(
                block_efficiency(attack, block),
                efficiency,
                "{attack:?} / {block:?}"
            );
        }
    }
    // MKUE p. 9, *Ranged and Siege Attack Phase* 4.a ; p. 24 : chaque résistance divise son élément.
    let cases: [(&[Resistance], [Efficiency; 4]); 5] = [
        (&[], [Full, Full, Full, Full]),
        (&[Resistance::Physical], [Halved, Full, Full, Full]),
        (&[Resistance::Fire], [Full, Halved, Full, Full]),
        (&[Resistance::Ice], [Full, Full, Halved, Full]),
        (
            &[Resistance::Fire, Resistance::Ice],
            [Full, Halved, Halved, Halved],
        ),
    ];
    for (resistances, expected) in cases {
        for (element, efficiency) in [Physical, Fire, Ice, ColdFire].into_iter().zip(expected) {
            assert_eq!(
                attack_efficiency(element, [resistances]),
                efficiency,
                "{resistances:?} / {element:?}"
            );
        }
    }
    assert_eq!(EnemyAttack::Ice(6).element(), Some(Ice));
    assert_eq!(EnemyAttack::Summon.element(), None);
}

// Spec 032 C15
#[test]
fn element_does_not_change_damage_to_hero() {
    // Compendium, *Combat* S4b : les Dragons de feu non paré inflige 9 points, 5 Blessures.
    let state = fight_one(fire_dragon(), &[], &[]);
    let state = to_block_phase(&state);
    let wounds: usize = events_of(&state, &Action::NextCombatPhase)
        .iter()
        .map(|event| match event {
            Event::WoundsTaken { wounds, .. } => wounds.len(),
            _ => 0,
        })
        .sum();
    assert_eq!(wounds, 5);
}

// Spec 032 C16
#[test]
fn view_shows_points_by_element_and_effective_total() {
    let (rage, march) = (card(Rage, 1), card(March, 1));
    let state = fight_one(ironclads(), &[rage, march], &[]);
    let state = done(&to_attack_phase(&state), declare(&[foe(&state, ORCS_2)]));
    let state = done(&state, option(rage, 0));
    let state = done(&state, sideways_attack(march));
    let combat = view(&state, SOLO).combat.expect("combat dans la vue");
    assert_eq!(
        combat.in_progress,
        Some(InProgress::Attack {
            targets: vec![foe(&state, ORCS_2)],
            points: ElementPoints {
                physical: 3,
                ..ElementPoints::default()
            },
        })
    );
    assert_eq!(combat.in_progress_total(), Some(1));
    assert_eq!(
        combat.efficiency(Effect::Attack {
            amount: 2,
            element: Element::Physical,
            kind: AttackKind::Melee
        }),
        Some(Efficiency::Halved)
    );
    assert_eq!(
        combat.efficiency(Effect::Attack {
            amount: 3,
            element: Element::Fire,
            kind: AttackKind::Melee
        }),
        Some(Efficiency::Full)
    );
    let state = done(&state, Action::ResolveAttack);
    let combat = state.combat(SOLO).expect("le combat continue");
    assert!(matches!(
        combat.history.last(),
        Some(CombatRecord::Attack {
            points: 1,
            needed: 3,
            success: false,
            ..
        })
    ));
}

// Spec 032 C17
#[test]
fn replaying_elemental_combat_rebuilds_state() {
    // Spec 024 : l'état se reconstruit depuis les événements, cartes de feu comprises.
    let versatility = card(BattleVersatility, 1);
    let start = to_attack_phase(&fight_one(ironclads(), &[versatility, card(Rage, 1)], &[]));
    let mut state = start.clone();
    let mut events = Vec::new();
    for action in [
        declare(&[foe(&state, ORCS_2)]),
        strong_with_die(versatility, VERSATILITY_FIRE_ATTACK, RED_DIE),
    ] {
        let produced = events_of(&state, &action);
        state = produced.iter().fold(state, apply);
        events.extend(produced);
    }
    let replayed = events.iter().fold(start, apply);
    assert_eq!(replayed, state);
    assert_eq!(total(&replayed), Some(3));
}

// Spec 032 L3
#[test]
fn a_single_halved_point_counts_zero() {
    let march = card(March, 1);
    let state = fight_one(ironclads(), &[march], &[]);
    let state = done(&to_attack_phase(&state), declare(&[foe(&state, ORCS_2)]));
    let state = done(&state, sideways_attack(march));
    assert_eq!(total(&state), Some(0));
    assert_eq!(state.play_area(SOLO).len(), 1);
}

#[test]
fn versatility_attack_option_is_physical() {
    // Compendium, *Combat* S5 : une Attaque sans élément écrit est physique.
    let versatility = card(BattleVersatility, 1);
    let state = fight_one(ironclads(), &[versatility], &[]);
    let state = done(&to_attack_phase(&state), declare(&[foe(&state, ORCS_2)]));
    let state = done(
        &state,
        strong_with_die(versatility, VERSATILITY_ATTACK, RED_DIE),
    );
    assert_eq!(total(&state), Some(2));
}
