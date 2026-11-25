//! Écran du plateau : ce qu'il faut dessiner, calculé sans egui (spec 011).

use std::collections::BTreeMap;

use arcanum_core::{
    Action, BasicColor, CityState, Direction, EXPLORE_COST, EnemyAbility, EnemyColor, EnemyToken,
    Feature, Hero, Hex, Inventory, Map, MapShape, MoveCost, MoveRules, NextTile, Placement,
    PlayerId, Rampaging, Route, RuinToken, Site, SpaceContent, Terrain, TileId, TilePosition,
    free_positions_touching, is_safe_space, mine_color, move_cost, placement, provoked,
    routes_ending_in,
};
use eframe::egui::{Color32, Pos2, Rect, Vec2, pos2, vec2};

use crate::enemy_text::{token_lines, token_names};
use crate::screen::{adventure_reward, enemy_groups_text, token_reward_text};
use crate::{BoardActionButton, CastDestination, Command};
use arcanum_i18n::Catalog;

use super::camera::BoardCamera;
use super::geometry::{hex_center, hex_corners, point_to_hex};
use super::images::{
    FOG_IMAGE, FORBIDDEN_COLOR, FORBIDDEN_IMAGE, ImageId, enemy_color, feature_color,
    feature_image, figure_image, flag_image, hero_color, hero_figure_image, relief_image,
    terrain_color, terrain_image,
};

/// Sous ce zoom, pictogrammes, jetons et figurine sont illisibles : on ne les dessine pas.
pub const OVERLAY_MIN_ZOOM: f32 = 16.0;
/// Éléments debout (spec 075) : pied, en unités du plateau depuis le centre de la case, et taille,
/// en rayons de case. Les pieds sont rangés pour que le tri par profondeur donne, sur une case :
/// site (ou ennemi à sa place), drapeau ou défenseur à sa gauche, héros devant.
const SITE_FOOT: Vec2 = vec2(0.0, 0.1);
/// Le site, agrandi de 1,3 par rapport à l'échelle des figurines pour les dominer (Mathieu).
const SITE_SIZE: Vec2 = vec2(1.3, 1.3);
/// Décor de relief : cadre de 1,8 × 2,4 rayons, le centre de la case à (0,5 ; 0,625) du cadre
/// (`assets/images/relief/README.md`).
const RELIEF_SIZE: Vec2 = vec2(1.8, 2.4);
const RELIEF_FOOT: Vec2 = vec2(0.5, 0.625);
/// Un ennemi debout : une figurine, plus haute que large (cadre 120 × 160).
const ENEMY_SIZE: Vec2 = vec2(0.75, 1.0);
/// Deux ennemis d'un Sombre Terrier, côte à côte et réduits (spec 044, C41).
const PAIR_DX: f32 = 0.3;
const PAIR_SIZE: Vec2 = vec2(0.5, 0.67);
/// Le défenseur, plus petit, à gauche du site (spec 043, C35).
const DEFENDER: (Vec2, Vec2) = (vec2(-0.5, 0.25), vec2(0.45, 0.6));
/// Le drapeau du héros, à gauche du site, à la place du défenseur qu'un site conquis n'a plus :
/// à droite, la figurine restée sur le site le cachait (issue privée 292).
const FLAG: (Vec2, Vec2) = (vec2(-0.5, 0.25), vec2(0.38, 0.38));
/// La figurine du héros, devant le site et à droite de son centre.
const HERO: (Vec2, Vec2) = (vec2(0.4, 0.55), vec2(0.6, 0.8));
/// Rayon du jeton de ruines, à gauche du site comme un défenseur, en rayons de case (spec 045,
/// C31).
const RUIN_TOKEN_RADIUS: f32 = 0.27;
/// Centre du jeton de ruines, en unités du plateau depuis le centre de la case (spec 075).
const RUIN_TOKEN_FOOT: Vec2 = vec2(-0.55, 0.4);
/// Rayon du jeton de coût, en rayons de case.
const COST_BADGE_RADIUS: f32 = 0.28;

/// Ce dont l'écran du plateau a besoin, sans egui.
pub struct BoardInput<'a> {
    pub map: &'a Map,
    pub figure: Option<Hex>,
    /// Héros de la partie : son illustration sert de figurine (spec 065).
    pub hero: Option<Hero>,
    /// Forme du plateau de jeu : ce qui est hors forme et sans tuile est hachuré (spec 012).
    pub shape: MapShape,
    pub camera: BoardCamera,
    pub area: Rect,
    pub cursor: Option<Pos2>,
    /// Inventaire du joueur, pour dire ce que rapporterait une mine (spec 021) ; absent hors
    /// partie.
    pub inventory: Option<Inventory>,
    /// Ce qu'il faut pour le déplacement (spec 014) ; absent hors partie.
    pub movement: Option<BoardMovement<'a>>,
    /// Jetons d'ennemis face visible sur le plateau de jeu (spec 030).
    /// Une liste par case (spec 044).
    pub enemy_tokens: &'a BTreeMap<Hex, Vec<EnemyToken>>,
    /// Jetons Bouclier posés, par case (spec 041).
    pub shields: &'a BTreeMap<Hex, PlayerId>,
    /// Défenseurs face cachée : leur case et leur couleur seulement (spec 043).
    /// Spec 062 : plusieurs couleurs sur une cité.
    pub face_down_defenders: &'a BTreeMap<Hex, Vec<EnemyColor>>,
    /// Cités en jeu : couleur, niveau, Boucliers, Seigneur (spec 062) ; une cité fermée n'y est pas.
    pub cities: &'a BTreeMap<Hex, CityState>,
    /// Ruines non conquises : leur jeton face visible, ou `None` face cachée (spec 045).
    pub ruins: &'a BTreeMap<Hex, Option<RuinToken>>,
    /// Boutons d'action à poser sur leur case (spec 075, C24) ; vides hors partie et en mode fin
    /// du tour (C33).
    pub actions: &'a [BoardActionButton],
    /// Case du menu du site ou d'une confirmation, à côté de laquelle ils s'ouvrent (spec 075,
    /// C30, C31).
    pub site_anchor: Option<Hex>,
    /// Temps de l'essai d'animation, en secondes (spec 075, Q6) : l'écran reste une fonction pure.
    pub time: f64,
    /// Dernier déplacement de la figurine, qu'elle suit en glissant (C35).
    pub walk: Option<HeroWalk<'a>>,
    /// Spec #5 : les ennemis vaincus par le dernier combat, que le plateau efface.
    pub departure: Option<BoardDeparture<'a>>,
    /// Spec #5 : la préférence « Animations » ; fausse, ni respiration, ni glissement, ni
    /// effacement (C14).
    pub animations: bool,
    /// Arrivées d'un déplacement par effet payé (spec 057, issue privée 257) : en évidence, un clic y
    /// joue le sort ; vides hors de ce choix.
    pub destinations: &'a [CastDestination],
}

/// Le trajet que vient de jouer la figurine, de sa case de départ à sa case d'arrivée, et
/// l'instant où il a commencé (spec 075, C35).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HeroWalk<'a> {
    pub path: &'a [Hex],
    pub started: f64,
}

/// Un ennemi vaincu par le dernier combat, que le plateau efface à son retour (spec #5) : sa case,
/// son jeton, et sa place parmi les `of` jetons que la case montrait avant le combat (paire réduite
/// d'un site d'aventure).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DepartingEnemy {
    pub hex: Hex,
    pub token: EnemyToken,
    pub place: usize,
    pub of: usize,
}

/// Les ennemis vaincus que le plateau efface, et l'instant où l'effacement a commencé (spec #5).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoardDeparture<'a> {
    pub enemies: &'a [DepartingEnemy],
    pub started: f64,
}

/// Durée de l'effacement d'un ennemi vaincu (spec #5, Q1).
pub const DEPARTURE_SECONDS: f64 = 0.8;

/// Respiration de la figurine du héros : ± 2 % de sa hauteur, période 3 s (spec 075, C34).
pub const BREATHING_AMPLITUDE: f32 = 0.02;
pub const BREATHING_PERIOD: f64 = 3.0;
/// Glissement de la figurine : 150 ms par pas (C35).
pub const WALK_STEP_SECONDS: f64 = 0.15;

/// Données de la vue joueur et actions légales qui servent au déplacement (spec 014).
#[derive(Clone, Copy, Debug)]
pub struct BoardMovement<'a> {
    /// Moment de la journée et effets en cours sur les terrains (spec 052, C17, C43).
    pub move_rules: &'a MoveRules,
    pub move_points: u32,
    /// Ce que révélerait une exploration (spec 015).
    pub next_tile: NextTile,
    /// Vide sans décision en attente : rien n'est alors mis en évidence ni cliquable.
    pub legal_actions: &'a [Action],
    /// Déplacement terminé par une provocation (spec 030) : plus de fantôme d'exploration.
    pub movement_ended: bool,
    /// Cases des ennemis provoqués ce tour (spec 030) : leur « ! » reste jusqu'à la fin du tour.
    pub enemies_to_fight: &'a [Hex],
}

