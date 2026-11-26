//! Ressource de version de `iter-arcanum.exe` (spec #70, C1) : les champs demandés à `winresource`.
//! Inclus par `build.rs` (sous Windows seulement) et par le test `tests/ressource_de_version.rs`,
//! pour que la construction des champs se teste sur toutes les plateformes.

/// Les champs de la ressource de version, tirés de la version du paquet.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VersionResource {
    /// Champs texte (`ProductName`, `FileVersion`…), dans l'ordre où ils sont posés.
    pub strings: Vec<(&'static str, String)>,
    /// Version numérique `X.Y.Z.0` des champs binaires `FILEVERSION` et `PRODUCTVERSION`.
    pub numeric_version: u64,
}

/// Les champs pour la version de paquet `package_version` (`0.2.0`, `0.2.0-dev`…).
pub fn version_resource(package_version: &str) -> VersionResource {
    let file = file_version(package_version);
    let strings = vec![
        ("ProductName", PRODUCT_NAME.to_owned()),
        ("FileDescription", PRODUCT_NAME.to_owned()),
        ("OriginalFilename", "iter-arcanum.exe".to_owned()),
        ("ProductVersion", package_version.to_owned()),
        ("FileVersion", file),
        ("LegalCopyright", "GPL-3.0-or-later".to_owned()),
    ];
    let [major, minor, patch] = numbers(package_version);
    VersionResource {
        strings,
        numeric_version: (major << 48) | (minor << 32) | (patch << 16),
    }
}

/// Nom du produit, celui du projet signé (condition de la SignPath Foundation).
const PRODUCT_NAME: &str = "Iter Arcanum";

/// `X`, `Y` et `Z` de `X.Y.Z[-suffixe]` ; 0 pour un nombre absent ou illisible.
fn numbers(package_version: &str) -> [u64; 3] {
    let core = package_version
        .split(['-', '+'])
        .next()
        .unwrap_or(package_version);
    let mut parts = core.split('.').map(|part| part.parse::<u64>().unwrap_or(0));
    [(); 3].map(|()| parts.next().unwrap_or(0))
}

/// La version de fichier `X.Y.Z.0`, sans suffixe de pré-version (`0.2.0-dev` → `0.2.0.0`).
pub fn file_version(package_version: &str) -> String {
    let [major, minor, patch] = numbers(package_version);
    format!("{major}.{minor}.{patch}.0")
}
