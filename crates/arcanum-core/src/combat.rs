//! Combat contre des ennemis errants (spec 031) : état du combat, et les actions qui en
//! dépendent.

use crate::GameRngs;
use crate::engine::combinations;
use crate::engine::draw_enemy_token;
use crate::state::PlayerState;
use crate::{
    Action, ActionError, AttackKind, CardId, DamageTarget, DeedCard, Effect, Element, Enemy,
    EnemyAbility, EnemyAttack, EnemyColor, EnemyToken, Event, GameState, Hex, PlayerId, Resistance,
    SidewaysEffect, UnitCard,
};
use crate::{
    BasicColor, ConvertedPoints, EnemyModifier, EnemyModifierKind, Modifier, ModifierKind,
    ModifierSource, PointPool,
};
use crate::{Feature, Rampaging};

/// Les quatre phases, dans l'ordre (MKUE p. 8, 3.a à 3.d).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum CombatPhase {
    RangedAndSiege,
    Block,
    AssignDamage,
    Attack,
}

/// Un ennemi du combat, désigné par son jeton (spec 044, Q1), sur la case `hex` : public.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CombatEnemy {
    pub hex: Hex,
    pub token: EnemyToken,
    /// Vaincu : son jeton a quitté le plateau de jeu, il ne prend plus part au combat
    /// (MKUE p. 9, 5.a).
    pub defeated: bool,
    /// Paré : il n'inflige pas de dégâts mais reste en jeu (MKUE p. 9, *Block Phase* 7).
    pub blocked: bool,
    /// Ciblé par une attaque ratée dans la phase en cours : plus de cible avant la phase suivante
    /// (Q10).
    pub attacked_this_phase: bool,
    /// Visé par une parade ratée : plus de cible de parade dans ce combat (Q10).
    pub block_failed: bool,
    /// La créature invoquée par cet Invocateur, pendant les phases de parade et d'assignation des
    /// dégâts : elle attaque à sa place (MKUE p. 9, *Block Phase* 2.a ; spec 033).
    pub summoned: Option<EnemyToken>,
    /// Ses dégâts ont été faits dans la phase d'assignation (spec 042).
    pub damage_assigned: bool,
    /// Le défenseur d'un assaut, fortifié par son site (MKUE p. 9, *Ranged and Siege Attack
    /// Phase* 3.a ; spec 043).
    pub site_fortified: bool,
    /// Ce qui le change jusqu'à la fin du combat : armure réduite, n'attaque pas (ADR 0008,
    /// spec 053).
    pub modifiers: Vec<EnemyModifier>,
    /// Détruit par un sort (spec 054) : aussi `defeated`.
    pub destroyed: bool,
    /// La phase où il a été vaincu ou détruit (Déluge de flammes, spec 054, Q4).
    pub defeated_in: Option<CombatPhase>,
    /// Le défenseur d'une cité, de sa couleur : il a le bonus de la cité jusqu'à la fin du
    /// combat, sa créature invoquée aussi (MKUE p. 10, *City Assaults* 1 ; spec 062).
    pub city: Option<BasicColor>,
}

impl CombatEnemy {
    /// L'armure qui compte : celle du jeton, moins les réductions, jamais sous leur minimum
    /// (spec 053).
    ///
    /// Spec 062 : la cité blanche ajoute 1, avant toute réduction (Compendium, *Cities* S7).
    pub fn effective_armor(&self) -> u8 {
        let city = u8::from(self.city == Some(BasicColor::White));
        self.modifiers.iter().fold(
            self.token.enemy().definition().armor + city,
            |armor, modifier| match modifier.kind {
                EnemyModifierKind::ArmorReduction { by, min } if armor > min => {
                    armor.saturating_sub(by).max(min)
                }
                _ => armor,
            },
        )
    }

    /// Les résistances qui comptent : celles du jeton, moins celles perdues (spec 054).
    pub fn effective_resistances(&self) -> Vec<Resistance> {
        let lost = |resistance: Resistance| {
            self.modifiers.iter().any(|modifier| match modifier.kind {
                EnemyModifierKind::LosesResistances => true,
                EnemyModifierKind::LosesResistance(lost) => lost == resistance,
                _ => false,
            })
        };
        self.token
            .enemy()
            .definition()
            .resistances
            .iter()
            .copied()
            .filter(|&resistance| !lost(resistance))
            .collect()
    }

    /// A perdu ses fortifications : ni celle de son jeton, ni celle de son site (spec 054).
    pub fn lost_fortifications(&self) -> bool {
        self.modifiers
            .iter()
            .any(|modifier| modifier.kind == EnemyModifierKind::LosesFortifications)
    }

    /// N'attaque pas dans ce combat (spec 053).
    pub fn does_not_attack(&self) -> bool {
        self.modifiers
            .iter()
            .any(|modifier| modifier.kind == EnemyModifierKind::DoesNotAttack)
    }

    /// Vrai si l'effet d'une carte ou d'une capacité de cette couleur l'atteint : pas un effet
    /// rouge contre une résistance au Feu, ni bleu contre une résistance à la Glace (MKUE p. 8,
    /// *Combat with Enemies* 5.a et 5.b ; spec 053).
    pub fn is_affected_by(&self, color: Option<BasicColor>) -> bool {
        // Spec 054 : les résistances qui comptent, sans celles perdues.
        let resistances = self.effective_resistances();
        match color {
            Some(BasicColor::Red) => !resistances.contains(&Resistance::Fire),
            Some(BasicColor::Blue) => !resistances.contains(&Resistance::Ice),
            _ => true,
        }
    }

    /// En jeu, non paré, qui attaque, et dont les dégâts restent à faire (MKUE p. 9, *Assign
    /// Damage Phase* 1 ; spec 042).
    pub fn has_pending_damage(&self) -> bool {
        !self.defeated && !self.blocked && !self.damage_assigned && self.damage().is_some()
    }

    /// Fortifié par la capacité imprimée sur son jeton (MKUE p. 24, *Defensive*).
    pub fn is_fortified(&self) -> bool {
        self.token
            .enemy()
            .definition()
            .abilities
            .contains(&EnemyAbility::Fortified)
    }

    /// Fortifié combien de fois : par son site, par sa capacité (spec 043). Deux fois, il n'est
    /// pas une cible de la phase d'attaque à distance (FAQ p3 ; Compendium, *Combat* S13). Plus
    /// du tout après Exposition (spec 054).
    pub fn fortification(&self) -> u8 {
        if self.lost_fortifications() {
            return 0;
        }
        u8::from(self.site_fortified) + u8::from(self.is_fortified())
    }

    /// Le jeton dont l'attaque compte : la créature invoquée s'il y en a une, sinon l'ennemi
    /// lui-même ; `None` pour un Invocateur sans créature (MKUE p. 9, *Block Phase* 2.a).
    pub fn attacker(&self) -> Option<EnemyToken> {
        // Spec 053 : un ennemi qui n'attaque pas n'a ni parade ni dégâts.
        if self.does_not_attack() {
            return None;
        }
        match self.summoned {
            Some(summoned) => Some(summoned),
            None => attack_value(self.token).map(|_| self.token),
        }
    }

    /// Valeur de l'attaque qui compte, quel que soit l'élément ; dans la cité bleue, +2 pour
    /// une attaque de glace ou de feu, +1 de feu glacé (MKUE p. 10, 1.b ; Compendium, *Cities* S8 ;
    /// spec 062).
    fn attack_value(&self) -> Option<u8> {
        let attacker = self.attacker()?;
        let value = attack_value(attacker)?;
        let bonus = match (self.city, attacker.enemy().definition().attack) {
            (Some(BasicColor::Blue), EnemyAttack::Fire(_) | EnemyAttack::Ice(_)) => 2,
            (Some(BasicColor::Blue), EnemyAttack::ColdFire(_)) => 1,
            _ => 0,
        };
        Some(value + bonus)
    }

    /// Élément de l'attaque qui compte.
    fn attack_element(&self) -> Option<Element> {
        self.attacker()
            .and_then(|token| token.enemy().definition().attack.element())
    }

    /// Imprimée sur le jeton qui attaque, ou gagnée de la cité (spec 062).
    fn attacker_has(&self, ability: EnemyAbility) -> bool {
        self.attacker()
            .is_some_and(|token| token.enemy().definition().abilities.contains(&ability))
            || self.city_ability() == Some(ability)
    }

    /// La capacité que la cité donne au jeton qui attaque, s'il ne l'a pas déjà : Brutal dans la
    /// cité rouge, Toxique dans la verte, pour une attaque physique (MKUE p. 10, 1.c et 1.d ;
    /// Compendium, *Cities* S5, S6).
    fn city_ability(&self) -> Option<EnemyAbility> {
        let attacker = self.attacker()?;
        let definition = attacker.enemy().definition();
        if !matches!(definition.attack, EnemyAttack::Physical(_)) {
            return None;
        }
        let ability = match self.city? {
            BasicColor::Red => EnemyAbility::Brutal,
            BasicColor::Green => EnemyAbility::Poison,
            _ => return None,
        };
        (!definition.abilities.contains(&ability)).then_some(ability)
    }

    /// La parade à atteindre : le double de l'attaque contre un ennemi vif (MKUE p. 9, *Block
    /// Phase* 6.a ; spec 033).
    pub fn block_needed(&self) -> Option<u32> {
        let attack = u32::from(self.attack_value()?);
        Some(if self.attacker_has(EnemyAbility::Swift) {
            2 * attack
        } else {
            attack
        })
    }

    /// Les dégâts s'il n'est pas paré : le double de l'attaque pour un ennemi brutal (MKUE p. 9,
    /// *Assign Damage Phase* 2.a ; spec 033).
    pub fn damage(&self) -> Option<u32> {
        let attack = u32::from(self.attack_value()?);
        Some(if self.attacker_has(EnemyAbility::Brutal) {
            2 * attack
        } else {
            attack
        })
    }

    /// Les Blessures du héros d'armure `armor` s'il n'est pas paré : une par tranche entamée de
    /// son armure (MKUE p. 9, *Assign Damage Phase* 5.b) ; l'armure est celle du niveau (spec 035).
    pub fn wounds_if_unblocked(&self, armor: u8) -> Option<u32> {
        self.damage()
            .map(|damage| damage.div_ceil(u32::from(armor.max(1))))
    }
}

/// Valeur de l'attaque imprimée sur un jeton ; `None` pour une invocation.
fn attack_value(token: EnemyToken) -> Option<u8> {
    match token.enemy().definition().attack {
        EnemyAttack::Physical(value)
        | EnemyAttack::Fire(value)
        | EnemyAttack::Ice(value)
        | EnemyAttack::ColdFire(value) => Some(value),
        EnemyAttack::Summon => None,
    }
}

/// Le bonus d'Endurance givrée renforcée contre cet ennemi : +1 par capacité et par résistance
/// imprimées sur le jeton, +1 par couleur d'attaque (feu, glace ; feu glacé : 2) ; l'invocation ne
/// compte pas (Compendium, *Cold Toughness* S1 ; FAQ p2 ; spec 033).
pub fn cold_toughness_bonus(enemy: Enemy) -> u8 {
    let definition = enemy.definition();
    let colors = match definition.attack {
        EnemyAttack::Fire(_) | EnemyAttack::Ice(_) => 1,
        EnemyAttack::ColdFire(_) => 2,
        EnemyAttack::Physical(_) | EnemyAttack::Summon => 0,
    };
    let count = definition.abilities.len() + definition.resistances.len() + colors;
    u8::try_from(count).unwrap_or(u8::MAX)
}

impl CombatEnemy {
    /// L'attaque qui compte, quel que soit l'élément : celle du jeton qui attaque, plus le bonus
    /// de la cité bleue (spec 062) ; `None` sans attaquant.
    pub fn effective_attack(&self) -> Option<u8> {
        self.attack_value()
    }

    /// Vrai si le jeton qui attaque a cette capacité, imprimée ou gagnée de sa cité (spec 062).
    pub fn has_ability(&self, ability: EnemyAbility) -> bool {
        self.attacker_has(ability)
    }

    /// Les capacités gagnées de sa cité (Brutal de la cité rouge, Toxique de la verte), absentes du
    /// jeton qui attaque (spec 062).
    pub fn city_abilities(&self) -> Vec<EnemyAbility> {
        self.city_ability().into_iter().collect()
    }
}

/// Une attaque ou une parade compte en plein, ou divisée par deux (MKUE p. 9, *Ranged and Siege
/// Attack Phase* 4.a ; *Block Phase* 5).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Efficiency {
    Full,
    Halved,
}

