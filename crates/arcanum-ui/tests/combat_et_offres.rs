//! Scénarios de la spec 070 côté affichage (combat et offres, refonte de l'interface).

use arcanum_core::{
    Action, BasicAction, BasicColor, CardId, Enemy, EnemyColor, EnemyToken, Event, GameState, Hex,
    ManaPayment, UndoStatus, Unit, UnitCard, apply, legal_actions, view,
};
use arcanum_i18n::{Catalog, Language};
use arcanum_ui::{
    CombatChoice, CombatScreen, Command, GameSession, Shortcut, combat_screen,
    repository_assets_dir,
};

// L'arène des tests de combat d'arcanum-core, partagée plutôt que recopiée (bilan de la spec 032).
#[path = "../../arcanum-core/tests/arene/mod.rs"]
mod arene;
use arene::*;

mod support;
use support::{play, session_of};

use BasicAction::*;

fn french() -> Catalog {
    Catalog::load(
        &repository_assets_dir().join("i18n"),
        &Language("fr".into()),
    )
    .expect("fr.json du dépôt")
}

fn fight_hand() -> Vec<CardId> {
    vec![
        card(March, 1),
        card(Rage, 1),
        card(Promise, 1),
        card(Swiftness, 1),
    ]
}

/// Bac à sable par événements, figurine au village, jetons `placed` ; la main `fight_hand`.
fn arena_with(placed: &[(EnemyToken, Hex)]) -> GameState {
    arena_events(VILLAGE, &fight_hand(), placed, &[])
        .iter()
        .fold(GameState::default(), apply)
}

/// Les Rôdeurs seuls, défiés depuis le village, combat engagé.
fn alone() -> GameState {
    let state = arena_with(&[(prowlers(), ORCS_2)]);
    let state = done(&state, Action::Challenge { enemy: ORCS_2 });
    done(&state, Action::StartCombat)
}

/// Rôdeurs et Creuseurs défiés, combat engagé.
fn both() -> GameState {
    let state = arena_with(&[(prowlers(), ORCS_2), (diggers(), PLAIN_SW)]);
    let state = done(&state, Action::Challenge { enemy: ORCS_2 });
    let state = done(&state, Action::Challenge { enemy: PLAIN_SW });
    done(&state, Action::StartCombat)
}

fn shown(state: &GameState, catalog: &Catalog) -> CombatScreen {
    shown_with(state, UndoStatus::Possible, catalog)
}

/// L'écran, le retour en arrière dans l'état `undo`.
fn shown_with(state: &GameState, undo: UndoStatus, catalog: &Catalog) -> CombatScreen {
    let legal = legal_actions(state, SOLO);
    combat_screen(
        &view(state, SOLO),
        &legal,
        &CombatChoice::default(),
        undo,
        catalog,
    )
    .expect("écran de combat")
}

/// L'action que joue la commande, qui doit être `Command::Play`.
fn played(state: &GameState, command: Option<Command>) -> Action {
    let Some(Command::Play(index)) = command else {
        panic!("le bouton joue une action légale : {command:?}");
    };
    legal_actions(state, SOLO)[index].clone()
}

// --- Ennemi coché d'office ---

// C1 — MKUE p. 9, *Ranged and Siege Attack Phase* 2 : un seul ennemi, la seule cible possible.
#[test]
fn single_target_is_selected_without_checkbox() {
    let catalog = french();
    let state = to_attack_phase(&alone());
    let screen = shown(&state, &catalog);
    assert_eq!(screen.enemies[0].target, None);
    let attack = screen.attack.as_ref().expect("Attaquer");
    assert_eq!(
        played(&state, attack.command),
        Action::DeclareAttack {
            targets: vec![prowlers()]
        }
    );
}

