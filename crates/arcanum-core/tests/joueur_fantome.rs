//! Scénarios de la spec 061 (Joueur fantôme) : mise en place, tactique et ordre du tour, tour du
//! fantôme, fins de manche, nouvelle manche, compétences, fin de partie, vue et journal.

#[path = "premier_choix/mod.rs"]
mod premier_choix;
mod support;

use std::collections::BTreeSet;

use arcanum_core::{
    Action, BasicAction, BasicColor, CardId, CardPlay, ConfigError, ContentError, ContentSources,
    DeedCard, DummyHeroChoice, DummyRule, Event, GameConfig, GameContent, GameRngs, GameState,
    Hero, Offer, PlayerId, RoundOrder, Scenario, ScenarioError, SidewaysEffect, Skill, Tactic,
    TimeOfDay, apply, decide, legal_actions, new_game,
};
use arcanum_core::{
    ActionError, CardDestination, CastTargets, CommonCard, CommonDeck, GameEndReason, ManaColor,
    ManaPayment, Spell, view,
};
use arcanum_core::{GameRunner, PlayerEvent, StepOutcome, filter_event, pending_decision};
use premier_choix::FirstChoice;
use support::arrangement::Setup;

const SOLO: PlayerId = PlayerId::SOLO;

fn reco_config(seed: u64, dummy_hero: DummyHeroChoice) -> GameConfig {
    GameConfig {
        seed,
        scenario: Scenario::first_reconnaissance(),
        hero: Hero::Tovak,
        dummy_hero,
    }
}

/// Une partie jouée directement par `decide` et `apply`, générateurs gardés d'une action à l'autre.
struct Game {
    state: GameState,
    rngs: GameRngs,
    seed: u64,
}

impl Game {
    fn new(config: &GameConfig) -> Self {
        let mut rngs = config.rngs();
        let events = new_game(config, &mut rngs);
        Self {
            state: events.iter().fold(GameState::default(), apply),
            rngs,
            seed: config.seed,
        }
    }

    /// Part d'une mise en place arrangée (issue privée 318) : les générateurs reprennent après elle.
    fn arranged(setup: &Setup, seed: u64) -> Self {
        let (_, positions) = setup.save();
        Self {
            state: setup.events().iter().fold(GameState::default(), apply),
            rngs: GameRngs::resume(seed, &positions),
            seed,
        }
    }

    #[track_caller]
    fn play(&mut self, action: Action) -> Vec<Event> {
        let events = decide(&self.state, SOLO, &action, &mut self.rngs)
            .unwrap_or_else(|error| panic!("{action:?} doit être légale : {error:?}"));
        self.state = events.iter().fold(self.state.clone(), apply);
        events
    }

    /// Épisode posé (ADR 0006) : des événements écrits par le test.
    fn pose(&mut self, events: &[Event]) {
        self.state = events.iter().fold(self.state.clone(), apply);
    }

    fn legal(&self) -> Vec<Action> {
        legal_actions(&self.state, SOLO)
    }

    /// Tour de remplissage (spec 028) : annoncer la fin de la manche si c'est légal ; sinon jouer
    /// la première carte de la main de côté, puis finir le tour sans défausser.
    fn filler_turn(&mut self) -> Vec<Event> {
        if self.legal().contains(&Action::AnnounceEndOfRound) {
            return self.play(Action::AnnounceEndOfRound);
        }
        let mut events = Vec::new();
        let playable = self
            .state
            .hand(SOLO)
            .iter()
            .copied()
            .find(|card| !card.card().is_wound());
        if let Some(card) = playable {
            events.extend(self.play(Action::PlayCard {
                card,
                play: CardPlay::Sideways(SidewaysEffect::Move),
            }));
        }
        events.extend(self.end_turn());
        events
    }

    /// La première fin de tour légale.
    fn end_turn(&mut self) -> Vec<Event> {
        let action = self
            .legal()
            .into_iter()
            .find(|action| matches!(action, Action::EndTurn { .. }))
            .expect("finir le tour est légal");
        self.play(action)
    }

    /// Tours de remplissage jusqu'au choix de la tactique de la manche suivante (ou la fin de la
    /// partie).
    fn finish_round(&mut self) -> Vec<Event> {
        let round = self.state.round();
        let mut events = Vec::new();
        while self.state.round() == round && self.state.game_end().is_none() {
            events.extend(self.filler_turn());
        }
        events
    }
}

fn choose(tactic: Tactic) -> Action {
    Action::ChooseTactic { tactic }
}

fn position(events: &[Event], wanted: impl Fn(&Event) -> bool) -> Option<usize> {
    events.iter().position(wanted)
}

fn is_dummy_turn(event: &Event) -> bool {
    matches!(event, Event::DummyTurnPlayed { .. })
}

fn turn_started(turn: u32) -> impl Fn(&Event) -> bool {
    move |event| matches!(event, Event::TurnStarted { turn: t, .. } if *t == turn)
}

fn dummy_tactic(events: &[Event]) -> Option<Tactic> {
    events.iter().find_map(|event| match event {
        Event::DummyTacticDrawn { tactic } => Some(*tactic),
        _ => None,
    })
}

fn flipped(events: &[Event]) -> Option<Vec<CardId>> {
    events.iter().find_map(|event| match event {
        Event::DummyTurnPlayed { cards } => Some(cards.clone()),
        _ => None,
    })
}

/// Première reconnaissance avec un fantôme Goldyx (cristaux vert, vert, bleu), `cards` en tête
/// de son deck.
fn goldyx_with_deck(cards: &[BasicAction]) -> Game {
    let config = reco_config(1, DummyHeroChoice::Hero(Hero::Goldyx));
    let cards: Vec<DeedCard> = cards.iter().map(|&card| DeedCard::Basic(card)).collect();
    Game::arranged(&Setup::new(config).dummy_deck(&cards), 1)
}

/// Le fantôme réduit à ses `left` cartes du dessus : les autres retournées par un tour posé.
fn dummy_deck_left(game: &mut Game, left: usize) {
    let deck = game.state.dummy().expect("un fantôme").deck().to_vec();
    game.pose(&[Event::DummyTurnPlayed {
        cards: deck[left..].to_vec(),
    }]);
}

fn setup_events(config: &GameConfig) -> Vec<Event> {
    new_game(config, &mut config.rngs())
}

/// Héros, cristaux, pile de compétences et deck d'un `DummySetUp`.
type DummySetUpData = (Hero, Vec<BasicColor>, Vec<Skill>, Vec<CardId>);

