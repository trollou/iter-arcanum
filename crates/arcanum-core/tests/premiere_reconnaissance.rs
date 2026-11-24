//! Scénarios de la spec 063 (Première reconnaissance en solo) : gloire par tuile, dernier tour,
//! fin de partie et décompte final.

mod arene;

use arcanum_core::{
    Achievement, Action, BasicAction, BasicColor, CardId, CountrysideOrder, Event, FinalScore,
    FinalScoring, GameConfig, GameEndReason, GameState, Hero, LastTurn, MapShape, Scenario,
    ScenarioEnd, ScenarioId, TileId, TilePosition, TimeOfDay, UndoError, apply, final_score,
    legal_actions, new_game, pending_decision, view,
};
use arene::*;

use BasicAction::*;

fn reco() -> Scenario {
    Scenario::first_reconnaissance().without_dummy()
}

fn pos(i: i32, j: i32) -> TilePosition {
    TilePosition::from_lattice(i, j)
}

/// La main des tests : deux Marche pour explorer, puis de quoi finir le tour.
fn hand() -> Vec<CardId> {
    vec![
        card(March, 1),
        card(March, 2),
        card(Stamina, 1),
        card(Rage, 1),
        card(Promise, 1),
    ]
}

/// L'arène de `scenario`, figurine au village, `next` derrière les tuiles de campagne 1 et 2.
fn at_village(scenario: Scenario, next: &[TileId]) -> Vec<Event> {
    Sandbox {
        scenario,
        figure: Some(VILLAGE),
        hand: hand(),
        next: next.to_vec(),
        ..Sandbox::default()
    }
    .events()
}

fn state_of(events: &[Event]) -> GameState {
    events.iter().fold(GameState::default(), apply)
}

/// Une Marche jouée : 2 points de mouvement, de quoi explorer l'emplacement (1, 1).
fn ready_to_explore(state: &GameState) -> GameState {
    done(state, basic(card(March, 1)))
}

fn explore() -> Action {
    Action::Explore {
        position: pos(1, 1),
    }
}

fn fame_events(events: &[Event]) -> Vec<u32> {
    events
        .iter()
        .filter_map(|event| match event {
            Event::FameGained { fame, .. } => Some(*fame),
            _ => None,
        })
        .collect()
}

/// La première fin de tour légale.
fn end_turn(state: &GameState) -> Action {
    legal_actions(state, SOLO)
        .into_iter()
        .find(|action| matches!(action, Action::EndTurn { .. }))
        .expect("finir le tour est légal")
}

/// Explore la tuile de cité royaume 5 (Première reconnaissance) : le tour de la révélation.
fn city_revealed() -> GameState {
    let state = ready_to_explore(&state_of(&at_village(reco(), &[TileId::Core(5)])));
    done(&state, explore())
}

/// Le dernier tour, commencé : la fin du tour de la révélation est passée.
fn last_turn() -> GameState {
    let state = city_revealed();
    let action = end_turn(&state);
    done(&state, action)
}

// --- Scénario et gloire ---

// C1 — MKUE p. 18, *Map* et *Solo Variant*
#[test]
fn first_reconnaissance_matches_livret_solo_variant() {
    let scenario = reco();
    assert_eq!(scenario.id(), ScenarioId::FirstReconnaissance);
    assert_eq!(scenario.shape(), MapShape::Wedge);
    let deck = scenario.tile_deck();
    assert_eq!(deck.countryside, 8);
    assert_eq!(deck.countryside_order, CountrysideOrder::ByNumber);
    assert_eq!((deck.core_city, deck.core_non_city), (1, 2));
    assert_eq!(scenario.rounds(), 4);
    assert_eq!(scenario.fame_per_tile(), 1);
    assert_eq!(scenario.end(), ScenarioEnd::CityRevealed);
    assert_eq!(scenario.scoring(), FinalScoring::AchievementsWithoutTitles);

    // À graine égale, même mise en place que le bac à sable, hors pioches communes et offres
    // (spec 050 : pas d'élites, actions avancées 1 à 16).
    let setup = |scenario| {
        let config = GameConfig {
            seed: 7,
            scenario,
            hero: Hero::Arythea,
            dummy_hero: Default::default(),
        };
        new_game(&config, &mut config.rngs())
            .into_iter()
            .filter(|event| {
                !matches!(
                    event,
                    Event::CommonDeckShuffled { .. } | Event::OfferCardDealt { .. }
                )
            })
            .collect::<Vec<_>>()
    };
    let (reco_events, sandbox_events) = (setup(reco()), setup(Scenario::sandbox()));
    assert_eq!(reco_events[1..], sandbox_events[1..]);
}

