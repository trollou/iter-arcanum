//! Embarque l'icône dans `iter-arcanum.exe` quand on compile pour Windows (spec 004).

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    // La cible, pas la machine qui compile : `cfg!(windows)` décrirait celle-ci.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        windows::embed_icon();
    }
}

#[cfg(windows)]
mod windows {
    use std::path::PathBuf;

    pub fn embed_icon() {
        let ico = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("assets")
            .join("images")
            .join("icon")
            .join("iter-arcanum.ico");
        println!("cargo:rerun-if-changed={}", ico.display());
        let mut resource = winresource::WindowsResource::new();
        resource.set_icon(&ico.to_string_lossy());
        // Spec 004, L1 : sans compilateur de ressources, l'exe se compile sans icône de fichier.
        if let Err(error) = resource.compile() {
            println!(
                "cargo:warning=icône non embarquée dans iter-arcanum.exe (rc.exe ou windres introuvable ?) : {error}"
            );
        }
    }
}

// Compilation croisée vers Windows depuis Linux : hors périmètre, l'exe n'a pas d'icône de fichier.
#[cfg(not(windows))]
mod windows {
    pub fn embed_icon() {
        println!(
            "cargo:warning=icône non embarquée : compilation croisée vers Windows non prise en charge"
        );
    }
}