/// Marque d'une case adjacente accessible : son jeton de coût, et sa mise en évidence si la
/// figurine peut y aller maintenant.
#[derive(Clone, Debug, PartialEq)]
pub struct MoveMark {
    pub hex: Hex,
    /// Coût du trajet retenu (spec 017) ; pour une voisine, celui de la case.
    pub cost: u8,
    /// Case où un clic agit maintenant, voisine ou lointaine (spec 036, Q2) : voile lumineux,
    /// contour épais, jeton normal.
    pub reachable: bool,
    /// Voisine de la figurine : un clic la joue en un pas, et le pas peut provoquer (spec 030).
    pub neighbor: bool,
    /// Le pas vers cette voisine provoque un ennemi errant : anneau orange et « ! » (spec 030).
    pub provokes: bool,
    /// Cible d'un assaut (forteresse ou tour de mage non conquise) : pastille rouge « ⚔ coût »
    /// au lieu de la pastille crème (spec 043, C37).
    pub assault: bool,
    pub corners: [Pos2; 6],
    pub badge_center: Pos2,
    pub badge_radius: f32,
}

/// Emplacement explorable qui touche la figurine, dessiné en fantôme (spec 015).
#[derive(Clone, Debug, PartialEq)]
pub struct ExploreGhost {
    pub position: TilePosition,
    /// Vrai si un clic l'explore maintenant : mis en évidence, sinon grisé.
    pub reachable: bool,
    /// Les sept cases, à l'écran.
    pub hexes: Vec<[Pos2; 6]>,
    /// Contour extérieur, à l'écran (dessiné en pointillé).
    pub segments: Vec<[Pos2; 2]>,
    /// Centre de la case centrale, où va le point d'interrogation.
    pub center: Pos2,
    /// Jeton de coût (centre, rayon) ; absent sous le zoom minimal des surimpressions.
    pub badge: Option<(Pos2, f32)>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HexDrawing {
    pub hex: Hex,
    /// Sommets à l'écran, dans l'ordre de [`hex_corners`].
    pub corners: [Pos2; 6],
    /// Rectangle qui englobe la case : là où va l'image du terrain.
    pub rect: Rect,
    pub image: ImageId,
    pub fallback: Color32,
}

/// Profondeur de la tranche du plateau, en rayons de case, sans écrasement (spec 075).
pub const SLAB_DEPTH: f32 = 0.22;

/// Tranche d'une case de tuile, sous son arête sud-est ou sud-ouest quand la voisine de ce côté
/// n'a pas de tuile (spec 075, C11).
#[derive(Clone, Debug, PartialEq)]
pub struct Slab {
    pub hex: Hex,
    /// `Direction::SE` ou `Direction::SW`.
    pub side: Direction,
    /// Les deux sommets de l'arête, puis les mêmes abaissés de la profondeur.
    pub corners: [Pos2; 4],
    /// Couleur du terrain assombrie.
    pub color: Color32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TileOutline {
    pub tile: TileId,
    /// Arêtes du contour extérieur, à l'écran.
    pub segments: Vec<[Pos2; 2]>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverlayKind {
    /// Décor de relief d'une forêt, de collines ou d'une montagne (spec 075, C12).
    Relief,
    Site,
    Rampaging,
    Figure,
    /// Drapeau du héros sur un site conquis ou un monastère incendié (spec 043, C38b).
    Flag,
    /// Défenseur d'un site fortifié, face cachée (dos de jeton) ou retourné (spec 043, C35).
    Defender,
    /// Jeton d'ennemi resté sur un site d'aventure (spec 044, C41) : pas un ennemi errant.
    SiteEnemy,
    /// Ennemi vaincu qui s'efface au retour sur le plateau (spec #5).
    Departing,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Overlay {
    pub hex: Hex,
    pub kind: OverlayKind,
    pub rect: Rect,
    /// Point posé sur la case, qui range l'élément de l'arrière vers l'avant (spec 075, C13) :
    /// le milieu du bas du rectangle, sauf pour le décor de relief, posé par le centre de sa case.
    pub foot: Pos2,
    pub image: ImageId,
    pub fallback: Color32,
    /// Couleur qui multiplie l'image : blanc, sauf le brouillard, à la couleur de la pile du
    /// défenseur face cachée (spec 075, C18).
    pub tint: Color32,
    /// Socle d'un ennemi, à la couleur de sa catégorie, peint à plat sous ses pieds (spec #14, C9
    /// à C12) ; aucun pour les autres éléments.
    pub base: Option<Color32>,
    /// Part de la hauteur de l'image qui reste debout (spec #5) : 1, sauf un ennemi vaincu qui
    /// s'affaisse ; l'image garde sa largeur et s'écrase dans `rect`.
    pub sink: f32,
}

impl Overlay {
    /// Un élément debout posé par le milieu du bas de son rectangle, sans teinte.
    fn standing(
        hex: Hex,
        kind: OverlayKind,
        rect: Rect,
        image: ImageId,
        fallback: Color32,
    ) -> Overlay {
        Overlay {
            hex,
            kind,
            rect,
            foot: rect.center_bottom(),
            image,
            fallback,
            tint: Color32::WHITE,
            base: None,
            sink: 1.0,
        }
    }

    /// Un ennemi debout : repli et socle à la couleur de sa pile (spec #14, C9 à C11).
    fn enemy(
        hex: Hex,
        kind: OverlayKind,
        rect: Rect,
        image: ImageId,
        color: EnemyColor,
    ) -> Overlay {
        Overlay {
            base: Some(enemy_color(color)),
            ..Overlay::standing(hex, kind, rect, image, enemy_color(color))
        }
    }
}

/// « ! » orange sur le jeton d'un ennemi qu'un pas voisin provoquerait, ou qui vient d'être
/// provoqué (spec 030, C24 bis et C26).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnemyAlert {
    /// Case du jeton.
    pub hex: Hex,
    pub center: Pos2,
    pub radius: f32,
}

/// Une pastille du jeton de ruines : la couleur du mana de l'autel, ou celle d'une pile d'ennemis
/// à tirer (spec 045, Q5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuinPip {
    Mana(BasicColor),
    Enemy(EnemyColor),
}

/// Le jeton de ruines d'une ruine non conquise, dessiné par le code : un hexagone jaune à côté du
/// site, avec ses pastilles face visible, uni face cachée (spec 045, C31).
#[derive(Clone, Debug, PartialEq)]
pub struct RuinTokenMark {
    pub hex: Hex,
    pub center: Pos2,
    /// Rayon de l'hexagone, à l'écran.
    pub radius: f32,
    pub face_up: bool,
    /// Vides face cachée.
    pub pips: Vec<RuinPip>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hovered {
    pub hex: Hex,
    pub corners: [Pos2; 6],
}

#[derive(Clone, Debug, PartialEq)]
pub struct BoardScreen {
    pub hexes: Vec<HexDrawing>,
    /// Cases de la zone interdite : emplacements sans tuile que la forme ne permet pas.
    pub forbidden: Vec<HexDrawing>,
    /// Tranches du plateau, sous les cases de tuile du bord sud (spec 075).
    pub slabs: Vec<Slab>,
    pub tile_outlines: Vec<TileOutline>,
    pub overlays: Vec<Overlay>,
    /// Jetons de ruines, dessinés après les pictogrammes (spec 045, C31).
    pub ruin_tokens: Vec<RuinTokenMark>,
    pub hovered: Option<Hovered>,
    /// Lignes de l'infobulle de survol, déjà traduites ; vides s'il n'y a rien à décrire (spec 037).
    pub hover_lines: Vec<String>,
    /// Cases adjacentes accessibles, dans l'ordre des directions (spec 014).
    pub move_marks: Vec<MoveMark>,
    /// Emplacements explorables qui touchent la figurine (spec 015).
    pub explore_ghosts: Vec<ExploreGhost>,
    /// Jetons d'ennemis marqués d'un « ! » (spec 030), dans l'ordre des cases.
    pub enemy_alerts: Vec<EnemyAlert>,
    /// Tracé du trajet vers la case survolée, des centres de la figurine à la case (spec 017) ;
    /// vide si elle n'est pas atteignable.
    pub route: Vec<Pos2>,
    /// Le trajet survolé finit par un assaut : tracé en rouge (spec 043, C37).
    pub route_assault: bool,
    /// Index de la ligne de l'infobulle à écrire en rouge et en gras : l'avertissement d'assaut
    /// (spec 043, C36).
    pub hover_alert: Option<usize>,
    /// Ce qu'enverrait un clic à la position du curseur : un pas, un trajet suivi (spec 036), une
    /// exploration, ou rien.
    pub click: Option<Command>,
    /// Boutons d'action posés sur leur case, de taille fixe (spec 075, C24 à C29).
    pub actions: Vec<BoardAction>,
    /// Coin haut gauche du menu du site ou de la confirmation, à côté de sa case (C30, C31).
    pub site_anchor: Option<Pos2>,
    /// Arrivées d'un déplacement par effet, mises en évidence comme les cases atteignables (spec
    /// 057, issue privée 257).
    pub destination_marks: Vec<Hovered>,
}

/// Diamètre d'un bouton d'action du plateau, en points, quel que soit le zoom (spec 075, C28).
pub const BOARD_ACTION_DIAMETER: f32 = 36.0;

/// Un bouton d'action posé sur sa case, à l'écran (spec 075).
#[derive(Clone, Debug, PartialEq)]
pub struct BoardAction {
    pub button: BoardActionButton,
    pub center: Pos2,
    pub radius: f32,
}

/// Direction de la voisine de l'autre côté de l'arête `k` (du sommet `k` au sommet `k + 1`).
const EDGE_DIRECTIONS: [Direction; 6] = [
    Direction::NE,
    Direction::E,
    Direction::SE,
    Direction::SW,
    Direction::W,
    Direction::NW,
];

pub fn board_screen(input: &BoardInput<'_>, catalog: &Catalog) -> BoardScreen {
    let BoardInput {
        map, camera, area, ..
    } = *input;
    let on_screen = |hex: Hex| hex_corners(hex).map(|corner| camera.to_screen(area, corner));
    let visible = |corners: &[Pos2; 6]| {
        area.width() > 0.0 && area.height() > 0.0 && Rect::from_points(corners).intersects(area)
    };

    let mut hexes = vec![];
    let mut tile_outlines = vec![];
    for (position, tile) in map.tiles() {
        let tile_hexes = position.hexes();
        let mut any_drawn = false;
        for hex in tile_hexes {
            let Some(content) = map.space(hex) else {
                continue;
            };
            let corners = on_screen(hex);
            if !visible(&corners) {
                continue;
            }
            any_drawn = true;
            hexes.push(HexDrawing {
                hex,
                corners,
                rect: Rect::from_points(&corners),
                image: terrain_image(content.terrain),
                fallback: terrain_color(content.terrain),
            });
        }
        if any_drawn {
            tile_outlines.push(TileOutline {
                tile,
                segments: outline(&tile_hexes, &on_screen),
            });
        }
    }

    let mut overlays = vec![];
    if camera.zoom >= OVERLAY_MIN_ZOOM {
        // Spec 075 : chaque élément est debout, son pied posé sur la case (en unités du plateau,
        // depuis le centre), sa taille en rayons de case, sans écrasement.
        let standing = |hex: Hex, (foot, size): (Vec2, Vec2)| {
            let foot = camera.to_screen(area, hex_center(hex) + foot);
            Rect::from_min_size(
                foot - vec2(size.x / 2.0, size.y) * camera.zoom,
                size * camera.zoom,
            )
        };
        for drawing in &hexes {
            let hex = drawing.hex;
            let first_on_hex = overlays.len();
            let fortified = is_fortified_site(input, hex);
            let site_tokens = adventure_tokens(input, hex);
            if !site_tokens.is_empty() {
                // Spec 044, C41 : les jetons restés sur un site d'aventure, à la place du symbole
                // imprimé comme ceux des ennemis errants ; deux côte à côte, réduits.
                let pair = site_tokens.len() > 1;
                for (place, token) in site_tokens.iter().enumerate() {
                    let rect = if pair {
                        let dx = if place == 0 { -PAIR_DX } else { PAIR_DX };
                        standing(hex, (vec2(dx, SITE_FOOT.y), PAIR_SIZE))
                    } else {
                        standing(hex, (SITE_FOOT, ENEMY_SIZE))
                    };
                    overlays.push(Overlay::enemy(
                        hex,
                        OverlayKind::SiteEnemy,
                        rect,
                        figure_image(token.enemy()),
                        token.color(),
                    ));
                }
            } else if let Some(token) =
                first_token(input, hex).filter(|_| !holds_site_tokens(input, hex))
            {
                // Le jeton se pose sur le symbole imprimé et le cache (spec 030).
                overlays.push(Overlay::enemy(
                    hex,
                    OverlayKind::Rampaging,
                    standing(hex, (SITE_FOOT, ENEMY_SIZE)),
                    figure_image(token.enemy()),
                    token.color(),
                ));
            } else if let Some(feature) = map
                .space(hex)
                .and_then(|content| content.feature)
                .filter(|feature| printed_feature_shown(*feature))
            {
                let (kind, size) = match feature {
                    Feature::Site(_) => (OverlayKind::Site, SITE_SIZE),
                    Feature::Rampaging(_) => (OverlayKind::Rampaging, ENEMY_SIZE),
                };
                overlays.push(Overlay::standing(
                    hex,
                    kind,
                    standing(hex, (SITE_FOOT, size)),
                    feature_image(feature),
                    feature_color(feature),
                ));
            }
            // Spec 043 : le défenseur, plus petit, à gauche du site, qui reste visible ; face
            // cachée, un brouillard à la couleur de sa pile (spec 075, C18).
            let defender = match face_down_color(input, hex) {
                Some(color) => Some((FOG_IMAGE, color, enemy_color(color))),
                None => first_token(input, hex)
                    .filter(|_| fortified)
                    .map(|token| (figure_image(token.enemy()), token.color(), Color32::WHITE)),
            };
            if let Some((image, color, tint)) = defender {
                overlays.push(Overlay {
                    tint,
                    ..Overlay::enemy(
                        hex,
                        OverlayKind::Defender,
                        standing(hex, DEFENDER),
                        image,
                        color,
                    )
                });
            }
            // Spec 043, C38b : le drapeau du héros (en solo, le seul à poser des Boucliers),
            // planté à gauche du site, hors de la figurine (issue privée 292).
            // Spec 062 : une cité conquise aussi.
            if (input.shields.contains_key(&hex)
                || input
                    .cities
                    .get(&hex)
                    .is_some_and(|city| city.is_conquered()))
                && let Some(hero) = input.hero
            {
                overlays.push(Overlay::standing(
                    hex,
                    OverlayKind::Flag,
                    standing(hex, FLAG),
                    flag_image(hero),
                    hero_color(hero),
                ));
            }
            // Spec 065 : la figurine est l'illustration du héros de la partie ; devant le site et
            // un peu à droite, pour ne pas en cacher le centre (spec 075, C14).
            if input.figure == Some(hex)
                && let Some(hero) = input.hero
            {
                // Essai d'animation (Q6) : le glissement le long du trajet, puis la respiration.
                let rect = match walking_point(input) {
                    Some(point) => {
                        let foot = camera.to_screen(area, point + HERO.0);
                        Rect::from_min_size(
                            foot - vec2(HERO.1.x / 2.0, HERO.1.y) * camera.zoom,
                            HERO.1 * camera.zoom,
                        )
                    }
                    None => standing(hex, HERO),
                };
                let breath = if input.animations {
                    breathing(input.time)
                } else {
                    1.0
                };
                let height = rect.height() * breath;
                let rect = Rect::from_min_max(pos2(rect.left(), rect.bottom() - height), rect.max);
                overlays.push(Overlay::standing(
                    hex,
                    OverlayKind::Figure,
                    rect,
                    hero_figure_image(hero),
                    hero_color(hero),
                ));
            }
            // Spec 075, C12 : le décor de relief, dégagé si la case porte autre chose.
            let occupied = overlays.len() > first_on_hex || input.ruins.contains_key(&hex);
            if let Some(content) = map.space(hex)
                && let Some(image) = relief_image(content.terrain, occupied)
            {
                let foot = camera.to_screen(area, hex_center(hex));
                let size = RELIEF_SIZE * camera.zoom;
                overlays.push(Overlay {
                    hex,
                    kind: OverlayKind::Relief,
                    rect: Rect::from_min_size(foot - size * RELIEF_FOOT, size),
                    foot,
                    image,
                    fallback: terrain_color(content.terrain),
                    tint: Color32::WHITE,
                    base: None,
                    sink: 1.0,
                });
            }
        }
        // Spec #5 : les ennemis vaincus s'effacent là où le plateau les dessinait avant le combat.
        for (enemy, remaining) in departing(input) {
            let hex = enemy.hex;
            if !hexes.iter().any(|drawing| drawing.hex == hex) {
                continue;
            }
            let rect = if is_fortified_site(input, hex) {
                standing(hex, DEFENDER)
            } else if (adventure_site(input, hex).is_some() || is_ruins(input, hex)) && enemy.of > 1
            {
                let dx = if enemy.place == 0 { -PAIR_DX } else { PAIR_DX };
                standing(hex, (vec2(dx, SITE_FOOT.y), PAIR_SIZE))
            } else {
                standing(hex, (SITE_FOOT, ENEMY_SIZE))
            };
            // Q1 : la figurine s'affaisse vers ses pieds en s'effaçant.
            let height = rect.height() * remaining;
            let rect = Rect::from_min_max(pos2(rect.left(), rect.bottom() - height), rect.max);
            overlays.push(Overlay {
                tint: Color32::WHITE.gamma_multiply(remaining),
                ..Overlay::standing(
                    hex,
                    OverlayKind::Departing,
                    rect,
                    figure_image(enemy.token.enemy()),
                    enemy_color(enemy.token.color()).gamma_multiply(remaining),
                )
            });
        }
        // De l'arrière vers l'avant (spec 075, C13) : par le pied, du haut de l'écran vers le bas,
        // puis de gauche à droite. Tri stable : sur une même case, l'ordre d'ajout reste.
        overlays.sort_by(|a, b| {
            let (a, b) = (a.foot, b.foot);
            a.y.total_cmp(&b.y).then(a.x.total_cmp(&b.x))
        });
    }

    let ruin_tokens = if camera.zoom >= OVERLAY_MIN_ZOOM {
        ruin_marks(input, &hexes)
    } else {
        vec![]
    };

    let forbidden = visible_hexes(camera, area)
        .into_iter()
        .filter(|hex| is_forbidden(input, *hex))
        .filter_map(|hex| {
            let corners = on_screen(hex);
            visible(&corners).then(|| HexDrawing {
                hex,
                corners,
                rect: Rect::from_points(&corners),
                image: FORBIDDEN_IMAGE,
                fallback: FORBIDDEN_COLOR,
            })
        })
        .collect();

    let routes = reachable_routes(input);
    let cursor_hex = input
        .cursor
        .filter(|cursor| area.contains(*cursor) && area.width() > 0.0 && area.height() > 0.0)
        .map(|cursor| point_to_hex(camera.to_board(area, cursor)));
    let hovered = cursor_hex.and_then(|hex| map.space(hex).map(|content| (hex, content)));
    let hover_lines = match (hovered, cursor_hex) {
        (Some((hex, content)), _) => hover_lines(input, &routes, hex, content, catalog),
        (None, Some(hex)) if is_forbidden(input, hex) => {
            vec![catalog.text("board.hover.forbidden")]
        }
        (None, Some(hex)) if is_free_position(input, hex) => {
            free_position_lines(input, hex, catalog)
        }
        // Rien à décrire : pas d'infobulle (spec 037, C3).
        _ => vec![],
    };

    let move_marks = if camera.zoom >= OVERLAY_MIN_ZOOM {
        move_marks(input, &routes, &on_screen)
    } else {
        vec![]
    };
    let explore_ghosts = explore_ghosts(input, &on_screen);
    let enemy_alerts = if camera.zoom >= OVERLAY_MIN_ZOOM {
        enemy_alerts(input, &move_marks)
    } else {
        vec![]
    };
    let route = match cursor_hex {
        Some(hex) if camera.zoom >= OVERLAY_MIN_ZOOM => route_line(input, &routes, hex),
        _ => vec![],
    };
    let route_assault =
        !route.is_empty() && cursor_hex.is_some_and(|hex| is_assault_target(input, hex));
    let assault_line = catalog.text("board.hover.assault");
    let hover_alert = hover_lines.iter().position(|line| *line == assault_line);
    // Issue privée 257 : pendant le choix d'une arrivée, seules les arrivées réagissent au clic.
    let destination_marks: Vec<Hovered> = input
        .destinations
        .iter()
        .map(|destination| Hovered {
            hex: destination.hex,
            corners: on_screen(destination.hex),
        })
        .filter(|mark| visible(&mark.corners))
        .collect();
    let destination_click = |hex: Hex| {
        input
            .destinations
            .iter()
            .find(|destination| destination.hex == hex)
            .map(|destination| destination.command)
    };
    let click = cursor_hex.and_then(|hex| {
        if !input.destinations.is_empty() {
            return destination_click(hex);
        }
        move_index(input, hex)
            .or_else(|| follow_index(input, hex))
            .or_else(|| explore_index(input, TilePosition::containing(hex)))
            .map(Command::Play)
    });

    let slabs = slabs(input, &hexes);
    // Spec 075, C27 : un bouton d'action passe avant la case qu'il recouvre, clic et infobulle.
    let actions = place_actions(input);
    let (hovered, hover_lines, hover_alert, route, route_assault, click) =
        match input.cursor.and_then(|cursor| {
            actions
                .iter()
                .find(|action| action.center.distance(cursor) <= action.radius)
        }) {
            Some(action) => (
                None,
                action.button.tooltip.clone(),
                None,
                vec![],
                false,
                action.button.command,
            ),
            None => (
                hovered,
                hover_lines,
                hover_alert,
                route,
                route_assault,
                click,
            ),
        };
    BoardScreen {
        hexes,
        forbidden,
        slabs,
        tile_outlines,
        overlays,
        ruin_tokens,
        hovered: hovered.map(|(hex, _)| Hovered {
            hex,
            corners: on_screen(hex),
        }),
        hover_lines,
        move_marks,
        explore_ghosts,
        enemy_alerts,
        route,
        route_assault,
        hover_alert,
        click,
        actions,
        site_anchor: input.site_anchor.map(|hex| site_anchor(input, hex)),
        destination_marks,
    }
}

/// Délai avant la prochaine image de l'essai d'animation : au plus 30 par seconde, et seulement
/// quand le plateau est affiché avec le héros et que la fenêtre a le focus (spec 075, L7).
pub fn animation_repaint(animated: bool, focused: bool) -> Option<std::time::Duration> {
    (animated && focused).then(|| std::time::Duration::from_millis(34))
}

/// Le plateau s'anime : figurine du héros (respiration, glissement) ou effacement en cours
/// (spec #5, C11), et animations permises (C14).
pub fn board_animated(input: &BoardInput<'_>) -> bool {
    input.animations
        && ((input.hero.is_some() && input.figure.is_some()) || departing(input).next().is_some())
}

/// Les ennemis en train de s'effacer, chacun avec la part de sa hauteur et de son opacité qui
/// reste : de 1 à 0 en `DEPARTURE_SECONDS`, d'un mouvement qui ralentit à la fin (Q1).
fn departing<'a>(input: &BoardInput<'a>) -> impl Iterator<Item = (&'a DepartingEnemy, f32)> {
    let departure = input.departure.filter(|_| input.animations);
    let progress = departure.map_or(1.0, |departure| {
        ((input.time - departure.started) / DEPARTURE_SECONDS).max(0.0)
    });
    departure
        .filter(|_| progress < 1.0)
        .into_iter()
        .flat_map(|departure| departure.enemies)
        .map(move |enemy| (enemy, ((1.0 - progress) * (1.0 - progress)) as f32))
}

/// Où est la figurine pendant son glissement, en unités du plateau (spec 075, C35) : entre deux
/// cases du trajet ; `None` une fois arrivée, ou si le trajet ne mène pas à sa case (L6).
fn walking_point(input: &BoardInput<'_>) -> Option<Pos2> {
    // C14 : sans animations, la figurine est aussitôt sur sa case.
    let walk = input.walk.filter(|_| input.animations)?;
    let figure = input.figure?;
    let arrival = walk.path.iter().rposition(|hex| *hex == figure)?;
    let path = &walk.path[..=arrival];
    let elapsed = (input.time - walk.started).max(0.0);
    let step = (elapsed / WALK_STEP_SECONDS) as usize;
    if step + 1 >= path.len() {
        return None;
    }
    let progress = ((elapsed - step as f64 * WALK_STEP_SECONDS) / WALK_STEP_SECONDS) as f32;
    let (from, to) = (hex_center(path[step]), hex_center(path[step + 1]));
    Some(from + (to - from) * progress)
}

/// Respiration de la figurine (C34) : sa hauteur multipliée par ce facteur, pieds posés.
fn breathing(time: f64) -> f32 {
    let phase = std::f64::consts::TAU * time / BREATHING_PERIOD;
    1.0 + BREATHING_AMPLITUDE * phase.sin() as f32
}

/// Place gardée au menu du site dans la zone du plateau, en points : il n'en sort pas.
const SITE_PANEL_ROOM: Vec2 = vec2(260.0, 120.0);

/// Coin haut gauche du menu du site ou d'une confirmation : à droite de la case, à hauteur des
/// figurines (spec 075, C30, C31), gardé dans la zone du plateau.
fn site_anchor(input: &BoardInput<'_>, hex: Hex) -> Pos2 {
    let BoardInput { camera, area, .. } = *input;
    let anchor = camera.to_screen(area, hex_center(hex)) + vec2(0.9, -0.9) * camera.zoom;
    pos2(
        anchor.x.clamp(
            area.left(),
            (area.right() - SITE_PANEL_ROOM.x).max(area.left()),
        ),
        anchor.y.clamp(
            area.top(),
            (area.bottom() - SITE_PANEL_ROOM.y).max(area.top()),
        ),
    )
}

/// Écart entre deux boutons d'action côte à côte, en points.
const BOARD_ACTION_GAP: f32 = 6.0;

/// Les boutons d'action à l'écran (spec 075) : une rangée par case, centrée au-dessus d'elle,
/// gardée dans la zone du plateau (C28) ; deux rangées qui se chevauchent s'écartent à
/// l'horizontale (L3).
fn place_actions(input: &BoardInput<'_>) -> Vec<BoardAction> {
    let BoardInput { camera, area, .. } = *input;
    if input.actions.is_empty() || area.width() <= 0.0 || area.height() <= 0.0 {
        return vec![];
    }
    let radius = BOARD_ACTION_DIAMETER / 2.0;
    let spacing = BOARD_ACTION_DIAMETER + BOARD_ACTION_GAP;
    // Une rangée par case, dans l'ordre des boutons : (case, centre de la rangée, nombre).
    let mut rows: Vec<(Hex, Pos2, usize)> = vec![];
    for button in input.actions {
        match rows.iter_mut().find(|(hex, _, _)| *hex == button.hex) {
            Some(row) => row.2 += 1,
            None => {
                let center = camera.to_screen(area, hex_center(button.hex));
                // Au-dessus du site et des figurines, sans quitter la case à petit zoom.
                let lift = (camera.zoom * 1.1).max(radius + 4.0);
                rows.push((button.hex, center - vec2(0.0, lift), 1));
            }
        }
    }
    let half_width = |count: usize| (count as f32 - 1.0) * spacing / 2.0 + radius;
    let keep_inside = |rows: &mut Vec<(Hex, Pos2, usize)>| {
        for (_, center, count) in rows.iter_mut() {
            let half = half_width(*count);
            center.x = center.x.clamp(
                area.left() + half,
                (area.right() - half).max(area.left() + half),
            );
            center.y = center.y.clamp(
                area.top() + radius,
                (area.bottom() - radius).max(area.top() + radius),
            );
        }
    };
    keep_inside(&mut rows);
    // L3 : quelques passes suffisent pour les quelques boutons d'un plateau.
    for _ in 0..4 {
        for first in 0..rows.len() {
            for second in first + 1..rows.len() {
                let (a, b) = (rows[first], rows[second]);
                if (a.1.y - b.1.y).abs() >= BOARD_ACTION_DIAMETER {
                    continue;
                }
                let needed = half_width(a.2) + half_width(b.2) + BOARD_ACTION_GAP;
                let distance = (a.1.x - b.1.x).abs();
                if distance >= needed {
                    continue;
                }
                let push = (needed - distance) / 2.0;
                // La rangée de gauche (la première à égalité) va à gauche.
                let sign = if a.1.x <= b.1.x { 1.0 } else { -1.0 };
                rows[first].1.x -= sign * push;
                rows[second].1.x += sign * push;
            }
        }
    }
    keep_inside(&mut rows);
    let mut placed: Vec<usize> = vec![0; rows.len()];
    input
        .actions
        .iter()
        .map(|button| {
            let row = rows
                .iter()
                .position(|(hex, _, _)| *hex == button.hex)
                .unwrap_or_default();
            let (_, center, count) = rows[row];
            let place = placed[row];
            placed[row] += 1;
            let offset = (place as f32 - (count as f32 - 1.0) / 2.0) * spacing;
            BoardAction {
                button: button.clone(),
                center: center + vec2(offset, 0.0),
                radius,
            }
        })
        .collect()
}

/// Tranches des cases de tuile dont la voisine sud-est ou sud-ouest n'a pas de tuile (spec 075,
/// C11) : sous l'arête de ce côté, la profondeur sans écrasement.
fn slabs(input: &BoardInput<'_>, hexes: &[HexDrawing]) -> Vec<Slab> {
    let depth = vec2(0.0, SLAB_DEPTH * input.camera.zoom);
    hexes
        .iter()
        .flat_map(|drawing| {
            [(Direction::SE, 2), (Direction::SW, 3)]
                .into_iter()
                .filter(|(side, _)| input.map.space(drawing.hex.neighbor(*side)).is_none())
                .map(|(side, first)| {
                    let (a, b) = (drawing.corners[first], drawing.corners[first + 1]);
                    Slab {
                        hex: drawing.hex,
                        side,
                        corners: [a, b, b + depth, a + depth],
                        color: darker(drawing.fallback),
                    }
                })
        })
        .collect()
}

/// Couleur de la tranche : celle du terrain, assombrie.
fn darker(color: Color32) -> Color32 {
    let [red, green, blue, alpha] = color.to_array();
    let shade = |channel: u8| (u16::from(channel) * 11 / 20) as u8;
    Color32::from_rgba_premultiplied(shade(red), shade(green), shade(blue), alpha)
}

/// Index de l'action légale qui explore cet emplacement, s'il y en a une (spec 015).
fn explore_index(input: &BoardInput<'_>, position: TilePosition) -> Option<usize> {
    input
        .movement?
        .legal_actions
        .iter()
        .position(|action| *action == Action::Explore { position })
}

/// Emplacements libres touchant la figurine où la prochaine tuile peut aller (spec 015, Q5).
fn explore_ghosts(
    input: &BoardInput<'_>,
    on_screen: &impl Fn(Hex) -> [Pos2; 6],
) -> Vec<ExploreGhost> {
    let (Some(movement), Some(figure)) = (input.movement, input.figure) else {
        return vec![];
    };
    if movement.movement_ended {
        return vec![];
    }
    let BoardInput { camera, area, .. } = *input;
    free_positions_touching(input.map, figure)
        .into_iter()
        .filter(|position| {
            placement(input.map, input.shape, *position, movement.next_tile) == Placement::Allowed
        })
        .map(|position| {
            let tile_hexes = position.hexes();
            let center = camera.to_screen(area, hex_center(position.center()));
            ExploreGhost {
                position,
                reachable: explore_index(input, position).is_some(),
                hexes: tile_hexes.iter().map(|hex| on_screen(*hex)).collect(),
                segments: outline(&tile_hexes, on_screen),
                center,
                badge: (camera.zoom >= OVERLAY_MIN_ZOOM).then(|| {
                    (
                        center + vec2(0.45, 0.35) * camera.zoom,
                        COST_BADGE_RADIUS * camera.zoom,
                    )
                }),
            }
        })
        .collect()
}

/// Vrai si la case est dans un emplacement libre voisin d'une tuile posée, en partie
/// seulement : hors partie, le survol d'une case vide reste neutre (spec 011).
fn is_free_position(input: &BoardInput<'_>, hex: Hex) -> bool {
    input.movement.is_some()
        && input
            .map
            .free_neighbor_positions()
            .contains(&TilePosition::containing(hex))
}

/// « Emplacement libre », et s'il touche la figurine, ce qu'une exploration donnerait (spec 015).
fn free_position_lines(input: &BoardInput<'_>, hex: Hex, catalog: &Catalog) -> Vec<String> {
    let position = TilePosition::containing(hex);
    let mut lines = vec![catalog.text("board.hover.free_position")];
    let touches = input
        .figure
        .is_some_and(|figure| free_positions_touching(input.map, figure).contains(&position));
    if let (Some(movement), true) = (input.movement, touches) {
        lines.push(catalog.format(
            "board.hover.explore_cost",
            &[("cost", &EXPLORE_COST.to_string())],
        ));
        let kind = match movement.next_tile {
            NextTile::Countryside => "countryside",
            NextTile::Core => "core",
            NextTile::RemovedCountryside | NextTile::RemovedCore => "removed",
            NextTile::None => "",
        };
        match placement(input.map, input.shape, position, movement.next_tile) {
            Placement::Allowed => {
                let missing = u32::from(EXPLORE_COST).saturating_sub(movement.move_points);
                if movement.movement_ended {
                    lines.push(catalog.text("board.hover.movement_ended"));
                } else if explore_index(input, position).is_some() {
                    lines.push(catalog.text(&format!("board.hover.click_to_explore.{kind}")));
                } else if missing > 0 {
                    lines.push(catalog.format(
                        "board.hover.missing_points",
                        &[("missing", &missing.to_string())],
                    ));
                }
            }
            Placement::TooFewNeighbors => {
                lines.push(catalog.text(&format!("board.hover.explore_limit.{kind}")));
            }
            Placement::NothingToReveal => lines.push(catalog.text("board.hover.nothing_to_reveal")),
            // Spec 064, C32 : MKUE p. 14, *Wedge Map*.
            Placement::CoreOnCoastline => lines.push(catalog.text("board.hover.core_on_coastline")),
            // Occupé ou hors de la forme : déjà exclu par l'appelant.
            Placement::Occupied | Placement::BehindCoastline => {}
        }
    }
    lines
}

/// Index de l'action légale qui déplace la figurine sur `hex`, s'il y en a une.
fn move_index(input: &BoardInput<'_>, hex: Hex) -> Option<usize> {
    input
        .movement?
        .legal_actions
        .iter()
        .position(|action| *action == Action::MoveHero { to: hex })
}

/// Index de l'action légale qui suit le trajet vers `hex`, case lointaine (spec 036).
fn follow_index(input: &BoardInput<'_>, hex: Hex) -> Option<usize> {
    input
        .movement?
        .legal_actions
        .iter()
        .position(|action| *action == Action::FollowRoute { to: hex })
}

/// Trajets retenus depuis la figurine (spec 017), qui évitent les ennemis errants et les pas qui
/// les provoqueraient (spec 030, C19) ; aucun hors partie.
fn reachable_routes(input: &BoardInput<'_>) -> Vec<Route> {
    match (input.movement, input.figure) {
        (Some(movement), Some(figure)) => routes_ending_in(
            input.map,
            figure,
            movement.move_rules,
            &enemy_hexes(input),
            &assault_targets(input),
            &conquered_sites(input),
        ),
        _ => vec![],
    }
}

/// Jetons à marquer d'un « ! » : ceux que provoquerait un pas vers une voisine marquée, ou, après
/// une provocation, les ennemis provoqués (spec 030, C24 bis et C26).
fn enemy_alerts(input: &BoardInput<'_>, marks: &[MoveMark]) -> Vec<EnemyAlert> {
    let (Some(movement), Some(figure)) = (input.movement, input.figure) else {
        return vec![];
    };
    let mut hexes: Vec<Hex> = if movement.movement_ended {
        movement.enemies_to_fight.to_vec()
    } else {
        let enemies = enemy_hexes(input);
        marks
            .iter()
            .filter(|mark| mark.provokes)
            .flat_map(|mark| provoked(figure, mark.hex, &enemies))
            .collect()
    };
    hexes.sort();
    hexes.dedup();
    let BoardInput { camera, area, .. } = *input;
    hexes
        .into_iter()
        .filter(|hex| input.enemy_tokens.contains_key(hex))
        .map(|hex| {
            let center = camera.to_screen(area, hex_center(hex));
            // En haut à droite du jeton, dessiné dans la moitié haute de la case, ou à gauche du
            // site pour un défenseur (spec 043).
            let offset = if is_fortified_site(input, hex) {
                vec2(-0.3, -0.45)
            } else {
                vec2(0.4, -0.85)
            };
            EnemyAlert {
                hex,
                center: center + offset * camera.zoom,
                radius: COST_BADGE_RADIUS * camera.zoom * 0.8,
            }
        })
        .collect()
}

/// Le premier jeton de cette case : l'ennemi errant ou le défenseur, seul sur sa case.
fn first_token(input: &BoardInput<'_>, hex: Hex) -> Option<EnemyToken> {
    input
        .enemy_tokens
        .get(&hex)
        .and_then(|tokens| tokens.first())
        .copied()
}

/// Les cases des ennemis errants : les jetons sans les défenseurs (spec 043) ni ceux restés sur
/// un site d'aventure (spec 044).
fn enemy_hexes(input: &BoardInput<'_>) -> Vec<Hex> {
    input
        .enemy_tokens
        .keys()
        .copied()
        .filter(|&hex| !holds_site_tokens(input, hex))
        .collect()
}

/// Le site d'aventure de cette case (ruines à part, spec 045).
fn adventure_site(input: &BoardInput<'_>, hex: Hex) -> Option<Site> {
    match input.map.space(hex).and_then(|space| space.feature) {
        Some(Feature::Site(
            site @ (Site::Dungeon | Site::Tomb | Site::MonsterDen | Site::SpawningGrounds),
        )) => Some(site),
        _ => None,
    }
}

/// Les jetons restés sur le site d'aventure de cette case (spec 044).
fn adventure_tokens(input: &BoardInput<'_>, hex: Hex) -> Vec<EnemyToken> {
    if adventure_site(input, hex).is_none() && !is_ruins(input, hex) {
        return Vec::new();
    }
    input.enemy_tokens.get(&hex).cloned().unwrap_or_default()
}

/// Une ruine, conquise ou non (spec 045).
fn is_ruins(input: &BoardInput<'_>, hex: Hex) -> bool {
    input.map.space(hex).and_then(|space| space.feature) == Some(Feature::Site(Site::Ruins))
}

/// Les jetons de ruines des cases dessinées : un hexagone à gauche du site, ses pastilles face
/// visible (spec 045, C31 ; Q5).
fn ruin_marks(input: &BoardInput<'_>, hexes: &[HexDrawing]) -> Vec<RuinTokenMark> {
    let camera = input.camera;
    hexes
        .iter()
        .filter_map(|drawing| {
            let token = input.ruins.get(&drawing.hex)?;
            // Posé à plat devant le site, à gauche (spec 075) : le sol l'écrase, le site ne le
            // cache pas.
            let center = camera.to_screen(input.area, hex_center(drawing.hex) + RUIN_TOKEN_FOOT);
            let pips = match token.map(|token| token.definition()) {
                None => Vec::new(),
                Some(definition) => match definition.altar() {
                    Some(color) => vec![RuinPip::Mana(color); 3],
                    None => definition
                        .enemies()
                        .iter()
                        .map(|&color| RuinPip::Enemy(color))
                        .collect(),
                },
            };
            Some(RuinTokenMark {
                hex: drawing.hex,
                center,
                radius: RUIN_TOKEN_RADIUS * camera.zoom,
                face_up: token.is_some(),
                pips,
            })
        })
        .collect()
}

/// Une ruine non conquise (spec 045, C32) : les ennemis qui y attendent, puis son jeton (autel,
/// ennemis et récompense) ou « face cachée ».
fn ruin_lines(input: &BoardInput<'_>, hex: Hex, catalog: &Catalog) -> Vec<String> {
    let Some(token) = input.ruins.get(&hex) else {
        return Vec::new();
    };
    let mut lines = Vec::new();
    let waiting = adventure_tokens(input, hex);
    if !waiting.is_empty() {
        lines.push(catalog.format(
            "board.hover.adventure.enemies_waiting",
            &[("enemies", &token_names(waiting, catalog))],
        ));
    }
    lines.push(match token {
        None => catalog.text("board.hover.ruins.face_down"),
        Some(token) => match token.definition().altar() {
            Some(color) => catalog.format(
                "board.hover.ruins.altar",
                &[("color", &catalog.text(mana_key(color)))],
            ),
            None => catalog.format(
                "board.hover.ruins.enemies",
                &[
                    (
                        "groups",
                        &enemy_groups_text(token.definition().enemies(), catalog),
                    ),
                    ("reward", &token_reward_text(*token, catalog)),
                ],
            ),
        },
    });
    lines
}

/// Les jetons de cette case appartiennent à son site (défenseurs, ennemis d'un site d'aventure) :
/// ce ne sont pas des ennemis errants (specs 043 et 044).
fn holds_site_tokens(input: &BoardInput<'_>, hex: Hex) -> bool {
    is_fortified_site(input, hex)
        || matches!(
            input.map.space(hex).and_then(|space| space.feature),
            Some(Feature::Site(site)) if site.is_adventure()
        )
}

/// Forteresse ou tour de mage, conquise ou non (spec 043) ; une cité en jeu (spec 062). Une cité
/// fermée (Première reconnaissance) n'est pas en jeu : elle reste une case fermée.
fn is_fortified_site(input: &BoardInput<'_>, hex: Hex) -> bool {
    match input.map.space(hex).and_then(|space| space.feature) {
        Some(Feature::Site(Site::Keep | Site::MageTower)) => true,
        Some(Feature::Site(Site::City(_))) => input.cities.contains_key(&hex),
        _ => false,
    }
}

/// Forteresse ou tour de mage non conquise, ou cité en jeu non conquise : y entrer est un assaut
/// (specs 043 et 062).
fn is_assault_target(input: &BoardInput<'_>, hex: Hex) -> bool {
    match input.cities.get(&hex) {
        Some(city) => !city.is_conquered(),
        None => is_fortified_site(input, hex) && !input.shields.contains_key(&hex),
    }
}

/// Site conquis : un Bouclier sur une forteresse ou une tour de mage, ou une cité conquise.
fn is_conquered_site(input: &BoardInput<'_>, hex: Hex) -> bool {
    match input.cities.get(&hex) {
        Some(city) => city.is_conquered(),
        None => is_fortified_site(input, hex) && input.shields.contains_key(&hex),
    }
}

fn assault_targets(input: &BoardInput<'_>) -> Vec<Hex> {
    input
        .map
        .tiles()
        .flat_map(|(position, _)| position.hexes())
        .filter(|&hex| is_assault_target(input, hex))
        .collect()
}

fn conquered_sites(input: &BoardInput<'_>) -> Vec<Hex> {
    input
        .shields
        .keys()
        .chain(input.cities.keys())
        .copied()
        .filter(|&hex| is_conquered_site(input, hex))
        .collect()
}

/// Coût de la case : une forteresse ou une tour de mage coûte son terrain (assaut, ou case
/// ordinaire une fois conquise, spec 043) ; une cité en jeu coûte 2, une cité fermée reste fermée
/// (spec 062).
fn space_cost(
    input: &BoardInput<'_>,
    hex: Hex,
    space: SpaceContent,
    rules: &MoveRules,
) -> MoveCost {
    match move_cost(space, rules) {
        MoveCost::Fortified(cost) if is_fortified_site(input, hex) => MoveCost::Points(cost),
        cost => cost,
    }
}

/// Jetons que provoquerait un pas de la figurine vers `hex` (spec 030).
fn provoked_tokens(input: &BoardInput<'_>, hex: Hex) -> Vec<EnemyToken> {
    let Some(figure) = input.figure else {
        return vec![];
    };
    provoked(figure, hex, &enemy_hexes(input))
        .into_iter()
        .filter_map(|enemy| first_token(input, enemy))
        .collect()
}

/// Trajet qu'on peut payer maintenant vers `hex`, s'il y en a un : seulement avec une décision en
/// attente, comme la mise en évidence des voisines (spec 014).
fn affordable_route<'r>(
    input: &BoardInput<'_>,
    routes: &'r [Route],
    hex: Hex,
) -> Option<&'r Route> {
    let movement = input.movement?;
    if movement.legal_actions.is_empty() {
        return None;
    }
    routes
        .iter()
        .find(|route| route.to == hex && route.cost <= movement.move_points)
}

