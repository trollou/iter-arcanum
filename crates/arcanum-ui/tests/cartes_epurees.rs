//! Scénarios de la spec #11 (cartes en main épurées) : la vue de la carte de la main, son texte
//! complet derrière « i », le titre du bloc renforcé, les pastilles de mana des boutons. Aucune
//! règle du livret : c'est de l'affichage (rappel : MKUE p. 4, *Deed Cards* 3.a à 3.d).

use arcanum_core::{
    AdvancedAction, Artifact, BasicAction, CardId, DeedCard, GameState, Spell, legal_actions, view,
};
use arcanum_i18n::{Catalog, Language};
use arcanum_ui::{
    CardInfo, DiscardChoice, GameScreen, HandCard, ScreenMode, ScreenPhase, game_screen,
    repository_assets_dir,
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

/// La carte telle que la main la montre, seule en main avec une Marche.
fn held_alone(card: CardId, catalog: &Catalog) -> HandCard {
    let state = state_with(&[card, self::card(March, 1)]);
    held(&screen_in(&state, &ScreenMode::Normal, catalog), card)
}

#[track_caller]
fn held(screen: &GameScreen, card: CardId) -> HandCard {
    screen
        .hand
        .iter()
        .find(|held| held.card == card)
        .cloned()
        .unwrap_or_else(|| panic!("{card:?} absente de la main"))
}

fn spell(spell: Spell) -> CardId {
    CardId::new(DeedCard::Spell(spell), 1)
}

fn info(name: &str, basic: &str, strong_title: &str, strong: &str) -> Option<CardInfo> {
    Some(CardInfo {
        name: name.into(),
        basic: basic.into(),
        strong_title: strong_title.into(),
        strong: strong.into(),
    })
}

// --- Texte complet derrière « i » ---

// Spec #11 C1 : une action de base, ses textes calculés.
#[test]
fn basic_action_info_holds_computed_text() {
    let catalog = french();
    let rage = held_alone(card(Rage, 1), &catalog);
    assert!(!rage.show_text);
    assert_eq!(
        rage.info,
        info("Rage", "Attaque 2 ou Parade 2", "Renforcé", "Attaque 4")
    );
}

// Spec #11 C2 (#12) : le texte écrit, y compris ce qui ne vaut qu'à plusieurs joueurs.
#[test]
fn spell_info_holds_written_text_for_other_players() {
    let catalog = french();
    let mind_read = held_alone(spell(Spell::MindRead), &catalog);
    assert_eq!(
        mind_read.info,
        info(
            "Lecture d'esprit",
            &catalog.text("card.mind_read.basic_text"),
            "Vol d'esprit",
            &catalog.text("card.mind_read.strong_text"),
        )
    );
    assert!(
        mind_read
            .info
            .as_ref()
            .is_some_and(|info| info.basic.contains("les autres défaussent"))
    );
    // En solo, seul le cristal joue : les quatre cristaux (spec 057).
    assert_eq!(mind_read.basic_effects.len(), 4);
}

// Spec #11 C3 : une action avancée, son texte écrit et non le texte calculé.
#[test]
fn advanced_action_info_holds_written_text() {
    let catalog = french();
    let pure_magic = held_alone(advanced(AdvancedAction::PureMagic), &catalog);
    let info = pure_magic.info.expect("Magie pure a un texte complet");
    assert_eq!(info.basic, catalog.text("card.pure_magic.basic_text"));
    assert_eq!(info.strong, catalog.text("card.pure_magic.strong_text"));
    assert_eq!(info.strong_title, "Renforcé");
}

// Spec #11 C4 (spec 077, C1) : une Blessure n'a pas de texte.
#[test]
fn wound_has_no_info() {
    let catalog = french();
    let wound = held_alone(wound(1), &catalog);
    assert_eq!(wound.info, None);
    assert!(!wound.show_text);
    assert!(wound.effect.is_empty());
}

// Spec #11 C5 (Q1) : un artefact écrit son texte sur lui, sans « i ».
#[test]
fn artifact_keeps_its_text() {
    let catalog = french();
    let ring = held_alone(artifact(Artifact::RubyRing), &catalog);
    assert_eq!(ring.info, None);
    assert!(ring.show_text);
    assert_eq!(ring.effect, "Jeton et cristal rouges ; Gloire +1");
    assert_eq!(
        ring.strong_text,
        "Mana rouge et noir illimité ce tour ; Gloire +1 par sort rouge lancé"
    );
}

// Spec #11 C6 : en mode choix, la carte se lit avant d'être défaussée.
#[test]
fn info_stays_in_choice_mode() {
    let catalog = french();
    let rage = card(Rage, 1);
    let improvisation = card(Improvisation, 1);
    let state = state_with(&[improvisation, rage]);
    let choosing = ScreenMode::ChoosingDiscard(DiscardChoice {
        card: improvisation,
        option: 0,
    });
    let rage = held(&screen_in(&state, &choosing, &catalog), rage);
    assert!(rage.basic_effects.is_empty());
    assert_eq!(
        rage.info,
        info("Rage", "Attaque 2 ou Parade 2", "Renforcé", "Attaque 4")
    );
}
