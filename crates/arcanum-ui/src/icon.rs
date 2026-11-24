//! Icône du jeu (spec 004) : icône de fenêtre, et chemin de l'`.ico` embarqué dans l'exe Windows par `build.rs`.

use std::path::PathBuf;

use eframe::egui::{IconData, SizeHint, ViewportBuilder};
use thiserror::Error;

/// Identifiant d'application : relie la fenêtre à un futur fichier `.desktop` sous Wayland.
pub const APP_ID: &str = "iter-arcanum";

/// Côté de l'icône de fenêtre, en pixels.
pub const WINDOW_ICON_SIZE: u32 = 256;

/// Tailles contenues dans `iter-arcanum.ico`, celles qu'utilise Windows selon l'affichage et l'échelle.
pub const ICO_SIZES: [u32; 5] = [16, 24, 32, 48, 256];

/// Source unique de l'icône, compilée dans le binaire : elle sert avant l'ouverture de la fenêtre,
/// même sans dossier des ressources.
pub const ICON_SVG: &[u8] = include_bytes!("../../../assets/images/icon/iter-arcanum.svg");

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum IconError {
    #[error("rendu SVG de l'icône impossible : {0}")]
    Svg(String),
}

/// `iter-arcanum.ico` du dépôt, dérivé du SVG par `cargo run -p arcanum-ui --example icone`.
pub fn icon_ico_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("assets")
        .join("images")
        .join("icon")
        .join("iter-arcanum.ico")
}

/// Rendu carré `size` × `size` d'un SVG, en alpha non prémultiplié comme l'attend la fenêtre.
pub fn render_icon(svg: &[u8], size: u32) -> Result<IconData, IconError> {
    let hint = SizeHint::Size {
        width: size,
        height: size,
        maintain_aspect_ratio: true,
    };
    let image = egui_extras::image::load_svg_bytes_with_size(svg, hint, &Default::default())
        .map_err(IconError::Svg)?;
    let rgba = image
        .pixels
        .iter()
        .flat_map(|pixel| pixel.to_srgba_unmultiplied())
        .collect();
    Ok(IconData {
        rgba,
        width: image.size[0] as u32,
        height: image.size[1] as u32,
    })
}

/// Icône de fenêtre, rendue depuis le SVG embarqué.
pub fn window_icon() -> Result<IconData, IconError> {
    render_icon(ICON_SVG, WINDOW_ICON_SIZE)
}

/// Réglages de la fenêtre principale (spec 002), avec l'icône quand elle a pu être préparée.
/// Plein écran dès l'ouverture si les préférences le demandent (spec 080, C12).
pub fn viewport(
    title: &str,
    icon: Result<IconData, IconError>,
    fullscreen_at_launch: bool,
) -> ViewportBuilder {
    let mut builder = ViewportBuilder::default()
        .with_title(title)
        .with_inner_size([1280.0, 800.0])
        .with_app_id(APP_ID);
    if fullscreen_at_launch {
        builder = builder.with_fullscreen(true);
    }
    match icon {
        Ok(icon) => builder.with_icon(icon),
        Err(error) => {
            // La fenêtre s'ouvre quand même, avec l'icône par défaut du système.
            log::error!("icône de fenêtre indisponible : {error}");
            builder
        }
    }
}
