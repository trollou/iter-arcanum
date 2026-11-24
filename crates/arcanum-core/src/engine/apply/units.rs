//! `apply` : unités et bannières.

use super::*;

/// `apply_event` pour les événements de ce thème ; les autres ne changent rien ici.
pub(super) fn unit_event(state: &mut GameState, event: &Event) {
    match event {
        // MKUE p. 8, 4.a : l'offre n'est pas complétée (spec 050).
        Event::UnitRecruited { player, unit, cost } => {
            if take_from_offer(state, Offer::Units, CommonCard::Unit(*unit))
                && let Some(p) = state.player_mut(*player)
            {
                p.units.push(OwnedUnit::recruited(*unit));
                p.spend_purchase_influence(*cost);
                // Spec 057 : un recrutement gratuit (Appel à la gloire) ne compte pas pour Récit
                // héroïque (spec 053 Q5) ; il consomme le recrutement gratuit.
                if *cost == 0 {
                    p.modifiers
                        .retain(|modifier| modifier.kind != ModifierKind::RecruitFromOffer);
                } else {
                    p.units_recruited = p.units_recruited.saturating_add(1);
                }
            }
        }
        // Spec 055 : Terreur, l'unité épuisée et l'ennemi qui n'attaque pas ; Courage, la bannière
        // retournée.
        Event::BannerUsed {
            player,
            unit,
            enemy,
        } => {
            if let Some(p) = state.player_mut(*player)
                && let Some(owned) = p.units.iter_mut().find(|owned| owned.card == *unit)
            {
                match enemy {
                    Some(enemy) => {
                        owned.ready = false;
                        if let Some(combat) = p.combat.as_mut() {
                            combat.pose_on_enemy(
                                *enemy,
                                &[ModifierKind::OnEnemy(EnemyModifierKind::DoesNotAttack)],
                                ModifierSource::Unit(*unit),
                                None,
                            );
                        }
                    }
                    None => owned.banner_flipped = true,
                }
            }
        }
        // Spec 055 (MKUE p. 5, 8.b) : les bannières reprises vont dans le deck par le mélange qui
        // suit ; toutes les bannières se remettent face visible.
        Event::BannersReclaimed { player, banners } => {
            if let Some(p) = state.player_mut(*player) {
                p.banner_reclaim = false;
                for owned in &mut p.units {
                    owned.banner_flipped = false;
                    if owned.banner.is_some_and(|banner| banners.contains(&banner)) {
                        owned.banner = None;
                    }
                }
            }
        }
        // Spec 055 : la bannière quitte la main pour l'unité ; c'est une carte jouée.
        Event::BannerAssigned { player, card, unit } => {
            if let Some(p) = state.player_mut(*player)
                && take_from_hand(&mut p.hand, *card)
            {
                if let Some(owned) = p.units.iter_mut().find(|owned| owned.card == *unit) {
                    owned.banner = Some(*card);
                    owned.banner_flipped = false;
                }
                p.played_elsewhere = true;
            }
        }
        // Spec 055 (MKUE p. 5, 8.a) : la bannière va dans la défausse.
        Event::BannerLost { player, card, unit } => {
            if let Some(p) = state.player_mut(*player)
                && let Some(owned) = p
                    .units
                    .iter_mut()
                    .find(|owned| owned.card == *unit && owned.banner == Some(*card))
            {
                owned.banner = None;
                owned.banner_flipped = false;
                p.discard_pile.push(*card);
            }
        }
        // MKUE p. 5, 4.a : l'unité quitte la partie, Blessures comprises.
        Event::UnitDisbanded { player, unit } => {
            if let Some(p) = state.player_mut(*player) {
                p.units.retain(|owned| owned.card != *unit);
            }
        }
        // MKUE p. 5, 5 : épuisée jusqu'à la fin de la manche ; l'effet compte comme une carte.
        Event::UnitActivated {
            player,
            unit,
            ability,
            option,
            effect,
            enemy,
        } => apply_unit_activated(state, player, unit, ability, option, effect, enemy),
        Event::UnitReadied { player, unit } => {
            if let Some(owned) = state
                .player_mut(*player)
                .and_then(|p| p.units.iter_mut().find(|owned| owned.card == *unit))
            {
                owned.ready = true;
            }
        }
        // MKUE p. 11, *Wounds and Healing* 2 : deux Blessures au plus.
        Event::UnitWounded {
            player,
            unit,
            wounds,
        } => {
            if let Some(owned) = state
                .player_mut(*player)
                .and_then(|p| p.units.iter_mut().find(|owned| owned.card == *unit))
            {
                owned.wounds = (owned.wounds + wounds).min(2);
            }
        }
        // MKUE p. 4, 2.g : blessées comprises, qui restent blessées.
        Event::UnitsReadied { player } => {
            if let Some(p) = state.player_mut(*player) {
                for owned in &mut p.units {
                    owned.ready = true;
                }
            }
        }
        Event::DamageAssignedToUnit {
            player,
            enemy,
            unit,
            wounds,
        } => apply_unit_damage(state, *player, *enemy, *unit, *wounds),
        Event::UnitDestroyed { player, unit } => {
            // Compendium, *Bonds of Loyalty* Q3 : le jeton de Loyauté se libère.
            if let Some(p) = state.player_mut(*player)
                && p.bonds_unit == Some(*unit)
            {
                p.bonds_unit = None;
            }
            apply_unit_destroyed(state, *player, *unit);
        }
        Event::UnitHealed { player, unit, cost } => {
            if let Some(p) = state.player_mut(*player) {
                p.healing = p.healing.saturating_sub(u32::from(*cost));
                spend_grail_fame(p, u32::from(*cost));
                if let Some(owned) = p.units.iter_mut().find(|owned| owned.card == *unit) {
                    owned.wounds = owned.wounds.saturating_sub(1);
                }
            }
        }
        // Spec 058 : le jeton de Loyauté sans faille reçoit l'unité.
        Event::UnitUnderBonds { player, unit } => {
            if let Some(p) = state.player_mut(*player) {
                p.bonds_unit = Some(*unit);
            }
        }
        // Spec 045 : gratuite, l'offre n'est pas complétée (Compendium, *Miscellaneous* S7).
        Event::UnitRewardTaken { player, unit } => {
            if take_from_offer(state, Offer::Units, CommonCard::Unit(*unit))
                && let Some(p) = state.player_mut(*player)
            {
                p.units.push(OwnedUnit::recruited(*unit));
            }
            if let Some(p) = state.player_mut(*player) {
                p.unit_choices = p.unit_choices.saturating_sub(1);
            }
        }
        Event::UnitRewardDeclined { player } => {
            if let Some(p) = state.player_mut(*player) {
                p.unit_choices = p.unit_choices.saturating_sub(1);
            }
        }
        _ => {}
    }
}

