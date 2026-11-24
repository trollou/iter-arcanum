//! Spec 061 côté affichage (Joueur fantôme) : section du panneau de droite, fenêtre de sa
//! défausse, ligne d'ordre du tour, bandeau du dernier tour, étape Options de la nouvelle partie
//! et fiche de Première reconnaissance.

use std::path::Path;

use arcanum_core::{
    Action, BasicColor, CardId, DummyHeroChoice, Event, GameConfig, GameState, Hero, Scenario,
    ScenarioId, Skill, Tactic, TimeOfDay, apply, legal_actions, view,
};
use arcanum_i18n::{Catalog, Language};
use arcanum_persistance::{SaveGame, save_auto};
use arcanum_ui::{
    ClientSession, Command, DummyOption, GameScreen, NewGameScreen, NewGameSheet, NewGameStep,
    OptionsSheet, ScreenMode, ScreenPhase, Shortcut, dummy_discard_window, game_screen,
    repository_assets_dir, scenario_sheet, sheet_keys_of_scenario,
};

// L'arène des tests d'arcanum-core, partagée plutôt que recopiée.
#[path = "../../arcanum-core/tests/arene/mod.rs"]
mod arene;
use arene::*;

mod support;
use support::session_of;

use arcanum_core::BasicAction::{
    Concentration, Determination, Improvisation, March, Promise, Rage, Stamina, Swiftness,
    Threaten, Tranquility,
};

// --- Outils ---

fn french() -> Catalog {
    Catalog::load(
        &repository_assets_dir().join("i18n"),
        &Language("fr".into()),
    )
    .expect("fr.json du dépôt")
}

fn state_of(events: &[Event]) -> GameState {
    events.iter().fold(GameState::default(), apply)
}

fn then(mut events: Vec<Event>, action: Action) -> Vec<Event> {
    let state = state_of(&events);
    assert!(
        legal_actions(&state, SOLO).contains(&action),
        "{action:?} doit être légale"
    );
    events.extend(events_of(&state, &action));
    events
}

fn screen_of(events: &[Event], catalog: &Catalog) -> GameScreen {
    let state = state_of(events);
    let legal = legal_actions(&state, SOLO);
    game_screen(
        &view(&state, SOLO),
        ScreenPhase::Playing {
            legal_actions: &legal,
        },
        &ScreenMode::Normal,
        catalog,
    )
}

/// Le deck du fantôme, dessus en premier : Rage, Célérité, Marche (verte), Endurance, Promesse,
/// puis 11 autres cartes (exemplaires n° 101 et plus, spec 061, *Identité des cartes*).
fn dummy_deck() -> Vec<CardId> {
    let mut deck = vec![
        card(Rage, 101),
        card(Swiftness, 101),
        card(March, 101),
        card(Stamina, 101),
        card(Promise, 101),
    ];
    deck.extend(
        [
            Threaten,
            Determination,
            Tranquility,
            Concentration,
            Improvisation,
            Rage,
            Swiftness,
            March,
            Stamina,
            Promise,
            Threaten,
        ]
        .into_iter()
        .zip([101, 101, 101, 101, 101, 102, 102, 102, 102, 102, 102])
        .map(|(action, copy)| card(action, copy)),
    );
    deck
}

/// Première reconnaissance avec fantôme : Tovak sur une plaine, la tactique de jour à choisir ;
/// deck du joueur vide ; le fantôme Goldyx (cristaux vert, vert, bleu), deux compétences dans sa pile.
fn decor() -> Vec<Event> {
    let hand = vec![card(March, 1), card(Rage, 1)];
    let mut events = Sandbox {
        scenario: Scenario::first_reconnaissance(),
        hero: Hero::Tovak,
        figure: Some(PLAIN_W),
        hand: hand.clone(),
        time: TimeOfDay::Day,
        tactic_pending: true,
        ..Sandbox::default()
    }
    .events();
    // Deck du joueur vide : l'annonce de la fin de la manche est possible (C40).
    events.push(Event::DeckShuffled {
        player: SOLO,
        order: hand.clone(),
    });
    events.extend(
        hand.iter()
            .map(|&card| Event::CardDrawn { player: SOLO, card }),
    );
    events.push(Event::DummySetUp {
        hero: Hero::Goldyx,
        crystals: vec![BasicColor::Green, BasicColor::Green, BasicColor::Blue],
        skills: vec![Skill::FreezingPower, Skill::PotionMaking],
        deck: dummy_deck(),
    });
    events
}

