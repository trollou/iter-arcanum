//! Icônes de l'interface (spec 071) : valeurs à icône, lignes du panneau, cristaux et jetons.
//! Sans fichier d'icône, le nom puis la valeur s'écrivent à la place (L1).

use eframe::egui::{
    self, Color32, Rect, Response, Sense, Stroke, TextureId, Vec2, load::SizedTexture,
};

use super::widgets::{card_color, mana_color};
use super::{IterArcanumApp, SWATCH_STROKE, full_uv};

use crate::{
    CRYSTAL_EMPTY_IMAGE, Command, CrystalGroup, DummyCrystals, Icon, IconButton, IconLine,
    IconValue, ImageId, ImagePaint, InventoryLine, TokenLine, ValuePiece, crystal_image,
};
use arcanum_core::ManaColor;

/// Une icône occupe la hauteur du texte, un peu plus (spec 071, « Interface »).
pub(super) fn icon_size(ui: &egui::Ui) -> f32 {
    ui.text_style_height(&egui::TextStyle::Body) + 4.0
}

impl IterArcanumApp {
    /// La texture de l'image à cette taille, ou rien si elle n'est pas chargée.
    pub(super) fn icon_texture(
        &mut self,
        ui: &egui::Ui,
        image: ImageId,
        size: f32,
    ) -> Option<TextureId> {
        let rect = Rect::from_min_size(egui::Pos2::ZERO, Vec2::splat(size));
        let pixels_per_point = ui.ctx().pixels_per_point();
        match self.paint_of(ui, image, Color32::TRANSPARENT, rect, pixels_per_point) {
            ImagePaint::Texture(texture) => Some(texture),
            ImagePaint::Fallback(_) | ImagePaint::Pending(_) => None,
        }
    }

    /// L'icône seule, carrée, avec l'infobulle ; `None` sans fichier.
    pub(super) fn icon_image(
        &mut self,
        ui: &mut egui::Ui,
        image: ImageId,
        size: f32,
    ) -> Option<Response> {
        let texture = self.icon_texture(ui, image, size)?;
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
        ui.painter().image(texture, rect, full_uv(), Color32::WHITE);
        Some(response)
    }

