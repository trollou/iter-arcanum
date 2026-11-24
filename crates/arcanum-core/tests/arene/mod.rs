//! Arène des tests (specs 030 à 036) : bac à sable construit par événements (`Sandbox`), ennemis
//! errants posés près du village, les actions du combat, et un joueur de test (`Puppet`).

// Module partagé par plusieurs crates de test (deroule_combat, elements_resistances) : chacune
// n'en utilise qu'une partie.
#![allow(dead_code)]

use arcanum_core::{
    Action, ActionError, BasicAction, BasicColor, CardId, CardPlay, CombatPhase, CommonCard,
    CommonDeck, DeedCard, Effect, Enemy, EnemyColor, EnemyToken, Event, EventLog, GameConfig,
    GameContent, GameRunner, GameState, Hero, Hex, ManaColor, ManaPayment, Offer, Player, PlayerId,
    PlayerView, RngPositions, RuinToken, Scenario, SidewaysEffect, StepOutcome, Tactic, TacticRule,
    TileId, TilePosition, TimeOfDay, UnitCard, UnitRank, apply, decide, legal_actions,
};

pub const SOLO: PlayerId = PlayerId::SOLO;

pub fn card(action: BasicAction, copy: u16) -> CardId {
    CardId::new(DeedCard::Basic(action), copy)
}

pub fn wound(copy: u16) -> CardId {
    CardId::new(DeedCard::Wound, copy)
}

pub fn config(seed: u64) -> GameConfig {
    GameConfig {
        seed,
        scenario: Scenario::sandbox(),
        hero: Hero::Tovak,
        dummy_hero: Default::default(),
    }
}

