//! Scénarios de la spec 053 (actions avancées : combat, unités et soins) : modificateurs d'ennemis,
//! règle des résistances aux effets, cartes et capacités d'unités du lot.

use arcanum_core::{
    Action, AdvancedAction, BasicColor, BoostTarget, CardId, CardPlay, CombatEnemy, DamageTarget,
    DeedCard, Effect, ElementPoints, Enemy, EnemyModifier, EnemyModifierKind, EnemyToken, Event,
    GameRunner, GameState, HealingTarget, Hex, InProgress, ManaColor, ManaPayment, ModifierSource,
    MoveCost, SpaceContent, Terrain, TimeOfDay, UndoStatus, Unit, UnitCard, apply, legal_actions,
    move_cost, routes, view,
};

mod arene;
use arene::*;

use AdvancedAction::*;
use arcanum_core::BasicAction::{Concentration, March, Promise, Rage, Threaten};

// --- Décors ---

/// Le lac de la campagne 1, voisin du village (spec 052).
const LAKE: Hex = Hex::new(2, -4);

/// Un décor de la spec 053 : le bac à sable (Arythea) de jour ou de nuit, la figurine, la main, les
/// dés de la Source, les ennemis posés, les unités déjà recrutées (dans l'ordre) et celles de
/// l'offre, puis des événements de plus (cristaux, Blessures…).
#[derive(Clone)]
struct Decor {
    time: TimeOfDay,
    figure: Hex,
    hand: Vec<CardId>,
    dice: Vec<ManaColor>,
    placed: Vec<(EnemyToken, Hex)>,
    owned: Vec<UnitCard>,
    offer: Vec<UnitCard>,
    extra: Vec<Event>,
}

impl Default for Decor {
    fn default() -> Self {
        Self {
            time: TimeOfDay::Day,
            figure: PLAIN_W,
            hand: Vec::new(),
            dice: Sandbox::default().dice,
            placed: Vec::new(),
            owned: Vec::new(),
            offer: Vec::new(),
            extra: Vec::new(),
        }
    }
}

impl Decor {
    fn events(&self) -> Vec<Event> {
        let mut events = Sandbox {
            time: self.time,
            figure: Some(self.figure),
            hand: self.hand.clone(),
            placed: self.placed.clone(),
            dice: self.dice.clone(),
            ..Sandbox::default()
        }
        .events();
        events.extend(unit_events(&self.owned, &self.offer));
        events.extend(self.extra.iter().cloned());
        events
    }

    fn state(&self) -> GameState {
        self.events().iter().fold(GameState::default(), apply)
    }

    /// Le décor, puis le combat engagé contre les ennemis des orques ② (phase à distance).
    fn fighting(&self) -> GameState {
        let state = done(&self.state(), challenge(ORCS_2));
        done(&state, Action::StartCombat)
    }
}

fn unit(unit: Unit, copy: u8) -> UnitCard {
    UnitCard::new(unit, copy)
}

fn peasants() -> UnitCard {
    unit(Unit::Peasants, 1)
}

fn illusionists() -> UnitCard {
    unit(Unit::Illusionists, 1)
}

fn freezers() -> UnitCard {
    unit(Unit::AmotepFreezers, 1)
}

fn altem_mages() -> UnitCard {
    unit(Unit::AltemMages, 1)
}

fn altem_guardians() -> UnitCard {
    unit(Unit::AltemGuardians, 1)
}

fn ice_mages() -> EnemyToken {
    token(Enemy::IceMages, 1)
}

fn fire_mages() -> EnemyToken {
    token(Enemy::FireMages, 1)
}

fn crystal(color: BasicColor) -> Event {
    Event::CrystalGained {
        player: SOLO,
        color,
    }
}

/// L'activation de la capacité `ability`, option `option`, payée par `mana`, visant `enemy`.
fn activate(
    of: UnitCard,
    ability: u8,
    option: u8,
    mana: &[ManaPayment],
    enemy: Option<EnemyToken>,
) -> Action {
    Action::ActivateUnit {
        unit: of,
        ability,
        option,
        mana: mana.to_vec(),
        target: None,
        enemy,
        leadership: false,
    }
}

/// L'option `option` de l'effet renforcé de `card`, payée par `mana`, avec le mana en plus
/// `extra`.
fn strong_with(card: CardId, option: u8, mana: ManaPayment, extra: Option<ManaPayment>) -> Action {
    Action::PlayCard {
        card,
        play: CardPlay::Strong {
            gain: None,
            option,
            discard: None,
            mana,
            set_dice: [None, None],
            with: None,
            extra_mana: extra,
        },
    }
}

/// L'option `option` de l'effet de base de `card`, payée par `mana`.
fn basic_paid(card: CardId, option: u8, mana: ManaPayment) -> Action {
    Action::PlayCard {
        card,
        play: CardPlay::Basic {
            die: None,
            gain: None,
            with: None,
            option,
            discard: None,
            mana: Some(mana),
        },
    }
}

/// Concentration renforcée (dé vert) sur l'option `option` renforcée de `card`.
fn concentrated(concentration: CardId, card: CardId, option: u8) -> Action {
    Action::PlayCard {
        card: concentration,
        play: CardPlay::Strong {
            gain: None,
            option: 0,
            discard: None,
            mana: ManaPayment::Die(GREEN_DIE),
            set_dice: [None, None],
            with: Some(BoostTarget {
                card,
                option,
                discard: None,
                set_dice: [None, None],
            }),
            extra_mana: None,
        },
    }
}

