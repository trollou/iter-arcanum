//! `apply` : le joueur fantôme (spec 061).

use super::*;
use crate::DummyState;
use crate::dummy::color_index;

/// `apply_event` pour les événements du fantôme ; sans fantôme, rien ne change (sauf `DummySetUp`).
pub(super) fn dummy_event(state: &mut GameState, event: &Event) {
    if let Some(dummy) = state.dummy.as_mut() {
        match event {
            Event::DummyTacticDrawn { tactic } => dummy.tactic = Some(*tactic),
            Event::DummyTurnPlayed { cards } => {
                dummy.deck.retain(|card| !cards.contains(card));
                dummy.discard.extend_from_slice(cards);
                dummy.last_turn = cards.clone();
            }
            Event::DummyAnnouncedEndOfRound => dummy.announced = true,
            Event::DummyCrystalGained { color } => dummy.crystals[color_index(*color)] += 1,
            Event::DummySkillRevealed { skill } => {
                if dummy.skills.first() == Some(skill) {
                    dummy.skills.remove(0);
                    state.common_skills.push(*skill);
                }
            }
            Event::DummyDeckShuffled { order } => {
                dummy.deck = order.clone();
                dummy.discard.clear();
                dummy.last_turn.clear();
            }
            _ => {}
        }
    }
    // L'action avancée quitte l'offre pour le deck du fantôme.
    if let Event::DummyCardAdded { card } = event
        && let Some(offer) = state.offers.get_mut(&Offer::AdvancedActions)
        && let Some(index) = offer.iter().position(|held| held.deed() == Some(*card))
    {
        offer.remove(index);
        if let Some(dummy) = state.dummy.as_mut() {
            dummy.deck.push(*card);
        }
    }
    if let Event::DummySetUp {
        hero,
        crystals,
        skills,
        deck,
    } = event
    {
        let mut counts = [0; 4];
        for &color in crystals {
            counts[color_index(color)] += 1;
        }
        state.dummy = Some(DummyState {
            hero: *hero,
            deck: deck.clone(),
            discard: Vec::new(),
            crystals: counts,
            tactic: None,
            skills: skills.clone(),
            last_turn: Vec::new(),
            announced: false,
        });
    }
}
