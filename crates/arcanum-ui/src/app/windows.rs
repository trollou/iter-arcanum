//! Fenêtres : chargement (spec 024), préférences (spec 080), « À propos » (spec #33) et fin de partie (spec 063) ; l'écran de nouvelle partie est
//! dans `new_game.rs` (spec 078).

use super::widgets::{enabled_shortcut_button, shortcut_button};
use eframe::egui::{self, Color32, Sense, TextStyle, Vec2, vec2};

use arcanum_i18n::Catalog;

use super::widgets::color_swatch;
use super::window_cards::{card_rows, level_card_rows, natural_width, window_limits};
use super::{IterArcanumApp, Thumbnails};
use crate::{
    AboutWindow, AltarWindow, Button, CARD_GAP, CardChoiceWindow, Command, CrystalChoice,
    LevelChoiceWindow, MeditationWindow, OffersWindow, PreferencesWindow, PreparationWindow,
    ReclaimWindow, ScoreWindow, SkillOffer, TacticCard, TacticDecision, TacticWindow, TokenChoice,
    row_width, volume_value_label,
};

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
        // Spec #33 : « À propos », exclusive des préférences (C7).
        if let Some(window) = self.client.about_window(&self.catalog) {
            clicked = about_window(ctx, &window, &self.catalog).or(clicked);
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
            // Spec #5 : la section « Affichage », en gras comme « Langue » et « Son ».
            ui.strong(&window.display_label);
            let mut fullscreen = window.fullscreen_at_launch;
            if ui
                .checkbox(&mut fullscreen, &window.fullscreen_label)
                .changed()
            {
                clicked = Some(Command::SetFullscreenAtLaunch(fullscreen));
            }
            ui.label(egui::RichText::new(&window.fullscreen_hint).weak());
            let mut animations = window.animations;
            if ui
                .checkbox(&mut animations, &window.animations_label)
                .changed()
            {
                clicked = Some(Command::SetAnimations(animations));
            }
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
            if close_button(ui, &window.close, catalog) {
                clicked = Some(window.close.command);
            }
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

/// Le bouton « Fermer » des préférences et d'« À propos », calé à droite ; `true` s'il est cliqué.
fn close_button(ui: &mut egui::Ui, close: &Button, catalog: &Catalog) -> bool {
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
        shortcut_button(ui, &close.label, close.shortcut.as_deref(), catalog).clicked()
    })
    .inner
}

/// Fenêtre « À propos » (spec #33, C4) : centrée, comme les préférences ; la croix fait comme
/// « Fermer ». Le lien « Code source » ouvre le navigateur.
fn about_window(ctx: &egui::Context, window: &AboutWindow, catalog: &Catalog) -> Option<Command> {
    let mut clicked = None;
    let mut open = true;
    // Plus large que les préférences : les lignes de licence tiennent sur une ligne.
    egui::Window::new(&window.title)
        .min_width(title_min_width(ctx, &window.title).max(480.0))
        .collapsible(false)
        .resizable(false)
        .open(&mut open)
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.heading(&window.name);
            ui.label(&window.version);
            ui.add_space(12.0);
            ui.label(&window.license_code);
            ui.label(&window.license_assets);
            ui.add_space(8.0);
            ui.label(egui::RichText::new(&window.unofficial).weak());
            ui.add_space(8.0);
            ui.add(egui::Hyperlink::from_label_and_url(
                &window.repository_label,
                &window.repository_url,
            ));
            ui.label(&window.sounds);
            ui.add_space(8.0);
            if close_button(ui, &window.close, catalog) {
                clicked = Some(window.close.command);
            }
        });
    if !open {
        clicked = Some(window.close.command);
    }
    clicked
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

