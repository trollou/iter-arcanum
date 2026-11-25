//! Scénarios de la spec #5 (disparition des monstres vaincus) : la disparition relevée par la
//! session à la fin d'un combat. Aucune règle du livret ici : chaque test cite le cas de la spec ;
//! les combats suivent les règles des specs 031, 043 et 044.

use arcanum_core::{Action, BasicAction, EnemyToken, GameState, Hex, legal_actions};
use arcanum_ui::{Command, DepartingEnemy, GameSession};

// L'arène des tests d'arcanum-core, partagée plutôt que recopiée.
#[path = "../../arcanum-core/tests/arene/mod.rs"]
mod arene;
use arene::{
    Adventure, DUNGEON, Fortified, KEEP, NEAR_KEEP, ORCS_2, PLAIN_W, SOLO, Units, basic, card,
    challenge, declare, diggers, minotaur, option, prowlers, sideways_attack, token,
};

mod support;
use support::{play, session_of};

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
