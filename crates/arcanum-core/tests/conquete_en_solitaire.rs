//! Scénarios de la spec 064 (Conquête en solitaire) : scénario et mise en place, côte, fin du
//! scénario, décompte et vue.

mod arene;

use arcanum_core::{
    Achievement, ConquestLine, ConquestProgress, ConquestScore, FinalScore, final_score, view,
};
use arcanum_core::{
    Action, CardPlay, CityLevels, CityRule, CommonCard, CommonDeck, CountrysideOrder, DeedCard,
    DummyHeroChoice, DummyRule, Event, FinalScoring, GameConfig, GameRngs, GameState, Hero,
    MapShape, Offer, Scenario, ScenarioEnd, ScenarioError, ScenarioId, SidewaysEffect, Spell,
    SpellSet, Tactic, TacticOrder, TacticRemoval, TacticRule, TileDeckRecipe, TileKind, TimeOfDay,
    apply, decide, legal_actions, new_game,
};
use arcanum_core::{
    BasicAction, BasicColor, GameEndReason, Hex, LastTurn, NextTile, Placement, TileId,
    TilePosition, next_tile, placement,
};
use arene::*;

fn conquest() -> Scenario {
    Scenario::solo_conquest()
}

fn config(seed: u64) -> GameConfig {
    GameConfig {
        seed,
        scenario: conquest(),
        hero: Hero::Arythea,
        dummy_hero: DummyHeroChoice::Random,
    }
}

fn setup(config: &GameConfig) -> Vec<Event> {
    new_game(config, &mut config.rngs())
}

/// Une partie jouée directement par `decide` et `apply`, générateurs gardés d'une action à l'autre.
struct Game {
    state: GameState,
    rngs: GameRngs,
}

impl Game {
    fn new(config: &GameConfig) -> Self {
        let mut rngs = config.rngs();
        let events = new_game(config, &mut rngs);
        Self {
            state: events.iter().fold(GameState::default(), apply),
            rngs,
        }
    }

    #[track_caller]
    fn play(&mut self, action: Action) -> Vec<Event> {
        let events = decide(&self.state, SOLO, &action, &mut self.rngs)
            .unwrap_or_else(|error| panic!("{action:?} doit être légale : {error:?}"));
        self.state = events.iter().fold(self.state.clone(), apply);
        events
    }

    /// Une décision de remplissage : la première tactique, l'annonce de la fin de la manche, la
    /// fin du tour, une carte de côté, sinon la première action légale.
    fn filler_step(&mut self) -> Vec<Event> {
        let legal = legal_actions(&self.state, SOLO);
        let wanted: [fn(&Action) -> bool; 4] = [
            |action| matches!(action, Action::ChooseTactic { .. }),
            |action| matches!(action, Action::AnnounceEndOfRound),
            |action| matches!(action, Action::EndTurn { .. }),
            |action| {
                matches!(
                    action,
                    Action::PlayCard {
                        play: CardPlay::Sideways(SidewaysEffect::Move),
                        ..
                    }
                )
            },
        ];
        let action = wanted
            .iter()
            .find_map(|wanted| legal.iter().find(|action| wanted(action)))
            .or(legal.first())
            .cloned()
            .expect("une action légale");
        self.play(action)
    }
}

// --- Scénario et mise en place ---

// C1 — MKUE p. 19, *Map*, *Special Rules*, *Scenario End*, *Scoring*
#[test]
fn solo_conquest_matches_livret() {
    let scenario = conquest();
    assert_eq!(scenario.id(), ScenarioId::SoloConquest);
    assert_eq!(Scenario::of(ScenarioId::SoloConquest), scenario);
    assert_eq!(scenario.shape(), MapShape::StandardWedge);
    assert_eq!(
        scenario.tile_deck(),
        TileDeckRecipe {
            countryside: 7,
            countryside_order: CountrysideOrder::Random,
            core_city: 2,
            core_non_city: 2,
        }
    );
    assert_eq!(scenario.rounds(), 6);
    assert_eq!(
        scenario.cities(),
        CityRule::Levels(CityLevels::new(&[5, 8]).expect("niveaux valides"))
    );
    assert_eq!(scenario.dummy(), DummyRule::Standard);
    assert_eq!(
        scenario.tactics(),
        TacticRule::Chosen {
            order: TacticOrder::PlayerFirst,
            removal: TacticRemoval::AllTaken,
        }
    );
    assert_eq!(scenario.fame_per_tile(), 0);
    assert_eq!(scenario.end(), ScenarioEnd::AllCitiesConquered);
    assert_eq!(scenario.scoring(), FinalScoring::SoloConquest);
    assert_eq!(scenario.spells(), SpellSet::WithoutCompetitive);
    // Q3 : le fantôme est obligatoire ; seule Première reconnaissance le rend facultatif.
    assert!(!scenario.dummy_optional());
    assert!(Scenario::first_reconnaissance().dummy_optional());
    assert!(!Scenario::sandbox().dummy_optional());
}