#[track_caller]
fn enemy_of(state: &GameState, token: EnemyToken) -> CombatEnemy {
    state
        .combat(SOLO)
        .and_then(|combat| combat.enemy(token))
        .cloned()
        .unwrap_or_else(|| panic!("{token:?} hors du combat"))
}

/// Les points de l'attaque ou de la parade en cours.
fn in_progress_points(state: &GameState) -> ElementPoints {
    match state
        .combat(SOLO)
        .and_then(|combat| combat.in_progress.clone())
    {
        Some(InProgress::Attack { points, .. } | InProgress::Block { points, .. }) => points,
        None => panic!("ni attaque ni parade en cours"),
    }
}

fn wounds_in_hand(state: &GameState) -> usize {
    state
        .hand(SOLO)
        .iter()
        .filter(|card| card.card().is_wound())
        .count()
}

/// Les façons de jouer `card` parmi les actions légales.
fn plays_of(state: &GameState, card: CardId) -> Vec<CardPlay> {
    legal_actions(state, SOLO)
        .into_iter()
        .filter_map(|action| match action {
            Action::PlayCard { card: played, play }
            | Action::PlayCardOnUnit {
                card: played, play, ..
            } if played == card => Some(play),
            _ => None,
        })
        .collect()
}

fn only_sideways(plays: &[CardPlay]) -> bool {
    plays
        .iter()
        .all(|play| matches!(play, CardPlay::Sideways(_)))
}

/// Les ennemis visés par les activations légales de `of`.
fn enemy_targets(state: &GameState, of: UnitCard) -> Vec<EnemyToken> {
    legal_actions(state, SOLO)
        .into_iter()
        .filter_map(|action| match action {
            Action::ActivateUnit {
                unit,
                enemy: Some(enemy),
                ..
            } if unit == of => Some(enemy),
            _ => None,
        })
        .collect()
}

fn armor_reduction(card: CardId) -> EnemyModifier {
    EnemyModifier {
        kind: EnemyModifierKind::ArmorReduction { by: 3, min: 1 },
        source: ModifierSource::Card(card),
    }
}

/// La fin du tour sans défausse, si elle est légale.
fn end_turn_plain(state: &GameState) -> Action {
    legal_actions(state, SOLO)
        .into_iter()
        .find(|action| matches!(action, Action::EndTurn { discard, reclaim: None, glade: None, .. } if discard.is_empty()))
        .expect("une fin de tour sans défausse")
}

/// Un combat contre `enemies` aux orques ②, avec Bouclier de glace en main, en parade contre
/// `blocked`, l'effet renforcé joué (dé bleu).
fn ice_shield_block(enemies: &[EnemyToken], blocked: EnemyToken) -> (GameState, CardId) {
    let ice_shield = advanced(IceShield);
    let state = Decor {
        hand: vec![ice_shield, card(Rage, 1)],
        placed: enemies.iter().map(|&enemy| (enemy, ORCS_2)).collect(),
        ..Decor::default()
    }
    .fighting();
    let state = done(&to_block_phase(&state), declare_block(blocked));
    (done(&state, strong(ice_shield, BLUE_DIE)), ice_shield)
}

// --- Résistances aux effets ---

// Spec 053 C1
#[test]
fn ice_resistance_ignores_blue_effects() {
    // MKUE p. 8, *Combat with Enemies* 5.b ; Compendium, *Ice Shield* S1.
    let (state, _) = ice_shield_block(&[ice_mages()], ice_mages());
    let mages = enemy_of(&state, ice_mages());
    assert!(mages.modifiers.is_empty());
    assert_eq!(mages.effective_armor(), 6);
    // La Parade de glace compte, divisée par deux contre une attaque de glace (spec 032).
    assert_eq!(in_progress_points(&state).ice, 3);
    assert_eq!(
        state.combat(SOLO).and_then(|c| c.in_progress_total()),
        Some(1)
    );
}

// Spec 053 C2
#[test]
fn freezers_blocked_by_ice_resistance_only() {
    // MKUE p. 8, 5.b ; Compendium, *Amotep Freezers* S1.
    for (enemy, resisted) in [(ice_mages(), true), (fire_mages(), false)] {
        let state = Decor {
            placed: vec![(enemy, ORCS_2)],
            owned: vec![freezers()],
            ..Decor::default()
        }
        .fighting();
        let state = done(
            &state,
            activate(freezers(), 1, 0, &[ManaPayment::Die(BLUE_DIE)], Some(enemy)),
        );
        let target = enemy_of(&state, enemy);
        assert_eq!(target.modifiers.is_empty(), resisted, "{enemy:?}");
        assert_eq!(target.does_not_attack(), !resisted, "{enemy:?}");
    }
}

// Spec 053 C3
#[test]
fn boosted_card_keeps_its_color() {
    // FAQ p2 ; Compendium, *Combat* Q7 : Bouclier de glace joué par Concentration reste bleu.
    let ice_shield = advanced(IceShield);
    let concentration = card(Concentration, 1);
    let state = Decor {
        hand: vec![concentration, ice_shield],
        placed: vec![(fire_mages(), ORCS_2)],
        ..Decor::default()
    }
    .fighting();
    let state = done(&to_block_phase(&state), declare_block(fire_mages()));
    let state = done(&state, concentrated(concentration, ice_shield, 0));
    assert_eq!(enemy_of(&state, fire_mages()).effective_armor(), 2);
}

