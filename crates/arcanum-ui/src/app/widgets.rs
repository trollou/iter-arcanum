//! Petits widgets du panneau latéral et de l'écran de combat : cartes, dés, cristaux, jetons.

use eframe::egui::{self, Color32, Sense, Stroke, Vec2};

use super::{CHOICE_STROKE, SWATCH_STROKE};
use crate::{Command, ReclaimMenu, SourceDie, SourceLine, StrongButton, hover_text};
use arcanum_core::{BasicColor, ManaColor};
use arcanum_i18n::Catalog;

/// Pastille de la couleur d'une carte.
pub(super) fn color_swatch(ui: &mut egui::Ui, color: BasicColor) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(12.0), Sense::hover());
    ui.painter().circle(
        rect.center(),
        6.0,
        card_color(color),
        Stroke::new(1.0, SWATCH_STROKE),
    );
}

/// Ligne « Source » : les dés utilisables, puis les épuisés grisés (spec 020) ; renvoie le dé
/// cliqué en mode « mettre un dé sur une couleur » (spec 025). Dessinée à la suite dans la barre
/// de menu, sans passer à la ligne (spec #10, L1).
pub(super) fn source_line(ui: &mut egui::Ui, source: &SourceLine) -> Option<Command> {
    let mut clicked = None;
    ui.label(&source.title);
    for die in &source.dice {
        // Spec 058, C6 : plus de dé ce tour, les dés grisés comme épuisés.
        clicked = source_die(ui, die, die.forbidden).or(clicked);
    }
    if let Some(title) = &source.taken_title {
        ui.weak(title);
        for die in &source.taken {
            taken_die(ui, die);
        }
    }
    // Spec 058, C39 : le dé d'Embrasure de la Source, gardé tel quel à la fin du tour.
    if let Some(keep) = &source.keep
        && ui.button(&keep.label).clicked()
    {
        clicked = Some(keep.command);
    }
    if let Some(title) = &source.depleted_title {
        ui.weak(title);
        for die in &source.depleted {
            clicked = source_die(ui, die, true).or(clicked);
        }
    }
    clicked
}

/// Largeur de la ligne de la Source, que la barre réserve avant de tronquer le nom du scénario
/// (spec #10, L1).
pub(super) fn source_width(ui: &egui::Ui, source: &SourceLine) -> f32 {
    let spacing = ui.spacing().item_spacing.x;
    let dice = source
        .dice
        .iter()
        .chain(&source.taken)
        .chain(&source.depleted)
        .map(|die| die_width(ui, die) + spacing);
    let titles = [&source.taken_title, &source.depleted_title]
        .into_iter()
        .flatten()
        .map(|title| text_width(ui, title, egui::TextStyle::Body) + spacing);
    let keep = source.keep.as_ref().map_or(0.0, |keep| {
        text_width(ui, &keep.label, egui::TextStyle::Button)
            + 2.0 * ui.spacing().button_padding.x
            + spacing
    });
    text_width(ui, &source.title, egui::TextStyle::Body)
        + spacing
        + dice.sum::<f32>()
        + titles.sum::<f32>()
        + keep
}

/// Largeur de la pastille d'un dé et de l'écart qui la sépare de son nom.
const DIE_SWATCH: f32 = 14.0 + 3.0;

/// Largeur d'un texte sur une ligne.
fn text_width(ui: &egui::Ui, text: &str, style: egui::TextStyle) -> f32 {
    ui.painter()
        .layout_no_wrap(
            text.to_owned(),
            style.resolve(ui.style()),
            Color32::PLACEHOLDER,
        )
        .size()
        .x
}

/// Largeur d'un dé de la Source : pastille et nom, dans un bouton s'il est cliquable.
fn die_width(ui: &egui::Ui, die: &SourceDie) -> f32 {
    match die.pick {
        Some(_) => {
            DIE_SWATCH
                + text_width(ui, &die.name, egui::TextStyle::Button)
                + 2.0 * ui.spacing().button_padding.x
        }
        None => DIE_SWATCH + text_width(ui, &die.name, egui::TextStyle::Body),
    }
}

/// Un dé : carré arrondi de sa couleur et son nom, avec son info-bulle ; un bouton quand on peut le
/// choisir (spec 025).
fn source_die(ui: &mut egui::Ui, die: &SourceDie, depleted: bool) -> Option<Command> {
    let color = mana_color(die.color);
    let fill = if depleted {
        color.gamma_multiply(0.35)
    } else {
        color
    };
    let mut clicked = None;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 3.0;
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(14.0), Sense::hover());
        ui.painter().rect(
            rect,
            3.0,
            fill,
            Stroke::new(1.0, SWATCH_STROKE),
            egui::StrokeKind::Inside,
        );
        match die.pick {
            Some(command) => {
                // Spec 060 : un dé coché pour Appel du mana.
                if ui
                    .add(egui::Button::new(&die.name).selected(die.checked))
                    .clicked()
                {
                    clicked = Some(command);
                }
            }
            None if depleted => {
                ui.weak(&die.name);
            }
            None => {
                ui.label(&die.name);
            }
        }
    })
    .response
    .on_hover_text(&die.tooltip);
    clicked
}

