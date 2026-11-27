//! Scénarios de la spec 040 côté affichage (Interaction et réputation).

use arcanum_core::UndoStatus;
use arcanum_core::{
    Action, BasicAction, CardId, Event, EventLog, GameRunner, GameState, RngPositions, apply,
    legal_actions, view,
};
use arcanum_i18n::{Catalog, Language};
use arcanum_ui::{
    CombatChoice, Command, GameScreen, GameSession, ScreenMode, ScreenPhase, Shortcut,
    combat_screen, game_screen, human_player, repository_assets_dir,
};

// L'arène des tests d'arcanum-core, partagée plutôt que recopiée.
#[path = "../../arcanum-core/tests/arene/mod.rs"]
mod arene;
use arene::*;

use BasicAction::*;

fn french() -> Catalog {
    Catalog::load(
        &repository_assets_dir().join("i18n"),
        &Language("fr".into()),
    )
    .expect("fr.json du dépôt")
}

fn screen_of(state: &GameState, catalog: &Catalog) -> GameScreen {
    let legal = legal_actions(state, SOLO);
    game_screen(
        &view(state, SOLO),
        ScreenPhase::Playing {
            legal_actions: &legal,
        },
        &ScreenMode::Normal,
        catalog,
    )
}

fn with_reputation(state: GameState, change: i8) -> GameState {
    apply(
        state,
        &Event::ReputationChanged {
            player: SOLO,
            change,
        },
    )
}

fn hand() -> [CardId; 4] {
    [
        card(NobleManners, 1),
        card(Threaten, 1),
        card(Promise, 1),
        card(March, 1),
    ]
}

// C26
#[test]
fn panel_shows_reputation() {
    let catalog = french();
    let state = with_reputation(arena(VILLAGE, &hand(), &[]), 2);
    assert_eq!(
        screen_of(&state, &catalog).reputation,
        "Réputation : +2 · bonus d'interaction +1"
    );
    let state = with_reputation(state, -6);
    assert_eq!(
        screen_of(&state, &catalog).reputation,
        "Réputation : −4 · bonus d'interaction −2"
    );
    let state = with_reputation(state, -9);
    assert_eq!(
        screen_of(&state, &catalog).reputation,
        "Réputation : X · interaction impossible"
    );
}

// C27
#[test]
fn panel_offers_interaction_and_warns() {
    let catalog = french();
    let state = arena(VILLAGE, &hand(), &[]);
    let legal = legal_actions(&state, SOLO);
    let index = legal
        .iter()
        .position(|action| *action == Action::Interact)
        .expect("Interact légale");
    let screen = screen_of(&state, &catalog);
    let button = screen.interact.expect("bouton Interagir");
    assert_eq!(button.label, "Interagir avec les habitants");
    assert_eq!(button.command, Command::Play(index));
    assert_eq!(screen.interaction_warning, None);

    let state = done(&state, basic(card(March, 1)));
    let state = done(&state, basic(card(Promise, 1)));
    assert_eq!(
        screen_of(&state, &catalog).interaction_warning.as_deref(),
        Some("Interagir fait perdre : 2 mouvement")
    );
    // Variante maison (issue #95) : l'influence jouée avant l'interaction reste, sans avertissement.
    let only_influence = done(&arena(VILLAGE, &hand(), &[]), basic(card(Promise, 1)));
    assert_eq!(
        screen_of(&only_influence, &catalog).interaction_warning,
        None
    );
    let plain = arena(PLAIN_W, &hand(), &[]);
    assert_eq!(screen_of(&plain, &catalog).interact, None);
}

