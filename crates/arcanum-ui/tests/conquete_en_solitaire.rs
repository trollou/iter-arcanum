//! Scénarios de la spec 064 (Conquête en solitaire) côté affichage, sans fenêtre : écran de
//! nouvelle partie, panneau, fenêtre de fin et infobulle de la côte (C26 à C33).

use arcanum_core::{
    Action, BasicAction, BasicColor, ConquestScore, DummyRule, Event, FinalScore, GameEndReason,
    GameState, Hex, Scenario, ScenarioId, TileId, TilePosition, legal_actions, view,
};
use arcanum_i18n::{Catalog, Language};
use arcanum_ui::{
    BoardCamera, DummyOption, GameScreen, GameSession, NewGameContext, NewGameForm, NewGameScreen,
    NewGameSheet, NewGameStep, ScreenMode, ScreenPhase, board_screen, form_config, game_screen,
    hex_center, new_game_screen, repository_assets_dir, sheet_keys_of_scenario,
};
use eframe::egui::{Pos2, Rect, pos2, vec2};

mod support;
use support::{board_input, play, session_of};

// L'arène des tests d'arcanum-core, partagée plutôt que recopiée.
#[path = "../../arcanum-core/tests/arene/mod.rs"]
mod arene;
use arene::*;

use BasicAction::*;

fn catalog(language: &str) -> Catalog {
    Catalog::load(
        &repository_assets_dir().join("i18n"),
        &Language(language.into()),
    )
    .expect("fichier de langue du dépôt")
}

fn french() -> Catalog {
    catalog("fr")
}

fn conquest() -> Scenario {
    Scenario::solo_conquest()
}

fn launch_context() -> NewGameContext {
    NewGameContext::Launch { autosave: None }
}

fn form_at(step: NewGameStep, scenario: ScenarioId) -> NewGameForm {
    NewGameForm {
        scenario,
        step,
        ..NewGameForm::default()
    }
}

fn bullets(catalog: &Catalog, key: &str) -> Vec<String> {
    catalog.text(key).lines().map(str::to_owned).collect()
}

// --- Écran de nouvelle partie ---

// C26 — Q4 : Première reconnaissance, Conquête en solitaire, bac à sable ; la fiche et ses textes.
#[test]
fn new_game_lists_solo_conquest() {
    let catalog = french();
    let screen = new_game_screen(
        &form_at(NewGameStep::Scenario, ScenarioId::SoloConquest),
        &launch_context(),
        &catalog,
    );
    let names: Vec<&str> = screen
        .scenarios
        .iter()
        .map(|choice| choice.name.as_str())
        .collect();
    assert_eq!(
        names,
        [
            "Première reconnaissance",
            "Conquête en solitaire",
            "Bac à sable"
        ]
    );
    assert_eq!(screen.scenarios[1].tagline, "Le scénario solo de référence");
    assert!(screen.scenarios[1].selected);
    let NewGameSheet::Scenario(sheet) = &screen.sheet else {
        panic!("fiche du scénario attendue : {:?}", screen.sheet);
    };
    assert_eq!(sheet.name, "Conquête en solitaire");
    assert_eq!(sheet.flavor, catalog.text("scenario.solo_conquest.flavor"));
    let facts: Vec<&str> = sheet.facts.iter().map(|fact| fact.value.as_str()).collect();
    assert_eq!(
        facts,
        [
            "6 manches : Jour, Nuit, Jour, Nuit, Jour, Nuit",
            "Solo",
            "Conquête"
        ]
    );
    let sections: Vec<Vec<String>> = sheet
        .sections
        .iter()
        .map(|section| section.bullets.clone())
        .collect();
    assert_eq!(
        sections,
        ["objective", "special_rules", "game_end", "scoring"]
            .map(|field| bullets(&catalog, &format!("scenario.solo_conquest.{field}")))
    );
    assert_eq!(sheet.missing, None);
    assert_eq!(sheet_keys_of_scenario(ScenarioId::SoloConquest).len(), 9);
    assert_eq!(screen.recap, "Conquête en solitaire · Arythea");
}

fn dummy_option(screen: &NewGameScreen) -> &DummyOption {
    match &screen.sheet {
        NewGameSheet::Options(sheet) => sheet.dummy.as_ref().expect("l'option du fantôme"),
        other => panic!("étape Options attendue : {other:?}"),
    }
}

// C27 — Q3 : le fantôme est obligatoire ; sa liste de héros reste.
#[test]
fn solo_conquest_dummy_is_required() {
    let catalog = french();
    // Case décochée plus tôt sur Première reconnaissance : sans effet ici.
    let form = NewGameForm {
        dummy: false,
        ..form_at(NewGameStep::Options, ScenarioId::SoloConquest)
    };
    let screen = new_game_screen(&form, &launch_context(), &catalog);
    let option = dummy_option(&screen);
    assert_eq!(
        option.required.as_deref(),
        Some("Ce scénario se joue avec le joueur fantôme.")
    );
    // L'aide de droite ne conseille pas de jouer sans lui.
    assert_eq!(option.help, "Ce scénario se joue avec le joueur fantôme.");
    assert!(option.hero_enabled);
    assert_eq!(option.heroes.len(), 4);
    assert_eq!(screen.recap, "Conquête en solitaire · Arythea");
    assert_eq!(form_config(&form, 1).scenario.dummy(), DummyRule::Standard);
    // Première reconnaissance garde sa case.
    let screen = new_game_screen(
        &form_at(NewGameStep::Options, ScenarioId::FirstReconnaissance),
        &launch_context(),
        &catalog,
    );
    let option = dummy_option(&screen);
    assert_eq!(option.required, None);
    assert_eq!(option.label, "Joueur fantôme");
}