/// Efficacité d'une parade de l'élément `block` contre une attaque de l'élément `attack` (MKUE
/// p. 9, *Block Phase* 5.a).
pub fn block_efficiency(attack: Element, block: Element) -> Efficiency {
    let efficient = match attack {
        Element::Physical => true,
        Element::Fire => matches!(block, Element::Ice | Element::ColdFire),
        Element::Ice => matches!(block, Element::Fire | Element::ColdFire),
        Element::ColdFire => block == Element::ColdFire,
    };
    if efficient {
        Efficiency::Full
    } else {
        Efficiency::Halved
    }
}

/// Efficacité d'une attaque de l'élément `element` contre un groupe dont chaque cible a ses
/// résistances : il suffit d'une cible résistante pour tout le groupe ; le feu glacé seulement si
/// une même cible résiste au feu et à la glace (MKUE p. 9, *Ranged and Siege Attack Phase* 4.a ;
/// Compendium, *Combat* S12).
pub fn attack_efficiency<'a>(
    element: Element,
    targets: impl IntoIterator<Item = &'a [Resistance]>,
) -> Efficiency {
    let resists = |resistances: &[Resistance]| match element {
        Element::Physical => resistances.contains(&Resistance::Physical),
        Element::Fire => resistances.contains(&Resistance::Fire),
        Element::Ice => resistances.contains(&Resistance::Ice),
        Element::ColdFire => {
            resistances.contains(&Resistance::Fire) && resistances.contains(&Resistance::Ice)
        }
    };
    if targets.into_iter().any(resists) {
        Efficiency::Halved
    } else {
        Efficiency::Full
    }
}

/// Les points d'une attaque ou d'une parade, par élément (spec 032).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ElementPoints {
    pub physical: u32,
    pub fire: u32,
    pub ice: u32,
    pub cold_fire: u32,
}

impl ElementPoints {
    pub fn get(&self, element: Element) -> u32 {
        match element {
            Element::Physical => self.physical,
            Element::Fire => self.fire,
            Element::Ice => self.ice,
            Element::ColdFire => self.cold_fire,
        }
    }

    fn add(&mut self, element: Element, amount: u32) {
        let points = match element {
            Element::Physical => &mut self.physical,
            Element::Fire => &mut self.fire,
            Element::Ice => &mut self.ice,
            Element::ColdFire => &mut self.cold_fire,
        };
        *points += amount;
    }

    /// La somme des points inefficaces (avant division) et celle des autres.
    pub fn split(&self, efficiency: impl Fn(Element) -> Efficiency) -> (u32, u32) {
        Element::ALL
            .into_iter()
            .fold((0, 0), |(halved, full), element| {
                match efficiency(element) {
                    Efficiency::Halved => (halved + self.get(element), full),
                    Efficiency::Full => (halved, full + self.get(element)),
                }
            })
    }

    /// Le total qui compte : une seule division, arrondie en dessous, sur la somme des inefficaces
    /// (MKUE p. 9, *Ranged and Siege Attack Phase* 4.a ; *Block Phase* 5.b).
    pub fn total(&self, efficiency: impl Fn(Element) -> Efficiency) -> u32 {
        let (halved, full) = self.split(efficiency);
        halved / 2 + full
    }
}

/// L'attaque ou la parade en cours, et ses points par élément (MKUE p. 9).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InProgress {
    Attack {
        targets: Vec<EnemyToken>,
        points: ElementPoints,
    },
    Block {
        target: EnemyToken,
        points: ElementPoints,
    },
}

/// Ce qui s'est passé dans le combat, pour l'écran de combat (spec 031, C36 bis) : une attaque ou
/// une parade validée, avec le total qui compte (spec 032), ou les Blessures d'un ennemi.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CombatRecord {
    Attack {
        phase: CombatPhase,
        targets: Vec<EnemyToken>,
        cards: Vec<CardId>,
        /// Les unités activées dans cette attaque (spec 042).
        units: Vec<UnitCard>,
        points: u32,
        /// La somme des armures des cibles.
        needed: u32,
        success: bool,
    },
    Block {
        target: EnemyToken,
        cards: Vec<CardId>,
        /// Les unités activées dans cette parade (spec 042).
        units: Vec<UnitCard>,
        points: u32,
        /// L'attaque de l'ennemi.
        needed: u32,
        success: bool,
    },
    /// Les dégâts d'un ennemi non paré (spec 031) : `attacker` est le jeton qui attaque (la
    /// créature invoquée pour un Invocateur), `damage` les points (doublés si brutal) ; les
    /// Blessures toxiques mises dans la défausse et les cartes défaussées par une attaque
    /// paralysante (spec 033).
    Damage {
        enemy: EnemyToken,
        attacker: EnemyToken,
        damage: u32,
        wounds: u8,
        poisoned: u8,
        paralyzed: u8,
    },
    /// Le héros est assommé après les dégâts de cet ennemi ; `cards` cartes défaussées de la main
    /// (spec 034).
    KnockedOut { enemy: EnemyToken, cards: u8 },
    /// La réputation gagnée par l'attaque qui a vaincu ces ennemis errants (spec 040).
    Reputation {
        targets: Vec<EnemyToken>,
        change: i8,
    },
    /// Les dégâts de cet ennemi assignés à une unité : ses Blessures (spec 042).
    UnitDamage {
        enemy: EnemyToken,
        unit: UnitCard,
        wounds: u8,
    },
    /// L'unité détruite par les dégâts de cet ennemi (spec 042).
    UnitDestroyed { enemy: EnemyToken, unit: UnitCard },
}

/// Le combat du tour, en cours ou fini ; vidé par la fin du tour.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Combat {
    /// Dans l'ordre des ennemis à combattre.
    pub enemies: Vec<CombatEnemy>,
    pub phase: CombatPhase,
    pub in_progress: Option<InProgress>,
    /// Les cartes jouées dans l'attaque ou la parade en cours, dans l'ordre.
    pub played: Vec<CardId>,
    /// Les unités activées dans l'attaque ou la parade en cours, dans l'ordre (spec 042).
    pub played_units: Vec<UnitCard>,
    /// Dans l'ordre où c'est arrivé.
    pub history: Vec<CombatRecord>,
    pub ended: bool,
    /// Blessures mises dans la main pendant ce combat, sans celles des attaques toxiques mises dans
    /// la défausse (MKUE p. 9, *Assign Damage Phase* 5.c ; spec 034).
    pub wounds_to_hand: u32,
    /// Le héros a été assommé dans ce combat : une seule fois (Compendium, *Combat* Q22).
    pub knocked_out: bool,
    /// L'ennemi dont les dégâts sont en cours d'assignation, et ses dégâts restants (spec 042).
    pub assigning: Option<(EnemyToken, u32)>,
    /// Une unité a été activée dans ce combat (Au cœur du combat, spec 053).
    pub unit_activated: bool,
    /// Ce que posera la parade en cours si elle réussit : attaque différée du Bouclier ardent,
    /// ennemi détruit du Bouclier explosif (spec 054).
    pub on_block_success: Vec<(ModifierKind, ModifierSource)>,
    /// Spec 055 (Épée de justice renforcée) : en phase d'attaque, l'Attaque physique ajoutée compte
    /// double.
    pub double_physical: bool,
    /// Les ennemis vaincus ou détruits, dans l'ordre : l'ordre des Boucliers d'une cité (spec 062).
    pub defeat_order: Vec<EnemyToken>,
    /// Au début de la phase de parade, l'invocation attend le choix de la Bannière de terreur
    /// (issue #8).
    pub summons_held: bool,
}

impl Combat {
    /// Les jetons ennemis affrontés par Déluge de flammes (spec 054, Q4) : ceux qui ne sont pas
    /// vaincus avant la phase en cours (vaincus pendant celle-ci compris) ; la créature invoquée
    /// n'est pas un jeton du combat.
    pub fn enemies_faced(&self) -> usize {
        self.enemies
            .iter()
            .filter(|enemy| !enemy.defeated || enemy.defeated_in == Some(self.phase))
            .count()
    }

    /// Vrai si l'effet se joue maintenant dans ce combat (pour l'écran : boutons de la main).
    pub fn accepts(&self, effect: Effect) -> bool {
        is_playable_in_combat(effect, self)
    }

    /// Efficacité de l'élément `element` dans l'attaque ou la parade en cours.
    fn efficiency_of(&self, element: Element) -> Option<Efficiency> {
        match &self.in_progress {
            Some(InProgress::Attack { targets, .. }) => Some(attack_efficiency(
                element,
                // Spec 054 : les résistances qui comptent.
                self.enemies_at(targets)
                    .map(CombatEnemy::effective_resistances)
                    .collect::<Vec<_>>()
                    .iter()
                    .map(Vec::as_slice),
            )),
            Some(InProgress::Block { target, .. }) => self
                .enemies_at(std::slice::from_ref(target))
                .find_map(CombatEnemy::attack_element)
                .map(|attack| block_efficiency(attack, element)),
            None => None,
        }
    }

    /// Comment compterait `effect` dans l'attaque ou la parade en cours : `None` si ce n'est pas
    /// une Attaque pendant une attaque, ou une Parade pendant une parade (repères de l'écran).
    pub fn efficiency(&self, effect: Effect) -> Option<Efficiency> {
        match (&self.in_progress, effect) {
            (Some(InProgress::Attack { .. }), Effect::Attack { element, .. })
            | (Some(InProgress::Block { .. }), Effect::Block { element, .. }) => {
                self.efficiency_of(element)
            }
            (Some(InProgress::Block { .. }), Effect::IceBlockPerAbility(_)) => {
                self.efficiency_of(Element::Ice)
            }
            (Some(InProgress::Block { .. }), Effect::SwiftBlock(_)) => {
                self.efficiency_of(Element::Physical)
            }
            _ => None,
        }
    }

    /// Le bonus d'Endurance givrée renforcée contre l'ennemi de la parade en cours : le jeton qui
    /// attaque, la créature invoquée pour un Invocateur (spec 033) ; `None` hors parade.
    pub fn block_bonus(&self) -> Option<u8> {
        let Some(InProgress::Block { target, .. }) = &self.in_progress else {
            return None;
        };
        self.enemies_at(std::slice::from_ref(target))
            .find_map(CombatEnemy::attacker)
            .map(|token| cold_toughness_bonus(token.enemy()))
    }

    /// Les points que l'effet ajoute à l'attaque ou à la parade en cours, avant division : ceux
    /// de l'effet, plus le bonus contre l'ennemi paré pour Endurance givrée renforcée.
    pub fn counted_points(&self, effect: Effect) -> Option<u32> {
        match effect {
            Effect::IceBlockPerAbility(amount) => {
                Some(u32::from(amount) + u32::from(self.block_bonus().unwrap_or(0)))
            }
            // Gardes d'Utem et Gardiens d'Altem (Lost Legion ; Compendium, *UTEM GUARDSMEN* S1).
            Effect::SwiftBlock(amount) => Some(if self.blocking_swift() {
                2 * u32::from(amount)
            } else {
                u32::from(amount)
            }),
            other => other.points().map(u32::from),
        }
    }

    /// Vrai si la parade en cours vise un ennemi dont l'attaque est Vif (spec 033).
    fn blocking_swift(&self) -> bool {
        let Some(InProgress::Block { target, .. }) = &self.in_progress else {
            return false;
        };
        self.enemies_at(std::slice::from_ref(target))
            .any(|enemy| enemy.attacker_has(EnemyAbility::Swift))
    }

    /// Les points inefficaces (avant division) et les autres de l'attaque ou de la parade en
    /// cours (ligne de calcul de l'écran).
    pub fn in_progress_split(&self) -> Option<(u32, u32)> {
        let points = match &self.in_progress {
            Some(InProgress::Attack { points, .. } | InProgress::Block { points, .. }) => points,
            None => return None,
        };
        Some(points.split(|element| self.efficiency_of(element).unwrap_or(Efficiency::Full)))
    }

    /// Le total qui compte de l'attaque ou de la parade en cours (spec 032).
    pub fn in_progress_total(&self) -> Option<u32> {
        self.in_progress_split()
            .map(|(halved, full)| halved / 2 + full)
    }

    pub(crate) fn new(enemies: &[(Hex, EnemyToken)]) -> Self {
        Combat {
            enemies: enemies
                .iter()
                .map(|&(hex, token)| CombatEnemy {
                    hex,
                    token,
                    defeated: false,
                    blocked: false,
                    attacked_this_phase: false,
                    block_failed: false,
                    summoned: None,
                    damage_assigned: false,
                    site_fortified: false,
                    modifiers: Vec::new(),
                    destroyed: false,
                    defeated_in: None,
                    city: None,
                })
                .collect(),
            phase: CombatPhase::RangedAndSiege,
            in_progress: None,
            played: Vec::new(),
            played_units: Vec::new(),
            history: Vec::new(),
            ended: false,
            wounds_to_hand: 0,
            knocked_out: false,
            assigning: None,
            unit_activated: false,
            double_physical: false,
            defeat_order: Vec::new(),
            summons_held: false,
            on_block_success: Vec::new(),
        }
    }

