//! Actions légales des compétences : usage, cibles, compétences jouées avec une carte de côté.

use super::*;

/// Spec 051 : les utilisations légales des compétences, dans l'ordre du gain, puis des options et
/// des unités visées. Une compétence prête (MKUE p. 5, *Using Skills* 4), codée (Q1), dont l'option
/// sert maintenant comme celle d'une carte (spec 052), au moment de la journée du combat (spec
/// 044) ; « sauf pendant un combat » hors combat (Compendium, *Inspiration* Q1) ; une seule
/// Motivation jusqu'à la fin du tour suivant (texte de la compétence).
pub(super) fn skill_actions(state: &GameState, player: PlayerId) -> Vec<Action> {
    let Some(p) = state.player(player) else {
        return Vec::new();
    };
    let in_combat = matches!(moment(p), Moment::Fighting(_));
    let time = p.combat_time(state.time_of_day);
    let mut actions = Vec::new();
    for owned in &p.skills {
        let definition = owned.skill.definition();
        // Spec 058 (Q2) : l'avantage d'Embrasure de la Source, au choix, un dé de base en plus.
        if owned.skill == Skill::SourceOpening {
            actions.extend(
                opening_dice(state, p)
                    .into_iter()
                    .map(|die| Action::UseSkill {
                        skill: Skill::SourceOpening,
                        option: 0,
                        unit: None,
                        target: Some(SkillTarget::Die(die)),
                    }),
            );
        }
        if owned.state != SkillState::Ready
            || (definition.out_of_combat && in_combat)
            || (owned.skill.is_motivation() && p.motivation_block > 0)
        {
            continue;
        }
        for (option, choice) in (0u8..).zip(definition.options()) {
            let effect = resolved_skill_effect(p, choice.effect(time));
            // Spec 058 : sans points (L4, aucune unité), ou sans Blessure à défausser, pas
            // proposée.
            if effect.points() == Some(0)
                || (effect == Effect::DiscardWoundToDraw
                    && !p.hand.iter().any(|card| card.card().is_wound()))
                || matches!(effect, Effect::ThrowWounds(count)
                    if p.hand.iter().filter(|card| card.card().is_wound()).count()
                        < usize::from(count))
                || !effect_playable(state, p, effect, &[])
            {
                continue;
            }
            // Spec 058 : les cibles (case du Vol…), une action par cible.
            if let Some(targets) = skill_targets(state, p, effect) {
                actions.extend(targets.into_iter().map(|target| Action::UseSkill {
                    skill: owned.skill,
                    option,
                    unit: None,
                    target,
                }));
                continue;
            }
            // Inspiration : une unité épuisée à rendre disponible, ou blessée à soigner.
            let units: Vec<Option<UnitCard>> = match effect {
                Effect::ReadyAnyUnit { .. } => {
                    ready_targets(p, u8::MAX).into_iter().map(Some).collect()
                }
                Effect::HealUnit => p
                    .units
                    .iter()
                    .filter(|owned| owned.is_wounded())
                    .map(|owned| Some(owned.card))
                    .collect(),
                _ => vec![None],
            };
            actions.extend(units.into_iter().map(|unit| Action::UseSkill {
                skill: owned.skill,
                option,
                unit,
                target: None,
            }));
        }
    }
    actions
}

