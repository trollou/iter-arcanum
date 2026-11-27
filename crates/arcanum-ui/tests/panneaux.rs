//! Scénarios de la spec 073 (panneau du bas et panneau de droite) : disposition des cartes,
//! fonction pure sans egui (C9 à C12), puis l'écran calculé (C7, C8, C13 à C37). Aucune règle
//! du livret : c'est de l'affichage (rappels : MKUE p. 4, *Deed Cards* 3.a à 3.e ; p. 3,
//! *Play Area*).

use arcanum_core::{
    Action, AttackKind, BasicAction, BasicColor, CardId, CardPlay, Effect, Element, Event,
    GameState, HealingTarget, ManaColor, SidewaysEffect, Unit, UnitCard, apply, legal_actions,
    view,
};
use arcanum_i18n::{Catalog, Language};
use arcanum_ui::{
    BandUv, CARD_GAP, CARD_WIDTH, Command, DiscardChoice, EffectButton, EffectFace, EndTurnChoice,
    GameScreen, GameSession, HandCard, Icon, SKILLS_WIDTH, ScreenMode, ScreenPhase, Shortcut,
    attack_icon, band_uv, block_icon, bottom_section_widths, card_height, fan_layout, game_screen,
    repository_assets_dir, row_width, section_gap, section_widths,
};

// L'arène des tests d'arcanum-core, partagée plutôt que recopiée.
#[path = "../../arcanum-core/tests/arene/mod.rs"]
mod arene;
use arene::*;

mod support;
use support::{first_reconnaissance_config, labels_and_commands, started_session};

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

fn screen_of(state: &GameState, catalog: &Catalog) -> GameScreen {
    screen_in(state, &ScreenMode::Normal, catalog)
}

/// L'index de cette action parmi les actions légales.
#[track_caller]
fn index_of(state: &GameState, wanted: &Action) -> usize {
    legal_actions(state, SOLO)
        .iter()
        .position(|action| action == wanted)
        .unwrap_or_else(|| panic!("{wanted:?} doit être légale"))
}

#[track_caller]
fn held(screen: &GameScreen, card: CardId) -> &HandCard {
    screen
        .hand
        .iter()
        .find(|held| held.card == card)
        .unwrap_or_else(|| panic!("{card:?} absente de la main"))
}

