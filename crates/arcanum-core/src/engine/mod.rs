use std::collections::BTreeMap;

use rand::seq::{IndexedRandom, SliceRandom};

use crate::combat::{
    Combat, Moment, PlayMoment, apply_defeat, apply_destroyed, apply_dismiss, apply_knock_out,
    apply_paralysis, apply_poison, apply_summon, apply_unit_damage, apply_unit_destroyed,
    apply_wounds, assign_actions, assign_damage_events, conversion_actions, engaging_actions,
    fear_holds_summons, influence_conversion, is_playable_now, moment, move_conversion,
    movement_over, next_phase_events, phase_actions, rampaging_reputation, release_summons_events,
    resolve_attack, resolve_block, sideways_in_combat, spell_enemy_targets, target_actions,
    unit_enemy_targets,
};
use crate::exploration::revealable_removed;

use crate::offer::common_deck_cards;
use crate::state::PlayerState;
use crate::turn_fact::turn_fact;
use crate::{
    Action, ActionError, Artifact, ArtifactChoices, AttackKind, BasicColor, BasicEffect,
    BoostTarget, CardId, CardPlay, Cost, DICE_PER_TURN, DeedDeck, DieSetting, Effect, Element,
    EnemyColor, EnemyToken, Event, Feature, GameConfig, GameContent, GameEndReason, GameRngs,
    GameState, HealingTarget, Hex, Interaction, Inventory, LastTurn, MAX_LEVEL, ManaColor,
    ManaPayment, MoveCost, PlayedCard, PlayerId, RngUsage, Route, STARTING_HAND_LIMIT, ScenarioEnd,
    SidewaysEffect, Site, TileDeck, TileId, TilePosition, TimeOfDay, level_token, mine_color,
    move_cost, provoked, roll_die, roll_source, routes_ending_in, source_size, tile,
};
use crate::{
    CardDefinition, CardDestination, CastTargets, DeedCard, DestroyedFrom, EnemyModifierKind,
    ManaChoice, Meditation, Modifier, ModifierDuration, ModifierKind, ModifierSource,
    is_safe_space,
};
use crate::{
    CenterAdvantage, OwnedSkill, PolarSource, Skill, SkillEffect, SkillMana, SkillState,
    SkillTarget, SkillUsage,
};
use crate::{CityRule, DummyHeroChoice, DummyRule, Hero, Tactic, TacticRemoval, TacticRule};
use crate::{
    CommonCard, CommonDeck, EXPLORE_COST, LevelReward, Offer, OwnedUnit, PlacedRuin, Placement,
    RecruitSite, RuinReward, RuinToken, Scenario, UnitCard, WoundPile, free_positions_touching,
    next_tile, placement,
};

mod apply;
mod decide;
mod events;
mod legal;

// Les sous-modules partagent leurs fonctions `pub(super)` par ces imports (`use super::*`).
use apply::*;
use events::*;
use legal::*;

pub use apply::apply;
pub use decide::decide;
pub use legal::{
    ALTAR_FAME, CITY_ADVANCED_ACTION_COST, CITY_ARTIFACT_COST, CITY_ELITE_UNIT_COST,
    MONASTERY_ADVANCED_ACTION_COST, MONASTERY_HEALING_COST, SPELL_COST, VILLAGE_HEALING_COST,
    bonds_cost, end_turn_allowed, glade_wound_available, leadership_effect, legal_actions,
    wings_of_night_cost,
};
pub(crate) use legal::{
    after_forced_withdrawal, combinations, forced_withdrawal_path, must_announce_end_of_round,
    recruitable_units, turn_action_taken, wings_of_night_castable,
};

/// Quel joueur doit choisir, et quel type de choix.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PendingDecision {
    pub player: PlayerId,
    pub kind: DecisionKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecisionKind {
    /// Choisir une action parmi les actions légales.
    ChooseAction,
}

