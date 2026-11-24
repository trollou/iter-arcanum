//! Cadrage : quelle partie du plateau de jeu est visible et à quelle échelle (spec 011).

use arcanum_core::Map;
use eframe::egui::{Pos2, Rect, Vec2, pos2, vec2};

use super::geometry::{hex_center, hex_corners};

/// Bornes du zoom : rayon d'une case à l'écran, en points. Le plancher absolu est celui où les
/// éléments debout restent dessinés (spec 075, C5).
pub const MIN_ZOOM: f32 = 16.0;
pub const MAX_ZOOM: f32 = 120.0;
/// Zoom d'un plateau vide ou d'une zone sans taille.
pub const DEFAULT_ZOOM: f32 = 40.0;
/// Facteur d'un cran de molette ou d'un appui sur `+`.
pub const ZOOM_STEP: f32 = 1.15;
/// Défilement aux flèches, en points par seconde.
pub const ARROW_SCROLL_SPEED: f32 = 800.0;
/// Une image plus longue (retour de veille) ne fait pas défiler davantage (L6).
pub const MAX_SCROLL_SECONDS: f32 = 0.1;
/// Inclinaison de la vue isométrique : la verticale du plateau est écrasée par ce facteur à
/// l'écran (spec 075, Q1).
pub const BOARD_TILT: f32 = 0.80;
/// Le dézoom s'arrête à cette fraction du zoom de cadrage (spec 075, Q2 ; 0,5 au lieu de 0,75,
/// Mathieu, 2026-11-11).
pub const FIT_ZOOM_OUT: f32 = 0.5;
/// Hauteur d'un élément debout au-dessus du centre de sa case, en rayons de case, sans
/// écrasement : le cadrage la laisse en haut (spec 075, C4).
pub const STANDING_HEIGHT: f32 = 1.2;
/// Marge du cadrage initial, de chaque côté.
const FIT_MARGIN: f32 = 0.05;

/// Point du plateau affiché au centre de la zone, et rayon d'une case à l'écran.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoardCamera {
    pub center: Pos2,
    pub zoom: f32,
}

/// Bornes de la caméra pour un plateau et une zone : zoom minimal et rectangle où reste le centre
/// (spec 075, C5 à C7).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CameraBounds {
    pub min_zoom: f32,
    /// Rectangle des tuiles posées, en unités du plateau ; aucun pour un plateau vide (L5).
    pub center: Option<Rect>,
}

impl CameraBounds {
    pub fn of(map: &Map, area: Rect) -> CameraBounds {
        let corners: Vec<Pos2> = map
            .tiles()
            .flat_map(|(position, _)| position.hexes())
            .flat_map(hex_corners)
            .collect();
        if corners.is_empty() {
            return CameraBounds {
                min_zoom: MIN_ZOOM,
                center: None,
            };
        }
        let fitted = BoardCamera::initial(map, area).zoom;
        CameraBounds {
            min_zoom: (FIT_ZOOM_OUT * fitted).max(MIN_ZOOM),
            center: Some(Rect::from_points(&corners)),
        }
    }
}

impl BoardCamera {
    /// Tout le plateau de jeu visible, avec la marge, au plus grand zoom permis.
    pub fn initial(map: &Map, area: Rect) -> BoardCamera {
        let default = BoardCamera {
            center: Pos2::ZERO,
            zoom: DEFAULT_ZOOM,
        };
        let hexes: Vec<_> = map
            .tiles()
            .flat_map(|(position, _)| position.hexes())
            .collect();
        if hexes.is_empty() || area.width() <= 0.0 || area.height() <= 0.0 {
            return default;
        }
        // Plateau projeté (spec 075, C4) : en unités du plateau à l'horizontale, écrasées à la
        // verticale. Le haut d'un élément debout ne grandit pas avec le zoom : il garde
        // STANDING_HEIGHT rayons, qu'on ajoute au-dessus du centre de chaque case.
        let projected = |point: Pos2| pos2(point.x, point.y * BOARD_TILT);
        let ground = Rect::from_points(
            &hexes
                .iter()
                .flat_map(|hex| hex_corners(*hex))
                .map(projected)
                .collect::<Vec<_>>(),
        );
        let standing = hexes
            .iter()
            .map(|hex| projected(hex_center(*hex)).y - STANDING_HEIGHT)
            .fold(f32::INFINITY, f32::min);
        let bounds =
            Rect::from_min_max(pos2(ground.left(), ground.top().min(standing)), ground.max);
        let available = area.size() * (1.0 - 2.0 * FIT_MARGIN);
        let zoom = (available.x / bounds.width()).min(available.y / bounds.height());
        let center = bounds.center();
        BoardCamera {
            center: pos2(center.x, center.y / BOARD_TILT),
            zoom: zoom.clamp(MIN_ZOOM, MAX_ZOOM),
        }
    }

