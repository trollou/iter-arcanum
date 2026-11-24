//! Écran de combat (spec 031) : phases, cartes d'ennemis, attaque ou parade en cours.

use super::widgets::shortcut_button;
use eframe::egui::{self, Color32, Frame, Sense, Stroke, Vec2, vec2};

use super::board::standing_image_rect;
use super::{CHOICE_STROKE, IterArcanumApp, RESISTANCE_TEXT, SWATCH_STROKE, TILE_STROKE, full_uv};
use crate::{CombatScreen, Command, EnemyCard, ImagePaint, StepState};

/// Écran de combat (spec 031), couleurs et tailles provisoires.
const COMBAT_BACKGROUND: Color32 = Color32::from_rgb(0x20, 0x21, 0x24);
const CURRENT_PHASE: Color32 = Color32::from_rgb(0x4D, 0x8B, 0xE0);
const DONE_PHASE: Color32 = Color32::from_rgb(0x5A, 0xA0, 0x5A);
const ENEMY_CARD_WIDTH: f32 = 340.0;
const ENEMY_TOKEN_SIZE: f32 = 72.0;

impl IterArcanumApp {
    /// Écran de combat (spec 031) : phases, cartes d'ennemis, attaque ou parade en cours ; la
    /// main et les unités sont dans le panneau du bas (spec 073, C37).
    pub(super) fn combat_area(&mut self, ui: &mut egui::Ui, combat: &CombatScreen) {
        let mut clicked = None;
        Frame::NONE
            .fill(COMBAT_BACKGROUND)
            .inner_margin(16.0)
            .show(ui, |ui| {
                ui.set_min_size(ui.available_size());
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.heading(&combat.title);
                        ui.add_space(12.0);
                        for (index, step) in combat.phases.iter().enumerate() {
                            if index > 0 {
                                ui.weak("›");
                            }
                            // Les polices d'egui n'ont pas de coche : elle est dessinée.
                            if step.state == StepState::Done {
                                check_mark(ui);
                            }
                            let text = egui::RichText::new(&step.label);
                            ui.label(match step.state {
                                StepState::Current => text.strong().color(CURRENT_PHASE),
                                StepState::Done => text,
                                StepState::Upcoming => text.weak(),
                            });
                        }
                        ui.add_space(12.0);
                        if let Some(button) = &combat.phase_button
                            && shortcut_button(
                                ui,
                                &button.label,
                                button.shortcut.as_deref(),
                                &self.catalog,
                            )
                            .clicked()
                        {
                            clicked = Some(button.command);
                        }
                        if shortcut_button(
                            ui,
                            &combat.board_toggle.label,
                            combat.board_toggle.shortcut.as_deref(),
                            &self.catalog,
                        )
                        .clicked()
                        {
                            clicked = Some(combat.board_toggle.command);
                        }
                    });
                    // Spec 044 : au donjon et au tombeau, les règles de Nuit et le combat seul, en
                    // tête de l'écran.
                    if let Some(note) = &combat.night_rules {
                        ui.colored_label(CHOICE_STROKE, note);
                    }
                    if let (Some(note), Some(_)) = (&combat.units_excluded, &combat.night_rules) {
                        ui.colored_label(CHOICE_STROKE, note);
                    }
                    // Spec 033, Q1 : l'invocation est un point de non-retour.
                    if let Some(warning) = &combat.summon_warning {
                        ui.colored_label(CHOICE_STROKE, warning);
                    }
                    // Spec 034 : assommé, jusqu'à la fin du combat.
                    if let Some(note) = &combat.knocked_out {
                        ui.colored_label(CHOICE_STROKE, note);
                    }
                    // Spec 042 : l'assignation des dégâts, à la place de « Phase suivante ».
                    for group in &combat.damage {
                        Frame::group(ui.style())
                            .stroke(Stroke::new(1.0, CHOICE_STROKE))
                            .show(ui, |ui| {
                                ui.strong(&group.text);
                                ui.horizontal_wrapped(|ui| {
                                    for button in &group.buttons {
                                        if shortcut_button(
                                            ui,
                                            &button.label,
                                            button.shortcut.as_deref(),
                                            &self.catalog,
                                        )
                                        .clicked()
                                        {
                                            clicked = Some(button.command);
                                        }
                                    }
                                });
                            });
                    }
                    ui.add_space(8.0);
                    ui.horizontal_wrapped(|ui| {
                        for enemy in &combat.enemies {
                            if let Some(command) = self.enemy_card(ui, enemy) {
                                clicked = Some(command);
                            }
                        }
                    });
                    if let Some(attack) = &combat.attack
                        && ui
                            .add_enabled(attack.command.is_some(), egui::Button::new(&attack.label))
                            .clicked()
                    {
                        clicked = attack.command;
                    }
                    if let Some(panel) = &combat.in_progress {
                        Frame::group(ui.style())
                            .stroke(Stroke::new(2.0, CHOICE_STROKE))
                            .show(ui, |ui| {
                                // Sur une ligne : la main ne descend pas à chaque carte jouée.
                                if !panel.cards.is_empty() {
                                    ui.add(egui::Label::new(panel.cards.join(", ")).wrap());
                                }
                                if let Some(needed) = &panel.needed {
                                    ui.label(needed);
                                }
                                // Spec 032 : ce que le groupe partage, l'élément qui pare, le calcul.
                                // Spec 033 : la parade doublée contre un ennemi vif.
                                for line in [&panel.shared, &panel.block_rule, &panel.swift]
                                    .into_iter()
                                    .flatten()
                                {
                                    ui.add(
                                        egui::Label::new(
                                            egui::RichText::new(line).color(RESISTANCE_TEXT),
                                        )
                                        .wrap(),
                                    );
                                }
                                if let Some(calculation) = &panel.calculation {
                                    ui.weak(calculation);
                                }
                                self.icon_value(ui, &panel.total, true);
                                // Spec 052, Q4 : un point converti par clic.
                                if let Some(button) = &panel.conversion
                                    && ui.button(&button.label).clicked()
                                {
                                    clicked = Some(button.command);
                                }
                                if let Some(button) = &panel.resolve
                                    && shortcut_button(
                                        ui,
                                        &button.label,
                                        button.shortcut.as_deref(),
                                        &self.catalog,
                                    )
                                    .clicked()
                                {
                                    clicked = Some(button.command);
                                }
                            });
                    }
                    ui.add_space(self.banner_height);
                });
            });
        if let Some(command) = clicked {
            self.client.command(command);
            ui.ctx().request_repaint();
        }
    }

    /// Carte d'un ennemi : son jeton, son nom, ses valeurs, son état, ses boutons et ce qui lui
    /// est arrivé dans ce combat.
    fn enemy_card(&mut self, ui: &mut egui::Ui, enemy: &EnemyCard) -> Option<Command> {
        let mut clicked = None;
        let stroke = if enemy.engaged {
            Stroke::new(3.0, CHOICE_STROKE)
        } else {
            Stroke::new(1.0, SWATCH_STROKE)
        };
        // Dans la rangée des ennemis : la carte se lit de haut en bas.
        Frame::group(ui.style()).stroke(stroke).show(ui, |ui| {
            ui.vertical(|ui| {
                ui.set_width(ENEMY_CARD_WIDTH);
                ui.horizontal_top(|ui| {
                    let (rect, token) =
                        ui.allocate_exact_size(Vec2::splat(ENEMY_TOKEN_SIZE), Sense::hover());
                    // Spec 070, C13 : la famille de l'ennemi au survol de sa figurine.
                    token.on_hover_text(&enemy.family);
                    let pixels_per_point = ui.ctx().pixels_per_point();
                    let tint = if enemy.defeated {
                        Color32::from_white_alpha(100)
                    } else {
                        Color32::WHITE
                    };
                    match self.paint_of(ui, enemy.image, enemy.fallback, rect, pixels_per_point) {
                        ImagePaint::Texture(texture) => {
                            // La figurine debout, à ses proportions (spec 075, C20b).
                            let size = ui
                                .ctx()
                                .tex_manager()
                                .read()
                                .meta(texture)
                                .map(|meta| meta.size);
                            let figure = standing_image_rect(rect, size);
                            ui.painter().image(texture, figure, full_uv(), tint);
                        }
                        ImagePaint::Fallback(color) | ImagePaint::Pending(color) => {
                            ui.painter().circle(
                                rect.center(),
                                rect.width() / 2.0,
                                color.gamma_multiply(if enemy.defeated { 0.4 } else { 1.0 }),
                                Stroke::new(1.5, TILE_STROKE),
                            );
                        }
                    }
                    ui.vertical(|ui| {
                        let name = egui::RichText::new(&enemy.name).strong();
                        ui.label(if enemy.defeated { name.weak() } else { name });
                        self.value_pieces(ui, &enemy.values, &enemy.resistances);
                        if let Some(abilities) = &enemy.abilities {
                            ui.label(abilities);
                        }

                        if let Some(status) = &enemy.status {
                            ui.weak(status);
                        }
                        // Spec 033 : la créature invoquée, puis l'enjeu de la parade.
                        if let Some(summoned) = &enemy.summoned {
                            Frame::group(ui.style())
                                .stroke(Stroke::new(1.5, SUMMONED_STROKE))
                                .show(ui, |ui| {
                                    ui.label(egui::RichText::new(&summoned.title).strong());
                                    self.value_pieces(ui, &summoned.values, &[]);
                                    if let Some(abilities) = &summoned.abilities {
                                        ui.label(abilities);
                                    }
                                    ui.weak(&summoned.note);
                                });
                        }
                        for line in &enemy.stakes {
                            ui.add(egui::Label::new(line).wrap());
                        }
                        if let Some(toggle) = &enemy.target {
                            let mut checked = toggle.checked;
                            if ui.checkbox(&mut checked, &toggle.label).changed() {
                                clicked = Some(toggle.command);
                            }
                        }
                        if let Some(button) = &enemy.block
                            && shortcut_button(
                                ui,
                                &button.label,
                                button.shortcut.as_deref(),
                                &self.catalog,
                            )
                            .clicked()
                        {
                            clicked = Some(button.command);
                        }
                    });
                });
                // Spec 070, C15 : replié par défaut, son titre compte les lignes.
                if let Some(title) = &enemy.history_title {
                    ui.separator();
                    egui::CollapsingHeader::new(egui::RichText::new(title).weak())
                        .id_salt(("combat_history", enemy.token.enemy(), enemy.token.copy()))
                        .default_open(false)
                        .show(ui, |ui| {
                            for line in &enemy.history {
                                ui.add(egui::Label::new(line).wrap());
                            }
                        });
                }
            })
        });
        clicked
    }
}

/// Coche d'une phase faite, de la hauteur du texte.
fn check_mark(ui: &mut egui::Ui) {
    let size = ui.text_style_height(&egui::TextStyle::Body);
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    let points = [
        rect.left_center() + vec2(size * 0.1, 0.0),
        rect.center_bottom() - vec2(size * 0.1, size * 0.2),
        rect.right_top() + vec2(-size * 0.1, size * 0.2),
    ];
    ui.painter()
        .line(points.to_vec(), Stroke::new(2.0, DONE_PHASE));
}

/// Cadre de la créature invoquée dans la carte de son Invocateur (spec 033).
const SUMMONED_STROKE: Color32 = Color32::from_rgb(0x8a, 0x6a, 0x40);