/// Événements de démarrage : la mise en place lit le scénario (spec 012) ; le hasard est
/// résolu ici, une fois pour toutes, chaque tirage dans le générateur de son usage (spec 038).
/// Le deck est celui du héros, puis on pioche jusqu'à la limite de main (spec 013).
pub fn new_game(config: &GameConfig, rngs: &mut GameRngs) -> Vec<Event> {
    let shape = config.scenario.shape();
    let tile_order = config.scenario.form_tile_deck(rngs.rng(RngUsage::Tiles));
    let mut deck_order = config.hero.starting_deck();
    deck_order.shuffle(rngs.rng(RngUsage::Deck(PlayerId::SOLO)));

    let mut events = vec![
        Event::GameStarted {
            config: config.clone(),
        },
        // MKUE p. 3 : la face de la tuile de départ est donnée par la forme du plateau de jeu.
        Event::TilePlaced {
            tile: shape.starting_tile(),
            position: TilePosition::ORIGIN,
        },
        Event::TileDeckFormed {
            order: tile_order.clone(),
        },
    ];
    // Les premières tuiles viennent du dessus de la pioche (MKUE p. 14).
    let revealed: Vec<(TileId, TilePosition)> = tile_order
        .into_iter()
        .zip(shape.first_positions())
        .collect();
    events.extend(
        revealed
            .iter()
            .map(|&(tile, position)| Event::TileRevealed { tile, position }),
    );
    events.extend([
        // La figurine entre par le portail, au centre de la tuile de départ (MKUE p. 3, p. 7).
        Event::HeroFigurePlaced {
            player: PlayerId::SOLO,
            hex: Hex::ORIGIN,
        },
        Event::DeckShuffled {
            player: PlayerId::SOLO,
            order: deck_order.clone(),
        },
    ]);
    // MKUE p. 3 : 5 cartes en main, la limite de main au niveau 1.
    events.extend(
        deck_order
            .into_iter()
            .take(STARTING_HAND_LIMIT)
            .map(|card| Event::CardDrawn {
                player: PlayerId::SOLO,
                card,
            }),
    );
    // Spec 020. Solo : un seul joueur réel (MKUE p. 3).
    events.push(Event::SourceRolled {
        dice: roll_source(source_size(1), rngs.rng(RngUsage::Source)),
    });
    // Spec 030 : les piles d'ennemis (MKUE p. 3), puis les ennemis errants des tuiles déjà posées,
    // dans l'ordre de pose.
    let mut piles = BTreeMap::new();
    for color in EnemyColor::ALL {
        let mut order = GameContent::embedded().enemy_tokens(color);
        order.shuffle(rngs.rng(RngUsage::EnemyPile(color)));
        events.push(Event::EnemyPileShuffled {
            color,
            order: order.clone(),
        });
        piles.insert(color, order);
    }
    // Spec 045 : la pile des jetons de ruines (MKUE p. 3), avec son propre générateur.
    let mut ruin_pile = GameContent::embedded().ruin_tokens();
    ruin_pile.shuffle(rngs.rng(RngUsage::RuinPile));
    events.push(Event::RuinPileShuffled {
        order: ruin_pile.clone(),
    });
    // Aucune défausse à la mise en place : ces jetons ne remélangent rien.
    let mut discards = BTreeMap::new();
    let mut monasteries = 0;
    for (tile, position) in
        std::iter::once((shape.starting_tile(), TilePosition::ORIGIN)).chain(revealed)
    {
        events.extend(rampaging_tokens(
            tile,
            position,
            &mut piles,
            &mut discards,
            rngs,
        ));
        events.extend(defenders(
            tile,
            position,
            None,
            &mut piles,
            &mut discards,
            rngs,
        ));
        events.extend(placed_ruins(tile, position, &mut ruin_pile, TimeOfDay::Day));
        monasteries += tile_monasteries(tile, position).len();
    }
    events.extend(setup_offers(config.scenario, monasteries, rngs));
    // Spec 051 : la pile de compétences du héros (MKUE p. 3), avec son propre générateur ; en solo,
    // la coopérative gardée et la compétitive retirée (Compendium, *Skill Tokens* S5).
    let mut skill_pile: Vec<Skill> = Skill::ALL
        .into_iter()
        .filter(|skill| skill.hero() == config.hero)
        .collect();
    skill_pile.shuffle(rngs.rng(RngUsage::SkillPile(PlayerId::SOLO)));
    events.push(Event::SkillPileShuffled {
        player: PlayerId::SOLO,
        order: skill_pile,
    });
    events.extend(dummy_setup(config, rngs));
    events
}

