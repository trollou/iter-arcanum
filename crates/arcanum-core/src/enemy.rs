//! Jetons d'ennemis (spec 030). Leurs valeurs sont dans `content/enemies.ron`.

use serde::Deserialize;

use crate::{Element, GameContent, Rampaging};

/// Couleur du dos d'un jeton d'ennemi : une pile par couleur (MKUE p. 3).
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, serde::Serialize,
)]
pub enum EnemyColor {
    /// Orques maraudeurs.
    Green,
    /// Gardes des forteresses.
    Grey,
    /// Tours de mage.
    Violet,
    /// Donjons et antres.
    Brown,
    /// Draconiens.
    Red,
    /// Cités.
    White,
}

impl EnemyColor {
    /// Les six piles, dans l'ordre de la mise en place (spec 030).
    pub const ALL: [EnemyColor; 6] = [
        EnemyColor::Green,
        EnemyColor::Grey,
        EnemyColor::Violet,
        EnemyColor::Brown,
        EnemyColor::Red,
        EnemyColor::White,
    ];

    /// Clé de libellé du groupe (« orques maraudeurs ») : arcanum-core ne manipule que des clés.
    pub fn label_key(self) -> &'static str {
        match self {
            EnemyColor::Green => "enemy.color.green",
            EnemyColor::Grey => "enemy.color.grey",
            EnemyColor::Violet => "enemy.color.violet",
            EnemyColor::Brown => "enemy.color.brown",
            EnemyColor::Red => "enemy.color.red",
            EnemyColor::White => "enemy.color.white",
        }
    }
}

impl Rampaging {
    /// La pile où l'on tire le jeton d'un ennemi errant imprimé (cartes de site *Marauding Orcs*
    /// et *Draconum*).
    pub fn color(self) -> EnemyColor {
        match self {
            Rampaging::OrcMarauders => EnemyColor::Green,
            Rampaging::Draconum => EnemyColor::Red,
        }
    }
}

/// Un ennemi (une sorte de jeton), d'après `../mage-knight/docs/regles/cartes/ennemis.md`.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, serde::Serialize,
)]
pub enum Enemy {
    Prowlers,
    Diggers,
    CursedHags,
    WolfRiders,
    Ironclads,
    OrcSummoners,
    Crossbowmen,
    Guardsmen,
    Swordsmen,
    Golems,
    Monks,
    Illusionists,
    IceMages,
    IceGolems,
    FireMages,
    FireGolems,
    Minotaur,
    Gargoyle,
    Medusa,
    CryptWorm,
    Werewolf,
    SwampDragon,
    FireDragon,
    IceDragon,
    HighDragon,
    Freezers,
    Gunners,
    AltemGuardsmen,
    AltemMages,
}

impl Enemy {
    pub const ALL: [Enemy; 29] = [
        Enemy::Prowlers,
        Enemy::Diggers,
        Enemy::CursedHags,
        Enemy::WolfRiders,
        Enemy::Ironclads,
        Enemy::OrcSummoners,
        Enemy::Crossbowmen,
        Enemy::Guardsmen,
        Enemy::Swordsmen,
        Enemy::Golems,
        Enemy::Monks,
        Enemy::Illusionists,
        Enemy::IceMages,
        Enemy::IceGolems,
        Enemy::FireMages,
        Enemy::FireGolems,
        Enemy::Minotaur,
        Enemy::Gargoyle,
        Enemy::Medusa,
        Enemy::CryptWorm,
        Enemy::Werewolf,
        Enemy::SwampDragon,
        Enemy::FireDragon,
        Enemy::IceDragon,
        Enemy::HighDragon,
        Enemy::Freezers,
        Enemy::Gunners,
        Enemy::AltemGuardsmen,
        Enemy::AltemMages,
    ];