fn dummy_set_up(events: &[Event]) -> Option<DummySetUpData> {
    events.iter().find_map(|event| match event {
        Event::DummySetUp {
            hero,
            crystals,
            skills,
            deck,
        } => Some((*hero, crystals.clone(), skills.clone(), deck.clone())),
        _ => None,
    })
}

fn cards_of(deck: &[CardId]) -> Vec<DeedCard> {
    let mut cards: Vec<DeedCard> = deck.iter().map(|card| card.card()).collect();
    cards.sort();
    cards
}

// --- Mise en place ---

// Spec 061 C1 — MKUE p. 15, *At the start of the game* : le fantôme est tiré parmi les trois
// héros que le joueur n'a pas pris.
#[test]
fn dummy_hero_is_one_of_the_three_others() {
    let mut seen = BTreeSet::new();
    for seed in 0..30 {
        let events = setup_events(&reco_config(seed, DummyHeroChoice::Random));
        let (hero, ..) = dummy_set_up(&events).expect("Première reconnaissance a un fantôme");
        assert_ne!(hero, Hero::Tovak, "graine {seed}");
        seen.insert(hero);
    }
    assert_eq!(
        seen,
        BTreeSet::from([Hero::Arythea, Hero::Goldyx, Hero::Norowas])
    );
}

// Spec 061 C1 bis — Q2 : le héros du fantôme choisi dans les Options est le sien, à toute graine.
#[test]
fn chosen_dummy_hero_is_used() {
    for seed in 0..10 {
        let events = setup_events(&reco_config(seed, DummyHeroChoice::Hero(Hero::Goldyx)));
        let (hero, crystals, skills, deck) = dummy_set_up(&events).expect("un fantôme");
        assert_eq!(hero, Hero::Goldyx, "graine {seed}");
        assert_eq!(
            crystals,
            vec![BasicColor::Green, BasicColor::Green, BasicColor::Blue]
        );
        assert!(skills.iter().all(|skill| skill.hero() == Hero::Goldyx));
        assert_eq!(cards_of(&deck), cards_of(&Hero::Goldyx.starting_deck()));
    }
}

// Spec 061 C2 — MKUE p. 15 ; scans des cartes de héros : un cristal par cercle coloré.
#[test]
fn dummy_crystals_are_three_basic_colors() {
    use BasicColor::*;
    let content = GameContent::embedded();
    assert_eq!(content.dummy_crystals(Hero::Arythea), vec![Red, Red, White]);
    assert_eq!(
        content.dummy_crystals(Hero::Goldyx),
        vec![Green, Green, Blue]
    );
    assert_eq!(
        content.dummy_crystals(Hero::Norowas),
        vec![White, White, Green]
    );
    assert_eq!(content.dummy_crystals(Hero::Tovak), vec![Blue, Blue, Red]);
    let two = ContentSources::EMBEDDED
        .heroes
        .replace("Arythea: [Red, Red, White]", "Arythea: [Red, Red]");
    let result = GameContent::parse(ContentSources {
        heroes: &two,
        ..ContentSources::EMBEDDED
    });
    assert!(
        matches!(result, Err(ContentError::InvalidDeck { .. })),
        "{:?}",
        result.err()
    );
}

// Spec 061 C3 — MKUE p. 15 ; spec 038 : le fantôme Goldyx a le deck de départ de Goldyx, ses
// cristaux, ses dix compétences ; le reste de la mise en place est celui de la graine sans fantôme.
#[test]
fn dummy_setup_uses_its_own_generator() {
    let config = reco_config(7, DummyHeroChoice::Hero(Hero::Goldyx));
    let game = Game::new(&config);
    let dummy = game.state.dummy().expect("un fantôme");
    assert_eq!(dummy.hero(), Hero::Goldyx);
    assert_eq!(dummy.deck().len(), 16);
    let cards = cards_of(dummy.deck());
    assert!(cards.contains(&DeedCard::Basic(BasicAction::WillFocus)));
    assert!(cards.contains(&DeedCard::Basic(BasicAction::CrystalJoy)));
    assert!(dummy.discard().is_empty());
    assert_eq!(dummy.crystals(BasicColor::Green), 2);
    assert_eq!(dummy.crystals(BasicColor::Blue), 1);
    assert_eq!(dummy.crystals(BasicColor::Red), 0);
    assert_eq!(dummy.skills().len(), 10);
    assert!(dummy.skills().contains(&Skill::SourceOpening));
    assert!(
        dummy
            .skills()
            .iter()
            .all(|skill| skill.hero() == Hero::Goldyx)
    );
    // Ses exemplaires ne se confondent pas avec ceux du joueur.
    let player_cards: BTreeSet<CardId> = game
        .state
        .deck(SOLO)
        .iter()
        .chain(game.state.hand(SOLO))
        .copied()
        .collect();
    assert!(dummy.deck().iter().all(|card| !player_cards.contains(card)));
    assert!(
        BasicColor::ALL
            .iter()
            .all(|&color| game.state.inventory(SOLO).crystals(color) == 0)
    );
    // Même carte, même Source, même deck, mêmes piles que sans fantôme.
    let without = GameConfig {
        scenario: Scenario::first_reconnaissance().without_dummy(),
        ..config.clone()
    };
    let others = |events: Vec<Event>| -> Vec<Event> {
        events
            .into_iter()
            .filter(|event| !matches!(event, Event::GameStarted { .. } | Event::DummySetUp { .. }))
            .collect()
    };
    assert_eq!(
        others(setup_events(&config)),
        others(setup_events(&without))
    );
}

// Spec 061 C4 — MKUE p. 15, *Note* ; Compendium, *Setup* S1 : le fantôme ne compte pas comme un
// joueur.
#[test]
fn dummy_is_not_counted_as_a_player() {
    let game = Game::new(&reco_config(3, DummyHeroChoice::Random));
    assert!(game.state.dummy().is_some());
    assert_eq!(game.state.source().len(), 3);
    assert_eq!(game.state.offer(Offer::Units).len(), 3);
}