fn tile_deck(events: &[Event]) -> Vec<arcanum_core::TileId> {
    events
        .iter()
        .find_map(|event| match event {
            Event::TileDeckFormed { order } => Some(order.clone()),
            _ => None,
        })
        .expect("pioche de tuiles formée")
}

fn revealed_tiles(events: &[Event]) -> usize {
    events
        .iter()
        .filter(|event| matches!(event, Event::TileRevealed { .. }))
        .count()
}

// C2 — MKUE p. 3 ; Compendium, *Setup* Q2, S3
#[test]
fn solo_conquest_shuffles_countryside() {
    let events = setup(&config(3));
    let state = events.iter().fold(GameState::default(), apply);
    // Deux tuiles de campagne révélées à la mise en place : il en reste 5, puis les 4 royaume.
    assert_eq!(revealed_tiles(&events), 2);
    let remaining = state.tile_deck().tiles().to_vec();
    assert_eq!(remaining.len(), 9);
    assert!(
        remaining[..5]
            .iter()
            .all(|tile| tile.kind() == TileKind::Countryside)
    );
    assert!(
        remaining[5..]
            .iter()
            .all(|tile| tile.kind() == TileKind::Core)
    );
    assert_eq!(remaining.iter().filter(|tile| tile.is_city()).count(), 2);
    // L'ordre de la campagne dépend de la graine.
    let other = tile_deck(&setup(&config(4)));
    assert_ne!(tile_deck(&events)[..7], other[..7]);
}

const COMPETITIVE: [Spell; 4] = [
    Spell::ManaMeltdown,
    Spell::ManaClaim,
    Spell::MindRead,
    Spell::EnergyFlow,
];

/// Les sorts de la pioche de la mise en place, offre comprise.
fn setup_spells(events: &[Event]) -> Vec<Spell> {
    events
        .iter()
        .find_map(|event| match event {
            Event::CommonDeckShuffled {
                deck: CommonDeck::Spells,
                order,
            } => Some(order.clone()),
            _ => None,
        })
        .expect("pioche de sorts mélangée")
        .into_iter()
        .filter_map(|card| match card {
            CommonCard::Deed(card) => match card.card() {
                DeedCard::Spell(spell) => Some(spell),
                _ => None,
            },
            CommonCard::Unit(_) => None,
        })
        .collect()
}

// C3 — MKUE p. 19, *Cards and Skills* : les quatre sorts compétitifs retirés.
#[test]
fn solo_conquest_removes_competitive_spells() {
    for seed in 0..5 {
        let spells = setup_spells(&setup(&config(seed)));
        assert_eq!(spells.len(), 16, "graine {seed}");
        assert!(
            !spells.iter().any(|spell| COMPETITIVE.contains(spell)),
            "graine {seed} : {spells:?}"
        );
    }
    // Première reconnaissance les garde.
    let reco = GameConfig {
        scenario: Scenario::first_reconnaissance(),
        ..config(0)
    };
    assert_eq!(setup_spells(&setup(&reco)).len(), 20);
}

// C4 — MKUE p. 3 ; Compendium, *Setup* S1 : le fantôme n'est pas un joueur.
#[test]
fn solo_conquest_setup_counts() {
    let events = setup(&config(5));
    let dice = events.iter().find_map(|event| match event {
        Event::SourceRolled { dice } => Some(dice.len()),
        _ => None,
    });
    assert_eq!(dice, Some(3));
    let units = events
        .iter()
        .filter(|event| {
            matches!(
                event,
                Event::OfferCardDealt {
                    offer: Offer::Units,
                    ..
                }
            )
        })
        .count();
    assert_eq!(units, 3);
    assert!(!conquest().village_unit_at_setup());
    assert!(conquest().elite_units());
    assert_eq!(
        conquest().advanced_actions(),
        arcanum_core::AdvancedActionSet::All
    );
}

