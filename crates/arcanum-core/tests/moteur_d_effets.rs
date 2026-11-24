//! Scénarios de la spec 052 (moteur d'effets : modificateurs, déplacement, retraite forcée,
//! conversions, Embuscade, fin du tour).

use arcanum_core::{
    Action, AdvancedAction, BasicAction, CardId, CardPlay, Event, GameRunner, GameState, Hex,
    InProgress, ManaPayment, Modifier, ModifierDuration, ModifierKind, ModifierSource, MoveCost,
    MoveRules, SpaceContent, Terrain, TerrainChange, Terrains, UndoStatus, Unit, UnitCard,
    legal_actions, move_cost, routes, view,
};

mod arene;
use arene::*;

use AdvancedAction::*;
use BasicAction::*;

// --- Décors ---

/// Le lac de la campagne 1, voisin du village et des orques ① (spec 052, exemple).
const LAKE: Hex = Hex::new(2, -4);

/// L'effet de base de `card` (option `option`), sans défausse ni mana.
fn basic_option(card: CardId, option: u8) -> Action {
    Action::PlayCard {
        card,
        play: CardPlay::Basic {
            die: None,
            gain: None,
            with: None,
            option,
            discard: None,
            mana: None,
        },
    }
}

/// Le bac à sable (Arythea) sur le village, avec `hand` en main et les dés de `Sandbox`.
fn on_village(hand: &[CardId]) -> GameState {
    Sandbox {
        figure: Some(VILLAGE),
        hand: hand.to_vec(),
        ..Sandbox::default()
    }
    .state()
}

/// La fin du tour sans défausse, si elle est légale.
fn end_turn_plain(state: &GameState) -> Action {
    arcanum_core::legal_actions(state, SOLO)
        .into_iter()
        .find(|action| matches!(action, Action::EndTurn { discard, reclaim: None, glade: None, .. } if discard.is_empty()))
        .expect("une fin de tour sans défausse")
}

fn path_finding_turn(card: CardId) -> Modifier {
    Modifier {
        kind: ModifierKind::TerrainCost {
            terrains: Terrains::ALL,
            change: TerrainChange::Reduce { by: 1, min: 2 },
        },
        source: ModifierSource::Card(card),
        duration: ModifierDuration::Turn,
    }
}

// --- Modificateurs (ADR 0008) ---

// Spec 052 C1
#[test]
fn modifiers_end_with_the_turn() {
    // MKUE p. 5, *Effects* 1.b : un effet dure le tour où il est joué.
    let path_finding = advanced(PathFinding);
    let state = on_village(&[path_finding, card(March, 1)]);
    let state = done(&state, basic(path_finding));
    assert_eq!(state.modifiers(SOLO), [path_finding_turn(path_finding)]);
    let state = done(&state, end_turn_plain(&state));
    assert!(state.modifiers(SOLO).is_empty());
}

// Spec 052 C3
#[test]
fn modifiers_follow_undo_and_reload() {
    // ADR 0008 ; spec 024 : le retour en arrière et la sauvegarde suivent l'état sans travail de
    // plus.
    let path_finding = advanced(PathFinding);
    let events = Sandbox {
        figure: Some(VILLAGE),
        hand: vec![path_finding, card(March, 1)],
        ..Sandbox::default()
    }
    .events();
    let (mut runner, puppet) = runner(events);
    play(&mut runner, &puppet, basic(path_finding));
    assert_eq!(runner.state().modifiers(SOLO).len(), 1);
    // Rechargée : le journal rejoué retrouve l'effet en cours.
    let reloaded = GameRunner::resume(
        runner.log().clone(),
        runner.rng_positions(),
        vec![Box::new(Puppet::default())],
    )
    .expect("journal valide");
    assert_eq!(
        reloaded.state().modifiers(SOLO),
        [path_finding_turn(path_finding)]
    );
    assert_eq!(runner.undo_status(), UndoStatus::Possible);
    assert_eq!(runner.undo(), Ok(()));
    assert!(runner.state().modifiers(SOLO).is_empty());
}

// Spec 052 C4
#[test]
fn view_lists_modifiers() {
    let path_finding = advanced(PathFinding);
    let state = on_village(&[path_finding]);
    let state = done(&state, basic(path_finding));
    assert_eq!(
        view(&state, SOLO).modifiers,
        [path_finding_turn(path_finding)]
    );
}

// --- Déplacement ---

/// Le coût d'une case de ce terrain, sans site, sous ces règles.
fn cost(rules: &MoveRules, terrain: Terrain) -> MoveCost {
    move_cost(
        SpaceContent {
            terrain,
            feature: None,
        },
        rules,
    )
}