// Spec 061 C5 — MKUE p. 18, *Solo Variant* : pas de fantôme dans le bac à sable, ni avec l'option.
#[test]
fn no_dummy_without_the_option() {
    let sandbox = GameConfig {
        scenario: Scenario::sandbox(),
        ..reco_config(1, DummyHeroChoice::Random)
    };
    assert!(Game::new(&sandbox).state.dummy().is_none());
    assert!(dummy_set_up(&setup_events(&sandbox)).is_none());
    let without = GameConfig {
        scenario: Scenario::first_reconnaissance().without_dummy(),
        ..reco_config(1, DummyHeroChoice::Random)
    };
    assert_eq!(without.scenario.dummy(), DummyRule::None);
    assert_eq!(
        Scenario::first_reconnaissance().dummy(),
        DummyRule::Standard
    );
    assert!(Game::new(&without).state.dummy().is_none());
}

// Spec 061 L9 : un fantôme suppose des tactiques.
#[test]
fn dummy_needs_tactics() {
    assert_eq!(
        Scenario::sandbox().with_dummy(DummyRule::Standard),
        Err(ScenarioError::DummyWithoutTactics)
    );
    assert_eq!(
        Scenario::first_reconnaissance()
            .without_dummy()
            .with_dummy(DummyRule::Standard),
        Ok(Scenario::first_reconnaissance())
    );
}

// Spec 061 L10 — MKUE p. 15 : le fantôme est un des héros que le joueur n'a pas pris.
#[test]
fn dummy_hero_differs_from_player_hero() {
    let same = reco_config(1, DummyHeroChoice::Hero(Hero::Tovak));
    assert_eq!(same.validate(), Err(ConfigError::DummyHeroIsPlayerHero));
    assert_eq!(
        reco_config(1, DummyHeroChoice::Hero(Hero::Goldyx)).validate(),
        Ok(())
    );
    assert_eq!(reco_config(1, DummyHeroChoice::Random).validate(), Ok(()));
}

// --- Tactiques et ordre du tour ---

// Spec 061 C6 — MKUE p. 19, *Special Rules* : le fantôme tire sa tactique au hasard parmi celles
// que le joueur a laissées, juste après sa prise.
#[test]
fn dummy_draws_among_remaining_tactics() {
    let mut seen = BTreeSet::new();
    for seed in 0..30 {
        let mut game = Game::new(&reco_config(seed, DummyHeroChoice::Random));
        let events = game.play(choose(Tactic::Planning));
        let chosen = position(&events, |event| matches!(event, Event::TacticChosen { .. }));
        let drawn = position(&events, |event| {
            matches!(event, Event::DummyTacticDrawn { .. })
        });
        assert!(chosen < drawn, "graine {seed} : {events:?}");
        let tactic = dummy_tactic(&events).expect("une tactique tirée");
        assert_eq!(tactic.time_of_day(), TimeOfDay::Day);
        assert_ne!(tactic, Tactic::Planning);
        assert_eq!(game.state.dummy().and_then(|d| d.tactic()), Some(tactic));
        seen.insert(tactic);
    }
    assert_eq!(seen.len(), 5, "{seen:?}");
}

// Spec 061 C7 — MKUE p. 4, 3.c et 4.a : Lève-tôt (1), le joueur joue en premier ; le fantôme
// joue entre ses tours.
#[test]
fn player_first_with_lower_tactic() {
    let mut game = Game::new(&reco_config(2, DummyHeroChoice::Random));
    let events = game.play(choose(Tactic::EarlyBird));
    assert!(position(&events, is_dummy_turn).is_none(), "{events:?}");
    assert!(position(&events, turn_started(1)).is_some());
    assert_eq!(game.state.round_order(), Some(RoundOrder::PlayerFirst));
    let events = game.filler_turn();
    let dummy = position(&events, is_dummy_turn).expect("le tour du fantôme");
    let next = position(&events, turn_started(2)).expect("le tour 2");
    assert!(dummy < next, "{events:?}");
}

// Spec 061 C8 — MKUE p. 4, 3.b et 3.c : Au bon moment (6), le fantôme joue en premier, dans la
// décision de la prise ; après le choix qui suit Faux départ, s'il y en a un.
#[test]
fn dummy_first_with_lower_tactic() {
    let mut game = Game::new(&reco_config(2, DummyHeroChoice::Random));
    let events = game.play(choose(Tactic::TheRightMoment));
    let kinds: Vec<&str> = events
        .iter()
        .filter_map(|event| match event {
            Event::TacticChosen { .. } => Some("TacticChosen"),
            Event::DummyTacticDrawn { .. } => Some("DummyTacticDrawn"),
            Event::DummyTurnPlayed { .. } => Some("DummyTurnPlayed"),
            Event::TurnStarted { .. } => Some("TurnStarted"),
            _ => None,
        })
        .collect();
    assert_eq!(
        kinds,
        [
            "TacticChosen",
            "DummyTacticDrawn",
            "DummyTurnPlayed",
            "TurnStarted"
        ]
    );
    assert_eq!(game.state.round_order(), Some(RoundOrder::DummyFirst));
    // Faux départ (2) pris, le fantôme tiré sur Lève-tôt (1) : une graine qui le donne.
    let mut game = (0..100)
        .map(|seed| Game::new(&reco_config(seed, DummyHeroChoice::Random)))
        .find(|game| {
            let mut probe = Game {
                state: game.state.clone(),
                rngs: game.rngs.clone(),
                seed: game.seed,
            };
            dummy_tactic(&probe.play(choose(Tactic::Rethink))) == Some(Tactic::EarlyBird)
        })
        .expect("une graine où le fantôme tire Lève-tôt");
    let events = game.play(choose(Tactic::Rethink));
    assert!(position(&events, is_dummy_turn).is_none(), "{events:?}");
    assert!(position(&events, turn_started(1)).is_none());
    let events = game.play(Action::RethinkDiscard { cards: Vec::new() });
    let dummy = position(&events, is_dummy_turn).expect("le tour du fantôme");
    let start = position(&events, turn_started(1)).expect("le tour 1");
    assert!(dummy < start, "{events:?}");
}

// Spec 061 C9 — MKUE p. 19, *Special Rules* ; Compendium, *Scenarios* Q8 : les deux tactiques
// prises sont retirées à la fin de la manche.
#[test]
fn both_tactics_are_removed() {
    let mut game = Game::new(&reco_config(4, DummyHeroChoice::Random));
    let events = game.play(choose(Tactic::Planning));
    let drawn = dummy_tactic(&events).expect("une tactique tirée");
    let events = game.finish_round();
    let removed = events
        .iter()
        .find_map(|event| match event {
            Event::TacticsRemoved { tactics } => Some(tactics.clone()),
            _ => None,
        })
        .expect("les tactiques retirées");
    assert!(removed.contains(&Tactic::Planning) && removed.contains(&drawn));
    game.play(choose(Tactic::FromTheDusk));
    game.finish_round();
    assert_eq!(game.state.round(), 3);
    let legal = game.legal();
    assert!(!legal.contains(&choose(Tactic::Planning)));
    assert!(!legal.contains(&choose(drawn)));
    let tactic = [Tactic::EarlyBird, Tactic::Rethink, Tactic::ManaSteal]
        .into_iter()
        .find(|&tactic| legal.contains(&choose(tactic)) && tactic != drawn)
        .expect("une tactique de Jour");
    let events = game.play(choose(tactic));
    let third = dummy_tactic(&events).expect("une tactique tirée");
    assert!(third != Tactic::Planning && third != drawn && third != tactic);
}

