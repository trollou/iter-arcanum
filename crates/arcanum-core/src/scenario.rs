//! Scénario : une valeur que la mise en place consulte, jamais du code de mise en place (spec 012).

use rand::seq::SliceRandom;
use thiserror::Error;

use crate::board::{TileId, TileKind, TilePosition, base_game_tiles};
use crate::{AdvancedActionSet, DummyRule, GameRng, TacticOrder, TacticRemoval, TacticRule};

/// Identifiant d'un scénario : sert à son nom (clé de libellé).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ScenarioId {
    /// Notre scénario d'entraînement, absent du livret.
    Sandbox,
    /// « Première reconnaissance » en solo (MKUE p. 18 ; spec 063).
    FirstReconnaissance,
    /// « Conquête en solitaire » (MKUE p. 19 ; spec 064).
    SoloConquest,
}

impl ScenarioId {
    /// Scénarios jouables, dans l'ordre de l'écran de nouvelle partie (spec 065).
    /// Spec 064, Q4 : l'ordre de la liste de l'écran de nouvelle partie.
    pub const ALL: [ScenarioId; 3] = [
        ScenarioId::FirstReconnaissance,
        ScenarioId::SoloConquest,
        ScenarioId::Sandbox,
    ];

    /// Clé de libellé du nom : arcanum-core ne manipule que des clés (ADR 0003).
    pub fn label_key(self) -> &'static str {
        match self {
            ScenarioId::Sandbox => "scenario.sandbox.name",
            ScenarioId::FirstReconnaissance => "scenario.first_reconnaissance.name",
            ScenarioId::SoloConquest => "scenario.solo_conquest.name",
        }
    }
}

/// Nombre de colonnes d'une forme ouverte (MKUE p. 14).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Columns {
    Three,
    Four,
    Five,
}

/// Forme du plateau de jeu (MKUE p. 14) : où une tuile peut être posée.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum MapShape {
    /// En coin : tuile de départ A, ouverture de 60° entre `A` et `B`.
    Wedge,
    /// En coin standard (MKUE p. 14, *Wedge Map* ; spec 064) : comme `Wedge`, plus le refus des
    /// tuiles royaume sur la côte, les emplacements `i = 0` ou `j = 0`.
    StandardWedge,
    /// Ouverte : tuile de départ B, ouverture de 120°, colonnes limitées.
    Open(Columns),
}

impl MapShape {
    /// Vrai si une tuile peut être posée à cet emplacement (pas derrière la côte, MKUE p. 7, 5.a).
    pub fn allows(self, position: TilePosition) -> bool {
        let (i, j) = position.lattice();
        match self {
            MapShape::Wedge | MapShape::StandardWedge => i >= 0 && j >= 0,
            MapShape::Open(columns) => {
                // Avec 4 colonnes, la colonne en plus est du côté de `A − B` (MKUE p. 14).
                let (lowest, highest) = match columns {
                    Columns::Three => (-1, 1),
                    Columns::Four => (-2, 1),
                    Columns::Five => (-2, 2),
                };
                i >= 0 && i + j >= 0 && (lowest..=highest).contains(&j)
            }
        }
    }

    /// Face de la tuile de départ : elle découle de la forme (MKUE p. 14).
    pub fn starting_tile(self) -> TileId {
        match self {
            MapShape::Wedge | MapShape::StandardWedge => TileId::StartingA,
            MapShape::Open(_) => TileId::StartingB,
        }
    }

    /// Emplacements des tuiles révélées à la mise en place, dans l'ordre de révélation
    /// (MKUE p. 3, illustration p. 14).
    pub fn first_positions(self) -> Vec<TilePosition> {
        let b = TilePosition::from_lattice(0, 1);
        let a = TilePosition::from_lattice(1, 0);
        match self {
            MapShape::Wedge | MapShape::StandardWedge => vec![b, a],
            MapShape::Open(_) => vec![b, a, TilePosition::from_lattice(1, -1)],
        }
    }
}

/// Ordre des tuiles de campagne dans la pioche.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum CountrysideOrder {
    /// Tirées au hasard (MKUE p. 3).
    Random,
    /// Les tuiles 1 à n, la 1 sur le dessus (« Première reconnaissance », MKUE p. 18).
    ByNumber,
}

/// Composition de la pioche de tuiles (MKUE p. 3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct TileDeckRecipe {
    pub countryside: u8,
    pub countryside_order: CountrysideOrder,
    pub core_city: u8,
    pub core_non_city: u8,
}

