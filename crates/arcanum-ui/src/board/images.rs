//! Images SVG du plateau de jeu et couleurs de repli (spec 011).

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::BoardActionKind;
use arcanum_core::{
    BasicColor, Enemy, EnemyColor, Feature, Hero, Rampaging, Site, Terrain, TimeOfDay,
};
use eframe::egui::{Color32, TextureId};

/// Fond de la zone du plateau de jeu.
pub const BOARD_BACKGROUND: Color32 = Color32::from_rgb(0x20, 0x21, 0x24);
/// Zone interdite par la forme du plateau de jeu (spec 012) : la mer, avec l'image des cases de
/// mer des tuiles (spec 075, C9), un peu assombrie au dessin.
pub const FORBIDDEN_IMAGE: ImageId = ImageId::new("terrain", "sea");
pub const FORBIDDEN_COLOR: Color32 = Color32::from_rgb(0x16, 0x38, 0x66);

/// Une image du dossier `assets/images` : son sous-dossier et son nom sans extension.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ImageId {
    dir: &'static str,
    stem: &'static str,
}

impl ImageId {
    pub const fn new(dir: &'static str, stem: &'static str) -> ImageId {
        ImageId { dir, stem }
    }

    /// Chemin du fichier, construit composant par composant (ADR 0004).
    pub fn path(&self, images_dir: &Path) -> PathBuf {
        images_dir.join(self.dir).join(format!("{}.svg", self.stem))
    }
}

const TERRAINS: [Terrain; 9] = [
    Terrain::Plains,
    Terrain::Hills,
    Terrain::Forest,
    Terrain::Wasteland,
    Terrain::Desert,
    Terrain::Swamp,
    Terrain::Lake,
    Terrain::Mountain,
    Terrain::Sea,
];

const COLORS: [BasicColor; 4] = [
    BasicColor::Red,
    BasicColor::Blue,
    BasicColor::White,
    BasicColor::Green,
];

pub fn terrain_image(terrain: Terrain) -> ImageId {
    let stem = match terrain {
        Terrain::Plains => "plains",
        Terrain::Hills => "hills",
        Terrain::Forest => "forest",
        Terrain::Wasteland => "wasteland",
        Terrain::Desert => "desert",
        Terrain::Swamp => "swamp",
        Terrain::Lake => "lake",
        Terrain::Mountain => "mountain",
        Terrain::Sea => "sea",
    };
    ImageId::new("terrain", stem)
}

pub fn feature_image(feature: Feature) -> ImageId {
    match feature {
        Feature::Site(site) => ImageId::new("site", site_stem(site)),
        // Jamais dessiné : un jeton couvre toujours le symbole, sinon l'ennemi est vaincu (spec
        // 031, Q5). Un ennemi inconnu de cette couleur, à défaut : le brouillard (spec 075).
        Feature::Rampaging(_) => FOG_IMAGE,
    }
}

/// Décor de relief d'un terrain (spec 075, C12) : `relief/<terrain>.svg`, ou sa variante
/// dégagée `relief/<terrain>_clear.svg` quand la case porte un site, un jeton ou une figurine ;
/// aucun pour un terrain plat.
pub fn relief_image(terrain: Terrain, clear: bool) -> Option<ImageId> {
    let stem = match (terrain, clear) {
        (Terrain::Forest, false) => "forest",
        (Terrain::Forest, true) => "forest_clear",
        (Terrain::Hills, false) => "hills",
        (Terrain::Hills, true) => "hills_clear",
        (Terrain::Mountain, false) => "mountain",
        (Terrain::Mountain, true) => "mountain_clear",
        _ => return None,
    };
    Some(ImageId::new("relief", stem))
}

/// Brouillard d'un défenseur face cachée, teinté à la couleur de sa pile (spec 075, C18).
pub const FOG_IMAGE: ImageId = ImageId::new("figure", "fog");

/// Figurine d'un ennemi (spec 075, C20b) : `figure/<id>.svg`, sur le plateau et sur l'écran de
/// combat ; l'identifiant est celui de sa clé de libellé (`enemy.prowlers.name` → `prowlers`).
pub fn figure_image(enemy: Enemy) -> ImageId {
    ImageId::new("figure", enemy_stem(enemy))
}