/// Centres de la figurine et des cases du trajet vers `hex`, à l'écran (spec 017).
fn route_line(input: &BoardInput<'_>, routes: &[Route], hex: Hex) -> Vec<Pos2> {
    let (Some(figure), Some(route)) = (input.figure, affordable_route(input, routes, hex)) else {
        return vec![];
    };
    let BoardInput { camera, area, .. } = *input;
    std::iter::once(figure)
        .chain(route.steps.iter().copied())
        .map(|hex| camera.to_screen(area, hex_center(hex)))
        .collect()
}

/// Jetons de coût : voisines accessibles d'abord, mises en évidence si jouables (spec 014), puis
/// cases lointaines atteignables, avec le coût de leur trajet (spec 017).
fn move_marks(
    input: &BoardInput<'_>,
    routes: &[Route],
    on_screen: &impl Fn(Hex) -> [Pos2; 6],
) -> Vec<MoveMark> {
    let (Some(movement), Some(figure)) = (input.movement, input.figure) else {
        return vec![];
    };
    // Après une provocation, plus aucun pas n'est possible ce tour (spec 030, C26).
    if movement.movement_ended {
        return vec![];
    }
    let BoardInput { camera, area, .. } = *input;
    let mark = |hex: Hex, cost: u8, reachable: bool, neighbor: bool| {
        let assault = is_assault_target(input, hex);
        let center = camera.to_screen(area, hex_center(hex));
        MoveMark {
            hex,
            cost,
            reachable,
            neighbor,
            provokes: neighbor && !provoked_tokens(input, hex).is_empty(),
            assault,
            corners: on_screen(hex),
            // En bas à droite, pour ne cacher ni le site ni la figurine.
            badge_center: center + vec2(0.45, 0.35) * camera.zoom,
            badge_radius: COST_BADGE_RADIUS * camera.zoom,
        }
    };
    let neighbors = figure.neighbors().into_iter().filter_map(|hex| {
        // Case occupée par un ennemi errant : jamais de marque (spec 030, C25).
        if input.enemy_tokens.contains_key(&hex) && !holds_site_tokens(input, hex) {
            return None;
        }
        let space = input.map.space(hex)?;
        let MoveCost::Points(cost) = space_cost(input, hex, space, movement.move_rules) else {
            return None;
        };
        Some(mark(hex, cost, move_index(input, hex).is_some(), true))
    });
    // Les trajets sont déjà dans l'ordre : coût, nombre de cases, directions.
    let far = routes
        .iter()
        .filter(|route| route.steps.len() > 1)
        .filter_map(|route| affordable_route(input, routes, route.to))
        .map(|route| {
            mark(
                route.to,
                u8::try_from(route.cost).unwrap_or(u8::MAX),
                follow_index(input, route.to).is_some(),
                false,
            )
        });
    neighbors.chain(far).collect()
}