    /// Les dégâts qui restent à assigner pour cet ennemi : ceux de l'assignation en cours, ou
    /// tous (spec 042).
    pub fn remaining_damage(&self, enemy: &CombatEnemy) -> Option<u32> {
        match self.assigning {
            Some((token, remaining)) if self.designates(enemy, token) => Some(remaining),
            _ => enemy.damage(),
        }
    }

    pub(crate) fn start_phase(&mut self, phase: CombatPhase) {
        self.phase = phase;
        for enemy in &mut self.enemies {
            enemy.attacked_this_phase = false;
        }
    }

    /// Vrai si `token` désigne cet ennemi du combat (spec 044, Q1).
    pub fn designates(&self, enemy: &CombatEnemy, token: EnemyToken) -> bool {
        enemy.token == token
    }

    /// L'ennemi du combat désigné par ce jeton.
    pub fn enemy(&self, token: EnemyToken) -> Option<&CombatEnemy> {
        self.enemies.iter().find(|enemy| enemy.token == token)
    }

    /// Les ennemis du combat désignés par ces jetons, dans l'ordre du combat.
    fn enemies_at<'a>(&'a self, tokens: &'a [EnemyToken]) -> impl Iterator<Item = &'a CombatEnemy> {
        self.enemies
            .iter()
            .filter(move |enemy| tokens.iter().any(|&token| self.designates(enemy, token)))
    }

    /// L'attaque en cours sur ces cibles, sans point (MKUE p. 9, *Ranged and Siege Attack Phase* 2).
    pub(crate) fn declare_attack(&mut self, targets: &[EnemyToken]) {
        self.in_progress = Some(InProgress::Attack {
            targets: targets.to_vec(),
            points: ElementPoints::default(),
        });
    }

    /// Ajoute les points de `card`, une carte d'Attaque, à l'attaque en cours, ou d'une carte de
    /// Parade à la parade en cours, à son élément (spec 032).
    pub(crate) fn add_points(&mut self, card: CardId, effect: Effect) {
        if self.add(effect) {
            self.played.push(card);
        }
    }

    /// Ajoute les points d'une capacité d'unité, comme ceux d'une carte (spec 042).
    pub(crate) fn add_unit_points(&mut self, unit: UnitCard, effect: Effect) {
        if self.add(effect) {
            self.played_units.push(unit);
        }
    }

    /// Ajoute des points obtenus par conversion (Agilité, Diplomatie ; spec 052) : ni carte ni
    /// unité.
    pub(crate) fn add_converted(&mut self, effect: Effect) {
        self.add(effect);
    }

    /// Vrai si une Attaque physique ajoutée maintenant compte double : Épée de justice renforcée,
    /// en phase d'attaque (spec 055 ; Compendium, *Sword of Justice* S1, S7).
    pub(crate) fn doubles_physical(&self) -> bool {
        self.double_physical && self.phase == CombatPhase::Attack
    }

    /// Ajoute les points de l'effet à l'attaque ou à la parade en cours ; faux s'il n'y compte pas.
    fn add(&mut self, effect: Effect) -> bool {
        // Chaque contribution physique de la phase d'attaque, bonus compris.
        let doubled = self.doubles_physical();
        match (&mut self.in_progress, effect) {
            (
                Some(InProgress::Attack { points, .. }),
                Effect::Attack {
                    amount,
                    element: Element::Physical,
                    ..
                },
            ) if doubled => {
                points.add(Element::Physical, 2 * u32::from(amount));
                true
            }
            (
                Some(InProgress::Attack { points, .. }),
                Effect::Attack {
                    amount, element, ..
                },
            )
            | (Some(InProgress::Block { points, .. }), Effect::Block { amount, element }) => {
                points.add(element, u32::from(amount));
                true
            }
            (Some(InProgress::Block { .. }), Effect::SwiftBlock(_)) => {
                let amount = self.counted_points(effect).unwrap_or(0);
                if let Some(InProgress::Block { points, .. }) = &mut self.in_progress {
                    points.add(Element::Physical, amount);
                }
                true
            }
            // Endurance givrée renforcée : le compte se fait sur le jeton qui attaque, la créature
            // invoquée pour un Invocateur (Compendium, *Cold Toughness* O4).
            (Some(InProgress::Block { .. }), Effect::IceBlockPerAbility(_)) => {
                let amount = self.counted_points(effect).unwrap_or(0);
                if let Some(InProgress::Block { points, .. }) = &mut self.in_progress {
                    points.add(Element::Ice, amount);
                }
                true
            }
            _ => false,
        }
    }

    /// Ferme l'attaque ou la parade en cours et l'inscrit dans l'historique.
    fn close(&mut self, success: bool) {
        let cards = std::mem::take(&mut self.played);
        let units = std::mem::take(&mut self.played_units);
        let points = self.in_progress_total().unwrap_or(0);
        let record = match self.in_progress.take() {
            Some(InProgress::Attack { targets, .. }) => CombatRecord::Attack {
                phase: self.phase,
                needed: self
                    .enemies_at(&targets)
                    .map(|enemy| u32::from(enemy.effective_armor()))
                    .sum(),
                targets,
                cards,
                units,
                points,
                success,
            },
            Some(InProgress::Block { target, .. }) => CombatRecord::Block {
                target,
                needed: self
                    .enemies_at(std::slice::from_ref(&target))
                    .find_map(CombatEnemy::block_needed)
                    .unwrap_or(0),
                cards,
                units,
                points,
                success,
            },
            None => return,
        };
        self.history.push(record);
    }

    /// La parade en cours contre cet ennemi, sans point (MKUE p. 9, *Block Phase* 3).
    pub(crate) fn declare_block(&mut self, target: EnemyToken) {
        self.in_progress = Some(InProgress::Block {
            target,
            points: ElementPoints::default(),
        });
    }

    /// Cet ennemi est paré ; plus de parade en cours (MKUE p. 9, *Block Phase* 7).
    pub(crate) fn block(&mut self, token: EnemyToken) {
        let blocked = self.marked(&[token]);
        for (enemy, blocked) in self.enemies.iter_mut().zip(blocked) {
            enemy.blocked |= blocked;
        }
        self.close(true);
    }

    /// Parade ratée : l'ennemi visé ne peut plus être paré dans ce combat (Q10).
    pub(crate) fn fail_block(&mut self) {
        if let Some(InProgress::Block { target, .. }) = self.in_progress {
            let failed = self.marked(&[target]);
            for (enemy, failed) in self.enemies.iter_mut().zip(failed) {
                enemy.block_failed |= failed;
            }
        }
        self.close(false);
    }

    /// Ces ennemis sont vaincus ; plus d'attaque en cours (MKUE p. 9, 5.a).
    fn defeat(&mut self, tokens: &[EnemyToken]) {
        let defeated = self.marked(tokens);
        let phase = self.phase;
        for (enemy, defeated) in self.enemies.iter_mut().zip(defeated) {
            if defeated && !enemy.defeated {
                enemy.defeated = true;
                enemy.defeated_in = Some(phase);
                self.defeat_order.push(enemy.token);
            }
        }
        self.close(true);
    }

    /// Attaque ratée : ses cibles ne peuvent plus être visées avant la phase suivante (Q10).
    pub(crate) fn fail_attack(&mut self) {
        if let Some(InProgress::Attack { targets, .. }) = &self.in_progress {
            let attacked = self.marked(targets);
            for (enemy, attacked) in self.enemies.iter_mut().zip(attacked) {
                enemy.attacked_this_phase |= attacked;
            }
        }
        self.close(false);
    }

    /// L'Invocateur de cette case a cette créature (MKUE p. 9, *Block Phase* 2).
    pub(crate) fn summon(&mut self, summoner: EnemyToken, token: EnemyToken) {
        let summoning = self.marked(&[summoner]);
        for (enemy, summoning) in self.enemies.iter_mut().zip(summoning) {
            if summoning {
                enemy.summoned = Some(token);
            }
        }
    }

    /// Pour chaque ennemi du combat, dans l'ordre, s'il est désigné par l'un de ces jetons.
    fn marked(&self, tokens: &[EnemyToken]) -> Vec<bool> {
        self.enemies
            .iter()
            .map(|enemy| tokens.iter().any(|&token| self.designates(enemy, token)))
            .collect()
    }

    /// Pose sur l'ennemi `target` les modificateurs d'ennemi de `kinds`, venus de `source` de
    /// couleur `color`, sauf résistance (spec 053). Contre un Invocateur qui a sa créature, c'est
    /// elle que l'effet vise, et elle quitte le combat avant l'armure : rien n'est posé
    /// (Compendium, *Ice Shield* S3). En phase d'attaque, « n'attaque pas » ne change plus rien.
    pub(crate) fn pose_on_enemy(
        &mut self,
        target: EnemyToken,
        kinds: &[ModifierKind],
        source: ModifierSource,
        color: Option<BasicColor>,
    ) {
        let phase = self.phase;
        let Some(enemy) = self.enemies.iter_mut().find(|enemy| enemy.token == target) else {
            return;
        };
        if enemy.summoned.is_some() || !enemy.is_affected_by(color) {
            return;
        }
        enemy.modifiers.extend(
            kinds
                .iter()
                .filter_map(|kind| kind.on_enemy())
                .filter(|&kind| {
                    !(phase == CombatPhase::Attack && kind == EnemyModifierKind::DoesNotAttack)
                })
                .map(|kind| EnemyModifier { kind, source }),
        );
    }

    /// Pose les modificateurs d'ennemi d'un sort de couleur `color` (spec 054) : sur l'ennemi visé
    /// `target`, ou sur chaque ennemi en jeu ; selon qu'il est fortifié au moment où le sort est
    /// joué pour Tremblement de terre (Compendium, *Earthquake* S1, S5).
    pub(crate) fn pose_spell(
        &mut self,
        target: Option<EnemyToken>,
        kinds: &[ModifierKind],
        source: ModifierSource,
        color: BasicColor,
    ) {
        let live: Vec<EnemyToken> = self
            .enemies
            .iter()
            .filter(|enemy| !enemy.defeated)
            .map(|enemy| enemy.token)
            .collect();
        for &kind in kinds {
            let (tokens, plain, fortified) = match kind {
                ModifierKind::OnTargetEnemy(kind) => (target.into_iter().collect(), kind, kind),
                ModifierKind::OnTargetEnemyByFortification { plain, fortified } => {
                    (target.into_iter().collect(), plain, fortified)
                }
                ModifierKind::OnAllEnemies(kind) => (live.clone(), kind, kind),
                ModifierKind::OnAllEnemiesByFortification { plain, fortified } => {
                    (live.clone(), plain, fortified)
                }
                _ => continue,
            };
            let chosen: Vec<(EnemyToken, EnemyModifierKind)> = tokens
                .into_iter()
                .map(|token| {
                    let is_fortified = self
                        .enemy(token)
                        .is_some_and(|enemy| enemy.fortification() > 0);
                    (token, if is_fortified { fortified } else { plain })
                })
                .collect();
            for (token, kind) in chosen {
                self.pose_on_enemy(token, &[ModifierKind::OnEnemy(kind)], source, Some(color));
            }
        }
    }

    /// Note la réputation rapportée par la dernière attaque réussie (spec 040).
    pub(crate) fn record_reputation(&mut self, change: i8) {
        let targets = self.history.iter().rev().find_map(|record| match record {
            CombatRecord::Attack {
                targets,
                success: true,
                ..
            } => Some(targets.clone()),
            _ => None,
        });
        if let Some(targets) = targets {
            self.history
                .push(CombatRecord::Reputation { targets, change });
        }
    }

    /// Retire la créature invoquée par l'Invocateur de cette case ; `None` s'il n'en a pas.
    pub(crate) fn dismiss(&mut self, summoner: EnemyToken) -> Option<EnemyToken> {
        self.enemies
            .iter_mut()
            .find(|enemy| enemy.token == summoner)
            .and_then(|enemy| enemy.summoned.take())
    }
}