    /// Valeurs du jeton, lues dans `content/enemies.ron`.
    pub fn definition(self) -> &'static EnemyDefinition {
        GameContent::embedded().enemy(self)
    }

    /// Clé de libellé du nom : arcanum-core ne manipule que des clés (ADR 0003).
    pub fn label_key(self) -> &'static str {
        match self {
            Enemy::Prowlers => "enemy.prowlers.name",
            Enemy::Diggers => "enemy.diggers.name",
            Enemy::CursedHags => "enemy.cursed_hags.name",
            Enemy::WolfRiders => "enemy.wolf_riders.name",
            Enemy::Ironclads => "enemy.ironclads.name",
            Enemy::OrcSummoners => "enemy.orc_summoners.name",
            Enemy::Crossbowmen => "enemy.crossbowmen.name",
            Enemy::Guardsmen => "enemy.guardsmen.name",
            Enemy::Swordsmen => "enemy.swordsmen.name",
            Enemy::Golems => "enemy.golems.name",
            Enemy::Monks => "enemy.monks.name",
            Enemy::Illusionists => "enemy.illusionists.name",
            Enemy::IceMages => "enemy.ice_mages.name",
            Enemy::IceGolems => "enemy.ice_golems.name",
            Enemy::FireMages => "enemy.fire_mages.name",
            Enemy::FireGolems => "enemy.fire_golems.name",
            Enemy::Minotaur => "enemy.minotaur.name",
            Enemy::Gargoyle => "enemy.gargoyle.name",
            Enemy::Medusa => "enemy.medusa.name",
            Enemy::CryptWorm => "enemy.crypt_worm.name",
            Enemy::Werewolf => "enemy.werewolf.name",
            Enemy::SwampDragon => "enemy.swamp_dragon.name",
            Enemy::FireDragon => "enemy.fire_dragon.name",
            Enemy::IceDragon => "enemy.ice_dragon.name",
            Enemy::HighDragon => "enemy.high_dragon.name",
            Enemy::Freezers => "enemy.freezers.name",
            Enemy::Gunners => "enemy.gunners.name",
            Enemy::AltemGuardsmen => "enemy.altem_guardsmen.name",
            Enemy::AltemMages => "enemy.altem_mages.name",
        }
    }
}

/// Attaque d'un ennemi : sa valeur et son élément, ou une invocation (MKUE p. 24).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum EnemyAttack {
    Physical(u8),
    Fire(u8),
    Ice(u8),
    ColdFire(u8),
    Summon,
}

impl EnemyAttack {
    /// L'élément de l'attaque, `None` pour une invocation (MKUE p. 24, *Offensive*).
    pub fn element(self) -> Option<Element> {
        match self {
            EnemyAttack::Physical(_) => Some(Element::Physical),
            EnemyAttack::Fire(_) => Some(Element::Fire),
            EnemyAttack::Ice(_) => Some(Element::Ice),
            EnemyAttack::ColdFire(_) => Some(Element::ColdFire),
            EnemyAttack::Summon => None,
        }
    }
}

/// Capacités d'un ennemi (MKUE p. 24) ; leurs effets viennent avec la spec 033.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
pub enum EnemyAbility {
    Fortified,
    Swift,
    Brutal,
    Poison,
    Paralyze,
}

/// Résistances d'un ennemi (MKUE p. 24, *Defensive* ; spec 032).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
pub enum Resistance {
    Physical,
    Fire,
    Ice,
}

/// Valeurs imprimées sur un jeton d'ennemi.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnemyDefinition {
    pub color: EnemyColor,
    /// Nombre d'exemplaires du jeton.
    pub count: u8,
    pub armor: u8,
    pub attack: EnemyAttack,
    pub abilities: Vec<EnemyAbility>,
    pub resistances: Vec<Resistance>,
    pub fame: u8,
}

/// Un exemplaire de jeton : l'ennemi et son numéro d'exemplaire, comme `CardId`.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct EnemyToken {
    enemy: Enemy,
    copy: u8,
}

impl EnemyToken {
    pub fn new(enemy: Enemy, copy: u8) -> Self {
        Self { enemy, copy }
    }

    pub fn enemy(self) -> Enemy {
        self.enemy
    }

    pub fn copy(self) -> u8 {
        self.copy
    }

    pub fn color(self) -> EnemyColor {
        self.enemy.definition().color
    }
}