/// Un dé pris ce tour : rond cerclé de sa couleur, pour le distinguer des dés libres (spec 022).
fn taken_die(ui: &mut egui::Ui, die: &SourceDie) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 3.0;
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(14.0), Sense::hover());
        ui.painter().circle(
            rect.center(),
            5.0,
            mana_color(die.color),
            Stroke::new(2.0, CHOICE_STROKE),
        );
        ui.label(&die.name);
    })
    .response
    .on_hover_text(&die.tooltip);
}

/// Bouton « Renforcer » : pastille de la couleur à payer, grisé avec sa raison (spec 022).
pub(super) fn strong_button(ui: &mut egui::Ui, button: &StrongButton) -> Option<Command> {
    let mut chosen = None;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 3.0;
        color_swatch(ui, button.color);
        // Repliable : « Renforcer : Mettre 2 dé(s) sur une couleur… » élargirait le panneau.
        let mut response = ui.add_enabled(button.enabled, egui::Button::new(&button.label).wrap());
        if let Some(tooltip) = &button.tooltip {
            response = response.on_disabled_hover_text(tooltip);
        }
        if response.clicked() {
            chosen = Some(button.command);
        }
    });
    chosen
}

/// « Reprendre Euphorie cristalline en défaussant : » et sa liste déroulante (spec 025).
pub(super) fn reclaim_menu(ui: &mut egui::Ui, menu: &ReclaimMenu) -> Option<Command> {
    let mut chosen = None;
    ui.horizontal_wrapped(|ui| {
        ui.label(&menu.label);
        egui::ComboBox::from_id_salt("reclaim")
            .selected_text(&menu.selected)
            .show_ui(ui, |ui| {
                for option in &menu.options {
                    if ui.button(&option.label).clicked() {
                        chosen = Some(option.command);
                    }
                }
            });
    });
    chosen
}

/// Couleurs provisoires des dés : celles des cartes, plus l'or et le noir.
pub(super) fn mana_color(color: ManaColor) -> Color32 {
    match color {
        ManaColor::Basic(basic) => card_color(basic),
        ManaColor::Gold => Color32::from_rgb(0xD4, 0xA0, 0x17),
        ManaColor::Black => Color32::from_rgb(0x22, 0x22, 0x26),
    }
}

/// La couleur d'un mot de couleur de mana dans un texte (spec #68, règle 8) : une variante
/// claire de la pastille, lisible sur le fond sombre ; le noir devient un gris violacé clair.
pub(super) fn mana_text_color(color: ManaColor) -> Color32 {
    match color {
        ManaColor::Basic(BasicColor::Red) => Color32::from_rgb(0xF0, 0x6E, 0x5E),
        ManaColor::Basic(BasicColor::Blue) => Color32::from_rgb(0x6F, 0xA8, 0xEC),
        ManaColor::Basic(BasicColor::White) => Color32::WHITE,
        ManaColor::Basic(BasicColor::Green) => Color32::from_rgb(0x6C, 0xC4, 0x70),
        ManaColor::Gold => Color32::from_rgb(0xEC, 0xC2, 0x4A),
        ManaColor::Black => Color32::from_rgb(0xB0, 0xA2, 0xCC),
    }
}

/// Couleurs provisoires des cartes (spec 013).
pub(super) fn card_color(color: BasicColor) -> Color32 {
    match color {
        BasicColor::Red => Color32::from_rgb(0xC0, 0x39, 0x2B),
        BasicColor::Blue => Color32::from_rgb(0x2F, 0x6D, 0xB5),
        BasicColor::White => Color32::from_rgb(0xEC, 0xE9, 0xE1),
        BasicColor::Green => Color32::from_rgb(0x3E, 0x8E, 0x41),
    }
}

/// Un bouton de l'écran, sa touche en infobulle (« Raccourci : Entrée », spec 071).
pub(super) fn shortcut_button(
    ui: &mut egui::Ui,
    label: &str,
    shortcut: Option<&str>,
    catalog: &Catalog,
) -> egui::Response {
    enabled_shortcut_button(ui, true, label, shortcut, catalog)
}

/// Comme `shortcut_button`, grisé si `enabled` est faux ; la touche reste dite en infobulle.
pub(super) fn enabled_shortcut_button(
    ui: &mut egui::Ui,
    enabled: bool,
    label: &str,
    shortcut: Option<&str>,
    catalog: &Catalog,
) -> egui::Response {
    // Spec #27, C21 : un bouton n'écrit que du texte, sans repère d'icône.
    let response = ui.add_enabled(
        enabled,
        egui::Button::new(crate::plain_text(label, catalog)),
    );
    match hover_text(None, shortcut, catalog) {
        Some(text) => response
            .on_hover_text(text.clone())
            .on_disabled_hover_text(text),
        None => response,
    }
}
