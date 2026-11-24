//! Moteur de règles : état, actions, événements et joueurs.
//! Pur : ni UI, ni fichiers, ni horloge, ni hasard non injecté.

mod action;
mod board;
mod card;
mod cards;
mod city;
mod combat;
mod config;
mod content;
mod crystal;
mod deck;
mod dummy;
mod enemy;
mod engine;
mod event;
mod exploration;
mod game_end;
mod hero;
mod ids;
mod level;
mod log;
mod mana;
mod modifier;
mod movement;
mod offer;
mod player;
mod reputation;
mod rng;
mod ruin;
mod runner;
mod scenario;
mod skill;
mod state;
mod tactic;
mod turn_fact;
mod unit;
mod view;

pub use action::{Action, ActionError, DamageTarget, HealingTarget, SkillTarget, WoundPile};
pub use board::{
    BasicColor, Direction, Feature, Hex, Map, MapError, Rampaging, Site, SpaceContent, Terrain,
    TileDeck, TileDeckCounts, TileDefinition, TileId, TileKind, TilePosition, base_game_tiles,
    tile,
};
pub use card::{
    ArtifactChoices, AttackKind, BasicAction, BasicEffect, BoostTarget, CardDefinition,
    CardDestination, CardPlay, CastTargets, Cost, DeedCard, DieSetting, Effect, Element,
    ManaPayment, PICKS, Picks, PlayedCard, PolarSource, STARTING_HAND_LIMIT, SidewaysEffect,
    SkillMana,
};
pub use cards::{AdvancedAction, Artifact, Spell};
pub use city::CityState;
pub use combat::{
    Assault, Combat, CombatEnemy, CombatPhase, CombatRecord, Efficiency, ElementPoints, InProgress,
    attack_efficiency, block_efficiency, cold_toughness_bonus,
};
pub use config::{ConfigError, GameConfig, GameRng};
pub use content::{ContentError, ContentSources, GameContent};
pub use crystal::{CRYSTAL_LIMIT, Inventory, mine_color};
pub use deck::DeedDeck;
pub use dummy::{DUMMY_COPY_OFFSET, DummyHeroChoice, DummyRule, DummyState, RoundOrder};
pub use enemy::{
    Enemy, EnemyAbility, EnemyAttack, EnemyColor, EnemyDefinition, EnemyToken, Resistance,
};
pub use engine::{
    ALTAR_FAME, CITY_ADVANCED_ACTION_COST, CITY_ARTIFACT_COST, CITY_ELITE_UNIT_COST, DecisionKind,
    MONASTERY_ADVANCED_ACTION_COST, MONASTERY_HEALING_COST, PendingDecision, SPELL_COST,
    VILLAGE_HEALING_COST, apply, bonds_cost, decide, end_turn_allowed, glade_wound_available,
    leadership_effect, legal_actions, new_game, pending_decision, simulate, wings_of_night_cost,
};
pub use event::{DestroyedFrom, EVENT_FORMAT_VERSION, Event, PlayerEvent, filter_event};
pub use exploration::{
    EXPLORE_COST, NextTile, Placement, free_positions_touching, next_tile, placement, removed_tiles,
};
pub use game_end::{
    Achievement, ConquestLine, ConquestProgress, ConquestScore, FinalScore, GameEndReason,
    LastTurn, conquest_progress, final_score,
};
pub use hero::{Hero, OwnCard};
pub use ids::{CardId, PlayerId};
pub use level::{FameTrack, LevelDefinition, LevelReward, LevelToken, MAX_LEVEL, level_token};
pub use log::{DecisionLog, DecisionRecord, EventLog, LoggedEvent};
pub use mana::{DICE_PER_TURN, ManaColor, basic_dice_needed, roll_die, roll_source, source_size};
pub use modifier::{
    ConvertedPoints, EnemyModifier, EnemyModifierKind, Modifier, ModifierDuration, ModifierKind,
    ModifierSource, PointPool, TerrainChange, Terrains,
};
pub use movement::{
    MoveCost, MoveRules, Route, TimeOfDay, is_safe_space, move_cost, provoked, reach, routes,
    routes_avoiding, routes_ending_in, steps,
};
pub use offer::{AdvancedActionSet, CommonCard, CommonDeck, Offer};
pub use player::Player;
pub use reputation::{Interaction, Reputation, ReputationTrack, reputation_bonus};
pub use rng::{GameRngs, RngPositions, RngUsage, UnknownRngUsage};
pub use ruin::{PlacedRuin, RuinKind, RuinReward, RuinToken, RuinTokenDefinition};
pub use runner::{GameRunner, ResumeError, RunnerError, StepOutcome, UndoError, UndoStatus};
pub use scenario::{
    CityLevels, CityRule, Columns, CountrysideOrder, FinalScoring, MAX_CITIES, MapShape, Scenario,
    ScenarioEnd, ScenarioError, ScenarioId, SpellSet, TileDeckRecipe,
};
pub use skill::{
    CenterAdvantage, CommonSkillView, OwnedSkill, Skill, SkillDefinition, SkillEffect, SkillOption,
    SkillState, SkillUsage, SkillView,
};
pub use state::{GameState, ManaChoice, Meditation};
pub use tactic::{
    Tactic, TacticOrder, TacticRemoval, TacticRule, TacticSlot, TacticState, TacticView,
};
pub use turn_fact::TurnFact;
pub use unit::{
    AbilityEffect, AbilityOption, OwnedUnit, RecruitSite, Unit, UnitAbility, UnitCard,
    UnitDefinition, UnitRank,
};
pub use view::ForcedWithdrawalPreview;
pub use view::{DummyView, PlayerView, view};
