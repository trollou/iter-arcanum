//! Scénarios de la spec 057 (sorts : déplacement, unités et mana) : les dix derniers sorts du jeu
//! de base, ce qui ne vise que les autres joueurs, Don pour la magie sur tous les sorts.

use arcanum_core::{
    Action, AdvancedAction, BasicColor, CardId, CardPlay, CastTargets, DeedCard, Effect,
    EnemyToken, Event, GameState, Hex, ManaColor, ManaPayment, Picks, Spell, TimeOfDay, Unit,
    UnitCard, apply, legal_actions,
};

mod arene;
use arene::*;

use Spell::*;
use arcanum_core::BasicAction::March;

// --- Décors ---

/// La forêt de la tuile de départ (bac à sable).
const FOREST: Hex = Hex::new(1, -1);

/// Un décor de la spec 057 : le bac à sable (Arythea) de jour ou de nuit, la figurine, la main, les
/// dés de la Source, les ennemis posés, les unités déjà recrutées (dans l'ordre) et celles de
/// l'offre, puis des événements de plus (cristaux, jetons, Blessures…).
#[derive(Clone)]
struct Decor {
    time: TimeOfDay,
    figure: Hex,
    hand: Vec<CardId>,
    dice: Vec<ManaColor>,
    placed: Vec<(EnemyToken, Hex)>,
    owned: Vec<UnitCard>,
    offer: Vec<UnitCard>,
    /// Les tuiles derrière les deux tuiles de campagne dans la pioche.
    next: Vec<arcanum_core::TileId>,
    extra: Vec<Event>,
}