/// Spec 061 (MKUE p. 15, *At the start of the game*) : le joueur fantôme, avec son propre
/// générateur, dans l'ordre héros (au hasard seulement), compétences, deck ; rien sans fantôme.
fn dummy_setup(config: &GameConfig, rngs: &mut GameRngs) -> Option<Event> {
    if config.scenario.dummy() == DummyRule::None {
        return None;
    }
    let rng = rngs.rng(RngUsage::Dummy);
    let others: Vec<Hero> = Hero::ALL
        .into_iter()
        .filter(|&hero| hero != config.hero)
        .collect();
    let hero = match config.dummy_hero {
        // `GameConfig::validate` refuse le héros du joueur (L10).
        DummyHeroChoice::Hero(hero) if hero != config.hero => hero,
        _ => *others.choose(rng)?,
    };
    // Compendium, *Dummy Player* S1 : ses compétences, comme celles du joueur en solo.
    let mut skills: Vec<Skill> = Skill::ALL
        .into_iter()
        .filter(|skill| skill.hero() == hero)
        .collect();
    skills.shuffle(rng);
    let mut deck = hero.dummy_deck();
    deck.shuffle(rng);
    Some(Event::DummySetUp {
        hero,
        crystals: GameContent::embedded().dummy_crystals(hero),
        skills,
        deck,
    })
}

/// Les cases des sites de cette tuile posée à cet emplacement, et leur site, case centrale
/// d'abord puis les côtés dans l'ordre de `Direction::ALL`.
fn tile_sites(tile_id: TileId, position: TilePosition) -> Vec<(Hex, Site)> {
    let Some(definition) = tile(tile_id) else {
        return Vec::new();
    };
    let spaces = std::iter::once(definition.center).chain(definition.sides);
    position
        .hexes()
        .into_iter()
        .zip(spaces)
        .filter_map(|(hex, space)| match space.feature {
            Some(Feature::Site(site)) => Some((hex, site)),
            _ => None,
        })
        .collect()
}

/// Les cases des monastères de cette tuile posée à cet emplacement (spec 041).
fn tile_monasteries(tile_id: TileId, position: TilePosition) -> Vec<Hex> {
    tile_sites(tile_id, position)
        .into_iter()
        .filter(|&(_, site)| site == Site::Monastery)
        .map(|(hex, _)| hex)
        .collect()
}

/// La pile du défenseur d'un site : gris pour une forteresse, violet pour une tour de mage
/// (cartes de site Keep et Mage Tower ; spec 043). Les cités attendent la spec 062.
fn defender_color(site: Site) -> Option<EnemyColor> {
    match site {
        Site::Keep => Some(EnemyColor::Grey),
        Site::MageTower => Some(EnemyColor::Violet),
        _ => None,
    }
}

/// Un défenseur face cachée par forteresse et tour de mage de la tuile, tiré comme les autres
/// jetons, pile de secours comprise (cartes de site, *When Revealed* ; specs 031, 033 et 043).
/// Plus aucun jeton nulle part : le site reste sans défenseur (L4).
///
/// Spec 062 : puis, avec `city_level` (le niveau que le scénario donne à la cité de la tuile ;
/// `None` pour des cités fermées), la cité entre en jeu avec un défenseur par cercle de sa
/// figurine (MKUE p. 7, 5.g ; carte City, *When Revealed*) ; plus aucun jeton, moins de défenseurs
/// (L1).
fn defenders(
    tile_id: TileId,
    position: TilePosition,
    city_level: Option<u8>,
    piles: &mut BTreeMap<EnemyColor, Vec<EnemyToken>>,
    discards: &mut BTreeMap<EnemyColor, Vec<EnemyToken>>,
    rngs: &mut GameRngs,
) -> Vec<Event> {
    let mut events = Vec::new();
    let sites = tile_sites(tile_id, position);
    let mut place = |events: &mut Vec<Event>, color: EnemyColor, hex: Hex| {
        let (drawn, token) = draw_enemy_token(color, piles, discards, rngs);
        events.extend(drawn);
        if let Some(token) = token {
            events.push(Event::DefenderPlaced { token, hex });
        }
    };
    for &(hex, site) in &sites {
        if let Some(color) = defender_color(site) {
            place(&mut events, color, hex);
        }
    }
    let Some(level) = city_level else {
        return events;
    };
    for &(hex, site) in &sites {
        let Site::City(color) = site else {
            continue;
        };
        events.push(Event::CityPlaced { hex, color, level });
        for &defender in GameContent::embedded().city_garrison(color, level) {
            place(&mut events, defender, hex);
        }
    }
    events
}

