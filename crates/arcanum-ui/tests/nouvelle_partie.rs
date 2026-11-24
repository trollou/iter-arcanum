//! Scénarios de la spec 065 (Nouvelle partie) côté affichage, sans fenêtre : écran de nouvelle
//! partie, illustrations des héros, lancement, fenêtre en cours de partie, « Continuer ».

use std::fs;
use std::path::Path;

use arcanum_core::{
    Action, BasicAction, BasicColor, CardPlay, Event, EventLog, GameConfig, GameEndReason,
    GameRunner, Hero, PlayerId, RngPositions, Scenario, ScenarioId, TileId, TilePosition,
};
use arcanum_i18n::{Catalog, Language};
use arcanum_persistance::{SaveGame, list, save_auto};
use arcanum_ui::{
    BoardCamera, ClientSession, Command, GameSession, HeroSheet, IllustrationFamily,
    IllustrationIndex, ImageId, NewGameContext, NewGameForm, NewGameScreen, NewGameSheet,
    NewGameStep, OverlayKind, ScenarioSheet, Shortcut, board_images, board_screen, hero_color,
    hero_illustration, hero_image, hero_sheet, human_player, known_illustrations, new_game_screen,
    repository_assets_dir, sheet_keys_of_hero, sheet_keys_of_scenario,
};
use chrono::{DateTime, Local, TimeZone};
use eframe::egui::{Color32, Pos2, Rect, pos2, vec2};

mod support;

use support::{board_input, first_reconnaissance_config};

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

fn fixed_clock() -> DateTime<Local> {
    Local
        .with_ymd_and_hms(2026, 10, 30, 14, 0, 0)
        .single()
        .expect("date non ambiguë")
}

fn clock_seed() -> u64 {
    77
}

const STEM: &str = "2026-10-30_14-00-00";

fn launch(dir: Option<&Path>) -> ClientSession {
    ClientSession::launch(dir.map(Path::to_path_buf))
        .with_clock(fixed_clock)
        .with_seed_source(clock_seed)
}

fn game(client: &ClientSession) -> &GameSession {
    client.game().expect("une partie en cours")
}

fn config(client: &ClientSession) -> GameConfig {
    game(client).config().expect("partie commencée")
}

fn launch_context() -> NewGameContext {
    NewGameContext::Launch { autosave: None }
}

/// Lance une partie depuis l'écran de lancement, avec `hero`, `scenario` et la graine `seed`.
fn start(client: &mut ClientSession, hero: Hero, scenario: ScenarioId, seed: &str) {
    client.command(Command::ChooseHero(hero));
    client.command(Command::ChooseScenario(scenario));
    client.set_seed_text(seed);
    client.command(Command::StartNewGame);
    client.run_frame();
}

/// Les événements de mise en place (décision 0) de la partie en cours.
fn setup_events(client: &ClientSession) -> Vec<Event> {
    game(client)
        .runner()
        .log()
        .entries()
        .iter()
        .filter(|entry| entry.decision == 0)
        .map(|entry| entry.event.clone())
        .collect()
}

