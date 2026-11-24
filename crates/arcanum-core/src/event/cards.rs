//! Événements sur les cartes du deck, de la main et de la défausse : point de non-retour (`blocks_undo`) et filtrage (`filter`).

use super::*;

/// Point de non-retour des événements du thème (voir `Event::blocks_undo`).
pub(super) fn blocks_undo(event: &Event) -> bool {
    match event {
        Event::DeckShuffled { .. }
        | Event::CardDrawn { .. } => true,
        Event::CardPlayed { .. }
        | Event::CardDiscarded { .. }
        | Event::CardReturnedToHand { .. }
        // Spec 034 : rien de caché, aucun hasard.
        | Event::WoundsHealed { .. }
        // Spec 044 : public, aucun hasard ; les jetons tirés suivent.
        | Event::WoundThrownAway { .. }
        // Spec 052 : publics, aucun hasard (la fin du tour est déjà un point de non-retour).
        | Event::SteadyTempoKept { .. }
        // Spec 056 : publics, aucun hasard (la fin du tour est déjà un point de non-retour).
        | Event::CardDestroyed { .. }
        | Event::DrawBonusUsed { .. }
        | Event::CardsPlacedOnDeck { .. }
        | Event::CardsReturnedToHand { .. }
        | Event::CardSetAside { .. } => false,
        // Spec 057 : un tirage au hasard.
        Event::DiscardPicked { .. } => true,
        // Spec 060 : un mélange (il suit la carte cherchée, `CardSearched`).
        Event::DeckReshuffled { .. } => true,
        // Invariant : `Event::theme` n'envoie ici que les événements du thème.
        _ => unreachable!("événement hors du thème cards"),
    }
}

/// L'événement tel que `viewer` a le droit de le voir (voir `filter_event`).
pub(super) fn filter(event: &Event, viewer: PlayerId) -> PlayerEvent {
    match event {
        Event::DeckShuffled { player, order } => PlayerEvent::DeckShuffled {
            player: *player,
            count: order.len(),
        },
        Event::CardDrawn { player, card } => PlayerEvent::CardDrawn {
            player: *player,
            card: (*player == viewer).then_some(*card),
        },
        Event::CardPlayed {
            player,
            card,
            play,
            effect,
        } => PlayerEvent::CardPlayed {
            player: *player,
            card: *card,
            play: *play,
            effect: *effect,
        },
        Event::CardDiscarded { player, card } => PlayerEvent::CardDiscarded {
            player: *player,
            card: *card,
        },
        Event::CardReturnedToHand { player, card } => PlayerEvent::CardReturnedToHand {
            player: *player,
            card: *card,
        },
        // Spec 034 : les Blessures jetées et les cartes défaussées sont publiques.
        Event::WoundsHealed { player, wounds } => PlayerEvent::WoundsHealed {
            player: *player,
            wounds: wounds.clone(),
        },
        Event::DeckReshuffled { player, order } => PlayerEvent::DeckReshuffled {
            player: *player,
            count: order.len(),
        },
        // Spec 056 : publics ; la carte détruite était dans la main ou la zone de jeu du joueur.
        Event::CardDestroyed { player, card, from } => PlayerEvent::CardDestroyed {
            player: *player,
            card: *card,
            from: *from,
        },
        Event::CardsReturnedToHand { player, cards } => PlayerEvent::CardsReturnedToHand {
            player: *player,
            cards: cards.clone(),
        },
        Event::CardSetAside { player, card } => PlayerEvent::CardSetAside {
            player: *player,
            card: *card,
        },
        Event::DiscardPicked { player, cards } => PlayerEvent::DiscardPicked {
            player: *player,
            cards: cards.clone(),
        },
        Event::CardsPlacedOnDeck { player, top, order } => PlayerEvent::CardsPlacedOnDeck {
            player: *player,
            top: *top,
            order: order.clone(),
        },
        Event::DrawBonusUsed { player } => PlayerEvent::DrawBonusUsed { player: *player },
        Event::SteadyTempoKept { player, card, top } => PlayerEvent::SteadyTempoKept {
            player: *player,
            card: *card,
            top: *top,
        },
        Event::WoundThrownAway { player, from } => PlayerEvent::WoundThrownAway {
            player: *player,
            from: *from,
        },
        // Invariant : `filter_event` n'envoie ici que les événements du thème.
        _ => unreachable!("événement hors du thème cards"),
    }
}
