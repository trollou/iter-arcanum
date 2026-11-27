//! Scénarios de la spec 075 (plateau isométrique) : boutons d'action posés sur les cases (issue
//! privée 169), menu du site, confirmations et essai d'animation du héros. Aucune règle nouvelle du
//! livret : les conditions des actions restent celles des specs 030, 031, 040 à 045 ; chaque test
//! cite le cas de la spec.

use arcanum_core::{
    Action, BasicAction, Direction, GameState, Hero, Hex, Map, MapShape, PlayerView, legal_actions,
    view,
};
use arcanum_i18n::{Catalog, Language};
use arcanum_ui::{
    BOARD_ACTION_DIAMETER, BREATHING_PERIOD, BoardActionKind, BoardCamera, BoardInput, BoardScreen,
    Command, EndTurnChoice, GameScreen, HeroWalk, Overlay, OverlayKind, ScreenMode, ScreenPhase,
    WALK_STEP_SECONDS, animation_repaint, board_screen, game_screen, hex_center,
    repository_assets_dir,
};
use eframe::egui::{Pos2, Rect, pos2, vec2};

// L'arène des tests d'arcanum-core, partagée plutôt que recopiée.
#[path = "../../arcanum-core/tests/arene/mod.rs"]
mod arene;
use arene::{
    MONASTERY, Monastery, ORCS_2, PLAIN_SW, PLAIN_W, SOLO, Units, VILLAGE, basic, card, challenge,
    done, option, prowlers, sideways,
};

mod support;
use support::{board_input, demo_board, map_input, play, session_of};

fn french() -> Catalog {
    Catalog::load(
        &repository_assets_dir().join("i18n"),
        &Language("fr".into()),
    )
    .expect("fr.json du dépôt")
}

fn area() -> Rect {
    Rect::from_min_size(pos2(0.0, 0.0), vec2(1000.0, 700.0))
}

fn camera_on(hex: Hex, zoom: f32) -> BoardCamera {
    BoardCamera {
        center: hex_center(hex),
        zoom,
    }
}