/// Les ennemis vaincus quittent le plateau de jeu pour la défausse de leur couleur, et rapportent
/// leur gloire (MKUE p. 9, *Ranged and Siege Attack Phase* 5.a et 5.b).
pub(crate) fn apply_defeat(
    state: &mut GameState,
    player: PlayerId,
    enemies: &[EnemyToken],
    fame: u32,
) {
    let mut defeated = Vec::new();
    if let Some(p) = state.player_mut(player) {
        if let Some(combat) = p.combat.as_mut() {
            let marked = combat.marked(enemies);
            defeated = combat
                .enemies
                .iter()
                .zip(marked)
                .filter(|&(_, marked)| marked)
                .map(|(enemy, _)| (enemy.hex, enemy.token))
                .collect();
            combat.defeat(enemies);
        }
        p.fame += fame;
    }
    // Hors combat (mise en place des tests), le jeton est cherché sur le plateau de jeu.
    for &token in enemies {
        if defeated.iter().all(|&(_, other)| other != token)
            && let Some(hex) = state.enemy_token_hex(token)
        {
            defeated.push((hex, token));
        }
    }
    for (hex, token) in defeated {
        state.discard_enemy_token(hex, token);
    }
}

/// L'assaut du tour (spec 043) : le site fortifié assailli et la case d'où le héros l'a assailli,
/// où il revient s'il bat en retraite (MKUE p. 10, *Combat Outcomes* 3.d).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Assault {
    pub site: Hex,
    pub from: Hex,
}

/// Où en est le joueur par rapport au combat du tour.
pub(crate) enum Moment<'a> {
    /// Pas d'ennemi à combattre, ou combat fini : le tour se déroule comme avant.
    Free,
    /// Ennemis provoqués ou défiés, combat pas encore engagé : seulement défier ou engager
    /// (Compendium, *Combat* Q1).
    Engaging,
    Fighting(&'a Combat),
}

pub(crate) fn moment(p: &PlayerState) -> Moment<'_> {
    match &p.combat {
        Some(combat) if !combat.ended => Moment::Fighting(combat),
        Some(_) => Moment::Free,
        None if p.enemies_to_fight.is_empty() => Moment::Free,
        None => Moment::Engaging,
    }
}

/// Ce dont dépend la jouabilité d'un effet (issue privée 111 ; spec 052) : le combat engagé, la phase
/// de mouvement finie ou non, l'interaction, et les effets en cours.
#[derive(Clone, Copy)]
pub(crate) struct PlayMoment<'a> {
    pub(crate) combat: Option<&'a Combat>,
    pub(crate) movement_ended: bool,
    pub(crate) interacting: bool,
    pub(crate) modifiers: &'a [Modifier],
    /// Le joueur a des unités qui prennent part au combat (Au cœur du combat, spec 053).
    pub(crate) units_fight: bool,
    /// Ailes de la nuit peut être lancé : le Mouvement sert en combat (spec 057, Q4).
    pub(crate) fly_in_combat: bool,
    /// Spec 057 : le tour en cours est un tour supplémentaire (Distorsion temporelle).
    pub(crate) extra_turn: bool,
    /// Spec 057 : l'offre d'unités a une unité (Appel à la gloire) ; et ses unités servent
    /// maintenant, hors d'un combat sans unités (Appel aux armes).
    pub(crate) unit_offer: bool,
    pub(crate) offer_units_fight: bool,
}

impl PlayMoment<'_> {
    /// La phase de mouvement : ni combat, ni action commencée, ni interaction.
    fn moving(&self) -> bool {
        self.combat.is_none() && !movement_over(self.movement_ended, self.interacting)
    }

    /// Un `Convert` en cours depuis cette réserve.
    fn converts_from(&self, pool: PointPool) -> bool {
        self.modifiers.iter().any(
            |modifier| matches!(modifier.kind, ModifierKind::Convert { from, .. } if from == pool),
        )
    }

    /// L'effet tel qu'il se joue : une Attaque est de siège avec `AttacksGainSiege` (Mages
    /// d'Altem, spec 053 ; Compendium, *Altem Mages* S7).
    fn as_siege(&self, effect: Effect) -> Effect {
        let siege = self
            .modifiers
            .iter()
            .any(|modifier| modifier.kind == ModifierKind::AttacksGainSiege);
        match effect.resolved() {
            Effect::Attack {
                amount, element, ..
            } if siege => Effect::Attack {
                amount,
                element,
                kind: AttackKind::Siege,
            },
            // Issue privée 272 (Compendium, *Sword of Justice* S6, S2) : l'Attaque de base de l'Épée de
            // justice, devenue de siège, se joue aussi en phase à distance.
            Effect::SwordDiscard if siege => Effect::Attack {
                amount: 3,
                element: Element::Physical,
                kind: AttackKind::Siege,
            },
            _ => effect,
        }
    }

    /// Vrai si ce modificateur sert maintenant (spec 052, tableau « Le modificateur sert »).
    fn serves(&self, kind: ModifierKind) -> bool {
        match kind {
            ModifierKind::TerrainCost { .. } | ModifierKind::Passable(_) => self.moving(),
            // Compendium, *Agility* S2 ; *Ambush* S4 : jusqu'à la fin du combat.
            ModifierKind::Convert {
                from: PointPool::Move,
                ..
            }
            | ModifierKind::NextCardBonus { .. } => self.moving() || self.combat.is_some(),
            // Compendium, *Diplomacy* S1 : jusqu'à la fin de la phase de parade.
            ModifierKind::Convert {
                from: PointPool::Influence,
                ..
            } => self.moving() || self.combat.is_some_and(before_attack_phase),
            // Spec 053, Q6 : « au début du combat », tant qu'aucune unité n'y a été activée, avec au
            // moins une unité.
            ModifierKind::UnitBonus { .. } => {
                self.units_fight && self.combat.is_some_and(|combat| !combat.unit_activated)
            }
            // Spec 053 (Mages d'Altem) : le feu glacé dans tout le combat ; le siège en phase
            // d'attaque à distance seulement (Compendium, *Combat* S6).
            ModifierKind::AttacksBecome(_) => self.combat.is_some(),
            ModifierKind::AttacksGainSiege => self
                .combat
                .is_some_and(|combat| combat.phase == CombatPhase::RangedAndSiege),
            // Spec 053 (Gardiens d'Altem) : avant l'assignation des dégâts finie.
            ModifierKind::UnitsAllResistances => self
                .combat
                .is_some_and(|combat| combat.phase != CombatPhase::Attack),
            // Un coût ou un à-côté : jamais une raison de jouer la carte.
            ModifierKind::EndOfTurnReputation(_)
            | ModifierKind::ReturnToDeck { .. }
            | ModifierKind::EndOfTurnPerRecruit { .. }
            | ModifierKind::NoDamageToUnits => false,
            // Spec 053 : la cible d'une capacité d'unité se choisit dans ses actions légales
            // (`unit_enemy_targets`) ; une carte se joue pour ses points (Bouclier de glace).
            ModifierKind::OnEnemy(_) | ModifierKind::OnUnfortifiedEnemy(_) => false,
            // Spec 056 : écrit avec chaque carte.
            // Spec 056 : Maîtrise du cristal se joue à tout moment, comme les cartes de mana.
            ModifierKind::ReturnSpentCrystals => true,
            ModifierKind::LearningDiscount { .. } => false,
            // Spec 054 : un sort qui pose un modificateur d'ennemi, dans les phases où il sert ;
            // ses cibles se choisissent dans ses actions légales.
            ModifierKind::OnTargetEnemy(kind) | ModifierKind::OnAllEnemies(kind) => self
                .combat
                .is_some_and(|combat| enemy_kind_serves(kind, combat.phase)),
            ModifierKind::OnTargetEnemyByFortification { plain, fortified }
            | ModifierKind::OnAllEnemiesByFortification { plain, fortified } => {
                self.combat.is_some_and(|combat| {
                    enemy_kind_serves(plain, combat.phase)
                        || enemy_kind_serves(fortified, combat.phase)
                })
            }
            // Spec 054, Q5 : Démolition en combat seulement, dans toute phase.
            ModifierKind::IgnoreSiteFortifications => self.combat.is_some(),
            ModifierKind::DeferredAttack { .. }
            | ModifierKind::DestroyBlockedEnemy
            | ModifierKind::CastFromOffer(_) => false,
            // Spec 057 : Distorsion spatiale, en phase de déplacement.
            ModifierKind::SpaceBending => self.moving(),
            // Spec 057 (Compendium, *Time Bending* S13) : pas pendant un tour supplémentaire.
            ModifierKind::TimeBending => !self.extra_turn,
            // Spec 057 : Appel aux armes et Appel à la gloire (L6).
            ModifierKind::UnitFromOffer => self.offer_units_fight,
            ModifierKind::RecruitFromOffer => self.unit_offer,
            ModifierKind::DrawBonus(_) => true,
            ModifierKind::ManaClaimPerTurn(_) => false,
            // Spec 055 : un à-côté des amulettes, qui se jouent pour leurs jetons.
            ModifierKind::DayRules | ModifierKind::NightMana => false,
            // Spec 055 : du mana, à tout moment.
            ModifierKind::EndlessMana(_) => true,
            // Spec 055 : la Bannière de gloire renforcée, en combat seulement.
            // Spec 055 (Compendium, *Banner of Protection* Q1) : à tout moment du tour.
            ModifierKind::ThrowAwayReceivedWounds => true,
            // Spec 055 : la gloire de l'Épée ne la rend pas jouable ; le doublement, en combat.
            ModifierKind::FamePerEnemyDefeated => false,
            ModifierKind::DoublePhysicalAttacks => self.combat.is_some(),
            // Spec 055 : suites du Soin du Graal, qui se joue pour lui.
            ModifierKind::GrailFame { .. } | ModifierKind::DrawPerHandWoundHealed => false,
            ModifierKind::UnitArmorBonus(_) | ModifierKind::FamePerUnitAttackOrBlock => {
                self.units_fight && self.combat.is_some()
            }
        }
    }
}

/// Vrai si un modificateur d'ennemi de cette sorte sert dans cette phase (spec 054) : « n'attaque
/// pas » avant les dégâts (Tourbillon, Frisson), l'armure réduite dans toute phase (*Tremor* S2),
/// fortifications et résistances perdues quand une attaque se déclare (Exposition).
fn enemy_kind_serves(kind: EnemyModifierKind, phase: CombatPhase) -> bool {
    match kind {
        EnemyModifierKind::DoesNotAttack => {
            matches!(phase, CombatPhase::RangedAndSiege | CombatPhase::Block)
        }
        EnemyModifierKind::ArmorReduction { .. } => true,
        EnemyModifierKind::LosesFortifications
        | EnemyModifierKind::LosesResistances
        | EnemyModifierKind::LosesResistance(_) => {
            matches!(phase, CombatPhase::RangedAndSiege | CombatPhase::Attack)
        }
    }
}

/// Les ennemis que vise un sort dont l'option pose ces modificateurs d'ennemi ou détruit
/// (`destroys`), d'une couleur `color`, dans l'ordre du combat (spec 054) : en jeu ; comme les
/// capacités d'unités pour « n'attaque pas » (spec 053, Q7) ; en phase d'attaque pour détruire ;
/// sinon dans une phase où le modificateur sert. Jamais un ennemi que la couleur n'atteint pas
/// (MKUE p. 8, 5).
pub(crate) fn spell_enemy_targets(
    combat: &Combat,
    kinds: &[EnemyModifierKind],
    destroys: bool,
    color: BasicColor,
) -> Vec<EnemyToken> {
    let candidates: Vec<EnemyToken> = if destroys {
        if combat.phase != CombatPhase::Attack {
            return Vec::new();
        }
        combat
            .enemies
            .iter()
            .filter(|enemy| !enemy.defeated)
            .map(|enemy| enemy.token)
            .collect()
    } else if kinds.contains(&EnemyModifierKind::DoesNotAttack) {
        let as_unit: Vec<ModifierKind> = kinds
            .iter()
            .map(|&kind| ModifierKind::OnEnemy(kind))
            .collect();
        unit_enemy_targets(combat, &as_unit)
    } else if kinds
        .iter()
        .any(|&kind| enemy_kind_serves(kind, combat.phase))
    {
        combat
            .enemies
            .iter()
            .filter(|enemy| !enemy.defeated)
            .map(|enemy| enemy.token)
            .collect()
    } else {
        Vec::new()
    };
    candidates
        .into_iter()
        .filter(|&token| {
            combat
                .enemy(token)
                .is_some_and(|enemy| enemy.is_affected_by(Some(color)))
        })
        .collect()
}

/// Avant la phase d'attaque : l'attaque à distance ou la parade.
fn before_attack_phase(combat: &Combat) -> bool {
    matches!(
        combat.phase,
        CombatPhase::RangedAndSiege | CombatPhase::Block
    )
}

