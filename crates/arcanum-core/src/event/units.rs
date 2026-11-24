//! Événements sur les unités et les bannières : point de non-retour (`blocks_undo`) et filtrage (`filter`).

use super::*;

/// Point de non-retour des événements du thème (voir `Event::blocks_undo`).
pub(super) fn blocks_undo(event: &Event) -> bool {
    match event {
        // Spec 058 : publics, aucun hasard.
        Event::UnitUnderBonds { .. }
        // Spec 042 : publics, aucun hasard.
        | Event::UnitRecruited { .. }
        | Event::UnitDisbanded { .. }
        // Spec 055 : publics, aucun hasard.
        | Event::BannerAssigned { .. }
        | Event::BannerLost { .. }
        | Event::BannersReclaimed { .. }
        | Event::BannerUsed { .. }
        | Event::UnitActivated { .. }
        | Event::UnitReadied { .. }
        | Event::UnitWounded { .. }
        | Event::DamageAssignedToUnit { .. }
        | Event::UnitDestroyed { .. }
        | Event::UnitHealed { .. }
        | Event::UnitsReadied { .. }
        // Spec 045 : public, aucun hasard.
        | Event::UnitRewardTaken { .. }
        | Event::UnitRewardDeclined { .. } => false,
        // Invariant : `Event::theme` n'envoie ici que les événements du thème.
        _ => unreachable!("événement hors du thème units"),
    }
}

/// L'événement tel que `viewer` a le droit de le voir (voir `filter_event`).
pub(super) fn filter(event: &Event, _viewer: PlayerId) -> PlayerEvent {
    match event {
        Event::UnitUnderBonds { player, unit } => PlayerEvent::UnitUnderBonds {
            player: *player,
            unit: *unit,
        },
        Event::UnitRecruited { player, unit, cost } => PlayerEvent::UnitRecruited {
            player: *player,
            unit: *unit,
            cost: *cost,
        },
        Event::BannerAssigned { player, card, unit } => PlayerEvent::BannerAssigned {
            player: *player,
            card: *card,
            unit: *unit,
        },
        Event::BannerLost { player, card, unit } => PlayerEvent::BannerLost {
            player: *player,
            card: *card,
            unit: *unit,
        },
        Event::BannersReclaimed { player, banners } => PlayerEvent::BannersReclaimed {
            player: *player,
            banners: banners.clone(),
        },
        Event::BannerUsed {
            player,
            unit,
            enemy,
        } => PlayerEvent::BannerUsed {
            player: *player,
            unit: *unit,
            enemy: *enemy,
        },
        Event::UnitDisbanded { player, unit } => PlayerEvent::UnitDisbanded {
            player: *player,
            unit: *unit,
        },
        Event::UnitActivated {
            player,
            unit,
            ability,
            option,
            effect,
            enemy,
        } => PlayerEvent::UnitActivated {
            player: *player,
            unit: *unit,
            ability: *ability,
            option: *option,
            effect: *effect,
            enemy: *enemy,
        },
        Event::UnitReadied { player, unit } => PlayerEvent::UnitReadied {
            player: *player,
            unit: *unit,
        },
        Event::UnitWounded {
            player,
            unit,
            wounds,
        } => PlayerEvent::UnitWounded {
            player: *player,
            unit: *unit,
            wounds: *wounds,
        },
        Event::DamageAssignedToUnit {
            player,
            enemy,
            unit,
            wounds,
        } => PlayerEvent::DamageAssignedToUnit {
            player: *player,
            enemy: *enemy,
            unit: *unit,
            wounds: *wounds,
        },
        Event::UnitDestroyed { player, unit } => PlayerEvent::UnitDestroyed {
            player: *player,
            unit: *unit,
        },
        Event::UnitHealed { player, unit, cost } => PlayerEvent::UnitHealed {
            player: *player,
            unit: *unit,
            cost: *cost,
        },
        Event::UnitsReadied { player } => PlayerEvent::UnitsReadied { player: *player },
        Event::UnitRewardTaken { player, unit } => PlayerEvent::UnitRewardTaken {
            player: *player,
            unit: *unit,
        },
        Event::UnitRewardDeclined { player } => PlayerEvent::UnitRewardDeclined { player: *player },
        // Invariant : `filter_event` n'envoie ici que les événements du thème.
        _ => unreachable!("événement hors du thème units"),
    }
}