fn files_in(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .expect("dossier lisible")
        .map(|entry| {
            entry
                .expect("entrée")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    names.sort();
    names
}

// --- Écran en étapes (spec 078) ---

/// Le formulaire par défaut, à l'étape `step`, avec `hero` et `scenario` cochés.
fn form_at(step: NewGameStep, hero: Hero, scenario: ScenarioId) -> NewGameForm {
    NewGameForm {
        hero,
        scenario,
        step,
        ..NewGameForm::default()
    }
}

fn scenario_sheet_of(screen: &NewGameScreen) -> &ScenarioSheet {
    match &screen.sheet {
        NewGameSheet::Scenario(sheet) => sheet,
        other => panic!("fiche du scénario attendue : {other:?}"),
    }
}

fn hero_sheet_of(screen: &NewGameScreen) -> &HeroSheet {
    match &screen.sheet {
        NewGameSheet::Hero(sheet) => sheet,
        other => panic!("fiche du héros attendue : {other:?}"),
    }
}

/// Les puces d'un libellé : une par ligne.
fn bullets(catalog: &Catalog, key: &str) -> Vec<String> {
    catalog.text(key).lines().map(str::to_owned).collect()
}

// Spec 078, C1 — MKUE p. 1, Game Flow 1 et 2.
#[test]
fn new_game_opens_on_scenario_step() {
    let catalog = french();
    let form = NewGameForm::default();
    assert_eq!(form.hero, Hero::Arythea);
    assert_eq!(form.scenario, ScenarioId::FirstReconnaissance);
    assert_eq!(form.seed, "");
    assert_eq!(form.step, NewGameStep::Scenario);
    assert!(!form.advanced_open);

    let screen = launch(None).new_game_screen(&catalog).expect("écran");
    assert_eq!(screen.title, "Nouvelle partie");
    assert_eq!(screen.step, NewGameStep::Scenario);
    let tabs: Vec<(u8, &str, bool)> = screen
        .steps
        .iter()
        .map(|tab| (tab.number, tab.label.as_str(), tab.active))
        .collect();
    // Spec 061 : l'étape Options suit Héros.
    assert_eq!(
        tabs,
        [
            (1, "Scénario", true),
            (2, "Héros", false),
            (3, "Options", false)
        ]
    );
    assert_eq!(
        screen
            .steps
            .iter()
            .map(|tab| tab.command)
            .collect::<Vec<_>>(),
        [
            Command::ChooseStep(NewGameStep::Scenario),
            Command::ChooseStep(NewGameStep::Hero),
            Command::ChooseStep(NewGameStep::Options)
        ]
    );
    assert_eq!(screen.recap_label, "Votre partie :");
    assert_eq!(screen.recap, "Première reconnaissance · Arythea");
    assert!(screen.heroes[0].selected);
    assert!(screen.scenarios[0].selected);
    assert!(!screen.advanced.open);
    assert_eq!(screen.advanced.label, "Avancé");
    assert!(screen.start.enabled);
    assert_eq!(screen.seed_invalid, None);
    assert_eq!(scenario_sheet_of(&screen).name, "Première reconnaissance");
}

// Spec 078, C2.
#[test]
fn scenario_list_shows_taglines() {
    let screen = new_game_screen(&NewGameForm::default(), &launch_context(), &french());
    assert_eq!(screen.list_title, "Scénarios");
    let list: Vec<(&str, &str)> = screen
        .scenarios
        .iter()
        .map(|choice| (choice.name.as_str(), choice.tagline.as_str()))
        .collect();
    assert_eq!(
        list,
        [
            (
                "Première reconnaissance",
                "Conseillé pour une première partie"
            ),
            // Spec 064, Q4 : Conquête en solitaire entre les deux.
            ("Conquête en solitaire", "Le scénario solo de référence"),
            ("Bac à sable", "Essai libre, sans score"),
        ]
    );
}

// Spec 078, C3 — MKUE p. 18 ; spec 063.
#[test]
fn first_reconnaissance_sheet() {
    let catalog = french();
    let screen = new_game_screen(&NewGameForm::default(), &launch_context(), &catalog);
    let sheet = scenario_sheet_of(&screen);
    assert_eq!(sheet.name, "Première reconnaissance");
    assert_eq!(
        sheet.flavor,
        catalog.text("scenario.first_reconnaissance.flavor")
    );
    let facts: Vec<(&str, &str)> = sheet
        .facts
        .iter()
        .map(|fact| (fact.label.as_str(), fact.value.as_str()))
        .collect();
    assert_eq!(
        facts,
        [
            ("Durée", "4 manches : Jour, Nuit, Jour, Nuit"),
            ("Joueurs", "Solo"),
            ("Type", "Entraînement"),
        ]
    );
    let sections: Vec<(&str, usize)> = sheet
        .sections
        .iter()
        .map(|section| (section.title.as_str(), section.bullets.len()))
        .collect();
    assert_eq!(
        sections,
        [
            ("Objectif", 2),
            // Spec 060 : une ligne de plus, les tactiques ; spec 061 : une autre, le fantôme.
            ("Règles spéciales", 4),
            ("Fin de partie", 2),
            ("Décompte", 2),
        ]
    );
    assert_eq!(
        sheet.sections[1].bullets,
        bullets(&catalog, "scenario.first_reconnaissance.special_rules")
    );
    // Spec 061, C42 : plus rien ne manque.
    assert_eq!(sheet.missing, None);
}

// Spec 078, C4.
#[test]
fn sandbox_sheet_has_nothing_missing() {
    let form = form_at(NewGameStep::Scenario, Hero::Arythea, ScenarioId::Sandbox);
    let screen = new_game_screen(&form, &launch_context(), &french());
    let sheet = scenario_sheet_of(&screen);
    assert_eq!(sheet.name, "Bac à sable");
    assert_eq!(sheet.facts[2].value, "Essai libre");
    assert_eq!(sheet.sections.len(), 4);
    for section in &sheet.sections {
        assert_eq!(section.bullets.len(), 1, "{section:?}");
    }
    assert_eq!(sheet.missing, None);
}

// Spec 078, C5 (écran) : le bac à sable coché, sa fiche et le récapitulatif.
#[test]
fn choosing_a_scenario_updates_sheet_and_recap() {
    let catalog = french();
    let mut client = launch(None);
    client.command(Command::ChooseScenario(ScenarioId::Sandbox));
    let screen = client.new_game_screen(&catalog).expect("écran");
    let selected: Vec<bool> = screen.scenarios.iter().map(|s| s.selected).collect();
    // Spec 064, Q4 : trois scénarios.
    assert_eq!(selected, [false, false, true]);
    assert_eq!(scenario_sheet_of(&screen).name, "Bac à sable");
    assert_eq!(screen.recap, "Bac à sable · Arythea");
    assert_eq!(screen.step, NewGameStep::Scenario);
}

// Spec 078, C8.
#[test]
fn hero_list_shows_portraits_and_taglines() {
    let catalog = french();
    let form = form_at(
        NewGameStep::Hero,
        Hero::Arythea,
        ScenarioId::FirstReconnaissance,
    );
    let screen = new_game_screen(&form, &launch_context(), &catalog);
    assert_eq!(screen.list_title, "Héros");
    let names: Vec<&str> = screen.heroes.iter().map(|h| h.name.as_str()).collect();
    assert_eq!(names, ["Arythea", "Goldyx", "Norowas", "Tovak"]);
    for choice in &screen.heroes {
        assert_eq!(choice.image, hero_image(choice.hero));
        assert_eq!(choice.fallback, hero_color(choice.hero));
        assert!(!choice.tagline.is_empty(), "{choice:?}");
    }
    assert_eq!(screen.heroes[0].tagline, "Feu, siège et nuit");
    assert_eq!(screen.heroes[3].command, Command::ChooseHero(Hero::Tovak));

    let form = form_at(
        NewGameStep::Hero,
        Hero::Tovak,
        ScenarioId::FirstReconnaissance,
    );
    let screen = new_game_screen(&form, &launch_context(), &catalog);
    let selected: Vec<bool> = screen.heroes.iter().map(|h| h.selected).collect();
    assert_eq!(selected, [false, false, false, true]);
    assert_eq!(hero_sheet_of(&screen).name, "Tovak");
    assert_eq!(screen.recap, "Première reconnaissance · Tovak");
}

// Spec 078, C9.
#[test]
fn hero_sheet_shows_description_and_style() {
    let catalog = french();
    let form = form_at(
        NewGameStep::Hero,
        Hero::Arythea,
        ScenarioId::FirstReconnaissance,
    );
    let screen = new_game_screen(&form, &launch_context(), &catalog);
    let sheet = hero_sheet_of(&screen);
    assert_eq!(sheet.name, "Arythea");
    assert_eq!(sheet.image, hero_image(Hero::Arythea));
    assert_eq!(sheet.fallback, hero_color(Hero::Arythea));
    assert_eq!(sheet.description, catalog.text("hero.arythea.description"));
    assert_eq!(sheet.style.title, "Style de jeu");
    assert_eq!(sheet.style.bullets.len(), 4);
    assert_eq!(sheet.style.bullets, bullets(&catalog, "hero.arythea.style"));
}

// Spec 078, C10 — MKUE p. 3 ; basic_actions.ron.
#[test]
fn hero_sheet_shows_own_cards_with_effects() {
    let sheet = hero_sheet(Hero::Arythea, &french());
    assert_eq!(sheet.own_cards_title, "Cartes propres");
    let cards: Vec<(BasicAction, BasicColor)> = sheet
        .own_cards
        .iter()
        .map(|card| (card.card, card.color))
        .collect();
    assert_eq!(
        cards,
        [
            (BattleVersatility, BasicColor::Red),
            (ManaPull, BasicColor::White)
        ]
    );
    let versatility = &sheet.own_cards[0];
    assert_eq!(versatility.name, "Polyvalence");
    assert_eq!(versatility.replaces, "à la place de Rage");
    assert_eq!(versatility.basic_title, "Base");
    assert_eq!(
        versatility.basic,
        "Attaque 2 ou Parade 2 ou Attaque à distance 1"
    );
    assert_eq!(versatility.strong_title, "Renforcé (mana rouge)");
    assert_eq!(
        versatility.strong,
        "Attaque 4 ou Parade 4 ou Attaque de feu 3 ou Parade de feu 3 ou Attaque à distance 3 ou Attaque de siège 2"
    );
    let pull = &sheet.own_cards[1];
    assert_eq!(pull.name, "Attraction de mana");
    assert_eq!(pull.replaces, "à la place de Source de mana");
    assert_eq!(pull.strong_title, "Renforcé (mana blanc)");
}

// Spec 078, C11.
#[test]
fn every_own_card_effect_is_written() {
    let catalog = french();
    for hero in Hero::ALL {
        let sheet = hero_sheet(hero, &catalog);
        assert_eq!(sheet.own_cards.len(), 2, "{hero:?}");
        for card in &sheet.own_cards {
            for text in [&card.basic, &card.strong] {
                assert!(!text.is_empty(), "{hero:?} {card:?}");
                assert!(!text.contains("pas encore"), "{hero:?} {text}");
                assert!(!text.contains('⟦'), "{hero:?} {text}");
            }
        }
    }
}

// Spec 078, C14 (écran) : graine invalide, volet ouvert qu'on ne peut refermer, « Commencer »
// grisé sur les deux étapes.
#[test]
fn invalid_seed_keeps_advanced_open() {
    let catalog = french();
    for step in NewGameStep::ALL {
        let form = NewGameForm {
            seed: "abc".into(),
            ..form_at(step, Hero::Arythea, ScenarioId::FirstReconnaissance)
        };
        let screen = new_game_screen(&form, &launch_context(), &catalog);
        assert!(screen.advanced.open, "{step:?}");
        assert!(!screen.advanced.can_close, "{step:?}");
        assert_eq!(screen.advanced.command, Command::ToggleAdvanced);
        assert_eq!(
            screen.seed_invalid,
            Some(catalog.text("new_game.seed_invalid"))
        );
        assert!(!screen.start.enabled, "{step:?}");
    }
}

// Spec 078, C6 (écran) : « Suivant » à l'étape Scénario seulement.
#[test]
fn next_only_on_scenario_step() {
    let screen = new_game_screen(&NewGameForm::default(), &launch_context(), &french());
    let next = screen.next.expect("Suivant");
    assert_eq!(next.label, "Suivant : choisir le héros");
    assert_eq!(next.command, Command::ChooseStep(NewGameStep::Hero));
    let form = form_at(
        NewGameStep::Hero,
        Hero::Arythea,
        ScenarioId::FirstReconnaissance,
    );
    let screen = new_game_screen(&form, &launch_context(), &french());
    assert_eq!(screen.next, None);
    let active: Vec<bool> = screen.steps.iter().map(|tab| tab.active).collect();
    // Spec 061 : une troisième étape, Options.
    assert_eq!(active, [false, true, false]);
}

// Spec 078, C13 (écran) : refermé sur une graine valide, le volet le dit.
#[test]
fn closed_advanced_label_tells_a_seed_is_entered() {
    let catalog = french();
    let open = NewGameForm {
        seed: "42".into(),
        advanced_open: true,
        ..NewGameForm::default()
    };
    let screen = new_game_screen(&open, &launch_context(), &catalog);
    assert!(screen.advanced.open);
    assert!(screen.advanced.can_close);
    assert_eq!(screen.advanced.label, "Avancé");
    let closed = NewGameForm {
        advanced_open: false,
        ..open
    };
    let screen = new_game_screen(&closed, &launch_context(), &catalog);
    assert!(!screen.advanced.open);
    assert_eq!(screen.advanced.label, "Avancé (graine saisie)");
}

// Spec 078, C17.
#[test]
fn every_scenario_and_hero_has_its_sheet_texts() {
    let mut keys: Vec<String> = Vec::new();
    // Accroche, ambiance, trois faits, quatre rubriques ; plus de « Pas encore dans le jeu » depuis
    // la spec 061.
    for (scenario, count) in [
        (ScenarioId::FirstReconnaissance, 9),
        (ScenarioId::Sandbox, 9),
    ] {
        let of_scenario = sheet_keys_of_scenario(scenario);
        assert_eq!(of_scenario.len(), count, "{scenario:?}");
        keys.extend(of_scenario);
    }
    for hero in Hero::ALL {
        let of_hero = sheet_keys_of_hero(hero);
        assert_eq!(of_hero.len(), 3, "{hero:?}");
        keys.extend(of_hero);
    }
    for language in ["fr", "en"] {
        let catalog = catalog(language);
        for key in &keys {
            let text = catalog.text(key);
            assert!(!text.contains('⟦'), "{language} : {key}");
            assert!(!text.trim().is_empty(), "{language} : {key}");
            assert!(
                text.lines().all(|line| !line.trim().is_empty()),
                "{language} : {key}"
            );
        }
    }
}

// --- Illustrations ---

// C4c — spec 011, C21.
#[test]
fn every_hero_image_exists() {
    let images_dir = repository_assets_dir().join("images");
    let mut texts = Vec::new();
    for hero in Hero::ALL {
        let image = hero_image(hero);
        let stem = format!("{:?}", hero).to_lowercase();
        assert_eq!(image, ImageId::new("hero", leak(stem)));
        let path = image.path(&images_dir);
        let text = fs::read_to_string(&path).unwrap_or_else(|_| panic!("{path:?} manque"));
        assert!(text.starts_with("<svg"), "{path:?} n'est pas un SVG");
        assert!(board_images().contains(&image));
        texts.push(text);
    }
    texts.sort();
    texts.dedup();
    assert_eq!(texts.len(), 4);
    assert!(!board_images().contains(&ImageId::new("hero", "figure")));
    let colors: Vec<Color32> = Hero::ALL.map(hero_color).to_vec();
    assert_eq!(
        colors,
        [
            Color32::from_rgb(0xC6, 0x28, 0x28),
            Color32::from_rgb(0x2E, 0x7D, 0x32),
            Color32::from_rgb(0x5C, 0x7F, 0xB0),
            Color32::from_rgb(0xC9, 0xA2, 0x27),
        ]
    );
}

fn leak(text: String) -> &'static str {
    Box::leak(text.into_boxed_str())
}

// C4d — MKUE p. 3, Player Area.
#[test]
fn figure_overlay_is_the_hero_image() {
    let session = GameSession::new(GameConfig {
        hero: Hero::Norowas,
        ..first_reconnaissance_config(3)
    });
    let view = session.view();
    let area = Rect::from_min_size(pos2(0.0, 0.0), vec2(1000.0, 700.0));
    let camera = BoardCamera {
        center: Pos2::ZERO,
        zoom: 60.0,
    };
    let screen = board_screen(&board_input(&view, &[], camera, area, None), &french());
    let figures: Vec<_> = screen
        .overlays
        .iter()
        .filter(|overlay| overlay.kind == OverlayKind::Figure)
        .collect();
    assert_eq!(figures.len(), 1);
    assert_eq!(figures[0].image, ImageId::new("figure", "norowas"));
    assert_eq!(figures[0].fallback, hero_color(Hero::Norowas));
}

// C4e
#[test]
fn panel_shows_hero_portrait() {
    let catalog = french();
    let mut session = GameSession::new(GameConfig {
        hero: Hero::Goldyx,
        ..first_reconnaissance_config(3)
    });
    session.run_frame();
    let portrait = session.screen(&catalog).portrait.expect("portrait");
    // Spec 075 (retour de Mathieu sur le fil B) : la figurine debout, plus le jeton rond.
    assert_eq!(portrait.image, ImageId::new("figure", "goldyx"));
    assert_eq!(portrait.fallback, hero_color(Hero::Goldyx));
    assert_eq!(portrait.name, "Goldyx");
}

// --- Lancer une partie ---

// C5 — MKUE p. 1 et 3.
#[test]
fn start_uses_chosen_hero_and_scenario() {
    let mut client = launch(None);
    assert!(client.game().is_none());
    start(&mut client, Hero::Goldyx, ScenarioId::Sandbox, "");

    assert!(client.new_game_screen(&french()).is_none());
    let config = config(&client);
    assert_eq!(config.hero, Hero::Goldyx);
    assert_eq!(config.scenario, Scenario::sandbox());
    assert_eq!(config.seed, clock_seed());
    let state = game(&client).runner().state();
    let mut cards: Vec<_> = [
        state.hand(PlayerId::SOLO),
        state.deck(PlayerId::SOLO),
        state.discard_pile(PlayerId::SOLO),
    ]
    .concat();
    let mut deck = Hero::Goldyx.starting_deck();
    deck.sort();
    cards.sort();
    assert_eq!(cards, deck);
}

// C6
#[test]
fn typed_seed_replays_the_same_setup() {
    let mut first = launch(None);
    start(
        &mut first,
        Hero::Tovak,
        ScenarioId::FirstReconnaissance,
        "42",
    );
    assert_eq!(config(&first).seed, 42);
    let mut second = launch(None);
    start(
        &mut second,
        Hero::Tovak,
        ScenarioId::FirstReconnaissance,
        "42",
    );
    assert_eq!(setup_events(&first), setup_events(&second));
}

// C7
#[test]
fn seed_is_trimmed_and_parsed() {
    for (text, seed) in [("007", 7), ("  42  ", 42)] {
        let mut client = launch(None);
        start(&mut client, Hero::Arythea, ScenarioId::Sandbox, text);
        assert_eq!(config(&client).seed, seed, "{text:?}");
    }
}

// C8
#[test]
fn invalid_seed_disables_start() {
    let catalog = french();
    for text in ["abc", "-1", "4.2", "18446744073709551616"] {
        let mut client = launch(None);
        client.set_seed_text(text);
        let screen = client.new_game_screen(&catalog).expect("écran ouvert");
        assert!(!screen.start.enabled, "{text:?}");
        assert_eq!(
            screen.seed_invalid,
            Some(catalog.text("new_game.seed_invalid")),
            "{text:?}"
        );
        client.command(Command::StartNewGame);
        assert!(client.game().is_none(), "{text:?}");
    }
    let mut client = launch(None);
    client.set_seed_text("18446744073709551615");
    assert!(
        client
            .new_game_screen(&catalog)
            .expect("écran")
            .start
            .enabled
    );
}

// C9 — spec 002.
#[test]
fn seed_is_never_displayed() {
    let catalog = french();
    for seed in ["987654321987", ""] {
        let mut client = launch(None).with_seed_source(|| 123_456_789_123);
        start(
            &mut client,
            Hero::Arythea,
            ScenarioId::FirstReconnaissance,
            seed,
        );
        let shown = format!(
            "{:?}{:?}",
            game(&client).screen(&catalog),
            game(&client).controls(&catalog)
        );
        assert!(!shown.contains("987654321987"));
        assert!(!shown.contains("123456789123"));
    }
}

// C10
#[test]
fn panel_shows_scenario_and_hero() {
    let catalog = french();
    let mut client = launch(None);
    start(
        &mut client,
        Hero::Goldyx,
        ScenarioId::FirstReconnaissance,
        "",
    );
    // Spec 074, C4 : le scénario dans la barre de menu, le héros dans le panneau de droite.
    assert_eq!(
        game(&client).menu_bar(&catalog).scenario.as_deref(),
        Some("Première reconnaissance")
    );
    assert_eq!(
        game(&client)
            .screen(&catalog)
            .portrait
            .map(|portrait| portrait.name)
            .as_deref(),
        Some("Goldyx")
    );
}

// --- En cours de partie ---

// C11
#[test]
fn new_game_window_prefills_current_choice() {
    let catalog = french();
    let mut client = launch(None);
    start(&mut client, Hero::Norowas, ScenarioId::Sandbox, "5");
    let before = game(&client).view();

    let button = game(&client).controls(&catalog).new_game;
    assert_eq!(button.label, "Nouvelle partie…");
    assert!(button.enabled);
    client.shortcut(Shortcut::NewGame);
    let screen = client.new_game_screen(&catalog).expect("fenêtre ouverte");
    assert!(screen.heroes[2].selected);
    // Spec 064, Q4 : le bac à sable est troisième.
    assert!(screen.scenarios[2].selected);
    assert_eq!(client.seed_text(), Some(""));
    // Spec 078, C15 : étape Scénario, volet fermé, même après une visite de l'étape Héros.
    assert_eq!(screen.step, NewGameStep::Scenario);
    assert!(!screen.advanced.open);
    assert!(screen.cancel.is_some());
    assert!(screen.continue_game.is_none());
    assert!(screen.load.is_none());
    assert_eq!(
        screen.abandon_warning,
        Some(catalog.text("new_game.abandon_warning"))
    );
    assert_eq!(game(&client).view(), before);

    client.command(Command::ChooseStep(NewGameStep::Hero));
    client.command(Command::ToggleAdvanced);
    client.command(Command::CancelNewGame);
    client.shortcut(Shortcut::NewGame);
    let screen = client.new_game_screen(&catalog).expect("fenêtre ouverte");
    assert_eq!(screen.step, NewGameStep::Scenario);
    assert!(!screen.advanced.open);
}

// C12
#[test]
fn cancel_returns_to_game() {
    let catalog = french();
    let mut client = launch(None);
    start(&mut client, Hero::Arythea, ScenarioId::Sandbox, "5");
    let before = game(&client).view();

    client.command(Command::OpenNewGame);
    client.command(Command::ChooseHero(Hero::Tovak));
    client.command(Command::CancelNewGame);
    assert!(client.new_game_screen(&catalog).is_none());
    assert_eq!(game(&client).view(), before);

    client.shortcut(Shortcut::NewGame);
    // Fenêtre ouverte : les boutons de la partie n'agissent pas.
    client.command(Command::OpenEndTurn);
    assert!(!game(&client).is_ending_turn());
    client.shortcut(Shortcut::Cancel);
    assert!(client.new_game_screen(&catalog).is_none());
    assert_eq!(game(&client).view(), before);
}

// C13 — spec 024.
#[test]
fn abandoned_game_is_saved_first() {
    let catalog = french();
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let mut client = launch(Some(dir.path()));
    start(&mut client, Hero::Arythea, ScenarioId::Sandbox, "5");
    let abandoned = game(&client).runner().log().len();

    client.shortcut(Shortcut::NewGame);
    client.command(Command::ChooseHero(Hero::Tovak));
    client.shortcut(Shortcut::Confirm);
    client.run_frame();

    assert_eq!(config(&client).hero, Hero::Tovak);
    assert_eq!(
        game(&client).controls(&catalog).notice,
        Some(catalog.format("new_game.previous_saved", &[("name", STEM)]))
    );
    let saved =
        arcanum_persistance::load(&dir.path().join(format!("{STEM}.json"))).expect("lisible");
    assert_eq!(saved.summary.hero, Hero::Arythea);
    assert_eq!(saved.events.len(), abandoned);
}

/// Une partie de Première reconnaissance terminée (comme les tests de la spec 063), avec Goldyx
/// pour que ses choix diffèrent des choix par défaut (issue privée 285).
fn finished_session() -> GameSession {
    let events = Sandbox {
        scenario: Scenario::first_reconnaissance().without_dummy(),
        hero: Hero::Goldyx,
        figure: Some(VILLAGE),
        hand: vec![
            card(March, 1),
            card(March, 2),
            card(Stamina, 1),
            card(Rage, 1),
            card(Promise, 1),
        ],
        next: vec![TileId::Core(5)],
        ..Sandbox::default()
    }
    .events();
    let mut log = EventLog::default();
    for event in events {
        log.push(0, event);
    }
    let (player, human) = human_player();
    let runner = GameRunner::resume(log, RngPositions::new(), vec![Box::new(player)])
        .expect("journal valide");
    let mut session = GameSession::with_runner(runner, human);
    session.run_frame();
    let mut act = |wanted: &dyn Fn(&Action) -> bool| {
        let index = session
            .legal_actions()
            .iter()
            .position(wanted)
            .expect("action légale");
        session.command(Command::Play(index));
        session.run_frame();
    };
    act(&|action| {
        *action
            == Action::PlayCard {
                card: card(March, 1),
                play: CardPlay::Basic {
                    die: None,
                    gain: None,
                    with: None,
                    option: 0,
                    discard: None,
                    mana: None,
                },
            }
    });
    act(&|action| {
        *action
            == Action::Explore {
                position: TilePosition::from_lattice(1, 1),
            }
    });
    act(&|action| matches!(action, Action::EndTurn { .. }));
    act(&|action| {
        matches!(
            action,
            Action::PlayCard {
                play: CardPlay::Sideways(_),
                ..
            }
        )
    });
    act(&|action| matches!(action, Action::EndTurn { .. }));
    assert_eq!(session.view().game_end, Some(GameEndReason::CityRevealed));
    session
}

// C14 — spec 063.
#[test]
fn end_window_offers_new_game() {
    let catalog = french();
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let mut client = launch(Some(dir.path())).with_game(finished_session());
    let before = files_in(dir.path());

    let window = game(&client).screen(&catalog).score_window.expect("fin");
    assert_eq!(window.new_game.label, "Nouvelle partie…");
    client.command(window.new_game.command);
    let screen = client.new_game_screen(&catalog).expect("fenêtre ouverte");
    assert_eq!(screen.abandon_warning, None);
    // La fenêtre de fin ne recouvre pas la fenêtre de nouvelle partie.
    let under = client.game_screen(&catalog).expect("partie");
    assert_eq!(under.score_window, None);

    client.command(Command::StartNewGame);
    assert_eq!(files_in(dir.path()), before);
    assert_eq!(config(&client).seed, clock_seed());
    assert_eq!(game(&client).controls(&catalog).notice, None);
}

// --- Au lancement ---

/// Écrit la sauvegarde automatique de `session` dans `dir`.
fn autosave(dir: &Path, session: &GameSession) {
    let game = SaveGame::from_runner(session.runner(), "2026-10-30T12:00:00+01:00")
        .expect("partie commencée");
    save_auto(dir, &game).expect("écriture");
}

// C15 — spec 024.
#[test]
fn continue_loads_autosave() {
    let catalog = french();
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let mut saved = GameSession::new(GameConfig {
        hero: Hero::Goldyx,
        ..first_reconnaissance_config(9)
    });
    saved.run_frame();
    autosave(dir.path(), &saved);

    let mut client = launch(Some(dir.path()));
    let screen = client.new_game_screen(&catalog).expect("écran");
    let button = screen.continue_game.expect("Continuer");
    assert_eq!(
        button.label,
        "Continuer : Première reconnaissance · Goldyx · Manche 1/4 · Jour (tour 1)"
    );
    assert!(screen.cancel.is_none());
    client.command(button.command);
    client.run_frame();
    assert!(client.new_game_screen(&catalog).is_none());
    assert_eq!(game(&client).view(), saved.view());
}

// C16
#[test]
fn no_continue_without_autosave() {
    let catalog = french();
    let empty = tempfile::tempdir().expect("dossier temporaire");
    let screen = launch(Some(empty.path()))
        .new_game_screen(&catalog)
        .expect("écran");
    assert!(screen.continue_game.is_none());
    assert!(screen.load.is_some());

    let finished = tempfile::tempdir().expect("dossier temporaire");
    autosave(finished.path(), &finished_session());
    let screen = launch(Some(finished.path()))
        .new_game_screen(&catalog)
        .expect("écran");
    assert!(screen.continue_game.is_none());

    let unreadable = tempfile::tempdir().expect("dossier temporaire");
    fs::write(unreadable.path().join("auto.json"), "pas du JSON").expect("écriture");
    let screen = launch(Some(unreadable.path()))
        .new_game_screen(&catalog)
        .expect("écran");
    assert!(screen.continue_game.is_none());
}

// C17 et L5 — spec 024.
#[test]
fn load_from_new_game_screen() {
    let catalog = french();
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let mut saved = GameSession::new(GameConfig {
        hero: Hero::Tovak,
        ..first_reconnaissance_config(9)
    });
    saved.run_frame();
    autosave(dir.path(), &saved);

    let mut client = launch(Some(dir.path()));
    let load = client
        .new_game_screen(&catalog)
        .and_then(|screen| screen.load)
        .expect("Charger…");
    client.command(load.command);
    assert!(client.new_game_screen(&catalog).is_none());
    let window = client.load_window(&catalog).expect("fenêtre de chargement");
    let line = window.lines[0].load.clone().expect("lisible");
    client.command(line.command);
    client.run_frame();
    assert!(client.load_window(&catalog).is_none());
    assert!(client.new_game_screen(&catalog).is_none());
    assert_eq!(game(&client).view(), saved.view());
}

// C17 bis : fermer la fenêtre de chargement ramène à l'écran de nouvelle partie.
#[test]
fn closing_load_window_returns_to_new_game_screen() {
    let catalog = french();
    let mut client = launch(None);
    client.command(Command::OpenLoad);
    assert!(client.load_window(&catalog).is_some());
    client.shortcut(Shortcut::Cancel);
    assert!(client.load_window(&catalog).is_none());
    assert!(client.new_game_screen(&catalog).is_some());
}

// Ctrl+O au lancement fait comme « Charger… » (spec 024) ; en cours de partie, la fenêtre de
// nouvelle partie ouverte, il n'ouvre rien.
#[test]
fn ctrl_o_at_launch_opens_load_window() {
    let catalog = french();
    let mut client = launch(None);
    client.shortcut(Shortcut::OpenLoad);
    assert!(client.load_window(&catalog).is_some());
    assert!(client.new_game_screen(&catalog).is_none());

    let mut client = launch(None);
    start(&mut client, Hero::Arythea, ScenarioId::Sandbox, "5");
    client.shortcut(Shortcut::NewGame);
    client.shortcut(Shortcut::OpenLoad);
    assert!(client.load_window(&catalog).is_none());
    assert!(client.new_game_screen(&catalog).is_some());
}

// C18 — spec 024.
#[test]
fn autosave_is_kept_before_new_game() {
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let mut saved = GameSession::new(first_reconnaissance_config(9));
    saved.run_frame();
    autosave(dir.path(), &saved);

    let mut client = launch(Some(dir.path()));
    start(&mut client, Hero::Tovak, ScenarioId::Sandbox, "");
    let copy = arcanum_persistance::load(&dir.path().join(format!("{STEM}.json"))).expect("copie");
    assert_eq!(copy.summary.hero, Hero::Arythea);
    assert_eq!(list(dir.path()).len(), 2);
}

// C19
#[test]
fn every_new_game_key_exists_in_french() {
    let keys = [
        "new_game.title",
        "new_game.button",
        "new_game.hint",
        "new_game.hero",
        "new_game.scenario",
        "new_game.recap",
        "new_game.recap_choice",
        "new_game.scenarios",
        "new_game.heroes",
        "new_game.next",
        "new_game.advanced",
        "new_game.advanced_filled",
        "new_game.fact_length",
        "new_game.fact_players",
        "new_game.fact_kind",
        "new_game.objective",
        "new_game.special_rules",
        "new_game.game_end",
        "new_game.scoring",
        "new_game.style",
        "new_game.own_cards_title",
        "new_game.card_replaces",
        "new_game.card_basic",
        "new_game.card_strong",
        "new_game.seed",
        "new_game.seed_hint",
        "new_game.seed_invalid",
        "new_game.start",
        "new_game.cancel",
        "new_game.continue",
        "new_game.load",
        "new_game.abandon_warning",
        "new_game.previous_saved",
    ];
    for language in ["fr", "en"] {
        let catalog = catalog(language);
        for key in keys {
            assert!(!catalog.text(key).contains('⟦'), "{language} : {key}");
        }
    }
}

// --- Cas limites ---

// L1
#[test]
fn start_without_saves_dir() {
    let catalog = french();
    let mut client = launch(None);
    let screen = client.new_game_screen(&catalog).expect("écran");
    assert!(screen.continue_game.is_none());
    start(&mut client, Hero::Arythea, ScenarioId::Sandbox, "");
    assert!(client.game().is_some());
    // En cours de partie, sans dossier, rien à sauvegarder : la nouvelle partie démarre.
    client.shortcut(Shortcut::NewGame);
    client.command(Command::ChooseHero(Hero::Goldyx));
    client.command(Command::StartNewGame);
    assert_eq!(config(&client).hero, Hero::Goldyx);
}

// L2
#[test]
fn failed_save_keeps_current_game() {
    let catalog = french();
    let dir = tempfile::tempdir().expect("dossier temporaire");
    // Un fichier à la place du dossier : la sauvegarde échoue.
    let blocked = dir.path().join("bloque");
    fs::write(&blocked, "").expect("écriture");
    let mut client = launch(Some(&blocked));
    start(&mut client, Hero::Arythea, ScenarioId::Sandbox, "5");
    let before = game(&client).view();

    client.shortcut(Shortcut::NewGame);
    client.command(Command::ChooseHero(Hero::Goldyx));
    client.command(Command::StartNewGame);
    let screen = client
        .new_game_screen(&catalog)
        .expect("fenêtre restée ouverte");
    assert_eq!(screen.notice, Some(catalog.text("save.failed")));
    assert_eq!(config(&client).hero, Hero::Arythea);
    assert_eq!(game(&client).view(), before);
}

// L3
#[test]
fn escape_does_nothing_at_launch() {
    let catalog = french();
    let mut client = launch(None);
    let screen = client.new_game_screen(&catalog).expect("écran");
    assert!(screen.cancel.is_none());
    client.shortcut(Shortcut::Cancel);
    client.command(Command::CancelNewGame);
    assert!(client.new_game_screen(&catalog).is_some());
}

// L4
#[test]
fn no_new_game_window_during_a_choice() {
    let catalog = french();
    let mut client = launch(None);
    start(&mut client, Hero::Arythea, ScenarioId::Sandbox, "5");
    client.command(Command::OpenEndTurn);
    assert!(game(&client).is_ending_turn());
    assert!(!game(&client).controls(&catalog).new_game.enabled);
    client.shortcut(Shortcut::NewGame);
    client.command(Command::OpenNewGame);
    assert!(client.new_game_screen(&catalog).is_none());
}

// L7
#[test]
fn seed_text_is_limited_to_twenty_characters() {
    let mut client = launch(None);
    client.set_seed_text("123456789012345678901234");
    assert_eq!(client.seed_text(), Some("12345678901234567890"));
}

// Entrée commence aussi au lancement, graine valide seulement.
#[test]
fn enter_starts_when_seed_is_valid() {
    let mut client = launch(None);
    client.set_seed_text("abc");
    client.shortcut(Shortcut::Confirm);
    assert!(client.game().is_none());
    client.set_seed_text("8");
    client.shortcut(Shortcut::Confirm);
    assert_eq!(config(&client).seed, 8);
}

// --- Étapes, volet et touches (spec 078) ---

// Spec 078, C6.
#[test]
fn next_goes_to_hero_step() {
    let catalog = french();
    let mut client = launch(None);
    let next = client
        .new_game_screen(&catalog)
        .and_then(|screen| screen.next)
        .expect("Suivant");
    client.command(next.command);
    let screen = client.new_game_screen(&catalog).expect("écran");
    assert_eq!(screen.step, NewGameStep::Hero);
    assert_eq!(screen.next, None);
    assert_eq!(hero_sheet_of(&screen).name, "Arythea");
}

// Spec 078, C7 et L1.
#[test]
fn step_bar_switches_steps() {
    let catalog = french();
    let mut client = launch(None);
    client.command(Command::ChooseScenario(ScenarioId::Sandbox));
    client.command(Command::ToggleAdvanced);
    client.set_seed_text("7");
    client.command(Command::ChooseStep(NewGameStep::Hero));
    let screen = client.new_game_screen(&catalog).expect("écran");
    assert_eq!(screen.step, NewGameStep::Hero);
    assert_eq!(screen.list_title, "Héros");
    client.command(Command::ChooseHero(Hero::Norowas));

    client.command(Command::ChooseStep(NewGameStep::Scenario));
    let screen = client.new_game_screen(&catalog).expect("écran");
    assert_eq!(screen.step, NewGameStep::Scenario);
    let active: Vec<bool> = screen.steps.iter().map(|tab| tab.active).collect();
    assert_eq!(active, [true, false, false]);
    assert_eq!(screen.recap, "Bac à sable · Norowas");
    assert!(screen.advanced.open);
    assert_eq!(client.seed_text(), Some("7"));
    // L1 : un autre scénario ne change pas le héros.
    client.command(Command::ChooseScenario(ScenarioId::FirstReconnaissance));
    let screen = client.new_game_screen(&catalog).expect("écran");
    assert_eq!(screen.recap, "Première reconnaissance · Norowas");
}

// Spec 078, C12.
#[test]
fn start_from_scenario_step_uses_checked_hero() {
    let mut client = launch(None);
    client.command(Command::ChooseScenario(ScenarioId::Sandbox));
    client.command(Command::StartNewGame);
    client.run_frame();
    let config = config(&client);
    assert_eq!(config.hero, Hero::Arythea);
    assert_eq!(config.scenario.id(), ScenarioId::Sandbox);
}

// Spec 078, C13.
#[test]
fn closed_advanced_keeps_the_seed() {
    let catalog = french();
    let mut client = launch(None);
    let screen = client.new_game_screen(&catalog).expect("écran");
    assert!(!screen.advanced.open);
    client.command(screen.advanced.command);
    let screen = client.new_game_screen(&catalog).expect("écran");
    assert!(screen.advanced.open);
    assert_eq!(screen.seed_label, catalog.text("new_game.seed"));
    assert_eq!(screen.seed_hint, catalog.text("new_game.seed_hint"));
    client.set_seed_text("42");
    client.command(Command::ToggleAdvanced);
    let screen = client.new_game_screen(&catalog).expect("écran");
    assert!(!screen.advanced.open);
    assert_eq!(screen.advanced.label, "Avancé (graine saisie)");
    assert_eq!(client.seed_text(), Some("42"));
    client.command(Command::StartNewGame);
    client.run_frame();
    assert_eq!(config(&client).seed, 42);
}

// Spec 078, C14 : graine invalide, le volet ne se referme pas.
#[test]
fn invalid_seed_refuses_to_close_advanced() {
    let catalog = french();
    let mut client = launch(None);
    client.command(Command::ToggleAdvanced);
    client.set_seed_text("abc");
    client.command(Command::ToggleAdvanced);
    assert!(
        client
            .new_game_screen(&catalog)
            .expect("écran")
            .advanced
            .open
    );
    // Corrigée, la graine laisse le volet tel que le joueur l'a laissé : ouvert.
    client.set_seed_text("42");
    assert!(
        client
            .new_game_screen(&catalog)
            .expect("écran")
            .advanced
            .open
    );
}

// Spec 078, C16 — Q1 : au lancement, les choix de la sauvegarde automatique lisible.
#[test]
fn launch_preselects_last_game() {
    let catalog = french();
    let selected = |client: &ClientSession| {
        let screen = client.new_game_screen(&catalog).expect("écran");
        let hero = screen.heroes.iter().find(|h| h.selected).map(|h| h.hero);
        let scenario = screen
            .scenarios
            .iter()
            .find(|s| s.selected)
            .map(|s| s.scenario);
        (scenario, hero)
    };
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let mut saved = GameSession::new(GameConfig {
        hero: Hero::Goldyx,
        scenario: Scenario::sandbox(),
        ..first_reconnaissance_config(9)
    });
    saved.run_frame();
    autosave(dir.path(), &saved);
    let client = launch(Some(dir.path()));
    assert_eq!(
        selected(&client),
        (Some(ScenarioId::Sandbox), Some(Hero::Goldyx))
    );
    assert!(
        client
            .new_game_screen(&catalog)
            .expect("écran")
            .continue_game
            .is_some()
    );

    // Partie terminée (Goldyx, Première reconnaissance) : ses choix aussi, sans « Continuer ».
    let finished = tempfile::tempdir().expect("dossier temporaire");
    autosave(finished.path(), &finished_session());
    let client = launch(Some(finished.path()));
    assert_eq!(
        selected(&client),
        (Some(ScenarioId::FirstReconnaissance), Some(Hero::Goldyx))
    );
    assert!(
        client
            .new_game_screen(&catalog)
            .expect("écran")
            .continue_game
            .is_none()
    );

    let unreadable = tempfile::tempdir().expect("dossier temporaire");
    fs::write(unreadable.path().join("auto.json"), "pas du JSON").expect("écriture");
    assert_eq!(
        selected(&launch(Some(unreadable.path()))),
        (Some(ScenarioId::FirstReconnaissance), Some(Hero::Arythea))
    );
}

// Spec 078, C18 — Q3.
#[test]
fn arrows_choose_and_switch_steps() {
    let catalog = french();
    let mut client = launch(None);
    let recap = |client: &ClientSession| client.new_game_screen(&catalog).expect("écran").recap;
    // Spec 064, Q4 : Conquête en solitaire entre les deux.
    client.shortcut(Shortcut::NextChoice);
    assert_eq!(recap(&client), "Conquête en solitaire · Arythea");
    client.shortcut(Shortcut::NextChoice);
    assert_eq!(recap(&client), "Bac à sable · Arythea");
    client.shortcut(Shortcut::NextChoice);
    assert_eq!(recap(&client), "Bac à sable · Arythea");
    client.shortcut(Shortcut::PreviousChoice);
    assert_eq!(recap(&client), "Conquête en solitaire · Arythea");
    client.shortcut(Shortcut::PreviousChoice);
    assert_eq!(recap(&client), "Première reconnaissance · Arythea");
    client.shortcut(Shortcut::PreviousChoice);
    assert_eq!(recap(&client), "Première reconnaissance · Arythea");

    client.shortcut(Shortcut::NextStep);
    assert_eq!(
        client.new_game_screen(&catalog).expect("écran").step,
        NewGameStep::Hero
    );
    for _ in 0..4 {
        client.shortcut(Shortcut::NextChoice);
    }
    assert_eq!(recap(&client), "Première reconnaissance · Tovak");
    client.shortcut(Shortcut::PreviousChoice);
    assert_eq!(recap(&client), "Première reconnaissance · Norowas");
    // Spec 061 : → mène à l'étape Options, la dernière.
    client.shortcut(Shortcut::NextStep);
    client.shortcut(Shortcut::NextStep);
    assert_eq!(
        client.new_game_screen(&catalog).expect("écran").step,
        NewGameStep::Options
    );
    client.shortcut(Shortcut::PreviousStep);
    assert_eq!(
        client.new_game_screen(&catalog).expect("écran").step,
        NewGameStep::Hero
    );
    client.shortcut(Shortcut::PreviousStep);
    assert_eq!(
        client.new_game_screen(&catalog).expect("écran").step,
        NewGameStep::Scenario
    );
}

// Spec 078, C18 : les flèches n'ont pas d'effet sur la partie, écran fermé.
#[test]
fn arrows_do_nothing_in_game() {
    let mut client = launch(None);
    start(&mut client, Hero::Arythea, ScenarioId::Sandbox, "5");
    let before = game(&client).view();
    for shortcut in [
        Shortcut::NextChoice,
        Shortcut::PreviousChoice,
        Shortcut::NextStep,
        Shortcut::PreviousStep,
    ] {
        client.shortcut(shortcut);
    }
    assert_eq!(game(&client).view(), before);
    assert!(client.new_game_screen(&french()).is_none());
}

// Spec 078, L2 : fermer la fenêtre de chargement ramène à l'étape où l'on était.
#[test]
fn closing_load_window_keeps_the_step() {
    let catalog = french();
    let mut client = launch(None);
    client.command(Command::ChooseStep(NewGameStep::Hero));
    client.command(Command::OpenLoad);
    client.command(Command::CloseLoad);
    assert_eq!(
        client.new_game_screen(&catalog).expect("écran").step,
        NewGameStep::Hero
    );
}

// Spec 078, L4 : Échap ferme l'écran en cours de partie, quelle que soit l'étape.
#[test]
fn escape_closes_from_hero_step_in_game() {
    let catalog = french();
    let mut client = launch(None);
    start(&mut client, Hero::Arythea, ScenarioId::Sandbox, "5");
    client.shortcut(Shortcut::NewGame);
    client.command(Command::ChooseStep(NewGameStep::Hero));
    client.shortcut(Shortcut::Cancel);
    assert!(client.new_game_screen(&catalog).is_none());
}

// Spec 078, C9b : la fiche et la liste portent le portrait généré du héros.
#[test]
fn hero_portraits_are_hero_art() {
    let catalog = french();
    for hero in Hero::ALL {
        let form = form_at(NewGameStep::Hero, hero, ScenarioId::FirstReconnaissance);
        let screen = new_game_screen(&form, &launch_context(), &catalog);
        let expected = hero_illustration(hero);
        assert_eq!(expected.family, IllustrationFamily::Hero);
        assert_eq!(expected.id, format!("{hero:?}").to_lowercase());
        assert_eq!(hero_sheet_of(&screen).portrait, expected);
        let choice = screen
            .heroes
            .iter()
            .find(|choice| choice.hero == hero)
            .expect("héros de la liste");
        assert_eq!(choice.portrait, expected);
    }
    assert!(
        known_illustrations(IllustrationFamily::Hero).contains(&hero_illustration(Hero::Tovak))
    );
}

// Spec 078, C9b : les quatre portraits sont dans assets/images/hero-art/, et l'index les connaît.
#[test]
fn every_hero_portrait_exists() {
    let images_dir = repository_assets_dir().join("images");
    let index = IllustrationIndex::load(&images_dir);
    for hero in Hero::ALL {
        let portrait = hero_illustration(hero);
        assert!(portrait.path(&images_dir).is_file(), "{portrait:?}");
        assert!(index.contains(portrait), "{portrait:?}");
    }
}
