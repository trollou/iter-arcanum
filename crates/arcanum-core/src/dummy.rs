//! Joueur fantôme (spec 061) : la règle du scénario, le choix de son héros dans la configuration,
//! et son état pendant la partie (MKUE p. 15, *Dummy Player*).

use crate::{BasicColor, CardId, Hero, Skill, Tactic};

/// Le joueur fantôme d'un scénario (spec 061, *Modèle*) : point d'extension, comme `TacticRule`.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum DummyRule {
    /// Pas de fantôme : le bac à sable, l'option « sans fantôme », et une sauvegarde d'avant la
    /// spec 061 (C35).
    #[default]
    None,
    /// Le fantôme de la variante solo (MKUE p. 15 ; p. 18, *Solo Variant*).
    Standard,
}

/// Le héros du fantôme choisi dans les Options de la nouvelle partie (spec 061, Q2).
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum DummyHeroChoice {
    /// Tiré parmi les trois héros que le joueur n'a pas pris (MKUE p. 15).
    #[default]
    Random,
    Hero(Hero),
}

/// Qui joue en premier dans la manche, d'après les deux tactiques (MKUE p. 4, 3.c).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoundOrder {
    PlayerFirst,
    DummyFirst,
}

/// Les exemplaires des cartes du fantôme sont numérotés à partir d'ici, pour ne pas se confondre
/// avec ceux du joueur (spec 061, *Identité des cartes*) : Rage n° 101, n° 102…
pub const DUMMY_COPY_OFFSET: u16 = 100;

/// Le fantôme pendant la partie (spec 061, *Modèle*) : ni figurine, ni main, ni niveau, ni unité.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DummyState {
    pub(crate) hero: Hero,
    /// Le dessus en premier : ordre caché.
    pub(crate) deck: Vec<CardId>,
    pub(crate) discard: Vec<CardId>,
    /// Cristaux par couleur, sans limite (MKUE p. 15, *When preparing a new Round*).
    pub(crate) crystals: [u8; 4],
    pub(crate) tactic: Option<Tactic>,
    /// Le dessus en premier : ordre caché.
    pub(crate) skills: Vec<Skill>,
    /// Les cartes retournées à son dernier tour.
    pub(crate) last_turn: Vec<CardId>,
    /// Il a annoncé la fin de la manche : le tour du joueur qui suit est le dernier.
    pub(crate) announced: bool,
}

impl DummyState {
    pub fn hero(&self) -> Hero {
        self.hero
    }

    /// L'ordre du deck, pour les tests et le moteur ; la vue n'en donne que le nombre.
    pub fn deck(&self) -> &[CardId] {
        &self.deck
    }

    pub fn discard(&self) -> &[CardId] {
        &self.discard
    }

    pub fn crystals(&self, color: BasicColor) -> u8 {
        self.crystals[color_index(color)]
    }

    pub fn tactic(&self) -> Option<Tactic> {
        self.tactic
    }

    /// La pile de compétences, le dessus en premier ; la vue n'en donne que le nombre.
    pub fn skills(&self) -> &[Skill] {
        &self.skills
    }

    pub fn last_turn(&self) -> &[CardId] {
        &self.last_turn
    }

    pub fn announced(&self) -> bool {
        self.announced
    }
}

pub(crate) fn color_index(color: BasicColor) -> usize {
    match color {
        BasicColor::Red => 0,
        BasicColor::Blue => 1,
        BasicColor::White => 2,
        BasicColor::Green => 3,
    }
}
