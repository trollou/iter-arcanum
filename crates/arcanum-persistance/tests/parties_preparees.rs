//! Parties préparées pour la vérification à l'œil (issue privée 64, `docs/verification-manuelle.md`) :
//! des sauvegardes ordinaires (spec 024) construites événement par événement, versionnées dans
//! `scripts/parties/` et chargées par `scripts/verif-manuelle.sh charger <nom>`.
//!
//! Si un test échoue après un changement voulu (format des événements, contenu), régénérer avec
//! `ARCANUM_REGENERER_DOCS=1 cargo test -p arcanum-persistance --test parties_preparees` et relire le diff.

use std::fs;
use std::path::PathBuf;

use arcanum_core::{
    Action, BasicAction, BasicColor, CardId, EnemyToken, Event, EventLog, GameConfig,
    GameEndReason, GameRngs, GameRunner, GameState, Hex, RngPositions, Scenario, ScenarioId,
    TileId, TilePosition, TurnFact, Unit, UnitCard, apply, decide, legal_actions, new_game,
};
use arcanum_persistance::{SaveGame, load, save};

// L'arène des tests de combat d'arcanum-core, partagée plutôt que recopiée (bilan de la spec 032).
#[path = "../../arcanum-core/tests/arene/mod.rs"]
mod arene;
use arene::*;

use BasicAction::*;

/// Date des parties préparées : fixe, pour que le fichier ne change pas à chaque génération.
const SAVED_AT: &str = "2026-01-01T00:00:00+00:00";

/// La vraie mise en place du bac à sable avec Arythea, graine 0 : la configuration, ses
/// événements et les positions des générateurs après elle. La suite d'une partie préparée tire
/// ainsi le même hasard qu'après cette mise en place (spec 038).
fn sandbox_setup() -> (GameConfig, Vec<Event>, RngPositions) {
    let config = arythea_config();
    let mut rngs = config.rngs();
    let events = new_game(&config, &mut rngs);
    (config, events, rngs.positions())
}

/// Bac à sable avec Arythea, de jour, au début du premier tour : tuiles de campagne 1 et 2,
/// toute la pioche de tuiles,
/// figurine sur `figure`, main `hand` (le deck de départ derrière), source verte, rouge, bleue
/// et blanche, les jetons `placed` sur leurs cases (pris en tête de leur pile).
fn arena(figure: Hex, hand: &[CardId], placed: &[(EnemyToken, Hex)]) -> Vec<Event> {
    let (_, setup, _) = sandbox_setup();
    // La pioche de tuiles de la vraie mise en place, pour pouvoir explorer.
    let tiles = setup
        .iter()
        .find_map(|event| match event {
            Event::TileDeckFormed { order } => Some(order.clone()),
            _ => None,
        })
        .expect("la mise en place forme la pioche de tuiles");
    // Rangée par numéro (MKUE p. 18) : les deux premières sont les tuiles de campagne 1 et 2.
    assert_eq!(tiles[..2], [TileId::Countryside(1), TileId::Countryside(2)]);
    arena_events(figure, hand, placed, &tiles[2..])
}

/// Les offres de la vraie mise en place du bac à sable (spec 050) : l'arène n'en a pas.
fn sandbox_offers() -> Vec<Event> {
    let (_, setup, _) = sandbox_setup();
    setup
        .into_iter()
        .filter(|event| {
            matches!(
                event,
                Event::CommonDeckShuffled { .. } | Event::OfferCardDealt { .. }
            )
        })
        .collect()
}

/// Issue privée 296 : le début du tour en cours, après les événements qui posent la situation (cristaux,
/// unités, sites conquis avant ce tour…), qui sans lui s'afficheraient dans « Ce tour ».
const TURN_STARTED: Event = Event::TurnStarted {
    player: SOLO,
    turn: 1,
};

/// La situation posée par `events`, au début du tour en cours (issue privée 296).
fn turn_start(mut events: Vec<Event>) -> Vec<Event> {
    events.push(TURN_STARTED);
    events
}

/// Issue privée 296 : la gloire et le niveau posés d'avance, fin de tour comprise, pour que leurs
/// bandeaux de niveau (spec 035) ne s'affichent pas ; le tour en cours commence ensuite par
/// `turn_start`. Seulement juste après la mise en place : la fin du tour vide aussi la zone de
/// jeu, les jetons de mana et le combat.
fn settled_progress(fame: u32, level: u8) -> Vec<Event> {
    let mut events = progress(fame, level);
    events.push(Event::TurnEnded { player: SOLO });
    events
}

/// Issue privée 296 : la conquête de la seule partie posée « ce tour » (la ruine des récompenses), après
/// le début du tour ; le reste de la situation avant lui.
fn conquered_this_turn(events: &mut Vec<Event>) {
    let at = events
        .iter()
        .position(|event| matches!(event, Event::SiteConquered { .. }))
        .expect("une conquête posée");
    let conquest = events.remove(at);
    events.push(TURN_STARTED);
    events.push(conquest);
}

/// Spec 041 : bac à sable, campagnes 3 en 2A, 4 en 2B et 5 en A + B (orques sur leurs cases),
/// figurine sur le monastère ; les offres d'actions avancées et de sorts de la vraie mise en place,
/// trois unités et l'action avancée du monastère dans l'offre d'unités, Paysans dans la zone.
fn monastery() -> Vec<Event> {
    let mut events = arena(
        MONASTERY,
        &[
            card(Promise, 1),
            card(Threaten, 1),
            card(Rage, 1),
            card(Determination, 1),
            wound(1),
        ],
        &[(prowlers(), Hex::new(5, -5)), (diggers(), Hex::new(1, -6))],
    );
    for (tile, (i, j)) in [(3, (2, 0)), (4, (0, 2)), (5, (1, 1))] {
        events.push(Event::TileRevealed {
            tile: TileId::Countryside(tile),
            position: TilePosition::from_lattice(i, j),
        });
    }
    let offers: Vec<Event> = sandbox_offers()
        .into_iter()
        .filter(|event| match event {
            Event::CommonDeckShuffled { deck, .. } => !matches!(
                deck,
                arcanum_core::CommonDeck::RegularUnits | arcanum_core::CommonDeck::EliteUnits
            ),
            Event::OfferCardDealt { offer, .. } => *offer != arcanum_core::Offer::Units,
            _ => true,
        })
        .collect();
    events.extend(offers);
    events.extend(unit_events(
        &[UnitCard::new(Unit::Peasants, 1)],
        &[
            UnitCard::new(Unit::Herbalists, 1),
            UnitCard::new(Unit::NorthernMonks, 1),
            UnitCard::new(Unit::Foresters, 1),
        ],
    ));
    let state = events.iter().fold(GameState::default(), apply);
    let top = state.common_deck(arcanum_core::CommonDeck::AdvancedActions)[0];
    events.push(Event::OfferCardDealt {
        offer: arcanum_core::Offer::Units,
        card: top,
    });
    events
}

/// Spec 043 : bac à sable, campagnes 3 en 2A et 4 en 2B explorées, figurine sur `figure`, main
/// `hand` ; les `defenders` posés face cachée sur leur site, puis les sites `conquered` (avant les
/// tuiles : pas de récompense sort due).
fn fortified(
    figure: Hex,
    hand: &[CardId],
    defenders: &[(EnemyToken, Hex)],
    conquered: &[Hex],
) -> Vec<Event> {
    let mut events: Vec<Event> = arena(figure, hand, defenders)
        .into_iter()
        .map(|event| match event {
            Event::EnemyTokenPlaced { token, hex } => Event::DefenderPlaced { token, hex },
            other => other,
        })
        .collect();
    events.extend(
        conquered
            .iter()
            .map(|&hex| Event::SiteConquered { player: SOLO, hex }),
    );
    for (tile, position) in [(3, KEEP_POSITION), (4, MAGE_TOWER_POSITION)] {
        events.push(Event::TileRevealed {
            tile: TileId::Countryside(tile),
            position,
        });
    }
    events
}

/// Les mêmes événements, la partie jouée avec la graine `seed` : seul le hasard à venir change
/// (spec 044, dés de récompense du Sombre Terrier).
fn with_seed(events: Vec<Event>, seed: u64) -> Vec<Event> {
    events
        .into_iter()
        .map(|event| match event {
            Event::GameStarted { config } => Event::GameStarted {
                config: GameConfig { seed, ..config },
            },
            other => other,
        })
        .collect()
}

/// Spec 062 : les offres de la vraie mise en place sans celle des unités ni ses pioches, puis
/// `owned` dans la zone des unités et `offer` dans l'offre d'unités (et la pioche d'élites, sauf
/// celles de l'offre, pour la cité blanche).
fn city_offers(owned: &[UnitCard], offer: &[UnitCard]) -> Vec<Event> {
    let mut events: Vec<Event> = sandbox_offers()
        .into_iter()
        .filter(|event| match event {
            Event::CommonDeckShuffled { deck, .. } => !matches!(
                deck,
                arcanum_core::CommonDeck::RegularUnits | arcanum_core::CommonDeck::EliteUnits
            ),
            Event::OfferCardDealt { offer, .. } => *offer != arcanum_core::Offer::Units,
            _ => true,
        })
        .collect();
    events.extend(unit_events(owned, offer));
    let elites: Vec<arcanum_core::CommonCard> =
        [Unit::AmotepGunners, Unit::AltemGuardians, Unit::AltemMages]
            .into_iter()
            .map(|unit| arcanum_core::CommonCard::Unit(UnitCard::new(unit, 2)))
            .collect();
    events.push(Event::CommonDeckShuffled {
        deck: arcanum_core::CommonDeck::EliteUnits,
        order: elites,
    });
    events
}

fn crystals(colors: &[BasicColor]) -> Vec<Event> {
    colors
        .iter()
        .map(|&color| Event::CrystalGained {
            player: SOLO,
            color,
        })
        .collect()
}

/// Spec 062 : la cité de `tile` conquise (3 Boucliers, le joueur Seigneur), figurine dessus, main
/// Promesse, Menace, Marche, Rage, Détermination ; les offres de la vraie mise en place, Bûcherons
/// et Catapultes dans l'offre d'unités ; un cristal blanc et un rouge.
fn conquered_city(tile: TileId) -> Vec<Event> {
    let mut events = City {
        tile,
        figure: CITY,
        hand: vec![
            card(Promise, 1),
            card(Threaten, 1),
            card(March, 1),
            card(Rage, 1),
            card(Determination, 1),
        ],
        defenders: Vec::new(),
        ..City::default()
    }
    .events();
    events.extend((0..3).map(|_| Event::CityShieldPlaced {
        player: SOLO,
        hex: CITY,
        enemy: guardsmen(),
    }));
    events.push(Event::CityConquered {
        hex: CITY,
        leader: SOLO,
    });
    events.extend(city_offers(
        &[],
        &[
            UnitCard::new(Unit::Foresters, 1),
            UnitCard::new(Unit::Catapults, 1),
        ],
    ));
    events.extend(crystals(&[BasicColor::White, BasicColor::Red]));
    turn_start(events)
}

fn conquered_city_check(runner: &GameRunner) {
    let state = runner.state();
    assert_eq!(state.figure(SOLO), Some(CITY));
    assert!(
        state
            .city(CITY)
            .is_some_and(|city| city.is_conquered() && city.shields.len() == 3)
    );
    assert!(legal_actions(state, SOLO).contains(&Action::Interact));
}

/// Une partie préparée : son nom de fichier, ses événements et ce qu'elle promet au chargement.
struct Prepared {
    name: &'static str,
    events: Vec<Event>,
    /// Les positions des générateurs après la construction, quand elle a tiré du hasard ; sinon
    /// celles de la mise en place.
    rngs: Option<RngPositions>,
    check: fn(&GameRunner),
}

