//! Scénarios de la spec 030 (ennemis errants et provocation).

use std::collections::BTreeSet;

use arcanum_core::{
    Action, BasicAction, CardPlay, ContentError, ContentSources, Enemy, EnemyAbility, EnemyAttack,
    EnemyColor, EnemyToken, Event, Feature, GameConfig, GameContent, GameState, Hero, Hex,
    PlayerEvent, PlayerId, Rampaging, Resistance, Scenario, TileId, TilePosition, TimeOfDay, apply,
    decide, filter_event, legal_actions, new_game, provoked, routes, routes_avoiding, view,
};

mod arene;
use arene::*;

use BasicAction::*;

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

/// Le bac à sable des tests de déplacement (`Sandbox::moving`) : tuiles de campagne 1 et 2 puis
/// `next` dans la pioche ; figurine sur `figure` ; `placed` sur le dessus des piles, puis posés.
fn setup(next: &[TileId], figure: Hex, placed: &[(EnemyToken, Hex)]) -> Vec<Event> {
    Sandbox {
        next: next.to_vec(),
        placed: placed.to_vec(),
        ..Sandbox::moving(Some(figure))
    }
    .events()
}

fn state_of(events: &[Event]) -> GameState {
    events.iter().fold(GameState::default(), apply)
}

/// Bac à sable avec ses deux jetons, la figurine sur `figure`.
fn sandbox(figure: Hex) -> GameState {
    state_of(&setup(&[], figure, &sandbox_tokens()))
}

fn step(to: Hex) -> Action {
    Action::MoveHero { to }
}

fn decided(state: &GameState, action: &Action) -> Vec<Event> {
    decide(state, SOLO, action, &mut config(0).rngs())
        .unwrap_or_else(|error| panic!("{action:?} doit être légale : {error:?}"))
}

/// Deux Marche jouées : 4 points de mouvement.
fn with_four_points(state: GameState) -> GameState {
    let state = done(&state, basic(card(March, 1)));
    done(&state, basic(card(March, 2)))
}

fn provocations(events: &[Event]) -> Vec<Vec<Hex>> {
    events
        .iter()
        .filter_map(|event| match event {
            Event::EnemiesProvoked { enemies, .. } => Some(enemies.clone()),
            _ => None,
        })
        .collect()
}

fn placed_tokens(events: &[Event]) -> Vec<(EnemyToken, Hex)> {
    events
        .iter()
        .filter_map(|event| match event {
            Event::EnemyTokenPlaced { token, hex } => Some((*token, *hex)),
            _ => None,
        })
        .collect()
}

fn moves(state: &GameState) -> Vec<Action> {
    legal_actions(state, SOLO)
        .into_iter()
        .filter(|action| matches!(action, Action::MoveHero { .. } | Action::Explore { .. }))
        .collect()
}

// --- Contenu ---

// C1 — cartes/ennemis.md : 60 jetons ; L5 : assez de jetons pour les ennemis errants imprimés.
#[test]
fn enemy_tokens_match_the_component_list() {
    let content = GameContent::embedded();
    let counts: Vec<usize> = EnemyColor::ALL
        .into_iter()
        .map(|color| content.enemy_tokens(color).len())
        .collect();
    assert_eq!(counts, [12, 10, 10, 10, 8, 10]);
    for color in EnemyColor::ALL {
        let tokens = content.enemy_tokens(color);
        assert!(tokens.iter().all(|token| token.color() == color));
        let distinct: BTreeSet<_> = tokens.iter().collect();
        assert_eq!(distinct.len(), tokens.len());
    }

    let printed = |wanted: Rampaging| {
        content
            .tiles()
            .iter()
            .flat_map(|tile| std::iter::once(tile.center).chain(tile.sides))
            .filter(|space| space.feature == Some(Feature::Rampaging(wanted)))
            .count()
    };
    assert!(printed(Rampaging::OrcMarauders) <= counts[0]);
    assert!(printed(Rampaging::Draconum) <= counts[4]);
}