// Spec 061 C10 — carte *The Right Moment* ; spec 057 : le tour de plus suit sans tour du fantôme ;
// le fantôme joue après lui. Idem pour le tour supplémentaire de Distorsion temporelle.
#[test]
fn extra_turn_skips_the_dummy() {
    let mut game = Game::new(&reco_config(5, DummyHeroChoice::Random));
    game.play(choose(Tactic::TheRightMoment));
    game.play(Action::AnnounceRightMoment);
    let events = game.filler_turn();
    assert!(position(&events, turn_started(2)).is_some(), "{events:?}");
    assert!(position(&events, is_dummy_turn).is_none(), "{events:?}");
    let events = game.filler_turn();
    let dummy = position(&events, is_dummy_turn).expect("le tour du fantôme");
    assert!(dummy < position(&events, turn_started(3)).expect("le tour 3"));
    // Distorsion temporelle (posée : son tour supplémentaire attend).
    game.pose(&[Event::CardsReturnedToHand {
        player: SOLO,
        cards: Vec::new(),
    }]);
    let events = game.filler_turn();
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::ExtraTurnStarted { .. })),
        "{events:?}"
    );
    assert!(position(&events, is_dummy_turn).is_none(), "{events:?}");
    let events = game.filler_turn();
    let dummy = position(&events, is_dummy_turn).expect("le tour du fantôme");
    assert!(dummy < position(&events, turn_started(4)).expect("le tour 4"));
}

// --- Le tour du fantôme ---

fn basic(cards: &[CardId]) -> Vec<BasicAction> {
    cards
        .iter()
        .map(|card| match card.card() {
            DeedCard::Basic(action) => action,
            other => panic!("{other:?}"),
        })
        .collect()
}

// Spec 061 C11 — MKUE p. 15, *When it is the Dummy player's turn* : 3 cartes, plus une par
// cristal de la couleur de la dernière ; rien d'autre (L6).
#[test]
fn dummy_flips_three_plus_matching_crystals() {
    use BasicAction::*;
    let mut game = goldyx_with_deck(&[Rage, Swiftness, March, Stamina, Promise]);
    let events = game.play(choose(Tactic::TheRightMoment));
    assert_eq!(
        basic(&flipped(&events).expect("le tour du fantôme")),
        [Rage, Swiftness, March, Stamina, Promise]
    );
    let dummy_events = events
        .iter()
        .filter(|event| format!("{event:?}").starts_with("Dummy"))
        .count();
    assert_eq!(
        dummy_events, 2,
        "tirage de la tactique et tour : {events:?}"
    );
    let dummy = game.state.dummy().expect("un fantôme");
    assert_eq!(dummy.deck().len(), 11);
    assert_eq!(dummy.discard().len(), 5);
    assert_eq!(
        basic(dummy.last_turn()),
        [Rage, Swiftness, March, Stamina, Promise]
    );
}

// Spec 061 C12 — MKUE p. 15 : la 3e carte rouge, aucun cristal rouge : 3 cartes.
#[test]
fn dummy_flips_three_without_matching_crystal() {
    use BasicAction::*;
    let mut game = goldyx_with_deck(&[Swiftness, March, Rage, Stamina]);
    let events = game.play(choose(Tactic::TheRightMoment));
    assert_eq!(
        basic(&flipped(&events).expect("un tour")),
        [Swiftness, March, Rage]
    );
}

// Spec 061 C13 — MKUE p. 15 : la 3e carte bleue, un cristal bleu : 4 cartes.
#[test]
fn dummy_flips_one_per_crystal() {
    use BasicAction::*;
    let mut game = goldyx_with_deck(&[Rage, Swiftness, Stamina, March, Promise]);
    let events = game.play(choose(Tactic::TheRightMoment));
    assert_eq!(
        basic(&flipped(&events).expect("un tour")),
        [Rage, Swiftness, Stamina, March]
    );
}

// Spec 061 C14 — MKUE p. 15, *Note* ; Compendium, *Dummy Player* S3 : il vide son deck sans rien
// annoncer ; l'annonce vient au début de son tour suivant (L1).
#[test]
fn dummy_announces_only_on_the_next_turn() {
    let mut game = Game::new(&reco_config(6, DummyHeroChoice::Random));
    game.play(choose(Tactic::EarlyBird));
    dummy_deck_left(&mut game, 2);
    let events = game.filler_turn();
    assert_eq!(
        flipped(&events).map(|cards| cards.len()),
        Some(2),
        "{events:?}"
    );
    assert!(!events.contains(&Event::DummyAnnouncedEndOfRound));
    assert!(game.state.dummy().is_some_and(|d| d.deck().is_empty()));
    let events = game.filler_turn();
    assert!(
        events.contains(&Event::DummyAnnouncedEndOfRound),
        "{events:?}"
    );
    assert!(position(&events, is_dummy_turn).is_none());
    assert!(game.state.dummy().is_some_and(|d| d.announced()));
}

// Spec 061 C15 — MKUE p. 15, *Note* : 4 cartes au deck, la 3e verte avec 2 cristaux verts : les
// 4, pas plus.
#[test]
fn dummy_flips_what_the_deck_has() {
    use BasicAction::*;
    let mut game = goldyx_with_deck(&[Rage, Swiftness, March, Stamina]);
    dummy_deck_left(&mut game, 4);
    let events = game.play(choose(Tactic::TheRightMoment));
    assert_eq!(
        basic(&flipped(&events).expect("un tour")),
        [Rage, Swiftness, March, Stamina]
    );
    assert!(!events.contains(&Event::DummyAnnouncedEndOfRound));
}

// --- Fin de la manche ---

