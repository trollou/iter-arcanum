//! Scénarios de la spec 024 (revenir en arrière), partie A : arcanum-core.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use arcanum_core::{
    Action, BasicAction, BasicColor, CardId, CardPlay, DeedCard, Event, EventLog, GameConfig,
    GameRunner, GameState, Hero, Hex, ManaColor, ManaPayment, Player, PlayerId, PlayerView,
    ResumeError, RngPositions, Scenario, StepOutcome, TileId, TilePosition, TimeOfDay, UndoError,
    UndoStatus, legal_actions,
};
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;

const SOLO: PlayerId = PlayerId::SOLO;

use BasicAction::*;

fn card(action: BasicAction, copy: u16) -> CardId {
    CardId::new(DeedCard::Basic(action), copy)
}

fn config(seed: u64) -> GameConfig {
    GameConfig {
        seed,
        scenario: Scenario::sandbox(),
        hero: Hero::Arythea,
        dummy_hero: Default::default(),
    }
}

fn pos(i: i32, j: i32) -> TilePosition {
    TilePosition::from_lattice(i, j)
}

fn green() -> ManaColor {
    ManaColor::Basic(BasicColor::Green)
}

/// Joueur piloté par le test : il garde les actions légales reçues et répond l'index déposé.
#[derive(Clone, Default)]
struct Puppet {
    legal: Rc<RefCell<Vec<Action>>>,
    choice: Rc<Cell<Option<usize>>>,
    requests: Rc<Cell<usize>>,
}

impl Player for Puppet {
    fn request(&mut self, _view: &PlayerView, legal_actions: &[Action]) {
        *self.legal.borrow_mut() = legal_actions.to_vec();
        self.requests.set(self.requests.get() + 1);
    }

    fn poll(&mut self) -> Option<usize> {
        self.choice.take()
    }
}

/// Joue la première action légale qui vérifie `wanted`, et la renvoie.
fn play_where(
    runner: &mut GameRunner,
    puppet: &Puppet,
    wanted: impl Fn(&Action) -> bool,
) -> Action {
    assert_eq!(runner.step(), Ok(StepOutcome::Waiting { player: SOLO }));
    let legal = puppet.legal.borrow().clone();
    let index = legal
        .iter()
        .position(wanted)
        .unwrap_or_else(|| panic!("aucune action légale ne convient parmi {legal:?}"));
    puppet.choice.set(Some(index));
    assert_eq!(runner.step(), Ok(StepOutcome::Advanced));
    legal[index].clone()
}

fn play(runner: &mut GameRunner, puppet: &Puppet, action: &Action) {
    play_where(runner, puppet, |candidate| candidate == action);
}