// C5 — MKUE p. 19, *Special Rules* ; Compendium, *Scenarios* Q8 : en six manches, chaque tactique
// est prise une fois ; la 3e manche d'un moment se joue avec les deux dernières.
#[test]
fn each_tactic_taken_once_in_six_rounds() {
    let mut game = Game::new(&config(11));
    let mut taken: Vec<(u32, Tactic)> = Vec::new();
    let mut last_choices: Vec<Tactic> = Vec::new();
    for _ in 0..20_000 {
        if game.state.game_end().is_some() {
            break;
        }
        let round = game.state.round();
        if round == 5 && last_choices.is_empty() {
            last_choices = legal_actions(&game.state, SOLO)
                .into_iter()
                .filter_map(|action| match action {
                    Action::ChooseTactic { tactic } => Some(tactic),
                    _ => None,
                })
                .collect();
        }
        for event in game.filler_step() {
            match event {
                Event::TacticChosen { tactic, .. } | Event::DummyTacticDrawn { tactic } => {
                    taken.push((round, tactic));
                }
                _ => {}
            }
        }
    }
    assert_eq!(game.state.round(), 6, "la partie va jusqu'à la 6e manche");
    for time in [TimeOfDay::Day, TimeOfDay::Night] {
        let mut tactics: Vec<Tactic> = taken
            .iter()
            .map(|&(_, tactic)| tactic)
            .filter(|tactic| tactic.time_of_day() == time)
            .collect();
        assert_eq!(tactics.len(), 6, "{time:?} : {taken:?}");
        tactics.sort();
        tactics.dedup();
        assert_eq!(tactics.len(), 6, "{time:?} : chacune une fois");
    }
    let mut fifth: Vec<Tactic> = taken
        .iter()
        .filter(|&&(round, _)| round == 5)
        .map(|&(_, tactic)| tactic)
        .collect();
    fifth.sort();
    last_choices.sort();
    assert_eq!(last_choices.len(), 2);
    assert_eq!(fifth, last_choices);
}

// L7 — une fin par conquête suppose des cités ouvertes et des tuiles royaume de cité.
#[test]
fn conquest_end_needs_cities() {
    let recipe = TileDeckRecipe {
        countryside: 7,
        countryside_order: CountrysideOrder::Random,
        core_city: 2,
        core_non_city: 2,
    };
    assert_eq!(
        Scenario::new(
            ScenarioId::SoloConquest,
            MapShape::StandardWedge,
            recipe,
            6,
            0,
            ScenarioEnd::AllCitiesConquered,
            FinalScoring::SoloConquest,
        ),
        Err(ScenarioError::ConquestWithoutCities)
    );
    // Cités fermées : refusé.
    assert_eq!(
        Scenario::first_reconnaissance().with_end(ScenarioEnd::AllCitiesConquered),
        Err(ScenarioError::ConquestWithoutCities)
    );
    // Pas de tuile royaume de cité : refusé.
    let without_city_tiles = Scenario::new(
        ScenarioId::Sandbox,
        MapShape::Wedge,
        TileDeckRecipe {
            core_city: 0,
            ..recipe
        },
        4,
        0,
        ScenarioEnd::RoundLimit,
        FinalScoring::None,
    )
    .expect("scénario valide")
    .with_cities(CityRule::Levels(CityLevels::new(&[5]).expect("niveau")));
    assert_eq!(
        without_city_tiles.with_end(ScenarioEnd::AllCitiesConquered),
        Err(ScenarioError::ConquestWithoutCities)
    );
    // Le bac à sable a une cité ouverte et une tuile de cité : permis.
    assert!(
        Scenario::sandbox()
            .with_end(ScenarioEnd::AllCitiesConquered)
            .is_ok()
    );
}

// --- Côte ---

fn pos(i: i32, j: i32) -> TilePosition {
    TilePosition::from_lattice(i, j)
}

/// L'arène de Conquête en solitaire, figurine au village, tuiles de campagne posées sur
/// `revealed`, `next` dans la pioche.
fn coast_map(revealed: &[(TileId, TilePosition)], next: &[TileId]) -> GameState {
    let mut events = Sandbox {
        scenario: conquest(),
        figure: Some(VILLAGE),
        hand: vec![card(BasicAction::March, 1), card(BasicAction::March, 2)],
        next: next.to_vec(),
        ..Sandbox::default()
    }
    .events();
    events.extend(
        revealed
            .iter()
            .map(|&(tile, position)| Event::TileRevealed { tile, position }),
    );
    state_of(&events)
}

