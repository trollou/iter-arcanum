//! Scénarios de la spec 054 (sorts : lancer un sort, sorts de combat) : paiement d'un sort, ennemi
//! visé, modificateurs d'ennemis nouveaux, ennemi détruit, attaque différée, Don pour la magie.

use arcanum_core::{
    Action, AdvancedAction, BasicColor, CardId, CardPlay, CombatEnemy, DeedCard, ElementPoints,
    Enemy, EnemyToken, Event, GameState, InProgress, ManaColor, ManaPayment, Resistance, Spell,
    TimeOfDay, apply, legal_actions, view,
};

mod arene;
use arene::*;

use Spell::*;
use arcanum_core::BasicAction::{Concentration, March, Rage};

// --- Décors ---

/// Un décor de la spec 054 : le bac à sable (Arythea) de jour ou de nuit, la figurine, la main, les
/// dés de la Source, les ennemis posés aux orques ②, puis des événements de plus (cristaux,
/// jetons, offre de sorts…).
#[derive(Clone)]
struct Decor {
    time: TimeOfDay,
    hand: Vec<CardId>,
    dice: Vec<ManaColor>,
    enemies: Vec<EnemyToken>,
    extra: Vec<Event>,
}

impl Default for Decor {
    fn default() -> Self {
        Self {
            time: TimeOfDay::Day,
            hand: Vec::new(),
            dice: Sandbox::default().dice,
            enemies: vec![prowlers()],
            extra: Vec::new(),
        }
    }
}

impl Decor {
    fn state(&self) -> GameState {
        self.events().iter().fold(GameState::default(), apply)
    }

    fn events(&self) -> Vec<Event> {
        let mut events = Sandbox {
            time: self.time,
            figure: Some(PLAIN_W),
            hand: self.hand.clone(),
            placed: self.enemies.iter().map(|&enemy| (enemy, ORCS_2)).collect(),
            dice: self.dice.clone(),
            ..Sandbox::default()
        }
        .events();
        events.extend(self.extra.iter().cloned());
        events
    }

    /// Le décor, puis le combat engagé contre les ennemis des orques ② (phase à distance).
    fn fighting(&self) -> GameState {
        let state = done(&self.state(), challenge(ORCS_2));
        done(&state, Action::StartCombat)
    }
}

fn spell(card: Spell) -> CardId {
    CardId::new(DeedCard::Spell(card), 1)
}

fn crystal(color: BasicColor) -> Event {
    Event::CrystalGained {
        player: SOLO,
        color,
    }
}

fn token_of(color: ManaColor) -> Event {
    Event::ManaTokenGained {
        player: SOLO,
        color,
    }
}

const RED: ManaColor = ManaColor::Basic(BasicColor::Red);
const BLUE: ManaColor = ManaColor::Basic(BasicColor::Blue);
const GREEN: ManaColor = ManaColor::Basic(BasicColor::Green);
const WHITE: ManaColor = ManaColor::Basic(BasicColor::White);

/// Le sort `card` lancé : option `option`, payé par `mana`, renforcé si `black`.
fn cast(card: CardId, option: u8, mana: ManaPayment, black: Option<ManaPayment>) -> Action {
    cast_full(card, option, mana, black, None, None)
}

/// Le sort `card` lancé, visant `enemy`.
fn cast_at(
    card: CardId,
    option: u8,
    mana: ManaPayment,
    black: Option<ManaPayment>,
    enemy: EnemyToken,
) -> Action {
    cast_full(card, option, mana, black, None, Some(enemy))
}

fn cast_full(
    card: CardId,
    option: u8,
    mana: ManaPayment,
    black: Option<ManaPayment>,
    extra_mana: Option<ManaPayment>,
    enemy: Option<EnemyToken>,
) -> Action {
    Action::PlayCard {
        card,
        play: CardPlay::Cast {
            option,
            mana,
            black,
            extra_mana,
            enemy,
            from_offer: false,
            targets: Default::default(),
        },
    }
}

/// Les façons de jouer `card` parmi les actions légales.
fn plays_of(state: &GameState, card: CardId) -> Vec<CardPlay> {
    legal_actions(state, SOLO)
        .into_iter()
        .filter_map(|action| match action {
            Action::PlayCard { card: played, play } if played == card => Some(play),
            _ => None,
        })
        .collect()
}

/// Les sorts lancés de `card` parmi les actions légales.
fn casts_of(state: &GameState, card: CardId) -> Vec<CardPlay> {
    plays_of(state, card)
        .into_iter()
        .filter(|play| matches!(play, CardPlay::Cast { .. }))
        .collect()
}

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

#[track_caller]
fn enemy_of(state: &GameState, token: EnemyToken) -> CombatEnemy {
    state
        .combat(SOLO)
        .and_then(|combat| combat.enemy(token))
        .cloned()
        .unwrap_or_else(|| panic!("{token:?} hors du combat"))
}

fn fire(points: u32) -> ElementPoints {
    ElementPoints {
        fire: points,
        ..ElementPoints::default()
    }
}

fn ice(points: u32) -> ElementPoints {
    ElementPoints {
        ice: points,
        ..ElementPoints::default()
    }
}

fn defeated(events: &[Event]) -> bool {
    events
        .iter()
        .any(|event| matches!(event, Event::EnemiesDefeated { enemies, .. } if !enemies.is_empty()))
}

// --- Lancer un sort ---

// Spec 054 C1
#[test]
fn spell_basic_costs_its_color() {
    // MKUE p. 4, 3.b : l'effet de base d'un sort se paie d'un mana de sa couleur.
    for (card, color, points) in [
        (spell(Fireball), BasicColor::Red, fire(5)),
        (spell(Snowstorm), BasicColor::Blue, ice(5)),
    ] {
        let decor = Decor {
            hand: vec![card],
            // Ni dé rouge ni dé bleu, ni or.
            dice: vec![GREEN, WHITE],
            ..Decor::default()
        };
        let state = done(&decor.fighting(), declare(&[prowlers()]));
        assert!(casts_of(&state, card).is_empty(), "{card:?} sans mana");
        let paid = Decor {
            extra: vec![crystal(color)],
            ..decor
        };
        let state = done(&paid.fighting(), declare(&[prowlers()]));
        let state = done(&state, cast(card, 0, ManaPayment::Crystal, None));
        assert_eq!(in_progress_points(&state), points, "{card:?}");
        assert!(defeated(&events_of(&state, &Action::ResolveAttack)));
    }
}

