//! Actions légales de fin du tour : récompenses d'unité, annonce de fin de manche, défausse, retraite
//! forcée, clairière.

use super::*;

/// Spec 045 : l'unité de la récompense, une de l'offre d'unités (jamais une action avancée de
/// monastère), dans l'ordre de l'offre. Sans jeton de commandement libre, renvoyer une unité
/// (même blessée) ou renoncer (MKUE p. 11, 5.d).
pub(in crate::engine) fn unit_reward_actions(state: &GameState, p: &PlayerState) -> Vec<Action> {
    let offered: Vec<UnitCard> = state
        .offer(Offer::Units)
        .iter()
        .filter_map(|card| card.unit())
        .collect();
    let full = ordinary_units(p).count() >= usize::from(level_token(p.level.max(1)).command_tokens);
    if !full {
        return offered
            .into_iter()
            .map(|unit| Action::TakeUnitReward {
                unit,
                disband: None,
            })
            .collect();
    }
    let mut actions: Vec<Action> = offered
        .into_iter()
        .flat_map(|unit| {
            ordinary_units(p).map(move |owned| Action::TakeUnitReward {
                unit,
                disband: Some(owned.card),
            })
        })
        .collect();
    actions.push(Action::DeclineUnitReward);
    actions
}

/// MKUE p. 6, 3 et 3.a : au début du tour, deck vide. Tout ce qu'on fait pendant un tour
/// commence par jouer une carte, donc « zone de jeu vide » dit que le tour n'est pas commencé.
pub(super) fn can_announce_end_of_round(state: &GameState, player: PlayerId) -> bool {
    // Spec 061 (MKUE p. 6, 3 : *if the End of the Round has not been announced yet*).
    !state.dummy.as_ref().is_some_and(|dummy| dummy.announced)
        && state
            .player(player)
            .is_some_and(|p| p.deck.cards().is_empty() && p.play_area.is_empty())
}

/// MKUE p. 6, 3.b (issue privée 86) : main et deck vides au début du tour (zone de jeu vide), le héros
/// doit annoncer la fin de la manche ; il ne peut ni finir son tour ni défier.
pub(crate) fn must_announce_end_of_round(p: &PlayerState) -> bool {
    p.play_area.is_empty() && p.hand.is_empty() && p.deck.cards().is_empty()
}

