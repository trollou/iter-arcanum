//! Décision des actions sur la carte : pas, trajet, exploration.

use super::*;

/// Les événements d'une action de ce thème, déjà validée par `decide`.
pub(super) fn map_actions(
    state: &GameState,
    player: PlayerId,
    action: &Action,
    rngs: &mut GameRngs,
) -> Result<Vec<Event>, ActionError> {
    match action {
        Action::MoveHero { to } => decide_move_hero(state, player, to),
        Action::FollowRoute { to } => {
            // MKUE p. 7, 3 : case par case, chaque pas paie la case où l'on entre (spec 036).
            let from = state.figure(player).ok_or(ActionError::NotLegal)?;
            let route = affordable_routes(state, player)
                .into_iter()
                .find(|route| route.to == *to)
                .ok_or(ActionError::NotLegal)?;
            let mut previous = from;
            let mut events = Vec::new();
            let assault = is_assault_target(state, route.to);
            let rules = state.move_rules(player);
            for &hex in &route.steps {
                let space = state.map.space(hex).ok_or(ActionError::NotLegal)?;
                let cost = match move_cost(space, &rules) {
                    MoveCost::Points(cost) => cost,
                    MoveCost::Fortified(cost)
                        if (assault && hex == route.to) || state.is_conquered(hex) =>
                    {
                        cost
                    }
                    MoveCost::Fortified(_) | MoveCost::Inaccessible => {
                        return Err(ActionError::NotLegal);
                    }
                };
                events.push(Event::HeroMoved {
                    player,
                    from: previous,
                    to: hex,
                    cost,
                });
                // Spec 045 (Q2) : le trajet s'arrête sur la ruine dont il retourne le jeton.
                if let Some(reveal) = ruin_reveal(state, hex) {
                    events.push(reveal);
                    return Ok(events);
                }
                // Spec 043 : chaque pas retourne les défenseurs voisins, de jour ; le trajet
                // s'arrête quand il retourne celui du site visé (Compendium, *Movement* S13).
                let revealed = reveals_next_to(state, reveal_time(state, player), hex, &events);
                let stop = revealed.iter().any(
                    |event| matches!(event, Event::EnemyTokenRevealed { hex, .. } if *hex == route.to),
                );
                events.extend(revealed);
                if stop {
                    return Ok(events);
                }
                if assault && hex == route.to {
                    events.extend(assault_events(state, player, previous, hex));
                }
                previous = hex;
            }
            Ok(events)
        }
        Action::Explore { position } => decide_explore(state, player, position, rngs),
        // L'aiguillage de `decide` n'envoie ici que les actions de ce thème.
        _ => Err(ActionError::NotLegal),
    }
}

/// `Action::MoveHero` : un pas vers une case voisine.
fn decide_move_hero(
    state: &GameState,
    player: PlayerId,
    to: &Hex,
) -> Result<Vec<Event>, ActionError> {
    let from = state.figure(player).ok_or(ActionError::NotLegal)?;
    let (_, cost) = affordable_moves(state, player)
        .into_iter()
        .find(|(hex, _)| hex == to)
        .ok_or(ActionError::NotLegal)?;
    let mut events = vec![Event::HeroMoved {
        player,
        from,
        to: *to,
        cost,
    }];
    events.extend(ruin_reveal(state, *to));
    events.extend(reveals_next_to(state, reveal_time(state, player), *to, &[]));
    if is_assault_target(state, *to) {
        events.extend(assault_events(state, player, from, *to));
    }
    // MKUE p. 7, 4.c : les ennemis voisins des deux cases sont provoqués (spec 030) ; aucun
    // avec Distorsion spatiale (spec 057).
    let enemies = if state.move_rules(player).space_bending {
        Vec::new()
    } else {
        rampaging_enemies(state)
    };
    let provoked = provoked(from, *to, &enemies);
    if !provoked.is_empty() {
        events.push(Event::EnemiesProvoked {
            player,
            enemies: provoked,
        });
    }
    Ok(events)
}

/// `Action::Explore` : la tuile posée, ses sites et leurs ennemis.
fn decide_explore(
    state: &GameState,
    player: PlayerId,
    position: &TilePosition,
    rngs: &mut GameRngs,
) -> Result<Vec<Event>, ActionError> {
    // Le dessus de la pioche, ou une tuile écartée au hasard (MKUE p. 7, 5.c et 5.e).
    let tile = match state.tile_deck.tiles().first() {
        Some(top) => *top,
        None => *revealable_removed(&state.map, &state.tile_deck)
            .choose(rngs.rng(RngUsage::Tiles))
            .ok_or(ActionError::NotLegal)?,
    };
    let mut events = vec![
        Event::Explored {
            player,
            position: *position,
            cost: EXPLORE_COST,
        },
        Event::TileRevealed {
            tile,
            position: *position,
        },
    ];
    // MKUE p. 7, 5.f : les ennemis errants de la tuile (spec 030).
    let mut piles = state.enemy_piles.clone();
    let mut discards = state.enemy_discards.clone();
    events.extend(rampaging_tokens(
        tile,
        *position,
        &mut piles,
        &mut discards,
        rngs,
    ));
    // Spec 043 : puis les défenseurs face cachée des sites fortifiés, retournés aussitôt
    // de jour s'ils touchent la figurine (Q4).
    // Spec 062 : la cité de la tuile et ses défenseurs, au niveau du scénario.
    let placed = defenders(
        tile,
        *position,
        next_city_level(state),
        &mut piles,
        &mut discards,
        rngs,
    );
    let revealed = state
        .figure(player)
        .map(|figure| reveals_next_to(state, reveal_time(state, player), figure, &placed))
        .unwrap_or_default();
    events.extend(placed);
    events.extend(revealed);
    // Spec 045 : le jeton de ruines de chaque ruine de la tuile.
    let mut ruin_pile = state.ruin_pile.clone();
    events.extend(placed_ruins(
        tile,
        *position,
        &mut ruin_pile,
        reveal_time(state, player),
    ));
    // Spec 041 : une action avancée par monastère de la tuile (MKUE p. 7, 5.f).
    events.extend(monastery_cards(
        state.common_deck(CommonDeck::AdvancedActions),
        tile_monasteries(tile, *position).len(),
    ));
    // MKUE p. 18, *First Reconnaissance* : gloire par tuile révélée, puis le dernier tour
    // annoncé par la première cité (spec 063).
    if let Some(scenario) = state.scenario {
        if scenario.fame_per_tile() > 0 {
            events.push(Event::FameGained {
                player,
                fame: u32::from(scenario.fame_per_tile()),
            });
        }
        if scenario.end() == ScenarioEnd::CityRevealed
            && tile.is_city()
            && state.last_turn == LastTurn::NotAnnounced
        {
            events.push(Event::LastTurnAnnounced { player });
        }
    }
    Ok(events)
}
