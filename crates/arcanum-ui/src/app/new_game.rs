//! Écran de nouvelle partie en étapes (spec 078) : plein écran, en-tête avec la barre d'étapes,
//! liste à gauche, fiche à droite, volet « Avancé » et pied avec les boutons. Couleurs et tailles
//! de la maquette validée (issue privée 162).

use eframe::egui::{
    self, Color32, CornerRadius, Frame, Key, Margin, RichText, Sense, Stroke, Vec2, pos2, vec2,
};

use super::IterArcanumApp;
use super::widgets::card_color;
use crate::{
    Button, Command, HeroSheet, NewGameScreen, NewGameSheet, NewGameStep, OptionsSheet,
    SEED_MAX_CHARS, ScenarioSheet, SheetSection, hover_text,
};

const PAGE: Color32 = Color32::from_rgb(0x1B, 0x1C, 0x1F);
const SHEET: Color32 = Color32::from_rgb(0x24, 0x26, 0x2B);
const BOX: Color32 = Color32::from_rgb(0x1D, 0x1F, 0x23);
const BOX_STROKE: Color32 = Color32::from_rgb(0x30, 0x33, 0x3A);
const ROW_SELECTED: Color32 = Color32::from_rgb(0x2A, 0x2C, 0x32);
const ADVANCED_BAND: Color32 = Color32::from_rgb(0x20, 0x22, 0x26);
const RECAP_FILL: Color32 = Color32::from_rgb(0x26, 0x28, 0x2D);
const GOLD: Color32 = Color32::from_rgb(0xE0, 0xB6, 0x4A);
const ON_GOLD: Color32 = Color32::from_rgb(0x1B, 0x1C, 0x1F);
const TITLE: Color32 = Color32::from_rgb(0xF6, 0xF3, 0xEA);
const STRONG: Color32 = Color32::from_rgb(0xF1, 0xEF, 0xE9);
const BODY: Color32 = Color32::from_rgb(0xDD, 0xDA, 0xD2);
const FLAVOR: Color32 = Color32::from_rgb(0xCF, 0xC8, 0xB4);
const MUTED: Color32 = Color32::from_rgb(0xA9, 0xA7, 0xA0);
const CAPTION: Color32 = Color32::from_rgb(0x9A, 0x98, 0x90);
const INACTIVE: Color32 = Color32::from_rgb(0x6B, 0x6E, 0x76);
const MISSING_FILL: Color32 = Color32::from_rgb(0x2B, 0x2A, 0x24);
const MISSING_STROKE: Color32 = Color32::from_rgb(0x4A, 0x45, 0x32);
const MISSING_TEXT: Color32 = Color32::from_rgb(0xD9, 0xCF, 0xA8);
const INVALID: Color32 = Color32::from_rgb(0xE5, 0x5B, 0x4B);

/// Largeur de la liste de gauche (spec 078, Interface).
const LIST_WIDTH: f32 = 340.0;
/// Illustration ronde d'un héros dans la liste, et sur sa fiche (plus petite si la fenêtre est
/// basse, L3).
const LIST_PORTRAIT: f32 = 56.0;
const SHEET_PORTRAIT: f32 = 300.0;
const SEED_FIELD_WIDTH: f32 = 220.0;