/// Un jeton de ruines du dessus de `pile` par ruine de la tuile, face visible de jour, face cachée
/// de nuit (carte *Ancient Ruins*, *When Revealed* ; spec 045). `pile` perd les jetons posés ; vide,
/// rien (impossible : plus de jetons que de ruines, C1).
fn placed_ruins(
    tile_id: TileId,
    position: TilePosition,
    pile: &mut Vec<RuinToken>,
    time: TimeOfDay,
) -> Vec<Event> {
    let mut events = Vec::new();
    for (hex, site) in tile_sites(tile_id, position) {
        if site != Site::Ruins || pile.is_empty() {
            continue;
        }
        events.push(Event::RuinTokenPlaced {
            token: pile.remove(0),
            hex,
            face_up: time == TimeOfDay::Day,
        });
    }
    events
}

/// Les cases des monastères du plateau de jeu, dans l'ordre des emplacements (spec 041).
pub(crate) fn board_monasteries(map: &crate::Map) -> Vec<Hex> {
    map.tiles()
        .flat_map(|(position, tile)| tile_monasteries(tile, position))
        .collect()
}

/// Une action avancée du dessus de `deck` par monastère, dans l'offre d'unités, à la suite des
/// unités (MKUE p. 7, 5.f ; p. 4, 2.c ; carte du monastère, *Training*). Pioche vide : rien.
fn monastery_cards(deck: &[CommonCard], monasteries: usize) -> Vec<Event> {
    deck.iter()
        .take(monasteries)
        .map(|&card| Event::OfferCardDealt {
            offer: Offer::Units,
            card,
        })
        .collect()
}

/// Les pioches communes et les offres de la mise en place (MKUE p. 3 ; spec 050), chaque pioche
/// avec son générateur : actions avancées, sorts, artefacts, unités régulières, puis d'élite si
/// le scénario les utilise ; puis 3 actions avancées, 3 sorts, 3 unités régulières, et une
/// action avancée par monastère de la mise en place (Compendium, *Map Revealing* S1 ; spec 041).
fn setup_offers(scenario: Scenario, monasteries: usize, rngs: &mut GameRngs) -> Vec<Event> {
    let mut events = Vec::new();
    let mut decks = BTreeMap::new();
    for deck in CommonDeck::ALL {
        if deck == CommonDeck::EliteUnits && !scenario.elite_units() {
            continue;
        }
        let mut order = common_deck_cards(deck, scenario.advanced_actions(), scenario.spells());
        let rng = rngs.rng(RngUsage::CommonDeck(deck));
        order.shuffle(rng);
        // MKUE p. 18 (Première reconnaissance) : sans unité de village dans les trois premières,
        // on remélange ; seul l'ordre final compte (spec 050, Q3).
        if deck == CommonDeck::RegularUnits
            && scenario.village_unit_at_setup()
            && order.iter().any(|card| is_village_unit(*card))
        {
            while !order
                .iter()
                .take(OFFER_SIZE)
                .any(|card| is_village_unit(*card))
            {
                order.shuffle(rng);
            }
        }
        events.push(Event::CommonDeckShuffled {
            deck,
            order: order.clone(),
        });
        decks.insert(deck, order);
    }
    for (offer, deck) in [
        (Offer::AdvancedActions, CommonDeck::AdvancedActions),
        (Offer::Spells, CommonDeck::Spells),
        (Offer::Units, CommonDeck::RegularUnits),
    ] {
        let pile = decks.entry(deck).or_default();
        let dealt = OFFER_SIZE.min(pile.len());
        events.extend(
            pile.drain(..dealt)
                .map(|card| Event::OfferCardDealt { offer, card }),
        );
    }
    let advanced = decks.entry(CommonDeck::AdvancedActions).or_default();
    events.extend(monastery_cards(advanced, monasteries));
    events
}