/// Fin du scénario (MKUE p. 18, *Scenario End* ; spec 063), point d'extension.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum ScenarioEnd {
    /// La limite de manches seule (MKUE p. 4, 5.a) : le bac à sable.
    #[default]
    RoundLimit,
    /// En plus, un dernier tour après la révélation d'une cité (MKUE p. 18, *First Reconnaissance*).
    CityRevealed,
    /// En plus, un dernier tour après la conquête de toutes les cités du scénario (MKUE p. 19,
    /// *Solo Conquest* ; spec 064).
    AllCitiesConquered,
}

/// Décompte final du scénario (spec 063), point d'extension.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum FinalScoring {
    /// La partie s'arrête sans score : le bac à sable.
    #[default]
    None,
    /// La gloire et les Hauts Faits, sans bonus de titre (MKUE p. 18, *Solo Variant* ; p. 15).
    AchievementsWithoutTitles,
    /// Les mêmes, plus les points des cités et du temps (MKUE p. 19, *Scoring* ; spec 064).
    SoloConquest,
}

/// Les sorts d'un scénario (spec 064).
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum SpellSet {
    /// Les 20 ; aussi la valeur d'une sauvegarde d'avant la spec 064.
    #[default]
    All,
    /// Sans les quatre sorts compétitifs, ceux qui visent les autres joueurs (MKUE p. 19,
    /// *Cards and Skills*).
    WithoutCompetitive,
}

/// Refus de créer un scénario incohérent.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum ScenarioError {
    #[error("{requested} tuiles de campagne demandées, {available} disponibles")]
    TooManyCountryside { requested: u8, available: usize },
    #[error("{requested} tuiles royaume de cité demandées, {available} disponibles")]
    TooManyCoreCity { requested: u8, available: usize },
    #[error("{requested} tuiles royaume sans cité demandées, {available} disponibles")]
    TooManyCoreNonCity { requested: u8, available: usize },
    #[error(
        "la forme révèle {needed} tuiles à la mise en place, la pioche n'a que {countryside} tuiles de campagne"
    )]
    NotEnoughForSetup { needed: usize, countryside: u8 },
    #[error("un scénario dure au moins une manche")]
    NoRound,
    #[error("un joueur fantôme suppose des tactiques")]
    DummyWithoutTactics,
    #[error("niveaux de cité invalides : une liste de 1 à {max} niveaux, chacun de 1 à 11")]
    InvalidCityLevels { max: usize },
    #[error("une fin par conquête suppose des cités ouvertes et des tuiles royaume de cité")]
    ConquestWithoutCities,
}

/// Le plus de cités qu'un scénario puisse révéler : autant que de tuiles royaume de cité.
pub const MAX_CITIES: usize = 4;

/// Les niveaux des cités d'un scénario, dans l'ordre de révélation (MKUE p. 18, *City Levels* ;
/// spec 062). Un tableau borné plutôt qu'un `Vec`, pour que `Scenario` reste `Copy`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct CityLevels {
    levels: [u8; MAX_CITIES],
    count: u8,
}

impl CityLevels {
    /// Refusé si la liste est vide, trop longue, ou si un niveau sort de 1 à 11.
    pub fn new(levels: &[u8]) -> Result<CityLevels, ScenarioError> {
        let invalid = ScenarioError::InvalidCityLevels { max: MAX_CITIES };
        if levels.is_empty()
            || levels.len() > MAX_CITIES
            || levels.iter().any(|level| !(1..=11).contains(level))
        {
            return Err(invalid);
        }
        let mut array = [0; MAX_CITIES];
        for (slot, &level) in array.iter_mut().zip(levels) {
            *slot = level;
        }
        Ok(CityLevels {
            levels: array,
            count: u8::try_from(levels.len()).map_err(|_| invalid)?,
        })
    }

    /// Le niveau de la `index`-ième cité révélée (0 pour la première) ; au-delà de la liste, le
    /// dernier (spec 062).
    pub fn level(&self, index: usize) -> u8 {
        let levels = self.as_slice();
        levels
            .get(index)
            .or(levels.last())
            .copied()
            .unwrap_or_default()
    }

    /// Les niveaux, dans l'ordre.
    pub fn as_slice(&self) -> &[u8] {
        &self.levels[..usize::from(self.count)]
    }
}

/// La règle des cités d'un scénario (spec 062) : fermées (ni niveau, ni défenseurs, ni entrée ;
/// Première reconnaissance, MKUE p. 18), ou ouvertes avec leurs niveaux.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum CityRule {
    /// Aussi la valeur d'une sauvegarde d'avant la spec 062.
    #[default]
    Closed,
    Levels(CityLevels),
}