impl IterArcanumApp {
    /// L'écran plein, dans `ui` (la place sous la barre de menu) ; `enabled` est faux quand la
    /// fenêtre de chargement est ouverte par-dessus (L2). Rend la commande cliquée.
    pub(super) fn new_game_page(
        &mut self,
        ui: &mut egui::Ui,
        screen: &NewGameScreen,
        enabled: bool,
    ) -> Option<Command> {
        let mut clicked = None;
        let page = Frame::NONE.fill(PAGE);
        egui::Panel::top("new_game_header")
            .resizable(false)
            .frame(page.inner_margin(Margin::symmetric(40, 20)))
            .show(ui, |ui| {
                ui.add_enabled_ui(enabled, |ui| header(ui, screen, &mut clicked));
            });
        egui::Panel::bottom("new_game_footer")
            .resizable(false)
            .frame(page.inner_margin(Margin::symmetric(40, 16)))
            .show(ui, |ui| {
                ui.add_enabled_ui(enabled, |ui| self.footer(ui, screen, &mut clicked));
            });
        if screen.advanced.open {
            egui::Panel::bottom("new_game_advanced")
                .resizable(false)
                .frame(
                    Frame::NONE
                        .fill(ADVANCED_BAND)
                        .inner_margin(Margin::symmetric(40, 14)),
                )
                .show(ui, |ui| {
                    ui.add_enabled_ui(enabled, |ui| self.seed_band(ui, screen, &mut clicked));
                });
        }
        egui::Panel::left("new_game_list")
            .resizable(false)
            .exact_size(LIST_WIDTH + 40.0)
            .show_separator_line(false)
            .frame(page.inner_margin(Margin {
                left: 40,
                right: 0,
                top: 24,
                bottom: 24,
            }))
            .show(ui, |ui| {
                ui.add_enabled_ui(enabled, |ui| self.list(ui, screen, &mut clicked));
            });
        egui::CentralPanel::default()
            .frame(page.inner_margin(Margin {
                left: 32,
                right: 40,
                top: 24,
                bottom: 24,
            }))
            .show(ui, |ui| {
                Frame::NONE
                    .fill(SHEET)
                    .stroke(Stroke::new(1.0, BOX_STROKE))
                    .corner_radius(CornerRadius::same(14))
                    .inner_margin(Margin::symmetric(36, 28))
                    .show(ui, |ui| {
                        ui.set_min_size(ui.available_size());
                        // L3 : la fiche défile, l'en-tête et le pied restent en place.
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .show(ui, |ui| match &screen.sheet {
                                NewGameSheet::Scenario(sheet) => scenario_sheet(ui, sheet),
                                NewGameSheet::Hero(sheet) => self.hero_sheet(ui, sheet),
                                NewGameSheet::Options(sheet) => options_help(ui, sheet),
                            });
                    });
            });
        clicked
    }

    fn list(&mut self, ui: &mut egui::Ui, screen: &NewGameScreen, clicked: &mut Option<Command>) {
        ui.label(
            RichText::new(screen.list_title.to_uppercase())
                .size(14.0)
                .strong()
                .color(CAPTION),
        );
        ui.add_space(8.0);
        let rows: Vec<Row> = match screen.step {
            NewGameStep::Scenario => screen
                .scenarios
                .iter()
                .map(|choice| Row {
                    name: &choice.name,
                    tagline: &choice.tagline,
                    selected: choice.selected,
                    accent: GOLD,
                    portrait: None,
                    command: choice.command,
                })
                .collect(),
            NewGameStep::Hero => screen
                .heroes
                .iter()
                .map(|choice| Row {
                    name: &choice.name,
                    tagline: &choice.tagline,
                    selected: choice.selected,
                    accent: crate::hero_color(choice.hero),
                    portrait: Some((choice.portrait, choice.image, choice.fallback)),
                    command: choice.command,
                })
                .collect(),
            // Spec 061 : la case du fantôme et la liste de son héros.
            NewGameStep::Options => {
                if let NewGameSheet::Options(sheet) = &screen.sheet {
                    options_list(ui, sheet, clicked);
                }
                return;
            }
        };
        for row in rows {
            let frame = Frame::NONE
                .fill(if row.selected { ROW_SELECTED } else { PAGE })
                .corner_radius(CornerRadius::same(10))
                .inner_margin(Margin::symmetric(16, 12));
            let response = frame
                .show(ui, |ui| {
                    ui.set_width(LIST_WIDTH - 32.0);
                    ui.horizontal(|ui| {
                        if let Some((portrait, image, fallback)) = row.portrait {
                            let (rect, _) =
                                ui.allocate_exact_size(Vec2::splat(LIST_PORTRAIT), Sense::hover());
                            self.paint_portrait(ui, portrait, image, fallback, rect);
                            ui.add_space(6.0);
                        }
                        ui.vertical(|ui| {
                            ui.label(RichText::new(row.name).size(18.0).strong().color(STRONG));
                            ui.label(RichText::new(row.tagline).size(14.0).color(MUTED));
                        });
                    });
                })
                .response;
            if row.selected {
                // Liseré à gauche : or pour un scénario, couleur du héros pour un héros.
                let rect = response.rect;
                ui.painter().rect_filled(
                    egui::Rect::from_min_max(rect.min, pos2(rect.min.x + 4.0, rect.max.y)),
                    CornerRadius::same(2),
                    row.accent,
                );
            }
            let response = ui.interact(response.rect, response.id.with("row"), Sense::click());
            if response.clicked() {
                *clicked = Some(row.command);
            }
            response.on_hover_cursor(egui::CursorIcon::PointingHand);
            ui.add_space(6.0);
        }
    }

