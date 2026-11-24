//! Déplacement (spec 014) : moment de la journée et coût de déplacement.

use std::collections::{HashMap, HashSet};

use crate::{
    Direction, Feature, Hex, Map, Modifier, ModifierKind, Site, SpaceContent, Terrain,
    TerrainChange,
};

/// Moment de la journée, côté visible du plateau Jour/Nuit.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum TimeOfDay {
    /// La partie commence de jour (MKUE p. 3).
    #[default]
    Day,
    Night,
}

impl TimeOfDay {
    /// Le plateau Jour/Nuit retourné : Jour et Nuit alternent (MKUE p. 3 ; p. 4, 2.a).
    pub fn flipped(self) -> TimeOfDay {
        match self {
            TimeOfDay::Day => TimeOfDay::Night,
            TimeOfDay::Night => TimeOfDay::Day,
        }
    }
}

/// Les règles de déplacement du moment (spec 052, ADR 0008) : le plateau Jour/Nuit et les
/// modificateurs de terrain en cours (`TerrainCost`, `Passable`). Construites depuis un
/// `TimeOfDay` pour les appels sans modificateur.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MoveRules {
    pub time: TimeOfDay,
    /// Les sortes de terrain en cours, dans l'ordre où elles ont été posées.
    pub terrain_modifiers: Vec<ModifierKind>,
    /// Distorsion spatiale en cours : les cases à 2 sont voisines, rien ne provoque (spec 057).
    pub space_bending: bool,
}

impl From<TimeOfDay> for MoveRules {
    fn from(time: TimeOfDay) -> Self {
        MoveRules {
            time,
            terrain_modifiers: Vec::new(),
            space_bending: false,
        }
    }
}

impl From<&MoveRules> for MoveRules {
    fn from(rules: &MoveRules) -> Self {
        rules.clone()
    }
}

impl MoveRules {
    /// Les règles du moment `time` avec les modificateurs de terrain de `modifiers`.
    pub fn new(time: TimeOfDay, modifiers: &[Modifier]) -> Self {
        MoveRules {
            time,
            terrain_modifiers: modifiers
                .iter()
                .filter_map(|modifier| match modifier.kind {
                    kind @ (ModifierKind::TerrainCost { .. } | ModifierKind::Passable(_)) => {
                        Some(kind)
                    }
                    // Spec 055 : les forêts à 3 la nuit (Amulette du soleil), les déserts à 3 le
                    // jour (Amulette des ténèbres).
                    ModifierKind::DayRules => Some(set_to_three(Terrain::Forest)),
                    ModifierKind::NightMana => Some(set_to_three(Terrain::Desert)),
                    _ => None,
                })
                .collect(),
            space_bending: modifiers
                .iter()
                .any(|modifier| modifier.kind == ModifierKind::SpaceBending),
        }
    }

    /// Le coût d'un terrain sous ces règles, `None` s'il reste infranchissable. Les changements de
    /// coût s'appliquent dans l'ordre le plus favorable au joueur (MKUE p. 7, 7.a ; Compendium,
    /// *Song of Wind* S4) ; un terrain rendu franchissable n'a de coût que celui qu'un changement
    /// lui fixe (Pont de glace renforcé).
    pub fn terrain_cost(&self, terrain: Terrain) -> Option<u8> {
        let passable = self
            .terrain_modifiers
            .contains(&ModifierKind::Passable(terrain));
        let printed = terrain_cost(terrain, self.time);
        if printed.is_none() && !passable {
            return None;
        }
        let changes: Vec<TerrainChange> = self
            .terrain_modifiers
            .iter()
            .filter_map(|kind| match kind {
                ModifierKind::TerrainCost { terrains, change } if terrains.contains(terrain) => {
                    Some(*change)
                }
                _ => None,
            })
            .collect();
        best_cost(printed, &changes)
    }
}

/// Le coût de jour de la forêt ou de nuit du désert (spec 055).
fn set_to_three(terrain: Terrain) -> ModifierKind {
    ModifierKind::TerrainCost {
        terrains: crate::Terrains::of(&[terrain]),
        change: TerrainChange::SetTo(3),
    }
}

/// Le coût le plus bas parmi les ordres possibles des changements `changes` à partir de `cost`
/// (`None` : pas encore de coût, seul un `SetTo` lui en donne un). Au plus quelques changements
/// sont en jeu : essayer chaque ordre suffit.
fn best_cost(cost: Option<u8>, changes: &[TerrainChange]) -> Option<u8> {
    if changes.is_empty() {
        return cost;
    }
    (0..changes.len())
        .filter_map(|first| {
            let next = match (cost, changes[first]) {
                (Some(cost), change) => Some(change.applied(cost)),
                (None, TerrainChange::SetTo(value)) => Some(value),
                (None, TerrainChange::Reduce { .. }) => None,
            };
            let mut rest = changes.to_vec();
            rest.remove(first);
            best_cost(next, &rest)
        })
        .min()
}

