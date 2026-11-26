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
    let _ = package_version;
    VersionResource::default()
}

/// La version de fichier `X.Y.Z.0`, sans suffixe de pré-version (`0.2.0-dev` → `0.2.0.0`).
pub fn file_version(package_version: &str) -> String {
    let _ = package_version;
    String::new()
}