// Spec 053 C4
#[test]
fn effects_on_units_ignore_enemy_resistances() {
    // MKUE p. 8, 5.c : Au cœur du combat (rouge) ne vise pas un ennemi.
    let into_the_heat = advanced(IntoTheHeat);
    let state = Decor {
        hand: vec![into_the_heat],
        placed: vec![(fire_mages(), ORCS_2)],
        owned: vec![peasants()],
        ..Decor::default()
    }
    .fighting();
    let state = done(&state, basic(into_the_heat));
    let state = done(&to_block_phase(&state), declare_block(fire_mages()));
    let state = done(&state, activate(peasants(), 0, 1, &[], None));
    assert_eq!(in_progress_points(&state), physical(4));
}

// --- Modificateurs d'ennemis ---

// Spec 053 C5
#[test]
fn reduced_armor_counts_in_attack() {
    let (state, _) = ice_shield_block(&[prowlers()], prowlers());
    let state = done(&state, Action::ResolveBlock);
    let state = skip_to_attack_phase(&state);
    let state = done(&state, declare(&[prowlers()]));
    let state = done(&state, sideways_attack(card(Rage, 1)));
    let events = events_of(&state, &Action::ResolveAttack);
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::EnemiesDefeated { .. })),
        "{events:?}"
    );
}

// Spec 053 C6
#[test]
fn armor_reductions_add_up_to_minimum_one() {
    let ice_shield = advanced(IceShield);
    let state = Decor {
        hand: vec![ice_shield],
        placed: vec![(guardsmen(), ORCS_2)],
        owned: vec![freezers()],
        extra: vec![crystal(BasicColor::Blue)],
        ..Decor::default()
    }
    .fighting();
    let state = done(&to_block_phase(&state), declare_block(guardsmen()));
    let state = done(&state, strong(ice_shield, BLUE_DIE));
    assert_eq!(enemy_of(&state, guardsmen()).effective_armor(), 4);
    let state = skip_to_attack_phase(&done(&state, Action::ResolveBlock));
    let state = done(
        &state,
        activate(freezers(), 1, 0, &[ManaPayment::Crystal], Some(guardsmen())),
    );
    let guards = enemy_of(&state, guardsmen());
    assert_eq!(guards.effective_armor(), 1);
    assert!(guards.is_fortified());
}

// Spec 053 C7
#[test]
fn enemy_that_does_not_attack() {
    // Compendium, *Ice Shield* S4 ; *Illusionists* Q1.
    let state = Decor {
        placed: vec![(prowlers(), ORCS_2)],
        owned: vec![illusionists()],
        ..Decor::default()
    }
    .fighting();
    let state = done(
        &state,
        activate(
            illusionists(),
            1,
            0,
            &[ManaPayment::Die(WHITE_DIE)],
            Some(prowlers()),
        ),
    );
    let state = to_block_phase(&state);
    assert!(block_targets(&state).is_empty());
    let events = events_of(&state, &Action::NextCombatPhase);
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::WoundsTaken { .. })),
        "{events:?}"
    );
    let state = events.iter().fold(state, apply);
    assert!(!enemy_of(&state, prowlers()).defeated);
    assert_eq!(attack_targets(&state), vec![vec![ORCS_2]]);
}

// Spec 053 C8
#[test]
fn enemy_modifiers_follow_combat_undo_and_reload() {
    // ADR 0008 ; spec 024.
    let decor = Decor {
        placed: vec![(prowlers(), ORCS_2)],
        owned: vec![illusionists()],
        ..Decor::default()
    };
    let (mut runner, puppet) = runner(decor.events());
    play(&mut runner, &puppet, challenge(ORCS_2));
    play(&mut runner, &puppet, Action::StartCombat);
    play(
        &mut runner,
        &puppet,
        activate(
            illusionists(),
            1,
            0,
            &[ManaPayment::Die(WHITE_DIE)],
            Some(prowlers()),
        ),
    );
    assert!(enemy_of(runner.state(), prowlers()).does_not_attack());
    let reloaded = GameRunner::resume(
        runner.log().clone(),
        runner.rng_positions(),
        vec![Box::new(Puppet::default())],
    )
    .expect("journal valide");
    assert!(enemy_of(reloaded.state(), prowlers()).does_not_attack());
    assert_eq!(runner.undo_status(), UndoStatus::Possible);
    assert_eq!(runner.undo(), Ok(()));
    assert!(enemy_of(runner.state(), prowlers()).modifiers.is_empty());
    // La fin du combat les efface.
    let state = done(&reloaded.state().clone(), Action::NextCombatPhase);
    let state = done(&state, Action::NextCombatPhase);
    let state = done(&state, Action::EndCombat);
    assert!(enemy_of(&state, prowlers()).modifiers.is_empty());
}

// --- Coup de sang ---

/// Phase d'attaque contre les Rôdeurs, attaque déclarée, Coup de sang en main avec `others`.
fn blood_rage_attack(others: &[CardId], extra: Vec<Event>) -> (GameState, CardId) {
    let blood_rage = advanced(BloodRage);
    let mut hand = vec![blood_rage];
    hand.extend_from_slice(others);
    let state = Decor {
        hand,
        placed: vec![(prowlers(), ORCS_2)],
        ..Decor::default()
    }
    .fighting();
    let state = extra.iter().fold(state, apply);
    let state = skip_to_attack_phase(&state);
    (done(&state, declare(&[prowlers()])), blood_rage)
}