impl Default for Decor {
    fn default() -> Self {
        Self {
            time: TimeOfDay::Day,
            figure: PLAIN_W,
            hand: Vec::new(),
            dice: Sandbox::default().dice,
            placed: vec![(prowlers(), ORCS_2)],
            owned: Vec::new(),
            offer: Vec::new(),
            next: Vec::new(),
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
            next: self.next.clone(),
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

fn spell(card: Spell) -> CardId {
    CardId::new(DeedCard::Spell(card), 1)
}

fn unit(unit: Unit, copy: u8) -> UnitCard {
    UnitCard::new(unit, copy)
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

fn black_token() -> Event {
    token_of(ManaColor::Black)
}

/// L'unité épuisée (activée pour rien).
fn exhaust(unit: UnitCard) -> Event {
    Event::UnitActivated {
        player: SOLO,
        unit,
        ability: 0,
        option: 0,
        effect: Effect::Nothing,
        enemy: None,
    }
}

fn wound_unit(unit: UnitCard) -> Event {
    Event::UnitWounded {
        player: SOLO,
        unit,
        wounds: 1,
    }
}

/// Le sort `card` lancé : option `option`, payé par `mana`, renforcé si `black`, avec ses choix.
fn cast_with(
    card: CardId,
    option: u8,
    mana: ManaPayment,
    black: Option<ManaPayment>,
    targets: CastTargets,
) -> Action {
    Action::PlayCard {
        card,
        play: CardPlay::Cast {
            option,
            mana,
            black,
            extra_mana: None,
            enemy: None,
            from_offer: false,
            targets,
        },
    }
}

fn cast(card: CardId, option: u8, mana: ManaPayment, black: Option<ManaPayment>) -> Action {
    cast_with(card, option, mana, black, CastTargets::default())
}

fn on_units(units: &[UnitCard]) -> CastTargets {
    CastTargets {
        units: Picks::of(units),
        ..CastTargets::default()
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

/// Les unités visées par les lancers légaux de `card`, renforcés si `strong`.
fn unit_targets(state: &GameState, card: CardId, strong: bool) -> Vec<Vec<UnitCard>> {
    let mut targets = Vec::new();
    for play in casts_of(state, card) {
        if let CardPlay::Cast {
            black, targets: t, ..
        } = play
            && black.is_some() == strong
            && !targets.contains(&t.units.to_vec())
        {
            targets.push(t.units.to_vec());
        }
    }
    targets
}

fn wounds_in_hand(state: &GameState) -> usize {
    state
        .hand(SOLO)
        .iter()
        .filter(|card| card.card().is_wound())
        .count()
}

fn owned(state: &GameState, of: UnitCard) -> arcanum_core::OwnedUnit {
    *state
        .units(SOLO)
        .iter()
        .find(|owned| owned.card == of)
        .unwrap_or_else(|| panic!("{of:?} absente de la zone"))
}

const GREEN_DIE_0: ManaPayment = ManaPayment::Die(GREEN_DIE);
const RED_DIE_1: ManaPayment = ManaPayment::Die(RED_DIE);
const WHITE_DIE_3: ManaPayment = ManaPayment::Die(WHITE_DIE);

// --- Contenu et Don pour la magie ---

// Spec 057 C1 (Fuite de mana)
#[test]
fn mana_meltdown_basic_sideways_only() {
    // Carte : la Fuite de mana ne vise que les autres joueurs ; rien en solo (Q2).
    let meltdown = spell(ManaMeltdown);
    let state = Decor {
        hand: vec![meltdown],
        ..Decor::default()
    }
    .state();
    assert!(
        casts_of(&state, meltdown)
            .iter()
            .all(|play| play.is_strong()),
        "{:?}",
        casts_of(&state, meltdown)
    );
    assert!(
        plays_of(&state, meltdown)
            .iter()
            .any(|play| matches!(play, CardPlay::Sideways(_)))
    );
}

// Spec 057 C2
#[test]
fn magic_talent_casts_any_spell() {
    // Spec 054 : Don pour la magie lance depuis l'offre un sort de la couleur défaussée ; il y
    // reste.
    let march = card(March, 1);
    let talent = advanced(AdvancedAction::MagicTalent);
    let mut extra = spell_offer(&[spell(Restoration), spell(Fireball)]);
    extra.push(crystal(BasicColor::Green));
    let state = Decor {
        hand: vec![talent, march],
        extra,
        ..Decor::default()
    }
    .state();
    let offer = state.offer(arcanum_core::Offer::Spells).to_vec();
    let state = done(
        &state,
        Action::PlayCard {
            card: talent,
            play: CardPlay::Basic {
                option: 0,
                discard: Some(march),
                mana: None,
                with: None,
                gain: None,
                die: None,
            },
        },
    );
    let from_offer = Action::PlayCard {
        card: spell(Restoration),
        play: CardPlay::Cast {
            option: 0,
            mana: ManaPayment::Crystal,
            black: None,
            extra_mana: None,
            enemy: None,
            from_offer: true,
            targets: CastTargets::default(),
        },
    };
    let state = done(&state, from_offer);
    assert_eq!(state.healing_points(SOLO), 3);
    assert_eq!(state.offer(arcanum_core::Offer::Spells), offer.as_slice());
}

// --- Restauration et Renaissance ---

// Spec 057 C10
#[test]
fn restoration_heals_more_in_forest() {
    // Carte : Soin 3, Soin 5 en forêt ; MKUE p. 6, 5.b : pas de soin en combat.
    let restoration = spell(Restoration);
    let hand = vec![restoration, wound(1), wound(2)];
    let peasants = unit(Unit::Peasants, 1);
    let in_forest = Decor {
        figure: FOREST,
        hand: hand.clone(),
        owned: vec![peasants],
        extra: vec![wound_unit(peasants)],
        ..Decor::default()
    }
    .state();
    let state = done(&in_forest, cast(restoration, 0, GREEN_DIE_0, None));
    assert_eq!(state.healing_points(SOLO), 5);
    let on_plain = Decor {
        figure: Hex::ORIGIN,
        hand: hand.clone(),
        ..Decor::default()
    }
    .state();
    let state = done(&on_plain, cast(restoration, 0, GREEN_DIE_0, None));
    assert_eq!(state.healing_points(SOLO), 3);
    let fighting = Decor {
        hand,
        ..Decor::default()
    }
    .fighting();
    assert!(casts_of(&fighting, restoration).is_empty());
}

// Spec 057 C11, L3
#[test]
fn rebirth_readies_levels() {
    // Carte ; Compendium, *Rebirth* S2 : des unités épuisées de 3 niveaux en tout, 5 en forêt.
    let rebirth = spell(Restoration);
    let crossbowmen = unit(Unit::UtemCrossbowmen, 1);
    let peasants = unit(Unit::Peasants, 1);
    let golems = unit(Unit::GuardianGolems, 1);
    let units = [crossbowmen, peasants, golems];
    let decor = |figure| Decor {
        time: TimeOfDay::Night,
        figure,
        hand: vec![rebirth],
        owned: units.to_vec(),
        extra: vec![
            exhaust(crossbowmen),
            exhaust(peasants),
            exhaust(golems),
            black_token(),
        ],
        ..Decor::default()
    };
    let in_forest = decor(FOREST).state();
    assert!(unit_targets(&in_forest, rebirth, true).contains(&units.to_vec()));
    let state = done(
        &in_forest,
        cast_with(
            rebirth,
            0,
            GREEN_DIE_0,
            Some(ManaPayment::Token),
            on_units(&units),
        ),
    );
    assert!(units.iter().all(|&of| owned(&state, of).ready));
    assert_eq!(state.healing_points(SOLO), 5);
    let on_plain = decor(PLAIN_W).state();
    let targets = unit_targets(&on_plain, rebirth, true);
    assert!(targets.contains(&vec![crossbowmen, peasants]));
    assert!(!targets.iter().any(|picked| picked.len() == 3));
    // L3 : deux fois la même unité, refusée ; une unité disponible n'est jamais visée.
    let rested = done(
        &on_plain,
        cast_with(
            rebirth,
            0,
            GREEN_DIE_0,
            Some(ManaPayment::Token),
            on_units(&[peasants]),
        ),
    );
    assert!(owned(&rested, peasants).ready);
    assert!(!owned(&rested, golems).ready);
    refused(
        &on_plain,
        cast_with(
            rebirth,
            0,
            GREEN_DIE_0,
            Some(ManaPayment::Token),
            on_units(&[peasants, peasants]),
        ),
    );
}

// --- Fuite de mana, Lecture d'esprit ---

// Spec 057 C22
#[test]
fn mana_radiance_wounds_then_crystals() {
    // Carte ; MKUE p. 5, « Gain » 2 : une Blessure par cristal de la couleur, puis deux cristaux.
    let radiance = spell(ManaMeltdown);
    let decor = Decor {
        time: TimeOfDay::Night,
        hand: vec![radiance],
        extra: vec![
            crystal(BasicColor::Green),
            crystal(BasicColor::Green),
            black_token(),
        ],
        ..Decor::default()
    };
    let state = decor.state();
    let green = BasicColor::ALL
        .iter()
        .position(|&color| color == BasicColor::Green)
        .and_then(|index| u8::try_from(index).ok())
        .unwrap_or_default();
    let after = done(
        &state,
        cast(radiance, green, RED_DIE_1, Some(ManaPayment::Token)),
    );
    assert_eq!(wounds_in_hand(&after), 2);
    assert_eq!(after.inventory(SOLO).crystals(BasicColor::Green), 3);
    assert_eq!(
        after.mana_tokens(SOLO),
        [ManaColor::Basic(BasicColor::Green)]
    );
    let after = done(
        &state,
        cast(radiance, 0, RED_DIE_1, Some(ManaPayment::Token)),
    );
    assert_eq!(wounds_in_hand(&after), 0);
    assert_eq!(after.inventory(SOLO).crystals(BasicColor::Red), 2);
}

// Spec 057 C25
#[test]
fn mind_read_gains_crystal() {
    // Carte ; MKUE p. 5, « Gain » 2 : un cristal de la couleur, un jeton au-delà de trois.
    let mind_read = spell(MindRead);
    let white = 2;
    let state = Decor {
        hand: vec![mind_read],
        ..Decor::default()
    }
    .state();
    let after = done(&state, cast(mind_read, white, WHITE_DIE_3, None));
    assert_eq!(after.inventory(SOLO).crystals(BasicColor::White), 1);
    let full = Decor {
        hand: vec![mind_read],
        extra: vec![crystal(BasicColor::White); 3],
        ..Decor::default()
    }
    .state();
    let after = done(&full, cast(mind_read, white, WHITE_DIE_3, None));
    assert_eq!(after.inventory(SOLO).crystals(BasicColor::White), 3);
    assert_eq!(
        after.mana_tokens(SOLO),
        [ManaColor::Basic(BasicColor::White)]
    );
    // Vol d'esprit : de même en solo.
    let night = Decor {
        time: TimeOfDay::Night,
        hand: vec![mind_read],
        extra: vec![black_token()],
        ..Decor::default()
    }
    .state();
    let after = done(
        &night,
        cast(mind_read, white, WHITE_DIE_3, Some(ManaPayment::Token)),
    );
    assert_eq!(after.inventory(SOLO).crystals(BasicColor::White), 1);
}

// --- Flux d'énergie et Vol d'énergie ---

// Spec 057 C26
#[test]
fn energy_flow_readies_any_level() {
    // Carte ; Compendium, *Energy Steal* Q1, S2, S3 : une unité épuisée de tout niveau ; pas en
    // combat ; renforcé, elle est aussi soignée.
    let flow = spell(EnergyFlow);
    let guardians = unit(Unit::AltemGuardians, 1);
    let state = Decor {
        hand: vec![flow],
        owned: vec![guardians],
        extra: vec![exhaust(guardians)],
        ..Decor::default()
    }
    .state();
    assert_eq!(unit_targets(&state, flow, false), vec![vec![guardians]]);
    let after = done(
        &state,
        cast_with(flow, 0, GREEN_DIE_0, None, on_units(&[guardians])),
    );
    assert!(owned(&after, guardians).ready);
    let fighting = Decor {
        hand: vec![flow],
        owned: vec![guardians],
        extra: vec![exhaust(guardians)],
        ..Decor::default()
    }
    .fighting();
    assert!(casts_of(&fighting, flow).is_empty());
    // Vol d'énergie : soignée aussi ; une unité disponible et blessée n'est pas une cible.
    let swordsmen = unit(Unit::UtemSwordsmen, 1);
    let peasants = unit(Unit::Peasants, 1);
    let state = Decor {
        time: TimeOfDay::Night,
        hand: vec![flow],
        owned: vec![swordsmen, peasants],
        extra: vec![
            exhaust(swordsmen),
            wound_unit(swordsmen),
            wound_unit(peasants),
            black_token(),
        ],
        ..Decor::default()
    }
    .state();
    assert_eq!(unit_targets(&state, flow, true), vec![vec![swordsmen]]);
    let after = done(
        &state,
        cast_with(
            flow,
            0,
            GREEN_DIE_0,
            Some(ManaPayment::Token),
            on_units(&[swordsmen]),
        ),
    );
    assert!(owned(&after, swordsmen).ready);
    assert_eq!(owned(&after, swordsmen).wounds, 0);
}

// --- Voyage souterrain et Attaque souterraine ---

/// Le lac de la campagne 1, voisin du village.
const LAKE: Hex = Hex::new(2, -4);

fn to(hex: Hex) -> CastTargets {
    CastTargets {
        destination: Some(hex),
        ..CastTargets::default()
    }
}

/// Les arrivées des lancers légaux de `card`, renforcés si `strong`.
fn destinations(state: &GameState, card: CardId, strong: bool) -> Vec<Hex> {
    let mut hexes = Vec::new();
    for play in casts_of(state, card) {
        if let CardPlay::Cast { black, targets, .. } = play
            && black.is_some() == strong
            && let Some(hex) = targets.destination
            && !hexes.contains(&hex)
        {
            hexes.push(hex);
        }
    }
    hexes
}

fn march_basic() -> Action {
    Action::PlayCard {
        card: card(March, 1),
        play: CardPlay::Basic {
            option: 0,
            discard: None,
            mana: None,
            with: None,
            gain: None,
            die: None,
        },
    }
}

// Spec 057 C3
#[test]
fn underground_travel_no_provocation() {
    // Carte ; MKUE p. 7, 7.c : sans payer ni provoquer ; l'action du tour reste possible.
    let underground = spell(UndergroundTravel);
    let state = Decor {
        hand: vec![underground, card(March, 1)],
        ..Decor::default()
    }
    .state();
    let state = done(&state, march_basic());
    let action = cast_with(underground, 0, GREEN_DIE_0, None, to(VILLAGE));
    let events = events_of(&state, &action);
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::EnemiesProvoked { .. })),
        "{events:?}"
    );
    let state = events.iter().fold(state, apply);
    assert_eq!(state.figure(SOLO), Some(VILLAGE));
    assert_eq!(state.move_points(SOLO), 2);
    assert!(legal_actions(&state, SOLO).contains(&Action::Interact));
}

// Spec 057 C4, L1
#[test]
fn underground_travel_destinations() {
    // Carte ; MKUE p. 7, 7.d ; Compendium, *Movement* S5 : 3 cases au plus, ni marais ni lac, une
    // case sécurisée ; montagnes, sites et ennemis se traversent.
    let underground = spell(UndergroundTravel);
    let state = Decor {
        hand: vec![underground],
        ..Decor::default()
    }
    .state();
    let hexes = destinations(&state, underground, false);
    assert!(hexes.contains(&VILLAGE), "{hexes:?}");
    assert!(!hexes.contains(&LAKE));
    assert!(!hexes.contains(&ORCS_2));
    assert!(!hexes.contains(&PLAIN_W));
    let from_portal = Decor {
        figure: Hex::ORIGIN,
        hand: vec![underground],
        ..Decor::default()
    }
    .state();
    let hexes = destinations(&from_portal, underground, false);
    assert!(hexes.contains(&Hex::new(3, -1)), "{hexes:?}");
    assert!(!hexes.contains(&Hex::new(4, -2)));
    // Une montagne se traverse, n'est pas une arrivée.
    let fortified = Fortified {
        figure: NEAR_TOWER,
        hand: vec![underground],
        ..Fortified::default()
    }
    .state();
    let hexes = destinations(&fortified, underground, false);
    assert!(hexes.contains(&Hex::new(3, -7)), "{hexes:?}");
    assert!(!hexes.contains(&Hex::new(3, -6)));
    // L1 : sans arrivée, ou vers la case du héros.
    refused(&state, cast(underground, 0, GREEN_DIE_0, None));
    refused(
        &state,
        cast_with(underground, 0, GREEN_DIE_0, None, to(PLAIN_W)),
    );
}

// Spec 057 C5
#[test]
fn underground_travel_only_in_movement_phase() {
    // MKUE p. 6, 5.c ; Compendium, *Movement* S2 : en phase de déplacement seulement.
    let underground = spell(UndergroundTravel);
    let decor = Decor {
        hand: vec![underground, card(March, 1)],
        ..Decor::default()
    };
    assert!(!destinations(&decor.state(), underground, false).is_empty());
    assert!(casts_of(&decor.fighting(), underground).is_empty());
    // Arrêté par une provocation.
    let state = done(&decor.state(), march_basic());
    let state = done(&state, Action::MoveHero { to: VILLAGE });
    assert!(casts_of(&state, underground).is_empty());
    // Après l'action du tour (interaction).
    let at_village = Decor {
        figure: VILLAGE,
        placed: Vec::new(),
        hand: vec![underground],
        ..Decor::default()
    }
    .state();
    let state = done(&at_village, Action::Interact);
    assert!(casts_of(&state, underground).is_empty());
}

// Spec 057 C6
#[test]
fn underground_attack_assault_and_retreat() {
    // Carte ; Compendium, *Underground Attack* S1 ; spec 043 : assaut, fortifications du site
    // ignorées, retraite vers la case de départ.
    let underground = spell(UndergroundTravel);
    let start = Hex::new(4, -2);
    let mut events = Fortified {
        time: TimeOfDay::Night,
        figure: start,
        hand: vec![underground],
        ..Fortified::default()
    }
    .events();
    events.push(black_token());
    let state = events.iter().fold(GameState::default(), apply);
    assert_eq!(destinations(&state, underground, true), vec![KEEP]);
    let state = done(
        &state,
        cast_with(
            underground,
            0,
            GREEN_DIE_0,
            Some(ManaPayment::Token),
            to(KEEP),
        ),
    );
    assert_eq!(state.figure(SOLO), Some(KEEP));
    assert_eq!(state.reputation(SOLO).value(), -1);
    assert_eq!(state.assault(SOLO).map(|assault| assault.from), Some(start));
    let state = done(&state, Action::StartCombat);
    let combat = state.combat(SOLO).expect("combat de l'assaut");
    let defender = combat.enemy(guardsmen()).expect("les Guardsmen défendent");
    assert!(!defender.site_fortified);
    assert_eq!(defender.fortification(), 1);
}

// Spec 062 C18 — Carte ; Compendium, *Cities* S10, *Forced Withdrawal* S4 : l'Attaque souterraine
// assaille une cité non conquise ; ses défenseurs face cachée sont retournés à l'engagement, sans la
// fortification du site ; la retraite ramène à la case de départ du sort.
#[test]
fn underground_attack_assaults_a_city() {
    let underground = spell(UndergroundTravel);
    let mut events = City {
        time: TimeOfDay::Night,
        hand: vec![underground],
        ..City::default()
    }
    .events();
    events.push(black_token());
    let state = events.iter().fold(GameState::default(), apply);
    assert!(destinations(&state, underground, true).contains(&CITY));
    let state = done(
        &state,
        cast_with(
            underground,
            0,
            GREEN_DIE_0,
            Some(ManaPayment::Token),
            to(CITY),
        ),
    );
    assert_eq!(state.figure(SOLO), Some(CITY));
    assert_eq!(
        state.assault(SOLO).map(|assault| assault.from),
        Some(VILLAGE)
    );
    let events = events_of(&state, &Action::StartCombat);
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(event, Event::EnemyTokenRevealed { .. }))
            .count(),
        3
    );
    let state = events.iter().fold(state, apply);
    let combat = state.combat(SOLO).expect("combat de l'assaut");
    assert_eq!(combat.enemies.len(), 3);
    assert!(combat.enemies.iter().all(|enemy| !enemy.site_fortified));
}

// --- Ailes du vent et Ailes de la nuit ---

fn distance(a: Hex, b: Hex) -> i32 {
    let dq = b.q - a.q;
    let dr = b.r - a.r;
    (dq.abs() + dr.abs() + (dq + dr).abs()) / 2
}

fn basic_of(card: CardId) -> Action {
    Action::PlayCard {
        card,
        play: CardPlay::Basic {
            option: 0,
            discard: None,
            mana: None,
            with: None,
            gain: None,
            die: None,
        },
    }
}

fn swordsmen() -> EnemyToken {
    token(arcanum_core::Enemy::Swordsmen, 1)
}

fn on_enemies(enemies: &[EnemyToken]) -> CastTargets {
    CastTargets {
        enemies: Picks::of(enemies),
        ..CastTargets::default()
    }
}

/// Les ensembles d'ennemis visés par les lancers légaux de `card`.
fn enemy_targets(state: &GameState, card: CardId) -> Vec<Vec<EnemyToken>> {
    let mut sets = Vec::new();
    for play in casts_of(state, card) {
        if let CardPlay::Cast { targets, .. } = play
            && !targets.enemies.is_empty()
            && !sets.contains(&targets.enemies.to_vec())
        {
            sets.push(targets.enemies.to_vec());
        }
    }
    sets
}

// Spec 057 C7
#[test]
fn wings_of_wind_spends_move_points() {
    // Carte ; Compendium, *Wings of Wind* S5 : une case par point dépensé, 5 au plus, sans
    // provoquer.
    let wings = spell(WingsOfWind);
    let swiftness = card(arcanum_core::BasicAction::Swiftness, 1);
    let state = Decor {
        figure: Hex::ORIGIN,
        hand: vec![wings, card(March, 1), swiftness],
        ..Decor::default()
    }
    .state();
    let state = done(&state, march_basic());
    let state = done(&state, basic_of(swiftness));
    assert_eq!(state.move_points(SOLO), 4);
    let hexes = destinations(&state, wings, false);
    assert!(hexes.contains(&VILLAGE), "{hexes:?}");
    assert!(hexes.iter().all(|&hex| distance(Hex::ORIGIN, hex) <= 4));
    assert!(!hexes.contains(&ORCS_2) && !hexes.contains(&LAKE));
    let events = events_of(&state, &cast_with(wings, 0, WHITE_DIE_3, None, to(VILLAGE)));
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::EnemiesProvoked { .. }))
    );
    let state = events.iter().fold(state, apply);
    assert_eq!(state.figure(SOLO), Some(VILLAGE));
    assert_eq!(state.move_points(SOLO), 1);
    // 5 cases au plus, même avec 6 points.
    let fortified = Fortified {
        figure: Hex::ORIGIN,
        hand: vec![wings, card(March, 1), card(March, 2), swiftness],
        ..Fortified::default()
    }
    .state();
    let mut state = fortified;
    for march in [card(March, 1), card(March, 2), swiftness] {
        state = done(&state, basic_of(march));
    }
    assert_eq!(state.move_points(SOLO), 6);
    let hexes = destinations(&state, wings, false);
    assert!(hexes.contains(&TWO_FROM_KEEP), "{hexes:?}");
    assert!(!hexes.contains(&NEAR_KEEP));
}