/// Trois cartes par offre en solo : unités, *actual players plus 2* (MKUE p. 3 ; Compendium,
/// *Setup* S1).
const OFFER_SIZE: usize = 3;

fn is_village_unit(card: CommonCard) -> bool {
    card.unit().is_some_and(|unit| {
        unit.unit
            .definition()
            .recruit
            .contains(&RecruitSite::Village)
    })
}

/// Un jeton face visible par ennemi errant imprimé sur la tuile, pris sur le dessus de la pile de
/// sa couleur, case centrale d'abord puis les côtés dans l'ordre de `Direction::ALL` (MKUE p. 7,
/// 5.f ; cartes de site *Marauding Orcs* et *Draconum*). `piles` perd les jetons posés.
///
/// Une pile vide est refaite de sa défausse, mélangée avec le générateur de la pile (MKUE p. 3,
/// *Enemy and Ruin token piles* ; spec 031) : `EnemyPileReshuffled` précède alors le jeton posé.
/// `discards` perd la défausse reprise.
fn rampaging_tokens(
    tile_id: TileId,
    position: TilePosition,
    piles: &mut BTreeMap<EnemyColor, Vec<EnemyToken>>,
    discards: &mut BTreeMap<EnemyColor, Vec<EnemyToken>>,
    rngs: &mut GameRngs,
) -> Vec<Event> {
    let Some(definition) = tile(tile_id) else {
        return Vec::new();
    };
    let spaces = std::iter::once(definition.center).chain(definition.sides);
    let mut events = Vec::new();
    for (hex, space) in position.hexes().into_iter().zip(spaces) {
        let Some(Feature::Rampaging(rampaging)) = space.feature else {
            continue;
        };
        let (drawn, token) = draw_enemy_token(rampaging.color(), piles, discards, rngs);
        events.extend(drawn);
        if let Some(token) = token {
            events.push(Event::EnemyTokenPlaced { token, hex });
        }
    }
    events
}

/// Tire le jeton du dessus de la pile `color` ; `piles` le perd. Pile vide : sa défausse est
/// mélangée pour la refaire, avec le générateur de la pile (MKUE p. 3, *Enemy and Ruin token
/// piles* ; spec 031). Pile et défausse vides : une autre pile qui a encore des jetons, choisie au
/// hasard avec le générateur de la pile vide (spec 033, Q2 ; Compendium, *Scenarios* S5). Rend
/// les `EnemyPileReshuffled` à produire avant le jeton, et le jeton (`None` si plus aucun jeton
/// nulle part).
pub(crate) fn draw_enemy_token(
    color: EnemyColor,
    piles: &mut BTreeMap<EnemyColor, Vec<EnemyToken>>,
    discards: &mut BTreeMap<EnemyColor, Vec<EnemyToken>>,
    rngs: &mut GameRngs,
) -> (Vec<Event>, Option<EnemyToken>) {
    let has_tokens = |piles: &BTreeMap<EnemyColor, Vec<EnemyToken>>,
                      discards: &BTreeMap<EnemyColor, Vec<EnemyToken>>,
                      color: EnemyColor| {
        piles.get(&color).is_some_and(|pile| !pile.is_empty())
            || discards
                .get(&color)
                .is_some_and(|discard| !discard.is_empty())
    };
    let color = if has_tokens(piles, discards, color) {
        color
    } else {
        let others: Vec<EnemyColor> = EnemyColor::ALL
            .into_iter()
            .filter(|&other| other != color && has_tokens(piles, discards, other))
            .collect();
        // Sans autre pile, pas de tirage : le générateur n'avance pas (spec 038).
        if others.is_empty() {
            return (Vec::new(), None);
        }
        match others.choose(rngs.rng(RngUsage::EnemyPile(color))) {
            Some(&other) => other,
            None => return (Vec::new(), None),
        }
    };
    let mut events = Vec::new();
    let pile = piles.entry(color).or_default();
    if pile.is_empty() {
        let mut order = discards.remove(&color).unwrap_or_default();
        order.shuffle(rngs.rng(RngUsage::EnemyPile(color)));
        events.push(Event::EnemyPileReshuffled {
            color,
            order: order.clone(),
        });
        *pile = order;
    }
    let token = (!pile.is_empty()).then(|| pile.remove(0));
    (events, token)
}

