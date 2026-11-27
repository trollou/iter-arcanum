//! Plateau de jeu : cases, figurines, trajet survolé, infobulle, molette et glissement.

use eframe::egui::{
    self, Color32, Event, Key, MouseWheelUnit, PointerButton, Pos2, Rect, Sense, Shape, Stroke,
    Vec2, pos2, vec2,
};

use super::widgets::{card_color, shortcut_button};
use super::window_cards::lone_card;
use super::{CHOICE_STROKE, IterArcanumApp, PROVOKE_STROKE, TILE_STROKE, full_uv};
use crate::{
    BOARD_BACKGROUND, BOARD_TILT, BoardAction, BoardActionKind, BoardCamera, BoardDeparture,
    BoardInput, BoardMovement, BoardScreen, CameraBounds, Command, ExploreGhost, GameScreen,
    HeroWalk, HexDrawing, ImagePaint, MoveMark, Overlay, OverlayKind, RuinPip, RuinTokenMark,
    animation_repaint, animation_time, board_action_image, board_animated, board_screen,
    enemy_color, night_veil,
};
use arcanum_core::{EXPLORE_COST, MapShape};

/// Points de molette qui font un cran, quand le système les donne en points (valeur d'egui).
const POINTS_PER_WHEEL_STEP: f32 = 40.0;
/// Jeton de ruines (spec 045, Q5) : jaune, bord brun ; le dos un peu plus sombre.
const RUIN_TOKEN_FILL: Color32 = Color32::from_rgb(0xF2, 0xC9, 0x2E);
const RUIN_TOKEN_BACK: Color32 = Color32::from_rgb(0xD9, 0xA8, 0x1E);
const RUIN_TOKEN_STROKE: Color32 = Color32::from_rgb(0x5C, 0x40, 0x10);
const HEX_STROKE: Color32 = Color32::from_rgba_premultiplied(0, 0, 0, 70);
const HOVER_STROKE: Color32 = Color32::from_rgb(0xFF, 0xF4, 0xC2);
/// Mise en évidence d'une case où la figurine peut aller (spec 014), couleurs provisoires.
const REACHABLE_FILL: Color32 = Color32::from_rgba_premultiplied(0x4D, 0x4A, 0x3C, 0x4D);
const REACHABLE_STROKE: Color32 = Color32::from_rgb(0xFF, 0xF6, 0xC8);
/// Largeur maximale de l'infobulle de survol : une ligne plus longue passe à la ligne (spec 037, L6).
const HOVER_TOOLTIP_WIDTH: f32 = 320.0;
/// Tracé du trajet survolé (spec 017).
const ROUTE_STROKE: Color32 = Color32::from_rgb(0xE7, 0x4C, 0x3C);
const COST_BADGE_FILL: Color32 = Color32::from_rgb(0xFF, 0xF6, 0xC8);
const COST_BADGE_DIMMED: Color32 = Color32::from_rgb(0x8A, 0x91, 0x8C);
const COST_BADGE_TEXT: Color32 = Color32::from_rgb(0x10, 0x20, 0x2E);
/// Cible d'assaut (spec 043, C37) : pastille rouge « ⚔ coût », trajet et avertissement rouges.
const ASSAULT_FILL: Color32 = Color32::from_rgb(0xB7, 0x1C, 0x1C);
const ASSAULT_TEXT: Color32 = Color32::from_rgb(0xFF, 0xF6, 0xF0);
const ASSAULT_ROUTE: Color32 = Color32::from_rgb(0xC6, 0x10, 0x10);
const ASSAULT_HOVER: Color32 = Color32::from_rgb(0xFF, 0x8A, 0x80);
/// La mer au-delà de la côte, un peu plus sombre que celle des tuiles (spec 075).
const SEA_SHADE: Color32 = Color32::from_rgb(0xB4, 0xBC, 0xCC);
/// Bouton d'action du plateau : fond sombre, ou rouge pour « Engager le combat » (spec 075).
const ACTION_FILL: Color32 = Color32::from_rgb(0x2A, 0x2C, 0x33);
const ACTION_FIGHT_FILL: Color32 = Color32::from_rgb(0x9E, 0x1B, 0x1B);
/// Voile d'un fantôme explorable, plus léger que celui d'une case jouable (spec 015).
const GHOST_FILL: Color32 = Color32::from_rgba_premultiplied(0x29, 0x28, 0x20, 0x29);

