//! Scénarios de la spec 035 côté affichage (Gloire et niveaux).

use arcanum_core::UndoStatus;
use arcanum_core::{
    Action, BasicAction, CardId, Enemy, EnemyToken, GameState, legal_actions, view,
};
use arcanum_i18n::{Catalog, Language};
use arcanum_ui::{
    CombatChoice, EndTurnChoice, GameScreen, ScreenMode, ScreenPhase, combat_screen, game_screen,
    repository_assets_dir,
};

// L'arène des tests de combat d'arcanum-core, partagée plutôt que recopiée.
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

fn ending(discard: &[CardId]) -> ScreenMode {
    ScreenMode::EndingTurn(EndTurnChoice {
        discard: discard.to_vec(),
        reclaim: None,
        glade: None,
        steady_tempo: None,
    })
}

fn end_turn(discard: &[CardId]) -> Action {
    Action::EndTurn {
        discard: discard.to_vec(),
        reclaim: None,
        glade: None,
        steady_tempo: false,
    }
}

// Spec 035 C15
#[test]
fn panel_shows_fame_level_and_stats() {
    let catalog = french();
    let state = arena_at(VILLAGE, &[card(March, 1)], &[], 9, 3);
    let screen = screen_of(&state, &ScreenMode::Normal, &catalog);
    // Spec 073, C23 : le niveau, puis la Gloire et le seuil du suivant (issue privée 259 : ce qui est
    // dessiné). Spec 071, C5 : icône Armure, lue ici par son texte de repli.
    assert_eq!(screen.player_level, "Niveau 3");
    assert_eq!(screen.player_fame.value, "9 / 15");
    assert_eq!(
        screen.level_stats.fallback_text(),
        "Armure 3 · Limite de main 5 · Jetons de commandement 2"
    );
    assert_eq!(screen.fame_pending, None);

    let state = arena_at(VILLAGE, &[card(March, 1)], &[], 104, 10);
    let screen = screen_of(&state, &ScreenMode::Normal, &catalog);
    assert_eq!(screen.player_level, "Niveau 10");
    assert_eq!(screen.player_fame.value, "104");
    assert_eq!(
        screen.level_stats.fallback_text(),
        "Armure 4 · Limite de main 7 · Jetons de commandement 5"
    );
}

// Spec 035 C16
#[test]
fn pending_level_shown_and_used_for_draw_count() {
    let catalog = french();
    let rage = card(Rage, 1);
    let state = arena_at(VILLAGE, &[rage, card(March, 1), card(March, 2)], &[], 26, 4);
    let state = done(&state, sideways(rage));
    let screen = screen_of(&state, &ScreenMode::Normal, &catalog);
    assert_eq!(
        screen.fame_pending.as_deref(),
        Some("Niveau 5 à la fin du tour")
    );
    // Deux cartes gardées, limite 6 au niveau 5 : quatre cartes piochées.
    let prompt = screen_of(&state, &ending(&[]), &catalog)
        .end_turn_prompt
        .expect("bandeau de fin du tour");
    assert_eq!(prompt.draw, "Vous piocherez 4 carte(s).");
}

// Spec 035 C17
#[test]
fn level_up_banner_at_next_turn() {
    let catalog = french();
    let (rage, march) = (card(Rage, 1), card(March, 1));
    let state = arena_at(VILLAGE, &[rage, march, card(March, 2)], &[], 8, 2);
    let state = done(&state, sideways(rage));
    let screen = screen_of(&state, &ScreenMode::Normal, &catalog);
    assert!(screen.level_gained.is_empty());
    let state = done(&state, end_turn(&[]));
    let screen = screen_of(&state, &ScreenMode::Normal, &catalog);
    assert_eq!(
        screen.level_gained,
        ["Niveau 3 : armure 3, limite de main 5, un jeton de commandement de plus"]
    );
    // Il disparaît à la première décision du tour.
    let state = done(&state, sideways(march));
    let screen = screen_of(&state, &ScreenMode::Normal, &catalog);
    assert!(screen.level_gained.is_empty());

    // Deux niveaux au même tour, dont un pair : un bandeau par niveau.
    let state = arena_at(VILLAGE, &[rage, march, card(March, 2)], &[], 16, 2);
    let state = done(&state, sideways(rage));
    let state = done(&state, end_turn(&[]));
    let screen = screen_of(&state, &ScreenMode::Normal, &catalog);
    assert_eq!(
        screen.level_gained,
        [
            "Niveau 3 : armure 3, limite de main 5, un jeton de commandement de plus",
            "Niveau 4 : une compétence et une action avancée gagnées",
        ]
    );
}

// Spec 035, Ce que le niveau change ailleurs : les enjeux de la parade suivent l'armure, et
// l'assommé la limite de main du niveau.
#[test]
fn combat_uses_level_armor_and_hand_limit() {
    let catalog = french();
    let hand = [card(March, 1), card(March, 2), card(Stamina, 1)];
    let state = arena_at(
        PLAIN_W,
        &hand,
        &[(EnemyToken::new(Enemy::Minotaur, 1), ORCS_2)],
        8,
        3,
    );
    let state = done(&state, challenge(ORCS_2));
    let state = done(&state, Action::StartCombat);
    let state = done(&state, Action::NextCombatPhase);
    let legal = legal_actions(&state, SOLO);
    let screen = combat_screen(
        &view(&state, SOLO),
        &legal,
        &CombatChoice::default(),
        UndoStatus::Possible,
        &catalog,
    )
    .expect("écran de combat");
    let stakes = screen.enemies[0].stakes.join(" / ");
    assert!(stakes.ends_with("4 blessure(s)"), "{stakes}");

    // Niveau 5 (limite 6) : Minotaure (4) et Épéistes (2) assomment, « 6 Blessures ».
    let placed = [
        (EnemyToken::new(Enemy::Minotaur, 1), ORCS_2),
        (EnemyToken::new(Enemy::Swordsmen, 1), HILL),
    ];
    let state = arena_at(PLAIN_W, &hand, &placed, 24, 5);
    let state = done(&state, challenge(ORCS_2));
    let state = done(&state, challenge(HILL));
    let state = done(&state, Action::StartCombat);
    let state = done(&state, Action::NextCombatPhase);
    let state = done(&state, Action::NextCombatPhase);
    let legal = legal_actions(&state, SOLO);
    let screen = combat_screen(
        &view(&state, SOLO),
        &legal,
        &CombatChoice::default(),
        UndoStatus::Possible,
        &catalog,
    )
    .expect("écran de combat");
    assert_eq!(
        screen.knocked_out.as_deref(),
        Some("Assommé : 6 Blessures reçues dans ce combat.")
    );
}