/// Vrai si une option se joue maintenant, mana mis à part (MKUE p. 6, 5 ; Compendium,
/// *Miscellaneous* S1 ; spec 052) : ses points servent, ou l'un de ses modificateurs sert. La
/// seule règle, pour les actions légales comme pour l'écran (issue privée 111).
pub(crate) fn is_playable_now(
    effect: Effect,
    option_modifiers: &[ModifierKind],
    moment: PlayMoment<'_>,
) -> bool {
    points_serve(effect, moment) || option_modifiers.iter().any(|&kind| moment.serves(kind))
}

/// Vrai si les points de l'effet servent maintenant : dans le combat engagé selon sa phase, ou
/// hors combat (MKUE p. 6, 5.c), le Mouvement pendant la phase de mouvement, l'Influence aussi en
/// interaction (MKUE p. 8, 2 ; issue privée 164). En combat, le Mouvement et l'Influence servent avec
/// une conversion en cours (spec 052).
fn points_serve(effect: Effect, moment: PlayMoment<'_>) -> bool {
    let influence = matches!(
        effect,
        Effect::Influence(_)
            | Effect::InfluenceReputation { .. }
            | Effect::InfluenceInInteraction { .. }
            | Effect::InfluencePerWound { .. }
            | Effect::InfluenceAndFame { .. }
    );
    match moment.combat {
        Some(combat) => match effect {
            // Spec 053 : Marche revigorante, son Mouvement seul (MKUE p. 8, 6).
            Effect::Move(_) | Effect::MoveAndHeal { .. } => {
                moment.converts_from(PointPool::Move) || moment.fly_in_combat
            }
            _ if influence => {
                moment.converts_from(PointPool::Influence) && before_attack_phase(combat)
            }
            _ => is_playable_in_combat(moment.as_siege(effect), combat),
        },
        None => {
            effect.is_playable_out_of_combat()
                // Spec 057 : un déplacement par effet, en phase de déplacement seulement.
                && !(matches!(
                    effect,
                    Effect::Move(_)
                        | Effect::UndergroundMove { .. }
                        | Effect::FlyMove { .. }
                        | Effect::Flight { .. }
                )
                    && !moment.moving())
                && !(influence && !moment.moving() && !moment.interacting)
                // Spec 058 (Q3) : l'Influence d'interaction, pendant l'interaction seulement.
                && !(matches!(effect, Effect::InteractionInfluence(_)) && !moment.interacting)
        }
    }
}

/// Vrai si la phase de mouvement du tour est finie : l'action a commencé (combat, même fini,
/// site d'aventure, offrande, provocation : `movement_ended`) ou une interaction (MKUE p. 8, 2 ;
/// spec 040). Le Mouvement ne sert plus (MKUE p. 6, 5, dernier tiret, et 6.d ; issue privée 164), sauf
/// par un modificateur qui sert encore (Agilité, Embuscade : spec 052, `PlayMoment::serves`).
pub(crate) fn movement_over(movement_ended: bool, interacting: bool) -> bool {
    movement_ended || interacting
}

/// Vrai si l'effet se joue à ce moment du combat. Mouvement et Influence ne se jouent plus
/// (MKUE p. 6, 6.e) ; le mana, toujours (MKUE p. 8, 6).
pub(crate) fn is_playable_in_combat(effect: Effect, combat: &Combat) -> bool {
    match effect.resolved() {
        Effect::Move(_)
        | Effect::Influence(_)
        | Effect::InfluenceReputation { .. }
        | Effect::InfluenceInInteraction { .. } => false,
        // Une attaque seulement dans une attaque en cours (MKUE p. 9, *Ranged and Siege Attack
        // Phase* 8) : à distance, le siège toujours, la distance si aucune cible n'est fortifiée
        // (3 et 3.a) ; ensuite toute attaque (MKUE p. 10, *Attack Phase* 1.a).
        Effect::Attack { kind, .. } => match (&combat.in_progress, combat.phase) {
            (Some(InProgress::Attack { targets, .. }), CombatPhase::RangedAndSiege) => match kind {
                AttackKind::Siege => true,
                // Spec 043 : fortifié par sa capacité ou par son site.
                AttackKind::Ranged => !combat
                    .enemies_at(targets)
                    .any(|enemy| enemy.fortification() > 0),
                AttackKind::Melee => false,
            },
            (Some(InProgress::Attack { .. }), CombatPhase::Attack) => true,
            _ => false,
        },
        // Parade : seulement dans une parade en cours (MKUE p. 9, *Block Phase* 10).
        Effect::Block { .. } | Effect::IceBlockPerAbility(_) => {
            matches!(combat.in_progress, Some(InProgress::Block { .. }))
        }
        Effect::GainToken(_)
        | Effect::GainCrystal(_)
        | Effect::PayForCrystal(_)
        | Effect::ExtraDie
        | Effect::ExtraDieBlackAnyColor
        | Effect::SetDice { .. }
        | Effect::Boost(_) => true,
        // Aucun soin en combat (MKUE p. 8, *Combat with Enemies* 6 ; spec 034).
        Effect::Heal(_) | Effect::DrawCards(_) => false,
        // Spec 042 : une parade, comme les autres.
        Effect::SwiftBlock(_) => matches!(combat.in_progress, Some(InProgress::Block { .. })),
        Effect::ReadyUnit(_) | Effect::TokenAndCrystal(_) | Effect::GainTokens(..) => false,
        // Spec 053 : le Mouvement de Marche revigorante se joue par une conversion
        // (`points_serve`) ; aucun soin, aucune Influence ; une option sans points, par ses
        // modificateurs. Coup de sang compte comme une Attaque (`Effect::resolved`).
        Effect::Nothing
        | Effect::MoveAndHeal { .. }
        | Effect::HealAndReady { .. }
        | Effect::InfluencePerWound { .. }
        | Effect::AttackWithWound { .. } => false,
        // Spec 056 : les cartes de mana, Entraînement et Don pour la magie, à tout moment.
        Effect::CrystalsFromDestroyed(_)
        | Effect::CrystalsOtherColors
        | Effect::WoundCrystalAndToken { .. }
        | Effect::WoundAndTokens(_)
        | Effect::GainOwnedCrystal(_)
        | Effect::CrystalFromDie
        | Effect::RerollSource
        | Effect::GainSpell(_)
        | Effect::TrainFromOffer(_) => true,
        // Spec 056 : Effet maximum se joue si sa cible sert maintenant (ses actions légales).
        Effect::Repeat(_) => true,
        // Spec 054 : détruire, en phase d'attaque seulement (Tornade, Désintégration) ; Don pour
        // la magie à tout moment. Déluge de flammes arrive résolu (`Effect::resolved`).
        Effect::DestroyEnemy => combat.phase == CombatPhase::Attack,
        Effect::CastFromOffer => true,
        Effect::AttackPerEnemy { .. } | Effect::BlockPerEnemy { .. } => false,
        // Spec 057 : Brûlure de mana à tout moment (icône spéciale) ; le soin et les unités rendues
        // disponibles, jamais en combat (Q5).
        Effect::WoundsPerCrystal(_) => true,
        Effect::OthersOnly
        | Effect::HealByTerrain { .. }
        | Effect::HealAndReadyLevels { .. }
        | Effect::ReadyAnyUnit { .. } => false,
        // Spec 057 : un déplacement par effet, jamais en combat (Compendium, *Movement* S2).
        Effect::UndergroundMove { .. } | Effect::FlyMove { .. } => false,
        // Spec 057 : Méditation, à tout moment.
        Effect::DiscardToDeck { .. } => true,
        // Spec 057 : Puits de mana, à tout moment.
        Effect::ClaimDie { .. } => true,
        // Spec 057 : comme Tourbillon, en phase à distance ou de parade (Q6 de la spec 054).
        Effect::DoesNotAttackPerMove => matches!(
            combat.phase,
            CombatPhase::RangedAndSiege | CombatPhase::Block
        ),
        // Spec 055 : une bannière s'assigne par `AssignBanner` ; la Bourse arrive résolue.
        Effect::BannerAssigned | Effect::InfluenceAndFame { .. } => false,
        // Spec 055 : des jetons, à tout moment.
        Effect::AmuletOfSun { .. }
        | Effect::AmuletOfDarkness { .. }
        | Effect::TokenCrystalAndFame(_) => true,
        // Spec 051 : cristal et jeton, Motivation, à tout moment ; le soin d'une unité, jamais en
        // combat.
        Effect::CrystalAndToken { .. } | Effect::DrawAndToken { .. } => true,
        Effect::HealUnit => false,
        // Spec 058 : l'Influence d'interaction et Même pas mal, jamais en combat ; Marche en avant
        // arrive résolue.
        Effect::CrystalColorInfluence
        | Effect::InteractionInfluence(_)
        | Effect::MovePerUnit { .. }
        | Effect::DiscardWoundToDraw
        | Effect::ThrowWounds(_) => false,
        // Spec 058 : le Vol, hors combat (en phase de mouvement) ; Botte secrète à toute phase ;
        // les coopératives, à tout moment.
        Effect::Flight { .. } => false,
        Effect::ResistanceBreak
        | Effect::SourceOpening
        | Effect::CalmWeather
        | Effect::ManaOverload => true,
        // Spec 055 : comme Tourbillon, en phase à distance ou de parade ; la Bannière de courage
        // renforcée, hors combat.
        Effect::CancelAttacks(_) => matches!(
            combat.phase,
            CombatPhase::RangedAndSiege | CombatPhase::Block
        ),
        Effect::ReadyAllUnits => false,
        // Spec 055 : une Attaque de siège 5, en phase à distance ou d'attaque.
        Effect::HornOfWrath { .. } => is_playable_in_combat(
            Effect::Attack {
                amount: 5,
                element: Element::Physical,
                kind: AttackKind::Siege,
            },
            combat,
        ),
        // Spec 055 : des cristaux et des jetons, à tout moment.
        Effect::GemPouchDice | Effect::TokensEachColorAndTime => true,
        // Spec 055 : comme Entraînement (spec 056), à tout moment.
        Effect::SpellAndCrystalFromOffer => true,
        // Spec 055 (Compendium, *Sword of Justice* S2) : une Attaque physique, en phase d'attaque.
        Effect::SwordDiscard => {
            combat.phase == CombatPhase::Attack
                && is_playable_in_combat(
                    Effect::Attack {
                        amount: 3,
                        element: Element::Physical,
                        kind: AttackKind::Melee,
                    },
                    combat,
                )
        }
    }
}

/// La conversion du Mouvement en Attaque possible maintenant (Agilité, spec 052 ; Compendium,
/// *Agility* S1, S2) : un `Convert` depuis le Mouvement, une attaque en cours qui accepte son
/// Attaque 1 (en phase d'attaque à distance, une attaque à distance contre des cibles non
/// fortifiées), assez de points ; la moins chère. Rend le genre de l'attaque et le taux.
pub(crate) fn move_conversion(p: &PlayerState, combat: &Combat) -> Option<(AttackKind, u8)> {
    if !matches!(combat.in_progress, Some(InProgress::Attack { .. })) {
        return None;
    }
    p.modifiers
        .iter()
        .filter_map(|modifier| match modifier.kind {
            ModifierKind::Convert {
                from: PointPool::Move,
                to: ConvertedPoints::Attack { element, kind },
                rate,
            } => Some((element, kind, rate)),
            _ => None,
        })
        .filter(|&(element, kind, rate)| {
            // La phase décide du genre : la mêlée seulement en phase d'attaque.
            let attack = Effect::Attack {
                amount: 1,
                element,
                kind,
            };
            p.move_points >= u32::from(rate) && is_playable_in_combat(attack, combat)
        })
        .min_by_key(|&(_, _, rate)| rate)
        .map(|(_, kind, rate)| (kind, rate))
}

/// La conversion de l'Influence en Parade possible maintenant (Diplomatie, spec 052 ; Compendium,
/// *Diplomacy* S1) : un `Convert` depuis l'Influence, une parade en cours, assez de points. Rend
/// l'élément de la Parade.
pub(crate) fn influence_conversion(p: &PlayerState, combat: &Combat) -> Option<Element> {
    if !matches!(combat.in_progress, Some(InProgress::Block { .. })) {
        return None;
    }
    p.modifiers.iter().find_map(|modifier| match modifier.kind {
        ModifierKind::Convert {
            from: PointPool::Influence,
            to: ConvertedPoints::Block { element },
            rate,
        } if p.influence_points >= i32::from(rate) => Some(element),
        _ => None,
    })
}

/// Les conversions légales en combat (spec 052) : Mouvement en Attaque, Influence en Parade.
pub(crate) fn conversion_actions(p: &PlayerState, combat: &Combat) -> Vec<Action> {
    let mut actions = Vec::new();
    if move_conversion(p, combat).is_some() {
        actions.push(Action::SpendMoveAsAttack);
    }
    if influence_conversion(p, combat).is_some() {
        actions.push(Action::SpendInfluenceAsBlock);
    }
    actions
}

