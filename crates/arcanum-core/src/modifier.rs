//! Effets en cours (modificateurs, spec 052, ADR 0008) : ce qu'un effet joué change aux règles pour
//! la suite du tour. Posés par l'`apply` de l'événement qui joue l'effet (`CardPlayed`,
//! `UnitActivated`), retirés par l'événement qui clôt leur durée.

use serde::Deserialize;

use crate::{
    AttackKind, BasicColor, CardDestination, CardId, Element, Resistance, Terrain, UnitCard,
};

/// Un effet en cours : sa sorte, d'où il vient, quand il disparaît.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Modifier {
    pub kind: ModifierKind,
    pub source: ModifierSource,
    pub duration: ModifierDuration,
}

/// La règle changée, paramétrée (ADR 0008) : une carte nouvelle qui change la même règle n'ajoute
/// que des données (`content/*.ron`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum ModifierKind {
    /// Coût de déplacement des terrains visés (MKUE p. 7, 7.a).
    TerrainCost {
        terrains: Terrains,
        change: TerrainChange,
    },
    /// Un terrain infranchissable devient franchissable (MKUE p. 7, 7.b) ; son coût vient d'un
    /// `TerrainCost` qui le fixe.
    Passable(Terrain),
    /// Dépenser des points d'une réserve comme d'autres points, `rate` points pour 1 (Agilité,
    /// Diplomatie).
    Convert {
        from: PointPool,
        to: ConvertedPoints,
        rate: u8,
    },
    /// Bonus de la prochaine carte jouée pour une Attaque ou une Parade (Embuscade) ; consommé par
    /// elle.
    NextCardBonus { attack: u8, block: u8 },
    /// Réputation changée à la fin du tour (Intimidation).
    EndOfTurnReputation(i8),
    /// À la fin du tour, la carte source peut aller sur (`top`) ou sous le deck au lieu de la
    /// défausse (Temporisation).
    ReturnToDeck { top: bool },
    /// Posé sur l'ennemi visé, pour le combat (spec 053) : celui de la parade en cours pour une
    /// carte (Bouclier de glace), l'ennemi choisi pour une capacité d'unité (Glaceurs d'Amotep).
    OnEnemy(EnemyModifierKind),
    /// Comme `OnEnemy`, sur un ennemi non fortifié seulement (Illusionnistes, spec 053).
    OnUnfortifiedEnemy(EnemyModifierKind),
    /// Les capacités d'unités qui donnent une Attaque ou une Parade en donnent autant de plus, pour
    /// le combat (Au cœur du combat, spec 053).
    UnitBonus { attack: u8, block: u8 },
    /// Les dégâts ne s'assignent qu'au héros (Au cœur du combat, spec 053).
    NoDamageToUnits,
    /// À la fin du tour, par unité recrutée ce tour : gloire et réputation (Récit héroïque,
    /// spec 053).
    EndOfTurnPerRecruit { fame: u8, reputation: i8 },
    /// Toute Attaque ajoutée ensuite est de cet élément, pour le combat (Mages d'Altem, spec 053).
    AttacksBecome(Element),
    /// Toute Attaque ajoutée ensuite est aussi de siège, pour le combat (Mages d'Altem, spec 053).
    AttacksGainSiege,
    /// Les unités du joueur sont résistantes au physique, au Feu et à la Glace (Gardiens
    /// d'Altem, spec 053).
    UnitsAllResistances,
    /// À la fin du tour, les cristaux dépensés ce tour reviennent dans l'inventaire (Maîtrise du
    /// cristal renforcée, spec 056).
    ReturnSpentCrystals,
    /// Une fois : dépenser `cost` Influence pour une action avancée de l'offre, mise à `to`
    /// (Apprentissage, spec 056) ; retiré par l'achat.
    LearningDiscount { cost: u8, to: CardDestination },
    /// Posé sur l'ennemi visé en jouant le sort, pour le combat (spec 054).
    OnTargetEnemy(EnemyModifierKind),
    /// Sur l'ennemi visé, `fortified` s'il est fortifié au moment où le sort est joué, `plain`
    /// sinon (Tremblement de terre, spec 054).
    OnTargetEnemyByFortification {
        plain: EnemyModifierKind,
        fortified: EnemyModifierKind,
    },
    /// Sur chaque ennemi du combat pas vaincu, sauf résistance (spec 054).
    OnAllEnemies(EnemyModifierKind),
    /// Sur chaque ennemi, selon qu'il est fortifié (Tremblement de terre, spec 054, *Earthquake*
    /// S5).
    OnAllEnemiesByFortification {
        plain: EnemyModifierKind,
        fortified: EnemyModifierKind,
    },
    /// Les défenseurs ne sont plus fortifiés par leur site, pour le tour (Démolition, spec 054).
    IgnoreSiteFortifications,
    /// Une Attaque à ajouter à une attaque de la phase d'attaque, pour le combat : posée par la
    /// parade réussie du Bouclier ardent (spec 054).
    DeferredAttack { amount: u8, element: Element },
    /// L'ennemi paré est détruit si la parade réussit (Bouclier explosif, spec 054).
    DestroyBlockedEnemy,
    /// Une fois : lancer un sort de cette couleur de l'offre de sorts (Don pour la magie, spec
    /// 054) ; retiré par ce sort.
    CastFromOffer(BasicColor),
    /// Pour le tour, les cases à distance 2 comptent comme voisines pour se déplacer et explorer, et
    /// aucun déplacement ne provoque (Distorsion spatiale, spec 057).
    SpaceBending,
    /// À la fin du tour, les cartes de la zone de jeu reviennent en main, sans pioche, et un tour
    /// supplémentaire suit (Distorsion temporelle, spec 057).
    TimeBending,
    /// Une fois ce tour : activer une capacité d'une unité de l'offre d'unités comme une des
    /// siennes, sans l'épuiser ni la blesser (Appel aux armes, spec 057) ; retiré par l'activation.
    UnitFromOffer,
    /// Une fois ce tour : recruter gratuitement une unité de l'offre, partout et à tout moment
    /// (Appel à la gloire, spec 057) ; retiré par le recrutement.
    RecruitFromOffer,
    /// La prochaine pioche du joueur va jusqu'à la limite de main plus ce nombre (Méditation,
    /// spec 057) ; dure jusqu'à cette pioche.
    DrawBonus(u8),
    /// Un jeton de cette couleur au début de chaque tour de la manche (Puits de mana, spec 057).
    ManaClaimPerTurn(BasicColor),
    /// Spec 055 (Amulette du soleil, de nuit) : forêts à 3, mana or utilisable, défenseurs et
    /// ruines révélés comme de jour, pour le tour.
    DayRules,
    /// Spec 055 (Amulette des ténèbres, de jour) : déserts à 3, mana noir utilisable comme de nuit,
    /// pour le tour.
    NightMana,
    /// Spec 055 (anneau renforcé) : tout mana de cette couleur ou noir se paie sans dé ni jeton,
    /// pour le tour ; Gloire +1 par sort de cette couleur lancé ce tour.
    EndlessMana(BasicColor),
    /// Spec 055 (Bannière de gloire renforcée) : Armure +1 pour les unités, pour le tour.
    UnitArmorBonus(u8),
    /// Spec 055 (Bannière de gloire renforcée) : Gloire +1 par unité qui attaque ou pare ce tour,
    /// une fois par unité.
    FamePerUnitAttackOrBlock,
    /// Spec 055 (Bannière de protection renforcée) : au début de la fin du tour, les Blessures
    /// reçues ce tour en main et dans la défausse sont jetées.
    ThrowAwayReceivedWounds,
    /// Spec 055 (Épée de justice) : Gloire +1 par ennemi vaincu ce tour, avant comme après.
    FamePerEnemyDefeated,
    /// Spec 055 (Épée de justice renforcée) : en phase d'attaque, chaque Attaque physique ajoutée
    /// ensuite compte double, pour le tour.
    DoublePhysicalAttacks,
    /// Spec 055 (Graal doré) : Gloire +1 par point de soin dépensé, pour les `left` premiers.
    GrailFame { left: u8 },
    /// Spec 055 (Graal doré renforcé) : une carte piochée par Blessure de la main jetée par un
    /// Soin, pour le tour.
    DrawPerHandWoundHealed,
}

