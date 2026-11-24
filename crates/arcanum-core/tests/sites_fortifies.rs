//! Scénarios de la spec 043 (Sites fortifiés).

use arcanum_core::{
    Action, BasicAction, CardId, CountrysideOrder, EnemyColor, EnemyToken, Event, FinalScoring,
    GameConfig, GameState, Hero, Hex, MapShape, PlayerEvent, Scenario, ScenarioEnd, ScenarioId,
    TileDeckRecipe, TileId, TilePosition, TimeOfDay, apply, filter_event, new_game, view,
};

use BasicAction::*;

mod arene;
use arene::*;

// --- Outils ---

/// Explorer `position` depuis `figure` après une Marche (Mouvement 2), avec `next` sur le dessus
/// de la pioche de tuiles : les événements de l'exploration et l'état après.
fn explore(next: TileId, figure: Hex, position: TilePosition) -> (Vec<Event>, GameState) {
    let state = Sandbox {
        figure: Some(figure),
        hand: vec![card(March, 1)],
        next: vec![next],
        ..Sandbox::default()
    }
    .state();
    let state = done(&state, basic(card(March, 1)));
    let events = events_of(&state, &Action::Explore { position });
    let after = events.iter().fold(state, apply);
    (events, after)
}

/// Le bac à sable avec les tuiles de campagne mélangées : la mise en place peut poser la
/// campagne 3 ou 4 (comme le test du monastère de la spec 041).
fn shuffled_countryside() -> Scenario {
    Scenario::new(
        ScenarioId::Sandbox,
        MapShape::Wedge,
        TileDeckRecipe {
            countryside: 8,
            countryside_order: CountrysideOrder::Random,
            core_city: 1,
            core_non_city: 2,
        },
        4,
        0,
        ScenarioEnd::RoundLimit,
        FinalScoring::None,
    )
    .expect("scénario de test valide")
}

// --- Défenseurs ---

// C1 — Carte Keep, *When Revealed* : un jeton gris face cachée sur la forteresse révélée.
#[test]
fn revealed_keep_gets_a_face_down_grey_defender() {
    let (events, state) = explore(TileId::Countryside(3), Hex::new(4, -2), KEEP_POSITION);
    let grey = Sandbox::default().state().enemy_pile(EnemyColor::Grey)[0];
    let placed = Event::DefenderPlaced {
        token: grey,
        hex: KEEP,
    };
    assert_events_in_order(
        &events,
        &[
            Event::TileRevealed {
                tile: TileId::Countryside(3),
                position: KEEP_POSITION,
            },
            placed.clone(),
        ],
    );
    assert_eq!(state.enemy_tokens_at(KEEP), [grey]);
    // Information cachée : la vue et l'événement filtré ne donnent que la couleur.
    let seen = view(&state, SOLO);
    assert_eq!(
        seen.face_down_defenders.into_iter().collect::<Vec<_>>(),
        [(KEEP, vec![EnemyColor::Grey])]
    );
    assert!(!seen.enemy_tokens.contains_key(&KEEP));
    assert_eq!(
        filter_event(&placed, SOLO),
        PlayerEvent::DefenderPlaced {
            color: EnemyColor::Grey,
            hex: KEEP,
        }
    );
}

// C2 — Carte Mage Tower, *When Revealed* : un jeton violet face cachée sur la tour de mage.
#[test]
fn revealed_mage_tower_gets_a_violet_defender() {
    // La campagne 4 explorée en A + B depuis le village : sa tour de mage est au centre.
    let tower = Hex::new(4, -5);
    let (events, state) = explore(TileId::Countryside(4), VILLAGE, MONASTERY_POSITION);
    let violet = Sandbox::default().state().enemy_pile(EnemyColor::Violet)[0];
    assert!(events.contains(&Event::DefenderPlaced {
        token: violet,
        hex: tower,
    }));
    assert_eq!(
        view(&state, SOLO).face_down_defenders.get(&tower),
        Some(&vec![EnemyColor::Violet])
    );
}

// C3 — Cartes de site : les sites fortifiés des tuiles de la mise en place ont leur défenseur.
#[test]
fn setup_fortified_sites_get_defenders() {
    let (events, state) = (0..200)
        .map(|seed| {
            let config = GameConfig {
                seed,
                scenario: shuffled_countryside(),
                hero: Hero::Tovak,
                dummy_hero: Default::default(),
            };
            let events = new_game(&config, &mut config.rngs());
            let state = events.iter().fold(GameState::default(), apply);
            (events, state)
        })
        .find(|(events, _)| {
            events.iter().any(|event| {
                matches!(
                    event,
                    Event::TileRevealed {
                        tile: TileId::Countryside(3),
                        ..
                    }
                )
            })
        })
        .expect("une graine pose la campagne 3 à la mise en place");
    let keep = state
        .map()
        .tiles()
        .find(|(_, tile)| *tile == TileId::Countryside(3))
        .map(|(position, _)| position.hexes()[2])
        .expect("campagne 3 posée");
    let defender = events.iter().find_map(|event| match event {
        Event::DefenderPlaced { token, hex } if *hex == keep => Some(*token),
        _ => None,
    });
    assert_eq!(defender.map(|token| token.color()), Some(EnemyColor::Grey));
    assert_eq!(
        view(&state, SOLO).face_down_defenders.get(&keep),
        Some(&vec![EnemyColor::Grey])
    );
}

// --- Retourner les défenseurs ---

fn move_to(hex: Hex) -> Action {
    Action::MoveHero { to: hex }
}

/// Le décor `Fortified`, une Marche jouée (Mouvement 2).
fn marching(decor: Fortified) -> GameState {
    done(&decor.state(), basic(card(March, 1)))
}

/// Deck et main vidés, puis l'annonce de fin de manche : ses événements et l'état après.
/// Un tour déjà commencé est d'abord fini (sans défausse).
fn new_round(state: &GameState) -> (Vec<Event>, GameState) {
    let mut state = state.clone();
    if !state.play_area(SOLO).is_empty() {
        let end = arcanum_core::legal_actions(&state, SOLO)
            .into_iter()
            .find(|action| matches!(action, Action::EndTurn { .. }))
            .expect("finir le tour est légal");
        state = done(&state, end);
    }
    let state = apply(
        state,
        &Event::DeckShuffled {
            player: SOLO,
            order: Vec::new(),
        },
    );
    let events = events_of(&state, &Action::AnnounceEndOfRound);
    let after = events.iter().fold(state, apply);
    (events, after)
}