    /// Icône puis valeur, le nom en infobulle ; sans fichier, « Mouvement 3 » (L1).
    pub(super) fn icon_value(&mut self, ui: &mut egui::Ui, value: &IconValue, strong: bool) {
        let size = icon_size(ui);
        let text = |text: String| {
            if strong {
                egui::RichText::new(text).strong()
            } else {
                egui::RichText::new(text)
            }
        };
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            match self.icon_image(ui, value.icon.image(), size) {
                Some(response) => {
                    icon_tooltip(response, value);
                    if !value.value.is_empty() {
                        icon_tooltip(ui.label(text(value.value.clone())), value);
                    }
                }
                None => {
                    ui.label(text(value.fallback_text()));
                }
            }
        });
    }

    /// L'encart de côté d'une carte (spec 071, C14, variante B) : la carte couchée, non cliquable,
    /// puis un bouton-icône par effet ; dessiné de droite à gauche. Sans fichier d'icône, des
    /// boutons texte « Mouvement 1 » (L1).
    pub(super) fn sideways_buttons(
        &mut self,
        ui: &mut egui::Ui,
        title: &str,
        buttons: &[IconButton],
    ) -> Option<Command> {
        const BUTTON_ICON: f32 = 16.0;
        let mut clicked = None;
        egui::Frame::new()
            .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
            .corner_radius(4.0)
            .inner_margin(egui::Margin::symmetric(3, 1))
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                ui.spacing_mut().button_padding = Vec2::new(3.0, 1.0);
                for button in buttons.iter().rev() {
                    let widget = match self.icon_texture(ui, button.icon.image(), BUTTON_ICON) {
                        Some(texture) => {
                            let image = egui::Image::from_texture(SizedTexture::new(
                                texture,
                                Vec2::splat(BUTTON_ICON),
                            ));
                            match &button.mark {
                                Some(mark) => egui::Button::image_and_text(image, mark.as_str()),
                                None => egui::Button::image(image),
                            }
                        }
                        None => egui::Button::new(button.label.as_str()),
                    };
                    if ui.add(widget).on_hover_text(&button.tooltip).clicked() {
                        clicked = Some(button.command);
                    }
                }
                match self.icon_image(ui, Icon::Sideways.image(), BUTTON_ICON) {
                    Some(response) => {
                        response.on_hover_text(title);
                    }
                    None => {
                        ui.weak(title);
                    }
                }
            });
        clicked
    }

    /// Une ligne de valeurs : icônes et mots sans icône (spec 071, C8 et C9), puis les
    /// résistances au bout (C10).
    pub(super) fn value_pieces(
        &mut self,
        ui: &mut egui::Ui,
        pieces: &[ValuePiece],
        resistances: &[IconValue],
    ) {
        ui.horizontal_wrapped(|ui| {
            for piece in pieces {
                match piece {
                    ValuePiece::Icon(value) => self.icon_value(ui, value, false),
                    ValuePiece::Text(text) => {
                        ui.label(text);
                    }
                }
                ui.add_space(4.0);
            }
            if !resistances.is_empty() {
                ui.separator();
            }
            for value in resistances {
                self.icon_value(ui, value, false);
            }
        });
    }

    /// Un texte à repères d'icône (spec #27), voir `marked_label`.
    pub(super) fn marked_label(
        &mut self,
        ui: &mut egui::Ui,
        text: &str,
        style: impl Fn(String) -> egui::RichText,
    ) {
        marked_label(ui, text, style, &self.catalog);
    }

    /// Les textures des icônes des repères à la hauteur du texte, gardées pour l'image dans le
    /// contexte egui : les dessins sans accès à l'application (fenêtres de cartes) les y lisent.
    pub(super) fn store_marker_icons(&mut self, ui: &egui::Ui) {
        let size = ui.text_style_height(&egui::TextStyle::Body);
        let textures: MarkerIcons = MarkerIcons(
            crate::ICON_MARKERS
                .iter()
                .filter_map(|&(_, icon)| {
                    self.icon_texture(ui, icon.image(), size)
                        .map(|texture| (icon, texture))
                })
                .collect(),
        );
        ui.ctx()
            .data_mut(|data| data.insert_temp(egui::Id::unique(MARKER_ICONS), textures));
    }

    /// Une rangée d'icônes sans valeur (capacités d'un ennemi, spec #27) ; rien si elle est vide.
    pub(super) fn icon_row(&mut self, ui: &mut egui::Ui, values: &[IconValue]) {
        if values.is_empty() {
            return;
        }
        ui.horizontal_wrapped(|ui| {
            for value in values {
                self.icon_value(ui, value, false);
            }
        });
    }

    /// Une ligne du panneau : ses icônes, puis son texte, replié s'il est long.
    pub(super) fn icon_line(&mut self, ui: &mut egui::Ui, line: &IconLine, strong: bool) {
        ui.horizontal_wrapped(|ui| {
            for value in &line.icons {
                self.icon_value(ui, value, strong);
                ui.add_space(4.0);
            }
            if !line.text.is_empty() {
                let text = egui::RichText::new(&line.text);
                ui.add(egui::Label::new(if strong { text.strong() } else { text }).wrap());
            }
        });
    }

    /// Ligne « Inventaire » : par couleur, un cristal plein par cristal possédé, un emplacement
    /// vide jusqu'à la limite (spec 021 ; spec 071, C6).
    pub(super) fn inventory_line(&mut self, ui: &mut egui::Ui, inventory: &InventoryLine) {
        ui.horizontal_wrapped(|ui| {
            ui.label(&inventory.title);
            for group in &inventory.groups {
                self.crystal_group(ui, group);
                ui.add_space(4.0);
            }
        });
    }

    /// Les cristaux du fantôme (spec 061, retour de Mathieu) : l'icône colorée une fois par
    /// cristal, à la suite, sans emplacement vide.
    pub(super) fn dummy_crystals(&mut self, ui: &mut egui::Ui, crystals: &DummyCrystals) {
        let size = icon_size(ui);
        ui.horizontal_wrapped(|ui| {
            ui.label(&crystals.title);
            for &color in &crystals.crystals {
                let (cell, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
                match self.icon_texture(ui, crystal_image(ManaColor::Basic(color)), size) {
                    Some(texture) => {
                        ui.painter().image(texture, cell, full_uv(), Color32::WHITE);
                    }
                    None => paint_diamond(ui, cell, card_color(color), true),
                }
                response.on_hover_text(&crystals.tooltip);
            }
        });
    }

    fn crystal_group(&mut self, ui: &mut egui::Ui, group: &CrystalGroup) {
        let size = icon_size(ui);
        // Un cristal est plus haut que large : les emplacements se chevauchent un peu.
        let step = size * 0.62;
        let width = step * f32::from(group.limit.saturating_sub(1)) + size;
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, size), Sense::hover());
        let full = crystal_image(ManaColor::Basic(group.color));
        let textures = (
            self.icon_texture(ui, full, size),
            self.icon_texture(ui, CRYSTAL_EMPTY_IMAGE, size),
        );
        for slot in 0..group.limit {
            let filled = slot < group.count;
            let cell = Rect::from_min_size(
                rect.min + Vec2::new(step * f32::from(slot), 0.0),
                Vec2::splat(size),
            );
            match (filled, textures) {
                (true, (Some(texture), _)) | (false, (_, Some(texture))) => {
                    ui.painter().image(texture, cell, full_uv(), Color32::WHITE);
                }
                _ => paint_diamond(ui, cell, card_color(group.color), filled),
            }
        }
        response.on_hover_text(&group.tooltip);
    }

    /// Ligne « Jetons : » : le cristal de la couleur de chaque jeton, or et noir compris (spec
    /// 025 ; spec 071, C7).
    pub(super) fn token_line(&mut self, ui: &mut egui::Ui, tokens: &TokenLine) {
        let size = icon_size(ui);
        ui.horizontal_wrapped(|ui| {
            ui.label(&tokens.title);
            for token in &tokens.tokens {
                let texture = self.icon_texture(ui, crystal_image(token.color), size);
                let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
                match texture {
                    Some(texture) => {
                        ui.painter().image(texture, rect, full_uv(), Color32::WHITE);
                    }
                    None => {
                        ui.painter().circle(
                            rect.center(),
                            6.0,
                            mana_color(token.color),
                            Stroke::new(1.0, SWATCH_STROKE),
                        );
                    }
                }
                response.on_hover_text(&token.tooltip);
            }
            // Spec 073, C26 : les jetons dépensés ce tour, grisés.
            for token in &tokens.spent {
                let texture = self.icon_texture(ui, crystal_image(token.color), size);
                let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
                let tint = Color32::from_white_alpha(70);
                match texture {
                    Some(texture) => {
                        ui.painter().image(texture, rect, full_uv(), tint);
                    }
                    None => {
                        ui.painter().circle(
                            rect.center(),
                            6.0,
                            mana_color(token.color).gamma_multiply(0.35),
                            Stroke::new(1.0, SWATCH_STROKE),
                        );
                    }
                }
                response.on_hover_text(&token.tooltip);
            }
        });
    }
}

