//! Contenu du jeu en données (spec 029, ADR 0005) : cartes, decks de départ et tuiles, lus depuis
//! les fichiers RON de `crates/arcanum-core/content/`, embarqués à la compilation. Aucun fichier n'est ouvert.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::marker::PhantomData;
use std::sync::LazyLock;

use serde::Deserialize;
use serde::de::{Deserializer, Error as _, MapAccess, SeqAccess, Visitor};
use thiserror::Error;

use crate::board::{Feature, SpaceContent, Terrain, TileDefinition, TileId};
use crate::level::{FameTrack, LevelDefinition, LevelReward, LevelToken, MAX_LEVEL};
use crate::reputation::{Reputation, ReputationTrack};
use crate::{AbilityEffect, AbilityOption, ManaColor, ModifierKind, UnitAbility};
use crate::{
    AdvancedAction, Artifact, AttackKind, BasicAction, BasicColor, BasicEffect, CardDefinition,
    CardDestination, CardId, Cost, DeedCard, Effect, Element, Enemy, EnemyAbility, EnemyAttack,
    EnemyColor, EnemyDefinition, EnemyToken, Hero, OwnCard, RecruitSite, Resistance, RuinKind,
    RuinToken, RuinTokenDefinition, Spell, Unit, UnitCard, UnitDefinition, UnitRank,
};
use crate::{Skill, SkillDefinition, SkillEffect, SkillOption, SkillUsage};

const BASIC_ACTIONS_FILE: &str = "basic_actions.ron";
const HEROES_FILE: &str = "heroes.ron";
const TILES_FILE: &str = "tiles.ron";
const ENEMIES_FILE: &str = "enemies.ron";
const LEVELS_FILE: &str = "levels.ron";
const ADVANCED_ACTIONS_FILE: &str = "advanced_actions.ron";
const SPELLS_FILE: &str = "spells.ron";
const ARTIFACTS_FILE: &str = "artifacts.ron";
const UNITS_FILE: &str = "units.ron";
const REPUTATION_FILE: &str = "reputation.ron";
const RUINS_FILE: &str = "ruins.ron";
const SKILLS_FILE: &str = "skills.ron";
const CITIES_FILE: &str = "cities.ron";

/// Cartes d'une pioche d'unités, régulières ou d'élite (MKUE p. 3 ; spec 050).
const UNITS_PER_DECK: usize = 20;

/// Taille du deck de départ (MKUE p. 3).
const STARTING_DECK_SIZE: usize = 16;

/// Le texte des fichiers de contenu, un champ par famille.
#[derive(Clone, Copy, Debug)]
pub struct ContentSources<'a> {
    pub basic_actions: &'a str,
    pub heroes: &'a str,
    pub tiles: &'a str,
    pub enemies: &'a str,
    pub levels: &'a str,
    pub advanced_actions: &'a str,
    pub spells: &'a str,
    pub artifacts: &'a str,
    pub units: &'a str,
    pub reputation: &'a str,
    /// Jetons de ruines (spec 045).
    pub ruins: &'a str,
    /// Compétences des héros (spec 051).
    pub skills: &'a str,
    /// Défenseurs des cités selon leur niveau (spec 062).
    pub cities: &'a str,
}

impl ContentSources<'static> {
    /// Les fichiers de `crates/arcanum-core/content/`, embarqués dans le binaire.
    pub const EMBEDDED: Self = ContentSources {
        basic_actions: include_str!("../content/basic_actions.ron"),
        heroes: include_str!("../content/heroes.ron"),
        tiles: include_str!("../content/tiles.ron"),
        enemies: include_str!("../content/enemies.ron"),
        levels: include_str!("../content/levels.ron"),
        advanced_actions: include_str!("../content/advanced_actions.ron"),
        spells: include_str!("../content/spells.ron"),
        artifacts: include_str!("../content/artifacts.ron"),
        units: include_str!("../content/units.ron"),
        reputation: include_str!("../content/reputation.ron"),
        ruins: include_str!("../content/ruins.ron"),
        skills: include_str!("../content/skills.ron"),
        cities: include_str!("../content/cities.ron"),
    };
}

