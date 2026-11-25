//! Scénarios de la spec 062 côté affichage (Cités) : C37 à C41.

use arcanum_core::{
    Action, CardId, CommonCard, CommonDeck, Enemy, EnemyToken, Event, GameState, Hex, Offer,
    Scenario, TileId, UndoStatus, Unit, UnitCard, legal_actions, view,
};
use arcanum_i18n::{Catalog, Language};
use arcanum_ui::{
    BoardCamera, BoardScreen, CombatChoice, Command, EndTurnChoice, GameScreen, OverlayKind,
    ScreenMode, ScreenPhase, board_screen, combat_screen, game_screen, hex_center,
    repository_assets_dir,
};
use eframe::egui::{Pos2, Rect, pos2, vec2};

// L'arène des tests d'arcanum-core, partagée plutôt que recopiée.
#[path = "../../arcanum-core/tests/arene/mod.rs"]
mod arene;
use arene::*;

mod support;
use support::board_input;

use arcanum_core::BasicAction::*;

// --- Outils ---

fn french() -> Catalog {
    Catalog::load(
        &repository_assets_dir().join("i18n"),
        &Language("fr".into()),
    )
    .expect("fr.json du dépôt")
}

fn screen_in(state: &GameState, mode: &ScreenMode, catalog: &Catalog) -> GameScreen {
    let legal = legal_actions(state, SOLO);
    game_screen(
        &view(state, SOLO),
        ScreenPhase::Playing {
            legal_actions: &legal,
        },
        mode,
        catalog,
    )
}

fn screen_of(state: &GameState, catalog: &Catalog) -> GameScreen {
    screen_in(state, &ScreenMode::Normal, catalog)
}

fn area() -> Rect {
    Rect::from_min_size(pos2(0.0, 0.0), vec2(1000.0, 700.0))
}

fn camera() -> BoardCamera {
    BoardCamera {
        center: Pos2::ZERO,
        zoom: 30.0,
    }
}

fn board(state: &GameState, cursor: Hex, catalog: &Catalog) -> BoardScreen {
    let seen = view(state, SOLO);
    let legal = legal_actions(state, SOLO);
    let camera = camera();
    board_screen(
        &board_input(
            &seen,
            &legal,
            camera,
            area(),
            Some(camera.to_screen(area(), hex_center(cursor))),
        ),
        catalog,
    )
}

fn hover(state: &GameState, catalog: &Catalog) -> Vec<String> {
    board(state, CITY, catalog).hover_lines
}

fn has(lines: &[String], wanted: &str) -> bool {
    lines.iter().any(|line| line == wanted)
}

fn move_to(hex: Hex) -> Action {
    Action::MoveHero { to: hex }
}

/// Le décor joué : ces cartes jouées pour leur effet de base.
fn playing(decor: City, cards: &[CardId]) -> GameState {
    cards
        .iter()
        .fold(decor.state(), |state, &card| done(&state, basic(card)))
}

fn white_defenders() -> Vec<EnemyToken> {
    vec![
        guardsmen(),
        token(Enemy::Gunners, 1),
        token(Enemy::Gunners, 2),
    ]
}

fn purchases(state: &GameState, catalog: &Catalog) -> Vec<(String, bool)> {
    screen_of(state, catalog)
        .purchases
        .into_iter()
        .map(|purchase| (purchase.label, purchase.command.is_some()))
        .collect()
}

fn index_of(state: &GameState, wanted: &Action) -> usize {
    legal_actions(state, SOLO)
        .iter()
        .position(|action| action == wanted)
        .unwrap_or_else(|| panic!("{wanted:?} doit être légale"))
}

// --- Infobulle ---

