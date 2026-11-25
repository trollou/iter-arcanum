//! Application eframe : l'état d'affichage, les raccourcis et l'assemblage de l'écran.
//! Chaque zone est dessinée dans son fichier (issue privée 99).

use std::collections::BTreeMap;
use std::path::PathBuf;
use widgets::shortcut_button;

use arcanum_audio::{AudioDirector, AudioPlayer, SilentPlayer, SoundCategory};
use arcanum_i18n::Catalog;
use arcanum_persistance::preferences::Volumes;
use eframe::egui::{
    self, Color32, Frame, Modifiers, Pos2, Rect, TextureFilter, TextureOptions, Vec2,
    load::SizeHint, load::SizedTexture, load::TexturePoll, pos2,
};

use crate::{
    BOARD_BACKGROUND, BoardCamera, ClientSession, Illustration, IllustrationIndex, ImageId,
    ImageLoad, ImagePaint, ImageWarnings, KEY_BINDINGS, KeyModifier, SourceLine, WALK_STEP_SECONDS,
    WindowRequest, catalog_or_empty, illustration_paint, image_paint, svg_size,
};

/// Textures des illustrations à dessiner dans une fenêtre (spec 072) : seulement celles chargées.
type Thumbnails = BTreeMap<Illustration, SizedTexture>;

mod board;
mod bottom_panel;
mod combat;
mod icons;
mod menu_bar;
mod new_game;
mod side_panel;
mod widgets;
mod window_cards;
mod windows;

const SIDE_PANEL_WIDTH: f32 = 340.0;
const CHOICE_STROKE: Color32 = Color32::from_rgb(0xF2, 0xC9, 0x4C);
const SWATCH_STROKE: Color32 = Color32::from_rgb(0x9A, 0xA0, 0xA6);
const TILE_STROKE: Color32 = Color32::from_rgb(0x2B, 0x22, 0x1A);
/// Anneau d'un pas qui provoque un ennemi errant (spec 030, C24).
const PROVOKE_STROKE: Color32 = Color32::from_rgb(0xF2, 0x8C, 0x28);
/// Texte des repères d'éléments et de résistances (spec 032) : gris bleuté, comme la maquette.
const RESISTANCE_TEXT: Color32 = Color32::from_rgb(0x9a, 0xa7, 0xb4);

/// Application eframe : fait avancer la partie, puis dessine l'écran.
/// Aucune logique ici : tout est dans `ClientSession`, `GameSession`, `game_screen`,
/// `new_game_screen`, `BoardCamera` et `board_screen`, testés sans fenêtre.
pub struct IterArcanumApp {
    client: ClientSession,
    /// Partie dont le plateau de jeu est cadré : une autre partie le recadre (spec 065).
    framed_generation: u64,
    /// Hauteur du bandeau du choix à l'image précédente : l'écran de combat la laisse libre en
    /// bas de son défilement, pour que le bandeau ne cache rien (issue privée 177).
    banner_height: f32,
    catalog: Catalog,
    /// Calculé à la première image, quand la taille de la zone est connue.
    camera: Option<BoardCamera>,
    recenter_requested: bool,
    images_dir: Option<PathBuf>,
    image_warnings: ImageWarnings,
    /// Les illustrations présentes, lues au lancement (spec 072, C4).
    illustrations: IllustrationIndex,
    /// Essai d'animation (spec 075, C35) : le déplacement en cours de glissement, par son numéro,
    /// et l'instant où il a commencé.
    walk_start: Option<(u64, f64)>,
    /// Spec #5 : la dernière disparition vue et l'instant où son effacement a commencé.
    departure_start: Option<(u64, f64)>,
    /// Issue #50 : facteur de ralenti des animations du plateau, 1 par défaut.
    animation_scale: f64,
    /// Dossiers des ressources candidats : le catalogue d'une autre langue y est relu (spec 080).
    assets_candidates: Vec<PathBuf>,
    /// Spec 081 : le lecteur (Kira, ou muet) et le directeur qui lui donne ses ordres.
    audio: Box<dyn AudioPlayer>,
    director: AudioDirector,
    /// Spec 081 : instant où enregistrer un volume réglé au clavier (C21).
    volume_save_at: Option<f64>,
}