impl IterArcanumApp {
    /// Infobulle de survol à côté du curseur (spec 037) : sans délai, même quand la souris bouge,
    /// et pas pendant un glissement du plateau (L1). `hovered` est faux quand une fenêtre recouvre
    /// le plateau (L3).
    fn show_hover_tooltip(
        &mut self,
        response: &egui::Response,
        lines: &[String],
        alert: Option<usize>,
    ) {
        if lines.is_empty() || !response.hovered() || response.dragged() {
            return;
        }
        // Un identifiant par contenu : egui ne remesure une infobulle qu'à son ouverture, et garderait
        // sinon la largeur de la précédente quand le curseur passe d'une case à l'autre.
        egui::Tooltip::always_open(
            response.ctx.clone(),
            response.layer_id,
            response.id.with(lines),
            egui::PopupAnchor::Pointer,
        )
        .width(HOVER_TOOLTIP_WIDTH)
        .show(|ui| {
            for (index, line) in lines.iter().enumerate() {
                // Spec 043, C36 : l'avertissement d'assaut en rouge et en gras.
                if alert == Some(index) {
                    ui.label(egui::RichText::new(line).strong().color(ASSAULT_HOVER));
                } else {
                    // Spec #27, C17 et C19 : l'icône devant le nom d'une capacité.
                    self.marked_label(ui, line, egui::RichText::new);
                }
            }
        });
    }

