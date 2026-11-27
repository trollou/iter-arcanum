//! Icônes de l'interface (spec 071) : un SVG de `assets/images/ui/` par dessin de la planche,
//! et son nom traduit, affiché en infobulle.

use std::collections::BTreeSet;

use arcanum_core::{AttackKind, BasicColor, Element, EnemyAbility, Hero, ManaColor, Resistance};

use arcanum_i18n::Catalog;

use crate::ImageId;

/// Un dessin de la planche qui porte un nom (infobulle).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Icon {
    Move,
    Influence,
    Armor,
    Fame,
    /// Épée, arc ou catapulte, à la couleur de l'élément.
    Attack {
        kind: AttackKind,
        element: Element,
    },
    /// Rondache, à la couleur de l'élément.
    Block(Element),
    Resistance(Resistance),
    /// Capacité d'un ennemi (spec #27) : tour, oiseau, tête de démon, goutte, silhouette pétrifiée.
    Ability(EnemyAbility),
    /// Griffe sortant d'un portail : l'attaque d'un Invocateur (spec #27).
    Summon,
    /// Carte couchée, devant les boutons de côté.
    Sideways,
    /// Dos d'une tuile de campagne, devant le nombre de sa pioche (spec 074, C8).
    TileCountryside,
    /// Dos d'une tuile royaume.
    TileCore,
    /// « i » rond bleu : le texte complet d'une carte de la main (spec #11). Hors de la planche
    /// nommée (`all`) : son infobulle est `card.info`.
    Info,
}

impl Icon {
    /// Toutes les icônes nommées de la planche (32).
    pub fn all() -> Vec<Icon> {
        let fixed = [Icon::Move, Icon::Influence, Icon::Armor, Icon::Fame];
        let attacks = [AttackKind::Melee, AttackKind::Ranged, AttackKind::Siege]
            .into_iter()
            .flat_map(|kind| Element::ALL.map(|element| Icon::Attack { kind, element }));
        let blocks = Element::ALL.map(Icon::Block);
        let resistances = RESISTANCES.map(Icon::Resistance);
        let abilities = ABILITIES.map(Icon::Ability);
        fixed
            .into_iter()
            .chain(attacks)
            .chain(blocks)
            .chain(resistances)
            .chain(abilities)
            .chain([Icon::Summon])
            .chain([Icon::Sideways, Icon::TileCountryside, Icon::TileCore])
            .collect()
    }

    /// Son fichier : `ui/<nom>.svg`.
    pub fn image(self) -> ImageId {
        let stem = match self {
            Icon::Move => "move",
            Icon::Influence => "influence",
            Icon::Armor => "armor",
            Icon::Fame => "fame",
            Icon::Attack { kind, element } => match (kind, element) {
                (AttackKind::Melee, Element::Physical) => "attack",
                (AttackKind::Melee, Element::Fire) => "attack_fire",
                (AttackKind::Melee, Element::Ice) => "attack_ice",
                (AttackKind::Melee, Element::ColdFire) => "attack_cold_fire",
                (AttackKind::Ranged, Element::Physical) => "ranged_attack",
                (AttackKind::Ranged, Element::Fire) => "ranged_attack_fire",
                (AttackKind::Ranged, Element::Ice) => "ranged_attack_ice",
                (AttackKind::Ranged, Element::ColdFire) => "ranged_attack_cold_fire",
                (AttackKind::Siege, Element::Physical) => "siege_attack",
                (AttackKind::Siege, Element::Fire) => "siege_attack_fire",
                (AttackKind::Siege, Element::Ice) => "siege_attack_ice",
                (AttackKind::Siege, Element::ColdFire) => "siege_attack_cold_fire",
            },
            Icon::Block(Element::Physical) => "block",
            Icon::Block(Element::Fire) => "block_fire",
            Icon::Block(Element::Ice) => "block_ice",
            Icon::Block(Element::ColdFire) => "block_cold_fire",
            Icon::Resistance(Resistance::Physical) => "resistance_physical",
            Icon::Resistance(Resistance::Fire) => "resistance_fire",
            Icon::Resistance(Resistance::Ice) => "resistance_ice",
            Icon::Ability(EnemyAbility::Fortified) => "ability_fortified",
            Icon::Ability(EnemyAbility::Swift) => "ability_swift",
            Icon::Ability(EnemyAbility::Brutal) => "ability_brutal",
            Icon::Ability(EnemyAbility::Poison) => "ability_poison",
            Icon::Ability(EnemyAbility::Paralyze) => "ability_paralyze",
            Icon::Summon => "summon",
            Icon::Sideways => "sideways",
            Icon::TileCountryside => "tile_countryside",
            Icon::TileCore => "tile_core",
            Icon::Info => "info",
        };
        ImageId::new("ui", stem)
    }