impl IterArcanumApp {
    pub fn new(client: ClientSession, catalog: Catalog, images_dir: Option<PathBuf>) -> Self {
        let illustrations = images_dir
            .as_deref()
            .map(IllustrationIndex::load)
            .unwrap_or_default();
        Self {
            framed_generation: client.generation(),
            banner_height: 0.0,
            client,
            catalog,
            camera: None,
            recenter_requested: false,
            images_dir,
            image_warnings: ImageWarnings::default(),
            illustrations,
            walk_start: None,
            departure_start: None,
            animation_scale: 1.0,
            assets_candidates: Vec::new(),
            audio: Box::new(SilentPlayer),
            director: AudioDirector::new(std::time::Duration::from_secs_f64(WALK_STEP_SECONDS)),
            volume_save_at: None,
        }
    }

    /// Issue #50 : les animations du plateau ralenties de ce facteur (`ARCANUM_ANIMATION_SCALE`).
    pub fn with_animation_scale(mut self, scale: f64) -> Self {
        self.animation_scale = scale;
        self
    }

    /// Spec 081 : le lecteur créé au lancement, réglé aux volumes des préférences.
    pub fn with_audio(mut self, audio: Box<dyn AudioPlayer>, volumes: Volumes) -> Self {
        self.audio = audio;
        self.apply_volumes(volumes);
        self
    }

    /// Transmet les quatre volumes au lecteur.
    fn apply_volumes(&mut self, volumes: Volumes) {
        for (category, volume) in [
            (SoundCategory::Master, volumes.master),
            (SoundCategory::Music, volumes.music),
            (SoundCategory::Ambience, volumes.ambience),
            (SoundCategory::Effects, volumes.effects),
        ] {
            for command in self.director.set_volume(category, volume) {
                self.audio.execute(&command);
            }
        }
    }

    /// Spec 081 : les événements nouveaux et la situation de l'image, traduits en ordres de son.
    fn direct_sound(&mut self) {
        let events = self.client.take_heard_events();
        let situation = self.client.sound_situation();
        for command in self.director.update(situation, &events) {
            self.audio.execute(&command);
        }
    }

    /// Les dossiers où relire un catalogue quand le joueur change de langue (spec 080).
    pub fn with_assets_candidates(mut self, candidates: Vec<PathBuf>) -> Self {
        self.assets_candidates = candidates;
        self
    }

    /// Raccourcis clavier : Échap quitte le mode en cours (specs 013 et 016), Fin ouvre le mode
    /// fin du tour, Entrée le confirme (spec 016) ; la session décide quand chacune agit.
    fn shortcuts(&mut self, ui: &egui::Ui) {
        if !ui.ctx().egui_wants_keyboard_input() {
            // La table des touches (spec 071, C25) : Ctrl+… consommées, pour qu'egui n'en fasse
            // rien d'autre ; O sans modificateur (Ctrl+O charge).
            let mut shortcuts = ui.input_mut(|input| {
                KEY_BINDINGS
                    .iter()
                    .filter(|binding| binding.modifier == KeyModifier::Ctrl)
                    .filter(|binding| input.consume_key(Modifiers::COMMAND, binding.key))
                    .map(|binding| binding.shortcut)
                    .collect::<Vec<_>>()
            });
            shortcuts.extend(ui.input(|input| {
                let plain = input.modifiers.is_none();
                KEY_BINDINGS
                    .iter()
                    .filter(|binding| match binding.modifier {
                        KeyModifier::Ctrl => false,
                        KeyModifier::None => plain,
                        KeyModifier::Any => true,
                    })
                    .filter(|binding| input.key_pressed(binding.key))
                    .map(|binding| binding.shortcut)
                    .collect::<Vec<_>>()
            }));
            for shortcut in shortcuts {
                self.client.shortcut(shortcut);
                ui.ctx().request_repaint();
            }
        }
    }

    /// Une illustration ronde dans `rect`, ou un disque de sa couleur de repli.
    fn paint_image(&mut self, ui: &egui::Ui, image: ImageId, fallback: Color32, rect: Rect) {
        let pixels_per_point = ui.ctx().pixels_per_point();
        match self.paint_of(ui, image, fallback, rect, pixels_per_point) {
            ImagePaint::Texture(texture) => {
                ui.painter().image(texture, rect, full_uv(), Color32::WHITE);
            }
            ImagePaint::Fallback(color) | ImagePaint::Pending(color) => {
                ui.painter()
                    .circle_filled(rect.center(), rect.width() / 2.0, color);
            }
        }
    }