// C2 — cartes/ennemis.md
#[test]
fn enemy_values_are_read_from_content() {
    let prowlers = Enemy::Prowlers.definition();
    assert_eq!(prowlers.color, EnemyColor::Green);
    assert_eq!(prowlers.armor, 3);
    assert_eq!(prowlers.attack, EnemyAttack::Physical(4));
    assert_eq!(prowlers.fame, 2);

    let high_dragon = Enemy::HighDragon.definition();
    assert_eq!(high_dragon.color, EnemyColor::Red);
    assert_eq!(high_dragon.armor, 9);
    assert_eq!(high_dragon.attack, EnemyAttack::ColdFire(6));
    assert_eq!(high_dragon.abilities, [EnemyAbility::Brutal]);
    assert_eq!(high_dragon.resistances, [Resistance::Fire, Resistance::Ice]);
    assert_eq!(high_dragon.fame, 9);
}

// C3 — cartes/README.md : la version Lost Legion fait foi.
#[test]
fn altem_enemies_use_lost_legion_errata() {
    assert_eq!(
        Enemy::AltemGuardsmen.definition().attack,
        EnemyAttack::Physical(5)
    );
    assert!(
        !Enemy::AltemMages
            .definition()
            .abilities
            .contains(&EnemyAbility::Fortified)
    );
}

fn with_enemies(text: &str) -> Result<GameContent, ContentError> {
    GameContent::parse(ContentSources {
        enemies: text,
        ..ContentSources::EMBEDDED
    })
}

// C4
#[test]
fn enemy_content_is_validated() {
    let embedded = ContentSources::EMBEDDED.enemies;
    let prowlers = "(id: Prowlers, color: Green, count: 2, armor: 3, attack: Physical(4), \
                    abilities: [], resistances: [], fame: 2),";

    let twice = embedded.replacen("[", &format!("[{prowlers}"), 1);
    match with_enemies(&twice) {
        Err(ContentError::Duplicate { file, id }) => {
            assert_eq!(file, "enemies.ron");
            assert_eq!(id, "Prowlers");
        }
        other => panic!("{other:?}"),
    }

    let none = embedded.replacen(
        "(id: Prowlers, color: Green, count: 2,",
        "(id: Prowlers, color: Green, count: 0,",
        1,
    );
    assert_ne!(none, embedded);
    assert!(matches!(
        with_enemies(&none),
        Err(ContentError::NoEnemyToken {
            enemy: Enemy::Prowlers
        })
    ));

    let missing = embedded.replacen(
        "(id: Prowlers, color: Green, count: 2, armor: 3, attack: Physical(4), abilities: [], resistances: [], fame: 2),",
        "",
        1,
    );
    assert_ne!(missing, embedded);
    assert!(matches!(
        with_enemies(&missing),
        Err(ContentError::Missing { .. })
    ));
}

// --- Mise en place et révélation ---

fn start(seed: u64) -> (GameState, Vec<Event>) {
    let config = config(seed);
    let events = new_game(&config, &mut config.rngs());
    (state_of(&events), events)
}

