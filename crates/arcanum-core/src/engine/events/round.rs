//! Événements pour `decide` : fin de la manche, nouvelle manche et tactiques (spec 060).

use super::*;

/// Fin de la manche annoncée (MKUE p. 6, 3 et 4) : en solo, sans joueur fantôme, elle finit
/// tout de suite. Si le nombre de manches n'est pas atteint, préparer la suivante (p. 4, 2),
/// dans l'ordre du livret : plateau Jour/Nuit (a), Source (b), puis cartes (g).
pub(in crate::engine) fn end_of_round(
    state: &GameState,
    player: PlayerId,
    rngs: &mut GameRngs,
) -> Vec<Event> {
    let mut events = vec![Event::RoundEnded { round: state.round }];
    let last = state
        .scenario
        .is_some_and(|scenario| state.round >= u32::from(scenario.rounds()));
    // MKUE p. 18 : une manche finie pendant le dernier tour termine la partie (spec 063), même
    // celui où la cité est révélée, quand le fantôme a annoncé la fin de la manche (spec 061,
    // C30) ; MKUE p. 4, 5.a : la dernière manche finie aussi.
    let end = if state.last_turn != LastTurn::NotAnnounced {
        Some(crate::game_end::last_turn_reason(state))
    } else {
        last.then_some(GameEndReason::LastRound)
    };
    if let Some(reason) = end {
        events.push(Event::GameEnded { reason });
        return events;
    }
    // Spec 060 : le retrait ne compte que pour les manches suivantes.
    events.extend(removed_tactics(state));
    let Some(p) = state.player(player) else {
        return events;
    };
    // Spec 055 (Q2) : avec des bannières assignées, leur reprise se choisit avant le mélange.
    if p.units.iter().any(|owned| owned.banner.is_some()) {
        return events;
    }
    // La nouvelle manche se prépare sur l'état d'après le retrait des tactiques, comme après la
    // reprise des bannières.
    let after = events.iter().fold(state.clone(), apply);
    events.extend(new_round_events(&after, player, rngs, &[]));
    events
}

/// Spec 060 : les tactiques retirées de la partie à la fin de la manche, selon la règle du
/// scénario (MKUE p. 19, *Special Rules* ; Compendium, *Scenarios* Q8).
fn removed_tactics(state: &GameState) -> Option<Event> {
    let rule = state.scenario.map(|scenario| scenario.tactics())?;
    let tactics: Vec<crate::Tactic> = match rule {
        TacticRule::None => return None,
        TacticRule::Chosen {
            removal: TacticRemoval::AllTaken,
            ..
        } => state
            .players
            .iter()
            .filter_map(|p| p.tactic.as_ref().map(|taken| taken.tactic))
            // Spec 061 : celle du fantôme aussi.
            .chain(state.dummy.as_ref().and_then(|dummy| dummy.tactic))
            .collect(),
    };
    (!tactics.is_empty()).then_some(Event::TacticsRemoved { tactics })
}

/// Spec 060 : les tactiques du moment de la journée que l'on peut encore prendre, par numéro :
/// ni retirées de la partie, ni déjà prises cette manche. Aucune sans règle des tactiques.
pub(crate) fn offered_tactics(state: &GameState) -> Vec<crate::Tactic> {
    if state
        .scenario
        .is_none_or(|scenario| scenario.tactics() == TacticRule::None)
    {
        return Vec::new();
    }
    crate::Tactic::of(state.time_of_day)
        .filter(|tactic| !state.removed_tactics.contains(tactic))
        .filter(|tactic| {
            !state.players.iter().any(|p| {
                p.tactic
                    .as_ref()
                    .is_some_and(|taken| taken.tactic == *tactic)
            })
        })
        .collect()
}