// Spec 057 C8, L2
#[test]
fn wings_of_night_extra_targets_cost_move() {
    // Carte (LL) ; Compendium, *Wings of Night* S1 : 0 point pour le 1er ennemi, 1 pour le 2e, 2
    // pour le 3e.
    let wings = spell(WingsOfWind);
    let state = Decor {
        time: TimeOfDay::Night,
        hand: vec![wings, card(March, 1)],
        placed: vec![
            (prowlers(), ORCS_2),
            (swordsmen(), ORCS_2),
            (wolf_riders(), ORCS_2),
        ],
        extra: vec![black_token()],
        ..Decor::default()
    }
    .fighting();
    let state = done(&state, march_basic());
    assert_eq!(state.move_points(SOLO), 2);
    let sets = enemy_targets(&state, wings);
    assert!(sets.contains(&vec![prowlers(), swordsmen()]), "{sets:?}");
    assert!(!sets.iter().any(|set| set.len() == 3));
    // L2 : deux fois le même ennemi.
    refused(
        &state,
        cast_with(
            wings,
            0,
            WHITE_DIE_3,
            Some(ManaPayment::Token),
            on_enemies(&[prowlers(), prowlers()]),
        ),
    );
    let state = done(
        &state,
        cast_with(
            wings,
            0,
            WHITE_DIE_3,
            Some(ManaPayment::Token),
            on_enemies(&[prowlers(), swordsmen()]),
        ),
    );
    assert_eq!(state.move_points(SOLO), 1);
    let combat = state.combat(SOLO).expect("combat");
    assert!(
        combat
            .enemy(prowlers())
            .is_some_and(|enemy| enemy.does_not_attack())
    );
    assert!(
        combat
            .enemy(swordsmen())
            .is_some_and(|enemy| enemy.does_not_attack())
    );
    assert!(
        !combat
            .enemy(wolf_riders())
            .is_some_and(|enemy| enemy.does_not_attack())
    );
}