// C2 — Les Rôdeurs vaincus à distance (Rapidité renforcée : attaque à distance 3), les Creuseurs
// restent seuls en phase d'attaque.
#[test]
fn last_target_left_is_selected() {
    let catalog = french();
    let state = both();
    let state = done(&state, declare(&[prowlers()]));
    let state = done(&state, strong(card(Swiftness, 1), WHITE_DIE));
    let state = done(&state, Action::ResolveAttack);
    let state = to_attack_phase(&state);
    let screen = shown(&state, &catalog);
    assert!(screen.enemies[0].defeated);
    assert_eq!(screen.enemies[1].target, None);
    let attack = screen.attack.as_ref().expect("Attaquer");
    assert_eq!(
        played(&state, attack.command),
        Action::DeclareAttack {
            targets: vec![diggers()]
        }
    );
}

// C3 — Deux cibles possibles : chacune sa case, décochée, « Attaquer » grisé.
#[test]
fn two_targets_keep_their_checkboxes() {
    let catalog = french();
    let state = to_attack_phase(&both());
    let screen = shown(&state, &catalog);
    for enemy in &screen.enemies {
        let toggle = enemy.target.as_ref().expect("case cible");
        assert!(!toggle.checked);
    }
    assert_eq!(screen.attack.as_ref().expect("Attaquer").command, None);
}

// L1 — Spec 031, Q10 : l'ennemi seul, attaqué et raté dans la phase, n'est plus une cible.
#[test]
fn failed_single_target_has_no_attack() {
    let catalog = french();
    let state = to_attack_phase(&alone());
    let state = done(&state, declare(&[prowlers()]));
    let state = done(&state, Action::ResolveAttack);
    let screen = shown(&state, &catalog);
    assert_eq!(screen.enemies[0].target, None);
    assert_eq!(screen.attack, None);
}

// --- Bouton « Parer » et carte d'ennemi ---

/// Combat engagé contre `placed`, défiés dans l'ordre, la pile marron réordonnée : `brown` dessus.
fn fighting_with_brown(placed: &[(EnemyToken, Hex)], brown: &[EnemyToken]) -> GameState {
    let mut state = arena_with(placed);
    let mut order = brown.to_vec();
    order.extend(
        state
            .enemy_pile(EnemyColor::Brown)
            .iter()
            .copied()
            .filter(|token| !brown.contains(token)),
    );
    state = apply(
        state,
        &Event::EnemyPileShuffled {
            color: EnemyColor::Brown,
            order,
        },
    );
    for &(_, hex) in placed {
        state = done(&state, Action::Challenge { enemy: hex });
    }
    done(&state, Action::StartCombat)
}

// C5
#[test]
fn block_button_is_named_parer() {
    let catalog = french();
    let state = done(&alone(), Action::NextCombatPhase);
    let screen = shown(&state, &catalog);
    let block = screen.enemies[0].block.as_ref().expect("Parer");
    assert_eq!(block.label, "Parer");
    assert_eq!(
        played(&state, Some(block.command)),
        Action::DeclareBlock { target: prowlers() }
    );
}

// C6 — Spec 033 : l'Invocateur paré par sa créature, le bouton se lit aussi « Parer ».
#[test]
fn summoned_block_button_is_named_parer() {
    let catalog = french();
    let minotaur = EnemyToken::new(Enemy::Minotaur, 1);
    let state = fighting_with_brown(&[(orc_summoners(), ORCS_2)], &[minotaur]);
    let state = done(&state, Action::NextCombatPhase);
    let screen = shown(&state, &catalog);
    assert_eq!(
        screen.enemies[0].summoned.as_ref().map(|card| card.token),
        Some(minotaur)
    );
    let block = screen.enemies[0].block.as_ref().expect("Parer");
    assert_eq!(block.label, "Parer");
}

// C13 — Le nom seul ; la famille dans l'infobulle du jeton.
#[test]
fn enemy_card_name_without_family() {
    let catalog = french();
    let screen = shown(&alone(), &catalog);
    assert_eq!(screen.enemies[0].name, "Rôdeurs");
    assert_eq!(screen.enemies[0].family, "orques maraudeurs");
}

