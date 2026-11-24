//! Scénarios de la spec 062 (Cités).

use arcanum_core::{
    Action, BasicAction, BasicColor, CityLevels, CityRule, ContentError, ContentSources, Enemy,
    EnemyColor, EnemyToken, Event, GameContent, GameState, Hex, Scenario, TileId, TilePosition,
    TimeOfDay, apply, view,
};

use BasicAction::*;

mod arene;
use arene::*;

// --- Outils ---

fn move_to(hex: Hex) -> Action {
    Action::MoveHero { to: hex }
}

/// Deux Marche jouées : 4 points de mouvement.
fn marching(decor: City) -> GameState {
    let state = decor.state();
    let state = done(&state, basic(card(March, 1)));
    done(&state, basic(card(March, 2)))
}

fn revealed(events: &[Event]) -> Vec<EnemyToken> {
    events
        .iter()
        .filter_map(|event| match event {
            Event::EnemyTokenRevealed { hex, token } if *hex == CITY => Some(*token),
            _ => None,
        })
        .collect()
}

/// Explorer A + B depuis le village avec `tile` sur le dessus de la pioche, dans `state` (une
/// Marche en main) : les événements.
fn explore_city(state: &GameState) -> Vec<Event> {
    let state = done(state, basic(card(March, 1)));
    events_of(
        &state,
        &Action::Explore {
            position: CITY_POSITION,
        },
    )
}

fn city_placed(events: &[Event]) -> Vec<(Hex, BasicColor, u8)> {
    events
        .iter()
        .filter_map(|event| match event {
            Event::CityPlaced { hex, color, level } => Some((*hex, *color, *level)),
            _ => None,
        })
        .collect()
}

// --- Contenu et scénario ---

// C1 — Cartes de cité (MKUE p. 7, 5.g) : les cercles de la figurine selon le niveau.
#[test]
fn city_garrisons_follow_city_cards() {
    use EnemyColor::*;
    let content = GameContent::embedded();
    assert_eq!(
        content.city_garrison(BasicColor::White, 5),
        [Grey, White, White]
    );
    assert_eq!(
        content.city_garrison(BasicColor::Blue, 5),
        [Violet, Violet, White]
    );
    assert_eq!(
        content.city_garrison(BasicColor::Red, 11),
        [Brown, Violet, White, White, White]
    );
    assert_eq!(content.city_garrison(BasicColor::Green, 1), [Grey, Brown]);
    assert!(content.city_garrison(BasicColor::Green, 12).is_empty());
}

// C2 — ADR 0005 : `parse` refuse un fichier des cités incohérent.
#[test]
fn city_content_rejects_incoherent_levels() {
    let parse = |cities: &str| {
        GameContent::parse(ContentSources {
            cities,
            ..ContentSources::EMBEDDED
        })
    };
    let embedded = ContentSources::EMBEDDED.cities;
    let invalid = |result: Result<GameContent, ContentError>| {
        matches!(result, Err(ContentError::InvalidCities { .. }))
    };
    // Niveau manquant : le niveau 11 de la cité bleue retiré.
    let missing_level = embedded.replacen(
        "            [Violet, Violet, White, White, White], // 11\n",
        "",
        1,
    );
    assert_ne!(missing_level, embedded);
    assert!(invalid(parse(&missing_level)));
    // Un douzième niveau.
    let twelfth = embedded.replacen(
        "            [Violet, Violet, White, White, White], // 11\n",
        "            [Violet, Violet, White, White, White], // 11\n            [White], // 12\n",
        1,
    );
    assert!(invalid(parse(&twelfth)));
    // Une couleur de cité absente : la cité verte devient une seconde bleue.
    let no_green = embedded.replacen("color: Green", "color: Blue", 1);
    assert!(matches!(
        parse(&no_green),
        Err(ContentError::Duplicate { .. } | ContentError::Missing { .. })
    ));
    // Un jeton vert ou rouge.
    for color in ["Green", "Red"] {
        let wrong = embedded.replacen(
            "[Grey, Violet], // 1",
            &format!("[{color}, Violet], // 1"),
            1,
        );
        assert!(invalid(parse(&wrong)), "jeton {color} refusé");
    }
}

// C3 — MKUE p. 18, *City Levels* : le scénario dit les niveaux, Première reconnaissance ferme
// ses cités ; une liste incohérente est refusée.
#[test]
fn scenario_city_levels() {
    assert_eq!(
        Scenario::sandbox().cities(),
        CityRule::Levels(CityLevels::new(&[5]).expect("niveaux valides"))
    );
    assert_eq!(Scenario::first_reconnaissance().cities(), CityRule::Closed);
    assert!(CityLevels::new(&[]).is_err());
    assert!(CityLevels::new(&[0]).is_err());
    assert!(CityLevels::new(&[12]).is_err());
    assert!(CityLevels::new(&[5, 5, 5, 5, 5]).is_err());
    let levels = CityLevels::new(&[5, 8]).expect("niveaux valides");
    assert_eq!(
        (levels.level(0), levels.level(1), levels.level(2)),
        (5, 8, 8)
    );
}

// --- Révélation ---

// C4 — MKUE p. 7, 5.g : la cité révélée reçoit ses défenseurs selon son niveau, face cachée.
#[test]
fn revealed_city_gets_its_defenders() {
    let decor = Sandbox {
        figure: Some(VILLAGE),
        hand: vec![card(March, 1)],
        next: vec![TileId::Core(6)],
        ..Sandbox::default()
    };
    let start = decor.state();
    let violet = start.enemy_pile(EnemyColor::Violet);
    let white = start.enemy_pile(EnemyColor::White);
    let events = explore_city(&start);
    let defenders = [violet[0], violet[1], white[0]];
    let mut expected = vec![
        Event::TileRevealed {
            tile: TileId::Core(6),
            position: CITY_POSITION,
        },
        Event::CityPlaced {
            hex: CITY,
            color: BasicColor::Blue,
            level: 5,
        },
    ];
    expected.extend(
        defenders
            .iter()
            .map(|&token| Event::DefenderPlaced { token, hex: CITY }),
    );
    assert_events_in_order(&events, &expected);
    let after = events.iter().fold(start, apply);
    assert_eq!(after.enemy_tokens_at(CITY), defenders);
    let city = after.city(CITY).expect("cité en jeu");
    assert_eq!((city.color, city.level), (BasicColor::Blue, 5));
    assert!(city.shields.is_empty() && city.leader.is_none());
    assert_eq!(
        view(&after, SOLO).face_down_defenders.get(&CITY),
        Some(&vec![
            EnemyColor::Violet,
            EnemyColor::Violet,
            EnemyColor::White
        ])
    );
}