/// De côté, en combat : Attaque 1 dans une attaque en cours de la phase d'attaque seulement
/// (MKUE p. 10, *Attack Phase* 1.b ; p. 9, *Ranged and Siege Attack Phase* 3.b), Parade 1 dans
/// une parade en cours (MKUE p. 9, *Block Phase* 4.b).
pub(crate) fn sideways_in_combat(combat: &Combat) -> Vec<SidewaysEffect> {
    match (&combat.in_progress, combat.phase) {
        (Some(InProgress::Attack { .. }), CombatPhase::Attack) => vec![SidewaysEffect::Attack],
        (Some(InProgress::Block { .. }), CombatPhase::Block) => vec![SidewaysEffect::Block],
        _ => Vec::new(),
    }
}

/// Les ennemis que le joueur peut défier, dans l'ordre de `Hex::neighbors` : un jeton voisin de
/// la figurine, pas encore à combattre, si le combat du tour n'est pas engagé (MKUE p. 6, 6.c ;
/// p. 8, 1.d, 2 et 2.c).
pub(crate) fn challengeable(state: &GameState, player: PlayerId) -> Vec<Hex> {
    let Some((p, figure)) = state
        .player(player)
        .and_then(|p| p.figure.map(|figure| (p, figure)))
    else {
        return Vec::new();
    };
    if p.combat.is_some() {
        return Vec::new();
    }
    // Spec 034 (Compendium, *Resting* S4) : une main de Blessures sans carte jouée ne peut que
    // se reposer ; défier rendrait la fin du tour impossible après le combat.
    if p.play_area.is_empty()
        && !p.hand.is_empty()
        && p.hand.iter().all(|card| card.card().is_wound())
    {
        return Vec::new();
    }
    if crate::engine::must_announce_end_of_round(p) {
        return Vec::new();
    }
    // Spec 043 : un défenseur ne se défie pas.
    let rampaging = crate::engine::rampaging_enemies(state);
    figure
        .neighbors()
        .into_iter()
        .filter(|hex| rampaging.contains(hex) && !p.enemies_to_fight.contains(hex))
        .collect()
}

/// Défier, puis engager le combat s'il y a des ennemis à combattre.
pub(crate) fn engaging_actions(state: &GameState, player: PlayerId) -> Vec<Action> {
    let mut actions: Vec<Action> = challengeable(state, player)
        .into_iter()
        .map(|enemy| Action::Challenge { enemy })
        .collect();
    if state
        .player(player)
        .is_some_and(|p| p.combat.is_none() && !p.enemies_to_fight.is_empty())
    {
        actions.push(Action::StartCombat);
    }
    actions
}

/// Les ennemis qu'une attaque peut viser dans la phase en cours, dans l'ordre du combat : en jeu,
/// pas déjà visés dans cette phase (Q10). Un fortifié aussi en phase d'attaque à distance : son
/// attaque n'acceptera que du siège (spec 032, Q1 et Q2). Aucune attaque pendant la parade ni
/// l'assignation des dégâts.
fn attack_targets(combat: &Combat) -> Vec<EnemyToken> {
    if !matches!(
        combat.phase,
        CombatPhase::RangedAndSiege | CombatPhase::Attack
    ) {
        return Vec::new();
    }
    combat
        .enemies
        .iter()
        .filter(|enemy| !enemy.defeated && !enemy.attacked_this_phase)
        // Spec 043 : doublement fortifié, aucune cible à distance (FAQ p3 ; Compendium,
        // *Combat* S13).
        .filter(|enemy| combat.phase != CombatPhase::RangedAndSiege || enemy.fortification() < 2)
        .map(|enemy| enemy.token)
        .collect()
}

/// Les ennemis qu'une parade peut viser, dans l'ordre du combat : en jeu, pas déjà parés ni visés
/// par une parade ratée (Q10), et qui attaquent (un Invocateur par sa créature, spec 033). Seulement
/// dans la phase de parade (MKUE p. 9, *Block Phase* 1 et 3).
fn block_targets(combat: &Combat) -> Vec<EnemyToken> {
    if combat.phase != CombatPhase::Block {
        return Vec::new();
    }
    combat
        .enemies
        .iter()
        .filter(|enemy| !enemy.defeated && !enemy.blocked && !enemy.block_failed)
        .filter(|enemy| enemy.attack_value().is_some())
        .map(|enemy| enemy.token)
        .collect()
}

/// Les ennemis qu'une capacité d'unité qui pose ces modificateurs peut viser, dans l'ordre du
/// combat (spec 053, Q7). « N'attaque pas » : en phase à distance ou de parade, un ennemi en jeu,
/// ni paré ni visé par une parade ratée, qui attaque encore ; en phase de parade, pas un
/// Invocateur (sa créature est tirée). Non fortifié pour les Illusionnistes (ni par sa capacité, ni
/// par son site). En phase d'attaque, seulement pour une armure réduite (Glaceurs), un ennemi en
/// jeu.
pub(crate) fn unit_enemy_targets(combat: &Combat, kinds: &[ModifierKind]) -> Vec<EnemyToken> {
    let stops_attack = kinds
        .iter()
        .any(|kind| kind.on_enemy() == Some(EnemyModifierKind::DoesNotAttack));
    let reduces_armor = kinds.iter().any(|kind| {
        matches!(
            kind.on_enemy(),
            Some(EnemyModifierKind::ArmorReduction { .. })
        )
    });
    let unfortified_only = kinds
        .iter()
        .any(|kind| matches!(kind, ModifierKind::OnUnfortifiedEnemy(_)));
    combat
        .enemies
        .iter()
        .filter(|enemy| !enemy.defeated)
        .filter(|enemy| !unfortified_only || enemy.fortification() == 0)
        .filter(|enemy| match combat.phase {
            CombatPhase::RangedAndSiege | CombatPhase::Block if stops_attack => {
                !enemy.blocked
                    && !enemy.block_failed
                    && !enemy.does_not_attack()
                    && (combat.phase == CombatPhase::RangedAndSiege
                        || (enemy.summoned.is_none() && attack_value(enemy.token).is_some()))
            }
            CombatPhase::Attack => reduces_armor,
            _ => false,
        })
        .map(|enemy| enemy.token)
        .collect()
}

/// Sans attaque ni parade en cours : choisir les cibles d'une attaque (tous les sous-ensembles non
/// vides, par taille croissante) ou l'ennemi d'une parade. Sinon, valider ce qui est en cours
/// (MKUE p. 9, *Ranged and Siege Attack Phase* 2 et 5 ; *Block Phase* 3 et 6).
pub(crate) fn target_actions(combat: &Combat) -> Vec<Action> {
    match combat.in_progress {
        Some(InProgress::Attack { .. }) => vec![Action::ResolveAttack],
        Some(InProgress::Block { .. }) => vec![Action::ResolveBlock],
        None => {
            let targets = attack_targets(combat);
            let mut actions: Vec<Action> = (1..=targets.len())
                .flat_map(|size| combinations(&targets, size))
                .map(|targets| Action::DeclareAttack { targets })
                .collect();
            actions.extend(
                block_targets(combat)
                    .into_iter()
                    .map(|target| Action::DeclareBlock { target }),
            );
            actions
        }
    }
}

/// Valide l'attaque en cours : vaincue si les points atteignent la somme des armures des cibles
/// (MKUE p. 9, *Ranged and Siege Attack Phase* 5 et 7), sans effet sinon (6).
pub(crate) fn resolve_attack(
    state: &GameState,
    player: PlayerId,
) -> Result<Vec<Event>, ActionError> {
    let combat = state
        .player(player)
        .and_then(|p| p.combat.as_ref())
        .ok_or(ActionError::NotLegal)?;
    let Some(InProgress::Attack { targets, .. }) = &combat.in_progress else {
        return Err(ActionError::NotLegal);
    };
    let points = combat.in_progress_total().unwrap_or(0);
    // Spec 053 : l'armure qui compte, réductions comprises.
    let (armor, fame) = combat
        .enemies_at(targets)
        .fold((0, 0), |(armor, fame), enemy| {
            (
                armor + u32::from(enemy.effective_armor()),
                fame + u32::from(enemy.token.enemy().definition().fame),
            )
        });
    if points < armor {
        return Ok(vec![Event::AttackFailed { player }]);
    }
    let mut events = vec![Event::EnemiesDefeated {
        player,
        enemies: targets.clone(),
        fame,
    }];
    events.extend(rampaging_reputation(state, player, combat, targets));
    Ok(events)
}

/// La réputation des ennemis errants vaincus ou détruits, d'après la case imprimée et non la
/// couleur du jeton (MKUE p. 10, *Combat Outcomes* 2.a ; spec 040, Q2), en un seul changement.
pub(crate) fn rampaging_reputation(
    state: &GameState,
    player: PlayerId,
    combat: &Combat,
    targets: &[EnemyToken],
) -> Option<Event> {
    let change: i8 = combat
        .enemies_at(targets)
        .map(
            |enemy| match state.map.space(enemy.hex).and_then(|space| space.feature) {
                Some(Feature::Rampaging(Rampaging::OrcMarauders)) => 1,
                Some(Feature::Rampaging(Rampaging::Draconum)) => 2,
                _ => 0,
            },
        )
        .sum();
    (change != 0).then_some(Event::ReputationChanged { player, change })
}

/// Un ennemi détruit par un sort (spec 054, Q2) : vaincu aussitôt, sa gloire, son jeton défaussé ;
/// retiré de l'attaque en cours. Une créature invoquée détruite part sans gloire, son Invocateur
/// reste (Compendium, *Exploding Shield* Q2).
pub(crate) fn apply_destroyed(state: &mut GameState, player: PlayerId, token: EnemyToken) {
    let mut discarded = None;
    let mut summoner = None;
    if let Some(p) = state.player_mut(player)
        && let Some(combat) = p.combat.as_mut()
    {
        let phase = combat.phase;
        if let Some(enemy) = combat
            .enemies
            .iter_mut()
            .find(|enemy| enemy.token == token && !enemy.defeated)
        {
            enemy.defeated = true;
            enemy.destroyed = true;
            enemy.defeated_in = Some(phase);
            combat.defeat_order.push(token);
            discarded = Some(enemy.hex);
            p.fame += u32::from(token.enemy().definition().fame);
            if let Some(InProgress::Attack { targets, .. }) = &mut combat.in_progress {
                targets.retain(|&target| target != token);
                // Sans cible, l'attaque en cours n'a plus d'objet.
                if targets.is_empty() {
                    combat.in_progress = None;
                    combat.played.clear();
                    combat.played_units.clear();
                }
            }
        } else {
            summoner = combat
                .enemies
                .iter()
                .find(|enemy| enemy.summoned == Some(token))
                .map(|enemy| enemy.token);
        }
    }
    if let Some(hex) = discarded {
        state.discard_enemy_token(hex, token);
    }
    if let Some(summoner) = summoner {
        apply_dismiss(state, player, summoner);
    }
}

/// Valide la parade en cours : l'ennemi est paré si les points atteignent son attaque (MKUE p. 9,
/// *Block Phase* 6), sans effet sinon (8).
pub(crate) fn resolve_block(
    state: &GameState,
    player: PlayerId,
) -> Result<Vec<Event>, ActionError> {
    let combat = state
        .player(player)
        .and_then(|p| p.combat.as_ref())
        .ok_or(ActionError::NotLegal)?;
    let Some(InProgress::Block { target, .. }) = &combat.in_progress else {
        return Err(ActionError::NotLegal);
    };
    let points = combat.in_progress_total().unwrap_or(0);
    let enemy = combat
        .enemies_at(std::slice::from_ref(target))
        .next()
        .ok_or(ActionError::NotLegal)?;
    let needed = enemy.block_needed().ok_or(ActionError::NotLegal)?;
    if points < needed {
        return Ok(vec![Event::BlockFailed { player }]);
    }
    let mut events = vec![Event::EnemyBlocked {
        player,
        enemy: *target,
    }];
    // Spec 054 : Bouclier explosif, l'ennemi paré détruit (la créature invoquée, sans gloire),
    // sauf résistance à la couleur du sort (Compendium, *Exploding Shield* S1, Q2).
    let destroyed = combat
        .on_block_success
        .iter()
        .filter(|(kind, _)| *kind == ModifierKind::DestroyBlockedEnemy)
        .any(|&(_, source)| {
            let color = match source {
                ModifierSource::Card(card) => card.card().definition().map(|d| d.color),
                ModifierSource::Unit(_) | ModifierSource::Skill(_) => None,
            };
            match enemy.summoned {
                Some(creature) => {
                    let resistances = creature.enemy().definition().resistances.clone();
                    match color {
                        Some(BasicColor::Red) => !resistances.contains(&Resistance::Fire),
                        Some(BasicColor::Blue) => !resistances.contains(&Resistance::Ice),
                        _ => true,
                    }
                }
                None => enemy.is_affected_by(color),
            }
        });
    if destroyed {
        let destroyed = enemy.summoned.unwrap_or(*target);
        events.push(Event::EnemyDestroyed {
            player,
            enemy: destroyed,
        });
        if enemy.summoned.is_none() {
            events.extend(rampaging_reputation(state, player, combat, &[*target]));
        }
        return Ok(events);
    }
    // Une créature invoquée parée va aussitôt dans la défausse, sans gloire (MKUE p. 9, *Block
    // Phase* 7.a).
    if enemy.summoned.is_some() {
        events.push(Event::SummonedEnemyDiscarded {
            player,
            summoner: *target,
        });
    }
    Ok(events)
}