/// Un scénario cohérent : il n'existe que par [`Scenario::new`], qui le valide.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Scenario {
    id: ScenarioId,
    shape: MapShape,
    tile_deck: TileDeckRecipe,
    /// Nombre de manches : la dernière finie, la partie est terminée (MKUE p. 4, 5.a ; spec 023).
    rounds: u8,
    /// Gloire rapportée par chaque tuile révélée par le joueur (spec 063). Les champs de la spec
    /// 063 manquent dans une sauvegarde d'avant : ils prennent les valeurs du bac à sable (L6).
    #[serde(default)]
    fame_per_tile: u8,
    #[serde(default)]
    end: ScenarioEnd,
    #[serde(default)]
    scoring: FinalScoring,
    /// Spec 050 : unités d'élite dans la pioche et l'offre, actions avancées toutes ou 1 à 16, au
    /// moins une unité de village dans l'offre de la mise en place. Absents d'une sauvegarde
    /// d'avant : les valeurs du bac à sable.
    #[serde(default = "yes")]
    elite_units: bool,
    #[serde(default)]
    advanced_actions: AdvancedActionSet,
    #[serde(default)]
    village_unit_at_setup: bool,
    /// Spec 060 : la règle des tactiques ; absente d'une sauvegarde d'avant, pas de tactique (C40).
    #[serde(default)]
    tactics: TacticRule,
    /// Spec 061 : le joueur fantôme ; absent d'une sauvegarde d'avant, pas de fantôme (C35).
    #[serde(default)]
    dummy: DummyRule,
    /// Spec 062 : les cités ; absentes d'une sauvegarde d'avant, fermées (C36).
    #[serde(default)]
    cities: CityRule,
    /// Spec 064 : les sorts ; absents d'une sauvegarde d'avant, tous (C25).
    #[serde(default)]
    spells: SpellSet,
}

fn yes() -> bool {
    true
}

impl Scenario {
    pub fn new(
        id: ScenarioId,
        shape: MapShape,
        tile_deck: TileDeckRecipe,
        rounds: u8,
        fame_per_tile: u8,
        end: ScenarioEnd,
        scoring: FinalScoring,
    ) -> Result<Scenario, ScenarioError> {
        if rounds == 0 {
            return Err(ScenarioError::NoRound);
        }
        let check = |requested: u8, available: usize, error: fn(u8, usize) -> ScenarioError| {
            if usize::from(requested) > available {
                Err(error(requested, available))
            } else {
                Ok(())
            }
        };
        check(
            tile_deck.countryside,
            countryside_tiles().len(),
            |requested, available| ScenarioError::TooManyCountryside {
                requested,
                available,
            },
        )?;
        check(
            tile_deck.core_city,
            core_tiles(true).len(),
            |requested, available| ScenarioError::TooManyCoreCity {
                requested,
                available,
            },
        )?;
        check(
            tile_deck.core_non_city,
            core_tiles(false).len(),
            |requested, available| ScenarioError::TooManyCoreNonCity {
                requested,
                available,
            },
        )?;
        // Les premières tuiles révélées sont des tuiles de campagne, au-dessus de la pioche.
        let needed = shape.first_positions().len();
        if usize::from(tile_deck.countryside) < needed {
            return Err(ScenarioError::NotEnoughForSetup {
                needed,
                countryside: tile_deck.countryside,
            });
        }
        let scenario = Scenario {
            id,
            shape,
            tile_deck,
            rounds,
            fame_per_tile,
            end,
            scoring,
            elite_units: true,
            advanced_actions: AdvancedActionSet::All,
            village_unit_at_setup: false,
            tactics: TacticRule::None,
            dummy: DummyRule::None,
            cities: CityRule::Closed,
            spells: SpellSet::All,
        };
        scenario.check_conquest()?;
        Ok(scenario)
    }

    /// Bac à sable : le plateau de jeu de « Première reconnaissance » en solo (MKUE p. 18).
    pub fn sandbox() -> Scenario {
        Scenario {
            id: ScenarioId::Sandbox,
            shape: MapShape::Wedge,
            tile_deck: TileDeckRecipe {
                countryside: 8,
                countryside_order: CountrysideOrder::ByNumber,
                core_city: 1,
                core_non_city: 2,
            },
            // « Première reconnaissance » en solo : 2 jours et 2 nuits (MKUE p. 18).
            rounds: 4,
            fame_per_tile: 0,
            end: ScenarioEnd::RoundLimit,
            scoring: FinalScoring::None,
            // Spec 050, Q8 : les élites et les 28 actions avancées, pour les tests.
            elite_units: true,
            advanced_actions: AdvancedActionSet::All,
            village_unit_at_setup: false,
            // Spec 060 (Q2) : pas de tactique, pour garder ses tests et ses parties préparées.
            tactics: TacticRule::None,
            dummy: DummyRule::None,
            // Spec 062 (Q1) : une cité de niveau 5.
            cities: CityRule::Levels(CityLevels {
                levels: [5, 0, 0, 0],
                count: 1,
            }),
            spells: SpellSet::All,
        }
    }

