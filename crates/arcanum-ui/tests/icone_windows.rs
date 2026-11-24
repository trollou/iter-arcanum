//! Spec 004, C8 : sous Windows, `iter-arcanum.exe` contient une ressource icône.
//! Compilé et lancé seulement sous Windows ; la CI Windows le fait tourner.
#![cfg(windows)]

use object::LittleEndian;
use object::pe::{IMAGE_DIRECTORY_ENTRY_RESOURCE, ImageNtHeaders64, RT_GROUP_ICON};
use object::read::pe::{ImageNtHeaders, ResourceDirectory, ResourceNameOrId};

// Spec 004, C8.
#[test]
fn windows_exe_has_icon_resource() {
    let data = std::fs::read(env!("CARGO_BIN_EXE_iter-arcanum")).expect("iter-arcanum.exe lisible");
    let dos = object::pe::ImageDosHeader::parse(&*data).expect("en-tête DOS");
    let mut offset = dos.nt_headers_offset().into();
    let (nt, directories) = ImageNtHeaders64::parse(&*data, &mut offset).expect("en-têtes PE");
    let sections = nt.sections(&*data, offset).expect("sections");
    let resources = directories
        .get(IMAGE_DIRECTORY_ENTRY_RESOURCE)
        .filter(|entry| entry.virtual_address.get(LittleEndian) != 0)
        .expect("iter-arcanum.exe n'a aucune ressource : l'icône n'a pas été embarquée (build.rs)");
    let bytes = resources.data(&*data, &sections).expect("ressources");
    let root = ResourceDirectory::new(bytes).root().expect("racine");
    let has_group_icon = root
        .entries
        .iter()
        .any(|entry| matches!(entry.name_or_id(), ResourceNameOrId::Id(RT_GROUP_ICON)));
    assert!(
        has_group_icon,
        "pas de groupe d'icônes dans iter-arcanum.exe"
    );
}