/// Le joueur prend Planification (4) ; le fantôme tire Faux départ (2, générateur des tests) et
/// joue avant le tour 1 : Rage, Célérité, Marche, puis 2 de plus pour ses 2 cristaux verts.
fn dummy_played_first() -> Vec<Event> {
    then(
        decor(),
        Action::ChooseTactic {
            tactic: Tactic::Planning,
        },
    )
}

// --- Section du panneau ---

// Spec 061, C37 (C31) — MKUE p. 15, *Note 1* : la section montre le héros, la tactique, le deck en
// évidence, la défausse, les cristaux, les compétences cachées et le dernier tour.
#[test]
fn dummy_panel_lines() {
    let catalog = french();
    let events = dummy_played_first();
    assert!(
        events.contains(&Event::DummyTacticDrawn {
            tactic: Tactic::Rethink
        }),
        "décor : le fantôme tire Faux départ"
    );
    let section = screen_of(&events, &catalog)
        .dummy
        .expect("la section du fantôme");
    assert_eq!(section.portrait.name, "Goldyx");
    assert_eq!(section.title, "Joueur fantôme : Goldyx");
    assert_eq!(section.tactic.as_deref(), Some("Tactique : 2 Faux départ"));
    assert_eq!(section.deck, "Deck : 11 carte(s)");
    assert_eq!(section.discard.label, "Défausse : 5 carte(s)");
    assert_eq!(section.discard.command, Some(Command::OpenDummyDiscard));
    // Retour de Mathieu : l'icône colorée de l'inventaire, une par cristal, à la suite, rangées par
    // couleur, sans emplacement vide.
    assert_eq!(section.crystals.title, "Cristaux");
    assert_eq!(
        section.crystals.crystals,
        [BasicColor::Blue, BasicColor::Green, BasicColor::Green]
    );
    assert_eq!(section.crystals.tooltip, "bleu : 1, vert : 2");
    assert_eq!(section.skills, "Compétences cachées : 2");
    assert_eq!(
        section.last_turn,
        "Dernier tour : Rage, Célérité, Marche, Endurance, Promesse ; vert : 2 de plus"
    );
    assert_eq!(section.announced, None);
}

// Spec 061, C37 : avant son premier tour, ni carte retournée ni défausse (bouton grisé).
#[test]
fn dummy_panel_before_its_first_turn() {
    let catalog = french();
    let events = then(
        decor(),
        Action::ChooseTactic {
            tactic: Tactic::EarlyBird,
        },
    );
    let section = screen_of(&events, &catalog)
        .dummy
        .expect("la section du fantôme");
    assert_eq!(section.deck, "Deck : 16 carte(s)");
    assert_eq!(section.discard.label, "Défausse : 0 carte(s)");
    assert_eq!(section.discard.command, None);
    assert_eq!(section.last_turn, "Dernier tour : aucune carte");
}

// Spec 061, C37 : trois cartes retournées (tour posé), pas de « de plus » ; tactique pas encore
// tirée.
#[test]
fn dummy_panel_three_cards_without_extra() {
    let catalog = french();
    let mut events = decor();
    events.push(Event::DummyTurnPlayed {
        cards: vec![card(Rage, 101), card(Swiftness, 101), card(March, 101)],
    });
    let section = screen_of(&events, &catalog)
        .dummy
        .expect("la section du fantôme");
    assert_eq!(section.last_turn, "Dernier tour : Rage, Célérité, Marche");
    assert_eq!(section.tactic, None);
}

// Spec 061, C37 : sans fantôme (bac à sable, ou case décochée), ni section, ni ligne d'ordre du
// tour.
#[test]
fn no_dummy_section_without_dummy() {
    let catalog = french();
    let events = Sandbox {
        figure: Some(PLAIN_W),
        hand: vec![card(March, 1)],
        ..Sandbox::default()
    }
    .events();
    let screen = screen_of(&events, &catalog);
    assert_eq!(screen.dummy, None);
    assert_eq!(screen.round_order, None);
}