// Spec 053 C9
#[test]
fn blood_rage_wound_for_attack() {
    // Compendium, *Blood Rage* S1.
    let (state, blood_rage) = blood_rage_attack(&[], Vec::new());
    let events = events_of(&state, &option(blood_rage, 1));
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::WoundFromCard { card, .. } if *card == blood_rage)),
        "{events:?}"
    );
    let state = events.iter().fold(state, apply);
    assert_eq!(in_progress_points(&state), physical(5));
    assert_eq!(wounds_in_hand(&state), 1);
}

// Spec 053 C10
#[test]
fn blood_rage_knock_out_keeps_attack() {
    // MKUE p. 9, 5.c ; Compendium, *Combat* S21.
    let march = card(March, 1);
    let rage = card(Rage, 1);
    let four = Event::WoundsTaken {
        player: SOLO,
        enemy: prowlers(),
        wounds: (1..=4).map(wound).collect(),
    };
    let (state, blood_rage) = blood_rage_attack(&[march, rage], vec![four]);
    let events = events_of(&state, &option(blood_rage, 1));
    assert!(events.contains(&Event::KnockedOut {
        player: SOLO,
        cards: vec![march, rage],
    }));
    let state = events.iter().fold(state, apply);
    assert_eq!(in_progress_points(&state), physical(5));
    assert!(state.hand(SOLO).iter().all(|card| card.card().is_wound()));
}

// Spec 053 C11
#[test]
fn concentration_boosts_blood_rage() {
    // MKUE p. 5, *"Composite" Effects* 1.b.
    let concentration = card(Concentration, 1);
    let (state, blood_rage) = blood_rage_attack(&[concentration], Vec::new());
    let state = done(&state, concentrated(concentration, blood_rage, 1));
    assert_eq!(in_progress_points(&state), physical(11));
    assert_eq!(wounds_in_hand(&state), 1);
}

// --- Bouclier de glace renforcé ---

// Spec 053 C12
#[test]
fn ice_shield_reduces_armor_even_if_block_fails() {
    // Compendium, *Ice Shield* Q2.
    let (state, _) = ice_shield_block(&[prowlers()], prowlers());
    assert_eq!(enemy_of(&state, prowlers()).effective_armor(), 1);
    let events = events_of(&state, &Action::ResolveBlock);
    assert_eq!(events, vec![Event::BlockFailed { player: SOLO }]);
    let state = events.iter().fold(state, apply);
    assert_eq!(enemy_of(&state, prowlers()).effective_armor(), 1);
}

// Spec 053 C13
#[test]
fn ice_shield_on_summoned_monster() {
    // Compendium, *Ice Shield* S3 : la créature, pas l'Invocateur.
    let (state, _) = ice_shield_block(&[orc_summoners()], orc_summoners());
    let summoners = enemy_of(&state, orc_summoners());
    assert!(summoners.summoned.is_some());
    assert!(summoners.modifiers.is_empty());
    assert_eq!(summoners.effective_armor(), 4);
}

// --- Marche revigorante et Régénération ---

// Spec 053 C14
#[test]
fn refreshing_walk_moves_and_heals() {
    let walk = advanced(RefreshingWalk);
    let state = Decor {
        hand: vec![walk, wound(1)],
        ..Decor::default()
    }
    .state();
    let state = done(&state, basic(walk));
    assert_eq!(state.move_points(SOLO), 2);
    assert_eq!(state.healing_points(SOLO), 1);
    let state = done(
        &state,
        Action::SpendHealing {
            on: HealingTarget::Wound,
        },
    );
    assert_eq!(wounds_in_hand(&state), 0);
}

// Spec 053 C15
#[test]
fn refreshing_walk_in_combat_moves_only() {
    // Carte (LL) ; MKUE p. 8, 6.
    let walk = advanced(RefreshingWalk);
    let agility = advanced(Agility);
    let state = Decor {
        hand: vec![agility, walk],
        placed: vec![(prowlers(), ORCS_2)],
        ..Decor::default()
    }
    .state();
    let state = done(&state, basic(agility));
    let state = done(&done(&state, challenge(ORCS_2)), Action::StartCombat);
    let before = state.move_points(SOLO);
    let state = done(&state, strong(walk, GREEN_DIE));
    assert_eq!(state.move_points(SOLO), before + 4);
    assert_eq!(state.healing_points(SOLO), 0);
}

// Spec 053 C16
#[test]
fn refreshing_walk_after_combat_heals() {
    // Compendium, *Refreshing Walk* S3.
    let walk = advanced(RefreshingWalk);
    let state = Decor {
        hand: vec![walk],
        placed: vec![(prowlers(), ORCS_2)],
        ..Decor::default()
    }
    .fighting();
    let state = done(&skip_to_attack_phase(&state), Action::EndCombat);
    let state = done(&state, basic(walk));
    assert_eq!(state.healing_points(SOLO), 1);
}

/// Les unités `owned` recrutées puis épuisées, dans un décor au village.
fn exhausted(owned: &[UnitCard], hand: &[CardId], placed: &[(EnemyToken, Hex)]) -> Decor {
    Decor {
        hand: hand.to_vec(),
        owned: owned.to_vec(),
        placed: placed.to_vec(),
        extra: owned
            .iter()
            .map(|&unit| Event::UnitActivated {
                player: SOLO,
                unit,
                ability: 0,
                option: 0,
                effect: Effect::Nothing,
                enemy: None,
            })
            .collect(),
        ..Decor::default()
    }
}

