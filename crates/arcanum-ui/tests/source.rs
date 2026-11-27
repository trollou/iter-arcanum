//! Scénarios de la spec 020 côté affichage (la Source dans le panneau latéral).

use arcanum_core::{
    BasicColor, Event, GameConfig, GameState, Hero, ManaColor, PlayerId, PlayerView, Scenario,
    TimeOfDay, apply, legal_actions, view,
};
use arcanum_i18n::{Catalog, Language};
use arcanum_ui::{
    GameScreen, GameSession, ScreenMode, ScreenPhase, game_screen, repository_assets_dir,
};

mod support;
use support::first_reconnaissance_config;

const SOLO: PlayerId = PlayerId::SOLO;

const RED: ManaColor = ManaColor::Basic(BasicColor::Red);
const BLUE: ManaColor = ManaColor::Basic(BasicColor::Blue);
const WHITE: ManaColor = ManaColor::Basic(BasicColor::White);
const GREEN: ManaColor = ManaColor::Basic(BasicColor::Green);

fn french() -> Catalog {
    Catalog::load(
        &repository_assets_dir().join("i18n"),
        &Language("fr".into()),
    )
    .expect("fr.json du dépôt")
}

/// Partie réduite à son démarrage et à la Source `dice`.
fn with_source(dice: &[ManaColor]) -> GameState {
    let config = GameConfig {
        seed: 0,
        scenario: Scenario::sandbox(),
        hero: Hero::Arythea,
        dummy_hero: Default::default(),
    };
    [
        Event::GameStarted { config },
        Event::SourceRolled {
            dice: dice.to_vec(),
        },
    ]
    .iter()
    .fold(GameState::default(), apply)
}

fn screen_at(state: &GameState, time: TimeOfDay, catalog: &Catalog) -> GameScreen {
    let legal = legal_actions(state, SOLO);
    let view = PlayerView {
        // La nuit n'arrive qu'avec la spec 023 : on la pose sur la vue.
        time_of_day: time,
        ..view(state, SOLO)
    };
    game_screen(
        &view,
        ScreenPhase::Playing {
            legal_actions: &legal,
        },
        &ScreenMode::Normal,
        catalog,
    )
}

fn names(dice: &[arcanum_ui::SourceDie]) -> Vec<&str> {
    dice.iter().map(|die| die.name.as_str()).collect()
}

// Spec 020 C12 — MKUE p. 5, 3.c
#[test]
fn panel_shows_source_dice() {
    let catalog = french();
    let state = with_source(&[ManaColor::Black, GREEN, BLUE]);
    let source = screen_at(&state, TimeOfDay::Day, &catalog).source;
    assert_eq!(source.title, "Source");
    assert_eq!(
        names(&source.dice),
        ["<green>vert</green>", "<blue>bleu</blue>"]
    );
    assert_eq!(source.dice[0].color, GREEN);
    assert_eq!(source.depleted_title.as_deref(), Some("épuisé :"));
    assert_eq!(names(&source.depleted), ["<black>noir</black>"]);

    let night = screen_at(&state, TimeOfDay::Night, &catalog).source;
    assert_eq!(
        names(&night.dice),
        [
            "<black>noir</black>",
            "<green>vert</green>",
            "<blue>bleu</blue>"
        ]
    );
    assert_eq!(night.depleted_title, None);
    assert!(night.depleted.is_empty());

    // La partie lancée par l'application montre ses trois dés.
    let mut session = GameSession::new(first_reconnaissance_config(42));
    session.run_frame();
    let source = session.screen(&catalog).source;
    assert_eq!(source.dice.len() + source.depleted.len(), 3);
}

// Spec 020 C13 — MKUE p. 5, 2.a, 2.b et 3.c
#[test]
fn source_die_tooltips() {
    let catalog = french();
    let state = with_source(&[BLUE, ManaColor::Gold, ManaColor::Black]);
    let day = screen_at(&state, TimeOfDay::Day, &catalog).source;
    assert_eq!(day.dice[0].tooltip, "Dé de mana <blue>bleu</blue>");
    assert_eq!(
        day.dice[1].tooltip,
        "Dé de mana or : peut servir de mana de n'importe quelle couleur de base, de jour."
    );
    assert_eq!(
        day.depleted[0].tooltip,
        "Dé de mana <black>noir</black> : épuisé, inutilisable de jour."
    );

    let night = screen_at(&state, TimeOfDay::Night, &catalog).source;
    assert_eq!(
        night.dice[1].tooltip,
        "Dé de mana noir : alimente certains effets, de nuit."
    );
    assert_eq!(
        night.depleted[0].tooltip,
        "Dé de mana <gold>or</gold> : épuisé, inutilisable de nuit."
    );
}

// Spec 020 C14
#[test]
fn every_source_key_exists_in_french() {
    let catalog = french();
    let state = with_source(&[RED, BLUE, WHITE, GREEN, ManaColor::Gold, ManaColor::Black]);
    for time in [TimeOfDay::Day, TimeOfDay::Night] {
        screen_at(&state, time, &catalog);
    }
    assert_eq!(catalog.gaps(), vec![]);
}