// --- Fenêtre de la défausse ---

// Spec 061, C38 : un clic sur la défausse du fantôme ouvre la fenêtre de la spec 073 avec ses
// cartes, la dernière retournée en tête, et le titre « Défausse du joueur fantôme ».
#[test]
fn dummy_discard_window_lists_cards() {
    let catalog = french();
    let events = dummy_played_first();
    let state = state_of(&events);
    let window = dummy_discard_window(&view(&state, SOLO), &catalog).expect("la fenêtre");
    assert_eq!(window.title, "Défausse du joueur fantôme");
    let names: Vec<&str> = window.cards.iter().map(|card| card.name.as_str()).collect();
    assert_eq!(
        names,
        ["Promesse", "Endurance", "Marche", "Célérité", "Rage"]
    );
    assert_eq!(window.close.command, Command::CloseDiscard);

    let mut session = session_of(events);
    assert_eq!(session.discard_window(&catalog), None);
    session.command(Command::OpenDummyDiscard);
    assert_eq!(session.discard_window(&catalog), Some(window));
    session.command(Command::CloseDiscard);
    assert_eq!(session.discard_window(&catalog), None);
}

// Spec 061, C38 : défausse du fantôme vide, pas de fenêtre.
#[test]
fn no_dummy_discard_window_when_empty() {
    let catalog = french();
    let state = state_of(&decor());
    assert_eq!(dummy_discard_window(&view(&state, SOLO), &catalog), None);
}

// --- Ordre du tour ---

// Spec 061, C39 — MKUE p. 4, 3.c : la ligne d'ordre du tour suit les deux tactiques.
#[test]
fn round_order_line() {
    let catalog = french();
    let dummy_first = screen_of(&dummy_played_first(), &catalog);
    assert_eq!(
        dummy_first.round_order.as_deref(),
        Some("Le joueur fantôme joue en premier")
    );
    let player_first = then(
        decor(),
        Action::ChooseTactic {
            tactic: Tactic::EarlyBird,
        },
    );
    assert_eq!(
        screen_of(&player_first, &catalog).round_order.as_deref(),
        Some("Vous jouez en premier")
    );
    // Avant les tactiques, l'ordre n'est pas connu.
    assert_eq!(screen_of(&decor(), &catalog).round_order, None);
}

// --- Dernier tour ---

// Spec 061, C40 — MKUE p. 15 : le fantôme a annoncé la fin de la manche ; un seul message, dans sa
// section (retour de Mathieu), et plus de bouton « Finir la manche… ».
#[test]
fn dummy_announcement_line() {
    let catalog = french();
    let events = dummy_played_first();
    let announce_label = catalog.text("round.announce.button_short");
    let has_announce = |screen: &GameScreen| {
        screen
            .bottom_buttons
            .iter()
            .any(|button| button.label == announce_label)
    };
    let before = screen_of(&events, &catalog);
    assert!(has_announce(&before), "décor : deck vide, annonce possible");

    let mut announced = events;
    announced.push(Event::DummyAnnouncedEndOfRound);
    let screen = screen_of(&announced, &catalog);
    let section = screen.dummy.clone().expect("la section du fantôme");
    assert_eq!(
        section.announced.as_deref(),
        Some("A annoncé la fin de la manche : c'est votre dernier tour.")
    );
    assert!(!has_announce(&screen));
}

// --- Écran de nouvelle partie ---

fn launch(dir: Option<&Path>) -> ClientSession {
    ClientSession::launch(dir.map(Path::to_path_buf)).with_seed_source(|| 7)
}

fn screen_at(client: &ClientSession, catalog: &Catalog) -> NewGameScreen {
    client
        .new_game_screen(catalog)
        .expect("écran de nouvelle partie")
}

fn options_of(screen: &NewGameScreen) -> &OptionsSheet {
    match &screen.sheet {
        NewGameSheet::Options(sheet) => sheet,
        other => panic!("étape Options attendue : {other:?}"),
    }
}