/// Figurine d'un héros sur le plateau (spec 075, C16) : `figure/<héros>.svg` ; le portrait
/// `hero/<héros>.svg` reste celui du panneau.
pub fn hero_figure_image(hero: Hero) -> ImageId {
    ImageId::new("figure", hero_stem(hero))
}

fn enemy_stem(enemy: Enemy) -> &'static str {
    let key = enemy.label_key();
    key.strip_prefix("enemy.")
        .and_then(|rest| rest.strip_suffix(".name"))
        .unwrap_or(key)
}

fn hero_stem(hero: Hero) -> &'static str {
    match hero {
        Hero::Arythea => "arythea",
        Hero::Goldyx => "goldyx",
        Hero::Norowas => "norowas",
        Hero::Tovak => "tovak",
    }
}

/// Couleur de repli d'un jeton : le fond du visuel de sa pile (spec 030).
pub fn enemy_color(color: EnemyColor) -> Color32 {
    match color {
        EnemyColor::Green => Color32::from_rgb(0x2E, 0x5E, 0x2E),
        EnemyColor::Grey => Color32::from_rgb(0x6B, 0x6B, 0x70),
        EnemyColor::Violet => Color32::from_rgb(0x5B, 0x3A, 0x7A),
        EnemyColor::Brown => Color32::from_rgb(0x6B, 0x4A, 0x2E),
        EnemyColor::Red => Color32::from_rgb(0x6E, 0x1E, 0x2A),
        EnemyColor::White => Color32::from_rgb(0xD8, 0xD8, 0xD0),
    }
}

fn site_stem(site: Site) -> &'static str {
    match site {
        Site::Portal => "portal",
        Site::Village => "village",
        Site::Monastery => "monastery",
        Site::MagicalGlade => "magical_glade",
        Site::Keep => "keep",
        Site::MageTower => "mage_tower",
        Site::Dungeon => "dungeon",
        Site::Tomb => "tomb",
        Site::MonsterDen => "monster_den",
        Site::SpawningGrounds => "spawning_grounds",
        Site::Ruins => "ruins",
        Site::Mine(BasicColor::Red) => "mine_red",
        Site::Mine(BasicColor::Blue) => "mine_blue",
        Site::Mine(BasicColor::White) => "mine_white",
        Site::Mine(BasicColor::Green) => "mine_green",
        Site::City(BasicColor::Red) => "city_red",
        Site::City(BasicColor::Blue) => "city_blue",
        Site::City(BasicColor::White) => "city_white",
        Site::City(BasicColor::Green) => "city_green",
    }
}

/// Icône soleil ou lune du panneau latéral (spec 023).
pub fn time_of_day_image(time: TimeOfDay) -> ImageId {
    match time {
        TimeOfDay::Day => ImageId::new("time_of_day", "day"),
        TimeOfDay::Night => ImageId::new("time_of_day", "night"),
    }
}

/// Voile posé sur le plateau de jeu selon le moment de la journée (spec 023, Q3).
pub fn night_veil(time: TimeOfDay) -> Option<Color32> {
    match time {
        TimeOfDay::Day => None,
        // Bleu nuit à moitié transparent : les terrains restent reconnaissables.
        TimeOfDay::Night => Some(Color32::from_rgba_unmultiplied(0x0B, 0x16, 0x3A, 0x78)),
    }
}

