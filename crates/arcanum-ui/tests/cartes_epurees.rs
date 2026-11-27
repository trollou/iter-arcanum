//! Scénarios de la spec #11 (cartes en main épurées) : la vue de la carte de la main, son texte
//! complet derrière « i », le titre du bloc renforcé, les pastilles de mana des boutons. Aucune
//! règle du livret : c'est de l'affichage (rappel : MKUE p. 4, *Deed Cards* 3.a à 3.d).

use arcanum_core::{
    AdvancedAction, Artifact, BasicAction, BasicColor, CardId, DeedCard, GameState, ManaColor,
    Spell, Unit, legal_actions, view,
};
use arcanum_i18n::{Catalog, Language};
use arcanum_ui::{
    CardInfo, DiscardChoice, EffectFace, GameScreen, HandCard, ScreenMode, ScreenPhase,
    game_screen, repository_assets_dir,
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

// Spec #11 C1 (une action de base, ses textes calculés) : remplacé par la spec #68, C2
// (`basic_action_info_shows_its_full_text`).

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
            .is_some_and(|info| info.basic.contains("Chaque autre joueur défausse"))
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
    assert_eq!(ring.effect, catalog.text("card.ruby_ring.basic_text"));
    assert_eq!(ring.strong_text, catalog.text("card.ruby_ring.strong_text"));
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
        info(
            "Rage",
            &catalog.text("card.rage.basic_text"),
            "Renforcé",
            &catalog.text("card.rage.strong_text"),
        )
    );
}

// --- Titre du bloc renforcé ---

// Spec #11 C7 : le titre et ses pastilles.
#[test]
fn strong_heading_names_its_mana() {
    let catalog = french();
    let rage = held_alone(card(Rage, 1), &catalog);
    assert_eq!(rage.strong_heading, "Renforcé");
    assert_eq!(rage.strong_colors, [ManaColor::Basic(BasicColor::Red)]);
    let chill = held_alone(spell(Spell::Chill), &catalog);
    assert_eq!(chill.strong_heading, "Frisson fatal");
    assert_eq!(
        chill.strong_colors,
        [ManaColor::Basic(BasicColor::Blue), ManaColor::Black]
    );
    let ring = held_alone(artifact(Artifact::RubyRing), &catalog);
    assert_eq!(ring.strong_heading, "si détruit");
    assert!(ring.strong_colors.is_empty());
}

// --- Le mana de la carte n'est plus répété ---

// Spec #11 C8 : les boutons renforcés d'une action n'ont plus la pastille de la carte.
#[test]
fn strong_buttons_omit_card_mana() {
    let catalog = french();
    for action in [Rage, March, Tranquility] {
        let held = held_alone(card(action, 1), &catalog);
        assert!(!held.strong_effects.is_empty(), "{action:?}");
        assert!(
            held.strong_effects
                .iter()
                .all(|button| button.color.is_none()),
            "{action:?}"
        );
    }
}

// Spec #11 C10 : un mana en plus garde sa pastille, même de la couleur de la carte.
#[test]
fn extra_mana_keeps_its_swatch() {
    let catalog = french();
    let pure_magic = held_alone(advanced(AdvancedAction::PureMagic), &catalog);
    let colors: Vec<Option<ManaColor>> = pure_magic.strong_effects[..4]
        .iter()
        .map(|button| button.color)
        .collect();
    let basic = |color| Some(ManaColor::Basic(color));
    assert_eq!(
        colors,
        [
            basic(BasicColor::Green),
            basic(BasicColor::White),
            basic(BasicColor::Blue),
            basic(BasicColor::Red),
        ]
    );
}

// --- Options payantes ---

// Spec #11 C12 : le bouton dit l'effet, la pastille le paiement ; l'infobulle garde tout.
#[test]
fn paid_option_face_omits_payment() {
    let catalog = french();
    let talent = held_alone(advanced(AdvancedAction::MagicTalent), &catalog);
    let red = talent
        .strong_effects
        .iter()
        .find(|button| button.color == Some(ManaColor::Basic(BasicColor::Red)))
        .expect("option renforcée rouge");
    assert_eq!(
        red.face,
        EffectFace::Text("un sort <red>rouge</red> de l'offre".into())
    );
    // Puis, à la ligne, la raison du grisé (pas de mana bleu).
    assert_eq!(
        red.tooltip.lines().next(),
        Some("Renforcer : Payer <red>rouge</red> : un sort <red>rouge</red> de l'offre")
    );
}

// --- Unités ---

// Spec #11 C14 : le texte de toute capacité à mana commence par son mana entre parenthèses, que
// le bouton retire (C13).
#[test]
fn unit_mana_ability_texts_start_with_mana() {
    for language in ["fr", "en"] {
        let catalog = Catalog::load(
            &repository_assets_dir().join("i18n"),
            &Language(language.into()),
        )
        .expect("catalogue du dépôt");
        for unit in Unit::ALL {
            for (index, ability) in unit.definition().abilities.iter().enumerate() {
                if ability.mana.is_none() {
                    continue;
                }
                let text = catalog.text(&unit.ability_key(index));
                let prefix = text
                    .strip_prefix('(')
                    .and_then(|rest| rest.split_once(") "));
                assert!(prefix.is_some(), "{language} {unit:?} {index} : {text}");
            }
        }
        assert_eq!(catalog.gaps(), vec![], "langue {language}");
    }
}

// Spec #68 C2 : le « i » d'une action de base montre ses textes complets, pas les effets calculés.
#[test]
fn basic_action_info_shows_its_full_text() {
    let catalog = french();
    let concentration = held_alone(card(Concentration, 1), &catalog);
    let info = concentration
        .info
        .expect("Concentration a un texte complet");
    assert_eq!(info.basic, catalog.text("card.concentration.basic_text"));
    assert_eq!(info.strong, catalog.text("card.concentration.strong_text"));
    assert!(!info.basic.starts_with('⟦'), "{}", info.basic);
}

// Spec #68 C8 : le recto d'un artefact montre son texte complet.
#[test]
fn artifact_face_shows_full_text() {
    let catalog = french();
    let ring = held_alone(artifact(Artifact::RubyRing), &catalog);
    assert_eq!(ring.effect, catalog.text("card.ruby_ring.basic_text"));
    assert_eq!(ring.strong_text, catalog.text("card.ruby_ring.strong_text"));
}
