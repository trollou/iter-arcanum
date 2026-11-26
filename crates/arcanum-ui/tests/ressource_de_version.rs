//! Spec #70, C1 : la ressource de version de `iter-arcanum.exe` nomme le produit et sa version.
//! La construction des champs vit dans `build/version_resource.rs`, partagé avec `build.rs`.

#[path = "../build/version_resource.rs"]
mod version_resource;

use version_resource::{file_version, version_resource};

fn field(resource: &version_resource::VersionResource, name: &str) -> Option<String> {
    resource
        .strings
        .iter()
        .find(|(key, _)| *key == name)
        .map(|(_, value)| value.clone())
}

// Spec #70, C1 : métadonnées imposées par la SignPath Foundation (nom du produit, même version).
#[test]
fn version_resource_names_the_product_and_version() {
    let resource = version_resource("0.2.0");
    assert_eq!(
        field(&resource, "ProductName").as_deref(),
        Some("Iter Arcanum")
    );
    assert_eq!(
        field(&resource, "FileDescription").as_deref(),
        Some("Iter Arcanum")
    );
    assert_eq!(
        field(&resource, "OriginalFilename").as_deref(),
        Some("iter-arcanum.exe")
    );
    assert_eq!(field(&resource, "ProductVersion").as_deref(), Some("0.2.0"));
    assert_eq!(field(&resource, "FileVersion").as_deref(), Some("0.2.0.0"));
    assert_eq!(
        field(&resource, "LegalCopyright").as_deref(),
        Some("GPL-3.0-or-later")
    );
    assert_eq!(resource.numeric_version, 2 << 32);
    let later = version_resource("1.10.3");
    assert_eq!(field(&later, "FileVersion").as_deref(), Some("1.10.3.0"));
    assert_eq!(later.numeric_version, (1 << 48) | (10 << 32) | (3 << 16));
}

// Spec #70, C1 : une version à suffixe donne X.Y.Z.0 ; la version du produit garde le suffixe.
#[test]
fn file_version_drops_the_prerelease_suffix() {
    assert_eq!(file_version("0.2.0-dev"), "0.2.0.0");
    assert_eq!(file_version("0.2.0-rc.1"), "0.2.0.0");
    assert_eq!(file_version("1.10.3"), "1.10.3.0");
    let dev = version_resource("0.2.0-dev");
    assert_eq!(field(&dev, "ProductVersion").as_deref(), Some("0.2.0-dev"));
    assert_eq!(dev.numeric_version, 2 << 32);
}
