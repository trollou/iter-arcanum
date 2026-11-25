//! Scénarios de la spec #5 (disparition des monstres vaincus) : la disparition relevée par la
//! session à la fin d'un combat. Aucune règle du livret ici : chaque test cite le cas de la spec ;
//! les combats suivent les règles des specs 031, 043 et 044.

use arcanum_core::{Action, BasicAction, EnemyToken, GameState, Hex, PlayerView, legal_actions};
use arcanum_ui::{
    BoardCamera, BoardDeparture, BoardInput, BoardScreen, Command, DEPARTURE_SECONDS,
    DepartingEnemy, GameSession, HeroWalk, Overlay, OverlayKind, animation_repaint, board_animated,
    board_screen, hex_center,
};
use eframe::egui::{Rect, pos2, vec2};

// L'arène des tests d'arcanum-core, partagée plutôt que recopiée.
#[path = "../../arcanum-core/tests/arene/mod.rs"]
mod arene;
use arene::{
    Adventure, DUNGEON, Fortified, KEEP, NEAR_KEEP, ORCS_2, PLAIN_W, SOLO, Units, basic, card,
    challenge, declare, diggers, minotaur, option, prowlers, sideways_attack, token,
};

mod support;
use support::{board_input, play, session_of};

use BasicAction::{Determination, March, Promise, Rage, Stamina, Threaten};

fn departing(hex: Hex, token: EnemyToken) -> DepartingEnemy {
    DepartingEnemy {
        hex,
        token,
        place: 0,
        of: 1,
    }
}

fn departure(session: &GameSession) -> Option<(u64, Vec<DepartingEnemy>)> {
    session
        .last_departure()
        .map(|(id, enemies)| (id, enemies.to_vec()))
}

/// Passe les phases jusqu'à l'attaque, sans parade ni choix de dégâts (spec 031).
#[track_caller]
fn to_attack(session: &mut GameSession) {
    play(session, &Action::NextCombatPhase);
    play(session, &Action::NextCombatPhase);
}

/// Attaque `targets` avec Rage (Attaque 2) et `sideways` joués de côté, puis la résout.
#[track_caller]
fn attack(
    session: &mut GameSession,
    targets: &[EnemyToken],
    rage: u16,
    sideways: &[(BasicAction, u16)],
) {
    play(session, &declare(targets));
    play(session, &option(card(Rage, rage), 0));
    for &(action, copy) in sideways {
        play(session, &sideways_attack(card(action, copy)));
    }
    play(session, &Action::ResolveAttack);
}

/// L'arène : le héros au village, les Rôdeurs voisins en (3, −3) ; défiés et vaincus avec Rage et
/// Marche, combat fini.
fn prowlers_beaten(decor: Units) -> GameSession {
    let mut session = session_of(decor.events());
    play(&mut session, &challenge(ORCS_2));
    play(&mut session, &Action::StartCombat);
    to_attack(&mut session);
    attack(&mut session, &[prowlers()], 1, &[(March, 1)]);
    play(&mut session, &Action::EndCombat);
    session
}

fn village() -> Units {
    Units {
        placed: vec![(prowlers(), ORCS_2)],
        ..Units::default()
    }
}

#[track_caller]
fn state_of(session: &GameSession) -> &GameState {
    session.runner().state()
}

// C1 — Les Rôdeurs vaincus disparaissent de (3, −3).
#[test]
fn defeated_rampaging_enemy_departs() {
    let session = prowlers_beaten(village());
    let (_, enemies) = departure(&session).expect("une disparition");
    assert_eq!(enemies, [departing(ORCS_2, prowlers())]);
}

// C2 — Deux ennemis défiés, seuls les Rôdeurs vaincus : les Creuseurs restent.
#[test]
fn only_defeated_enemies_depart() {
    let next_to_village = PLAIN_W;
    let decor = Units {
        placed: vec![(prowlers(), ORCS_2), (diggers(), next_to_village)],
        ..Units::default()
    };
    let mut session = session_of(decor.events());
    play(&mut session, &challenge(ORCS_2));
    play(&mut session, &challenge(next_to_village));
    play(&mut session, &Action::StartCombat);
    to_attack(&mut session);
    attack(&mut session, &[prowlers()], 1, &[(March, 1)]);
    play(&mut session, &Action::EndCombat);
    let (_, enemies) = departure(&session).expect("une disparition");
    assert_eq!(enemies, [departing(ORCS_2, prowlers())]);
    assert_eq!(
        session.view().enemy_tokens.get(&next_to_village),
        Some(&vec![diggers()])
    );
}