// C5 — MKUE p. 19, *Cities* : le niveau suit l'ordre de révélation ; au-delà de la liste, le
// dernier.
#[test]
fn city_level_follows_reveal_order() {
    let second_level = |levels: &[u8]| {
        let scenario = Scenario::sandbox().with_cities(CityRule::Levels(
            CityLevels::new(levels).expect("niveaux valides"),
        ));
        let mut events = Sandbox {
            scenario,
            figure: Some(VILLAGE),
            hand: vec![card(March, 1)],
            next: vec![TileId::Core(6)],
            ..Sandbox::default()
        }
        .events();
        // Une première cité déjà révélée, ailleurs.
        events.push(Event::TileRevealed {
            tile: TileId::Core(7),
            position: TilePosition::from_lattice(2, 1),
        });
        events.push(Event::CityPlaced {
            hex: Hex::new(7, -7),
            color: BasicColor::White,
            level: levels[0],
        });
        let state = events.iter().fold(GameState::default(), apply);
        city_placed(&explore_city(&state))
    };
    assert_eq!(second_level(&[5, 8]), [(CITY, BasicColor::Blue, 8)]);
    assert_eq!(second_level(&[5]), [(CITY, BasicColor::Blue, 5)]);
}

// C6 — MKUE p. 18 : dans Première reconnaissance, les cités restent fermées, sans défenseur.
#[test]
fn first_reconnaissance_cities_stay_closed() {
    let state = Sandbox {
        scenario: Scenario::first_reconnaissance().without_dummy(),
        figure: Some(VILLAGE),
        hand: vec![card(March, 1)],
        next: vec![TileId::Core(6)],
        ..Sandbox::default()
    }
    .state();
    let events = explore_city(&state);
    assert!(city_placed(&events).is_empty());
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, Event::DefenderPlaced { hex, .. } if *hex == CITY))
    );
    assert!(events.iter().fold(state, apply).city(CITY).is_none());
}

// C7 — Carte City (*revealed when a player is adjacent to the city, even at Night*) ;
// Compendium, *Movement* S8.
#[test]
fn city_defenders_revealed_next_to_it_even_at_night() {
    for time in [TimeOfDay::Night, TimeOfDay::Day] {
        let state = marching(City {
            time,
            ..City::default()
        });
        let events = events_of(&state, &move_to(NEAR_CITY));
        assert_eq!(revealed(&events), blue_defenders(), "{time:?}");
        let after = events.iter().fold(state, apply);
        assert!(!after.is_face_down(CITY));
        assert_eq!(
            view(&after, SOLO).enemy_tokens.get(&CITY),
            Some(&blue_defenders())
        );
    }
}

// C8 — Spec 043 ; Compendium, *Movement* S13 : le trajet vers la cité s'arrête sur le pas qui
// retourne ses défenseurs, même de nuit.
#[test]
fn route_to_city_stops_when_defenders_are_revealed() {
    let state = marching(City {
        time: TimeOfDay::Night,
        ..City::default()
    });
    let state = done(&state, basic(card(Stamina, 1)));
    let events = events_of(&state, &Action::FollowRoute { to: CITY });
    assert_eq!(
        events[0],
        Event::HeroMoved {
            player: SOLO,
            from: VILLAGE,
            to: NEAR_CITY,
            cost: 3,
        }
    );
    assert_eq!(revealed(&events), blue_defenders());
    assert!(!events.iter().any(|event| matches!(
        event,
        Event::HeroMoved { to, .. } | Event::AssaultStarted { site: to, .. } if *to == CITY
    )));
}

// --- Assaut ---

/// La cité de `tile`, défenseurs visibles, la figurine sur `figure`, voisine.
fn before_assault(tile: TileId, figure: Hex, defenders: Vec<EnemyToken>) -> City {
    City {
        tile,
        figure,
        defenders,
        face_up: true,
        ..City::default()
    }
}

// C9 — MKUE p. 7, 3.a (note) et 4.a ; Compendium, *Movement* S4 : entrer dans une cité est un
// assaut qui coûte 2, quel que soit son terrain.
#[test]
fn entering_a_city_is_an_assault_costing_two() {
    // Cité bleue (plaine) et cité verte (marais), depuis le côté S-O de leur tuile.
    let green = vec![token(Enemy::Minotaur, 1)];
    for (tile, defenders) in [
        (TileId::Core(6), blue_defenders()),
        (TileId::Core(5), green),
    ] {
        let decor = before_assault(tile, NEAR_CITY, defenders);
        // Marche (Mouvement 2), Promesse (Influence 2).
        let state = done(&decor.state(), basic(card(March, 1)));
        let state = done(&state, basic(card(Promise, 1)));
        let events = events_of(&state, &move_to(CITY));
        assert_events_in_order(
            &events,
            &[
                Event::HeroMoved {
                    player: SOLO,
                    from: NEAR_CITY,
                    to: CITY,
                    cost: 2,
                },
                Event::AssaultStarted {
                    player: SOLO,
                    site: CITY,
                    from: NEAR_CITY,
                },
                Event::ReputationChanged {
                    player: SOLO,
                    change: -1,
                },
            ],
        );
        let after = events.iter().fold(state, apply);
        assert_eq!(after.move_points(SOLO), 0);
        assert_eq!(after.influence_points(SOLO), 0);
        assert_eq!(after.reputation(SOLO).value(), -1);
    }
}