/// La nouvelle manche après `RoundEnded` (MKUE p. 4, 2), les bannières `reclaimed` reprises dans
/// le deck avant son mélange (spec 055).
pub(in crate::engine) fn new_round_events(
    state: &GameState,
    player: PlayerId,
    rngs: &mut GameRngs,
    reclaimed: &[CardId],
) -> Vec<Event> {
    let mut events = Vec::new();
    let Some(p) = state.player(player) else {
        return events;
    };
    let time_of_day = state.time_of_day.flipped();
    events.push(Event::RoundStarted {
        round: state.round + 1,
        time_of_day,
    });
    // Spec 043 (Compendium, *End of Round* S6) : une manche de Jour retourne les défenseurs
    // voisins de la figurine.
    if let Some(figure) = p.figure {
        events.extend(reveals_next_to(state, time_of_day, figure, &[]));
    }
    // Spec 045 (carte *Ancient Ruins*) : une manche de Jour retourne tous les jetons de ruines.
    if time_of_day == TimeOfDay::Day {
        events.extend(
            state
                .ruins
                .iter()
                .filter(|(_, placed)| !placed.face_up)
                .map(|(&hex, placed)| Event::RuinTokenRevealed {
                    hex,
                    token: placed.token,
                }),
        );
    }
    events.push(Event::SourceRolled {
        // Solo : un seul joueur réel, comme à la mise en place (MKUE p. 3).
        dice: roll_source(source_size(1), rngs.rng(RngUsage::Source)),
    });
    events.extend(refreshed_offers(state));
    // MKUE p. 4, 2.g ; p. 5, 9 : les unités redeviennent disponibles (spec 042).
    if !p.units.is_empty() {
        events.push(Event::UnitsReadied { player });
    }
    // Le mélange part de la suite deck, défausse, main, pour que la graine suffise.
    let mut order: Vec<CardId> = p
        .deck
        .cards()
        .iter()
        .chain(&p.discard_pile)
        .chain(&p.hand)
        // Spec 057 (Q3) : la carte écartée par Distorsion temporelle revient dans le deck.
        .chain(&p.set_aside)
        // Spec 055 : les bannières reprises.
        .chain(reclaimed)
        .copied()
        .collect();
    order.shuffle(rngs.rng(RngUsage::Deck(player)));
    let drawn: Vec<Event> = order
        .iter()
        // MKUE p. 4, 2.g : jusqu'à la limite de main du niveau (spec 035), forteresses
        // comprises (spec 043).
        .take(draw_limit(state, player, p))
        .map(|&card| Event::CardDrawn { player, card })
        .collect();
    let bonus_used = !drawn.is_empty() && draw_bonus(p) > 0;
    events.push(Event::DeckShuffled { player, order });
    events.extend(drawn);
    if bonus_used {
        events.push(Event::DrawBonusUsed { player });
    }
    // Spec 061 (MKUE p. 4, 2.g ; p. 15) : le fantôme reprend toutes ses cartes, l'action avancée
    // ajoutée comprise, et les mélange.
    let refreshed = events.iter().fold(state.clone(), apply);
    if let Some(dummy) = refreshed.dummy.as_ref() {
        let mut order: Vec<CardId> = dummy.deck.iter().chain(&dummy.discard).copied().collect();
        order.shuffle(rngs.rng(RngUsage::Dummy));
        events.push(Event::DummyDeckShuffled { order });
    }
    // Spec 060 : avec des tactiques à choisir, c'est la prise qui ouvre le tour 1.
    let after = events.iter().fold(state.clone(), apply);
    if !after.tactic_choice {
        events.push(Event::TurnStarted { player, turn: 1 });
    }
    events
}