// Spec 057 C9
#[test]
fn move_effects_in_combat_for_wings_of_night() {
    // MKUE p. 6, 5.c ; Q4 : le Mouvement se joue en combat quand Ailes de la nuit peut être lancé.
    let march = card(March, 1);
    let moves = |state: &GameState| {
        plays_of(state, march)
            .iter()
            .any(|play| matches!(play, CardPlay::Basic { option: 0, .. }))
    };
    let without = Decor {
        time: TimeOfDay::Night,
        hand: vec![march],
        extra: vec![black_token()],
        ..Decor::default()
    }
    .fighting();
    assert!(!moves(&without));
    let with = Decor {
        time: TimeOfDay::Night,
        hand: vec![spell(WingsOfWind), march],
        extra: vec![black_token()],
        ..Decor::default()
    }
    .fighting();
    assert!(moves(&with));
    // De jour, pas de mana noir : pas d'Ailes de la nuit.
    let day = Decor {
        hand: vec![spell(WingsOfWind), march],
        ..Decor::default()
    }
    .fighting();
    assert!(!moves(&day));
}

// --- Distorsion spatiale et Distorsion temporelle ---

const BLUE_DIE_2: ManaPayment = ManaPayment::Die(2);

// Spec 057 C12
#[test]
fn space_bending_two_spaces_as_adjacent() {
    // Carte ; Compendium, *Space Bending* Q1, S3 : une case à 2 comme une voisine, à son coût ;
    // aucun déplacement du tour ne provoque.
    let bending = spell(SpaceBending);
    let state = Decor {
        figure: Hex::ORIGIN,
        hand: vec![bending, card(March, 1)],
        ..Decor::default()
    }
    .state();
    let state = done(&state, cast(bending, 0, BLUE_DIE_2, None));
    let state = done(&state, march_basic());
    let state = done(&state, Action::MoveHero { to: PLAIN_SW });
    assert_eq!(state.figure(SOLO), Some(PLAIN_SW));
    assert_eq!(state.move_points(SOLO), 0);
    // Un pas ordinaire ne provoque pas les orques.
    let state = Decor {
        hand: vec![bending, card(March, 1)],
        ..Decor::default()
    }
    .state();
    let state = done(&state, cast(bending, 0, BLUE_DIE_2, None));
    let state = done(&state, march_basic());
    let events = events_of(&state, &Action::MoveHero { to: VILLAGE });
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::EnemiesProvoked { .. })),
        "{events:?}"
    );
    // Les trajets en tiennent compte : le village à 2 cases du portail, pour 2 points.
    let state = Decor {
        figure: Hex::ORIGIN,
        hand: vec![bending, card(March, 1), card(March, 2)],
        ..Decor::default()
    }
    .state();
    let state = done(&state, cast(bending, 0, BLUE_DIE_2, None));
    let state = done(&state, march_basic());
    let state = done(&state, basic_of(card(March, 2)));
    let state = done(&state, Action::FollowRoute { to: VILLAGE });
    assert_eq!(state.figure(SOLO), Some(VILLAGE));
}