// C14 — Plus d'état « en jeu » : seulement « paré », « vaincu » ou une restriction.
#[test]
fn enemy_card_status_omits_in_play() {
    let catalog = french();
    let screen = shown(&both(), &catalog);
    assert_eq!(screen.enemies[0].status, None);
    // MKUE p. 9, *Ranged and Siege Attack Phase* 3 : fortifiés, seul le siège les atteint.
    assert_eq!(
        screen.enemies[1].status.as_deref(),
        Some("fortifié : siège seulement")
    );
    let state = done(&both(), declare(&[prowlers()]));
    let state = done(&state, strong(card(Swiftness, 1), WHITE_DIE));
    let state = done(&state, Action::ResolveAttack);
    assert_eq!(
        shown(&state, &catalog).enemies[0].status.as_deref(),
        Some("vaincu")
    );
}

// C15 — « Dans ce combat (2) », absent tant qu'il n'y a pas de ligne.
#[test]
fn enemy_history_title_counts_lines() {
    let catalog = french();
    let state = done(&alone(), Action::NextCombatPhase);
    assert_eq!(shown(&state, &catalog).enemies[0].history_title, None);
    // MKUE p. 9, *Block Phase* 6 : parade ratée, puis les dégâts des Rôdeurs.
    let state = done(&state, Action::DeclareBlock { target: prowlers() });
    let state = done(&state, Action::ResolveBlock);
    let state = done(&state, Action::NextCombatPhase);
    let screen = shown(&state, &catalog);
    assert_eq!(screen.enemies[0].history.len(), 2);
    assert_eq!(
        screen.enemies[0].history_title.as_deref(),
        Some("Dans ce combat (2)")
    );
}

// C16 — Spec 033, Q4 : l'enjeu de la parade sur deux lignes ; Chevaucheurs de loups, attaque 3,
// vifs.
#[test]
fn block_stakes_on_two_lines() {
    let catalog = french();
    let state = arena_with(&[(wolf_riders(), ORCS_2)]);
    let state = done(&state, Action::Challenge { enemy: ORCS_2 });
    let state = done(&state, Action::StartCombat);
    let state = done(&state, Action::NextCombatPhase);
    assert_eq!(
        shown(&state, &catalog).enemies[0].stakes,
        [
            "À parer : 6 (vif, 2 × 3)",
            "Si non paré : 3 dégâts, 2 blessure(s)"
        ]
    );
}

// --- Parade insuffisante ---

/// Les Rôdeurs seuls (attaque 4), en phase de parade, la parade déclarée.
fn blocking() -> GameState {
    let state = done(&alone(), Action::NextCombatPhase);
    done(&state, Action::DeclareBlock { target: prowlers() })
}

/// Le bouton du cadre de l'attaque ou de la parade en cours.
fn in_progress_button(screen: &CombatScreen) -> arcanum_ui::Button {
    screen
        .in_progress
        .as_ref()
        .expect("attaque ou parade en cours")
        .resolve
        .clone()
        .expect("bouton du cadre")
}

// C7 — MKUE p. 9, *Block Phase* 6 : sans carte, puis avec moins de points qu'il n'en faut.
#[test]
fn insufficient_block_offers_cancel() {
    let catalog = french();
    let state = blocking();
    let button = in_progress_button(&shown(&state, &catalog));
    assert_eq!(button.label, "Annuler");
    assert_eq!(button.command, Command::CancelBlock);
    // Rage : Parade 2, contre l'attaque 4 des Rôdeurs.
    let state = done(&state, option(card(Rage, 1), 1));
    let screen = shown(&state, &catalog);
    assert_eq!(
        screen
            .in_progress
            .as_ref()
            .map(|panel| panel.total.value.as_str()),
        Some("2 / 4")
    );
    let button = in_progress_button(&screen);
    assert_eq!(button.label, "Annuler");
    assert_eq!(button.command, Command::CancelBlock);
}

// C8 — MKUE p. 9, *Block Phase* 5 : Rage (2), Marche et Promesse de côté (1 + 1) : 4 / 4.
#[test]
fn sufficient_block_offers_resolve() {
    let catalog = french();
    let state = done(&blocking(), option(card(Rage, 1), 1));
    let state = done(&state, sideways_block(card(March, 1)));
    let state = done(&state, sideways_block(card(Promise, 1)));
    let button = in_progress_button(&shown(&state, &catalog));
    assert_eq!(button.label, "Valider la parade");
    assert_eq!(played(&state, Some(button.command)), Action::ResolveBlock);
}