// Spec 054 C2
#[test]
fn strong_spell_not_by_day() {
    // MKUE p. 5, 2.a : de jour, pas de mana noir.
    let fireball = spell(Fireball);
    let state = Decor {
        hand: vec![fireball],
        dice: vec![RED, ManaColor::Black],
        extra: vec![crystal(BasicColor::Red), token_of(ManaColor::Black)],
        ..Decor::default()
    }
    .fighting();
    let state = done(&state, declare(&[prowlers()]));
    let casts = casts_of(&state, fireball);
    assert!(!casts.is_empty());
    assert!(
        casts
            .iter()
            .all(|play| matches!(play, CardPlay::Cast { black: None, .. })),
        "{casts:?}"
    );
}

// Spec 054 C3
#[test]
fn strong_spell_red_and_black_at_night() {
    // MKUE p. 4, 3.b ; p. 5, 2.b.
    let fireball = spell(Fireball);
    let (black_die, gold_die) = (0, 1);
    let state = Decor {
        time: TimeOfDay::Night,
        hand: vec![fireball],
        dice: vec![ManaColor::Black, ManaColor::Gold, GREEN],
        extra: vec![crystal(BasicColor::Red)],
        ..Decor::default()
    }
    .fighting();
    let state = done(&state, declare(&[prowlers()]));
    // Le dé or est épuisé de nuit : il ne paie ni le rouge ni le noir.
    for play in casts_of(&state, fireball) {
        let CardPlay::Cast { mana, black, .. } = play else {
            unreachable!()
        };
        assert_ne!(mana, ManaPayment::Die(gold_die));
        assert_ne!(black, Some(ManaPayment::Die(gold_die)));
    }
    let state = done(
        &state,
        cast(
            fireball,
            0,
            ManaPayment::Crystal,
            Some(ManaPayment::Die(black_die)),
        ),
    );
    assert_eq!(wounds_in_hand(&state), 1);
    assert_eq!(in_progress_points(&state), fire(8));
}

// Spec 054 C4
#[test]
fn strong_spell_in_dungeon_by_day() {
    // FAQ p1 : le combat d'un donjon suit les règles de nuit (spec 044).
    let fireball = spell(Fireball);
    let state = Adventure {
        hand: vec![fireball, card(Rage, 1)],
        dice: vec![ManaColor::Black, GREEN, BLUE, WHITE],
        ..Adventure::default()
    }
    .state();
    let state = apply(state, &token_of(RED));
    let entered = done(&state, Action::EnterAdventureSite);
    let attacking = done(&entered, declare(&[minotaur()]));
    let strong = cast(fireball, 0, ManaPayment::Token, Some(ManaPayment::Die(0)));
    assert!(legal_actions(&attacking, SOLO).contains(&strong));
    let state = done(&attacking, strong);
    assert_eq!(in_progress_points(&state), fire(8));
}

// Spec 054 C5
#[test]
fn concentration_does_not_boost_spells() {
    // Compendium, *Basic Action Cards* S6 : Concentration, des actions seulement.
    let fireball = spell(Fireball);
    let concentration = card(Concentration, 1);
    let state = Decor {
        hand: vec![concentration, fireball],
        ..Decor::default()
    }
    .fighting();
    let state = done(&state, declare(&[prowlers()]));
    assert!(!legal_actions(&state, SOLO).iter().any(|action| matches!(
        action,
        Action::PlayCard {
            play: CardPlay::Strong { with: Some(target), .. },
            ..
        } if target.card == fireball
    )));
}

// Spec 054 C6
#[test]
fn combat_spells_only_in_combat() {
    // Spec 052 : un effet qui ne sert pas hors combat ; de côté seulement (MKUE p. 4, 3.e).
    let spells = [
        Fireball,
        Snowstorm,
        Expose,
        Tremor,
        FlameWall,
        ManaBolt,
        Whirlwind,
        BurningShield,
        Chill,
        Demolish,
    ]
    .map(spell);
    let mut extra: Vec<Event> = BasicColor::ALL.into_iter().map(crystal).collect();
    extra.push(token_of(ManaColor::Black));
    let state = Decor {
        time: TimeOfDay::Night,
        hand: spells.to_vec(),
        extra,
        ..Decor::default()
    }
    .state();
    for card in spells {
        let plays = plays_of(&state, card);
        assert!(!plays.is_empty(), "{card:?}");
        assert!(
            plays
                .iter()
                .all(|play| matches!(play, CardPlay::Sideways(_))),
            "{card:?} : {plays:?}"
        );
    }
}

// Spec 054 L1
#[test]
fn strong_spell_needs_black() {
    // MKUE p. 5, 2.b : sans noir, ou payé d'or de nuit, refusé.
    let fireball = spell(Fireball);
    let gold_die = 1;
    let state = Decor {
        time: TimeOfDay::Night,
        hand: vec![fireball],
        dice: vec![RED, ManaColor::Gold],
        extra: vec![crystal(BasicColor::Red)],
        ..Decor::default()
    }
    .fighting();
    let state = done(&state, declare(&[prowlers()]));
    refused(
        &state,
        cast(
            fireball,
            0,
            ManaPayment::Crystal,
            Some(ManaPayment::Die(gold_die)),
        ),
    );
    refused(&state, cast(fireball, 0, ManaPayment::Die(gold_die), None));
    // Le sort de base sans mana : refusé aussi.
    refused(&state, cast(fireball, 0, ManaPayment::Token, None));
}

// --- Ennemis et décors des sorts qui visent un ennemi ---

fn swordsmen() -> EnemyToken {
    token(Enemy::Swordsmen, 1)
}

fn golems() -> EnemyToken {
    token(Enemy::Golems, 1)
}

fn ice_mages() -> EnemyToken {
    token(Enemy::IceMages, 1)
}

fn fire_mages() -> EnemyToken {
    token(Enemy::FireMages, 1)
}

/// Les cristaux `colors`, puis un jeton noir si `black`.
fn mana(colors: &[BasicColor], black: bool) -> Vec<Event> {
    let mut events: Vec<Event> = colors.iter().map(|&color| crystal(color)).collect();
    if black {
        events.push(token_of(ManaColor::Black));
    }
    events
}

/// Un combat de nuit contre `enemies` aux orques ②, la main `hand`, les cristaux `colors` et un
/// jeton noir.
fn night_fight(hand: &[CardId], enemies: &[EnemyToken], colors: &[BasicColor]) -> GameState {
    Decor {
        time: TimeOfDay::Night,
        hand: hand.to_vec(),
        enemies: enemies.to_vec(),
        extra: mana(colors, true),
        ..Decor::default()
    }
    .fighting()
}

/// Un combat de jour contre `enemies` aux orques ②, la main `hand`, les cristaux `colors`.
fn day_fight(hand: &[CardId], enemies: &[EnemyToken], colors: &[BasicColor]) -> GameState {
    Decor {
        hand: hand.to_vec(),
        enemies: enemies.to_vec(),
        extra: mana(colors, false),
        ..Decor::default()
    }
    .fighting()
}

