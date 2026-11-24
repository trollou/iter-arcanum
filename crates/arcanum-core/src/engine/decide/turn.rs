//! Décision des actions du tour et de la manche : tactiques, fin du tour, fin de la manche.

use super::*;

/// Les événements d'une action de ce thème, déjà validée par `decide`.
pub(super) fn turn_actions(
    state: &GameState,
    player: PlayerId,
    action: &Action,
    rngs: &mut GameRngs,
) -> Result<Vec<Event>, ActionError> {
    match action {
        Action::AnnounceEndOfRound => Ok(announce_end_of_round(state)),
        // Spec 060 (MKUE p. 4, 3) : la prise ouvre le tour 1.
        Action::ChooseTactic { tactic } => Ok(choose_tactic_events(state, player, *tactic, rngs)),
        Action::RethinkDiscard { cards } => rethink_events(state, player, cards, rngs),
        Action::PrepareCard { card } => preparation_events(state, player, *card, rngs),
        // Spec 060 (carte) : le dé posé sur la tactique, puis le tour 1.
        Action::StealDie { die } => Ok(vec![
            Event::DieStolen { player, die: *die },
            Event::TurnStarted { player, turn: 1 },
        ]),
        // Spec 060 (carte) : les dés choisis relancés, le résultat gardé.
        Action::ManaSearch { dice } => Ok(std::iter::once(Event::ManaSearchUsed { player })
            .chain(dice.iter().map(|&die| Event::DieRerolled {
                die,
                color: roll_die(rngs.rng(RngUsage::Source)),
            }))
            .collect()),
        Action::MidnightMeditation { cards } => meditation_events(state, player, cards, rngs),
        // Spec 060 (carte) : la carte du dessus sous la tactique, ou les cartes en main.
        Action::SparingPower { take: true } => Ok(vec![Event::StoredCardsTaken { player }]),
        Action::SparingPower { take: false } => {
            let card = *state.deck(player).first().ok_or(ActionError::NotLegal)?;
            Ok(vec![Event::CardStored { player, card }])
        }
        Action::UseLongNight => long_night_events(state, player, rngs),
        // Spec 060 (Q5) : refusée, le tour suivant commence avec la main incomplète.
        Action::DeclineLongNight => Ok(turn_end_draw(state, player, &[], 0)),
        Action::AnnounceRightMoment => Ok(vec![Event::RightMomentAnnounced { player }]),
        Action::EndTurn {
            discard,
            reclaim,
            glade,
            steady_tempo,
        } => decide_end_turn(state, player, discard, reclaim, glade, steady_tempo, rngs),
        // Spec 055 (MKUE p. 5, 8.a) : la bannière déjà portée par l'unité va dans la défausse.
        Action::ReclaimBanners { banners } => {
            let mut events = vec![Event::BannersReclaimed {
                player,
                banners: banners.clone(),
            }];
            events.extend(new_round_events(state, player, rngs, banners));
            Ok(events)
        }
        // L'aiguillage de `decide` n'envoie ici que les actions de ce thème.
        _ => Err(ActionError::NotLegal),
    }
}

/// `Action::EndTurn` : la fin du tour, ses récompenses et la pioche.
fn decide_end_turn(
    state: &GameState,
    player: PlayerId,
    discard: &[CardId],
    reclaim: &Option<CardId>,
    glade: &Option<WoundPile>,
    steady_tempo: &bool,
    rngs: &mut GameRngs,
) -> Result<Vec<Event>, ActionError> {
    // Spec 052 : la suite s'évalue après la retraite forcée (MKUE p. 11, 2).
    let withdrawal = forced_withdrawal_event(state, player);
    let after = after_forced_withdrawal(state, player);
    let p = after.player(player).ok_or(ActionError::NotLegal)?;
    let mut events = thrown_wound_events(p, player);
    events.extend(returned_dice_events(state, p, player, rngs));
    // MKUE p. 11, 2 : puis la retraite forcée.
    events.extend(withdrawal);
    events.extend(modifier_end_events(p, player));
    // Spec 025 : Euphorie cristalline revient en main avant que la zone de jeu soit
    // défaussée.
    if let Some(reclaim) = reclaim {
        events.push(Event::CardDiscarded {
            player,
            card: *reclaim,
        });
        events.push(Event::CardReturnedToHand {
            player,
            card: reclaimable(p).ok_or(ActionError::NotLegal)?,
        });
    }
    // Spec 052 : Temporisation quitte la zone de jeu avant sa défausse (MKUE p. 11, 3.b).
    if *steady_tempo {
        let (card, top) = steady_tempo_keep(p).ok_or(ActionError::NotLegal)?;
        events.push(Event::SteadyTempoKept { player, card, top });
    }
    // Spec 056 : Maîtrise du cristal renforcée, avant la défausse de la zone de jeu.
    let mut inventory = p.inventory;
    let returned = after.crystals_to_return(player);
    for &color in &returned {
        inventory.add(color);
    }
    if !returned.is_empty() {
        events.push(Event::CrystalsReturned {
            player,
            colors: returned,
        });
    }
    // Spec 057 : Distorsion temporelle, les cartes jouées reviennent en main, la carte du
    // sort (ou Don pour la magie qui l'a lancé depuis l'offre) écartée.
    events.extend(time_bending_events(p, player));
    // Spec 058 (Q1) : l'avantage du centre ne dure que ce tour suivant.
    events.extend(
        p.center_advantages
            .iter()
            .filter(|advantage| advantage.active)
            .map(|advantage| Event::CenterSkillReturned {
                player,
                skill: advantage.skill,
            }),
    );
    events.push(Event::TurnEnded { player });
    // MKUE p. 11, 4.b : bénéfice de la case, après le nettoyage (3) et avant la
    // pioche (7) ; rien si la couleur est déjà pleine.
    if let Some(color) = p.figure.and_then(|hex| mine_color(&state.map, hex))
        && !inventory.is_full(color)
    {
        inventory.add(color);
        events.push(Event::CrystalGained { player, color });
    }
    // Spec 044 : la Blessure de la clairière, bénéfice de la case (MKUE p. 11, 4.a).
    if let Some(from) = glade {
        events.push(Event::WoundThrownAway {
            player,
            from: *from,
        });
    }
    // Spec 045 : le lot de cristaux d'une ruine, un de chaque couleur non pleine (MKUE
    // p. 11, 5.a), après le cristal de la mine (et les cristaux rendus, spec 056).
    for _ in 0..p.crystal_set_rewards {
        for color in BasicColor::ALL {
            if !inventory.is_full(color) {
                inventory.add(color);
                events.push(Event::CrystalGained { player, color });
            }
        }
    }
    // La suite se calcule sur l'état après la fin du tour (spec 044) : un dé or l'interrompt.
    let after = events.iter().fold(state.clone(), apply);
    events.extend(turn_end_rewards(&after, player, discard, rngs));
    Ok(events)
}