// C3 — Assaut de la forteresse, défenseur face cachée vaincu : il disparaît sous sa figure
// retournée, le drapeau est déjà posé.
#[test]
fn defeated_defender_departs() {
    let defender = prowlers();
    let decor = Fortified {
        figure: NEAR_KEEP,
        keep: Some((defender, false)),
        tower: None,
        hand: vec![
            card(March, 1),
            card(Stamina, 1),
            card(Rage, 1),
            card(Promise, 1),
            card(Threaten, 1),
        ],
        ..Fortified::default()
    };
    let mut session = session_of(decor.events());
    play(&mut session, &basic(card(March, 1)));
    play(&mut session, &basic(card(Stamina, 1)));
    play(&mut session, &Action::MoveHero { to: KEEP });
    play(&mut session, &Action::StartCombat);
    to_attack(&mut session);
    attack(&mut session, &[defender], 1, &[(Promise, 1)]);
    play(&mut session, &Action::EndCombat);
    let (_, enemies) = departure(&session).expect("une disparition");
    assert_eq!(enemies, [departing(KEEP, defender)]);
    assert_eq!(state_of(&session).shields().get(&KEEP), Some(&SOLO));
}

// C4 — Le Minotaure tiré en entrant dans le donjon n'a jamais été sur le plateau.
#[test]
fn enemy_drawn_with_the_combat_does_not_depart() {
    let decor = Adventure {
        hand: vec![
            card(Rage, 1),
            card(Rage, 2),
            card(Determination, 1),
            card(March, 1),
            card(Promise, 1),
        ],
        // Armure et main assez hautes pour ne pas être assommé par le Minotaure (spec 034).
        level: 7,
        ..Adventure::default()
    };
    let mut session = session_of(decor.events());
    play(&mut session, &Action::EnterAdventureSite);
    to_attack(&mut session);
    play(&mut session, &declare(&[minotaur()]));
    for action in [
        option(card(Rage, 1), 0),
        option(card(Rage, 2), 0),
        sideways_attack(card(March, 1)),
    ] {
        play(&mut session, &action);
    }
    play(&mut session, &Action::ResolveAttack);
    play(&mut session, &Action::EndCombat);
    assert_eq!(
        state_of(&session).shields().get(&DUNGEON),
        Some(&SOLO),
        "donjon conquis"
    );
    assert_eq!(departure(&session), None);
}

/// Finit le tour en gardant les cartes en main.
#[track_caller]
fn end_turn(session: &mut GameSession) {
    let action = legal_actions(state_of(session), SOLO)
        .into_iter()
        .find(|action| matches!(action, Action::EndTurn { .. }))
        .expect("fin du tour proposée");
    play(session, &action);
}

/// L'arène avec des Rôdeurs en (3, −3) et d'autres en (2, −2), et deux Rages et deux Marches.
fn two_fights() -> Units {
    Units {
        placed: vec![
            (prowlers(), ORCS_2),
            (token(arcanum_core::Enemy::Prowlers, 2), PLAIN_W),
        ],
        hand: vec![
            card(Rage, 1),
            card(March, 1),
            card(Rage, 2),
            card(March, 2),
            card(Promise, 1),
        ],
        ..Units::default()
    }
}

// C5 — Combat fini sans vaincu : aucune disparition, celle du combat précédent n'est pas rejouée.
#[test]
fn no_departure_without_defeat() {
    let mut session = prowlers_beaten(two_fights());
    assert!(departure(&session).is_some());
    end_turn(&mut session);
    play(&mut session, &challenge(PLAIN_W));
    play(&mut session, &Action::StartCombat);
    to_attack(&mut session);
    play(&mut session, &Action::EndCombat);
    assert_eq!(departure(&session), None);
}

