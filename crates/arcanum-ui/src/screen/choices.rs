//! Fenêtres de choix (spec #20) : l'action avancée et l'unité d'une récompense (spec 045), la carte
//! d'un effet « gagner » (spec 056, spec 055) ; seulement les cartes à prendre maintenant.

use arcanum_core::{CommonCard, Offer};

use super::*;

/// Vrai pendant le choix de l'action avancée d'une récompense : une carte attend et aucun niveau
/// pair n'est à servir, la récompense passant avant les niveaux (spec 045, C24, C41).
fn choosing_reward_advanced_action(view: &PlayerView) -> bool {
    view.advanced_action_choices > 0
        && view.drawn_artifacts.is_empty()
        && view.spell_choices == 0
        && view.skill_choice.is_empty()
        && super::offers::choosing_level(view).is_none()
}

/// « Récompense : prenez une action avancée » : les cartes de l'offre d'actions avancées, chacune
/// avec « Prendre », sans fermeture (C11).
pub(super) fn advanced_action_reward(
    view: &PlayerView,
    legal_actions: &[Action],
    catalog: &Catalog,
) -> Option<CardChoiceWindow> {
    choosing_reward_advanced_action(view).then(|| CardChoiceWindow {
        title: catalog.text("reward.choose_advanced_action"),
        cards: offer_of(view, Offer::AdvancedActions)
            .iter()
            .filter(|card| matches!(card, CommonCard::Deed(_)))
            .map(|&card| super::offers::offer_card(card, legal_actions, catalog))
            .collect(),
        close: None,
    })
}

/// « Récompense : prenez une unité de l'offre » : les unités de l'offre seulement, pas une action
/// avancée de monastère (C12, C13) ; « Renoncer » sans jeton de commandement libre.
pub(super) fn unit_reward(
    view: &PlayerView,
    legal_actions: &[Action],
    catalog: &Catalog,
) -> Option<CardChoiceWindow> {
    let choosing = view.unit_choices > 0
        && view.drawn_artifacts.is_empty()
        && view.spell_choices == 0
        && view.advanced_action_choices == 0
        && view.skill_choice.is_empty();
    choosing.then(|| CardChoiceWindow {
        title: catalog.text("reward.choose_unit"),
        cards: offer_of(view, Offer::Units)
            .iter()
            .filter_map(|card| match card {
                CommonCard::Unit(unit) => Some(*unit),
                CommonCard::Deed(_) => None,
            })
            .map(|unit| OfferCardView {
                take: super::offers::unit_reward_take(unit, legal_actions, catalog),
                ..super::offers::unit_card(unit, catalog)
            })
            .collect(),
        close: legal_actions
            .iter()
            .position(|action| *action == Action::DeclineUnitReward)
            .map(|index| Button {
                label: catalog.text("reward.decline_unit"),
                command: Command::Play(index),
                shortcut: None,
            }),
    })
}

/// La carte d'un effet « gagner » : seulement les cartes prenables des offres, « Prendre » sur
/// chacune, « Annuler » (C14 à C16, Q3, L3).
pub(super) fn gain_choice(
    view: &PlayerView,
    legal_actions: &[Action],
    mode: &ScreenMode,
    catalog: &Catalog,
) -> Option<CardChoiceWindow> {
    let ScreenMode::ChoosingGain(choice) = mode else {
        return None;
    };
    if !super::gain::choosing_offer_card(mode, legal_actions) {
        return None;
    }
    let cards = [Offer::AdvancedActions, Offer::Spells, Offer::Units]
        .into_iter()
        .flat_map(|offer| offer_of(view, offer).iter().copied())
        .filter_map(|card| {
            let take = super::gain::gain_take(card, view, legal_actions, mode, catalog)?;
            let CommonCard::Deed(deed) = card else {
                return None;
            };
            Some(OfferCardView {
                take: Some(take),
                ..super::hand::window_card(deed, catalog)
            })
        })
        .collect();
    Some(CardChoiceWindow {
        title: catalog.format(
            "gain_choice.title",
            &[("card", &card_name(choice.card, catalog))],
        ),
        cards,
        close: Some(Button {
            label: catalog.text("discard_choice.cancel"),
            command: Command::CancelChoice,
            shortcut: shortcut_label(Shortcut::Cancel, catalog),
        }),
    })
}

fn offer_of(view: &PlayerView, offer: Offer) -> &[CommonCard] {
    view.offers.get(&offer).map_or(&[][..], Vec::as_slice)
}