/// L'assaut de la cité de `decor` depuis sa voisine : Marche jouée, le pas, le combat engagé.
fn assaulting(decor: City) -> GameState {
    let state = done(&decor.state(), basic(card(March, 1)));
    let state = done(&state, move_to(CITY));
    done(&state, Action::StartCombat)
}

// C10 — Compendium, *Cities* S1 ; FAQ p2 : l'assaut combat tous les défenseurs, fortifiés, et
// l'ennemi errant provoqué par le pas, sans fortification.
#[test]
fn city_assault_fights_all_defenders() {
    let mountain = Hex::new(3, -5);
    let state = assaulting(City {
        placed: vec![(prowlers(), mountain)],
        ..before_assault(TileId::Core(6), NEAR_CITY, blue_defenders())
    });
    let combat = state.combat(SOLO).expect("combat de l'assaut");
    let fortified: Vec<(EnemyToken, bool)> = combat
        .enemies
        .iter()
        .map(|enemy| (enemy.token, enemy.site_fortified))
        .collect();
    assert_eq!(fortified.len(), 4);
    for defender in blue_defenders() {
        assert!(fortified.contains(&(defender, true)), "{defender:?}");
    }
    assert!(fortified.contains(&(prowlers(), false)));
}

/// Les cibles de chaque `DeclareAttack` légal, par leurs jetons.
fn target_sets(state: &GameState) -> Vec<Vec<EnemyToken>> {
    arcanum_core::legal_actions(state, SOLO)
        .into_iter()
        .filter_map(|action| match action {
            Action::DeclareAttack { targets } => Some(targets),
            _ => None,
        })
        .collect()
}

/// Cité blanche niveau 5 de la spec : des Gardes et deux Canonniers.
fn white_defenders() -> Vec<EnemyToken> {
    vec![
        guardsmen(),
        token(Enemy::Gunners, 1),
        token(Enemy::Gunners, 2),
    ]
}

// C11 — MKUE p. 9, 3.a ; Compendium, *Cities* Q2 : les Gardes, doublement fortifiés, ne sont pas
// une cible à distance ; un Canonnier seul est une cible de siège.
#[test]
fn city_defenders_are_fortified() {
    let state = assaulting(before_assault(
        TileId::Core(7),
        NEAR_WHITE_CITY,
        white_defenders(),
    ));
    let combat = state.combat(SOLO).expect("combat de l'assaut");
    assert_eq!(
        combat.enemy(guardsmen()).map(|e| e.fortification()),
        Some(2)
    );
    let sets = target_sets(&state);
    assert!(!sets.is_empty());
    assert!(
        sets.iter().all(|set| !set.contains(&guardsmen())),
        "{sets:?}"
    );
    let gunner = token(Enemy::Gunners, 1);
    assert!(sets.contains(&vec![gunner]));
    let aimed = done(&state, declare(&[gunner]));
    let combat = aimed.combat(SOLO).expect("combat de l'assaut");
    let attack = |kind| arcanum_core::Effect::Attack {
        amount: 3,
        element: arcanum_core::Element::Physical,
        kind,
    };
    assert!(!combat.accepts(attack(arcanum_core::AttackKind::Ranged)));
    assert!(combat.accepts(attack(arcanum_core::AttackKind::Siege)));
}

// --- Bonus de la cité ---

/// Assaut de la cité de `tile` défendue par `defenders`, depuis sa voisine : le combat engagé.
fn city_combat(
    tile: TileId,
    defenders: Vec<EnemyToken>,
    hand: Vec<arcanum_core::CardId>,
) -> GameState {
    let figure = if tile == TileId::Core(7) {
        NEAR_WHITE_CITY
    } else {
        NEAR_CITY
    };
    let mut hand = hand;
    hand.insert(0, card(March, 1));
    assaulting(City {
        hand,
        ..before_assault(tile, figure, defenders)
    })
}

fn enemy(state: &GameState, token: EnemyToken) -> arcanum_core::CombatEnemy {
    state
        .combat(SOLO)
        .and_then(|combat| combat.enemy(token))
        .cloned()
        .unwrap_or_else(|| panic!("{token:?} hors du combat"))
}

// C12 — MKUE p. 10, *City Assaults* 1.a ; Compendium, *Cities* S7 : Armure +1, avant les
// réductions : Bouclier de glace renforcé sur les Gardes donne 8 − 3 = 5.
#[test]
fn white_city_adds_armor_before_reductions() {
    let ice_shield = advanced(arcanum_core::AdvancedAction::IceShield);
    let state = city_combat(TileId::Core(7), white_defenders(), vec![ice_shield]);
    assert_eq!(enemy(&state, guardsmen()).effective_armor(), 8);
    let state = done(&to_block_phase(&state), declare_block(guardsmen()));
    let state = done(&state, strong(ice_shield, BLUE_DIE));
    assert_eq!(enemy(&state, guardsmen()).effective_armor(), 5);
}

// C13 — MKUE p. 10, 1.b ; Compendium, *Cities* S8 : attaque de glace ou de feu +2, de feu glacé
// +1 ; l'attaque physique ne change pas ; Vif double l'attaque augmentée.
#[test]
fn blue_city_raises_ice_and_fire_attacks() {
    let defenders = vec![
        token(Enemy::IceMages, 1),
        token(Enemy::FireMages, 1),
        token(Enemy::AltemMages, 1),
        guardsmen(),
        token(Enemy::Freezers, 1),
    ];
    let state = city_combat(TileId::Core(6), defenders.clone(), Vec::new());
    let attacks: Vec<(Option<u8>, Option<u32>)> = defenders
        .iter()
        .map(|&token| {
            let enemy = enemy(&state, token);
            (enemy.effective_attack(), enemy.block_needed())
        })
        .collect();
    assert_eq!(
        attacks,
        [
            (Some(7), Some(7)),
            (Some(8), Some(8)),
            (Some(5), Some(5)),
            (Some(3), Some(3)),
            (Some(5), Some(10)),
        ]
    );
    // Les Mages d'Altem (brutaux) : le double de l'attaque augmentée.
    assert_eq!(
        enemy(&state, token(Enemy::AltemMages, 1)).damage(),
        Some(10)
    );
}