// --- Panneau ---

/// Une case hors du plateau de jeu pour l'autre cité, posée par événements : seul son état compte.
const OTHER_CITY: Hex = Hex::new(30, -30);

/// Conquête en solitaire (sans fantôme : le décor n'en a pas), la cité bleue sans défenseur à
/// côté de la figurine ; l'autre cité conquise si `other_conquered`.
fn city_without_defenders(other_conquered: bool) -> GameSession {
    let mut events = City {
        scenario: conquest().without_dummy(),
        figure: NEAR_CITY,
        defenders: Vec::new(),
        hand: vec![card(March, 1), card(March, 2), card(Rage, 1)],
        ..City::default()
    }
    .events();
    events.push(Event::CityPlaced {
        hex: OTHER_CITY,
        color: BasicColor::Green,
        level: 5,
    });
    if other_conquered {
        events.push(Event::CityConquered {
            hex: OTHER_CITY,
            leader: SOLO,
        });
    }
    session_of(events)
}

fn end_turn(session: &mut GameSession) {
    let action = session
        .legal_actions()
        .iter()
        .find(|action| matches!(action, Action::EndTurn { .. }))
        .cloned()
        .expect("finir le tour");
    play(session, &action);
}

// C28 — la progression des cités sous la ligne de la manche.
#[test]
fn panel_shows_cities_progress() {
    let catalog = french();
    let none = city_without_defenders(false);
    assert_eq!(
        none.screen(&catalog).cities_progress.as_deref(),
        Some("Cités conquises : 0/2")
    );
    let one = city_without_defenders(true);
    assert_eq!(
        one.screen(&catalog).cities_progress.as_deref(),
        Some("Cités conquises : 1/2")
    );
    // Aucune pour Première reconnaissance ni le bac à sable.
    let sandbox = session_of(
        Sandbox {
            figure: Some(VILLAGE),
            ..Sandbox::default()
        }
        .events(),
    );
    assert_eq!(sandbox.screen(&catalog).cities_progress, None);
}

// C29 — MKUE p. 19, *Scenario End* : le bandeau après la conquête de la dernière cité, puis
// « Dernier tour ».
#[test]
fn panel_announces_last_turn_after_conquest() {
    let catalog = french();
    let mut session = city_without_defenders(true);
    play(&mut session, &basic(card(March, 1)));
    play(&mut session, &Action::MoveHero { to: CITY });
    let screen = session.screen(&catalog);
    assert_eq!(
        screen.last_turn_next.as_deref(),
        Some("Toutes les cités sont conquises : après ce tour, encore un dernier tour.")
    );
    assert_eq!(
        screen.cities_progress.as_deref(),
        Some("Cités conquises : 2/2")
    );
    assert_eq!(screen.last_turn, None);
    end_turn(&mut session);
    let screen = session.screen(&catalog);
    assert_eq!(screen.last_turn.as_deref(), Some("Dernier tour"));
    assert_eq!(screen.last_turn_next, None);
}

// --- Fenêtre de fin ---

/// Conquête en solitaire, `conquered` cités conquises, puis la fin posée avec `reason`.
fn ended(conquered: usize, reason: GameEndReason) -> GameState {
    let mut events = Sandbox {
        scenario: conquest().without_dummy(),
        figure: Some(VILLAGE),
        ..Sandbox::default()
    }
    .events();
    let cities = [OTHER_CITY, Hex::new(31, -31)];
    for (index, hex) in cities.into_iter().enumerate() {
        events.push(Event::CityPlaced {
            hex,
            color: BasicColor::Green,
            level: 5,
        });
        if index < conquered {
            events.push(Event::CityConquered { hex, leader: SOLO });
        }
    }
    events.push(Event::GameEnded { reason });
    state_of(&events)
}

fn finished_screen(state: &GameState, score: Option<FinalScore>, catalog: &Catalog) -> GameScreen {
    let mut seen = view(state, SOLO);
    if score.is_some() {
        seen.final_score = score;
    }
    game_screen(&seen, ScreenPhase::Finished, &ScreenMode::Normal, catalog)
}

/// Le décompte de l'exemple de la spec : 151.
fn example_score() -> FinalScore {
    FinalScore {
        fame: 62,
        crystals: 4,
        wounds: 3,
        advanced_actions: 3,
        spells: 2,
        unit_levels: 3,
        artifacts: 1,
        shields: 1,
        adventure_shields: 1,
        conquest: Some(ConquestScore {
            conquered: 2,
            cities: 2,
            unplayed_rounds: 1,
            dummy_deck: 7,
            round_announced: false,
            all_conquered: true,
        }),
    }
}