/// Vrai si la case est dans un emplacement sans tuile que la forme ne permet pas.
fn is_forbidden(input: &BoardInput<'_>, hex: Hex) -> bool {
    let position = TilePosition::containing(hex);
    !input.shape.allows(position) && input.map.space(hex).is_none()
}

/// Cases dont le centre est proche du rectangle de la zone, ligne par ligne (une de marge de
/// chaque côté) ; le filtre exact se fait ensuite sur les sommets.
fn visible_hexes(camera: BoardCamera, area: Rect) -> Vec<Hex> {
    if area.width() <= 0.0 || area.height() <= 0.0 {
        return vec![];
    }
    let corners = [
        area.left_top(),
        area.right_top(),
        area.left_bottom(),
        area.right_bottom(),
    ]
    .map(|corner| camera.to_board(area, corner));
    let bounds = Rect::from_points(&corners);
    let sqrt_3 = 3.0_f32.sqrt();
    // Bornes entières d'un intervalle de flottants finis, élargies d'une case.
    let rows = (bounds.top() / 1.5).floor() as i32 - 1..=(bounds.bottom() / 1.5).ceil() as i32 + 1;
    let mut hexes = vec![];
    for r in rows {
        let shift = r as f32 / 2.0;
        let first = (bounds.left() / sqrt_3 - shift).floor() as i32 - 1;
        let last = (bounds.right() / sqrt_3 - shift).ceil() as i32 + 1;
        hexes.extend((first..=last).map(|q| Hex::new(q, r)));
    }
    hexes
}

