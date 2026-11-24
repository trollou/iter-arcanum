//! Événements sur la renommée, les niveaux, la réputation et les compétences : point de non-retour (`blocks_undo`) et filtrage (`filter`).

use super::*;

/// Point de non-retour des événements du thème (voir `Event::blocks_undo`).
pub(super) fn blocks_undo(event: &Event) -> bool {
    match event {
        // Spec 051 : hasard de la mise en place.
        Event::SkillPileShuffled { .. }
        // Spec 051 : deux compétences révélées.
        | Event::SkillsRevealed { .. } => true,
        // Spec 035 : public, et `TurnEnded`, qui le précède, est déjà un point de non-retour.
        Event::LevelGained { .. }
        // Spec 063 : publics, et `TileRevealed`, qui les précède, est déjà un point de
        // non-retour.
        | Event::FameGained { .. }
        // Spec 051 : publics, aucun hasard.
        | Event::SkillGained { .. }
        | Event::SkillTakenFromCommonOffer { .. }
        | Event::SkillToCommonOffer { .. }
        | Event::SkillUsed { .. }
        // Spec 058 : publics, aucun hasard.
        | Event::SkillPlacedInCenter { .. }
        | Event::CenterAdvantageUsed { .. }
        | Event::CenterSkillReturned { .. }
        // Spec 040 : public, aucun hasard.
        | Event::ReputationChanged { .. } => false,
        // Invariant : `Event::theme` n'envoie ici que les événements du thème.
        _ => unreachable!("événement hors du thème progression"),
    }
}

/// L'événement tel que `viewer` a le droit de le voir (voir `filter_event`).
pub(super) fn filter(event: &Event, _viewer: PlayerId) -> PlayerEvent {
    match event {
        Event::LevelGained { player, level } => PlayerEvent::LevelGained {
            player: *player,
            level: *level,
        },
        // Spec 063 : publics.
        Event::FameGained { player, fame } => PlayerEvent::FameGained {
            player: *player,
            fame: *fame,
        },
        Event::SkillsRevealed { player, skills } => PlayerEvent::SkillsRevealed {
            player: *player,
            skills: skills.clone(),
        },
        Event::SkillGained { player, skill } => PlayerEvent::SkillGained {
            player: *player,
            skill: *skill,
        },
        Event::SkillTakenFromCommonOffer { player, skill } => {
            PlayerEvent::SkillTakenFromCommonOffer {
                player: *player,
                skill: *skill,
            }
        }
        Event::SkillToCommonOffer { skill } => PlayerEvent::SkillToCommonOffer { skill: *skill },
        Event::SkillUsed {
            player,
            skill,
            option,
            effect,
        } => PlayerEvent::SkillUsed {
            player: *player,
            skill: *skill,
            option: *option,
            effect: *effect,
        },
        Event::SkillPlacedInCenter {
            player,
            skill,
            color,
        } => PlayerEvent::SkillPlacedInCenter {
            player: *player,
            skill: *skill,
            color: *color,
        },
        Event::CenterAdvantageUsed { player, skill } => PlayerEvent::CenterAdvantageUsed {
            player: *player,
            skill: *skill,
        },
        Event::CenterSkillReturned { player, skill } => PlayerEvent::CenterSkillReturned {
            player: *player,
            skill: *skill,
        },
        Event::ReputationChanged { player, change } => PlayerEvent::ReputationChanged {
            player: *player,
            change: *change,
        },
        Event::SkillPileShuffled { player, order } => PlayerEvent::SkillPileShuffled {
            player: *player,
            count: order.len(),
        },
        // Invariant : `filter_event` n'envoie ici que les événements du thème.
        _ => unreachable!("événement hors du thème progression"),
    }
}
