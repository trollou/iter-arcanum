//! Un générateur par usage du hasard (spec 038, ADR 0007) : un tirage ajouté à un usage ne décale
//! pas les autres.

use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

use rand::SeedableRng;

use crate::{CommonDeck, EnemyColor, GameRng, PlayerId};

/// Famille de tirages qui partagent une même suite.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RngUsage {
    /// Formation de la pioche de tuiles, tuile écartée tirée au hasard.
    Tiles,
    /// Lancers et relances des dés de la Source.
    Source,
    /// Mélanges du deck d'un joueur.
    Deck(PlayerId),
    /// Mélanges d'une pile de jetons d'ennemis.
    EnemyPile(EnemyColor),
    /// Mélanges d'une pioche commune : actions avancées, sorts, artefacts, unités (spec 050).
    CommonDeck(CommonDeck),
    /// Dés de récompense lancés de la réserve : dé du donjon, cristaux aléatoires (spec 044).
    RewardDice,
    /// Mélange de la pile des jetons de ruines à la mise en place (spec 045).
    RuinPile,
    /// Cartes tirées au hasard dans la défausse d'un joueur (Méditation, spec 057).
    DiscardPick(PlayerId),
    /// Dés de mana lancés par un artefact : Corne de la colère, Poche de mana (spec 055).
    ArtifactDice,
    /// Mélange de la pile de compétences d'un joueur à la mise en place (spec 051).
    SkillPile(PlayerId),
    /// Spec 061 : le joueur fantôme (héros, compétences, deck, mélanges, tactique).
    Dummy,
}

/// Où en est le générateur de chaque usage déjà tiré : ce que la sauvegarde enregistre.
pub type RngPositions = BTreeMap<RngUsage, u128>;

impl RngUsage {
    /// Numéro du flux ChaCha8 de l'usage. Il ne change jamais une fois livré : le changer
    /// changerait toutes les parties (ADR 0007).
    pub fn stream(self) -> u64 {
        match self {
            RngUsage::Tiles => 1,
            RngUsage::Source => 2,
            RngUsage::RewardDice => 3,
            RngUsage::RuinPile => 4,
            RngUsage::ArtifactDice => 5,
            RngUsage::Dummy => 6,
            // 100 à 199 : usages par joueur.
            RngUsage::Deck(player) => 100 + u64::from(player.0),
            // 150 à 199 : tirages dans la défausse d'un joueur (spec 057).
            RngUsage::DiscardPick(player) => 150 + u64::from(player.0),
            // 120 à 149 : piles de compétences (spec 051).
            RngUsage::SkillPile(player) => 120 + u64::from(player.0),
            // 200 à 299 : piles de jetons.
            RngUsage::EnemyPile(color) => 200 + pile_index(color),
            // 300 à 399 : pioches communes (spec 050).
            RngUsage::CommonDeck(deck) => 300 + common_deck_index(deck),
        }
    }
}

fn pile_index(color: EnemyColor) -> u64 {
    match color {
        EnemyColor::Green => 0,
        EnemyColor::Grey => 1,
        EnemyColor::Violet => 2,
        EnemyColor::Brown => 3,
        EnemyColor::Red => 4,
        EnemyColor::White => 5,
    }
}

fn common_deck_index(deck: CommonDeck) -> u64 {
    match deck {
        CommonDeck::AdvancedActions => 0,
        CommonDeck::Spells => 1,
        CommonDeck::Artifacts => 2,
        CommonDeck::RegularUnits => 3,
        CommonDeck::EliteUnits => 4,
    }
}

fn common_deck_name(deck: CommonDeck) -> &'static str {
    match deck {
        CommonDeck::AdvancedActions => "advanced_actions",
        CommonDeck::Spells => "spells",
        CommonDeck::Artifacts => "artifacts",
        CommonDeck::RegularUnits => "regular_units",
        CommonDeck::EliteUnits => "elite_units",
    }
}

fn color_name(color: EnemyColor) -> &'static str {
    match color {
        EnemyColor::Green => "green",
        EnemyColor::Grey => "grey",
        EnemyColor::Violet => "violet",
        EnemyColor::Brown => "brown",
        EnemyColor::Red => "red",
        EnemyColor::White => "white",
    }
}

