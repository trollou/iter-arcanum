//! Scénarios de la spec 031 côté affichage (déroulé d'un combat).

use arcanum_core::UndoStatus;
use arcanum_core::{
    Action, AttackKind, BasicAction, BasicColor, CardId, CardPlay, Effect, Element, Enemy,
    EnemyColor, EnemyToken, Event, GameConfig, GameState, Hero, Hex, Resistance, Scenario,
    SidewaysEffect, apply, decide, legal_actions, view,
};
use arcanum_i18n::{Catalog, Language};
use arcanum_ui::{
    BoardCamera, CombatChoice, CombatScreen, Command, EndTurnChoice, GameScreen, GameSession,
    HandCard, Icon, OverlayKind, ScreenMode, ScreenPhase, Shortcut, StepState, ValuePiece,
    board_screen, combat_screen, effect_text, game_screen, hex_center, repository_assets_dir,
};
use eframe::egui::{Pos2, Rect, pos2, vec2};

mod support;

// L'arène des tests de combat d'arcanum-core, partagée plutôt que recopiée (bilan de la spec 032).
#[path = "../../arcanum-core/tests/arene/mod.rs"]
mod arene;
use arene::*;

use support::{board_input, combat_buttons, combat_labels, play, session_of, to_fight_banner};

use BasicAction::*;

fn french() -> Catalog {
    Catalog::load(
        &repository_assets_dir().join("i18n"),
        &Language("fr".into()),
    )
    .expect("fr.json du dépôt")
}

fn config() -> GameConfig {
    GameConfig {
        seed: 0,
        scenario: Scenario::sandbox(),
        hero: Hero::Arythea,
        dummy_hero: Default::default(),
    }
}

/// Partie dont la main est exactement `hand`, le deck d'Arythea derrière. Les Blessures y sont
/// mises par le deck : l'événement qui les donne arrive avec la phase d'assignation des dégâts.
fn with_hand(hand: &[CardId]) -> GameState {
    let mut order = hand.to_vec();
    order.extend(
        Hero::Arythea
            .starting_deck()
            .into_iter()
            .filter(|card| !hand.contains(card)),
    );
    let mut events = vec![
        Event::GameStarted { config: config() },
        Event::DeckShuffled {
            player: SOLO,
            order,
        },
    ];
    events.extend(
        hand.iter()
            .map(|&card| Event::CardDrawn { player: SOLO, card }),
    );
    events.iter().fold(GameState::default(), apply)
}

fn sideways(state: &GameState, held: CardId) -> GameState {
    let action = Action::PlayCard {
        card: held,
        play: CardPlay::Sideways(SidewaysEffect::Influence),
    };
    decide(state, SOLO, &action, &mut config().rngs())
        .expect("légal")
        .iter()
        .fold(state.clone(), apply)
}

