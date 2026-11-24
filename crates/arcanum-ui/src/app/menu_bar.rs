//! Barre de menu en haut de la fenêtre (spec 074) : le Menu, puis le scénario, la manche, les
//! pioches de tuiles et, à droite, le message de la dernière sauvegarde.

use arcanum_core::TimeOfDay;
use eframe::egui::{self, Color32, Sense, Vec2};

use super::{IterArcanumApp, full_uv};
use crate::{Clock, Command, ImagePaint, MenuBar, MenuItem, time_of_day_image};

/// Couleurs de repli de l'icône du moment de la journée (spec 023).
const CLOCK_DAY: Color32 = Color32::from_rgb(0xFF, 0xD5, 0x4A);
const CLOCK_NIGHT: Color32 = Color32::from_rgb(0xC9, 0xD4, 0xF2);

impl IterArcanumApp {
    /// La ligne de la barre ; rend la commande choisie dans le Menu.
    pub(super) fn menu_bar(&mut self, ui: &mut egui::Ui, bar: &MenuBar) -> Option<Command> {
        let mut clicked = None;
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button(&bar.title, |ui| {
                for item in &bar.items {
                    match item {
                        MenuItem::Separator => {
                            ui.separator();
                        }
                        MenuItem::Entry(entry) => {
                            let mut button = egui::Button::new(&entry.label);
                            if let Some(key) = &entry.shortcut {
                                button = button.shortcut_text(key.as_str());
                            }
                            if let Some(checked) = entry.checked {
                                button = button.selected(checked);
                            }
                            let mut response = ui.add_enabled(entry.enabled, button);
                            if let Some(hint) = &entry.hint {
                                response =
                                    response.on_hover_text(hint).on_disabled_hover_text(hint);
                            }
                            if response.clicked() {
                                clicked = Some(entry.command);
                                ui.close();
                            }
                        }
                    }
                }
            });
            ui.separator();
            // L7 : à droite d'abord le message, puis le reste de gauche à droite ; le nom du
            // scénario se tronque en dernier recours.
            let notice_width = bar.notice.as_ref().map_or(0.0, |notice| {
                ui.fonts_mut(|fonts| {
                    fonts
                        .layout_no_wrap(
                            notice.clone(),
                            egui::TextStyle::Body.resolve(ui.style()),
                            Color32::WHITE,
                        )
                        .size()
                        .x
                })
            });
            if let Some(scenario) = &bar.scenario {
                let right = notice_width + 260.0;
                let width = (ui.available_width() - right).max(40.0);
                ui.scope(|ui| {
                    ui.set_max_width(width);
                    ui.add(egui::Label::new(egui::RichText::new(scenario).strong()).truncate());
                });
                ui.separator();
            }
            self.clock_icon(ui, &bar.clock);
            ui.label(&bar.round);
            ui.separator();
            for deck in &bar.tile_decks {
                self.icon_value(ui, deck, false);
            }
            if let Some(notice) = &bar.notice {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(notice);
                });
            }
        });
        clicked
    }

    /// Soleil ou lune de la hauteur du texte, avec son info-bulle (spec 023).
    fn clock_icon(&mut self, ui: &mut egui::Ui, clock: &Clock) {
        let size = ui.text_style_height(&egui::TextStyle::Body) + 2.0;
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
        let fallback = match clock.time_of_day {
            TimeOfDay::Day => CLOCK_DAY,
            TimeOfDay::Night => CLOCK_NIGHT,
        };
        let pixels_per_point = ui.ctx().pixels_per_point();
        match self.paint_of(
            ui,
            time_of_day_image(clock.time_of_day),
            fallback,
            rect,
            pixels_per_point,
        ) {
            ImagePaint::Texture(texture) => {
                ui.painter().image(texture, rect, full_uv(), Color32::WHITE);
            }
            ImagePaint::Fallback(color) | ImagePaint::Pending(color) => {
                ui.painter()
                    .circle_filled(rect.center(), size / 2.0 - 1.0, color);
            }
        }
        response.on_hover_text(&clock.tooltip);
    }
}