/// Erreur de lecture ou de validation d'un fichier de contenu : une erreur de développement,
/// jamais montrée au joueur.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ContentError {
    #[error("{file}:{line}:{column} : {message}")]
    Syntax {
        file: &'static str,
        line: usize,
        column: usize,
        message: String,
    },
    #[error("{file} : {id} n'a pas de définition")]
    Missing { file: &'static str, id: String },
    #[error("{file} : {id} est défini deux fois")]
    Duplicate { file: &'static str, id: String },
    #[error("{file} : {id} n'existe pas dans le jeu de base")]
    Unknown { file: &'static str, id: String },
    #[error("heroes.ron : {reason}")]
    InvalidDeck { reason: String },
    #[error("{file} : effet invalide pour {card} ({reason})")]
    InvalidEffect {
        file: &'static str,
        card: String,
        reason: String,
    },
    #[error("{file} : numéro {number} invalide ou en double")]
    InvalidNumber { file: &'static str, number: u8 },
    #[error("units.ron : {reason}")]
    InvalidUnits { reason: String },
    #[error("enemies.ron : {enemy:?} n'a aucun exemplaire")]
    NoEnemyToken { enemy: Enemy },
    #[error("levels.ron : {reason}")]
    InvalidLevels { reason: String },
    #[error("reputation.ron : {reason}")]
    InvalidReputation { reason: String },
    #[error("ruins.ron : {token:?} n'a pas d'ennemi ou pas de récompense")]
    InvalidRuin { token: RuinToken },
    #[error("skills.ron : {hero:?} n'a pas dix compétences dont une coopérative")]
    InvalidSkills { hero: Hero },
    #[error("cities.ron : {reason}")]
    InvalidCities { reason: String },
}

/// Ensemble des définitions lues depuis les fichiers de contenu.
#[derive(Debug)]
pub struct GameContent {
    cards: BTreeMap<BasicAction, CardEntry>,
    standard_deck: Vec<(BasicAction, u8)>,
    replacements: BTreeMap<Hero, Vec<Replacement>>,
    /// Les cercles colorés de chaque héros, pour le joueur fantôme (spec 061).
    dummy_crystals: BTreeMap<Hero, Vec<BasicColor>>,
    tiles: Vec<TileDefinition>,
    /// Dans l'ordre du fichier, qui est l'ordre des jetons d'une pile avant mélange.
    enemies: Vec<(Enemy, EnemyDefinition)>,
    fame_track: FameTrack,
    reputation_track: ReputationTrack,
    advanced_actions: BTreeMap<AdvancedAction, NumberedCardEntry>,
    spells: BTreeMap<Spell, NumberedCardEntry>,
    artifacts: BTreeMap<Artifact, ArtifactEntry>,
    units: BTreeMap<Unit, UnitDefinition>,
    /// Dans l'ordre du fichier, qui est l'ordre de la pile avant mélange (spec 045).
    ruins: Vec<(RuinToken, RuinTokenDefinition)>,
    /// Les compétences (spec 051).
    skills: BTreeMap<Skill, SkillDefinition>,
    /// Les couleurs des défenseurs de chaque cité, niveaux 1 à 11 (spec 062).
    cities: CityGarrisons,
}

static EMBEDDED: LazyLock<GameContent> = LazyLock::new(|| {
    // Invariant : `embedded_content_is_valid` lit ces mêmes fichiers à chaque CI (spec 029, Q3).
    GameContent::parse(ContentSources::EMBEDDED)
        .unwrap_or_else(|error| panic!("contenu embarqué invalide : {error}"))
});

impl GameContent {
    /// Le contenu embarqué, lu et validé au premier accès.
    pub fn embedded() -> &'static GameContent {
        &EMBEDDED
    }

    /// Lit et valide les fichiers de contenu.
    pub fn parse(sources: ContentSources<'_>) -> Result<GameContent, ContentError> {
        let cards: Entries<BasicAction, CardEntry> =
            read(BASIC_ACTIONS_FILE, sources.basic_actions)?;
        let heroes: HeroesFile = read(HEROES_FILE, sources.heroes)?;
        let tiles: Vec<TileEntry> = read(TILES_FILE, sources.tiles)?;

        let cards = exhaustive(BASIC_ACTIONS_FILE, cards.0, BasicAction::ALL)?;
        for (card, entry) in &cards {
            entry.validate(*card)?;
        }
        let standard_deck = unique(HEROES_FILE, heroes.standard_deck.0)?;
        let replacements = exhaustive(HEROES_FILE, heroes.heroes.0, Hero::ALL)?;
        validate_decks(&standard_deck, &replacements)?;
        let dummy_crystals = exhaustive(HEROES_FILE, heroes.dummy_crystals.0, Hero::ALL)?;
        validate_dummy_crystals(&dummy_crystals)?;
        let tiles = validate_tiles(tiles)?;
        let enemies: Vec<EnemyEntry> = read(ENEMIES_FILE, sources.enemies)?;
        let enemies = validate_enemies(enemies)?;
        let levels: Vec<LevelEntry> = read(LEVELS_FILE, sources.levels)?;
        let fame_track = validate_levels(levels)?;
        let reputation: Vec<ReputationEntry> = read(REPUTATION_FILE, sources.reputation)?;
        let reputation_track = validate_reputation(reputation)?;
        let advanced_actions = validate_numbered(
            ADVANCED_ACTIONS_FILE,
            read(ADVANCED_ACTIONS_FILE, sources.advanced_actions)?,
            AdvancedAction::ALL,
        )?;
        let spells =
            validate_numbered(SPELLS_FILE, read(SPELLS_FILE, sources.spells)?, Spell::ALL)?;
        let artifacts: Entries<Artifact, ArtifactEntry> = read(ARTIFACTS_FILE, sources.artifacts)?;
        let artifacts = exhaustive(ARTIFACTS_FILE, artifacts.0, Artifact::ALL)?;
        unique_numbers(
            ARTIFACTS_FILE,
            artifacts.values().map(|entry| entry.number),
            16,
        )?;
        let units = validate_units(read(UNITS_FILE, sources.units)?)?;
        let ruins = validate_ruins(read(RUINS_FILE, sources.ruins)?)?;
        let skills = validate_skills(read(SKILLS_FILE, sources.skills)?)?;
        let cities = validate_cities(read(CITIES_FILE, sources.cities)?)?;

        Ok(GameContent {
            cards,
            standard_deck,
            replacements,
            dummy_crystals,
            tiles,
            enemies,
            fame_track,
            reputation_track,
            advanced_actions,
            spells,
            artifacts,
            units,
            ruins,
            skills,
            cities,
        })
    }

    /// Les couleurs des défenseurs d'une cité de cette couleur et de ce niveau, dans l'ordre des
    /// cercles de sa figurine (MKUE p. 7, 5.g ; cartes de cité ; spec 062). Niveau hors de 1 à 11 :
    /// aucun.
    pub fn city_garrison(&self, color: BasicColor, level: u8) -> &[EnemyColor] {
        let Some((_, levels)) = self.cities.iter().find(|(city, _)| *city == color) else {
            return &[];
        };
        usize::from(level)
            .checked_sub(1)
            .and_then(|index| levels.get(index))
            .map_or(&[], Vec::as_slice)
    }

    /// La définition d'une compétence (spec 051).
    pub fn skill(&self, skill: Skill) -> &SkillDefinition {
        // Invariant : `parse` exige une définition pour chaque compétence de `Skill::ALL`.
        &self.skills[&skill]
    }

    /// La définition d'une action avancée (spec 050).
    pub fn advanced_action(&'static self, card: AdvancedAction) -> CardDefinition {
        // Invariant : `parse` refuse un contenu où une carte n'a pas de définition.
        self.advanced_actions[&card].definition()
    }

    /// Numéro n/28 imprimé sur une action avancée (spec 050).
    pub fn advanced_action_number(&self, card: AdvancedAction) -> u8 {
        self.advanced_actions[&card].number
    }

    /// La définition d'un sort : sa couleur, effets non codés jusqu'à la spec 054.
    pub fn spell(&'static self, card: Spell) -> CardDefinition {
        self.spells[&card].definition()
    }

    /// Les effets d'un artefact, non codés jusqu'à la spec 055 ; il n'a pas de couleur.
    pub fn artifact(&'static self, card: Artifact) -> (BasicEffect, BasicEffect) {
        let entry = &self.artifacts[&card];
        (
            entry.basic.to_effect(Cost::None),
            entry.strong.to_effect(Cost::None),
        )
    }

    /// Les valeurs imprimées d'une unité (spec 050).
    pub fn unit(&self, unit: Unit) -> &UnitDefinition {
        // Invariant : `parse` refuse un contenu où une unité n'a pas de définition.
        &self.units[&unit]
    }

    /// Les cartes d'une pioche d'unités, numérotées par exemplaire, dans l'ordre du fichier : la
    /// pioche avant mélange (MKUE p. 3).
    pub fn unit_cards(&self, rank: UnitRank) -> Vec<UnitCard> {
        Unit::ALL
            .into_iter()
            .filter(|unit| self.units[unit].rank == rank)
            .flat_map(|unit| {
                (1..=self.units[&unit].count).map(move |copy| UnitCard::new(unit, copy))
            })
            .collect()
    }

    /// La définition d'une carte. Les options pointent dans le contenu, d'où `'static`.
    pub fn card(&'static self, action: BasicAction) -> CardDefinition {
        // Invariant : `parse` refuse un contenu où une carte n'a pas de définition.
        let entry = &self.cards[&action];
        CardDefinition {
            color: entry.color,
            basic: entry.basic.to_effect(entry.cost),
            strong: entry.strong.to_effect(entry.cost),
            reclaim: entry.reclaim,
            healing: entry.healing,
        }
    }

    /// La piste de gloire et les jetons de niveau (spec 035).
    pub fn fame_track(&self) -> &FameTrack {
        &self.fame_track
    }

    /// La piste de réputation (spec 040).
    pub fn reputation_track(&self) -> &ReputationTrack {
        &self.reputation_track
    }

    /// Les cristaux du joueur fantôme de ce héros, un par cercle coloré (spec 061).
    pub fn dummy_crystals(&self, hero: Hero) -> Vec<BasicColor> {
        self.dummy_crystals.get(&hero).cloned().unwrap_or_default()
    }

    /// Deck de départ : 16 cartes, numérotées par exemplaire, dans l'ordre du deck standard (MKUE p. 3).
    pub fn starting_deck(&self, hero: Hero) -> Vec<CardId> {
        let replacements = self.replacements.get(&hero).map_or(&[][..], Vec::as_slice);
        let mut deck = Vec::new();
        for &(action, count) in &self.standard_deck {
            let replaced: Vec<BasicAction> = replacements
                .iter()
                .filter(|r| r.replaced == action)
                .map(|r| r.by)
                .collect();
            // `parse` garantit qu'on ne remplace pas plus d'exemplaires qu'il n'y en a.
            let kept = usize::from(count).saturating_sub(replaced.len());
            deck.extend((1..=kept).map(|copy| copy_of(action, copy)));
            deck.extend(replaced.into_iter().map(|by| copy_of(by, 1)));
        }
        deck
    }

    /// Cartes propres d'un héros, dans l'ordre du fichier (spec 065).
    pub fn own_cards(&self, hero: Hero) -> Vec<OwnCard> {
        self.replacements
            .get(&hero)
            .map_or(&[][..], Vec::as_slice)
            .iter()
            .map(|r| OwnCard {
                card: r.by,
                replaced: r.replaced,
            })
            .collect()
    }

    /// Les tuiles, dans l'ordre du fichier.
    pub fn tiles(&self) -> &[TileDefinition] {
        &self.tiles
    }

    /// Les valeurs d'un ennemi (spec 030).
    pub fn enemy(&self, enemy: Enemy) -> &EnemyDefinition {
        // Invariant : `parse` refuse un contenu où un ennemi n'a pas de définition.
        self.enemies
            .iter()
            .find(|(id, _)| *id == enemy)
            .map(|(_, definition)| definition)
            .unwrap_or_else(|| panic!("{enemy:?} sans définition"))
    }

    /// Les jetons de ruines, dans l'ordre du fichier : la pile avant mélange (MKUE p. 3 ;
    /// spec 045).
    pub fn ruin_tokens(&self) -> Vec<RuinToken> {
        self.ruins.iter().map(|(token, _)| *token).collect()
    }

    /// La définition d'un jeton de ruines (spec 045).
    pub fn ruin_token(&self, token: RuinToken) -> &RuinTokenDefinition {
        // Invariant : `parse` refuse un contenu où un jeton n'a pas de définition.
        self.ruins
            .iter()
            .find(|(id, _)| *id == token)
            .map(|(_, definition)| definition)
            .unwrap_or_else(|| panic!("{token:?} sans définition"))
    }

    /// Tous les jetons d'une couleur, numérotés par exemplaire, dans l'ordre du fichier : la pile
    /// avant mélange (MKUE p. 3).
    pub fn enemy_tokens(&self, color: EnemyColor) -> Vec<EnemyToken> {
        self.enemies
            .iter()
            .filter(|(_, definition)| definition.color == color)
            .flat_map(|(enemy, definition)| {
                (1..=definition.count).map(|copy| EnemyToken::new(*enemy, copy))
            })
            .collect()
    }
}

fn copy_of(action: BasicAction, copy: usize) -> CardId {
    // Au plus 2 exemplaires d'une carte au départ : la conversion ne peut pas déborder.
    CardId::new(DeedCard::Basic(action), copy as u16)
}

fn read<'a, T: Deserialize<'a>>(file: &'static str, text: &'a str) -> Result<T, ContentError> {
    ron::from_str(text).map_err(|error| ContentError::Syntax {
        file,
        line: error.span.start.line,
        column: error.span.start.col,
        message: error.code.to_string(),
    })
}

/// Refuse un identifiant en double, en gardant l'ordre du fichier.
fn unique<K: Ord + Copy + fmt::Debug, V>(
    file: &'static str,
    entries: Vec<(K, V)>,
) -> Result<Vec<(K, V)>, ContentError> {
    let mut seen = BTreeSet::new();
    for (key, _) in &entries {
        if !seen.insert(*key) {
            return Err(ContentError::Duplicate {
                file,
                id: format!("{key:?}"),
            });
        }
    }
    Ok(entries)
}

/// Chaque identifiant attendu a exactement une définition.
fn exhaustive<K: Ord + Copy + fmt::Debug, V>(
    file: &'static str,
    entries: Vec<(K, V)>,
    expected: impl IntoIterator<Item = K>,
) -> Result<BTreeMap<K, V>, ContentError> {
    let map: BTreeMap<K, V> = unique(file, entries)?.into_iter().collect();
    match expected.into_iter().find(|key| !map.contains_key(key)) {
        Some(key) => Err(ContentError::Missing {
            file,
            id: format!("{key:?}"),
        }),
        None => Ok(map),
    }
}

// --- Cartes ---

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CardEntry {
    color: BasicColor,
    #[serde(default = "no_cost")]
    cost: Cost,
    basic: EffectEntry,
    strong: EffectEntry,
    /// Reprise en main à la fin du tour (spec 025).
    #[serde(default)]
    reclaim: bool,
    /// Carte de soin, jamais jouée en combat (spec 034).
    #[serde(default)]
    healing: bool,
}

fn no_cost() -> Cost {
    Cost::None
}

impl CardEntry {
    fn validate(&self, card: BasicAction) -> Result<(), ContentError> {
        validate_effects(
            BASIC_ACTIONS_FILE,
            &format!("{card:?}"),
            &self.basic,
            &self.strong,
        )
    }
}

/// Bornes du moteur : au moins une option, au moins 1 point (spec 029, L5) ; un mana à payer en
/// plus seulement dans l'effet de base (spec 025).
fn validate_effects(
    file: &'static str,
    card: &str,
    basic: &EffectEntry,
    strong: &EffectEntry,
) -> Result<(), ContentError> {
    let invalid = |reason: &str| {
        Err(ContentError::InvalidEffect {
            file,
            card: card.to_string(),
            reason: reason.to_string(),
        })
    };
    for effect in [basic, strong] {
        let EffectEntry::Options(OptionsEntry {
            effects: options, ..
        }) = effect
        else {
            continue;
        };
        if options.is_empty() {
            return invalid("aucune option");
        }
        if options.iter().any(|option| option.points() == Some(0)) {
            return invalid("0 point");
        }
        // Spec 034 : au moins 1 point de soin, au moins une carte piochée.
        if options
            .iter()
            .any(|option| matches!(option, Effect::Heal(0) | Effect::DrawCards(0)))
        {
            return invalid("soin ou pioche de 0");
        }
        // Spec 025 : un ou deux dés mis (`CardPlay::Strong::set_dice`), au moins un jeton.
        // Spec 025 : un bonus de Concentration d'au moins 1.
        if options.contains(&Effect::Boost(0)) {
            return invalid("bonus de 0");
        }
        if options.iter().any(|option| {
                matches!(option, Effect::SetDice { dice, tokens } if !(1..=2).contains(dice) || *tokens == 0)
            }) {
                return invalid("dés mis sur une couleur : 1 ou 2 dés, au moins un jeton");
            }
    }
    if let EffectEntry::Options(OptionsEntry {
        effects: options, ..
    }) = strong
        && options.iter().any(|option| option.mana_cost().is_some())
    {
        return invalid("mana à payer en plus dans l'effet renforcé");
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
enum EffectEntry {
    Options(#[serde(deserialize_with = "options_from_text")] OptionsEntry),
    Unsupported,
}

impl EffectEntry {
    fn to_effect(&'static self, cost: Cost) -> BasicEffect {
        match self {
            EffectEntry::Options(OptionsEntry {
                effects,
                modifiers,
                mana,
            }) => BasicEffect::Options {
                options: effects,
                cost,
                modifiers,
                mana,
            },
            EffectEntry::Unsupported => BasicEffect::Unsupported,
        }
    }
}

/// Forme d'un effet dans les fichiers, proche du texte des cartes (« Ranged Attack 3 »).
#[derive(Deserialize)]
enum EffectText {
    Move(u8),
    Influence(u8),
    Attack(u8),
    RangedAttack(u8),
    RangedFireAttack(u8),
    RangedIceAttack(u8),
    SiegeAttack(u8),
    FireAttack(u8),
    IceAttack(u8),
    ColdFireAttack(u8),
    SiegeFireAttack(u8),
    SiegeIceAttack(u8),
    Block(u8),
    FireBlock(u8),
    IceBlock(u8),
    ColdFireBlock(u8),
    /// Parade qui compte double contre Vif (spec 042).
    SwiftBlock(u8),
    /// Rendre disponible une unité de ce niveau au plus (spec 042).
    ReadyUnit(u8),
    /// Un jeton et un cristal de cette couleur (spec 042).
    TokenAndCrystal(BasicColor),
    IceBlockPerAbility(u8),
    GainToken(BasicColor),
    GainCrystal(BasicColor),
    PayForCrystal(BasicColor),
    ExtraDie,
    ExtraDieBlackAnyColor,
    SetDice {
        dice: u8,
        tokens: u8,
    },
    Boost(u8),
    Heal(u8),
    DrawCards(u8),
    InfluenceReputation {
        amount: u8,
        reputation: i8,
    },
    InfluenceInInteraction {
        amount: u8,
        fame: u8,
        #[serde(default)]
        reputation: i8,
    },
    /// Un effet de points et les modificateurs qu'il pose (spec 052, ADR 0008) :
    /// `With(Move(2), [TerrainCost(terrains: All, change: Reduce(by: 1, min: 2))])`.
    With(Box<EffectText>, Vec<ModifierKind>),
    /// Une option qui coûte un mana de cette couleur en plus (Magie pure, spec 053) :
    /// `Paying(Green, Move(4))`.
    Paying(BasicColor, Box<EffectText>),
    /// Aucun point, seulement les modificateurs de `With` (spec 053).
    Nothing,
    AttackWithWound(u8),
    /// Spec 054 : Tempête de feu, Blizzard.
    SiegeFireAttackWithWound(u8),
    SiegeIceAttackWithWound(u8),
    /// Spec 054 : Déluge de flammes.
    FireAttackPerEnemy {
        amount: u8,
        per: u8,
    },
    FireBlockPerEnemy {
        amount: u8,
        per: u8,
    },
    DestroyEnemy,
    CastFromOffer,
    MoveAndHeal {
        movement: u8,
        heal: u8,
    },
    HealAndReady {
        heal: u8,
        level: u8,
    },
    InfluencePerWound {
        base: u8,
        per: u8,
    },
    /// Spec 056 : effets des actions avancées de mana et de cartes.
    WoundCrystalAndToken {
        crystal: BasicColor,
        token: ManaColor,
    },
    WoundAndTokens(u8),
    CrystalsFromDestroyed(u8),
    CrystalsOtherColors,
    GainOwnedCrystal(BasicColor),
    CrystalFromDie,
    RerollSource,
    Repeat(u8),
    GainSpell(BasicColor),
    TrainFromOffer(CardDestination),
    /// Spec 057 : sorts de déplacement, d'unités et de mana.
    WoundsPerCrystal(BasicColor),
    OthersOnly,
    HealByTerrain {
        heal: u8,
        forest: u8,
    },
    HealAndReadyLevels {
        heal: u8,
        forest_heal: u8,
        levels: u8,
        forest_levels: u8,
    },
    ReadyAnyUnit {
        heal: bool,
    },
    UndergroundMove {
        assault: bool,
    },
    FlyMove {
        max: u8,
    },
    DoesNotAttackPerMove,
    DiscardToDeck {
        random: bool,
    },
    ClaimDie {
        each_turn: bool,
    },
    /// Spec 055 : artefacts.
    BannerAssigned,
    InfluenceAndFame {
        amount: u8,
        fame: u8,
    },
    AmuletOfSun {
        tokens: u8,
    },
    AmuletOfDarkness {
        tokens: u8,
    },
    TokenCrystalAndFame(BasicColor),
    CrystalAndToken {
        crystal: BasicColor,
        token: ManaColor,
    },
    DrawAndToken {
        cards: u8,
        token: BasicColor,
    },
    HealUnit,
    CrystalColorInfluence,
    MovePerUnit {
        max: u8,
    },
    DiscardWoundToDraw,
    ThrowWounds(u8),
    Flight {
        spaces: u8,
        cost: u8,
    },
    ResistanceBreak,
    SourceOpening,
    CalmWeather,
    ManaOverload,
    CancelAttacks(u8),
    ReadyAllUnits,
    SwordDiscard,
    HornOfWrath {
        extra: u8,
    },
    GemPouchDice,
    TokensEachColorAndTime,
    SpellAndCrystalFromOffer,
}

/// Les options d'un effet de carte : leurs effets de points, et les modificateurs que chacune pose
/// (spec 052), vides quand aucune n'en pose.
#[derive(Debug)]
struct OptionsEntry {
    effects: Vec<Effect>,
    modifiers: Vec<Vec<ModifierKind>>,
    /// Le mana en plus de chaque option (spec 053), vide quand aucune n'en coûte.
    mana: Vec<Option<BasicColor>>,
}

/// Les options d'un effet, écrites dans la forme des fichiers de contenu. `Effect` garde sa forme
/// complète pour les sauvegardes (spec 024).
fn options_from_text<'de, D: Deserializer<'de>>(deserializer: D) -> Result<OptionsEntry, D::Error> {
    let mut effects = Vec::new();
    let mut modifiers = Vec::new();
    let mut mana = Vec::new();
    for text in Vec::<EffectText>::deserialize(deserializer)? {
        let (paid, text) = text.into_paid();
        let (effect, kinds) = text.into_option();
        effects.push(effect);
        modifiers.push(kinds);
        mana.push(paid);
    }
    if modifiers.iter().all(Vec::is_empty) {
        modifiers.clear();
    }
    if mana.iter().all(Option::is_none) {
        mana.clear();
    }
    Ok(OptionsEntry {
        effects,
        modifiers,
        mana,
    })
}

impl EffectText {
    /// Le mana en plus de l'option (`Paying`, spec 053) et le reste de sa forme.
    fn into_paid(self) -> (Option<BasicColor>, EffectText) {
        match self {
            EffectText::Paying(color, text) => (Some(color), *text),
            other => (None, other),
        }
    }

    /// L'effet de points de l'option et les modificateurs qu'elle pose (spec 052, ADR 0008).
    fn into_option(self) -> (Effect, Vec<ModifierKind>) {
        match self {
            EffectText::With(effect, modifiers) => {
                let (effect, mut inner) = effect.into_option();
                inner.extend(modifiers);
                (effect, inner)
            }
            other => (other.into_effect(), Vec::new()),
        }
    }

    fn into_effect(self) -> Effect {
        let attack = |amount, element, kind| Effect::Attack {
            amount,
            element,
            kind,
        };
        let block = |amount, element| Effect::Block { amount, element };
        match self {
            EffectText::Move(amount) => Effect::Move(amount),
            EffectText::Influence(amount) => Effect::Influence(amount),
            EffectText::Attack(amount) => attack(amount, Element::Physical, AttackKind::Melee),
            EffectText::RangedAttack(amount) => {
                attack(amount, Element::Physical, AttackKind::Ranged)
            }
            EffectText::RangedFireAttack(amount) => {
                attack(amount, Element::Fire, AttackKind::Ranged)
            }
            EffectText::RangedIceAttack(amount) => attack(amount, Element::Ice, AttackKind::Ranged),
            EffectText::SiegeAttack(amount) => attack(amount, Element::Physical, AttackKind::Siege),
            EffectText::FireAttack(amount) => attack(amount, Element::Fire, AttackKind::Melee),
            EffectText::IceAttack(amount) => attack(amount, Element::Ice, AttackKind::Melee),
            EffectText::ColdFireAttack(amount) => {
                attack(amount, Element::ColdFire, AttackKind::Melee)
            }
            EffectText::SiegeFireAttack(amount) => attack(amount, Element::Fire, AttackKind::Siege),
            EffectText::SiegeIceAttack(amount) => attack(amount, Element::Ice, AttackKind::Siege),
            EffectText::ColdFireBlock(amount) => block(amount, Element::ColdFire),
            EffectText::SwiftBlock(amount) => Effect::SwiftBlock(amount),
            EffectText::ReadyUnit(level) => Effect::ReadyUnit(level),
            EffectText::TokenAndCrystal(color) => Effect::TokenAndCrystal(color),
            EffectText::Block(amount) => block(amount, Element::Physical),
            EffectText::FireBlock(amount) => block(amount, Element::Fire),
            EffectText::IceBlock(amount) => block(amount, Element::Ice),
            EffectText::IceBlockPerAbility(amount) => Effect::IceBlockPerAbility(amount),
            EffectText::GainToken(color) => Effect::GainToken(color),
            EffectText::GainCrystal(color) => Effect::GainCrystal(color),
            EffectText::PayForCrystal(color) => Effect::PayForCrystal(color),
            EffectText::With(effect, _) | EffectText::Paying(_, effect) => effect.into_effect(),
            EffectText::Nothing => Effect::Nothing,
            EffectText::AttackWithWound(amount) => Effect::AttackWithWound {
                amount,
                element: Element::Physical,
                kind: AttackKind::Melee,
            },
            EffectText::SiegeFireAttackWithWound(amount) => Effect::AttackWithWound {
                amount,
                element: Element::Fire,
                kind: AttackKind::Siege,
            },
            EffectText::SiegeIceAttackWithWound(amount) => Effect::AttackWithWound {
                amount,
                element: Element::Ice,
                kind: AttackKind::Siege,
            },
            EffectText::FireAttackPerEnemy { amount, per } => Effect::AttackPerEnemy {
                amount,
                element: Element::Fire,
                per,
            },
            EffectText::FireBlockPerEnemy { amount, per } => Effect::BlockPerEnemy {
                amount,
                element: Element::Fire,
                per,
            },
            EffectText::DestroyEnemy => Effect::DestroyEnemy,
            EffectText::CastFromOffer => Effect::CastFromOffer,
            EffectText::MoveAndHeal { movement, heal } => Effect::MoveAndHeal { movement, heal },
            EffectText::HealAndReady { heal, level } => Effect::HealAndReady { heal, level },
            EffectText::InfluencePerWound { base, per } => Effect::InfluencePerWound { base, per },
            EffectText::WoundCrystalAndToken { crystal, token } => {
                Effect::WoundCrystalAndToken { crystal, token }
            }
            EffectText::WoundAndTokens(count) => Effect::WoundAndTokens(count),
            EffectText::CrystalsFromDestroyed(count) => Effect::CrystalsFromDestroyed(count),
            EffectText::CrystalsOtherColors => Effect::CrystalsOtherColors,
            EffectText::GainOwnedCrystal(color) => Effect::GainOwnedCrystal(color),
            EffectText::CrystalFromDie => Effect::CrystalFromDie,
            EffectText::RerollSource => Effect::RerollSource,
            EffectText::Repeat(times) => Effect::Repeat(times),
            EffectText::GainSpell(color) => Effect::GainSpell(color),
            EffectText::TrainFromOffer(to) => Effect::TrainFromOffer(to),
            EffectText::WoundsPerCrystal(color) => Effect::WoundsPerCrystal(color),
            EffectText::OthersOnly => Effect::OthersOnly,
            EffectText::HealByTerrain { heal, forest } => Effect::HealByTerrain { heal, forest },
            EffectText::HealAndReadyLevels {
                heal,
                forest_heal,
                levels,
                forest_levels,
            } => Effect::HealAndReadyLevels {
                heal,
                forest_heal,
                levels,
                forest_levels,
            },
            EffectText::ReadyAnyUnit { heal } => Effect::ReadyAnyUnit { heal },
            EffectText::UndergroundMove { assault } => Effect::UndergroundMove { assault },
            EffectText::FlyMove { max } => Effect::FlyMove { max },
            EffectText::DoesNotAttackPerMove => Effect::DoesNotAttackPerMove,
            EffectText::DiscardToDeck { random } => Effect::DiscardToDeck { random },
            EffectText::ClaimDie { each_turn } => Effect::ClaimDie { each_turn },
            EffectText::BannerAssigned => Effect::BannerAssigned,
            EffectText::AmuletOfSun { tokens } => Effect::AmuletOfSun { tokens },
            EffectText::AmuletOfDarkness { tokens } => Effect::AmuletOfDarkness { tokens },
            EffectText::TokenCrystalAndFame(color) => Effect::TokenCrystalAndFame(color),
            EffectText::CrystalAndToken { crystal, token } => {
                Effect::CrystalAndToken { crystal, token }
            }
            EffectText::DrawAndToken { cards, token } => Effect::DrawAndToken { cards, token },
            EffectText::HealUnit => Effect::HealUnit,
            EffectText::CrystalColorInfluence => Effect::CrystalColorInfluence,
            EffectText::MovePerUnit { max } => Effect::MovePerUnit { max },
            EffectText::DiscardWoundToDraw => Effect::DiscardWoundToDraw,
            EffectText::ThrowWounds(count) => Effect::ThrowWounds(count),
            EffectText::Flight { spaces, cost } => Effect::Flight { spaces, cost },
            EffectText::ResistanceBreak => Effect::ResistanceBreak,
            EffectText::SourceOpening => Effect::SourceOpening,
            EffectText::CalmWeather => Effect::CalmWeather,
            EffectText::ManaOverload => Effect::ManaOverload,
            EffectText::CancelAttacks(count) => Effect::CancelAttacks(count),
            EffectText::ReadyAllUnits => Effect::ReadyAllUnits,
            EffectText::SwordDiscard => Effect::SwordDiscard,
            EffectText::HornOfWrath { extra } => Effect::HornOfWrath { extra },
            EffectText::GemPouchDice => Effect::GemPouchDice,
            EffectText::TokensEachColorAndTime => Effect::TokensEachColorAndTime,
            EffectText::SpellAndCrystalFromOffer => Effect::SpellAndCrystalFromOffer,
            EffectText::InfluenceAndFame { amount, fame } => {
                Effect::InfluenceAndFame { amount, fame }
            }
            EffectText::ExtraDie => Effect::ExtraDie,
            EffectText::ExtraDieBlackAnyColor => Effect::ExtraDieBlackAnyColor,
            EffectText::SetDice { dice, tokens } => Effect::SetDice { dice, tokens },
            EffectText::Boost(bonus) => Effect::Boost(bonus),
            EffectText::Heal(amount) => Effect::Heal(amount),
            EffectText::DrawCards(count) => Effect::DrawCards(count),
            EffectText::InfluenceReputation { amount, reputation } => {
                Effect::InfluenceReputation { amount, reputation }
            }
            EffectText::InfluenceInInteraction {
                amount,
                fame,
                reputation,
            } => Effect::InfluenceInInteraction {
                amount,
                fame,
                reputation,
            },
        }
    }
}

// --- Héros ---

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HeroesFile {
    standard_deck: Entries<BasicAction, u8>,
    heroes: Entries<Hero, Vec<Replacement>>,
    dummy_crystals: Entries<Hero, Vec<BasicColor>>,
}

/// Une carte propre au héros et la carte du deck standard qu'elle remplace.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Replacement {
    replaced: BasicAction,
    by: BasicAction,
}

/// Spec 061 (MKUE p. 15) : trois cercles colorés par carte de héros.
fn validate_dummy_crystals(crystals: &BTreeMap<Hero, Vec<BasicColor>>) -> Result<(), ContentError> {
    match crystals.iter().find(|(_, colors)| colors.len() != 3) {
        Some((hero, colors)) => Err(ContentError::InvalidDeck {
            reason: format!(
                "{hero:?} a {} cristal(aux) de joueur fantôme au lieu de 3",
                colors.len()
            ),
        }),
        None => Ok(()),
    }
}

fn validate_decks(
    standard_deck: &[(BasicAction, u8)],
    replacements: &BTreeMap<Hero, Vec<Replacement>>,
) -> Result<(), ContentError> {
    let size: usize = standard_deck.iter().map(|(_, n)| usize::from(*n)).sum();
    if size != STARTING_DECK_SIZE {
        return Err(ContentError::InvalidDeck {
            reason: format!("le deck standard a {size} cartes au lieu de {STARTING_DECK_SIZE}"),
        });
    }
    for (hero, replacements) in replacements {
        for replacement in replacements {
            let count = standard_deck
                .iter()
                .find(|(action, _)| *action == replacement.replaced)
                .map_or(0, |(_, n)| usize::from(*n));
            let replaced = replacements
                .iter()
                .filter(|r| r.replaced == replacement.replaced)
                .count();
            if replaced > count {
                return Err(ContentError::InvalidDeck {
                    reason: format!(
                        "{hero:?} remplace {:?}, absente du deck standard ou en trop peu d'exemplaires",
                        replacement.replaced
                    ),
                });
            }
        }
    }
    Ok(())
}

// --- Tuiles ---

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TileEntry {
    id: TileId,
    center: SpaceEntry,
    sides: Sides,
}

/// Une case : `(terrain, élément)`.
#[derive(Clone, Copy, Deserialize)]
struct SpaceEntry(Terrain, Option<Feature>);

impl From<SpaceEntry> for SpaceContent {
    fn from(SpaceEntry(terrain, feature): SpaceEntry) -> Self {
        SpaceContent { terrain, feature }
    }
}

/// Les 21 tuiles du jeu de base : 2 faces de départ, 11 de campagne, 8 royaumes (MKUE p. 3).
fn base_game_tile_ids() -> impl Iterator<Item = TileId> {
    [TileId::StartingA, TileId::StartingB]
        .into_iter()
        .chain((1..=11).map(TileId::Countryside))
        .chain((1..=8).map(TileId::Core))
}

fn validate_tiles(entries: Vec<TileEntry>) -> Result<Vec<TileDefinition>, ContentError> {
    let entries = unique(TILES_FILE, entries.into_iter().map(|t| (t.id, t)).collect())?;
    let expected: BTreeSet<TileId> = base_game_tile_ids().collect();
    if let Some((id, _)) = entries.iter().find(|(id, _)| !expected.contains(id)) {
        return Err(ContentError::Unknown {
            file: TILES_FILE,
            id: format!("{id:?}"),
        });
    }
    let defined: BTreeSet<TileId> = entries.iter().map(|(id, _)| *id).collect();
    if let Some(id) = expected.iter().find(|id| !defined.contains(id)) {
        return Err(ContentError::Missing {
            file: TILES_FILE,
            id: format!("{id:?}"),
        });
    }
    Ok(entries
        .into_iter()
        .map(|(id, tile)| TileDefinition {
            id,
            center: tile.center.into(),
            sides: tile.sides.0.map(SpaceContent::from),
        })
        .collect())
}

// --- Lecture ---

/// Les six côtés d'une tuile, écrits en liste `[...]` (serde lit un tableau Rust comme un tuple `(...)`).
struct Sides([SpaceEntry; 6]);

impl<'de> Deserialize<'de> for Sides {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct SidesVisitor;

        impl<'de> Visitor<'de> for SidesVisitor {
            type Value = Sides;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("une liste de six cases")
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut sides = Vec::with_capacity(6);
                while let Some(side) = seq.next_element()? {
                    sides.push(side);
                }
                let count = sides.len();
                sides
                    .try_into()
                    .map(Sides)
                    .map_err(|_| A::Error::invalid_length(count, &self))
            }
        }

        deserializer.deserialize_seq(SidesVisitor)
    }
}

/// Une table RON `{ clé: valeur }` lue dans l'ordre du fichier, doublons compris, pour pouvoir les refuser.
struct Entries<K, V>(Vec<(K, V)>);

impl<'de, K: Deserialize<'de>, V: Deserialize<'de>> Deserialize<'de> for Entries<K, V> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct EntriesVisitor<K, V>(PhantomData<(K, V)>);

        impl<'de, K: Deserialize<'de>, V: Deserialize<'de>> Visitor<'de> for EntriesVisitor<K, V> {
            type Value = Entries<K, V>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("une table { identifiant: définition }")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut entries = Vec::new();
                while let Some(entry) = map.next_entry()? {
                    entries.push(entry);
                }
                Ok(Entries(entries))
            }
        }

        deserializer.deserialize_map(EntriesVisitor(PhantomData))
    }
}