/// L'effet renforcé de `card`, payé par le dé `die`, option `option`.
fn strong_option(card: CardId, die: u8, option: u8) -> Action {
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

// Spec 052 C11
#[test]
fn path_finding_reduces_by_one_minimum_two() {
    // MKUE p. 7, 7.a : un effet peut réduire le coût de déplacement.
    let path_finding = advanced(PathFinding);
    let state = done(&on_village(&[path_finding]), basic(path_finding));
    let rules = state.move_rules(SOLO);
    assert_eq!(cost(&rules, Terrain::Forest), MoveCost::Points(2));
    assert_eq!(cost(&rules, Terrain::Hills), MoveCost::Points(2));
    assert_eq!(cost(&rules, Terrain::Desert), MoveCost::Points(4));
    assert_eq!(cost(&rules, Terrain::Swamp), MoveCost::Points(4));
    assert_eq!(cost(&rules, Terrain::Plains), MoveCost::Points(2));
}

// Spec 052 C12
#[test]
fn strong_path_finding_sets_two() {
    // Compendium, *Path Finding* S1 : tous les terrains accessibles à 2, pas les lacs ni les
    // montagnes.
    let path_finding = advanced(PathFinding);
    let state = done(
        &on_village(&[path_finding]),
        strong_option(path_finding, GREEN_DIE, 0),
    );
    let rules = state.move_rules(SOLO);
    for terrain in [
        Terrain::Plains,
        Terrain::Hills,
        Terrain::Forest,
        Terrain::Wasteland,
        Terrain::Desert,
        Terrain::Swamp,
    ] {
        assert_eq!(cost(&rules, terrain), MoveCost::Points(2), "{terrain:?}");
    }
    assert_eq!(cost(&rules, Terrain::Lake), MoveCost::Inaccessible);
    assert_eq!(cost(&rules, Terrain::Mountain), MoveCost::Inaccessible);
}

// Spec 052 C13
#[test]
fn frost_bridge_swamp_costs_one() {
    // Carte Pont de glace : marais à 1 ; le lac reste infranchissable sans l'effet renforcé.
    let frost_bridge = advanced(FrostBridge);
    let state = done(&on_village(&[frost_bridge]), basic(frost_bridge));
    let rules = state.move_rules(SOLO);
    assert_eq!(cost(&rules, Terrain::Swamp), MoveCost::Points(1));
    assert_eq!(cost(&rules, Terrain::Lake), MoveCost::Inaccessible);
}

// Spec 052 C14
#[test]
fn strong_frost_bridge_crosses_lakes() {
    // MKUE p. 7, 7.b : un effet peut rendre un terrain franchissable.
    let frost_bridge = advanced(FrostBridge);
    let state = done(
        &on_village(&[frost_bridge]),
        strong_option(frost_bridge, BLUE_DIE, 0),
    );
    let rules = state.move_rules(SOLO);
    assert_eq!(cost(&rules, Terrain::Lake), MoveCost::Points(1));
    assert_eq!(cost(&rules, Terrain::Mountain), MoveCost::Inaccessible);
    assert!(legal_actions(&state, SOLO).contains(&Action::MoveHero { to: LAKE }));
    assert!(
        routes(state.map(), VILLAGE, &rules)
            .iter()
            .any(|route| route.to == LAKE && route.cost == 1)
    );
}

// Spec 052 C15
#[test]
fn terrain_reductions_use_best_order() {
    // MKUE p. 7, 7.a ; Compendium, *Song of Wind* S4 : l'ordre le plus favorable au joueur.
    let frost_bridge = advanced(FrostBridge);
    let path_finding = advanced(PathFinding);
    let state = on_village(&[frost_bridge, path_finding]);
    let state = done(&state, basic(frost_bridge));
    let state = done(&state, strong_option(path_finding, GREEN_DIE, 0));
    let rules = state.move_rules(SOLO);
    assert_eq!(cost(&rules, Terrain::Swamp), MoveCost::Points(1));
    assert_eq!(cost(&rules, Terrain::Forest), MoveCost::Points(2));
}

fn foresters() -> UnitCard {
    UnitCard::new(Unit::Foresters, 1)
}

fn activate(unit: UnitCard, ability: u8, option: u8) -> Action {
    Action::ActivateUnit {
        enemy: None,
        unit,
        ability,
        option,
        mana: Vec::new(),
        target: None,
        leadership: false,
    }
}

// Spec 052 C16
#[test]
fn foresters_reduce_forest_hills_swamp() {
    // Carte des Bûcherons : forêts, collines et marais −1 pour le tour, jusqu'à 0.
    let state = Units {
        owned: vec![foresters()],
        ..Units::default()
    }
    .state();
    let state = done(&state, activate(foresters(), 0, 0));
    assert_eq!(state.move_points(SOLO), 2);
    let rules = state.move_rules(SOLO);
    assert_eq!(cost(&rules, Terrain::Forest), MoveCost::Points(2));
    assert_eq!(cost(&rules, Terrain::Hills), MoveCost::Points(2));
    assert_eq!(cost(&rules, Terrain::Swamp), MoveCost::Points(4));
    assert_eq!(cost(&rules, Terrain::Plains), MoveCost::Points(2));
}

/// La forêt à l'O de la clairière (campagne 1), à deux pas du village par la clairière.
const WEST_FOREST: Hex = Hex::new(0, -3);

// Spec 052 C17
#[test]
fn routes_use_reduced_costs() {
    // Spec 017 et 036 : les cases atteignables et le trajet suivi paient les coûts réduits.
    let path_finding = advanced(PathFinding);
    let state = on_village(&[path_finding, card(March, 1)]);
    let state = done(&state, basic(path_finding));
    let state = done(&state, basic(card(March, 1)));
    assert_eq!(state.move_points(SOLO), 4);
    let follow = Action::FollowRoute { to: WEST_FOREST };
    assert!(legal_actions(&state, SOLO).contains(&follow));
    let events = events_of(&state, &follow);
    let costs: Vec<u8> = events
        .iter()
        .filter_map(|event| match event {
            Event::HeroMoved { cost, .. } => Some(*cost),
            _ => None,
        })
        .collect();
    assert_eq!(costs, [2, 2]);
}

// Spec 052 C18
#[test]
fn modifier_survives_action_start() {
    // MKUE p. 6, 6.e : les points de Mouvement sont perdus au début de l'action, pas l'effet.
    let path_finding = advanced(PathFinding);
    let state = arena(VILLAGE, &[path_finding], &[(prowlers(), ORCS_2)]);
    let state = done(&state, basic(path_finding));
    let state = done(&state, challenge(ORCS_2));
    assert_eq!(state.move_points(SOLO), 0);
    assert_eq!(state.modifiers(SOLO), [path_finding_turn(path_finding)]);
}

// Spec 052 C43
#[test]
fn view_move_rules_follow_modifiers() {
    let path_finding = advanced(PathFinding);
    let state = done(&on_village(&[path_finding]), basic(path_finding));
    assert_eq!(
        cost(&view(&state, SOLO).move_rules, Terrain::Forest),
        MoveCost::Points(2)
    );
}

// --- Quand une carte se joue ---

/// Les façons de jouer `card` parmi les actions légales, de côté exclu.
fn effect_plays(state: &GameState, card: CardId) -> Vec<CardPlay> {
    legal_actions(state, SOLO)
        .into_iter()
        .filter_map(|action| match action {
            Action::PlayCard { card: played, play } if played == card => Some(play),
            _ => None,
        })
        .filter(|play| !matches!(play, CardPlay::Sideways(_)))
        .collect()
}

/// Les Rôdeurs défiés depuis le village avec `hand`, en phase d'attaque sans dégâts subis.
fn attack_phase_against_prowlers(hand: &[CardId]) -> GameState {
    skip_to_attack_phase(&fight(VILLAGE, hand, &[(prowlers(), ORCS_2)], &[ORCS_2]))
}

/// Le combat contre les Rôdeurs fini, sans les avoir vaincus.
fn after_combat(hand: &[CardId]) -> GameState {
    let state = attack_phase_against_prowlers(hand);
    done(&state, Action::EndCombat)
}

// Spec 052 C6
#[test]
fn agility_playable_in_combat() {
    // Compendium, *Agility* S2 : Agilité se joue en combat, ses points vont dans la réserve.
    let agility = advanced(Agility);
    let march = card(March, 1);
    let state = attack_phase_against_prowlers(&[agility, march]);
    assert!(effect_plays(&state, march).is_empty());
    assert!(legal_actions(&state, SOLO).contains(&basic(agility)));
    let state = done(&state, basic(agility));
    assert_eq!(state.move_points(SOLO), 2);
}

// Spec 052 C7
#[test]
fn diplomacy_playable_until_block_phase_ends() {
    // Compendium, *Diplomacy* S1 ; *Miscellaneous* S1.
    let diplomacy = advanced(Diplomacy);
    let fighting = fight(VILLAGE, &[diplomacy], &[(prowlers(), ORCS_2)], &[ORCS_2]);
    let blocking = to_block_phase(&fighting);
    assert!(legal_actions(&blocking, SOLO).contains(&basic(diplomacy)));
    let blocking = done(&blocking, basic(diplomacy));
    assert_eq!(blocking.influence_points(SOLO), 2);
    let attacking = skip_to_attack_phase(&fighting);
    assert!(effect_plays(&attacking, diplomacy).is_empty());
}

// Spec 052 C8
#[test]
fn ambush_playable_in_combat() {
    // Compendium, *Ambush* S4 : jouée en combat, juste avant la carte visée.
    let ambush = advanced(Ambush);
    let state = attack_phase_against_prowlers(&[ambush, card(Rage, 1)]);
    let state = done(&state, declare(&[foe(&state, ORCS_2)]));
    assert!(legal_actions(&state, SOLO).contains(&basic(ambush)));
}

// Spec 052 C9
#[test]
fn movement_cards_not_playable_after_combat() {
    // MKUE p. 6, 5 : le Mouvement ne sert plus après le combat, ni ces effets.
    let cards = [
        advanced(PathFinding),
        advanced(Agility),
        advanced(Ambush),
        advanced(SteadyTempo),
    ];
    let state = after_combat(&cards);
    for card in cards {
        assert!(effect_plays(&state, card).is_empty(), "{card:?}");
    }
}

// Spec 052 C10
#[test]
fn intimidate_playable_when_points_serve() {
    // MKUE p. 6, 5 ; Compendium, *Miscellaneous* S1 : un effet se joue si ses points servent.
    let intimidate = advanced(Intimidate);
    let state = after_combat(&[intimidate]);
    assert!(effect_plays(&state, intimidate).is_empty());
    let state = Units {
        hand: vec![intimidate],
        ..Units::default()
    }
    .state();
    let state = done(&state, Action::Interact);
    assert!(legal_actions(&state, SOLO).contains(&basic_option(intimidate, 0)));
    assert!(!legal_actions(&state, SOLO).contains(&basic_option(intimidate, 1)));
}

// --- Conversions ---

/// Les points de l'attaque ou de la parade en cours.
fn in_progress_points(state: &GameState) -> arcanum_core::ElementPoints {
    match state
        .combat(SOLO)
        .and_then(|combat| combat.in_progress.clone())
    {
        Some(InProgress::Attack { points, .. } | InProgress::Block { points, .. }) => points,
        None => panic!("ni attaque ni parade en cours"),
    }
}

/// Agilité (`strong` : renforcée par le dé blanc) jouée sur le village, puis les ennemis `placed`
/// défiés et le combat engagé.
fn agility_then_fight(
    strong: bool,
    placed: &[(arcanum_core::EnemyToken, Hex)],
    hand: &[CardId],
) -> GameState {
    let agility = advanced(Agility);
    let mut full = vec![agility];
    full.extend_from_slice(hand);
    let state = arena(VILLAGE, &full, placed);
    let play = if strong {
        strong_option(agility, WHITE_DIE, 0)
    } else {
        basic(agility)
    };
    let mut state = done(&state, play);
    for &(_, hex) in placed {
        state = done(&state, challenge(hex));
    }
    done(&state, Action::StartCombat)
}

// Spec 052 C28
#[test]
fn agility_converts_move_to_attack() {
    // Compendium, *Agility* S2 : 1 Mouvement pour Attaque 1 dans la phase d'attaque.
    let march = card(March, 1);
    let state = agility_then_fight(false, &[(prowlers(), ORCS_2)], &[march]);
    let state = skip_to_attack_phase(&state);
    let state = done(&state, declare(&[foe(&state, ORCS_2)]));
    let state = done(&state, basic(march));
    assert_eq!(state.move_points(SOLO), 2);
    assert!(legal_actions(&state, SOLO).contains(&Action::SpendMoveAsAttack));
    let state = done(&state, Action::SpendMoveAsAttack);
    let state = done(&state, Action::SpendMoveAsAttack);
    assert_eq!(in_progress_points(&state), physical(2));
    assert_eq!(state.move_points(SOLO), 0);
    refused(&state, Action::SpendMoveAsAttack);
}

// Spec 052 C29
#[test]
fn strong_agility_ranged_two_for_one() {
    // Compendium, *Agility* S1 : 2 Mouvement pour Attaque à distance 1, pas contre un fortifié.
    let march = card(March, 1);
    let state = agility_then_fight(true, &[(prowlers(), ORCS_2)], &[march]);
    let state = done(&state, declare(&[foe(&state, ORCS_2)]));
    let state = done(&state, basic(march));
    let events = events_of(&state, &Action::SpendMoveAsAttack);
    assert_eq!(
        events,
        [Event::MoveSpentAsAttack {
            player: SOLO,
            ranged: true,
        }]
    );
    let state = events.iter().fold(state, arcanum_core::apply);
    assert_eq!(in_progress_points(&state), physical(1));
    assert_eq!(state.move_points(SOLO), 0);
    // Contre des Creuseurs (fortifiés) : non.
    let state = agility_then_fight(true, &[(diggers(), ORCS_2)], &[march]);
    let state = done(&state, declare(&[foe(&state, ORCS_2)]));
    let state = done(&state, basic(march));
    refused(&state, Action::SpendMoveAsAttack);
    // Agilité de base : pas de conversion à distance.
    let state = agility_then_fight(false, &[(prowlers(), ORCS_2)], &[march]);
    let state = done(&state, declare(&[foe(&state, ORCS_2)]));
    let state = done(&state, basic(march));
    refused(&state, Action::SpendMoveAsAttack);
}

// Spec 052 C30
#[test]
fn agility_does_not_keep_movement_points() {
    // Compendium, *Movement* S1 : les points de Mouvement d'avant le combat sont perdus.
    let state = agility_then_fight(false, &[(prowlers(), ORCS_2)], &[]);
    assert_eq!(state.move_points(SOLO), 0);
}

/// Les ennemis `placed` défiés depuis le village, combat engagé, en phase de parade.
fn block_phase_against(hand: &[CardId], placed: &[(arcanum_core::EnemyToken, Hex)]) -> GameState {
    let hexes: Vec<Hex> = placed.iter().map(|&(_, hex)| hex).collect();
    to_block_phase(&fight(VILLAGE, hand, placed, &hexes))
}

// Spec 052 C31
#[test]
fn diplomacy_converts_influence_to_block() {
    // Compendium, *Diplomacy* S1 : 1 Influence pour Parade 1.
    let diplomacy = advanced(Diplomacy);
    let promise = card(Promise, 1);
    let threaten = card(Threaten, 1);
    let state = block_phase_against(&[diplomacy, promise, threaten], &[(prowlers(), ORCS_2)]);
    let state = done(&state, basic(diplomacy));
    let state = done(&state, declare_block(foe(&state, ORCS_2)));
    let state = done(&state, basic(promise));
    let state = done(&state, basic(threaten));
    assert_eq!(state.influence_points(SOLO), 6);
    let mut state = state;
    for _ in 0..4 {
        state = done(&state, Action::SpendInfluenceAsBlock);
    }
    assert_eq!(in_progress_points(&state), physical(4));
    let events = events_of(&state, &Action::ResolveBlock);
    assert!(matches!(events[0], Event::EnemyBlocked { .. }));
}

// Spec 052 C32
#[test]
fn strong_diplomacy_chosen_element() {
    // Compendium, *Diplomacy* S2, S3 ; spec 032 : une Parade de glace compte en plein contre le
    // feu.
    let diplomacy = advanced(Diplomacy);
    let promise = card(Promise, 1);
    let fire_mages = token(arcanum_core::Enemy::FireMages, 1);
    let state = block_phase_against(&[diplomacy, promise], &[(fire_mages, ORCS_2)]);
    let state = done(&state, strong_option(diplomacy, WHITE_DIE, 0));
    let state = done(&state, declare_block(foe(&state, ORCS_2)));
    let state = done(&state, basic(promise));
    let events = events_of(&state, &Action::SpendInfluenceAsBlock);
    assert_eq!(
        events,
        [Event::InfluenceSpentAsBlock {
            player: SOLO,
            element: arcanum_core::Element::Ice,
        }]
    );
    let mut state = state;
    for _ in 0..6 {
        state = done(&state, Action::SpendInfluenceAsBlock);
    }
    let combat = state.combat(SOLO).expect("combat engagé");
    assert_eq!(combat.in_progress_total(), Some(6));
    let events = events_of(&state, &Action::ResolveBlock);
    assert!(matches!(events[0], Event::EnemyBlocked { .. }));
}

// Spec 052 C33
#[test]
fn diplomacy_pool_serves_several_blocks() {
    // Compendium, *Diplomacy* S1 : l'Influence restante sert à une autre parade.
    let diplomacy = advanced(Diplomacy);
    let hand = [
        diplomacy,
        card(Promise, 1),
        card(Threaten, 1),
        card(Promise, 2),
    ];
    let state = block_phase_against(&hand, &[(prowlers(), ORCS_2), (diggers(), PLAIN_W)]);
    let mut state = done(&state, basic(diplomacy));
    for &card in &hand[1..] {
        state = done(&state, basic(card));
    }
    assert_eq!(state.influence_points(SOLO), 8);
    for (hex, needed) in [(ORCS_2, 4), (PLAIN_W, 3)] {
        state = done(&state, declare_block(foe(&state, hex)));
        for _ in 0..needed {
            state = done(&state, Action::SpendInfluenceAsBlock);
        }
        state = done(&state, Action::ResolveBlock);
    }
    assert_eq!(state.influence_points(SOLO), 1);
    let combat = state.combat(SOLO).expect("combat engagé");
    assert!(combat.enemies.iter().all(|enemy| enemy.blocked));
}

// Spec 052 L1 et L2
#[test]
fn conversions_need_their_moment() {
    let march = card(March, 1);
    // Sans attaque en cours, ou pendant une parade : refusée.
    let state = agility_then_fight(false, &[(prowlers(), ORCS_2)], &[march]);
    let state = skip_to_attack_phase(&state);
    let state = done(&state, basic(march));
    refused(&state, Action::SpendMoveAsAttack);
    // Influence en phase d'attaque : refusée.
    let diplomacy = advanced(Diplomacy);
    let state = block_phase_against(&[diplomacy], &[(prowlers(), ORCS_2)]);
    let state = done(&state, basic(diplomacy));
    let state = skip_to_attack_phase(&state);
    let state = done(&state, declare(&[foe(&state, ORCS_2)]));
    refused(&state, Action::SpendInfluenceAsBlock);
}

// --- Embuscade ---

/// Le bonus d'Embuscade en attente, s'il y en a un.
fn pending_bonus(state: &GameState) -> Option<(u8, u8)> {
    state
        .modifiers(SOLO)
        .iter()
        .find_map(|modifier| match modifier.kind {
            ModifierKind::NextCardBonus { attack, block } => Some((attack, block)),
            _ => None,
        })
}

/// Concentration renforcée (dé vert) qui joue l'option 0 de `target`.
fn concentration_on(concentration: CardId, target: CardId) -> Action {
    Action::PlayCard {
        card: concentration,
        play: CardPlay::Strong {
            gain: None,
            extra_mana: None,
            option: 0,
            discard: None,
            mana: ManaPayment::Die(GREEN_DIE),
            set_dice: [None, None],
            with: Some(arcanum_core::BoostTarget {
                card: target,
                option: 0,
                discard: None,
                set_dice: [None, None],
            }),
        },
    }
}

// Spec 052 C2
#[test]
fn unused_ambush_ends_with_turn() {
    // MKUE p. 5, *Effects* 1.b : le bonus non utilisé disparaît avec le tour.
    let ambush = advanced(Ambush);
    let state = on_village(&[ambush, card(March, 1)]);
    let state = done(&state, basic(ambush));
    assert_eq!(pending_bonus(&state), Some((1, 2)));
    let state = done(&state, end_turn_plain(&state));
    assert_eq!(pending_bonus(&state), None);
}

// Spec 052 C5
#[test]
fn concentration_boosts_points_not_modifiers() {
    // Compendium, *Will Focus* S1 ; *Ambush* S5 : Concentration renforce les points, pas les
    // effets en cours.
    let concentration = card(Concentration, 1);
    let path_finding = advanced(PathFinding);
    let state = on_village(&[concentration, path_finding]);
    let state = done(&state, concentration_on(concentration, path_finding));
    assert_eq!(state.move_points(SOLO), 6);
    let rules = state.move_rules(SOLO);
    for terrain in [Terrain::Forest, Terrain::Desert, Terrain::Plains] {
        assert_eq!(cost(&rules, terrain), MoveCost::Points(2), "{terrain:?}");
    }
    let ambush = advanced(Ambush);
    let state = on_village(&[concentration, ambush]);
    let state = done(&state, concentration_on(concentration, ambush));
    assert_eq!(state.move_points(SOLO), 6);
    assert_eq!(pending_bonus(&state), Some((2, 4)));
}

/// Embuscade (`strong` : renforcée par le dé vert) jouée sur le village, puis les Prowlers défiés
/// et le combat engagé, `hand` en main.
fn ambush_then_fight(strong: bool, hand: &[CardId]) -> GameState {
    let ambush = advanced(Ambush);
    let mut full = vec![ambush];
    full.extend_from_slice(hand);
    let state = arena(VILLAGE, &full, &[(prowlers(), ORCS_2)]);
    let play = if strong {
        strong_option(ambush, GREEN_DIE, 0)
    } else {
        basic(ambush)
    };
    let state = done(&state, play);
    let state = done(&state, challenge(ORCS_2));
    done(&state, Action::StartCombat)
}

// Spec 052 C34
#[test]
fn ambush_bonus_on_first_attack_card() {
    // Embuscade (carte) : +1 à la première carte d'Attaque, une seule fois.
    let rage = card(Rage, 1);
    let second = card(Rage, 2);
    let state = skip_to_attack_phase(&ambush_then_fight(false, &[rage, second]));
    let state = done(&state, declare(&[foe(&state, ORCS_2)]));
    let state = done(&state, basic(rage));
    assert_eq!(in_progress_points(&state), physical(3));
    assert_eq!(pending_bonus(&state), None);
    let state = done(&state, basic(second));
    assert_eq!(in_progress_points(&state), physical(5));
}

// Spec 052 C35
#[test]
fn ambush_applies_to_sideways_block() {
    // FAQ p2 ; Compendium, *Ambush* S2 : une carte jouée de côté pour la Parade reçoit le bonus.
    let march = card(March, 1);
    let state = to_block_phase(&ambush_then_fight(true, &[march]));
    let state = done(&state, declare_block(foe(&state, ORCS_2)));
    let state = done(&state, sideways_block(march));
    assert_eq!(in_progress_points(&state), physical(5));
    assert_eq!(pending_bonus(&state), None);
}

// Spec 052 C36
#[test]
fn ambush_ignores_units() {
    // FAQ p2 ; Compendium, *Ambush* S3 : les unités ne profitent pas d'Embuscade.
    let ambush = advanced(Ambush);
    let swordsmen = UnitCard::new(Unit::UtemSwordsmen, 1);
    let state = Units {
        hand: vec![ambush],
        owned: vec![swordsmen],
        placed: vec![(prowlers(), ORCS_2)],
        ..Units::default()
    }
    .state();
    let state = done(&state, basic(ambush));
    let state = done(&state, challenge(ORCS_2));
    let state = skip_to_attack_phase(&done(&state, Action::StartCombat));
    let state = done(&state, declare(&[foe(&state, ORCS_2)]));
    let state = done(&state, activate(swordsmen, 0, 0));
    assert_eq!(in_progress_points(&state), physical(3));
    assert_eq!(pending_bonus(&state), Some((1, 2)));
}

// Spec 052 C37
#[test]
fn ambush_ignores_conversions() {
    // Compendium, *Ambush* S6 : l'Attaque d'Agilité ne vient pas d'une carte d'Attaque.
    let ambush = advanced(Ambush);
    let march = card(March, 1);
    let state = agility_then_fight(false, &[(prowlers(), ORCS_2)], &[ambush, march]);
    let state = skip_to_attack_phase(&state);
    let state = done(&state, declare(&[foe(&state, ORCS_2)]));
    let state = done(&state, basic(ambush));
    let state = done(&state, basic(march));
    let state = done(&state, Action::SpendMoveAsAttack);
    assert_eq!(in_progress_points(&state), physical(1));
    assert_eq!(pending_bonus(&state), Some((1, 2)));
}

// --- Case sécurisée et retraite forcée ---

/// La clairière de la campagne 1, voisine du village et du lac.
const GLADE: arcanum_core::Hex = Hex::new(1, -3);

/// Les cases de la campagne 11 posée en 2B (`docs/reference/bac-a-sable.md`) : lacs à l'E et au N-E,
/// collines au N-O et au S-E, tour de mage au centre (`MAGE_TOWER`).
const LAKE_E: Hex = Hex::new(3, -6);
const LAKE_NE: Hex = Hex::new(3, -7);
const HILLS_NW: Hex = Hex::new(2, -7);
const HILLS_SE: Hex = Hex::new(2, -5);

fn frost_bridge() -> CardId {
    advanced(FrostBridge)
}

fn move_to(to: Hex) -> Action {
    Action::MoveHero { to }
}

/// Le bac à sable avec la figurine sur `figure`, `hand` en main, puis Pont de glace renforcé
/// (dé bleu) joué : Mouvement 4, lacs à 1.
fn frost_bridge_on(sandbox: Sandbox) -> GameState {
    let state = sandbox.state();
    done(&state, strong_option(frost_bridge(), BLUE_DIE, 0))
}

/// Le bac à sable, figurine sur `figure`, Pont de glace en main, la campagne 11 posée en 2B (avec
/// le défenseur violet de sa tour si `defended`).
fn countryside_11(figure: Hex, defended: bool) -> GameState {
    let tile = arcanum_core::TileId::Countryside(11);
    let sandbox = Sandbox {
        figure: Some(figure),
        hand: vec![frost_bridge(), card(March, 1)],
        next: vec![tile],
        ..Sandbox::default()
    };
    let violet = sandbox.state().enemy_pile(arcanum_core::EnemyColor::Violet)[0];
    let mut events = sandbox.events();
    events.push(Event::TileRevealed {
        tile,
        position: MAGE_TOWER_POSITION,
    });
    if defended {
        events.push(Event::DefenderPlaced {
            token: violet,
            hex: MAGE_TOWER,
        });
    }
    let state = events
        .iter()
        .fold(GameState::default(), arcanum_core::apply);
    done(&state, strong_option(frost_bridge(), BLUE_DIE, 0))
}

/// Pont de glace renforcé joué sur le village, puis le pas sur le lac de la campagne 1.
fn on_lake_from_village() -> GameState {
    let state = frost_bridge_on(Sandbox {
        figure: Some(VILLAGE),
        hand: vec![frost_bridge(), card(March, 1)],
        ..Sandbox::default()
    });
    done(&state, move_to(LAKE))
}

fn withdrawal(path: &[Hex], wounds: &[CardId]) -> Event {
    Event::ForcedWithdrawal {
        player: SOLO,
        path: path.to_vec(),
        wounds: wounds.to_vec(),
    }
}

// Spec 052 C19
#[test]
fn forced_withdrawal_from_lake() {
    // MKUE p. 11, *End of the Turn* 2 : retour sur ses pas jusqu'à une case sécurisée, une
    // Blessure par case.
    let state = on_lake_from_village();
    let events = events_of(&state, &end_turn_plain(&state));
    assert!(events.contains(&withdrawal(&[VILLAGE], &[wound(1)])));
    let after = events.iter().fold(state, arcanum_core::apply);
    assert_eq!(after.figure(SOLO), Some(VILLAGE));
    assert!(after.hand(SOLO).contains(&wound(1)));
}

// Spec 052 C20
#[test]
fn forced_withdrawal_counts_each_space() {
    // MKUE p. 11, *End of the Turn* 2.b : une Blessure par case parcourue.
    let state = countryside_11(HILLS_NW, false);
    let state = done(&state, move_to(LAKE_NE));
    let state = done(&state, move_to(LAKE_E));
    let events = events_of(&state, &end_turn_plain(&state));
    assert!(events.contains(&withdrawal(&[LAKE_NE, HILLS_NW], &[wound(1), wound(2)])));
}

// Spec 052 C21
#[test]
fn end_turn_evaluated_after_withdrawal() {
    // MKUE p. 11, 2 et 4.a : la clairière d'arrivée jette la Blessure de la retraite.
    let state = frost_bridge_on(Sandbox {
        figure: Some(GLADE),
        hand: vec![frost_bridge()],
        ..Sandbox::default()
    });
    let state = done(&state, move_to(LAKE));
    let glade_end = Action::EndTurn {
        discard: Vec::new(),
        reclaim: None,
        glade: Some(arcanum_core::WoundPile::Hand),
        steady_tempo: false,
    };
    assert!(legal_actions(&state, SOLO).contains(&glade_end));
    let events = events_of(&state, &glade_end);
    assert_events_in_order(
        &events,
        &[
            withdrawal(&[GLADE], &[wound(1)]),
            Event::WoundThrownAway {
                player: SOLO,
                from: arcanum_core::WoundPile::Hand,
            },
        ],
    );
}

// Spec 052 C22
#[test]
fn forced_withdrawal_does_not_knock_out() {
    // Spec 034 : l'assommé ne vient que des Blessures d'un combat.
    let state = on_lake_from_village();
    let before = state.hand(SOLO).to_vec();
    let wounds: Vec<CardId> = (1..=6).map(wound).collect();
    let after = arcanum_core::apply(state, &withdrawal(&[VILLAGE], &wounds));
    let mut expected = before;
    expected.extend(&wounds);
    assert_eq!(after.hand(SOLO), expected);
    assert_eq!(after.figure(SOLO), Some(VILLAGE));
}

// Spec 052 C23
#[test]
fn forced_withdrawal_provokes_nobody() {
    // Compendium, *Forced Withdrawal* S3 : le retour ne provoque pas.
    let state = on_lake_from_village();
    // Des Rôdeurs sur la clairière, voisine du lac et du village.
    let state = arcanum_core::apply(
        state,
        &Event::EnemyTokenPlaced {
            token: prowlers(),
            hex: GLADE,
        },
    );
    let events = events_of(&state, &end_turn_plain(&state));
    assert!(events.contains(&withdrawal(&[VILLAGE], &[wound(1)])));
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::EnemiesProvoked { .. }))
    );
}