/// L'assaut de la tour de mage défendue par `defender`, de nuit, la main `hand`, les cristaux
/// `colors` et un jeton noir (spec 043).
fn tower_assault(defender: EnemyToken, hand: &[CardId], colors: &[BasicColor]) -> GameState {
    let decor = Fortified {
        figure: NEAR_TOWER,
        time: TimeOfDay::Night,
        hand: hand.to_vec(),
        keep: None,
        tower: Some((defender, true)),
        ..Fortified::default()
    };
    let mut events = decor.events();
    events.extend(mana(colors, true));
    events.extend(assault(MAGE_TOWER, NEAR_TOWER, defender));
    events.iter().fold(GameState::default(), apply)
}

/// L'assaut de la forteresse défendue par les Gardes, de jour, la main `hand`, les cristaux
/// `colors` (spec 043).
fn keep_assault(hand: &[CardId], colors: &[BasicColor]) -> GameState {
    let decor = Fortified {
        figure: NEAR_KEEP,
        hand: hand.to_vec(),
        keep: Some((guardsmen(), true)),
        tower: None,
        ..Fortified::default()
    };
    let mut events = decor.events();
    events.extend(mana(colors, false));
    events.extend(assault(KEEP, NEAR_KEEP, guardsmen()));
    events.iter().fold(GameState::default(), apply)
}

/// Les événements d'un assaut du site `site` depuis `from`, contre `defender`.
fn assault(site: arcanum_core::Hex, from: arcanum_core::Hex, defender: EnemyToken) -> [Event; 2] {
    [
        Event::AssaultStarted {
            player: SOLO,
            site,
            from,
        },
        Event::CombatStarted {
            player: SOLO,
            enemies: vec![(site, defender)],
        },
    ]
}

const BLACK_TOKEN: Option<ManaPayment> = Some(ManaPayment::Token);

/// Les ennemis visés par les sorts lancés de `card` parmi les actions légales.
fn cast_targets(state: &GameState, card: CardId) -> Vec<EnemyToken> {
    let mut targets: Vec<EnemyToken> = Vec::new();
    for play in casts_of(state, card) {
        if let CardPlay::Cast {
            enemy: Some(enemy), ..
        } = play
            && !targets.contains(&enemy)
        {
            targets.push(enemy);
        }
    }
    targets
}

// --- Exposition ---

// Spec 054 C7, C31
#[test]
fn expose_removes_fortification() {
    // Carte ; MKUE p. 9, 3.a ; Q3 : sans attaque en cours, les points d'Exposition sont perdus.
    let (expose, fireball) = (spell(Expose), spell(Fireball));
    let state = day_fight(
        &[expose, fireball],
        &[diggers()],
        &[BasicColor::White, BasicColor::Red],
    );
    assert_eq!(enemy_of(&state, diggers()).fortification(), 1);
    let state = done(
        &state,
        cast_at(expose, 0, ManaPayment::Crystal, None, diggers()),
    );
    assert_eq!(enemy_of(&state, diggers()).fortification(), 0);
    let seen = view(&state, SOLO);
    let combat = seen.combat.as_ref().expect("combat dans la vue");
    assert_eq!(combat.enemy(diggers()).map(|e| e.fortification()), Some(0));
    let state = done(&state, declare(&[diggers()]));
    let state = done(&state, cast(fireball, 0, ManaPayment::Crystal, None));
    assert!(defeated(&events_of(&state, &Action::ResolveAttack)));
    // Avec une attaque en cours sur les Rôdeurs, les points d'Exposition y vont.
    let state = day_fight(&[expose], &[prowlers(), diggers()], &[BasicColor::White]);
    let state = done(&state, declare(&[prowlers()]));
    let state = done(
        &state,
        cast_at(expose, 0, ManaPayment::Crystal, None, diggers()),
    );
    assert_eq!(in_progress_points(&state).physical, 2);
}

// Spec 054 C8
#[test]
fn expose_removes_resistances() {
    // Carte ; MKUE p. 9, 4.a.
    let expose = spell(Expose);
    let (rage, march) = (card(Rage, 1), card(March, 1));
    let state = day_fight(&[expose, rage, march], &[golems()], &[BasicColor::White]);
    let state = done(
        &state,
        cast_at(expose, 0, ManaPayment::Crystal, None, golems()),
    );
    assert!(
        enemy_of(&state, golems())
            .effective_resistances()
            .is_empty()
    );
    let state = done(&skip_to_attack_phase(&state), declare(&[golems()]));
    let state = done(&state, option(rage, 0));
    let state = done(&state, sideways_attack(march));
    assert_eq!(
        state.combat(SOLO).and_then(|c| c.in_progress_total()),
        Some(3)
    );
}

// Spec 054 C9
#[test]
fn mass_expose_all_enemies_one_option() {
    // Carte ; spec 053 : un effet bleu atteint un ennemi qui n'est plus résistant à la Glace.
    let expose = spell(Expose);
    let ice_shield = advanced(AdvancedAction::IceShield);
    let state = tower_assault(ice_mages(), &[expose, ice_shield], &[BasicColor::White]);
    assert_eq!(enemy_of(&state, ice_mages()).fortification(), 1);
    let fortifications = done(&state, cast(expose, 0, ManaPayment::Crystal, BLACK_TOKEN));
    let mages = enemy_of(&fortifications, ice_mages());
    assert_eq!(mages.fortification(), 0);
    assert_eq!(mages.effective_resistances(), vec![Resistance::Ice]);
    let resistances = done(&state, cast(expose, 1, ManaPayment::Crystal, BLACK_TOKEN));
    let mages = enemy_of(&resistances, ice_mages());
    assert_eq!(mages.fortification(), 1);
    assert!(mages.effective_resistances().is_empty());
    let state = done(&to_block_phase(&resistances), declare_block(ice_mages()));
    let state = done(&state, strong(ice_shield, BLUE_DIE));
    assert_eq!(enemy_of(&state, ice_mages()).effective_armor(), 3);
}

// --- Secousse et Tremblement de terre ---

// Spec 054 C10
#[test]
fn tremor_target_or_all() {
    // Carte ; Compendium, *Tremor* S1.
    let tremor = spell(Tremor);
    let state = day_fight(&[tremor], &[swordsmen(), prowlers()], &[BasicColor::Green]);
    let one = done(
        &state,
        cast_at(tremor, 0, ManaPayment::Crystal, None, swordsmen()),
    );
    assert_eq!(enemy_of(&one, swordsmen()).effective_armor(), 2);
    assert_eq!(enemy_of(&one, prowlers()).effective_armor(), 3);
    let all = done(&state, cast(tremor, 1, ManaPayment::Crystal, None));
    assert_eq!(enemy_of(&all, swordsmen()).effective_armor(), 3);
    assert_eq!(enemy_of(&all, prowlers()).effective_armor(), 1);
}