/// Ce que coûte l'entrée dans une case.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MoveCost {
    Points(u8),
    /// Lac, montagne, littoral (MKUE p. 7, 3.a).
    Inaccessible,
    /// Site fortifié : y entrer est un assaut (MKUE p. 7, 4.a), pas encore disponible
    /// (specs 043 et 062). Garde le coût qu'aura l'entrée.
    Fortified(u8),
}

/// Coût de déplacement d'une case selon le plateau Jour/Nuit (MKUE p. 7, 3) et les modificateurs
/// de terrain (spec 052).
pub fn move_cost(space: SpaceContent, rules: impl Into<MoveRules>) -> MoveCost {
    cost_under(space, &rules.into())
}

fn cost_under(space: SpaceContent, rules: &MoveRules) -> MoveCost {
    let site = match space.feature {
        Some(Feature::Site(site)) => Some(site),
        _ => None,
    };
    // MKUE p. 7, 3.a, note : une cité coûte 2 quel que soit son terrain.
    let points = if matches!(site, Some(Site::City(_))) {
        2
    } else {
        match rules.terrain_cost(space.terrain) {
            Some(points) => points,
            None => return MoveCost::Inaccessible,
        }
    };
    if site.is_some_and(Site::is_fortified) {
        MoveCost::Fortified(points)
    } else {
        MoveCost::Points(points)
    }
}

/// Vrai si `hex` est une case sécurisée (MKUE p. 7, 7.d ; spec 052) : accessible sans effet, et pas
/// un site fortifié non conquis (`conquered` : les sites conquis).
pub fn is_safe_space(map: &Map, hex: Hex, conquered: &[Hex]) -> bool {
    let Some(space) = map.space(hex) else {
        return false;
    };
    match move_cost(space, TimeOfDay::Day) {
        MoveCost::Points(_) => true,
        MoveCost::Inaccessible => false,
        MoveCost::Fortified(_) => conquered.contains(&hex),
    }
}

fn terrain_cost(terrain: Terrain, time: TimeOfDay) -> Option<u8> {
    let night = time == TimeOfDay::Night;
    match terrain {
        Terrain::Plains => Some(2),
        Terrain::Hills => Some(3),
        Terrain::Forest => Some(if night { 5 } else { 3 }),
        Terrain::Wasteland => Some(4),
        Terrain::Desert => Some(if night { 3 } else { 5 }),
        Terrain::Swamp => Some(5),
        // Le littoral n'est pas sur le plateau Jour/Nuit : la mer est hors du jeu.
        Terrain::Lake | Terrain::Mountain | Terrain::Sea => None,
    }
}

impl Site {
    /// Forteresse, tour de mage, cité (MKUE p. 7, 4.a).
    pub fn is_fortified(self) -> bool {
        matches!(self, Site::Keep | Site::MageTower | Site::City(_))
    }
}

/// Les cases révélées à `max` pas au plus de `from` par des cases que `passable` accepte, avec
/// leur distance, dans l'ordre de la recherche (distance, puis directions de [`Direction::ALL`]) ;
/// `from` exclue (déplacement par effet, spec 057).
pub fn reach(
    map: &Map,
    from: Hex,
    max: u8,
    passable: impl Fn(SpaceContent) -> bool,
) -> Vec<(Hex, u8)> {
    let mut seen = HashSet::from([from]);
    let mut result = Vec::new();
    let mut frontier = vec![from];
    for distance in 1..=max {
        let mut next = Vec::new();
        for hex in frontier {
            for neighbor in hex.neighbors() {
                if seen.contains(&neighbor) {
                    continue;
                }
                let Some(space) = map.space(neighbor) else {
                    continue;
                };
                if !passable(space) {
                    continue;
                }
                seen.insert(neighbor);
                result.push((neighbor, distance));
                next.push(neighbor);
            }
        }
        frontier = next;
    }
    result
}

/// Les cases où un pas peut mener depuis `hex` : ses voisines dans l'ordre de [`Direction::ALL`],
/// puis avec Distorsion spatiale (`space_bending`) les douze cases à distance 2, pour chaque
/// direction la case en ligne droite puis celle d'entre elle et la direction suivante (spec 057).
pub fn steps(hex: Hex, space_bending: bool) -> Vec<Hex> {
    let mut result: Vec<Hex> = hex.neighbors().to_vec();
    if space_bending {
        for (index, direction) in Direction::ALL.into_iter().enumerate() {
            let next = Direction::ALL[(index + 1) % Direction::ALL.len()];
            let near = hex.neighbor(direction);
            result.push(near.neighbor(direction));
            result.push(near.neighbor(next));
        }
    }
    result
}