// Le bac à sable garde ses valeurs neutres (spec 012).
#[test]
fn sandbox_keeps_neutral_scenario_rules() {
    let sandbox = Scenario::sandbox();
    assert_eq!(sandbox.fame_per_tile(), 0);
    assert_eq!(sandbox.end(), ScenarioEnd::RoundLimit);
    assert_eq!(sandbox.scoring(), FinalScoring::None);
}

// C2 — MKUE p. 18 : les tuiles de la mise en place ne sont pas révélées par le joueur.
#[test]
fn setup_tiles_give_no_fame() {
    let config = GameConfig {
        seed: 3,
        scenario: reco(),
        hero: Hero::Arythea,
        dummy_hero: Default::default(),
    };
    let events = new_game(&config, &mut config.rngs());
    assert!(fame_events(&events).is_empty());
    assert_eq!(state_of(&events).fame(SOLO), 0);
}

// C3 — MKUE p. 18, *Special Rules*
#[test]
fn revealing_a_tile_gives_one_fame() {
    let state = ready_to_explore(&state_of(&at_village(reco(), &[TileId::Countryside(3)])));
    let events = events_of(&state, &explore());
    assert!(matches!(events[1], Event::TileRevealed { .. }));
    assert_eq!(
        events.last(),
        Some(&Event::FameGained {
            player: SOLO,
            fame: 1
        })
    );
    assert_eq!(events.iter().fold(state, apply).fame(SOLO), 1);
}

// La gloire suit les jetons d'ennemis posés par la tuile (spec 030), défenseurs compris, retournés
// s'ils touchent la figurine (spec 043), et les jetons de ruines (spec 045).
#[test]
fn fame_comes_after_the_rampaging_tokens() {
    // Royaume 2 : un draconien errant (et, depuis la spec 043, le défenseur de son site fortifié).
    let state = ready_to_explore(&state_of(&at_village(reco(), &[TileId::Core(2)])));
    let events = events_of(&state, &explore());
    let fame = events
        .iter()
        .position(|event| matches!(event, Event::FameGained { .. }))
        .expect("gloire de la tuile");
    assert!(
        events[..fame]
            .iter()
            .any(|event| matches!(event, Event::EnemyTokenPlaced { .. }))
    );
    assert!(matches!(
        events[fame - 1],
        Event::EnemyTokenPlaced { .. }
            | Event::DefenderPlaced { .. }
            | Event::EnemyTokenRevealed { .. }
            | Event::RuinTokenPlaced { .. }
    ));
    assert_eq!(fame, events.len() - 1);
}

// C4
#[test]
fn sandbox_gives_no_fame_for_tiles() {
    let state = ready_to_explore(&state_of(&at_village(
        Scenario::sandbox(),
        &[TileId::Countryside(3)],
    )));
    let events = events_of(&state, &explore());
    assert!(fame_events(&events).is_empty());
    assert_eq!(events.iter().fold(state, apply).fame(SOLO), 0);
}

// C5 — MKUE p. 11, 6 ; spec 035
#[test]
fn tile_fame_levels_up_at_end_of_turn() {
    let mut events = at_village(reco(), &[TileId::Countryside(3)]);
    events.extend(progress(2, 1));
    let state = done(&ready_to_explore(&state_of(&events)), explore());
    assert_eq!((state.fame(SOLO), state.level(SOLO)), (3, 1));
    assert_eq!(state.pending_level(SOLO), Some(2));

    let action = end_turn(&state);
    let events = events_of(&state, &action);
    assert!(events.contains(&Event::LevelGained {
        player: SOLO,
        level: 2
    }));
    assert_eq!(events.iter().fold(state, apply).level(SOLO), 2);
}

// C6 — MKUE p. 18, *no matter which type* ; spec 015
#[test]
fn removed_tile_revealed_gives_fame() {
    // Pioche vide (les tuiles 1 et 2 sont posées), la tuile 3 en (2, 0) : l'emplacement (1, 1)
    // ne se remplit qu'avec une tuile écartée.
    let mut events = at_village(reco(), &[]);
    events.push(Event::TileRevealed {
        tile: TileId::Countryside(3),
        position: pos(2, 0),
    });
    let state = ready_to_explore(&state_of(&events));
    assert!(state.tile_deck().is_empty());
    let events = events_of(&state, &explore());
    assert_eq!(fame_events(&events), [1]);
}

// --- Fin du scénario ---