/// L'arène : le héros au village (2, −3), les Rôdeurs voisins en (3, −3).
fn village_with_prowlers() -> GameState {
    Units {
        placed: vec![(prowlers(), ORCS_2)],
        ..Units::default()
    }
    .state()
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

/// Le plateau comme l'application le dessine : les boutons d'action de l'écran de partie.
fn board_of(
    state: &GameState,
    camera: BoardCamera,
    cursor: Option<Pos2>,
    catalog: &Catalog,
) -> BoardScreen {
    let seen: PlayerView = view(state, SOLO);
    let legal = legal_actions(state, SOLO);
    let screen = screen_of(state, catalog);
    board_screen(
        &BoardInput {
            actions: &screen.board_actions,
            ..board_input(&seen, &legal, camera, area(), cursor)
        },
        catalog,
    )
}

#[track_caller]
fn index_of(state: &GameState, wanted: &Action) -> usize {
    legal_actions(state, SOLO)
        .iter()
        .position(|action| action == wanted)
        .unwrap_or_else(|| panic!("{wanted:?} doit être légale"))
}

/// Les boutons du plateau résumés : case, sorte, commande.
fn summary(screen: &BoardScreen) -> Vec<(Hex, BoardActionKind, Option<Command>)> {
    screen
        .actions
        .iter()
        .map(|action| (action.button.hex, action.button.kind, action.button.command))
        .collect()
}

/// Deux points de Mouvement en réserve (Marche, effet de base).
fn with_march(state: &GameState) -> GameState {
    done(state, option(card(BasicAction::March, 1), 0))
}

// Spec 075, C24.
#[test]
fn board_actions_sit_on_their_hex() {
    let catalog = french();
    let state = village_with_prowlers();
    let camera = camera_on(VILLAGE, 60.0);
    let screen = board_of(&state, camera, None, &catalog);
    assert_eq!(
        summary(&screen),
        [
            (
                VILLAGE,
                BoardActionKind::Interact,
                Some(Command::Play(index_of(&state, &Action::Interact)))
            ),
            (
                VILLAGE,
                BoardActionKind::Plunder,
                Some(Command::Play(index_of(&state, &Action::PlunderVillage)))
            ),
            (
                ORCS_2,
                BoardActionKind::Challenge,
                Some(Command::Play(index_of(&state, &challenge(ORCS_2))))
            ),
        ]
    );
    // Au-dessus de sa case, côte à côte pour une même case.
    for action in &screen.actions {
        let hex = camera.to_screen(area(), hex_center(action.button.hex));
        assert!(action.center.y < hex.y, "{action:?}");
        assert!((action.center.x - hex.x).abs() < 2.0 * BOARD_ACTION_DIAMETER);
    }
    assert_eq!(screen.actions[0].center.y, screen.actions[1].center.y);
    assert!(screen.actions[0].center.x < screen.actions[1].center.x);
    assert_eq!(
        screen
            .actions
            .iter()
            .map(|action| action.button.label.as_str())
            .collect::<Vec<_>>(),
        ["Interagir", "Piller", "Défier"]
    );
}

// Spec 075, C25.
#[test]
fn no_board_action_without_site_or_enemy() {
    let catalog = french();
    let state = Units {
        figure: PLAIN_SW,
        ..Units::default()
    }
    .state();
    assert!(screen_of(&state, &catalog).board_actions.is_empty());
    let screen = board_of(&state, camera_on(PLAIN_SW, 60.0), None, &catalog);
    assert!(screen.actions.is_empty());
}

// Spec 075, C26.
#[test]
fn board_action_tooltip_carries_the_warning() {
    let catalog = french();
    let state = with_march(&village_with_prowlers());
    let screen = screen_of(&state, &catalog);
    let interact = screen
        .board_actions
        .iter()
        .find(|button| button.kind == BoardActionKind::Interact)
        .expect("bouton Interagir");
    assert_eq!(
        interact.tooltip,
        [
            "Interagir avec les habitants",
            "Interagir fait perdre : 2 mouvement",
            "Raccourci : I",
        ]
    );
    let challenge = screen
        .board_actions
        .iter()
        .find(|button| button.kind == BoardActionKind::Challenge)
        .expect("bouton Défier");
    assert_eq!(challenge.tooltip, ["Défier Rôdeurs (orques maraudeurs)"]);
}

// Spec 075, C27.
#[test]
fn board_action_wins_over_the_hex_click() {
    let catalog = french();
    let state = with_march(&village_with_prowlers());
    let camera = camera_on(VILLAGE, 60.0);
    let actions = board_of(&state, camera, None, &catalog).actions;
    assert!(!actions.is_empty());
    for action in actions {
        let screen = board_of(&state, camera, Some(action.center), &catalog);
        assert_eq!(screen.click, action.button.command, "{action:?}");
        // L'infobulle est celle du bouton, pas celle de la case.
        assert_eq!(screen.hover_lines, action.button.tooltip);
        assert!(screen.route.is_empty());
    }
}

// Spec 075, C28.
#[test]
fn board_actions_keep_their_size_and_stay_visible() {
    let catalog = french();
    let state = village_with_prowlers();
    for zoom in [16.0, 60.0, 120.0] {
        let screen = board_of(&state, camera_on(VILLAGE, zoom), None, &catalog);
        assert_eq!(screen.actions.len(), 3);
        for action in &screen.actions {
            assert_eq!(action.radius * 2.0, BOARD_ACTION_DIAMETER);
        }
    }
    // Le village au coin haut gauche de la zone du plateau : les boutons restent dedans.
    let camera = camera_on(VILLAGE, 60.0);
    let corner = camera.to_board(area(), area().left_top());
    let camera = BoardCamera {
        center: camera.center + (hex_center(VILLAGE) - corner),
        ..camera
    };
    let screen = board_of(&state, camera, None, &catalog);
    assert_eq!(screen.actions.len(), 3);
    for action in &screen.actions {
        assert!(
            area().shrink(action.radius - 0.01).contains(action.center),
            "{action:?}"
        );
    }
}

// Spec 075, L3.
#[test]
fn neighbor_board_actions_do_not_overlap() {
    let catalog = french();
    let screen = board_of(
        &village_with_prowlers(),
        camera_on(VILLAGE, 16.0),
        None,
        &catalog,
    );
    let actions = &screen.actions;
    assert_eq!(actions.len(), 3);
    for (index, a) in actions.iter().enumerate() {
        for b in &actions[index + 1..] {
            assert!(
                a.center.distance(b.center) >= a.radius + b.radius - 0.01,
                "{a:?} et {b:?} se chevauchent"
            );
        }
    }
    // Chacun reste au plus près de sa case : le Défier à droite des boutons du village.
    let challenge = actions
        .iter()
        .find(|action| action.button.kind == BoardActionKind::Challenge)
        .expect("Défier");
    assert!(
        actions
            .iter()
            .filter(|action| action.button.hex == VILLAGE)
            .all(|action| action.center.x < challenge.center.x)
    );
}

// Spec 075, C29 — Q9 : un bouton sur chaque ennemi provoqué.
#[test]
fn start_combat_button_sits_on_the_provoked_enemy() {
    let catalog = french();
    let state = done(
        &with_march(&village_with_prowlers()),
        Action::MoveHero { to: PLAIN_W },
    );
    let screen = screen_of(&state, &catalog);
    let start = index_of(&state, &Action::StartCombat);
    let fights: Vec<_> = screen
        .board_actions
        .iter()
        .filter(|button| button.kind == BoardActionKind::StartCombat)
        .collect();
    assert_eq!(fights.len(), 1);
    assert_eq!(fights[0].hex, ORCS_2);
    assert_eq!(fights[0].command, Some(Command::Play(start)));
    assert_eq!(fights[0].label, "Combattre");
    assert_eq!(
        fights[0].tooltip,
        [
            "À combattre : Rôdeurs (orques maraudeurs)",
            "Engager le combat"
        ]
    );
}

// Spec 075, C33.
#[test]
fn no_board_action_while_ending_the_turn() {
    let catalog = french();
    let state = village_with_prowlers();
    let ending = ScreenMode::EndingTurn(EndTurnChoice::default());
    assert!(
        screen_in(&state, &ending, &catalog)
            .board_actions
            .is_empty()
    );
}

// Spec 075, C30 — Q5 : le menu du site, que le bouton bulle replie et déplie.
// Issue privée 307 — MKUE p. 7, 1 : replier le menu juste après avoir commencé l'interaction, sans rien y
// avoir fait, la reprend ; le héros peut de nouveau se déplacer, et le bouton d'interaction revient.
#[test]
fn folding_untouched_interaction_cancels_it() {
    let catalog = french();
    let mut session = session_of(Units::default().events());
    play(&mut session, &basic(card(BasicAction::March, 1)));
    play(&mut session, &Action::Interact);
    session.command(Command::ToggleSiteMenu);
    session.run_frame();
    let screen = session.screen(&catalog);
    assert_eq!(screen.site_menu, None);
    assert!(screen.interact.is_some(), "le bouton d'interaction revient");
    assert!(
        session
            .legal_actions()
            .iter()
            .any(|action| matches!(action, Action::MoveHero { .. })),
        "le héros peut de nouveau se déplacer"
    );
    // L'interaction suivante rouvre le menu déplié.
    play(&mut session, &Action::Interact);
    assert_eq!(
        session.screen(&catalog).site_menu.map(|menu| menu.open),
        Some(true)
    );
}

#[test]
fn site_menu_lists_interaction_options() {
    let catalog = french();
    let mut session = session_of(Units::default().events());
    assert_eq!(session.screen(&catalog).site_menu, None);
    play(&mut session, &Action::Interact);
    let screen = session.screen(&catalog);
    let menu = screen.site_menu.clone().expect("menu du site");
    assert_eq!(menu.hex, VILLAGE);
    assert_eq!(menu.title, "Interaction : Village");
    assert!(menu.open);
    // Les lignes d'aujourd'hui, dans le même ordre : recrutement, puis achats.
    assert_eq!(
        screen
            .purchases
            .iter()
            .map(|button| (button.label.as_str(), button.command))
            .collect::<Vec<_>>(),
        [("Acheter un point de soin (3)", None)]
    );
    // Le bouton bulle reste sur la case et replie le menu.
    let bubble: Vec<_> = screen
        .board_actions
        .iter()
        .filter(|button| button.kind == BoardActionKind::Interact)
        .collect();
    assert_eq!(bubble.len(), 1);
    assert_eq!(bubble[0].hex, VILLAGE);
    assert_eq!(bubble[0].command, Some(Command::ToggleSiteMenu));
    assert_eq!(bubble[0].tooltip, ["Interaction : Village"]);
    // Issue privée 307 : une carte jouée dans l'interaction, pour que replier le menu ne la reprenne pas.
    play(&mut session, &basic(card(BasicAction::Promise, 1)));
    session.command(Command::ToggleSiteMenu);
    assert_eq!(
        session.screen(&catalog).site_menu.map(|menu| menu.open),
        Some(false)
    );
    session.command(Command::ToggleSiteMenu);
    assert_eq!(
        session.screen(&catalog).site_menu.map(|menu| menu.open),
        Some(true)
    );
    // Replié, puis l'interaction annulée : il disparaît, et revient ouvert.
    session.command(Command::ToggleSiteMenu);
    session.command(Command::Undo);
    session.command(Command::Undo);
    session.run_frame();
    assert_eq!(session.screen(&catalog).site_menu, None);
    play(&mut session, &Action::Interact);
    assert_eq!(
        session.screen(&catalog).site_menu.map(|menu| menu.open),
        Some(true)
    );
    // Le plateau ouvre le menu à côté de la case du village.
    let screen = session.screen(&catalog);
    let seen = session.view();
    let legal = session.legal_actions();
    let camera = camera_on(VILLAGE, 60.0);
    let board = board_screen(
        &BoardInput {
            actions: &screen.board_actions,
            site_anchor: screen.site_menu.map(|menu| menu.hex),
            ..board_input(&seen, &legal, camera, area(), None)
        },
        &catalog,
    );
    let anchor = board.site_anchor.expect("menu attaché à la case");
    let village = camera.to_screen(area(), hex_center(VILLAGE));
    assert!(anchor.distance(village) < 2.0 * camera.zoom, "{anchor:?}");
    assert!(area().contains(anchor));
}

// Spec 075, C31.
#[test]
fn burn_confirmation_is_anchored_to_the_site() {
    let catalog = french();
    let mut session = session_of(Monastery::default().events());
    let screen = session.screen(&catalog);
    assert!(screen.burn.is_some());
    assert_eq!(screen.site_prompt_hex, None);
    session.command(Command::OpenBurn);
    let screen = session.screen(&catalog);
    assert_eq!(
        screen.burn_confirm.map(|prompt| prompt.buttons.len()),
        Some(2)
    );
    assert_eq!(screen.site_prompt_hex, Some(MONASTERY));
    let seen = session.view();
    let legal = session.legal_actions();
    let camera = camera_on(MONASTERY, 60.0);
    let board = board_screen(
        &BoardInput {
            site_anchor: screen.site_prompt_hex,
            ..board_input(&seen, &legal, camera, area(), None)
        },
        &catalog,
    );
    let anchor = board.site_anchor.expect("confirmation attachée à la case");
    let monastery = camera.to_screen(area(), hex_center(MONASTERY));
    assert!(anchor.distance(monastery) < 2.0 * camera.zoom, "{anchor:?}");
}

// --- Essai d'animation (spec 075, Q6) ---

/// Le plateau de démonstration, la figurine d'Arythea sur `figure`, à l'instant `time`.
fn demo_at(map: &Map, figure: Hex, time: f64, walk: Option<HeroWalk<'_>>) -> BoardScreen {
    let camera = camera_on(Hex::ORIGIN, 60.0);
    board_screen(
        &BoardInput {
            figure: Some(figure),
            hero: Some(Hero::Arythea),
            shape: MapShape::Open(arcanum_core::Columns::Five),
            time,
            walk,
            ..map_input(map, camera, area(), None)
        },
        &french(),
    )
}

fn hero_of(screen: &BoardScreen) -> Overlay {
    screen
        .overlays
        .iter()
        .find(|overlay| overlay.kind == OverlayKind::Figure)
        .cloned()
        .expect("figurine du héros")
}

fn others(screen: &BoardScreen) -> Vec<Overlay> {
    screen
        .overlays
        .iter()
        .filter(|overlay| overlay.kind != OverlayKind::Figure)
        .cloned()
        .collect()
}

// Spec 075, C34.
#[test]
fn hero_breathing_is_periodic_and_alone() {
    let demo = demo_board();
    let at = |time: f64| demo_at(&demo.map, demo.figure, time, None);
    let rest = hero_of(&at(0.0));
    let height = rest.rect.height();
    let quarter = BREATHING_PERIOD / 4.0;
    let high = hero_of(&at(quarter));
    let low = hero_of(&at(3.0 * quarter));
    assert!(
        (high.rect.height() / height - 1.02).abs() < 1e-3,
        "{high:?}"
    );
    assert!((low.rect.height() / height - 0.98).abs() < 1e-3, "{low:?}");
    // Les pieds restent posés sur la case.
    assert_eq!(high.foot, rest.foot);
    assert!((high.rect.bottom() - rest.rect.bottom()).abs() < 1e-3);
    let back = hero_of(&at(BREATHING_PERIOD));
    assert!((back.rect.height() - height).abs() < 1e-3);
    // Les autres éléments debout ne bougent pas.
    assert_eq!(others(&at(quarter)), others(&at(0.0)));
}

// Spec 075, C35.
#[test]
fn figure_slides_along_its_route() {
    let demo = demo_board();
    let a = Hex::ORIGIN;
    let b = a.neighbor(Direction::E);
    let c = b.neighbor(Direction::NE);
    let d = c.neighbor(Direction::E);
    let resting = |hex: Hex| hero_of(&demo_at(&demo.map, hex, 0.0, None)).foot;
    let started = 10.0;
    // Un pas : à mi-chemin à 75 ms, sur sa case à 150 ms.
    let step = [a, b];
    let walk = Some(HeroWalk {
        path: &step,
        started,
    });
    let foot = |time: f64, figure: Hex, walk| hero_of(&demo_at(&demo.map, figure, time, walk)).foot;
    let half = WALK_STEP_SECONDS / 2.0;
    let middle = |from: Hex, to: Hex| resting(from) + (resting(to) - resting(from)) / 2.0;
    assert!(foot(started + half, b, walk).distance(middle(a, b)) < 0.01);
    assert!(foot(started + WALK_STEP_SECONDS, b, walk).distance(resting(b)) < 0.01);
    // Trois cases : la figurine suit le trajet dans l'ordre.
    let route = [a, b, c, d];
    let walk = Some(HeroWalk {
        path: &route,
        started,
    });
    for (index, pair) in route.windows(2).enumerate() {
        let time = started + WALK_STEP_SECONDS * index as f64 + half;
        assert!(
            foot(time, d, walk).distance(middle(pair[0], pair[1])) < 0.01,
            "pas {index}"
        );
    }
    assert!(foot(started + 3.0 * WALK_STEP_SECONDS, d, walk).distance(resting(d)) < 0.01);
}

// Spec 075, L6 : après un retour en arrière, la figurine est aussitôt sur sa case.
#[test]
fn walk_is_dropped_on_undo() {
    let mut session = session_of(Units::default().events());
    assert_eq!(session.last_walk(), None);
    play(&mut session, &option(card(BasicAction::March, 1), 0));
    play(&mut session, &Action::MoveHero { to: PLAIN_W });
    let (first, path) = session.last_walk().expect("déplacement joué");
    assert_eq!(path, [VILLAGE, PLAIN_W]);
    // Deux points de plus, joués de côté, pour revenir au village.
    play(&mut session, &sideways(card(BasicAction::Promise, 1)));
    play(&mut session, &sideways(card(BasicAction::Threaten, 1)));
    play(&mut session, &Action::MoveHero { to: VILLAGE });
    let (second, path) = session.last_walk().expect("second déplacement");
    assert_ne!(first, second);
    assert_eq!(path, [PLAIN_W, VILLAGE]);
    session.command(Command::Undo);
    session.run_frame();
    assert_eq!(session.last_walk(), None);
}

// Spec 075, L7 : 30 images par seconde au plus, et rien sans le focus.
#[test]
fn animation_repaints_only_with_focus() {
    let delay = animation_repaint(true, true).expect("image suivante demandée");
    assert!(delay.as_secs_f64() >= 1.0 / 30.0 - 1e-6);
    assert_eq!(animation_repaint(true, false), None);
    assert_eq!(animation_repaint(false, true), None);
}
