//! Scénarios de la spec 012 (scénario et mise en place).

use std::collections::BTreeSet;

use arcanum_core::{
    Columns, CountrysideOrder, Event, FinalScoring, GameConfig, GameRunner, GameState, Hero, Hex,
    MapShape, Player, PlayerEvent, PlayerId, PlayerView, Scenario, ScenarioEnd, ScenarioError,
    ScenarioId, TileDeckCounts, TileDeckRecipe, TileId, TileKind, TilePosition, apply, new_game,
    view,
};

const SOLO: PlayerId = PlayerId::SOLO;

/// Emplacement par ses coordonnées d'emplacement : `i × A + j × B`.
fn at(i: i32, j: i32) -> TilePosition {
    TilePosition::from_lattice(i, j)
}

fn b() -> TilePosition {
    at(0, 1)
}

fn a() -> TilePosition {
    at(1, 0)
}

fn a_minus_b() -> TilePosition {
    at(1, -1)
}

/// Scénario de test : diffère du bac à sable sur tout ce que la mise en place lit.
/// Il reprend l'identifiant du bac à sable, qui ne sert qu'à son nom.
fn test_scenario() -> Scenario {
    Scenario::new(
        ScenarioId::Sandbox,
        MapShape::Open(Columns::Four),
        TileDeckRecipe {
            countryside: 7,
            countryside_order: CountrysideOrder::Random,
            core_city: 2,
            core_non_city: 2,
        },
        2,
        0,
        ScenarioEnd::RoundLimit,
        FinalScoring::None,
    )
    .expect("le scénario de test est cohérent")
}

fn config(seed: u64, scenario: Scenario) -> GameConfig {
    GameConfig {
        seed,
        scenario,
        hero: Hero::Tovak,
        dummy_hero: Default::default(),
    }
}

fn start(config: &GameConfig) -> (GameState, Vec<Event>) {
    let mut rng = config.rngs();
    let events = new_game(config, &mut rng);
    let state = events.iter().fold(GameState::default(), apply);
    (state, events)
}

fn tile_deck_order(events: &[Event]) -> Vec<TileId> {
    events
        .iter()
        .find_map(|event| match event {
            Event::TileDeckFormed { order } => Some(order.clone()),
            _ => None,
        })
        .unwrap_or_default()
}

fn placed(state: &GameState) -> Vec<(TilePosition, TileId)> {
    state.map().tiles().collect()
}

fn tile_at(state: &GameState, position: TilePosition) -> Option<TileId> {
    state
        .map()
        .tiles()
        .find(|(placed, _)| *placed == position)
        .map(|(_, tile)| tile)
}

// --- Formes du plateau de jeu ---

// Spec 012 C1 — MKUE p. 7, 5.a : les emplacements forment un réseau de base A = (3, −2), B = (1, −3).
#[test]
fn lattice_coordinates_round_trip() {
    assert_eq!(TilePosition::ORIGIN.lattice(), (0, 0));
    let origin = TilePosition::ORIGIN;
    let from_center = |q, r| TilePosition::from_center(Hex::new(q, r)).expect("centre du pavage");
    assert_eq!(from_center(3, -2).lattice(), (1, 0));
    assert_eq!(from_center(1, -3).lattice(), (0, 1));
    assert_eq!(from_center(2, 1).lattice(), (1, -1));
    assert_eq!(a(), from_center(3, -2));
    assert_eq!(b(), from_center(1, -3));
    assert!(origin.neighbors().contains(&a_minus_b()));

    for i in -10..=10 {
        for j in -10..=10 {
            let position = at(i, j);
            assert_eq!(position.lattice(), (i, j));
            assert_eq!(TilePosition::from_center(position.center()), Some(position));
        }
    }
}

// Spec 012 C2 — MKUE p. 14 : le coin s'ouvre entre A et B.
#[test]
fn wedge_shape_opens_between_a_and_b() {
    let shape = MapShape::Wedge;
    for (i, j) in [(0, 0), (0, 1), (1, 0), (2, 0), (1, 1), (0, 2), (5, 3)] {
        assert!(shape.allows(at(i, j)), "({i}, {j}) doit être permis");
    }
    for (i, j) in [(1, -1), (-1, 1), (-1, 0), (0, -1)] {
        assert!(!shape.allows(at(i, j)), "({i}, {j}) doit être refusé");
    }
}