// C4 — Carte Keep ; Compendium, *Movement* S8 : de jour, un pas à côté retourne le défenseur.
#[test]
fn day_step_next_to_keep_reveals_defender() {
    let state = marching(Fortified::default());
    let events = events_of(&state, &move_to(NEAR_KEEP));
    let revealed = Event::EnemyTokenRevealed {
        hex: KEEP,
        token: guardsmen(),
    };
    assert_events_in_order(
        &events,
        &[
            Event::HeroMoved {
                player: SOLO,
                from: TWO_FROM_KEEP,
                to: NEAR_KEEP,
                cost: 2,
            },
            revealed,
        ],
    );
    let seen = view(&events.iter().fold(state, apply), SOLO);
    assert_eq!(seen.enemy_tokens.get(&KEEP), Some(&vec![guardsmen()]));
    assert!(!seen.face_down_defenders.contains_key(&KEEP));
}

// C5 — Compendium, *Miscellaneous* S10 : de nuit, rien n'est retourné en passant.
#[test]
fn night_step_reveals_nothing() {
    let state = marching(Fortified {
        time: TimeOfDay::Night,
        ..Fortified::default()
    });
    let events = events_of(&state, &move_to(NEAR_KEEP));
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::EnemyTokenRevealed { .. }))
    );
    let seen = view(&events.iter().fold(state, apply), SOLO);
    assert_eq!(
        seen.face_down_defenders.get(&KEEP),
        Some(&vec![EnemyColor::Grey])
    );
}

// C6 — Carte Keep (lecture « est voisine », Q4) : une exploration depuis une case voisine du site
// retourne aussitôt son défenseur.
#[test]
fn day_exploration_next_to_site_reveals_defender() {
    // Une tuile déjà posée en 3A : sa case S-O (8, -5) touche la forteresse de la campagne 3 en 2A.
    let figure = Hex::new(8, -5);
    let mut events = Sandbox {
        figure: Some(figure),
        hand: vec![card(March, 1)],
        next: vec![TileId::Countryside(8), TileId::Countryside(3)],
        ..Sandbox::default()
    }
    .events();
    events.push(Event::TileRevealed {
        tile: TileId::Countryside(8),
        position: TilePosition::from_lattice(3, 0),
    });
    let state = events.iter().fold(GameState::default(), apply);
    let state = done(&state, basic(card(March, 1)));
    let explored = events_of(
        &state,
        &Action::Explore {
            position: KEEP_POSITION,
        },
    );
    let grey = state.enemy_pile(EnemyColor::Grey)[0];
    assert_events_in_order(
        &explored,
        &[
            Event::DefenderPlaced {
                token: grey,
                hex: KEEP,
            },
            Event::EnemyTokenRevealed {
                hex: KEEP,
                token: grey,
            },
        ],
    );
}

// C7 — Compendium, *End of Round* S6 : une manche de Jour retourne les défenseurs voisins.
#[test]
fn day_round_reveals_adjacent_defenders() {
    let state = Fortified {
        figure: NEAR_KEEP,
        time: TimeOfDay::Night,
        ..Fortified::default()
    }
    .state();
    let (events, after) = new_round(&state);
    let started = events
        .iter()
        .position(|event| {
            matches!(
                event,
                Event::RoundStarted {
                    time_of_day: TimeOfDay::Day,
                    ..
                }
            )
        })
        .expect("une manche de Jour commence");
    let revealed = events
        .iter()
        .position(|event| {
            *event
                == Event::EnemyTokenRevealed {
                    hex: KEEP,
                    token: guardsmen(),
                }
        })
        .expect("le défenseur voisin est retourné");
    assert!(started < revealed);
    assert!(!view(&after, SOLO).face_down_defenders.contains_key(&KEEP));
}

// C8 — Compendium, *Movement* S8 : un défenseur retourné le reste la nuit suivante.
#[test]
fn revealed_defender_stays_face_up() {
    let state = marching(Fortified::default());
    let state = done(&state, move_to(NEAR_KEEP));
    let (_, night) = new_round(&state);
    assert_eq!(night.time_of_day(), TimeOfDay::Night);
    let seen = view(&night, SOLO);
    assert_eq!(seen.enemy_tokens.get(&KEEP), Some(&vec![guardsmen()]));
    assert!(!seen.face_down_defenders.contains_key(&KEEP));
}

// C9 — MKUE p. 7, 4.c : un défenseur n'est ni provoqué, ni défié, et un trajet passe à côté.
#[test]
fn defenders_are_not_rampaging() {
    let decor = Fortified {
        figure: NEAR_KEEP,
        keep: Some((guardsmen(), true)),
        ..Fortified::default()
    };
    let state = [card(March, 1), card(March, 2), card(Stamina, 1)]
        .into_iter()
        .fold(decor.state(), |state, card| done(&state, basic(card)));
    let legal = arcanum_core::legal_actions(&state, SOLO);
    assert!(!legal.contains(&challenge(KEEP)), "{legal:?}");
    // Centre de la campagne 3 (forêt), voisin de la forteresse comme la case de départ.
    let step = events_of(&state, &move_to(Hex::new(6, -4)));
    assert!(
        !step
            .iter()
            .any(|event| matches!(event, Event::EnemiesProvoked { .. })),
        "{step:?}"
    );
    // La colline E de la campagne 3, par la forêt, à côté de la forteresse.
    assert!(
        legal.contains(&Action::FollowRoute {
            to: Hex::new(7, -4)
        }),
        "{legal:?}"
    );
}

// C10 — Spec 024 : on ne revient pas en arrière avant un défenseur posé ou retourné.
#[test]
fn revealing_a_defender_blocks_undo() {
    assert!(
        Event::DefenderPlaced {
            token: guardsmen(),
            hex: KEEP,
        }
        .blocks_undo()
    );
    assert!(
        Event::EnemyTokenRevealed {
            hex: KEEP,
            token: guardsmen(),
        }
        .blocks_undo()
    );
    let (mut runner, puppet) = runner(Fortified::default().events());
    play(&mut runner, &puppet, basic(card(March, 1)));
    play(&mut runner, &puppet, move_to(NEAR_KEEP));
    assert_eq!(
        runner.undo_status(),
        arcanum_core::UndoStatus::InformationRevealed
    );
}

// --- Assaut ---

/// Le décor, ces cartes jouées pour leur effet de base.
fn playing(decor: Fortified, cards: &[CardId]) -> GameState {
    cards
        .iter()
        .fold(decor.state(), |state, &card| done(&state, basic(card)))
}