// Spec 052 C24
#[test]
fn failed_assault_from_lake_then_forced_withdrawal() {
    // MKUE p. 10, 3.d ; FAQ p3 : la retraite d'assaut ne blesse pas, la retraite forcée suit
    // depuis le lac, d'une case.
    let state = countryside_11(HILLS_SE, true);
    let state = done(&state, move_to(LAKE_E));
    let state = done(&state, move_to(MAGE_TOWER));
    let state = skip_to_attack_phase(&done(&state, Action::StartCombat));
    let state = done(&state, Action::EndCombat);
    assert_eq!(state.figure(SOLO), Some(LAKE_E));
    let events = events_of(&state, &end_turn_plain(&state));
    assert!(events.contains(&withdrawal(&[HILLS_SE], &[wound(1)])));
}

// Spec 052 C25
#[test]
fn no_withdrawal_on_safe_space() {
    let march = card(March, 1);
    let state = on_village(&[march]);
    let state = done(&state, basic(march));
    let state = done(&state, move_to(PLAIN_W));
    let events = events_of(&state, &end_turn_plain(&state));
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::ForcedWithdrawal { .. }))
    );
}

// Spec 052 C26
#[test]
fn view_announces_forced_withdrawal() {
    let state = on_lake_from_village();
    assert_eq!(
        view(&state, SOLO).forced_withdrawal,
        Some(arcanum_core::ForcedWithdrawalPreview {
            to: VILLAGE,
            wounds: 1,
        })
    );
    // Sur une case sécurisée, rien ; pendant l'assaut, rien non plus.
    let state = countryside_11(HILLS_SE, true);
    assert_eq!(view(&state, SOLO).forced_withdrawal, None);
    let state = done(&state, move_to(LAKE_E));
    assert!(view(&state, SOLO).forced_withdrawal.is_some());
    let state = done(&state, move_to(MAGE_TOWER));
    assert_eq!(view(&state, SOLO).forced_withdrawal, None);
}