/// Un bouton d'effet résumé : son visage (icône et valeur, ou texte), sa pastille, actif ou non.
#[derive(Debug, PartialEq)]
enum Face {
    Icon(Icon, &'static str),
    Crystal(ManaColor),
    Text(&'static str),
}

fn summary(buttons: &[EffectButton]) -> Vec<(String, Option<ManaColor>, bool)> {
    buttons
        .iter()
        .map(|button| {
            let face = match &button.face {
                EffectFace::Icon(value) => format!("{:?} {}", value.icon, value.value),
                EffectFace::Crystal { color, .. } => format!("{color:?}"),
                EffectFace::Text(text) => text.clone(),
            };
            (face, button.color, button.command.is_some())
        })
        .collect()
}

fn expected(buttons: &[(Face, Option<ManaColor>, bool)]) -> Vec<(String, Option<ManaColor>, bool)> {
    buttons
        .iter()
        .map(|(face, color, enabled)| {
            let face = match face {
                Face::Icon(icon, value) => format!("{icon:?} {value}"),
                Face::Crystal(color) => format!("{color:?}"),
                Face::Text(text) => (*text).to_string(),
            };
            (face, *color, *enabled)
        })
        .collect()
}

fn sword() -> Icon {
    attack_icon(AttackKind::Melee, Element::Physical)
}

fn shield() -> Icon {
    block_icon(Element::Physical)
}

const RED: Option<ManaColor> = Some(ManaColor::Basic(BasicColor::Red));
const BLUE: Option<ManaColor> = Some(ManaColor::Basic(BasicColor::Blue));

/// Les abscisses arrondies au point, comme la spec les donne.
fn rounded(positions: &[f32]) -> Vec<i32> {
    positions.iter().map(|x| x.round() as i32).collect()
}

// --- Largeur des sections ---

// Spec 073 C9, nombres de la spec 077 C3 (cartes de 216 points).
#[test]
fn section_widths_share_space() {
    assert_eq!(row_width(2), 440.0);
    assert_eq!(row_width(5), 1112.0);
    assert_eq!(row_width(0), 0.0);
    assert_eq!(row_width(1), CARD_WIDTH);
    // Tout tient : chaque section à sa largeur naturelle.
    assert_eq!(
        section_widths(&[row_width(2), row_width(5)], 1600.0),
        vec![440.0, 1112.0]
    );
    // 3 unités (664) et 7 cartes (1 560) dans 1 228 points : la part des unités (367) est sous
    // 400, elles gardent 400 et la main prend le reste.
    let widths = section_widths(&[row_width(3), row_width(7)], 1228.0);
    assert_eq!(rounded(&widths), vec![400, 828]);
    // Assez de place pour que chacune garde sa part en proportion : 664 / 2 224 de 1 800.
    let widths = section_widths(&[row_width(3), row_width(7)], 1800.0);
    assert_eq!(rounded(&widths), vec![537, 1263]);
    // Une section plus petite que 400 garde sa largeur naturelle (une seule unité).
    let widths = section_widths(&[row_width(1), row_width(9)], 1000.0);
    assert_eq!(rounded(&widths), vec![216, 784]);
}

// Issue privée 295 : la place prise par les sections et leurs écarts, espacements d'egui compris.
fn occupied(widths: &[f32], spacing: f32) -> f32 {
    widths.iter().sum::<f32>() + (widths.len() - 1) as f32 * section_gap(spacing)
}

// Spec 051 et issue privée 295 : 3 unités, 7 cartes en main (2 Blessures reçues en combat) et la
// section Compétences tiennent dans un panneau de 1 280 points, sans déborder à droite.
#[test]
fn bottom_sections_with_skills_fit_in_panel() {
    let spacing = 8.0;
    let widths = bottom_section_widths(row_width(3), row_width(7), true, 1280.0, spacing);
    assert_eq!(widths.len(), 3);
    assert_eq!(widths[2], SKILLS_WIDTH);
    assert!(occupied(&widths, spacing) <= 1280.0, "{widths:?}");
}

// Spec 073 C9 : sans compétence, Unités et Main se partagent tout le panneau, écart compris.
#[test]
fn bottom_sections_without_skills_fit_in_panel() {
    let spacing = 8.0;
    let widths = bottom_section_widths(row_width(3), row_width(7), false, 1280.0, spacing);
    assert_eq!(widths.len(), 2);
    assert!(occupied(&widths, spacing) <= 1280.0, "{widths:?}");
}

// Spec 073 C9 : quand tout tient, chaque section garde sa largeur naturelle.
#[test]
fn bottom_sections_keep_natural_widths_when_room() {
    let widths = bottom_section_widths(row_width(2), row_width(5), true, 2400.0, 8.0);
    assert_eq!(widths, vec![440.0, 1112.0, SKILLS_WIDTH]);
    assert_eq!(section_gap(8.0), 32.0);
}

// --- Chevauchement ---

// Spec 073 C10, nombres de la spec 077 C4.
#[test]
fn fan_layout_overlaps_when_short() {
    let layout = fan_layout(7, 600.0, None);
    assert_eq!(
        rounded(&layout.positions),
        vec![0, 64, 128, 192, 256, 320, 384]
    );
    // La suivante par-dessus la précédente.
    assert_eq!(layout.draw_order, vec![0, 1, 2, 3, 4, 5, 6]);
    let layout = fan_layout(5, 1112.0, None);
    assert_eq!(rounded(&layout.positions), vec![0, 224, 448, 672, 896]);
    assert_eq!(CARD_WIDTH + CARD_GAP, 224.0);
    assert_eq!(fan_layout(0, 600.0, None).positions, Vec::<f32>::new());
    assert_eq!(fan_layout(1, 100.0, None).positions, vec![0.0]);
}

// Spec 073 C11, nombres de la spec 077 C5.
#[test]
fn fan_layout_hovered_card_is_whole() {
    let layout = fan_layout(7, 600.0, Some(3));
    assert_eq!(
        rounded(&layout.positions),
        vec![0, 34, 67, 101, 317, 350, 384]
    );
    // La carte survolée par-dessus toutes les autres.
    assert_eq!(layout.draw_order.last(), Some(&3));
    assert_eq!(layout.draw_order.len(), 7);
    // Sans chevauchement, le survol ne change rien.
    let layout = fan_layout(5, 1112.0, Some(1));
    assert_eq!(rounded(&layout.positions), vec![0, 224, 448, 672, 896]);
}

// Spec 073 C12, nombres de la spec 077 C6.
#[test]
fn fan_layout_last_and_too_narrow() {
    assert_eq!(
        fan_layout(7, 600.0, Some(6)).positions,
        fan_layout(7, 600.0, None).positions
    );
    let layout = fan_layout(4, 300.0, Some(1));
    assert_eq!(rounded(&layout.positions), vec![0, 0, 84, 84]);
    assert_eq!(layout.draw_order.last(), Some(&1));
}

// Spec 073, « Carte survolée » : celle du dessus sous le pointeur, la survolée comptant comme celle
// du dessus.
// Spec 073 C12 bis (remarque de Mathieu : Cristallisation tronquée), nombres de la spec 077 C7.
#[test]
fn hovered_card_grows_to_its_content() {
    assert_eq!(card_height(150.0, 800.0), 240.0);
    assert_eq!(card_height(300.0, 800.0), 300.0);
    assert_eq!(card_height(300.0, 260.0), 260.0);
}

fn uv_permille(uv: BandUv) -> (f32, f32, f32, f32) {
    let r = |x: f32| (x * 1000.0).round();
    (r(uv.left), r(uv.top), r(uv.right), r(uv.bottom))
}

// Spec #9 C1 : une image en bande de 404 × 168 remplit la bande de 202 × 84 sans recadrage.
#[test]
fn band_keeps_whole_banner_image() {
    assert_eq!(
        uv_permille(band_uv(202.0, 84.0, 404.0, 168.0)),
        (0.0, 0.0, 1000.0, 1000.0)
    );
}

// Spec #9 C2 (spec 077 C2) : une image carrée garde son milieu en hauteur, toute sa largeur.
#[test]
fn band_crops_square_image_as_before() {
    assert_eq!(
        uv_permille(band_uv(202.0, 84.0, 256.0, 256.0)),
        (0.0, 292.0, 1000.0, 708.0)
    );
}

// Spec #9 C3 : une image en bande dans un carré garde toute sa hauteur et le milieu de sa largeur.
#[test]
fn square_box_crops_banner_sides() {
    assert_eq!(
        uv_permille(band_uv(64.0, 64.0, 404.0, 168.0)),
        (292.0, 0.0, 708.0, 1000.0)
    );
}

// Spec #9 C4 : une taille nulle ne recadre rien.
#[test]
fn band_uv_degenerate_sizes() {
    let whole = (0.0, 0.0, 1000.0, 1000.0);
    assert_eq!(uv_permille(band_uv(0.0, 84.0, 404.0, 168.0)), whole);
    assert_eq!(uv_permille(band_uv(202.0, 0.0, 404.0, 168.0)), whole);
    assert_eq!(uv_permille(band_uv(202.0, 84.0, 0.0, 168.0)), whole);
    assert_eq!(uv_permille(band_uv(202.0, 84.0, 404.0, 0.0)), whole);
}

// Spec 077 C6 : avec des cartes de 216 points, la carte sous le pointeur ne change pas.
#[test]
fn card_under_pointer_is_the_top_one() {
    let layout = fan_layout(7, 600.0, None);
    assert_eq!(layout.card_at(10.0), Some(0));
    assert_eq!(layout.card_at(70.0), Some(1));
    assert_eq!(layout.card_at(599.0), Some(6));
    assert_eq!(layout.card_at(-1.0), None);
    assert_eq!(layout.card_at(600.0), None);
    let layout = fan_layout(7, 600.0, Some(3));
    assert_eq!(layout.card_at(200.0), Some(3));
    assert_eq!(layout.card_at(330.0), Some(4));
}

// --- Panneau du bas : unités ---

// Spec 073 C8
#[test]
fn empty_units_section() {
    let catalog = french();
    let state = Units::default().state();
    let section = screen_of(&state, &catalog).units;
    assert_eq!(section.header, "Unités : 0/1");
    assert_eq!(section.empty.as_deref(), Some("Aucune unité"));
    let state = Units {
        owned: vec![UnitCard::new(Unit::Peasants, 1)],
        ..Units::default()
    }
    .state();
    assert_eq!(screen_of(&state, &catalog).units.empty, None);
}

// --- Carte de la main ---

// Spec 073 C13 — MKUE p. 4, *Deed Cards* 3.a à 3.d : effet de base et effet renforcé.
#[test]
fn hand_card_shows_every_option() {
    let catalog = french();
    let rage = card(Rage, 1);
    let state = state_with(&[rage, card(March, 1)]);
    let screen = screen_of(&state, &catalog);
    let rage = held(&screen, rage);
    assert_eq!(rage.effect, "Attaque 2 ou Parade 2");
    assert_eq!(
        summary(&rage.basic_effects),
        expected(&[
            (Face::Icon(sword(), "2"), None, false),
            (Face::Icon(shield(), "2"), None, false),
        ])
    );
    assert_eq!(rage.strong_text, "Attaque 4");
    assert_eq!(
        summary(&rage.strong_effects),
        // Spec #11 C8 : le mana de la carte est dans le titre du bloc, plus sur le bouton.
        expected(&[(Face::Icon(sword(), "4"), None, false)])
    );
    assert_eq!(screen.strong_title, "Renforcé");
}

// Spec 073 C14
#[test]
fn effect_button_plays_legal_option() {
    let catalog = french();
    let rage = card(Rage, 1);
    let state = fight(
        VILLAGE,
        &[rage, card(March, 1)],
        &[(prowlers(), ORCS_2)],
        &[ORCS_2],
    );
    let state = to_block_phase(&state);
    let state = done(&state, declare_block(prowlers()));
    let screen = screen_of(&state, &catalog);
    let buttons = &held(&screen, rage).basic_effects;
    assert_eq!(buttons[0].command, None);
    assert_eq!(
        buttons[1].command,
        Some(Command::Play(index_of(&state, &option(rage, 1))))
    );
    assert_eq!(buttons[1].tooltip, "Jouer : Parade 2");
}

// Spec 073 C15 — spec 022 : la raison du grisé.
#[test]
fn effect_button_tooltips() {
    let catalog = french();
    let march = card(March, 1);
    let state = state_with(&[march]);
    let screen = screen_of(&state, &catalog);
    let march = held(&screen, march);
    assert_eq!(
        summary(&march.basic_effects),
        expected(&[(Face::Icon(Icon::Move, "2"), None, true)])
    );
    assert_eq!(march.basic_effects[0].tooltip, "Jouer : Mouvement 2");
    assert_eq!(
        summary(&march.strong_effects),
        // Spec #11 C8 : plus de pastille verte sur le bouton renforcé.
        expected(&[(Face::Icon(Icon::Move, "4"), None, false)])
    );
    assert_eq!(
        march.strong_effects[0].tooltip,
        "Renforcer : Mouvement 4\nIl faut un mana <green>vert</green> : un dé <green>vert</green> ou <gold>or</gold> de la Source, un cristal ou un jeton <green>vert</green>."
    );
}

// Spec 073 C16
#[test]
fn effect_buttons_without_icon() {
    let catalog = french();
    let tranquility = card(Tranquility, 1);
    let concentration = card(Concentration, 1);
    let state = state_with(&[tranquility, concentration]);
    let screen = screen_of(&state, &catalog);
    let tranquility = held(&screen, tranquility);
    assert_eq!(
        summary(&tranquility.basic_effects),
        expected(&[
            // Le moteur offre le Soin même sans Blessure en main (spec 034).
            (Face::Text("Soin 1"), None, true),
            (Face::Text("Piocher 1 carte(s)"), None, true),
        ])
    );
    assert_eq!(
        summary(&tranquility.strong_effects),
        expected(&[
            // Spec #11 C8 : plus de pastille verte sur les boutons renforcés.
            (Face::Text("Soin 2"), None, false),
            (Face::Text("Piocher 2 carte(s)"), None, false),
        ])
    );
    let concentration = held(&screen, concentration);
    assert_eq!(
        summary(&concentration.basic_effects),
        expected(&[
            (
                Face::Crystal(ManaColor::Basic(BasicColor::Blue)),
                None,
                true
            ),
            (
                Face::Crystal(ManaColor::Basic(BasicColor::White)),
                None,
                true
            ),
            (Face::Crystal(ManaColor::Basic(BasicColor::Red)), None, true),
        ])
    );
}

// Spec 073 C17, remplacé par la spec #11 C11 (Q2) : les options payantes dans le bloc de base,
// au format « pastille : effet » ; le bloc renforcé sans pastille.
#[test]
fn paid_option_in_basic_block() {
    let catalog = french();
    let crystallize = card(Crystallize, 1);
    let state = state_with(&[crystallize]);
    let state = apply(
        state,
        &Event::CrystalGained {
            player: SOLO,
            color: BasicColor::Red,
        },
    );
    let screen = screen_of(&state, &catalog);
    let crystallize = held(&screen, crystallize);
    let white = Some(ManaColor::Basic(BasicColor::White));
    let green = Some(ManaColor::Basic(BasicColor::Green));
    assert_eq!(
        summary(&crystallize.basic_effects),
        expected(&[
            (Face::Crystal(ManaColor::Basic(BasicColor::Red)), RED, true),
            (
                Face::Crystal(ManaColor::Basic(BasicColor::Blue)),
                BLUE,
                false
            ),
            (
                Face::Crystal(ManaColor::Basic(BasicColor::White)),
                white,
                false
            ),
            (
                Face::Crystal(ManaColor::Basic(BasicColor::Green)),
                green,
                false
            ),
        ])
    );
    assert_eq!(
        crystallize.basic_effects[0].command,
        crystallize.paid_plays[0].command.into()
    );
    let strong = &crystallize.strong_effects;
    assert_eq!(strong.len(), 4);
    assert!(strong.iter().all(|button| button.color.is_none()));
}

// Spec 073 C18 — comme la spec 071, C17.
#[test]
fn wound_card_has_no_button() {
    let catalog = french();
    let state = state_with(&[wound(1), card(March, 1)]);
    let screen = screen_of(&state, &catalog);
    let wound = held(&screen, wound(1));
    assert!(wound.basic_effects.is_empty());
    assert!(wound.strong_effects.is_empty());
    assert!(wound.sideways.is_empty());
}

// Spec 073 C19 — comme les specs 013 et 016.
#[test]
fn modes_keep_their_card_buttons() {
    let catalog = french();
    let march = card(March, 1);
    let improvisation = card(Improvisation, 1);
    let state = state_with(&[improvisation, march, card(Rage, 1)]);
    let ending = ScreenMode::EndingTurn(EndTurnChoice::default());
    let screen = screen_in(&state, &ending, &catalog);
    for card in &screen.hand {
        assert!(card.discard_toggle.is_some(), "{}", card.name);
        assert!(
            card.basic_effects
                .iter()
                .chain(&card.strong_effects)
                .all(|button| button.command.is_none()),
            "{}",
            card.name
        );
    }
    let choosing = ScreenMode::ChoosingDiscard(DiscardChoice {
        card: improvisation,
        option: 0,
    });
    let screen = screen_in(&state, &choosing, &catalog);
    let march = held(&screen, march);
    assert!(march.basic_effects.is_empty());
    assert!(march.strong_effects.is_empty());
    let labels: Vec<&str> = march.plays.iter().map(|b| b.label.as_str()).collect();
    assert_eq!(labels, ["Défausser"]);
}

// Spec 073 C19 bis : en mode Concentration, la carte à renforcer montre dans son bloc renforcé un
// bouton d'effet actif par option, augmentée du bonus et sans pastille (gratuite) ; ses options de
// base restent grisées (remarques de Mathieu : Concentration renforcée sur Célérité introuvable en
// phase d'attaque à distance, puis bouton attendu sous « Renforcé »).
#[test]
fn boost_mode_keeps_its_card_buttons() {
    // MKUE p. 9, *Ranged and Siege Attack Phase* 8 : l'attaque à distance se joue une fois
    // l'attaque déclarée.
    let catalog = french();
    let (concentration, swiftness) = (card(Concentration, 1), card(Swiftness, 1));
    let state = fight(
        VILLAGE,
        &[concentration, swiftness, card(March, 1)],
        &[(prowlers(), ORCS_2)],
        &[ORCS_2],
    );
    let state = done(&state, declare(&[prowlers()]));
    let screen = screen_of(&state, &catalog);
    let boost = held(&screen, concentration)
        .strong_effects
        .iter()
        .find_map(|button| match button.command {
            Some(Command::ChooseBoost(choice)) => Some(choice),
            _ => None,
        })
        .expect("Concentration renforcée offerte");
    let screen = screen_in(&state, &ScreenMode::Boosting(boost), &catalog);
    let swiftness = held(&screen, swiftness);
    assert!(swiftness.plays.is_empty());
    assert!(
        swiftness
            .basic_effects
            .iter()
            .all(|button| button.command.is_none())
    );
    let ranged = attack_icon(AttackKind::Ranged, Element::Physical);
    assert_eq!(
        summary(&swiftness.strong_effects),
        expected(&[(Face::Icon(ranged, "5"), None, true)])
    );
    assert_eq!(
        swiftness.strong_effects[0].tooltip,
        "Renforcer avec Concentration : Attaque à distance 5"
    );
}

// --- Unités ---

fn golems() -> UnitCard {
    UnitCard::new(Unit::GuardianGolems, 1)
}

fn peasants() -> UnitCard {
    UnitCard::new(Unit::Peasants, 1)
}

// Spec 073 C20
#[test]
fn unit_effect_buttons() {
    let catalog = french();
    let state = Units {
        level: 3,
        owned: vec![golems()],
        ..Units::default()
    }
    .state();
    let screen = screen_of(&state, &catalog);
    assert_eq!(
        summary(&screen.units.units[0].effects),
        expected(&[
            (Face::Icon(sword(), "2"), None, false),
            (Face::Icon(shield(), "2"), None, false),
            (Face::Icon(block_icon(Element::Fire), "4"), RED, false),
            (Face::Icon(block_icon(Element::Ice), "4"), BLUE, false),
        ])
    );
}

// Spec #11 C13 : la pastille dit le mana, le bouton ne le répète plus.
#[test]
fn unit_mana_ability_omits_mana_prefix() {
    let catalog = french();
    let state = Units {
        level: 3,
        owned: vec![UnitCard::new(Unit::Illusionists, 1)],
        ..Units::default()
    }
    .state();
    let screen = screen_of(&state, &catalog);
    let ability = &screen.units.units[0].effects[1];
    assert_eq!(
        ability.face,
        EffectFace::Text("Un ennemi [[fortified]]non fortifié n'attaque pas".into())
    );
    assert_eq!(ability.color, Some(ManaColor::Basic(BasicColor::White)));
}

// Spec 073 C21
#[test]
fn unit_option_plays_directly() {
    let catalog = french();
    let state = Units {
        owned: vec![peasants()],
        ..Units::default()
    }
    .state();
    let screen = screen_of(&state, &catalog);
    let effects = &screen.units.units[0].effects;
    assert_eq!(
        summary(effects),
        expected(&[
            (Face::Icon(sword(), "2"), None, false),
            (Face::Icon(shield(), "2"), None, false),
            (Face::Icon(Icon::Influence, "2"), None, true),
            (Face::Icon(Icon::Move, "2"), None, true),
        ])
    );
    let influence = Action::ActivateUnit {
        enemy: None,
        unit: peasants(),
        ability: 1,
        option: 0,
        mana: Vec::new(),
        target: None,
        leadership: false,
    };
    assert_eq!(
        effects[2].command,
        Some(Command::Play(index_of(&state, &influence)))
    );
}

// Spec 073 C22 — spec 042 : plusieurs paiements ouvrent le choix réduit à l'option.
#[test]
fn unit_option_opens_mana_choice() {
    let catalog = french();
    let state = Units {
        level: 3,
        owned: vec![golems()],
        placed: vec![(prowlers(), ORCS_2)],
        ..Units::default()
    }
    .state();
    let state = apply(
        state,
        &Event::CrystalGained {
            player: SOLO,
            color: BasicColor::Red,
        },
    );
    let state = done(&state, challenge(ORCS_2));
    let state = done(&state, Action::StartCombat);
    let state = to_block_phase(&state);
    let state = done(&state, declare_block(prowlers()));
    let screen = screen_of(&state, &catalog);
    let fire = &screen.units.units[0].effects[2];
    let command = Command::ChooseUnitAbility {
        unit: golems(),
        ability: 1,
        option: Some(0),
    };
    assert_eq!(fire.command, Some(command));
    let mode = ScreenMode::ChoosingUnitAbility {
        unit: golems(),
        ability: 1,
        option: Some(0),
    };
    let prompt = screen_in(&state, &mode, &catalog)
        .unit_prompt
        .expect("choix du mana");
    let labels: Vec<&str> = prompt.payments.iter().map(|b| b.label.as_str()).collect();
    assert_eq!(
        labels,
        [
            "Parade de feu 4 · Dé <red>rouge</red>",
            "Parade de feu 4 · Cristal <red>rouge</red> (1 sur 3)"
        ]
    );
}

// --- Panneau du bas : sections ---

// Spec 073 C7 : l'ordre des sections est celui du dessin (`app/bottom_panel.rs`), vérifié à
// l'œil ; le champ que lisait ce test n'était pas dessiné (issue privée 259).

// --- Panneau de droite ---

// Spec 073 C23
#[test]
fn player_header() {
    let catalog = french();
    let screen = screen_of(&Units::default().state(), &catalog);
    assert_eq!(screen.portrait.expect("portrait").name, "Arythea");
    assert_eq!(screen.player_level, "Niveau 1");
    assert_eq!(screen.player_fame.icon, Icon::Fame);
    assert_eq!(screen.player_fame.value, "0 / 3");
    let top = Units {
        level: 10,
        ..Units::default()
    }
    .state();
    let screen = screen_of(&top, &catalog);
    assert_eq!(screen.player_level, "Niveau 10");
    assert_eq!(screen.player_fame.value, "0");
}

// Spec 073 C24 — MKUE p. 3, *Play Area* : la défausse est consultable, le deck caché.
#[test]
fn deck_and_discard_line() {
    let catalog = french();
    let state = state_with(&move_hand());
    let screen = screen_of(&state, &catalog);
    // La spec compte 11 cartes avec le deck d'Arythea ; celui de l'arène (Tovak, plus les
    // exemplaires de `move_hand`) en garde 12.
    assert_eq!(screen.deck, "Deck : 12 carte(s)");
    assert_eq!(
        labels_and_commands(std::slice::from_ref(&screen.discard_button)),
        [("Défausse : 0 carte(s)", &None)]
    );
    let end_turn = legal_actions(&state, SOLO)
        .into_iter()
        .find(|action| matches!(action, Action::EndTurn { discard, .. } if discard.len() == 2))
        .expect("fin du tour qui défausse deux cartes");
    let state = done(&state, end_turn);
    let screen = screen_of(&state, &catalog);
    assert_eq!(
        labels_and_commands(std::slice::from_ref(&screen.discard_button)),
        [("Défausse : 2 carte(s)", &Some(Command::OpenDiscard))]
    );
}

// Spec 075, C32 (remplace la section Ici de la spec 073, C25 : issue privée 169) : les actions du
// lieu passent sur le plateau ; seuls les boutons de soin hors combat restent, sous les points.
#[test]
fn healing_buttons_follow_the_points() {
    let catalog = french();
    let village = Units {
        placed: vec![(prowlers(), ORCS_2)],
        ..Units::default()
    }
    .state();
    // Interagir, piller et défier sont sur le plateau, pas dans le panneau.
    let screen = screen_of(&village, &catalog);
    assert_eq!(screen.board_actions.len(), 3);
    assert!(screen.player_buttons().is_empty());
    let state = Units {
        hand: vec![wound(1), card(March, 1)],
        ..Units::default()
    }
    .state();
    let state = apply(
        state,
        &Event::CardPlayed {
            player: SOLO,
            card: card(March, 1),
            play: CardPlay::Sideways(SidewaysEffect::Move),
            effect: Effect::Heal(2),
        },
    );
    let screen = screen_of(&state, &catalog);
    assert_eq!(
        screen.player_buttons(),
        [(
            "Jeter une Blessure (1)",
            &Command::Play(index_of(
                &state,
                &Action::SpendHealing {
                    on: HealingTarget::Wound
                }
            ))
        )]
    );
}

// Spec 073 C26 — spec 025 : les jetons dépensés restent visibles jusqu'à la fin du tour.
#[test]
fn spent_tokens_shown() {
    let catalog = french();
    let concentration = card(Concentration, 1);
    let stamina = card(Stamina, 1);
    let state = state_with(&[concentration, stamina, card(March, 1)]);
    let state = done(&state, option(concentration, 0));
    let tokens = screen_of(&state, &catalog).tokens.expect("jetons");
    assert_eq!(tokens.tokens.len(), 1);
    assert!(tokens.spent.is_empty());
    let strong = Action::PlayCard {
        card: stamina,
        play: CardPlay::Strong {
            gain: None,
            extra_mana: None,
            option: 0,
            discard: None,
            mana: arcanum_core::ManaPayment::Token,
            set_dice: [None, None],
            with: None,
        },
    };
    let state = done(&state, strong);
    let tokens = screen_of(&state, &catalog).tokens.expect("jetons");
    assert!(tokens.tokens.is_empty());
    let spent: Vec<(ManaColor, &str)> = tokens
        .spent
        .iter()
        .map(|token| (token.color, token.tooltip.as_str()))
        .collect();
    assert_eq!(
        spent,
        [(
            ManaColor::Basic(BasicColor::Blue),
            "Jeton de mana <blue>bleu</blue> : dépensé ce tour"
        )]
    );
}

// Spec 073 C27
#[test]
fn played_card_tooltip() {
    let catalog = french();
    let march = card(March, 1);
    let state = done(&state_with(&[march]), option(march, 0));
    let screen = screen_of(&state, &catalog);
    assert_eq!(
        screen.play_area[0].tooltip,
        "Marche\nMouvement 2\nRenforcé : Mouvement 4"
    );
}

/// Ces événements du joueur, appliqués dans l'ordre.
fn applied(state: GameState, events: &[Event]) -> GameState {
    events.iter().fold(state, apply)
}

// Spec 073 C28
#[test]
fn turn_facts_lines() {
    let catalog = french();
    // Les Paysans dans l'offre d'unités : un recrutement sans objet ne fait pas de fait.
    let state = applied(
        Units {
            offer: vec![peasants()],
            ..Units::default()
        }
        .state(),
        &[
            Event::EnemiesDefeated {
                player: SOLO,
                enemies: vec![prowlers()],
                fame: 2,
            },
            Event::UnitRecruited {
                player: SOLO,
                unit: peasants(),
                cost: 4,
            },
            Event::ReputationChanged {
                player: SOLO,
                change: -1,
            },
            Event::WoundsTaken {
                player: SOLO,
                enemy: prowlers(),
                wounds: vec![wound(1), wound(2)],
            },
        ],
    );
    let screen = screen_of(&state, &catalog);
    assert_eq!(screen.this_turn_title, "Ce tour");
    assert_eq!(
        screen.turn_facts,
        [
            "Rôdeurs vaincu(s) : +2 Gloire",
            "Unité gagnée : Paysans",
            "Réputation −1",
            "2 Blessure(s) reçue(s)"
        ]
    );
    assert_eq!(screen.nothing_this_turn, None);
}

// Spec 073 C28
#[test]
fn no_turn_fact_line() {
    let catalog = french();
    let screen = screen_of(&Units::default().state(), &catalog);
    assert!(screen.turn_facts.is_empty());
    assert_eq!(
        screen.nothing_this_turn.as_deref(),
        Some("Rien de marquant")
    );
}

// Spec 073 C29 — spec 041 : la récompense du monastère incendié.
#[test]
fn pending_rewards_under_this_turn() {
    let catalog = french();
    let state = applied(
        Units::default().state(),
        &[Event::MonasteryBurned {
            player: SOLO,
            hex: VILLAGE,
        }],
    );
    let screen = screen_of(&state, &catalog);
    assert_eq!(screen.turn_facts, ["Monastère incendié"]);
    assert!(screen.reward_pending.is_some());
    assert_eq!(screen.nothing_this_turn, None);
}

// --- Boutons du bas ---

/// Un bouton du bas : libellé, commande (`None` grisé), infobulle, touche.
type BottomButton<'a> = (&'a str, Option<Command>, Option<&'a str>, Option<&'a str>);

fn bottom(screen: &GameScreen) -> Vec<BottomButton<'_>> {
    screen
        .bottom_buttons
        .iter()
        .map(|button| {
            (
                button.label.as_str(),
                button.command,
                button.tooltip.as_deref(),
                button.shortcut.as_deref(),
            )
        })
        .collect()
}

