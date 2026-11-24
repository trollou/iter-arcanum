//! Fenêtres : chargement (spec 024), préférences (spec 080) et fin de partie (spec 063) ; l'écran de nouvelle partie est
//! dans `new_game.rs` (spec 078).

use super::widgets::{enabled_shortcut_button, shortcut_button};
use eframe::egui::{self, Color32, Sense, TextStyle, Vec2, vec2};

use arcanum_i18n::Catalog;

use super::widgets::{card_color, color_swatch, with_thumbnail};
use super::{IterArcanumApp, Thumbnails};
use crate::{
    AltarWindow, ArtifactChoice, Command, CrystalChoice, MeditationWindow, OffersWindow,
    PreferencesWindow, PreparationWindow, ReclaimWindow, ScoreWindow, SkillChoiceWindow,
    SkillOffer, TacticCard, TacticDecision, TacticWindow, TokenChoice, volume_value_label,
};

/// Largeur d'une colonne de la fenêtre Offres (spec 050).
/// Spec 070, C18 : élargies pour le texte à la taille du jeu, la fenêtre tient en 1600 × 900.
const OFFER_COLUMN_WIDTH: f32 = 300.0;
const OFFER_SWATCH: f32 = 10.0;

/// Largeur à réserver pour que le titre d'une fenêtre ne soit pas tronqué : egui ne le fait pas
/// revenir à la ligne, il suit la largeur du contenu (issue privée 135).
fn title_min_width(ctx: &egui::Context, title: &str) -> f32 {
    let style = ctx.global_style();
    // Issue privée 236 : la barre de titre d'egui (0.37) entoure le titre de la marge intérieure de la
    // fenêtre et de deux espacements (un de chaque côté), et le cadre en ajoute son trait ; la
    // largeur minimale ne compte ni la marge intérieure du contenu ni le trait.
    let margin = style.spacing.window_margin.sum().x
        + 2.0 * style.spacing.item_spacing.x
        + 2.0 * style.visuals.window_stroke.width;
    let font = TextStyle::Heading.resolve(&style);
    let text = ctx.fonts_mut(|f| f.layout_no_wrap(title.to_owned(), font, Color32::WHITE));
    text.size().x + margin
}

