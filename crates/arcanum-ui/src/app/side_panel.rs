//! Panneau latéral : bandeaux, héros, zone de jeu et boutons.

use super::bottom_panel::bottom_hint;
use super::widgets::shortcut_button;
use eframe::egui::{self, Color32, Sense, Vec2};

use super::board::standing_image_rect;
use super::widgets::color_swatch;
use super::windows::{
    altar_window, artifact_window, crystal_window, meditation_window, offers_window,
    preparation_window, reclaim_window, score_window, skill_choice_window, tactic_decision_window,
    tactic_window, token_window,
};
use super::{IterArcanumApp, Thumbnails, full_uv};
use crate::{ArtifactChoice, Command, DummySection, GameScreen, HeroPortrait, ImageId, ImagePaint};

/// Portrait du héros dans le panneau (spec 065).
const PORTRAIT_SIZE: f32 = 48.0;
/// Orange de l'annonce de la fin de la manche par le fantôme (spec 061, retour de Mathieu).
const ANNOUNCEMENT: Color32 = Color32::from_rgb(0xF0, 0x8C, 0x2A);
/// Portrait du héros du fantôme, plus petit (spec 061).
const DUMMY_PORTRAIT_SIZE: f32 = 32.0;

/// Bandeaux du haut du panneau : progression des cités (spec 064, C28), dernier tour (spec 063),
/// niveaux gagnés (spec 035), nouvelle manche et ordre du tour (spec 061, C39).
fn banners(ui: &mut egui::Ui, screen: &GameScreen) {
    let strong = [&screen.last_turn, &screen.last_turn_next]
        .into_iter()
        .flatten()
        .chain(&screen.level_gained);
    let plain = [&screen.new_round, &screen.round_order]
        .into_iter()
        .flatten();
    if let Some(progress) = &screen.cities_progress {
        ui.label(progress);
    }
    for line in strong {
        ui.add(egui::Label::new(egui::RichText::new(line).strong()).wrap());
    }
    for line in plain {
        ui.add(egui::Label::new(line).wrap());
    }
}

impl IterArcanumApp {
    /// Le milieu du panneau de droite (spec 073) : bandeaux, joueur, zone de jeu et ce
    /// tour ; la section Ici est remplacée par les boutons du plateau (spec 075).
    fn right_panel_body(&mut self, ui: &mut egui::Ui, screen: &GameScreen) -> Option<Command> {
        let mut clicked = None;
        let button = |app: &Self, ui: &mut egui::Ui, button: &crate::Button| {
            shortcut_button(ui, &button.label, button.shortcut.as_deref(), &app.catalog)
                .clicked()
                .then_some(button.command)
        };
        banners(ui, screen);
        if let Some(message) = &screen.message {
            ui.strong(message);
        }
        if let Some(reopen) = &screen.reopen_score {
            clicked = button(self, ui, reopen).or(clicked);
        }
        // Spec #10, C8 : la Source est dans la barre de menu ; un seul trait avant le joueur.
        ui.separator();
        // Joueur (C23, C24).
        if let Some(portrait) = &screen.portrait {
            self.portrait(ui, portrait, screen);
        }
        self.inventory_line(ui, &screen.inventory);
        self.icon_line(ui, &screen.level_stats, false);
        ui.add(egui::Label::new(&screen.reputation).wrap());
        // Spec 062, C40 : le bonus des Boucliers d'une cité, sous la réputation.
        if let Some(line) = &screen.city_shields {
            ui.label(line);
        }
        self.icon_line(ui, &screen.points, true);
        // Spec 058, C18 : l'Influence d'interaction de Fortune clinquante.
        if let Some(line) = &screen.interaction_influence {
            ui.label(line);
        }
        // Spec 060 : la tactique, son texte en infobulle, ce qu'elle porte et ses boutons.
        if let Some(tactic) = &screen.tactic {
            ui.horizontal_wrapped(|ui| {
                ui.strong(format!("{} · {}", tactic.number, tactic.name))
                    .on_hover_text(&tactic.text);
                if let Some(flipped) = &tactic.flipped {
                    ui.weak(flipped);
                }
            });
            for note in &tactic.notes {
                ui.add(egui::Label::new(note).wrap());
            }
            for tactic_button in &tactic.buttons {
                clicked = button(self, ui, tactic_button).or(clicked);
            }
        }
        // Spec 075, C32 : les soins hors combat, sous les points ; les actions du lieu sont sur
        // le plateau.
        for healing in &screen.healing {
            clicked = button(self, ui, healing).or(clicked);
        }
        for line in [&screen.fame_pending, &screen.keep_hand_limit]
            .into_iter()
            .flatten()
        {
            ui.add(egui::Label::new(line).wrap());
        }
        ui.horizontal(|ui| {
            ui.label(&screen.deck);
            let discard = &screen.discard_button;
            let hint = bottom_hint(None, discard.shortcut.as_deref(), &self.catalog);
            if ui
                .add_enabled(discard.command.is_some(), egui::Button::new(&discard.label))
                .on_hover_text(&hint)
                .on_disabled_hover_text(&hint)
                .clicked()
            {
                clicked = discard.command;
            }
        });
        if let Some(set_aside) = &screen.set_aside {
            ui.add(egui::Label::new(set_aside).wrap());
        }
        // Spec 061, C37 : le joueur fantôme, sous le joueur.
        if let Some(dummy) = &screen.dummy {
            ui.separator();
            clicked = self.dummy_section(ui, dummy).or(clicked);
        }
        // Zone de jeu : jetons, cartes jouées (C26, C27).
        ui.separator();
        ui.heading(&screen.play_area_title);
        if let Some(tokens) = &screen.tokens {
            self.token_line(ui, tokens);
        }
        for line in &screen.play_area {
            // Ligne repliée : une ligne longue ne doit pas élargir le panneau.
            ui.horizontal_top(|ui| {
                color_swatch(ui, line.color);
                ui.add(egui::Label::new(&line.text).wrap())
                    .on_hover_text(&line.tooltip);
            });
        }
        if let Some(empty) = &screen.play_area_empty {
            ui.weak(empty);
        }
        // Spec 052, C4 : les effets en cours, sous les cartes jouées.
        if let Some(list) = &screen.modifiers {
            ui.strong(&list.title);
            for line in &list.lines {
                ui.add(egui::Label::new(line).wrap());
            }
        }
        // Ce tour : les faits, puis les récompenses en attente (C28, C29).
        ui.strong(&screen.this_turn_title);
        for line in &screen.turn_facts {
            ui.add(egui::Label::new(line).wrap());
        }
        for line in [
            &screen.reward_pending,
            &screen.spell_reward_pending,
            &screen.ruin_reward_pending,
            &screen.reward_die,
            &screen.crystal_reward_pending,
            &screen.glade_token,
            &screen.revealed_ruins,
            &screen.withdrawal,
        ]
        .into_iter()
        .flatten()
        {
            ui.add(egui::Label::new(line).wrap());
        }
        if let Some(nothing) = &screen.nothing_this_turn {
            ui.weak(nothing);
        }
        clicked
    }