    pub fn recenter(&mut self, map: &Map, area: Rect) {
        *self = BoardCamera::initial(map, area);
    }

    /// Vue isométrique : la verticale est écrasée par l'inclinaison (spec 075).
    pub fn to_screen(&self, area: Rect, point: Pos2) -> Pos2 {
        area.center() + self.board_to_screen_vec(point - self.center)
    }

    pub fn to_board(&self, area: Rect, point: Pos2) -> Pos2 {
        self.center + self.screen_to_board_vec(point - area.center())
    }

    fn board_to_screen_vec(&self, delta: Vec2) -> Vec2 {
        vec2(delta.x, delta.y * BOARD_TILT) * self.zoom
    }

    fn screen_to_board_vec(&self, delta: Vec2) -> Vec2 {
        vec2(delta.x, delta.y / BOARD_TILT) / self.zoom
    }

    /// Zoome de `steps` crans (négatif pour dézoomer) en gardant fixe le point sous `anchor`.
    pub fn zoom_at(&mut self, area: Rect, anchor: Pos2, steps: f32) {
        self.zoom_between(area, anchor, steps, MIN_ZOOM);
    }

    /// Comme [`BoardCamera::zoom_at`], sans dézoomer sous la borne du plateau. Un zoom déjà plus
    /// petit (zone agrandie) n'est pas relevé d'office.
    pub fn zoom_within(&mut self, area: Rect, anchor: Pos2, steps: f32, bounds: &CameraBounds) {
        self.zoom_between(area, anchor, steps, bounds.min_zoom.min(self.zoom));
        self.keep_within(bounds);
    }

    fn zoom_between(&mut self, area: Rect, anchor: Pos2, steps: f32, min_zoom: f32) {
        let factor = ZOOM_STEP.powf(steps);
        if factor.is_nan() {
            return;
        }
        let fixed = self.to_board(area, anchor);
        self.zoom = (self.zoom * factor).clamp(min_zoom.max(MIN_ZOOM), MAX_ZOOM);
        self.center = fixed - self.screen_to_board_vec(anchor - area.center());
    }

    /// Ramène le centre au-dessus du rectangle des tuiles posées (spec 075, C7).
    pub fn keep_within(&mut self, bounds: &CameraBounds) {
        if let Some(rect) = bounds.center {
            self.center = self.center.clamp(rect.min, rect.max);
        }
    }

    /// Le plateau suit le curseur.
    pub fn drag(&mut self, delta: Vec2) {
        self.center -= self.screen_to_board_vec(delta);
    }

    /// Défilement aux flèches : `direction` vaut −1, 0 ou 1 sur chaque axe (droite et bas positifs).
    pub fn scroll(&mut self, direction: Vec2, seconds: f32) {
        let seconds = seconds.clamp(0.0, MAX_SCROLL_SECONDS);
        self.drag(-direction * ARROW_SCROLL_SPEED * seconds);
    }
}

impl Default for BoardCamera {
    fn default() -> Self {
        BoardCamera {
            center: pos2(0.0, 0.0),
            zoom: DEFAULT_ZOOM,
        }
    }
}