fn assault_events(site: Hex, from: Hex) -> [Event; 2] {
    [
        Event::AssaultStarted {
            player: SOLO,
            site,
            from,
        },
        Event::ReputationChanged {
            player: SOLO,
            change: -1,
        },
    ]
}

fn face_up_keep() -> Option<(EnemyToken, bool)> {
    Some((guardsmen(), true))
}

// C11 — MKUE p. 7, 4.a ; p. 8, 1.a ; p. 6, 6.b et 6.e : entrer dans la forteresse est un assaut.
#[test]
fn stepping_into_a_keep_is_an_assault() {
    let decor = Fortified {
        figure: NEAR_KEEP,
        keep: face_up_keep(),
        ..Fortified::default()
    };
    // Marche, Endurance (Mouvement 4), Promesse (Influence 2).
    let state = playing(decor, &[card(March, 1), card(Stamina, 1), card(Promise, 1)]);
    let events = events_of(&state, &move_to(KEEP));
    let [started, reputation] = assault_events(KEEP, NEAR_KEEP);
    assert_events_in_order(
        &events,
        &[
            Event::HeroMoved {
                player: SOLO,
                from: NEAR_KEEP,
                to: KEEP,
                cost: 3,
            },
            started,
            reputation,
        ],
    );
    let after = events.iter().fold(state, apply);
    assert_eq!(after.move_points(SOLO), 0);
    assert_eq!(after.influence_points(SOLO), 0);
    assert_eq!(
        after.assault(SOLO),
        Some(arcanum_core::Assault {
            site: KEEP,
            from: NEAR_KEEP,
        })
    );
    assert_eq!(after.reputation(SOLO).value(), -1);
    let legal = arcanum_core::legal_actions(&after, SOLO);
    assert!(legal.contains(&Action::StartCombat), "{legal:?}");
    assert!(
        legal
            .iter()
            .all(|action| matches!(action, Action::StartCombat | Action::Challenge { .. })),
        "{legal:?}"
    );
}

// C12 — Compendium, *Movement* S4 ; spec 062 : l'assaut coûte le terrain, les cités restent
// fermées.
#[test]
fn assault_costs_the_terrain_and_cities_stay_closed() {
    let decor = Fortified {
        figure: NEAR_KEEP,
        keep: face_up_keep(),
        ..Fortified::default()
    };
    // Mouvement 2 : la colline de la forteresse en coûte 3.
    let state = playing(decor.clone(), &[card(March, 1)]);
    refused(&state, move_to(KEEP));
    let state = playing(decor, &[card(March, 1), card(Stamina, 1)]);
    assert!(arcanum_core::legal_actions(&state, SOLO).contains(&move_to(KEEP)));
    // Cité au centre de la tuile royaume 5 posée en 2A, depuis son côté O.
    let mut events = Sandbox {
        figure: Some(TWO_FROM_KEEP),
        hand: vec![card(March, 1), card(March, 2)],
        next: vec![TileId::Core(5)],
        ..Sandbox::default()
    }
    .events();
    events.push(Event::TileRevealed {
        tile: TileId::Core(5),
        position: KEEP_POSITION,
    });
    let state = events.iter().fold(GameState::default(), apply);
    let state = done(&done(&state, basic(card(March, 1))), basic(card(March, 2)));
    refused(&state, move_to(Hex::new(6, -4)));
}

// L1 — Pas de point de mouvement, pas d'assaut : une main de Blessures sans carte jouée.
#[test]
fn assault_needs_move_points() {
    let state = Fortified {
        figure: NEAR_KEEP,
        keep: face_up_keep(),
        hand: vec![wound(1), wound(2)],
        ..Fortified::default()
    }
    .state();
    let legal = arcanum_core::legal_actions(&state, SOLO);
    assert!(!legal.contains(&move_to(KEEP)), "{legal:?}");
}

// C13 — Q2 ; spec 036 : un trajet peut finir par un assaut, jamais traverser un site non conquis.
#[test]
fn route_can_end_in_an_assault() {
    let decor = Fortified {
        keep: face_up_keep(),
        ..Fortified::default()
    };
    let state = playing(decor, &[card(March, 1), card(March, 2), card(Stamina, 1)]);
    let legal = arcanum_core::legal_actions(&state, SOLO);
    let route = Action::FollowRoute { to: KEEP };
    assert!(legal.contains(&route), "{legal:?}");
    let events = events_of(&state, &route);
    let [started, reputation] = assault_events(KEEP, NEAR_KEEP);
    assert_events_in_order(
        &events,
        &[
            Event::HeroMoved {
                player: SOLO,
                from: TWO_FROM_KEEP,
                to: NEAR_KEEP,
                cost: 2,
            },
            Event::HeroMoved {
                player: SOLO,
                from: NEAR_KEEP,
                to: KEEP,
                cost: 3,
            },
            started,
            reputation,
        ],
    );
    for action in legal {
        if let Action::FollowRoute { to } = action
            && to != KEEP
        {
            assert!(
                !events_of(&state, &action).iter().any(|event| matches!(
                    event,
                    Event::HeroMoved { to: KEEP, .. } | Event::AssaultStarted { .. }
                )),
                "{to:?}"
            );
        }
    }
}

// C13b — Compendium, *Movement* S13 : de jour, le trajet s'arrête sur le pas qui retourne le
// défenseur visé ; de nuit, il va jusqu'à l'assaut.
#[test]
fn route_stops_when_defender_is_revealed() {
    let hand = [card(March, 1), card(March, 2), card(Stamina, 1)];
    let route = Action::FollowRoute { to: KEEP };
    let state = playing(Fortified::default(), &hand);
    let events = events_of(&state, &route);
    assert!(events.contains(&Event::EnemyTokenRevealed {
        hex: KEEP,
        token: guardsmen(),
    }));
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::AssaultStarted { .. })),
        "{events:?}"
    );
    assert_eq!(
        events.iter().fold(state, apply).figure(SOLO),
        Some(NEAR_KEEP)
    );
    let night = playing(
        Fortified {
            time: TimeOfDay::Night,
            ..Fortified::default()
        },
        &hand,
    );
    let events = events_of(&night, &route);
    assert_events_in_order(&events, &assault_events(KEEP, NEAR_KEEP));
}