// Spec 052 C27
#[test]
fn safe_space_rules() {
    // MKUE p. 7, 7.d : accessible sans effet, pas un site fortifié non conquis.
    let mountain = Hex::new(3, -6);
    let mut events = Sandbox {
        next: vec![
            arcanum_core::TileId::Countryside(3),
            arcanum_core::TileId::Countryside(4),
        ],
        ..Sandbox::default()
    }
    .events();
    events.push(Event::TileRevealed {
        tile: arcanum_core::TileId::Countryside(3),
        position: KEEP_POSITION,
    });
    events.push(Event::TileRevealed {
        tile: arcanum_core::TileId::Countryside(4),
        position: MAGE_TOWER_POSITION,
    });
    let state = events
        .iter()
        .fold(GameState::default(), arcanum_core::apply);
    let safe = |state: &GameState, hex| {
        arcanum_core::is_safe_space(state.map(), hex, &state.conquered_sites())
    };
    assert!(safe(&state, PLAIN_W));
    assert!(!safe(&state, LAKE));
    assert!(!safe(&state, mountain));
    assert!(!safe(&state, KEEP));
    let state = arcanum_core::apply(
        state,
        &Event::SiteConquered {
            player: SOLO,
            hex: KEEP,
        },
    );
    assert!(safe(&state, KEEP));
}

