use rand_chacha::ChaCha8Rng;

use thiserror::Error;

use crate::{DummyHeroChoice, GameRngs, Hero, Scenario};

/// Générateur du moteur : ChaCha8, même suite d'une plateforme à l'autre.
pub type GameRng = ChaCha8Rng;

/// Ce qui suffit à créer une partie (spec 012) : scénario, héros, graine.
/// Le deck vient du héros (spec 013). Les variantes y auront leur place quand la première existera.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GameConfig {
    pub seed: u64,
    /// La valeur entière du scénario : une sauvegarde rejoue la même mise en place
    /// même si sa définition change plus tard.
    pub scenario: Scenario,
    pub hero: Hero,
    /// Le héros du joueur fantôme, s'il y en a un (spec 061, Q2) ; absent d'une sauvegarde
    /// d'avant : au hasard.
    #[serde(default)]
    pub dummy_hero: DummyHeroChoice,
}

/// Refus d'une configuration incohérente (spec 061, L10).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum ConfigError {
    #[error("le joueur fantôme ne peut pas prendre le héros du joueur")]
    DummyHeroIsPlayerHero,
}

impl GameConfig {
    /// Les générateurs de la partie, un par usage, dérivés de la graine (spec 038).
    pub fn rngs(&self) -> GameRngs {
        GameRngs::new(self.seed)
    }

    /// Vérifie la configuration avant de créer la partie (spec 061, L10).
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.dummy_hero == DummyHeroChoice::Hero(self.hero) {
            return Err(ConfigError::DummyHeroIsPlayerHero);
        }
        Ok(())
    }
}