/// Toutes les images du plateau de jeu (76), sans doublon : terrains, sites, portraits et drapeaux
/// des héros, figurines (héros, ennemis, brouillard) et décors de relief (spec 075, C20).
pub fn board_images() -> Vec<ImageId> {
    let sites = [
        Site::Portal,
        Site::Village,
        Site::Monastery,
        Site::MagicalGlade,
        Site::Keep,
        Site::MageTower,
        Site::Dungeon,
        Site::Tomb,
        Site::MonsterDen,
        Site::SpawningGrounds,
        Site::Ruins,
    ]
    .into_iter()
    .chain(COLORS.map(Site::Mine))
    .chain(COLORS.map(Site::City))
    .map(Feature::Site);
    let reliefs = [Terrain::Forest, Terrain::Hills, Terrain::Mountain]
        .into_iter()
        .flat_map(|terrain| [false, true].map(|clear| relief_image(terrain, clear)))
        .flatten();
    let images: BTreeSet<ImageId> = TERRAINS
        .map(terrain_image)
        .into_iter()
        .chain(sites.map(feature_image))
        .chain(Hero::ALL.map(hero_image))
        .chain(Hero::ALL.map(flag_image))
        .chain(Hero::ALL.map(hero_figure_image))
        .chain(Enemy::ALL.map(figure_image))
        .chain([FOG_IMAGE])
        .chain(reliefs)
        .chain(BOARD_ACTION_KINDS.map(board_action_image))
        .collect();
    images.into_iter().collect()
}

/// Les sortes de boutons d'action du plateau, dans l'ordre de la spec 075.
pub const BOARD_ACTION_KINDS: [BoardActionKind; 7] = [
    BoardActionKind::Challenge,
    BoardActionKind::StartCombat,
    BoardActionKind::Interact,
    BoardActionKind::Plunder,
    BoardActionKind::Burn,
    BoardActionKind::Enter,
    BoardActionKind::Tribute,
];

/// Icône d'un bouton d'action du plateau (spec 075) : `action/<icône>.svg` ; « Défier » et
/// « Engager le combat » partagent les épées, sur un fond différent.
pub fn board_action_image(kind: BoardActionKind) -> ImageId {
    let stem = match kind {
        BoardActionKind::Challenge | BoardActionKind::StartCombat => "swords",
        BoardActionKind::Interact => "speech",
        BoardActionKind::Plunder => "purse",
        BoardActionKind::Burn => "flame",
        BoardActionKind::Enter => "door",
        BoardActionKind::Tribute => "altar",
    };
    ImageId::new("action", stem)
}

/// Drapeau d'un héros, à sa couleur, planté sur un site conquis ou un monastère incendié (spec 043,
/// C38b ; il remplace le jeton Bouclier dessiné de la spec 041) : `flag/<héros>.svg`.
pub fn flag_image(hero: Hero) -> ImageId {
    let stem = match hero {
        Hero::Arythea => "arythea",
        Hero::Goldyx => "goldyx",
        Hero::Norowas => "norowas",
        Hero::Tovak => "tovak",
    };
    ImageId::new("flag", stem)
}

/// Illustration d'un héros (spec 065) : sa figurine sur le plateau de jeu, son portrait dans le
/// panneau et sur l'écran de nouvelle partie.
pub fn hero_image(hero: Hero) -> ImageId {
    let stem = match hero {
        Hero::Arythea => "arythea",
        Hero::Goldyx => "goldyx",
        Hero::Norowas => "norowas",
        Hero::Tovak => "tovak",
    };
    ImageId::new("hero", stem)
}

/// Couleur de repli d'un héros, celle du fond et de la bordure de son médaillon (spec 065).
pub fn hero_color(hero: Hero) -> Color32 {
    match hero {
        Hero::Arythea => Color32::from_rgb(0xC6, 0x28, 0x28),
        Hero::Goldyx => Color32::from_rgb(0x2E, 0x7D, 0x32),
        Hero::Norowas => Color32::from_rgb(0x5C, 0x7F, 0xB0),
        Hero::Tovak => Color32::from_rgb(0xC9, 0xA2, 0x27),
    }
}

/// Couleur de repli d'un terrain, base aussi de son image.
pub fn terrain_color(terrain: Terrain) -> Color32 {
    match terrain {
        Terrain::Plains => Color32::from_rgb(0xA5, 0xD3, 0x6A),
        Terrain::Hills => Color32::from_rgb(0xC9, 0xA1, 0x5E),
        Terrain::Forest => Color32::from_rgb(0x2F, 0x7A, 0x36),
        Terrain::Wasteland => Color32::from_rgb(0x9A, 0x95, 0x80),
        Terrain::Desert => Color32::from_rgb(0xEA, 0xD5, 0x8A),
        Terrain::Swamp => Color32::from_rgb(0x5E, 0x7D, 0x6B),
        Terrain::Lake => Color32::from_rgb(0x5A, 0xA0, 0xE0),
        Terrain::Mountain => Color32::from_rgb(0x6E, 0x5E, 0x58),
        Terrain::Sea => Color32::from_rgb(0x1E, 0x4A, 0x86),
    }
}