    /// Image convertie à une taille de la série (spec 011), sinon couleur de repli.
    fn paint_of(
        &mut self,
        ui: &egui::Ui,
        image: ImageId,
        fallback: Color32,
        rect: Rect,
        pixels_per_point: f32,
    ) -> ImagePaint {
        let Some(images_dir) = &self.images_dir else {
            return ImagePaint::Fallback(fallback);
        };
        let path = image.path(images_dir);
        let uri = format!("file://{}", path.display());
        let height = svg_size(rect.height() * pixels_per_point);
        let load =
            match ui
                .ctx()
                .try_load_texture(&uri, TextureOptions::LINEAR, SizeHint::Height(height))
            {
                Ok(TexturePoll::Ready { texture }) => ImageLoad::Ready(texture.id),
                Ok(TexturePoll::Pending { .. }) => ImageLoad::Pending,
                Err(error) => ImageLoad::Failed(error.to_string()),
            };
        image_paint(load, image, fallback, &path, &mut self.image_warnings)
    }
}

impl IterArcanumApp {
    /// La texture d'une illustration, si son PNG est présent et chargé (spec 072). Une image
    /// absente n'est jamais demandée : rien dans les logs (C9).
    fn illustration_texture(
        &mut self,
        ctx: &egui::Context,
        illustration: Illustration,
    ) -> Option<SizedTexture> {
        if !self.illustrations.contains(illustration) {
            return None;
        }
        let path = illustration.path(self.images_dir.as_ref()?);
        let uri = format!("file://{}", path.display());
        // Mipmaps : le PNG de 512 px reste net réduit en vignette.
        let options = TextureOptions {
            mipmap_mode: Some(TextureFilter::Linear),
            ..TextureOptions::LINEAR
        };
        // Spec #9 : la taille de l'image décide de son recadrage dans la bande.
        let mut size = Vec2::ZERO;
        let load = match ctx.try_load_texture(&uri, options, SizeHint::default()) {
            Ok(TexturePoll::Ready { texture }) => {
                size = texture.size;
                ImageLoad::Ready(texture.id)
            }
            Ok(TexturePoll::Pending { .. }) => ImageLoad::Pending,
            Err(error) => ImageLoad::Failed(error.to_string()),
        };
        illustration_paint(load, &path, &mut self.image_warnings)
            .map(|id| SizedTexture::new(id, size))
    }

    /// Les textures chargées de ces illustrations, pour une fenêtre dessinée hors de l'application.
    fn thumbnails(
        &mut self,
        ctx: &egui::Context,
        illustrations: impl IntoIterator<Item = Illustration>,
    ) -> Thumbnails {
        illustrations
            .into_iter()
            .filter_map(|illustration| {
                let texture = self.illustration_texture(ctx, illustration)?;
                Some((illustration, texture))
            })
            .collect()
    }
}

fn full_uv() -> Rect {
    Rect::from_min_max(pos2(0.0, 0.0), Pos2::new(1.0, 1.0))
}

impl eframe::App for IterArcanumApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.client.run_frame() {
            ctx.request_repaint();
        }
        self.direct_sound();
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        // Spec 074 : l'état de la fenêtre pour la case Plein écran, et la croix qui fait comme
        // Quitter (C16).
        let (fullscreen, close) = ctx.input(|input| {
            let viewport = input.viewport();
            (viewport.fullscreen, viewport.close_requested())
        });
        if let Some(fullscreen) = fullscreen {
            self.client.set_fullscreen(fullscreen);
        }
        if close && !self.client.window_close_requested() {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        }
        self.screen(ui);
        // Les demandes de la session, transmises à la fenêtre (C11 à C13).
        for request in self.client.take_window_requests() {
            match request {
                WindowRequest::Recenter => self.recenter_requested = true,
                WindowRequest::Fullscreen(on) => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(on));
                }
                WindowRequest::Close => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
                // Spec 080 : le catalogue de la langue choisie, et le titre de la fenêtre.
                WindowRequest::Language(language) => {
                    self.catalog = catalog_or_empty(&self.assets_candidates, &language);
                    ctx.send_viewport_cmd(egui::ViewportCommand::Title(
                        self.catalog.text("app.title"),
                    ));
                }
                // Spec 081 : les volumes au lecteur, l'échantillon du curseur tenu.
                WindowRequest::Volumes(volumes) => self.apply_volumes(volumes),
                WindowRequest::Preview(category) => {
                    for command in self.director.preview(category) {
                        self.audio.execute(&command);
                    }
                }
            }
            ctx.request_repaint();
        }
    }
}

