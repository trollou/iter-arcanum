//! Scénarios de la spec 004 (icône du jeu), sans fenêtre.
//! Aucune règle du livret ici : chaque test cite le cas de la spec.

use std::fs::File;

use arcanum_ui::{
    APP_ID, ICO_SIZES, ICON_SVG, IconError, WINDOW_ICON_SIZE, icon_ico_path, render_icon, viewport,
    window_icon,
};
use eframe::egui::{IconData, Vec2};

fn pixel(icon: &IconData, x: u32, y: u32) -> [u8; 4] {
    let start = ((y * icon.width + x) * 4) as usize;
    let mut rgba = [0; 4];
    rgba.copy_from_slice(&icon.rgba[start..start + 4]);
    rgba
}

// Spec 004, C1.
#[test]
fn window_icon_is_256_pixels_square() {
    let icon = window_icon().expect("icône de fenêtre");
    assert_eq!(WINDOW_ICON_SIZE, 256);
    assert_eq!((icon.width, icon.height), (256, 256));
    assert_eq!(icon.rgba.len(), 256 * 256 * 4);
}

// Spec 004, C2.
#[test]
fn window_icon_has_transparent_corners_and_opaque_center() {
    let icon = window_icon().expect("icône de fenêtre");
    for (x, y) in [(0, 0), (255, 0), (0, 255), (255, 255)] {
        assert_eq!(pixel(&icon, x, y)[3], 0, "coin ({x}, {y})");
    }
    assert_eq!(pixel(&icon, 128, 128)[3], 255);
}

// Spec 004, C3 : un pixel opaque garde sa couleur, un pixel transparent vaut (0, 0, 0, 0).
#[test]
fn window_icon_alpha_is_not_premultiplied() {
    // Un carré à moitié transparent : prémultiplié, il vaudrait (128, 0, 0, 128).
    let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 8 8" width="8" height="8">
        <rect x="0" y="0" width="4" height="8" fill="#FF0000" fill-opacity="0.5"/>
    </svg>"##;
    let icon = render_icon(svg, 8).expect("SVG valide");
    let [r, g, b, a] = pixel(&icon, 1, 1);
    assert!((127..=129).contains(&a), "alpha {a}");
    assert!(r >= 254 && g == 0 && b == 0, "couleur ({r}, {g}, {b})");
    assert_eq!(pixel(&icon, 6, 1), [0, 0, 0, 0]);

    let icon = window_icon().expect("icône de fenêtre");
    assert_eq!(pixel(&icon, 0, 0), [0, 0, 0, 0]);
}

// Spec 004, C4.
#[test]
fn viewport_carries_icon_and_app_id() {
    let builder = viewport("Iter Arcanum", window_icon(), false);
    assert_eq!(builder.title.as_deref(), Some("Iter Arcanum"));
    assert_eq!(builder.inner_size, Some(Vec2::new(1280.0, 800.0)));
    assert_eq!(builder.app_id.as_deref(), Some(APP_ID));
    assert_eq!(APP_ID, "iter-arcanum");
    let icon = builder.icon.expect("icône");
    assert_eq!((icon.width, icon.height), (256, 256));
}

// Spec 082, C1.
#[test]
fn ico_file_is_named_iter_arcanum() {
    // Spec 082, C1 : l'icône de l'exe porte le nouvel identifiant.
    assert!(
        icon_ico_path().ends_with("iter-arcanum.ico"),
        "{:?}",
        icon_ico_path()
    );
}

// Spec 004, C5 (la ligne de log est vérifiée dans `icone_logs.rs`).
#[test]
fn viewport_without_icon_when_icon_fails() {
    let failed = render_icon(b"pas un svg", WINDOW_ICON_SIZE);
    assert!(matches!(failed, Err(IconError::Svg(_))), "{failed:?}");
    let builder = viewport("Iter Arcanum", failed, false);
    assert!(builder.icon.is_none());
    assert_eq!(builder.title.as_deref(), Some("Iter Arcanum"));
    assert_eq!(builder.inner_size, Some(Vec2::new(1280.0, 800.0)));
    assert_eq!(builder.app_id.as_deref(), Some(APP_ID));
}

fn read_ico() -> ico::IconDir {
    let file = File::open(icon_ico_path()).expect("iter-arcanum.ico présent");
    ico::IconDir::read(file).expect(".ico lisible")
}

// Spec 004, C6.
#[test]
fn ico_contains_expected_sizes() {
    assert_eq!(ICO_SIZES, [16, 24, 32, 48, 256]);
    let mut sizes: Vec<u32> = read_ico()
        .entries()
        .iter()
        .map(|entry| {
            assert_eq!(entry.width(), entry.height());
            entry.width()
        })
        .collect();
    sizes.sort_unstable();
    assert_eq!(sizes, ICO_SIZES);
}

// Spec 004, C7 : l'`.ico` versionné doit suivre le SVG.
#[test]
fn ico_matches_svg() {
    const TOLERANCE: u8 = 2;
    for entry in read_ico().entries() {
        let size = entry.width();
        let stored = entry.decode().expect("image décodable");
        let expected = render_icon(ICON_SVG, size).expect("SVG embarqué valide");
        let worst = stored
            .rgba_data()
            .iter()
            .zip(&expected.rgba)
            .map(|(a, b)| a.abs_diff(*b))
            .max()
            .unwrap_or(0);
        assert!(
            worst <= TOLERANCE,
            "iter-arcanum.ico ({size} px) ne suit plus iter-arcanum.svg (écart {worst}) : \
             lancer `cargo run -p arcanum-ui --example icone`"
        );
    }
}