/// `Event::UnitActivated` : l'unité activée et son effet.
fn apply_unit_activated(
    state: &mut GameState,
    player: &PlayerId,
    unit: &UnitCard,
    ability: &u8,
    option: &u8,
    effect: &Effect,
    enemy: &Option<EnemyToken>,
) {
    if let Some(p) = state.player_mut(*player) {
        if let Some(owned) = p.units.iter_mut().find(|owned| owned.card == *unit) {
            owned.ready = false;
        } else if let Some(index) = p
            .modifiers
            .iter()
            .position(|modifier| modifier.kind == ModifierKind::UnitFromOffer)
        {
            // Spec 057 : Appel aux armes, une unité de l'offre, une fois ; elle y reste.
            p.modifiers.remove(index);
        } else {
            return;
        }
        p.unit_activated = true;
        add_effect_points(p, *effect);
        // Spec 055 : les unités qui attaquent ou parent ce tour (Bannière de gloire).
        if is_attack_or_block(*effect) && !p.units_fought.contains(unit) {
            p.units_fought.push(*unit);
        }
        let effect = with_attack_changes(
            &p.modifiers,
            p.combat.as_ref(),
            with_glory_banner(p, *unit, with_unit_bonus(&p.modifiers, *effect)),
        );
        if let Some(combat) = p.combat.as_mut() {
            combat.unit_activated = true;
            combat.add_unit_points(*unit, effect);
        }
        // Spec 052 : les modificateurs de l'option activée (Bûcherons).
        let definition = unit.unit.definition().abilities.get(usize::from(*ability));
        let modifiers = definition
            .and_then(|definition| definition.options().get(usize::from(*option)))
            .map(|choice| choice.modifiers.clone())
            .unwrap_or_default();
        let source = ModifierSource::Unit(*unit);
        add_modifiers(p, &modifiers, source);
        // Spec 053 : sur l'ennemi visé, de la couleur du mana de la capacité.
        if let Some(combat) = p.combat.as_mut()
            && let Some(target) = enemy
        {
            let color = match definition.and_then(|definition| definition.mana) {
                Some(ManaColor::Basic(color)) => Some(color),
                _ => None,
            };
            combat.pose_on_enemy(*target, &modifiers, source, color);
        }
    }
}

fn with_glory_banner(p: &PlayerState, unit: UnitCard, effect: Effect) -> Effect {
    if !glory_banner(p, unit) {
        return effect;
    }
    match effect {
        Effect::Attack {
            amount,
            element,
            kind,
        } => Effect::Attack {
            amount: amount.saturating_add(1),
            element,
            kind,
        },
        Effect::Block { amount, element } => Effect::Block {
            amount: amount.saturating_add(1),
            element,
        },
        Effect::SwiftBlock(amount) => Effect::SwiftBlock(amount.saturating_add(1)),
        other => other,
    }
}

/// L'effet d'une capacité d'unité avec le bonus d'Au cœur du combat : une Attaque ou une Parade,
/// de tout genre et de tout élément ; rien sur les autres effets (FAQ p2 ; spec 053).
fn with_unit_bonus(modifiers: &[Modifier], effect: Effect) -> Effect {
    let (attack, block) = modifiers
        .iter()
        .fold((0u8, 0u8), |(attack, block), modifier| {
            match modifier.kind {
                ModifierKind::UnitBonus {
                    attack: more_attack,
                    block: more_block,
                } => (
                    attack.saturating_add(more_attack),
                    block.saturating_add(more_block),
                ),
                _ => (attack, block),
            }
        });
    match effect {
        Effect::Attack {
            amount,
            element,
            kind,
        } => Effect::Attack {
            amount: amount.saturating_add(attack),
            element,
            kind,
        },
        Effect::Block { amount, element } => Effect::Block {
            amount: amount.saturating_add(block),
            element,
        },
        Effect::SwiftBlock(amount) => Effect::SwiftBlock(amount.saturating_add(block)),
        other => other,
    }
}
