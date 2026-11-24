//! Scénarios de la spec 052 côté affichage (moteur d'effets) : effets en cours, coûts réduits,
//! case non sécurisée, conversions en combat, retraite forcée et Temporisation à la fin du tour.

use arcanum_core::{
    Action, AdvancedAction, CardId, CardPlay, GameState, Hex, ManaPayment, UndoStatus, Unit,
    UnitCard, legal_actions, view,
};
use arcanum_i18n::{Catalog, Language};
use arcanum_ui::{
    BoardCamera, BoardScreen, CombatChoice, CombatScreen, Command, DiscardToggle, EndTurnChoice,
    EndTurnPrompt, GameScreen, ScreenMode, ScreenPhase, board_screen, combat_screen, game_screen,
    hex_center, repository_assets_dir,
};
use eframe::egui::{Pos2, Rect, pos2, vec2};

// L'arène des tests d'arcanum-core, partagée plutôt que recopiée.
#[path = "../../arcanum-core/tests/arene/mod.rs"]
mod arene;
use arene::*;

mod support;
use support::{board_input, play, session_of};

use AdvancedAction::*;
use arcanum_core::BasicAction::*;

// --- Outils ---

/// Le lac de la campagne 1, voisin du village (spec 052, exemple).
const LAKE: Hex = Hex::new(2, -4);
/// La forêt à l'O de la clairière, à deux pas du village par la clairière.
const WEST_FOREST: Hex = Hex::new(0, -3);

