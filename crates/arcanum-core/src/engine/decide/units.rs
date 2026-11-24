//! Décision des actions d'unités : activation, bannières.

use super::*;

/// Les événements d'une action de ce thème, déjà validée par `decide`.
pub(super) fn unit_actions(
    state: &GameState,
    player: PlayerId,
    action: &Action,
) -> Result<Vec<Event>, ActionError> {
    match action {
        Action::ActivateUnit { .. } => activation_events(state, player, action),
        // Spec 055 : Terreur, Gloire +1 ; Courage, l'unité rendue disponible.
        Action::UseBanner { unit, enemy } => {
            let mut events = vec![Event::BannerUsed {
                player,
                unit: *unit,
                enemy: *enemy,
            }];
            events.push(match enemy {
                Some(_) => Event::FameGained { player, fame: 1 },
                None => Event::UnitReadied {
                    player,
                    unit: *unit,
                },
            });
            Ok(events)
        }
        Action::AssignBanner { card, unit } => {
            let p = state.player(player).ok_or(ActionError::NotLegal)?;
            let mut events: Vec<Event> = p
                .units
                .iter()
                .filter(|owned| owned.card == *unit)
                .filter_map(|owned| owned.banner)
                .map(|banner| Event::BannerLost {
                    player,
                    card: banner,
                    unit: *unit,
                })
                .collect();
            events.push(Event::BannerAssigned {
                player,
                card: *card,
                unit: *unit,
            });
            Ok(events)
        }
        // L'aiguillage de `decide` n'envoie ici que les actions de ce thème.
        _ => Err(ActionError::NotLegal),
    }
}