    fn hero_sheet(&mut self, ui: &mut egui::Ui, sheet: &HeroSheet) {
        // L3 : l'illustration se réduit quand la fenêtre est basse.
        let portrait = SHEET_PORTRAIT.min(ui.ctx().content_rect().height() * 0.3);
        ui.horizontal_top(|ui| {
            let (rect, _) = ui.allocate_exact_size(Vec2::splat(portrait), Sense::hover());
            self.paint_portrait(ui, sheet.portrait, sheet.image, sheet.fallback, rect);
            ui.add_space(28.0);
            ui.vertical(|ui| {
                ui.label(RichText::new(&sheet.name).size(34.0).strong().color(TITLE));
                ui.add_space(4.0);
                ui.label(
                    RichText::new(&sheet.description)
                        .size(18.0)
                        .italics()
                        .color(FLAVOR),
                );
                ui.add_space(16.0);
                section(ui, &sheet.style);
                ui.add_space(16.0);
                ui.label(
                    RichText::new(&sheet.own_cards_title)
                        .size(17.0)
                        .strong()
                        .color(GOLD),
                );
                ui.add_space(6.0);
                ui.columns(sheet.own_cards.len().max(1), |columns| {
                    for (ui, card) in columns.iter_mut().zip(&sheet.own_cards) {
                        let response = Frame::NONE
                            .fill(BOX)
                            .stroke(Stroke::new(1.0, BOX_STROKE))
                            .corner_radius(CornerRadius::same(12))
                            .inner_margin(Margin {
                                left: 16,
                                right: 16,
                                top: 20,
                                bottom: 14,
                            })
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                // Hors de la disposition justifiée des colonnes, qui étirerait
                                // les espaces du texte.
                                ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                                    ui.horizontal_wrapped(|ui| {
                                        ui.label(
                                            RichText::new(&card.name)
                                                .size(18.0)
                                                .strong()
                                                .color(STRONG),
                                        );
                                        ui.label(
                                            RichText::new(&card.replaces).size(13.0).color(MUTED),
                                        );
                                    });
                                    ui.add_space(6.0);
                                    effect_block(ui, &card.basic_title, &card.basic);
                                    ui.add_space(6.0);
                                    effect_block(ui, &card.strong_title, &card.strong);
                                });
                            })
                            .response;
                        // Bandeau de la couleur de la carte, en haut.
                        let rect = response.rect;
                        ui.painter().rect_filled(
                            egui::Rect::from_min_max(rect.min, pos2(rect.max.x, rect.min.y + 8.0)),
                            CornerRadius {
                                nw: 12,
                                ne: 12,
                                sw: 0,
                                se: 0,
                            },
                            card_color(card.color),
                        );
                    }
                });
            });
        });
    }

    /// Le portrait généré du héros, aux coins arrondis (C9b) ; sans lui, son médaillon SVG.
    fn paint_portrait(
        &mut self,
        ui: &egui::Ui,
        portrait: crate::Illustration,
        image: crate::ImageId,
        fallback: Color32,
        rect: egui::Rect,
    ) {
        match self.illustration_texture(ui.ctx(), portrait) {
            Some(texture) => {
                // Arrondi proportionnel : 10 points pour la liste, davantage en grand.
                let radius = (rect.width() / 6.0).clamp(8.0, 24.0);
                egui::Image::from_texture((texture, rect.size()))
                    .corner_radius(CornerRadius::same(radius as u8))
                    .paint_at(ui, rect);
            }
            None => self.paint_image(ui, image, fallback, rect),
        }
    }

    fn seed_band(
        &mut self,
        ui: &mut egui::Ui,
        screen: &NewGameScreen,
        clicked: &mut Option<Command>,
    ) {
        ui.horizontal(|ui| {
            ui.label(RichText::new(&screen.seed_label).size(15.0).color(BODY));
            let mut seed = self.client.seed_text().unwrap_or_default().to_string();
            let response = ui.add(
                egui::TextEdit::singleline(&mut seed)
                    .char_limit(SEED_MAX_CHARS)
                    .font(egui::FontId::proportional(16.0))
                    .desired_width(SEED_FIELD_WIDTH),
            );
            if response.changed() {
                self.client.set_seed_text(&seed);
            }
            // Entrée depuis le champ de la graine commence aussi la partie (spec 065).
            if response.lost_focus() && ui.input(|input| input.key_pressed(Key::Enter)) {
                *clicked = Some(screen.start.command);
            }
            ui.label(RichText::new(&screen.seed_hint).size(14.0).color(MUTED));
        });
        if let Some(invalid) = &screen.seed_invalid {
            ui.label(RichText::new(invalid).size(14.0).color(INVALID));
        }
    }

    fn footer(&mut self, ui: &mut egui::Ui, screen: &NewGameScreen, clicked: &mut Option<Command>) {
        ui.horizontal(|ui| {
            for button in [&screen.continue_game, &screen.load, &screen.preferences]
                .into_iter()
                .flatten()
            {
                if self.page_button(ui, button, true, false).clicked() {
                    *clicked = Some(button.command);
                }
            }
            ui.vertical(|ui| {
                if let Some(warning) = &screen.abandon_warning {
                    ui.label(RichText::new(warning).size(15.0).color(BODY));
                }
                if let Some(notice) = &screen.notice {
                    ui.label(RichText::new(notice).size(15.0).color(INVALID));
                }
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // De droite à gauche : Suivant, Commencer, Annuler, Avancé.
                if let Some(next) = &screen.next
                    && self.page_button(ui, next, true, true).clicked()
                {
                    *clicked = Some(next.command);
                }
                let start = Button {
                    label: screen.start.label.clone(),
                    command: screen.start.command,
                    shortcut: screen.start.shortcut.clone(),
                };
                // Bouton principal : Commencer à l'étape Héros, Suivant à l'étape Scénario.
                if self
                    .page_button(ui, &start, screen.start.enabled, screen.next.is_none())
                    .clicked()
                {
                    *clicked = Some(start.command);
                }
                if let Some(cancel) = &screen.cancel
                    && self.page_button(ui, cancel, true, false).clicked()
                {
                    *clicked = Some(cancel.command);
                }
                ui.add_space(12.0);
                // L'en-tête repliable dessine sa flèche ; une place fixe le garde à droite.
                let text_width = ui
                    .painter()
                    .layout_no_wrap(
                        screen.advanced.label.clone(),
                        egui::FontId::proportional(15.0),
                        BODY,
                    )
                    .size()
                    .x;
                let spacing = ui.spacing();
                let width = text_width + spacing.icon_width + spacing.icon_spacing + 16.0;
                ui.add_space(8.0);
                let header = ui
                    .allocate_ui_with_layout(
                        vec2(width, 24.0),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            egui::CollapsingHeader::new(
                                RichText::new(&screen.advanced.label).size(15.0).color(BODY),
                            )
                            .id_salt("new_game_advanced_header")
                            .open(Some(screen.advanced.open))
                            .show(ui, |_| {})
                            .header_response
                        },
                    )
                    .inner;
                if header.clicked() && (!screen.advanced.open || screen.advanced.can_close) {
                    *clicked = Some(screen.advanced.command);
                }
            });
        });
    }

    /// Un bouton du pied, texte agrandi ; `primary` : fond or.
    fn page_button(
        &self,
        ui: &mut egui::Ui,
        button: &Button,
        enabled: bool,
        primary: bool,
    ) -> egui::Response {
        let text = RichText::new(&button.label).size(17.0);
        let widget = if primary {
            egui::Button::new(text.strong().color(ON_GOLD)).fill(GOLD)
        } else {
            egui::Button::new(text)
        }
        .min_size(vec2(0.0, 40.0));
        let response = ui.add_enabled(enabled, widget);
        match hover_text(None, button.shortcut.as_deref(), &self.catalog) {
            Some(text) => response
                .on_hover_text(text.clone())
                .on_disabled_hover_text(text),
            None => response,
        }
    }
}