// C37 — Infobulle d'une cité : nom et niveau, défenseurs (couleurs face cachée, puis noms), bonus
// de la cité, ce qu'on y achète, la limite de main ; vers la cité, les lignes de l'assaut.
#[test]
fn city_tooltip_lines() {
    let catalog = french();
    // Face cachée, figurine au village (à deux pas).
    let hidden = hover(&City::default().state(), &catalog);
    assert_eq!(
        hidden.first().map(String::as_str),
        Some("Tuile royaume 6"),
        "{hidden:?}"
    );
    for line in [
        "Cité bleue, niveau 5",
        "Défenseurs face cachée : tour de mage, tour de mage, cité",
        "Bonus des défenseurs : attaque de glace ou de feu +2, de feu glacé +1",
        "On y achète des sorts : 7 Influence et un mana",
        "Finir le tour ici ou à côté : limite de main +1, +2 pour le Seigneur",
    ] {
        assert!(has(&hidden, line), "{line} absent de {hidden:?}");
    }
    assert!(!has(&hidden, "Cité bleue"), "{hidden:?}");
    assert!(
        !hidden.iter().any(|line| line.starts_with("Vos Boucliers")),
        "{hidden:?}"
    );

    // Retournés, figurine voisine avec deux Marches : leurs noms, puis l'assaut.
    let next = playing(
        City {
            figure: NEAR_CITY,
            face_up: true,
            ..City::default()
        },
        &[card(March, 1), card(March, 2)],
    );
    let lines = hover(&next, &catalog);
    let defenders = lines
        .iter()
        .find(|line| line.starts_with("Défenseurs : "))
        .unwrap_or_else(|| panic!("défenseurs absents de {lines:?}"));
    for name in ["Mages de glace", "Mages de feu", "Glaceurs"] {
        assert!(defenders.contains(name), "{defenders}");
    }
    assert!(
        has(&lines, "⚔ Assaut : Réputation −1, combat obligatoire"),
        "{lines:?}"
    );
    // Les Glaceurs (Fortifié) sont doublement fortifiés, les mages fortifiés par le site.
    assert!(
        has(
            &lines,
            "Défenseur doublement fortifié : aucun tir à distance"
        ),
        "{lines:?}"
    );
    assert!(has(&lines, "Coût de déplacement : 2"), "{lines:?}");
    assert!(has(&lines, "Cliquez pour vous y déplacer"), "{lines:?}");
    assert!(!has(
        &lines,
        "Cité : ni conquise ni visitée dans ce scénario"
    ));

    // Assaut repoussé : un Bouclier, les Mages de feu vaincus.
    let mut events = City {
        figure: NEAR_CITY,
        defenders: vec![token(Enemy::IceMages, 1), token(Enemy::Freezers, 1)],
        face_up: true,
        ..City::default()
    }
    .events();
    events.push(Event::CityShieldPlaced {
        player: SOLO,
        hex: CITY,
        enemy: token(Enemy::FireMages, 1),
    });
    let lines = hover(&state_of(&events), &catalog);
    assert!(has(&lines, "Vos Boucliers : 1"), "{lines:?}");
    assert!(
        !has(&lines, "Conquise : vous en êtes le Seigneur"),
        "{lines:?}"
    );

    // Conquise : Boucliers, Seigneur, achats ; plus de défenseurs ni de bonus, plus d'assaut.
    let conquered = state_of(&conquered_city(
        TileId::Core(7),
        NEAR_WHITE_CITY,
        3,
        SOLO,
        vec![],
    ));
    let lines = hover(&conquered, &catalog);
    for line in [
        "Cité blanche, niveau 5",
        "Vos Boucliers : 3",
        "Conquise : vous en êtes le Seigneur",
        "On y recrute toutes les unités ; unité d'élite ajoutée à l'offre pour 2 Influence",
        "Finir le tour ici ou à côté : limite de main +1, +2 pour le Seigneur",
    ] {
        assert!(has(&lines, line), "{line} absent de {lines:?}");
    }
    assert!(
        !lines
            .iter()
            .any(|line| line.starts_with("Bonus des défenseurs")
                || line.starts_with("Défenseurs")
                || line.starts_with('⚔')),
        "{lines:?}"
    );
    // Les achats des cités rouge et verte.
    for (tile, offer) in [
        (TileId::Core(8), "On y achète des artefacts : 12 Influence"),
        (
            TileId::Core(5),
            "On y achète des actions avancées : 6 Influence",
        ),
    ] {
        let lines = hover(
            &state_of(&conquered_city(tile, NEAR_CITY, 1, SOLO, vec![])),
            &catalog,
        );
        assert!(has(&lines, offer), "{offer} absent de {lines:?}");
    }
}

