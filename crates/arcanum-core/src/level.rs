//! Piste de gloire et jetons de niveau (spec 035) : les dix niveaux, lus dans `content/levels.ron`.

use serde::Deserialize;

use crate::GameContent;

/// Le niveau ne dépasse jamais 10 ; la gloire, elle, continue de compter (Compendium,
/// *Miscellaneous* S8 (9)).
pub const MAX_LEVEL: u8 = 10;

/// Le jeton de niveau en vigueur : armure, limite de main et jetons de commandement (MKUE p. 3,
/// *Player Area* ; scans `misc/level_token_*.png`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LevelToken {
    pub armor: u8,
    pub hand_limit: u8,
    pub command_tokens: u8,
}

/// Ce que donne un niveau (MKUE p. 11, *End of the Turn* 6.a et 6.b).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum LevelReward {
    /// Le niveau 1, celui de la mise en place.
    Start,
    /// Niveau impair : le jeton de niveau suivant, et un jeton de commandement de plus.
    LevelToken,
    /// Niveau pair : compétence et action avancée (spec 051).
    SkillAndAdvancedAction,
}

/// Une ligne de la piste de gloire : la gloire qui y mène, le jeton en vigueur, la récompense.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LevelDefinition {
    pub fame: u32,
    pub token: LevelToken,
    pub reward: LevelReward,
}

/// La piste de gloire : les dix niveaux, dans l'ordre (MKUE p. 3 ; scan `misc/fame_board.jpg`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FameTrack {
    levels: Vec<LevelDefinition>,
}

impl FameTrack {
    /// Invariant : `levels` a été validé par `GameContent::parse` (dix niveaux, seuils croissants à
    /// partir de 0).
    pub(crate) fn new(levels: Vec<LevelDefinition>) -> Self {
        FameTrack { levels }
    }

    /// Les niveaux 1 à 10, dans l'ordre.
    pub fn levels(&self) -> &[LevelDefinition] {
        &self.levels
    }

    /// La définition d'un niveau, ramené entre 1 et 10.
    pub fn definition(&self, level: u8) -> LevelDefinition {
        let index = usize::from(level.clamp(1, MAX_LEVEL) - 1);
        // Invariant : dix niveaux validés au chargement du contenu.
        self.levels[index]
    }

    /// Le niveau de la ligne où se trouve cette gloire : atteindre un seuil franchit la ligne.
    pub fn level_for_fame(&self, fame: u32) -> u8 {
        let reached = self
            .levels
            .iter()
            .filter(|level| level.fame <= fame)
            .count();
        u8::try_from(reached)
            .unwrap_or(MAX_LEVEL)
            .clamp(1, MAX_LEVEL)
    }

    /// La gloire du niveau suivant ; aucune au niveau 10.
    pub fn next_threshold(&self, level: u8) -> Option<u32> {
        self.levels.get(usize::from(level)).map(|next| next.fame)
    }
}

/// Le jeton de niveau en vigueur au niveau `level`, d'après le contenu embarqué.
pub fn level_token(level: u8) -> LevelToken {
    GameContent::embedded().fame_track().definition(level).token
}
