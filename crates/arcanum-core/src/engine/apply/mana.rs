//! `apply` : Source, dés, cristaux et jetons de mana.

use super::*;

/// `apply_event` pour les événements de ce thème ; les autres ne changent rien ici.
pub(super) fn mana_event(state: &mut GameState, event: &Event) {
    match event {
        Event::SourceRolled { dice } => {
            state.source = dice.clone();
            // Spec 023 : la Source réinitialisée n'a plus de dé pris.
            for p in &mut state.players {
                p.dice_taken.clear();
                p.dice_set.clear();
            }
        }
        Event::CrystalGained { player, color } => {
            if let Some(p) = state.player_mut(*player) {
                p.inventory.add(*color);
                // Spec 044 : seul `ChooseCrystalColor` donne un cristal pendant un choix de couleur ;
                // spec 055 : la Poche de mana d'abord.
                if p.pouch_choices > 0 {
                    p.pouch_choices -= 1;
                } else {
                    p.crystal_choices = p.crystal_choices.saturating_sub(1);
                }
                // Spec 056 : le cristal de Rituel de sang renforcé, une fois.
                if p.mana_choice.is_some_and(|choice| choice.crystal_offered) {
                    p.mana_choice = None;
                }
            }
        }
        Event::DieTaken { player, die, .. } => {
            if let Some(p) = state.player_mut(*player)
                && !p.dice_taken.contains(die)
            {
                p.dice_taken.push(*die);
                // Spec 058 : le dé de l'avantage d'Embrasure de la Source.
                if p.opening_die_due {
                    p.opening_die_due = false;
                    p.opening_die = Some(*die);
                }
            }
        }
        Event::CrystalUsed { player, color } => {
            if let Some(p) = state.player_mut(*player) {
                p.inventory.remove(*color);
                // Spec 056 : pour Maîtrise du cristal renforcée.
                p.crystals_spent.push(*color);
            }
        }
        Event::DieReturned { player, die, color } => {
            if let Some(face) = state.source.get_mut(usize::from(*die)) {
                *face = *color;
            }
            if let Some(p) = state.player_mut(*player) {
                p.dice_taken.retain(|taken| taken != die);
                p.dice_set.retain(|set| set != die);
            }
        }
        Event::ManaTokenGained { player, color } => {
            if let Some(p) = state.player_mut(*player) {
                p.mana_tokens.push(*color);
                // Spec 056 : un jeton choisi pour Rituel de sang renforcé, ou le cristal proposé
                // devenu jeton (inventaire plein).
                if let Some(choice) = p.mana_choice.as_mut() {
                    if choice.tokens_left > 0 {
                        choice.tokens_left -= 1;
                        choice.crystal_offered = choice.tokens_left == 0;
                    } else if choice.crystal_offered {
                        p.mana_choice = None;
                    }
                }
            }
        }
        Event::ManaTokenUsed { player, color } => {
            if let Some(p) = state.player_mut(*player)
                && let Some(index) = p.mana_tokens.iter().position(|token| token == color)
            {
                p.mana_tokens.remove(index);
                // Spec 073 : la zone de jeu montre aussi les jetons dépensés.
                p.spent_mana_tokens.push(*color);
            }
        }
        Event::DieSet { player, die, color } => {
            if let Some(face) = state.source.get_mut(usize::from(*die)) {
                *face = ManaColor::Basic(*color);
            }
            if let Some(p) = state.player_mut(*player)
                && !p.dice_set.contains(die)
            {
                p.dice_set.push(*die);
            }
        }
        // Spec 055 : le mana illimité ne retire rien.
        Event::EndlessManaUsed { .. } => {}
        Event::RewardDieRolled { player, color } => {
            if let Some(p) = state.player_mut(*player) {
                if p.combat.is_some() {
                    // Le dé du donjon, lancé pendant le tour (carte du donjon) : or ou noir, un
                    // sort ; sinon, un artefact.
                    if matches!(color, ManaColor::Gold | ManaColor::Black) {
                        p.spell_rewards += 1;
                    } else {
                        p.artifact_rewards += 1;
                    }
                    p.reward_die = Some(*color);
                } else {
                    // Un cristal aléatoire, après `TurnEnded` : or, un choix de couleur s'il en
                    // reste une non pleine.
                    p.crystal_rewards = p.crystal_rewards.saturating_sub(1);
                    let open = BasicColor::ALL
                        .into_iter()
                        .any(|color| !p.inventory.is_full(color));
                    if *color == ManaColor::Gold && open {
                        p.crystal_choices += 1;
                    }
                }
            }
        }
        Event::SourceDieKept { player, .. } => {
            if let Some(p) = state.player_mut(*player) {
                p.opening_die_kept = true;
            }
        }
        Event::CrystalDeclined { player } => {
            if let Some(p) = state.player_mut(*player) {
                p.mana_choice = None;
            }
        }
        Event::CrystalsReturned { player, colors } => {
            if let Some(p) = state.player_mut(*player) {
                for &color in colors {
                    p.inventory.add(color);
                }
            }
        }
        Event::DieKept { player, die } => {
            if let Some(p) = state.player_mut(*player) {
                p.kept_die = Some(*die);
            }
        }
        Event::DieRerolled { die, color } => {
            if let Some(face) = state.source.get_mut(usize::from(*die)) {
                *face = *color;
            }
        }
        _ => {}
    }
}