// Spec 054 C11
#[test]
fn earthquake_doubles_against_fortified() {
    // Compendium, *Earthquake* S4, S5.
    let tremor = spell(Tremor);
    let state = night_fight(&[tremor], &[guardsmen(), prowlers()], &[BasicColor::Green]);
    let all = done(&state, cast(tremor, 1, ManaPayment::Crystal, BLACK_TOKEN));
    assert_eq!(enemy_of(&all, guardsmen()).effective_armor(), 3);
    assert_eq!(enemy_of(&all, prowlers()).effective_armor(), 1);
    // En phase de parade aussi.
    let blocking = to_block_phase(&state);
    assert!(legal_actions(&blocking, SOLO).contains(&cast(
        tremor,
        1,
        ManaPayment::Crystal,
        BLACK_TOKEN
    )));
}

// Spec 054 C12
#[test]
fn earthquake_after_lost_fortification() {
    // Compendium, *Earthquake* S1 : fortifié au moment où le sort est joué.
    let (expose, tremor) = (spell(Expose), spell(Tremor));
    let state = night_fight(
        &[expose, tremor],
        &[guardsmen()],
        &[BasicColor::White, BasicColor::Green],
    );
    let state = done(
        &state,
        cast_at(expose, 0, ManaPayment::Crystal, None, guardsmen()),
    );
    let state = done(
        &state,
        cast_at(tremor, 0, ManaPayment::Crystal, BLACK_TOKEN, guardsmen()),
    );
    assert_eq!(enemy_of(&state, guardsmen()).effective_armor(), 4);
}

// --- Frisson ---

// Spec 054 C22
#[test]
fn chill_removes_fire_resistance() {
    // Carte ; Compendium, *Chill* S1 : rien sur un ennemi résistant à la Glace.
    let (chill, fireball) = (spell(Chill), spell(Fireball));
    let state = day_fight(
        &[chill, fireball],
        &[fire_mages(), ice_mages()],
        &[BasicColor::Blue, BasicColor::Red],
    );
    assert_eq!(cast_targets(&state, chill), vec![fire_mages()]);
    let state = done(
        &state,
        cast_at(chill, 0, ManaPayment::Crystal, None, fire_mages()),
    );
    let mages = enemy_of(&state, fire_mages());
    assert!(mages.does_not_attack());
    assert!(!mages.effective_resistances().contains(&Resistance::Fire));
    let state = done(&skip_to_attack_phase(&state), declare(&[fire_mages()]));
    let state = done(&state, cast(fireball, 0, ManaPayment::Crystal, None));
    assert_eq!(
        state.combat(SOLO).and_then(|c| c.in_progress_total()),
        Some(5)
    );
}

// Spec 054 C23
#[test]
fn lethal_chill_armor() {
    let chill = spell(Chill);
    let state = night_fight(&[chill], &[swordsmen()], &[BasicColor::Blue]);
    let state = done(
        &state,
        cast_at(chill, 0, ManaPayment::Crystal, BLACK_TOKEN, swordsmen()),
    );
    let swords = enemy_of(&state, swordsmen());
    assert!(swords.does_not_attack());
    assert_eq!(swords.effective_armor(), 1);
}

// Spec 054 L2
#[test]
fn targeted_spell_needs_a_live_enemy() {
    let (expose, tremor) = (spell(Expose), spell(Tremor));
    let state = day_fight(
        &[expose, tremor],
        &[prowlers(), diggers()],
        &[BasicColor::White, BasicColor::Green],
    );
    refused(&state, cast(expose, 0, ManaPayment::Crystal, None));
    refused(&state, cast(tremor, 0, ManaPayment::Crystal, None));
    let state = apply(
        state,
        &Event::EnemiesDefeated {
            player: SOLO,
            enemies: vec![prowlers()],
            fame: 0,
        },
    );
    refused(
        &state,
        cast_at(expose, 0, ManaPayment::Crystal, None, prowlers()),
    );
    refused(
        &state,
        cast_at(tremor, 0, ManaPayment::Crystal, None, prowlers()),
    );
}

// --- Tourbillon et Tornade ---

// Spec 054 C17
#[test]
fn whirlwind_does_not_attack() {
    // Carte ; spec 053 (Illusionnistes, Q7).
    let whirlwind = spell(Whirlwind);
    let state = day_fight(
        &[whirlwind],
        &[orc_summoners(), diggers()],
        &[BasicColor::White],
    );
    assert_eq!(
        cast_targets(&state, whirlwind),
        vec![orc_summoners(), diggers()]
    );
    let state = done(
        &state,
        cast_at(whirlwind, 0, ManaPayment::Crystal, None, orc_summoners()),
    );
    let events = events_of(&state, &Action::NextCombatPhase);
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::EnemySummoned { .. })),
        "{events:?}"
    );
    // Un ennemi paré n'est plus une cible.
    let (rage, determination) = (
        card(Rage, 1),
        card(arcanum_core::BasicAction::Determination, 1),
    );
    let state = day_fight(
        &[whirlwind, rage, determination],
        &[prowlers(), diggers()],
        &[BasicColor::White],
    );
    let state = done(&to_block_phase(&state), declare_block(prowlers()));
    let state = done(&state, option(rage, 1));
    let state = done(&state, option(determination, 1));
    let state = done(&state, Action::ResolveBlock);
    assert_eq!(cast_targets(&state, whirlwind), vec![diggers()]);
}

// Spec 054 C18
#[test]
fn tornado_destroys_in_attack_phase() {
    // Carte : en phase d'attaque seulement ; vaincu, sa gloire (Q2).
    let whirlwind = spell(Whirlwind);
    let state = night_fight(&[whirlwind], &[fire_mages()], &[BasicColor::White]);
    let blocking = to_block_phase(&state);
    assert!(
        !casts_of(&blocking, whirlwind)
            .iter()
            .any(|play| matches!(play, CardPlay::Cast { black: Some(_), .. }))
    );
    let attacking = skip_to_attack_phase(&state);
    let fame = attacking.fame(SOLO);
    let events = events_of(
        &attacking,
        &cast_at(
            whirlwind,
            0,
            ManaPayment::Crystal,
            BLACK_TOKEN,
            fire_mages(),
        ),
    );
    assert!(events.contains(&Event::EnemyDestroyed {
        player: SOLO,
        enemy: fire_mages(),
    }));
    let state = events.iter().fold(attacking, apply);
    let mages = enemy_of(&state, fire_mages());
    assert!(mages.defeated && mages.destroyed);
    assert_eq!(state.fame(SOLO), fame + 5);
}