/// Le renouvellement des offres d'une nouvelle manche (MKUE p. 4, 2.c à 2.e ; spec 050) : les
/// unités sous leur pioche puis trois nouvelles (élite, régulière, élite une fois une tuile
/// royaume révélée, si le scénario utilise les élites) ; pour les actions avancées puis les sorts,
/// la dernière place sous la pioche et une carte en première place. Aucun hasard : les pioches
/// sont déjà mélangées.
fn refreshed_offers(state: &GameState) -> Vec<Event> {
    let mut decks = state.common_decks.clone();
    let mut events = Vec::new();
    let deal = |decks: &mut BTreeMap<CommonDeck, Vec<CommonCard>>, offer, deck| {
        let pile = decks.entry(deck).or_default();
        (!pile.is_empty()).then(|| Event::OfferCardDealt {
            offer,
            card: pile.remove(0),
        })
    };
    for &card in state.offer(Offer::Units) {
        events.push(Event::OfferCardReturned {
            offer: Offer::Units,
            card,
        });
        decks.entry(card.deck()).or_default().push(card);
    }
    let core_revealed = state
        .map
        .tiles()
        .any(|(_, tile)| matches!(tile, TileId::Core(_)));
    let elite = core_revealed && state.scenario.is_some_and(|s| s.elite_units());
    for slot in 0..OFFER_SIZE {
        // Compendium, *Silver Units* S1 : on commence par une élite ; L2 : une régulière si la
        // pioche d'élite est vide.
        let wanted = if elite && slot % 2 == 0 {
            CommonDeck::EliteUnits
        } else {
            CommonDeck::RegularUnits
        };
        let dealt = deal(&mut decks, Offer::Units, wanted)
            .or_else(|| deal(&mut decks, Offer::Units, CommonDeck::RegularUnits));
        events.extend(dealt);
    }
    // Spec 041 : une action avancée par monastère du plateau de jeu, après les unités et avant
    // le renouvellement de l'offre d'actions avancées (MKUE p. 4, 2.c et 2.d ; FAQ p3).
    let monasteries = board_monasteries(&state.map)
        .into_iter()
        .filter(|&hex| !state.is_burned_monastery(hex))
        .count();
    let advanced = decks.entry(CommonDeck::AdvancedActions).or_default();
    let dealt = monastery_cards(advanced, monasteries);
    advanced.drain(..dealt.len());
    events.extend(dealt);
    for (offer, deck) in [
        (Offer::AdvancedActions, CommonDeck::AdvancedActions),
        (Offer::Spells, CommonDeck::Spells),
    ] {
        if let Some(&lowest) = state.offer(offer).last() {
            events.extend(lowest_card_leaves(state, offer, lowest, &mut decks));
        }
        events.extend(deal(&mut decks, offer, deck));
    }
    events
}

/// La dernière place d'une offre à la nouvelle manche (MKUE p. 4, 2.d et 2.e) : sous sa pioche ;
/// avec un fantôme (MKUE p. 15, *When preparing a new Round* ; FAQ p3), l'action avancée va dans
/// son deck, et il gagne un cristal de la couleur du sort.
fn lowest_card_leaves(
    state: &GameState,
    offer: Offer,
    lowest: CommonCard,
    decks: &mut BTreeMap<CommonDeck, Vec<CommonCard>>,
) -> Vec<Event> {
    let dummy = state.dummy.is_some();
    if let (true, Offer::AdvancedActions, Some(card)) = (dummy, offer, lowest.deed()) {
        return vec![Event::DummyCardAdded { card }];
    }
    decks.entry(lowest.deck()).or_default().push(lowest);
    let mut events = vec![Event::OfferCardReturned {
        offer,
        card: lowest,
    }];
    if let (true, Offer::Spells, Some(definition)) = (
        dummy,
        offer,
        lowest.deed().and_then(|card| card.card().definition()),
    ) {
        events.push(Event::DummyCrystalGained {
            color: definition.color,
        });
    }
    events
}

/// Spec 060 : la prise d'une tactique, et ce qu'elle fait aussitôt (MKUE p. 4, 3.b) ; le tour 1
/// commence, sauf si un choix suit la prise (Q3).
pub(in crate::engine) fn choose_tactic_events(
    state: &GameState,
    player: PlayerId,
    tactic: Tactic,
    rngs: &mut GameRngs,
) -> Vec<Event> {
    let mut events = vec![Event::TacticChosen { player, tactic }];
    // Spec 061 (MKUE p. 4, 3.a) : le fantôme tire la sienne juste après.
    let after = apply(state.clone(), &events[0]);
    events.extend(dummy_tactic(&after, rngs));
    let deck = state.deck(player);
    let follows = match tactic {
        // Carte ; Compendium, *Great Start* S1 : deux cartes, ce que le deck peut.
        Tactic::GreatStart => {
            events.extend(
                deck.iter()
                    .take(2)
                    .map(|&card| Event::CardDrawn { player, card }),
            );
            false
        }
        Tactic::Rethink => true,
        // L3 : sans dé de couleur de base, rien à voler.
        Tactic::ManaSteal => state
            .player(player)
            .is_some_and(|p| !stealable_dice(state, p).is_empty()),
        // L5 : deck vide, rien à chercher.
        Tactic::Preparation => !deck.is_empty(),
        _ => false,
    };
    if !follows {
        events.push(Event::TurnStarted { player, turn: 1 });
    }
    events
}

