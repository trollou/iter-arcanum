//! `apply` : partie, carte, déplacement, jetons posés sur la carte et ruines.

use super::*;

/// `apply_event` pour les événements de ce thème ; les autres ne changent rien ici.
pub(super) fn map_event(state: &mut GameState, event: &Event) {
    match event {
        Event::GameStarted { config } => {
            *state = GameState {
                players: vec![PlayerState {
                    hero: Some(config.hero),
                    // MKUE p. 3 : le jeton de niveau 1-2 (spec 035).
                    level: 1,
                    ..PlayerState::default()
                }],
                scenario: Some(config.scenario),
                turn: 1,
                round: 1,
                ..GameState::default()
            };
            // Spec 060 : la première manche commence par le choix des tactiques.
            state.tactic_choice = !offered_tactics(state).is_empty();
        }
        Event::TilePlaced { tile, position } => {
            // Une pose refusée (emplacement occupé, tuile déjà posée) ne change rien.
            let _ = state.map.place(*tile, *position);
        }
        Event::TileDeckFormed { order } => {
            state.tile_deck = TileDeck::new(order.clone());
        }
        Event::TileRevealed { tile, position } => {
            if state.map.place(*tile, *position).is_ok() {
                state.tile_deck.remove(*tile);
            }
        }
        Event::HeroFigurePlaced { player, hex } => {
            if let Some(p) = state.player_mut(*player) {
                p.figure = Some(*hex);
                p.turn_path = vec![*hex];
            }
        }
        Event::HeroMoved {
            player,
            from,
            to,
            cost,
        } => {
            if let Some(p) = state.player_mut(*player)
                && p.figure == Some(*from)
                && let Some(left) = p.move_points.checked_sub(u32::from(*cost))
            {
                p.figure = Some(*to);
                p.move_points = left;
                p.moved_or_fought = true;
                p.turn_path.push(*to);
            }
        }
        Event::Explored { player, cost, .. } => {
            if let Some(p) = state.player_mut(*player)
                && let Some(left) = p.move_points.checked_sub(u32::from(*cost))
            {
                p.move_points = left;
                p.moved_or_fought = true;
            }
        }
        Event::EnemyPileShuffled { color, order } => {
            state.enemy_piles.insert(*color, order.clone());
        }
        Event::EnemyPileReshuffled { color, order } => {
            state.enemy_piles.insert(*color, order.clone());
            state.enemy_discards.remove(color);
        }
        Event::EnemyTokenPlaced { token, hex } => {
            // Seul le dessus de sa pile se pose ; il s'ajoute à ceux de sa case (spec 044, Q1).
            if let Some(pile) = state.enemy_piles.get_mut(&token.color())
                && pile.first() == Some(token)
            {
                pile.remove(0);
                state.enemy_tokens.entry(*hex).or_default().push(*token);
            }
        }
        // Spec 043 : comme un jeton d'ennemi errant, mais face cachée.
        // Spec 062 : une cité en reçoit plusieurs, tous face cachée.
        Event::DefenderPlaced { token, hex } => {
            let free = !state.has_enemy_token(*hex) || state.face_down.contains(hex);
            if free
                && let Some(pile) = state.enemy_piles.get_mut(&token.color())
                && pile.first() == Some(token)
            {
                pile.remove(0);
                state.enemy_tokens.entry(*hex).or_default().push(*token);
                state.face_down.insert(*hex);
            }
        }
        // Spec 062 : la cité entre en jeu, sans Bouclier.
        Event::CityPlaced { hex, color, level } => {
            state
                .cities
                .entry(*hex)
                .or_insert_with(|| crate::CityState::new(*color, *level));
        }
        Event::EnemyTokenRevealed { hex, token } => {
            if state.enemy_tokens_at(*hex).contains(token) {
                state.face_down.remove(hex);
            }
        }
        Event::LastTurnAnnounced { .. } => {
            if state.last_turn == LastTurn::NotAnnounced {
                state.last_turn = LastTurn::Next;
            }
        }
        Event::GameEnded { reason } => {
            state.game_end = Some(*reason);
        }
        // Sans coût ni provocation : la figurine revient sur sa case (MKUE p. 10, 3.d).
        Event::HeroWithdrew { player, from, to } => {
            if let Some(p) = state.player_mut(*player)
                && p.figure == Some(*from)
            {
                p.figure = Some(*to);
                // Spec 052 : la figurine est revenue d'où elle venait, le site quitte le trajet.
                if p.turn_path.last() == Some(from) {
                    p.turn_path.pop();
                }
            }
        }
        Event::EnemyTokenDiscarded { hex, token } => state.discard_enemy_token(*hex, *token),
        // Spec 052 (MKUE p. 11, 2) : retour sur ses pas, les Blessures en main, sans assommer
        // (hors combat) ni provoquer.
        Event::ForcedWithdrawal {
            player,
            path,
            wounds,
        } => {
            if let Some(p) = state.player_mut(*player)
                && let Some(&to) = path.last()
            {
                p.figure = Some(to);
                let kept = p.turn_path.len().saturating_sub(path.len());
                p.turn_path.truncate(kept.max(1));
                p.hand.extend(wounds.iter().copied());
                p.wounds_given = p
                    .wounds_given
                    .saturating_add(u16::try_from(wounds.len()).unwrap_or(u16::MAX));
            }
        }
        Event::RuinPileShuffled { order } => {
            state.ruin_pile = order.clone();
        }
        // Seul le dessus de la pile se pose (spec 045).
        Event::RuinTokenPlaced {
            token,
            hex,
            face_up,
        } => {
            if state.ruin_pile.first() == Some(token) {
                state.ruin_pile.remove(0);
                state.ruins.insert(
                    *hex,
                    PlacedRuin {
                        token: *token,
                        face_up: *face_up,
                    },
                );
            }
        }
        Event::RuinTokenRevealed { hex, token } => {
            if let Some(placed) = state.ruins.get_mut(hex)
                && placed.token == *token
            {
                placed.face_up = true;
                // Révélée sous la figurine qui vient d'y entrer (pas par une nouvelle manche).
                for p in &mut state.players {
                    if p.figure == Some(*hex) && p.moved_or_fought {
                        p.revealed_ruins.push(*hex);
                    }
                }
            }
        }
        _ => {}
    }
}