/// Les fins de tour légales (spec 016, 025, 034) : pour chaque sous-ensemble des cartes non-Blessure
/// de la main, par taille croissante puis dans l'ordre de la main, autant de Blessures qu'on peut en
/// défausser, de 0 à toutes ; les Blessures étant identiques, ce sont les premières de la main (une
/// combinaison par nombre). Puis, si Euphorie cristalline a été jouée, une reprise par carte de la
/// main (une seule Blessure), chacune avec ses défausses. Seules restent celles que permet
/// `end_turn_allowed`. Chaque `discard` est dans l'ordre de la main.
pub(super) fn end_turn_actions(state: &GameState, player: PlayerId) -> Vec<Action> {
    let after = after_forced_withdrawal(state, player);
    let state = after.as_ref();
    let Some(p) = state.player(player) else {
        return Vec::new();
    };
    if must_announce_end_of_round(p) {
        return Vec::new();
    }
    let can_rest = !p.moved_or_fought;
    let allowed = |discard: &[CardId], reclaim: Option<CardId>| {
        end_turn_allowed(
            &p.hand,
            &p.play_area,
            p.played_elsewhere,
            can_rest,
            discard,
            reclaim,
        )
    };
    let in_hand_order = |cards: Vec<CardId>| -> Vec<CardId> {
        p.hand
            .iter()
            .copied()
            .filter(|card| cards.contains(card))
            .collect()
    };
    // Toutes les défausses de `cards` : sous-ensembles des non-Blessures, puis 0 à n Blessures.
    let discards = |cards: &[CardId]| -> Vec<Vec<CardId>> {
        let (wounds, others): (Vec<CardId>, Vec<CardId>) = cards
            .iter()
            .copied()
            .partition(|card| card.card().is_wound());
        (0..=others.len())
            .flat_map(|size| combinations(&others, size))
            .flat_map(|chosen| {
                (0..=wounds.len())
                    .map(|count| {
                        let mut discard = chosen.clone();
                        discard.extend(&wounds[..count]);
                        in_hand_order(discard)
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    };
    let mut actions: Vec<Action> = discards(&p.hand)
        .into_iter()
        .filter(|discard| allowed(discard, None))
        .map(|discard| Action::EndTurn {
            discard,
            reclaim: None,
            glade: None,
            steady_tempo: false,
        })
        .collect();
    if reclaimable(p).is_some() {
        let first_wound = p.hand.iter().copied().find(|card| card.card().is_wound());
        let candidates = p
            .hand
            .iter()
            .copied()
            .filter(|card| !card.card().is_wound() || Some(*card) == first_wound);
        for reclaim in candidates {
            let others: Vec<CardId> = p.hand.iter().copied().filter(|&c| c != reclaim).collect();
            actions.extend(
                discards(&others)
                    .into_iter()
                    .filter(|discard| allowed(discard, Some(reclaim)))
                    .map(|discard| Action::EndTurn {
                        discard,
                        reclaim: Some(reclaim),
                        glade: None,
                        steady_tempo: false,
                    }),
            );
        }
    }
    // Spec 044 : sur une clairière, pour chaque défausse, sans jeter, puis de la main, puis de la
    // défausse.
    actions
        .into_iter()
        .flat_map(|action| {
            let Action::EndTurn {
                discard, reclaim, ..
            } = action
            else {
                return vec![action];
            };
            [None, Some(WoundPile::Hand), Some(WoundPile::DiscardPile)]
                .into_iter()
                .filter(|&glade| glade_allowed(state, p, &discard, reclaim, glade))
                .flat_map(|glade| {
                    // Spec 052 : sans garder Temporisation, puis en la gardant.
                    let keeps: &[bool] = if steady_tempo_keep(p).is_some() {
                        &[false, true]
                    } else {
                        &[false]
                    };
                    keeps
                        .iter()
                        .map(|&steady_tempo| Action::EndTurn {
                            discard: discard.clone(),
                            reclaim,
                            glade,
                            steady_tempo,
                        })
                        .collect::<Vec<_>>()
                })
                .collect()
        })
        .collect()
}

/// Les cases de la retraite forcée (spec 052 ; MKUE p. 11, 2) : en remontant le trajet du tour
/// jusqu'à la première case sécurisée, celle-ci comprise ; `None` sur une case sécurisée. La case
/// du début du tour est sécurisée : le retour en trouve toujours une.
pub(crate) fn forced_withdrawal_path(state: &GameState, p: &PlayerState) -> Option<Vec<Hex>> {
    let figure = p.figure?;
    let conquered = state.conquered_sites();
    let safe = |hex: Hex| is_safe_space(&state.map, hex, &conquered);
    if safe(figure) {
        return None;
    }
    let mut path = Vec::new();
    for &hex in p.turn_path.iter().rev().skip(1) {
        path.push(hex);
        if safe(hex) {
            return Some(path);
        }
    }
    None
}

/// L'événement de la retraite forcée en tête de la fin du tour (spec 052) : les Blessures suivent
/// la numérotation des Blessures déjà données.
pub(in crate::engine) fn forced_withdrawal_event(
    state: &GameState,
    player: PlayerId,
) -> Option<Event> {
    let p = state.player(player)?;
    let path = forced_withdrawal_path(state, p)?;
    let wounds = (1..=path.len())
        .map(|offset| {
            let copy = p
                .wounds_given
                .saturating_add(u16::try_from(offset).unwrap_or(u16::MAX));
            CardId::new(DeedCard::Wound, copy)
        })
        .collect();
    Some(Event::ForcedWithdrawal {
        player,
        path,
        wounds,
    })
}

/// L'état sur lequel s'évalue la fin du tour : après la retraite forcée, s'il y en a une (spec 052 ;
/// MKUE p. 11, 2 à 4).
pub(crate) fn after_forced_withdrawal(
    state: &GameState,
    player: PlayerId,
) -> std::borrow::Cow<'_, GameState> {
    match forced_withdrawal_event(state, player) {
        Some(event) => std::borrow::Cow::Owned(apply(state.clone(), &event)),
        None => std::borrow::Cow::Borrowed(state),
    }
}

/// La Temporisation que la fin du tour peut garder (spec 052 ; carte) : la carte et `top` (sur le
/// deck) ; sous le deck seulement s'il n'est pas vide.
pub(in crate::engine) fn steady_tempo_keep(p: &PlayerState) -> Option<(CardId, bool)> {
    p.modifiers
        .iter()
        .find_map(|modifier| match (modifier.kind, modifier.source) {
            (ModifierKind::ReturnToDeck { top }, ModifierSource::Card(card))
                if (top || !p.deck.is_empty())
                    && p.play_area.iter().any(|played| played.card == card) =>
            {
                Some((card, top))
            }
            _ => None,
        })
}

/// Vrai si la figurine du joueur est sur une clairière magique (spec 044).
pub(in crate::engine) fn on_glade(state: &GameState, p: &PlayerState) -> bool {
    p.figure
        .and_then(|hex| state.map.space(hex))
        .map(|space| space.feature)
        == Some(Some(Feature::Site(Site::MagicalGlade)))
}

/// Vrai si la fin du tour peut jeter une Blessure de `glade` (spec 044 ; carte de la clairière) :
/// la figurine sur une clairière magique, et une Blessure à l'endroit dit. Rien à jeter :
/// toujours permis.
pub(in crate::engine) fn glade_allowed(
    state: &GameState,
    p: &PlayerState,
    discard: &[CardId],
    reclaim: Option<CardId>,
    glade: Option<WoundPile>,
) -> bool {
    let Some(pile) = glade else {
        return true;
    };
    on_glade(state, p) && glade_wound_available(&p.hand, &p.discard_pile, discard, reclaim, pile)
}

/// Vrai s'il y a une Blessure à jeter sur la clairière depuis `pile` (spec 044) : dans la main
/// `hand` hors de la défausse de fin de tour `discard` et de la reprise `reclaim`, ou dans la
/// défausse `discard_pile` d'avant la fin du tour (L9). Sert au moteur et à l'écran (C47).
pub fn glade_wound_available(
    hand: &[CardId],
    discard_pile: &[CardId],
    discard: &[CardId],
    reclaim: Option<CardId>,
    pile: WoundPile,
) -> bool {
    let wound = |card: &CardId| card.card().is_wound();
    match pile {
        WoundPile::Hand => {
            let leaving = discard
                .iter()
                .chain(reclaim.as_ref())
                .filter(|c| wound(c))
                .count();
            hand.iter().filter(|c| wound(c)).count() > leaving
        }
        WoundPile::DiscardPile => discard_pile.iter().any(wound),
    }
}

/// Vrai si finir le tour en défaussant `discard` et en reprenant Euphorie cristalline avec
/// `reclaim` respecte les règles de la défausse de fin de tour, pour une main `hand`, une zone de
/// jeu `play_area`, et un héros qui peut (`can_rest`) ou non se reposer. Sert au moteur et à
/// l'écran (spec 034) ; ne dit pas si c'est le moment de finir le tour.
///
/// - Sans Blessure : au moins une carte défaussée si aucune n'a été jouée, sauf main vide
///   (MKUE p. 6, 5.a).
/// - Repos standard : au moins une carte non-Blessure et des Blessures (MKUE p. 6, 7.a).
/// - Repos prolongé : la main n'a que des Blessures, on en défausse une (MKUE p. 6, 7.a).
/// - Le repos n'est permis que si le héros ne s'est ni déplacé ni n'a combattu (MKUE p. 6, 7).
///   Une main de Blessures après un combat sans carte jouée finit le tour sans défausse : les
///   cartes défaussées par Assommé ou Paralysant comptent pour le tour minimum (issue privée 85).
/// - `reclaim` : une carte de la main, une Blessure seulement après Euphorie cristalline
///   renforcée (spec 025 ; spec 034) ; elle ne compte pas pour le repos (L4).
pub fn end_turn_allowed(
    hand: &[CardId],
    play_area: &[PlayedCard],
    played_elsewhere: bool,
    can_rest: bool,
    discard: &[CardId],
    reclaim: Option<CardId>,
) -> bool {
    // Chaque carte nommée une fois, et en main (les Blessures identiques s'échangent).
    let mut remaining = hand.to_vec();
    for card in discard.iter().chain(reclaim.as_ref()) {
        match remaining.iter().position(|held| held == card) {
            Some(index) => {
                remaining.remove(index);
            }
            None => return false,
        }
    }
    if let Some(reclaim) = reclaim {
        let Some(played) = play_area.iter().find(|played| {
            played
                .card
                .card()
                .definition()
                .is_some_and(|definition| definition.reclaim)
                && !matches!(played.play, CardPlay::Sideways(_))
        }) else {
            return false;
        };
        let strong = matches!(
            played.play,
            CardPlay::Strong { .. } | CardPlay::Boosted { .. }
        );
        if reclaim.card().is_wound() && !strong {
            return false;
        }
    }
    let wounds = discard.iter().filter(|card| card.card().is_wound()).count();
    let others = discard.len() - wounds;
    // La main, sans la carte de la reprise.
    let only_wounds = hand
        .iter()
        .filter(|card| Some(**card) != reclaim)
        .all(|card| card.card().is_wound());
    if wounds == 0 {
        // MKUE p. 6, 5.a (issue privée 86) : main vide sans carte jouée, donc vide dès le début du tour
        // (une carte ne quitte la main qu'en étant jouée ; Assommé y laisse des Blessures).
        // Issue privée 85 : une main de Blessures après un combat sans carte jouée vient d'Assommé ou de
        // Paralysant, dont les cartes défaussées comptent pour le tour minimum (ou d'une main vide
        // au début du tour).
        return others > 0
            || !play_area.is_empty()
            // Spec 055 : un artefact détruit ou une bannière assignée sont des cartes jouées
            // (Compendium, *Minimum Turn* S2).
            || played_elsewhere
            || hand.is_empty()
            || (only_wounds && !can_rest);
    }
    if only_wounds {
        return wounds == 1 && can_rest;
    }
    can_rest && others > 0
}

/// La carte de la zone de jeu qui peut être reprise en main : jouée pour son effet, pas de côté
/// (spec 025, L9).
pub(in crate::engine) fn reclaimable(p: &PlayerState) -> Option<CardId> {
    p.play_area
        .iter()
        .find(|played| {
            played
                .card
                .card()
                .definition()
                .is_some_and(|definition| definition.reclaim)
                && !matches!(played.play, CardPlay::Sideways(_))
        })
        .map(|played| played.card)
}