// C14 — Compendium, *Influence* S5 ; spec 040 : l'assaut reste permis sur la case X (−7).
#[test]
fn assault_works_on_x() {
    let decor = Fortified {
        figure: NEAR_KEEP,
        keep: face_up_keep(),
        ..Fortified::default()
    };
    let state = apply(
        playing(decor, &[card(March, 1), card(Stamina, 1)]),
        &Event::ReputationChanged {
            player: SOLO,
            change: -7,
        },
    );
    assert!(state.reputation(SOLO).is_x());
    let after = done(&state, move_to(KEEP));
    assert_eq!(after.reputation(SOLO).value(), -7);
    assert!(after.assault(SOLO).is_some());
}

/// La tour de mage, Mages de feu face visible ou non, figurine sur `figure`, les Rôdeurs sur les
/// orques de la campagne 4 ; Marche, Marche et Endurance jouées (Mouvement 6).
fn tower(figure: Hex, time: TimeOfDay, face_up: bool) -> GameState {
    playing(
        Fortified {
            figure,
            time,
            keep: None,
            tower: Some((fire_mages(), face_up)),
            placed: vec![(prowlers(), TOWER_ORCS)],
            ..Fortified::default()
        },
        &[card(March, 1), card(March, 2), card(Stamina, 1)],
    )
}

fn fire_mages() -> EnemyToken {
    token(arcanum_core::Enemy::FireMages, 1)
}

// C17 — Compendium, *Combat* Q1.a et S14 : de nuit, on défie avant de voir le défenseur, retourné
// à l'engagement.
#[test]
fn night_assault_reveals_defender_at_engagement() {
    // Désert N-O de la campagne 4 : voisin de la tour, pas des orques.
    let state = done(
        &tower(Hex::new(2, -7), TimeOfDay::Night, false),
        move_to(MAGE_TOWER),
    );
    assert!(state.is_face_down(MAGE_TOWER));
    let state = done(&state, challenge(TOWER_ORCS));
    assert!(state.is_face_down(MAGE_TOWER));
    let events = events_of(&state, &Action::StartCombat);
    assert_eq!(
        events,
        [
            Event::EnemyTokenRevealed {
                hex: MAGE_TOWER,
                token: fire_mages(),
            },
            Event::CombatStarted {
                player: SOLO,
                enemies: vec![(MAGE_TOWER, fire_mages()), (TOWER_ORCS, prowlers())],
            },
        ]
    );
}

// L7 — Un défenseur déjà retourné ne l'est pas une seconde fois à l'engagement.
#[test]
fn face_up_defender_is_not_revealed_again() {
    let state = done(
        &tower(Hex::new(2, -7), TimeOfDay::Day, true),
        move_to(MAGE_TOWER),
    );
    assert_eq!(
        events_of(&state, &Action::StartCombat),
        [Event::CombatStarted {
            player: SOLO,
            enemies: vec![(MAGE_TOWER, fire_mages())],
        }]
    );
}

// C18 — MKUE p. 8, 2.b : le pas de l'assaut provoque les ennemis errants voisins des deux cases ;
// ils combattent avec le défenseur.
#[test]
fn provoked_rampaging_enemies_join_unfortified() {
    let state = tower(NEAR_TOWER, TimeOfDay::Day, true);
    let events = events_of(&state, &move_to(MAGE_TOWER));
    let [started, reputation] = assault_events(MAGE_TOWER, NEAR_TOWER);
    assert_events_in_order(
        &events,
        &[
            started,
            reputation,
            Event::EnemiesProvoked {
                player: SOLO,
                enemies: vec![TOWER_ORCS],
            },
        ],
    );
    let state = events.iter().fold(state, apply);
    assert_eq!(
        events_of(&state, &Action::StartCombat),
        [Event::CombatStarted {
            player: SOLO,
            enemies: vec![(MAGE_TOWER, fire_mages()), (TOWER_ORCS, prowlers())],
        }]
    );
}

// --- Fortification du site ---

fn attack(kind: arcanum_core::AttackKind) -> arcanum_core::Effect {
    arcanum_core::Effect::Attack {
        amount: 3,
        element: arcanum_core::Element::Physical,
        kind,
    }
}

/// Assaut de la forteresse depuis sa voisine, défenseur `defender` face visible, les jetons
/// `placed` posés : Marche et Endurance jouées, le pas, le combat engagé.
fn assaulting(defender: EnemyToken, placed: &[(EnemyToken, Hex)], hand: &[CardId]) -> GameState {
    let decor = Fortified {
        figure: NEAR_KEEP,
        keep: Some((defender, true)),
        placed: placed.to_vec(),
        hand: hand.to_vec(),
        owned: vec![arcanum_core::UnitCard::new(arcanum_core::Unit::Peasants, 1)],
        ..Fortified::default()
    };
    let state = playing(decor, &[card(March, 1), card(Stamina, 1)]);
    let state = done(&state, move_to(KEEP));
    done(&state, Action::StartCombat)
}

fn default_hand() -> Vec<CardId> {
    Fortified::default().hand
}

fn swordsmen() -> EnemyToken {
    token(arcanum_core::Enemy::Swordsmen, 1)
}

// C15 — MKUE p. 9, 3.a : le défenseur est fortifié par son site, le siège seul le vise à distance.
#[test]
fn defender_is_fortified_by_its_site() {
    let state = assaulting(swordsmen(), &[], &default_hand());
    let combat = state.combat(SOLO).expect("combat engagé");
    assert!(combat.enemies[0].site_fortified);
    assert_eq!(combat.enemies[0].fortification(), 1);
    let state = done(&state, declare(&[foe(&state, KEEP)]));
    let combat = state.combat(SOLO).expect("combat engagé");
    assert!(!combat.accepts(attack(arcanum_core::AttackKind::Ranged)));
    assert!(combat.accepts(attack(arcanum_core::AttackKind::Siege)));
}

// C16 — MKUE p. 9, 3.a ; p. 10, 1.a ; Compendium, *Combat* S13 : des Gardes (Fortifié) sur leur
// site sont doublement fortifiés : aucune cible à distance, seuls ou en groupe ; en phase
// d'attaque, si. C18 : l'ennemi errant du même combat n'est pas fortifié.
#[test]
fn fortified_defender_is_fortified_twice() {
    // Les Rôdeurs au centre de la campagne 3, voisins de la forteresse et de sa voisine N-O.
    let forest = Hex::new(6, -4);
    let state = assaulting(guardsmen(), &[(prowlers(), forest)], &default_hand());
    let combat = state.combat(SOLO).expect("combat engagé");
    assert_eq!(combat.enemies.len(), 2);
    assert_eq!(combat.enemies[0].fortification(), 2);
    assert_eq!(attack_targets(&state), [vec![forest]]);
    let aimed = done(&state, declare(&[foe(&state, forest)]));
    assert!(
        aimed
            .combat(SOLO)
            .expect("combat engagé")
            .accepts(attack(arcanum_core::AttackKind::Ranged))
    );
    let attack_phase = skip_to_attack_phase(&state);
    assert_eq!(
        attack_targets(&attack_phase),
        [vec![KEEP], vec![forest], vec![KEEP, forest]]
    );
}