/// Fenêtre Offres (spec 050), centrée : les cartes de la main en rangées (spec #32) ; actions
/// avancées et sorts côte à côte quand la largeur le permet, unités dessous (Q1) ; elle défile
/// plutôt que de dépasser l'écran (L2). « Prendre » pendant le choix d'une action avancée ; la
/// croix et « Fermer » seulement hors du choix.
pub(super) fn offers_window(
    ctx: &egui::Context,
    window: &OffersWindow,
    thumbnails: &Thumbnails,
    catalog: &Catalog,
) -> Option<Command> {
    let mut clicked = None;
    let mut open = true;
    let (max_width, max_height) = window_limits(ctx);
    let frame = egui::Window::new(&window.title)
        .id(egui::Id::unique("offers_window"))
        .min_width(title_min_width(ctx, &window.title))
        .max_width(max_width)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .open(&mut open);
    frame.show(ctx, |ui| {
        let id = egui::Id::unique("offers_rows");
        egui::ScrollArea::vertical()
            .max_height(max_height)
            .show(ui, |ui| {
                let (top, bottom) = window.columns.split_at(window.columns.len().min(2));
                let gap = 2.0 * CARD_GAP;
                let side_by_side = top
                    .iter()
                    .map(|column| natural_width(&column.cards))
                    .sum::<f32>()
                    + gap
                    <= max_width;
                let mut offer =
                    |ui: &mut egui::Ui, index: usize, width: f32, peer: Option<usize>| {
                        let column = &window.columns[index];
                        ui.vertical(|ui| {
                            ui.strong(&column.title);
                            let shown = card_rows(
                                ui,
                                id.with(index),
                                peer.map(|peer| id.with(peer)),
                                &column.cards,
                                width,
                                thumbnails,
                                catalog,
                            );
                            clicked = shown.or(clicked);
                        });
                    };
                if side_by_side {
                    ui.horizontal_top(|ui| {
                        ui.spacing_mut().item_spacing.x = gap;
                        for (index, column) in top.iter().enumerate() {
                            offer(ui, index, natural_width(&column.cards), Some(1 - index));
                        }
                    });
                } else {
                    for index in 0..top.len() {
                        offer(ui, index, max_width, None);
                    }
                }
                for index in top.len()..top.len() + bottom.len() {
                    offer(ui, index, max_width, None);
                }
            });
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

pub(super) fn card_choice_window(
    ctx: &egui::Context,
    window: &CardChoiceWindow,
    id: &'static str,
    thumbnails: &Thumbnails,
    catalog: &Catalog,
) -> Option<Command> {
    let (max_width, max_height) = window_limits(ctx);
    egui::Window::new(&window.title)
        .id(egui::Id::unique(id))
        .min_width(title_min_width(ctx, &window.title))
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .max_width(max_width)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .max_height(max_height)
                .show(ui, |ui| {
                    let id = egui::Id::unique(("choice_rows", id));
                    let mut clicked =
                        card_rows(ui, id, None, &window.cards, max_width, thumbnails, catalog);
                    // Spec #20 : « Renoncer » ou « Annuler » sous les cartes.
                    if let Some(close) = &window.close
                        && shortcut_button(ui, &close.label, close.shortcut.as_deref(), catalog)
                            .clicked()
                    {
                        clicked = Some(close.command);
                    }
                    clicked
                })
                .inner
        })
        .and_then(|response| response.inner)
        .flatten()
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

/// Largeur d'une tuile de compétence : celle d'une carte, cadre compris (maquette de la spec #20).
const SKILL_TILE_WIDTH: f32 = crate::CARD_WIDTH - 16.0;

/// Hauteur visée de la zone qui défile de la fenêtre du niveau pair : compétences et une rangée
/// de cartes en 1600 × 900.
const LEVEL_WINDOW_HEIGHT: f32 = 560.0;

/// Fenêtre du niveau pair (spec #20, maquette) : en haut du plateau, compétences révélées et
/// offre commune côte à côte (la note sous l'offre commune), une tuile par compétence qu'un clic
/// encadre comme une tactique ; dessous, « Actions avancées », l'aide tant que rien n'est encadré,
/// et l'offre en cartes hors zone, « Prendre » grisé pour les cartes que la compétence ne laisse pas.
pub(super) fn level_choice_window(
    ctx: &egui::Context,
    window: &LevelChoiceWindow,
    thumbnails: &Thumbnails,
    catalog: &Catalog,
) -> Option<Command> {
    let mut clicked = None;
    let (max_width, max_height) = window_limits(ctx);
    egui::Window::new(&window.title)
        .id(egui::Id::unique("level_choice_window"))
        .min_width(title_min_width(ctx, &window.title))
        .max_width(max_width)
        .collapsible(false)
        .resizable(false)
        // En haut : la main reste visible dessous, comme pour la tactique (spec 060).
        .anchor(egui::Align2::CENTER_TOP, vec2(0.0, 40.0))
        .show(ctx, |ui| {
            // Sans hauteur minimale, egui ne donne à la zone qui défile que sa hauteur de
            // l'image précédente : les cartes seraient coupées (L1).
            egui::ScrollArea::vertical()
                .max_height(max_height)
                .min_scrolled_height(max_height.min(LEVEL_WINDOW_HEIGHT))
                .show(ui, |ui| {
                    ui.horizontal_top(|ui| {
                        if !window.revealed.is_empty() {
                            ui.vertical(|ui| {
                                ui.strong(&window.revealed_title);
                                clicked = skill_row(ui, &window.revealed, catalog).or(clicked);
                            });
                        }
                        if !window.common.is_empty() {
                            ui.add_space(16.0);
                            ui.vertical(|ui| {
                                let tiles = window.common.len() as f32;
                                ui.set_max_width(crate::CARD_WIDTH * tiles);
                                ui.strong(&window.common_title);
                                clicked = skill_row(ui, &window.common, catalog).or(clicked);
                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(&window.common_note).weak(),
                                    )
                                    .wrap(),
                                );
                            });
                        }
                    });
                    if !window.advanced_actions.is_empty() {
                        ui.separator();
                        ui.horizontal(|ui| {
                            ui.strong(&window.advanced_actions_title);
                            if let Some(hint) = &window.hint {
                                let color = ui.visuals().warn_fg_color;
                                ui.label(egui::RichText::new(hint).color(color));
                            }
                        });
                        let id = egui::Id::unique("level_choice_rows");
                        let width = row_width(window.advanced_actions.len()).min(max_width);
                        if let Some(command) = level_card_rows(
                            ui,
                            id,
                            &window.advanced_actions,
                            width,
                            thumbnails,
                            catalog,
                        ) {
                            clicked = Some(command);
                        }
                    }
                });
        });
    clicked
}