/// Passer à la phase suivante, ou terminer le combat depuis la phase d'attaque (MKUE p. 10,
/// *Combat Outcomes* 1).
pub(crate) fn phase_actions(combat: &Combat) -> Vec<Action> {
    if combat.in_progress.is_some() {
        return Vec::new();
    }
    // Spec 042 : des dégâts restent à assigner.
    if combat.phase == CombatPhase::AssignDamage
        && combat.enemies.iter().any(CombatEnemy::has_pending_damage)
    {
        return Vec::new();
    }
    // Plus d'ennemi : les phases restantes ne feraient rien (spec 031, L4).
    if combat.enemies.iter().all(|enemy| enemy.defeated) {
        return vec![Action::EndCombat];
    }
    vec![match combat.phase {
        CombatPhase::Attack => Action::EndCombat,
        CombatPhase::RangedAndSiege | CombatPhase::Block | CombatPhase::AssignDamage => {
            Action::NextCombatPhase
        }
    }]
}

/// La phase qui suit, sans la phase d'assignation des dégâts : elle se résout seule (Q3).
fn next_phase(phase: CombatPhase) -> Option<CombatPhase> {
    match phase {
        CombatPhase::RangedAndSiege => Some(CombatPhase::Block),
        CombatPhase::Block | CombatPhase::AssignDamage => Some(CombatPhase::Attack),
        CombatPhase::Attack => None,
    }
}

/// Les dégâts de chaque ennemi en jeu, qui attaque et n'est pas paré, dans l'ordre du combat :
/// une Blessure par tranche entamée de l'armure du niveau (spec 035), chaque ennemi se comptant à
/// part (MKUE p. 9, *Assign Damage Phase* 2, 5.a et 5.b ; Compendium, *Combat* S38). Puis, selon ses capacités
/// (MKUE p. 10, *Assign Damage Phase* 6 ; spec 033) : autant de Blessures dans la défausse
/// (toxique), la main défaussée sauf les Blessures (paralysant, une seule fois), la créature
/// invoquée défaussée. Les numéros d'exemplaire suivent les Blessures déjà données (la pile est
/// illimitée, MKUE p. 3).
fn damage_events(player: PlayerId, p: &PlayerState, combat: &Combat) -> Vec<Event> {
    let mut hero = HeroDamage::new(p, combat);
    let mut events = Vec::new();
    for enemy in combat
        .enemies
        .iter()
        .filter(|enemy| enemy.has_pending_damage())
    {
        let Some(damage) = enemy.damage() else {
            continue;
        };
        events.extend(hero.events(player, enemy, damage));
        events.extend(dismissed(player, enemy));
    }
    events
}

/// La créature invoquée défaussée une fois ses dégâts faits (MKUE p. 10, 6.c ; spec 033).
fn dismissed(player: PlayerId, enemy: &CombatEnemy) -> Option<Event> {
    enemy.summoned.map(|_| Event::SummonedEnemyDiscarded {
        player,
        summoner: enemy.token,
    })
}

/// Les dégâts faits au héros pendant une phase d'assignation, suivis d'un ennemi à l'autre : les
/// numéros d'exemplaire des Blessures, la main pour l'assommé et le paralysant (specs 031, 033,
/// 034, 035).
struct HeroDamage {
    next: u16,
    non_wounds: Vec<CardId>,
    wounds_to_hand: u32,
    knock_out_wounds: u32,
    knocked_out: bool,
    paralyzed: bool,
    armor: u8,
}

impl HeroDamage {
    fn new(p: &PlayerState, combat: &Combat) -> Self {
        // MKUE p. 9, 5.c : la limite de main non modifiée, celle du jeton de niveau (spec 035).
        let token = p.level_token();
        Self {
            next: p.wounds_given,
            non_wounds: p
                .hand
                .iter()
                .copied()
                .filter(|card| !card.card().is_wound())
                .collect(),
            wounds_to_hand: combat.wounds_to_hand,
            knock_out_wounds: u32::from(token.hand_limit),
            knocked_out: combat.knocked_out,
            paralyzed: false,
            armor: token.armor,
        }
    }

    // 65 535 numéros (`CardId` sur 16 bits, issue privée 76) : une partie n'en approche pas.
    fn take(&mut self, count: u32) -> Vec<CardId> {
        (0..count)
            .map(|_| {
                self.next = self.next.saturating_add(1);
                CardId::new(DeedCard::Wound, self.next)
            })
            .collect()
    }

    /// `damage` dégâts de `enemy` au héros : une Blessure par tranche entamée de son armure (MKUE
    /// p. 9, 5.a et 5.b), puis l'assommé (5.c) et les capacités de l'ennemi (MKUE p. 10, 6).
    fn events(&mut self, player: PlayerId, enemy: &CombatEnemy, damage: u32) -> Vec<Event> {
        let count = damage.div_ceil(u32::from(self.armor.max(1)));
        let wounds = self.take(count);
        let mut events = vec![Event::WoundsTaken {
            player,
            enemy: enemy.token,
            wounds,
        }];
        self.wounds_to_hand += count;
        if !self.knocked_out && self.wounds_to_hand >= self.knock_out_wounds {
            self.knocked_out = true;
            events.push(Event::KnockedOut {
                player,
                cards: std::mem::take(&mut self.non_wounds),
            });
        }
        if enemy.attacker_has(EnemyAbility::Poison) {
            events.push(Event::PoisonWoundsDiscarded {
                player,
                enemy: enemy.token,
                wounds: self.take(count),
            });
        }
        if enemy.attacker_has(EnemyAbility::Paralyze) && count > 0 && !self.paralyzed {
            self.paralyzed = true;
            let cards = std::mem::take(&mut self.non_wounds);
            if !cards.is_empty() {
                events.push(Event::HandParalyzed { player, cards });
            }
        }
        events
    }
}

/// Les unités qui peuvent recevoir des dégâts, dans l'ordre de la zone : non blessées, sans dégâts
/// déjà reçus dans ce combat (MKUE p. 9, 4 ; FAQ p3 ; Compendium, *Combat* S35), hors incendie.
pub(crate) fn damageable_units(p: &PlayerState) -> Vec<UnitCard> {
    // Specs 041 et 044 : aucune unité ne reçoit de dégâts pendant l'incendie d'un monastère, ni
    // dans un donjon ou un tombeau.
    if p.units_sit_out() {
        return Vec::new();
    }
    // Spec 053 : Au cœur du combat, pas de dégâts sur les unités ce tour.
    if p.modifiers
        .iter()
        .any(|modifier| modifier.kind == ModifierKind::NoDamageToUnits)
    {
        return Vec::new();
    }
    p.units
        .iter()
        .filter(|owned| !owned.is_wounded() && !owned.damaged)
        .map(|owned| owned.card)
        .collect()
}

/// Ce que `damage` dégâts de `enemy` font à cette unité : ses Blessures et les dégâts restants
/// (MKUE p. 9, *Assign Damage Phase* 4.a et 4.b ; p. 24, *Unit Resistances*). Résistante, elle
/// absorbe d'abord son armure sans Blessure ; toxique, deux Blessures (MKUE p. 10, 6.a).
pub(crate) fn unit_absorbs(
    enemy: &CombatEnemy,
    unit: UnitCard,
    damage: u32,
    all_resistances: bool,
    armor_bonus: u8,
    banner_resistances: &[Resistance],
) -> (u8, u32) {
    let definition = unit.unit.definition();
    let armor = u32::from(definition.armor.saturating_add(armor_bonus));
    let element = enemy.attack_element().unwrap_or(Element::Physical);
    // Spec 053 : Gardiens d'Altem, toutes les résistances (MKUE p. 9, 4.b).
    let resistant = all_resistances
        || attack_efficiency(
            element,
            [definition.resistances.as_slice(), banner_resistances],
        ) == Efficiency::Halved;
    let (wounds, remaining) = if !resistant {
        (1, damage.saturating_sub(armor))
    } else if damage <= armor {
        (0, 0)
    } else {
        (1, damage.saturating_sub(2 * armor))
    };
    let wounds = if wounds > 0 && enemy.attacker_has(EnemyAbility::Poison) {
        2
    } else {
        wounds
    };
    (wounds, remaining)
}

/// Spec 055 (Bannière de gloire) : l'Armure de plus de cette unité, par la bannière qu'elle porte
/// et par la bannière renforcée.
pub(crate) fn unit_armor_bonus(p: &PlayerState, unit: UnitCard) -> u8 {
    let banner = [
        crate::Artifact::BannerOfGlory,
        crate::Artifact::BannerOfProtection,
    ]
    .into_iter()
    .any(|banner| carries(p, unit, banner));
    p.modifiers
        .iter()
        .fold(u8::from(banner), |bonus, modifier| match modifier.kind {
            ModifierKind::UnitArmorBonus(more) => bonus.saturating_add(more),
            _ => bonus,
        })
}

/// Vrai si l'unité porte cette bannière (spec 055).
pub(crate) fn carries(p: &PlayerState, unit: UnitCard, banner: crate::Artifact) -> bool {
    p.units.iter().any(|owned| {
        owned.card == unit
            && owned
                .banner
                .is_some_and(|card| card.card() == DeedCard::Artifact(banner))
    })
}

/// Spec 055 (Bannière de protection) : les résistances au Feu et à la Glace de l'unité porteuse.
fn banner_resistances(p: &PlayerState, unit: UnitCard) -> &'static [Resistance] {
    if carries(p, unit, crate::Artifact::BannerOfProtection) {
        &[Resistance::Fire, Resistance::Ice]
    } else {
        &[]
    }
}

/// Les unités du joueur ont toutes les résistances (Gardiens d'Altem, spec 053).
fn has_all_resistances(p: &PlayerState) -> bool {
    p.modifiers
        .iter()
        .any(|modifier| modifier.kind == ModifierKind::UnitsAllResistances)
}