// C28
#[test]
fn panel_shows_interaction_in_progress() {
    let catalog = french();
    let state = with_reputation(arena(VILLAGE, &hand(), &[]), 2);
    let before = screen_of(&state, &catalog);
    assert_eq!(before.site_menu, None);
    assert!(before.purchases.is_empty());
    let state = done(&state, Action::Interact);
    let screen = screen_of(&state, &catalog);
    // Spec 075, C30 : le titre du menu du site (issue privée 259 : ce qui est dessiné).
    assert_eq!(
        screen.site_menu.as_ref().map(|menu| menu.title.as_str()),
        Some("Interaction : Village")
    );
    // Spec 041, C34 : « Rien à acheter ici » laisse la place au point de soin, grisé faute
    // d'influence.
    assert_eq!(
        screen
            .purchases
            .iter()
            .map(|button| (button.label.as_str(), button.command))
            .collect::<Vec<_>>(),
        [("Acheter un point de soin (3)", None)]
    );
    assert_eq!(screen.interact, None);
    assert_eq!(screen.points.fallback_text(), "Mouvement 0 · Influence 1");
}

// C29
#[test]
fn negative_influence_is_shown() {
    let catalog = french();
    let state = with_reputation(arena(VILLAGE, &hand(), &[]), -4);
    let state = done(&state, Action::Interact);
    assert_eq!(
        screen_of(&state, &catalog).points.fallback_text(),
        "Mouvement 0 · Influence −2"
    );
}

// C30
#[test]
fn reputation_texts() {
    let catalog = french();
    let state = arena(VILLAGE, &hand(), &[]);
    let screen = screen_of(&state, &catalog);
    let text = |card: CardId| {
        let held = screen
            .hand
            .iter()
            .find(|held| held.card == card)
            .expect("carte en main");
        (held.effect.clone(), held.strong_effect.clone())
    };
    assert_eq!(
        text(card(Threaten, 1)),
        (
            "Influence 2".to_string(),
            "Renforcé : Influence 5 ; Réputation −1".to_string()
        )
    );
    assert_eq!(
        text(card(NobleManners, 1)),
        (
            "Influence 2 ; en interaction : Gloire +1".to_string(),
            "Renforcé : Influence 4 ; en interaction : Gloire +1, Réputation +1".to_string()
        )
    );

    // La victoire sur les orques ajoute la réputation à l'historique du combat.
    let swiftness = card(Swiftness, 1);
    let state = arena(
        VILLAGE,
        &[swiftness, card(March, 1)],
        &[(prowlers(), ORCS_2)],
    );
    let state = done(&state, challenge(ORCS_2));
    let state = done(&state, Action::StartCombat);
    let state = done(&state, declare(&[foe(&state, ORCS_2)]));
    let state = done(&state, strong(swiftness, WHITE_DIE));
    let state = done(&state, Action::ResolveAttack);
    let legal = legal_actions(&state, SOLO);
    let screen = combat_screen(
        &view(&state, SOLO),
        &legal,
        &CombatChoice::default(),
        UndoStatus::Possible,
        &catalog,
    )
    .expect("écran de combat");
    let history = &screen.enemies[0].history;
    assert_eq!(history.last().map(String::as_str), Some("Réputation +1"));
}

// Raccourci I (docs/raccourcis.md).
#[test]
fn i_key_interacts() {
    let mut log = EventLog::default();
    for event in arena_events(VILLAGE, &hand(), &[], &[]) {
        log.push(0, event);
    }
    let (player, human) = human_player();
    let runner = GameRunner::resume(log, RngPositions::new(), vec![Box::new(player)])
        .expect("journal valide");
    let mut session = GameSession::with_runner(runner, human);
    session.run_frame();
    session.shortcut(Shortcut::Interact);
    session.run_frame();
    assert!(session.runner().state().interaction(SOLO).is_some());
}

// C14 côté écran : en interaction, Marche n'offre plus de bouton de mouvement, de base ni renforcé.
#[test]
fn no_move_buttons_in_interaction() {
    let catalog = french();
    let state = done(&arena(VILLAGE, &hand(), &[]), Action::Interact);
    let screen = screen_of(&state, &catalog);
    let march = screen
        .hand
        .iter()
        .find(|held| held.card == card(March, 1))
        .expect("Marche en main");
    assert!(march.plays.is_empty(), "{:?}", march.plays);
    assert!(march.strong_plays.is_empty(), "{:?}", march.strong_plays);
}