    pub(super) fn board(
        &mut self,
        ui: &mut egui::Ui,
        game_screen: &GameScreen,
        with_actions: bool,
    ) {
        let Some(game) = self.client.game() else {
            return;
        };
        // Boutons d'action, menu du site et confirmations (spec 075, C24 à C33).
        // Le plateau de jeu de la vue joueur (spec 012), jamais l'état complet.
        let view = game.view();
        let legal_actions = game.legal_actions();
        let shows_moves = game.board_shows_moves();
        let (area, response) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
        let map = &view.map;
        // Toujours présent après la mise en place ; le coin n'est qu'une valeur de repli.
        let shape = view
            .scenario
            .map_or(MapShape::Wedge, |scenario| scenario.shape());
        let camera = self
            .camera
            .get_or_insert_with(|| BoardCamera::initial(map, area));

        // Bornes de la caméra (spec 075, C5 à C7) : elles suivent le plateau et la zone.
        let bounds = CameraBounds::of(map, area);
        if self.recenter_requested || ui.input(|input| input.key_pressed(Key::Home)) {
            camera.recenter(map, area);
            self.recenter_requested = false;
        }
        if response.dragged_by(PointerButton::Primary) || response.dragged_by(PointerButton::Middle)
        {
            camera.drag(response.drag_delta());
            camera.keep_within(&bounds);
        }
        if let Some(cursor) = response.hover_pos() {
            let steps = ui.input(|input| wheel_steps(&input.events));
            if steps != 0.0 {
                camera.zoom_within(area, cursor, steps, &bounds);
            }
        }
        if !ui.ctx().egui_wants_keyboard_input() {
            let (key_steps, direction, seconds) = ui.input(|input| {
                let pressed = |keys: &[Key]| keys.iter().any(|key| input.key_pressed(*key));
                let down = |key: Key| if input.key_down(key) { 1.0 } else { 0.0 };
                let steps = if pressed(&[Key::Plus, Key::Equals]) {
                    1.0
                } else if pressed(&[Key::Minus]) {
                    -1.0
                } else {
                    0.0
                };
                let direction = Vec2::new(
                    down(Key::ArrowRight) - down(Key::ArrowLeft),
                    down(Key::ArrowDown) - down(Key::ArrowUp),
                );
                (steps, direction, input.stable_dt)
            });
            if key_steps != 0.0 {
                camera.zoom_within(area, area.center(), key_steps, &bounds);
            }
            if direction != Vec2::ZERO {
                camera.scroll(direction, seconds);
                camera.keep_within(&bounds);
                // Le défilement continue tant que la flèche reste enfoncée.
                ui.ctx().request_repaint();
            }
        }

        // Essai d'animation (spec 075, Q6) : le temps d'egui, et le début du dernier glissement.
        // Issue #50 : ralenti au besoin, pour capturer les animations sous Xvfb.
        let time = animation_time(ui.input(|input| input.time), self.animation_scale);
        let walk = game.last_walk().map(|(id, path)| {
            let started = match self.walk_start {
                Some((known, started)) if known == id => started,
                _ => {
                    self.walk_start = Some((id, time));
                    time
                }
            };
            HeroWalk { path, started }
        });
        // Spec #5 : l'effacement commence à la première image du plateau après le combat.
        let departure = game.last_departure().map(|(id, enemies)| {
            let started = match self.departure_start {
                Some((known, started)) if known == id => started,
                _ => {
                    self.departure_start = Some((id, time));
                    time
                }
            };
            BoardDeparture { enemies, started }
        });
        let animations = self.client.preferences().animations;
        let site_anchor = game_screen
            .site_menu
            .as_ref()
            .filter(|menu| menu.open)
            .map(|menu| menu.hex)
            .or(game_screen.site_prompt_hex);
        let input = BoardInput {
            map,
            figure: view.figure,
            hero: view.hero,
            shape,
            camera: *camera,
            area,
            cursor: response.hover_pos(),
            inventory: Some(view.inventory),
            // Mode fin du tour : ni case en évidence ni clic (spec 016) ; choix d'une arrivée :
            // seules les arrivées (issue privée 257).
            movement: (shows_moves && game_screen.cast_destinations.is_empty()).then_some(
                BoardMovement {
                    move_rules: &view.move_rules,
                    move_points: view.move_points,
                    next_tile: view.next_tile,
                    legal_actions: &legal_actions,
                    movement_ended: view.movement_ended,
                    enemies_to_fight: &view.enemies_to_fight,
                },
            ),
            enemy_tokens: &view.enemy_tokens,
            shields: &view.shields,
            face_down_defenders: &view.face_down_defenders,
            cities: &view.cities,
            ruins: &view.ruins,
            // Mode fin du tour : ni bouton d'action (C33).
            actions: if with_actions && shows_moves {
                &game_screen.board_actions
            } else {
                &[]
            },
            site_anchor: site_anchor.filter(|_| with_actions),
            time,
            walk,
            departure,
            animations,
            destinations: &game_screen.cast_destinations,
        };
        let screen = board_screen(&input, &self.catalog);
        // L7 : au plus 30 images par seconde, et seulement avec le focus ; rien sans animations
        // (spec #5, C14).
        let animated = board_animated(&input);
        self.paint(ui, area, &screen, night_veil(view.time_of_day));
        // Un clic simple, jamais un glissement, sur une case mise en évidence (spec 014) ou sur
        // un bouton d'action (spec 075, C27).
        let mut clicked = response.clicked().then_some(screen.click).flatten();
        // Spec #20 : les illustrations des cartes montrées au survol des achats.
        let thumbnails = self.thumbnails(
            ui.ctx(),
            game_screen
                .recruit
                .iter()
                .chain(&game_screen.purchases)
                .filter_map(|purchase| purchase.card.as_ref())
                .map(|card| card.illustration),
        );
        if let Some(anchor) = screen.site_anchor
            && let Some(command) =
                site_panel(ui.ctx(), anchor, game_screen, &thumbnails, &self.catalog)
        {
            clicked = Some(command);
        }
        if let Some(command) = clicked {
            self.client.command(command);
            ui.ctx().request_repaint();
        }
        self.show_hover_tooltip(&response, &screen.hover_lines, screen.hover_alert);
        if let Some(delay) = animation_repaint(animated, ui.input(|input| input.focused)) {
            ui.ctx().request_repaint_after(delay);
        }
    }

