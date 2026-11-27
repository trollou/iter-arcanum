//! Panneau du bas (spec 073) : le bandeau du choix en cours, puis les sections Unités et Main en
//! cartes posées en rangée, qui se chevauchent quand la place manque et se poussent au survol.
//! La disposition est calculée par `layout.rs` (testé sans egui) ; ici, seulement le dessin.

use eframe::egui::{
    self, Align, Color32, Frame, Id, Layout, Rect, Sense, Stroke, UiBuilder, Vec2,
    load::SizedTexture, pos2, vec2,
};

use super::icons::{icon_size, icon_tooltip, marked_label};
use super::widgets::{mana_color, reclaim_menu, shortcut_button, strong_button};
use super::{CHOICE_STROKE, IterArcanumApp, RESISTANCE_TEXT, SWATCH_STROKE};
use crate::{
    CARD_GAP, CARD_HEIGHT, CARD_WIDTH, Command, DiscardWindow, EffectButton, EffectFace,
    GameScreen, HandCard, SECTION_GAP, SKILLS_WIDTH, SkillsSection, UnitLine, band_uv,
    bottom_section_widths, card_height, crystal_image, fan_layout, hover_text, row_width,
};
use crate::{CardInfo, Icon};
use arcanum_core::ManaColor;

/// Durée du glissement d'une carte vers sa place (spec 073, « Animation »).
const SLIDE_SECONDS: f32 = 0.15;
/// Largeur d'une section Unités vide : « Unités : 0/1 » et « Aucune unité ».
const EMPTY_UNITS_WIDTH: f32 = 120.0;
/// Icône d'un bouton d'effet, à la hauteur du texte d'un bouton.
const EFFECT_ICON: f32 = 16.0;
/// Marge horizontale d'un bouton d'effet, et écart entre sa pastille, « : » et le bouton.
const EFFECT_PADDING: f32 = 3.0;
const EFFECT_GAP: f32 = 1.0;
/// Le « i » d'une carte, à la taille des icônes de côté (spec #11).
const INFO_ICON: f32 = 16.0;
/// Largeur maximale de la fenêtre du texte d'une carte (spec #11).
const INFO_WIDTH: f32 = 260.0;
/// Hauteur de la bande d'illustration d'une carte posée, sur toute sa largeur (spec 077).
const ART_BAND_HEIGHT: f32 = 84.0;
/// Côté de l'image d'une Blessure, seule sous son nom (spec 077).
const WOUND_ART: f32 = 180.0;
/// Fond d'une carte : opaque, pour cacher celle qu'elle couvre.
const CARD_FILL: Color32 = Color32::from_rgb(0x2A, 0x2B, 0x2F);
/// Marge intérieure et bordure d'une carte : le contenu prend la place qui reste, pour que la
/// carte tienne dans `CARD_WIDTH` × `CARD_HEIGHT` sans que son bord droit soit coupé.
const CARD_MARGIN: f32 = 6.0;
const CARD_STROKE: f32 = 1.0;

/// Le cadre d'une carte posée, dans le panneau du bas, la fenêtre de la défausse et les fenêtres
/// des offres et des choix (spec #32).
pub(super) fn card_frame() -> Frame {
    Frame::new()
        .fill(CARD_FILL)
        .stroke(Stroke::new(CARD_STROKE, SWATCH_STROKE))
        .corner_radius(6.0)
        .inner_margin(CARD_MARGIN)
}

/// Taille du contenu d'une carte, cadre déduit.
pub(super) fn card_content_size() -> Vec2 {
    vec2(CARD_WIDTH, CARD_HEIGHT) - Vec2::splat(2.0 * (CARD_MARGIN + CARD_STROKE))
}

/// L'illustration en bande sur toute la largeur de la carte, recadrée au centre selon la taille
/// de l'image (spec 077, C2 ; spec #9).
pub(super) fn art_band(ui: &mut egui::Ui, texture: SizedTexture) {
    let width = ui.available_width();
    let uv = band_uv(width, ART_BAND_HEIGHT, texture.size.x, texture.size.y);
    ui.add(
        egui::Image::new(SizedTexture::new(texture.id, vec2(width, ART_BAND_HEIGHT)))
            .uv(Rect::from_min_max(
                pos2(uv.left, uv.top),
                pos2(uv.right, uv.bottom),
            ))
            .corner_radius(3.0),
    );
}