// --- Ennemis ---

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EnemyEntry {
    id: Enemy,
    color: EnemyColor,
    count: u8,
    armor: u8,
    attack: EnemyAttack,
    abilities: Vec<EnemyAbility>,
    resistances: Vec<Resistance>,
    fame: u8,
}

fn validate_enemies(
    entries: Vec<EnemyEntry>,
) -> Result<Vec<(Enemy, EnemyDefinition)>, ContentError> {
    let entries = unique(
        ENEMIES_FILE,
        entries.into_iter().map(|e| (e.id, e)).collect(),
    )?;
    if let Some(enemy) = Enemy::ALL
        .into_iter()
        .find(|enemy| !entries.iter().any(|(id, _)| id == enemy))
    {
        return Err(ContentError::Missing {
            file: ENEMIES_FILE,
            id: format!("{enemy:?}"),
        });
    }
    entries
        .into_iter()
        .map(|(enemy, entry)| {
            if entry.count == 0 {
                return Err(ContentError::NoEnemyToken { enemy });
            }
            Ok((
                enemy,
                EnemyDefinition {
                    color: entry.color,
                    count: entry.count,
                    armor: entry.armor,
                    attack: entry.attack,
                    abilities: entry.abilities,
                    resistances: entry.resistances,
                    fame: entry.fame,
                },
            ))
        })
        .collect()
}