impl IterArcanumApp {
    /// Fenêtres communes au lancement et à la partie : chargement (spec 024) et nouvelle partie
    /// (spec 065).
    pub(super) fn windows(&mut self, ctx: &egui::Context) {
        let mut clicked = None;
        if let Some(window) = self.client.load_window(&self.catalog) {
            egui::Window::new(&window.title)
                .min_width(title_min_width(ctx, &window.title))
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    if let Some(empty) = &window.empty {
                        ui.label(empty);
                    }
                    for line in &window.lines {
                        ui.horizontal(|ui| {
                            ui.add_enabled(line.load.is_some(), egui::Label::new(&line.text));
                            if let Some(button) = &line.load
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
        }
        // Spec 080 : la fenêtre des préférences, au-dessus de l'écran et de ses fenêtres.
        if let Some(window) = self.client.preferences_window(&self.catalog) {
            let edit = preferences_window(ctx, &window, &self.catalog);
            clicked = edit.clicked.or(clicked);
            // Spec 081 : un curseur réglé au clavier s'enregistre 1 s après son dernier
            // changement (C21) ; glissé, il s'enregistre au lâcher.
            if edit.keyboard_volume {
                self.volume_save_at = Some(ctx.input(|input| input.time) + VOLUME_KEY_DELAY);
            }
            for command in edit.volumes {
                self.client.command(command);
                ctx.request_repaint();
            }
        }
        if let Some(deadline) = self.volume_save_at {
            let now = ctx.input(|input| input.time);
            if now >= deadline {
                self.volume_save_at = None;
                self.client.command(Command::SaveVolumes);
            } else {
                ctx.request_repaint_after(std::time::Duration::from_secs_f64(deadline - now));
            }
        }
        // Spec 074, Q1 : la confirmation de Quitter, par-dessus tout le reste.
        if let Some(window) = self.client.quit_window(&self.catalog) {
            egui::Window::new(&window.title)
                .min_width(title_min_width(ctx, &window.title))
                .collapsible(false)
                .resizable(false)
                .order(egui::Order::Foreground)
                .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.label(&window.text);
                    ui.horizontal(|ui| {
                        for button in [&window.quit, &window.cancel] {
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
        if let Some(command) = clicked {
            self.client.command(command);
            ctx.request_repaint();
        }
    }
}

/// Délai entre le dernier changement d'un curseur au clavier et l'enregistrement (spec 081).
const VOLUME_KEY_DELAY: f64 = 1.0;

/// Ce que la fenêtre des préférences a reçu dans l'image.
struct PreferencesEdit {
    /// La commande cliquée.
    clicked: Option<Command>,
    /// Spec 081 : `SetVolume` pendant un glissé ou au clavier, `SaveVolumes` au lâcher.
    volumes: Vec<Command>,
    /// Un curseur a changé sans glissé (flèches du clavier).
    keyboard_volume: bool,
}

/// Fenêtre des préférences (spec 080) : la langue, le plein écran au lancement, les volumes
/// (spec 081). La croix fait comme « Fermer ».
fn preferences_window(
    ctx: &egui::Context,
    window: &PreferencesWindow,
    catalog: &Catalog,
) -> PreferencesEdit {
    let mut clicked = None;
    let mut volumes = Vec::new();
    let mut keyboard_volume = false;
    let mut open = true;
    egui::Window::new(&window.title)
        .min_width(title_min_width(ctx, &window.title).max(320.0))
        .collapsible(false)
        .resizable(false)
        .open(&mut open)
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.strong(&window.language_label);
            let selected = window
                .languages
                .iter()
                .find(|option| option.checked)
                .map_or("", |option| option.label.as_str());
            egui::ComboBox::from_id_salt("preferences_language")
                .selected_text(selected)
                .width(280.0)
                .show_ui(ui, |ui| {
                    for option in &window.languages {
                        if ui.selectable_label(option.checked, &option.label).clicked()
                            && !option.checked
                        {
                            clicked = Some(option.command);
                        }
                    }
                });
            ui.add_space(12.0);
            let mut fullscreen = window.fullscreen_at_launch;
            if ui
                .checkbox(&mut fullscreen, &window.fullscreen_label)
                .changed()
            {
                clicked = Some(Command::SetFullscreenAtLaunch(fullscreen));
            }
            ui.label(egui::RichText::new(&window.fullscreen_hint).weak());
            ui.add_space(12.0);
            ui.strong(&window.sound_label);
            egui::Grid::new("preferences_volumes")
                .num_columns(2)
                .show(ui, |ui| {
                    for slider in &window.volumes {
                        ui.label(&slider.label);
                        let mut value = slider.value;
                        let response = ui.add(
                            egui::Slider::new(&mut value, 0..=100).custom_formatter(|v, _| {
                                // Valeur entière de 0 à 100 : la conversion ne perd rien.
                                volume_value_label(v.round().clamp(0.0, 100.0) as u8, catalog)
                            }),
                        );
                        if response.changed() {
                            volumes.push(Command::SetVolume(slider.category, value));
                            keyboard_volume |= !response.dragged();
                        }
                        if response.drag_stopped() {
                            volumes.push(Command::SaveVolumes);
                        }
                        ui.end_row();
                    }
                });
            if let Some(no_audio) = &window.no_audio {
                ui.colored_label(ui.visuals().warn_fg_color, no_audio);
            }
            if let Some(not_saved) = &window.not_saved {
                ui.add_space(8.0);
                ui.colored_label(ui.visuals().warn_fg_color, not_saved);
            }
            ui.add_space(8.0);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                if shortcut_button(
                    ui,
                    &window.close.label,
                    window.close.shortcut.as_deref(),
                    catalog,
                )
                .clicked()
                {
                    clicked = Some(window.close.command);
                }
            });
        });
    if !open {
        clicked = Some(window.close.command);
    }
    PreferencesEdit {
        clicked,
        volumes,
        keyboard_volume,
    }
}

/// Fenêtre de fin (spec 063), centrée, avec « Nouvelle partie… » (spec 065) : la commande cliquée.
pub(super) fn score_window(
    ctx: &egui::Context,
    window: &ScoreWindow,
    catalog: &Catalog,
) -> Option<Command> {
    let mut clicked = None;
    egui::Window::new(&window.title)
        .min_width(title_min_width(ctx, &window.title))
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.label(&window.reason);
            // Spec 064 : la mission accomplie ou échouée, en gras sous la raison.
            if let Some(outcome) = &window.outcome {
                ui.strong(outcome);
            }
            ui.add_space(8.0);
            for line in &window.lines {
                ui.label(line);
            }
            ui.strong(&window.total);
            ui.small(&window.no_titles);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                if shortcut_button(
                    ui,
                    &window.close.label,
                    window.close.shortcut.as_deref(),
                    catalog,
                )
                .clicked()
                {
                    clicked = Some(window.close.command);
                }
                // Spec 065 : à côté de « Voir le plateau ».
                if shortcut_button(
                    ui,
                    &window.new_game.label,
                    window.new_game.shortcut.as_deref(),
                    catalog,
                )
                .clicked()
                {
                    clicked = Some(window.new_game.command);
                }
            });
        });
    clicked
}