// Spec 054 L5
#[test]
fn all_destroyed_is_a_victory() {
    // Spec 031 : plus d'ennemi, le combat se termine.
    let whirlwind = spell(Whirlwind);
    let state = skip_to_attack_phase(&night_fight(
        &[whirlwind],
        &[prowlers()],
        &[BasicColor::White],
    ));
    let state = done(
        &state,
        cast_at(whirlwind, 0, ManaPayment::Crystal, BLACK_TOKEN, prowlers()),
    );
    assert!(legal_actions(&state, SOLO).contains(&Action::EndCombat));
    assert!(state.enemy_tokens_at(ORCS_2).is_empty());
}

// Spec 054 C32
#[test]
fn destroyed_enemy_follows_undo_and_reload() {
    // Spec 024 : le retour en arrière et la sauvegarde rechargée.
    let whirlwind = spell(Whirlwind);
    let decor = Decor {
        time: TimeOfDay::Night,
        hand: vec![whirlwind],
        extra: mana(&[BasicColor::White], true),
        ..Decor::default()
    };
    let (mut runner, puppet) = runner(decor.events());
    play(&mut runner, &puppet, challenge(ORCS_2));
    play(&mut runner, &puppet, Action::StartCombat);
    play(&mut runner, &puppet, Action::NextCombatPhase);
    play(&mut runner, &puppet, Action::NextCombatPhase);
    let fame = runner.state().fame(SOLO);
    play(
        &mut runner,
        &puppet,
        cast_at(whirlwind, 0, ManaPayment::Crystal, BLACK_TOKEN, prowlers()),
    );
    assert!(enemy_of(runner.state(), prowlers()).destroyed);
    let reloaded = arcanum_core::GameRunner::resume(
        runner.log().clone(),
        runner.rng_positions(),
        vec![Box::new(Puppet::default())],
    )
    .expect("journal valide");
    assert!(enemy_of(reloaded.state(), prowlers()).destroyed);
    assert_eq!(reloaded.state().fame(SOLO), fame + 2);
    assert_eq!(runner.undo(), Ok(()));
    assert!(!enemy_of(runner.state(), prowlers()).defeated);
    assert_eq!(runner.state().fame(SOLO), fame);
}

// --- Démolition et Désintégration ---

// Spec 054 C24
#[test]
fn demolish_ignores_site_fortifications() {
    // Carte ; Compendium, *Demolish* S1, S5.
    let demolish = spell(Demolish);
    let state = keep_assault(&[demolish], &[BasicColor::Red]);
    assert_eq!(enemy_of(&state, guardsmen()).fortification(), 2);
    let state = done(&state, cast(demolish, 0, ManaPayment::Crystal, None));
    let guards = enemy_of(&state, guardsmen());
    assert_eq!(guards.fortification(), 1);
    assert_eq!(guards.effective_armor(), 6);
    assert_eq!(attack_targets(&state), vec![vec![KEEP]]);
    assert!(
        state
            .modifiers(SOLO)
            .iter()
            .any(|modifier| modifier.kind == arcanum_core::ModifierKind::IgnoreSiteFortifications)
    );
    // Au tour suivant, un nouvel assaut les retrouve doublement fortifiés.
    let mut events = vec![
        Event::CombatEnded { player: SOLO },
        Event::TurnEnded { player: SOLO },
    ];
    events.extend(assault(KEEP, NEAR_KEEP, guardsmen()));
    let state = events.iter().fold(state, apply);
    assert!(state.modifiers(SOLO).is_empty());
    assert_eq!(enemy_of(&state, guardsmen()).fortification(), 2);
}

// Spec 054 C25
#[test]
fn demolish_armor_not_on_fire_resistant() {
    // FAQ p1 ; Compendium, *Demolish* Q2.
    let demolish = spell(Demolish);
    let state = tower_assault(fire_mages(), &[demolish], &[BasicColor::Red]);
    let state = done(&state, cast(demolish, 0, ManaPayment::Crystal, None));
    let mages = enemy_of(&state, fire_mages());
    assert_eq!(mages.fortification(), 0);
    assert_eq!(mages.effective_armor(), 5);
}

// Spec 054 C26
#[test]
fn disintegrate_destroys_and_weakens_others() {
    // Compendium, *Disintegrate* S1, S3.
    let demolish = spell(Demolish);
    let state = skip_to_attack_phase(&night_fight(
        &[demolish],
        &[prowlers(), swordsmen(), fire_mages()],
        &[BasicColor::Red],
    ));
    assert_eq!(
        cast_targets(&state, demolish),
        vec![prowlers(), swordsmen()]
    );
    let state = done(
        &state,
        cast_at(demolish, 0, ManaPayment::Crystal, BLACK_TOKEN, prowlers()),
    );
    assert!(enemy_of(&state, prowlers()).destroyed);
    assert_eq!(enemy_of(&state, swordsmen()).effective_armor(), 4);
    assert_eq!(enemy_of(&state, fire_mages()).effective_armor(), 5);
}

// --- Mur de flammes et Déluge de flammes ---

// Spec 054 C13
#[test]
fn flame_wall_block() {
    let flame_wall = spell(FlameWall);
    let state = day_fight(&[flame_wall], &[swordsmen()], &[BasicColor::Red]);
    let state = done(&to_block_phase(&state), declare_block(swordsmen()));
    let state = done(&state, cast(flame_wall, 1, ManaPayment::Crystal, None));
    assert_eq!(in_progress_points(&state), fire(7));
    let events = events_of(&state, &Action::ResolveBlock);
    assert!(events.contains(&Event::EnemyBlocked {
        player: SOLO,
        enemy: swordsmen(),
    }));
}

/// La pile brune avec le Minotaure sur le dessus : la créature tirée par les Invocateurs.
fn minotaur_on_top() -> Event {
    let mut order = vec![minotaur()];
    order.extend(
        arcanum_core::GameContent::embedded()
            .enemy_tokens(arcanum_core::EnemyColor::Brown)
            .into_iter()
            .filter(|&token| token != minotaur()),
    );
    Event::EnemyPileShuffled {
        color: arcanum_core::EnemyColor::Brown,
        order,
    }
}

fn defeat(state: &GameState, enemy: EnemyToken) -> GameState {
    apply(
        state.clone(),
        &Event::EnemiesDefeated {
            player: SOLO,
            enemies: vec![enemy],
            fame: 0,
        },
    )
}