// Spec 057 C13
#[test]
fn space_bending_explores_two_spaces_away() {
    // Compendium, *Space Bending* S2 ; MKUE p. 7, 5 : explorer depuis une case à 2 de
    // l'emplacement, pour 2 points.
    let bending = spell(SpaceBending);
    let position = arcanum_core::TilePosition::from_lattice(1, 1);
    let decor = Decor {
        figure: PLAIN_SW,
        hand: vec![bending, card(March, 1)],
        next: vec![arcanum_core::TileId::Countryside(3)],
        ..Decor::default()
    };
    let state = done(&decor.state(), march_basic());
    assert!(!legal_actions(&state, SOLO).contains(&Action::Explore { position }));
    let state = done(&state, cast(bending, 0, BLUE_DIE_2, None));
    assert!(legal_actions(&state, SOLO).contains(&Action::Explore { position }));
}

// Spec 057 C14
#[test]
fn space_bending_assault_retreat_and_no_reveal() {
    // Compendium, *Space Bending* S5, S7 : l'assaut repoussé ramène à la case du saut ; de jour,
    // une case à 2 ne révèle pas les défenseurs.
    let bending = spell(SpaceBending);
    let hand = vec![bending, card(March, 1), card(March, 2)];
    let state = Fortified {
        figure: TWO_FROM_KEEP,
        hand: hand.clone(),
        ..Fortified::default()
    }
    .state();
    let state = done(&state, cast(bending, 0, BLUE_DIE_2, None));
    let state = done(&state, march_basic());
    let state = done(&state, basic_of(card(March, 2)));
    let jumped = done(
        &state,
        Action::MoveHero {
            to: Hex::new(6, -3),
        },
    );
    assert_eq!(jumped.figure(SOLO), Some(Hex::new(6, -3)));
    assert!(jumped.is_face_down(KEEP));
    let assault = done(&state, Action::MoveHero { to: KEEP });
    assert_eq!(
        assault.assault(SOLO).map(|assault| assault.from),
        Some(TWO_FROM_KEEP)
    );
}

/// La première fin de tour légale.
fn end_turn(state: &GameState) -> Action {
    legal_actions(state, SOLO)
        .into_iter()
        .find(|action| matches!(action, Action::EndTurn { .. }))
        .expect("finir le tour est légal")
}

fn improvised(card: CardId, discard: CardId) -> Action {
    Action::PlayCard {
        card,
        play: CardPlay::Basic {
            option: 0,
            discard: Some(discard),
            mana: None,
            with: None,
            gain: None,
            die: None,
        },
    }
}

fn has(events: &[Event], pick: impl Fn(&Event) -> bool) -> bool {
    events.iter().any(pick)
}

// Spec 057 C15
#[test]
fn time_bending_extra_turn() {
    // Carte ; MKUE p. 11 ; Compendium, *Time Bending* S7, S13 ; Q3 : les cartes jouées reviennent
    // en main, sans pioche ; un tour supplémentaire ; la carte écartée jusqu'à la fin de la manche.
    use arcanum_core::BasicAction::{Concentration, Improvisation, Stamina, Swiftness};
    let bending = spell(SpaceBending);
    let other = CardId::new(DeedCard::Spell(SpaceBending), 2);
    let played = [
        card(March, 1),
        card(Swiftness, 1),
        card(Stamina, 1),
        card(Improvisation, 1),
    ];
    let mut hand = vec![bending, other, card(Concentration, 1)];
    hand.extend(played);
    let state = Decor {
        time: TimeOfDay::Night,
        hand,
        extra: vec![black_token()],
        ..Decor::default()
    }
    .state();
    let state = done(&state, march_basic());
    let state = done(&state, sideways(card(Swiftness, 1)));
    let state = done(&state, sideways(card(Stamina, 1)));
    let state = done(
        &state,
        improvised(card(Improvisation, 1), card(Concentration, 1)),
    );
    let state = done(
        &state,
        cast(bending, 0, BLUE_DIE_2, Some(ManaPayment::Token)),
    );
    let turn = state.turn();
    let events = events_of(&state, &end_turn(&state));
    assert!(
        !has(&events, |event| matches!(event, Event::CardDrawn { .. })),
        "{events:?}"
    );
    assert!(has(&events, |event| matches!(
        event,
        Event::ExtraTurnStarted { .. }
    )));
    let state = events.iter().fold(state, apply);
    for card in played {
        assert!(state.hand(SOLO).contains(&card), "{card:?} en main");
    }
    assert!(state.discard_pile(SOLO).contains(&card(Concentration, 1)));
    assert!(!state.hand(SOLO).contains(&bending));
    assert_eq!(state.set_aside(SOLO), [bending]);
    assert!(state.extra_turn(SOLO));
    assert_eq!(state.turn(), turn);
    assert!(state.mana_tokens(SOLO).is_empty());
    // Pendant le tour supplémentaire, Distorsion temporelle n'est pas proposée (S13).
    let state = apply(apply(state, &black_token()), &crystal(BasicColor::Blue));
    let casts = casts_of(&state, other);
    assert!(
        casts
            .iter()
            .any(|play| matches!(play, CardPlay::Cast { black: None, .. }))
    );
    assert!(
        !casts
            .iter()
            .any(|play| matches!(play, CardPlay::Cast { black: Some(_), .. })),
        "{casts:?}"
    );

    // À la fin de la manche, la carte écartée revient dans le deck (Q3) : ici le deck entier en
    // main, la fin de la manche s'annonce au début du tour supplémentaire.
    let mut hand = vec![bending];
    hand.extend(arcanum_core::Hero::Arythea.starting_deck());
    let state = Decor {
        time: TimeOfDay::Night,
        hand,
        extra: vec![black_token()],
        ..Decor::default()
    }
    .state();
    let state = done(
        &state,
        cast(bending, 0, BLUE_DIE_2, Some(ManaPayment::Token)),
    );
    let state = done(&state, end_turn(&state));
    assert!(state.extra_turn(SOLO));
    let events = events_of(&state, &Action::AnnounceEndOfRound);
    assert!(
        has(&events, |event| matches!(
            event,
            Event::DeckShuffled { order, .. } if order.contains(&bending)
        )),
        "{events:?}"
    );
    let state = events.iter().fold(state, apply);
    assert!(state.set_aside(SOLO).is_empty());
    assert!(!state.extra_turn(SOLO));
}