/// Nom de l'usage dans la sauvegarde : `tiles`, `source`, `deck/0`, `enemy_pile/green`…
impl fmt::Display for RngUsage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RngUsage::Tiles => f.write_str("tiles"),
            RngUsage::Source => f.write_str("source"),
            RngUsage::RewardDice => f.write_str("reward_dice"),
            RngUsage::RuinPile => f.write_str("ruin_pile"),
            RngUsage::ArtifactDice => f.write_str("artifact_dice"),
            RngUsage::Dummy => f.write_str("dummy"),
            RngUsage::Deck(player) => write!(f, "deck/{}", player.0),
            RngUsage::DiscardPick(player) => write!(f, "discard_pick/{}", player.0),
            RngUsage::SkillPile(player) => write!(f, "skill_pile/{}", player.0),
            RngUsage::EnemyPile(color) => write!(f, "enemy_pile/{}", color_name(*color)),
            RngUsage::CommonDeck(deck) => write!(f, "common_deck/{}", common_deck_name(*deck)),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("usage du hasard inconnu : {0}")]
pub struct UnknownRngUsage(pub String);

impl FromStr for RngUsage {
    type Err = UnknownRngUsage;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let unknown = || UnknownRngUsage(text.to_string());
        match text.split_once('/') {
            None if text == "tiles" => Ok(RngUsage::Tiles),
            None if text == "source" => Ok(RngUsage::Source),
            None if text == "reward_dice" => Ok(RngUsage::RewardDice),
            None if text == "ruin_pile" => Ok(RngUsage::RuinPile),
            None if text == "artifact_dice" => Ok(RngUsage::ArtifactDice),
            None if text == "dummy" => Ok(RngUsage::Dummy),
            Some(("deck", player)) => player
                .parse()
                .map(|index| RngUsage::Deck(PlayerId(index)))
                .map_err(|_| unknown()),
            Some(("discard_pick", player)) => player
                .parse()
                .map(|index| RngUsage::DiscardPick(PlayerId(index)))
                .map_err(|_| unknown()),
            Some(("skill_pile", player)) => player
                .parse()
                .map(|index| RngUsage::SkillPile(PlayerId(index)))
                .map_err(|_| unknown()),
            Some(("enemy_pile", name)) => EnemyColor::ALL
                .into_iter()
                .find(|&color| color_name(color) == name)
                .map(RngUsage::EnemyPile)
                .ok_or_else(unknown),
            Some(("common_deck", name)) => CommonDeck::ALL
                .into_iter()
                .find(|&deck| common_deck_name(deck) == name)
                .map(RngUsage::CommonDeck)
                .ok_or_else(unknown),
            _ => Err(unknown()),
        }
    }
}

impl serde::Serialize for RngUsage {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> serde::Deserialize<'de> for RngUsage {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        text.parse().map_err(serde::de::Error::custom)
    }
}

/// Les générateurs de la partie, tous dérivés de sa graine. Chacun n'est créé qu'à son premier
/// tirage : un usage jamais tiré n'a pas de position.
#[derive(Clone, Debug)]
pub struct GameRngs {
    seed: u64,
    generators: BTreeMap<RngUsage, GameRng>,
}

impl GameRngs {
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            generators: BTreeMap::new(),
        }
    }

    /// Reprend les générateurs aux positions données ; un usage absent repart du début de son
    /// flux, comme dans la partie d'origine où il n'avait encore rien tiré.
    pub fn resume(seed: u64, positions: &RngPositions) -> Self {
        let mut rngs = Self::new(seed);
        for (&usage, &position) in positions {
            rngs.rng(usage).set_word_pos(position);
        }
        rngs
    }

    /// Le générateur de l'usage, créé au besoin.
    pub fn rng(&mut self, usage: RngUsage) -> &mut GameRng {
        let seed = self.seed;
        self.generators.entry(usage).or_insert_with(|| {
            let mut rng = GameRng::seed_from_u64(seed);
            rng.set_stream(usage.stream());
            rng
        })
    }

    pub fn positions(&self) -> RngPositions {
        self.generators
            .iter()
            .map(|(&usage, rng)| (usage, rng.get_word_pos()))
            .collect()
    }
}