// C37 — Plateau : une cité conquise porte le drapeau du joueur ; une cité assaillable, la pastille
// rouge de l'assaut.
#[test]
fn conquered_city_shows_a_flag() {
    let catalog = french();
    let conquered = state_of(&conquered_city(
        TileId::Core(7),
        NEAR_WHITE_CITY,
        3,
        SOLO,
        vec![],
    ));
    let screen = board(&conquered, CITY, &catalog);
    assert!(
        screen
            .overlays
            .iter()
            .any(|overlay| overlay.hex == CITY && overlay.kind == OverlayKind::Flag)
    );
    let open = playing(
        City {
            figure: NEAR_CITY,
            face_up: true,
            ..City::default()
        },
        &[card(March, 1)],
    );
    let screen = board(&open, CITY, &catalog);
    assert!(
        screen
            .move_marks
            .iter()
            .any(|mark| mark.hex == CITY && mark.reachable),
        "{:?}",
        screen.move_marks
    );
}

// C38 — Première reconnaissance : la cité reste fermée, « Cité : ni conquise ni visitée dans ce
// scénario ».
#[test]
fn closed_city_tooltip() {
    let catalog = french();
    let events: Vec<Event> = City {
        scenario: Scenario::first_reconnaissance().without_dummy(),
        figure: NEAR_CITY,
        defenders: Vec::new(),
        ..City::default()
    }
    .events()
    .into_iter()
    .filter(|event| !matches!(event, Event::CityPlaced { .. }))
    .collect();
    let state = done(&state_of(&events), basic(card(March, 1)));
    let lines = hover(&state, &catalog);
    assert!(
        has(&lines, "Cité : ni conquise ni visitée dans ce scénario"),
        "{lines:?}"
    );
    assert!(has(&lines, "Cité bleue"), "{lines:?}");
    assert!(
        !lines.iter().any(|line| line.contains("niveau")),
        "{lines:?}"
    );
}

// --- Combat ---

/// L'assaut de la cité de `tile` défendue par `defenders`, depuis sa voisine : le combat engagé.
fn city_combat(tile: TileId, defenders: Vec<EnemyToken>) -> GameState {
    let figure = if tile == TileId::Core(7) {
        NEAR_WHITE_CITY
    } else {
        NEAR_CITY
    };
    let state = playing(
        City {
            tile,
            figure,
            defenders,
            face_up: true,
            ..City::default()
        },
        &[card(March, 1)],
    );
    let state = done(&state, move_to(CITY));
    done(&state, Action::StartCombat)
}

fn enemy_card(state: &GameState, wanted: EnemyToken, catalog: &Catalog) -> arcanum_ui::EnemyCard {
    combat_screen(
        &view(state, SOLO),
        &legal_actions(state, SOLO),
        &CombatChoice::default(),
        UndoStatus::Possible,
        catalog,
    )
    .expect("écran de combat")
    .enemies
    .into_iter()
    .find(|enemy| enemy.token == wanted)
    .expect("ennemi du combat")
}

fn stakes(enemy: &arcanum_ui::EnemyCard) -> Vec<String> {
    enemy.stakes.clone()
}