fn basic(card: CardId) -> Action {
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

fn strong(card: CardId, mana: ManaPayment) -> Action {
    Action::PlayCard {
        card,
        play: CardPlay::Strong {
            gain: None,
            extra_mana: None,
            option: 0,
            discard: None,
            mana,
            set_dice: [None, None],
            with: None,
        },
    }
}

/// Mise en place construite par événements (décision 0) : bac à sable, Arythea, `hand` en main ;
/// si `full_deck`, le reste du deck de départ dessous, sinon le deck est vide. Figurine sur
/// `figure`, Source `dice`, puis `extra`.
fn setup(
    figure: Hex,
    hand: &[CardId],
    full_deck: bool,
    dice: &[ManaColor],
    extra: &[Event],
) -> EventLog {
    let mut order = hand.to_vec();
    if full_deck {
        order.extend(
            Hero::Arythea
                .starting_deck()
                .into_iter()
                .filter(|card| !hand.contains(card)),
        );
    }
    let mut tiles: Vec<TileId> = (1..=8).map(TileId::Countryside).collect();
    tiles.extend([TileId::Core(2), TileId::Core(6), TileId::Core(3)]);
    let mut events = vec![
        Event::GameStarted { config: config(0) },
        Event::TilePlaced {
            tile: TileId::StartingA,
            position: TilePosition::ORIGIN,
        },
        Event::TileDeckFormed {
            order: tiles.clone(),
        },
    ];
    events.extend(
        tiles
            .iter()
            .zip([pos(0, 1), pos(1, 0)])
            .map(|(&tile, position)| Event::TileRevealed { tile, position }),
    );
    events.extend([
        Event::HeroFigurePlaced {
            player: SOLO,
            hex: figure,
        },
        Event::DeckShuffled {
            player: SOLO,
            order,
        },
    ]);
    events.extend(
        hand.iter()
            .map(|&card| Event::CardDrawn { player: SOLO, card }),
    );
    events.push(Event::SourceRolled {
        dice: dice.to_vec(),
    });
    events.extend(extra.iter().cloned());

    let mut log = EventLog::default();
    for event in events {
        log.push(0, event);
    }
    log
}

fn resume(log: EventLog) -> (GameRunner, Puppet) {
    let puppet = Puppet::default();
    let runner = GameRunner::resume(log, RngPositions::new(), vec![Box::new(puppet.clone())])
        .expect("journal valide");
    (runner, puppet)
}

fn hand() -> [CardId; 5] {
    [
        card(March, 1),
        card(March, 2),
        card(Stamina, 1),
        card(Improvisation, 1),
        card(Concentration, 1),
    ]
}

/// Bac à sable au bord de la tuile de départ, deck complet, Source verte.
fn sandbox() -> (GameRunner, Puppet) {
    resume(setup(
        Hex::new(4, -3),
        &hand(),
        true,
        &[green(), green(), ManaColor::Basic(BasicColor::White)],
        &[],
    ))
}

/// État, journal et journal des décisions : tout ce que revenir en arrière doit restaurer.
fn snapshot(runner: &GameRunner) -> (GameState, usize, usize) {
    (
        runner.state().clone(),
        runner.log().len(),
        runner.decision_log().len(),
    )
}

// Spec 024 C1 — MKUE p. 5, Reverting 1
#[test]
fn undo_play_card_restores_state() {
    let (mut runner, puppet) = sandbox();
    let before = snapshot(&runner);

    play(&mut runner, &puppet, &basic(card(March, 1)));
    assert_eq!(runner.state().move_points(SOLO), 2);
    assert_eq!(runner.undo_status(), UndoStatus::Possible);

    assert_eq!(runner.undo(), Ok(()));
    assert_eq!(snapshot(&runner), before);
    assert!(runner.state().hand(SOLO).contains(&card(March, 1)));
    assert!(runner.state().play_area(SOLO).is_empty());
    assert_eq!(runner.state().move_points(SOLO), 0);
}

// Spec 024 C2 — MKUE p. 5, Reverting 1
#[test]
fn undo_move_restores_figure_and_points() {
    let (mut runner, puppet) = sandbox();
    play(&mut runner, &puppet, &basic(card(March, 1)));
    let before = snapshot(&runner);

    play_where(&mut runner, &puppet, |action| {
        matches!(action, Action::MoveHero { .. })
    });
    assert_ne!(runner.state().figure(SOLO), Some(Hex::new(4, -3)));

    assert_eq!(runner.undo(), Ok(()));
    assert_eq!(snapshot(&runner), before);
    assert_eq!(runner.state().figure(SOLO), Some(Hex::new(4, -3)));
    assert_eq!(runner.state().move_points(SOLO), 2);
}

// Spec 024 C3 — MKUE p. 5, Reverting 2.a
#[test]
fn undo_restores_mana_payment() {
    let crystal = Event::CrystalGained {
        player: SOLO,
        color: BasicColor::Green,
    };
    let (mut runner, puppet) = resume(setup(
        Hex::new(4, -3),
        &hand(),
        true,
        &[green(), green(), ManaColor::Basic(BasicColor::White)],
        &[crystal],
    ));
    let start = snapshot(&runner);

    // Dé de la Source.
    play(
        &mut runner,
        &puppet,
        &strong(card(March, 1), ManaPayment::Die(0)),
    );
    assert_eq!(runner.state().dice_taken(SOLO), &[0]);
    let after_die = snapshot(&runner);

    // Cristal.
    play(
        &mut runner,
        &puppet,
        &strong(card(March, 2), ManaPayment::Crystal),
    );
    assert_eq!(
        runner.state().inventory(SOLO).crystals(BasicColor::Green),
        0
    );
    let after_crystal = snapshot(&runner);

    // Jeton : Concentration donne un jeton bleu, qui paie Endurance renforcée.
    play(&mut runner, &puppet, &basic(card(Concentration, 1)));
    play(
        &mut runner,
        &puppet,
        &strong(card(Stamina, 1), ManaPayment::Token),
    );
    assert!(runner.state().mana_tokens(SOLO).is_empty());

    assert_eq!(runner.undo(), Ok(()));
    assert_eq!(
        runner.state().mana_tokens(SOLO),
        &[arcanum_core::ManaColor::Basic(BasicColor::Blue)]
    );
    assert_eq!(runner.undo(), Ok(()));
    assert_eq!(snapshot(&runner), after_crystal);
    assert_eq!(runner.undo(), Ok(()));
    assert_eq!(snapshot(&runner), after_die);
    assert_eq!(
        runner.state().inventory(SOLO).crystals(BasicColor::Green),
        1
    );
    assert_eq!(runner.undo(), Ok(()));
    assert_eq!(snapshot(&runner), start);
    assert!(runner.state().dice_taken(SOLO).is_empty());
}

// Spec 024 C3 — MKUE p. 5, Reverting 2 : mettre un dé sur une couleur ne lance aucun dé.
#[test]
fn undo_restores_set_die() {
    let pull = card(ManaPull, 1);
    let (mut runner, puppet) = resume(setup(
        Hex::new(4, -3),
        &[pull, card(March, 1), card(Stamina, 1)],
        true,
        &[
            ManaColor::Basic(BasicColor::White),
            ManaColor::Basic(BasicColor::Red),
            ManaColor::Gold,
        ],
        &[],
    ));
    let before = snapshot(&runner);

    play_where(&mut runner, &puppet, |action| {
        matches!(
            action,
            Action::PlayCard { card, play: CardPlay::Strong { set_dice: [Some(_), _], .. } }
                if *card == pull
        )
    });
    assert!(!runner.state().dice_set(SOLO).is_empty());
    assert_eq!(runner.undo_status(), UndoStatus::Possible);

    assert_eq!(runner.undo(), Ok(()));
    assert_eq!(snapshot(&runner), before);
    assert!(runner.state().dice_set(SOLO).is_empty());
}

// Spec 024 C4
#[test]
fn undo_removes_whole_decision() {
    let (mut runner, puppet) = sandbox();
    play(&mut runner, &puppet, &basic(card(March, 1)));
    let before = snapshot(&runner);

    let improvisation = card(Improvisation, 1);
    play_where(&mut runner, &puppet, |action| {
        matches!(
            action,
            Action::PlayCard { card, play: CardPlay::Basic { discard: Some(_), .. } }
                if *card == improvisation
        )
    });
    assert!(
        runner.log().len() >= before.1 + 2,
        "carte jouée et carte défaussée"
    );

    assert_eq!(runner.undo(), Ok(()));
    assert_eq!(snapshot(&runner), before);
    assert_eq!(runner.state().hand(SOLO).len(), 4);
}

// Spec 024 C5 — MKUE p. 5, Reverting 2
#[test]
fn undo_stops_at_revealed_tile() {
    let (mut runner, puppet) = sandbox();
    play(&mut runner, &puppet, &basic(card(March, 1)));
    play(
        &mut runner,
        &puppet,
        &Action::Explore {
            position: pos(2, 0),
        },
    );

    assert_eq!(runner.undo_status(), UndoStatus::InformationRevealed);
    let explored = snapshot(&runner);
    assert_eq!(runner.undo(), Err(UndoError::InformationRevealed));
    assert_eq!(snapshot(&runner), explored);

    play(&mut runner, &puppet, &basic(card(March, 2)));
    assert_eq!(runner.undo(), Ok(()));
    assert_eq!(snapshot(&runner), explored);
    assert_eq!(runner.undo(), Err(UndoError::InformationRevealed));
}

fn end_turn(action: &Action) -> bool {
    matches!(action, Action::EndTurn { .. })
}

// Spec 024 C6 — MKUE p. 5, Reverting 1
#[test]
fn undo_stops_at_turn_start() {
    // Avec pioche et dé relancé.
    let (mut runner, puppet) = sandbox();
    play(
        &mut runner,
        &puppet,
        &strong(card(March, 1), ManaPayment::Die(0)),
    );
    play_where(&mut runner, &puppet, end_turn);
    assert_eq!(runner.undo_status(), UndoStatus::NothingSinceTurnStart);
    let started = snapshot(&runner);
    assert_eq!(runner.undo(), Err(UndoError::NothingSinceTurnStart));
    assert_eq!(snapshot(&runner), started);

    // Deck vide, aucun dé pris : rien n'est révélé, la fin du tour ferme quand même le tour.
    let (mut runner, puppet) = resume(setup(
        Hex::new(4, -3),
        &hand(),
        false,
        &[green(), green(), green()],
        &[],
    ));
    play(&mut runner, &puppet, &basic(card(March, 1)));
    play_where(&mut runner, &puppet, end_turn);
    assert_eq!(runner.undo(), Err(UndoError::NothingSinceTurnStart));
}

// Spec 024 C7 — MKUE p. 5, Reverting 2 (mélange et Source relancée)
#[test]
fn undo_stops_at_round_start() {
    let (mut runner, puppet) = resume(setup(
        Hex::new(4, -3),
        &hand(),
        false,
        &[green(), green(), green()],
        &[],
    ));
    play(&mut runner, &puppet, &Action::AnnounceEndOfRound);
    assert_eq!(runner.state().round(), 2);
    assert_eq!(runner.undo_status(), UndoStatus::NothingSinceTurnStart);
    assert_eq!(runner.undo(), Err(UndoError::NothingSinceTurnStart));
}

// Spec 024 L1 : début de partie.
#[test]
fn nothing_to_undo_at_game_start() {
    let puppet = Puppet::default();
    let mut runner = GameRunner::new(config(3), vec![Box::new(puppet)]);
    assert_eq!(runner.undo_status(), UndoStatus::NothingSinceTurnStart);
    assert_eq!(runner.undo(), Err(UndoError::NothingSinceTurnStart));
}

// Spec 024 C8 — table des points de non-retour.
#[test]
fn blocks_undo_follows_table() {
    let march = card(March, 1);
    let play = CardPlay::Basic {
        die: None,
        gain: None,
        with: None,
        option: 0,
        discard: None,
        mana: None,
    };
    let cases = [
        (Event::GameStarted { config: config(0) }, true),
        (
            Event::TilePlaced {
                tile: TileId::StartingA,
                position: TilePosition::ORIGIN,
            },
            true,
        ),
        (Event::TileDeckFormed { order: vec![] }, true),
        (
            Event::TileRevealed {
                tile: TileId::Countryside(1),
                position: pos(1, 0),
            },
            true,
        ),
        (
            Event::HeroFigurePlaced {
                player: SOLO,
                hex: Hex::ORIGIN,
            },
            true,
        ),
        (
            Event::DeckShuffled {
                player: SOLO,
                order: vec![],
            },
            true,
        ),
        (
            Event::CardDrawn {
                player: SOLO,
                card: march,
            },
            true,
        ),
        (
            Event::CardPlayed {
                player: SOLO,
                card: march,
                play,
                effect: arcanum_core::Effect::Move(2),
            },
            false,
        ),
        (
            Event::CardDiscarded {
                player: SOLO,
                card: march,
            },
            false,
        ),
        (
            Event::HeroMoved {
                player: SOLO,
                from: Hex::ORIGIN,
                to: Hex::new(1, 0),
                cost: 2,
            },
            false,
        ),
        (
            Event::Explored {
                player: SOLO,
                position: pos(1, 0),
                cost: 2,
            },
            false,
        ),
        (Event::TurnEnded { player: SOLO }, true),
        (
            Event::TurnStarted {
                player: SOLO,
                turn: 2,
            },
            true,
        ),
        (Event::SourceRolled { dice: vec![] }, true),
        (
            Event::CrystalGained {
                player: SOLO,
                color: BasicColor::Green,
            },
            false,
        ),
        (
            Event::DieTaken {
                player: SOLO,
                die: 0,
                color: green(),
            },
            false,
        ),
        (
            Event::CrystalUsed {
                player: SOLO,
                color: BasicColor::Green,
            },
            false,
        ),
        (
            Event::DieReturned {
                player: SOLO,
                die: 0,
                color: green(),
            },
            true,
        ),
        (
            Event::ManaTokenGained {
                player: SOLO,
                color: arcanum_core::ManaColor::Basic(BasicColor::Green),
            },
            false,
        ),
        (
            Event::ManaTokenUsed {
                player: SOLO,
                color: arcanum_core::ManaColor::Basic(BasicColor::Green),
            },
            false,
        ),
        (
            Event::CardReturnedToHand {
                player: SOLO,
                card: march,
            },
            false,
        ),
        (
            Event::DieSet {
                player: SOLO,
                die: 0,
                color: BasicColor::Green,
            },
            false,
        ),
        (Event::RoundEnded { round: 1 }, true),
        (
            Event::RoundStarted {
                round: 2,
                time_of_day: TimeOfDay::Night,
            },
            true,
        ),
    ];
    for (event, expected) in cases {
        assert_eq!(event.blocks_undo(), expected, "{event:?}");
    }
}

/// Joueur au hasard, à graine fixe : il ne choisit que parmi les actions légales.
struct RandomPlayer {
    rng: ChaCha8Rng,
    count: usize,
    legal: Rc<RefCell<Vec<Action>>>,
}

impl Player for RandomPlayer {
    fn request(&mut self, _view: &PlayerView, legal_actions: &[Action]) {
        self.count = legal_actions.len();
        *self.legal.borrow_mut() = legal_actions.to_vec();
    }

    fn poll(&mut self) -> Option<usize> {
        Some(self.rng.random_range(0..self.count))
    }
}

/// Fait jouer un joueur au hasard et appelle `check` après chaque décision.
fn random_walk(seed: u64, mut check: impl FnMut(&GameRunner, RngPositions, &GameState, &[Action])) {
    let legal = Rc::new(RefCell::new(Vec::new()));
    let player = RandomPlayer {
        rng: ChaCha8Rng::seed_from_u64(seed),
        count: 0,
        legal: legal.clone(),
    };
    let mut runner = GameRunner::new(config(seed), vec![Box::new(player)]);
    for _ in 0..400 {
        let position = runner.rng_positions();
        let before = runner.state().clone();
        match runner.step().expect("pas sans erreur") {
            StepOutcome::Finished => return,
            StepOutcome::Waiting { .. } => {}
            StepOutcome::Advanced => check(&runner, position, &before, &legal.borrow()),
        }
    }
}

// Spec 024 C8 — le hasard n'est tiré que dans des décisions de non-retour.
#[test]
fn only_no_return_decisions_use_rng() {
    for seed in 0..10 {
        random_walk(seed, |runner, position_before, _, _| {
            if runner.undo_status() == UndoStatus::Possible {
                assert_eq!(runner.rng_positions(), position_before, "graine {seed}");
            }
        });
    }
}

// Spec 024 C9
#[test]
fn undo_then_redo_is_transparent() {
    let actions = [
        basic(card(March, 1)),
        strong(card(March, 2), ManaPayment::Die(0)),
        basic(card(Stamina, 1)),
    ];

    let (mut straight, puppet) = sandbox();
    for action in &actions {
        play(&mut straight, &puppet, action);
    }

    let (mut reverted, puppet) = sandbox();
    for action in &actions {
        play(&mut reverted, &puppet, action);
    }
    for _ in &actions {
        assert_eq!(reverted.undo(), Ok(()));
    }
    for action in &actions {
        play(&mut reverted, &puppet, action);
    }

    assert_eq!(reverted.state(), straight.state());
    assert_eq!(reverted.log(), straight.log());
    assert_eq!(reverted.decision_log(), straight.decision_log());
    assert_eq!(reverted.rng_positions(), straight.rng_positions());
}

// Spec 024 C10
#[test]
fn undo_asks_player_again() {
    let (mut runner, puppet) = sandbox();
    assert_eq!(runner.step(), Ok(StepOutcome::Waiting { player: SOLO }));
    let first_legal = puppet.legal.borrow().clone();
    play(&mut runner, &puppet, &basic(card(March, 1)));
    let requests = puppet.requests.get();

    assert_eq!(runner.undo(), Ok(()));
    assert_eq!(runner.step(), Ok(StepOutcome::Waiting { player: SOLO }));
    assert_eq!(puppet.requests.get(), requests + 1);
    assert_eq!(*puppet.legal.borrow(), first_legal);
}

// Spec 024 C11
#[test]
fn replay_after_undo_matches_state() {
    let (mut runner, puppet) = sandbox();
    play(&mut runner, &puppet, &basic(card(March, 1)));
    play_where(&mut runner, &puppet, |action| {
        matches!(action, Action::MoveHero { .. })
    });
    assert_eq!(runner.undo(), Ok(()));

    assert_eq!(&runner.log().replay(), runner.state());
}

// Spec 024 C12 — les joueurs ne reçoivent que les actions du jeu : revenir en arrière n'en est pas une.
#[test]
fn undo_is_not_a_legal_action() {
    for seed in 0..5 {
        random_walk(seed, |_, _, before, offered| {
            assert_eq!(offered, legal_actions(before, SOLO).as_slice());
        });
    }
}

// Spec 024 — le journal porte le numéro de la décision qui a produit chaque événement.
#[test]
fn log_entries_carry_decision_numbers() {
    let (mut runner, puppet) = sandbox();
    let setup_len = runner.log().len();
    play(&mut runner, &puppet, &basic(card(March, 1)));
    play(&mut runner, &puppet, &basic(card(March, 2)));

    let entries = runner.log().entries();
    assert!(entries[..setup_len].iter().all(|entry| entry.decision == 0));
    assert_eq!(entries[setup_len].decision, 1);
    assert_eq!(entries.last().map(|entry| entry.decision), Some(2));
}

// Spec 024 L2 : après la dernière manche, rien à reprendre.
#[test]
fn nothing_to_undo_after_game_end() {
    let mut extra = Vec::new();
    for round in 1..=3 {
        extra.push(Event::RoundEnded { round });
        extra.push(Event::RoundStarted {
            round: round + 1,
            time_of_day: if round % 2 == 1 {
                TimeOfDay::Night
            } else {
                TimeOfDay::Day
            },
        });
    }
    let (mut runner, puppet) = resume(setup(
        Hex::new(4, -3),
        &hand(),
        false,
        &[green(), green(), green()],
        &extra,
    ));
    play(&mut runner, &puppet, &Action::AnnounceEndOfRound);
    assert_eq!(runner.step(), Ok(StepOutcome::Finished));
    assert_eq!(runner.undo(), Err(UndoError::NothingSinceTurnStart));
}

// Spec 024 L4
#[test]
fn resume_refuses_invalid_logs() {
    let resume_with = |log: EventLog| {
        GameRunner::resume(log, RngPositions::new(), vec![Box::new(Puppet::default())]).map(|_| ())
    };

    assert_eq!(
        resume_with(EventLog::default()),
        Err(ResumeError::NoGameStarted)
    );

    let mut log = EventLog::default();
    log.push(0, Event::RoundEnded { round: 1 });
    assert_eq!(resume_with(log), Err(ResumeError::NoGameStarted));

    let mut log = setup(Hex::new(4, -3), &hand(), true, &[green()], &[]);
    log.push(2, Event::TurnEnded { player: SOLO });
    log.push(1, Event::TurnEnded { player: SOLO });
    assert!(matches!(
        resume_with(log),
        Err(ResumeError::DecisionsOutOfOrder { .. })
    ));
}

// Spec 024 C13 (arcanum-core) : une partie reprise à la même position du générateur tire le même hasard.
#[test]
fn resumed_runner_draws_same_randomness() {
    let (mut straight, puppet) = sandbox();
    play(
        &mut straight,
        &puppet,
        &strong(card(March, 1), ManaPayment::Die(0)),
    );
    let log = straight.log().clone();
    let position = straight.rng_positions();
    let before_end = straight.state().clone();
    play_where(&mut straight, &puppet, end_turn);

    let puppet = Puppet::default();
    let mut resumed =
        GameRunner::resume(log, position, vec![Box::new(puppet.clone())]).expect("journal valide");
    assert_eq!(resumed.state(), &before_end);
    play_where(&mut resumed, &puppet, end_turn);

    assert_eq!(resumed.state(), straight.state());
    // Le dé relancé à la fin du tour est le même.
    assert_eq!(resumed.state().source(), straight.state().source());
}