// C19 — Spec 042 : les unités combattent pendant un assaut.
#[test]
fn units_fight_in_assaults() {
    let state = to_block_phase(&assaulting(swordsmen(), &[], &default_hand()));
    let state = done(&state, declare_block(foe(&state, KEEP)));
    let legal = arcanum_core::legal_actions(&state, SOLO);
    assert!(
        legal
            .iter()
            .any(|action| matches!(action, Action::ActivateUnit { .. })),
        "{legal:?}"
    );
}

// C20 — Compendium, *Combat* S36 : la gloire entière du défenseur.
#[test]
fn defender_gives_full_fame() {
    let crossbowmen = token(arcanum_core::Enemy::Crossbowmen, 1);
    let hand = [
        card(March, 1),
        card(Stamina, 1),
        card(Rage, 1),
        card(Promise, 1),
        card(Threaten, 1),
    ];
    let state = skip_to_attack_phase(&assaulting(crossbowmen, &[], &hand));
    let state = done(&state, declare(&[foe(&state, KEEP)]));
    let state = done(&state, option(card(Rage, 1), 0));
    let state = done(&state, sideways_attack(card(Promise, 1)));
    let state = done(&state, sideways_attack(card(Threaten, 1)));
    let events = events_of(&state, &Action::ResolveAttack);
    assert!(
        events.contains(&Event::EnemiesDefeated {
            player: SOLO,
            enemies: vec![foe(&state, KEEP)],
            fame: 3,
        }),
        "{events:?}"
    );
}

// --- Issue ---

fn crossbowmen() -> EnemyToken {
    token(arcanum_core::Enemy::Crossbowmen, 1)
}

/// Main de l'assaut contre les Arbalétriers (armure 4) : Marche et Endurance pour le pas, Rage
/// (Attaque 2) et deux cartes de côté pour l'attaque.
fn assault_hand() -> Vec<CardId> {
    vec![
        card(March, 1),
        card(Stamina, 1),
        card(Rage, 1),
        card(Promise, 1),
        card(Threaten, 1),
    ]
}

/// En phase d'attaque, les Arbalétriers du site vaincus (Attaque 4).
fn defeat_keep(state: &GameState) -> GameState {
    let state = done(&skip_to_attack_phase(state), declare(&[foe(state, KEEP)]));
    let state = done(&state, option(card(Rage, 1), 0));
    let state = done(&state, sideways_attack(card(Promise, 1)));
    let state = done(&state, sideways_attack(card(Threaten, 1)));
    done(&state, Action::ResolveAttack)
}

// C21 — MKUE p. 10, 2.c ; Compendium, *Movement* S11 : le défenseur vaincu, le site est conquis.
#[test]
fn victory_conquers_the_keep() {
    let state = defeat_keep(&assaulting(crossbowmen(), &[], &assault_hand()));
    let events = events_of(&state, &Action::EndCombat);
    assert_eq!(
        events,
        [
            Event::CombatEnded { player: SOLO },
            Event::SiteConquered {
                player: SOLO,
                hex: KEEP,
            },
        ]
    );
    let after = events.iter().fold(state, apply);
    assert_eq!(after.shields().get(&KEEP), Some(&SOLO));
    assert_eq!(after.conquered_sites(), [KEEP]);
    assert_eq!(after.figure(SOLO), Some(KEEP));
    let legal = arcanum_core::legal_actions(&after, SOLO);
    assert!(
        !legal.iter().any(|action| matches!(
            action,
            Action::MoveHero { .. }
                | Action::FollowRoute { .. }
                | Action::Explore { .. }
                | Action::Interact
        )),
        "{legal:?}"
    );
}

// C22 — MKUE p. 10, 2.c, note ; Compendium, *Cities* S3 : la conquête ne dépend pas des ennemis
// errants du combat.
#[test]
fn conquest_ignores_rampaging_enemies() {
    let forest = Hex::new(6, -4);
    let state = defeat_keep(&assaulting(
        crossbowmen(),
        &[(prowlers(), forest)],
        &assault_hand(),
    ));
    let after = done(&state, Action::EndCombat);
    assert_eq!(after.shields().get(&KEEP), Some(&SOLO));
    assert_eq!(after.enemy_tokens_at(forest), [prowlers()]);
}

// C23 — MKUE p. 10, 3.d ; Compendium, *Forced Withdrawal* S2 et S3 : le défenseur non vaincu, le
// héros bat en retraite sans coût ni Blessure ; C24 (*Combat* S19) : sans provoquer.
#[test]
fn failed_assault_withdraws() {
    let forest = Hex::new(6, -4);
    let state = skip_to_attack_phase(&assaulting(
        crossbowmen(),
        &[(prowlers(), forest)],
        &assault_hand(),
    ));
    let events = events_of(&state, &Action::EndCombat);
    assert_eq!(
        events,
        [
            Event::CombatEnded { player: SOLO },
            Event::HeroWithdrew {
                player: SOLO,
                from: KEEP,
                to: NEAR_KEEP,
            },
        ]
    );
    let after = events.iter().fold(state.clone(), apply);
    assert_eq!(after.figure(SOLO), Some(NEAR_KEEP));
    assert_eq!(after.hand(SOLO), state.hand(SOLO));
    assert_eq!(after.enemy_tokens_at(KEEP), [crossbowmen()]);
    assert!(after.shields().is_empty());
}

// L2 — Spec 034 : assommé pendant l'assaut, vaincre le défenseur conquiert quand même le site.
#[test]
fn knocked_out_hero_still_conquers() {
    let state = apply(
        assaulting(crossbowmen(), &[], &assault_hand()),
        &Event::KnockedOut {
            player: SOLO,
            cards: Vec::new(),
        },
    );
    let after = done(&defeat_keep(&state), Action::EndCombat);
    assert_eq!(after.shields().get(&KEEP), Some(&SOLO));
}