/// Une rangée de tuiles de compétence ; rend la commande cliquée (encadrer, ou « Prendre » quand
/// l'offre d'actions avancées est vide, C7).
fn skill_row(ui: &mut egui::Ui, offers: &[SkillOffer], catalog: &Catalog) -> Option<Command> {
    let mut clicked = None;
    ui.horizontal_top(|ui| {
        for offer in offers {
            clicked = skill_tile(ui, offer, catalog).or(clicked);
        }
    });
    clicked
}

/// Une tuile de compétence : nom, fréquence, texte, encadrée comme une tactique (spec 060) ;
/// grisée quand elle ne se prend pas (compétence du héros dans l'offre commune).
fn skill_tile(ui: &mut egui::Ui, offer: &SkillOffer, catalog: &Catalog) -> Option<Command> {
    let mut clicked = None;
    let visuals = ui.visuals();
    let stroke = if offer.selected {
        egui::Stroke::new(2.5, visuals.selection.stroke.color)
    } else {
        egui::Stroke::new(1.0, visuals.widgets.noninteractive.bg_stroke.color)
    };
    let fill = if offer.selected {
        visuals.selection.bg_fill.gamma_multiply(0.4)
    } else {
        Color32::TRANSPARENT
    };
    let response = egui::Frame::group(ui.style())
        .stroke(stroke)
        .fill(fill)
        .show(ui, |ui| {
            ui.set_width(SKILL_TILE_WIDTH);
            ui.vertical(|ui| {
                ui.set_width(SKILL_TILE_WIDTH);
                // La tuile entière se clique : pas de curseur de sélection de texte dessus.
                ui.style_mut().interaction.selectable_labels = false;
                ui.add_enabled_ui(offer.select.is_some() || offer.take.is_some(), |ui| {
                    ui.horizontal(|ui| {
                        ui.strong(&offer.name);
                        ui.weak(&offer.usage);
                    });
                    ui.add(egui::Label::new(&offer.text).wrap());
                });
                if let Some(take) = &offer.take
                    && shortcut_button(ui, &take.label, take.shortcut.as_deref(), catalog).clicked()
                {
                    clicked = Some(take.command);
                }
                if let Some(note) = &offer.note {
                    ui.add(egui::Label::new(egui::RichText::new(note).weak().italics()).wrap());
                }
            });
        })
        .response;
    if let Some(select) = offer.select {
        let response = ui.interact(response.rect, response.id.with("skill"), Sense::click());
        if response
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .clicked()
        {
            clicked = clicked.or(Some(select));
        }
    }
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