/// Spec 044, issue privée 131 : la fenêtre de la couleur du cristal est longue à atteindre en jouant (deux
/// monstres marron dans le même combat) ; la partie s'arrête donc juste devant. Sombre Terrier
/// conquis ce tour (ses jetons vaincus), Marche jouée de côté, fin du tour lancée avec la graine 15 :
/// dé rouge (cristal rouge), dé or, et le choix de la couleur attend ; ensuite, deux artefacts tirés. Rend aussi les positions des
/// générateurs, pour que le dernier dé (blanc) suive au chargement.
fn spawning_grounds_won() -> (Vec<Event>, RngPositions) {
    let mut events = with_seed(
        Adventure {
            figure: SPAWNING_GROUNDS,
            hand: vec![
                card(Rage, 1),
                card(Determination, 1),
                card(Swiftness, 1),
                card(Stamina, 1),
                card(March, 1),
            ],
            owned: vec![UnitCard::new(Unit::UtemSwordsmen, 1)],
            ..Adventure::default()
        }
        .events(),
        15,
    );
    // La pioche d'artefacts de la vraie mise en place : le Sombre Terrier en fait tirer deux.
    events.extend(sandbox_offers().into_iter().filter(|event| {
        matches!(
            event,
            Event::CommonDeckShuffled {
                deck: arcanum_core::CommonDeck::Artifacts,
                ..
            }
        )
    }));
    events.push(TURN_STARTED);
    events.push(Event::SiteConquered {
        player: SOLO,
        hex: SPAWNING_GROUNDS,
    });
    let mut state = events.iter().fold(GameState::default(), apply);
    let mut rngs = GameRngs::resume(15, &sandbox_setup().2);
    let mut play = |state: &mut GameState, action: Action| {
        let produced = decide(state, SOLO, &action, &mut rngs)
            .unwrap_or_else(|error| panic!("{action:?} doit être légale : {error:?}"));
        *state = produced.iter().fold(state.clone(), apply);
        events.extend(produced);
    };
    play(&mut state, sideways(card(March, 1)));
    let end_turn = legal_actions(&state, SOLO)
        .into_iter()
        .find(|action| matches!(action, Action::EndTurn { .. }))
        .expect("finir le tour est légal");
    play(&mut state, end_turn);
    (events, rngs.positions())
}

/// Spec 045, issue privée 131 : la ruine grise et violette (7, −5) conquise ce tour, les Paysans au seul
/// jeton de commandement, Forestiers et Herboristes dans l'offre ; Marche jouée de côté et fin du
/// tour lancée : la partie s'arrête sur le choix de l'unité de la récompense.
fn ruin_unit_reward() -> (Vec<Event>, RngPositions) {
    let mut events = Ruins {
        token: arcanum_core::RuinToken::GreyViolet,
        conquered: true,
        hand: vec![
            card(March, 1),
            card(Rage, 1),
            card(Determination, 1),
            card(Swiftness, 1),
            card(Stamina, 1),
        ],
        owned: vec![UnitCard::new(Unit::Peasants, 1)],
        offer: vec![
            UnitCard::new(Unit::Foresters, 1),
            UnitCard::new(Unit::Herbalists, 1),
        ],
        ..Ruins::default()
    }
    .events();
    conquered_this_turn(&mut events);
    let mut state = events.iter().fold(GameState::default(), apply);
    let mut rngs = GameRngs::resume(0, &sandbox_setup().2);
    let mut play = |state: &mut GameState, action: Action| {
        let produced = decide(state, SOLO, &action, &mut rngs)
            .unwrap_or_else(|error| panic!("{action:?} doit être légale : {error:?}"));
        *state = produced.iter().fold(state.clone(), apply);
        events.extend(produced);
    };
    play(&mut state, sideways(card(March, 1)));
    let end_turn = legal_actions(&state, SOLO)
        .into_iter()
        .find(|action| matches!(action, Action::EndTurn { .. }))
        .expect("finir le tour est légal");
    play(&mut state, end_turn);
    (events, rngs.positions())
}

/// Première reconnaissance jouée jusqu'au bout depuis `cite-en-vue` (issue privée 98) : Marche, explorer
/// (1, 1) révèle la cité, fin du tour, Endurance au dernier tour puis fin de partie (spec 063).
fn finished_reconnaissance() -> Vec<Event> {
    let mut events = city_in_view();
    let mut state = events.iter().fold(GameState::default(), apply);
    let mut play = |state: &mut GameState, action: Action| {
        let produced = events_of(state, &action);
        *state = produced.iter().fold(state.clone(), apply);
        events.extend(produced);
    };
    let end_turn = |state: &GameState| {
        legal_actions(state, SOLO)
            .into_iter()
            .find(|action| matches!(action, Action::EndTurn { .. }))
            .expect("finir le tour est légal")
    };
    play(&mut state, basic(card(March, 1)));
    play(
        &mut state,
        Action::Explore {
            position: TilePosition::from_lattice(1, 1),
        },
    );
    let action = end_turn(&state);
    play(&mut state, action);
    play(&mut state, basic(card(Stamina, 1)));
    let action = end_turn(&state);
    play(&mut state, action);
    events
}

/// Première reconnaissance, la tuile de cité (royaume 5) prochaine de la pioche ; au village,
/// deux Marche pour explorer l'emplacement (1, 1).
fn city_in_view() -> Vec<Event> {
    Sandbox {
        scenario: Scenario::first_reconnaissance().without_dummy(),
        figure: Some(VILLAGE),
        hand: vec![
            card(March, 1),
            card(March, 2),
            card(Stamina, 1),
            card(Rage, 1),
            card(Promise, 1),
        ],
        next: vec![TileId::Core(5), TileId::Countryside(3), TileId::Core(2)],
        ..Sandbox::default()
    }
    .events()
}

/// Spec 051 : Tovak dans Première reconnaissance, sur le village voisin des Rôdeurs, niveau 3 avec
/// 15 de gloire (niveau 4 à la fin du tour) ; Bretteur de glace, Motivation (retournée) et Magie
/// inutile gagnées, les sept autres compétences de Tovak dans la pile ; les offres de la vraie mise
/// en place. Finir le tour ouvre la fenêtre du choix de compétence.
fn skills() -> Vec<Event> {
    use arcanum_core::{Effect, Skill};
    let owned = [
        Skill::ColdSwordsmanship,
        Skill::MotivationTovak,
        Skill::WhoNeedsMagic,
    ];
    let mut events = Sandbox {
        scenario: Scenario::first_reconnaissance().without_dummy(),
        hero: arcanum_core::Hero::Tovak,
        figure: Some(VILLAGE),
        hand: vec![
            card(March, 1),
            card(March, 2),
            card(Rage, 1),
            card(Swiftness, 1),
            card(Stamina, 1),
        ],
        placed: vec![(prowlers(), ORCS_2)],
        ..Sandbox::default()
    }
    .events();
    // Les niveaux d'abord : sans pile ni offre, le niveau 2 n'attend ni compétence ni carte.
    events.extend(settled_progress(15, 3));
    events.extend(sandbox_offers());
    events.push(Event::SkillPileShuffled {
        player: SOLO,
        order: Skill::ALL
            .into_iter()
            .filter(|skill| skill.hero() == arcanum_core::Hero::Tovak && !owned.contains(skill))
            .collect(),
    });
    events.extend(owned.map(|skill| Event::SkillGained {
        player: SOLO,
        skill,
    }));
    events.push(Event::SkillUsed {
        player: SOLO,
        skill: Skill::MotivationTovak,
        option: 0,
        effect: Effect::DrawAndToken {
            cards: 2,
            token: BasicColor::Blue,
        },
    });
    events
}

/// Spec 058 : `hero` sur le village, voisin de `enemy` (orques ②), la main `hand`, les compétences `owned`
/// gagnées, au niveau `level`, les cristaux `crystals` ; les offres de la vraie mise en place.
fn second_batch(
    hero: arcanum_core::Hero,
    hand: Vec<CardId>,
    owned: &[arcanum_core::Skill],
    level: u8,
    crystals: &[BasicColor],
    enemy: EnemyToken,
) -> Vec<Event> {
    let mut events = Sandbox {
        hero,
        figure: Some(VILLAGE),
        hand,
        placed: vec![(enemy, ORCS_2)],
        ..Sandbox::default()
    }
    .events();
    events.extend(settled_progress(0, level));
    events.extend(sandbox_offers());
    events.extend(owned.iter().map(|&skill| Event::SkillGained {
        player: SOLO,
        skill,
    }));
    events.extend(crystals.iter().map(|&color| Event::CrystalGained {
        player: SOLO,
        color,
    }));
    events
}

/// Spec 060 : Première reconnaissance, Tovak sur le village, `hand` en main ; le choix de la
/// tactique de `time` attend.
fn tactics_decor(time: arcanum_core::TimeOfDay, hand: Vec<CardId>) -> Vec<Event> {
    Sandbox {
        scenario: Scenario::first_reconnaissance().without_dummy(),
        hero: arcanum_core::Hero::Tovak,
        figure: Some(VILLAGE),
        hand,
        time,
        tactic_pending: true,
        ..Sandbox::default()
    }
    .events()
}

fn tactics_hand() -> Vec<CardId> {
    vec![
        card(March, 1),
        card(March, 2),
        card(Stamina, 1),
        card(Rage, 1),
        card(Promise, 1),
    ]
}

/// Les événements `events`, puis ceux des `actions` jouées dans l'ordre.
fn then_played(mut events: Vec<Event>, actions: &[Action]) -> Vec<Event> {
    for action in actions {
        let state = events.iter().fold(GameState::default(), apply);
        events.extend(events_of(&state, action));
    }
    events
}

/// Spec 060 : manche 3 (Jour), Planification et Préparation retirées aux manches d'avant ; le
/// choix de la tactique attend.
fn tactics_third_round() -> Vec<Event> {
    let mut events = tactics_decor(arcanum_core::TimeOfDay::Day, tactics_hand());
    let at = events
        .iter()
        .position(|event| matches!(event, Event::HeroFigurePlaced { .. }))
        .expect("la figurine posée");
    events.splice(
        at..at,
        [
            Event::RoundStarted {
                round: 3,
                time_of_day: arcanum_core::TimeOfDay::Day,
            },
            Event::TacticsRemoved {
                tactics: vec![
                    arcanum_core::Tactic::Planning,
                    arcanum_core::Tactic::Preparation,
                ],
            },
        ],
    );
    events
}

/// Spec 060 : la manche 3 de `tactics_third_round`, Vol de mana pris et le dé bleu posé dessus.
fn tactics_stolen_die() -> Vec<Event> {
    then_played(
        tactics_third_round(),
        &[
            Action::ChooseTactic {
                tactic: arcanum_core::Tactic::ManaSteal,
            },
            Action::StealDie { die: 2 },
        ],
    )
}

/// Spec 060 : manche 2 (Nuit), Provisions prise, deux cartes posées dessous (tours 1 et 2).
fn tactics_night() -> Vec<Event> {
    let mut events = then_played(
        tactics_decor(arcanum_core::TimeOfDay::Night, tactics_hand()),
        &[
            Action::ChooseTactic {
                tactic: arcanum_core::Tactic::SparingPower,
            },
            Action::SparingPower { take: false },
        ],
    );
    events.push(Event::TurnStarted {
        player: SOLO,
        turn: 2,
    });
    then_played(events, &[Action::SparingPower { take: false }])
}