/// Le joueur tout son deck en main : il pourrait annoncer la fin de la manche (MKUE p. 6, 3.a).
fn player_deck_drawn(game: &mut Game) {
    let drawn: Vec<Event> = game
        .state
        .deck(SOLO)
        .iter()
        .map(|&card| Event::CardDrawn { player: SOLO, card })
        .collect();
    game.pose(&drawn);
}

/// Lève-tôt pris, le deck du fantôme vidé (posé) : à la fin du tour 1, il annonce la fin de la
/// manche ; le tour 2 du joueur est le dernier de la manche.
fn dummy_announced(seed: u64) -> Game {
    let mut game = Game::new(&reco_config(seed, DummyHeroChoice::Random));
    game.play(choose(Tactic::EarlyBird));
    dummy_deck_left(&mut game, 0);
    let events = game.filler_turn();
    assert!(
        events.contains(&Event::DummyAnnouncedEndOfRound),
        "{events:?}"
    );
    game
}

fn is_round_end(event: &Event) -> bool {
    matches!(event, Event::RoundEnded { .. })
}

// Spec 061 C17 — MKUE p. 15 ; p. 6, 3 : le fantôme a annoncé, le joueur joue un dernier tour sans
// pouvoir annoncer ; sa fin, sans pioche, finit la manche.
#[test]
fn dummy_announcement_gives_one_last_turn() {
    let mut game = dummy_announced(8);
    player_deck_drawn(&mut game);
    assert!(!game.legal().contains(&Action::AnnounceEndOfRound));
    assert_eq!(
        decide(
            &game.state,
            SOLO,
            &Action::AnnounceEndOfRound,
            &mut game.rngs.clone()
        ),
        Err(ActionError::NotLegal)
    );
    let events = game.filler_turn();
    let ended =
        position(&events, |event| matches!(event, Event::TurnEnded { .. })).expect("le tour finit");
    let round_end = position(&events, is_round_end).expect("la manche finit");
    assert!(
        !events[ended..round_end]
            .iter()
            .any(|event| matches!(event, Event::CardDrawn { .. })),
        "{events:?}"
    );
    assert_eq!(game.state.round(), 2);
    assert!(
        game.legal()
            .iter()
            .all(|action| matches!(action, Action::ChooseTactic { .. }))
    );
}

// Spec 061 C18 — MKUE p. 11, 6 ; spec 035 : un niveau gagné au dernier tour se fait, avec son
// choix, avant la fin de la manche.
#[test]
fn last_turn_still_levels_up() {
    let mut game = dummy_announced(8);
    let fame = view(&game.state, SOLO).next_level_fame.expect("niveau 2");
    game.pose(&[Event::FameGained { player: SOLO, fame }]);
    let mut events = game.filler_turn();
    assert!(position(&events, is_round_end).is_none(), "{events:?}");
    while game.state.round() == 1 {
        let action = game.legal()[0].clone();
        events.extend(game.play(action));
    }
    let level = position(&events, |event| {
        matches!(event, Event::LevelGained { level: 2, .. })
    })
    .expect("le niveau 2");
    let skill = position(&events, |event| matches!(event, Event::SkillGained { .. }))
        .expect("la compétence choisie");
    let round_end = position(&events, is_round_end).expect("la manche finit");
    assert!(level < skill && skill < round_end, "{events:?}");
    // Aucun autre tour entre le choix et la fin de la manche.
    assert!(
        !events[skill..round_end]
            .iter()
            .any(|event| matches!(event, Event::TurnStarted { .. })),
        "{events:?}"
    );
}

// Spec 061 C19 — MKUE p. 4, 4.f : Méditation jouée au dernier tour, son bonus sert à la pioche de
// la nouvelle manche.
#[test]
fn draw_bonus_carries_to_next_round() {
    let mut game = dummy_announced(8);
    player_deck_drawn(&mut game);
    let meditation = CardId::new(DeedCard::Spell(Spell::Meditation), 1);
    game.pose(&[
        Event::CardDrawn {
            player: SOLO,
            card: meditation,
        },
        Event::ManaTokenGained {
            player: SOLO,
            color: ManaColor::Basic(BasicColor::Green),
        },
    ]);
    game.play(Action::PlayCard {
        card: meditation,
        play: CardPlay::Cast {
            option: 0,
            mana: ManaPayment::Token,
            black: None,
            extra_mana: None,
            enemy: None,
            from_offer: false,
            targets: CastTargets::default(),
        },
    });
    if let Some(place) = game
        .legal()
        .into_iter()
        .find(|action| matches!(action, Action::PlaceOnDeck { .. }))
    {
        game.play(place);
    }
    let events = game.end_turn();
    let round_end = position(&events, is_round_end).expect("la manche finit");
    let bonus = position(&events, |event| {
        matches!(event, Event::DrawBonusUsed { .. })
    })
    .expect("le bonus sert à la nouvelle manche");
    assert!(round_end < bonus, "{events:?}");
    let drawn = events[round_end..]
        .iter()
        .filter(|event| matches!(event, Event::CardDrawn { .. }))
        .count();
    assert_eq!(drawn, 7, "{events:?}");
}

// Spec 061 C20 — Compendium, *Time Bending* S2 : le tour supplémentaire se joue avant la fin de la
// manche (L3).
#[test]
fn time_bending_on_last_turn_of_round() {
    let mut game = dummy_announced(8);
    game.pose(&[Event::CardsReturnedToHand {
        player: SOLO,
        cards: Vec::new(),
    }]);
    let events = game.filler_turn();
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::ExtraTurnStarted { .. })),
        "{events:?}"
    );
    assert!(position(&events, is_round_end).is_none());
    let events = game.filler_turn();
    assert!(position(&events, is_round_end).is_some(), "{events:?}");
}

// Spec 061 C21 — MKUE p. 6, 3 : le joueur annonce, le fantôme joue un dernier tour, à vide si son
// deck l'est, sans annoncer en plus (L2).
#[test]
fn player_announcement_gives_dummy_one_last_turn() {
    let mut game = Game::new(&reco_config(9, DummyHeroChoice::Random));
    game.play(choose(Tactic::EarlyBird));
    player_deck_drawn(&mut game);
    let events = game.play(Action::AnnounceEndOfRound);
    let dummy = position(&events, is_dummy_turn).expect("son dernier tour");
    assert!(dummy < position(&events, is_round_end).expect("la manche finit"));
    assert!(flipped(&events).is_some_and(|cards| !cards.is_empty()));

    let mut game = Game::new(&reco_config(9, DummyHeroChoice::Random));
    game.play(choose(Tactic::EarlyBird));
    dummy_deck_left(&mut game, 0);
    player_deck_drawn(&mut game);
    let events = game.play(Action::AnnounceEndOfRound);
    assert_eq!(flipped(&events), Some(Vec::new()), "{events:?}");
    assert!(!events.contains(&Event::DummyAnnouncedEndOfRound));
    assert!(position(&events, is_round_end).is_some());
}