// Spec 057, Don pour la magie
#[test]
fn time_bending_from_offer_sets_magic_talent_aside() {
    // MKUE p. 5, *"Composite" Effects* 1.c : Don pour la magie est écarté à la place du sort, qui
    // reste dans l'offre.
    use arcanum_core::BasicAction::Stamina;
    let talent = advanced(AdvancedAction::MagicTalent);
    let stamina = card(Stamina, 1);
    let mut extra = spell_offer(&[spell(SpaceBending)]);
    extra.extend([crystal(BasicColor::Blue), black_token()]);
    let state = Decor {
        time: TimeOfDay::Night,
        hand: vec![talent, stamina, card(March, 1)],
        extra,
        ..Decor::default()
    }
    .state();
    let state = done(&state, improvised(talent, stamina));
    let from_offer = Action::PlayCard {
        card: spell(SpaceBending),
        play: CardPlay::Cast {
            option: 0,
            mana: ManaPayment::Crystal,
            black: Some(ManaPayment::Token),
            extra_mana: None,
            enemy: None,
            from_offer: true,
            targets: CastTargets::default(),
        },
    };
    let state = done(&state, from_offer);
    let state = done(&state, end_turn(&state));
    assert_eq!(state.set_aside(SOLO), [talent]);
    assert!(state.extra_turn(SOLO));
    assert!(state.discard_pile(SOLO).contains(&stamina));
    assert!(
        state
            .offer(arcanum_core::Offer::Spells)
            .iter()
            .any(|card| card.deed() == Some(spell(SpaceBending)))
    );
}

// Spec 057 C16
#[test]
fn time_bending_after_end_announced() {
    // Compendium, *Time Bending* S2, S10 : au dernier tour, le tour supplémentaire a lieu avant
    // la fin de la partie (le bac à sable, dernier tour annoncé comme dans Première
    // reconnaissance).
    let bending = spell(SpaceBending);
    let state = Decor {
        time: TimeOfDay::Night,
        hand: vec![bending, card(March, 1)],
        extra: vec![
            Event::LastTurnAnnounced { player: SOLO },
            Event::TurnStarted {
                player: SOLO,
                turn: 2,
            },
            black_token(),
        ],
        ..Decor::default()
    }
    .state();
    assert_eq!(state.last_turn(), arcanum_core::LastTurn::Current);
    let state = done(
        &state,
        cast(bending, 0, BLUE_DIE_2, Some(ManaPayment::Token)),
    );
    let events = events_of(&state, &end_turn(&state));
    assert!(!has(&events, |event| matches!(
        event,
        Event::GameEnded { .. }
    )));
    assert!(has(&events, |event| matches!(
        event,
        Event::ExtraTurnStarted { .. }
    )));
    let state = events.iter().fold(state, apply);
    let state = done(&state, march_basic());
    let events = events_of(&state, &end_turn(&state));
    assert!(matches!(events.last(), Some(Event::GameEnded { .. })));
}

// --- Appel aux armes et Appel à la gloire ---

/// Les activations légales de `unit`.
fn activations_of(state: &GameState, of: UnitCard) -> Vec<Action> {
    legal_actions(state, SOLO)
        .into_iter()
        .filter(|action| matches!(action, Action::ActivateUnit { unit, .. } if *unit == of))
        .collect()
}

fn in_unit_offer(state: &GameState, of: UnitCard) -> bool {
    state
        .offer(arcanum_core::Offer::Units)
        .iter()
        .any(|card| card.unit() == Some(of))
}

// Spec 057 C17
#[test]
fn call_to_arms_uses_offer_unit() {
    // Carte ; Compendium, *Call to Arms* S2, S3, S15 ; Q6 : une capacité d'une unité de l'offre,
    // une fois ; l'unité reste dans l'offre et ne prend pas de dégâts.
    let call = spell(CallToArms);
    let (crossbowmen, golems, swordsmen) = (
        unit(Unit::UtemCrossbowmen, 1),
        unit(Unit::GuardianGolems, 1),
        unit(Unit::UtemSwordsmen, 1),
    );
    let decor = Decor {
        hand: vec![call, card(March, 1)],
        offer: vec![crossbowmen, golems, swordsmen],
        extra: vec![crystal(BasicColor::White)],
        ..Decor::default()
    };
    let state = decor.fighting();
    assert!(activations_of(&state, crossbowmen).is_empty());
    let state = done(&state, cast(call, 0, ManaPayment::Crystal, None));
    let state = done(&state, declare(&[prowlers()]));
    let ranged = Action::ActivateUnit {
        unit: crossbowmen,
        ability: 1,
        option: 0,
        mana: Vec::new(),
        target: None,
        enemy: None,
        leadership: false,
    };
    assert!(legal_actions(&state, SOLO).contains(&ranged));
    let after = done(&state, ranged);
    assert!(in_unit_offer(&after, crossbowmen));
    assert!(after.units(SOLO).is_empty());
    assert!(activations_of(&after, crossbowmen).is_empty());
    assert!(activations_of(&after, golems).is_empty());

    // En parade : les Golems gardiens paient leur mana rouge pour la Parade de feu 4 ; les
    // Guerriers d'Utem ne proposent que leur première capacité.
    let blocking = done(
        &to_block_phase(&decor.fighting()),
        declare_block(prowlers()),
    );
    let blocking = done(&blocking, cast(call, 0, ManaPayment::Crystal, None));
    let fire_block: Vec<Action> = activations_of(&blocking, golems)
        .into_iter()
        .filter(|action| matches!(action, Action::ActivateUnit { ability: 1, .. }))
        .collect();
    assert!(!fire_block.is_empty());
    assert!(
        fire_block.iter().all(
            |action| matches!(action, Action::ActivateUnit { mana, .. } if mana == &[RED_DIE_1])
        )
    );
    let swords = activations_of(&blocking, swordsmen);
    assert!(!swords.is_empty());
    assert!(
        swords
            .iter()
            .all(|action| matches!(action, Action::ActivateUnit { ability: 0, .. }))
    );

    // Dans le combat d'un donjon, sans unités : rien (spec 044).
    let state = Adventure {
        hand: vec![call, card(March, 1)],
        ..Adventure::default()
    }
    .state();
    let mut events = arene::unit_events(&[], &[crossbowmen]);
    events.push(crystal(BasicColor::White));
    let state = events.iter().fold(state, apply);
    let entered = done(&state, Action::EnterAdventureSite);
    assert!(casts_of(&entered, call).is_empty());
    assert!(activations_of(&entered, crossbowmen).is_empty());
}

// Spec 057 C18
#[test]
fn call_to_glory_recruits_free() {
    // Carte ; Compendium, *Call to Glory* S2 ; spec 053 Q5 : sans Influence ni site, un
    // jeton de commandement : les Paysans renvoyés ; pas compté pour Récit héroïque.
    let call = spell(CallToArms);
    let peasants = unit(Unit::Peasants, 1);
    let swordsmen = unit(Unit::UtemSwordsmen, 1);
    let tale = advanced(AdvancedAction::HeroicTale);
    let state = Decor {
        time: TimeOfDay::Night,
        hand: vec![call, tale, card(March, 1)],
        owned: vec![peasants],
        offer: vec![swordsmen],
        extra: vec![black_token(), crystal(BasicColor::White)],
        ..Decor::default()
    }
    .state();
    let state = done(&state, basic_of(tale));
    let recruit = Action::RecruitUnit {
        unit: swordsmen,
        disband: Some(peasants),
        bonds: false,
    };
    assert!(!legal_actions(&state, SOLO).contains(&recruit));
    let state = done(
        &state,
        cast(call, 0, ManaPayment::Crystal, Some(ManaPayment::Token)),
    );
    assert!(legal_actions(&state, SOLO).contains(&recruit));
    let events = events_of(&state, &recruit);
    assert!(events.contains(&Event::UnitRecruited {
        player: SOLO,
        unit: swordsmen,
        cost: 0,
    }));
    let state = events.iter().fold(state, apply);
    assert!(owned(&state, swordsmen).ready);
    assert!(state.units(SOLO).iter().all(|owned| owned.card != peasants));
    assert!(
        !legal_actions(&state, SOLO)
            .iter()
            .any(|action| matches!(action, Action::RecruitUnit { .. }))
    );
    let events = events_of(&state, &end_turn(&state));
    assert!(
        !has(&events, |event| matches!(
            event,
            Event::ReputationChanged { .. }
        )),
        "{events:?}"
    );
}