/// Trajet retenu vers une case (spec 017) : les cases traversées depuis la case de départ
/// (exclue), l'arrivée comprise, et le coût total.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Route {
    pub to: Hex,
    pub steps: Vec<Hex>,
    pub cost: u32,
}

/// Trajets retenus depuis `from` vers chaque case qu'on peut rejoindre (spec 017), du premier au
/// dernier dans l'ordre de départage : coût, puis nombre de cases, puis suite de directions dans
/// l'ordre de [`Direction::ALL`]. On ne traverse que des cases posées dont le coût est un nombre de
/// points (ni infranchissable, ni site fortifié). Ne dépend pas des points disponibles.
pub fn routes(map: &Map, from: Hex, rules: impl Into<MoveRules>) -> Vec<Route> {
    routes_avoiding(map, from, rules, &[])
}

/// Les jetons d'`enemies` que provoque un pas de `from` à `to` : ceux qui touchent les deux cases
/// (MKUE p. 7, 4.c), dans l'ordre d'`enemies`.
pub fn provoked(from: Hex, to: Hex, enemies: &[Hex]) -> Vec<Hex> {
    enemies
        .iter()
        .copied()
        .filter(|enemy| enemy.is_adjacent(from) && enemy.is_adjacent(to))
        .collect()
}

/// Comme [`routes`], avec les jetons d'ennemis errants d'`enemies` (spec 030) : un trajet n'entre
/// jamais sur leur case (MKUE p. 7, 4.b) et ne contient aucun pas qui en provoque un (4.c ; choix
/// de Mathieu, spec 030, C19).
pub fn routes_avoiding(
    map: &Map,
    from: Hex,
    rules: impl Into<MoveRules>,
    enemies: &[Hex],
) -> Vec<Route> {
    routes_ending_in(map, from, rules, enemies, &[], &[])
}

/// Comme [`routes_avoiding`], et des trajets qui finissent sur un site d'`assaults` (forteresse ou
/// tour de mage non conquise, au coût de son terrain) sans jamais le traverser (spec 043, Q2) ;
/// les sites de `conquered` sont des cases ordinaires de leur terrain.
pub fn routes_ending_in(
    map: &Map,
    from: Hex,
    rules: impl Into<MoveRules>,
    enemies: &[Hex],
    assaults: &[Hex],
    conquered: &[Hex],
) -> Vec<Route> {
    let rules = rules.into();
    // Dijkstra : la clé (coût, longueur, directions) est un ordre total qui se prolonge bien
    // (ajouter le même pas à deux trajets garde leur ordre), donc le premier trajet fixé est le bon.
    // Le plateau de jeu compte au plus quelques centaines de cases : une recherche linéaire du
    // minimum suffit.
    // Un pas se range par sa place dans `steps` (les directions d'abord, spec 057).
    type Key = (u32, usize, Vec<u8>);
    let mut settled = HashSet::from([from]);
    let mut frontier: HashMap<Hex, (Key, Vec<Hex>)> = HashMap::new();
    let mut result = vec![];
    let mut current = (from, (0, 0, vec![]), vec![]);
    loop {
        let (hex, (cost, _, directions), steps_so_far) = current;
        // Un site assailli finit le trajet : on ne repart pas de lui.
        let terminal = assaults.contains(&hex);
        let candidates = if terminal {
            Vec::new()
        } else {
            steps(hex, rules.space_bending)
        };
        for (index, next) in (0u8..).zip(candidates) {
            // Spec 057 : avec Distorsion spatiale, aucun pas ne provoque.
            if settled.contains(&next)
                || enemies.contains(&next)
                || (!rules.space_bending && !provoked(hex, next, enemies).is_empty())
            {
                continue;
            }
            let points = match map.space(next).map(|space| cost_under(space, &rules)) {
                Some(MoveCost::Points(points)) => points,
                Some(MoveCost::Fortified(points))
                    if assaults.contains(&next) || conquered.contains(&next) =>
                {
                    points
                }
                _ => continue,
            };
            let mut next_directions = directions.clone();
            next_directions.push(index);
            let key = (
                cost + u32::from(points),
                steps_so_far.len() + 1,
                next_directions,
            );
            if frontier.get(&next).is_none_or(|(best, _)| key < *best) {
                let mut next_steps = steps_so_far.clone();
                next_steps.push(next);
                frontier.insert(next, (key, next_steps));
            }
        }
        let Some(best) = frontier
            .iter()
            // Deux cases n'ont jamais la même suite de directions : le minimum est unique.
            .min_by(|(_, (a, _)), (_, (b, _))| a.cmp(b))
            .map(|(hex, _)| *hex)
        else {
            return result;
        };
        let Some((key, steps)) = frontier.remove(&best) else {
            return result;
        };
        settled.insert(best);
        result.push(Route {
            to: best,
            steps: steps.clone(),
            cost: key.0,
        });
        current = (best, key, steps);
    }
}
