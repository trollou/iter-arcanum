//! Annonce de la fin de la manche (spec 023) et mode fin du tour (spec 016).

use std::borrow::Cow;

use super::*;

/// Bandeau d'annonce : facultative ou obligatoire (MKUE p. 6, 3), ou sa confirmation.
pub(super) fn announce_prompt(
    view: &PlayerView,
    index: usize,
    confirming: bool,
    catalog: &Catalog,
) -> AnnouncePrompt {
    if confirming {
        return AnnouncePrompt {
            text: catalog.format(
                "round.announce.confirm_prompt",
                &[("count", &view.hand.len().to_string())],
            ),
            buttons: vec![
                Button {
                    label: catalog.text("round.announce.confirm"),
                    command: Command::Play(index),
                    shortcut: None,
                },
                Button {
                    label: catalog.text("round.announce.cancel"),
                    command: Command::CancelChoice,
                    shortcut: shortcut_label(Shortcut::Cancel, catalog),
                },
            ],
        };
    }
    let mandatory = view.hand.is_empty();
    AnnouncePrompt {
        text: catalog.text(if mandatory {
            "round.announce.mandatory"
        } else {
            "round.announce.optional"
        }),
        buttons: vec![Button {
            label: catalog.text("round.announce.button"),
            command: if mandatory {
                Command::Play(index)
            } else {
                Command::OpenAnnounce
            },
            shortcut: None,
        }],
    }
}

/// Bandeau du mode fin du tour : cartes à piocher, et « Finir le tour » actif seulement si
/// l'action correspondante est légale.
pub(super) fn end_turn_prompt(
    view: &PlayerView,
    ending: &EndTurnChoice,
    legal_actions: &[Action],
    catalog: &Catalog,
) -> EndTurnPrompt {
    let forced_withdrawal = view.forced_withdrawal.map(|withdrawal| {
        catalog.format(
            "end_turn.forced_withdrawal",
            &[("count", &withdrawal.wounds.to_string())],
        )
    });
    let steady_tempo = steady_tempo_toggle(view, ending, legal_actions, catalog);
    let view = &*after_withdrawal(view);
    let action = ending.action(view, legal_actions);
    let (discard, reclaim, glade) = match &action {
        Action::EndTurn {
            discard,
            reclaim,
            glade,
            ..
        } => (discard.as_slice(), *reclaim, *glade),
        _ => (&[][..], None, None),
    };
    let discarded = discard.len();
    // MKUE p. 11, 7.b à 7.d : jusqu'à la limite de main, dans la limite du deck. La reprise
    // défausse une carte et en rend une : la main gardée ne change pas (spec 025). La Blessure
    // jetée de la main sur une clairière n'est plus gardée (spec 044, C47).
    let thrown = usize::from(glade == Some(WoundPile::Hand));
    let kept = view.hand.len() - discarded - thrown;
    // Spec 035 : avec la limite du niveau atteint à la fin du tour.
    // Plus le bonus d'une forteresse possédée près de la figurine (spec 043, C29), ou d'une cité
    // conquise, le plus grand des deux (spec 062, C41).
    let limit = view.keep_hand_bonus.max(view.city_hand_bonus)
        + view.pending_level.map_or(view.hand_limit, |level| {
            usize::from(level_token(level).hand_limit)
        });
    // Spec 057 : plus les cartes de Méditation ; aucune avant le tour supplémentaire de
    // Distorsion temporelle (Compendium, *Time Bending* S7).
    let bonus: usize = view
        .modifiers
        .iter()
        .map(|modifier| match modifier.kind {
            ModifierKind::DrawBonus(bonus) => usize::from(bonus),
            _ => 0,
        })
        .sum();
    let extra_turn_next = !view.extra_turn
        && view
            .modifiers
            .iter()
            .any(|modifier| modifier.kind == ModifierKind::TimeBending);
    let count = if extra_turn_next {
        0
    } else {
        (limit + bonus).saturating_sub(kept).min(view.deck_size)
    };
    // Le tour minimum (MKUE p. 6, 5.a) n'est pas rempli : main vide, Assommé ou Paralysant
    // (issues privées 85 et 86) le remplissent sans carte jouée.
    let must_discard = discarded == 0
        && !end_turn_allowed(
            &view.hand,
            &view.play_area,
            view.played_elsewhere,
            view.can_rest,
            &[],
            None,
        );
    // Spec 034 : une Blessure cochée, c'est un repos ; prolongé si la main n'a que des Blessures.
    let text = if discard.iter().any(|card| card.card().is_wound()) {
        let only_wounds = view
            .hand
            .iter()
            .filter(|card| Some(**card) != reclaim)
            .all(|card| card.card().is_wound());
        if only_wounds {
            "end_turn.rest_slow"
        } else {
            "end_turn.rest_standard"
        }
    } else {
        "end_turn.prompt"
    };
    let can_end_turn = legal_actions
        .iter()
        .any(|action| matches!(action, Action::EndTurn { .. }));
    let has_wounds = view.hand.iter().any(|card| card.card().is_wound());
    EndTurnPrompt {
        text: catalog.text(text),
        draw: if count == 0 {
            catalog.text("end_turn.draw_none")
        } else {
            catalog.format("end_turn.draw", &[("count", &count.to_string())])
        },
        must_discard: must_discard.then(|| catalog.text("end_turn.must_discard")),
        confirm: Button {
            label: catalog.text("end_turn.confirm"),
            command: Command::ConfirmEndTurn,
            shortcut: shortcut_label(Shortcut::Confirm, catalog),
        },
        // Les Blessures étant identiques, la défausse cochée est vérifiée par la règle, pas
        // comparée aux actions légales (spec 034).
        confirm_enabled: can_end_turn
            && end_turn_allowed(
                &view.hand,
                &view.play_area,
                view.played_elsewhere,
                view.can_rest,
                discard,
                reclaim,
            ),
        cancel: Button {
            label: catalog.text("end_turn.cancel"),
            command: Command::CancelChoice,
            shortcut: shortcut_label(Shortcut::Cancel, catalog),
        },
        reclaim: reclaim_menu(view, ending, legal_actions, catalog),
        no_rest: (has_wounds && !view.can_rest).then(|| catalog.text("end_turn.no_rest")),
        glade: glade_menu(view, discard, reclaim, glade, catalog),
        forced_withdrawal,
        steady_tempo,
    }
}