/// Spec 058, groupe A : les jeux de côté de `card` avec une compétence prête, dans l'ordre des
/// compétences, de leurs options, du mana de Pouvoir universel (couleurs de base, puis noir) et des
/// sortes de points de côté que le moment permet (Compendium, *Universal Power* S1).
pub(super) fn skill_sideways_plays(
    state: &GameState,
    p: &PlayerState,
    card: CardId,
    kinds: &[SidewaysEffect],
) -> Vec<CardPlay> {
    let mut plays = Vec::new();
    for owned in &p.skills {
        let skill = owned.skill;
        // Spec 058 : l'avantage du Rituel de guérison, au tour suivant.
        let ready_sideways =
            owned.state == SkillState::Ready && skill.definition().effect == SkillEffect::Sideways;
        let advantage = skill == Skill::RitualOfPain && p.center_advantage(skill);
        if !ready_sideways && !advantage {
            continue;
        }
        for option in 0..skill.sideways_options() {
            // Compendium, *Who Needs Magic* Q1 : +3 seulement sans dé de la Source ce tour (Q4).
            if skill == Skill::WhoNeedsMagic && option == 1 && !p.dice_taken.is_empty() {
                continue;
            }
            let manas: Vec<Option<SkillMana>> = if skill == Skill::UniversalPower {
                let mut manas: Vec<Option<SkillMana>> = BasicColor::ALL
                    .into_iter()
                    .flat_map(|color| {
                        mana_payments(state, p, color)
                            .into_iter()
                            .map(move |payment| {
                                Some(SkillMana {
                                    color: ManaColor::Basic(color),
                                    payment,
                                })
                            })
                    })
                    .collect();
                manas.extend(black_payments(state, p).into_iter().map(|payment| {
                    Some(SkillMana {
                        color: ManaColor::Black,
                        payment,
                    })
                }));
                manas
            } else {
                vec![None]
            };
            for mana in manas {
                if skill
                    .sideways_value(option, card.card(), mana.map(|mana| mana.color))
                    .is_none()
                {
                    continue;
                }
                plays.extend(kinds.iter().map(|&sideways| CardPlay::SidewaysSkill {
                    sideways,
                    skill,
                    option,
                    mana,
                }));
            }
        }
    }
    plays
}

/// Spec 058 : les cibles d'un effet de compétence qui en demande une, dans l'ordre de la recherche ;
/// `None` pour un effet sans cible.
fn skill_targets(
    state: &GameState,
    p: &PlayerState,
    effect: Effect,
) -> Option<Vec<Option<SkillTarget>>> {
    match effect {
        Effect::Flight { spaces, cost } => Some(
            flight_destinations(state, p, spaces, cost)
                .into_iter()
                .map(|hex| Some(SkillTarget::Space(hex)))
                .collect(),
        ),
        // Compendium, *SOURCE Opening* S1 : un dé pas pris ce tour, ou aucun.
        Effect::SourceOpening => {
            let mut targets: Vec<Option<SkillTarget>> = (0u8..)
                .zip(&state.source)
                .filter(|&(die, _)| die_free(p, die))
                .map(|(die, _)| Some(SkillTarget::Die(die)))
                .collect();
            targets.push(None);
            Some(targets)
        }
        // Texte de Surcharge de mana : une couleur autre que l'or.
        Effect::ManaOverload => Some(
            BasicColor::ALL
                .into_iter()
                .map(ManaColor::Basic)
                .chain([ManaColor::Black])
                .map(|color| Some(SkillTarget::Color(color)))
                .collect(),
        ),
        // Compendium, *Resistance Break* S2, S3 : un ennemi pas vaincu, avec des résistances qui
        // comptent maintenant.
        Effect::ResistanceBreak => Some(match moment(p) {
            Moment::Fighting(combat) => combat
                .enemies
                .iter()
                .filter(|enemy| !enemy.defeated && !enemy.effective_resistances().is_empty())
                .map(|enemy| Some(SkillTarget::Enemy(enemy.token)))
                .collect(),
            Moment::Free | Moment::Engaging => Vec::new(),
        }),
        _ => None,
    }
}

/// Temps calme : −2 tout de suite, −1 au tour suivant (texte).
pub(in crate::engine) const CALM_NOW: u8 = 2;

pub(in crate::engine) const CALM_ADVANTAGE: u8 = 1;