    fn paint(&mut self, ui: &egui::Ui, area: Rect, screen: &BoardScreen, veil: Option<Color32>) {
        let painter = ui.painter_at(area);
        painter.rect_filled(area, 0.0, BOARD_BACKGROUND);
        let pixels_per_point = ui.ctx().pixels_per_point();

        // Le sol (spec 075) : la mer au-delà de la côte, plus sombre que celle des tuiles, puis la
        // tranche du plateau, puis les cases des tuiles, écrasées par la caméra.
        for hex in &screen.forbidden {
            self.paint_hex(ui, &painter, hex, pixels_per_point, SEA_SHADE);
        }
        for slab in &screen.slabs {
            painter.add(Shape::convex_polygon(
                slab.corners.to_vec(),
                slab.color,
                Stroke::new(1.0, HEX_STROKE),
            ));
        }
        for hex in &screen.hexes {
            self.paint_hex(ui, &painter, hex, pixels_per_point, Color32::WHITE);
            painter.add(Shape::closed_line(
                hex.corners.to_vec(),
                Stroke::new(1.0, HEX_STROKE),
            ));
        }
        for outline in &screen.tile_outlines {
            for segment in &outline.segments {
                painter.line_segment(*segment, Stroke::new(2.5, TILE_STROKE));
            }
        }
        // Voile de nuit (spec 023) : sur le sol, sous les marques du jeu pour qu'elles restent
        // lisibles ; les éléments debout en prennent la teinte (spec 075, L8).
        if let Some(veil) = veil {
            painter.rect_filled(area, 0.0, veil);
        }
        // Marques au sol, écrasées comme lui : fantômes, cases atteignables, trajet, survol.
        for ghost in &screen.explore_ghosts {
            paint_ghost_ground(&painter, ghost);
        }
        for mark in screen.move_marks.iter().filter(|mark| mark.reachable) {
            // Contour épais : un clic y agit, voisine ou lointaine (spec 036, Q2).
            painter.add(Shape::convex_polygon(
                mark.corners.to_vec(),
                REACHABLE_FILL,
                Stroke::new(3.0, REACHABLE_STROKE),
            ));
        }
        // Issue privée 257 : les arrivées d'un déplacement par effet, comme les cases atteignables.
        for mark in &screen.destination_marks {
            painter.add(Shape::convex_polygon(
                mark.corners.to_vec(),
                REACHABLE_FILL,
                Stroke::new(3.0, REACHABLE_STROKE),
            ));
        }
        if let Some(hovered) = &screen.hovered {
            painter.add(Shape::closed_line(
                hovered.corners.to_vec(),
                Stroke::new(3.0, HOVER_STROKE),
            ));
        }
        paint_route(
            &painter,
            &screen.route,
            if screen.route_assault {
                ASSAULT_ROUTE
            } else {
                ROUTE_STROKE
            },
        );
        // Spec 045, C31 : les jetons de ruines, hexagones jaunes posés à plat.
        for mark in &screen.ruin_tokens {
            paint_ruin_token(&painter, mark);
        }
        // Éléments debout, déjà rangés de l'arrière vers l'avant (spec 075, C13).
        let tint = veil.map_or(Color32::WHITE, veil_tint);
        for overlay in &screen.overlays {
            // Socle du héros à sa couleur, d'un ennemi à celle de sa catégorie (spec #14, C9).
            if overlay.kind == OverlayKind::Figure {
                paint_base(&painter, overlay, overlay.fallback, tint);
            } else if let Some(base) = overlay.base {
                paint_base(&painter, overlay, base, tint);
            }
            // Spec #5 : un ennemi vaincu s'affaisse ; son image est demandée à sa taille debout,
            // déjà chargée, puis écrasée vers ses pieds.
            let standing = if overlay.sink < 1.0 {
                let height = overlay.rect.height() / overlay.sink.max(f32::EPSILON);
                Rect::from_min_max(
                    pos2(overlay.rect.left(), overlay.rect.bottom() - height),
                    overlay.rect.max,
                )
            } else {
                overlay.rect
            };
            match self.paint_of(
                ui,
                overlay.image,
                overlay.fallback,
                standing,
                pixels_per_point,
            ) {
                ImagePaint::Texture(texture) => {
                    let size = ui
                        .ctx()
                        .tex_manager()
                        .read()
                        .meta(texture)
                        .map(|meta| meta.size);
                    let rect = standing_image_rect(standing, size);
                    let rect = Rect::from_min_max(
                        pos2(rect.left(), rect.bottom() - rect.height() * overlay.sink),
                        rect.max,
                    );
                    // Le brouillard prend la couleur de sa pile (spec 075, C18).
                    painter.image(texture, rect, full_uv(), tint * overlay.tint);
                }
                // Sans son image, le décor de relief n'est pas remplacé : le sol nu suffit.
                // Pendant le chargement, rien non plus : pas de disque blanc (issue privée 201).
                ImagePaint::Pending(_) => {}
                ImagePaint::Fallback(_) if overlay.kind == OverlayKind::Relief => {}
                ImagePaint::Fallback(color) => {
                    // Un disque posé sur le pied de l'élément (spec 075, L1).
                    let radius = overlay.rect.width().min(overlay.rect.height()) / 2.0;
                    painter.circle(
                        overlay.rect.center_bottom() - vec2(0.0, radius),
                        radius,
                        color.gamma_multiply_u8(tint.r()),
                        Stroke::new(1.5, TILE_STROKE),
                    );
                }
            }
        }
        // Pastilles droites, par-dessus les éléments debout pour rester lisibles (spec 075).
        for ghost in &screen.explore_ghosts {
            paint_ghost_marks(&painter, ghost);
        }
        for mark in &screen.move_marks {
            paint_cost_badge(&painter, mark);
        }
        for alert in &screen.enemy_alerts {
            painter.circle(
                alert.center,
                alert.radius,
                PROVOKE_STROKE,
                Stroke::new(1.5, TILE_STROKE),
            );
            painter.text(
                alert.center,
                egui::Align2::CENTER_CENTER,
                "!",
                egui::FontId::proportional(alert.radius * 1.6),
                COST_BADGE_TEXT,
            );
        }
        // Boutons d'action, par-dessus tout, voile de nuit compris (spec 075, L8).
        for action in &screen.actions {
            self.paint_board_action(ui, &painter, action, pixels_per_point);
        }
    }