// C6 — MKUE p. 14, *Wedge Map* : *Core (brown) tiles cannot be added to the coastline*.
#[test]
fn core_tile_not_on_coastline() {
    // (1, 1) posée : (0, 2) touche (0, 1) et (1, 1), (2, 0) touche (1, 0) et (1, 1).
    let state = coast_map(&[(TileId::Countryside(3), pos(1, 1))], &[TileId::Core(5)]);
    assert_eq!(next_tile(state.map(), state.tile_deck()), NextTile::Core);
    for coast in [pos(0, 2), pos(2, 0)] {
        assert_eq!(
            placement(state.map(), MapShape::StandardWedge, coast, NextTile::Core),
            Placement::CoreOnCoastline,
            "{coast:?}"
        );
    }
    // À l'intérieur, (1, 1) libre et deux voisines : permis, et l'exploration est légale.
    let state = coast_map(&[], &[TileId::Core(5)]);
    assert_eq!(
        placement(
            state.map(),
            MapShape::StandardWedge,
            pos(1, 1),
            NextTile::Core
        ),
        Placement::Allowed
    );
    let state = done(&state, basic(card(BasicAction::March, 1)));
    assert!(legal_actions(&state, SOLO).contains(&Action::Explore {
        position: pos(1, 1)
    }));
}

// C7 — MKUE p. 14 : la règle ne vise que les tuiles royaume.
#[test]
fn countryside_tile_on_coastline() {
    let state = coast_map(
        &[(TileId::Countryside(3), pos(1, 1))],
        &[TileId::Countryside(4)],
    );
    assert_eq!(
        placement(
            state.map(),
            MapShape::StandardWedge,
            pos(0, 2),
            NextTile::Countryside
        ),
        Placement::Allowed
    );
}

// C8 — MKUE p. 7, 5.e ; p. 14 : une tuile royaume écartée non plus.
#[test]
fn removed_core_tile_not_on_coastline() {
    // (0, 2) touche (0, 1), (1, 1) et (1, 2) : trois voisines, assez pour une tuile écartée.
    let state = coast_map(
        &[
            (TileId::Countryside(3), pos(1, 1)),
            (TileId::Countryside(4), pos(1, 2)),
        ],
        &[],
    );
    assert_eq!(
        placement(
            state.map(),
            MapShape::StandardWedge,
            pos(0, 2),
            NextTile::RemovedCore
        ),
        Placement::CoreOnCoastline
    );
    assert_eq!(
        placement(
            state.map(),
            MapShape::StandardWedge,
            pos(0, 2),
            NextTile::RemovedCountryside
        ),
        Placement::Allowed
    );
}

// C9 — MKUE p. 14 (*similar to the one in the starting scenario, with one extra rule*) : la
// forme en coin de Première reconnaissance garde la tuile de cité sur la côte.
#[test]
fn first_reconnaissance_allows_core_on_coastline() {
    let state = coast_map(&[(TileId::Countryside(3), pos(1, 1))], &[TileId::Core(5)]);
    assert_eq!(Scenario::first_reconnaissance().shape(), MapShape::Wedge);
    assert_eq!(
        placement(state.map(), MapShape::Wedge, pos(0, 2), NextTile::Core),
        Placement::Allowed
    );
}

// --- Fin du scénario ---

/// Une case hors du plateau de jeu pour l'autre cité, posée par événements : seul son état compte.
const OTHER_CITY: Hex = Hex::new(30, -30);

fn other_city_placed() -> Event {
    Event::CityPlaced {
        hex: OTHER_CITY,
        color: BasicColor::Green,
        level: 5,
    }
}

fn other_city_conquered() -> Event {
    Event::CityConquered {
        hex: OTHER_CITY,
        leader: SOLO,
    }
}

/// L'assaut de la cité bleue (Conquête en solitaire) depuis sa voisine, combat engagé ; `extra` :
/// des événements posés avant (l'autre cité).
fn conquest_assault(extra: &[Event]) -> GameState {
    let decor = City {
        scenario: conquest(),
        figure: NEAR_CITY,
        face_up: true,
        hand: vec![card(BasicAction::March, 1), card(BasicAction::Stamina, 1)],
        ..City::default()
    };
    let mut events = decor.events();
    events.extend_from_slice(extra);
    let state = done(&state_of(&events), basic(card(BasicAction::March, 1)));
    let state = done(&state, Action::MoveHero { to: CITY });
    done(&state, Action::StartCombat)
}

/// Les défenseurs de la cité bleue vaincus en phase d'attaque.
fn all_defeated(state: &GameState) -> GameState {
    blue_defenders()
        .into_iter()
        .fold(skip_to_attack_phase(state), |state, token| {
            apply(
                state,
                &Event::EnemiesDefeated {
                    player: SOLO,
                    enemies: vec![token],
                    fame: u32::from(token.enemy().definition().fame),
                },
            )
        })
}