/// Le coût de tous les terrains baissé de `by` pour le tour, jamais sous 1 (Temps calme, spec 058).
pub(in crate::engine) fn calm_weather(by: u8) -> Modifier {
    Modifier {
        kind: ModifierKind::TerrainCost {
            terrains: crate::Terrains::ALL,
            change: crate::TerrainChange::Reduce { by, min: 1 },
        },
        source: ModifierSource::Skill(Skill::CalmingTheWeather),
        duration: ModifierDuration::Turn,
    }
}

/// Le bonus de l'avantage de Surcharge de mana (texte).
pub(in crate::engine) const MANA_OVERLOAD_BONUS: u8 = 4;

/// La couleur du mana qui alimente la carte jouée (spec 058, Q5) : l'effet renforcé d'une action,
/// ou un sort lancé ; `None` pour un jeu de côté, un effet de base, un artefact.
pub(in crate::engine) fn powered_color(card: CardId, play: &CardPlay) -> Option<BasicColor> {
    match play {
        CardPlay::Strong { .. } | CardPlay::Cast { .. } => {
            card.card().definition().map(|definition| definition.color)
        }
        _ => None,
    }
}

/// Garder tel quel le dé de l'avantage d'Embrasure de la Source (Compendium, *SOURCE Opening* S1,
/// S3), une fois.
pub(super) fn keep_die_action(p: &PlayerState) -> Option<Action> {
    (p.opening_die.is_some() && !p.opening_die_kept).then_some(Action::KeepSourceDie)
}

/// Les dés que l'avantage d'Embrasure de la Source peut prendre en plus (Compendium, *SOURCE
/// Opening* S4, S5) : un dé de couleur de base pas pris, pendant le tour de l'avantage, sauf après
/// Magie inutile à +3.
fn opening_dice(state: &GameState, p: &PlayerState) -> Vec<u8> {
    if !p.center_advantage(Skill::SourceOpening) || p.source_dice_forbidden {
        return Vec::new();
    }
    (0u8..)
        .zip(&state.source)
        .filter(|&(die, face)| matches!(face, ManaColor::Basic(_)) && die_free(p, die))
        .map(|(die, _)| die)
        .collect()
}

/// Les arrivées du Vol (texte ; Compendium, *Flight* S3, S4, Q1) : `spaces` cases « voisines » au
/// plus (deux fois plus loin avec Distorsion spatiale), par toutes les cases révélées, vers une case
/// sûre sans ennemi errant ; `cost` Mouvement à payer.
fn flight_destinations(state: &GameState, p: &PlayerState, spaces: u8, cost: u8) -> Vec<Hex> {
    let Some(from) = p.figure else {
        return Vec::new();
    };
    if p.move_points < u32::from(cost) {
        return Vec::new();
    }
    let bending = p
        .modifiers
        .iter()
        .any(|modifier| modifier.kind == ModifierKind::SpaceBending);
    let max = if bending { spaces * 2 } else { spaces };
    let rampaging = rampaging_enemies(state);
    let conquered = state.conquered_sites();
    crate::reach(&state.map, from, max, |_| true)
        .into_iter()
        .map(|(hex, _)| hex)
        .filter(|&hex| is_safe_space(&state.map, hex, &conquered) && !rampaging.contains(&hex))
        .collect()
}

/// L'effet d'une compétence au moment de l'utiliser (spec 058) : Fortune clinquante compte les
/// couleurs de cristal, Marche en avant les unités disponibles et non blessées.
pub(in crate::engine) fn resolved_skill_effect(p: &PlayerState, effect: Effect) -> Effect {
    match effect {
        Effect::CrystalColorInfluence => {
            let colors = BasicColor::ALL
                .into_iter()
                .filter(|&color| p.inventory.crystals(color) > 0)
                .count();
            Effect::InteractionInfluence(u8::try_from(colors).unwrap_or(u8::MAX))
        }
        Effect::MovePerUnit { max } => {
            let ready = p
                .units
                .iter()
                .filter(|owned| owned.ready && !owned.is_wounded())
                .count();
            Effect::Move(u8::try_from(ready).unwrap_or(u8::MAX).min(max))
        }
        other => other,
    }
}