/// Fenêtre Offres (spec 050), centrée : trois colonnes, « Prendre » pendant le choix d'une action
/// avancée ; la croix et « Fermer » seulement hors du choix.
pub(super) fn offers_window(
    ctx: &egui::Context,
    window: &OffersWindow,
    thumbnails: &Thumbnails,
    catalog: &Catalog,
) -> Option<Command> {
    let mut clicked = None;
    let mut open = true;
    let mut frame = egui::Window::new(&window.title)
        .id(egui::Id::unique("offers_window"))
        .min_width(title_min_width(ctx, &window.title))
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0));
    if window.closable {
        frame = frame.open(&mut open);
    }
    frame.show(ctx, |ui| {
        ui.horizontal_top(|ui| {
            for column in &window.columns {
                ui.vertical(|ui| {
                    ui.set_width(OFFER_COLUMN_WIDTH);
                    ui.strong(&column.title);
                    for card in &column.cards {
                        ui.separator();
                        // Spec 072, C6 : la vignette en haut à droite, comme sur une carte.
                        let texture = thumbnails.get(&card.illustration).copied();
                        with_thumbnail(ui, texture, |ui| {
                            ui.horizontal(|ui| {
                                if let Some(color) = card.color {
                                    let (rect, _) = ui.allocate_exact_size(
                                        Vec2::splat(OFFER_SWATCH),
                                        Sense::hover(),
                                    );
                                    ui.painter().rect_filled(rect, 2.0, card_color(color));
                                }
                                ui.strong(&card.name);
                                // Spec 041 : « monastère » après le nom d'une action avancée de
                                // l'offre d'unités.
                                if let Some(mark) = &card.mark {
                                    ui.small(mark);
                                }
                            });
                            for line in &card.lines {
                                ui.add(egui::Label::new(line).wrap());
                            }
                            if let Some(take) = &card.take
                                && shortcut_button(
                                    ui,
                                    &take.label,
                                    take.shortcut.as_deref(),
                                    catalog,
                                )
                                .clicked()
                            {
                                clicked = Some(take.command);
                            }
                            // Spec 057 : les capacités d'une unité de l'offre (Appel aux armes).
                            for button in &card.uses {
                                if ui.button(&button.label).clicked() {
                                    clicked = Some(button.command);
                                }
                            }
                        });
                    }
                });
            }
        });
        if window.closable
            && shortcut_button(
                ui,
                &window.close.label,
                window.close.shortcut.as_deref(),
                catalog,
            )
            .clicked()
        {
            clicked = Some(window.close.command);
        }
        // Spec 045 : « Renoncer » à l'unité de récompense, sans jeton de commandement libre.
        if let Some(decline) = &window.decline
            && shortcut_button(ui, &decline.label, decline.shortcut.as_deref(), catalog).clicked()
        {
            clicked = Some(decline.command);
        }
    });
    if !open {
        clicked = Some(window.close.command);
    }
    clicked
}

