//! Événements sur la Source, les dés, les cristaux et les jetons de mana : point de non-retour (`blocks_undo`) et filtrage (`filter`).

use super::*;

/// Point de non-retour des événements du thème (voir `Event::blocks_undo`).
pub(super) fn blocks_undo(event: &Event) -> bool {
    match event {
        Event::SourceRolled { .. }
        | Event::DieReturned { .. }
        // Spec 044 : un dé est lancé.
        | Event::RewardDieRolled { .. }
        // Spec 056 : un dé est lancé (MKUE p. 5, *Reverting* 2).
        | Event::DieRerolled { .. } => true,
        Event::CrystalGained { .. }
        | Event::DieTaken { .. }
        | Event::CrystalUsed { .. }
        | Event::ManaTokenGained { .. }
        | Event::ManaTokenUsed { .. }
        | Event::DieSet { .. }
        // Spec 058 : publics, aucun hasard.
        | Event::SourceDieKept { .. }
        // Spec 055 : publics, aucun hasard.
        | Event::EndlessManaUsed { .. }
        // Spec 056 : publics, aucun hasard (la fin du tour est déjà un point de non-retour).
        | Event::CrystalDeclined { .. }
        | Event::CrystalsReturned { .. }
        | Event::DieKept { .. } => false,
        // Invariant : `Event::theme` n'envoie ici que les événements du thème.
        _ => unreachable!("événement hors du thème mana"),
    }
}

/// L'événement tel que `viewer` a le droit de le voir (voir `filter_event`).
pub(super) fn filter(event: &Event, _viewer: PlayerId) -> PlayerEvent {
    match event {
        Event::SourceRolled { dice } => PlayerEvent::SourceRolled { dice: dice.clone() },
        Event::CrystalGained { player, color } => PlayerEvent::CrystalGained {
            player: *player,
            color: *color,
        },
        Event::DieTaken { player, die, color } => PlayerEvent::DieTaken {
            player: *player,
            die: *die,
            color: *color,
        },
        Event::CrystalUsed { player, color } => PlayerEvent::CrystalUsed {
            player: *player,
            color: *color,
        },
        Event::DieReturned { player, die, color } => PlayerEvent::DieReturned {
            player: *player,
            die: *die,
            color: *color,
        },
        Event::ManaTokenGained { player, color } => PlayerEvent::ManaTokenGained {
            player: *player,
            color: *color,
        },
        Event::ManaTokenUsed { player, color } => PlayerEvent::ManaTokenUsed {
            player: *player,
            color: *color,
        },
        Event::DieSet { player, die, color } => PlayerEvent::DieSet {
            player: *player,
            die: *die,
            color: *color,
        },
        Event::SourceDieKept { player, die } => PlayerEvent::SourceDieKept {
            player: *player,
            die: *die,
        },
        Event::DieRerolled { die, color } => PlayerEvent::DieRerolled {
            die: *die,
            color: *color,
        },
        Event::CrystalDeclined { player } => PlayerEvent::CrystalDeclined { player: *player },
        Event::CrystalsReturned { player, colors } => PlayerEvent::CrystalsReturned {
            player: *player,
            colors: colors.clone(),
        },
        Event::DieKept { player, die } => PlayerEvent::DieKept {
            player: *player,
            die: *die,
        },
        Event::EndlessManaUsed { player, color } => PlayerEvent::EndlessManaUsed {
            player: *player,
            color: *color,
        },
        Event::RewardDieRolled { player, color } => PlayerEvent::RewardDieRolled {
            player: *player,
            color: *color,
        },
        // Invariant : `filter_event` n'envoie ici que les événements du thème.
        _ => unreachable!("événement hors du thème mana"),
    }
}