/// Spec 060, Q5 : Longue Nuit prise, deck de 2 cartes, une carte dans la défausse ; Marche jouée
/// de côté et le tour fini en défaussant 3 cartes : la pioche s'arrête sur la fenêtre de Longue
/// Nuit.
fn tactics_long_night() -> Vec<Event> {
    let deck = [card(Swiftness, 1), card(Swiftness, 2)];
    let mut drawn = tactics_hand();
    drawn.push(card(Threaten, 1));
    let mut events = tactics_decor(arcanum_core::TimeOfDay::Night, drawn.clone());
    let mut order = drawn;
    order.extend(deck);
    let at = events
        .iter()
        .position(|event| matches!(event, Event::DeckShuffled { .. }))
        .expect("le deck mélangé");
    events[at] = Event::DeckShuffled {
        player: SOLO,
        order,
    };
    events.push(Event::CardDiscarded {
        player: SOLO,
        card: card(Threaten, 1),
    });
    let events = then_played(
        events,
        &[
            Action::ChooseTactic {
                tactic: arcanum_core::Tactic::LongNight,
            },
            sideways(card(March, 1)),
        ],
    );
    let state = events.iter().fold(GameState::default(), apply);
    let end = legal_actions(&state, SOLO)
        .into_iter()
        .find(|action| {
            matches!(action, Action::EndTurn { discard, .. }
                if *discard == [card(March, 2), card(Stamina, 1), card(Rage, 1)])
        })
        .expect("la fin du tour qui défausse 3 cartes");
    then_played(events, &[end])
}

/// Spec 061 : le deck de départ de `hero` pour le fantôme (exemplaires n° 101 et plus), dans l'ordre
/// du contenu.
fn dummy_deck(hero: arcanum_core::Hero) -> Vec<CardId> {
    hero.starting_deck()
        .into_iter()
        .map(|card| CardId::new(card.card(), card.copy() + arcanum_core::DUMMY_COPY_OFFSET))
        .collect()
}

/// Spec 061 : Première reconnaissance avec fantôme Goldyx (cristaux vert, vert, bleu), Tovak sur le
/// village, la main des tactiques ; les offres de la vraie mise en place ; le choix de la
/// tactique de jour attend.
fn dummy_decor() -> Vec<Event> {
    let mut events = Sandbox {
        scenario: Scenario::first_reconnaissance(),
        hero: arcanum_core::Hero::Tovak,
        figure: Some(VILLAGE),
        hand: tactics_hand(),
        tactic_pending: true,
        ..Sandbox::default()
    }
    .events();
    events.extend(sandbox_offers());
    events.push(dummy_set_up());
    events
}

fn dummy_set_up() -> Event {
    use arcanum_core::Skill;
    Event::DummySetUp {
        hero: arcanum_core::Hero::Goldyx,
        crystals: vec![BasicColor::Green, BasicColor::Green, BasicColor::Blue],
        skills: Skill::ALL
            .into_iter()
            .filter(|skill| skill.hero() == arcanum_core::Hero::Goldyx)
            .collect(),
        deck: dummy_deck(arcanum_core::Hero::Goldyx),
    }
}

/// Spec 061 : `dummy_decor`, le fantôme a déjà retourné 14 cartes (4 tours de 3, puis 2) ; le
/// joueur prend Lève-tôt et joue en premier : deck du fantôme à 2 cartes, début du tour 1.
fn dummy_announcing() -> Vec<Event> {
    let mut events = dummy_decor();
    let deck = dummy_deck(arcanum_core::Hero::Goldyx);
    events.extend(
        [0..3, 3..6, 6..9, 9..12, 12..14].map(|range| Event::DummyTurnPlayed {
            cards: deck[range].to_vec(),
        }),
    );
    then_played(
        events,
        &[Action::ChooseTactic {
            tactic: arcanum_core::Tactic::EarlyBird,
        }],
    )
}

/// Un tour de remplissage : la première carte jouée de côté, puis la première fin de tour.
fn filler_turn(state: &GameState) -> GameState {
    let card = state
        .hand(SOLO)
        .iter()
        .copied()
        .find(|card| !card.card().is_wound())
        .expect("une carte à jouer");
    let state = done(state, sideways(card));
    let end = legal_actions(&state, SOLO)
        .into_iter()
        .find(|action| matches!(action, Action::EndTurn { .. }))
        .expect("finir le tour est légal");
    done(&state, end)
}

/// Spec 061 : `dummy_decor` au niveau 1 avec 1 gloire, la pile de compétences de Tovak mélangée,
/// Lève-tôt prise ; les Rôdeurs à côté : les vaincre (+2) mène au niveau 2 à la fin du tour.
fn dummy_skill() -> Vec<Event> {
    use arcanum_core::Skill;
    let mut events = Sandbox {
        scenario: Scenario::first_reconnaissance(),
        hero: arcanum_core::Hero::Tovak,
        figure: Some(VILLAGE),
        hand: vec![
            card(Swiftness, 1),
            card(March, 1),
            card(Rage, 1),
            card(Promise, 1),
            card(Stamina, 1),
        ],
        placed: vec![(prowlers(), ORCS_2)],
        tactic_pending: true,
        ..Sandbox::default()
    }
    .events();
    events.extend(settled_progress(1, 1));
    events.extend(sandbox_offers());
    events.push(Event::SkillPileShuffled {
        player: SOLO,
        order: Skill::ALL
            .into_iter()
            .filter(|skill| skill.hero() == arcanum_core::Hero::Tovak)
            .collect(),
    });
    events.push(dummy_set_up());
    then_played(
        events,
        &[Action::ChooseTactic {
            tactic: arcanum_core::Tactic::EarlyBird,
        }],
    )
}

// --- Conquête en solitaire (spec 064) ---

/// La main des parties de Conquête en solitaire : Marche, Détermination, Rage, Célérité, Endurance.
fn conquest_hand() -> Vec<CardId> {
    vec![
        card(March, 1),
        card(Determination, 1),
        card(Rage, 1),
        card(Swiftness, 1),
        card(Stamina, 1),
    ]
}

/// Spec 064 : les tuiles de campagne 3 à 7 posées en (1, 1), (0, 2), (1, 2), (0, 3) et (1, 3) ;
/// la prochaine est la tuile royaume 2. Figurine sur la colline E de (1, 1), (5, −5) : elle touche
/// (2, 0), sur la côte (refusée à la tuile royaume), et (2, 1), à l'intérieur (permise).
fn conquest_coast() -> Vec<Event> {
    let mut next: Vec<TileId> = (3..=7).map(TileId::Countryside).collect();
    next.extend([2, 6, 3, 7].map(TileId::Core));
    let mut events = Sandbox {
        scenario: Scenario::solo_conquest().without_dummy(),
        figure: Some(Hex::new(5, -5)),
        hand: conquest_hand(),
        next,
        ..Sandbox::default()
    }
    .events();
    events.extend(
        [(1, 1), (0, 2), (1, 2), (0, 3), (1, 3)]
            .into_iter()
            .zip(3..)
            .map(|((i, j), tile)| Event::TileRevealed {
                tile: TileId::Countryside(tile),
                position: TilePosition::from_lattice(i, j),
            }),
    );
    turn_start(events)
}

/// La cité bleue (royaume 6) posée en (2, 1) et conquise, 3 Boucliers du joueur : la première.
fn first_city_conquered() -> Vec<Event> {
    let hex = Hex::new(7, -7);
    let mut events = vec![
        Event::TileRevealed {
            tile: TileId::Core(6),
            position: TilePosition::from_lattice(2, 1),
        },
        Event::CityPlaced {
            hex,
            color: BasicColor::Blue,
            level: 5,
        },
    ];
    events.extend((0..3).map(|_| Event::CityShieldPlaced {
        player: SOLO,
        hex,
        enemy: guardsmen(),
    }));
    events.push(Event::CityConquered { hex, leader: SOLO });
    events
}

/// La manche `round` de `events` : la manche 1 de jour (ou 2 de nuit) de l'arène devient `round`.
fn at_round(
    mut events: Vec<Event>,
    round: u32,
    time_of_day: arcanum_core::TimeOfDay,
) -> Vec<Event> {
    events.retain(|event| !matches!(event, Event::RoundStarted { .. }));
    // Juste après les deux tuiles de la mise en place, comme la manche de nuit de l'arène.
    events.insert(5, Event::RoundStarted { round, time_of_day });
    events
}

/// Le choix de la tactique en attente : la tactique posée par l'arène retirée, le fantôme mis en
/// place, puis le joueur prend `tactic` (le fantôme tire la sienne).
fn with_dummy_tactic(mut events: Vec<Event>, tactic: arcanum_core::Tactic) -> Vec<Event> {
    events.retain(|event| !matches!(event, Event::TacticChosen { .. }));
    events.push(dummy_set_up());
    then_played(events, &[Action::ChooseTactic { tactic }])
}

/// Spec 064 : manche 3 de jour ; la cité bleue conquise ; la cité rouge niveau 8 (royaume 8) en
/// A + B, défendue par le seul Minotaure, face visible, 3 Boucliers du joueur dessus ; figurine
/// voisine (3, −4) ; cristaux bleu et rouge ; fantôme Goldyx.
fn conquest_last_city() -> Vec<Event> {
    let mut events: Vec<Event> = City {
        scenario: Scenario::solo_conquest(),
        tile: TileId::Core(8),
        figure: NEAR_CITY,
        hand: conquest_hand(),
        defenders: vec![token(arcanum_core::Enemy::Minotaur, 1)],
        face_up: true,
        ..City::default()
    }
    .events()
    .into_iter()
    .map(|event| match event {
        Event::CityPlaced { hex, color, .. } => Event::CityPlaced {
            hex,
            color,
            level: 8,
        },
        other => other,
    })
    .collect();
    events.extend(first_city_conquered());
    events.extend((0..3).map(|_| Event::CityShieldPlaced {
        player: SOLO,
        hex: CITY,
        enemy: guardsmen(),
    }));
    events.extend(crystals(&[BasicColor::Blue, BasicColor::Red]));
    let events = at_round(events, 3, arcanum_core::TimeOfDay::Day);
    with_dummy_tactic(events, arcanum_core::Tactic::EarlyBird)
}

/// `conquete-derniere-cite` jouée jusqu'au bout : Marche, assaut, le Minotaure vaincu (posé en
/// phase d'attaque : le combat se vérifie à l'œil depuis `conquete-derniere-cite`), fin du combat,
/// fin du tour, puis le dernier tour (Célérité de côté) et sa fin.
fn conquest_won() -> Vec<Event> {
    let mut events = conquest_last_city();
    let mut state = events.iter().fold(GameState::default(), apply);
    let play = |state: &mut GameState, events: &mut Vec<Event>, action: Action| {
        let produced = events_of(state, &action);
        *state = produced.iter().fold(state.clone(), apply);
        events.extend(produced);
    };
    let end_turn = |state: &GameState| {
        legal_actions(state, SOLO)
            .into_iter()
            .find(|action| matches!(action, Action::EndTurn { .. }))
            .expect("finir le tour est légal")
    };
    play(&mut state, &mut events, basic(card(March, 1)));
    play(&mut state, &mut events, Action::MoveHero { to: CITY });
    play(&mut state, &mut events, Action::StartCombat);
    let minotaur = token(arcanum_core::Enemy::Minotaur, 1);
    let posed = [
        Event::CombatPhaseStarted {
            player: SOLO,
            phase: arcanum_core::CombatPhase::Attack,
        },
        Event::EnemiesDefeated {
            player: SOLO,
            enemies: vec![minotaur],
            fame: u32::from(minotaur.enemy().definition().fame),
        },
    ];
    for event in posed {
        state = apply(state, &event);
        events.push(event);
    }
    play(&mut state, &mut events, Action::EndCombat);
    let action = end_turn(&state);
    play(&mut state, &mut events, action);
    play(&mut state, &mut events, sideways(card(Swiftness, 1)));
    let action = end_turn(&state);
    play(&mut state, &mut events, action);
    events
}