// Spec 054 C14
#[test]
fn flame_wave_counts_enemies_faced() {
    // Compendium, *Flame Wave* S2, S3, Q4 : ni le vaincu d'avant la phase, ni la créature.
    let (first, second) = (spell(FlameWall), CardId::new(DeedCard::Spell(FlameWall), 2));
    let mut extra = mana(&[BasicColor::Red, BasicColor::Red], true);
    extra.push(token_of(ManaColor::Black));
    extra.push(minotaur_on_top());
    let state = Decor {
        time: TimeOfDay::Night,
        hand: vec![first, second],
        enemies: vec![prowlers(), orc_summoners(), diggers()],
        extra,
        ..Decor::default()
    }
    .fighting();
    let state = defeat(&state, prowlers());
    let state = done(&to_block_phase(&state), declare_block(orc_summoners()));
    let state = done(&state, cast(first, 1, ManaPayment::Crystal, BLACK_TOKEN));
    assert_eq!(in_progress_points(&state), fire(11));
    let state = skip_to_attack_phase(&done(&state, Action::ResolveBlock));
    let state = defeat(&state, diggers());
    let state = done(&state, declare(&[orc_summoners()]));
    let state = done(&state, cast(second, 0, ManaPayment::Crystal, BLACK_TOKEN));
    assert_eq!(in_progress_points(&state), fire(9));
}

// --- Éclair de mana ---

/// Les options des sorts lancés de `card` parmi les actions légales.
fn cast_options(state: &GameState, card: CardId) -> Vec<u8> {
    let mut options = Vec::new();
    for play in casts_of(state, card) {
        if let CardPlay::Cast { option, .. } = play
            && !options.contains(&option)
        {
            options.push(option);
        }
    }
    options
}

// Spec 054 C15
#[test]
fn mana_bolt_option_by_paid_color() {
    // Carte ; Compendium, *Mana Bolt* S1 : une option par couleur payée en plus.
    let mana_bolt = spell(ManaBolt);
    let all = [
        BasicColor::Blue,
        BasicColor::Red,
        BasicColor::White,
        BasicColor::Green,
    ];
    let state = day_fight(&[mana_bolt], &[prowlers()], &all);
    let ranged = done(&state, declare(&[prowlers()]));
    assert_eq!(cast_options(&ranged, mana_bolt), vec![2, 3]);
    let attacking = done(&skip_to_attack_phase(&state), declare(&[prowlers()]));
    let state = done(
        &attacking,
        cast_full(
            mana_bolt,
            1,
            ManaPayment::Crystal,
            None,
            Some(ManaPayment::Crystal),
            None,
        ),
    );
    assert_eq!(
        in_progress_points(&state),
        ElementPoints {
            cold_fire: 7,
            ..ElementPoints::default()
        }
    );
}

// Spec 054 C16
#[test]
fn mana_thunderbolt_three_mana() {
    let mana_bolt = spell(ManaBolt);
    let state = night_fight(
        &[mana_bolt],
        &[prowlers()],
        &[BasicColor::Blue, BasicColor::Green],
    );
    let state = done(&state, declare(&[prowlers()]));
    let state = done(
        &state,
        cast_full(
            mana_bolt,
            3,
            ManaPayment::Crystal,
            BLACK_TOKEN,
            Some(ManaPayment::Crystal),
            None,
        ),
    );
    assert_eq!(in_progress_points(&state), ice(8));
    assert_eq!(state.inventory(SOLO).crystals(BasicColor::Blue), 0);
    assert_eq!(state.inventory(SOLO).crystals(BasicColor::Green), 0);
}

// --- Bouclier ardent et Bouclier explosif ---

fn deferred_attack() -> arcanum_core::ModifierKind {
    arcanum_core::ModifierKind::DeferredAttack {
        amount: 4,
        element: arcanum_core::Element::Fire,
    }
}

// Spec 054 C19, L3, C31
#[test]
fn burning_shield_attack_after_successful_block() {
    // Carte ; Compendium, *Burning Shield* S2, S3.
    let shield = spell(BurningShield);
    let state = day_fight(&[shield], &[prowlers(), wolf_riders()], &[BasicColor::Red]);
    let state = done(&to_block_phase(&state), declare_block(prowlers()));
    let state = done(&state, cast(shield, 0, ManaPayment::Crystal, None));
    refused(&state, Action::UseDeferredAttack);
    let state = done(&state, Action::ResolveBlock);
    assert!(
        view(&state, SOLO)
            .modifiers
            .iter()
            .any(|modifier| modifier.kind == deferred_attack())
    );
    refused(&state, Action::UseDeferredAttack);
    let state = skip_to_attack_phase(&state);
    refused(&state, Action::UseDeferredAttack);
    let state = done(&state, declare(&[wolf_riders()]));
    let state = done(&state, Action::UseDeferredAttack);
    assert_eq!(in_progress_points(&state), fire(4));
    refused(&state, Action::UseDeferredAttack);
    assert!(defeated(&events_of(&state, &Action::ResolveAttack)));
    // Parade ratée : pas d'attaque différée.
    let state = day_fight(&[shield], &[swordsmen()], &[BasicColor::Red]);
    let state = done(&to_block_phase(&state), declare_block(swordsmen()));
    let state = done(&state, cast(shield, 0, ManaPayment::Crystal, None));
    let state = done(&state, Action::ResolveBlock);
    assert!(
        !state
            .modifiers(SOLO)
            .iter()
            .any(|modifier| modifier.kind == deferred_attack())
    );
}

// Spec 054 C20
#[test]
fn exploding_shield_destroys_blocked_enemy() {
    // Carte ; Compendium, *Exploding Shield* S1 : pas un résistant au Feu.
    let shield = spell(BurningShield);
    let determination = card(arcanum_core::BasicAction::Determination, 1);
    for (enemy, destroyed) in [(prowlers(), true), (token(Enemy::FireGolems, 1), false)] {
        let state = night_fight(&[shield, determination], &[enemy], &[BasicColor::Red]);
        let fame = state.fame(SOLO);
        let state = done(&to_block_phase(&state), declare_block(enemy));
        let state = done(&state, cast(shield, 0, ManaPayment::Crystal, BLACK_TOKEN));
        let state = done(&state, option(determination, 1));
        let events = events_of(&state, &Action::ResolveBlock);
        assert!(events.contains(&Event::EnemyBlocked {
            player: SOLO,
            enemy,
        }));
        assert_eq!(
            events.contains(&Event::EnemyDestroyed {
                player: SOLO,
                enemy,
            }),
            destroyed,
            "{enemy:?}"
        );
        let state = events.iter().fold(state, apply);
        assert_eq!(enemy_of(&state, enemy).destroyed, destroyed);
        if destroyed {
            assert_eq!(state.fame(SOLO), fame + 2);
        }
    }
}

