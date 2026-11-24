//! Décision des actions de cartes : jouer, poser sur le deck, apprendre, soigner.

use super::*;

/// Les événements d'une action de ce thème, déjà validée par `decide`.
pub(super) fn card_actions(
    state: &GameState,
    player: PlayerId,
    action: &Action,
    rngs: &mut GameRngs,
) -> Result<Vec<Event>, ActionError> {
    match action {
        Action::PlayCard { card, play } => play_card_events(state, player, card, play, rngs),
        // Spec 042 : la carte, puis l'unité rendue disponible.
        Action::PlayCardOnUnit { card, play, target } => {
            let mut events = play_card_events(state, player, card, play, rngs)?;
            events.push(Event::UnitReadied {
                player,
                unit: *target,
            });
            Ok(events)
        }
        Action::PlaceOnDeck { top, order } => Ok(vec![Event::CardsPlacedOnDeck {
            player,
            top: *top,
            order: order.clone(),
        }]),
        Action::LearnAdvancedAction { card } => {
            let p = state.player(player).ok_or(ActionError::NotLegal)?;
            let (cost, to) = learning_discount(p).ok_or(ActionError::NotLegal)?;
            let offer = Offer::AdvancedActions;
            let mut events = vec![
                Event::AdvancedActionLearned {
                    player,
                    card: *card,
                    cost,
                },
                Event::OfferCardGained {
                    player,
                    offer,
                    card: CommonCard::Deed(*card),
                    to,
                },
            ];
            let top = state.common_deck(CommonDeck::AdvancedActions).first();
            events.extend(top.map(|&card| Event::OfferCardDealt { offer, card }));
            Ok(events)
        }
        Action::SpendHealing { on } => {
            if !healing_actions(state, player).contains(action) {
                return Err(ActionError::NotLegal);
            }
            let p = state.player(player).ok_or(ActionError::NotLegal)?;
            Ok(vec![match on {
                // MKUE p. 11, 3.c : la première Blessure de la main (identiques).
                HealingTarget::Wound => Event::WoundsHealed {
                    player,
                    wounds: p
                        .hand
                        .iter()
                        .copied()
                        .filter(|card| card.card().is_wound())
                        .take(1)
                        .collect(),
                },
                // FAQ p3 : une Blessure à la fois, pour le niveau de l'unité.
                HealingTarget::Unit(unit) => Event::UnitHealed {
                    player,
                    unit: *unit,
                    cost: unit.unit.definition().level,
                },
            }])
        }
        // L'aiguillage de `decide` n'envoie ici que les actions de ce thème.
        _ => Err(ActionError::NotLegal),
    }
}
