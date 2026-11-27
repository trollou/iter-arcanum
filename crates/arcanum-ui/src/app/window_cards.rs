//! Cartes des fenêtres Offres et des choix de l'artefact et du sort (spec #32) : dessinées comme
//! une carte de la main, posées en rangées de cartes de même hauteur.

use eframe::egui::{self, Align, Layout, vec2};

use arcanum_core::ManaColor;
use arcanum_i18n::Catalog;

use super::Thumbnails;
use super::bottom_panel::{art_band, card_content_size, card_frame, mana_swatch, strong_separator};
use super::icons::marked_label;
use super::widgets::shortcut_button;
use crate::{CARD_GAP, Command, OfferCardView, cards_per_row, row_width};

/// Place gardée autour d'une fenêtre : elle ne dépasse jamais l'écran (L1, L2).
const SCREEN_MARGIN: f32 = 32.0;
/// Hauteur gardée hors des cartes : barre de titre, boutons du bas et marges (L2).
const WINDOW_CHROME: f32 = 110.0;

/// La largeur et la hauteur maximales d'une fenêtre de cartes, et celle de sa zone qui défile.
pub(super) fn window_limits(ctx: &egui::Context) -> (f32, f32) {
    let screen = ctx.content_rect();
    (
        (screen.width() - 2.0 * SCREEN_MARGIN).max(row_width(1)),
        (screen.height() - 2.0 * SCREEN_MARGIN - WINDOW_CHROME).max(100.0),
    )
}

/// La largeur naturelle de cette offre, posée sur une rangée.
pub(super) fn natural_width(cards: &[OfferCardView]) -> f32 {
    row_width(cards.len())
}

/// Ces cartes en rangées dans la largeur `width` (C8), passage à la ligne au besoin (L1) ; les
/// cartes d'une rangée à la hauteur de la plus haute (L3), mesurée à l'image précédente sous `id`.
/// La première rangée de l'offre posée à côté (`peer`, Q1) compte aussi : actions avancées et
/// sorts forment une seule rangée.
pub(super) fn card_rows(
    ui: &mut egui::Ui,
    id: egui::Id,
    peer: Option<egui::Id>,
    cards: &[OfferCardView],
    width: f32,
    thumbnails: &Thumbnails,
    catalog: &Catalog,
) -> Option<Command> {
    let mut clicked = None;
    let per_row = cards_per_row(width);
    let ctx = ui.ctx().clone();
    let remembered = |id: egui::Id| ctx.data(|data| data.get_temp::<f32>(id));
    for (index, row) in cards.chunks(per_row).enumerate() {
        let id = id.with(index);
        let height = remembered(id).unwrap_or(0.0).max(
            peer.filter(|_| index == 0)
                .and_then(|peer| remembered(peer.with(0)))
                .unwrap_or(0.0),
        );
        let mut tallest: f32 = 0.0;
        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = CARD_GAP;
            for card in row {
                let (natural, command) = window_card(ui, card, height, thumbnails, catalog);
                tallest = tallest.max(natural);
                clicked = command.or(clicked);
            }
        });
        if remembered(id) != Some(tallest) {
            ctx.data_mut(|data| data.insert_temp(id, tallest));
            ctx.request_repaint();
        }
    }
    clicked
}

/// Une carte : en-tête (pastille, nom, marque, et à droite le bouton, L4), bande, texte de base,
/// séparation de la main et texte renforcé, boutons « Utiliser ». Rend la hauteur naturelle de son
/// contenu ; la carte est allongée jusqu'à `height`.
fn window_card(
    ui: &mut egui::Ui,
    card: &OfferCardView,
    height: f32,
    thumbnails: &Thumbnails,
    catalog: &Catalog,
) -> (f32, Option<Command>) {
    let mut clicked = None;
    let natural = card_frame()
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.set_width(card_content_size().x);
                ui.spacing_mut().item_spacing.y = 3.0;
                let row = vec2(ui.available_width(), ui.spacing().interact_size.y);
                ui.allocate_ui_with_layout(row, Layout::right_to_left(Align::Center), |ui| {
                    if let Some(take) = &card.take
                        && shortcut_button(ui, &take.label, take.shortcut.as_deref(), catalog)
                            .clicked()
                    {
                        clicked = Some(take.command);
                    }
                    // Spec 041 : « monastère », une action avancée de l'offre d'unités.
                    if let Some(mark) = &card.mark {
                        ui.small(mark);
                    }
                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                        if let Some(color) = card.color {
                            mana_swatch(ui, ManaColor::Basic(color));
                        }
                        // Tronqué, le nom entier s'affiche au survol (L4).
                        ui.add(
                            egui::Label::new(egui::RichText::new(&card.name).strong()).truncate(),
                        );
                    });
                });
                // Spec 077, C2 : la bande sous le nom, rien de réservé sans image (L3).
                if let Some(&texture) = thumbnails.get(&card.illustration) {
                    art_band(ui, texture);
                }
                match card.lines.as_slice() {
                    [basic, strong] if !card.strong_heading.is_empty() => {
                        marked_label(ui, basic, egui::RichText::new, catalog);
                        // Un sort a pour pastilles sa couleur puis le noir (spec #11, C7).
                        let spell = card.strong_colors.last() == Some(&ManaColor::Black);
                        strong_separator(ui, spell, &card.strong_heading, &card.strong_colors);
                        marked_label(ui, strong, super::icons::weak_text, catalog);
                    }
                    lines => {
                        for line in lines {
                            marked_label(ui, line, egui::RichText::new, catalog);
                        }
                    }
                }
                // Spec 057 : les capacités d'une unité de l'offre (Appel aux armes).
                for button in &card.uses {
                    if ui
                        .add(egui::Button::new(crate::plain_text(&button.label, catalog)).wrap())
                        .clicked()
                    {
                        clicked = Some(button.command);
                    }
                }
                let natural = ui.min_rect().height();
                // `set_min_height` compterait cette hauteur depuis le curseur, sous le contenu :
                // on allonge depuis le haut de la carte.
                let top = ui.min_rect().top();
                ui.expand_to_include_y(top + height - 2.0 * frame_padding());
                natural + 2.0 * frame_padding()
            })
            .inner
        })
        .inner;
    (natural, clicked)
}

/// Marge intérieure et bordure du cadre d'une carte, d'un côté.
fn frame_padding() -> f32 {
    (crate::CARD_WIDTH - card_content_size().x) / 2.0
}