    /// « Première reconnaissance » en solo (MKUE p. 18) : le plateau de jeu du bac à sable, 1 gloire
    /// par tuile révélée, un dernier tour après la cité, le décompte des Hauts Faits sans titres
    /// (spec 063), et le joueur fantôme (spec 061), que `without_dummy` retire.
    pub fn first_reconnaissance() -> Scenario {
        Scenario {
            id: ScenarioId::FirstReconnaissance,
            fame_per_tile: 1,
            end: ScenarioEnd::CityRevealed,
            scoring: FinalScoring::AchievementsWithoutTitles,
            // MKUE p. 18, *Special Rules* ; Compendium, *Miscellaneous* S8 (spec 050, Q3).
            elite_units: false,
            advanced_actions: AdvancedActionSet::FirstSixteen,
            village_unit_at_setup: true,
            // MKUE p. 18, *Solo Variant* ; p. 19, *Special Rules* (spec 060).
            tactics: TacticRule::Chosen {
                order: TacticOrder::PlayerFirst,
                removal: TacticRemoval::AllTaken,
            },
            // MKUE p. 18, *Solo Variant* : *Use a Dummy player* (spec 061, Q1).
            dummy: DummyRule::Standard,
            // MKUE p. 18 : *Cities cannot be conquered nor entered* (spec 062, Q2).
            cities: CityRule::Closed,
            ..Scenario::sandbox()
        }
    }

    /// « Conquête en solitaire » (MKUE p. 19 ; spec 064).
    pub fn solo_conquest() -> Scenario {
        Scenario {
            id: ScenarioId::SoloConquest,
            // MKUE p. 19, *Map* ; p. 14, *Wedge Map* : pas de tuile royaume sur la côte.
            shape: MapShape::StandardWedge,
            // 7 tuiles de campagne mélangées, 2 royaume de cité, 2 sans cité (MKUE p. 19 ; p. 3).
            tile_deck: TileDeckRecipe {
                countryside: 7,
                countryside_order: CountrysideOrder::Random,
                core_city: 2,
                core_non_city: 2,
            },
            // Jour, Nuit, Jour, Nuit, Jour, Nuit.
            rounds: 6,
            fame_per_tile: 0,
            end: ScenarioEnd::AllCitiesConquered,
            scoring: FinalScoring::SoloConquest,
            elite_units: true,
            advanced_actions: AdvancedActionSet::All,
            village_unit_at_setup: false,
            // MKUE p. 19, *Special Rules* (spec 060).
            tactics: TacticRule::Chosen {
                order: TacticOrder::PlayerFirst,
                removal: TacticRemoval::AllTaken,
            },
            // MKUE p. 19, *Dummy Player* : *one standard Dummy player* (Q3 : obligatoire).
            dummy: DummyRule::Standard,
            // MKUE p. 19, *Cities* : la première révélée de niveau 5, la seconde de niveau 8.
            cities: CityRule::Levels(CityLevels {
                levels: [5, 8, 0, 0],
                count: 2,
            }),
            // MKUE p. 19, *Cards and Skills*.
            spells: SpellSet::WithoutCompetitive,
        }
    }

    /// Le scénario d'un identifiant (spec 065).
    pub fn of(id: ScenarioId) -> Scenario {
        match id {
            ScenarioId::Sandbox => Scenario::sandbox(),
            ScenarioId::FirstReconnaissance => Scenario::first_reconnaissance(),
            ScenarioId::SoloConquest => Scenario::solo_conquest(),
        }
    }

    /// Spec 064 (Q3) : le joueur fantôme peut être retiré à la nouvelle partie.
    pub fn dummy_optional(&self) -> bool {
        // MKUE p. 18 : *you may also ignore the Dummy player in your first game*.
        self.id == ScenarioId::FirstReconnaissance
    }

    /// Spec 064 (L7) : le même scénario avec cette fin ; une fin par conquête est refusée sans
    /// cités ouvertes ni tuile royaume de cité.
    pub fn with_end(self, end: ScenarioEnd) -> Result<Scenario, ScenarioError> {
        let scenario = Scenario { end, ..self };
        scenario.check_conquest()?;
        Ok(scenario)
    }