/// Spec 060 (carte ; Compendium, *Rethink* S1) : défausser `cards`, piocher autant, puis remélanger
/// la défausse dans le deck ; sans carte défaussée, rien ne change (C10).
pub(in crate::engine) fn rethink_events(
    state: &GameState,
    player: PlayerId,
    cards: &[CardId],
    rngs: &mut GameRngs,
) -> Result<Vec<Event>, ActionError> {
    let p = state.player(player).ok_or(ActionError::NotLegal)?;
    let mut events: Vec<Event> = cards
        .iter()
        .map(|&card| Event::CardDiscarded { player, card })
        .collect();
    if !cards.is_empty() {
        let drawn = cards.len().min(p.deck.len());
        events.extend(
            p.deck.cards()[..drawn]
                .iter()
                .map(|&card| Event::CardDrawn { player, card }),
        );
        let mut order: Vec<CardId> = p.deck.cards()[drawn..]
            .iter()
            .chain(&p.discard_pile)
            .chain(cards)
            .copied()
            .collect();
        order.shuffle(rngs.rng(RngUsage::Deck(player)));
        events.push(Event::DeckReshuffled { player, order });
    }
    events.push(Event::TurnStarted { player, turn: 1 });
    Ok(events)
}

/// Spec 060 (carte ; Compendium, *Preparation* S1) : la carte cherchée en main, puis le deck
/// mélangé.
pub(in crate::engine) fn preparation_events(
    state: &GameState,
    player: PlayerId,
    card: CardId,
    rngs: &mut GameRngs,
) -> Result<Vec<Event>, ActionError> {
    let p = state.player(player).ok_or(ActionError::NotLegal)?;
    let mut order: Vec<CardId> = p
        .deck
        .cards()
        .iter()
        .copied()
        .filter(|&held| held != card)
        .collect();
    order.shuffle(rngs.rng(RngUsage::Deck(player)));
    Ok(vec![
        Event::CardSearched { player, card },
        Event::DeckReshuffled { player, order },
        Event::TurnStarted { player, turn: 1 },
    ])
}

/// Spec 060 (carte) : Longue Nuit face visible, avec une défausse à remélanger.
pub(in crate::engine) fn long_night_usable(p: &PlayerState) -> bool {
    p.tactic
        .as_ref()
        .is_some_and(|tactic| tactic.tactic == Tactic::LongNight && !tactic.flipped)
        && !p.discard_pile.is_empty()
}

/// Spec 060 (carte ; FAQ p2) : 3 cartes de la défausse au hasard (moins si elle en a moins) dans
/// le deck ; si la pioche de fin de tour attendait, elle continue (Q5).
pub(in crate::engine) fn long_night_events(
    state: &GameState,
    player: PlayerId,
    rngs: &mut GameRngs,
) -> Result<Vec<Event>, ActionError> {
    let p = state.player(player).ok_or(ActionError::NotLegal)?;
    let mut cards = p.discard_pile.clone();
    cards.shuffle(rngs.rng(RngUsage::Deck(player)));
    cards.truncate(3);
    let mut events = vec![Event::LongNightUsed { player, cards }];
    if let Some(missing) = p.draw_stopped {
        let after = events.iter().fold(state.clone(), apply);
        let deck = after.deck(player).to_vec();
        events.extend(turn_end_draw(&after, player, &deck, missing));
    }
    Ok(events)
}

/// Spec 060 (carte ; Compendium, *Midnight Meditation* S2) : les cartes dans le deck, le deck
/// mélangé, puis autant de cartes piochées.
pub(in crate::engine) fn meditation_events(
    state: &GameState,
    player: PlayerId,
    cards: &[CardId],
    rngs: &mut GameRngs,
) -> Result<Vec<Event>, ActionError> {
    let p = state.player(player).ok_or(ActionError::NotLegal)?;
    let mut order: Vec<CardId> = p.deck.cards().iter().chain(cards).copied().collect();
    order.shuffle(rngs.rng(RngUsage::Deck(player)));
    let mut events = vec![Event::CardsShuffledIntoDeck {
        player,
        cards: cards.to_vec(),
    }];
    events.extend(
        order
            .iter()
            .take(cards.len())
            .map(|&card| Event::CardDrawn { player, card }),
    );
    events.insert(1, Event::DeckReshuffled { player, order });
    Ok(events)
}