/// Fenêtre « Récompense : prenez un artefact » (spec 041) : un artefact par colonne, nom, texte et
/// « Prendre » ; elle ne se ferme pas, le choix est obligatoire.
/// La fenêtre du choix de la couleur du cristal (spec 044, C46) : un bouton par couleur, grisé
/// si l'inventaire en est plein.
pub(super) fn crystal_window(
    ctx: &egui::Context,
    window: &CrystalChoice,
    catalog: &Catalog,
) -> Option<Command> {
    let mut clicked = None;
    egui::Window::new(&window.title)
        .id(egui::Id::unique("crystal_window"))
        .min_width(title_min_width(ctx, &window.title))
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                for button in &window.colors {
                    color_swatch(ui, button.color);
                    if enabled_shortcut_button(
                        ui,
                        button.command.is_some(),
                        &button.label,
                        button.shortcut.as_deref(),
                        catalog,
                    )
                    .clicked()
                    {
                        clicked = button.command;
                    }
                    ui.add_space(6.0);
                }
            });
        });
    clicked
}

/// La fenêtre du choix en attente de Rituel de sang renforcé (spec 056, C4, C5) : six couleurs de
/// jeton, puis les paiements du cristal et « Non merci ».
pub(super) fn token_window(
    ctx: &egui::Context,
    window: &TokenChoice,
    catalog: &Catalog,
) -> Option<Command> {
    let mut clicked = None;
    egui::Window::new(&window.title)
        // Une fenêtre par étape : egui garde la largeur d'une fenêtre, le titre plus long du
        // cristal serait tronqué.
        .id(egui::Id::unique(if window.tokens.is_empty() {
            "token_window_crystal"
        } else {
            "token_window"
        }))
        .min_width(title_min_width(ctx, &window.title))
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                for button in &window.tokens {
                    super::bottom_panel::mana_swatch(ui, button.color);
                    if shortcut_button(ui, &button.label, button.shortcut.as_deref(), catalog)
                        .clicked()
                    {
                        clicked = Some(button.command);
                    }
                    ui.add_space(6.0);
                }
            });
            for button in window.payments.iter().chain(&window.decline) {
                if shortcut_button(ui, &button.label, button.shortcut.as_deref(), catalog).clicked()
                {
                    clicked = Some(button.command);
                }
            }
        });
    clicked
}

/// La fenêtre de Méditation (spec 057) : les cartes, numérotées dans l'ordre choisi, puis « Sur le
/// deck » et « Sous le deck ».
pub(super) fn meditation_window(
    ctx: &egui::Context,
    window: &MeditationWindow,
    catalog: &Catalog,
) -> Option<Command> {
    let mut clicked = None;
    egui::Window::new(&window.title)
        .id(egui::Id::unique("meditation_window"))
        .min_width(title_min_width(ctx, &window.title))
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                for card in &window.cards {
                    let label = match card.place {
                        Some(place) => format!("{place}. {}", card.name),
                        None => card.name.clone(),
                    };
                    let button = egui::Button::new(label).selected(card.place.is_some());
                    if ui.add_enabled(card.pick.is_some(), button).clicked() {
                        clicked = card.pick;
                    }
                }
            });
            ui.horizontal(|ui| {
                for button in [&window.top, &window.bottom].into_iter().flatten() {
                    if shortcut_button(ui, &button.label, button.shortcut.as_deref(), catalog)
                        .clicked()
                    {
                        clicked = Some(button.command);
                    }
                }
            });
        });
    clicked
}