// C6 — Deux combats de suite : le numéro de la disparition change.
#[test]
fn each_combat_has_its_own_departure() {
    let mut session = prowlers_beaten(two_fights());
    let (first, _) = departure(&session).expect("première disparition");
    end_turn(&mut session);
    play(&mut session, &challenge(PLAIN_W));
    play(&mut session, &Action::StartCombat);
    to_attack(&mut session);
    let second_prowlers = token(arcanum_core::Enemy::Prowlers, 2);
    attack(&mut session, &[second_prowlers], 2, &[(March, 2)]);
    play(&mut session, &Action::EndCombat);
    let (second, enemies) = departure(&session).expect("seconde disparition");
    assert_ne!(first, second);
    assert_eq!(enemies, [departing(PLAIN_W, second_prowlers)]);
}

// C7 — Après un retour en arrière, aucune disparition (comme le glissement, spec 075, L6).
#[test]
fn departure_is_dropped_on_undo() {
    let mut session = prowlers_beaten(village());
    assert!(departure(&session).is_some());
    session.command(Command::Undo);
    session.run_frame();
    assert_eq!(departure(&session), None);
}

// --- Effacement sur le plateau (C8 à C11, C14) ---

fn camera() -> BoardCamera {
    BoardCamera {
        center: hex_center(ORCS_2),
        zoom: 60.0,
    }
}

fn area() -> Rect {
    Rect::from_min_size(pos2(0.0, 0.0), vec2(1000.0, 700.0))
}

/// Le village et les Rôdeurs voisins, tels que la vue les donne.
fn village_view() -> PlayerView {
    session_of(village().events()).view()
}

/// La même vue sans les Rôdeurs : le plateau après leur défaite.
fn without_prowlers(view: &PlayerView) -> PlayerView {
    let mut after = view.clone();
    after.enemy_tokens.remove(&ORCS_2);
    after
}

const STARTED: f64 = 10.0;

fn prowlers_departing() -> [DepartingEnemy; 1] {
    [departing(ORCS_2, prowlers())]
}

/// Le plateau de `view` à l'instant `time`, avec l'effacement des Rôdeurs commencé à `STARTED`.
fn board_at(
    view: &PlayerView,
    time: f64,
    enemies: &[DepartingEnemy],
    animations: bool,
) -> BoardScreen {
    board_screen(
        &BoardInput {
            time,
            departure: Some(BoardDeparture {
                enemies,
                started: STARTED,
            }),
            animations,
            ..board_input(view, &[], camera(), area(), None)
        },
        &french(),
    )
}

fn of_kind(screen: &BoardScreen, kind: OverlayKind) -> Vec<Overlay> {
    screen
        .overlays
        .iter()
        .filter(|overlay| overlay.kind == kind)
        .cloned()
        .collect()
}

fn french() -> arcanum_i18n::Catalog {
    arcanum_i18n::Catalog::load(
        &arcanum_ui::repository_assets_dir().join("i18n"),
        &arcanum_i18n::Language("fr".into()),
    )
    .expect("fr.json du dépôt")
}

/// La figurine des Rôdeurs avant le combat.
fn prowlers_figure(view: &PlayerView) -> Overlay {
    let before = board_screen(
        &BoardInput {
            time: STARTED,
            ..board_input(view, &[], camera(), area(), None)
        },
        &french(),
    );
    let [figure] = of_kind(&before, OverlayKind::Rampaging)
        .try_into()
        .expect("les Rôdeurs sur le plateau");
    figure
}

// C8 — Au début, à la place et à la taille de sa figurine d'avant le combat, opaque.
#[test]
fn departing_figure_starts_where_it_stood() {
    let view = village_view();
    let stood = prowlers_figure(&view);
    let after = without_prowlers(&view);
    let enemies = prowlers_departing();
    let screen = board_at(&after, STARTED, &enemies, true);
    let [departing] = of_kind(&screen, OverlayKind::Departing)
        .try_into()
        .expect("les Rôdeurs s'effacent");
    assert_eq!(departing.rect, stood.rect);
    assert_eq!(departing.foot, stood.foot);
    assert_eq!(departing.image, stood.image);
    assert_eq!(departing.tint.a(), 255);
}