/// Les assignations légales de la phase d'assignation des dégâts (spec 042) : l'ennemi commencé,
/// ou chaque ennemi non paré dans l'ordre du combat (MKUE p. 9, 1.b) ; pour chacun, les unités
/// qui peuvent recevoir des dégâts, dans l'ordre de la zone, puis le héros.
pub(crate) fn assign_actions(p: &PlayerState, combat: &Combat) -> Vec<Action> {
    if combat.phase != CombatPhase::AssignDamage {
        return Vec::new();
    }
    let units = damageable_units(p);
    combat
        .enemies
        .iter()
        .filter(|enemy| enemy.has_pending_damage())
        .filter(|enemy| {
            combat
                .assigning
                .is_none_or(|(token, _)| combat.designates(enemy, token))
        })
        .flat_map(|enemy| {
            units
                .iter()
                .map(|&unit| DamageTarget::Unit(unit))
                .chain(std::iter::once(DamageTarget::Hero))
                .map(|to| Action::AssignDamage {
                    enemy: enemy.token,
                    to,
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

/// Les événements d'une assignation (spec 042) : les dégâts sur l'unité (détruite si l'ennemi est
/// paralysant), ou sur le héros ; le reste au héros quand plus aucune unité ne peut en recevoir
/// (L6) ; l'ennemi fini, les autres résolus seuls sans unité possible (C29), puis la phase
/// d'attaque une fois tous les dégâts faits.
pub(crate) fn assign_damage_events(
    state: &GameState,
    player: PlayerId,
    token: EnemyToken,
    to: DamageTarget,
) -> Result<Vec<Event>, ActionError> {
    let p = state.player(player).ok_or(ActionError::NotLegal)?;
    let combat = p.combat.as_ref().ok_or(ActionError::NotLegal)?;
    let enemy = combat
        .enemies
        .iter()
        .find(|enemy| combat.designates(enemy, token) && enemy.has_pending_damage())
        .ok_or(ActionError::NotLegal)?;
    let mut remaining = combat
        .remaining_damage(enemy)
        .ok_or(ActionError::NotLegal)?;
    let mut units = damageable_units(p);
    let mut hero = HeroDamage::new(p, combat);
    let mut events = Vec::new();
    match to {
        DamageTarget::Unit(unit) => {
            let (wounds, left) = unit_absorbs(
                enemy,
                unit,
                remaining,
                has_all_resistances(p),
                unit_armor_bonus(p, unit),
                banner_resistances(p, unit),
            );
            events.push(Event::DamageAssignedToUnit {
                player,
                enemy: token,
                unit,
                wounds,
            });
            // MKUE p. 10, 6.b ; Compendium, *Combat* S47 : détruite après avoir absorbé.
            if wounds > 0 && enemy.attacker_has(EnemyAbility::Paralyze) {
                events.push(Event::UnitDestroyed { player, unit });
            }
            units.retain(|&other| other != unit);
            remaining = left;
            if remaining > 0 && units.is_empty() {
                events.extend(hero.events(player, enemy, remaining));
                remaining = 0;
            }
        }
        DamageTarget::Hero => {
            events.extend(hero.events(player, enemy, remaining));
            remaining = 0;
        }
    }
    if remaining > 0 {
        return Ok(events);
    }
    events.extend(dismissed(player, enemy));
    let others: Vec<&CombatEnemy> = combat
        .enemies
        .iter()
        .filter(|other| !combat.designates(other, token) && other.has_pending_damage())
        .collect();
    if units.is_empty() {
        for other in &others {
            if let Some(damage) = other.damage() {
                events.extend(hero.events(player, other, damage));
            }
            events.extend(dismissed(player, other));
        }
    }
    if others.is_empty() || units.is_empty() {
        events.push(Event::CombatPhaseStarted {
            player,
            phase: CombatPhase::Attack,
        });
    }
    Ok(events)
}

/// Au début de la phase de parade, chaque Invocateur en jeu tire sa créature, dans l'ordre du
/// combat (MKUE p. 9, *Block Phase* 2 ; Compendium, *ORC SUMMONERS* S1).
fn summon_events(
    state: &GameState,
    player: PlayerId,
    combat: &Combat,
    rngs: &mut GameRngs,
) -> Vec<Event> {
    let mut piles = state.enemy_piles.clone();
    let mut discards = state.enemy_discards.clone();
    let mut events = Vec::new();
    for enemy in combat.enemies.iter().filter(|enemy| !enemy.defeated) {
        // Spec 053 : un Invocateur qui n'attaque pas ne tire pas de créature (Compendium,
        // *Illusionists* S2).
        if enemy.token.enemy().definition().attack != EnemyAttack::Summon || enemy.does_not_attack()
        {
            continue;
        }
        let (drawn, token) = draw_enemy_token(EnemyColor::Brown, &mut piles, &mut discards, rngs);
        events.extend(drawn);
        if let Some(token) = token {
            events.push(Event::EnemySummoned {
                player,
                summoner: enemy.token,
                token,
            });
        }
    }
    events
}

/// Passer à la phase suivante. Depuis la phase de parade, la phase d'assignation des dégâts se
/// résout aussitôt (Q3) : un `WoundsTaken` par ennemi qui blesse, ou rien et la phase est sautée
/// (MKUE p. 9, *Assign Damage Phase* 1.a).
pub(crate) fn next_phase_events(
    state: &GameState,
    player: PlayerId,
    rngs: &mut GameRngs,
) -> Result<Vec<Event>, ActionError> {
    let p = state.player(player).ok_or(ActionError::NotLegal)?;
    let combat = p.combat.as_ref().ok_or(ActionError::NotLegal)?;
    let phase = next_phase(combat.phase).ok_or(ActionError::NotLegal)?;
    let started = |phase| Event::CombatPhaseStarted { player, phase };
    match combat.phase {
        CombatPhase::RangedAndSiege => {
            let mut events = vec![started(phase)];
            events.extend(summon_events(state, player, combat, rngs));
            return Ok(events);
        }
        CombatPhase::Block => {}
        CombatPhase::AssignDamage | CombatPhase::Attack => return Ok(vec![started(phase)]),
    }
    // Spec 042 : une unité peut recevoir des dégâts, la phase attend les assignations.
    if combat.enemies.iter().any(CombatEnemy::has_pending_damage) && !damageable_units(p).is_empty()
    {
        return Ok(vec![started(CombatPhase::AssignDamage)]);
    }
    let damage = damage_events(player, p, combat);
    if damage.is_empty() {
        return Ok(vec![started(phase)]);
    }
    let mut events = vec![started(CombatPhase::AssignDamage)];
    events.extend(damage);
    events.push(started(phase));
    Ok(events)
}

/// Le nombre de numéros d'exemplaire que ces Blessures consomment (issue privée 76).
fn copies(wounds: &[CardId]) -> u16 {
    u16::try_from(wounds.len()).unwrap_or(u16::MAX)
}

/// Les Blessures vont dans la main (MKUE p. 11, *Wounds and Healing* 1).
pub(crate) fn apply_wounds(
    state: &mut GameState,
    player: PlayerId,
    enemy: EnemyToken,
    wounds: &[CardId],
) {
    if let Some(p) = state.player_mut(player) {
        p.hand.extend(wounds.iter().copied());
        p.wounds_given = p.wounds_given.saturating_add(copies(wounds));
        let given = u8::try_from(wounds.len()).unwrap_or(u8::MAX);
        if let Some(combat) = p.combat.as_mut() {
            combat.wounds_to_hand += u32::from(copies(wounds));
        }
        if let Some(combat) = p
            .combat
            .as_mut()
            .filter(|combat| combat.phase == CombatPhase::AssignDamage)
        {
            // Spec 042 : les dégâts de cet ennemi sont faits.
            let hit = combat.marked(&[enemy]);
            for (attacking, hit) in combat.enemies.iter_mut().zip(hit) {
                attacking.damage_assigned |= hit;
            }
            if combat.assigning.is_some_and(|(token, _)| token == enemy) {
                combat.assigning = None;
            }
        }
        if let Some(combat) = p.combat.as_mut()
            && let Some(attacking) = combat.enemy(enemy)
        {
            let record = CombatRecord::Damage {
                enemy,
                attacker: attacking.attacker().unwrap_or(attacking.token),
                damage: attacking.damage().unwrap_or(0),
                wounds: given,
                poisoned: 0,
                paralyzed: 0,
            };
            combat.history.push(record);
        }
    }
}

/// Le dernier dégât inscrit pour cet ennemi, pour y ajouter l'effet de ses capacités.
fn last_damage(combat: &mut Combat, enemy: EnemyToken) -> Option<&mut CombatRecord> {
    combat
        .history
        .iter_mut()
        .rev()
        .find(|record| matches!(record, CombatRecord::Damage { enemy: hex, .. } if *hex == enemy))
}

/// Attaque toxique : les Blessures vont dans la défausse (MKUE p. 10, 6.a ; spec 033).
pub(crate) fn apply_poison(
    state: &mut GameState,
    player: PlayerId,
    enemy: EnemyToken,
    wounds: &[CardId],
) {
    if let Some(p) = state.player_mut(player) {
        p.discard_pile.extend(wounds.iter().copied());
        p.wounds_given = p.wounds_given.saturating_add(copies(wounds));
        let given = u8::try_from(wounds.len()).unwrap_or(u8::MAX);
        if let Some(CombatRecord::Damage { poisoned, .. }) = p
            .combat
            .as_mut()
            .and_then(|combat| last_damage(combat, enemy))
        {
            *poisoned = given;
        }
    }
}

/// Attaque paralysante : ces cartes de la main vont dans la défausse (MKUE p. 10, 6.b ; spec 033).
/// Le dégât qui l'a causée est le dernier inscrit.
pub(crate) fn apply_paralysis(state: &mut GameState, player: PlayerId, cards: &[CardId]) {
    if let Some(p) = state.player_mut(player) {
        p.hand.retain(|card| !cards.contains(card));
        p.discard_pile.extend(cards.iter().copied());
        let count = u8::try_from(cards.len()).unwrap_or(u8::MAX);
        if let Some(CombatRecord::Damage { paralyzed, .. }) = p
            .combat
            .as_mut()
            .and_then(|combat| combat.history.last_mut())
        {
            *paralyzed = count;
        }
    }
}

/// Assommé : ces cartes de la main vont dans la défausse (MKUE p. 9, 5.c ; spec 034). Le dégât qui
/// l'a causé est le dernier inscrit.
pub(crate) fn apply_knock_out(state: &mut GameState, player: PlayerId, cards: &[CardId]) {
    if let Some(p) = state.player_mut(player) {
        p.hand.retain(|card| !cards.contains(card));
        p.discard_pile.extend(cards.iter().copied());
        if let Some(combat) = p.combat.as_mut() {
            combat.knocked_out = true;
            let enemy = combat.history.iter().rev().find_map(|record| match record {
                CombatRecord::Damage { enemy, .. } => Some(*enemy),
                _ => None,
            });
            if let Some(enemy) = enemy {
                combat.history.push(CombatRecord::KnockedOut {
                    enemy,
                    cards: u8::try_from(cards.len()).unwrap_or(u8::MAX),
                });
            }
        }
    }
}

/// La créature invoquée va dans la défausse de sa couleur (MKUE p. 9, *Block Phase* 7.a ; p. 10,
/// 6.c ; spec 033).
pub(crate) fn apply_dismiss(state: &mut GameState, player: PlayerId, summoner: EnemyToken) {
    let token = state
        .player_mut(player)
        .and_then(|p| p.combat.as_mut())
        .and_then(|combat| combat.dismiss(summoner));
    if let Some(token) = token {
        state
            .enemy_discards
            .entry(token.color())
            .or_default()
            .push(token);
    }
}

/// L'Invocateur de cette case tire sa créature : le jeton quitte le dessus de sa pile (spec 033).
pub(crate) fn apply_summon(
    state: &mut GameState,
    player: PlayerId,
    summoner: EnemyToken,
    token: EnemyToken,
) {
    let Some(pile) = state.enemy_piles.get_mut(&token.color()) else {
        return;
    };
    if pile.first() != Some(&token) {
        return;
    }
    pile.remove(0);
    if let Some(combat) = state.player_mut(player).and_then(|p| p.combat.as_mut()) {
        combat.summon(summoner, token);
    }
}

/// Les dégâts de cet ennemi sur l'unité (spec 042) : ses Blessures, plus de dégâts dans ce
/// combat ; l'ennemi est fini quand plus rien ne reste, sinon il reste en cours d'assignation.
pub(crate) fn apply_unit_damage(
    state: &mut GameState,
    player: PlayerId,
    enemy: EnemyToken,
    unit: UnitCard,
    wounds: u8,
) {
    let Some(p) = state.player_mut(player) else {
        return;
    };
    let all_resistances = has_all_resistances(p);
    let armor_bonus = unit_armor_bonus(p, unit);
    let resistances = banner_resistances(p, unit);
    let Some(owned) = p.units.iter_mut().find(|owned| owned.card == unit) else {
        return;
    };
    owned.damaged = true;
    owned.wounds = (owned.wounds + wounds).min(2);
    let Some(combat) = p.combat.as_mut() else {
        return;
    };
    let Some(index) = combat.enemies.iter().position(|e| e.token == enemy) else {
        return;
    };
    let attacking = combat.enemies[index].clone();
    let Some(damage) = combat.remaining_damage(&attacking) else {
        return;
    };
    let (_, remaining) = unit_absorbs(
        &attacking,
        unit,
        damage,
        all_resistances,
        armor_bonus,
        resistances,
    );
    if remaining == 0 {
        combat.enemies[index].damage_assigned = true;
        combat.assigning = None;
    } else {
        combat.assigning = Some((enemy, remaining));
    }
    combat.history.push(CombatRecord::UnitDamage {
        enemy,
        unit,
        wounds,
    });
}

/// L'unité détruite quitte la partie, Blessures comprises (MKUE p. 10, 6.b ; spec 042).
pub(crate) fn apply_unit_destroyed(state: &mut GameState, player: PlayerId, unit: UnitCard) {
    let Some(p) = state.player_mut(player) else {
        return;
    };
    p.units.retain(|owned| owned.card != unit);
    if let Some(combat) = p.combat.as_mut() {
        let enemy = combat.history.iter().rev().find_map(|record| match record {
            CombatRecord::UnitDamage {
                enemy, unit: hit, ..
            } if *hit == unit => Some(*enemy),
            _ => None,
        });
        if let Some(enemy) = enemy {
            combat
                .history
                .push(CombatRecord::UnitDestroyed { enemy, unit });
        }
    }
}