// Spec 012 C3 — MKUE p. 14 : au plus 5 colonnes, la côte se brise après deux tuiles.
#[test]
fn open_shape_has_five_columns() {
    let shape = MapShape::Open(Columns::Five);
    for (i, j) in [
        (0, 0),
        (0, 1),
        (1, 0),
        (1, -1),
        (0, 2),
        (2, -2),
        (10, 2),
        (10, -2),
    ] {
        assert!(shape.allows(at(i, j)), "({i}, {j}) doit être permis");
    }
    for (i, j) in [(-1, 1), (-1, 0), (0, 3), (3, -3), (10, 3)] {
        assert!(!shape.allows(at(i, j)), "({i}, {j}) doit être refusé");
    }
}

// Spec 012 C4 — MKUE p. 14 : 4 colonnes s'ouvrent plus à droite (côté A − B), 3 colonnes des deux côtés.
#[test]
fn limited_open_shapes_drop_outer_columns() {
    let four = MapShape::Open(Columns::Four);
    assert!(four.allows(at(2, -2)));
    assert!(!four.allows(at(0, 2)));

    let three = MapShape::Open(Columns::Three);
    assert!(!three.allows(at(2, -2)));
    assert!(!three.allows(at(0, 2)));
    assert!(three.allows(at(10, 1)));
    assert!(three.allows(at(10, -1)));
}

// Spec 012 C5 — MKUE p. 3 et p. 14 : la forme donne la face de départ et les premières tuiles.
#[test]
fn shape_gives_starting_side_and_first_positions() {
    assert_eq!(MapShape::Wedge.starting_tile(), TileId::StartingA);
    assert_eq!(MapShape::Wedge.first_positions(), vec![b(), a()]);
    for columns in [Columns::Three, Columns::Four, Columns::Five] {
        let shape = MapShape::Open(columns);
        assert_eq!(shape.starting_tile(), TileId::StartingB);
        assert_eq!(shape.first_positions(), vec![b(), a(), a_minus_b()]);
    }
    for shape in [
        MapShape::Wedge,
        MapShape::Open(Columns::Three),
        MapShape::Open(Columns::Four),
        MapShape::Open(Columns::Five),
    ] {
        assert!(shape.allows(TilePosition::ORIGIN));
        for position in shape.first_positions() {
            assert!(shape.allows(position), "{shape:?} : {position:?}");
        }
    }
}

// --- Mise en place ---

// Spec 012 C6 — MKUE p. 3 et p. 18 : plateau de « Première reconnaissance » en solo.
#[test]
fn sandbox_setup_follows_first_reconnaissance_map() {
    let (state, _) = start(&config(3, Scenario::sandbox()));

    assert_eq!(
        tile_at(&state, TilePosition::ORIGIN),
        Some(TileId::StartingA)
    );
    assert_eq!(tile_at(&state, b()), Some(TileId::Countryside(1)));
    assert_eq!(tile_at(&state, a()), Some(TileId::Countryside(2)));
    assert_eq!(placed(&state).len(), 3);

    let deck = state.tile_deck().tiles();
    assert_eq!(deck.len(), 9);
    assert_eq!(
        &deck[..6],
        &(3..=8).map(TileId::Countryside).collect::<Vec<_>>()[..]
    );
    assert!(deck[6..].iter().all(|tile| tile.kind() == TileKind::Core));
    assert_eq!(deck[6..].iter().filter(|tile| tile.is_city()).count(), 1);
}

// Spec 012 C7 — MKUE p. 3 : tuiles royaume tirées au hasard, une de cité et deux sans cité.
#[test]
fn core_tiles_are_drawn_at_random() {
    let cores: BTreeSet<Vec<TileId>> = (0..20)
        .map(|seed| {
            let (state, _) = start(&config(seed, Scenario::sandbox()));
            let core: Vec<TileId> = state.tile_deck().tiles()[6..].to_vec();
            assert_eq!(core.iter().filter(|tile| tile.is_city()).count(), 1);
            assert_eq!(core.iter().filter(|tile| !tile.is_city()).count(), 2);
            core
        })
        .collect();
    assert!(cores.len() >= 2);
}