// Spec 073 C30
#[test]
fn end_turn_button_always_present() {
    let catalog = french();
    let screen = screen_of(&state_with(&move_hand()), &catalog);
    assert_eq!(
        bottom(&screen),
        [
            ("Offres…", Some(Command::OpenOffers), None, Some("O")),
            (
                "Finir le tour…",
                Some(Command::OpenEndTurn),
                None,
                Some("Fin")
            ),
        ]
    );
    let state = fight(VILLAGE, &move_hand(), &[(prowlers(), ORCS_2)], &[ORCS_2]);
    let screen = screen_of(&state, &catalog);
    assert_eq!(
        bottom(&screen)[1],
        (
            "Finir le tour…",
            None,
            Some("Finissez d'abord le combat"),
            Some("Fin")
        )
    );
}

// Spec 073 C31
#[test]
fn end_turn_button_disabled_reasons() {
    let catalog = french();
    let state = state_with(&move_hand());
    let choosing = ScreenMode::ChoosingDiscard(DiscardChoice {
        card: card(Improvisation, 1),
        option: 0,
    });
    let screen = screen_in(&state, &choosing, &catalog);
    assert_eq!(
        bottom(&screen)[1].2,
        Some("Terminez d'abord le choix en cours")
    );
    let finished = game_screen(
        &view(&state, SOLO),
        ScreenPhase::Finished,
        &ScreenMode::Normal,
        &catalog,
    );
    let buttons = bottom(&finished);
    let end_turn = buttons.last().expect("Finir le tour…");
    assert_eq!(end_turn.1, None);
    assert_eq!(end_turn.2, Some("La partie est terminée"));
}