// --- Jetons de ruines (spec 045) ---

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RuinEntry {
    id: RuinToken,
    kind: RuinKind,
}

/// Un exemplaire de chaque jeton, dans l'ordre du fichier ; un jeton d'ennemis a des couleurs et
/// une récompense.
fn validate_ruins(
    entries: Vec<RuinEntry>,
) -> Result<Vec<(RuinToken, RuinTokenDefinition)>, ContentError> {
    let entries = unique(
        RUINS_FILE,
        entries.into_iter().map(|e| (e.id, e.kind)).collect(),
    )?;
    if let Some(token) = RuinToken::ALL
        .into_iter()
        .find(|token| !entries.iter().any(|(id, _)| id == token))
    {
        return Err(ContentError::Missing {
            file: RUINS_FILE,
            id: format!("{token:?}"),
        });
    }
    if let Some((token, _)) = entries.iter().find(|(_, kind)| {
        matches!(kind, RuinKind::Enemies { colors, rewards } if colors.is_empty() || rewards.is_empty())
    }) {
        return Err(ContentError::InvalidRuin { token: *token });
    }
    Ok(entries
        .into_iter()
        .map(|(token, kind)| (token, RuinTokenDefinition { kind }))
        .collect())
}

// --- Cités (spec 062) ---

/// Le nombre de niveaux d'une cité (MKUE p. 18, *City Levels*).
const CITY_LEVELS: u8 = 11;