impl IterArcanumApp {
    /// Le panneau du bas : Unités et Main. Le bandeau du choix en cours est posé à part, sur le
    /// bas du plateau (`choice_overlay`).
    pub(super) fn bottom_panel(&mut self, ui: &mut egui::Ui, screen: &GameScreen) {
        let Some(game) = self.client.game() else {
            return;
        };
        // Spec 041 et 044 : pendant un incendie ou au donjon, les unités ne combattent pas.
        let units_excluded = game
            .combat_screen(&self.catalog)
            .and_then(|combat| combat.units_excluded);
        let mut clicked = None;
        // Spec 051 : la section Compétences garde sa largeur, Unités et Main se partagent le reste.
        // Une section Unités vide ne prend que la place de son titre (spec 073).
        let units_width = if screen.units.units.is_empty() {
            EMPTY_UNITS_WIDTH
        } else {
            row_width(screen.units.units.len())
        };
        let widths = bottom_section_widths(
            units_width,
            row_width(screen.hand.len()),
            screen.skills.is_some(),
            ui.available_width(),
            ui.spacing().item_spacing.x,
        );
        ui.horizontal_top(|ui| {
            ui.vertical(|ui| {
                ui.set_width(widths[0]);
                ui.strong(&screen.units.header);
                // Spec 058, C30 : l'emplacement de Loyauté sans faille, libre ou occupé.
                if let Some(token) = &screen.units.bonds_token {
                    ui.add(egui::Label::new(token).wrap());
                }
                if let Some(note) = &units_excluded {
                    ui.add(egui::Label::new(note).wrap());
                } else if let Some(empty) = &screen.units.empty {
                    ui.weak(empty);
                } else {
                    let units = &screen.units.units;
                    let command =
                        self.fan_row(ui, "units_row", units.len(), widths[0], |app, ui, i| {
                            app.unit_tile(ui, &units[i])
                        });
                    clicked = clicked.or(command);
                }
            });
            ui.add_space(SECTION_GAP);
            ui.vertical(|ui| {
                ui.set_width(widths[1].max(EMPTY_UNITS_WIDTH));
                ui.strong(&screen.hand_title);
                if let Some(empty) = &screen.hand_empty {
                    ui.weak(empty);
                } else {
                    let hand = &screen.hand;
                    let command =
                        self.fan_row(ui, "hand_row", hand.len(), widths[1], |app, ui, i| {
                            app.card_tile(ui, &hand[i], &screen.sideways_title)
                        });
                    clicked = clicked.or(command);
                }
            });
            if let Some(skills) = &screen.skills {
                ui.add_space(SECTION_GAP);
                clicked = self.skills_column(ui, skills).or(clicked);
            }
        });
        if let Some(command) = clicked {
            self.client.command(command);
            ui.ctx().request_repaint();
        }
    }

    /// Une rangée de `n` cartes dans `width` points : chevauchement, survol et glissement
    /// (spec 073, C10 à C12). Une carte couverte ne se clique pas sous ses voisines (L2). Une
    /// carte trop haute est coupée et marquée « … » ; survolée, elle grandit vers le haut
    /// (C12 bis), sa hauteur mesurée au dessin précédent.
    fn fan_row(
        &mut self,
        ui: &mut egui::Ui,
        salt: &str,
        n: usize,
        width: f32,
        mut draw: impl FnMut(&mut Self, &mut egui::Ui, usize) -> Option<Command>,
    ) -> Option<Command> {
        let id = Id::unique(salt);
        let heights_id = id.with("heights");
        let (rect, _) = ui.allocate_exact_size(vec2(width, CARD_HEIGHT), Sense::hover());
        // La carte survolée au dessin précédent, si la rangée l'a encore : une carte jouée la
        // raccourcit (plantage vu par Mathieu, carte jouée de côté).
        let previous: Option<usize> = ui
            .data(|data| data.get_temp(id))
            .flatten()
            .filter(|&k| k < n);
        let mut heights: Vec<f32> = ui
            .data(|data| data.get_temp(heights_id))
            .unwrap_or_default();
        heights.resize(n, CARD_HEIGHT);
        let ceiling = rect.bottom() - ui.ctx().content_rect().top();
        let lifted = |index: usize, x: f32| {
            let height = card_height(heights[index], ceiling);
            Rect::from_min_max(
                pos2(rect.left() + x, rect.bottom() - height),
                pos2(rect.left() + x + CARD_WIDTH, rect.bottom()),
            )
        };
        let before = fan_layout(n, width, previous);
        // La carte agrandie reste survolée tant que le pointeur est sur elle, même au-dessus de
        // la rangée. Aucune ne s'agrandit pendant qu'une liste déroulante est ouverte : son aire,
        // de même ordre et plus récente, couvrirait la liste (issue #24, Euphorie cristalline).
        let popup_open = egui::Popup::is_any_open(ui.ctx());
        let hovered = ui
            .ctx()
            .pointer_hover_pos()
            .filter(|_| !popup_open)
            .and_then(|pointer| {
                if rect.contains(pointer) {
                    before.card_at(pointer.x - rect.left())
                } else {
                    previous.filter(|&k| lifted(k, before.positions[k]).contains(pointer))
                }
            });
        if hovered != previous {
            ui.data_mut(|data| data.insert_temp(id, hovered));
            ui.ctx().request_repaint();
        }
        let layout = fan_layout(n, width, hovered);
        let mut clicked = None;
        let mut measured = heights.clone();
        for &index in &layout.draw_order {
            let x = ui.ctx().animate_value_with_time(
                id.with(index),
                layout.positions[index],
                SLIDE_SECONDS,
            );
            let grows = hovered == Some(index) && heights[index] > CARD_HEIGHT;
            let (command, height) = if grows {
                let card = lifted(index, x);
                egui::Area::new(id.with(("lifted", index)))
                    .order(egui::Order::Foreground)
                    .fixed_pos(card.min)
                    .show(ui.ctx(), |ui| {
                        self.card_in(ui, card, id.with(("card", index)), |app, ui| {
                            draw(app, ui, index)
                        })
                    })
                    .inner
            } else {
                let card =
                    Rect::from_min_size(rect.min + vec2(x, 0.0), vec2(CARD_WIDTH, CARD_HEIGHT));
                let mut child = ui.new_child(
                    UiBuilder::new()
                        .id_salt((salt, index))
                        .max_rect(card)
                        .layout(Layout::top_down(Align::Min)),
                );
                child.set_clip_rect(card.intersect(ui.clip_rect()));
                self.card_in(&mut child, card, id.with(("card", index)), |app, ui| {
                    draw(app, ui, index)
                })
            };
            clicked = clicked.or(command);
            measured[index] = height;
        }
        if measured != heights {
            ui.data_mut(|data| data.insert_temp(heights_id, measured));
            ui.ctx().request_repaint();
        }
        clicked
    }