// C30 — MKUE p. 19, *Scoring* ; carte de score : l'exemple de la spec.
#[test]
fn end_window_shows_conquest_score() {
    let state = ended(2, GameEndReason::AllCitiesConquered);
    let window = finished_screen(&state, Some(example_score()), &french())
        .score_window
        .expect("fenêtre de fin");
    assert_eq!(
        window.reason,
        "Toutes les cités sont conquises et le dernier tour est joué."
    );
    assert_eq!(window.outcome.as_deref(), Some("Mission accomplie !"));
    assert_eq!(
        window.lines,
        [
            "Gloire : 62",
            "Le Savant : 7",
            "Le Collectionneur (1 artefact(s), 4 cristaux) : 4",
            "Le Commandant : 3",
            "Le Conquérant (1 Bouclier(s)) : 2",
            "L'Aventurier : 2",
            "La Victime (3 Blessure(s)) : -6",
            "Cités conquises (2) : 20",
            "Toutes les cités conquises : 15",
            "Manches non jouées (1) : 30",
            "Deck du joueur fantôme (7 carte(s)) : 7",
            "Fin de manche non annoncée : 5",
        ]
    );
    assert_eq!(window.total, "Score final : 151");
    assert_eq!(window.no_titles, "Solo : pas de bonus de titre.");
}

// C31 — MKUE p. 19 (Q2) : mission échouée, seules les cités conquises comptent.
#[test]
fn end_window_shows_failed_mission() {
    let state = ended(1, GameEndReason::LastRound);
    let window = finished_screen(&state, None, &french())
        .score_window
        .expect("fenêtre de fin");
    assert_eq!(window.reason, "La dernière manche est jouée.");
    assert_eq!(
        window.outcome.as_deref(),
        Some("Mission échouée : des cités restent à conquérir.")
    );
    assert_eq!(window.lines.len(), 12);
    assert_eq!(
        window.lines[7..],
        [
            "Cités conquises (1) : 10",
            "Toutes les cités conquises : 0",
            "Manches non jouées (0) : 0",
            "Deck du joueur fantôme (0 carte(s)) : 0",
            "Fin de manche non annoncée : 0",
        ]
    );
    assert_eq!(window.total, "Score final : 10");
}

// --- Infobulle de la côte ---

fn area() -> Rect {
    Rect::from_min_size(pos2(0.0, 0.0), vec2(1000.0, 700.0))
}

// C32 — MKUE p. 14, *Wedge Map* : survol d'un emplacement de la côte, prochaine tuile royaume.
#[test]
fn hover_explains_core_on_coastline() {
    // (1, 1) posée, figurine sur son côté O, voisin de (0, 2), sur la côte (i = 0).
    let mut events = Sandbox {
        scenario: conquest().without_dummy(),
        figure: Some(Hex::new(3, -5)),
        hand: vec![card(March, 1)],
        next: vec![TileId::Core(5)],
        ..Sandbox::default()
    }
    .events();
    events.push(Event::TileRevealed {
        tile: TileId::Countryside(3),
        position: TilePosition::from_lattice(1, 1),
    });
    let state = done(&state_of(&events), basic(card(March, 1)));
    let seen = view(&state, SOLO);
    let legal = legal_actions(&state, SOLO);
    let camera = BoardCamera {
        center: Pos2::ZERO,
        zoom: 30.0,
    };
    let coast = Hex::new(2, -6);
    let screen = board_screen(
        &board_input(
            &seen,
            &legal,
            camera,
            area(),
            Some(camera.to_screen(area(), hex_center(coast))),
        ),
        &french(),
    );
    assert_eq!(
        screen.hover_lines.last().map(String::as_str),
        Some("Pas de tuile royaume sur la côte : explorez un emplacement plus à l'intérieur")
    );
}

// C33 — chaque clé nouvelle existe en français et en anglais.
#[test]
fn every_solo_conquest_key_exists() {
    let french = french();
    let english = catalog("en");
    let won = ended(2, GameEndReason::AllCitiesConquered);
    let lost = ended(1, GameEndReason::LastRound);
    for catalog in [&french, &english] {
        finished_screen(&won, Some(example_score()), catalog);
        finished_screen(&lost, None, catalog);
        for step in NewGameStep::ALL {
            new_game_screen(
                &form_at(step, ScenarioId::SoloConquest),
                &launch_context(),
                catalog,
            );
        }
        for key in sheet_keys_of_scenario(ScenarioId::SoloConquest) {
            catalog.text(&key);
        }
        let mut session = city_without_defenders(true);
        session.screen(catalog);
        play(&mut session, &basic(card(March, 1)));
        play(&mut session, &Action::MoveHero { to: CITY });
        session.screen(catalog);
        catalog.text("board.hover.core_on_coastline");
    }
    assert_eq!(french.gaps(), vec![]);
    assert_eq!(english.gaps(), vec![]);
}