// C39 — Écran de combat : armure et attaque modifiées par la cité, comme les réductions ;
// capacités gagnées marquées « (cité) ».
#[test]
fn combat_shows_city_bonus() {
    let catalog = french();
    // Cité blanche : Gardiens, armure 7 + 1.
    let white = city_combat(TileId::Core(7), white_defenders());
    let guards = enemy_card(&white, guardsmen(), &catalog);
    let status = guards.status.clone().unwrap_or_default();
    assert!(status.contains("Armure 8 (au lieu de 7)"), "{status}");

    // Cité bleue : Mages de glace, attaque de glace 5 + 2 ; Glaceurs vifs, 2 × (3 + 2).
    let blue = to_block_phase(&city_combat(TileId::Core(6), blue_defenders()));
    let mages = enemy_card(&blue, token(Enemy::IceMages, 1), &catalog);
    let status = mages.status.clone().unwrap_or_default();
    assert!(status.contains("Attaque 7 (au lieu de 5)"), "{status}");
    assert!(
        stakes(&mages).contains(&"À parer : 7".to_string()),
        "{:?}",
        mages.stakes
    );
    let freezers = enemy_card(&blue, token(Enemy::Freezers, 1), &catalog);
    assert!(
        stakes(&freezers).contains(&"À parer : 10 (vif, 2 × 5)".to_string()),
        "{:?}",
        freezers.stakes
    );

    // Cité rouge : Gardiens (attaque physique 3) brutaux.
    let red = to_block_phase(&city_combat(TileId::Core(8), vec![guardsmen()]));
    let guards = enemy_card(&red, guardsmen(), &catalog);
    let abilities = guards.abilities.clone().unwrap_or_default();
    assert!(abilities.contains("brutal (cité)"), "{abilities}");
    assert!(
        stakes(&guards)
            .iter()
            .any(|line| line.starts_with("Si non paré : 6 (brutal, 2 × 3)")),
        "{:?}",
        guards.stakes
    );
    // Hors de la cité rouge, rien de gagné.
    let abilities = enemy_card(&white, guardsmen(), &catalog)
        .abilities
        .unwrap_or_default();
    assert!(!abilities.contains("(cité)"), "{abilities}");
}

// --- Interaction ---

// C40 — Interaction dans une cité : en-tête, ligne des Boucliers, achats de sa couleur, pas de
// soin, recrutement selon la cité.
#[test]
fn city_interaction_buttons() {
    let catalog = french();
    let artifacts: Vec<CardId> = arcanum_core::Artifact::ALL
        .into_iter()
        .map(artifact)
        .collect();
    let red = interacting_in_city(
        TileId::Core(8),
        9,
        deal(CommonDeck::Artifacts, Offer::Spells, &artifacts, 0),
    );
    let screen = screen_of(&red, &catalog);
    assert_eq!(
        screen.site_menu.as_ref().map(|menu| menu.title.as_str()),
        Some("Interaction : Cité rouge")
    );
    assert_eq!(
        screen.city_shields.as_deref(),
        Some("Boucliers dans la cité : Influence +3")
    );
    assert_eq!(
        purchases(&red, &catalog),
        [("Acheter un artefact (12)".to_string(), true)]
    );
    // Trop peu d'influence : grisé.
    let poor = interacting_in_city(
        TileId::Core(8),
        1,
        deal(CommonDeck::Artifacts, Offer::Spells, &artifacts, 0),
    );
    assert_eq!(
        purchases(&poor, &catalog),
        [("Acheter un artefact (12)".to_string(), false)]
    );

    // Cité verte : les actions avancées de l'offre, puis la carte du dessus de la pioche.
    use arcanum_core::AdvancedAction::*;
    let cards = [
        advanced(IceShield),
        advanced(FireBolt),
        advanced(Agility),
        advanced(Diplomacy),
    ];
    let green = interacting_in_city(
        TileId::Core(5),
        3,
        deal(
            CommonDeck::AdvancedActions,
            Offer::AdvancedActions,
            &cards,
            3,
        ),
    );
    let labels: Vec<(String, bool)> = purchases(&green, &catalog);
    assert_eq!(labels.len(), 4, "{labels:?}");
    assert!(
        labels[..3]
            .iter()
            .all(|(label, enabled)| label.starts_with("Acheter : ")
                && label.ends_with("(6)")
                && *enabled)
    );
    assert_eq!(
        labels[3],
        (
            "Acheter la carte du dessus de la pioche d'actions avancées (6)".to_string(),
            true
        )
    );
    let screen = screen_of(&green, &catalog);
    let top = screen
        .purchases
        .iter()
        .find(|purchase| purchase.label.starts_with("Acheter la carte du dessus"))
        .and_then(|purchase| purchase.command);
    assert_eq!(
        top,
        Some(Command::Play(index_of(
            &green,
            &Action::BuyAdvancedActionFromDeck
        )))
    );

    // Cité blanche : l'unité d'élite, et toutes les unités de l'offre au recrutement.
    let gunners = UnitCard::new(Unit::AmotepGunners, 1);
    let foresters = UnitCard::new(Unit::Foresters, 1);
    let mut extra = unit_events(&[], &[foresters]);
    extra.push(Event::CommonDeckShuffled {
        deck: CommonDeck::EliteUnits,
        order: vec![CommonCard::Unit(gunners)],
    });
    let white = interacting_in_city(TileId::Core(7), 3, extra.clone());
    assert_eq!(
        purchases(&white, &catalog),
        [("Ajouter une unité d'élite à l'offre (2)".to_string(), true)]
    );
    let foresters_name = catalog.text(Unit::Foresters.label_key());
    let recruits = |state: &GameState| -> Vec<String> {
        screen_of(state, &catalog)
            .recruit
            .into_iter()
            .map(|button| button.label)
            .collect()
    };
    assert!(
        recruits(&white)
            .iter()
            .any(|label| label.contains(&foresters_name)),
        "{:?}",
        recruits(&white)
    );
    // Dans la cité bleue, pas les Bûcherons (icône Village seulement), ni soin.
    let blue = interacting_in_city(TileId::Core(6), 3, extra);
    assert!(
        !recruits(&blue)
            .iter()
            .any(|label| label.contains(&foresters_name)),
        "{:?}",
        recruits(&blue)
    );
    for state in [&red, &green, &white, &blue] {
        assert!(
            !purchases(state, &catalog)
                .iter()
                .any(|(label, _)| label.contains("soin")),
            "{:?}",
            purchases(state, &catalog)
        );
    }
}