/// Les Blessures reçues ce tour et jetées au début de la fin du tour.
fn thrown_wound_events(p: &PlayerState, player: PlayerId) -> Vec<Event> {
    // Spec 055 (Bannière de protection renforcée ; Compendium, *Banner of Protection* Q1) :
    // au début de la fin du tour, les Blessures reçues ce tour sont jetées, au plus celles
    // qui restent.
    let mut events: Vec<Event> = Vec::new();
    if p.modifiers
        .iter()
        .any(|modifier| modifier.kind == ModifierKind::ThrowAwayReceivedWounds)
    {
        for (pile, cards, received) in [
            (WoundPile::Hand, &p.hand, p.wounds_received_hand),
            (
                WoundPile::DiscardPile,
                &p.discard_pile,
                p.wounds_received_discard,
            ),
        ] {
            let present = cards.iter().filter(|card| card.card().is_wound()).count();
            let count = present.min(usize::try_from(received).unwrap_or(usize::MAX));
            events.extend((0..count).map(|_| Event::WoundThrownAway { player, from: pile }));
        }
    }
    events
}

/// Les dés pris, posés ou volés, rendus à la Source à la fin du tour.
fn returned_dice_events(
    state: &GameState,
    p: &PlayerState,
    player: PlayerId,
    rngs: &mut GameRngs,
) -> Vec<Event> {
    let mut events = Vec::new();
    // MKUE p. 11, 1 : d'abord relancer et rendre les dés pris, dans l'ordre où ils ont
    // été pris ; une simple relance, sans la règle de la moitié (p. 5, 3.a).
    // Spec 058 : le dé d'Embrasure de la Source gardé revient tel quel.
    events.extend(p.dice_taken.iter().map(|&die| Event::DieReturned {
        player,
        die,
        color: match state.source.get(usize::from(die)) {
            Some(&face) if p.opening_die == Some(die) && p.opening_die_kept => face,
            _ => roll_die(rngs.rng(RngUsage::Source)),
        },
    }));
    // Spec 025 : un dé mis sur une couleur revient tel quel, sans tirage.
    events.extend(p.dice_set.iter().filter_map(|&die| {
        let color = *state.source.get(usize::from(die))?;
        Some(Event::DieReturned { player, die, color })
    }));
    // Spec 060 (carte Vol de mana) : le dé volé qui a servi, relancé et rendu.
    if p.tactic
        .as_ref()
        .is_some_and(|tactic| tactic.flipped && tactic.stolen_die.is_some())
    {
        events.push(Event::StolenDieReturned {
            player,
            color: roll_die(rngs.rng(RngUsage::Source)),
        });
    }
    events
}

/// La réputation et la gloire que des effets du tour donnent à sa fin.
fn modifier_end_events(p: &PlayerState, player: PlayerId) -> Vec<Event> {
    let mut events = Vec::new();
    // Spec 052 : la réputation de fin de tour (Intimidation), même sur X (sans effet).
    events.extend(
        p.modifiers
            .iter()
            .filter_map(|modifier| match modifier.kind {
                ModifierKind::EndOfTurnReputation(change) => {
                    Some(Event::ReputationChanged { player, change })
                }
                _ => None,
            }),
    );
    // Spec 053 : Récit héroïque, par unité recrutée ce tour.
    let recruits = p.units_recruited;
    for modifier in &p.modifiers {
        if let ModifierKind::EndOfTurnPerRecruit { fame, reputation } = modifier.kind
            && recruits > 0
        {
            if fame > 0 {
                events.push(Event::FameGained {
                    player,
                    fame: u32::from(fame) * u32::from(recruits),
                });
            }
            if reputation != 0 {
                let change = i16::from(reputation) * i16::from(recruits);
                events.push(Event::ReputationChanged {
                    player,
                    change: i8::try_from(change).unwrap_or(i8::MAX),
                });
            }
        }
    }
    events
}