// Spec 012 C8 — MKUE p. 3 et p. 14 : le scénario de test a sa propre forme et sa propre pioche.
#[test]
fn test_scenario_uses_its_own_shape_and_tile_deck() {
    let (state, events) = start(&config(5, test_scenario()));
    let order = tile_deck_order(&events);

    assert_eq!(
        tile_at(&state, TilePosition::ORIGIN),
        Some(TileId::StartingB)
    );
    assert_eq!(tile_at(&state, b()), Some(order[0]));
    assert_eq!(tile_at(&state, a()), Some(order[1]));
    assert_eq!(tile_at(&state, a_minus_b()), Some(order[2]));
    assert_eq!(state.tile_deck().tiles(), &order[3..]);
    assert_eq!(state.tile_deck().len(), 8);

    assert_eq!(order.len(), 11);
    let countryside: BTreeSet<TileId> = order[..7].iter().copied().collect();
    assert_eq!(countryside.len(), 7);
    assert!(
        countryside
            .iter()
            .all(|tile| matches!(tile, TileId::Countryside(n) if (1..=11).contains(n)))
    );
    let core = &order[7..];
    assert!(core.iter().all(|tile| tile.kind() == TileKind::Core));
    assert_eq!(core.iter().filter(|tile| tile.is_city()).count(), 2);
    assert_eq!(core.iter().filter(|tile| !tile.is_city()).count(), 2);
}

// Spec 012 C9 : la pioche ne dépend que de la graine.
#[test]
fn tile_deck_depends_only_on_seed() {
    let (_, first) = start(&config(9, test_scenario()));
    let (_, again) = start(&config(9, test_scenario()));
    assert_eq!(first, again);

    let countrysides: BTreeSet<Vec<TileId>> = (0..20)
        .map(|seed| {
            let (_, events) = start(&config(seed, test_scenario()));
            tile_deck_order(&events)[..7].to_vec()
        })
        .collect();
    assert!(countrysides.len() >= 2);
}

// Spec 012 C10 — MKUE p. 3 et p. 7 (8.d) : la figurine entre par le portail.
#[test]
fn hero_figure_starts_on_the_portal() {
    for scenario in [Scenario::sandbox(), test_scenario()] {
        let (state, _) = start(&config(1, scenario));
        assert_eq!(state.figure(SOLO), Some(Hex::ORIGIN));
    }
}