// L4 — Plus aucun jeton pour le défenseur : entrer conquiert le site sans combat, Réputation −1.
#[test]
fn undefended_site_is_conquered_on_entry() {
    let decor = Fortified {
        figure: NEAR_KEEP,
        keep: None,
        ..Fortified::default()
    };
    let state = playing(decor, &[card(March, 1), card(Stamina, 1)]);
    let events = events_of(&state, &move_to(KEEP));
    let [started, reputation] = assault_events(KEEP, NEAR_KEEP);
    assert_events_in_order(
        &events,
        &[
            started,
            reputation,
            Event::SiteConquered {
                player: SOLO,
                hex: KEEP,
            },
        ],
    );
}

// C25 — MKUE p. 7, 4.a : un site conquis est une case ordinaire, qu'on traverse.
#[test]
fn conquered_site_is_an_ordinary_space() {
    let decor = Fortified {
        figure: NEAR_KEEP,
        keep: None,
        tower: None,
        conquered: vec![KEEP, MAGE_TOWER],
        ..Fortified::default()
    };
    let state = playing(
        decor.clone(),
        &[card(March, 1), card(March, 2), card(Stamina, 1)],
    );
    assert_eq!(
        events_of(&state, &move_to(KEEP)),
        [Event::HeroMoved {
            player: SOLO,
            from: NEAR_KEEP,
            to: KEEP,
            cost: 3,
        }]
    );
    let inside = done(&state, move_to(KEEP));
    assert!(arcanum_core::legal_actions(&inside, SOLO).contains(&move_to(Hex::new(7, -4))));
    // De nuit, le plus court chemin de la plaine S-E de la campagne 4 à son désert N-O passe par
    // la tour conquise (3 + 3).
    let night = playing(
        Fortified {
            figure: NEAR_TOWER,
            time: TimeOfDay::Night,
            ..decor
        },
        &[card(March, 1), card(March, 2), card(Stamina, 1)],
    );
    let events = events_of(
        &night,
        &Action::FollowRoute {
            to: Hex::new(2, -7),
        },
    );
    assert!(
        events.contains(&Event::HeroMoved {
            player: SOLO,
            from: NEAR_TOWER,
            to: MAGE_TOWER,
            cost: 3,
        }),
        "{events:?}"
    );
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::AssaultStarted { .. }))
    );
}

// --- Site conquis ---

fn spell(spell: arcanum_core::Spell) -> CardId {
    CardId::new(arcanum_core::DeedCard::Spell(spell), 1)
}

fn fireball() -> CardId {
    spell(arcanum_core::Spell::Fireball)
}

fn snowstorm() -> CardId {
    spell(arcanum_core::Spell::Snowstorm)
}

fn expose() -> CardId {
    spell(arcanum_core::Spell::Expose)
}

fn unit(unit: arcanum_core::Unit) -> arcanum_core::UnitCard {
    arcanum_core::UnitCard::new(unit, 1)
}

fn with_crystal(state: GameState, color: arcanum_core::BasicColor) -> GameState {
    apply(
        state,
        &Event::CrystalGained {
            player: SOLO,
            color,
        },
    )
}

/// La forteresse conquise, la figurine dessus (ou à côté), sans défenseur.
fn own_keep(figure: Hex) -> Fortified {
    Fortified {
        figure,
        keep: None,
        conquered: vec![KEEP],
        ..Fortified::default()
    }
}

/// La tour de mage conquise, la figurine dessus, l'offre Éclair de feu, Tempête de neige,
/// Exposition ; l'interaction commencée, `influence` points.
fn at_conquered_tower(
    time: TimeOfDay,
    dice: Vec<arcanum_core::ManaColor>,
    influence: u8,
) -> GameState {
    let decor = Fortified {
        figure: MAGE_TOWER,
        time,
        dice,
        tower: None,
        conquered: vec![MAGE_TOWER],
        spells: vec![fireball(), snowstorm(), expose()],
        ..Fortified::default()
    };
    let state = done(&decor.state(), Action::Interact);
    with_influence(&state, influence)
}

/// Les moyens de paiement des `BuySpell` légaux pour ce sort.
fn spell_payments(state: &GameState, spell: CardId) -> Vec<arcanum_core::ManaPayment> {
    arcanum_core::legal_actions(state, SOLO)
        .into_iter()
        .filter_map(|action| match action {
            Action::BuySpell { card, mana } if card == spell => Some(mana),
            _ => None,
        })
        .collect()
}

fn recruits(state: &GameState) -> Vec<Action> {
    arcanum_core::legal_actions(state, SOLO)
        .into_iter()
        .filter(|action| matches!(action, Action::RecruitUnit { .. }))
        .collect()
}

fn recruit(unit: arcanum_core::UnitCard) -> Action {
    Action::RecruitUnit {
        unit,
        disband: None,
        bonds: false,
    }
}

// C26 — Carte Keep ; MKUE p. 8, 4.a : dans sa forteresse, on recrute les unités de forteresse.
#[test]
fn own_keep_recruits_keep_units() {
    use arcanum_core::Unit::*;
    let state = Fortified {
        offer: vec![unit(Peasants), unit(UtemSwordsmen), unit(UtemCrossbowmen)],
        ..own_keep(KEEP)
    }
    .state();
    assert!(arcanum_core::legal_actions(&state, SOLO).contains(&Action::Interact));
    let state = with_influence(&done(&state, Action::Interact), 9);
    assert_eq!(
        recruits(&state),
        [recruit(unit(UtemSwordsmen)), recruit(unit(UtemCrossbowmen))]
    );
}

// L8 — Spec 042 : sans jeton de commandement libre, recruter renvoie une unité, à la forteresse
// aussi.
#[test]
fn keep_recruit_with_full_command_disbands() {
    use arcanum_core::Unit::*;
    let state = Fortified {
        owned: vec![unit(Peasants)],
        offer: vec![unit(UtemSwordsmen)],
        ..own_keep(KEEP)
    }
    .state();
    let state = with_influence(&done(&state, Action::Interact), 6);
    assert_eq!(
        recruits(&state),
        [Action::RecruitUnit {
            unit: unit(UtemSwordsmen),
            disband: Some(unit(Peasants)),
            bonds: false,
        }]
    );
}

