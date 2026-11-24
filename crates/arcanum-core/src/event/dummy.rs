//! Événements sur le joueur fantôme (spec 061) : point de non-retour (`blocks_undo`) et filtrage (`filter`).

use super::*;

/// Point de non-retour des événements du thème (voir `Event::blocks_undo`).
pub(super) fn blocks_undo(event: &Event) -> bool {
    match event {
        // Spec 061 : mise en place, tirage, cartes d'un deck caché, mélange, compétence d'une
        // pile cachée ; l'action avancée et le cristal suivent `RoundEnded`.
        Event::DummySetUp { .. }
        | Event::DummyTacticDrawn { .. }
        | Event::DummyTurnPlayed { .. }
        | Event::DummyDeckShuffled { .. }
        | Event::DummySkillRevealed { .. }
        | Event::DummyCardAdded { .. }
        | Event::DummyCrystalGained { .. } => true,
        // Spec 061 : public ; il suit la fin du tour du joueur, déjà un point de non-retour.
        Event::DummyAnnouncedEndOfRound => false,
        // Invariant : `Event::theme` n'envoie ici que les événements du thème.
        _ => unreachable!("événement hors du thème dummy"),
    }
}

/// L'événement tel que `viewer` a le droit de le voir (voir `filter_event`).
pub(super) fn filter(event: &Event, _viewer: PlayerId) -> PlayerEvent {
    match event {
        // Spec 061 (C32) : l'ordre du deck et de la pile du fantôme reste caché.
        Event::DummySetUp {
            hero,
            crystals,
            skills,
            deck,
        } => PlayerEvent::DummySetUp {
            hero: *hero,
            crystals: crystals.clone(),
            skills: sorted(skills),
            deck: sorted(deck),
        },
        Event::DummyTacticDrawn { tactic } => PlayerEvent::DummyTacticDrawn { tactic: *tactic },
        Event::DummyTurnPlayed { cards } => PlayerEvent::DummyTurnPlayed {
            cards: cards.clone(),
        },
        Event::DummyAnnouncedEndOfRound => PlayerEvent::DummyAnnouncedEndOfRound,
        Event::DummyCardAdded { card } => PlayerEvent::DummyCardAdded { card: *card },
        Event::DummyCrystalGained { color } => PlayerEvent::DummyCrystalGained { color: *color },
        Event::DummyDeckShuffled { order } => PlayerEvent::DummyDeckShuffled { count: order.len() },
        Event::DummySkillRevealed { skill } => PlayerEvent::DummySkillRevealed { skill: *skill },
        // Invariant : `filter_event` n'envoie ici que les événements du thème.
        _ => unreachable!("événement hors du thème dummy"),
    }
}