/// Arêtes des sept cases qui ne bordent pas une autre case de la tuile.
fn outline(tile_hexes: &[Hex; 7], on_screen: &impl Fn(Hex) -> [Pos2; 6]) -> Vec<[Pos2; 2]> {
    let mut segments = vec![];
    for hex in tile_hexes {
        let corners = on_screen(*hex);
        for (edge, direction) in EDGE_DIRECTIONS.into_iter().enumerate() {
            if !tile_hexes.contains(&hex.neighbor(direction)) {
                segments.push([corners[edge], corners[(edge + 1) % 6]]);
            }
        }
    }
    segments
}

fn hover_lines(
    input: &BoardInput<'_>,
    routes: &[Route],
    hex: Hex,
    content: SpaceContent,
    catalog: &Catalog,
) -> Vec<String> {
    let position = TilePosition::containing(hex);
    let mut lines = vec![];
    if let Some((_, tile)) = input.map.tiles().find(|(placed, _)| *placed == position) {
        lines.push(tile_name(tile, catalog));
    }
    lines.push(catalog.text(terrain_key(content.terrain)));
    // Un défenseur n'est pas un ennemi errant : le site reste nommé (spec 043).
    let token = first_token(input, hex);
    let rampaging = token.filter(|_| !holds_site_tokens(input, hex));
    if let Some(token) = rampaging {
        // Le jeton remplace le symbole imprimé (spec 030, C23).
        lines.extend(token_lines(token, catalog));
    } else if let Some(city) = input.cities.get(&hex) {
        // Spec 062, Q3 : tout ce qui concerne une cité se lit dans son infobulle (C37).
        lines.extend(city_tooltip_lines(input, hex, city, catalog));
    } else if let Some(feature) = content
        .feature
        .filter(|feature| printed_feature_shown(*feature))
    {
        let marked = input.shields.contains_key(&hex);
        lines.push(match feature {
            // Spec 045, Q6 : une ruine marquée d'un Bouclier est conquise (drapeau à l'écran).
            Feature::Site(Site::Ruins) if marked => catalog.text("site.conquered.ruins"),
            // Spec 044 : un site d'aventure marqué d'un Bouclier est conquis.
            Feature::Site(site) if marked && adventure_site(input, hex).is_some() => catalog
                .format(
                    "site.conquered.adventure",
                    &[("site", &catalog.text(feature_key(Feature::Site(site))))],
                ),
            // Spec 041 : un monastère marqué d'un Bouclier est incendié.
            Feature::Site(Site::Monastery) if marked => catalog.text("site.monastery.burned"),
            Feature::Site(Site::Keep) if marked => catalog.text("site.conquered.keep"),
            Feature::Site(Site::MageTower) if marked => catalog.text("site.conquered.mage_tower"),
            _ => catalog.text(feature_key(feature)),
        });
        if feature == Feature::Site(Site::Ruins) {
            lines.extend(ruin_lines(input, hex, catalog));
        } else if let Some(site) = adventure_site(input, hex) {
            lines.extend(adventure_lines(input, hex, site, marked, catalog));
        } else if feature == Feature::Site(Site::MagicalGlade) {
            lines.push(catalog.text("board.hover.glade"));
        } else if let Some(color) = face_down_color(input, hex) {
            lines.push(catalog.format(
                "enemy.face_down",
                &[("group", &catalog.text(color.label_key()))],
            ));
        } else if let Some(token) = token {
            lines.extend(token_lines(token, catalog));
        }
    }
    // MKUE p. 11, 4.b : finir le tour sur une mine rapporte un cristal, sauf couleur pleine.
    if let (Some(inventory), Some(color)) = (input.inventory, mine_color(input.map, hex)) {
        lines.push(if inventory.is_full(color) {
            catalog.text("board.hover.mine.full")
        } else {
            catalog.format(
                "board.hover.mine.gain",
                &[("color", &catalog.text(mana_key(color)))],
            )
        });
    }
    if input.figure == Some(hex) {
        lines.push(catalog.text("board.hover.hero"));
    }
    if rampaging.is_some() {
        lines.push(catalog.text("board.hover.enemy_space"));
    } else if let Some(movement) = input.movement {
        lines.extend(move_lines(input, routes, movement, hex, content, catalog));
    }
    lines
}