fn dummy_option(screen: &NewGameScreen) -> &DummyOption {
    options_of(screen)
        .dummy
        .as_ref()
        .expect("la case du fantôme")
}

fn started_config(client: &ClientSession) -> GameConfig {
    client
        .game()
        .expect("une partie en cours")
        .config()
        .expect("partie commencée")
}

fn start(client: &mut ClientSession) {
    client.command(Command::StartNewGame);
    client.run_frame();
}

// Spec 061, C41 — Q1 : l'étape Options suit Héros ; la case du fantôme est cochée par défaut pour
// Première reconnaissance ; décochée, la partie se crée sans fantôme et le récapitulatif le dit.
#[test]
fn options_step_toggles_dummy() {
    let catalog = french();
    let mut client = launch(None);
    let tabs: Vec<(u8, String)> = screen_at(&client, &catalog)
        .steps
        .iter()
        .map(|tab| (tab.number, tab.label.clone()))
        .collect();
    assert_eq!(
        tabs,
        [
            (1, "Scénario".to_owned()),
            (2, "Héros".to_owned()),
            (3, "Options".to_owned())
        ]
    );
    client.command(Command::ChooseStep(NewGameStep::Hero));
    client.shortcut(Shortcut::NextStep);
    let screen = screen_at(&client, &catalog);
    assert_eq!(screen.step, NewGameStep::Options);
    assert_eq!(screen.list_title, "Options");
    let option = dummy_option(&screen);
    assert_eq!(option.label, "Joueur fantôme");
    assert!(option.checked);
    assert!(option.hero_enabled);
    assert_eq!(option.command, Command::ToggleDummy);
    assert_eq!(option.help, catalog.text("new_game.options.dummy_help"));
    assert_eq!(options_of(&screen).none, None);
    assert_eq!(screen.recap, "Première reconnaissance · Arythea");

    client.command(Command::ToggleDummy);
    let screen = screen_at(&client, &catalog);
    let option = dummy_option(&screen);
    assert!(!option.checked);
    assert!(!option.hero_enabled);
    assert_eq!(
        screen.recap,
        "Première reconnaissance · Arythea · sans joueur fantôme"
    );
    client.shortcut(Shortcut::PreviousStep);
    assert_eq!(screen_at(&client, &catalog).step, NewGameStep::Hero);

    start(&mut client);
    let config = started_config(&client);
    assert_eq!(
        config.scenario,
        Scenario::first_reconnaissance().without_dummy()
    );
    assert_eq!(client.game().expect("partie").view().dummy, None);
}

// Spec 061, C41 : le bac à sable n'a pas d'option.
#[test]
fn sandbox_has_no_option() {
    let catalog = french();
    let mut client = launch(None);
    client.command(Command::ChooseScenario(ScenarioId::Sandbox));
    client.command(Command::ChooseStep(NewGameStep::Options));
    let screen = screen_at(&client, &catalog);
    let options = options_of(&screen);
    assert_eq!(options.dummy, None);
    assert_eq!(
        options.none.as_deref(),
        Some("Pas d'option pour ce scénario.")
    );
    assert_eq!(screen.recap, "Bac à sable · Arythea");
}