// Spec 073 C32 — spec 016 : MKUE p. 6, 5.a, au moins une carte à défausser sans carte jouée.
#[test]
fn end_turn_mode_buttons() {
    let catalog = french();
    let state = state_with(&move_hand());
    let ending = ScreenMode::EndingTurn(EndTurnChoice::default());
    let screen = screen_in(&state, &ending, &catalog);
    assert_eq!(
        bottom(&screen),
        [
            ("Annuler", Some(Command::CancelChoice), None, Some("Échap")),
            ("Finir le tour", None, None, Some("Entrée")),
        ]
    );
    assert!(screen.end_turn_prompt.is_some());
    let ending = ScreenMode::EndingTurn(EndTurnChoice {
        discard: vec![card(March, 1)],
        ..EndTurnChoice::default()
    });
    let screen = screen_in(&state, &ending, &catalog);
    assert_eq!(bottom(&screen)[1].1, Some(Command::ConfirmEndTurn));
}

// Spec 073 C33 — spec 023 : MKUE p. 4, annonce de la fin de la manche.
#[test]
fn round_end_button() {
    let catalog = french();
    let order: Vec<CardId> = move_hand().to_vec();
    let state = state_with_deck(order, 5);
    let screen = screen_of(&state, &catalog);
    assert_eq!(
        bottom(&screen)
            .iter()
            .map(|button| button.0)
            .collect::<Vec<_>>(),
        ["Offres…", "Finir la manche…", "Finir le tour…"]
    );
    assert_eq!(
        bottom(&screen)[1].2,
        Some(
            "Votre deck est vide : vous pouvez annoncer la fin de la manche au lieu de jouer ce tour."
        )
    );
    assert_eq!(bottom(&screen)[1].1, Some(Command::OpenAnnounce));
    assert_eq!(screen.round_end_confirm, None);
    let screen = screen_in(&state, &ScreenMode::ConfirmingEndOfRound, &catalog);
    assert!(screen.round_end_confirm.is_some());
    // Main vide : l'annonce est directe.
    let empty = state_with_deck(Vec::new(), 0);
    let screen = screen_of(&empty, &catalog);
    let announce = index_of(&empty, &Action::AnnounceEndOfRound);
    assert!(
        bottom(&screen)
            .iter()
            .any(|button| button.0 == "Finir la manche…"
                && button.1 == Some(Command::Play(announce)))
    );
}

