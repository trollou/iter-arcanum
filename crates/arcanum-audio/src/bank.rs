//! Banque de sons (spec 081) : les fichiers trouvés dans `assets/sounds/` au lancement, rangés par
//! musique, ambiance et effet. Un fichier manquant n'est pas une erreur : le son reste muet (C24).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::{Ambience, Cue, Music};

/// Dossier des sons, dans le dossier des ressources.
pub const SOUNDS_DIR: &str = "sounds";

/// Les fichiers de sons trouvés, chacun à sa place.
#[derive(Clone, Debug, Default)]
pub struct SoundBank {
    music: BTreeMap<Music, PathBuf>,
    ambience: BTreeMap<Ambience, PathBuf>,
    /// Les variantes de chaque effet, dans l'ordre de leur numéro.
    cues: BTreeMap<Cue, Vec<PathBuf>>,
    /// Effets sans fichier déjà signalés dans les logs (un seul `warn` chacun).
    warned: BTreeSet<Cue>,
}

impl SoundBank {
    /// Liste `music/`, `ambience/` et `effects/` de `dir`.
    pub fn scan(dir: &Path) -> Self {
        if !dir.is_dir() {
            log::info!(
                "aucun dossier des sons ({}) : le jeu reste muet",
                dir.display()
            );
            return Self::default();
        }
        let music = Music::ALL
            .into_iter()
            .filter_map(|music| {
                existing(dir.join("music").join(format!("{}.ogg", music.file_stem())))
                    .map(|path| (music, path))
            })
            .collect();
        let ambience = Ambience::ALL
            .into_iter()
            .filter_map(|ambience| {
                existing(
                    dir.join("ambience")
                        .join(format!("{}.ogg", ambience.file_stem())),
                )
                .map(|path| (ambience, path))
            })
            .collect();
        let effects = ogg_files(&dir.join("effects"));
        let cues = Cue::ALL
            .into_iter()
            .map(|cue| (cue, variants_of(cue, &effects)))
            .filter(|(_, variants)| !variants.is_empty())
            .collect();
        Self {
            music,
            ambience,
            cues,
            warned: BTreeSet::new(),
        }
    }

    pub fn music(&self, music: Music) -> Option<&Path> {
        self.music.get(&music).map(PathBuf::as_path)
    }

    pub fn ambience(&self, ambience: Ambience) -> Option<&Path> {
        self.ambience.get(&ambience).map(PathBuf::as_path)
    }

    /// Les variantes d'un effet (aucune s'il n'a pas de fichier).
    pub fn variants(&self, cue: Cue) -> &[PathBuf] {
        self.cues.get(&cue).map_or(&[], Vec::as_slice)
    }

    /// Le fichier de la `nth` lecture d'un effet : ses variantes à tour de rôle (C16).
    pub fn cue_file(&mut self, cue: Cue, nth: usize) -> Option<&Path> {
        let variants = self.cues.get(&cue).map_or(&[][..], Vec::as_slice);
        if variants.is_empty() {
            if self.warned.insert(cue) {
                log::warn!(
                    "effet sonore sans fichier : effects/{}.ogg",
                    cue.file_stem()
                );
            }
            return None;
        }
        variants.get(nth % variants.len()).map(PathBuf::as_path)
    }
}

fn existing(path: PathBuf) -> Option<PathBuf> {
    path.is_file().then_some(path)
}

/// Les fichiers `.ogg` d'un dossier (aucun s'il manque).
fn ogg_files(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && path.extension().is_some_and(|ext| ext == "ogg"))
        .collect()
}

/// Les variantes d'un effet : `<nom>.ogg` ou `<nom>_<N>.ogg`, dans l'ordre de leur numéro.
fn variants_of(cue: Cue, files: &[PathBuf]) -> Vec<PathBuf> {
    let stem = cue.file_stem();
    let mut numbered: Vec<(u32, PathBuf)> = files
        .iter()
        .filter_map(|path| {
            let name = path.file_stem()?.to_str()?;
            let number = if name == stem {
                0
            } else {
                name.strip_prefix(stem)?.strip_prefix('_')?.parse().ok()?
            };
            Some((number, path.clone()))
        })
        .collect();
    numbered.sort();
    numbered.into_iter().map(|(_, path)| path).collect()
}