/// Pour chaque couleur de cité, les couleurs des défenseurs de chaque niveau.
type CityGarrisons = Vec<(BasicColor, Vec<Vec<EnemyColor>>)>;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CityEntry {
    color: BasicColor,
    levels: Vec<Vec<EnemyColor>>,
}

/// Chaque couleur de cité une fois, onze niveaux, des jetons gris, violets, marron ou blancs.
fn validate_cities(entries: Vec<CityEntry>) -> Result<CityGarrisons, ContentError> {
    let invalid = |reason: String| ContentError::InvalidCities { reason };
    let mut cities: CityGarrisons = Vec::new();
    for entry in entries {
        if cities.iter().any(|(color, _)| *color == entry.color) {
            return Err(ContentError::Duplicate {
                file: CITIES_FILE,
                id: format!("{:?}", entry.color),
            });
        }
        if entry.levels.len() != usize::from(CITY_LEVELS) {
            return Err(invalid(format!(
                "la cité {:?} a {} niveaux au lieu de {CITY_LEVELS}",
                entry.color,
                entry.levels.len()
            )));
        }
        let defender = |color: &EnemyColor| {
            matches!(
                color,
                EnemyColor::Grey | EnemyColor::Violet | EnemyColor::Brown | EnemyColor::White
            )
        };
        if entry.levels.iter().flatten().any(|color| !defender(color)) {
            return Err(invalid(format!(
                "la cité {:?} a un défenseur vert ou rouge",
                entry.color
            )));
        }
        cities.push((entry.color, entry.levels));
    }
    for color in [
        BasicColor::Red,
        BasicColor::Blue,
        BasicColor::White,
        BasicColor::Green,
    ] {
        if !cities.iter().any(|(city, _)| *city == color) {
            return Err(ContentError::Missing {
                file: CITIES_FILE,
                id: format!("{color:?}"),
            });
        }
    }
    Ok(cities)
}

