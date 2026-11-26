//! Produit `assets/images/icon/iter-arcanum.ico` à partir du SVG embarqué (spec 004).
//! À relancer quand le SVG change : `cargo run -p arcanum-ui --example icone`.
//!
//! Avec `--iconset <dossier>`, écrit à la place les PNG d'un `.iconset` macOS, que
//! `iconutil -c icns` convertit en `.icns` (spec #33 : produit par le workflow de release, jamais commité).

use std::fs::File;
use std::path::Path;
use std::process::ExitCode;

use arcanum_ui::{ICO_SIZES, ICON_SVG, icon_ico_path, render_icon};
use ico::{IconDir, IconDirEntry, IconImage, ResourceType};

/// Côtés de base d'un `.iconset` ; chacun existe aussi en `@2x` (côté double).
const ICONSET_SIZES: [u32; 5] = [16, 32, 128, 256, 512];

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.as_slice() {
        [] => write_ico(),
        [flag, dir] if flag == "--iconset" => write_iconset(Path::new(dir)),
        _ => Err("usage : icone [--iconset <dossier>]".to_owned()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn write_ico() -> Result<(), String> {
    let mut dir = IconDir::new(ResourceType::Icon);
    for size in ICO_SIZES {
        let icon = render_icon(ICON_SVG, size).map_err(|error| error.to_string())?;
        let image = IconImage::from_rgba_data(icon.width, icon.height, icon.rgba);
        let entry = IconDirEntry::encode(&image)
            .map_err(|error| format!("encodage {size} px impossible : {error}"))?;
        dir.add_entry(entry);
    }
    let path = icon_ico_path();
    File::create(&path)
        .and_then(|file| dir.write(file))
        .map_err(|error| format!("écriture de {} impossible : {error}", path.display()))?;
    println!("écrit : {}", path.display());
    Ok(())
}

fn write_iconset(dir: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dir)
        .map_err(|error| format!("création de {} impossible : {error}", dir.display()))?;
    for base in ICONSET_SIZES {
        for (scale, suffix) in [(1, ""), (2, "@2x")] {
            let size = base * scale;
            let icon = render_icon(ICON_SVG, size).map_err(|error| error.to_string())?;
            let path = dir.join(format!("icon_{base}x{base}{suffix}.png"));
            image::save_buffer(
                &path,
                &icon.rgba,
                icon.width,
                icon.height,
                image::ExtendedColorType::Rgba8,
            )
            .map_err(|error| format!("écriture de {} impossible : {error}", path.display()))?;
        }
    }
    println!("écrit : {}", dir.display());
    Ok(())
}
