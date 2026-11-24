//! Événements pour `decide` : le joueur fantôme (spec 061), joué par le moteur dans la décision
//! qui précède son tour ; il ne décide jamais de rien.

use super::*;
use crate::{DummyState, RoundOrder};

/// Spec 061 (MKUE p. 4, 3.c et 4.a) : avant chaque `TurnStarted` du joueur, le tour du fantôme
/// quand il doit jouer. Les événements sont rejoués pour connaître l'état de chaque tour, quelle
/// que soit l'action qui l'ouvre (prise de la tactique, Faux départ, fin du tour…).
pub(in crate::engine) fn with_dummy_turns(
    state: &GameState,
    player: PlayerId,
    events: Vec<Event>,
) -> Vec<Event> {
    if state.dummy.is_none() {
        return events;
    }
    let mut current = state.clone();
    let mut result = Vec::with_capacity(events.len());
    for event in events {
        let opens_turn = matches!(event, Event::TurnStarted { player: p, .. } if p == player);
        if let Event::TurnStarted { player: p, turn } = event
            && p == player
            && let Some(dummy) = dummy_turn_before(&current, player, turn)
        {
            current = apply(current, &dummy);
            result.push(dummy);
        }
        current = apply(current, &event);
        result.push(event);
        // L5 (MKUE p. 6, 3.d et 4) : le fantôme a annoncé et le joueur n'a plus rien, son dernier
        // tour est passé ; `decide` complète la fin de la manche.
        if opens_turn
            && current.dummy.as_ref().is_some_and(|dummy| dummy.announced)
            && current
                .player(player)
                .is_some_and(must_announce_end_of_round)
        {
            result.push(Event::RoundEnded {
                round: current.round,
            });
        }
    }
    result
}

/// Spec 061 (MKUE p. 6, 3 ; p. 4, 4.b) : le joueur annonce la fin de la manche, le fantôme joue
/// un dernier tour (à vide si son deck l'est, sans annoncer en plus, L2), sauf une fois le
/// dernier tour de la partie annoncé (Q3) ; `decide` complète la fin de la manche.
pub(in crate::engine) fn announce_end_of_round(state: &GameState) -> Vec<Event> {
    let last_dummy_turn = state
        .dummy
        .as_ref()
        .filter(|_| state.last_turn == LastTurn::NotAnnounced)
        .map(|dummy| match dummy_turn(dummy) {
            Event::DummyAnnouncedEndOfRound => Event::DummyTurnPlayed { cards: Vec::new() },
            turn => turn,
        });
    last_dummy_turn
        .into_iter()
        .chain([Event::RoundEnded { round: state.round }])
        .collect()
}

/// Spec 061 : une décision qui finit par `RoundEnded` (dernier tour annoncé par le fantôme, ou
/// annonce du joueur) reçoit la suite de la fin de la manche, calculée sur l'état qui précède
/// `RoundEnded` : retrait des tactiques, nouvelle manche ou fin de partie.
pub(in crate::engine) fn with_round_end(
    state: &GameState,
    player: PlayerId,
    mut events: Vec<Event>,
    rngs: &mut GameRngs,
) -> Vec<Event> {
    if !matches!(events.last(), Some(Event::RoundEnded { .. })) {
        return events;
    }
    events.pop();
    let before = events.iter().fold(state.clone(), apply);
    events.extend(end_of_round(&before, player, rngs));
    events
}

/// Le tour du fantôme avant le tour `turn` du joueur : au tour 1 s'il a la plus petite tactique,
/// ensuite entre deux tours du joueur, sauf avant le tour de plus d'Au bon moment ; jamais après
/// son annonce ni une fois le dernier tour de la partie annoncé (Q3).
fn dummy_turn_before(state: &GameState, player: PlayerId, turn: u32) -> Option<Event> {
    let dummy = state.dummy.as_ref()?;
    if dummy.announced || state.last_turn != LastTurn::NotAnnounced {
        return None;
    }
    let plays = if turn == 1 {
        state.round_order() == Some(RoundOrder::DummyFirst)
    } else {
        // Spec 060 (carte *The Right Moment*) : le tour de plus suit « juste après celui-ci ».
        !state
            .player(player)
            .and_then(|p| p.tactic.as_ref())
            .is_some_and(|tactic| tactic.right_moment)
    };
    plays.then(|| dummy_turn(dummy))
}

/// Le tour du fantôme (MKUE p. 15, *When it is the Dummy player's turn*) : deck vide, il annonce
/// la fin de la manche ; sinon 3 cartes (ce que le deck a), plus une par cristal de la couleur de
/// la dernière (Compendium, *Dummy Player* S3).
pub(in crate::engine) fn dummy_turn(dummy: &DummyState) -> Event {
    if dummy.deck.is_empty() {
        return Event::DummyAnnouncedEndOfRound;
    }
    let first = dummy.deck.len().min(3);
    let extra = dummy.deck[first - 1]
        .card()
        .definition()
        .map_or(0, |definition| dummy.crystals(definition.color));
    Event::DummyTurnPlayed {
        cards: dummy
            .deck
            .iter()
            .take(first + usize::from(extra))
            .copied()
            .collect(),
    }
}

/// Spec 061 (MKUE p. 19, *Special Rules*) : la tactique du fantôme, tirée au hasard parmi celles
/// qui restent après la prise du joueur ; aucune s'il n'en reste pas (L8).
pub(in crate::engine) fn dummy_tactic(state: &GameState, rngs: &mut GameRngs) -> Option<Event> {
    state.dummy.as_ref()?;
    let tactic = *offered_tactics(state).choose(rngs.rng(RngUsage::Dummy))?;
    Some(Event::DummyTacticDrawn { tactic })
}