// Spec 053 C17
#[test]
fn regeneration_readies_and_heals() {
    let regeneration = advanced(Regeneration);
    let state = exhausted(&[peasants(), freezers()], &[regeneration], &[]).state();
    let targets = |state: &GameState, strong: bool| -> Vec<UnitCard> {
        legal_actions(state, SOLO)
            .into_iter()
            .filter_map(|action| match action {
                Action::PlayCardOnUnit { card, play, target } if card == regeneration => {
                    (matches!(play, CardPlay::Strong { .. }) == strong).then_some(target)
                }
                _ => None,
            })
            .collect()
    };
    assert_eq!(targets(&state, false), vec![peasants()]);
    assert!(targets(&state, true).contains(&freezers()));
    let state = done(
        &state,
        Action::PlayCardOnUnit {
            card: regeneration,
            play: CardPlay::Basic {
                die: None,
                gain: None,
                with: None,
                option: 0,
                discard: None,
                mana: None,
            },
            target: peasants(),
        },
    );
    assert!(state.units(SOLO)[0].ready);
    assert_eq!(state.healing_points(SOLO), 1);
}

// Spec 053 C18
#[test]
fn regeneration_not_in_combat() {
    // MKUE p. 8, 6 ; Q4.
    let regeneration = advanced(Regeneration);
    let decor = exhausted(&[peasants()], &[regeneration], &[(prowlers(), ORCS_2)]);
    let state = decor.fighting();
    assert!(only_sideways(&plays_of(&state, regeneration)));
}

// --- Mélodie du vent ---

fn cost(state: &GameState, terrain: Terrain) -> MoveCost {
    move_cost(
        SpaceContent {
            terrain,
            feature: None,
        },
        state.move_rules(SOLO),
    )
}

// Spec 053 C19
#[test]
fn song_of_wind_reduces_open_terrains() {
    let song = advanced(SongOfWind);
    let decor = Decor {
        hand: vec![song],
        ..Decor::default()
    };
    let state = done(&decor.state(), basic(song));
    assert_eq!(cost(&state, Terrain::Plains), MoveCost::Points(1));
    assert_eq!(cost(&state, Terrain::Desert), MoveCost::Points(4));
    assert_eq!(cost(&state, Terrain::Wasteland), MoveCost::Points(3));
    let state = done(&decor.state(), strong(song, WHITE_DIE));
    assert_eq!(cost(&state, Terrain::Plains), MoveCost::Points(0));
    assert_eq!(cost(&state, Terrain::Desert), MoveCost::Points(3));
    assert_eq!(cost(&state, Terrain::Wasteland), MoveCost::Points(2));
}

// Spec 053 C20
#[test]
fn song_of_wind_with_path_finding() {
    // Compendium, *Song of Wind* S4 : l'ordre le plus favorable (spec 052).
    let song = advanced(SongOfWind);
    let path_finding = advanced(PathFinding);
    let state = Decor {
        hand: vec![song, path_finding],
        ..Decor::default()
    }
    .state();
    for order in [[song, path_finding], [path_finding, song]] {
        let state = order
            .iter()
            .fold(state.clone(), |state, &card| done(&state, basic(card)));
        assert_eq!(cost(&state, Terrain::Plains), MoveCost::Points(1));
    }
}

// Spec 053 C21
#[test]
fn strong_song_of_wind_crosses_lakes() {
    // Carte ; spec 052 ; Q3 : le mana bleu payé en jouant la carte.
    let song = advanced(SongOfWind);
    let decor = Decor {
        figure: VILLAGE,
        hand: vec![song],
        extra: vec![crystal(BasicColor::Blue)],
        ..Decor::default()
    };
    let lakes = strong_with(
        song,
        1,
        ManaPayment::Die(WHITE_DIE),
        Some(ManaPayment::Crystal),
    );
    let state = done(&decor.state(), lakes);
    assert_eq!(cost(&state, Terrain::Lake), MoveCost::Points(0));
    assert!(
        routes(state.map(), VILLAGE, state.move_rules(SOLO))
            .iter()
            .any(|route| route.to == LAKE)
    );
    // Sans mana bleu, l'option des lacs n'est pas légale.
    let without = Decor {
        extra: Vec::new(),
        ..decor
    }
    .state();
    assert!(
        !plays_of(&without, song)
            .iter()
            .any(|play| matches!(play, CardPlay::Strong { option: 1, .. }))
    );
}

// --- Magie pure ---

// Spec 053 C22
#[test]
fn pure_magic_option_by_paid_color() {
    // Compendium, *Pure Magic* S2.
    let pure_magic = advanced(PureMagic);
    let state = Decor {
        hand: vec![pure_magic],
        ..Decor::default()
    }
    .state();
    let block = basic_paid(pure_magic, 2, ManaPayment::Die(BLUE_DIE));
    assert!(!legal_actions(&state, SOLO).contains(&block));
    let state = done(
        &state,
        basic_paid(pure_magic, 0, ManaPayment::Die(GREEN_DIE)),
    );
    assert_eq!(state.move_points(SOLO), 4);
}