    /// Une carte dans `card` : fond, bordure, contenu coupé à la carte et « … » s'il dépasse.
    /// Rend la commande cliquée et la hauteur que demande le contenu, cadre compris.
    fn card_in(
        &mut self,
        ui: &mut egui::Ui,
        card: Rect,
        id: Id,
        draw: impl FnOnce(&mut Self, &mut egui::Ui) -> Option<Command>,
    ) -> (Option<Command>, f32) {
        // Le fond de la carte prend les clics : ceux des cartes qu'elle couvre ne passent pas.
        ui.interact(card, id, Sense::click());
        let clip = card.intersect(ui.clip_rect());
        ui.painter().with_clip_rect(clip).rect(
            card,
            6.0,
            CARD_FILL,
            Stroke::new(CARD_STROKE, SWATCH_STROKE),
            egui::StrokeKind::Inside,
        );
        let frame = CARD_MARGIN + CARD_STROKE;
        let inner = card.shrink(frame);
        // Hauteur libre pour mesurer le contenu ; seul ce qui tient dans la carte se voit.
        let mut content = ui.new_child(
            UiBuilder::new()
                .id_salt(id.with("content"))
                .max_rect(Rect::from_min_size(
                    inner.min,
                    vec2(inner.width(), f32::from(u16::MAX)),
                ))
                .layout(Layout::top_down(Align::Min)),
        );
        content.set_clip_rect(inner.intersect(clip));
        content.set_width(inner.width());
        let command = draw(self, &mut content);
        let height = content.min_rect().height() + 2.0 * frame;
        if height > card.height() + 0.5 {
            let painter = ui.painter().with_clip_rect(clip);
            let galley = painter.layout_no_wrap(
                "…".to_owned(),
                egui::FontId::proportional(16.0),
                ui.visuals().strong_text_color(),
            );
            let corner = card.right_bottom() - vec2(frame, frame);
            let mark = Rect::from_min_max(corner - galley.size() - vec2(4.0, 0.0), corner);
            painter.rect_filled(mark, 3.0, CARD_FILL);
            painter.galley(mark.min + vec2(2.0, 0.0), galley, Color32::PLACEHOLDER);
        }
        (command, height)
    }