// C27 — MKUE p. 8, 4.d ; Compendium, *Miscellaneous* S7 : la tour vend un sort, l'offre est
// complétée aussitôt.
#[test]
fn mage_tower_sells_spells() {
    let state = with_crystal(
        at_conquered_tower(TimeOfDay::Day, Fortified::default().dice, 7),
        arcanum_core::BasicColor::Red,
    );
    let next = state.common_deck(arcanum_core::CommonDeck::Spells)[0];
    let buy = Action::BuySpell {
        card: fireball(),
        mana: arcanum_core::ManaPayment::Crystal,
    };
    let events = events_of(&state, &buy);
    assert_eq!(
        events,
        [
            Event::SpellBought {
                player: SOLO,
                card: fireball(),
                cost: 7,
            },
            Event::CrystalUsed {
                player: SOLO,
                color: arcanum_core::BasicColor::Red,
            },
            Event::OfferCardDealt {
                offer: arcanum_core::Offer::Spells,
                card: next,
            },
        ]
    );
    let after = events.iter().fold(state, apply);
    assert_eq!(after.deck(SOLO).first(), Some(&fireball()));
    assert_eq!(after.offer(arcanum_core::Offer::Spells).len(), 3);
    assert_eq!(after.influence_points(SOLO), 0);
}

// C28 — FAQ p3 ; Compendium, *Mana* S4 ; spec 022 : un mana de la couleur du sort, l'or de jour.
#[test]
fn spell_purchase_needs_matching_mana() {
    use arcanum_core::{BasicColor::*, ManaColor, ManaPayment};
    let dice = vec![
        ManaColor::Basic(Blue),
        ManaColor::Basic(Green),
        ManaColor::Gold,
    ];
    let day = at_conquered_tower(TimeOfDay::Day, dice.clone(), 7);
    // Dé or, de jour : Éclair de feu (rouge) ; aucun dé rouge, aucun cristal.
    assert_eq!(spell_payments(&day, fireball()), [ManaPayment::Die(2)]);
    assert_eq!(
        spell_payments(&day, snowstorm()),
        [ManaPayment::Die(0), ManaPayment::Die(2)]
    );
    // L'or ne vaut rien la nuit.
    let night = at_conquered_tower(TimeOfDay::Night, dice.clone(), 7);
    assert!(spell_payments(&night, fireball()).is_empty());
    // Le dé du tour déjà pris.
    let taken = apply(
        day.clone(),
        &Event::DieTaken {
            player: SOLO,
            die: 1,
            color: ManaColor::Basic(Green),
        },
    );
    assert!(spell_payments(&taken, fireball()).is_empty());
    // 6 influence.
    let poor = at_conquered_tower(TimeOfDay::Day, dice, 6);
    assert!(spell_payments(&poor, fireball()).is_empty());
}

// L6 — Compendium, *Miscellaneous* S7 : deux sorts dans la même interaction.
#[test]
fn two_spells_in_one_interaction() {
    use arcanum_core::{BasicColor::*, ManaPayment};
    let state = at_conquered_tower(TimeOfDay::Day, Fortified::default().dice, 14);
    let state = with_crystal(with_crystal(state, Red), Blue);
    let state = done(
        &state,
        Action::BuySpell {
            card: fireball(),
            mana: ManaPayment::Crystal,
        },
    );
    let state = done(
        &state,
        Action::BuySpell {
            card: snowstorm(),
            mana: ManaPayment::Crystal,
        },
    );
    assert_eq!(&state.deck(SOLO)[..2], [snowstorm(), fireball()]);
    assert_eq!(state.offer(arcanum_core::Offer::Spells).len(), 3);
}

/// Les cartes piochées à la fin du tour, sans défausse, après une Marche jouée.
fn end_turn_draws(state: &GameState) -> usize {
    let state = done(state, basic(card(March, 1)));
    events_of(
        &state,
        &Action::EndTurn {
            discard: Vec::new(),
            reclaim: None,
            glade: None,
            steady_tempo: false,
        },
    )
    .iter()
    .filter(|event| matches!(event, Event::CardDrawn { .. }))
    .count()
}

// C29 — Carte Keep ; Compendium, *End of Turn* S8 : + le nombre de forteresses possédées, sur
// l'une d'elles ou à côté.
#[test]
fn owned_keeps_raise_hand_limit() {
    // Main de 4 après la Marche : limite 5 + 1.
    assert_eq!(end_turn_draws(&own_keep(KEEP).state()), 2);
    assert_eq!(end_turn_draws(&own_keep(NEAR_KEEP).state()), 2);
    assert_eq!(end_turn_draws(&own_keep(TWO_FROM_KEEP).state()), 1);
    // La campagne 10 en 2A + B : sa forteresse (côté S-O) est voisine de la plaine N-O de la
    // campagne 3, comme celle de la campagne 3. Deux possédées : + 2, pas + 4.
    let second = Hex::new(6, -6);
    let state = own_keep(NEAR_KEEP).state();
    let state = apply(
        state,
        &Event::TileRevealed {
            tile: TileId::Countryside(10),
            position: TilePosition::from_lattice(2, 1),
        },
    );
    let state = apply(
        state,
        &Event::SiteConquered {
            player: SOLO,
            hex: second,
        },
    );
    assert_eq!(state.conquered_sites().len(), 2);
    assert_eq!(end_turn_draws(&state), 3);
}

// C30 — MKUE p. 4, 2.g ; p. 9, 5.c : la nouvelle manche suit la limite augmentée, pas le seuil
// d'assommé.
#[test]
fn keep_bonus_at_new_round_not_for_knock_out() {
    // Sept cartes en main, deck vide : la manche suivante en mélange sept et en pioche 5 + 1.
    let state = apply(
        own_keep(KEEP).state(),
        &Event::DeckShuffled {
            player: SOLO,
            order: Vec::new(),
        },
    );
    let state = [
        March,
        Stamina,
        Rage,
        Promise,
        Swiftness,
        Determination,
        Concentration,
    ]
    .into_iter()
    .map(|action| card(action, 1))
    .fold(state, |state, card| {
        apply(state, &Event::CardDrawn { player: SOLO, card })
    });
    let events = events_of(&state, &Action::AnnounceEndOfRound);
    let drawn = events
        .iter()
        .filter(|event| matches!(event, Event::CardDrawn { .. }))
        .count();
    assert_eq!(drawn, 6);
    // Minotaure (Brutal, 5) : cinq Blessures, la limite du niveau, assomment à côté de la
    // forteresse aussi.
    let minotaur_hex = Hex::new(6, -4);
    let state = Fortified {
        placed: vec![(token(arcanum_core::Enemy::Minotaur, 1), minotaur_hex)],
        ..own_keep(NEAR_KEEP)
    }
    .state();
    let state = done(&state, challenge(minotaur_hex));
    let state = to_block_phase(&done(&state, Action::StartCombat));
    let events = events_of(&state, &Action::NextCombatPhase);
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::KnockedOut { .. }))
    );
}