/// L'infobulle d'une cité en jeu (spec 062, C37) : son nom et son niveau, ses défenseurs (couleurs
/// face cachée, noms retournés) et le bonus qu'ils tirent de la cité, les Boucliers du joueur, sa
/// conquête, ce qu'on y achète et la limite de main.
fn city_tooltip_lines(
    input: &BoardInput<'_>,
    hex: Hex,
    city: &CityState,
    catalog: &Catalog,
) -> Vec<String> {
    let color = city.color;
    let mut lines = vec![catalog.format(
        "board.hover.city.level",
        &[
            (
                "city",
                &catalog.text(feature_key(Feature::Site(Site::City(color)))),
            ),
            ("level", &city.level.to_string()),
        ],
    )];
    let face_down = input
        .face_down_defenders
        .get(&hex)
        .cloned()
        .unwrap_or_default();
    if !face_down.is_empty() {
        lines.push(catalog.format(
            "board.hover.city.face_down",
            &[("groups", &enemy_groups_text(&face_down, catalog))],
        ));
    }
    let defenders = input.enemy_tokens.get(&hex).cloned().unwrap_or_default();
    if !defenders.is_empty() {
        lines.push(catalog.format(
            "board.hover.city.defenders",
            &[("enemies", &token_names(defenders.iter().copied(), catalog))],
        ));
    }
    if !city.is_conquered() {
        lines.push(catalog.text(match color {
            BasicColor::White => "board.hover.city.bonus_white",
            BasicColor::Blue => "board.hover.city.bonus_blue",
            BasicColor::Red => "board.hover.city.bonus_red",
            BasicColor::Green => "board.hover.city.bonus_green",
        }));
    }
    // Solo : tous les Boucliers sont ceux du joueur, et il est le Seigneur d'une cité conquise.
    if !city.shields.is_empty() {
        lines.push(catalog.format(
            "board.hover.city.shields",
            &[("count", &city.shields.len().to_string())],
        ));
    }
    if city.is_conquered() {
        lines.push(catalog.text("board.hover.city.leader"));
    }
    lines.push(catalog.text(match color {
        BasicColor::White => "board.hover.city.offer_white",
        BasicColor::Blue => "board.hover.city.offer_blue",
        BasicColor::Red => "board.hover.city.offer_red",
        BasicColor::Green => "board.hover.city.offer_green",
    }));
    lines.push(catalog.text("board.hover.city.hand_limit"));
    lines
}

