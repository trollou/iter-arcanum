//! `apply` : tour, manche et tactiques (spec 060).

use super::*;

/// `apply_event` pour les événements de ce thème ; les autres ne changent rien ici.
pub(super) fn turn_event(state: &mut GameState, event: &Event) {
    match event {
        Event::TurnEnded { player } => apply_turn_ended(state, player),
        Event::TurnStarted { player, turn } => {
            state.turn = *turn;
            // Spec 050 : un choix d'action avancée ne survit pas à la fin du tour.
            if let Some(p) = state.player_mut(*player) {
                start_turn(p);
                p.extra_turn = false;
                // Spec 060 : le tour de plus d'Au bon moment commence ; il survit à Distorsion (L7).
                if let Some(tactic) = p.tactic.as_mut() {
                    tactic.right_moment = false;
                }
            }
            // Spec 063 : le tour qui suit l'annonce est le dernier.
            if state.last_turn == LastTurn::Next {
                state.last_turn = LastTurn::Current;
            }
        }
        Event::TacticChosen { player, tactic } => {
            if let Some(p) = state.player_mut(*player) {
                p.tactic = Some(crate::TacticState::new(*tactic));
                // Spec 060 (Q3) : le choix « en la prenant » suit, jusqu'au `TurnStarted`.
                p.tactic_follow_up = matches!(
                    tactic,
                    Tactic::Rethink | Tactic::ManaSteal | Tactic::Preparation
                );
            }
            state.tactic_choice = state.players.iter().any(|p| p.tactic.is_none());
        }
        Event::DieStolen { player, die } => {
            if let Some(tactic) = state.player_mut(*player).and_then(|p| p.tactic.as_mut()) {
                tactic.stolen_die = Some(*die);
            }
        }
        Event::LongNightUsed { player, cards } => {
            if let Some(p) = state.player_mut(*player) {
                for card in cards {
                    if let Some(index) = p.discard_pile.iter().position(|held| held == card) {
                        p.discard_pile.remove(index);
                    }
                }
                let mut deck = p.deck.cards().to_vec();
                deck.extend_from_slice(cards);
                p.deck = DeedDeck::new(deck);
                if let Some(tactic) = p.tactic.as_mut() {
                    tactic.flipped = true;
                }
            }
        }
        Event::CardStored { player, card } => {
            if let Some(p) = state.player_mut(*player)
                && p.deck.remove(*card)
            {
                p.sparing_pending = false;
                if let Some(tactic) = p.tactic.as_mut() {
                    tactic.stored.push(*card);
                }
            }
        }
        Event::StoredCardsTaken { player } => {
            if let Some(p) = state.player_mut(*player) {
                p.sparing_pending = false;
                if let Some(tactic) = p.tactic.as_mut() {
                    tactic.flipped = true;
                    p.hand.append(&mut tactic.stored);
                }
            }
        }
        Event::CardsShuffledIntoDeck { player, cards } => {
            if let Some(p) = state.player_mut(*player) {
                let mut deck = p.deck.cards().to_vec();
                for &card in cards {
                    if take_from_hand(&mut p.hand, card) {
                        deck.push(card);
                    }
                }
                p.deck = DeedDeck::new(deck);
                if let Some(tactic) = p.tactic.as_mut() {
                    tactic.flipped = true;
                }
            }
        }
        Event::ManaSearchUsed { player } => {
            if let Some(tactic) = state.player_mut(*player).and_then(|p| p.tactic.as_mut()) {
                tactic.mana_search_used = true;
            }
        }
        Event::DrawStopped { player, missing } => {
            if let Some(p) = state.player_mut(*player) {
                p.draw_stopped = Some(*missing);
            }
        }
        Event::RightMomentAnnounced { player } => {
            if let Some(tactic) = state.player_mut(*player).and_then(|p| p.tactic.as_mut()) {
                tactic.right_moment = true;
                tactic.flipped = true;
            }
        }
        Event::StolenDieUsed { player } => {
            if let Some(tactic) = state.player_mut(*player).and_then(|p| p.tactic.as_mut()) {
                tactic.flipped = true;
            }
        }
        Event::StolenDieReturned { player, color } => {
            let die = state
                .player_mut(*player)
                .and_then(|p| p.tactic.as_mut())
                .and_then(|tactic| tactic.stolen_die.take());
            if let Some(face) = die.and_then(|die| state.source.get_mut(usize::from(die))) {
                *face = *color;
            }
        }
        Event::CardSearched { player, card } => {
            if let Some(p) = state.player_mut(*player)
                && p.deck.remove(*card)
            {
                p.hand.push(*card);
            }
        }
        Event::TacticsRemoved { tactics } => {
            state.removed_tactics.extend(tactics.iter().copied());
        }
        Event::RoundEnded { .. } => {
            state.round_over = true;
            // Annoncer la fin de la manche est une décision du tour : plus de bandeau (spec 035).
            for p in &mut state.players {
                p.levels_gained.clear();
                // Spec 057 : Puits de mana finit avec la manche ; le dé gardé revient.
                p.modifiers
                    .retain(|modifier| modifier.duration != ModifierDuration::Round);
                p.kept_die = None;
                // Spec 055 (Q2) : le choix des bannières à reprendre attend.
                p.banner_reclaim = p.units.iter().any(|owned| owned.banner.is_some());
                // Spec 060 (MKUE p. 4, 2.f) : les tactiques sont reprises.
                // Spec 060 (MKUE p. 4, 2.g) : les cartes sous Provisions rejoignent les autres.
                if let Some(tactic) = p.tactic.as_mut() {
                    p.discard_pile.append(&mut tactic.stored);
                }
                p.tactic = None;
            }
            // Spec 061 : la tactique du fantôme est reprise ; son annonce ne vaut que pour la manche.
            if let Some(dummy) = state.dummy.as_mut() {
                dummy.tactic = None;
                dummy.announced = false;
            }
        }
        Event::RoundStarted { round, time_of_day } => {
            state.round = *round;
            state.time_of_day = *time_of_day;
            state.round_over = false;
            // Spec 051 : les compétences retournées face visible (MKUE p. 4, 2.g) ; Motivation
            // de nouveau permise (Compendium, *Motivation* S5).
            for p in &mut state.players {
                for owned in &mut p.skills {
                    owned.state = SkillState::Ready;
                }
                p.motivation_block = 0;
                // Spec 058 (Compendium, *Skill Tokens* S4) : la fin de la manche efface les
                // avantages du centre.
                p.center_advantages.clear();
            }
            // Spec 060 (MKUE p. 4, 3) : le choix des tactiques ouvre la manche.
            state.tactic_choice = !offered_tactics(state).is_empty();
        }
        Event::ExtraTurnStarted { player } => {
            if let Some(p) = state.player_mut(*player) {
                start_turn(p);
                p.extra_turn_pending = false;
                p.extra_turn = true;
            }
        }
        _ => {}
    }
}