    /// Une fin par conquête suppose des cités ouvertes et des tuiles royaume de cité (L7).
    fn check_conquest(&self) -> Result<(), ScenarioError> {
        let open = matches!(self.cities, CityRule::Levels(_));
        if self.end == ScenarioEnd::AllCitiesConquered && (!open || self.tile_deck.core_city == 0) {
            return Err(ScenarioError::ConquestWithoutCities);
        }
        Ok(())
    }

    /// Spec 064 : les sorts du scénario.
    pub fn spells(&self) -> SpellSet {
        self.spells
    }

    pub fn id(&self) -> ScenarioId {
        self.id
    }

    pub fn rounds(&self) -> u8 {
        self.rounds
    }

    pub fn shape(&self) -> MapShape {
        self.shape
    }

    pub fn tile_deck(&self) -> TileDeckRecipe {
        self.tile_deck
    }

    pub fn fame_per_tile(&self) -> u8 {
        self.fame_per_tile
    }

    pub fn end(&self) -> ScenarioEnd {
        self.end
    }

    pub fn scoring(&self) -> FinalScoring {
        self.scoring
    }

    /// Le scénario utilise les unités d'élite (spec 050).
    pub fn elite_units(&self) -> bool {
        self.elite_units
    }

    /// La règle des tactiques (spec 060).
    pub fn tactics(&self) -> TacticRule {
        self.tactics
    }

    /// Spec 061 : le joueur fantôme du scénario.
    pub fn dummy(&self) -> DummyRule {
        self.dummy
    }

    /// Spec 061 (Q1) : le même scénario sans joueur fantôme, l'option de la nouvelle partie.
    pub fn without_dummy(self) -> Scenario {
        Scenario {
            dummy: DummyRule::None,
            ..self
        }
    }

    /// Spec 061 (L9) : le même scénario avec ce joueur fantôme ; refusé sans tactiques.
    pub fn with_dummy(self, dummy: DummyRule) -> Result<Scenario, ScenarioError> {
        if dummy != DummyRule::None && self.tactics == TacticRule::None {
            return Err(ScenarioError::DummyWithoutTactics);
        }
        Ok(Scenario { dummy, ..self })
    }

    /// Spec 062 : la règle des cités.
    pub fn cities(&self) -> CityRule {
        self.cities
    }

    /// Spec 062 : le même scénario avec cette règle des cités (scénarios de test).
    pub fn with_cities(self, cities: CityRule) -> Scenario {
        Scenario { cities, ..self }
    }

    pub fn advanced_actions(&self) -> AdvancedActionSet {
        self.advanced_actions
    }

    /// Au moins une unité de village dans l'offre de la mise en place (spec 050).
    pub fn village_unit_at_setup(&self) -> bool {
        self.village_unit_at_setup
    }

    /// Ordre de la pioche de tuiles : tuiles royaume mélangées sous les tuiles de campagne
    /// (MKUE p. 3). L'ordre d'usage du générateur fait partie du contrat de la spec 012.
    pub(crate) fn form_tile_deck(&self, rng: &mut GameRng) -> Vec<TileId> {
        let recipe = self.tile_deck;
        let mut countryside = countryside_tiles();
        if recipe.countryside_order == CountrysideOrder::Random {
            countryside.shuffle(rng);
        }
        countryside.truncate(usize::from(recipe.countryside));

        let mut city = core_tiles(true);
        city.shuffle(rng);
        city.truncate(usize::from(recipe.core_city));
        let mut non_city = core_tiles(false);
        non_city.shuffle(rng);
        non_city.truncate(usize::from(recipe.core_non_city));

        let mut core = city;
        core.extend(non_city);
        core.shuffle(rng);
        if recipe.countryside_order == CountrysideOrder::Random {
            countryside.shuffle(rng);
        }
        countryside.extend(core);
        countryside
    }
}

/// Tuiles de campagne du jeu de base, par numéro croissant.
fn countryside_tiles() -> Vec<TileId> {
    let mut tiles: Vec<TileId> = base_game_tiles()
        .iter()
        .map(|definition| definition.id)
        .filter(|id| id.kind() == TileKind::Countryside)
        .collect();
    tiles.sort();
    tiles
}

/// Tuiles royaume de cité (`true`) ou sans cité (`false`), par numéro croissant.
fn core_tiles(city: bool) -> Vec<TileId> {
    let mut tiles: Vec<TileId> = base_game_tiles()
        .iter()
        .map(|definition| definition.id)
        .filter(|id| id.kind() == TileKind::Core && id.is_city() == city)
        .collect();
    tiles.sort();
    tiles
}