impl ModifierKind {
    /// La durée d'un modificateur de cette sorte (ADR 0008) : le combat pour les bonus d'unités et
    /// les attaques changées (spec 053), le tour sinon.
    pub fn duration(self) -> ModifierDuration {
        match self {
            ModifierKind::UnitBonus { .. }
            | ModifierKind::AttacksBecome(_)
            | ModifierKind::AttacksGainSiege
            | ModifierKind::OnEnemy(_)
            | ModifierKind::OnUnfortifiedEnemy(_)
            | ModifierKind::OnTargetEnemy(_)
            | ModifierKind::OnTargetEnemyByFortification { .. }
            | ModifierKind::OnAllEnemies(_)
            | ModifierKind::OnAllEnemiesByFortification { .. }
            | ModifierKind::DeferredAttack { .. }
            | ModifierKind::DestroyBlockedEnemy => ModifierDuration::Combat,
            ModifierKind::DrawBonus(_) => ModifierDuration::UntilDraw,
            ModifierKind::ManaClaimPerTurn(_) => ModifierDuration::Round,
            _ => ModifierDuration::Turn,
        }
    }

    /// Vrai si le modificateur va dans les effets en cours du joueur ; sinon il va sur un ou des
    /// ennemis du combat (specs 053, 054), ou attend la parade réussie (spec 054).
    pub fn is_player_modifier(self) -> bool {
        !matches!(
            self,
            ModifierKind::OnEnemy(_)
                | ModifierKind::OnUnfortifiedEnemy(_)
                | ModifierKind::OnTargetEnemy(_)
                | ModifierKind::OnTargetEnemyByFortification { .. }
                | ModifierKind::OnAllEnemies(_)
                | ModifierKind::OnAllEnemiesByFortification { .. }
                | ModifierKind::DeferredAttack { .. }
                | ModifierKind::DestroyBlockedEnemy
        )
    }