// --- Niveaux ---

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LevelEntry {
    fame: u32,
    armor: u8,
    hand_limit: u8,
    command_tokens: u8,
    reward: LevelReward,
}

/// Dix niveaux, seuils croissants à partir de 0, valeurs du jeton qui ne baissent pas (spec 035).
fn validate_levels(entries: Vec<LevelEntry>) -> Result<FameTrack, ContentError> {
    let invalid = |reason: String| ContentError::InvalidLevels { reason };
    if entries.len() != usize::from(MAX_LEVEL) {
        return Err(invalid(format!(
            "{} niveaux au lieu de {MAX_LEVEL}",
            entries.len()
        )));
    }
    if entries.first().is_some_and(|first| first.fame != 0) {
        return Err(invalid("le niveau 1 doit commencer à 0 de gloire".into()));
    }
    for (index, pair) in entries.windows(2).enumerate() {
        let (before, after) = (&pair[0], &pair[1]);
        let level = index + 2;
        if after.fame <= before.fame {
            return Err(invalid(format!(
                "le seuil du niveau {level} ne dépasse pas celui du niveau d'avant"
            )));
        }
        if after.armor < before.armor
            || after.hand_limit < before.hand_limit
            || after.command_tokens < before.command_tokens
        {
            return Err(invalid(format!(
                "le jeton du niveau {level} baisse une valeur du niveau d'avant"
            )));
        }
    }
    Ok(FameTrack::new(
        entries
            .into_iter()
            .map(|entry| LevelDefinition {
                fame: entry.fame,
                token: LevelToken {
                    armor: entry.armor,
                    hand_limit: entry.hand_limit,
                    command_tokens: entry.command_tokens,
                },
                reward: entry.reward,
            })
            .collect(),
    ))
}