// Spec 053 C23
#[test]
fn strong_pure_magic_two_mana() {
    // Compendium, *Pure Magic* S2 ; MKUE p. 5, 2 : l'or de jour vaut une couleur de base.
    let pure_magic = advanced(PureMagic);
    let decor = Decor {
        hand: vec![pure_magic],
        placed: vec![(prowlers(), ORCS_2)],
        extra: vec![crystal(BasicColor::Blue)],
        ..Decor::default()
    };
    let state = done(
        &to_block_phase(&decor.fighting()),
        declare_block(prowlers()),
    );
    let state = done(
        &state,
        strong_with(
            pure_magic,
            2,
            ManaPayment::Die(BLUE_DIE),
            Some(ManaPayment::Crystal),
        ),
    );
    assert_eq!(in_progress_points(&state), physical(7));
    let gold = Decor {
        dice: vec![
            ManaColor::Gold,
            ManaColor::Basic(BasicColor::Green),
            ManaColor::Basic(BasicColor::White),
        ],
        ..decor
    };
    let state = skip_to_attack_phase(&gold.fighting());
    let state = done(&state, declare(&[prowlers()]));
    let state = done(
        &state,
        strong_with(
            pure_magic,
            3,
            ManaPayment::Crystal,
            Some(ManaPayment::Die(0)),
        ),
    );
    assert_eq!(in_progress_points(&state), physical(7));
}

// --- Récit héroïque et Besoin d'aide ---

/// La réputation gagnée par la fin du tour.
fn end_turn_reputation(state: &GameState) -> (i32, u32) {
    let events = events_of(state, &end_turn_plain(state));
    let reputation = events
        .iter()
        .map(|event| match event {
            Event::ReputationChanged { change, .. } => i32::from(*change),
            _ => 0,
        })
        .sum();
    let fame = events
        .iter()
        .map(|event| match event {
            Event::FameGained { fame, .. } => *fame,
            _ => 0,
        })
        .sum();
    (reputation, fame)
}

fn heroic_tale_village(offer: &[UnitCard], hand: &[CardId]) -> GameState {
    let state = Decor {
        figure: VILLAGE,
        hand: hand.to_vec(),
        offer: offer.to_vec(),
        ..Decor::default()
    }
    .state();
    done(&state, Action::Interact)
}

fn recruit(unit: UnitCard, disband: Option<UnitCard>) -> Action {
    Action::RecruitUnit {
        unit,
        disband,
        bonds: false,
    }
}

// Spec 053 C24
#[test]
fn heroic_tale_per_recruit() {
    let heroic_tale = advanced(HeroicTale);
    let promise = card(Promise, 1);
    let state = heroic_tale_village(&[peasants()], &[heroic_tale, promise]);
    let state = done(&done(&state, basic(heroic_tale)), basic(promise));
    let state = done(&state, recruit(peasants(), None));
    assert_eq!(end_turn_reputation(&state), (1, 0));
    let state = heroic_tale_village(&[peasants()], &[heroic_tale]);
    let state = done(&state, strong(heroic_tale, WHITE_DIE));
    let state = done(&state, recruit(peasants(), None));
    assert_eq!(end_turn_reputation(&state), (1, 1));
}

// Spec 053 C25
#[test]
fn heroic_tale_counts_all_recruits() {
    // Compendium, *Heroic Tale* S1.
    let heroic_tale = advanced(HeroicTale);
    let promise = card(Promise, 1);
    let threaten = card(Threaten, 1);
    let march = card(March, 1);
    let second = unit(Unit::Peasants, 2);
    let state = heroic_tale_village(
        &[peasants(), second],
        &[heroic_tale, promise, threaten, march],
    );
    let state = done(&done(&state, basic(promise)), basic(threaten));
    let state = done(&state, recruit(peasants(), None));
    let state = done(&state, basic(heroic_tale));
    let state = done(
        &state,
        Action::PlayCard {
            card: march,
            play: CardPlay::Sideways(arcanum_core::SidewaysEffect::Influence),
        },
    );
    let state = done(&state, recruit(second, Some(peasants())));
    assert_eq!(end_turn_reputation(&state), (2, 0));
}

// Spec 053 C26
#[test]
fn in_need_counts_wounds() {
    let in_need = advanced(InNeed);
    let decor = Decor {
        hand: vec![in_need, wound(1), wound(2)],
        owned: vec![peasants()],
        extra: vec![Event::UnitWounded {
            player: SOLO,
            unit: peasants(),
            wounds: 1,
        }],
        ..Decor::default()
    };
    let state = done(&decor.state(), basic(in_need));
    assert_eq!(state.influence_points(SOLO), 6);
    let state = done(&decor.state(), strong(in_need, GREEN_DIE));
    assert_eq!(state.influence_points(SOLO), 11);
}

// --- Au cœur du combat ---

// Spec 053 C27
#[test]
fn into_the_heat_boosts_unit_attack_and_block() {
    // FAQ p2 ; Compendium, *Into the Heat* S1.
    let into_the_heat = advanced(IntoTheHeat);
    let decor = Decor {
        hand: vec![into_the_heat],
        placed: vec![(prowlers(), ORCS_2)],
        owned: vec![peasants()],
        ..Decor::default()
    };
    let state = done(&decor.fighting(), basic(into_the_heat));
    let attack = done(&skip_to_attack_phase(&state), declare(&[prowlers()]));
    let attack = done(&attack, activate(peasants(), 0, 0, &[], None));
    assert_eq!(in_progress_points(&attack), physical(4));
    let block = done(&to_block_phase(&state), declare_block(prowlers()));
    let block = done(&block, activate(peasants(), 0, 1, &[], None));
    assert_eq!(in_progress_points(&block), physical(4));
}