// --- Fin du tour ---

/// La fin du tour sans défausse, Temporisation gardée ou non.
fn end_turn_keeping(steady_tempo: bool) -> Action {
    Action::EndTurn {
        discard: Vec::new(),
        reclaim: None,
        glade: None,
        steady_tempo,
    }
}

// Spec 052 C38
#[test]
fn intimidate_reputation_at_end_of_turn() {
    // Carte ; Compendium, *Intimidate* S1 : réputation −1 à la fin du tour, rien de plus sur X.
    let intimidate = advanced(Intimidate);
    let state = on_village(&[intimidate]);
    let state = done(&state, basic_option(intimidate, 0));
    let events = events_of(&state, &end_turn_plain(&state));
    assert!(events.contains(&Event::ReputationChanged {
        player: SOLO,
        change: -1,
    }));
    let after = events.iter().fold(state, arcanum_core::apply);
    assert_eq!(after.reputation(SOLO).value(), -1);
    // Sur la case X, la réputation ne bouge plus.
    let state = arcanum_core::apply(
        on_village(&[intimidate]),
        &Event::ReputationChanged {
            player: SOLO,
            change: -14,
        },
    );
    assert!(state.reputation(SOLO).is_x());
    let state = done(&state, basic_option(intimidate, 0));
    let state = done(&state, end_turn_plain(&state));
    assert!(state.reputation(SOLO).is_x());
}