impl IterArcanumApp {
    /// L'écran d'une image : lancement, ou partie avec sa barre, ses panneaux et ses fenêtres.
    fn screen(&mut self, ui: &mut egui::Ui) {
        self.shortcuts(ui);
        // Une autre partie : le plateau de jeu est recadré sur elle (spec 065).
        if self.client.generation() != self.framed_generation {
            self.framed_generation = self.client.generation();
            self.camera = None;
        }
        let ctx = ui.ctx().clone();
        // Au lancement : l'écran de nouvelle partie seul (spec 078), sans plateau ni panneau.
        if self.client.game().is_none() {
            self.new_game_central(ui);
            self.windows(&ctx);
            return;
        }
        // Spec 078, Q4 : en cours de partie, l'écran de nouvelle partie prend toute la place sous
        // la barre de menu ; le plateau et les panneaux ne sont pas dessinés.
        if self.client.new_game_page(&self.catalog).is_some() {
            // L3 : la barre sans la Source, comme le plateau et les panneaux.
            self.menu_bar_panel(ui, None);
            self.new_game_central(ui);
            self.windows(&ctx);
            return;
        }
        let Some(game) = self.client.game() else {
            return;
        };
        let combat = game.combat_screen(&self.catalog);
        // Calculé une fois par image, pour les panneaux, le plateau et le bandeau (issues privées 204 et 212).
        let Some(game_screen) = self.client.game_screen(&self.catalog) else {
            return;
        };
        self.menu_bar_panel(ui, Some(&game_screen.source));
        // Spec 073 : le panneau du bas sur toute la largeur, le panneau de droite au-dessus.
        egui::Panel::bottom("bottom_panel")
            .resizable(false)
            .show(ui, |ui| self.bottom_panel(ui, &game_screen));
        egui::Panel::right("side_panel")
            .resizable(false)
            .exact_size(SIDE_PANEL_WIDTH)
            .show(ui, |ui| self.side_panel(ui, &game_screen));
        // Spec 031, Q11 : pendant un combat, l'écran de combat prend la place du plateau de jeu.
        let central = egui::CentralPanel::default()
            .frame(Frame::NONE.fill(BOARD_BACKGROUND))
            .show(ui, |ui| match &combat {
                Some(combat) if !combat.viewing_board => self.combat_area(ui, combat),
                Some(combat) => {
                    let clicked = ui
                        .horizontal(|ui| {
                            ui.add_space(8.0);
                            shortcut_button(
                                ui,
                                &combat.board_toggle.label,
                                combat.board_toggle.shortcut.as_deref(),
                                &self.catalog,
                            )
                            .clicked()
                        })
                        .inner;
                    if clicked {
                        self.client.command(combat.board_toggle.command);
                        ui.ctx().request_repaint();
                    }
                    // Spec 075 : pas de bouton d'action du plateau sous « Voir le plateau ».
                    self.board(ui, &game_screen, false);
                }
                None => self.board(ui, &game_screen, true),
            })
            .response
            .rect;
        self.choice_overlay(&ctx, central, &game_screen);
        self.windows(&ctx);
    }
}

impl IterArcanumApp {
    /// Spec 074 : la barre de menu sur toute la largeur, au-dessus des panneaux ; spec #10 : avec
    /// la Source de l'écran de la partie, dont un dé cliqué rend sa commande comme le Menu.
    fn menu_bar_panel(&mut self, ui: &mut egui::Ui, source: Option<&SourceLine>) {
        if let Some(bar) = self.client.menu_bar(&self.catalog) {
            let clicked = egui::Panel::top("menu_bar")
                .resizable(false)
                .show(ui, |ui| self.menu_bar(ui, &bar, source))
                .inner;
            if let Some(command) = clicked {
                self.client.command(command);
                ui.ctx().request_repaint();
            }
        }
    }

    /// L'écran de nouvelle partie dans la place qui reste (spec 078) ; sans lui, un fond uni.
    fn new_game_central(&mut self, ui: &mut egui::Ui) {
        let page = self.client.new_game_page(&self.catalog);
        let clicked = egui::CentralPanel::default()
            .frame(Frame::NONE.fill(BOARD_BACKGROUND))
            .show(ui, |ui| {
                page.and_then(|(screen, enabled)| self.new_game_page(ui, &screen, enabled))
            })
            .inner;
        if let Some(command) = clicked {
            self.client.command(command);
            ui.ctx().request_repaint();
        }
    }
}