/// Une ligne de la liste de gauche.
struct Row<'a> {
    name: &'a str,
    tagline: &'a str,
    selected: bool,
    accent: Color32,
    portrait: Option<(crate::Illustration, crate::ImageId, Color32)>,
    command: Command,
}

/// En-tête : titre, barre d'étapes, récapitulatif à droite.
fn header(ui: &mut egui::Ui, screen: &NewGameScreen, clicked: &mut Option<Command>) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(&screen.title)
                .size(28.0)
                .strong()
                .color(TITLE),
        );
        ui.add_space(32.0);
        for (index, tab) in screen.steps.iter().enumerate() {
            if index > 0 {
                // Séparateur dessiné, pas écrit (spec 078, Libellés).
                let (rect, _) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::hover());
                let c = rect.center();
                ui.painter().line_segment(
                    [pos2(c.x - 3.0, c.y - 5.0), pos2(c.x + 3.0, c.y)],
                    Stroke::new(1.5, INACTIVE),
                );
                ui.painter().line_segment(
                    [pos2(c.x + 3.0, c.y), pos2(c.x - 3.0, c.y + 5.0)],
                    Stroke::new(1.5, INACTIVE),
                );
            }
            let color = if tab.active { GOLD } else { BODY };
            let label = RichText::new(format!("{}  {}", tab.number, tab.label))
                .size(16.0)
                .strong()
                .color(if tab.active { ON_GOLD } else { color });
            let button = egui::Button::new(label)
                .corner_radius(CornerRadius::same(20))
                .min_size(vec2(0.0, 36.0));
            let button = if tab.active {
                button.fill(GOLD)
            } else {
                button.fill(PAGE).stroke(Stroke::new(1.0, BOX_STROKE))
            };
            if ui.add(button).clicked() {
                *clicked = Some(tab.command);
            }
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            Frame::NONE
                .fill(RECAP_FILL)
                .corner_radius(CornerRadius::same(8))
                .inner_margin(Margin::symmetric(14, 8))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(&screen.recap)
                            .size(15.0)
                            .strong()
                            .color(STRONG),
                    );
                });
            ui.label(RichText::new(&screen.recap_label).size(15.0).color(MUTED));
        });
    });
}

