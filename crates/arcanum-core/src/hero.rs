use serde::Deserialize;

use crate::{BasicAction, CardId, GameContent};

/// Héros du jeu de base (MKUE p. 1, choix du héros). Ceux des extensions viendront plus tard.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, serde::Serialize,
)]
pub enum Hero {
    Arythea,
    Goldyx,
    Norowas,
    Tovak,
}

impl Hero {
    pub const ALL: [Hero; 4] = [Hero::Arythea, Hero::Goldyx, Hero::Norowas, Hero::Tovak];

    /// Deck de départ : 16 cartes, numérotées par exemplaire (MKUE p. 3), lu dans `content/heroes.ron` (spec 029).
    pub fn starting_deck(self) -> Vec<CardId> {
        GameContent::embedded().starting_deck(self)
    }

    /// Le deck du joueur fantôme de ce héros (spec 061) : son deck de départ, ses exemplaires
    /// numérotés à partir de `DUMMY_COPY_OFFSET` pour ne pas se confondre avec ceux du joueur.
    pub fn dummy_deck(self) -> Vec<CardId> {
        self.starting_deck()
            .into_iter()
            .map(|card| CardId::new(card.card(), card.copy() + crate::DUMMY_COPY_OFFSET))
            .collect()
    }

    /// Cartes propres du héros, chacune avec la carte du deck standard qu'elle remplace
    /// (MKUE p. 3), dans l'ordre de `content/heroes.ron` (spec 065).
    pub fn own_cards(self) -> Vec<OwnCard> {
        GameContent::embedded().own_cards(self)
    }

    /// Clé du nom du héros : arcanum-core ne manipule que des clés (ADR 0003). Sert au résumé d'une
    /// sauvegarde (spec 024).
    pub fn label_key(self) -> &'static str {
        match self {
            Hero::Arythea => "hero.arythea.name",
            Hero::Goldyx => "hero.goldyx.name",
            Hero::Norowas => "hero.norowas.name",
            Hero::Tovak => "hero.tovak.name",
        }
    }
}

/// Une carte propre d'un héros et la carte commune qu'elle remplace (spec 065).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OwnCard {
    pub card: BasicAction,
    pub replaced: BasicAction,
}