// C7 — MKUE p. 18, *Scenario End*
#[test]
fn revealing_city_announces_last_turn() {
    let state = ready_to_explore(&state_of(&at_village(reco(), &[TileId::Core(5)])));
    let events = events_of(&state, &explore());
    let tail = &events[events.len() - 2..];
    assert_eq!(
        tail,
        [
            Event::FameGained {
                player: SOLO,
                fame: 1
            },
            Event::LastTurnAnnounced { player: SOLO },
        ]
    );
    let state = events.iter().fold(state, apply);
    assert_eq!(state.last_turn(), LastTurn::Next);
    assert_eq!(state.game_end(), None);
    // Le joueur continue son tour normalement.
    let legal = legal_actions(&state, SOLO);
    assert!(
        legal
            .iter()
            .any(|action| matches!(action, Action::PlayCard { .. }))
    );
    assert!(
        legal
            .iter()
            .any(|action| matches!(action, Action::EndTurn { .. }))
    );
}

// C8
#[test]
fn turn_after_city_is_the_last_one() {
    let state = city_revealed();
    let action = end_turn(&state);
    let events = events_of(&state, &action);
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::CardDrawn { .. }))
    );
    assert!(matches!(events.last(), Some(Event::TurnStarted { .. })));
    let state = events.iter().fold(state, apply);
    assert_eq!(state.last_turn(), LastTurn::Current);
    assert!(pending_decision(&state).is_some());
}

// C9 — MKUE p. 4, 4.c
#[test]
fn game_ends_after_last_turn() {
    let state = last_turn();
    let state = done(&state, basic(card(Stamina, 1)));
    let action = end_turn(&state);
    let events = events_of(&state, &action);
    assert!(events.contains(&Event::TurnEnded { player: SOLO }));
    assert_eq!(
        events.last(),
        Some(&Event::GameEnded {
            reason: GameEndReason::CityRevealed
        })
    );
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::CardDrawn { .. } | Event::TurnStarted { .. }))
    );
    let state = events.iter().fold(state, apply);
    assert_eq!(state.game_end(), Some(GameEndReason::CityRevealed));
    assert_eq!(pending_decision(&state), None);
    assert!(legal_actions(&state, SOLO).is_empty());
}

// L5 — un changement de niveau gagné pendant le dernier tour se fait avant `GameEnded`.
#[test]
fn level_gained_on_last_turn_comes_before_game_end() {
    let mut events = at_village(reco(), &[TileId::Countryside(3), TileId::Core(5)]);
    events.extend(progress(2, 1));
    events.extend([
        Event::LastTurnAnnounced { player: SOLO },
        Event::TurnStarted {
            player: SOLO,
            turn: 2,
        },
    ]);
    let state = done(&ready_to_explore(&state_of(&events)), explore());
    let action = end_turn(&state);
    let events = events_of(&state, &action);
    let level = events
        .iter()
        .position(|event| matches!(event, Event::LevelGained { level: 2, .. }))
        .expect("niveau 2 gagné");
    assert_eq!(level, events.len() - 2);
    assert!(matches!(events.last(), Some(Event::GameEnded { .. })));
}

/// Le dernier tour commence, deck vide : la main est `hand`, le reste est dans la défausse.
fn last_turn_with_empty_deck(hand: &[CardId]) -> GameState {
    let mut events = at_village(reco(), &[]);
    events.extend([
        Event::LastTurnAnnounced { player: SOLO },
        Event::TurnStarted {
            player: SOLO,
            turn: 2,
        },
        Event::DeckShuffled {
            player: SOLO,
            order: hand.to_vec(),
        },
    ]);
    events.extend(
        hand.iter()
            .map(|&card| Event::CardDrawn { player: SOLO, card }),
    );
    state_of(&events)
}

// C10 — MKUE p. 18 ; p. 4, 4.d
#[test]
fn round_end_during_last_turn_ends_game() {
    let state = last_turn_with_empty_deck(&hand());
    assert!(legal_actions(&state, SOLO).contains(&Action::AnnounceEndOfRound));
    let events = events_of(&state, &Action::AnnounceEndOfRound);
    assert_eq!(
        events,
        [
            Event::RoundEnded { round: 1 },
            Event::GameEnded {
                reason: GameEndReason::CityRevealed
            },
        ]
    );
    let state = events.iter().fold(state, apply);
    assert_eq!(state.round(), 1);
    assert_eq!(pending_decision(&state), None);
}

// C11 — MKUE p. 6, 3.b
#[test]
fn empty_hand_on_last_turn_ends_game() {
    let state = last_turn_with_empty_deck(&[]);
    assert_eq!(legal_actions(&state, SOLO), [Action::AnnounceEndOfRound]);
    let events = events_of(&state, &Action::AnnounceEndOfRound);
    assert_eq!(
        events.last(),
        Some(&Event::GameEnded {
            reason: GameEndReason::CityRevealed
        })
    );
}