// Spec 061 L5 — MKUE p. 6, 3.d et 4 : le fantôme a annoncé, plus rien en main ni dans le deck au
// début du dernier tour : le tour est passé, la manche finit sans décision.
#[test]
fn forfeit_when_nothing_left_after_dummy_announcement() {
    let mut game = Game::new(&reco_config(10, DummyHeroChoice::Random));
    game.play(choose(Tactic::EarlyBird));
    dummy_deck_left(&mut game, 0);
    player_deck_drawn(&mut game);
    let discarded: Vec<Event> = game.state.hand(SOLO)[1..]
        .iter()
        .map(|&card| Event::CardDiscarded { player: SOLO, card })
        .collect();
    game.pose(&discarded);
    let last = game.state.hand(SOLO)[0];
    let mut events = game.play(Action::PlayCard {
        card: last,
        play: CardPlay::Sideways(SidewaysEffect::Move),
    });
    events.extend(game.end_turn());
    let announced = position(&events, |event| *event == Event::DummyAnnouncedEndOfRound)
        .expect("le fantôme annonce");
    let start = position(&events, turn_started(2)).expect("le dernier tour");
    let round_end = position(&events, is_round_end).expect("la manche finit");
    assert!(announced < start && start < round_end, "{events:?}");
    assert!(
        !events[start..round_end]
            .iter()
            .any(|event| matches!(event, Event::ManaTokenGained { .. })),
        "{events:?}"
    );
    assert_eq!(game.state.round(), 2);
}

// --- Nouvelle manche ---

fn spell_card(spell: Spell) -> CommonCard {
    CommonCard::Deed(CardId::new(DeedCard::Spell(spell), 1))
}

/// La tactique sans effet du moment, le deck du joueur pioché, puis l'annonce de la fin de la
/// manche : la manche suivante attend sa tactique (ou la partie est finie).
fn announce_round(game: &mut Game) -> Vec<Event> {
    // Les tactiques sans choix qui suit la prise : il en reste toujours une (deux retirées par
    // manche).
    let calm = match game.state.time_of_day() {
        TimeOfDay::Day => [Tactic::EarlyBird, Tactic::Planning, Tactic::TheRightMoment],
        TimeOfDay::Night => [Tactic::FromTheDusk, Tactic::LongNight, Tactic::ManaSearch],
    };
    let legal = game.legal();
    let tactic = calm
        .into_iter()
        .find(|&tactic| legal.contains(&choose(tactic)))
        .expect("une tactique sans choix");
    game.play(choose(tactic));
    player_deck_drawn(game);
    game.play(Action::AnnounceEndOfRound)
}

// Spec 061 C22 — MKUE p. 15, *When preparing a new Round* ; p. 4, 2.d ; FAQ p3 : l'action
// avancée en dernière place va dans le deck du fantôme, pas sous la pioche.
#[test]
fn lowest_advanced_action_joins_dummy_deck() {
    let mut game = Game::new(&reco_config(11, DummyHeroChoice::Random));
    let lowest = *game
        .state
        .offer(Offer::AdvancedActions)
        .last()
        .expect("une offre d'actions avancées");
    let card = lowest.deed().expect("une carte d'action");
    let events = announce_round(&mut game);
    assert!(
        events.contains(&Event::DummyCardAdded { card }),
        "{events:?}"
    );
    assert!(!events.iter().any(|event| matches!(
        event,
        Event::OfferCardReturned {
            offer: Offer::AdvancedActions,
            ..
        }
    )));
    assert!(
        !game
            .state
            .common_deck(CommonDeck::AdvancedActions)
            .contains(&lowest)
    );
    assert!(!game.state.offer(Offer::AdvancedActions).contains(&lowest));
    assert_eq!(game.state.offer(Offer::AdvancedActions).len(), 3);
    assert!(game.state.dummy().is_some_and(|d| d.deck().contains(&card)));
}

// Spec 061 C23 — MKUE p. 15 : le sort en dernière place va sous la pioche ; le fantôme gagne un
// cristal de sa couleur, sans limite.
#[test]
fn dummy_gains_crystal_of_lowest_spell() {
    let config = reco_config(12, DummyHeroChoice::Hero(Hero::Arythea));
    let setup = Setup::new(config).offer(
        CommonDeck::Spells,
        &[
            spell_card(Spell::BurningShield),
            spell_card(Spell::FlameWall),
            spell_card(Spell::Fireball),
        ],
    );
    let mut game = Game::arranged(&setup, 12);
    assert_eq!(
        game.state.offer(Offer::Spells).last(),
        Some(&spell_card(Spell::Fireball))
    );
    let red = |game: &Game| game.state.dummy().map(|d| d.crystals(BasicColor::Red));
    assert_eq!(red(&game), Some(2));
    let events = announce_round(&mut game);
    let returned = position(&events, |event| {
        *event
            == Event::OfferCardReturned {
                offer: Offer::Spells,
                card: spell_card(Spell::Fireball),
            }
    })
    .expect("le sort sous la pioche");
    let crystal = position(&events, |event| {
        *event
            == Event::DummyCrystalGained {
                color: BasicColor::Red,
            }
    })
    .expect("le cristal du fantôme");
    assert!(returned < crystal);
    assert_eq!(red(&game), Some(3));
    announce_round(&mut game);
    assert_eq!(red(&game), Some(4));
}

// Spec 061 C24 — MKUE p. 4, 2.g ; p. 15 : à la nouvelle manche, deck, défausse et action avancée
// ajoutée réunis et mélangés, après le mélange du joueur.
#[test]
fn dummy_reshuffles_everything() {
    let mut game = Game::new(&reco_config(13, DummyHeroChoice::Random));
    game.play(choose(Tactic::EarlyBird));
    game.filler_turn();
    assert!(game.state.dummy().is_some_and(|d| !d.discard().is_empty()));
    player_deck_drawn(&mut game);
    let events = game.play(Action::AnnounceEndOfRound);
    let player = position(&events, |event| matches!(event, Event::DeckShuffled { .. }))
        .expect("le mélange du joueur");
    let dummy_shuffle = position(&events, |event| {
        matches!(event, Event::DummyDeckShuffled { .. })
    })
    .expect("le mélange du fantôme");
    assert!(player < dummy_shuffle, "{events:?}");
    let dummy = game.state.dummy().expect("un fantôme");
    assert_eq!(dummy.deck().len(), 17);
    assert!(dummy.discard().is_empty());
    assert!(dummy.last_turn().is_empty());
}