// Spec 054 C21
#[test]
fn exploding_shield_on_summoned_monster() {
    // FAQ p2 ; Compendium, *Exploding Shield* Q2 : la créature détruite, sans gloire.
    let shield = spell(BurningShield);
    let rage = card(Rage, 1);
    let mut extra = mana(&[BasicColor::Red], true);
    extra.push(minotaur_on_top());
    let state = Decor {
        time: TimeOfDay::Night,
        hand: vec![shield, rage],
        enemies: vec![orc_summoners()],
        extra,
        ..Decor::default()
    }
    .fighting();
    let fame = state.fame(SOLO);
    let state = to_block_phase(&state);
    assert_eq!(enemy_of(&state, orc_summoners()).summoned, Some(minotaur()));
    let state = done(&state, declare_block(orc_summoners()));
    let state = done(&state, cast(shield, 0, ManaPayment::Crystal, BLACK_TOKEN));
    let state = done(&state, option(rage, 1));
    let events = events_of(&state, &Action::ResolveBlock);
    assert!(events.contains(&Event::EnemyDestroyed {
        player: SOLO,
        enemy: minotaur(),
    }));
    let state = events.iter().fold(state, apply);
    let summoners = enemy_of(&state, orc_summoners());
    assert!(!summoners.defeated);
    assert_eq!(summoners.summoned, None);
    assert_eq!(state.fame(SOLO), fame);
}

// --- Don pour la magie ---

fn magic_talent() -> CardId {
    advanced(AdvancedAction::MagicTalent)
}

/// Don pour la magie de base, en défaussant `discarded`.
fn talent_discarding(discarded: CardId) -> Action {
    Action::PlayCard {
        card: magic_talent(),
        play: CardPlay::Basic {
            option: 0,
            discard: Some(discarded),
            mana: None,
            with: None,
            gain: None,
            die: None,
        },
    }
}

/// Le sort `card` lancé depuis l'offre de sorts.
fn cast_from_offer(card: CardId, mana: ManaPayment, black: Option<ManaPayment>) -> Action {
    Action::PlayCard {
        card,
        play: CardPlay::Cast {
            option: 0,
            mana,
            black,
            extra_mana: None,
            enemy: None,
            from_offer: true,
            targets: Default::default(),
        },
    }
}

/// Les sorts lançables depuis l'offre parmi les actions légales.
fn offer_casts(state: &GameState) -> Vec<CardId> {
    let mut cards = Vec::new();
    for action in legal_actions(state, SOLO) {
        if let Action::PlayCard {
            card,
            play: CardPlay::Cast {
                from_offer: true, ..
            },
        } = action
            && !cards.contains(&card)
        {
            cards.push(card);
        }
    }
    cards
}

/// Un combat contre les Rôdeurs, Don pour la magie et `others` en main, l'offre de sorts `offer`.
fn talent_fight(
    time: TimeOfDay,
    others: &[CardId],
    offer: &[Spell],
    colors: &[BasicColor],
) -> GameState {
    let mut hand = vec![magic_talent()];
    hand.extend_from_slice(others);
    let mut extra = spell_offer(&offer.iter().map(|&card| spell(card)).collect::<Vec<_>>());
    extra.extend(mana(colors, time == TimeOfDay::Night));
    Decor {
        time,
        hand,
        extra,
        ..Decor::default()
    }
    .fighting()
}

// Spec 054 C27, C31
#[test]
fn magic_talent_casts_spell_from_offer() {
    // Carte ; Compendium, *Magic Talent* S1 : le sort reste dans l'offre.
    let rage = card(Rage, 1);
    let state = talent_fight(
        TimeOfDay::Day,
        &[rage],
        &[Fireball, Snowstorm, Tremor],
        &[BasicColor::Red],
    );
    let offer = state.offer(arcanum_core::Offer::Spells).to_vec();
    let state = done(&state, talent_discarding(rage));
    assert!(view(&state, SOLO).modifiers.iter().any(
        |modifier| modifier.kind == arcanum_core::ModifierKind::CastFromOffer(BasicColor::Red)
    ));
    let state = done(&state, declare(&[prowlers()]));
    assert_eq!(offer_casts(&state), vec![spell(Fireball)]);
    let state = done(
        &state,
        cast_from_offer(spell(Fireball), ManaPayment::Crystal, None),
    );
    assert_eq!(in_progress_points(&state), fire(5));
    assert_eq!(state.offer(arcanum_core::Offer::Spells), offer.as_slice());
    assert!(offer_casts(&state).is_empty());
    assert!(!state.hand(SOLO).contains(&spell(Fireball)));
    assert!(
        !state
            .modifiers(SOLO)
            .iter()
            .any(|modifier| matches!(modifier.kind, arcanum_core::ModifierKind::CastFromOffer(_)))
    );
}

// Spec 054 C28
#[test]
fn magic_talent_same_color() {
    // Carte, version Lost Legion : un sort de la couleur de la carte défaussée.
    let march = card(March, 1);
    let state = talent_fight(
        TimeOfDay::Day,
        &[march],
        &[Fireball, Snowstorm, Tremor],
        &[BasicColor::Green],
    );
    let state = done(&state, talent_discarding(march));
    assert_eq!(offer_casts(&state), vec![spell(Tremor)]);
    // Une main de Blessures : rien à défausser.
    let state = talent_fight(
        TimeOfDay::Day,
        &[wound(1), wound(2)],
        &[Fireball, Snowstorm, Tremor],
        &[],
    );
    assert!(
        plays_of(&state, magic_talent())
            .iter()
            .all(|play| matches!(play, CardPlay::Sideways(_)))
    );
}

// Spec 054 C29
#[test]
fn magic_talent_strong_spell_from_offer() {
    let rage = card(Rage, 1);
    let state = talent_fight(
        TimeOfDay::Night,
        &[rage],
        &[Fireball, Snowstorm, Tremor],
        &[BasicColor::Red],
    );
    let state = done(&state, talent_discarding(rage));
    let state = done(&state, declare(&[prowlers()]));
    let state = done(
        &state,
        cast_from_offer(spell(Fireball), ManaPayment::Crystal, BLACK_TOKEN),
    );
    assert_eq!(wounds_in_hand(&state), 1);
    assert_eq!(in_progress_points(&state), fire(8));
}