// C9 — Spec 033 : Chevaucheurs de loups (attaque 3, vifs) ; 3 points de parade ne suffisent pas.
#[test]
fn swift_block_counts_doubled_value() {
    let catalog = french();
    let state = arena_with(&[(wolf_riders(), ORCS_2)]);
    let state = done(&state, Action::Challenge { enemy: ORCS_2 });
    let state = done(&state, Action::StartCombat);
    let state = done(&state, Action::NextCombatPhase);
    let state = done(
        &state,
        Action::DeclareBlock {
            target: wolf_riders(),
        },
    );
    let state = done(&state, option(card(Rage, 1), 1));
    let state = done(&state, sideways_block(card(March, 1)));
    let button = in_progress_button(&shown(&state, &catalog));
    assert_eq!(button.label, "Annuler");
}

// C10 — MKUE p. 9, *Block Phase* 5.b ; spec 032 : contre la glace du Dragon de glace (attaque 6),
// Polyvalence renforcée (Parade de feu 3) et Détermination renforcée (Parade 5, divisée) font
// 3 + 2 = 5, pas 8 ; Marche de côté (divisée avec elle) amène 3 + (5 + 1) ÷ 2 = 6.
#[test]
fn inefficient_block_counts_halved_total() {
    let catalog = french();
    let hand = [
        card(BattleVersatility, 1),
        card(Determination, 1),
        card(March, 1),
    ];
    let mut events = arena_events(VILLAGE, &hand, &[(ice_dragon(), ORCS_2)], &[]);
    events.extend(
        [BasicColor::Red, BasicColor::Blue].map(|color| Event::CrystalGained {
            player: SOLO,
            color,
        }),
    );
    let state = events.iter().fold(GameState::default(), apply);
    let state = done(&state, Action::Challenge { enemy: ORCS_2 });
    let state = done(&state, Action::StartCombat);
    let state = done(&state, Action::NextCombatPhase);
    let state = done(
        &state,
        Action::DeclareBlock {
            target: ice_dragon(),
        },
    );
    let state = done(&state, strong_crystal(card(BattleVersatility, 1), 3));
    let state = done(&state, strong_crystal(card(Determination, 1), 0));
    let screen = shown(&state, &catalog);
    assert_eq!(
        screen
            .in_progress
            .as_ref()
            .map(|panel| panel.total.value.as_str()),
        Some("5 / 6")
    );
    assert_eq!(in_progress_button(&screen).label, "Annuler");
    let state = done(&state, sideways_block(card(March, 1)));
    let button = in_progress_button(&shown(&state, &catalog));
    assert_eq!(button.label, "Valider la parade");
}

// L3 — Retour en arrière impossible (partie chargée en pleine parade) : « Valider la parade »
// reste proposé, même insuffisant, pour ne pas bloquer le joueur.
#[test]
fn block_without_undo_keeps_resolve() {
    let catalog = french();
    let state = blocking();
    let button = in_progress_button(&shown_with(
        &state,
        UndoStatus::NothingSinceTurnStart,
        &catalog,
    ));
    assert_eq!(button.label, "Valider la parade");
    assert_eq!(played(&state, Some(button.command)), Action::ResolveBlock);
}

// --- Attaque insuffisante ---

// C19 — MKUE p. 9, *Ranged and Siege Attack Phase* 5 et 6 : Marche de côté (1) contre l'armure 3
// des Rôdeurs, puis Rage (2) en plus : 3 / 3.
#[test]
fn insufficient_attack_offers_cancel() {
    let catalog = french();
    let state = to_attack_phase(&alone());
    let state = done(&state, declare(&[prowlers()]));
    let button = in_progress_button(&shown(&state, &catalog));
    assert_eq!(button.label, "Annuler");
    assert_eq!(button.command, Command::CancelAttack);
    let state = done(&state, sideways_attack(card(March, 1)));
    let button = in_progress_button(&shown(&state, &catalog));
    assert_eq!(button.label, "Annuler");
    assert_eq!(button.command, Command::CancelAttack);
}

