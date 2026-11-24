//! Événements sur le tour, la manche et les tactiques : point de non-retour (`blocks_undo`) et filtrage (`filter`).

use super::*;

/// Point de non-retour des événements du thème (voir `Event::blocks_undo`).
pub(super) fn blocks_undo(event: &Event) -> bool {
    match event {
        Event::TurnEnded { .. }
        | Event::TurnStarted { .. }
        | Event::RoundEnded { .. }
        | Event::RoundStarted { .. } => true,
        // Spec 056 : publics, aucun hasard (la fin du tour est déjà un point de non-retour).
        Event::ExtraTurnStarted { .. } => false,
        // Spec 060 : la prise est publique, `TurnStarted` la suit (C3) ; le retrait suit
        // `RoundEnded`.
        Event::TacticChosen { .. } => false,
        Event::TacticsRemoved { .. } => true,
        // Spec 060 : la carte cherchée est suivie du mélange (`DeckReshuffled`).
        Event::CardSearched { .. } => false,
        // Spec 060 : publics ; le dé rendu est relancé.
        Event::DieStolen { .. } | Event::StolenDieUsed { .. } => false,
        Event::StolenDieReturned { .. } => true,
        Event::RightMomentAnnounced { .. } => false,
        // Spec 060 : un tirage au hasard ; l'arrêt suit une pioche.
        Event::LongNightUsed { .. } => true,
        Event::DrawStopped { .. } => false,
        Event::ManaSearchUsed { .. } => false,
        Event::CardsShuffledIntoDeck { .. } => false,
        // Spec 060 : une carte inconnue tirée du deck.
        Event::CardStored { .. } => true,
        Event::StoredCardsTaken { .. } => false,
        // Invariant : `Event::theme` n'envoie ici que les événements du thème.
        _ => unreachable!("événement hors du thème turn"),
    }
}

/// L'événement tel que `viewer` a le droit de le voir (voir `filter_event`).
pub(super) fn filter(event: &Event, viewer: PlayerId) -> PlayerEvent {
    match event {
        Event::TurnEnded { player } => PlayerEvent::TurnEnded { player: *player },
        Event::TurnStarted { player, turn } => PlayerEvent::TurnStarted {
            player: *player,
            turn: *turn,
        },
        Event::RoundEnded { round } => PlayerEvent::RoundEnded { round: *round },
        Event::RoundStarted { round, time_of_day } => PlayerEvent::RoundStarted {
            round: *round,
            time_of_day: *time_of_day,
        },
        Event::TacticChosen { player, tactic } => PlayerEvent::TacticChosen {
            player: *player,
            tactic: *tactic,
        },
        Event::TacticsRemoved { tactics } => PlayerEvent::TacticsRemoved {
            tactics: tactics.clone(),
        },
        Event::DieStolen { player, die } => PlayerEvent::DieStolen {
            player: *player,
            die: *die,
        },
        Event::StolenDieUsed { player } => PlayerEvent::StolenDieUsed { player: *player },
        Event::RightMomentAnnounced { player } => {
            PlayerEvent::RightMomentAnnounced { player: *player }
        }
        Event::LongNightUsed { player, cards } => {
            let mut cards = cards.clone();
            cards.sort();
            PlayerEvent::LongNightUsed {
                player: *player,
                cards,
            }
        }
        Event::ManaSearchUsed { player } => PlayerEvent::ManaSearchUsed { player: *player },
        Event::CardStored { player, .. } => PlayerEvent::CardStored { player: *player },
        Event::StoredCardsTaken { player } => PlayerEvent::StoredCardsTaken { player: *player },
        Event::CardsShuffledIntoDeck { player, cards } => PlayerEvent::CardsShuffledIntoDeck {
            player: *player,
            cards: cards.clone(),
        },
        Event::DrawStopped { player, missing } => PlayerEvent::DrawStopped {
            player: *player,
            missing: *missing,
        },
        Event::StolenDieReturned { player, color } => PlayerEvent::StolenDieReturned {
            player: *player,
            color: *color,
        },
        // La carte cherchée n'est connue que de son joueur (spec 060, C36).
        Event::CardSearched { player, card } => PlayerEvent::CardSearched {
            player: *player,
            card: (*player == viewer).then_some(*card),
        },
        Event::ExtraTurnStarted { player } => PlayerEvent::ExtraTurnStarted { player: *player },
        // Invariant : `filter_event` n'envoie ici que les événements du thème.
        _ => unreachable!("événement hors du thème turn"),
    }
}