/// Vrai si cette case porte une forteresse ou une tour de mage : son jeton est un défenseur, pas
/// un ennemi errant (MKUE p. 7, 4.b et 4.c ; spec 043).
///
/// Spec 062 : les cités aussi ; leurs jetons sont leurs défenseurs.
pub(crate) fn is_fortified_site(state: &GameState, hex: Hex) -> bool {
    matches!(
        state.map.space(hex).and_then(|space| space.feature),
        Some(Feature::Site(Site::Keep | Site::MageTower | Site::City(_)))
    )
}

/// Vrai si cette case porte une cité (spec 062).
pub(crate) fn is_city(state: &GameState, hex: Hex) -> bool {
    matches!(
        state.map.space(hex).and_then(|space| space.feature),
        Some(Feature::Site(Site::City(_)))
    )
}

/// Le niveau que le scénario donne à la prochaine cité révélée, selon le nombre de cités déjà en
/// jeu ; `None` si les cités sont fermées (MKUE p. 18, *City Levels* ; spec 062).
pub(crate) fn next_city_level(state: &GameState) -> Option<u8> {
    match state.scenario?.cities() {
        CityRule::Closed => None,
        CityRule::Levels(levels) => Some(levels.level(state.cities.len())),
    }
}

/// Vrai si un pas vers cette case est un assaut : forteresse ou tour de mage non conquise
/// (MKUE p. 7, 4.a ; spec 043).
///
/// Spec 062 : une cité en jeu non conquise ; une cité fermée n'est jamais en jeu.
pub(crate) fn is_assault_target(state: &GameState, hex: Hex) -> bool {
    if is_city(state, hex) {
        return state.city(hex).is_some_and(|city| !city.is_conquered());
    }
    is_fortified_site(state, hex) && !state.shields.contains_key(&hex)
}

/// Les cases des ennemis errants : les jetons du plateau de jeu sans les défenseurs (spec 043) ni
/// les jetons d'un site d'aventure (spec 044 ; MKUE p. 7, 4.d). Seuls eux bloquent une case, sont
/// provoqués, défiés ou évités par les trajets.
pub(crate) fn rampaging_enemies(state: &GameState) -> Vec<Hex> {
    state
        .enemy_tokens
        .keys()
        .copied()
        .filter(|&hex| !is_fortified_site(state, hex) && !is_adventure_site(state, hex))
        .collect()
}

/// Vrai si cette case porte un site d'aventure (spec 044).
pub(crate) fn is_adventure_site(state: &GameState, hex: Hex) -> bool {
    matches!(
        state.map.space(hex).and_then(|space| space.feature),
        Some(Feature::Site(site)) if site.is_adventure()
    )
}

/// Les forteresses et tours de mage du plateau de jeu, qu'un pas ou un trajet peut assaillir
/// (spec 043).
fn assault_targets(state: &GameState) -> Vec<Hex> {
    state
        .map
        .tiles()
        .flat_map(|(position, tile)| tile_sites(tile, position))
        .filter(|&(hex, _)| is_assault_target(state, hex))
        .map(|(hex, _)| hex)
        .collect()
}

/// L'assaut qui commence par le pas de `from` vers le site `site` (MKUE p. 7, 4.a ; spec 043) :
/// Réputation −1 quelle que soit l'issue (Compendium, *Combat* Q16). Sans défenseur (plus aucun
/// jeton à la révélation, L4), le site est conquis aussitôt.
fn assault_events(state: &GameState, player: PlayerId, from: Hex, site: Hex) -> Vec<Event> {
    let mut events = vec![
        Event::AssaultStarted { player, site, from },
        Event::ReputationChanged { player, change: -1 },
    ];
    if !state.has_enemy_token(site) {
        // Spec 062 (L1) : une cité sans défenseur est conquise sans Bouclier.
        events.push(if state.city(site).is_some() {
            Event::CityConquered {
                hex: site,
                leader: player,
            }
        } else {
            Event::SiteConquered { player, hex: site }
        });
        // Spec 064 : la dernière cité conquise annonce le dernier tour.
        events.extend(crate::game_end::conquest_last_turn(state, player, &events));
    }
    events
}