/// Spec 064 : nuit, manche 6 ; la cité bleue conquise ; le fantôme a retourné tout son deck et
/// annoncé la fin de la manche : ce tour est le dernier du joueur.
fn conquest_failed() -> Vec<Event> {
    let mut events = Sandbox {
        scenario: Scenario::solo_conquest(),
        figure: Some(VILLAGE),
        hand: conquest_hand(),
        time: arcanum_core::TimeOfDay::Night,
        ..Sandbox::default()
    }
    .events();
    events.extend(first_city_conquered());
    let events = at_round(events, 6, arcanum_core::TimeOfDay::Night);
    let mut events = with_dummy_tactic(events, arcanum_core::Tactic::FromTheDusk);
    let deck = dummy_deck(arcanum_core::Hero::Goldyx);
    events.extend(deck.chunks(3).map(|cards| Event::DummyTurnPlayed {
        cards: cards.to_vec(),
    }));
    events.push(Event::DummyAnnouncedEndOfRound);
    events
}

fn prepared() -> Vec<Prepared> {
    vec![
        Prepared {
            name: "combat-deux-ennemis",
            rngs: None,
            events: arena(
                PLAIN_W,
                &[
                    card(March, 1),
                    card(March, 2),
                    card(Promise, 1),
                    card(Concentration, 1),
                    card(Rage, 1),
                ],
                &[(prowlers(), ORCS_2), (diggers(), HILL)],
            ),
            check: |runner| {
                // MKUE p. 8, 2.c : les deux ennemis voisins peuvent être défiés dès le début du tour.
                let legal = legal_actions(runner.state(), SOLO);
                for enemy in [ORCS_2, HILL] {
                    assert!(
                        legal.contains(&Action::Challenge { enemy }),
                        "défier {enemy:?} : {legal:?}"
                    );
                }
            },
        },
        Prepared {
            name: "combat-elements",
            rngs: None,
            events: {
                // Spec 032 : Cogneurs (résistance physique) et Dragons de glace voisins ; un cristal
                // rouge et un bleu pour renforcer deux cartes dans le tour.
                let mut events = arena(
                    PLAIN_W,
                    &[
                        card(BattleVersatility, 1),
                        card(Rage, 1),
                        card(Determination, 1),
                        card(Stamina, 1),
                        card(March, 1),
                    ],
                    &[
                        (EnemyToken::new(arcanum_core::Enemy::Ironclads, 1), ORCS_2),
                        (EnemyToken::new(arcanum_core::Enemy::IceDragon, 1), HILL),
                    ],
                );
                events.extend([BasicColor::Red, BasicColor::Blue].map(|color| {
                    Event::CrystalGained {
                        player: SOLO,
                        color,
                    }
                }));
                turn_start(events)
            },
            check: |runner| {
                let legal = legal_actions(runner.state(), SOLO);
                for enemy in [ORCS_2, HILL] {
                    assert!(legal.contains(&Action::Challenge { enemy }));
                }
            },
        },
        Prepared {
            name: "combat-capacites",
            rngs: None,
            events: {
                // Spec 033 : Invocateurs orques (la créature tirée sera un Minotaure, brutal) et
                // Chevaucheurs de loups (vifs) voisins ; un cristal bleu pour Détermination renforcée.
                let mut events = arena(
                    PLAIN_W,
                    &[
                        card(Determination, 1),
                        card(Rage, 1),
                        card(March, 1),
                        card(Stamina, 1),
                        card(Swiftness, 1),
                    ],
                    &[(orc_summoners(), ORCS_2), (wolf_riders(), HILL)],
                );
                let minotaur = EnemyToken::new(arcanum_core::Enemy::Minotaur, 1);
                let mut brown = vec![minotaur];
                brown.extend(
                    arcanum_core::GameContent::embedded()
                        .enemy_tokens(arcanum_core::EnemyColor::Brown)
                        .into_iter()
                        .filter(|token| *token != minotaur),
                );
                events.push(Event::EnemyPileShuffled {
                    color: arcanum_core::EnemyColor::Brown,
                    order: brown,
                });
                events.push(Event::CrystalGained {
                    player: SOLO,
                    color: BasicColor::Blue,
                });
                turn_start(events)
            },
            check: |runner| {
                let legal = legal_actions(runner.state(), SOLO);
                for enemy in [ORCS_2, HILL] {
                    assert!(legal.contains(&Action::Challenge { enemy }));
                }
            },
        },
        Prepared {
            name: "main-blessures",
            rngs: None,
            events: arena(
                VILLAGE,
                &[
                    wound(1),
                    wound(2),
                    card(March, 1),
                    card(Promise, 1),
                    card(Rage, 1),
                ],
                &[],
            ),
            check: |runner| {
                let hand = runner.state().hand(SOLO);
                assert_eq!(hand.len(), 5);
                assert!(hand.contains(&wound(1)) && hand.contains(&wound(2)));
            },
        },
        Prepared {
            // Spec 034 : soin et repos.
            name: "soins",
            rngs: None,
            events: arena(
                VILLAGE,
                &[
                    wound(1),
                    wound(2),
                    wound(3),
                    card(Tranquility, 1),
                    card(Rage, 1),
                ],
                &[],
            ),
            check: |runner| {
                let hand = runner.state().hand(SOLO);
                assert_eq!(hand.len(), 5);
                assert!(hand.contains(&card(Tranquility, 1)));
                assert!(runner.state().can_rest(SOLO));
            },
        },
        Prepared {
            // Spec 034 : un Minotaure (brutal, 5 Blessures non paré) assomme le héros.
            name: "combat-assomme",
            rngs: None,
            events: arena(
                PLAIN_W,
                &[
                    card(March, 1),
                    card(March, 2),
                    card(Stamina, 1),
                    card(Promise, 1),
                    card(Rage, 1),
                ],
                &[(EnemyToken::new(arcanum_core::Enemy::Minotaur, 1), ORCS_2)],
            ),
            check: |runner| {
                let legal = legal_actions(runner.state(), SOLO);
                assert!(legal.contains(&Action::Challenge { enemy: ORCS_2 }));
            },
        },
        Prepared {
            // Spec 035 : gloire 6, niveau 2 ; vaincre les Rôdeurs (+2) mène au niveau 3.
            name: "niveau-3",
            rngs: None,
            events: {
                let mut events = arena(
                    VILLAGE,
                    &[
                        card(Swiftness, 1),
                        card(March, 1),
                        card(Rage, 1),
                        card(Promise, 1),
                        card(Stamina, 1),
                    ],
                    &[(prowlers(), ORCS_2)],
                );
                events.extend(settled_progress(6, 2));
                turn_start(events)
            },
            check: |runner| {
                let state = runner.state();
                assert_eq!((state.fame(SOLO), state.level(SOLO)), (6, 2));
                let legal = legal_actions(state, SOLO);
                assert!(legal.contains(&Action::Challenge { enemy: ORCS_2 }));
            },
        },
        Prepared {
            // Spec 050 : gloire 1, niveau 1 ; vaincre les Rôdeurs (+2) mène au niveau 2, et la fin
            // du tour ouvre la fenêtre du niveau pair : une compétence, puis une action avancée
            // de l'offre (spec #20).
            name: "niveau-pair",
            rngs: None,
            events: {
                let mut events = arena(
                    VILLAGE,
                    &[
                        card(Swiftness, 1),
                        card(March, 1),
                        card(Rage, 1),
                        card(Promise, 1),
                        card(Stamina, 1),
                    ],
                    &[(prowlers(), ORCS_2)],
                );
                events.extend(sandbox_offers());
                events.extend(settled_progress(1, 1));
                turn_start(events)
            },
            check: |runner| {
                let state = runner.state();
                assert_eq!((state.fame(SOLO), state.level(SOLO)), (1, 1));
                assert_eq!(state.offer(arcanum_core::Offer::AdvancedActions).len(), 3);
                let legal = legal_actions(state, SOLO);
                assert!(legal.contains(&Action::Challenge { enemy: ORCS_2 }));
            },
        },
        Prepared {
            // Spec 051 : trois compétences gagnées, le niveau 4 attend la fin du tour (fenêtre du
            // niveau pair, spec #20).
            name: "competences",
            rngs: None,
            events: turn_start(skills()),
            check: |runner| {
                let state = runner.state();
                assert_eq!((state.fame(SOLO), state.level(SOLO)), (15, 3));
                assert_eq!(state.skills(SOLO).len(), 3);
                assert_eq!(state.skill_pile(SOLO).len(), 7);
                assert!(
                    state
                        .skills(SOLO)
                        .iter()
                        .any(|owned| owned.state == arcanum_core::SkillState::Flipped)
                );
                let legal = legal_actions(state, SOLO);
                assert!(legal.contains(&Action::Challenge { enemy: ORCS_2 }));
            },
        },
        Prepared {
            // Spec 058 : Tovak, cartes de côté (Flexibilité, Magie inutile), Botte secrète contre
            // les Cogneurs (résistants au physique), Surcharge de mana ; une Blessure en main.
            name: "competences-cote",
            rngs: None,
            events: {
                use arcanum_core::Skill::*;
                turn_start(second_batch(
                    arcanum_core::Hero::Tovak,
                    vec![
                        wound(1),
                        card(Rage, 1),
                        card(March, 1),
                        card(Stamina, 1),
                        card(Swiftness, 1),
                    ],
                    &[IDontGiveADamn, WhoNeedsMagic, ResistanceBreak, ManaOverload],
                    1,
                    &[],
                    ironclads(),
                ))
            },
            check: |runner| {
                let state = runner.state();
                assert_eq!(state.hero(SOLO), Some(arcanum_core::Hero::Tovak));
                assert_eq!(state.skills(SOLO).len(), 4);
                assert!(legal_actions(state, SOLO).contains(&Action::Challenge { enemy: ORCS_2 }));
            },
        },
        Prepared {
            // Spec 058 : Goldyx près du lac, Pouvoir universel, Vol, Embrasure de la Source,
            // Fortune clinquante ; cristaux rouge, bleu et blanc.
            name: "competences-goldyx",
            rngs: None,
            events: {
                use arcanum_core::Skill::*;
                turn_start(second_batch(
                    arcanum_core::Hero::Goldyx,
                    vec![
                        card(March, 1),
                        card(Rage, 1),
                        card(Promise, 1),
                        card(Stamina, 1),
                        card(Swiftness, 1),
                    ],
                    &[UniversalPower, Flight, SourceOpening, GlitteringFortune],
                    1,
                    &[BasicColor::Red, BasicColor::Blue, BasicColor::White],
                    prowlers(),
                ))
            },
            check: |runner| {
                let state = runner.state();
                assert_eq!(state.hero(SOLO), Some(arcanum_core::Hero::Goldyx));
                assert_eq!(state.skills(SOLO).len(), 4);
                assert!(legal_actions(state, SOLO).contains(&Action::Interact));
            },
        },
        Prepared {
            // Spec 058 : Norowas niveau 5, Charisme, Marche en avant, Loyauté sans faille, Temps
            // calme ; trois unités prêtes, deux unités dans l'offre pour le jeton de Loyauté.
            name: "competences-norowas",
            rngs: None,
            events: {
                use arcanum_core::Skill::*;
                let mut events = second_batch(
                    arcanum_core::Hero::Norowas,
                    vec![
                        card(Promise, 1),
                        card(Threaten, 1),
                        card(March, 1),
                        card(Rage, 1),
                        card(Determination, 1),
                    ],
                    &[Leadership, ForwardMarch, BondsOfLoyalty, CalmingTheWeather],
                    5,
                    &[],
                    prowlers(),
                );
                events.extend(unit_events(
                    &[
                        UnitCard::new(Unit::Peasants, 1),
                        UnitCard::new(Unit::UtemCrossbowmen, 1),
                        UnitCard::new(Unit::Foresters, 1),
                    ],
                    &[
                        UnitCard::new(Unit::Herbalists, 1),
                        UnitCard::new(Unit::UtemGuardsmen, 1),
                    ],
                ));
                turn_start(events)
            },
            check: |runner| {
                let state = runner.state();
                assert_eq!(state.hero(SOLO), Some(arcanum_core::Hero::Norowas));
                assert_eq!(state.units(SOLO).len(), 3);
                assert_eq!(state.bonds_unit(SOLO), None);
                let legal = legal_actions(state, SOLO);
                assert!(legal.contains(&Action::Interact));
                assert!(legal.contains(&Action::Challenge { enemy: ORCS_2 }));
            },
        },
        Prepared {
            // Spec 058 : Arythea, Souffrance sublimée, Invocation, Polarisation, Rituel de
            // guérison ; deux Blessures en main, un cristal bleu.
            name: "competences-arythea",
            rngs: None,
            events: {
                use arcanum_core::Skill::*;
                turn_start(second_batch(
                    arcanum_core::Hero::Arythea,
                    vec![
                        wound(1),
                        wound(2),
                        card(Rage, 1),
                        card(March, 1),
                        card(Swiftness, 1),
                    ],
                    &[PowerOfPain, Invocation, Polarization, RitualOfPain],
                    1,
                    &[BasicColor::Blue],
                    prowlers(),
                ))
            },
            check: |runner| {
                let state = runner.state();
                assert_eq!(state.hero(SOLO), Some(arcanum_core::Hero::Arythea));
                assert_eq!(state.skills(SOLO).len(), 4);
                assert!(legal_actions(state, SOLO).contains(&Action::Challenge { enemy: ORCS_2 }));
            },
        },
        Prepared {
            // Spec 040 : Norowas sur le village, voisin des Rôdeurs ; un cristal blanc et un rouge.
            name: "interaction",
            rngs: None,
            events: {
                let mut events: Vec<Event> = arena(
                    VILLAGE,
                    &[
                        card(NobleManners, 1),
                        card(Threaten, 1),
                        card(Promise, 1),
                        card(March, 1),
                        card(Rage, 1),
                    ],
                    &[(prowlers(), ORCS_2)],
                )
                .into_iter()
                .map(|event| match event {
                    Event::GameStarted { config } => Event::GameStarted {
                        config: GameConfig {
                            hero: arcanum_core::Hero::Norowas,
                            ..config
                        },
                    },
                    other => other,
                })
                .collect();
                events.extend([BasicColor::White, BasicColor::Red].map(|color| {
                    Event::CrystalGained {
                        player: SOLO,
                        color,
                    }
                }));
                turn_start(events)
            },
            check: |runner| {
                let state = runner.state();
                assert_eq!(state.hero(SOLO), Some(arcanum_core::Hero::Norowas));
                let legal = legal_actions(state, SOLO);
                assert!(legal.contains(&Action::Interact));
                assert!(legal.contains(&Action::Challenge { enemy: ORCS_2 }));
            },
        },
        Prepared {
            // Spec 042 : niveau 3 sur le village, voisin des Rôdeurs ; Paysans disponibles et
            // Golems gardiens blessés dans la zone, Gardes d'Utem dans l'offre ; un cristal vert.
            name: "unites",
            rngs: None,
            events: {
                let mut events = arena(
                    VILLAGE,
                    &[
                        card(Promise, 1),
                        card(Threaten, 1),
                        card(Tranquility, 1),
                        card(Rage, 1),
                        card(Determination, 1),
                    ],
                    &[(prowlers(), ORCS_2)],
                );
                events.extend(settled_progress(0, 3));
                let golems = UnitCard::new(Unit::GuardianGolems, 1);
                events.extend(unit_events(
                    &[UnitCard::new(Unit::Peasants, 1), golems],
                    &[UnitCard::new(Unit::UtemGuardsmen, 1)],
                ));
                events.push(Event::UnitWounded {
                    player: SOLO,
                    unit: golems,
                    wounds: 1,
                });
                events.push(Event::CrystalGained {
                    player: SOLO,
                    color: BasicColor::Green,
                });
                turn_start(events)
            },
            check: |runner| {
                let state = runner.state();
                assert_eq!(state.units(SOLO).len(), 2);
                assert!(state.units(SOLO)[1].is_wounded());
                assert_eq!(state.command_tokens(SOLO), 2);
                let legal = legal_actions(state, SOLO);
                assert!(legal.contains(&Action::Interact));
                assert!(legal.contains(&Action::Challenge { enemy: ORCS_2 }));
            },
        },
        Prepared {
            // Spec 041 : campagnes 3, 4 et 5 explorées, figurine sur le monastère de la
            // campagne 5 ; une action avancée dans l'offre d'unités, Paysans dans la zone.
            name: "monastere",
            rngs: None,
            events: turn_start(monastery()),
            check: |runner| {
                let state = runner.state();
                assert_eq!(state.figure(SOLO), Some(MONASTERY));
                assert_eq!(state.reputation(SOLO).value(), 0);
                assert!(
                    state
                        .offer(arcanum_core::Offer::Units)
                        .iter()
                        .any(|card| card.deed().is_some())
                );
                assert_eq!(state.units(SOLO).len(), 1);
                let legal = legal_actions(state, SOLO);
                assert!(legal.contains(&Action::Interact));
                assert!(legal.contains(&Action::BurnMonastery));
            },
        },
        Prepared {
            // Spec 043 : à deux pas de la forteresse, Gardes face cachée ; de quoi marcher (6),
            // parer et attaquer (Rage et Détermination, un cristal rouge et un bleu, Épéistes
            // d'Utem) ; la tour de mage garde ses Mages de feu cachés.
            name: "forteresse",
            rngs: None,
            events: {
                let mut events = fortified(
                    TWO_FROM_KEEP,
                    &[
                        card(March, 1),
                        card(Stamina, 1),
                        card(Swiftness, 1),
                        card(Rage, 1),
                        card(Determination, 1),
                    ],
                    &[
                        (guardsmen(), KEEP),
                        (
                            EnemyToken::new(arcanum_core::Enemy::FireMages, 1),
                            MAGE_TOWER,
                        ),
                    ],
                    &[],
                );
                events.extend(unit_events(&[UnitCard::new(Unit::UtemSwordsmen, 1)], &[]));
                events.extend([BasicColor::Red, BasicColor::Blue].map(|color| {
                    Event::CrystalGained {
                        player: SOLO,
                        color,
                    }
                }));
                turn_start(events)
            },
            check: |runner| {
                let state = runner.state();
                assert!(state.is_face_down(KEEP) && state.is_face_down(MAGE_TOWER));
                assert_eq!(state.figure(SOLO), Some(TWO_FROM_KEEP));
                assert_eq!(state.units(SOLO).len(), 1);
            },
        },
        Prepared {
            // Spec 043 : forteresse conquise, tour de mage conquise ce tour (la fenêtre du sort en
            // récompense s'ouvre à la fin du tour), figurine sur la tour ; offres de la vraie mise
            // en place, Golems gardiens, Illusionnistes et Épéistes d'Utem dans l'offre d'unités ;
            // de l'influence en main, un cristal rouge et un blanc.
            name: "tour-conquise",
            rngs: None,
            events: {
                let mut events = fortified(
                    MAGE_TOWER,
                    &[
                        card(Promise, 1),
                        card(Threaten, 1),
                        card(Rage, 1),
                        card(March, 1),
                        card(Determination, 1),
                    ],
                    &[],
                    &[KEEP],
                );
                events.extend(sandbox_offers().into_iter().filter(|event| match event {
                    Event::CommonDeckShuffled { deck, .. } => !matches!(
                        deck,
                        arcanum_core::CommonDeck::RegularUnits
                            | arcanum_core::CommonDeck::EliteUnits
                    ),
                    Event::OfferCardDealt { offer, .. } => *offer != arcanum_core::Offer::Units,
                    _ => true,
                }));
                events.extend(unit_events(
                    &[],
                    &[
                        UnitCard::new(Unit::GuardianGolems, 1),
                        UnitCard::new(Unit::Illusionists, 1),
                        UnitCard::new(Unit::UtemSwordsmen, 1),
                    ],
                ));
                events.extend([BasicColor::Red, BasicColor::White].map(|color| {
                    Event::CrystalGained {
                        player: SOLO,
                        color,
                    }
                }));
                events.push(TURN_STARTED);
                events.push(Event::SiteConquered {
                    player: SOLO,
                    hex: MAGE_TOWER,
                });
                events
            },
            check: |runner| {
                let state = runner.state();
                assert_eq!(state.conquered_sites().len(), 2);
                assert_eq!(state.spell_rewards(SOLO), 1);
                assert_eq!(state.offer(arcanum_core::Offer::Spells).len(), 3);
                assert!(legal_actions(state, SOLO).contains(&Action::Interact));
            },
        },
        Prepared {
            // Spec 044 : campagne 7 en 2A, figurine sur son donjon (6, −3) ; un Minotaure en tête
            // de la pile marron ; Paysans dans la zone, un jeton or, un cristal rouge et un bleu :
            // parer (Détermination renforcée) et attaquer (Rage renforcée, une carte de côté).
            name: "donjon",
            rngs: None,
            events: {
                let mut events = Adventure {
                    hand: vec![
                        card(Determination, 1),
                        card(Rage, 1),
                        card(March, 1),
                        card(Swiftness, 1),
                        card(Stamina, 1),
                    ],
                    owned: vec![UnitCard::new(Unit::Peasants, 1)],
                    ..Adventure::default()
                }
                .events();
                events.push(Event::ManaTokenGained {
                    player: SOLO,
                    color: arcanum_core::ManaColor::Gold,
                });
                events.extend([BasicColor::Red, BasicColor::Blue].map(|color| {
                    Event::CrystalGained {
                        player: SOLO,
                        color,
                    }
                }));
                turn_start(events)
            },
            check: |runner| {
                let state = runner.state();
                assert_eq!(state.figure(SOLO), Some(DUNGEON));
                assert_eq!(state.units(SOLO).len(), 1);
                assert!(legal_actions(state, SOLO).contains(&Action::EnterAdventureSite));
            },
        },
        Prepared {
            // Spec 044 : tuile royaume 1 en 2B, Sombre Terrier (1, −6) conquis ce tour, fin du
            // tour lancée ; graine 15 : dés rouge, or, et la fenêtre de la couleur attend.
            name: "sombre-terrier",
            events: spawning_grounds_won().0,
            rngs: Some(spawning_grounds_won().1),
            // Issue privée 131 : la partie s'arrête sur l'écran visé, la fenêtre de la couleur du
            // cristal ; choisir lance le dernier dé (blanc), tiré de la suite du hasard sauvegardée.
            check: |runner| {
                let state = runner.state();
                assert_eq!(state.figure(SOLO), Some(SPAWNING_GROUNDS));
                assert_eq!(state.shields().get(&SPAWNING_GROUNDS), Some(&SOLO));
                assert_eq!(state.crystal_choices(SOLO), 1);
                assert_eq!(
                    legal_actions(state, SOLO),
                    [
                        BasicColor::Red,
                        BasicColor::Blue,
                        BasicColor::White,
                        BasicColor::Green
                    ]
                    .map(|color| Action::ChooseCrystalColor { color })
                );
                let choice = Action::ChooseCrystalColor {
                    color: BasicColor::Green,
                };
                let mut rngs = GameRngs::resume(15, &runner.rng_positions());
                let chosen = decide(state, SOLO, &choice, &mut rngs).expect("légale");
                assert!(
                    chosen.contains(&Event::RewardDieRolled {
                        player: SOLO,
                        color: arcanum_core::ManaColor::Basic(BasicColor::White),
                    }),
                    "{chosen:?}"
                );
                // Puis les deux artefacts tirés attendent le choix (MKUE p. 11, 5).
                let after = chosen.iter().fold(state.clone(), apply);
                let takes = legal_actions(&after, SOLO);
                assert_eq!(takes.len(), 2, "{takes:?}");
                assert!(
                    takes
                        .iter()
                        .all(|action| matches!(action, Action::TakeArtifact { .. }))
                );
            },
        },
        Prepared {
            // Spec 044 : figurine sur la clairière de la campagne 1 (1, −3), une Blessure en main
            // et une dans la défausse : finir le tour propose de la jeter ; le tour suivant
            // commence avec un jeton de mana or.
            name: "clairiere",
            rngs: None,
            events: {
                let mut events = Adventure {
                    figure: GLADE,
                    hand: vec![
                        card(March, 1),
                        card(Rage, 1),
                        card(Promise, 1),
                        wound(1),
                        card(Stamina, 1),
                    ],
                    ..Adventure::default()
                }
                .events();
                events.push(Event::CardDrawn {
                    player: SOLO,
                    card: wound(2),
                });
                events.push(Event::CardDiscarded {
                    player: SOLO,
                    card: wound(2),
                });
                turn_start(events)
            },
            check: |runner| {
                let state = runner.state();
                assert_eq!(state.figure(SOLO), Some(GLADE));
                assert!(state.hand(SOLO).contains(&wound(1)));
                assert!(state.discard_pile(SOLO).contains(&wound(2)));
            },
        },
        Prepared {
            // Spec 052, exemple : figurine sur le village de la campagne 1, Pont de glace en main
            // (dé bleu dans la Source pour le renforcer), Rôdeurs sur (1, −4) : entrer dans le lac
            // (2, −4), défier, finir le tour sur le lac (retraite forcée) ; Temporisation et
            // Intimidation pour la fin du tour.
            name: "lac",
            rngs: None,
            events: arena(
                VILLAGE,
                &[
                    advanced(arcanum_core::AdvancedAction::FrostBridge),
                    advanced(arcanum_core::AdvancedAction::SteadyTempo),
                    advanced(arcanum_core::AdvancedAction::Intimidate),
                    card(March, 1),
                    card(Rage, 1),
                ],
                &[(prowlers(), ORCS_1)],
            ),
            check: |runner| {
                let state = runner.state();
                assert_eq!(state.figure(SOLO), Some(VILLAGE));
                let frost_bridge = advanced(arcanum_core::AdvancedAction::FrostBridge);
                assert!(legal_actions(state, SOLO).iter().any(|action| matches!(
                    action,
                    Action::PlayCard { card, play: arcanum_core::CardPlay::Strong { .. } }
                        if *card == frost_bridge
                )));
            },
        },
        Prepared {
            // Spec 052 : Agilité, Embuscade et Diplomatie en main sur le village, Rôdeurs voisins
            // en (3, −3) : jouer les cartes, défier, convertir le Mouvement en Attaque et
            // l'Influence en Parade.
            name: "agilite",
            rngs: None,
            events: arena(
                VILLAGE,
                &[
                    advanced(arcanum_core::AdvancedAction::Agility),
                    advanced(arcanum_core::AdvancedAction::Ambush),
                    advanced(arcanum_core::AdvancedAction::Diplomacy),
                    card(March, 1),
                    card(Promise, 1),
                ],
                &[(prowlers(), ORCS_2)],
            ),
            check: |runner| {
                let legal = legal_actions(runner.state(), SOLO);
                assert!(legal.contains(&Action::Challenge { enemy: ORCS_2 }));
            },
        },
        Prepared {
            // Spec 053, exemple : Rôdeurs et Invocateurs orques ensemble en (3, −3) ; Illusionnistes
            // et Paysans disponibles ; Bouclier de glace, Au cœur du combat, Coup de sang, Magie
            // pure et Rage en main ; un cristal bleu pour le second mana de Magie pure.
            name: "actions-avancees",
            rngs: None,
            events: {
                let mut events = arena(
                    PLAIN_W,
                    &[
                        advanced(arcanum_core::AdvancedAction::IceShield),
                        advanced(arcanum_core::AdvancedAction::IntoTheHeat),
                        advanced(arcanum_core::AdvancedAction::BloodRage),
                        advanced(arcanum_core::AdvancedAction::PureMagic),
                        card(Rage, 1),
                    ],
                    &[(prowlers(), ORCS_2), (orc_summoners(), ORCS_2)],
                );
                events.extend(settled_progress(0, 3));
                events.extend(unit_events(
                    &[
                        UnitCard::new(Unit::Illusionists, 1),
                        UnitCard::new(Unit::Peasants, 1),
                    ],
                    &[],
                ));
                events.push(Event::CrystalGained {
                    player: SOLO,
                    color: BasicColor::Blue,
                });
                turn_start(events)
            },
            check: |runner| {
                let state = runner.state();
                assert_eq!(state.units(SOLO).len(), 2);
                let legal = legal_actions(state, SOLO);
                assert!(legal.contains(&Action::Challenge { enemy: ORCS_2 }));
            },
        },
        Prepared {
            // Spec 056 : actions avancées de mana et de cartes, offres choisies.
            name: "actions-avancees-mana",
            rngs: None,
            events: {
                use arcanum_core::AdvancedAction::*;
                let mut events = arena(
                    PLAIN_W,
                    &[
                        advanced(MaximalEffect),
                        advanced(Training),
                        advanced(BloodRitual),
                        advanced(ManaStorm),
                        advanced(Decompose),
                        card(March, 1),
                        card(Rage, 1),
                    ],
                    &[],
                );
                events.extend(advanced_offer(&[
                    advanced(PathFinding),
                    advanced(FireBolt),
                    advanced(Agility),
                ]));
                let spell = |spell| CardId::new(arcanum_core::DeedCard::Spell(spell), 1);
                events.extend(spell_offer(&[
                    spell(arcanum_core::Spell::Fireball),
                    spell(arcanum_core::Spell::Snowstorm),
                    spell(arcanum_core::Spell::Tremor),
                ]));
                for color in [BasicColor::Red, BasicColor::Blue] {
                    events.push(Event::CrystalGained {
                        player: SOLO,
                        color,
                    });
                }
                turn_start(events)
            },
            check: |runner| {
                use arcanum_core::{AdvancedAction::*, CardPlay};
                let state = runner.state();
                let plays = |of: CardId| -> Vec<CardPlay> {
                    legal_actions(state, SOLO)
                        .into_iter()
                        .filter_map(|action| match action {
                            Action::PlayCard { card, play } if card == of => Some(play),
                            _ => None,
                        })
                        .collect()
                };
                // Effet maximum sur Marche, Entraînement en détruisant Marche pour Chemins de
                // traverse, Tempête de mana sur un dé de la Source, Décomposition sur Rage.
                assert!(plays(advanced(MaximalEffect)).iter().any(|play| matches!(
                    play,
                    CardPlay::Basic { with: Some(target), .. } if target.card == card(March, 1)
                )));
                assert!(plays(advanced(Training)).iter().any(|play| matches!(
                    play,
                    CardPlay::Basic { discard: Some(destroyed), gain: Some(gained), .. }
                        if *destroyed == card(March, 1) && *gained == advanced(PathFinding)
                )));
                assert!(
                    plays(advanced(ManaStorm))
                        .iter()
                        .any(|play| matches!(play, CardPlay::Basic { die: Some(_), .. }))
                );
                assert!(plays(advanced(Decompose)).iter().any(|play| matches!(
                    play,
                    CardPlay::Basic { discard: Some(destroyed), .. } if *destroyed == card(Rage, 1)
                )));
            },
        },
        Prepared {
            // Spec 054 : de nuit, assaut de la tour de mage défendue par des Mages de feu (combat
            // engagé, phase à distance) ; sorts en main, Don pour la magie et Rage, offre de sorts
            // Éclair de feu, Tempête de neige, Secousse ; un cristal de chaque couleur et un jeton
            // noir.
            name: "sorts",
            rngs: None,
            events: {
                use arcanum_core::Spell::*;
                let spell = |spell| CardId::new(arcanum_core::DeedCard::Spell(spell), 1);
                let fire_mages = EnemyToken::new(arcanum_core::Enemy::FireMages, 1);
                // La figurine sur la tour, comme après le déplacement qui lance l'assaut : sur
                // sa case de départ, l'écran la croirait déjà en retraite.
                let mut events = Fortified {
                    figure: MAGE_TOWER,
                    time: arcanum_core::TimeOfDay::Night,
                    hand: vec![
                        spell(Expose),
                        spell(Demolish),
                        spell(Chill),
                        spell(BurningShield),
                        spell(Whirlwind),
                        advanced(arcanum_core::AdvancedAction::MagicTalent),
                        card(Rage, 1),
                    ],
                    keep: None,
                    tower: Some((fire_mages, true)),
                    spells: vec![spell(Fireball), spell(Snowstorm), spell(Tremor)],
                    ..Fortified::default()
                }
                .events();
                for color in [
                    BasicColor::Red,
                    BasicColor::Blue,
                    BasicColor::White,
                    BasicColor::Green,
                ] {
                    events.push(Event::CrystalGained {
                        player: SOLO,
                        color,
                    });
                }
                events.push(Event::ManaTokenGained {
                    player: SOLO,
                    color: arcanum_core::ManaColor::Black,
                });
                events.push(TURN_STARTED);
                events.push(Event::AssaultStarted {
                    player: SOLO,
                    site: MAGE_TOWER,
                    from: NEAR_TOWER,
                });
                events.push(Event::CombatStarted {
                    player: SOLO,
                    enemies: vec![(MAGE_TOWER, fire_mages)],
                });
                events
            },
            check: |runner| {
                use arcanum_core::{CardPlay, Spell::*};
                let state = runner.state();
                assert!(state.combat(SOLO).is_some());
                assert_eq!(state.figure(SOLO), Some(MAGE_TOWER));
                let casts = |of: arcanum_core::Spell| {
                    legal_actions(state, SOLO).into_iter().any(|action| {
                        matches!(action, Action::PlayCard { card, play: CardPlay::Cast { .. } }
                            if card == CardId::new(arcanum_core::DeedCard::Spell(of), 1))
                    })
                };
                // Exposition et Frisson visent les Mages de feu ; Démolition et Tourbillon aussi
                // se lancent en phase à distance.
                for spell in [Expose, Chill, Demolish, Whirlwind] {
                    assert!(casts(spell), "{spell:?}");
                }
            },
        },
        Prepared {
            // Spec 057 : de nuit au village (un dé noir dans la Source pour un sort renforcé), Voyage souterrain, Méditation, Puits de mana, Appel
            // aux armes et Distorsion spatiale en main ; Marche, Rage et Endurance dans la
            // défausse (pour Méditation) ; un cristal de chaque couleur ; Paysans et Forestiers
            // dans l'offre d'unités (pour Appel aux armes).
            name: "sorts-deplacement",
            rngs: None,
            events: {
                use arcanum_core::Spell::*;
                let spell = |spell| CardId::new(arcanum_core::DeedCard::Spell(spell), 1);
                let discarded = [card(March, 1), card(Rage, 1), card(Stamina, 1)];
                let mut hand = vec![
                    spell(UndergroundTravel),
                    spell(Meditation),
                    spell(ManaClaim),
                    spell(CallToArms),
                    spell(SpaceBending),
                ];
                hand.extend(discarded);
                use arcanum_core::ManaColor::{Basic, Black};
                let mut events = Sandbox {
                    figure: Some(VILLAGE),
                    hand,
                    time: arcanum_core::TimeOfDay::Night,
                    dice: vec![
                        Basic(BasicColor::Green),
                        Basic(BasicColor::Red),
                        Basic(BasicColor::Blue),
                        Basic(BasicColor::White),
                        Black,
                    ],
                    ..Sandbox::default()
                }
                .events();
                events.extend(
                    discarded
                        .iter()
                        .map(|&card| Event::CardDiscarded { player: SOLO, card }),
                );
                for color in [
                    BasicColor::Red,
                    BasicColor::Blue,
                    BasicColor::White,
                    BasicColor::Green,
                ] {
                    events.push(Event::CrystalGained {
                        player: SOLO,
                        color,
                    });
                }
                events.extend(unit_events(
                    &[],
                    &[
                        UnitCard::new(Unit::Peasants, 1),
                        UnitCard::new(Unit::Foresters, 1),
                    ],
                ));
                turn_start(events)
            },
            check: |runner| {
                use arcanum_core::{CardPlay, Spell::*};
                let state = runner.state();
                assert_eq!(state.figure(SOLO), Some(VILLAGE));
                assert_eq!(state.hand(SOLO).len(), 5);
                let casts = |of: arcanum_core::Spell| {
                    legal_actions(state, SOLO).into_iter().any(|action| {
                        matches!(action, Action::PlayCard { card, play: CardPlay::Cast { .. } }
                            if card == CardId::new(arcanum_core::DeedCard::Spell(of), 1))
                    })
                };
                for spell in [
                    UndergroundTravel,
                    Meditation,
                    ManaClaim,
                    CallToArms,
                    SpaceBending,
                ] {
                    assert!(casts(spell), "{spell:?}");
                }
            },
        },
        Prepared {
            // Spec 057, issue privée 258 : de nuit, voisin de trois ennemis des orques (3, −3), Ailes du
            // vent et Restauration en main, trois unités épuisées : les ennemis d'Ailes de la nuit
            // se cochent sur l'écran de combat, les unités de Renaissance dans le panneau du bas.
            name: "sorts-cibles",
            rngs: None,
            events: {
                use arcanum_core::ManaColor::{Basic, Black};
                use arcanum_core::Spell::*;
                let spell = |spell| CardId::new(arcanum_core::DeedCard::Spell(spell), 1);
                let units = [
                    UnitCard::new(Unit::UtemCrossbowmen, 1),
                    UnitCard::new(Unit::Peasants, 1),
                    UnitCard::new(Unit::GuardianGolems, 1),
                ];
                let mut events = Sandbox {
                    figure: Some(PLAIN_W),
                    hand: vec![
                        spell(WingsOfWind),
                        spell(Restoration),
                        card(March, 1),
                        card(Rage, 1),
                        card(Stamina, 1),
                    ],
                    placed: vec![
                        (prowlers(), ORCS_2),
                        (token(arcanum_core::Enemy::Swordsmen, 1), ORCS_2),
                        (wolf_riders(), ORCS_2),
                    ],
                    time: arcanum_core::TimeOfDay::Night,
                    dice: vec![
                        Basic(BasicColor::Green),
                        Basic(BasicColor::Red),
                        Basic(BasicColor::Blue),
                        Basic(BasicColor::White),
                        Black,
                    ],
                    ..Sandbox::default()
                }
                .events();
                events.extend(unit_events(&units, &[]));
                events.extend(units.iter().map(|&unit| Event::UnitActivated {
                    player: SOLO,
                    unit,
                    ability: 0,
                    option: 0,
                    effect: arcanum_core::Effect::Nothing,
                    enemy: None,
                }));
                // Un seul dé de la Source par tour : le noir des deux sorts renforcés en jetons,
                // le blanc d'Ailes du vent en cristal.
                for _ in 0..2 {
                    events.push(Event::ManaTokenGained {
                        player: SOLO,
                        color: Black,
                    });
                }
                events.push(Event::CrystalGained {
                    player: SOLO,
                    color: BasicColor::White,
                });
                turn_start(events)
            },
            check: |runner| {
                use arcanum_core::{CardPlay, Spell::*};
                let state = runner.state();
                assert_eq!(state.figure(SOLO), Some(PLAIN_W));
                assert!(state.units(SOLO).iter().all(|owned| !owned.ready));
                let restoration = CardId::new(arcanum_core::DeedCard::Spell(Restoration), 1);
                assert!(legal_actions(state, SOLO).into_iter().any(|action| {
                    matches!(action, Action::PlayCard { card, play: CardPlay::Cast { targets, .. } }
                        if card == restoration && targets.units.len() >= 2)
                }));
            },
        },
        Prepared {
            // Spec 055 : huit artefacts en main avec Marche et Rage, Paysans portant la
            // Bannière de protection, Gardes d'Utem sans bannière ; Coup de sang dans l'offre
            // d'actions avancées (Livre de sagesse), Rôdeurs et Soldats voisins.
            name: "artefacts",
            rngs: None,
            events: {
                use arcanum_core::Artifact::*;
                let peasants = UnitCard::new(Unit::Peasants, 1);
                let guardsmen = UnitCard::new(Unit::UtemGuardsmen, 1);
                // Dix cartes au plus : la fin du tour énumère les sous-ensembles de la main.
                let mut hand: Vec<CardId> = [
                    BannerOfGlory,
                    BannerOfFear,
                    BannerOfProtection,
                    RubyRing,
                    SwordOfJustice,
                    HornOfWrath,
                    EndlessGemPouch,
                    BookOfWisdom,
                    AmuletOfDarkness,
                ]
                .map(artifact)
                .to_vec();
                hand.extend([card(March, 1), card(Rage, 1)]);
                let mut events = Sandbox {
                    figure: Some(PLAIN_W),
                    hand,
                    placed: vec![
                        (prowlers(), ORCS_2),
                        (token(arcanum_core::Enemy::Swordsmen, 1), ORCS_2),
                    ],
                    ..Sandbox::default()
                }
                .events();
                events.extend(unit_events(&[peasants, guardsmen], &[]));
                events.extend(advanced_offer(&[advanced(
                    arcanum_core::AdvancedAction::BloodRage,
                )]));
                events.push(Event::BannerAssigned {
                    player: SOLO,
                    card: artifact(BannerOfProtection),
                    unit: peasants,
                });
                turn_start(events)
            },
            check: |runner| {
                let state = runner.state();
                let protection = artifact(arcanum_core::Artifact::BannerOfProtection);
                assert!(
                    state
                        .units(SOLO)
                        .iter()
                        .any(|owned| owned.banner == Some(protection))
                );
                assert!(
                    legal_actions(state, SOLO)
                        .iter()
                        .any(|action| matches!(action, Action::AssignBanner { .. }))
                );
                assert!(legal_actions(state, SOLO).iter().any(|action| matches!(
                    action,
                    Action::PlayCard {
                        play: arcanum_core::CardPlay::ThrowAway { .. },
                        ..
                    }
                )));
            },
        },
        Prepared {
            // Spec 062 : nuit, cité bleue niveau 5 face cachée (Mages de glace, Mages de feu,
            // Glaceurs), figurine au village, à deux pas ; Marche ×2 et Endurance pour y aller,
            // Rage et Détermination ; un cristal bleu et un rouge ; Catapultes dans la zone.
            name: "cite-bleue",
            rngs: None,
            events: {
                let mut events = City {
                    figure: VILLAGE,
                    time: arcanum_core::TimeOfDay::Night,
                    hand: vec![
                        card(March, 1),
                        card(March, 2),
                        card(Determination, 1),
                        card(Rage, 1),
                        card(Stamina, 1),
                    ],
                    ..City::default()
                }
                .events();
                events.extend(city_offers(&[UnitCard::new(Unit::Catapults, 1)], &[]));
                events.extend(crystals(&[BasicColor::Blue, BasicColor::Red]));
                turn_start(events)
            },
            check: |runner| {
                let state = runner.state();
                assert!(state.is_face_down(CITY));
                assert_eq!(state.figure(SOLO), Some(VILLAGE));
                assert_eq!(state.time_of_day(), arcanum_core::TimeOfDay::Night);
                assert_eq!(state.units(SOLO).len(), 1);
            },
        },
        Prepared {
            // Spec 062 : jour, voisine de la cité blanche niveau 5 (Gardiens, deux Canonniers),
            // défenseurs retournés ; Marche pour l'assaut, Bouclier de glace avec un cristal bleu.
            name: "cite-blanche",
            rngs: None,
            events: {
                let mut events = City {
                    tile: TileId::Core(7),
                    figure: NEAR_WHITE_CITY,
                    defenders: vec![
                        guardsmen(),
                        EnemyToken::new(arcanum_core::Enemy::Gunners, 1),
                        EnemyToken::new(arcanum_core::Enemy::Gunners, 2),
                    ],
                    face_up: true,
                    hand: vec![
                        card(March, 1),
                        advanced(arcanum_core::AdvancedAction::IceShield),
                        card(Rage, 1),
                        card(Determination, 1),
                        card(Stamina, 1),
                    ],
                    ..City::default()
                }
                .events();
                events.extend(crystals(&[BasicColor::Blue]));
                turn_start(events)
            },
            check: |runner| {
                let state = runner.state();
                assert_eq!(state.figure(SOLO), Some(NEAR_WHITE_CITY));
                assert!(!state.is_face_down(CITY));
                assert_eq!(state.city(CITY).map(|city| city.level), Some(5));
            },
        },
        Prepared {
            // Spec 062 : cité blanche conquise ; interagir (+3 des Boucliers), recruter les
            // Bûcherons, ajouter une unité d'élite ; finir le tour : limite de main 7.
            name: "cite-blanche-conquise",
            rngs: None,
            events: conquered_city(TileId::Core(7)),
            check: conquered_city_check,
        },
        Prepared {
            // Spec 062 : cité rouge conquise ; Promesse et Menace renforcées (cristaux blanc et
            // rouge) et les 3 Boucliers : 12 Influence, un artefact.
            name: "cite-rouge-conquise",
            rngs: None,
            events: conquered_city(TileId::Core(8)),
            check: conquered_city_check,
        },
        Prepared {
            // Spec 062 : cité verte conquise ; une action avancée de l'offre, puis la carte du
            // dessus de la pioche.
            name: "cite-verte-conquise",
            rngs: None,
            events: conquered_city(TileId::Core(5)),
            check: conquered_city_check,
        },
        Prepared {
            // Spec 063.
            name: "cite-en-vue",
            rngs: None,
            events: city_in_view(),
            check: |runner| {
                let state = runner.state();
                assert_eq!(
                    state.scenario().map(|scenario| scenario.id()),
                    Some(ScenarioId::FirstReconnaissance)
                );
                assert_eq!(state.tile_deck().tiles().first(), Some(&TileId::Core(5)));
                assert_eq!(state.figure(SOLO), Some(VILLAGE));
            },
        },
        Prepared {
            // Spec 063 et 065 : partie terminée, la fenêtre de fin s'ouvre dès le chargement.
            name: "fin-de-partie",
            rngs: None,
            events: finished_reconnaissance(),
            check: |runner| {
                let state = runner.state();
                assert_eq!(state.game_end(), Some(GameEndReason::CityRevealed));
                assert!(legal_actions(state, SOLO).is_empty());
            },
        },
        Prepared {
            // Spec 045 : campagne 8 en 2A, figurine sur la ruine (7, −5), jeton autel bleu face
            // visible ; un cristal bleu, un jeton bleu, un dé bleu dans la Source : l'offrande.
            name: "ruines-autel",
            rngs: None,
            events: {
                let mut events = Ruins {
                    hand: vec![
                        card(March, 1),
                        card(Rage, 1),
                        card(Determination, 1),
                        card(Swiftness, 1),
                        card(Stamina, 1),
                    ],
                    ..Ruins::default()
                }
                .events();
                events.push(Event::CrystalGained {
                    player: SOLO,
                    color: BasicColor::Blue,
                });
                events.push(Event::ManaTokenGained {
                    player: SOLO,
                    color: arcanum_core::ManaColor::Basic(BasicColor::Blue),
                });
                turn_start(events)
            },
            check: |runner| {
                let state = runner.state();
                assert_eq!(state.figure(SOLO), Some(RUINS));
                assert!(
                    legal_actions(state, SOLO)
                        .iter()
                        .any(|action| matches!(action, Action::PayAltarTribute { .. }))
                );
            },
        },
        Prepared {
            // Spec 045 : nuit, figurine sur la clairière (6, −5) voisine de la ruine (7, −5) face
            // cachée (jeton gris et violet, récompense une unité) ; un trajet vers la plaine
            // (7, −4) passe par la ruine et s'y arrête.
            name: "ruines-nuit",
            rngs: None,
            events: Ruins {
                figure: RUINS_GLADE,
                time: arcanum_core::TimeOfDay::Night,
                token: arcanum_core::RuinToken::GreyViolet,
                face_up: false,
                hand: vec![
                    card(March, 1),
                    card(March, 2),
                    card(Stamina, 1),
                    card(Swiftness, 1),
                    card(Rage, 1),
                ],
                ..Ruins::default()
            }
            .events(),
            check: |runner| {
                let state = runner.state();
                assert_eq!(state.figure(SOLO), Some(RUINS_GLADE));
                assert!(
                    state
                        .ruins()
                        .get(&RUINS)
                        .is_some_and(|placed| !placed.face_up)
                );
            },
        },
        Prepared {
            // Spec 045, issue privée 131 : la partie s'arrête sur le choix de l'unité de la récompense,
            // sans jeton de commandement libre (renvoyer les Paysans ou renoncer).
            name: "ruines-unite",
            events: ruin_unit_reward().0,
            rngs: Some(ruin_unit_reward().1),
            check: |runner| {
                let state = runner.state();
                assert_eq!(state.unit_choices(SOLO), 1);
                assert_eq!(
                    legal_actions(state, SOLO).last(),
                    Some(&Action::DeclineUnitReward)
                );
            },
        },
        Prepared {
            // Spec 060 : manche 1, la fenêtre du choix de la tactique ouverte.
            name: "tactiques-jour",
            rngs: None,
            events: tactics_decor(arcanum_core::TimeOfDay::Day, tactics_hand()),
            check: |runner| {
                let legal = legal_actions(runner.state(), SOLO);
                assert_eq!(legal.len(), 6);
                assert!(
                    legal
                        .iter()
                        .all(|action| matches!(action, Action::ChooseTactic { .. }))
                );
            },
        },
        Prepared {
            // Spec 060 : manche 3, Planification retirée : un trou dans la grille du choix.
            name: "tactiques-manche-3",
            rngs: None,
            events: tactics_third_round(),
            check: |runner| {
                let state = runner.state();
                assert_eq!(state.round(), 3);
                assert!(
                    state
                        .removed_tactics()
                        .contains(&arcanum_core::Tactic::Planning)
                );
                assert!(!legal_actions(state, SOLO).contains(&Action::ChooseTactic {
                    tactic: arcanum_core::Tactic::Planning
                }));
            },
        },
        Prepared {
            // Spec 060 : manche 3, Vol de mana pris et le dé bleu posé dessus.
            name: "tactiques-de-vole",
            rngs: None,
            events: tactics_stolen_die(),
            check: |runner| {
                let state = runner.state();
                let tactic = state.tactic(SOLO).expect("Vol de mana pris");
                assert_eq!(tactic.tactic, arcanum_core::Tactic::ManaSteal);
                assert_eq!(tactic.stolen_die, Some(2));
            },
        },
        Prepared {
            // Spec 060 : manche 2, Provisions prise, deux cartes dessous ; avant le tour 2, le
            // choix de Provisions est fait.
            name: "tactiques-nuit",
            rngs: None,
            events: tactics_night(),
            check: |runner| {
                let tactic = runner.state().tactic(SOLO).expect("Provisions prise");
                assert_eq!(tactic.tactic, arcanum_core::Tactic::SparingPower);
                assert_eq!(tactic.stored_count(), 2);
            },
        },
        Prepared {
            // Spec 060, Q5 : la pioche de fin de tour arrêtée, la fenêtre de Longue Nuit ouverte.
            name: "tactiques-longue-nuit",
            rngs: None,
            events: tactics_long_night(),
            check: |runner| {
                assert_eq!(
                    legal_actions(runner.state(), SOLO),
                    [Action::UseLongNight, Action::DeclineLongNight]
                );
            },
        },
        Prepared {
            // Spec 061 : la fenêtre du choix de la tactique ouverte, fantôme Goldyx ; prendre Au
            // bon moment (6) le fait jouer en premier, avant le tour 1.
            name: "fantome-premier",
            rngs: None,
            events: dummy_decor(),
            check: |runner| {
                let state = runner.state();
                let dummy = state.dummy().expect("le fantôme");
                assert_eq!(dummy.hero(), arcanum_core::Hero::Goldyx);
                assert_eq!(dummy.deck().len(), 16);
                let after = done(
                    state,
                    Action::ChooseTactic {
                        tactic: arcanum_core::Tactic::TheRightMoment,
                    },
                );
                assert_eq!(
                    after.round_order(),
                    Some(arcanum_core::RoundOrder::DummyFirst)
                );
                let dummy = after.dummy().expect("le fantôme");
                assert!(dummy.last_turn().len() >= 3);
            },
        },
        Prepared {
            // Spec 061 : deck du fantôme à 2 cartes, début du tour 1 ; finir le tour lui fait
            // retourner ses 2 cartes, finir le suivant le fait annoncer, et le dernier tour mène à
            // la manche 2, l'action avancée et le cristal gagnés.
            name: "fantome-annonce",
            rngs: None,
            events: dummy_announcing(),
            check: |runner| {
                let state = runner.state().clone();
                assert_eq!(state.dummy().expect("le fantôme").deck().len(), 2);
                let state = filler_turn(&state);
                let dummy = state.dummy().expect("le fantôme");
                assert_eq!(dummy.last_turn().len(), 2);
                assert!(!dummy.announced());
                let state = filler_turn(&state);
                assert!(state.dummy().expect("le fantôme").announced());
                let state = filler_turn(&state);
                assert_eq!(state.round(), 2);
                let dummy = state.dummy().expect("le fantôme");
                assert_eq!(dummy.deck().len(), 17);
                let crystals: u8 = [
                    BasicColor::Red,
                    BasicColor::Blue,
                    BasicColor::White,
                    BasicColor::Green,
                ]
                .into_iter()
                .map(|color| dummy.crystals(color))
                .sum();
                assert_eq!(crystals, 4);
            },
        },
        Prepared {
            // Spec 061 : 1 gloire, les Rôdeurs à côté ; les vaincre et finir le tour mènent à la
            // fenêtre du niveau 2, où la compétence du fantôme apparaît dans l'offre commune ;
            // encadrée, seule la dernière action avancée se prend (spec #20, C3).
            name: "fantome-competence",
            rngs: None,
            events: dummy_skill(),
            check: |runner| {
                let state = runner.state();
                assert_eq!((state.fame(SOLO), state.level(SOLO)), (1, 1));
                assert!(!state.dummy().expect("le fantôme").skills().is_empty());
                assert!(state.common_skills().is_empty());
                let legal = legal_actions(state, SOLO);
                assert!(legal.contains(&Action::Challenge { enemy: ORCS_2 }));
            },
        },
        Prepared {
            // Spec 064, C32 : la côte refusée à la tuile royaume, l'intérieur permis.
            name: "conquete-cote",
            rngs: None,
            events: conquest_coast(),
            check: |runner| {
                let state = runner.state();
                assert_eq!(
                    state.scenario().map(|scenario| scenario.id()),
                    Some(ScenarioId::SoloConquest)
                );
                assert_eq!(state.tile_deck().tiles().first(), Some(&TileId::Core(2)));
            },
        },
        Prepared {
            // Spec 064, C28 à C30 : la dernière cité à prendre, 1/2 au panneau.
            name: "conquete-derniere-cite",
            rngs: None,
            events: conquest_last_city(),
            check: |runner| {
                let state = runner.state();
                let progress = arcanum_core::conquest_progress(state).expect("la progression");
                assert_eq!((progress.conquered, progress.cities), (1, 2));
                assert_eq!(state.round(), 3);
                assert!(state.dummy().is_some());
            },
        },
        Prepared {
            // Spec 064, C30 : mission accomplie, la fenêtre de fin s'ouvre au chargement.
            name: "conquete-accomplie",
            rngs: None,
            events: conquest_won(),
            check: |runner| {
                let state = runner.state();
                assert_eq!(state.game_end(), Some(GameEndReason::AllCitiesConquered));
                assert!(legal_actions(state, SOLO).is_empty());
            },
        },
        Prepared {
            // Spec 064, C31 : dernier tour de la 6e manche, une cité conquise.
            name: "conquete-echouee",
            rngs: None,
            events: conquest_failed(),
            check: |runner| {
                let state = runner.state();
                assert_eq!(state.round(), 6);
                assert_eq!(state.game_end(), None);
                assert!(state.dummy().is_some_and(|dummy| dummy.deck().is_empty()));
            },
        },
    ]
}