    /// Clé de son nom, l'infobulle ; la carte couchée garde « Jouer de côté » (`play.sideways`).
    pub fn name_key(self) -> &'static str {
        match self {
            Icon::Move => "icon.move",
            Icon::Influence => "icon.influence",
            Icon::Armor => "icon.armor",
            Icon::Fame => "icon.fame",
            Icon::Attack { kind, element } => match (kind, element) {
                (AttackKind::Melee, Element::Physical) => "icon.attack.physical",
                (AttackKind::Melee, Element::Fire) => "icon.attack.fire",
                (AttackKind::Melee, Element::Ice) => "icon.attack.ice",
                (AttackKind::Melee, Element::ColdFire) => "icon.attack.cold_fire",
                (AttackKind::Ranged, Element::Physical) => "icon.ranged_attack.physical",
                (AttackKind::Ranged, Element::Fire) => "icon.ranged_attack.fire",
                (AttackKind::Ranged, Element::Ice) => "icon.ranged_attack.ice",
                (AttackKind::Ranged, Element::ColdFire) => "icon.ranged_attack.cold_fire",
                (AttackKind::Siege, Element::Physical) => "icon.siege_attack.physical",
                (AttackKind::Siege, Element::Fire) => "icon.siege_attack.fire",
                (AttackKind::Siege, Element::Ice) => "icon.siege_attack.ice",
                (AttackKind::Siege, Element::ColdFire) => "icon.siege_attack.cold_fire",
            },
            Icon::Block(Element::Physical) => "icon.block.physical",
            Icon::Block(Element::Fire) => "icon.block.fire",
            Icon::Block(Element::Ice) => "icon.block.ice",
            Icon::Block(Element::ColdFire) => "icon.block.cold_fire",
            Icon::Resistance(Resistance::Physical) => "icon.resistance.physical",
            Icon::Resistance(Resistance::Fire) => "icon.resistance.fire",
            Icon::Resistance(Resistance::Ice) => "icon.resistance.ice",
            Icon::Ability(EnemyAbility::Fortified) => "icon.ability.fortified",
            Icon::Ability(EnemyAbility::Swift) => "icon.ability.swift",
            Icon::Ability(EnemyAbility::Brutal) => "icon.ability.brutal",
            Icon::Ability(EnemyAbility::Poison) => "icon.ability.poison",
            Icon::Ability(EnemyAbility::Paralyze) => "icon.ability.paralyze",
            Icon::Summon => "icon.summon",
            Icon::Sideways => "play.sideways",
            Icon::TileCountryside => "icon.tile.countryside",
            Icon::TileCore => "icon.tile.core",
            Icon::Info => "card.info",
        }
    }
}

/// Une valeur du jeu affichée avec son icône : « 3 » ou « 3 / 7 », vide pour une résistance ; son
/// nom en infobulle. Sans fichier d'icône, l'écran écrit le nom puis la valeur (L1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IconValue {
    pub icon: Icon,
    pub value: String,
    pub tooltip: String,
    /// La règle affichée sous le nom dans l'infobulle (spec #27) : capacités, résistances et
    /// attaques élémentaires d'un ennemi ; `None` ailleurs.
    pub rule: Option<String>,
}

impl IconValue {
    /// Le texte de repli : « Mouvement 3 », ou le nom seul sans valeur.
    pub fn fallback_text(&self) -> String {
        if self.value.is_empty() {
            self.tooltip.clone()
        } else {
            format!("{} {}", self.tooltip, self.value)
        }
    }
}

/// Bouton-icône (spec 071, C14) : jouer une carte de côté pour un effet, d'un clic.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IconButton {
    pub icon: Icon,
    /// Le texte de repli, sans fichier d'icône (L1) : « Mouvement 1 », « Attaque 1 ÷2 ».
    pub label: String,
    /// « ÷2 » quand l'effet compte à moitié (spec 032, C16).
    pub mark: Option<String>,
    /// « Jouer de côté : Attaque 1 ÷2 ».
    pub tooltip: String,
    pub command: crate::Command,
}