fn announces_last_turn(events: &[Event]) -> bool {
    events
        .iter()
        .any(|event| matches!(event, Event::LastTurnAnnounced { .. }))
}

// C10 — MKUE p. 19, *Scenario End* ; Compendium, *Final Scoring* S3 : « toutes » les cités du
// scénario, révélées ou non.
#[test]
fn one_city_conquered_is_not_the_end() {
    // La seconde cité jamais révélée.
    let state = all_defeated(&conquest_assault(&[]));
    let events = events_of(&state, &Action::EndCombat);
    assert!(events.contains(&Event::CityConquered {
        hex: CITY,
        leader: SOLO
    }));
    assert!(!announces_last_turn(&events), "{events:?}");
    // La seconde révélée, pas conquise.
    let state = all_defeated(&conquest_assault(&[other_city_placed()]));
    let events = events_of(&state, &Action::EndCombat);
    assert!(!announces_last_turn(&events), "{events:?}");
}

/// La seconde cité conquise : la fin du combat de l'assaut, appliquée.
fn last_city_conquered() -> (GameState, Vec<Event>) {
    let state = all_defeated(&conquest_assault(&[
        other_city_placed(),
        other_city_conquered(),
    ]));
    let events = events_of(&state, &Action::EndCombat);
    (events.iter().fold(state, apply), events)
}

// C11 — MKUE p. 19, *Scenario End* : *When all cities are conquered, you have one last turn*.
#[test]
fn conquering_last_city_announces_last_turn() {
    let (state, events) = last_city_conquered();
    let conquered = events
        .iter()
        .position(|event| matches!(event, Event::CityConquered { hex, .. } if *hex == CITY))
        .expect("la cité conquise");
    assert_eq!(
        events.get(conquered + 1),
        Some(&Event::LastTurnAnnounced { player: SOLO }),
        "{events:?}"
    );
    assert_eq!(state.last_turn(), LastTurn::Next);
    assert_eq!(state.game_end(), None);
    // Le joueur finit son tour normalement.
    assert!(
        legal_actions(&state, SOLO)
            .iter()
            .any(|action| matches!(action, Action::EndTurn { .. }))
    );
    // Une cité sans défenseur, conquise en y entrant, annonce aussi le dernier tour.
    let decor = City {
        scenario: conquest(),
        figure: NEAR_CITY,
        defenders: Vec::new(),
        hand: vec![card(BasicAction::March, 1)],
        ..City::default()
    };
    let mut events = decor.events();
    events.extend([other_city_placed(), other_city_conquered()]);
    let state = done(&state_of(&events), basic(card(BasicAction::March, 1)));
    let events = events_of(&state, &Action::MoveHero { to: CITY });
    let conquered = events
        .iter()
        .position(|event| matches!(event, Event::CityConquered { .. }))
        .expect("la cité conquise");
    assert_eq!(
        events.get(conquered + 1),
        Some(&Event::LastTurnAnnounced { player: SOLO }),
        "{events:?}"
    );
}

impl Game {
    fn choose_first_tactic(&mut self) {
        let action = legal_actions(&self.state, SOLO)
            .into_iter()
            .find(|action| matches!(action, Action::ChooseTactic { .. }))
            .expect("choix de la tactique");
        self.play(action);
    }

    fn pose(&mut self, events: &[Event]) {
        self.state = events.iter().fold(self.state.clone(), apply);
    }

    /// Tour de remplissage : annoncer la fin de la manche si c'est légal ; sinon jouer la première
    /// carte de la main de côté, puis finir le tour.
    fn filler_turn(&mut self) -> Vec<Event> {
        if legal_actions(&self.state, SOLO).contains(&Action::AnnounceEndOfRound) {
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
        let action = legal_actions(&self.state, SOLO)
            .into_iter()
            .find(|action| matches!(action, Action::EndTurn { .. }))
            .expect("finir le tour est légal");
        events.extend(self.play(action));
        events
    }
}

/// Les deux cités conquises et le dernier tour annoncé, posés pendant le tour en cours.
fn both_conquered() -> Vec<Event> {
    vec![
        other_city_placed(),
        other_city_conquered(),
        Event::CityPlaced {
            hex: CITY,
            color: BasicColor::Blue,
            level: 8,
        },
        Event::CityConquered {
            hex: CITY,
            leader: SOLO,
        },
        Event::LastTurnAnnounced { player: SOLO },
    ]
}

const CONQUEST_END: Event = Event::GameEnded {
    reason: GameEndReason::AllCitiesConquered,
};

// C12 — MKUE p. 19 : *you have one last turn (the Dummy player does not)*.
#[test]
fn game_ends_after_last_turn_of_conquest() {
    let mut game = Game::new(&config(21));
    game.choose_first_tactic();
    game.pose(&both_conquered());
    let events = game.filler_turn();
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::TurnStarted { .. })),
        "{events:?}"
    );
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::DummyTurnPlayed { .. })),
        "{events:?}"
    );
    assert_eq!(game.state.last_turn(), LastTurn::Current);
    let events = game.filler_turn();
    assert_eq!(events.last(), Some(&CONQUEST_END), "{events:?}");
    assert_eq!(
        game.state.game_end(),
        Some(GameEndReason::AllCitiesConquered)
    );
    assert!(legal_actions(&game.state, SOLO).is_empty());
}