/// `Event::TurnEnded` : la fin du tour.
fn apply_turn_ended(state: &mut GameState, player: &PlayerId) {
    let spells_offered = !state.offer(Offer::Spells).is_empty();
    let advanced_offered = !state.offer(Offer::AdvancedActions).is_empty();
    let units_offered = state
        .offer(Offer::Units)
        .iter()
        .any(|card| card.unit().is_some());
    if let Some(p) = state.player_mut(*player) {
        // MKUE p. 11, 3.b : dans l'ordre où les cartes ont été jouées.
        p.discard_pile
            .extend(p.play_area.drain(..).map(|played| played.card));
        p.move_points = 0;
        p.influence_points = 0;
        p.interaction_influence = 0;
        p.source_dice_forbidden = false;
        // Spec 058 (Q1) : l'avantage posé ce tour servira au tour suivant.
        let calming = p
            .center_advantages
            .iter()
            .any(|advantage| advantage.skill == Skill::CalmingTheWeather && !advantage.active);
        for advantage in &mut p.center_advantages {
            advantage.active = true;
        }
        p.opening_die = None;
        p.opening_die_kept = false;
        p.opening_die_due = false;
        // MKUE p. 11, 3.d : les points de soin non dépensés sont perdus.
        p.healing = 0;
        // MKUE p. 11, 3.a : les jetons retournent à la réserve.
        p.mana_tokens.clear();
        p.spent_mana_tokens.clear();
        // Spec 056 : Maîtrise du cristal ne vaut que pour les cristaux du tour.
        p.crystals_spent.clear();
        // Les dés de plus ne valaient que pour ce tour (spec 025).
        p.extra_dice = 0;
        p.black_any_dice = 0;
        p.any_color_dice = false;
        // Le combat du tour est fini (spec 031) : ses ennemis non vaincus restent.
        p.movement_ended = false;
        p.enemies_to_fight.clear();
        p.combat = None;
        p.moved_or_fought = false;
        p.interaction = None;
        p.burning = None;
        p.assault = None;
        p.adventure = None;
        p.night_adventure = false;
        p.reward_die = None;
        // Spec 043 : les récompenses sort deviennent des choix, sauf offre vide.
        p.spell_choices = if spells_offered { p.spell_rewards } else { 0 };
        p.spell_rewards = 0;
        // Spec 045 : les récompenses de ruines deviennent des choix, sauf offre vide
        // (Compendium, *Miscellaneous* S11) ; le lot de cristaux est déjà donné.
        p.crystal_set_rewards = 0;
        if advanced_offered {
            p.advanced_action_choices += p.advanced_action_rewards;
        }
        p.advanced_action_rewards = 0;
        p.unit_choices = if units_offered { p.unit_rewards } else { 0 };
        p.unit_rewards = 0;
        p.plundered = false;
        p.units_recruited = 0;
        p.unit_activated = false;
        // Spec 035 : le bandeau des niveaux gagnés ne vaut que pour le tour qui suit.
        p.levels_gained.clear();
        // Spec 051 : les compétences « une fois par tour » de nouveau prêtes (MKUE p. 5,
        // *Using Skills* 4.c).
        for owned in &mut p.skills {
            if owned.state == SkillState::UsedThisTurn {
                owned.state = SkillState::Ready;
            }
        }
        p.motivation_block = p.motivation_block.saturating_sub(1);
        // MKUE p. 5, *Effects* 1.b ; ADR 0008 : les effets du tour finissent avec lui.
        p.modifiers
            .retain(|modifier| modifier.duration != ModifierDuration::Turn);
        // Spec 058 (Q2) : l'avantage de Temps calme, d'office pour le tour suivant.
        if calming {
            p.modifiers.push(calm_weather(CALM_ADVANTAGE));
        }
    }
}