// --- Limite de main ---

// C41 — Panneau : « Limite de main : 7 (+2 cité) » près d'une cité dont on est le Seigneur ; la
// pioche de fin du tour en tient compte.
#[test]
fn hand_limit_city_line() {
    let catalog = french();
    let near = state_of(&conquered_city(
        TileId::Core(6),
        NEAR_CITY,
        2,
        SOLO,
        City::default().hand,
    ));
    assert_eq!(
        screen_of(&near, &catalog).keep_hand_limit.as_deref(),
        Some("Limite de main : 7 (+2 cité)")
    );
    // Une Marche jouée, rien défaussé : 4 cartes gardées, 3 piochées.
    let played = done(&near, basic(card(March, 1)));
    let ending = ScreenMode::EndingTurn(EndTurnChoice {
        discard: Vec::new(),
        reclaim: None,
        glade: None,
        steady_tempo: None,
    });
    assert_eq!(
        screen_in(&played, &ending, &catalog)
            .end_turn_prompt
            .expect("bandeau")
            .draw,
        "Vous piocherez 3 carte(s)."
    );
    // Loin de la cité, rien.
    let far = state_of(&conquered_city(TileId::Core(6), VILLAGE, 2, SOLO, vec![]));
    assert_eq!(screen_of(&far, &catalog).keep_hand_limit, None);
}

// Historique : la conquête d'une cité s'écrit « Cité bleue conquise ».
#[test]
fn city_conquest_in_turn_facts() {
    let catalog = french();
    let mut events = City {
        figure: CITY,
        defenders: Vec::new(),
        ..City::default()
    }
    .events();
    events.push(Event::CityConquered {
        hex: CITY,
        leader: SOLO,
    });
    let screen = screen_of(&state_of(&events), &catalog);
    assert!(
        screen
            .turn_facts
            .iter()
            .any(|line| line == "Cité bleue conquise"),
        "{:?}",
        screen.turn_facts
    );
}