// C5 — MKUE p. 3, Enemy and Ruin token piles
#[test]
fn setup_shuffles_six_enemy_piles() {
    let (_, events) = start(1);
    let rolled = events
        .iter()
        .position(|event| matches!(event, Event::SourceRolled { .. }))
        .expect("la Source est lancée");
    let first_token = events
        .iter()
        .position(|event| matches!(event, Event::EnemyTokenPlaced { .. }))
        .expect("le bac à sable a des orques");
    let shuffles: Vec<(EnemyColor, &Vec<EnemyToken>)> = events
        .iter()
        .enumerate()
        .filter_map(|(index, event)| match event {
            Event::EnemyPileShuffled { color, order } => {
                assert!(
                    rolled < index && index < first_token,
                    "ordre des événements"
                );
                Some((*color, order))
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        shuffles.iter().map(|(color, _)| *color).collect::<Vec<_>>(),
        EnemyColor::ALL
    );
    for (color, order) in shuffles {
        let mut sorted = order.clone();
        sorted.sort();
        let mut all = GameContent::embedded().enemy_tokens(color);
        all.sort();
        assert_eq!(sorted, all);
    }
}

// C6 — MKUE p. 3, Map ; carte de site Marauding Orcs
#[test]
fn setup_places_tokens_on_printed_orcs() {
    let (state, _) = start(2);
    let tokens = state.enemy_tokens();
    assert_eq!(tokens.keys().copied().collect::<Vec<_>>(), [ORCS_1, ORCS_2]);
    assert!(tokens.values().all(|tokens| {
        tokens
            .iter()
            .all(|token| token.color() == EnemyColor::Green)
    }));
    assert_eq!(state.enemy_pile(EnemyColor::Green).len(), 10);
    assert_eq!(state.enemy_pile(EnemyColor::Red).len(), 8);
}

// C7
#[test]
fn enemy_draws_follow_the_seed() {
    let tokens = |seed| start(seed).0.enemy_tokens().clone();
    assert_eq!(tokens(3), tokens(3));
    assert!((4..20).any(|seed| tokens(seed) != tokens(3)));
}

/// Bac à sable dont la pioche donne `next`, la figurine au village avec 4 points.
fn before_exploring(next: TileId) -> GameState {
    with_four_points(state_of(&setup(&[next], VILLAGE, &sandbox_tokens())))
}

fn explore() -> Action {
    Action::Explore {
        position: pos(1, 1),
    }
}

// C8 — MKUE p. 7, 5.f
#[test]
fn revealing_a_tile_places_its_rampaging_enemies() {
    let state = before_exploring(TileId::Core(5));
    let green = state.enemy_pile(EnemyColor::Green).to_vec();
    let events = decided(&state, &explore());
    assert!(matches!(events[0], Event::Explored { .. }));
    assert!(matches!(events[1], Event::TileRevealed { .. }));
    // Royaume 5 en (1,1), centre (4,−5) : orques à l'E puis au S-O (spec 062 : puis la cité
    // verte et ses défenseurs, après les errants).
    assert_eq!(
        events[2..4],
        [
            Event::EnemyTokenPlaced {
                token: green[0],
                hex: Hex::new(5, -5),
            },
            Event::EnemyTokenPlaced {
                token: green[1],
                hex: Hex::new(3, -4),
            },
        ]
    );
    let after = events.iter().fold(state, apply);
    // Spec 062 : la case de la cité compte aussi.
    assert_eq!(after.enemy_tokens().len(), 5);
    assert_eq!(after.enemy_pile(EnemyColor::Green), &green[2..]);
}

// C9 — carte de site Draconum
#[test]
fn draconum_comes_from_the_red_pile() {
    let state = before_exploring(TileId::Core(2));
    let red = state.enemy_pile(EnemyColor::Red)[0];
    let events = decided(&state, &explore());
    // Royaume 2 en (1,1) : draconien au S-E du centre (4,−5).
    assert_eq!(placed_tokens(&events), [(red, Hex::new(4, -4))]);
}

// C10
#[test]
fn tile_without_rampaging_places_no_token() {
    let state = before_exploring(TileId::Countryside(3));
    let events = decided(&state, &explore());
    assert!(placed_tokens(&events).is_empty());
}

// --- Déplacement ---

// C11 — MKUE p. 7, 4.b ; L1
#[test]
fn cannot_enter_a_rampaging_enemy_space() {
    let state = with_four_points(sandbox(VILLAGE));
    assert!(!legal_actions(&state, SOLO).contains(&step(ORCS_2)));
    refused(&state, step(ORCS_2));
}

// C12 — MKUE p. 7, 4.c
#[test]
fn moving_along_an_enemy_provokes_it() {
    let state = with_four_points(sandbox(VILLAGE));
    let events = decided(&state, &step(PLAIN_W));
    assert!(matches!(events[0], Event::HeroMoved { to: PLAIN_W, .. }));
    assert_eq!(
        events[1],
        Event::EnemiesProvoked {
            player: SOLO,
            enemies: vec![ORCS_2],
        }
    );
    let after = events.iter().fold(state, apply);
    assert_eq!(after.figure(SOLO), Some(PLAIN_W));
    assert_eq!(after.move_points(SOLO), 0);
    assert!(after.movement_ended(SOLO));
    assert_eq!(after.enemies_to_fight(SOLO), [ORCS_2]);
}

// C13 — MKUE p. 7, 4.c
#[test]
fn entering_next_to_an_enemy_does_not_provoke() {
    let state = with_four_points(sandbox(PLAIN_SW));
    let events = decided(&state, &step(VILLAGE));
    assert!(provocations(&events).is_empty());
}

// C14 — MKUE p. 7, 4.c
#[test]
fn leaving_an_enemy_does_not_provoke() {
    let state = with_four_points(sandbox(VILLAGE));
    let events = decided(&state, &step(PLAIN_SW));
    assert!(provocations(&events).is_empty());
}

// C15 — MKUE p. 8, 2.a
#[test]
fn one_step_can_provoke_two_enemies() {
    let mut tokens = sandbox_tokens();
    tokens.push((token(Enemy::Prowlers, 2), PLAIN_SW));
    let state = with_four_points(state_of(&setup(&[], VILLAGE, &tokens)));
    let events = decided(&state, &step(PLAIN_W));
    let provoked: Vec<BTreeSet<Hex>> = provocations(&events)
        .into_iter()
        .map(|enemies| enemies.into_iter().collect())
        .collect();
    assert_eq!(provoked, [BTreeSet::from([PLAIN_SW, ORCS_2])]);
}

/// Au village avec 4 points, puis le pas qui provoque les orques ②.
fn after_provocation() -> GameState {
    let state = with_four_points(sandbox(VILLAGE));
    done(&state, step(PLAIN_W))
}

// C16 — MKUE p. 7, 4.c ; p. 6, 6.d ; L1, L2. Depuis la spec 031 (C1), seul « Engager le combat »
// suit la provocation : plus de carte jouable, plus de fin du tour avant le combat.
#[test]
fn provocation_ends_movement_for_the_turn() {
    let state = after_provocation();
    assert!(moves(&state).is_empty());
    assert_eq!(legal_actions(&state, SOLO), vec![Action::StartCombat]);
    refused(&state, basic(card(Stamina, 1)));
    refused(&state, step(VILLAGE));
    refused(&state, explore());
}

// C17 — FAQ p2 ; L4
#[test]
fn exploring_does_not_provoke() {
    let state = before_exploring(TileId::Core(5));
    let events = decided(&state, &explore());
    assert!(provocations(&events).is_empty());
    let after = events.iter().fold(state, apply);
    assert!(!after.movement_ended(SOLO));
    assert!(after.enemies_to_fight(SOLO).is_empty());
    assert!(legal_actions(&after, SOLO).contains(&step(PLAIN_SW)));
}

// C18 — Q1. Depuis la spec 031 (C21), le combat est mené, sans victoire, avant la fin du tour.
#[test]
fn provoked_enemies_stay_after_end_of_turn() {
    let mut state = done(&after_provocation(), Action::StartCombat);
    for action in [
        Action::NextCombatPhase,
        Action::NextCombatPhase,
        Action::EndCombat,
    ] {
        state = done(&state, action);
    }
    let state = done(
        &state,
        Action::EndTurn {
            discard: Vec::new(),
            reclaim: None,
            glade: None,
            steady_tempo: false,
        },
    );
    assert!(!state.movement_ended(SOLO));
    assert!(state.enemies_to_fight(SOLO).is_empty());
    assert_eq!(state.enemy_tokens_at(ORCS_2), [token(Enemy::Prowlers, 1)]);

    // Deux cartes de côté : 2 points, de quoi entrer sur une plaine voisine.
    let mut state = state;
    for _ in 0..2 {
        // Pas une Blessure, reçue des Rôdeurs non parés (spec 031).
        let card = *state
            .hand(SOLO)
            .iter()
            .find(|card| !card.card().is_wound())
            .expect("une carte jouable");
        state = done(
            &state,
            Action::PlayCard {
                card,
                play: CardPlay::Sideways(arcanum_core::SidewaysEffect::Move),
            },
        );
    }
    assert!(
        legal_actions(&state, SOLO)
            .iter()
            .any(|action| matches!(action, Action::MoveHero { .. }))
    );
}

// C19 — MKUE p. 7, 4.b et 4.c
#[test]
fn routes_avoid_enemies_and_provocations() {
    let state = sandbox(VILLAGE);
    let enemies: Vec<Hex> = state.enemy_tokens().keys().copied().collect();
    let to_mine = |enemies: &[Hex]| {
        routes_avoiding(state.map(), VILLAGE, TimeOfDay::Day, enemies)
            .into_iter()
            .find(|route| route.to == GREEN_MINE)
            .expect("la mine est atteignable")
    };
    // Sans jetons, le trajet le plus court passe par la plaine (2,−2), voisine des orques ②.
    assert_eq!(to_mine(&[]).cost, 5);
    let route = to_mine(&enemies);
    assert_eq!(route.cost, 7);
    assert_eq!(route.steps, [PLAIN_SW, PLAIN_W, GREEN_MINE]);

    let all = routes_avoiding(state.map(), VILLAGE, TimeOfDay::Day, &enemies);
    assert!(all.iter().all(|route| !enemies.contains(&route.to)));
    for route in &all {
        let mut from = VILLAGE;
        for &hex in &route.steps {
            assert!(!enemies.contains(&hex));
            assert!(provoked(from, hex, &enemies).is_empty(), "{route:?}");
            from = hex;
        }
    }
    // `routes` (spec 017) reste le calcul sans jetons.
    assert_eq!(
        routes(state.map(), VILLAGE, TimeOfDay::Day),
        routes_avoiding(state.map(), VILLAGE, TimeOfDay::Day, &[])
    );
}

// --- Vue, journal et boucle ---

// C20
#[test]
fn view_shows_tokens_but_not_pile_order() {
    let state = after_provocation();
    let seen = view(&state, SOLO);
    assert_eq!(&seen.enemy_tokens, state.enemy_tokens());
    assert_eq!(seen.enemy_piles.get(&EnemyColor::Green), Some(&10));
    assert_eq!(seen.enemy_piles.get(&EnemyColor::White), Some(&10));
    assert!(seen.movement_ended);
    assert_eq!(seen.enemies_to_fight, [ORCS_2]);
}

// C21
#[test]
fn filtered_log_hides_enemy_pile_order() {
    let order = GameContent::embedded().enemy_tokens(EnemyColor::Red);
    let shuffled = Event::EnemyPileShuffled {
        color: EnemyColor::Red,
        order,
    };
    assert_eq!(
        filter_event(&shuffled, SOLO),
        PlayerEvent::EnemyPileShuffled {
            color: EnemyColor::Red,
            count: 8,
        }
    );
    let placed = Event::EnemyTokenPlaced {
        token: token(Enemy::Prowlers, 1),
        hex: ORCS_2,
    };
    assert_eq!(
        filter_event(&placed, PlayerId(1)),
        PlayerEvent::EnemyTokenPlaced {
            token: token(Enemy::Prowlers, 1),
            hex: ORCS_2,
        }
    );
    let provoked = Event::EnemiesProvoked {
        player: SOLO,
        enemies: vec![ORCS_2],
    };
    assert_eq!(
        filter_event(&provoked, PlayerId(1)),
        PlayerEvent::EnemiesProvoked {
            player: SOLO,
            enemies: vec![ORCS_2],
        }
    );
}

// C22
#[test]
fn replaying_log_rebuilds_enemies() {
    let (mut runner, puppet) = runner(setup(&[TileId::Core(5)], VILLAGE, &sandbox_tokens()));
    play(&mut runner, &puppet, basic(card(March, 1)));
    play(&mut runner, &puppet, basic(card(March, 2)));
    play(&mut runner, &puppet, explore());
    play(&mut runner, &puppet, step(PLAIN_W));
    assert!(runner.state().movement_ended(SOLO));
    // Spec 062 : avec la case de la cité verte et ses défenseurs.
    assert_eq!(runner.state().enemy_tokens().len(), 5);

    let replayed = runner
        .log()
        .entries()
        .iter()
        .fold(GameState::default(), |state, entry| {
            apply(state, &entry.event)
        });
    assert_eq!(&replayed, runner.state());
}

// C27 — MKUE p. 5, Reverting ; L7
#[test]
fn provoking_step_can_be_taken_back() {
    let (mut runner, puppet) = runner(setup(&[], VILLAGE, &sandbox_tokens()));
    play(&mut runner, &puppet, basic(card(March, 1)));
    play(&mut runner, &puppet, basic(card(March, 2)));
    let before = runner.state().clone();
    let rng = runner.rng_positions();

    play(&mut runner, &puppet, step(PLAIN_W));
    assert!(runner.state().movement_ended(SOLO));
    assert_eq!(runner.undo(), Ok(()));
    assert_eq!(runner.state(), &before);
    assert_eq!(runner.rng_positions(), rng);
    assert!(!runner.state().movement_ended(SOLO));
    assert!(runner.state().enemies_to_fight(SOLO).is_empty());
    assert_eq!(runner.state().move_points(SOLO), 4);
}