fn parties_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("scripts")
        .join("parties")
}

fn runner_of(events: &[Event], rngs: Option<RngPositions>) -> GameRunner {
    let mut log = EventLog::default();
    // Une seule décision, la mise en place, comme une partie neuve.
    for event in events {
        log.push(0, event.clone());
    }
    GameRunner::resume(log, rngs.unwrap_or_else(|| sandbox_setup().2), Vec::new())
        .unwrap_or_else(|error| panic!("partie préparée invalide : {error}"))
}

/// Le fichier tel que `save` l'écrit.
fn expected_file(game: &SaveGame, name: &str) -> String {
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let path = save(dir.path(), name, game).expect("écriture de la sauvegarde");
    fs::read_to_string(path).expect("relecture de la sauvegarde")
}

const HINT: &str = "partie préparée périmée : régénérer avec \
ARCANUM_REGENERER_DOCS=1 cargo test -p arcanum-persistance --test parties_preparees";

#[test]
fn prepared_games_match_their_construction() {
    let dir = parties_dir();
    let regenerate = std::env::var_os("ARCANUM_REGENERER_DOCS").is_some();
    for party in prepared() {
        let game = SaveGame::from_runner(&runner_of(&party.events, party.rngs), SAVED_AT)
            .expect("partie commencée");
        let expected = expected_file(&game, party.name);
        let path = dir.join(format!("{}.json", party.name));
        if regenerate {
            fs::create_dir_all(&dir).expect("dossier scripts/parties");
            fs::write(&path, &expected).expect("écriture de la partie préparée");
            continue;
        }
        let found = fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("{} absent : {HINT}", path.display()));
        assert!(found == expected, "{} : {HINT}", path.display());
    }
}