    pub(super) fn side_panel(&mut self, ui: &mut egui::Ui, screen: &GameScreen) {
        let Some(game) = self.client.game() else {
            return;
        };
        let offers = game.offers_window(&self.catalog);
        let discard = game.discard_window(&self.catalog);
        let mut clicked = None;
        // Spec 073 : les boutons du bas restent visibles, le milieu défile.
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                for button in &screen.bottom_buttons {
                    let hint = bottom_hint(
                        button.tooltip.as_deref(),
                        button.shortcut.as_deref(),
                        &self.catalog,
                    );
                    let response = ui
                        .add_enabled(button.command.is_some(), egui::Button::new(&button.label))
                        .on_hover_text(&hint)
                        .on_disabled_hover_text(&hint);
                    if response.clicked() {
                        clicked = button.command;
                    }
                }
            });
            ui.separator();
            ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    clicked = self.right_panel_body(ui, screen).or(clicked);
                });
            });
        });
        // Spec 050 : la fenêtre Offres, ouverte d'office pendant le choix d'une action avancée.
        if let Some(window) = &offers {
            let illustrations = window.columns.iter().flat_map(|column| &column.cards);
            let thumbnails = self.thumbnails(ui.ctx(), illustrations.map(|card| card.illustration));
            if let Some(command) = offers_window(ui.ctx(), window, &thumbnails, &self.catalog) {
                clicked = Some(command);
            }
        }
        // Spec 073 : la fenêtre de la défausse.
        if let Some(window) = &discard {
            self.discard_window(ui.ctx(), window, screen);
        }
        // Spec 045 : la fenêtre de l'offrande à l'autel.
        if let Some(window) = &screen.altar_window
            && let Some(command) = altar_window(ui.ctx(), window, &self.catalog)
        {
            clicked = Some(command);
        }
        // Spec 063 : la fenêtre de fin, centrée sur le plateau de jeu.
        // Spec 044 : la couleur du cristal d'un dé or, avant l'artefact.
        if let Some(window) = &screen.crystal_choice
            && let Some(command) = crystal_window(ui.ctx(), window, &self.catalog)
        {
            clicked = Some(command);
        }
        // Spec 055 : les bannières à reprendre à la fin de la manche.
        if let Some(window) = &screen.banner_reclaim
            && let Some(command) = reclaim_window(ui.ctx(), window)
        {
            clicked = Some(command);
        }
        // Spec 056 : le choix en attente de Rituel de sang renforcé.
        if let Some(window) = &screen.blood_ritual
            && let Some(command) = token_window(ui.ctx(), window, &self.catalog)
        {
            clicked = Some(command);
        }
        // Spec 060 : le choix de la tactique, Préparation, Provisions et Longue Nuit.
        if let Some(window) = &screen.tactic_choice
            && let Some(command) = tactic_window(ui.ctx(), window, &self.catalog)
        {
            clicked = Some(command);
        }
        if let Some(window) = &screen.preparation
            && let Some(command) = preparation_window(ui.ctx(), window)
        {
            clicked = Some(command);
        }
        if let Some(window) = &screen.tactic_decision
            && let Some(command) = tactic_decision_window(ui.ctx(), window)
        {
            clicked = Some(command);
        }
        // Spec 057 : les cartes de Méditation qui attendent leur place.
        if let Some(window) = &screen.meditation
            && let Some(command) = meditation_window(ui.ctx(), window, &self.catalog)
        {
            clicked = Some(command);
        }
        // Spec 041 : la fenêtre du choix de l'artefact, à la fin du tour.
        if let Some(window) = &screen.artifact_choice {
            let thumbnails = self.choice_thumbnails(ui.ctx(), window);
            if let Some(command) = artifact_window(
                ui.ctx(),
                window,
                "artifact_window",
                &thumbnails,
                &self.catalog,
            ) {
                clicked = Some(command);
            }
        }
        // Spec 043 : la fenêtre du choix du sort, comme celle de l'artefact, qui passe avant.
        if screen.artifact_choice.is_none()
            && let Some(window) = &screen.spell_choice
        {
            let thumbnails = self.choice_thumbnails(ui.ctx(), window);
            if let Some(command) =
                artifact_window(ui.ctx(), window, "spell_window", &thumbnails, &self.catalog)
            {
                clicked = Some(command);
            }
        }
        // Spec 051 : la fenêtre du choix de compétence d'un niveau pair.
        if let Some(window) = &screen.skill_choice
            && let Some(command) = skill_choice_window(ui.ctx(), window, &self.catalog)
        {
            clicked = Some(command);
        }
        if let Some(window) = &screen.score_window
            && let Some(command) = score_window(ui.ctx(), window, &self.catalog)
        {
            clicked = Some(command);
        }
        if let Some(command) = clicked {
            self.client.command(command);
            ui.ctx().request_repaint();
        }
    }

    /// Les vignettes d'une fenêtre de choix de l'artefact ou du sort (spec 072, C6).
    fn choice_thumbnails(&mut self, ctx: &egui::Context, window: &ArtifactChoice) -> Thumbnails {
        self.thumbnails(ctx, window.cards.iter().map(|card| card.illustration))
    }

    /// La section du joueur fantôme (spec 061, C37) : petit portrait et nom, puis son tempo.
    fn dummy_section(&mut self, ui: &mut egui::Ui, dummy: &DummySection) -> Option<Command> {
        let mut clicked = None;
        ui.horizontal(|ui| {
            self.figure(
                ui,
                dummy.portrait.image,
                dummy.portrait.fallback,
                DUMMY_PORTRAIT_SIZE,
            );
            ui.vertical(|ui| {
                ui.strong(&dummy.title);
                if let Some(tactic) = &dummy.tactic {
                    ui.label(tactic);
                }
            });
        });
        ui.horizontal(|ui| {
            ui.strong(&dummy.deck);
            let discard = &dummy.discard;
            if ui
                .add_enabled(discard.command.is_some(), egui::Button::new(&discard.label))
                .clicked()
            {
                clicked = discard.command;
            }
        });
        self.dummy_crystals(ui, &dummy.crystals);
        for line in [&dummy.skills, &dummy.last_turn] {
            ui.add(egui::Label::new(line).wrap());
        }
        if let Some(announced) = &dummy.announced {
            ui.add(
                egui::Label::new(egui::RichText::new(announced).strong().color(ANNOUNCEMENT))
                    .wrap(),
            );
        }
        clicked
    }

    /// Une figurine debout dans un carré de `size` points, ou un disque de sa couleur (spec 075).
    fn figure(&mut self, ui: &mut egui::Ui, image: ImageId, fallback: Color32, size: f32) {
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
        let pixels_per_point = ui.ctx().pixels_per_point();
        match self.paint_of(ui, image, fallback, rect, pixels_per_point) {
            ImagePaint::Texture(texture) => {
                let size = ui
                    .ctx()
                    .tex_manager()
                    .read()
                    .meta(texture)
                    .map(|meta| meta.size);
                let figure = standing_image_rect(rect, size);
                ui.painter()
                    .image(texture, figure, full_uv(), Color32::WHITE);
            }
            ImagePaint::Fallback(color) | ImagePaint::Pending(color) => {
                ui.painter()
                    .circle_filled(rect.center(), rect.width() / 2.0, color);
            }
        }
    }

    /// Portrait du héros, son nom et « Niveau 3 » ; à droite, la Gloire (spec 073, C23).
    fn portrait(&mut self, ui: &mut egui::Ui, portrait: &HeroPortrait, screen: &GameScreen) {
        ui.horizontal(|ui| {
            // La figurine debout, à ses proportions (spec 075).
            self.figure(ui, portrait.image, portrait.fallback, PORTRAIT_SIZE);
            ui.vertical(|ui| {
                ui.strong(&portrait.name);
                ui.label(&screen.player_level);
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                self.icon_value(ui, &screen.player_fame, true);
            });
        });
    }
}