// Spec 053 C28
#[test]
fn into_the_heat_only_at_start() {
    // Carte ; Q6.
    let into_the_heat = advanced(IntoTheHeat);
    let decor = Decor {
        hand: vec![into_the_heat],
        placed: vec![(prowlers(), ORCS_2)],
        owned: vec![peasants()],
        ..Decor::default()
    };
    let state = decor.fighting();
    assert!(!only_sideways(&plays_of(&state, into_the_heat)));
    let state = done(&to_block_phase(&state), declare_block(prowlers()));
    let state = done(&state, activate(peasants(), 0, 1, &[], None));
    assert!(only_sideways(&plays_of(&state, into_the_heat)));
    // Sans unité.
    let state = Decor {
        owned: Vec::new(),
        ..decor
    }
    .fighting();
    assert!(only_sideways(&plays_of(&state, into_the_heat)));
}

// Spec 053 C29
#[test]
fn into_the_heat_no_damage_to_units() {
    // Carte ; MKUE p. 9, *Assign Damage Phase* 3 et 5.
    let into_the_heat = advanced(IntoTheHeat);
    let state = Decor {
        hand: vec![into_the_heat],
        placed: vec![(prowlers(), ORCS_2)],
        owned: vec![peasants()],
        ..Decor::default()
    }
    .fighting();
    let state = to_block_phase(&done(&state, basic(into_the_heat)));
    let events = events_of(&state, &Action::NextCombatPhase);
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::WoundsTaken { .. })),
        "{events:?}"
    );
    let state = events.iter().fold(state, apply);
    assert!(
        !legal_actions(&state, SOLO)
            .iter()
            .any(|action| matches!(action, Action::AssignDamage { .. }))
    );
}

// --- Capacités d'unités ---

// Spec 053 C30
#[test]
fn illusionists_cancel_unfortified_attack() {
    // Carte ; Compendium, *Illusionists* S2.
    let state = Decor {
        placed: vec![(orc_summoners(), ORCS_2), (diggers(), ORCS_2)],
        owned: vec![illusionists()],
        ..Decor::default()
    }
    .fighting();
    assert_eq!(enemy_targets(&state, illusionists()), vec![orc_summoners()]);
    let state = done(
        &state,
        activate(
            illusionists(),
            1,
            0,
            &[ManaPayment::Die(WHITE_DIE)],
            Some(orc_summoners()),
        ),
    );
    let events = events_of(&state, &Action::NextCombatPhase);
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::EnemySummoned { .. })),
        "{events:?}"
    );
}

// Spec 053 C31
#[test]
fn freezers_reduce_armor_in_attack_phase() {
    // Carte ; Q7.
    let state = Decor {
        placed: vec![(guardsmen(), ORCS_2)],
        owned: vec![freezers()],
        ..Decor::default()
    }
    .fighting();
    let state = skip_to_attack_phase(&state);
    let state = done(
        &state,
        activate(
            freezers(),
            1,
            0,
            &[ManaPayment::Die(BLUE_DIE)],
            Some(guardsmen()),
        ),
    );
    assert_eq!(enemy_of(&state, guardsmen()).effective_armor(), 4);
}

/// Un combat de nuit contre `enemy`, dé noir en tête de la Source, Mages d'Altem et Paysans
/// recrutés, Rage en main.
fn altem_night(enemy: EnemyToken) -> GameState {
    Decor {
        time: TimeOfDay::Night,
        hand: vec![card(Rage, 1)],
        dice: vec![
            ManaColor::Black,
            ManaColor::Basic(BasicColor::Green),
            ManaColor::Basic(BasicColor::Red),
        ],
        placed: vec![(enemy, ORCS_2)],
        owned: vec![altem_mages(), peasants()],
        ..Decor::default()
    }
    .fighting()
}

// Spec 053 C32
#[test]
fn altem_mages_cold_fire_not_retroactive() {
    // Compendium, *Altem Mages* Q2 ; MKUE p. 9, 4.a.
    let state = skip_to_attack_phase(&altem_night(fire_mages()));
    let state = done(&state, declare(&[fire_mages()]));
    let state = done(&state, option(card(Rage, 1), 0));
    let state = done(
        &state,
        activate(altem_mages(), 2, 0, &[ManaPayment::Die(0)], None),
    );
    let state = done(&state, activate(peasants(), 0, 0, &[], None));
    assert_eq!(
        in_progress_points(&state),
        ElementPoints {
            physical: 2,
            cold_fire: 2,
            ..ElementPoints::default()
        }
    );
    assert_eq!(
        state.combat(SOLO).and_then(|c| c.in_progress_total()),
        Some(4)
    );
}

// Spec 053 C33
#[test]
fn altem_mages_siege_in_ranged_phase() {
    // Compendium, *Altem Mages* S7.
    let state = altem_night(diggers());
    let state = done(
        &state,
        activate(altem_mages(), 2, 1, &[ManaPayment::Die(0)], None),
    );
    let state = done(&state, declare(&[diggers()]));
    let state = done(&state, option(card(Rage, 1), 0));
    assert_eq!(in_progress_points(&state), physical(2));
}

// Spec 053 C34
#[test]
fn altem_mages_black_only_at_night() {
    // MKUE p. 5 : le mana noir, de nuit seulement.
    let day = Decor {
        dice: vec![
            ManaColor::Black,
            ManaColor::Basic(BasicColor::Green),
            ManaColor::Basic(BasicColor::Red),
        ],
        placed: vec![(prowlers(), ORCS_2)],
        owned: vec![altem_mages()],
        ..Decor::default()
    }
    .fighting();
    assert!(
        !legal_actions(&day, SOLO)
            .iter()
            .any(|action| matches!(action, Action::ActivateUnit { ability: 2, .. }))
    );
    let night = altem_night(prowlers());
    assert!(
        legal_actions(&night, SOLO)
            .iter()
            .any(|action| matches!(action, Action::ActivateUnit { ability: 2, .. }))
    );
}

