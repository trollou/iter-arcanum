//! Événements sur les sites, l'interaction, les offres et les artefacts : point de non-retour (`blocks_undo`) et filtrage (`filter`).

use super::*;

/// Point de non-retour des événements du thème (voir `Event::blocks_undo`).
pub(super) fn blocks_undo(event: &Event) -> bool {
    match event {
        // Spec 050 : hasard de la mise en place, carte révélée.
        Event::CommonDeckShuffled { .. }
        | Event::OfferCardDealt { .. }
        // Spec 041 : les artefacts tirés sont révélés.
        | Event::ArtifactsDrawn { .. }
        // Spec 055 : un dé est lancé.
        | Event::ArtifactDieRolled { .. }
        // Spec 062 : une carte cachée est révélée.
        | Event::AdvancedActionDrawnBlind { .. }
        | Event::EliteUnitAdded { .. } => true,
        // Spec 050 : publics ; la carte gagnée était visible dans l'offre.
        Event::OfferCardReturned { .. }
        | Event::OfferCardGained { .. }
        // Spec 040 : public, aucun hasard.
        | Event::InteractionStarted { .. }
        // Spec 041 : publics, aucun hasard.
        | Event::HealingBought { .. }
        | Event::AdvancedActionBought { .. }
        | Event::SpellBought { .. }
        | Event::MonasteryBurnStarted { .. }
        | Event::MonasteryBurned { .. }
        // Spec 043 : publics, aucun hasard.
        | Event::SiteConquered { .. }
        // Spec 062 : publics, aucun hasard.
        | Event::CityShieldPlaced { .. }
        | Event::CityConquered { .. }
        | Event::ArtifactBought { .. }
        | Event::ArtifactTaken { .. }
        | Event::ArtifactReturned { .. }
        | Event::VillagePlundered { .. }
        // Spec 044 : public, aucun hasard ; les jetons tirés suivent.
        | Event::AdventureSiteEntered { .. }
        // Spec 045 : public, aucun hasard.
        | Event::AltarTributePaid { .. }
        // Spec 056 : publics, aucun hasard (la fin du tour est déjà un point de non-retour).
        | Event::AdvancedActionLearned { .. } => false,
        // Invariant : `Event::theme` n'envoie ici que les événements du thème.
        _ => unreachable!("événement hors du thème sites"),
    }
}

/// L'événement tel que `viewer` a le droit de le voir (voir `filter_event`).
pub(super) fn filter(event: &Event, _viewer: PlayerId) -> PlayerEvent {
    match event {
        Event::CommonDeckShuffled { deck, order } => PlayerEvent::CommonDeckShuffled {
            deck: *deck,
            count: order.len(),
        },
        Event::OfferCardDealt { offer, card } => PlayerEvent::OfferCardDealt {
            offer: *offer,
            card: *card,
        },
        Event::OfferCardReturned { offer, card } => PlayerEvent::OfferCardReturned {
            offer: *offer,
            card: *card,
        },
        Event::OfferCardGained {
            player,
            offer,
            card,
            to,
        } => PlayerEvent::OfferCardGained {
            player: *player,
            offer: *offer,
            card: *card,
            to: *to,
        },
        Event::AdvancedActionLearned { player, card, cost } => PlayerEvent::AdvancedActionLearned {
            player: *player,
            card: *card,
            cost: *cost,
        },
        Event::InteractionStarted {
            player,
            site,
            bonus,
            city_shields,
        } => PlayerEvent::InteractionStarted {
            player: *player,
            site: *site,
            bonus: *bonus,
            city_shields: *city_shields,
        },
        Event::AdvancedActionDrawnBlind { player, card, cost } => {
            PlayerEvent::AdvancedActionDrawnBlind {
                player: *player,
                card: *card,
                cost: *cost,
            }
        }
        Event::ArtifactBought { player, cost } => PlayerEvent::ArtifactBought {
            player: *player,
            cost: *cost,
        },
        Event::EliteUnitAdded { player, card, cost } => PlayerEvent::EliteUnitAdded {
            player: *player,
            card: *card,
            cost: *cost,
        },
        Event::ArtifactDieRolled {
            player,
            card,
            color,
        } => PlayerEvent::ArtifactDieRolled {
            player: *player,
            card: *card,
            color: *color,
        },
        Event::HealingBought { player, cost } => PlayerEvent::HealingBought {
            player: *player,
            cost: *cost,
        },
        Event::AdvancedActionBought { player, card, cost } => PlayerEvent::AdvancedActionBought {
            player: *player,
            card: *card,
            cost: *cost,
        },
        Event::SpellBought { player, card, cost } => PlayerEvent::SpellBought {
            player: *player,
            card: *card,
            cost: *cost,
        },
        Event::MonasteryBurnStarted { player, hex } => PlayerEvent::MonasteryBurnStarted {
            player: *player,
            hex: *hex,
        },
        Event::MonasteryBurned { player, hex } => PlayerEvent::MonasteryBurned {
            player: *player,
            hex: *hex,
        },
        Event::SiteConquered { player, hex } => PlayerEvent::SiteConquered {
            player: *player,
            hex: *hex,
        },
        Event::CityShieldPlaced { player, hex, enemy } => PlayerEvent::CityShieldPlaced {
            player: *player,
            hex: *hex,
            enemy: *enemy,
        },
        Event::CityConquered { hex, leader } => PlayerEvent::CityConquered {
            hex: *hex,
            leader: *leader,
        },
        // Solo : le joueur voit ses artefacts tirés (multijoueur hors périmètre).
        Event::ArtifactsDrawn { player, cards } => PlayerEvent::ArtifactsDrawn {
            player: *player,
            cards: cards.clone(),
        },
        Event::ArtifactTaken { player, card } => PlayerEvent::ArtifactTaken {
            player: *player,
            card: *card,
        },
        Event::ArtifactReturned { card } => PlayerEvent::ArtifactReturned { card: *card },
        Event::VillagePlundered { player } => PlayerEvent::VillagePlundered { player: *player },
        Event::AdventureSiteEntered { player, hex } => PlayerEvent::AdventureSiteEntered {
            player: *player,
            hex: *hex,
        },
        Event::AltarTributePaid { player, hex } => PlayerEvent::AltarTributePaid {
            player: *player,
            hex: *hex,
        },
        // Invariant : `filter_event` n'envoie ici que les événements du thème.
        _ => unreachable!("événement hors du thème sites"),
    }
}