pub(super) fn artifact_window(
    ctx: &egui::Context,
    window: &ArtifactChoice,
    id: &'static str,
    thumbnails: &Thumbnails,
    catalog: &Catalog,
) -> Option<Command> {
    let mut clicked = None;
    egui::Window::new(&window.title)
        .id(egui::Id::unique(id))
        .min_width(title_min_width(ctx, &window.title))
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.horizontal_top(|ui| {
                for card in &window.cards {
                    ui.vertical(|ui| {
                        ui.set_width(OFFER_COLUMN_WIDTH);
                        // Spec 072, C6 : la vignette en haut à droite, comme sur une carte.
                        let texture = thumbnails.get(&card.illustration).copied();
                        with_thumbnail(ui, texture, |ui| {
                            ui.strong(&card.name);
                            for line in &card.lines {
                                ui.add(egui::Label::new(line).wrap());
                            }
                            if let Some(take) = &card.take
                                && shortcut_button(
                                    ui,
                                    &take.label,
                                    take.shortcut.as_deref(),
                                    catalog,
                                )
                                .clicked()
                            {
                                clicked = Some(take.command);
                            }
                        });
                    });
                }
            });
        });
    clicked
}

/// Taille d'une tactique dans la grille du choix (spec 060).
const TACTIC_CARD: Vec2 = vec2(230.0, 120.0);
const TACTIC_NUMBER_SIZE: f32 = 36.0;

/// La fenêtre du choix de la tactique (spec 060) : 2 lignes de 3, chaque tactique à la place de
/// son numéro, un trou pour une tactique retirée ; un clic l'encadre, « Prendre » la prend.
pub(super) fn tactic_window(
    ctx: &egui::Context,
    window: &TacticWindow,
    catalog: &Catalog,
) -> Option<Command> {
    let mut clicked = None;
    egui::Window::new(&window.title)
        .id(egui::Id::unique("tactic_window"))
        .min_width(title_min_width(ctx, &window.title))
        .collapsible(false)
        .resizable(false)
        // En haut : la main reste visible dessous, pour choisir en connaissance de cause.
        .anchor(egui::Align2::CENTER_TOP, vec2(0.0, 60.0))
        .show(ctx, |ui| {
            for row in window.slots.chunks(3) {
                ui.horizontal_top(|ui| {
                    for slot in row {
                        clicked = tactic_card(ui, slot.as_ref()).or(clicked);
                    }
                });
            }
            ui.add_space(4.0);
            ui.vertical_centered(|ui| {
                if enabled_shortcut_button(
                    ui,
                    window.take.command.is_some(),
                    &window.take.label,
                    window.take.shortcut.as_deref(),
                    catalog,
                )
                .clicked()
                {
                    clicked = window.take.command;
                }
            });
        });
    clicked
}

/// Une place de la grille : la tactique (grand numéro, nom, texte), ou un trou.
fn tactic_card(ui: &mut egui::Ui, card: Option<&TacticCard>) -> Option<Command> {
    let Some(card) = card else {
        // Retirée de la partie : la place reste vide, les autres ne bougent pas.
        ui.allocate_exact_size(TACTIC_CARD + Vec2::splat(12.0), Sense::hover());
        return None;
    };
    let visuals = ui.visuals();
    let stroke = if card.selected {
        egui::Stroke::new(3.0, visuals.selection.stroke.color)
    } else {
        egui::Stroke::new(1.0, visuals.widgets.noninteractive.bg_stroke.color)
    };
    let fill = if card.selected {
        visuals.selection.bg_fill.gamma_multiply(0.4)
    } else {
        visuals.extreme_bg_color
    };
    let response = egui::Frame::group(ui.style())
        .stroke(stroke)
        .fill(fill)
        .show(ui, |ui| {
            ui.set_width(TACTIC_CARD.x);
            ui.set_height(TACTIC_CARD.y);
            let enabled = card.select.is_some();
            // Dans la rangée de la grille, le cadre hérite d'une mise en page horizontale.
            ui.vertical(|ui| {
                ui.set_width(TACTIC_CARD.x);
                // La carte entière se clique : pas de curseur de sélection de texte dessus.
                ui.style_mut().interaction.selectable_labels = false;
                if !enabled {
                    ui.disable();
                }
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(card.number.to_string())
                            .size(TACTIC_NUMBER_SIZE)
                            .strong(),
                    );
                    ui.add(egui::Label::new(egui::RichText::new(&card.name).heading()).wrap());
                });
                ui.add(egui::Label::new(&card.text).wrap());
            });
        })
        .response;
    let response = ui.interact(response.rect, response.id.with("tactic"), Sense::click());
    if card.select.is_some() {
        response
            .clone()
            .on_hover_cursor(egui::CursorIcon::PointingHand);
    }
    response.clicked().then_some(card.select).flatten()
}