    /// Une carte de la main posée (spec 073, « Carte de la main ») : en-tête, « i » et encart de
    /// côté, bande d'illustration (spec 077), boutons de base, séparation du renforcé et ses
    /// boutons (spec #11).
    pub(super) fn card_tile(
        &mut self,
        ui: &mut egui::Ui,
        card: &HandCard,
        sideways_title: &str,
    ) -> Option<Command> {
        let mut clicked = None;
        ui.spacing_mut().item_spacing.y = 3.0;
        // L'encart de côté, ou la case « Défausser » en fin du tour (issue privée 282), se pose d'abord à
        // droite : le nom, tronqué au besoin, prend la place qui reste à gauche.
        let row = vec2(ui.available_width(), ui.spacing().interact_size.y);
        // Issue #69 : les boutons du mode choix (« Défausser », « Détruire ») se posent en haut à
        // droite, comme la case « Défausser » de la fin du tour et « Prendre » des fenêtres.
        let choosing = card.basic_effects.is_empty()
            && card.strong_effects.is_empty()
            && card.unit_plays.is_empty()
            && !card.plays.is_empty();
        ui.allocate_ui_with_layout(row, Layout::right_to_left(Align::Center), |ui| {
            if choosing {
                for button in &card.plays {
                    if shortcut_button(ui, &button.label, button.shortcut.as_deref(), &self.catalog)
                        .clicked()
                    {
                        clicked = Some(button.command);
                    }
                }
            }
            if !card.sideways.is_empty() {
                clicked = self
                    .sideways_buttons(ui, sideways_title, &card.sideways)
                    .or(clicked);
            }
            if let Some(toggle) = &card.discard_toggle {
                let mut checked = toggle.checked;
                if ui.checkbox(&mut checked, &toggle.label).changed() {
                    clicked = Some(toggle.command);
                }
            }
            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                if let Some(color) = card.color {
                    mana_swatch(ui, ManaColor::Basic(color));
                }
                // Spec #11 : le « i » juste après le nom, qui se tronque avant lui (L1) ; posé à
                // droite, il serait caché par la carte voisine qui chevauche.
                let reserved = if card.info.is_some() {
                    INFO_ICON + 2.0 * ui.spacing().button_padding.x + ui.spacing().item_spacing.x
                } else {
                    0.0
                };
                // Une Blessure ne se joue pas : grisée (spec 031).
                let name = egui::RichText::new(&card.name);
                ui.scope(|ui| {
                    ui.set_max_width((ui.available_width() - reserved).max(0.0));
                    ui.add(
                        // Spec 055 : un artefact n'a pas de couleur, mais se joue.
                        egui::Label::new(
                            if card.color.is_some() || card.card.card().is_artifact() {
                                name.strong()
                            } else {
                                name.weak()
                            },
                        )
                        .truncate(),
                    );
                });
                if let Some(info) = &card.info {
                    self.info_button(ui, card, info);
                }
            });
        });
        // Spec 077 : l'image en bande sous le nom ; celle d'une Blessure, seule, en grand.
        if let Some(texture) = self.illustration_texture(ui.ctx(), card.illustration) {
            if card.art_fills_card {
                // Place réservée à la hauteur exacte de l'image, centrée dans la largeur.
                let (row, _) =
                    ui.allocate_exact_size(vec2(ui.available_width(), WOUND_ART), Sense::hover());
                let image = Rect::from_center_size(row.center(), Vec2::splat(WOUND_ART));
                egui::Image::new(SizedTexture::new(texture.id, Vec2::splat(WOUND_ART)))
                    .paint_at(ui, image);
            } else {
                art_band(ui, texture);
            }
        }
        // Spec #11 : seul un artefact écrit son texte ; les boutons portent celui des autres (Q1).
        if card.show_text && !card.effect.is_empty() {
            self.marked_label(ui, &card.effect, egui::RichText::new);
        }
        if let Some(note) = &card.combat_note {
            ui.colored_label(RESISTANCE_TEXT, note);
        }
        clicked = self.effect_buttons(ui, &card.basic_effects).or(clicked);
        // Mode choix (défausse, Concentration…) : son bouton est dans l'en-tête, à la place des
        // boutons d'effet.
        if choosing {
            for button in card.paid_plays.iter().chain(&card.strong_plays) {
                clicked = strong_button(ui, button).or(clicked);
            }
        }
        if !card.strong_text.is_empty() && (card.color.is_some() || card.card.card().is_artifact())
        {
            strong_separator(
                ui,
                card.card.card().is_spell(),
                &card.strong_heading,
                &card.strong_colors,
            );
            if card.show_text {
                self.marked_label(ui, &card.strong_text, super::icons::weak_text);
            }
            clicked = self.effect_buttons(ui, &card.strong_effects).or(clicked);
        }
        // Rajeunissement : une unité à rendre disponible par bouton (spec 042).
        for button in &card.unit_plays {
            let label = crate::plain_text(&button.label, &self.catalog);
            if ui.add(egui::Button::new(label).wrap()).clicked() {
                clicked = Some(button.command);
            }
        }
        clicked
    }

    /// Le « i » d'une carte et la fenêtre de son texte complet (spec #11, C1 à C3, L2) : au clic,
    /// au-dessus du bouton ; une seule ouverte à la fois, fermée par un clic ailleurs, Échap ou un
    /// second clic.
    fn info_button(&mut self, ui: &mut egui::Ui, card: &HandCard, info: &CardInfo) {
        let tooltip = self.catalog.text(Icon::Info.name_key());
        let widget = match self.icon_texture(ui, Icon::Info.image(), INFO_ICON) {
            Some(texture) => egui::Button::image(egui::Image::from_texture(SizedTexture::new(
                texture,
                Vec2::splat(INFO_ICON),
            )))
            .frame(false),
            None => egui::Button::new(tooltip.as_str()),
        };
        let response = ui.add(widget).on_hover_text(&tooltip);
        // Un identifiant par carte, pas par widget : la fenêtre reste ouverte quand la carte
        // agrandie au survol retombe dans la rangée (spec 073, C12 bis).
        egui::Popup::from_toggle_button_response(&response)
            .id(Id::unique(("card_info", card.card)))
            .align(egui::RectAlign::TOP_START)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
            .show(|ui| {
                ui.set_max_width(INFO_WIDTH);
                ui.strong(&info.name);
                marked_label(ui, &info.basic, egui::RichText::new, &self.catalog);
                ui.separator();
                ui.strong(&info.strong_title);
                marked_label(ui, &info.strong, egui::RichText::new, &self.catalog);
            });
    }

    /// Une unité posée (spec 073, « Unité ») : en-tête, bande d'illustration (spec 077), armure et
    /// résistances, état, puis un bouton d'effet par option ; pendant le choix, « Renvoyer » en haut
    /// à droite à leur place (issue #69).
    fn unit_tile(&mut self, ui: &mut egui::Ui, line: &UnitLine) -> Option<Command> {
        let mut clicked = None;
        ui.spacing_mut().item_spacing.y = 3.0;
        // Issue #69 : « Renvoyer » et la case « cible » en haut à droite, comme sur une carte.
        let row = vec2(ui.available_width(), ui.spacing().interact_size.y);
        ui.allocate_ui_with_layout(row, Layout::right_to_left(Align::Center), |ui| {
            if let Some(button) = &line.disband
                && shortcut_button(ui, &button.label, button.shortcut.as_deref(), &self.catalog)
                    .clicked()
            {
                clicked = Some(button.command);
            }
            // Issue privée 258 : l'unité visée par le sort en cours se coche.
            if let Some(toggle) = &line.spell_target {
                let mut checked = toggle.checked;
                if ui.checkbox(&mut checked, &toggle.label).changed() {
                    clicked = Some(toggle.command);
                }
            }
            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                ui.add(egui::Label::new(egui::RichText::new(&line.text).strong()).truncate());
            });
        });
        if let Some(texture) = self.illustration_texture(ui.ctx(), line.illustration) {
            art_band(ui, texture);
        }
        ui.horizontal_wrapped(|ui| {
            self.icon_value(ui, &line.armor, false);
            for resistance in &line.resistances {
                self.icon_value(ui, resistance, false);
            }
        });
        ui.add(egui::Label::new(&line.state).wrap());
        // Spec 055 : la bannière portée, grisée si retournée, son nom en infobulle ; ses capacités.
        if let Some(banner) = &line.banner {
            let text = egui::RichText::new(format!("⚑ {}", banner.name));
            ui.add(
                egui::Label::new(if banner.flipped {
                    text.weak()
                } else {
                    text.strong()
                })
                .truncate(),
            )
            .on_hover_text(&banner.tooltip);
        }
        for button in &line.banner_uses {
            let label = crate::plain_text(&button.label, &self.catalog);
            if ui.add(egui::Button::new(label).wrap()).clicked() {
                clicked = Some(button.command);
            }
        }
        if line.disband.is_none() {
            clicked = self.effect_buttons(ui, &line.effects).or(clicked);
        }
        clicked
    }

    /// La section Compétences (spec 051) : le titre et la pile, puis une ligne par compétence,
    /// nom et fréquence, son état et ses boutons d'options ; le texte en infobulle.
    fn skills_column(&mut self, ui: &mut egui::Ui, section: &SkillsSection) -> Option<Command> {
        let mut clicked = None;
        ui.vertical(|ui| {
            ui.set_width(SKILLS_WIDTH);
            let top = ui.cursor().min.y;
            ui.horizontal(|ui| {
                ui.strong(&section.title);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.weak(&section.pile);
                });
            });
            // Issue privée 302 : la rangée du titre, plus haute que celui de la Main, ne doit pas pousser
            // la liste sous le bas de la fenêtre ; le surplus défile, barre toujours visible.
            let title_height = ui.cursor().min.y - top;
            let hand_title_height =
                ui.text_style_height(&egui::TextStyle::Body) + ui.spacing().item_spacing.y;
            ui.spacing_mut().scroll = egui::style::ScrollStyle::thin();
            egui::ScrollArea::vertical()
                .id_salt("skills_column")
                .max_height(CARD_HEIGHT - (title_height - hand_title_height).max(0.0))
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    for line in &section.skills {
                        let response = card_frame().inner_margin(4.0).show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.spacing_mut().item_spacing.y = 2.0;
                            ui.horizontal(|ui| {
                                let name = egui::RichText::new(&line.name).strong();
                                // La fréquence garde sa place : le nom se tronque avant elle,
                                // plutôt que de la recouvrir (issue privée 302, Surcharge de mana).
                                let usage_width = ui
                                    .painter()
                                    .layout_no_wrap(
                                        line.usage.clone(),
                                        egui::TextStyle::Body.resolve(ui.style()),
                                        Color32::PLACEHOLDER,
                                    )
                                    .size()
                                    .x;
                                let name_width = (ui.available_width()
                                    - usage_width
                                    - ui.spacing().item_spacing.x)
                                    .max(0.0);
                                ui.scope(|ui| {
                                    ui.set_max_width(name_width);
                                    ui.add(
                                        egui::Label::new(if line.state.is_some() {
                                            name.weak()
                                        } else {
                                            name
                                        })
                                        .truncate(),
                                    );
                                });
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.weak(&line.usage);
                                });
                            });
                            if let Some(state) = &line.state {
                                ui.add(egui::Label::new(egui::RichText::new(state).weak()).wrap());
                            }
                            self.effect_buttons(ui, &line.options)
                        });
                        clicked = response.inner.or(clicked);
                        response.response.on_hover_text(&line.tooltip);
                        ui.add_space(2.0);
                    }
                });
        });
        clicked
    }

    /// Les boutons d'effet, à la suite, repliés à la ligne.
    fn effect_buttons(&mut self, ui: &mut egui::Ui, buttons: &[EffectButton]) -> Option<Command> {
        // Sans bouton, pas de rangée vide : la Blessure déborderait (spec 077, L3).
        if buttons.is_empty() {
            return None;
        }
        let mut clicked = None;
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            for button in buttons {
                clicked = self.effect_button(ui, button).or(clicked);
            }
        });
        clicked
    }

    /// Un bouton d'effet (spec 073) : la pastille du mana à payer, puis l'icône et sa valeur, un
    /// cristal, ou le texte de l'effet ; grisé sans commande, l'infobulle dit pourquoi.
    fn effect_button(&mut self, ui: &mut egui::Ui, button: &EffectButton) -> Option<Command> {
        let mark = button
            .mark
            .as_deref()
            .map(|mark| format!(" {mark}"))
            .unwrap_or_default();
        // Un bouton qui ne tient pas au bout de la ligne passe à la ligne suivante, plutôt que de
        // s'écraser en colonne ou de sortir de la carte, pastille comprise (spec #11, #13).
        let natural = effect_button_width(ui, button, &mark, &self.catalog);
        let row_started = ui.cursor().min.x > ui.max_rect().min.x + 1.0;
        if row_started && natural > ui.available_size_before_wrap().x {
            ui.end_row();
        }
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = EFFECT_GAP;
            ui.spacing_mut().button_padding = Vec2::new(EFFECT_PADDING, 1.0);
            // Spec #11 : le mana en plus, « pastille : effet ».
            if let Some(color) = button.color {
                mana_swatch(ui, color);
                ui.label(":");
            }
            let image = |texture| {
                egui::Image::from_texture(SizedTexture::new(texture, Vec2::splat(EFFECT_ICON)))
            };
            // Le texte de repli, sans repère d'icône (spec #27, C21) ; sans la condition, dessinée
            // à côté.
            let plain = crate::plain_text(
                &EffectButton {
                    condition: None,
                    ..button.clone()
                }
                .fallback_text(),
                &self.catalog,
            );
            let widget = match &button.face {
                EffectFace::Icon(value) => {
                    match self.icon_texture(ui, value.icon.image(), EFFECT_ICON) {
                        Some(texture) => egui::Button::image_and_text(
                            image(texture),
                            format!("{}{mark}", value.value),
                        ),
                        None => egui::Button::new(plain.as_str()).wrap(),
                    }
                }
                EffectFace::Crystal { color, .. } => {
                    match self.icon_texture(ui, crystal_image(*color), EFFECT_ICON) {
                        Some(texture) => egui::Button::image(image(texture)),
                        None => egui::Button::new(plain.as_str()).wrap(),
                    }
                }
                EffectFace::Text(_) => egui::Button::new(plain.as_str()).wrap(),
            };
            // Spec #27, C21 : egui n'écrit que du texte dans un bouton et son infobulle.
            let tooltip = crate::plain_text(&button.tooltip, &self.catalog);
            let response = ui
                .add_enabled(button.command.is_some(), widget)
                .on_hover_text(&tooltip)
                .on_disabled_hover_text(&tooltip);
            if let Some(condition) = &button.condition {
                self.effect_condition(ui, condition);
            }
            response.clicked().then_some(button.command).flatten()
        })
        .inner
    }

    /// Spec #27, C13 : « ×2 si » et l'icône Vif, qui a son infobulle, après le bouton ; sans
    /// fichier d'icône, son nom.
    fn effect_condition(&mut self, ui: &mut egui::Ui, condition: &crate::EffectCondition) {
        ui.label(&condition.text);
        match self.icon_image(ui, condition.icon.icon.image(), EFFECT_ICON) {
            Some(icon) => {
                icon_tooltip(icon, &condition.icon);
            }
            None => {
                ui.label(&condition.icon.tooltip);
            }
        }
    }

    /// Le bandeau du choix en cours posé par-dessus le bas de `area`, le plateau ou l'écran de
    /// combat (issue privée 177) : il n'agrandit pas le panneau du bas, le plateau garde sa taille.
    pub(super) fn choice_overlay(
        &mut self,
        ctx: &egui::Context,
        area: egui::Rect,
        screen: &GameScreen,
    ) {
        let shown = egui::Area::new(Id::unique("choice_banner"))
            .pivot(egui::Align2::LEFT_BOTTOM)
            .fixed_pos(area.left_bottom())
            .constrain(false)
            .show(ctx, |ui| {
                ui.set_max_width(area.width());
                self.choice_banner(ui, screen)
            });
        self.banner_height = shown.response.rect.height();
        let clicked = shown.inner;
        if let Some(command) = clicked {
            self.client.command(command);
            ctx.request_repaint();
        }
    }

    /// Le bandeau du choix en cours (spec 073) : carte à défausser,
    /// Concentration, unité à renvoyer, mana, dés, fin du tour, confirmation de la fin de la
    /// manche.
    fn choice_banner(&mut self, ui: &mut egui::Ui, screen: &GameScreen) -> Option<Command> {
        let mut clicked = None;
        let catalog = &self.catalog;
        let banner = |ui: &mut egui::Ui, add: &mut dyn FnMut(&mut egui::Ui)| {
            // Opaque : il couvre le bas du plateau.
            Frame::group(ui.style())
                .fill(ui.visuals().panel_fill)
                .stroke(Stroke::new(1.0, CHOICE_STROKE))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    add(ui);
                });
        };
        for prompt in [
            &screen.boost_prompt,
            &screen.discard_prompt,
            &screen.disband_prompt,
        ]
        .into_iter()
        .flatten()
        {
            banner(ui, &mut |ui| {
                ui.horizontal_wrapped(|ui| {
                    marked_label(ui, &prompt.text, egui::RichText::new, &self.catalog);
                    if shortcut_button(
                        ui,
                        &prompt.cancel.label,
                        prompt.cancel.shortcut.as_deref(),
                        catalog,
                    )
                    .clicked()
                    {
                        clicked = Some(prompt.cancel.command);
                    }
                });
            });
        }
        if let Some(prompt) = screen.mana_prompt.as_ref().or(screen.unit_prompt.as_ref()) {
            banner(ui, &mut |ui| {
                ui.horizontal_wrapped(|ui| {
                    marked_label(ui, &prompt.text, egui::RichText::new, &self.catalog);
                    for button in prompt.payments.iter().chain([&prompt.cancel]) {
                        if shortcut_button(ui, &button.label, button.shortcut.as_deref(), catalog)
                            .clicked()
                        {
                            clicked = Some(button.command);
                        }
                    }
                });
            });
        }
        if let Some(prompt) = &screen.set_dice_prompt {
            banner(ui, &mut |ui| {
                ui.horizontal_wrapped(|ui| {
                    marked_label(ui, &prompt.text, egui::RichText::new, &self.catalog);
                    for button in &prompt.colors {
                        clicked = strong_button(ui, button).or(clicked);
                    }
                    if shortcut_button(
                        ui,
                        &prompt.cancel.label,
                        prompt.cancel.shortcut.as_deref(),
                        catalog,
                    )
                    .clicked()
                    {
                        clicked = Some(prompt.cancel.command);
                    }
                });
            });
        }
        // Spec 060 : Faux départ, Vol de mana, Appel du mana, Méditation nocturne.
        for prompt in [&screen.round_end_confirm, &screen.tactic_prompt]
            .into_iter()
            .flatten()
        {
            banner(ui, &mut |ui| {
                ui.horizontal_wrapped(|ui| {
                    marked_label(ui, &prompt.text, egui::RichText::new, &self.catalog);
                    for button in &prompt.buttons {
                        if shortcut_button(ui, &button.label, button.shortcut.as_deref(), catalog)
                            .clicked()
                        {
                            clicked = Some(button.command);
                        }
                    }
                });
            });
        }
        // Spec 016 : le texte du mode fin du tour ; ses boutons sont en bas du panneau de droite.
        if let Some(prompt) = &screen.end_turn_prompt {
            banner(ui, &mut |ui| {
                ui.horizontal_wrapped(|ui| {
                    marked_label(ui, &prompt.text, egui::RichText::new, &self.catalog);
                    ui.label(&prompt.draw);
                });
                // Spec 052, C26 : la retraite forcée vient avant tout le reste de la fin du tour.
                for line in [
                    &prompt.forced_withdrawal,
                    &prompt.must_discard,
                    &prompt.no_rest,
                ]
                .into_iter()
                .flatten()
                {
                    ui.add(egui::Label::new(line).wrap());
                }
                // Spec 052, C39 : garder Temporisation, cochée d'office.
                if let Some(toggle) = &prompt.steady_tempo {
                    let mut checked = toggle.checked;
                    if ui.checkbox(&mut checked, &toggle.label).clicked() {
                        clicked = Some(toggle.command);
                    }
                }
                if let Some(menu) = &prompt.reclaim
                    && let Some(command) = reclaim_menu(ui, menu)
                {
                    clicked = Some(command);
                }
                // Spec 044 : la Blessure de la clairière, à choix unique.
                if let Some(menu) = &prompt.glade {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(&menu.label);
                        for option in &menu.options {
                            let response = ui.add_enabled(
                                option.enabled,
                                egui::RadioButton::new(option.selected, &option.label),
                            );
                            if response.clicked() {
                                clicked = Some(option.command);
                            }
                        }
                    });
                }
            });
        }
        clicked
    }

    /// La fenêtre de la défausse (spec 073, C35) : ses cartes en rangée, sans bouton.
    pub(super) fn discard_window(
        &mut self,
        ctx: &egui::Context,
        window: &DiscardWindow,
        screen: &GameScreen,
    ) {
        let mut clicked = None;
        let sideways = screen.sideways_title.clone();
        egui::Window::new(&window.title)
            .id(Id::unique("discard_window"))
            .collapsible(false)
            // À la taille de ses cartes (issue privée 177) : sans `auto_sized`, egui donne à la
            // fenêtre sa largeur par défaut, bien plus large qu'une carte seule.
            .auto_sized()
            .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .show(ctx, |ui| {
                let max_width = ctx.content_rect().width() * 0.8;
                egui::ScrollArea::horizontal()
                    .max_width(max_width)
                    .show(ui, |ui| {
                        ui.horizontal_top(|ui| {
                            ui.spacing_mut().item_spacing.x = CARD_GAP;
                            for card in &window.cards {
                                card_frame().show(ui, |ui| {
                                    let size = card_content_size();
                                    ui.set_width(size.x);
                                    ui.set_min_height(size.y);
                                    ui.vertical(|ui| {
                                        self.card_tile(ui, card, &sideways);
                                    });
                                });
                            }
                        });
                    });
                if shortcut_button(
                    ui,
                    &window.close.label,
                    window.close.shortcut.as_deref(),
                    &self.catalog,
                )
                .clicked()
                {
                    clicked = Some(window.close.command);
                }
            });
        if let Some(command) = clicked {
            self.client.command(command);
            ctx.request_repaint();
        }
    }
}

