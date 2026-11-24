//! Produit `assets/images/icon/iter-arcanum.ico` à partir du SVG embarqué (spec 004).
//! À relancer quand le SVG change : `cargo run -p arcanum-ui --example icone`.

use std::fs::File;
use std::process::ExitCode;

use arcanum_ui::{ICO_SIZES, ICON_SVG, icon_ico_path, render_icon};
use ico::{IconDir, IconDirEntry, IconImage, ResourceType};

fn main() -> ExitCode {
    let mut dir = IconDir::new(ResourceType::Icon);
    for size in ICO_SIZES {
        let icon = match render_icon(ICON_SVG, size) {
            Ok(icon) => icon,
            Err(error) => {
                eprintln!("{error}");
                return ExitCode::FAILURE;
            }
        };
        let image = IconImage::from_rgba_data(icon.width, icon.height, icon.rgba);
        match IconDirEntry::encode(&image) {
            Ok(entry) => dir.add_entry(entry),
            Err(error) => {
                eprintln!("encodage {size} px impossible : {error}");
                return ExitCode::FAILURE;
            }
        }
    }
    let path = icon_ico_path();
    let written = File::create(&path).and_then(|file| dir.write(file));
    match written {
        Ok(()) => {
            println!("écrit : {}", path.display());
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("écriture de {} impossible : {error}", path.display());
            ExitCode::FAILURE
        }
    }
}