// Spec 073 C34 — spec 050 : comme la touche O.
#[test]
fn offers_button_always_present() {
    let catalog = french();
    let state = fight(VILLAGE, &move_hand(), &[(prowlers(), ORCS_2)], &[ORCS_2]);
    let screen = screen_of(&state, &catalog);
    assert_eq!(
        bottom(&screen)[0],
        (
            "Offres…",
            None,
            Some("Impossible pendant un combat"),
            Some("O")
        )
    );
    let state = state_with(&move_hand());
    let choosing = ScreenMode::ChoosingDiscard(DiscardChoice {
        card: card(Improvisation, 1),
        option: 0,
    });
    let screen = screen_in(&state, &choosing, &catalog);
    assert_eq!(
        bottom(&screen)[0],
        (
            "Offres…",
            None,
            Some("Terminez d'abord le choix en cours"),
            Some("O")
        )
    );
}

// --- Défausse ---

fn session() -> GameSession {
    let mut session = started_session(first_reconnaissance_config(42));
    session.run_frame();
    session
}

/// Joue la première carte de côté, puis finit le tour en défaussant la suivante.
fn end_first_turn(session: &mut GameSession, catalog: &Catalog) {
    let command = session.screen(catalog).hand[0].sideways[0].command;
    session.command(command);
    session.run_frame();
    session.command(Command::OpenEndTurn);
    let discarded = session.screen(catalog).hand[0].card;
    session.command(Command::ToggleEndTurnDiscard(discarded));
    session.command(Command::ConfirmEndTurn);
    session.run_frame();
}