// C14 — MKUE p. 10, 1.c ; Compendium, *Cities* S5, S6 : une attaque physique devient brutale,
// une seule fois ; une attaque de feu non.
#[test]
fn red_city_makes_physical_attackers_brutal() {
    let minotaur = token(Enemy::Minotaur, 1);
    let gargoyle = token(Enemy::Gargoyle, 1);
    let fire_mages = token(Enemy::FireMages, 1);
    let state = city_combat(
        TileId::Core(8),
        vec![minotaur, gargoyle, fire_mages],
        Vec::new(),
    );
    assert_eq!(enemy(&state, minotaur).damage(), Some(10));
    assert_eq!(enemy(&state, gargoyle).damage(), Some(10));
    assert_eq!(enemy(&state, gargoyle).wounds_if_unblocked(2), Some(5));
    assert_eq!(enemy(&state, fire_mages).damage(), Some(6));
    assert!(enemy(&state, minotaur).city_abilities().is_empty());
    assert_eq!(
        enemy(&state, gargoyle).city_abilities(),
        [arcanum_core::EnemyAbility::Brutal]
    );
}

// C15 — MKUE p. 10, 1.d : une attaque physique devient toxique ; de feu ou de glace non.
#[test]
fn green_city_makes_physical_attackers_poisonous() {
    use arcanum_core::EnemyAbility::Poison;
    let worm = token(Enemy::CryptWorm, 1);
    let gunners = token(Enemy::Gunners, 1);
    let freezers = token(Enemy::Freezers, 1);
    let state = city_combat(
        TileId::Core(5),
        vec![guardsmen(), worm, gunners, freezers],
        Vec::new(),
    );
    assert!(enemy(&state, guardsmen()).has_ability(Poison));
    assert!(enemy(&state, worm).has_ability(Poison));
    assert!(!enemy(&state, gunners).has_ability(Poison));
    assert!(!enemy(&state, freezers).has_ability(Poison));
    assert_eq!(enemy(&state, worm).city_abilities(), [Poison]);
}

// C16 — Compendium, *Cities* S4 ; FAQ p2 : le monstre invoqué dans la cité rouge est brutal et
// fortifié par elle.
#[test]
fn summoned_monster_gets_city_bonus() {
    let illusionists = token(Enemy::Illusionists, 1);
    // Les deux Minotaures sur la cité : la Gargouille est en tête de la pile marron.
    let defenders = vec![
        illusionists,
        token(Enemy::Minotaur, 1),
        token(Enemy::Minotaur, 2),
    ];
    let state = to_block_phase(&city_combat(TileId::Core(8), defenders, Vec::new()));
    let summoner = enemy(&state, illusionists);
    assert_eq!(summoner.attacker(), Some(token(Enemy::Gargoyle, 1)));
    assert_eq!(summoner.damage(), Some(10));
    assert!(summoner.site_fortified);
}

// C17 — FAQ p2 : Endurance givrée renforcée ne compte que les capacités imprimées : 5 + 1
// (Fortifié des Gardes), ni la fortification du site ni le Toxique de la cité verte.
#[test]
fn cold_toughness_ignores_city_abilities() {
    let state = to_block_phase(&city_combat(TileId::Core(5), vec![guardsmen()], Vec::new()));
    let state = done(&state, declare_block(guardsmen()));
    let combat = state.combat(SOLO).expect("combat de l'assaut");
    assert_eq!(
        combat.counted_points(arcanum_core::Effect::IceBlockPerAbility(5)),
        Some(6)
    );
}

// --- Issue ---