/// De jour (`time`), les défenseurs face cachée voisins de `hex` sont retournés (cartes de site ;
/// Compendium, *Movement* S8 ; spec 043, Q4) ; de nuit, rien (*Miscellaneous* S10), sauf ceux
/// d'une cité (carte City : *even at Night* ; spec 062). `pending` : les événements produits mais
/// pas encore appliqués (défenseurs posés ou déjà retournés).
fn reveals_next_to(state: &GameState, time: TimeOfDay, hex: Hex, pending: &[Event]) -> Vec<Event> {
    let mut hidden: BTreeMap<Hex, Vec<EnemyToken>> = state
        .face_down
        .iter()
        .map(|hex| (*hex, state.enemy_tokens_at(*hex).to_vec()))
        .collect();
    let mut cities: Vec<Hex> = state.cities.keys().copied().collect();
    for event in pending {
        match event {
            Event::DefenderPlaced { token, hex } => {
                hidden.entry(*hex).or_default().push(*token);
            }
            Event::EnemyTokenRevealed { hex, .. } => {
                hidden.remove(hex);
            }
            Event::CityPlaced { hex, .. } => cities.push(*hex),
            _ => {}
        }
    }
    hidden
        .into_iter()
        .filter(|(defender, _)| {
            defender.is_adjacent(hex) && (time == TimeOfDay::Day || cities.contains(defender))
        })
        .flat_map(|(hex, tokens)| {
            tokens
                .into_iter()
                .map(move |token| Event::EnemyTokenRevealed { hex, token })
        })
        .collect()
}

/// Le moment des révélations du joueur : le Jour avec l'Amulette du soleil (spec 055, Q6), sinon
/// celui du plateau.
fn reveal_time(state: &GameState, player: PlayerId) -> TimeOfDay {
    state
        .player(player)
        .map_or(state.time_of_day, |p| p.gold_time(state.time_of_day))
}

/// La figurine entre sur `hex` : son jeton de ruines face cachée est retourné (carte *Ancient
/// Ruins* ; Compendium, *Movement* S8 ; spec 045), sans rien tirer ni obliger à rien (FAQ p3).
fn ruin_reveal(state: &GameState, hex: Hex) -> Option<Event> {
    let placed = state.ruins.get(&hex)?;
    (!placed.face_up).then_some(Event::RuinTokenRevealed {
        hex,
        token: placed.token,
    })
}

/// Le joueur doit choisir tant qu'il a une action légale : une carte en main (toute carte se
/// joue au moins de côté), un déplacement (spec 014) ou une exploration (spec 015) qu'il peut
/// payer, ou finir son tour (spec 016).
pub fn pending_decision(state: &GameState) -> Option<PendingDecision> {
    // Spec 023 : entre deux manches, et après la dernière, personne ne choisit ; spec 063 : ni
    // après la fin de partie.
    // Spec 055 (Q2) : sauf le choix des bannières à reprendre, qui attend entre les deux manches.
    if state.game_end.is_some()
        || (state.round_over && !state.players.iter().any(|p| p.banner_reclaim))
    {
        return None;
    }
    let player = PlayerId::SOLO;
    (!available_actions(state, player).is_empty()).then_some(PendingDecision {
        player,
        kind: DecisionKind::ChooseAction,
    })
}

/// Joue une action sur une copie de l'état, sans journal (pour le MCTS).
pub fn simulate(
    state: &GameState,
    player: PlayerId,
    action: &Action,
    rngs: &mut GameRngs,
) -> Result<GameState, ActionError> {
    let events = decide(state, player, action, rngs)?;
    Ok(events.iter().fold(state.clone(), apply))
}