/// Début de la 4e manche, de nuit, deck et main vides : seule action, l'annonce.
fn fourth_round_with_empty_deck(scenario: Scenario) -> GameState {
    let mut events = Sandbox {
        scenario,
        figure: Some(VILLAGE),
        time: TimeOfDay::Night,
        ..Sandbox::default()
    }
    .events();
    events.extend([
        Event::RoundStarted {
            round: 4,
            time_of_day: TimeOfDay::Night,
        },
        Event::DeckShuffled {
            player: SOLO,
            order: Vec::new(),
        },
    ]);
    state_of(&events)
}

// C12 — MKUE p. 4, 5.a ; spec 023
#[test]
fn round_limit_ends_game_with_game_ended() {
    for scenario in [reco(), Scenario::sandbox()] {
        let state = fourth_round_with_empty_deck(scenario);
        let events = events_of(&state, &Action::AnnounceEndOfRound);
        assert_eq!(
            events,
            [
                Event::RoundEnded { round: 4 },
                Event::GameEnded {
                    reason: GameEndReason::LastRound
                },
            ],
            "{:?}",
            scenario.id()
        );
        let state = events.iter().fold(state, apply);
        assert_eq!(state.game_end(), Some(GameEndReason::LastRound));
    }
}

// C13
#[test]
fn sandbox_ignores_city_reveal() {
    let state = ready_to_explore(&state_of(&at_village(
        Scenario::sandbox(),
        &[TileId::Core(5)],
    )));
    let events = events_of(&state, &explore());
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::LastTurnAnnounced { .. }))
    );
    let state = events.iter().fold(state, apply);
    assert_eq!(state.last_turn(), LastTurn::NotAnnounced);
    let action = end_turn(&state);
    let state = done(&state, action);
    assert_eq!(state.last_turn(), LastTurn::NotAnnounced);
    assert!(pending_decision(&state).is_some());
}

// L1 — une seconde cité révélée n'annonce rien de plus.
#[test]
fn second_city_announces_nothing() {
    let mut events = at_village(reco(), &[TileId::Core(5)]);
    events.push(Event::LastTurnAnnounced { player: SOLO });
    let state = ready_to_explore(&state_of(&events));
    let events = events_of(&state, &explore());
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::LastTurnAnnounced { .. }))
    );
}

// --- Décompte ---

/// Partie de Première reconnaissance : gloire `fame`, cristaux `crystals`, la main `hand`, le deck
/// `deck` derrière elle, puis `discarded` (pris dans la main) dans la défausse.
fn scored(
    fame: u32,
    crystals: &[BasicColor],
    hand: &[CardId],
    deck: &[CardId],
    discarded: &[CardId],
) -> GameState {
    let mut events = at_village(reco(), &[]);
    let mut order = hand.to_vec();
    order.extend_from_slice(deck);
    events.push(Event::DeckShuffled {
        player: SOLO,
        order,
    });
    events.extend(
        hand.iter()
            .map(|&card| Event::CardDrawn { player: SOLO, card }),
    );
    events.extend(
        discarded
            .iter()
            .map(|&card| Event::CardDiscarded { player: SOLO, card }),
    );
    events.extend(progress(fame, 1));
    events.extend(crystals.iter().map(|&color| Event::CrystalGained {
        player: SOLO,
        color,
    }));
    state_of(&events)
}

// C14 — MKUE p. 15 ; Compendium, *Final Scoring* S4
#[test]
fn final_score_counts_achievements_without_titles() {
    use BasicColor::*;
    let state = scored(
        17,
        &[Red, Red, Blue],
        &[wound(1), card(March, 1), card(Rage, 1)],
        &[wound(2), card(Stamina, 1)],
        &[],
    );
    let score = final_score(&state, SOLO);
    assert_eq!(
        score,
        FinalScore {
            fame: 17,
            crystals: 3,
            wounds: 2,
            // Spec 050 : ni action avancée ni sort.
            advanced_actions: 0,
            spells: 0,
            // Spec 042 : aucune unité.
            unit_levels: 0,
            // Spec 041 : ni artefact ni Bouclier.
            artifacts: 0,
            shields: 0,
            // Spec 044 : aucun site d'aventure.
            adventure_shields: 0,
            // Spec 064 : pas de partie « conquête » hors de Conquête en solitaire.
            conquest: None,
        }
    );
    let points: Vec<i64> = Achievement::ALL
        .into_iter()
        .map(|achievement| score.points(achievement))
        .collect();
    assert_eq!(points, [0, 1, 0, 0, 0, -4]);
    assert_eq!(score.total(), 14);
}