/// L'étape Options à gauche (spec 061, C41) : la case du fantôme, puis la liste déroulante de son
/// héros, grisée sans fantôme ; ou « Pas d'option pour ce scénario. ».
fn options_list(ui: &mut egui::Ui, sheet: &OptionsSheet, clicked: &mut Option<Command>) {
    if let Some(none) = &sheet.none {
        ui.label(RichText::new(none).size(16.0).color(MUTED));
    }
    let Some(dummy) = &sheet.dummy else {
        return;
    };
    let mut checked = dummy.checked;
    // Spec 064, Q3 : fantôme obligatoire, le texte à la place de la case.
    if let Some(required) = &dummy.required {
        ui.add(egui::Label::new(RichText::new(required).size(16.0).color(STRONG)).wrap());
    } else if ui
        .checkbox(
            &mut checked,
            RichText::new(&dummy.label)
                .size(18.0)
                .strong()
                .color(STRONG),
        )
        .changed()
    {
        *clicked = Some(dummy.command);
    }
    ui.add_space(12.0);
    ui.add_enabled_ui(dummy.hero_enabled, |ui| {
        ui.label(RichText::new(&dummy.hero_label).size(14.0).color(CAPTION));
        egui::ComboBox::from_id_salt("dummy_hero")
            .selected_text(&dummy.hero_selected)
            .width(LIST_WIDTH - 32.0)
            .show_ui(ui, |ui| {
                for item in &dummy.heroes {
                    if ui.selectable_label(item.selected, &item.label).clicked() {
                        *clicked = Some(item.command);
                    }
                }
            });
    });
}