const MARKER_ICONS: &str = "marker_icons";

/// Un morceau de texte estompé, pour `marked_label`.
pub(super) fn weak_text(text: String) -> egui::RichText {
    egui::RichText::new(text).weak()
}

/// Les textures des icônes des repères (`store_marker_icons`).
#[derive(Clone, Default)]
struct MarkerIcons(std::collections::HashMap<Icon, TextureId>);

/// Un texte à repères d'icône (spec #27) : chaque repère devient l'icône, à la hauteur du texte,
/// avec son nom et sa règle en infobulle ; sans fichier d'icône, son nom. Un texte sans repère
/// s'écrit tel quel. `style` met en forme chaque morceau de texte ; le tout est replié s'il est
/// long.
pub(super) fn marked_label(
    ui: &mut egui::Ui,
    text: &str,
    style: impl Fn(String) -> egui::RichText,
    catalog: &arcanum_i18n::Catalog,
) {
    if !text.contains("[[") {
        ui.add(egui::Label::new(style(text.to_string())).wrap());
        return;
    }
    let icons: MarkerIcons = ui
        .ctx()
        .data(|data| data.get_temp(egui::Id::unique(MARKER_ICONS)))
        .unwrap_or_default();
    let size = ui.text_style_height(&egui::TextStyle::Body);
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        for piece in crate::icon_pieces(text, catalog) {
            match piece {
                ValuePiece::Text(words) => {
                    ui.add(egui::Label::new(style(words)).wrap());
                }
                ValuePiece::Icon(value) => match icons.0.get(&value.icon) {
                    Some(&texture) => {
                        let (rect, response) =
                            ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
                        ui.painter().image(texture, rect, full_uv(), Color32::WHITE);
                        icon_tooltip(response, &value);
                    }
                    None => {
                        ui.label(style(value.tooltip.clone()));
                    }
                },
            }
        }
    });
}

/// L'infobulle d'une icône : son nom, ou son nom en gras et sa règle dessous, repliée (spec #27,
/// « Infobulle à deux étages »).
pub(super) fn icon_tooltip(response: Response, value: &IconValue) -> Response {
    match &value.rule {
        None => response.on_hover_text(&value.tooltip),
        Some(rule) => response.on_hover_ui(|ui| {
            ui.set_max_width(320.0);
            ui.strong(&value.tooltip);
            ui.add(egui::Label::new(rule).wrap());
        }),
    }
}

/// Le losange de repli d'un cristal (spec 021), sans fichier d'icône.
fn paint_diamond(ui: &egui::Ui, cell: Rect, color: Color32, filled: bool) {
    let center = cell.center();
    let diamond = vec![
        center + Vec2::new(0.0, -6.0),
        center + Vec2::new(5.0, 0.0),
        center + Vec2::new(0.0, 6.0),
        center + Vec2::new(-5.0, 0.0),
    ];
    ui.painter().add(egui::Shape::convex_polygon(
        diamond,
        if filled { color } else { Color32::TRANSPARENT },
        Stroke::new(1.0, if filled { SWATCH_STROKE } else { color }),
    ));
}