/// Le titre du bloc renforcé (spec #11, C7). Un sort : un trait, puis ses pastilles et son nom
/// renforcé en gras. Une action : « —— pastille Renforcé —— » centré ; un artefact, « —— si
/// détruit —— », sans pastille. Partagé par la main et les fenêtres (spec #32).
pub(super) fn strong_separator(
    ui: &mut egui::Ui,
    spell: bool,
    heading: &str,
    colors: &[ManaColor],
) {
    if spell {
        ui.separator();
        ui.horizontal(|ui| {
            for color in colors {
                mana_swatch(ui, *color);
            }
            ui.add(egui::Label::new(egui::RichText::new(heading).strong()).truncate());
        });
        return;
    }
    let row = ui.available_width();
    let spacing = ui.spacing().item_spacing.x;
    let label = text_width(ui, heading, egui::TextStyle::Body);
    let swatches = colors.len() as f32 * (icon_size(ui) * 0.6 + spacing);
    let side = ((row - label - swatches) / 2.0 - spacing).max(4.0);
    let stroke = ui.visuals().widgets.noninteractive.bg_stroke;
    let line = |ui: &mut egui::Ui, width: f32| {
        let (rect, _) = ui.allocate_exact_size(vec2(width, 1.0), Sense::hover());
        ui.painter().hline(rect.x_range(), rect.center().y, stroke);
    };
    let height = ui.spacing().interact_size.y;
    ui.allocate_ui_with_layout(
        vec2(row, height),
        Layout::left_to_right(Align::Center),
        |ui| {
            line(ui, side);
            for color in colors {
                mana_swatch(ui, *color);
            }
            ui.weak(heading);
            let rest = ui.available_width();
            line(ui, rest);
        },
    );
}