// Spec 012 C11 : ordre des événements de démarrage.
#[test]
fn setup_events_come_in_order() {
    let kinds = |scenario| {
        let (_, events) = start(&config(2, scenario));
        events
            .iter()
            .map(|event| match event {
                Event::GameStarted { .. } => "GameStarted",
                Event::TilePlaced { .. } => "TilePlaced",
                Event::TileDeckFormed { .. } => "TileDeckFormed",
                Event::TileRevealed { .. } => "TileRevealed",
                Event::HeroFigurePlaced { .. } => "HeroFigurePlaced",
                Event::DeckShuffled { .. } => "DeckShuffled",
                Event::CardDrawn { .. } => "CardDrawn",
                Event::CardPlayed { .. } => "CardPlayed",
                Event::CardDiscarded { .. } => "CardDiscarded",
                Event::HeroMoved { .. } => "HeroMoved",
                Event::Explored { .. } => "Explored",
                Event::TurnEnded { .. } => "TurnEnded",
                Event::TurnStarted { .. } => "TurnStarted",
                Event::SourceRolled { .. } => "SourceRolled",
                Event::CrystalGained { .. } => "CrystalGained",
                Event::DieTaken { .. } => "DieTaken",
                Event::CrystalUsed { .. } => "CrystalUsed",
                Event::DieReturned { .. } => "DieReturned",
                Event::ManaTokenGained { .. } => "ManaTokenGained",
                Event::ManaTokenUsed { .. } => "ManaTokenUsed",
                Event::CardReturnedToHand { .. } => "CardReturnedToHand",
                Event::DieSet { .. } => "DieSet",
                Event::RoundEnded { .. } => "RoundEnded",
                Event::RoundStarted { .. } => "RoundStarted",
                Event::EnemyPileShuffled { .. } => "EnemyPileShuffled",
                Event::EnemyPileReshuffled { .. } => "EnemyPileReshuffled",
                Event::EnemyTokenPlaced { .. } => "EnemyTokenPlaced",
                Event::DefenderPlaced { .. } => "DefenderPlaced",
                Event::CityPlaced { .. } => "CityPlaced",
                Event::CityShieldPlaced { .. } => "CityShieldPlaced",
                Event::CityConquered { .. } => "CityConquered",
                Event::AdvancedActionDrawnBlind { .. } => "AdvancedActionDrawnBlind",
                Event::ArtifactBought { .. } => "ArtifactBought",
                Event::EliteUnitAdded { .. } => "EliteUnitAdded",
                Event::EnemyTokenRevealed { .. } => "EnemyTokenRevealed",
                Event::EnemiesProvoked { .. } => "EnemiesProvoked",
                Event::AssaultStarted { .. } => "AssaultStarted",
                Event::EnemyChallenged { .. } => "EnemyChallenged",
                Event::CombatStarted { .. } => "CombatStarted",
                Event::CombatPhaseStarted { .. } => "CombatPhaseStarted",
                Event::CombatEnded { .. } => "CombatEnded",
                Event::AttackDeclared { .. } => "AttackDeclared",
                Event::EnemiesDefeated { .. } => "EnemiesDefeated",
                Event::AttackFailed { .. } => "AttackFailed",
                Event::BlockDeclared { .. } => "BlockDeclared",
                Event::EnemyBlocked { .. } => "EnemyBlocked",
                Event::BlockFailed { .. } => "BlockFailed",
                Event::WoundsTaken { .. } => "WoundsTaken",
                Event::WoundFromCard { .. } => "WoundFromCard",
                Event::PoisonWoundsDiscarded { .. } => "PoisonWoundsDiscarded",
                Event::HandParalyzed { .. } => "HandParalyzed",
                Event::EnemySummoned { .. } => "EnemySummoned",
                Event::SummonedEnemyDiscarded { .. } => "SummonedEnemyDiscarded",
                Event::WoundsHealed { .. } => "WoundsHealed",
                Event::KnockedOut { .. } => "KnockedOut",
                Event::LevelGained { .. } => "LevelGained",
                Event::FameGained { .. } => "FameGained",
                Event::LastTurnAnnounced { .. } => "LastTurnAnnounced",
                Event::GameEnded { .. } => "GameEnded",
                Event::TacticChosen { .. } => "TacticChosen",
                Event::TacticsRemoved { .. } => "TacticsRemoved",
                Event::DeckReshuffled { .. } => "DeckReshuffled",
                Event::CardSearched { .. } => "CardSearched",
                Event::DieStolen { .. } => "DieStolen",
                Event::RightMomentAnnounced { .. } => "RightMomentAnnounced",
                Event::LongNightUsed { .. } => "LongNightUsed",
                Event::DrawStopped { .. } => "DrawStopped",
                Event::ManaSearchUsed { .. } => "ManaSearchUsed",
                Event::CardsShuffledIntoDeck { .. } => "CardsShuffledIntoDeck",
                Event::CardStored { .. } => "CardStored",
                Event::StoredCardsTaken { .. } => "StoredCardsTaken",
                Event::StolenDieUsed { .. } => "StolenDieUsed",
                Event::StolenDieReturned { .. } => "StolenDieReturned",
                Event::DummySetUp { .. } => "DummySetUp",
                Event::DummyTacticDrawn { .. } => "DummyTacticDrawn",
                Event::DummyTurnPlayed { .. } => "DummyTurnPlayed",
                Event::DummyAnnouncedEndOfRound => "DummyAnnouncedEndOfRound",
                Event::DummyCardAdded { .. } => "DummyCardAdded",
                Event::DummyCrystalGained { .. } => "DummyCrystalGained",
                Event::DummyDeckShuffled { .. } => "DummyDeckShuffled",
                Event::DummySkillRevealed { .. } => "DummySkillRevealed",
                Event::CommonDeckShuffled { .. } => "CommonDeckShuffled",
                Event::OfferCardDealt { .. } => "OfferCardDealt",
                Event::OfferCardReturned { .. } => "OfferCardReturned",
                Event::OfferCardGained { .. } => "OfferCardGained",
                Event::ReputationChanged { .. } => "ReputationChanged",
                Event::InteractionStarted { .. } => "InteractionStarted",
                Event::UnitRecruited { .. } => "UnitRecruited",
                Event::UnitDisbanded { .. } => "UnitDisbanded",
                Event::BannerAssigned { .. } => "BannerAssigned",
                Event::BannerLost { .. } => "BannerLost",
                Event::BannersReclaimed { .. } => "BannersReclaimed",
                Event::EndlessManaUsed { .. } => "EndlessManaUsed",
                Event::BannerUsed { .. } => "BannerUsed",
                Event::ArtifactDieRolled { .. } => "ArtifactDieRolled",
                Event::UnitActivated { .. } => "UnitActivated",
                Event::UnitReadied { .. } => "UnitReadied",
                Event::UnitWounded { .. } => "UnitWounded",
                Event::DamageAssignedToUnit { .. } => "DamageAssignedToUnit",
                Event::UnitDestroyed { .. } => "UnitDestroyed",
                Event::UnitHealed { .. } => "UnitHealed",
                Event::UnitsReadied { .. } => "UnitsReadied",
                Event::HealingBought { .. } => "HealingBought",
                Event::AdvancedActionBought { .. } => "AdvancedActionBought",
                Event::SpellBought { .. } => "SpellBought",
                Event::MonasteryBurnStarted { .. } => "MonasteryBurnStarted",
                Event::AdventureSiteEntered { .. } => "AdventureSiteEntered",
                Event::RewardDieRolled { .. } => "RewardDieRolled",
                Event::WoundThrownAway { .. } => "WoundThrownAway",
                Event::MonasteryBurned { .. } => "MonasteryBurned",
                Event::EnemyTokenDiscarded { .. } => "EnemyTokenDiscarded",
                Event::SiteConquered { .. } => "SiteConquered",
                Event::HeroWithdrew { .. } => "HeroWithdrew",
                Event::ArtifactsDrawn { .. } => "ArtifactsDrawn",
                Event::ArtifactTaken { .. } => "ArtifactTaken",
                Event::ArtifactReturned { .. } => "ArtifactReturned",
                Event::VillagePlundered { .. } => "VillagePlundered",
                Event::RuinPileShuffled { .. } => "RuinPileShuffled",
                Event::SkillPileShuffled { .. } => "SkillPileShuffled",
                Event::SkillsRevealed { .. } => "SkillsRevealed",
                Event::SkillGained { .. } => "SkillGained",
                Event::SkillTakenFromCommonOffer { .. } => "SkillTakenFromCommonOffer",
                Event::SkillToCommonOffer { .. } => "SkillToCommonOffer",
                Event::SkillUsed { .. } => "SkillUsed",
                Event::SkillPlacedInCenter { .. } => "SkillPlacedInCenter",
                Event::CenterAdvantageUsed { .. } => "CenterAdvantageUsed",
                Event::CenterSkillReturned { .. } => "CenterSkillReturned",
                Event::UnitUnderBonds { .. } => "UnitUnderBonds",
                Event::ResistanceBroken { .. } => "ResistanceBroken",
                Event::SourceDieKept { .. } => "SourceDieKept",
                Event::RuinTokenPlaced { .. } => "RuinTokenPlaced",
                Event::RuinTokenRevealed { .. } => "RuinTokenRevealed",
                Event::AltarTributePaid { .. } => "AltarTributePaid",
                Event::UnitRewardTaken { .. } => "UnitRewardTaken",
                Event::UnitRewardDeclined { .. } => "UnitRewardDeclined",
                Event::MoveSpentAsAttack { .. } => "MoveSpentAsAttack",
                Event::InfluenceSpentAsBlock { .. } => "InfluenceSpentAsBlock",
                Event::ForcedWithdrawal { .. } => "ForcedWithdrawal",
                Event::SteadyTempoKept { .. } => "SteadyTempoKept",
                Event::CardDestroyed { .. } => "CardDestroyed",
                Event::DieRerolled { .. } => "DieRerolled",
                Event::CrystalDeclined { .. } => "CrystalDeclined",
                Event::AdvancedActionLearned { .. } => "AdvancedActionLearned",
                Event::CrystalsReturned { .. } => "CrystalsReturned",
                Event::EnemyDestroyed { .. } => "EnemyDestroyed",
                Event::DeferredAttackUsed { .. } => "DeferredAttackUsed",
                Event::CardsReturnedToHand { .. } => "CardsReturnedToHand",
                Event::CardSetAside { .. } => "CardSetAside",
                Event::ExtraTurnStarted { .. } => "ExtraTurnStarted",
                Event::DiscardPicked { .. } => "DiscardPicked",
                Event::CardsPlacedOnDeck { .. } => "CardsPlacedOnDeck",
                Event::DrawBonusUsed { .. } => "DrawBonusUsed",
                Event::DieKept { .. } => "DieKept",
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(
        kinds(Scenario::sandbox()),
        [
            "GameStarted",
            "TilePlaced",
            "TileDeckFormed",
            "TileRevealed",
            "TileRevealed",
            "HeroFigurePlaced",
            "DeckShuffled",
            // Spec 013 : la main de départ.
            "CardDrawn",
            "CardDrawn",
            "CardDrawn",
            "CardDrawn",
            "CardDrawn",
            // Spec 020 : la Source.
            "SourceRolled",
            // Spec 030 : les six piles d'ennemis, mélangées en dernier, puis les orques des tuiles 1 et 2.
            "EnemyPileShuffled",
            "EnemyPileShuffled",
            "EnemyPileShuffled",
            "EnemyPileShuffled",
            "EnemyPileShuffled",
            "EnemyPileShuffled",
            // Spec 045 : la pile des jetons de ruines, mélangée après les piles d'ennemis.
            "RuinPileShuffled",
            "EnemyTokenPlaced",
            "EnemyTokenPlaced",
            // Spec 050 : les cinq pioches communes, puis trois cartes par offre.
            "CommonDeckShuffled",
            "CommonDeckShuffled",
            "CommonDeckShuffled",
            "CommonDeckShuffled",
            "CommonDeckShuffled",
            "OfferCardDealt",
            "OfferCardDealt",
            "OfferCardDealt",
            "OfferCardDealt",
            "OfferCardDealt",
            "OfferCardDealt",
            "OfferCardDealt",
            "OfferCardDealt",
            "OfferCardDealt",
            // Spec 051 : la pile de compétences du héros, en dernier.
            "SkillPileShuffled",
        ]
    );
    // Tuiles de campagne tirées au hasard : autant de jetons que d'ennemis errants imprimés, et un
    // défenseur par site fortifié (spec 043, C3), tuile par tuile.
    let (state, _) = start(&config(2, test_scenario()));
    let defenders = state.face_down_defenders().len();
    let expected = [
        "GameStarted",
        "TilePlaced",
        "TileDeckFormed",
        "TileRevealed",
        "TileRevealed",
        "TileRevealed",
        "HeroFigurePlaced",
        "DeckShuffled",
        // Spec 013 : la main de départ.
        "CardDrawn",
        "CardDrawn",
        "CardDrawn",
        "CardDrawn",
        "CardDrawn",
        // Spec 020 : la Source.
        "SourceRolled",
        // Spec 030 : les six piles d'ennemis, mélangées en dernier, puis les jetons des tuiles révélées.
        "EnemyPileShuffled",
        "EnemyPileShuffled",
        "EnemyPileShuffled",
        "EnemyPileShuffled",
        "EnemyPileShuffled",
        "EnemyPileShuffled",
        // Spec 045 : la pile des jetons de ruines.
        "RuinPileShuffled",
    ];
    // Spec 050 : les pioches communes et les offres, après les jetons ; spec 051 : la pile de
    // compétences en dernier.
    let offers = [
        vec!["CommonDeckShuffled"; 5],
        vec!["OfferCardDealt"; 9],
        vec!["SkillPileShuffled"],
    ]
    .concat();
    let actual = kinds(test_scenario());
    let (head, rest) = actual.split_at(expected.len());
    let (tokens, tail) = rest.split_at(rest.len() - offers.len());
    assert_eq!(head, expected);
    assert_eq!(tail, offers);
    // Spec 045 : un jeton de ruines par ruine des tuiles révélées.
    assert_eq!(
        tokens.len(),
        state.enemy_tokens().len() + state.ruins().len()
    );
    assert_eq!(
        tokens
            .iter()
            .filter(|kind| **kind == "DefenderPlaced")
            .count(),
        defenders
    );
    assert!(tokens.iter().all(|kind| matches!(
        *kind,
        "EnemyTokenPlaced" | "DefenderPlaced" | "RuinTokenPlaced"
    )));
}

/// Joueur qui ne répond jamais : la partie reste à la mise en place.
struct Silent;

impl Player for Silent {
    fn request(&mut self, _view: &PlayerView, _legal_actions: &[arcanum_core::Action]) {}

    fn poll(&mut self) -> Option<usize> {
        None
    }
}

// Spec 012 C12 : rejouer le journal redonne le plateau de jeu, la pioche et la figurine.
#[test]
fn replaying_setup_rebuilds_the_map() {
    for scenario in [Scenario::sandbox(), test_scenario()] {
        let runner = GameRunner::new(config(17, scenario), vec![Box::new(Silent)]);
        let replayed = runner.log().replay();
        assert_eq!(&replayed, runner.state());
        assert_eq!(replayed.map(), runner.state().map());
        assert_eq!(replayed.tile_deck(), runner.state().tile_deck());
        assert_eq!(replayed.figure(SOLO), Some(Hex::ORIGIN));
    }
}

// Spec 012 C13 — MKUE p. 1 : la configuration porte le scénario et le héros.
#[test]
fn config_carries_scenario_and_hero() {
    let config = config(4, test_scenario());
    let (state, events) = start(&config);
    assert_eq!(
        events[0],
        Event::GameStarted {
            config: config.clone()
        }
    );
    assert_eq!(state.scenario(), Some(&test_scenario()));
    assert_eq!(state.hero(SOLO), Some(Hero::Tovak));
}

// --- Information cachée ---

// Spec 012 C14 — MKUE p. 3 : la pioche de tuiles n'est vue que par son dos.
#[test]
fn view_shows_tile_deck_backs_only() {
    let (state, _) = start(&config(6, Scenario::sandbox()));
    let seen = view(&state, SOLO);

    assert_eq!(seen.map.tiles().count(), 3);
    assert_eq!(&seen.map, state.map());
    assert_eq!(seen.scenario, Some(Scenario::sandbox()));
    assert_eq!(
        seen.scenario.map(|scenario| scenario.shape()),
        Some(MapShape::Wedge)
    );
    assert_eq!(seen.hero, Some(Hero::Tovak));
    assert_eq!(seen.figure, Some(Hex::ORIGIN));
    assert_eq!(
        seen.tile_deck,
        TileDeckCounts {
            countryside: 6,
            core: 3
        }
    );

    // Deux pioches qui ne diffèrent que par leurs tuiles royaume donnent la même vue.
    let mut other = state.tile_deck().tiles().to_vec();
    other.swap(6, 8);
    let shuffled = apply(state.clone(), &Event::TileDeckFormed { order: other });
    assert_ne!(shuffled, state);
    assert_eq!(view(&shuffled, SOLO), seen);
}

// Spec 012 C15 : le journal filtré cache l'ordre de la pioche, pas les tuiles posées.
#[test]
fn filtered_log_hides_tile_deck_order() {
    let runner = GameRunner::new(config(8, Scenario::sandbox()), vec![Box::new(Silent)]);
    let filtered = runner.log().filtered_for(SOLO);

    assert!(filtered.contains(&PlayerEvent::TileDeckFormed {
        counts: TileDeckCounts {
            countryside: 8,
            core: 3
        }
    }));
    assert!(filtered.contains(&PlayerEvent::TilePlaced {
        tile: TileId::StartingA,
        position: TilePosition::ORIGIN
    }));
    assert!(filtered.contains(&PlayerEvent::TileRevealed {
        tile: TileId::Countryside(1),
        position: b()
    }));
    assert!(filtered.contains(&PlayerEvent::HeroFigurePlaced {
        player: SOLO,
        hex: Hex::ORIGIN
    }));
}

// --- Cas limites ---

fn recipe(countryside: u8, core_city: u8, core_non_city: u8) -> TileDeckRecipe {
    TileDeckRecipe {
        countryside,
        countryside_order: CountrysideOrder::Random,
        core_city,
        core_non_city,
    }
}

// Spec 012 L1 : une composition impossible est refusée par le constructeur.
#[test]
fn impossible_tile_deck_is_refused() {
    let new = |shape, recipe| {
        Scenario::new(
            ScenarioId::Sandbox,
            shape,
            recipe,
            4,
            0,
            ScenarioEnd::RoundLimit,
            FinalScoring::None,
        )
    };
    assert!(new(MapShape::Wedge, recipe(11, 4, 4)).is_ok());
    assert_eq!(
        new(MapShape::Wedge, recipe(12, 1, 1)),
        Err(ScenarioError::TooManyCountryside {
            requested: 12,
            available: 11
        })
    );
    assert_eq!(
        new(MapShape::Wedge, recipe(8, 5, 1)),
        Err(ScenarioError::TooManyCoreCity {
            requested: 5,
            available: 4
        })
    );
    assert_eq!(
        new(MapShape::Wedge, recipe(8, 1, 5)),
        Err(ScenarioError::TooManyCoreNonCity {
            requested: 5,
            available: 4
        })
    );
    assert_eq!(
        new(MapShape::Open(Columns::Five), recipe(2, 1, 1)),
        Err(ScenarioError::NotEnoughForSetup {
            needed: 3,
            countryside: 2
        })
    );
    assert!(new(MapShape::Wedge, recipe(2, 0, 0)).is_ok());
}

// Spec 012 L2 — MKUE p. 18 : « par numéro » prend les plus petits numéros, la tuile 1 sur le dessus.
#[test]
fn by_number_uses_the_lowest_numbers() {
    let scenario = Scenario::new(
        ScenarioId::Sandbox,
        MapShape::Wedge,
        TileDeckRecipe {
            countryside: 5,
            countryside_order: CountrysideOrder::ByNumber,
            core_city: 0,
            core_non_city: 0,
        },
        4,
        0,
        ScenarioEnd::RoundLimit,
        FinalScoring::None,
    )
    .expect("cohérent");
    let (state, events) = start(&config(12, scenario));
    assert_eq!(
        tile_deck_order(&events),
        (1..=5).map(TileId::Countryside).collect::<Vec<_>>()
    );
    assert_eq!(tile_at(&state, b()), Some(TileId::Countryside(1)));
    assert_eq!(tile_at(&state, a()), Some(TileId::Countryside(2)));
}

// Spec 012 L3 : `apply` reste infaillible face à un événement de tuile sans objet.
#[test]
fn invalid_tile_event_changes_nothing() {
    let (state, _) = start(&config(10, Scenario::sandbox()));

    let occupied = apply(
        state.clone(),
        &Event::TileRevealed {
            tile: TileId::Countryside(3),
            position: TilePosition::ORIGIN,
        },
    );
    assert_eq!(occupied, state);

    let already_placed = apply(
        state.clone(),
        &Event::TileRevealed {
            tile: TileId::Countryside(1),
            position: at(2, 0),
        },
    );
    assert_eq!(already_placed, state);

    let starting_again = apply(
        state.clone(),
        &Event::TilePlaced {
            tile: TileId::StartingB,
            position: at(3, 0),
        },
    );
    assert_eq!(starting_again, state);
}

// Spec 012 L4 : une pioche sans tuile royaume est permise.
#[test]
fn tile_deck_without_core_tiles() {
    let scenario = Scenario::new(
        ScenarioId::Sandbox,
        MapShape::Wedge,
        recipe(6, 0, 0),
        4,
        0,
        ScenarioEnd::RoundLimit,
        FinalScoring::None,
    )
    .expect("cohérent");
    let (state, _) = start(&config(14, scenario));
    assert_eq!(state.tile_deck().len(), 4);
    assert_eq!(state.tile_deck().count(TileKind::Core), 0);
    assert_eq!(state.tile_deck().count(TileKind::Countryside), 4);
}