// Spec 057 L6
#[test]
fn call_to_glory_needs_unit_offer() {
    let call = spell(CallToArms);
    let state = Decor {
        time: TimeOfDay::Night,
        hand: vec![call],
        extra: vec![black_token(), crystal(BasicColor::White)],
        ..Decor::default()
    }
    .state();
    assert!(
        !casts_of(&state, call)
            .iter()
            .any(|play| matches!(play, CardPlay::Cast { black: Some(_), .. }))
    );
}

// --- Méditation et Transe ---

/// Cinq cartes du deck de départ d'Arythea, mises dans la défausse par le décor.
fn discarded() -> Vec<CardId> {
    use arcanum_core::BasicAction::{Concentration, Promise, Stamina, Swiftness, Threaten};
    vec![
        card(Stamina, 1),
        card(Swiftness, 1),
        card(Concentration, 1),
        card(Promise, 1),
        card(Threaten, 1),
    ]
}

/// Le décor de Méditation : en main le sort, Marche et `discard`, puis `discard` défaussées.
fn meditation_decor(time: TimeOfDay, discard: &[CardId]) -> Decor {
    let mut hand = vec![spell(Meditation), card(March, 1)];
    hand.extend(discard);
    let mut extra: Vec<Event> = discard
        .iter()
        .map(|&card| Event::CardDiscarded { player: SOLO, card })
        .collect();
    extra.push(black_token());
    Decor {
        time,
        hand,
        extra,
        ..Decor::default()
    }
}