    /// Un bouton d'action : disque sombre (rouge pour « Engager le combat »), bord doré, icône ;
    /// pâli quand il est grisé (spec 075).
    fn paint_board_action(
        &mut self,
        ui: &egui::Ui,
        painter: &egui::Painter,
        action: &BoardAction,
        pixels_per_point: f32,
    ) {
        let enabled = action.button.command.is_some();
        let fill = match action.button.kind {
            BoardActionKind::StartCombat => ACTION_FIGHT_FILL,
            _ => ACTION_FILL,
        };
        let (fill, stroke) = if enabled {
            (fill, CHOICE_STROKE)
        } else {
            (fill.gamma_multiply(0.6), COST_BADGE_DIMMED)
        };
        painter.circle(action.center, action.radius, fill, Stroke::new(2.0, stroke));
        let icon = Rect::from_center_size(action.center, Vec2::splat(action.radius * 1.35));
        match self.paint_of(
            ui,
            board_action_image(action.button.kind),
            fill,
            icon,
            pixels_per_point,
        ) {
            ImagePaint::Texture(texture) => {
                let tint = if enabled {
                    Color32::WHITE
                } else {
                    Color32::from_gray(140)
                };
                painter.image(texture, icon, full_uv(), tint);
            }
            // Sans son icône, le nom court du bouton.
            ImagePaint::Fallback(_) | ImagePaint::Pending(_) => {
                painter.text(
                    action.center,
                    egui::Align2::CENTER_CENTER,
                    &action.button.label,
                    egui::FontId::proportional(10.0),
                    COST_BADGE_FILL,
                );
            }
        }
    }