// --- Réputation (spec 040) ---

/// Une case de la piste de réputation : la case X, ou un bonus (négatif : malus).
#[derive(Deserialize)]
enum ReputationEntry {
    X,
    Bonus(i8),
}

/// Quinze cases, de −7 à +7, la case X au bas et elle seule.
fn validate_reputation(entries: Vec<ReputationEntry>) -> Result<ReputationTrack, ContentError> {
    let invalid = |reason: &str| ContentError::InvalidReputation {
        reason: reason.to_string(),
    };
    let spaces = usize::from((Reputation::MAX - Reputation::MIN).unsigned_abs()) + 1;
    if entries.len() != spaces {
        return Err(invalid(&format!(
            "{} cases au lieu de {spaces}",
            entries.len()
        )));
    }
    let bonuses: Vec<Option<i8>> = entries
        .into_iter()
        .map(|entry| match entry {
            ReputationEntry::X => None,
            ReputationEntry::Bonus(bonus) => Some(bonus),
        })
        .collect();
    if bonuses.first() != Some(&None) || bonuses[1..].iter().any(Option::is_none) {
        return Err(invalid("la case X est au bas de la piste, et elle seule"));
    }
    Ok(ReputationTrack::new(bonuses))
}

// --- Actions avancées, sorts, artefacts, unités (spec 050) ---

/// Une action avancée ou un sort : numéro imprimé, couleur, deux effets.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct NumberedCardEntry {
    number: u8,
    color: BasicColor,
    /// Ce qu'il faut payer en plus pour les deux effets (Décomposition, Entraînement : détruire une
    /// carte action, spec 056).
    #[serde(default = "no_cost")]
    cost: Cost,
    basic: EffectEntry,
    strong: EffectEntry,
    /// Carte de soin, aucun effet en combat (Régénération, spec 053, Q4).
    #[serde(default)]
    healing: bool,
}