// Spec 061, C41 — Q2 : la liste du héros du fantôme, « Au hasard » en tête puis les trois autres
// héros ; un héros choisi passe dans la partie et le récapitulatif ; prendre ce héros à l'étape
// Héros ramène la liste à « Au hasard ».
#[test]
fn options_step_chooses_dummy_hero() {
    let catalog = french();
    let mut client = launch(None);
    client.command(Command::ChooseHero(Hero::Tovak));
    client.command(Command::ChooseStep(NewGameStep::Options));
    let screen = screen_at(&client, &catalog);
    let option = dummy_option(&screen);
    assert_eq!(option.hero_label, "Héros du joueur fantôme");
    assert_eq!(option.hero_selected, "Au hasard");
    let items: Vec<(&str, bool, Command)> = option
        .heroes
        .iter()
        .map(|item| (item.label.as_str(), item.selected, item.command))
        .collect();
    let choose = |hero| Command::ChooseDummyHero(DummyHeroChoice::Hero(hero));
    assert_eq!(
        items,
        [
            (
                "Au hasard",
                true,
                Command::ChooseDummyHero(DummyHeroChoice::Random)
            ),
            ("Arythea", false, choose(Hero::Arythea)),
            ("Goldyx", false, choose(Hero::Goldyx)),
            ("Norowas", false, choose(Hero::Norowas)),
        ]
    );

    client.command(choose(Hero::Goldyx));
    let screen = screen_at(&client, &catalog);
    assert_eq!(dummy_option(&screen).hero_selected, "Goldyx");
    assert_eq!(
        screen.recap,
        "Première reconnaissance · Tovak · joueur fantôme : Goldyx"
    );

    // Le joueur prend Goldyx : la liste revient à « Au hasard ».
    client.command(Command::ChooseHero(Hero::Goldyx));
    let screen = screen_at(&client, &catalog);
    assert_eq!(dummy_option(&screen).hero_selected, "Au hasard");
    assert_eq!(screen.recap, "Première reconnaissance · Goldyx");

    client.command(Command::ChooseHero(Hero::Tovak));
    client.command(choose(Hero::Norowas));
    start(&mut client);
    let config = started_config(&client);
    assert_eq!(config.dummy_hero, DummyHeroChoice::Hero(Hero::Norowas));
    let dummy = client
        .game()
        .expect("partie")
        .view()
        .dummy
        .expect("le fantôme");
    assert_eq!(dummy.hero, Hero::Norowas);
}

// Spec 061, C41 (retour de Mathieu du 2026-11-19) : la case et le héros du fantôme ne sont pas
// retenus ; chaque nouvelle partie repart de la case cochée et de « Au hasard », en cours de partie
// (Ctrl+N) comme au lancement (sauvegarde automatique). Scénario et héros restent repris (spec 078).
#[test]
fn options_reset_to_defaults_for_next_game() {
    let catalog = french();
    let mut client = launch(None);
    client.command(Command::ChooseHero(Hero::Tovak));
    client.command(Command::ChooseDummyHero(DummyHeroChoice::Hero(
        Hero::Goldyx,
    )));
    client.command(Command::ToggleDummy);
    start(&mut client);
    client.shortcut(Shortcut::NewGame);
    client.command(Command::ChooseStep(NewGameStep::Options));
    let screen = screen_at(&client, &catalog);
    assert!(dummy_option(&screen).checked);
    assert_eq!(dummy_option(&screen).hero_selected, "Au hasard");
    assert_eq!(screen.recap, "Première reconnaissance · Tovak");

    client.command(Command::ChooseDummyHero(DummyHeroChoice::Hero(
        Hero::Norowas,
    )));
    start(&mut client);
    let dir = tempfile::tempdir().expect("dossier temporaire");
    let game = SaveGame::from_runner(
        client.game().expect("partie").runner(),
        "2026-11-19T12:00:00+01:00",
    )
    .expect("partie commencée");
    save_auto(dir.path(), &game).expect("écriture");
    let mut relaunched = launch(Some(dir.path()));
    relaunched.command(Command::ChooseStep(NewGameStep::Options));
    let screen = screen_at(&relaunched, &catalog);
    assert!(dummy_option(&screen).checked);
    assert_eq!(dummy_option(&screen).hero_selected, "Au hasard");
    assert_eq!(screen.recap, "Première reconnaissance · Tovak");
}

// --- Fiche du scénario ---

// Spec 061, C42 : plus de ligne « Pas encore dans le jeu » ; les règles spéciales parlent du
// fantôme.
#[test]
fn first_reconnaissance_sheet_mentions_dummy() {
    let catalog = french();
    let sheet = scenario_sheet(ScenarioId::FirstReconnaissance, &catalog);
    assert_eq!(sheet.missing, None);
    let rules = &sheet.sections[1].bullets;
    assert_eq!(rules.len(), 4);
    assert_eq!(
        rules[3],
        "Joueur fantôme : il retourne des cartes de son deck à chaque tour et annonce la fin de la \
         manche quand il est vide."
    );
    assert!(
        !sheet_keys_of_scenario(ScenarioId::FirstReconnaissance)
            .contains(&"scenario.first_reconnaissance.missing".to_owned())
    );
}
