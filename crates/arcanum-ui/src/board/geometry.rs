//! Géométrie des cases à pointe en haut, en unités du plateau : rayon 1 (spec 011).

use arcanum_core::Hex;
use eframe::egui::{Pos2, pos2};

const SQRT_3: f32 = 1.732_050_8;

/// Centre de la case : `x = √3 × (q + r/2)`, `y = 1,5 × r`, `y` vers le bas comme l'écran.
pub fn hex_center(hex: Hex) -> Pos2 {
    let (q, r) = (hex.q as f32, hex.r as f32);
    pos2(SQRT_3 * (q + r / 2.0), 1.5 * r)
}

/// Les six sommets, en partant de la pointe du haut puis dans le sens des aiguilles d'une montre
/// (angles −90°, −30°, 30°, 90°, 150°, 210°).
pub fn hex_corners(hex: Hex) -> [Pos2; 6] {
    let center = hex_center(hex);
    [
        (0.0, -1.0),
        (SQRT_3 / 2.0, -0.5),
        (SQRT_3 / 2.0, 0.5),
        (0.0, 1.0),
        (-SQRT_3 / 2.0, 0.5),
        (-SQRT_3 / 2.0, -0.5),
    ]
    .map(|(dx, dy)| pos2(center.x + dx, center.y + dy))
}

/// La case qui contient un point du plateau (arrondi en coordonnées cubiques).
pub fn point_to_hex(point: Pos2) -> Hex {
    let q = SQRT_3 / 3.0 * point.x - point.y / 3.0;
    let r = 2.0 / 3.0 * point.y;
    let s = -q - r;
    let (mut rq, mut rr, rs) = (q.round(), r.round(), s.round());
    let (dq, dr, ds) = ((rq - q).abs(), (rr - r).abs(), (rs - s).abs());
    // On corrige la coordonnée la plus mal arrondie pour garder q + r + s = 0.
    if dq > dr && dq > ds {
        rq = -rr - rs;
    } else if dr > ds {
        rr = -rq - rs;
    }
    Hex::new(rq as i32, rr as i32)
}