// C19
#[test]
fn sufficient_attack_offers_resolve() {
    let catalog = french();
    let state = to_attack_phase(&alone());
    let state = done(&state, declare(&[prowlers()]));
    let state = done(&state, sideways_attack(card(March, 1)));
    let state = done(&state, option(card(Rage, 1), 0));
    let button = in_progress_button(&shown(&state, &catalog));
    assert_eq!(button.label, "Valider l'attaque");
    assert_eq!(played(&state, Some(button.command)), Action::ResolveAttack);
}

// C21 — Comme L3, pour l'attaque.
#[test]
fn attack_without_undo_keeps_resolve() {
    let catalog = french();
    let state = to_attack_phase(&alone());
    let state = done(&state, declare(&[prowlers()]));
    let button = in_progress_button(&shown_with(
        &state,
        UndoStatus::NothingSinceTurnStart,
        &catalog,
    ));
    assert_eq!(button.label, "Valider l'attaque");
    assert_eq!(played(&state, Some(button.command)), Action::ResolveAttack);
}

// --- Dans la session : Annuler, Entrée et Échap ---

fn combat(session: &GameSession, catalog: &Catalog) -> CombatScreen {
    session.combat_screen(catalog).expect("écran de combat")
}

fn peasants() -> UnitCard {
    UnitCard::new(Unit::Peasants, 1)
}

/// Les Chevaucheurs de loups seuls (attaque 3, vifs : 6 à parer), les Paysans recrutés, en phase
/// de parade.
fn wolf_session() -> GameSession {
    let events = Units {
        owned: vec![peasants()],
        placed: vec![(wolf_riders(), ORCS_2)],
        ..Units::default()
    }
    .events();
    let mut session = session_of(events);
    play(&mut session, &Action::Challenge { enemy: ORCS_2 });
    play(&mut session, &Action::StartCombat);
    play(&mut session, &Action::NextCombatPhase);
    session
}

/// Parade déclarée contre les Chevaucheurs : Marche et Promesse de côté, les Paysans (Parade 2),
/// 4 / 6.
fn insufficient_block(session: &mut GameSession) {
    play(
        session,
        &Action::DeclareBlock {
            target: wolf_riders(),
        },
    );
    play(session, &sideways_block(card(March, 1)));
    play(session, &sideways_block(card(Promise, 1)));
    play(
        session,
        &Action::ActivateUnit {
            enemy: None,
            unit: peasants(),
            ability: 0,
            option: 1,
            mana: Vec::<ManaPayment>::new(),
            target: None,
            leadership: false,
        },
    );
}

/// Rien de la parade ne reste : ni parade en cours, ni carte jouée, l'unité prête, l'ennemi
/// toujours à parer.
fn assert_block_cancelled(session: &GameSession, catalog: &Catalog) {
    let screen = combat(session, catalog);
    assert_eq!(screen.in_progress, None);
    let seen = session.view();
    assert!(seen.hand.contains(&card(March, 1)) && seen.hand.contains(&card(Promise, 1)));
    assert!(seen.units.iter().all(|unit| unit.ready));
    assert!(screen.enemies[0].block.is_some());
}

// C11 — MKUE p. 5, *Reverting* : « Annuler » reprend tout jusqu'avant « Parer ».
#[test]
fn cancel_block_reverts_to_before_declaration() {
    let catalog = french();
    let mut session = wolf_session();
    insufficient_block(&mut session);
    let screen = combat(&session, &catalog);
    assert_eq!(
        screen
            .in_progress
            .as_ref()
            .map(|panel| panel.total.value.as_str()),
        Some("4 / 6")
    );
    let button = in_progress_button(&screen);
    assert_eq!(button.label, "Annuler");
    session.command(button.command);
    session.run_frame();
    assert_block_cancelled(&session, &catalog);
}

