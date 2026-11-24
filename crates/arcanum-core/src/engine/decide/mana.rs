//! Décision des actions de mana : jetons, cristaux, dé de la Source gardé.

use super::*;

/// Les événements d'une action de ce thème, déjà validée par `decide`.
pub(super) fn mana_actions(
    state: &GameState,
    player: PlayerId,
    action: &Action,
    rngs: &mut GameRngs,
) -> Result<Vec<Event>, ActionError> {
    match action {
        // Spec 056 : écrites avec chaque carte.
        Action::ChooseTokenColor { color } => Ok(vec![Event::ManaTokenGained {
            player,
            color: *color,
        }]),
        Action::PayForCrystal { color, mana } => {
            let p = state.player(player).ok_or(ActionError::NotLegal)?;
            let mut inventory = p.inventory;
            let mut events = payment_events(state, p, player, *mana, *color, &mut inventory)?;
            events.push(gained_crystal(&mut inventory, player, *color));
            Ok(events)
        }
        Action::DeclineCrystal => Ok(vec![Event::CrystalDeclined { player }]),
        Action::ChooseCrystalColor { color } => {
            // Compendium, *End of Turn* Q4 : le cristal choisi, puis la suite de la fin du tour.
            let mut events = vec![Event::CrystalGained {
                player,
                color: *color,
            }];
            // Spec 055 : le dé or de la Poche de mana, pendant le tour : le cristal seul.
            if state.player(player).is_some_and(|p| p.pouch_choices > 0) {
                return Ok(events);
            }
            let after = events.iter().fold(state.clone(), apply);
            events.extend(turn_end_rewards(&after, player, &[], rngs));
            Ok(events)
        }
        Action::KeepSourceDie => {
            let p = state.player(player).ok_or(ActionError::NotLegal)?;
            match p.opening_die {
                Some(die) if !p.opening_die_kept => Ok(vec![Event::SourceDieKept { player, die }]),
                _ => Err(ActionError::NotLegal),
            }
        }
        // L'aiguillage de `decide` n'envoie ici que les actions de ce thème.
        _ => Err(ActionError::NotLegal),
    }
}