fn french() -> Catalog {
    Catalog::load(
        &repository_assets_dir().join("i18n"),
        &Language("fr".into()),
    )
    .expect("fr.json du dépôt")
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

fn index_of(state: &GameState, wanted: &Action) -> usize {
    legal_actions(state, SOLO)
        .iter()
        .position(|action| action == wanted)
        .unwrap_or_else(|| panic!("{wanted:?} doit être légale"))
}

fn board(state: &GameState, cursor: Option<Hex>, catalog: &Catalog) -> BoardScreen {
    let seen = view(state, SOLO);
    let legal = legal_actions(state, SOLO);
    let area = Rect::from_min_size(pos2(0.0, 0.0), vec2(1000.0, 700.0));
    let camera = BoardCamera {
        center: Pos2::ZERO,
        zoom: 30.0,
    };
    board_screen(
        &board_input(
            &seen,
            &legal,
            camera,
            area,
            cursor.map(|hex| camera.to_screen(area, hex_center(hex))),
        ),
        catalog,
    )
}

fn combat_of(state: &GameState, catalog: &Catalog) -> CombatScreen {
    let legal = legal_actions(state, SOLO);
    combat_screen(
        &view(state, SOLO),
        &legal,
        &CombatChoice::default(),
        UndoStatus::Possible,
        catalog,
    )
    .expect("combat engagé")
}

fn end_turn_prompt(state: &GameState, catalog: &Catalog) -> EndTurnPrompt {
    screen_in(
        state,
        &ScreenMode::EndingTurn(EndTurnChoice::default()),
        catalog,
    )
    .end_turn_prompt
    .expect("mode fin du tour")
}

fn strong_option(card: CardId, die: u8, option: u8) -> Action {
    Action::PlayCard {
        card,
        play: CardPlay::Strong {
            gain: None,
            extra_mana: None,
            option,
            discard: None,
            mana: ManaPayment::Die(die),
            set_dice: [None, None],
            with: None,
        },
    }
}

fn basic_option(card: CardId, option: u8) -> Action {
    Action::PlayCard {
        card,
        play: CardPlay::Basic {
            die: None,
            gain: None,
            with: None,
            option,
            discard: None,
            mana: None,
        },
    }
}

fn on_village(hand: &[CardId]) -> Sandbox {
    Sandbox {
        figure: Some(VILLAGE),
        hand: hand.to_vec(),
        ..Sandbox::default()
    }
}

/// Pont de glace renforcé (dé bleu) joué depuis `figure`.
fn frost_bridge_from(figure: Hex) -> GameState {
    let frost_bridge = advanced(FrostBridge);
    let state = Sandbox {
        figure: Some(figure),
        hand: vec![frost_bridge, card(March, 1)],
        ..Sandbox::default()
    }
    .state();
    done(&state, strong_option(frost_bridge, BLUE_DIE, 0))
}

// --- Effets en cours (C4) ---

// Spec 052 C4 (écran) : une ligne par effet en cours, sa source puis son texte.
#[test]
fn play_area_lists_modifiers() {
    let catalog = french();
    let path_finding = advanced(PathFinding);
    let intimidate = advanced(Intimidate);
    let state = on_village(&[path_finding, intimidate]).state();
    assert_eq!(screen_of(&state, &catalog).modifiers, None);
    let state = done(&state, basic(path_finding));
    let state = done(&state, basic_option(intimidate, 0));
    let list = screen_of(&state, &catalog)
        .modifiers
        .expect("des effets en cours");
    assert_eq!(list.title, "Effets en cours");
    assert_eq!(
        list.lines,
        [
            "Chemins de traverse : tous les terrains : −1 (min. 2)",
            "Intimidation : réputation −1 à la fin du tour",
        ]
    );
}

// Spec 052 C4 (écran) : les autres sortes d'effet, et une unité pour source.
#[test]
fn modifier_lines_name_each_kind() {
    let catalog = french();
    let lines = |state: &GameState| {
        screen_of(state, &catalog)
            .modifiers
            .map(|list| list.lines)
            .unwrap_or_default()
    };
    // Pont de glace renforcé : lacs et marais à 1, lacs franchissables.
    assert_eq!(
        lines(&frost_bridge_from(VILLAGE)),
        [
            "Pont de glace : Marais, Lac : 1",
            "Pont de glace : Lac franchissable",
        ]
    );
    // Agilité renforcée, Embuscade, Temporisation.
    let agility = advanced(Agility);
    let ambush = advanced(Ambush);
    let steady = advanced(SteadyTempo);
    let state = on_village(&[agility, ambush, steady]).state();
    let state = done(&state, strong_option(agility, WHITE_DIE, 0));
    let state = done(&state, basic(ambush));
    let state = done(&state, basic(steady));
    assert_eq!(
        lines(&state),
        [
            "Agilité : 1 Mouvement en Attaque 1",
            "Agilité : 2 Mouvement en Attaque à distance 1",
            "Embuscade : prochaine carte : Attaque +1 ou Parade +2",
            "Temporisation : sous le deck à la fin du tour",
        ]
    );
    // Diplomatie renforcée, option glace.
    let diplomacy = advanced(Diplomacy);
    let state = on_village(&[diplomacy]).state();
    let state = done(&state, strong_option(diplomacy, WHITE_DIE, 0));
    assert_eq!(lines(&state), ["Diplomatie : Influence en Parade de glace"]);
    // Bûcherons.
    let foresters = UnitCard::new(Unit::Foresters, 1);
    let state = Units {
        owned: vec![foresters],
        ..Units::default()
    }
    .state();
    let state = done(
        &state,
        Action::ActivateUnit {
            enemy: None,
            unit: foresters,
            ability: 0,
            option: 0,
            mana: Vec::new(),
            target: None,
            leadership: false,
        },
    );
    assert_eq!(
        lines(&state),
        ["Bûcherons : Colline, Forêt, Marais : −1 (min. 0)"]
    );
}

// Spec 052 : une carte qui pose un effet en cours montre son texte écrit, pas ses seuls points.
#[test]
fn hand_shows_written_text_of_modifier_cards() {
    let catalog = french();
    let path_finding = advanced(PathFinding);
    let state = on_village(&[path_finding]).state();
    let screen = screen_of(&state, &catalog);
    let held = screen
        .hand
        .iter()
        .find(|held| held.card == path_finding)
        .expect("carte en main");
    assert_eq!(
        held.effect,
        "Mouvement 2 ; tous les terrains coûtent 1 de moins (min. 2)"
    );
}

// --- Coûts et trajets (C17, C43) ---

// Spec 052 C17 (écran) : la forêt à deux pas coûte 2 + 2 avec Chemins de traverse, donc
// atteignable avec 4 points ; elle ne l'est pas sans (2 + 3).
#[test]
fn board_routes_follow_reduced_costs() {
    let catalog = french();
    let path_finding = advanced(PathFinding);
    let march = card(March, 1);
    let state = on_village(&[path_finding, march, card(Stamina, 1)]).state();
    let without = done(&state, basic(march));
    let without = done(&without, basic(card(Stamina, 1)));
    assert!(
        !board(&without, None, &catalog)
            .move_marks
            .iter()
            .any(|mark| mark.hex == WEST_FOREST)
    );
    let state = done(&state, basic(path_finding));
    let state = done(&state, basic(march));
    let screen = board(&state, Some(WEST_FOREST), &catalog);
    let mark = screen
        .move_marks
        .iter()
        .find(|mark| mark.hex == WEST_FOREST)
        .expect("forêt atteignable");
    assert_eq!(mark.cost, 4);
    assert!(mark.reachable);
    assert!(
        screen
            .hover_lines
            .contains(&"Coût de déplacement : 2".to_string())
    );
}

// Spec 052 C14, C27 (écran) : le lac coûte 1, et son infobulle prévient de la retraite forcée.
#[test]
fn lake_hover_warns_unsafe_space() {
    let catalog = french();
    let state = frost_bridge_from(VILLAGE);
    let screen = board(&state, Some(LAKE), &catalog);
    assert!(
        screen
            .move_marks
            .iter()
            .any(|mark| mark.hex == LAKE && mark.cost == 1 && mark.reachable)
    );
    assert!(
        screen
            .hover_lines
            .contains(&"Coût de déplacement : 1".to_string())
    );
    assert!(
        screen.hover_lines.contains(
            &"Case non sécurisée : retraite forcée si vous y finissez le tour".to_string()
        )
    );
    // Une case sécurisée n'a pas l'avertissement.
    let plain = board(&state, Some(PLAIN_W), &catalog);
    assert!(
        !plain
            .hover_lines
            .iter()
            .any(|line| line.starts_with("Case non sécurisée"))
    );
}

// --- Conversions (C28, C29, C31) ---

/// Agilité (renforcée si `strong`) jouée sur le village, puis les Rôdeurs défiés et le combat
/// engagé, Marche en main.
fn agility_fight(strong: bool) -> GameState {
    let agility = advanced(Agility);
    let state = arena(VILLAGE, &[agility, card(March, 1)], &[(prowlers(), ORCS_2)]);
    let play = if strong {
        strong_option(agility, WHITE_DIE, 0)
    } else {
        basic(agility)
    };
    let state = done(&state, play);
    let state = done(&state, challenge(ORCS_2));
    done(&state, Action::StartCombat)
}

// Spec 052 C28 (écran) : un bouton par point converti dans l'attaque en cours.
#[test]
fn attack_shows_move_as_attack_button() {
    let catalog = french();
    let state = skip_to_attack_phase(&agility_fight(false));
    let state = done(&state, declare(&[foe(&state, ORCS_2)]));
    let panel = combat_of(&state, &catalog).in_progress.expect("attaque");
    assert_eq!(panel.conversion, None, "aucun Mouvement en réserve");
    let state = done(&state, basic(card(March, 1)));
    let panel = combat_of(&state, &catalog).in_progress.expect("attaque");
    let button = panel.conversion.expect("bouton de conversion");
    assert_eq!(button.label, "Mouvement en Attaque (2 restant)");
    assert_eq!(
        button.command,
        Command::Play(index_of(&state, &Action::SpendMoveAsAttack))
    );
}

// Spec 052 C29 (écran) : en phase d'attaque à distance, avec Agilité renforcée.
#[test]
fn ranged_attack_shows_two_move_button() {
    let catalog = french();
    let state = agility_fight(true);
    let state = done(&state, declare(&[foe(&state, ORCS_2)]));
    let state = done(&state, basic(card(March, 1)));
    let button = combat_of(&state, &catalog)
        .in_progress
        .and_then(|panel| panel.conversion)
        .expect("bouton de conversion");
    assert_eq!(
        button.label,
        "2 Mouvement en Attaque à distance (2 restant)"
    );
}

// Spec 052 C31 (écran) : Influence en Parade dans la parade en cours.
#[test]
fn block_shows_influence_as_block_button() {
    let catalog = french();
    let diplomacy = advanced(Diplomacy);
    let promise = card(Promise, 1);
    let state = to_block_phase(&fight(
        VILLAGE,
        &[diplomacy, promise],
        &[(prowlers(), ORCS_2)],
        &[ORCS_2],
    ));
    let state = done(&state, basic(diplomacy));
    let state = done(&state, declare_block(foe(&state, ORCS_2)));
    let button = combat_of(&state, &catalog)
        .in_progress
        .and_then(|panel| panel.conversion)
        .expect("bouton de conversion");
    assert_eq!(button.label, "Influence en Parade (2 restante)");
    assert_eq!(
        button.command,
        Command::Play(index_of(&state, &Action::SpendInfluenceAsBlock))
    );
}

// --- Fin du tour (C21, C26, C39, C40) ---

// Spec 052 C26 (écran) : la fenêtre de fin du tour annonce la retraite forcée.
#[test]
fn end_turn_announces_forced_withdrawal() {
    let catalog = french();
    let state = frost_bridge_from(VILLAGE);
    assert_eq!(end_turn_prompt(&state, &catalog).forced_withdrawal, None);
    let state = done(&state, Action::MoveHero { to: LAKE });
    let prompt = end_turn_prompt(&state, &catalog);
    assert_eq!(
        prompt.forced_withdrawal.as_deref(),
        Some("Retraite forcée : 1 case(s), 1 Blessure(s) en main")
    );
}

// Spec 052 C21 (écran) : la clairière et la pioche suivent la main d'après retraite.
#[test]
fn end_turn_follows_hand_after_withdrawal() {
    let catalog = french();
    let state = frost_bridge_from(GLADE);
    let state = done(&state, Action::MoveHero { to: LAKE });
    let prompt = end_turn_prompt(&state, &catalog);
    let glade = prompt.glade.expect("clairière d'arrivée");
    let hand = &glade.options[0];
    assert!(hand.enabled, "la Blessure de la retraite se jette");
    assert!(hand.selected, "de la main par défaut");
    // Main d'après retraite : Marche et la Blessure, puis la Blessure jetée ; la limite de 5
    // fait piocher 4 cartes.
    assert_eq!(prompt.draw, "Vous piocherez 4 carte(s).");
    let mode = ScreenMode::EndingTurn(EndTurnChoice::default());
    assert_eq!(
        EndTurnChoice::default().action(&view(&state, SOLO), &legal_actions(&state, SOLO)),
        Action::EndTurn {
            discard: Vec::new(),
            reclaim: None,
            glade: Some(arcanum_core::WoundPile::Hand),
            steady_tempo: false,
        }
    );
    assert!(
        screen_in(&state, &mode, &catalog)
            .end_turn_prompt
            .expect("mode fin du tour")
            .confirm_enabled
    );
}

// Spec 052 C39 (écran) : la case Temporisation, cochée d'office, absente quand garder n'est pas
// légal.
#[test]
fn end_turn_offers_to_keep_steady_tempo() {
    let catalog = french();
    let steady = advanced(SteadyTempo);
    let state = done(&on_village(&[steady]).state(), basic(steady));
    assert_eq!(
        end_turn_prompt(&state, &catalog).steady_tempo,
        Some(DiscardToggle {
            label: "Garder Temporisation sous le deck".to_string(),
            checked: true,
            command: Command::ToggleSteadyTempo,
        })
    );
    // Deck vide : garder n'est pas légal.
    let empty = done(&state_with_deck(vec![steady], 1), basic(steady));
    assert_eq!(end_turn_prompt(&empty, &catalog).steady_tempo, None);
    assert!(matches!(
        EndTurnChoice::default().action(&view(&empty, SOLO), &legal_actions(&empty, SOLO)),
        Action::EndTurn {
            steady_tempo: false,
            ..
        }
    ));
}

// Spec 052 C40 (écran) : renforcée, « sur le deck ».
#[test]
fn strong_steady_tempo_label() {
    let catalog = french();
    let steady = advanced(SteadyTempo);
    let state = done(
        &on_village(&[steady]).state(),
        strong_option(steady, BLUE_DIE, 0),
    );
    assert_eq!(
        end_turn_prompt(&state, &catalog)
            .steady_tempo
            .map(|toggle| toggle.label)
            .as_deref(),
        Some("Garder Temporisation sur le deck")
    );
}

// Spec 052 C39 (écran) : la case cochée garde la carte, décochée la défausse.
#[test]
fn steady_tempo_toggle_drives_end_turn() {
    let steady = advanced(SteadyTempo);
    let sandbox = on_village(&[steady, card(March, 1)]);
    for (toggles, kept) in [(0, true), (1, false), (2, true)] {
        let mut session = session_of(sandbox.events());
        play(&mut session, &basic(steady));
        session.command(Command::OpenEndTurn);
        for _ in 0..toggles {
            session.command(Command::ToggleSteadyTempo);
        }
        let catalog = french();
        let toggle = session
            .screen(&catalog)
            .end_turn_prompt
            .and_then(|prompt| prompt.steady_tempo)
            .expect("case Temporisation");
        assert_eq!(toggle.checked, kept);
        session.command(Command::ConfirmEndTurn);
        session.run_frame();
        let state = session.runner().state();
        assert_eq!(state.deck(SOLO).last() == Some(&steady), kept);
        assert_eq!(state.discard_pile(SOLO).contains(&steady), !kept);
    }
}

// Spec 043 C29 et spec 052 C21 (issue privée 217) : la pioche annoncée suit la limite de main d'une
// forteresse possédée, voisine de la figurine (Compendium, *End of Turn* S8).
#[test]
fn end_turn_draw_follows_keep_hand_limit() {
    let catalog = french();
    let state = Fortified {
        figure: NEAR_KEEP,
        conquered: vec![KEEP],
        ..Fortified::default()
    }
    .state();
    let state = done(&state, basic(card(March, 1)));
    // Main de 4 après la Marche : limite 5 + 1 forteresse = 2 cartes piochées.
    assert_eq!(
        end_turn_prompt(&state, &catalog).draw,
        "Vous piocherez 2 carte(s)."
    );
}