// Spec 073 C35 — MKUE p. 3, *Play Area* : la défausse est face visible.
#[test]
fn discard_window_lists_cards() {
    let catalog = french();
    let mut session = session();
    end_first_turn(&mut session, &catalog);
    let discard = session.view().discard;
    assert_eq!(discard.len(), 2);
    assert_eq!(session.discard_window(&catalog), None);
    session.command(Command::OpenDiscard);
    let window = session
        .discard_window(&catalog)
        .expect("fenêtre de la défausse");
    assert_eq!(window.title, "Défausse");
    let cards: Vec<CardId> = window.cards.iter().map(|card| card.card).collect();
    let mut latest_first = discard.clone();
    latest_first.reverse();
    assert_eq!(cards, latest_first);
    for card in &window.cards {
        assert!(!card.name.is_empty());
        assert!(!card.effect.is_empty());
        assert!(!card.strong_text.is_empty());
        assert!(card.plays.is_empty() && card.sideways.is_empty());
        assert!(
            card.basic_effects
                .iter()
                .chain(&card.strong_effects)
                .all(|b| b.command.is_none())
        );
    }
    assert_eq!(
        (window.close.label.as_str(), window.close.command),
        ("Fermer", Command::CloseDiscard)
    );
    session.command(Command::CloseDiscard);
    assert_eq!(session.discard_window(&catalog), None);
    session.shortcut(Shortcut::Discard);
    assert!(session.discard_window(&catalog).is_some());
    session.shortcut(Shortcut::Cancel);
    assert_eq!(session.discard_window(&catalog), None);
    session.shortcut(Shortcut::Discard);
    session.shortcut(Shortcut::Discard);
    assert_eq!(session.discard_window(&catalog), None);
    assert_eq!(
        session.screen(&catalog).discard_button.shortcut.as_deref(),
        Some("D")
    );
}