// Spec 061 C25 : offres vides (posé), ni action avancée ni cristal pour le fantôme.
#[test]
fn empty_offers_add_nothing_to_dummy() {
    let mut game = Game::new(&reco_config(14, DummyHeroChoice::Random));
    let emptied: Vec<Event> = [Offer::AdvancedActions, Offer::Spells]
        .into_iter()
        .flat_map(|offer| {
            game.state
                .offer(offer)
                .iter()
                .map(move |&card| Event::OfferCardGained {
                    player: SOLO,
                    offer,
                    card,
                    to: CardDestination::DiscardPile,
                })
                .collect::<Vec<_>>()
        })
        .collect();
    game.pose(&emptied);
    let events = announce_round(&mut game);
    assert!(position(&events, |event| matches!(event, Event::RoundStarted { .. })).is_some());
    assert!(!events.iter().any(|event| matches!(
        event,
        Event::DummyCardAdded { .. } | Event::DummyCrystalGained { .. }
    )));
    assert_eq!(game.state.dummy().map(|d| d.deck().len()), Some(16));
}

// Spec 061 L7 — spec 023, C15 : la dernière manche finie, rien pour le fantôme.
#[test]
fn last_round_adds_nothing_to_dummy() {
    let mut game = Game::new(&reco_config(15, DummyHeroChoice::Random));
    for _ in 0..3 {
        announce_round(&mut game);
    }
    assert_eq!(game.state.round(), 4);
    let events = announce_round(&mut game);
    assert!(events.contains(&Event::GameEnded {
        reason: GameEndReason::LastRound
    }));
    assert!(!events.iter().any(|event| matches!(
        event,
        Event::DummyCardAdded { .. }
            | Event::DummyCrystalGained { .. }
            | Event::DummyDeckShuffled { .. }
    )));
}

// --- Compétences ---

fn choose_skill(skill: Skill) -> Action {
    Action::ChooseSkill { skill }
}

fn revealed(events: &[Event]) -> Vec<Skill> {
    events
        .iter()
        .find_map(|event| match event {
            Event::SkillsRevealed { skills, .. } => Some(skills.clone()),
            _ => None,
        })
        .unwrap_or_default()
}

fn skill_choices(game: &Game) -> Vec<Skill> {
    game.legal()
        .into_iter()
        .filter_map(|action| match action {
            Action::ChooseSkill { skill } => Some(skill),
            _ => None,
        })
        .collect()
}

/// La gloire portée à `fame` (posée), puis un tour de remplissage : le changement de niveau.
fn level_up_to(game: &mut Game, fame: u32) -> Vec<Event> {
    let gained = fame - game.state.fame(SOLO);
    game.pose(&[Event::FameGained {
        player: SOLO,
        fame: gained,
    }]);
    game.filler_turn()
}

/// Première reconnaissance, fantôme Goldyx, Lève-tôt pris.
fn goldyx_dummy_game() -> Game {
    let mut game = Game::new(&reco_config(16, DummyHeroChoice::Hero(Hero::Goldyx)));
    game.play(choose(Tactic::EarlyBird));
    game
}

fn dummy_skill_top(game: &Game) -> Skill {
    game.state.dummy().expect("un fantôme").skills()[0]
}

// Spec 061 C26 — MKUE p. 15, *Skills in a Solo Game* : le joueur gagne une compétence, celle du
// dessus de la pile du fantôme va dans l'offre commune, après son choix.
#[test]
fn dummy_skill_revealed_after_skill_gain() {
    let mut game = goldyx_dummy_game();
    let top = dummy_skill_top(&game);
    let pair = revealed(&level_up_to(&mut game, 3));
    assert_eq!(pair.len(), 2);
    let events = game.play(choose_skill(pair[0]));
    let other = position(&events, |event| {
        *event == Event::SkillToCommonOffer { skill: pair[1] }
    })
    .expect("l'autre dans l'offre commune");
    let dummy = position(&events, |event| {
        *event == Event::DummySkillRevealed { skill: top }
    })
    .expect("la compétence du fantôme");
    assert!(other < dummy, "{events:?}");
    assert_eq!(game.state.dummy().map(|d| d.skills().len()), Some(9));
    assert!(game.state.common_skills().contains(&top));
}

// Spec 061 C27 — MKUE p. 15 ; spec 051 : au niveau 4, la compétence du fantôme révélée au niveau 2
// se prend (seconde option) ; une autre du fantôme est révélée ensuite.
#[test]
fn player_can_take_dummy_skill() {
    let mut game = goldyx_dummy_game();
    let top = dummy_skill_top(&game);
    let pair = revealed(&level_up_to(&mut game, 3));
    let mut events = game.play(choose_skill(pair[0]));
    while position(&events, |event| matches!(event, Event::TurnStarted { .. })).is_none() {
        let action = game.legal()[0].clone();
        events = game.play(action);
    }
    let second = dummy_skill_top(&game);
    let pair4 = revealed(&level_up_to(&mut game, 15));
    assert_eq!(pair4.len(), 2);
    let choices = skill_choices(&game);
    assert!(choices.contains(&top), "{choices:?}");
    assert!(!choices.contains(&pair[1]), "{choices:?}");
    let events = game.play(choose_skill(top));
    assert!(events.contains(&Event::SkillTakenFromCommonOffer {
        player: SOLO,
        skill: top,
    }));
    assert!(events.contains(&Event::DummySkillRevealed { skill: second }));
    let lowest = *game
        .state
        .offer(Offer::AdvancedActions)
        .last()
        .expect("une offre d'actions avancées");
    assert_eq!(
        game.legal(),
        vec![Action::GainAdvancedAction {
            card: lowest.deed().expect("une carte d'action"),
        }]
    );
}

// Spec 061 C28 — MKUE p. 15 (*the next time*) : la compétence du fantôme révélée au niveau 2 n'est
// pas proposée dans ce même choix.
#[test]
fn dummy_skill_not_available_same_level() {
    let mut game = goldyx_dummy_game();
    let top = dummy_skill_top(&game);
    let pair = revealed(&level_up_to(&mut game, 3));
    let mut choices = skill_choices(&game);
    choices.sort();
    let mut expected = pair.clone();
    expected.sort();
    assert_eq!(choices, expected);
    assert!(!choices.contains(&top));
}