// Spec 053 C35
#[test]
fn altem_guardians_give_all_resistances() {
    // MKUE p. 9, 4.b.
    let state = Decor {
        placed: vec![(wolf_riders(), ORCS_2)],
        owned: vec![altem_guardians(), peasants()],
        ..Decor::default()
    }
    .fighting();
    let state = done(
        &state,
        activate(
            altem_guardians(),
            2,
            0,
            &[ManaPayment::Die(GREEN_DIE)],
            None,
        ),
    );
    let state = done(&to_block_phase(&state), Action::NextCombatPhase);
    let events = events_of(
        &state,
        &Action::AssignDamage {
            enemy: wolf_riders(),
            to: DamageTarget::Unit(peasants()),
        },
    );
    assert!(events.contains(&Event::DamageAssignedToUnit {
        player: SOLO,
        enemy: wolf_riders(),
        unit: peasants(),
        wounds: 0,
    }));
}

// --- Contenu et vue ---

// Spec 053 C36
#[test]
fn second_lot_is_supported() {
    let unsupported: Vec<AdvancedAction> = AdvancedAction::ALL
        .into_iter()
        .filter(|&card| {
            let definition = DeedCard::Advanced(card)
                .definition()
                .expect("une action avancée a une définition");
            definition.basic == arcanum_core::BasicEffect::Unsupported
                || definition.strong == arcanum_core::BasicEffect::Unsupported
        })
        .collect();
    // Spec 056 : le second lot quitte `Unsupported` carte par carte ; seules ses cartes peuvent
    // encore l'être (le contrôle complet est `advanced_actions_all_supported`, spec 056 C32).
    let second_lot = [
        BloodRitual,
        Decompose,
        CrystalMastery,
        ManaStorm,
        MaximalEffect,
        MagicTalent,
        Learning,
        Training,
    ];
    assert!(
        unsupported.iter().all(|card| second_lot.contains(card)),
        "{unsupported:?}"
    );
    assert!(Unit::ALL.into_iter().all(|unit| {
        unit.definition()
            .abilities
            .iter()
            .all(|ability| ability.is_supported())
    }));
}

// Spec 053 C37
#[test]
fn view_shows_enemy_modifiers() {
    let (state, ice_shield) = ice_shield_block(&[prowlers()], prowlers());
    let seen = view(&state, SOLO);
    let enemy = &seen.combat.as_ref().expect("combat public").enemies[0];
    assert_eq!(enemy.modifiers, vec![armor_reduction(ice_shield)]);
    assert_eq!(enemy.effective_armor(), 1);
}

// --- Cas limites ---

// Spec 053 L1
#[test]
fn strong_ice_shield_needs_a_block_in_progress() {
    let ice_shield = advanced(IceShield);
    let state = Decor {
        hand: vec![ice_shield],
        placed: vec![(prowlers(), ORCS_2)],
        ..Decor::default()
    }
    .fighting();
    let state = done(&skip_to_attack_phase(&state), declare(&[prowlers()]));
    refused(&state, strong(ice_shield, BLUE_DIE));
}

// Spec 053 L2
#[test]
fn illusionists_need_a_target_that_still_attacks() {
    let state = Decor {
        hand: vec![card(Rage, 1)],
        placed: vec![(prowlers(), ORCS_2)],
        owned: vec![illusionists(), peasants()],
        ..Decor::default()
    }
    .fighting();
    let white = [ManaPayment::Die(WHITE_DIE)];
    refused(&state, activate(illusionists(), 1, 0, &white, None));
    let state = done(&to_block_phase(&state), declare_block(prowlers()));
    let state = done(&state, activate(peasants(), 0, 1, &[], None));
    let state = done(&state, option(card(Rage, 1), 1));
    let state = done(&state, Action::ResolveBlock);
    assert!(enemy_of(&state, prowlers()).blocked);
    refused(
        &state,
        activate(illusionists(), 1, 0, &white, Some(prowlers())),
    );
}

// Spec 053 L3
#[test]
fn strong_pure_magic_needs_extra_mana() {
    let pure_magic = advanced(PureMagic);
    let state = Decor {
        hand: vec![pure_magic],
        ..Decor::default()
    }
    .state();
    refused(
        &state,
        strong_with(pure_magic, 0, ManaPayment::Die(BLUE_DIE), None),
    );
}

// Spec 053 L4
#[test]
fn two_illusionists_on_two_enemies() {
    let second = unit(Unit::Illusionists, 2);
    let state = Decor {
        placed: vec![(prowlers(), ORCS_2), (wolf_riders(), ORCS_2)],
        owned: vec![illusionists(), second],
        extra: vec![crystal(BasicColor::White)],
        ..Decor::default()
    }
    .fighting();
    let state = done(
        &state,
        activate(
            illusionists(),
            1,
            0,
            &[ManaPayment::Die(WHITE_DIE)],
            Some(prowlers()),
        ),
    );
    let state = done(
        &state,
        activate(second, 1, 0, &[ManaPayment::Crystal], Some(wolf_riders())),
    );
    assert!(enemy_of(&state, prowlers()).does_not_attack());
    assert!(enemy_of(&state, wolf_riders()).does_not_attack());
}