// C13 — Compendium, *Scenarios* O30 : le fantôme avait annoncé ; la manche finit pendant le
// dernier tour, la partie avec.
#[test]
fn round_end_after_conquest_ends_game() {
    let mut game = Game::new(&config(22));
    game.choose_first_tactic();
    // Le fantôme réduit à rien : son tour suivant annonce la fin de la manche.
    let deck = game.state.dummy().expect("un fantôme").deck().to_vec();
    game.pose(&[Event::DummyTurnPlayed { cards: deck }]);
    let events = game.filler_turn();
    assert!(
        events.contains(&Event::DummyAnnouncedEndOfRound),
        "{events:?}"
    );
    game.pose(&both_conquered());
    let events = game.filler_turn();
    let round_end = events
        .iter()
        .position(|event| matches!(event, Event::RoundEnded { .. }))
        .expect("la manche finit");
    let game_end = events
        .iter()
        .position(|event| *event == CONQUEST_END)
        .expect("la partie finit");
    assert!(round_end < game_end, "{events:?}");
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::RoundStarted { .. }))
    );
}

// C14 — Compendium, *Scenarios* O30 ; spec 063, C10 : dernier tour deck vide, l'annonce finit
// la manche et la partie.
#[test]
fn announcing_on_last_turn_of_conquest_ends_game() {
    let hand = [card(BasicAction::March, 1)];
    let mut events = Sandbox {
        scenario: conquest(),
        figure: Some(VILLAGE),
        ..Sandbox::default()
    }
    .events();
    events.extend(both_conquered());
    events.extend([
        Event::TurnStarted {
            player: SOLO,
            turn: 2,
        },
        Event::DeckShuffled {
            player: SOLO,
            order: hand.to_vec(),
        },
        Event::CardDrawn {
            player: SOLO,
            card: hand[0],
        },
    ]);
    let state = state_of(&events);
    assert_eq!(state.last_turn(), LastTurn::Current);
    let events = events_of(&state, &Action::AnnounceEndOfRound);
    assert_eq!(events, [Event::RoundEnded { round: 1 }, CONQUEST_END]);
}

// C15 — MKUE p. 4, 5.a : sans la seconde cité, la partie s'arrête à la fin de la 6e manche.
#[test]
fn solo_conquest_ends_after_sixth_round() {
    let mut events = Sandbox {
        scenario: conquest(),
        figure: Some(VILLAGE),
        time: TimeOfDay::Night,
        ..Sandbox::default()
    }
    .events();
    events.extend([
        other_city_placed(),
        other_city_conquered(),
        Event::RoundStarted {
            round: 6,
            time_of_day: TimeOfDay::Night,
        },
        Event::TacticChosen {
            player: SOLO,
            tactic: no_effect_tactics(TimeOfDay::Night)[0],
        },
        Event::DeckShuffled {
            player: SOLO,
            order: Vec::new(),
        },
    ]);
    let state = state_of(&events);
    let events = events_of(&state, &Action::AnnounceEndOfRound);
    assert_eq!(
        events,
        [
            Event::RoundEnded { round: 6 },
            Event::GameEnded {
                reason: GameEndReason::LastRound
            },
        ]
    );
}

// --- Décompte ---

fn conquest_points(score: &ConquestScore) -> Vec<i64> {
    ConquestLine::ALL
        .into_iter()
        .map(|line| score.points(line))
        .collect()
}