/// Préparation (spec 060, C30) : les cartes du deck, une par bouton.
pub(super) fn preparation_window(
    ctx: &egui::Context,
    window: &PreparationWindow,
) -> Option<Command> {
    let mut clicked = None;
    egui::Window::new(&window.title)
        .id(egui::Id::unique("preparation_window"))
        .min_width(title_min_width(ctx, &window.title))
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                for card in &window.cards {
                    if ui.button(&card.label).clicked() {
                        clicked = Some(card.command);
                    }
                }
            });
        });
    clicked
}

/// Provisions avant le tour, Longue Nuit pendant la pioche (spec 060) : le texte et deux boutons.
pub(super) fn tactic_decision_window(
    ctx: &egui::Context,
    window: &TacticDecision,
) -> Option<Command> {
    let mut clicked = None;
    egui::Window::new(&window.text)
        .id(egui::Id::unique("tactic_decision_window"))
        .min_width(title_min_width(ctx, &window.text))
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                for button in &window.buttons {
                    if ui
                        .add_enabled(button.command.is_some(), egui::Button::new(&button.label))
                        .clicked()
                    {
                        clicked = button.command;
                    }
                }
            });
        });
    clicked
}

/// Fenêtre du choix de compétence d'un niveau pair (spec 051, C4 à C8), centrée : les deux
/// compétences révélées côte à côte, puis l'offre commune et sa note. Pas de croix : le choix est
/// obligatoire, comme celui de l'action avancée.
pub(super) fn skill_choice_window(
    ctx: &egui::Context,
    window: &SkillChoiceWindow,
    catalog: &Catalog,
) -> Option<Command> {
    let mut clicked = None;
    let mut row = |ui: &mut egui::Ui, offers: &[SkillOffer]| {
        ui.horizontal_top(|ui| {
            for offer in offers {
                ui.vertical(|ui| {
                    ui.set_width(OFFER_COLUMN_WIDTH);
                    ui.horizontal(|ui| {
                        ui.strong(&offer.name);
                        ui.weak(&offer.usage);
                    });
                    ui.add(egui::Label::new(&offer.text).wrap());
                    if let Some(take) = &offer.take
                        && shortcut_button(ui, &take.label, take.shortcut.as_deref(), catalog)
                            .clicked()
                    {
                        clicked = Some(take.command);
                    }
                    if let Some(note) = &offer.note {
                        ui.add(egui::Label::new(egui::RichText::new(note).weak()).wrap());
                    }
                });
            }
        });
    };
    egui::Window::new(&window.title)
        .id(egui::Id::unique("skill_choice_window"))
        .min_width(title_min_width(ctx, &window.title))
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.strong(&window.revealed_title);
            row(ui, &window.revealed);
            if !window.common.is_empty() {
                ui.separator();
                ui.strong(&window.common_title);
                ui.add(egui::Label::new(&window.common_note).wrap());
                row(ui, &window.common);
            }
        });
    clicked
}

/// Fenêtre de l'offrande à l'autel (spec 045, C34), centrée : une ligne-bouton par paiement,
/// numérotée pour les touches 1 à 9, la note et « Annuler ».
pub(super) fn altar_window(
    ctx: &egui::Context,
    window: &AltarWindow,
    catalog: &Catalog,
) -> Option<Command> {
    let mut clicked = None;
    egui::Window::new(&window.title)
        .id(egui::Id::unique("altar_window"))
        .min_width(title_min_width(ctx, &window.title))
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .show(ctx, |ui| {
            for (place, payment) in window.payments.iter().enumerate() {
                let label = if place < 9 {
                    format!("{}  {}", place + 1, payment.label)
                } else {
                    payment.label.clone()
                };
                if shortcut_button(ui, &label, payment.shortcut.as_deref(), catalog).clicked() {
                    clicked = Some(payment.command);
                }
            }
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(&window.note);
                if shortcut_button(
                    ui,
                    &window.cancel.label,
                    window.cancel.shortcut.as_deref(),
                    catalog,
                )
                .clicked()
                {
                    clicked = Some(window.cancel.command);
                }
            });
        });
    clicked
}