// --- Récompense sort, décompte ---

/// Sur la tour de mage, conquise ce tour (sa récompense sort due), l'offre Éclair de feu, Tempête de
/// neige, Exposition ; une Marche jouée.
fn tower_conquered_this_turn() -> GameState {
    let decor = Fortified {
        figure: MAGE_TOWER,
        tower: None,
        spells: vec![fireball(), snowstorm(), expose()],
        ..Fortified::default()
    };
    let state = apply(
        decor.state(),
        &Event::SiteConquered {
            player: SOLO,
            hex: MAGE_TOWER,
        },
    );
    done(&state, basic(card(March, 1)))
}

fn end_turn() -> Action {
    Action::EndTurn {
        discard: Vec::new(),
        reclaim: None,
        glade: None,
        steady_tempo: false,
    }
}

fn take_spell(card: CardId) -> Action {
    Action::TakeSpell { card }
}

// C31 — MKUE p. 11, 5.c : la récompense de la tour, un sort de l'offre au choix, avant la pioche.
#[test]
fn mage_tower_reward_is_a_spell_from_the_offer() {
    let state = tower_conquered_this_turn();
    assert_eq!(state.spell_rewards(SOLO), 1);
    let events = events_of(&state, &end_turn());
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::CardDrawn { .. }))
    );
    let waiting = events.iter().fold(state, apply);
    assert_eq!(
        arcanum_core::legal_actions(&waiting, SOLO),
        [
            take_spell(fireball()),
            take_spell(snowstorm()),
            take_spell(expose())
        ]
    );
    let next = waiting.common_deck(arcanum_core::CommonDeck::Spells)[0];
    let events = events_of(&waiting, &take_spell(snowstorm()));
    assert_events_in_order(
        &events,
        &[
            Event::OfferCardGained {
                to: arcanum_core::CardDestination::DeckTop,
                player: SOLO,
                offer: arcanum_core::Offer::Spells,
                card: arcanum_core::CommonCard::Deed(snowstorm()),
            },
            Event::OfferCardDealt {
                offer: arcanum_core::Offer::Spells,
                card: next,
            },
            Event::CardDrawn {
                player: SOLO,
                card: snowstorm(),
            },
        ],
    );
    let after = events.iter().fold(waiting, apply);
    assert!(after.hand(SOLO).contains(&snowstorm()));
    assert_eq!(after.offer(arcanum_core::Offer::Spells).len(), 3);
    assert_eq!(after.spell_choices(SOLO), 0);
}

// C32 — MKUE p. 11, 5 et 6 : le sort d'abord, puis l'action avancée du niveau pair.
#[test]
fn spell_reward_comes_before_level_up() {
    let aa = arcanum_core::AdvancedAction::ALL.map(advanced);
    let state = apply(
        tower_conquered_this_turn(),
        &Event::CommonDeckShuffled {
            deck: arcanum_core::CommonDeck::AdvancedActions,
            order: aa
                .iter()
                .map(|&card| arcanum_core::CommonCard::Deed(card))
                .collect(),
        },
    );
    let state = apply(
        state,
        &Event::OfferCardDealt {
            offer: arcanum_core::Offer::AdvancedActions,
            card: arcanum_core::CommonCard::Deed(aa[0]),
        },
    );
    // Niveau 2 à 3 de gloire.
    let state = apply(
        state,
        &Event::EnemiesDefeated {
            player: SOLO,
            enemies: Vec::new(),
            fame: 3,
        },
    );
    let state = done(&state, end_turn());
    assert!(
        arcanum_core::legal_actions(&state, SOLO)
            .iter()
            .all(|action| matches!(action, Action::TakeSpell { .. }))
    );
    let events = events_of(&state, &take_spell(fireball()));
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::CardDrawn { .. }))
    );
    let state = events.iter().fold(state, apply);
    assert_eq!(
        arcanum_core::legal_actions(&state, SOLO),
        [Action::GainAdvancedAction { card: aa[0] }]
    );
}

// Issue privée 317 — MKUE p. 11, 5.c puis 6.b : les compétences du niveau pair ne sont révélées qu'après
// le choix du sort de la récompense, pas dès la fin du tour.
#[test]
fn skills_revealed_after_spell_reward() {
    use arcanum_core::Skill;
    let pile = vec![
        Skill::DarkPaths,
        Skill::BurningPower,
        Skill::HotSwordsmanship,
        Skill::DarkNegotiation,
    ];
    let state = apply(
        tower_conquered_this_turn(),
        &Event::SkillPileShuffled {
            player: SOLO,
            order: pile.clone(),
        },
    );
    // Niveau 2 à 3 de gloire.
    let state = apply(
        state,
        &Event::EnemiesDefeated {
            player: SOLO,
            enemies: Vec::new(),
            fame: 3,
        },
    );
    let events = events_of(&state, &end_turn());
    assert!(events.contains(&Event::LevelGained {
        player: SOLO,
        level: 2,
    }));
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::SkillsRevealed { .. }))
    );
    let state = events.iter().fold(state, apply);
    assert_eq!(state.skill_pile(SOLO), &pile[..]);
    let events = events_of(&state, &take_spell(fireball()));
    assert!(events.contains(&Event::SkillsRevealed {
        player: SOLO,
        skills: pile[..2].to_vec(),
    }));
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::CardDrawn { .. }))
    );
    let state = events.iter().fold(state, apply);
    assert_eq!(
        arcanum_core::legal_actions(&state, SOLO),
        [
            Action::ChooseSkill { skill: pile[0] },
            Action::ChooseSkill { skill: pile[1] }
        ]
    );
}

// C33 — MKUE p. 15 : Le Conquérant, 2 par Bouclier sur forteresse ou tour de mage.
#[test]
fn conqueror_counts_keeps_and_towers() {
    let state = Fortified {
        keep: None,
        tower: None,
        conquered: vec![KEEP, MAGE_TOWER],
        ..Fortified::default()
    }
    .state();
    let score = arcanum_core::final_score(&state, SOLO);
    assert_eq!(score.points(arcanum_core::Achievement::Conqueror), 4);
}