fn basic_color(color: BasicColor) -> Color32 {
    match color {
        BasicColor::Red => Color32::from_rgb(0xD8, 0x40, 0x40),
        BasicColor::Blue => Color32::from_rgb(0x3F, 0x7F, 0xD6),
        BasicColor::White => Color32::from_rgb(0xF2, 0xF2, 0xF2),
        BasicColor::Green => Color32::from_rgb(0x3F, 0xA3, 0x4D),
    }
}

/// Couleur de repli d'un site ou d'un ennemi errant.
pub fn feature_color(feature: Feature) -> Color32 {
    match feature {
        Feature::Site(Site::Mine(color) | Site::City(color)) => basic_color(color),
        Feature::Site(_) => Color32::from_rgb(0xFA, 0xF5, 0xE6),
        Feature::Rampaging(Rampaging::OrcMarauders) => Color32::from_rgb(0x2E, 0x5E, 0x2E),
        Feature::Rampaging(Rampaging::Draconum) => Color32::from_rgb(0x8E, 0x2A, 0x2A),
    }
}

/// Taille de conversion d'un SVG : la plus petite de la série qui couvre la taille affichée,
/// pour ne pas reconvertir l'image à chaque cran de zoom.
pub fn svg_size(pixels: f32) -> u32 {
    const SERIES: [u32; 4] = [32, 64, 128, 256];
    SERIES
        .into_iter()
        .find(|size| *size as f32 >= pixels)
        .unwrap_or(256)
}

/// État du chargement d'une image, tel que egui le rapporte.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImageLoad {
    Ready(TextureId),
    Pending,
    Failed(String),
}

/// Ce qu'il faut peindre : l'image, ou sa couleur de repli.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImagePaint {
    Texture(TextureId),
    Fallback(Color32),
    /// Image en cours de chargement : le sol suffit, pas de disque blanc qui clignote (issue privée 201).
    Pending(Color32),
}

/// Images déjà signalées manquantes, pour ne les écrire qu'une fois dans les logs : SVG et
/// illustrations PNG (spec 072, L1), par chemin.
#[derive(Debug, Default)]
pub struct ImageWarnings {
    reported: BTreeSet<PathBuf>,
}

/// Image chargée, sinon couleur de repli ; un échec est signalé une seule fois.
pub fn image_paint(
    load: ImageLoad,
    // Le chemin identifie l'image : `ImageId` garde la signature de la spec 011.
    _image: ImageId,
    fallback: Color32,
    path: &Path,
    warnings: &mut ImageWarnings,
) -> ImagePaint {
    match load {
        ImageLoad::Ready(texture) => ImagePaint::Texture(texture),
        ImageLoad::Pending => ImagePaint::Pending(fallback),
        ImageLoad::Failed(reason) => {
            warn_once(path, &reason, warnings);
            ImagePaint::Fallback(fallback)
        }
    }
}

/// La texture d'une illustration chargée, sinon rien : l'écran reste celui sans image ; un PNG
/// illisible est signalé une seule fois (spec 072, L1).
pub fn illustration_paint(
    load: ImageLoad,
    path: &Path,
    warnings: &mut ImageWarnings,
) -> Option<TextureId> {
    match load {
        ImageLoad::Ready(texture) => Some(texture),
        ImageLoad::Pending => None,
        ImageLoad::Failed(reason) => {
            warn_once(path, &reason, warnings);
            None
        }
    }
}

fn warn_once(path: &Path, reason: &str, warnings: &mut ImageWarnings) {
    if warnings.reported.insert(path.to_path_buf()) {
        log::warn!("image illisible {} : {reason}", path.display());
    }
}