fn place_orders(state: &GameState) -> Vec<(bool, Vec<CardId>)> {
    legal_actions(state, SOLO)
        .into_iter()
        .filter_map(|action| match action {
            Action::PlaceOnDeck { top, order } => Some((top, order)),
            _ => None,
        })
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

// Spec 057 C19
#[test]
fn meditation_random_cards_on_deck() {
    // Carte ; Compendium, *Meditation* S2 : deux cartes tirées au hasard, remises sur le deck dans
    // l'ordre choisi ; la pioche suivante va jusqu'à la limite + 2.
    let meditation = spell(Meditation);
    let state = meditation_decor(TimeOfDay::Day, &discarded()).state();
    let events = events_of(&state, &cast(meditation, 0, GREEN_DIE_0, None));
    let picked = events
        .iter()
        .find_map(|event| match event {
            Event::DiscardPicked { cards, .. } => Some(cards.clone()),
            _ => None,
        })
        .expect("deux cartes tirées");
    assert_eq!(picked.len(), 2);
    assert!(picked.iter().all(|card| discarded().contains(card)));
    assert!(events.iter().any(Event::blocks_undo));
    let state = events.iter().fold(state, apply);
    // L4 : tant que les cartes attendent, seule leur place se choisit.
    let legal = legal_actions(&state, SOLO);
    assert_eq!(legal.len(), 4, "{legal:?}");
    let reversed = vec![picked[1], picked[0]];
    let place = Action::PlaceOnDeck {
        top: true,
        order: reversed.clone(),
    };
    assert!(legal.contains(&place));
    let state = done(&state, place);
    assert_eq!(&state.deck(SOLO)[..2], reversed.as_slice());
    assert_eq!(state.discard_pile(SOLO).len(), 3);
    let state = done(&state, march_basic());
    let events = events_of(&state, &end_turn(&state));
    let cards = drawn(&events);
    assert_eq!(cards.len(), 7);
    assert_eq!(&cards[..2], reversed.as_slice());
    let state = events.iter().fold(state, apply);
    assert!(state.modifiers(SOLO).is_empty());
}

// Spec 057 C20
#[test]
fn trance_chooses_cards() {
    // Carte ; Compendium, *Meditation* S4 : deux cartes choisies, sous le deck ; moins s'il y en a
    // moins ; jouable défausse vide.
    let trance = spell(Meditation);
    let strong = cast(trance, 0, GREEN_DIE_0, Some(ManaPayment::Token));
    let state = meditation_decor(TimeOfDay::Night, &discarded()).state();
    let state = done(&state, strong.clone());
    let orders = place_orders(&state);
    assert_eq!(orders.len(), 2 * 5 * 4);
    let chosen = vec![discarded()[4], discarded()[0]];
    let state = done(
        &state,
        Action::PlaceOnDeck {
            top: false,
            order: chosen.clone(),
        },
    );
    let deck = state.deck(SOLO);
    assert_eq!(&deck[deck.len() - 2..], chosen.as_slice());

    let one = &discarded()[..1];
    let state = done(
        &meditation_decor(TimeOfDay::Night, one).state(),
        strong.clone(),
    );
    assert_eq!(
        place_orders(&state),
        [(true, one.to_vec()), (false, one.to_vec())]
    );

    let state = done(&meditation_decor(TimeOfDay::Night, &[]).state(), strong);
    assert!(place_orders(&state).is_empty());
    let state = done(&state, march_basic());
    assert_eq!(drawn(&events_of(&state, &end_turn(&state))).len(), 7);
}

// Spec 057 C21
#[test]
fn meditation_draw_bonus_carries_over() {
    // Compendium, *Meditation* S1, O3 : deck vide, la pioche de la nouvelle manche profite des
    // cartes en plus ; deux Méditations : + 4.
    let meditation = spell(Meditation);
    let mut hand = vec![meditation];
    hand.extend(arcanum_core::Hero::Arythea.starting_deck());
    let state = Decor {
        hand,
        ..Decor::default()
    }
    .state();
    let state = done(&state, cast(meditation, 0, GREEN_DIE_0, None));
    let events = events_of(&state, &end_turn(&state));
    assert!(drawn(&events).is_empty());
    let state = events.iter().fold(state, apply);
    let events = events_of(&state, &Action::AnnounceEndOfRound);
    assert_eq!(drawn(&events).len(), 7);

    let other = CardId::new(DeedCard::Spell(Meditation), 2);
    let state = Decor {
        hand: vec![meditation, other, card(March, 1)],
        extra: vec![crystal(BasicColor::Green)],
        ..Decor::default()
    }
    .state();
    let state = done(&state, cast(meditation, 0, GREEN_DIE_0, None));
    let state = done(&state, cast(other, 0, ManaPayment::Crystal, None));
    let state = done(&state, march_basic());
    assert_eq!(drawn(&events_of(&state, &end_turn(&state))).len(), 9);
}

// Spec 057 L4
#[test]
fn place_on_deck_refused_without_pending_cards() {
    let state = meditation_decor(TimeOfDay::Day, &discarded()).state();
    refused(
        &state,
        Action::PlaceOnDeck {
            top: true,
            order: discarded()[..2].to_vec(),
        },
    );
}

// --- Puits de mana ---

const RED: ManaColor = ManaColor::Basic(BasicColor::Red);

fn on_die(die: u8) -> CastTargets {
    CastTargets {
        die: Some(die),
        ..CastTargets::default()
    }
}

/// Le décor de Puits de mana : Source bleue, rouge et noire, de jour, un cristal bleu ; en main
/// le sort et `hand`.
fn claim_decor(hand: &[CardId]) -> GameState {
    let mut cards = vec![spell(ManaClaim)];
    cards.extend(hand);
    Decor {
        hand: cards,
        dice: vec![ManaColor::Basic(BasicColor::Blue), RED, ManaColor::Black],
        extra: vec![crystal(BasicColor::Blue)],
        ..Decor::default()
    }
    .state()
}

fn tokens_gained(events: &[Event]) -> Vec<ManaColor> {
    events
        .iter()
        .filter_map(|event| match event {
            Event::ManaTokenGained { color, .. } => Some(*color),
            _ => None,
        })
        .collect()
}

// Spec 057 C23
#[test]
fn mana_claim_keeps_die_until_end_of_round() {
    // Carte ; Q7 : le dé rouge gardé jusqu'à la fin de la manche, trois jetons rouges ; ce n'est
    // pas le dé du tour.
    use arcanum_core::BasicAction::Stamina;
    let claim = spell(ManaClaim);
    let state = claim_decor(&[card(March, 1), card(Stamina, 1)]);
    let events = events_of(
        &state,
        &cast_with(claim, 0, ManaPayment::Crystal, None, on_die(RED_DIE)),
    );
    assert!(events.contains(&Event::DieKept {
        player: SOLO,
        die: RED_DIE,
    }));
    assert_eq!(tokens_gained(&events), [RED, RED, RED]);
    let state = events.iter().fold(state, apply);
    assert_eq!(state.kept_die(SOLO), Some(RED_DIE));
    assert!(legal_actions(&state, SOLO).contains(&strong(card(Stamina, 1), 0)));
    let state = done(&state, march_basic());
    let events = events_of(&state, &end_turn(&state));
    assert!(!events.iter().any(|event| matches!(
        event,
        Event::DieReturned { die, .. } if *die == RED_DIE
    )));
    let state = events.iter().fold(state, apply);
    assert_eq!(state.kept_die(SOLO), Some(RED_DIE));
    let legal = format!("{:?}", legal_actions(&state, SOLO));
    assert!(!legal.contains("Die(1)"), "{legal}");

    // À la nouvelle manche, le dé revient dans la Source.
    let mut hand = vec![claim];
    hand.extend(arcanum_core::Hero::Arythea.starting_deck());
    let state = Decor {
        hand,
        dice: vec![ManaColor::Basic(BasicColor::Blue), RED, ManaColor::Black],
        extra: vec![crystal(BasicColor::Blue)],
        ..Decor::default()
    }
    .state();
    let state = done(
        &state,
        cast_with(claim, 0, ManaPayment::Crystal, None, on_die(RED_DIE)),
    );
    let state = done(&state, end_turn(&state));
    let state = done(&state, Action::AnnounceEndOfRound);
    assert_eq!(state.kept_die(SOLO), None);
}

// Spec 057 C24
#[test]
fn mana_claim_token_each_turn() {
    // Carte : un jeton rouge au début de chaque tour suivant de la manche ; le dé noir n'est pas
    // proposé.
    let claim = spell(ManaClaim);
    let state = claim_decor(&[card(March, 1)]);
    let dice: Vec<u8> = casts_of(&state, claim)
        .into_iter()
        .filter_map(|play| match play {
            CardPlay::Cast { targets, .. } => targets.die,
            _ => None,
        })
        .collect();
    assert!(dice.contains(&RED_DIE) && dice.contains(&0));
    assert!(!dice.contains(&2));
    let each_turn = cast_with(claim, 1, ManaPayment::Crystal, None, on_die(RED_DIE));
    let events = events_of(&state, &each_turn);
    assert!(tokens_gained(&events).is_empty());
    let state = events.iter().fold(state, apply);
    let state = done(&state, march_basic());
    let events = events_of(&state, &end_turn(&state));
    let started = events
        .iter()
        .position(|event| matches!(event, Event::TurnStarted { .. }))
        .expect("tour suivant");
    assert_eq!(tokens_gained(&events[started..]), [RED]);

    // Plus rien à la manche suivante.
    let mut hand = vec![claim];
    hand.extend(arcanum_core::Hero::Arythea.starting_deck());
    let state = Decor {
        hand,
        dice: vec![ManaColor::Basic(BasicColor::Blue), RED, ManaColor::Black],
        extra: vec![crystal(BasicColor::Blue)],
        ..Decor::default()
    }
    .state();
    let state = done(&state, each_turn);
    let state = done(&state, end_turn(&state));
    let events = events_of(&state, &Action::AnnounceEndOfRound);
    assert!(tokens_gained(&events).is_empty(), "{events:?}");
}

// Spec 057 L5
#[test]
fn mana_claim_needs_basic_die() {
    let claim = spell(ManaClaim);
    let state = Decor {
        hand: vec![claim],
        dice: vec![ManaColor::Black, ManaColor::Gold],
        extra: vec![crystal(BasicColor::Blue)],
        ..Decor::default()
    }
    .state();
    assert!(casts_of(&state, claim).is_empty());
}

// --- Contenu et vue ---

// Spec 057 C1
#[test]
fn all_spells_supported() {
    // Contenu ; Q2 : plus aucun sort n'est seulement jouable de côté.
    use arcanum_core::BasicEffect;
    for card in Spell::ALL {
        let definition = spell(card)
            .card()
            .definition()
            .expect("un sort a une définition");
        assert!(
            !matches!(definition.basic, BasicEffect::Unsupported)
                && !matches!(definition.strong, BasicEffect::Unsupported),
            "{card:?}"
        );
    }
}

// Spec 057 C27
#[test]
fn view_shows_spell_states() {
    // Spec 052 : le dé gardé, les cartes écartées, le tour supplémentaire, les cartes de
    // Méditation en attente, le Mouvement en combat et les modificateurs nouveaux, dans la vue.
    let (claim, bending) = (spell(ManaClaim), spell(SpaceBending));
    let state = Decor {
        time: TimeOfDay::Night,
        hand: vec![claim, bending, card(March, 1)],
        dice: vec![ManaColor::Basic(BasicColor::Blue), RED, ManaColor::Black],
        extra: vec![
            crystal(BasicColor::Blue),
            crystal(BasicColor::Blue),
            black_token(),
        ],
        ..Decor::default()
    }
    .state();
    let state = done(
        &state,
        cast_with(claim, 1, ManaPayment::Crystal, None, on_die(RED_DIE)),
    );
    let state = done(
        &state,
        cast(bending, 0, ManaPayment::Crystal, Some(ManaPayment::Token)),
    );
    let state = done(&state, end_turn(&state));
    let seen = arcanum_core::view(&state, SOLO);
    assert_eq!(seen.kept_die, Some(RED_DIE));
    assert_eq!(seen.set_aside, [bending]);
    assert!(seen.extra_turn);
    assert!(
        seen.modifiers.iter().any(|modifier| modifier.kind
            == arcanum_core::ModifierKind::ManaClaimPerTurn(BasicColor::Red))
    );

    let state = meditation_decor(TimeOfDay::Day, &discarded()).state();
    let state = done(&state, cast(spell(Meditation), 0, GREEN_DIE_0, None));
    let seen = arcanum_core::view(&state, SOLO);
    let pending = seen.meditation.expect("cartes en attente");
    assert_eq!(pending.cards.len(), 2);
    assert!(
        seen.modifiers
            .iter()
            .any(|modifier| modifier.kind == arcanum_core::ModifierKind::DrawBonus(2))
    );

    let fighting = Decor {
        time: TimeOfDay::Night,
        hand: vec![spell(WingsOfWind), card(March, 1)],
        extra: vec![black_token()],
        ..Decor::default()
    }
    .fighting();
    assert!(arcanum_core::view(&fighting, SOLO).fly_in_combat);
}