/// Un morceau d'une ligne de valeurs : une valeur à icône, ou un mot qui n'a pas d'icône
/// (« invocation », spec 071 C9).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValuePiece {
    Icon(IconValue),
    Text(String),
    /// Un mot de couleur de mana, dans sa couleur (spec #68, balise `<red>…</red>`).
    Colored {
        text: String,
        color: ManaColor,
    },
}

/// Les noms des repères d'icône `[[nom]]` d'un libellé, et leur icône : capacités des ennemis
/// (spec #27, Q3 et Q4), puis points des textes de cartes, nommés comme leurs clés `effect.*`
/// (spec #68, Q2).
pub const ICON_MARKERS: [(&str, Icon); 26] = [
    ("fortified", Icon::Ability(EnemyAbility::Fortified)),
    ("swift", Icon::Ability(EnemyAbility::Swift)),
    ("brutal", Icon::Ability(EnemyAbility::Brutal)),
    ("poison", Icon::Ability(EnemyAbility::Poison)),
    ("paralyze", Icon::Ability(EnemyAbility::Paralyze)),
    ("summon", Icon::Summon),
    ("move", Icon::Move),
    ("influence", Icon::Influence),
    ("armor", Icon::Armor),
    ("fame", Icon::Fame),
    (
        "attack",
        attack_marker(AttackKind::Melee, Element::Physical),
    ),
    (
        "fire_attack",
        attack_marker(AttackKind::Melee, Element::Fire),
    ),
    ("ice_attack", attack_marker(AttackKind::Melee, Element::Ice)),
    (
        "cold_fire_attack",
        attack_marker(AttackKind::Melee, Element::ColdFire),
    ),
    (
        "ranged_attack",
        attack_marker(AttackKind::Ranged, Element::Physical),
    ),
    (
        "ranged_fire_attack",
        attack_marker(AttackKind::Ranged, Element::Fire),
    ),
    (
        "ranged_ice_attack",
        attack_marker(AttackKind::Ranged, Element::Ice),
    ),
    (
        "ranged_cold_fire_attack",
        attack_marker(AttackKind::Ranged, Element::ColdFire),
    ),
    (
        "siege_attack",
        attack_marker(AttackKind::Siege, Element::Physical),
    ),
    (
        "siege_fire_attack",
        attack_marker(AttackKind::Siege, Element::Fire),
    ),
    (
        "siege_ice_attack",
        attack_marker(AttackKind::Siege, Element::Ice),
    ),
    (
        "siege_cold_fire_attack",
        attack_marker(AttackKind::Siege, Element::ColdFire),
    ),
    ("block", Icon::Block(Element::Physical)),
    ("fire_block", Icon::Block(Element::Fire)),
    ("ice_block", Icon::Block(Element::Ice)),
    ("cold_fire_block", Icon::Block(Element::ColdFire)),
];

const fn attack_marker(kind: AttackKind, element: Element) -> Icon {
    Icon::Attack { kind, element }
}

/// Les balises de couleur `<nom>…</nom>` d'un libellé (spec #68, règle 8), et leur couleur.
pub const COLOR_TAGS: [(&str, ManaColor); 6] = [
    ("red", ManaColor::Basic(BasicColor::Red)),
    ("blue", ManaColor::Basic(BasicColor::Blue)),
    ("white", ManaColor::Basic(BasicColor::White)),
    ("green", ManaColor::Basic(BasicColor::Green)),
    ("gold", ManaColor::Gold),
    ("black", ManaColor::Black),
];