    /// Vrai si le modificateur vise l'ennemi choisi en jouant le sort (spec 054).
    pub fn targets_enemy(self) -> bool {
        matches!(
            self,
            ModifierKind::OnTargetEnemy(_) | ModifierKind::OnTargetEnemyByFortification { .. }
        )
    }

    /// La sorte posée sur un ennemi, s'il en est une (spec 053).
    pub fn on_enemy(self) -> Option<EnemyModifierKind> {
        match self {
            ModifierKind::OnEnemy(kind) | ModifierKind::OnUnfortifiedEnemy(kind) => Some(kind),
            _ => None,
        }
    }
}

/// Ce qui change un ennemi du combat jusqu'à la fin du combat (ADR 0008, spec 053).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum EnemyModifierKind {
    /// L'armure baisse de `by`, jamais sous `min` (Bouclier de glace, Glaceurs d'Amotep).
    ArmorReduction { by: u8, min: u8 },
    /// L'ennemi n'attaque pas dans ce combat (Illusionnistes, Glaceurs d'Amotep).
    DoesNotAttack,
    /// Ni Fortifié par son jeton, ni fortifié par son site (Exposition, spec 054).
    LosesFortifications,
    /// Plus aucune résistance : physique, Feu, Glace (Exposition, spec 054).
    LosesResistances,
    /// Plus cette résistance (Frisson : le Feu ; spec 054).
    LosesResistance(Resistance),
}

/// Un modificateur d'ennemi : sa sorte et d'où il vient.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EnemyModifier {
    pub kind: EnemyModifierKind,
    pub source: ModifierSource,
}

/// Les terrains visés par un `TerrainCost` : tous, ou une liste (gardée en masque de bits pour que
/// la sorte reste `Copy`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(from = "TerrainsText")]
pub struct Terrains(u16);

#[derive(Deserialize)]
enum TerrainsText {
    All,
    Only(Vec<Terrain>),
}

impl From<TerrainsText> for Terrains {
    fn from(text: TerrainsText) -> Self {
        match text {
            TerrainsText::All => Terrains::ALL,
            TerrainsText::Only(terrains) => Terrains::of(&terrains),
        }
    }
}

impl Terrains {
    pub const ALL: Terrains = Terrains(u16::MAX);

    pub fn of(terrains: &[Terrain]) -> Self {
        Terrains(
            terrains
                .iter()
                .fold(0, |bits, &terrain| bits | Self::bit(terrain)),
        )
    }

    fn bit(terrain: Terrain) -> u16 {
        1 << (terrain as u16)
    }

    pub fn contains(self, terrain: Terrain) -> bool {
        self.0 & Self::bit(terrain) != 0
    }

    /// Tous les terrains (« tous les terrains » à l'écran).
    pub fn is_all(self) -> bool {
        self == Terrains::ALL
    }

    /// Les terrains visés, dans l'ordre de `Terrain::ALL`.
    pub fn list(self) -> Vec<Terrain> {
        Terrain::ALL
            .into_iter()
            .filter(|&terrain| self.contains(terrain))
            .collect()
    }
}