// --- Fin de la partie ---

fn city_revealed() -> Event {
    Event::LastTurnAnnounced { player: SOLO }
}

// Spec 061 C29 — MKUE p. 18 et p. 19, *Scenario End* (Q3) : une cité révélée (posé), le fantôme
// ne joue plus ; le tour suivant du joueur est le dernier de la partie.
#[test]
fn dummy_does_not_play_after_city_revealed() {
    let mut game = Game::new(&reco_config(17, DummyHeroChoice::Random));
    game.play(choose(Tactic::EarlyBird));
    game.pose(&[city_revealed()]);
    let events = game.filler_turn();
    assert!(position(&events, is_dummy_turn).is_none(), "{events:?}");
    assert!(position(&events, turn_started(2)).is_some());
    let events = game.filler_turn();
    assert!(position(&events, is_dummy_turn).is_none(), "{events:?}");
    assert!(events.contains(&Event::GameEnded {
        reason: GameEndReason::CityRevealed
    }));
}

// Spec 061 C30 — MKUE p. 18, *Scenario End* (Q3) : le fantôme a annoncé, la cité est révélée
// (posé) pendant le dernier tour de la manche : la manche finit, la partie aussi.
#[test]
fn round_ending_during_last_turns_ends_game() {
    let mut game = dummy_announced(18);
    game.pose(&[city_revealed()]);
    let events = game.filler_turn();
    let round_end = position(&events, is_round_end).expect("la manche finit");
    let game_end = position(&events, |event| {
        *event
            == Event::GameEnded {
                reason: GameEndReason::CityRevealed,
            }
    })
    .expect("la partie finit");
    assert!(round_end < game_end, "{events:?}");
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::RoundStarted { .. }))
    );
}

// Spec 061 C16 — ADR 0002 : le fantôme ne décide jamais ; le joueur « premier choix » joue
// Première reconnaissance avec fantôme jusqu'à la fin, sans blocage.
#[test]
fn dummy_never_has_a_decision() {
    for seed in 0..20 {
        let config = reco_config(seed, DummyHeroChoice::Random);
        let mut runner = GameRunner::new(config, vec![Box::new(FirstChoice::default())]);
        let mut finished = false;
        for _ in 0..20_000 {
            if let Some(decision) = pending_decision(runner.state()) {
                assert_eq!(decision.player, SOLO);
            }
            if runner.step().expect("la partie avance") == StepOutcome::Finished {
                finished = true;
                break;
            }
        }
        assert!(finished, "graine {seed} : partie bloquée");
        assert!(runner.state().game_end().is_some(), "graine {seed}");
        assert!(runner.state().dummy().is_some());
    }
}

// --- Vue, journal, retour en arrière ---

// Spec 061 C31 — MKUE p. 15, *Note 1* ; Compendium, *Miscellaneous* S2 : la vue montre le
// fantôme, sans l'ordre de son deck ni de sa pile ; et qui joue en premier.
#[test]
fn view_shows_dummy_without_deck_order() {
    use BasicAction::*;
    let mut game = goldyx_with_deck(&[Rage, Swiftness, March, Stamina, Promise]);
    game.play(choose(Tactic::TheRightMoment));
    let seen = view(&game.state, SOLO);
    let dummy = game.state.dummy().expect("un fantôme");
    let shown = seen.dummy.clone().expect("le fantôme dans la vue");
    assert_eq!(shown.hero, Hero::Goldyx);
    assert_eq!(shown.tactic, dummy.tactic());
    assert!(shown.tactic.is_some());
    assert_eq!(shown.deck_size, 11);
    assert_eq!(shown.discard, dummy.discard());
    assert_eq!(shown.crystals(BasicColor::Green), 2);
    assert_eq!(shown.crystals(BasicColor::Blue), 1);
    assert_eq!(
        basic(&shown.last_turn),
        [Rage, Swiftness, March, Stamina, Promise]
    );
    assert!(!shown.announced);
    assert_eq!(shown.skill_count, 10);
    assert_eq!(seen.round_order, Some(RoundOrder::DummyFirst));
    // Rien de la vue ne nomme une carte du deck caché.
    let text = format!("{shown:?}");
    for card in dummy.deck() {
        assert!(!text.contains(&format!("{card:?}")), "{card:?} dans {text}");
    }
}

// Spec 061 C32 — spec 024 ; ADR 0001 : le journal filtré ne donne l'ordre ni du deck ni de la pile
// du fantôme.
#[test]
fn log_hides_dummy_deck_order() {
    let events = setup_events(&reco_config(19, DummyHeroChoice::Random));
    let (hero, crystals, mut skills, mut deck) = dummy_set_up(&events).expect("un fantôme");
    let set_up = events
        .iter()
        .find(|event| matches!(event, Event::DummySetUp { .. }))
        .expect("un fantôme");
    skills.sort();
    deck.sort();
    assert_eq!(
        filter_event(set_up, SOLO),
        PlayerEvent::DummySetUp {
            hero,
            crystals,
            skills,
            deck,
        }
    );
    let shuffled = Event::DummyDeckShuffled {
        order: Hero::Goldyx.dummy_deck(),
    };
    assert_eq!(
        filter_event(&shuffled, SOLO),
        PlayerEvent::DummyDeckShuffled { count: 16 }
    );
}

// Spec 061 C33 — spec 024 : le tour du fantôme et le tirage de sa tactique sont des points de
// non-retour.
#[test]
fn dummy_turn_blocks_undo() {
    assert!(Event::DummyTurnPlayed { cards: Vec::new() }.blocks_undo());
    assert!(
        Event::DummyTacticDrawn {
            tactic: Tactic::EarlyBird
        }
        .blocks_undo()
    );
}

// Spec 061 C36 — ADR 0001 : rejouer le journal d'une partie avec fantôme redonne le même état.
#[test]
fn replaying_log_rebuilds_dummy() {
    let config = reco_config(20, DummyHeroChoice::Random);
    let mut runner = GameRunner::new(config, vec![Box::new(FirstChoice::default())]);
    for _ in 0..5_000 {
        if runner.state().round() >= 3 || runner.state().game_end().is_some() {
            break;
        }
        runner.step().expect("la partie avance");
    }
    assert!(runner.state().dummy().is_some_and(|d| d.deck().len() > 16));
    assert_eq!(&runner.log().replay(), runner.state());
}