// C14 — les Blessures de la défausse comptent aussi.
#[test]
fn wounds_in_discard_pile_count() {
    let state = scored(
        0,
        &[],
        &[wound(1), card(March, 1)],
        &[wound(2)],
        &[wound(1)],
    );
    assert_eq!(state.discard_pile(SOLO), [wound(1)]);
    assert_eq!(final_score(&state, SOLO).wounds, 2);
}

// C15 — MKUE p. 15
#[test]
fn loot_counts_one_per_two_crystals() {
    use BasicColor::*;
    for (crystals, points) in [
        (&[Red][..], 0),
        (&[Red, Red][..], 1),
        (&[Red, Blue, Green, White, White][..], 2),
    ] {
        let state = scored(0, crystals, &[card(March, 1)], &[], &[]);
        assert_eq!(
            final_score(&state, SOLO).points(Achievement::Loot),
            points,
            "{crystals:?}"
        );
    }
}

// C16 — Compendium, *Final Scoring* S5
#[test]
fn healed_wounds_do_not_count() {
    let mut events = at_village(reco(), &[]);
    events.extend([
        Event::DeckShuffled {
            player: SOLO,
            order: vec![wound(1), wound(2), card(March, 1)],
        },
        Event::CardDrawn {
            player: SOLO,
            card: wound(1),
        },
        Event::CardDrawn {
            player: SOLO,
            card: wound(2),
        },
        Event::WoundsHealed {
            player: SOLO,
            wounds: vec![wound(1)],
        },
    ]);
    let score = final_score(&state_of(&events), SOLO);
    assert_eq!(score.wounds, 1);
    assert_eq!(score.points(Achievement::Beating), -2);
}

// C17
#[test]
fn final_score_can_be_negative() {
    let state = scored(2, &[], &[wound(1), wound(2)], &[wound(3)], &[wound(2)]);
    assert_eq!(final_score(&state, SOLO).total(), -4);
}

// C18
#[test]
fn sandbox_has_no_final_score() {
    let state = fourth_round_with_empty_deck(Scenario::sandbox());
    let state = done(&state, Action::AnnounceEndOfRound);
    let player_view = view(&state, SOLO);
    assert_eq!(player_view.game_end, Some(GameEndReason::LastRound));
    assert_eq!(player_view.final_score, None);
}

// C19 — « Information cachée » : le score n'entre dans la vue qu'une fois la partie terminée.
#[test]
fn final_score_only_after_game_end() {
    let state = last_turn();
    assert_eq!(view(&state, SOLO).final_score, None);
    let state = done(&state, basic(card(Stamina, 1)));
    let action = end_turn(&state);
    let state = done(&state, action);
    let player_view = view(&state, SOLO);
    assert_eq!(player_view.game_end, Some(GameEndReason::CityRevealed));
    assert_eq!(player_view.final_score, Some(final_score(&state, SOLO)));
    assert_eq!(player_view.final_score.map(|score| score.fame), Some(1));
}

// --- Retour en arrière ---

// C20 — spec 024
#[test]
fn last_turn_survives_undo_after_reveal() {
    let (mut runner, puppet) = runner(at_village(reco(), &[TileId::Core(5)]));
    play(&mut runner, &puppet, basic(card(March, 1)));
    play(&mut runner, &puppet, explore());
    assert_eq!(runner.state().last_turn(), LastTurn::Next);
    play(&mut runner, &puppet, basic(card(Stamina, 1)));
    assert_eq!(runner.undo(), Ok(()));
    assert_eq!(runner.state().last_turn(), LastTurn::Next);
    assert_eq!(runner.state().fame(SOLO), 1);
    assert_eq!(runner.undo(), Err(UndoError::InformationRevealed));
}

// L7 — `GameEnded` est un point de non-retour.
#[test]
fn finished_game_cannot_be_undone() {
    let mut events = at_village(reco(), &[]);
    events.extend([
        Event::LastTurnAnnounced { player: SOLO },
        Event::TurnStarted {
            player: SOLO,
            turn: 2,
        },
    ]);
    let (mut runner, puppet) = runner(events);
    play(&mut runner, &puppet, basic(card(March, 1)));
    let action = end_turn(runner.state());
    play(&mut runner, &puppet, action);
    assert_eq!(runner.state().game_end(), Some(GameEndReason::CityRevealed));
    assert!(runner.undo().is_err());
    assert_eq!(runner.step(), Ok(arcanum_core::StepOutcome::Finished));
}