    /// Image de la case, ou polygone de sa couleur de repli.
    fn paint_hex(
        &mut self,
        ui: &egui::Ui,
        painter: &egui::Painter,
        hex: &HexDrawing,
        pixels_per_point: f32,
        tint: Color32,
    ) {
        match self.paint_of(ui, hex.image, hex.fallback, hex.rect, pixels_per_point) {
            ImagePaint::Texture(texture) => {
                painter.image(texture, hex.rect, full_uv(), tint);
            }
            ImagePaint::Fallback(color) | ImagePaint::Pending(color) => {
                painter.add(Shape::convex_polygon(
                    hex.corners.to_vec(),
                    color,
                    Stroke::NONE,
                ));
            }
        }
    }
}

/// Le menu du site pendant une interaction, puis les confirmations de l'incendie et de l'entrée,
/// dans un cadre attaché à la case (spec 075, C30, C31). Renvoie le bouton cliqué.
fn site_panel(
    ctx: &egui::Context,
    anchor: Pos2,
    screen: &GameScreen,
    thumbnails: &super::Thumbnails,
    catalog: &arcanum_i18n::Catalog,
) -> Option<Command> {
    let mut clicked = None;
    egui::Area::new(egui::Id::unique("site_panel"))
        .order(egui::Order::Foreground)
        .fixed_pos(anchor)
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style())
                .stroke(Stroke::new(1.0, CHOICE_STROKE))
                .show(ui, |ui| {
                    ui.set_max_width(SITE_PANEL_WIDTH);
                    if let Some(menu) = screen.site_menu.as_ref().filter(|menu| menu.open) {
                        ui.strong(&menu.title);
                        // Issue privée 341 : les recrutements trop chers restent visibles, grisés.
                        for purchase in screen.recruit.iter().chain(&screen.purchases) {
                            let response = ui.add_enabled(
                                purchase.command.is_some(),
                                egui::Button::new(&purchase.label).wrap(),
                            );
                            // Spec #20 (C21 à C23, L4) : la carte hors zone de l'achat au survol,
                            // achat grisé compris ; le texte pour un achat sans carte.
                            let response = match (&purchase.card, &purchase.tooltip) {
                                (Some(card), _) => {
                                    let show = |ui: &mut egui::Ui| {
                                        lone_card(ui, card, thumbnails, catalog);
                                    };
                                    response.on_hover_ui(show).on_disabled_hover_ui(show)
                                }
                                // Spec #27, C21 : l'infobulle d'un bouton, sans repère d'icône.
                                (None, Some(tooltip)) => {
                                    let tooltip = crate::plain_text(tooltip, catalog);
                                    response
                                        .on_hover_text(&tooltip)
                                        .on_disabled_hover_text(&tooltip)
                                }
                                (None, None) => response,
                            };
                            if response.clicked() {
                                clicked = purchase.command;
                            }
                        }
                    }
                    for prompt in [&screen.burn_confirm, &screen.adventure_confirm]
                        .into_iter()
                        .flatten()
                    {
                        ui.add(egui::Label::new(&prompt.text).wrap());
                        ui.horizontal_wrapped(|ui| {
                            for button in &prompt.buttons {
                                if shortcut_button(
                                    ui,
                                    &button.label,
                                    button.shortcut.as_deref(),
                                    catalog,
                                )
                                .clicked()
                                {
                                    clicked = Some(button.command);
                                }
                            }
                        });
                    }
                });
        });
    clicked
}

/// Largeur du menu du site, en points.
const SITE_PANEL_WIDTH: f32 = 240.0;

/// Teinte des éléments debout la nuit : le voile, appliqué à l'image plutôt que posé dessus, pour
/// que les marques du jeu dessinées après lui restent claires (spec 075, L8).
fn veil_tint(veil: Color32) -> Color32 {
    let keep = 255 - u16::from(veil.a());
    let channel = |premultiplied: u8| (keep + u16::from(premultiplied)).min(255) as u8;
    Color32::from_rgb(channel(veil.r()), channel(veil.g()), channel(veil.b()))
}

/// Socle d'une figurine, un disque cerné à plat sous ses pieds : le héros à sa couleur (spec 075,
/// Héros), un ennemi à celle de sa catégorie (spec #14, Q2).
fn paint_base(painter: &egui::Painter, overlay: &Overlay, color: Color32, tint: Color32) {
    paint_figure_base(painter, overlay.foot, overlay.rect.width(), color * tint);
}