/// Un site d'aventure (spec 044, C43) : les ennemis qui y attendent, puis ce qu'y entrer
/// rapporte, sauf un antre ou un Sombre Terrier conquis, où l'on n'entre plus.
fn adventure_lines(
    input: &BoardInput<'_>,
    hex: Hex,
    site: Site,
    conquered: bool,
    catalog: &Catalog,
) -> Vec<String> {
    let mut lines = Vec::new();
    let waiting = adventure_tokens(input, hex);
    if !waiting.is_empty() {
        lines.push(catalog.format(
            "board.hover.adventure.enemies_waiting",
            &[("enemies", &token_names(waiting, catalog))],
        ));
    }
    if !conquered || matches!(site, Site::Dungeon | Site::Tomb) {
        lines.push(catalog.format(
            "board.hover.adventure.fight",
            &[("reward", &adventure_reward(site, conquered, catalog))],
        ));
    }
    lines
}

/// Coût de la case, et pour une case adjacente ce qu'un clic ferait (spec 014).
fn move_lines(
    input: &BoardInput<'_>,
    routes: &[Route],
    movement: BoardMovement<'_>,
    hex: Hex,
    content: SpaceContent,
    catalog: &Catalog,
) -> Vec<String> {
    let cost = match space_cost(input, hex, content, movement.move_rules) {
        MoveCost::Points(cost) => cost,
        MoveCost::Inaccessible => return vec![catalog.text("board.hover.inaccessible")],
        MoveCost::Fortified(_) => return vec![catalog.text("board.hover.fortified")],
    };
    let mut lines = Vec::new();
    // Spec 043, C36 : l'assaut d'abord, puis la fortification du défenseur s'il en a un.
    if is_assault_target(input, hex) && !movement.movement_ended && input.figure != Some(hex) {
        lines.push(catalog.text("board.hover.assault"));
        // Spec 062 : une cité a plusieurs défenseurs ; chaque sorte de fortification a sa ligne.
        let tokens = input.enemy_tokens.get(&hex).map_or(&[][..], Vec::as_slice);
        let face_down = input
            .face_down_defenders
            .get(&hex)
            .map_or(&[][..], Vec::as_slice);
        let printed = |token: &EnemyToken| {
            token
                .enemy()
                .definition()
                .abilities
                .contains(&EnemyAbility::Fortified)
        };
        let twice = face_down.is_empty() && tokens.iter().any(printed);
        let once = !face_down.is_empty() || tokens.iter().any(|token| !printed(token));
        if twice {
            lines.push(catalog.text("board.hover.defender_fortified_twice"));
        }
        if once {
            lines.push(catalog.text("board.hover.defender_fortified"));
        }
    }
    lines.push(catalog.format("board.hover.move_cost", &[("cost", &cost.to_string())]));
    // Spec 052, C27 : un lac franchi n'est pas une case sécurisée. Une cible d'assaut a déjà son
    // avertissement : sa retraite d'assaut passe avant.
    if !is_assault_target(input, hex) && !is_safe_space(input.map, hex, &conquered_sites(input)) {
        lines.push(catalog.text("board.hover.unsafe_space"));
    }
    // Plus de pas ce tour (spec 030, C26 bis) : ne pas laisser croire qu'il manque des points.
    if movement.movement_ended {
        lines.push(catalog.text("board.hover.movement_ended"));
        return lines;
    }
    if input.figure.is_some_and(|figure| figure.is_adjacent(hex)) {
        let provoked = provoked_tokens(input, hex);
        if !provoked.is_empty() {
            lines.push(catalog.format(
                "board.hover.provokes",
                &[("enemies", &token_names(provoked, catalog))],
            ));
        }
        let missing = u32::from(cost).saturating_sub(movement.move_points);
        if move_index(input, hex).is_some() {
            lines.push(catalog.text("board.hover.click_to_move"));
        } else if missing > 0 {
            lines.push(catalog.format(
                "board.hover.missing_points",
                &[("missing", &missing.to_string())],
            ));
        }
    } else if let Some(route) = routes.iter().find(|route| route.to == hex) {
        // Case lointaine (spec 017) : son trajet si on peut le payer, sinon ce qui manque.
        if affordable_route(input, routes, hex).is_some() {
            lines.push(catalog.format(
                "board.hover.route",
                &[
                    ("steps", &route.steps.len().to_string()),
                    ("cost", &route.cost.to_string()),
                ],
            ));
            if follow_index(input, hex).is_some() {
                lines.push(catalog.text("board.hover.click_to_follow"));
            }
        } else if route.cost > movement.move_points {
            let missing = route.cost - movement.move_points;
            lines.push(catalog.format(
                "board.hover.missing_points",
                &[("missing", &missing.to_string())],
            ));
        }
    }
    lines
}