/// L'étape Options à droite : ce que fait le fantôme (spec 061, Interface).
fn options_help(ui: &mut egui::Ui, sheet: &OptionsSheet) {
    if let Some(dummy) = &sheet.dummy {
        ui.label(RichText::new(&dummy.label).size(32.0).strong().color(TITLE));
        ui.add_space(12.0);
        ui.add(egui::Label::new(RichText::new(&dummy.help).size(17.0).color(BODY)).wrap());
    }
}

fn scenario_sheet(ui: &mut egui::Ui, sheet: &ScenarioSheet) {
    ui.label(RichText::new(&sheet.name).size(32.0).strong().color(TITLE));
    ui.add_space(6.0);
    ui.label(
        RichText::new(&sheet.flavor)
            .size(18.0)
            .italics()
            .color(FLAVOR),
    );
    ui.add_space(18.0);
    ui.columns(sheet.facts.len().max(1), |columns| {
        for (ui, fact) in columns.iter_mut().zip(&sheet.facts) {
            Frame::NONE
                .fill(BOX)
                .stroke(Stroke::new(1.0, BOX_STROKE))
                .corner_radius(CornerRadius::same(10))
                .inner_margin(Margin::symmetric(16, 14))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                        ui.label(
                            RichText::new(fact.label.to_uppercase())
                                .size(13.0)
                                .strong()
                                .color(CAPTION),
                        );
                        ui.label(RichText::new(&fact.value).size(17.0).color(STRONG));
                    });
                });
        }
    });
    ui.add_space(18.0);
    // Rubriques sur deux colonnes : Objectif et Fin de partie à gauche, Règles spéciales et
    // Décompte à droite, dans l'ordre de lecture ligne par ligne.
    for pair in sheet.sections.chunks(2) {
        ui.columns(2, |columns| {
            for (ui, section_data) in columns.iter_mut().zip(pair) {
                section(ui, section_data);
            }
        });
        ui.add_space(18.0);
    }
    if let Some(missing) = &sheet.missing {
        Frame::NONE
            .fill(MISSING_FILL)
            .stroke(Stroke::new(1.0, MISSING_STROKE))
            .corner_radius(CornerRadius::same(10))
            .inner_margin(Margin::symmetric(16, 12))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(RichText::new(missing).size(15.0).color(MISSING_TEXT));
            });
    }
}

/// Une rubrique : titre en or, puis ses puces.
fn section(ui: &mut egui::Ui, section: &SheetSection) {
    ui.label(
        RichText::new(&section.title)
            .size(17.0)
            .strong()
            .color(GOLD),
    );
    ui.add_space(4.0);
    for bullet in &section.bullets {
        ui.horizontal_top(|ui| {
            // Puce dessinée : aucun caractère nouveau dans la police.
            let (rect, _) = ui.allocate_exact_size(vec2(12.0, 20.0), Sense::hover());
            ui.painter()
                .circle_filled(pos2(rect.center().x, rect.min.y + 11.0), 2.5, BODY);
            ui.add(egui::Label::new(RichText::new(bullet).size(16.0).color(BODY)).wrap());
        });
    }
}

/// Un effet de carte propre : titre en petites capitales, puis le texte.
fn effect_block(ui: &mut egui::Ui, title: &str, text: &str) {
    ui.label(
        RichText::new(title.to_uppercase())
            .size(12.0)
            .strong()
            .color(CAPTION),
    );
    ui.add(egui::Label::new(RichText::new(text).size(15.0).color(BODY)).wrap());
}