/// Le disque d'un socle, aplati par l'inclinaison du plateau, pour une figurine de cette largeur
/// posée sur ce pied ; repris sous la figurine des cartes d'ennemis (spec #14, C13).
pub(super) fn paint_figure_base(painter: &egui::Painter, foot: Pos2, width: f32, fill: Color32) {
    let radius = vec2(0.42, 0.42 * BOARD_TILT * 0.45) * width;
    let center = foot - vec2(0.0, radius.y * 0.4);
    painter.add(egui::epaint::EllipseShape {
        center,
        radius,
        fill,
        // Le trait s'efface avec le socle d'un ennemi vaincu (spec #5) ; opaque sinon.
        stroke: Stroke::new(1.5, TILE_STROKE.gamma_multiply(f32::from(fill.a()) / 255.0)),
        angle: 0.0,
    });
}

/// Rectangle d'une image debout : la plus grande qui tient dans le rectangle de l'élément, à ses
/// proportions, le bas posé sur le pied (spec 075). Un jeton rond reste rond.
pub(super) fn standing_image_rect(rect: Rect, size: Option<[usize; 2]>) -> Rect {
    let Some([width, height]) = size.filter(|[width, height]| *width > 0 && *height > 0) else {
        return rect;
    };
    let ratio = width as f32 / height as f32;
    let fitted = if rect.width() / rect.height() > ratio {
        vec2(rect.height() * ratio, rect.height())
    } else {
        vec2(rect.width(), rect.width() / ratio)
    };
    Rect::from_min_size(
        rect.center_bottom() - vec2(fitted.x / 2.0, fitted.y),
        fitted,
    )
}

/// Fantôme d'un emplacement explorable, au sol : voile et contour en pointillé (spec 015).
fn paint_ghost_ground(painter: &egui::Painter, ghost: &ExploreGhost) {
    let color = if ghost.reachable {
        REACHABLE_STROKE
    } else {
        COST_BADGE_DIMMED
    };
    if ghost.reachable {
        for corners in &ghost.hexes {
            painter.add(Shape::convex_polygon(
                corners.to_vec(),
                GHOST_FILL,
                Stroke::NONE,
            ));
        }
    }
    let width = if ghost.reachable { 2.5 } else { 1.5 };
    for segment in &ghost.segments {
        painter.extend(Shape::dashed_line(
            segment,
            Stroke::new(width, color),
            6.0,
            4.0,
        ));
    }
}

/// Point d'interrogation et pastille de coût d'un fantôme, droits.
fn paint_ghost_marks(painter: &egui::Painter, ghost: &ExploreGhost) {
    let color = if ghost.reachable {
        REACHABLE_STROKE
    } else {
        COST_BADGE_DIMMED
    };
    let radius = ghost
        .hexes
        .first()
        .map_or(0.0, |corners| (corners[0] - ghost.center).length());
    painter.text(
        ghost.center,
        egui::Align2::CENTER_CENTER,
        "?",
        egui::FontId::proportional(radius * 1.1),
        color,
    );
    if let Some((center, badge_radius)) = ghost.badge {
        let fill = if ghost.reachable {
            COST_BADGE_FILL
        } else {
            COST_BADGE_DIMMED
        };
        painter.circle(center, badge_radius, fill, Stroke::new(1.5, TILE_STROKE));
        painter.text(
            center,
            egui::Align2::CENTER_CENTER,
            EXPLORE_COST.to_string(),
            egui::FontId::proportional(badge_radius * 1.3),
            COST_BADGE_TEXT,
        );
    }
}

