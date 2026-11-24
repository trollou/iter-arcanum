//! Tactiques (spec 060) : les douze cartes, la règle du scénario qui dit comment on les choisit et
//! ce qui est retiré, et l'état de la tactique d'un joueur pendant la manche.

use crate::{CardId, ManaColor, PlayerId, TimeOfDay};

/// Les douze tactiques, six de Jour puis six de Nuit, par numéro (MKUE p. 3 ; cartes).
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum Tactic {
    EarlyBird,
    Rethink,
    ManaSteal,
    Planning,
    GreatStart,
    TheRightMoment,
    FromTheDusk,
    LongNight,
    ManaSearch,
    MidnightMeditation,
    Preparation,
    SparingPower,
}

impl Tactic {
    pub const ALL: [Tactic; 12] = [
        Tactic::EarlyBird,
        Tactic::Rethink,
        Tactic::ManaSteal,
        Tactic::Planning,
        Tactic::GreatStart,
        Tactic::TheRightMoment,
        Tactic::FromTheDusk,
        Tactic::LongNight,
        Tactic::ManaSearch,
        Tactic::MidnightMeditation,
        Tactic::Preparation,
        Tactic::SparingPower,
    ];

    /// Le numéro de la carte, 1 à 6 : il ne sert qu'à l'ordre du tour (spec 061).
    pub fn number(self) -> u8 {
        match self {
            Tactic::EarlyBird | Tactic::FromTheDusk => 1,
            Tactic::Rethink | Tactic::LongNight => 2,
            Tactic::ManaSteal | Tactic::ManaSearch => 3,
            Tactic::Planning | Tactic::MidnightMeditation => 4,
            Tactic::GreatStart | Tactic::Preparation => 5,
            Tactic::TheRightMoment | Tactic::SparingPower => 6,
        }
    }

    pub fn time_of_day(self) -> TimeOfDay {
        match self {
            Tactic::EarlyBird
            | Tactic::Rethink
            | Tactic::ManaSteal
            | Tactic::Planning
            | Tactic::GreatStart
            | Tactic::TheRightMoment => TimeOfDay::Day,
            _ => TimeOfDay::Night,
        }
    }

    /// Les six tactiques de ce moment de la journée, par numéro.
    pub fn of(time: TimeOfDay) -> impl Iterator<Item = Tactic> {
        Tactic::ALL
            .into_iter()
            .filter(move |tactic| tactic.time_of_day() == time)
    }

    /// Identifiant des clés de libellés : `tactic.<id>.name`, `tactic.<id>.text` (ADR 0003).
    pub fn id(self) -> &'static str {
        match self {
            Tactic::EarlyBird => "early_bird",
            Tactic::Rethink => "rethink",
            Tactic::ManaSteal => "mana_steal",
            Tactic::Planning => "planning",
            Tactic::GreatStart => "great_start",
            Tactic::TheRightMoment => "right_moment",
            Tactic::FromTheDusk => "from_the_dusk",
            Tactic::LongNight => "long_night",
            Tactic::ManaSearch => "mana_search",
            Tactic::MidnightMeditation => "midnight_meditation",
            Tactic::Preparation => "preparation",
            Tactic::SparingPower => "sparing_power",
        }
    }
}

/// La règle des tactiques d'un scénario (spec 060, *Modèle*) : point d'extension.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum TacticRule {
    /// Pas de tactique : le bac à sable, et une sauvegarde d'avant la spec 060 (C40).
    #[default]
    None,
    /// Une tactique choisie au début de chaque manche.
    Chosen {
        order: TacticOrder,
        removal: TacticRemoval,
    },
}

/// Qui choisit sa tactique en premier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum TacticOrder {
    /// Le joueur d'abord, parmi toutes les restantes (MKUE p. 19, *Special Rules*).
    PlayerFirst,
}

/// Ce qui est retiré de la partie à la fin de la manche.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum TacticRemoval {
    /// Toutes les tactiques prises (MKUE p. 19, *Special Rules* ; Compendium, *Scenarios* Q8).
    AllTaken,
}

/// Une tactique de la manche telle que le choix la montre (remarque de Mathieu, grille 2 × 3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TacticSlot {
    Available,
    /// Retirée de la partie : un trou à sa place.
    Removed,
    /// Prise cette manche par ce joueur (avec le joueur fantôme, spec 061).
    Taken(PlayerId),
}

/// La tactique d'un joueur pendant la manche et son état (spec 060, *Modèle*).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TacticState {
    pub tactic: Tactic,
    /// Retournée face cachée : elle a servi (Compendium, *Miscellaneous* (12)).
    pub flipped: bool,
    /// Vol de mana : la position dans la Source du dé posé sur la tactique.
    pub stolen_die: Option<u8>,
    /// Provisions : les cartes posées dessous, face cachée.
    pub(crate) stored: Vec<CardId>,
    /// Appel du mana utilisé ce tour.
    pub mana_search_used: bool,
    /// Au bon moment annoncé : un tour de plus après celui-ci.
    pub right_moment: bool,
}

impl TacticState {
    /// Le nombre de cartes sous Provisions : lesquelles, personne ne le sait (Q4).
    pub fn stored_count(&self) -> usize {
        self.stored.len()
    }

    pub(crate) fn new(tactic: Tactic) -> Self {
        Self {
            tactic,
            flipped: false,
            stolen_die: None,
            stored: Vec::new(),
            mana_search_used: false,
            right_moment: false,
        }
    }
}

/// La tactique d'un joueur dans sa vue (spec 060, C35) : les cartes sous Provisions, comptées
/// seulement (Q4) ; le dé volé, sa position dans la Source et sa couleur.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TacticView {
    pub tactic: Tactic,
    pub flipped: bool,
    pub stolen_die: Option<(u8, ManaColor)>,
    pub stored: usize,
    pub mana_search_used: bool,
    pub right_moment: bool,
}