/// Largeur d'un texte sur une ligne, dans ce style.
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

/// Largeur d'un bouton d'effet posé, pastille et « : » compris (spec #11) : de quoi savoir s'il
/// tient au bout de la ligne, quel que soit son visage.
fn effect_button_width(
    ui: &egui::Ui,
    button: &EffectButton,
    mark: &str,
    catalog: &arcanum_i18n::Catalog,
) -> f32 {
    let face = match &button.face {
        EffectFace::Icon(value) => {
            EFFECT_ICON
                + ui.spacing().icon_spacing
                + text_width(
                    ui,
                    &format!("{}{mark}", value.value),
                    egui::TextStyle::Button,
                )
        }
        EffectFace::Crystal { .. } => EFFECT_ICON,
        EffectFace::Text(_) => text_width(
            ui,
            &crate::plain_text(&button.fallback_text(), catalog),
            egui::TextStyle::Button,
        ),
    };
    // Spec #27, C13 : « ×2 si » et l'icône, après le bouton.
    let condition = button.condition.as_ref().map_or(0.0, |condition| {
        2.0 * EFFECT_GAP + text_width(ui, &condition.text, egui::TextStyle::Body) + EFFECT_ICON
    });
    let swatch = if button.color.is_some() {
        icon_size(ui) * 0.6 + text_width(ui, ":", egui::TextStyle::Body) + 2.0 * EFFECT_GAP
    } else {
        0.0
    };
    swatch + face + 2.0 * EFFECT_PADDING + condition
}

/// Pastille d'une couleur de mana : celle d'une carte, du bloc renforcé, d'un bouton d'effet.
pub(super) fn mana_swatch(ui: &mut egui::Ui, color: ManaColor) {
    let size = icon_size(ui) * 0.6;
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    ui.painter().circle(
        rect.center(),
        size / 2.0,
        mana_color(color),
        Stroke::new(1.0, SWATCH_STROKE),
    );
}

/// L'infobulle d'un bouton du bas : sa raison, puis sa touche (spec 071).
pub(super) fn bottom_hint(
    tooltip: Option<&str>,
    shortcut: Option<&str>,
    catalog: &arcanum_i18n::Catalog,
) -> String {
    hover_text(tooltip, shortcut, catalog).unwrap_or_default()
}