/// Pastille de coût d'une case accessible (spec 014), rouge « ⚔ coût » pour un assaut (spec 043),
/// cerclée d'orange si le pas provoque (spec 030).
fn paint_cost_badge(painter: &egui::Painter, mark: &MoveMark) {
    let (fill, text, label) = match (mark.assault, mark.reachable) {
        (true, true) => (ASSAULT_FILL, ASSAULT_TEXT, format!("⚔{}", mark.cost)),
        (_, true) => (COST_BADGE_FILL, COST_BADGE_TEXT, mark.cost.to_string()),
        (true, false) => (COST_BADGE_DIMMED, ASSAULT_FILL, format!("⚔{}", mark.cost)),
        (false, false) => (COST_BADGE_DIMMED, COST_BADGE_TEXT, mark.cost.to_string()),
    };
    // La pastille d'assaut est plus large : « ⚔ » et le coût.
    let radius = if mark.assault {
        mark.badge_radius * 1.15
    } else {
        mark.badge_radius
    };
    painter.circle(
        mark.badge_center,
        radius,
        fill,
        Stroke::new(1.5, TILE_STROKE),
    );
    painter.text(
        mark.badge_center,
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(mark.badge_radius * if mark.assault { 1.05 } else { 1.3 }),
        text,
    );
    if mark.provokes {
        // Anneau orange autour du jeton de coût ; le « ! » va sur l'ennemi (spec 030, C24).
        painter.circle_stroke(
            mark.badge_center,
            mark.badge_radius * 1.35,
            Stroke::new(mark.badge_radius * 0.3, PROVOKE_STROKE),
        );
    }
}

/// Crans de molette de l'image, positifs vers le haut (zoom avant).
fn wheel_steps(events: &[Event]) -> f32 {
    events
        .iter()
        .map(|event| match event {
            Event::MouseWheel { unit, delta, .. } => match unit {
                MouseWheelUnit::Point => delta.y / POINTS_PER_WHEEL_STEP,
                MouseWheelUnit::Line | MouseWheelUnit::Page => delta.y,
            },
            _ => 0.0,
        })
        .sum()
}

/// Tracé du trajet survolé (spec 017) : pointillés de centre en centre, flèche sur la case visée.
fn paint_route(painter: &egui::Painter, route: &[Pos2], color: Color32) {
    let [.., before, last] = route else {
        return;
    };
    let width = 4.0;
    painter.extend(Shape::dashed_line(
        route,
        Stroke::new(width, color),
        10.0,
        6.0,
    ));
    let direction = (*last - *before).normalized();
    let normal = vec2(-direction.y, direction.x);
    let size = 4.0 * width;
    painter.add(Shape::convex_polygon(
        vec![
            *last,
            *last - direction * size + normal * size / 2.0,
            *last - direction * size - normal * size / 2.0,
        ],
        color,
        Stroke::NONE,
    ));
}

/// Un jeton de ruines : hexagone jaune, pointe en haut ; face visible, ses pastilles en ligne (deux)
/// ou en triangle (trois) (spec 045, C31).
fn paint_ruin_token(painter: &egui::Painter, mark: &RuinTokenMark) {
    let corners: Vec<Pos2> = (0..6)
        .map(|k| {
            let angle = std::f32::consts::FRAC_PI_3 * k as f32 - std::f32::consts::FRAC_PI_2;
            // Posé à plat : écrasé comme le sol (spec 075).
            mark.center + vec2(angle.cos(), angle.sin() * BOARD_TILT) * mark.radius
        })
        .collect();
    let fill = if mark.face_up {
        RUIN_TOKEN_FILL
    } else {
        RUIN_TOKEN_BACK
    };
    painter.add(Shape::convex_polygon(
        corners,
        fill,
        Stroke::new((mark.radius * 0.1).max(1.0), RUIN_TOKEN_STROKE),
    ));
    let pip = mark.radius * 0.27;
    let spacing = mark.radius * 0.58;
    let places: Vec<Vec2> = match mark.pips.len() {
        0 => vec![],
        1 => vec![Vec2::ZERO],
        2 => vec![vec2(-0.5, 0.0), vec2(0.5, 0.0)],
        _ => vec![vec2(0.0, -0.5), vec2(-0.5, 0.4), vec2(0.5, 0.4)],
    };
    for (place, color) in places.iter().zip(&mark.pips) {
        let fill = match *color {
            RuinPip::Mana(color) => card_color(color),
            RuinPip::Enemy(color) => enemy_color(color),
        };
        painter.circle(
            mark.center + vec2(place.x, place.y * BOARD_TILT) * spacing,
            pip,
            fill,
            Stroke::new(1.0, RUIN_TOKEN_STROKE),
        );
    }
}