/// La fenêtre de fin de manche (spec 055, C9) : une ligne par bannière, « Garder sur l'unité » ou
/// « Reprendre » (le choix en surbrillance), puis « Valider ».
pub(super) fn reclaim_window(ctx: &egui::Context, window: &ReclaimWindow) -> Option<Command> {
    let mut clicked = None;
    egui::Window::new(&window.title)
        .id(egui::Id::unique("reclaim_window"))
        .min_width(title_min_width(ctx, &window.title))
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .show(ctx, |ui| {
            for line in &window.banners {
                ui.horizontal(|ui| {
                    ui.label(&line.text);
                    for (button, selected) in [(&line.keep, !line.taken), (&line.take, line.taken)]
                    {
                        if ui
                            .add(egui::Button::new(&button.label).selected(selected))
                            .clicked()
                        {
                            clicked = Some(button.command);
                        }
                    }
                });
            }
            if let Some(confirm) = &window.confirm
                && ui.button(&confirm.label).clicked()
            {
                clicked = Some(confirm.command);
            }
        });
    clicked
}

#[cfg(test)]
mod tests {
    use super::*;
    use arcanum_core::BasicColor;

    use crate::CrystalButton;

    /// Fait tourner `show` sur quelques images, puis dit si un texte affiché a été tronqué (« … »).
    /// egui fixe la largeur d'une fenêtre à taille automatique d'après l'image précédente.
    fn shows_elided_text(mut show: impl FnMut(&egui::Context)) -> bool {
        let ctx = egui::Context::default();
        let mut elided = false;
        for _ in 0..4 {
            let mut out = ctx.run_ui(egui::RawInput::default(), |ui| show(ui.ctx()));
            out.textures_delta.clear();
            elided = out.shapes.iter().any(|clipped| {
                matches!(&clipped.shape, egui::epaint::Shape::Text(text) if text.galley.elided)
            });
        }
        elided
    }

    fn empty_catalog() -> Catalog {
        Catalog::from_json_str(arcanum_i18n::Language("fr".into()), "{}").expect("catalogue vide")
    }

    // Issues privées 135 et 236 : un titre plus large que la largeur par défaut d'une fenêtre ne doit
    // pas être tronqué ; c'est le cas où la marge de la barre de titre compte.
    #[test]
    fn long_window_title_is_not_elided() {
        let title = "Récompense : choisissez la couleur du cristal (dé or)";
        assert!(!shows_elided_text(|ctx| {
            egui::Window::new(title)
                .min_width(title_min_width(ctx, title))
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.label("x");
                });
        }));
    }

    // Issue privée 135 : le titre long de la fenêtre du cristal ne doit pas être coupé.
    #[test]
    fn crystal_window_title_is_not_elided() {
        let window = CrystalChoice {
            title: "Récompense : choisissez la couleur du cristal (dé or)".into(),
            colors: vec![CrystalButton {
                color: BasicColor::Red,
                label: "R".into(),
                command: None,
                shortcut: None,
            }],
        };
        let catalog = empty_catalog();
        assert!(!shows_elided_text(|ctx| {
            crystal_window(ctx, &window, &catalog);
        }));
    }

    // Spec 056 : le titre du choix du cristal de Rituel de sang renforcé, vu tronqué sous Xvfb.
    #[test]
    fn blood_rite_crystal_window_title_is_not_elided() {
        let window = TokenChoice {
            title: "Rituel de sang : payer un mana de base pour un cristal ?".into(),
            tokens: Vec::new(),
            payments: Vec::new(),
            decline: None,
        };
        let catalog = empty_catalog();
        assert!(!shows_elided_text(|ctx| {
            token_window(ctx, &window, &catalog);
        }));
    }
}