// C16 — MKUE p. 19, *Scoring* ; carte de score : l'exemple de la spec, 151 ligne par ligne.
#[test]
fn solo_conquest_score_example() {
    let conquest = ConquestScore {
        conquered: 2,
        cities: 2,
        unplayed_rounds: 1,
        dummy_deck: 7,
        round_announced: false,
        all_conquered: true,
    };
    let score = FinalScore {
        fame: 62,
        crystals: 4,
        wounds: 3,
        advanced_actions: 3,
        spells: 2,
        // Niveau 2, puis 3 blessée : 1.
        unit_levels: 3,
        artifacts: 1,
        shields: 1,
        adventure_shields: 1,
        conquest: Some(conquest),
    };
    let achievements: Vec<i64> = Achievement::ALL
        .into_iter()
        .map(|achievement| score.points(achievement))
        .collect();
    assert_eq!(achievements, [7, 4, 3, 2, 2, -6]);
    assert_eq!(conquest_points(&conquest), [20, 15, 30, 7, 5]);
    assert_eq!(conquest.total(), 77);
    assert_eq!(score.total(), 151);
}

/// Conquête en solitaire à la manche `round` de jour, `conquered` cités conquises (sur 2), puis
/// la fin posée avec `reason`.
fn ended_at(round: u32, conquered: usize, reason: GameEndReason) -> GameState {
    let mut events = Sandbox {
        scenario: conquest(),
        figure: Some(VILLAGE),
        ..Sandbox::default()
    }
    .events();
    if round > 1 {
        events.extend([
            Event::RoundStarted {
                round,
                time_of_day: TimeOfDay::Day,
            },
            Event::TacticChosen {
                player: SOLO,
                tactic: no_effect_tactics(TimeOfDay::Day)[0],
            },
        ]);
    }
    events.extend(both_conquered().into_iter().take(2 * conquered));
    events.push(Event::GameEnded { reason });
    state_of(&events)
}

fn conquest_of(state: &GameState) -> ConquestScore {
    final_score(state, SOLO)
        .conquest
        .expect("partie « conquête » du décompte")
}

// C17 — MKUE p. 19 : 30 par manche non jouée.
#[test]
fn unplayed_rounds_score_thirty_each() {
    let score = conquest_of(&ended_at(4, 2, GameEndReason::AllCitiesConquered));
    assert_eq!(score.unplayed_rounds, 2);
    assert_eq!(score.points(ConquestLine::UnplayedRounds), 60);
    let score = conquest_of(&ended_at(6, 2, GameEndReason::AllCitiesConquered));
    assert_eq!(score.points(ConquestLine::UnplayedRounds), 0);
}

// C12 et C18 — MKUE p. 19 : sans annonce, +5 ; le deck du fantôme compte, une carte 1 point.
#[test]
fn unannounced_round_and_dummy_deck_score() {
    let mut game = Game::new(&config(21));
    game.choose_first_tactic();
    game.pose(&both_conquered());
    game.filler_turn();
    game.filler_turn();
    let deck = game.state.dummy().expect("un fantôme").deck().len();
    let score = conquest_of(&game.state);
    assert!(score.all_conquered);
    assert!(!score.round_announced);
    assert_eq!(score.dummy_deck, u32::try_from(deck).expect("petit deck"));
    assert_eq!(
        conquest_points(&score),
        [20, 15, 150, i64::try_from(deck).expect("petit deck"), 5]
    );
}

// C18 — MKUE p. 19 : la fin de la manche annoncée pendant la dernière manche, par le fantôme
// (C13) ou par le joueur (C14) : 0 pour cette ligne.
#[test]
fn announced_round_end_scores_nothing() {
    // Par le fantôme : son deck est vide.
    let mut game = Game::new(&config(22));
    game.choose_first_tactic();
    let deck = game.state.dummy().expect("un fantôme").deck().to_vec();
    game.pose(&[Event::DummyTurnPlayed { cards: deck }]);
    game.filler_turn();
    game.pose(&both_conquered());
    game.filler_turn();
    assert_eq!(
        game.state.game_end(),
        Some(GameEndReason::AllCitiesConquered)
    );
    let score = conquest_of(&game.state);
    assert!(score.round_announced);
    assert_eq!(score.points(ConquestLine::RoundNotAnnounced), 0);
    assert_eq!(score.points(ConquestLine::DummyDeck), 0);
    // Par le joueur, au dernier tour deck vide.
    let mut events = Sandbox {
        scenario: conquest(),
        figure: Some(VILLAGE),
        ..Sandbox::default()
    }
    .events();
    events.extend(both_conquered());
    events.extend([
        Event::TurnStarted {
            player: SOLO,
            turn: 2,
        },
        Event::DeckShuffled {
            player: SOLO,
            order: Vec::new(),
        },
    ]);
    let state = done(&state_of(&events), Action::AnnounceEndOfRound);
    let score = conquest_of(&state);
    assert!(score.all_conquered);
    assert!(score.round_announced);
    assert_eq!(score.points(ConquestLine::RoundNotAnnounced), 0);
}

