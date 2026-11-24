//! Couleurs de mana et Source (spec 020).

use rand::seq::IndexedRandom;

use crate::{BasicColor, GameRng, TimeOfDay};

/// Couleur de mana : quatre couleurs de base, or et noir (MKUE p. 5, « Using mana » 1 et 2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ManaColor {
    Basic(BasicColor),
    Gold,
    Black,
}

impl ManaColor {
    /// Les six faces d'un dé de mana, une par couleur.
    pub const ALL: [ManaColor; 6] = [
        ManaColor::Basic(BasicColor::Red),
        ManaColor::Basic(BasicColor::Blue),
        ManaColor::Basic(BasicColor::White),
        ManaColor::Basic(BasicColor::Green),
        ManaColor::Gold,
        ManaColor::Black,
    ];

    pub fn is_basic(self) -> bool {
        matches!(self, ManaColor::Basic(_))
    }

    /// MKUE p. 5, 3.c : un dé de la Source noir est épuisé de jour, or de nuit.
    pub fn is_depleted(self, time: TimeOfDay) -> bool {
        matches!(
            (self, time),
            (ManaColor::Black, TimeOfDay::Day) | (ManaColor::Gold, TimeOfDay::Night)
        )
    }
}

impl ManaColor {
    /// La couleur opposée sur le diagramme des jetons (Polarisation, spec 058) : rouge et bleu,
    /// blanc et vert, or et noir.
    pub fn opposite(self) -> ManaColor {
        match self {
            ManaColor::Basic(BasicColor::Red) => ManaColor::Basic(BasicColor::Blue),
            ManaColor::Basic(BasicColor::Blue) => ManaColor::Basic(BasicColor::Red),
            ManaColor::Basic(BasicColor::White) => ManaColor::Basic(BasicColor::Green),
            ManaColor::Basic(BasicColor::Green) => ManaColor::Basic(BasicColor::White),
            ManaColor::Gold => ManaColor::Black,
            ManaColor::Black => ManaColor::Gold,
        }
    }
}

/// Dés qu'un joueur peut prendre dans la Source à chaque tour (MKUE p. 5, 3).
pub const DICE_PER_TURN: usize = 1;

impl ManaColor {
    /// Ce mana peut-il payer un effet de couleur `color` ? L'or vaut toute couleur de base
    /// de jour ; le noir ne paie jamais une couleur de base (MKUE p. 5, 2.a et 2.b).
    pub fn can_pay(self, color: BasicColor, time: TimeOfDay) -> bool {
        match self {
            ManaColor::Basic(own) => own == color,
            ManaColor::Gold => time == TimeOfDay::Day,
            ManaColor::Black => false,
        }
    }
}

/// Nombre de dés de la Source : joueurs réels plus 2 (MKUE p. 3).
pub fn source_size(real_players: usize) -> usize {
    real_players + 2
}

/// Dés de couleur de base exigés : au moins la moitié, arrondi au-dessus (MKUE p. 3).
pub fn basic_dice_needed(count: usize) -> usize {
    count.div_ceil(2)
}

/// Une face au hasard parmi les six.
pub fn roll_die(rng: &mut GameRng) -> ManaColor {
    // Invariant : `ALL` n'est pas vide, `choose` rend toujours une face.
    *ManaColor::ALL.choose(rng).expect("un dé a six faces")
}

/// Lance `count` dés dans l'ordre, puis relance ensemble les dés or et noir, dans l'ordre des
/// dés, tant que la règle de la moitié n'est pas respectée (MKUE p. 3). Cet ordre d'usage du
/// générateur fait partie du contrat.
pub fn roll_source(count: usize, rng: &mut GameRng) -> Vec<ManaColor> {
    let mut dice: Vec<ManaColor> = (0..count).map(|_| roll_die(rng)).collect();
    // Sans borne : chaque relance a au moins une chance sur trois de réussir (spec 020, L2).
    while dice.iter().filter(|die| die.is_basic()).count() < basic_dice_needed(count) {
        for die in dice.iter_mut().filter(|die| !die.is_basic()) {
            *die = roll_die(rng);
        }
    }
    dice
}