// Spec 052 C39
#[test]
fn steady_tempo_under_deck() {
    // Carte : sous le deck au lieu de la défausse, si le deck n'est pas vide.
    let steady = advanced(SteadyTempo);
    let state = on_village(&[steady]);
    let state = done(&state, basic(steady));
    assert!(legal_actions(&state, SOLO).contains(&end_turn_keeping(true)));
    let events = events_of(&state, &end_turn_keeping(true));
    assert!(events.contains(&Event::SteadyTempoKept {
        player: SOLO,
        card: steady,
        top: false,
    }));
    let after = events.iter().fold(state, arcanum_core::apply);
    assert_eq!(after.deck(SOLO).last(), Some(&steady));
    assert!(!after.discard_pile(SOLO).contains(&steady));
    // Deck vide : garder n'est pas une action légale.
    let state = state_with_deck(vec![steady], 1);
    let state = done(&state, basic(steady));
    assert!(legal_actions(&state, SOLO).contains(&end_turn_keeping(false)));
    assert!(!legal_actions(&state, SOLO).contains(&end_turn_keeping(true)));
}

// Spec 052 C40
#[test]
fn strong_steady_tempo_on_top() {
    // Carte ; Compendium, *Steady Tempo* S2 : renforcée, sur le deck même vide, et piochée.
    let steady = advanced(SteadyTempo);
    let state = arcanum_core::apply(
        state_with_deck(vec![steady], 1),
        &Event::CrystalGained {
            player: SOLO,
            color: arcanum_core::BasicColor::Blue,
        },
    );
    let play = Action::PlayCard {
        card: steady,
        play: CardPlay::Strong {
            gain: None,
            extra_mana: None,
            option: 0,
            discard: None,
            mana: ManaPayment::Crystal,
            set_dice: [None, None],
            with: None,
        },
    };
    let state = done(&state, play);
    assert!(legal_actions(&state, SOLO).contains(&end_turn_keeping(true)));
    let events = events_of(&state, &end_turn_keeping(true));
    assert_events_in_order(
        &events,
        &[
            Event::SteadyTempoKept {
                player: SOLO,
                card: steady,
                top: true,
            },
            Event::TurnEnded { player: SOLO },
            Event::CardDrawn {
                player: SOLO,
                card: steady,
            },
        ],
    );
}