// C9 — Le mouvement ralentit à la fin (Q1) : au milieu de la durée, la hauteur et l'opacité sont au
// quart, les pieds posés au même endroit ; à la fin, plus rien.
#[test]
fn departing_figure_sinks_and_fades() {
    let view = village_view();
    let stood = prowlers_figure(&view);
    let after = without_prowlers(&view);
    let enemies = prowlers_departing();
    let middle = board_at(&after, STARTED + DEPARTURE_SECONDS / 2.0, &enemies, true);
    let [departing] = of_kind(&middle, OverlayKind::Departing)
        .try_into()
        .expect("en cours");
    assert_eq!(departing.foot, stood.foot);
    assert_eq!(departing.rect.bottom(), stood.rect.bottom());
    assert_eq!(departing.rect.width(), stood.rect.width());
    assert!((departing.rect.height() - stood.rect.height() / 4.0).abs() < 0.01);
    assert!((f32::from(departing.tint.a()) / 255.0 - 0.25).abs() < 0.01);

    let done = board_at(&after, STARTED + DEPARTURE_SECONDS, &enemies, true);
    assert!(of_kind(&done, OverlayKind::Departing).is_empty());
}

// C10 — Les autres éléments debout ne changent pas pendant l'effacement.
#[test]
fn departure_leaves_other_overlays_alone() {
    let view = without_prowlers(&village_view());
    let time = STARTED + DEPARTURE_SECONDS / 3.0;
    let with = board_at(&view, time, &prowlers_departing(), true);
    let without = board_at(&view, time, &[], true);
    let others: Vec<Overlay> = with
        .overlays
        .iter()
        .filter(|overlay| overlay.kind != OverlayKind::Departing)
        .cloned()
        .collect();
    assert_eq!(others, without.overlays);
}

// C11 — Une image suivante tant que l'effacement dure, plus après (ici sans figurine du héros).
#[test]
fn departure_requests_repaints_until_done() {
    let view = without_prowlers(&village_view());
    let enemies = prowlers_departing();
    let input = |time| BoardInput {
        figure: None,
        time,
        departure: Some(BoardDeparture {
            enemies: &enemies,
            started: STARTED,
        }),
        ..board_input(&view, &[], camera(), area(), None)
    };
    assert!(board_animated(&input(STARTED + 0.1)));
    assert!(animation_repaint(board_animated(&input(STARTED + 0.1)), true).is_some());
    assert!(!board_animated(&input(STARTED + DEPARTURE_SECONDS)));
    assert_eq!(
        animation_repaint(board_animated(&input(STARTED + DEPARTURE_SECONDS)), true),
        None
    );
}

// C14 — Animations décochées : hauteur du héros constante, déplacement aussitôt fini, rien ne
// s'efface, aucune image suivante.
#[test]
fn no_board_animation_when_disabled() {
    let view = without_prowlers(&village_view());
    let enemies = prowlers_departing();
    let figure = view.figure.expect("figurine");
    let path = [PLAIN_W, figure];
    let board = |time: f64, animations| {
        board_screen(
            &BoardInput {
                time,
                walk: Some(HeroWalk {
                    path: &path,
                    started: STARTED,
                }),
                departure: Some(BoardDeparture {
                    enemies: &enemies,
                    started: STARTED,
                }),
                animations,
                ..board_input(&view, &[], camera(), area(), None)
            },
            &french(),
        )
    };
    let hero = |screen: &BoardScreen| -> Overlay {
        let [hero] = of_kind(screen, OverlayKind::Figure)
            .try_into()
            .expect("la figurine");
        hero
    };
    // Au repos, longtemps après : la place de la figurine sur sa case.
    let resting = hero(&board(STARTED + 100.0, false));
    for time in [STARTED, STARTED + 0.05, STARTED + 0.4, STARTED + 0.75] {
        let screen = board(time, false);
        assert_eq!(hero(&screen).rect, resting.rect, "à {time}");
        assert!(of_kind(&screen, OverlayKind::Departing).is_empty());
        let input = BoardInput {
            time,
            departure: Some(BoardDeparture {
                enemies: &enemies,
                started: STARTED,
            }),
            animations: false,
            ..board_input(&view, &[], camera(), area(), None)
        };
        assert!(!board_animated(&input));
    }
    // Avec les animations, la même figurine bouge.
    assert_ne!(hero(&board(STARTED + 0.05, true)).rect, resting.rect);
}