/// Changement du coût d'un terrain.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum TerrainChange {
    /// Moins `by`, sans descendre sous `min` ; un coût déjà sous `min` ne remonte pas.
    Reduce { by: u8, min: u8 },
    /// À cette valeur ; un coût déjà plus bas ne remonte pas.
    SetTo(u8),
}

impl TerrainChange {
    /// Le coût après ce changement.
    pub fn applied(self, cost: u8) -> u8 {
        match self {
            TerrainChange::Reduce { by, min } => {
                if cost <= min {
                    cost
                } else {
                    cost.saturating_sub(by).max(min)
                }
            }
            TerrainChange::SetTo(value) => cost.min(value),
        }
    }
}

/// La réserve de points dépensée par une conversion.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum PointPool {
    Move,
    Influence,
}

/// Ce que donne une conversion : une Attaque d'un genre ou une Parade d'un élément.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum ConvertedPoints {
    Attack { element: Element, kind: AttackKind },
    Block { element: Element },
}

/// D'où vient un modificateur ; porte la couleur de l'effet pour les résistances (spec 053).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ModifierSource {
    Card(CardId),
    Unit(UnitCard),
    /// Une compétence (spec 058 : Botte secrète, Temps calme).
    Skill(crate::Skill),
}

/// Quand un modificateur disparaît (ADR 0008) : `TurnEnded` retire les `Turn`, `CombatEnded` les
/// `Combat`. `UntilNextTurn` et `Round` viendront avec les compétences et les specs qui en ont
/// besoin.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ModifierDuration {
    Turn,
    Combat,
    /// Jusqu'à la prochaine pioche du joueur qui tire au moins une carte (`DrawBonusUsed`, spec
    /// 057).
    UntilDraw,
    /// Jusqu'à la fin de la manche (`RoundEnded`, spec 057).
    Round,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::PlayerState;
    use crate::{AdvancedAction, DeedCard, Event, GameState, PlayerId, apply};

    fn modifier(duration: ModifierDuration) -> Modifier {
        Modifier {
            kind: ModifierKind::EndOfTurnReputation(-1),
            source: ModifierSource::Card(CardId::new(
                DeedCard::Advanced(AdvancedAction::Intimidate),
                1,
            )),
            duration,
        }
    }

    fn with_modifiers(modifiers: Vec<Modifier>) -> GameState {
        GameState {
            players: vec![PlayerState {
                modifiers,
                combat: Some(crate::Combat::new(&[])),
                ..PlayerState::default()
            }],
            ..GameState::default()
        }
    }

    // ADR 0008 : chaque événement de fin retire les modificateurs de sa durée, jamais « tout ».
    #[test]
    fn combat_end_removes_only_combat_modifiers() {
        let state = with_modifiers(vec![
            modifier(ModifierDuration::Turn),
            modifier(ModifierDuration::Combat),
        ]);
        let state = apply(
            state,
            &Event::CombatEnded {
                player: PlayerId::SOLO,
            },
        );
        assert_eq!(
            state.modifiers(PlayerId::SOLO),
            [modifier(ModifierDuration::Turn)]
        );
    }

    // MKUE p. 5, *Effects* 1.b : la fin du tour retire les modificateurs du tour.
    #[test]
    fn turn_end_removes_turn_modifiers() {
        let state = with_modifiers(vec![modifier(ModifierDuration::Turn)]);
        let state = apply(
            state,
            &Event::TurnEnded {
                player: PlayerId::SOLO,
            },
        );
        assert!(state.modifiers(PlayerId::SOLO).is_empty());
    }

    // MKUE p. 7, 7.a : « min. 2 » ne remonte pas un coût plus bas ; « à 2 » non plus.
    #[test]
    fn terrain_changes_never_raise_a_cost() {
        let reduce = TerrainChange::Reduce { by: 1, min: 2 };
        assert_eq!(reduce.applied(5), 4);
        assert_eq!(reduce.applied(3), 2);
        assert_eq!(reduce.applied(2), 2);
        assert_eq!(reduce.applied(1), 1);
        assert_eq!(TerrainChange::SetTo(2).applied(5), 2);
        assert_eq!(TerrainChange::SetTo(2).applied(1), 1);
        // Bûcherons : jamais sous 0.
        assert_eq!(TerrainChange::Reduce { by: 1, min: 0 }.applied(1), 0);
    }
}