// Spec 052 C41
#[test]
fn end_of_turn_order() {
    // MKUE p. 11, 2 et 3 : retraite forcée, réputation, puis les cartes jouées.
    let intimidate = advanced(Intimidate);
    let steady = advanced(SteadyTempo);
    let state = on_village(&[frost_bridge(), intimidate, steady]);
    let state = done(&state, strong_option(frost_bridge(), BLUE_DIE, 0));
    let state = done(&state, basic_option(intimidate, 0));
    let state = done(&state, basic(steady));
    let state = done(&state, move_to(LAKE));
    assert!(legal_actions(&state, SOLO).contains(&end_turn_keeping(true)));
    let events = events_of(&state, &end_turn_keeping(true));
    assert_events_in_order(
        &events,
        &[
            withdrawal(&[VILLAGE], &[wound(1)]),
            Event::ReputationChanged {
                player: SOLO,
                change: -1,
            },
            Event::SteadyTempoKept {
                player: SOLO,
                card: steady,
                top: false,
            },
            Event::TurnEnded { player: SOLO },
        ],
    );
}

// --- Contenu ---

// Spec 052 C42
#[test]
fn first_lot_is_supported() {
    use arcanum_core::{BasicEffect, DeedCard};
    let supported: Vec<AdvancedAction> = AdvancedAction::ALL
        .into_iter()
        .filter(|&card| {
            let definition = DeedCard::Advanced(card)
                .definition()
                .expect("une action avancée a une définition");
            // Bouclier de glace a déjà son effet de base (spec 050) : seule compte la carte
            // entière.
            definition.basic != BasicEffect::Unsupported
                && definition.strong != BasicEffect::Unsupported
        })
        .collect();
    // Les quatre Projectiles l'étaient déjà (spec 050) ; la spec 052 en ajoute sept.
    let expected = vec![
        FireBolt,
        IceBolt,
        SwiftBolt,
        CrushingBolt,
        PathFinding,
        FrostBridge,
        Agility,
        Intimidate,
        SteadyTempo,
        Diplomacy,
        Ambush,
    ];
    // Spec 053 : d'autres cartes le sont devenues (C36) ; celles de la 052 le restent.
    assert!(expected.iter().all(|card| supported.contains(card)));
    assert!(foresters().unit.definition().abilities[0].is_supported());
}

// Spec 052 C21 (issue privée 217) : la vue expose l'état d'après la retraite forcée, calculé par le moteur.
#[test]
fn view_exposes_state_after_forced_withdrawal() {
    let state = frost_bridge_on(Sandbox {
        figure: Some(GLADE),
        hand: vec![frost_bridge()],
        ..Sandbox::default()
    });
    assert_eq!(view(&state, SOLO).after_withdrawal, None);
    let state = done(&state, move_to(LAKE));
    let before = view(&state, SOLO);
    let after = before.after_withdrawal.expect("vue d'après retraite");
    assert_eq!(after.figure, Some(GLADE));
    assert!(after.hand.contains(&wound(1)));
    assert_eq!(after.forced_withdrawal, None);
    assert_eq!(after.after_withdrawal, None);
}