#[test]
fn prepared_games_load_in_their_situation() {
    for party in prepared() {
        let path = parties_dir().join(format!("{}.json", party.name));
        let runner = load(&path)
            .unwrap_or_else(|error| panic!("{} : {error}", path.display()))
            .resume(Vec::new())
            .unwrap_or_else(|error| panic!("{} : {error}", path.display()));
        (party.check)(&runner);
    }
}

#[test]
fn every_prepared_game_is_listed_in_the_manual_check_page() {
    let page = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("docs")
            .join("verification-manuelle.md"),
    )
    .expect("docs/verification-manuelle.md existe");
    for party in prepared() {
        assert!(
            page.contains(&format!("| `{}` |", party.name)),
            "{} absente du tableau de docs/verification-manuelle.md",
            party.name
        );
    }
}

/// Issue privée 296 : une partie préparée commence au début d'un tour, sans fait « Ce tour » ni bandeau
/// de niveau venus de sa construction (cristaux, unités, gloire et niveau posés d'avance). Seule la
/// conquête posée « ce tour » (récompenses de fin de tour) ouvre les faits du tour, suivie de ce
/// que le tour joué a produit.
#[test]
fn prepared_games_start_without_turn_facts_nor_level_banners() {
    let mut wrong = Vec::new();
    for party in prepared() {
        let runner = runner_of(&party.events, party.rngs);
        let state = runner.state();
        let facts = state.turn_facts(SOLO);
        let posed = !matches!(facts.first(), None | Some(TurnFact::SiteConquered { .. }));
        if posed || !state.levels_gained(SOLO).is_empty() {
            wrong.push(format!(
                "{} (faits du tour {facts:?}, bandeaux de niveau {:?})",
                party.name,
                state.levels_gained(SOLO)
            ));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}