/// En phase d'attaque de l'assaut de la cité bleue, ces défenseurs vaincus, dans l'ordre.
fn defeating(state: &GameState, defeated: &[EnemyToken]) -> GameState {
    defeated
        .iter()
        .fold(skip_to_attack_phase(state), |state, &token| {
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

fn blue_assault() -> GameState {
    city_combat(TileId::Core(6), blue_defenders(), Vec::new())
}

fn fire_mages() -> EnemyToken {
    token(Enemy::FireMages, 1)
}

// C19 — MKUE p. 10, 1.a et 3.d ; Compendium, *Cities* S23 : un Bouclier par défenseur vaincu,
// puis la retraite ; les autres défenseurs restent, face visible.
#[test]
fn failed_city_assault_keeps_shields() {
    let state = defeating(&blue_assault(), &[fire_mages()]);
    assert_eq!(state.fame(SOLO), 5);
    let events = events_of(&state, &Action::EndCombat);
    assert_eq!(
        events,
        [
            Event::CombatEnded { player: SOLO },
            Event::CityShieldPlaced {
                player: SOLO,
                hex: CITY,
                enemy: fire_mages(),
            },
            Event::HeroWithdrew {
                player: SOLO,
                from: CITY,
                to: NEAR_CITY,
            },
        ]
    );
    let after = events.iter().fold(state, apply);
    assert_eq!(after.figure(SOLO), Some(NEAR_CITY));
    assert_eq!(
        after.enemy_tokens_at(CITY),
        [token(Enemy::IceMages, 1), token(Enemy::Freezers, 1)]
    );
    assert!(!after.is_face_down(CITY));
    let city = after.city(CITY).expect("cité en jeu");
    assert_eq!(city.shields, [SOLO]);
    assert_eq!(city.leader, None);
    assert!(after.shields().is_empty());
}

// C20 — Compendium, *Cities* S23 ; FAQ p2 : un nouvel assaut coûte de nouveau 1 de Réputation et
// ne combat que les défenseurs restants.
#[test]
fn second_assault_faces_remaining_defenders() {
    let remaining = vec![token(Enemy::IceMages, 1), token(Enemy::Freezers, 1)];
    let decor = City {
        hand: vec![card(March, 1)],
        ..before_assault(TileId::Core(6), NEAR_CITY, remaining.clone())
    };
    let mut events = decor.events();
    events.push(Event::CityShieldPlaced {
        player: SOLO,
        hex: CITY,
        enemy: fire_mages(),
    });
    let state = events.iter().fold(GameState::default(), apply);
    let state = done(&state, basic(card(March, 1)));
    let state = done(&state, move_to(CITY));
    assert_eq!(state.reputation(SOLO).value(), -1);
    let state = done(&state, Action::StartCombat);
    let fought: Vec<EnemyToken> = state
        .combat(SOLO)
        .expect("combat de l'assaut")
        .enemies
        .iter()
        .map(|enemy| enemy.token)
        .collect();
    assert_eq!(fought, remaining);
    assert_eq!(state.city(CITY).map(|city| city.shields.len()), Some(1));
}

// C21 — MKUE p. 10, 2.c et *City Assaults* 2 ; Compendium, *Cities* S3 : tous les défenseurs
// vaincus, un Bouclier pour chacun dans l'ordre, la cité conquise ; l'errant provoqué reste.
#[test]
fn city_conquered_when_all_defenders_are_defeated() {
    let mountain = Hex::new(3, -5);
    let state = assaulting(City {
        placed: vec![(prowlers(), mountain)],
        ..before_assault(TileId::Core(6), NEAR_CITY, blue_defenders())
    });
    let order = [
        token(Enemy::Freezers, 1),
        fire_mages(),
        token(Enemy::IceMages, 1),
    ];
    let state = defeating(&state, &order);
    let events = events_of(&state, &Action::EndCombat);
    let mut expected = vec![Event::CombatEnded { player: SOLO }];
    expected.extend(order.iter().map(|&enemy| Event::CityShieldPlaced {
        player: SOLO,
        hex: CITY,
        enemy,
    }));
    expected.push(Event::CityConquered {
        hex: CITY,
        leader: SOLO,
    });
    assert_eq!(events, expected);
    let after = events.iter().fold(state, apply);
    assert_eq!(after.figure(SOLO), Some(CITY));
    assert_eq!(after.enemy_tokens_at(mountain), [prowlers()]);
    let city = after.city(CITY).expect("cité en jeu");
    assert_eq!(city.shields, [SOLO, SOLO, SOLO]);
    assert_eq!(city.leader, Some(SOLO));
}

// C22 — Compendium, *Cities* S21 : le tour de la conquête, ni interaction ni déplacement.
#[test]
fn no_interaction_after_conquering_a_city() {
    let state = defeating(&blue_assault(), &blue_defenders());
    let after = done(&state, Action::EndCombat);
    assert_eq!(after.city(CITY).and_then(|city| city.leader), Some(SOLO));
    let legal = arcanum_core::legal_actions(&after, SOLO);
    assert!(
        !legal.iter().any(|action| matches!(
            action,
            Action::MoveHero { .. } | Action::FollowRoute { .. } | Action::Interact
        )),
        "{legal:?}"
    );
}

// L1 — Plus aucun défenseur : le premier assaut conquiert la cité, sans Bouclier.
#[test]
fn undefended_city_is_conquered_on_entry() {
    let state = done(
        &before_assault(TileId::Core(6), NEAR_CITY, Vec::new()).state(),
        basic(card(March, 1)),
    );
    let events = events_of(&state, &move_to(CITY));
    assert!(events.contains(&Event::CityConquered {
        hex: CITY,
        leader: SOLO,
    }));
    assert!(!events.iter().any(|event| matches!(
        event,
        Event::SiteConquered { .. } | Event::CityShieldPlaced { .. }
    )));
    let after = events.iter().fold(state, apply);
    assert!(after.city(CITY).is_some_and(|city| city.shields.is_empty()));
}

// --- Cité conquise ---

fn legal(state: &GameState) -> Vec<Action> {
    arcanum_core::legal_actions(state, SOLO)
}

// C23 — Carte City, *While Conquered* ; MKUE p. 18 : une cité conquise est une case ordinaire au
// coût 2, qu'on traverse, où l'on finit son tour sans retraite forcée.
#[test]
fn conquered_city_is_an_ordinary_space() {
    let state = state_of(&conquered_city(
        TileId::Core(6),
        NEAR_CITY,
        3,
        SOLO,
        City::default().hand,
    ));
    let state = done(&state, basic(card(March, 1)));
    let events = events_of(&state, &move_to(CITY));
    assert_eq!(
        events,
        [Event::HeroMoved {
            player: SOLO,
            from: NEAR_CITY,
            to: CITY,
            cost: 2,
        }]
    );
    let after = events.iter().fold(state.clone(), apply);
    assert!(view(&after, SOLO).forced_withdrawal.is_none());
    // Un trajet la traverse : de sa voisine S-O à sa voisine N-E (plaine), 2 + 2.
    let state = done(&state, basic(card(March, 2)));
    let beyond = Hex::new(5, -6);
    let events = events_of(&state, &Action::FollowRoute { to: beyond });
    assert!(events.contains(&Event::HeroMoved {
        player: SOLO,
        from: NEAR_CITY,
        to: CITY,
        cost: 2,
    }));
}

// C24 — MKUE p. 8, 3.a et 5.a : +1 d'influence par Bouclier du joueur, une fois, au début.
#[test]
fn city_interaction_adds_shield_bonus() {
    let mut events = conquered_city(TileId::Core(6), CITY, 3, SOLO, vec![card(Promise, 1)]);
    // Réputation +2 : bonus de +1 (la case +1 ne donne rien).
    events.push(Event::ReputationChanged {
        player: SOLO,
        change: 2,
    });
    let state = state_of(&events);
    let events = events_of(&state, &Action::Interact);
    assert_eq!(
        events,
        [Event::InteractionStarted {
            player: SOLO,
            site: arcanum_core::Site::City(BasicColor::Blue),
            bonus: 1,
            city_shields: 3,
        }]
    );
    let state = events.iter().fold(state, apply);
    let state = done(&state, basic(card(Promise, 1)));
    assert_eq!(state.influence_points(SOLO), 6);
    assert_eq!(
        view(&state, SOLO).interaction.map(|i| i.city_shields),
        Some(3)
    );
}

// C25 — Compendium, *Cities* S19 : interaction permise dans une cité conquise par un autre, sans
// bonus ; refusée dans une cité non conquise, même avec des Boucliers du joueur.
#[test]
fn city_interaction_needs_a_conquered_city() {
    let other = arcanum_core::PlayerId(1);
    let state = state_of(&conquered_city(
        TileId::Core(6),
        CITY,
        2,
        other,
        vec![card(Promise, 1)],
    ));
    let events = events_of(&state, &Action::Interact);
    assert!(matches!(
        events[..],
        [Event::InteractionStarted {
            city_shields: 0,
            ..
        }]
    ));
    let mut events = City {
        figure: CITY,
        face_up: true,
        ..City::default()
    }
    .events();
    events.push(Event::CityShieldPlaced {
        player: SOLO,
        hex: CITY,
        enemy: guardsmen(),
    });
    refused(&state_of(&events), Action::Interact);
}

fn unit(unit: arcanum_core::Unit) -> arcanum_core::UnitCard {
    arcanum_core::UnitCard::new(unit, 1)
}

fn recruits(state: &GameState) -> Vec<arcanum_core::UnitCard> {
    legal(state)
        .into_iter()
        .filter_map(|action| match action {
            Action::RecruitUnit { unit, .. } => Some(unit),
            _ => None,
        })
        .collect()
}

// C26 — MKUE p. 8, 4.a et 4.e ; carte City : les unités à icône Cité ; dans la cité blanche,
// toutes.
#[test]
fn city_recruits_city_units_white_city_all() {
    let catapults = unit(arcanum_core::Unit::Catapults);
    let foresters = unit(arcanum_core::Unit::Foresters);
    let offer = || unit_events(&[], &[catapults, foresters]);
    let blue = interacting_in_city(TileId::Core(6), 9, offer());
    assert_eq!(recruits(&blue), [catapults]);
    let white = interacting_in_city(TileId::Core(7), 9, offer());
    assert_eq!(recruits(&white), [catapults, foresters]);
}

fn elite_deck(cards: &[arcanum_core::UnitCard]) -> Event {
    Event::CommonDeckShuffled {
        deck: arcanum_core::CommonDeck::EliteUnits,
        order: cards
            .iter()
            .map(|&card| arcanum_core::CommonCard::Unit(card))
            .collect(),
    }
}

// C27 — MKUE p. 8, 4.e : dans la cité blanche, 2 influence : la carte du dessus des unités
// d'élite rejoint l'offre ; deux fois, deux cartes.
#[test]
fn white_city_adds_elite_units_to_offer() {
    let gunners = unit(arcanum_core::Unit::AmotepGunners);
    let guardians = unit(arcanum_core::Unit::AltemGuardians);
    let state = interacting_in_city(TileId::Core(7), 2, vec![elite_deck(&[gunners, guardians])]);
    let offer_before = state.offer(arcanum_core::Offer::Units).len();
    let events = events_of(&state, &Action::AddEliteUnit);
    assert_eq!(
        events,
        [Event::EliteUnitAdded {
            player: SOLO,
            card: arcanum_core::CommonCard::Unit(gunners),
            cost: 2,
        }]
    );
    let state = events.iter().fold(state, apply);
    assert_eq!(
        state.offer(arcanum_core::Offer::Units).len(),
        offer_before + 1
    );
    assert!(
        state
            .offer(arcanum_core::Offer::Units)
            .contains(&arcanum_core::CommonCard::Unit(gunners))
    );
    // L'influence de la cité (3 Boucliers) suffit pour une seconde carte.
    let state = done(&state, Action::AddEliteUnit);
    assert_eq!(
        state.offer(arcanum_core::Offer::Units).len(),
        offer_before + 2
    );
    // Ailleurs que dans la cité blanche, non.
    let blue = interacting_in_city(TileId::Core(6), 2, vec![elite_deck(&[gunners])]);
    assert!(!legal(&blue).contains(&Action::AddEliteUnit));
}

fn spell(spell: arcanum_core::Spell) -> arcanum_core::CardId {
    arcanum_core::CardId::new(arcanum_core::DeedCard::Spell(spell), 1)
}

// C28 — MKUE p. 8, 4.e : la cité bleue vend des sorts comme la tour de mage, offre complétée.
#[test]
fn blue_city_sells_spells() {
    use arcanum_core::Spell::*;
    let spells = [
        spell(Snowstorm),
        spell(Fireball),
        spell(Restoration),
        spell(Demolish),
    ];
    let state = interacting_in_city(
        TileId::Core(6),
        4,
        deal(
            arcanum_core::CommonDeck::Spells,
            arcanum_core::Offer::Spells,
            &spells,
            3,
        ),
    );
    let buy = legal(&state)
        .into_iter()
        .find(|action| matches!(action, Action::BuySpell { card, .. } if *card == spell(Snowstorm)))
        .expect("Tempête de neige à vendre (7 influence et un mana bleu)");
    let events = events_of(&state, &buy);
    assert!(events.contains(&Event::OfferCardDealt {
        offer: arcanum_core::Offer::Spells,
        card: arcanum_core::CommonCard::Deed(spell(Demolish)),
    }));
    let state = events.iter().fold(state, apply);
    assert_eq!(state.deck(SOLO).first(), Some(&spell(Snowstorm)));
}

// C29 — MKUE p. 8, 4.e ; Compendium, *Cities* S21 : la cité verte vend une action avancée de
// l'offre d'actions avancées (complétée), ou la carte du dessus de leur pioche, pour 6.
#[test]
fn green_city_sells_advanced_actions() {
    use arcanum_core::AdvancedAction::*;
    let cards = [
        advanced(IceShield),
        advanced(FireBolt),
        advanced(Agility),
        advanced(Diplomacy),
        advanced(PathFinding),
    ];
    let decor = || {
        deal(
            arcanum_core::CommonDeck::AdvancedActions,
            arcanum_core::Offer::AdvancedActions,
            &cards,
            3,
        )
    };
    let state = interacting_in_city(TileId::Core(5), 3, decor());
    let events = events_of(&state, &Action::BuyAdvancedAction { card: cards[0] });
    assert!(events.contains(&Event::OfferCardDealt {
        offer: arcanum_core::Offer::AdvancedActions,
        card: arcanum_core::CommonCard::Deed(cards[3]),
    }));
    let after = events.iter().fold(state, apply);
    assert_eq!(after.deck(SOLO).first(), Some(&cards[0]));
    assert!(
        !after
            .offer(arcanum_core::Offer::AdvancedActions)
            .contains(&arcanum_core::CommonCard::Deed(cards[0]))
    );
    // La carte du dessus de la pioche, à l'aveugle.
    let state = interacting_in_city(TileId::Core(5), 3, decor());
    let events = events_of(&state, &Action::BuyAdvancedActionFromDeck);
    assert_eq!(
        events,
        [Event::AdvancedActionDrawnBlind {
            player: SOLO,
            card: cards[3],
            cost: 6,
        }]
    );
    let after = events.iter().fold(state, apply);
    assert_eq!(after.deck(SOLO).first(), Some(&cards[3]));
    assert_eq!(after.offer(arcanum_core::Offer::AdvancedActions).len(), 3);
    assert_eq!(after.influence_points(SOLO), 0);
}

// C30 — MKUE p. 8, 4.e ; p. 11, 5.b ; Compendium, *End of Turn* S5 : la cité rouge vend des
// artefacts, tirés à la fin du tour comme une récompense.
#[test]
fn red_city_sells_artifacts() {
    let artifacts: Vec<arcanum_core::CardId> = arcanum_core::Artifact::ALL
        .into_iter()
        .map(artifact)
        .collect();
    let state = interacting_in_city(
        TileId::Core(8),
        21,
        deal(
            arcanum_core::CommonDeck::Artifacts,
            arcanum_core::Offer::Spells,
            &artifacts,
            0,
        ),
    );
    assert_eq!(
        events_of(&state, &Action::BuyArtifact),
        [Event::ArtifactBought {
            player: SOLO,
            cost: 12,
        }]
    );
    let state = done(&done(&state, Action::BuyArtifact), Action::BuyArtifact);
    assert_eq!(state.artifact_rewards(SOLO), 2);
    assert_eq!(state.influence_points(SOLO), 0);
    let end = legal(&state)
        .into_iter()
        .find(|action| matches!(action, Action::EndTurn { .. }))
        .expect("fin du tour");
    let drawn = events_of(&state, &end)
        .into_iter()
        .find_map(|event| match event {
            Event::ArtifactsDrawn { cards, .. } => Some(cards),
            _ => None,
        })
        .expect("artefacts tirés");
    assert_eq!(drawn.len(), 3);
}

// C31 — Compendium, *Cities* S21 : aucune cité ne vend de soin.
#[test]
fn cities_sell_no_healing() {
    for tile in [
        TileId::Core(5),
        TileId::Core(6),
        TileId::Core(7),
        TileId::Core(8),
    ] {
        let state = interacting_in_city(tile, 9, Vec::new());
        assert!(state.interaction(SOLO).is_some(), "{tile:?}");
        assert!(!legal(&state).contains(&Action::BuyHealing), "{tile:?}");
    }
}

// L7 — Pioche d'artefacts trop courte : l'achat d'un artefact est refusé.
#[test]
fn artifact_purchase_needs_artifacts_left() {
    let state = interacting_in_city(TileId::Core(8), 9, Vec::new());
    assert!(!legal(&state).contains(&Action::BuyArtifact));
}

// --- Limite de main, Le Conquérant ---

/// Plaine de la campagne 1, village, à deux pas de la cité.
const TWO_FROM_CITY: Hex = Hex::new(2, -3);

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

/// La cité bleue conquise par le joueur seul (deux Boucliers), la figurine sur `figure`.
fn led_city(figure: Hex) -> GameState {
    state_of(&conquered_city(
        TileId::Core(6),
        figure,
        2,
        SOLO,
        City::default().hand,
    ))
}

// C32 — MKUE p. 11, 7.b ; carte de cité ; Compendium, *End of Turn* S8, *Cities* S20 : sur une
// cité conquise ou à côté, +1 avec un Bouclier, +2 pour le Seigneur ; le plus grand des bonus de
// forteresse et de cité.
#[test]
fn conquered_city_raises_hand_limit() {
    // Main de 4 après la Marche : limite 5 + 2.
    assert_eq!(end_turn_draws(&led_city(CITY)), 3);
    assert_eq!(end_turn_draws(&led_city(NEAR_CITY)), 3);
    assert_eq!(view(&led_city(NEAR_CITY), SOLO).city_hand_bonus, 2);
    assert_eq!(end_turn_draws(&led_city(TWO_FROM_CITY)), 1);
    assert_eq!(view(&led_city(TWO_FROM_CITY), SOLO).city_hand_bonus, 0);
    // Deux Boucliers sur une cité pas encore conquise : rien.
    let mut events = City {
        figure: NEAR_CITY,
        ..City::default()
    }
    .events();
    events.extend((0..2).map(|_| Event::CityShieldPlaced {
        player: SOLO,
        hex: CITY,
        enemy: guardsmen(),
    }));
    assert_eq!(end_turn_draws(&state_of(&events)), 1);
    // La campagne 3 en A (à la place de la campagne 2) : sa forteresse possédée (4, -3) et la
    // cité sont voisines de (4, -4). Bonus 1 et 2 : le plus grand, pas la somme.
    let between = Hex::new(4, -4);
    let mut events: Vec<Event> =
        conquered_city(TileId::Core(6), between, 2, SOLO, City::default().hand)
            .into_iter()
            .filter(|event| {
                !matches!(event, Event::TileRevealed { position, .. }
                if *position == TilePosition::from_lattice(1, 0))
            })
            .collect();
    events.push(Event::TileRevealed {
        tile: TileId::Countryside(3),
        position: TilePosition::from_lattice(1, 0),
    });
    let keep = Hex::new(4, -3);
    events.push(Event::SiteConquered {
        player: SOLO,
        hex: keep,
    });
    let state = state_of(&events);
    assert_eq!(state.keep_hand_bonus(SOLO), 1);
    assert_eq!(end_turn_draws(&state), 3);
}

// C32 — MKUE p. 4, 2.g ; p. 9, 5.c : la nouvelle manche suit le bonus, pas le seuil d'assommé.
#[test]
fn city_bonus_at_new_round_not_for_knock_out() {
    // Sept cartes de plus, deck vide : la manche suivante pioche 5 + 2.
    let state = apply(
        led_city(CITY),
        &Event::DeckShuffled {
            player: SOLO,
            order: Vec::new(),
        },
    );
    let state = [
        Swiftness,
        Determination,
        Concentration,
        Tranquility,
        Crystallize,
        Improvisation,
        Threaten,
    ]
    .into_iter()
    .map(|action| card(action, 1))
    .fold(state, |state, card| {
        apply(state, &Event::CardDrawn { player: SOLO, card })
    });
    let drawn = events_of(&state, &Action::AnnounceEndOfRound)
        .iter()
        .filter(|event| matches!(event, Event::CardDrawn { .. }))
        .count();
    assert_eq!(drawn, 7);
    // Le seuil d'assommé reste celui du niveau.
    assert_eq!(view(&led_city(CITY), SOLO).hand_limit, 5);
}

// C33 — MKUE p. 15 ; carte de score : Le Conquérant ne compte que les forteresses et les tours.
#[test]
fn conqueror_ignores_city_shields() {
    let score = arcanum_core::final_score(&led_city(NEAR_CITY), SOLO);
    assert_eq!(score.points(arcanum_core::Achievement::Conqueror), 0);
}

// --- Vue, retour en arrière ---

// C34 — Spec 062, *Information cachée* : chaque cité révélée, ses défenseurs face cachée (couleurs
// seulement), ses Boucliers dans l'ordre, son Seigneur ; en combat, les valeurs effectives.
#[test]
fn view_shows_cities() {
    let state = City::default().state();
    let seen = view(&state, SOLO);
    assert_eq!(
        seen.cities.get(&CITY),
        Some(&arcanum_core::CityState::new(BasicColor::Blue, 5))
    );
    assert_eq!(seen.face_down_defenders.get(&CITY).map(Vec::len), Some(3));
    assert!(!seen.enemy_tokens.contains_key(&CITY));
    // Conquise : deux Boucliers du joueur, Seigneur.
    let seen = view(&led_city(NEAR_CITY), SOLO);
    let city = seen.cities.get(&CITY).expect("cité vue");
    assert_eq!(city.shields, vec![SOLO, SOLO]);
    assert_eq!(city.leader, Some(SOLO));
    // En combat dans la cité blanche : les Gardes à 8 d'armure, de la cité blanche.
    let state = city_combat(TileId::Core(7), white_defenders(), Vec::new());
    let seen = view(&state, SOLO);
    let guards = seen
        .combat
        .as_ref()
        .and_then(|combat| combat.enemy(guardsmen()))
        .expect("Gardes en combat");
    assert_eq!(guards.city, Some(BasicColor::White));
    assert_eq!(guards.effective_armor(), 8);
}

// C35 — Spec 024 ; MKUE p. 5, *Reverting* 2 : le pas qui retourne les défenseurs ne s'annule pas,
// un assaut contre des défenseurs déjà visibles si ; les achats à l'aveugle non, l'artefact si.
#[test]
fn city_undo_points() {
    let (mut game, puppet) = runner(City::default().events());
    play(&mut game, &puppet, basic(card(March, 1)));
    play(&mut game, &puppet, basic(card(March, 2)));
    play(&mut game, &puppet, move_to(NEAR_CITY));
    assert_eq!(
        game.undo_status(),
        arcanum_core::UndoStatus::InformationRevealed
    );
    let (mut game, puppet) =
        runner(before_assault(TileId::Core(6), NEAR_CITY, blue_defenders()).events());
    play(&mut game, &puppet, basic(card(March, 1)));
    play(&mut game, &puppet, move_to(CITY));
    assert_eq!(game.undo_status(), arcanum_core::UndoStatus::Possible);
    assert!(
        Event::CityPlaced {
            hex: CITY,
            color: BasicColor::Blue,
            level: 5,
        }
        .blocks_undo()
    );
    assert!(
        Event::AdvancedActionDrawnBlind {
            player: SOLO,
            card: advanced(arcanum_core::AdvancedAction::IceShield),
            cost: 6,
        }
        .blocks_undo()
    );
    assert!(
        Event::EliteUnitAdded {
            player: SOLO,
            card: arcanum_core::CommonCard::Unit(unit(arcanum_core::Unit::AltemGuardians)),
            cost: 2,
        }
        .blocks_undo()
    );
    assert!(
        !Event::ArtifactBought {
            player: SOLO,
            cost: 12
        }
        .blocks_undo()
    );
}