fn screen_of(state: &GameState, mode: &ScreenMode, catalog: &Catalog) -> GameScreen {
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

fn ending() -> ScreenMode {
    ScreenMode::EndingTurn(EndTurnChoice {
        discard: Vec::new(),
        reclaim: None,
        glade: None,
        steady_tempo: None,
    })
}

fn held(screen: &GameScreen, card: CardId) -> &HandCard {
    screen
        .hand
        .iter()
        .find(|held| held.card == card)
        .expect("carte en main")
}

// Spec 031 C38
#[test]
fn hand_shows_wounds_and_panel_shows_fame() {
    // MKUE p. 4, 3.d : une Blessure ne se joue d'aucune façon.
    let catalog = french();
    let state = with_hand(&[wound(1), card(March, 1)]);
    let screen = screen_of(&state, &ScreenMode::Normal, &catalog);
    let shown = held(&screen, wound(1));
    assert_eq!(shown.name, "Blessure");
    assert_eq!(shown.color, None);
    assert!(shown.plays.is_empty() && shown.sideways.is_empty());
    assert!(shown.strong_plays.is_empty() && shown.paid_plays.is_empty());
    assert!(!held(&screen, card(March, 1)).sideways.is_empty());
    // Spec 073, C23 : la Gloire et le seuil du niveau suivant (issue privée 259 : ce qui est dessiné).
    assert_eq!(screen.player_level, "Niveau 1");
    assert_eq!(screen.player_fame.value, "0 / 3");
}

// Spec 031 C28 et C28 bis, côté affichage
#[test]
fn wounds_get_a_discard_box_only_for_slow_recovery() {
    // MKUE p. 11, 7.a : pas de Blessure défaussée en fin de tour ; p. 6, 7.a : sauf au repos.
    // Spec 034 (C17) : une carte jouée, la main n'a plus que des Blessures, repos prolongé permis ;
    // sans repos, pas de case : spec 034, C31 (`blessures_et_soins.rs`).
    let catalog = french();
    let state = sideways(&with_hand(&[wound(1), card(March, 1)]), card(March, 1));
    let screen = screen_of(&state, &ending(), &catalog);
    assert!(held(&screen, wound(1)).discard_toggle.is_some());

    let state = with_hand(&[wound(1), wound(2)]);
    let screen = screen_of(&state, &ending(), &catalog);
    assert!(held(&screen, wound(1)).discard_toggle.is_some());
    assert!(held(&screen, wound(2)).discard_toggle.is_some());
}

// --- Écran de combat ---

/// Plaine de la tuile 1, voisine du village : on y pose un second jeton pour un combat à deux.
const PLAIN_SW: Hex = Hex::new(1, -2);

/// Bac à sable construit par événements, figurine au village, Rôdeurs sur les orques ② et
/// Creuseurs sur la plaine voisine ; la main `hand`, le deck d'Arythea derrière.
fn arena(hand: &[CardId]) -> GameState {
    arena_events(hand).iter().fold(GameState::default(), apply)
}

/// Les événements de `arena`.
fn arena_events(hand: &[CardId]) -> Vec<Event> {
    arena_events_with(hand, &[(prowlers(), ORCS_2), (diggers(), PLAIN_SW)])
}

/// Les événements de `arena`, avec les jetons `placed` (une pile par couleur, ces jetons dessus).
fn arena_events_with(hand: &[CardId], placed: &[(EnemyToken, Hex)]) -> Vec<Event> {
    arene::arena_events(VILLAGE, hand, placed, &[])
}

fn fight_hand() -> Vec<CardId> {
    vec![card(March, 1), card(Rage, 1), card(Promise, 1)]
}

/// Les deux ennemis défiés depuis le village, combat engagé.
fn fighting() -> GameState {
    let state = arena(&fight_hand());
    let state = done(&state, Action::Challenge { enemy: ORCS_2 });
    let state = done(&state, Action::Challenge { enemy: PLAIN_SW });
    done(&state, Action::StartCombat)
}

fn combat_of(state: &GameState, choice: &CombatChoice, catalog: &Catalog) -> Option<CombatScreen> {
    let legal = legal_actions(state, SOLO);
    combat_screen(
        &view(state, SOLO),
        &legal,
        choice,
        UndoStatus::Possible,
        catalog,
    )
}

fn shown(state: &GameState, catalog: &Catalog) -> CombatScreen {
    combat_of(state, &CombatChoice::default(), catalog).expect("écran de combat")
}

/// L'action que joue la commande, qui doit être `Command::Play`.
fn played(state: &GameState, command: Option<Command>) -> Action {
    let Some(Command::Play(index)) = command else {
        panic!("le bouton joue une action légale : {command:?}");
    };
    legal_actions(state, SOLO)[index].clone()
}

// Spec 031 C35
#[test]
fn panel_offers_challenge_and_start() {
    // MKUE p. 7, *Challenging Rampaging Enemies* : défier un ennemi voisin.
    let catalog = french();
    // Spec 075 : les boutons du plateau, lus par leur infobulle (issue privée 259).
    let labels = |state: &GameState| -> Vec<String> {
        combat_labels(&screen_of(state, &ScreenMode::Normal, &catalog))
    };
    let state = arena(&fight_hand());
    assert_eq!(
        labels(&state),
        [
            "Défier Rôdeurs (orques maraudeurs)",
            "Défier Creuseurs (orques maraudeurs)"
        ]
    );
    let state = done(&state, Action::Challenge { enemy: ORCS_2 });
    assert_eq!(
        to_fight_banner(&screen_of(&state, &ScreenMode::Normal, &catalog)).as_deref(),
        Some("À combattre : Rôdeurs (orques maraudeurs)")
    );
    assert_eq!(
        labels(&state),
        ["Défier Creuseurs (orques maraudeurs)", "Engager le combat"]
    );
    let screen = screen_of(&state, &ScreenMode::Normal, &catalog);
    assert_eq!(
        played(&state, combat_buttons(&screen)[1].command),
        Action::StartCombat
    );
    // En combat, les boutons des phases sont sur l'écran de combat, plus dans le panneau.
    assert!(labels(&done(&state, Action::StartCombat)).is_empty());
}

// Spec 031 C36 — Q11
#[test]
fn combat_screen_replaces_the_board() {
    let catalog = french();
    let state = arena(&fight_hand());
    let state = done(&state, Action::Challenge { enemy: ORCS_2 });
    assert_eq!(combat_of(&state, &CombatChoice::default(), &catalog), None);

    let state = done(&state, Action::StartCombat);
    let screen = shown(&state, &catalog);
    assert_eq!(screen.title, "Combat");
    assert!(!screen.viewing_board);
    assert_eq!(screen.board_toggle.label, "Voir le plateau");
    assert_eq!(screen.board_toggle.command, Command::ToggleCombatBoard);
    // Le plateau montré pendant le combat n'offre aucun déplacement.
    assert!(
        legal_actions(&state, SOLO)
            .iter()
            .all(|action| !matches!(action, Action::MoveHero { .. } | Action::Explore { .. }))
    );
    let choice = CombatChoice {
        targets: Vec::new(),
        viewing_board: true,
    };
    let screen = combat_of(&state, &choice, &catalog).expect("écran de combat");
    assert!(screen.viewing_board);
    assert_eq!(screen.board_toggle.label, "Retour au combat");

    // MKUE p. 9, 5.a : le combat fini, le plateau revient.
    // La phase d'assignation des dégâts est automatique : deux pas mènent à l'attaque.
    let state = done(&state, Action::NextCombatPhase);
    let state = done(&state, Action::NextCombatPhase);
    let state = done(&state, Action::EndCombat);
    assert_eq!(combat_of(&state, &choice, &catalog), None);
}

// Spec 031 C36 bis
#[test]
fn combat_screen_lists_enemies_totals_and_history() {
    // MKUE p. 8 et 9 : les quatre phases ; parade, dégâts, attaque.
    let catalog = french();
    let state = fighting();
    let screen = shown(&state, &catalog);
    let steps: Vec<(&str, StepState)> = screen
        .phases
        .iter()
        .map(|step| (step.label.as_str(), step.state))
        .collect();
    assert_eq!(
        steps,
        [
            ("Phase d'attaque à distance", StepState::Current),
            ("Phase de parade", StepState::Upcoming),
            ("Phase d'assignation des dégâts", StepState::Upcoming),
            ("Phase d'attaque", StepState::Upcoming),
        ]
    );
    // Spec 033 : plus rien d'ignoré, la ligne a disparu ; pas d'Invocateur, pas d'avertissement.
    assert_eq!(screen.summon_warning, None);
    assert_eq!(
        screen
            .phase_button
            .as_ref()
            .map(|button| button.label.as_str()),
        Some("Phase suivante")
    );
    let [rodeurs, fouisseurs] = &screen.enemies[..] else {
        panic!("deux ennemis : {:?}", screen.enemies);
    };
    // Spec 070, C13 et C14 : le nom seul, plus d'état « en jeu ».
    assert_eq!(rodeurs.name, "Rôdeurs");
    // Spec 071, C8 : valeurs en icônes, lues par leur texte de repli.
    assert_eq!(
        values_text(&rodeurs.values),
        "Armure 3 · Attaque 4 · Gloire 2"
    );
    assert_eq!(rodeurs.status, None);
    assert!(rodeurs.target.is_some());
    // Fortifiés : seul le siège les atteint à distance (MKUE p. 9, *Ranged and Siege Attack Phase*
    // 3) ; depuis la spec 032 (Q1), ils sont tout de même une cible.
    assert_eq!(
        fouisseurs.status.as_deref(),
        Some("fortifié : siège seulement")
    );
    assert!(fouisseurs.target.is_some());

    // Parade : Rage (Parade 2) contre les Rôdeurs (attaque 4), ratée.
    let state = done(&state, Action::NextCombatPhase);
    let screen = shown(&state, &catalog);
    assert_eq!(screen.phases[0].state, StepState::Done);
    assert_eq!(screen.phases[1].state, StepState::Current);
    assert_eq!(screen.attack, None);
    let block = screen.enemies[0].block.as_ref().expect("Parer");
    // Spec 070, C5.
    assert_eq!(block.label, "Parer");
    let state = done(&state, played(&state, Some(block.command)));
    let state = done(&state, option(card(Rage, 1), 1));
    let screen = shown(&state, &catalog);
    assert!(screen.enemies[0].engaged && !screen.enemies[1].engaged);
    let panel = screen.in_progress.as_ref().expect("parade en cours");
    assert_eq!(panel.cards, ["Rage"]);
    assert_eq!(panel.needed, None);
    assert_eq!(panel.total.value, "2 / 4");
    // Spec 070, C7 : insuffisante, l'écran propose « Annuler » ; la parade ratée passe par
    // l'action légale du moteur.
    let resolve = panel.resolve.as_ref().expect("Annuler");
    assert_eq!(resolve.label, "Annuler");
    assert_eq!(resolve.command, Command::CancelBlock);
    let state = done(&state, Action::ResolveBlock);

    // Dégâts, puis attaque : Marche de côté (Attaque 1) contre les Rôdeurs (armure 3), ratée.
    let state = done(&state, Action::NextCombatPhase);
    let state = done(
        &state,
        Action::DeclareAttack {
            targets: vec![foe(&state, ORCS_2)],
        },
    );
    let state = done(&state, sideways_attack(card(March, 1)));
    let screen = shown(&state, &catalog);
    let panel = screen.in_progress.as_ref().expect("attaque en cours");
    assert_eq!(panel.cards, ["Marche"]);
    assert_eq!(
        panel.needed.as_deref(),
        Some("Il faut la somme des armures : 3")
    );
    assert_eq!(panel.total.value, "1 / 3");
    let state = done(&state, Action::ResolveAttack);
    let screen = shown(&state, &catalog);
    assert_eq!(
        screen.enemies[0].history,
        [
            "Phase de parade : Rage, 2 / 4 ratée",
            "Dégâts : Rôdeurs, 4 : 2 blessure(s)",
            "Phase d'attaque : Marche, 1 / 3 ratée",
        ]
    );
    assert_eq!(
        screen.enemies[1].history,
        ["Dégâts : Creuseurs, 3 : 2 blessure(s)"]
    );
    assert_eq!(
        screen
            .phase_button
            .as_ref()
            .map(|button| button.label.as_str()),
        Some("Terminer le combat")
    );
}

// Spec 031 C36 bis : un ennemi vaincu reste affiché, grisé.
#[test]
fn defeated_enemy_stays_on_the_combat_screen() {
    // MKUE p. 9, 5.a.
    let catalog = french();
    let state = fighting();
    // La phase d'assignation des dégâts est automatique : deux pas mènent à l'attaque.
    let state = done(&state, Action::NextCombatPhase);
    let state = done(&state, Action::NextCombatPhase);
    let state = done(
        &state,
        Action::DeclareAttack {
            targets: vec![foe(&state, ORCS_2)],
        },
    );
    // Rage (Attaque 2) et Marche de côté (Attaque 1) : 3 contre l'armure 3 des Rôdeurs.
    let state = done(&state, option(card(Rage, 1), 0));
    let state = done(&state, sideways_attack(card(March, 1)));
    let state = done(&state, Action::ResolveAttack);
    let screen = shown(&state, &catalog);
    let rodeurs = &screen.enemies[0];
    assert!(rodeurs.defeated);
    assert_eq!(rodeurs.status.as_deref(), Some("vaincu"));
    assert_eq!(rodeurs.target, None);
    // Spec 040 : la réputation des orques vaincus suit la ligne de l'attaque.
    assert_eq!(
        rodeurs.history[rodeurs.history.len() - 2..],
        [
            "Phase d'attaque : Rage, Marche, 3 / 3 réussie",
            "Réputation +1"
        ]
    );
}

/// Les Rôdeurs vaincus en phase d'attaque : Rage (Attaque 2) et Marche de côté (Attaque 1).
fn prowlers_defeated() -> GameState {
    let state = fighting();
    let state = done(&state, Action::NextCombatPhase);
    let state = done(&state, Action::NextCombatPhase);
    let state = done(
        &state,
        Action::DeclareAttack {
            targets: vec![foe(&state, ORCS_2)],
        },
    );
    let state = done(&state, option(card(Rage, 1), 0));
    let state = done(&state, sideways_attack(card(March, 1)));
    done(&state, Action::ResolveAttack)
}

// Spec 031 C38 : la gloire des ennemis vaincus.
#[test]
fn panel_shows_fame_after_a_victory() {
    // MKUE p. 9, 5.b : les Rôdeurs rapportent 2 de gloire.
    let catalog = french();
    let screen = screen_of(&prowlers_defeated(), &ScreenMode::Normal, &catalog);
    assert_eq!(screen.player_fame.value, "2 / 3");
}

// Spec 031 C39 — Q5
#[test]
fn defeated_enemy_space_shows_plain_terrain() {
    let catalog = french();
    let state = done(&prowlers_defeated(), Action::EndCombat);
    let seen = view(&state, SOLO);
    let legal = legal_actions(&state, SOLO);
    let camera = BoardCamera {
        center: Pos2::ZERO,
        zoom: 30.0,
    };
    let area = Rect::from_min_size(pos2(0.0, 0.0), vec2(1000.0, 700.0));
    let cursor = camera.to_screen(area, hex_center(ORCS_2));
    let screen = board_screen(
        &board_input(&seen, &legal, camera, area, Some(cursor)),
        &catalog,
    );
    assert!(
        screen
            .overlays
            .iter()
            .all(|overlay| !(overlay.hex == ORCS_2 && overlay.kind == OverlayKind::Rampaging))
    );
    assert_eq!(screen.hover_lines[..2], ["Tuile de campagne 2", "Colline"]);
    assert!(
        screen
            .hover_lines
            .iter()
            .all(|line| !line.contains("orques") && !line.contains("ennemi")),
        "{:?}",
        screen.hover_lines
    );
    // Les Creuseurs, eux, restent sur leur case.
    assert!(
        screen
            .overlays
            .iter()
            .any(|overlay| overlay.hex == PLAIN_SW && overlay.kind == OverlayKind::Rampaging)
    );
}

// Spec 031 C37
#[test]
fn target_selection_declares_the_attack() {
    // MKUE p. 9, 4.a : une attaque peut viser plusieurs ennemis.
    let catalog = french();
    let state = fighting();
    // La phase d'assignation des dégâts est automatique : deux pas mènent à l'attaque.
    let state = done(&state, Action::NextCombatPhase);
    let state = done(&state, Action::NextCombatPhase);
    let screen = shown(&state, &catalog);
    let attack = screen.attack.as_ref().expect("Attaquer");
    assert_eq!(attack.label, "Attaquer");
    assert_eq!(attack.command, None);
    let toggle = screen.enemies[1].target.as_ref().expect("case cible");
    assert_eq!(toggle.label, "cible");
    assert!(!toggle.checked);
    assert_eq!(toggle.command, Command::ToggleTarget(foe(&state, PLAIN_SW)));

    // Cochés dans l'autre ordre : l'attaque garde l'ordre du combat.
    let choice = CombatChoice {
        targets: vec![foe(&state, PLAIN_SW), foe(&state, ORCS_2)],
        viewing_board: false,
    };
    let screen = combat_of(&state, &choice, &catalog).expect("écran de combat");
    assert!(
        screen
            .enemies
            .iter()
            .all(|enemy| enemy.target.as_ref().is_some_and(|toggle| toggle.checked))
    );
    let attack = screen.attack.as_ref().expect("Attaquer");
    assert_eq!(
        played(&state, attack.command),
        Action::DeclareAttack {
            targets: vec![foe(&state, ORCS_2), foe(&state, PLAIN_SW)]
        }
    );

    // Une attaque déclarée : plus de case ni de bouton « Attaquer » ; la somme des armures.
    let state = done(
        &state,
        Action::DeclareAttack {
            targets: vec![foe(&state, ORCS_2), foe(&state, PLAIN_SW)],
        },
    );
    let screen = shown(&state, &catalog);
    assert!(screen.enemies.iter().all(|enemy| enemy.target.is_none()));
    assert!(screen.enemies.iter().all(|enemy| enemy.engaged));
    assert_eq!(screen.attack, None);
    let panel = screen.in_progress.as_ref().expect("attaque en cours");
    assert_eq!(
        panel.needed.as_deref(),
        Some("Il faut la somme des armures : 3 + 3 = 6")
    );
    assert_eq!(panel.total.value, "0 / 6");

    // Ratée (Q10) : ces ennemis ne se ciblent plus dans cette phase.
    let state = done(&state, Action::ResolveAttack);
    let screen = shown(&state, &catalog);
    assert!(screen.enemies.iter().all(|enemy| enemy.target.is_none()));
    assert_eq!(screen.attack, None);
    assert_eq!(
        screen.enemies[0].history,
        [
            "Dégâts : Rôdeurs, 4 : 2 blessure(s)",
            "Phase d'attaque : —, 0 / 6 ratée"
        ]
    );
}

// --- Dans la fenêtre : session et raccourci ---

/// Session de l'humain sur l'arène, la partie avancée jusqu'à sa première décision.
fn session() -> GameSession {
    session_of(arena_events(&fight_hand()))
}

fn click(session: &mut GameSession, command: Option<Command>) {
    session.command(command.expect("bouton actif"));
    session.run_frame();
}

// Spec 031 C36 et C37, dans la session ; raccourci Entrée (Q8, docs/raccourcis.md).
#[test]
fn session_selects_targets_and_enter_resolves() {
    let catalog = french();
    let mut session = session();
    play(&mut session, &Action::Challenge { enemy: ORCS_2 });
    play(&mut session, &Action::StartCombat);
    let combat = |session: &GameSession| session.combat_screen(&catalog).expect("écran de combat");

    // « Voir le plateau », puis « Retour au combat ».
    let toggle = combat(&session).board_toggle.command;
    click(&mut session, Some(toggle));
    assert!(combat(&session).viewing_board);
    click(&mut session, Some(toggle));
    assert!(!combat(&session).viewing_board);

    // Entrée sans attaque en cours : rien ne se passe.
    session.shortcut(Shortcut::Confirm);
    session.run_frame();
    assert_eq!(combat(&session).phases[0].state, StepState::Current);

    play(&mut session, &Action::NextCombatPhase);
    play(&mut session, &Action::NextCombatPhase);
    // Spec 070, C1 : les Rôdeurs seuls sont cochés d'office, sans case.
    assert_eq!(combat(&session).enemies[0].target, None);
    let attack = combat(&session).attack.expect("Attaquer");
    click(&mut session, attack.command);
    assert!(combat(&session).enemies[0].engaged);
    // L'attaque déclarée, la case cochée est oubliée.
    play(&mut session, &option(card(Rage, 1), 0));
    play(&mut session, &sideways_attack(card(March, 1)));

    // MKUE p. 9, 5.a : Entrée valide l'attaque, 3 contre l'armure 3.
    session.shortcut(Shortcut::Confirm);
    session.run_frame();
    let screen = combat(&session);
    assert_eq!(screen.in_progress, None);
    assert!(screen.enemies[0].defeated);
}

// Spec 031 C40
#[test]
fn every_combat_key_exists_in_french() {
    for code in ["fr", "en"] {
        let catalog = Catalog::load(
            &repository_assets_dir().join("i18n"),
            &Language(code.into()),
        )
        .expect("fichier de langue du dépôt");
        let compute = |state: &GameState| {
            screen_of(state, &ScreenMode::Normal, &catalog);
            combat_of(state, &CombatChoice::default(), &catalog);
            combat_of(
                state,
                &CombatChoice {
                    targets: vec![foe(state, ORCS_2)],
                    viewing_board: true,
                },
                &catalog,
            );
        };
        let state = arena(&fight_hand());
        compute(&state);
        let state = done(&state, Action::Challenge { enemy: ORCS_2 });
        compute(&state);
        let state = done(&state, Action::Challenge { enemy: PLAIN_SW });
        let state = done(&state, Action::StartCombat);
        compute(&state);
        let state = done(&state, Action::NextCombatPhase);
        compute(&state);
        let state = done(
            &state,
            Action::DeclareBlock {
                target: foe(&state, ORCS_2),
            },
        );
        let state = done(&state, option(card(Rage, 1), 1));
        compute(&state);
        let state = done(&state, Action::ResolveBlock);
        // Les Blessures arrivent dans la main avec la phase suivante.
        let state = done(&state, Action::NextCombatPhase);
        compute(&state);
        let state = done(
            &state,
            Action::DeclareAttack {
                targets: vec![foe(&state, ORCS_2)],
            },
        );
        compute(&state);
        compute(&prowlers_defeated());
        compute(&done(&prowlers_defeated(), Action::EndCombat));
        assert_eq!(catalog.gaps(), vec![], "{code}");
    }
}

// Spec 031, Interface : en combat, la main ne propose que ce qui se joue maintenant.
#[test]
fn hand_in_combat_offers_only_what_plays_now() {
    // MKUE p. 6, 6.e : plus de Mouvement en combat ; p. 9 : une Attaque dans une attaque en cours.
    let catalog = french();
    let strong = |state: &GameState, held: CardId| -> Vec<String> {
        let screen = screen_of(state, &ScreenMode::Normal, &catalog);
        screen
            .hand
            .iter()
            .find(|shown| shown.card == held)
            .expect("carte en main")
            .strong_plays
            .iter()
            .map(|button| button.label.clone())
            .collect()
    };
    let state = fighting();
    assert!(strong(&state, card(March, 1)).is_empty());
    assert!(strong(&state, card(Rage, 1)).is_empty());
    // Le bandeau des ennemis à combattre laisse la place à l'écran de combat.
    assert_eq!(
        to_fight_banner(&screen_of(&state, &ScreenMode::Normal, &catalog)),
        None
    );

    let state = done(&state, Action::NextCombatPhase);
    let state = done(&state, Action::NextCombatPhase);
    let state = done(
        &state,
        Action::DeclareAttack {
            targets: vec![foe(&state, ORCS_2)],
        },
    );
    assert_eq!(strong(&state, card(Rage, 1)), ["Renforcer : Attaque 4"]);
    assert!(strong(&state, card(March, 1)).is_empty());
    let state = done(&state, Action::ResolveAttack);
    let state = done(&state, Action::EndCombat);
    // Combat terminé : le tour reprend, sans bandeau ; Marche ne se renforce plus, le Mouvement
    // ne sert plus après l'action (MKUE p. 6, 6.d ; issue privée 164).
    assert_eq!(
        to_fight_banner(&screen_of(&state, &ScreenMode::Normal, &catalog)),
        None
    );
    assert!(strong(&state, card(March, 1)).is_empty());
}

// --- Spec 032 : éléments et résistances à l'écran ---

/// Combat engagé depuis le village contre les jetons `placed`, tous défiés ; `crystals` dans
/// l'inventaire pour renforcer des cartes.
fn fighting_with(
    hand: &[CardId],
    placed: &[(EnemyToken, Hex)],
    crystals: &[BasicColor],
) -> GameState {
    let mut events = arena_events_with(hand, placed);
    events.extend(crystals.iter().map(|&color| Event::CrystalGained {
        player: SOLO,
        color,
    }));
    let mut state = events.iter().fold(GameState::default(), apply);
    for &(_, hex) in placed {
        state = done(&state, Action::Challenge { enemy: hex });
    }
    done(&state, Action::StartCombat)
}

/// Rôdeurs et Cogneurs ciblés ensemble en phase d'attaque.
fn group_attack() -> GameState {
    let hand = [card(Rage, 1), card(BattleVersatility, 1), card(March, 1)];
    let placed = [(prowlers(), ORCS_2), (ironclads(), PLAIN_SW)];
    let state = fighting_with(&hand, &placed, &[BasicColor::Red]);
    // Sans dégâts : les 6 Blessures assommeraient le héros et défausseraient la main (spec 034).
    let state = skip_to_attack_phase(&state);
    done(
        &state,
        Action::DeclareAttack {
            targets: vec![foe(&state, ORCS_2), foe(&state, PLAIN_SW)],
        },
    )
}

// Spec 032 C18
#[test]
fn attack_shows_shared_resistances_and_calculation() {
    // MKUE p. 9, *Ranged and Siege Attack Phase* 4.a ; Compendium, *Combat* S12.
    let catalog = french();
    let state = group_attack();
    let state = done(&state, option(card(Rage, 1), 0));
    let state = done(&state, sideways_attack(card(March, 1)));
    let state = done(&state, strong_crystal(card(BattleVersatility, 1), 2));
    let screen = shown(&state, &catalog);
    let cuirasses = &screen.enemies[1];
    // Spec 071, C10 : une icône de résistance, son nom en infobulle.
    assert_eq!(
        cuirasses
            .resistances
            .iter()
            .map(|value| (value.icon, value.tooltip.as_str()))
            .collect::<Vec<_>>(),
        [(
            Icon::Resistance(Resistance::Physical),
            "Résistance physique"
        )]
    );
    let panel = screen.in_progress.as_ref().expect("attaque en cours");
    assert_eq!(
        panel.shared.as_deref(),
        Some("Le groupe partage : résistance physique (Cogneurs)")
    );
    assert_eq!(panel.cards, ["Rage ÷2", "Marche ÷2", "Polyvalence"]);
    assert_eq!(
        panel.calculation.as_deref(),
        Some("Divisées par deux : (2 + 1) ÷ 2 = 1 · Autres : 3")
    );
    assert_eq!(panel.total.value, "4 / 6");
    assert_eq!(panel.block_rule, None);
}

// Spec 032 C19
#[test]
fn hand_marks_how_cards_will_count() {
    let catalog = french();
    let screen = screen_of(&group_attack(), &ScreenMode::Normal, &catalog);
    let rage = held(&screen, card(Rage, 1));
    let labels = |buttons: &[arcanum_ui::Button]| -> Vec<String> {
        buttons.iter().map(|button| button.label.clone()).collect()
    };
    assert_eq!(labels(&rage.plays), ["Jouer : Attaque 2 ÷2"]);
    let versatility = held(&screen, card(BattleVersatility, 1));
    let strong: Vec<String> = versatility
        .strong_plays
        .iter()
        .map(|button| button.label.clone())
        .collect();
    assert_eq!(
        strong,
        [
            "Renforcer : Attaque 4 ÷2",
            "Renforcer : Attaque de feu 3",
            "Renforcer : Attaque à distance 3 ÷2",
            "Renforcer : Attaque de siège 2 ÷2",
        ]
    );
    assert_eq!(
        // Spec 071, C16 : le bouton-icône garde « ÷2 » dans son texte de repli.
        held(&screen, card(March, 1))
            .sideways
            .iter()
            .map(|button| button.label.as_str())
            .collect::<Vec<_>>(),
        ["Attaque 1 ÷2"]
    );
    // Sans attaque en cours, aucun repère.
    let state = fighting_with(&[card(Rage, 1)], &[(ironclads(), ORCS_2)], &[]);
    let state = done(&state, Action::NextCombatPhase);
    let state = done(
        &state,
        Action::DeclareBlock {
            target: foe(&state, ORCS_2),
        },
    );
    let screen = screen_of(&state, &ScreenMode::Normal, &catalog);
    assert_eq!(
        labels(&held(&screen, card(Rage, 1)).plays),
        ["Jouer : Parade 2"]
    );
}

// Spec 032 C20
#[test]
fn block_shows_efficient_elements() {
    // MKUE p. 9, *Block Phase* 5.
    let catalog = french();
    let hand = [
        card(BattleVersatility, 1),
        card(Determination, 1),
        card(March, 1),
    ];
    let state = fighting_with(
        &hand,
        &[(ice_dragon(), ORCS_2)],
        &[BasicColor::Red, BasicColor::Blue],
    );
    let state = done(&state, Action::NextCombatPhase);
    let state = done(
        &state,
        Action::DeclareBlock {
            target: foe(&state, ORCS_2),
        },
    );
    let state = done(&state, strong_crystal(card(BattleVersatility, 1), 3));
    let state = done(&state, strong_crystal(card(Determination, 1), 0));
    let state = done(
        &state,
        Action::PlayCard {
            card: card(March, 1),
            play: CardPlay::Sideways(SidewaysEffect::Block),
        },
    );
    let screen = shown(&state, &catalog);
    assert_eq!(
        values_text(&screen.enemies[0].values),
        "Armure 7 · Attaque de glace 6 · Gloire 8"
    );
    let panel = screen.in_progress.as_ref().expect("parade en cours");
    assert_eq!(
        panel.block_rule.as_deref(),
        Some(
            "Contre une attaque de glace, les parades autres que de feu et de feu glacé sont divisées par deux"
        )
    );
    assert_eq!(panel.shared, None);
    assert_eq!(
        panel.cards,
        ["Polyvalence", "Détermination ÷2", "Marche ÷2"]
    );
    assert_eq!(
        panel.calculation.as_deref(),
        Some("Divisées par deux : (5 + 1) ÷ 2 = 3 · Autres : 3")
    );
    assert_eq!(panel.total.value, "6 / 6");
}

// Spec 032 C21
#[test]
fn fortified_group_marks_ranged_cards() {
    // MKUE p. 9, *Ranged and Siege Attack Phase* 3.a ; Compendium, *Combat* S12.
    let catalog = french();
    let hand = [card(Swiftness, 1), card(BattleVersatility, 1)];
    let state = fighting_with(&hand, &[(diggers(), ORCS_2)], &[BasicColor::White]);
    // Spec 070, C1 : seule cible possible, les Creuseurs sont cochés d'office, sans case.
    let screen = shown(&state, &catalog);
    assert!(screen.enemies[0].target.is_none());
    assert!(screen.attack.is_some_and(|attack| attack.command.is_some()));
    let state = done(
        &state,
        Action::DeclareAttack {
            targets: vec![foe(&state, ORCS_2)],
        },
    );
    let panel = shown(&state, &catalog)
        .in_progress
        .expect("attaque en cours");
    assert_eq!(
        panel.shared.as_deref(),
        Some("Le groupe partage : fortifié (Creuseurs) : attaques de siège seulement")
    );
    let screen = screen_of(&state, &ScreenMode::Normal, &catalog);
    let swiftness = held(&screen, card(Swiftness, 1));
    assert!(swiftness.strong_plays.is_empty());
    assert_eq!(swiftness.combat_note.as_deref(), Some("siège seulement"));
    let versatility = held(&screen, card(BattleVersatility, 1));
    assert_eq!(versatility.combat_note.as_deref(), Some("siège seulement"));
    assert_eq!(
        versatility
            .strong_plays
            .iter()
            .map(|button| button.label.as_str())
            .collect::<Vec<_>>(),
        ["Renforcer : Attaque de siège 2"]
    );
}

// Spec 032 C22
#[test]
fn every_elemental_effect_has_a_label() {
    let catalog = french();
    let mut labels = Vec::new();
    for element in Element::ALL {
        for kind in [AttackKind::Melee, AttackKind::Ranged, AttackKind::Siege] {
            labels.push(effect_text(
                Effect::Attack {
                    amount: 3,
                    element,
                    kind,
                },
                &catalog,
            ));
        }
        labels.push(effect_text(Effect::Block { amount: 3, element }, &catalog));
    }
    assert!(labels.contains(&"Attaque à distance de feu 3".to_string()));
    assert!(labels.contains(&"Attaque de siège de feu glacé 3".to_string()));
    assert!(labels.contains(&"Parade de feu glacé 3".to_string()));
    let mut unique = labels.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(unique.len(), labels.len(), "{labels:?}");
    assert_eq!(catalog.gaps(), vec![]);
}

// Spec 032 C23
#[test]
fn every_element_key_exists_in_french() {
    for code in ["fr", "en"] {
        let catalog = Catalog::load(
            &repository_assets_dir().join("i18n"),
            &Language(code.into()),
        )
        .expect("fichier de langue du dépôt");
        let compute = |state: &GameState| {
            screen_of(state, &ScreenMode::Normal, &catalog);
            combat_of(state, &CombatChoice::default(), &catalog);
        };
        let state = group_attack();
        compute(&state);
        compute(&done(&state, option(card(Rage, 1), 0)));
        let hand = [card(BattleVersatility, 1), card(March, 1)];
        for dragon in [Enemy::IceDragon, Enemy::FireDragon, Enemy::HighDragon] {
            let state = fighting_with(&hand, &[(EnemyToken::new(dragon, 1), ORCS_2)], &[]);
            let state = done(&state, Action::NextCombatPhase);
            let state = done(
                &state,
                Action::DeclareBlock {
                    target: foe(&state, ORCS_2),
                },
            );
            compute(&done(
                &state,
                Action::PlayCard {
                    card: card(March, 1),
                    play: CardPlay::Sideways(SidewaysEffect::Block),
                },
            ));
        }
        let state = fighting_with(&[card(Swiftness, 1)], &[(diggers(), ORCS_2)], &[]);
        compute(&done(
            &state,
            Action::DeclareAttack {
                targets: vec![foe(&state, ORCS_2)],
            },
        ));
        assert_eq!(catalog.gaps(), vec![], "{code}");
    }
}

// --- Spec 033 : capacités des ennemis ---

/// Combat engagé depuis le village contre `placed`, la pile marron réordonnée : `brown` dessus.
fn fighting_abilities(
    hand: &[CardId],
    placed: &[(EnemyToken, Hex)],
    brown: &[EnemyToken],
    crystals: &[BasicColor],
) -> GameState {
    let state = fighting_with(hand, placed, crystals);
    let mut order = brown.to_vec();
    order.extend(
        state
            .enemy_pile(EnemyColor::Brown)
            .iter()
            .copied()
            .filter(|token| !brown.contains(token)),
    );
    apply(
        state,
        &Event::EnemyPileShuffled {
            color: EnemyColor::Brown,
            order,
        },
    )
}

fn minotaur() -> EnemyToken {
    EnemyToken::new(Enemy::Minotaur, 1)
}

// Spec 033 C28
#[test]
fn ranged_phase_warns_about_summon() {
    // Spec 033, Q1 : le tirage de la créature est un point de non-retour.
    let catalog = french();
    let state = fighting_abilities(&[], &[(orc_summoners(), ORCS_2)], &[minotaur()], &[]);
    assert_eq!(
        shown(&state, &catalog).summon_warning.as_deref(),
        Some(
            "Au début de la phase de parade, Invocateurs orques invoquent une créature (pile \
             marron) : pas de retour en arrière après."
        )
    );
    let state = done(&state, Action::NextCombatPhase);
    assert_eq!(shown(&state, &catalog).summon_warning, None);
    let state = fighting_with(&[], &[(prowlers(), ORCS_2)], &[]);
    assert_eq!(shown(&state, &catalog).summon_warning, None);
}

// Spec 033 C29
#[test]
fn block_phase_shows_summoned_monster_and_stakes() {
    // MKUE p. 9, *Block Phase* 2.a : la créature invoquée remplace l'Invocateur.
    let catalog = french();
    let placed = [(orc_summoners(), ORCS_2), (ironclads(), PLAIN_SW)];
    let state = fighting_abilities(&[], &placed, &[minotaur()], &[]);
    let state = done(&state, Action::NextCombatPhase);
    let screen = shown(&state, &catalog);
    let summoners = &screen.enemies[0];
    let summoned = summoners.summoned.as_ref().expect("créature invoquée");
    assert_eq!(summoned.token, minotaur());
    assert_eq!(summoned.title, "Invoque : Minotaure");
    assert_eq!(summoned.note, "retirée après les dégâts, sans gloire");
    assert_eq!(summoned.abilities.as_deref(), Some("Capacités : brutal"));
    assert_eq!(
        screen.enemies[1].abilities.as_deref(),
        Some("Capacités : brutal")
    );
    assert_eq!(
        summoners.block.as_ref().map(|button| button.label.as_str()),
        Some("Parer")
    );
    // Spec 070, C6 et C16 : « Parer » seul, l'enjeu sur deux lignes.
    assert_eq!(
        summoners.stakes,
        [
            "À parer : 5",
            "Si non paré : 10 (brutal, 2 × 5) dégâts, 5 blessure(s)"
        ]
    );
    assert_eq!(
        screen.enemies[1].stakes,
        [
            "À parer : 4",
            "Si non paré : 8 (brutal, 2 × 4) dégâts, 4 blessure(s)"
        ]
    );
}

// Spec 033 C30
#[test]
fn swift_block_shows_doubled_target() {
    // MKUE p. 9, *Block Phase* 6.a : Chevaucheurs de loups, attaque 3, vifs.
    let catalog = french();
    let state = fighting_with(&[card(March, 1)], &[(wolf_riders(), ORCS_2)], &[]);
    let state = done(&state, Action::NextCombatPhase);
    assert_eq!(
        shown(&state, &catalog).enemies[0].stakes,
        [
            "À parer : 6 (vif, 2 × 3)",
            "Si non paré : 3 dégâts, 2 blessure(s)"
        ]
    );
    let state = done(
        &state,
        Action::DeclareBlock {
            target: foe(&state, ORCS_2),
        },
    );
    let state = done(&state, sideways_block(card(March, 1)));
    let screen = shown(&state, &catalog);
    let panel = screen.in_progress.as_ref().expect("parade en cours");
    assert_eq!(
        panel.swift.as_deref(),
        Some("Vif : il faut parer le double de son attaque, 2 × 3 = 6")
    );
    assert_eq!(panel.total.value, "1 / 6");
}

// Spec 033 C31
#[test]
fn history_shows_ability_effects() {
    let catalog = french();
    let damage_line = |enemy: Enemy, hand: &[CardId]| -> Vec<String> {
        let state = fighting_with(hand, &[(EnemyToken::new(enemy, 1), ORCS_2)], &[]);
        let state = done(&state, Action::NextCombatPhase);
        let state = done(&state, Action::NextCombatPhase);
        shown(&state, &catalog).enemies[0].history.clone()
    };
    assert_eq!(
        damage_line(Enemy::Ironclads, &[]),
        ["Dégâts : Cogneurs, brutal 2 × 4 = 8 : 4 blessure(s)"]
    );
    assert_eq!(
        damage_line(Enemy::CursedHags, &[]),
        ["Dégâts : Sorcières maudites, 3 : 2 blessure(s), toxique : 2 de plus dans la défausse"]
    );
    assert_eq!(
        damage_line(Enemy::IceDragon, &[card(March, 1), card(Rage, 1)]),
        [
            "Dégâts : Dragons de glace, 6 : 3 blessure(s), paralysant : 2 carte(s) défaussée(s) \
             de la main"
        ]
    );
}

// Spec 033 C32
#[test]
fn every_ability_key_exists_in_french() {
    for code in ["fr", "en"] {
        let catalog = Catalog::load(
            &repository_assets_dir().join("i18n"),
            &Language(code.into()),
        )
        .expect("fichier de langue du dépôt");
        let compute = |state: &GameState| {
            screen_of(state, &ScreenMode::Normal, &catalog);
            combat_of(state, &CombatChoice::default(), &catalog);
        };
        let cold_toughness = card(ColdToughness, 1);
        let placed = [(orc_summoners(), ORCS_2), (wolf_riders(), PLAIN_SW)];
        let state = fighting_abilities(
            &[cold_toughness, card(March, 1)],
            &placed,
            &[minotaur()],
            &[BasicColor::Blue],
        );
        compute(&state);
        let state = done(&state, Action::NextCombatPhase);
        compute(&state);
        let blocking = done(
            &state,
            Action::DeclareBlock {
                target: foe(&state, PLAIN_SW),
            },
        );
        compute(&blocking);
        // Endurance givrée renforcée dit son bonus contre l'ennemi paré.
        let label = screen_of(&blocking, &ScreenMode::Normal, &catalog)
            .hand
            .iter()
            .find(|held| held.card == cold_toughness)
            .map(|held| {
                held.strong_plays
                    .iter()
                    .map(|button| button.label.clone())
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .unwrap_or_default();
        if code == "fr" {
            assert!(
                label.contains("Parade de glace 5 + 1 (capacités de l'ennemi paré)"),
                "{label}"
            );
        }
        let state = done(&state, Action::NextCombatPhase);
        compute(&state);
        for enemy in [Enemy::CursedHags, Enemy::IceDragon] {
            let state = fighting_with(
                &[card(March, 1)],
                &[(EnemyToken::new(enemy, 1), ORCS_2)],
                &[],
            );
            let state = done(&state, Action::NextCombatPhase);
            compute(&done(&state, Action::NextCombatPhase));
        }
        assert_eq!(catalog.gaps(), vec![], "{code}");
    }
}

/// Les valeurs d'une carte d'ennemi en texte de repli (spec 071, L1) : « Armure 3 · Attaque 4 ».
fn values_text(pieces: &[ValuePiece]) -> String {
    pieces
        .iter()
        .map(|piece| match piece {
            ValuePiece::Icon(value) => value.fallback_text(),
            ValuePiece::Text(text) => text.clone(),
        })
        .collect::<Vec<_>>()
        .join(" · ")
}