impl NumberedCardEntry {
    fn definition(&'static self) -> CardDefinition {
        CardDefinition {
            color: self.color,
            basic: self.basic.to_effect(self.cost),
            strong: self.strong.to_effect(self.cost),
            reclaim: false,
            healing: self.healing,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ArtifactEntry {
    number: u8,
    basic: EffectEntry,
    strong: EffectEntry,
}

/// Chaque carte a une définition, des effets valides, et un numéro unique de 1 au nombre de
/// cartes de la famille.
fn validate_numbered<K: Ord + Copy + fmt::Debug, const N: usize>(
    file: &'static str,
    entries: Entries<K, NumberedCardEntry>,
    expected: [K; N],
) -> Result<BTreeMap<K, NumberedCardEntry>, ContentError> {
    let cards = exhaustive(file, entries.0, expected)?;
    for (card, entry) in &cards {
        validate_effects(file, &format!("{card:?}"), &entry.basic, &entry.strong)?;
    }
    unique_numbers(file, cards.values().map(|entry| entry.number), N)?;
    Ok(cards)
}

fn unique_numbers(
    file: &'static str,
    numbers: impl Iterator<Item = u8>,
    count: usize,
) -> Result<(), ContentError> {
    let mut seen = BTreeSet::new();
    for number in numbers {
        if number == 0 || usize::from(number) > count || !seen.insert(number) {
            return Err(ContentError::InvalidNumber { file, number });
        }
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UnitEntry {
    rank: UnitRank,
    count: u8,
    influence: u8,
    level: u8,
    armor: u8,
    resistances: Vec<Resistance>,
    recruit: Vec<RecruitSite>,
    abilities: Vec<AbilityEntry>,
}

/// Forme d'une capacité d'unité dans `units.ron` (spec 042).
#[derive(Deserialize)]
enum AbilityEntry {
    /// Une option au choix, sans mana.
    Options(Vec<EffectText>),
    /// Une option au choix, après un mana de cette couleur.
    Mana(BasicColor, Vec<EffectText>),
    /// Une option au choix ; l'unité reçoit une Blessure (Épéistes d'Utem).
    Wounding(Vec<EffectText>),
    /// Une option au choix, chacune avec le mana payé en plus (Mages d'Altem).
    Priced(Vec<(EffectText, Vec<BasicColor>)>),
    /// Une option au choix, après un mana noir, de nuit seulement (Mages d'Altem, spec 053).
    BlackMana(Vec<EffectText>),
    /// Deux jetons de mana de n'importe quelles couleurs : une option par paire, dans l'ordre de
    /// `ManaColor::ALL` (Mages d'Altem ; Compendium, *ALTEM MAGES* Q3).
    TwoTokensAnyColors,
    /// Pas encore jouée (spec 042, Q2), sans mana ou avec le mana imprimé.
    Unsupported,
    UnsupportedMana(ManaColor),
}

impl AbilityEntry {
    fn into_ability(self) -> UnitAbility {
        let options = |effects: Vec<EffectText>| {
            effects
                .into_iter()
                .map(|text| {
                    let (effect, modifiers) = text.into_option();
                    AbilityOption {
                        modifiers,
                        ..AbilityOption::new(effect)
                    }
                })
                .collect()
        };
        let supported = |mana, options, wounds_unit| UnitAbility {
            mana,
            effect: AbilityEffect::Options {
                options,
                wounds_unit,
            },
        };
        match self {
            AbilityEntry::Options(effects) => supported(None, options(effects), false),
            AbilityEntry::Mana(color, effects) => {
                supported(Some(ManaColor::Basic(color)), options(effects), false)
            }
            AbilityEntry::Wounding(effects) => supported(None, options(effects), true),
            AbilityEntry::BlackMana(effects) => {
                supported(Some(ManaColor::Black), options(effects), false)
            }
            AbilityEntry::Priced(priced) => supported(
                None,
                priced
                    .into_iter()
                    .map(|(text, extra_mana)| {
                        let (effect, modifiers) = text.into_option();
                        AbilityOption {
                            effect,
                            extra_mana,
                            modifiers,
                        }
                    })
                    .collect(),
                false,
            ),
            AbilityEntry::TwoTokensAnyColors => {
                let mut pairs = Vec::new();
                for (index, &first) in ManaColor::ALL.iter().enumerate() {
                    for &second in &ManaColor::ALL[index..] {
                        pairs.push(AbilityOption::new(Effect::GainTokens(first, second)));
                    }
                }
                supported(None, pairs, false)
            }
            AbilityEntry::Unsupported => UnitAbility {
                mana: None,
                effect: AbilityEffect::Unsupported,
            },
            AbilityEntry::UnsupportedMana(color) => UnitAbility {
                mana: Some(color),
                effect: AbilityEffect::Unsupported,
            },
        }
    }
}

/// Chaque unité a une définition, un niveau de I à IV, au moins un exemplaire et un lieu de
/// recrutement ; 20 cartes par pioche (MKUE p. 3).
fn validate_units(
    entries: Entries<Unit, UnitEntry>,
) -> Result<BTreeMap<Unit, UnitDefinition>, ContentError> {
    let units = exhaustive(UNITS_FILE, entries.0, Unit::ALL)?;
    let invalid = |reason: String| ContentError::InvalidUnits { reason };
    for (unit, entry) in &units {
        if entry.count == 0 || !(1..=4).contains(&entry.level) || entry.recruit.is_empty() {
            return Err(invalid(format!(
                "{unit:?} : au moins un exemplaire, un niveau de I à IV et un lieu de recrutement"
            )));
        }
        // Spec 042 : au moins une capacité, et une option par capacité codée.
        let empty = |ability: &AbilityEntry| match ability {
            AbilityEntry::Options(effects)
            | AbilityEntry::Mana(_, effects)
            | AbilityEntry::BlackMana(effects)
            | AbilityEntry::Wounding(effects) => effects.is_empty(),
            AbilityEntry::Priced(priced) => priced.is_empty(),
            AbilityEntry::TwoTokensAnyColors
            | AbilityEntry::Unsupported
            | AbilityEntry::UnsupportedMana(_) => false,
        };
        if entry.abilities.is_empty() || entry.abilities.iter().any(empty) {
            return Err(invalid(format!(
                "{unit:?} : au moins une capacité, et une option par capacité"
            )));
        }
    }
    for rank in [UnitRank::Regular, UnitRank::Elite] {
        let cards: usize = units
            .values()
            .filter(|entry| entry.rank == rank)
            .map(|entry| usize::from(entry.count))
            .sum();
        if cards != UNITS_PER_DECK {
            return Err(invalid(format!(
                "la pioche {rank:?} a {cards} cartes au lieu de {UNITS_PER_DECK}"
            )));
        }
    }
    Ok(units
        .into_iter()
        .map(|(unit, entry)| {
            (
                unit,
                UnitDefinition {
                    rank: entry.rank,
                    count: entry.count,
                    influence: entry.influence,
                    level: entry.level,
                    armor: entry.armor,
                    resistances: entry.resistances,
                    recruit: entry.recruit,
                    abilities: entry
                        .abilities
                        .into_iter()
                        .map(AbilityEntry::into_ability)
                        .collect(),
                },
            )
        })
        .collect())
}

/// Forme d'une compétence dans `skills.ron` (spec 051).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SkillEntry {
    hero: Hero,
    usage: SkillUsage,
    #[serde(default)]
    cooperative: bool,
    #[serde(default)]
    out_of_combat: bool,
    effect: SkillEffectEntry,
}

#[derive(Deserialize)]
enum SkillEffectEntry {
    /// Une option au choix.
    Options(Vec<EffectText>),
    /// Une option au choix, chacune avec sa valeur de jour et de nuit.
    DayNight(Vec<(EffectText, EffectText)>),
    /// Une carte jouée de côté vaut plus (spec 058, groupe A) : valeurs codées en Rust.
    Sideways,
    /// Un mana payé autrement (spec 058, groupe B) : proposé dans les paiements.
    Payment,
    /// Un bonus porté par l'activation d'une unité (Charisme, spec 058).
    Leadership,
    /// Un effet permanent, sans utilisation (Loyauté sans faille, spec 058).
    Permanent,
    /// Second lot (spec 058).
    Unsupported,
}

/// Chaque compétence a une définition, avec au moins une option si elle est codée ; dix par héros,
/// dont une coopérative (MKUE p. 3 ; Compendium, *Skill Tokens* S5).
fn validate_skills(
    entries: Entries<Skill, SkillEntry>,
) -> Result<BTreeMap<Skill, SkillDefinition>, ContentError> {
    let skills = exhaustive(SKILLS_FILE, entries.0, Skill::ALL)?;
    let invalid = |skill: Skill, reason: &str| ContentError::InvalidEffect {
        file: SKILLS_FILE,
        card: format!("{skill:?}"),
        reason: reason.to_string(),
    };
    let mut definitions = BTreeMap::new();
    for (skill, entry) in skills {
        let effect = match entry.effect {
            SkillEffectEntry::Options(effects) => SkillEffect::Options(
                effects
                    .into_iter()
                    .map(|text| SkillOption {
                        day: text.into_effect(),
                        night: None,
                    })
                    .collect(),
            ),
            SkillEffectEntry::DayNight(effects) => SkillEffect::Options(
                effects
                    .into_iter()
                    .map(|(day, night)| SkillOption {
                        day: day.into_effect(),
                        night: Some(night.into_effect()),
                    })
                    .collect(),
            ),
            SkillEffectEntry::Sideways => SkillEffect::Sideways,
            SkillEffectEntry::Payment => SkillEffect::Payment,
            SkillEffectEntry::Leadership => SkillEffect::Leadership,
            SkillEffectEntry::Permanent => SkillEffect::Permanent,
            SkillEffectEntry::Unsupported => SkillEffect::Unsupported,
        };
        if effect == SkillEffect::Options(Vec::new()) {
            return Err(invalid(skill, "aucune option"));
        }
        definitions.insert(
            skill,
            SkillDefinition {
                hero: entry.hero,
                usage: entry.usage,
                cooperative: entry.cooperative,
                out_of_combat: entry.out_of_combat,
                effect,
            },
        );
    }
    for hero in Hero::ALL {
        let own: Vec<&SkillDefinition> = definitions
            .values()
            .filter(|definition| definition.hero == hero)
            .collect();
        let cooperative = own
            .iter()
            .filter(|definition| definition.cooperative)
            .count();
        if own.len() != 10 || cooperative != 1 {
            return Err(ContentError::InvalidSkills { hero });
        }
    }
    Ok(definitions)
}