/// La vue telle que la laisserait la retraite forcée annoncée, calculée par arcanum-core (spec 052, C21 ;
/// issue privée 217). La défausse, la clairière et la pioche proposées se calculent sur elle, comme les
/// actions légales `EndTurn` du moteur.
pub(super) fn after_withdrawal(view: &PlayerView) -> Cow<'_, PlayerView> {
    Cow::Borrowed(view.after_withdrawal.as_deref().unwrap_or(view))
}

/// Vrai si garder Temporisation est une fin de tour légale (spec 052, C39).
pub(super) fn steady_tempo_offered(legal_actions: &[Action]) -> bool {
    legal_actions.iter().any(|action| {
        matches!(
            action,
            Action::EndTurn {
                steady_tempo: true,
                ..
            }
        )
    })
}

/// La case « Garder Temporisation sous (sur) le deck », cochée d'office, quand garder est légal
/// (Q5).
fn steady_tempo_toggle(
    view: &PlayerView,
    ending: &EndTurnChoice,
    legal_actions: &[Action],
    catalog: &Catalog,
) -> Option<DiscardToggle> {
    if !steady_tempo_offered(legal_actions) {
        return None;
    }
    let top = view
        .modifiers
        .iter()
        .any(|modifier| modifier.kind == ModifierKind::ReturnToDeck { top: true });
    Some(DiscardToggle {
        label: catalog.text(if top {
            "end_turn.steady_tempo_strong"
        } else {
            "end_turn.steady_tempo"
        }),
        checked: ending.steady_tempo.unwrap_or(true),
        command: Command::ToggleSteadyTempo,
    })
}

/// La Blessure jetée sur une clairière (carte de la clairière ; Q5) : le choix fait s'il reste
/// possible, sinon la main s'il y a une Blessure à jeter, sinon la défausse ; rien hors d'une
/// clairière.
pub(super) fn glade_choice(
    view: &PlayerView,
    chosen: Option<Option<WoundPile>>,
    discard: &[CardId],
    reclaim: Option<CardId>,
) -> Option<WoundPile> {
    if !on_glade(view) {
        return None;
    }
    let available = |pile| glade_wound_available(&view.hand, &view.discard, discard, reclaim, pile);
    match chosen {
        Some(None) => None,
        Some(Some(pile)) if available(pile) => Some(pile),
        _ => [WoundPile::Hand, WoundPile::DiscardPile]
            .into_iter()
            .find(|&pile| available(pile)),
    }
}

/// La ligne de la clairière : de la main, de la défausse, ne pas jeter ; grisés sans Blessure à
/// cet endroit, le choix en cours coché.
fn glade_menu(
    view: &PlayerView,
    discard: &[CardId],
    reclaim: Option<CardId>,
    glade: Option<WoundPile>,
    catalog: &Catalog,
) -> Option<GladeMenu> {
    on_glade(view).then(|| GladeMenu {
        label: catalog.text("end_turn.glade"),
        options: [
            (Some(WoundPile::Hand), "end_turn.glade_hand"),
            (Some(WoundPile::DiscardPile), "end_turn.glade_discard"),
            (None, "end_turn.glade_none"),
        ]
        .into_iter()
        .map(|(pile, key)| GladeOption {
            label: catalog.text(key),
            command: Command::ChooseGlade(pile),
            enabled: pile.is_none_or(|pile| {
                glade_wound_available(&view.hand, &view.discard, discard, reclaim, pile)
            }),
            selected: pile == glade,
        })
        .collect(),
    })
}

/// Le menu de reprise, si une fin de tour avec reprise est légale (spec 025).
pub(super) fn reclaim_menu(
    view: &PlayerView,
    ending: &EndTurnChoice,
    legal_actions: &[Action],
    catalog: &Catalog,
) -> Option<ReclaimMenu> {
    let offered = legal_actions.iter().any(|action| {
        matches!(
            action,
            Action::EndTurn {
                reclaim: Some(_),
                ..
            }
        )
    });
    // La carte reprise : jouée pour son effet, pas de côté (spec 025, L9).
    let reclaimed = view.play_area.iter().find(|played| {
        played
            .card
            .card()
            .definition()
            .is_some_and(|definition| definition.reclaim)
            && !matches!(played.play, CardPlay::Sideways(_))
    })?;
    if !offered {
        return None;
    }
    let none = catalog.text("end_turn.reclaim_none");
    let selected = ending
        .reclaim
        .filter(|card| view.hand.contains(card))
        .map_or_else(|| none.clone(), |card| card_name(card, catalog));
    let mut options = vec![Button {
        label: none,
        command: Command::ChooseReclaim(None),
        shortcut: None,
    }];
    options.extend(view.hand.iter().map(|&card| Button {
        label: card_name(card, catalog),
        command: Command::ChooseReclaim(Some(card)),
        shortcut: None,
    }));
    Some(ReclaimMenu {
        label: catalog.format(
            "end_turn.reclaim",
            &[("card", &card_name(reclaimed.card, catalog))],
        ),
        selected,
        options,
    })
}