// Spec 073 C36
#[test]
fn empty_discard_has_no_window() {
    let catalog = french();
    let mut session = session();
    assert!(session.view().discard.is_empty());
    assert_eq!(session.screen(&catalog).discard_button.command, None);
    session.command(Command::OpenDiscard);
    assert_eq!(session.discard_window(&catalog), None);
    session.shortcut(Shortcut::Discard);
    assert_eq!(session.discard_window(&catalog), None);
}

// --- Combat ---

// Spec 073 C37 — la main et les unités passent dans le panneau du bas, avec les boutons de la phase.
#[test]
fn combat_hand_in_bottom_panel() {
    let catalog = french();
    let rage = card(Rage, 1);
    let state = Units {
        hand: vec![rage, card(March, 1)],
        level: 3,
        owned: vec![golems()],
        placed: vec![(prowlers(), ORCS_2)],
        ..Units::default()
    }
    .state();
    let state = done(&state, challenge(ORCS_2));
    let state = done(&state, Action::StartCombat);
    let state = to_block_phase(&state);
    let state = done(&state, declare_block(prowlers()));
    let screen = screen_of(&state, &catalog);
    assert_eq!(screen.hand.len(), 2);
    assert!(held(&screen, rage).basic_effects[1].command.is_some());
    // La parade physique des Golems gardiens se joue en phase de parade.
    assert!(screen.units.units[0].effects[1].command.is_some());
}