// C19 — MKUE p. 19 (Q2) : mission échouée, seules les cités conquises comptent.
#[test]
fn failed_mission_scores_cities_only() {
    let mut game = Game::new(&config(23));
    game.choose_first_tactic();
    let deck = game.state.dummy().expect("un fantôme").deck().to_vec();
    game.pose(&[
        Event::DummyTurnPlayed {
            cards: deck[5..].to_vec(),
        },
        other_city_placed(),
        other_city_conquered(),
        Event::RoundEnded { round: 6 },
        Event::GameEnded {
            reason: GameEndReason::LastRound,
        },
    ]);
    let score = conquest_of(&game.state);
    assert_eq!(score.dummy_deck, 5);
    assert!(!score.all_conquered);
    assert_eq!(conquest_points(&score), [10, 0, 0, 0, 0]);
    let player_view = view(&game.state, SOLO);
    assert_eq!(
        player_view
            .final_score
            .and_then(|score| score.conquest)
            .map(|conquest| conquest.all_conquered),
        Some(false)
    );
}

// C20 — Compendium, *Final Scoring* S2, *Scenarios* O31 (Q1) : les Boucliers des cités ne
// comptent ni dans le Conquérant ni comme points de Seigneur.
#[test]
fn city_shields_score_nothing_in_solo() {
    let mut events = City {
        scenario: conquest(),
        defenders: Vec::new(),
        ..City::default()
    }
    .events();
    events.extend((0..3).map(|_| Event::CityShieldPlaced {
        player: SOLO,
        hex: CITY,
        enemy: guardsmen(),
    }));
    events.extend([
        Event::CityConquered {
            hex: CITY,
            leader: SOLO,
        },
        Event::GameEnded {
            reason: GameEndReason::LastRound,
        },
    ]);
    let score = final_score(&state_of(&events), SOLO);
    assert_eq!(score.shields, 0);
    let conquest = score.conquest.expect("partie « conquête »");
    assert_eq!(conquest.total(), 10);
    let achievements: i64 = Achievement::ALL
        .into_iter()
        .map(|achievement| score.points(achievement))
        .sum();
    assert_eq!(score.total(), i64::from(score.fame) + achievements + 10);
}

// C21 — spec 063, C14 : Première reconnaissance n'a pas de partie « conquête ».
#[test]
fn first_reconnaissance_score_unchanged() {
    let mut events = Sandbox {
        scenario: Scenario::first_reconnaissance().without_dummy(),
        figure: Some(VILLAGE),
        ..Sandbox::default()
    }
    .events();
    events.push(Event::GameEnded {
        reason: GameEndReason::CityRevealed,
    });
    let state = state_of(&events);
    assert_eq!(final_score(&state, SOLO).conquest, None);
    assert!(view(&state, SOLO).final_score.is_some());
}

// --- Vue ---

// C22 — la progression des cités, seulement pour Conquête en solitaire.
#[test]
fn view_shows_conquest_progress() {
    let events = Sandbox {
        scenario: conquest(),
        figure: Some(VILLAGE),
        ..Sandbox::default()
    }
    .events();
    let state = state_of(&events);
    let progress = |state: &GameState| view(state, SOLO).conquest;
    assert_eq!(
        progress(&state),
        Some(ConquestProgress {
            conquered: 0,
            cities: 2
        })
    );
    let state = [other_city_placed(), other_city_conquered()]
        .iter()
        .fold(state, apply);
    assert_eq!(
        progress(&state),
        Some(ConquestProgress {
            conquered: 1,
            cities: 2
        })
    );
    for scenario in [Scenario::first_reconnaissance(), Scenario::sandbox()] {
        let state = Sandbox {
            scenario,
            figure: Some(VILLAGE),
            ..Sandbox::default()
        }
        .state();
        assert_eq!(progress(&state), None, "{:?}", scenario.id());
    }
}

// C23 — spec 063, C19 : le décompte n'entre dans la vue qu'après la fin de la partie.
#[test]
fn conquest_score_only_after_game_end() {
    let mut game = Game::new(&config(21));
    game.choose_first_tactic();
    game.pose(&both_conquered());
    game.filler_turn();
    assert_eq!(view(&game.state, SOLO).final_score, None);
    game.filler_turn();
    let score = view(&game.state, SOLO).final_score;
    assert_eq!(score, Some(final_score(&game.state, SOLO)));
    assert!(score.and_then(|score| score.conquest).is_some());
}
