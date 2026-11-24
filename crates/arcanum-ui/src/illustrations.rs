//! Illustrations générées des cartes et des unités (spec 072, ADR 0009) : PNG de
//! `assets/images/<famille>/<identifiant>.png`, affichés en vignette s'ils sont présents.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use arcanum_core::{AdvancedAction, Artifact, BasicAction, DeedCard, Hero, Spell, Unit};

/// Famille d'illustrations : un dossier de `assets/images`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IllustrationFamily {
    /// Unités : `unit-art`.
    Unit,
    /// Cartes (actions, Blessure, sorts, artefacts) : `card-art`.
    Card,
    /// Portraits des héros, sur l'écran de nouvelle partie (spec 078, C9b) : `hero-art`.
    Hero,
}

impl IllustrationFamily {
    pub const ALL: [IllustrationFamily; 3] = [
        IllustrationFamily::Unit,
        IllustrationFamily::Card,
        IllustrationFamily::Hero,
    ];

    /// Le dossier de la famille dans `assets/images`.
    pub fn dir(self) -> &'static str {
        match self {
            IllustrationFamily::Unit => "unit-art",
            IllustrationFamily::Card => "card-art",
            IllustrationFamily::Hero => "hero-art",
        }
    }
}

/// Une illustration : sa famille et l'identifiant de son élément (clé de libellé sans préfixe ni
/// suffixe).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Illustration {
    pub family: IllustrationFamily,
    pub id: &'static str,
}

impl Illustration {
    /// Chemin du PNG, construit composant par composant (ADR 0004).
    pub fn path(&self, images_dir: &Path) -> PathBuf {
        images_dir
            .join(self.family.dir())
            .join(format!("{}.png", self.id))
    }
}

/// L'illustration d'une unité : `unit.peasants.name` → `unit-art/peasants`.
pub fn unit_illustration(unit: Unit) -> Illustration {
    Illustration {
        family: IllustrationFamily::Unit,
        id: label_id(unit.label_key(), "unit."),
    }
}

/// L'illustration d'une carte : `card.rage.name` → `card-art/rage`.
pub fn card_illustration(card: DeedCard) -> Illustration {
    Illustration {
        family: IllustrationFamily::Card,
        id: label_id(card.label_key(), "card."),
    }
}

/// Le portrait d'un héros : `hero.arythea.name` → `hero-art/arythea` (spec 078, C9b).
pub fn hero_illustration(hero: Hero) -> Illustration {
    Illustration {
        family: IllustrationFamily::Hero,
        id: label_id(hero.label_key(), "hero."),
    }
}

/// `card.rage.name` → `rage`, comme `figure_image` (spec 075).
fn label_id(key: &'static str, prefix: &str) -> &'static str {
    key.strip_prefix(prefix)
        .and_then(|rest| rest.strip_suffix(".name"))
        .unwrap_or(key)
}

/// Toutes les illustrations possibles d'une famille, une par élément du contenu.
pub fn known_illustrations(family: IllustrationFamily) -> Vec<Illustration> {
    match family {
        IllustrationFamily::Unit => Unit::ALL.into_iter().map(unit_illustration).collect(),
        IllustrationFamily::Card => BasicAction::ALL
            .into_iter()
            .map(DeedCard::Basic)
            .chain([DeedCard::Wound])
            .chain(AdvancedAction::ALL.into_iter().map(DeedCard::Advanced))
            .chain(Spell::ALL.into_iter().map(DeedCard::Spell))
            .chain(Artifact::ALL.into_iter().map(DeedCard::Artifact))
            .map(card_illustration)
            .collect(),
        IllustrationFamily::Hero => Hero::ALL.into_iter().map(hero_illustration).collect(),
    }
}

/// Les illustrations présentes dans le dossier des images, lu une fois au lancement : l'écran ne
/// demande jamais un fichier absent (spec 072, C4 et C9).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IllustrationIndex {
    present: BTreeSet<Illustration>,
}

impl IllustrationIndex {
    /// Sans dossier, ou sans dossier de famille, l'index est vide (L2) ; un fichier inconnu est
    /// ignoré (L3).
    pub fn load(images_dir: &Path) -> IllustrationIndex {
        let present = IllustrationFamily::ALL
            .into_iter()
            .flat_map(known_illustrations)
            .filter(|illustration| illustration.path(images_dir).is_file())
            .collect();
        IllustrationIndex { present }
    }

    pub fn contains(&self, illustration: Illustration) -> bool {
        self.present.contains(&illustration)
    }
}
