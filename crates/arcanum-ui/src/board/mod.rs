//! Affichage du plateau de jeu (spec 011) : tout ce qui se calcule sans egui.
//! L'application ne fait que lire les entrées, appeler ces fonctions et dessiner le résultat.

mod camera;
mod geometry;
mod images;
mod screen;

pub use camera::{
    ARROW_SCROLL_SPEED, BOARD_TILT, BoardCamera, CameraBounds, DEFAULT_ZOOM, FIT_ZOOM_OUT,
    MAX_SCROLL_SECONDS, MAX_ZOOM, MIN_ZOOM, STANDING_HEIGHT, ZOOM_STEP,
};
pub use geometry::{hex_center, hex_corners, point_to_hex};
pub use images::{
    BOARD_ACTION_KINDS, BOARD_BACKGROUND, FOG_IMAGE, FORBIDDEN_COLOR, FORBIDDEN_IMAGE, ImageId,
    ImageLoad, ImagePaint, ImageWarnings, board_action_image, board_images, enemy_color,
    feature_color, feature_image, figure_image, flag_image, hero_color, hero_figure_image,
    hero_image, illustration_paint, image_paint, night_veil, relief_image, svg_size, terrain_color,
    terrain_image, time_of_day_image,
};
pub use screen::{
    BOARD_ACTION_DIAMETER, BREATHING_AMPLITUDE, BREATHING_PERIOD, BoardAction, BoardInput,
    BoardMovement, BoardScreen, EnemyAlert, ExploreGhost, HeroWalk, HexDrawing, Hovered, MoveMark,
    OVERLAY_MIN_ZOOM, Overlay, OverlayKind, RuinPip, RuinTokenMark, SLAB_DEPTH, Slab, TileOutline,
    WALK_STEP_SECONDS, animation_repaint, board_screen,
};
pub(crate) use screen::{feature_key, terrain_key};
