//! Événements sur la partie, la carte, le déplacement et les ruines : point de non-retour (`blocks_undo`) et filtrage (`filter`).

use super::*;

/// Point de non-retour des événements du thème (voir `Event::blocks_undo`).
pub(super) fn blocks_undo(event: &Event) -> bool {
    match event {
        Event::GameStarted { .. }
        | Event::TilePlaced { .. }
        | Event::TileDeckFormed { .. }
        | Event::HeroFigurePlaced { .. }
        | Event::TileRevealed { .. }
        // Spec 030 : hasard de la mise en place, jeton révélé.
        | Event::EnemyPileShuffled { .. }
        | Event::EnemyTokenPlaced { .. }
        // Spec 043 : le tirage du défenseur a eu lieu.
        | Event::DefenderPlaced { .. }
        // Spec 062 : la cité arrive avec sa tuile.
        | Event::CityPlaced { .. }
        | Event::EnemyTokenRevealed { .. }
        // Spec 031 : hasard du remélange d'une pile.
        | Event::EnemyPileReshuffled { .. }
        // Spec 045 : hasard de la mise en place, jeton posé (révélé de jour).
        | Event::RuinPileShuffled { .. }
        | Event::RuinTokenPlaced { .. }
        | Event::RuinTokenRevealed { .. } => true,
        Event::HeroMoved { .. }
        | Event::Explored { .. }
        // Spec 063 : publics, et `TileRevealed`, qui les précède, est déjà un point de
        // non-retour.
        | Event::LastTurnAnnounced { .. }
        // Spec 041 : publics, aucun hasard.
        | Event::EnemyTokenDiscarded { .. }
        // Spec 043 : publics, aucun hasard.
        | Event::HeroWithdrew { .. }
        // Spec 052 : publics, aucun hasard (la fin du tour est déjà un point de non-retour).
        | Event::ForcedWithdrawal { .. } => false,
        // Spec 063 : rien ne se joue après, rien à annuler.
        Event::GameEnded { .. } => true,
        // Invariant : `Event::theme` n'envoie ici que les événements du thème.
        _ => unreachable!("événement hors du thème map"),
    }
}

/// L'événement tel que `viewer` a le droit de le voir (voir `filter_event`).
pub(super) fn filter(event: &Event, _viewer: PlayerId) -> PlayerEvent {
    match event {
        Event::GameStarted { config } => PlayerEvent::GameStarted {
            config: config.clone(),
        },
        Event::TilePlaced { tile, position } => PlayerEvent::TilePlaced {
            tile: *tile,
            position: *position,
        },
        Event::TileDeckFormed { order } => PlayerEvent::TileDeckFormed {
            counts: counts_of(order),
        },
        Event::TileRevealed { tile, position } => PlayerEvent::TileRevealed {
            tile: *tile,
            position: *position,
        },
        Event::HeroFigurePlaced { player, hex } => PlayerEvent::HeroFigurePlaced {
            player: *player,
            hex: *hex,
        },
        Event::HeroMoved {
            player,
            from,
            to,
            cost,
        } => PlayerEvent::HeroMoved {
            player: *player,
            from: *from,
            to: *to,
            cost: *cost,
        },
        Event::Explored {
            player,
            position,
            cost,
        } => PlayerEvent::Explored {
            player: *player,
            position: *position,
            cost: *cost,
        },
        Event::EnemyPileShuffled { color, order } => PlayerEvent::EnemyPileShuffled {
            color: *color,
            count: order.len(),
        },
        Event::EnemyPileReshuffled { color, order } => PlayerEvent::EnemyPileReshuffled {
            color: *color,
            count: order.len(),
        },
        Event::EnemyTokenPlaced { token, hex } => PlayerEvent::EnemyTokenPlaced {
            token: *token,
            hex: *hex,
        },
        Event::DefenderPlaced { token, hex } => PlayerEvent::DefenderPlaced {
            color: token.color(),
            hex: *hex,
        },
        Event::CityPlaced { hex, color, level } => PlayerEvent::CityPlaced {
            hex: *hex,
            color: *color,
            level: *level,
        },
        Event::EnemyTokenRevealed { hex, token } => PlayerEvent::EnemyTokenRevealed {
            hex: *hex,
            token: *token,
        },
        Event::LastTurnAnnounced { player } => PlayerEvent::LastTurnAnnounced { player: *player },
        Event::GameEnded { reason } => PlayerEvent::GameEnded { reason: *reason },
        Event::EnemyTokenDiscarded { hex, token } => PlayerEvent::EnemyTokenDiscarded {
            hex: *hex,
            token: *token,
        },
        Event::HeroWithdrew { player, from, to } => PlayerEvent::HeroWithdrew {
            player: *player,
            from: *from,
            to: *to,
        },
        Event::ForcedWithdrawal {
            player,
            path,
            wounds,
        } => PlayerEvent::ForcedWithdrawal {
            player: *player,
            path: path.clone(),
            wounds: wounds.clone(),
        },
        Event::RuinPileShuffled { order } => PlayerEvent::RuinPileShuffled { count: order.len() },
        Event::RuinTokenPlaced {
            token,
            hex,
            face_up,
        } => PlayerEvent::RuinTokenPlaced {
            token: face_up.then_some(*token),
            hex: *hex,
            face_up: *face_up,
        },
        Event::RuinTokenRevealed { hex, token } => PlayerEvent::RuinTokenRevealed {
            hex: *hex,
            token: *token,
        },
        // Invariant : `filter_event` n'envoie ici que les événements du thème.
        _ => unreachable!("événement hors du thème map"),
    }
}