/// Un morceau d'un texte à repères : du texte, une icône et ce qui la suit (un mot collé, ou
/// rien), ou un mot de couleur.
enum Marked<'a> {
    Text(&'a str),
    Icon { icon: Icon, before_word: bool },
    Colored { text: &'a str, color: ManaColor },
}

/// Un repère connu au début de `text` : `[[nom]]` ou `<nom>mot</nom>`, et ce qui reste après.
fn leading_mark(text: &str) -> Option<(Marked<'_>, &str)> {
    if let Some(after) = text.strip_prefix("[[") {
        let end = after.find("]]")?;
        let &(_, icon) = ICON_MARKERS
            .iter()
            .find(|(name, _)| *name == &after[..end])?;
        let rest = &after[end + 2..];
        let before_word = rest.chars().next().is_some_and(char::is_alphanumeric);
        return Some((Marked::Icon { icon, before_word }, rest));
    }
    let after = text.strip_prefix('<')?;
    let end = after.find('>')?;
    let (name, inner) = (&after[..end], &after[end + 1..]);
    let &(_, color) = COLOR_TAGS.iter().find(|(tag, _)| *tag == name)?;
    let close = format!("</{name}>");
    let stop = inner.find(&close)?;
    Some((
        Marked::Colored {
            text: &inner[..stop],
            color,
        },
        &inner[stop + close.len()..],
    ))
}

/// Découpe un texte en texte, repères et mots de couleur connus ; un repère ou une balise
/// inconnus, ou une balise non fermée, restent du texte.
fn marked(text: &str) -> Vec<Marked<'_>> {
    let mut pieces = Vec::new();
    let mut plain_start = 0;
    let mut cursor = 0;
    while let Some(offset) = text[cursor..].find(['[', '<']) {
        let at = cursor + offset;
        match leading_mark(&text[at..]) {
            Some((mark, rest)) => {
                if at > plain_start {
                    pieces.push(Marked::Text(&text[plain_start..at]));
                }
                pieces.push(mark);
                cursor = text.len() - rest.len();
                plain_start = cursor;
            }
            None => cursor = at + 1,
        }
    }
    if plain_start < text.len() {
        pieces.push(Marked::Text(&text[plain_start..]));
    }
    pieces
}

/// Découpe un texte à repères en morceaux de texte, en icônes, sans valeur, nommées et avec leur
/// règle (spec #27, C15), et en mots de couleur (spec #68, C9) ; un texte sans repère donne un
/// seul morceau.
pub fn icon_pieces(text: &str, catalog: &Catalog) -> Vec<ValuePiece> {
    marked(text)
        .into_iter()
        .map(|piece| match piece {
            Marked::Text(text) => ValuePiece::Text(text.to_string()),
            Marked::Icon { icon, .. } => ValuePiece::Icon(enemy_icon_value(icon, "", catalog)),
            Marked::Colored { text, color } => ValuePiece::Colored {
                text: text.to_string(),
                color,
            },
        })
        .collect()
}

/// Le texte sans repère, en morceaux de couleur (spec #68, Q5) : ceux de `plain_text`, avec la
/// couleur d'un mot balisé ; pour un bouton ou une infobulle mis en forme.
pub fn colored_text(text: &str, catalog: &Catalog) -> Vec<(String, Option<ManaColor>)> {
    let mut pieces: Vec<(String, Option<ManaColor>)> = Vec::new();
    let mut push = |text: &str, color: Option<ManaColor>| match pieces.last_mut() {
        Some((last, last_color)) if *last_color == color => last.push_str(text),
        _ => pieces.push((text.to_string(), color)),
    };
    let mut after_name = false;
    for piece in marked(text) {
        match piece {
            Marked::Text(text) => {
                push(text, None);
                after_name = false;
            }
            Marked::Colored { text, color } => {
                push(text, Some(color));
                after_name = false;
            }
            Marked::Icon {
                before_word: true, ..
            } => after_name = false,
            Marked::Icon { icon, .. } => {
                if after_name {
                    push(" ", None);
                }
                push(&catalog.text(icon.name_key()), None);
                after_name = true;
            }
        }
    }
    pieces
}

/// Le texte sans repère ni balise (spec #27, C15 et C21 ; spec #68, C10) : un repère suivi d'un
/// mot disparaît, un repère seul devient le nom de son icône (« Vif »), un mot balisé reste sans
/// sa balise. Deux repères seuls à la suite (« doublement fortifié ») donnent deux noms séparés
/// d'une espace.
pub fn plain_text(text: &str, catalog: &Catalog) -> String {
    colored_text(text, catalog)
        .into_iter()
        .map(|(text, _)| text)
        .collect()
}

/// Une valeur à icône, son nom traduit en infobulle.
pub fn icon_value(icon: Icon, value: impl Into<String>, catalog: &Catalog) -> IconValue {
    IconValue {
        icon,
        value: value.into(),
        tooltip: catalog.text(icon.name_key()),
        rule: None,
    }
}

