//! Spec 011, C22 : une image manquante est remplacée par sa couleur de repli et signalée une
//! seule fois dans les logs. Fichier à part : il installe un logger global de capture.

mod support;

use arcanum_core::{BasicAction, DeedCard, Terrain};
use arcanum_ui::{
    ImageLoad, ImagePaint, ImageWarnings, card_illustration, illustration_paint, image_paint,
    repository_assets_dir, terrain_color, terrain_image,
};
use eframe::egui::{Color32, TextureId};
use log::Level;
use support::logs::{captured, logged};

// Spec 011, C22.
#[test]
fn missing_image_falls_back_and_warns_once() {
    captured();
    let image = terrain_image(Terrain::Forest);
    let path = image.path(&repository_assets_dir().join("images").join("absent"));
    let fallback = terrain_color(Terrain::Forest);
    let mut warnings = ImageWarnings::default();

    for _ in 0..5 {
        let paint = image_paint(
            ImageLoad::Failed("introuvable".into()),
            image,
            fallback,
            &path,
            &mut warnings,
        );
        assert_eq!(paint, ImagePaint::Fallback(fallback));
    }
    let logged = logged(Level::Warn);
    assert_eq!(logged.len(), 1, "{logged:?}");
    assert!(
        logged[0].contains(&path.display().to_string()),
        "{logged:?}"
    );
}

// Spec 011, C22 : en cours de chargement, le repli s'affiche sans rien signaler ;
// une fois chargée, l'image est utilisée.
#[test]
fn pending_image_falls_back_silently_then_shows() {
    captured();
    let image = terrain_image(Terrain::Lake);
    let path = image.path(&repository_assets_dir().join("images"));
    let fallback = terrain_color(Terrain::Lake);
    let mut warnings = ImageWarnings::default();

    let pending = image_paint(ImageLoad::Pending, image, fallback, &path, &mut warnings);
    // Issue privée 201 : en cours de chargement, ce n'est pas un échec.
    assert_eq!(pending, ImagePaint::Pending(fallback));
    let texture = TextureId::Managed(7);
    let ready = image_paint(
        ImageLoad::Ready(texture),
        image,
        fallback,
        &path,
        &mut warnings,
    );
    assert_eq!(ready, ImagePaint::Texture(texture));
    assert_eq!(logged(Level::Warn), Vec::<String>::new());
    assert_ne!(fallback, Color32::TRANSPARENT);
}

// Spec 072, L1 : un PNG présent mais illisible laisse l'écran sans image et n'est signalé qu'une
// fois ; en cours de chargement, rien n'est signalé ; chargé, il est dessiné.
#[test]
fn unreadable_illustration_falls_back_and_warns_once() {
    captured();
    let illustration = card_illustration(DeedCard::Basic(BasicAction::Rage));
    let path = illustration.path(&repository_assets_dir().join("images").join("absent"));
    let mut warnings = ImageWarnings::default();

    assert_eq!(
        illustration_paint(ImageLoad::Pending, &path, &mut warnings),
        None
    );
    assert!(logged(Level::Warn).is_empty());
    for _ in 0..5 {
        let paint = illustration_paint(ImageLoad::Failed("illisible".into()), &path, &mut warnings);
        assert_eq!(paint, None);
    }
    let logged = logged(Level::Warn);
    assert_eq!(logged.len(), 1, "{logged:?}");
    assert!(
        logged[0].contains(&path.display().to_string()),
        "{logged:?}"
    );
    let texture = TextureId::Managed(9);
    assert_eq!(
        illustration_paint(ImageLoad::Ready(texture), &path, &mut warnings),
        Some(texture)
    );
}
