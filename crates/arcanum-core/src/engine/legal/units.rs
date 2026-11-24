//! Actions légales des unités : activation, commandement.

use super::*;

/// Les unités qu'un effet « rendre disponible » de ce niveau peut viser, dans l'ordre de la zone :
/// épuisées, de ce niveau au plus (MKUE p. 5, 6 ; Compendium, *Silver Units* S10).
pub(super) fn ready_targets(p: &PlayerState, level: u8) -> Vec<UnitCard> {
    p.units
        .iter()
        .filter(|owned| !owned.ready && owned.card.unit.definition().level <= level)
        .map(|owned| owned.card)
        .collect()
}

/// Les activations légales d'unités (spec 042), dans l'ordre de la zone, puis des capacités, de
/// leurs options, des paiements et des cibles : une unité disponible et non blessée (MKUE p. 5,
/// 5 ; p. 11, 2.a), une capacité codée (Q2) dont l'effet se joue maintenant, comme celui d'une
/// carte ; soin et « rendre disponible » jamais en combat (MKUE p. 8, *Combat with Enemies* 6).
pub(super) fn unit_actions(state: &GameState, player: PlayerId) -> Vec<Action> {
    let Some(p) = state.player(player) else {
        return Vec::new();
    };
    let in_combat = matches!(moment(p), Moment::Fighting(_));
    // Specs 041 et 044 : les unités ne participent pas au combat d'un incendie, d'un donjon ou
    // d'un tombeau (cartes de site).
    // Spec 058 (Compendium, *Bonds of Loyalty* Q1) : sauf l'unité sous Loyauté sans faille.
    let sit_out = in_combat && p.units_sit_out();
    if sit_out && p.bonds_unit.is_none() {
        return Vec::new();
    }
    let mut units: Vec<(UnitCard, bool)> = p
        .units
        .iter()
        .filter(|owned| owned.ready && !owned.is_wounded())
        .filter(|owned| !sit_out || Some(owned.card) == p.bonds_unit)
        .map(|owned| (owned.card, false))
        .collect();
    // Spec 057 : Appel aux armes, les unités de l'offre (Compendium, *Call to Arms* S2).
    if !sit_out
        && p.modifiers
            .iter()
            .any(|modifier| modifier.kind == ModifierKind::UnitFromOffer)
    {
        units.extend(
            state
                .offer(Offer::Units)
                .iter()
                .filter_map(|card| card.unit())
                .map(|unit| (unit, true)),
        );
    }
    let leadership_ready = p
        .skills
        .iter()
        .any(|owned| owned.skill == Skill::Leadership && owned.state == SkillState::Ready);
    let phase = match moment(p) {
        Moment::Fighting(combat) => Some(combat.phase),
        Moment::Free | Moment::Engaging => None,
    };
    let mut actions = Vec::new();
    for (unit, from_offer) in units {
        for (ability, definition) in (0u8..).zip(&unit.unit.definition().abilities) {
            // Spec 057 (Q6) : une unité de l'offre ne se blesse pas (Guerriers d'Utem).
            if from_offer && definition.wounds_unit() {
                continue;
            }
            for (option, choice) in (0u8..).zip(definition.options()) {
                let effect = choice.effect;
                // Spec 053 : une capacité qui vise un ennemi se joue sur chaque cible légale du
                // combat, et pas du tout sans cible (Q7).
                let on_enemy = choice
                    .modifiers
                    .iter()
                    .any(|kind| kind.on_enemy().is_some());
                let enemies: Vec<Option<EnemyToken>> = match moment(p) {
                    Moment::Fighting(combat) if on_enemy => {
                        unit_enemy_targets(combat, &choice.modifiers)
                            .into_iter()
                            .map(Some)
                            .collect()
                    }
                    _ if on_enemy => Vec::new(),
                    _ => vec![None],
                };
                let not_in_combat = matches!(effect, Effect::Heal(_) | Effect::ReadyUnit(_));
                if (in_combat && not_in_combat)
                    || enemies.is_empty()
                    || (!on_enemy && !effect_playable(state, p, effect, &choice.modifiers))
                {
                    continue;
                }
                let targets: Vec<Option<UnitCard>> = match effect {
                    Effect::ReadyUnit(level) => {
                        ready_targets(p, level).into_iter().map(Some).collect()
                    }
                    _ => vec![None],
                };
                let mut colors: Vec<BasicColor> = Vec::new();
                let lists = match definition.mana {
                    // Spec 053 : le mana noir des Mages d'Altem, de nuit seulement (MKUE p. 5).
                    Some(ManaColor::Black) => black_payments(state, p)
                        .into_iter()
                        .map(|payment| vec![payment])
                        .collect(),
                    Some(ManaColor::Gold) => continue,
                    Some(ManaColor::Basic(color)) => {
                        colors.push(color);
                        colors.extend(&choice.extra_mana);
                        payment_lists(state, p, &colors)
                    }
                    None => {
                        colors.extend(&choice.extra_mana);
                        payment_lists(state, p, &colors)
                    }
                };
                // Spec 058 : Charisme, si la capacité donne des points qu'il augmente.
                let mut leaderships = vec![false];
                if leadership_ready && leadership_effect(effect, phase).is_some() {
                    leaderships.push(true);
                }
                for mana in lists {
                    for &enemy in &enemies {
                        for &target in &targets {
                            actions.extend(leaderships.iter().map(|&leadership| {
                                Action::ActivateUnit {
                                    unit,
                                    ability,
                                    option,
                                    mana: mana.clone(),
                                    target,
                                    enemy,
                                    leadership,
                                }
                            }));
                        }
                    }
                }
            }
        }
    }
    actions.extend(banner_use_actions(p));
    actions
}

/// La capacité activée avec Charisme (texte ; Compendium, *Leadership* S1, S2) : Parade +3, Attaque
/// +2, Attaque à distance +1 en phase à distance (+2 ensuite, comme toute Attaque) ; un siège, rien
/// en phase à distance. `None` si la capacité ne donne pas ces points. L'écran en tire la valeur du
/// second bouton de la capacité (spec 058, Interface).
pub fn leadership_effect(effect: Effect, phase: Option<crate::CombatPhase>) -> Option<Effect> {
    let ranged_phase = phase == Some(crate::CombatPhase::RangedAndSiege);
    match effect {
        Effect::Block { amount, element } => Some(Effect::Block {
            amount: amount + 3,
            element,
        }),
        Effect::SwiftBlock(amount) => Some(Effect::SwiftBlock(amount + 3)),
        Effect::Attack {
            amount,
            element,
            kind,
        } => {
            let bonus = match (kind, ranged_phase) {
                (AttackKind::Siege, true) => return None,
                (AttackKind::Ranged, true) => 1,
                _ => 2,
            };
            Some(Effect::Attack {
                amount: amount + bonus,
                element,
                kind,
            })
        }
        _ => None,
    }
}