// C12 — Entrée ne valide pas une parade insuffisante.
#[test]
fn enter_does_not_resolve_insufficient_block() {
    let catalog = french();
    let mut session = wolf_session();
    insufficient_block(&mut session);
    session.shortcut(Shortcut::Confirm);
    session.run_frame();
    let screen = combat(&session, &catalog);
    assert_eq!(
        screen
            .in_progress
            .as_ref()
            .map(|panel| panel.total.value.as_str()),
        Some("4 / 6")
    );
}

// C12 — Échap annule la parade insuffisante, comme « Annuler ».
#[test]
fn escape_cancels_insufficient_block() {
    let catalog = french();
    let mut session = wolf_session();
    insufficient_block(&mut session);
    session.shortcut(Shortcut::Cancel);
    session.run_frame();
    assert_block_cancelled(&session, &catalog);
}

/// Les Rôdeurs seuls, en phase d'attaque, l'attaque déclarée et Marche jouée de côté : 1 / 3.
fn attack_session() -> GameSession {
    let events = arena_events(VILLAGE, &fight_hand(), &[(prowlers(), ORCS_2)], &[]);
    let mut session = session_of(events);
    play(&mut session, &Action::Challenge { enemy: ORCS_2 });
    play(&mut session, &Action::StartCombat);
    play(&mut session, &Action::NextCombatPhase);
    play(&mut session, &Action::NextCombatPhase);
    play(&mut session, &declare(&[prowlers()]));
    play(&mut session, &sideways_attack(card(March, 1)));
    session
}

/// Plus d'attaque en cours, Marche revenue en main, les Rôdeurs toujours une cible.
fn assert_attack_cancelled(session: &GameSession, catalog: &Catalog) {
    let screen = combat(session, catalog);
    assert_eq!(screen.in_progress, None);
    assert!(session.view().hand.contains(&card(March, 1)));
    assert!(
        screen
            .attack
            .as_ref()
            .is_some_and(|attack| attack.command.is_some())
    );
}

// C20 — MKUE p. 5, *Reverting* : « Annuler » reprend tout jusqu'avant « Attaquer » ; Entrée ne
// valide pas.
#[test]
fn cancel_attack_reverts_to_before_declaration() {
    let catalog = french();
    let mut session = attack_session();
    session.shortcut(Shortcut::Confirm);
    session.run_frame();
    let screen = combat(&session, &catalog);
    assert_eq!(
        screen
            .in_progress
            .as_ref()
            .map(|panel| panel.total.value.as_str()),
        Some("1 / 3")
    );
    let button = in_progress_button(&screen);
    assert_eq!(button.label, "Annuler");
    session.command(button.command);
    session.run_frame();
    assert_attack_cancelled(&session, &catalog);
}

// C20 — Échap annule l'attaque insuffisante.
#[test]
fn escape_cancels_insufficient_attack() {
    let catalog = french();
    let mut session = attack_session();
    session.shortcut(Shortcut::Cancel);
    session.run_frame();
    assert_attack_cancelled(&session, &catalog);
}

// --- Raccourcis (spec 071) ---

// Spec 071, C20 : « Valider » dit Entrée, « Annuler » dit Échap.
#[test]
fn combat_buttons_show_shortcut() {
    let catalog = french();
    let shortcut = |state: &GameState| {
        let button = in_progress_button(&shown(state, &catalog));
        (button.label, button.shortcut)
    };
    let state = blocking();
    assert_eq!(
        shortcut(&state),
        ("Annuler".to_string(), Some("Échap".to_string()))
    );
    let state = done(&state, option(card(Rage, 1), 1));
    let state = done(&state, sideways_block(card(March, 1)));
    let state = done(&state, sideways_block(card(Promise, 1)));
    assert_eq!(
        shortcut(&state),
        ("Valider la parade".to_string(), Some("Entrée".to_string()))
    );
    let state = to_attack_phase(&alone());
    let state = done(&state, declare(&[prowlers()]));
    assert_eq!(
        shortcut(&state),
        ("Annuler".to_string(), Some("Échap".to_string()))
    );
    let state = done(&state, option(card(Rage, 1), 0));
    let state = done(&state, sideways_attack(card(March, 1)));
    assert_eq!(
        shortcut(&state),
        ("Valider l'attaque".to_string(), Some("Entrée".to_string()))
    );
}