/// Partie de bac à sable construite par événements : les trois tuiles de la mise en place,
/// la figurine sur le portail, le deck `order` (dessus en premier) et ses `drawn` premières
/// cartes en main. Les Blessures y sont mises par le deck : l'événement qui les donne arrive
/// avec la phase d'assignation des dégâts.
pub fn state_with_deck(order: Vec<CardId>, drawn: usize) -> GameState {
    let mut events = vec![
        Event::GameStarted { config: config(0) },
        Event::TilePlaced {
            tile: TileId::StartingA,
            position: TilePosition::ORIGIN,
        },
        Event::TilePlaced {
            tile: TileId::Countryside(1),
            position: TilePosition::from_lattice(0, 1),
        },
        Event::TilePlaced {
            tile: TileId::Countryside(2),
            position: TilePosition::from_lattice(1, 0),
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
    events.extend(
        order
            .into_iter()
            .take(drawn)
            .map(|card| Event::CardDrawn { player: SOLO, card }),
    );
    events.iter().fold(GameState::default(), apply)
}

/// Partie dont la main est exactement `hand`, le deck de départ de Tovak derrière.
pub fn state_with(hand: &[CardId]) -> GameState {
    let mut order = hand.to_vec();
    order.extend(
        Hero::Tovak
            .starting_deck()
            .into_iter()
            .filter(|card| !hand.contains(card)),
    );
    state_with_deck(order, hand.len())
}

/// Événements produits par l'action, qui doit être légale.
pub fn events_of(state: &GameState, action: &Action) -> Vec<Event> {
    decide(state, SOLO, action, &mut config(0).rngs())
        .unwrap_or_else(|error| panic!("{action:?} doit être légale : {error:?}"))
}

/// Vérifie que les événements `expected` apparaissent dans `events`, dans cet ordre, d'autres
/// pouvant s'intercaler (issue privée 110) : un nouvel effet de victoire (réputation, monastère brûlé…)
/// ne casse pas les tests qui ne le concernent pas. Comparer la liste exacte (`assert_eq!`) seulement
/// quand c'est l'objet du test : un ordre complet qui compte pour les règles, ou l'absence d'effet.
#[track_caller]
pub fn assert_events_in_order(events: &[Event], expected: &[Event]) {
    let mut rest = events.iter();
    for wanted in expected {
        assert!(
            rest.any(|event| event == wanted),
            "{wanted:?} absent, ou hors de l'ordre attendu {expected:?}, dans {events:?}"
        );
    }
}

pub fn done(state: &GameState, action: Action) -> GameState {
    events_of(state, &action).iter().fold(state.clone(), apply)
}

pub fn sideways(card: CardId) -> Action {
    Action::PlayCard {
        card,
        play: CardPlay::Sideways(SidewaysEffect::Move),
    }
}

pub fn refused(state: &GameState, action: Action) {
    assert_eq!(
        decide(state, SOLO, &action, &mut config(0).rngs()),
        Err(ActionError::NotLegal),
        "{action:?}"
    );
}

/// Le village de la tuile de campagne 1, voisin des orques de la tuile 2.
pub const VILLAGE: Hex = Hex::new(2, -3);

/// Les orques imprimés de la tuile de campagne 2 (côté N-O).
pub const ORCS_2: Hex = Hex::new(3, -3);

/// Les orques imprimés de la tuile de campagne 1 (côté N-O), loin du village.
pub const ORCS_1: Hex = Hex::new(1, -4);

/// Plaine de la tuile 2, voisine du village et des orques ②.
pub const PLAIN_W: Hex = Hex::new(2, -2);

/// Plaine de la tuile 1, voisine du village mais pas des orques ②.
pub const PLAIN_SW: Hex = Hex::new(1, -2);

/// La mine verte de la tuile 2.
pub const GREEN_MINE: Hex = Hex::new(2, -1);

/// La colline au centre de la tuile 2 : voisine de la plaine, pas du village.
pub const HILL: Hex = Hex::new(3, -2);

pub fn token(enemy: Enemy, copy: u8) -> EnemyToken {
    EnemyToken::new(enemy, copy)
}

pub fn prowlers() -> EnemyToken {
    token(Enemy::Prowlers, 1)
}

pub fn diggers() -> EnemyToken {
    token(Enemy::Diggers, 1)
}

pub fn arythea_config() -> GameConfig {
    GameConfig {
        hero: Hero::Arythea,
        ..config(0)
    }
}

/// Mise en place du bac à sable construite par événements, avec Arythea : tuiles de campagne 1
/// et 2 ; figurine sur `figure` ; la main `hand`, le deck d'Arythea derrière ; les six piles,
/// avec `placed` sur le dessus (dans l'ordre), puis ces jetons posés sur leurs cases.
pub fn arena(figure: Hex, hand: &[CardId], placed: &[(EnemyToken, Hex)]) -> GameState {
    arena_events(figure, hand, placed, &[])
        .iter()
        .fold(GameState::default(), apply)
}

/// La gloire de départ (sans ennemi vaincu sur le plateau) et les niveaux déjà gagnés (spec 035).
pub fn progress(fame: u32, level: u8) -> Vec<Event> {
    let mut events = Vec::new();
    if fame > 0 {
        events.push(Event::EnemiesDefeated {
            player: SOLO,
            enemies: Vec::new(),
            fame,
        });
    }
    events.extend((2..=level).map(|level| Event::LevelGained {
        player: SOLO,
        level,
    }));
    events
}

/// L'arène, avec une gloire et un niveau de départ (spec 035).
pub fn arena_at(
    figure: Hex,
    hand: &[CardId],
    placed: &[(EnemyToken, Hex)],
    fame: u32,
    level: u8,
) -> GameState {
    let mut events = arena_events(figure, hand, placed, &[]);
    events.extend(progress(fame, level));
    events.iter().fold(GameState::default(), apply)
}

/// Les événements de `arena`, avec `next` derrière les deux tuiles de campagne dans la pioche.
pub fn arena_events(
    figure: Hex,
    hand: &[CardId],
    placed: &[(EnemyToken, Hex)],
    next: &[TileId],
) -> Vec<Event> {
    Sandbox {
        figure: Some(figure),
        hand: hand.to_vec(),
        placed: placed.to_vec(),
        next: next.to_vec(),
        ..Sandbox::default()
    }
    .events()
}

/// Mise en place du bac à sable construite par événements (specs 030, 031, 036…), avec le héros
/// `hero` (Arythea par défaut) : tuiles de campagne 1 et 2 puis `next` dans la pioche, la figurine
/// sur `figure` (aucune si `None`), la main `hand` sur le deck de départ du héros, les dés de la
/// Source `dice`, les six piles d'ennemis avec `placed` sur le dessus, puis ces jetons posés sur
/// leurs cases. De nuit, la manche 2 commence avant que la figurine soit posée. Le scénario est le
/// bac à sable, sauf pour les tests de « Première reconnaissance » (spec 063), qui a le même
/// plateau de jeu.
#[derive(Clone, Debug)]
pub struct Sandbox {
    pub scenario: Scenario,
    pub hero: Hero,
    pub figure: Option<Hex>,
    pub hand: Vec<CardId>,
    pub placed: Vec<(EnemyToken, Hex)>,
    pub next: Vec<TileId>,
    pub time: TimeOfDay,
    pub dice: Vec<ManaColor>,
    /// Le dessus de la pile des jetons de ruines, le reste dans l'ordre du contenu (spec 045).
    pub ruins: Vec<RuinToken>,
    /// Spec 060 : avec un scénario à tactiques, le choix attend (`true`) ; sinon la tactique sans
    /// effet du moment est prise, pour que les tests d'avant la spec ne changent pas.
    pub tactic_pending: bool,
}

impl Default for Sandbox {
    fn default() -> Self {
        Self {
            scenario: Scenario::sandbox(),
            hero: Hero::Arythea,
            figure: None,
            hand: Vec::new(),
            placed: Vec::new(),
            next: Vec::new(),
            time: TimeOfDay::Day,
            dice: vec![
                ManaColor::Basic(BasicColor::Green),
                ManaColor::Basic(BasicColor::Red),
                ManaColor::Basic(BasicColor::Blue),
                // Une blanche pour les effets renforcés de Célérité (spec 031, attaques).
                ManaColor::Basic(BasicColor::White),
            ],
            ruins: Vec::new(),
            tactic_pending: false,
        }
    }
}

impl Sandbox {
    /// Le bac à sable des tests de déplacement (specs 030 et 036) : la main `move_hand()`, les
    /// jetons `sandbox_tokens()`, les dés vert, vert et blanc.
    pub fn moving(figure: Option<Hex>) -> Self {
        Self {
            figure,
            hand: move_hand().to_vec(),
            placed: sandbox_tokens(),
            dice: vec![
                ManaColor::Basic(BasicColor::Green),
                ManaColor::Basic(BasicColor::Green),
                ManaColor::Basic(BasicColor::White),
            ],
            ..Self::default()
        }
    }

    pub fn events(&self) -> Vec<Event> {
        let mut order = self.hand.clone();
        order.extend(
            self.hero
                .starting_deck()
                .into_iter()
                .filter(|card| !self.hand.contains(card)),
        );
        let mut tiles = vec![TileId::Countryside(1), TileId::Countryside(2)];
        tiles.extend_from_slice(&self.next);
        let mut events = vec![
            Event::GameStarted {
                config: GameConfig {
                    scenario: self.scenario,
                    hero: self.hero,
                    ..arythea_config()
                },
            },
            Event::TilePlaced {
                tile: TileId::StartingA,
                position: TilePosition::ORIGIN,
            },
            Event::TileDeckFormed {
                order: tiles.clone(),
            },
            Event::TileRevealed {
                tile: tiles[0],
                position: TilePosition::from_lattice(0, 1),
            },
            Event::TileRevealed {
                tile: tiles[1],
                position: TilePosition::from_lattice(1, 0),
            },
        ];
        if self.time == TimeOfDay::Night {
            events.push(Event::RoundStarted {
                round: 2,
                time_of_day: TimeOfDay::Night,
            });
        }
        events.extend(
            self.figure
                .map(|hex| Event::HeroFigurePlaced { player: SOLO, hex }),
        );
        events.push(Event::DeckShuffled {
            player: SOLO,
            order,
        });
        events.extend(
            self.hand
                .iter()
                .map(|&card| Event::CardDrawn { player: SOLO, card }),
        );
        events.push(Event::SourceRolled {
            dice: self.dice.clone(),
        });
        for color in EnemyColor::ALL {
            let mut pile: Vec<EnemyToken> = self
                .placed
                .iter()
                .map(|&(token, _)| token)
                .filter(|token| token.color() == color)
                .collect();
            pile.extend(
                GameContent::embedded()
                    .enemy_tokens(color)
                    .into_iter()
                    .filter(|token| !self.placed.iter().any(|(placed, _)| placed == token)),
            );
            events.push(Event::EnemyPileShuffled { color, order: pile });
        }
        events.push(Event::RuinPileShuffled {
            order: self
                .ruins
                .iter()
                .copied()
                .chain(
                    GameContent::embedded()
                        .ruin_tokens()
                        .into_iter()
                        .filter(|token| !self.ruins.contains(token)),
                )
                .collect(),
        });
        events.extend(
            self.placed
                .iter()
                .map(|&(token, hex)| Event::EnemyTokenPlaced { token, hex }),
        );
        if self.scenario.tactics() != TacticRule::None && !self.tactic_pending {
            events.push(Event::TacticChosen {
                player: SOLO,
                tactic: no_effect_tactics(self.time)[0],
            });
        }
        events
    }

    pub fn state(&self) -> GameState {
        self.events().iter().fold(GameState::default(), apply)
    }
}

/// Spec 060 : les tactiques sans effet de ce moment de la journée, prises par les tests qui ne
/// portent pas sur les tactiques : Lève-tôt (Au crépuscule), ou, une fois retirée, une tactique
/// qui n'agit que si on s'en sert (Au bon moment, Appel du mana).
pub fn no_effect_tactics(time: TimeOfDay) -> [Tactic; 2] {
    match time {
        TimeOfDay::Day => [Tactic::EarlyBird, Tactic::TheRightMoment],
        TimeOfDay::Night => [Tactic::FromTheDusk, Tactic::ManaSearch],
    }
}

/// Spec 060 : la tactique sans effet prise si son choix attend, pour les tests d'avant la spec
/// qui partent de la mise en place de Première reconnaissance ; sinon l'état tel quel.
pub fn past_tactic_choice(state: GameState) -> GameState {
    let legal = legal_actions(&state, SOLO);
    let tactic = no_effect_tactics(state.time_of_day())
        .map(|tactic| Action::ChooseTactic { tactic })
        .into_iter()
        .find(|action| legal.contains(action));
    match tactic {
        Some(action) => done(&state, action),
        None => state,
    }
}

/// La main des tests de déplacement : Marche ×2, Endurance, Improvisation, Concentration.
pub fn move_hand() -> [CardId; 5] {
    [
        card(BasicAction::March, 1),
        card(BasicAction::March, 2),
        card(BasicAction::Stamina, 1),
        card(BasicAction::Improvisation, 1),
        card(BasicAction::Concentration, 1),
    ]
}

/// Les jetons du bac à sable construit : Creuseurs sur les orques ①, Rôdeurs sur les ②.
pub fn sandbox_tokens() -> Vec<(EnemyToken, Hex)> {
    vec![(diggers(), ORCS_1), (prowlers(), ORCS_2)]
}

/// Joueur de test : garde les actions légales reçues et rend le choix qu'on lui donne.
#[derive(Clone, Default)]
pub struct Puppet {
    legal: std::rc::Rc<std::cell::RefCell<Vec<Action>>>,
    choice: std::rc::Rc<std::cell::Cell<Option<usize>>>,
}

impl Player for Puppet {
    fn request(&mut self, _view: &PlayerView, legal_actions: &[Action]) {
        *self.legal.borrow_mut() = legal_actions.to_vec();
    }

    fn poll(&mut self) -> Option<usize> {
        self.choice.take()
    }
}

/// Un déroulement qui reprend le journal `events`, avec `Puppet` pour joueur.
pub fn runner(events: Vec<Event>) -> (GameRunner, Puppet) {
    let mut log = EventLog::default();
    for event in events {
        log.push(0, event);
    }
    let puppet = Puppet::default();
    let runner = GameRunner::resume(log, RngPositions::new(), vec![Box::new(puppet.clone())])
        .expect("journal valide");
    (runner, puppet)
}

/// Fait jouer `action`, qui doit être légale, par `puppet`.
pub fn play(runner: &mut GameRunner, puppet: &Puppet, action: Action) {
    assert_eq!(runner.step(), Ok(StepOutcome::Waiting { player: SOLO }));
    let index = puppet
        .legal
        .borrow()
        .iter()
        .position(|candidate| *candidate == action)
        .unwrap_or_else(|| panic!("{action:?} absente des actions légales"));
    puppet.choice.set(Some(index));
    assert_eq!(runner.step(), Ok(StepOutcome::Advanced));
}

pub fn basic(card: CardId) -> Action {
    Action::PlayCard {
        card,
        play: CardPlay::Basic {
            die: None,
            gain: None,
            with: None,
            option: 0,
            discard: None,
            mana: None,
        },
    }
}

pub fn challenge(enemy: Hex) -> Action {
    Action::Challenge { enemy }
}

/// Positions des dés de la Source de `arena`.
pub const GREEN_DIE: u8 = 0;

pub const RED_DIE: u8 = 1;

pub const WHITE_DIE: u8 = 3;

pub fn wolf_riders() -> EnemyToken {
    token(Enemy::WolfRiders, 1)
}

/// Combat engagé : figurine sur `figure`, main `hand`, `placed` posés, puis les ennemis
/// `challenged` défiés dans l'ordre.
pub fn fight(
    figure: Hex,
    hand: &[CardId],
    placed: &[(EnemyToken, Hex)],
    challenged: &[Hex],
) -> GameState {
    let mut state = arena(figure, hand, placed);
    for &enemy in challenged {
        state = done(&state, challenge(enemy));
    }
    done(&state, Action::StartCombat)
}

/// Passe la parade et les dégâts : phase d'attaque.
pub fn to_attack_phase(state: &GameState) -> GameState {
    let state = done(state, Action::NextCombatPhase);
    let state = done(&state, Action::NextCombatPhase);
    assert_eq!(
        state.combat(SOLO).map(|combat| combat.phase),
        Some(CombatPhase::Attack)
    );
    state
}

/// Passe en phase d'attaque sans subir de dégâts, par l'événement : pour les tests d'attaque que
/// des Blessures assommeraient (spec 034), le héros perdant alors ses cartes.
pub fn skip_to_attack_phase(state: &GameState) -> GameState {
    apply(
        state.clone(),
        &Event::CombatPhaseStarted {
            player: SOLO,
            phase: CombatPhase::Attack,
        },
    )
}

pub fn declare(targets: &[EnemyToken]) -> Action {
    Action::DeclareAttack {
        targets: targets.to_vec(),
    }
}

/// Le jeton de l'ennemi de cette case : celui du combat, sinon le premier du plateau de jeu. Les
/// tests d'avant la spec 044 visent un ennemi par sa case, seul sur elle.
#[track_caller]
pub fn foe(state: &GameState, hex: Hex) -> EnemyToken {
    state
        .combat(SOLO)
        .and_then(|combat| combat.enemies.iter().find(|enemy| enemy.hex == hex))
        .map(|enemy| enemy.token)
        .or_else(|| state.enemy_tokens_at(hex).first().copied())
        .unwrap_or_else(|| panic!("aucun ennemi en {hex:?}"))
}

/// La case de l'ennemi du combat désigné par ce jeton.
fn hex_of(state: &GameState, token: EnemyToken) -> Hex {
    state
        .combat(SOLO)
        .and_then(|combat| combat.enemy(token))
        .map(|enemy| enemy.hex)
        .unwrap_or_else(|| panic!("{token:?} hors du combat"))
}

/// Les cibles de chaque `DeclareAttack` légal, par leur case, dans l'ordre des actions légales.
pub fn attack_targets(state: &GameState) -> Vec<Vec<Hex>> {
    legal_actions(state, SOLO)
        .into_iter()
        .filter_map(|action| match action {
            Action::DeclareAttack { targets } => Some(
                targets
                    .into_iter()
                    .map(|token| hex_of(state, token))
                    .collect(),
            ),
            _ => None,
        })
        .collect()
}

/// L'option `option` de l'effet de base de `card`, sans défausse ni mana.
pub fn option(card: CardId, option: u8) -> Action {
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

/// L'effet renforcé de `card` (option 0), payé par le dé `die`.
pub fn strong(card: CardId, die: u8) -> Action {
    Action::PlayCard {
        card,
        play: CardPlay::Strong {
            gain: None,
            extra_mana: None,
            option: 0,
            discard: None,
            mana: ManaPayment::Die(die),
            set_dice: [None, None],
            with: None,
        },
    }
}

pub fn sideways_attack(card: CardId) -> Action {
    Action::PlayCard {
        card,
        play: CardPlay::Sideways(SidewaysEffect::Attack),
    }
}

/// Les effets des cartes jouables parmi les actions légales.
pub fn playable_effects(state: &GameState) -> Vec<Effect> {
    legal_actions(state, SOLO)
        .iter()
        .filter_map(|action| match action {
            Action::PlayCard { card, play } => card.effect(*play),
            _ => None,
        })
        .collect()
}

pub fn attack_effects(effects: &[Effect]) -> Vec<Effect> {
    effects
        .iter()
        .copied()
        .filter(|effect| matches!(effect, Effect::Attack { .. }))
        .collect()
}

pub const BLUE_DIE: u8 = 2;

pub fn monks() -> EnemyToken {
    token(Enemy::Monks, 1)
}

pub fn ironclads() -> EnemyToken {
    token(Enemy::Ironclads, 1)
}

pub fn orc_summoners() -> EnemyToken {
    token(Enemy::OrcSummoners, 1)
}

/// Passe à la phase suivante depuis la phase d'attaque à distance : la phase de parade.
pub fn to_block_phase(state: &GameState) -> GameState {
    let state = done(state, Action::NextCombatPhase);
    assert_eq!(
        state.combat(SOLO).map(|combat| combat.phase),
        Some(CombatPhase::Block)
    );
    state
}

pub fn declare_block(target: EnemyToken) -> Action {
    Action::DeclareBlock { target }
}

/// Les cibles de chaque `DeclareBlock` légal, par leur case, dans l'ordre des actions légales.
pub fn block_targets(state: &GameState) -> Vec<Hex> {
    legal_actions(state, SOLO)
        .into_iter()
        .filter_map(|action| match action {
            Action::DeclareBlock { target } => Some(hex_of(state, target)),
            _ => None,
        })
        .collect()
}

pub fn sideways_block(card: CardId) -> Action {
    Action::PlayCard {
        card,
        play: CardPlay::Sideways(SidewaysEffect::Block),
    }
}

/// Les effets de Parade des cartes jouables parmi les actions légales.
pub fn block_effects(state: &GameState) -> Vec<Effect> {
    playable_effects(state)
        .into_iter()
        .filter(|effect| matches!(effect, Effect::Block { .. }))
        .collect()
}

pub fn current_phase(state: &GameState) -> Option<CombatPhase> {
    state.combat(SOLO).map(|combat| combat.phase)
}

/// Des points d'attaque ou de parade tous physiques (spec 032).
pub fn physical(points: u32) -> arcanum_core::ElementPoints {
    arcanum_core::ElementPoints {
        physical: points,
        ..arcanum_core::ElementPoints::default()
    }
}

pub fn ice_dragon() -> EnemyToken {
    token(Enemy::IceDragon, 1)
}

pub fn fire_dragon() -> EnemyToken {
    token(Enemy::FireDragon, 1)
}

pub fn high_dragon() -> EnemyToken {
    token(Enemy::HighDragon, 1)
}

/// L'option `option` de l'effet renforcé de `card`, payé par un cristal.
pub fn strong_crystal(card: CardId, option: u8) -> Action {
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

// --- Unités (spec 042) ---

/// Un décor de la spec 042 : l'arène (Arythea), un niveau, des unités déjà dans la zone (dans
/// l'ordre), d'autres dans l'offre d'unités.
#[derive(Clone, Debug)]
pub struct Units {
    pub figure: Hex,
    pub hand: Vec<CardId>,
    pub level: u8,
    pub owned: Vec<UnitCard>,
    pub offer: Vec<UnitCard>,
    pub placed: Vec<(EnemyToken, Hex)>,
    pub dice: Vec<ManaColor>,
}

impl Default for Units {
    fn default() -> Self {
        Self {
            figure: VILLAGE,
            hand: vec![
                card(BasicAction::Promise, 1),
                card(BasicAction::Threaten, 1),
                card(BasicAction::March, 1),
                card(BasicAction::Rage, 1),
            ],
            level: 1,
            owned: Vec::new(),
            offer: Vec::new(),
            placed: Vec::new(),
            dice: Sandbox::default().dice,
        }
    }
}

impl Units {
    pub fn events(&self) -> Vec<Event> {
        let mut events = Sandbox {
            figure: Some(self.figure),
            hand: self.hand.clone(),
            placed: self.placed.clone(),
            dice: self.dice.clone(),
            ..Sandbox::default()
        }
        .events();
        events.extend(progress(0, self.level));
        events.extend(unit_events(&self.owned, &self.offer));
        events
    }

    pub fn state(&self) -> GameState {
        self.events().iter().fold(GameState::default(), apply)
    }
}

/// Les unités `owned` recrutées sans coût, dans l'ordre, puis `offer` dans l'offre d'unités : les
/// pioches d'unités mélangées pour les donner (spec 042).
pub fn unit_events(owned: &[UnitCard], offer: &[UnitCard]) -> Vec<Event> {
    let mut events = Vec::new();
    for (deck, rank) in [
        (CommonDeck::RegularUnits, UnitRank::Regular),
        (CommonDeck::EliteUnits, UnitRank::Elite),
    ] {
        let order = owned
            .iter()
            .chain(offer)
            .filter(|card| card.unit.definition().rank == rank)
            .map(|&card| CommonCard::Unit(card))
            .collect();
        events.push(Event::CommonDeckShuffled { deck, order });
    }
    let dealt = |card: UnitCard| Event::OfferCardDealt {
        offer: Offer::Units,
        card: CommonCard::Unit(card),
    };
    for &unit in owned {
        events.push(dealt(unit));
        events.push(Event::UnitRecruited {
            player: SOLO,
            unit,
            cost: 0,
        });
    }
    events.extend(offer.iter().map(|&card| dealt(card)));
    events
}

/// Spec 041 : le monastère de la campagne 5 posée à l'emplacement A + B (côté N-E).
pub const MONASTERY: Hex = Hex::new(5, -6);

/// Forêt de la campagne 5 (côté N-O), voisine du monastère.
pub const NEAR_MONASTERY: Hex = Hex::new(4, -6);

/// L'emplacement A + B, où la campagne 5 est posée (ou explorée depuis le village).
pub const MONASTERY_POSITION: TilePosition = TilePosition::from_lattice(1, 1);

pub fn advanced(card: arcanum_core::AdvancedAction) -> CardId {
    CardId::new(DeedCard::Advanced(card), 1)
}

pub fn artifact(card: arcanum_core::Artifact) -> CardId {
    CardId::new(DeedCard::Artifact(card), 1)
}

/// Un décor de la spec 041 : l'arène (Arythea) avec la campagne 5 posée en A + B (ou au-dessus de
/// la pioche de tuiles si `revealed` est faux), la pioche d'actions avancées (`monastery_cards`
/// sur le dessus, entrées dans l'offre d'unités à la suite de `offer`), la pioche d'artefacts
/// (`artifacts` sur le dessus), des unités dans la zone et un niveau.
#[derive(Clone, Debug)]
pub struct Monastery {
    pub figure: Hex,
    pub hand: Vec<CardId>,
    pub revealed: bool,
    /// La tuile posée en A + B, ou au-dessus de la pioche : la campagne 5 par défaut.
    pub tile: TileId,
    pub level: u8,
    pub owned: Vec<UnitCard>,
    pub offer: Vec<UnitCard>,
    pub monastery_cards: Vec<CardId>,
    pub artifacts: Vec<CardId>,
    pub placed: Vec<(EnemyToken, Hex)>,
}

impl Default for Monastery {
    fn default() -> Self {
        Self {
            figure: MONASTERY,
            hand: Units::default().hand,
            revealed: true,
            tile: TileId::Countryside(5),
            level: 1,
            owned: Vec::new(),
            offer: Vec::new(),
            monastery_cards: vec![advanced(arcanum_core::AdvancedAction::IceShield)],
            artifacts: Vec::new(),
            placed: Vec::new(),
        }
    }
}

impl Monastery {
    pub fn events(&self) -> Vec<Event> {
        let mut events = Sandbox {
            figure: Some(self.figure),
            hand: self.hand.clone(),
            placed: self.placed.clone(),
            next: vec![self.tile],
            ..Sandbox::default()
        }
        .events();
        if self.revealed {
            events.push(Event::TileRevealed {
                tile: self.tile,
                position: MONASTERY_POSITION,
            });
        }
        events.extend(progress(0, self.level));
        let deck = |top: &[CardId], all: Vec<CardId>| -> Vec<CommonCard> {
            top.iter()
                .copied()
                .chain(all.into_iter().filter(|card| !top.contains(card)))
                .map(CommonCard::Deed)
                .collect()
        };
        events.push(Event::CommonDeckShuffled {
            deck: CommonDeck::AdvancedActions,
            order: deck(
                &self.monastery_cards,
                arcanum_core::AdvancedAction::ALL.map(advanced).to_vec(),
            ),
        });
        events.push(Event::CommonDeckShuffled {
            deck: CommonDeck::Artifacts,
            order: deck(
                &self.artifacts,
                arcanum_core::Artifact::ALL.map(artifact).to_vec(),
            ),
        });
        events.extend(unit_events(&self.owned, &self.offer));
        events.extend(
            self.monastery_cards
                .iter()
                .map(|&card| Event::OfferCardDealt {
                    offer: Offer::Units,
                    card: CommonCard::Deed(card),
                }),
        );
        events
    }

    pub fn state(&self) -> GameState {
        self.events().iter().fold(GameState::default(), apply)
    }
}

// --- Sites fortifiés (spec 043) ---

/// L'emplacement 2A, où la campagne 3 est posée (ou explorée depuis le village de la tuile 2).
pub const KEEP_POSITION: TilePosition = TilePosition::from_lattice(2, 0);

/// La forteresse de la campagne 3 posée en 2A (côté N-E, colline).
pub const KEEP: Hex = Hex::new(7, -5);

/// L'emplacement 2B, où la campagne 4 est posée.
pub const MAGE_TOWER_POSITION: TilePosition = TilePosition::from_lattice(0, 2);

/// La tour de mage de la campagne 4 posée en 2B (centre, désert).
pub const MAGE_TOWER: Hex = Hex::new(2, -6);

/// Plaine de la campagne 3 (côté N-O), voisine de la forteresse.
pub const NEAR_KEEP: Hex = Hex::new(6, -5);

/// Plaine de la campagne 3 (côté O), à deux pas de la forteresse.
pub const TWO_FROM_KEEP: Hex = Hex::new(5, -4);

/// Plaine de la campagne 4 (côté S-E), voisine de la tour de mage.
pub const NEAR_TOWER: Hex = Hex::new(2, -5);

/// La colline des orques de la campagne 4 (côté S-O), voisine de la tour de mage et de `NEAR_TOWER`.
pub const TOWER_ORCS: Hex = Hex::new(1, -5);

pub fn guardsmen() -> EnemyToken {
    token(Enemy::Guardsmen, 1)
}

/// Un décor de la spec 043 : l'arène (Arythea), la campagne 3 posée en 2A (forteresse `KEEP`) et
/// la campagne 4 en 2B (tour de mage `MAGE_TOWER`), chacune avec son défenseur, face cachée ou
/// visible ; d'autres jetons posés (`placed`).
#[derive(Clone, Debug)]
pub struct Fortified {
    pub figure: Hex,
    pub hand: Vec<CardId>,
    pub time: TimeOfDay,
    /// Le défenseur de la forteresse, et s'il est face visible.
    pub keep: Option<(EnemyToken, bool)>,
    pub tower: Option<(EnemyToken, bool)>,
    pub placed: Vec<(EnemyToken, Hex)>,
    pub dice: Vec<ManaColor>,
    /// Unités dans la zone, recrutées sans coût (spec 042).
    pub owned: Vec<UnitCard>,
    /// Sites déjà conquis par le joueur (forteresse, tour de mage), sans défenseur à poser.
    pub conquered: Vec<Hex>,
    /// L'offre d'unités.
    pub offer: Vec<UnitCard>,
    /// L'offre de sorts, dans l'ordre ; vide, la pioche de sorts n'est pas mélangée.
    pub spells: Vec<CardId>,
}

impl Default for Fortified {
    fn default() -> Self {
        Self {
            figure: TWO_FROM_KEEP,
            hand: vec![
                card(BasicAction::March, 1),
                card(BasicAction::March, 2),
                card(BasicAction::Stamina, 1),
                card(BasicAction::Rage, 1),
                card(BasicAction::Promise, 1),
            ],
            time: TimeOfDay::Day,
            keep: Some((guardsmen(), false)),
            tower: Some((token(Enemy::FireMages, 1), false)),
            placed: Vec::new(),
            dice: Sandbox::default().dice,
            owned: Vec::new(),
            conquered: Vec::new(),
            offer: Vec::new(),
            spells: Vec::new(),
        }
    }
}

impl Fortified {
    pub fn defenders(&self) -> Vec<(EnemyToken, Hex, bool)> {
        self.keep
            .map(|(token, up)| (token, KEEP, up))
            .into_iter()
            .chain(self.tower.map(|(token, up)| (token, MAGE_TOWER, up)))
            .collect()
    }

    pub fn events(&self) -> Vec<Event> {
        let defenders = self.defenders();
        let mut placed = self.placed.clone();
        placed.extend(defenders.iter().map(|&(token, hex, _)| (token, hex)));
        let mut events: Vec<Event> = Sandbox {
            figure: Some(self.figure),
            hand: self.hand.clone(),
            placed,
            next: vec![TileId::Countryside(3), TileId::Countryside(4)],
            time: self.time,
            dice: self.dice.clone(),
            ..Sandbox::default()
        }
        .events()
        .into_iter()
        .map(|event| match event {
            Event::EnemyTokenPlaced { token, hex }
                if defenders.iter().any(|&(t, h, _)| (t, h) == (token, hex)) =>
            {
                Event::DefenderPlaced { token, hex }
            }
            other => other,
        })
        .collect();
        // Avant les tuiles : un site conquis avant la partie ne doit pas de récompense sort.
        events.extend(
            self.conquered
                .iter()
                .map(|&hex| Event::SiteConquered { player: SOLO, hex }),
        );
        events.push(Event::TileRevealed {
            tile: TileId::Countryside(3),
            position: KEEP_POSITION,
        });
        events.push(Event::TileRevealed {
            tile: TileId::Countryside(4),
            position: MAGE_TOWER_POSITION,
        });
        events.extend(
            defenders
                .iter()
                .filter(|(_, _, up)| *up)
                .map(|&(token, hex, _)| Event::EnemyTokenRevealed { hex, token }),
        );
        if !self.owned.is_empty() || !self.offer.is_empty() {
            events.extend(unit_events(&self.owned, &self.offer));
        }
        if !self.spells.is_empty() {
            let spell = |spell: arcanum_core::Spell| CardId::new(DeedCard::Spell(spell), 1);
            // Chaque carte entre en tête de l'offre : la dernière est donnée d'abord.
            let dealt: Vec<CardId> = self.spells.iter().rev().copied().collect();
            let order = dealt
                .iter()
                .copied()
                .chain(
                    arcanum_core::Spell::ALL
                        .map(spell)
                        .into_iter()
                        .filter(|card| !self.spells.contains(card)),
                )
                .map(CommonCard::Deed)
                .collect();
            events.push(Event::CommonDeckShuffled {
                deck: CommonDeck::Spells,
                order,
            });
            events.extend(dealt.iter().map(|&card| Event::OfferCardDealt {
                offer: Offer::Spells,
                card: CommonCard::Deed(card),
            }));
        }
        events
    }

    pub fn state(&self) -> GameState {
        self.events().iter().fold(GameState::default(), apply)
    }
}

// --- Sites d'aventure (spec 044) ---

/// L'emplacement A + B, où la campagne 6 est posée (antre).
pub const DEN_POSITION: TilePosition = TilePosition::from_lattice(1, 1);

/// L'antre de la campagne 6 posée en A + B (côté O, colline).
pub const MONSTER_DEN: Hex = Hex::new(3, -5);

/// Colline de la campagne 6 (côté S-O), voisine de l'antre.
pub const NEAR_DEN: Hex = Hex::new(3, -4);

/// L'emplacement 2A, où la campagne 7 est posée (donjon).
pub const DUNGEON_POSITION: TilePosition = TilePosition::from_lattice(2, 0);

/// Le donjon de la campagne 7 posée en 2A (côté S-E, plaine).
pub const DUNGEON: Hex = Hex::new(6, -3);

/// Plaine de la campagne 7 (côté S-O), voisine du donjon.
pub const NEAR_DUNGEON: Hex = Hex::new(5, -3);

/// L'emplacement 2B, où la tuile royaume 1 est posée (tombeau, Sombre Terrier).
pub const CORE_POSITION: TilePosition = TilePosition::from_lattice(0, 2);

/// Le tombeau de la tuile royaume 1 posée en 2B (côté N-E, désert).
pub const TOMB: Hex = Hex::new(3, -7);

/// Le Sombre Terrier de la tuile royaume 1 posée en 2B (côté O, colline).
pub const SPAWNING_GROUNDS: Hex = Hex::new(1, -6);

/// La clairière magique au centre de la campagne 1 (bac à sable).
pub const GLADE: Hex = Hex::new(1, -3);

pub fn minotaur() -> EnemyToken {
    token(Enemy::Minotaur, 1)
}

pub fn medusa() -> EnemyToken {
    token(Enemy::Medusa, 1)
}

pub fn gargoyle() -> EnemyToken {
    token(Enemy::Gargoyle, 1)
}

pub fn swamp_dragon() -> EnemyToken {
    token(Enemy::SwampDragon, 1)
}

/// Un décor de la spec 044 : l'arène (Arythea) avec la campagne 6 en A + B (antre), la campagne 7
/// en 2A (donjon) et la tuile royaume 1 en 2B (tombeau, Sombre Terrier) ; les jetons `brown` et
/// `red` sur le dessus de leurs piles, d'autres posés (`placed` : ennemis restés sur un antre ou
/// un Sombre Terrier, ennemis errants) ; des sites déjà conquis, des unités, un niveau.
#[derive(Clone, Debug)]
pub struct Adventure {
    pub figure: Hex,
    pub hand: Vec<CardId>,
    pub time: TimeOfDay,
    pub dice: Vec<ManaColor>,
    pub placed: Vec<(EnemyToken, Hex)>,
    pub brown: Vec<EnemyToken>,
    pub red: Vec<EnemyToken>,
    pub conquered: Vec<Hex>,
    pub owned: Vec<UnitCard>,
    pub level: u8,
}

impl Default for Adventure {
    fn default() -> Self {
        Self {
            figure: DUNGEON,
            hand: vec![
                card(BasicAction::March, 1),
                card(BasicAction::Rage, 1),
                card(BasicAction::Determination, 1),
                card(BasicAction::Swiftness, 1),
                card(BasicAction::Promise, 1),
            ],
            time: TimeOfDay::Day,
            dice: Sandbox::default().dice,
            placed: Vec::new(),
            brown: vec![minotaur(), medusa()],
            red: vec![swamp_dragon()],
            conquered: Vec::new(),
            owned: Vec::new(),
            level: 1,
        }
    }
}

impl Adventure {
    pub fn events(&self) -> Vec<Event> {
        let tiles = [
            (TileId::Countryside(6), DEN_POSITION),
            (TileId::Countryside(7), DUNGEON_POSITION),
            (TileId::Core(1), CORE_POSITION),
        ];
        let mut events = Sandbox {
            figure: Some(self.figure),
            hand: self.hand.clone(),
            placed: self.placed.clone(),
            next: tiles.iter().map(|&(tile, _)| tile).collect(),
            time: self.time,
            dice: self.dice.clone(),
            ..Sandbox::default()
        }
        .events();
        events.extend(
            tiles
                .iter()
                .map(|&(tile, position)| Event::TileRevealed { tile, position }),
        );
        for (color, top) in [
            (EnemyColor::Brown, &self.brown),
            (EnemyColor::Red, &self.red),
        ] {
            let placed: Vec<EnemyToken> = self.placed.iter().map(|&(token, _)| token).collect();
            let order = top
                .iter()
                .copied()
                .chain(
                    GameContent::embedded()
                        .enemy_tokens(color)
                        .into_iter()
                        .filter(|token| !top.contains(token) && !placed.contains(token)),
                )
                .collect();
            events.push(Event::EnemyPileShuffled { color, order });
        }
        events.extend(
            self.conquered
                .iter()
                .map(|&hex| Event::SiteConquered { player: SOLO, hex }),
        );
        events.extend(progress(0, self.level));
        if !self.owned.is_empty() {
            events.extend(unit_events(&self.owned, &[]));
        }
        events
    }

    pub fn state(&self) -> GameState {
        self.events().iter().fold(GameState::default(), apply)
    }
}

// --- Ruines (spec 045) ---

/// L'emplacement 2A, où la campagne 8 est posée (ruines).
pub const RUINS_POSITION: TilePosition = TilePosition::from_lattice(2, 0);

/// Les ruines de la campagne 8 posée en 2A (côté N-E, forêt).
pub const RUINS: Hex = Hex::new(7, -5);

/// Plaine de la campagne 8 (côté E), voisine des ruines.
pub const NEAR_RUINS: Hex = Hex::new(7, -4);

/// Forêt de la campagne 8 (côté O), à deux pas des ruines.
pub const TWO_FROM_RUINS: Hex = Hex::new(5, -4);

/// La clairière magique de la campagne 8 (côté N-O), voisine des ruines : de nuit, le trajet le
/// moins cher vers `NEAR_RUINS` passe par les ruines.
pub const RUINS_GLADE: Hex = Hex::new(6, -5);

/// Un décor de la spec 045 : l'arène (Arythea) avec la campagne 8 posée en 2A, son jeton de ruines
/// `token` face visible ou cachée sur `RUINS` ; les jetons `tops` sur le dessus de leurs piles,
/// d'autres posés (`placed` : ennemis restés sur la ruine, ennemis errants) ; la ruine déjà
/// conquise si `conquered` ; des unités dans la zone et dans l'offre, une gloire et un niveau.
#[derive(Clone, Debug)]
pub struct Ruins {
    pub figure: Hex,
    pub hand: Vec<CardId>,
    pub time: TimeOfDay,
    pub dice: Vec<ManaColor>,
    pub token: RuinToken,
    pub face_up: bool,
    pub placed: Vec<(EnemyToken, Hex)>,
    pub tops: Vec<EnemyToken>,
    pub conquered: bool,
    pub owned: Vec<UnitCard>,
    pub offer: Vec<UnitCard>,
    pub fame: u32,
    pub level: u8,
    /// Le dessus de la pioche d'artefacts ; vide, elle n'est pas mélangée.
    pub artifacts: Vec<CardId>,
    /// L'offre de sorts, dans l'ordre ; vide, la pioche de sorts n'est pas mélangée.
    pub spells: Vec<CardId>,
    /// L'offre d'actions avancées, dans l'ordre, puis les actions avancées de monastère de l'offre
    /// d'unités ; vides, la pioche d'actions avancées n'est pas mélangée.
    pub advanced: Vec<CardId>,
    pub monastery_advanced: Vec<CardId>,
}

impl Default for Ruins {
    fn default() -> Self {
        Self {
            figure: RUINS,
            hand: Adventure::default().hand,
            time: TimeOfDay::Day,
            dice: Sandbox::default().dice,
            token: RuinToken::BlueAltar,
            face_up: true,
            placed: Vec::new(),
            tops: Vec::new(),
            conquered: false,
            owned: Vec::new(),
            offer: Vec::new(),
            fame: 0,
            level: 1,
            artifacts: Vec::new(),
            spells: Vec::new(),
            advanced: Vec::new(),
            monastery_advanced: Vec::new(),
        }
    }
}

/// La pioche commune `deck` mélangée avec `top` sur le dessus, toutes ses autres cartes derrière.
fn common_deck_with(deck: CommonDeck, top: &[CardId], all: Vec<CardId>) -> Event {
    Event::CommonDeckShuffled {
        deck,
        order: top
            .iter()
            .copied()
            .chain(all.into_iter().filter(|card| !top.contains(card)))
            .map(CommonCard::Deed)
            .collect(),
    }
}

/// `cards` dans l'offre `offer`, depuis le dessus de leur pioche ; une carte d'action avancée ou de
/// sort entre en tête de l'offre : la dernière est donnée d'abord.
fn dealt(offer: Offer, cards: &[CardId]) -> Vec<Event> {
    cards
        .iter()
        .map(|&card| Event::OfferCardDealt {
            offer,
            card: CommonCard::Deed(card),
        })
        .collect()
}

impl Ruins {
    pub fn events(&self) -> Vec<Event> {
        let mut events = Sandbox {
            figure: Some(self.figure),
            hand: self.hand.clone(),
            placed: self.placed.clone(),
            next: vec![TileId::Countryside(8)],
            time: self.time,
            dice: self.dice.clone(),
            ruins: vec![self.token],
            ..Sandbox::default()
        }
        .events();
        let placed: Vec<EnemyToken> = self.placed.iter().map(|&(token, _)| token).collect();
        for color in EnemyColor::ALL {
            let top: Vec<EnemyToken> = self
                .tops
                .iter()
                .copied()
                .filter(|token| token.color() == color)
                .collect();
            if top.is_empty() {
                continue;
            }
            let order = top
                .iter()
                .copied()
                .chain(
                    GameContent::embedded()
                        .enemy_tokens(color)
                        .into_iter()
                        .filter(|token| !top.contains(token) && !placed.contains(token)),
                )
                .collect();
            events.push(Event::EnemyPileShuffled { color, order });
        }
        events.push(Event::TileRevealed {
            tile: TileId::Countryside(8),
            position: RUINS_POSITION,
        });
        events.push(Event::RuinTokenPlaced {
            token: self.token,
            hex: RUINS,
            face_up: self.face_up,
        });
        if self.conquered {
            events.push(Event::SiteConquered {
                player: SOLO,
                hex: RUINS,
            });
        }
        events.extend(progress(self.fame, self.level));
        if !self.owned.is_empty() || !self.offer.is_empty() {
            events.extend(unit_events(&self.owned, &self.offer));
        }
        if !self.artifacts.is_empty() {
            events.push(common_deck_with(
                CommonDeck::Artifacts,
                &self.artifacts,
                arcanum_core::Artifact::ALL.map(artifact).to_vec(),
            ));
        }
        if !self.spells.is_empty() {
            let spell = |spell: arcanum_core::Spell| CardId::new(DeedCard::Spell(spell), 1);
            let reversed: Vec<CardId> = self.spells.iter().rev().copied().collect();
            events.push(common_deck_with(
                CommonDeck::Spells,
                &reversed,
                arcanum_core::Spell::ALL.map(spell).to_vec(),
            ));
            events.extend(dealt(Offer::Spells, &reversed));
        }
        if !self.advanced.is_empty() || !self.monastery_advanced.is_empty() {
            let mut top: Vec<CardId> = self.advanced.iter().rev().copied().collect();
            top.extend(&self.monastery_advanced);
            events.push(common_deck_with(
                CommonDeck::AdvancedActions,
                &top,
                arcanum_core::AdvancedAction::ALL.map(advanced).to_vec(),
            ));
            let reversed: Vec<CardId> = self.advanced.iter().rev().copied().collect();
            events.extend(dealt(Offer::AdvancedActions, &reversed));
            events.extend(dealt(Offer::Units, &self.monastery_advanced));
        }
        events
    }

    pub fn state(&self) -> GameState {
        self.events().iter().fold(GameState::default(), apply)
    }
}

/// L'offre d'actions avancées `cards`, dans l'ordre (première place en tête), et la pioche
/// mélangée avec ces cartes sur le dessus, toutes les autres derrière (spec 056).
pub fn advanced_offer(cards: &[CardId]) -> Vec<Event> {
    let reversed: Vec<CardId> = cards.iter().rev().copied().collect();
    let mut events = vec![common_deck_with(
        CommonDeck::AdvancedActions,
        &reversed,
        arcanum_core::AdvancedAction::ALL.map(advanced).to_vec(),
    )];
    events.extend(dealt(Offer::AdvancedActions, &reversed));
    events
}

/// L'offre de sorts `cards`, dans l'ordre, comme `advanced_offer` (spec 056).
pub fn spell_offer(cards: &[CardId]) -> Vec<Event> {
    let spell = |spell: arcanum_core::Spell| CardId::new(DeedCard::Spell(spell), 1);
    let reversed: Vec<CardId> = cards.iter().rev().copied().collect();
    let mut events = vec![common_deck_with(
        CommonDeck::Spells,
        &reversed,
        arcanum_core::Spell::ALL.map(spell).to_vec(),
    )];
    events.extend(dealt(Offer::Spells, &reversed));
    events
}

// --- Cités (spec 062) ---

/// L'emplacement A + B, où la tuile royaume de cité est posée.
pub const CITY_POSITION: TilePosition = TilePosition::from_lattice(1, 1);

/// La cité, au centre de la tuile royaume posée en A + B.
pub const CITY: Hex = Hex::new(4, -5);

/// Le côté S-O de la tuile de cité, voisin de la cité et du village (collines sur la royaume 6).
pub const NEAR_CITY: Hex = Hex::new(3, -4);

pub fn city_color(tile: TileId) -> BasicColor {
    match tile {
        TileId::Core(5) => BasicColor::Green,
        TileId::Core(6) => BasicColor::Blue,
        TileId::Core(7) => BasicColor::White,
        _ => BasicColor::Red,
    }
}

/// Un décor de la spec 062 : l'arène (Arythea), une tuile royaume de cité posée en A + B, sa cité
/// au niveau 5 avec ses défenseurs (face cachée, ou visibles) ; d'autres jetons posés.
#[derive(Clone, Debug)]
pub struct City {
    pub scenario: Scenario,
    pub tile: TileId,
    pub figure: Hex,
    pub hand: Vec<CardId>,
    pub time: TimeOfDay,
    pub defenders: Vec<EnemyToken>,
    pub face_up: bool,
    pub placed: Vec<(EnemyToken, Hex)>,
}

impl Default for City {
    fn default() -> Self {
        City {
            scenario: Scenario::sandbox(),
            tile: TileId::Core(6),
            figure: VILLAGE,
            hand: vec![
                card(BasicAction::March, 1),
                card(BasicAction::March, 2),
                card(BasicAction::Stamina, 1),
                card(BasicAction::Rage, 1),
                card(BasicAction::Promise, 1),
            ],
            time: TimeOfDay::Day,
            defenders: blue_defenders(),
            face_up: false,
            placed: Vec::new(),
        }
    }
}

impl City {
    pub fn events(&self) -> Vec<Event> {
        let mut placed = self.placed.clone();
        placed.extend(self.defenders.iter().map(|&token| (token, CITY)));
        let mut events: Vec<Event> = Sandbox {
            scenario: self.scenario,
            figure: Some(self.figure),
            hand: self.hand.clone(),
            placed,
            next: vec![self.tile],
            time: self.time,
            ..Sandbox::default()
        }
        .events()
        .into_iter()
        .filter(|event| {
            !matches!(event, Event::EnemyTokenPlaced { token, hex }
                if *hex == CITY && self.defenders.contains(token))
        })
        .collect();
        events.push(Event::TileRevealed {
            tile: self.tile,
            position: CITY_POSITION,
        });
        events.push(Event::CityPlaced {
            hex: CITY,
            color: city_color(self.tile),
            level: 5,
        });
        events.extend(
            self.defenders
                .iter()
                .map(|&token| Event::DefenderPlaced { token, hex: CITY }),
        );
        if self.face_up {
            events.extend(
                self.defenders
                    .iter()
                    .map(|&token| Event::EnemyTokenRevealed { hex: CITY, token }),
            );
        }
        events
    }

    pub fn state(&self) -> GameState {
        self.events().iter().fold(GameState::default(), apply)
    }
}

/// Cité bleue niveau 5 de l'exemple de la spec : Mages de glace, Mages de feu, Glaceurs.
pub fn blue_defenders() -> Vec<EnemyToken> {
    vec![
        token(Enemy::IceMages, 1),
        token(Enemy::FireMages, 1),
        token(Enemy::Freezers, 1),
    ]
}

/// La plaine N-E de la tuile royaume 7, voisine de la cité blanche.
pub const NEAR_WHITE_CITY: Hex = Hex::new(5, -6);

/// L'état obtenu en appliquant `events` à une partie vide.
pub fn state_of(events: &[Event]) -> GameState {
    events.iter().fold(GameState::default(), apply)
}

/// La cité de `tile` conquise, sans défenseur, `shields` Boucliers de `owner` dessus (Seigneur),
/// la figurine sur `figure`, `hand` en main.
pub fn conquered_city(
    tile: TileId,
    figure: Hex,
    shields: usize,
    owner: PlayerId,
    hand: Vec<CardId>,
) -> Vec<Event> {
    let mut events = City {
        tile,
        figure,
        hand,
        defenders: Vec::new(),
        ..City::default()
    }
    .events();
    events.extend((0..shields).map(|_| Event::CityShieldPlaced {
        player: owner,
        hex: CITY,
        enemy: guardsmen(),
    }));
    events.push(Event::CityConquered {
        hex: CITY,
        leader: owner,
    });
    events
}

/// `amount` points d'influence de plus (Promesse jouée de côté, pour la valeur voulue).
pub fn with_influence(state: &GameState, amount: u8) -> GameState {
    apply(
        state.clone(),
        &Event::CardPlayed {
            player: SOLO,
            card: card(BasicAction::Promise, 1),
            play: CardPlay::Sideways(SidewaysEffect::Influence),
            effect: Effect::Influence(amount),
        },
    )
}

/// L'interaction commencée dans la cité conquise de `tile` (3 Boucliers du joueur), `influence`
/// points de plus ; `extra` : des événements de décor ajoutés avant.
pub fn interacting_in_city(tile: TileId, influence: u8, extra: Vec<Event>) -> GameState {
    let mut events = conquered_city(tile, CITY, 3, SOLO, vec![card(BasicAction::Promise, 1)]);
    events.extend(extra);
    let state = done(&state_of(&events), Action::Interact);
    with_influence(&state, influence)
}

/// La pioche commune `deck` dans cet ordre, ses `dealt` premières cartes dans l'offre `offer`.
pub fn deal(deck: CommonDeck, offer: Offer, cards: &[CardId], dealt: usize) -> Vec<Event> {
    let order: Vec<CommonCard> = cards.iter().map(|&c| CommonCard::Deed(c)).collect();
    let mut events = vec![Event::CommonDeckShuffled {
        deck,
        order: order.clone(),
    }];
    events.extend(
        order
            .into_iter()
            .take(dealt)
            .map(|card| Event::OfferCardDealt { offer, card }),
    );
    events
}
