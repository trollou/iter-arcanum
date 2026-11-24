//! Scénarios de la spec 034 côté affichage (Blessures et soins).

use arcanum_core::UndoStatus;
use arcanum_core::{Action, BasicAction, CardId, Enemy, GameState, Hex, legal_actions, view};
use arcanum_i18n::{Catalog, Language};
use arcanum_ui::{
    CombatChoice, EndTurnChoice, GameScreen, HandCard, ScreenMode, ScreenPhase, combat_screen,
    game_screen, repository_assets_dir,
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

fn held(screen: &GameScreen, card: CardId) -> &HandCard {
    screen
        .hand
        .iter()
        .find(|held| held.card == card)
        .expect("carte en main")
}

// Spec 034 C31
#[test]
fn end_turn_mode_offers_wounds_when_resting() {
    let catalog = french();
    let (march, rage) = (card(March, 1), card(Rage, 1));
    let state = state_with(&[wound(1), wound(2), march, rage]);
    let screen = screen_of(&state, &ending(&[]), &catalog);
    for wound in [wound(1), wound(2)] {
        assert!(held(&screen, wound).discard_toggle.is_some(), "{wound:?}");
    }
    let prompt = screen.end_turn_prompt.expect("bandeau");
    assert_eq!(prompt.no_rest, None);

    let state = done(&state, basic(march));
    let state = done(&state, Action::MoveHero { to: Hex::new(1, 0) });
    let screen = screen_of(&state, &ending(&[]), &catalog);
    for wound in [wound(1), wound(2)] {
        assert!(held(&screen, wound).discard_toggle.is_none(), "{wound:?}");
    }
    let prompt = screen.end_turn_prompt.expect("bandeau");
    assert_eq!(
        prompt.no_rest.as_deref(),
        Some(
            "Pas de repos : vous vous êtes déplacé ou avez combattu ce tour, les Blessures restent en main."
        )
    );
}

// Spec 034 C32
#[test]
fn rest_banner_follows_checked_cards() {
    let catalog = french();
    let rage = card(Rage, 1);
    let state = state_with(&[wound(1), wound(2), card(March, 1), rage]);
    let standard = "Repos standard : défaussez une carte qui n'est pas une Blessure, et autant de \
                    Blessures que vous voulez.";
    let prompt = screen_of(&state, &ending(&[rage, wound(2)]), &catalog)
        .end_turn_prompt
        .expect("bandeau");
    assert_eq!(prompt.text, standard);
    assert!(prompt.confirm_enabled);
    let prompt = screen_of(&state, &ending(&[wound(2)]), &catalog)
        .end_turn_prompt
        .expect("bandeau");
    assert_eq!(prompt.text, standard);
    assert!(!prompt.confirm_enabled);

    let state = state_with(&[wound(1), wound(2), wound(3)]);
    let prompt = screen_of(&state, &ending(&[wound(3)]), &catalog)
        .end_turn_prompt
        .expect("bandeau");
    assert_eq!(
        prompt.text,
        "Repos prolongé : votre main n'a que des Blessures, défaussez-en une."
    );
    assert!(prompt.confirm_enabled);
    let prompt = screen_of(&state, &ending(&[wound(1), wound(3)]), &catalog)
        .end_turn_prompt
        .expect("bandeau");
    assert!(!prompt.confirm_enabled);
}

// Spec 034 : Tranquillité affiche ses options, grisées en combat avec leur raison.
#[test]
fn healing_card_shows_its_options_and_combat_note() {
    let catalog = french();
    let tranquility = card(Tranquility, 1);
    let state = state_with(&[wound(1), tranquility]);
    let screen = screen_of(&state, &ScreenMode::Normal, &catalog);
    let shown = held(&screen, tranquility);
    assert_eq!(shown.effect, "Soin 1 ou Piocher 1 carte(s)");
    assert_eq!(shown.plays.len(), 2);
    assert_eq!(shown.combat_note, None);

    let state = fight(
        VILLAGE,
        &[tranquility, card(Rage, 1)],
        &[(prowlers(), ORCS_2)],
        &[ORCS_2],
    );
    let screen = screen_of(&state, &ScreenMode::Normal, &catalog);
    let shown = held(&screen, tranquility);
    assert!(shown.plays.is_empty());
    assert_eq!(shown.combat_note.as_deref(), Some("Soin : pas en combat"));
}

// Spec 034 C33
#[test]
fn history_shows_knock_out() {
    let catalog = french();
    let hand = [card(March, 1), card(March, 2), card(Stamina, 1)];
    let state = fight(
        PLAIN_W,
        &hand,
        &[(token(Enemy::Minotaur, 1), ORCS_2)],
        &[ORCS_2],
    );
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
        Some("Assommé : 5 Blessures reçues dans ce combat.")
    );
    let history = &screen.enemies[0].history;
    assert_eq!(
        history.last().map(String::as_str),
        Some("Assommé : 3 carte(s) défaussée(s) de la main")
    );
}

// Spec 034 C33b (issue privée 85) — MKUE p. 6, 5.a : héros assommé sans carte jouée, les cartes défaussées
// comptent pour le tour minimum ; « Finir le tour » sans défausse, sans demander d'en défausser une.
#[test]
fn knocked_out_hero_is_not_asked_to_discard() {
    let catalog = french();
    let hand = [card(March, 1), card(March, 2), card(Stamina, 1)];
    let state = fight(
        PLAIN_W,
        &hand,
        &[(token(Enemy::Minotaur, 1), ORCS_2)],
        &[ORCS_2],
    );
    let state = done(&state, Action::NextCombatPhase);
    let state = done(&state, Action::NextCombatPhase);
    let state = done(&state, Action::EndCombat);
    let screen = screen_of(&state, &ending(&[]), &catalog);
    let prompt = screen.end_turn_prompt.expect("bandeau de fin du tour");
    assert_eq!(prompt.must_discard, None);
    assert!(prompt.confirm_enabled);
}
