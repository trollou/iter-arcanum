//! Jetons de ruines (spec 045) : identifiants en Rust, définitions lues dans `content/ruins.ron`
//! (ADR 0005). Un exemplaire de chaque jeton du jeu de base (MKUE p. 3, *Enemy and Ruin token
//! piles* ; `../mage-knight/docs/regles/cartes/ennemis.md`, *Jetons de ruines*).

use serde::{Deserialize, Serialize};

use crate::{BasicColor, EnemyColor, GameContent};

/// Un jeton de ruines (jeton jaune) : quatre autels, huit jetons d'ennemis, nommés par leur
/// couleur ou les couleurs de leurs piles d'ennemis.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RuinToken {
    GreenAltar,
    BlueAltar,
    WhiteAltar,
    RedAltar,
    BrownRed,
    GreenRed,
    GreyWhite,
    GreyBrown,
    GreyViolet,
    GreenBrown,
    GreenGreen,
    BrownViolet,
}

impl RuinToken {
    pub const ALL: [RuinToken; 12] = [
        RuinToken::GreenAltar,
        RuinToken::BlueAltar,
        RuinToken::WhiteAltar,
        RuinToken::RedAltar,
        RuinToken::BrownRed,
        RuinToken::GreenRed,
        RuinToken::GreyWhite,
        RuinToken::GreyBrown,
        RuinToken::GreyViolet,
        RuinToken::GreenBrown,
        RuinToken::GreenGreen,
        RuinToken::BrownViolet,
    ];

    /// La définition du jeton, lue dans le contenu embarqué.
    pub fn definition(self) -> &'static RuinTokenDefinition {
        GameContent::embedded().ruin_token(self)
    }

    /// Vrai pour un autel.
    pub fn is_altar(self) -> bool {
        self.definition().altar().is_some()
    }
}

/// Ce que montre un jeton de ruines.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub enum RuinKind {
    /// Autel : 3 mana de cette couleur en offrande, 7 gloire (carte *Ancient Ruins*, *Altar*).
    Altar(BasicColor),
    /// Ennemis : un jeton tiré par couleur de pile, et la récompense du jeton (carte, *Enemies*).
    Enemies {
        colors: Vec<EnemyColor>,
        rewards: Vec<RuinReward>,
    },
}

/// Une récompense d'un jeton de ruines, prise en fin de tour (MKUE p. 10, *Combat Outcomes* 2.b ;
/// p. 11, 5).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum RuinReward {
    Artifact,
    Spell,
    AdvancedAction,
    /// Un cristal de chaque couleur de base.
    CrystalSet,
    /// Une unité de l'offre d'unités, gratuite.
    Unit,
}

/// La définition d'un jeton de ruines (`content/ruins.ron`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuinTokenDefinition {
    pub kind: RuinKind,
}

impl RuinTokenDefinition {
    /// La couleur de l'autel, ou rien pour un jeton d'ennemis.
    pub fn altar(&self) -> Option<BasicColor> {
        match self.kind {
            RuinKind::Altar(color) => Some(color),
            RuinKind::Enemies { .. } => None,
        }
    }

    /// Les couleurs des piles d'ennemis à tirer ; aucune pour un autel.
    pub fn enemies(&self) -> &[EnemyColor] {
        match &self.kind {
            RuinKind::Altar(_) => &[],
            RuinKind::Enemies { colors, .. } => colors,
        }
    }

    /// La récompense du jeton ; aucune pour un autel (la gloire se gagne aussitôt).
    pub fn rewards(&self) -> &[RuinReward] {
        match &self.kind {
            RuinKind::Altar(_) => &[],
            RuinKind::Enemies { rewards, .. } => rewards,
        }
    }
}

/// Un jeton de ruines posé sur une ruine non conquise, et sa face (spec 045).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlacedRuin {
    pub token: RuinToken,
    pub face_up: bool,
}