/// Une valeur à icône d'un ennemi (spec #27, C3 à C11) : son nom, puis sa règle quand l'icône en
/// a une (capacité, invocation, résistance, attaque élémentaire).
pub fn enemy_icon_value(icon: Icon, value: impl Into<String>, catalog: &Catalog) -> IconValue {
    IconValue {
        rule: enemy_rule_key(icon).map(|key| catalog.text(key)),
        ..icon_value(icon, value, catalog)
    }
}

/// La clé de la règle d'une icône d'ennemi ; `None` pour une icône sans règle (attaque physique,
/// Armure, Gloire…).
fn enemy_rule_key(icon: Icon) -> Option<&'static str> {
    Some(match icon {
        Icon::Ability(EnemyAbility::Fortified) => "icon.rule.fortified",
        Icon::Ability(EnemyAbility::Swift) => "icon.rule.swift",
        Icon::Ability(EnemyAbility::Brutal) => "icon.rule.brutal",
        Icon::Ability(EnemyAbility::Poison) => "icon.rule.poison",
        Icon::Ability(EnemyAbility::Paralyze) => "icon.rule.paralyze",
        Icon::Summon => "icon.rule.summon",
        Icon::Resistance(Resistance::Physical) => "icon.rule.resistance_physical",
        Icon::Resistance(Resistance::Fire) => "icon.rule.resistance_fire",
        Icon::Resistance(Resistance::Ice) => "icon.rule.resistance_ice",
        Icon::Attack { element, .. } => match element {
            Element::Fire => "icon.rule.attack_fire",
            Element::Ice => "icon.rule.attack_ice",
            Element::ColdFire => "icon.rule.attack_cold_fire",
            Element::Physical => return None,
        },
        _ => return None,
    })
}

/// L'icône d'une attaque : sa sorte et son élément (C2).
pub fn attack_icon(kind: AttackKind, element: Element) -> Icon {
    Icon::Attack { kind, element }
}

/// L'icône d'une parade : la rondache de son élément (C2).
pub fn block_icon(element: Element) -> Icon {
    Icon::Block(element)
}

/// Cristal plein d'une couleur : les quatre couleurs de base, l'or et le noir des jetons.
pub fn crystal_image(color: ManaColor) -> ImageId {
    let stem = match color {
        ManaColor::Basic(BasicColor::Red) => "crystal_red",
        ManaColor::Basic(BasicColor::Blue) => "crystal_blue",
        ManaColor::Basic(BasicColor::White) => "crystal_white",
        ManaColor::Basic(BasicColor::Green) => "crystal_green",
        ManaColor::Gold => "crystal_gold",
        ManaColor::Black => "crystal_black",
    };
    ImageId::new("ui", stem)
}

/// Emplacement vide d'un cristal (C6).
pub const CRYSTAL_EMPTY_IMAGE: ImageId = ImageId::new("ui", "crystal_empty");

/// Emblème d'un héros, repris sur son drapeau (C26) : flamme, griffes, feuille, hache.
pub fn hero_emblem(hero: Hero) -> ImageId {
    let stem = match hero {
        Hero::Arythea => "hero_arythea",
        Hero::Goldyx => "hero_goldyx",
        Hero::Norowas => "hero_norowas",
        Hero::Tovak => "hero_tovak",
    };
    ImageId::new("ui", stem)
}

/// Toutes les images de `ui/`, sans doublon (44).
pub fn ui_images() -> Vec<ImageId> {
    let images: BTreeSet<ImageId> = Icon::all()
        .into_iter()
        .map(Icon::image)
        .chain(ManaColor::ALL.map(crystal_image))
        .chain([CRYSTAL_EMPTY_IMAGE])
        .chain(Hero::ALL.map(hero_emblem))
        .chain([Icon::Info.image()])
        .collect();
    images.into_iter().collect()
}

const RESISTANCES: [Resistance; 3] = [Resistance::Physical, Resistance::Fire, Resistance::Ice];

/// Les capacités des ennemis du jeu de base (spec #27, Q1), dans l'ordre de la planche.
const ABILITIES: [EnemyAbility; 5] = [
    EnemyAbility::Fortified,
    EnemyAbility::Swift,
    EnemyAbility::Brutal,
    EnemyAbility::Poison,
    EnemyAbility::Paralyze,
];