fn tile_name(tile: TileId, catalog: &Catalog) -> String {
    match tile {
        TileId::StartingA => catalog.text("tile.starting_a"),
        TileId::StartingB => catalog.text("tile.starting_b"),
        TileId::Countryside(number) => {
            catalog.format("tile.countryside", &[("number", &number.to_string())])
        }
        TileId::Core(number) => catalog.format("tile.core", &[("number", &number.to_string())]),
    }
}

pub(crate) fn terrain_key(terrain: Terrain) -> &'static str {
    match terrain {
        Terrain::Plains => "terrain.plains.name",
        Terrain::Hills => "terrain.hills.name",
        Terrain::Forest => "terrain.forest.name",
        Terrain::Wasteland => "terrain.wasteland.name",
        Terrain::Desert => "terrain.desert.name",
        Terrain::Swamp => "terrain.swamp.name",
        Terrain::Lake => "terrain.lake.name",
        Terrain::Mountain => "terrain.mountain.name",
        Terrain::Sea => "terrain.sea.name",
    }
}

/// Un symbole d'ennemi errant sans jeton est celui d'un ennemi vaincu : sa case se dessine comme
/// son terrain seul (spec 031, Q5). Un jeton est posé dès que la tuile est révélée (spec 030).
fn printed_feature_shown(feature: Feature) -> bool {
    !matches!(feature, Feature::Rampaging(_))
}

pub(crate) fn feature_key(feature: Feature) -> &'static str {
    match feature {
        Feature::Site(site) => match site {
            Site::Portal => "site.portal.name",
            Site::Village => "site.village.name",
            Site::Monastery => "site.monastery.name",
            Site::MagicalGlade => "site.magical_glade.name",
            Site::Keep => "site.keep.name",
            Site::MageTower => "site.mage_tower.name",
            Site::Dungeon => "site.dungeon.name",
            Site::Tomb => "site.tomb.name",
            Site::MonsterDen => "site.monster_den.name",
            Site::SpawningGrounds => "site.spawning_grounds.name",
            Site::Ruins => "site.ruins.name",
            Site::Mine(BasicColor::Red) => "site.mine.red",
            Site::Mine(BasicColor::Blue) => "site.mine.blue",
            Site::Mine(BasicColor::White) => "site.mine.white",
            Site::Mine(BasicColor::Green) => "site.mine.green",
            Site::City(BasicColor::Red) => "site.city.red",
            Site::City(BasicColor::Blue) => "site.city.blue",
            Site::City(BasicColor::White) => "site.city.white",
            Site::City(BasicColor::Green) => "site.city.green",
        },
        Feature::Rampaging(Rampaging::OrcMarauders) => "enemy.orc_marauders.name",
        Feature::Rampaging(Rampaging::Draconum) => "enemy.draconum.name",
    }
}

/// Nom de la couleur au masculin singulier, partagé avec la Source (spec 020).
fn mana_key(color: BasicColor) -> &'static str {
    match color {
        BasicColor::Red => "mana.red",
        BasicColor::Blue => "mana.blue",
        BasicColor::White => "mana.white",
        BasicColor::Green => "mana.green",
    }
}

/// La couleur du premier défenseur face cachée de cette case (spec 043 ; une cité en a plusieurs,
/// spec 062).
fn face_down_color(input: &BoardInput, hex: Hex) -> Option<EnemyColor> {
    input
        .face_down_defenders
        .get(&hex)
        .and_then(|colors| colors.first())
        .copied()
}