// Spec 054 L4
#[test]
fn magic_talent_one_spell_only() {
    let rage = card(Rage, 1);
    let state = talent_fight(
        TimeOfDay::Day,
        &[rage],
        &[Fireball, FlameWall, ManaMeltdown],
        &[BasicColor::Red, BasicColor::Red],
    );
    let state = done(&state, talent_discarding(rage));
    let state = done(&state, declare(&[prowlers()]));
    // Un sort de la spec 057 n'est jamais proposé.
    assert_eq!(offer_casts(&state), vec![spell(Fireball)]);
    let state = done(
        &state,
        cast_from_offer(spell(Fireball), ManaPayment::Crystal, None),
    );
    let state = skip_to_attack_phase(&done(&state, Action::ResolveAttack));
    refused(
        &state,
        cast_from_offer(spell(FlameWall), ManaPayment::Crystal, None),
    );
}

/// Le sort `card` lancé depuis l'offre avec son option `option`.
fn cast_option_from_offer(card: CardId, option: u8) -> Action {
    Action::PlayCard {
        card,
        play: CardPlay::Cast {
            option,
            mana: ManaPayment::Crystal,
            black: None,
            extra_mana: None,
            enemy: None,
            from_offer: true,
            targets: Default::default(),
        },
    }
}

/// Le fait du tour du sort `card` lancé depuis l'offre, s'il y est.
fn offer_cast_fact(state: &GameState, card: CardId) -> Option<arcanum_core::PlayedCard> {
    view(state, SOLO)
        .turn_facts
        .into_iter()
        .find_map(|fact| match fact {
            arcanum_core::TurnFact::SpellCastFromOffer { played } if played.card == card => {
                Some(played)
            }
            _ => None,
        })
}

// Spec 054 C33, issue privée 246
#[test]
fn offer_spell_ranged_victory() {
    // Compendium, *Magic Talent* S1 : le sort de l'offre se joue comme s'il était en main.
    let rage = card(Rage, 1);
    let state = talent_fight(
        TimeOfDay::Day,
        &[rage],
        &[Fireball, Snowstorm, Tremor],
        &[BasicColor::Red],
    );
    let state = done(&state, talent_discarding(rage));
    let state = done(&state, declare(&[prowlers()]));
    let cast = cast_from_offer(spell(Fireball), ManaPayment::Crystal, None);
    let state = done(&state, cast.clone());
    let played = offer_cast_fact(&state, spell(Fireball)).expect("fait du tour du sort de l'offre");
    let Action::PlayCard { play, .. } = cast else {
        unreachable!()
    };
    assert_eq!(played.play, play);
    // Le sort reste dans l'offre : il n'entre pas dans la zone de jeu.
    assert!(
        !state
            .play_area(SOLO)
            .iter()
            .any(|played| played.card == spell(Fireball))
    );
    let events = events_of(&state, &Action::ResolveAttack);
    assert!(defeated(&events));
    let state = events.iter().fold(state, apply);
    assert!(offer_casts(&state).is_empty());
}

// Spec 054 C33, issue privée 246
#[test]
fn offer_spell_block() {
    // Carte Mur de flammes : Parade de feu 7 contre l'Attaque 4 des Prowlers.
    let rage = card(Rage, 1);
    let state = talent_fight(
        TimeOfDay::Day,
        &[rage],
        &[FlameWall, Snowstorm, Tremor],
        &[BasicColor::Red],
    );
    let state = done(&state, talent_discarding(rage));
    let state = done(&to_block_phase(&state), declare_block(prowlers()));
    let state = done(&state, cast_option_from_offer(spell(FlameWall), 1));
    assert_eq!(in_progress_points(&state), fire(7));
    assert!(offer_cast_fact(&state, spell(FlameWall)).is_some());
    let events = events_of(&state, &Action::ResolveBlock);
    assert!(
        events.iter().any(
            |event| matches!(event, Event::EnemyBlocked { enemy, .. } if *enemy == prowlers())
        ),
        "{events:?}"
    );
}

// Spec 054 C33, issue privée 246
#[test]
fn offer_spell_attack_conquers_tower() {
    // MKUE p. 10, 2.c et 3.d : le défenseur vaincu, la tour est conquise ; sinon, retraite.
    let (rage, expose) = (card(Rage, 1), spell(Expose));
    let hand = [magic_talent(), rage, expose];
    let colors = [BasicColor::Red, BasicColor::White];
    let attack = |state: &GameState| {
        let state = done(state, talent_discarding(rage));
        let state = done(&skip_to_attack_phase(&state), declare(&[fire_mages()]));
        done(
            &state,
            cast_from_offer(spell(Fireball), ManaPayment::Crystal, None),
        )
    };
    let with_offer = |state: GameState| {
        spell_offer(&[spell(Fireball), spell(Snowstorm), spell(Tremor)])
            .iter()
            .fold(state, apply)
    };
    // Sans Exposition : Feu contre résistance au Feu, 5 divisé par deux, l'assaut est repoussé.
    let state = attack(&with_offer(tower_assault(fire_mages(), &hand, &colors)));
    assert!(offer_cast_fact(&state, spell(Fireball)).is_some());
    let events = events_of(&state, &Action::ResolveAttack);
    assert!(!defeated(&events));
    let state = events.iter().fold(state, apply);
    let events = events_of(&state, &Action::EndCombat);
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::HeroWithdrew { .. })),
        "{events:?}"
    );
    // Après Exposition : plus fortifiés ni résistants, les Fire Mages sont vaincus, la tour conquise.
    let state = with_offer(tower_assault(fire_mages(), &hand, &colors));
    let state = done(
        &state,
        cast_at(expose, 0, ManaPayment::Crystal, None, fire_mages()),
    );
    let state = attack(&state);
    let events = events_of(&state, &Action::ResolveAttack);
    assert!(defeated(&events), "{events:?}");
    let state = events.iter().fold(state, apply);
    assert_eq!(
        events_of(&state, &Action::EndCombat),
        [
            Event::CombatEnded { player: SOLO },
            Event::SiteConquered {
                player: SOLO,
                hex: MAGE_TOWER,
            },
        ]
    );
}

// --- Contenu ---

// Spec 054 C30
#[test]
fn combat_spells_supported() {
    use arcanum_core::BasicEffect;
    let combat = [
        Fireball,
        Snowstorm,
        Expose,
        Tremor,
        FlameWall,
        ManaBolt,
        Whirlwind,
        BurningShield,
        Chill,
        Demolish,
    ];
    for card in Spell::ALL {
        let definition = spell(card)
            .card()
            .definition()
            .expect("un sort a une définition");
        let supported = !matches!(definition.basic, BasicEffect::Unsupported)
            && !matches!(definition.strong, BasicEffect::Unsupported);
        // Spec 057 : les autres sorts aussi (C1, `all_spells_supported`).
        if combat.contains(&card) {
            assert!(supported, "{card:?}");
        }
    }
    let talent = magic_talent().card().definition().expect("définie");
    assert!(!matches!(talent.basic, BasicEffect::Unsupported));
}
